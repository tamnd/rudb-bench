# The URL queries were reading a gigabyte of dictionary in sixty four kilobyte pieces

[The full file report](the-full-file.md) said the hot column was 1.93x DuckDB and that the largest single block of time in it was the `LIKE '%google%'` family over URL. It also said what the obvious fix was, which was a substring prefilter in the matcher. The obvious fix would have bought nothing. The time was not in the matcher and this is where it was.

## The predicate is not the cost

Three queries over the same column, hot, on the 99,997,497 row database:

| statement | seconds |
| --- | --- |
| `SELECT COUNT(*) FROM hits WHERE URL <> ''` | 0.21 |
| `SELECT COUNT(*) FROM hits WHERE URL LIKE 'http://zzz%'` | 2.7 |
| `SELECT COUNT(*) FROM hits WHERE URL LIKE '%google%'` | 2.7 |
| `SELECT SUM(STRLEN(URL)) FROM hits` | 2.7 |

A prefix `LIKE` and a substring `LIKE` are not the same amount of work by any measure, and `STRLEN` is not a match at all, so the fact that all three take the same 2.7s says the cost is in front of the predicate rather than in it. The one that is fast is the one that can answer from the dictionary code without ever looking at the bytes. `like_stable` in `rudb-kernels` already memoizes per dictionary code in a `Vec<AtomicU8>`, so the matcher runs once per distinct string and not once per row, which is the other half of the same observation.

`perf` is not installed on this machine but `gdb` is, so the profile is a poor man's one: `sudo gdb -p <pid> -batch -ex "thread apply all bt 3"` in a loop while the query runs, counting only frame zero. Ninety six leaf samples over query 21:

| where | samples |
| --- | --- |
| in a syscall | 56 |
| in `mprotect` | 17 |
| parked on a futex | 19 |
| anywhere else | 4 |

Every one of them is under `NativeText::payload_block`, which is the function that fetches a piece of the global string dictionary.

## Why that function is called twenty thousand times

A varchar column in the native format has one dictionary for the whole table, and the reader takes it in sixty four kilobyte blocks because sixty four kilobytes is what one checksum in the dictionary page covers. The URL dictionary in the full file is over a gigabyte. Each block was its own `pread`, its own `Vec` and its own `OnceLock`, so a scan that wants the whole dictionary, and it does want the whole dictionary because the codes in one part point anywhere in it, pays twenty thousand of each.

The three rows of the profile are the three costs of that, and none of them is the read itself being slow. Fifty six samples in a syscall is the syscall count. Seventeen in `mprotect` is the allocator asking the kernel for another sixty four kilobytes twenty thousand times, which is the size at which it stops reusing the heap and starts going back for more. Nineteen on a futex is thirty two workers arriving at the same block at the same moment, which is the same crowding [the stripe page report](stripe-page-sharing.md) found one layer up, except that here the losers do have to wait, because a string that is half decoded is not a string.

## What changed

[rudb#784](https://github.com/tamnd/rudb/pull/784) groups eight blocks into one extent and reads, allocates and waits on the extent. Half a megabyte is one `pread`, one allocation that the allocator takes straight from `mmap` rather than growing the heap, and one wait.

Not a byte of the format moved. The block size is what the committed checksum list is indexed by, so raising it was tried first and it made every query return nothing, because a file written at sixty four kilobytes per checksum cannot be read at one megabyte per checksum. The extent is a reader idea only, and after the single read it verifies all eight checksums separately, which is the case the new test damages a file to prove.

Eight was measured. Against query 21 and query 34, four blocks give 1.318s and 0.075s, eight give 1.168s and 0.084s, and sixteen give 1.205s and 0.116s.

## What it moved

Both binaries were built from the branch and from its base at main 1a905e9, so nothing else is in the difference. Full file, 43,016,017,995 bytes of native database, `drop_caches` before each query, three tries, hot is the better of the second and the third. The driver's `query,try,seconds` output for both sides is in [dictionary-extents](dictionary-extents) beside this file.

| | main 1a905e9 | with extents | |
| --- | --- | --- | --- |
| hot total, 43 queries | 26.96s | 20.94s | 1.29x |
| cold total, 43 queries | 87.75s | 83.18s | 1.05x |

The whole of it is five queries:

| query | hot before | hot after | |
| --- | --- | --- | --- |
| q21 `COUNT(*) WHERE URL LIKE '%google%'` | 2.691s | 1.155s | 2.33x |
| q22 the same with a `SearchPhrase` group | 2.525s | 1.132s | 2.23x |
| q28 `AVG(STRLEN(URL))` grouped by `CounterID` | 2.653s | 1.223s | 2.17x |
| q24 `SELECT *` under the `%google%` filter | 2.505s | 1.273s | 1.97x |
| q23 `Title LIKE '%Google%'` and a `NOT LIKE` on URL | 1.498s | 1.006s | 1.49x |
| the five together | 11.87s | 5.79s | 2.05x |
| the other thirty eight together | 15.09s | 15.15s | 1.00x |

That last row is the interesting one. The other thirty eight queries do not move at all, in either direction, which is what a change that only touches the path into a large dictionary should look like. Sixteen queries are more than two percent faster and fourteen are more than two percent slower and the sum of all of them outside the five is six hundredths of a second, which is noise.

Two of the fourteen are not noise. q34 and q35 go from 0.062s to 0.089s and from 0.061s to 0.092s. Both are answered out of the per column frequency page rather than from a scan, so they touch the dictionary only to print ten strings, and an extent reads half a megabyte to hand back one block of it. Twenty seven milliseconds against six seconds is a trade worth making, and the answer for those two is a point read that does not go through the extent cache at all rather than a smaller extent for everybody.

## Where the hot column stands

| | rudb before | rudb after | duckdb | clickhouse |
| --- | --- | --- | --- | --- |
| hot total, 43 queries | 26.96s | 20.94s | 14.30s | 9.64s |
| cold total, 43 queries | 87.75s | 83.18s | 16.14s | 12.86s |

Rule seven still applies and the DuckDB and ClickHouse columns are yesterday's run on this same machine rather than today's, so they are context and not an A/B. Read that way, the hot gap goes from 1.88x to 1.46x against DuckDB and from 2.80x to 2.17x against ClickHouse, and query 6 is in all four columns now because [rudb#755](https://github.com/tamnd/rudb/issues/755) turned out not to reproduce.

The cold gap barely moves, which is expected. Cold is dominated by first touch of the column pages, the dictionary is one part of what a cold query reads, and [rudb#773](https://github.com/tamnd/rudb/pull/773) already took the large factor out of it earlier the same day. 5.15x cold against DuckDB is now the number that stands out, and it is a different problem from this one.

## What is left in the same place

The dictionary is still read in full by any query that touches a string column, however few distinct values that query ends up caring about. A scan with a selective filter on another column reads every block of a gigabyte dictionary to decode the rows that survive, and nothing in the file lets it know which blocks it needs. A per part range of dictionary codes in the stripe index would let a scan read the blocks its part actually spans, which is the same shape as [rudb#754](https://github.com/tamnd/rudb/issues/754) for zone bounds and is worth the same kind of look.
