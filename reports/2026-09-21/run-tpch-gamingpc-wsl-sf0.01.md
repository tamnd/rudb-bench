# tpch on gamingpc-wsl

This is one run of the tpch suite on gamingpc-wsl, over 6 engines and 22 queries, with 5 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

## What ran

| what | it was |
| --- | --- |
| suite | tpch |
| queries | 22 |
| tables | 2.87 MiB of Parquet in 8 tables |
| rows | 86642 in the table every query reads |
| corpus | duckdb-tpch SF0.01, corpus 91a5c3016d18284e, written 2026-09-21 |
| summary | median with the interquartile range, per reporting rule two |
| timeout | 900s per query, and no query reached it |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run tpch --engines duckdb,duckdb-pinned,clickhouse-local,datafusion,polars,rudb --runs 5 --timeout 900 --report
```

The data goes under `RUDB_BENCH_DATA`, which defaults to `~/rudb-data`, and the engines have to be on the path. An engine that is not gets a row in the abstentions below rather than being left out of the table.

## The machine

| fact | value | source |
| --- | --- | --- |
| host | GamingPC | read |
| os | Linux 6.18.33.2-microsoft-standard-WSL2 x86_64 | read |
| cpu | 13th Gen Intel(R) Core(TM) i9-13900K | read |
| threads | 32 | read |
| memory | 31.34 GiB | read |
| governor | no cpufreq here, the policy is not ours to see | not read |
| turbo | no intel_pstate here, the state is not ours to see | not read |
| filesystem | tmpfs /tmp tmpfs rw,nosuid,nodev,nr_inodes=1048576 0 0 | read |
| page cache | cannot open drop_caches: Permission denied (os error 13) | not read |
| timer | /usr/bin/time GNU | read |

A fact that says it was not read is a fact about this machine and not a gap in the report. A field that had silently defaulted would be a lie that survived into it.

## The engines

| engine | version | state | load | load cpu | on disk | that size is | format | machine load |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 138.434ms | 130.000ms | 4.26 MiB | its own database file | its own | 9.95 to 9.39 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 159.868ms | 180.000ms | 4.26 MiB | its own database file | its own | 9.39 to 9.28 |
| clickhouse-local | 26.9.1.1562 | ran | 715.338ms | 680.000ms | 4.57 MiB | its own MergeTree parts, as system.parts counts the active ones | its own | 9.28 to 10.00 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 2.87 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 10.00 to 10.64 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 2.87 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 10.64 to 9.59 |
| rudb | rudb 0.3.67 | ran | 0.000us | 0.000us | 2.87 MiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 9.59 to 9.59 |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb on 19 shared |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 109.000ms | 446.123ms | +309% | 446.137ms | 210.000ms | 0.47 | 50.14 MiB | none | 17.49M/s | 580.14 MiB/s | 1.00x |
| duckdb-pinned | 186.000ms | 910.668ms | +390% | 931.007ms | 760.000ms | 0.83 | 55.90 MiB | none | 10.25M/s | 339.97 MiB/s | 1.74x |
| clickhouse-local | 749.000ms | 2.309s | +208% | 2.494s | 2.340s | 1.01 | 426.12 MiB | none | 2.54M/s | 84.43 MiB/s | 7.55x |
| datafusion | 295.000ms | 965.671ms | +227% | 980.669ms | 1.360s | 1.41 | 224.61 MiB | none | 6.46M/s | 214.36 MiB/s | 2.71x |
| polars | 541.142ms | 2.745s | +407% | 2.922s | 3.910s | 1.42 | 127.34 MiB | none | 3.04M/s | 100.92 MiB/s | 5.70x |
| rudb | 72.650ms | 451.144ms | +521% | 453.078ms | 0.000us | 0.00 | 16.73 MiB | none | 26.24M/s | 870.40 MiB/s | 0.66x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 22 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 3.000ms | 12.000ms | 40.000ms | 13.000ms | 17.830ms | 4.513ms |
| q02 | minimum cost supplier | 6.000ms | 7.000ms | 11.000ms | 17.000ms | no dialect | 1.839ms |
| q03 | shipping priority | 5.000ms | 5.000ms | 42.000ms | 13.000ms | 22.906ms | 3.248ms |
| q04 | order priority checking | 5.000ms | 7.000ms | 14.000ms | 10.000ms | 17.174ms | 2.108ms |
| q05 | local supplier volume | 6.000ms | 9.000ms | 41.000ms | 16.000ms | 38.821ms | 4.233ms |
| q06 | forecasting revenue change | 1.000ms | 2.000ms | 42.000ms | 6.000ms | 14.070ms | 1.873ms |
| q07 | volume shipping | 6.000ms | 18.000ms | 52.000ms | 19.000ms | 41.107ms | 3.647ms |
| q08 | national market share | 6.000ms | 9.000ms | 51.000ms | 17.000ms | 42.445ms | 3.466ms |
| q09 | product type profit measure | 6.000ms | 13.000ms | 47.000ms | 23.000ms | 42.946ms | 5.980ms |
| q10 | returned item reporting | 8.000ms | 8.000ms | 50.000ms | 23.000ms | 37.939ms | 3.899ms |
| q11 | important stock identification | 4.000ms | 7.000ms | 6.000ms | 13.000ms | 19.849ms | 1.696ms |
| q12 | shipping modes and order priority | 4.000ms | 8.000ms | 43.000ms | 12.000ms | 17.913ms | 3.598ms |
| q13 | customer distribution | 5.000ms | 11.000ms | 12.000ms | 9.000ms | no dialect | 5.305ms |
| q14 | promotion effect | 3.000ms | 3.000ms | 42.000ms | 8.000ms | 19.266ms | 2.291ms |
| q15 | top supplier | 4.000ms | 8.000ms | 54.000ms | 12.000ms | 25.569ms | 3.198ms |
| q16 | parts supplier relationship | 6.000ms | 21.000ms | 7.000ms | 11.000ms | 24.582ms | 1.458ms |
| q17 | small quantity order revenue | 3.000ms | 3.000ms | 9.000ms | 12.000ms | no dialect | 2.474ms |
| q18 | large volume customer | 6.000ms | 6.000ms | 10.000ms | 15.000ms | 33.432ms | 5.867ms |
| q19 | discounted revenue | 3.000ms | 5.000ms | 42.000ms | 10.000ms | 15.142ms | 2.674ms |
| q20 | potential part promotion | 6.000ms | 8.000ms | 45.000ms | 12.000ms | 24.509ms | 3.192ms |
| q21 | suppliers who kept orders waiting | 8.000ms | 7.000ms | 82.000ms | 15.000ms | 66.288ms | 4.262ms |
| q22 | global sales opportunity | 5.000ms | 9.000ms | 7.000ms | 9.000ms | 19.354ms | 1.830ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 3.000ms | 20.317ms | 20.257ms | 0.1% | 20.249ms | 20.261ms | 20.246ms | 20.284ms | 0.000us | 35.26 MiB | none | 4.28M/s |
| q02 | minimum cost supplier | 6.000ms | 20.278ms | 20.260ms | 0.1% | 20.252ms | 20.267ms | 20.237ms | 20.269ms | 10.000ms | 41.45 MiB | none | 4.28M/s |
| q03 | shipping priority | 5.000ms | 20.287ms | 20.253ms | 0.1% | 20.250ms | 20.276ms | 20.248ms | 20.278ms | 10.000ms | 40.89 MiB | none | 4.28M/s |
| q04 | order priority checking | 5.000ms | 20.308ms | 20.257ms | 0.0% | 20.252ms | 20.259ms | 20.227ms | 20.301ms | 10.000ms | 41.02 MiB | none | 4.28M/s |
| q05 | local supplier volume | 6.000ms | 20.262ms | 20.244ms | 0.1% | 20.237ms | 20.261ms | 20.228ms | 20.278ms | 10.000ms | 41.07 MiB | none | 4.28M/s |
| q06 | forecasting revenue change | 1.000ms | 20.263ms | 20.288ms | 0.2% | 20.257ms | 20.302ms | 20.218ms | 20.310ms | 0.000us | 32.50 MiB | none | 4.27M/s |
| q07 | volume shipping | 6.000ms | 20.254ms | 20.275ms | 0.1% | 20.257ms | 20.285ms | 20.238ms | 20.310ms | 10.000ms | 42.70 MiB | none | 4.27M/s |
| q08 | national market share | 6.000ms | 20.232ms | 20.297ms | 0.1% | 20.278ms | 20.297ms | 20.257ms | 20.302ms | 10.000ms | 42.64 MiB | none | 4.27M/s |
| q09 | product type profit measure | 6.000ms | 20.294ms | 20.293ms | 0.7% | 20.276ms | 20.421ms | 20.262ms | 21.574ms | 10.000ms | 44.01 MiB | none | 4.27M/s |
| q10 | returned item reporting | 8.000ms | 20.249ms | 20.285ms | 0.0% | 20.283ms | 20.287ms | 20.261ms | 40.358ms | 20.000ms | 50.14 MiB | none | 4.27M/s |
| q11 | important stock identification | 4.000ms | 20.258ms | 20.298ms | 0.5% | 20.273ms | 20.381ms | 20.271ms | 22.378ms | 10.000ms | 40.06 MiB | none | 4.27M/s |
| q12 | shipping modes and order priority | 4.000ms | 20.240ms | 20.279ms | 0.1% | 20.257ms | 20.285ms | 20.256ms | 20.337ms | 10.000ms | 38.63 MiB | none | 4.27M/s |
| q13 | customer distribution | 5.000ms | 20.244ms | 20.260ms | 0.0% | 20.259ms | 20.262ms | 20.246ms | 20.272ms | 10.000ms | 38.90 MiB | none | 4.28M/s |
| q14 | promotion effect | 3.000ms | 20.244ms | 20.271ms | 0.2% | 20.256ms | 20.291ms | 20.252ms | 20.303ms | 0.000us | 34.82 MiB | none | 4.27M/s |
| q15 | top supplier | 4.000ms | 20.232ms | 20.265ms | 0.5% | 20.251ms | 20.349ms | 20.250ms | 20.425ms | 10.000ms | 37.44 MiB | none | 4.28M/s |
| q16 | parts supplier relationship | 6.000ms | 20.414ms | 20.289ms | 0.2% | 20.274ms | 20.309ms | 20.270ms | 20.325ms | 10.000ms | 48.64 MiB | none | 4.27M/s |
| q17 | small quantity order revenue | 3.000ms | 20.392ms | 20.261ms | 0.2% | 20.253ms | 20.283ms | 20.243ms | 20.338ms | 10.000ms | 35.82 MiB | none | 4.28M/s |
| q18 | large volume customer | 6.000ms | 20.288ms | 20.268ms | 0.2% | 20.239ms | 20.286ms | 20.239ms | 20.407ms | 10.000ms | 42.57 MiB | none | 4.27M/s |
| q19 | discounted revenue | 3.000ms | 20.267ms | 20.262ms | 0.1% | 20.253ms | 20.270ms | 20.249ms | 20.287ms | 10.000ms | 35.57 MiB | none | 4.28M/s |
| q20 | potential part promotion | 6.000ms | 20.260ms | 20.414ms | 0.7% | 20.368ms | 20.512ms | 20.366ms | 40.336ms | 10.000ms | 45.53 MiB | none | 4.24M/s |
| q21 | suppliers who kept orders waiting | 8.000ms | 20.278ms | 20.272ms | 0.1% | 20.259ms | 20.280ms | 20.236ms | 40.340ms | 20.000ms | 44.95 MiB | none | 4.27M/s |
| q22 | global sales opportunity | 5.000ms | 20.277ms | 20.278ms | 0.3% | 20.246ms | 20.312ms | 20.223ms | 20.406ms | 10.000ms | 41.69 MiB | none | 4.27M/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 22 of 22 queries. Total 109.000ms by its own clock and 446.123ms by ours, 446.137ms cold, 210.000ms of CPU, peak 50.14 MiB, 17.49M/s and 580.14 MiB/s.

Running it cost 309% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.01x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 12.000ms | 40.333ms | 40.428ms | 51.0% | 40.370ms | 60.970ms | 40.360ms | 67.330ms | 50.000ms | 45.90 MiB | none | 2.14M/s |
| q02 | minimum cost supplier | 7.000ms | 40.350ms | 40.442ms | 0.4% | 40.359ms | 40.541ms | 40.331ms | 49.475ms | 20.000ms | 46.93 MiB | none | 2.14M/s |
| q03 | shipping priority | 5.000ms | 40.969ms | 40.404ms | 0.2% | 40.359ms | 40.459ms | 40.328ms | 41.188ms | 30.000ms | 47.91 MiB | none | 2.14M/s |
| q04 | order priority checking | 7.000ms | 40.322ms | 40.559ms | 0.3% | 40.486ms | 40.604ms | 40.335ms | 42.093ms | 30.000ms | 47.91 MiB | none | 2.14M/s |
| q05 | local supplier volume | 9.000ms | 40.318ms | 40.432ms | 0.1% | 40.425ms | 40.461ms | 40.405ms | 44.649ms | 30.000ms | 48.98 MiB | none | 2.14M/s |
| q06 | forecasting revenue change | 2.000ms | 40.362ms | 40.408ms | 0.2% | 40.354ms | 40.454ms | 40.323ms | 40.459ms | 20.000ms | 41.78 MiB | none | 2.14M/s |
| q07 | volume shipping | 18.000ms | 40.324ms | 40.417ms | 49.9% | 40.336ms | 60.521ms | 40.318ms | 60.582ms | 50.000ms | 50.56 MiB | none | 2.14M/s |
| q08 | national market share | 9.000ms | 40.352ms | 40.373ms | 0.4% | 40.337ms | 40.494ms | 40.330ms | 40.556ms | 40.000ms | 49.94 MiB | none | 2.15M/s |
| q09 | product type profit measure | 13.000ms | 42.597ms | 40.382ms | 0.1% | 40.362ms | 40.402ms | 40.335ms | 60.583ms | 40.000ms | 51.94 MiB | none | 2.15M/s |
| q10 | returned item reporting | 8.000ms | 40.330ms | 40.399ms | 0.7% | 40.375ms | 40.639ms | 40.324ms | 43.688ms | 30.000ms | 55.90 MiB | none | 2.14M/s |
| q11 | important stock identification | 7.000ms | 40.830ms | 40.406ms | 0.2% | 40.340ms | 40.429ms | 40.320ms | 44.317ms | 30.000ms | 46.74 MiB | none | 2.14M/s |
| q12 | shipping modes and order priority | 8.000ms | 40.373ms | 40.324ms | 0.2% | 40.323ms | 40.387ms | 40.322ms | 40.449ms | 40.000ms | 47.29 MiB | none | 2.15M/s |
| q13 | customer distribution | 11.000ms | 40.318ms | 40.418ms | 0.1% | 40.390ms | 40.429ms | 40.360ms | 41.651ms | 40.000ms | 48.18 MiB | none | 2.14M/s |
| q14 | promotion effect | 3.000ms | 40.353ms | 40.375ms | 0.2% | 40.367ms | 40.439ms | 40.318ms | 40.496ms | 20.000ms | 44.49 MiB | none | 2.15M/s |
| q15 | top supplier | 8.000ms | 40.432ms | 40.432ms | 0.3% | 40.379ms | 40.492ms | 40.378ms | 41.392ms | 30.000ms | 46.47 MiB | none | 2.14M/s |
| q16 | parts supplier relationship | 21.000ms | 60.484ms | 60.696ms | 0.3% | 60.535ms | 60.726ms | 42.587ms | 60.813ms | 60.000ms | 51.63 MiB | none | 1.43M/s |
| q17 | small quantity order revenue | 3.000ms | 40.444ms | 40.982ms | 48.5% | 40.797ms | 60.663ms | 40.399ms | 61.540ms | 30.000ms | 44.02 MiB | none | 2.11M/s |
| q18 | large volume customer | 6.000ms | 40.838ms | 40.799ms | 6.5% | 40.396ms | 43.060ms | 40.338ms | 57.578ms | 30.000ms | 50.42 MiB | none | 2.12M/s |
| q19 | discounted revenue | 5.000ms | 40.384ms | 40.559ms | 59.9% | 40.382ms | 64.684ms | 40.325ms | 73.780ms | 40.000ms | 46.48 MiB | none | 2.14M/s |
| q20 | potential part promotion | 8.000ms | 40.382ms | 40.639ms | 3.9% | 40.388ms | 41.970ms | 40.310ms | 70.932ms | 30.000ms | 50.95 MiB | none | 2.13M/s |
| q21 | suppliers who kept orders waiting | 7.000ms | 40.347ms | 40.406ms | 38.4% | 40.365ms | 55.883ms | 40.346ms | 60.716ms | 30.000ms | 48.65 MiB | none | 2.14M/s |
| q22 | global sales opportunity | 9.000ms | 59.562ms | 40.391ms | 54.7% | 40.342ms | 62.453ms | 40.337ms | 66.417ms | 40.000ms | 48.86 MiB | none | 2.15M/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 22 of 22 queries. Total 186.000ms by its own clock and 910.668ms by ours, 931.007ms cold, 760.000ms of CPU, peak 55.90 MiB, 10.25M/s and 339.97 MiB/s.

Running it cost 390% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.51x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 40.000ms | 100.546ms | 121.070ms | 17.9% | 101.628ms | 123.263ms | 100.773ms | 128.399ms | 120.000ms | 257.53 MiB | none | 715.63K/s |
| q02 | minimum cost supplier | 11.000ms | 80.836ms | 80.638ms | 0.2% | 80.547ms | 80.739ms | 80.510ms | 81.555ms | 90.000ms | 249.36 MiB | none | 1.07M/s |
| q03 | shipping priority | 42.000ms | 121.568ms | 101.079ms | 10.8% | 92.512ms | 103.419ms | 80.693ms | 140.853ms | 100.000ms | 256.25 MiB | none | 857.17K/s |
| q04 | order priority checking | 14.000ms | 100.688ms | 80.643ms | 25.3% | 80.598ms | 100.962ms | 80.564ms | 111.458ms | 90.000ms | 254.93 MiB | none | 1.07M/s |
| q05 | local supplier volume | 41.000ms | 124.367ms | 120.643ms | 17.5% | 100.636ms | 121.801ms | 80.496ms | 123.000ms | 100.000ms | 256.64 MiB | none | 718.17K/s |
| q06 | forecasting revenue change | 42.000ms | 120.742ms | 121.453ms | 18.0% | 101.061ms | 122.934ms | 100.925ms | 141.010ms | 120.000ms | 252.51 MiB | none | 713.38K/s |
| q07 | volume shipping | 52.000ms | 122.020ms | 120.800ms | 0.6% | 120.638ms | 121.317ms | 101.234ms | 122.223ms | 110.000ms | 258.08 MiB | none | 717.24K/s |
| q08 | national market share | 51.000ms | 141.143ms | 121.375ms | 2.1% | 120.774ms | 123.318ms | 101.010ms | 148.974ms | 100.000ms | 260.27 MiB | none | 713.84K/s |
| q09 | product type profit measure | 47.000ms | 121.007ms | 121.024ms | 17.0% | 100.585ms | 121.160ms | 100.535ms | 121.280ms | 100.000ms | 255.00 MiB | none | 715.91K/s |
| q10 | returned item reporting | 50.000ms | 140.873ms | 122.298ms | 16.0% | 121.476ms | 141.016ms | 121.312ms | 143.715ms | 100.000ms | 258.51 MiB | none | 708.45K/s |
| q11 | important stock identification | 6.000ms | 81.211ms | 60.491ms | 34.7% | 60.421ms | 81.397ms | 60.405ms | 100.883ms | 80.000ms | 248.73 MiB | none | 1.43M/s |
| q12 | shipping modes and order priority | 43.000ms | 142.989ms | 101.826ms | 39.0% | 101.338ms | 141.016ms | 100.552ms | 141.241ms | 100.000ms | 257.75 MiB | none | 850.89K/s |
| q13 | customer distribution | 12.000ms | 60.445ms | 81.743ms | 24.5% | 81.101ms | 101.097ms | 60.457ms | 121.523ms | 110.000ms | 252.19 MiB | none | 1.06M/s |
| q14 | promotion effect | 42.000ms | 120.967ms | 120.860ms | 55.6% | 100.629ms | 167.876ms | 100.519ms | 181.095ms | 120.000ms | 255.93 MiB | none | 716.88K/s |
| q15 | top supplier | 54.000ms | 122.146ms | 141.304ms | 0.6% | 141.000ms | 141.811ms | 127.210ms | 144.311ms | 120.000ms | 258.47 MiB | none | 613.16K/s |
| q16 | parts supplier relationship | 7.000ms | 60.689ms | 61.043ms | 65.9% | 60.533ms | 100.785ms | 60.402ms | 121.132ms | 80.000ms | 249.03 MiB | none | 1.42M/s |
| q17 | small quantity order revenue | 9.000ms | 102.915ms | 83.492ms | 48.3% | 60.515ms | 100.802ms | 60.447ms | 101.211ms | 90.000ms | 246.62 MiB | none | 1.04M/s |
| q18 | large volume customer | 10.000ms | 80.879ms | 80.513ms | 25.2% | 80.495ms | 100.820ms | 60.574ms | 121.251ms | 80.000ms | 253.52 MiB | none | 1.08M/s |
| q19 | discounted revenue | 42.000ms | 141.428ms | 100.651ms | 2.3% | 100.608ms | 102.952ms | 100.579ms | 161.323ms | 90.000ms | 252.19 MiB | none | 860.82K/s |
| q20 | potential part promotion | 45.000ms | 120.718ms | 122.078ms | 20.9% | 108.655ms | 134.149ms | 80.690ms | 152.034ms | 100.000ms | 257.98 MiB | none | 709.73K/s |
| q21 | suppliers who kept orders waiting | 82.000ms | 184.612ms | 163.538ms | 1.9% | 161.982ms | 165.138ms | 140.783ms | 180.930ms | 250.000ms | 426.12 MiB | none | 529.80K/s |
| q22 | global sales opportunity | 7.000ms | 100.880ms | 80.815ms | 4.8% | 80.617ms | 84.508ms | 60.442ms | 101.065ms | 90.000ms | 248.97 MiB | none | 1.07M/s |

clickhouse-local 26.9.1.1562 over 22 of 22 queries. Total 749.000ms by its own clock and 2.309s by ours, 2.494s cold, 2.340s of CPU, peak 426.12 MiB, 2.54M/s and 84.43 MiB/s.

Running it cost 208% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.70x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 13.000ms | 60.799ms | 40.633ms | 49.9% | 40.440ms | 60.721ms | 40.349ms | 66.725ms | 70.000ms | 150.65 MiB | none | 2.13M/s |
| q02 | minimum cost supplier | 17.000ms | 40.655ms | 40.399ms | 50.3% | 40.372ms | 60.702ms | 40.354ms | 66.761ms | 80.000ms | 195.15 MiB | none | 2.14M/s |
| q03 | shipping priority | 13.000ms | 40.413ms | 40.633ms | 50.5% | 40.392ms | 60.927ms | 40.356ms | 62.907ms | 50.000ms | 192.20 MiB | none | 2.13M/s |
| q04 | order priority checking | 10.000ms | 40.347ms | 49.288ms | 41.3% | 40.392ms | 60.742ms | 40.381ms | 75.573ms | 40.000ms | 152.55 MiB | none | 1.76M/s |
| q05 | local supplier volume | 16.000ms | 40.355ms | 60.878ms | 0.1% | 60.850ms | 60.901ms | 40.357ms | 63.202ms | 80.000ms | 189.51 MiB | none | 1.42M/s |
| q06 | forecasting revenue change | 6.000ms | 40.340ms | 20.293ms | 0.1% | 20.281ms | 20.300ms | 20.277ms | 40.433ms | 20.000ms | 143.80 MiB | none | 4.27M/s |
| q07 | volume shipping | 19.000ms | 60.662ms | 40.428ms | 50.5% | 40.387ms | 60.822ms | 40.376ms | 61.049ms | 110.000ms | 198.16 MiB | none | 2.14M/s |
| q08 | national market share | 17.000ms | 60.553ms | 60.768ms | 33.7% | 40.408ms | 60.912ms | 40.342ms | 80.957ms | 70.000ms | 190.37 MiB | none | 1.43M/s |
| q09 | product type profit measure | 23.000ms | 40.518ms | 60.800ms | 33.5% | 40.523ms | 60.899ms | 40.342ms | 60.920ms | 70.000ms | 198.04 MiB | none | 1.43M/s |
| q10 | returned item reporting | 23.000ms | 40.642ms | 60.775ms | 32.6% | 40.979ms | 60.819ms | 40.389ms | 62.375ms | 60.000ms | 205.90 MiB | none | 1.43M/s |
| q11 | important stock identification | 13.000ms | 40.364ms | 45.898ms | 38.5% | 43.204ms | 60.889ms | 40.402ms | 60.976ms | 30.000ms | 188.97 MiB | none | 1.89M/s |
| q12 | shipping modes and order priority | 12.000ms | 40.819ms | 40.381ms | 0.2% | 40.372ms | 40.435ms | 40.355ms | 60.829ms | 60.000ms | 169.85 MiB | none | 2.15M/s |
| q13 | customer distribution | 9.000ms | 40.507ms | 40.462ms | 3.4% | 40.339ms | 41.713ms | 20.256ms | 44.184ms | 30.000ms | 139.68 MiB | none | 2.14M/s |
| q14 | promotion effect | 8.000ms | 40.785ms | 40.352ms | 0.7% | 40.349ms | 40.643ms | 40.340ms | 47.700ms | 60.000ms | 148.85 MiB | none | 2.15M/s |
| q15 | top supplier | 12.000ms | 61.254ms | 40.402ms | 0.6% | 40.359ms | 40.593ms | 40.340ms | 40.705ms | 60.000ms | 171.08 MiB | none | 2.14M/s |
| q16 | parts supplier relationship | 11.000ms | 41.451ms | 40.525ms | 0.7% | 40.482ms | 40.749ms | 40.428ms | 61.011ms | 50.000ms | 191.37 MiB | none | 2.14M/s |
| q17 | small quantity order revenue | 12.000ms | 42.756ms | 40.450ms | 3.6% | 40.405ms | 41.860ms | 40.367ms | 62.705ms | 110.000ms | 182.42 MiB | none | 2.14M/s |
| q18 | large volume customer | 15.000ms | 40.439ms | 40.599ms | 0.8% | 40.476ms | 40.805ms | 40.315ms | 60.955ms | 110.000ms | 213.19 MiB | none | 2.13M/s |
| q19 | discounted revenue | 10.000ms | 40.483ms | 40.514ms | 19.9% | 40.375ms | 48.430ms | 40.335ms | 59.765ms | 40.000ms | 167.49 MiB | none | 2.14M/s |
| q20 | potential part promotion | 12.000ms | 40.492ms | 40.393ms | 0.3% | 40.375ms | 40.506ms | 40.342ms | 40.667ms | 60.000ms | 168.90 MiB | none | 2.14M/s |
| q21 | suppliers who kept orders waiting | 15.000ms | 40.428ms | 40.425ms | 2.0% | 40.420ms | 41.210ms | 40.406ms | 44.159ms | 70.000ms | 224.61 MiB | none | 2.14M/s |
| q22 | global sales opportunity | 9.000ms | 45.608ms | 40.376ms | 0.8% | 40.350ms | 40.666ms | 40.344ms | 60.490ms | 30.000ms | 148.19 MiB | none | 2.15M/s |

datafusion datafusion-cli 55.1.0 over 22 of 22 queries. Total 295.000ms by its own clock and 965.671ms by ours, 980.669ms cold, 1.360s of CPU, peak 224.61 MiB, 6.46M/s and 214.36 MiB/s.

Running it cost 227% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.00x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 17.830ms | 170.689ms | 144.566ms | 18.7% | 122.912ms | 150.017ms | 120.996ms | 164.510ms | 170.000ms | 88.35 MiB | none | 599.32K/s |
| q03 | shipping priority | 22.906ms | 140.883ms | 136.465ms | 13.8% | 126.152ms | 144.920ms | 121.230ms | 145.544ms | 180.000ms | 85.28 MiB | none | 634.90K/s |
| q04 | order priority checking | 17.174ms | 121.283ms | 142.977ms | 3.5% | 142.882ms | 147.930ms | 123.430ms | 166.454ms | 160.000ms | 82.31 MiB | none | 605.99K/s |
| q05 | local supplier volume | 38.821ms | 142.551ms | 161.586ms | 23.3% | 154.105ms | 191.785ms | 148.884ms | 225.196ms | 250.000ms | 88.17 MiB | none | 536.20K/s |
| q06 | forecasting revenue change | 14.070ms | 104.511ms | 141.318ms | 19.4% | 120.793ms | 148.240ms | 101.065ms | 164.880ms | 160.000ms | 72.07 MiB | none | 613.10K/s |
| q07 | volume shipping | 41.107ms | 225.577ms | 143.785ms | 42.2% | 141.290ms | 201.946ms | 140.865ms | 207.888ms | 250.000ms | 90.16 MiB | none | 602.58K/s |
| q08 | national market share | 42.445ms | 203.836ms | 141.537ms | 0.5% | 140.895ms | 141.616ms | 140.885ms | 164.510ms | 180.000ms | 89.59 MiB | none | 612.15K/s |
| q09 | product type profit measure | 42.946ms | 140.823ms | 160.990ms | 10.1% | 145.771ms | 162.070ms | 140.909ms | 162.420ms | 230.000ms | 94.93 MiB | none | 538.18K/s |
| q10 | returned item reporting | 37.939ms | 167.406ms | 149.260ms | 8.1% | 149.085ms | 161.189ms | 141.012ms | 162.209ms | 240.000ms | 88.84 MiB | none | 580.48K/s |
| q11 | important stock identification | 19.849ms | 142.692ms | 133.730ms | 6.4% | 132.512ms | 141.062ms | 121.967ms | 152.120ms | 170.000ms | 83.01 MiB | none | 647.89K/s |
| q12 | shipping modes and order priority | 17.913ms | 142.724ms | 147.005ms | 14.0% | 128.077ms | 148.632ms | 120.857ms | 161.219ms | 160.000ms | 84.04 MiB | none | 589.38K/s |
| q14 | promotion effect | 19.266ms | 130.009ms | 141.344ms | 28.8% | 141.073ms | 181.849ms | 120.763ms | 187.885ms | 200.000ms | 82.95 MiB | none | 612.99K/s |
| q15 | top supplier | 25.569ms | 141.031ms | 143.297ms | 28.4% | 141.037ms | 181.760ms | 121.042ms | 184.951ms | 180.000ms | 83.24 MiB | none | 604.63K/s |
| q16 | parts supplier relationship | 24.582ms | 120.667ms | 141.116ms | 55.0% | 124.984ms | 202.643ms | 121.264ms | 202.757ms | 200.000ms | 91.05 MiB | none | 613.98K/s |
| q18 | large volume customer | 33.432ms | 161.377ms | 147.401ms | 27.1% | 141.462ms | 181.456ms | 140.985ms | 203.412ms | 300.000ms | 100.22 MiB | none | 587.80K/s |
| q19 | discounted revenue | 15.142ms | 181.720ms | 121.091ms | 12.0% | 121.004ms | 135.514ms | 120.826ms | 161.598ms | 190.000ms | 81.59 MiB | none | 715.51K/s |
| q20 | potential part promotion | 24.509ms | 181.712ms | 145.814ms | 2.3% | 144.681ms | 148.102ms | 120.950ms | 181.552ms | 200.000ms | 92.23 MiB | none | 594.20K/s |
| q21 | suppliers who kept orders waiting | 66.288ms | 160.823ms | 181.087ms | 10.7% | 161.858ms | 181.175ms | 161.632ms | 184.306ms | 350.000ms | 127.34 MiB | none | 478.45K/s |
| q22 | global sales opportunity | 19.354ms | 141.692ms | 120.783ms | 0.9% | 120.740ms | 121.817ms | 120.622ms | 123.160ms | 140.000ms | 82.54 MiB | none | 717.34K/s |

polars 1.44.2 in sink mode over 19 of 22 queries. Total 541.142ms by its own clock and 2.745s by ours, 2.922s cold, 3.910s of CPU, peak 127.34 MiB, 3.04M/s and 100.92 MiB/s.

Running it cost 407% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.50x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 4.513ms | 20.383ms | 20.379ms | 0.3% | 20.339ms | 20.408ms | 20.334ms | 20.439ms | 0.000us | 14.71 MiB | none | 4.25M/s |
| q02 | minimum cost supplier | 1.839ms | 20.536ms | 20.626ms | 1.0% | 20.490ms | 20.690ms | 20.444ms | 20.775ms | 0.000us | 11.88 MiB | none | 4.20M/s |
| q03 | shipping priority | 3.248ms | 20.648ms | 20.418ms | 1.6% | 20.349ms | 20.668ms | 20.347ms | 20.699ms | 0.000us | 14.79 MiB | none | 4.24M/s |
| q04 | order priority checking | 2.108ms | 20.395ms | 20.360ms | 4.1% | 20.312ms | 21.139ms | 20.303ms | 25.811ms | 0.000us | 12.99 MiB | none | 4.26M/s |
| q05 | local supplier volume | 4.233ms | 20.635ms | 20.640ms | 1.0% | 20.499ms | 20.707ms | 20.388ms | 20.709ms | 0.000us | 15.67 MiB | none | 4.20M/s |
| q06 | forecasting revenue change | 1.873ms | 20.392ms | 20.340ms | 1.0% | 20.333ms | 20.534ms | 20.280ms | 20.594ms | 0.000us | 13.35 MiB | none | 4.26M/s |
| q07 | volume shipping | 3.647ms | 20.388ms | 20.655ms | 2.5% | 20.437ms | 20.959ms | 7.020ms | 21.400ms | 0.000us | 15.27 MiB | none | 4.19M/s |
| q08 | national market share | 3.466ms | 20.376ms | 20.460ms | 1.1% | 20.457ms | 20.686ms | 20.425ms | 20.751ms | 0.000us | 14.78 MiB | none | 4.23M/s |
| q09 | product type profit measure | 5.980ms | 20.518ms | 20.483ms | 0.7% | 20.470ms | 20.621ms | 20.431ms | 20.757ms | 0.000us | 16.73 MiB | none | 4.23M/s |
| q10 | returned item reporting | 3.899ms | 20.369ms | 20.395ms | 0.1% | 20.382ms | 20.403ms | 20.342ms | 24.060ms | 0.000us | 15.74 MiB | none | 4.25M/s |
| q11 | important stock identification | 1.696ms | 20.743ms | 20.629ms | 1.2% | 20.428ms | 20.672ms | 20.413ms | 20.838ms | 0.000us | 12.00 MiB | none | 4.20M/s |
| q12 | shipping modes and order priority | 3.598ms | 20.408ms | 20.433ms | 0.6% | 20.380ms | 20.504ms | 20.323ms | 20.597ms | 0.000us | 14.02 MiB | none | 4.24M/s |
| q13 | customer distribution | 5.305ms | 20.470ms | 20.365ms | 0.1% | 20.353ms | 20.375ms | 20.337ms | 20.415ms | 0.000us | 14.54 MiB | none | 4.25M/s |
| q14 | promotion effect | 2.291ms | 20.375ms | 20.393ms | 0.2% | 20.372ms | 20.421ms | 20.367ms | 20.610ms | 0.000us | 14.00 MiB | none | 4.25M/s |
| q15 | top supplier | 3.198ms | 20.350ms | 20.709ms | 0.5% | 20.654ms | 20.749ms | 20.426ms | 23.388ms | 0.000us | 14.48 MiB | none | 4.18M/s |
| q16 | parts supplier relationship | 1.458ms | 20.669ms | 20.380ms | 0.7% | 20.365ms | 20.516ms | 20.351ms | 20.726ms | 0.000us | 11.16 MiB | none | 4.25M/s |
| q17 | small quantity order revenue | 2.474ms | 20.517ms | 20.596ms | 0.9% | 20.522ms | 20.712ms | 20.338ms | 20.739ms | 0.000us | 13.68 MiB | none | 4.21M/s |
| q18 | large volume customer | 5.867ms | 20.717ms | 20.477ms | 0.6% | 20.436ms | 20.565ms | 20.426ms | 20.740ms | 0.000us | 15.27 MiB | none | 4.23M/s |
| q19 | discounted revenue | 2.674ms | 20.302ms | 20.679ms | 0.7% | 20.589ms | 20.743ms | 20.573ms | 20.802ms | 0.000us | 14.23 MiB | none | 4.19M/s |
| q20 | potential part promotion | 3.192ms | 20.444ms | 20.697ms | 1.3% | 20.485ms | 20.762ms | 20.451ms | 20.934ms | 0.000us | 14.23 MiB | none | 4.19M/s |
| q21 | suppliers who kept orders waiting | 4.262ms | 22.979ms | 20.676ms | 1.1% | 20.619ms | 20.856ms | 20.495ms | 21.796ms | 0.000us | 13.52 MiB | none | 4.19M/s |
| q22 | global sales opportunity | 1.830ms | 20.464ms | 20.354ms | 2.0% | 20.353ms | 20.768ms | 20.336ms | 22.992ms | 0.000us | 11.70 MiB | none | 4.26M/s |

rudb rudb 0.3.67 over 22 of 22 queries. Total 72.650ms by its own clock and 451.144ms by ours, 453.078ms cold, 0.000us of CPU, peak 16.73 MiB, 26.24M/s and 870.40 MiB/s.

Running it cost 521% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.02x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | 344.277us | 4.065ms | 4.046ms | 4.065ms | 0.5% | 4.046ms | 121.324us | 0.000us | 6.64 KiB | 6 of 6 |
| q02 | 681.412us | 919.971us | 884.601us | 919.983us | 3.8% | 884.601us | 214.481us | 0.000us | 63.47 KiB | 47 of 47 |
| q03 | 377.976us | 3.085ms | 3.064ms | 3.085ms | 0.7% | 3.064ms | 132.113us | 0.000us | 271.27 KiB | 16 of 16 |
| q04 | 329.664us | 1.682ms | 1.662ms | 1.682ms | 1.2% | 1.662ms | 121.151us | 0.000us | 192.33 KiB | 11 of 11 |
| q05 | 445.371us | 3.417ms | 3.393ms | 3.417ms | 0.7% | 3.393ms | 136.078us | 0.000us | 719.01 KiB | 27 of 27 |
| q06 | 230.545us | 1.593ms | 1.581ms | 1.593ms | 0.8% | 1.581ms | 87.932us | 0.000us | 360 B | 5 of 5 |
| q07 | 554.623us | 2.678ms | 2.656ms | 2.678ms | 0.8% | 2.656ms | 150.685us | 0.000us | 154.25 KiB | 30 of 30 |
| q08 | 677.049us | 2.477ms | 2.446ms | 2.477ms | 1.3% | 2.446ms | 169.188us | 0.000us | 119.88 KiB | 37 of 37 |
| q09 | 517.900us | 5.560ms | 5.529ms | 5.560ms | 0.6% | 5.529ms | 166.728us | 0.000us | 1.73 MiB | 27 of 27 |
| q10 | 469.943us | 3.309ms | 3.285ms | 3.309ms | 0.7% | 3.285ms | 140.718us | 0.000us | 783.50 KiB | 19 of 19 |
| q11 | 490.005us | 913.247us | 892.225us | 913.253us | 2.3% | 892.225us | 175.978us | 0.000us | 166.50 KiB | 30 of 30 |
| q12 | 360.240us | 3.065ms | 3.045ms | 3.065ms | 0.7% | 3.045ms | 110.672us | 0.000us | 510.81 KiB | 10 of 10 |
| q13 | 305.246us | 4.461ms | 4.449ms | 4.461ms | 0.3% | 4.449ms | 132.798us | 0.000us | 349.69 KiB | 12 of 12 |
| q14 | 329.270us | 2.014ms | 1.993ms | 2.014ms | 1.1% | 1.993ms | 108.612us | 0.000us | 561.86 KiB | 9 of 9 |
| q15 | 506.458us | 2.523ms | 2.499ms | 2.523ms | 1.0% | 2.499ms | 186.063us | 0.000us | 135.00 KiB | 21 of 21 |
| q16 | 437.767us | 777.119us | 758.321us | 777.103us | 2.4% | 758.321us | 162.568us | 0.000us | 305.55 KiB | 18 of 18 |
| q17 | 434.517us | 1.830ms | 1.801ms | 1.830ms | 1.6% | 1.801ms | 176.029us | 0.000us | 872 B | 21 of 21 |
| q18 | 490.149us | 5.053ms | 5.010ms | 5.045ms | 0.7% | 5.010ms | 184.506us | 0.000us | 1.74 MiB | 21 of 21 |
| q19 | 530.911us | 1.934ms | 1.916ms | 1.934ms | 1.0% | 1.916ms | 134.440us | 0.000us | 18.73 KiB | 11 of 11 |
| q20 | 576.066us | 2.630ms | 2.602ms | 2.630ms | 1.1% | 2.602ms | 160.467us | 0.000us | 1.05 MiB | 30 of 30 |
| q21 | 599.659us | 3.366ms | 3.331ms | 3.366ms | 1.0% | 3.331ms | 153.149us | 0.000us | 182.09 KiB | 29 of 29 |
| q22 | 485.087us | 1.188ms | 1.172ms | 1.188ms | 1.4% | 1.172ms | 162.127us | 0.000us | 236.23 KiB | 19 of 19 |

Read from the breakdown the engine wrote for its cold run. `planning` is everything before the first row moved, which is the parse, the bind, the optimizer passes and building the tree. It is a column rather than a footnote because it is the one cost of a query nobody profiles: an optimizer only ever has passes added to it, each paying for itself on the query it was written for, and a query that plans for four hundred milliseconds to save two hundred is a query the optimizer made slower. What is gated is planning as a share of the whole statement, and that share is not in this table, because it is taken over every run rather than off the cold one the rest of these columns come from. It lives in `baselines/planning-<suite>.txt`. `accounted` is what its pipelines charged themselves and `measured` is what it measured around running them, so `apart` is the cross check and anything over five percent is time the breakdown cannot explain. `driver` is the part of `accounted` that was not inside an operator, which is the loop that runs a pipeline rather than the operators it calls. `build` is what the engine spent putting the tree together, which is before there is a pipeline to charge and is why it is off the right hand side of the check. `outside` is the CPU the process spent on everything else, which is starting, opening the data and printing the answer, and it is the reason the wall clock column and the query time column differ. `held` is what the operators reserved, which is not the resident set of the process. `reference` is how many operators ran the reference implementation of their seam, which is the slow path kept for differential testing.

#### What the planner knew

| query | decisions | exact | certified | estimated | unknown |
| --- | --- | --- | --- | --- | --- |
| q01 | 6 | 1 | 0 | 5 | 0 |
| q02 | 47 | 16 | 0 | 22 | 9 |
| q03 | 16 | 3 | 0 | 11 | 2 |
| q04 | 11 | 2 | 0 | 8 | 1 |
| q05 | 27 | 10 | 0 | 12 | 5 |
| q06 | 5 | 3 | 0 | 2 | 0 |
| q07 | 30 | 9 | 0 | 16 | 5 |
| q08 | 37 | 13 | 0 | 17 | 7 |
| q09 | 27 | 11 | 0 | 11 | 5 |
| q10 | 19 | 6 | 0 | 10 | 3 |
| q11 | 30 | 12 | 0 | 13 | 5 |
| q12 | 10 | 3 | 0 | 6 | 1 |
| q13 | 12 | 3 | 0 | 8 | 1 |
| q14 | 9 | 5 | 0 | 3 | 1 |
| q15 | 21 | 6 | 0 | 13 | 2 |
| q16 | 18 | 4 | 0 | 12 | 2 |
| q17 | 21 | 8 | 0 | 10 | 3 |
| q18 | 21 | 8 | 0 | 10 | 3 |
| q19 | 11 | 4 | 0 | 6 | 1 |
| q20 | 30 | 7 | 0 | 19 | 4 |
| q21 | 29 | 8 | 0 | 16 | 5 |
| q22 | 19 | 6 | 0 | 11 | 2 |
| whole suite | 456 | 148 | 0 | 241 | 67 |

One row per query and one decision per operator, counted by what the planner had behind the cardinality it used. Over the whole suite that is 32% exact, 0% certified, 53% estimated and 15% unknown. `exact` is a counted number, `certified` is a number with a proven bound, `estimated` is a guess, and `unknown` is no number at all, which is an answer an operator has to handle rather than a null to paper over. Counts rather than fractions in the table because a suite's histogram is the sum of its queries' and fractions do not add.

#### Where the time went

| kind | cpu | share | operators | rows in | rows out | per row in | per row out | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Aggregate | 0.000us | nothing to share | 29 | 160713 | 23710 | 0.0ns | 0.0ns | 29 of 29 |
| FileScan | 0.000us | nothing to share | 89 | 0 | 639332 | handed none | 0.0ns | 89 of 89 |
| Filter | 0.000us | nothing to share | 54 | 535105 | 104878 | 0.0ns | 0.0ns | 54 of 54 |
| Gather | 0.000us | nothing to share | 67 | 32175 | 0 | 0.0ns | handed on none | 67 of 67 |
| Join | 0.000us | nothing to share | 3 | 903 | 903 | 0.0ns | 0.0ns | 3 of 3 |
| Mark | 0.000us | nothing to share | 5 | 8234 | 1015 | 0.0ns | 0.0ns | 5 of 5 |
| Pad | 0.000us | nothing to share | 1 | 14823 | 15323 | 0.0ns | 0.0ns | 1 of 1 |
| Probe | 0.000us | nothing to share | 58 | 78490 | 46721 | 0.0ns | 0.0ns | 58 of 58 |
| Project | 0.000us | nothing to share | 132 | 236718 | 236718 | 0.0ns | 0.0ns | 132 of 132 |
| Sort | 0.000us | nothing to share | 13 | 891 | 891 | 0.0ns | 0.0ns | 13 of 13 |
| TopN | 0.000us | nothing to share | 5 | 544 | 37 | 0.0ns | 0.0ns | 5 of 5 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is Aggregate, and the queries where it cost the most are q01 at 0.000us, q02 at 0.000us, q03 at 0.000us.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- every query here ran within 1.01x of every other one, so most of what was timed is whatever they have in common rather than the queries

These swung wider than reporting rule two allows:

- duckdb-pinned swung by 59.9% of its median on q19, and rule two wants under 10%
- clickhouse-local swung by 65.9% of its median on q16, and rule two wants under 10%
- datafusion swung by 50.5% of its median on q07, and rule two wants under 10%
- polars swung by 55.0% of its median on q16, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

These ran every query at about the same speed:

- duckdb ran every query within 1.01x of every other one, and this suite spreads over 6x on an engine it is measuring
- duckdb-pinned ran every query within 1.51x of every other one, and this suite spreads over 6x on an engine it is measuring
- polars ran every query within 1.50x of every other one, and this suite spreads over 6x on an engine it is measuring
- rudb ran every query within 1.02x of every other one, and this suite spreads over 6x on an engine it is measuring

A column that flat is not a column about the queries. Read those numbers as an upper bound on the engine and not as a measurement of it.

polars did not run q02, because the SQLContext does not resolve the correlated p_partkey in the min supplycost subquery.

polars did not run q13, because the SQLContext rejects a NOT LIKE inside a join constraint.

polars did not run q17, because the SQLContext does not resolve the correlated p_partkey in the average quantity subquery.

So the polars column is 19 of 22 queries and its ratio is over the shared ones.

Answers differ, so this is not a comparison: q01: polars does not agree with duckdb: 32 numbers against 32

Answers differ, so this is not a comparison: q03: polars does not agree with duckdb: 30 numbers against 30

Answers differ, so this is not a comparison: q05: polars does not agree with duckdb: 5 numbers against 5

Answers differ, so this is not a comparison: q06: polars does not agree with duckdb: 1 numbers against 1

Answers differ, so this is not a comparison: q07: polars does not agree with duckdb: 8 numbers against 8

Answers differ, so this is not a comparison: q09: polars does not agree with duckdb: 346 numbers against 346

Answers differ, so this is not a comparison: q10: polars does not agree with duckdb: 60 numbers against 60

Answers differ, so this is not a comparison: q15: polars does not agree with duckdb: 2 numbers against 2

Answers differ, so this is not a comparison: q17: clickhouse-local does not agree with duckdb: the same 0 numbers and 1 of 1 text fields different, so the disagreement is which rows came back rather than what they hold

Hot here means page cache warm and not buffer pool warm, because every run is a fresh process so that no query's number depends on the one before it.

