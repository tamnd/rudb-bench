# clickbench on gamingpc-wsl

This is one run of the clickbench suite on gamingpc-wsl, over 6 engines and 43 queries, with 5 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

It ran over 999975 rows, one out of every 100 of the 99997497 in the full file, which is a development loop and not the suite. Nothing here is comparable to a full run or to anybody else's number.

## What ran

| what | it was |
| --- | --- |
| suite | clickbench |
| queries | 43 |
| tables | 214.81 MiB of Parquet in 1 table |
| rows | 999975 in the table every query reads |
| sample | 999975 rows, one out of every 100 of the 99997497 in the full file |
| corpus | no manifest beside the data, so this run cannot say where it came from |
| summary | median with the interquartile range, per reporting rule two |
| timeout | 600s per query, and no query reached it |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --rows 1000000 --engines duckdb,duckdb-pinned,clickhouse-local,datafusion,polars,rudb --runs 5 --timeout 600 --report
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
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 3.738s | 10.560s | 312.01 MiB | its own database file | its own | 3.88 to 5.32 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 3.640s | 8.650s | 268.76 MiB | its own database file | its own | 5.32 to 4.59 |
| clickhouse-local | 26.9.1.1562 | ran | 801.357ms | 2.370s | 230.62 MiB | its own MergeTree parts, as system.parts counts the active ones | its own | 4.59 to 5.84 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 214.81 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 5.84 to 6.99 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 214.81 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 6.99 to 6.16 |
| rudb | rudb 0.3.45 | ran | 0.000us | 0.000us | 214.81 MiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 6.16 to 6.55 |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb on 39 shared |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 577.000ms | 1.375s | +138% | 1.417s | 3.590s | 2.61 | 307.69 MiB | none | 74.52M/s | 15.63 GiB/s | 1.00x |
| duckdb-pinned | 565.000ms | 2.019s | +257% | 2.045s | 3.620s | 1.79 | 310.49 MiB | none | 76.10M/s | 15.97 GiB/s | 0.97x |
| clickhouse-local | 3.005s | 6.334s | +111% | 6.499s | 8.490s | 1.34 | 469.76 MiB | none | 14.31M/s | 3.00 GiB/s | 6.03x |
| datafusion | 1.199s | 2.486s | +107% | 2.451s | 20.490s | 8.24 | 1.16 GiB | none | 35.86M/s | 7.52 GiB/s | 2.36x |
| polars | 1.222s | 5.118s | +319% | 5.189s | 10.350s | 2.02 | 433.07 MiB | none | 31.92M/s | 6.70 GiB/s | 2.67x |
| rudb | 927.438ms | 1.627s | +75% | 1.665s | 4.310s | 2.65 | 162.24 MiB | none | 46.36M/s | 9.73 GiB/s | 1.78x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 1.000ms | 5.000ms | 1.000ms | 9.799ms | 11.943ms |
| q2 | filtered count | 2.000ms | 2.000ms | 5.000ms | 14.000ms | 11.486ms | 12.277ms |
| q3 | three aggregates | 3.000ms | 2.000ms | 57.000ms | 18.000ms | 11.966ms | 12.468ms |
| q4 | average | 3.000ms | 3.000ms | 41.000ms | 14.000ms | 12.310ms | 12.204ms |
| q5 | count distinct, high card | 12.000ms | 9.000ms | 81.000ms | 26.000ms | 25.219ms | 14.321ms |
| q6 | count distinct, strings | 10.000ms | 7.000ms | 67.000ms | 27.000ms | 25.953ms | 23.169ms |
| q7 | min and max of a date | 1.000ms | 1.000ms | 22.000ms | 1.000ms | 11.735ms | 12.193ms |
| q8 | group by, low card | 2.000ms | 6.000ms | 51.000ms | 15.000ms | 18.361ms | 12.348ms |
| q9 | group by and count distinct | 16.000ms | 13.000ms | 72.000ms | 37.000ms | 41.384ms | 18.084ms |
| q10 | group by, several aggregates | 20.000ms | 15.000ms | 80.000ms | 32.000ms | 47.871ms | 19.932ms |
| q11 | group by a string and count distinct | 7.000ms | 9.000ms | 84.000ms | 27.000ms | 29.147ms | 15.560ms |
| q12 | group by two strings and count distinct | 8.000ms | 8.000ms | 61.000ms | 28.000ms | 31.162ms | 16.044ms |
| q13 | group by a string and top k | 8.000ms | 9.000ms | 64.000ms | 34.000ms | 27.923ms | 20.822ms |
| q14 | group by a string and count distinct | 14.000ms | 13.000ms | 73.000ms | 39.000ms | 39.449ms | 25.581ms |
| q15 | group by two columns and top k | 10.000ms | 11.000ms | 81.000ms | 36.000ms | 29.565ms | 23.632ms |
| q16 | group by, very high card | 14.000ms | 11.000ms | 48.000ms | 29.000ms | 31.505ms | 29.060ms |
| q17 | group by two, very high card | 20.000ms | 19.000ms | 84.000ms | 45.000ms | 46.529ms | 42.099ms |
| q18 | group by two, no ordering | 22.000ms | 18.000ms | 64.000ms | 42.000ms | 36.110ms | 16.017ms |
| q19 | group by with an extract | 26.000ms | 20.000ms | 91.000ms | 51.000ms | 50.748ms | 47.204ms |
| q20 | point lookup | 2.000ms | 3.000ms | 50.000ms | 13.000ms | 10.919ms | 11.796ms |
| q21 | substring scan | 17.000ms | 13.000ms | 74.000ms | 21.000ms | 34.873ms | 22.645ms |
| q22 | substring scan and group by | 17.000ms | 16.000ms | 91.000ms | 33.000ms | 42.203ms | 24.413ms |
| q23 | two substring scans and group by | 23.000ms | 25.000ms | 82.000ms | 44.000ms | 65.681ms | 38.119ms |
| q24 | select star and top k | 50.000ms | 43.000ms | 197.000ms | 82.000ms | 109.688ms | 32.512ms |
| q25 | top k by a date | 6.000ms | 6.000ms | 79.000ms | 22.000ms | 20.883ms | 14.894ms |
| q26 | top k by a string | 6.000ms | 5.000ms | 51.000ms | 25.000ms | 18.225ms | 14.641ms |
| q27 | top k by two columns | 6.000ms | 6.000ms | 77.000ms | 26.000ms | 23.343ms | 15.849ms |
| q28 | group by with a string length | 18.000ms | 18.000ms | 56.000ms | 31.000ms | no dialect | 24.979ms |
| q29 | group by a regular expression | 86.000ms | 89.000ms | 71.000ms | 48.000ms | no dialect | 47.907ms |
| q30 | ninety sums over one column | 5.000ms | 16.000ms | 31.000ms | 23.000ms | 20.557ms | 13.877ms |
| q31 | group by two and several aggregates | 8.000ms | 10.000ms | 84.000ms | 30.000ms | 33.746ms | 18.351ms |
| q32 | group by a high card pair | 9.000ms | 12.000ms | 75.000ms | 30.000ms | 28.603ms | 18.291ms |
| q33 | group by a high card pair, unfiltered | 19.000ms | 20.000ms | 83.000ms | 41.000ms | 43.317ms | 26.842ms |
| q34 | group by a long string | 31.000ms | 29.000ms | 82.000ms | 56.000ms | 55.239ms | 48.663ms |
| q35 | group by a constant and a long string | 33.000ms | 30.000ms | 77.000ms | 59.000ms | 60.107ms | 48.916ms |
| q36 | group by four expressions | 13.000ms | 11.000ms | 51.000ms | 32.000ms | no dialect | 26.402ms |
| q37 | date range and group by a URL | 5.000ms | 4.000ms | 80.000ms | 11.000ms | 21.554ms | 13.069ms |
| q38 | date range and group by a title | 4.000ms | 4.000ms | 95.000ms | 10.000ms | 21.587ms | 13.770ms |
| q39 | date range, group by and offset | 4.000ms | 5.000ms | 90.000ms | 9.000ms | 17.762ms | 12.910ms |
| q40 | date range, a case and a wide group by | 6.000ms | 7.000ms | 79.000ms | 13.000ms | 20.001ms | 16.815ms |
| q41 | date range with an IN and a hash | 3.000ms | 4.000ms | 70.000ms | 8.000ms | 17.949ms | 12.153ms |
| q42 | date range and a deep offset | 4.000ms | 8.000ms | 76.000ms | 8.000ms | 17.265ms | 12.047ms |
| q43 | minute buckets over a date range | 3.000ms | 4.000ms | 73.000ms | 8.000ms | no dialect | 12.618ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 20.537ms | 20.420ms | 0.3% | 20.387ms | 20.441ms | 20.351ms | 20.487ms | 0.000us | 27.25 MiB | none | 48.97M/s |
| q2 | filtered count | 2.000ms | 20.579ms | 20.344ms | 0.2% | 20.336ms | 20.379ms | 20.310ms | 20.385ms | 10.000ms | 29.46 MiB | none | 49.15M/s |
| q3 | three aggregates | 3.000ms | 20.368ms | 20.421ms | 0.4% | 20.364ms | 20.439ms | 20.347ms | 20.456ms | 10.000ms | 31.68 MiB | none | 48.97M/s |
| q4 | average | 3.000ms | 20.298ms | 20.414ms | 0.3% | 20.366ms | 20.425ms | 20.327ms | 20.714ms | 10.000ms | 36.74 MiB | none | 48.98M/s |
| q5 | count distinct, high card | 12.000ms | 40.375ms | 40.334ms | 0.2% | 40.326ms | 40.390ms | 40.325ms | 40.402ms | 70.000ms | 89.74 MiB | none | 24.79M/s |
| q6 | count distinct, strings | 10.000ms | 40.412ms | 20.266ms | 0.4% | 20.258ms | 20.336ms | 20.253ms | 40.349ms | 60.000ms | 63.99 MiB | none | 49.34M/s |
| q7 | min and max of a date | 1.000ms | 20.264ms | 20.255ms | 0.1% | 20.236ms | 20.256ms | 20.234ms | 20.264ms | 0.000us | 27.49 MiB | none | 49.37M/s |
| q8 | group by, low card | 2.000ms | 20.251ms | 20.256ms | 0.0% | 20.251ms | 20.257ms | 20.246ms | 20.277ms | 10.000ms | 31.25 MiB | none | 49.37M/s |
| q9 | group by and count distinct | 16.000ms | 42.521ms | 40.404ms | 0.2% | 40.391ms | 40.458ms | 40.320ms | 40.620ms | 120.000ms | 112.98 MiB | none | 24.75M/s |
| q10 | group by, several aggregates | 20.000ms | 40.353ms | 40.334ms | 0.1% | 40.325ms | 40.367ms | 40.321ms | 40.372ms | 150.000ms | 124.68 MiB | none | 24.79M/s |
| q11 | group by a string and count distinct | 7.000ms | 20.253ms | 20.258ms | 0.0% | 20.257ms | 20.261ms | 20.241ms | 20.275ms | 40.000ms | 61.00 MiB | none | 49.36M/s |
| q12 | group by two strings and count distinct | 8.000ms | 20.255ms | 20.264ms | 0.1% | 20.256ms | 20.269ms | 20.251ms | 20.297ms | 40.000ms | 64.93 MiB | none | 49.35M/s |
| q13 | group by a string and top k | 8.000ms | 20.258ms | 20.256ms | 0.0% | 20.254ms | 20.258ms | 20.250ms | 20.285ms | 40.000ms | 68.25 MiB | none | 49.37M/s |
| q14 | group by a string and count distinct | 14.000ms | 40.475ms | 40.361ms | 0.0% | 40.360ms | 40.372ms | 40.336ms | 40.427ms | 90.000ms | 116.75 MiB | none | 24.78M/s |
| q15 | group by two columns and top k | 10.000ms | 20.263ms | 20.243ms | 0.1% | 20.243ms | 20.269ms | 20.243ms | 20.301ms | 60.000ms | 70.99 MiB | none | 49.40M/s |
| q16 | group by, very high card | 14.000ms | 40.318ms | 40.363ms | 0.0% | 40.351ms | 40.370ms | 40.349ms | 40.372ms | 100.000ms | 107.04 MiB | none | 24.77M/s |
| q17 | group by two, very high card | 20.000ms | 40.348ms | 40.365ms | 0.2% | 40.362ms | 40.428ms | 40.348ms | 40.446ms | 160.000ms | 160.64 MiB | none | 24.77M/s |
| q18 | group by two, no ordering | 22.000ms | 40.346ms | 40.398ms | 0.2% | 40.348ms | 40.428ms | 40.333ms | 41.886ms | 160.000ms | 174.86 MiB | none | 24.75M/s |
| q19 | group by with an extract | 26.000ms | 40.370ms | 40.565ms | 49.4% | 40.474ms | 60.527ms | 40.356ms | 60.548ms | 220.000ms | 179.85 MiB | none | 24.65M/s |
| q20 | point lookup | 2.000ms | 20.302ms | 20.284ms | 0.3% | 20.266ms | 20.324ms | 20.250ms | 20.460ms | 10.000ms | 36.71 MiB | none | 49.30M/s |
| q21 | substring scan | 17.000ms | 40.552ms | 40.439ms | 0.2% | 40.387ms | 40.465ms | 40.350ms | 40.502ms | 100.000ms | 97.50 MiB | none | 24.73M/s |
| q22 | substring scan and group by | 17.000ms | 40.390ms | 40.373ms | 0.0% | 40.371ms | 40.380ms | 40.349ms | 40.424ms | 90.000ms | 112.49 MiB | none | 24.77M/s |
| q23 | two substring scans and group by | 23.000ms | 40.335ms | 40.367ms | 0.2% | 40.363ms | 40.436ms | 40.340ms | 40.446ms | 130.000ms | 134.25 MiB | none | 24.77M/s |
| q24 | select star and top k | 50.000ms | 80.534ms | 80.632ms | 0.1% | 80.604ms | 80.654ms | 80.591ms | 81.350ms | 230.000ms | 204.85 MiB | none | 12.40M/s |
| q25 | top k by a date | 6.000ms | 20.306ms | 20.305ms | 0.4% | 20.267ms | 20.340ms | 20.258ms | 20.367ms | 30.000ms | 47.83 MiB | none | 49.25M/s |
| q26 | top k by a string | 6.000ms | 40.355ms | 20.380ms | 0.3% | 20.330ms | 20.387ms | 20.265ms | 20.419ms | 30.000ms | 38.49 MiB | none | 49.07M/s |
| q27 | top k by two columns | 6.000ms | 20.282ms | 20.304ms | 0.0% | 20.301ms | 20.307ms | 20.265ms | 40.454ms | 30.000ms | 42.51 MiB | none | 49.25M/s |
| q28 | group by with a string length | 18.000ms | 40.352ms | 40.403ms | 0.4% | 40.379ms | 40.559ms | 40.365ms | 40.639ms | 90.000ms | 109.25 MiB | none | 24.75M/s |
| q29 | group by a regular expression | 86.000ms | 100.684ms | 100.603ms | 0.0% | 100.579ms | 100.608ms | 100.561ms | 100.625ms | 600.000ms | 160.58 MiB | none | 9.94M/s |
| q30 | ninety sums over one column | 5.000ms | 20.269ms | 20.259ms | 0.1% | 20.248ms | 20.259ms | 20.210ms | 20.259ms | 10.000ms | 33.43 MiB | none | 49.36M/s |
| q31 | group by two and several aggregates | 8.000ms | 20.279ms | 20.259ms | 0.0% | 20.259ms | 20.260ms | 20.247ms | 20.270ms | 40.000ms | 70.10 MiB | none | 49.36M/s |
| q32 | group by a high card pair | 9.000ms | 20.311ms | 20.263ms | 0.0% | 20.259ms | 20.264ms | 20.258ms | 40.358ms | 50.000ms | 78.84 MiB | none | 49.35M/s |
| q33 | group by a high card pair, unfiltered | 19.000ms | 40.444ms | 40.375ms | 0.1% | 40.372ms | 40.396ms | 40.368ms | 40.417ms | 160.000ms | 199.33 MiB | none | 24.77M/s |
| q34 | group by a long string | 31.000ms | 60.452ms | 60.464ms | 0.1% | 60.460ms | 60.537ms | 60.459ms | 61.231ms | 230.000ms | 298.50 MiB | none | 16.54M/s |
| q35 | group by a constant and a long string | 33.000ms | 60.465ms | 60.533ms | 0.4% | 60.526ms | 60.741ms | 60.434ms | 60.899ms | 240.000ms | 307.69 MiB | none | 16.52M/s |
| q36 | group by four expressions | 13.000ms | 40.358ms | 40.410ms | 0.2% | 40.363ms | 40.440ms | 40.363ms | 42.594ms | 100.000ms | 112.40 MiB | none | 24.75M/s |
| q37 | date range and group by a URL | 5.000ms | 20.299ms | 20.254ms | 0.1% | 20.246ms | 20.259ms | 20.245ms | 20.260ms | 10.000ms | 38.25 MiB | none | 49.37M/s |
| q38 | date range and group by a title | 4.000ms | 20.263ms | 20.260ms | 0.2% | 20.258ms | 20.290ms | 20.253ms | 20.302ms | 10.000ms | 37.03 MiB | none | 49.36M/s |
| q39 | date range, group by and offset | 4.000ms | 20.261ms | 20.250ms | 0.1% | 20.246ms | 20.257ms | 20.245ms | 20.257ms | 10.000ms | 37.25 MiB | none | 49.38M/s |
| q40 | date range, a case and a wide group by | 6.000ms | 20.249ms | 20.258ms | 0.1% | 20.251ms | 20.263ms | 20.251ms | 20.283ms | 20.000ms | 45.75 MiB | none | 49.36M/s |
| q41 | date range with an IN and a hash | 3.000ms | 20.251ms | 20.266ms | 0.0% | 20.261ms | 20.269ms | 20.260ms | 20.276ms | 10.000ms | 37.66 MiB | none | 49.34M/s |
| q42 | date range and a deep offset | 4.000ms | 20.252ms | 20.252ms | 0.0% | 20.249ms | 20.253ms | 20.239ms | 20.257ms | 10.000ms | 36.29 MiB | none | 49.38M/s |
| q43 | minute buckets over a date range | 3.000ms | 20.248ms | 20.257ms | 0.0% | 20.254ms | 20.260ms | 20.249ms | 20.273ms | 0.000us | 35.71 MiB | none | 49.36M/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 577.000ms by its own clock and 1.375s by ours, 1.417s cold, 3.590s of CPU, peak 307.69 MiB, 74.52M/s and 15.63 GiB/s.

Running it cost 138% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 4.97x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 40.322ms | 40.342ms | 0.1% | 40.333ms | 40.353ms | 40.332ms | 40.423ms | 10.000ms | 39.79 MiB | none | 24.79M/s |
| q2 | filtered count | 2.000ms | 40.362ms | 40.366ms | 0.0% | 40.358ms | 40.373ms | 40.345ms | 40.424ms | 20.000ms | 41.57 MiB | none | 24.77M/s |
| q3 | three aggregates | 2.000ms | 40.329ms | 40.347ms | 0.0% | 40.346ms | 40.365ms | 40.343ms | 40.371ms | 20.000ms | 43.79 MiB | none | 24.78M/s |
| q4 | average | 3.000ms | 40.354ms | 40.352ms | 1.5% | 40.345ms | 40.935ms | 40.333ms | 40.961ms | 20.000ms | 48.77 MiB | none | 24.78M/s |
| q5 | count distinct, high card | 9.000ms | 40.427ms | 40.443ms | 4.5% | 40.380ms | 42.187ms | 40.311ms | 73.207ms | 70.000ms | 96.60 MiB | none | 24.73M/s |
| q6 | count distinct, strings | 7.000ms | 40.430ms | 40.364ms | 0.1% | 40.331ms | 40.365ms | 40.312ms | 40.398ms | 50.000ms | 81.04 MiB | none | 24.77M/s |
| q7 | min and max of a date | 1.000ms | 40.833ms | 40.374ms | 1.3% | 40.312ms | 40.822ms | 40.312ms | 40.886ms | 20.000ms | 40.11 MiB | none | 24.77M/s |
| q8 | group by, low card | 6.000ms | 40.295ms | 40.488ms | 0.4% | 40.397ms | 40.546ms | 40.312ms | 40.654ms | 20.000ms | 44.36 MiB | none | 24.70M/s |
| q9 | group by and count distinct | 13.000ms | 63.972ms | 40.486ms | 0.4% | 40.393ms | 40.541ms | 40.354ms | 42.792ms | 80.000ms | 114.39 MiB | none | 24.70M/s |
| q10 | group by, several aggregates | 15.000ms | 40.338ms | 40.475ms | 0.3% | 40.354ms | 40.495ms | 40.329ms | 40.582ms | 110.000ms | 133.62 MiB | none | 24.71M/s |
| q11 | group by a string and count distinct | 9.000ms | 40.467ms | 40.485ms | 0.0% | 40.473ms | 40.493ms | 40.453ms | 46.333ms | 40.000ms | 69.91 MiB | none | 24.70M/s |
| q12 | group by two strings and count distinct | 8.000ms | 40.468ms | 40.540ms | 0.4% | 40.527ms | 40.682ms | 40.342ms | 45.751ms | 40.000ms | 72.10 MiB | none | 24.67M/s |
| q13 | group by a string and top k | 9.000ms | 40.423ms | 40.423ms | 0.2% | 40.358ms | 40.459ms | 40.347ms | 40.462ms | 50.000ms | 84.61 MiB | none | 24.74M/s |
| q14 | group by a string and count distinct | 13.000ms | 40.354ms | 40.375ms | 0.2% | 40.347ms | 40.443ms | 40.345ms | 60.542ms | 80.000ms | 131.10 MiB | none | 24.77M/s |
| q15 | group by two columns and top k | 11.000ms | 40.366ms | 40.393ms | 0.1% | 40.382ms | 40.430ms | 40.350ms | 40.563ms | 50.000ms | 92.36 MiB | none | 24.76M/s |
| q16 | group by, very high card | 11.000ms | 40.359ms | 41.117ms | 1.7% | 40.766ms | 41.481ms | 40.392ms | 42.149ms | 80.000ms | 105.17 MiB | none | 24.32M/s |
| q17 | group by two, very high card | 19.000ms | 61.612ms | 60.479ms | 0.0% | 60.479ms | 60.485ms | 60.439ms | 60.605ms | 170.000ms | 185.67 MiB | none | 16.53M/s |
| q18 | group by two, no ordering | 18.000ms | 60.422ms | 60.433ms | 0.1% | 60.417ms | 60.462ms | 60.395ms | 60.606ms | 130.000ms | 193.67 MiB | none | 16.55M/s |
| q19 | group by with an extract | 20.000ms | 61.973ms | 60.441ms | 0.0% | 60.427ms | 60.454ms | 60.427ms | 60.462ms | 170.000ms | 189.45 MiB | none | 16.54M/s |
| q20 | point lookup | 3.000ms | 40.348ms | 40.365ms | 0.4% | 40.335ms | 40.513ms | 40.312ms | 40.514ms | 30.000ms | 48.03 MiB | none | 24.77M/s |
| q21 | substring scan | 13.000ms | 40.436ms | 40.370ms | 0.1% | 40.350ms | 40.385ms | 40.331ms | 40.495ms | 110.000ms | 102.60 MiB | none | 24.77M/s |
| q22 | substring scan and group by | 16.000ms | 60.449ms | 40.394ms | 0.3% | 40.358ms | 40.497ms | 40.355ms | 60.946ms | 110.000ms | 120.30 MiB | none | 24.76M/s |
| q23 | two substring scans and group by | 25.000ms | 60.499ms | 60.435ms | 0.2% | 60.425ms | 60.526ms | 60.414ms | 60.806ms | 130.000ms | 155.59 MiB | none | 16.55M/s |
| q24 | select star and top k | 43.000ms | 80.488ms | 80.629ms | 0.1% | 80.560ms | 80.662ms | 80.500ms | 80.817ms | 180.000ms | 215.92 MiB | none | 12.40M/s |
| q25 | top k by a date | 6.000ms | 40.502ms | 40.480ms | 0.3% | 40.418ms | 40.541ms | 40.374ms | 40.688ms | 30.000ms | 61.99 MiB | none | 24.70M/s |
| q26 | top k by a string | 5.000ms | 40.314ms | 40.497ms | 0.0% | 40.484ms | 40.498ms | 40.366ms | 43.276ms | 30.000ms | 56.04 MiB | none | 24.69M/s |
| q27 | top k by two columns | 6.000ms | 40.328ms | 40.402ms | 0.2% | 40.360ms | 40.443ms | 40.345ms | 41.160ms | 30.000ms | 62.49 MiB | none | 24.75M/s |
| q28 | group by with a string length | 18.000ms | 40.329ms | 60.443ms | 0.3% | 60.408ms | 60.586ms | 40.361ms | 60.652ms | 110.000ms | 114.38 MiB | none | 16.54M/s |
| q29 | group by a regular expression | 89.000ms | 120.961ms | 120.835ms | 0.2% | 120.738ms | 120.949ms | 120.737ms | 140.958ms | 560.000ms | 181.15 MiB | none | 8.28M/s |
| q30 | ninety sums over one column | 16.000ms | 40.340ms | 40.343ms | 0.0% | 40.340ms | 40.349ms | 40.333ms | 40.359ms | 30.000ms | 54.86 MiB | none | 24.79M/s |
| q31 | group by two and several aggregates | 10.000ms | 40.388ms | 40.379ms | 0.2% | 40.343ms | 40.411ms | 40.342ms | 40.922ms | 70.000ms | 92.93 MiB | none | 24.76M/s |
| q32 | group by a high card pair | 12.000ms | 40.376ms | 40.372ms | 0.1% | 40.368ms | 40.426ms | 40.339ms | 40.501ms | 70.000ms | 99.67 MiB | none | 24.77M/s |
| q33 | group by a high card pair, unfiltered | 20.000ms | 60.463ms | 60.485ms | 0.2% | 60.436ms | 60.552ms | 60.425ms | 60.604ms | 190.000ms | 214.62 MiB | none | 16.53M/s |
| q34 | group by a long string | 29.000ms | 60.729ms | 60.506ms | 0.4% | 60.455ms | 60.682ms | 60.391ms | 60.741ms | 210.000ms | 303.36 MiB | none | 16.53M/s |
| q35 | group by a constant and a long string | 30.000ms | 60.449ms | 60.492ms | 0.0% | 60.488ms | 60.497ms | 60.439ms | 60.740ms | 230.000ms | 310.49 MiB | none | 16.53M/s |
| q36 | group by four expressions | 11.000ms | 40.344ms | 40.370ms | 0.2% | 40.355ms | 40.427ms | 40.345ms | 43.142ms | 80.000ms | 108.14 MiB | none | 24.77M/s |
| q37 | date range and group by a URL | 4.000ms | 40.369ms | 40.374ms | 0.1% | 40.373ms | 40.408ms | 40.354ms | 40.863ms | 30.000ms | 51.36 MiB | none | 24.77M/s |
| q38 | date range and group by a title | 4.000ms | 40.387ms | 40.382ms | 0.3% | 40.353ms | 40.476ms | 40.351ms | 42.399ms | 30.000ms | 49.36 MiB | none | 24.76M/s |
| q39 | date range, group by and offset | 5.000ms | 40.375ms | 40.335ms | 0.0% | 40.332ms | 40.347ms | 40.325ms | 40.351ms | 20.000ms | 48.87 MiB | none | 24.79M/s |
| q40 | date range, a case and a wide group by | 7.000ms | 40.369ms | 40.351ms | 0.0% | 40.349ms | 40.359ms | 40.342ms | 40.369ms | 30.000ms | 57.86 MiB | none | 24.78M/s |
| q41 | date range with an IN and a hash | 4.000ms | 40.360ms | 40.359ms | 0.0% | 40.356ms | 40.368ms | 40.338ms | 40.379ms | 20.000ms | 49.27 MiB | none | 24.78M/s |
| q42 | date range and a deep offset | 8.000ms | 40.966ms | 40.340ms | 0.0% | 40.338ms | 40.353ms | 40.327ms | 40.358ms | 40.000ms | 49.09 MiB | none | 24.79M/s |
| q43 | minute buckets over a date range | 4.000ms | 40.321ms | 40.349ms | 0.0% | 40.347ms | 40.358ms | 40.343ms | 40.947ms | 20.000ms | 47.93 MiB | none | 24.78M/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 565.000ms by its own clock and 2.019s by ours, 2.045s cold, 3.620s of CPU, peak 310.49 MiB, 76.10M/s and 15.97 GiB/s.

Running it cost 257% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.00x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 5.000ms | 80.477ms | 80.486ms | 0.0% | 80.485ms | 80.497ms | 80.459ms | 100.551ms | 80.000ms | 241.81 MiB | none | 12.42M/s |
| q2 | filtered count | 5.000ms | 80.475ms | 80.484ms | 0.1% | 80.474ms | 80.523ms | 80.465ms | 81.414ms | 80.000ms | 241.52 MiB | none | 12.42M/s |
| q3 | three aggregates | 57.000ms | 140.722ms | 140.686ms | 14.3% | 120.628ms | 140.697ms | 120.613ms | 140.749ms | 110.000ms | 257.09 MiB | none | 7.11M/s |
| q4 | average | 41.000ms | 100.579ms | 120.632ms | 0.1% | 120.621ms | 120.700ms | 100.609ms | 120.722ms | 110.000ms | 260.60 MiB | none | 8.29M/s |
| q5 | count distinct, high card | 81.000ms | 160.894ms | 160.889ms | 0.0% | 160.878ms | 160.899ms | 140.782ms | 160.921ms | 220.000ms | 361.27 MiB | none | 6.22M/s |
| q6 | count distinct, strings | 67.000ms | 140.800ms | 140.721ms | 0.3% | 140.719ms | 141.170ms | 140.717ms | 160.850ms | 160.000ms | 305.17 MiB | none | 7.11M/s |
| q7 | min and max of a date | 22.000ms | 100.503ms | 100.575ms | 0.0% | 100.569ms | 100.603ms | 100.561ms | 100.715ms | 100.000ms | 244.21 MiB | none | 9.94M/s |
| q8 | group by, low card | 51.000ms | 161.061ms | 120.707ms | 0.4% | 120.657ms | 121.119ms | 120.625ms | 160.785ms | 120.000ms | 254.05 MiB | none | 8.28M/s |
| q9 | group by and count distinct | 72.000ms | 140.704ms | 141.181ms | 14.2% | 140.723ms | 160.802ms | 140.710ms | 160.825ms | 200.000ms | 335.34 MiB | none | 7.08M/s |
| q10 | group by, several aggregates | 80.000ms | 162.242ms | 160.867ms | 0.1% | 160.847ms | 160.960ms | 160.769ms | 181.006ms | 230.000ms | 343.59 MiB | none | 6.22M/s |
| q11 | group by a string and count distinct | 84.000ms | 140.916ms | 160.797ms | 0.0% | 160.759ms | 160.815ms | 140.735ms | 180.840ms | 140.000ms | 281.01 MiB | none | 6.22M/s |
| q12 | group by two strings and count distinct | 61.000ms | 140.749ms | 140.732ms | 0.2% | 140.724ms | 140.976ms | 140.712ms | 180.875ms | 190.000ms | 286.44 MiB | none | 7.11M/s |
| q13 | group by a string and top k | 64.000ms | 140.733ms | 140.746ms | 0.0% | 140.705ms | 140.761ms | 120.582ms | 140.783ms | 180.000ms | 316.50 MiB | none | 7.10M/s |
| q14 | group by a string and count distinct | 73.000ms | 160.794ms | 140.803ms | 14.2% | 140.725ms | 160.783ms | 140.702ms | 161.038ms | 200.000ms | 345.41 MiB | none | 7.10M/s |
| q15 | group by two columns and top k | 81.000ms | 140.823ms | 160.735ms | 12.7% | 140.708ms | 161.105ms | 140.706ms | 180.858ms | 210.000ms | 333.41 MiB | none | 6.22M/s |
| q16 | group by, very high card | 48.000ms | 140.741ms | 120.807ms | 0.1% | 120.664ms | 120.828ms | 120.634ms | 121.265ms | 200.000ms | 335.18 MiB | none | 8.28M/s |
| q17 | group by two, very high card | 84.000ms | 160.792ms | 161.016ms | 12.4% | 160.824ms | 180.868ms | 140.789ms | 180.934ms | 330.000ms | 429.08 MiB | none | 6.21M/s |
| q18 | group by two, no ordering | 64.000ms | 160.766ms | 140.803ms | 14.2% | 140.789ms | 160.801ms | 140.712ms | 160.887ms | 170.000ms | 321.10 MiB | none | 7.10M/s |
| q19 | group by with an extract | 91.000ms | 160.912ms | 180.827ms | 11.1% | 160.901ms | 181.047ms | 160.870ms | 182.321ms | 390.000ms | 445.27 MiB | none | 5.53M/s |
| q20 | point lookup | 50.000ms | 160.767ms | 120.695ms | 0.1% | 120.680ms | 120.743ms | 120.648ms | 140.733ms | 120.000ms | 260.90 MiB | none | 8.29M/s |
| q21 | substring scan | 74.000ms | 160.798ms | 160.809ms | 0.0% | 160.801ms | 160.850ms | 140.739ms | 160.905ms | 200.000ms | 318.35 MiB | none | 6.22M/s |
| q22 | substring scan and group by | 91.000ms | 140.717ms | 160.777ms | 0.0% | 160.774ms | 160.781ms | 160.748ms | 160.791ms | 270.000ms | 336.15 MiB | none | 6.22M/s |
| q23 | two substring scans and group by | 82.000ms | 161.048ms | 161.146ms | 0.2% | 160.912ms | 161.214ms | 160.900ms | 161.542ms | 310.000ms | 366.70 MiB | none | 6.21M/s |
| q24 | select star and top k | 197.000ms | 281.559ms | 281.260ms | 7.2% | 261.237ms | 281.450ms | 241.209ms | 281.679ms | 490.000ms | 441.94 MiB | none | 3.56M/s |
| q25 | top k by a date | 79.000ms | 140.845ms | 160.801ms | 12.5% | 140.690ms | 160.857ms | 140.664ms | 161.257ms | 150.000ms | 291.16 MiB | none | 6.22M/s |
| q26 | top k by a string | 51.000ms | 160.822ms | 120.634ms | 16.7% | 120.634ms | 140.732ms | 100.563ms | 140.778ms | 130.000ms | 288.16 MiB | none | 8.29M/s |
| q27 | top k by two columns | 77.000ms | 200.918ms | 160.788ms | 0.1% | 160.782ms | 160.908ms | 160.780ms | 161.068ms | 150.000ms | 289.03 MiB | none | 6.22M/s |
| q28 | group by with a string length | 56.000ms | 140.739ms | 140.696ms | 14.2% | 120.864ms | 140.854ms | 120.628ms | 141.162ms | 130.000ms | 278.55 MiB | none | 7.11M/s |
| q29 | group by a regular expression | 71.000ms | 160.803ms | 160.792ms | 12.5% | 140.766ms | 160.813ms | 140.734ms | 160.868ms | 330.000ms | 409.03 MiB | none | 6.22M/s |
| q30 | ninety sums over one column | 31.000ms | 100.565ms | 100.596ms | 19.9% | 100.576ms | 120.638ms | 100.570ms | 140.712ms | 110.000ms | 257.01 MiB | none | 9.94M/s |
| q31 | group by two and several aggregates | 84.000ms | 180.917ms | 160.821ms | 0.0% | 160.791ms | 160.870ms | 140.717ms | 187.582ms | 190.000ms | 304.18 MiB | none | 6.22M/s |
| q32 | group by a high card pair | 75.000ms | 160.759ms | 160.771ms | 12.5% | 140.702ms | 160.781ms | 140.685ms | 161.061ms | 210.000ms | 320.23 MiB | none | 6.22M/s |
| q33 | group by a high card pair, unfiltered | 83.000ms | 163.153ms | 160.811ms | 0.0% | 160.807ms | 160.817ms | 141.228ms | 160.821ms | 290.000ms | 384.77 MiB | none | 6.22M/s |
| q34 | group by a long string | 82.000ms | 160.875ms | 160.858ms | 12.6% | 160.834ms | 181.054ms | 141.095ms | 181.272ms | 390.000ms | 469.76 MiB | none | 6.22M/s |
| q35 | group by a constant and a long string | 77.000ms | 160.795ms | 160.809ms | 0.0% | 160.802ms | 160.812ms | 141.004ms | 160.819ms | 360.000ms | 468.54 MiB | none | 6.22M/s |
| q36 | group by four expressions | 51.000ms | 141.145ms | 120.754ms | 0.1% | 120.656ms | 120.755ms | 120.655ms | 140.714ms | 190.000ms | 329.99 MiB | none | 8.28M/s |
| q37 | date range and group by a URL | 80.000ms | 160.764ms | 160.808ms | 0.1% | 160.782ms | 160.875ms | 140.694ms | 180.869ms | 130.000ms | 267.90 MiB | none | 6.22M/s |
| q38 | date range and group by a title | 95.000ms | 180.855ms | 180.846ms | 11.1% | 160.775ms | 180.874ms | 140.737ms | 181.233ms | 140.000ms | 264.86 MiB | none | 5.53M/s |
| q39 | date range, group by and offset | 90.000ms | 160.773ms | 160.783ms | 0.0% | 160.767ms | 160.786ms | 140.708ms | 180.843ms | 170.000ms | 268.91 MiB | none | 6.22M/s |
| q40 | date range, a case and a wide group by | 79.000ms | 160.792ms | 160.840ms | 12.7% | 140.719ms | 161.178ms | 140.668ms | 161.817ms | 190.000ms | 277.23 MiB | none | 6.22M/s |
| q41 | date range with an IN and a hash | 70.000ms | 141.063ms | 140.881ms | 0.3% | 140.752ms | 141.110ms | 120.730ms | 144.587ms | 130.000ms | 264.32 MiB | none | 7.10M/s |
| q42 | date range and a deep offset | 76.000ms | 160.817ms | 140.778ms | 14.3% | 140.744ms | 160.874ms | 140.730ms | 180.978ms | 170.000ms | 263.72 MiB | none | 7.10M/s |
| q43 | minute buckets over a date range | 73.000ms | 140.693ms | 140.680ms | 0.0% | 140.679ms | 140.689ms | 140.675ms | 141.008ms | 120.000ms | 260.65 MiB | none | 7.11M/s |

clickhouse-local 26.9.1.1562 over 43 of 43 queries. Total 3.005s by its own clock and 6.334s by ours, 6.499s cold, 8.490s of CPU, peak 469.76 MiB, 14.31M/s and 3.00 GiB/s.

Running it cost 111% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.49x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 40.331ms | 20.253ms | 0.1% | 20.251ms | 20.267ms | 20.249ms | 20.288ms | 10.000ms | 90.76 MiB | none | 49.37M/s |
| q2 | filtered count | 14.000ms | 40.733ms | 40.396ms | 0.1% | 40.360ms | 40.401ms | 40.332ms | 40.410ms | 150.000ms | 204.57 MiB | none | 24.75M/s |
| q3 | three aggregates | 18.000ms | 40.847ms | 40.408ms | 0.3% | 40.384ms | 40.498ms | 40.377ms | 40.804ms | 240.000ms | 407.89 MiB | none | 24.75M/s |
| q4 | average | 14.000ms | 40.450ms | 40.394ms | 0.7% | 40.376ms | 40.661ms | 40.359ms | 40.847ms | 180.000ms | 241.07 MiB | none | 24.76M/s |
| q5 | count distinct, high card | 26.000ms | 60.461ms | 60.523ms | 0.3% | 60.499ms | 60.701ms | 60.463ms | 61.248ms | 460.000ms | 568.09 MiB | none | 16.52M/s |
| q6 | count distinct, strings | 27.000ms | 60.493ms | 60.521ms | 0.4% | 60.438ms | 60.667ms | 60.422ms | 61.866ms | 400.000ms | 586.27 MiB | none | 16.52M/s |
| q7 | min and max of a date | 1.000ms | 20.279ms | 20.261ms | 0.1% | 20.258ms | 20.269ms | 20.251ms | 20.278ms | 10.000ms | 89.50 MiB | none | 49.35M/s |
| q8 | group by, low card | 15.000ms | 40.336ms | 40.403ms | 0.0% | 40.398ms | 40.413ms | 40.338ms | 40.418ms | 170.000ms | 216.58 MiB | none | 24.75M/s |
| q9 | group by and count distinct | 37.000ms | 60.550ms | 60.797ms | 0.5% | 60.597ms | 60.902ms | 60.525ms | 63.170ms | 710.000ms | 658.47 MiB | none | 16.45M/s |
| q10 | group by, several aggregates | 32.000ms | 60.578ms | 60.529ms | 0.0% | 60.507ms | 60.529ms | 60.439ms | 60.558ms | 530.000ms | 602.11 MiB | none | 16.52M/s |
| q11 | group by a string and count distinct | 27.000ms | 60.872ms | 60.516ms | 0.2% | 60.462ms | 60.592ms | 60.452ms | 62.487ms | 380.000ms | 414.39 MiB | none | 16.52M/s |
| q12 | group by two strings and count distinct | 28.000ms | 60.445ms | 60.638ms | 0.7% | 60.533ms | 60.985ms | 60.428ms | 61.045ms | 430.000ms | 429.90 MiB | none | 16.49M/s |
| q13 | group by a string and top k | 34.000ms | 60.488ms | 60.704ms | 0.9% | 60.532ms | 61.109ms | 60.439ms | 61.133ms | 640.000ms | 633.07 MiB | none | 16.47M/s |
| q14 | group by a string and count distinct | 39.000ms | 60.665ms | 60.915ms | 3.9% | 60.581ms | 62.962ms | 60.490ms | 81.873ms | 700.000ms | 698.38 MiB | none | 16.42M/s |
| q15 | group by two columns and top k | 36.000ms | 60.707ms | 60.897ms | 1.9% | 60.692ms | 61.849ms | 60.612ms | 63.322ms | 680.000ms | 640.32 MiB | none | 16.42M/s |
| q16 | group by, very high card | 29.000ms | 60.794ms | 60.633ms | 0.3% | 60.612ms | 60.804ms | 60.567ms | 61.276ms | 570.000ms | 612.84 MiB | none | 16.49M/s |
| q17 | group by two, very high card | 45.000ms | 80.824ms | 80.874ms | 0.9% | 80.842ms | 81.540ms | 80.802ms | 81.960ms | 890.000ms | 927.88 MiB | none | 12.36M/s |
| q18 | group by two, no ordering | 42.000ms | 80.701ms | 80.767ms | 0.2% | 80.688ms | 80.817ms | 80.575ms | 81.212ms | 800.000ms | 922.96 MiB | none | 12.38M/s |
| q19 | group by with an extract | 51.000ms | 82.460ms | 80.767ms | 0.2% | 80.714ms | 80.879ms | 80.642ms | 81.270ms | 970.000ms | 992.21 MiB | none | 12.38M/s |
| q20 | point lookup | 13.000ms | 40.425ms | 40.399ms | 0.1% | 40.364ms | 40.399ms | 40.355ms | 40.439ms | 140.000ms | 227.43 MiB | none | 24.75M/s |
| q21 | substring scan | 21.000ms | 40.353ms | 40.390ms | 0.1% | 40.385ms | 40.412ms | 40.362ms | 40.418ms | 370.000ms | 495.26 MiB | none | 24.76M/s |
| q22 | substring scan and group by | 33.000ms | 61.845ms | 60.664ms | 1.4% | 60.549ms | 61.386ms | 60.503ms | 62.300ms | 610.000ms | 499.40 MiB | none | 16.48M/s |
| q23 | two substring scans and group by | 44.000ms | 63.868ms | 80.674ms | 25.0% | 60.805ms | 80.978ms | 60.512ms | 84.318ms | 800.000ms | 551.13 MiB | none | 12.40M/s |
| q24 | select star and top k | 82.000ms | 121.130ms | 120.986ms | 13.9% | 104.236ms | 121.045ms | 103.504ms | 124.231ms | 1.770s | 942.17 MiB | none | 8.27M/s |
| q25 | top k by a date | 22.000ms | 40.352ms | 60.540ms | 32.9% | 40.646ms | 60.542ms | 40.412ms | 60.562ms | 280.000ms | 392.63 MiB | none | 16.52M/s |
| q26 | top k by a string | 25.000ms | 40.352ms | 60.505ms | 0.1% | 60.445ms | 60.506ms | 60.431ms | 60.509ms | 460.000ms | 443.27 MiB | none | 16.53M/s |
| q27 | top k by two columns | 26.000ms | 60.523ms | 60.623ms | 0.9% | 60.431ms | 60.974ms | 60.413ms | 60.998ms | 450.000ms | 440.18 MiB | none | 16.49M/s |
| q28 | group by with a string length | 31.000ms | 60.500ms | 60.637ms | 0.2% | 60.575ms | 60.685ms | 60.449ms | 61.046ms | 640.000ms | 672.93 MiB | none | 16.49M/s |
| q29 | group by a regular expression | 48.000ms | 80.573ms | 80.936ms | 1.4% | 80.670ms | 81.826ms | 80.623ms | 82.752ms | 810.000ms | 1017.02 MiB | none | 12.36M/s |
| q30 | ninety sums over one column | 23.000ms | 40.347ms | 60.416ms | 33.3% | 40.353ms | 60.445ms | 40.351ms | 60.450ms | 140.000ms | 233.52 MiB | none | 16.55M/s |
| q31 | group by two and several aggregates | 30.000ms | 60.549ms | 60.586ms | 0.1% | 60.540ms | 60.592ms | 60.516ms | 61.870ms | 510.000ms | 462.69 MiB | none | 16.51M/s |
| q32 | group by a high card pair | 30.000ms | 60.621ms | 60.627ms | 0.4% | 60.519ms | 60.744ms | 60.512ms | 61.499ms | 530.000ms | 479.44 MiB | none | 16.49M/s |
| q33 | group by a high card pair, unfiltered | 41.000ms | 80.748ms | 80.754ms | 0.2% | 80.696ms | 80.858ms | 80.665ms | 81.526ms | 780.000ms | 763.00 MiB | none | 12.38M/s |
| q34 | group by a long string | 56.000ms | 100.990ms | 80.930ms | 2.9% | 80.866ms | 83.222ms | 80.735ms | 103.888ms | 1.030s | 1.16 GiB | none | 12.36M/s |
| q35 | group by a constant and a long string | 59.000ms | 80.782ms | 82.218ms | 2.2% | 81.385ms | 83.202ms | 81.209ms | 100.832ms | 1.300s | 1.10 GiB | none | 12.16M/s |
| q36 | group by four expressions | 32.000ms | 61.312ms | 60.787ms | 1.8% | 60.693ms | 61.804ms | 60.569ms | 62.093ms | 650.000ms | 647.87 MiB | none | 16.45M/s |
| q37 | date range and group by a URL | 11.000ms | 40.369ms | 40.390ms | 0.2% | 40.358ms | 40.447ms | 40.356ms | 40.450ms | 70.000ms | 207.89 MiB | none | 24.76M/s |
| q38 | date range and group by a title | 10.000ms | 40.359ms | 40.395ms | 0.1% | 40.380ms | 40.411ms | 40.364ms | 40.472ms | 40.000ms | 167.04 MiB | none | 24.75M/s |
| q39 | date range, group by and offset | 9.000ms | 40.376ms | 40.363ms | 0.2% | 40.350ms | 40.436ms | 40.344ms | 40.488ms | 40.000ms | 157.40 MiB | none | 24.77M/s |
| q40 | date range, a case and a wide group by | 13.000ms | 40.362ms | 40.382ms | 0.1% | 40.359ms | 40.395ms | 40.353ms | 40.429ms | 50.000ms | 208.24 MiB | none | 24.76M/s |
| q41 | date range with an IN and a hash | 8.000ms | 40.421ms | 40.360ms | 0.0% | 40.357ms | 40.362ms | 40.340ms | 40.435ms | 30.000ms | 152.51 MiB | none | 24.78M/s |
| q42 | date range and a deep offset | 8.000ms | 40.373ms | 40.371ms | 0.1% | 40.369ms | 40.404ms | 40.342ms | 40.414ms | 30.000ms | 145.51 MiB | none | 24.77M/s |
| q43 | minute buckets over a date range | 8.000ms | 40.377ms | 40.358ms | 0.1% | 40.356ms | 40.413ms | 40.354ms | 40.444ms | 40.000ms | 147.65 MiB | none | 24.78M/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 1.199s by its own clock and 2.486s by ours, 2.451s cold, 20.490s of CPU, peak 1.16 GiB, 35.86M/s and 7.52 GiB/s.

Running it cost 107% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 5.97x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 9.799ms | 100.592ms | 100.653ms | 0.1% | 100.613ms | 100.712ms | 100.583ms | 120.709ms | 140.000ms | 69.45 MiB | none | 9.93M/s |
| q2 | filtered count | 11.486ms | 131.374ms | 100.693ms | 20.8% | 100.577ms | 121.511ms | 100.560ms | 130.061ms | 210.000ms | 116.12 MiB | none | 9.93M/s |
| q3 | three aggregates | 11.966ms | 120.662ms | 120.686ms | 0.1% | 120.622ms | 120.798ms | 100.604ms | 120.807ms | 130.000ms | 132.98 MiB | none | 8.29M/s |
| q4 | average | 12.310ms | 100.600ms | 100.575ms | 0.1% | 100.552ms | 100.654ms | 100.550ms | 120.683ms | 110.000ms | 131.25 MiB | none | 9.94M/s |
| q5 | count distinct, high card | 25.219ms | 120.675ms | 121.133ms | 0.4% | 121.131ms | 121.581ms | 120.774ms | 122.319ms | 180.000ms | 151.75 MiB | none | 8.26M/s |
| q6 | count distinct, strings | 25.953ms | 120.664ms | 120.906ms | 0.5% | 120.636ms | 121.295ms | 120.635ms | 121.780ms | 190.000ms | 165.64 MiB | none | 8.27M/s |
| q7 | min and max of a date | 11.735ms | 100.595ms | 120.663ms | 16.7% | 100.635ms | 120.775ms | 100.618ms | 121.059ms | 160.000ms | 119.53 MiB | none | 8.29M/s |
| q8 | group by, low card | 18.361ms | 120.735ms | 120.698ms | 0.0% | 120.687ms | 120.747ms | 120.601ms | 140.863ms | 150.000ms | 119.99 MiB | none | 8.28M/s |
| q9 | group by and count distinct | 41.384ms | 140.732ms | 140.994ms | 0.4% | 140.829ms | 141.386ms | 140.752ms | 141.678ms | 320.000ms | 223.87 MiB | none | 7.09M/s |
| q10 | group by, several aggregates | 47.871ms | 160.915ms | 160.946ms | 0.3% | 160.856ms | 161.323ms | 160.793ms | 161.340ms | 360.000ms | 233.58 MiB | none | 6.21M/s |
| q11 | group by a string and count distinct | 29.147ms | 120.690ms | 120.646ms | 0.1% | 120.634ms | 120.713ms | 120.634ms | 120.717ms | 180.000ms | 160.38 MiB | none | 8.29M/s |
| q12 | group by two strings and count distinct | 31.162ms | 120.781ms | 120.727ms | 0.1% | 120.691ms | 120.811ms | 120.643ms | 140.861ms | 200.000ms | 164.19 MiB | none | 8.28M/s |
| q13 | group by a string and top k | 27.923ms | 120.668ms | 120.700ms | 0.1% | 120.672ms | 120.837ms | 120.611ms | 140.792ms | 200.000ms | 158.50 MiB | none | 8.28M/s |
| q14 | group by a string and count distinct | 39.449ms | 140.803ms | 140.839ms | 0.1% | 140.759ms | 140.863ms | 140.721ms | 140.976ms | 280.000ms | 195.20 MiB | none | 7.10M/s |
| q15 | group by two columns and top k | 29.565ms | 140.779ms | 120.703ms | 0.2% | 120.674ms | 120.893ms | 120.657ms | 140.736ms | 210.000ms | 163.32 MiB | none | 8.28M/s |
| q16 | group by, very high card | 31.505ms | 140.717ms | 140.786ms | 0.1% | 140.750ms | 140.871ms | 140.721ms | 141.058ms | 230.000ms | 163.70 MiB | none | 7.10M/s |
| q17 | group by two, very high card | 46.529ms | 141.856ms | 161.114ms | 11.9% | 141.901ms | 161.128ms | 140.930ms | 161.500ms | 360.000ms | 264.07 MiB | none | 6.21M/s |
| q18 | group by two, no ordering | 36.110ms | 141.050ms | 141.196ms | 0.2% | 141.062ms | 141.353ms | 140.811ms | 142.567ms | 350.000ms | 261.46 MiB | none | 7.08M/s |
| q19 | group by with an extract | 50.748ms | 140.819ms | 161.324ms | 0.1% | 161.147ms | 161.347ms | 161.052ms | 161.552ms | 420.000ms | 280.48 MiB | none | 6.20M/s |
| q20 | point lookup | 10.919ms | 121.418ms | 100.624ms | 20.0% | 100.572ms | 120.711ms | 100.563ms | 125.115ms | 140.000ms | 124.69 MiB | none | 9.94M/s |
| q21 | substring scan | 34.873ms | 146.039ms | 141.872ms | 1.2% | 141.177ms | 142.905ms | 140.922ms | 144.380ms | 300.000ms | 223.02 MiB | none | 7.05M/s |
| q22 | substring scan and group by | 42.203ms | 140.700ms | 140.801ms | 0.2% | 140.796ms | 141.115ms | 140.787ms | 141.535ms | 350.000ms | 248.86 MiB | none | 7.10M/s |
| q23 | two substring scans and group by | 65.681ms | 180.980ms | 161.027ms | 0.0% | 161.002ms | 161.051ms | 160.985ms | 161.277ms | 660.000ms | 388.66 MiB | none | 6.21M/s |
| q24 | select star and top k | 109.688ms | 201.169ms | 203.080ms | 9.8% | 201.140ms | 221.071ms | 201.118ms | 221.085ms | 1.060s | 413.68 MiB | none | 4.92M/s |
| q25 | top k by a date | 20.883ms | 120.760ms | 120.750ms | 0.6% | 120.717ms | 121.402ms | 120.700ms | 122.207ms | 150.000ms | 165.02 MiB | none | 8.28M/s |
| q26 | top k by a string | 18.225ms | 120.785ms | 121.204ms | 0.4% | 120.862ms | 121.326ms | 120.734ms | 122.247ms | 140.000ms | 141.60 MiB | none | 8.25M/s |
| q27 | top k by two columns | 23.343ms | 140.765ms | 120.659ms | 0.2% | 120.646ms | 120.872ms | 120.640ms | 120.963ms | 170.000ms | 170.26 MiB | none | 8.29M/s |
| q30 | ninety sums over one column | 20.557ms | 120.743ms | 120.914ms | 0.2% | 120.715ms | 121.001ms | 120.646ms | 121.140ms | 150.000ms | 125.05 MiB | none | 8.27M/s |
| q31 | group by two and several aggregates | 33.746ms | 140.746ms | 140.752ms | 0.1% | 140.736ms | 140.860ms | 140.704ms | 141.034ms | 210.000ms | 182.44 MiB | none | 7.10M/s |
| q32 | group by a high card pair | 28.603ms | 141.394ms | 120.829ms | 0.8% | 120.724ms | 121.695ms | 120.647ms | 140.727ms | 200.000ms | 200.67 MiB | none | 8.28M/s |
| q33 | group by a high card pair, unfiltered | 43.317ms | 141.306ms | 141.221ms | 0.5% | 140.795ms | 141.438ms | 140.741ms | 141.828ms | 370.000ms | 289.26 MiB | none | 7.08M/s |
| q34 | group by a long string | 55.239ms | 160.800ms | 160.927ms | 0.0% | 160.904ms | 160.973ms | 160.838ms | 161.555ms | 540.000ms | 410.04 MiB | none | 6.21M/s |
| q35 | group by a constant and a long string | 60.107ms | 161.256ms | 162.495ms | 1.2% | 160.880ms | 162.761ms | 160.846ms | 181.092ms | 630.000ms | 433.07 MiB | none | 6.15M/s |
| q37 | date range and group by a URL | 21.554ms | 120.845ms | 120.669ms | 0.1% | 120.651ms | 120.792ms | 120.650ms | 120.800ms | 160.000ms | 92.03 MiB | none | 8.29M/s |
| q38 | date range and group by a title | 21.587ms | 120.651ms | 120.637ms | 0.0% | 120.633ms | 120.665ms | 120.620ms | 120.688ms | 150.000ms | 90.20 MiB | none | 8.29M/s |
| q39 | date range, group by and offset | 17.762ms | 120.669ms | 120.666ms | 0.1% | 120.664ms | 120.730ms | 120.645ms | 120.972ms | 160.000ms | 86.78 MiB | none | 8.29M/s |
| q40 | date range, a case and a wide group by | 20.001ms | 120.688ms | 120.857ms | 0.7% | 120.727ms | 121.528ms | 120.713ms | 122.309ms | 150.000ms | 98.01 MiB | none | 8.27M/s |
| q41 | date range with an IN and a hash | 17.949ms | 120.692ms | 120.873ms | 1.0% | 120.755ms | 121.915ms | 120.659ms | 133.907ms | 140.000ms | 87.60 MiB | none | 8.27M/s |
| q42 | date range and a deep offset | 17.265ms | 120.766ms | 120.755ms | 0.1% | 120.726ms | 120.890ms | 120.637ms | 140.885ms | 140.000ms | 85.46 MiB | none | 8.28M/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 1.222s by its own clock and 5.118s by ours, 5.189s cold, 10.350s of CPU, peak 433.07 MiB, 31.92M/s and 6.70 GiB/s.

Running it cost 319% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.02x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 11.943ms | 20.293ms | 20.268ms | 0.0% | 20.265ms | 20.272ms | 20.262ms | 20.278ms | 10.000ms | 16.25 MiB | none | 49.34M/s |
| q2 | filtered count | 12.277ms | 20.274ms | 20.294ms | 0.4% | 20.288ms | 20.361ms | 20.273ms | 20.365ms | 20.000ms | 16.65 MiB | none | 49.27M/s |
| q3 | three aggregates | 12.468ms | 20.301ms | 20.273ms | 0.0% | 20.271ms | 20.275ms | 20.268ms | 20.282ms | 20.000ms | 16.27 MiB | none | 49.32M/s |
| q4 | average | 12.204ms | 20.288ms | 20.275ms | 0.0% | 20.274ms | 20.277ms | 20.265ms | 20.279ms | 10.000ms | 19.04 MiB | none | 49.32M/s |
| q5 | count distinct, high card | 14.321ms | 40.365ms | 40.407ms | 0.1% | 40.388ms | 40.416ms | 40.387ms | 40.533ms | 30.000ms | 39.64 MiB | none | 24.75M/s |
| q6 | count distinct, strings | 23.169ms | 40.518ms | 40.414ms | 0.1% | 40.395ms | 40.430ms | 40.383ms | 40.444ms | 100.000ms | 51.02 MiB | none | 24.74M/s |
| q7 | min and max of a date | 12.193ms | 20.298ms | 20.274ms | 0.1% | 20.268ms | 20.286ms | 20.265ms | 20.349ms | 20.000ms | 16.44 MiB | none | 49.32M/s |
| q8 | group by, low card | 12.348ms | 20.285ms | 20.284ms | 0.1% | 20.282ms | 20.307ms | 20.280ms | 20.363ms | 10.000ms | 16.39 MiB | none | 49.30M/s |
| q9 | group by and count distinct | 18.084ms | 40.481ms | 40.445ms | 0.2% | 40.395ms | 40.470ms | 40.387ms | 40.519ms | 60.000ms | 71.29 MiB | none | 24.72M/s |
| q10 | group by, several aggregates | 19.932ms | 40.483ms | 41.286ms | 3.1% | 40.761ms | 42.042ms | 40.405ms | 42.960ms | 90.000ms | 79.32 MiB | none | 24.22M/s |
| q11 | group by a string and count distinct | 15.560ms | 40.388ms | 40.472ms | 0.1% | 40.417ms | 40.475ms | 40.399ms | 43.747ms | 30.000ms | 21.99 MiB | none | 24.71M/s |
| q12 | group by two strings and count distinct | 16.044ms | 40.475ms | 40.396ms | 0.0% | 40.394ms | 40.398ms | 40.379ms | 40.403ms | 40.000ms | 22.70 MiB | none | 24.75M/s |
| q13 | group by a string and top k | 20.822ms | 40.426ms | 40.414ms | 0.0% | 40.406ms | 40.417ms | 40.360ms | 41.085ms | 80.000ms | 45.64 MiB | none | 24.74M/s |
| q14 | group by a string and count distinct | 25.581ms | 40.497ms | 40.413ms | 0.0% | 40.409ms | 40.414ms | 40.389ms | 40.415ms | 110.000ms | 63.30 MiB | none | 24.74M/s |
| q15 | group by two columns and top k | 23.632ms | 40.714ms | 40.412ms | 0.0% | 40.412ms | 40.413ms | 40.410ms | 40.527ms | 110.000ms | 53.82 MiB | none | 24.74M/s |
| q16 | group by, very high card | 29.060ms | 40.390ms | 42.718ms | 4.9% | 41.377ms | 43.456ms | 40.380ms | 45.046ms | 150.000ms | 124.79 MiB | none | 23.41M/s |
| q17 | group by two, very high card | 42.099ms | 60.487ms | 60.498ms | 0.0% | 60.498ms | 60.511ms | 60.480ms | 60.595ms | 220.000ms | 96.98 MiB | none | 16.53M/s |
| q18 | group by two, no ordering | 16.017ms | 41.143ms | 40.625ms | 1.1% | 40.450ms | 40.895ms | 40.376ms | 41.216ms | 60.000ms | 24.48 MiB | none | 24.61M/s |
| q19 | group by with an extract | 47.204ms | 60.550ms | 60.500ms | 0.0% | 60.477ms | 60.501ms | 60.460ms | 60.679ms | 270.000ms | 110.93 MiB | none | 16.53M/s |
| q20 | point lookup | 11.796ms | 20.298ms | 20.277ms | 0.0% | 20.273ms | 20.282ms | 20.270ms | 20.291ms | 20.000ms | 18.41 MiB | none | 49.32M/s |
| q21 | substring scan | 22.645ms | 40.990ms | 40.576ms | 1.0% | 40.435ms | 40.859ms | 40.416ms | 41.152ms | 150.000ms | 71.66 MiB | none | 24.64M/s |
| q22 | substring scan and group by | 24.413ms | 40.415ms | 40.421ms | 0.0% | 40.418ms | 40.428ms | 40.417ms | 40.496ms | 190.000ms | 68.45 MiB | none | 24.74M/s |
| q23 | two substring scans and group by | 38.119ms | 60.638ms | 60.718ms | 1.3% | 60.637ms | 61.398ms | 60.529ms | 67.367ms | 380.000ms | 88.87 MiB | none | 16.47M/s |
| q24 | select star and top k | 32.512ms | 40.520ms | 40.463ms | 0.1% | 40.435ms | 40.467ms | 40.429ms | 40.550ms | 210.000ms | 92.73 MiB | none | 24.71M/s |
| q25 | top k by a date | 14.894ms | 40.402ms | 40.630ms | 0.7% | 40.441ms | 40.741ms | 40.423ms | 40.768ms | 60.000ms | 25.08 MiB | none | 24.61M/s |
| q26 | top k by a string | 14.641ms | 40.561ms | 40.408ms | 4.9% | 40.389ms | 42.363ms | 40.387ms | 44.618ms | 50.000ms | 21.80 MiB | none | 24.75M/s |
| q27 | top k by two columns | 15.849ms | 40.390ms | 41.184ms | 1.7% | 40.843ms | 41.527ms | 40.690ms | 46.369ms | 70.000ms | 24.64 MiB | none | 24.28M/s |
| q28 | group by with a string length | 24.979ms | 40.426ms | 40.509ms | 0.3% | 40.447ms | 40.560ms | 40.435ms | 44.212ms | 180.000ms | 73.13 MiB | none | 24.69M/s |
| q29 | group by a regular expression | 47.907ms | 60.493ms | 62.425ms | 6.8% | 60.512ms | 64.740ms | 60.502ms | 69.192ms | 370.000ms | 109.85 MiB | none | 16.02M/s |
| q30 | ninety sums over one column | 13.877ms | 40.387ms | 40.384ms | 0.0% | 40.384ms | 40.392ms | 40.379ms | 40.437ms | 20.000ms | 16.28 MiB | none | 24.76M/s |
| q31 | group by two and several aggregates | 18.351ms | 43.388ms | 40.411ms | 3.8% | 40.407ms | 41.930ms | 40.397ms | 45.131ms | 70.000ms | 35.80 MiB | none | 24.75M/s |
| q32 | group by a high card pair | 18.291ms | 41.131ms | 41.572ms | 2.8% | 41.060ms | 42.236ms | 40.392ms | 48.478ms | 70.000ms | 36.74 MiB | none | 24.05M/s |
| q33 | group by a high card pair, unfiltered | 26.842ms | 60.457ms | 40.396ms | 0.1% | 40.390ms | 40.412ms | 40.341ms | 42.273ms | 120.000ms | 87.69 MiB | none | 24.75M/s |
| q34 | group by a long string | 48.663ms | 60.512ms | 60.477ms | 0.0% | 60.473ms | 60.500ms | 60.462ms | 60.513ms | 340.000ms | 161.86 MiB | none | 16.53M/s |
| q35 | group by a constant and a long string | 48.916ms | 60.478ms | 60.486ms | 0.0% | 60.472ms | 60.500ms | 60.456ms | 60.511ms | 340.000ms | 162.24 MiB | none | 16.53M/s |
| q36 | group by four expressions | 26.402ms | 42.690ms | 42.810ms | 6.5% | 40.500ms | 43.282ms | 40.399ms | 45.401ms | 130.000ms | 110.29 MiB | none | 23.36M/s |
| q37 | date range and group by a URL | 13.069ms | 20.330ms | 20.313ms | 98.8% | 20.290ms | 40.370ms | 20.284ms | 40.376ms | 20.000ms | 21.29 MiB | none | 49.23M/s |
| q38 | date range and group by a title | 13.770ms | 40.395ms | 40.401ms | 0.0% | 40.399ms | 40.415ms | 40.397ms | 40.441ms | 20.000ms | 20.31 MiB | none | 24.75M/s |
| q39 | date range, group by and offset | 12.910ms | 40.385ms | 20.287ms | 99.2% | 20.284ms | 40.403ms | 20.277ms | 40.415ms | 10.000ms | 20.25 MiB | none | 49.29M/s |
| q40 | date range, a case and a wide group by | 16.815ms | 40.389ms | 40.409ms | 0.0% | 40.408ms | 40.410ms | 40.393ms | 40.523ms | 20.000ms | 28.18 MiB | none | 24.75M/s |
| q41 | date range with an IN and a hash | 12.153ms | 20.339ms | 20.291ms | 0.1% | 20.291ms | 20.311ms | 20.290ms | 20.387ms | 10.000ms | 17.76 MiB | none | 49.28M/s |
| q42 | date range and a deep offset | 12.047ms | 20.298ms | 20.303ms | 0.1% | 20.301ms | 20.319ms | 20.277ms | 20.329ms | 10.000ms | 17.80 MiB | none | 49.25M/s |
| q43 | minute buckets over a date range | 12.618ms | 20.291ms | 20.305ms | 0.1% | 20.299ms | 20.312ms | 20.273ms | 20.337ms | 10.000ms | 17.64 MiB | none | 49.25M/s |

rudb rudb 0.3.45 over 43 of 43 queries. Total 927.438ms by its own clock and 1.627s by ours, 1.665s cold, 4.310s of CPU, peak 162.24 MiB, 46.36M/s and 9.73 GiB/s.

Running it cost 75% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.08x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | 6.718ms | 1.150ms | 2.921ms | 2.926ms | 0.2% | 1.528ms | 3.099ms | 13.974ms | 896 B | 4 of 4 |
| q2 | 6.669ms | 1.921ms | 6.268ms | 6.273ms | 0.1% | 2.515ms | 3.076ms | 10.650ms | 896 B | 5 of 5 |
| q3 | 6.767ms | 2.251ms | 6.897ms | 6.903ms | 0.1% | 1.265ms | 3.119ms | 0.000us | 1.22 KiB | 4 of 4 |
| q4 | 6.642ms | 1.571ms | 7.320ms | 7.324ms | 0.1% | 1.751ms | 3.071ms | 9.605ms | 896 B | 4 of 4 |
| q5 | 6.654ms | 3.277ms | 12.053ms | 12.059ms | 0.0% | 1.376ms | 3.007ms | 4.934ms | 21.66 MiB | 4 of 4 |
| q6 | 6.738ms | 12.042ms | 82.258ms | 82.264ms | 0.0% | 2.321ms | 3.002ms | 14.734ms | 16.03 MiB | 5 of 5 |
| q7 | 6.802ms | 1.361ms | 7.563ms | 7.567ms | 0.1% | 1.568ms | 3.082ms | 0.000us | 1.00 KiB | 4 of 4 |
| q8 | 6.704ms | 1.491ms | 6.205ms | 6.209ms | 0.1% | 1.514ms | 3.016ms | 10.775ms | 8.32 KiB | 6 of 6 |
| q9 | 6.705ms | 9.309ms | 22.629ms | 22.634ms | 0.0% | 1.350ms | 3.088ms | 14.278ms | 47.22 MiB | 5 of 5 |
| q10 | 6.700ms | 9.997ms | 62.875ms | 62.881ms | 0.0% | 1.967ms | 3.014ms | 24.104ms | 42.67 MiB | 5 of 5 |
| q11 | 6.702ms | 4.113ms | 23.854ms | 23.859ms | 0.0% | 2.464ms | 3.034ms | 3.107ms | 984.44 KiB | 6 of 6 |
| q12 | 6.671ms | 4.649ms | 27.773ms | 27.778ms | 0.0% | 1.882ms | 3.000ms | 9.222ms | 1.05 MiB | 6 of 6 |
| q13 | 6.650ms | 9.923ms | 70.153ms | 70.157ms | 0.0% | 3.435ms | 2.960ms | 6.883ms | 15.09 MiB | 6 of 6 |
| q14 | 6.784ms | 14.306ms | 99.197ms | 99.202ms | 0.0% | 2.257ms | 2.966ms | 7.833ms | 28.76 MiB | 6 of 6 |
| q15 | 6.767ms | 14.638ms | 93.979ms | 93.985ms | 0.0% | 3.133ms | 3.069ms | 12.946ms | 19.96 MiB | 6 of 6 |
| q16 | 6.742ms | 17.113ms | 90.329ms | 90.336ms | 0.0% | 1.946ms | 3.053ms | 56.611ms | 52.98 MiB | 5 of 5 |
| q17 | 6.939ms | 29.846ms | 201.658ms | 201.664ms | 0.0% | 2.426ms | 3.077ms | 5.259ms | 54.29 MiB | 5 of 5 |
| q18 | 6.720ms | 5.336ms | 48.285ms | 48.290ms | 0.0% | 1.970ms | 3.009ms | 8.700ms | 14.03 KiB | 5 of 5 |
| q19 | 6.795ms | 39.329ms | 227.898ms | 227.904ms | 0.0% | 2.349ms | 3.111ms | 18.985ms | 63.18 MiB | 5 of 5 |
| q20 | 6.676ms | 1.202ms | 7.116ms | 7.119ms | 0.0% | 1.731ms | 2.977ms | 9.904ms | 0 B | 4 of 4 |
| q21 | 7.113ms | 15.869ms | 148.584ms | 148.589ms | 0.0% | 3.863ms | 3.047ms | 18.364ms | 896 B | 5 of 5 |
| q22 | 6.801ms | 14.059ms | 179.330ms | 179.335ms | 0.0% | 2.453ms | 3.071ms | 7.593ms | 12.07 KiB | 6 of 6 |
| q23 | 6.690ms | 26.744ms | 374.777ms | 374.782ms | 0.0% | 2.592ms | 3.010ms | 2.208ms | 46.82 KiB | 6 of 6 |
| q24 | 7.058ms | 21.300ms | 166.995ms | 167.019ms | 0.0% | 3.251ms | 3.145ms | 39.835ms | 42.23 KiB | 8 of 8 |
| q25 | 6.722ms | 3.975ms | 48.170ms | 48.188ms | 0.0% | 2.337ms | 3.063ms | 8.749ms | 52.26 KiB | 6 of 6 |
| q26 | 6.872ms | 5.131ms | 30.947ms | 30.951ms | 0.0% | 1.729ms | 3.073ms | 5.976ms | 47.94 KiB | 5 of 5 |
| q27 | 6.732ms | 5.903ms | 41.930ms | 41.933ms | 0.0% | 2.940ms | 2.954ms | 5.113ms | 71.10 KiB | 6 of 6 |
| q28 | 6.618ms | 14.671ms | 170.858ms | 170.863ms | 0.0% | 2.906ms | 3.054ms | 6.082ms | 748.52 KiB | 7 of 7 |
| q29 | 6.876ms | 33.646ms | 302.492ms | 302.496ms | 0.0% | 2.735ms | 3.129ms | 54.375ms | 31.92 MiB | 7 of 7 |
| q30 | 7.891ms | 1.409ms | 8.350ms | 8.358ms | 0.1% | 1.999ms | 3.016ms | 8.626ms | 29.59 KiB | 4 of 4 |
| q31 | 6.790ms | 8.154ms | 47.151ms | 47.157ms | 0.0% | 1.892ms | 3.110ms | 9.733ms | 6.37 MiB | 6 of 6 |
| q32 | 6.752ms | 6.498ms | 57.038ms | 57.043ms | 0.0% | 2.208ms | 3.027ms | 9.930ms | 6.44 MiB | 6 of 6 |
| q33 | 6.961ms | 22.014ms | 68.052ms | 68.058ms | 0.0% | 1.868ms | 3.236ms | 28.705ms | 57.46 MiB | 5 of 5 |
| q34 | 7.706ms | 37.932ms | 332.226ms | 332.231ms | 0.0% | 3.190ms | 3.041ms | 14.728ms | 113.51 MiB | 5 of 5 |
| q35 | 6.669ms | 37.853ms | 331.833ms | 331.839ms | 0.0% | 2.733ms | 3.003ms | 5.158ms | 115.95 MiB | 5 of 5 |
| q36 | 6.884ms | 16.644ms | 93.121ms | 93.128ms | 0.0% | 2.222ms | 3.116ms | 53.756ms | 51.79 MiB | 6 of 6 |
| q37 | 6.695ms | 2.188ms | 4.799ms | 4.805ms | 0.1% | 502.283us | 2.975ms | 12.220ms | 971.75 KiB | 6 of 6 |
| q38 | 6.801ms | 2.992ms | 6.472ms | 6.477ms | 0.1% | 1.250ms | 2.941ms | 10.582ms | 261.56 KiB | 6 of 6 |
| q39 | 6.747ms | 2.503ms | 4.998ms | 5.001ms | 0.1% | 1.526ms | 3.082ms | 11.917ms | 104.82 KiB | 6 of 6 |
| q40 | 6.767ms | 6.322ms | 11.172ms | 11.179ms | 0.1% | 519.492us | 2.955ms | 5.866ms | 3.35 MiB | 6 of 6 |
| q41 | 6.975ms | 1.800ms | 2.907ms | 2.912ms | 0.1% | 1.312ms | 3.218ms | 13.870ms | 90.81 KiB | 6 of 6 |
| q42 | 6.740ms | 1.533ms | 2.317ms | 2.321ms | 0.2% | 987.877us | 2.975ms | 4.704ms | 234.22 KiB | 6 of 6 |
| q43 | 6.956ms | 1.488ms | 2.420ms | 2.424ms | 0.2% | 458.079us | 3.193ms | 4.384ms | 402.55 KiB | 6 of 6 |

Read from the breakdown the engine wrote for its cold run. `planning` is everything before the first row moved, which is the parse, the bind, the optimizer passes and building the tree. It is a column rather than a footnote because it is the one cost of a query nobody profiles: an optimizer only ever has passes added to it, each paying for itself on the query it was written for, and a query that plans for four hundred milliseconds to save two hundred is a query the optimizer made slower. What is gated is planning as a share of the whole statement, and that share is not in this table, because it is taken over every run rather than off the cold one the rest of these columns come from. It lives in `baselines/planning-<suite>.txt`. `accounted` is what its pipelines charged themselves and `measured` is what it measured around running them, so `apart` is the cross check and anything over five percent is time the breakdown cannot explain. `driver` is the part of `accounted` that was not inside an operator, which is the loop that runs a pipeline rather than the operators it calls. `build` is what the engine spent putting the tree together, which is before there is a pipeline to charge and is why it is off the right hand side of the check. `outside` is the CPU the process spent on everything else, which is starting, opening the data and printing the answer, and it is the reason the wall clock column and the query time column differ. `held` is what the operators reserved, which is not the resident set of the process. `reference` is how many operators ran the reference implementation of their seam, which is the slow path kept for differential testing.

#### What the planner knew

| query | decisions | exact | certified | estimated | unknown |
| --- | --- | --- | --- | --- | --- |
| q1 | 4 | 4 | 0 | 0 | 0 |
| q2 | 5 | 3 | 0 | 2 | 0 |
| q3 | 4 | 4 | 0 | 0 | 0 |
| q4 | 4 | 4 | 0 | 0 | 0 |
| q5 | 4 | 4 | 0 | 0 | 0 |
| q6 | 5 | 4 | 0 | 1 | 0 |
| q7 | 4 | 4 | 0 | 0 | 0 |
| q8 | 6 | 1 | 0 | 5 | 0 |
| q9 | 5 | 2 | 0 | 3 | 0 |
| q10 | 5 | 2 | 0 | 3 | 0 |
| q11 | 6 | 1 | 0 | 5 | 0 |
| q12 | 6 | 1 | 0 | 5 | 0 |
| q13 | 6 | 1 | 0 | 5 | 0 |
| q14 | 6 | 1 | 0 | 5 | 0 |
| q15 | 6 | 1 | 0 | 5 | 0 |
| q16 | 5 | 2 | 0 | 3 | 0 |
| q17 | 5 | 2 | 0 | 3 | 0 |
| q18 | 5 | 2 | 0 | 3 | 0 |
| q19 | 5 | 2 | 0 | 3 | 0 |
| q20 | 4 | 1 | 0 | 3 | 0 |
| q21 | 5 | 3 | 0 | 2 | 0 |
| q22 | 6 | 1 | 0 | 5 | 0 |
| q23 | 6 | 1 | 0 | 5 | 0 |
| q24 | 8 | 1 | 0 | 7 | 0 |
| q25 | 6 | 1 | 0 | 5 | 0 |
| q26 | 5 | 1 | 0 | 4 | 0 |
| q27 | 6 | 1 | 0 | 5 | 0 |
| q28 | 7 | 1 | 0 | 6 | 0 |
| q29 | 7 | 1 | 0 | 6 | 0 |
| q30 | 4 | 4 | 0 | 0 | 0 |
| q31 | 6 | 1 | 0 | 5 | 0 |
| q32 | 6 | 1 | 0 | 5 | 0 |
| q33 | 5 | 2 | 0 | 3 | 0 |
| q34 | 5 | 2 | 0 | 3 | 0 |
| q35 | 5 | 2 | 0 | 3 | 0 |
| q36 | 6 | 2 | 0 | 4 | 0 |
| q37 | 6 | 1 | 0 | 5 | 0 |
| q38 | 6 | 1 | 0 | 5 | 0 |
| q39 | 6 | 1 | 0 | 5 | 0 |
| q40 | 6 | 1 | 0 | 5 | 0 |
| q41 | 6 | 1 | 0 | 5 | 0 |
| q42 | 6 | 1 | 0 | 5 | 0 |
| q43 | 6 | 1 | 0 | 5 | 0 |
| whole suite | 235 | 78 | 0 | 157 | 0 |

One row per query and one decision per operator, counted by what the planner had behind the cardinality it used. Over the whole suite that is 33% exact, 0% certified, 67% estimated and 0% unknown. `exact` is a counted number, `certified` is a number with a proven bound, `estimated` is a guess, and `unknown` is no number at all, which is an answer an operator has to handle rather than a null to paper over. Counts rather than fractions in the table because a suite's histogram is the sum of its queries' and fractions do not add.

#### Where the time went

| kind | cpu | share | operators | rows in | rows out | per row in | per row out | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| FileScan | 1.686s | 48.8% | 43 | 0 | 36070379 | handed none | 46.8ns | 43 of 43 |
| Aggregate | 1.522s | 44.0% | 39 | 19727227 | 113503 | 77.1ns | 13405.8ns | 39 of 39 |
| Filter | 187.096ms | 5.4% | 28 | 19070804 | 3014046 | 9.8ns | 62.1ns | 28 of 28 |
| Project | 47.496ms | 1.4% | 91 | 20413718 | 20413718 | 2.3ns | 2.3ns | 91 of 91 |
| TopN | 7.709ms | 0.2% | 31 | 399865 | 270 | 19.3ns | 28552.1ns | 31 of 31 |
| Fetch | 5.564ms | 0.2% | 1 | 10 | 10 | 556441.6ns | 556441.6ns | 1 of 1 |
| Sort | 8.406us | 0.0% | 1 | 13 | 13 | 646.6ns | 646.6ns | 1 of 1 |
| Limit | 0.584us | 0.0% | 1 | 10 | 10 | 58.4ns | 58.4ns | 1 of 1 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is FileScan, and the queries where it cost the most are q23 at 343.496ms, q22 at 150.251ms, q24 at 139.888ms.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 999975 rows, one out of every 100 of the 99997497 in the full file, which is a development loop rather than the suite
- q19 swung by 49.4% of its median, and rule two wants under 10%

These swung wider than reporting rule two allows:

- duckdb swung by 49.4% of its median on q19, and rule two wants under 10%
- clickhouse-local swung by 19.9% of its median on q30, and rule two wants under 10%
- datafusion swung by 33.3% of its median on q30, and rule two wants under 10%
- polars swung by 20.8% of its median on q2, and rule two wants under 10%
- rudb swung by 99.2% of its median on q39, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

polars did not run q28, because the SQLContext has no strlen and no character length function.

polars did not run q29, because the SQLContext has no regexp_replace.

polars did not run q36, because the SQLContext rejects the repeated ClientIP output name in the group by.

polars did not run q43, because the SQLContext has no date_trunc.

So the polars column is 39 of 43 queries and its ratio is over the shared ones.

All 6 engines agreed on every answer the data settles, which is 31 of 43 queries, to the last significant digit of a double.

These were answered differently and the data does not say which is right:

- q16: ORDER BY COUNT(*) DESC LIMIT 10 over user IDs, where the tenth place is a tie between many users with the same count and the engine picks.
- q17: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q18: LIMIT 10 with no ORDER BY at all, so the ten rows are whichever ten the group by handed back first and the query did not ask for any particular ten.
- q19: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q23: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q22, and the same on a one million row sample where seven of the ten places have a count of one.
- q24: ORDER BY EventTime LIMIT 10, and EventTime has one second resolution over a corpus with millions of rows a second, so the tenth row is a tie.
- q32: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q33: ORDER BY COUNT(*) DESC LIMIT 10 over WatchID and ClientIP unfiltered, where almost every group has a count of one and the ten that come back are arbitrary.
- q40: ORDER BY PageViews DESC LIMIT 10 OFFSET 1000 with ties at the cut, as q39.
- q41: ORDER BY PageViews DESC LIMIT 10 OFFSET 100 with ties at the cut, as q39.
- q43: DATE_TRUNC on a timestamp, and ClickHouse renders a DateTime in the machine's timezone where DuckDB renders a TIMESTAMP in none, so the same epoch second prints seven hours apart on gamingpc-wsl and two hours apart on server3.

So they are not checked, and a wrong answer from any engine on one of them would go unnoticed here. Every other query in the suite is checked in full.

These were answered differently and which engine is wrong is settled:

- q4: AVG(UserID) over a hundred million bigints, where the pinned DuckDB sums the column short by a multiple of 2^64 and rudb does not. Its own hugeint and decimal paths, the same column summed out of a table, its own per counter group sums, and adding the column up outside a database all give rudb's answer, and it reduces to two statements over a 1.3 MB one column parquet file. Written up as 14.10.1 in tamnd/rudb `spec/14-testing.md`, which is the list of places where DuckDB is wrong and we are not. tamnd/rudb-compat#12.

So they are a known difference rather than an open one, and the write up is where to go to disagree with that.

Hot here means page cache warm and not buffer pool warm, because every run is a fresh process so that no query's number depends on the one before it.

