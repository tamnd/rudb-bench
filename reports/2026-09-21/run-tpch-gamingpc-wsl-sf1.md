# tpch on gamingpc-wsl

This is one run of the tpch suite on gamingpc-wsl, over 6 engines and 22 queries, with 5 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

## What ran

| what | it was |
| --- | --- |
| suite | tpch |
| queries | 22 |
| tables | 310.61 MiB of Parquet in 8 tables |
| rows | 8661245 in the table every query reads |
| corpus | no manifest beside the data, so this run cannot say where it came from |
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
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 1.083s | 10.000s | 258.01 MiB | its own database file | its own | 7.06 to 10.01 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 994.491ms | 8.990s | 272.76 MiB | its own database file | its own | 10.01 to 10.84 |
| clickhouse-local | 26.9.1.1562 | ran | 2.035s | 5.600s | 458.77 MiB | its own MergeTree parts, as system.parts counts the active ones | its own | 10.84 to 12.67 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 310.61 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 12.67 to 16.04 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 310.61 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 16.04 to 10.84 |
| rudb | rudb 0.3.67 | ran | 0.000us | 0.000us | 310.61 MiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 10.84 to 11.01 |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

These engines started while the machine was already above half of that, which means they were measured against somebody else's work rather than on an idle box: polars. Their numbers are inflated by an amount nothing here can recover, and by a different amount each, depending on how much of the column was CPU bound. One case where that reading is unfair is a sweep that runs engines back to back, where the average has not had a minute to come down from the engine before.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb on 19 shared |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 766.000ms | 1.232s | +61% | 1.294s | 3.470s | 2.82 | 283.89 MiB | none | 248.76M/s | 8.71 GiB/s | 1.00x |
| duckdb-pinned | 860.000ms | 1.566s | +82% | 1.629s | 3.540s | 2.26 | 259.80 MiB | none | 221.57M/s | 7.76 GiB/s | 1.13x |
| clickhouse-local | 3.079s | 4.957s | +61% | 5.024s | 20.100s | 4.06 | 839.93 MiB | none | 61.89M/s | 2.17 GiB/s | 3.90x |
| datafusion | 1.144s | 1.758s | +54% | 1.763s | 17.820s | 10.14 | 1.37 GiB | none | 166.56M/s | 5.83 GiB/s | 1.43x |
| polars | 2.657s | 4.745s | +79% | 4.720s | 42.810s | 9.02 | 2.51 GiB | none | 61.94M/s | 2.17 GiB/s | 3.84x |
| rudb | 1.092s | 1.549s | +42% | 1.567s | 8.200s | 5.29 | 421.99 MiB | none | 174.56M/s | 6.11 GiB/s | 1.40x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 22 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 41.000ms | 58.000ms | 146.000ms | 53.000ms | 105.246ms | 34.365ms |
| q02 | minimum cost supplier | 12.000ms | 13.000ms | 106.000ms | 25.000ms | no dialect | 17.892ms |
| q03 | shipping priority | 34.000ms | 47.000ms | 146.000ms | 45.000ms | 53.746ms | 37.155ms |
| q04 | order priority checking | 21.000ms | 23.000ms | 133.000ms | 29.000ms | 42.103ms | 23.887ms |
| q05 | local supplier volume | 39.000ms | 35.000ms | 172.000ms | 64.000ms | 76.179ms | 50.428ms |
| q06 | forecasting revenue change | 17.000ms | 16.000ms | 113.000ms | 26.000ms | 35.598ms | 11.938ms |
| q07 | volume shipping | 47.000ms | 62.000ms | 157.000ms | 77.000ms | 97.796ms | 38.681ms |
| q08 | national market share | 39.000ms | 53.000ms | 172.000ms | 52.000ms | 204.583ms | 43.010ms |
| q09 | product type profit measure | 82.000ms | 68.000ms | 166.000ms | 68.000ms | 676.182ms | 127.684ms |
| q10 | returned item reporting | 59.000ms | 66.000ms | 176.000ms | 66.000ms | 89.900ms | 92.236ms |
| q11 | important stock identification | 7.000ms | 12.000ms | 66.000ms | 18.000ms | 37.004ms | 17.452ms |
| q12 | shipping modes and order priority | 21.000ms | 32.000ms | 150.000ms | 44.000ms | 38.438ms | 36.239ms |
| q13 | customer distribution | 32.000ms | 39.000ms | 126.000ms | 35.000ms | no dialect | 72.896ms |
| q14 | promotion effect | 28.000ms | 30.000ms | 122.000ms | 30.000ms | 38.975ms | 36.971ms |
| q15 | top supplier | 27.000ms | 44.000ms | 163.000ms | 40.000ms | 36.939ms | 37.664ms |
| q16 | parts supplier relationship | 23.000ms | 28.000ms | 93.000ms | 49.000ms | 47.117ms | 35.112ms |
| q17 | small quantity order revenue | 31.000ms | 30.000ms | 154.000ms | 94.000ms | no dialect | 30.316ms |
| q18 | large volume customer | 33.000ms | 33.000ms | 142.000ms | 105.000ms | 350.652ms | 83.647ms |
| q19 | discounted revenue | 60.000ms | 62.000ms | 139.000ms | 41.000ms | 44.215ms | 31.841ms |
| q20 | potential part promotion | 37.000ms | 48.000ms | 133.000ms | 56.000ms | 110.367ms | 82.544ms |
| q21 | suppliers who kept orders waiting | 56.000ms | 36.000ms | 235.000ms | 104.000ms | 533.967ms | 77.112ms |
| q22 | global sales opportunity | 20.000ms | 25.000ms | 69.000ms | 23.000ms | 37.780ms | 72.515ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 41.000ms | 60.387ms | 60.470ms | 0.2% | 60.455ms | 60.595ms | 60.437ms | 60.947ms | 170.000ms | 108.84 MiB | none | 143.23M/s |
| q02 | minimum cost supplier | 12.000ms | 40.481ms | 40.457ms | 0.3% | 40.348ms | 40.461ms | 40.318ms | 40.474ms | 40.000ms | 64.15 MiB | none | 214.08M/s |
| q03 | shipping priority | 34.000ms | 60.527ms | 60.415ms | 33.3% | 60.410ms | 80.546ms | 60.403ms | 80.730ms | 140.000ms | 134.64 MiB | none | 143.36M/s |
| q04 | order priority checking | 21.000ms | 40.614ms | 40.477ms | 0.4% | 40.342ms | 40.506ms | 40.340ms | 41.009ms | 110.000ms | 108.57 MiB | none | 213.98M/s |
| q05 | local supplier volume | 39.000ms | 60.557ms | 60.430ms | 0.1% | 60.425ms | 60.456ms | 60.399ms | 80.615ms | 150.000ms | 137.64 MiB | none | 143.33M/s |
| q06 | forecasting revenue change | 17.000ms | 40.547ms | 40.362ms | 0.1% | 40.337ms | 40.383ms | 40.320ms | 61.775ms | 70.000ms | 98.75 MiB | none | 214.59M/s |
| q07 | volume shipping | 47.000ms | 60.816ms | 60.511ms | 0.2% | 60.402ms | 60.541ms | 60.401ms | 60.571ms | 160.000ms | 151.82 MiB | none | 143.14M/s |
| q08 | national market share | 39.000ms | 60.783ms | 60.566ms | 1.7% | 60.538ms | 61.580ms | 60.432ms | 80.652ms | 130.000ms | 138.70 MiB | none | 143.01M/s |
| q09 | product type profit measure | 82.000ms | 120.847ms | 100.976ms | 0.4% | 100.888ms | 101.287ms | 100.579ms | 102.253ms | 350.000ms | 283.89 MiB | none | 85.78M/s |
| q10 | returned item reporting | 59.000ms | 80.553ms | 80.646ms | 0.2% | 80.500ms | 80.660ms | 80.497ms | 101.447ms | 260.000ms | 211.77 MiB | none | 107.40M/s |
| q11 | important stock identification | 7.000ms | 20.320ms | 20.283ms | 0.1% | 20.274ms | 20.284ms | 20.273ms | 20.331ms | 30.000ms | 59.44 MiB | none | 427.01M/s |
| q12 | shipping modes and order priority | 21.000ms | 40.485ms | 40.470ms | 0.8% | 40.345ms | 40.675ms | 40.345ms | 41.063ms | 110.000ms | 114.50 MiB | none | 214.02M/s |
| q13 | customer distribution | 32.000ms | 60.523ms | 60.474ms | 0.3% | 60.414ms | 60.574ms | 60.374ms | 60.665ms | 260.000ms | 147.38 MiB | none | 143.22M/s |
| q14 | promotion effect | 28.000ms | 60.601ms | 40.486ms | 49.2% | 40.482ms | 60.421ms | 40.331ms | 60.713ms | 100.000ms | 130.32 MiB | none | 213.93M/s |
| q15 | top supplier | 27.000ms | 60.531ms | 40.322ms | 0.1% | 40.322ms | 40.342ms | 40.322ms | 40.487ms | 110.000ms | 130.88 MiB | none | 214.80M/s |
| q16 | parts supplier relationship | 23.000ms | 41.242ms | 40.950ms | 0.7% | 40.917ms | 41.211ms | 40.913ms | 41.414ms | 80.000ms | 108.95 MiB | none | 211.51M/s |
| q17 | small quantity order revenue | 31.000ms | 40.347ms | 60.418ms | 33.6% | 40.355ms | 60.657ms | 40.321ms | 60.791ms | 120.000ms | 119.88 MiB | none | 143.36M/s |
| q18 | large volume customer | 33.000ms | 61.038ms | 60.821ms | 0.5% | 60.533ms | 60.837ms | 60.420ms | 61.737ms | 370.000ms | 232.20 MiB | none | 142.40M/s |
| q19 | discounted revenue | 60.000ms | 101.166ms | 80.656ms | 0.5% | 80.523ms | 80.933ms | 80.512ms | 81.817ms | 210.000ms | 156.68 MiB | none | 107.39M/s |
| q20 | potential part promotion | 37.000ms | 60.431ms | 60.614ms | 0.2% | 60.566ms | 60.672ms | 60.423ms | 81.249ms | 130.000ms | 137.07 MiB | none | 142.89M/s |
| q21 | suppliers who kept orders waiting | 56.000ms | 80.563ms | 80.509ms | 0.0% | 80.507ms | 80.519ms | 80.481ms | 80.620ms | 320.000ms | 179.39 MiB | none | 107.58M/s |
| q22 | global sales opportunity | 20.000ms | 40.325ms | 40.491ms | 0.4% | 40.338ms | 40.497ms | 40.317ms | 40.547ms | 50.000ms | 55.69 MiB | none | 213.91M/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 22 of 22 queries. Total 766.000ms by its own clock and 1.232s by ours, 1.294s cold, 3.470s of CPU, peak 283.89 MiB, 248.76M/s and 8.71 GiB/s.

Running it cost 61% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 4.98x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 58.000ms | 80.545ms | 92.979ms | 21.6% | 80.501ms | 100.562ms | 80.470ms | 103.764ms | 200.000ms | 120.52 MiB | none | 93.15M/s |
| q02 | minimum cost supplier | 13.000ms | 40.344ms | 40.381ms | 0.2% | 40.322ms | 40.390ms | 40.318ms | 40.468ms | 40.000ms | 67.23 MiB | none | 214.49M/s |
| q03 | shipping priority | 47.000ms | 80.484ms | 80.671ms | 0.3% | 80.535ms | 80.781ms | 60.397ms | 80.858ms | 150.000ms | 140.66 MiB | none | 107.37M/s |
| q04 | order priority checking | 23.000ms | 60.747ms | 60.563ms | 0.0% | 60.558ms | 60.584ms | 60.435ms | 64.741ms | 100.000ms | 104.96 MiB | none | 143.01M/s |
| q05 | local supplier volume | 35.000ms | 60.403ms | 60.548ms | 32.9% | 60.545ms | 80.475ms | 60.415ms | 80.483ms | 150.000ms | 151.95 MiB | none | 143.05M/s |
| q06 | forecasting revenue change | 16.000ms | 40.321ms | 40.913ms | 1.4% | 40.577ms | 41.167ms | 40.438ms | 60.622ms | 80.000ms | 108.79 MiB | none | 211.70M/s |
| q07 | volume shipping | 62.000ms | 100.677ms | 100.566ms | 0.1% | 100.551ms | 100.610ms | 100.542ms | 100.680ms | 200.000ms | 159.64 MiB | none | 86.13M/s |
| q08 | national market share | 53.000ms | 80.661ms | 80.478ms | 0.2% | 80.463ms | 80.626ms | 80.460ms | 100.944ms | 170.000ms | 147.96 MiB | none | 107.62M/s |
| q09 | product type profit measure | 68.000ms | 121.647ms | 100.610ms | 0.0% | 100.605ms | 100.650ms | 100.591ms | 100.792ms | 360.000ms | 234.70 MiB | none | 86.09M/s |
| q10 | returned item reporting | 66.000ms | 125.261ms | 100.665ms | 0.2% | 100.569ms | 100.721ms | 100.567ms | 120.767ms | 240.000ms | 219.55 MiB | none | 86.04M/s |
| q11 | important stock identification | 12.000ms | 40.390ms | 40.361ms | 0.0% | 40.355ms | 40.367ms | 40.347ms | 47.186ms | 50.000ms | 64.03 MiB | none | 214.60M/s |
| q12 | shipping modes and order priority | 32.000ms | 80.498ms | 60.517ms | 6.0% | 60.431ms | 64.053ms | 60.421ms | 74.502ms | 140.000ms | 126.42 MiB | none | 143.12M/s |
| q13 | customer distribution | 39.000ms | 80.551ms | 80.865ms | 26.0% | 60.603ms | 81.664ms | 60.558ms | 82.153ms | 250.000ms | 178.00 MiB | none | 107.11M/s |
| q14 | promotion effect | 30.000ms | 60.551ms | 60.741ms | 0.9% | 60.581ms | 61.142ms | 60.472ms | 80.579ms | 100.000ms | 136.23 MiB | none | 142.59M/s |
| q15 | top supplier | 44.000ms | 83.735ms | 80.501ms | 0.2% | 80.479ms | 80.631ms | 80.469ms | 80.790ms | 150.000ms | 140.48 MiB | none | 107.59M/s |
| q16 | parts supplier relationship | 28.000ms | 67.088ms | 61.065ms | 0.3% | 61.035ms | 61.208ms | 61.001ms | 63.039ms | 110.000ms | 111.15 MiB | none | 141.84M/s |
| q17 | small quantity order revenue | 30.000ms | 62.376ms | 60.558ms | 33.3% | 60.434ms | 80.575ms | 60.400ms | 81.125ms | 130.000ms | 127.77 MiB | none | 143.02M/s |
| q18 | large volume customer | 33.000ms | 60.478ms | 60.672ms | 0.7% | 60.619ms | 61.016ms | 60.441ms | 61.699ms | 320.000ms | 259.80 MiB | none | 142.75M/s |
| q19 | discounted revenue | 62.000ms | 80.547ms | 100.657ms | 20.3% | 80.605ms | 101.082ms | 80.515ms | 110.449ms | 180.000ms | 148.22 MiB | none | 86.05M/s |
| q20 | potential part promotion | 48.000ms | 80.489ms | 80.731ms | 0.1% | 80.665ms | 80.735ms | 80.516ms | 85.158ms | 150.000ms | 142.68 MiB | none | 107.28M/s |
| q21 | suppliers who kept orders waiting | 36.000ms | 80.659ms | 60.435ms | 0.1% | 60.410ms | 60.442ms | 60.407ms | 80.469ms | 200.000ms | 141.91 MiB | none | 143.32M/s |
| q22 | global sales opportunity | 25.000ms | 60.382ms | 60.565ms | 0.3% | 60.430ms | 60.621ms | 49.171ms | 60.701ms | 70.000ms | 77.57 MiB | none | 143.01M/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 22 of 22 queries. Total 860.000ms by its own clock and 1.566s by ours, 1.629s cold, 3.540s of CPU, peak 259.80 MiB, 221.57M/s and 7.76 GiB/s.

Running it cost 82% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.49x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 146.000ms | 221.205ms | 261.376ms | 5.4% | 247.365ms | 261.518ms | 201.245ms | 264.134ms | 1.390s | 540.83 MiB | none | 33.14M/s |
| q02 | minimum cost supplier | 106.000ms | 187.767ms | 207.273ms | 9.5% | 201.474ms | 221.193ms | 201.189ms | 221.936ms | 530.000ms | 491.12 MiB | none | 41.79M/s |
| q03 | shipping priority | 146.000ms | 202.941ms | 222.556ms | 8.9% | 222.342ms | 242.156ms | 221.088ms | 242.855ms | 1.020s | 523.74 MiB | none | 38.92M/s |
| q04 | order priority checking | 133.000ms | 261.653ms | 221.144ms | 9.2% | 201.253ms | 221.632ms | 200.902ms | 264.688ms | 660.000ms | 510.71 MiB | none | 39.17M/s |
| q05 | local supplier volume | 172.000ms | 241.852ms | 252.645ms | 7.7% | 242.424ms | 261.754ms | 225.808ms | 301.547ms | 1.200s | 542.09 MiB | none | 34.28M/s |
| q06 | forecasting revenue change | 113.000ms | 202.906ms | 181.733ms | 1.6% | 181.236ms | 184.092ms | 181.120ms | 201.330ms | 530.000ms | 437.16 MiB | none | 47.66M/s |
| q07 | volume shipping | 157.000ms | 282.676ms | 225.710ms | 15.6% | 221.369ms | 256.492ms | 212.356ms | 258.780ms | 1.310s | 490.76 MiB | none | 38.37M/s |
| q08 | national market share | 172.000ms | 262.446ms | 243.428ms | 9.6% | 222.484ms | 245.954ms | 221.470ms | 263.222ms | 1.070s | 495.23 MiB | none | 35.58M/s |
| q09 | product type profit measure | 166.000ms | 221.401ms | 241.895ms | 0.7% | 241.740ms | 243.539ms | 241.478ms | 261.777ms | 1.430s | 610.54 MiB | none | 35.81M/s |
| q10 | returned item reporting | 176.000ms | 245.333ms | 268.730ms | 5.4% | 267.297ms | 281.806ms | 243.453ms | 302.209ms | 1.020s | 530.58 MiB | none | 32.23M/s |
| q11 | important stock identification | 66.000ms | 181.136ms | 181.262ms | 0.2% | 181.185ms | 181.494ms | 161.183ms | 181.745ms | 250.000ms | 389.65 MiB | none | 47.78M/s |
| q12 | shipping modes and order priority | 150.000ms | 221.334ms | 223.352ms | 13.9% | 222.672ms | 253.704ms | 202.374ms | 262.034ms | 1.030s | 485.33 MiB | none | 38.78M/s |
| q13 | customer distribution | 126.000ms | 221.592ms | 221.361ms | 12.1% | 203.000ms | 229.716ms | 202.653ms | 233.690ms | 1.040s | 754.59 MiB | none | 39.13M/s |
| q14 | promotion effect | 122.000ms | 241.489ms | 201.338ms | 9.2% | 183.338ms | 201.786ms | 180.863ms | 237.902ms | 600.000ms | 453.54 MiB | none | 43.02M/s |
| q15 | top supplier | 163.000ms | 222.740ms | 242.891ms | 8.3% | 241.166ms | 261.212ms | 222.122ms | 269.359ms | 1.100s | 474.68 MiB | none | 35.66M/s |
| q16 | parts supplier relationship | 93.000ms | 182.603ms | 182.180ms | 11.7% | 182.042ms | 203.300ms | 161.964ms | 203.529ms | 280.000ms | 390.36 MiB | none | 47.54M/s |
| q17 | small quantity order revenue | 154.000ms | 231.765ms | 221.291ms | 0.5% | 221.169ms | 222.262ms | 181.078ms | 241.116ms | 850.000ms | 376.11 MiB | none | 39.14M/s |
| q18 | large volume customer | 142.000ms | 264.534ms | 222.008ms | 0.8% | 221.186ms | 222.992ms | 201.735ms | 262.971ms | 1.110s | 629.59 MiB | none | 39.01M/s |
| q19 | discounted revenue | 139.000ms | 221.594ms | 204.446ms | 9.2% | 202.605ms | 221.403ms | 189.461ms | 226.133ms | 910.000ms | 388.83 MiB | none | 42.36M/s |
| q20 | potential part promotion | 133.000ms | 202.029ms | 205.729ms | 3.9% | 201.286ms | 209.341ms | 182.919ms | 241.093ms | 660.000ms | 438.60 MiB | none | 42.10M/s |
| q21 | suppliers who kept orders waiting | 235.000ms | 342.219ms | 343.342ms | 5.8% | 324.680ms | 344.447ms | 302.523ms | 362.354ms | 1.850s | 839.93 MiB | none | 25.23M/s |
| q22 | global sales opportunity | 69.000ms | 161.115ms | 180.884ms | 11.0% | 161.328ms | 181.171ms | 121.473ms | 201.567ms | 260.000ms | 377.98 MiB | none | 47.88M/s |

clickhouse-local 26.9.1.1562 over 22 of 22 queries. Total 3.079s by its own clock and 4.957s by ours, 5.024s cold, 20.100s of CPU, peak 839.93 MiB, 61.89M/s and 2.17 GiB/s.

Running it cost 61% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.90x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 53.000ms | 81.239ms | 80.748ms | 0.2% | 80.657ms | 80.800ms | 63.609ms | 83.120ms | 1.150s | 605.48 MiB | none | 107.26M/s |
| q02 | minimum cost supplier | 25.000ms | 60.640ms | 40.370ms | 0.1% | 40.369ms | 40.405ms | 40.357ms | 40.414ms | 120.000ms | 371.55 MiB | none | 214.55M/s |
| q03 | shipping priority | 45.000ms | 80.585ms | 80.785ms | 0.2% | 80.666ms | 80.843ms | 60.527ms | 81.466ms | 720.000ms | 601.09 MiB | none | 107.21M/s |
| q04 | order priority checking | 29.000ms | 60.583ms | 60.488ms | 0.1% | 60.471ms | 60.528ms | 60.453ms | 60.602ms | 390.000ms | 569.08 MiB | none | 143.19M/s |
| q05 | local supplier volume | 64.000ms | 100.933ms | 100.764ms | 19.1% | 81.619ms | 100.853ms | 80.854ms | 101.937ms | 910.000ms | 719.23 MiB | none | 85.96M/s |
| q06 | forecasting revenue change | 26.000ms | 41.790ms | 60.506ms | 31.4% | 41.600ms | 60.586ms | 40.370ms | 60.697ms | 480.000ms | 504.39 MiB | none | 143.15M/s |
| q07 | volume shipping | 77.000ms | 101.459ms | 101.343ms | 0.5% | 101.081ms | 101.597ms | 100.872ms | 101.658ms | 1.330s | 881.83 MiB | none | 85.46M/s |
| q08 | national market share | 52.000ms | 80.888ms | 80.749ms | 1.0% | 80.653ms | 81.435ms | 80.579ms | 81.559ms | 680.000ms | 656.73 MiB | none | 107.26M/s |
| q09 | product type profit measure | 68.000ms | 100.714ms | 101.000ms | 0.2% | 100.828ms | 101.027ms | 100.792ms | 103.334ms | 940.000ms | 992.42 MiB | none | 85.75M/s |
| q10 | returned item reporting | 66.000ms | 101.059ms | 100.775ms | 0.4% | 100.731ms | 101.128ms | 80.869ms | 102.092ms | 820.000ms | 765.56 MiB | none | 85.95M/s |
| q11 | important stock identification | 18.000ms | 40.384ms | 40.518ms | 0.6% | 40.386ms | 40.619ms | 40.372ms | 41.340ms | 80.000ms | 254.81 MiB | none | 213.76M/s |
| q12 | shipping modes and order priority | 44.000ms | 60.445ms | 60.586ms | 0.2% | 60.532ms | 60.673ms | 60.521ms | 60.694ms | 760.000ms | 600.04 MiB | none | 142.96M/s |
| q13 | customer distribution | 35.000ms | 60.779ms | 60.753ms | 0.3% | 60.613ms | 60.797ms | 60.518ms | 61.455ms | 450.000ms | 533.99 MiB | none | 142.56M/s |
| q14 | promotion effect | 30.000ms | 60.608ms | 60.658ms | 0.1% | 60.615ms | 60.661ms | 60.507ms | 60.720ms | 470.000ms | 450.91 MiB | none | 142.79M/s |
| q15 | top supplier | 40.000ms | 60.524ms | 60.585ms | 0.6% | 60.508ms | 60.886ms | 60.506ms | 82.393ms | 660.000ms | 558.11 MiB | none | 142.96M/s |
| q16 | parts supplier relationship | 49.000ms | 81.955ms | 81.836ms | 0.1% | 81.829ms | 81.937ms | 81.808ms | 81.974ms | 140.000ms | 449.60 MiB | none | 105.84M/s |
| q17 | small quantity order revenue | 94.000ms | 122.824ms | 121.517ms | 0.4% | 121.173ms | 121.601ms | 121.155ms | 121.702ms | 1.970s | 1.04 GiB | none | 71.28M/s |
| q18 | large volume customer | 105.000ms | 121.378ms | 141.174ms | 15.5% | 121.208ms | 143.149ms | 121.117ms | 181.509ms | 2.080s | 1.17 GiB | none | 61.35M/s |
| q19 | discounted revenue | 41.000ms | 82.199ms | 60.512ms | 2.1% | 60.510ms | 61.786ms | 60.502ms | 80.630ms | 670.000ms | 588.86 MiB | none | 143.13M/s |
| q20 | potential part promotion | 56.000ms | 80.824ms | 81.122ms | 1.3% | 80.957ms | 82.027ms | 80.871ms | 82.583ms | 1.060s | 911.00 MiB | none | 106.77M/s |
| q21 | suppliers who kept orders waiting | 104.000ms | 140.984ms | 140.870ms | 13.2% | 122.524ms | 141.109ms | 120.952ms | 144.025ms | 1.870s | 1.37 GiB | none | 61.48M/s |
| q22 | global sales opportunity | 23.000ms | 40.486ms | 40.471ms | 0.3% | 40.347ms | 40.482ms | 40.332ms | 40.501ms | 70.000ms | 212.23 MiB | none | 214.01M/s |

datafusion datafusion-cli 55.1.0 over 22 of 22 queries. Total 1.144s by its own clock and 1.758s by ours, 1.763s cold, 17.820s of CPU, peak 1.37 GiB, 166.56M/s and 5.83 GiB/s.

Running it cost 54% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.50x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 105.246ms | 224.577ms | 222.453ms | 0.9% | 221.971ms | 224.082ms | 221.490ms | 225.976ms | 1.900s | 955.08 MiB | none | 38.94M/s |
| q03 | shipping priority | 53.746ms | 160.886ms | 161.341ms | 0.2% | 161.108ms | 161.425ms | 160.958ms | 161.802ms | 430.000ms | 325.59 MiB | none | 53.68M/s |
| q04 | order priority checking | 42.103ms | 140.969ms | 142.381ms | 1.1% | 140.980ms | 142.505ms | 140.912ms | 146.673ms | 310.000ms | 233.00 MiB | none | 60.83M/s |
| q05 | local supplier volume | 76.179ms | 181.045ms | 181.109ms | 0.0% | 181.107ms | 181.195ms | 180.965ms | 181.472ms | 580.000ms | 487.19 MiB | none | 47.82M/s |
| q06 | forecasting revenue change | 35.598ms | 120.978ms | 126.221ms | 12.8% | 125.226ms | 141.428ms | 123.776ms | 141.472ms | 300.000ms | 301.98 MiB | none | 68.62M/s |
| q07 | volume shipping | 97.796ms | 201.905ms | 202.555ms | 0.5% | 201.697ms | 202.755ms | 201.486ms | 202.778ms | 1.010s | 423.02 MiB | none | 42.76M/s |
| q08 | national market share | 204.583ms | 324.948ms | 326.343ms | 1.5% | 324.195ms | 329.239ms | 307.992ms | 329.891ms | 3.680s | 1.13 GiB | none | 26.54M/s |
| q09 | product type profit measure | 676.182ms | 873.561ms | 831.865ms | 0.5% | 827.449ms | 831.940ms | 813.484ms | 835.426ms | 11.980s | 2.51 GiB | none | 10.41M/s |
| q10 | returned item reporting | 89.900ms | 182.218ms | 183.960ms | 0.8% | 183.674ms | 185.159ms | 180.939ms | 185.570ms | 460.000ms | 280.91 MiB | none | 47.08M/s |
| q11 | important stock identification | 37.004ms | 141.035ms | 140.946ms | 0.2% | 140.824ms | 141.168ms | 140.767ms | 141.688ms | 200.000ms | 182.73 MiB | none | 61.45M/s |
| q12 | shipping modes and order priority | 38.438ms | 141.286ms | 141.208ms | 0.2% | 141.154ms | 141.412ms | 141.032ms | 145.494ms | 350.000ms | 290.25 MiB | none | 61.34M/s |
| q14 | promotion effect | 38.975ms | 145.110ms | 141.083ms | 0.3% | 140.818ms | 141.219ms | 128.554ms | 144.639ms | 220.000ms | 185.57 MiB | none | 61.39M/s |
| q15 | top supplier | 36.939ms | 126.891ms | 123.581ms | 14.6% | 122.756ms | 140.842ms | 120.759ms | 145.218ms | 220.000ms | 180.11 MiB | none | 70.09M/s |
| q16 | parts supplier relationship | 47.117ms | 142.851ms | 141.778ms | 0.3% | 141.480ms | 141.876ms | 141.316ms | 141.930ms | 230.000ms | 185.01 MiB | none | 61.09M/s |
| q18 | large volume customer | 350.652ms | 490.523ms | 487.886ms | 3.8% | 486.427ms | 504.950ms | 484.477ms | 508.783ms | 7.440s | 2.35 GiB | none | 17.75M/s |
| q19 | discounted revenue | 44.215ms | 124.036ms | 147.909ms | 3.5% | 143.126ms | 148.241ms | 142.212ms | 148.305ms | 270.000ms | 209.72 MiB | none | 58.56M/s |
| q20 | potential part promotion | 110.367ms | 224.988ms | 225.204ms | 1.3% | 224.462ms | 227.353ms | 223.821ms | 228.845ms | 1.300s | 773.87 MiB | none | 38.46M/s |
| q21 | suppliers who kept orders waiting | 533.967ms | 650.564ms | 675.803ms | 3.0% | 655.941ms | 676.115ms | 652.201ms | 683.951ms | 11.750s | 1.58 GiB | none | 12.82M/s |
| q22 | global sales opportunity | 37.780ms | 121.267ms | 140.896ms | 0.3% | 140.708ms | 141.115ms | 120.963ms | 146.037ms | 180.000ms | 130.63 MiB | none | 61.47M/s |

polars 1.44.2 in sink mode over 19 of 22 queries. Total 2.657s by its own clock and 4.745s by ours, 4.720s cold, 42.810s of CPU, peak 2.51 GiB, 61.94M/s and 2.17 GiB/s.

Running it cost 79% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 6.73x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 34.365ms | 66.698ms | 60.588ms | 0.1% | 60.571ms | 60.623ms | 60.535ms | 60.639ms | 480.000ms | 136.09 MiB | none | 142.95M/s |
| q02 | minimum cost supplier | 17.892ms | 40.545ms | 40.676ms | 0.1% | 40.628ms | 40.681ms | 40.557ms | 43.071ms | 60.000ms | 80.68 MiB | none | 212.93M/s |
| q03 | shipping priority | 37.155ms | 60.631ms | 60.706ms | 0.2% | 60.643ms | 60.756ms | 60.638ms | 60.782ms | 310.000ms | 176.16 MiB | none | 142.67M/s |
| q04 | order priority checking | 23.887ms | 40.537ms | 40.438ms | 0.0% | 40.436ms | 40.445ms | 40.433ms | 40.466ms | 170.000ms | 111.73 MiB | none | 214.19M/s |
| q05 | local supplier volume | 50.428ms | 80.789ms | 80.637ms | 24.7% | 60.885ms | 80.766ms | 60.765ms | 81.019ms | 430.000ms | 210.93 MiB | none | 107.41M/s |
| q06 | forecasting revenue change | 11.938ms | 20.398ms | 20.386ms | 0.6% | 20.373ms | 20.488ms | 20.307ms | 20.500ms | 150.000ms | 116.47 MiB | none | 424.86M/s |
| q07 | volume shipping | 38.681ms | 60.605ms | 60.859ms | 0.4% | 60.624ms | 60.876ms | 60.619ms | 60.944ms | 360.000ms | 179.77 MiB | none | 142.32M/s |
| q08 | national market share | 43.010ms | 60.911ms | 60.920ms | 40.2% | 60.773ms | 85.255ms | 60.695ms | 96.935ms | 380.000ms | 187.97 MiB | none | 142.17M/s |
| q09 | product type profit measure | 127.684ms | 144.261ms | 161.281ms | 0.1% | 161.210ms | 161.321ms | 147.565ms | 161.453ms | 830.000ms | 421.99 MiB | none | 53.70M/s |
| q10 | returned item reporting | 92.236ms | 123.785ms | 121.137ms | 0.2% | 121.116ms | 121.327ms | 121.020ms | 123.457ms | 460.000ms | 272.03 MiB | none | 71.50M/s |
| q11 | important stock identification | 17.452ms | 40.522ms | 40.580ms | 0.2% | 40.535ms | 40.596ms | 40.535ms | 40.711ms | 60.000ms | 75.99 MiB | none | 213.44M/s |
| q12 | shipping modes and order priority | 36.239ms | 73.696ms | 60.726ms | 2.6% | 60.656ms | 62.238ms | 60.522ms | 91.724ms | 390.000ms | 162.27 MiB | none | 142.63M/s |
| q13 | customer distribution | 72.896ms | 80.760ms | 100.830ms | 0.3% | 100.813ms | 101.129ms | 100.748ms | 102.644ms | 640.000ms | 268.00 MiB | none | 85.90M/s |
| q14 | promotion effect | 36.971ms | 60.786ms | 60.664ms | 6.4% | 60.662ms | 64.538ms | 60.639ms | 97.907ms | 250.000ms | 212.21 MiB | none | 142.77M/s |
| q15 | top supplier | 37.664ms | 60.906ms | 60.710ms | 31.7% | 60.690ms | 79.947ms | 60.558ms | 81.134ms | 390.000ms | 178.39 MiB | none | 142.67M/s |
| q16 | parts supplier relationship | 35.112ms | 41.409ms | 41.553ms | 48.2% | 41.423ms | 61.465ms | 41.326ms | 61.465ms | 70.000ms | 71.48 MiB | none | 208.44M/s |
| q17 | small quantity order revenue | 30.316ms | 60.724ms | 40.659ms | 0.3% | 40.640ms | 40.771ms | 40.604ms | 42.041ms | 320.000ms | 126.10 MiB | none | 213.02M/s |
| q18 | large volume customer | 83.647ms | 100.873ms | 100.958ms | 0.0% | 100.950ms | 100.989ms | 100.916ms | 101.009ms | 860.000ms | 288.73 MiB | none | 85.79M/s |
| q19 | discounted revenue | 31.841ms | 40.469ms | 41.591ms | 29.7% | 40.546ms | 52.885ms | 40.435ms | 60.826ms | 270.000ms | 146.51 MiB | none | 208.25M/s |
| q20 | potential part promotion | 82.544ms | 101.493ms | 107.139ms | 11.2% | 102.246ms | 114.295ms | 101.146ms | 142.107ms | 540.000ms | 333.68 MiB | none | 80.84M/s |
| q21 | suppliers who kept orders waiting | 77.112ms | 105.059ms | 105.336ms | 3.7% | 101.491ms | 105.405ms | 100.945ms | 111.099ms | 650.000ms | 216.47 MiB | none | 82.22M/s |
| q22 | global sales opportunity | 72.515ms | 100.899ms | 80.768ms | 0.1% | 80.739ms | 80.791ms | 80.667ms | 100.572ms | 130.000ms | 64.26 MiB | none | 107.24M/s |

rudb rudb 0.3.67 over 22 of 22 queries. Total 1.092s by its own clock and 1.549s by ours, 1.567s cold, 8.200s of CPU, peak 421.99 MiB, 174.56M/s and 6.11 GiB/s.

Running it cost 42% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 7.91x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | 687.626us | 49.371ms | 545.391ms | 545.399ms | 0.0% | 545.391ms | 288.372us | 4.313ms | 25.28 KiB | 6 of 6 |
| q02 | 666.391us | 15.771ms | 64.141ms | 64.201ms | 0.1% | 64.141ms | 214.397us | 0.000us | 5.35 MiB | 47 of 47 |
| q03 | 822.278us | 41.471ms | 288.627ms | 288.647ms | 0.0% | 288.627ms | 356.997us | 20.996ms | 25.27 MiB | 16 of 16 |
| q04 | 761.137us | 22.002ms | 165.708ms | 165.725ms | 0.0% | 165.708ms | 342.110us | 3.933ms | 18.33 MiB | 11 of 11 |
| q05 | 907.947us | 49.038ms | 382.222ms | 382.251ms | 0.0% | 382.222ms | 379.737us | 37.370ms | 39.74 MiB | 27 of 27 |
| q06 | 578.620us | 11.321ms | 146.776ms | 146.784ms | 0.0% | 146.776ms | 252.137us | 0.000us | 896 B | 5 of 5 |
| q07 | 958.855us | 37.180ms | 335.796ms | 335.829ms | 0.0% | 335.796ms | 338.098us | 13.833ms | 20.52 MiB | 30 of 30 |
| q08 | 1.006ms | 37.688ms | 356.122ms | 356.163ms | 0.0% | 356.122ms | 349.792us | 3.488ms | 12.96 MiB | 37 of 37 |
| q09 | 918.226us | 120.578ms | 611.104ms | 611.159ms | 0.0% | 611.104ms | 375.409us | 188.466ms | 195.28 MiB | 27 of 27 |
| q10 | 981.539us | 97.717ms | 420.631ms | 420.660ms | 0.0% | 420.631ms | 361.712us | 88.978ms | 83.03 MiB | 19 of 19 |
| q11 | 575.261us | 16.315ms | 61.203ms | 61.242ms | 0.1% | 61.203ms | 261.987us | 0.000us | 12.28 MiB | 30 of 30 |
| q12 | 746.919us | 45.722ms | 417.886ms | 417.903ms | 0.0% | 417.886ms | 321.769us | 51.775ms | 48.47 MiB | 10 of 10 |
| q13 | 324.084us | 68.234ms | 606.389ms | 606.404ms | 0.0% | 606.389ms | 145.681us | 23.450ms | 35.53 MiB | 12 of 12 |
| q14 | 833.020us | 33.508ms | 262.910ms | 262.926ms | 0.0% | 262.910ms | 324.163us | 0.000us | 97.52 MiB | 9 of 9 |
| q15 | 1.159ms | 36.189ms | 375.792ms | 375.812ms | 0.0% | 375.792ms | 517.828us | 13.670ms | 13.81 MiB | 21 of 21 |
| q16 | 388.280us | 28.284ms | 62.158ms | 62.215ms | 0.1% | 62.158ms | 138.703us | 0.000us | 19.90 MiB | 18 of 18 |
| q17 | 992.915us | 37.798ms | 293.406ms | 293.430ms | 0.0% | 293.406ms | 432.419us | 16.138ms | 3.11 MiB | 21 of 21 |
| q18 | 1.286ms | 80.915ms | 687.761ms | 687.795ms | 0.0% | 687.761ms | 560.797us | 171.644ms | 108.27 MiB | 21 of 21 |
| q19 | 931.958us | 30.400ms | 268.838ms | 268.853ms | 0.0% | 268.838ms | 361.869us | 785.328us | 1.61 MiB | 11 of 11 |
| q20 | 810.898us | 72.278ms | 385.633ms | 385.673ms | 0.0% | 385.633ms | 310.130us | 134.017ms | 103.74 MiB | 30 of 30 |
| q21 | 1.613ms | 81.171ms | 588.247ms | 588.279ms | 0.0% | 588.247ms | 643.547us | 51.077ms | 91.20 MiB | 29 of 29 |
| q22 | 523.115us | 74.993ms | 118.878ms | 118.905ms | 0.0% | 118.878ms | 199.596us | 10.895ms | 22.90 MiB | 19 of 19 |

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
| Aggregate | 0.000us | nothing to share | 29 | 15965639 | 2312314 | 0.0ns | 0.0ns | 29 of 29 |
| FileScan | 0.000us | nothing to share | 89 | 0 | 66114817 | handed none | 0.0ns | 89 of 89 |
| Filter | 0.000us | nothing to share | 54 | 55372398 | 11120073 | 0.0ns | 0.0ns | 54 of 54 |
| Gather | 0.000us | nothing to share | 67 | 3399648 | 0 | 0.0ns | handed on none | 67 of 67 |
| Join | 0.000us | nothing to share | 3 | 81833 | 81833 | 0.0ns | 0.0ns | 3 of 3 |
| Mark | 0.000us | nothing to share | 5 | 1747204 | 218687 | 0.0ns | 0.0ns | 5 of 5 |
| Pad | 0.000us | nothing to share | 1 | 1484298 | 1534302 | 0.0ns | 0.0ns | 1 of 1 |
| Probe | 0.000us | nothing to share | 58 | 8779355 | 5518663 | 0.0ns | 0.0ns | 58 of 58 |
| Project | 0.000us | nothing to share | 132 | 24573176 | 24573176 | 0.0ns | 0.0ns | 132 of 132 |
| Sort | 0.000us | nothing to share | 13 | 19795 | 19795 | 0.0ns | 0.0ns | 13 of 13 |
| TopN | 0.000us | nothing to share | 5 | 50515 | 287 | 0.0ns | 0.0ns | 5 of 5 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is Aggregate, and the queries where it cost the most are q01 at 0.000us, q02 at 0.000us, q03 at 0.000us.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- q03 swung by 33.3% of its median, and rule two wants under 10%
- q14 swung by 49.2% of its median, and rule two wants under 10%
- q17 swung by 33.6% of its median, and rule two wants under 10%

These swung wider than reporting rule two allows:

- duckdb swung by 49.2% of its median on q14, and rule two wants under 10%
- duckdb-pinned swung by 33.3% of its median on q17, and rule two wants under 10%
- clickhouse-local swung by 15.6% of its median on q07, and rule two wants under 10%
- datafusion swung by 31.4% of its median on q06, and rule two wants under 10%
- polars swung by 14.6% of its median on q15, and rule two wants under 10%
- rudb swung by 48.2% of its median on q16, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

These ran every query at about the same speed:

- clickhouse-local ran every query within 1.90x of every other one, and this suite spreads over 6x on an engine it is measuring

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

Answers differ, so this is not a comparison: q15: polars does not agree with duckdb: 2 numbers against 2

Answers differ, so this is not a comparison: q19: polars does not agree with duckdb: 1 numbers against 1

Hot here means page cache warm and not buffer pool warm, because every run is a fresh process so that no query's number depends on the one before it.

