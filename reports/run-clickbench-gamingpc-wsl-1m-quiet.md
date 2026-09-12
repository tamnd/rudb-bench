# clickbench on gamingpc-wsl

This is one run of the clickbench suite on gamingpc-wsl, over 7 engines and 43 queries, with 1 hot run of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

It ran over 999975 rows, one out of every 100 of the 99997497 in the full file, which is a development loop and not the suite. Nothing here is comparable to a full run or to anybody else's number.

## What ran

| what | it was |
| --- | --- |
| suite | clickbench |
| queries | 43 |
| tables | 138.85 MiB of Parquet in 1 table |
| rows | 999975 in the table every query reads |
| sample | 999975 rows, one out of every 100 of the 99997497 in the full file |
| summary | median with the interquartile range, per reporting rule two |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --rows 1000000 --runs 1 --report
```

The data goes under `RUDB_BENCH_DATA`, which defaults to `~/rudb-data`, and the engines have to be on the path. An engine that is not gets a row in the abstentions below rather than being left out of the table.

## The machine

| fact | value | source |
| --- | --- | --- |

A fact that says it was not read is a fact about this machine and not a gap in the report. A field that had silently defaulted would be a lie that survived into it.

## The engines

| engine | version | state | load | load cpu | on disk | that size is | format | machine load |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 1.840s | 9.330s | 572.76 MiB | its own database file | its own | 3.51 to 3.47 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 2.014s | 9.030s | 501.01 MiB | its own database file | its own | 3.47 to 4.15 |
| clickhouse-local | 26.9.1.1162 | ran | 769.335ms | 2.650s | 230.55 MiB | its own MergeTree parts, as system.parts counts the active ones | its own | 3.68 to 5.50 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 138.85 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 3.82 to 3.82 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 138.85 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 3.82 to 6.26 |
| clickhouse-server | 26.9.1.1162 | ran | 2.875s | not read | 143.71 MiB | its own MergeTree parts, sorted by hits by (CounterID, EventDate, UserID, EventTime, WatchID), as system.parts counts the active ones | its own | 5.50 to 5.30 |
| rudb | rudb 0.2.31 | ran | 0.000us | 0.000us | 138.85 MiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 4.15 to 3.82 |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 594.000ms | 1.051s | +77% | 1.064s | 3.720s | 3.54 | 306.81 MiB | none | 72.39M/s | 9.82 GiB/s | 1.00x |
| duckdb-pinned | 593.000ms | 1.597s | +169% | 1.628s | 3.900s | 2.44 | 307.52 MiB | none | 72.51M/s | 9.83 GiB/s | 1.00x |
| clickhouse-local | 2.866s | 5.205s | +82% | 5.038s | 6.570s | 1.26 | 425.77 MiB | none | 15.00M/s | 2.03 GiB/s | 4.82x |
| datafusion | 970.000ms | 1.523s | +57% | 1.546s | 8.700s | 5.71 | 1023.43 MiB | none | 44.33M/s | 6.01 GiB/s | 1.63x |
| polars | 1.216s | 4.795s | +294% | 4.801s | 10.340s | 2.16 | 520.05 MiB | none | 32.06M/s | 4.35 GiB/s | 2.05x |
| clickhouse-server | 449.000ms | 1.726s | +284% | 1.718s | not read | not read | not read | not read | 95.77M/s | 12.99 GiB/s | 0.76x |
| rudb | 9.854s | 9.977s | +1% | 10.010s | 9.560s | 0.96 | 286.17 MiB | none | 4.16M/s | 577.73 MiB/s | 16.59x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | clickhouse-server | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 0.000us | 1.000ms | 4.000ms | 1.000ms | 5.928ms | 1.000ms | 1.000ms |
| q2 | filtered count | 2.000ms | 2.000ms | 3.000ms | 8.000ms | 7.386ms | 1.000ms | 5.000ms |
| q3 | three aggregates | 2.000ms | 2.000ms | 70.000ms | 5.000ms | 8.674ms | 3.000ms | 15.000ms |
| q4 | average | 2.000ms | 3.000ms | 40.000ms | 6.000ms | 11.269ms | 3.000ms | 13.000ms |
| q5 | count distinct, high card | 12.000ms | 11.000ms | 98.000ms | 16.000ms | 21.237ms | 12.000ms | 213.000ms |
| q6 | count distinct, strings | 11.000ms | 9.000ms | 82.000ms | 18.000ms | 20.837ms | 11.000ms | 85.000ms |
| q7 | min and max of a date | 1.000ms | 1.000ms | 6.000ms | 1.000ms | 6.075ms | 1.000ms | 6.000ms |
| q8 | group by, low card | 2.000ms | 7.000ms | 32.000ms | 7.000ms | 15.132ms | 3.000ms | 5.000ms |
| q9 | group by and count distinct | 14.000ms | 15.000ms | 43.000ms | 22.000ms | 38.266ms | 15.000ms | 260.000ms |
| q10 | group by, several aggregates | 20.000ms | 19.000ms | 67.000ms | 19.000ms | 55.828ms | 31.000ms | 287.000ms |
| q11 | group by a string and count distinct | 7.000ms | 10.000ms | 66.000ms | 14.000ms | 24.581ms | 5.000ms | 29.000ms |
| q12 | group by two strings and count distinct | 8.000ms | 8.000ms | 67.000ms | 18.000ms | 25.601ms | 5.000ms | 34.000ms |
| q13 | group by a string and top k | 9.000ms | 10.000ms | 43.000ms | 20.000ms | 23.643ms | 11.000ms | 130.000ms |
| q14 | group by a string and count distinct | 14.000ms | 15.000ms | 66.000ms | 28.000ms | 38.147ms | 14.000ms | 162.000ms |
| q15 | group by two columns and top k | 12.000ms | 10.000ms | 46.000ms | 17.000ms | 26.345ms | 12.000ms | 145.000ms |
| q16 | group by, very high card | 14.000ms | 11.000ms | 38.000ms | 18.000ms | 28.710ms | 10.000ms | 219.000ms |
| q17 | group by two, very high card | 21.000ms | 20.000ms | 86.000ms | 35.000ms | 42.579ms | 20.000ms | 367.000ms |
| q18 | group by two, no ordering | 20.000ms | 19.000ms | 94.000ms | 31.000ms | 35.153ms | 7.000ms | 285.000ms |
| q19 | group by with an extract | 23.000ms | 23.000ms | 113.000ms | 39.000ms | 51.367ms | 23.000ms | no dialect |
| q20 | point lookup | 2.000ms | 3.000ms | 35.000ms | 7.000ms | 8.628ms | 1.000ms | 13.000ms |
| q21 | substring scan | 17.000ms | 15.000ms | 53.000ms | 21.000ms | 46.652ms | 12.000ms | 282.000ms |
| q22 | substring scan and group by | 18.000ms | 18.000ms | 88.000ms | 26.000ms | 51.007ms | 8.000ms | 317.000ms |
| q23 | two substring scans and group by | 22.000ms | 23.000ms | 95.000ms | 52.000ms | 83.545ms | 14.000ms | 704.000ms |
| q24 | select star and top k | 50.000ms | 42.000ms | 160.000ms | 97.000ms | 87.387ms | 20.000ms | 1.532s |
| q25 | top k by a date | 5.000ms | 4.000ms | 65.000ms | 13.000ms | 13.712ms | 4.000ms | 122.000ms |
| q26 | top k by a string | 4.000ms | 5.000ms | 38.000ms | 11.000ms | 13.912ms | 6.000ms | 108.000ms |
| q27 | top k by two columns | 5.000ms | 5.000ms | 78.000ms | 14.000ms | 16.976ms | 7.000ms | 137.000ms |
| q28 | group by with a string length | 17.000ms | 18.000ms | 52.000ms | 22.000ms | no dialect | 5.000ms | 282.000ms |
| q29 | group by a regular expression | 99.000ms | 90.000ms | 63.000ms | 52.000ms | no dialect | 31.000ms | 803.000ms |
| q30 | ninety sums over one column | 5.000ms | 16.000ms | 60.000ms | 14.000ms | 12.742ms | 7.000ms | 128.000ms |
| q31 | group by two and several aggregates | 11.000ms | 13.000ms | 86.000ms | 16.000ms | 30.742ms | 10.000ms | 151.000ms |
| q32 | group by a high card pair | 10.000ms | 13.000ms | 80.000ms | 19.000ms | 22.886ms | 24.000ms | 154.000ms |
| q33 | group by a high card pair, unfiltered | 23.000ms | 22.000ms | 94.000ms | 35.000ms | 37.745ms | 14.000ms | no dialect |
| q34 | group by a long string | 34.000ms | 29.000ms | 69.000ms | 47.000ms | 64.679ms | 27.000ms | 521.000ms |
| q35 | group by a constant and a long string | 34.000ms | 30.000ms | 72.000ms | 50.000ms | 74.915ms | 35.000ms | 543.000ms |
| q36 | group by four expressions | 15.000ms | 11.000ms | 54.000ms | 19.000ms | no dialect | 9.000ms | 343.000ms |
| q37 | date range and group by a URL | 5.000ms | 5.000ms | 103.000ms | 24.000ms | 37.920ms | 4.000ms | 263.000ms |
| q38 | date range and group by a title | 4.000ms | 5.000ms | 81.000ms | 31.000ms | 35.511ms | 4.000ms | 283.000ms |
| q39 | date range, group by and offset | 3.000ms | 5.000ms | 63.000ms | 20.000ms | 28.897ms | 4.000ms | 263.000ms |
| q40 | date range, a case and a wide group by | 6.000ms | 7.000ms | 81.000ms | 31.000ms | 30.835ms | 6.000ms | 503.000ms |
| q41 | date range with an IN and a hash | 3.000ms | 5.000ms | 93.000ms | 9.000ms | 15.537ms | 3.000ms | 48.000ms |
| q42 | date range and a deep offset | 4.000ms | 9.000ms | 61.000ms | 9.000ms | 15.450ms | 3.000ms | 52.000ms |
| q43 | minute buckets over a date range | 4.000ms | 4.000ms | 78.000ms | 8.000ms | no dialect | 3.000ms | 41.000ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 0.000us | 9.936ms | 9.854ms | 0.0% | 9.854ms | 9.854ms | 9.854ms | 9.854ms | 0.000us | 27.04 MiB | none | 101.48M/s |
| q2 | filtered count | 2.000ms | 11.201ms | 11.585ms | 0.0% | 11.585ms | 11.585ms | 11.585ms | 11.585ms | 0.000us | 30.70 MiB | none | 86.32M/s |
| q3 | three aggregates | 2.000ms | 12.395ms | 11.467ms | 0.0% | 11.467ms | 11.467ms | 11.467ms | 11.467ms | 10.000ms | 33.54 MiB | none | 87.20M/s |
| q4 | average | 2.000ms | 13.094ms | 12.551ms | 0.0% | 12.551ms | 12.551ms | 12.551ms | 12.551ms | 10.000ms | 37.29 MiB | none | 79.67M/s |
| q5 | count distinct, high card | 12.000ms | 23.311ms | 23.003ms | 0.0% | 23.003ms | 23.003ms | 23.003ms | 23.003ms | 80.000ms | 82.79 MiB | none | 43.47M/s |
| q6 | count distinct, strings | 11.000ms | 21.013ms | 21.827ms | 0.0% | 21.827ms | 21.827ms | 21.827ms | 21.827ms | 70.000ms | 64.54 MiB | none | 45.81M/s |
| q7 | min and max of a date | 1.000ms | 10.310ms | 10.031ms | 0.0% | 10.031ms | 10.031ms | 10.031ms | 10.031ms | 0.000us | 27.28 MiB | none | 99.69M/s |
| q8 | group by, low card | 2.000ms | 11.416ms | 11.668ms | 0.0% | 11.668ms | 11.668ms | 11.668ms | 11.668ms | 0.000us | 32.05 MiB | none | 85.70M/s |
| q9 | group by and count distinct | 14.000ms | 28.650ms | 25.385ms | 0.0% | 25.385ms | 25.385ms | 25.385ms | 25.385ms | 110.000ms | 103.55 MiB | none | 39.39M/s |
| q10 | group by, several aggregates | 20.000ms | 30.793ms | 31.568ms | 0.0% | 31.568ms | 31.568ms | 31.568ms | 31.568ms | 150.000ms | 127.05 MiB | none | 31.68M/s |
| q11 | group by a string and count distinct | 7.000ms | 17.783ms | 17.402ms | 0.0% | 17.402ms | 17.402ms | 17.402ms | 17.402ms | 40.000ms | 62.55 MiB | none | 57.46M/s |
| q12 | group by two strings and count distinct | 8.000ms | 17.842ms | 18.556ms | 0.0% | 18.556ms | 18.556ms | 18.556ms | 18.556ms | 40.000ms | 65.49 MiB | none | 53.89M/s |
| q13 | group by a string and top k | 9.000ms | 18.701ms | 19.367ms | 0.0% | 19.367ms | 19.367ms | 19.367ms | 19.367ms | 50.000ms | 69.05 MiB | none | 51.63M/s |
| q14 | group by a string and count distinct | 14.000ms | 26.188ms | 24.853ms | 0.0% | 24.853ms | 24.853ms | 24.853ms | 24.853ms | 100.000ms | 118.94 MiB | none | 40.24M/s |
| q15 | group by two columns and top k | 12.000ms | 21.065ms | 21.726ms | 0.0% | 21.726ms | 21.726ms | 21.726ms | 21.726ms | 60.000ms | 73.74 MiB | none | 46.03M/s |
| q16 | group by, very high card | 14.000ms | 25.217ms | 24.685ms | 0.0% | 24.685ms | 24.685ms | 24.685ms | 24.685ms | 100.000ms | 109.85 MiB | none | 40.51M/s |
| q17 | group by two, very high card | 21.000ms | 31.235ms | 32.551ms | 0.0% | 32.551ms | 32.551ms | 32.551ms | 32.551ms | 180.000ms | 178.90 MiB | none | 30.72M/s |
| q18 | group by two, no ordering | 20.000ms | 33.486ms | 31.584ms | 0.0% | 31.584ms | 31.584ms | 31.584ms | 31.584ms | 140.000ms | 177.85 MiB | none | 31.66M/s |
| q19 | group by with an extract | 23.000ms | 34.827ms | 33.741ms | 0.0% | 33.741ms | 33.741ms | 33.741ms | 33.741ms | 190.000ms | 173.62 MiB | none | 29.64M/s |
| q20 | point lookup | 2.000ms | 11.709ms | 12.083ms | 0.0% | 12.083ms | 12.083ms | 12.083ms | 12.083ms | 10.000ms | 36.94 MiB | none | 82.76M/s |
| q21 | substring scan | 17.000ms | 29.097ms | 28.668ms | 0.0% | 28.668ms | 28.668ms | 28.668ms | 28.668ms | 90.000ms | 97.73 MiB | none | 34.88M/s |
| q22 | substring scan and group by | 18.000ms | 30.229ms | 28.742ms | 0.0% | 28.742ms | 28.742ms | 28.742ms | 28.742ms | 90.000ms | 112.55 MiB | none | 34.79M/s |
| q23 | two substring scans and group by | 22.000ms | 31.720ms | 33.578ms | 0.0% | 33.578ms | 33.578ms | 33.578ms | 33.578ms | 140.000ms | 134.50 MiB | none | 29.78M/s |
| q24 | select star and top k | 50.000ms | 62.270ms | 62.391ms | 0.0% | 62.391ms | 62.391ms | 62.391ms | 62.391ms | 240.000ms | 215.22 MiB | none | 16.03M/s |
| q25 | top k by a date | 5.000ms | 15.848ms | 16.164ms | 0.0% | 16.164ms | 16.164ms | 16.164ms | 16.164ms | 20.000ms | 48.46 MiB | none | 61.86M/s |
| q26 | top k by a string | 4.000ms | 14.780ms | 14.264ms | 0.0% | 14.264ms | 14.264ms | 14.264ms | 14.264ms | 20.000ms | 39.29 MiB | none | 70.10M/s |
| q27 | top k by two columns | 5.000ms | 15.184ms | 14.693ms | 0.0% | 14.693ms | 14.693ms | 14.693ms | 14.693ms | 20.000ms | 44.73 MiB | none | 68.06M/s |
| q28 | group by with a string length | 17.000ms | 28.404ms | 27.558ms | 0.0% | 27.558ms | 27.558ms | 27.558ms | 27.558ms | 90.000ms | 109.55 MiB | none | 36.29M/s |
| q29 | group by a regular expression | 99.000ms | 115.203ms | 113.241ms | 0.0% | 113.241ms | 113.241ms | 113.241ms | 113.241ms | 670.000ms | 161.79 MiB | none | 8.83M/s |
| q30 | ninety sums over one column | 5.000ms | 14.481ms | 14.560ms | 0.0% | 14.560ms | 14.560ms | 14.560ms | 14.560ms | 0.000us | 34.79 MiB | none | 68.68M/s |
| q31 | group by two and several aggregates | 11.000ms | 19.957ms | 21.049ms | 0.0% | 21.049ms | 21.049ms | 21.049ms | 21.049ms | 60.000ms | 72.26 MiB | none | 47.51M/s |
| q32 | group by a high card pair | 10.000ms | 20.744ms | 19.838ms | 0.0% | 19.838ms | 19.838ms | 19.838ms | 19.838ms | 60.000ms | 81.85 MiB | none | 50.41M/s |
| q33 | group by a high card pair, unfiltered | 23.000ms | 32.960ms | 34.115ms | 0.0% | 34.115ms | 34.115ms | 34.115ms | 34.115ms | 170.000ms | 202.71 MiB | none | 29.31M/s |
| q34 | group by a long string | 34.000ms | 48.873ms | 46.600ms | 0.0% | 46.600ms | 46.600ms | 46.600ms | 46.600ms | 260.000ms | 297.55 MiB | none | 21.46M/s |
| q35 | group by a constant and a long string | 34.000ms | 51.166ms | 47.725ms | 0.0% | 47.725ms | 47.725ms | 47.725ms | 47.725ms | 300.000ms | 306.81 MiB | none | 20.95M/s |
| q36 | group by four expressions | 15.000ms | 25.828ms | 24.792ms | 0.0% | 24.792ms | 24.792ms | 24.792ms | 24.792ms | 90.000ms | 114.10 MiB | none | 40.33M/s |
| q37 | date range and group by a URL | 5.000ms | 14.218ms | 13.935ms | 0.0% | 13.935ms | 13.935ms | 13.935ms | 13.935ms | 10.000ms | 37.30 MiB | none | 71.76M/s |
| q38 | date range and group by a title | 4.000ms | 14.691ms | 14.579ms | 0.0% | 14.579ms | 14.579ms | 14.579ms | 14.579ms | 10.000ms | 36.80 MiB | none | 68.59M/s |
| q39 | date range, group by and offset | 3.000ms | 13.129ms | 13.125ms | 0.0% | 13.125ms | 13.125ms | 13.125ms | 13.125ms | 0.000us | 35.30 MiB | none | 76.19M/s |
| q40 | date range, a case and a wide group by | 6.000ms | 16.109ms | 16.656ms | 0.0% | 16.656ms | 16.656ms | 16.656ms | 16.656ms | 20.000ms | 46.05 MiB | none | 60.04M/s |
| q41 | date range with an IN and a hash | 3.000ms | 13.174ms | 12.956ms | 0.0% | 12.956ms | 12.956ms | 12.956ms | 12.956ms | 0.000us | 37.27 MiB | none | 77.18M/s |
| q42 | date range and a deep offset | 4.000ms | 12.656ms | 12.991ms | 0.0% | 12.991ms | 12.991ms | 12.991ms | 12.991ms | 10.000ms | 36.30 MiB | none | 76.97M/s |
| q43 | minute buckets over a date range | 4.000ms | 12.791ms | 12.976ms | 0.0% | 12.976ms | 12.976ms | 12.976ms | 12.976ms | 10.000ms | 35.80 MiB | none | 77.06M/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 594.000ms by its own clock and 1.051s by ours, 1.064s cold, 3.720s of CPU, peak 306.81 MiB, 72.39M/s and 9.82 GiB/s.

Running it cost 77% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 11.49x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 22.803ms | 23.081ms | 0.0% | 23.081ms | 23.081ms | 23.081ms | 23.081ms | 20.000ms | 40.39 MiB | none | 43.32M/s |
| q2 | filtered count | 2.000ms | 25.083ms | 24.617ms | 0.0% | 24.617ms | 24.617ms | 24.617ms | 24.617ms | 20.000ms | 43.25 MiB | none | 40.62M/s |
| q3 | three aggregates | 2.000ms | 24.958ms | 25.059ms | 0.0% | 25.059ms | 25.059ms | 25.059ms | 25.059ms | 20.000ms | 46.01 MiB | none | 39.90M/s |
| q4 | average | 3.000ms | 27.719ms | 25.565ms | 0.0% | 25.565ms | 25.565ms | 25.565ms | 25.565ms | 20.000ms | 49.77 MiB | none | 39.12M/s |
| q5 | count distinct, high card | 11.000ms | 34.124ms | 33.414ms | 0.0% | 33.414ms | 33.414ms | 33.414ms | 33.414ms | 70.000ms | 95.65 MiB | none | 29.93M/s |
| q6 | count distinct, strings | 9.000ms | 30.618ms | 31.510ms | 0.0% | 31.510ms | 31.510ms | 31.510ms | 31.510ms | 50.000ms | 81.27 MiB | none | 31.74M/s |
| q7 | min and max of a date | 1.000ms | 23.492ms | 23.906ms | 0.0% | 23.906ms | 23.906ms | 23.906ms | 23.906ms | 10.000ms | 40.52 MiB | none | 41.83M/s |
| q8 | group by, low card | 7.000ms | 30.572ms | 28.388ms | 0.0% | 28.388ms | 28.388ms | 28.388ms | 28.388ms | 20.000ms | 46.29 MiB | none | 35.23M/s |
| q9 | group by and count distinct | 15.000ms | 38.381ms | 37.086ms | 0.0% | 37.086ms | 37.086ms | 37.086ms | 37.086ms | 110.000ms | 121.55 MiB | none | 26.96M/s |
| q10 | group by, several aggregates | 19.000ms | 42.939ms | 42.739ms | 0.0% | 42.739ms | 42.739ms | 42.739ms | 42.739ms | 130.000ms | 132.38 MiB | none | 23.40M/s |
| q11 | group by a string and count distinct | 10.000ms | 30.054ms | 34.639ms | 0.0% | 34.639ms | 34.639ms | 34.639ms | 34.639ms | 50.000ms | 75.50 MiB | none | 28.87M/s |
| q12 | group by two strings and count distinct | 8.000ms | 30.591ms | 30.941ms | 0.0% | 30.941ms | 30.941ms | 30.941ms | 30.941ms | 60.000ms | 76.55 MiB | none | 32.32M/s |
| q13 | group by a string and top k | 10.000ms | 32.504ms | 32.941ms | 0.0% | 32.941ms | 32.941ms | 32.941ms | 32.941ms | 60.000ms | 85.54 MiB | none | 30.36M/s |
| q14 | group by a string and count distinct | 15.000ms | 38.668ms | 38.604ms | 0.0% | 38.604ms | 38.604ms | 38.604ms | 38.604ms | 100.000ms | 132.04 MiB | none | 25.90M/s |
| q15 | group by two columns and top k | 10.000ms | 34.033ms | 33.392ms | 0.0% | 33.392ms | 33.392ms | 33.392ms | 33.392ms | 70.000ms | 94.79 MiB | none | 29.95M/s |
| q16 | group by, very high card | 11.000ms | 35.082ms | 34.655ms | 0.0% | 34.655ms | 34.655ms | 34.655ms | 34.655ms | 90.000ms | 111.27 MiB | none | 28.86M/s |
| q17 | group by two, very high card | 20.000ms | 41.819ms | 44.350ms | 0.0% | 44.350ms | 44.350ms | 44.350ms | 44.350ms | 180.000ms | 195.54 MiB | none | 22.55M/s |
| q18 | group by two, no ordering | 19.000ms | 43.020ms | 43.246ms | 0.0% | 43.246ms | 43.246ms | 43.246ms | 43.246ms | 150.000ms | 195.33 MiB | none | 23.12M/s |
| q19 | group by with an extract | 23.000ms | 46.476ms | 49.739ms | 0.0% | 49.739ms | 49.739ms | 49.739ms | 49.739ms | 190.000ms | 224.41 MiB | none | 20.10M/s |
| q20 | point lookup | 3.000ms | 25.424ms | 25.546ms | 0.0% | 25.546ms | 25.546ms | 25.546ms | 25.546ms | 20.000ms | 48.52 MiB | none | 39.14M/s |
| q21 | substring scan | 15.000ms | 37.267ms | 38.899ms | 0.0% | 38.899ms | 38.899ms | 38.899ms | 38.899ms | 120.000ms | 102.24 MiB | none | 25.71M/s |
| q22 | substring scan and group by | 18.000ms | 42.791ms | 41.321ms | 0.0% | 41.321ms | 41.321ms | 41.321ms | 41.321ms | 110.000ms | 121.29 MiB | none | 24.20M/s |
| q23 | two substring scans and group by | 23.000ms | 47.594ms | 47.503ms | 0.0% | 47.503ms | 47.503ms | 47.503ms | 47.503ms | 140.000ms | 154.59 MiB | none | 21.05M/s |
| q24 | select star and top k | 42.000ms | 67.741ms | 69.016ms | 0.0% | 69.016ms | 69.016ms | 69.016ms | 69.016ms | 190.000ms | 218.89 MiB | none | 14.49M/s |
| q25 | top k by a date | 4.000ms | 28.474ms | 27.310ms | 0.0% | 27.310ms | 27.310ms | 27.310ms | 27.310ms | 40.000ms | 62.53 MiB | none | 36.62M/s |
| q26 | top k by a string | 5.000ms | 27.724ms | 26.964ms | 0.0% | 26.964ms | 26.964ms | 26.964ms | 26.964ms | 40.000ms | 57.03 MiB | none | 37.09M/s |
| q27 | top k by two columns | 5.000ms | 29.321ms | 27.733ms | 0.0% | 27.733ms | 27.733ms | 27.733ms | 27.733ms | 40.000ms | 63.27 MiB | none | 36.06M/s |
| q28 | group by with a string length | 18.000ms | 48.422ms | 42.067ms | 0.0% | 42.067ms | 42.067ms | 42.067ms | 42.067ms | 110.000ms | 117.04 MiB | none | 23.77M/s |
| q29 | group by a regular expression | 90.000ms | 137.030ms | 115.372ms | 0.0% | 115.372ms | 115.372ms | 115.372ms | 115.372ms | 580.000ms | 177.67 MiB | none | 8.67M/s |
| q30 | ninety sums over one column | 16.000ms | 37.514ms | 38.580ms | 0.0% | 38.580ms | 38.580ms | 38.580ms | 38.580ms | 40.000ms | 55.52 MiB | none | 25.92M/s |
| q31 | group by two and several aggregates | 13.000ms | 39.511ms | 34.895ms | 0.0% | 34.895ms | 34.895ms | 34.895ms | 34.895ms | 70.000ms | 95.05 MiB | none | 28.66M/s |
| q32 | group by a high card pair | 13.000ms | 35.491ms | 36.131ms | 0.0% | 36.131ms | 36.131ms | 36.131ms | 36.131ms | 80.000ms | 102.05 MiB | none | 27.68M/s |
| q33 | group by a high card pair, unfiltered | 22.000ms | 46.986ms | 46.259ms | 0.0% | 46.259ms | 46.259ms | 46.259ms | 46.259ms | 210.000ms | 217.66 MiB | none | 21.62M/s |
| q34 | group by a long string | 29.000ms | 55.718ms | 54.572ms | 0.0% | 54.572ms | 54.572ms | 54.572ms | 54.572ms | 210.000ms | 304.04 MiB | none | 18.32M/s |
| q35 | group by a constant and a long string | 30.000ms | 59.060ms | 57.657ms | 0.0% | 57.657ms | 57.657ms | 57.657ms | 57.657ms | 240.000ms | 307.52 MiB | none | 17.34M/s |
| q36 | group by four expressions | 11.000ms | 36.043ms | 35.550ms | 0.0% | 35.550ms | 35.550ms | 35.550ms | 35.550ms | 80.000ms | 111.03 MiB | none | 28.13M/s |
| q37 | date range and group by a URL | 5.000ms | 27.113ms | 26.891ms | 0.0% | 26.891ms | 26.891ms | 26.891ms | 26.891ms | 20.000ms | 50.54 MiB | none | 37.19M/s |
| q38 | date range and group by a title | 5.000ms | 26.290ms | 26.788ms | 0.0% | 26.788ms | 26.788ms | 26.788ms | 26.788ms | 20.000ms | 48.53 MiB | none | 37.33M/s |
| q39 | date range, group by and offset | 5.000ms | 26.734ms | 27.211ms | 0.0% | 27.211ms | 27.211ms | 27.211ms | 27.211ms | 20.000ms | 48.48 MiB | none | 36.75M/s |
| q40 | date range, a case and a wide group by | 7.000ms | 29.789ms | 28.850ms | 0.0% | 28.850ms | 28.850ms | 28.850ms | 28.850ms | 30.000ms | 58.54 MiB | none | 34.66M/s |
| q41 | date range with an IN and a hash | 5.000ms | 26.173ms | 25.943ms | 0.0% | 25.943ms | 25.943ms | 25.943ms | 25.943ms | 20.000ms | 49.55 MiB | none | 38.55M/s |
| q42 | date range and a deep offset | 9.000ms | 30.730ms | 30.617ms | 0.0% | 30.617ms | 30.617ms | 30.617ms | 30.617ms | 30.000ms | 50.04 MiB | none | 32.66M/s |
| q43 | minute buckets over a date range | 4.000ms | 26.024ms | 27.364ms | 0.0% | 27.364ms | 27.364ms | 27.364ms | 27.364ms | 20.000ms | 48.02 MiB | none | 36.54M/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 593.000ms by its own clock and 1.597s by ours, 1.628s cold, 3.900s of CPU, peak 307.52 MiB, 72.51M/s and 9.83 GiB/s.

Running it cost 169% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 5.00x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 4.000ms | 53.233ms | 54.447ms | 0.0% | 54.447ms | 54.447ms | 54.447ms | 54.447ms | 60.000ms | 202.43 MiB | none | 18.37M/s |
| q2 | filtered count | 3.000ms | 53.673ms | 51.555ms | 0.0% | 51.555ms | 51.555ms | 51.555ms | 51.555ms | 50.000ms | 202.51 MiB | none | 19.40M/s |
| q3 | three aggregates | 70.000ms | 128.672ms | 138.801ms | 0.0% | 138.801ms | 138.801ms | 138.801ms | 138.801ms | 90.000ms | 216.32 MiB | none | 7.20M/s |
| q4 | average | 40.000ms | 97.214ms | 87.166ms | 0.0% | 87.166ms | 87.166ms | 87.166ms | 87.166ms | 70.000ms | 216.95 MiB | none | 11.47M/s |
| q5 | count distinct, high card | 98.000ms | 133.874ms | 153.410ms | 0.0% | 153.410ms | 153.410ms | 153.410ms | 153.410ms | 170.000ms | 314.54 MiB | none | 6.52M/s |
| q6 | count distinct, strings | 82.000ms | 123.484ms | 135.340ms | 0.0% | 135.340ms | 135.340ms | 135.340ms | 135.340ms | 130.000ms | 258.07 MiB | none | 7.39M/s |
| q7 | min and max of a date | 6.000ms | 52.437ms | 52.014ms | 0.0% | 52.014ms | 52.014ms | 52.014ms | 52.014ms | 50.000ms | 205.07 MiB | none | 19.23M/s |
| q8 | group by, low card | 32.000ms | 78.222ms | 83.180ms | 0.0% | 83.180ms | 83.180ms | 83.180ms | 83.180ms | 80.000ms | 212.70 MiB | none | 12.02M/s |
| q9 | group by and count distinct | 43.000ms | 98.616ms | 96.772ms | 0.0% | 96.772ms | 96.772ms | 96.772ms | 96.772ms | 160.000ms | 290.34 MiB | none | 10.33M/s |
| q10 | group by, several aggregates | 67.000ms | 134.587ms | 119.930ms | 0.0% | 119.930ms | 119.930ms | 119.930ms | 119.930ms | 170.000ms | 298.39 MiB | none | 8.34M/s |
| q11 | group by a string and count distinct | 66.000ms | 128.982ms | 120.169ms | 0.0% | 120.169ms | 120.169ms | 120.169ms | 120.169ms | 100.000ms | 239.70 MiB | none | 8.32M/s |
| q12 | group by two strings and count distinct | 67.000ms | 120.004ms | 122.249ms | 0.0% | 122.249ms | 122.249ms | 122.249ms | 122.249ms | 130.000ms | 244.63 MiB | none | 8.18M/s |
| q13 | group by a string and top k | 43.000ms | 98.415ms | 94.829ms | 0.0% | 94.829ms | 94.829ms | 94.829ms | 94.829ms | 130.000ms | 266.21 MiB | none | 10.55M/s |
| q14 | group by a string and count distinct | 66.000ms | 117.504ms | 121.250ms | 0.0% | 121.250ms | 121.250ms | 121.250ms | 121.250ms | 140.000ms | 306.39 MiB | none | 8.25M/s |
| q15 | group by two columns and top k | 46.000ms | 135.249ms | 99.323ms | 0.0% | 99.323ms | 99.323ms | 99.323ms | 99.323ms | 150.000ms | 285.66 MiB | none | 10.07M/s |
| q16 | group by, very high card | 38.000ms | 103.637ms | 90.504ms | 0.0% | 90.504ms | 90.504ms | 90.504ms | 90.504ms | 150.000ms | 281.83 MiB | none | 11.05M/s |
| q17 | group by two, very high card | 86.000ms | 145.890ms | 144.551ms | 0.0% | 144.551ms | 144.551ms | 144.551ms | 144.551ms | 270.000ms | 382.89 MiB | none | 6.92M/s |
| q18 | group by two, no ordering | 94.000ms | 103.303ms | 150.909ms | 0.0% | 150.909ms | 150.909ms | 150.909ms | 150.909ms | 120.000ms | 279.46 MiB | none | 6.63M/s |
| q19 | group by with an extract | 113.000ms | 156.210ms | 170.177ms | 0.0% | 170.177ms | 170.177ms | 170.177ms | 170.177ms | 300.000ms | 409.70 MiB | none | 5.88M/s |
| q20 | point lookup | 35.000ms | 79.771ms | 82.750ms | 0.0% | 82.750ms | 82.750ms | 82.750ms | 82.750ms | 70.000ms | 217.57 MiB | none | 12.08M/s |
| q21 | substring scan | 53.000ms | 102.372ms | 106.236ms | 0.0% | 106.236ms | 106.236ms | 106.236ms | 106.236ms | 130.000ms | 275.25 MiB | none | 9.41M/s |
| q22 | substring scan and group by | 88.000ms | 143.026ms | 141.039ms | 0.0% | 141.039ms | 141.039ms | 141.039ms | 141.039ms | 160.000ms | 295.32 MiB | none | 7.09M/s |
| q23 | two substring scans and group by | 95.000ms | 184.291ms | 151.820ms | 0.0% | 151.820ms | 151.820ms | 151.820ms | 151.820ms | 240.000ms | 321.81 MiB | none | 6.59M/s |
| q24 | select star and top k | 160.000ms | 229.026ms | 219.347ms | 0.0% | 219.347ms | 219.347ms | 219.347ms | 219.347ms | 300.000ms | 396.94 MiB | none | 4.56M/s |
| q25 | top k by a date | 65.000ms | 128.700ms | 118.532ms | 0.0% | 118.532ms | 118.532ms | 118.532ms | 118.532ms | 110.000ms | 252.47 MiB | none | 8.44M/s |
| q26 | top k by a string | 38.000ms | 113.990ms | 96.857ms | 0.0% | 96.857ms | 96.857ms | 96.857ms | 96.857ms | 110.000ms | 246.32 MiB | none | 10.32M/s |
| q27 | top k by two columns | 78.000ms | 102.761ms | 132.508ms | 0.0% | 132.508ms | 132.508ms | 132.508ms | 132.508ms | 100.000ms | 244.21 MiB | none | 7.55M/s |
| q28 | group by with a string length | 52.000ms | 90.795ms | 107.952ms | 0.0% | 107.952ms | 107.952ms | 107.952ms | 107.952ms | 100.000ms | 236.57 MiB | none | 9.26M/s |
| q29 | group by a regular expression | 63.000ms | 106.144ms | 117.914ms | 0.0% | 117.914ms | 117.914ms | 117.914ms | 117.914ms | 270.000ms | 348.23 MiB | none | 8.48M/s |
| q30 | ninety sums over one column | 60.000ms | 106.389ms | 113.679ms | 0.0% | 113.679ms | 113.679ms | 113.679ms | 113.679ms | 90.000ms | 216.38 MiB | none | 8.80M/s |
| q31 | group by two and several aggregates | 86.000ms | 140.760ms | 140.025ms | 0.0% | 140.025ms | 140.025ms | 140.025ms | 140.025ms | 160.000ms | 262.32 MiB | none | 7.14M/s |
| q32 | group by a high card pair | 80.000ms | 129.655ms | 135.695ms | 0.0% | 135.695ms | 135.695ms | 135.695ms | 135.695ms | 180.000ms | 277.53 MiB | none | 7.37M/s |
| q33 | group by a high card pair, unfiltered | 94.000ms | 136.512ms | 148.884ms | 0.0% | 148.884ms | 148.884ms | 148.884ms | 148.884ms | 260.000ms | 347.07 MiB | none | 6.72M/s |
| q34 | group by a long string | 69.000ms | 123.640ms | 125.523ms | 0.0% | 125.523ms | 125.523ms | 125.523ms | 125.523ms | 390.000ms | 419.61 MiB | none | 7.97M/s |
| q35 | group by a constant and a long string | 72.000ms | 138.555ms | 128.321ms | 0.0% | 128.321ms | 128.321ms | 128.321ms | 128.321ms | 410.000ms | 425.77 MiB | none | 7.79M/s |
| q36 | group by four expressions | 54.000ms | 108.635ms | 102.461ms | 0.0% | 102.461ms | 102.461ms | 102.461ms | 102.461ms | 200.000ms | 290.70 MiB | none | 9.76M/s |
| q37 | date range and group by a URL | 103.000ms | 128.926ms | 159.472ms | 0.0% | 159.472ms | 159.472ms | 159.472ms | 159.472ms | 90.000ms | 226.57 MiB | none | 6.27M/s |
| q38 | date range and group by a title | 81.000ms | 91.035ms | 137.295ms | 0.0% | 137.295ms | 137.295ms | 137.295ms | 137.295ms | 100.000ms | 224.18 MiB | none | 7.28M/s |
| q39 | date range, group by and offset | 63.000ms | 128.992ms | 119.114ms | 0.0% | 119.114ms | 119.114ms | 119.114ms | 119.114ms | 140.000ms | 228.82 MiB | none | 8.40M/s |
| q40 | date range, a case and a wide group by | 81.000ms | 123.286ms | 135.429ms | 0.0% | 135.429ms | 135.429ms | 135.429ms | 135.429ms | 170.000ms | 235.50 MiB | none | 7.38M/s |
| q41 | date range with an IN and a hash | 93.000ms | 114.767ms | 146.974ms | 0.0% | 146.974ms | 146.974ms | 146.974ms | 146.974ms | 90.000ms | 220.79 MiB | none | 6.80M/s |
| q42 | date range and a deep offset | 61.000ms | 131.361ms | 117.967ms | 0.0% | 117.967ms | 117.967ms | 117.967ms | 117.967ms | 90.000ms | 223.50 MiB | none | 8.48M/s |
| q43 | minute buckets over a date range | 78.000ms | 91.032ms | 132.733ms | 0.0% | 132.733ms | 132.733ms | 132.733ms | 132.733ms | 90.000ms | 221.12 MiB | none | 7.53M/s |

clickhouse-local 26.9.1.1162 over 43 of 43 queries. Total 2.866s by its own clock and 5.205s by ours, 5.038s cold, 6.570s of CPU, peak 425.77 MiB, 15.00M/s and 2.03 GiB/s.

Running it cost 82% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 4.25x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 15.394ms | 12.281ms | 0.0% | 12.281ms | 12.281ms | 12.281ms | 12.281ms | 0.000us | 80.64 MiB | none | 81.42M/s |
| q2 | filtered count | 8.000ms | 18.818ms | 19.969ms | 0.0% | 19.969ms | 19.969ms | 19.969ms | 19.969ms | 90.000ms | 140.91 MiB | none | 50.08M/s |
| q3 | three aggregates | 5.000ms | 20.422ms | 17.552ms | 0.0% | 17.552ms | 17.552ms | 17.552ms | 17.552ms | 40.000ms | 163.98 MiB | none | 56.97M/s |
| q4 | average | 6.000ms | 17.120ms | 18.030ms | 0.0% | 18.030ms | 18.030ms | 18.030ms | 18.030ms | 40.000ms | 138.30 MiB | none | 55.46M/s |
| q5 | count distinct, high card | 16.000ms | 30.663ms | 28.635ms | 0.0% | 28.635ms | 28.635ms | 28.635ms | 28.635ms | 180.000ms | 399.01 MiB | none | 34.92M/s |
| q6 | count distinct, strings | 18.000ms | 30.601ms | 30.176ms | 0.0% | 30.176ms | 30.176ms | 30.176ms | 30.176ms | 220.000ms | 390.25 MiB | none | 33.14M/s |
| q7 | min and max of a date | 1.000ms | 12.798ms | 12.839ms | 0.0% | 12.839ms | 12.839ms | 12.839ms | 12.839ms | 10.000ms | 81.36 MiB | none | 77.89M/s |
| q8 | group by, low card | 7.000ms | 18.177ms | 18.862ms | 0.0% | 18.862ms | 18.862ms | 18.862ms | 18.862ms | 40.000ms | 134.58 MiB | none | 53.02M/s |
| q9 | group by and count distinct | 22.000ms | 32.414ms | 34.127ms | 0.0% | 34.127ms | 34.127ms | 34.127ms | 34.127ms | 260.000ms | 449.29 MiB | none | 29.30M/s |
| q10 | group by, several aggregates | 19.000ms | 33.193ms | 31.177ms | 0.0% | 31.177ms | 31.177ms | 31.177ms | 31.177ms | 130.000ms | 306.42 MiB | none | 32.07M/s |
| q11 | group by a string and count distinct | 14.000ms | 25.415ms | 26.011ms | 0.0% | 26.011ms | 26.011ms | 26.011ms | 26.011ms | 150.000ms | 253.63 MiB | none | 38.44M/s |
| q12 | group by two strings and count distinct | 18.000ms | 30.095ms | 30.030ms | 0.0% | 30.030ms | 30.030ms | 30.030ms | 30.030ms | 240.000ms | 298.68 MiB | none | 33.30M/s |
| q13 | group by a string and top k | 20.000ms | 34.088ms | 32.124ms | 0.0% | 32.124ms | 32.124ms | 32.124ms | 32.124ms | 230.000ms | 386.86 MiB | none | 31.13M/s |
| q14 | group by a string and count distinct | 28.000ms | 42.510ms | 42.081ms | 0.0% | 42.081ms | 42.081ms | 42.081ms | 42.081ms | 360.000ms | 528.39 MiB | none | 23.76M/s |
| q15 | group by two columns and top k | 17.000ms | 33.464ms | 29.244ms | 0.0% | 29.244ms | 29.244ms | 29.244ms | 29.244ms | 120.000ms | 375.18 MiB | none | 34.19M/s |
| q16 | group by, very high card | 18.000ms | 28.408ms | 31.247ms | 0.0% | 31.247ms | 31.247ms | 31.247ms | 31.247ms | 200.000ms | 409.41 MiB | none | 32.00M/s |
| q17 | group by two, very high card | 35.000ms | 46.350ms | 49.267ms | 0.0% | 49.267ms | 49.267ms | 49.267ms | 49.267ms | 510.000ms | 635.77 MiB | none | 20.30M/s |
| q18 | group by two, no ordering | 31.000ms | 44.558ms | 44.488ms | 0.0% | 44.488ms | 44.488ms | 44.488ms | 44.488ms | 310.000ms | 663.97 MiB | none | 22.48M/s |
| q19 | group by with an extract | 39.000ms | 55.346ms | 53.392ms | 0.0% | 53.392ms | 53.392ms | 53.392ms | 53.392ms | 510.000ms | 747.22 MiB | none | 18.73M/s |
| q20 | point lookup | 7.000ms | 20.575ms | 19.625ms | 0.0% | 19.625ms | 19.625ms | 19.625ms | 19.625ms | 110.000ms | 145.09 MiB | none | 50.95M/s |
| q21 | substring scan | 21.000ms | 34.987ms | 33.063ms | 0.0% | 33.063ms | 33.063ms | 33.063ms | 33.063ms | 150.000ms | 257.93 MiB | none | 30.24M/s |
| q22 | substring scan and group by | 26.000ms | 39.587ms | 38.530ms | 0.0% | 38.530ms | 38.530ms | 38.530ms | 38.530ms | 220.000ms | 344.72 MiB | none | 25.95M/s |
| q23 | two substring scans and group by | 52.000ms | 65.264ms | 65.002ms | 0.0% | 65.002ms | 65.002ms | 65.002ms | 65.002ms | 320.000ms | 515.69 MiB | none | 15.38M/s |
| q24 | select star and top k | 97.000ms | 123.090ms | 112.564ms | 0.0% | 112.564ms | 112.564ms | 112.564ms | 112.564ms | 700.000ms | 1023.43 MiB | none | 8.88M/s |
| q25 | top k by a date | 13.000ms | 23.560ms | 25.490ms | 0.0% | 25.490ms | 25.490ms | 25.490ms | 25.490ms | 100.000ms | 298.11 MiB | none | 39.23M/s |
| q26 | top k by a string | 11.000ms | 23.868ms | 23.856ms | 0.0% | 23.856ms | 23.856ms | 23.856ms | 23.856ms | 90.000ms | 264.68 MiB | none | 41.92M/s |
| q27 | top k by two columns | 14.000ms | 25.861ms | 26.611ms | 0.0% | 26.611ms | 26.611ms | 26.611ms | 26.611ms | 120.000ms | 317.85 MiB | none | 37.58M/s |
| q28 | group by with a string length | 22.000ms | 35.541ms | 35.208ms | 0.0% | 35.208ms | 35.208ms | 35.208ms | 35.208ms | 180.000ms | 328.78 MiB | none | 28.40M/s |
| q29 | group by a regular expression | 52.000ms | 65.339ms | 68.502ms | 0.0% | 68.502ms | 68.502ms | 68.502ms | 68.502ms | 400.000ms | 502.97 MiB | none | 14.60M/s |
| q30 | ninety sums over one column | 14.000ms | 26.858ms | 27.407ms | 0.0% | 27.407ms | 27.407ms | 27.407ms | 27.407ms | 40.000ms | 151.51 MiB | none | 36.49M/s |
| q31 | group by two and several aggregates | 16.000ms | 31.500ms | 28.237ms | 0.0% | 28.237ms | 28.237ms | 28.237ms | 28.237ms | 110.000ms | 319.65 MiB | none | 35.41M/s |
| q32 | group by a high card pair | 19.000ms | 32.226ms | 32.426ms | 0.0% | 32.426ms | 32.426ms | 32.426ms | 32.426ms | 140.000ms | 340.19 MiB | none | 30.84M/s |
| q33 | group by a high card pair, unfiltered | 35.000ms | 46.794ms | 48.268ms | 0.0% | 48.268ms | 48.268ms | 48.268ms | 48.268ms | 500.000ms | 639.09 MiB | none | 20.72M/s |
| q34 | group by a long string | 47.000ms | 68.097ms | 62.511ms | 0.0% | 62.511ms | 62.511ms | 62.511ms | 62.511ms | 490.000ms | 777.38 MiB | none | 16.00M/s |
| q35 | group by a constant and a long string | 50.000ms | 63.336ms | 63.444ms | 0.0% | 63.444ms | 63.444ms | 63.444ms | 63.444ms | 610.000ms | 788.91 MiB | none | 15.76M/s |
| q36 | group by four expressions | 19.000ms | 31.664ms | 32.223ms | 0.0% | 32.223ms | 32.223ms | 32.223ms | 32.223ms | 240.000ms | 405.70 MiB | none | 31.03M/s |
| q37 | date range and group by a URL | 24.000ms | 35.676ms | 37.513ms | 0.0% | 37.513ms | 37.513ms | 37.513ms | 37.513ms | 160.000ms | 222.10 MiB | none | 26.66M/s |
| q38 | date range and group by a title | 31.000ms | 45.338ms | 43.916ms | 0.0% | 43.916ms | 43.916ms | 43.916ms | 43.916ms | 80.000ms | 187.93 MiB | none | 22.77M/s |
| q39 | date range, group by and offset | 20.000ms | 32.361ms | 31.249ms | 0.0% | 31.249ms | 31.249ms | 31.249ms | 31.249ms | 80.000ms | 186.14 MiB | none | 32.00M/s |
| q40 | date range, a case and a wide group by | 31.000ms | 41.881ms | 43.002ms | 0.0% | 43.002ms | 43.002ms | 43.002ms | 43.002ms | 100.000ms | 222.39 MiB | none | 23.25M/s |
| q41 | date range with an IN and a hash | 9.000ms | 20.941ms | 21.364ms | 0.0% | 21.364ms | 21.364ms | 21.364ms | 21.364ms | 40.000ms | 143.88 MiB | none | 46.81M/s |
| q42 | date range and a deep offset | 9.000ms | 21.401ms | 20.585ms | 0.0% | 20.585ms | 20.585ms | 20.585ms | 20.585ms | 40.000ms | 145.81 MiB | none | 48.58M/s |
| q43 | minute buckets over a date range | 8.000ms | 20.547ms | 20.426ms | 0.0% | 20.426ms | 20.426ms | 20.426ms | 20.426ms | 40.000ms | 132.11 MiB | none | 48.96M/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 970.000ms by its own clock and 1.523s by ours, 1.546s cold, 8.700s of CPU, peak 1023.43 MiB, 44.33M/s and 6.01 GiB/s.

Running it cost 57% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 9.17x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 5.928ms | 112.870ms | 96.048ms | 0.0% | 96.048ms | 96.048ms | 96.048ms | 96.048ms | 110.000ms | 62.13 MiB | none | 10.41M/s |
| q2 | filtered count | 7.386ms | 96.666ms | 93.640ms | 0.0% | 93.640ms | 93.640ms | 93.640ms | 93.640ms | 120.000ms | 68.40 MiB | none | 10.68M/s |
| q3 | three aggregates | 8.674ms | 99.749ms | 99.973ms | 0.0% | 99.973ms | 99.973ms | 99.973ms | 99.973ms | 160.000ms | 74.18 MiB | none | 10.00M/s |
| q4 | average | 11.269ms | 99.062ms | 101.042ms | 0.0% | 101.042ms | 101.042ms | 101.042ms | 101.042ms | 160.000ms | 86.70 MiB | none | 9.90M/s |
| q5 | count distinct, high card | 21.237ms | 109.156ms | 114.222ms | 0.0% | 114.222ms | 114.222ms | 114.222ms | 114.222ms | 190.000ms | 132.45 MiB | none | 8.75M/s |
| q6 | count distinct, strings | 20.837ms | 112.287ms | 109.505ms | 0.0% | 109.505ms | 109.505ms | 109.505ms | 109.505ms | 170.000ms | 137.41 MiB | none | 9.13M/s |
| q7 | min and max of a date | 6.075ms | 91.223ms | 98.948ms | 0.0% | 98.948ms | 98.948ms | 98.948ms | 98.948ms | 290.000ms | 71.38 MiB | none | 10.11M/s |
| q8 | group by, low card | 15.132ms | 104.769ms | 102.205ms | 0.0% | 102.205ms | 102.205ms | 102.205ms | 102.205ms | 130.000ms | 77.26 MiB | none | 9.78M/s |
| q9 | group by and count distinct | 38.266ms | 135.160ms | 130.054ms | 0.0% | 130.054ms | 130.054ms | 130.054ms | 130.054ms | 300.000ms | 238.45 MiB | none | 7.69M/s |
| q10 | group by, several aggregates | 55.828ms | 137.862ms | 150.493ms | 0.0% | 150.493ms | 150.493ms | 150.493ms | 150.493ms | 370.000ms | 236.18 MiB | none | 6.64M/s |
| q11 | group by a string and count distinct | 24.581ms | 115.155ms | 112.823ms | 0.0% | 112.823ms | 112.823ms | 112.823ms | 112.823ms | 180.000ms | 102.26 MiB | none | 8.86M/s |
| q12 | group by two strings and count distinct | 25.601ms | 112.457ms | 112.522ms | 0.0% | 112.522ms | 112.522ms | 112.522ms | 112.522ms | 200.000ms | 104.70 MiB | none | 8.89M/s |
| q13 | group by a string and top k | 23.643ms | 117.577ms | 114.121ms | 0.0% | 114.121ms | 114.121ms | 114.121ms | 114.121ms | 200.000ms | 120.96 MiB | none | 8.76M/s |
| q14 | group by a string and count distinct | 38.147ms | 130.251ms | 127.763ms | 0.0% | 127.763ms | 127.763ms | 127.763ms | 127.763ms | 310.000ms | 175.82 MiB | none | 7.83M/s |
| q15 | group by two columns and top k | 26.345ms | 115.814ms | 113.390ms | 0.0% | 113.390ms | 113.390ms | 113.390ms | 113.390ms | 210.000ms | 137.98 MiB | none | 8.82M/s |
| q16 | group by, very high card | 28.710ms | 116.701ms | 121.470ms | 0.0% | 121.470ms | 121.470ms | 121.470ms | 121.470ms | 250.000ms | 156.13 MiB | none | 8.23M/s |
| q17 | group by two, very high card | 42.579ms | 141.991ms | 130.875ms | 0.0% | 130.875ms | 130.875ms | 130.875ms | 130.875ms | 390.000ms | 272.38 MiB | none | 7.64M/s |
| q18 | group by two, no ordering | 35.153ms | 121.413ms | 125.403ms | 0.0% | 125.403ms | 125.403ms | 125.403ms | 125.403ms | 330.000ms | 278.51 MiB | none | 7.97M/s |
| q19 | group by with an extract | 51.367ms | 141.986ms | 140.740ms | 0.0% | 140.740ms | 140.740ms | 140.740ms | 140.740ms | 450.000ms | 296.56 MiB | none | 7.11M/s |
| q20 | point lookup | 8.628ms | 92.356ms | 101.141ms | 0.0% | 101.141ms | 101.141ms | 101.141ms | 101.141ms | 130.000ms | 80.88 MiB | none | 9.89M/s |
| q21 | substring scan | 46.652ms | 134.009ms | 136.795ms | 0.0% | 136.795ms | 136.795ms | 136.795ms | 136.795ms | 290.000ms | 185.60 MiB | none | 7.31M/s |
| q22 | substring scan and group by | 51.007ms | 149.583ms | 145.971ms | 0.0% | 145.971ms | 145.971ms | 145.971ms | 145.971ms | 370.000ms | 212.00 MiB | none | 6.85M/s |
| q23 | two substring scans and group by | 83.545ms | 189.394ms | 186.303ms | 0.0% | 186.303ms | 186.303ms | 186.303ms | 186.303ms | 750.000ms | 473.52 MiB | none | 5.37M/s |
| q24 | select star and top k | 87.387ms | 194.498ms | 187.101ms | 0.0% | 187.101ms | 187.101ms | 187.101ms | 187.101ms | 950.000ms | 509.22 MiB | none | 5.34M/s |
| q25 | top k by a date | 13.712ms | 105.291ms | 103.545ms | 0.0% | 103.545ms | 103.545ms | 103.545ms | 103.545ms | 130.000ms | 104.23 MiB | none | 9.66M/s |
| q26 | top k by a string | 13.912ms | 100.506ms | 104.682ms | 0.0% | 104.682ms | 104.682ms | 104.682ms | 104.682ms | 120.000ms | 90.04 MiB | none | 9.55M/s |
| q27 | top k by two columns | 16.976ms | 104.762ms | 108.822ms | 0.0% | 108.822ms | 108.822ms | 108.822ms | 108.822ms | 130.000ms | 107.64 MiB | none | 9.19M/s |
| q30 | ninety sums over one column | 12.742ms | 101.381ms | 100.903ms | 0.0% | 100.903ms | 100.903ms | 100.903ms | 100.903ms | 120.000ms | 74.53 MiB | none | 9.91M/s |
| q31 | group by two and several aggregates | 30.742ms | 119.013ms | 120.119ms | 0.0% | 120.119ms | 120.119ms | 120.119ms | 120.119ms | 220.000ms | 123.01 MiB | none | 8.32M/s |
| q32 | group by a high card pair | 22.886ms | 114.594ms | 114.139ms | 0.0% | 114.139ms | 114.139ms | 114.139ms | 114.139ms | 180.000ms | 132.64 MiB | none | 8.76M/s |
| q33 | group by a high card pair, unfiltered | 37.745ms | 133.004ms | 133.542ms | 0.0% | 133.542ms | 133.542ms | 133.542ms | 133.542ms | 330.000ms | 289.72 MiB | none | 7.49M/s |
| q34 | group by a long string | 64.679ms | 166.991ms | 168.484ms | 0.0% | 168.484ms | 168.484ms | 168.484ms | 168.484ms | 500.000ms | 448.62 MiB | none | 5.94M/s |
| q35 | group by a constant and a long string | 74.915ms | 180.946ms | 181.456ms | 0.0% | 181.456ms | 181.456ms | 181.456ms | 181.456ms | 590.000ms | 520.05 MiB | none | 5.51M/s |
| q37 | date range and group by a URL | 37.920ms | 129.825ms | 129.474ms | 0.0% | 129.474ms | 129.474ms | 129.474ms | 129.474ms | 180.000ms | 152.82 MiB | none | 7.72M/s |
| q38 | date range and group by a title | 35.511ms | 125.513ms | 126.607ms | 0.0% | 126.607ms | 126.607ms | 126.607ms | 126.607ms | 190.000ms | 143.48 MiB | none | 7.90M/s |
| q39 | date range, group by and offset | 28.897ms | 117.520ms | 119.994ms | 0.0% | 119.994ms | 119.994ms | 119.994ms | 119.994ms | 160.000ms | 111.01 MiB | none | 8.33M/s |
| q40 | date range, a case and a wide group by | 30.835ms | 120.591ms | 123.366ms | 0.0% | 123.366ms | 123.366ms | 123.366ms | 123.366ms | 190.000ms | 137.02 MiB | none | 8.11M/s |
| q41 | date range with an IN and a hash | 15.537ms | 105.346ms | 103.043ms | 0.0% | 103.043ms | 103.043ms | 103.043ms | 103.043ms | 130.000ms | 87.30 MiB | none | 9.70M/s |
| q42 | date range and a deep offset | 15.450ms | 103.253ms | 103.888ms | 0.0% | 103.888ms | 103.888ms | 103.888ms | 103.888ms | 160.000ms | 84.19 MiB | none | 9.63M/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 1.216s by its own clock and 4.795s by ours, 4.801s cold, 10.340s of CPU, peak 520.05 MiB, 32.06M/s and 4.35 GiB/s.

Running it cost 294% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.00x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-server in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 31.103ms | 30.709ms | 0.0% | 30.709ms | 30.709ms | 30.709ms | 30.709ms | not read | not read | not read | 32.56M/s |
| q2 | filtered count | 1.000ms | 30.811ms | 29.833ms | 0.0% | 29.833ms | 29.833ms | 29.833ms | 29.833ms | not read | not read | not read | 33.52M/s |
| q3 | three aggregates | 3.000ms | 33.249ms | 32.524ms | 0.0% | 32.524ms | 32.524ms | 32.524ms | 32.524ms | not read | not read | not read | 30.75M/s |
| q4 | average | 3.000ms | 32.866ms | 32.365ms | 0.0% | 32.365ms | 32.365ms | 32.365ms | 32.365ms | not read | not read | not read | 30.90M/s |
| q5 | count distinct, high card | 12.000ms | 43.541ms | 41.101ms | 0.0% | 41.101ms | 41.101ms | 41.101ms | 41.101ms | not read | not read | not read | 24.33M/s |
| q6 | count distinct, strings | 11.000ms | 39.069ms | 42.654ms | 0.0% | 42.654ms | 42.654ms | 42.654ms | 42.654ms | not read | not read | not read | 23.44M/s |
| q7 | min and max of a date | 1.000ms | 30.624ms | 29.814ms | 0.0% | 29.814ms | 29.814ms | 29.814ms | 29.814ms | not read | not read | not read | 33.54M/s |
| q8 | group by, low card | 3.000ms | 32.282ms | 32.349ms | 0.0% | 32.349ms | 32.349ms | 32.349ms | 32.349ms | not read | not read | not read | 30.91M/s |
| q9 | group by and count distinct | 15.000ms | 47.494ms | 44.186ms | 0.0% | 44.186ms | 44.186ms | 44.186ms | 44.186ms | not read | not read | not read | 22.63M/s |
| q10 | group by, several aggregates | 31.000ms | 44.577ms | 59.966ms | 0.0% | 59.966ms | 59.966ms | 59.966ms | 59.966ms | not read | not read | not read | 16.68M/s |
| q11 | group by a string and count distinct | 5.000ms | 34.833ms | 35.006ms | 0.0% | 35.006ms | 35.006ms | 35.006ms | 35.006ms | not read | not read | not read | 28.57M/s |
| q12 | group by two strings and count distinct | 5.000ms | 34.530ms | 35.414ms | 0.0% | 35.414ms | 35.414ms | 35.414ms | 35.414ms | not read | not read | not read | 28.24M/s |
| q13 | group by a string and top k | 11.000ms | 42.402ms | 40.767ms | 0.0% | 40.767ms | 40.767ms | 40.767ms | 40.767ms | not read | not read | not read | 24.53M/s |
| q14 | group by a string and count distinct | 14.000ms | 43.968ms | 43.552ms | 0.0% | 43.552ms | 43.552ms | 43.552ms | 43.552ms | not read | not read | not read | 22.96M/s |
| q15 | group by two columns and top k | 12.000ms | 41.615ms | 41.513ms | 0.0% | 41.513ms | 41.513ms | 41.513ms | 41.513ms | not read | not read | not read | 24.09M/s |
| q16 | group by, very high card | 10.000ms | 41.762ms | 40.021ms | 0.0% | 40.021ms | 40.021ms | 40.021ms | 40.021ms | not read | not read | not read | 24.99M/s |
| q17 | group by two, very high card | 20.000ms | 51.872ms | 51.026ms | 0.0% | 51.026ms | 51.026ms | 51.026ms | 51.026ms | not read | not read | not read | 19.60M/s |
| q18 | group by two, no ordering | 7.000ms | 37.005ms | 36.644ms | 0.0% | 36.644ms | 36.644ms | 36.644ms | 36.644ms | not read | not read | not read | 27.29M/s |
| q19 | group by with an extract | 23.000ms | 52.646ms | 52.357ms | 0.0% | 52.357ms | 52.357ms | 52.357ms | 52.357ms | not read | not read | not read | 19.10M/s |
| q20 | point lookup | 1.000ms | 31.927ms | 30.608ms | 0.0% | 30.608ms | 30.608ms | 30.608ms | 30.608ms | not read | not read | not read | 32.67M/s |
| q21 | substring scan | 12.000ms | 43.855ms | 41.103ms | 0.0% | 41.103ms | 41.103ms | 41.103ms | 41.103ms | not read | not read | not read | 24.33M/s |
| q22 | substring scan and group by | 8.000ms | 45.497ms | 37.124ms | 0.0% | 37.124ms | 37.124ms | 37.124ms | 37.124ms | not read | not read | not read | 26.94M/s |
| q23 | two substring scans and group by | 14.000ms | 46.837ms | 43.361ms | 0.0% | 43.361ms | 43.361ms | 43.361ms | 43.361ms | not read | not read | not read | 23.06M/s |
| q24 | select star and top k | 20.000ms | 53.747ms | 45.003ms | 0.0% | 45.003ms | 45.003ms | 45.003ms | 45.003ms | not read | not read | not read | 22.22M/s |
| q25 | top k by a date | 4.000ms | 33.586ms | 33.577ms | 0.0% | 33.577ms | 33.577ms | 33.577ms | 33.577ms | not read | not read | not read | 29.78M/s |
| q26 | top k by a string | 6.000ms | 34.056ms | 34.882ms | 0.0% | 34.882ms | 34.882ms | 34.882ms | 34.882ms | not read | not read | not read | 28.67M/s |
| q27 | top k by two columns | 7.000ms | 33.956ms | 36.072ms | 0.0% | 36.072ms | 36.072ms | 36.072ms | 36.072ms | not read | not read | not read | 27.72M/s |
| q28 | group by with a string length | 5.000ms | 34.808ms | 34.022ms | 0.0% | 34.022ms | 34.022ms | 34.022ms | 34.022ms | not read | not read | not read | 29.39M/s |
| q29 | group by a regular expression | 31.000ms | 57.956ms | 61.598ms | 0.0% | 61.598ms | 61.598ms | 61.598ms | 61.598ms | not read | not read | not read | 16.23M/s |
| q30 | ninety sums over one column | 7.000ms | 37.838ms | 37.600ms | 0.0% | 37.600ms | 37.600ms | 37.600ms | 37.600ms | not read | not read | not read | 26.60M/s |
| q31 | group by two and several aggregates | 10.000ms | 40.438ms | 39.314ms | 0.0% | 39.314ms | 39.314ms | 39.314ms | 39.314ms | not read | not read | not read | 25.44M/s |
| q32 | group by a high card pair | 24.000ms | 40.738ms | 53.948ms | 0.0% | 53.948ms | 53.948ms | 53.948ms | 53.948ms | not read | not read | not read | 18.54M/s |
| q33 | group by a high card pair, unfiltered | 14.000ms | 44.471ms | 44.896ms | 0.0% | 44.896ms | 44.896ms | 44.896ms | 44.896ms | not read | not read | not read | 22.27M/s |
| q34 | group by a long string | 27.000ms | 57.626ms | 57.182ms | 0.0% | 57.182ms | 57.182ms | 57.182ms | 57.182ms | not read | not read | not read | 17.49M/s |
| q35 | group by a constant and a long string | 35.000ms | 56.229ms | 68.647ms | 0.0% | 68.647ms | 68.647ms | 68.647ms | 68.647ms | not read | not read | not read | 14.57M/s |
| q36 | group by four expressions | 9.000ms | 39.390ms | 38.901ms | 0.0% | 38.901ms | 38.901ms | 38.901ms | 38.901ms | not read | not read | not read | 25.71M/s |
| q37 | date range and group by a URL | 4.000ms | 34.382ms | 34.547ms | 0.0% | 34.547ms | 34.547ms | 34.547ms | 34.547ms | not read | not read | not read | 28.95M/s |
| q38 | date range and group by a title | 4.000ms | 34.413ms | 36.188ms | 0.0% | 36.188ms | 36.188ms | 36.188ms | 36.188ms | not read | not read | not read | 27.63M/s |
| q39 | date range, group by and offset | 4.000ms | 33.863ms | 33.740ms | 0.0% | 33.740ms | 33.740ms | 33.740ms | 33.740ms | not read | not read | not read | 29.64M/s |
| q40 | date range, a case and a wide group by | 6.000ms | 35.506ms | 34.770ms | 0.0% | 34.770ms | 34.770ms | 34.770ms | 34.770ms | not read | not read | not read | 28.76M/s |
| q41 | date range with an IN and a hash | 3.000ms | 32.287ms | 32.883ms | 0.0% | 32.883ms | 32.883ms | 32.883ms | 32.883ms | not read | not read | not read | 30.41M/s |
| q42 | date range and a deep offset | 3.000ms | 35.716ms | 32.784ms | 0.0% | 32.784ms | 32.784ms | 32.784ms | 32.784ms | not read | not read | not read | 30.50M/s |
| q43 | minute buckets over a date range | 3.000ms | 32.248ms | 31.666ms | 0.0% | 31.666ms | 31.666ms | 31.666ms | 31.666ms | not read | not read | not read | 31.58M/s |

clickhouse-server 26.9.1.1162 over 43 of 43 queries. Total 449.000ms by its own clock and 1.726s by ours, 1.718s cold, no reading of CPU, peak not read, 95.77M/s and 12.99 GiB/s.

Running it cost 284% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.30x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 2.438ms | 2.370ms | 0.0% | 2.370ms | 2.370ms | 2.370ms | 2.370ms | 0.000us | 4.49 MiB | none | 421.93M/s |
| q2 | filtered count | 5.000ms | 6.494ms | 6.599ms | 0.0% | 6.599ms | 6.599ms | 6.599ms | 6.599ms | 0.000us | 6.09 MiB | none | 151.53M/s |
| q3 | three aggregates | 15.000ms | 16.493ms | 16.993ms | 0.0% | 16.993ms | 16.993ms | 16.993ms | 16.993ms | 10.000ms | 7.49 MiB | none | 58.85M/s |
| q4 | average | 13.000ms | 14.774ms | 14.482ms | 0.0% | 14.482ms | 14.482ms | 14.482ms | 14.482ms | 10.000ms | 9.01 MiB | none | 69.05M/s |
| q5 | count distinct, high card | 213.000ms | 215.732ms | 217.347ms | 0.0% | 217.347ms | 217.347ms | 217.347ms | 217.347ms | 210.000ms | 98.11 MiB | none | 4.60M/s |
| q6 | count distinct, strings | 85.000ms | 87.584ms | 86.908ms | 0.0% | 86.908ms | 86.908ms | 86.908ms | 86.908ms | 70.000ms | 25.29 MiB | none | 11.51M/s |
| q7 | min and max of a date | 6.000ms | 8.094ms | 7.918ms | 0.0% | 7.918ms | 7.918ms | 7.918ms | 7.918ms | 0.000us | 6.09 MiB | none | 126.29M/s |
| q8 | group by, low card | 5.000ms | 7.281ms | 6.812ms | 0.0% | 6.812ms | 6.812ms | 6.812ms | 6.812ms | 0.000us | 6.14 MiB | none | 146.80M/s |
| q9 | group by and count distinct | 260.000ms | 269.300ms | 264.852ms | 0.0% | 264.852ms | 264.852ms | 264.852ms | 264.852ms | 260.000ms | 110.86 MiB | none | 3.78M/s |
| q10 | group by, several aggregates | 287.000ms | 289.278ms | 292.391ms | 0.0% | 292.391ms | 292.391ms | 292.391ms | 292.391ms | 280.000ms | 112.82 MiB | none | 3.42M/s |
| q11 | group by a string and count distinct | 29.000ms | 30.264ms | 30.550ms | 0.0% | 30.550ms | 30.550ms | 30.550ms | 30.550ms | 20.000ms | 15.25 MiB | none | 32.73M/s |
| q12 | group by two strings and count distinct | 34.000ms | 36.375ms | 36.347ms | 0.0% | 36.347ms | 36.347ms | 36.347ms | 36.347ms | 30.000ms | 16.40 MiB | none | 27.51M/s |
| q13 | group by a string and top k | 130.000ms | 130.197ms | 132.564ms | 0.0% | 132.564ms | 132.564ms | 132.564ms | 132.564ms | 120.000ms | 42.38 MiB | none | 7.54M/s |
| q14 | group by a string and count distinct | 162.000ms | 162.833ms | 164.464ms | 0.0% | 164.464ms | 164.464ms | 164.464ms | 164.464ms | 150.000ms | 62.05 MiB | none | 6.08M/s |
| q15 | group by two columns and top k | 145.000ms | 146.509ms | 147.184ms | 0.0% | 147.184ms | 147.184ms | 147.184ms | 147.184ms | 140.000ms | 50.30 MiB | none | 6.79M/s |
| q16 | group by, very high card | 219.000ms | 217.644ms | 221.515ms | 0.0% | 221.515ms | 221.515ms | 221.515ms | 221.515ms | 210.000ms | 184.17 MiB | none | 4.51M/s |
| q17 | group by two, very high card | 367.000ms | 372.773ms | 371.131ms | 0.0% | 371.131ms | 371.131ms | 371.131ms | 371.131ms | 360.000ms | 271.62 MiB | none | 2.69M/s |
| q18 | group by two, no ordering | 285.000ms | 282.141ms | 288.482ms | 0.0% | 288.482ms | 288.482ms | 288.482ms | 288.482ms | 280.000ms | 271.43 MiB | none | 3.47M/s |
| q20 | point lookup | 13.000ms | 14.521ms | 14.376ms | 0.0% | 14.376ms | 14.376ms | 14.376ms | 14.376ms | 0.000us | 9.15 MiB | none | 69.56M/s |
| q21 | substring scan | 282.000ms | 294.481ms | 287.430ms | 0.0% | 287.430ms | 287.430ms | 287.430ms | 287.430ms | 280.000ms | 43.46 MiB | none | 3.48M/s |
| q22 | substring scan and group by | 317.000ms | 328.130ms | 320.634ms | 0.0% | 320.634ms | 320.634ms | 320.634ms | 320.634ms | 310.000ms | 46.63 MiB | none | 3.12M/s |
| q23 | two substring scans and group by | 704.000ms | 748.013ms | 706.861ms | 0.0% | 706.861ms | 706.861ms | 706.861ms | 706.861ms | 700.000ms | 69.15 MiB | none | 1.41M/s |
| q24 | select star and top k | 1.532s | 1.545s | 1.540s | 0.0% | 1.540s | 1.540s | 1.540s | 1.540s | 1.530s | 213.48 MiB | none | 649.35K/s |
| q25 | top k by a date | 122.000ms | 127.311ms | 124.227ms | 0.0% | 124.227ms | 124.227ms | 124.227ms | 124.227ms | 110.000ms | 12.95 MiB | none | 8.05M/s |
| q26 | top k by a string | 108.000ms | 109.990ms | 110.436ms | 0.0% | 110.436ms | 110.436ms | 110.436ms | 110.436ms | 100.000ms | 11.02 MiB | none | 9.05M/s |
| q27 | top k by two columns | 137.000ms | 141.870ms | 139.372ms | 0.0% | 139.372ms | 139.372ms | 139.372ms | 139.372ms | 130.000ms | 13.18 MiB | none | 7.17M/s |
| q28 | group by with a string length | 282.000ms | 280.835ms | 285.419ms | 0.0% | 285.419ms | 285.419ms | 285.419ms | 285.419ms | 270.000ms | 45.66 MiB | none | 3.50M/s |
| q29 | group by a regular expression | 803.000ms | 804.980ms | 806.384ms | 0.0% | 806.384ms | 806.384ms | 806.384ms | 806.384ms | 790.000ms | 97.17 MiB | none | 1.24M/s |
| q30 | ninety sums over one column | 128.000ms | 129.006ms | 130.157ms | 0.0% | 130.157ms | 130.157ms | 130.157ms | 130.157ms | 120.000ms | 6.79 MiB | none | 7.68M/s |
| q31 | group by two and several aggregates | 151.000ms | 150.931ms | 153.384ms | 0.0% | 153.384ms | 153.384ms | 153.384ms | 153.384ms | 150.000ms | 69.00 MiB | none | 6.52M/s |
| q32 | group by a high card pair | 154.000ms | 151.243ms | 156.437ms | 0.0% | 156.437ms | 156.437ms | 156.437ms | 156.437ms | 140.000ms | 74.79 MiB | none | 6.39M/s |
| q34 | group by a long string | 521.000ms | 516.349ms | 526.849ms | 0.0% | 526.849ms | 526.849ms | 526.849ms | 526.849ms | 510.000ms | 253.63 MiB | none | 1.90M/s |
| q35 | group by a constant and a long string | 543.000ms | 565.347ms | 549.589ms | 0.0% | 549.589ms | 549.589ms | 549.589ms | 549.589ms | 540.000ms | 286.17 MiB | none | 1.82M/s |
| q36 | group by four expressions | 343.000ms | 329.975ms | 345.867ms | 0.0% | 345.867ms | 345.867ms | 345.867ms | 345.867ms | 330.000ms | 275.47 MiB | none | 2.89M/s |
| q37 | date range and group by a URL | 263.000ms | 259.434ms | 266.868ms | 0.0% | 266.868ms | 266.868ms | 266.868ms | 266.868ms | 250.000ms | 52.30 MiB | none | 3.75M/s |
| q38 | date range and group by a title | 283.000ms | 286.130ms | 285.656ms | 0.0% | 285.656ms | 285.656ms | 285.656ms | 285.656ms | 270.000ms | 40.05 MiB | none | 3.50M/s |
| q39 | date range, group by and offset | 263.000ms | 265.581ms | 266.275ms | 0.0% | 266.275ms | 266.275ms | 266.275ms | 266.275ms | 250.000ms | 48.86 MiB | none | 3.76M/s |
| q40 | date range, a case and a wide group by | 503.000ms | 515.117ms | 505.666ms | 0.0% | 505.666ms | 505.666ms | 505.666ms | 505.666ms | 500.000ms | 72.75 MiB | none | 1.98M/s |
| q41 | date range with an IN and a hash | 48.000ms | 52.515ms | 50.127ms | 0.0% | 50.127ms | 50.127ms | 50.127ms | 50.127ms | 40.000ms | 14.73 MiB | none | 19.95M/s |
| q42 | date range and a deep offset | 52.000ms | 53.749ms | 53.486ms | 0.0% | 53.486ms | 53.486ms | 53.486ms | 53.486ms | 50.000ms | 14.90 MiB | none | 18.70M/s |
| q43 | minute buckets over a date range | 41.000ms | 43.632ms | 42.764ms | 0.0% | 42.764ms | 42.764ms | 42.764ms | 42.764ms | 40.000ms | 12.19 MiB | none | 23.38M/s |

rudb rudb 0.2.31 over 41 of 43 queries. Total 9.854s by its own clock and 9.977s by ours, 10.010s cold, 9.560s of CPU, peak 286.17 MiB, 4.16M/s and 577.73 MiB/s.

Running it cost 1% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 649.77x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 999975 rows, one out of every 100 of the 99997497 in the full file, which is a development loop rather than the suite
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

- polars ran every query within 2.00x of every other one, and this suite spreads over 6x on an engine it is measuring

A column that flat is not a column about the queries. Read those numbers as an upper bound on the engine and not as a measurement of it.

polars did not run q28, because the SQLContext has no strlen and no character length function.

polars did not run q29, because the SQLContext has no regexp_replace.

polars did not run q36, because the SQLContext rejects the repeated ClientIP output name in the group by.

polars did not run q43, because the SQLContext has no date_trunc.

So the polars column is 39 of 43 queries and its ratio is over the shared ones.

rudb did not run q19, because rudb's hash aggregate does not spill and this groups by a key that is close to unique over a hundred million rows, so the table outgrows the machine. tamnd/rudb#220, milestone E5.

rudb did not run q33, because rudb's hash aggregate does not spill and this groups by a key that is close to unique over a hundred million rows, so the table outgrows the machine. tamnd/rudb#220, milestone E5.

So the rudb column is 41 of 43 queries and its ratio is over the shared ones.

Answers differ, so this is not a comparison: q23: duckdb-pinned does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q23: clickhouse-local does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q23: datafusion does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q23: polars does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q23: clickhouse-server does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q23: rudb does not agree with duckdb: 20 numbers against 20

These were answered differently and the data does not say which is right:

- q16: ORDER BY COUNT(*) DESC LIMIT 10 over user IDs, where the tenth place is a tie between many users with the same count and the engine picks.
- q17: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q18: LIMIT 10 with no ORDER BY at all, so the ten rows are whichever ten the group by handed back first and the query did not ask for any particular ten.
- q19: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
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

Hot here means page cache warm and not buffer pool warm for every row but clickhouse-server, which stayed up across the whole suite with its own caches warm. That is the stronger kind of hot, so a ratio against that column is a ratio between two different quantities.

