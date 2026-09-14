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
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 3.746s | 15.100s | 567.76 MiB | its own database file | its own | 6.14 to 9.98 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 3.731s | 14.860s | 501.26 MiB | its own database file | its own | 9.98 to 13.46 |
| clickhouse-local | 26.9.1.1162 | ran | 1.445s | 3.780s | 230.59 MiB | its own MergeTree parts, as system.parts counts the active ones | its own | 25.26 to 27.41 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 138.85 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 19.61 to 21.84 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 138.85 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 21.84 to 25.26 |
| clickhouse-server | 26.9.1.1162 | ran | 5.643s | not read | 143.71 MiB | its own MergeTree parts, sorted by hits by (CounterID, EventDate, UserID, EventTime, WatchID), as system.parts counts the active ones | its own | 27.41 to 25.35 |
| rudb | rudb 0.2.31 | ran | 0.000us | 0.000us | 138.85 MiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 13.46 to 19.61 |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

These engines started while the machine was already above half of that, which means they were measured against somebody else's work rather than on an idle box: clickhouse-local, datafusion, polars, clickhouse-server. Their numbers are inflated by an amount nothing here can recover, and by a different amount each, depending on how much of the column was CPU bound. One case where that reading is unfair is a sweep that runs engines back to back, where the average has not had a minute to come down from the engine before.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 1.879s | 2.825s | +50% | 2.665s | 6.310s | 2.23 | 265.56 MiB | 1.00 MiB | 22.88M/s | 3.10 GiB/s | 1.00x |
| duckdb-pinned | 1.831s | 3.752s | +105% | 3.762s | 6.810s | 1.82 | 246.53 MiB | none | 23.48M/s | 3.18 GiB/s | 0.97x |
| clickhouse-local | 5.582s | 10.488s | +88% | 10.385s | 10.850s | 1.03 | 398.46 MiB | 4.36 MiB | 7.70M/s | 1.04 GiB/s | 2.97x |
| datafusion | 1.952s | 3.119s | +60% | 3.211s | 7.890s | 2.53 | 982.70 MiB | none | 22.03M/s | 2.99 GiB/s | 1.04x |
| polars | 2.623s | 10.591s | +304% | 10.518s | 15.430s | 1.46 | 535.10 MiB | none | 14.87M/s | 2.02 GiB/s | 1.40x |
| clickhouse-server | 670.000ms | 2.332s | +248% | 2.352s | not read | not read | not read | not read | 64.18M/s | 8.70 GiB/s | 0.36x |
| rudb | 17.694s | 17.929s | +1% | 17.876s | 17.480s | 0.97 | 286.15 MiB | none | 2.32M/s | 321.74 MiB/s | 9.42x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | clickhouse-server | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 0.000us | 1.000ms | 5.000ms | 2.000ms | 12.679ms | 1.000ms | 1.000ms |
| q2 | filtered count | 3.000ms | 4.000ms | 5.000ms | 10.000ms | 12.862ms | 1.000ms | 10.000ms |
| q3 | three aggregates | 7.000ms | 9.000ms | 118.000ms | 13.000ms | 14.015ms | 4.000ms | 31.000ms |
| q4 | average | 5.000ms | 9.000ms | 46.000ms | 9.000ms | 18.741ms | 3.000ms | 31.000ms |
| q5 | count distinct, high card | 40.000ms | 32.000ms | 141.000ms | 25.000ms | 50.295ms | 22.000ms | 401.000ms |
| q6 | count distinct, strings | 35.000ms | 29.000ms | 172.000ms | 43.000ms | 44.514ms | 21.000ms | 173.000ms |
| q7 | min and max of a date | 1.000ms | 1.000ms | 14.000ms | 3.000ms | 11.487ms | 2.000ms | 14.000ms |
| q8 | group by, low card | 3.000ms | 16.000ms | 102.000ms | 13.000ms | 20.145ms | 4.000ms | 11.000ms |
| q9 | group by and count distinct | 56.000ms | 46.000ms | 102.000ms | 43.000ms | 94.805ms | 22.000ms | 608.000ms |
| q10 | group by, several aggregates | 61.000ms | 58.000ms | 164.000ms | 43.000ms | 104.026ms | 35.000ms | 576.000ms |
| q11 | group by a string and count distinct | 31.000ms | 18.000ms | 150.000ms | 24.000ms | 37.396ms | 6.000ms | 50.000ms |
| q12 | group by two strings and count distinct | 34.000ms | 24.000ms | 122.000ms | 25.000ms | 37.652ms | 7.000ms | 73.000ms |
| q13 | group by a string and top k | 27.000ms | 32.000ms | 111.000ms | 35.000ms | 46.616ms | 21.000ms | 207.000ms |
| q14 | group by a string and count distinct | 44.000ms | 57.000ms | 124.000ms | 47.000ms | 92.880ms | 16.000ms | 267.000ms |
| q15 | group by two columns and top k | 32.000ms | 33.000ms | 117.000ms | 38.000ms | 47.703ms | 19.000ms | 250.000ms |
| q16 | group by, very high card | 40.000ms | 46.000ms | 75.000ms | 31.000ms | 69.711ms | 18.000ms | 389.000ms |
| q17 | group by two, very high card | 68.000ms | 70.000ms | 202.000ms | 86.000ms | 108.841ms | 42.000ms | 691.000ms |
| q18 | group by two, no ordering | 67.000ms | 50.000ms | 122.000ms | 75.000ms | 118.132ms | 11.000ms | 530.000ms |
| q19 | group by with an extract | 78.000ms | 69.000ms | 181.000ms | 72.000ms | 119.232ms | 38.000ms | no dialect |
| q20 | point lookup | 4.000ms | 8.000ms | 82.000ms | 8.000ms | 14.152ms | 1.000ms | 23.000ms |
| q21 | substring scan | 45.000ms | 30.000ms | 149.000ms | 44.000ms | 95.956ms | 17.000ms | 458.000ms |
| q22 | substring scan and group by | 56.000ms | 41.000ms | 177.000ms | 64.000ms | 119.105ms | 8.000ms | 566.000ms |
| q23 | two substring scans and group by | 77.000ms | 73.000ms | 230.000ms | 113.000ms | 176.025ms | 17.000ms | 1.092s |
| q24 | select star and top k | 127.000ms | 105.000ms | 339.000ms | 231.000ms | 155.880ms | 39.000ms | 2.649s |
| q25 | top k by a date | 18.000ms | 14.000ms | 129.000ms | 27.000ms | 31.468ms | 6.000ms | 220.000ms |
| q26 | top k by a string | 16.000ms | 9.000ms | 140.000ms | 26.000ms | 36.886ms | 8.000ms | 187.000ms |
| q27 | top k by two columns | 10.000ms | 14.000ms | 84.000ms | 27.000ms | 35.207ms | 9.000ms | 235.000ms |
| q28 | group by with a string length | 60.000ms | 57.000ms | 62.000ms | 59.000ms | no dialect | 7.000ms | 497.000ms |
| q29 | group by a regular expression | 344.000ms | 223.000ms | 173.000ms | 109.000ms | no dialect | 49.000ms | 1.461s |
| q30 | ninety sums over one column | 11.000ms | 36.000ms | 70.000ms | 29.000ms | 38.212ms | 8.000ms | 238.000ms |
| q31 | group by two and several aggregates | 36.000ms | 44.000ms | 129.000ms | 36.000ms | 59.919ms | 14.000ms | 275.000ms |
| q32 | group by a high card pair | 34.000ms | 62.000ms | 134.000ms | 36.000ms | 46.969ms | 24.000ms | 279.000ms |
| q33 | group by a high card pair, unfiltered | 71.000ms | 108.000ms | 191.000ms | 60.000ms | 116.992ms | 20.000ms | no dialect |
| q34 | group by a long string | 109.000ms | 121.000ms | 201.000ms | 90.000ms | 171.550ms | 47.000ms | 939.000ms |
| q35 | group by a constant and a long string | 117.000ms | 168.000ms | 163.000ms | 99.000ms | 198.744ms | 53.000ms | 1.064s |
| q36 | group by four expressions | 47.000ms | 33.000ms | 146.000ms | 35.000ms | no dialect | 15.000ms | 660.000ms |
| q37 | date range and group by a URL | 10.000ms | 10.000ms | 143.000ms | 38.000ms | 61.123ms | 5.000ms | 497.000ms |
| q38 | date range and group by a title | 12.000ms | 9.000ms | 123.000ms | 48.000ms | 53.693ms | 5.000ms | 454.000ms |
| q39 | date range, group by and offset | 10.000ms | 12.000ms | 140.000ms | 33.000ms | 46.557ms | 6.000ms | 481.000ms |
| q40 | date range, a case and a wide group by | 8.000ms | 12.000ms | 105.000ms | 51.000ms | 55.310ms | 7.000ms | 835.000ms |
| q41 | date range with an IN and a hash | 5.000ms | 7.000ms | 137.000ms | 17.000ms | 25.868ms | 4.000ms | 106.000ms |
| q42 | date range and a deep offset | 11.000ms | 22.000ms | 129.000ms | 19.000ms | 21.766ms | 4.000ms | 91.000ms |
| q43 | minute buckets over a date range | 9.000ms | 9.000ms | 133.000ms | 16.000ms | no dialect | 4.000ms | 74.000ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 0.000us | 20.940ms | 20.801ms | 0.0% | 20.801ms | 20.801ms | 20.801ms | 20.801ms | 10.000ms | 26.98 MiB | none | 48.07M/s |
| q2 | filtered count | 3.000ms | 24.896ms | 22.213ms | 0.0% | 22.213ms | 22.213ms | 22.213ms | 22.213ms | 20.000ms | 31.79 MiB | 92.00 KiB | 45.02M/s |
| q3 | three aggregates | 7.000ms | 26.140ms | 27.052ms | 0.0% | 27.052ms | 27.052ms | 27.052ms | 27.052ms | 30.000ms | 34.79 MiB | 960.00 KiB | 36.96M/s |
| q4 | average | 5.000ms | 25.552ms | 26.006ms | 0.0% | 26.006ms | 26.006ms | 26.006ms | 26.006ms | 30.000ms | 38.07 MiB | none | 38.45M/s |
| q5 | count distinct, high card | 40.000ms | 50.954ms | 59.045ms | 0.0% | 59.045ms | 59.045ms | 59.045ms | 59.045ms | 130.000ms | 73.54 MiB | none | 16.94M/s |
| q6 | count distinct, strings | 35.000ms | 59.364ms | 55.859ms | 0.0% | 55.859ms | 55.859ms | 55.859ms | 55.859ms | 110.000ms | 63.53 MiB | none | 17.90M/s |
| q7 | min and max of a date | 1.000ms | 22.084ms | 22.306ms | 0.0% | 22.306ms | 22.306ms | 22.306ms | 22.306ms | 10.000ms | 27.73 MiB | none | 44.83M/s |
| q8 | group by, low card | 3.000ms | 23.463ms | 24.491ms | 0.0% | 24.491ms | 24.491ms | 24.491ms | 24.491ms | 20.000ms | 33.75 MiB | none | 40.83M/s |
| q9 | group by and count distinct | 56.000ms | 72.980ms | 73.178ms | 0.0% | 73.178ms | 73.178ms | 73.178ms | 73.178ms | 190.000ms | 98.75 MiB | none | 13.66M/s |
| q10 | group by, several aggregates | 61.000ms | 86.029ms | 84.851ms | 0.0% | 84.851ms | 84.851ms | 84.851ms | 84.851ms | 210.000ms | 105.31 MiB | none | 11.79M/s |
| q11 | group by a string and count distinct | 31.000ms | 42.499ms | 52.347ms | 0.0% | 52.347ms | 52.347ms | 52.347ms | 52.347ms | 80.000ms | 62.12 MiB | none | 19.10M/s |
| q12 | group by two strings and count distinct | 34.000ms | 51.180ms | 55.885ms | 0.0% | 55.885ms | 55.885ms | 55.885ms | 55.885ms | 90.000ms | 65.06 MiB | none | 17.89M/s |
| q13 | group by a string and top k | 27.000ms | 41.296ms | 49.578ms | 0.0% | 49.578ms | 49.578ms | 49.578ms | 49.578ms | 80.000ms | 65.00 MiB | none | 20.17M/s |
| q14 | group by a string and count distinct | 44.000ms | 67.035ms | 67.889ms | 0.0% | 67.889ms | 67.889ms | 67.889ms | 67.889ms | 140.000ms | 110.81 MiB | none | 14.73M/s |
| q15 | group by two columns and top k | 32.000ms | 47.476ms | 53.762ms | 0.0% | 53.762ms | 53.762ms | 53.762ms | 53.762ms | 90.000ms | 69.06 MiB | none | 18.60M/s |
| q16 | group by, very high card | 40.000ms | 72.829ms | 63.138ms | 0.0% | 63.138ms | 63.138ms | 63.138ms | 63.138ms | 140.000ms | 91.56 MiB | none | 15.84M/s |
| q17 | group by two, very high card | 68.000ms | 71.875ms | 93.360ms | 0.0% | 93.360ms | 93.360ms | 93.360ms | 93.360ms | 230.000ms | 144.23 MiB | none | 10.71M/s |
| q18 | group by two, no ordering | 67.000ms | 104.113ms | 92.331ms | 0.0% | 92.331ms | 92.331ms | 92.331ms | 92.331ms | 240.000ms | 159.59 MiB | none | 10.83M/s |
| q19 | group by with an extract | 78.000ms | 94.993ms | 104.650ms | 0.0% | 104.650ms | 104.650ms | 104.650ms | 104.650ms | 280.000ms | 162.13 MiB | none | 9.56M/s |
| q20 | point lookup | 4.000ms | 21.008ms | 22.869ms | 0.0% | 22.869ms | 22.869ms | 22.869ms | 22.869ms | 30.000ms | 37.78 MiB | none | 43.73M/s |
| q21 | substring scan | 45.000ms | 61.512ms | 67.100ms | 0.0% | 67.100ms | 67.100ms | 67.100ms | 67.100ms | 170.000ms | 95.39 MiB | none | 14.90M/s |
| q22 | substring scan and group by | 56.000ms | 85.078ms | 80.635ms | 0.0% | 80.635ms | 80.635ms | 80.635ms | 80.635ms | 170.000ms | 109.56 MiB | none | 12.40M/s |
| q23 | two substring scans and group by | 77.000ms | 85.148ms | 101.189ms | 0.0% | 101.189ms | 101.189ms | 101.189ms | 101.189ms | 230.000ms | 131.11 MiB | 20.74 MiB | 9.88M/s |
| q24 | select star and top k | 127.000ms | 149.778ms | 155.467ms | 0.0% | 155.467ms | 155.467ms | 155.467ms | 155.467ms | 410.000ms | 212.13 MiB | 4.75 MiB | 6.43M/s |
| q25 | top k by a date | 18.000ms | 43.163ms | 40.184ms | 0.0% | 40.184ms | 40.184ms | 40.184ms | 40.184ms | 50.000ms | 49.68 MiB | none | 24.88M/s |
| q26 | top k by a string | 16.000ms | 31.479ms | 36.751ms | 0.0% | 36.751ms | 36.751ms | 36.751ms | 36.751ms | 50.000ms | 39.04 MiB | none | 27.21M/s |
| q27 | top k by two columns | 10.000ms | 31.546ms | 30.197ms | 0.0% | 30.197ms | 30.197ms | 30.197ms | 30.197ms | 40.000ms | 41.29 MiB | none | 33.12M/s |
| q28 | group by with a string length | 60.000ms | 76.259ms | 82.645ms | 0.0% | 82.645ms | 82.645ms | 82.645ms | 82.645ms | 220.000ms | 106.81 MiB | none | 12.10M/s |
| q29 | group by a regular expression | 344.000ms | 277.201ms | 374.584ms | 0.0% | 374.584ms | 374.584ms | 374.584ms | 374.584ms | 1.110s | 157.73 MiB | 4.00 MiB | 2.67M/s |
| q30 | ninety sums over one column | 11.000ms | 35.769ms | 31.375ms | 0.0% | 31.375ms | 31.375ms | 31.375ms | 31.375ms | 30.000ms | 35.29 MiB | none | 31.87M/s |
| q31 | group by two and several aggregates | 36.000ms | 53.598ms | 58.698ms | 0.0% | 58.698ms | 58.698ms | 58.698ms | 58.698ms | 100.000ms | 72.99 MiB | none | 17.04M/s |
| q32 | group by a high card pair | 34.000ms | 60.945ms | 54.613ms | 0.0% | 54.613ms | 54.613ms | 54.613ms | 54.613ms | 110.000ms | 82.35 MiB | 256.00 KiB | 18.31M/s |
| q33 | group by a high card pair, unfiltered | 71.000ms | 92.997ms | 90.454ms | 0.0% | 90.454ms | 90.454ms | 90.454ms | 90.454ms | 230.000ms | 177.38 MiB | none | 11.06M/s |
| q34 | group by a long string | 109.000ms | 136.385ms | 133.021ms | 0.0% | 133.021ms | 133.021ms | 133.021ms | 133.021ms | 440.000ms | 265.56 MiB | none | 7.52M/s |
| q35 | group by a constant and a long string | 117.000ms | 133.838ms | 144.124ms | 0.0% | 144.124ms | 144.124ms | 144.124ms | 144.124ms | 400.000ms | 247.50 MiB | none | 6.94M/s |
| q36 | group by four expressions | 47.000ms | 68.368ms | 69.399ms | 0.0% | 69.399ms | 69.399ms | 69.399ms | 69.399ms | 170.000ms | 91.80 MiB | none | 14.41M/s |
| q37 | date range and group by a URL | 10.000ms | 30.644ms | 28.948ms | 0.0% | 28.948ms | 28.948ms | 28.948ms | 28.948ms | 30.000ms | 39.30 MiB | none | 34.54M/s |
| q38 | date range and group by a title | 12.000ms | 28.309ms | 30.769ms | 0.0% | 30.769ms | 30.769ms | 30.769ms | 30.769ms | 30.000ms | 37.86 MiB | none | 32.50M/s |
| q39 | date range, group by and offset | 10.000ms | 36.139ms | 28.514ms | 0.0% | 28.514ms | 28.514ms | 28.514ms | 28.514ms | 30.000ms | 36.50 MiB | none | 35.07M/s |
| q40 | date range, a case and a wide group by | 8.000ms | 31.342ms | 26.641ms | 0.0% | 26.641ms | 26.641ms | 26.641ms | 26.641ms | 30.000ms | 45.84 MiB | 500.00 KiB | 37.54M/s |
| q41 | date range with an IN and a hash | 5.000ms | 27.510ms | 22.157ms | 0.0% | 22.157ms | 22.157ms | 22.157ms | 22.157ms | 20.000ms | 38.66 MiB | none | 45.13M/s |
| q42 | date range and a deep offset | 11.000ms | 29.922ms | 31.386ms | 0.0% | 31.386ms | 31.386ms | 31.386ms | 31.386ms | 40.000ms | 37.62 MiB | none | 31.86M/s |
| q43 | minute buckets over a date range | 9.000ms | 31.714ms | 34.747ms | 0.0% | 34.747ms | 34.747ms | 34.747ms | 34.747ms | 40.000ms | 36.30 MiB | none | 28.78M/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 1.879s by its own clock and 2.825s by ours, 2.665s cold, 6.310s of CPU, peak 265.56 MiB, 22.88M/s and 3.10 GiB/s.

Running it cost 50% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 18.01x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 43.604ms | 43.115ms | 0.0% | 43.115ms | 43.115ms | 43.115ms | 43.115ms | 40.000ms | 40.54 MiB | none | 23.19M/s |
| q2 | filtered count | 4.000ms | 39.584ms | 50.588ms | 0.0% | 50.588ms | 50.588ms | 50.588ms | 50.588ms | 40.000ms | 43.32 MiB | none | 19.77M/s |
| q3 | three aggregates | 9.000ms | 52.078ms | 55.904ms | 0.0% | 55.904ms | 55.904ms | 55.904ms | 55.904ms | 50.000ms | 46.97 MiB | none | 17.89M/s |
| q4 | average | 9.000ms | 50.838ms | 51.340ms | 0.0% | 51.340ms | 51.340ms | 51.340ms | 51.340ms | 50.000ms | 50.97 MiB | 1.50 MiB | 19.48M/s |
| q5 | count distinct, high card | 32.000ms | 73.678ms | 71.133ms | 0.0% | 71.133ms | 71.133ms | 71.133ms | 71.133ms | 130.000ms | 92.53 MiB | 32.00 KiB | 14.06M/s |
| q6 | count distinct, strings | 29.000ms | 67.797ms | 78.415ms | 0.0% | 78.415ms | 78.415ms | 78.415ms | 78.415ms | 110.000ms | 76.92 MiB | 512.00 KiB | 12.75M/s |
| q7 | min and max of a date | 1.000ms | 43.954ms | 43.075ms | 0.0% | 43.075ms | 43.075ms | 43.075ms | 43.075ms | 50.000ms | 41.26 MiB | none | 23.21M/s |
| q8 | group by, low card | 16.000ms | 59.743ms | 55.745ms | 0.0% | 55.745ms | 55.745ms | 55.745ms | 55.745ms | 60.000ms | 47.16 MiB | none | 17.94M/s |
| q9 | group by and count distinct | 46.000ms | 85.457ms | 87.288ms | 0.0% | 87.288ms | 87.288ms | 87.288ms | 87.288ms | 150.000ms | 110.52 MiB | 256.00 KiB | 11.46M/s |
| q10 | group by, several aggregates | 58.000ms | 114.218ms | 100.780ms | 0.0% | 100.780ms | 100.780ms | 100.780ms | 100.780ms | 190.000ms | 116.09 MiB | 72.00 KiB | 9.92M/s |
| q11 | group by a string and count distinct | 18.000ms | 59.564ms | 54.687ms | 0.0% | 54.687ms | 54.687ms | 54.687ms | 54.687ms | 80.000ms | 72.78 MiB | none | 18.29M/s |
| q12 | group by two strings and count distinct | 24.000ms | 72.082ms | 68.539ms | 0.0% | 68.539ms | 68.539ms | 68.539ms | 68.539ms | 90.000ms | 77.31 MiB | none | 14.59M/s |
| q13 | group by a string and top k | 32.000ms | 65.453ms | 80.904ms | 0.0% | 80.904ms | 80.904ms | 80.904ms | 80.904ms | 110.000ms | 79.84 MiB | none | 12.36M/s |
| q14 | group by a string and count distinct | 57.000ms | 98.254ms | 105.035ms | 0.0% | 105.035ms | 105.035ms | 105.035ms | 105.035ms | 160.000ms | 125.62 MiB | none | 9.52M/s |
| q15 | group by two columns and top k | 33.000ms | 81.148ms | 78.690ms | 0.0% | 78.690ms | 78.690ms | 78.690ms | 78.690ms | 110.000ms | 84.78 MiB | none | 12.71M/s |
| q16 | group by, very high card | 46.000ms | 80.614ms | 83.283ms | 0.0% | 83.283ms | 83.283ms | 83.283ms | 83.283ms | 150.000ms | 98.82 MiB | none | 12.01M/s |
| q17 | group by two, very high card | 70.000ms | 106.008ms | 124.714ms | 0.0% | 124.714ms | 124.714ms | 124.714ms | 124.714ms | 290.000ms | 161.15 MiB | none | 8.02M/s |
| q18 | group by two, no ordering | 50.000ms | 119.633ms | 87.299ms | 0.0% | 87.299ms | 87.299ms | 87.299ms | 87.299ms | 230.000ms | 171.33 MiB | none | 11.45M/s |
| q19 | group by with an extract | 69.000ms | 117.901ms | 112.935ms | 0.0% | 112.935ms | 112.935ms | 112.935ms | 112.935ms | 260.000ms | 179.54 MiB | 512.00 KiB | 8.85M/s |
| q20 | point lookup | 8.000ms | 56.486ms | 56.153ms | 0.0% | 56.153ms | 56.153ms | 56.153ms | 56.153ms | 60.000ms | 49.26 MiB | none | 17.81M/s |
| q21 | substring scan | 30.000ms | 87.696ms | 76.604ms | 0.0% | 76.604ms | 76.604ms | 76.604ms | 76.604ms | 180.000ms | 102.26 MiB | 5.75 MiB | 13.05M/s |
| q22 | substring scan and group by | 41.000ms | 102.976ms | 79.393ms | 0.0% | 79.393ms | 79.393ms | 79.393ms | 79.393ms | 190.000ms | 121.78 MiB | none | 12.60M/s |
| q23 | two substring scans and group by | 73.000ms | 127.185ms | 125.901ms | 0.0% | 125.901ms | 125.901ms | 125.901ms | 125.901ms | 250.000ms | 150.29 MiB | 3.25 MiB | 7.94M/s |
| q24 | select star and top k | 105.000ms | 150.037ms | 158.306ms | 0.0% | 158.306ms | 158.306ms | 158.306ms | 158.306ms | 350.000ms | 219.90 MiB | 4.25 MiB | 6.32M/s |
| q25 | top k by a date | 14.000ms | 59.852ms | 56.651ms | 0.0% | 56.651ms | 56.651ms | 56.651ms | 56.651ms | 60.000ms | 62.51 MiB | none | 17.65M/s |
| q26 | top k by a string | 9.000ms | 60.402ms | 61.054ms | 0.0% | 61.054ms | 61.054ms | 61.054ms | 61.054ms | 70.000ms | 57.76 MiB | none | 16.38M/s |
| q27 | top k by two columns | 14.000ms | 53.512ms | 58.187ms | 0.0% | 58.187ms | 58.187ms | 58.187ms | 58.187ms | 70.000ms | 63.77 MiB | none | 17.19M/s |
| q28 | group by with a string length | 57.000ms | 106.520ms | 101.682ms | 0.0% | 101.682ms | 101.682ms | 101.682ms | 101.682ms | 210.000ms | 116.51 MiB | none | 9.83M/s |
| q29 | group by a regular expression | 223.000ms | 306.322ms | 270.789ms | 0.0% | 270.789ms | 270.789ms | 270.789ms | 270.789ms | 980.000ms | 181.24 MiB | 3.00 MiB | 3.69M/s |
| q30 | ninety sums over one column | 36.000ms | 74.089ms | 82.049ms | 0.0% | 82.049ms | 82.049ms | 82.049ms | 82.049ms | 80.000ms | 56.77 MiB | none | 12.19M/s |
| q31 | group by two and several aggregates | 44.000ms | 80.540ms | 91.955ms | 0.0% | 91.955ms | 91.955ms | 91.955ms | 91.955ms | 120.000ms | 96.42 MiB | none | 10.87M/s |
| q32 | group by a high card pair | 62.000ms | 102.674ms | 106.819ms | 0.0% | 106.819ms | 106.819ms | 106.819ms | 106.819ms | 150.000ms | 98.53 MiB | 256.00 KiB | 9.36M/s |
| q33 | group by a high card pair, unfiltered | 108.000ms | 157.519ms | 158.194ms | 0.0% | 158.194ms | 158.194ms | 158.194ms | 158.194ms | 300.000ms | 174.90 MiB | none | 6.32M/s |
| q34 | group by a long string | 121.000ms | 170.367ms | 166.460ms | 0.0% | 166.460ms | 166.460ms | 166.460ms | 166.460ms | 350.000ms | 246.53 MiB | none | 6.01M/s |
| q35 | group by a constant and a long string | 168.000ms | 168.497ms | 218.434ms | 0.0% | 218.434ms | 218.434ms | 218.434ms | 218.434ms | 470.000ms | 246.16 MiB | none | 4.58M/s |
| q36 | group by four expressions | 33.000ms | 71.882ms | 79.607ms | 0.0% | 79.607ms | 79.607ms | 79.607ms | 79.607ms | 160.000ms | 98.82 MiB | none | 12.56M/s |
| q37 | date range and group by a URL | 10.000ms | 60.840ms | 49.307ms | 0.0% | 49.307ms | 49.307ms | 49.307ms | 49.307ms | 60.000ms | 51.28 MiB | none | 20.28M/s |
| q38 | date range and group by a title | 9.000ms | 47.712ms | 48.074ms | 0.0% | 48.074ms | 48.074ms | 48.074ms | 48.074ms | 50.000ms | 49.42 MiB | none | 20.80M/s |
| q39 | date range, group by and offset | 12.000ms | 52.234ms | 57.699ms | 0.0% | 57.699ms | 57.699ms | 57.699ms | 57.699ms | 60.000ms | 49.91 MiB | none | 17.33M/s |
| q40 | date range, a case and a wide group by | 12.000ms | 62.024ms | 48.915ms | 0.0% | 48.915ms | 48.915ms | 48.915ms | 48.915ms | 60.000ms | 57.53 MiB | none | 20.44M/s |
| q41 | date range with an IN and a hash | 7.000ms | 53.378ms | 49.473ms | 0.0% | 49.473ms | 49.473ms | 49.473ms | 49.473ms | 50.000ms | 51.07 MiB | 1.48 MiB | 20.21M/s |
| q42 | date range and a deep offset | 22.000ms | 57.171ms | 60.248ms | 0.0% | 60.248ms | 60.248ms | 60.248ms | 60.248ms | 70.000ms | 50.97 MiB | none | 16.60M/s |
| q43 | minute buckets over a date range | 9.000ms | 60.464ms | 56.355ms | 0.0% | 56.355ms | 56.355ms | 56.355ms | 56.355ms | 60.000ms | 48.03 MiB | none | 17.74M/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 1.831s by its own clock and 3.752s by ours, 3.762s cold, 6.810s of CPU, peak 246.53 MiB, 23.48M/s and 3.18 GiB/s.

Running it cost 105% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 6.29x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 5.000ms | 109.218ms | 104.711ms | 0.0% | 104.711ms | 104.711ms | 104.711ms | 104.711ms | 120.000ms | 202.12 MiB | none | 9.55M/s |
| q2 | filtered count | 5.000ms | 90.681ms | 99.926ms | 0.0% | 99.926ms | 99.926ms | 99.926ms | 99.926ms | 110.000ms | 202.87 MiB | none | 10.01M/s |
| q3 | three aggregates | 118.000ms | 185.217ms | 227.560ms | 0.0% | 227.560ms | 227.560ms | 227.560ms | 227.560ms | 160.000ms | 214.36 MiB | 4.00 KiB | 4.39M/s |
| q4 | average | 46.000ms | 164.674ms | 146.606ms | 0.0% | 146.606ms | 146.606ms | 146.606ms | 146.606ms | 140.000ms | 217.18 MiB | none | 6.82M/s |
| q5 | count distinct, high card | 141.000ms | 319.621ms | 252.530ms | 0.0% | 252.530ms | 252.530ms | 252.530ms | 252.530ms | 290.000ms | 315.57 MiB | none | 3.96M/s |
| q6 | count distinct, strings | 172.000ms | 262.511ms | 285.425ms | 0.0% | 285.425ms | 285.425ms | 285.425ms | 285.425ms | 210.000ms | 251.12 MiB | none | 3.50M/s |
| q7 | min and max of a date | 14.000ms | 108.534ms | 107.700ms | 0.0% | 107.700ms | 107.700ms | 107.700ms | 107.700ms | 120.000ms | 204.93 MiB | none | 9.28M/s |
| q8 | group by, low card | 102.000ms | 197.278ms | 211.174ms | 0.0% | 211.174ms | 211.174ms | 211.174ms | 211.174ms | 170.000ms | 215.05 MiB | 536.00 KiB | 4.74M/s |
| q9 | group by and count distinct | 102.000ms | 257.547ms | 219.456ms | 0.0% | 219.456ms | 219.456ms | 219.456ms | 219.456ms | 260.000ms | 284.04 MiB | none | 4.56M/s |
| q10 | group by, several aggregates | 164.000ms | 280.596ms | 287.843ms | 0.0% | 287.843ms | 287.843ms | 287.843ms | 287.843ms | 300.000ms | 278.92 MiB | none | 3.47M/s |
| q11 | group by a string and count distinct | 150.000ms | 192.327ms | 266.441ms | 0.0% | 266.441ms | 266.441ms | 266.441ms | 266.441ms | 180.000ms | 234.54 MiB | none | 3.75M/s |
| q12 | group by two strings and count distinct | 122.000ms | 258.345ms | 225.092ms | 0.0% | 225.092ms | 225.092ms | 225.092ms | 225.092ms | 190.000ms | 240.02 MiB | none | 4.44M/s |
| q13 | group by a string and top k | 111.000ms | 184.990ms | 229.888ms | 0.0% | 229.888ms | 229.888ms | 229.888ms | 229.888ms | 250.000ms | 265.90 MiB | none | 4.35M/s |
| q14 | group by a string and count distinct | 124.000ms | 256.102ms | 241.411ms | 0.0% | 241.411ms | 241.411ms | 241.411ms | 241.411ms | 270.000ms | 293.95 MiB | none | 4.14M/s |
| q15 | group by two columns and top k | 117.000ms | 261.743ms | 227.205ms | 0.0% | 227.205ms | 227.205ms | 227.205ms | 227.205ms | 270.000ms | 277.09 MiB | 4.00 KiB | 4.40M/s |
| q16 | group by, very high card | 75.000ms | 225.255ms | 187.773ms | 0.0% | 187.773ms | 187.773ms | 187.773ms | 187.773ms | 270.000ms | 275.29 MiB | none | 5.33M/s |
| q17 | group by two, very high card | 202.000ms | 243.137ms | 322.175ms | 0.0% | 322.175ms | 322.175ms | 322.175ms | 322.175ms | 450.000ms | 363.62 MiB | none | 3.10M/s |
| q18 | group by two, no ordering | 122.000ms | 282.917ms | 230.445ms | 0.0% | 230.445ms | 230.445ms | 230.445ms | 230.445ms | 230.000ms | 256.11 MiB | none | 4.34M/s |
| q19 | group by with an extract | 181.000ms | 307.917ms | 301.532ms | 0.0% | 301.532ms | 301.532ms | 301.532ms | 301.532ms | 460.000ms | 371.01 MiB | none | 3.32M/s |
| q20 | point lookup | 82.000ms | 188.876ms | 197.567ms | 0.0% | 197.567ms | 197.567ms | 197.567ms | 197.567ms | 170.000ms | 217.79 MiB | none | 5.06M/s |
| q21 | substring scan | 149.000ms | 267.132ms | 262.146ms | 0.0% | 262.146ms | 262.146ms | 262.146ms | 262.146ms | 270.000ms | 253.38 MiB | none | 3.81M/s |
| q22 | substring scan and group by | 177.000ms | 250.982ms | 296.663ms | 0.0% | 296.663ms | 296.663ms | 296.663ms | 296.663ms | 310.000ms | 285.79 MiB | none | 3.37M/s |
| q23 | two substring scans and group by | 230.000ms | 360.987ms | 354.121ms | 0.0% | 354.121ms | 354.121ms | 354.121ms | 354.121ms | 380.000ms | 304.47 MiB | none | 2.82M/s |
| q24 | select star and top k | 339.000ms | 462.935ms | 462.253ms | 0.0% | 462.253ms | 462.253ms | 462.253ms | 462.253ms | 460.000ms | 331.37 MiB | 4.00 KiB | 2.16M/s |
| q25 | top k by a date | 129.000ms | 190.686ms | 226.314ms | 0.0% | 226.314ms | 226.314ms | 226.314ms | 226.314ms | 190.000ms | 239.56 MiB | none | 4.42M/s |
| q26 | top k by a string | 140.000ms | 179.976ms | 243.085ms | 0.0% | 243.085ms | 243.085ms | 243.085ms | 243.085ms | 180.000ms | 237.77 MiB | none | 4.11M/s |
| q27 | top k by two columns | 84.000ms | 297.010ms | 224.359ms | 0.0% | 224.359ms | 224.359ms | 224.359ms | 224.359ms | 190.000ms | 237.66 MiB | none | 4.46M/s |
| q28 | group by with a string length | 62.000ms | 207.443ms | 170.442ms | 0.0% | 170.442ms | 170.442ms | 170.442ms | 170.442ms | 160.000ms | 230.41 MiB | none | 5.87M/s |
| q29 | group by a regular expression | 173.000ms | 257.643ms | 301.489ms | 0.0% | 301.489ms | 301.489ms | 301.489ms | 301.489ms | 470.000ms | 356.29 MiB | 96.00 KiB | 3.32M/s |
| q30 | ninety sums over one column | 70.000ms | 214.411ms | 179.466ms | 0.0% | 179.466ms | 179.466ms | 179.466ms | 179.466ms | 160.000ms | 215.07 MiB | none | 5.57M/s |
| q31 | group by two and several aggregates | 129.000ms | 252.249ms | 238.536ms | 0.0% | 238.536ms | 238.536ms | 238.536ms | 238.536ms | 230.000ms | 252.34 MiB | none | 4.19M/s |
| q32 | group by a high card pair | 134.000ms | 280.579ms | 240.216ms | 0.0% | 240.216ms | 240.216ms | 240.216ms | 240.216ms | 270.000ms | 269.63 MiB | none | 4.16M/s |
| q33 | group by a high card pair, unfiltered | 191.000ms | 265.642ms | 313.563ms | 0.0% | 313.563ms | 313.563ms | 313.563ms | 313.563ms | 390.000ms | 329.39 MiB | none | 3.19M/s |
| q34 | group by a long string | 201.000ms | 306.237ms | 323.855ms | 0.0% | 323.855ms | 323.855ms | 323.855ms | 323.855ms | 500.000ms | 391.90 MiB | none | 3.09M/s |
| q35 | group by a constant and a long string | 163.000ms | 251.700ms | 284.359ms | 0.0% | 284.359ms | 284.359ms | 284.359ms | 284.359ms | 510.000ms | 398.46 MiB | none | 3.52M/s |
| q36 | group by four expressions | 146.000ms | 219.356ms | 275.541ms | 0.0% | 275.541ms | 275.541ms | 275.541ms | 275.541ms | 280.000ms | 280.61 MiB | none | 3.63M/s |
| q37 | date range and group by a URL | 143.000ms | 242.506ms | 257.366ms | 0.0% | 257.366ms | 257.366ms | 257.366ms | 257.366ms | 190.000ms | 225.36 MiB | none | 3.89M/s |
| q38 | date range and group by a title | 123.000ms | 250.714ms | 234.972ms | 0.0% | 234.972ms | 234.972ms | 234.972ms | 234.972ms | 190.000ms | 223.08 MiB | none | 4.26M/s |
| q39 | date range, group by and offset | 140.000ms | 287.243ms | 255.428ms | 0.0% | 255.428ms | 255.428ms | 255.428ms | 255.428ms | 190.000ms | 224.59 MiB | none | 3.91M/s |
| q40 | date range, a case and a wide group by | 105.000ms | 271.035ms | 223.665ms | 0.0% | 223.665ms | 223.665ms | 223.665ms | 223.665ms | 200.000ms | 234.58 MiB | none | 4.47M/s |
| q41 | date range with an IN and a hash | 137.000ms | 222.304ms | 261.108ms | 0.0% | 261.108ms | 261.108ms | 261.108ms | 261.108ms | 170.000ms | 221.91 MiB | 100.00 KiB | 3.83M/s |
| q42 | date range and a deep offset | 129.000ms | 247.609ms | 242.512ms | 0.0% | 242.512ms | 242.512ms | 242.512ms | 242.512ms | 160.000ms | 220.00 MiB | none | 4.12M/s |
| q43 | minute buckets over a date range | 133.000ms | 221.576ms | 243.882ms | 0.0% | 243.882ms | 243.882ms | 243.882ms | 243.882ms | 180.000ms | 219.28 MiB | none | 4.10M/s |

clickhouse-local 26.9.1.1162 over 43 of 43 queries. Total 5.582s by its own clock and 10.488s by ours, 10.385s cold, 10.850s of CPU, peak 398.46 MiB, 7.70M/s and 1.04 GiB/s.

Running it cost 88% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 4.63x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 2.000ms | 68.417ms | 26.192ms | 0.0% | 26.192ms | 26.192ms | 26.192ms | 26.192ms | 20.000ms | 81.01 MiB | 71.03 MiB | 38.18M/s |
| q2 | filtered count | 10.000ms | 40.491ms | 33.108ms | 0.0% | 33.108ms | 33.108ms | 33.108ms | 33.108ms | 50.000ms | 123.72 MiB | 52.00 KiB | 30.20M/s |
| q3 | three aggregates | 13.000ms | 41.065ms | 39.731ms | 0.0% | 39.731ms | 39.731ms | 39.731ms | 39.731ms | 70.000ms | 149.85 MiB | 12.00 KiB | 25.17M/s |
| q4 | average | 9.000ms | 30.156ms | 32.862ms | 0.0% | 32.862ms | 32.862ms | 32.862ms | 32.862ms | 40.000ms | 121.80 MiB | none | 30.43M/s |
| q5 | count distinct, high card | 25.000ms | 64.249ms | 51.286ms | 0.0% | 51.286ms | 51.286ms | 51.286ms | 51.286ms | 110.000ms | 274.42 MiB | none | 19.50M/s |
| q6 | count distinct, strings | 43.000ms | 53.788ms | 68.203ms | 0.0% | 68.203ms | 68.203ms | 68.203ms | 68.203ms | 150.000ms | 299.88 MiB | none | 14.66M/s |
| q7 | min and max of a date | 3.000ms | 29.764ms | 27.229ms | 0.0% | 27.229ms | 27.229ms | 27.229ms | 27.229ms | 20.000ms | 80.30 MiB | 4.00 KiB | 36.72M/s |
| q8 | group by, low card | 13.000ms | 38.823ms | 38.253ms | 0.0% | 38.253ms | 38.253ms | 38.253ms | 38.253ms | 50.000ms | 125.14 MiB | none | 26.14M/s |
| q9 | group by and count distinct | 43.000ms | 59.105ms | 68.189ms | 0.0% | 68.189ms | 68.189ms | 68.189ms | 68.189ms | 170.000ms | 302.84 MiB | none | 14.66M/s |
| q10 | group by, several aggregates | 43.000ms | 65.846ms | 68.419ms | 0.0% | 68.419ms | 68.419ms | 68.419ms | 68.419ms | 170.000ms | 288.11 MiB | none | 14.62M/s |
| q11 | group by a string and count distinct | 24.000ms | 49.284ms | 46.964ms | 0.0% | 46.964ms | 46.964ms | 46.964ms | 46.964ms | 80.000ms | 202.00 MiB | 476.00 KiB | 21.29M/s |
| q12 | group by two strings and count distinct | 25.000ms | 52.229ms | 50.127ms | 0.0% | 50.127ms | 50.127ms | 50.127ms | 50.127ms | 90.000ms | 207.70 MiB | none | 19.95M/s |
| q13 | group by a string and top k | 35.000ms | 60.021ms | 58.838ms | 0.0% | 58.838ms | 58.838ms | 58.838ms | 58.838ms | 120.000ms | 305.29 MiB | none | 17.00M/s |
| q14 | group by a string and count distinct | 47.000ms | 85.141ms | 72.885ms | 0.0% | 72.885ms | 72.885ms | 72.885ms | 72.885ms | 180.000ms | 384.26 MiB | none | 13.72M/s |
| q15 | group by two columns and top k | 38.000ms | 61.840ms | 62.773ms | 0.0% | 62.773ms | 62.773ms | 62.773ms | 62.773ms | 120.000ms | 301.15 MiB | none | 15.93M/s |
| q16 | group by, very high card | 31.000ms | 52.443ms | 53.963ms | 0.0% | 53.963ms | 53.963ms | 53.963ms | 53.963ms | 130.000ms | 301.26 MiB | none | 18.53M/s |
| q17 | group by two, very high card | 86.000ms | 103.049ms | 138.597ms | 0.0% | 138.597ms | 138.597ms | 138.597ms | 138.597ms | 350.000ms | 426.07 MiB | none | 7.21M/s |
| q18 | group by two, no ordering | 75.000ms | 86.401ms | 100.140ms | 0.0% | 100.140ms | 100.140ms | 100.140ms | 100.140ms | 320.000ms | 428.42 MiB | none | 9.99M/s |
| q19 | group by with an extract | 72.000ms | 104.798ms | 97.523ms | 0.0% | 97.523ms | 97.523ms | 97.523ms | 97.523ms | 300.000ms | 481.40 MiB | 32.00 KiB | 10.25M/s |
| q20 | point lookup | 8.000ms | 34.280ms | 34.436ms | 0.0% | 34.436ms | 34.436ms | 34.436ms | 34.436ms | 30.000ms | 124.81 MiB | none | 29.04M/s |
| q21 | substring scan | 44.000ms | 70.053ms | 67.811ms | 0.0% | 67.811ms | 67.811ms | 67.811ms | 67.811ms | 170.000ms | 242.53 MiB | none | 14.75M/s |
| q22 | substring scan and group by | 64.000ms | 76.163ms | 87.946ms | 0.0% | 87.946ms | 87.946ms | 87.946ms | 87.946ms | 210.000ms | 305.48 MiB | none | 11.37M/s |
| q23 | two substring scans and group by | 113.000ms | 160.170ms | 139.992ms | 0.0% | 139.992ms | 139.992ms | 139.992ms | 139.992ms | 520.000ms | 463.21 MiB | none | 7.14M/s |
| q24 | select star and top k | 231.000ms | 304.418ms | 263.280ms | 0.0% | 263.280ms | 263.280ms | 263.280ms | 263.280ms | 1.110s | 982.70 MiB | none | 3.80M/s |
| q25 | top k by a date | 27.000ms | 55.758ms | 56.691ms | 0.0% | 56.691ms | 56.691ms | 56.691ms | 56.691ms | 140.000ms | 256.86 MiB | none | 17.64M/s |
| q26 | top k by a string | 26.000ms | 47.968ms | 48.785ms | 0.0% | 48.785ms | 48.785ms | 48.785ms | 48.785ms | 90.000ms | 218.52 MiB | none | 20.50M/s |
| q27 | top k by two columns | 27.000ms | 46.397ms | 56.757ms | 0.0% | 56.757ms | 56.757ms | 56.757ms | 56.757ms | 110.000ms | 253.61 MiB | none | 17.62M/s |
| q28 | group by with a string length | 59.000ms | 74.580ms | 94.872ms | 0.0% | 94.872ms | 94.872ms | 94.872ms | 94.872ms | 210.000ms | 288.96 MiB | none | 10.54M/s |
| q29 | group by a regular expression | 109.000ms | 160.569ms | 142.693ms | 0.0% | 142.693ms | 142.693ms | 142.693ms | 142.693ms | 500.000ms | 445.94 MiB | none | 7.01M/s |
| q30 | ninety sums over one column | 29.000ms | 52.482ms | 53.259ms | 0.0% | 53.259ms | 53.259ms | 53.259ms | 53.259ms | 50.000ms | 130.21 MiB | 52.00 KiB | 18.78M/s |
| q31 | group by two and several aggregates | 36.000ms | 62.539ms | 61.504ms | 0.0% | 61.504ms | 61.504ms | 61.504ms | 61.504ms | 140.000ms | 321.44 MiB | none | 16.26M/s |
| q32 | group by a high card pair | 36.000ms | 69.087ms | 65.458ms | 0.0% | 65.458ms | 65.458ms | 65.458ms | 65.458ms | 120.000ms | 291.00 MiB | none | 15.28M/s |
| q33 | group by a high card pair, unfiltered | 60.000ms | 85.769ms | 91.670ms | 0.0% | 91.670ms | 91.670ms | 91.670ms | 91.670ms | 290.000ms | 434.27 MiB | none | 10.91M/s |
| q34 | group by a long string | 90.000ms | 144.975ms | 119.958ms | 0.0% | 119.958ms | 119.958ms | 119.958ms | 119.958ms | 460.000ms | 551.49 MiB | none | 8.34M/s |
| q35 | group by a constant and a long string | 99.000ms | 126.387ms | 128.378ms | 0.0% | 128.378ms | 128.378ms | 128.378ms | 128.378ms | 460.000ms | 561.82 MiB | none | 7.79M/s |
| q36 | group by four expressions | 35.000ms | 53.647ms | 63.087ms | 0.0% | 63.087ms | 63.087ms | 63.087ms | 63.087ms | 140.000ms | 272.65 MiB | none | 15.85M/s |
| q37 | date range and group by a URL | 38.000ms | 71.561ms | 62.757ms | 0.0% | 62.757ms | 62.757ms | 62.757ms | 62.757ms | 100.000ms | 176.09 MiB | none | 15.93M/s |
| q38 | date range and group by a title | 48.000ms | 77.922ms | 78.912ms | 0.0% | 78.912ms | 78.912ms | 78.912ms | 78.912ms | 110.000ms | 177.07 MiB | none | 12.67M/s |
| q39 | date range, group by and offset | 33.000ms | 68.242ms | 58.926ms | 0.0% | 58.926ms | 58.926ms | 58.926ms | 58.926ms | 90.000ms | 168.34 MiB | none | 16.97M/s |
| q40 | date range, a case and a wide group by | 51.000ms | 80.674ms | 76.908ms | 0.0% | 76.908ms | 76.908ms | 76.908ms | 76.908ms | 130.000ms | 202.70 MiB | none | 13.00M/s |
| q41 | date range with an IN and a hash | 17.000ms | 46.654ms | 44.279ms | 0.0% | 44.279ms | 44.279ms | 44.279ms | 44.279ms | 60.000ms | 132.87 MiB | none | 22.58M/s |
| q42 | date range and a deep offset | 19.000ms | 48.104ms | 45.322ms | 0.0% | 45.322ms | 45.322ms | 45.322ms | 45.322ms | 60.000ms | 134.10 MiB | none | 22.06M/s |
| q43 | minute buckets over a date range | 16.000ms | 45.356ms | 40.177ms | 0.0% | 40.177ms | 40.177ms | 40.177ms | 40.177ms | 50.000ms | 133.36 MiB | none | 24.89M/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 1.952s by its own clock and 3.119s by ours, 3.211s cold, 7.890s of CPU, peak 982.70 MiB, 22.03M/s and 2.99 GiB/s.

Running it cost 60% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 10.05x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 12.679ms | 228.682ms | 229.334ms | 0.0% | 229.334ms | 229.334ms | 229.334ms | 229.334ms | 230.000ms | 64.01 MiB | 34.11 MiB | 4.36M/s |
| q2 | filtered count | 12.862ms | 201.025ms | 209.672ms | 0.0% | 209.672ms | 209.672ms | 209.672ms | 209.672ms | 210.000ms | 70.16 MiB | 304.00 KiB | 4.77M/s |
| q3 | three aggregates | 14.015ms | 222.247ms | 187.448ms | 0.0% | 187.448ms | 187.448ms | 187.448ms | 187.448ms | 200.000ms | 73.91 MiB | 744.00 KiB | 5.33M/s |
| q4 | average | 18.741ms | 208.444ms | 215.245ms | 0.0% | 215.245ms | 215.245ms | 215.245ms | 215.245ms | 210.000ms | 87.52 MiB | none | 4.65M/s |
| q5 | count distinct, high card | 50.295ms | 257.419ms | 261.429ms | 0.0% | 261.429ms | 261.429ms | 261.429ms | 261.429ms | 350.000ms | 138.93 MiB | none | 3.83M/s |
| q6 | count distinct, strings | 44.514ms | 243.077ms | 247.451ms | 0.0% | 247.451ms | 247.451ms | 247.451ms | 247.451ms | 310.000ms | 141.36 MiB | none | 4.04M/s |
| q7 | min and max of a date | 11.487ms | 191.933ms | 216.687ms | 0.0% | 216.687ms | 216.687ms | 216.687ms | 216.687ms | 220.000ms | 72.77 MiB | none | 4.61M/s |
| q8 | group by, low card | 20.145ms | 195.245ms | 217.217ms | 0.0% | 217.217ms | 217.217ms | 217.217ms | 217.217ms | 240.000ms | 79.74 MiB | none | 4.60M/s |
| q9 | group by and count distinct | 94.805ms | 311.584ms | 291.564ms | 0.0% | 291.564ms | 291.564ms | 291.564ms | 291.564ms | 480.000ms | 232.58 MiB | none | 3.43M/s |
| q10 | group by, several aggregates | 104.026ms | 307.280ms | 316.396ms | 0.0% | 316.396ms | 316.396ms | 316.396ms | 316.396ms | 520.000ms | 232.25 MiB | none | 3.16M/s |
| q11 | group by a string and count distinct | 37.396ms | 224.615ms | 241.820ms | 0.0% | 241.820ms | 241.820ms | 241.820ms | 241.820ms | 280.000ms | 104.07 MiB | none | 4.14M/s |
| q12 | group by two strings and count distinct | 37.652ms | 244.040ms | 240.027ms | 0.0% | 240.027ms | 240.027ms | 240.027ms | 240.027ms | 290.000ms | 106.14 MiB | none | 4.17M/s |
| q13 | group by a string and top k | 46.616ms | 235.568ms | 256.542ms | 0.0% | 256.542ms | 256.542ms | 256.542ms | 256.542ms | 310.000ms | 124.30 MiB | none | 3.90M/s |
| q14 | group by a string and count distinct | 92.880ms | 336.032ms | 276.290ms | 0.0% | 276.290ms | 276.290ms | 276.290ms | 276.290ms | 410.000ms | 180.79 MiB | none | 3.62M/s |
| q15 | group by two columns and top k | 47.703ms | 259.525ms | 260.535ms | 0.0% | 260.535ms | 260.535ms | 260.535ms | 260.535ms | 340.000ms | 136.81 MiB | none | 3.84M/s |
| q16 | group by, very high card | 69.711ms | 282.959ms | 272.349ms | 0.0% | 272.349ms | 272.349ms | 272.349ms | 272.349ms | 370.000ms | 158.96 MiB | none | 3.67M/s |
| q17 | group by two, very high card | 108.841ms | 333.946ms | 327.751ms | 0.0% | 327.751ms | 327.751ms | 327.751ms | 327.751ms | 580.000ms | 289.36 MiB | none | 3.05M/s |
| q18 | group by two, no ordering | 118.132ms | 334.657ms | 321.755ms | 0.0% | 321.755ms | 321.755ms | 321.755ms | 321.755ms | 550.000ms | 278.52 MiB | none | 3.11M/s |
| q19 | group by with an extract | 119.232ms | 355.640ms | 332.744ms | 0.0% | 332.744ms | 332.744ms | 332.744ms | 332.744ms | 610.000ms | 306.88 MiB | none | 3.01M/s |
| q20 | point lookup | 14.152ms | 196.796ms | 199.318ms | 0.0% | 199.318ms | 199.318ms | 199.318ms | 199.318ms | 200.000ms | 81.81 MiB | 16.00 KiB | 5.02M/s |
| q21 | substring scan | 95.956ms | 330.065ms | 301.837ms | 0.0% | 301.837ms | 301.837ms | 301.837ms | 301.837ms | 480.000ms | 185.08 MiB | 8.80 MiB | 3.31M/s |
| q22 | substring scan and group by | 119.105ms | 259.799ms | 316.277ms | 0.0% | 316.277ms | 316.277ms | 316.277ms | 316.277ms | 510.000ms | 212.39 MiB | none | 3.16M/s |
| q23 | two substring scans and group by | 176.025ms | 360.968ms | 387.414ms | 0.0% | 387.414ms | 387.414ms | 387.414ms | 387.414ms | 900.000ms | 455.10 MiB | none | 2.58M/s |
| q24 | select star and top k | 155.880ms | 361.240ms | 374.111ms | 0.0% | 374.111ms | 374.111ms | 374.111ms | 374.111ms | 1.030s | 518.94 MiB | none | 2.67M/s |
| q25 | top k by a date | 31.468ms | 239.971ms | 243.915ms | 0.0% | 243.915ms | 243.915ms | 243.915ms | 243.915ms | 290.000ms | 103.05 MiB | none | 4.10M/s |
| q26 | top k by a string | 36.886ms | 231.845ms | 233.741ms | 0.0% | 233.741ms | 233.741ms | 233.741ms | 233.741ms | 260.000ms | 93.16 MiB | none | 4.28M/s |
| q27 | top k by two columns | 35.207ms | 205.481ms | 254.924ms | 0.0% | 254.924ms | 254.924ms | 254.924ms | 254.924ms | 290.000ms | 103.55 MiB | none | 3.92M/s |
| q30 | ninety sums over one column | 38.212ms | 237.116ms | 231.172ms | 0.0% | 231.172ms | 231.172ms | 231.172ms | 231.172ms | 250.000ms | 75.49 MiB | none | 4.33M/s |
| q31 | group by two and several aggregates | 59.919ms | 257.743ms | 277.212ms | 0.0% | 277.212ms | 277.212ms | 277.212ms | 277.212ms | 350.000ms | 123.13 MiB | none | 3.61M/s |
| q32 | group by a high card pair | 46.969ms | 265.651ms | 254.016ms | 0.0% | 254.016ms | 254.016ms | 254.016ms | 254.016ms | 310.000ms | 132.38 MiB | none | 3.94M/s |
| q33 | group by a high card pair, unfiltered | 116.992ms | 291.386ms | 326.700ms | 0.0% | 326.700ms | 326.700ms | 326.700ms | 326.700ms | 550.000ms | 293.29 MiB | none | 3.06M/s |
| q34 | group by a long string | 171.550ms | 410.153ms | 408.296ms | 0.0% | 408.296ms | 408.296ms | 408.296ms | 408.296ms | 760.000ms | 441.46 MiB | none | 2.45M/s |
| q35 | group by a constant and a long string | 198.744ms | 423.334ms | 430.677ms | 0.0% | 430.677ms | 430.677ms | 430.677ms | 430.677ms | 900.000ms | 535.10 MiB | none | 2.32M/s |
| q37 | date range and group by a URL | 61.123ms | 270.656ms | 256.884ms | 0.0% | 256.884ms | 256.884ms | 256.884ms | 256.884ms | 310.000ms | 149.41 MiB | 16.00 KiB | 3.89M/s |
| q38 | date range and group by a title | 53.693ms | 253.579ms | 237.215ms | 0.0% | 237.215ms | 237.215ms | 237.215ms | 237.215ms | 280.000ms | 142.45 MiB | none | 4.22M/s |
| q39 | date range, group by and offset | 46.557ms | 259.919ms | 236.842ms | 0.0% | 236.842ms | 236.842ms | 236.842ms | 236.842ms | 260.000ms | 110.23 MiB | none | 4.22M/s |
| q40 | date range, a case and a wide group by | 55.310ms | 259.381ms | 262.590ms | 0.0% | 262.590ms | 262.590ms | 262.590ms | 262.590ms | 320.000ms | 138.13 MiB | none | 3.81M/s |
| q41 | date range with an IN and a hash | 25.868ms | 228.679ms | 214.708ms | 0.0% | 214.708ms | 214.708ms | 214.708ms | 214.708ms | 240.000ms | 89.04 MiB | 116.00 KiB | 4.66M/s |
| q42 | date range and a deep offset | 21.766ms | 200.759ms | 221.178ms | 0.0% | 221.178ms | 221.178ms | 221.178ms | 221.178ms | 230.000ms | 85.65 MiB | none | 4.52M/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 2.623s by its own clock and 10.591s by ours, 10.518s cold, 15.430s of CPU, peak 535.10 MiB, 14.87M/s and 2.02 GiB/s.

Running it cost 304% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.30x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-server in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 42.995ms | 39.668ms | 0.0% | 39.668ms | 39.668ms | 39.668ms | 39.668ms | not read | not read | not read | 25.21M/s |
| q2 | filtered count | 1.000ms | 39.806ms | 36.708ms | 0.0% | 36.708ms | 36.708ms | 36.708ms | 36.708ms | not read | not read | not read | 27.24M/s |
| q3 | three aggregates | 4.000ms | 39.996ms | 41.033ms | 0.0% | 41.033ms | 41.033ms | 41.033ms | 41.033ms | not read | not read | not read | 24.37M/s |
| q4 | average | 3.000ms | 47.112ms | 40.838ms | 0.0% | 40.838ms | 40.838ms | 40.838ms | 40.838ms | not read | not read | not read | 24.49M/s |
| q5 | count distinct, high card | 22.000ms | 67.918ms | 60.892ms | 0.0% | 60.892ms | 60.892ms | 60.892ms | 60.892ms | not read | not read | not read | 16.42M/s |
| q6 | count distinct, strings | 21.000ms | 54.534ms | 61.638ms | 0.0% | 61.638ms | 61.638ms | 61.638ms | 61.638ms | not read | not read | not read | 16.22M/s |
| q7 | min and max of a date | 2.000ms | 43.907ms | 37.439ms | 0.0% | 37.439ms | 37.439ms | 37.439ms | 37.439ms | not read | not read | not read | 26.71M/s |
| q8 | group by, low card | 4.000ms | 43.129ms | 39.049ms | 0.0% | 39.049ms | 39.049ms | 39.049ms | 39.049ms | not read | not read | not read | 25.61M/s |
| q9 | group by and count distinct | 22.000ms | 60.295ms | 68.892ms | 0.0% | 68.892ms | 68.892ms | 68.892ms | 68.892ms | not read | not read | not read | 14.52M/s |
| q10 | group by, several aggregates | 35.000ms | 72.102ms | 75.121ms | 0.0% | 75.121ms | 75.121ms | 75.121ms | 75.121ms | not read | not read | not read | 13.31M/s |
| q11 | group by a string and count distinct | 6.000ms | 46.921ms | 48.135ms | 0.0% | 48.135ms | 48.135ms | 48.135ms | 48.135ms | not read | not read | not read | 20.77M/s |
| q12 | group by two strings and count distinct | 7.000ms | 46.185ms | 44.699ms | 0.0% | 44.699ms | 44.699ms | 44.699ms | 44.699ms | not read | not read | not read | 22.37M/s |
| q13 | group by a string and top k | 21.000ms | 53.860ms | 61.724ms | 0.0% | 61.724ms | 61.724ms | 61.724ms | 61.724ms | not read | not read | not read | 16.20M/s |
| q14 | group by a string and count distinct | 16.000ms | 51.380ms | 54.904ms | 0.0% | 54.904ms | 54.904ms | 54.904ms | 54.904ms | not read | not read | not read | 18.21M/s |
| q15 | group by two columns and top k | 19.000ms | 61.222ms | 55.521ms | 0.0% | 55.521ms | 55.521ms | 55.521ms | 55.521ms | not read | not read | not read | 18.01M/s |
| q16 | group by, very high card | 18.000ms | 62.443ms | 58.243ms | 0.0% | 58.243ms | 58.243ms | 58.243ms | 58.243ms | not read | not read | not read | 17.17M/s |
| q17 | group by two, very high card | 42.000ms | 70.933ms | 89.072ms | 0.0% | 89.072ms | 89.072ms | 89.072ms | 89.072ms | not read | not read | not read | 11.23M/s |
| q18 | group by two, no ordering | 11.000ms | 53.969ms | 51.323ms | 0.0% | 51.323ms | 51.323ms | 51.323ms | 51.323ms | not read | not read | not read | 19.48M/s |
| q19 | group by with an extract | 38.000ms | 78.793ms | 77.178ms | 0.0% | 77.178ms | 77.178ms | 77.178ms | 77.178ms | not read | not read | not read | 12.96M/s |
| q20 | point lookup | 1.000ms | 42.278ms | 42.026ms | 0.0% | 42.026ms | 42.026ms | 42.026ms | 42.026ms | not read | not read | not read | 23.79M/s |
| q21 | substring scan | 17.000ms | 70.755ms | 65.129ms | 0.0% | 65.129ms | 65.129ms | 65.129ms | 65.129ms | not read | not read | not read | 15.35M/s |
| q22 | substring scan and group by | 8.000ms | 63.926ms | 45.952ms | 0.0% | 45.952ms | 45.952ms | 45.952ms | 45.952ms | not read | not read | not read | 21.76M/s |
| q23 | two substring scans and group by | 17.000ms | 71.334ms | 56.178ms | 0.0% | 56.178ms | 56.178ms | 56.178ms | 56.178ms | not read | not read | not read | 17.80M/s |
| q24 | select star and top k | 39.000ms | 71.502ms | 69.244ms | 0.0% | 69.244ms | 69.244ms | 69.244ms | 69.244ms | not read | not read | not read | 14.44M/s |
| q25 | top k by a date | 6.000ms | 44.274ms | 49.040ms | 0.0% | 49.040ms | 49.040ms | 49.040ms | 49.040ms | not read | not read | not read | 20.39M/s |
| q26 | top k by a string | 8.000ms | 45.346ms | 42.594ms | 0.0% | 42.594ms | 42.594ms | 42.594ms | 42.594ms | not read | not read | not read | 23.48M/s |
| q27 | top k by two columns | 9.000ms | 53.525ms | 48.272ms | 0.0% | 48.272ms | 48.272ms | 48.272ms | 48.272ms | not read | not read | not read | 20.72M/s |
| q28 | group by with a string length | 7.000ms | 45.152ms | 51.340ms | 0.0% | 51.340ms | 51.340ms | 51.340ms | 51.340ms | not read | not read | not read | 19.48M/s |
| q29 | group by a regular expression | 49.000ms | 90.376ms | 90.411ms | 0.0% | 90.411ms | 90.411ms | 90.411ms | 90.411ms | not read | not read | not read | 11.06M/s |
| q30 | ninety sums over one column | 8.000ms | 48.914ms | 47.015ms | 0.0% | 47.015ms | 47.015ms | 47.015ms | 47.015ms | not read | not read | not read | 21.27M/s |
| q31 | group by two and several aggregates | 14.000ms | 55.228ms | 59.870ms | 0.0% | 59.870ms | 59.870ms | 59.870ms | 59.870ms | not read | not read | not read | 16.70M/s |
| q32 | group by a high card pair | 24.000ms | 52.007ms | 62.522ms | 0.0% | 62.522ms | 62.522ms | 62.522ms | 62.522ms | not read | not read | not read | 15.99M/s |
| q33 | group by a high card pair, unfiltered | 20.000ms | 62.648ms | 64.373ms | 0.0% | 64.373ms | 64.373ms | 64.373ms | 64.373ms | not read | not read | not read | 15.53M/s |
| q34 | group by a long string | 47.000ms | 84.971ms | 82.308ms | 0.0% | 82.308ms | 82.308ms | 82.308ms | 82.308ms | not read | not read | not read | 12.15M/s |
| q35 | group by a constant and a long string | 53.000ms | 89.687ms | 87.897ms | 0.0% | 87.897ms | 87.897ms | 87.897ms | 87.897ms | not read | not read | not read | 11.38M/s |
| q36 | group by four expressions | 15.000ms | 47.662ms | 47.135ms | 0.0% | 47.135ms | 47.135ms | 47.135ms | 47.135ms | not read | not read | not read | 21.22M/s |
| q37 | date range and group by a URL | 5.000ms | 38.576ms | 39.427ms | 0.0% | 39.427ms | 39.427ms | 39.427ms | 39.427ms | not read | not read | not read | 25.36M/s |
| q38 | date range and group by a title | 5.000ms | 39.747ms | 38.957ms | 0.0% | 38.957ms | 38.957ms | 38.957ms | 38.957ms | not read | not read | not read | 25.67M/s |
| q39 | date range, group by and offset | 6.000ms | 40.444ms | 42.500ms | 0.0% | 42.500ms | 42.500ms | 42.500ms | 42.500ms | not read | not read | not read | 23.53M/s |
| q40 | date range, a case and a wide group by | 7.000ms | 42.619ms | 41.197ms | 0.0% | 41.197ms | 41.197ms | 41.197ms | 41.197ms | not read | not read | not read | 24.27M/s |
| q41 | date range with an IN and a hash | 4.000ms | 38.663ms | 39.530ms | 0.0% | 39.530ms | 39.530ms | 39.530ms | 39.530ms | not read | not read | not read | 25.30M/s |
| q42 | date range and a deep offset | 4.000ms | 38.210ms | 39.714ms | 0.0% | 39.714ms | 39.714ms | 39.714ms | 39.714ms | not read | not read | not read | 25.18M/s |
| q43 | minute buckets over a date range | 4.000ms | 36.653ms | 37.262ms | 0.0% | 37.262ms | 37.262ms | 37.262ms | 37.262ms | not read | not read | not read | 26.84M/s |

clickhouse-server 26.9.1.1162 over 43 of 43 queries. Total 670.000ms by its own clock and 2.332s by ours, 2.352s cold, no reading of CPU, peak not read, 64.18M/s and 8.70 GiB/s.

Running it cost 248% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.46x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 4.708ms | 3.752ms | 0.0% | 3.752ms | 3.752ms | 3.752ms | 3.752ms | 0.000us | 4.64 MiB | none | 266.52M/s |
| q2 | filtered count | 10.000ms | 14.055ms | 14.130ms | 0.0% | 14.130ms | 14.130ms | 14.130ms | 14.130ms | 0.000us | 6.11 MiB | none | 70.77M/s |
| q3 | three aggregates | 31.000ms | 34.726ms | 34.775ms | 0.0% | 34.775ms | 34.775ms | 34.775ms | 34.775ms | 20.000ms | 7.50 MiB | none | 28.76M/s |
| q4 | average | 31.000ms | 33.743ms | 34.804ms | 0.0% | 34.804ms | 34.804ms | 34.804ms | 34.804ms | 30.000ms | 9.07 MiB | none | 28.73M/s |
| q5 | count distinct, high card | 401.000ms | 468.903ms | 407.592ms | 0.0% | 407.592ms | 407.592ms | 407.592ms | 407.592ms | 390.000ms | 98.10 MiB | none | 2.45M/s |
| q6 | count distinct, strings | 173.000ms | 163.820ms | 177.715ms | 0.0% | 177.715ms | 177.715ms | 177.715ms | 177.715ms | 170.000ms | 25.19 MiB | none | 5.63M/s |
| q7 | min and max of a date | 14.000ms | 16.674ms | 17.270ms | 0.0% | 17.270ms | 17.270ms | 17.270ms | 17.270ms | 10.000ms | 5.95 MiB | none | 57.90M/s |
| q8 | group by, low card | 11.000ms | 14.913ms | 15.034ms | 0.0% | 15.034ms | 15.034ms | 15.034ms | 15.034ms | 0.000us | 6.14 MiB | none | 66.51M/s |
| q9 | group by and count distinct | 608.000ms | 564.597ms | 616.907ms | 0.0% | 616.907ms | 616.907ms | 616.907ms | 616.907ms | 610.000ms | 110.86 MiB | none | 1.62M/s |
| q10 | group by, several aggregates | 576.000ms | 643.850ms | 584.596ms | 0.0% | 584.596ms | 584.596ms | 584.596ms | 584.596ms | 570.000ms | 112.83 MiB | 648.00 KiB | 1.71M/s |
| q11 | group by a string and count distinct | 50.000ms | 63.387ms | 54.055ms | 0.0% | 54.055ms | 54.055ms | 54.055ms | 54.055ms | 40.000ms | 15.34 MiB | none | 18.50M/s |
| q12 | group by two strings and count distinct | 73.000ms | 81.886ms | 77.294ms | 0.0% | 77.294ms | 77.294ms | 77.294ms | 77.294ms | 70.000ms | 16.46 MiB | none | 12.94M/s |
| q13 | group by a string and top k | 207.000ms | 227.407ms | 210.800ms | 0.0% | 210.800ms | 210.800ms | 210.800ms | 210.800ms | 190.000ms | 42.21 MiB | none | 4.74M/s |
| q14 | group by a string and count distinct | 267.000ms | 286.728ms | 270.955ms | 0.0% | 270.955ms | 270.955ms | 270.955ms | 270.955ms | 260.000ms | 62.04 MiB | none | 3.69M/s |
| q15 | group by two columns and top k | 250.000ms | 233.144ms | 253.833ms | 0.0% | 253.833ms | 253.833ms | 253.833ms | 253.833ms | 250.000ms | 50.34 MiB | none | 3.94M/s |
| q16 | group by, very high card | 389.000ms | 458.434ms | 393.437ms | 0.0% | 393.437ms | 393.437ms | 393.437ms | 393.437ms | 380.000ms | 184.04 MiB | none | 2.54M/s |
| q17 | group by two, very high card | 691.000ms | 710.830ms | 699.955ms | 0.0% | 699.955ms | 699.955ms | 699.955ms | 699.955ms | 690.000ms | 271.54 MiB | none | 1.43M/s |
| q18 | group by two, no ordering | 530.000ms | 506.923ms | 537.564ms | 0.0% | 537.564ms | 537.564ms | 537.564ms | 537.564ms | 530.000ms | 271.55 MiB | none | 1.86M/s |
| q20 | point lookup | 23.000ms | 34.700ms | 25.875ms | 0.0% | 25.875ms | 25.875ms | 25.875ms | 25.875ms | 10.000ms | 9.01 MiB | none | 38.65M/s |
| q21 | substring scan | 458.000ms | 492.811ms | 461.909ms | 0.0% | 461.909ms | 461.909ms | 461.909ms | 461.909ms | 450.000ms | 40.15 MiB | none | 2.16M/s |
| q22 | substring scan and group by | 566.000ms | 572.218ms | 570.243ms | 0.0% | 570.243ms | 570.243ms | 570.243ms | 570.243ms | 560.000ms | 46.71 MiB | none | 1.75M/s |
| q23 | two substring scans and group by | 1.092s | 1.190s | 1.102s | 0.0% | 1.102s | 1.102s | 1.102s | 1.102s | 1.090s | 69.20 MiB | none | 907.82K/s |
| q24 | select star and top k | 2.649s | 2.482s | 2.665s | 0.0% | 2.665s | 2.665s | 2.665s | 2.665s | 2.660s | 213.66 MiB | none | 375.16K/s |
| q25 | top k by a date | 220.000ms | 237.664ms | 223.831ms | 0.0% | 223.831ms | 223.831ms | 223.831ms | 223.831ms | 210.000ms | 13.11 MiB | none | 4.47M/s |
| q26 | top k by a string | 187.000ms | 176.223ms | 190.713ms | 0.0% | 190.713ms | 190.713ms | 190.713ms | 190.713ms | 180.000ms | 11.09 MiB | none | 5.24M/s |
| q27 | top k by two columns | 235.000ms | 234.028ms | 239.086ms | 0.0% | 239.086ms | 239.086ms | 239.086ms | 239.086ms | 230.000ms | 13.17 MiB | none | 4.18M/s |
| q28 | group by with a string length | 497.000ms | 476.196ms | 501.807ms | 0.0% | 501.807ms | 501.807ms | 501.807ms | 501.807ms | 490.000ms | 48.75 MiB | none | 1.99M/s |
| q29 | group by a regular expression | 1.461s | 1.340s | 1.466s | 0.0% | 1.466s | 1.466s | 1.466s | 1.466s | 1.450s | 96.97 MiB | none | 681.91K/s |
| q30 | ninety sums over one column | 238.000ms | 241.884ms | 241.479ms | 0.0% | 241.479ms | 241.479ms | 241.479ms | 241.479ms | 230.000ms | 6.93 MiB | none | 4.14M/s |
| q31 | group by two and several aggregates | 275.000ms | 258.398ms | 279.131ms | 0.0% | 279.131ms | 279.131ms | 279.131ms | 279.131ms | 270.000ms | 70.25 MiB | none | 3.58M/s |
| q32 | group by a high card pair | 279.000ms | 256.162ms | 282.705ms | 0.0% | 282.705ms | 282.705ms | 282.705ms | 282.705ms | 270.000ms | 74.93 MiB | none | 3.54M/s |
| q34 | group by a long string | 939.000ms | 973.895ms | 959.057ms | 0.0% | 959.057ms | 959.057ms | 959.057ms | 959.057ms | 950.000ms | 253.49 MiB | none | 1.04M/s |
| q35 | group by a constant and a long string | 1.064s | 1.084s | 1.076s | 0.0% | 1.076s | 1.076s | 1.076s | 1.076s | 1.070s | 286.15 MiB | none | 929.02K/s |
| q36 | group by four expressions | 660.000ms | 694.398ms | 665.274ms | 0.0% | 665.274ms | 665.274ms | 665.274ms | 665.274ms | 650.000ms | 275.54 MiB | none | 1.50M/s |
| q37 | date range and group by a URL | 497.000ms | 478.287ms | 504.358ms | 0.0% | 504.358ms | 504.358ms | 504.358ms | 504.358ms | 500.000ms | 52.36 MiB | none | 1.98M/s |
| q38 | date range and group by a title | 454.000ms | 475.432ms | 458.993ms | 0.0% | 458.993ms | 458.993ms | 458.993ms | 458.993ms | 440.000ms | 40.59 MiB | none | 2.18M/s |
| q39 | date range, group by and offset | 481.000ms | 473.760ms | 486.558ms | 0.0% | 486.558ms | 486.558ms | 486.558ms | 486.558ms | 470.000ms | 48.71 MiB | none | 2.06M/s |
| q40 | date range, a case and a wide group by | 835.000ms | 933.362ms | 840.814ms | 0.0% | 840.814ms | 840.814ms | 840.814ms | 840.814ms | 830.000ms | 72.64 MiB | none | 1.19M/s |
| q41 | date range with an IN and a hash | 106.000ms | 80.628ms | 110.098ms | 0.0% | 110.098ms | 110.098ms | 110.098ms | 110.098ms | 100.000ms | 15.18 MiB | none | 9.08M/s |
| q42 | date range and a deep offset | 91.000ms | 81.095ms | 94.774ms | 0.0% | 94.774ms | 94.774ms | 94.774ms | 94.774ms | 90.000ms | 14.89 MiB | none | 10.55M/s |
| q43 | minute buckets over a date range | 74.000ms | 80.048ms | 77.948ms | 0.0% | 77.948ms | 77.948ms | 77.948ms | 77.948ms | 70.000ms | 12.17 MiB | none | 12.83M/s |

rudb rudb 0.2.31 over 41 of 43 queries. Total 17.694s by its own clock and 17.929s by ours, 17.876s cold, 17.480s of CPU, peak 286.15 MiB, 2.32M/s and 321.74 MiB/s.

Running it cost 1% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 710.42x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

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

