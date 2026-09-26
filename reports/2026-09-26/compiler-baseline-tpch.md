# Compiler baseline: TPC-H SF1 and SF10, DuckDB pin and rudb

This is box 2 of milestone C0 (tamnd/rudb#1828): the TPC-H baselines at SF1 and SF10, retaken on the DuckDB pin and rudb's current engine on the same machine and in the same way as the ClickBench baseline beside it (`compiler-baseline-clickbench-3way.md`).

To reproduce it, lay out a work directory as `run.sh` describes and run:

```
sudo bash -c 'for s in 1 10; do SCALE=$s W=/path/to/workdir ENGINES="duckdb rudb" reports/2026-09-26/compiler-baseline-tpch/run.sh; done'
```

Then `python3 reports/2026-09-26/compiler-baseline-tpch/analyze.py /path/to/workdir/out/tpch-sf10` (or `tpch-sf1`) prints the tables below.

## What ran

| | |
| --- | --- |
| machine | gamingpc, i9-13900K, 32 hardware threads, 31 GiB, WSL2 (Linux 6.18.33.2-microsoft-standard-WSL2), ext4 |
| pinning | `taskset -c 0-15`, the P-cores, with `RUDB_BENCH_THREADS=16` and `RUDB_BENCH_MEMORY=20GiB` |
| load gate | `RUDB_BENCH_LOAD_LIMIT=2` |
| protocol | `--protocol upstream --runs 4`: the page cache dropped before each query, then 1 cold try and 3 hot tries, `--timeout 600` |
| data | generated for this run by `rudb-bench generate tpch` with DuckDB 1.5.5's tpch extension (the pin is a development build and cannot download the extension). SF1 is corpus 5739c03f28043b03, 8,661,245 rows. SF10 is corpus 02eb57f3a084bd10, 86,586,082 rows. The manifests are in the harness reports |
| rudb | 191f4360, `RUDB_BENCH_STORED_ANSWERS=off` |
| DuckDB | v2.0.0-dev84237 cc7e7bac7f, the pin, run as the `duckdb` engine |
| harness | rudb-bench ad68b0a |
| when | SF1 09:35 to 09:36 UTC, SF10 09:36 to 09:39 UTC on 2026-09-26, right after the ClickBench run |

ClickHouse was asked as well, to match the ClickBench run, but it did not get past loading at either scale: `clickhouse client` answered `INSERT INTO lineitem FORMAT Parquet` with `Code: 108. No data to insert`. The same binary loaded `hits.parquet` without trouble, so this is a gap in how the harness loads a multi table suite into the ClickHouse server, and it is left for a harness fix. The failed load is in `sf*/clickhouse-server.*`.

Load times: at SF1 DuckDB 1.27s and rudb 0.83s, at SF10 DuckDB 7.20s (2.62 GiB) and rudb 5.34s (2.49 GiB). Each engine's scratch was deleted as soon as its pass ended.

As in the ClickBench report, perf is not available in WSL, so the figure is the median of the 3 hot tries by the engine's own clock, with the harness clock beside it. The harness clock moves in 20 ms steps under `--timeout` and also pays for starting the process, which is most of a TPC-H SF1 query. These are wall times for comparing the two engines within this run, not absolute numbers.

The one minute load before each query stayed between 0.35 and 3 (median 1.8 at SF10), with nothing else running. The local-ocr vLLM service held 17,183 MiB of GPU memory at 0% utilization in every sample, and keepalive-gpc was left alone.

## SF1

rudb is faster than the pin on 21 of 22 queries by the engine clock, with a geometric mean ratio of 0.39x (best 0.19x on q19, worst 1.15x on q21). Totals: DuckDB 560 ms, rudb 238 ms. Cold totals: DuckDB 2.5 s, rudb 1.5 s.

| query | DuckDB | rudb | rudb / DuckDB | DuckDB wall | rudb wall | rudb / DuckDB, wall | DuckDB cold | rudb cold |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| q01 | 35.0 ms | 10.5 ms | 0.30x | 80.6 ms | 20.4 ms | 0.25x | 124.5 ms | 63.4 ms |
| q02 | 10.0 ms | 5.2 ms | 0.52x | 40.5 ms | 20.6 ms | 0.51x | 103.1 ms | 42.9 ms |
| q03 | 23.0 ms | 10.7 ms | 0.47x | 60.5 ms | 20.5 ms | 0.34x | 123.6 ms | 83.6 ms |
| q04 | 17.0 ms | 7.2 ms | 0.42x | 60.5 ms | 20.5 ms | 0.34x | 62.9 ms | 63.2 ms |
| q05 | 21.0 ms | 12.2 ms | 0.58x | 60.6 ms | 20.6 ms | 0.34x | 103.1 ms | 82.8 ms |
| q06 | 11.0 ms | 3.1 ms | 0.29x | 40.4 ms | 20.5 ms | 0.51x | 103.1 ms | 42.7 ms |
| q07 | 35.0 ms | 9.8 ms | 0.28x | 80.7 ms | 20.5 ms | 0.25x | 83.0 ms | 83.7 ms |
| q08 | 33.0 ms | 8.9 ms | 0.27x | 80.6 ms | 20.5 ms | 0.25x | 123.1 ms | 83.3 ms |
| q09 | 42.0 ms | 21.1 ms | 0.50x | 80.7 ms | 40.6 ms | 0.50x | 143.1 ms | 103.0 ms |
| q10 | 43.0 ms | 13.5 ms | 0.31x | 80.7 ms | 20.4 ms | 0.25x | 143.3 ms | 82.8 ms |
| q11 | 11.0 ms | 5.0 ms | 0.45x | 40.5 ms | 20.6 ms | 0.51x | 102.9 ms | 42.8 ms |
| q12 | 23.0 ms | 13.7 ms | 0.59x | 60.6 ms | 20.4 ms | 0.34x | 102.9 ms | 83.1 ms |
| q13 | 37.0 ms | 12.9 ms | 0.35x | 80.7 ms | 20.4 ms | 0.25x | 123.1 ms | 42.9 ms |
| q14 | 21.0 ms | 5.5 ms | 0.26x | 60.6 ms | 20.5 ms | 0.34x | 123.5 ms | 63.0 ms |
| q15 | 25.0 ms | 5.8 ms | 0.23x | 60.5 ms | 20.5 ms | 0.34x | 102.9 ms | 63.2 ms |
| q16 | 29.0 ms | 19.7 ms | 0.68x | 61.1 ms | 41.6 ms | 0.68x | 124.0 ms | 64.1 ms |
| q17 | 18.0 ms | 6.3 ms | 0.35x | 60.5 ms | 20.5 ms | 0.34x | 103.2 ms | 62.8 ms |
| q18 | 28.0 ms | 14.0 ms | 0.50x | 60.6 ms | 20.5 ms | 0.34x | 123.2 ms | 62.8 ms |
| q19 | 25.0 ms | 4.9 ms | 0.19x | 60.6 ms | 20.4 ms | 0.34x | 123.2 ms | 62.7 ms |
| q20 | 22.0 ms | 8.7 ms | 0.39x | 60.6 ms | 20.6 ms | 0.34x | 123.7 ms | 84.7 ms |
| q21 | 29.0 ms | 33.4 ms | 1.15x | 63.1 ms | 40.5 ms | 0.64x | 122.9 ms | 104.3 ms |
| q22 | 22.0 ms | 5.9 ms | 0.27x | 60.5 ms | 20.5 ms | 0.34x | 103.1 ms | 42.9 ms |
| total | 560.0 ms | 237.9 ms | | 1.40 s | 512.1 ms | | | |

## SF10

rudb is faster than the pin on 21 of 22 queries by the engine clock, with a geometric mean ratio of 0.53x (best 0.22x on q06, worst 1.64x on q21). Totals: DuckDB 2.76 s, rudb 1.72 s. Cold, rudb is behind: 7.6 s against 5.7 s for DuckDB, and slower on 17 of 22 queries.

| query | DuckDB | rudb | rudb / DuckDB | DuckDB wall | rudb wall | rudb / DuckDB, wall | DuckDB cold | rudb cold |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| q01 | 118.0 ms | 71.1 ms | 0.60x | 161.2 ms | 84.0 ms | 0.52x | 245.8 ms | 307.9 ms |
| q02 | 35.0 ms | 16.8 ms | 0.48x | 80.7 ms | 40.7 ms | 0.50x | 123.2 ms | 104.4 ms |
| q03 | 109.0 ms | 84.7 ms | 0.78x | 162.2 ms | 102.4 ms | 0.63x | 284.1 ms | 446.1 ms |
| q04 | 115.0 ms | 73.6 ms | 0.64x | 161.5 ms | 104.0 ms | 0.64x | 225.5 ms | 385.6 ms |
| q05 | 133.0 ms | 78.3 ms | 0.59x | 181.1 ms | 104.1 ms | 0.57x | 263.9 ms | 425.6 ms |
| q06 | 73.0 ms | 16.2 ms | 0.22x | 121.1 ms | 40.7 ms | 0.34x | 203.5 ms | 285.1 ms |
| q07 | 137.0 ms | 58.1 ms | 0.42x | 181.4 ms | 82.8 ms | 0.46x | 264.8 ms | 405.1 ms |
| q08 | 136.0 ms | 73.4 ms | 0.54x | 182.1 ms | 104.4 ms | 0.57x | 304.1 ms | 425.0 ms |
| q09 | 305.0 ms | 209.3 ms | 0.69x | 365.7 ms | 244.2 ms | 0.67x | 488.8 ms | 527.0 ms |
| q10 | 195.0 ms | 95.1 ms | 0.49x | 243.2 ms | 123.9 ms | 0.51x | 344.8 ms | 445.5 ms |
| q11 | 28.0 ms | 19.1 ms | 0.68x | 60.5 ms | 40.6 ms | 0.67x | 123.3 ms | 103.6 ms |
| q12 | 138.0 ms | 96.9 ms | 0.70x | 181.1 ms | 125.1 ms | 0.69x | 243.5 ms | 404.5 ms |
| q13 | 198.0 ms | 152.4 ms | 0.77x | 245.2 ms | 165.2 ms | 0.67x | 327.6 ms | 228.0 ms |
| q14 | 105.0 ms | 37.3 ms | 0.36x | 161.4 ms | 60.9 ms | 0.38x | 244.9 ms | 324.1 ms |
| q15 | 106.0 ms | 32.7 ms | 0.31x | 160.9 ms | 60.7 ms | 0.38x | 224.0 ms | 304.9 ms |
| q16 | 78.0 ms | 50.4 ms | 0.65x | 122.7 ms | 62.6 ms | 0.51x | 165.7 ms | 125.8 ms |
| q17 | 101.0 ms | 35.5 ms | 0.35x | 141.4 ms | 62.0 ms | 0.44x | 225.5 ms | 364.3 ms |
| q18 | 209.0 ms | 121.1 ms | 0.58x | 263.7 ms | 143.1 ms | 0.54x | 348.2 ms | 468.9 ms |
| q19 | 118.0 ms | 27.8 ms | 0.24x | 161.1 ms | 43.9 ms | 0.27x | 303.8 ms | 305.4 ms |
| q20 | 109.0 ms | 57.9 ms | 0.53x | 162.1 ms | 81.0 ms | 0.50x | 264.6 ms | 387.1 ms |
| q21 | 171.0 ms | 280.2 ms | 1.64x | 222.9 ms | 305.2 ms | 1.37x | 331.9 ms | 647.3 ms |
| q22 | 47.0 ms | 30.3 ms | 0.64x | 80.9 ms | 40.7 ms | 0.50x | 143.5 ms | 143.8 ms |
| total | 2.76 s | 1.72 s | | 3.80 s | 2.22 s | | | |

## Against the goal of 10x

No query reaches 10x at either scale. The best is 0.19x (q19 at SF1) and 0.22x (q06 at SF10). The ratio gets worse from SF1 to SF10 (0.39x to 0.53x geometric mean), and q21 is the one query rudb loses at both scales, by 1.15x and 1.64x.

## Files

- `run.sh`: the driver that ran, with the work directory as `W` and the scale as `SCALE`.
- `analyze.py`: prints the tables from one scale's outputs. `sf1/analysis.txt` and `sf10/analysis.txt` are its output.
- `sf*/<engine>.harness.log`: the harness output with one line per query (cold, fastest and median hot, on both clocks).
- `sf*/<engine>.run.md`: the harness's own report for each pass.
- `sf*/load.log` and `sf*/progress.log`: load and GPU samples every 10 seconds, and when each pass started and ended.
