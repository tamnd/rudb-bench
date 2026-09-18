# The full file under the official ClickBench driver

This is the same fork of the official driver as yesterday, pointed at the real dataset rather than a sample: 99,997,497 rows, the 14,779,976,446 byte `hits.parquet`, on `gamingpc-wsl`. DuckDB and ClickHouse both finished. rudb could not load the file at all, which is the finding, and the rest of the numbers are what they are measured against. The two result files the driver wrote are in [full-file](full-file) beside this one in the driver's own `query,try,seconds` format.

## What ran

| engine | version | result |
| --- | --- | --- |
| rudb | 0.3.31 | load failed after 6m23s, no queries ran |
| duckdb | v2.0.0-dev84237 (Development Version) cc7e7bac7f | forty three queries, exit zero |
| clickhouse | 26.9.1.1162 | forty three queries, exit zero |

ClickHouse ran against the same private server instance on port 9001 as yesterday, for the same reason: this machine's system daemon holds somebody else's loaded `hits` table and the packaged entry starts with `CREATE OR REPLACE TABLE hits`. Same binary, same MergeTree, same settings, different data path and port. The concurrency test was off, so this is the forty three queries and nothing else.

## The two engines that finished

| engine | load | size on disk | cold total | hot total |
| --- | --- | --- | --- | --- |
| duckdb | 40.834s | 20,462,972,928 | 16.14s | 14.28s |
| clickhouse | 86.995s | 9,577,085,269 | 12.87s | 9.63s |

Hot is the sum over the forty three queries of the best of three tries. Cold is the sum of the first try after the driver dropped the page cache, and the section below says why that column is not usable here.

## Against what the board publishes

The point of running the official driver is that its output can be put next to somebody else's. These are the current `c6a.4xlarge` entries in the fork, DuckDB from 11 May 2026 and ClickHouse from 16 September 2026, against what the same driver produced here.

| | duckdb board | duckdb here | clickhouse board | clickhouse here |
| --- | --- | --- | --- | --- |
| load | 126s | 40.834s | 220s | 86.995s |
| size | 20,457,467,904 | 20,462,972,928 | 9,420,233,268 | 9,577,085,269 |
| cold | 115.19s | 16.14s | 110.20s | 12.87s |
| hot | 26.25s | 14.28s | 18.55s | 9.63s |

The size column is the check worth having. DuckDB's database is within 0.03% of the published one and ClickHouse's is within 1.7%, which is about as close as two runs of the same loader on the same data get, and it means the load did the same work rather than something cheaper that happens to answer the queries.

The hot column then says the machine and not much else. Two engines that share no code land at 1.84x and 1.93x the published number, and an i9-13900K with 32 threads against sixteen vCPUs of a c6a.4xlarge is about that. Two independent engines agreeing on the factor to within five percent is the reason to trust the factor.

The cold column is not a cold number and should not be read as one. It is seven to nine times faster than the published cold column, which is not a plausible thing for hardware to do when the hot numbers moved by 1.9x. What happens is that `drop_caches` inside WSL2 empties the Linux page cache and has no effect on the Windows host's cache of the virtual disk underneath it, so the first try still reads out of memory, just somebody else's memory. What is left in the column after that is the cost of the syscalls and of the guest crossing into the host, which is real and is not disk. So it is a floor rather than a cold number: an engine that looks bad in it would look worse against a real disk, and an engine that looks fine in it has been told nothing about. It is kept in the raw files because runs are kept as they came out, and it is flagged here because a number that looks like a cold measurement and is not is worse than a missing one.

## What rudb did

The load ran for six minutes and twenty three seconds, wrote 42,218,416,686 bytes, and then failed with `Invalid Input Error: invalid rudb native file: directory exceeds the configured bound`. No query ran, so there is no rudb column anywhere above.

The cause is structural rather than a bad constant. A stripe is one appended chunk, the directory is a single contiguous buffer whose size grows linearly in the number of stripes, and each stripe writes twenty bytes a column plus twenty more for each varchar column's code membership index plus a zone entry a column, and a varchar zone bound stores the whole string rather than a prefix. A hundred million rows of a hundred and five column table with long URLs in it puts that buffer over the fixed 128 MB `MAX_DIRECTORY` cap. It is filed as [rudb#745](https://github.com/tamnd/rudb/issues/745) with three ways out: cut stripes at their own row count so the stripe count stops tracking the append pattern, truncate varchar zone bounds to a prefix, or page the directory so it never has to be one buffer.

Until that is fixed rudb has no ClickBench number. Not a slow one, not a partial one, none, because the benchmark is defined over this file and rudb cannot open it. Everything measured on a sample is a development number whatever it prints.

## The projection from yesterday's ladder was wrong by two times

Yesterday's four size ladder fitted a fixed cost a query and a cost a row for each engine and pushed the two constants out to the full file. It predicted about 29s for DuckDB and about 23s for ClickHouse on this machine. The measurements are 14.28s and 9.63s, so the projection was 2.0x high for one and 2.4x high for the other.

The reason is that the slope between a hundred thousand rows and a million is measured in a regime where neither engine is using the machine. Per query work there is under a millisecond and no sensible engine spreads that over 32 threads, so the per row constant that comes out of a sample ladder is close to a single threaded constant. At a hundred million rows both engines use the whole box. A per row cost measured where parallelism is off cannot project a run where parallelism is on, and the size of the error is roughly the width of the machine, which is what the 2.0x and 2.4x are.

The weak check yesterday's report leaned on was that the projected 29s landed near the board's published 26.25s. That agreement was two errors cancelling: the projection was twice too high and the board's machine is about half as fast. Stating it as a sanity check was a mistake and this paragraph is the correction. What survives from that ladder is the fixed cost per query, which is measured at the size where it dominates and is not affected by any of this: 0.39ms a query for rudb against 1.91ms for ClickHouse and 2.86ms for DuckDB. What does not survive is anything projected off the slope.

This matters most for rudb. At a million rows rudb was using about one core, so of the three its measured slope is the one most likely to be a single threaded slope, and there is no way to tell from a sample whether that is because a million rows is too small to be worth splitting or because rudb does not split it. The full file is what answers that question and rudb cannot run the full file, so #745 is not a storage bug that can wait behind kernel work. It is the thing standing between us and the only measurement the goal is stated in.

## The two engines against each other

ClickHouse finishes the suite in 0.67x DuckDB's hot time and wins forty one of the forty three queries. DuckDB's two are q9, the `COUNT(DISTINCT UserID)` grouped by RegionID, at 0.201s against 0.276s, and q33, the group by WatchID and ClientIP, at 1.213s against 1.228s.

The largest ratios are on the cheapest queries and are mostly about metadata rather than about scanning. q2, `COUNT(*) WHERE AdvEngineID <> 0`, is 26x. q20, the point lookup on UserID, is 20x. q1, the bare `COUNT(*)`, is 9x. All three are hundredths of a second for DuckDB and thousandths for ClickHouse, and they are worth about 0.07s of the 4.6s gap between the two engines put together.

The ratios that pay are q22 at 6.7x, the `URL LIKE '%google%'` group by SearchPhrase, 0.470s against 0.070s, and q28 at 5.8x, the average URL length grouped by CounterID with a HAVING, 0.444s against 0.077s. Both are string work over the widest columns in the table.

The expensive end of the suite is the same handful for both. DuckDB spends 2.001s on q29, 1.319s on q35, 1.268s on q19, 1.265s on q34 and 1.213s on q33. ClickHouse spends 1.228s on q33, 1.135s on q19, 1.052s on q35, 1.042s on q34 and 0.788s on q29. Those five are 49% of DuckDB's total and 54% of ClickHouse's. They are also, allowing for q29, the same queries that were rudb's worst at a million rows yesterday, which says the concentration is a property of the suite rather than of any one engine, and that a kernel that fixes the high cardinality group by fixes most of what there is to fix for everybody.

## What this does not say

The cold column is not cold, for the reason above. This is one machine and rule seven still holds, so nothing here may be read against a number from server1, server2 or server3. The board comparison is across machines by construction and is used here only for shape and for the size check, never as a ranking. There is no concurrency result because the concurrency test was switched off.

## Reproducing it

From a checkout of the fork with the dataset already in place, for each of the three system directories in turn:

```
export BENCH_SKIP_DOWNLOAD=yes
export BENCH_CONCURRENT_DURATION=0
cd duckdb && cp ~/rudb-data/hits.parquet hits.parquet && ./benchmark.sh
```

The ClickHouse entry has to be pointed at a server that nobody else is using before this is safe to run on a shared machine.
