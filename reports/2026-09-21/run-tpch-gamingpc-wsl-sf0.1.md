# tpch on gamingpc-wsl

This is one run of the tpch suite on gamingpc-wsl, over 6 engines and 22 queries, with 5 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

## What ran

| what | it was |
| --- | --- |
| suite | tpch |
| queries | 22 |
| tables | 29.13 MiB of Parquet in 8 tables |
| rows | 866151 in the table every query reads |
| corpus | duckdb-tpch SF0.1, corpus ea3327af3c635851, written 2026-09-21 |
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
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 430.165ms | 870.000ms | 25.76 MiB | its own database file | its own | 9.59 to 9.06 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 398.684ms | 830.000ms | 27.51 MiB | its own database file | its own | 9.06 to 8.58 |
| clickhouse-local | 26.9.1.1562 | ran | 849.088ms | 920.000ms | 45.29 MiB | its own MergeTree parts, as system.parts counts the active ones | its own | 8.58 to 8.78 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 29.13 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 8.78 to 8.08 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 29.13 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 8.08 to 7.06 |
| rudb | rudb 0.3.67 | ran | 0.000us | 0.000us | 29.13 MiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 7.06 to 7.06 |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb on 19 shared |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 192.000ms | 526.575ms | +174% | 528.065ms | 540.000ms | 1.03 | 126.07 MiB | none | 99.25M/s | 3.26 GiB/s | 1.00x |
| duckdb-pinned | 265.000ms | 949.664ms | +258% | 971.456ms | 930.000ms | 0.98 | 132.22 MiB | none | 71.91M/s | 2.36 GiB/s | 1.43x |
| clickhouse-local | 1.252s | 2.750s | +120% | 2.701s | 3.670s | 1.33 | 486.86 MiB | none | 15.22M/s | 511.91 MiB/s | 6.88x |
| datafusion | 365.000ms | 908.707ms | +149% | 909.081ms | 1.650s | 1.82 | 439.23 MiB | none | 52.21M/s | 1.71 GiB/s | 1.91x |
| polars | 748.537ms | 2.583s | +245% | 2.601s | 4.360s | 1.69 | 419.36 MiB | none | 21.99M/s | 739.46 MiB/s | 4.56x |
| rudb | 270.865ms | 528.869ms | +95% | 572.544ms | 470.000ms | 0.89 | 68.20 MiB | none | 70.35M/s | 2.31 GiB/s | 1.35x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 22 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 6.000ms | 15.000ms | 69.000ms | 14.000ms | 41.727ms | 9.330ms |
| q02 | minimum cost supplier | 7.000ms | 7.000ms | 17.000ms | 18.000ms | no dialect | 7.014ms |
| q03 | shipping priority | 9.000ms | 11.000ms | 60.000ms | 15.000ms | 32.344ms | 10.799ms |
| q04 | order priority checking | 7.000ms | 14.000ms | 52.000ms | 11.000ms | 21.011ms | 7.741ms |
| q05 | local supplier volume | 9.000ms | 13.000ms | 60.000ms | 17.000ms | 46.970ms | 13.412ms |
| q06 | forecasting revenue change | 2.000ms | 3.000ms | 60.000ms | 8.000ms | 13.497ms | 3.547ms |
| q07 | volume shipping | 8.000ms | 19.000ms | 63.000ms | 22.000ms | 54.523ms | 10.938ms |
| q08 | national market share | 8.000ms | 11.000ms | 79.000ms | 19.000ms | 54.983ms | 9.496ms |
| q09 | product type profit measure | 14.000ms | 14.000ms | 90.000ms | 28.000ms | 73.061ms | 25.476ms |
| q10 | returned item reporting | 27.000ms | 28.000ms | 51.000ms | 20.000ms | 44.245ms | 16.141ms |
| q11 | important stock identification | 6.000ms | 9.000ms | 11.000ms | 15.000ms | 25.242ms | 7.545ms |
| q12 | shipping modes and order priority | 6.000ms | 11.000ms | 75.000ms | 14.000ms | 24.183ms | 13.122ms |
| q13 | customer distribution | 15.000ms | 15.000ms | 50.000ms | 18.000ms | no dialect | 36.609ms |
| q14 | promotion effect | 4.000ms | 6.000ms | 66.000ms | 10.000ms | 15.278ms | 7.444ms |
| q15 | top supplier | 5.000ms | 10.000ms | 69.000ms | 14.000ms | 20.127ms | 9.066ms |
| q16 | parts supplier relationship | 8.000ms | 21.000ms | 11.000ms | 16.000ms | 28.919ms | 7.106ms |
| q17 | small quantity order revenue | 6.000ms | 8.000ms | 56.000ms | 16.000ms | no dialect | 6.084ms |
| q18 | large volume customer | 9.000ms | 10.000ms | 50.000ms | 24.000ms | 56.904ms | 22.546ms |
| q19 | discounted revenue | 5.000ms | 8.000ms | 62.000ms | 13.000ms | 18.706ms | 7.030ms |
| q20 | potential part promotion | 9.000ms | 11.000ms | 68.000ms | 18.000ms | 30.549ms | 14.267ms |
| q21 | suppliers who kept orders waiting | 16.000ms | 12.000ms | 123.000ms | 23.000ms | 124.899ms | 16.799ms |
| q22 | global sales opportunity | 6.000ms | 9.000ms | 10.000ms | 12.000ms | 21.369ms | 9.353ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 6.000ms | 20.229ms | 20.263ms | 0.1% | 20.258ms | 20.278ms | 20.257ms | 20.388ms | 20.000ms | 41.95 MiB | none | 42.74M/s |
| q02 | minimum cost supplier | 7.000ms | 20.248ms | 20.268ms | 0.1% | 20.262ms | 20.282ms | 20.259ms | 20.315ms | 20.000ms | 45.14 MiB | none | 42.74M/s |
| q03 | shipping priority | 9.000ms | 20.279ms | 20.260ms | 0.1% | 20.247ms | 20.264ms | 20.245ms | 20.325ms | 30.000ms | 59.51 MiB | none | 42.75M/s |
| q04 | order priority checking | 7.000ms | 20.241ms | 20.259ms | 0.2% | 20.249ms | 20.292ms | 20.244ms | 20.302ms | 20.000ms | 52.46 MiB | none | 42.75M/s |
| q05 | local supplier volume | 9.000ms | 20.260ms | 20.254ms | 0.1% | 20.249ms | 20.269ms | 20.247ms | 20.321ms | 20.000ms | 60.64 MiB | none | 42.77M/s |
| q06 | forecasting revenue change | 2.000ms | 20.249ms | 20.266ms | 0.0% | 20.261ms | 20.270ms | 20.255ms | 20.295ms | 10.000ms | 37.75 MiB | none | 42.74M/s |
| q07 | volume shipping | 8.000ms | 20.292ms | 20.252ms | 0.1% | 20.250ms | 20.280ms | 20.249ms | 20.283ms | 30.000ms | 53.64 MiB | none | 42.77M/s |
| q08 | national market share | 8.000ms | 20.246ms | 20.256ms | 0.0% | 20.249ms | 20.259ms | 20.238ms | 20.281ms | 20.000ms | 50.70 MiB | none | 42.76M/s |
| q09 | product type profit measure | 14.000ms | 40.353ms | 40.490ms | 0.5% | 40.403ms | 40.614ms | 40.320ms | 40.708ms | 40.000ms | 68.57 MiB | none | 21.39M/s |
| q10 | returned item reporting | 27.000ms | 40.383ms | 40.366ms | 0.2% | 40.345ms | 40.424ms | 40.328ms | 40.475ms | 70.000ms | 126.07 MiB | none | 21.46M/s |
| q11 | important stock identification | 6.000ms | 20.385ms | 20.355ms | 0.1% | 20.341ms | 20.356ms | 20.312ms | 20.412ms | 10.000ms | 42.88 MiB | none | 42.55M/s |
| q12 | shipping modes and order priority | 6.000ms | 20.234ms | 20.271ms | 0.2% | 20.267ms | 20.301ms | 20.267ms | 20.311ms | 20.000ms | 46.70 MiB | none | 42.73M/s |
| q13 | customer distribution | 15.000ms | 40.457ms | 40.330ms | 0.3% | 40.314ms | 40.417ms | 40.302ms | 40.604ms | 30.000ms | 48.89 MiB | none | 21.48M/s |
| q14 | promotion effect | 4.000ms | 20.369ms | 20.255ms | 0.7% | 20.243ms | 20.376ms | 20.233ms | 20.407ms | 10.000ms | 43.07 MiB | none | 42.76M/s |
| q15 | top supplier | 5.000ms | 20.383ms | 20.250ms | 0.5% | 20.230ms | 20.332ms | 20.227ms | 20.368ms | 10.000ms | 43.64 MiB | none | 42.77M/s |
| q16 | parts supplier relationship | 8.000ms | 20.362ms | 20.338ms | 0.2% | 20.337ms | 20.369ms | 20.333ms | 20.505ms | 20.000ms | 55.95 MiB | none | 42.59M/s |
| q17 | small quantity order revenue | 6.000ms | 20.237ms | 20.268ms | 0.3% | 20.236ms | 20.290ms | 20.234ms | 20.359ms | 20.000ms | 50.32 MiB | none | 42.73M/s |
| q18 | large volume customer | 9.000ms | 20.377ms | 20.387ms | 0.1% | 20.383ms | 20.399ms | 20.379ms | 20.437ms | 40.000ms | 59.33 MiB | none | 42.49M/s |
| q19 | discounted revenue | 5.000ms | 20.375ms | 20.237ms | 0.0% | 20.236ms | 20.240ms | 20.230ms | 20.260ms | 20.000ms | 45.51 MiB | none | 42.80M/s |
| q20 | potential part promotion | 9.000ms | 20.230ms | 20.374ms | 0.0% | 20.370ms | 20.376ms | 20.246ms | 20.378ms | 30.000ms | 53.43 MiB | none | 42.51M/s |
| q21 | suppliers who kept orders waiting | 16.000ms | 41.483ms | 40.342ms | 0.5% | 40.339ms | 40.531ms | 40.338ms | 40.999ms | 40.000ms | 62.64 MiB | none | 21.47M/s |
| q22 | global sales opportunity | 6.000ms | 20.394ms | 20.234ms | 0.1% | 20.229ms | 20.258ms | 20.228ms | 20.278ms | 10.000ms | 43.19 MiB | none | 42.81M/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 22 of 22 queries. Total 192.000ms by its own clock and 526.575ms by ours, 528.065ms cold, 540.000ms of CPU, peak 126.07 MiB, 99.25M/s and 3.26 GiB/s.

Running it cost 174% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.00x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 15.000ms | 40.421ms | 40.392ms | 0.3% | 40.328ms | 40.448ms | 40.314ms | 40.645ms | 50.000ms | 51.89 MiB | none | 21.44M/s |
| q02 | minimum cost supplier | 7.000ms | 40.312ms | 40.394ms | 0.4% | 40.328ms | 40.470ms | 40.323ms | 43.100ms | 30.000ms | 49.45 MiB | none | 21.44M/s |
| q03 | shipping priority | 11.000ms | 40.328ms | 40.598ms | 0.4% | 40.463ms | 40.620ms | 40.456ms | 40.661ms | 30.000ms | 66.27 MiB | none | 21.33M/s |
| q04 | order priority checking | 14.000ms | 40.483ms | 40.332ms | 0.5% | 40.325ms | 40.544ms | 40.314ms | 60.536ms | 50.000ms | 66.89 MiB | none | 21.48M/s |
| q05 | local supplier volume | 13.000ms | 40.454ms | 40.468ms | 0.5% | 40.457ms | 40.673ms | 40.338ms | 44.276ms | 40.000ms | 68.75 MiB | none | 21.40M/s |
| q06 | forecasting revenue change | 3.000ms | 40.467ms | 40.409ms | 0.6% | 40.375ms | 40.599ms | 40.308ms | 41.109ms | 30.000ms | 47.02 MiB | none | 21.43M/s |
| q07 | volume shipping | 19.000ms | 40.334ms | 60.378ms | 33.7% | 40.371ms | 60.693ms | 40.326ms | 61.415ms | 60.000ms | 60.49 MiB | none | 14.35M/s |
| q08 | national market share | 11.000ms | 40.438ms | 40.317ms | 0.1% | 40.312ms | 40.342ms | 40.308ms | 40.376ms | 40.000ms | 57.92 MiB | none | 21.48M/s |
| q09 | product type profit measure | 14.000ms | 60.560ms | 40.459ms | 0.1% | 40.426ms | 40.479ms | 40.354ms | 40.533ms | 60.000ms | 72.44 MiB | none | 21.41M/s |
| q10 | returned item reporting | 28.000ms | 60.401ms | 60.448ms | 3.8% | 60.438ms | 62.723ms | 60.412ms | 62.985ms | 70.000ms | 132.22 MiB | none | 14.33M/s |
| q11 | important stock identification | 9.000ms | 40.454ms | 40.451ms | 0.4% | 40.441ms | 40.614ms | 40.386ms | 40.699ms | 40.000ms | 48.70 MiB | none | 21.41M/s |
| q12 | shipping modes and order priority | 11.000ms | 40.728ms | 40.447ms | 0.9% | 40.401ms | 40.754ms | 40.330ms | 42.721ms | 40.000ms | 55.55 MiB | none | 21.41M/s |
| q13 | customer distribution | 15.000ms | 41.308ms | 40.424ms | 0.2% | 40.371ms | 40.465ms | 40.332ms | 40.775ms | 50.000ms | 59.67 MiB | none | 21.43M/s |
| q14 | promotion effect | 6.000ms | 41.045ms | 40.526ms | 2.1% | 40.404ms | 41.254ms | 40.317ms | 42.810ms | 30.000ms | 51.30 MiB | none | 21.37M/s |
| q15 | top supplier | 10.000ms | 60.555ms | 40.392ms | 1.5% | 40.324ms | 40.911ms | 40.300ms | 40.918ms | 30.000ms | 53.47 MiB | none | 21.44M/s |
| q16 | parts supplier relationship | 21.000ms | 60.646ms | 60.573ms | 22.4% | 47.064ms | 60.657ms | 44.388ms | 60.669ms | 60.000ms | 57.20 MiB | none | 14.30M/s |
| q17 | small quantity order revenue | 8.000ms | 40.380ms | 40.397ms | 0.2% | 40.335ms | 40.435ms | 40.328ms | 40.558ms | 40.000ms | 56.70 MiB | none | 21.44M/s |
| q18 | large volume customer | 10.000ms | 40.570ms | 40.496ms | 0.1% | 40.491ms | 40.516ms | 40.331ms | 41.034ms | 40.000ms | 68.68 MiB | none | 21.39M/s |
| q19 | discounted revenue | 8.000ms | 40.313ms | 40.397ms | 0.3% | 40.350ms | 40.462ms | 40.313ms | 40.463ms | 30.000ms | 53.99 MiB | none | 21.44M/s |
| q20 | potential part promotion | 11.000ms | 40.325ms | 40.496ms | 0.8% | 40.343ms | 40.658ms | 40.322ms | 41.769ms | 40.000ms | 58.43 MiB | none | 21.39M/s |
| q21 | suppliers who kept orders waiting | 12.000ms | 40.479ms | 40.461ms | 0.4% | 40.327ms | 40.495ms | 40.323ms | 40.518ms | 40.000ms | 57.40 MiB | none | 21.41M/s |
| q22 | global sales opportunity | 9.000ms | 40.456ms | 40.410ms | 0.5% | 40.322ms | 40.530ms | 40.299ms | 40.646ms | 30.000ms | 51.44 MiB | none | 21.43M/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 22 of 22 queries. Total 265.000ms by its own clock and 949.664ms by ours, 971.456ms cold, 930.000ms of CPU, peak 132.22 MiB, 71.91M/s and 2.36 GiB/s.

Running it cost 258% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.50x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 69.000ms | 143.073ms | 144.188ms | 1.7% | 141.711ms | 144.218ms | 141.159ms | 144.788ms | 280.000ms | 308.85 MiB | none | 6.01M/s |
| q02 | minimum cost supplier | 17.000ms | 80.808ms | 80.800ms | 8.8% | 80.660ms | 87.805ms | 80.492ms | 101.063ms | 110.000ms | 255.93 MiB | none | 10.72M/s |
| q03 | shipping priority | 60.000ms | 121.200ms | 122.045ms | 16.5% | 120.666ms | 140.750ms | 120.585ms | 141.493ms | 160.000ms | 294.23 MiB | none | 7.10M/s |
| q04 | order priority checking | 52.000ms | 100.565ms | 120.837ms | 0.1% | 120.788ms | 120.861ms | 100.619ms | 121.046ms | 140.000ms | 278.74 MiB | none | 7.17M/s |
| q05 | local supplier volume | 60.000ms | 121.903ms | 121.020ms | 16.3% | 121.008ms | 140.683ms | 120.959ms | 143.592ms | 140.000ms | 279.27 MiB | none | 7.16M/s |
| q06 | forecasting revenue change | 60.000ms | 140.778ms | 120.869ms | 0.4% | 120.851ms | 121.351ms | 120.670ms | 140.768ms | 140.000ms | 282.95 MiB | none | 7.17M/s |
| q07 | volume shipping | 63.000ms | 128.279ms | 141.280ms | 19.0% | 121.444ms | 148.264ms | 120.987ms | 163.315ms | 140.000ms | 290.16 MiB | none | 6.13M/s |
| q08 | national market share | 79.000ms | 143.007ms | 141.171ms | 5.2% | 141.158ms | 148.539ms | 140.837ms | 160.977ms | 180.000ms | 292.65 MiB | none | 6.14M/s |
| q09 | product type profit measure | 90.000ms | 160.814ms | 160.862ms | 0.4% | 160.785ms | 161.365ms | 140.716ms | 161.690ms | 210.000ms | 352.32 MiB | none | 5.38M/s |
| q10 | returned item reporting | 51.000ms | 120.688ms | 120.908ms | 0.2% | 120.788ms | 120.985ms | 101.176ms | 121.331ms | 160.000ms | 289.62 MiB | none | 7.16M/s |
| q11 | important stock identification | 11.000ms | 60.604ms | 80.687ms | 0.8% | 80.649ms | 81.281ms | 80.514ms | 100.748ms | 100.000ms | 252.43 MiB | none | 10.73M/s |
| q12 | shipping modes and order priority | 75.000ms | 140.846ms | 141.585ms | 0.5% | 140.954ms | 141.650ms | 120.801ms | 161.186ms | 200.000ms | 293.05 MiB | none | 6.12M/s |
| q13 | customer distribution | 50.000ms | 101.026ms | 120.779ms | 16.7% | 100.619ms | 120.791ms | 100.588ms | 121.067ms | 110.000ms | 268.95 MiB | none | 7.17M/s |
| q14 | promotion effect | 66.000ms | 121.174ms | 140.886ms | 0.4% | 140.692ms | 141.218ms | 120.638ms | 164.000ms | 130.000ms | 289.86 MiB | none | 6.15M/s |
| q15 | top supplier | 69.000ms | 140.756ms | 141.163ms | 0.2% | 140.985ms | 141.261ms | 140.826ms | 141.290ms | 170.000ms | 300.34 MiB | none | 6.14M/s |
| q16 | parts supplier relationship | 11.000ms | 61.899ms | 80.776ms | 0.1% | 80.710ms | 80.779ms | 80.569ms | 81.016ms | 100.000ms | 252.31 MiB | none | 10.72M/s |
| q17 | small quantity order revenue | 56.000ms | 120.800ms | 121.076ms | 0.3% | 120.852ms | 121.199ms | 101.503ms | 141.570ms | 130.000ms | 282.78 MiB | none | 7.15M/s |
| q18 | large volume customer | 50.000ms | 120.728ms | 121.132ms | 18.0% | 121.117ms | 142.979ms | 120.826ms | 143.990ms | 190.000ms | 300.10 MiB | none | 7.15M/s |
| q19 | discounted revenue | 62.000ms | 141.926ms | 121.331ms | 4.5% | 121.102ms | 126.570ms | 120.687ms | 140.667ms | 190.000ms | 289.79 MiB | none | 7.14M/s |
| q20 | potential part promotion | 68.000ms | 140.834ms | 141.609ms | 0.4% | 141.273ms | 141.900ms | 140.967ms | 161.906ms | 150.000ms | 284.99 MiB | none | 6.12M/s |
| q21 | suppliers who kept orders waiting | 123.000ms | 208.664ms | 184.320ms | 10.8% | 181.166ms | 201.036ms | 167.142ms | 203.283ms | 450.000ms | 486.86 MiB | none | 4.70M/s |
| q22 | global sales opportunity | 10.000ms | 80.467ms | 80.700ms | 0.4% | 80.470ms | 80.760ms | 60.748ms | 80.792ms | 90.000ms | 251.66 MiB | none | 10.73M/s |

clickhouse-local 26.9.1.1562 over 22 of 22 queries. Total 1.252s by its own clock and 2.750s by ours, 2.701s cold, 3.670s of CPU, peak 486.86 MiB, 15.22M/s and 511.91 MiB/s.

Running it cost 120% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.28x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 14.000ms | 40.380ms | 40.347ms | 0.0% | 40.340ms | 40.348ms | 40.331ms | 40.387ms | 70.000ms | 204.74 MiB | none | 21.47M/s |
| q02 | minimum cost supplier | 18.000ms | 40.345ms | 40.338ms | 0.0% | 40.336ms | 40.339ms | 40.330ms | 40.359ms | 60.000ms | 227.76 MiB | none | 21.47M/s |
| q03 | shipping priority | 15.000ms | 40.378ms | 40.342ms | 0.1% | 40.336ms | 40.359ms | 40.335ms | 40.388ms | 60.000ms | 217.80 MiB | none | 21.47M/s |
| q04 | order priority checking | 11.000ms | 40.590ms | 40.380ms | 0.1% | 40.366ms | 40.394ms | 40.353ms | 40.766ms | 40.000ms | 211.50 MiB | none | 21.45M/s |
| q05 | local supplier volume | 17.000ms | 40.347ms | 40.445ms | 0.1% | 40.414ms | 40.447ms | 40.396ms | 40.962ms | 130.000ms | 246.35 MiB | none | 21.42M/s |
| q06 | forecasting revenue change | 8.000ms | 40.345ms | 40.344ms | 0.0% | 40.344ms | 40.347ms | 40.329ms | 40.393ms | 40.000ms | 179.25 MiB | none | 21.47M/s |
| q07 | volume shipping | 22.000ms | 40.340ms | 40.354ms | 0.0% | 40.347ms | 40.356ms | 40.346ms | 40.424ms | 90.000ms | 297.71 MiB | none | 21.46M/s |
| q08 | national market share | 19.000ms | 40.431ms | 40.380ms | 0.1% | 40.366ms | 40.399ms | 40.346ms | 40.620ms | 90.000ms | 260.41 MiB | none | 21.45M/s |
| q09 | product type profit measure | 28.000ms | 60.536ms | 60.728ms | 0.5% | 60.660ms | 60.977ms | 60.633ms | 62.224ms | 100.000ms | 363.75 MiB | none | 14.26M/s |
| q10 | returned item reporting | 20.000ms | 40.380ms | 40.358ms | 0.1% | 40.341ms | 40.368ms | 40.339ms | 40.676ms | 100.000ms | 284.15 MiB | none | 21.46M/s |
| q11 | important stock identification | 15.000ms | 40.424ms | 40.406ms | 0.0% | 40.402ms | 40.418ms | 40.393ms | 40.511ms | 40.000ms | 179.45 MiB | none | 21.44M/s |
| q12 | shipping modes and order priority | 14.000ms | 40.465ms | 40.343ms | 0.7% | 40.342ms | 40.621ms | 40.334ms | 41.339ms | 70.000ms | 224.65 MiB | none | 21.47M/s |
| q13 | customer distribution | 18.000ms | 40.354ms | 40.338ms | 0.0% | 40.335ms | 40.343ms | 40.333ms | 40.370ms | 40.000ms | 182.11 MiB | none | 21.47M/s |
| q14 | promotion effect | 10.000ms | 40.341ms | 40.374ms | 0.2% | 40.347ms | 40.425ms | 40.319ms | 40.430ms | 60.000ms | 215.90 MiB | none | 21.45M/s |
| q15 | top supplier | 14.000ms | 40.355ms | 40.357ms | 0.1% | 40.344ms | 40.392ms | 40.340ms | 40.481ms | 70.000ms | 214.74 MiB | none | 21.46M/s |
| q16 | parts supplier relationship | 16.000ms | 40.628ms | 40.541ms | 0.2% | 40.486ms | 40.554ms | 40.481ms | 40.638ms | 30.000ms | 217.38 MiB | none | 21.36M/s |
| q17 | small quantity order revenue | 16.000ms | 40.587ms | 40.359ms | 0.3% | 40.357ms | 40.482ms | 40.353ms | 40.500ms | 110.000ms | 290.63 MiB | none | 21.46M/s |
| q18 | large volume customer | 24.000ms | 40.346ms | 40.485ms | 0.3% | 40.437ms | 40.551ms | 40.337ms | 41.215ms | 120.000ms | 328.71 MiB | none | 21.39M/s |
| q19 | discounted revenue | 13.000ms | 40.367ms | 40.387ms | 0.2% | 40.372ms | 40.469ms | 40.361ms | 40.772ms | 50.000ms | 214.74 MiB | none | 21.45M/s |
| q20 | potential part promotion | 18.000ms | 40.376ms | 40.354ms | 0.0% | 40.353ms | 40.362ms | 40.336ms | 40.391ms | 90.000ms | 312.35 MiB | none | 21.46M/s |
| q21 | suppliers who kept orders waiting | 23.000ms | 40.391ms | 40.418ms | 0.6% | 40.355ms | 40.580ms | 40.355ms | 60.463ms | 160.000ms | 439.23 MiB | none | 21.43M/s |
| q22 | global sales opportunity | 12.000ms | 40.373ms | 40.331ms | 0.0% | 40.330ms | 40.332ms | 40.329ms | 40.412ms | 30.000ms | 164.91 MiB | none | 21.48M/s |

datafusion datafusion-cli 55.1.0 over 22 of 22 queries. Total 365.000ms by its own clock and 908.707ms by ours, 909.081ms cold, 1.650s of CPU, peak 439.23 MiB, 52.21M/s and 1.71 GiB/s.

Running it cost 149% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.51x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 41.727ms | 141.296ms | 141.177ms | 0.7% | 140.929ms | 141.896ms | 140.855ms | 143.509ms | 240.000ms | 180.21 MiB | none | 6.14M/s |
| q03 | shipping priority | 32.344ms | 120.727ms | 120.840ms | 0.4% | 120.678ms | 121.176ms | 120.649ms | 141.802ms | 200.000ms | 115.04 MiB | none | 7.17M/s |
| q04 | order priority checking | 21.011ms | 120.845ms | 121.268ms | 0.8% | 120.867ms | 121.822ms | 120.724ms | 122.305ms | 150.000ms | 102.07 MiB | none | 7.14M/s |
| q05 | local supplier volume | 46.970ms | 141.306ms | 140.834ms | 0.9% | 140.764ms | 142.038ms | 140.738ms | 142.609ms | 200.000ms | 137.79 MiB | none | 6.15M/s |
| q06 | forecasting revenue change | 13.497ms | 100.701ms | 100.728ms | 0.1% | 100.689ms | 100.838ms | 100.614ms | 120.651ms | 130.000ms | 96.86 MiB | none | 8.60M/s |
| q07 | volume shipping | 54.523ms | 141.027ms | 161.006ms | 12.7% | 140.825ms | 161.295ms | 140.770ms | 161.667ms | 220.000ms | 160.57 MiB | none | 5.38M/s |
| q08 | national market share | 54.983ms | 161.846ms | 160.846ms | 12.1% | 141.601ms | 161.016ms | 140.722ms | 161.587ms | 230.000ms | 170.98 MiB | none | 5.38M/s |
| q09 | product type profit measure | 73.061ms | 181.483ms | 161.645ms | 12.2% | 161.350ms | 181.065ms | 160.874ms | 181.226ms | 340.000ms | 255.64 MiB | none | 5.36M/s |
| q10 | returned item reporting | 44.245ms | 141.369ms | 140.863ms | 0.3% | 140.764ms | 141.251ms | 140.711ms | 143.072ms | 220.000ms | 112.21 MiB | none | 6.15M/s |
| q11 | important stock identification | 25.242ms | 120.774ms | 121.140ms | 0.3% | 120.877ms | 121.218ms | 120.761ms | 143.447ms | 160.000ms | 92.77 MiB | none | 7.15M/s |
| q12 | shipping modes and order priority | 24.183ms | 142.070ms | 120.754ms | 0.2% | 120.681ms | 120.928ms | 120.615ms | 121.407ms | 200.000ms | 105.28 MiB | none | 7.17M/s |
| q14 | promotion effect | 15.278ms | 100.628ms | 100.708ms | 0.2% | 100.639ms | 100.879ms | 100.588ms | 120.695ms | 150.000ms | 86.31 MiB | none | 8.60M/s |
| q15 | top supplier | 20.127ms | 120.588ms | 121.064ms | 1.3% | 120.786ms | 122.337ms | 120.613ms | 125.515ms | 170.000ms | 90.38 MiB | none | 7.15M/s |
| q16 | parts supplier relationship | 28.919ms | 121.241ms | 120.702ms | 0.0% | 120.682ms | 120.704ms | 120.681ms | 121.016ms | 180.000ms | 100.07 MiB | none | 7.18M/s |
| q18 | large volume customer | 56.904ms | 160.877ms | 162.119ms | 1.3% | 161.068ms | 163.216ms | 160.865ms | 163.249ms | 290.000ms | 223.75 MiB | none | 5.34M/s |
| q19 | discounted revenue | 18.706ms | 120.882ms | 121.132ms | 0.2% | 120.952ms | 121.190ms | 120.821ms | 125.866ms | 220.000ms | 89.34 MiB | none | 7.15M/s |
| q20 | potential part promotion | 30.549ms | 120.873ms | 122.011ms | 16.9% | 120.701ms | 141.360ms | 120.689ms | 141.362ms | 200.000ms | 116.61 MiB | none | 7.10M/s |
| q21 | suppliers who kept orders waiting | 124.899ms | 221.580ms | 222.193ms | 0.9% | 221.590ms | 223.556ms | 221.199ms | 241.162ms | 680.000ms | 419.36 MiB | none | 3.90M/s |
| q22 | global sales opportunity | 21.369ms | 120.722ms | 121.761ms | 0.5% | 121.141ms | 121.777ms | 121.086ms | 122.836ms | 180.000ms | 87.24 MiB | none | 7.11M/s |

polars 1.44.2 in sink mode over 19 of 22 queries. Total 748.537ms by its own clock and 2.583s by ours, 2.601s cold, 4.360s of CPU, peak 419.36 MiB, 21.99M/s and 739.46 MiB/s.

Running it cost 245% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.21x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 9.330ms | 40.424ms | 20.458ms | 0.7% | 20.321ms | 20.465ms | 20.299ms | 20.502ms | 40.000ms | 45.14 MiB | none | 42.34M/s |
| q02 | minimum cost supplier | 7.014ms | 20.445ms | 20.428ms | 0.1% | 20.425ms | 20.446ms | 20.411ms | 20.537ms | 0.000us | 17.92 MiB | none | 42.40M/s |
| q03 | shipping priority | 10.799ms | 20.323ms | 20.400ms | 0.9% | 20.360ms | 20.537ms | 20.341ms | 20.540ms | 20.000ms | 47.45 MiB | none | 42.46M/s |
| q04 | order priority checking | 7.741ms | 20.319ms | 20.327ms | 0.2% | 20.312ms | 20.361ms | 20.309ms | 20.445ms | 10.000ms | 35.52 MiB | none | 42.61M/s |
| q05 | local supplier volume | 13.412ms | 20.358ms | 20.445ms | 0.3% | 20.392ms | 20.452ms | 20.388ms | 20.471ms | 30.000ms | 51.45 MiB | none | 42.37M/s |
| q06 | forecasting revenue change | 3.547ms | 20.341ms | 20.295ms | 0.1% | 20.281ms | 20.310ms | 20.271ms | 20.420ms | 0.000us | 34.85 MiB | none | 42.68M/s |
| q07 | volume shipping | 10.938ms | 20.517ms | 20.410ms | 0.5% | 20.402ms | 20.495ms | 20.398ms | 20.537ms | 20.000ms | 49.96 MiB | none | 42.44M/s |
| q08 | national market share | 9.496ms | 20.417ms | 20.473ms | 0.6% | 20.438ms | 20.559ms | 20.422ms | 20.564ms | 20.000ms | 47.01 MiB | none | 42.31M/s |
| q09 | product type profit measure | 25.476ms | 40.481ms | 40.476ms | 0.5% | 40.474ms | 40.674ms | 40.471ms | 40.712ms | 50.000ms | 68.20 MiB | none | 21.40M/s |
| q10 | returned item reporting | 16.141ms | 42.991ms | 20.421ms | 98.2% | 20.382ms | 40.431ms | 20.361ms | 40.537ms | 30.000ms | 55.99 MiB | none | 42.41M/s |
| q11 | important stock identification | 7.545ms | 20.512ms | 20.505ms | 0.1% | 20.489ms | 20.514ms | 20.465ms | 20.561ms | 0.000us | 18.52 MiB | none | 42.24M/s |
| q12 | shipping modes and order priority | 13.122ms | 20.299ms | 20.312ms | 0.0% | 20.310ms | 20.313ms | 20.302ms | 20.317ms | 20.000ms | 42.03 MiB | none | 42.64M/s |
| q13 | customer distribution | 36.609ms | 40.387ms | 40.406ms | 0.0% | 40.390ms | 40.407ms | 40.385ms | 40.646ms | 40.000ms | 35.17 MiB | none | 21.44M/s |
| q14 | promotion effect | 7.444ms | 20.303ms | 20.293ms | 0.1% | 20.290ms | 20.305ms | 20.281ms | 20.435ms | 10.000ms | 35.50 MiB | none | 42.68M/s |
| q15 | top supplier | 9.066ms | 20.339ms | 20.360ms | 0.0% | 20.356ms | 20.362ms | 20.336ms | 20.501ms | 20.000ms | 43.98 MiB | none | 42.54M/s |
| q16 | parts supplier relationship | 7.106ms | 21.042ms | 20.522ms | 0.7% | 20.467ms | 20.601ms | 20.458ms | 20.823ms | 0.000us | 16.23 MiB | none | 42.21M/s |
| q17 | small quantity order revenue | 6.084ms | 20.702ms | 20.364ms | 0.6% | 20.360ms | 20.487ms | 20.359ms | 20.487ms | 10.000ms | 37.48 MiB | none | 42.53M/s |
| q18 | large volume customer | 22.546ms | 40.474ms | 40.458ms | 0.0% | 40.456ms | 40.459ms | 40.443ms | 40.594ms | 50.000ms | 57.00 MiB | none | 21.41M/s |
| q19 | discounted revenue | 7.030ms | 20.433ms | 20.313ms | 0.2% | 20.311ms | 20.347ms | 20.311ms | 20.461ms | 20.000ms | 42.27 MiB | none | 42.64M/s |
| q20 | potential part promotion | 14.267ms | 20.384ms | 20.405ms | 0.3% | 20.400ms | 20.460ms | 20.394ms | 20.548ms | 30.000ms | 49.74 MiB | none | 42.45M/s |
| q21 | suppliers who kept orders waiting | 16.799ms | 40.568ms | 40.486ms | 0.1% | 40.486ms | 40.544ms | 20.405ms | 40.659ms | 50.000ms | 55.98 MiB | none | 21.39M/s |
| q22 | global sales opportunity | 9.353ms | 20.484ms | 20.314ms | 0.3% | 20.313ms | 20.373ms | 20.306ms | 20.455ms | 0.000us | 16.74 MiB | none | 42.64M/s |

rudb rudb 0.3.67 over 22 of 22 queries. Total 270.865ms by its own clock and 528.869ms by ours, 572.544ms cold, 470.000ms of CPU, peak 68.20 MiB, 70.35M/s and 2.31 GiB/s.

Running it cost 95% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.00x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | 386.589us | 17.993ms | 44.013ms | 44.032ms | 0.0% | 44.013ms | 130.474us | 0.000us | 6.64 KiB | 6 of 6 |
| q02 | 638.235us | 6.301ms | 6.252ms | 6.297ms | 0.7% | 6.252ms | 198.074us | 0.000us | 525.90 KiB | 47 of 47 |
| q03 | 369.577us | 14.756ms | 28.288ms | 28.309ms | 0.1% | 28.288ms | 118.191us | 21.573ms | 2.49 MiB | 16 of 16 |
| q04 | 334.135us | 5.990ms | 16.109ms | 16.128ms | 0.1% | 16.109ms | 119.108us | 0.000us | 1.87 MiB | 11 of 11 |
| q05 | 540.640us | 14.949ms | 33.015ms | 33.045ms | 0.1% | 33.015ms | 168.696us | 0.000us | 3.92 MiB | 27 of 27 |
| q06 | 270.229us | 3.247ms | 14.108ms | 14.121ms | 0.1% | 14.108ms | 102.580us | 0.000us | 896 B | 5 of 5 |
| q07 | 608.660us | 9.983ms | 28.987ms | 29.015ms | 0.1% | 28.987ms | 180.484us | 804.296us | 2.23 MiB | 30 of 30 |
| q08 | 669.011us | 8.328ms | 26.048ms | 26.088ms | 0.2% | 26.048ms | 195.530us | 3.717ms | 1.34 MiB | 37 of 37 |
| q09 | 556.472us | 27.508ms | 51.946ms | 51.984ms | 0.1% | 51.946ms | 182.003us | 7.834ms | 21.96 MiB | 27 of 27 |
| q10 | 502.989us | 23.817ms | 45.484ms | 45.519ms | 0.1% | 45.484ms | 164.306us | 24.316ms | 7.64 MiB | 19 of 19 |
| q11 | 556.828us | 6.370ms | 6.339ms | 6.370ms | 0.5% | 6.339ms | 212.380us | 0.000us | 1.53 MiB | 30 of 30 |
| q12 | 347.048us | 12.778ms | 27.780ms | 27.800ms | 0.1% | 27.780ms | 98.153us | 0.000us | 4.87 MiB | 10 of 10 |
| q13 | 311.312us | 35.602ms | 44.285ms | 44.297ms | 0.0% | 44.285ms | 131.177us | 0.000us | 3.77 MiB | 12 of 12 |
| q14 | 328.105us | 7.864ms | 12.363ms | 12.381ms | 0.1% | 12.363ms | 115.373us | 0.000us | 6.57 MiB | 9 of 9 |
| q15 | 542.915us | 12.261ms | 27.678ms | 27.706ms | 0.1% | 27.678ms | 205.185us | 2.089ms | 1.28 MiB | 21 of 21 |
| q16 | 395.116us | 5.627ms | 5.604ms | 5.627ms | 0.4% | 5.604ms | 134.674us | 0.000us | 2.82 MiB | 18 of 18 |
| q17 | 407.069us | 11.057ms | 21.963ms | 21.988ms | 0.1% | 21.963ms | 169.403us | 7.843ms | 321.06 KiB | 21 of 21 |
| q18 | 521.159us | 21.958ms | 54.181ms | 54.217ms | 0.1% | 54.181ms | 186.836us | 5.596ms | 11.30 MiB | 21 of 21 |
| q19 | 603.655us | 6.265ms | 21.338ms | 21.361ms | 0.1% | 21.338ms | 182.030us | 0.000us | 165.89 KiB | 11 of 11 |
| q20 | 528.486us | 13.675ms | 29.481ms | 29.520ms | 0.1% | 29.481ms | 169.023us | 0.000us | 8.51 MiB | 30 of 30 |
| q21 | 675.110us | 16.091ms | 49.886ms | 49.930ms | 0.1% | 49.886ms | 200.456us | 0.000us | 9.86 MiB | 29 of 29 |
| q22 | 493.849us | 8.855ms | 9.441ms | 9.457ms | 0.2% | 9.441ms | 156.957us | 0.000us | 2.28 MiB | 19 of 19 |

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
| Aggregate | 0.000us | nothing to share | 29 | 1600989 | 233362 | 0.0ns | 0.0ns | 29 of 29 |
| FileScan | 0.000us | nothing to share | 89 | 0 | 6626516 | handed none | 0.0ns | 89 of 89 |
| Filter | 0.000us | nothing to share | 54 | 5556507 | 1126483 | 0.0ns | 0.0ns | 54 of 54 |
| Gather | 0.000us | nothing to share | 67 | 345233 | 0 | 0.0ns | handed on none | 67 of 67 |
| Join | 0.000us | nothing to share | 3 | 8831 | 8831 | 0.0ns | 0.0ns | 3 of 3 |
| Mark | 0.000us | nothing to share | 5 | 188770 | 23830 | 0.0ns | 0.0ns | 5 of 5 |
| Pad | 0.000us | nothing to share | 1 | 148467 | 153467 | 0.0ns | 0.0ns | 1 of 1 |
| Probe | 0.000us | nothing to share | 58 | 872795 | 559740 | 0.0ns | 0.0ns | 58 of 58 |
| Project | 0.000us | nothing to share | 132 | 2469742 | 2469742 | 0.0ns | 0.0ns | 132 of 132 |
| Sort | 0.000us | nothing to share | 13 | 5554 | 5554 | 0.0ns | 0.0ns | 13 of 13 |
| TopN | 0.000us | nothing to share | 5 | 5079 | 126 | 0.0ns | 0.0ns | 5 of 5 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is Aggregate, and the queries where it cost the most are q01 at 0.000us, q02 at 0.000us, q03 at 0.000us.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven

These swung wider than reporting rule two allows:

- duckdb-pinned swung by 33.7% of its median on q07, and rule two wants under 10%
- clickhouse-local swung by 19.0% of its median on q07, and rule two wants under 10%
- polars swung by 16.9% of its median on q20, and rule two wants under 10%
- rudb swung by 98.2% of its median on q10, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

These ran every query at about the same speed:

- duckdb-pinned ran every query within 1.50x of every other one, and this suite spreads over 6x on an engine it is measuring
- datafusion ran every query within 1.51x of every other one, and this suite spreads over 6x on an engine it is measuring
- rudb ran every query within 2.00x of every other one, and this suite spreads over 6x on an engine it is measuring

A column that flat is not a column about the queries. Read those numbers as an upper bound on the engine and not as a measurement of it.

polars did not run q02, because the SQLContext does not resolve the correlated p_partkey in the min supplycost subquery.

polars did not run q13, because the SQLContext rejects a NOT LIKE inside a join constraint.

polars did not run q17, because the SQLContext does not resolve the correlated p_partkey in the average quantity subquery.

So the polars column is 19 of 22 queries and its ratio is over the shared ones.

Answers differ, so this is not a comparison: q03: polars does not agree with duckdb: 30 numbers against 30

Answers differ, so this is not a comparison: q05: polars does not agree with duckdb: 5 numbers against 5

Answers differ, so this is not a comparison: q06: polars does not agree with duckdb: 1 numbers against 1

Answers differ, so this is not a comparison: q07: polars does not agree with duckdb: 8 numbers against 8

Answers differ, so this is not a comparison: q09: polars does not agree with duckdb: 350 numbers against 350

Answers differ, so this is not a comparison: q10: polars does not agree with duckdb: 60 numbers against 60

Answers differ, so this is not a comparison: q14: polars does not agree with duckdb: 1 numbers against 1

Hot here means page cache warm and not buffer pool warm, because every run is a fresh process so that no query's number depends on the one before it.

