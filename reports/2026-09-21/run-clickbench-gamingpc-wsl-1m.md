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
| timeout | 900s per query, and no query reached it |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --rows 1000000 --engines duckdb,duckdb-pinned,clickhouse-local,datafusion,polars,rudb --runs 5 --timeout 900 --report
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
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 3.300s | 8.980s | 310.01 MiB | its own database file | its own | 1.94 to 2.51 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 3.435s | 6.450s | 268.76 MiB | its own database file | its own | 2.51 to 2.32 |
| clickhouse-local | 26.9.1.1562 | ran | 751.023ms | 2.330s | 230.52 MiB | its own MergeTree parts, as system.parts counts the active ones | its own | 2.32 to 5.88 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 214.81 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 5.88 to 8.25 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 214.81 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 8.25 to 5.26 |
| rudb | rudb 0.3.67 | ran | 0.000us | 0.000us | 214.81 MiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 5.26 to 4.92 |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb on 39 shared |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 530.000ms | 1.355s | +156% | 1.401s | 3.180s | 2.35 | 303.39 MiB | none | 81.13M/s | 17.02 GiB/s | 1.00x |
| duckdb-pinned | 578.000ms | 1.982s | +243% | 2.084s | 3.450s | 1.74 | 305.39 MiB | none | 74.39M/s | 15.61 GiB/s | 1.09x |
| clickhouse-local | 2.817s | 5.928s | +110% | 5.938s | 8.200s | 1.38 | 468.82 MiB | none | 15.26M/s | 3.20 GiB/s | 6.21x |
| datafusion | 1.156s | 2.356s | +104% | 2.368s | 20.280s | 8.61 | 1.13 GiB | none | 37.20M/s | 7.80 GiB/s | 2.50x |
| polars | 1.274s | 4.984s | +291% | 5.115s | 10.000s | 2.01 | 437.08 MiB | none | 30.62M/s | 6.42 GiB/s | 3.06x |
| rudb | 679.170ms | 1.425s | +110% | 1.550s | 4.490s | 3.15 | 344.17 MiB | none | 63.31M/s | 13.28 GiB/s | 1.43x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 1.000ms | 5.000ms | 1.000ms | 9.215ms | 7.604ms |
| q2 | filtered count | 2.000ms | 2.000ms | 5.000ms | 12.000ms | 11.154ms | 8.696ms |
| q3 | three aggregates | 2.000ms | 2.000ms | 67.000ms | 16.000ms | 12.500ms | 8.841ms |
| q4 | average | 3.000ms | 4.000ms | 35.000ms | 13.000ms | 11.014ms | 9.016ms |
| q5 | count distinct, high card | 11.000ms | 11.000ms | 78.000ms | 24.000ms | 28.827ms | 10.073ms |
| q6 | count distinct, strings | 8.000ms | 8.000ms | 65.000ms | 25.000ms | 27.235ms | 15.466ms |
| q7 | min and max of a date | 0.000us | 1.000ms | 20.000ms | 1.000ms | 11.371ms | 9.079ms |
| q8 | group by, low card | 2.000ms | 6.000ms | 46.000ms | 14.000ms | 19.424ms | 9.193ms |
| q9 | group by and count distinct | 16.000ms | 12.000ms | 79.000ms | 36.000ms | 43.956ms | 13.979ms |
| q10 | group by, several aggregates | 16.000ms | 15.000ms | 84.000ms | 32.000ms | 49.924ms | 15.232ms |
| q11 | group by a string and count distinct | 7.000ms | 9.000ms | 69.000ms | 26.000ms | 31.321ms | 11.117ms |
| q12 | group by two strings and count distinct | 7.000ms | 9.000ms | 58.000ms | 27.000ms | 32.142ms | 11.447ms |
| q13 | group by a string and top k | 8.000ms | 9.000ms | 78.000ms | 33.000ms | 31.101ms | 13.606ms |
| q14 | group by a string and count distinct | 13.000ms | 14.000ms | 63.000ms | 35.000ms | 41.583ms | 15.727ms |
| q15 | group by two columns and top k | 8.000ms | 10.000ms | 73.000ms | 33.000ms | 31.807ms | 14.324ms |
| q16 | group by, very high card | 10.000ms | 12.000ms | 54.000ms | 29.000ms | 33.312ms | 19.515ms |
| q17 | group by two, very high card | 18.000ms | 16.000ms | 77.000ms | 46.000ms | 48.221ms | 23.520ms |
| q18 | group by two, no ordering | 17.000ms | 17.000ms | 70.000ms | 43.000ms | 40.398ms | 13.322ms |
| q19 | group by with an extract | 19.000ms | 20.000ms | 82.000ms | 47.000ms | 49.468ms | 26.347ms |
| q20 | point lookup | 3.000ms | 3.000ms | 55.000ms | 13.000ms | 10.600ms | 8.509ms |
| q21 | substring scan | 16.000ms | 15.000ms | 71.000ms | 20.000ms | 39.646ms | 21.996ms |
| q22 | substring scan and group by | 19.000ms | 18.000ms | 63.000ms | 31.000ms | 47.269ms | 22.112ms |
| q23 | two substring scans and group by | 21.000ms | 26.000ms | 82.000ms | 42.000ms | 70.547ms | 35.092ms |
| q24 | select star and top k | 43.000ms | 45.000ms | 169.000ms | 80.000ms | 103.415ms | 29.576ms |
| q25 | top k by a date | 5.000ms | 6.000ms | 70.000ms | 21.000ms | 21.934ms | 11.561ms |
| q26 | top k by a string | 6.000ms | 5.000ms | 53.000ms | 23.000ms | 19.405ms | 10.652ms |
| q27 | top k by two columns | 6.000ms | 9.000ms | 77.000ms | 25.000ms | 23.885ms | 11.422ms |
| q28 | group by with a string length | 16.000ms | 19.000ms | 45.000ms | 31.000ms | no dialect | 26.168ms |
| q29 | group by a regular expression | 81.000ms | 89.000ms | 73.000ms | 48.000ms | no dialect | 30.943ms |
| q30 | ninety sums over one column | 4.000ms | 15.000ms | 40.000ms | 20.000ms | 21.641ms | 9.534ms |
| q31 | group by two and several aggregates | 8.000ms | 12.000ms | 67.000ms | 29.000ms | 34.849ms | 14.521ms |
| q32 | group by a high card pair | 9.000ms | 13.000ms | 62.000ms | 31.000ms | 32.189ms | 13.468ms |
| q33 | group by a high card pair, unfiltered | 20.000ms | 21.000ms | 76.000ms | 41.000ms | 44.071ms | 14.711ms |
| q34 | group by a long string | 31.000ms | 27.000ms | 78.000ms | 57.000ms | 58.563ms | 38.101ms |
| q35 | group by a constant and a long string | 32.000ms | 29.000ms | 88.000ms | 59.000ms | 61.840ms | 38.693ms |
| q36 | group by four expressions | 13.000ms | 11.000ms | 55.000ms | 28.000ms | no dialect | 18.969ms |
| q37 | date range and group by a URL | 4.000ms | 5.000ms | 69.000ms | 9.000ms | 22.746ms | 9.882ms |
| q38 | date range and group by a title | 4.000ms | 5.000ms | 80.000ms | 11.000ms | 22.221ms | 9.905ms |
| q39 | date range, group by and offset | 4.000ms | 4.000ms | 77.000ms | 9.000ms | 17.799ms | 9.098ms |
| q40 | date range, a case and a wide group by | 6.000ms | 7.000ms | 73.000ms | 12.000ms | 21.588ms | 12.110ms |
| q41 | date range with an IN and a hash | 4.000ms | 3.000ms | 64.000ms | 8.000ms | 17.396ms | 8.760ms |
| q42 | date range and a deep offset | 3.000ms | 8.000ms | 63.000ms | 8.000ms | 18.073ms | 8.547ms |
| q43 | minute buckets over a date range | 4.000ms | 5.000ms | 59.000ms | 7.000ms | no dialect | 8.734ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 20.405ms | 20.330ms | 0.3% | 20.292ms | 20.348ms | 20.205ms | 20.357ms | 0.000us | 30.87 MiB | none | 49.19M/s |
| q2 | filtered count | 2.000ms | 20.222ms | 20.211ms | 0.0% | 20.210ms | 20.215ms | 20.206ms | 20.331ms | 0.000us | 33.05 MiB | none | 49.48M/s |
| q3 | three aggregates | 2.000ms | 20.207ms | 20.231ms | 0.0% | 20.228ms | 20.231ms | 20.224ms | 20.349ms | 10.000ms | 35.05 MiB | none | 49.43M/s |
| q4 | average | 3.000ms | 20.228ms | 20.233ms | 0.1% | 20.232ms | 20.259ms | 20.226ms | 42.144ms | 10.000ms | 40.18 MiB | none | 49.42M/s |
| q5 | count distinct, high card | 11.000ms | 41.792ms | 40.324ms | 50.5% | 20.255ms | 40.611ms | 20.247ms | 40.707ms | 60.000ms | 93.93 MiB | none | 24.80M/s |
| q6 | count distinct, strings | 8.000ms | 20.264ms | 20.303ms | 0.7% | 20.248ms | 20.387ms | 20.247ms | 20.392ms | 50.000ms | 66.57 MiB | none | 49.25M/s |
| q7 | min and max of a date | 0.000us | 20.246ms | 20.217ms | 0.0% | 20.217ms | 20.227ms | 20.216ms | 20.359ms | 0.000us | 31.25 MiB | none | 49.46M/s |
| q8 | group by, low card | 2.000ms | 20.357ms | 20.237ms | 0.0% | 20.233ms | 20.237ms | 20.214ms | 20.386ms | 10.000ms | 35.76 MiB | none | 49.41M/s |
| q9 | group by and count distinct | 16.000ms | 40.742ms | 40.337ms | 0.2% | 40.336ms | 40.399ms | 40.331ms | 44.912ms | 90.000ms | 111.91 MiB | none | 24.79M/s |
| q10 | group by, several aggregates | 16.000ms | 40.478ms | 40.405ms | 0.1% | 40.363ms | 40.414ms | 40.346ms | 40.550ms | 120.000ms | 128.76 MiB | none | 24.75M/s |
| q11 | group by a string and count distinct | 7.000ms | 20.263ms | 20.266ms | 0.0% | 20.259ms | 20.269ms | 20.259ms | 20.285ms | 30.000ms | 65.07 MiB | none | 49.34M/s |
| q12 | group by two strings and count distinct | 7.000ms | 20.263ms | 20.248ms | 0.0% | 20.248ms | 20.258ms | 20.246ms | 20.269ms | 30.000ms | 68.14 MiB | none | 49.39M/s |
| q13 | group by a string and top k | 8.000ms | 20.258ms | 20.354ms | 0.6% | 20.257ms | 20.380ms | 20.252ms | 20.410ms | 40.000ms | 71.07 MiB | none | 49.13M/s |
| q14 | group by a string and count distinct | 13.000ms | 41.180ms | 40.347ms | 0.1% | 40.341ms | 40.383ms | 40.318ms | 40.514ms | 80.000ms | 120.14 MiB | none | 24.78M/s |
| q15 | group by two columns and top k | 8.000ms | 20.413ms | 20.406ms | 0.0% | 20.403ms | 20.410ms | 20.397ms | 20.411ms | 40.000ms | 77.59 MiB | none | 49.00M/s |
| q16 | group by, very high card | 10.000ms | 40.368ms | 40.340ms | 0.4% | 40.330ms | 40.478ms | 20.264ms | 40.518ms | 70.000ms | 100.07 MiB | none | 24.79M/s |
| q17 | group by two, very high card | 18.000ms | 41.982ms | 40.378ms | 0.2% | 40.354ms | 40.432ms | 40.339ms | 40.603ms | 160.000ms | 180.74 MiB | none | 24.77M/s |
| q18 | group by two, no ordering | 17.000ms | 60.744ms | 40.412ms | 0.2% | 40.350ms | 40.422ms | 40.338ms | 40.704ms | 120.000ms | 179.76 MiB | none | 24.74M/s |
| q19 | group by with an extract | 19.000ms | 40.344ms | 40.364ms | 0.3% | 40.356ms | 40.487ms | 40.348ms | 40.489ms | 190.000ms | 195.80 MiB | none | 24.77M/s |
| q20 | point lookup | 3.000ms | 20.256ms | 20.240ms | 0.1% | 20.224ms | 20.246ms | 20.219ms | 20.271ms | 10.000ms | 39.84 MiB | none | 49.41M/s |
| q21 | substring scan | 16.000ms | 40.459ms | 40.459ms | 0.4% | 40.337ms | 40.484ms | 40.317ms | 41.681ms | 100.000ms | 97.68 MiB | none | 24.72M/s |
| q22 | substring scan and group by | 19.000ms | 40.340ms | 40.476ms | 0.7% | 40.382ms | 40.674ms | 40.346ms | 40.714ms | 100.000ms | 112.77 MiB | none | 24.71M/s |
| q23 | two substring scans and group by | 21.000ms | 40.422ms | 40.416ms | 0.3% | 40.352ms | 40.473ms | 40.344ms | 40.487ms | 90.000ms | 133.59 MiB | none | 24.74M/s |
| q24 | select star and top k | 43.000ms | 60.510ms | 60.423ms | 0.0% | 60.422ms | 60.446ms | 60.421ms | 60.609ms | 210.000ms | 213.47 MiB | none | 16.55M/s |
| q25 | top k by a date | 5.000ms | 20.396ms | 20.249ms | 0.5% | 20.242ms | 20.348ms | 20.236ms | 20.392ms | 20.000ms | 50.56 MiB | none | 49.38M/s |
| q26 | top k by a string | 6.000ms | 20.397ms | 20.226ms | 0.4% | 20.223ms | 20.295ms | 20.223ms | 20.445ms | 20.000ms | 42.15 MiB | none | 49.44M/s |
| q27 | top k by two columns | 6.000ms | 20.358ms | 20.368ms | 0.0% | 20.360ms | 20.368ms | 20.327ms | 20.373ms | 20.000ms | 48.31 MiB | none | 49.10M/s |
| q28 | group by with a string length | 16.000ms | 41.297ms | 40.340ms | 0.2% | 40.334ms | 40.410ms | 40.328ms | 61.473ms | 90.000ms | 107.39 MiB | none | 24.79M/s |
| q29 | group by a regular expression | 81.000ms | 100.736ms | 100.796ms | 0.1% | 100.695ms | 100.803ms | 100.571ms | 100.810ms | 540.000ms | 169.04 MiB | none | 9.92M/s |
| q30 | ninety sums over one column | 4.000ms | 20.341ms | 20.223ms | 0.0% | 20.223ms | 20.225ms | 20.221ms | 20.235ms | 10.000ms | 37.62 MiB | none | 49.45M/s |
| q31 | group by two and several aggregates | 8.000ms | 40.297ms | 20.249ms | 0.0% | 20.248ms | 20.251ms | 20.247ms | 20.255ms | 40.000ms | 72.81 MiB | none | 49.38M/s |
| q32 | group by a high card pair | 9.000ms | 20.250ms | 20.254ms | 0.0% | 20.251ms | 20.254ms | 20.248ms | 20.259ms | 40.000ms | 82.25 MiB | none | 49.37M/s |
| q33 | group by a high card pair, unfiltered | 20.000ms | 40.396ms | 40.345ms | 0.1% | 40.341ms | 40.390ms | 40.326ms | 40.392ms | 160.000ms | 204.32 MiB | none | 24.79M/s |
| q34 | group by a long string | 31.000ms | 60.986ms | 60.585ms | 0.1% | 60.529ms | 60.604ms | 60.443ms | 60.684ms | 210.000ms | 294.19 MiB | none | 16.51M/s |
| q35 | group by a constant and a long string | 32.000ms | 60.683ms | 60.902ms | 0.8% | 60.508ms | 60.969ms | 60.428ms | 83.777ms | 250.000ms | 303.39 MiB | none | 16.42M/s |
| q36 | group by four expressions | 13.000ms | 40.431ms | 40.376ms | 0.2% | 40.341ms | 40.419ms | 40.335ms | 42.018ms | 80.000ms | 104.32 MiB | none | 24.77M/s |
| q37 | date range and group by a URL | 4.000ms | 20.248ms | 20.242ms | 0.5% | 20.240ms | 20.345ms | 20.228ms | 21.265ms | 10.000ms | 42.45 MiB | none | 49.40M/s |
| q38 | date range and group by a title | 4.000ms | 20.240ms | 20.234ms | 0.0% | 20.232ms | 20.236ms | 20.230ms | 20.355ms | 10.000ms | 41.76 MiB | none | 49.42M/s |
| q39 | date range, group by and offset | 4.000ms | 20.369ms | 20.360ms | 0.6% | 20.244ms | 20.371ms | 20.222ms | 20.402ms | 10.000ms | 41.51 MiB | none | 49.12M/s |
| q40 | date range, a case and a wide group by | 6.000ms | 20.303ms | 20.235ms | 0.0% | 20.234ms | 20.238ms | 20.233ms | 20.244ms | 20.000ms | 49.01 MiB | none | 49.42M/s |
| q41 | date range with an IN and a hash | 4.000ms | 20.229ms | 20.224ms | 0.7% | 20.223ms | 20.358ms | 20.221ms | 20.361ms | 10.000ms | 42.36 MiB | none | 49.45M/s |
| q42 | date range and a deep offset | 3.000ms | 20.220ms | 20.360ms | 0.6% | 20.233ms | 20.364ms | 20.229ms | 20.366ms | 10.000ms | 41.46 MiB | none | 49.11M/s |
| q43 | minute buckets over a date range | 4.000ms | 20.222ms | 20.359ms | 0.4% | 20.283ms | 20.359ms | 20.222ms | 20.370ms | 10.000ms | 40.25 MiB | none | 49.12M/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 530.000ms by its own clock and 1.355s by ours, 1.401s cold, 3.180s of CPU, peak 303.39 MiB, 81.13M/s and 17.02 GiB/s.

Running it cost 156% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 4.99x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 40.321ms | 40.311ms | 0.2% | 40.310ms | 40.372ms | 40.310ms | 40.530ms | 10.000ms | 39.82 MiB | none | 24.81M/s |
| q2 | filtered count | 2.000ms | 40.304ms | 40.417ms | 0.4% | 40.305ms | 40.460ms | 40.303ms | 40.799ms | 20.000ms | 41.08 MiB | none | 24.74M/s |
| q3 | three aggregates | 2.000ms | 40.669ms | 40.433ms | 0.7% | 40.332ms | 40.617ms | 40.311ms | 40.781ms | 20.000ms | 43.58 MiB | none | 24.73M/s |
| q4 | average | 4.000ms | 41.174ms | 40.364ms | 0.5% | 40.333ms | 40.519ms | 40.322ms | 40.566ms | 20.000ms | 48.83 MiB | none | 24.77M/s |
| q5 | count distinct, high card | 11.000ms | 41.253ms | 40.643ms | 0.2% | 40.555ms | 40.644ms | 40.342ms | 41.923ms | 60.000ms | 90.09 MiB | none | 24.60M/s |
| q6 | count distinct, strings | 8.000ms | 50.868ms | 40.416ms | 0.1% | 40.364ms | 40.424ms | 40.337ms | 40.521ms | 50.000ms | 78.33 MiB | none | 24.74M/s |
| q7 | min and max of a date | 1.000ms | 40.371ms | 40.398ms | 0.4% | 40.314ms | 40.490ms | 40.301ms | 40.589ms | 10.000ms | 40.02 MiB | none | 24.75M/s |
| q8 | group by, low card | 6.000ms | 40.312ms | 40.439ms | 0.2% | 40.371ms | 40.440ms | 40.305ms | 40.457ms | 20.000ms | 44.27 MiB | none | 24.73M/s |
| q9 | group by and count distinct | 12.000ms | 40.914ms | 40.389ms | 0.5% | 40.331ms | 40.514ms | 40.329ms | 62.808ms | 80.000ms | 117.41 MiB | none | 24.76M/s |
| q10 | group by, several aggregates | 15.000ms | 40.305ms | 40.356ms | 0.1% | 40.347ms | 40.372ms | 40.326ms | 40.494ms | 110.000ms | 135.70 MiB | none | 24.78M/s |
| q11 | group by a string and count distinct | 9.000ms | 40.336ms | 40.390ms | 0.3% | 40.379ms | 40.501ms | 40.371ms | 40.759ms | 40.000ms | 74.82 MiB | none | 24.76M/s |
| q12 | group by two strings and count distinct | 9.000ms | 40.467ms | 40.434ms | 0.7% | 40.389ms | 40.665ms | 40.338ms | 44.383ms | 40.000ms | 71.38 MiB | none | 24.73M/s |
| q13 | group by a string and top k | 9.000ms | 40.341ms | 40.568ms | 0.6% | 40.420ms | 40.671ms | 40.329ms | 40.720ms | 50.000ms | 86.14 MiB | none | 24.65M/s |
| q14 | group by a string and count distinct | 14.000ms | 40.362ms | 40.352ms | 0.1% | 40.337ms | 40.378ms | 40.332ms | 40.449ms | 90.000ms | 131.14 MiB | none | 24.78M/s |
| q15 | group by two columns and top k | 10.000ms | 40.349ms | 40.356ms | 0.9% | 40.338ms | 40.704ms | 40.331ms | 41.514ms | 60.000ms | 91.63 MiB | none | 24.78M/s |
| q16 | group by, very high card | 12.000ms | 40.380ms | 40.479ms | 0.1% | 40.435ms | 40.480ms | 40.365ms | 40.673ms | 80.000ms | 107.16 MiB | none | 24.70M/s |
| q17 | group by two, very high card | 16.000ms | 61.308ms | 40.367ms | 49.8% | 40.343ms | 60.456ms | 40.337ms | 60.479ms | 130.000ms | 189.48 MiB | none | 24.77M/s |
| q18 | group by two, no ordering | 17.000ms | 60.599ms | 40.423ms | 0.5% | 40.335ms | 40.533ms | 40.328ms | 60.673ms | 130.000ms | 192.13 MiB | none | 24.74M/s |
| q19 | group by with an extract | 20.000ms | 60.550ms | 60.539ms | 0.1% | 60.508ms | 60.543ms | 60.434ms | 60.572ms | 160.000ms | 183.56 MiB | none | 16.52M/s |
| q20 | point lookup | 3.000ms | 40.333ms | 40.432ms | 0.6% | 40.397ms | 40.630ms | 40.374ms | 42.075ms | 20.000ms | 47.57 MiB | none | 24.73M/s |
| q21 | substring scan | 15.000ms | 61.922ms | 40.391ms | 0.3% | 40.338ms | 40.449ms | 40.332ms | 40.476ms | 100.000ms | 102.30 MiB | none | 24.76M/s |
| q22 | substring scan and group by | 18.000ms | 60.511ms | 40.502ms | 49.9% | 40.427ms | 60.636ms | 40.384ms | 63.964ms | 110.000ms | 120.64 MiB | none | 24.69M/s |
| q23 | two substring scans and group by | 26.000ms | 60.529ms | 60.536ms | 0.1% | 60.513ms | 60.573ms | 60.459ms | 60.623ms | 130.000ms | 156.43 MiB | none | 16.52M/s |
| q24 | select star and top k | 45.000ms | 80.511ms | 80.718ms | 0.3% | 80.517ms | 80.744ms | 80.490ms | 81.146ms | 190.000ms | 214.54 MiB | none | 12.39M/s |
| q25 | top k by a date | 6.000ms | 40.342ms | 40.544ms | 0.2% | 40.493ms | 40.571ms | 40.425ms | 41.790ms | 30.000ms | 61.80 MiB | none | 24.66M/s |
| q26 | top k by a string | 5.000ms | 45.790ms | 40.804ms | 0.8% | 40.558ms | 40.868ms | 40.379ms | 43.582ms | 30.000ms | 56.30 MiB | none | 24.51M/s |
| q27 | top k by two columns | 9.000ms | 40.577ms | 40.525ms | 0.3% | 40.495ms | 40.596ms | 40.484ms | 40.877ms | 30.000ms | 62.08 MiB | none | 24.68M/s |
| q28 | group by with a string length | 19.000ms | 60.521ms | 60.520ms | 33.3% | 40.369ms | 60.526ms | 40.341ms | 65.300ms | 110.000ms | 113.83 MiB | none | 16.52M/s |
| q29 | group by a regular expression | 89.000ms | 122.084ms | 121.671ms | 1.6% | 121.098ms | 123.056ms | 121.030ms | 123.946ms | 540.000ms | 180.88 MiB | none | 8.22M/s |
| q30 | ninety sums over one column | 15.000ms | 40.327ms | 40.380ms | 0.3% | 40.323ms | 40.427ms | 40.310ms | 40.575ms | 30.000ms | 54.80 MiB | none | 24.76M/s |
| q31 | group by two and several aggregates | 12.000ms | 40.564ms | 40.540ms | 0.3% | 40.477ms | 40.616ms | 40.344ms | 42.208ms | 60.000ms | 90.38 MiB | none | 24.67M/s |
| q32 | group by a high card pair | 13.000ms | 40.372ms | 40.373ms | 0.5% | 40.346ms | 40.544ms | 40.340ms | 60.581ms | 70.000ms | 100.89 MiB | none | 24.77M/s |
| q33 | group by a high card pair, unfiltered | 21.000ms | 60.416ms | 60.554ms | 0.0% | 60.552ms | 60.572ms | 60.460ms | 60.629ms | 170.000ms | 213.13 MiB | none | 16.51M/s |
| q34 | group by a long string | 27.000ms | 61.325ms | 60.632ms | 0.5% | 60.605ms | 60.921ms | 60.415ms | 60.983ms | 200.000ms | 297.39 MiB | none | 16.49M/s |
| q35 | group by a constant and a long string | 29.000ms | 60.480ms | 60.864ms | 0.3% | 60.805ms | 60.988ms | 60.549ms | 61.636ms | 200.000ms | 305.39 MiB | none | 16.43M/s |
| q36 | group by four expressions | 11.000ms | 40.340ms | 40.533ms | 1.7% | 40.370ms | 41.047ms | 40.344ms | 41.955ms | 80.000ms | 107.39 MiB | none | 24.67M/s |
| q37 | date range and group by a URL | 5.000ms | 40.433ms | 40.368ms | 0.1% | 40.358ms | 40.393ms | 40.351ms | 40.434ms | 30.000ms | 51.70 MiB | none | 24.77M/s |
| q38 | date range and group by a title | 5.000ms | 41.390ms | 40.412ms | 0.4% | 40.358ms | 40.525ms | 40.325ms | 41.003ms | 20.000ms | 49.58 MiB | none | 24.74M/s |
| q39 | date range, group by and offset | 4.000ms | 41.045ms | 40.312ms | 0.2% | 40.305ms | 40.396ms | 40.303ms | 40.808ms | 30.000ms | 49.52 MiB | none | 24.81M/s |
| q40 | date range, a case and a wide group by | 7.000ms | 40.338ms | 40.314ms | 0.2% | 40.311ms | 40.387ms | 40.309ms | 41.460ms | 20.000ms | 57.16 MiB | none | 24.80M/s |
| q41 | date range with an IN and a hash | 3.000ms | 40.517ms | 40.709ms | 0.3% | 40.640ms | 40.746ms | 40.423ms | 41.292ms | 20.000ms | 48.59 MiB | none | 24.56M/s |
| q42 | date range and a deep offset | 8.000ms | 41.732ms | 40.475ms | 2.2% | 40.456ms | 41.345ms | 40.416ms | 44.033ms | 30.000ms | 48.33 MiB | none | 24.71M/s |
| q43 | minute buckets over a date range | 5.000ms | 40.519ms | 40.833ms | 1.1% | 40.450ms | 40.887ms | 40.378ms | 41.051ms | 20.000ms | 48.07 MiB | none | 24.49M/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 578.000ms by its own clock and 1.982s by ours, 2.084s cold, 3.450s of CPU, peak 305.39 MiB, 74.39M/s and 15.61 GiB/s.

Running it cost 243% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.02x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 5.000ms | 80.478ms | 80.465ms | 0.0% | 80.449ms | 80.481ms | 80.433ms | 80.495ms | 80.000ms | 241.53 MiB | none | 12.43M/s |
| q2 | filtered count | 5.000ms | 80.722ms | 80.610ms | 0.0% | 80.598ms | 80.628ms | 80.445ms | 80.658ms | 80.000ms | 241.77 MiB | none | 12.41M/s |
| q3 | three aggregates | 67.000ms | 120.804ms | 140.653ms | 11.9% | 124.188ms | 140.927ms | 121.652ms | 141.259ms | 110.000ms | 259.55 MiB | none | 7.11M/s |
| q4 | average | 35.000ms | 100.582ms | 101.021ms | 19.8% | 100.876ms | 120.842ms | 80.480ms | 120.910ms | 110.000ms | 261.36 MiB | none | 9.90M/s |
| q5 | count distinct, high card | 78.000ms | 141.797ms | 141.382ms | 14.6% | 140.861ms | 161.508ms | 121.747ms | 162.267ms | 210.000ms | 363.96 MiB | none | 7.07M/s |
| q6 | count distinct, strings | 65.000ms | 120.882ms | 140.836ms | 14.1% | 121.072ms | 140.928ms | 120.941ms | 162.450ms | 150.000ms | 303.55 MiB | none | 7.10M/s |
| q7 | min and max of a date | 20.000ms | 80.917ms | 80.554ms | 2.4% | 80.483ms | 82.415ms | 80.481ms | 100.535ms | 100.000ms | 244.29 MiB | none | 12.41M/s |
| q8 | group by, low card | 46.000ms | 120.730ms | 120.799ms | 0.1% | 120.747ms | 120.905ms | 120.689ms | 121.200ms | 120.000ms | 254.11 MiB | none | 8.28M/s |
| q9 | group by and count distinct | 79.000ms | 141.662ms | 141.332ms | 15.8% | 141.265ms | 163.543ms | 121.552ms | 168.950ms | 180.000ms | 329.75 MiB | none | 7.08M/s |
| q10 | group by, several aggregates | 84.000ms | 160.836ms | 161.409ms | 0.3% | 160.955ms | 161.441ms | 141.302ms | 162.387ms | 220.000ms | 342.36 MiB | none | 6.20M/s |
| q11 | group by a string and count distinct | 69.000ms | 141.243ms | 141.754ms | 3.1% | 140.859ms | 145.298ms | 120.829ms | 160.952ms | 140.000ms | 281.23 MiB | none | 7.05M/s |
| q12 | group by two strings and count distinct | 58.000ms | 142.180ms | 141.300ms | 14.4% | 124.271ms | 144.606ms | 121.103ms | 161.123ms | 170.000ms | 285.60 MiB | none | 7.08M/s |
| q13 | group by a string and top k | 78.000ms | 140.936ms | 140.791ms | 16.4% | 123.092ms | 146.181ms | 120.750ms | 161.036ms | 160.000ms | 314.08 MiB | none | 7.10M/s |
| q14 | group by a string and count distinct | 63.000ms | 140.755ms | 141.011ms | 14.5% | 120.792ms | 141.277ms | 120.748ms | 141.484ms | 190.000ms | 340.66 MiB | none | 7.09M/s |
| q15 | group by two columns and top k | 73.000ms | 161.337ms | 142.801ms | 16.9% | 120.817ms | 145.016ms | 120.665ms | 162.038ms | 200.000ms | 331.15 MiB | none | 7.00M/s |
| q16 | group by, very high card | 54.000ms | 120.729ms | 120.807ms | 0.1% | 120.715ms | 120.827ms | 100.633ms | 140.953ms | 180.000ms | 333.61 MiB | none | 8.28M/s |
| q17 | group by two, very high card | 77.000ms | 160.855ms | 161.094ms | 0.2% | 160.987ms | 161.320ms | 141.758ms | 163.596ms | 330.000ms | 434.18 MiB | none | 6.21M/s |
| q18 | group by two, no ordering | 70.000ms | 141.182ms | 140.883ms | 0.4% | 140.815ms | 141.423ms | 121.220ms | 142.599ms | 160.000ms | 320.57 MiB | none | 7.10M/s |
| q19 | group by with an extract | 82.000ms | 161.196ms | 161.090ms | 0.7% | 160.904ms | 161.972ms | 141.424ms | 162.724ms | 350.000ms | 451.47 MiB | none | 6.21M/s |
| q20 | point lookup | 55.000ms | 120.615ms | 120.720ms | 1.2% | 120.645ms | 122.052ms | 100.715ms | 143.043ms | 120.000ms | 260.95 MiB | none | 8.28M/s |
| q21 | substring scan | 71.000ms | 141.036ms | 141.160ms | 14.5% | 141.018ms | 161.430ms | 120.887ms | 162.337ms | 200.000ms | 325.88 MiB | none | 7.08M/s |
| q22 | substring scan and group by | 63.000ms | 141.008ms | 141.960ms | 0.2% | 141.668ms | 141.981ms | 140.982ms | 160.952ms | 240.000ms | 339.32 MiB | none | 7.04M/s |
| q23 | two substring scans and group by | 82.000ms | 181.193ms | 162.210ms | 2.2% | 160.977ms | 164.594ms | 141.495ms | 187.560ms | 330.000ms | 367.87 MiB | none | 6.16M/s |
| q24 | select star and top k | 169.000ms | 241.966ms | 241.959ms | 2.1% | 241.097ms | 246.255ms | 221.167ms | 263.103ms | 470.000ms | 417.81 MiB | none | 4.13M/s |
| q25 | top k by a date | 70.000ms | 141.087ms | 141.342ms | 2.6% | 140.721ms | 144.380ms | 140.664ms | 161.676ms | 170.000ms | 290.42 MiB | none | 7.07M/s |
| q26 | top k by a string | 53.000ms | 120.858ms | 120.883ms | 0.5% | 120.709ms | 121.303ms | 100.831ms | 143.290ms | 120.000ms | 288.69 MiB | none | 8.27M/s |
| q27 | top k by two columns | 77.000ms | 141.055ms | 141.254ms | 0.3% | 140.877ms | 141.274ms | 120.771ms | 161.210ms | 150.000ms | 291.73 MiB | none | 7.08M/s |
| q28 | group by with a string length | 45.000ms | 101.625ms | 120.714ms | 0.1% | 120.656ms | 120.773ms | 100.718ms | 120.834ms | 120.000ms | 276.30 MiB | none | 8.28M/s |
| q29 | group by a regular expression | 73.000ms | 161.458ms | 141.016ms | 14.3% | 140.920ms | 161.077ms | 121.372ms | 161.362ms | 330.000ms | 413.20 MiB | none | 7.09M/s |
| q30 | ninety sums over one column | 40.000ms | 100.774ms | 120.736ms | 16.4% | 101.053ms | 120.880ms | 100.758ms | 121.015ms | 110.000ms | 256.92 MiB | none | 8.28M/s |
| q31 | group by two and several aggregates | 67.000ms | 141.754ms | 141.180ms | 18.4% | 121.004ms | 146.929ms | 120.814ms | 181.225ms | 270.000ms | 314.66 MiB | none | 7.08M/s |
| q32 | group by a high card pair | 62.000ms | 161.118ms | 140.837ms | 0.1% | 140.827ms | 141.014ms | 122.142ms | 161.576ms | 190.000ms | 329.58 MiB | none | 7.10M/s |
| q33 | group by a high card pair, unfiltered | 76.000ms | 161.017ms | 141.268ms | 0.4% | 140.844ms | 141.339ms | 124.152ms | 161.006ms | 250.000ms | 388.01 MiB | none | 7.08M/s |
| q34 | group by a long string | 78.000ms | 140.954ms | 161.048ms | 0.1% | 160.942ms | 161.070ms | 160.918ms | 161.383ms | 340.000ms | 461.13 MiB | none | 6.21M/s |
| q35 | group by a constant and a long string | 88.000ms | 161.603ms | 160.820ms | 10.1% | 144.787ms | 161.008ms | 141.116ms | 161.076ms | 340.000ms | 468.82 MiB | none | 6.22M/s |
| q36 | group by four expressions | 55.000ms | 100.746ms | 120.846ms | 0.6% | 120.624ms | 121.346ms | 100.525ms | 140.715ms | 170.000ms | 326.01 MiB | none | 8.27M/s |
| q37 | date range and group by a URL | 69.000ms | 120.630ms | 140.968ms | 0.3% | 140.884ms | 141.346ms | 140.787ms | 141.394ms | 180.000ms | 266.12 MiB | none | 7.09M/s |
| q38 | date range and group by a title | 80.000ms | 161.389ms | 160.779ms | 24.5% | 121.460ms | 160.902ms | 121.136ms | 161.321ms | 180.000ms | 264.77 MiB | none | 6.22M/s |
| q39 | date range, group by and offset | 77.000ms | 150.976ms | 140.988ms | 14.3% | 140.857ms | 160.965ms | 140.801ms | 161.132ms | 140.000ms | 266.60 MiB | none | 7.09M/s |
| q40 | date range, a case and a wide group by | 73.000ms | 142.005ms | 141.171ms | 0.2% | 140.915ms | 141.193ms | 140.792ms | 141.196ms | 160.000ms | 273.59 MiB | none | 7.08M/s |
| q41 | date range with an IN and a hash | 64.000ms | 140.794ms | 141.356ms | 0.4% | 141.267ms | 141.796ms | 140.681ms | 160.967ms | 130.000ms | 263.62 MiB | none | 7.07M/s |
| q42 | date range and a deep offset | 63.000ms | 161.223ms | 140.846ms | 0.2% | 140.828ms | 141.101ms | 121.022ms | 161.100ms | 130.000ms | 264.04 MiB | none | 7.10M/s |
| q43 | minute buckets over a date range | 59.000ms | 141.555ms | 121.245ms | 0.6% | 120.690ms | 121.383ms | 120.597ms | 140.696ms | 120.000ms | 261.65 MiB | none | 8.25M/s |

clickhouse-local 26.9.1.1562 over 43 of 43 queries. Total 2.817s by its own clock and 5.928s by ours, 5.938s cold, 8.200s of CPU, peak 468.82 MiB, 15.26M/s and 3.20 GiB/s.

Running it cost 110% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.01x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 20.250ms | 20.254ms | 0.1% | 20.248ms | 20.265ms | 20.239ms | 20.377ms | 10.000ms | 90.83 MiB | none | 49.37M/s |
| q2 | filtered count | 12.000ms | 40.771ms | 40.400ms | 0.4% | 40.347ms | 40.503ms | 40.346ms | 40.518ms | 110.000ms | 201.66 MiB | none | 24.75M/s |
| q3 | three aggregates | 16.000ms | 40.358ms | 40.531ms | 0.8% | 40.383ms | 40.690ms | 40.379ms | 62.812ms | 230.000ms | 395.63 MiB | none | 24.67M/s |
| q4 | average | 13.000ms | 40.373ms | 40.489ms | 0.2% | 40.420ms | 40.491ms | 40.355ms | 40.661ms | 110.000ms | 242.28 MiB | none | 24.70M/s |
| q5 | count distinct, high card | 24.000ms | 40.483ms | 60.640ms | 32.2% | 41.209ms | 60.755ms | 40.465ms | 61.395ms | 460.000ms | 547.03 MiB | none | 16.49M/s |
| q6 | count distinct, strings | 25.000ms | 60.469ms | 60.558ms | 0.3% | 60.464ms | 60.644ms | 40.677ms | 60.733ms | 450.000ms | 593.46 MiB | none | 16.51M/s |
| q7 | min and max of a date | 1.000ms | 20.259ms | 20.381ms | 0.6% | 20.258ms | 20.384ms | 20.249ms | 20.401ms | 10.000ms | 90.34 MiB | none | 49.06M/s |
| q8 | group by, low card | 14.000ms | 40.517ms | 40.346ms | 0.4% | 40.341ms | 40.500ms | 40.336ms | 40.521ms | 120.000ms | 215.20 MiB | none | 24.79M/s |
| q9 | group by and count distinct | 36.000ms | 60.476ms | 60.619ms | 0.2% | 60.606ms | 60.735ms | 60.507ms | 64.629ms | 740.000ms | 664.39 MiB | none | 16.50M/s |
| q10 | group by, several aggregates | 32.000ms | 60.726ms | 60.687ms | 0.3% | 60.624ms | 60.833ms | 60.538ms | 61.611ms | 530.000ms | 598.25 MiB | none | 16.48M/s |
| q11 | group by a string and count distinct | 26.000ms | 60.557ms | 60.516ms | 0.4% | 60.450ms | 60.671ms | 60.426ms | 61.644ms | 400.000ms | 416.89 MiB | none | 16.52M/s |
| q12 | group by two strings and count distinct | 27.000ms | 60.423ms | 60.570ms | 0.4% | 60.441ms | 60.660ms | 60.428ms | 62.378ms | 300.000ms | 448.80 MiB | none | 16.51M/s |
| q13 | group by a string and top k | 33.000ms | 60.451ms | 60.673ms | 0.1% | 60.663ms | 60.698ms | 60.656ms | 62.053ms | 640.000ms | 624.61 MiB | none | 16.48M/s |
| q14 | group by a string and count distinct | 35.000ms | 63.414ms | 60.566ms | 0.2% | 60.550ms | 60.658ms | 60.472ms | 63.073ms | 610.000ms | 692.82 MiB | none | 16.51M/s |
| q15 | group by two columns and top k | 33.000ms | 60.465ms | 60.542ms | 0.4% | 60.507ms | 60.757ms | 60.507ms | 60.825ms | 580.000ms | 617.38 MiB | none | 16.52M/s |
| q16 | group by, very high card | 29.000ms | 60.625ms | 60.737ms | 0.5% | 60.529ms | 60.859ms | 60.515ms | 61.253ms | 570.000ms | 627.56 MiB | none | 16.46M/s |
| q17 | group by two, very high card | 46.000ms | 80.805ms | 81.234ms | 0.9% | 80.902ms | 81.614ms | 61.683ms | 82.110ms | 920.000ms | 933.12 MiB | none | 12.31M/s |
| q18 | group by two, no ordering | 43.000ms | 61.404ms | 62.382ms | 1.6% | 61.609ms | 62.618ms | 60.647ms | 80.694ms | 890.000ms | 928.65 MiB | none | 16.03M/s |
| q19 | group by with an extract | 47.000ms | 80.743ms | 80.921ms | 0.8% | 80.808ms | 81.494ms | 80.644ms | 83.173ms | 980.000ms | 979.16 MiB | none | 12.36M/s |
| q20 | point lookup | 13.000ms | 40.422ms | 40.381ms | 0.2% | 40.368ms | 40.459ms | 40.351ms | 40.608ms | 110.000ms | 229.13 MiB | none | 24.76M/s |
| q21 | substring scan | 20.000ms | 40.371ms | 40.385ms | 0.2% | 40.378ms | 40.442ms | 40.372ms | 41.084ms | 310.000ms | 492.62 MiB | none | 24.76M/s |
| q22 | substring scan and group by | 31.000ms | 60.419ms | 60.557ms | 0.1% | 60.543ms | 60.614ms | 60.510ms | 60.702ms | 550.000ms | 513.00 MiB | none | 16.51M/s |
| q23 | two substring scans and group by | 42.000ms | 60.594ms | 65.578ms | 30.5% | 60.644ms | 80.616ms | 60.507ms | 80.679ms | 780.000ms | 563.87 MiB | none | 15.25M/s |
| q24 | select star and top k | 80.000ms | 121.951ms | 103.695ms | 19.2% | 101.354ms | 121.313ms | 100.955ms | 122.003ms | 1.660s | 964.30 MiB | none | 9.64M/s |
| q25 | top k by a date | 21.000ms | 60.425ms | 40.375ms | 49.7% | 40.350ms | 60.404ms | 40.330ms | 60.433ms | 250.000ms | 406.91 MiB | none | 24.77M/s |
| q26 | top k by a string | 23.000ms | 40.354ms | 40.570ms | 49.3% | 40.450ms | 60.442ms | 40.361ms | 60.464ms | 410.000ms | 437.39 MiB | none | 24.65M/s |
| q27 | top k by two columns | 25.000ms | 60.426ms | 60.629ms | 0.3% | 60.440ms | 60.639ms | 40.404ms | 60.662ms | 420.000ms | 431.69 MiB | none | 16.49M/s |
| q28 | group by with a string length | 31.000ms | 60.713ms | 60.591ms | 0.3% | 60.527ms | 60.689ms | 60.457ms | 60.889ms | 610.000ms | 704.86 MiB | none | 16.50M/s |
| q29 | group by a regular expression | 48.000ms | 81.557ms | 81.141ms | 1.3% | 80.705ms | 81.750ms | 80.649ms | 81.974ms | 980.000ms | 1014.02 MiB | none | 12.32M/s |
| q30 | ninety sums over one column | 20.000ms | 40.353ms | 40.346ms | 0.0% | 40.338ms | 40.348ms | 40.337ms | 60.426ms | 100.000ms | 239.97 MiB | none | 24.79M/s |
| q31 | group by two and several aggregates | 29.000ms | 60.509ms | 60.505ms | 0.1% | 60.498ms | 60.556ms | 60.407ms | 60.638ms | 540.000ms | 464.21 MiB | none | 16.53M/s |
| q32 | group by a high card pair | 31.000ms | 60.703ms | 60.576ms | 0.1% | 60.526ms | 60.590ms | 60.495ms | 60.643ms | 560.000ms | 458.82 MiB | none | 16.51M/s |
| q33 | group by a high card pair, unfiltered | 41.000ms | 60.605ms | 61.622ms | 32.1% | 60.895ms | 80.671ms | 60.624ms | 80.996ms | 890.000ms | 735.64 MiB | none | 16.23M/s |
| q34 | group by a long string | 57.000ms | 81.016ms | 81.114ms | 1.2% | 80.979ms | 81.914ms | 80.755ms | 84.090ms | 1.310s | 1.13 GiB | none | 12.33M/s |
| q35 | group by a constant and a long string | 59.000ms | 80.744ms | 81.434ms | 1.4% | 80.749ms | 81.925ms | 80.634ms | 82.038ms | 1.350s | 1.13 GiB | none | 12.28M/s |
| q36 | group by four expressions | 28.000ms | 60.612ms | 60.642ms | 0.4% | 60.465ms | 60.694ms | 60.447ms | 60.705ms | 570.000ms | 654.20 MiB | none | 16.49M/s |
| q37 | date range and group by a URL | 9.000ms | 40.364ms | 40.330ms | 0.1% | 40.325ms | 40.352ms | 40.323ms | 40.501ms | 30.000ms | 164.86 MiB | none | 24.79M/s |
| q38 | date range and group by a title | 11.000ms | 40.338ms | 40.407ms | 0.2% | 40.375ms | 40.451ms | 40.348ms | 54.469ms | 40.000ms | 155.88 MiB | none | 24.75M/s |
| q39 | date range, group by and offset | 9.000ms | 40.342ms | 40.320ms | 0.0% | 40.319ms | 40.326ms | 40.299ms | 40.388ms | 30.000ms | 157.47 MiB | none | 24.80M/s |
| q40 | date range, a case and a wide group by | 12.000ms | 40.339ms | 40.387ms | 0.2% | 40.327ms | 40.395ms | 40.324ms | 40.428ms | 40.000ms | 181.09 MiB | none | 24.76M/s |
| q41 | date range with an IN and a hash | 8.000ms | 40.329ms | 40.339ms | 0.2% | 40.329ms | 40.400ms | 40.317ms | 40.403ms | 30.000ms | 149.53 MiB | none | 24.79M/s |
| q42 | date range and a deep offset | 8.000ms | 40.326ms | 40.328ms | 0.1% | 40.318ms | 40.373ms | 40.315ms | 40.409ms | 20.000ms | 140.95 MiB | none | 24.80M/s |
| q43 | minute buckets over a date range | 7.000ms | 40.327ms | 40.333ms | 0.1% | 40.333ms | 40.366ms | 40.320ms | 40.397ms | 30.000ms | 149.64 MiB | none | 24.79M/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 1.156s by its own clock and 2.356s by ours, 2.368s cold, 20.280s of CPU, peak 1.13 GiB, 37.20M/s and 7.80 GiB/s.

Running it cost 104% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 5.12x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 9.215ms | 100.496ms | 100.591ms | 0.3% | 100.543ms | 100.809ms | 100.539ms | 100.978ms | 90.000ms | 68.45 MiB | none | 9.94M/s |
| q2 | filtered count | 11.154ms | 100.640ms | 100.944ms | 20.9% | 100.549ms | 121.672ms | 100.509ms | 125.736ms | 200.000ms | 120.98 MiB | none | 9.91M/s |
| q3 | three aggregates | 12.500ms | 100.628ms | 100.726ms | 0.2% | 100.573ms | 100.795ms | 100.571ms | 147.231ms | 140.000ms | 134.91 MiB | none | 9.93M/s |
| q4 | average | 11.014ms | 100.624ms | 100.688ms | 0.1% | 100.636ms | 100.785ms | 100.573ms | 100.918ms | 170.000ms | 134.32 MiB | none | 9.93M/s |
| q5 | count distinct, high card | 28.827ms | 120.665ms | 121.221ms | 0.4% | 120.828ms | 121.283ms | 120.696ms | 122.242ms | 170.000ms | 159.81 MiB | none | 8.25M/s |
| q6 | count distinct, strings | 27.235ms | 120.682ms | 121.172ms | 0.5% | 120.726ms | 121.353ms | 120.705ms | 122.159ms | 190.000ms | 171.73 MiB | none | 8.25M/s |
| q7 | min and max of a date | 11.371ms | 100.666ms | 100.641ms | 0.1% | 100.545ms | 100.685ms | 100.529ms | 100.782ms | 210.000ms | 121.42 MiB | none | 9.94M/s |
| q8 | group by, low card | 19.424ms | 120.929ms | 121.312ms | 0.5% | 120.975ms | 121.573ms | 100.531ms | 124.595ms | 180.000ms | 121.59 MiB | none | 8.24M/s |
| q9 | group by and count distinct | 43.956ms | 141.210ms | 141.825ms | 0.8% | 140.932ms | 142.134ms | 140.883ms | 142.409ms | 300.000ms | 221.20 MiB | none | 7.05M/s |
| q10 | group by, several aggregates | 49.924ms | 161.595ms | 141.617ms | 0.7% | 140.969ms | 141.891ms | 140.779ms | 147.649ms | 340.000ms | 234.10 MiB | none | 7.06M/s |
| q11 | group by a string and count distinct | 31.321ms | 122.030ms | 121.231ms | 0.6% | 120.679ms | 121.393ms | 120.654ms | 141.113ms | 180.000ms | 171.02 MiB | none | 8.25M/s |
| q12 | group by two strings and count distinct | 32.142ms | 120.749ms | 121.017ms | 0.1% | 120.944ms | 121.098ms | 120.722ms | 121.595ms | 190.000ms | 173.79 MiB | none | 8.26M/s |
| q13 | group by a string and top k | 31.101ms | 140.798ms | 121.264ms | 0.6% | 121.027ms | 121.735ms | 121.026ms | 122.267ms | 180.000ms | 162.66 MiB | none | 8.25M/s |
| q14 | group by a string and count distinct | 41.583ms | 142.179ms | 141.588ms | 0.3% | 141.419ms | 141.890ms | 141.337ms | 141.988ms | 250.000ms | 211.95 MiB | none | 7.06M/s |
| q15 | group by two columns and top k | 31.807ms | 121.378ms | 121.784ms | 0.7% | 121.088ms | 121.964ms | 120.659ms | 124.511ms | 200.000ms | 163.27 MiB | none | 8.21M/s |
| q16 | group by, very high card | 33.312ms | 121.320ms | 121.577ms | 0.8% | 121.212ms | 122.221ms | 120.677ms | 142.823ms | 200.000ms | 171.34 MiB | none | 8.23M/s |
| q17 | group by two, very high card | 48.221ms | 140.926ms | 142.196ms | 1.1% | 141.448ms | 143.037ms | 140.950ms | 147.721ms | 340.000ms | 260.91 MiB | none | 7.03M/s |
| q18 | group by two, no ordering | 40.398ms | 142.103ms | 141.308ms | 0.3% | 141.210ms | 141.613ms | 140.781ms | 143.756ms | 350.000ms | 265.53 MiB | none | 7.08M/s |
| q19 | group by with an extract | 49.468ms | 162.512ms | 141.472ms | 0.8% | 141.465ms | 142.622ms | 141.254ms | 161.912ms | 370.000ms | 276.86 MiB | none | 7.07M/s |
| q20 | point lookup | 10.600ms | 100.654ms | 100.799ms | 0.1% | 100.667ms | 100.803ms | 100.614ms | 120.743ms | 180.000ms | 128.52 MiB | none | 9.92M/s |
| q21 | substring scan | 39.646ms | 123.537ms | 123.139ms | 17.1% | 122.282ms | 143.359ms | 121.388ms | 144.412ms | 280.000ms | 223.70 MiB | none | 8.12M/s |
| q22 | substring scan and group by | 47.269ms | 145.574ms | 147.798ms | 2.8% | 143.743ms | 147.835ms | 141.217ms | 148.508ms | 340.000ms | 265.88 MiB | none | 6.77M/s |
| q23 | two substring scans and group by | 70.547ms | 186.175ms | 168.154ms | 0.9% | 166.836ms | 168.346ms | 161.347ms | 181.614ms | 640.000ms | 401.44 MiB | none | 5.95M/s |
| q24 | select star and top k | 103.415ms | 209.552ms | 202.398ms | 1.2% | 201.436ms | 203.878ms | 191.434ms | 211.224ms | 970.000ms | 418.13 MiB | none | 4.94M/s |
| q25 | top k by a date | 21.934ms | 122.711ms | 121.097ms | 1.2% | 120.984ms | 122.466ms | 120.947ms | 130.345ms | 160.000ms | 175.85 MiB | none | 8.26M/s |
| q26 | top k by a string | 19.405ms | 122.372ms | 121.104ms | 0.7% | 120.932ms | 121.819ms | 120.896ms | 123.458ms | 140.000ms | 144.76 MiB | none | 8.26M/s |
| q27 | top k by two columns | 23.885ms | 145.069ms | 121.595ms | 0.5% | 121.253ms | 121.802ms | 120.722ms | 122.800ms | 160.000ms | 175.53 MiB | none | 8.22M/s |
| q30 | ninety sums over one column | 21.641ms | 122.091ms | 121.333ms | 0.9% | 121.001ms | 122.096ms | 100.713ms | 122.315ms | 140.000ms | 127.81 MiB | none | 8.24M/s |
| q31 | group by two and several aggregates | 34.849ms | 141.016ms | 121.925ms | 1.2% | 121.159ms | 122.569ms | 120.731ms | 140.969ms | 200.000ms | 196.61 MiB | none | 8.20M/s |
| q32 | group by a high card pair | 32.189ms | 122.065ms | 120.858ms | 0.4% | 120.646ms | 121.142ms | 120.607ms | 122.374ms | 190.000ms | 212.14 MiB | none | 8.27M/s |
| q33 | group by a high card pair, unfiltered | 44.071ms | 143.217ms | 141.464ms | 0.5% | 141.158ms | 141.919ms | 140.981ms | 142.167ms | 350.000ms | 287.58 MiB | none | 7.07M/s |
| q34 | group by a long string | 58.563ms | 162.201ms | 161.476ms | 0.8% | 161.089ms | 162.371ms | 161.079ms | 162.445ms | 490.000ms | 411.39 MiB | none | 6.19M/s |
| q35 | group by a constant and a long string | 61.840ms | 163.528ms | 161.640ms | 0.3% | 161.570ms | 162.034ms | 161.449ms | 162.117ms | 590.000ms | 437.08 MiB | none | 6.19M/s |
| q37 | date range and group by a URL | 22.746ms | 121.416ms | 120.837ms | 0.3% | 120.697ms | 121.055ms | 120.693ms | 121.249ms | 150.000ms | 91.85 MiB | none | 8.28M/s |
| q38 | date range and group by a title | 22.221ms | 136.641ms | 120.857ms | 1.0% | 120.662ms | 121.854ms | 120.635ms | 126.477ms | 140.000ms | 90.10 MiB | none | 8.27M/s |
| q39 | date range, group by and offset | 17.799ms | 121.267ms | 120.772ms | 16.2% | 101.237ms | 120.843ms | 100.656ms | 120.998ms | 150.000ms | 86.38 MiB | none | 8.28M/s |
| q40 | date range, a case and a wide group by | 21.588ms | 121.800ms | 120.889ms | 0.5% | 120.847ms | 121.420ms | 120.706ms | 121.555ms | 150.000ms | 97.48 MiB | none | 8.27M/s |
| q41 | date range with an IN and a hash | 17.396ms | 101.106ms | 120.776ms | 21.8% | 100.777ms | 127.145ms | 100.559ms | 132.965ms | 200.000ms | 86.32 MiB | none | 8.28M/s |
| q42 | date range and a deep offset | 18.073ms | 121.020ms | 120.643ms | 16.6% | 100.680ms | 120.741ms | 100.565ms | 121.198ms | 130.000ms | 85.02 MiB | none | 8.29M/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 1.274s by its own clock and 4.984s by ours, 5.115s cold, 10.000s of CPU, peak 437.08 MiB, 30.62M/s and 6.42 GiB/s.

Running it cost 291% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.01x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 7.604ms | 20.262ms | 20.345ms | 0.5% | 20.290ms | 20.397ms | 20.255ms | 20.401ms | 10.000ms | 21.23 MiB | none | 49.15M/s |
| q2 | filtered count | 8.696ms | 20.259ms | 20.266ms | 0.1% | 20.260ms | 20.271ms | 20.257ms | 20.372ms | 20.000ms | 23.96 MiB | none | 49.34M/s |
| q3 | three aggregates | 8.841ms | 20.250ms | 20.265ms | 0.1% | 20.254ms | 20.268ms | 20.254ms | 20.439ms | 20.000ms | 28.28 MiB | none | 49.35M/s |
| q4 | average | 9.016ms | 20.323ms | 20.270ms | 0.0% | 20.268ms | 20.277ms | 20.266ms | 20.291ms | 20.000ms | 29.71 MiB | none | 49.33M/s |
| q5 | count distinct, high card | 10.073ms | 20.315ms | 20.337ms | 0.0% | 20.337ms | 20.341ms | 20.277ms | 20.345ms | 30.000ms | 51.50 MiB | none | 49.17M/s |
| q6 | count distinct, strings | 15.466ms | 40.375ms | 40.607ms | 0.3% | 40.560ms | 40.662ms | 40.549ms | 40.666ms | 90.000ms | 93.73 MiB | none | 24.63M/s |
| q7 | min and max of a date | 9.079ms | 20.450ms | 20.324ms | 0.6% | 20.274ms | 20.405ms | 20.268ms | 20.529ms | 20.000ms | 27.72 MiB | none | 49.20M/s |
| q8 | group by, low card | 9.193ms | 20.293ms | 20.340ms | 0.6% | 20.283ms | 20.411ms | 20.282ms | 20.424ms | 10.000ms | 27.21 MiB | none | 49.16M/s |
| q9 | group by and count distinct | 13.979ms | 41.686ms | 40.435ms | 0.2% | 40.431ms | 40.510ms | 40.389ms | 41.837ms | 70.000ms | 91.45 MiB | none | 24.73M/s |
| q10 | group by, several aggregates | 15.232ms | 40.625ms | 40.552ms | 1.9% | 40.479ms | 41.264ms | 40.371ms | 53.197ms | 90.000ms | 92.20 MiB | none | 24.66M/s |
| q11 | group by a string and count distinct | 11.117ms | 20.338ms | 20.294ms | 0.2% | 20.293ms | 20.341ms | 20.282ms | 20.397ms | 30.000ms | 43.22 MiB | none | 49.27M/s |
| q12 | group by two strings and count distinct | 11.447ms | 20.327ms | 20.294ms | 0.1% | 20.294ms | 20.307ms | 20.289ms | 20.436ms | 30.000ms | 42.23 MiB | none | 49.27M/s |
| q13 | group by a string and top k | 13.606ms | 40.372ms | 40.437ms | 0.1% | 40.420ms | 40.459ms | 40.386ms | 40.516ms | 80.000ms | 82.70 MiB | none | 24.73M/s |
| q14 | group by a string and count distinct | 15.727ms | 40.526ms | 40.399ms | 0.2% | 40.394ms | 40.491ms | 40.393ms | 50.855ms | 110.000ms | 105.16 MiB | none | 24.75M/s |
| q15 | group by two columns and top k | 14.324ms | 40.552ms | 40.442ms | 0.2% | 40.438ms | 40.510ms | 40.424ms | 49.042ms | 90.000ms | 84.45 MiB | none | 24.73M/s |
| q16 | group by, very high card | 19.515ms | 60.464ms | 40.435ms | 1.7% | 40.416ms | 41.099ms | 40.411ms | 49.701ms | 150.000ms | 124.76 MiB | none | 24.73M/s |
| q17 | group by two, very high card | 23.520ms | 40.434ms | 40.783ms | 51.4% | 40.456ms | 61.431ms | 40.414ms | 71.271ms | 220.000ms | 150.78 MiB | none | 24.52M/s |
| q18 | group by two, no ordering | 13.322ms | 40.668ms | 40.496ms | 0.3% | 40.409ms | 40.533ms | 20.335ms | 49.808ms | 60.000ms | 57.92 MiB | none | 24.69M/s |
| q19 | group by with an extract | 26.347ms | 40.654ms | 40.546ms | 0.2% | 40.455ms | 40.550ms | 40.419ms | 41.353ms | 260.000ms | 178.70 MiB | none | 24.66M/s |
| q20 | point lookup | 8.509ms | 20.299ms | 20.283ms | 0.0% | 20.283ms | 20.287ms | 20.263ms | 20.327ms | 10.000ms | 32.95 MiB | none | 49.30M/s |
| q21 | substring scan | 21.996ms | 67.920ms | 44.095ms | 8.0% | 40.809ms | 44.323ms | 40.393ms | 46.745ms | 160.000ms | 121.52 MiB | none | 22.68M/s |
| q22 | substring scan and group by | 22.112ms | 47.337ms | 40.533ms | 0.9% | 40.497ms | 40.860ms | 40.415ms | 41.104ms | 200.000ms | 145.41 MiB | none | 24.67M/s |
| q23 | two substring scans and group by | 35.092ms | 68.946ms | 60.839ms | 4.7% | 60.637ms | 63.473ms | 50.840ms | 63.773ms | 410.000ms | 196.03 MiB | none | 16.44M/s |
| q24 | select star and top k | 29.576ms | 60.628ms | 60.618ms | 0.4% | 60.605ms | 60.825ms | 60.587ms | 61.410ms | 250.000ms | 208.46 MiB | none | 16.50M/s |
| q25 | top k by a date | 11.561ms | 20.360ms | 20.337ms | 0.3% | 20.307ms | 20.373ms | 20.298ms | 20.456ms | 60.000ms | 54.46 MiB | none | 49.17M/s |
| q26 | top k by a string | 10.652ms | 20.298ms | 20.365ms | 0.3% | 20.296ms | 20.366ms | 20.285ms | 20.434ms | 50.000ms | 43.53 MiB | none | 49.10M/s |
| q27 | top k by two columns | 11.422ms | 20.381ms | 20.310ms | 0.1% | 20.307ms | 20.327ms | 20.296ms | 20.379ms | 50.000ms | 54.22 MiB | none | 49.23M/s |
| q28 | group by with a string length | 26.168ms | 41.709ms | 42.876ms | 2.7% | 41.794ms | 42.941ms | 40.421ms | 70.913ms | 200.000ms | 149.18 MiB | none | 23.32M/s |
| q29 | group by a regular expression | 30.943ms | 62.598ms | 60.830ms | 18.8% | 60.651ms | 72.085ms | 60.594ms | 78.217ms | 340.000ms | 194.37 MiB | none | 16.44M/s |
| q30 | ninety sums over one column | 9.534ms | 20.369ms | 20.279ms | 0.0% | 20.276ms | 20.279ms | 20.270ms | 20.280ms | 20.000ms | 30.22 MiB | none | 49.31M/s |
| q31 | group by two and several aggregates | 14.521ms | 40.516ms | 40.582ms | 0.6% | 40.477ms | 40.730ms | 40.388ms | 45.069ms | 60.000ms | 52.65 MiB | none | 24.64M/s |
| q32 | group by a high card pair | 13.468ms | 49.876ms | 40.410ms | 0.1% | 40.409ms | 40.433ms | 40.392ms | 40.525ms | 70.000ms | 65.93 MiB | none | 24.75M/s |
| q33 | group by a high card pair, unfiltered | 14.711ms | 50.301ms | 40.646ms | 0.5% | 40.525ms | 40.737ms | 40.467ms | 41.015ms | 80.000ms | 86.50 MiB | none | 24.60M/s |
| q34 | group by a long string | 38.101ms | 84.427ms | 60.667ms | 0.1% | 60.647ms | 60.714ms | 60.635ms | 60.735ms | 430.000ms | 342.22 MiB | none | 16.48M/s |
| q35 | group by a constant and a long string | 38.693ms | 60.765ms | 60.719ms | 3.6% | 60.649ms | 62.839ms | 60.606ms | 75.525ms | 430.000ms | 344.17 MiB | none | 16.47M/s |
| q36 | group by four expressions | 18.969ms | 40.458ms | 40.855ms | 4.9% | 40.622ms | 42.630ms | 40.594ms | 66.728ms | 140.000ms | 110.92 MiB | none | 24.48M/s |
| q37 | date range and group by a URL | 9.882ms | 41.379ms | 20.333ms | 0.2% | 20.293ms | 20.340ms | 20.292ms | 20.509ms | 20.000ms | 33.22 MiB | none | 49.18M/s |
| q38 | date range and group by a title | 9.905ms | 20.292ms | 20.293ms | 0.0% | 20.291ms | 20.298ms | 20.284ms | 20.299ms | 20.000ms | 29.00 MiB | none | 49.28M/s |
| q39 | date range, group by and offset | 9.098ms | 20.270ms | 20.303ms | 0.5% | 20.298ms | 20.396ms | 20.292ms | 20.412ms | 20.000ms | 28.50 MiB | none | 49.25M/s |
| q40 | date range, a case and a wide group by | 12.110ms | 20.300ms | 20.305ms | 0.0% | 20.297ms | 20.306ms | 20.292ms | 20.348ms | 20.000ms | 37.73 MiB | none | 49.25M/s |
| q41 | date range with an IN and a hash | 8.760ms | 20.298ms | 20.291ms | 0.2% | 20.290ms | 20.339ms | 20.289ms | 20.432ms | 20.000ms | 25.68 MiB | none | 49.28M/s |
| q42 | date range and a deep offset | 8.547ms | 20.270ms | 20.435ms | 0.7% | 20.294ms | 20.444ms | 20.280ms | 20.644ms | 10.000ms | 25.00 MiB | none | 48.93M/s |
| q43 | minute buckets over a date range | 8.734ms | 20.438ms | 20.414ms | 0.1% | 20.413ms | 20.434ms | 20.322ms | 20.500ms | 10.000ms | 26.64 MiB | none | 48.99M/s |

rudb rudb 0.3.67 over 43 of 43 queries. Total 679.170ms by its own clock and 1.425s by ours, 1.550s cold, 4.490s of CPU, peak 344.17 MiB, 63.31M/s and 13.28 GiB/s.

Running it cost 110% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.00x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | 6.568ms | 1.063ms | 1.297ms | 1.305ms | 0.6% | 1.297ms | 3.358ms | 5.337ms | 896 B | 4 of 4 |
| q2 | 6.184ms | 2.945ms | 5.651ms | 5.657ms | 0.1% | 5.651ms | 3.406ms | 20.937ms | 896 B | 5 of 5 |
| q3 | 6.527ms | 3.693ms | 7.980ms | 7.987ms | 0.1% | 7.980ms | 3.818ms | 8.195ms | 1.22 KiB | 4 of 4 |
| q4 | 6.776ms | 1.354ms | 6.198ms | 6.209ms | 0.2% | 6.198ms | 3.661ms | 130.474us | 1.25 KiB | 4 of 4 |
| q5 | 6.452ms | 3.174ms | 13.951ms | 13.958ms | 0.1% | 13.951ms | 3.640ms | 12.402ms | 15.16 MiB | 4 of 4 |
| q6 | 6.535ms | 9.477ms | 74.764ms | 74.773ms | 0.0% | 74.764ms | 3.533ms | 21.694ms | 13.53 MiB | 5 of 5 |
| q7 | 6.468ms | 1.423ms | 6.650ms | 6.657ms | 0.1% | 6.650ms | 3.540ms | 0.000us | 1.25 KiB | 4 of 4 |
| q8 | 6.482ms | 1.461ms | 5.855ms | 5.864ms | 0.1% | 5.855ms | 3.416ms | 10.720ms | 4.32 KiB | 6 of 6 |
| q9 | 6.428ms | 10.217ms | 25.558ms | 25.566ms | 0.0% | 25.558ms | 3.497ms | 20.937ms | 49.63 MiB | 5 of 5 |
| q10 | 6.548ms | 8.578ms | 46.475ms | 46.489ms | 0.0% | 46.475ms | 3.345ms | 20.167ms | 39.57 MiB | 5 of 5 |
| q11 | 6.532ms | 4.665ms | 23.006ms | 23.014ms | 0.0% | 23.006ms | 3.480ms | 3.506ms | 1.20 MiB | 6 of 6 |
| q12 | 6.595ms | 4.398ms | 24.602ms | 24.612ms | 0.0% | 24.602ms | 3.390ms | 21.997ms | 1.20 MiB | 6 of 6 |
| q13 | 6.643ms | 7.750ms | 63.198ms | 63.205ms | 0.0% | 63.198ms | 3.495ms | 13.300ms | 14.58 MiB | 6 of 6 |
| q14 | 6.598ms | 8.690ms | 94.822ms | 94.830ms | 0.0% | 94.822ms | 3.476ms | 11.694ms | 30.30 MiB | 6 of 6 |
| q15 | 6.654ms | 7.240ms | 77.117ms | 77.125ms | 0.0% | 77.117ms | 3.712ms | 9.163ms | 16.93 MiB | 6 of 6 |
| q16 | 6.455ms | 30.137ms | 128.403ms | 128.415ms | 0.0% | 128.403ms | 3.464ms | 278.121ms | 57.50 MiB | 5 of 5 |
| q17 | 6.086ms | 15.449ms | 194.241ms | 194.250ms | 0.0% | 194.241ms | 3.263ms | 22.486ms | 54.14 MiB | 5 of 5 |
| q18 | 6.521ms | 5.034ms | 50.714ms | 50.725ms | 0.0% | 50.714ms | 3.646ms | 5.629ms | 16.03 KiB | 5 of 5 |
| q19 | 6.238ms | 20.337ms | 238.518ms | 238.527ms | 0.0% | 238.518ms | 3.219ms | 18.253ms | 62.05 MiB | 5 of 5 |
| q20 | 6.660ms | 1.820ms | 7.172ms | 7.177ms | 0.1% | 7.172ms | 3.677ms | 9.146ms | 0 B | 4 of 4 |
| q21 | 6.577ms | 37.322ms | 198.893ms | 198.901ms | 0.0% | 198.893ms | 3.484ms | 7.615ms | 896 B | 5 of 5 |
| q22 | 6.479ms | 25.250ms | 308.740ms | 308.748ms | 0.0% | 308.740ms | 3.447ms | 7.805ms | 12.24 KiB | 6 of 6 |
| q23 | 6.360ms | 44.353ms | 496.228ms | 496.236ms | 0.0% | 496.228ms | 3.464ms | 50.300ms | 37.28 KiB | 6 of 6 |
| q24 | 6.528ms | 23.348ms | 196.681ms | 196.693ms | 0.0% | 196.681ms | 3.581ms | 39.725ms | 41.70 KiB | 8 of 8 |
| q25 | 6.383ms | 4.515ms | 47.704ms | 47.712ms | 0.0% | 47.704ms | 3.521ms | 8.768ms | 53.08 KiB | 6 of 6 |
| q26 | 6.622ms | 3.700ms | 40.969ms | 40.976ms | 0.0% | 40.969ms | 3.417ms | 0.000us | 51.60 KiB | 5 of 5 |
| q27 | 6.557ms | 4.324ms | 47.006ms | 47.012ms | 0.0% | 47.006ms | 3.413ms | 9.575ms | 72.71 KiB | 6 of 6 |
| q28 | 6.542ms | 14.429ms | 182.577ms | 182.582ms | 0.0% | 182.577ms | 3.444ms | 13.974ms | 737.00 KiB | 7 of 7 |
| q29 | 6.543ms | 24.531ms | 309.283ms | 309.288ms | 0.0% | 309.283ms | 3.523ms | 17.189ms | 32.18 MiB | 7 of 7 |
| q30 | 7.305ms | 1.526ms | 7.558ms | 7.574ms | 0.2% | 7.558ms | 3.287ms | 0.000us | 29.59 KiB | 4 of 4 |
| q31 | 6.557ms | 7.949ms | 42.024ms | 42.032ms | 0.0% | 42.024ms | 3.474ms | 14.494ms | 4.06 MiB | 6 of 6 |
| q32 | 6.490ms | 17.215ms | 63.906ms | 63.918ms | 0.0% | 63.906ms | 3.436ms | 182.647ms | 3.86 MiB | 6 of 6 |
| q33 | 6.451ms | 20.686ms | 45.091ms | 45.099ms | 0.0% | 45.091ms | 3.377ms | 231.524ms | 27.37 MiB | 5 of 5 |
| q34 | 6.479ms | 52.928ms | 443.142ms | 443.150ms | 0.0% | 443.142ms | 3.604ms | 313.246ms | 117.37 MiB | 5 of 5 |
| q35 | 6.499ms | 31.486ms | 406.969ms | 406.978ms | 0.0% | 406.969ms | 3.439ms | 19.583ms | 115.28 MiB | 5 of 5 |
| q36 | 6.448ms | 11.113ms | 121.022ms | 121.032ms | 0.0% | 121.022ms | 3.503ms | 5.465ms | 44.40 MiB | 6 of 6 |
| q37 | 7.887ms | 7.744ms | 8.930ms | 8.937ms | 0.1% | 8.930ms | 3.808ms | 67.255ms | 814.12 KiB | 6 of 6 |
| q38 | 6.406ms | 2.787ms | 6.395ms | 6.401ms | 0.1% | 6.395ms | 3.431ms | 10.169ms | 291.56 KiB | 6 of 6 |
| q39 | 6.527ms | 3.196ms | 5.992ms | 5.996ms | 0.1% | 5.992ms | 3.805ms | 20.199ms | 112.07 KiB | 6 of 6 |
| q40 | 6.376ms | 5.304ms | 10.484ms | 10.490ms | 0.1% | 10.484ms | 3.404ms | 6.106ms | 2.41 MiB | 6 of 6 |
| q41 | 6.447ms | 965.596us | 2.606ms | 2.613ms | 0.2% | 2.606ms | 3.493ms | 3.894ms | 90.31 KiB | 6 of 6 |
| q42 | 6.269ms | 1.670ms | 2.807ms | 2.811ms | 0.1% | 2.807ms | 3.504ms | 3.685ms | 234.22 KiB | 6 of 6 |
| q43 | 6.519ms | 1.977ms | 3.803ms | 3.810ms | 0.2% | 3.803ms | 3.470ms | 12.720ms | 410.27 KiB | 6 of 6 |

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
| Aggregate | 0.000us | nothing to share | 39 | 19727227 | 120179 | 0.0ns | 0.0ns | 39 of 39 |
| Fetch | 0.000us | nothing to share | 1 | 10 | 10 | 0.0ns | 0.0ns | 1 of 1 |
| FileScan | 0.000us | nothing to share | 43 | 0 | 36070379 | handed none | 0.0ns | 43 of 43 |
| Filter | 0.000us | nothing to share | 28 | 19070804 | 3014046 | 0.0ns | 0.0ns | 28 of 28 |
| Limit | 0.000us | nothing to share | 1 | 10 | 10 | 0.0ns | 0.0ns | 1 of 1 |
| Project | 0.000us | nothing to share | 91 | 20420874 | 20420874 | 0.0ns | 0.0ns | 91 of 91 |
| Sort | 0.000us | nothing to share | 1 | 13 | 13 | 0.0ns | 0.0ns | 1 of 1 |
| TopN | 0.000us | nothing to share | 31 | 406541 | 270 | 0.0ns | 0.0ns | 31 of 31 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is Aggregate, and the queries where it cost the most are q1 at 0.000us, q2 at 0.000us, q3 at 0.000us.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 999975 rows, one out of every 100 of the 99997497 in the full file, which is a development loop rather than the suite
- q5 swung by 50.5% of its median, and rule two wants under 10%

These swung wider than reporting rule two allows:

- duckdb swung by 50.5% of its median on q5, and rule two wants under 10%
- duckdb-pinned swung by 49.9% of its median on q22, and rule two wants under 10%
- clickhouse-local swung by 24.5% of its median on q38, and rule two wants under 10%
- datafusion swung by 49.7% of its median on q25, and rule two wants under 10%
- polars swung by 21.8% of its median on q41, and rule two wants under 10%
- rudb swung by 51.4% of its median on q17, and rule two wants under 10%

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

