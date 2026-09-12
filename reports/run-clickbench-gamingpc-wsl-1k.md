# clickbench on gamingpc-wsl

This is one run of the clickbench suite on gamingpc-wsl, over 7 engines and 43 queries, with 1 hot run of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

It ran over 1000 rows, one out of every 99998 of the 99997497 in the full file, which is a development loop and not the suite. Nothing here is comparable to a full run or to anybody else's number.

## What ran

| what | it was |
| --- | --- |
| suite | clickbench |
| queries | 43 |
| tables | 225.88 KiB of Parquet in 1 table |
| rows | 1000 in the table every query reads |
| sample | 1000 rows, one out of every 99998 of the 99997497 in the full file |
| summary | median with the interquartile range, per reporting rule two |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --rows 1000 --runs 1 --report
```

The data goes under `RUDB_BENCH_DATA`, which defaults to `~/rudb-data`, and the engines have to be on the path. An engine that is not gets a row in the abstentions below rather than being left out of the table.

## The machine

| fact | value | source |
| --- | --- | --- |

A fact that says it was not read is a fact about this machine and not a gap in the report. A field that had silently defaulted would be a lie that survived into it.

## The engines

| engine | version | state | load | load cpu | on disk | that size is | format | machine load |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 58.086ms | 40.000ms | 1.01 MiB | its own database file | its own | 4.08 to 4.08 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 67.448ms | 50.000ms | 1.01 MiB | its own database file | its own | 4.08 to 4.08 |
| clickhouse-local | 26.9.1.1162 | ran | 153.684ms | 110.000ms | 362.94 KiB | its own MergeTree parts, as system.parts counts the active ones | its own | 3.76 to 3.62 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 225.88 KiB | the source Parquet, this engine has no storage format of its own | the Parquet | 4.08 to 3.91 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 225.88 KiB | the source Parquet, this engine has no storage format of its own | the Parquet | 3.91 to 3.76 |
| clickhouse-server | 26.9.1.1162 | ran | 171.417ms | not read | 356.92 KiB | its own MergeTree parts, sorted by hits by (CounterID, EventDate, UserID, EventTime, WatchID), as system.parts counts the active ones | its own | 3.62 to 3.49 |
| rudb | rudb 0.2.31 | ran | 0.000us | 0.000us | 225.88 KiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 4.08 to 4.08 |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 92.000ms | 470.340ms | +411% | 476.229ms | 240.000ms | 0.51 | 36.52 MiB | none | 467.39K/s | 103.10 MiB/s | 1.00x |
| duckdb-pinned | 128.000ms | 1.044s | +716% | 1.104s | 870.000ms | 0.83 | 52.95 MiB | none | 335.94K/s | 74.10 MiB/s | 1.39x |
| clickhouse-local | 150.000ms | 2.248s | +1398% | 2.254s | 2.820s | 1.25 | 208.24 MiB | none | 286.67K/s | 63.23 MiB/s | 1.63x |
| datafusion | 213.000ms | 686.074ms | +222% | 727.320ms | 700.000ms | 1.02 | 183.72 MiB | none | 201.88K/s | 44.53 MiB/s | 2.32x |
| polars | 577.483ms | 3.935s | +581% | 4.032s | 6.720s | 1.71 | 83.65 MiB | none | 67.53K/s | 14.90 MiB/s | 6.28x |
| clickhouse-server | 162.000ms | 1.444s | +791% | 1.367s | not read | not read | not read | not read | 265.43K/s | 58.55 MiB/s | 1.76x |
| rudb | 20.000ms | 69.900ms | +250% | 70.860ms | 0.000us | 0.00 | 6.08 MiB | none | 2.05M/s | 452.20 MiB/s | 0.22x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | clickhouse-server | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 1.000ms | 2.000ms | 1.000ms | 5.776ms | 1.000ms | 0.000us |
| q2 | filtered count | 1.000ms | 1.000ms | 2.000ms | 3.000ms | 6.613ms | 1.000ms | 0.000us |
| q3 | three aggregates | 1.000ms | 1.000ms | 2.000ms | 1.000ms | 6.154ms | 2.000ms | 0.000us |
| q4 | average | 0.000us | 1.000ms | 3.000ms | 1.000ms | 9.621ms | 1.000ms | 0.000us |
| q5 | count distinct, high card | 2.000ms | 2.000ms | 2.000ms | 3.000ms | 10.602ms | 1.000ms | 0.000us |
| q6 | count distinct, strings | 2.000ms | 2.000ms | 2.000ms | 3.000ms | 14.139ms | 1.000ms | 0.000us |
| q7 | min and max of a date | 1.000ms | 1.000ms | 3.000ms | 1.000ms | 5.029ms | 1.000ms | 0.000us |
| q8 | group by, low card | 1.000ms | 6.000ms | 4.000ms | 5.000ms | 13.974ms | 2.000ms | 0.000us |
| q9 | group by and count distinct | 3.000ms | 3.000ms | 3.000ms | 6.000ms | 25.233ms | 2.000ms | 0.000us |
| q10 | group by, several aggregates | 4.000ms | 5.000ms | 3.000ms | 5.000ms | 25.170ms | 16.000ms | 1.000ms |
| q11 | group by a string and count distinct | 3.000ms | 4.000ms | 3.000ms | 9.000ms | 20.604ms | 2.000ms | 0.000us |
| q12 | group by two strings and count distinct | 3.000ms | 4.000ms | 3.000ms | 8.000ms | 19.491ms | 2.000ms | 0.000us |
| q13 | group by a string and top k | 2.000ms | 2.000ms | 3.000ms | 6.000ms | 14.629ms | 2.000ms | 0.000us |
| q14 | group by a string and count distinct | 4.000ms | 4.000ms | 3.000ms | 9.000ms | 20.572ms | 2.000ms | 0.000us |
| q15 | group by two columns and top k | 2.000ms | 3.000ms | 3.000ms | 6.000ms | 15.982ms | 2.000ms | 0.000us |
| q16 | group by, very high card | 2.000ms | 2.000ms | 3.000ms | 3.000ms | 18.765ms | 2.000ms | 1.000ms |
| q17 | group by two, very high card | 3.000ms | 3.000ms | 3.000ms | 4.000ms | 20.986ms | 2.000ms | 1.000ms |
| q18 | group by two, no ordering | 2.000ms | 2.000ms | 3.000ms | 4.000ms | 11.913ms | 2.000ms | 1.000ms |
| q19 | group by with an extract | 3.000ms | 3.000ms | 3.000ms | 5.000ms | 23.628ms | 2.000ms | no dialect |
| q20 | point lookup | 0.000us | 1.000ms | 3.000ms | 2.000ms | 5.335ms | 1.000ms | 0.000us |
| q21 | substring scan | 1.000ms | 1.000ms | 3.000ms | 3.000ms | 6.250ms | 1.000ms | 1.000ms |
| q22 | substring scan and group by | 1.000ms | 2.000ms | 3.000ms | 5.000ms | 15.037ms | 3.000ms | 1.000ms |
| q23 | two substring scans and group by | 1.000ms | 3.000ms | 3.000ms | 6.000ms | 18.848ms | 29.000ms | 1.000ms |
| q24 | select star and top k | 4.000ms | 9.000ms | 6.000ms | 8.000ms | 10.126ms | 4.000ms | 3.000ms |
| q25 | top k by a date | 2.000ms | 2.000ms | 4.000ms | 4.000ms | 13.025ms | 2.000ms | 0.000us |
| q26 | top k by a string | 1.000ms | 1.000ms | 3.000ms | 3.000ms | 12.538ms | 1.000ms | 0.000us |
| q27 | top k by two columns | 1.000ms | 1.000ms | 4.000ms | 3.000ms | 12.996ms | 1.000ms | 0.000us |
| q28 | group by with a string length | 2.000ms | 2.000ms | 3.000ms | 7.000ms | no dialect | 2.000ms | 1.000ms |
| q29 | group by a regular expression | 3.000ms | 4.000ms | 4.000ms | 7.000ms | no dialect | 21.000ms | 1.000ms |
| q30 | ninety sums over one column | 4.000ms | 14.000ms | 7.000ms | 10.000ms | 11.375ms | 6.000ms | 1.000ms |
| q31 | group by two and several aggregates | 3.000ms | 3.000ms | 3.000ms | 6.000ms | 17.446ms | 2.000ms | 0.000us |
| q32 | group by a high card pair | 2.000ms | 3.000ms | 3.000ms | 8.000ms | 18.270ms | 15.000ms | 0.000us |
| q33 | group by a high card pair, unfiltered | 3.000ms | 3.000ms | 3.000ms | 4.000ms | 18.806ms | 2.000ms | no dialect |
| q34 | group by a long string | 2.000ms | 2.000ms | 3.000ms | 4.000ms | 14.818ms | 2.000ms | 1.000ms |
| q35 | group by a constant and a long string | 2.000ms | 2.000ms | 3.000ms | 4.000ms | 18.732ms | 2.000ms | 1.000ms |
| q36 | group by four expressions | 3.000ms | 3.000ms | 3.000ms | 4.000ms | no dialect | 2.000ms | 1.000ms |
| q37 | date range and group by a URL | 3.000ms | 3.000ms | 5.000ms | 6.000ms | 15.498ms | 3.000ms | 1.000ms |
| q38 | date range and group by a title | 2.000ms | 2.000ms | 5.000ms | 6.000ms | 17.151ms | 3.000ms | 1.000ms |
| q39 | date range, group by and offset | 2.000ms | 1.000ms | 5.000ms | 6.000ms | 14.944ms | 3.000ms | 1.000ms |
| q40 | date range, a case and a wide group by | 2.000ms | 3.000ms | 5.000ms | 6.000ms | 17.680ms | 3.000ms | 1.000ms |
| q41 | date range with an IN and a hash | 3.000ms | 3.000ms | 6.000ms | 7.000ms | 14.727ms | 3.000ms | 0.000us |
| q42 | date range and a deep offset | 3.000ms | 7.000ms | 5.000ms | 6.000ms | 15.000ms | 3.000ms | 0.000us |
| q43 | minute buckets over a date range | 2.000ms | 3.000ms | 5.000ms | 5.000ms | no dialect | 2.000ms | 0.000us |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 9.020ms | 9.057ms | 0.0% | 9.057ms | 9.057ms | 9.057ms | 9.057ms | 0.000us | 26.89 MiB | none | 110.41K/s |
| q2 | filtered count | 1.000ms | 9.645ms | 9.523ms | 0.0% | 9.523ms | 9.523ms | 9.523ms | 9.523ms | 0.000us | 28.04 MiB | none | 105.01K/s |
| q3 | three aggregates | 1.000ms | 9.752ms | 9.408ms | 0.0% | 9.408ms | 9.408ms | 9.408ms | 9.408ms | 0.000us | 28.08 MiB | 4.00 KiB | 106.29K/s |
| q4 | average | 0.000us | 9.925ms | 9.294ms | 0.0% | 9.294ms | 9.294ms | 9.294ms | 9.294ms | 0.000us | 27.74 MiB | none | 107.60K/s |
| q5 | count distinct, high card | 2.000ms | 11.551ms | 10.960ms | 0.0% | 10.960ms | 10.960ms | 10.960ms | 10.960ms | 0.000us | 29.23 MiB | none | 91.24K/s |
| q6 | count distinct, strings | 2.000ms | 10.762ms | 10.863ms | 0.0% | 10.863ms | 10.863ms | 10.863ms | 10.863ms | 10.000ms | 29.83 MiB | none | 92.06K/s |
| q7 | min and max of a date | 1.000ms | 8.983ms | 9.134ms | 0.0% | 9.134ms | 9.134ms | 9.134ms | 9.134ms | 0.000us | 27.23 MiB | none | 109.48K/s |
| q8 | group by, low card | 1.000ms | 10.267ms | 9.527ms | 0.0% | 9.527ms | 9.527ms | 9.527ms | 9.527ms | 10.000ms | 29.44 MiB | none | 104.96K/s |
| q9 | group by and count distinct | 3.000ms | 11.796ms | 11.706ms | 0.0% | 11.706ms | 11.706ms | 11.706ms | 11.706ms | 0.000us | 34.75 MiB | none | 85.43K/s |
| q10 | group by, several aggregates | 4.000ms | 12.420ms | 12.528ms | 0.0% | 12.528ms | 12.528ms | 12.528ms | 12.528ms | 10.000ms | 36.52 MiB | none | 79.82K/s |
| q11 | group by a string and count distinct | 3.000ms | 12.074ms | 11.842ms | 0.0% | 11.842ms | 11.842ms | 11.842ms | 11.842ms | 10.000ms | 33.00 MiB | none | 84.45K/s |
| q12 | group by two strings and count distinct | 3.000ms | 12.157ms | 11.967ms | 0.0% | 11.967ms | 11.967ms | 11.967ms | 11.967ms | 10.000ms | 36.00 MiB | none | 83.56K/s |
| q13 | group by a string and top k | 2.000ms | 10.740ms | 11.165ms | 0.0% | 11.165ms | 11.165ms | 11.165ms | 11.165ms | 0.000us | 30.75 MiB | none | 89.57K/s |
| q14 | group by a string and count distinct | 4.000ms | 12.422ms | 12.381ms | 0.0% | 12.381ms | 12.381ms | 12.381ms | 12.381ms | 10.000ms | 36.00 MiB | none | 80.77K/s |
| q15 | group by two columns and top k | 2.000ms | 11.114ms | 11.100ms | 0.0% | 11.100ms | 11.100ms | 11.100ms | 11.100ms | 10.000ms | 31.75 MiB | none | 90.09K/s |
| q16 | group by, very high card | 2.000ms | 11.614ms | 10.842ms | 0.0% | 10.842ms | 10.842ms | 10.842ms | 10.842ms | 10.000ms | 32.34 MiB | none | 92.23K/s |
| q17 | group by two, very high card | 3.000ms | 12.236ms | 11.774ms | 0.0% | 11.774ms | 11.774ms | 11.774ms | 11.774ms | 0.000us | 33.04 MiB | none | 84.93K/s |
| q18 | group by two, no ordering | 2.000ms | 11.610ms | 11.439ms | 0.0% | 11.439ms | 11.439ms | 11.439ms | 11.439ms | 10.000ms | 35.07 MiB | none | 87.42K/s |
| q19 | group by with an extract | 3.000ms | 11.415ms | 11.846ms | 0.0% | 11.846ms | 11.846ms | 11.846ms | 11.846ms | 10.000ms | 34.05 MiB | none | 84.42K/s |
| q20 | point lookup | 0.000us | 9.247ms | 9.270ms | 0.0% | 9.270ms | 9.270ms | 9.270ms | 9.270ms | 0.000us | 27.33 MiB | none | 107.87K/s |
| q21 | substring scan | 1.000ms | 10.179ms | 9.668ms | 0.0% | 9.668ms | 9.668ms | 9.668ms | 9.668ms | 10.000ms | 27.99 MiB | none | 103.43K/s |
| q22 | substring scan and group by | 1.000ms | 10.344ms | 9.801ms | 0.0% | 9.801ms | 9.801ms | 9.801ms | 9.801ms | 10.000ms | 28.75 MiB | none | 102.03K/s |
| q23 | two substring scans and group by | 1.000ms | 10.742ms | 10.063ms | 0.0% | 10.063ms | 10.063ms | 10.063ms | 10.063ms | 0.000us | 28.94 MiB | none | 99.37K/s |
| q24 | select star and top k | 4.000ms | 13.421ms | 13.175ms | 0.0% | 13.175ms | 13.175ms | 13.175ms | 13.175ms | 10.000ms | 33.57 MiB | none | 75.90K/s |
| q25 | top k by a date | 2.000ms | 10.859ms | 10.695ms | 0.0% | 10.695ms | 10.695ms | 10.695ms | 10.695ms | 10.000ms | 30.48 MiB | none | 93.50K/s |
| q26 | top k by a string | 1.000ms | 9.451ms | 9.550ms | 0.0% | 9.550ms | 9.550ms | 9.550ms | 9.550ms | 10.000ms | 27.73 MiB | none | 104.71K/s |
| q27 | top k by two columns | 1.000ms | 9.534ms | 9.907ms | 0.0% | 9.907ms | 9.907ms | 9.907ms | 9.907ms | 0.000us | 28.23 MiB | none | 100.94K/s |
| q28 | group by with a string length | 2.000ms | 11.561ms | 11.049ms | 0.0% | 11.049ms | 11.049ms | 11.049ms | 11.049ms | 10.000ms | 30.00 MiB | none | 90.51K/s |
| q29 | group by a regular expression | 3.000ms | 11.425ms | 11.628ms | 0.0% | 11.628ms | 11.628ms | 11.628ms | 11.628ms | 0.000us | 31.21 MiB | none | 86.00K/s |
| q30 | ninety sums over one column | 4.000ms | 12.629ms | 12.408ms | 0.0% | 12.408ms | 12.408ms | 12.408ms | 12.408ms | 0.000us | 31.42 MiB | none | 80.59K/s |
| q31 | group by two and several aggregates | 3.000ms | 11.023ms | 11.402ms | 0.0% | 11.402ms | 11.402ms | 11.402ms | 11.402ms | 0.000us | 33.34 MiB | none | 87.70K/s |
| q32 | group by a high card pair | 2.000ms | 11.373ms | 11.125ms | 0.0% | 11.125ms | 11.125ms | 11.125ms | 11.125ms | 10.000ms | 33.30 MiB | none | 89.89K/s |
| q33 | group by a high card pair, unfiltered | 3.000ms | 11.225ms | 11.251ms | 0.0% | 11.251ms | 11.251ms | 11.251ms | 11.251ms | 10.000ms | 33.50 MiB | none | 88.88K/s |
| q34 | group by a long string | 2.000ms | 11.622ms | 11.144ms | 0.0% | 11.144ms | 11.144ms | 11.144ms | 11.144ms | 0.000us | 31.00 MiB | none | 89.73K/s |
| q35 | group by a constant and a long string | 2.000ms | 11.476ms | 11.295ms | 0.0% | 11.295ms | 11.295ms | 11.295ms | 11.295ms | 10.000ms | 31.33 MiB | none | 88.53K/s |
| q36 | group by four expressions | 3.000ms | 11.273ms | 11.363ms | 0.0% | 11.363ms | 11.363ms | 11.363ms | 11.363ms | 10.000ms | 33.07 MiB | none | 88.00K/s |
| q37 | date range and group by a URL | 3.000ms | 10.842ms | 11.140ms | 0.0% | 11.140ms | 11.140ms | 11.140ms | 11.140ms | 0.000us | 31.99 MiB | none | 89.77K/s |
| q38 | date range and group by a title | 2.000ms | 10.974ms | 11.003ms | 0.0% | 11.003ms | 11.003ms | 11.003ms | 11.003ms | 10.000ms | 30.75 MiB | none | 90.88K/s |
| q39 | date range, group by and offset | 2.000ms | 9.842ms | 9.869ms | 0.0% | 9.869ms | 9.869ms | 9.869ms | 9.869ms | 10.000ms | 28.75 MiB | none | 101.33K/s |
| q40 | date range, a case and a wide group by | 2.000ms | 11.300ms | 11.572ms | 0.0% | 11.572ms | 11.572ms | 11.572ms | 11.572ms | 10.000ms | 32.00 MiB | none | 86.42K/s |
| q41 | date range with an IN and a hash | 3.000ms | 12.404ms | 13.458ms | 0.0% | 13.458ms | 13.458ms | 13.458ms | 13.458ms | 10.000ms | 32.07 MiB | none | 74.31K/s |
| q42 | date range and a deep offset | 3.000ms | 13.207ms | 11.016ms | 0.0% | 11.016ms | 11.016ms | 11.016ms | 11.016ms | 0.000us | 30.50 MiB | none | 90.78K/s |
| q43 | minute buckets over a date range | 2.000ms | 10.773ms | 11.132ms | 0.0% | 11.132ms | 11.132ms | 11.132ms | 11.132ms | 0.000us | 30.67 MiB | none | 89.83K/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 92.000ms by its own clock and 470.340ms by ours, 476.229ms cold, 240.000ms of CPU, peak 36.52 MiB, 467.39K/s and 103.10 MiB/s.

Running it cost 411% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.49x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 23.047ms | 21.362ms | 0.0% | 21.362ms | 21.362ms | 21.362ms | 21.362ms | 10.000ms | 39.82 MiB | none | 46.81K/s |
| q2 | filtered count | 1.000ms | 22.281ms | 21.799ms | 0.0% | 21.799ms | 21.799ms | 21.799ms | 21.799ms | 20.000ms | 39.80 MiB | none | 45.87K/s |
| q3 | three aggregates | 1.000ms | 21.841ms | 21.741ms | 0.0% | 21.741ms | 21.741ms | 21.741ms | 21.741ms | 20.000ms | 40.21 MiB | none | 46.00K/s |
| q4 | average | 1.000ms | 22.166ms | 23.145ms | 0.0% | 23.145ms | 23.145ms | 23.145ms | 23.145ms | 10.000ms | 40.56 MiB | none | 43.21K/s |
| q5 | count distinct, high card | 2.000ms | 22.637ms | 22.869ms | 0.0% | 22.869ms | 22.869ms | 22.869ms | 22.869ms | 20.000ms | 42.64 MiB | 32.00 KiB | 43.73K/s |
| q6 | count distinct, strings | 2.000ms | 23.001ms | 22.677ms | 0.0% | 22.677ms | 22.677ms | 22.677ms | 22.677ms | 10.000ms | 41.34 MiB | none | 44.10K/s |
| q7 | min and max of a date | 1.000ms | 22.035ms | 25.548ms | 0.0% | 25.548ms | 25.548ms | 25.548ms | 25.548ms | 90.000ms | 40.14 MiB | none | 39.14K/s |
| q8 | group by, low card | 6.000ms | 26.752ms | 26.735ms | 0.0% | 26.735ms | 26.735ms | 26.735ms | 26.735ms | 30.000ms | 42.84 MiB | none | 37.40K/s |
| q9 | group by and count distinct | 3.000ms | 25.135ms | 24.684ms | 0.0% | 24.684ms | 24.684ms | 24.684ms | 24.684ms | 20.000ms | 46.60 MiB | none | 40.51K/s |
| q10 | group by, several aggregates | 5.000ms | 26.281ms | 25.830ms | 0.0% | 25.830ms | 25.830ms | 25.830ms | 25.830ms | 20.000ms | 49.18 MiB | 72.00 KiB | 38.71K/s |
| q11 | group by a string and count distinct | 4.000ms | 25.165ms | 25.140ms | 0.0% | 25.140ms | 25.140ms | 25.140ms | 25.140ms | 20.000ms | 46.12 MiB | none | 39.78K/s |
| q12 | group by two strings and count distinct | 4.000ms | 23.827ms | 24.110ms | 0.0% | 24.110ms | 24.110ms | 24.110ms | 24.110ms | 20.000ms | 45.62 MiB | none | 41.48K/s |
| q13 | group by a string and top k | 2.000ms | 23.615ms | 23.454ms | 0.0% | 23.454ms | 23.454ms | 23.454ms | 23.454ms | 10.000ms | 42.58 MiB | none | 42.64K/s |
| q14 | group by a string and count distinct | 4.000ms | 25.196ms | 24.419ms | 0.0% | 24.419ms | 24.419ms | 24.419ms | 24.419ms | 20.000ms | 49.42 MiB | none | 40.95K/s |
| q15 | group by two columns and top k | 3.000ms | 23.051ms | 23.979ms | 0.0% | 23.979ms | 23.979ms | 23.979ms | 23.979ms | 20.000ms | 43.01 MiB | none | 41.70K/s |
| q16 | group by, very high card | 2.000ms | 23.554ms | 24.213ms | 0.0% | 24.213ms | 24.213ms | 24.213ms | 24.213ms | 20.000ms | 43.41 MiB | none | 41.30K/s |
| q17 | group by two, very high card | 3.000ms | 29.422ms | 23.933ms | 0.0% | 23.933ms | 23.933ms | 23.933ms | 23.933ms | 20.000ms | 44.06 MiB | none | 41.78K/s |
| q18 | group by two, no ordering | 2.000ms | 23.223ms | 23.731ms | 0.0% | 23.731ms | 23.731ms | 23.731ms | 23.731ms | 20.000ms | 44.19 MiB | none | 42.14K/s |
| q19 | group by with an extract | 3.000ms | 23.734ms | 24.570ms | 0.0% | 24.570ms | 24.570ms | 24.570ms | 24.570ms | 20.000ms | 44.73 MiB | none | 40.70K/s |
| q20 | point lookup | 1.000ms | 23.418ms | 22.693ms | 0.0% | 22.693ms | 22.693ms | 22.693ms | 22.693ms | 10.000ms | 39.34 MiB | none | 44.07K/s |
| q21 | substring scan | 1.000ms | 22.259ms | 22.333ms | 0.0% | 22.333ms | 22.333ms | 22.333ms | 22.333ms | 10.000ms | 40.33 MiB | none | 44.78K/s |
| q22 | substring scan and group by | 2.000ms | 23.198ms | 22.526ms | 0.0% | 22.526ms | 22.526ms | 22.526ms | 22.526ms | 20.000ms | 41.02 MiB | none | 44.39K/s |
| q23 | two substring scans and group by | 3.000ms | 74.089ms | 26.028ms | 0.0% | 26.028ms | 26.028ms | 26.028ms | 26.028ms | 20.000ms | 43.07 MiB | none | 38.42K/s |
| q24 | select star and top k | 9.000ms | 31.373ms | 30.569ms | 0.0% | 30.569ms | 30.569ms | 30.569ms | 30.569ms | 30.000ms | 47.70 MiB | none | 32.71K/s |
| q25 | top k by a date | 2.000ms | 22.697ms | 22.448ms | 0.0% | 22.448ms | 22.448ms | 22.448ms | 22.448ms | 10.000ms | 40.32 MiB | none | 44.55K/s |
| q26 | top k by a string | 1.000ms | 23.668ms | 23.154ms | 0.0% | 23.154ms | 23.154ms | 23.154ms | 23.154ms | 20.000ms | 40.32 MiB | none | 43.19K/s |
| q27 | top k by two columns | 1.000ms | 22.498ms | 22.193ms | 0.0% | 22.193ms | 22.193ms | 22.193ms | 22.193ms | 10.000ms | 40.56 MiB | none | 45.06K/s |
| q28 | group by with a string length | 2.000ms | 23.808ms | 23.039ms | 0.0% | 23.039ms | 23.039ms | 23.039ms | 23.039ms | 20.000ms | 43.00 MiB | none | 43.40K/s |
| q29 | group by a regular expression | 4.000ms | 24.498ms | 24.628ms | 0.0% | 24.628ms | 24.628ms | 24.628ms | 24.628ms | 20.000ms | 43.09 MiB | none | 40.60K/s |
| q30 | ninety sums over one column | 14.000ms | 36.326ms | 36.276ms | 0.0% | 36.276ms | 36.276ms | 36.276ms | 36.276ms | 30.000ms | 52.95 MiB | none | 27.57K/s |
| q31 | group by two and several aggregates | 3.000ms | 24.218ms | 24.326ms | 0.0% | 24.326ms | 24.326ms | 24.326ms | 24.326ms | 20.000ms | 45.28 MiB | none | 41.11K/s |
| q32 | group by a high card pair | 3.000ms | 25.019ms | 23.954ms | 0.0% | 23.954ms | 23.954ms | 23.954ms | 23.954ms | 20.000ms | 45.14 MiB | none | 41.75K/s |
| q33 | group by a high card pair, unfiltered | 3.000ms | 24.252ms | 24.177ms | 0.0% | 24.177ms | 24.177ms | 24.177ms | 24.177ms | 20.000ms | 45.53 MiB | none | 41.36K/s |
| q34 | group by a long string | 2.000ms | 23.494ms | 24.644ms | 0.0% | 24.644ms | 24.644ms | 24.644ms | 24.644ms | 10.000ms | 42.64 MiB | none | 40.58K/s |
| q35 | group by a constant and a long string | 2.000ms | 23.382ms | 23.385ms | 0.0% | 23.385ms | 23.385ms | 23.385ms | 23.385ms | 20.000ms | 42.39 MiB | none | 42.76K/s |
| q36 | group by four expressions | 3.000ms | 27.501ms | 24.134ms | 0.0% | 24.134ms | 24.134ms | 24.134ms | 24.134ms | 20.000ms | 44.20 MiB | none | 41.44K/s |
| q37 | date range and group by a URL | 3.000ms | 23.486ms | 23.411ms | 0.0% | 23.411ms | 23.411ms | 23.411ms | 23.411ms | 20.000ms | 42.77 MiB | none | 42.71K/s |
| q38 | date range and group by a title | 2.000ms | 24.257ms | 23.931ms | 0.0% | 23.931ms | 23.931ms | 23.931ms | 23.931ms | 20.000ms | 42.53 MiB | none | 41.79K/s |
| q39 | date range, group by and offset | 1.000ms | 24.026ms | 22.844ms | 0.0% | 22.844ms | 22.844ms | 22.844ms | 22.844ms | 20.000ms | 41.02 MiB | none | 43.78K/s |
| q40 | date range, a case and a wide group by | 3.000ms | 24.866ms | 24.045ms | 0.0% | 24.045ms | 24.045ms | 24.045ms | 24.045ms | 20.000ms | 44.56 MiB | none | 41.59K/s |
| q41 | date range with an IN and a hash | 3.000ms | 23.263ms | 23.474ms | 0.0% | 23.474ms | 23.474ms | 23.474ms | 23.474ms | 10.000ms | 44.26 MiB | none | 42.60K/s |
| q42 | date range and a deep offset | 7.000ms | 28.836ms | 27.969ms | 0.0% | 27.969ms | 27.969ms | 27.969ms | 27.969ms | 30.000ms | 43.45 MiB | none | 35.75K/s |
| q43 | minute buckets over a date range | 3.000ms | 23.674ms | 23.942ms | 0.0% | 23.942ms | 23.942ms | 23.942ms | 23.942ms | 20.000ms | 42.06 MiB | none | 41.77K/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 128.000ms by its own clock and 1.044s by ours, 1.104s cold, 870.000ms of CPU, peak 52.95 MiB, 335.94K/s and 74.10 MiB/s.

Running it cost 716% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.70x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 2.000ms | 53.843ms | 53.978ms | 0.0% | 53.978ms | 53.978ms | 53.978ms | 53.978ms | 60.000ms | 199.88 MiB | none | 18.53K/s |
| q2 | filtered count | 2.000ms | 48.854ms | 66.132ms | 0.0% | 66.132ms | 66.132ms | 66.132ms | 66.132ms | 200.000ms | 200.39 MiB | none | 15.12K/s |
| q3 | three aggregates | 2.000ms | 47.393ms | 47.734ms | 0.0% | 47.734ms | 47.734ms | 47.734ms | 47.734ms | 50.000ms | 203.25 MiB | none | 20.95K/s |
| q4 | average | 3.000ms | 47.429ms | 51.769ms | 0.0% | 51.769ms | 51.769ms | 51.769ms | 51.769ms | 50.000ms | 202.50 MiB | none | 19.32K/s |
| q5 | count distinct, high card | 2.000ms | 47.994ms | 47.657ms | 0.0% | 47.657ms | 47.657ms | 47.657ms | 47.657ms | 50.000ms | 202.67 MiB | none | 20.98K/s |
| q6 | count distinct, strings | 2.000ms | 53.723ms | 55.415ms | 0.0% | 55.415ms | 55.415ms | 55.415ms | 55.415ms | 60.000ms | 202.23 MiB | none | 18.05K/s |
| q7 | min and max of a date | 3.000ms | 54.568ms | 54.107ms | 0.0% | 54.107ms | 54.107ms | 54.107ms | 54.107ms | 60.000ms | 202.29 MiB | none | 18.48K/s |
| q8 | group by, low card | 4.000ms | 51.346ms | 50.599ms | 0.0% | 50.599ms | 50.599ms | 50.599ms | 50.599ms | 50.000ms | 204.98 MiB | 1.09 MiB | 19.76K/s |
| q9 | group by and count distinct | 3.000ms | 50.565ms | 47.542ms | 0.0% | 47.542ms | 47.542ms | 47.542ms | 47.542ms | 50.000ms | 203.94 MiB | none | 21.03K/s |
| q10 | group by, several aggregates | 3.000ms | 51.123ms | 47.935ms | 0.0% | 47.935ms | 47.935ms | 47.935ms | 47.935ms | 40.000ms | 205.09 MiB | none | 20.86K/s |
| q11 | group by a string and count distinct | 3.000ms | 51.658ms | 52.153ms | 0.0% | 52.153ms | 52.153ms | 52.153ms | 52.153ms | 130.000ms | 205.36 MiB | none | 19.17K/s |
| q12 | group by two strings and count distinct | 3.000ms | 48.250ms | 48.076ms | 0.0% | 48.076ms | 48.076ms | 48.076ms | 48.076ms | 50.000ms | 205.94 MiB | none | 20.80K/s |
| q13 | group by a string and top k | 3.000ms | 47.949ms | 51.184ms | 0.0% | 51.184ms | 51.184ms | 51.184ms | 51.184ms | 60.000ms | 205.38 MiB | none | 19.54K/s |
| q14 | group by a string and count distinct | 3.000ms | 49.740ms | 54.582ms | 0.0% | 54.582ms | 54.582ms | 54.582ms | 54.582ms | 50.000ms | 205.89 MiB | none | 18.32K/s |
| q15 | group by two columns and top k | 3.000ms | 50.458ms | 52.115ms | 0.0% | 52.115ms | 52.115ms | 52.115ms | 52.115ms | 50.000ms | 205.43 MiB | none | 19.19K/s |
| q16 | group by, very high card | 3.000ms | 56.742ms | 57.713ms | 0.0% | 57.713ms | 57.713ms | 57.713ms | 57.713ms | 70.000ms | 204.44 MiB | none | 17.33K/s |
| q17 | group by two, very high card | 3.000ms | 56.020ms | 54.404ms | 0.0% | 54.404ms | 54.404ms | 54.404ms | 54.404ms | 50.000ms | 204.20 MiB | none | 18.38K/s |
| q18 | group by two, no ordering | 3.000ms | 53.418ms | 54.138ms | 0.0% | 54.138ms | 54.138ms | 54.138ms | 54.138ms | 50.000ms | 205.12 MiB | none | 18.47K/s |
| q19 | group by with an extract | 3.000ms | 53.008ms | 48.301ms | 0.0% | 48.301ms | 48.301ms | 48.301ms | 48.301ms | 50.000ms | 204.99 MiB | none | 20.70K/s |
| q20 | point lookup | 3.000ms | 50.705ms | 74.411ms | 0.0% | 74.411ms | 74.411ms | 74.411ms | 74.411ms | 340.000ms | 202.33 MiB | none | 13.44K/s |
| q21 | substring scan | 3.000ms | 52.821ms | 48.278ms | 0.0% | 48.278ms | 48.278ms | 48.278ms | 48.278ms | 50.000ms | 203.36 MiB | none | 20.71K/s |
| q22 | substring scan and group by | 3.000ms | 48.329ms | 54.472ms | 0.0% | 54.472ms | 54.472ms | 54.472ms | 54.472ms | 60.000ms | 204.59 MiB | none | 18.36K/s |
| q23 | two substring scans and group by | 3.000ms | 51.556ms | 51.206ms | 0.0% | 51.206ms | 51.206ms | 51.206ms | 51.206ms | 60.000ms | 204.93 MiB | none | 19.53K/s |
| q24 | select star and top k | 6.000ms | 96.144ms | 52.739ms | 0.0% | 52.739ms | 52.739ms | 52.739ms | 52.739ms | 50.000ms | 204.66 MiB | 4.00 KiB | 18.96K/s |
| q25 | top k by a date | 4.000ms | 49.657ms | 48.197ms | 0.0% | 48.197ms | 48.197ms | 48.197ms | 48.197ms | 50.000ms | 204.69 MiB | none | 20.75K/s |
| q26 | top k by a string | 3.000ms | 52.863ms | 53.858ms | 0.0% | 53.858ms | 53.858ms | 53.858ms | 53.858ms | 60.000ms | 203.60 MiB | none | 18.57K/s |
| q27 | top k by two columns | 4.000ms | 53.058ms | 54.295ms | 0.0% | 54.295ms | 54.295ms | 54.295ms | 54.295ms | 60.000ms | 203.91 MiB | none | 18.42K/s |
| q28 | group by with a string length | 3.000ms | 49.665ms | 49.237ms | 0.0% | 49.237ms | 49.237ms | 49.237ms | 49.237ms | 50.000ms | 205.12 MiB | none | 20.31K/s |
| q29 | group by a regular expression | 4.000ms | 53.359ms | 49.619ms | 0.0% | 49.619ms | 49.619ms | 49.619ms | 49.619ms | 40.000ms | 206.44 MiB | none | 20.15K/s |
| q30 | ninety sums over one column | 7.000ms | 53.387ms | 53.763ms | 0.0% | 53.763ms | 53.763ms | 53.763ms | 53.763ms | 60.000ms | 206.47 MiB | none | 18.60K/s |
| q31 | group by two and several aggregates | 3.000ms | 48.666ms | 50.853ms | 0.0% | 50.853ms | 50.853ms | 50.853ms | 50.853ms | 50.000ms | 205.44 MiB | none | 19.66K/s |
| q32 | group by a high card pair | 3.000ms | 53.902ms | 49.581ms | 0.0% | 49.581ms | 49.581ms | 49.581ms | 49.581ms | 50.000ms | 205.44 MiB | none | 20.17K/s |
| q33 | group by a high card pair, unfiltered | 3.000ms | 48.459ms | 49.684ms | 0.0% | 49.684ms | 49.684ms | 49.684ms | 49.684ms | 50.000ms | 204.72 MiB | none | 20.13K/s |
| q34 | group by a long string | 3.000ms | 47.254ms | 47.610ms | 0.0% | 47.610ms | 47.610ms | 47.610ms | 47.610ms | 50.000ms | 204.45 MiB | none | 21.00K/s |
| q35 | group by a constant and a long string | 3.000ms | 52.526ms | 55.524ms | 0.0% | 55.524ms | 55.524ms | 55.524ms | 55.524ms | 70.000ms | 205.19 MiB | none | 18.01K/s |
| q36 | group by four expressions | 3.000ms | 48.296ms | 47.749ms | 0.0% | 47.749ms | 47.749ms | 47.749ms | 47.749ms | 50.000ms | 204.89 MiB | none | 20.94K/s |
| q37 | date range and group by a URL | 5.000ms | 54.926ms | 49.185ms | 0.0% | 49.185ms | 49.185ms | 49.185ms | 49.185ms | 60.000ms | 207.45 MiB | none | 20.33K/s |
| q38 | date range and group by a title | 5.000ms | 50.036ms | 51.497ms | 0.0% | 51.497ms | 51.497ms | 51.497ms | 51.497ms | 50.000ms | 206.89 MiB | none | 19.42K/s |
| q39 | date range, group by and offset | 5.000ms | 49.364ms | 50.191ms | 0.0% | 50.191ms | 50.191ms | 50.191ms | 50.191ms | 60.000ms | 206.08 MiB | none | 19.92K/s |
| q40 | date range, a case and a wide group by | 5.000ms | 50.019ms | 51.039ms | 0.0% | 51.039ms | 51.039ms | 51.039ms | 51.039ms | 60.000ms | 207.70 MiB | none | 19.59K/s |
| q41 | date range with an IN and a hash | 6.000ms | 58.019ms | 57.329ms | 0.0% | 57.329ms | 57.329ms | 57.329ms | 57.329ms | 60.000ms | 208.24 MiB | 3.41 MiB | 17.44K/s |
| q42 | date range and a deep offset | 5.000ms | 50.871ms | 51.224ms | 0.0% | 51.224ms | 51.224ms | 51.224ms | 51.224ms | 50.000ms | 206.92 MiB | none | 19.52K/s |
| q43 | minute buckets over a date range | 5.000ms | 56.341ms | 50.495ms | 0.0% | 50.495ms | 50.495ms | 50.495ms | 50.495ms | 50.000ms | 207.24 MiB | none | 19.80K/s |

clickhouse-local 26.9.1.1162 over 43 of 43 queries. Total 150.000ms by its own clock and 2.248s by ours, 2.254s cold, 2.820s of CPU, peak 208.24 MiB, 286.67K/s and 63.23 MiB/s.

Running it cost 1398% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.57x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 47.098ms | 13.580ms | 0.0% | 13.580ms | 13.580ms | 13.580ms | 13.580ms | 0.000us | 80.29 MiB | 70.61 MiB | 73.64K/s |
| q2 | filtered count | 3.000ms | 16.112ms | 14.303ms | 0.0% | 14.303ms | 14.303ms | 14.303ms | 14.303ms | 10.000ms | 104.73 MiB | 32.00 KiB | 69.92K/s |
| q3 | three aggregates | 1.000ms | 12.343ms | 11.937ms | 0.0% | 11.937ms | 11.937ms | 11.937ms | 11.937ms | 10.000ms | 82.20 MiB | none | 83.77K/s |
| q4 | average | 1.000ms | 12.712ms | 12.689ms | 0.0% | 12.689ms | 12.689ms | 12.689ms | 12.689ms | 10.000ms | 81.21 MiB | none | 78.81K/s |
| q5 | count distinct, high card | 3.000ms | 16.489ms | 13.728ms | 0.0% | 13.728ms | 13.728ms | 13.728ms | 13.728ms | 20.000ms | 114.63 MiB | none | 72.84K/s |
| q6 | count distinct, strings | 3.000ms | 14.758ms | 13.526ms | 0.0% | 13.526ms | 13.526ms | 13.526ms | 13.526ms | 10.000ms | 118.18 MiB | none | 73.93K/s |
| q7 | min and max of a date | 1.000ms | 11.367ms | 10.859ms | 0.0% | 10.859ms | 10.859ms | 10.859ms | 10.859ms | 0.000us | 79.68 MiB | none | 92.09K/s |
| q8 | group by, low card | 5.000ms | 14.408ms | 17.496ms | 0.0% | 17.496ms | 17.496ms | 17.496ms | 17.496ms | 20.000ms | 101.88 MiB | none | 57.16K/s |
| q9 | group by and count distinct | 6.000ms | 17.214ms | 18.825ms | 0.0% | 18.825ms | 18.825ms | 18.825ms | 18.825ms | 40.000ms | 134.11 MiB | none | 53.12K/s |
| q10 | group by, several aggregates | 5.000ms | 15.258ms | 15.361ms | 0.0% | 15.361ms | 15.361ms | 15.361ms | 15.361ms | 10.000ms | 117.44 MiB | none | 65.10K/s |
| q11 | group by a string and count distinct | 9.000ms | 17.928ms | 20.858ms | 0.0% | 20.858ms | 20.858ms | 20.858ms | 20.858ms | 40.000ms | 150.60 MiB | 308.00 KiB | 47.94K/s |
| q12 | group by two strings and count distinct | 8.000ms | 18.459ms | 18.979ms | 0.0% | 18.979ms | 18.979ms | 18.979ms | 18.979ms | 40.000ms | 146.46 MiB | none | 52.69K/s |
| q13 | group by a string and top k | 6.000ms | 19.398ms | 17.127ms | 0.0% | 17.127ms | 17.127ms | 17.127ms | 17.127ms | 20.000ms | 130.84 MiB | none | 58.39K/s |
| q14 | group by a string and count distinct | 9.000ms | 20.734ms | 20.062ms | 0.0% | 20.062ms | 20.062ms | 20.062ms | 20.062ms | 50.000ms | 183.72 MiB | none | 49.85K/s |
| q15 | group by two columns and top k | 6.000ms | 16.911ms | 16.369ms | 0.0% | 16.369ms | 16.369ms | 16.369ms | 16.369ms | 20.000ms | 127.70 MiB | none | 61.09K/s |
| q16 | group by, very high card | 3.000ms | 14.724ms | 13.752ms | 0.0% | 13.752ms | 13.752ms | 13.752ms | 13.752ms | 10.000ms | 107.08 MiB | none | 72.72K/s |
| q17 | group by two, very high card | 4.000ms | 14.181ms | 14.515ms | 0.0% | 14.515ms | 14.515ms | 14.515ms | 14.515ms | 10.000ms | 115.09 MiB | none | 68.89K/s |
| q18 | group by two, no ordering | 4.000ms | 13.885ms | 14.918ms | 0.0% | 14.918ms | 14.918ms | 14.918ms | 14.918ms | 10.000ms | 114.54 MiB | none | 67.03K/s |
| q19 | group by with an extract | 5.000ms | 16.085ms | 15.593ms | 0.0% | 15.593ms | 15.593ms | 15.593ms | 15.593ms | 20.000ms | 120.85 MiB | none | 64.13K/s |
| q20 | point lookup | 2.000ms | 13.236ms | 12.784ms | 0.0% | 12.784ms | 12.784ms | 12.784ms | 12.784ms | 10.000ms | 92.38 MiB | none | 78.22K/s |
| q21 | substring scan | 3.000ms | 13.498ms | 13.537ms | 0.0% | 13.537ms | 13.537ms | 13.537ms | 13.537ms | 10.000ms | 101.31 MiB | none | 73.87K/s |
| q22 | substring scan and group by | 5.000ms | 15.738ms | 15.875ms | 0.0% | 15.875ms | 15.875ms | 15.875ms | 15.875ms | 20.000ms | 120.22 MiB | none | 62.99K/s |
| q23 | two substring scans and group by | 6.000ms | 18.582ms | 16.583ms | 0.0% | 16.583ms | 16.583ms | 16.583ms | 16.583ms | 20.000ms | 122.46 MiB | none | 60.30K/s |
| q24 | select star and top k | 8.000ms | 18.843ms | 19.624ms | 0.0% | 19.624ms | 19.624ms | 19.624ms | 19.624ms | 20.000ms | 115.85 MiB | none | 50.96K/s |
| q25 | top k by a date | 4.000ms | 13.858ms | 13.833ms | 0.0% | 13.833ms | 13.833ms | 13.833ms | 13.833ms | 0.000us | 107.20 MiB | none | 72.29K/s |
| q26 | top k by a string | 3.000ms | 15.175ms | 13.285ms | 0.0% | 13.285ms | 13.285ms | 13.285ms | 13.285ms | 10.000ms | 100.48 MiB | none | 75.27K/s |
| q27 | top k by two columns | 3.000ms | 13.386ms | 13.900ms | 0.0% | 13.900ms | 13.900ms | 13.900ms | 13.900ms | 10.000ms | 100.69 MiB | none | 71.94K/s |
| q28 | group by with a string length | 7.000ms | 15.753ms | 18.339ms | 0.0% | 18.339ms | 18.339ms | 18.339ms | 18.339ms | 20.000ms | 119.68 MiB | none | 54.53K/s |
| q29 | group by a regular expression | 7.000ms | 21.643ms | 18.594ms | 0.0% | 18.594ms | 18.594ms | 18.594ms | 18.594ms | 20.000ms | 141.62 MiB | none | 53.78K/s |
| q30 | ninety sums over one column | 10.000ms | 21.819ms | 20.120ms | 0.0% | 20.120ms | 20.120ms | 20.120ms | 20.120ms | 10.000ms | 87.52 MiB | 52.00 KiB | 49.70K/s |
| q31 | group by two and several aggregates | 6.000ms | 17.434ms | 17.069ms | 0.0% | 17.069ms | 17.069ms | 17.069ms | 17.069ms | 10.000ms | 134.33 MiB | none | 58.59K/s |
| q32 | group by a high card pair | 8.000ms | 18.419ms | 18.803ms | 0.0% | 18.803ms | 18.803ms | 18.803ms | 18.803ms | 30.000ms | 129.26 MiB | none | 53.18K/s |
| q33 | group by a high card pair, unfiltered | 4.000ms | 15.843ms | 14.926ms | 0.0% | 14.926ms | 14.926ms | 14.926ms | 14.926ms | 10.000ms | 117.73 MiB | none | 67.00K/s |
| q34 | group by a long string | 4.000ms | 14.878ms | 15.465ms | 0.0% | 15.465ms | 15.465ms | 15.465ms | 15.465ms | 10.000ms | 119.35 MiB | none | 64.66K/s |
| q35 | group by a constant and a long string | 4.000ms | 15.515ms | 15.092ms | 0.0% | 15.092ms | 15.092ms | 15.092ms | 15.092ms | 20.000ms | 123.16 MiB | none | 66.26K/s |
| q36 | group by four expressions | 4.000ms | 14.012ms | 14.836ms | 0.0% | 14.836ms | 14.836ms | 14.836ms | 14.836ms | 10.000ms | 103.70 MiB | none | 67.40K/s |
| q37 | date range and group by a URL | 6.000ms | 18.466ms | 17.121ms | 0.0% | 17.121ms | 17.121ms | 17.121ms | 17.121ms | 20.000ms | 128.54 MiB | 388.00 KiB | 58.41K/s |
| q38 | date range and group by a title | 6.000ms | 17.049ms | 18.368ms | 0.0% | 18.368ms | 18.368ms | 18.368ms | 18.368ms | 20.000ms | 124.32 MiB | none | 54.44K/s |
| q39 | date range, group by and offset | 6.000ms | 16.409ms | 17.106ms | 0.0% | 17.106ms | 17.106ms | 17.106ms | 17.106ms | 20.000ms | 116.93 MiB | none | 58.46K/s |
| q40 | date range, a case and a wide group by | 6.000ms | 17.687ms | 16.986ms | 0.0% | 16.986ms | 16.986ms | 16.986ms | 16.986ms | 10.000ms | 123.40 MiB | none | 58.87K/s |
| q41 | date range with an IN and a hash | 7.000ms | 16.698ms | 17.271ms | 0.0% | 17.271ms | 17.271ms | 17.271ms | 17.271ms | 20.000ms | 116.53 MiB | none | 57.90K/s |
| q42 | date range and a deep offset | 6.000ms | 17.294ms | 16.368ms | 0.0% | 16.368ms | 16.368ms | 16.368ms | 16.368ms | 10.000ms | 110.72 MiB | none | 61.09K/s |
| q43 | minute buckets over a date range | 5.000ms | 16.021ms | 15.712ms | 0.0% | 15.712ms | 15.712ms | 15.712ms | 15.712ms | 10.000ms | 111.18 MiB | none | 63.65K/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 213.000ms by its own clock and 686.074ms by ours, 727.320ms cold, 700.000ms of CPU, peak 183.72 MiB, 201.88K/s and 44.53 MiB/s.

Running it cost 222% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.92x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 5.776ms | 113.547ms | 86.926ms | 0.0% | 86.926ms | 86.926ms | 86.926ms | 86.926ms | 90.000ms | 61.68 MiB | 34.17 MiB | 11.50K/s |
| q2 | filtered count | 6.613ms | 92.411ms | 89.510ms | 0.0% | 89.510ms | 89.510ms | 89.510ms | 89.510ms | 90.000ms | 66.82 MiB | 312.00 KiB | 11.17K/s |
| q3 | three aggregates | 6.154ms | 90.879ms | 89.529ms | 0.0% | 89.529ms | 89.529ms | 89.529ms | 89.529ms | 100.000ms | 64.30 MiB | 736.00 KiB | 11.17K/s |
| q4 | average | 9.621ms | 97.964ms | 96.367ms | 0.0% | 96.367ms | 96.367ms | 96.367ms | 96.367ms | 120.000ms | 63.64 MiB | none | 10.38K/s |
| q5 | count distinct, high card | 10.602ms | 100.804ms | 95.742ms | 0.0% | 95.742ms | 95.742ms | 95.742ms | 95.742ms | 150.000ms | 72.94 MiB | none | 10.44K/s |
| q6 | count distinct, strings | 14.139ms | 97.277ms | 97.094ms | 0.0% | 97.094ms | 97.094ms | 97.094ms | 97.094ms | 140.000ms | 73.90 MiB | none | 10.30K/s |
| q7 | min and max of a date | 5.029ms | 87.113ms | 88.349ms | 0.0% | 88.349ms | 88.349ms | 88.349ms | 88.349ms | 120.000ms | 63.23 MiB | none | 11.32K/s |
| q8 | group by, low card | 13.974ms | 111.044ms | 99.291ms | 0.0% | 99.291ms | 99.291ms | 99.291ms | 99.291ms | 110.000ms | 75.78 MiB | none | 10.07K/s |
| q9 | group by and count distinct | 25.233ms | 108.320ms | 110.980ms | 0.0% | 110.980ms | 110.980ms | 110.980ms | 110.980ms | 130.000ms | 80.96 MiB | none | 9.01K/s |
| q10 | group by, several aggregates | 25.170ms | 108.745ms | 111.104ms | 0.0% | 111.104ms | 111.104ms | 111.104ms | 111.104ms | 160.000ms | 82.08 MiB | none | 9.00K/s |
| q11 | group by a string and count distinct | 20.604ms | 103.796ms | 112.735ms | 0.0% | 112.735ms | 112.735ms | 112.735ms | 112.735ms | 350.000ms | 82.18 MiB | none | 8.87K/s |
| q12 | group by two strings and count distinct | 19.491ms | 108.551ms | 105.262ms | 0.0% | 105.262ms | 105.262ms | 105.262ms | 105.262ms | 140.000ms | 83.06 MiB | none | 9.50K/s |
| q13 | group by a string and top k | 14.629ms | 99.353ms | 99.535ms | 0.0% | 99.535ms | 99.535ms | 99.535ms | 99.535ms | 150.000ms | 76.03 MiB | none | 10.05K/s |
| q14 | group by a string and count distinct | 20.572ms | 104.987ms | 105.691ms | 0.0% | 105.691ms | 105.691ms | 105.691ms | 105.691ms | 170.000ms | 81.45 MiB | none | 9.46K/s |
| q15 | group by two columns and top k | 15.982ms | 99.161ms | 101.570ms | 0.0% | 101.570ms | 101.570ms | 101.570ms | 101.570ms | 140.000ms | 76.76 MiB | none | 9.85K/s |
| q16 | group by, very high card | 18.765ms | 119.624ms | 101.721ms | 0.0% | 101.721ms | 101.721ms | 101.721ms | 101.721ms | 140.000ms | 74.25 MiB | none | 9.83K/s |
| q17 | group by two, very high card | 20.986ms | 105.399ms | 105.865ms | 0.0% | 105.865ms | 105.865ms | 105.865ms | 105.865ms | 140.000ms | 76.68 MiB | none | 9.45K/s |
| q18 | group by two, no ordering | 11.913ms | 96.513ms | 93.989ms | 0.0% | 93.989ms | 93.989ms | 93.989ms | 93.989ms | 130.000ms | 73.62 MiB | none | 10.64K/s |
| q19 | group by with an extract | 23.628ms | 110.246ms | 107.031ms | 0.0% | 107.031ms | 107.031ms | 107.031ms | 107.031ms | 170.000ms | 76.62 MiB | none | 9.34K/s |
| q20 | point lookup | 5.335ms | 91.673ms | 85.926ms | 0.0% | 85.926ms | 85.926ms | 85.926ms | 85.926ms | 100.000ms | 63.00 MiB | 12.00 KiB | 11.64K/s |
| q21 | substring scan | 6.250ms | 97.867ms | 90.073ms | 0.0% | 90.073ms | 90.073ms | 90.073ms | 90.073ms | 130.000ms | 67.51 MiB | 8.80 MiB | 11.10K/s |
| q22 | substring scan and group by | 15.037ms | 101.677ms | 100.741ms | 0.0% | 100.741ms | 100.741ms | 100.741ms | 100.741ms | 160.000ms | 76.38 MiB | none | 9.93K/s |
| q23 | two substring scans and group by | 18.848ms | 104.058ms | 102.700ms | 0.0% | 102.700ms | 102.700ms | 102.700ms | 102.700ms | 150.000ms | 83.65 MiB | none | 9.74K/s |
| q24 | select star and top k | 10.126ms | 97.182ms | 95.918ms | 0.0% | 95.918ms | 95.918ms | 95.918ms | 95.918ms | 170.000ms | 73.41 MiB | none | 10.43K/s |
| q25 | top k by a date | 13.025ms | 105.149ms | 100.439ms | 0.0% | 100.439ms | 100.439ms | 100.439ms | 100.439ms | 190.000ms | 71.86 MiB | none | 9.96K/s |
| q26 | top k by a string | 12.538ms | 95.672ms | 95.415ms | 0.0% | 95.415ms | 95.415ms | 95.415ms | 95.415ms | 170.000ms | 69.06 MiB | none | 10.48K/s |
| q27 | top k by two columns | 12.996ms | 100.321ms | 97.081ms | 0.0% | 97.081ms | 97.081ms | 97.081ms | 97.081ms | 110.000ms | 71.57 MiB | none | 10.30K/s |
| q30 | ninety sums over one column | 11.375ms | 94.333ms | 102.987ms | 0.0% | 102.987ms | 102.987ms | 102.987ms | 102.987ms | 300.000ms | 67.36 MiB | none | 9.71K/s |
| q31 | group by two and several aggregates | 17.446ms | 105.673ms | 104.302ms | 0.0% | 104.302ms | 104.302ms | 104.302ms | 104.302ms | 150.000ms | 77.05 MiB | none | 9.59K/s |
| q32 | group by a high card pair | 18.270ms | 117.273ms | 104.413ms | 0.0% | 104.413ms | 104.413ms | 104.413ms | 104.413ms | 190.000ms | 77.88 MiB | none | 9.58K/s |
| q33 | group by a high card pair, unfiltered | 18.806ms | 118.319ms | 102.539ms | 0.0% | 102.539ms | 102.539ms | 102.539ms | 102.539ms | 130.000ms | 76.99 MiB | none | 9.75K/s |
| q34 | group by a long string | 14.818ms | 97.795ms | 102.360ms | 0.0% | 102.360ms | 102.360ms | 102.360ms | 102.360ms | 200.000ms | 75.09 MiB | none | 9.77K/s |
| q35 | group by a constant and a long string | 18.732ms | 115.565ms | 106.535ms | 0.0% | 106.535ms | 106.535ms | 106.535ms | 106.535ms | 260.000ms | 75.02 MiB | none | 9.39K/s |
| q37 | date range and group by a URL | 15.498ms | 102.582ms | 108.562ms | 0.0% | 108.562ms | 108.562ms | 108.562ms | 108.562ms | 360.000ms | 80.06 MiB | 20.00 KiB | 9.21K/s |
| q38 | date range and group by a title | 17.151ms | 106.711ms | 107.675ms | 0.0% | 107.675ms | 107.675ms | 107.675ms | 107.675ms | 290.000ms | 79.48 MiB | none | 9.29K/s |
| q39 | date range, group by and offset | 14.944ms | 113.116ms | 105.728ms | 0.0% | 105.728ms | 105.728ms | 105.728ms | 105.728ms | 250.000ms | 76.94 MiB | none | 9.46K/s |
| q40 | date range, a case and a wide group by | 17.680ms | 106.906ms | 110.182ms | 0.0% | 110.182ms | 110.182ms | 110.182ms | 110.182ms | 240.000ms | 80.24 MiB | none | 9.08K/s |
| q41 | date range with an IN and a hash | 14.727ms | 99.269ms | 107.118ms | 0.0% | 107.118ms | 107.118ms | 107.118ms | 107.118ms | 250.000ms | 80.03 MiB | 120.00 KiB | 9.34K/s |
| q42 | date range and a deep offset | 15.000ms | 105.473ms | 105.577ms | 0.0% | 105.577ms | 105.577ms | 105.577ms | 105.577ms | 180.000ms | 79.22 MiB | none | 9.47K/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 577.483ms by its own clock and 3.935s by ours, 4.032s cold, 6.720s of CPU, peak 83.65 MiB, 67.53K/s and 14.90 MiB/s.

Running it cost 581% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.31x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-server in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 29.635ms | 30.102ms | 0.0% | 30.102ms | 30.102ms | 30.102ms | 30.102ms | not read | not read | not read | 33.22K/s |
| q2 | filtered count | 1.000ms | 31.171ms | 30.955ms | 0.0% | 30.955ms | 30.955ms | 30.955ms | 30.955ms | not read | not read | not read | 32.30K/s |
| q3 | three aggregates | 2.000ms | 30.576ms | 33.054ms | 0.0% | 33.054ms | 33.054ms | 33.054ms | 33.054ms | not read | not read | not read | 30.25K/s |
| q4 | average | 1.000ms | 31.694ms | 31.219ms | 0.0% | 31.219ms | 31.219ms | 31.219ms | 31.219ms | not read | not read | not read | 32.03K/s |
| q5 | count distinct, high card | 1.000ms | 34.336ms | 34.583ms | 0.0% | 34.583ms | 34.583ms | 34.583ms | 34.583ms | not read | not read | not read | 28.92K/s |
| q6 | count distinct, strings | 1.000ms | 30.386ms | 31.283ms | 0.0% | 31.283ms | 31.283ms | 31.283ms | 31.283ms | not read | not read | not read | 31.97K/s |
| q7 | min and max of a date | 1.000ms | 31.205ms | 33.222ms | 0.0% | 33.222ms | 33.222ms | 33.222ms | 33.222ms | not read | not read | not read | 30.10K/s |
| q8 | group by, low card | 2.000ms | 31.348ms | 31.724ms | 0.0% | 31.724ms | 31.724ms | 31.724ms | 31.724ms | not read | not read | not read | 31.52K/s |
| q9 | group by and count distinct | 2.000ms | 30.432ms | 31.839ms | 0.0% | 31.839ms | 31.839ms | 31.839ms | 31.839ms | not read | not read | not read | 31.41K/s |
| q10 | group by, several aggregates | 16.000ms | 30.883ms | 44.546ms | 0.0% | 44.546ms | 44.546ms | 44.546ms | 44.546ms | not read | not read | not read | 22.45K/s |
| q11 | group by a string and count distinct | 2.000ms | 32.073ms | 30.904ms | 0.0% | 30.904ms | 30.904ms | 30.904ms | 30.904ms | not read | not read | not read | 32.36K/s |
| q12 | group by two strings and count distinct | 2.000ms | 31.059ms | 31.624ms | 0.0% | 31.624ms | 31.624ms | 31.624ms | 31.624ms | not read | not read | not read | 31.62K/s |
| q13 | group by a string and top k | 2.000ms | 31.924ms | 29.825ms | 0.0% | 29.825ms | 29.825ms | 29.825ms | 29.825ms | not read | not read | not read | 33.53K/s |
| q14 | group by a string and count distinct | 2.000ms | 32.380ms | 31.027ms | 0.0% | 31.027ms | 31.027ms | 31.027ms | 31.027ms | not read | not read | not read | 32.23K/s |
| q15 | group by two columns and top k | 2.000ms | 32.700ms | 31.133ms | 0.0% | 31.133ms | 31.133ms | 31.133ms | 31.133ms | not read | not read | not read | 32.12K/s |
| q16 | group by, very high card | 2.000ms | 31.679ms | 31.401ms | 0.0% | 31.401ms | 31.401ms | 31.401ms | 31.401ms | not read | not read | not read | 31.85K/s |
| q17 | group by two, very high card | 2.000ms | 30.303ms | 30.990ms | 0.0% | 30.990ms | 30.990ms | 30.990ms | 30.990ms | not read | not read | not read | 32.27K/s |
| q18 | group by two, no ordering | 2.000ms | 31.059ms | 31.640ms | 0.0% | 31.640ms | 31.640ms | 31.640ms | 31.640ms | not read | not read | not read | 31.61K/s |
| q19 | group by with an extract | 2.000ms | 30.306ms | 31.156ms | 0.0% | 31.156ms | 31.156ms | 31.156ms | 31.156ms | not read | not read | not read | 32.10K/s |
| q20 | point lookup | 1.000ms | 34.526ms | 30.890ms | 0.0% | 30.890ms | 30.890ms | 30.890ms | 30.890ms | not read | not read | not read | 32.37K/s |
| q21 | substring scan | 1.000ms | 30.389ms | 30.652ms | 0.0% | 30.652ms | 30.652ms | 30.652ms | 30.652ms | not read | not read | not read | 32.62K/s |
| q22 | substring scan and group by | 3.000ms | 31.979ms | 33.026ms | 0.0% | 33.026ms | 33.026ms | 33.026ms | 33.026ms | not read | not read | not read | 30.28K/s |
| q23 | two substring scans and group by | 29.000ms | 30.855ms | 60.485ms | 0.0% | 60.485ms | 60.485ms | 60.485ms | 60.485ms | not read | not read | not read | 16.53K/s |
| q24 | select star and top k | 4.000ms | 34.924ms | 33.525ms | 0.0% | 33.525ms | 33.525ms | 33.525ms | 33.525ms | not read | not read | not read | 29.83K/s |
| q25 | top k by a date | 2.000ms | 30.468ms | 32.970ms | 0.0% | 32.970ms | 32.970ms | 32.970ms | 32.970ms | not read | not read | not read | 30.33K/s |
| q26 | top k by a string | 1.000ms | 31.098ms | 30.890ms | 0.0% | 30.890ms | 30.890ms | 30.890ms | 30.890ms | not read | not read | not read | 32.37K/s |
| q27 | top k by two columns | 1.000ms | 30.880ms | 31.189ms | 0.0% | 31.189ms | 31.189ms | 31.189ms | 31.189ms | not read | not read | not read | 32.06K/s |
| q28 | group by with a string length | 2.000ms | 31.844ms | 32.177ms | 0.0% | 32.177ms | 32.177ms | 32.177ms | 32.177ms | not read | not read | not read | 31.08K/s |
| q29 | group by a regular expression | 21.000ms | 33.132ms | 51.175ms | 0.0% | 51.175ms | 51.175ms | 51.175ms | 51.175ms | not read | not read | not read | 19.54K/s |
| q30 | ninety sums over one column | 6.000ms | 36.778ms | 35.686ms | 0.0% | 35.686ms | 35.686ms | 35.686ms | 35.686ms | not read | not read | not read | 28.02K/s |
| q31 | group by two and several aggregates | 2.000ms | 31.555ms | 31.198ms | 0.0% | 31.198ms | 31.198ms | 31.198ms | 31.198ms | not read | not read | not read | 32.05K/s |
| q32 | group by a high card pair | 15.000ms | 31.359ms | 45.123ms | 0.0% | 45.123ms | 45.123ms | 45.123ms | 45.123ms | not read | not read | not read | 22.16K/s |
| q33 | group by a high card pair, unfiltered | 2.000ms | 31.651ms | 30.685ms | 0.0% | 30.685ms | 30.685ms | 30.685ms | 30.685ms | not read | not read | not read | 32.59K/s |
| q34 | group by a long string | 2.000ms | 30.185ms | 30.712ms | 0.0% | 30.712ms | 30.712ms | 30.712ms | 30.712ms | not read | not read | not read | 32.56K/s |
| q35 | group by a constant and a long string | 2.000ms | 31.526ms | 30.920ms | 0.0% | 30.920ms | 30.920ms | 30.920ms | 30.920ms | not read | not read | not read | 32.34K/s |
| q36 | group by four expressions | 2.000ms | 31.894ms | 30.883ms | 0.0% | 30.883ms | 30.883ms | 30.883ms | 30.883ms | not read | not read | not read | 32.38K/s |
| q37 | date range and group by a URL | 3.000ms | 31.918ms | 31.986ms | 0.0% | 31.986ms | 31.986ms | 31.986ms | 31.986ms | not read | not read | not read | 31.26K/s |
| q38 | date range and group by a title | 3.000ms | 31.615ms | 31.781ms | 0.0% | 31.781ms | 31.781ms | 31.781ms | 31.781ms | not read | not read | not read | 31.47K/s |
| q39 | date range, group by and offset | 3.000ms | 32.236ms | 34.774ms | 0.0% | 34.774ms | 34.774ms | 34.774ms | 34.774ms | not read | not read | not read | 28.76K/s |
| q40 | date range, a case and a wide group by | 3.000ms | 32.993ms | 32.042ms | 0.0% | 32.042ms | 32.042ms | 32.042ms | 32.042ms | not read | not read | not read | 31.21K/s |
| q41 | date range with an IN and a hash | 3.000ms | 33.992ms | 32.784ms | 0.0% | 32.784ms | 32.784ms | 32.784ms | 32.784ms | not read | not read | not read | 30.50K/s |
| q42 | date range and a deep offset | 3.000ms | 32.178ms | 34.381ms | 0.0% | 34.381ms | 34.381ms | 34.381ms | 34.381ms | not read | not read | not read | 29.09K/s |
| q43 | minute buckets over a date range | 2.000ms | 31.676ms | 31.415ms | 0.0% | 31.415ms | 31.415ms | 31.415ms | 31.415ms | not read | not read | not read | 31.83K/s |

clickhouse-server 26.9.1.1162 over 43 of 43 queries. Total 162.000ms by its own clock and 1.444s by ours, 1.367s cold, no reading of CPU, peak not read, 265.43K/s and 58.55 MiB/s.

Running it cost 791% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.03x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 0.000us | 1.513ms | 1.378ms | 0.0% | 1.378ms | 1.378ms | 1.378ms | 1.378ms | 0.000us | 4.33 MiB | none | 725.69K/s |
| q2 | filtered count | 0.000us | 1.404ms | 1.374ms | 0.0% | 1.374ms | 1.374ms | 1.374ms | 1.374ms | 0.000us | 4.51 MiB | none | 727.80K/s |
| q3 | three aggregates | 0.000us | 1.456ms | 1.385ms | 0.0% | 1.385ms | 1.385ms | 1.385ms | 1.385ms | 0.000us | 4.39 MiB | none | 722.02K/s |
| q4 | average | 0.000us | 1.431ms | 1.311ms | 0.0% | 1.311ms | 1.311ms | 1.311ms | 1.311ms | 0.000us | 4.25 MiB | none | 762.78K/s |
| q5 | count distinct, high card | 0.000us | 1.386ms | 1.450ms | 0.0% | 1.450ms | 1.450ms | 1.450ms | 1.450ms | 0.000us | 4.26 MiB | none | 689.66K/s |
| q6 | count distinct, strings | 0.000us | 1.397ms | 1.398ms | 0.0% | 1.398ms | 1.398ms | 1.398ms | 1.398ms | 0.000us | 4.26 MiB | none | 715.31K/s |
| q7 | min and max of a date | 0.000us | 1.371ms | 1.320ms | 0.0% | 1.320ms | 1.320ms | 1.320ms | 1.320ms | 0.000us | 4.48 MiB | none | 757.58K/s |
| q8 | group by, low card | 0.000us | 1.305ms | 1.359ms | 0.0% | 1.359ms | 1.359ms | 1.359ms | 1.359ms | 0.000us | 4.40 MiB | none | 735.84K/s |
| q9 | group by and count distinct | 0.000us | 1.515ms | 1.492ms | 0.0% | 1.492ms | 1.492ms | 1.492ms | 1.492ms | 0.000us | 4.49 MiB | none | 670.24K/s |
| q10 | group by, several aggregates | 1.000ms | 1.747ms | 1.628ms | 0.0% | 1.628ms | 1.628ms | 1.628ms | 1.628ms | 0.000us | 4.73 MiB | none | 614.25K/s |
| q11 | group by a string and count distinct | 0.000us | 1.408ms | 1.341ms | 0.0% | 1.341ms | 1.341ms | 1.341ms | 1.341ms | 0.000us | 4.50 MiB | none | 745.71K/s |
| q12 | group by two strings and count distinct | 0.000us | 1.380ms | 1.431ms | 0.0% | 1.431ms | 1.431ms | 1.431ms | 1.431ms | 0.000us | 4.49 MiB | none | 698.81K/s |
| q13 | group by a string and top k | 0.000us | 1.485ms | 1.468ms | 0.0% | 1.468ms | 1.468ms | 1.468ms | 1.468ms | 0.000us | 4.49 MiB | none | 681.20K/s |
| q14 | group by a string and count distinct | 0.000us | 1.535ms | 1.481ms | 0.0% | 1.481ms | 1.481ms | 1.481ms | 1.481ms | 0.000us | 4.45 MiB | none | 675.22K/s |
| q15 | group by two columns and top k | 0.000us | 1.492ms | 1.553ms | 0.0% | 1.553ms | 1.553ms | 1.553ms | 1.553ms | 0.000us | 4.47 MiB | none | 643.92K/s |
| q16 | group by, very high card | 1.000ms | 1.526ms | 1.582ms | 0.0% | 1.582ms | 1.582ms | 1.582ms | 1.582ms | 0.000us | 4.71 MiB | none | 632.11K/s |
| q17 | group by two, very high card | 1.000ms | 1.731ms | 1.744ms | 0.0% | 1.744ms | 1.744ms | 1.744ms | 1.744ms | 0.000us | 4.76 MiB | none | 573.39K/s |
| q18 | group by two, no ordering | 1.000ms | 1.728ms | 1.593ms | 0.0% | 1.593ms | 1.593ms | 1.593ms | 1.593ms | 0.000us | 4.75 MiB | none | 627.75K/s |
| q20 | point lookup | 0.000us | 1.310ms | 1.326ms | 0.0% | 1.326ms | 1.326ms | 1.326ms | 1.326ms | 0.000us | 4.43 MiB | none | 754.15K/s |
| q21 | substring scan | 1.000ms | 1.723ms | 1.757ms | 0.0% | 1.757ms | 1.757ms | 1.757ms | 1.757ms | 0.000us | 4.71 MiB | none | 569.15K/s |
| q22 | substring scan and group by | 1.000ms | 1.773ms | 1.835ms | 0.0% | 1.835ms | 1.835ms | 1.835ms | 1.835ms | 0.000us | 4.74 MiB | none | 544.96K/s |
| q23 | two substring scans and group by | 1.000ms | 2.412ms | 2.405ms | 0.0% | 2.405ms | 2.405ms | 2.405ms | 2.405ms | 0.000us | 4.97 MiB | none | 415.80K/s |
| q24 | select star and top k | 3.000ms | 4.478ms | 4.133ms | 0.0% | 4.133ms | 4.133ms | 4.133ms | 4.133ms | 0.000us | 6.08 MiB | none | 241.95K/s |
| q25 | top k by a date | 0.000us | 1.448ms | 1.492ms | 0.0% | 1.492ms | 1.492ms | 1.492ms | 1.492ms | 0.000us | 4.76 MiB | none | 670.24K/s |
| q26 | top k by a string | 0.000us | 1.396ms | 1.446ms | 0.0% | 1.446ms | 1.446ms | 1.446ms | 1.446ms | 0.000us | 4.47 MiB | none | 691.56K/s |
| q27 | top k by two columns | 0.000us | 1.461ms | 1.482ms | 0.0% | 1.482ms | 1.482ms | 1.482ms | 1.482ms | 0.000us | 4.75 MiB | none | 674.76K/s |
| q28 | group by with a string length | 1.000ms | 1.818ms | 1.763ms | 0.0% | 1.763ms | 1.763ms | 1.763ms | 1.763ms | 0.000us | 4.74 MiB | none | 567.21K/s |
| q29 | group by a regular expression | 1.000ms | 2.456ms | 2.315ms | 0.0% | 2.315ms | 2.315ms | 2.315ms | 2.315ms | 0.000us | 4.90 MiB | none | 431.97K/s |
| q30 | ninety sums over one column | 1.000ms | 2.419ms | 2.390ms | 0.0% | 2.390ms | 2.390ms | 2.390ms | 2.390ms | 0.000us | 4.96 MiB | none | 418.41K/s |
| q31 | group by two and several aggregates | 0.000us | 1.546ms | 1.514ms | 0.0% | 1.514ms | 1.514ms | 1.514ms | 1.514ms | 0.000us | 4.51 MiB | none | 660.50K/s |
| q32 | group by a high card pair | 0.000us | 1.540ms | 1.500ms | 0.0% | 1.500ms | 1.500ms | 1.500ms | 1.500ms | 0.000us | 4.48 MiB | none | 666.67K/s |
| q34 | group by a long string | 1.000ms | 2.014ms | 2.000ms | 0.0% | 2.000ms | 2.000ms | 2.000ms | 2.000ms | 0.000us | 4.75 MiB | none | 500.00K/s |
| q35 | group by a constant and a long string | 1.000ms | 2.104ms | 2.095ms | 0.0% | 2.095ms | 2.095ms | 2.095ms | 2.095ms | 0.000us | 4.76 MiB | none | 477.33K/s |
| q36 | group by four expressions | 1.000ms | 1.774ms | 1.694ms | 0.0% | 1.694ms | 1.694ms | 1.694ms | 1.694ms | 0.000us | 4.99 MiB | none | 590.32K/s |
| q37 | date range and group by a URL | 1.000ms | 1.802ms | 1.903ms | 0.0% | 1.903ms | 1.903ms | 1.903ms | 1.903ms | 0.000us | 4.76 MiB | none | 525.49K/s |
| q38 | date range and group by a title | 1.000ms | 1.890ms | 1.918ms | 0.0% | 1.918ms | 1.918ms | 1.918ms | 1.918ms | 0.000us | 4.73 MiB | none | 521.38K/s |
| q39 | date range, group by and offset | 1.000ms | 1.833ms | 1.853ms | 0.0% | 1.853ms | 1.853ms | 1.853ms | 1.853ms | 0.000us | 4.75 MiB | none | 539.67K/s |
| q40 | date range, a case and a wide group by | 1.000ms | 2.304ms | 2.347ms | 0.0% | 2.347ms | 2.347ms | 2.347ms | 2.347ms | 0.000us | 4.95 MiB | none | 426.08K/s |
| q41 | date range with an IN and a hash | 0.000us | 1.465ms | 1.492ms | 0.0% | 1.492ms | 1.492ms | 1.492ms | 1.492ms | 0.000us | 4.50 MiB | none | 670.24K/s |
| q42 | date range and a deep offset | 0.000us | 1.480ms | 1.489ms | 0.0% | 1.489ms | 1.489ms | 1.489ms | 1.489ms | 0.000us | 4.51 MiB | none | 671.59K/s |
| q43 | minute buckets over a date range | 0.000us | 1.604ms | 1.463ms | 0.0% | 1.463ms | 1.463ms | 1.463ms | 1.463ms | 0.000us | 4.81 MiB | none | 683.53K/s |

rudb rudb 0.2.31 over 41 of 43 queries. Total 20.000ms by its own clock and 69.900ms by ours, 70.860ms cold, 0.000us of CPU, peak 6.08 MiB, 2.05M/s and 452.20 MiB/s.

Running it cost 250% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.15x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 1000 rows, one out of every 99998 of the 99997497 in the full file, which is a development loop rather than the suite
- every query here ran within 1.49x of every other one, so most of what was timed is whatever they have in common rather than the queries
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

- duckdb ran every query within 1.49x of every other one, and this suite spreads over 6x on an engine it is measuring
- duckdb-pinned ran every query within 1.70x of every other one, and this suite spreads over 6x on an engine it is measuring
- clickhouse-local ran every query within 1.57x of every other one, and this suite spreads over 6x on an engine it is measuring
- datafusion ran every query within 1.92x of every other one, and this suite spreads over 6x on an engine it is measuring
- polars ran every query within 1.31x of every other one, and this suite spreads over 6x on an engine it is measuring

A column that flat is not a column about the queries. Read those numbers as an upper bound on the engine and not as a measurement of it.

polars did not run q28, because the SQLContext has no strlen and no character length function.

polars did not run q29, because the SQLContext has no regexp_replace.

polars did not run q36, because the SQLContext rejects the repeated ClientIP output name in the group by.

polars did not run q43, because the SQLContext has no date_trunc.

So the polars column is 39 of 43 queries and its ratio is over the shared ones.

rudb did not run q19, because rudb's hash aggregate does not spill and this groups by a key that is close to unique over a hundred million rows, so the table outgrows the machine. tamnd/rudb#220, milestone E5.

rudb did not run q33, because rudb's hash aggregate does not spill and this groups by a key that is close to unique over a hundred million rows, so the table outgrows the machine. tamnd/rudb#220, milestone E5.

So the rudb column is 41 of 43 queries and its ratio is over the shared ones.

Answers differ, so this is not a comparison: q13: duckdb-pinned does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q13: clickhouse-local does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q13: datafusion does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q13: polars does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q13: clickhouse-server does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q13: rudb does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q14: duckdb-pinned does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q14: clickhouse-local does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q14: datafusion does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q14: polars does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q14: clickhouse-server does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q14: rudb does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q15: clickhouse-local does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q15: datafusion does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q15: polars does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q15: clickhouse-server does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q15: rudb does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q34: duckdb-pinned does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q34: clickhouse-local does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q34: datafusion does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q34: polars does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q34: clickhouse-server does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q34: rudb does not agree with duckdb: 10 numbers against 10

These were answered differently and the data does not say which is right:

- q16: ORDER BY COUNT(*) DESC LIMIT 10 over user IDs, where the tenth place is a tie between many users with the same count and the engine picks.
- q17: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q18: LIMIT 10 with no ORDER BY at all, so the ten rows are whichever ten the group by handed back first and the query did not ask for any particular ten.
- q19: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q31: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q32: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q33: ORDER BY COUNT(*) DESC LIMIT 10 over WatchID and ClientIP unfiltered, where almost every group has a count of one and the ten that come back are arbitrary.
- q35: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q36: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.

So they are not checked, and a wrong answer from any engine on one of them would go unnoticed here. Every other query in the suite is checked in full.

These were answered differently and which engine is wrong is settled:

- q4: AVG(UserID) over a hundred million bigints, where the pinned DuckDB sums the column short by a multiple of 2^64 and rudb does not. Its own hugeint and decimal paths, the same column summed out of a table, its own per counter group sums, and adding the column up outside a database all give rudb's answer, and it reduces to two statements over a 1.3 MB one column parquet file. Written up as 14.10.1 in tamnd/rudb `spec/14-testing.md`, which is the list of places where DuckDB is wrong and we are not. tamnd/rudb-compat#12.

So they are a known difference rather than an open one, and the write up is where to go to disagree with that.

Hot here means page cache warm and not buffer pool warm for every row but clickhouse-server, which stayed up across the whole suite with its own caches warm. That is the stronger kind of hot, so a ratio against that column is a ratio between two different quantities.

