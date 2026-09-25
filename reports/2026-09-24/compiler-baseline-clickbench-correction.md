# Correction to the ClickBench compiler baseline

The query time ratios in [compiler-baseline-clickbench.md](compiler-baseline-clickbench.md) (PR #236) are withdrawn. That means the query time rows of the summary table, the rudb/duckdb column of the per query table, and every sentence built on them, including "closer to a tenth". The instruction counts and the frontend timings stay, because load and the problems below do not move them much. Do not quote the time ratios from that report anywhere.

## Why

The run was not a fair comparison, for seven reasons.

1. server3 was saturated for the whole run, with a load average between 22 and 65 on 8 threads, and the engines got very different shares of it. DuckDB ran first at a load of 43 and rudb ran last at 22, so part of the gap was the machine and not the engines.
2. ClickHouse ran as `clickhouse local`, a new process for every query, over a table created with `ORDER BY tuple()`. The upstream ClickBench entry runs a server over a table with the real primary key and a config that loads the primary key eagerly. What we timed was process startup and a table with no key.
3. The ClickHouse memory cap was left dynamic, at 0.9 of whatever memory was free when it started. That is what ran q29 out of memory at 4.57 GiB. DuckDB and rudb had no matching cap, so each engine had a different memory budget.
4. The cold run never dropped the page cache, so it was not cold, and the report used the median of five hot runs. Upstream runs three tries, drops the page cache before the first, and takes the best of the second and third.
5. DuckDB was a development build, and the database was written without `-storage_version latest`. A released DuckDB with the current storage format is the fair opponent.
6. Nothing recorded which data file the run used, so there was no way to confirm that all three engines read the same file.
7. rudb answers q1 and q3 to q7 from column summaries it writes at load time, without reading the rows, and it answers q2 by reading only the rows that match. On those queries rudb looks up a stored answer while the other engines scan. The report did not say so, and those queries pull the geomean down a lot.

## What replaces it

The harness now runs ClickBench the upstream way by default (`rudb-bench run clickbench`, with `--protocol upstream`). The engines load one at a time and then take turns on each query, and the order rotates by one engine per query. Before each engine's turn the harness waits until the load average is below the number of hardware threads and records the reading. It then drops the page cache (stopping and restarting the ClickHouse server around the drop), runs three tries, and reports the best of the last two. ClickHouse runs as a server with the upstream primary key, the upstream load path and the eager-load config. All three engines get the same explicit memory budget, 80% of RAM unless `RUDB_BENCH_MEMORY` says otherwise, and the report records it. DuckDB loads with `-storage_version latest`, and the harness refuses to publish against a DuckDB development build. `--sample-file` pins the exact Parquet file, and a manifest with its path, row count, size and sha256 is written beside the file and beside the report. The per query table flags every query rudb's own metrics say it answered from stored summaries, marks q1 to q7 as summary shaped, and the headline gives the ratios both with and without q1 to q7.

The rerun on the same 10M sample under the new harness, on server2 with every load reading below its 6 hardware threads, is [clickbench-fair-10m-stored-answers-on.md](../2026-09-25/clickbench-fair-10m-stored-answers-on.md). In that run rudb still answered q1, q3 to q7 and q30 from stored summaries. The report flags those queries and gives the ratios with and without q1 to q7.
