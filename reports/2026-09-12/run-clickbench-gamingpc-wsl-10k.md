# clickbench on gamingpc-wsl

This is one run of the clickbench suite on gamingpc-wsl, over 7 engines and 43 queries, with 1 hot run of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

It ran over 10000 rows, one out of every 10000 of the 99997497 in the full file, which is a development loop and not the suite. Nothing here is comparable to a full run or to anybody else's number.

## What ran

| what | it was |
| --- | --- |
| suite | clickbench |
| queries | 43 |
| tables | 1.70 MiB of Parquet in 1 table |
| rows | 10000 in the table every query reads |
| sample | 10000 rows, one out of every 10000 of the 99997497 in the full file |
| summary | median with the interquartile range, per reporting rule two |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --rows 10000 --runs 1 --report
```

The data goes under `RUDB_BENCH_DATA`, which defaults to `~/rudb-data`, and the engines have to be on the path. An engine that is not gets a row in the abstentions below rather than being left out of the table.

## The machine

| fact | value | source |
| --- | --- | --- |

A fact that says it was not read is a fact about this machine and not a gap in the report. A field that had silently defaulted would be a lie that survived into it.

## The engines

| engine | version | state | load | load cpu | on disk | that size is | format | machine load |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 109.148ms | 90.000ms | 4.01 MiB | its own database file | its own | 3.49 to 3.37 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 129.691ms | 120.000ms | 3.76 MiB | its own database file | its own | 3.37 to 3.37 |
| clickhouse-local | 26.9.1.1162 | ran | 148.513ms | 220.000ms | 2.68 MiB | its own MergeTree parts, as system.parts counts the active ones | its own | 3.16 to 2.98 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 1.70 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 3.37 to 3.26 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 1.70 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 3.26 to 3.16 |
| clickhouse-server | 26.9.1.1162 | ran | 202.283ms | not read | 2.61 MiB | its own MergeTree parts, sorted by hits by (CounterID, EventDate, UserID, EventTime, WatchID), as system.parts counts the active ones | its own | 2.98 to 2.90 |
| rudb | rudb 0.2.31 | ran | 0.000us | 0.000us | 1.70 MiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 3.37 to 3.37 |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 124.000ms | 511.618ms | +313% | 498.024ms | 430.000ms | 0.84 | 38.50 MiB | none | 3.47M/s | 588.54 MiB/s | 1.00x |
| duckdb-pinned | 148.000ms | 1.049s | +609% | 1.059s | 890.000ms | 0.85 | 52.77 MiB | none | 2.91M/s | 493.10 MiB/s | 1.19x |
| clickhouse-local | 183.000ms | 2.371s | +1196% | 2.347s | 3.520s | 1.48 | 212.51 MiB | none | 2.35M/s | 398.79 MiB/s | 1.48x |
| datafusion | 327.000ms | 812.483ms | +148% | 799.749ms | 2.510s | 3.09 | 189.13 MiB | none | 1.31M/s | 223.18 MiB/s | 2.64x |
| polars | 620.765ms | 4.094s | +560% | 4.114s | 8.920s | 2.18 | 87.39 MiB | none | 628.26K/s | 106.63 MiB/s | 5.01x |
| clickhouse-server | 149.000ms | 1.435s | +863% | 1.384s | not read | not read | not read | not read | 2.89M/s | 489.79 MiB/s | 1.20x |
| rudb | 121.000ms | 168.949ms | +40% | 170.324ms | 20.000ms | 0.12 | 19.83 MiB | none | 3.39M/s | 575.08 MiB/s | 0.98x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | clickhouse-server | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 10.000ms | 0.000us | 2.000ms | 1.000ms | 5.535ms | 1.000ms | 0.000us |
| q2 | filtered count | 1.000ms | 1.000ms | 2.000ms | 4.000ms | 5.809ms | 1.000ms | 0.000us |
| q3 | three aggregates | 1.000ms | 1.000ms | 3.000ms | 3.000ms | 6.218ms | 2.000ms | 0.000us |
| q4 | average | 1.000ms | 1.000ms | 3.000ms | 2.000ms | 5.362ms | 1.000ms | 0.000us |
| q5 | count distinct, high card | 2.000ms | 2.000ms | 3.000ms | 6.000ms | 12.622ms | 2.000ms | 1.000ms |
| q6 | count distinct, strings | 2.000ms | 2.000ms | 3.000ms | 7.000ms | 13.143ms | 2.000ms | 1.000ms |
| q7 | min and max of a date | 1.000ms | 1.000ms | 3.000ms | 1.000ms | 7.830ms | 1.000ms | 0.000us |
| q8 | group by, low card | 1.000ms | 5.000ms | 4.000ms | 5.000ms | 14.426ms | 2.000ms | 0.000us |
| q9 | group by and count distinct | 4.000ms | 4.000ms | 3.000ms | 8.000ms | 20.957ms | 2.000ms | 2.000ms |
| q10 | group by, several aggregates | 4.000ms | 6.000ms | 3.000ms | 8.000ms | 25.581ms | 16.000ms | 2.000ms |
| q11 | group by a string and count distinct | 4.000ms | 3.000ms | 3.000ms | 9.000ms | 21.039ms | 2.000ms | 1.000ms |
| q12 | group by two strings and count distinct | 3.000ms | 4.000ms | 3.000ms | 9.000ms | 22.534ms | 2.000ms | 1.000ms |
| q13 | group by a string and top k | 3.000ms | 3.000ms | 3.000ms | 8.000ms | 15.600ms | 2.000ms | 1.000ms |
| q14 | group by a string and count distinct | 4.000ms | 4.000ms | 3.000ms | 12.000ms | 22.475ms | 2.000ms | 2.000ms |
| q15 | group by two columns and top k | 2.000ms | 2.000ms | 4.000ms | 9.000ms | 19.253ms | 2.000ms | 2.000ms |
| q16 | group by, very high card | 3.000ms | 2.000ms | 3.000ms | 6.000ms | 19.151ms | 2.000ms | 2.000ms |
| q17 | group by two, very high card | 3.000ms | 3.000ms | 4.000ms | 9.000ms | 24.577ms | 3.000ms | 3.000ms |
| q18 | group by two, no ordering | 3.000ms | 3.000ms | 3.000ms | 9.000ms | 13.517ms | 2.000ms | 3.000ms |
| q19 | group by with an extract | 4.000ms | 3.000ms | 4.000ms | 7.000ms | 27.215ms | 3.000ms | no dialect |
| q20 | point lookup | 1.000ms | 1.000ms | 4.000ms | 3.000ms | 6.334ms | 1.000ms | 0.000us |
| q21 | substring scan | 1.000ms | 2.000ms | 4.000ms | 5.000ms | 9.119ms | 1.000ms | 4.000ms |
| q22 | substring scan and group by | 1.000ms | 3.000ms | 4.000ms | 7.000ms | 18.870ms | 3.000ms | 4.000ms |
| q23 | two substring scans and group by | 4.000ms | 6.000ms | 8.000ms | 10.000ms | 22.718ms | 4.000ms | 9.000ms |
| q24 | select star and top k | 6.000ms | 11.000ms | 8.000ms | 13.000ms | 12.527ms | 4.000ms | 21.000ms |
| q25 | top k by a date | 2.000ms | 1.000ms | 4.000ms | 4.000ms | 12.430ms | 2.000ms | 1.000ms |
| q26 | top k by a string | 1.000ms | 1.000ms | 3.000ms | 4.000ms | 14.245ms | 2.000ms | 1.000ms |
| q27 | top k by two columns | 1.000ms | 1.000ms | 4.000ms | 5.000ms | 11.878ms | 2.000ms | 1.000ms |
| q28 | group by with a string length | 3.000ms | 3.000ms | 4.000ms | 7.000ms | no dialect | 3.000ms | 4.000ms |
| q29 | group by a regular expression | 8.000ms | 7.000ms | 8.000ms | 13.000ms | no dialect | 22.000ms | 9.000ms |
| q30 | ninety sums over one column | 3.000ms | 13.000ms | 8.000ms | 11.000ms | 12.069ms | 6.000ms | 3.000ms |
| q31 | group by two and several aggregates | 3.000ms | 3.000ms | 4.000ms | 7.000ms | 13.490ms | 2.000ms | 2.000ms |
| q32 | group by a high card pair | 2.000ms | 3.000ms | 4.000ms | 9.000ms | 16.454ms | 15.000ms | 2.000ms |
| q33 | group by a high card pair, unfiltered | 3.000ms | 3.000ms | 4.000ms | 6.000ms | 17.372ms | 2.000ms | no dialect |
| q34 | group by a long string | 4.000ms | 3.000ms | 5.000ms | 10.000ms | 19.757ms | 3.000ms | 6.000ms |
| q35 | group by a constant and a long string | 3.000ms | 4.000ms | 4.000ms | 14.000ms | 20.777ms | 3.000ms | 7.000ms |
| q36 | group by four expressions | 3.000ms | 3.000ms | 3.000ms | 7.000ms | no dialect | 2.000ms | 4.000ms |
| q37 | date range and group by a URL | 3.000ms | 4.000ms | 6.000ms | 10.000ms | 22.872ms | 3.000ms | 4.000ms |
| q38 | date range and group by a title | 3.000ms | 4.000ms | 6.000ms | 12.000ms | 19.769ms | 3.000ms | 4.000ms |
| q39 | date range, group by and offset | 3.000ms | 4.000ms | 6.000ms | 11.000ms | 15.774ms | 3.000ms | 4.000ms |
| q40 | date range, a case and a wide group by | 3.000ms | 5.000ms | 7.000ms | 12.000ms | 19.639ms | 4.000ms | 7.000ms |
| q41 | date range with an IN and a hash | 3.000ms | 4.000ms | 6.000ms | 7.000ms | 15.653ms | 3.000ms | 1.000ms |
| q42 | date range and a deep offset | 2.000ms | 6.000ms | 5.000ms | 8.000ms | 16.174ms | 3.000ms | 1.000ms |
| q43 | minute buckets over a date range | 2.000ms | 3.000ms | 5.000ms | 9.000ms | no dialect | 3.000ms | 1.000ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 10.000ms | 9.348ms | 17.787ms | 0.0% | 17.787ms | 17.787ms | 17.787ms | 17.787ms | 80.000ms | 26.92 MiB | none | 562.21K/s |
| q2 | filtered count | 1.000ms | 10.011ms | 9.804ms | 0.0% | 9.804ms | 9.804ms | 9.804ms | 9.804ms | 10.000ms | 27.98 MiB | none | 1.02M/s |
| q3 | three aggregates | 1.000ms | 10.346ms | 9.382ms | 0.0% | 9.382ms | 9.382ms | 9.382ms | 9.382ms | 10.000ms | 28.23 MiB | none | 1.07M/s |
| q4 | average | 1.000ms | 9.228ms | 9.339ms | 0.0% | 9.339ms | 9.339ms | 9.339ms | 9.339ms | 0.000us | 27.73 MiB | none | 1.07M/s |
| q5 | count distinct, high card | 2.000ms | 10.646ms | 10.929ms | 0.0% | 10.929ms | 10.929ms | 10.929ms | 10.929ms | 0.000us | 30.23 MiB | none | 915.00K/s |
| q6 | count distinct, strings | 2.000ms | 10.987ms | 11.294ms | 0.0% | 11.294ms | 11.294ms | 11.294ms | 11.294ms | 10.000ms | 30.23 MiB | none | 885.43K/s |
| q7 | min and max of a date | 1.000ms | 9.086ms | 9.096ms | 0.0% | 9.096ms | 9.096ms | 9.096ms | 9.096ms | 0.000us | 27.23 MiB | none | 1.10M/s |
| q8 | group by, low card | 1.000ms | 9.921ms | 9.797ms | 0.0% | 9.797ms | 9.797ms | 9.797ms | 9.797ms | 10.000ms | 29.50 MiB | none | 1.02M/s |
| q9 | group by and count distinct | 4.000ms | 12.374ms | 13.158ms | 0.0% | 13.158ms | 13.158ms | 13.158ms | 13.158ms | 10.000ms | 37.25 MiB | none | 759.99K/s |
| q10 | group by, several aggregates | 4.000ms | 13.649ms | 12.818ms | 0.0% | 12.818ms | 12.818ms | 12.818ms | 12.818ms | 10.000ms | 38.50 MiB | none | 780.15K/s |
| q11 | group by a string and count distinct | 4.000ms | 12.034ms | 12.025ms | 0.0% | 12.025ms | 12.025ms | 12.025ms | 12.025ms | 10.000ms | 34.94 MiB | none | 831.60K/s |
| q12 | group by two strings and count distinct | 3.000ms | 12.455ms | 12.195ms | 0.0% | 12.195ms | 12.195ms | 12.195ms | 12.195ms | 20.000ms | 37.25 MiB | none | 820.01K/s |
| q13 | group by a string and top k | 3.000ms | 11.374ms | 11.164ms | 0.0% | 11.164ms | 11.164ms | 11.164ms | 11.164ms | 10.000ms | 31.50 MiB | none | 895.74K/s |
| q14 | group by a string and count distinct | 4.000ms | 12.349ms | 12.584ms | 0.0% | 12.584ms | 12.584ms | 12.584ms | 12.584ms | 10.000ms | 37.45 MiB | none | 794.66K/s |
| q15 | group by two columns and top k | 2.000ms | 11.296ms | 11.556ms | 0.0% | 11.556ms | 11.556ms | 11.556ms | 11.556ms | 0.000us | 31.50 MiB | none | 865.35K/s |
| q16 | group by, very high card | 3.000ms | 11.087ms | 11.070ms | 0.0% | 11.070ms | 11.070ms | 11.070ms | 11.070ms | 10.000ms | 33.07 MiB | none | 903.34K/s |
| q17 | group by two, very high card | 3.000ms | 12.033ms | 11.609ms | 0.0% | 11.609ms | 11.609ms | 11.609ms | 11.609ms | 10.000ms | 35.09 MiB | none | 861.40K/s |
| q18 | group by two, no ordering | 3.000ms | 12.022ms | 11.807ms | 0.0% | 11.807ms | 11.807ms | 11.807ms | 11.807ms | 10.000ms | 35.99 MiB | none | 846.96K/s |
| q19 | group by with an extract | 4.000ms | 12.002ms | 12.525ms | 0.0% | 12.525ms | 12.525ms | 12.525ms | 12.525ms | 10.000ms | 36.07 MiB | none | 798.40K/s |
| q20 | point lookup | 1.000ms | 10.002ms | 9.081ms | 0.0% | 9.081ms | 9.081ms | 9.081ms | 9.081ms | 10.000ms | 27.48 MiB | none | 1.10M/s |
| q21 | substring scan | 1.000ms | 10.269ms | 10.141ms | 0.0% | 10.141ms | 10.141ms | 10.141ms | 10.141ms | 10.000ms | 29.07 MiB | none | 986.10K/s |
| q22 | substring scan and group by | 1.000ms | 10.792ms | 10.703ms | 0.0% | 10.703ms | 10.703ms | 10.703ms | 10.703ms | 0.000us | 29.75 MiB | none | 934.32K/s |
| q23 | two substring scans and group by | 4.000ms | 12.444ms | 13.149ms | 0.0% | 13.149ms | 13.149ms | 13.149ms | 13.149ms | 10.000ms | 34.00 MiB | none | 760.51K/s |
| q24 | select star and top k | 6.000ms | 15.645ms | 15.589ms | 0.0% | 15.589ms | 15.589ms | 15.589ms | 15.589ms | 20.000ms | 37.55 MiB | none | 641.48K/s |
| q25 | top k by a date | 2.000ms | 11.149ms | 10.546ms | 0.0% | 10.546ms | 10.546ms | 10.546ms | 10.546ms | 10.000ms | 30.48 MiB | none | 948.23K/s |
| q26 | top k by a string | 1.000ms | 9.826ms | 9.630ms | 0.0% | 9.630ms | 9.630ms | 9.630ms | 9.630ms | 0.000us | 27.98 MiB | none | 1.04M/s |
| q27 | top k by two columns | 1.000ms | 9.856ms | 9.703ms | 0.0% | 9.703ms | 9.703ms | 9.703ms | 9.703ms | 10.000ms | 28.67 MiB | none | 1.03M/s |
| q28 | group by with a string length | 3.000ms | 11.834ms | 12.354ms | 0.0% | 12.354ms | 12.354ms | 12.354ms | 12.354ms | 10.000ms | 32.00 MiB | none | 809.45K/s |
| q29 | group by a regular expression | 8.000ms | 16.069ms | 16.557ms | 0.0% | 16.557ms | 16.557ms | 16.557ms | 16.557ms | 10.000ms | 32.74 MiB | none | 603.97K/s |
| q30 | ninety sums over one column | 3.000ms | 12.953ms | 12.618ms | 0.0% | 12.618ms | 12.618ms | 12.618ms | 12.618ms | 10.000ms | 31.48 MiB | none | 792.52K/s |
| q31 | group by two and several aggregates | 3.000ms | 11.615ms | 12.013ms | 0.0% | 12.013ms | 12.013ms | 12.013ms | 12.013ms | 10.000ms | 34.32 MiB | none | 832.43K/s |
| q32 | group by a high card pair | 2.000ms | 12.586ms | 11.439ms | 0.0% | 11.439ms | 11.439ms | 11.439ms | 11.439ms | 10.000ms | 34.30 MiB | none | 874.20K/s |
| q33 | group by a high card pair, unfiltered | 3.000ms | 12.477ms | 12.509ms | 0.0% | 12.509ms | 12.509ms | 12.509ms | 12.509ms | 0.000us | 35.29 MiB | none | 799.42K/s |
| q34 | group by a long string | 4.000ms | 12.400ms | 12.628ms | 0.0% | 12.628ms | 12.628ms | 12.628ms | 12.628ms | 10.000ms | 35.25 MiB | none | 791.89K/s |
| q35 | group by a constant and a long string | 3.000ms | 12.499ms | 20.160ms | 0.0% | 20.160ms | 20.160ms | 20.160ms | 20.160ms | 10.000ms | 35.45 MiB | none | 496.03K/s |
| q36 | group by four expressions | 3.000ms | 11.735ms | 11.734ms | 0.0% | 11.734ms | 11.734ms | 11.734ms | 11.734ms | 10.000ms | 34.57 MiB | none | 852.22K/s |
| q37 | date range and group by a URL | 3.000ms | 12.231ms | 12.007ms | 0.0% | 12.007ms | 12.007ms | 12.007ms | 12.007ms | 0.000us | 32.52 MiB | none | 832.85K/s |
| q38 | date range and group by a title | 3.000ms | 11.856ms | 11.628ms | 0.0% | 11.628ms | 11.628ms | 11.628ms | 11.628ms | 10.000ms | 32.75 MiB | none | 859.99K/s |
| q39 | date range, group by and offset | 3.000ms | 11.130ms | 11.580ms | 0.0% | 11.580ms | 11.580ms | 11.580ms | 11.580ms | 10.000ms | 32.00 MiB | none | 863.56K/s |
| q40 | date range, a case and a wide group by | 3.000ms | 12.276ms | 11.897ms | 0.0% | 11.897ms | 11.897ms | 11.897ms | 11.897ms | 10.000ms | 33.75 MiB | none | 840.55K/s |
| q41 | date range with an IN and a hash | 3.000ms | 11.727ms | 11.745ms | 0.0% | 11.745ms | 11.745ms | 11.745ms | 11.745ms | 0.000us | 33.50 MiB | none | 851.43K/s |
| q42 | date range and a deep offset | 2.000ms | 11.549ms | 11.321ms | 0.0% | 11.321ms | 11.321ms | 11.321ms | 11.321ms | 10.000ms | 32.50 MiB | none | 883.31K/s |
| q43 | minute buckets over a date range | 2.000ms | 10.856ms | 10.947ms | 0.0% | 10.947ms | 10.947ms | 10.947ms | 10.947ms | 10.000ms | 31.48 MiB | none | 913.49K/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 124.000ms by its own clock and 511.618ms by ours, 498.024ms cold, 430.000ms of CPU, peak 38.50 MiB, 3.47M/s and 588.54 MiB/s.

Running it cost 313% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.22x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 0.000us | 21.927ms | 21.274ms | 0.0% | 21.274ms | 21.274ms | 21.274ms | 21.274ms | 20.000ms | 39.83 MiB | none | 470.06K/s |
| q2 | filtered count | 1.000ms | 21.421ms | 22.416ms | 0.0% | 22.416ms | 22.416ms | 22.416ms | 22.416ms | 10.000ms | 40.08 MiB | none | 446.11K/s |
| q3 | three aggregates | 1.000ms | 22.218ms | 21.574ms | 0.0% | 21.574ms | 21.574ms | 21.574ms | 21.574ms | 10.000ms | 40.07 MiB | none | 463.52K/s |
| q4 | average | 1.000ms | 22.266ms | 23.283ms | 0.0% | 23.283ms | 23.283ms | 23.283ms | 23.283ms | 40.000ms | 40.33 MiB | none | 429.50K/s |
| q5 | count distinct, high card | 2.000ms | 23.027ms | 22.726ms | 0.0% | 22.726ms | 22.726ms | 22.726ms | 22.726ms | 20.000ms | 43.32 MiB | none | 440.02K/s |
| q6 | count distinct, strings | 2.000ms | 22.693ms | 23.135ms | 0.0% | 23.135ms | 23.135ms | 23.135ms | 23.135ms | 20.000ms | 41.70 MiB | none | 432.25K/s |
| q7 | min and max of a date | 1.000ms | 21.936ms | 21.323ms | 0.0% | 21.323ms | 21.323ms | 21.323ms | 21.323ms | 20.000ms | 39.90 MiB | none | 468.98K/s |
| q8 | group by, low card | 5.000ms | 25.956ms | 25.842ms | 0.0% | 25.842ms | 25.842ms | 25.842ms | 25.842ms | 20.000ms | 42.65 MiB | none | 386.97K/s |
| q9 | group by and count distinct | 4.000ms | 25.353ms | 24.746ms | 0.0% | 24.746ms | 24.746ms | 24.746ms | 24.746ms | 30.000ms | 49.20 MiB | none | 404.11K/s |
| q10 | group by, several aggregates | 6.000ms | 25.526ms | 26.973ms | 0.0% | 26.973ms | 26.973ms | 26.973ms | 26.973ms | 30.000ms | 49.65 MiB | none | 370.74K/s |
| q11 | group by a string and count distinct | 3.000ms | 24.214ms | 23.992ms | 0.0% | 23.992ms | 23.992ms | 23.992ms | 23.992ms | 20.000ms | 46.83 MiB | none | 416.81K/s |
| q12 | group by two strings and count distinct | 4.000ms | 24.288ms | 24.736ms | 0.0% | 24.736ms | 24.736ms | 24.736ms | 24.736ms | 20.000ms | 46.85 MiB | none | 404.27K/s |
| q13 | group by a string and top k | 3.000ms | 23.476ms | 23.175ms | 0.0% | 23.175ms | 23.175ms | 23.175ms | 23.175ms | 20.000ms | 42.02 MiB | none | 431.50K/s |
| q14 | group by a string and count distinct | 4.000ms | 25.084ms | 25.549ms | 0.0% | 25.549ms | 25.549ms | 25.549ms | 25.549ms | 20.000ms | 48.44 MiB | none | 391.40K/s |
| q15 | group by two columns and top k | 2.000ms | 24.959ms | 23.958ms | 0.0% | 23.958ms | 23.958ms | 23.958ms | 23.958ms | 20.000ms | 43.39 MiB | none | 417.40K/s |
| q16 | group by, very high card | 2.000ms | 23.685ms | 23.177ms | 0.0% | 23.177ms | 23.177ms | 23.177ms | 23.177ms | 20.000ms | 44.36 MiB | none | 431.46K/s |
| q17 | group by two, very high card | 3.000ms | 24.262ms | 23.480ms | 0.0% | 23.480ms | 23.480ms | 23.480ms | 23.480ms | 20.000ms | 45.10 MiB | none | 425.89K/s |
| q18 | group by two, no ordering | 3.000ms | 24.320ms | 22.641ms | 0.0% | 22.641ms | 22.641ms | 22.641ms | 22.641ms | 10.000ms | 45.16 MiB | none | 441.68K/s |
| q19 | group by with an extract | 3.000ms | 23.945ms | 24.156ms | 0.0% | 24.156ms | 24.156ms | 24.156ms | 24.156ms | 20.000ms | 45.89 MiB | none | 413.98K/s |
| q20 | point lookup | 1.000ms | 23.352ms | 22.046ms | 0.0% | 22.046ms | 22.046ms | 22.046ms | 22.046ms | 20.000ms | 39.33 MiB | none | 453.60K/s |
| q21 | substring scan | 2.000ms | 24.019ms | 23.051ms | 0.0% | 23.051ms | 23.051ms | 23.051ms | 23.051ms | 20.000ms | 41.08 MiB | none | 433.82K/s |
| q22 | substring scan and group by | 3.000ms | 23.901ms | 23.393ms | 0.0% | 23.393ms | 23.393ms | 23.393ms | 23.393ms | 10.000ms | 41.89 MiB | none | 427.48K/s |
| q23 | two substring scans and group by | 6.000ms | 26.041ms | 26.917ms | 0.0% | 26.917ms | 26.917ms | 26.917ms | 26.917ms | 20.000ms | 48.29 MiB | none | 371.51K/s |
| q24 | select star and top k | 11.000ms | 32.232ms | 31.336ms | 0.0% | 31.336ms | 31.336ms | 31.336ms | 31.336ms | 30.000ms | 52.58 MiB | none | 319.12K/s |
| q25 | top k by a date | 1.000ms | 22.007ms | 21.919ms | 0.0% | 21.919ms | 21.919ms | 21.919ms | 21.919ms | 20.000ms | 40.81 MiB | none | 456.23K/s |
| q26 | top k by a string | 1.000ms | 22.184ms | 22.294ms | 0.0% | 22.294ms | 22.294ms | 22.294ms | 22.294ms | 10.000ms | 40.58 MiB | none | 448.55K/s |
| q27 | top k by two columns | 1.000ms | 22.516ms | 21.958ms | 0.0% | 21.958ms | 21.958ms | 21.958ms | 21.958ms | 20.000ms | 40.82 MiB | none | 455.41K/s |
| q28 | group by with a string length | 3.000ms | 25.838ms | 24.217ms | 0.0% | 24.217ms | 24.217ms | 24.217ms | 24.217ms | 20.000ms | 44.33 MiB | none | 412.93K/s |
| q29 | group by a regular expression | 7.000ms | 28.644ms | 28.534ms | 0.0% | 28.534ms | 28.534ms | 28.534ms | 28.534ms | 30.000ms | 45.12 MiB | none | 350.46K/s |
| q30 | ninety sums over one column | 13.000ms | 36.213ms | 35.339ms | 0.0% | 35.339ms | 35.339ms | 35.339ms | 35.339ms | 30.000ms | 52.77 MiB | none | 282.97K/s |
| q31 | group by two and several aggregates | 3.000ms | 25.314ms | 23.917ms | 0.0% | 23.917ms | 23.917ms | 23.917ms | 23.917ms | 20.000ms | 46.01 MiB | none | 418.11K/s |
| q32 | group by a high card pair | 3.000ms | 24.957ms | 24.786ms | 0.0% | 24.786ms | 24.786ms | 24.786ms | 24.786ms | 10.000ms | 46.03 MiB | none | 403.45K/s |
| q33 | group by a high card pair, unfiltered | 3.000ms | 23.720ms | 24.439ms | 0.0% | 24.439ms | 24.439ms | 24.439ms | 24.439ms | 20.000ms | 46.31 MiB | none | 409.18K/s |
| q34 | group by a long string | 3.000ms | 24.758ms | 24.338ms | 0.0% | 24.338ms | 24.338ms | 24.338ms | 24.338ms | 20.000ms | 45.26 MiB | none | 410.88K/s |
| q35 | group by a constant and a long string | 4.000ms | 24.647ms | 24.498ms | 0.0% | 24.498ms | 24.498ms | 24.498ms | 24.498ms | 20.000ms | 45.39 MiB | none | 408.20K/s |
| q36 | group by four expressions | 3.000ms | 24.417ms | 24.142ms | 0.0% | 24.142ms | 24.142ms | 24.142ms | 24.142ms | 20.000ms | 44.07 MiB | none | 414.22K/s |
| q37 | date range and group by a URL | 4.000ms | 24.459ms | 24.133ms | 0.0% | 24.133ms | 24.133ms | 24.133ms | 24.133ms | 20.000ms | 45.00 MiB | none | 414.37K/s |
| q38 | date range and group by a title | 4.000ms | 24.390ms | 24.597ms | 0.0% | 24.597ms | 24.597ms | 24.597ms | 24.597ms | 20.000ms | 45.33 MiB | none | 406.55K/s |
| q39 | date range, group by and offset | 4.000ms | 24.002ms | 24.765ms | 0.0% | 24.765ms | 24.765ms | 24.765ms | 24.765ms | 30.000ms | 44.02 MiB | none | 403.80K/s |
| q40 | date range, a case and a wide group by | 5.000ms | 26.103ms | 25.613ms | 0.0% | 25.613ms | 25.613ms | 25.613ms | 25.613ms | 20.000ms | 46.58 MiB | none | 390.43K/s |
| q41 | date range with an IN and a hash | 4.000ms | 24.094ms | 23.447ms | 0.0% | 23.447ms | 23.447ms | 23.447ms | 23.447ms | 20.000ms | 45.22 MiB | none | 426.49K/s |
| q42 | date range and a deep offset | 6.000ms | 27.483ms | 27.379ms | 0.0% | 27.379ms | 27.379ms | 27.379ms | 27.379ms | 30.000ms | 44.45 MiB | none | 365.24K/s |
| q43 | minute buckets over a date range | 3.000ms | 23.337ms | 23.851ms | 0.0% | 23.851ms | 23.851ms | 23.851ms | 23.851ms | 20.000ms | 43.31 MiB | none | 419.27K/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 148.000ms by its own clock and 1.049s by ours, 1.059s cold, 890.000ms of CPU, peak 52.77 MiB, 2.91M/s and 493.10 MiB/s.

Running it cost 609% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.66x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 2.000ms | 56.078ms | 60.451ms | 0.0% | 60.451ms | 60.451ms | 60.451ms | 60.451ms | 350.000ms | 200.30 MiB | none | 165.42K/s |
| q2 | filtered count | 2.000ms | 46.771ms | 47.713ms | 0.0% | 47.713ms | 47.713ms | 47.713ms | 47.713ms | 50.000ms | 201.13 MiB | none | 209.59K/s |
| q3 | three aggregates | 3.000ms | 52.421ms | 49.290ms | 0.0% | 49.290ms | 49.290ms | 49.290ms | 49.290ms | 50.000ms | 203.81 MiB | none | 202.88K/s |
| q4 | average | 3.000ms | 53.835ms | 53.238ms | 0.0% | 53.238ms | 53.238ms | 53.238ms | 53.238ms | 60.000ms | 203.82 MiB | none | 187.84K/s |
| q5 | count distinct, high card | 3.000ms | 57.381ms | 66.703ms | 0.0% | 66.703ms | 66.703ms | 66.703ms | 66.703ms | 340.000ms | 205.22 MiB | none | 149.92K/s |
| q6 | count distinct, strings | 3.000ms | 49.590ms | 48.362ms | 0.0% | 48.362ms | 48.362ms | 48.362ms | 48.362ms | 50.000ms | 203.57 MiB | none | 206.77K/s |
| q7 | min and max of a date | 3.000ms | 48.038ms | 48.979ms | 0.0% | 48.979ms | 48.979ms | 48.979ms | 48.979ms | 50.000ms | 203.13 MiB | none | 204.17K/s |
| q8 | group by, low card | 4.000ms | 50.188ms | 50.323ms | 0.0% | 50.323ms | 50.323ms | 50.323ms | 50.323ms | 50.000ms | 205.82 MiB | none | 198.72K/s |
| q9 | group by and count distinct | 3.000ms | 53.584ms | 52.641ms | 0.0% | 52.641ms | 52.641ms | 52.641ms | 52.641ms | 60.000ms | 205.82 MiB | none | 189.97K/s |
| q10 | group by, several aggregates | 3.000ms | 56.672ms | 55.210ms | 0.0% | 55.210ms | 55.210ms | 55.210ms | 55.210ms | 60.000ms | 206.48 MiB | none | 181.13K/s |
| q11 | group by a string and count distinct | 3.000ms | 51.972ms | 56.927ms | 0.0% | 56.927ms | 56.927ms | 56.927ms | 56.927ms | 70.000ms | 206.68 MiB | none | 175.66K/s |
| q12 | group by two strings and count distinct | 3.000ms | 50.323ms | 55.836ms | 0.0% | 55.836ms | 55.836ms | 55.836ms | 55.836ms | 70.000ms | 206.46 MiB | none | 179.10K/s |
| q13 | group by a string and top k | 3.000ms | 56.930ms | 53.412ms | 0.0% | 53.412ms | 53.412ms | 53.412ms | 53.412ms | 50.000ms | 206.07 MiB | none | 187.22K/s |
| q14 | group by a string and count distinct | 3.000ms | 59.254ms | 54.157ms | 0.0% | 54.157ms | 54.157ms | 54.157ms | 54.157ms | 50.000ms | 207.32 MiB | none | 184.65K/s |
| q15 | group by two columns and top k | 4.000ms | 55.466ms | 54.149ms | 0.0% | 54.149ms | 54.149ms | 54.149ms | 54.149ms | 60.000ms | 207.57 MiB | none | 184.68K/s |
| q16 | group by, very high card | 3.000ms | 55.925ms | 52.471ms | 0.0% | 52.471ms | 52.471ms | 52.471ms | 52.471ms | 50.000ms | 206.65 MiB | none | 190.58K/s |
| q17 | group by two, very high card | 4.000ms | 49.823ms | 58.893ms | 0.0% | 58.893ms | 58.893ms | 58.893ms | 58.893ms | 60.000ms | 208.82 MiB | none | 169.80K/s |
| q18 | group by two, no ordering | 3.000ms | 56.243ms | 55.763ms | 0.0% | 55.763ms | 55.763ms | 55.763ms | 55.763ms | 50.000ms | 207.53 MiB | none | 179.33K/s |
| q19 | group by with an extract | 4.000ms | 59.768ms | 50.474ms | 0.0% | 50.474ms | 50.474ms | 50.474ms | 50.474ms | 50.000ms | 209.38 MiB | none | 198.12K/s |
| q20 | point lookup | 4.000ms | 54.127ms | 55.407ms | 0.0% | 55.407ms | 55.407ms | 55.407ms | 55.407ms | 50.000ms | 203.06 MiB | none | 180.48K/s |
| q21 | substring scan | 4.000ms | 55.660ms | 53.045ms | 0.0% | 53.045ms | 53.045ms | 53.045ms | 53.045ms | 60.000ms | 206.57 MiB | none | 188.52K/s |
| q22 | substring scan and group by | 4.000ms | 53.546ms | 55.629ms | 0.0% | 55.629ms | 55.629ms | 55.629ms | 55.629ms | 60.000ms | 208.32 MiB | none | 179.76K/s |
| q23 | two substring scans and group by | 8.000ms | 52.993ms | 57.123ms | 0.0% | 57.123ms | 57.123ms | 57.123ms | 57.123ms | 90.000ms | 212.51 MiB | none | 175.06K/s |
| q24 | select star and top k | 8.000ms | 83.849ms | 53.254ms | 0.0% | 53.254ms | 53.254ms | 53.254ms | 53.254ms | 60.000ms | 208.80 MiB | none | 187.78K/s |
| q25 | top k by a date | 4.000ms | 49.108ms | 56.452ms | 0.0% | 56.452ms | 56.452ms | 56.452ms | 56.452ms | 110.000ms | 205.76 MiB | none | 177.14K/s |
| q26 | top k by a string | 3.000ms | 53.228ms | 56.529ms | 0.0% | 56.529ms | 56.529ms | 56.529ms | 56.529ms | 60.000ms | 204.75 MiB | none | 176.90K/s |
| q27 | top k by two columns | 4.000ms | 55.620ms | 50.991ms | 0.0% | 50.991ms | 50.991ms | 50.991ms | 50.991ms | 50.000ms | 206.30 MiB | none | 196.11K/s |
| q28 | group by with a string length | 4.000ms | 51.371ms | 66.175ms | 0.0% | 66.175ms | 66.175ms | 66.175ms | 66.175ms | 340.000ms | 207.82 MiB | none | 151.11K/s |
| q29 | group by a regular expression | 8.000ms | 53.428ms | 58.590ms | 0.0% | 58.590ms | 58.590ms | 58.590ms | 58.590ms | 60.000ms | 210.32 MiB | none | 170.68K/s |
| q30 | ninety sums over one column | 8.000ms | 61.221ms | 61.584ms | 0.0% | 61.584ms | 61.584ms | 61.584ms | 61.584ms | 60.000ms | 206.62 MiB | none | 162.38K/s |
| q31 | group by two and several aggregates | 4.000ms | 51.045ms | 51.929ms | 0.0% | 51.929ms | 51.929ms | 51.929ms | 51.929ms | 50.000ms | 206.82 MiB | none | 192.57K/s |
| q32 | group by a high card pair | 4.000ms | 54.481ms | 49.383ms | 0.0% | 49.383ms | 49.383ms | 49.383ms | 49.383ms | 50.000ms | 206.79 MiB | none | 202.50K/s |
| q33 | group by a high card pair, unfiltered | 4.000ms | 55.091ms | 55.609ms | 0.0% | 55.609ms | 55.609ms | 55.609ms | 55.609ms | 60.000ms | 208.57 MiB | none | 179.83K/s |
| q34 | group by a long string | 5.000ms | 50.854ms | 53.222ms | 0.0% | 53.222ms | 53.222ms | 53.222ms | 53.222ms | 50.000ms | 209.05 MiB | none | 187.89K/s |
| q35 | group by a constant and a long string | 4.000ms | 57.873ms | 56.260ms | 0.0% | 56.260ms | 56.260ms | 56.260ms | 56.260ms | 50.000ms | 208.52 MiB | none | 177.75K/s |
| q36 | group by four expressions | 3.000ms | 55.001ms | 55.464ms | 0.0% | 55.464ms | 55.464ms | 55.464ms | 55.464ms | 60.000ms | 207.04 MiB | none | 180.30K/s |
| q37 | date range and group by a URL | 6.000ms | 57.134ms | 60.967ms | 0.0% | 60.967ms | 60.967ms | 60.967ms | 60.967ms | 70.000ms | 211.07 MiB | none | 164.02K/s |
| q38 | date range and group by a title | 6.000ms | 59.553ms | 55.418ms | 0.0% | 55.418ms | 55.418ms | 55.418ms | 55.418ms | 50.000ms | 211.56 MiB | none | 180.45K/s |
| q39 | date range, group by and offset | 6.000ms | 52.246ms | 53.052ms | 0.0% | 53.052ms | 53.052ms | 53.052ms | 53.052ms | 110.000ms | 210.82 MiB | none | 188.49K/s |
| q40 | date range, a case and a wide group by | 7.000ms | 51.864ms | 52.361ms | 0.0% | 52.361ms | 52.361ms | 52.361ms | 52.361ms | 60.000ms | 212.07 MiB | none | 190.98K/s |
| q41 | date range with an IN and a hash | 6.000ms | 54.093ms | 63.596ms | 0.0% | 63.596ms | 63.596ms | 63.596ms | 63.596ms | 80.000ms | 209.57 MiB | none | 157.24K/s |
| q42 | date range and a deep offset | 5.000ms | 51.153ms | 57.705ms | 0.0% | 57.705ms | 57.705ms | 57.705ms | 57.705ms | 140.000ms | 208.79 MiB | none | 173.30K/s |
| q43 | minute buckets over a date range | 5.000ms | 52.143ms | 56.372ms | 0.0% | 56.372ms | 56.372ms | 56.372ms | 56.372ms | 60.000ms | 208.52 MiB | none | 177.39K/s |

clickhouse-local 26.9.1.1162 over 43 of 43 queries. Total 183.000ms by its own clock and 2.371s by ours, 2.347s cold, 3.520s of CPU, peak 212.51 MiB, 2.35M/s and 398.79 MiB/s.

Running it cost 1196% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.40x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 13.255ms | 13.796ms | 0.0% | 13.796ms | 13.796ms | 13.796ms | 13.796ms | 10.000ms | 80.35 MiB | none | 724.85K/s |
| q2 | filtered count | 4.000ms | 17.749ms | 15.404ms | 0.0% | 15.404ms | 15.404ms | 15.404ms | 15.404ms | 10.000ms | 106.59 MiB | none | 649.18K/s |
| q3 | three aggregates | 3.000ms | 13.706ms | 13.740ms | 0.0% | 13.740ms | 13.740ms | 13.740ms | 13.740ms | 10.000ms | 103.93 MiB | none | 727.80K/s |
| q4 | average | 2.000ms | 12.704ms | 14.361ms | 0.0% | 14.361ms | 14.361ms | 14.361ms | 14.361ms | 10.000ms | 92.35 MiB | none | 696.33K/s |
| q5 | count distinct, high card | 6.000ms | 16.704ms | 16.481ms | 0.0% | 16.481ms | 16.481ms | 16.481ms | 16.481ms | 30.000ms | 135.57 MiB | none | 606.76K/s |
| q6 | count distinct, strings | 7.000ms | 17.598ms | 18.124ms | 0.0% | 18.124ms | 18.124ms | 18.124ms | 18.124ms | 40.000ms | 154.35 MiB | none | 551.75K/s |
| q7 | min and max of a date | 1.000ms | 12.028ms | 11.375ms | 0.0% | 11.375ms | 11.375ms | 11.375ms | 11.375ms | 0.000us | 80.00 MiB | none | 879.12K/s |
| q8 | group by, low card | 5.000ms | 15.311ms | 16.112ms | 0.0% | 16.112ms | 16.112ms | 16.112ms | 16.112ms | 30.000ms | 122.57 MiB | none | 620.66K/s |
| q9 | group by and count distinct | 8.000ms | 20.332ms | 19.336ms | 0.0% | 19.336ms | 19.336ms | 19.336ms | 19.336ms | 70.000ms | 150.05 MiB | none | 517.17K/s |
| q10 | group by, several aggregates | 8.000ms | 19.532ms | 18.222ms | 0.0% | 18.222ms | 18.222ms | 18.222ms | 18.222ms | 60.000ms | 149.79 MiB | none | 548.79K/s |
| q11 | group by a string and count distinct | 9.000ms | 19.787ms | 19.814ms | 0.0% | 19.814ms | 19.814ms | 19.814ms | 19.814ms | 70.000ms | 170.75 MiB | none | 504.69K/s |
| q12 | group by two strings and count distinct | 9.000ms | 20.009ms | 20.596ms | 0.0% | 20.596ms | 20.596ms | 20.596ms | 20.596ms | 80.000ms | 180.55 MiB | none | 485.53K/s |
| q13 | group by a string and top k | 8.000ms | 19.031ms | 19.399ms | 0.0% | 19.399ms | 19.399ms | 19.399ms | 19.399ms | 100.000ms | 167.39 MiB | none | 515.49K/s |
| q14 | group by a string and count distinct | 12.000ms | 21.056ms | 23.529ms | 0.0% | 23.529ms | 23.529ms | 23.529ms | 23.529ms | 160.000ms | 189.13 MiB | none | 425.01K/s |
| q15 | group by two columns and top k | 9.000ms | 22.217ms | 21.565ms | 0.0% | 21.565ms | 21.565ms | 21.565ms | 21.565ms | 140.000ms | 170.64 MiB | none | 463.71K/s |
| q16 | group by, very high card | 6.000ms | 17.785ms | 17.350ms | 0.0% | 17.350ms | 17.350ms | 17.350ms | 17.350ms | 40.000ms | 133.04 MiB | none | 576.37K/s |
| q17 | group by two, very high card | 9.000ms | 18.355ms | 20.110ms | 0.0% | 20.110ms | 20.110ms | 20.110ms | 20.110ms | 80.000ms | 155.17 MiB | none | 497.27K/s |
| q18 | group by two, no ordering | 9.000ms | 17.890ms | 19.745ms | 0.0% | 19.745ms | 19.745ms | 19.745ms | 19.745ms | 90.000ms | 154.33 MiB | none | 506.46K/s |
| q19 | group by with an extract | 7.000ms | 19.023ms | 18.488ms | 0.0% | 18.488ms | 18.488ms | 18.488ms | 18.488ms | 30.000ms | 144.11 MiB | none | 540.89K/s |
| q20 | point lookup | 3.000ms | 15.276ms | 14.687ms | 0.0% | 14.687ms | 14.687ms | 14.687ms | 14.687ms | 30.000ms | 115.64 MiB | none | 680.87K/s |
| q21 | substring scan | 5.000ms | 15.223ms | 15.725ms | 0.0% | 15.725ms | 15.725ms | 15.725ms | 15.725ms | 50.000ms | 122.75 MiB | none | 635.93K/s |
| q22 | substring scan and group by | 7.000ms | 18.192ms | 18.011ms | 0.0% | 18.011ms | 18.011ms | 18.011ms | 18.011ms | 40.000ms | 143.84 MiB | none | 555.22K/s |
| q23 | two substring scans and group by | 10.000ms | 19.384ms | 21.217ms | 0.0% | 21.217ms | 21.217ms | 21.217ms | 21.217ms | 50.000ms | 160.14 MiB | none | 471.32K/s |
| q24 | select star and top k | 13.000ms | 26.525ms | 24.448ms | 0.0% | 24.448ms | 24.448ms | 24.448ms | 24.448ms | 20.000ms | 148.52 MiB | none | 409.03K/s |
| q25 | top k by a date | 4.000ms | 15.732ms | 15.520ms | 0.0% | 15.520ms | 15.520ms | 15.520ms | 15.520ms | 10.000ms | 124.67 MiB | none | 644.33K/s |
| q26 | top k by a string | 4.000ms | 14.921ms | 15.218ms | 0.0% | 15.218ms | 15.218ms | 15.218ms | 15.218ms | 10.000ms | 119.52 MiB | none | 657.12K/s |
| q27 | top k by two columns | 5.000ms | 14.912ms | 15.260ms | 0.0% | 15.260ms | 15.260ms | 15.260ms | 15.260ms | 20.000ms | 112.52 MiB | none | 655.31K/s |
| q28 | group by with a string length | 7.000ms | 18.341ms | 18.012ms | 0.0% | 18.012ms | 18.012ms | 18.012ms | 18.012ms | 20.000ms | 148.43 MiB | none | 555.19K/s |
| q29 | group by a regular expression | 13.000ms | 22.394ms | 24.888ms | 0.0% | 24.888ms | 24.888ms | 24.888ms | 24.888ms | 30.000ms | 171.05 MiB | none | 401.80K/s |
| q30 | ninety sums over one column | 11.000ms | 22.564ms | 22.533ms | 0.0% | 22.533ms | 22.533ms | 22.533ms | 22.533ms | 20.000ms | 105.35 MiB | none | 443.79K/s |
| q31 | group by two and several aggregates | 7.000ms | 19.732ms | 18.279ms | 0.0% | 18.279ms | 18.279ms | 18.279ms | 18.279ms | 30.000ms | 152.23 MiB | none | 547.08K/s |
| q32 | group by a high card pair | 9.000ms | 18.698ms | 20.884ms | 0.0% | 20.884ms | 20.884ms | 20.884ms | 20.884ms | 40.000ms | 158.12 MiB | none | 478.84K/s |
| q33 | group by a high card pair, unfiltered | 6.000ms | 18.180ms | 17.163ms | 0.0% | 17.163ms | 17.163ms | 17.163ms | 17.163ms | 40.000ms | 150.17 MiB | none | 582.65K/s |
| q34 | group by a long string | 10.000ms | 20.830ms | 21.307ms | 0.0% | 21.307ms | 21.307ms | 21.307ms | 21.307ms | 80.000ms | 177.98 MiB | none | 469.33K/s |
| q35 | group by a constant and a long string | 14.000ms | 20.800ms | 25.176ms | 0.0% | 25.176ms | 25.176ms | 25.176ms | 25.176ms | 110.000ms | 184.49 MiB | none | 397.20K/s |
| q36 | group by four expressions | 7.000ms | 17.834ms | 17.655ms | 0.0% | 17.655ms | 17.655ms | 17.655ms | 17.655ms | 50.000ms | 144.96 MiB | none | 566.41K/s |
| q37 | date range and group by a URL | 10.000ms | 22.102ms | 20.809ms | 0.0% | 20.809ms | 20.809ms | 20.809ms | 20.809ms | 90.000ms | 168.03 MiB | none | 480.56K/s |
| q38 | date range and group by a title | 12.000ms | 22.242ms | 23.170ms | 0.0% | 23.170ms | 23.170ms | 23.170ms | 23.170ms | 140.000ms | 171.68 MiB | none | 431.59K/s |
| q39 | date range, group by and offset | 11.000ms | 21.223ms | 22.702ms | 0.0% | 22.702ms | 22.702ms | 22.702ms | 22.702ms | 170.000ms | 154.16 MiB | none | 440.49K/s |
| q40 | date range, a case and a wide group by | 12.000ms | 23.398ms | 22.915ms | 0.0% | 22.915ms | 22.915ms | 22.915ms | 22.915ms | 140.000ms | 154.13 MiB | none | 436.40K/s |
| q41 | date range with an IN and a hash | 7.000ms | 17.269ms | 18.691ms | 0.0% | 18.691ms | 18.691ms | 18.691ms | 18.691ms | 70.000ms | 141.20 MiB | none | 535.02K/s |
| q42 | date range and a deep offset | 8.000ms | 20.081ms | 19.666ms | 0.0% | 19.666ms | 19.666ms | 19.666ms | 19.666ms | 90.000ms | 144.35 MiB | none | 508.49K/s |
| q43 | minute buckets over a date range | 9.000ms | 19.824ms | 21.000ms | 0.0% | 21.000ms | 21.000ms | 21.000ms | 21.000ms | 100.000ms | 139.41 MiB | none | 476.19K/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 327.000ms by its own clock and 812.483ms by ours, 799.749ms cold, 2.510s of CPU, peak 189.13 MiB, 1.31M/s and 223.18 MiB/s.

Running it cost 148% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.21x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 5.535ms | 93.286ms | 97.723ms | 0.0% | 97.723ms | 97.723ms | 97.723ms | 97.723ms | 320.000ms | 62.00 MiB | none | 102.33K/s |
| q2 | filtered count | 5.809ms | 91.435ms | 91.969ms | 0.0% | 91.969ms | 91.969ms | 91.969ms | 91.969ms | 210.000ms | 66.53 MiB | none | 108.73K/s |
| q3 | three aggregates | 6.218ms | 100.480ms | 92.946ms | 0.0% | 92.946ms | 92.946ms | 92.946ms | 92.946ms | 260.000ms | 65.47 MiB | none | 107.59K/s |
| q4 | average | 5.362ms | 93.069ms | 105.104ms | 0.0% | 105.104ms | 105.104ms | 105.104ms | 105.104ms | 430.000ms | 62.42 MiB | none | 95.14K/s |
| q5 | count distinct, high card | 12.622ms | 99.048ms | 98.157ms | 0.0% | 98.157ms | 98.157ms | 98.157ms | 98.157ms | 120.000ms | 73.48 MiB | none | 101.88K/s |
| q6 | count distinct, strings | 13.143ms | 97.124ms | 98.102ms | 0.0% | 98.102ms | 98.102ms | 98.102ms | 98.102ms | 140.000ms | 74.48 MiB | none | 101.93K/s |
| q7 | min and max of a date | 7.830ms | 90.445ms | 93.366ms | 0.0% | 93.366ms | 93.366ms | 93.366ms | 93.366ms | 120.000ms | 62.95 MiB | none | 107.11K/s |
| q8 | group by, low card | 14.426ms | 97.985ms | 98.391ms | 0.0% | 98.391ms | 98.391ms | 98.391ms | 98.391ms | 150.000ms | 75.42 MiB | none | 101.64K/s |
| q9 | group by and count distinct | 20.957ms | 109.354ms | 105.818ms | 0.0% | 105.818ms | 105.818ms | 105.818ms | 105.818ms | 160.000ms | 83.05 MiB | none | 94.50K/s |
| q10 | group by, several aggregates | 25.581ms | 111.512ms | 111.354ms | 0.0% | 111.354ms | 111.354ms | 111.354ms | 111.354ms | 160.000ms | 86.80 MiB | none | 89.80K/s |
| q11 | group by a string and count distinct | 21.039ms | 108.172ms | 107.511ms | 0.0% | 107.511ms | 107.511ms | 107.511ms | 107.511ms | 150.000ms | 81.26 MiB | none | 93.01K/s |
| q12 | group by two strings and count distinct | 22.534ms | 108.509ms | 107.730ms | 0.0% | 107.730ms | 107.730ms | 107.730ms | 107.730ms | 160.000ms | 83.41 MiB | none | 92.82K/s |
| q13 | group by a string and top k | 15.600ms | 103.741ms | 102.432ms | 0.0% | 102.432ms | 102.432ms | 102.432ms | 102.432ms | 160.000ms | 76.45 MiB | none | 97.63K/s |
| q14 | group by a string and count distinct | 22.475ms | 106.959ms | 108.334ms | 0.0% | 108.334ms | 108.334ms | 108.334ms | 108.334ms | 180.000ms | 83.50 MiB | none | 92.31K/s |
| q15 | group by two columns and top k | 19.253ms | 102.070ms | 105.110ms | 0.0% | 105.110ms | 105.110ms | 105.110ms | 105.110ms | 200.000ms | 78.01 MiB | none | 95.14K/s |
| q16 | group by, very high card | 19.151ms | 113.155ms | 105.973ms | 0.0% | 105.973ms | 105.973ms | 105.973ms | 105.973ms | 140.000ms | 75.21 MiB | none | 94.36K/s |
| q17 | group by two, very high card | 24.577ms | 112.452ms | 113.478ms | 0.0% | 113.478ms | 113.478ms | 113.478ms | 113.478ms | 210.000ms | 78.79 MiB | none | 88.12K/s |
| q18 | group by two, no ordering | 13.517ms | 115.827ms | 98.433ms | 0.0% | 98.433ms | 98.433ms | 98.433ms | 98.433ms | 140.000ms | 76.19 MiB | none | 101.59K/s |
| q19 | group by with an extract | 27.215ms | 116.014ms | 129.912ms | 0.0% | 129.912ms | 129.912ms | 129.912ms | 129.912ms | 620.000ms | 80.41 MiB | none | 76.98K/s |
| q20 | point lookup | 6.334ms | 99.146ms | 96.190ms | 0.0% | 96.190ms | 96.190ms | 96.190ms | 96.190ms | 310.000ms | 64.69 MiB | none | 103.96K/s |
| q21 | substring scan | 9.119ms | 106.945ms | 96.229ms | 0.0% | 96.229ms | 96.229ms | 96.229ms | 96.229ms | 170.000ms | 69.26 MiB | none | 103.92K/s |
| q22 | substring scan and group by | 18.870ms | 107.064ms | 114.569ms | 0.0% | 114.569ms | 114.569ms | 114.569ms | 114.569ms | 340.000ms | 78.67 MiB | none | 87.28K/s |
| q23 | two substring scans and group by | 22.718ms | 110.094ms | 109.746ms | 0.0% | 109.746ms | 109.746ms | 109.746ms | 109.746ms | 160.000ms | 87.39 MiB | none | 91.12K/s |
| q24 | select star and top k | 12.527ms | 102.257ms | 102.785ms | 0.0% | 102.785ms | 102.785ms | 102.785ms | 102.785ms | 180.000ms | 75.29 MiB | none | 97.29K/s |
| q25 | top k by a date | 12.430ms | 99.434ms | 103.185ms | 0.0% | 103.185ms | 103.185ms | 103.185ms | 103.185ms | 290.000ms | 74.01 MiB | none | 96.91K/s |
| q26 | top k by a string | 14.245ms | 97.941ms | 107.659ms | 0.0% | 107.659ms | 107.659ms | 107.659ms | 107.659ms | 320.000ms | 71.98 MiB | none | 92.89K/s |
| q27 | top k by two columns | 11.878ms | 109.514ms | 107.853ms | 0.0% | 107.853ms | 107.853ms | 107.853ms | 107.853ms | 280.000ms | 72.95 MiB | none | 92.72K/s |
| q30 | ninety sums over one column | 12.069ms | 113.025ms | 97.671ms | 0.0% | 97.671ms | 97.671ms | 97.671ms | 97.671ms | 130.000ms | 67.06 MiB | none | 102.38K/s |
| q31 | group by two and several aggregates | 13.490ms | 107.077ms | 96.307ms | 0.0% | 96.307ms | 96.307ms | 96.307ms | 96.307ms | 110.000ms | 80.20 MiB | none | 103.83K/s |
| q32 | group by a high card pair | 16.454ms | 111.713ms | 104.280ms | 0.0% | 104.280ms | 104.280ms | 104.280ms | 104.280ms | 170.000ms | 78.96 MiB | none | 95.90K/s |
| q33 | group by a high card pair, unfiltered | 17.372ms | 106.726ms | 103.068ms | 0.0% | 103.068ms | 103.068ms | 103.068ms | 103.068ms | 140.000ms | 79.41 MiB | none | 97.02K/s |
| q34 | group by a long string | 19.757ms | 107.401ms | 107.980ms | 0.0% | 107.980ms | 107.980ms | 107.980ms | 107.980ms | 160.000ms | 80.35 MiB | none | 92.61K/s |
| q35 | group by a constant and a long string | 20.777ms | 108.481ms | 120.945ms | 0.0% | 120.945ms | 120.945ms | 120.945ms | 120.945ms | 550.000ms | 82.08 MiB | none | 82.68K/s |
| q37 | date range and group by a URL | 22.872ms | 116.677ms | 115.244ms | 0.0% | 115.244ms | 115.244ms | 115.244ms | 115.244ms | 350.000ms | 82.95 MiB | none | 86.77K/s |
| q38 | date range and group by a title | 19.769ms | 106.387ms | 107.442ms | 0.0% | 107.442ms | 107.442ms | 107.442ms | 107.442ms | 160.000ms | 81.58 MiB | none | 93.07K/s |
| q39 | date range, group by and offset | 15.774ms | 105.912ms | 118.457ms | 0.0% | 118.457ms | 118.457ms | 118.457ms | 118.457ms | 550.000ms | 80.31 MiB | none | 84.42K/s |
| q40 | date range, a case and a wide group by | 19.639ms | 108.331ms | 107.764ms | 0.0% | 107.764ms | 107.764ms | 107.764ms | 107.764ms | 190.000ms | 81.58 MiB | none | 92.80K/s |
| q41 | date range with an IN and a hash | 15.653ms | 115.673ms | 100.650ms | 0.0% | 100.650ms | 100.650ms | 100.650ms | 100.650ms | 130.000ms | 80.50 MiB | none | 99.35K/s |
| q42 | date range and a deep offset | 16.174ms | 103.166ms | 104.438ms | 0.0% | 104.438ms | 104.438ms | 104.438ms | 104.438ms | 200.000ms | 80.00 MiB | none | 95.75K/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 620.765ms by its own clock and 4.094s by ours, 4.114s cold, 8.920s of CPU, peak 87.39 MiB, 628.26K/s and 106.63 MiB/s.

Running it cost 560% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.41x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-server in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 29.997ms | 30.534ms | 0.0% | 30.534ms | 30.534ms | 30.534ms | 30.534ms | not read | not read | not read | 327.50K/s |
| q2 | filtered count | 1.000ms | 29.549ms | 44.988ms | 0.0% | 44.988ms | 44.988ms | 44.988ms | 44.988ms | not read | not read | not read | 222.28K/s |
| q3 | three aggregates | 2.000ms | 31.213ms | 30.715ms | 0.0% | 30.715ms | 30.715ms | 30.715ms | 30.715ms | not read | not read | not read | 325.57K/s |
| q4 | average | 1.000ms | 29.980ms | 31.106ms | 0.0% | 31.106ms | 31.106ms | 31.106ms | 31.106ms | not read | not read | not read | 321.48K/s |
| q5 | count distinct, high card | 2.000ms | 31.636ms | 31.318ms | 0.0% | 31.318ms | 31.318ms | 31.318ms | 31.318ms | not read | not read | not read | 319.31K/s |
| q6 | count distinct, strings | 2.000ms | 30.634ms | 30.645ms | 0.0% | 30.645ms | 30.645ms | 30.645ms | 30.645ms | not read | not read | not read | 326.32K/s |
| q7 | min and max of a date | 1.000ms | 31.108ms | 30.896ms | 0.0% | 30.896ms | 30.896ms | 30.896ms | 30.896ms | not read | not read | not read | 323.67K/s |
| q8 | group by, low card | 2.000ms | 30.647ms | 31.603ms | 0.0% | 31.603ms | 31.603ms | 31.603ms | 31.603ms | not read | not read | not read | 316.43K/s |
| q9 | group by and count distinct | 2.000ms | 31.128ms | 30.874ms | 0.0% | 30.874ms | 30.874ms | 30.874ms | 30.874ms | not read | not read | not read | 323.90K/s |
| q10 | group by, several aggregates | 16.000ms | 31.621ms | 45.512ms | 0.0% | 45.512ms | 45.512ms | 45.512ms | 45.512ms | not read | not read | not read | 219.72K/s |
| q11 | group by a string and count distinct | 2.000ms | 30.718ms | 30.232ms | 0.0% | 30.232ms | 30.232ms | 30.232ms | 30.232ms | not read | not read | not read | 330.78K/s |
| q12 | group by two strings and count distinct | 2.000ms | 31.835ms | 31.322ms | 0.0% | 31.322ms | 31.322ms | 31.322ms | 31.322ms | not read | not read | not read | 319.26K/s |
| q13 | group by a string and top k | 2.000ms | 32.243ms | 29.733ms | 0.0% | 29.733ms | 29.733ms | 29.733ms | 29.733ms | not read | not read | not read | 336.33K/s |
| q14 | group by a string and count distinct | 2.000ms | 32.057ms | 31.952ms | 0.0% | 31.952ms | 31.952ms | 31.952ms | 31.952ms | not read | not read | not read | 312.97K/s |
| q15 | group by two columns and top k | 2.000ms | 30.833ms | 31.749ms | 0.0% | 31.749ms | 31.749ms | 31.749ms | 31.749ms | not read | not read | not read | 314.97K/s |
| q16 | group by, very high card | 2.000ms | 30.514ms | 30.629ms | 0.0% | 30.629ms | 30.629ms | 30.629ms | 30.629ms | not read | not read | not read | 326.49K/s |
| q17 | group by two, very high card | 3.000ms | 31.472ms | 31.156ms | 0.0% | 31.156ms | 31.156ms | 31.156ms | 31.156ms | not read | not read | not read | 320.97K/s |
| q18 | group by two, no ordering | 2.000ms | 31.900ms | 30.509ms | 0.0% | 30.509ms | 30.509ms | 30.509ms | 30.509ms | not read | not read | not read | 327.77K/s |
| q19 | group by with an extract | 3.000ms | 32.075ms | 33.166ms | 0.0% | 33.166ms | 33.166ms | 33.166ms | 33.166ms | not read | not read | not read | 301.51K/s |
| q20 | point lookup | 1.000ms | 32.022ms | 29.713ms | 0.0% | 29.713ms | 29.713ms | 29.713ms | 29.713ms | not read | not read | not read | 336.55K/s |
| q21 | substring scan | 1.000ms | 31.474ms | 30.589ms | 0.0% | 30.589ms | 30.589ms | 30.589ms | 30.589ms | not read | not read | not read | 326.91K/s |
| q22 | substring scan and group by | 3.000ms | 31.724ms | 32.794ms | 0.0% | 32.794ms | 32.794ms | 32.794ms | 32.794ms | not read | not read | not read | 304.93K/s |
| q23 | two substring scans and group by | 4.000ms | 33.668ms | 34.770ms | 0.0% | 34.770ms | 34.770ms | 34.770ms | 34.770ms | not read | not read | not read | 287.60K/s |
| q24 | select star and top k | 4.000ms | 33.913ms | 34.211ms | 0.0% | 34.211ms | 34.211ms | 34.211ms | 34.211ms | not read | not read | not read | 292.30K/s |
| q25 | top k by a date | 2.000ms | 35.268ms | 30.671ms | 0.0% | 30.671ms | 30.671ms | 30.671ms | 30.671ms | not read | not read | not read | 326.04K/s |
| q26 | top k by a string | 2.000ms | 31.399ms | 30.696ms | 0.0% | 30.696ms | 30.696ms | 30.696ms | 30.696ms | not read | not read | not read | 325.78K/s |
| q27 | top k by two columns | 2.000ms | 30.333ms | 30.392ms | 0.0% | 30.392ms | 30.392ms | 30.392ms | 30.392ms | not read | not read | not read | 329.03K/s |
| q28 | group by with a string length | 3.000ms | 32.441ms | 32.213ms | 0.0% | 32.213ms | 32.213ms | 32.213ms | 32.213ms | not read | not read | not read | 310.43K/s |
| q29 | group by a regular expression | 22.000ms | 36.230ms | 53.140ms | 0.0% | 53.140ms | 53.140ms | 53.140ms | 53.140ms | not read | not read | not read | 188.18K/s |
| q30 | ninety sums over one column | 6.000ms | 37.712ms | 34.827ms | 0.0% | 34.827ms | 34.827ms | 34.827ms | 34.827ms | not read | not read | not read | 287.13K/s |
| q31 | group by two and several aggregates | 2.000ms | 31.851ms | 30.902ms | 0.0% | 30.902ms | 30.902ms | 30.902ms | 30.902ms | not read | not read | not read | 323.60K/s |
| q32 | group by a high card pair | 15.000ms | 31.331ms | 46.302ms | 0.0% | 46.302ms | 46.302ms | 46.302ms | 46.302ms | not read | not read | not read | 215.97K/s |
| q33 | group by a high card pair, unfiltered | 2.000ms | 34.816ms | 31.746ms | 0.0% | 31.746ms | 31.746ms | 31.746ms | 31.746ms | not read | not read | not read | 315.00K/s |
| q34 | group by a long string | 3.000ms | 31.703ms | 31.757ms | 0.0% | 31.757ms | 31.757ms | 31.757ms | 31.757ms | not read | not read | not read | 314.89K/s |
| q35 | group by a constant and a long string | 3.000ms | 31.677ms | 32.692ms | 0.0% | 32.692ms | 32.692ms | 32.692ms | 32.692ms | not read | not read | not read | 305.89K/s |
| q36 | group by four expressions | 2.000ms | 31.103ms | 32.148ms | 0.0% | 32.148ms | 32.148ms | 32.148ms | 32.148ms | not read | not read | not read | 311.06K/s |
| q37 | date range and group by a URL | 3.000ms | 32.494ms | 33.151ms | 0.0% | 33.151ms | 33.151ms | 33.151ms | 33.151ms | not read | not read | not read | 301.65K/s |
| q38 | date range and group by a title | 3.000ms | 33.223ms | 33.627ms | 0.0% | 33.627ms | 33.627ms | 33.627ms | 33.627ms | not read | not read | not read | 297.38K/s |
| q39 | date range, group by and offset | 3.000ms | 34.062ms | 33.611ms | 0.0% | 33.611ms | 33.611ms | 33.611ms | 33.611ms | not read | not read | not read | 297.52K/s |
| q40 | date range, a case and a wide group by | 4.000ms | 33.625ms | 34.244ms | 0.0% | 34.244ms | 34.244ms | 34.244ms | 34.244ms | not read | not read | not read | 292.02K/s |
| q41 | date range with an IN and a hash | 3.000ms | 34.625ms | 33.399ms | 0.0% | 33.399ms | 33.399ms | 33.399ms | 33.399ms | not read | not read | not read | 299.41K/s |
| q42 | date range and a deep offset | 3.000ms | 33.644ms | 36.221ms | 0.0% | 36.221ms | 36.221ms | 36.221ms | 36.221ms | not read | not read | not read | 276.08K/s |
| q43 | minute buckets over a date range | 3.000ms | 32.370ms | 32.875ms | 0.0% | 32.875ms | 32.875ms | 32.875ms | 32.875ms | not read | not read | not read | 304.18K/s |

clickhouse-server 26.9.1.1162 over 43 of 43 queries. Total 149.000ms by its own clock and 1.435s by ours, 1.384s cold, no reading of CPU, peak not read, 2.89M/s and 489.79 MiB/s.

Running it cost 863% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.79x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 0.000us | 1.543ms | 1.359ms | 0.0% | 1.359ms | 1.359ms | 1.359ms | 1.359ms | 0.000us | 4.26 MiB | none | 7.36M/s |
| q2 | filtered count | 0.000us | 1.469ms | 1.477ms | 0.0% | 1.477ms | 1.477ms | 1.477ms | 1.477ms | 0.000us | 4.50 MiB | none | 6.77M/s |
| q3 | three aggregates | 0.000us | 1.557ms | 1.526ms | 0.0% | 1.526ms | 1.526ms | 1.526ms | 1.526ms | 0.000us | 4.74 MiB | none | 6.55M/s |
| q4 | average | 0.000us | 1.530ms | 1.452ms | 0.0% | 1.452ms | 1.452ms | 1.452ms | 1.452ms | 0.000us | 4.49 MiB | none | 6.89M/s |
| q5 | count distinct, high card | 1.000ms | 2.352ms | 2.411ms | 0.0% | 2.411ms | 2.411ms | 2.411ms | 2.411ms | 0.000us | 5.49 MiB | none | 4.15M/s |
| q6 | count distinct, strings | 1.000ms | 2.210ms | 2.251ms | 0.0% | 2.251ms | 2.251ms | 2.251ms | 2.251ms | 0.000us | 4.64 MiB | none | 4.44M/s |
| q7 | min and max of a date | 0.000us | 1.456ms | 1.433ms | 0.0% | 1.433ms | 1.433ms | 1.433ms | 1.433ms | 0.000us | 4.50 MiB | none | 6.98M/s |
| q8 | group by, low card | 0.000us | 1.470ms | 1.392ms | 0.0% | 1.392ms | 1.392ms | 1.392ms | 1.392ms | 0.000us | 4.46 MiB | none | 7.18M/s |
| q9 | group by and count distinct | 2.000ms | 3.030ms | 2.966ms | 0.0% | 2.966ms | 2.966ms | 2.966ms | 2.966ms | 0.000us | 5.75 MiB | none | 3.37M/s |
| q10 | group by, several aggregates | 2.000ms | 3.390ms | 3.275ms | 0.0% | 3.275ms | 3.275ms | 3.275ms | 3.275ms | 0.000us | 6.21 MiB | none | 3.05M/s |
| q11 | group by a string and count distinct | 1.000ms | 1.658ms | 1.676ms | 0.0% | 1.676ms | 1.676ms | 1.676ms | 1.676ms | 0.000us | 4.75 MiB | none | 5.97M/s |
| q12 | group by two strings and count distinct | 1.000ms | 1.666ms | 1.714ms | 0.0% | 1.714ms | 1.714ms | 1.714ms | 1.714ms | 0.000us | 4.66 MiB | none | 5.83M/s |
| q13 | group by a string and top k | 1.000ms | 2.408ms | 2.607ms | 0.0% | 2.607ms | 2.607ms | 2.607ms | 2.607ms | 0.000us | 5.21 MiB | none | 3.84M/s |
| q14 | group by a string and count distinct | 2.000ms | 2.698ms | 2.625ms | 0.0% | 2.625ms | 2.625ms | 2.625ms | 2.625ms | 0.000us | 5.49 MiB | none | 3.81M/s |
| q15 | group by two columns and top k | 2.000ms | 2.918ms | 2.722ms | 0.0% | 2.722ms | 2.722ms | 2.722ms | 2.722ms | 0.000us | 5.51 MiB | none | 3.67M/s |
| q16 | group by, very high card | 2.000ms | 3.123ms | 3.162ms | 0.0% | 3.162ms | 3.162ms | 3.162ms | 3.162ms | 0.000us | 6.23 MiB | none | 3.16M/s |
| q17 | group by two, very high card | 3.000ms | 4.406ms | 4.617ms | 0.0% | 4.617ms | 4.617ms | 4.617ms | 4.617ms | 0.000us | 7.23 MiB | none | 2.17M/s |
| q18 | group by two, no ordering | 3.000ms | 3.528ms | 3.620ms | 0.0% | 3.620ms | 3.620ms | 3.620ms | 3.620ms | 0.000us | 7.26 MiB | none | 2.76M/s |
| q20 | point lookup | 0.000us | 1.480ms | 1.457ms | 0.0% | 1.457ms | 1.457ms | 1.457ms | 1.457ms | 0.000us | 4.72 MiB | none | 6.86M/s |
| q21 | substring scan | 4.000ms | 5.133ms | 5.140ms | 0.0% | 5.140ms | 5.140ms | 5.140ms | 5.140ms | 0.000us | 6.24 MiB | none | 1.95M/s |
| q22 | substring scan and group by | 4.000ms | 5.441ms | 5.530ms | 0.0% | 5.530ms | 5.530ms | 5.530ms | 5.530ms | 0.000us | 6.47 MiB | none | 1.81M/s |
| q23 | two substring scans and group by | 9.000ms | 10.723ms | 10.360ms | 0.0% | 10.360ms | 10.360ms | 10.360ms | 10.360ms | 0.000us | 8.89 MiB | none | 965.25K/s |
| q24 | select star and top k | 21.000ms | 22.307ms | 22.172ms | 0.0% | 22.172ms | 22.172ms | 22.172ms | 22.172ms | 20.000ms | 19.83 MiB | none | 451.02K/s |
| q25 | top k by a date | 1.000ms | 2.594ms | 2.520ms | 0.0% | 2.520ms | 2.520ms | 2.520ms | 2.520ms | 0.000us | 5.19 MiB | none | 3.97M/s |
| q26 | top k by a string | 1.000ms | 2.322ms | 2.252ms | 0.0% | 2.252ms | 2.252ms | 2.252ms | 2.252ms | 0.000us | 4.70 MiB | none | 4.44M/s |
| q27 | top k by two columns | 1.000ms | 2.628ms | 2.579ms | 0.0% | 2.579ms | 2.579ms | 2.579ms | 2.579ms | 0.000us | 5.19 MiB | none | 3.88M/s |
| q28 | group by with a string length | 4.000ms | 5.178ms | 5.174ms | 0.0% | 5.174ms | 5.174ms | 5.174ms | 5.174ms | 0.000us | 6.45 MiB | none | 1.93M/s |
| q29 | group by a regular expression | 9.000ms | 10.432ms | 10.102ms | 0.0% | 10.102ms | 10.102ms | 10.102ms | 10.102ms | 0.000us | 6.65 MiB | none | 989.90K/s |
| q30 | ninety sums over one column | 3.000ms | 3.627ms | 3.690ms | 0.0% | 3.690ms | 3.690ms | 3.690ms | 3.690ms | 0.000us | 5.15 MiB | none | 2.71M/s |
| q31 | group by two and several aggregates | 2.000ms | 2.687ms | 2.753ms | 0.0% | 2.753ms | 2.753ms | 2.753ms | 2.753ms | 0.000us | 5.73 MiB | none | 3.63M/s |
| q32 | group by a high card pair | 2.000ms | 2.709ms | 2.829ms | 0.0% | 2.829ms | 2.829ms | 2.829ms | 2.829ms | 0.000us | 5.76 MiB | none | 3.53M/s |
| q34 | group by a long string | 6.000ms | 7.522ms | 7.424ms | 0.0% | 7.424ms | 7.424ms | 7.424ms | 7.424ms | 0.000us | 7.83 MiB | none | 1.35M/s |
| q35 | group by a constant and a long string | 7.000ms | 8.059ms | 7.731ms | 0.0% | 7.731ms | 7.731ms | 7.731ms | 7.731ms | 0.000us | 8.37 MiB | none | 1.29M/s |
| q36 | group by four expressions | 4.000ms | 4.607ms | 4.833ms | 0.0% | 4.833ms | 4.833ms | 4.833ms | 4.833ms | 0.000us | 8.46 MiB | none | 2.07M/s |
| q37 | date range and group by a URL | 4.000ms | 5.056ms | 5.065ms | 0.0% | 5.065ms | 5.065ms | 5.065ms | 5.065ms | 0.000us | 6.65 MiB | none | 1.97M/s |
| q38 | date range and group by a title | 4.000ms | 6.098ms | 5.602ms | 0.0% | 5.602ms | 5.602ms | 5.602ms | 5.602ms | 0.000us | 6.76 MiB | none | 1.79M/s |
| q39 | date range, group by and offset | 4.000ms | 5.095ms | 5.004ms | 0.0% | 5.004ms | 5.004ms | 5.004ms | 5.004ms | 0.000us | 6.63 MiB | none | 2.00M/s |
| q40 | date range, a case and a wide group by | 7.000ms | 8.054ms | 8.142ms | 0.0% | 8.142ms | 8.142ms | 8.142ms | 8.142ms | 0.000us | 8.37 MiB | none | 1.23M/s |
| q41 | date range with an IN and a hash | 1.000ms | 2.043ms | 2.117ms | 0.0% | 2.117ms | 2.117ms | 2.117ms | 2.117ms | 0.000us | 5.39 MiB | none | 4.72M/s |
| q42 | date range and a deep offset | 1.000ms | 2.115ms | 2.186ms | 0.0% | 2.186ms | 2.186ms | 2.186ms | 2.186ms | 0.000us | 5.24 MiB | none | 4.57M/s |
| q43 | minute buckets over a date range | 1.000ms | 2.002ms | 2.002ms | 0.0% | 2.002ms | 2.002ms | 2.002ms | 2.002ms | 0.000us | 5.27 MiB | none | 5.00M/s |

rudb rudb 0.2.31 over 41 of 43 queries. Total 121.000ms by its own clock and 168.949ms by ours, 170.324ms cold, 20.000ms of CPU, peak 19.83 MiB, 3.39M/s and 575.08 MiB/s.

Running it cost 40% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 16.31x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 10000 rows, one out of every 10000 of the 99997497 in the full file, which is a development loop rather than the suite
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

- duckdb-pinned ran every query within 1.66x of every other one, and this suite spreads over 6x on an engine it is measuring
- clickhouse-local ran every query within 1.40x of every other one, and this suite spreads over 6x on an engine it is measuring
- polars ran every query within 1.41x of every other one, and this suite spreads over 6x on an engine it is measuring
- clickhouse-server ran every query within 1.79x of every other one, and this suite spreads over 6x on an engine it is measuring

A column that flat is not a column about the queries. Read those numbers as an upper bound on the engine and not as a measurement of it.

polars did not run q28, because the SQLContext has no strlen and no character length function.

polars did not run q29, because the SQLContext has no regexp_replace.

polars did not run q36, because the SQLContext rejects the repeated ClientIP output name in the group by.

polars did not run q43, because the SQLContext has no date_trunc.

So the polars column is 39 of 43 queries and its ratio is over the shared ones.

rudb did not run q19, because rudb's hash aggregate does not spill and this groups by a key that is close to unique over a hundred million rows, so the table outgrows the machine. tamnd/rudb#220, milestone E5.

rudb did not run q33, because rudb's hash aggregate does not spill and this groups by a key that is close to unique over a hundred million rows, so the table outgrows the machine. tamnd/rudb#220, milestone E5.

So the rudb column is 41 of 43 queries and its ratio is over the shared ones.

Answers differ, so this is not a comparison: q11: duckdb-pinned does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q11: clickhouse-local does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q11: datafusion does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q11: polars does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q11: clickhouse-server does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q11: rudb does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q12: duckdb-pinned does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q12: clickhouse-local does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q12: datafusion does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q12: polars does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q12: clickhouse-server does not agree with duckdb: 20 numbers against 20

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

Answers differ, so this is not a comparison: q15: duckdb-pinned does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q15: clickhouse-local does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q15: datafusion does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q15: polars does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q15: clickhouse-server does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q15: rudb does not agree with duckdb: 20 numbers against 20

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

Hot here means page cache warm and not buffer pool warm for every row but clickhouse-server, which stayed up across the whole suite with its own caches warm. That is the stronger kind of hot, so a ratio against that column is a ratio between two different quantities.

