# clickbench on gamingpc-wsl

This is one run of the clickbench suite on gamingpc-wsl, over 6 engines and 43 queries, with 5 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

It ran over 99998 rows, one out of every 1000 of the 99997497 in the full file, which is a development loop and not the suite. Nothing here is comparable to a full run or to anybody else's number.

## What ran

| what | it was |
| --- | --- |
| suite | clickbench |
| queries | 43 |
| tables | 22.85 MiB of Parquet in 1 table |
| rows | 99998 in the table every query reads |
| sample | 99998 rows, one out of every 1000 of the 99997497 in the full file |
| corpus | no manifest beside the data, so this run cannot say where it came from |
| summary | median with the interquartile range, per reporting rule two |
| timeout | 900s per query, and no query reached it |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --rows 100000 --engines duckdb,duckdb-pinned,clickhouse-local,datafusion,polars,rudb --runs 5 --timeout 900 --report
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
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 447.543ms | 780.000ms | 31.01 MiB | its own database file | its own | 2.00 to 1.84 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 551.375ms | 840.000ms | 28.76 MiB | its own database file | its own | 1.84 to 1.72 |
| clickhouse-local | 26.9.1.1562 | ran | 327.366ms | 320.000ms | 24.33 MiB | its own MergeTree parts, as system.parts counts the active ones | its own | 1.72 to 1.44 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 22.85 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 1.44 to 1.60 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 22.85 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 1.60 to 2.11 |
| rudb | rudb 0.3.67 | ran | 0.000us | 0.000us | 22.85 MiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 2.11 to 1.94 |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb on 39 shared |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 314.000ms | 1.072s | +242% | 1.053s | 720.000ms | 0.67 | 70.07 MiB | none | 13.69M/s | 3.06 GiB/s | 1.00x |
| duckdb-pinned | 338.000ms | 1.816s | +437% | 1.797s | 1.180s | 0.65 | 81.55 MiB | none | 12.72M/s | 2.84 GiB/s | 1.10x |
| clickhouse-local | 693.000ms | 3.646s | +426% | 3.486s | 3.100s | 0.85 | 291.98 MiB | none | 6.20M/s | 1.38 GiB/s | 2.62x |
| datafusion | 375.000ms | 1.375s | +267% | 1.438s | 2.440s | 1.77 | 407.48 MiB | none | 11.47M/s | 2.56 GiB/s | 1.40x |
| polars | 726.690ms | 4.383s | +503% | 4.492s | 6.710s | 1.53 | 130.59 MiB | none | 5.37M/s | 1.20 GiB/s | 3.00x |
| rudb | 176.633ms | 873.662ms | +395% | 877.947ms | 450.000ms | 0.52 | 92.64 MiB | none | 24.34M/s | 5.43 GiB/s | 0.64x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 1.000ms | 5.000ms | 1.000ms | 6.440ms | 1.694ms |
| q2 | filtered count | 1.000ms | 1.000ms | 5.000ms | 6.000ms | 6.294ms | 1.949ms |
| q3 | three aggregates | 1.000ms | 1.000ms | 6.000ms | 5.000ms | 6.536ms | 2.101ms |
| q4 | average | 1.000ms | 1.000ms | 6.000ms | 4.000ms | 6.725ms | 1.933ms |
| q5 | count distinct, high card | 5.000ms | 4.000ms | 10.000ms | 8.000ms | 14.701ms | 2.157ms |
| q6 | count distinct, strings | 5.000ms | 3.000ms | 8.000ms | 9.000ms | 14.698ms | 4.053ms |
| q7 | min and max of a date | 0.000us | 1.000ms | 10.000ms | 1.000ms | 6.367ms | 2.082ms |
| q8 | group by, low card | 1.000ms | 6.000ms | 11.000ms | 6.000ms | 16.099ms | 2.075ms |
| q9 | group by and count distinct | 7.000ms | 6.000ms | 10.000ms | 10.000ms | 26.070ms | 2.897ms |
| q10 | group by, several aggregates | 10.000ms | 9.000ms | 11.000ms | 10.000ms | 29.817ms | 3.512ms |
| q11 | group by a string and count distinct | 4.000ms | 5.000ms | 8.000ms | 10.000ms | 24.671ms | 2.790ms |
| q12 | group by two strings and count distinct | 5.000ms | 5.000ms | 8.000ms | 10.000ms | 25.249ms | 2.769ms |
| q13 | group by a string and top k | 4.000ms | 4.000ms | 10.000ms | 10.000ms | 19.250ms | 4.127ms |
| q14 | group by a string and count distinct | 6.000ms | 7.000ms | 11.000ms | 13.000ms | 25.690ms | 4.786ms |
| q15 | group by two columns and top k | 4.000ms | 4.000ms | 10.000ms | 11.000ms | 20.227ms | 4.073ms |
| q16 | group by, very high card | 6.000ms | 5.000ms | 10.000ms | 8.000ms | 21.164ms | 4.432ms |
| q17 | group by two, very high card | 11.000ms | 10.000ms | 18.000ms | 10.000ms | 26.738ms | 5.106ms |
| q18 | group by two, no ordering | 12.000ms | 11.000ms | 10.000ms | 11.000ms | 18.421ms | 3.227ms |
| q19 | group by with an extract | 12.000ms | 11.000ms | 17.000ms | 12.000ms | 27.853ms | 6.016ms |
| q20 | point lookup | 1.000ms | 1.000ms | 11.000ms | 5.000ms | 6.928ms | 1.807ms |
| q21 | substring scan | 8.000ms | 9.000ms | 13.000ms | 6.000ms | 13.314ms | 5.235ms |
| q22 | substring scan and group by | 9.000ms | 10.000ms | 15.000ms | 10.000ms | 21.523ms | 5.145ms |
| q23 | two substring scans and group by | 11.000ms | 15.000ms | 17.000ms | 12.000ms | 31.862ms | 8.779ms |
| q24 | select star and top k | 27.000ms | 29.000ms | 139.000ms | 18.000ms | 48.586ms | 9.248ms |
| q25 | top k by a date | 4.000ms | 3.000ms | 12.000ms | 7.000ms | 11.489ms | 2.471ms |
| q26 | top k by a string | 2.000ms | 2.000ms | 7.000ms | 6.000ms | 11.153ms | 2.842ms |
| q27 | top k by two columns | 3.000ms | 3.000ms | 12.000ms | 7.000ms | 13.737ms | 2.870ms |
| q28 | group by with a string length | 10.000ms | 11.000ms | 7.000ms | 9.000ms | no dialect | 5.059ms |
| q29 | group by a regular expression | 51.000ms | 54.000ms | 31.000ms | 13.000ms | no dialect | 9.146ms |
| q30 | ninety sums over one column | 4.000ms | 14.000ms | 10.000ms | 13.000ms | 12.799ms | 3.283ms |
| q31 | group by two and several aggregates | 5.000ms | 5.000ms | 9.000ms | 9.000ms | 22.078ms | 3.943ms |
| q32 | group by a high card pair | 5.000ms | 5.000ms | 9.000ms | 9.000ms | 17.905ms | 3.970ms |
| q33 | group by a high card pair, unfiltered | 9.000ms | 9.000ms | 15.000ms | 10.000ms | 20.957ms | 3.704ms |
| q34 | group by a long string | 19.000ms | 17.000ms | 25.000ms | 12.000ms | 24.908ms | 9.555ms |
| q35 | group by a constant and a long string | 20.000ms | 19.000ms | 25.000ms | 12.000ms | 27.516ms | 8.731ms |
| q36 | group by four expressions | 8.000ms | 5.000ms | 10.000ms | 8.000ms | no dialect | 5.161ms |
| q37 | date range and group by a URL | 3.000ms | 4.000ms | 15.000ms | 8.000ms | 17.723ms | 3.635ms |
| q38 | date range and group by a title | 3.000ms | 4.000ms | 14.000ms | 9.000ms | 18.811ms | 4.002ms |
| q39 | date range, group by and offset | 3.000ms | 5.000ms | 36.000ms | 7.000ms | 16.392ms | 3.682ms |
| q40 | date range, a case and a wide group by | 4.000ms | 6.000ms | 39.000ms | 9.000ms | 16.405ms | 5.445ms |
| q41 | date range with an IN and a hash | 3.000ms | 3.000ms | 13.000ms | 7.000ms | 14.608ms | 2.321ms |
| q42 | date range and a deep offset | 3.000ms | 7.000ms | 13.000ms | 7.000ms | 14.986ms | 2.267ms |
| q43 | minute buckets over a date range | 3.000ms | 3.000ms | 12.000ms | 7.000ms | no dialect | 2.552ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 20.258ms | 20.280ms | 0.1% | 20.253ms | 20.282ms | 20.251ms | 20.329ms | 10.000ms | 30.62 MiB | none | 4.93M/s |
| q2 | filtered count | 1.000ms | 20.240ms | 20.239ms | 0.0% | 20.232ms | 20.239ms | 20.226ms | 20.242ms | 0.000us | 31.56 MiB | none | 4.94M/s |
| q3 | three aggregates | 1.000ms | 20.292ms | 20.241ms | 0.0% | 20.238ms | 20.245ms | 20.231ms | 20.253ms | 0.000us | 32.12 MiB | none | 4.94M/s |
| q4 | average | 1.000ms | 20.251ms | 20.278ms | 0.0% | 20.273ms | 20.279ms | 20.250ms | 20.297ms | 0.000us | 32.19 MiB | none | 4.93M/s |
| q5 | count distinct, high card | 5.000ms | 20.289ms | 20.265ms | 0.1% | 20.257ms | 20.273ms | 20.257ms | 20.355ms | 10.000ms | 39.67 MiB | none | 4.93M/s |
| q6 | count distinct, strings | 5.000ms | 20.251ms | 20.247ms | 0.1% | 20.247ms | 20.258ms | 20.246ms | 20.264ms | 10.000ms | 37.55 MiB | none | 4.94M/s |
| q7 | min and max of a date | 0.000us | 20.267ms | 20.263ms | 0.0% | 20.261ms | 20.264ms | 20.253ms | 20.275ms | 0.000us | 31.00 MiB | none | 4.94M/s |
| q8 | group by, low card | 1.000ms | 20.314ms | 20.260ms | 0.1% | 20.242ms | 20.271ms | 20.238ms | 20.273ms | 10.000ms | 34.01 MiB | none | 4.94M/s |
| q9 | group by and count distinct | 7.000ms | 20.280ms | 20.270ms | 0.1% | 20.263ms | 20.279ms | 20.259ms | 20.284ms | 20.000ms | 50.14 MiB | none | 4.93M/s |
| q10 | group by, several aggregates | 10.000ms | 20.280ms | 20.267ms | 0.2% | 20.263ms | 20.309ms | 20.255ms | 40.350ms | 30.000ms | 52.39 MiB | none | 4.93M/s |
| q11 | group by a string and count distinct | 4.000ms | 20.255ms | 20.267ms | 0.0% | 20.263ms | 20.268ms | 20.256ms | 20.270ms | 20.000ms | 42.95 MiB | none | 4.93M/s |
| q12 | group by two strings and count distinct | 5.000ms | 20.254ms | 20.267ms | 0.1% | 20.259ms | 20.289ms | 20.256ms | 20.339ms | 10.000ms | 44.64 MiB | none | 4.93M/s |
| q13 | group by a string and top k | 4.000ms | 20.275ms | 20.254ms | 0.1% | 20.248ms | 20.264ms | 20.244ms | 20.279ms | 10.000ms | 39.20 MiB | none | 4.94M/s |
| q14 | group by a string and count distinct | 6.000ms | 20.256ms | 20.274ms | 0.1% | 20.266ms | 20.277ms | 20.258ms | 20.294ms | 20.000ms | 51.51 MiB | none | 4.93M/s |
| q15 | group by two columns and top k | 4.000ms | 20.269ms | 20.283ms | 0.0% | 20.278ms | 20.283ms | 20.274ms | 20.288ms | 10.000ms | 40.64 MiB | none | 4.93M/s |
| q16 | group by, very high card | 6.000ms | 20.286ms | 20.278ms | 0.1% | 20.266ms | 20.279ms | 20.261ms | 20.284ms | 20.000ms | 45.32 MiB | none | 4.93M/s |
| q17 | group by two, very high card | 11.000ms | 40.625ms | 40.385ms | 0.1% | 40.345ms | 40.397ms | 20.280ms | 40.425ms | 30.000ms | 52.63 MiB | none | 2.48M/s |
| q18 | group by two, no ordering | 12.000ms | 40.386ms | 40.372ms | 0.0% | 40.372ms | 40.390ms | 40.371ms | 40.499ms | 30.000ms | 61.13 MiB | none | 2.48M/s |
| q19 | group by with an extract | 12.000ms | 40.340ms | 40.378ms | 0.1% | 40.372ms | 40.406ms | 40.363ms | 40.417ms | 30.000ms | 54.94 MiB | none | 2.48M/s |
| q20 | point lookup | 1.000ms | 20.346ms | 20.254ms | 0.2% | 20.235ms | 20.281ms | 20.233ms | 20.306ms | 0.000us | 31.93 MiB | none | 4.94M/s |
| q21 | substring scan | 8.000ms | 20.293ms | 20.248ms | 0.1% | 20.241ms | 20.260ms | 20.234ms | 20.267ms | 10.000ms | 38.06 MiB | none | 4.94M/s |
| q22 | substring scan and group by | 9.000ms | 20.241ms | 20.247ms | 0.0% | 20.245ms | 20.248ms | 20.243ms | 20.257ms | 20.000ms | 42.14 MiB | none | 4.94M/s |
| q23 | two substring scans and group by | 11.000ms | 20.256ms | 40.348ms | 0.0% | 40.345ms | 40.362ms | 40.339ms | 40.413ms | 20.000ms | 49.51 MiB | none | 2.48M/s |
| q24 | select star and top k | 27.000ms | 40.329ms | 40.381ms | 0.1% | 40.343ms | 40.390ms | 40.339ms | 40.451ms | 40.000ms | 70.07 MiB | none | 2.48M/s |
| q25 | top k by a date | 4.000ms | 20.265ms | 20.264ms | 0.1% | 20.252ms | 20.275ms | 20.245ms | 20.289ms | 10.000ms | 35.87 MiB | none | 4.93M/s |
| q26 | top k by a string | 2.000ms | 20.244ms | 20.265ms | 0.1% | 20.247ms | 20.274ms | 20.244ms | 20.277ms | 0.000us | 32.55 MiB | none | 4.93M/s |
| q27 | top k by two columns | 3.000ms | 20.243ms | 20.259ms | 0.0% | 20.252ms | 20.260ms | 20.239ms | 20.271ms | 0.000us | 33.37 MiB | none | 4.94M/s |
| q28 | group by with a string length | 10.000ms | 20.260ms | 20.241ms | 0.0% | 20.238ms | 20.242ms | 20.234ms | 20.252ms | 20.000ms | 42.39 MiB | none | 4.94M/s |
| q29 | group by a regular expression | 51.000ms | 80.445ms | 80.590ms | 0.0% | 80.563ms | 80.595ms | 80.476ms | 80.599ms | 110.000ms | 52.75 MiB | none | 1.24M/s |
| q30 | ninety sums over one column | 4.000ms | 20.258ms | 20.241ms | 0.3% | 20.236ms | 20.286ms | 20.234ms | 20.364ms | 0.000us | 35.38 MiB | none | 4.94M/s |
| q31 | group by two and several aggregates | 5.000ms | 20.246ms | 20.252ms | 0.1% | 20.245ms | 20.258ms | 20.241ms | 20.259ms | 10.000ms | 41.89 MiB | none | 4.94M/s |
| q32 | group by a high card pair | 5.000ms | 20.258ms | 20.244ms | 0.0% | 20.243ms | 20.245ms | 20.239ms | 20.255ms | 10.000ms | 42.45 MiB | none | 4.94M/s |
| q33 | group by a high card pair, unfiltered | 9.000ms | 20.263ms | 20.259ms | 0.0% | 20.258ms | 20.263ms | 20.249ms | 20.266ms | 20.000ms | 53.95 MiB | none | 4.94M/s |
| q34 | group by a long string | 19.000ms | 40.324ms | 40.385ms | 0.1% | 40.361ms | 40.410ms | 40.348ms | 40.445ms | 40.000ms | 63.46 MiB | none | 2.48M/s |
| q35 | group by a constant and a long string | 20.000ms | 40.351ms | 40.386ms | 0.1% | 40.374ms | 40.417ms | 40.351ms | 40.439ms | 50.000ms | 65.76 MiB | none | 2.48M/s |
| q36 | group by four expressions | 8.000ms | 20.262ms | 20.261ms | 0.1% | 20.256ms | 20.272ms | 20.256ms | 20.275ms | 20.000ms | 48.50 MiB | none | 4.94M/s |
| q37 | date range and group by a URL | 3.000ms | 20.257ms | 20.282ms | 0.1% | 20.269ms | 20.292ms | 20.265ms | 20.297ms | 10.000ms | 37.45 MiB | none | 4.93M/s |
| q38 | date range and group by a title | 3.000ms | 20.313ms | 20.263ms | 0.3% | 20.242ms | 20.295ms | 20.240ms | 20.303ms | 10.000ms | 37.39 MiB | none | 4.93M/s |
| q39 | date range, group by and offset | 3.000ms | 20.236ms | 20.254ms | 0.1% | 20.252ms | 20.266ms | 20.243ms | 20.270ms | 10.000ms | 36.39 MiB | none | 4.94M/s |
| q40 | date range, a case and a wide group by | 4.000ms | 20.299ms | 20.291ms | 0.0% | 20.286ms | 20.294ms | 20.283ms | 20.312ms | 10.000ms | 39.88 MiB | none | 4.93M/s |
| q41 | date range with an IN and a hash | 3.000ms | 20.315ms | 20.288ms | 0.1% | 20.285ms | 20.297ms | 20.282ms | 20.624ms | 10.000ms | 39.32 MiB | none | 4.93M/s |
| q42 | date range and a deep offset | 3.000ms | 20.275ms | 20.266ms | 0.1% | 20.260ms | 20.270ms | 20.243ms | 20.271ms | 10.000ms | 38.26 MiB | none | 4.93M/s |
| q43 | minute buckets over a date range | 3.000ms | 20.262ms | 20.268ms | 0.0% | 20.267ms | 20.268ms | 20.261ms | 20.321ms | 10.000ms | 36.94 MiB | none | 4.93M/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 314.000ms by its own clock and 1.072s by ours, 1.053s cold, 720.000ms of CPU, peak 70.07 MiB, 13.69M/s and 3.06 GiB/s.

Running it cost 242% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.98x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 40.356ms | 40.370ms | 0.1% | 40.368ms | 40.425ms | 40.368ms | 40.509ms | 10.000ms | 39.59 MiB | none | 2.48M/s |
| q2 | filtered count | 1.000ms | 40.363ms | 40.378ms | 0.1% | 40.373ms | 40.397ms | 40.366ms | 40.461ms | 20.000ms | 40.07 MiB | none | 2.48M/s |
| q3 | three aggregates | 1.000ms | 40.385ms | 40.422ms | 0.2% | 40.403ms | 40.481ms | 40.350ms | 40.505ms | 20.000ms | 40.52 MiB | none | 2.47M/s |
| q4 | average | 1.000ms | 40.367ms | 40.384ms | 0.1% | 40.368ms | 40.390ms | 40.350ms | 40.407ms | 10.000ms | 41.08 MiB | none | 2.48M/s |
| q5 | count distinct, high card | 4.000ms | 40.369ms | 40.347ms | 0.0% | 40.346ms | 40.359ms | 40.346ms | 40.395ms | 30.000ms | 49.84 MiB | none | 2.48M/s |
| q6 | count distinct, strings | 3.000ms | 40.416ms | 40.403ms | 0.0% | 40.390ms | 40.405ms | 40.335ms | 40.421ms | 20.000ms | 44.33 MiB | none | 2.48M/s |
| q7 | min and max of a date | 1.000ms | 40.468ms | 40.378ms | 0.0% | 40.377ms | 40.397ms | 40.375ms | 40.454ms | 20.000ms | 40.02 MiB | none | 2.48M/s |
| q8 | group by, low card | 6.000ms | 40.359ms | 40.412ms | 0.1% | 40.387ms | 40.425ms | 40.369ms | 40.436ms | 30.000ms | 42.85 MiB | none | 2.47M/s |
| q9 | group by and count distinct | 6.000ms | 40.376ms | 40.376ms | 0.0% | 40.369ms | 40.386ms | 40.364ms | 40.398ms | 30.000ms | 57.61 MiB | none | 2.48M/s |
| q10 | group by, several aggregates | 9.000ms | 40.392ms | 40.370ms | 0.1% | 40.355ms | 40.378ms | 40.342ms | 40.384ms | 40.000ms | 63.11 MiB | none | 2.48M/s |
| q11 | group by a string and count distinct | 5.000ms | 40.344ms | 40.400ms | 0.1% | 40.372ms | 40.402ms | 40.344ms | 40.411ms | 30.000ms | 49.62 MiB | none | 2.48M/s |
| q12 | group by two strings and count distinct | 5.000ms | 40.482ms | 40.382ms | 0.1% | 40.372ms | 40.393ms | 40.371ms | 40.401ms | 30.000ms | 49.13 MiB | none | 2.48M/s |
| q13 | group by a string and top k | 4.000ms | 40.359ms | 40.375ms | 0.1% | 40.368ms | 40.405ms | 40.346ms | 40.691ms | 20.000ms | 45.39 MiB | none | 2.48M/s |
| q14 | group by a string and count distinct | 7.000ms | 40.400ms | 40.371ms | 0.1% | 40.368ms | 40.400ms | 40.356ms | 40.550ms | 30.000ms | 55.89 MiB | none | 2.48M/s |
| q15 | group by two columns and top k | 4.000ms | 40.380ms | 40.376ms | 0.1% | 40.363ms | 40.394ms | 40.338ms | 40.400ms | 20.000ms | 46.59 MiB | none | 2.48M/s |
| q16 | group by, very high card | 5.000ms | 40.376ms | 40.378ms | 0.0% | 40.374ms | 40.379ms | 40.369ms | 40.391ms | 30.000ms | 52.73 MiB | none | 2.48M/s |
| q17 | group by two, very high card | 10.000ms | 40.394ms | 40.362ms | 0.1% | 40.347ms | 40.369ms | 40.342ms | 40.415ms | 30.000ms | 62.36 MiB | none | 2.48M/s |
| q18 | group by two, no ordering | 11.000ms | 40.363ms | 40.347ms | 0.1% | 40.347ms | 40.371ms | 40.340ms | 40.381ms | 40.000ms | 69.13 MiB | none | 2.48M/s |
| q19 | group by with an extract | 11.000ms | 40.349ms | 40.352ms | 0.0% | 40.349ms | 40.359ms | 40.342ms | 40.468ms | 40.000ms | 63.86 MiB | none | 2.48M/s |
| q20 | point lookup | 1.000ms | 40.364ms | 40.412ms | 0.2% | 40.384ms | 40.446ms | 40.376ms | 40.477ms | 20.000ms | 39.83 MiB | none | 2.47M/s |
| q21 | substring scan | 9.000ms | 40.447ms | 40.371ms | 0.0% | 40.371ms | 40.387ms | 40.364ms | 40.408ms | 30.000ms | 46.57 MiB | none | 2.48M/s |
| q22 | substring scan and group by | 10.000ms | 40.353ms | 40.362ms | 0.0% | 40.360ms | 40.379ms | 40.345ms | 40.422ms | 20.000ms | 51.07 MiB | none | 2.48M/s |
| q23 | two substring scans and group by | 15.000ms | 40.378ms | 40.360ms | 0.1% | 40.349ms | 40.395ms | 40.330ms | 40.686ms | 40.000ms | 60.45 MiB | none | 2.48M/s |
| q24 | select star and top k | 29.000ms | 60.447ms | 60.477ms | 0.0% | 60.469ms | 60.482ms | 60.432ms | 60.486ms | 50.000ms | 81.55 MiB | none | 1.65M/s |
| q25 | top k by a date | 3.000ms | 40.369ms | 40.417ms | 0.1% | 40.373ms | 40.431ms | 40.371ms | 40.494ms | 20.000ms | 42.40 MiB | none | 2.47M/s |
| q26 | top k by a string | 2.000ms | 40.373ms | 40.370ms | 0.1% | 40.357ms | 40.404ms | 40.341ms | 40.465ms | 20.000ms | 41.83 MiB | none | 2.48M/s |
| q27 | top k by two columns | 3.000ms | 40.418ms | 40.397ms | 0.1% | 40.360ms | 40.415ms | 40.347ms | 40.702ms | 20.000ms | 42.83 MiB | none | 2.48M/s |
| q28 | group by with a string length | 11.000ms | 40.354ms | 40.350ms | 0.1% | 40.348ms | 40.384ms | 40.341ms | 40.401ms | 30.000ms | 51.08 MiB | none | 2.48M/s |
| q29 | group by a regular expression | 54.000ms | 80.471ms | 80.486ms | 0.0% | 80.485ms | 80.505ms | 80.472ms | 80.528ms | 70.000ms | 54.63 MiB | none | 1.24M/s |
| q30 | ninety sums over one column | 14.000ms | 40.349ms | 40.349ms | 0.0% | 40.339ms | 40.357ms | 40.332ms | 40.388ms | 30.000ms | 52.91 MiB | none | 2.48M/s |
| q31 | group by two and several aggregates | 5.000ms | 40.397ms | 40.375ms | 0.1% | 40.353ms | 40.391ms | 40.335ms | 40.403ms | 20.000ms | 48.84 MiB | none | 2.48M/s |
| q32 | group by a high card pair | 5.000ms | 40.349ms | 40.347ms | 0.1% | 40.343ms | 40.367ms | 40.337ms | 40.410ms | 20.000ms | 49.11 MiB | none | 2.48M/s |
| q33 | group by a high card pair, unfiltered | 9.000ms | 40.371ms | 40.380ms | 0.1% | 40.363ms | 40.388ms | 40.347ms | 40.524ms | 30.000ms | 64.09 MiB | none | 2.48M/s |
| q34 | group by a long string | 17.000ms | 40.674ms | 40.341ms | 0.0% | 40.338ms | 40.346ms | 40.332ms | 40.386ms | 50.000ms | 73.39 MiB | none | 2.48M/s |
| q35 | group by a constant and a long string | 19.000ms | 40.394ms | 60.452ms | 33.3% | 40.355ms | 60.459ms | 40.337ms | 60.492ms | 40.000ms | 73.39 MiB | none | 1.65M/s |
| q36 | group by four expressions | 5.000ms | 40.348ms | 40.352ms | 0.0% | 40.351ms | 40.356ms | 40.349ms | 40.380ms | 30.000ms | 52.16 MiB | none | 2.48M/s |
| q37 | date range and group by a URL | 4.000ms | 40.429ms | 40.393ms | 0.1% | 40.381ms | 40.407ms | 40.370ms | 40.473ms | 20.000ms | 46.32 MiB | none | 2.48M/s |
| q38 | date range and group by a title | 4.000ms | 40.568ms | 40.373ms | 0.1% | 40.370ms | 40.401ms | 40.349ms | 40.630ms | 20.000ms | 45.35 MiB | none | 2.48M/s |
| q39 | date range, group by and offset | 5.000ms | 40.345ms | 40.350ms | 0.0% | 40.348ms | 40.357ms | 40.344ms | 40.368ms | 20.000ms | 45.34 MiB | none | 2.48M/s |
| q40 | date range, a case and a wide group by | 6.000ms | 40.345ms | 40.421ms | 0.1% | 40.413ms | 40.458ms | 40.385ms | 40.715ms | 20.000ms | 49.39 MiB | none | 2.47M/s |
| q41 | date range with an IN and a hash | 3.000ms | 40.452ms | 40.361ms | 0.0% | 40.361ms | 40.374ms | 40.346ms | 40.435ms | 20.000ms | 45.84 MiB | none | 2.48M/s |
| q42 | date range and a deep offset | 7.000ms | 40.374ms | 40.386ms | 0.1% | 40.371ms | 40.407ms | 40.359ms | 40.735ms | 20.000ms | 45.58 MiB | none | 2.48M/s |
| q43 | minute buckets over a date range | 3.000ms | 40.349ms | 40.389ms | 0.0% | 40.383ms | 40.400ms | 40.337ms | 40.411ms | 20.000ms | 43.95 MiB | none | 2.48M/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 338.000ms by its own clock and 1.816s by ours, 1.797s cold, 1.180s of CPU, peak 81.55 MiB, 12.72M/s and 2.84 GiB/s.

Running it cost 437% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.00x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 5.000ms | 60.431ms | 60.410ms | 0.0% | 60.392ms | 60.412ms | 60.391ms | 60.458ms | 50.000ms | 238.74 MiB | none | 1.66M/s |
| q2 | filtered count | 5.000ms | 60.413ms | 60.435ms | 0.0% | 60.420ms | 60.435ms | 60.402ms | 60.466ms | 60.000ms | 239.71 MiB | none | 1.65M/s |
| q3 | three aggregates | 6.000ms | 60.390ms | 60.443ms | 33.2% | 60.419ms | 80.507ms | 60.397ms | 80.532ms | 60.000ms | 243.58 MiB | none | 1.65M/s |
| q4 | average | 6.000ms | 60.440ms | 80.539ms | 25.0% | 60.428ms | 80.544ms | 60.402ms | 80.896ms | 60.000ms | 241.61 MiB | none | 1.24M/s |
| q5 | count distinct, high card | 10.000ms | 80.505ms | 80.537ms | 0.1% | 80.523ms | 80.574ms | 80.492ms | 80.660ms | 60.000ms | 247.42 MiB | none | 1.24M/s |
| q6 | count distinct, strings | 8.000ms | 60.435ms | 80.509ms | 24.9% | 60.444ms | 80.515ms | 60.400ms | 80.535ms | 60.000ms | 244.87 MiB | none | 1.24M/s |
| q7 | min and max of a date | 10.000ms | 80.557ms | 80.529ms | 0.0% | 80.514ms | 80.547ms | 80.497ms | 80.550ms | 60.000ms | 241.49 MiB | none | 1.24M/s |
| q8 | group by, low card | 11.000ms | 80.540ms | 80.510ms | 0.0% | 80.500ms | 80.523ms | 80.490ms | 80.553ms | 60.000ms | 244.61 MiB | none | 1.24M/s |
| q9 | group by and count distinct | 10.000ms | 60.404ms | 80.515ms | 0.1% | 80.487ms | 80.546ms | 80.468ms | 80.561ms | 70.000ms | 247.99 MiB | none | 1.24M/s |
| q10 | group by, several aggregates | 11.000ms | 80.528ms | 80.538ms | 0.0% | 80.538ms | 80.539ms | 80.495ms | 80.564ms | 70.000ms | 250.36 MiB | none | 1.24M/s |
| q11 | group by a string and count distinct | 8.000ms | 80.468ms | 80.464ms | 24.9% | 60.466ms | 80.510ms | 60.424ms | 80.532ms | 60.000ms | 245.77 MiB | none | 1.24M/s |
| q12 | group by two strings and count distinct | 8.000ms | 60.450ms | 80.489ms | 24.8% | 60.506ms | 80.491ms | 60.382ms | 80.548ms | 60.000ms | 247.51 MiB | none | 1.24M/s |
| q13 | group by a string and top k | 10.000ms | 80.562ms | 80.599ms | 0.1% | 80.529ms | 80.642ms | 80.506ms | 80.706ms | 60.000ms | 249.76 MiB | none | 1.24M/s |
| q14 | group by a string and count distinct | 11.000ms | 80.499ms | 80.653ms | 0.2% | 80.641ms | 80.774ms | 80.514ms | 80.834ms | 70.000ms | 252.70 MiB | none | 1.24M/s |
| q15 | group by two columns and top k | 10.000ms | 80.507ms | 80.522ms | 0.0% | 80.521ms | 80.539ms | 80.515ms | 80.647ms | 70.000ms | 251.88 MiB | none | 1.24M/s |
| q16 | group by, very high card | 10.000ms | 60.417ms | 80.498ms | 0.1% | 80.493ms | 80.585ms | 80.465ms | 80.627ms | 60.000ms | 249.79 MiB | none | 1.24M/s |
| q17 | group by two, very high card | 18.000ms | 80.524ms | 80.515ms | 0.1% | 80.478ms | 80.543ms | 80.470ms | 80.552ms | 80.000ms | 262.04 MiB | none | 1.24M/s |
| q18 | group by two, no ordering | 10.000ms | 80.529ms | 80.735ms | 0.4% | 80.506ms | 80.825ms | 80.486ms | 81.047ms | 70.000ms | 250.90 MiB | none | 1.24M/s |
| q19 | group by with an extract | 17.000ms | 81.231ms | 80.479ms | 0.2% | 80.470ms | 80.653ms | 80.451ms | 80.670ms | 80.000ms | 262.84 MiB | none | 1.24M/s |
| q20 | point lookup | 11.000ms | 80.553ms | 80.925ms | 0.4% | 80.592ms | 80.937ms | 80.505ms | 86.186ms | 70.000ms | 241.93 MiB | none | 1.24M/s |
| q21 | substring scan | 13.000ms | 80.668ms | 80.604ms | 0.2% | 80.463ms | 80.613ms | 80.445ms | 81.064ms | 70.000ms | 247.60 MiB | none | 1.24M/s |
| q22 | substring scan and group by | 15.000ms | 80.442ms | 80.585ms | 0.1% | 80.466ms | 80.585ms | 80.428ms | 80.853ms | 80.000ms | 251.64 MiB | none | 1.24M/s |
| q23 | two substring scans and group by | 17.000ms | 80.501ms | 80.470ms | 0.2% | 80.468ms | 80.651ms | 80.466ms | 80.653ms | 80.000ms | 255.34 MiB | none | 1.24M/s |
| q24 | select star and top k | 139.000ms | 201.126ms | 221.042ms | 7.4% | 204.861ms | 221.321ms | 202.083ms | 221.628ms | 160.000ms | 291.98 MiB | none | 452.39K/s |
| q25 | top k by a date | 12.000ms | 80.711ms | 80.669ms | 0.4% | 80.542ms | 80.840ms | 80.459ms | 81.206ms | 70.000ms | 246.92 MiB | none | 1.24M/s |
| q26 | top k by a string | 7.000ms | 60.409ms | 80.514ms | 0.1% | 80.445ms | 80.523ms | 60.387ms | 80.671ms | 60.000ms | 244.61 MiB | none | 1.24M/s |
| q27 | top k by two columns | 12.000ms | 80.509ms | 80.618ms | 0.2% | 80.581ms | 80.724ms | 80.546ms | 81.198ms | 70.000ms | 246.41 MiB | none | 1.24M/s |
| q28 | group by with a string length | 7.000ms | 80.447ms | 80.494ms | 0.2% | 80.471ms | 80.602ms | 80.457ms | 81.260ms | 70.000ms | 246.25 MiB | none | 1.24M/s |
| q29 | group by a regular expression | 31.000ms | 100.630ms | 100.577ms | 0.1% | 100.531ms | 100.666ms | 100.531ms | 100.992ms | 90.000ms | 278.30 MiB | none | 994.24K/s |
| q30 | ninety sums over one column | 10.000ms | 80.436ms | 80.567ms | 0.1% | 80.529ms | 80.588ms | 80.466ms | 80.786ms | 70.000ms | 245.00 MiB | none | 1.24M/s |
| q31 | group by two and several aggregates | 9.000ms | 80.786ms | 80.642ms | 0.3% | 80.457ms | 80.693ms | 80.441ms | 81.018ms | 80.000ms | 248.97 MiB | none | 1.24M/s |
| q32 | group by a high card pair | 9.000ms | 80.598ms | 80.754ms | 0.1% | 80.634ms | 80.755ms | 80.518ms | 80.944ms | 70.000ms | 250.51 MiB | none | 1.24M/s |
| q33 | group by a high card pair, unfiltered | 15.000ms | 80.459ms | 80.739ms | 0.3% | 80.591ms | 80.867ms | 80.587ms | 80.893ms | 80.000ms | 260.26 MiB | none | 1.24M/s |
| q34 | group by a long string | 25.000ms | 100.803ms | 100.609ms | 0.1% | 100.569ms | 100.621ms | 80.646ms | 100.847ms | 90.000ms | 276.35 MiB | none | 993.93K/s |
| q35 | group by a constant and a long string | 25.000ms | 100.535ms | 100.582ms | 0.1% | 100.503ms | 100.608ms | 80.488ms | 100.842ms | 90.000ms | 275.81 MiB | none | 994.19K/s |
| q36 | group by four expressions | 10.000ms | 60.388ms | 80.588ms | 0.5% | 80.568ms | 80.970ms | 80.515ms | 81.285ms | 70.000ms | 250.05 MiB | none | 1.24M/s |
| q37 | date range and group by a URL | 15.000ms | 81.427ms | 80.568ms | 0.3% | 80.492ms | 80.722ms | 80.452ms | 83.262ms | 70.000ms | 251.03 MiB | none | 1.24M/s |
| q38 | date range and group by a title | 14.000ms | 80.448ms | 80.654ms | 2.8% | 80.555ms | 82.821ms | 80.457ms | 100.546ms | 70.000ms | 250.39 MiB | none | 1.24M/s |
| q39 | date range, group by and offset | 36.000ms | 100.658ms | 100.849ms | 0.5% | 100.558ms | 101.018ms | 100.542ms | 102.388ms | 70.000ms | 252.30 MiB | none | 991.57K/s |
| q40 | date range, a case and a wide group by | 39.000ms | 100.711ms | 100.774ms | 0.5% | 100.745ms | 101.243ms | 100.700ms | 123.992ms | 80.000ms | 255.00 MiB | none | 992.30K/s |
| q41 | date range with an IN and a hash | 13.000ms | 81.177ms | 80.596ms | 0.3% | 80.508ms | 80.719ms | 80.453ms | 80.825ms | 70.000ms | 248.56 MiB | none | 1.24M/s |
| q42 | date range and a deep offset | 13.000ms | 80.752ms | 80.529ms | 0.3% | 80.516ms | 80.780ms | 80.457ms | 81.310ms | 80.000ms | 248.39 MiB | none | 1.24M/s |
| q43 | minute buckets over a date range | 12.000ms | 80.600ms | 80.724ms | 0.3% | 80.601ms | 80.837ms | 80.522ms | 80.843ms | 70.000ms | 247.48 MiB | none | 1.24M/s |

clickhouse-local 26.9.1.1562 over 43 of 43 queries. Total 693.000ms by its own clock and 3.646s by ours, 3.486s cold, 3.100s of CPU, peak 291.98 MiB, 6.20M/s and 1.38 GiB/s.

Running it cost 426% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.66x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 20.230ms | 20.231ms | 0.0% | 20.226ms | 20.232ms | 20.220ms | 20.242ms | 10.000ms | 82.43 MiB | none | 4.94M/s |
| q2 | filtered count | 6.000ms | 40.397ms | 20.261ms | 0.2% | 20.251ms | 20.282ms | 20.250ms | 40.325ms | 50.000ms | 143.81 MiB | none | 4.94M/s |
| q3 | three aggregates | 5.000ms | 20.253ms | 20.255ms | 0.7% | 20.251ms | 20.390ms | 20.249ms | 20.398ms | 30.000ms | 147.03 MiB | none | 4.94M/s |
| q4 | average | 4.000ms | 20.391ms | 20.248ms | 0.1% | 20.246ms | 20.262ms | 20.244ms | 20.427ms | 20.000ms | 122.20 MiB | none | 4.94M/s |
| q5 | count distinct, high card | 8.000ms | 40.378ms | 40.329ms | 49.8% | 20.270ms | 40.336ms | 20.262ms | 40.342ms | 50.000ms | 222.52 MiB | none | 2.48M/s |
| q6 | count distinct, strings | 9.000ms | 40.334ms | 40.336ms | 0.1% | 40.335ms | 40.366ms | 40.324ms | 40.477ms | 40.000ms | 248.29 MiB | none | 2.48M/s |
| q7 | min and max of a date | 1.000ms | 20.251ms | 20.304ms | 0.6% | 20.229ms | 20.358ms | 20.220ms | 20.360ms | 0.000us | 82.34 MiB | none | 4.93M/s |
| q8 | group by, low card | 6.000ms | 20.368ms | 20.269ms | 0.1% | 20.255ms | 20.283ms | 20.248ms | 20.448ms | 30.000ms | 137.68 MiB | none | 4.93M/s |
| q9 | group by and count distinct | 10.000ms | 40.330ms | 40.346ms | 0.3% | 40.341ms | 40.471ms | 40.339ms | 40.475ms | 60.000ms | 261.35 MiB | none | 2.48M/s |
| q10 | group by, several aggregates | 10.000ms | 40.580ms | 40.477ms | 0.4% | 40.343ms | 40.496ms | 40.336ms | 40.589ms | 80.000ms | 250.70 MiB | none | 2.47M/s |
| q11 | group by a string and count distinct | 10.000ms | 40.335ms | 40.393ms | 0.3% | 40.346ms | 40.484ms | 40.341ms | 40.487ms | 80.000ms | 208.59 MiB | none | 2.48M/s |
| q12 | group by two strings and count distinct | 10.000ms | 40.339ms | 40.346ms | 0.4% | 40.342ms | 40.487ms | 40.341ms | 40.511ms | 70.000ms | 208.85 MiB | none | 2.48M/s |
| q13 | group by a string and top k | 10.000ms | 40.487ms | 40.472ms | 0.3% | 40.378ms | 40.498ms | 40.344ms | 40.560ms | 90.000ms | 250.80 MiB | none | 2.47M/s |
| q14 | group by a string and count distinct | 13.000ms | 40.770ms | 40.370ms | 0.4% | 40.363ms | 40.504ms | 40.360ms | 40.526ms | 110.000ms | 286.41 MiB | none | 2.48M/s |
| q15 | group by two columns and top k | 11.000ms | 40.796ms | 40.439ms | 0.3% | 40.390ms | 40.498ms | 40.337ms | 40.506ms | 50.000ms | 238.83 MiB | none | 2.47M/s |
| q16 | group by, very high card | 8.000ms | 20.402ms | 20.270ms | 1.0% | 20.261ms | 20.455ms | 20.259ms | 40.390ms | 60.000ms | 228.52 MiB | none | 4.93M/s |
| q17 | group by two, very high card | 10.000ms | 40.340ms | 40.355ms | 0.3% | 40.351ms | 40.470ms | 40.341ms | 40.601ms | 70.000ms | 311.04 MiB | none | 2.48M/s |
| q18 | group by two, no ordering | 11.000ms | 40.403ms | 40.355ms | 0.1% | 40.348ms | 40.371ms | 40.343ms | 41.280ms | 70.000ms | 303.39 MiB | none | 2.48M/s |
| q19 | group by with an extract | 12.000ms | 40.891ms | 40.355ms | 0.0% | 40.340ms | 40.360ms | 40.330ms | 40.512ms | 100.000ms | 311.94 MiB | none | 2.48M/s |
| q20 | point lookup | 5.000ms | 20.254ms | 20.251ms | 0.1% | 20.241ms | 20.252ms | 20.236ms | 20.518ms | 20.000ms | 124.45 MiB | none | 4.94M/s |
| q21 | substring scan | 6.000ms | 40.493ms | 20.255ms | 0.1% | 20.254ms | 20.273ms | 20.246ms | 20.370ms | 30.000ms | 163.80 MiB | none | 4.94M/s |
| q22 | substring scan and group by | 10.000ms | 40.338ms | 40.468ms | 0.3% | 40.331ms | 40.472ms | 40.330ms | 40.475ms | 50.000ms | 205.32 MiB | none | 2.47M/s |
| q23 | two substring scans and group by | 12.000ms | 40.379ms | 40.351ms | 0.0% | 40.344ms | 40.354ms | 40.341ms | 40.357ms | 80.000ms | 224.32 MiB | none | 2.48M/s |
| q24 | select star and top k | 18.000ms | 40.485ms | 40.482ms | 0.2% | 40.435ms | 40.533ms | 40.397ms | 40.992ms | 150.000ms | 407.48 MiB | none | 2.47M/s |
| q25 | top k by a date | 7.000ms | 20.420ms | 20.263ms | 0.7% | 20.262ms | 20.410ms | 20.257ms | 20.414ms | 50.000ms | 196.86 MiB | none | 4.93M/s |
| q26 | top k by a string | 6.000ms | 20.271ms | 20.267ms | 0.0% | 20.260ms | 20.271ms | 20.258ms | 20.305ms | 40.000ms | 184.52 MiB | none | 4.93M/s |
| q27 | top k by two columns | 7.000ms | 20.270ms | 20.279ms | 0.1% | 20.260ms | 20.280ms | 20.260ms | 20.399ms | 50.000ms | 182.90 MiB | none | 4.93M/s |
| q28 | group by with a string length | 9.000ms | 40.480ms | 40.345ms | 0.0% | 40.341ms | 40.351ms | 20.286ms | 40.595ms | 70.000ms | 244.70 MiB | none | 2.48M/s |
| q29 | group by a regular expression | 13.000ms | 40.553ms | 40.459ms | 0.4% | 40.411ms | 40.569ms | 40.391ms | 40.602ms | 130.000ms | 345.61 MiB | none | 2.47M/s |
| q30 | ninety sums over one column | 13.000ms | 40.370ms | 40.417ms | 0.2% | 40.405ms | 40.493ms | 40.366ms | 40.649ms | 50.000ms | 143.62 MiB | none | 2.47M/s |
| q31 | group by two and several aggregates | 9.000ms | 40.333ms | 40.347ms | 0.1% | 40.343ms | 40.381ms | 40.341ms | 40.484ms | 70.000ms | 211.91 MiB | none | 2.48M/s |
| q32 | group by a high card pair | 9.000ms | 40.334ms | 40.354ms | 0.2% | 40.348ms | 40.415ms | 40.332ms | 40.553ms | 60.000ms | 205.15 MiB | none | 2.48M/s |
| q33 | group by a high card pair, unfiltered | 10.000ms | 40.368ms | 40.341ms | 0.0% | 40.339ms | 40.347ms | 40.335ms | 40.369ms | 70.000ms | 260.43 MiB | none | 2.48M/s |
| q34 | group by a long string | 12.000ms | 40.367ms | 40.466ms | 0.6% | 40.360ms | 40.600ms | 40.354ms | 40.642ms | 100.000ms | 333.32 MiB | none | 2.47M/s |
| q35 | group by a constant and a long string | 12.000ms | 41.036ms | 40.505ms | 0.5% | 40.411ms | 40.631ms | 40.353ms | 40.761ms | 100.000ms | 350.83 MiB | none | 2.47M/s |
| q36 | group by four expressions | 8.000ms | 40.344ms | 20.258ms | 0.0% | 20.257ms | 20.259ms | 20.256ms | 40.345ms | 40.000ms | 244.40 MiB | none | 4.94M/s |
| q37 | date range and group by a URL | 8.000ms | 20.396ms | 20.278ms | 0.6% | 20.259ms | 20.386ms | 20.257ms | 20.409ms | 40.000ms | 165.59 MiB | none | 4.93M/s |
| q38 | date range and group by a title | 9.000ms | 40.328ms | 40.339ms | 49.8% | 20.265ms | 40.348ms | 20.262ms | 40.784ms | 40.000ms | 173.61 MiB | none | 2.48M/s |
| q39 | date range, group by and offset | 7.000ms | 20.259ms | 20.396ms | 0.9% | 20.258ms | 20.438ms | 20.256ms | 20.468ms | 30.000ms | 141.91 MiB | none | 4.90M/s |
| q40 | date range, a case and a wide group by | 9.000ms | 40.375ms | 40.329ms | 49.8% | 20.263ms | 40.330ms | 20.263ms | 41.689ms | 30.000ms | 146.43 MiB | none | 2.48M/s |
| q41 | date range with an IN and a hash | 7.000ms | 20.256ms | 20.269ms | 0.1% | 20.260ms | 20.285ms | 20.252ms | 40.333ms | 30.000ms | 136.39 MiB | none | 4.93M/s |
| q42 | date range and a deep offset | 7.000ms | 20.251ms | 20.395ms | 0.1% | 20.378ms | 20.395ms | 20.251ms | 20.455ms | 20.000ms | 134.15 MiB | none | 4.90M/s |
| q43 | minute buckets over a date range | 7.000ms | 20.389ms | 20.257ms | 0.2% | 20.256ms | 20.286ms | 20.253ms | 20.317ms | 20.000ms | 129.91 MiB | none | 4.94M/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 375.000ms by its own clock and 1.375s by ours, 1.438s cold, 2.440s of CPU, peak 407.48 MiB, 11.47M/s and 2.56 GiB/s.

Running it cost 267% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.00x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 6.440ms | 122.583ms | 100.600ms | 0.1% | 100.562ms | 100.649ms | 100.553ms | 100.748ms | 140.000ms | 63.29 MiB | none | 994.01K/s |
| q2 | filtered count | 6.294ms | 100.540ms | 100.601ms | 0.1% | 100.555ms | 100.608ms | 100.507ms | 100.609ms | 200.000ms | 66.30 MiB | none | 994.01K/s |
| q3 | three aggregates | 6.536ms | 100.607ms | 100.683ms | 0.1% | 100.618ms | 100.760ms | 100.616ms | 121.372ms | 130.000ms | 67.56 MiB | none | 993.20K/s |
| q4 | average | 6.725ms | 100.820ms | 100.844ms | 0.3% | 100.645ms | 100.982ms | 100.602ms | 101.009ms | 120.000ms | 65.09 MiB | none | 991.61K/s |
| q5 | count distinct, high card | 14.701ms | 100.672ms | 100.817ms | 0.3% | 100.776ms | 101.040ms | 100.659ms | 129.956ms | 130.000ms | 79.93 MiB | none | 991.87K/s |
| q6 | count distinct, strings | 14.698ms | 100.720ms | 100.694ms | 20.1% | 100.552ms | 120.788ms | 100.498ms | 121.448ms | 150.000ms | 80.74 MiB | none | 993.09K/s |
| q7 | min and max of a date | 6.367ms | 100.649ms | 100.639ms | 0.2% | 100.618ms | 100.789ms | 100.503ms | 100.809ms | 150.000ms | 66.34 MiB | none | 993.63K/s |
| q8 | group by, low card | 16.099ms | 121.237ms | 100.762ms | 19.9% | 100.681ms | 120.751ms | 100.679ms | 130.762ms | 150.000ms | 76.22 MiB | none | 992.42K/s |
| q9 | group by and count distinct | 26.070ms | 121.910ms | 120.798ms | 0.1% | 120.796ms | 120.902ms | 120.624ms | 121.473ms | 170.000ms | 98.38 MiB | none | 827.81K/s |
| q10 | group by, several aggregates | 29.817ms | 120.686ms | 121.162ms | 0.2% | 121.023ms | 121.260ms | 120.857ms | 121.287ms | 170.000ms | 102.43 MiB | none | 825.32K/s |
| q11 | group by a string and count distinct | 24.671ms | 121.443ms | 120.913ms | 0.3% | 120.706ms | 121.042ms | 120.659ms | 146.884ms | 190.000ms | 85.08 MiB | none | 827.02K/s |
| q12 | group by two strings and count distinct | 25.249ms | 120.641ms | 121.048ms | 0.3% | 120.686ms | 121.056ms | 120.598ms | 121.648ms | 170.000ms | 86.92 MiB | none | 826.10K/s |
| q13 | group by a string and top k | 19.250ms | 120.917ms | 120.935ms | 0.2% | 120.906ms | 121.138ms | 120.803ms | 122.047ms | 140.000ms | 84.07 MiB | none | 826.87K/s |
| q14 | group by a string and count distinct | 25.690ms | 120.765ms | 120.839ms | 0.2% | 120.696ms | 120.949ms | 120.647ms | 121.201ms | 180.000ms | 96.84 MiB | none | 827.53K/s |
| q15 | group by two columns and top k | 20.227ms | 121.958ms | 124.918ms | 7.5% | 122.737ms | 132.091ms | 120.625ms | 134.874ms | 250.000ms | 85.36 MiB | none | 800.51K/s |
| q16 | group by, very high card | 21.164ms | 120.650ms | 121.025ms | 0.8% | 120.858ms | 121.843ms | 120.723ms | 122.116ms | 160.000ms | 85.41 MiB | none | 826.26K/s |
| q17 | group by two, very high card | 26.738ms | 120.776ms | 120.943ms | 0.4% | 120.695ms | 121.187ms | 120.647ms | 121.538ms | 170.000ms | 100.77 MiB | none | 826.82K/s |
| q18 | group by two, no ordering | 18.421ms | 120.787ms | 121.110ms | 0.4% | 120.989ms | 121.422ms | 101.793ms | 122.169ms | 140.000ms | 97.66 MiB | none | 825.68K/s |
| q19 | group by with an extract | 27.853ms | 121.575ms | 120.888ms | 0.2% | 120.675ms | 120.954ms | 120.623ms | 145.189ms | 180.000ms | 104.93 MiB | none | 827.19K/s |
| q20 | point lookup | 6.928ms | 100.694ms | 100.682ms | 0.1% | 100.630ms | 100.696ms | 100.617ms | 121.224ms | 240.000ms | 66.64 MiB | none | 993.21K/s |
| q21 | substring scan | 13.314ms | 100.615ms | 100.607ms | 0.1% | 100.582ms | 100.721ms | 100.574ms | 100.795ms | 150.000ms | 80.35 MiB | none | 993.95K/s |
| q22 | substring scan and group by | 21.523ms | 122.705ms | 121.108ms | 0.7% | 120.950ms | 121.803ms | 120.908ms | 121.984ms | 150.000ms | 88.65 MiB | none | 825.69K/s |
| q23 | two substring scans and group by | 31.862ms | 148.829ms | 122.022ms | 3.9% | 121.950ms | 126.657ms | 121.699ms | 129.534ms | 190.000ms | 118.13 MiB | none | 819.51K/s |
| q24 | select star and top k | 48.586ms | 149.038ms | 144.308ms | 2.4% | 141.898ms | 145.395ms | 141.185ms | 145.444ms | 350.000ms | 121.99 MiB | none | 692.95K/s |
| q25 | top k by a date | 11.489ms | 100.549ms | 100.589ms | 0.1% | 100.527ms | 100.655ms | 100.525ms | 100.743ms | 190.000ms | 73.96 MiB | none | 994.13K/s |
| q26 | top k by a string | 11.153ms | 100.541ms | 100.579ms | 0.1% | 100.519ms | 100.586ms | 100.509ms | 100.696ms | 240.000ms | 72.67 MiB | none | 994.23K/s |
| q27 | top k by two columns | 13.737ms | 129.606ms | 100.608ms | 0.1% | 100.534ms | 100.623ms | 100.504ms | 100.732ms | 190.000ms | 75.57 MiB | none | 993.94K/s |
| q30 | ninety sums over one column | 12.799ms | 100.702ms | 100.691ms | 0.1% | 100.689ms | 100.764ms | 100.634ms | 100.841ms | 150.000ms | 69.40 MiB | none | 993.12K/s |
| q31 | group by two and several aggregates | 22.078ms | 121.837ms | 121.451ms | 0.7% | 121.184ms | 122.093ms | 121.014ms | 137.850ms | 150.000ms | 85.70 MiB | none | 823.36K/s |
| q32 | group by a high card pair | 17.905ms | 121.351ms | 120.869ms | 0.2% | 120.817ms | 121.070ms | 100.716ms | 121.648ms | 130.000ms | 85.99 MiB | none | 827.33K/s |
| q33 | group by a high card pair, unfiltered | 20.957ms | 121.972ms | 122.619ms | 0.7% | 122.264ms | 123.120ms | 120.992ms | 132.187ms | 150.000ms | 105.31 MiB | none | 815.52K/s |
| q34 | group by a long string | 24.908ms | 120.846ms | 120.840ms | 0.2% | 120.749ms | 121.031ms | 120.656ms | 121.440ms | 170.000ms | 124.02 MiB | none | 827.53K/s |
| q35 | group by a constant and a long string | 27.516ms | 120.616ms | 121.118ms | 0.5% | 121.071ms | 121.734ms | 120.980ms | 122.531ms | 170.000ms | 130.59 MiB | none | 825.62K/s |
| q37 | date range and group by a URL | 17.723ms | 122.527ms | 121.142ms | 0.9% | 120.963ms | 122.025ms | 101.020ms | 123.463ms | 140.000ms | 84.59 MiB | none | 825.46K/s |
| q38 | date range and group by a title | 18.811ms | 126.993ms | 120.871ms | 0.8% | 120.730ms | 121.704ms | 120.703ms | 123.440ms | 150.000ms | 83.99 MiB | none | 827.31K/s |
| q39 | date range, group by and offset | 16.392ms | 101.018ms | 100.828ms | 21.9% | 100.661ms | 122.708ms | 100.534ms | 129.755ms | 180.000ms | 81.73 MiB | none | 991.77K/s |
| q40 | date range, a case and a wide group by | 16.405ms | 100.615ms | 100.781ms | 21.6% | 100.734ms | 122.503ms | 100.537ms | 125.140ms | 160.000ms | 85.10 MiB | none | 992.23K/s |
| q41 | date range with an IN and a hash | 14.608ms | 100.617ms | 100.783ms | 19.9% | 100.702ms | 120.724ms | 100.527ms | 120.997ms | 190.000ms | 80.84 MiB | none | 992.21K/s |
| q42 | date range and a deep offset | 14.986ms | 100.596ms | 100.635ms | 0.3% | 100.624ms | 100.901ms | 100.520ms | 122.046ms | 180.000ms | 79.49 MiB | none | 993.67K/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 726.690ms by its own clock and 4.383s by ours, 4.492s cold, 6.710s of CPU, peak 130.59 MiB, 5.37M/s and 1.20 GiB/s.

Running it cost 503% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.43x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.694ms | 20.288ms | 20.248ms | 0.1% | 20.248ms | 20.262ms | 20.242ms | 20.477ms | 0.000us | 11.99 MiB | none | 4.94M/s |
| q2 | filtered count | 1.949ms | 24.432ms | 20.272ms | 1.3% | 20.250ms | 20.514ms | 20.236ms | 23.846ms | 0.000us | 12.47 MiB | none | 4.93M/s |
| q3 | three aggregates | 2.101ms | 20.238ms | 20.247ms | 0.9% | 20.246ms | 20.437ms | 20.234ms | 20.502ms | 0.000us | 12.98 MiB | none | 4.94M/s |
| q4 | average | 1.933ms | 20.491ms | 20.310ms | 0.4% | 20.235ms | 20.312ms | 20.231ms | 20.339ms | 0.000us | 13.67 MiB | none | 4.92M/s |
| q5 | count distinct, high card | 2.157ms | 20.251ms | 20.250ms | 0.2% | 20.243ms | 20.285ms | 20.242ms | 20.391ms | 10.000ms | 14.27 MiB | none | 4.94M/s |
| q6 | count distinct, strings | 4.053ms | 20.245ms | 20.266ms | 0.2% | 20.263ms | 20.314ms | 20.262ms | 20.461ms | 10.000ms | 22.68 MiB | none | 4.93M/s |
| q7 | min and max of a date | 2.082ms | 20.301ms | 20.483ms | 0.9% | 20.455ms | 20.634ms | 20.382ms | 20.636ms | 0.000us | 12.66 MiB | none | 4.88M/s |
| q8 | group by, low card | 2.075ms | 20.329ms | 20.254ms | 0.0% | 20.253ms | 20.254ms | 20.244ms | 20.259ms | 0.000us | 13.45 MiB | none | 4.94M/s |
| q9 | group by and count distinct | 2.897ms | 20.249ms | 20.399ms | 0.7% | 20.280ms | 20.414ms | 20.275ms | 20.436ms | 0.000us | 24.14 MiB | none | 4.90M/s |
| q10 | group by, several aggregates | 3.512ms | 20.440ms | 20.276ms | 0.0% | 20.275ms | 20.282ms | 20.272ms | 20.284ms | 20.000ms | 26.70 MiB | none | 4.93M/s |
| q11 | group by a string and count distinct | 2.790ms | 20.274ms | 20.411ms | 0.5% | 20.411ms | 20.523ms | 20.266ms | 20.623ms | 10.000ms | 16.20 MiB | none | 4.90M/s |
| q12 | group by two strings and count distinct | 2.769ms | 20.302ms | 20.255ms | 0.0% | 20.251ms | 20.261ms | 20.248ms | 20.404ms | 0.000us | 15.39 MiB | none | 4.94M/s |
| q13 | group by a string and top k | 4.127ms | 20.682ms | 20.455ms | 0.9% | 20.324ms | 20.509ms | 20.307ms | 20.587ms | 0.000us | 22.23 MiB | none | 4.89M/s |
| q14 | group by a string and count distinct | 4.786ms | 20.273ms | 20.289ms | 0.2% | 20.289ms | 20.324ms | 20.272ms | 20.519ms | 0.000us | 29.99 MiB | none | 4.93M/s |
| q15 | group by two columns and top k | 4.073ms | 20.276ms | 20.278ms | 0.1% | 20.272ms | 20.284ms | 20.265ms | 20.299ms | 0.000us | 21.21 MiB | none | 4.93M/s |
| q16 | group by, very high card | 4.432ms | 20.263ms | 20.274ms | 0.0% | 20.271ms | 20.278ms | 20.266ms | 20.370ms | 10.000ms | 27.47 MiB | none | 4.93M/s |
| q17 | group by two, very high card | 5.106ms | 20.275ms | 20.338ms | 1.0% | 20.291ms | 20.488ms | 20.276ms | 20.538ms | 20.000ms | 42.22 MiB | none | 4.92M/s |
| q18 | group by two, no ordering | 3.227ms | 20.281ms | 20.266ms | 0.0% | 20.262ms | 20.270ms | 20.253ms | 20.387ms | 20.000ms | 18.94 MiB | none | 4.93M/s |
| q19 | group by with an extract | 6.016ms | 20.402ms | 20.365ms | 0.6% | 20.315ms | 20.443ms | 20.299ms | 20.483ms | 20.000ms | 50.23 MiB | none | 4.91M/s |
| q20 | point lookup | 1.807ms | 20.362ms | 20.264ms | 0.7% | 20.243ms | 20.377ms | 20.233ms | 20.638ms | 0.000us | 13.98 MiB | none | 4.93M/s |
| q21 | substring scan | 5.235ms | 20.478ms | 20.320ms | 0.7% | 20.293ms | 20.444ms | 20.267ms | 20.470ms | 10.000ms | 30.41 MiB | none | 4.92M/s |
| q22 | substring scan and group by | 5.145ms | 20.370ms | 20.424ms | 0.7% | 20.290ms | 20.424ms | 20.283ms | 20.426ms | 20.000ms | 35.98 MiB | none | 4.90M/s |
| q23 | two substring scans and group by | 8.779ms | 20.355ms | 20.378ms | 0.5% | 20.318ms | 20.422ms | 20.307ms | 20.443ms | 40.000ms | 57.11 MiB | none | 4.91M/s |
| q24 | select star and top k | 9.248ms | 20.372ms | 20.337ms | 0.2% | 20.325ms | 20.364ms | 20.321ms | 20.448ms | 60.000ms | 77.98 MiB | none | 4.92M/s |
| q25 | top k by a date | 2.471ms | 20.428ms | 20.273ms | 0.0% | 20.272ms | 20.280ms | 20.266ms | 20.289ms | 0.000us | 17.97 MiB | none | 4.93M/s |
| q26 | top k by a string | 2.842ms | 20.246ms | 20.246ms | 0.0% | 20.240ms | 20.247ms | 20.239ms | 20.281ms | 20.000ms | 13.93 MiB | none | 4.94M/s |
| q27 | top k by two columns | 2.870ms | 20.250ms | 20.261ms | 0.1% | 20.252ms | 20.274ms | 20.246ms | 20.299ms | 0.000us | 16.70 MiB | none | 4.94M/s |
| q28 | group by with a string length | 5.059ms | 20.293ms | 20.297ms | 0.0% | 20.296ms | 20.299ms | 20.293ms | 20.300ms | 20.000ms | 35.70 MiB | none | 4.93M/s |
| q29 | group by a regular expression | 9.146ms | 20.299ms | 20.308ms | 0.4% | 20.305ms | 20.389ms | 20.301ms | 20.430ms | 30.000ms | 56.91 MiB | none | 4.92M/s |
| q30 | ninety sums over one column | 3.283ms | 20.302ms | 20.261ms | 0.2% | 20.256ms | 20.295ms | 20.250ms | 20.395ms | 0.000us | 14.28 MiB | none | 4.94M/s |
| q31 | group by two and several aggregates | 3.943ms | 20.396ms | 20.404ms | 0.2% | 20.382ms | 20.414ms | 20.291ms | 20.432ms | 0.000us | 15.75 MiB | none | 4.90M/s |
| q32 | group by a high card pair | 3.970ms | 20.257ms | 20.255ms | 0.7% | 20.254ms | 20.404ms | 20.250ms | 20.412ms | 20.000ms | 16.44 MiB | none | 4.94M/s |
| q33 | group by a high card pair, unfiltered | 3.704ms | 20.253ms | 20.406ms | 0.1% | 20.404ms | 20.415ms | 20.273ms | 20.445ms | 0.000us | 22.75 MiB | none | 4.90M/s |
| q34 | group by a long string | 9.555ms | 20.282ms | 20.318ms | 0.1% | 20.313ms | 20.329ms | 20.309ms | 20.391ms | 40.000ms | 92.64 MiB | none | 4.92M/s |
| q35 | group by a constant and a long string | 8.731ms | 20.370ms | 20.314ms | 0.5% | 20.312ms | 20.412ms | 20.305ms | 20.420ms | 40.000ms | 90.89 MiB | none | 4.92M/s |
| q36 | group by four expressions | 5.161ms | 20.330ms | 20.526ms | 0.6% | 20.439ms | 20.566ms | 20.306ms | 20.933ms | 10.000ms | 28.91 MiB | none | 4.87M/s |
| q37 | date range and group by a URL | 3.635ms | 20.312ms | 20.277ms | 0.1% | 20.255ms | 20.284ms | 20.251ms | 20.285ms | 0.000us | 19.24 MiB | none | 4.93M/s |
| q38 | date range and group by a title | 4.002ms | 20.269ms | 20.254ms | 0.0% | 20.251ms | 20.260ms | 20.247ms | 20.261ms | 10.000ms | 17.96 MiB | none | 4.94M/s |
| q39 | date range, group by and offset | 3.682ms | 20.252ms | 20.275ms | 0.0% | 20.266ms | 20.275ms | 20.266ms | 20.276ms | 0.000us | 18.89 MiB | none | 4.93M/s |
| q40 | date range, a case and a wide group by | 5.445ms | 20.262ms | 20.273ms | 0.0% | 20.269ms | 20.276ms | 20.260ms | 20.482ms | 0.000us | 24.00 MiB | none | 4.93M/s |
| q41 | date range with an IN and a hash | 2.321ms | 20.264ms | 20.283ms | 0.2% | 20.256ms | 20.288ms | 20.249ms | 20.511ms | 10.000ms | 14.73 MiB | none | 4.93M/s |
| q42 | date range and a deep offset | 2.267ms | 20.254ms | 20.400ms | 0.7% | 20.353ms | 20.496ms | 20.250ms | 20.630ms | 0.000us | 14.23 MiB | none | 4.90M/s |
| q43 | minute buckets over a date range | 2.552ms | 20.397ms | 20.371ms | 0.6% | 20.264ms | 20.391ms | 20.263ms | 20.753ms | 0.000us | 15.02 MiB | none | 4.91M/s |

rudb rudb 0.3.67 over 43 of 43 queries. Total 176.633ms by its own clock and 873.662ms by ours, 877.947ms cold, 450.000ms of CPU, peak 92.64 MiB, 24.34M/s and 5.43 GiB/s.

Running it cost 395% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.01x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | 973.985us | 1.005ms | 1.042ms | 1.052ms | 0.9% | 1.042ms | 414.109us | 0.000us | 896 B | 4 of 4 |
| q2 | 864.097us | 1.096ms | 788.529us | 797.003us | 1.1% | 788.529us | 338.564us | 8.864ms | 896 B | 5 of 5 |
| q3 | 920.516us | 880.771us | 954.751us | 971.070us | 1.7% | 954.751us | 330.194us | 0.000us | 1.22 KiB | 4 of 4 |
| q4 | 861.502us | 858.200us | 1.020ms | 1.030ms | 0.9% | 1.020ms | 337.642us | 0.000us | 896 B | 4 of 4 |
| q5 | 910.137us | 1.081ms | 1.427ms | 1.436ms | 0.6% | 1.427ms | 335.383us | 8.229ms | 1.10 MiB | 4 of 4 |
| q6 | 926.601us | 2.795ms | 6.573ms | 6.581ms | 0.1% | 6.573ms | 350.537us | 3.068ms | 2.38 MiB | 5 of 5 |
| q7 | 1.063ms | 1.386ms | 782.124us | 789.640us | 1.0% | 782.124us | 395.297us | 0.000us | 1.00 KiB | 4 of 4 |
| q8 | 922.996us | 735.397us | 954.835us | 963.810us | 0.9% | 954.835us | 372.483us | 0.000us | 2.84 KiB | 6 of 6 |
| q9 | 923.738us | 2.666ms | 4.734ms | 4.744ms | 0.2% | 4.734ms | 387.242us | 4.869ms | 4.88 MiB | 5 of 5 |
| q10 | 1.014ms | 2.109ms | 7.701ms | 7.715ms | 0.2% | 7.701ms | 409.995us | 0.000us | 5.03 MiB | 5 of 5 |
| q11 | 1.033ms | 1.236ms | 3.148ms | 3.159ms | 0.3% | 3.148ms | 386.730us | 6.454ms | 163.12 KiB | 6 of 6 |
| q12 | 903.550us | 1.471ms | 2.387ms | 2.396ms | 0.4% | 2.387ms | 345.324us | 0.000us | 187.22 KiB | 6 of 6 |
| q13 | 963.001us | 8.098ms | 8.010ms | 8.018ms | 0.1% | 8.010ms | 342.576us | 1.639ms | 1.47 MiB | 6 of 6 |
| q14 | 942.404us | 4.110ms | 10.397ms | 10.409ms | 0.1% | 10.397ms | 355.496us | 0.000us | 3.08 MiB | 6 of 6 |
| q15 | 965.073us | 2.962ms | 7.134ms | 7.145ms | 0.2% | 7.134ms | 361.099us | 0.000us | 2.84 MiB | 6 of 6 |
| q16 | 1.094ms | 4.695ms | 10.231ms | 10.238ms | 0.1% | 10.231ms | 449.656us | 0.000us | 5.88 MiB | 5 of 5 |
| q17 | 968.807us | 4.141ms | 17.247ms | 17.259ms | 0.1% | 17.247ms | 354.891us | 2.386ms | 7.43 MiB | 5 of 5 |
| q18 | 1.029ms | 1.481ms | 5.885ms | 5.897ms | 0.2% | 5.885ms | 433.039us | 3.669ms | 5.65 KiB | 5 of 5 |
| q19 | 933.977us | 5.923ms | 19.902ms | 19.914ms | 0.1% | 19.902ms | 339.301us | 9.747ms | 8.07 MiB | 5 of 5 |
| q20 | 982.812us | 665.003us | 1.697ms | 1.704ms | 0.4% | 1.697ms | 369.668us | 0.000us | 0 B | 4 of 4 |
| q21 | 888.180us | 11.531ms | 16.822ms | 16.829ms | 0.0% | 16.822ms | 354.115us | 0.000us | 896 B | 5 of 5 |
| q22 | 1.064ms | 3.942ms | 18.032ms | 18.044ms | 0.1% | 18.032ms | 455.946us | 1.500ms | 3.26 KiB | 6 of 6 |
| q23 | 1.082ms | 7.392ms | 38.749ms | 38.767ms | 0.0% | 38.749ms | 402.862us | 0.000us | 13.16 KiB | 6 of 6 |
| q24 | 1.272ms | 8.042ms | 21.987ms | 22.001ms | 0.1% | 21.987ms | 487.105us | 37.512ms | 31.88 KiB | 8 of 8 |
| q25 | 980.692us | 1.414ms | 6.127ms | 6.136ms | 0.1% | 6.127ms | 378.487us | 0.000us | 39.88 KiB | 6 of 6 |
| q26 | 1.006ms | 1.921ms | 3.146ms | 3.163ms | 0.6% | 3.146ms | 364.981us | 16.472ms | 35.12 KiB | 5 of 5 |
| q27 | 918.723us | 2.068ms | 3.741ms | 3.749ms | 0.2% | 3.741ms | 359.776us | 5.891ms | 55.92 KiB | 6 of 6 |
| q28 | 1.044ms | 3.602ms | 17.362ms | 17.369ms | 0.0% | 17.362ms | 402.679us | 0.000us | 368.50 KiB | 7 of 7 |
| q29 | 1.106ms | 7.556ms | 29.167ms | 29.175ms | 0.0% | 29.167ms | 434.229us | 0.000us | 4.26 MiB | 7 of 7 |
| q30 | 2.305ms | 897.872us | 1.464ms | 1.480ms | 1.1% | 1.464ms | 433.981us | 0.000us | 29.59 KiB | 4 of 4 |
| q31 | 1.109ms | 2.722ms | 4.478ms | 4.491ms | 0.3% | 4.478ms | 390.640us | 15.118ms | 372.66 KiB | 6 of 6 |
| q32 | 1.044ms | 2.815ms | 4.603ms | 4.615ms | 0.3% | 4.603ms | 384.927us | 0.000us | 369.61 KiB | 6 of 6 |
| q33 | 932.233us | 3.010ms | 4.627ms | 4.638ms | 0.3% | 4.627ms | 338.809us | 15.023ms | 2.31 MiB | 5 of 5 |
| q34 | 945.827us | 9.730ms | 31.662ms | 31.677ms | 0.0% | 31.662ms | 349.830us | 0.000us | 13.57 MiB | 5 of 5 |
| q35 | 952.433us | 7.600ms | 45.979ms | 45.990ms | 0.0% | 45.979ms | 368.732us | 3.642ms | 15.80 MiB | 5 of 5 |
| q36 | 1.070ms | 3.176ms | 12.787ms | 12.795ms | 0.1% | 12.787ms | 425.657us | 6.779ms | 5.18 MiB | 6 of 6 |
| q37 | 979.251us | 2.737ms | 4.704ms | 4.717ms | 0.3% | 4.704ms | 368.201us | 0.000us | 110.58 KiB | 6 of 6 |
| q38 | 1.037ms | 2.802ms | 4.726ms | 4.748ms | 0.5% | 4.726ms | 361.782us | 14.890ms | 33.21 KiB | 6 of 6 |
| q39 | 959.114us | 2.465ms | 4.207ms | 4.218ms | 0.2% | 4.207ms | 358.030us | 0.000us | 15.78 KiB | 6 of 6 |
| q40 | 1.082ms | 7.071ms | 6.741ms | 6.755ms | 0.2% | 6.741ms | 404.761us | 0.000us | 633.74 KiB | 6 of 6 |
| q41 | 1.062ms | 1.208ms | 1.452ms | 1.464ms | 0.8% | 1.452ms | 396.979us | 8.139ms | 31.12 KiB | 6 of 6 |
| q42 | 971.949us | 1.157ms | 1.139ms | 1.151ms | 1.1% | 1.139ms | 363.377us | 0.000us | 29.80 KiB | 6 of 6 |
| q43 | 1.008ms | 1.133ms | 1.221ms | 1.231ms | 0.8% | 1.221ms | 374.811us | 8.394ms | 158.39 KiB | 6 of 6 |

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
| Aggregate | 0.000us | nothing to share | 39 | 1973753 | 20808 | 0.0ns | 0.0ns | 39 of 39 |
| Fetch | 0.000us | nothing to share | 1 | 10 | 10 | 0.0ns | 0.0ns | 1 of 1 |
| FileScan | 0.000us | nothing to share | 43 | 0 | 3719285 | handed none | 0.0ns | 43 of 43 |
| Filter | 0.000us | nothing to share | 28 | 2019319 | 301032 | 0.0ns | 0.0ns | 28 of 28 |
| Limit | 0.000us | nothing to share | 1 | 10 | 10 | 0.0ns | 0.0ns | 1 of 1 |
| Project | 0.000us | nothing to share | 91 | 2049731 | 2049731 | 0.0ns | 0.0ns | 91 of 91 |
| Sort | 0.000us | nothing to share | 1 | 8 | 8 | 0.0ns | 0.0ns | 1 of 1 |
| TopN | 0.000us | nothing to share | 31 | 48026 | 228 | 0.0ns | 0.0ns | 31 of 31 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is Aggregate, and the queries where it cost the most are q1 at 0.000us, q2 at 0.000us, q3 at 0.000us.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 99998 rows, one out of every 1000 of the 99997497 in the full file, which is a development loop rather than the suite

These swung wider than reporting rule two allows:

- duckdb-pinned swung by 33.3% of its median on q35, and rule two wants under 10%
- clickhouse-local swung by 33.2% of its median on q3, and rule two wants under 10%
- datafusion swung by 49.8% of its median on q38, and rule two wants under 10%
- polars swung by 21.9% of its median on q39, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

These ran every query at about the same speed:

- duckdb-pinned ran every query within 2.00x of every other one, and this suite spreads over 6x on an engine it is measuring
- polars ran every query within 1.43x of every other one, and this suite spreads over 6x on an engine it is measuring
- rudb ran every query within 1.01x of every other one, and this suite spreads over 6x on an engine it is measuring

A column that flat is not a column about the queries. Read those numbers as an upper bound on the engine and not as a measurement of it.

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
- q31: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q32: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q33: ORDER BY COUNT(*) DESC LIMIT 10 over WatchID and ClientIP unfiltered, where almost every group has a count of one and the ten that come back are arbitrary.
- q36: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q38: ORDER BY PageViews DESC LIMIT 10 over page titles, a tie at the cut as q37. Seen on a hundred thousand row sample, where the tenth and eleventh titles both had thirteen views and the two engines kept a different one. Sampling makes this more likely than the full file does, because the counts are smaller and so more of them collide.

So they are not checked, and a wrong answer from any engine on one of them would go unnoticed here. Every other query in the suite is checked in full.

These were answered differently and which engine is wrong is settled:

- q4: AVG(UserID) over a hundred million bigints, where the pinned DuckDB sums the column short by a multiple of 2^64 and rudb does not. Its own hugeint and decimal paths, the same column summed out of a table, its own per counter group sums, and adding the column up outside a database all give rudb's answer, and it reduces to two statements over a 1.3 MB one column parquet file. Written up as 14.10.1 in tamnd/rudb `spec/14-testing.md`, which is the list of places where DuckDB is wrong and we are not. tamnd/rudb-compat#12.

So they are a known difference rather than an open one, and the write up is where to go to disagree with that.

Hot here means page cache warm and not buffer pool warm, because every run is a fresh process so that no query's number depends on the one before it.

