# Compiler baseline: ClickBench at 100M rows, ClickHouse, DuckDB and rudb

This is box 1 of milestone C0 (tamnd/rudb#1828). It is the number the compiler work has to beat: ClickBench over the full hits table (99,997,497 rows) on ClickHouse, DuckDB and rudb's current engine, cold and hot, run the same way on the same machine.

To reproduce it, lay out a work directory as `run.sh` describes (rudb-bench at dff7219 or later in `bench/`, the three binaries in `bin/`, `hits.parquet` in `data/`) and run:

```
sudo env W=/path/to/workdir ENGINES="clickhouse-server duckdb rudb" reports/2026-09-26/compiler-baseline-clickbench-3way/run.sh
```

Then `python3 reports/2026-09-26/compiler-baseline-clickbench-3way/analyze.py /path/to/workdir/out` prints the tables below.

## What ran

| | |
| --- | --- |
| machine | gamingpc, i9-13900K, 32 hardware threads, 31 GiB, WSL2 (Linux 6.18.33.2-microsoft-standard-WSL2), ext4 on /dev/sdd |
| pinning | every engine under `taskset -c 0-15`, the eight P-cores and their hyperthreads, `RUDB_BENCH_THREADS=16`, `RUDB_BENCH_MEMORY=20GiB` |
| load gate | `RUDB_BENCH_LOAD_LIMIT=2`, so the harness waited before a query while the one minute load was over 2 |
| protocol | `--protocol upstream --runs 4`: the page cache dropped before each query, then 1 cold try and 3 hot tries, `--timeout 600` |
| data | `hits.parquet`, 14,779,976,446 bytes, sha256 a390f6cb782f6aaef278c72fc1dd86c4f30bc843ebab3c159e9bd4d45ddb079f, the same file as on server3 |
| rudb | 191f4360 (tamnd/rudb#2002), release build, `RUDB_BENCH_STORED_ANSWERS=off` so every query ran with `SET stored_answers = false` |
| DuckDB | v2.0.0-dev84237 cc7e7bac7f, the pinned build. It is also the `duckdb` on PATH, so the harness runs it as the `duckdb` engine and refuses a separate pinned row |
| ClickHouse | 26.9.1.1138, the binary from server3 copied over (sha256 c0b2f0e9603a3dcb...), not gamingpc's own 26.9.1.1562, run as `clickhouse-server` |
| harness | rudb-bench dff7219, which prints the median of the hot tries on each progress line (#248) |
| order | DuckDB 08:44 to 08:54 UTC, rudb 08:54 to 09:00, ClickHouse 09:10 to 09:25 (it started once its binary had arrived) |

Each engine loaded the table into its own scratch directory, ran its 43 queries, and the scratch was deleted as soon as its pass ended. Load times: DuckDB 42.1s (19.05 GiB), rudb 45.5s (10.66 GiB), ClickHouse 41.3s (9.37 GiB).

## Why the median of wall times and not instructions

The plan was instructions retired per query, as in `scripts/instructions-per-query.py`. perf is not installed in this WSL2 image, so there is no instruction count. The figure here is the median of the 3 hot tries of each query instead.

There are two wall clocks per try, and both are in the tables:

- The engine's own clock, the time the engine says the query took. This is the main figure, and it is what the ratios use.
- The harness clock around the whole subprocess. It also pays for starting the process, opening the database and printing the answer. Because a run with `--timeout` polls the child every 20 ms (`wait_within` in `src/data.rs`), this clock moves in steps of about 20 ms. That makes it too coarse for the many queries that finish in under 100 ms, which is why it is the second figure.

These are wall times on one machine in one run. They are for comparing the three engines against each other within this run, not absolute numbers to set next to the public ClickBench board.

## The machine while it ran

The one minute load before each query had a median of 1.85 for ClickHouse, 2.62 for DuckDB and 1.88 for rudb, and a maximum of 4.59, 2.84 and 3.01. Most of that is the engines' own threads decaying out of the average between queries, since nothing else ran. The per query load is in `analysis.txt`, next to the cold times.

The local-ocr vLLM service (olmOCR-2-7B-1025-FP8, pid 144) stayed up the whole time, as asked. It held 17,183 MiB of the RTX 4090's memory at 0% GPU utilization in every sample, 180 samples at 10 second intervals (`load.log`). The keepalive-gpc process (pid 1432) was left alone.

## Per query

Median of the 3 hot tries. The ratio is rudb's time divided by the faster of ClickHouse and DuckDB on that query, so 0.10x is rudb 10 times faster and anything over 1x is rudb losing.

| query | ClickHouse | DuckDB | rudb | rudb / best rival | ClickHouse wall | DuckDB wall | rudb wall | rudb / best rival, wall |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| q1 | 1.0 ms | 9.0 ms | 3.0 ms | 2.97x against ClickHouse | 40.3 ms | 40.4 ms | 20.3 ms | 0.50x |
| q2 | 1.0 ms | 26.0 ms | 7.3 ms | 7.29x against ClickHouse | 40.3 ms | 60.5 ms | 40.4 ms | 1.00x |
| q3 | 21.0 ms | 46.0 ms | 23.1 ms | 1.10x against ClickHouse | 63.1 ms | 80.6 ms | 40.6 ms | 0.64x |
| q4 | 27.0 ms | 50.0 ms | 18.6 ms | 0.69x against ClickHouse | 80.5 ms | 80.6 ms | 44.0 ms | 0.55x |
| q5 | 166.0 ms | 183.0 ms | 155.9 ms | 0.94x against ClickHouse | 204.4 ms | 228.0 ms | 204.2 ms | 1.00x |
| q6 | 238.0 ms | 217.0 ms | 358.4 ms | 1.65x against DuckDB | 284.1 ms | 268.2 ms | 381.8 ms | 1.42x |
| q7 | 14.0 ms | 16.0 ms | 6.2 ms | 0.44x against ClickHouse | 60.8 ms | 60.5 ms | 20.3 ms | 0.34x |
| q8 | 8.0 ms | 22.0 ms | 8.4 ms | 1.05x against ClickHouse | 60.5 ms | 60.5 ms | 40.9 ms | 0.68x |
| q9 | 313.0 ms | 232.0 ms | 106.3 ms | 0.46x against DuckDB | 366.8 ms | 285.9 ms | 144.1 ms | 0.50x |
| q10 | 348.0 ms | 317.0 ms | 140.1 ms | 0.44x against DuckDB | 390.7 ms | 368.6 ms | 182.3 ms | 0.49x |
| q11 | 94.0 ms | 85.0 ms | 31.1 ms | 0.37x against DuckDB | 144.9 ms | 121.2 ms | 64.0 ms | 0.53x |
| q12 | 104.0 ms | 106.0 ms | 37.3 ms | 0.36x against ClickHouse | 143.8 ms | 142.3 ms | 60.5 ms | 0.43x |
| q13 | 233.0 ms | 244.0 ms | 35.2 ms | 0.15x against ClickHouse | 288.0 ms | 305.5 ms | 60.8 ms | 0.21x |
| q14 | 376.0 ms | 463.0 ms | 85.1 ms | 0.23x against ClickHouse | 424.4 ms | 547.8 ms | 124.0 ms | 0.29x |
| q15 | 274.0 ms | 278.0 ms | 76.9 ms | 0.28x against ClickHouse | 309.9 ms | 343.7 ms | 104.7 ms | 0.34x |
| q16 (heavy hitters) | 152.0 ms | 220.0 ms | 0.7 ms | 0.00x against ClickHouse | 184.4 ms | 265.3 ms | 20.4 ms | 0.11x |
| q17 | 649.0 ms | 543.0 ms | 19.3 ms | 0.04x against DuckDB | 692.0 ms | 668.2 ms | 40.5 ms | 0.06x |
| q18 | 248.0 ms | 385.0 ms | 35.0 ms | 0.14x against ClickHouse | 286.6 ms | 465.7 ms | 60.6 ms | 0.21x |
| q19 | 1.18 s | 991.0 ms | 298.2 ms | 0.30x against DuckDB | 1.21 s | 1.19 s | 364.9 ms | 0.31x |
| q20 | 2.0 ms | 31.0 ms | 2.9 ms | 1.44x against ClickHouse | 40.4 ms | 60.5 ms | 20.3 ms | 0.50x |
| q21 | 234.0 ms | 402.0 ms | 218.9 ms | 0.94x against ClickHouse | 283.9 ms | 464.8 ms | 246.0 ms | 0.87x |
| q22 | 70.0 ms | 440.0 ms | 297.1 ms | 4.24x against ClickHouse | 120.7 ms | 508.4 ms | 345.7 ms | 2.87x |
| q23 | 352.0 ms | 488.0 ms | 288.5 ms | 0.82x against ClickHouse | 391.0 ms | 627.0 ms | 324.0 ms | 0.83x |
| q24 | 62.0 ms | 88.0 ms | 101.8 ms | 1.64x against ClickHouse | 103.4 ms | 140.8 ms | 121.3 ms | 1.17x |
| q25 | 36.0 ms | 38.0 ms | 15.7 ms | 0.44x against ClickHouse | 80.6 ms | 80.5 ms | 40.4 ms | 0.50x |
| q26 | 111.0 ms | 88.0 ms | 62.4 ms | 0.71x against DuckDB | 153.7 ms | 127.4 ms | 80.8 ms | 0.63x |
| q27 | 38.0 ms | 39.0 ms | 44.7 ms | 1.18x against ClickHouse | 84.1 ms | 80.5 ms | 60.5 ms | 0.75x |
| q28 | 94.0 ms | 352.0 ms | 221.0 ms | 2.35x against ClickHouse | 142.5 ms | 424.9 ms | 256.0 ms | 1.80x |
| q29 | 995.0 ms | 3.50 s | 1.01 s | 1.02x against ClickHouse | 1.04 s | 3.66 s | 1.06 s | 1.02x |
| q30 | 27.0 ms | 44.0 ms | 17.9 ms | 0.66x against ClickHouse | 73.4 ms | 80.6 ms | 41.7 ms | 0.57x |
| q31 | 149.0 ms | 249.0 ms | 96.5 ms | 0.65x against ClickHouse | 204.0 ms | 304.0 ms | 126.0 ms | 0.62x |
| q32 | 210.0 ms | 351.0 ms | 102.5 ms | 0.49x against ClickHouse | 248.3 ms | 431.0 ms | 144.1 ms | 0.58x |
| q33 | 1.26 s | 1.25 s | 280.2 ms | 0.22x against DuckDB | 1.31 s | 1.54 s | 347.6 ms | 0.27x |
| q34 (heavy hitters) | 1.34 s | 1.12 s | 0.7 ms | 0.00x against DuckDB | 1.39 s | 1.38 s | 20.4 ms | 0.01x |
| q35 | 1.34 s | 1.21 s | 97.8 ms | 0.08x against DuckDB | 1.39 s | 1.48 s | 124.1 ms | 0.09x |
| q36 (heavy hitters) | 129.0 ms | 190.0 ms | 0.7 ms | 0.01x against ClickHouse | 166.0 ms | 243.6 ms | 20.3 ms | 0.12x |
| q37 | 21.0 ms | 30.0 ms | 24.6 ms | 1.17x against ClickHouse | 60.5 ms | 60.4 ms | 40.4 ms | 0.67x |
| q38 | 11.0 ms | 19.0 ms | 15.4 ms | 1.40x against ClickHouse | 60.5 ms | 60.5 ms | 40.4 ms | 0.67x |
| q39 | 11.0 ms | 22.0 ms | 22.7 ms | 2.06x against ClickHouse | 60.5 ms | 60.5 ms | 40.4 ms | 0.67x |
| q40 | 47.0 ms | 48.0 ms | 62.2 ms | 1.32x against ClickHouse | 100.7 ms | 80.6 ms | 80.6 ms | 1.00x |
| q41 | 8.0 ms | 19.0 ms | 6.5 ms | 0.81x against ClickHouse | 60.5 ms | 60.4 ms | 20.4 ms | 0.34x |
| q42 | 6.0 ms | 19.0 ms | 8.5 ms | 1.42x against ClickHouse | 60.5 ms | 60.4 ms | 40.4 ms | 0.67x |
| q43 | 6.0 ms | 18.0 ms | 3.9 ms | 0.64x against ClickHouse | 60.5 ms | 60.4 ms | 20.4 ms | 0.34x |
| total | 11.01 s | 14.48 s | 4.45 s | | 12.97 s | 17.66 s | 5.69 s | |

## Against the goal of 10x on every query

By the engine clock, rudb is at or under 0.1x of the best rival on 5 of 43 queries (q16, q17, q34, q35, q36) and under 1x on 27. The geometric mean of the ratio is 0.47x, and the total hot time is 4.45 s against 11.01 s for ClickHouse and 14.48 s for DuckDB.

Three of those five do not measure a scan. q16, q34 and q36 each ask for the top 10 groups by count, and rudb answers them in about 0.7 ms with no CPU charged and 25 MiB read, against 129 ms to 1.34 s for the rivals. rudb keeps exact heavy hitter lists per column when it loads a table (`crates/rudb-native`), and `stored_answers = false` does not turn those off. Leaving those three out, the geometric mean is 0.69x, rudb is at or under 0.1x on 2 of 40 (q17 at 0.04x and q35 at 0.08x), and it is slower than the best rival on 16 of 40.

The queries where rudb loses by the engine clock, worst first:

| query | rudb / best rival | what it is |
| --- | ---: | --- |
| q2 | 7.29x | filtered count, ClickHouse 1.0 ms |
| q22 | 4.24x | substring scan and group by, ClickHouse 70 ms |
| q1 | 2.97x | count, ClickHouse 1.0 ms |
| q28 | 2.35x | group by with a string length, ClickHouse 94 ms |
| q39 | 2.06x | date range, group by and offset |
| q6 | 1.65x | count distinct over strings, DuckDB 217 ms |
| q24 | 1.64x | select star and top k |
| q20 | 1.44x | point lookup |
| q42 | 1.42x | date range and a deep offset |
| q38 | 1.40x | date range and group by a title |
| q40 | 1.32x | date range, a case and a wide group by |
| q27 | 1.18x | top k by two columns |
| q37 | 1.17x | date range and group by a URL |
| q3 | 1.10x | three aggregates |
| q8 | 1.05x | group by, low cardinality |
| q29 | 1.02x | group by a regular expression |

By the harness clock the picture is kinder to rudb (geometric mean 0.46x, 37 of 43 under 1x), mostly because the 20 ms steps hide the gaps on the fast queries, where ClickHouse leads by the engine clock. It is in the table for completeness and not for the goal.

Cold, rudb is the slowest of the three on most of q2 to q15 (about 650 to 800 ms, against 40 to 500 ms for ClickHouse). Its cold tries read around 4 GiB for queries that touch one or two columns, which is the first thing to look at for cold.

## Earlier attempt on server3

This run was first tried on server3 with instructions retired as the metric. The machine stayed at a load of 25 to 50 from other tenants all day, one pass was stopped when the disk filled, and a first rudb instructions pass was taken with stored answers still on. None of that is used here. The instructions script can now turn stored answers off (#247), for whenever the run can be repeated on a machine with perf.

## Files

- `run.sh`: the driver that ran, with the work directory as `W`.
- `analyze.py` and `analysis.txt`: the tables above, joined from the progress lines and the load samples.
- `<engine>.harness.log`: the harness output, one line per query with the cold try, the fastest and median hot try on both clocks, and every load gate wait.
- `<engine>.run.md`: the harness's own report for each engine's pass.
- `load.log`: the load average and GPU state every 10 seconds. `progress.log`: when each pass started and ended, with free disk.
