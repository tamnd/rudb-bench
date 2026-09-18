# rudb on the full ClickBench file, for the first time

Yesterday's run of the official driver on the real dataset had two engines in it because rudb could not load the file. [rudb#752](https://github.com/tamnd/rudb/pull/752) fixed that, and this is the same driver, the same fork, the same 99,997,497 rows out of the same 14,779,976,446 byte `hits.parquet`, on the same `gamingpc-wsl`, with rudb in the table. The driver's own `query,try,seconds` output is in [full-file](full-file) beside this, `rudb-full.csv` next to the DuckDB and ClickHouse files from yesterday's run.

## Three engines, forty three queries

| engine | version | load | size on disk | cold total | hot total | answered |
| --- | --- | --- | --- | --- | --- | --- |
| rudb | 0.3.33 | 324.297s | 43,015,574,093 | 637.90s | 27.07s | 42 of 43 |
| duckdb | v2.0.0-dev84237 cc7e7bac7f | 40.834s | 20,462,972,928 | 15.87s | 14.03s | 43 |
| clickhouse | 26.9.1.1162 | 86.995s | 9,577,085,269 | 12.63s | 9.43s | 43 |

The totals are over the forty two queries all three engines answered, so DuckDB and ClickHouse are a little lower here than in yesterday's report, which summed all forty three. Hot is the best of three tries. Cold is the first try after the driver dropped the page cache, and unlike every previous run on this machine it is a real number, which the section below is about.

The one rudb could not answer is query 6, `SELECT COUNT(DISTINCT SearchPhrase) FROM hits`, which failed on all three tries with `Out of Memory Error: could not allocate 23.8 MiB (25.0 GiB/25.0 GiB used)`. DuckDB answers it in 0.277s. That is [rudb#755](https://github.com/tamnd/rudb/issues/755) and it is a single path rather than anything structural: `COUNT(DISTINCT x)` with no `GROUP BY` builds a hash table keyed by the whole varchar for every row, when the column is dictionary encoded on disk and the directory already knows the code count.

## What the load and the size say

The predictions in yesterday's ten million row report were 330s and 43.5 GB, extrapolated linearly from ten million rows and from the size ratio between one and ten million. The measurements are 324.297s and 43.0 GB. Nothing bends between ten million and a hundred million rows, in either direction, which is worth knowing on its own: whatever is wrong with the load and the size is wrong at every scale and is not a large file effect.

Load is 7.9x DuckDB and 3.7x ClickHouse. Size is 2.10x DuckDB and 4.49x ClickHouse. The size is the same 2x that was there at ten million rows, and it is compression rather than layout: the native format stores a dictionary per column and delta encodes what it can, and DuckDB's file grew 4.7x for ten times the rows because its compression improves with scale while rudb's grew 8.8x because it barely does.

Five and a half minutes of load is the part that will be felt first by anybody trying this, and it is a hundred and five columns of a hundred million rows through one insert, so there is a lot of room in it.

## The cold column, which is finally a cold column

Every previous run on this machine had to put a warning next to the cold column, because `drop_caches` inside WSL2 empties the guest page cache and the Windows host still holds the virtual disk blocks, so the first try reads out of somebody else's memory. At a hundred million rows that stops being true for rudb. The database is 43 GB and the machine has 31 GB, so the host cannot hold it, and what the first try measures is a disk.

| engine | cold | hot | cold over hot |
| --- | --- | --- | --- |
| rudb | 637.90s | 27.07s | 23.6x |
| duckdb | 15.87s | 14.03s | 1.13x |
| clickhouse | 12.63s | 9.43s | 1.34x |

This is the same 18x that [rudb#748](https://github.com/tamnd/rudb/issues/748) opened on at ten million rows, now at 23.6x with the host cache out of the way, and it is 40x DuckDB's cold total. It is also the single largest number in this report by a wide margin. A hot total of 1.93x DuckDB is a gap that kernel work closes. A cold total of 40x is a different kind of problem and it will not be closed by making anything faster.

The per query shape is the same as at ten million. The worst are the wide string scans: q32 at 67.4x cold over hot, q10 at 41.2x, q23 at 39.8x with 47.089s cold against 1.184s hot, q19 at 33.7x. The median across the forty two is 28.6x. Cold cost tracks bytes touched, which is what it should do, and the constant in front of it is more than an order of magnitude too large.

#752 ruled out the explanation that was on the table. It cut the directory by sixty four and made each stripe write one page per column, which is what #748 guessed was the fix, and the ten million row cold total moved from 62.772s to 61.817s. The extent mechanism it replaced was already grouping thirty two chunks and already laying each column out contiguously, so the change was a factor of two on something that was never the bottleneck.

What is left to check, in the order worth checking it: whether the read count is actually high, which `strace -c -e trace=pread64` on one cold query settles in five minutes and which nobody has run yet; whether the first touch of a varchar column decodes the whole dictionary page regardless of how much of the column the query wants, which would be per byte, would scale with the file, and would be invisible warm because the dictionary stays decoded; and whether the checksum verification on first read is being done over whole pages when only part of the page was asked for.

## The hot column, which is the one the kernel work inherits

27.07s against DuckDB's 14.03s is 1.93x, and against ClickHouse's 9.43s is 2.87x. That is the base number the 10x goal is measured from and it is the first time it exists at full scale.

It is not spread evenly. Five queries are faster than both DuckDB and ClickHouse, and four of them are the same shape:

| query | rudb | duckdb | clickhouse |
| --- | --- | --- | --- |
| q16 `GROUP BY UserID ORDER BY COUNT(*) DESC LIMIT 10` | 0.0005s | 0.189s | 0.138s |
| q34 `GROUP BY URL ORDER BY c DESC LIMIT 10` | 0.0615s | 1.265s | 1.042s |
| q35 `GROUP BY 1, URL ORDER BY c DESC LIMIT 10` | 0.0566s | 1.319s | 1.052s |
| q36 `GROUP BY ClientIP, ClientIP - 1, ... LIMIT 10` | 0.0006s | 0.157s | 0.116s |
| q13 `SearchPhrase <> '' GROUP BY SearchPhrase LIMIT 10` | 0.207s | 0.251s | 0.214s |

The first four are answered from the per column frequency page the writer stores, not from a scan, which is why they are three orders of magnitude off the other two engines rather than two times. That is a real capability and it is also part of what the 324s load is paying for, so it belongs in the load column as much as in the query column. It is worth saying plainly that the driver does not check answers, so what makes those numbers trustworthy is the compat corpus and not this run.

The other end is where the work is:

| query | rudb | duckdb | rudb over duckdb |
| --- | --- | --- | --- |
| q24 `SELECT * FROM hits WHERE URL LIKE '%google%' ORDER BY EventTime LIMIT 10` | 1.805s | 0.109s | 16.6x |
| q40 the big `CASE WHEN` group over five columns | 0.417s | 0.051s | 8.2x |
| q39 `GROUP BY URL` under five predicates | 0.194s | 0.024s | 8.1x |
| q7 `SELECT MIN(EventDate), MAX(EventDate)` | 0.123s | 0.016s | 7.7x |
| q21 `SELECT COUNT(*) FROM hits WHERE URL LIKE '%google%'` | 2.327s | 0.324s | 7.2x |

q7 is the odd one and probably the easiest. A minimum and a maximum of a column with a zone map per part should be answered from the zone maps without reading anything, and 0.123s says it is reading.

The `LIKE '%google%'` family is the largest single block of time in the hot column. q21, q22 and q24 together are 6.2s of the 27.07s and all three are the same substring scan over URL. A substring prefilter in `rudb-regex`, which is already on the list, is worth about a fifth of the hot total on its own.

q29, the `REGEXP_REPLACE` over Referer, is 3.846s and is the largest query in the run for every engine, 14.2% of rudb's hot total and 14.3% of DuckDB's. We are 1.92x there, which is the same as the overall number, so it is not where the gap is even though it is where the time is.

## What this changes

E0 said rudb runs the suite end to end. Until today that was true up to somewhere between ten million and a hundred million rows and the full file run had to leave rudb out. It is now true on the file the benchmark is actually about, with one query out and filed, and there are three base numbers to improve on: 324.297s of load, 27.07s hot, and 637.90s cold.

The order they deserve attention in is not the order of their sizes. Hot at 1.93x is the number the goal is written against. Cold at 23.6x is the number a user would actually meet first, since a fresh database is cold by definition, and it is the one where we do not yet know what is wrong.
