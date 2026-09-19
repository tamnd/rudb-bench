# tpch on gamingpc-wsl

This is one run of the tpch suite on gamingpc-wsl, over 5 engines and 22 queries, with 5 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

## What ran

| what | it was |
| --- | --- |
| suite | tpch |
| queries | 22 |
| tables | 33.20 GiB of Parquet in 8 tables |
| rows | the suite does not declare one |
| summary | median with the interquartile range, per reporting rule two |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run tpch --engines duckdb,duckdb-pinned,clickhouse-local,datafusion,rudb --runs 5 --report
```

The data goes under `RUDB_BENCH_DATA`, which defaults to `~/rudb-data`, and the engines have to be on the path. An engine that is not gets a row in the abstentions below rather than being left out of the table.

## The machine

| fact | value | source |
| --- | --- | --- |

A fact that says it was not read is a fact about this machine and not a gap in the report. A field that had silently defaulted would be a lie that survived into it.

## The engines

| engine | version | state | load | load cpu | on disk | that size is | format | machine load |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 49.879s | 1125.220s | 26.24 GiB | its own database file | its own | 4.90 to 28.56 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 57.578s | 1338.580s | 27.96 GiB | its own database file | its own | 28.56 to 29.65 |
| clickhouse-local | 26.9.1.1562 | ran | 92.422s | 1313.190s | 28.54 GiB | its own MergeTree parts, as system.parts counts the active ones | its own | 27.43 to 26.37 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 33.20 GiB | the source Parquet, this engine has no storage format of its own | the Parquet | 26.37 to 35.91 |
| rudb | rudb 0.3.57 | ran | 0.000us | 0.000us | 33.20 GiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 5.74 to 4.90 |
| polars | unknown | nothing saved for it on this machine yet | n/a | n/a | n/a | n/a | n/a | n/a |
| clickhouse-server | unknown | nothing saved for it on this machine yet | n/a | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: datafusion, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

These engines started while the machine was already above half of that, which means they were measured against somebody else's work rather than on an idle box: duckdb-pinned, clickhouse-local, datafusion. Their numbers are inflated by an amount nothing here can recover, and by a different amount each, depending on how much of the column was CPU bound. One case where that reading is unfair is a sweep that runs engines back to back, where the average has not had a minute to come down from the engine before.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb on 16 shared |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 28.527s | 31.107s | +9% | 32.831s | 523.250s | 16.82 | 16.64 GiB | none | no row count | 25.61 GiB/s | 1.00x |
| duckdb-pinned | 26.740s | 31.634s | +18% | 34.001s | 408.950s | 12.93 | 16.82 GiB | 322.74 MiB | no row count | 27.32 GiB/s | 0.93x |
| clickhouse-local | 130.667s | 135.607s | +4% | 136.292s | 2332.800s | 17.20 | 17.03 GiB | 4.59 GiB | no row count | 5.59 GiB/s | 3.84x |
| datafusion | 118.513s | 121.156s | +2% | 147.124s | 2255.540s | 18.62 | 28.26 GiB | 15.16 GiB | no row count | 6.16 GiB/s | 2.43x |
| rudb | 120.226s | 122.255s | +2% | 163.822s | 994.850s | 8.14 | 16.48 GiB | 1.07 GiB | no row count | 4.42 GiB/s | 6.91x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 22 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | rudb |
| --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 813.000ms | 1.177s | 3.637s | 3.586s | did not run |
| q02 | minimum cost supplier | 251.000ms | 205.000ms | 24.614s | 1.101s | 2.349s |
| q03 | shipping priority | 1.020s | 1.424s | 2.583s | 2.399s | 4.843s |
| q04 | order priority checking | 979.000ms | 1.188s | 1.922s | 1.169s | 16.213s |
| q05 | local supplier volume | 1.142s | 1.105s | 2.337s | 5.305s | 7.337s |
| q06 | forecasting revenue change | 744.000ms | 908.000ms | 1.309s | 875.000ms | 886.216ms |
| q07 | volume shipping | 1.425s | 1.368s | 2.567s | 8.255s | did not run |
| q08 | national market share | 1.431s | 1.417s | 3.012s | 4.113s | 4.819s |
| q09 | product type profit measure | 3.017s | 3.032s | 40.949s | 9.973s | did not run |
| q10 | returned item reporting | 2.289s | 1.491s | 3.027s | 3.648s | 6.117s |
| q11 | important stock identification | 200.000ms | 213.000ms | 439.000ms | 923.000ms | 1.469s |
| q12 | shipping modes and order priority | 1.594s | 1.454s | 1.986s | 1.260s | 3.627s |
| q13 | customer distribution | 2.067s | 1.552s | 3.686s | 2.728s | 11.666s |
| q14 | promotion effect | 1.103s | 1.155s | 1.379s | 1.480s | did not run |
| q15 | top supplier | 1.056s | 984.000ms | 3.006s | 2.673s | 5.299s |
| q16 | parts supplier relationship | 549.000ms | 526.000ms | 575.000ms | 2.996s | 16.361s |
| q17 | small quantity order revenue | 1.109s | 1.076s | 13.529s | 8.042s | 16.165s |
| q18 | large volume customer | 2.228s | 2.101s | 5.275s | 21.992s | did not run |
| q19 | discounted revenue | 1.413s | 1.344s | 2.506s | 2.103s | 5.102s |
| q20 | potential part promotion | 1.185s | 1.030s | 1.638s | 2.626s | 10.269s |
| q21 | suppliers who kept orders waiting | 2.549s | 1.659s | 10.018s | 30.913s | did not run |
| q22 | global sales opportunity | 363.000ms | 331.000ms | 673.000ms | 353.000ms | 7.704s |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 813.000ms | 1.250s | 912.709ms | 0.7% | 909.033ms | 915.456ms | 890.138ms | 929.902ms | 18.400s | 5.07 GiB | 5.01 GiB | no row count |
| q02 | minimum cost supplier | 251.000ms | 322.871ms | 279.481ms | 6.7% | 278.758ms | 297.353ms | 272.123ms | 314.296ms | 2.460s | 1020.29 MiB | 561.59 MiB | no row count |
| q03 | shipping priority | 1.020s | 1.263s | 1.134s | 1.4% | 1.134s | 1.149s | 1.117s | 1.174s | 23.130s | 6.83 GiB | 2.54 GiB | no row count |
| q04 | order priority checking | 979.000ms | 1.267s | 1.071s | 1.0% | 1.061s | 1.072s | 1.051s | 1.102s | 17.080s | 5.10 GiB | 4.28 GiB | no row count |
| q05 | local supplier volume | 1.142s | 1.414s | 1.262s | 1.6% | 1.258s | 1.277s | 1.248s | 1.287s | 25.180s | 7.65 GiB | 3.12 GiB | no row count |
| q06 | forecasting revenue change | 744.000ms | 863.215ms | 824.933ms | 14.3% | 817.159ms | 935.444ms | 795.227ms | 965.859ms | 7.520s | 5.02 GiB | none | no row count |
| q07 | volume shipping | 1.425s | 1.644s | 1.554s | 1.6% | 1.530s | 1.555s | 1.427s | 1.572s | 23.660s | 9.22 GiB | none | no row count |
| q08 | national market share | 1.431s | 1.616s | 1.562s | 0.4% | 1.556s | 1.562s | 1.484s | 1.570s | 17.030s | 8.84 GiB | 1.41 GiB | no row count |
| q09 | product type profit measure | 3.017s | 3.278s | 3.255s | 20.0% | 2.978s | 3.630s | 2.969s | 3.699s | 69.190s | 16.64 GiB | 226.70 MiB | no row count |
| q10 | returned item reporting | 2.289s | 2.372s | 2.489s | 0.6% | 2.488s | 2.503s | 1.963s | 2.567s | 33.890s | 10.03 GiB | 1.66 GiB | no row count |
| q11 | important stock identification | 200.000ms | 255.859ms | 229.593ms | 4.9% | 220.474ms | 231.830ms | 220.245ms | 236.062ms | 2.300s | 1.16 GiB | 186.87 MiB | no row count |
| q12 | shipping modes and order priority | 1.594s | 1.629s | 1.697s | 6.0% | 1.611s | 1.712s | 1.550s | 1.719s | 14.230s | 5.71 GiB | 3.53 GiB | no row count |
| q13 | customer distribution | 2.067s | 2.759s | 2.204s | 4.0% | 2.188s | 2.277s | 2.188s | 3.507s | 59.300s | 7.84 GiB | 2.78 GiB | no row count |
| q14 | promotion effect | 1.103s | 1.219s | 1.218s | 2.2% | 1.197s | 1.224s | 1.060s | 1.248s | 14.480s | 7.86 GiB | 55.42 MiB | no row count |
| q15 | top supplier | 1.056s | 1.202s | 1.170s | 6.0% | 1.151s | 1.222s | 1.117s | 1.256s | 12.590s | 7.53 GiB | 1.61 MiB | no row count |
| q16 | parts supplier relationship | 549.000ms | 805.640ms | 608.758ms | 37.0% | 552.645ms | 777.676ms | 543.805ms | 788.915ms | 10.060s | 2.27 GiB | 45.92 MiB | no row count |
| q17 | small quantity order revenue | 1.109s | 1.253s | 1.220s | 4.2% | 1.203s | 1.255s | 1.147s | 1.278s | 17.040s | 6.31 GiB | 3.93 MiB | no row count |
| q18 | large volume customer | 2.228s | 2.217s | 2.414s | 8.2% | 2.221s | 2.418s | 2.149s | 3.019s | 51.510s | 9.32 GiB | 164.19 MiB | no row count |
| q19 | discounted revenue | 1.413s | 1.612s | 1.573s | 2.2% | 1.557s | 1.591s | 1.486s | 1.692s | 24.250s | 10.10 GiB | 736.00 KiB | no row count |
| q20 | potential part promotion | 1.185s | 1.430s | 1.314s | 0.7% | 1.311s | 1.321s | 1.310s | 1.326s | 15.040s | 7.22 GiB | 24.32 MiB | no row count |
| q21 | suppliers who kept orders waiting | 2.549s | 2.734s | 2.723s | 1.3% | 2.695s | 2.729s | 2.684s | 2.735s | 54.960s | 9.18 GiB | 1.74 MiB | no row count |
| q22 | global sales opportunity | 363.000ms | 424.733ms | 391.850ms | 1.3% | 388.037ms | 393.035ms | 384.889ms | 397.435ms | 9.950s | 1.09 GiB | none | no row count |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 22 of 22 queries. Total 28.527s by its own clock and 31.107s by ours, 32.831s cold, 523.250s of CPU, peak 16.64 GiB, no row count and 25.61 GiB/s.

Running it cost 9% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 14.18x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 1.177s | 1.617s | 1.398s | 11.4% | 1.393s | 1.553s | 1.390s | 1.616s | 13.670s | 5.01 GiB | 4.95 GiB | no row count |
| q02 | minimum cost supplier | 205.000ms | 266.168ms | 248.204ms | 6.1% | 246.084ms | 261.202ms | 240.021ms | 264.904ms | 1.370s | 957.09 MiB | 438.96 MiB | no row count |
| q03 | shipping priority | 1.424s | 1.461s | 1.684s | 13.8% | 1.636s | 1.869s | 1.467s | 2.464s | 20.940s | 6.87 GiB | 2.58 GiB | no row count |
| q04 | order priority checking | 1.188s | 1.308s | 1.373s | 3.4% | 1.370s | 1.417s | 1.344s | 1.449s | 14.780s | 4.76 GiB | 4.36 GiB | no row count |
| q05 | local supplier volume | 1.105s | 1.598s | 1.366s | 2.1% | 1.342s | 1.371s | 1.340s | 1.377s | 21.180s | 7.80 GiB | 3.39 GiB | no row count |
| q06 | forecasting revenue change | 908.000ms | 1.088s | 1.094s | 2.0% | 1.073s | 1.094s | 1.064s | 1.096s | 5.800s | 4.97 GiB | none | no row count |
| q07 | volume shipping | 1.368s | 1.673s | 1.667s | 1.1% | 1.653s | 1.671s | 1.651s | 1.690s | 14.850s | 9.47 GiB | none | no row count |
| q08 | national market share | 1.417s | 1.763s | 1.707s | 1.1% | 1.697s | 1.715s | 1.686s | 1.716s | 14.810s | 9.03 GiB | 1.47 GiB | no row count |
| q09 | product type profit measure | 3.032s | 4.857s | 3.411s | 21.0% | 3.072s | 3.788s | 3.017s | 4.012s | 57.370s | 16.82 GiB | 240.23 MiB | no row count |
| q10 | returned item reporting | 1.491s | 1.887s | 1.834s | 0.2% | 1.832s | 1.836s | 1.830s | 1.838s | 32.810s | 10.02 GiB | 1.63 GiB | no row count |
| q11 | important stock identification | 213.000ms | 310.370ms | 253.227ms | 4.2% | 248.034ms | 258.744ms | 245.030ms | 266.044ms | 2.110s | 1.04 GiB | 178.86 MiB | no row count |
| q12 | shipping modes and order priority | 1.454s | 1.691s | 1.663s | 2.1% | 1.640s | 1.675s | 1.592s | 1.711s | 10.690s | 5.76 GiB | 5.00 GiB | no row count |
| q13 | customer distribution | 1.552s | 1.915s | 1.813s | 2.6% | 1.802s | 1.849s | 1.752s | 1.873s | 42.250s | 7.86 GiB | 3.08 GiB | no row count |
| q14 | promotion effect | 1.155s | 1.427s | 1.372s | 2.6% | 1.365s | 1.401s | 1.298s | 1.419s | 11.700s | 7.88 GiB | 53.37 MiB | no row count |
| q15 | top supplier | 984.000ms | 1.182s | 1.212s | 2.6% | 1.185s | 1.216s | 1.124s | 1.254s | 11.690s | 7.39 GiB | 7.58 MiB | no row count |
| q16 | parts supplier relationship | 526.000ms | 651.634ms | 628.314ms | 6.9% | 607.792ms | 651.411ms | 605.830ms | 665.594ms | 7.750s | 2.24 GiB | 53.55 MiB | no row count |
| q17 | small quantity order revenue | 1.076s | 1.279s | 1.284s | 5.7% | 1.272s | 1.345s | 1.260s | 2.269s | 13.450s | 6.39 GiB | 4.60 MiB | no row count |
| q18 | large volume customer | 2.101s | 2.723s | 2.465s | 3.8% | 2.451s | 2.544s | 2.391s | 2.556s | 48.350s | 10.54 GiB | 699.60 MiB | no row count |
| q19 | discounted revenue | 1.344s | 1.659s | 1.620s | 1.0% | 1.616s | 1.631s | 1.592s | 1.633s | 14.580s | 8.76 GiB | 1.83 GiB | no row count |
| q20 | potential part promotion | 1.030s | 1.293s | 1.262s | 0.9% | 1.257s | 1.268s | 1.250s | 1.269s | 13.060s | 7.18 GiB | 994.93 MiB | no row count |
| q21 | suppliers who kept orders waiting | 1.659s | 1.952s | 1.907s | 0.7% | 1.894s | 1.908s | 1.892s | 1.912s | 28.690s | 7.98 GiB | 68.45 MiB | no row count |
| q22 | global sales opportunity | 331.000ms | 400.798ms | 373.950ms | 2.0% | 373.193ms | 380.760ms | 369.675ms | 396.668ms | 7.050s | 1.85 GiB | 3.36 MiB | no row count |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 22 of 22 queries. Total 26.740s by its own clock and 31.634s by ours, 34.001s cold, 408.950s of CPU, peak 16.82 GiB, no row count and 27.32 GiB/s.

Running it cost 18% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 13.74x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 3.637s | 4.164s | 3.827s | 2.5% | 3.772s | 3.867s | 3.713s | 3.873s | 110.960s | 1.70 GiB | 3.79 GiB | no row count |
| q02 | minimum cost supplier | 24.614s | 28.267s | 25.012s | 5.6% | 24.767s | 26.159s | 24.696s | 26.446s | 181.580s | 14.12 GiB | 10.67 GiB | no row count |
| q03 | shipping priority | 2.583s | 3.034s | 2.811s | 1.3% | 2.786s | 2.824s | 2.786s | 2.829s | 77.190s | 2.89 GiB | 3.15 GiB | no row count |
| q04 | order priority checking | 1.922s | 2.254s | 2.127s | 0.8% | 2.120s | 2.137s | 2.083s | 2.143s | 54.820s | 1.94 GiB | 2.16 GiB | no row count |
| q05 | local supplier volume | 2.337s | 2.526s | 2.537s | 2.0% | 2.499s | 2.549s | 2.486s | 2.579s | 69.200s | 2.13 GiB | 137.71 MiB | no row count |
| q06 | forecasting revenue change | 1.309s | 1.501s | 1.477s | 0.4% | 1.473s | 1.479s | 1.470s | 1.484s | 38.340s | 995.82 MiB | 628.95 MiB | no row count |
| q07 | volume shipping | 2.567s | 2.805s | 2.772s | 0.3% | 2.767s | 2.775s | 2.766s | 2.780s | 77.060s | 2.35 GiB | 12.92 MiB | no row count |
| q08 | national market share | 3.012s | 3.287s | 3.243s | 0.8% | 3.220s | 3.246s | 3.220s | 3.397s | 89.010s | 3.82 GiB | 134.94 MiB | no row count |
| q09 | product type profit measure | 40.949s | 36.697s | 41.427s | 13.6% | 37.990s | 43.641s | 37.362s | 43.976s | 343.690s | 17.03 GiB | 7.44 GiB | no row count |
| q10 | returned item reporting | 3.027s | 3.433s | 3.326s | 1.9% | 3.282s | 3.346s | 3.243s | 3.355s | 88.690s | 5.34 GiB | 1.04 GiB | no row count |
| q11 | important stock identification | 439.000ms | 564.513ms | 570.078ms | 5.4% | 559.860ms | 590.839ms | 558.418ms | 613.533ms | 10.490s | 829.70 MiB | 181.18 MiB | no row count |
| q12 | shipping modes and order priority | 1.986s | 2.276s | 2.133s | 0.7% | 2.124s | 2.138s | 2.095s | 2.165s | 57.200s | 1.29 GiB | 3.78 GiB | no row count |
| q13 | customer distribution | 3.686s | 4.229s | 4.003s | 0.8% | 3.980s | 4.010s | 3.979s | 4.135s | 109.070s | 12.23 GiB | 743.35 MiB | no row count |
| q14 | promotion effect | 1.379s | 1.604s | 1.525s | 1.1% | 1.516s | 1.533s | 1.509s | 1.567s | 38.810s | 1.45 GiB | 628.80 MiB | no row count |
| q15 | top supplier | 3.006s | 3.232s | 3.202s | 0.4% | 3.202s | 3.215s | 3.185s | 3.232s | 84.590s | 3.19 GiB | 236.24 MiB | no row count |
| q16 | parts supplier relationship | 575.000ms | 723.528ms | 723.921ms | 1.6% | 713.667ms | 725.353ms | 695.983ms | 753.325ms | 13.880s | 1.65 GiB | 86.64 MiB | no row count |
| q17 | small quantity order revenue | 13.529s | 14.678s | 13.752s | 1.5% | 13.649s | 13.859s | 13.609s | 15.792s | 399.120s | 15.84 GiB | 4.54 GiB | no row count |
| q18 | large volume customer | 5.275s | 5.416s | 5.590s | 6.0% | 5.456s | 5.793s | 5.412s | 5.795s | 150.490s | 13.84 GiB | 1.00 GiB | no row count |
| q19 | discounted revenue | 2.506s | 2.659s | 2.632s | 13.8% | 2.607s | 2.971s | 2.557s | 3.293s | 71.040s | 712.10 MiB | 1.02 GiB | no row count |
| q20 | potential part promotion | 1.638s | 1.937s | 1.772s | 1.2% | 1.756s | 1.777s | 1.737s | 1.856s | 46.040s | 1.03 GiB | 1.59 GiB | no row count |
| q21 | suppliers who kept orders waiting | 10.018s | 10.173s | 10.346s | 6.4% | 10.157s | 10.824s | 9.233s | 10.902s | 208.480s | 7.46 GiB | 476.12 MiB | no row count |
| q22 | global sales opportunity | 673.000ms | 831.901ms | 798.482ms | 1.3% | 797.197ms | 807.364ms | 766.776ms | 823.079ms | 13.050s | 809.77 MiB | 104.32 MiB | no row count |

clickhouse-local 26.9.1.1562 over 22 of 22 queries. Total 130.667s by its own clock and 135.607s by ours, 136.292s cold, 2332.800s of CPU, peak 17.03 GiB, no row count and 5.59 GiB/s.

Running it cost 4% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 72.67x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 3.586s | 3.846s | 3.640s | 1.2% | 3.623s | 3.666s | 3.623s | 3.947s | 100.020s | 754.25 MiB | 5.08 GiB | no row count |
| q02 | minimum cost supplier | 1.101s | 1.256s | 1.168s | 1.8% | 1.156s | 1.177s | 1.152s | 1.226s | 24.230s | 2.29 GiB | 1.13 GiB | no row count |
| q03 | shipping priority | 2.399s | 2.757s | 2.466s | 0.3% | 2.459s | 2.467s | 2.454s | 2.585s | 60.240s | 2.11 GiB | 3.31 GiB | no row count |
| q04 | order priority checking | 1.169s | 1.776s | 1.248s | 11.7% | 1.201s | 1.347s | 1.199s | 1.365s | 25.940s | 6.60 GiB | 9.19 GiB | no row count |
| q05 | local supplier volume | 5.305s | 5.034s | 5.406s | 5.5% | 5.311s | 5.607s | 5.306s | 6.618s | 105.850s | 2.90 GiB | 5.92 GiB | no row count |
| q06 | forecasting revenue change | 875.000ms | 1.280s | 930.782ms | 3.8% | 926.879ms | 962.015ms | 921.874ms | 1.000s | 21.760s | 758.05 MiB | 4.24 MiB | no row count |
| q07 | volume shipping | 8.255s | 13.981s | 8.454s | 3.4% | 8.215s | 8.504s | 8.136s | 8.631s | 225.220s | 11.22 GiB | 728.02 MiB | no row count |
| q08 | national market share | 4.113s | 4.291s | 4.182s | 5.5% | 4.135s | 4.366s | 4.115s | 4.450s | 102.290s | 4.96 GiB | 2.68 GiB | no row count |
| q09 | product type profit measure | 9.973s | 9.748s | 10.214s | 16.7% | 9.143s | 10.848s | 8.941s | 13.073s | 210.370s | 14.44 GiB | 4.93 GiB | no row count |
| q10 | returned item reporting | 3.648s | 4.044s | 3.775s | 8.0% | 3.703s | 4.007s | 3.694s | 4.084s | 92.840s | 6.51 GiB | 11.77 GiB | no row count |
| q11 | important stock identification | 923.000ms | 1.069s | 975.472ms | 0.4% | 972.627ms | 976.095ms | 971.535ms | 977.253ms | 23.970s | 1.29 GiB | 612.11 MiB | no row count |
| q12 | shipping modes and order priority | 1.260s | 1.479s | 1.318s | 0.9% | 1.314s | 1.327s | 1.296s | 1.331s | 32.100s | 1.75 GiB | 3.23 GiB | no row count |
| q13 | customer distribution | 2.728s | 2.840s | 2.791s | 4.5% | 2.769s | 2.894s | 2.753s | 2.989s | 78.990s | 2.73 GiB | 1.45 GiB | no row count |
| q14 | promotion effect | 1.480s | 2.075s | 1.554s | 5.3% | 1.547s | 1.629s | 1.522s | 1.782s | 35.370s | 1.60 GiB | 3.70 GiB | no row count |
| q15 | top supplier | 2.673s | 2.594s | 2.737s | 1.4% | 2.732s | 2.772s | 2.672s | 2.851s | 61.210s | 1.32 GiB | 55.39 MiB | no row count |
| q16 | parts supplier relationship | 2.996s | 12.717s | 3.123s | 14.4% | 2.829s | 3.277s | 2.661s | 3.503s | 15.770s | 5.69 GiB | 435.41 MiB | no row count |
| q17 | small quantity order revenue | 8.042s | 8.337s | 8.110s | 0.5% | 8.098s | 8.136s | 8.096s | 8.697s | 222.200s | 3.21 GiB | 1.32 GiB | no row count |
| q18 | large volume customer | 21.992s | 26.786s | 22.723s | 11.2% | 22.476s | 25.011s | 21.627s | 25.918s | 347.780s | 28.26 GiB | 5.62 GiB | no row count |
| q19 | discounted revenue | 2.103s | 2.790s | 2.160s | 0.6% | 2.155s | 2.168s | 2.149s | 2.226s | 54.100s | 1.38 GiB | 14.70 GiB | no row count |
| q20 | potential part promotion | 2.626s | 2.743s | 2.690s | 0.4% | 2.687s | 2.698s | 2.685s | 2.832s | 70.130s | 4.83 GiB | 899.97 MiB | no row count |
| q21 | suppliers who kept orders waiting | 30.913s | 35.211s | 31.081s | 12.5% | 27.748s | 31.625s | 27.111s | 34.406s | 336.710s | 21.44 GiB | 16.30 GiB | no row count |
| q22 | global sales opportunity | 353.000ms | 468.375ms | 408.245ms | 0.9% | 405.548ms | 409.044ms | 400.929ms | 412.908ms | 8.450s | 1.30 GiB | 617.53 MiB | no row count |

datafusion datafusion-cli 55.1.0 over 22 of 22 queries. Total 118.513s by its own clock and 121.156s by ours, 147.124s cold, 2255.540s of CPU, peak 28.26 GiB, no row count and 6.16 GiB/s.

Running it cost 2% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 76.13x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q02 | minimum cost supplier | 2.349s | 2.820s | 2.443s | 1.4% | 2.431s | 2.465s | 2.413s | 2.540s | 24.620s | 3.05 GiB | 3.62 GiB | no row count |
| q03 | shipping priority | 4.843s | 4.932s | 4.925s | 9.5% | 4.488s | 4.958s | 4.248s | 5.046s | 55.890s | 3.37 GiB | 10.46 MiB | no row count |
| q04 | order priority checking | 16.213s | 37.906s | 16.412s | 4.3% | 16.096s | 16.794s | 16.042s | 33.585s | 100.110s | 16.46 GiB | 1.01 GiB | no row count |
| q05 | local supplier volume | 7.337s | 7.568s | 7.458s | 2.8% | 7.397s | 7.603s | 7.315s | 8.257s | 88.290s | 5.46 GiB | 7.00 GiB | no row count |
| q06 | forecasting revenue change | 886.216ms | 980.617ms | 929.476ms | 0.2% | 928.710ms | 930.392ms | 927.220ms | 933.625ms | 13.270s | 137.04 MiB | 348.40 MiB | no row count |
| q08 | national market share | 4.819s | 5.170s | 4.924s | 0.2% | 4.922s | 4.933s | 4.894s | 4.995s | 62.130s | 3.46 GiB | 2.74 GiB | no row count |
| q10 | returned item reporting | 6.117s | 8.046s | 6.304s | 0.9% | 6.304s | 6.359s | 6.255s | 6.442s | 72.130s | 8.99 GiB | 14.98 GiB | no row count |
| q11 | important stock identification | 1.469s | 1.809s | 1.535s | 5.2% | 1.533s | 1.614s | 1.524s | 1.621s | 9.750s | 1.76 GiB | 3.07 GiB | no row count |
| q12 | shipping modes and order priority | 3.627s | 4.115s | 3.777s | 0.9% | 3.769s | 3.804s | 3.745s | 3.834s | 38.910s | 5.77 GiB | 5.34 GiB | no row count |
| q13 | customer distribution | 11.666s | 12.218s | 11.852s | 0.2% | 11.845s | 11.869s | 11.827s | 11.929s | 106.500s | 10.95 GiB | 3.17 GiB | no row count |
| q15 | top supplier | 5.299s | 5.583s | 5.380s | 1.4% | 5.332s | 5.405s | 5.311s | 5.432s | 59.680s | 2.83 GiB | 5.00 GiB | no row count |
| q16 | parts supplier relationship | 16.361s | 31.525s | 16.573s | 5.3% | 15.949s | 16.830s | 15.163s | 17.691s | 19.200s | 16.48 GiB | 2.81 GiB | no row count |
| q17 | small quantity order revenue | 16.165s | 16.511s | 16.277s | 0.6% | 16.227s | 16.331s | 16.211s | 16.543s | 167.270s | 6.15 GiB | 2.96 GiB | no row count |
| q19 | discounted revenue | 5.102s | 5.463s | 5.184s | 0.2% | 5.174s | 5.186s | 5.156s | 5.198s | 60.960s | 3.09 GiB | 3.07 GiB | no row count |
| q20 | potential part promotion | 10.269s | 11.021s | 10.467s | 4.6% | 10.458s | 10.935s | 10.257s | 11.059s | 79.260s | 10.01 GiB | 2.73 GiB | no row count |
| q22 | global sales opportunity | 7.704s | 8.155s | 7.816s | 0.9% | 7.752s | 7.820s | 7.723s | 8.186s | 36.880s | 4.30 GiB | 2.90 GiB | no row count |

rudb rudb 0.3.57 over 16 of 16 queries. Total 120.226s by its own clock and 122.255s by ours, 163.822s cold, 994.850s of CPU, peak 16.48 GiB, no row count and 4.42 GiB/s.

Running it cost 2% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 17.83x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q02 | 4.725ms | 2.694s | 30.815s | 18.829s | 63.7% | 12.422s | 2.234ms | 9.069s | 1.49 GiB | 42 of 42 |
| q03 | 58.768ms | 4.698s | 92.775s | 52.501s | 76.7% | 41.563s | 31.848ms | 3.547s | 2.44 GiB | 16 of 16 |
| q04 | 54.711ms | 21.452s | 89.979s | 78.233s | 15.0% | 13.421s | 33.625ms | 40.144s | 13.42 GiB | 14 of 14 |
| q05 | 57.106ms | 7.247s | 149.718s | 81.164s | 84.5% | 69.949s | 33.676ms | 7.622s | 4.06 GiB | 27 of 27 |
| q06 | 49.197ms | 874.043ms | 13.618s | 13.618s | 0.0% | 720.548ms | 28.759ms | 73.294ms | 896 B | 5 of 5 |
| q08 | 63.646ms | 4.853s | 101.875s | 60.702s | 67.8% | 42.728s | 36.835ms | 3.131s | 2.26 GiB | 37 of 37 |
| q10 | 53.850ms | 7.647s | 121.621s | 77.697s | 56.5% | 45.158s | 29.432ms | 4.494s | 7.93 GiB | 19 of 19 |
| q11 | 3.333ms | 1.738s | 16.643s | 9.500s | 75.2% | 7.325s | 1.615ms | 358.440ms | 1.23 GiB | 30 of 30 |
| q12 | 56.811ms | 3.694s | 45.592s | 40.627s | 12.2% | 5.892s | 32.569ms | 880.242ms | 5.05 GiB | 10 of 10 |
| q13 | 5.161ms | 11.884s | 123.867s | 87.269s | 41.9% | 37.042s | 2.699ms | 22.928s | 9.65 GiB | 12 of 12 |
| q15 | 110.184ms | 5.342s | 60.239s | 60.238s | 0.0% | 1.540s | 65.469ms | 1.186s | 1.24 GiB | 21 of 21 |
| q16 | 2.417ms | 27.241s | 25.055s | 21.648s | 15.7% | 3.536s | 1.091ms | 470.997ms | 12.30 GiB | 18 of 18 |
| q17 | 105.566ms | 16.220s | 187.244s | 165.839s | 12.9% | 23.466s | 62.447ms | 3.538s | 6.86 GiB | 16 of 16 |
| q19 | 47.522ms | 5.248s | 111.583s | 58.678s | 90.2% | 54.106s | 26.607ms | 4.616s | 2.95 GiB | 11 of 11 |
| q20 | 45.937ms | 10.574s | 75.467s | 69.768s | 8.2% | 7.041s | 26.582ms | 11.645s | 7.13 GiB | 30 of 30 |
| q22 | 6.192ms | 7.995s | 31.453s | 30.675s | 2.5% | 1.042s | 3.131ms | 8.302s | 4.02 GiB | 22 of 22 |

Read from the breakdown the engine wrote for its cold run. `planning` is everything before the first row moved, which is the parse, the bind, the optimizer passes and building the tree. It is a column rather than a footnote because it is the one cost of a query nobody profiles: an optimizer only ever has passes added to it, each paying for itself on the query it was written for, and a query that plans for four hundred milliseconds to save two hundred is a query the optimizer made slower. What is gated is planning as a share of the whole statement, and that share is not in this table, because it is taken over every run rather than off the cold one the rest of these columns come from. It lives in `baselines/planning-<suite>.txt`. `accounted` is what its pipelines charged themselves and `measured` is what it measured around running them, so `apart` is the cross check and anything over five percent is time the breakdown cannot explain. `driver` is the part of `accounted` that was not inside an operator, which is the loop that runs a pipeline rather than the operators it calls. `build` is what the engine spent putting the tree together, which is before there is a pipeline to charge and is why it is off the right hand side of the check. `outside` is the CPU the process spent on everything else, which is starting, opening the data and printing the answer, and it is the reason the wall clock column and the query time column differ. `held` is what the operators reserved, which is not the resident set of the process. `reference` is how many operators ran the reference implementation of their seam, which is the slow path kept for differential testing.

#### Where the time went

| kind | cpu | share | operators | rows in | rows out | per row in | per row out | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Aggregate | 319.013s | 35.0% | 24 | 1509319887 | 258981677 | 211.4ns | 1231.8ns | 24 of 24 |
| FileScan | 306.044s | 33.6% | 62 | 0 | 6116735757 | handed none | 50.0ns | 62 of 62 |
| Probe | 160.763s | 17.7% | 42 | 618731390 | 439685115 | 259.8ns | 365.6ns | 42 of 42 |
| Filter | 94.049s | 10.3% | 42 | 5175375472 | 834363656 | 18.2ns | 112.7ns | 42 of 42 |
| Join | 18.469s | 2.0% | 4 | 20105220 | 20105220 | 918.6ns | 918.6ns | 4 of 4 |
| Project | 9.537s | 1.0% | 97 | 2064579822 | 2064579822 | 4.6ns | 4.6ns | 97 of 97 |
| Gather | 2.515s | 0.3% | 46 | 491097705 | 0 | 5.1ns | handed on none | 46 of 46 |
| TopN | 150.334ms | 0.0% | 3 | 5062366 | 130 | 29.7ns | 1156415.4ns | 3 of 3 |
| Sort | 49.668ms | 0.0% | 10 | 45878 | 45878 | 1082.6ns | 1082.6ns | 10 of 10 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is Aggregate, and the queries where it cost the most are q17 at 132.129s, q04 at 53.423s, q20 at 41.112s.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- q06 swung by 14.3% of its median, and rule two wants under 10%
- q09 swung by 20.0% of its median, and rule two wants under 10%
- q16 swung by 37.0% of its median, and rule two wants under 10%

These swung wider than reporting rule two allows:

- duckdb swung by 37.0% of its median on q16, and rule two wants under 10%
- duckdb-pinned swung by 21.0% of its median on q09, and rule two wants under 10%
- clickhouse-local swung by 13.8% of its median on q19, and rule two wants under 10%
- datafusion swung by 16.7% of its median on q09, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

rudb ran q01 and it failed, INTERNAL Error: an unscaled decimal of 11016932248183655467 does not fit the run its precision chose.

rudb ran q07 and it failed, Out of Memory Error: could not allocate 19.0 MiB (25.0 GiB/25.0 GiB used).

rudb ran q09 and it failed, Out of Memory Error: could not allocate 3.0 GiB (22.9 GiB/25.0 GiB used).

rudb ran q14 and it failed, Out of Range Error: Overflow in multiplication of DECIMAL(18) (10000 * 452593436477868). You might want to add an explicit cast to a bigger decimal.

rudb ran q18 and it failed, Out of Memory Error: could not allocate 288.0 MiB (25.0 GiB/25.0 GiB used).

rudb ran q21 and it failed, Out of Memory Error: could not allocate 4.5 GiB (22.5 GiB/25.0 GiB used).

So the rudb column is 16 of 22 queries, and the 6 that failed are a defect in the engine rather than a gap in this harness.

All 5 engines agreed on every answer the data settles, which is 22 of 22 queries, to the last significant digit of a double.

Hot here means page cache warm and not buffer pool warm, because every run is a fresh process so that no query's number depends on the one before it.

