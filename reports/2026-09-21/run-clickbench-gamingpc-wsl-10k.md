# clickbench on gamingpc-wsl

This is one run of the clickbench suite on gamingpc-wsl, over 6 engines and 43 queries, with 5 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

It ran over 10000 rows, one out of every 10000 of the 99997497 in the full file, which is a development loop and not the suite. Nothing here is comparable to a full run or to anybody else's number.

## What ran

| what | it was |
| --- | --- |
| suite | clickbench |
| queries | 43 |
| tables | 2.47 MiB of Parquet in 1 table |
| rows | 10000 in the table every query reads |
| sample | 10000 rows, one out of every 10000 of the 99997497 in the full file |
| corpus | no manifest beside the data, so this run cannot say where it came from |
| summary | median with the interquartile range, per reporting rule two |
| timeout | 900s per query, and no query reached it |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --rows 10000 --engines duckdb,duckdb-pinned,clickhouse-local,datafusion,polars,rudb --runs 5 --timeout 900 --report
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
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 94.185ms | 100.000ms | 4.51 MiB | its own database file | its own | 4.17 to 3.84 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 119.431ms | 130.000ms | 3.76 MiB | its own database file | its own | 3.84 to 3.72 |
| clickhouse-local | 26.9.1.1562 | ran | 130.822ms | 140.000ms | 2.68 MiB | its own MergeTree parts, as system.parts counts the active ones | its own | 3.72 to 3.19 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 2.47 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 3.19 to 2.78 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 2.47 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 2.78 to 2.17 |
| rudb | rudb 0.3.67 | ran | 0.000us | 0.000us | 2.47 MiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 2.17 to 2.00 |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb on 39 shared |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 117.000ms | 870.891ms | +644% | 872.697ms | 290.000ms | 0.33 | 43.82 MiB | none | 3.68M/s | 906.76 MiB/s | 1.00x |
| duckdb-pinned | 157.000ms | 1.736s | +1006% | 1.737s | 950.000ms | 0.55 | 52.99 MiB | none | 2.74M/s | 675.74 MiB/s | 1.38x |
| clickhouse-local | 360.000ms | 2.920s | +711% | 2.880s | 2.580s | 0.88 | 251.21 MiB | none | 1.19M/s | 294.70 MiB/s | 3.19x |
| datafusion | 275.000ms | 952.976ms | +247% | 973.612ms | 1.560s | 1.64 | 199.03 MiB | none | 1.56M/s | 385.78 MiB/s | 2.46x |
| polars | 561.828ms | 4.317s | +668% | 4.408s | 6.250s | 1.45 | 88.75 MiB | none | 694.16K/s | 171.27 MiB/s | 5.56x |
| rudb | 75.844ms | 874.709ms | +1053% | 874.240ms | 0.000us | 0.00 | 18.45 MiB | none | 5.67M/s | 1.37 GiB/s | 0.67x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 1.000ms | 5.000ms | 1.000ms | 5.930ms | 747.448us |
| q2 | filtered count | 1.000ms | 1.000ms | 5.000ms | 5.000ms | 6.228ms | 870.474us |
| q3 | three aggregates | 0.000us | 1.000ms | 6.000ms | 4.000ms | 6.103ms | 822.376us |
| q4 | average | 1.000ms | 1.000ms | 5.000ms | 3.000ms | 6.565ms | 911.195us |
| q5 | count distinct, high card | 2.000ms | 2.000ms | 6.000ms | 6.000ms | 12.357ms | 890.716us |
| q6 | count distinct, strings | 2.000ms | 2.000ms | 6.000ms | 6.000ms | 12.849ms | 1.254ms |
| q7 | min and max of a date | 1.000ms | 1.000ms | 9.000ms | 1.000ms | 6.479ms | 832.774us |
| q8 | group by, low card | 1.000ms | 6.000ms | 11.000ms | 5.000ms | 12.842ms | 826.978us |
| q9 | group by and count distinct | 4.000ms | 4.000ms | 6.000ms | 9.000ms | 21.591ms | 1.168ms |
| q10 | group by, several aggregates | 5.000ms | 6.000ms | 6.000ms | 7.000ms | 27.009ms | 1.710ms |
| q11 | group by a string and count distinct | 4.000ms | 4.000ms | 6.000ms | 9.000ms | 20.092ms | 933.609us |
| q12 | group by two strings and count distinct | 3.000ms | 4.000ms | 6.000ms | 8.000ms | 22.252ms | 988.957us |
| q13 | group by a string and top k | 3.000ms | 3.000ms | 6.000ms | 7.000ms | 14.291ms | 1.110ms |
| q14 | group by a string and count distinct | 4.000ms | 4.000ms | 6.000ms | 9.000ms | 20.877ms | 1.233ms |
| q15 | group by two columns and top k | 3.000ms | 3.000ms | 6.000ms | 7.000ms | 15.411ms | 1.172ms |
| q16 | group by, very high card | 3.000ms | 3.000ms | 6.000ms | 5.000ms | 17.553ms | 1.849ms |
| q17 | group by two, very high card | 3.000ms | 3.000ms | 8.000ms | 6.000ms | 22.012ms | 2.703ms |
| q18 | group by two, no ordering | 3.000ms | 3.000ms | 6.000ms | 6.000ms | 13.069ms | 1.185ms |
| q19 | group by with an extract | 3.000ms | 3.000ms | 8.000ms | 7.000ms | 23.882ms | 3.072ms |
| q20 | point lookup | 0.000us | 1.000ms | 10.000ms | 3.000ms | 5.523ms | 700.133us |
| q21 | substring scan | 1.000ms | 2.000ms | 7.000ms | 4.000ms | 7.684ms | 2.108ms |
| q22 | substring scan and group by | 2.000ms | 3.000ms | 8.000ms | 6.000ms | 14.696ms | 2.402ms |
| q23 | two substring scans and group by | 4.000ms | 6.000ms | 9.000ms | 9.000ms | 20.999ms | 4.518ms |
| q24 | select star and top k | 6.000ms | 10.000ms | 15.000ms | 12.000ms | 11.157ms | 2.538ms |
| q25 | top k by a date | 2.000ms | 2.000ms | 11.000ms | 4.000ms | 9.421ms | 1.057ms |
| q26 | top k by a string | 1.000ms | 1.000ms | 6.000ms | 4.000ms | 9.885ms | 1.034ms |
| q27 | top k by two columns | 1.000ms | 1.000ms | 10.000ms | 5.000ms | 11.331ms | 1.123ms |
| q28 | group by with a string length | 3.000ms | 4.000ms | 7.000ms | 6.000ms | no dialect | 2.320ms |
| q29 | group by a regular expression | 7.000ms | 8.000ms | 12.000ms | 10.000ms | no dialect | 2.938ms |
| q30 | ninety sums over one column | 3.000ms | 15.000ms | 9.000ms | 11.000ms | 10.149ms | 2.129ms |
| q31 | group by two and several aggregates | 3.000ms | 4.000ms | 7.000ms | 7.000ms | 14.506ms | 1.427ms |
| q32 | group by a high card pair | 3.000ms | 3.000ms | 7.000ms | 7.000ms | 14.889ms | 1.408ms |
| q33 | group by a high card pair, unfiltered | 3.000ms | 3.000ms | 7.000ms | 6.000ms | 15.509ms | 1.431ms |
| q34 | group by a long string | 3.000ms | 4.000ms | 8.000ms | 7.000ms | 17.460ms | 4.309ms |
| q35 | group by a constant and a long string | 3.000ms | 4.000ms | 8.000ms | 7.000ms | 19.416ms | 4.214ms |
| q36 | group by four expressions | 3.000ms | 3.000ms | 7.000ms | 5.000ms | no dialect | 1.823ms |
| q37 | date range and group by a URL | 3.000ms | 3.000ms | 13.000ms | 7.000ms | 17.523ms | 2.120ms |
| q38 | date range and group by a title | 3.000ms | 4.000ms | 13.000ms | 9.000ms | 17.371ms | 2.892ms |
| q39 | date range, group by and offset | 3.000ms | 3.000ms | 13.000ms | 8.000ms | 14.419ms | 2.247ms |
| q40 | date range, a case and a wide group by | 4.000ms | 5.000ms | 14.000ms | 9.000ms | 14.941ms | 3.436ms |
| q41 | date range with an IN and a hash | 3.000ms | 3.000ms | 12.000ms | 6.000ms | 14.239ms | 1.283ms |
| q42 | date range and a deep offset | 3.000ms | 7.000ms | 12.000ms | 6.000ms | 13.318ms | 1.092ms |
| q43 | minute buckets over a date range | 3.000ms | 3.000ms | 12.000ms | 6.000ms | no dialect | 1.015ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 20.345ms | 20.221ms | 0.7% | 20.221ms | 20.356ms | 20.208ms | 20.370ms | 0.000us | 30.87 MiB | none | 494.54K/s |
| q2 | filtered count | 1.000ms | 20.484ms | 20.218ms | 0.3% | 20.202ms | 20.260ms | 20.200ms | 20.374ms | 0.000us | 31.81 MiB | none | 494.60K/s |
| q3 | three aggregates | 0.000us | 20.351ms | 20.229ms | 0.2% | 20.217ms | 20.255ms | 20.215ms | 20.423ms | 0.000us | 31.87 MiB | none | 494.35K/s |
| q4 | average | 1.000ms | 20.381ms | 20.237ms | 0.3% | 20.212ms | 20.276ms | 20.203ms | 20.302ms | 0.000us | 31.43 MiB | none | 494.13K/s |
| q5 | count distinct, high card | 2.000ms | 20.359ms | 20.240ms | 0.9% | 20.238ms | 20.417ms | 20.208ms | 20.523ms | 10.000ms | 34.94 MiB | none | 494.07K/s |
| q6 | count distinct, strings | 2.000ms | 20.232ms | 20.257ms | 0.1% | 20.251ms | 20.266ms | 20.242ms | 20.294ms | 10.000ms | 34.31 MiB | none | 493.66K/s |
| q7 | min and max of a date | 1.000ms | 20.226ms | 20.268ms | 0.2% | 20.248ms | 20.280ms | 20.237ms | 20.295ms | 0.000us | 31.00 MiB | none | 493.40K/s |
| q8 | group by, low card | 1.000ms | 20.295ms | 20.259ms | 0.1% | 20.255ms | 20.270ms | 20.243ms | 20.331ms | 0.000us | 34.07 MiB | none | 493.60K/s |
| q9 | group by and count distinct | 4.000ms | 20.256ms | 20.271ms | 0.0% | 20.268ms | 20.271ms | 20.265ms | 20.275ms | 10.000ms | 42.00 MiB | none | 493.31K/s |
| q10 | group by, several aggregates | 5.000ms | 20.289ms | 20.260ms | 0.0% | 20.253ms | 20.262ms | 20.234ms | 20.266ms | 10.000ms | 43.51 MiB | none | 493.58K/s |
| q11 | group by a string and count distinct | 4.000ms | 20.245ms | 20.244ms | 0.0% | 20.242ms | 20.250ms | 20.224ms | 20.259ms | 10.000ms | 38.89 MiB | none | 493.98K/s |
| q12 | group by two strings and count distinct | 3.000ms | 20.440ms | 20.242ms | 0.0% | 20.242ms | 20.249ms | 20.231ms | 20.271ms | 10.000ms | 40.14 MiB | none | 494.01K/s |
| q13 | group by a string and top k | 3.000ms | 20.246ms | 20.256ms | 0.0% | 20.249ms | 20.258ms | 20.245ms | 20.291ms | 10.000ms | 35.95 MiB | none | 493.68K/s |
| q14 | group by a string and count distinct | 4.000ms | 20.336ms | 20.248ms | 0.0% | 20.243ms | 20.248ms | 20.238ms | 20.251ms | 10.000ms | 43.82 MiB | none | 493.89K/s |
| q15 | group by two columns and top k | 3.000ms | 20.244ms | 20.264ms | 0.0% | 20.255ms | 20.264ms | 20.237ms | 20.275ms | 10.000ms | 36.26 MiB | none | 493.49K/s |
| q16 | group by, very high card | 3.000ms | 20.264ms | 20.278ms | 0.0% | 20.275ms | 20.281ms | 20.256ms | 20.295ms | 0.000us | 38.39 MiB | none | 493.14K/s |
| q17 | group by two, very high card | 3.000ms | 20.236ms | 20.240ms | 0.0% | 20.239ms | 20.246ms | 20.237ms | 20.256ms | 10.000ms | 39.81 MiB | none | 494.06K/s |
| q18 | group by two, no ordering | 3.000ms | 20.248ms | 20.248ms | 0.0% | 20.247ms | 20.254ms | 20.244ms | 20.258ms | 0.000us | 41.32 MiB | none | 493.88K/s |
| q19 | group by with an extract | 3.000ms | 20.261ms | 20.243ms | 0.0% | 20.240ms | 20.243ms | 20.237ms | 20.249ms | 10.000ms | 40.32 MiB | none | 494.00K/s |
| q20 | point lookup | 0.000us | 20.238ms | 20.244ms | 0.0% | 20.242ms | 20.246ms | 20.240ms | 20.331ms | 0.000us | 31.18 MiB | none | 493.97K/s |
| q21 | substring scan | 1.000ms | 20.359ms | 20.242ms | 0.1% | 20.233ms | 20.246ms | 20.212ms | 20.251ms | 0.000us | 32.25 MiB | none | 494.02K/s |
| q22 | substring scan and group by | 2.000ms | 20.217ms | 20.227ms | 0.4% | 20.211ms | 20.288ms | 20.206ms | 20.425ms | 10.000ms | 33.45 MiB | none | 494.39K/s |
| q23 | two substring scans and group by | 4.000ms | 20.208ms | 20.216ms | 0.0% | 20.213ms | 20.218ms | 20.209ms | 20.224ms | 10.000ms | 39.01 MiB | none | 494.66K/s |
| q24 | select star and top k | 6.000ms | 20.228ms | 20.370ms | 0.8% | 20.255ms | 20.422ms | 20.212ms | 20.438ms | 10.000ms | 41.85 MiB | none | 490.91K/s |
| q25 | top k by a date | 2.000ms | 20.234ms | 20.213ms | 0.0% | 20.208ms | 20.214ms | 20.208ms | 20.235ms | 10.000ms | 35.12 MiB | none | 494.72K/s |
| q26 | top k by a string | 1.000ms | 20.208ms | 20.217ms | 0.1% | 20.206ms | 20.224ms | 20.205ms | 20.293ms | 0.000us | 32.00 MiB | none | 494.64K/s |
| q27 | top k by two columns | 1.000ms | 20.212ms | 20.212ms | 0.0% | 20.210ms | 20.218ms | 20.207ms | 20.265ms | 0.000us | 32.37 MiB | none | 494.76K/s |
| q28 | group by with a string length | 3.000ms | 20.195ms | 20.214ms | 0.0% | 20.211ms | 20.218ms | 20.204ms | 20.222ms | 0.000us | 37.07 MiB | none | 494.71K/s |
| q29 | group by a regular expression | 7.000ms | 20.352ms | 20.260ms | 0.6% | 20.222ms | 20.346ms | 20.206ms | 20.346ms | 10.000ms | 37.46 MiB | none | 493.59K/s |
| q30 | ninety sums over one column | 3.000ms | 20.222ms | 20.226ms | 0.6% | 20.220ms | 20.332ms | 20.220ms | 20.334ms | 10.000ms | 35.38 MiB | none | 494.40K/s |
| q31 | group by two and several aggregates | 3.000ms | 20.223ms | 20.283ms | 0.4% | 20.279ms | 20.362ms | 20.238ms | 20.410ms | 10.000ms | 38.63 MiB | none | 493.01K/s |
| q32 | group by a high card pair | 3.000ms | 20.365ms | 20.230ms | 0.0% | 20.224ms | 20.231ms | 20.224ms | 20.356ms | 10.000ms | 38.26 MiB | none | 494.31K/s |
| q33 | group by a high card pair, unfiltered | 3.000ms | 20.352ms | 20.228ms | 0.7% | 20.223ms | 20.362ms | 20.216ms | 20.381ms | 10.000ms | 38.95 MiB | none | 494.37K/s |
| q34 | group by a long string | 3.000ms | 20.369ms | 20.289ms | 0.5% | 20.258ms | 20.367ms | 20.228ms | 20.409ms | 10.000ms | 38.40 MiB | none | 492.87K/s |
| q35 | group by a constant and a long string | 3.000ms | 20.226ms | 20.229ms | 0.1% | 20.221ms | 20.235ms | 20.219ms | 20.239ms | 10.000ms | 38.95 MiB | none | 494.35K/s |
| q36 | group by four expressions | 3.000ms | 20.279ms | 20.268ms | 0.7% | 20.225ms | 20.364ms | 20.185ms | 20.365ms | 0.000us | 37.64 MiB | none | 493.40K/s |
| q37 | date range and group by a URL | 3.000ms | 20.238ms | 20.239ms | 0.1% | 20.228ms | 20.247ms | 20.223ms | 20.275ms | 10.000ms | 36.88 MiB | none | 494.10K/s |
| q38 | date range and group by a title | 3.000ms | 20.259ms | 20.361ms | 0.4% | 20.282ms | 20.362ms | 20.228ms | 20.363ms | 10.000ms | 36.20 MiB | none | 491.14K/s |
| q39 | date range, group by and offset | 3.000ms | 20.607ms | 20.228ms | 0.6% | 20.223ms | 20.335ms | 20.223ms | 20.355ms | 10.000ms | 35.82 MiB | none | 494.36K/s |
| q40 | date range, a case and a wide group by | 4.000ms | 20.224ms | 20.325ms | 0.3% | 20.291ms | 20.353ms | 20.245ms | 20.373ms | 10.000ms | 38.26 MiB | none | 492.00K/s |
| q41 | date range with an IN and a hash | 3.000ms | 20.268ms | 20.314ms | 0.2% | 20.284ms | 20.316ms | 20.270ms | 48.623ms | 10.000ms | 38.50 MiB | none | 492.28K/s |
| q42 | date range and a deep offset | 3.000ms | 20.261ms | 20.286ms | 0.1% | 20.279ms | 20.297ms | 20.278ms | 41.467ms | 10.000ms | 37.20 MiB | none | 492.95K/s |
| q43 | minute buckets over a date range | 3.000ms | 20.614ms | 20.247ms | 0.1% | 20.243ms | 20.270ms | 20.242ms | 20.282ms | 10.000ms | 35.75 MiB | none | 493.90K/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 117.000ms by its own clock and 870.891ms by ours, 872.697ms cold, 290.000ms of CPU, peak 43.82 MiB, 3.68M/s and 906.76 MiB/s.

Running it cost 644% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.01x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 40.337ms | 40.439ms | 0.1% | 40.383ms | 40.444ms | 40.370ms | 40.523ms | 20.000ms | 39.59 MiB | none | 247.28K/s |
| q2 | filtered count | 1.000ms | 40.385ms | 40.343ms | 0.1% | 40.331ms | 40.368ms | 40.329ms | 40.371ms | 20.000ms | 40.33 MiB | none | 247.88K/s |
| q3 | three aggregates | 1.000ms | 40.344ms | 40.332ms | 0.1% | 40.321ms | 40.378ms | 40.319ms | 40.438ms | 20.000ms | 40.32 MiB | none | 247.94K/s |
| q4 | average | 1.000ms | 40.385ms | 40.382ms | 0.1% | 40.365ms | 40.396ms | 40.349ms | 40.439ms | 20.000ms | 40.33 MiB | none | 247.63K/s |
| q5 | count distinct, high card | 2.000ms | 40.448ms | 40.393ms | 0.1% | 40.359ms | 40.417ms | 40.313ms | 40.419ms | 20.000ms | 43.41 MiB | none | 247.57K/s |
| q6 | count distinct, strings | 2.000ms | 40.318ms | 40.365ms | 0.1% | 40.331ms | 40.390ms | 40.328ms | 40.409ms | 20.000ms | 42.52 MiB | none | 247.74K/s |
| q7 | min and max of a date | 1.000ms | 40.374ms | 40.450ms | 0.2% | 40.391ms | 40.468ms | 40.391ms | 40.660ms | 20.000ms | 39.90 MiB | none | 247.22K/s |
| q8 | group by, low card | 6.000ms | 40.383ms | 40.377ms | 0.1% | 40.329ms | 40.382ms | 40.322ms | 40.392ms | 20.000ms | 42.97 MiB | none | 247.67K/s |
| q9 | group by and count distinct | 4.000ms | 40.353ms | 40.400ms | 0.0% | 40.399ms | 40.413ms | 40.389ms | 40.530ms | 30.000ms | 48.88 MiB | none | 247.52K/s |
| q10 | group by, several aggregates | 6.000ms | 40.388ms | 40.417ms | 0.1% | 40.399ms | 40.439ms | 40.387ms | 40.475ms | 30.000ms | 50.41 MiB | none | 247.42K/s |
| q11 | group by a string and count distinct | 4.000ms | 40.464ms | 40.481ms | 0.4% | 40.402ms | 40.557ms | 40.400ms | 40.815ms | 30.000ms | 47.97 MiB | none | 247.03K/s |
| q12 | group by two strings and count distinct | 4.000ms | 40.456ms | 40.396ms | 0.1% | 40.374ms | 40.421ms | 40.369ms | 40.641ms | 30.000ms | 48.66 MiB | none | 247.55K/s |
| q13 | group by a string and top k | 3.000ms | 40.457ms | 40.391ms | 0.1% | 40.372ms | 40.420ms | 40.348ms | 40.422ms | 20.000ms | 42.89 MiB | none | 247.58K/s |
| q14 | group by a string and count distinct | 4.000ms | 40.391ms | 40.348ms | 0.1% | 40.348ms | 40.373ms | 40.347ms | 40.472ms | 30.000ms | 50.57 MiB | none | 247.84K/s |
| q15 | group by two columns and top k | 3.000ms | 40.432ms | 40.386ms | 0.1% | 40.364ms | 40.411ms | 40.330ms | 40.413ms | 20.000ms | 44.07 MiB | none | 247.61K/s |
| q16 | group by, very high card | 3.000ms | 40.364ms | 40.391ms | 0.1% | 40.337ms | 40.392ms | 40.311ms | 40.503ms | 20.000ms | 44.35 MiB | none | 247.58K/s |
| q17 | group by two, very high card | 3.000ms | 40.337ms | 40.343ms | 0.0% | 40.329ms | 40.343ms | 40.329ms | 40.356ms | 20.000ms | 45.11 MiB | none | 247.87K/s |
| q18 | group by two, no ordering | 3.000ms | 40.346ms | 40.384ms | 0.1% | 40.363ms | 40.393ms | 40.359ms | 40.452ms | 20.000ms | 45.18 MiB | none | 247.62K/s |
| q19 | group by with an extract | 3.000ms | 40.387ms | 40.338ms | 0.0% | 40.335ms | 40.342ms | 40.320ms | 40.342ms | 20.000ms | 46.09 MiB | none | 247.90K/s |
| q20 | point lookup | 1.000ms | 40.643ms | 40.374ms | 0.1% | 40.356ms | 40.399ms | 40.337ms | 40.491ms | 20.000ms | 39.33 MiB | none | 247.69K/s |
| q21 | substring scan | 2.000ms | 40.352ms | 40.344ms | 0.0% | 40.342ms | 40.353ms | 40.309ms | 40.370ms | 20.000ms | 41.08 MiB | none | 247.87K/s |
| q22 | substring scan and group by | 3.000ms | 40.390ms | 40.318ms | 0.0% | 40.314ms | 40.321ms | 40.307ms | 40.404ms | 20.000ms | 42.58 MiB | none | 248.03K/s |
| q23 | two substring scans and group by | 6.000ms | 40.376ms | 40.373ms | 0.2% | 40.365ms | 40.447ms | 40.346ms | 40.561ms | 30.000ms | 48.67 MiB | none | 247.69K/s |
| q24 | select star and top k | 10.000ms | 40.334ms | 40.381ms | 0.1% | 40.349ms | 40.383ms | 40.342ms | 40.406ms | 30.000ms | 52.99 MiB | none | 247.64K/s |
| q25 | top k by a date | 2.000ms | 40.338ms | 40.352ms | 0.0% | 40.341ms | 40.352ms | 40.326ms | 40.353ms | 20.000ms | 41.04 MiB | none | 247.82K/s |
| q26 | top k by a string | 1.000ms | 40.339ms | 40.345ms | 0.0% | 40.340ms | 40.349ms | 40.337ms | 40.353ms | 20.000ms | 40.82 MiB | none | 247.86K/s |
| q27 | top k by two columns | 1.000ms | 40.353ms | 40.360ms | 0.0% | 40.355ms | 40.370ms | 40.343ms | 40.978ms | 20.000ms | 40.58 MiB | none | 247.77K/s |
| q28 | group by with a string length | 4.000ms | 40.364ms | 40.376ms | 0.0% | 40.375ms | 40.395ms | 40.358ms | 40.410ms | 20.000ms | 44.58 MiB | none | 247.67K/s |
| q29 | group by a regular expression | 8.000ms | 40.408ms | 40.358ms | 0.1% | 40.347ms | 40.377ms | 40.335ms | 40.425ms | 20.000ms | 45.61 MiB | none | 247.78K/s |
| q30 | ninety sums over one column | 15.000ms | 40.339ms | 40.344ms | 0.1% | 40.339ms | 40.381ms | 40.335ms | 40.691ms | 30.000ms | 52.96 MiB | none | 247.87K/s |
| q31 | group by two and several aggregates | 4.000ms | 40.397ms | 40.354ms | 0.0% | 40.345ms | 40.360ms | 40.344ms | 40.393ms | 20.000ms | 46.09 MiB | none | 247.81K/s |
| q32 | group by a high card pair | 3.000ms | 40.330ms | 40.370ms | 0.1% | 40.346ms | 40.388ms | 40.345ms | 40.421ms | 20.000ms | 45.62 MiB | none | 247.71K/s |
| q33 | group by a high card pair, unfiltered | 3.000ms | 40.348ms | 40.344ms | 0.0% | 40.342ms | 40.347ms | 40.330ms | 40.361ms | 20.000ms | 46.57 MiB | none | 247.87K/s |
| q34 | group by a long string | 4.000ms | 40.379ms | 40.379ms | 0.2% | 40.345ms | 40.441ms | 40.339ms | 40.468ms | 20.000ms | 45.02 MiB | none | 247.65K/s |
| q35 | group by a constant and a long string | 4.000ms | 40.427ms | 40.320ms | 0.1% | 40.314ms | 40.357ms | 40.293ms | 40.366ms | 20.000ms | 45.64 MiB | none | 248.02K/s |
| q36 | group by four expressions | 3.000ms | 40.454ms | 40.335ms | 0.1% | 40.328ms | 40.361ms | 40.328ms | 40.364ms | 20.000ms | 44.64 MiB | none | 247.92K/s |
| q37 | date range and group by a URL | 3.000ms | 40.371ms | 40.365ms | 0.1% | 40.360ms | 40.394ms | 40.359ms | 40.414ms | 20.000ms | 45.33 MiB | none | 247.74K/s |
| q38 | date range and group by a title | 4.000ms | 40.387ms | 40.391ms | 0.1% | 40.357ms | 40.410ms | 40.345ms | 40.458ms | 20.000ms | 45.08 MiB | none | 247.58K/s |
| q39 | date range, group by and offset | 3.000ms | 40.424ms | 40.462ms | 0.1% | 40.408ms | 40.463ms | 40.353ms | 40.539ms | 20.000ms | 44.08 MiB | none | 247.15K/s |
| q40 | date range, a case and a wide group by | 5.000ms | 40.489ms | 40.363ms | 0.0% | 40.355ms | 40.368ms | 40.345ms | 40.475ms | 20.000ms | 46.51 MiB | none | 247.75K/s |
| q41 | date range with an IN and a hash | 3.000ms | 40.339ms | 40.348ms | 0.1% | 40.345ms | 40.370ms | 40.323ms | 40.458ms | 20.000ms | 44.89 MiB | none | 247.84K/s |
| q42 | date range and a deep offset | 7.000ms | 40.336ms | 40.348ms | 0.1% | 40.346ms | 40.383ms | 40.319ms | 40.391ms | 30.000ms | 45.27 MiB | none | 247.84K/s |
| q43 | minute buckets over a date range | 3.000ms | 40.419ms | 40.352ms | 0.1% | 40.352ms | 40.394ms | 40.332ms | 40.403ms | 20.000ms | 43.56 MiB | none | 247.82K/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 157.000ms by its own clock and 1.736s by ours, 1.737s cold, 950.000ms of CPU, peak 52.99 MiB, 2.74M/s and 675.74 MiB/s.

Running it cost 1006% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.00x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 5.000ms | 80.635ms | 60.412ms | 0.1% | 60.385ms | 60.423ms | 60.375ms | 60.448ms | 60.000ms | 239.00 MiB | none | 165.53K/s |
| q2 | filtered count | 5.000ms | 60.395ms | 60.391ms | 0.0% | 60.381ms | 60.402ms | 60.379ms | 60.409ms | 60.000ms | 240.55 MiB | none | 165.59K/s |
| q3 | three aggregates | 6.000ms | 60.397ms | 60.406ms | 0.0% | 60.390ms | 60.409ms | 60.386ms | 60.445ms | 50.000ms | 241.65 MiB | none | 165.55K/s |
| q4 | average | 5.000ms | 60.404ms | 60.403ms | 0.0% | 60.395ms | 60.407ms | 60.393ms | 60.431ms | 50.000ms | 241.54 MiB | none | 165.55K/s |
| q5 | count distinct, high card | 6.000ms | 60.406ms | 60.405ms | 0.1% | 60.399ms | 60.454ms | 60.395ms | 60.507ms | 60.000ms | 242.91 MiB | none | 165.55K/s |
| q6 | count distinct, strings | 6.000ms | 60.413ms | 60.416ms | 0.0% | 60.411ms | 60.419ms | 60.305ms | 60.425ms | 50.000ms | 241.83 MiB | none | 165.52K/s |
| q7 | min and max of a date | 9.000ms | 60.407ms | 60.406ms | 33.2% | 60.402ms | 80.454ms | 60.384ms | 80.501ms | 60.000ms | 241.18 MiB | none | 165.55K/s |
| q8 | group by, low card | 11.000ms | 80.532ms | 80.508ms | 0.0% | 80.507ms | 80.533ms | 80.484ms | 80.683ms | 60.000ms | 243.13 MiB | none | 124.21K/s |
| q9 | group by and count distinct | 6.000ms | 60.431ms | 60.420ms | 0.0% | 60.396ms | 60.426ms | 60.391ms | 60.476ms | 60.000ms | 243.54 MiB | none | 165.51K/s |
| q10 | group by, several aggregates | 6.000ms | 60.422ms | 60.401ms | 0.0% | 60.400ms | 60.409ms | 60.399ms | 80.481ms | 60.000ms | 244.98 MiB | none | 165.56K/s |
| q11 | group by a string and count distinct | 6.000ms | 60.491ms | 60.409ms | 0.0% | 60.405ms | 60.424ms | 60.394ms | 60.437ms | 60.000ms | 243.57 MiB | none | 165.54K/s |
| q12 | group by two strings and count distinct | 6.000ms | 60.404ms | 60.407ms | 0.0% | 60.396ms | 60.412ms | 60.396ms | 80.546ms | 50.000ms | 245.15 MiB | none | 165.54K/s |
| q13 | group by a string and top k | 6.000ms | 60.456ms | 60.443ms | 0.0% | 60.431ms | 60.452ms | 60.392ms | 80.504ms | 60.000ms | 244.48 MiB | none | 165.45K/s |
| q14 | group by a string and count distinct | 6.000ms | 60.489ms | 60.405ms | 0.0% | 60.402ms | 60.414ms | 60.385ms | 80.464ms | 60.000ms | 245.51 MiB | none | 165.55K/s |
| q15 | group by two columns and top k | 6.000ms | 60.394ms | 60.432ms | 0.2% | 60.414ms | 60.510ms | 60.401ms | 60.537ms | 60.000ms | 245.32 MiB | none | 165.48K/s |
| q16 | group by, very high card | 6.000ms | 60.399ms | 60.414ms | 0.1% | 60.408ms | 60.489ms | 60.395ms | 60.516ms | 60.000ms | 244.29 MiB | none | 165.52K/s |
| q17 | group by two, very high card | 8.000ms | 60.407ms | 60.492ms | 33.2% | 60.415ms | 80.487ms | 60.393ms | 80.525ms | 60.000ms | 246.84 MiB | none | 165.31K/s |
| q18 | group by two, no ordering | 6.000ms | 80.514ms | 60.441ms | 0.0% | 60.426ms | 60.447ms | 60.374ms | 80.566ms | 60.000ms | 244.66 MiB | none | 165.45K/s |
| q19 | group by with an extract | 8.000ms | 60.383ms | 60.486ms | 0.1% | 60.416ms | 60.489ms | 60.412ms | 80.465ms | 60.000ms | 248.51 MiB | none | 165.33K/s |
| q20 | point lookup | 10.000ms | 80.472ms | 80.520ms | 0.1% | 80.510ms | 80.577ms | 80.486ms | 80.589ms | 60.000ms | 241.68 MiB | none | 124.19K/s |
| q21 | substring scan | 7.000ms | 60.494ms | 60.490ms | 33.1% | 60.419ms | 80.465ms | 60.402ms | 80.475ms | 60.000ms | 244.58 MiB | none | 165.32K/s |
| q22 | substring scan and group by | 8.000ms | 60.425ms | 80.491ms | 24.9% | 60.478ms | 80.520ms | 60.388ms | 80.674ms | 60.000ms | 247.02 MiB | none | 124.24K/s |
| q23 | two substring scans and group by | 9.000ms | 60.402ms | 60.443ms | 33.2% | 60.401ms | 80.468ms | 60.398ms | 80.485ms | 60.000ms | 251.21 MiB | none | 165.44K/s |
| q24 | select star and top k | 15.000ms | 80.457ms | 80.503ms | 0.0% | 80.490ms | 80.521ms | 80.456ms | 80.621ms | 70.000ms | 246.48 MiB | none | 124.22K/s |
| q25 | top k by a date | 11.000ms | 80.477ms | 80.500ms | 0.1% | 80.487ms | 80.549ms | 80.480ms | 80.622ms | 60.000ms | 243.97 MiB | none | 124.22K/s |
| q26 | top k by a string | 6.000ms | 60.412ms | 60.418ms | 0.0% | 60.402ms | 60.427ms | 60.402ms | 60.446ms | 60.000ms | 242.55 MiB | none | 165.51K/s |
| q27 | top k by two columns | 10.000ms | 60.415ms | 80.459ms | 24.7% | 60.599ms | 80.460ms | 60.401ms | 80.472ms | 60.000ms | 243.73 MiB | none | 124.29K/s |
| q28 | group by with a string length | 7.000ms | 60.513ms | 60.447ms | 0.2% | 60.411ms | 60.507ms | 60.408ms | 80.544ms | 60.000ms | 245.83 MiB | none | 165.43K/s |
| q29 | group by a regular expression | 12.000ms | 80.458ms | 80.548ms | 0.1% | 80.516ms | 80.576ms | 80.477ms | 80.580ms | 70.000ms | 247.71 MiB | none | 124.15K/s |
| q30 | ninety sums over one column | 9.000ms | 60.418ms | 80.471ms | 24.8% | 60.494ms | 80.475ms | 60.405ms | 80.609ms | 60.000ms | 245.29 MiB | none | 124.27K/s |
| q31 | group by two and several aggregates | 7.000ms | 60.416ms | 60.474ms | 0.1% | 60.418ms | 60.483ms | 60.414ms | 80.529ms | 60.000ms | 245.12 MiB | none | 165.36K/s |
| q32 | group by a high card pair | 7.000ms | 60.408ms | 60.411ms | 0.0% | 60.404ms | 60.419ms | 60.395ms | 60.480ms | 60.000ms | 245.40 MiB | none | 165.53K/s |
| q33 | group by a high card pair, unfiltered | 7.000ms | 60.410ms | 60.435ms | 0.1% | 60.401ms | 60.458ms | 60.390ms | 80.482ms | 50.000ms | 246.87 MiB | none | 165.47K/s |
| q34 | group by a long string | 8.000ms | 60.405ms | 60.421ms | 0.1% | 60.408ms | 60.469ms | 60.396ms | 80.481ms | 60.000ms | 247.72 MiB | none | 165.51K/s |
| q35 | group by a constant and a long string | 8.000ms | 60.461ms | 80.485ms | 24.9% | 60.489ms | 80.493ms | 60.464ms | 80.601ms | 60.000ms | 247.29 MiB | none | 124.25K/s |
| q36 | group by four expressions | 7.000ms | 60.423ms | 60.427ms | 0.1% | 60.410ms | 60.475ms | 60.405ms | 61.689ms | 50.000ms | 245.21 MiB | none | 165.49K/s |
| q37 | date range and group by a URL | 13.000ms | 80.470ms | 80.495ms | 0.0% | 80.494ms | 80.497ms | 80.487ms | 80.634ms | 60.000ms | 249.72 MiB | none | 124.23K/s |
| q38 | date range and group by a title | 13.000ms | 80.610ms | 80.577ms | 0.1% | 80.527ms | 80.579ms | 80.518ms | 80.581ms | 70.000ms | 249.05 MiB | none | 124.11K/s |
| q39 | date range, group by and offset | 13.000ms | 80.564ms | 80.524ms | 0.1% | 80.505ms | 80.558ms | 80.466ms | 80.570ms | 70.000ms | 249.14 MiB | none | 124.19K/s |
| q40 | date range, a case and a wide group by | 14.000ms | 80.526ms | 80.480ms | 0.1% | 80.476ms | 80.519ms | 80.466ms | 80.579ms | 60.000ms | 250.27 MiB | none | 124.25K/s |
| q41 | date range with an IN and a hash | 12.000ms | 80.485ms | 80.505ms | 0.0% | 80.504ms | 80.526ms | 80.487ms | 80.569ms | 70.000ms | 247.79 MiB | none | 124.22K/s |
| q42 | date range and a deep offset | 12.000ms | 80.489ms | 80.578ms | 0.2% | 80.522ms | 80.645ms | 80.506ms | 80.675ms | 60.000ms | 247.48 MiB | none | 124.10K/s |
| q43 | minute buckets over a date range | 12.000ms | 80.546ms | 80.559ms | 0.0% | 80.557ms | 80.572ms | 80.554ms | 80.771ms | 70.000ms | 247.26 MiB | none | 124.13K/s |

clickhouse-local 26.9.1.1562 over 43 of 43 queries. Total 360.000ms by its own clock and 2.920s by ours, 2.880s cold, 2.580s of CPU, peak 251.21 MiB, 1.19M/s and 294.70 MiB/s.

Running it cost 711% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.33x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 20.284ms | 20.257ms | 0.3% | 20.241ms | 20.298ms | 20.232ms | 20.306ms | 10.000ms | 80.31 MiB | none | 493.67K/s |
| q2 | filtered count | 5.000ms | 20.378ms | 20.390ms | 0.3% | 20.334ms | 20.392ms | 20.246ms | 20.393ms | 30.000ms | 131.55 MiB | none | 490.44K/s |
| q3 | three aggregates | 4.000ms | 20.249ms | 20.262ms | 0.1% | 20.254ms | 20.270ms | 20.249ms | 20.284ms | 20.000ms | 128.27 MiB | none | 493.53K/s |
| q4 | average | 3.000ms | 20.275ms | 20.285ms | 0.1% | 20.276ms | 20.289ms | 20.275ms | 20.296ms | 20.000ms | 115.22 MiB | none | 492.97K/s |
| q5 | count distinct, high card | 6.000ms | 20.277ms | 20.272ms | 0.1% | 20.268ms | 20.289ms | 20.267ms | 20.312ms | 40.000ms | 140.88 MiB | none | 493.29K/s |
| q6 | count distinct, strings | 6.000ms | 20.292ms | 20.280ms | 0.0% | 20.277ms | 20.281ms | 20.275ms | 20.318ms | 40.000ms | 157.16 MiB | none | 493.09K/s |
| q7 | min and max of a date | 1.000ms | 20.286ms | 20.286ms | 0.0% | 20.285ms | 20.291ms | 20.275ms | 20.308ms | 0.000us | 80.13 MiB | none | 492.96K/s |
| q8 | group by, low card | 5.000ms | 20.282ms | 20.273ms | 0.0% | 20.271ms | 20.274ms | 20.269ms | 20.296ms | 20.000ms | 125.29 MiB | none | 493.27K/s |
| q9 | group by and count distinct | 9.000ms | 20.299ms | 20.289ms | 99.0% | 20.277ms | 40.366ms | 20.277ms | 40.370ms | 120.000ms | 167.04 MiB | none | 492.87K/s |
| q10 | group by, several aggregates | 7.000ms | 20.281ms | 20.273ms | 0.1% | 20.267ms | 20.289ms | 20.261ms | 40.361ms | 60.000ms | 161.47 MiB | none | 493.26K/s |
| q11 | group by a string and count distinct | 9.000ms | 40.422ms | 20.417ms | 98.2% | 20.302ms | 40.350ms | 20.286ms | 40.368ms | 120.000ms | 185.14 MiB | none | 489.79K/s |
| q12 | group by two strings and count distinct | 8.000ms | 20.528ms | 20.264ms | 0.1% | 20.258ms | 20.272ms | 20.252ms | 20.276ms | 50.000ms | 188.62 MiB | none | 493.48K/s |
| q13 | group by a string and top k | 7.000ms | 20.255ms | 20.316ms | 0.3% | 20.263ms | 20.333ms | 20.260ms | 20.351ms | 60.000ms | 174.61 MiB | none | 492.21K/s |
| q14 | group by a string and count distinct | 9.000ms | 40.389ms | 20.268ms | 0.2% | 20.267ms | 20.309ms | 20.266ms | 40.461ms | 50.000ms | 192.25 MiB | none | 493.40K/s |
| q15 | group by two columns and top k | 7.000ms | 20.408ms | 20.275ms | 0.0% | 20.275ms | 20.279ms | 20.260ms | 20.433ms | 40.000ms | 158.55 MiB | none | 493.22K/s |
| q16 | group by, very high card | 5.000ms | 20.266ms | 20.338ms | 0.3% | 20.291ms | 20.346ms | 20.273ms | 20.387ms | 30.000ms | 137.66 MiB | none | 491.69K/s |
| q17 | group by two, very high card | 6.000ms | 20.286ms | 20.283ms | 0.0% | 20.281ms | 20.283ms | 20.265ms | 20.285ms | 30.000ms | 158.57 MiB | none | 493.02K/s |
| q18 | group by two, no ordering | 6.000ms | 20.279ms | 20.265ms | 0.0% | 20.260ms | 20.267ms | 20.252ms | 20.348ms | 30.000ms | 157.22 MiB | none | 493.46K/s |
| q19 | group by with an extract | 7.000ms | 20.252ms | 20.278ms | 0.1% | 20.268ms | 20.281ms | 20.263ms | 20.282ms | 30.000ms | 154.02 MiB | none | 493.15K/s |
| q20 | point lookup | 3.000ms | 20.400ms | 20.286ms | 0.6% | 20.270ms | 20.402ms | 20.265ms | 20.427ms | 10.000ms | 119.58 MiB | none | 492.95K/s |
| q21 | substring scan | 4.000ms | 20.252ms | 20.249ms | 0.0% | 20.245ms | 20.251ms | 20.241ms | 20.303ms | 10.000ms | 119.40 MiB | none | 493.86K/s |
| q22 | substring scan and group by | 6.000ms | 20.257ms | 20.270ms | 0.7% | 20.264ms | 20.405ms | 20.254ms | 20.479ms | 30.000ms | 150.02 MiB | none | 493.35K/s |
| q23 | two substring scans and group by | 9.000ms | 20.265ms | 20.271ms | 0.0% | 20.270ms | 20.274ms | 20.261ms | 40.343ms | 60.000ms | 174.73 MiB | none | 493.32K/s |
| q24 | select star and top k | 12.000ms | 40.344ms | 40.371ms | 0.3% | 40.344ms | 40.482ms | 40.335ms | 40.546ms | 30.000ms | 150.96 MiB | none | 247.70K/s |
| q25 | top k by a date | 4.000ms | 20.303ms | 20.250ms | 0.0% | 20.246ms | 20.253ms | 20.246ms | 20.402ms | 10.000ms | 124.39 MiB | none | 493.82K/s |
| q26 | top k by a string | 4.000ms | 20.253ms | 20.300ms | 0.5% | 20.261ms | 20.371ms | 20.256ms | 20.404ms | 20.000ms | 117.78 MiB | none | 492.61K/s |
| q27 | top k by two columns | 5.000ms | 20.248ms | 20.265ms | 0.1% | 20.253ms | 20.273ms | 20.246ms | 20.413ms | 10.000ms | 126.31 MiB | none | 493.47K/s |
| q28 | group by with a string length | 6.000ms | 20.249ms | 20.267ms | 0.6% | 20.262ms | 20.390ms | 20.256ms | 20.413ms | 30.000ms | 159.67 MiB | none | 493.42K/s |
| q29 | group by a regular expression | 10.000ms | 40.470ms | 40.339ms | 0.0% | 40.335ms | 40.340ms | 20.267ms | 40.504ms | 50.000ms | 199.03 MiB | none | 247.90K/s |
| q30 | ninety sums over one column | 11.000ms | 40.628ms | 40.373ms | 0.6% | 40.368ms | 40.598ms | 40.332ms | 40.859ms | 20.000ms | 104.85 MiB | none | 247.69K/s |
| q31 | group by two and several aggregates | 7.000ms | 20.240ms | 20.260ms | 0.4% | 20.259ms | 20.338ms | 20.252ms | 20.406ms | 30.000ms | 155.67 MiB | none | 493.58K/s |
| q32 | group by a high card pair | 7.000ms | 20.391ms | 20.398ms | 0.5% | 20.293ms | 20.400ms | 20.264ms | 20.405ms | 40.000ms | 162.79 MiB | none | 490.25K/s |
| q33 | group by a high card pair, unfiltered | 6.000ms | 20.269ms | 20.394ms | 0.7% | 20.261ms | 20.401ms | 20.253ms | 20.403ms | 20.000ms | 142.10 MiB | none | 490.34K/s |
| q34 | group by a long string | 7.000ms | 20.356ms | 20.327ms | 0.6% | 20.277ms | 20.399ms | 20.265ms | 20.415ms | 30.000ms | 169.01 MiB | none | 491.96K/s |
| q35 | group by a constant and a long string | 7.000ms | 20.273ms | 20.396ms | 0.6% | 20.267ms | 20.396ms | 20.256ms | 20.409ms | 30.000ms | 186.29 MiB | none | 490.30K/s |
| q36 | group by four expressions | 5.000ms | 20.397ms | 20.284ms | 0.5% | 20.260ms | 20.356ms | 20.252ms | 20.400ms | 20.000ms | 140.73 MiB | none | 493.00K/s |
| q37 | date range and group by a URL | 7.000ms | 20.252ms | 20.356ms | 0.4% | 20.323ms | 20.401ms | 20.261ms | 20.409ms | 20.000ms | 152.34 MiB | none | 491.26K/s |
| q38 | date range and group by a title | 9.000ms | 20.262ms | 20.272ms | 0.4% | 20.267ms | 20.343ms | 20.259ms | 40.482ms | 50.000ms | 158.25 MiB | none | 493.29K/s |
| q39 | date range, group by and offset | 8.000ms | 20.252ms | 20.278ms | 0.2% | 20.256ms | 20.306ms | 20.256ms | 20.403ms | 80.000ms | 151.80 MiB | none | 493.14K/s |
| q40 | date range, a case and a wide group by | 9.000ms | 20.343ms | 40.351ms | 0.1% | 40.343ms | 40.369ms | 20.255ms | 40.533ms | 80.000ms | 156.16 MiB | none | 247.83K/s |
| q41 | date range with an IN and a hash | 6.000ms | 20.269ms | 20.261ms | 0.1% | 20.254ms | 20.271ms | 20.250ms | 20.450ms | 30.000ms | 132.20 MiB | none | 493.56K/s |
| q42 | date range and a deep offset | 6.000ms | 20.389ms | 20.334ms | 0.6% | 20.257ms | 20.388ms | 20.252ms | 20.399ms | 30.000ms | 131.23 MiB | none | 491.79K/s |
| q43 | minute buckets over a date range | 6.000ms | 20.260ms | 20.254ms | 0.3% | 20.252ms | 20.314ms | 20.246ms | 20.396ms | 20.000ms | 132.54 MiB | none | 493.74K/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 275.000ms by its own clock and 952.976ms by ours, 973.612ms cold, 1.560s of CPU, peak 199.03 MiB, 1.56M/s and 385.78 MiB/s.

Running it cost 247% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.99x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 5.930ms | 100.632ms | 100.653ms | 0.2% | 100.610ms | 100.802ms | 100.604ms | 100.910ms | 190.000ms | 61.75 MiB | none | 99.35K/s |
| q2 | filtered count | 6.228ms | 100.563ms | 100.635ms | 0.1% | 100.627ms | 100.747ms | 100.565ms | 121.301ms | 220.000ms | 66.72 MiB | none | 99.37K/s |
| q3 | three aggregates | 6.103ms | 121.192ms | 100.612ms | 0.1% | 100.578ms | 100.686ms | 100.577ms | 100.708ms | 90.000ms | 65.69 MiB | none | 99.39K/s |
| q4 | average | 6.565ms | 100.634ms | 100.688ms | 0.2% | 100.616ms | 100.775ms | 100.610ms | 120.757ms | 100.000ms | 63.77 MiB | none | 99.32K/s |
| q5 | count distinct, high card | 12.357ms | 128.290ms | 100.677ms | 0.0% | 100.674ms | 100.695ms | 100.578ms | 131.773ms | 130.000ms | 74.60 MiB | none | 99.33K/s |
| q6 | count distinct, strings | 12.849ms | 100.713ms | 120.812ms | 17.4% | 100.601ms | 121.611ms | 100.551ms | 129.311ms | 170.000ms | 74.18 MiB | none | 82.77K/s |
| q7 | min and max of a date | 6.479ms | 100.591ms | 100.781ms | 0.1% | 100.770ms | 100.914ms | 100.734ms | 122.399ms | 220.000ms | 65.08 MiB | none | 99.22K/s |
| q8 | group by, low card | 12.842ms | 122.456ms | 100.671ms | 19.9% | 100.620ms | 120.693ms | 100.619ms | 120.753ms | 120.000ms | 75.54 MiB | none | 99.33K/s |
| q9 | group by and count distinct | 21.591ms | 120.686ms | 120.723ms | 0.0% | 120.719ms | 120.768ms | 100.703ms | 121.078ms | 220.000ms | 83.71 MiB | none | 82.83K/s |
| q10 | group by, several aggregates | 27.009ms | 121.057ms | 120.738ms | 0.1% | 120.736ms | 120.865ms | 120.641ms | 120.885ms | 210.000ms | 85.59 MiB | none | 82.82K/s |
| q11 | group by a string and count distinct | 20.092ms | 120.753ms | 120.817ms | 0.7% | 120.708ms | 121.535ms | 120.699ms | 121.673ms | 140.000ms | 82.93 MiB | none | 82.77K/s |
| q12 | group by two strings and count distinct | 22.252ms | 121.120ms | 120.970ms | 0.2% | 120.897ms | 121.112ms | 120.710ms | 140.775ms | 160.000ms | 84.23 MiB | none | 82.67K/s |
| q13 | group by a string and top k | 14.291ms | 100.649ms | 100.676ms | 20.0% | 100.618ms | 120.713ms | 100.591ms | 132.075ms | 130.000ms | 77.08 MiB | none | 99.33K/s |
| q14 | group by a string and count distinct | 20.877ms | 120.817ms | 120.864ms | 0.4% | 120.789ms | 121.248ms | 120.650ms | 141.054ms | 170.000ms | 83.89 MiB | none | 82.74K/s |
| q15 | group by two columns and top k | 15.411ms | 100.706ms | 100.647ms | 20.4% | 100.644ms | 121.161ms | 100.559ms | 132.841ms | 140.000ms | 77.61 MiB | none | 99.36K/s |
| q16 | group by, very high card | 17.553ms | 120.696ms | 121.233ms | 0.5% | 120.762ms | 121.424ms | 120.655ms | 126.000ms | 140.000ms | 75.82 MiB | none | 82.49K/s |
| q17 | group by two, very high card | 22.012ms | 120.750ms | 120.685ms | 0.1% | 120.673ms | 120.748ms | 120.619ms | 120.768ms | 150.000ms | 78.96 MiB | none | 82.86K/s |
| q18 | group by two, no ordering | 13.069ms | 100.719ms | 100.671ms | 0.2% | 100.563ms | 100.727ms | 100.561ms | 120.671ms | 120.000ms | 76.66 MiB | none | 99.33K/s |
| q19 | group by with an extract | 23.882ms | 120.744ms | 120.834ms | 0.1% | 120.755ms | 120.849ms | 120.720ms | 120.854ms | 150.000ms | 80.38 MiB | none | 82.76K/s |
| q20 | point lookup | 5.523ms | 100.602ms | 100.754ms | 0.3% | 100.654ms | 100.931ms | 100.627ms | 122.601ms | 230.000ms | 63.55 MiB | none | 99.25K/s |
| q21 | substring scan | 7.684ms | 100.676ms | 100.794ms | 19.9% | 100.626ms | 120.719ms | 100.552ms | 124.852ms | 190.000ms | 69.01 MiB | none | 99.21K/s |
| q22 | substring scan and group by | 14.696ms | 100.602ms | 100.666ms | 0.1% | 100.631ms | 100.749ms | 100.538ms | 100.782ms | 130.000ms | 79.23 MiB | none | 99.34K/s |
| q23 | two substring scans and group by | 20.999ms | 120.755ms | 121.085ms | 1.1% | 120.761ms | 122.077ms | 120.711ms | 142.348ms | 210.000ms | 88.75 MiB | none | 82.59K/s |
| q24 | select star and top k | 11.157ms | 100.580ms | 100.773ms | 0.2% | 100.652ms | 100.848ms | 100.557ms | 120.863ms | 130.000ms | 76.91 MiB | none | 99.23K/s |
| q25 | top k by a date | 9.421ms | 100.608ms | 100.786ms | 25.5% | 100.687ms | 126.348ms | 100.561ms | 126.490ms | 150.000ms | 72.23 MiB | none | 99.22K/s |
| q26 | top k by a string | 9.885ms | 129.218ms | 127.518ms | 21.2% | 100.699ms | 127.673ms | 100.595ms | 129.454ms | 320.000ms | 72.05 MiB | none | 78.42K/s |
| q27 | top k by two columns | 11.331ms | 100.758ms | 100.643ms | 0.1% | 100.640ms | 100.734ms | 100.591ms | 123.694ms | 110.000ms | 73.00 MiB | none | 99.36K/s |
| q30 | ninety sums over one column | 10.149ms | 129.703ms | 100.588ms | 0.2% | 100.573ms | 100.731ms | 100.568ms | 120.697ms | 120.000ms | 68.54 MiB | none | 99.42K/s |
| q31 | group by two and several aggregates | 14.506ms | 124.468ms | 120.788ms | 1.0% | 120.700ms | 121.934ms | 100.657ms | 123.604ms | 140.000ms | 80.21 MiB | none | 82.79K/s |
| q32 | group by a high card pair | 14.889ms | 100.592ms | 121.851ms | 1.6% | 120.677ms | 122.678ms | 100.802ms | 126.283ms | 160.000ms | 80.12 MiB | none | 82.07K/s |
| q33 | group by a high card pair, unfiltered | 15.509ms | 120.620ms | 120.720ms | 16.7% | 100.792ms | 120.993ms | 100.581ms | 121.196ms | 130.000ms | 81.39 MiB | none | 82.84K/s |
| q34 | group by a long string | 17.460ms | 120.726ms | 120.900ms | 0.3% | 120.795ms | 121.201ms | 100.634ms | 128.277ms | 190.000ms | 80.66 MiB | none | 82.71K/s |
| q35 | group by a constant and a long string | 19.416ms | 121.192ms | 121.024ms | 0.5% | 120.914ms | 121.568ms | 120.882ms | 123.051ms | 170.000ms | 82.52 MiB | none | 82.63K/s |
| q37 | date range and group by a URL | 17.523ms | 123.112ms | 120.670ms | 0.1% | 120.669ms | 120.806ms | 100.531ms | 121.074ms | 170.000ms | 80.74 MiB | none | 82.87K/s |
| q38 | date range and group by a title | 17.371ms | 121.797ms | 120.649ms | 16.4% | 100.901ms | 120.741ms | 100.728ms | 120.791ms | 180.000ms | 81.79 MiB | none | 82.88K/s |
| q39 | date range, group by and offset | 14.419ms | 100.627ms | 100.826ms | 20.3% | 100.554ms | 120.972ms | 100.526ms | 131.716ms | 140.000ms | 79.41 MiB | none | 99.18K/s |
| q40 | date range, a case and a wide group by | 14.941ms | 126.409ms | 100.641ms | 0.1% | 100.586ms | 100.703ms | 100.516ms | 120.914ms | 150.000ms | 80.18 MiB | none | 99.36K/s |
| q41 | date range with an IN and a hash | 14.239ms | 100.518ms | 120.731ms | 16.7% | 100.659ms | 120.860ms | 100.591ms | 120.946ms | 130.000ms | 80.64 MiB | none | 82.83K/s |
| q42 | date range and a deep offset | 13.318ms | 120.820ms | 100.632ms | 0.1% | 100.619ms | 100.696ms | 100.611ms | 125.152ms | 130.000ms | 79.38 MiB | none | 99.37K/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 561.828ms by its own clock and 4.317s by ours, 4.408s cold, 6.250s of CPU, peak 88.75 MiB, 694.16K/s and 171.27 MiB/s.

Running it cost 668% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.27x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 747.448us | 20.344ms | 20.303ms | 0.1% | 20.287ms | 20.313ms | 20.271ms | 20.342ms | 0.000us | 10.49 MiB | none | 492.53K/s |
| q2 | filtered count | 870.474us | 20.343ms | 20.374ms | 0.4% | 20.330ms | 20.412ms | 20.328ms | 20.433ms | 0.000us | 10.74 MiB | none | 490.81K/s |
| q3 | three aggregates | 822.376us | 20.345ms | 20.347ms | 0.0% | 20.347ms | 20.347ms | 20.337ms | 20.351ms | 0.000us | 10.50 MiB | none | 491.48K/s |
| q4 | average | 911.195us | 20.362ms | 20.349ms | 0.0% | 20.343ms | 20.350ms | 20.336ms | 20.414ms | 0.000us | 11.22 MiB | none | 491.42K/s |
| q5 | count distinct, high card | 890.716us | 20.324ms | 20.390ms | 0.3% | 20.342ms | 20.397ms | 20.337ms | 20.427ms | 0.000us | 11.00 MiB | none | 490.43K/s |
| q6 | count distinct, strings | 1.254ms | 20.338ms | 20.348ms | 0.1% | 20.340ms | 20.367ms | 20.330ms | 20.399ms | 0.000us | 11.95 MiB | none | 491.45K/s |
| q7 | min and max of a date | 832.774us | 20.340ms | 20.315ms | 0.1% | 20.312ms | 20.328ms | 20.297ms | 20.349ms | 0.000us | 10.95 MiB | none | 492.26K/s |
| q8 | group by, low card | 826.978us | 20.354ms | 20.298ms | 0.0% | 20.297ms | 20.305ms | 20.297ms | 20.335ms | 0.000us | 10.95 MiB | none | 492.66K/s |
| q9 | group by and count distinct | 1.168ms | 20.281ms | 20.335ms | 0.1% | 20.324ms | 20.342ms | 20.310ms | 20.347ms | 0.000us | 11.68 MiB | none | 491.77K/s |
| q10 | group by, several aggregates | 1.710ms | 20.355ms | 20.371ms | 0.3% | 20.326ms | 20.397ms | 20.322ms | 20.423ms | 0.000us | 12.94 MiB | none | 490.88K/s |
| q11 | group by a string and count distinct | 933.609us | 20.343ms | 20.392ms | 0.3% | 20.340ms | 20.397ms | 20.317ms | 20.455ms | 0.000us | 11.20 MiB | none | 490.39K/s |
| q12 | group by two strings and count distinct | 988.957us | 20.345ms | 20.329ms | 0.1% | 20.325ms | 20.344ms | 20.297ms | 20.463ms | 0.000us | 11.27 MiB | none | 491.91K/s |
| q13 | group by a string and top k | 1.110ms | 20.329ms | 20.312ms | 0.1% | 20.302ms | 20.331ms | 20.296ms | 20.347ms | 0.000us | 11.75 MiB | none | 492.32K/s |
| q14 | group by a string and count distinct | 1.233ms | 20.305ms | 20.297ms | 0.2% | 20.293ms | 20.326ms | 20.281ms | 20.618ms | 0.000us | 11.73 MiB | none | 492.69K/s |
| q15 | group by two columns and top k | 1.172ms | 20.305ms | 20.353ms | 0.2% | 20.310ms | 20.361ms | 20.297ms | 20.692ms | 0.000us | 11.42 MiB | none | 491.34K/s |
| q16 | group by, very high card | 1.849ms | 20.358ms | 20.309ms | 0.1% | 20.307ms | 20.320ms | 20.290ms | 20.330ms | 0.000us | 12.27 MiB | none | 492.39K/s |
| q17 | group by two, very high card | 2.703ms | 20.282ms | 20.362ms | 0.4% | 20.307ms | 20.379ms | 20.305ms | 20.441ms | 0.000us | 13.68 MiB | none | 491.11K/s |
| q18 | group by two, no ordering | 1.185ms | 20.338ms | 20.351ms | 0.3% | 20.337ms | 20.389ms | 20.319ms | 20.430ms | 0.000us | 11.02 MiB | none | 491.38K/s |
| q19 | group by with an extract | 3.072ms | 20.346ms | 20.339ms | 0.0% | 20.336ms | 20.340ms | 20.333ms | 20.351ms | 0.000us | 14.68 MiB | none | 491.67K/s |
| q20 | point lookup | 700.133us | 20.303ms | 20.314ms | 0.1% | 20.306ms | 20.320ms | 20.304ms | 20.330ms | 0.000us | 10.72 MiB | none | 492.26K/s |
| q21 | substring scan | 2.108ms | 20.314ms | 20.332ms | 0.1% | 20.324ms | 20.351ms | 20.312ms | 20.363ms | 0.000us | 13.19 MiB | none | 491.82K/s |
| q22 | substring scan and group by | 2.402ms | 20.352ms | 20.359ms | 0.0% | 20.357ms | 20.362ms | 20.348ms | 20.392ms | 0.000us | 13.69 MiB | none | 491.17K/s |
| q23 | two substring scans and group by | 4.518ms | 20.346ms | 20.355ms | 0.1% | 20.341ms | 20.355ms | 20.327ms | 20.355ms | 0.000us | 16.02 MiB | none | 491.28K/s |
| q24 | select star and top k | 2.538ms | 20.330ms | 20.341ms | 0.1% | 20.340ms | 20.358ms | 20.331ms | 20.374ms | 0.000us | 14.51 MiB | none | 491.62K/s |
| q25 | top k by a date | 1.057ms | 20.351ms | 20.429ms | 0.9% | 20.315ms | 20.496ms | 20.304ms | 20.575ms | 0.000us | 11.45 MiB | none | 489.51K/s |
| q26 | top k by a string | 1.034ms | 20.373ms | 20.392ms | 0.1% | 20.374ms | 20.396ms | 20.374ms | 20.432ms | 0.000us | 10.76 MiB | none | 490.40K/s |
| q27 | top k by two columns | 1.123ms | 20.379ms | 20.353ms | 0.2% | 20.332ms | 20.372ms | 20.311ms | 20.391ms | 0.000us | 11.52 MiB | none | 491.32K/s |
| q28 | group by with a string length | 2.320ms | 20.303ms | 20.336ms | 0.0% | 20.330ms | 20.340ms | 20.325ms | 20.448ms | 0.000us | 13.96 MiB | none | 491.73K/s |
| q29 | group by a regular expression | 2.938ms | 20.331ms | 20.375ms | 0.1% | 20.371ms | 20.400ms | 20.367ms | 20.490ms | 0.000us | 14.64 MiB | none | 490.80K/s |
| q30 | ninety sums over one column | 2.129ms | 20.334ms | 20.328ms | 0.1% | 20.312ms | 20.332ms | 20.306ms | 20.352ms | 0.000us | 11.16 MiB | none | 491.93K/s |
| q31 | group by two and several aggregates | 1.427ms | 20.304ms | 20.341ms | 0.1% | 20.335ms | 20.352ms | 20.331ms | 20.353ms | 0.000us | 11.75 MiB | none | 491.63K/s |
| q32 | group by a high card pair | 1.408ms | 20.318ms | 20.331ms | 0.0% | 20.330ms | 20.336ms | 20.316ms | 20.338ms | 0.000us | 12.00 MiB | none | 491.86K/s |
| q33 | group by a high card pair, unfiltered | 1.431ms | 20.304ms | 20.320ms | 0.1% | 20.319ms | 20.341ms | 20.310ms | 20.343ms | 0.000us | 12.20 MiB | none | 492.12K/s |
| q34 | group by a long string | 4.309ms | 20.336ms | 20.397ms | 0.3% | 20.340ms | 20.405ms | 20.336ms | 20.416ms | 0.000us | 18.45 MiB | none | 490.27K/s |
| q35 | group by a constant and a long string | 4.214ms | 20.329ms | 20.332ms | 0.2% | 20.323ms | 20.364ms | 20.314ms | 20.414ms | 0.000us | 18.45 MiB | none | 491.84K/s |
| q36 | group by four expressions | 1.823ms | 20.341ms | 20.338ms | 0.4% | 20.292ms | 20.380ms | 20.285ms | 20.422ms | 0.000us | 12.43 MiB | none | 491.69K/s |
| q37 | date range and group by a URL | 2.120ms | 20.313ms | 20.303ms | 0.0% | 20.299ms | 20.304ms | 20.287ms | 20.331ms | 0.000us | 13.48 MiB | none | 492.54K/s |
| q38 | date range and group by a title | 2.892ms | 20.310ms | 20.336ms | 0.2% | 20.306ms | 20.350ms | 20.272ms | 20.366ms | 0.000us | 13.73 MiB | none | 491.75K/s |
| q39 | date range, group by and offset | 2.247ms | 20.368ms | 20.332ms | 0.3% | 20.319ms | 20.386ms | 20.303ms | 20.423ms | 0.000us | 13.47 MiB | none | 491.84K/s |
| q40 | date range, a case and a wide group by | 3.436ms | 20.329ms | 20.322ms | 0.3% | 20.313ms | 20.375ms | 20.301ms | 20.385ms | 0.000us | 15.00 MiB | none | 492.08K/s |
| q41 | date range with an IN and a hash | 1.283ms | 20.310ms | 20.361ms | 0.1% | 20.354ms | 20.377ms | 20.335ms | 20.379ms | 0.000us | 11.74 MiB | none | 491.14K/s |
| q42 | date range and a deep offset | 1.092ms | 20.334ms | 20.298ms | 0.3% | 20.278ms | 20.335ms | 20.275ms | 20.358ms | 0.000us | 11.71 MiB | none | 492.67K/s |
| q43 | minute buckets over a date range | 1.015ms | 20.288ms | 20.311ms | 0.1% | 20.303ms | 20.318ms | 20.296ms | 20.329ms | 0.000us | 11.64 MiB | none | 492.34K/s |

rudb rudb 0.3.67 over 43 of 43 queries. Total 75.844ms by its own clock and 874.709ms by ours, 874.240ms cold, 0.000us of CPU, peak 18.45 MiB, 5.67M/s and 1.37 GiB/s.

Running it cost 1053% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.01x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | 434.999us | 196.186us | 165.869us | 179.475us | 7.6% | 165.869us | 147.710us | 0.000us | 896 B | 4 of 4 |
| q2 | 452.770us | 248.087us | 266.469us | 281.734us | 5.4% | 266.469us | 155.387us | 0.000us | 896 B | 5 of 5 |
| q3 | 429.955us | 291.788us | 313.005us | 326.668us | 4.2% | 313.005us | 135.451us | 0.000us | 1.22 KiB | 4 of 4 |
| q4 | 423.950us | 273.332us | 361.526us | 373.170us | 3.1% | 361.526us | 139.633us | 0.000us | 896 B | 4 of 4 |
| q5 | 429.135us | 376.631us | 451.222us | 462.179us | 2.4% | 451.222us | 135.137us | 0.000us | 114.81 KiB | 4 of 4 |
| q6 | 443.388us | 624.112us | 794.216us | 806.179us | 1.5% | 794.216us | 164.947us | 0.000us | 348.56 KiB | 5 of 5 |
| q7 | 494.232us | 274.974us | 294.943us | 308.417us | 4.4% | 294.943us | 149.501us | 0.000us | 1.00 KiB | 4 of 4 |
| q8 | 457.574us | 248.747us | 273.904us | 286.074us | 4.3% | 273.904us | 147.621us | 0.000us | 2.31 KiB | 6 of 6 |
| q9 | 461.366us | 562.517us | 699.086us | 711.107us | 1.7% | 699.086us | 149.714us | 0.000us | 627.09 KiB | 5 of 5 |
| q10 | 476.231us | 1.029ms | 1.292ms | 1.312ms | 1.5% | 1.292ms | 143.700us | 0.000us | 911.03 KiB | 5 of 5 |
| q11 | 494.666us | 327.413us | 446.406us | 459.242us | 2.8% | 446.406us | 148.410us | 0.000us | 12.83 KiB | 6 of 6 |
| q12 | 524.703us | 412.766us | 551.890us | 579.411us | 4.7% | 551.890us | 163.636us | 0.000us | 15.45 KiB | 6 of 6 |
| q13 | 472.573us | 545.274us | 727.570us | 740.882us | 1.8% | 727.570us | 146.340us | 0.000us | 190.28 KiB | 6 of 6 |
| q14 | 424.630us | 599.950us | 781.925us | 794.719us | 1.6% | 781.925us | 136.752us | 0.000us | 358.25 KiB | 6 of 6 |
| q15 | 430.396us | 530.218us | 700.802us | 713.000us | 1.7% | 700.802us | 137.587us | 0.000us | 195.06 KiB | 6 of 6 |
| q16 | 459.873us | 1.334ms | 1.514ms | 1.530ms | 1.1% | 1.514ms | 146.313us | 0.000us | 963.00 KiB | 5 of 5 |
| q17 | 436.922us | 1.903ms | 2.143ms | 2.153ms | 0.4% | 2.143ms | 136.493us | 0.000us | 1.22 MiB | 5 of 5 |
| q18 | 452.804us | 584.237us | 733.197us | 749.079us | 2.1% | 733.197us | 150.520us | 0.000us | 2.54 KiB | 5 of 5 |
| q19 | 484.314us | 2.474ms | 2.746ms | 2.763ms | 0.6% | 2.746ms | 158.924us | 0.000us | 1.54 MiB | 5 of 5 |
| q20 | 406.835us | 191.968us | 265.772us | 275.764us | 3.6% | 265.772us | 116.061us | 0.000us | 0 B | 4 of 4 |
| q21 | 437.164us | 1.512ms | 2.076ms | 2.087ms | 0.5% | 2.076ms | 146.822us | 0.000us | 896 B | 5 of 5 |
| q22 | 510.654us | 1.760ms | 2.181ms | 2.191ms | 0.5% | 2.181ms | 172.313us | 0.000us | 1.00 KiB | 6 of 6 |
| q23 | 519.144us | 3.783ms | 4.574ms | 4.590ms | 0.3% | 4.574ms | 172.552us | 0.000us | 3.44 KiB | 6 of 6 |
| q24 | 618.158us | 2.106ms | 2.514ms | 2.523ms | 0.4% | 2.514ms | 213.458us | 0.000us | 0 B | 8 of 8 |
| q25 | 443.474us | 463.524us | 634.212us | 644.036us | 1.5% | 634.212us | 124.016us | 0.000us | 7.66 KiB | 6 of 6 |
| q26 | 495.064us | 426.777us | 581.264us | 592.142us | 1.8% | 581.264us | 156.591us | 0.000us | 7.10 KiB | 5 of 5 |
| q27 | 459.765us | 509.150us | 696.944us | 709.158us | 1.7% | 696.944us | 156.154us | 0.000us | 10.02 KiB | 6 of 6 |
| q28 | 503.906us | 1.616ms | 2.026ms | 2.035ms | 0.4% | 2.026ms | 147.921us | 0.000us | 115.16 KiB | 7 of 7 |
| q29 | 551.547us | 2.260ms | 2.849ms | 2.858ms | 0.3% | 2.849ms | 175.004us | 0.000us | 530.81 KiB | 7 of 7 |
| q30 | 1.696ms | 248.667us | 252.944us | 269.214us | 6.0% | 252.944us | 167.451us | 0.000us | 29.59 KiB | 4 of 4 |
| q31 | 484.840us | 703.162us | 889.788us | 906.360us | 1.8% | 889.788us | 144.952us | 0.000us | 110.27 KiB | 6 of 6 |
| q32 | 497.637us | 759.483us | 945.395us | 961.611us | 1.7% | 945.395us | 170.317us | 0.000us | 116.05 KiB | 6 of 6 |
| q33 | 486.315us | 810.209us | 975.791us | 989.297us | 1.4% | 975.791us | 140.637us | 0.000us | 324.52 KiB | 5 of 5 |
| q34 | 488.174us | 3.646ms | 4.174ms | 4.184ms | 0.2% | 4.174ms | 151.182us | 0.000us | 2.37 MiB | 5 of 5 |
| q35 | 454.608us | 3.589ms | 4.179ms | 4.192ms | 0.3% | 4.179ms | 149.210us | 0.000us | 2.37 MiB | 5 of 5 |
| q36 | 485.361us | 1.365ms | 1.464ms | 1.474ms | 0.7% | 1.464ms | 153.066us | 0.000us | 853.45 KiB | 6 of 6 |
| q37 | 463.464us | 1.537ms | 1.574ms | 1.587ms | 0.8% | 1.574ms | 142.039us | 0.000us | 12.08 KiB | 6 of 6 |
| q38 | 497.909us | 2.181ms | 2.216ms | 2.231ms | 0.7% | 2.216ms | 169.139us | 0.000us | 6.65 KiB | 6 of 6 |
| q39 | 529.744us | 1.697ms | 1.769ms | 1.786ms | 0.9% | 1.769ms | 183.139us | 0.000us | 3.78 KiB | 6 of 6 |
| q40 | 528.391us | 2.637ms | 2.661ms | 2.677ms | 0.6% | 2.661ms | 159.290us | 0.000us | 66.70 KiB | 6 of 6 |
| q41 | 545.927us | 597.765us | 637.329us | 650.574us | 2.0% | 637.329us | 153.161us | 0.000us | 3.19 KiB | 6 of 6 |
| q42 | 539.838us | 504.375us | 538.659us | 555.945us | 3.1% | 538.659us | 172.342us | 0.000us | 3.55 KiB | 6 of 6 |
| q43 | 484.527us | 437.140us | 478.852us | 490.800us | 2.4% | 478.852us | 147.031us | 0.000us | 18.28 KiB | 6 of 6 |

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
| Aggregate | 0.000us | nothing to share | 39 | 197368 | 7872 | 0.0ns | 0.0ns | 39 of 39 |
| Fetch | 0.000us | nothing to share | 1 | 0 | 0 | handed none | handed on none | 1 of 1 |
| FileScan | 0.000us | nothing to share | 43 | 0 | 417512 | handed none | 0.0ns | 43 of 43 |
| Filter | 0.000us | nothing to share | 28 | 247512 | 30025 | 0.0ns | 0.0ns | 28 of 28 |
| Limit | 0.000us | nothing to share | 1 | 10 | 10 | 0.0ns | 0.0ns | 1 of 1 |
| Project | 0.000us | nothing to share | 91 | 211214 | 211214 | 0.0ns | 0.0ns | 91 of 91 |
| Sort | 0.000us | nothing to share | 1 | 6 | 6 | 0.0ns | 0.0ns | 1 of 1 |
| TopN | 0.000us | nothing to share | 31 | 10504 | 211 | 0.0ns | 0.0ns | 31 of 31 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is Aggregate, and the queries where it cost the most are q1 at 0.000us, q2 at 0.000us, q3 at 0.000us.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 10000 rows, one out of every 10000 of the 99997497 in the full file, which is a development loop rather than the suite
- every query here ran within 1.01x of every other one, so most of what was timed is whatever they have in common rather than the queries

These swung wider than reporting rule two allows:

- clickhouse-local swung by 33.2% of its median on q23, and rule two wants under 10%
- datafusion swung by 99.0% of its median on q9, and rule two wants under 10%
- polars swung by 25.5% of its median on q25, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

These ran every query at about the same speed:

- duckdb ran every query within 1.01x of every other one, and this suite spreads over 6x on an engine it is measuring
- duckdb-pinned ran every query within 1.00x of every other one, and this suite spreads over 6x on an engine it is measuring
- clickhouse-local ran every query within 1.33x of every other one, and this suite spreads over 6x on an engine it is measuring
- datafusion ran every query within 1.99x of every other one, and this suite spreads over 6x on an engine it is measuring
- polars ran every query within 1.27x of every other one, and this suite spreads over 6x on an engine it is measuring
- rudb ran every query within 1.01x of every other one, and this suite spreads over 6x on an engine it is measuring

A column that flat is not a column about the queries. Read those numbers as an upper bound on the engine and not as a measurement of it.

polars did not run q28, because the SQLContext has no strlen and no character length function.

polars did not run q29, because the SQLContext has no regexp_replace.

polars did not run q36, because the SQLContext rejects the repeated ClientIP output name in the group by.

polars did not run q43, because the SQLContext has no date_trunc.

So the polars column is 39 of 43 queries and its ratio is over the shared ones.

Answers differ, so this is not a comparison: q11: duckdb-pinned does not agree with duckdb: the same 10 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q11: clickhouse-local does not agree with duckdb: the same 10 numbers and 1 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q11: datafusion does not agree with duckdb: the same 10 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q11: polars does not agree with duckdb: the same 10 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q11: rudb does not agree with duckdb: the same 10 numbers and 1 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q12: duckdb-pinned does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q12: clickhouse-local does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q12: datafusion does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q12: polars does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q12: rudb does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q13: duckdb-pinned does not agree with duckdb: the same 10 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q13: clickhouse-local does not agree with duckdb: the same 10 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q13: datafusion does not agree with duckdb: the same 10 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q13: polars does not agree with duckdb: the same 10 numbers and 1 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q13: rudb does not agree with duckdb: the same 10 numbers and 1 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q14: duckdb-pinned does not agree with duckdb: the same 10 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q14: clickhouse-local does not agree with duckdb: the same 10 numbers and 1 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q14: datafusion does not agree with duckdb: the same 10 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q14: polars does not agree with duckdb: the same 10 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q14: rudb does not agree with duckdb: the same 10 numbers and 1 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q15: duckdb-pinned does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q15: clickhouse-local does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q15: datafusion does not agree with duckdb: the same 20 numbers and 1 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q15: polars does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q15: rudb does not agree with duckdb: the same 20 numbers and 1 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

These were answered differently and the data does not say which is right:

- q16: ORDER BY COUNT(*) DESC LIMIT 10 over user IDs, where the tenth place is a tie between many users with the same count and the engine picks.
- q17: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q18: LIMIT 10 with no ORDER BY at all, so the ten rows are whichever ten the group by handed back first and the query did not ask for any particular ten.
- q19: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q31: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q32: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q33: ORDER BY COUNT(*) DESC LIMIT 10 over WatchID and ClientIP unfiltered, where almost every group has a count of one and the ten that come back are arbitrary.
- q36: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q37: ORDER BY PageViews DESC LIMIT 10 over URLs, where the tail of the ten is a tie and the engine picks which URL fills it.
- q38: ORDER BY PageViews DESC LIMIT 10 over page titles, a tie at the cut as q37. Seen on a hundred thousand row sample, where the tenth and eleventh titles both had thirteen views and the two engines kept a different one. Sampling makes this more likely than the full file does, because the counts are smaller and so more of them collide.

So they are not checked, and a wrong answer from any engine on one of them would go unnoticed here. Every other query in the suite is checked in full.

These were answered differently and which engine is wrong is settled:

- q4: AVG(UserID) over a hundred million bigints, where the pinned DuckDB sums the column short by a multiple of 2^64 and rudb does not. Its own hugeint and decimal paths, the same column summed out of a table, its own per counter group sums, and adding the column up outside a database all give rudb's answer, and it reduces to two statements over a 1.3 MB one column parquet file. Written up as 14.10.1 in tamnd/rudb `spec/14-testing.md`, which is the list of places where DuckDB is wrong and we are not. tamnd/rudb-compat#12.

So they are a known difference rather than an open one, and the write up is where to go to disagree with that.

Hot here means page cache warm and not buffer pool warm, because every run is a fresh process so that no query's number depends on the one before it.

