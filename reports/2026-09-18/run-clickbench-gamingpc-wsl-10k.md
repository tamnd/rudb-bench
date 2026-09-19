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
| timeout | 600s per query, and no query reached it |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --rows 10000 --engines duckdb,duckdb-pinned,clickhouse-local,datafusion,polars,rudb --runs 5 --timeout 600 --report
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
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 97.873ms | 100.000ms | 4.26 MiB | its own database file | its own | 2.83 to 2.68 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 119.027ms | 120.000ms | 3.76 MiB | its own database file | its own | 2.68 to 2.73 |
| clickhouse-local | 26.9.1.1562 | ran | 148.304ms | 190.000ms | 2.68 MiB | its own MergeTree parts, as system.parts counts the active ones | its own | 2.73 to 2.52 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 2.47 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 2.52 to 2.44 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 2.47 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 2.44 to 2.56 |
| rudb | rudb 0.3.45 | ran | 0.000us | 0.000us | 2.47 MiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 2.56 to 2.60 |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb on 39 shared |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 120.000ms | 871.668ms | +626% | 872.048ms | 360.000ms | 0.41 | 40.25 MiB | none | 3.58M/s | 884.09 MiB/s | 1.00x |
| duckdb-pinned | 148.000ms | 1.736s | +1073% | 1.736s | 930.000ms | 0.54 | 53.41 MiB | none | 2.91M/s | 716.83 MiB/s | 1.26x |
| clickhouse-local | 398.000ms | 3.422s | +760% | 3.392s | 2.990s | 0.87 | 250.84 MiB | none | 1.08M/s | 266.56 MiB/s | 3.43x |
| datafusion | 300.000ms | 1.054s | +251% | 1.054s | 2.000s | 1.90 | 201.09 MiB | none | 1.43M/s | 353.64 MiB/s | 2.57x |
| polars | 585.086ms | 4.408s | +653% | 4.369s | 5.140s | 1.17 | 89.39 MiB | none | 666.57K/s | 164.46 MiB/s | 5.63x |
| rudb | 77.588ms | 875.480ms | +1028% | 875.947ms | 0.000us | 0.00 | 13.24 MiB | none | 5.54M/s | 1.34 GiB/s | 0.66x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 0.000us | 5.000ms | 1.000ms | 6.645ms | 804.648us |
| q2 | filtered count | 0.000us | 1.000ms | 5.000ms | 4.000ms | 7.239ms | 825.597us |
| q3 | three aggregates | 1.000ms | 1.000ms | 6.000ms | 4.000ms | 6.894ms | 846.560us |
| q4 | average | 0.000us | 1.000ms | 6.000ms | 4.000ms | 7.002ms | 849.688us |
| q5 | count distinct, high card | 2.000ms | 2.000ms | 7.000ms | 6.000ms | 13.276ms | 818.870us |
| q6 | count distinct, strings | 2.000ms | 2.000ms | 6.000ms | 7.000ms | 12.810ms | 1.287ms |
| q7 | min and max of a date | 1.000ms | 1.000ms | 11.000ms | 1.000ms | 6.500ms | 875.967us |
| q8 | group by, low card | 1.000ms | 5.000ms | 12.000ms | 5.000ms | 13.804ms | 939.854us |
| q9 | group by and count distinct | 4.000ms | 3.000ms | 6.000ms | 10.000ms | 22.250ms | 1.051ms |
| q10 | group by, several aggregates | 5.000ms | 5.000ms | 7.000ms | 7.000ms | 25.415ms | 1.374ms |
| q11 | group by a string and count distinct | 3.000ms | 4.000ms | 7.000ms | 9.000ms | 21.503ms | 1.009ms |
| q12 | group by two strings and count distinct | 4.000ms | 4.000ms | 7.000ms | 9.000ms | 22.298ms | 1.099ms |
| q13 | group by a string and top k | 3.000ms | 3.000ms | 7.000ms | 10.000ms | 15.636ms | 1.286ms |
| q14 | group by a string and count distinct | 3.000ms | 4.000ms | 7.000ms | 9.000ms | 21.593ms | 1.514ms |
| q15 | group by two columns and top k | 3.000ms | 3.000ms | 7.000ms | 8.000ms | 17.002ms | 1.485ms |
| q16 | group by, very high card | 3.000ms | 2.000ms | 7.000ms | 7.000ms | 19.355ms | 1.715ms |
| q17 | group by two, very high card | 3.000ms | 3.000ms | 8.000ms | 6.000ms | 21.348ms | 2.283ms |
| q18 | group by two, no ordering | 3.000ms | 3.000ms | 7.000ms | 6.000ms | 13.450ms | 1.176ms |
| q19 | group by with an extract | 3.000ms | 3.000ms | 8.000ms | 7.000ms | 24.043ms | 2.720ms |
| q20 | point lookup | 1.000ms | 0.000us | 12.000ms | 3.000ms | 6.082ms | 794.705us |
| q21 | substring scan | 1.000ms | 2.000ms | 8.000ms | 4.000ms | 8.414ms | 2.073ms |
| q22 | substring scan and group by | 2.000ms | 2.000ms | 8.000ms | 7.000ms | 15.492ms | 2.408ms |
| q23 | two substring scans and group by | 5.000ms | 6.000ms | 10.000ms | 9.000ms | 22.192ms | 4.348ms |
| q24 | select star and top k | 7.000ms | 11.000ms | 16.000ms | 12.000ms | 11.595ms | 2.360ms |
| q25 | top k by a date | 2.000ms | 1.000ms | 12.000ms | 5.000ms | 10.608ms | 1.085ms |
| q26 | top k by a string | 1.000ms | 1.000ms | 6.000ms | 4.000ms | 11.271ms | 1.023ms |
| q27 | top k by two columns | 1.000ms | 1.000ms | 12.000ms | 4.000ms | 11.865ms | 1.119ms |
| q28 | group by with a string length | 3.000ms | 4.000ms | 8.000ms | 6.000ms | no dialect | 2.403ms |
| q29 | group by a regular expression | 7.000ms | 7.000ms | 12.000ms | 13.000ms | no dialect | 3.231ms |
| q30 | ninety sums over one column | 4.000ms | 14.000ms | 10.000ms | 11.000ms | 11.396ms | 2.267ms |
| q31 | group by two and several aggregates | 3.000ms | 3.000ms | 7.000ms | 7.000ms | 15.535ms | 1.862ms |
| q32 | group by a high card pair | 3.000ms | 4.000ms | 7.000ms | 8.000ms | 16.446ms | 1.924ms |
| q33 | group by a high card pair, unfiltered | 3.000ms | 3.000ms | 8.000ms | 6.000ms | 15.761ms | 1.960ms |
| q34 | group by a long string | 4.000ms | 4.000ms | 8.000ms | 8.000ms | 16.891ms | 3.689ms |
| q35 | group by a constant and a long string | 4.000ms | 4.000ms | 8.000ms | 9.000ms | 19.259ms | 4.148ms |
| q36 | group by four expressions | 3.000ms | 3.000ms | 7.000ms | 6.000ms | no dialect | 1.876ms |
| q37 | date range and group by a URL | 3.000ms | 4.000ms | 15.000ms | 8.000ms | 17.772ms | 2.512ms |
| q38 | date range and group by a title | 3.000ms | 3.000ms | 15.000ms | 9.000ms | 17.076ms | 2.972ms |
| q39 | date range, group by and offset | 3.000ms | 4.000ms | 15.000ms | 10.000ms | 15.229ms | 2.418ms |
| q40 | date range, a case and a wide group by | 3.000ms | 5.000ms | 16.000ms | 9.000ms | 15.605ms | 3.516ms |
| q41 | date range with an IN and a hash | 3.000ms | 3.000ms | 14.000ms | 7.000ms | 14.036ms | 1.266ms |
| q42 | date range and a deep offset | 3.000ms | 6.000ms | 14.000ms | 7.000ms | 14.498ms | 1.177ms |
| q43 | minute buckets over a date range | 3.000ms | 3.000ms | 14.000ms | 8.000ms | no dialect | 1.197ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 20.256ms | 20.265ms | 0.0% | 20.264ms | 20.273ms | 20.256ms | 20.319ms | 0.000us | 26.83 MiB | none | 493.46K/s |
| q2 | filtered count | 0.000us | 20.267ms | 20.266ms | 0.0% | 20.259ms | 20.267ms | 20.255ms | 20.308ms | 0.000us | 28.00 MiB | none | 493.44K/s |
| q3 | three aggregates | 1.000ms | 20.266ms | 20.271ms | 0.1% | 20.263ms | 20.279ms | 20.260ms | 20.280ms | 0.000us | 28.33 MiB | none | 493.32K/s |
| q4 | average | 0.000us | 20.274ms | 20.274ms | 0.1% | 20.267ms | 20.295ms | 20.261ms | 20.329ms | 0.000us | 27.75 MiB | none | 493.25K/s |
| q5 | count distinct, high card | 2.000ms | 20.491ms | 20.264ms | 0.0% | 20.260ms | 20.271ms | 20.258ms | 20.321ms | 10.000ms | 30.24 MiB | none | 493.48K/s |
| q6 | count distinct, strings | 2.000ms | 20.257ms | 20.274ms | 0.1% | 20.265ms | 20.280ms | 20.258ms | 20.290ms | 10.000ms | 30.73 MiB | none | 493.23K/s |
| q7 | min and max of a date | 1.000ms | 20.259ms | 20.265ms | 0.0% | 20.264ms | 20.268ms | 20.260ms | 20.339ms | 10.000ms | 27.24 MiB | none | 493.47K/s |
| q8 | group by, low card | 1.000ms | 20.255ms | 20.266ms | 0.0% | 20.261ms | 20.267ms | 20.257ms | 20.287ms | 10.000ms | 29.47 MiB | none | 493.45K/s |
| q9 | group by and count distinct | 4.000ms | 20.289ms | 20.278ms | 0.1% | 20.275ms | 20.288ms | 20.272ms | 20.298ms | 10.000ms | 37.25 MiB | none | 493.14K/s |
| q10 | group by, several aggregates | 5.000ms | 20.274ms | 20.264ms | 0.0% | 20.264ms | 20.267ms | 20.251ms | 20.279ms | 20.000ms | 40.25 MiB | none | 493.48K/s |
| q11 | group by a string and count distinct | 3.000ms | 20.304ms | 20.271ms | 0.1% | 20.256ms | 20.274ms | 20.255ms | 20.275ms | 10.000ms | 35.25 MiB | none | 493.32K/s |
| q12 | group by two strings and count distinct | 4.000ms | 20.261ms | 20.270ms | 0.0% | 20.262ms | 20.271ms | 20.256ms | 20.272ms | 10.000ms | 37.49 MiB | none | 493.33K/s |
| q13 | group by a string and top k | 3.000ms | 20.275ms | 20.277ms | 0.0% | 20.275ms | 20.278ms | 20.273ms | 20.284ms | 0.000us | 31.50 MiB | none | 493.18K/s |
| q14 | group by a string and count distinct | 3.000ms | 20.260ms | 20.271ms | 0.1% | 20.256ms | 20.274ms | 20.256ms | 20.286ms | 10.000ms | 38.44 MiB | none | 493.32K/s |
| q15 | group by two columns and top k | 3.000ms | 20.276ms | 20.275ms | 0.1% | 20.265ms | 20.279ms | 20.262ms | 20.316ms | 10.000ms | 31.95 MiB | none | 493.22K/s |
| q16 | group by, very high card | 3.000ms | 20.274ms | 20.259ms | 0.1% | 20.253ms | 20.267ms | 20.251ms | 20.270ms | 10.000ms | 33.84 MiB | none | 493.61K/s |
| q17 | group by two, very high card | 3.000ms | 20.262ms | 20.269ms | 0.1% | 20.267ms | 20.278ms | 20.262ms | 20.290ms | 10.000ms | 35.59 MiB | none | 493.37K/s |
| q18 | group by two, no ordering | 3.000ms | 20.271ms | 20.273ms | 0.1% | 20.264ms | 20.278ms | 20.259ms | 20.279ms | 10.000ms | 36.36 MiB | none | 493.27K/s |
| q19 | group by with an extract | 3.000ms | 20.272ms | 20.265ms | 0.2% | 20.256ms | 20.287ms | 20.252ms | 20.289ms | 10.000ms | 35.92 MiB | none | 493.47K/s |
| q20 | point lookup | 1.000ms | 20.325ms | 20.269ms | 0.0% | 20.267ms | 20.271ms | 20.257ms | 20.357ms | 0.000us | 27.50 MiB | none | 493.37K/s |
| q21 | substring scan | 1.000ms | 20.262ms | 20.266ms | 0.1% | 20.266ms | 20.284ms | 20.256ms | 20.284ms | 0.000us | 28.98 MiB | none | 493.43K/s |
| q22 | substring scan and group by | 2.000ms | 20.416ms | 20.262ms | 0.0% | 20.261ms | 20.265ms | 20.260ms | 20.269ms | 10.000ms | 30.25 MiB | none | 493.53K/s |
| q23 | two substring scans and group by | 5.000ms | 20.256ms | 20.265ms | 0.0% | 20.265ms | 20.268ms | 20.265ms | 20.270ms | 10.000ms | 34.88 MiB | none | 493.45K/s |
| q24 | select star and top k | 7.000ms | 20.255ms | 20.265ms | 0.1% | 20.257ms | 20.273ms | 20.254ms | 20.284ms | 10.000ms | 37.60 MiB | none | 493.45K/s |
| q25 | top k by a date | 2.000ms | 20.263ms | 20.269ms | 0.1% | 20.259ms | 20.270ms | 20.259ms | 20.278ms | 10.000ms | 31.24 MiB | none | 493.36K/s |
| q26 | top k by a string | 1.000ms | 20.270ms | 20.297ms | 0.2% | 20.266ms | 20.301ms | 20.263ms | 20.428ms | 0.000us | 27.99 MiB | none | 492.69K/s |
| q27 | top k by two columns | 1.000ms | 20.260ms | 20.266ms | 0.1% | 20.256ms | 20.273ms | 20.252ms | 20.281ms | 10.000ms | 28.74 MiB | none | 493.44K/s |
| q28 | group by with a string length | 3.000ms | 20.284ms | 20.264ms | 0.0% | 20.263ms | 20.268ms | 20.259ms | 20.364ms | 10.000ms | 32.43 MiB | none | 493.48K/s |
| q29 | group by a regular expression | 7.000ms | 20.260ms | 20.270ms | 0.0% | 20.270ms | 20.279ms | 20.261ms | 20.341ms | 20.000ms | 34.18 MiB | none | 493.33K/s |
| q30 | ninety sums over one column | 4.000ms | 20.251ms | 20.266ms | 0.0% | 20.264ms | 20.270ms | 20.262ms | 20.280ms | 0.000us | 31.49 MiB | none | 493.44K/s |
| q31 | group by two and several aggregates | 3.000ms | 20.272ms | 20.270ms | 0.1% | 20.268ms | 20.282ms | 20.259ms | 20.307ms | 10.000ms | 34.72 MiB | none | 493.35K/s |
| q32 | group by a high card pair | 3.000ms | 20.272ms | 20.270ms | 0.0% | 20.263ms | 20.270ms | 20.262ms | 20.277ms | 10.000ms | 34.73 MiB | none | 493.34K/s |
| q33 | group by a high card pair, unfiltered | 3.000ms | 20.273ms | 20.269ms | 0.0% | 20.266ms | 20.270ms | 20.260ms | 20.277ms | 10.000ms | 35.11 MiB | none | 493.37K/s |
| q34 | group by a long string | 4.000ms | 20.325ms | 20.275ms | 0.0% | 20.270ms | 20.280ms | 20.266ms | 20.282ms | 10.000ms | 34.50 MiB | none | 493.22K/s |
| q35 | group by a constant and a long string | 4.000ms | 20.271ms | 20.274ms | 0.0% | 20.273ms | 20.277ms | 20.265ms | 20.279ms | 10.000ms | 36.50 MiB | none | 493.25K/s |
| q36 | group by four expressions | 3.000ms | 20.275ms | 20.274ms | 0.0% | 20.273ms | 20.275ms | 20.267ms | 20.329ms | 10.000ms | 34.39 MiB | none | 493.24K/s |
| q37 | date range and group by a URL | 3.000ms | 20.274ms | 20.280ms | 0.0% | 20.275ms | 20.284ms | 20.269ms | 20.295ms | 10.000ms | 33.50 MiB | none | 493.09K/s |
| q38 | date range and group by a title | 3.000ms | 20.273ms | 20.283ms | 0.1% | 20.278ms | 20.292ms | 20.268ms | 20.323ms | 10.000ms | 32.76 MiB | none | 493.03K/s |
| q39 | date range, group by and offset | 3.000ms | 20.268ms | 20.273ms | 0.1% | 20.265ms | 20.279ms | 20.264ms | 20.298ms | 10.000ms | 32.49 MiB | none | 493.27K/s |
| q40 | date range, a case and a wide group by | 3.000ms | 20.285ms | 20.272ms | 0.0% | 20.269ms | 20.274ms | 20.266ms | 20.278ms | 10.000ms | 34.99 MiB | none | 493.29K/s |
| q41 | date range with an IN and a hash | 3.000ms | 20.282ms | 20.299ms | 0.2% | 20.271ms | 20.311ms | 20.260ms | 20.320ms | 10.000ms | 33.17 MiB | none | 492.64K/s |
| q42 | date range and a deep offset | 3.000ms | 20.272ms | 20.276ms | 0.0% | 20.274ms | 20.279ms | 20.271ms | 20.286ms | 10.000ms | 32.31 MiB | none | 493.19K/s |
| q43 | minute buckets over a date range | 3.000ms | 20.262ms | 20.279ms | 0.1% | 20.272ms | 20.288ms | 20.267ms | 20.310ms | 10.000ms | 31.99 MiB | none | 493.13K/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 120.000ms by its own clock and 871.668ms by ours, 872.048ms cold, 360.000ms of CPU, peak 40.25 MiB, 3.58M/s and 884.09 MiB/s.

Running it cost 626% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.00x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 0.000us | 40.388ms | 40.373ms | 0.2% | 40.372ms | 40.451ms | 40.366ms | 40.593ms | 20.000ms | 39.77 MiB | none | 247.69K/s |
| q2 | filtered count | 1.000ms | 40.364ms | 40.367ms | 0.0% | 40.354ms | 40.369ms | 40.353ms | 40.377ms | 20.000ms | 40.10 MiB | none | 247.73K/s |
| q3 | three aggregates | 1.000ms | 40.373ms | 40.373ms | 0.1% | 40.362ms | 40.419ms | 40.354ms | 42.088ms | 20.000ms | 40.54 MiB | none | 247.69K/s |
| q4 | average | 1.000ms | 40.350ms | 40.479ms | 0.3% | 40.407ms | 40.512ms | 40.388ms | 40.526ms | 20.000ms | 40.54 MiB | none | 247.04K/s |
| q5 | count distinct, high card | 2.000ms | 40.417ms | 40.389ms | 0.2% | 40.366ms | 40.435ms | 40.356ms | 40.437ms | 20.000ms | 43.57 MiB | none | 247.59K/s |
| q6 | count distinct, strings | 2.000ms | 40.384ms | 40.382ms | 0.0% | 40.370ms | 40.382ms | 40.365ms | 40.438ms | 20.000ms | 42.04 MiB | none | 247.64K/s |
| q7 | min and max of a date | 1.000ms | 40.415ms | 40.375ms | 0.0% | 40.374ms | 40.386ms | 40.351ms | 40.392ms | 20.000ms | 40.10 MiB | none | 247.68K/s |
| q8 | group by, low card | 5.000ms | 40.377ms | 40.376ms | 0.0% | 40.372ms | 40.378ms | 40.366ms | 40.400ms | 30.000ms | 42.86 MiB | none | 247.67K/s |
| q9 | group by and count distinct | 3.000ms | 40.347ms | 40.360ms | 0.0% | 40.356ms | 40.364ms | 40.353ms | 40.381ms | 20.000ms | 49.39 MiB | none | 247.77K/s |
| q10 | group by, several aggregates | 5.000ms | 40.372ms | 40.357ms | 0.0% | 40.355ms | 40.374ms | 40.354ms | 40.375ms | 30.000ms | 50.12 MiB | none | 247.79K/s |
| q11 | group by a string and count distinct | 4.000ms | 40.348ms | 40.365ms | 0.1% | 40.357ms | 40.380ms | 40.353ms | 40.396ms | 20.000ms | 47.21 MiB | none | 247.74K/s |
| q12 | group by two strings and count distinct | 4.000ms | 40.346ms | 40.363ms | 0.0% | 40.361ms | 40.372ms | 40.356ms | 40.389ms | 30.000ms | 47.97 MiB | none | 247.75K/s |
| q13 | group by a string and top k | 3.000ms | 40.360ms | 40.359ms | 0.1% | 40.357ms | 40.378ms | 40.357ms | 40.379ms | 20.000ms | 43.11 MiB | none | 247.78K/s |
| q14 | group by a string and count distinct | 4.000ms | 40.348ms | 40.365ms | 0.0% | 40.362ms | 40.370ms | 40.359ms | 40.393ms | 30.000ms | 49.91 MiB | none | 247.74K/s |
| q15 | group by two columns and top k | 3.000ms | 40.373ms | 40.362ms | 0.5% | 40.356ms | 40.545ms | 40.353ms | 40.796ms | 20.000ms | 43.86 MiB | none | 247.76K/s |
| q16 | group by, very high card | 2.000ms | 40.357ms | 40.370ms | 0.0% | 40.365ms | 40.370ms | 40.355ms | 40.383ms | 20.000ms | 43.92 MiB | none | 247.71K/s |
| q17 | group by two, very high card | 3.000ms | 40.378ms | 40.361ms | 0.0% | 40.361ms | 40.362ms | 40.356ms | 40.381ms | 20.000ms | 45.12 MiB | none | 247.76K/s |
| q18 | group by two, no ordering | 3.000ms | 40.354ms | 40.364ms | 0.0% | 40.356ms | 40.365ms | 40.351ms | 40.366ms | 20.000ms | 45.42 MiB | none | 247.75K/s |
| q19 | group by with an extract | 3.000ms | 40.362ms | 40.368ms | 0.0% | 40.368ms | 40.369ms | 40.365ms | 40.385ms | 20.000ms | 45.75 MiB | none | 247.72K/s |
| q20 | point lookup | 0.000us | 40.387ms | 40.371ms | 0.0% | 40.370ms | 40.379ms | 40.349ms | 40.497ms | 10.000ms | 39.29 MiB | none | 247.70K/s |
| q21 | substring scan | 2.000ms | 40.366ms | 40.368ms | 0.0% | 40.362ms | 40.374ms | 40.356ms | 40.390ms | 20.000ms | 41.85 MiB | none | 247.72K/s |
| q22 | substring scan and group by | 2.000ms | 40.372ms | 40.373ms | 0.0% | 40.367ms | 40.382ms | 40.363ms | 40.382ms | 20.000ms | 42.55 MiB | none | 247.69K/s |
| q23 | two substring scans and group by | 6.000ms | 40.363ms | 40.357ms | 0.0% | 40.352ms | 40.357ms | 40.348ms | 40.371ms | 30.000ms | 48.58 MiB | none | 247.79K/s |
| q24 | select star and top k | 11.000ms | 40.368ms | 40.370ms | 0.1% | 40.359ms | 40.384ms | 40.349ms | 40.405ms | 30.000ms | 53.41 MiB | none | 247.71K/s |
| q25 | top k by a date | 1.000ms | 40.348ms | 40.372ms | 0.0% | 40.362ms | 40.374ms | 40.361ms | 40.493ms | 10.000ms | 41.07 MiB | none | 247.70K/s |
| q26 | top k by a string | 1.000ms | 40.354ms | 40.367ms | 0.0% | 40.364ms | 40.381ms | 40.355ms | 40.513ms | 20.000ms | 40.80 MiB | none | 247.73K/s |
| q27 | top k by two columns | 1.000ms | 40.357ms | 40.363ms | 0.1% | 40.359ms | 40.381ms | 40.358ms | 40.386ms | 20.000ms | 41.04 MiB | none | 247.75K/s |
| q28 | group by with a string length | 4.000ms | 40.379ms | 40.373ms | 0.1% | 40.352ms | 40.394ms | 40.335ms | 40.408ms | 20.000ms | 45.09 MiB | none | 247.69K/s |
| q29 | group by a regular expression | 7.000ms | 40.354ms | 40.370ms | 0.1% | 40.355ms | 40.377ms | 40.346ms | 40.386ms | 30.000ms | 45.61 MiB | none | 247.71K/s |
| q30 | ninety sums over one column | 14.000ms | 40.365ms | 40.348ms | 0.0% | 40.344ms | 40.350ms | 40.343ms | 40.374ms | 30.000ms | 53.07 MiB | none | 247.85K/s |
| q31 | group by two and several aggregates | 3.000ms | 40.353ms | 40.374ms | 0.0% | 40.372ms | 40.389ms | 40.353ms | 40.432ms | 20.000ms | 46.16 MiB | none | 247.69K/s |
| q32 | group by a high card pair | 4.000ms | 40.364ms | 40.365ms | 0.0% | 40.363ms | 40.372ms | 40.358ms | 40.391ms | 20.000ms | 45.51 MiB | none | 247.74K/s |
| q33 | group by a high card pair, unfiltered | 3.000ms | 40.375ms | 40.363ms | 0.0% | 40.359ms | 40.376ms | 40.358ms | 40.386ms | 20.000ms | 46.17 MiB | none | 247.75K/s |
| q34 | group by a long string | 4.000ms | 40.369ms | 40.356ms | 0.0% | 40.351ms | 40.366ms | 40.348ms | 40.368ms | 20.000ms | 45.35 MiB | none | 247.80K/s |
| q35 | group by a constant and a long string | 4.000ms | 40.382ms | 40.364ms | 0.0% | 40.353ms | 40.371ms | 40.352ms | 40.531ms | 20.000ms | 46.11 MiB | none | 247.74K/s |
| q36 | group by four expressions | 3.000ms | 40.369ms | 40.369ms | 0.0% | 40.364ms | 40.375ms | 40.356ms | 40.376ms | 20.000ms | 44.41 MiB | none | 247.71K/s |
| q37 | date range and group by a URL | 4.000ms | 40.354ms | 40.374ms | 0.1% | 40.362ms | 40.396ms | 40.356ms | 40.408ms | 20.000ms | 44.87 MiB | none | 247.68K/s |
| q38 | date range and group by a title | 3.000ms | 40.351ms | 40.378ms | 0.0% | 40.374ms | 40.388ms | 40.303ms | 40.407ms | 20.000ms | 45.61 MiB | none | 247.66K/s |
| q39 | date range, group by and offset | 4.000ms | 40.373ms | 40.360ms | 0.0% | 40.360ms | 40.364ms | 40.348ms | 40.368ms | 20.000ms | 44.59 MiB | none | 247.77K/s |
| q40 | date range, a case and a wide group by | 5.000ms | 40.368ms | 40.380ms | 0.0% | 40.367ms | 40.380ms | 40.366ms | 40.389ms | 20.000ms | 46.87 MiB | none | 247.65K/s |
| q41 | date range with an IN and a hash | 3.000ms | 40.368ms | 40.355ms | 0.0% | 40.352ms | 40.368ms | 40.351ms | 40.426ms | 20.000ms | 45.17 MiB | none | 247.80K/s |
| q42 | date range and a deep offset | 6.000ms | 40.362ms | 40.349ms | 0.1% | 40.344ms | 40.368ms | 40.343ms | 40.376ms | 30.000ms | 45.11 MiB | none | 247.84K/s |
| q43 | minute buckets over a date range | 3.000ms | 40.397ms | 40.360ms | 0.0% | 40.357ms | 40.362ms | 40.354ms | 40.371ms | 20.000ms | 43.29 MiB | none | 247.77K/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 148.000ms by its own clock and 1.736s by ours, 1.736s cold, 930.000ms of CPU, peak 53.41 MiB, 2.91M/s and 716.83 MiB/s.

Running it cost 1073% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.00x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 5.000ms | 60.417ms | 80.503ms | 0.0% | 80.488ms | 80.506ms | 60.429ms | 80.509ms | 70.000ms | 238.35 MiB | none | 124.22K/s |
| q2 | filtered count | 5.000ms | 80.546ms | 80.506ms | 0.1% | 80.497ms | 80.544ms | 60.376ms | 81.674ms | 70.000ms | 239.22 MiB | none | 124.21K/s |
| q3 | three aggregates | 6.000ms | 60.393ms | 80.485ms | 25.0% | 60.419ms | 80.563ms | 60.410ms | 80.677ms | 70.000ms | 242.20 MiB | none | 124.25K/s |
| q4 | average | 6.000ms | 80.499ms | 60.448ms | 0.1% | 60.413ms | 60.503ms | 60.401ms | 80.514ms | 60.000ms | 241.99 MiB | none | 165.43K/s |
| q5 | count distinct, high card | 7.000ms | 80.485ms | 80.539ms | 0.0% | 80.524ms | 80.555ms | 80.500ms | 80.802ms | 60.000ms | 243.11 MiB | none | 124.16K/s |
| q6 | count distinct, strings | 6.000ms | 60.436ms | 80.557ms | 0.1% | 80.501ms | 80.564ms | 80.489ms | 80.579ms | 70.000ms | 241.77 MiB | none | 124.14K/s |
| q7 | min and max of a date | 11.000ms | 80.494ms | 80.504ms | 0.0% | 80.498ms | 80.512ms | 80.490ms | 80.522ms | 70.000ms | 241.36 MiB | none | 124.22K/s |
| q8 | group by, low card | 12.000ms | 80.512ms | 80.514ms | 0.0% | 80.503ms | 80.517ms | 80.502ms | 80.736ms | 70.000ms | 243.55 MiB | none | 124.20K/s |
| q9 | group by and count distinct | 6.000ms | 83.183ms | 80.558ms | 0.0% | 80.548ms | 80.577ms | 80.519ms | 80.588ms | 70.000ms | 244.63 MiB | none | 124.13K/s |
| q10 | group by, several aggregates | 7.000ms | 80.498ms | 80.529ms | 0.1% | 80.507ms | 80.557ms | 80.494ms | 80.596ms | 70.000ms | 244.04 MiB | none | 124.18K/s |
| q11 | group by a string and count distinct | 7.000ms | 80.515ms | 80.492ms | 0.0% | 80.486ms | 80.505ms | 60.441ms | 80.560ms | 70.000ms | 245.07 MiB | none | 124.24K/s |
| q12 | group by two strings and count distinct | 7.000ms | 80.589ms | 60.434ms | 33.4% | 60.408ms | 80.580ms | 60.398ms | 80.623ms | 60.000ms | 245.02 MiB | none | 165.47K/s |
| q13 | group by a string and top k | 7.000ms | 87.137ms | 80.497ms | 24.9% | 60.442ms | 80.516ms | 60.424ms | 80.523ms | 70.000ms | 245.30 MiB | none | 124.23K/s |
| q14 | group by a string and count distinct | 7.000ms | 80.496ms | 80.523ms | 0.0% | 80.507ms | 80.538ms | 80.506ms | 80.542ms | 70.000ms | 244.82 MiB | none | 124.19K/s |
| q15 | group by two columns and top k | 7.000ms | 80.553ms | 80.505ms | 0.0% | 80.503ms | 80.523ms | 80.489ms | 80.555ms | 70.000ms | 245.34 MiB | none | 124.22K/s |
| q16 | group by, very high card | 7.000ms | 80.534ms | 80.535ms | 0.0% | 80.517ms | 80.539ms | 80.502ms | 80.578ms | 70.000ms | 244.61 MiB | none | 124.17K/s |
| q17 | group by two, very high card | 8.000ms | 80.487ms | 80.519ms | 0.0% | 80.519ms | 80.546ms | 80.512ms | 80.566ms | 70.000ms | 247.63 MiB | none | 124.19K/s |
| q18 | group by two, no ordering | 7.000ms | 80.562ms | 80.528ms | 0.0% | 80.513ms | 80.537ms | 80.504ms | 80.645ms | 70.000ms | 245.07 MiB | none | 124.18K/s |
| q19 | group by with an extract | 8.000ms | 80.548ms | 80.563ms | 0.1% | 80.527ms | 80.581ms | 80.524ms | 80.742ms | 60.000ms | 248.13 MiB | none | 124.13K/s |
| q20 | point lookup | 12.000ms | 80.528ms | 80.509ms | 0.0% | 80.506ms | 80.530ms | 80.497ms | 80.586ms | 70.000ms | 242.13 MiB | none | 124.21K/s |
| q21 | substring scan | 8.000ms | 80.577ms | 80.563ms | 0.0% | 80.559ms | 80.575ms | 80.486ms | 80.669ms | 70.000ms | 244.35 MiB | none | 124.13K/s |
| q22 | substring scan and group by | 8.000ms | 80.525ms | 80.511ms | 0.0% | 80.500ms | 80.528ms | 60.486ms | 80.666ms | 60.000ms | 245.88 MiB | none | 124.21K/s |
| q23 | two substring scans and group by | 10.000ms | 80.615ms | 80.543ms | 0.0% | 80.538ms | 80.551ms | 80.504ms | 80.572ms | 70.000ms | 250.84 MiB | none | 124.16K/s |
| q24 | select star and top k | 16.000ms | 80.549ms | 80.511ms | 0.0% | 80.509ms | 80.518ms | 80.499ms | 80.521ms | 80.000ms | 247.11 MiB | none | 124.21K/s |
| q25 | top k by a date | 12.000ms | 80.496ms | 80.507ms | 0.0% | 80.505ms | 80.509ms | 80.490ms | 80.572ms | 70.000ms | 243.96 MiB | none | 124.21K/s |
| q26 | top k by a string | 6.000ms | 80.505ms | 80.508ms | 24.9% | 60.475ms | 80.539ms | 60.418ms | 80.543ms | 70.000ms | 242.86 MiB | none | 124.21K/s |
| q27 | top k by two columns | 12.000ms | 80.527ms | 80.529ms | 0.0% | 80.511ms | 80.544ms | 80.509ms | 80.544ms | 70.000ms | 243.80 MiB | none | 124.18K/s |
| q28 | group by with a string length | 8.000ms | 60.427ms | 80.510ms | 0.0% | 80.503ms | 80.511ms | 80.498ms | 80.625ms | 70.000ms | 246.46 MiB | none | 124.21K/s |
| q29 | group by a regular expression | 12.000ms | 80.491ms | 80.606ms | 0.1% | 80.563ms | 80.659ms | 80.508ms | 80.757ms | 80.000ms | 249.52 MiB | none | 124.06K/s |
| q30 | ninety sums over one column | 10.000ms | 80.522ms | 80.530ms | 0.1% | 80.505ms | 80.545ms | 80.498ms | 80.550ms | 70.000ms | 244.37 MiB | none | 124.18K/s |
| q31 | group by two and several aggregates | 7.000ms | 80.535ms | 80.555ms | 0.1% | 80.521ms | 80.607ms | 80.509ms | 80.647ms | 70.000ms | 245.68 MiB | none | 124.14K/s |
| q32 | group by a high card pair | 7.000ms | 80.525ms | 80.504ms | 0.0% | 80.494ms | 80.524ms | 60.422ms | 80.558ms | 70.000ms | 244.84 MiB | none | 124.22K/s |
| q33 | group by a high card pair, unfiltered | 8.000ms | 80.531ms | 80.555ms | 0.0% | 80.549ms | 80.568ms | 80.508ms | 81.201ms | 60.000ms | 246.29 MiB | none | 124.14K/s |
| q34 | group by a long string | 8.000ms | 80.491ms | 80.548ms | 0.0% | 80.547ms | 80.584ms | 80.516ms | 80.592ms | 70.000ms | 247.74 MiB | none | 124.15K/s |
| q35 | group by a constant and a long string | 8.000ms | 80.690ms | 80.517ms | 0.0% | 80.512ms | 80.524ms | 80.509ms | 80.565ms | 70.000ms | 247.98 MiB | none | 124.20K/s |
| q36 | group by four expressions | 7.000ms | 80.568ms | 80.516ms | 0.1% | 80.491ms | 80.534ms | 60.438ms | 80.574ms | 60.000ms | 246.33 MiB | none | 124.20K/s |
| q37 | date range and group by a URL | 15.000ms | 80.532ms | 80.510ms | 0.0% | 80.503ms | 80.516ms | 80.473ms | 80.532ms | 80.000ms | 249.23 MiB | none | 124.21K/s |
| q38 | date range and group by a title | 15.000ms | 80.514ms | 80.516ms | 0.0% | 80.498ms | 80.518ms | 80.475ms | 80.605ms | 70.000ms | 248.94 MiB | none | 124.20K/s |
| q39 | date range, group by and offset | 15.000ms | 80.512ms | 80.504ms | 0.0% | 80.492ms | 80.524ms | 80.485ms | 80.528ms | 70.000ms | 250.07 MiB | none | 124.22K/s |
| q40 | date range, a case and a wide group by | 16.000ms | 80.519ms | 80.539ms | 0.0% | 80.513ms | 80.544ms | 80.489ms | 80.630ms | 80.000ms | 250.53 MiB | none | 124.16K/s |
| q41 | date range with an IN and a hash | 14.000ms | 80.504ms | 80.509ms | 0.0% | 80.504ms | 80.516ms | 80.498ms | 80.526ms | 70.000ms | 248.63 MiB | none | 124.21K/s |
| q42 | date range and a deep offset | 14.000ms | 80.492ms | 80.498ms | 0.0% | 80.496ms | 80.499ms | 80.489ms | 80.543ms | 80.000ms | 247.17 MiB | none | 124.23K/s |
| q43 | minute buckets over a date range | 14.000ms | 80.496ms | 80.498ms | 0.0% | 80.491ms | 80.501ms | 80.489ms | 80.509ms | 70.000ms | 247.36 MiB | none | 124.23K/s |

clickhouse-local 26.9.1.1562 over 43 of 43 queries. Total 398.000ms by its own clock and 3.422s by ours, 3.392s cold, 2.990s of CPU, peak 250.84 MiB, 1.08M/s and 266.56 MiB/s.

Running it cost 760% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.33x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 20.320ms | 20.323ms | 0.1% | 20.314ms | 20.329ms | 20.280ms | 20.352ms | 10.000ms | 80.21 MiB | none | 492.05K/s |
| q2 | filtered count | 4.000ms | 20.269ms | 20.281ms | 0.0% | 20.280ms | 20.287ms | 20.277ms | 20.295ms | 30.000ms | 121.52 MiB | none | 493.07K/s |
| q3 | three aggregates | 4.000ms | 20.279ms | 20.296ms | 0.0% | 20.294ms | 20.299ms | 20.285ms | 20.322ms | 30.000ms | 122.67 MiB | none | 492.70K/s |
| q4 | average | 4.000ms | 20.276ms | 20.285ms | 0.0% | 20.281ms | 20.287ms | 20.273ms | 20.289ms | 30.000ms | 123.12 MiB | none | 492.98K/s |
| q5 | count distinct, high card | 6.000ms | 20.294ms | 20.299ms | 0.1% | 20.277ms | 20.302ms | 20.275ms | 20.322ms | 30.000ms | 144.95 MiB | none | 492.64K/s |
| q6 | count distinct, strings | 7.000ms | 20.286ms | 20.284ms | 0.0% | 20.282ms | 20.292ms | 20.279ms | 20.308ms | 50.000ms | 154.50 MiB | none | 493.01K/s |
| q7 | min and max of a date | 1.000ms | 20.281ms | 20.283ms | 0.1% | 20.275ms | 20.289ms | 20.272ms | 20.315ms | 0.000us | 80.05 MiB | none | 493.01K/s |
| q8 | group by, low card | 5.000ms | 20.277ms | 20.294ms | 0.1% | 20.293ms | 20.305ms | 20.287ms | 41.184ms | 20.000ms | 130.05 MiB | none | 492.75K/s |
| q9 | group by and count distinct | 10.000ms | 40.404ms | 40.373ms | 0.2% | 40.372ms | 40.448ms | 20.378ms | 40.452ms | 120.000ms | 167.70 MiB | none | 247.69K/s |
| q10 | group by, several aggregates | 7.000ms | 40.372ms | 20.294ms | 0.0% | 20.293ms | 20.295ms | 20.287ms | 40.365ms | 50.000ms | 156.86 MiB | none | 492.75K/s |
| q11 | group by a string and count distinct | 9.000ms | 20.321ms | 20.287ms | 0.3% | 20.273ms | 20.333ms | 20.272ms | 40.383ms | 80.000ms | 168.93 MiB | none | 492.92K/s |
| q12 | group by two strings and count distinct | 9.000ms | 20.299ms | 40.385ms | 49.8% | 20.314ms | 40.427ms | 20.301ms | 40.446ms | 60.000ms | 193.00 MiB | none | 247.62K/s |
| q13 | group by a string and top k | 10.000ms | 20.271ms | 40.358ms | 49.5% | 20.370ms | 40.360ms | 20.282ms | 40.362ms | 120.000ms | 175.73 MiB | none | 247.78K/s |
| q14 | group by a string and count distinct | 9.000ms | 20.369ms | 40.365ms | 49.7% | 20.316ms | 40.371ms | 20.286ms | 40.448ms | 50.000ms | 196.67 MiB | none | 247.74K/s |
| q15 | group by two columns and top k | 8.000ms | 20.381ms | 20.360ms | 98.6% | 20.294ms | 40.368ms | 20.291ms | 40.382ms | 90.000ms | 176.45 MiB | none | 491.16K/s |
| q16 | group by, very high card | 7.000ms | 20.381ms | 20.289ms | 0.0% | 20.288ms | 20.290ms | 20.284ms | 20.297ms | 90.000ms | 142.53 MiB | none | 492.87K/s |
| q17 | group by two, very high card | 6.000ms | 40.363ms | 20.308ms | 0.0% | 20.301ms | 20.309ms | 20.285ms | 20.317ms | 20.000ms | 148.52 MiB | none | 492.43K/s |
| q18 | group by two, no ordering | 6.000ms | 20.283ms | 20.294ms | 0.0% | 20.292ms | 20.294ms | 20.277ms | 20.305ms | 30.000ms | 153.82 MiB | none | 492.76K/s |
| q19 | group by with an extract | 7.000ms | 20.292ms | 20.290ms | 0.0% | 20.288ms | 20.292ms | 20.281ms | 20.346ms | 40.000ms | 158.57 MiB | none | 492.85K/s |
| q20 | point lookup | 3.000ms | 20.290ms | 20.294ms | 0.1% | 20.284ms | 20.297ms | 20.284ms | 20.301ms | 20.000ms | 117.70 MiB | none | 492.76K/s |
| q21 | substring scan | 4.000ms | 20.291ms | 20.298ms | 0.1% | 20.287ms | 20.299ms | 20.286ms | 24.104ms | 30.000ms | 131.11 MiB | none | 492.67K/s |
| q22 | substring scan and group by | 7.000ms | 20.281ms | 20.288ms | 0.0% | 20.280ms | 20.289ms | 20.266ms | 20.291ms | 30.000ms | 158.36 MiB | none | 492.89K/s |
| q23 | two substring scans and group by | 9.000ms | 40.354ms | 20.295ms | 98.9% | 20.274ms | 40.350ms | 20.274ms | 40.353ms | 30.000ms | 160.08 MiB | none | 492.74K/s |
| q24 | select star and top k | 12.000ms | 40.366ms | 40.384ms | 0.0% | 40.378ms | 40.385ms | 40.362ms | 40.394ms | 30.000ms | 148.56 MiB | none | 247.62K/s |
| q25 | top k by a date | 5.000ms | 20.298ms | 20.290ms | 0.0% | 20.283ms | 20.290ms | 20.267ms | 20.299ms | 20.000ms | 123.98 MiB | none | 492.86K/s |
| q26 | top k by a string | 4.000ms | 20.284ms | 20.296ms | 0.1% | 20.286ms | 20.301ms | 20.282ms | 20.305ms | 20.000ms | 135.36 MiB | none | 492.71K/s |
| q27 | top k by two columns | 4.000ms | 20.288ms | 20.285ms | 0.0% | 20.284ms | 20.288ms | 20.282ms | 20.288ms | 20.000ms | 130.55 MiB | none | 492.96K/s |
| q28 | group by with a string length | 6.000ms | 20.296ms | 20.290ms | 0.1% | 20.286ms | 20.299ms | 20.281ms | 20.299ms | 40.000ms | 159.80 MiB | none | 492.84K/s |
| q29 | group by a regular expression | 13.000ms | 40.379ms | 40.394ms | 0.1% | 40.384ms | 40.408ms | 40.382ms | 40.452ms | 140.000ms | 201.09 MiB | none | 247.56K/s |
| q30 | ninety sums over one column | 11.000ms | 40.403ms | 40.627ms | 1.0% | 40.425ms | 40.828ms | 40.415ms | 41.152ms | 20.000ms | 115.08 MiB | none | 246.14K/s |
| q31 | group by two and several aggregates | 7.000ms | 20.325ms | 20.288ms | 0.0% | 20.284ms | 20.290ms | 20.280ms | 20.317ms | 40.000ms | 163.64 MiB | none | 492.90K/s |
| q32 | group by a high card pair | 8.000ms | 20.285ms | 20.290ms | 0.1% | 20.287ms | 20.299ms | 20.278ms | 40.391ms | 70.000ms | 169.88 MiB | none | 492.85K/s |
| q33 | group by a high card pair, unfiltered | 6.000ms | 20.306ms | 20.290ms | 0.1% | 20.286ms | 20.298ms | 20.282ms | 20.328ms | 30.000ms | 159.56 MiB | none | 492.85K/s |
| q34 | group by a long string | 8.000ms | 20.285ms | 20.295ms | 0.1% | 20.283ms | 20.311ms | 20.278ms | 40.466ms | 40.000ms | 177.54 MiB | none | 492.74K/s |
| q35 | group by a constant and a long string | 9.000ms | 20.289ms | 20.377ms | 98.4% | 20.306ms | 40.365ms | 20.289ms | 40.462ms | 70.000ms | 182.43 MiB | none | 490.75K/s |
| q36 | group by four expressions | 6.000ms | 20.277ms | 20.287ms | 0.1% | 20.279ms | 20.307ms | 20.270ms | 20.307ms | 20.000ms | 136.36 MiB | none | 492.93K/s |
| q37 | date range and group by a URL | 8.000ms | 20.286ms | 20.288ms | 0.0% | 20.285ms | 20.291ms | 20.280ms | 40.378ms | 40.000ms | 163.34 MiB | none | 492.90K/s |
| q38 | date range and group by a title | 9.000ms | 20.288ms | 20.288ms | 99.0% | 20.282ms | 40.357ms | 20.272ms | 40.440ms | 40.000ms | 166.09 MiB | none | 492.90K/s |
| q39 | date range, group by and offset | 10.000ms | 40.376ms | 40.382ms | 49.8% | 20.326ms | 40.455ms | 20.285ms | 40.459ms | 110.000ms | 157.68 MiB | none | 247.63K/s |
| q40 | date range, a case and a wide group by | 9.000ms | 40.385ms | 40.367ms | 49.8% | 20.304ms | 40.392ms | 20.282ms | 40.453ms | 30.000ms | 150.20 MiB | none | 247.73K/s |
| q41 | date range with an IN and a hash | 7.000ms | 20.282ms | 20.301ms | 0.1% | 20.292ms | 20.307ms | 20.286ms | 20.364ms | 50.000ms | 137.17 MiB | none | 492.59K/s |
| q42 | date range and a deep offset | 7.000ms | 20.291ms | 20.280ms | 0.0% | 20.278ms | 20.288ms | 20.272ms | 20.293ms | 50.000ms | 141.31 MiB | none | 493.10K/s |
| q43 | minute buckets over a date range | 8.000ms | 20.292ms | 20.283ms | 0.1% | 20.271ms | 20.291ms | 20.270ms | 20.291ms | 60.000ms | 142.11 MiB | none | 493.02K/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 300.000ms by its own clock and 1.054s by ours, 1.054s cold, 2.000s of CPU, peak 201.09 MiB, 1.43M/s and 353.64 MiB/s.

Running it cost 251% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.00x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 6.645ms | 100.622ms | 100.603ms | 0.0% | 100.597ms | 100.609ms | 100.545ms | 100.961ms | 100.000ms | 62.87 MiB | none | 99.40K/s |
| q2 | filtered count | 7.239ms | 120.860ms | 100.646ms | 0.1% | 100.595ms | 100.697ms | 100.588ms | 127.190ms | 140.000ms | 66.38 MiB | none | 99.36K/s |
| q3 | three aggregates | 6.894ms | 100.591ms | 100.592ms | 0.0% | 100.587ms | 100.606ms | 100.580ms | 100.629ms | 90.000ms | 65.57 MiB | none | 99.41K/s |
| q4 | average | 7.002ms | 100.575ms | 100.594ms | 0.0% | 100.588ms | 100.604ms | 100.578ms | 100.692ms | 90.000ms | 64.84 MiB | none | 99.41K/s |
| q5 | count distinct, high card | 13.276ms | 100.635ms | 100.657ms | 0.2% | 100.596ms | 100.846ms | 100.556ms | 101.060ms | 130.000ms | 74.55 MiB | none | 99.35K/s |
| q6 | count distinct, strings | 12.810ms | 101.004ms | 100.638ms | 21.3% | 100.617ms | 122.057ms | 100.568ms | 130.372ms | 120.000ms | 74.32 MiB | none | 99.37K/s |
| q7 | min and max of a date | 6.500ms | 100.589ms | 100.610ms | 0.0% | 100.609ms | 100.641ms | 100.592ms | 121.665ms | 100.000ms | 64.62 MiB | none | 99.39K/s |
| q8 | group by, low card | 13.804ms | 133.323ms | 120.705ms | 16.7% | 100.653ms | 120.783ms | 100.633ms | 123.156ms | 170.000ms | 75.42 MiB | none | 82.85K/s |
| q9 | group by and count distinct | 22.250ms | 120.665ms | 120.710ms | 0.0% | 120.703ms | 120.737ms | 120.676ms | 120.831ms | 160.000ms | 83.25 MiB | none | 82.84K/s |
| q10 | group by, several aggregates | 25.415ms | 120.795ms | 120.935ms | 0.2% | 120.695ms | 120.962ms | 120.679ms | 121.057ms | 170.000ms | 85.75 MiB | none | 82.69K/s |
| q11 | group by a string and count distinct | 21.503ms | 120.657ms | 120.825ms | 0.4% | 120.689ms | 121.145ms | 120.677ms | 121.306ms | 150.000ms | 82.95 MiB | none | 82.76K/s |
| q12 | group by two strings and count distinct | 22.298ms | 120.728ms | 120.694ms | 0.1% | 120.669ms | 120.795ms | 120.647ms | 121.078ms | 160.000ms | 82.96 MiB | none | 82.85K/s |
| q13 | group by a string and top k | 15.636ms | 122.270ms | 120.872ms | 1.0% | 120.789ms | 122.017ms | 120.697ms | 123.541ms | 130.000ms | 76.30 MiB | none | 82.73K/s |
| q14 | group by a string and count distinct | 21.593ms | 121.326ms | 120.683ms | 0.0% | 120.676ms | 120.696ms | 120.658ms | 120.735ms | 160.000ms | 84.31 MiB | none | 82.86K/s |
| q15 | group by two columns and top k | 17.002ms | 120.545ms | 120.772ms | 0.7% | 120.704ms | 121.537ms | 120.678ms | 123.758ms | 130.000ms | 78.61 MiB | none | 82.80K/s |
| q16 | group by, very high card | 19.355ms | 121.090ms | 120.912ms | 0.3% | 120.694ms | 121.092ms | 120.663ms | 121.669ms | 140.000ms | 75.54 MiB | none | 82.70K/s |
| q17 | group by two, very high card | 21.348ms | 120.686ms | 120.674ms | 0.1% | 120.660ms | 120.738ms | 120.656ms | 120.740ms | 150.000ms | 79.11 MiB | none | 82.87K/s |
| q18 | group by two, no ordering | 13.450ms | 120.813ms | 120.673ms | 16.6% | 100.652ms | 120.676ms | 100.571ms | 121.076ms | 140.000ms | 76.93 MiB | none | 82.87K/s |
| q19 | group by with an extract | 24.043ms | 120.681ms | 120.683ms | 0.0% | 120.666ms | 120.695ms | 120.560ms | 120.762ms | 160.000ms | 80.69 MiB | none | 82.86K/s |
| q20 | point lookup | 6.082ms | 100.610ms | 100.580ms | 0.0% | 100.580ms | 100.602ms | 100.446ms | 100.618ms | 100.000ms | 64.29 MiB | none | 99.42K/s |
| q21 | substring scan | 8.414ms | 100.599ms | 100.593ms | 0.1% | 100.578ms | 100.650ms | 100.545ms | 100.675ms | 100.000ms | 68.72 MiB | none | 99.41K/s |
| q22 | substring scan and group by | 15.492ms | 100.594ms | 120.669ms | 0.2% | 120.658ms | 120.900ms | 100.660ms | 124.368ms | 130.000ms | 78.85 MiB | none | 82.87K/s |
| q23 | two substring scans and group by | 22.192ms | 120.670ms | 120.735ms | 0.1% | 120.686ms | 120.752ms | 120.646ms | 120.835ms | 160.000ms | 89.39 MiB | none | 82.83K/s |
| q24 | select star and top k | 11.595ms | 100.590ms | 100.592ms | 0.0% | 100.591ms | 100.594ms | 100.581ms | 100.614ms | 110.000ms | 76.33 MiB | none | 99.41K/s |
| q25 | top k by a date | 10.608ms | 100.649ms | 100.593ms | 0.0% | 100.573ms | 100.605ms | 100.571ms | 120.678ms | 110.000ms | 72.38 MiB | none | 99.41K/s |
| q26 | top k by a string | 11.271ms | 100.579ms | 100.599ms | 0.1% | 100.576ms | 100.638ms | 100.566ms | 120.668ms | 110.000ms | 71.81 MiB | none | 99.40K/s |
| q27 | top k by two columns | 11.865ms | 100.596ms | 100.579ms | 0.0% | 100.570ms | 100.582ms | 100.569ms | 100.659ms | 110.000ms | 72.30 MiB | none | 99.42K/s |
| q30 | ninety sums over one column | 11.396ms | 100.567ms | 100.593ms | 0.0% | 100.577ms | 100.607ms | 100.558ms | 100.626ms | 120.000ms | 67.64 MiB | none | 99.41K/s |
| q31 | group by two and several aggregates | 15.535ms | 100.583ms | 120.823ms | 0.1% | 120.663ms | 120.827ms | 120.656ms | 121.145ms | 130.000ms | 80.24 MiB | none | 82.77K/s |
| q32 | group by a high card pair | 16.446ms | 121.344ms | 121.026ms | 0.5% | 120.690ms | 121.336ms | 120.669ms | 128.299ms | 130.000ms | 80.12 MiB | none | 82.63K/s |
| q33 | group by a high card pair, unfiltered | 15.761ms | 126.390ms | 120.688ms | 0.1% | 120.662ms | 120.774ms | 100.798ms | 120.786ms | 130.000ms | 80.34 MiB | none | 82.86K/s |
| q34 | group by a long string | 16.891ms | 100.637ms | 120.775ms | 0.1% | 120.690ms | 120.829ms | 120.686ms | 121.540ms | 140.000ms | 80.00 MiB | none | 82.80K/s |
| q35 | group by a constant and a long string | 19.259ms | 120.952ms | 120.738ms | 0.1% | 120.705ms | 120.822ms | 120.648ms | 121.430ms | 150.000ms | 82.46 MiB | none | 82.82K/s |
| q37 | date range and group by a URL | 17.772ms | 121.560ms | 120.829ms | 0.7% | 120.721ms | 121.563ms | 120.699ms | 123.504ms | 140.000ms | 81.43 MiB | none | 82.76K/s |
| q38 | date range and group by a title | 17.076ms | 120.715ms | 121.240ms | 1.0% | 120.644ms | 121.854ms | 100.603ms | 122.112ms | 140.000ms | 81.78 MiB | none | 82.48K/s |
| q39 | date range, group by and offset | 15.229ms | 100.647ms | 100.750ms | 20.2% | 100.673ms | 120.977ms | 100.627ms | 121.527ms | 130.000ms | 79.40 MiB | none | 99.26K/s |
| q40 | date range, a case and a wide group by | 15.605ms | 100.689ms | 120.907ms | 0.2% | 120.662ms | 120.907ms | 100.645ms | 121.289ms | 130.000ms | 81.12 MiB | none | 82.71K/s |
| q41 | date range with an IN and a hash | 14.036ms | 120.679ms | 120.724ms | 16.6% | 100.690ms | 120.727ms | 100.582ms | 120.757ms | 130.000ms | 80.98 MiB | none | 82.83K/s |
| q42 | date range and a deep offset | 14.498ms | 120.639ms | 120.682ms | 16.6% | 101.296ms | 121.345ms | 100.670ms | 124.455ms | 150.000ms | 79.52 MiB | none | 82.86K/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 585.086ms by its own clock and 4.408s by ours, 4.369s cold, 5.140s of CPU, peak 89.39 MiB, 666.57K/s and 164.46 MiB/s.

Running it cost 653% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.21x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 804.648us | 20.328ms | 20.353ms | 0.4% | 20.312ms | 20.402ms | 20.304ms | 20.450ms | 0.000us | 9.29 MiB | none | 491.33K/s |
| q2 | filtered count | 825.597us | 20.324ms | 20.300ms | 0.1% | 20.296ms | 20.306ms | 20.290ms | 20.338ms | 0.000us | 9.80 MiB | none | 492.62K/s |
| q3 | three aggregates | 846.560us | 20.292ms | 20.296ms | 0.1% | 20.295ms | 20.316ms | 20.290ms | 20.342ms | 0.000us | 9.32 MiB | none | 492.70K/s |
| q4 | average | 849.688us | 20.331ms | 20.321ms | 0.3% | 20.311ms | 20.371ms | 20.311ms | 20.596ms | 0.000us | 9.70 MiB | none | 492.10K/s |
| q5 | count distinct, high card | 818.870us | 20.300ms | 20.310ms | 0.1% | 20.305ms | 20.320ms | 20.288ms | 20.347ms | 0.000us | 9.61 MiB | none | 492.38K/s |
| q6 | count distinct, strings | 1.287ms | 20.377ms | 20.310ms | 0.2% | 20.290ms | 20.325ms | 20.288ms | 20.340ms | 0.000us | 9.82 MiB | none | 492.37K/s |
| q7 | min and max of a date | 875.967us | 20.310ms | 20.396ms | 0.2% | 20.362ms | 20.408ms | 20.303ms | 20.499ms | 0.000us | 9.53 MiB | none | 490.29K/s |
| q8 | group by, low card | 939.854us | 20.343ms | 20.321ms | 0.1% | 20.314ms | 20.340ms | 20.312ms | 20.696ms | 0.000us | 9.77 MiB | none | 492.10K/s |
| q9 | group by and count distinct | 1.051ms | 20.336ms | 20.485ms | 1.2% | 20.382ms | 20.623ms | 20.303ms | 20.722ms | 0.000us | 9.80 MiB | none | 488.17K/s |
| q10 | group by, several aggregates | 1.374ms | 20.307ms | 20.310ms | 0.0% | 20.307ms | 20.315ms | 20.300ms | 20.331ms | 0.000us | 9.96 MiB | none | 492.37K/s |
| q11 | group by a string and count distinct | 1.009ms | 20.323ms | 20.312ms | 0.1% | 20.310ms | 20.338ms | 20.304ms | 20.551ms | 0.000us | 9.77 MiB | none | 492.32K/s |
| q12 | group by two strings and count distinct | 1.099ms | 20.298ms | 20.305ms | 0.1% | 20.302ms | 20.313ms | 20.293ms | 20.313ms | 0.000us | 9.82 MiB | none | 492.49K/s |
| q13 | group by a string and top k | 1.286ms | 20.302ms | 20.311ms | 0.1% | 20.310ms | 20.322ms | 20.307ms | 20.384ms | 0.000us | 9.99 MiB | none | 492.35K/s |
| q14 | group by a string and count distinct | 1.514ms | 20.315ms | 20.322ms | 0.0% | 20.319ms | 20.326ms | 20.305ms | 20.654ms | 0.000us | 10.35 MiB | none | 492.09K/s |
| q15 | group by two columns and top k | 1.485ms | 20.355ms | 20.324ms | 0.2% | 20.314ms | 20.360ms | 20.300ms | 20.378ms | 0.000us | 10.09 MiB | none | 492.04K/s |
| q16 | group by, very high card | 1.715ms | 20.317ms | 20.318ms | 0.3% | 20.307ms | 20.372ms | 20.302ms | 20.431ms | 0.000us | 10.32 MiB | none | 492.17K/s |
| q17 | group by two, very high card | 2.283ms | 20.290ms | 20.330ms | 0.1% | 20.315ms | 20.343ms | 20.306ms | 20.551ms | 0.000us | 10.93 MiB | none | 491.89K/s |
| q18 | group by two, no ordering | 1.176ms | 20.827ms | 20.312ms | 0.2% | 20.291ms | 20.326ms | 20.287ms | 20.327ms | 0.000us | 9.45 MiB | none | 492.33K/s |
| q19 | group by with an extract | 2.720ms | 20.391ms | 20.336ms | 0.1% | 20.336ms | 20.357ms | 20.325ms | 20.422ms | 0.000us | 11.27 MiB | none | 491.73K/s |
| q20 | point lookup | 794.705us | 20.367ms | 20.409ms | 0.1% | 20.395ms | 20.409ms | 20.395ms | 20.429ms | 0.000us | 9.78 MiB | none | 489.99K/s |
| q21 | substring scan | 2.073ms | 20.409ms | 20.399ms | 0.4% | 20.361ms | 20.448ms | 20.361ms | 20.659ms | 0.000us | 10.96 MiB | none | 490.22K/s |
| q22 | substring scan and group by | 2.408ms | 20.405ms | 20.403ms | 0.0% | 20.401ms | 20.404ms | 20.399ms | 20.415ms | 0.000us | 11.17 MiB | none | 490.13K/s |
| q23 | two substring scans and group by | 4.348ms | 20.454ms | 20.401ms | 0.1% | 20.390ms | 20.414ms | 20.365ms | 20.477ms | 0.000us | 12.73 MiB | none | 490.16K/s |
| q24 | select star and top k | 2.360ms | 20.408ms | 20.364ms | 0.1% | 20.361ms | 20.372ms | 20.360ms | 20.373ms | 0.000us | 11.84 MiB | none | 491.05K/s |
| q25 | top k by a date | 1.085ms | 20.351ms | 20.360ms | 0.0% | 20.359ms | 20.367ms | 20.357ms | 20.403ms | 0.000us | 10.03 MiB | none | 491.16K/s |
| q26 | top k by a string | 1.023ms | 20.329ms | 20.351ms | 0.1% | 20.347ms | 20.376ms | 20.346ms | 20.387ms | 0.000us | 9.55 MiB | none | 491.37K/s |
| q27 | top k by two columns | 1.119ms | 20.362ms | 20.394ms | 0.2% | 20.364ms | 20.406ms | 20.360ms | 20.727ms | 0.000us | 10.00 MiB | none | 490.35K/s |
| q28 | group by with a string length | 2.403ms | 20.352ms | 20.352ms | 0.1% | 20.342ms | 20.353ms | 20.334ms | 20.369ms | 0.000us | 11.48 MiB | none | 491.36K/s |
| q29 | group by a regular expression | 3.231ms | 20.333ms | 20.351ms | 0.1% | 20.342ms | 20.359ms | 20.341ms | 20.399ms | 0.000us | 11.66 MiB | none | 491.38K/s |
| q30 | ninety sums over one column | 2.267ms | 20.385ms | 20.360ms | 0.1% | 20.349ms | 20.362ms | 20.347ms | 20.399ms | 0.000us | 9.70 MiB | none | 491.16K/s |
| q31 | group by two and several aggregates | 1.862ms | 20.361ms | 20.349ms | 0.1% | 20.346ms | 20.365ms | 20.343ms | 20.432ms | 0.000us | 10.15 MiB | none | 491.43K/s |
| q32 | group by a high card pair | 1.924ms | 20.377ms | 20.373ms | 0.3% | 20.350ms | 20.412ms | 20.349ms | 20.659ms | 0.000us | 10.10 MiB | none | 490.86K/s |
| q33 | group by a high card pair, unfiltered | 1.960ms | 20.365ms | 20.364ms | 0.0% | 20.362ms | 20.367ms | 20.359ms | 20.396ms | 0.000us | 9.82 MiB | none | 491.05K/s |
| q34 | group by a long string | 3.689ms | 20.364ms | 20.382ms | 0.1% | 20.365ms | 20.393ms | 20.346ms | 20.426ms | 0.000us | 13.24 MiB | none | 490.63K/s |
| q35 | group by a constant and a long string | 4.148ms | 20.409ms | 20.414ms | 0.1% | 20.404ms | 20.428ms | 20.385ms | 20.428ms | 0.000us | 13.11 MiB | none | 489.87K/s |
| q36 | group by four expressions | 1.876ms | 20.438ms | 20.458ms | 0.3% | 20.440ms | 20.500ms | 20.417ms | 20.512ms | 0.000us | 10.37 MiB | none | 488.81K/s |
| q37 | date range and group by a URL | 2.512ms | 20.419ms | 20.432ms | 0.1% | 20.407ms | 20.437ms | 20.398ms | 20.659ms | 0.000us | 11.21 MiB | none | 489.43K/s |
| q38 | date range and group by a title | 2.972ms | 20.463ms | 20.430ms | 0.2% | 20.401ms | 20.434ms | 20.386ms | 20.445ms | 0.000us | 11.09 MiB | none | 489.48K/s |
| q39 | date range, group by and offset | 2.418ms | 20.406ms | 20.386ms | 0.0% | 20.384ms | 20.393ms | 20.360ms | 20.399ms | 0.000us | 11.28 MiB | none | 490.52K/s |
| q40 | date range, a case and a wide group by | 3.516ms | 20.370ms | 20.382ms | 0.1% | 20.374ms | 20.388ms | 20.344ms | 20.411ms | 0.000us | 12.16 MiB | none | 490.63K/s |
| q41 | date range with an IN and a hash | 1.266ms | 20.404ms | 20.365ms | 0.1% | 20.352ms | 20.379ms | 20.344ms | 20.387ms | 0.000us | 10.45 MiB | none | 491.05K/s |
| q42 | date range and a deep offset | 1.177ms | 20.363ms | 20.382ms | 0.3% | 20.379ms | 20.437ms | 20.361ms | 20.478ms | 0.000us | 10.22 MiB | none | 490.63K/s |
| q43 | minute buckets over a date range | 1.197ms | 20.385ms | 20.350ms | 0.1% | 20.341ms | 20.372ms | 20.336ms | 20.378ms | 0.000us | 10.31 MiB | none | 491.41K/s |

rudb rudb 0.3.45 over 43 of 43 queries. Total 77.588ms by its own clock and 875.480ms by ours, 875.947ms cold, 0.000us of CPU, peak 13.24 MiB, 5.54M/s and 1.34 GiB/s.

Running it cost 1028% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.01x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | 418.896us | 244.397us | 163.035us | 167.982us | 2.9% | 89.081us | 115.534us | 0.000us | 896 B | 4 of 4 |
| q2 | 364.683us | 256.123us | 210.672us | 215.957us | 2.4% | 86.073us | 114.080us | 0.000us | 896 B | 5 of 5 |
| q3 | 452.428us | 249.831us | 242.783us | 249.209us | 2.6% | 79.857us | 102.834us | 0.000us | 1.22 KiB | 4 of 4 |
| q4 | 350.811us | 250.636us | 257.691us | 263.000us | 2.0% | 74.571us | 106.423us | 0.000us | 896 B | 4 of 4 |
| q5 | 373.062us | 234.696us | 295.808us | 299.838us | 1.3% | 80.145us | 102.373us | 0.000us | 132.25 KiB | 4 of 4 |
| q6 | 357.832us | 753.736us | 837.003us | 843.994us | 0.8% | 87.572us | 112.881us | 0.000us | 372.59 KiB | 5 of 5 |
| q7 | 403.225us | 250.161us | 256.267us | 261.446us | 2.0% | 68.925us | 145.702us | 0.000us | 1.00 KiB | 4 of 4 |
| q8 | 401.961us | 275.647us | 233.249us | 239.563us | 2.6% | 81.526us | 109.444us | 0.000us | 2.31 KiB | 6 of 6 |
| q9 | 368.099us | 409.273us | 487.856us | 494.140us | 1.3% | 74.001us | 101.742us | 0.000us | 491.56 KiB | 5 of 5 |
| q10 | 389.556us | 721.882us | 881.520us | 890.110us | 1.0% | 90.628us | 102.693us | 0.000us | 544.69 KiB | 5 of 5 |
| q11 | 374.127us | 362.508us | 400.594us | 407.478us | 1.7% | 86.446us | 110.532us | 0.000us | 18.50 KiB | 6 of 6 |
| q12 | 393.857us | 464.080us | 549.297us | 555.515us | 1.1% | 228.470us | 113.967us | 0.000us | 12.96 KiB | 6 of 6 |
| q13 | 383.401us | 665.981us | 738.257us | 744.346us | 0.8% | 84.891us | 108.818us | 0.000us | 190.28 KiB | 6 of 6 |
| q14 | 382.374us | 894.399us | 1.000ms | 1.007ms | 0.6% | 83.362us | 108.482us | 0.000us | 371.86 KiB | 6 of 6 |
| q15 | 437.738us | 838.120us | 939.304us | 945.567us | 0.7% | 91.502us | 109.282us | 0.000us | 262.56 KiB | 6 of 6 |
| q16 | 382.235us | 1.102ms | 1.165ms | 1.170ms | 0.4% | 84.574us | 122.073us | 0.000us | 643.16 KiB | 5 of 5 |
| q17 | 399.234us | 1.757ms | 1.958ms | 1.964ms | 0.3% | 105.665us | 117.354us | 0.000us | 820.31 KiB | 5 of 5 |
| q18 | 372.101us | 540.163us | 643.866us | 651.542us | 1.2% | 75.449us | 106.400us | 0.000us | 2.54 KiB | 5 of 5 |
| q19 | 409.244us | 2.025ms | 2.296ms | 2.305ms | 0.4% | 102.569us | 113.939us | 0.000us | 941.88 KiB | 5 of 5 |
| q20 | 361.995us | 294.025us | 184.456us | 188.290us | 2.0% | 71.580us | 111.490us | 0.000us | 0 B | 4 of 4 |
| q21 | 394.213us | 1.470ms | 1.809ms | 1.815ms | 0.4% | 87.946us | 127.116us | 0.000us | 896 B | 5 of 5 |
| q22 | 436.698us | 1.684ms | 2.082ms | 2.089ms | 0.3% | 86.771us | 138.564us | 0.000us | 1.00 KiB | 6 of 6 |
| q23 | 473.915us | 3.515ms | 4.428ms | 4.439ms | 0.2% | 93.337us | 131.975us | 0.000us | 3.39 KiB | 6 of 6 |
| q24 | 610.890us | 1.698ms | 2.140ms | 2.146ms | 0.2% | 81.917us | 192.907us | 0.000us | 0 B | 8 of 8 |
| q25 | 426.776us | 484.099us | 629.285us | 635.345us | 1.0% | 80.843us | 116.649us | 0.000us | 7.66 KiB | 6 of 6 |
| q26 | 366.612us | 372.566us | 480.239us | 486.475us | 1.3% | 75.328us | 111.396us | 0.000us | 7.10 KiB | 5 of 5 |
| q27 | 386.323us | 465.187us | 604.022us | 612.151us | 1.3% | 75.776us | 117.054us | 0.000us | 10.02 KiB | 6 of 6 |
| q28 | 434.160us | 1.798ms | 2.203ms | 2.209ms | 0.3% | 92.395us | 135.599us | 0.000us | 115.16 KiB | 7 of 7 |
| q29 | 443.882us | 2.486ms | 3.133ms | 3.138ms | 0.1% | 86.154us | 124.142us | 0.000us | 530.81 KiB | 7 of 7 |
| q30 | 1.717ms | 315.616us | 283.728us | 295.254us | 3.9% | 101.672us | 127.169us | 0.000us | 29.59 KiB | 4 of 4 |
| q31 | 416.776us | 1.068ms | 1.091ms | 1.103ms | 1.1% | 99.526us | 117.628us | 0.000us | 57.84 KiB | 6 of 6 |
| q32 | 457.619us | 1.029ms | 1.138ms | 1.150ms | 1.0% | 104.719us | 119.248us | 0.000us | 58.19 KiB | 6 of 6 |
| q33 | 424.959us | 1.228ms | 1.219ms | 1.228ms | 0.8% | 100.587us | 112.058us | 0.000us | 316.38 KiB | 5 of 5 |
| q34 | 422.984us | 3.068ms | 3.609ms | 3.616ms | 0.2% | 97.623us | 108.978us | 0.000us | 2.16 MiB | 5 of 5 |
| q35 | 378.938us | 3.095ms | 3.696ms | 3.702ms | 0.2% | 101.575us | 109.977us | 0.000us | 2.16 MiB | 5 of 5 |
| q36 | 470.586us | 1.125ms | 1.191ms | 1.198ms | 0.6% | 103.933us | 118.640us | 0.000us | 579.09 KiB | 6 of 6 |
| q37 | 528.687us | 1.838ms | 1.782ms | 1.793ms | 0.6% | 100.478us | 177.391us | 0.000us | 12.08 KiB | 6 of 6 |
| q38 | 439.153us | 2.249ms | 2.205ms | 2.216ms | 0.5% | 94.330us | 123.891us | 0.000us | 6.65 KiB | 6 of 6 |
| q39 | 435.558us | 1.534ms | 1.496ms | 1.507ms | 0.8% | 89.642us | 124.665us | 0.000us | 3.78 KiB | 6 of 6 |
| q40 | 549.830us | 2.641ms | 2.597ms | 2.611ms | 0.5% | 96.981us | 134.155us | 0.000us | 66.70 KiB | 6 of 6 |
| q41 | 473.839us | 585.993us | 539.014us | 549.468us | 1.9% | 92.940us | 129.169us | 0.000us | 3.19 KiB | 6 of 6 |
| q42 | 520.929us | 507.196us | 468.226us | 481.536us | 2.8% | 115.097us | 124.607us | 0.000us | 3.55 KiB | 6 of 6 |
| q43 | 476.692us | 714.353us | 663.484us | 673.921us | 1.5% | 91.082us | 131.396us | 0.000us | 18.28 KiB | 6 of 6 |

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
| FileScan | 28.283ms | 57.4% | 43 | 0 | 417512 | handed none | 67.7ns | 43 of 43 |
| Aggregate | 17.254ms | 35.0% | 39 | 197368 | 3140 | 87.4ns | 5494.8ns | 39 of 39 |
| Filter | 2.253ms | 4.6% | 28 | 247512 | 30025 | 9.1ns | 75.0ns | 28 of 28 |
| Project | 872.520us | 1.8% | 91 | 206002 | 206002 | 4.2ns | 4.2ns | 91 of 91 |
| TopN | 581.727us | 1.2% | 31 | 5772 | 211 | 100.8ns | 2757.0ns | 31 of 31 |
| Sort | 5.681us | 0.0% | 1 | 6 | 6 | 946.8ns | 946.8ns | 1 of 1 |
| Limit | 0.336us | 0.0% | 1 | 10 | 10 | 33.6ns | 33.6ns | 1 of 1 |
| Fetch | 0.000us | 0.0% | 1 | 0 | 0 | handed none | handed on none | 1 of 1 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is FileScan, and the queries where it cost the most are q23 at 3.980ms, q40 at 2.197ms, q38 at 1.951ms.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 10000 rows, one out of every 10000 of the 99997497 in the full file, which is a development loop rather than the suite
- every query here ran within 1.00x of every other one, so most of what was timed is whatever they have in common rather than the queries

These swung wider than reporting rule two allows:

- clickhouse-local swung by 33.4% of its median on q12, and rule two wants under 10%
- datafusion swung by 99.0% of its median on q38, and rule two wants under 10%
- polars swung by 21.3% of its median on q6, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

These ran every query at about the same speed:

- duckdb ran every query within 1.00x of every other one, and this suite spreads over 6x on an engine it is measuring
- duckdb-pinned ran every query within 1.00x of every other one, and this suite spreads over 6x on an engine it is measuring
- clickhouse-local ran every query within 1.33x of every other one, and this suite spreads over 6x on an engine it is measuring
- polars ran every query within 1.21x of every other one, and this suite spreads over 6x on an engine it is measuring
- rudb ran every query within 1.01x of every other one, and this suite spreads over 6x on an engine it is measuring

A column that flat is not a column about the queries. Read those numbers as an upper bound on the engine and not as a measurement of it.

polars did not run q28, because the SQLContext has no strlen and no character length function.

polars did not run q29, because the SQLContext has no regexp_replace.

polars did not run q36, because the SQLContext rejects the repeated ClientIP output name in the group by.

polars did not run q43, because the SQLContext has no date_trunc.

So the polars column is 39 of 43 queries and its ratio is over the shared ones.

Answers differ, so this is not a comparison: q11: duckdb-pinned does not agree with duckdb: the same 10 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q11: clickhouse-local does not agree with duckdb: the same 10 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q11: datafusion does not agree with duckdb: the same 10 numbers and 1 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q11: polars does not agree with duckdb: the same 10 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q11: rudb does not agree with duckdb: the same 10 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q12: duckdb-pinned does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q12: clickhouse-local does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q12: datafusion does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q12: rudb does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q13: duckdb-pinned does not agree with duckdb: the same 10 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q13: clickhouse-local does not agree with duckdb: the same 10 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q13: datafusion does not agree with duckdb: the same 10 numbers and 1 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q13: polars does not agree with duckdb: the same 10 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q13: rudb does not agree with duckdb: the same 10 numbers and 1 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q14: duckdb-pinned does not agree with duckdb: the same 10 numbers and 1 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q14: clickhouse-local does not agree with duckdb: the same 10 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q14: datafusion does not agree with duckdb: the same 10 numbers and 1 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q14: polars does not agree with duckdb: the same 10 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q14: rudb does not agree with duckdb: the same 10 numbers and 1 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q15: duckdb-pinned does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q15: clickhouse-local does not agree with duckdb: the same 20 numbers and 4 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q15: datafusion does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q15: polars does not agree with duckdb: the same 20 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q15: rudb does not agree with duckdb: the same 20 numbers and 4 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

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

