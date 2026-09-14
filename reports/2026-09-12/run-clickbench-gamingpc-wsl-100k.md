# clickbench on gamingpc-wsl

This is one run of the clickbench suite on gamingpc-wsl, over 7 engines and 43 queries, with 1 hot run of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

It ran over 99998 rows, one out of every 1000 of the 99997497 in the full file, which is a development loop and not the suite. Nothing here is comparable to a full run or to anybody else's number.

## What ran

| what | it was |
| --- | --- |
| suite | clickbench |
| queries | 43 |
| tables | 15.11 MiB of Parquet in 1 table |
| rows | 99998 in the table every query reads |
| sample | 99998 rows, one out of every 1000 of the 99997497 in the full file |
| summary | median with the interquartile range, per reporting rule two |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --rows 100000 --runs 1 --report
```

The data goes under `RUDB_BENCH_DATA`, which defaults to `~/rudb-data`, and the engines have to be on the path. An engine that is not gets a row in the abstentions below rather than being left out of the table.

## The machine

| fact | value | source |
| --- | --- | --- |

A fact that says it was not read is a fact about this machine and not a gap in the report. A field that had silently defaulted would be a lie that survived into it.

## The engines

| engine | version | state | load | load cpu | on disk | that size is | format | machine load |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 556.497ms | 620.000ms | 33.76 MiB | its own database file | its own | 4.03 to 4.03 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 659.306ms | 780.000ms | 28.76 MiB | its own database file | its own | 4.03 to 3.95 |
| clickhouse-local | 26.9.1.1162 | ran | 334.760ms | 330.000ms | 24.32 MiB | its own MergeTree parts, as system.parts counts the active ones | its own | 5.57 to 5.29 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 15.11 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 3.95 to 5.80 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 15.11 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 5.80 to 5.57 |
| clickhouse-server | 26.9.1.1162 | ran | 530.237ms | not read | 23.93 MiB | its own MergeTree parts, sorted by hits by (CounterID, EventDate, UserID, EventTime, WatchID), as system.parts counts the active ones | its own | 5.29 to 5.10 |
| rudb | rudb 0.2.31 | ran | 0.000us | 0.000us | 15.11 MiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 3.95 to 3.95 |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 305.000ms | 701.310ms | +130% | 706.421ms | 670.000ms | 0.96 | 65.89 MiB | none | 14.10M/s | 2.08 GiB/s | 1.00x |
| duckdb-pinned | 332.000ms | 1.261s | +280% | 1.253s | 1.170s | 0.93 | 81.05 MiB | none | 12.95M/s | 1.91 GiB/s | 1.09x |
| clickhouse-local | 455.000ms | 2.691s | +491% | 2.672s | 2.760s | 1.03 | 246.50 MiB | none | 9.45M/s | 1.39 GiB/s | 1.49x |
| datafusion | 565.000ms | 1.055s | +87% | 1.072s | 2.450s | 2.32 | 331.86 MiB | none | 7.61M/s | 1.12 GiB/s | 1.85x |
| polars | 844.594ms | 4.308s | +410% | 4.259s | 8.280s | 1.92 | 136.36 MiB | none | 4.62M/s | 697.91 MiB/s | 2.77x |
| clickhouse-server | 235.000ms | 1.519s | +546% | 1.527s | not read | not read | not read | not read | 18.30M/s | 2.70 GiB/s | 0.77x |
| rudb | 1.074s | 1.136s | +6% | 1.135s | 810.000ms | 0.71 | 158.63 MiB | none | 3.82M/s | 576.98 MiB/s | 3.52x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | clickhouse-server | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 0.000us | 1.000ms | 2.000ms | 1.000ms | 6.157ms | 1.000ms | 0.000us |
| q2 | filtered count | 1.000ms | 1.000ms | 2.000ms | 3.000ms | 6.373ms | 1.000ms | 1.000ms |
| q3 | three aggregates | 1.000ms | 2.000ms | 3.000ms | 4.000ms | 5.887ms | 2.000ms | 2.000ms |
| q4 | average | 1.000ms | 1.000ms | 3.000ms | 3.000ms | 5.761ms | 2.000ms | 2.000ms |
| q5 | count distinct, high card | 6.000ms | 5.000ms | 7.000ms | 8.000ms | 16.356ms | 5.000ms | 12.000ms |
| q6 | count distinct, strings | 5.000ms | 3.000ms | 5.000ms | 9.000ms | 16.673ms | 3.000ms | 9.000ms |
| q7 | min and max of a date | 1.000ms | 1.000ms | 3.000ms | 1.000ms | 6.719ms | 1.000ms | 1.000ms |
| q8 | group by, low card | 1.000ms | 6.000ms | 4.000ms | 5.000ms | 16.350ms | 2.000ms | 1.000ms |
| q9 | group by and count distinct | 7.000ms | 7.000ms | 6.000ms | 11.000ms | 27.647ms | 4.000ms | 17.000ms |
| q10 | group by, several aggregates | 9.000ms | 9.000ms | 6.000ms | 10.000ms | 29.767ms | 18.000ms | 19.000ms |
| q11 | group by a string and count distinct | 4.000ms | 4.000ms | 4.000ms | 9.000ms | 27.109ms | 2.000ms | 3.000ms |
| q12 | group by two strings and count distinct | 5.000ms | 5.000ms | 4.000ms | 10.000ms | 24.633ms | 2.000ms | 4.000ms |
| q13 | group by a string and top k | 4.000ms | 5.000ms | 6.000ms | 12.000ms | 20.120ms | 3.000ms | 13.000ms |
| q14 | group by a string and count distinct | 6.000ms | 6.000ms | 7.000ms | 15.000ms | 26.078ms | 4.000ms | 15.000ms |
| q15 | group by two columns and top k | 5.000ms | 4.000ms | 6.000ms | 10.000ms | 22.810ms | 4.000ms | 14.000ms |
| q16 | group by, very high card | 6.000ms | 5.000ms | 7.000ms | 8.000ms | 19.727ms | 4.000ms | 20.000ms |
| q17 | group by two, very high card | 11.000ms | 10.000ms | 13.000ms | 12.000ms | 30.772ms | 9.000ms | 32.000ms |
| q18 | group by two, no ordering | 11.000ms | 10.000ms | 6.000ms | 12.000ms | 17.793ms | 3.000ms | 25.000ms |
| q19 | group by with an extract | 13.000ms | 11.000ms | 14.000ms | 14.000ms | 29.745ms | 9.000ms | no dialect |
| q20 | point lookup | 0.000us | 1.000ms | 4.000ms | 4.000ms | 5.976ms | 1.000ms | 2.000ms |
| q21 | substring scan | 8.000ms | 8.000ms | 8.000ms | 10.000ms | 24.493ms | 5.000ms | 35.000ms |
| q22 | substring scan and group by | 9.000ms | 10.000ms | 10.000ms | 14.000ms | 32.862ms | 3.000ms | 39.000ms |
| q23 | two substring scans and group by | 11.000ms | 13.000ms | 12.000ms | 31.000ms | 44.922ms | 5.000ms | 82.000ms |
| q24 | select star and top k | 24.000ms | 28.000ms | 115.000ms | 60.000ms | 64.621ms | 28.000ms | 189.000ms |
| q25 | top k by a date | 4.000ms | 3.000ms | 6.000ms | 8.000ms | 13.169ms | 2.000ms | 13.000ms |
| q26 | top k by a string | 2.000ms | 2.000ms | 4.000ms | 7.000ms | 11.530ms | 2.000ms | 11.000ms |
| q27 | top k by two columns | 2.000ms | 3.000ms | 5.000ms | 8.000ms | 16.278ms | 2.000ms | 14.000ms |
| q28 | group by with a string length | 9.000ms | 11.000ms | 5.000ms | 14.000ms | no dialect | 3.000ms | 34.000ms |
| q29 | group by a regular expression | 50.000ms | 54.000ms | 27.000ms | 33.000ms | no dialect | 23.000ms | 83.000ms |
| q30 | ninety sums over one column | 4.000ms | 14.000ms | 8.000ms | 11.000ms | 12.954ms | 6.000ms | 14.000ms |
| q31 | group by two and several aggregates | 4.000ms | 5.000ms | 6.000ms | 12.000ms | 20.731ms | 3.000ms | 15.000ms |
| q32 | group by a high card pair | 5.000ms | 5.000ms | 6.000ms | 11.000ms | 17.457ms | 16.000ms | 15.000ms |
| q33 | group by a high card pair, unfiltered | 9.000ms | 9.000ms | 12.000ms | 11.000ms | 21.287ms | 7.000ms | no dialect |
| q34 | group by a long string | 18.000ms | 17.000ms | 21.000ms | 25.000ms | 31.948ms | 12.000ms | 57.000ms |
| q35 | group by a constant and a long string | 21.000ms | 19.000ms | 20.000ms | 25.000ms | 35.246ms | 11.000ms | 65.000ms |
| q36 | group by four expressions | 7.000ms | 5.000ms | 7.000ms | 10.000ms | no dialect | 4.000ms | 33.000ms |
| q37 | date range and group by a URL | 3.000ms | 3.000ms | 7.000ms | 17.000ms | 28.841ms | 4.000ms | 32.000ms |
| q38 | date range and group by a title | 3.000ms | 4.000ms | 7.000ms | 31.000ms | 28.772ms | 3.000ms | 37.000ms |
| q39 | date range, group by and offset | 3.000ms | 4.000ms | 31.000ms | 20.000ms | 21.035ms | 3.000ms | 33.000ms |
| q40 | date range, a case and a wide group by | 4.000ms | 5.000ms | 9.000ms | 22.000ms | 22.280ms | 4.000ms | 61.000ms |
| q41 | date range with an IN and a hash | 3.000ms | 3.000ms | 6.000ms | 8.000ms | 16.592ms | 3.000ms | 7.000ms |
| q42 | date range and a deep offset | 3.000ms | 7.000ms | 6.000ms | 8.000ms | 17.123ms | 3.000ms | 7.000ms |
| q43 | minute buckets over a date range | 2.000ms | 3.000ms | 5.000ms | 8.000ms | no dialect | 3.000ms | 6.000ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 0.000us | 9.603ms | 9.300ms | 0.0% | 9.300ms | 9.300ms | 9.300ms | 9.300ms | 0.000us | 26.95 MiB | none | 10.75M/s |
| q2 | filtered count | 1.000ms | 10.189ms | 9.746ms | 0.0% | 9.746ms | 9.746ms | 9.746ms | 9.746ms | 10.000ms | 27.98 MiB | none | 10.26M/s |
| q3 | three aggregates | 1.000ms | 10.226ms | 9.484ms | 0.0% | 9.484ms | 9.484ms | 9.484ms | 9.484ms | 0.000us | 28.24 MiB | none | 10.54M/s |
| q4 | average | 1.000ms | 10.509ms | 10.709ms | 0.0% | 10.709ms | 10.709ms | 10.709ms | 10.709ms | 10.000ms | 28.32 MiB | none | 9.34M/s |
| q5 | count distinct, high card | 6.000ms | 15.249ms | 14.492ms | 0.0% | 14.492ms | 14.492ms | 14.492ms | 14.492ms | 20.000ms | 34.20 MiB | none | 6.90M/s |
| q6 | count distinct, strings | 5.000ms | 14.176ms | 13.950ms | 0.0% | 13.950ms | 13.950ms | 13.950ms | 13.950ms | 10.000ms | 31.97 MiB | none | 7.17M/s |
| q7 | min and max of a date | 1.000ms | 9.353ms | 9.112ms | 0.0% | 9.112ms | 9.112ms | 9.112ms | 9.112ms | 0.000us | 26.99 MiB | none | 10.97M/s |
| q8 | group by, low card | 1.000ms | 9.908ms | 9.727ms | 0.0% | 9.727ms | 9.727ms | 9.727ms | 9.727ms | 10.000ms | 29.25 MiB | none | 10.28M/s |
| q9 | group by and count distinct | 7.000ms | 16.594ms | 16.406ms | 0.0% | 16.406ms | 16.406ms | 16.406ms | 16.406ms | 20.000ms | 42.50 MiB | none | 6.10M/s |
| q10 | group by, several aggregates | 9.000ms | 18.405ms | 17.735ms | 0.0% | 17.735ms | 17.735ms | 17.735ms | 17.735ms | 20.000ms | 46.00 MiB | none | 5.64M/s |
| q11 | group by a string and count distinct | 4.000ms | 13.607ms | 14.048ms | 0.0% | 14.048ms | 14.048ms | 14.048ms | 14.048ms | 10.000ms | 36.69 MiB | none | 7.12M/s |
| q12 | group by two strings and count distinct | 5.000ms | 13.194ms | 13.278ms | 0.0% | 13.278ms | 13.278ms | 13.278ms | 13.278ms | 20.000ms | 39.00 MiB | none | 7.53M/s |
| q13 | group by a string and top k | 4.000ms | 13.067ms | 13.573ms | 0.0% | 13.573ms | 13.573ms | 13.573ms | 13.573ms | 10.000ms | 33.25 MiB | none | 7.37M/s |
| q14 | group by a string and count distinct | 6.000ms | 15.642ms | 15.775ms | 0.0% | 15.775ms | 15.775ms | 15.775ms | 15.775ms | 20.000ms | 44.25 MiB | none | 6.34M/s |
| q15 | group by two columns and top k | 5.000ms | 13.812ms | 13.239ms | 0.0% | 13.239ms | 13.239ms | 13.239ms | 13.239ms | 10.000ms | 35.25 MiB | none | 7.55M/s |
| q16 | group by, very high card | 6.000ms | 15.498ms | 14.907ms | 0.0% | 14.907ms | 14.907ms | 14.907ms | 14.907ms | 10.000ms | 39.80 MiB | none | 6.71M/s |
| q17 | group by two, very high card | 11.000ms | 20.800ms | 20.535ms | 0.0% | 20.535ms | 20.535ms | 20.535ms | 20.535ms | 20.000ms | 47.09 MiB | none | 4.87M/s |
| q18 | group by two, no ordering | 11.000ms | 22.995ms | 20.722ms | 0.0% | 20.722ms | 20.722ms | 20.722ms | 20.722ms | 30.000ms | 56.00 MiB | none | 4.83M/s |
| q19 | group by with an extract | 13.000ms | 20.339ms | 22.254ms | 0.0% | 22.254ms | 22.254ms | 22.254ms | 22.254ms | 60.000ms | 49.57 MiB | none | 4.49M/s |
| q20 | point lookup | 0.000us | 10.161ms | 9.617ms | 0.0% | 9.617ms | 9.617ms | 9.617ms | 9.617ms | 10.000ms | 28.07 MiB | none | 10.40M/s |
| q21 | substring scan | 8.000ms | 16.430ms | 17.363ms | 0.0% | 17.363ms | 17.363ms | 17.363ms | 17.363ms | 10.000ms | 33.98 MiB | none | 5.76M/s |
| q22 | substring scan and group by | 9.000ms | 17.746ms | 17.841ms | 0.0% | 17.841ms | 17.841ms | 17.841ms | 17.841ms | 20.000ms | 37.55 MiB | none | 5.60M/s |
| q23 | two substring scans and group by | 11.000ms | 20.778ms | 20.421ms | 0.0% | 20.421ms | 20.421ms | 20.421ms | 20.421ms | 20.000ms | 43.75 MiB | none | 4.90M/s |
| q24 | select star and top k | 24.000ms | 34.513ms | 34.695ms | 0.0% | 34.695ms | 34.695ms | 34.695ms | 34.695ms | 50.000ms | 65.89 MiB | none | 2.88M/s |
| q25 | top k by a date | 4.000ms | 12.217ms | 12.100ms | 0.0% | 12.100ms | 12.100ms | 12.100ms | 12.100ms | 10.000ms | 31.73 MiB | none | 8.26M/s |
| q26 | top k by a string | 2.000ms | 10.906ms | 10.723ms | 0.0% | 10.723ms | 10.723ms | 10.723ms | 10.723ms | 10.000ms | 28.48 MiB | none | 9.33M/s |
| q27 | top k by two columns | 2.000ms | 11.249ms | 10.791ms | 0.0% | 10.791ms | 10.791ms | 10.791ms | 10.791ms | 0.000us | 29.72 MiB | none | 9.27M/s |
| q28 | group by with a string length | 9.000ms | 18.297ms | 18.549ms | 0.0% | 18.549ms | 18.549ms | 18.549ms | 18.549ms | 20.000ms | 37.00 MiB | none | 5.39M/s |
| q29 | group by a regular expression | 50.000ms | 60.967ms | 60.524ms | 0.0% | 60.524ms | 60.524ms | 60.524ms | 60.524ms | 60.000ms | 45.96 MiB | none | 1.65M/s |
| q30 | ninety sums over one column | 4.000ms | 13.160ms | 14.021ms | 0.0% | 14.021ms | 14.021ms | 14.021ms | 14.021ms | 0.000us | 31.48 MiB | none | 7.13M/s |
| q31 | group by two and several aggregates | 4.000ms | 13.901ms | 13.707ms | 0.0% | 13.707ms | 13.707ms | 13.707ms | 13.707ms | 0.000us | 37.01 MiB | none | 7.30M/s |
| q32 | group by a high card pair | 5.000ms | 13.690ms | 13.578ms | 0.0% | 13.578ms | 13.578ms | 13.578ms | 13.578ms | 10.000ms | 36.82 MiB | none | 7.36M/s |
| q33 | group by a high card pair, unfiltered | 9.000ms | 18.683ms | 18.120ms | 0.0% | 18.120ms | 18.120ms | 18.120ms | 18.120ms | 10.000ms | 43.85 MiB | none | 5.52M/s |
| q34 | group by a long string | 18.000ms | 29.111ms | 28.081ms | 0.0% | 28.081ms | 28.081ms | 28.081ms | 28.081ms | 20.000ms | 56.77 MiB | none | 3.56M/s |
| q35 | group by a constant and a long string | 21.000ms | 29.222ms | 30.356ms | 0.0% | 30.356ms | 30.356ms | 30.356ms | 30.356ms | 50.000ms | 59.82 MiB | none | 3.29M/s |
| q36 | group by four expressions | 7.000ms | 17.215ms | 16.947ms | 0.0% | 16.947ms | 16.947ms | 16.947ms | 16.947ms | 30.000ms | 44.07 MiB | none | 5.90M/s |
| q37 | date range and group by a URL | 3.000ms | 12.346ms | 12.251ms | 0.0% | 12.251ms | 12.251ms | 12.251ms | 12.251ms | 10.000ms | 33.33 MiB | none | 8.16M/s |
| q38 | date range and group by a title | 3.000ms | 12.234ms | 11.628ms | 0.0% | 11.628ms | 11.628ms | 11.628ms | 11.628ms | 0.000us | 32.25 MiB | none | 8.60M/s |
| q39 | date range, group by and offset | 3.000ms | 12.206ms | 13.007ms | 0.0% | 13.007ms | 13.007ms | 13.007ms | 13.007ms | 10.000ms | 33.25 MiB | none | 7.69M/s |
| q40 | date range, a case and a wide group by | 4.000ms | 12.967ms | 12.829ms | 0.0% | 12.829ms | 12.829ms | 12.829ms | 12.829ms | 10.000ms | 34.75 MiB | none | 7.79M/s |
| q41 | date range with an IN and a hash | 3.000ms | 11.880ms | 11.695ms | 0.0% | 11.695ms | 11.695ms | 11.695ms | 11.695ms | 0.000us | 34.75 MiB | none | 8.55M/s |
| q42 | date range and a deep offset | 3.000ms | 11.867ms | 11.772ms | 0.0% | 11.772ms | 11.772ms | 11.772ms | 11.772ms | 10.000ms | 33.46 MiB | none | 8.49M/s |
| q43 | minute buckets over a date range | 2.000ms | 11.509ms | 11.702ms | 0.0% | 11.702ms | 11.702ms | 11.702ms | 11.702ms | 10.000ms | 31.98 MiB | none | 8.55M/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 305.000ms by its own clock and 701.310ms by ours, 706.421ms cold, 670.000ms of CPU, peak 65.89 MiB, 14.10M/s and 2.08 GiB/s.

Running it cost 130% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 6.64x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 22.301ms | 21.697ms | 0.0% | 21.697ms | 21.697ms | 21.697ms | 21.697ms | 10.000ms | 39.79 MiB | none | 4.61M/s |
| q2 | filtered count | 1.000ms | 22.375ms | 21.796ms | 0.0% | 21.796ms | 21.796ms | 21.796ms | 21.796ms | 10.000ms | 40.06 MiB | none | 4.59M/s |
| q3 | three aggregates | 2.000ms | 22.267ms | 22.318ms | 0.0% | 22.318ms | 22.318ms | 22.318ms | 22.318ms | 20.000ms | 40.32 MiB | none | 4.48M/s |
| q4 | average | 1.000ms | 22.137ms | 21.613ms | 0.0% | 21.613ms | 21.613ms | 21.613ms | 21.613ms | 10.000ms | 41.07 MiB | none | 4.63M/s |
| q5 | count distinct, high card | 5.000ms | 24.770ms | 25.838ms | 0.0% | 25.838ms | 25.838ms | 25.838ms | 25.838ms | 20.000ms | 46.71 MiB | none | 3.87M/s |
| q6 | count distinct, strings | 3.000ms | 24.367ms | 26.955ms | 0.0% | 26.955ms | 26.955ms | 26.955ms | 26.955ms | 40.000ms | 44.20 MiB | none | 3.71M/s |
| q7 | min and max of a date | 1.000ms | 22.387ms | 21.571ms | 0.0% | 21.571ms | 21.571ms | 21.571ms | 21.571ms | 10.000ms | 39.85 MiB | none | 4.64M/s |
| q8 | group by, low card | 6.000ms | 26.893ms | 26.159ms | 0.0% | 26.159ms | 26.159ms | 26.159ms | 26.159ms | 20.000ms | 42.78 MiB | none | 3.82M/s |
| q9 | group by and count distinct | 7.000ms | 28.591ms | 28.745ms | 0.0% | 28.745ms | 28.745ms | 28.745ms | 28.745ms | 30.000ms | 53.17 MiB | none | 3.48M/s |
| q10 | group by, several aggregates | 9.000ms | 31.969ms | 30.277ms | 0.0% | 30.277ms | 30.277ms | 30.277ms | 30.277ms | 40.000ms | 59.30 MiB | none | 3.30M/s |
| q11 | group by a string and count distinct | 4.000ms | 25.457ms | 24.756ms | 0.0% | 24.756ms | 24.756ms | 24.756ms | 24.756ms | 20.000ms | 49.85 MiB | none | 4.04M/s |
| q12 | group by two strings and count distinct | 5.000ms | 27.121ms | 26.839ms | 0.0% | 26.839ms | 26.839ms | 26.839ms | 26.839ms | 20.000ms | 48.55 MiB | none | 3.73M/s |
| q13 | group by a string and top k | 5.000ms | 25.190ms | 25.765ms | 0.0% | 25.765ms | 25.765ms | 25.765ms | 25.765ms | 20.000ms | 45.39 MiB | none | 3.88M/s |
| q14 | group by a string and count distinct | 6.000ms | 28.304ms | 27.975ms | 0.0% | 27.975ms | 27.975ms | 27.975ms | 27.975ms | 30.000ms | 53.08 MiB | none | 3.57M/s |
| q15 | group by two columns and top k | 4.000ms | 25.378ms | 25.538ms | 0.0% | 25.538ms | 25.538ms | 25.538ms | 25.538ms | 20.000ms | 46.01 MiB | none | 3.92M/s |
| q16 | group by, very high card | 5.000ms | 25.508ms | 26.106ms | 0.0% | 26.106ms | 26.106ms | 26.106ms | 26.106ms | 20.000ms | 51.59 MiB | none | 3.83M/s |
| q17 | group by two, very high card | 10.000ms | 31.892ms | 31.732ms | 0.0% | 31.732ms | 31.732ms | 31.732ms | 31.732ms | 30.000ms | 59.84 MiB | none | 3.15M/s |
| q18 | group by two, no ordering | 10.000ms | 32.136ms | 31.859ms | 0.0% | 31.859ms | 31.859ms | 31.859ms | 31.859ms | 40.000ms | 69.39 MiB | none | 3.14M/s |
| q19 | group by with an extract | 11.000ms | 32.803ms | 34.166ms | 0.0% | 34.166ms | 34.166ms | 34.166ms | 34.166ms | 30.000ms | 62.36 MiB | none | 2.93M/s |
| q20 | point lookup | 1.000ms | 22.617ms | 21.904ms | 0.0% | 21.904ms | 21.904ms | 21.904ms | 21.904ms | 10.000ms | 39.82 MiB | none | 4.57M/s |
| q21 | substring scan | 8.000ms | 30.173ms | 29.795ms | 0.0% | 29.795ms | 29.795ms | 29.795ms | 29.795ms | 30.000ms | 47.06 MiB | none | 3.36M/s |
| q22 | substring scan and group by | 10.000ms | 31.424ms | 32.307ms | 0.0% | 32.307ms | 32.307ms | 32.307ms | 32.307ms | 30.000ms | 50.83 MiB | none | 3.10M/s |
| q23 | two substring scans and group by | 13.000ms | 37.232ms | 34.100ms | 0.0% | 34.100ms | 34.100ms | 34.100ms | 34.100ms | 30.000ms | 56.10 MiB | none | 2.93M/s |
| q24 | select star and top k | 28.000ms | 49.160ms | 51.094ms | 0.0% | 51.094ms | 51.094ms | 51.094ms | 51.094ms | 50.000ms | 81.05 MiB | none | 1.96M/s |
| q25 | top k by a date | 3.000ms | 23.319ms | 24.064ms | 0.0% | 24.064ms | 24.064ms | 24.064ms | 24.064ms | 20.000ms | 42.71 MiB | none | 4.16M/s |
| q26 | top k by a string | 2.000ms | 23.456ms | 24.067ms | 0.0% | 24.067ms | 24.067ms | 24.067ms | 24.067ms | 20.000ms | 41.83 MiB | none | 4.15M/s |
| q27 | top k by two columns | 3.000ms | 23.670ms | 24.918ms | 0.0% | 24.918ms | 24.918ms | 24.918ms | 24.918ms | 10.000ms | 42.71 MiB | none | 4.01M/s |
| q28 | group by with a string length | 11.000ms | 32.299ms | 31.838ms | 0.0% | 31.838ms | 31.838ms | 31.838ms | 31.838ms | 30.000ms | 50.26 MiB | none | 3.14M/s |
| q29 | group by a regular expression | 54.000ms | 73.851ms | 76.499ms | 0.0% | 76.499ms | 76.499ms | 76.499ms | 76.499ms | 120.000ms | 54.62 MiB | none | 1.31M/s |
| q30 | ninety sums over one column | 14.000ms | 35.773ms | 37.743ms | 0.0% | 37.743ms | 37.743ms | 37.743ms | 37.743ms | 40.000ms | 53.20 MiB | none | 2.65M/s |
| q31 | group by two and several aggregates | 5.000ms | 25.879ms | 27.678ms | 0.0% | 27.678ms | 27.678ms | 27.678ms | 27.678ms | 30.000ms | 48.39 MiB | none | 3.61M/s |
| q32 | group by a high card pair | 5.000ms | 26.962ms | 26.069ms | 0.0% | 26.069ms | 26.069ms | 26.069ms | 26.069ms | 20.000ms | 48.77 MiB | none | 3.84M/s |
| q33 | group by a high card pair, unfiltered | 9.000ms | 28.934ms | 30.045ms | 0.0% | 30.045ms | 30.045ms | 30.045ms | 30.045ms | 30.000ms | 61.03 MiB | none | 3.33M/s |
| q34 | group by a long string | 17.000ms | 38.700ms | 40.074ms | 0.0% | 40.074ms | 40.074ms | 40.074ms | 40.074ms | 50.000ms | 71.01 MiB | none | 2.50M/s |
| q35 | group by a constant and a long string | 19.000ms | 40.421ms | 41.549ms | 0.0% | 41.549ms | 41.549ms | 41.549ms | 41.549ms | 40.000ms | 72.89 MiB | none | 2.41M/s |
| q36 | group by four expressions | 5.000ms | 25.882ms | 25.838ms | 0.0% | 25.838ms | 25.838ms | 25.838ms | 25.838ms | 30.000ms | 50.07 MiB | none | 3.87M/s |
| q37 | date range and group by a URL | 3.000ms | 24.739ms | 26.079ms | 0.0% | 26.079ms | 26.079ms | 26.079ms | 26.079ms | 20.000ms | 46.27 MiB | none | 3.83M/s |
| q38 | date range and group by a title | 4.000ms | 23.866ms | 25.283ms | 0.0% | 25.283ms | 25.283ms | 25.283ms | 25.283ms | 20.000ms | 45.33 MiB | none | 3.96M/s |
| q39 | date range, group by and offset | 4.000ms | 27.025ms | 25.218ms | 0.0% | 25.218ms | 25.218ms | 25.218ms | 25.218ms | 20.000ms | 45.27 MiB | none | 3.97M/s |
| q40 | date range, a case and a wide group by | 5.000ms | 27.518ms | 26.771ms | 0.0% | 26.771ms | 26.771ms | 26.771ms | 26.771ms | 20.000ms | 48.83 MiB | none | 3.74M/s |
| q41 | date range with an IN and a hash | 3.000ms | 25.366ms | 24.850ms | 0.0% | 24.850ms | 24.850ms | 24.850ms | 24.850ms | 20.000ms | 46.28 MiB | none | 4.02M/s |
| q42 | date range and a deep offset | 7.000ms | 28.372ms | 27.414ms | 0.0% | 27.414ms | 27.414ms | 27.414ms | 27.414ms | 20.000ms | 45.52 MiB | none | 3.65M/s |
| q43 | minute buckets over a date range | 3.000ms | 24.171ms | 24.143ms | 0.0% | 24.143ms | 24.143ms | 24.143ms | 24.143ms | 20.000ms | 43.82 MiB | none | 4.14M/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 332.000ms by its own clock and 1.261s by ours, 1.253s cold, 1.170s of CPU, peak 81.05 MiB, 12.95M/s and 1.91 GiB/s.

Running it cost 280% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.55x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 2.000ms | 48.060ms | 48.547ms | 0.0% | 48.547ms | 48.547ms | 48.547ms | 48.547ms | 50.000ms | 200.80 MiB | none | 2.06M/s |
| q2 | filtered count | 2.000ms | 48.174ms | 47.897ms | 0.0% | 47.897ms | 47.897ms | 47.897ms | 47.897ms | 60.000ms | 200.54 MiB | none | 2.09M/s |
| q3 | three aggregates | 3.000ms | 54.940ms | 54.510ms | 0.0% | 54.510ms | 54.510ms | 54.510ms | 54.510ms | 50.000ms | 204.32 MiB | none | 1.83M/s |
| q4 | average | 3.000ms | 61.111ms | 50.116ms | 0.0% | 50.116ms | 50.116ms | 50.116ms | 50.116ms | 50.000ms | 203.57 MiB | none | 2.00M/s |
| q5 | count distinct, high card | 7.000ms | 59.174ms | 59.360ms | 0.0% | 59.360ms | 59.360ms | 59.360ms | 59.360ms | 60.000ms | 209.05 MiB | none | 1.68M/s |
| q6 | count distinct, strings | 5.000ms | 52.420ms | 51.695ms | 0.0% | 51.695ms | 51.695ms | 51.695ms | 51.695ms | 50.000ms | 206.28 MiB | none | 1.93M/s |
| q7 | min and max of a date | 3.000ms | 49.364ms | 50.828ms | 0.0% | 50.828ms | 50.828ms | 50.828ms | 50.828ms | 60.000ms | 203.07 MiB | none | 1.97M/s |
| q8 | group by, low card | 4.000ms | 51.208ms | 51.505ms | 0.0% | 51.505ms | 51.505ms | 51.505ms | 51.505ms | 50.000ms | 206.32 MiB | none | 1.94M/s |
| q9 | group by and count distinct | 6.000ms | 52.745ms | 52.028ms | 0.0% | 52.028ms | 52.028ms | 52.028ms | 52.028ms | 50.000ms | 209.74 MiB | none | 1.92M/s |
| q10 | group by, several aggregates | 6.000ms | 54.074ms | 52.997ms | 0.0% | 52.997ms | 52.997ms | 52.997ms | 52.997ms | 50.000ms | 210.82 MiB | none | 1.89M/s |
| q11 | group by a string and count distinct | 4.000ms | 53.592ms | 51.383ms | 0.0% | 51.383ms | 51.383ms | 51.383ms | 51.383ms | 50.000ms | 207.56 MiB | none | 1.95M/s |
| q12 | group by two strings and count distinct | 4.000ms | 50.522ms | 57.078ms | 0.0% | 57.078ms | 57.078ms | 57.078ms | 57.078ms | 60.000ms | 207.57 MiB | none | 1.75M/s |
| q13 | group by a string and top k | 6.000ms | 52.047ms | 57.598ms | 0.0% | 57.598ms | 57.598ms | 57.598ms | 57.598ms | 60.000ms | 210.32 MiB | none | 1.74M/s |
| q14 | group by a string and count distinct | 7.000ms | 62.210ms | 60.405ms | 0.0% | 60.405ms | 60.405ms | 60.405ms | 60.405ms | 60.000ms | 213.41 MiB | none | 1.66M/s |
| q15 | group by two columns and top k | 6.000ms | 53.884ms | 59.000ms | 0.0% | 59.000ms | 59.000ms | 59.000ms | 59.000ms | 60.000ms | 212.07 MiB | none | 1.69M/s |
| q16 | group by, very high card | 7.000ms | 56.298ms | 53.928ms | 0.0% | 53.928ms | 53.928ms | 53.928ms | 53.928ms | 50.000ms | 210.82 MiB | none | 1.85M/s |
| q17 | group by two, very high card | 13.000ms | 69.357ms | 66.514ms | 0.0% | 66.514ms | 66.514ms | 66.514ms | 66.514ms | 60.000ms | 221.74 MiB | none | 1.50M/s |
| q18 | group by two, no ordering | 6.000ms | 59.310ms | 60.654ms | 0.0% | 60.654ms | 60.654ms | 60.654ms | 60.654ms | 70.000ms | 210.82 MiB | none | 1.65M/s |
| q19 | group by with an extract | 14.000ms | 67.339ms | 68.488ms | 0.0% | 68.488ms | 68.488ms | 68.488ms | 68.488ms | 70.000ms | 223.12 MiB | none | 1.46M/s |
| q20 | point lookup | 4.000ms | 55.251ms | 50.962ms | 0.0% | 50.962ms | 50.962ms | 50.962ms | 50.962ms | 50.000ms | 204.05 MiB | none | 1.96M/s |
| q21 | substring scan | 8.000ms | 61.166ms | 61.081ms | 0.0% | 61.081ms | 61.081ms | 61.081ms | 61.081ms | 60.000ms | 208.54 MiB | none | 1.64M/s |
| q22 | substring scan and group by | 10.000ms | 63.303ms | 64.484ms | 0.0% | 64.484ms | 64.484ms | 64.484ms | 64.484ms | 70.000ms | 213.32 MiB | none | 1.55M/s |
| q23 | two substring scans and group by | 12.000ms | 67.071ms | 67.584ms | 0.0% | 67.584ms | 67.584ms | 67.584ms | 67.584ms | 70.000ms | 214.67 MiB | none | 1.48M/s |
| q24 | select star and top k | 115.000ms | 179.829ms | 174.761ms | 0.0% | 174.761ms | 174.761ms | 174.761ms | 174.761ms | 130.000ms | 246.50 MiB | none | 572.20K/s |
| q25 | top k by a date | 6.000ms | 57.422ms | 57.951ms | 0.0% | 57.951ms | 57.951ms | 57.951ms | 57.951ms | 50.000ms | 207.57 MiB | none | 1.73M/s |
| q26 | top k by a string | 4.000ms | 58.251ms | 59.799ms | 0.0% | 59.799ms | 59.799ms | 59.799ms | 59.799ms | 70.000ms | 206.32 MiB | none | 1.67M/s |
| q27 | top k by two columns | 5.000ms | 52.780ms | 58.812ms | 0.0% | 58.812ms | 58.812ms | 58.812ms | 58.812ms | 60.000ms | 207.82 MiB | none | 1.70M/s |
| q28 | group by with a string length | 5.000ms | 57.709ms | 61.166ms | 0.0% | 61.166ms | 61.166ms | 61.166ms | 61.166ms | 60.000ms | 208.72 MiB | none | 1.63M/s |
| q29 | group by a regular expression | 27.000ms | 81.100ms | 78.945ms | 0.0% | 78.945ms | 78.945ms | 78.945ms | 78.945ms | 80.000ms | 239.01 MiB | none | 1.27M/s |
| q30 | ninety sums over one column | 8.000ms | 61.433ms | 63.188ms | 0.0% | 63.188ms | 63.188ms | 63.188ms | 63.188ms | 70.000ms | 207.23 MiB | none | 1.58M/s |
| q31 | group by two and several aggregates | 6.000ms | 73.092ms | 57.287ms | 0.0% | 57.287ms | 57.287ms | 57.287ms | 57.287ms | 70.000ms | 210.56 MiB | none | 1.75M/s |
| q32 | group by a high card pair | 6.000ms | 55.191ms | 59.078ms | 0.0% | 59.078ms | 59.078ms | 59.078ms | 59.078ms | 50.000ms | 212.07 MiB | none | 1.69M/s |
| q33 | group by a high card pair, unfiltered | 12.000ms | 59.248ms | 70.846ms | 0.0% | 70.846ms | 70.846ms | 70.846ms | 70.846ms | 150.000ms | 221.53 MiB | none | 1.41M/s |
| q34 | group by a long string | 21.000ms | 75.473ms | 75.780ms | 0.0% | 75.780ms | 75.780ms | 75.780ms | 75.780ms | 90.000ms | 238.05 MiB | none | 1.32M/s |
| q35 | group by a constant and a long string | 20.000ms | 76.035ms | 74.575ms | 0.0% | 74.575ms | 74.575ms | 74.575ms | 74.575ms | 70.000ms | 237.41 MiB | none | 1.34M/s |
| q36 | group by four expressions | 7.000ms | 60.471ms | 59.447ms | 0.0% | 59.447ms | 59.447ms | 59.447ms | 59.447ms | 60.000ms | 211.57 MiB | none | 1.68M/s |
| q37 | date range and group by a URL | 7.000ms | 60.955ms | 54.988ms | 0.0% | 54.988ms | 54.988ms | 54.988ms | 54.988ms | 50.000ms | 212.05 MiB | none | 1.82M/s |
| q38 | date range and group by a title | 7.000ms | 60.671ms | 62.507ms | 0.0% | 62.507ms | 62.507ms | 62.507ms | 62.507ms | 70.000ms | 211.06 MiB | none | 1.60M/s |
| q39 | date range, group by and offset | 31.000ms | 61.499ms | 83.966ms | 0.0% | 83.966ms | 83.966ms | 83.966ms | 83.966ms | 70.000ms | 212.06 MiB | none | 1.19M/s |
| q40 | date range, a case and a wide group by | 9.000ms | 63.624ms | 62.060ms | 0.0% | 62.060ms | 62.060ms | 62.060ms | 62.060ms | 60.000ms | 214.77 MiB | none | 1.61M/s |
| q41 | date range with an IN and a hash | 6.000ms | 56.594ms | 59.451ms | 0.0% | 59.451ms | 59.451ms | 59.451ms | 59.451ms | 60.000ms | 209.46 MiB | none | 1.68M/s |
| q42 | date range and a deep offset | 6.000ms | 57.599ms | 57.860ms | 0.0% | 57.860ms | 57.860ms | 57.860ms | 57.860ms | 60.000ms | 208.82 MiB | none | 1.73M/s |
| q43 | minute buckets over a date range | 5.000ms | 60.460ms | 59.966ms | 0.0% | 59.966ms | 59.966ms | 59.966ms | 59.966ms | 60.000ms | 208.61 MiB | none | 1.67M/s |

clickhouse-local 26.9.1.1162 over 43 of 43 queries. Total 455.000ms by its own clock and 2.691s by ours, 2.672s cold, 2.760s of CPU, peak 246.50 MiB, 9.45M/s and 1.39 GiB/s.

Running it cost 491% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.65x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 12.560ms | 13.193ms | 0.0% | 13.193ms | 13.193ms | 13.193ms | 13.193ms | 10.000ms | 80.53 MiB | none | 7.58M/s |
| q2 | filtered count | 3.000ms | 15.156ms | 14.828ms | 0.0% | 14.828ms | 14.828ms | 14.828ms | 14.828ms | 20.000ms | 103.99 MiB | none | 6.74M/s |
| q3 | three aggregates | 4.000ms | 15.393ms | 14.274ms | 0.0% | 14.274ms | 14.274ms | 14.274ms | 14.274ms | 10.000ms | 109.66 MiB | none | 7.01M/s |
| q4 | average | 3.000ms | 14.736ms | 13.248ms | 0.0% | 13.248ms | 13.248ms | 13.248ms | 13.248ms | 10.000ms | 105.67 MiB | none | 7.55M/s |
| q5 | count distinct, high card | 8.000ms | 22.625ms | 18.597ms | 0.0% | 18.597ms | 18.597ms | 18.597ms | 18.597ms | 30.000ms | 152.94 MiB | none | 5.38M/s |
| q6 | count distinct, strings | 9.000ms | 21.390ms | 19.841ms | 0.0% | 19.841ms | 19.841ms | 19.841ms | 19.841ms | 40.000ms | 168.09 MiB | none | 5.04M/s |
| q7 | min and max of a date | 1.000ms | 12.142ms | 11.183ms | 0.0% | 11.183ms | 11.183ms | 11.183ms | 11.183ms | 0.000us | 80.06 MiB | none | 8.94M/s |
| q8 | group by, low card | 5.000ms | 15.922ms | 16.561ms | 0.0% | 16.561ms | 16.561ms | 16.561ms | 16.561ms | 30.000ms | 124.65 MiB | none | 6.04M/s |
| q9 | group by and count distinct | 11.000ms | 24.055ms | 21.939ms | 0.0% | 21.939ms | 21.939ms | 21.939ms | 21.939ms | 60.000ms | 194.30 MiB | none | 4.56M/s |
| q10 | group by, several aggregates | 10.000ms | 21.698ms | 21.269ms | 0.0% | 21.269ms | 21.269ms | 21.269ms | 21.269ms | 50.000ms | 165.18 MiB | none | 4.70M/s |
| q11 | group by a string and count distinct | 9.000ms | 20.767ms | 20.360ms | 0.0% | 20.360ms | 20.360ms | 20.360ms | 20.360ms | 60.000ms | 187.36 MiB | none | 4.91M/s |
| q12 | group by two strings and count distinct | 10.000ms | 22.681ms | 20.550ms | 0.0% | 20.550ms | 20.550ms | 20.550ms | 20.550ms | 70.000ms | 193.41 MiB | none | 4.87M/s |
| q13 | group by a string and top k | 12.000ms | 22.732ms | 22.894ms | 0.0% | 22.894ms | 22.894ms | 22.894ms | 22.894ms | 90.000ms | 194.88 MiB | none | 4.37M/s |
| q14 | group by a string and count distinct | 15.000ms | 26.732ms | 26.828ms | 0.0% | 26.828ms | 26.828ms | 26.828ms | 26.828ms | 140.000ms | 225.08 MiB | none | 3.73M/s |
| q15 | group by two columns and top k | 10.000ms | 23.222ms | 21.651ms | 0.0% | 21.651ms | 21.651ms | 21.651ms | 21.651ms | 30.000ms | 179.56 MiB | none | 4.62M/s |
| q16 | group by, very high card | 8.000ms | 18.654ms | 19.629ms | 0.0% | 19.629ms | 19.629ms | 19.629ms | 19.629ms | 30.000ms | 144.54 MiB | none | 5.09M/s |
| q17 | group by two, very high card | 12.000ms | 22.514ms | 22.586ms | 0.0% | 22.586ms | 22.586ms | 22.586ms | 22.586ms | 30.000ms | 161.82 MiB | none | 4.43M/s |
| q18 | group by two, no ordering | 12.000ms | 22.971ms | 23.739ms | 0.0% | 23.739ms | 23.739ms | 23.739ms | 23.739ms | 40.000ms | 181.80 MiB | none | 4.21M/s |
| q19 | group by with an extract | 14.000ms | 29.881ms | 26.117ms | 0.0% | 26.117ms | 26.117ms | 26.117ms | 26.117ms | 40.000ms | 223.24 MiB | none | 3.83M/s |
| q20 | point lookup | 4.000ms | 14.298ms | 14.543ms | 0.0% | 14.543ms | 14.543ms | 14.543ms | 14.543ms | 20.000ms | 108.93 MiB | none | 6.88M/s |
| q21 | substring scan | 10.000ms | 22.468ms | 21.018ms | 0.0% | 21.018ms | 21.018ms | 21.018ms | 21.018ms | 20.000ms | 127.29 MiB | none | 4.76M/s |
| q22 | substring scan and group by | 14.000ms | 25.581ms | 25.585ms | 0.0% | 25.585ms | 25.585ms | 25.585ms | 25.585ms | 20.000ms | 146.20 MiB | none | 3.91M/s |
| q23 | two substring scans and group by | 31.000ms | 42.906ms | 42.577ms | 0.0% | 42.577ms | 42.577ms | 42.577ms | 42.577ms | 60.000ms | 179.23 MiB | none | 2.35M/s |
| q24 | select star and top k | 60.000ms | 75.115ms | 73.817ms | 0.0% | 73.817ms | 73.817ms | 73.817ms | 73.817ms | 70.000ms | 212.02 MiB | none | 1.35M/s |
| q25 | top k by a date | 8.000ms | 19.956ms | 18.869ms | 0.0% | 18.869ms | 18.869ms | 18.869ms | 18.869ms | 20.000ms | 121.19 MiB | none | 5.30M/s |
| q26 | top k by a string | 7.000ms | 17.200ms | 18.391ms | 0.0% | 18.391ms | 18.391ms | 18.391ms | 18.391ms | 20.000ms | 120.39 MiB | none | 5.44M/s |
| q27 | top k by two columns | 8.000ms | 19.206ms | 18.965ms | 0.0% | 18.965ms | 18.965ms | 18.965ms | 18.965ms | 10.000ms | 123.51 MiB | none | 5.27M/s |
| q28 | group by with a string length | 14.000ms | 24.784ms | 25.279ms | 0.0% | 25.279ms | 25.279ms | 25.279ms | 25.279ms | 40.000ms | 150.42 MiB | none | 3.96M/s |
| q29 | group by a regular expression | 33.000ms | 43.280ms | 45.397ms | 0.0% | 45.397ms | 45.397ms | 45.397ms | 45.397ms | 60.000ms | 191.72 MiB | none | 2.20M/s |
| q30 | ninety sums over one column | 11.000ms | 23.933ms | 22.435ms | 0.0% | 22.435ms | 22.435ms | 22.435ms | 22.435ms | 10.000ms | 114.93 MiB | none | 4.46M/s |
| q31 | group by two and several aggregates | 12.000ms | 22.117ms | 23.511ms | 0.0% | 23.511ms | 23.511ms | 23.511ms | 23.511ms | 40.000ms | 154.37 MiB | none | 4.25M/s |
| q32 | group by a high card pair | 11.000ms | 22.640ms | 22.887ms | 0.0% | 22.887ms | 22.887ms | 22.887ms | 22.887ms | 30.000ms | 166.06 MiB | none | 4.37M/s |
| q33 | group by a high card pair, unfiltered | 11.000ms | 25.243ms | 23.065ms | 0.0% | 23.065ms | 23.065ms | 23.065ms | 23.065ms | 50.000ms | 190.30 MiB | none | 4.34M/s |
| q34 | group by a long string | 25.000ms | 37.089ms | 37.329ms | 0.0% | 37.329ms | 37.329ms | 37.329ms | 37.329ms | 150.000ms | 331.86 MiB | none | 2.68M/s |
| q35 | group by a constant and a long string | 25.000ms | 36.214ms | 36.978ms | 0.0% | 36.978ms | 36.978ms | 36.978ms | 36.978ms | 180.000ms | 328.13 MiB | none | 2.70M/s |
| q36 | group by four expressions | 10.000ms | 22.181ms | 21.524ms | 0.0% | 21.524ms | 21.524ms | 21.524ms | 21.524ms | 70.000ms | 182.54 MiB | none | 4.65M/s |
| q37 | date range and group by a URL | 17.000ms | 28.480ms | 28.812ms | 0.0% | 28.812ms | 28.812ms | 28.812ms | 28.812ms | 150.000ms | 181.86 MiB | none | 3.47M/s |
| q38 | date range and group by a title | 31.000ms | 40.166ms | 42.884ms | 0.0% | 42.884ms | 42.884ms | 42.884ms | 42.884ms | 290.000ms | 188.70 MiB | none | 2.33M/s |
| q39 | date range, group by and offset | 20.000ms | 29.126ms | 31.050ms | 0.0% | 31.050ms | 31.050ms | 31.050ms | 31.050ms | 220.000ms | 166.38 MiB | none | 3.22M/s |
| q40 | date range, a case and a wide group by | 22.000ms | 35.471ms | 33.192ms | 0.0% | 33.192ms | 33.192ms | 33.192ms | 33.192ms | 60.000ms | 171.17 MiB | none | 3.01M/s |
| q41 | date range with an IN and a hash | 8.000ms | 19.940ms | 19.037ms | 0.0% | 19.037ms | 19.037ms | 19.037ms | 19.037ms | 20.000ms | 136.97 MiB | none | 5.25M/s |
| q42 | date range and a deep offset | 8.000ms | 19.376ms | 19.660ms | 0.0% | 19.660ms | 19.660ms | 19.660ms | 19.660ms | 30.000ms | 136.09 MiB | none | 5.09M/s |
| q43 | minute buckets over a date range | 8.000ms | 19.168ms | 19.089ms | 0.0% | 19.089ms | 19.089ms | 19.089ms | 19.089ms | 20.000ms | 128.02 MiB | none | 5.24M/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 565.000ms by its own clock and 1.055s by ours, 1.072s cold, 2.450s of CPU, peak 331.86 MiB, 7.61M/s and 1.12 GiB/s.

Running it cost 87% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 6.60x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 6.157ms | 93.899ms | 90.214ms | 0.0% | 90.214ms | 90.214ms | 90.214ms | 90.214ms | 100.000ms | 62.64 MiB | none | 1.11M/s |
| q2 | filtered count | 6.373ms | 104.812ms | 104.223ms | 0.0% | 104.223ms | 104.223ms | 104.223ms | 104.223ms | 550.000ms | 66.66 MiB | none | 959.46K/s |
| q3 | three aggregates | 5.887ms | 89.639ms | 85.881ms | 0.0% | 85.881ms | 85.881ms | 85.881ms | 85.881ms | 80.000ms | 66.25 MiB | none | 1.16M/s |
| q4 | average | 5.761ms | 94.218ms | 100.914ms | 0.0% | 100.914ms | 100.914ms | 100.914ms | 100.914ms | 390.000ms | 65.71 MiB | none | 990.92K/s |
| q5 | count distinct, high card | 16.356ms | 113.637ms | 101.327ms | 0.0% | 101.327ms | 101.327ms | 101.327ms | 101.327ms | 170.000ms | 80.75 MiB | none | 986.88K/s |
| q6 | count distinct, strings | 16.673ms | 99.994ms | 104.728ms | 0.0% | 104.728ms | 104.728ms | 104.728ms | 104.728ms | 210.000ms | 80.88 MiB | none | 954.84K/s |
| q7 | min and max of a date | 6.719ms | 93.065ms | 101.643ms | 0.0% | 101.643ms | 101.643ms | 101.643ms | 101.643ms | 370.000ms | 64.23 MiB | none | 983.82K/s |
| q8 | group by, low card | 16.350ms | 99.404ms | 105.356ms | 0.0% | 105.356ms | 105.356ms | 105.356ms | 105.356ms | 190.000ms | 76.51 MiB | none | 949.14K/s |
| q9 | group by and count distinct | 27.647ms | 120.253ms | 114.400ms | 0.0% | 114.400ms | 114.400ms | 114.400ms | 114.400ms | 180.000ms | 100.93 MiB | none | 874.11K/s |
| q10 | group by, several aggregates | 29.767ms | 116.405ms | 117.345ms | 0.0% | 117.345ms | 117.345ms | 117.345ms | 117.345ms | 180.000ms | 102.79 MiB | none | 852.17K/s |
| q11 | group by a string and count distinct | 27.109ms | 111.749ms | 115.426ms | 0.0% | 115.426ms | 115.426ms | 115.426ms | 115.426ms | 180.000ms | 84.73 MiB | none | 866.34K/s |
| q12 | group by two strings and count distinct | 24.633ms | 117.116ms | 123.891ms | 0.0% | 123.891ms | 123.891ms | 123.891ms | 123.891ms | 550.000ms | 85.98 MiB | none | 807.14K/s |
| q13 | group by a string and top k | 20.120ms | 103.166ms | 116.062ms | 0.0% | 116.062ms | 116.062ms | 116.062ms | 116.062ms | 300.000ms | 82.83 MiB | none | 861.59K/s |
| q14 | group by a string and count distinct | 26.078ms | 116.712ms | 114.184ms | 0.0% | 114.184ms | 114.184ms | 114.184ms | 114.184ms | 180.000ms | 96.10 MiB | none | 875.76K/s |
| q15 | group by two columns and top k | 22.810ms | 115.188ms | 110.706ms | 0.0% | 110.706ms | 110.706ms | 110.706ms | 110.706ms | 160.000ms | 84.66 MiB | none | 903.28K/s |
| q16 | group by, very high card | 19.727ms | 110.880ms | 108.318ms | 0.0% | 108.318ms | 108.318ms | 108.318ms | 108.318ms | 170.000ms | 85.35 MiB | none | 923.19K/s |
| q17 | group by two, very high card | 30.772ms | 111.876ms | 127.999ms | 0.0% | 127.999ms | 127.999ms | 127.999ms | 127.999ms | 410.000ms | 100.29 MiB | none | 781.24K/s |
| q18 | group by two, no ordering | 17.793ms | 105.504ms | 104.638ms | 0.0% | 104.638ms | 104.638ms | 104.638ms | 104.638ms | 140.000ms | 97.88 MiB | none | 955.66K/s |
| q19 | group by with an extract | 29.745ms | 116.677ms | 117.674ms | 0.0% | 117.674ms | 117.674ms | 117.674ms | 117.674ms | 170.000ms | 102.65 MiB | none | 849.79K/s |
| q20 | point lookup | 5.976ms | 101.244ms | 99.228ms | 0.0% | 99.228ms | 99.228ms | 99.228ms | 99.228ms | 420.000ms | 65.26 MiB | none | 1.01M/s |
| q21 | substring scan | 24.493ms | 109.296ms | 113.311ms | 0.0% | 113.311ms | 113.311ms | 113.311ms | 113.311ms | 230.000ms | 78.29 MiB | none | 882.51K/s |
| q22 | substring scan and group by | 32.862ms | 120.679ms | 121.051ms | 0.0% | 121.051ms | 121.051ms | 121.051ms | 121.051ms | 140.000ms | 88.61 MiB | none | 826.08K/s |
| q23 | two substring scans and group by | 44.922ms | 132.195ms | 134.628ms | 0.0% | 134.628ms | 134.628ms | 134.628ms | 134.628ms | 220.000ms | 118.84 MiB | none | 742.77K/s |
| q24 | select star and top k | 64.621ms | 146.718ms | 153.870ms | 0.0% | 153.870ms | 153.870ms | 153.870ms | 153.870ms | 320.000ms | 119.65 MiB | none | 649.89K/s |
| q25 | top k by a date | 13.169ms | 100.831ms | 98.770ms | 0.0% | 98.770ms | 98.770ms | 98.770ms | 98.770ms | 110.000ms | 75.07 MiB | none | 1.01M/s |
| q26 | top k by a string | 11.530ms | 95.127ms | 100.026ms | 0.0% | 100.026ms | 100.026ms | 100.026ms | 100.026ms | 160.000ms | 72.25 MiB | none | 999.72K/s |
| q27 | top k by two columns | 16.278ms | 110.490ms | 100.757ms | 0.0% | 100.757ms | 100.757ms | 100.757ms | 100.757ms | 190.000ms | 75.72 MiB | none | 992.47K/s |
| q30 | ninety sums over one column | 12.954ms | 101.679ms | 96.275ms | 0.0% | 96.275ms | 96.275ms | 96.275ms | 96.275ms | 110.000ms | 68.54 MiB | none | 1.04M/s |
| q31 | group by two and several aggregates | 20.731ms | 108.875ms | 106.790ms | 0.0% | 106.790ms | 106.790ms | 106.790ms | 106.790ms | 140.000ms | 84.04 MiB | none | 936.40K/s |
| q32 | group by a high card pair | 17.457ms | 104.220ms | 102.873ms | 0.0% | 102.873ms | 102.873ms | 102.873ms | 102.873ms | 150.000ms | 84.86 MiB | none | 972.05K/s |
| q33 | group by a high card pair, unfiltered | 21.287ms | 107.353ms | 109.959ms | 0.0% | 109.959ms | 109.959ms | 109.959ms | 109.959ms | 150.000ms | 105.21 MiB | none | 909.41K/s |
| q34 | group by a long string | 31.948ms | 119.767ms | 120.762ms | 0.0% | 120.762ms | 120.762ms | 120.762ms | 120.762ms | 170.000ms | 129.65 MiB | none | 828.06K/s |
| q35 | group by a constant and a long string | 35.246ms | 119.318ms | 124.706ms | 0.0% | 124.706ms | 124.706ms | 124.706ms | 124.706ms | 170.000ms | 136.36 MiB | none | 801.87K/s |
| q37 | date range and group by a URL | 28.841ms | 116.752ms | 116.395ms | 0.0% | 116.395ms | 116.395ms | 116.395ms | 116.395ms | 140.000ms | 102.30 MiB | none | 859.13K/s |
| q38 | date range and group by a title | 28.772ms | 116.542ms | 116.213ms | 0.0% | 116.213ms | 116.213ms | 116.213ms | 116.213ms | 160.000ms | 104.56 MiB | none | 860.47K/s |
| q39 | date range, group by and offset | 21.035ms | 107.302ms | 110.660ms | 0.0% | 110.660ms | 110.660ms | 110.660ms | 110.660ms | 170.000ms | 88.98 MiB | none | 903.65K/s |
| q40 | date range, a case and a wide group by | 22.280ms | 109.928ms | 107.990ms | 0.0% | 107.990ms | 107.990ms | 107.990ms | 107.990ms | 150.000ms | 96.32 MiB | none | 925.99K/s |
| q41 | date range with an IN and a hash | 16.592ms | 98.838ms | 102.731ms | 0.0% | 102.731ms | 102.731ms | 102.731ms | 102.731ms | 120.000ms | 83.63 MiB | none | 973.40K/s |
| q42 | date range and a deep offset | 17.123ms | 97.975ms | 106.444ms | 0.0% | 106.444ms | 106.444ms | 106.444ms | 106.444ms | 180.000ms | 81.39 MiB | none | 939.44K/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 844.594ms by its own clock and 4.308s by ours, 4.259s cold, 8.280s of CPU, peak 136.36 MiB, 4.62M/s and 697.91 MiB/s.

Running it cost 410% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.79x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-server in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 30.696ms | 31.051ms | 0.0% | 31.051ms | 31.051ms | 31.051ms | 31.051ms | not read | not read | not read | 3.22M/s |
| q2 | filtered count | 1.000ms | 31.092ms | 29.489ms | 0.0% | 29.489ms | 29.489ms | 29.489ms | 29.489ms | not read | not read | not read | 3.39M/s |
| q3 | three aggregates | 2.000ms | 29.973ms | 30.398ms | 0.0% | 30.398ms | 30.398ms | 30.398ms | 30.398ms | not read | not read | not read | 3.29M/s |
| q4 | average | 2.000ms | 31.437ms | 31.101ms | 0.0% | 31.101ms | 31.101ms | 31.101ms | 31.101ms | not read | not read | not read | 3.22M/s |
| q5 | count distinct, high card | 5.000ms | 35.529ms | 34.276ms | 0.0% | 34.276ms | 34.276ms | 34.276ms | 34.276ms | not read | not read | not read | 2.92M/s |
| q6 | count distinct, strings | 3.000ms | 32.967ms | 31.548ms | 0.0% | 31.548ms | 31.548ms | 31.548ms | 31.548ms | not read | not read | not read | 3.17M/s |
| q7 | min and max of a date | 1.000ms | 31.318ms | 31.032ms | 0.0% | 31.032ms | 31.032ms | 31.032ms | 31.032ms | not read | not read | not read | 3.22M/s |
| q8 | group by, low card | 2.000ms | 31.584ms | 32.103ms | 0.0% | 32.103ms | 32.103ms | 32.103ms | 32.103ms | not read | not read | not read | 3.11M/s |
| q9 | group by and count distinct | 4.000ms | 34.000ms | 33.828ms | 0.0% | 33.828ms | 33.828ms | 33.828ms | 33.828ms | not read | not read | not read | 2.96M/s |
| q10 | group by, several aggregates | 18.000ms | 34.441ms | 48.140ms | 0.0% | 48.140ms | 48.140ms | 48.140ms | 48.140ms | not read | not read | not read | 2.08M/s |
| q11 | group by a string and count distinct | 2.000ms | 31.418ms | 32.499ms | 0.0% | 32.499ms | 32.499ms | 32.499ms | 32.499ms | not read | not read | not read | 3.08M/s |
| q12 | group by two strings and count distinct | 2.000ms | 31.559ms | 30.680ms | 0.0% | 30.680ms | 30.680ms | 30.680ms | 30.680ms | not read | not read | not read | 3.26M/s |
| q13 | group by a string and top k | 3.000ms | 35.411ms | 34.827ms | 0.0% | 34.827ms | 34.827ms | 34.827ms | 34.827ms | not read | not read | not read | 2.87M/s |
| q14 | group by a string and count distinct | 4.000ms | 34.496ms | 35.536ms | 0.0% | 35.536ms | 35.536ms | 35.536ms | 35.536ms | not read | not read | not read | 2.81M/s |
| q15 | group by two columns and top k | 4.000ms | 34.337ms | 34.147ms | 0.0% | 34.147ms | 34.147ms | 34.147ms | 34.147ms | not read | not read | not read | 2.93M/s |
| q16 | group by, very high card | 4.000ms | 35.470ms | 34.447ms | 0.0% | 34.447ms | 34.447ms | 34.447ms | 34.447ms | not read | not read | not read | 2.90M/s |
| q17 | group by two, very high card | 9.000ms | 39.181ms | 38.019ms | 0.0% | 38.019ms | 38.019ms | 38.019ms | 38.019ms | not read | not read | not read | 2.63M/s |
| q18 | group by two, no ordering | 3.000ms | 33.243ms | 32.330ms | 0.0% | 32.330ms | 32.330ms | 32.330ms | 32.330ms | not read | not read | not read | 3.09M/s |
| q19 | group by with an extract | 9.000ms | 37.912ms | 39.468ms | 0.0% | 39.468ms | 39.468ms | 39.468ms | 39.468ms | not read | not read | not read | 2.53M/s |
| q20 | point lookup | 1.000ms | 31.285ms | 31.460ms | 0.0% | 31.460ms | 31.460ms | 31.460ms | 31.460ms | not read | not read | not read | 3.18M/s |
| q21 | substring scan | 5.000ms | 35.115ms | 34.078ms | 0.0% | 34.078ms | 34.078ms | 34.078ms | 34.078ms | not read | not read | not read | 2.93M/s |
| q22 | substring scan and group by | 3.000ms | 35.989ms | 32.435ms | 0.0% | 32.435ms | 32.435ms | 32.435ms | 32.435ms | not read | not read | not read | 3.08M/s |
| q23 | two substring scans and group by | 5.000ms | 37.110ms | 34.397ms | 0.0% | 34.397ms | 34.397ms | 34.397ms | 34.397ms | not read | not read | not read | 2.91M/s |
| q24 | select star and top k | 28.000ms | 61.239ms | 53.336ms | 0.0% | 53.336ms | 53.336ms | 53.336ms | 53.336ms | not read | not read | not read | 1.87M/s |
| q25 | top k by a date | 2.000ms | 43.548ms | 32.171ms | 0.0% | 32.171ms | 32.171ms | 32.171ms | 32.171ms | not read | not read | not read | 3.11M/s |
| q26 | top k by a string | 2.000ms | 30.913ms | 31.411ms | 0.0% | 31.411ms | 31.411ms | 31.411ms | 31.411ms | not read | not read | not read | 3.18M/s |
| q27 | top k by two columns | 2.000ms | 30.913ms | 31.861ms | 0.0% | 31.861ms | 31.861ms | 31.861ms | 31.861ms | not read | not read | not read | 3.14M/s |
| q28 | group by with a string length | 3.000ms | 34.815ms | 32.868ms | 0.0% | 32.868ms | 32.868ms | 32.868ms | 32.868ms | not read | not read | not read | 3.04M/s |
| q29 | group by a regular expression | 23.000ms | 52.813ms | 53.846ms | 0.0% | 53.846ms | 53.846ms | 53.846ms | 53.846ms | not read | not read | not read | 1.86M/s |
| q30 | ninety sums over one column | 6.000ms | 37.399ms | 36.001ms | 0.0% | 36.001ms | 36.001ms | 36.001ms | 36.001ms | not read | not read | not read | 2.78M/s |
| q31 | group by two and several aggregates | 3.000ms | 32.488ms | 32.953ms | 0.0% | 32.953ms | 32.953ms | 32.953ms | 32.953ms | not read | not read | not read | 3.03M/s |
| q32 | group by a high card pair | 16.000ms | 33.338ms | 46.408ms | 0.0% | 46.408ms | 46.408ms | 46.408ms | 46.408ms | not read | not read | not read | 2.15M/s |
| q33 | group by a high card pair, unfiltered | 7.000ms | 37.872ms | 36.954ms | 0.0% | 36.954ms | 36.954ms | 36.954ms | 36.954ms | not read | not read | not read | 2.71M/s |
| q34 | group by a long string | 12.000ms | 44.039ms | 42.130ms | 0.0% | 42.130ms | 42.130ms | 42.130ms | 42.130ms | not read | not read | not read | 2.37M/s |
| q35 | group by a constant and a long string | 11.000ms | 43.493ms | 40.104ms | 0.0% | 40.104ms | 40.104ms | 40.104ms | 40.104ms | not read | not read | not read | 2.49M/s |
| q36 | group by four expressions | 4.000ms | 36.216ms | 38.364ms | 0.0% | 38.364ms | 38.364ms | 38.364ms | 38.364ms | not read | not read | not read | 2.61M/s |
| q37 | date range and group by a URL | 4.000ms | 32.692ms | 35.337ms | 0.0% | 35.337ms | 35.337ms | 35.337ms | 35.337ms | not read | not read | not read | 2.83M/s |
| q38 | date range and group by a title | 3.000ms | 34.700ms | 33.348ms | 0.0% | 33.348ms | 33.348ms | 33.348ms | 33.348ms | not read | not read | not read | 3.00M/s |
| q39 | date range, group by and offset | 3.000ms | 33.084ms | 32.597ms | 0.0% | 32.597ms | 32.597ms | 32.597ms | 32.597ms | not read | not read | not read | 3.07M/s |
| q40 | date range, a case and a wide group by | 4.000ms | 33.556ms | 34.311ms | 0.0% | 34.311ms | 34.311ms | 34.311ms | 34.311ms | not read | not read | not read | 2.91M/s |
| q41 | date range with an IN and a hash | 3.000ms | 34.219ms | 32.937ms | 0.0% | 32.937ms | 32.937ms | 32.937ms | 32.937ms | not read | not read | not read | 3.04M/s |
| q42 | date range and a deep offset | 3.000ms | 32.836ms | 32.350ms | 0.0% | 32.350ms | 32.350ms | 32.350ms | 32.350ms | not read | not read | not read | 3.09M/s |
| q43 | minute buckets over a date range | 3.000ms | 31.459ms | 31.994ms | 0.0% | 31.994ms | 31.994ms | 31.994ms | 31.994ms | not read | not read | not read | 3.13M/s |

clickhouse-server 26.9.1.1162 over 43 of 43 queries. Total 235.000ms by its own clock and 1.519s by ours, 1.527s cold, no reading of CPU, peak not read, 18.30M/s and 2.70 GiB/s.

Running it cost 546% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.83x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 0.000us | 1.468ms | 1.407ms | 0.0% | 1.407ms | 1.407ms | 1.407ms | 1.407ms | 0.000us | 4.27 MiB | none | 71.07M/s |
| q2 | filtered count | 1.000ms | 1.982ms | 1.932ms | 0.0% | 1.932ms | 1.932ms | 1.932ms | 1.932ms | 0.000us | 4.96 MiB | none | 51.76M/s |
| q3 | three aggregates | 2.000ms | 3.049ms | 3.028ms | 0.0% | 3.028ms | 3.028ms | 3.028ms | 3.028ms | 0.000us | 5.96 MiB | none | 33.02M/s |
| q4 | average | 2.000ms | 2.923ms | 2.855ms | 0.0% | 2.855ms | 2.855ms | 2.855ms | 2.855ms | 0.000us | 7.72 MiB | none | 35.03M/s |
| q5 | count distinct, high card | 12.000ms | 13.768ms | 13.825ms | 0.0% | 13.825ms | 13.825ms | 13.825ms | 13.825ms | 0.000us | 14.88 MiB | none | 7.23M/s |
| q6 | count distinct, strings | 9.000ms | 9.852ms | 9.727ms | 0.0% | 9.727ms | 9.727ms | 9.727ms | 9.727ms | 0.000us | 7.28 MiB | none | 10.28M/s |
| q7 | min and max of a date | 1.000ms | 2.112ms | 2.071ms | 0.0% | 2.071ms | 2.071ms | 2.071ms | 2.071ms | 0.000us | 5.01 MiB | none | 48.28M/s |
| q8 | group by, low card | 1.000ms | 1.925ms | 1.907ms | 0.0% | 1.907ms | 1.907ms | 1.907ms | 1.907ms | 0.000us | 5.15 MiB | none | 52.44M/s |
| q9 | group by and count distinct | 17.000ms | 18.248ms | 18.048ms | 0.0% | 18.048ms | 18.048ms | 18.048ms | 18.048ms | 10.000ms | 16.10 MiB | none | 5.54M/s |
| q10 | group by, several aggregates | 19.000ms | 20.624ms | 20.592ms | 0.0% | 20.592ms | 20.592ms | 20.592ms | 20.592ms | 10.000ms | 17.15 MiB | none | 4.86M/s |
| q11 | group by a string and count distinct | 3.000ms | 4.587ms | 4.485ms | 0.0% | 4.485ms | 4.485ms | 4.485ms | 4.485ms | 0.000us | 7.76 MiB | none | 22.30M/s |
| q12 | group by two strings and count distinct | 4.000ms | 5.246ms | 5.170ms | 0.0% | 5.170ms | 5.170ms | 5.170ms | 5.170ms | 0.000us | 7.74 MiB | none | 19.34M/s |
| q13 | group by a string and top k | 13.000ms | 14.645ms | 14.297ms | 0.0% | 14.297ms | 14.297ms | 14.297ms | 14.297ms | 10.000ms | 10.29 MiB | none | 6.99M/s |
| q14 | group by a string and count distinct | 15.000ms | 16.873ms | 16.595ms | 0.0% | 16.595ms | 16.595ms | 16.595ms | 16.595ms | 10.000ms | 13.43 MiB | none | 6.03M/s |
| q15 | group by two columns and top k | 14.000ms | 14.512ms | 15.249ms | 0.0% | 15.249ms | 15.249ms | 15.249ms | 15.249ms | 10.000ms | 11.55 MiB | none | 6.56M/s |
| q16 | group by, very high card | 20.000ms | 19.556ms | 21.042ms | 0.0% | 21.042ms | 21.042ms | 21.042ms | 21.042ms | 10.000ms | 24.72 MiB | none | 4.75M/s |
| q17 | group by two, very high card | 32.000ms | 35.373ms | 33.648ms | 0.0% | 33.648ms | 33.648ms | 33.648ms | 33.648ms | 20.000ms | 33.96 MiB | none | 2.97M/s |
| q18 | group by two, no ordering | 25.000ms | 26.279ms | 26.080ms | 0.0% | 26.080ms | 26.080ms | 26.080ms | 26.080ms | 10.000ms | 33.79 MiB | none | 3.83M/s |
| q20 | point lookup | 2.000ms | 3.229ms | 3.038ms | 0.0% | 3.038ms | 3.038ms | 3.038ms | 3.038ms | 0.000us | 7.76 MiB | none | 32.92M/s |
| q21 | substring scan | 35.000ms | 36.016ms | 36.669ms | 0.0% | 36.669ms | 36.669ms | 36.669ms | 36.669ms | 30.000ms | 24.95 MiB | none | 2.73M/s |
| q22 | substring scan and group by | 39.000ms | 39.252ms | 41.068ms | 0.0% | 41.068ms | 41.068ms | 41.068ms | 41.068ms | 30.000ms | 26.97 MiB | none | 2.43M/s |
| q23 | two substring scans and group by | 82.000ms | 85.877ms | 83.788ms | 0.0% | 83.788ms | 83.788ms | 83.788ms | 83.788ms | 70.000ms | 49.86 MiB | none | 1.19M/s |
| q24 | select star and top k | 189.000ms | 194.487ms | 194.506ms | 0.0% | 194.506ms | 194.506ms | 194.506ms | 194.506ms | 190.000ms | 158.63 MiB | none | 514.11K/s |
| q25 | top k by a date | 13.000ms | 14.416ms | 13.936ms | 0.0% | 13.936ms | 13.936ms | 13.936ms | 13.936ms | 10.000ms | 8.28 MiB | none | 7.18M/s |
| q26 | top k by a string | 11.000ms | 12.473ms | 11.833ms | 0.0% | 11.833ms | 11.833ms | 11.833ms | 11.833ms | 10.000ms | 7.32 MiB | none | 8.45M/s |
| q27 | top k by two columns | 14.000ms | 15.057ms | 14.989ms | 0.0% | 14.989ms | 14.989ms | 14.989ms | 14.989ms | 10.000ms | 8.16 MiB | none | 6.67M/s |
| q28 | group by with a string length | 34.000ms | 34.878ms | 35.964ms | 0.0% | 35.964ms | 35.964ms | 35.964ms | 35.964ms | 30.000ms | 26.16 MiB | none | 2.78M/s |
| q29 | group by a regular expression | 83.000ms | 84.947ms | 85.044ms | 0.0% | 85.044ms | 85.044ms | 85.044ms | 85.044ms | 70.000ms | 20.50 MiB | none | 1.18M/s |
| q30 | ninety sums over one column | 14.000ms | 15.225ms | 14.914ms | 0.0% | 14.914ms | 14.914ms | 14.914ms | 14.914ms | 10.000ms | 5.54 MiB | none | 6.70M/s |
| q31 | group by two and several aggregates | 15.000ms | 17.149ms | 16.587ms | 0.0% | 16.587ms | 16.587ms | 16.587ms | 16.587ms | 10.000ms | 15.34 MiB | none | 6.03M/s |
| q32 | group by a high card pair | 15.000ms | 16.392ms | 16.723ms | 0.0% | 16.723ms | 16.723ms | 16.723ms | 16.723ms | 10.000ms | 16.05 MiB | none | 5.98M/s |
| q34 | group by a long string | 57.000ms | 60.747ms | 59.635ms | 0.0% | 59.635ms | 59.635ms | 59.635ms | 59.635ms | 40.000ms | 39.02 MiB | none | 1.68M/s |
| q35 | group by a constant and a long string | 65.000ms | 67.134ms | 67.607ms | 0.0% | 67.607ms | 67.607ms | 67.607ms | 67.607ms | 60.000ms | 49.78 MiB | none | 1.48M/s |
| q36 | group by four expressions | 33.000ms | 34.097ms | 34.089ms | 0.0% | 34.089ms | 34.089ms | 34.089ms | 34.089ms | 20.000ms | 40.47 MiB | none | 2.93M/s |
| q37 | date range and group by a URL | 32.000ms | 34.739ms | 33.495ms | 0.0% | 33.495ms | 33.495ms | 33.495ms | 33.495ms | 20.000ms | 28.39 MiB | none | 2.99M/s |
| q38 | date range and group by a title | 37.000ms | 38.961ms | 38.973ms | 0.0% | 38.973ms | 38.973ms | 38.973ms | 38.973ms | 20.000ms | 29.11 MiB | none | 2.57M/s |
| q39 | date range, group by and offset | 33.000ms | 34.380ms | 35.019ms | 0.0% | 35.019ms | 35.019ms | 35.019ms | 35.019ms | 30.000ms | 29.21 MiB | none | 2.86M/s |
| q40 | date range, a case and a wide group by | 61.000ms | 60.066ms | 62.629ms | 0.0% | 62.629ms | 62.629ms | 62.629ms | 62.629ms | 50.000ms | 46.29 MiB | none | 1.60M/s |
| q41 | date range with an IN and a hash | 7.000ms | 7.945ms | 7.832ms | 0.0% | 7.832ms | 7.832ms | 7.832ms | 7.832ms | 0.000us | 10.84 MiB | none | 12.77M/s |
| q42 | date range and a deep offset | 7.000ms | 7.640ms | 8.087ms | 0.0% | 8.087ms | 8.087ms | 8.087ms | 8.087ms | 0.000us | 10.73 MiB | none | 12.37M/s |
| q43 | minute buckets over a date range | 6.000ms | 7.032ms | 7.125ms | 0.0% | 7.125ms | 7.125ms | 7.125ms | 7.125ms | 0.000us | 9.23 MiB | none | 14.03M/s |

rudb rudb 0.2.31 over 41 of 43 queries. Total 1.074s by its own clock and 1.136s by ours, 1.135s cold, 810.000ms of CPU, peak 158.63 MiB, 3.82M/s and 576.98 MiB/s.

Running it cost 6% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 138.24x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 99998 rows, one out of every 1000 of the 99997497 in the full file, which is a development loop rather than the suite
- q1 has 1 hot runs and rule two wants at least five
- q2 has 1 hot runs and rule two wants at least five
- q3 has 1 hot runs and rule two wants at least five
- q4 has 1 hot runs and rule two wants at least five
- q5 has 1 hot runs and rule two wants at least five
- q6 has 1 hot runs and rule two wants at least five
- q7 has 1 hot runs and rule two wants at least five
- q8 has 1 hot runs and rule two wants at least five
- q9 has 1 hot runs and rule two wants at least five
- q10 has 1 hot runs and rule two wants at least five
- q11 has 1 hot runs and rule two wants at least five
- q12 has 1 hot runs and rule two wants at least five
- q13 has 1 hot runs and rule two wants at least five
- q14 has 1 hot runs and rule two wants at least five
- q15 has 1 hot runs and rule two wants at least five
- q16 has 1 hot runs and rule two wants at least five
- q17 has 1 hot runs and rule two wants at least five
- q18 has 1 hot runs and rule two wants at least five
- q19 has 1 hot runs and rule two wants at least five
- q20 has 1 hot runs and rule two wants at least five
- q21 has 1 hot runs and rule two wants at least five
- q22 has 1 hot runs and rule two wants at least five
- q23 has 1 hot runs and rule two wants at least five
- q24 has 1 hot runs and rule two wants at least five
- q25 has 1 hot runs and rule two wants at least five
- q26 has 1 hot runs and rule two wants at least five
- q27 has 1 hot runs and rule two wants at least five
- q28 has 1 hot runs and rule two wants at least five
- q29 has 1 hot runs and rule two wants at least five
- q30 has 1 hot runs and rule two wants at least five
- q31 has 1 hot runs and rule two wants at least five
- q32 has 1 hot runs and rule two wants at least five
- q33 has 1 hot runs and rule two wants at least five
- q34 has 1 hot runs and rule two wants at least five
- q35 has 1 hot runs and rule two wants at least five
- q36 has 1 hot runs and rule two wants at least five
- q37 has 1 hot runs and rule two wants at least five
- q38 has 1 hot runs and rule two wants at least five
- q39 has 1 hot runs and rule two wants at least five
- q40 has 1 hot runs and rule two wants at least five
- q41 has 1 hot runs and rule two wants at least five
- q42 has 1 hot runs and rule two wants at least five
- q43 has 1 hot runs and rule two wants at least five

These ran every query at about the same speed:

- polars ran every query within 1.79x of every other one, and this suite spreads over 6x on an engine it is measuring
- clickhouse-server ran every query within 1.83x of every other one, and this suite spreads over 6x on an engine it is measuring

A column that flat is not a column about the queries. Read those numbers as an upper bound on the engine and not as a measurement of it.

polars did not run q28, because the SQLContext has no strlen and no character length function.

polars did not run q29, because the SQLContext has no regexp_replace.

polars did not run q36, because the SQLContext rejects the repeated ClientIP output name in the group by.

polars did not run q43, because the SQLContext has no date_trunc.

So the polars column is 39 of 43 queries and its ratio is over the shared ones.

rudb did not run q19, because rudb's hash aggregate does not spill and this groups by a key that is close to unique over a hundred million rows, so the table outgrows the machine. tamnd/rudb#220, milestone E5.

rudb did not run q33, because rudb's hash aggregate does not spill and this groups by a key that is close to unique over a hundred million rows, so the table outgrows the machine. tamnd/rudb#220, milestone E5.

So the rudb column is 41 of 43 queries and its ratio is over the shared ones.

Answers differ, so this is not a comparison: q23: datafusion does not agree with duckdb: 14 numbers against 14

These were answered differently and the data does not say which is right:

- q16: ORDER BY COUNT(*) DESC LIMIT 10 over user IDs, where the tenth place is a tie between many users with the same count and the engine picks.
- q17: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q18: LIMIT 10 with no ORDER BY at all, so the ten rows are whichever ten the group by handed back first and the query did not ask for any particular ten.
- q19: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
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

Hot here means page cache warm and not buffer pool warm for every row but clickhouse-server, which stayed up across the whole suite with its own caches warm. That is the stronger kind of hot, so a ratio against that column is a ratio between two different quantities.

