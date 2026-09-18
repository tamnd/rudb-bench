# The cold column was eight workers reading the same page

[The full file report](the-full-file.md) left one number a lot larger than everything else in it. rudb answered the forty two queries it could answer in 27.07s warm and 637.90s cold, a ratio of 23.6x where DuckDB pays 1.13x and ClickHouse pays 1.34x, and 40x DuckDB's cold total. [rudb#748](https://github.com/tamnd/rudb/issues/748) had been open on that ratio since the ten million row run and its own guess about the cause had already been measured and disproven. This is what it actually was.

## Counting the reads

#748 asked for one experiment before anybody acted on it, which was to count the reads under `strace` on one cold query and settle whether the problem is read count or something else. That experiment takes five minutes and it answers the question in one line.

`SELECT MIN(EventDate), MAX(EventDate) FROM hits` over the 99,997,497 row database, cold, under `strace -f -c -e trace=pread64`:

| | |
| --- | --- |
| calls | 25,536 |
| bytes | 3,199,084,135 |
| size of the column on disk | about 400 MB |
| stripes in the table | 1,526 |

Half the calls are 776 bytes, which is an index page, and half are 262,272 bytes, which is a stripe page. There are 1,526 stripes and there were 12,762 page reads, so every page was read 8.36 times, and the query moved eight times the bytes it needed to look at.

8.36 is not a coincidence and it is not a cache that is too small. It is the number of scan instances the pipeline gave that query.

## Why a scan reads everything once per worker

The reader keeps four stripes per column, and the comment above the code that filled that cache said the file is never read under the lock, so two workers that want the same page at the same time can both read it and the second one to finish finds the first one's copy and drops its own, which costs one duplicated read and never costs a worker a wait.

The first half of that is true. The second half is true per worker and wrong for the scan. A scan hands parts out one at a time and in order, so the workers on a column are never more than a few parts apart, and a stripe is sixty four parts. Every worker crosses into a new stripe within a few parts of every other worker, they all look in the cache at about the same moment, they all miss, and they all read. It is not one duplicated read somewhere. It is every page of every column read once per worker, for the whole scan, and #753 had just taken the worker count on a hundred million rows from sixteen to thirty two.

## The fix, and the one that was thrown away

[rudb#773](https://github.com/tamnd/rudb/pull/773) makes a worker that finds the page already being read do neither of the two obvious things. It does not wait for it and it does not read it again. It comes back with the index alone, which sends the caller down the path that reads the one part it came for, a few kilobytes against a quarter of a megabyte, and it picks the page up out of the cache on its next part.

Waiting is the other way to write this and it is worse. A `Condvar` version of the same fix was written first and measured, and it took the hot total from 27.07s to 32.99s. The pages that matter are the wide string ones, they take milliseconds to copy even warm, and a lock that makes seven workers stop for all of that on every stripe boundary costs more than the read it saves. That version is not in the tree and the only reason to record it is that it looks like the obvious fix and it is not.

## What it moved

Both binaries are built at caa5ef6 so that #753 is on both sides, both run against the same 43,015,574,093 byte `hits.db` on `gamingpc-wsl`, three tries per query with `drop_caches` before the first, which is the same protocol the official driver uses. The per query output is in [stripe-page-sharing](stripe-page-sharing) as `rudb-caa5ef6.csv` and `rudb-single-flight.csv`.

| | cold total | hot total |
| --- | --- | --- |
| caa5ef6 | 629.85s | 27.70s |
| with page sharing | 88.15s | 26.94s |
| | 7.15x | 1.03x |

Twenty three of the forty three queries are at least five times faster cold and eight of them are at least ten times faster. The largest are q19 at 11.86x, q25 at 11.05x, q32 at 10.88x, q23 at 10.63x where 47.367s becomes 4.455s, q12 at 10.35x, q10 at 10.17x and q14 at 10.06x. The queries that were already flat, the ones answered out of the directory without reading anything, do not move, which is the control this needed.

Against the other two engines on the same file, where DuckDB is 16.14s cold and ClickHouse is 12.86s over the same forty three queries in yesterday's run, cold goes from 39.0x DuckDB to 5.5x and from 49.0x ClickHouse to 6.9x. Both of those engine totals are over all forty three, which rudb can now be compared against directly because it answers all forty three, which the section at the end is about.

## The warm column is not flat underneath

The warm total barely moves, 27.70s to 26.94s, and that hides two things pulling against each other.

Twenty nine queries are at least ten percent faster warm, up to 3.73x on q4, because a page that is read once is also a page that is copied once and checksummed once, and the guest page cache was serving the duplicates but it was not serving them free.

Five are slower, and they are all the same shape:

| query | caa5ef6 hot | with page sharing | |
| --- | --- | --- | --- |
| q24 `SELECT * FROM hits WHERE URL LIKE '%google%' ORDER BY EventTime LIMIT 10` | 1.825s | 2.482s | 0.74x |
| q22 `SearchPhrase, MIN(URL), COUNT(*)` under `URL LIKE '%google%'` | 2.067s | 2.555s | 0.81x |
| q23 the same group under `Title LIKE '%Google%'` with a `COUNT(DISTINCT UserID)` | 1.211s | 1.501s | 0.81x |
| q28 `GROUP BY CounterID` with `AVG(STRLEN(URL))` over `URL <> ''` | 2.218s | 2.623s | 0.85x |
| q21 `SELECT COUNT(*) FROM hits WHERE URL LIKE '%google%'` | 2.312s | 2.673s | 0.86x |

These are the wide string scans, which are exactly the queries where the loser of the race goes on to read most of the stripe part by part anyway. Sixty four part reads of a few kilobytes cost more than one page read of a quarter of a megabyte when you were going to want all sixty four. The trade is right on the totals and it is the wrong trade for a worker that already knows it wants the whole stripe, which is a thing the reader could be told and currently is not.

## What is left

Cold at 88.15s against DuckDB's 16.14s is 5.5x, which is no longer the largest number in the report and is still the largest ratio in it. The warm gap is 26.94s against 14.30s, which is 1.88x. So the order of attention has gone back to where the goal is written, with one caveat worth keeping: a fresh database is cold by definition, and a user meets the cold column first.

The remaining cold cost is now roughly one read per page per column per query, which is the floor for a format that stores a page per column per stripe. Going below it means not reading pages that cannot contain an answer, which is [rudb#754](https://github.com/tamnd/rudb/issues/754), per part zone bounds in the index page.

One correction to the full file report belongs here rather than there, because it is about that run and not about this change. It recorded q6, `SELECT COUNT(DISTINCT SearchPhrase) FROM hits`, as failing on all three tries with an out of memory error. It does not fail. Built at the branch head that run used, at the commit it merged to, and at caa5ef6, the query answers in about 0.02s with 6,019,103, which is the value DuckDB gives for the same column of the same parquet file. The failure recorded in that run does not reproduce at any of the three and there is no code change between them that touches the path, so what is in that report is a measurement nobody has been able to repeat. It is filed that way on [rudb#755](https://github.com/tamnd/rudb/issues/755).
