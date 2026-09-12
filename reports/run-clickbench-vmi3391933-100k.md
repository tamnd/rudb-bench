# clickbench on vmi3391933

This is one run of the clickbench suite on vmi3391933, over 7 engines and 43 queries, with 1 hot run of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

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

| engine | version | state | load | load cpu | on disk | that size is | format |
| --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 2.685s | 3.140s | 31.26 MiB | its own database file | its own |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 3.148s | 3.670s | 28.76 MiB | its own database file | its own |
| clickhouse-local | 26.9.1.1138 | ran | 4.957s | 2.980s | 24.32 MiB | its own MergeTree parts, as system.parts counts the active ones | its own |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 15.11 MiB | the source Parquet, this engine has no storage format of its own | the Parquet |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 15.11 MiB | the source Parquet, this engine has no storage format of its own | the Parquet |
| clickhouse-server | 26.9.1.1138 | ran | 2.590s | not read | 23.93 MiB | its own MergeTree parts, sorted by hits by (CounterID, EventDate, UserID, EventTime, WatchID), as system.parts counts the active ones | its own |
| rudb | rudb 0.2.28 | ran | 0.000us | 0.000us | 15.11 MiB | the source Parquet, this engine has no storage format of its own yet | the Parquet |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 1.834s | 4.911s | +168% | 4.559s | 7.330s | 1.49 | 63.75 MiB | none | 2.34M/s | 354.37 MiB/s | 1.00x |
| duckdb-pinned | 2.040s | 8.284s | +306% | 8.772s | 9.330s | 1.13 | 75.77 MiB | none | 2.11M/s | 318.58 MiB/s | 1.11x |
| clickhouse-local | 4.439s | 33.021s | +644% | 35.267s | 32.220s | 0.98 | 286.02 MiB | none | 968.67K/s | 146.41 MiB/s | 2.42x |
| datafusion | 3.621s | 7.688s | +112% | 8.235s | 8.420s | 1.10 | 182.08 MiB | none | 1.19M/s | 179.48 MiB/s | 1.97x |
| polars | 6.104s | 36.543s | +499% | 39.042s | 26.910s | 0.74 | 120.02 MiB | 2.74 MiB | 638.90K/s | 96.57 MiB/s | 3.33x |
| clickhouse-server | 1.572s | 12.994s | +727% | 13.431s | not read | not read | not read | not read | 2.74M/s | 413.43 MiB/s | 0.86x |
| rudb | 4.812s | 5.626s | +17% | 5.955s | 4.720s | 0.84 | 158.39 MiB | none | 852.02K/s | 128.78 MiB/s | 2.62x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | clickhouse-server | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 2.000ms | 2.000ms | 64.000ms | 10.000ms | 50.765ms | 6.000ms | 1.000ms |
| q2 | filtered count | 6.000ms | 3.000ms | 105.000ms | 54.000ms | 43.312ms | 5.000ms | 3.000ms |
| q3 | three aggregates | 4.000ms | 4.000ms | 70.000ms | 10.000ms | 25.724ms | 7.000ms | 8.000ms |
| q4 | average | 6.000ms | 7.000ms | 33.000ms | 9.000ms | 138.264ms | 8.000ms | 7.000ms |
| q5 | count distinct, high card | 40.000ms | 29.000ms | 107.000ms | 47.000ms | 36.673ms | 25.000ms | 89.000ms |
| q6 | count distinct, strings | 61.000ms | 11.000ms | 105.000ms | 166.000ms | 156.034ms | 27.000ms | 55.000ms |
| q7 | min and max of a date | 5.000ms | 3.000ms | 43.000ms | 9.000ms | 68.250ms | 10.000ms | 10.000ms |
| q8 | group by, low card | 7.000ms | 25.000ms | 110.000ms | 64.000ms | 34.992ms | 7.000ms | 6.000ms |
| q9 | group by and count distinct | 68.000ms | 31.000ms | 139.000ms | 61.000ms | 65.348ms | 19.000ms | 181.000ms |
| q10 | group by, several aggregates | 46.000ms | 63.000ms | 87.000ms | 28.000ms | 134.127ms | 73.000ms | 204.000ms |
| q11 | group by a string and count distinct | 21.000ms | 18.000ms | 98.000ms | 27.000ms | 70.732ms | 8.000ms | 14.000ms |
| q12 | group by two strings and count distinct | 29.000ms | 24.000ms | 76.000ms | 21.000ms | 58.629ms | 18.000ms | 15.000ms |
| q13 | group by a string and top k | 14.000ms | 14.000ms | 66.000ms | 44.000ms | 102.377ms | 29.000ms | 43.000ms |
| q14 | group by a string and count distinct | 25.000ms | 25.000ms | 90.000ms | 77.000ms | 102.786ms | 42.000ms | 51.000ms |
| q15 | group by two columns and top k | 70.000ms | 15.000ms | 120.000ms | 56.000ms | 133.577ms | 29.000ms | 44.000ms |
| q16 | group by, very high card | 45.000ms | 41.000ms | 84.000ms | 34.000ms | 68.726ms | 23.000ms | 119.000ms |
| q17 | group by two, very high card | 51.000ms | 45.000ms | 260.000ms | 346.000ms | 118.950ms | 66.000ms | 123.000ms |
| q18 | group by two, no ordering | 35.000ms | 37.000ms | 57.000ms | 43.000ms | 77.185ms | 13.000ms | 126.000ms |
| q19 | group by with an extract | 213.000ms | 50.000ms | 158.000ms | 62.000ms | 186.472ms | 83.000ms | no dialect |
| q20 | point lookup | 3.000ms | 5.000ms | 54.000ms | 11.000ms | 38.024ms | 5.000ms | 16.000ms |
| q21 | substring scan | 33.000ms | 67.000ms | 137.000ms | 48.000ms | 79.561ms | 29.000ms | 162.000ms |
| q22 | substring scan and group by | 42.000ms | 134.000ms | 99.000ms | 55.000ms | 228.290ms | 16.000ms | 226.000ms |
| q23 | two substring scans and group by | 37.000ms | 113.000ms | 131.000ms | 97.000ms | 404.858ms | 33.000ms | 419.000ms |
| q24 | select star and top k | 128.000ms | 143.000ms | 506.000ms | 953.000ms | 484.297ms | 264.000ms | 676.000ms |
| q25 | top k by a date | 13.000ms | 18.000ms | 76.000ms | 47.000ms | 361.675ms | 8.000ms | 39.000ms |
| q26 | top k by a string | 18.000ms | 12.000ms | 34.000ms | 25.000ms | 79.990ms | 7.000ms | 33.000ms |
| q27 | top k by two columns | 14.000ms | 14.000ms | 42.000ms | 42.000ms | 116.938ms | 14.000ms | 49.000ms |
| q28 | group by with a string length | 47.000ms | 48.000ms | 39.000ms | 109.000ms | no dialect | 10.000ms | 121.000ms |
| q29 | group by a regular expression | 207.000ms | 370.000ms | 228.000ms | 142.000ms | no dialect | 204.000ms | 284.000ms |
| q30 | ninety sums over one column | 24.000ms | 74.000ms | 50.000ms | 71.000ms | 105.340ms | 49.000ms | 55.000ms |
| q31 | group by two and several aggregates | 30.000ms | 22.000ms | 29.000ms | 86.000ms | 55.722ms | 66.000ms | 64.000ms |
| q32 | group by a high card pair | 22.000ms | 20.000ms | 38.000ms | 67.000ms | 170.071ms | 42.000ms | 57.000ms |
| q33 | group by a high card pair, unfiltered | 50.000ms | 105.000ms | 155.000ms | 51.000ms | 236.754ms | 49.000ms | no dialect |
| q34 | group by a long string | 96.000ms | 101.000ms | 114.000ms | 122.000ms | 375.904ms | 62.000ms | 211.000ms |
| q35 | group by a constant and a long string | 90.000ms | 117.000ms | 137.000ms | 118.000ms | 733.854ms | 69.000ms | 245.000ms |
| q36 | group by four expressions | 51.000ms | 24.000ms | 42.000ms | 36.000ms | no dialect | 17.000ms | 163.000ms |
| q37 | date range and group by a URL | 11.000ms | 19.000ms | 169.000ms | 54.000ms | 296.787ms | 16.000ms | 153.000ms |
| q38 | date range and group by a title | 14.000ms | 13.000ms | 66.000ms | 88.000ms | 264.418ms | 29.000ms | 196.000ms |
| q39 | date range, group by and offset | 37.000ms | 20.000ms | 97.000ms | 49.000ms | 97.984ms | 12.000ms | 173.000ms |
| q40 | date range, a case and a wide group by | 57.000ms | 80.000ms | 77.000ms | 78.000ms | 155.452ms | 26.000ms | 288.000ms |
| q41 | date range with an IN and a hash | 23.000ms | 16.000ms | 107.000ms | 21.000ms | 56.659ms | 20.000ms | 30.000ms |
| q42 | date range and a deep offset | 26.000ms | 40.000ms | 111.000ms | 37.000ms | 88.632ms | 14.000ms | 31.000ms |
| q43 | minute buckets over a date range | 13.000ms | 18.000ms | 29.000ms | 46.000ms | no dialect | 13.000ms | 22.000ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 2.000ms | 52.791ms | 54.573ms | 0.0% | 54.573ms | 54.573ms | 54.573ms | 54.573ms | 50.000ms | 25.88 MiB | none | 1.83M/s |
| q2 | filtered count | 6.000ms | 47.657ms | 59.234ms | 0.0% | 59.234ms | 59.234ms | 59.234ms | 59.234ms | 70.000ms | 26.62 MiB | none | 1.69M/s |
| q3 | three aggregates | 4.000ms | 51.828ms | 45.714ms | 0.0% | 45.714ms | 45.714ms | 45.714ms | 45.714ms | 40.000ms | 27.12 MiB | none | 2.19M/s |
| q4 | average | 6.000ms | 51.611ms | 80.872ms | 0.0% | 80.872ms | 80.872ms | 80.872ms | 80.872ms | 70.000ms | 27.25 MiB | none | 1.24M/s |
| q5 | count distinct, high card | 40.000ms | 85.789ms | 93.182ms | 0.0% | 93.182ms | 93.182ms | 93.182ms | 93.182ms | 150.000ms | 32.75 MiB | none | 1.07M/s |
| q6 | count distinct, strings | 61.000ms | 111.192ms | 188.390ms | 0.0% | 188.390ms | 188.390ms | 188.390ms | 188.390ms | 240.000ms | 30.50 MiB | none | 530.80K/s |
| q7 | min and max of a date | 5.000ms | 45.236ms | 81.063ms | 0.0% | 81.063ms | 81.063ms | 81.063ms | 81.063ms | 90.000ms | 25.88 MiB | none | 1.23M/s |
| q8 | group by, low card | 7.000ms | 50.780ms | 53.909ms | 0.0% | 53.909ms | 53.909ms | 53.909ms | 53.909ms | 50.000ms | 28.25 MiB | none | 1.85M/s |
| q9 | group by and count distinct | 68.000ms | 96.593ms | 152.148ms | 0.0% | 152.148ms | 152.148ms | 152.148ms | 152.148ms | 420.000ms | 38.38 MiB | none | 657.24K/s |
| q10 | group by, several aggregates | 46.000ms | 116.236ms | 134.788ms | 0.0% | 134.788ms | 134.788ms | 134.788ms | 134.788ms | 160.000ms | 41.62 MiB | none | 741.89K/s |
| q11 | group by a string and count distinct | 21.000ms | 79.874ms | 88.329ms | 0.0% | 88.329ms | 88.329ms | 88.329ms | 88.329ms | 140.000ms | 34.62 MiB | none | 1.13M/s |
| q12 | group by two strings and count distinct | 29.000ms | 101.254ms | 86.003ms | 0.0% | 86.003ms | 86.003ms | 86.003ms | 86.003ms | 100.000ms | 34.25 MiB | none | 1.16M/s |
| q13 | group by a string and top k | 14.000ms | 64.471ms | 78.604ms | 0.0% | 78.604ms | 78.604ms | 78.604ms | 78.604ms | 80.000ms | 32.50 MiB | none | 1.27M/s |
| q14 | group by a string and count distinct | 25.000ms | 69.969ms | 86.151ms | 0.0% | 86.151ms | 86.151ms | 86.151ms | 86.151ms | 130.000ms | 42.12 MiB | none | 1.16M/s |
| q15 | group by two columns and top k | 70.000ms | 65.464ms | 139.977ms | 0.0% | 139.977ms | 139.977ms | 139.977ms | 139.977ms | 170.000ms | 32.75 MiB | none | 714.39K/s |
| q16 | group by, very high card | 45.000ms | 126.253ms | 108.561ms | 0.0% | 108.561ms | 108.561ms | 108.561ms | 108.561ms | 130.000ms | 37.38 MiB | none | 921.12K/s |
| q17 | group by two, very high card | 51.000ms | 92.346ms | 98.228ms | 0.0% | 98.228ms | 98.228ms | 98.228ms | 98.228ms | 140.000ms | 41.62 MiB | none | 1.02M/s |
| q18 | group by two, no ordering | 35.000ms | 150.217ms | 86.166ms | 0.0% | 86.166ms | 86.166ms | 86.166ms | 86.166ms | 120.000ms | 42.38 MiB | none | 1.16M/s |
| q19 | group by with an extract | 213.000ms | 177.318ms | 472.735ms | 0.0% | 472.735ms | 472.735ms | 472.735ms | 472.735ms | 1.340s | 43.75 MiB | none | 211.53K/s |
| q20 | point lookup | 3.000ms | 92.145ms | 53.007ms | 0.0% | 53.007ms | 53.007ms | 53.007ms | 53.007ms | 40.000ms | 26.88 MiB | none | 1.89M/s |
| q21 | substring scan | 33.000ms | 132.751ms | 107.570ms | 0.0% | 107.570ms | 107.570ms | 107.570ms | 107.570ms | 120.000ms | 33.00 MiB | none | 929.61K/s |
| q22 | substring scan and group by | 42.000ms | 133.164ms | 146.008ms | 0.0% | 146.008ms | 146.008ms | 146.008ms | 146.008ms | 150.000ms | 36.25 MiB | none | 684.88K/s |
| q23 | two substring scans and group by | 37.000ms | 145.323ms | 83.230ms | 0.0% | 83.230ms | 83.230ms | 83.230ms | 83.230ms | 80.000ms | 40.12 MiB | none | 1.20M/s |
| q24 | select star and top k | 128.000ms | 208.574ms | 196.811ms | 0.0% | 196.811ms | 196.811ms | 196.811ms | 196.811ms | 300.000ms | 63.75 MiB | none | 508.09K/s |
| q25 | top k by a date | 13.000ms | 89.345ms | 107.887ms | 0.0% | 107.887ms | 107.887ms | 107.887ms | 107.887ms | 130.000ms | 30.75 MiB | none | 926.88K/s |
| q26 | top k by a string | 18.000ms | 50.492ms | 74.297ms | 0.0% | 74.297ms | 74.297ms | 74.297ms | 74.297ms | 70.000ms | 27.38 MiB | none | 1.35M/s |
| q27 | top k by two columns | 14.000ms | 123.971ms | 76.966ms | 0.0% | 76.966ms | 76.966ms | 76.966ms | 76.966ms | 80.000ms | 28.50 MiB | none | 1.30M/s |
| q28 | group by with a string length | 47.000ms | 117.049ms | 87.872ms | 0.0% | 87.872ms | 87.872ms | 87.872ms | 87.872ms | 90.000ms | 35.88 MiB | none | 1.14M/s |
| q29 | group by a regular expression | 207.000ms | 360.067ms | 259.815ms | 0.0% | 259.815ms | 259.815ms | 259.815ms | 259.815ms | 490.000ms | 45.88 MiB | none | 384.88K/s |
| q30 | ninety sums over one column | 24.000ms | 73.981ms | 82.156ms | 0.0% | 82.156ms | 82.156ms | 82.156ms | 82.156ms | 60.000ms | 29.88 MiB | none | 1.22M/s |
| q31 | group by two and several aggregates | 30.000ms | 70.828ms | 92.136ms | 0.0% | 92.136ms | 92.136ms | 92.136ms | 92.136ms | 170.000ms | 34.50 MiB | none | 1.09M/s |
| q32 | group by a high card pair | 22.000ms | 65.059ms | 74.038ms | 0.0% | 74.038ms | 74.038ms | 74.038ms | 74.038ms | 90.000ms | 35.88 MiB | none | 1.35M/s |
| q33 | group by a high card pair, unfiltered | 50.000ms | 80.313ms | 107.598ms | 0.0% | 107.598ms | 107.598ms | 107.598ms | 107.598ms | 180.000ms | 43.25 MiB | none | 929.37K/s |
| q34 | group by a long string | 96.000ms | 135.088ms | 231.989ms | 0.0% | 231.989ms | 231.989ms | 231.989ms | 231.989ms | 280.000ms | 53.00 MiB | none | 431.05K/s |
| q35 | group by a constant and a long string | 90.000ms | 138.197ms | 140.642ms | 0.0% | 140.642ms | 140.642ms | 140.642ms | 140.642ms | 250.000ms | 54.00 MiB | none | 711.01K/s |
| q36 | group by four expressions | 51.000ms | 122.844ms | 119.543ms | 0.0% | 119.543ms | 119.543ms | 119.543ms | 119.543ms | 140.000ms | 37.50 MiB | none | 836.50K/s |
| q37 | date range and group by a URL | 11.000ms | 108.193ms | 88.662ms | 0.0% | 88.662ms | 88.662ms | 88.662ms | 88.662ms | 80.000ms | 30.75 MiB | none | 1.13M/s |
| q38 | date range and group by a title | 14.000ms | 93.589ms | 81.934ms | 0.0% | 81.934ms | 81.934ms | 81.934ms | 81.934ms | 130.000ms | 30.88 MiB | none | 1.22M/s |
| q39 | date range, group by and offset | 37.000ms | 58.784ms | 105.892ms | 0.0% | 105.892ms | 105.892ms | 105.892ms | 105.892ms | 170.000ms | 30.75 MiB | none | 944.34K/s |
| q40 | date range, a case and a wide group by | 57.000ms | 89.353ms | 128.937ms | 0.0% | 128.937ms | 128.937ms | 128.937ms | 128.937ms | 220.000ms | 32.75 MiB | none | 775.56K/s |
| q41 | date range with an IN and a hash | 23.000ms | 189.622ms | 117.003ms | 0.0% | 117.003ms | 117.003ms | 117.003ms | 117.003ms | 100.000ms | 31.88 MiB | none | 854.66K/s |
| q42 | date range and a deep offset | 26.000ms | 172.855ms | 82.815ms | 0.0% | 82.815ms | 82.815ms | 82.815ms | 82.815ms | 140.000ms | 31.25 MiB | none | 1.21M/s |
| q43 | minute buckets over a date range | 13.000ms | 70.404ms | 75.368ms | 0.0% | 75.368ms | 75.368ms | 75.368ms | 75.368ms | 80.000ms | 30.38 MiB | none | 1.33M/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 1.834s by its own clock and 4.911s by ours, 4.559s cold, 7.330s of CPU, peak 63.75 MiB, 2.34M/s and 354.37 MiB/s.

Running it cost 168% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 10.34x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 2.000ms | 114.852ms | 102.392ms | 0.0% | 102.392ms | 102.392ms | 102.392ms | 102.392ms | 120.000ms | 35.16 MiB | none | 976.62K/s |
| q2 | filtered count | 3.000ms | 93.413ms | 137.180ms | 0.0% | 137.180ms | 137.180ms | 137.180ms | 137.180ms | 140.000ms | 35.52 MiB | none | 728.95K/s |
| q3 | three aggregates | 4.000ms | 169.183ms | 114.797ms | 0.0% | 114.797ms | 114.797ms | 114.797ms | 114.797ms | 130.000ms | 35.78 MiB | none | 871.09K/s |
| q4 | average | 7.000ms | 191.384ms | 158.143ms | 0.0% | 158.143ms | 158.143ms | 158.143ms | 158.143ms | 160.000ms | 36.28 MiB | none | 632.33K/s |
| q5 | count distinct, high card | 29.000ms | 141.284ms | 150.904ms | 0.0% | 150.904ms | 150.904ms | 150.904ms | 150.904ms | 150.000ms | 43.03 MiB | none | 662.66K/s |
| q6 | count distinct, strings | 11.000ms | 120.544ms | 86.330ms | 0.0% | 86.330ms | 86.330ms | 86.330ms | 86.330ms | 80.000ms | 38.78 MiB | none | 1.16M/s |
| q7 | min and max of a date | 3.000ms | 124.582ms | 114.367ms | 0.0% | 114.367ms | 114.367ms | 114.367ms | 114.367ms | 140.000ms | 35.40 MiB | none | 874.36K/s |
| q8 | group by, low card | 25.000ms | 194.602ms | 119.060ms | 0.0% | 119.060ms | 119.060ms | 119.060ms | 119.060ms | 130.000ms | 38.15 MiB | none | 839.90K/s |
| q9 | group by and count distinct | 31.000ms | 198.733ms | 130.754ms | 0.0% | 130.754ms | 130.754ms | 130.754ms | 130.754ms | 150.000ms | 48.15 MiB | none | 764.78K/s |
| q10 | group by, several aggregates | 63.000ms | 255.874ms | 218.115ms | 0.0% | 218.115ms | 218.115ms | 218.115ms | 218.115ms | 330.000ms | 50.90 MiB | none | 458.46K/s |
| q11 | group by a string and count distinct | 18.000ms | 130.386ms | 106.950ms | 0.0% | 106.950ms | 106.950ms | 106.950ms | 106.950ms | 90.000ms | 41.94 MiB | none | 935.00K/s |
| q12 | group by two strings and count distinct | 24.000ms | 147.284ms | 188.354ms | 0.0% | 188.354ms | 188.354ms | 188.354ms | 188.354ms | 300.000ms | 41.89 MiB | none | 530.90K/s |
| q13 | group by a string and top k | 14.000ms | 116.970ms | 137.511ms | 0.0% | 137.511ms | 137.511ms | 137.511ms | 137.511ms | 120.000ms | 40.55 MiB | none | 727.20K/s |
| q14 | group by a string and count distinct | 25.000ms | 128.991ms | 142.308ms | 0.0% | 142.308ms | 142.308ms | 142.308ms | 142.308ms | 130.000ms | 46.15 MiB | none | 702.69K/s |
| q15 | group by two columns and top k | 15.000ms | 135.239ms | 114.613ms | 0.0% | 114.613ms | 114.613ms | 114.613ms | 114.613ms | 120.000ms | 41.40 MiB | none | 872.48K/s |
| q16 | group by, very high card | 41.000ms | 131.908ms | 180.381ms | 0.0% | 180.381ms | 180.381ms | 180.381ms | 180.381ms | 270.000ms | 45.40 MiB | none | 554.37K/s |
| q17 | group by two, very high card | 45.000ms | 242.030ms | 137.838ms | 0.0% | 137.838ms | 137.838ms | 137.838ms | 137.838ms | 170.000ms | 49.52 MiB | none | 725.47K/s |
| q18 | group by two, no ordering | 37.000ms | 186.491ms | 134.074ms | 0.0% | 134.074ms | 134.074ms | 134.074ms | 134.074ms | 130.000ms | 50.53 MiB | none | 745.84K/s |
| q19 | group by with an extract | 50.000ms | 157.049ms | 184.710ms | 0.0% | 184.710ms | 184.710ms | 184.710ms | 184.710ms | 220.000ms | 56.27 MiB | none | 541.38K/s |
| q20 | point lookup | 5.000ms | 154.053ms | 356.847ms | 0.0% | 356.847ms | 356.847ms | 356.847ms | 356.847ms | 330.000ms | 35.27 MiB | none | 280.23K/s |
| q21 | substring scan | 67.000ms | 333.989ms | 260.822ms | 0.0% | 260.822ms | 260.822ms | 260.822ms | 260.822ms | 240.000ms | 42.40 MiB | none | 383.40K/s |
| q22 | substring scan and group by | 134.000ms | 221.470ms | 282.135ms | 0.0% | 282.135ms | 282.135ms | 282.135ms | 282.135ms | 280.000ms | 46.27 MiB | none | 354.43K/s |
| q23 | two substring scans and group by | 113.000ms | 310.492ms | 261.520ms | 0.0% | 261.520ms | 261.520ms | 261.520ms | 261.520ms | 330.000ms | 52.00 MiB | none | 382.37K/s |
| q24 | select star and top k | 143.000ms | 398.495ms | 282.880ms | 0.0% | 282.880ms | 282.880ms | 282.880ms | 282.880ms | 350.000ms | 75.77 MiB | none | 353.50K/s |
| q25 | top k by a date | 18.000ms | 104.963ms | 108.625ms | 0.0% | 108.625ms | 108.625ms | 108.625ms | 108.625ms | 100.000ms | 38.28 MiB | none | 920.58K/s |
| q26 | top k by a string | 12.000ms | 139.925ms | 179.554ms | 0.0% | 179.554ms | 179.554ms | 179.554ms | 179.554ms | 170.000ms | 37.65 MiB | none | 556.92K/s |
| q27 | top k by two columns | 14.000ms | 126.992ms | 158.669ms | 0.0% | 158.669ms | 158.669ms | 158.669ms | 158.669ms | 110.000ms | 38.28 MiB | none | 630.23K/s |
| q28 | group by with a string length | 48.000ms | 425.905ms | 185.295ms | 0.0% | 185.295ms | 185.295ms | 185.295ms | 185.295ms | 170.000ms | 45.14 MiB | none | 539.67K/s |
| q29 | group by a regular expression | 370.000ms | 458.394ms | 496.979ms | 0.0% | 496.979ms | 496.979ms | 496.979ms | 496.979ms | 760.000ms | 50.14 MiB | none | 201.21K/s |
| q30 | ninety sums over one column | 74.000ms | 320.923ms | 228.819ms | 0.0% | 228.819ms | 228.819ms | 228.819ms | 228.819ms | 200.000ms | 47.64 MiB | none | 437.02K/s |
| q31 | group by two and several aggregates | 22.000ms | 194.766ms | 155.104ms | 0.0% | 155.104ms | 155.104ms | 155.104ms | 155.104ms | 210.000ms | 43.77 MiB | none | 644.72K/s |
| q32 | group by a high card pair | 20.000ms | 147.207ms | 151.165ms | 0.0% | 151.165ms | 151.165ms | 151.165ms | 151.165ms | 140.000ms | 44.27 MiB | none | 661.52K/s |
| q33 | group by a high card pair, unfiltered | 105.000ms | 178.353ms | 230.759ms | 0.0% | 230.759ms | 230.759ms | 230.759ms | 230.759ms | 410.000ms | 55.40 MiB | none | 433.34K/s |
| q34 | group by a long string | 101.000ms | 247.575ms | 349.782ms | 0.0% | 349.782ms | 349.782ms | 349.782ms | 349.782ms | 480.000ms | 60.65 MiB | none | 285.89K/s |
| q35 | group by a constant and a long string | 117.000ms | 317.577ms | 259.531ms | 0.0% | 259.531ms | 259.531ms | 259.531ms | 259.531ms | 350.000ms | 62.15 MiB | none | 385.30K/s |
| q36 | group by four expressions | 24.000ms | 246.776ms | 201.282ms | 0.0% | 201.282ms | 201.282ms | 201.282ms | 201.282ms | 180.000ms | 43.65 MiB | none | 496.81K/s |
| q37 | date range and group by a URL | 19.000ms | 285.318ms | 284.007ms | 0.0% | 284.007ms | 284.007ms | 284.007ms | 284.007ms | 290.000ms | 41.15 MiB | none | 352.10K/s |
| q38 | date range and group by a title | 13.000ms | 196.359ms | 260.960ms | 0.0% | 260.960ms | 260.960ms | 260.960ms | 260.960ms | 250.000ms | 40.09 MiB | none | 383.19K/s |
| q39 | date range, group by and offset | 20.000ms | 200.739ms | 133.727ms | 0.0% | 133.727ms | 133.727ms | 133.727ms | 133.727ms | 140.000ms | 40.90 MiB | none | 747.78K/s |
| q40 | date range, a case and a wide group by | 80.000ms | 202.124ms | 252.291ms | 0.0% | 252.291ms | 252.291ms | 252.291ms | 252.291ms | 290.000ms | 45.03 MiB | none | 396.36K/s |
| q41 | date range with an IN and a hash | 16.000ms | 166.070ms | 185.425ms | 0.0% | 185.425ms | 185.425ms | 185.425ms | 185.425ms | 140.000ms | 41.14 MiB | none | 539.29K/s |
| q42 | date range and a deep offset | 40.000ms | 257.335ms | 242.080ms | 0.0% | 242.080ms | 242.080ms | 242.080ms | 242.080ms | 220.000ms | 41.25 MiB | none | 413.08K/s |
| q43 | minute buckets over a date range | 18.000ms | 155.853ms | 117.151ms | 0.0% | 117.151ms | 117.151ms | 117.151ms | 117.151ms | 110.000ms | 39.28 MiB | none | 853.58K/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 2.040s by its own clock and 8.284s by ours, 8.772s cold, 9.330s of CPU, peak 75.77 MiB, 2.11M/s and 318.58 MiB/s.

Running it cost 306% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 5.76x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 64.000ms | 887.311ms | 941.953ms | 0.0% | 941.953ms | 941.953ms | 941.953ms | 941.953ms | 1.180s | 232.62 MiB | 208.00 KiB | 106.16K/s |
| q2 | filtered count | 105.000ms | 949.046ms | 835.442ms | 0.0% | 835.442ms | 835.442ms | 835.442ms | 835.442ms | 570.000ms | 233.42 MiB | 276.00 KiB | 119.69K/s |
| q3 | three aggregates | 70.000ms | 999.893ms | 708.131ms | 0.0% | 708.131ms | 708.131ms | 708.131ms | 708.131ms | 440.000ms | 236.00 MiB | 992.00 KiB | 141.21K/s |
| q4 | average | 33.000ms | 914.636ms | 686.023ms | 0.0% | 686.023ms | 686.023ms | 686.023ms | 686.023ms | 440.000ms | 235.86 MiB | 4.00 KiB | 145.76K/s |
| q5 | count distinct, high card | 107.000ms | 940.229ms | 902.352ms | 0.0% | 902.352ms | 902.352ms | 902.352ms | 902.352ms | 1.040s | 240.12 MiB | 416.00 KiB | 110.82K/s |
| q6 | count distinct, strings | 105.000ms | 957.364ms | 996.284ms | 0.0% | 996.284ms | 996.284ms | 996.284ms | 996.284ms | 730.000ms | 238.75 MiB | 256.00 KiB | 100.37K/s |
| q7 | min and max of a date | 43.000ms | 849.696ms | 806.789ms | 0.0% | 806.789ms | 806.789ms | 806.789ms | 806.789ms | 650.000ms | 235.12 MiB | 480.00 KiB | 123.95K/s |
| q8 | group by, low card | 110.000ms | 916.403ms | 796.283ms | 0.0% | 796.283ms | 796.283ms | 796.283ms | 796.283ms | 590.000ms | 237.25 MiB | 1.60 MiB | 125.58K/s |
| q9 | group by and count distinct | 139.000ms | 684.345ms | 840.092ms | 0.0% | 840.092ms | 840.092ms | 840.092ms | 840.092ms | 510.000ms | 241.38 MiB | 500.00 KiB | 119.03K/s |
| q10 | group by, several aggregates | 87.000ms | 1.330s | 743.835ms | 0.0% | 743.835ms | 743.835ms | 743.835ms | 743.835ms | 580.000ms | 243.38 MiB | none | 134.44K/s |
| q11 | group by a string and count distinct | 98.000ms | 656.734ms | 955.525ms | 0.0% | 955.525ms | 955.525ms | 955.525ms | 955.525ms | 700.000ms | 238.50 MiB | 428.00 KiB | 104.65K/s |
| q12 | group by two strings and count distinct | 76.000ms | 844.382ms | 1.078s | 0.0% | 1.078s | 1.078s | 1.078s | 1.078s | 630.000ms | 239.50 MiB | 568.00 KiB | 92.80K/s |
| q13 | group by a string and top k | 66.000ms | 902.094ms | 822.133ms | 0.0% | 822.133ms | 822.133ms | 822.133ms | 822.133ms | 450.000ms | 242.50 MiB | 64.00 KiB | 121.63K/s |
| q14 | group by a string and count distinct | 90.000ms | 1.118s | 925.053ms | 0.0% | 925.053ms | 925.053ms | 925.053ms | 925.053ms | 450.000ms | 246.38 MiB | none | 108.10K/s |
| q15 | group by two columns and top k | 120.000ms | 821.794ms | 972.479ms | 0.0% | 972.479ms | 972.479ms | 972.479ms | 972.479ms | 550.000ms | 243.75 MiB | 332.00 KiB | 102.83K/s |
| q16 | group by, very high card | 84.000ms | 828.065ms | 720.933ms | 0.0% | 720.933ms | 720.933ms | 720.933ms | 720.933ms | 490.000ms | 243.62 MiB | 128.00 KiB | 138.71K/s |
| q17 | group by two, very high card | 260.000ms | 956.104ms | 1.239s | 0.0% | 1.239s | 1.239s | 1.239s | 1.239s | 720.000ms | 256.65 MiB | 68.00 KiB | 80.72K/s |
| q18 | group by two, no ordering | 57.000ms | 721.323ms | 850.227ms | 0.0% | 850.227ms | 850.227ms | 850.227ms | 850.227ms | 540.000ms | 242.50 MiB | 208.00 KiB | 117.61K/s |
| q19 | group by with an extract | 158.000ms | 1.017s | 837.141ms | 0.0% | 837.141ms | 837.141ms | 837.141ms | 837.141ms | 610.000ms | 255.46 MiB | none | 119.45K/s |
| q20 | point lookup | 54.000ms | 641.795ms | 952.421ms | 0.0% | 952.421ms | 952.421ms | 952.421ms | 952.421ms | 980.000ms | 236.38 MiB | 212.00 KiB | 104.99K/s |
| q21 | substring scan | 137.000ms | 772.095ms | 961.266ms | 0.0% | 961.266ms | 961.266ms | 961.266ms | 961.266ms | 1.100s | 241.00 MiB | none | 104.03K/s |
| q22 | substring scan and group by | 99.000ms | 754.382ms | 730.526ms | 0.0% | 730.526ms | 730.526ms | 730.526ms | 730.526ms | 520.000ms | 243.50 MiB | 92.00 KiB | 136.88K/s |
| q23 | two substring scans and group by | 131.000ms | 936.008ms | 978.527ms | 0.0% | 978.527ms | 978.527ms | 978.527ms | 978.527ms | 810.000ms | 245.38 MiB | none | 102.19K/s |
| q24 | select star and top k | 506.000ms | 1.250s | 1.092s | 0.0% | 1.092s | 1.092s | 1.092s | 1.092s | 1.010s | 286.02 MiB | 872.00 KiB | 91.59K/s |
| q25 | top k by a date | 76.000ms | 651.327ms | 709.503ms | 0.0% | 709.503ms | 709.503ms | 709.503ms | 709.503ms | 590.000ms | 239.14 MiB | none | 140.94K/s |
| q26 | top k by a string | 34.000ms | 736.993ms | 742.678ms | 0.0% | 742.678ms | 742.678ms | 742.678ms | 742.678ms | 530.000ms | 239.25 MiB | 224.00 KiB | 134.65K/s |
| q27 | top k by two columns | 42.000ms | 747.086ms | 673.343ms | 0.0% | 673.343ms | 673.343ms | 673.343ms | 673.343ms | 580.000ms | 239.88 MiB | none | 148.51K/s |
| q28 | group by with a string length | 39.000ms | 650.404ms | 691.197ms | 0.0% | 691.197ms | 691.197ms | 691.197ms | 691.197ms | 500.000ms | 239.40 MiB | 128.00 KiB | 144.67K/s |
| q29 | group by a regular expression | 228.000ms | 1.580s | 819.966ms | 0.0% | 819.966ms | 819.966ms | 819.966ms | 819.966ms | 730.000ms | 271.75 MiB | 14.99 MiB | 121.95K/s |
| q30 | ninety sums over one column | 50.000ms | 737.539ms | 612.721ms | 0.0% | 612.721ms | 612.721ms | 612.721ms | 612.721ms | 530.000ms | 239.75 MiB | 412.00 KiB | 163.20K/s |
| q31 | group by two and several aggregates | 29.000ms | 628.660ms | 620.492ms | 0.0% | 620.492ms | 620.492ms | 620.492ms | 620.492ms | 1.150s | 242.50 MiB | 216.00 KiB | 161.16K/s |
| q32 | group by a high card pair | 38.000ms | 547.730ms | 465.106ms | 0.0% | 465.106ms | 465.106ms | 465.106ms | 465.106ms | 1.230s | 244.14 MiB | none | 215.00K/s |
| q33 | group by a high card pair, unfiltered | 155.000ms | 379.374ms | 505.876ms | 0.0% | 505.876ms | 505.876ms | 505.876ms | 505.876ms | 690.000ms | 253.38 MiB | none | 197.67K/s |
| q34 | group by a long string | 114.000ms | 764.119ms | 643.495ms | 0.0% | 643.495ms | 643.495ms | 643.495ms | 643.495ms | 1.250s | 270.38 MiB | none | 155.40K/s |
| q35 | group by a constant and a long string | 137.000ms | 672.008ms | 478.404ms | 0.0% | 478.404ms | 478.404ms | 478.404ms | 478.404ms | 770.000ms | 270.25 MiB | none | 209.02K/s |
| q36 | group by four expressions | 42.000ms | 634.725ms | 566.379ms | 0.0% | 566.379ms | 566.379ms | 566.379ms | 566.379ms | 1.020s | 243.27 MiB | 256.00 KiB | 176.56K/s |
| q37 | date range and group by a URL | 169.000ms | 656.261ms | 776.048ms | 0.0% | 776.048ms | 776.048ms | 776.048ms | 776.048ms | 1.340s | 244.00 MiB | 556.00 KiB | 128.86K/s |
| q38 | date range and group by a title | 66.000ms | 603.771ms | 512.157ms | 0.0% | 512.157ms | 512.157ms | 512.157ms | 512.157ms | 650.000ms | 243.30 MiB | none | 195.25K/s |
| q39 | date range, group by and offset | 97.000ms | 751.685ms | 606.132ms | 0.0% | 606.132ms | 606.132ms | 606.132ms | 606.132ms | 740.000ms | 244.70 MiB | none | 164.98K/s |
| q40 | date range, a case and a wide group by | 77.000ms | 939.201ms | 535.801ms | 0.0% | 535.801ms | 535.801ms | 535.801ms | 535.801ms | 1.150s | 247.12 MiB | 108.00 KiB | 186.63K/s |
| q41 | date range with an IN and a hash | 107.000ms | 757.951ms | 593.295ms | 0.0% | 593.295ms | 593.295ms | 593.295ms | 593.295ms | 500.000ms | 243.14 MiB | 256.00 KiB | 168.55K/s |
| q42 | date range and a deep offset | 111.000ms | 522.319ms | 668.331ms | 0.0% | 668.331ms | 668.331ms | 668.331ms | 668.331ms | 1.480s | 241.88 MiB | 92.00 KiB | 149.62K/s |
| q43 | minute buckets over a date range | 29.000ms | 655.431ms | 428.973ms | 0.0% | 428.973ms | 428.973ms | 428.973ms | 428.973ms | 500.000ms | 242.03 MiB | 64.00 KiB | 233.11K/s |

clickhouse-local 26.9.1.1138 over 43 of 43 queries. Total 4.439s by its own clock and 33.021s by ours, 35.267s cold, 32.220s of CPU, peak 286.02 MiB, 968.67K/s and 146.41 MiB/s.

Running it cost 644% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.89x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 10.000ms | 253.802ms | 115.275ms | 0.0% | 115.275ms | 115.275ms | 115.275ms | 115.275ms | 90.000ms | 76.81 MiB | none | 867.47K/s |
| q2 | filtered count | 54.000ms | 389.555ms | 261.334ms | 0.0% | 261.334ms | 261.334ms | 261.334ms | 261.334ms | 240.000ms | 94.06 MiB | none | 382.64K/s |
| q3 | three aggregates | 10.000ms | 179.555ms | 66.817ms | 0.0% | 66.817ms | 66.817ms | 66.817ms | 66.817ms | 60.000ms | 90.01 MiB | none | 1.50M/s |
| q4 | average | 9.000ms | 94.008ms | 63.687ms | 0.0% | 63.687ms | 63.687ms | 63.687ms | 63.687ms | 50.000ms | 84.51 MiB | none | 1.57M/s |
| q5 | count distinct, high card | 47.000ms | 369.031ms | 112.340ms | 0.0% | 112.340ms | 112.340ms | 112.340ms | 112.340ms | 150.000ms | 129.49 MiB | none | 890.14K/s |
| q6 | count distinct, strings | 166.000ms | 135.910ms | 304.388ms | 0.0% | 304.388ms | 304.388ms | 304.388ms | 304.388ms | 290.000ms | 121.59 MiB | none | 328.52K/s |
| q7 | min and max of a date | 9.000ms | 107.680ms | 102.432ms | 0.0% | 102.432ms | 102.432ms | 102.432ms | 102.432ms | 60.000ms | 76.67 MiB | none | 976.24K/s |
| q8 | group by, low card | 64.000ms | 112.928ms | 228.326ms | 0.0% | 228.326ms | 228.326ms | 228.326ms | 228.326ms | 480.000ms | 89.37 MiB | none | 437.96K/s |
| q9 | group by and count distinct | 61.000ms | 162.562ms | 162.412ms | 0.0% | 162.412ms | 162.412ms | 162.412ms | 162.412ms | 190.000ms | 132.46 MiB | none | 615.71K/s |
| q10 | group by, several aggregates | 28.000ms | 114.510ms | 96.091ms | 0.0% | 96.091ms | 96.091ms | 96.091ms | 96.091ms | 90.000ms | 125.48 MiB | none | 1.04M/s |
| q11 | group by a string and count distinct | 27.000ms | 86.037ms | 88.506ms | 0.0% | 88.506ms | 88.506ms | 88.506ms | 88.506ms | 80.000ms | 109.04 MiB | none | 1.13M/s |
| q12 | group by two strings and count distinct | 21.000ms | 90.676ms | 122.435ms | 0.0% | 122.435ms | 122.435ms | 122.435ms | 122.435ms | 110.000ms | 107.20 MiB | none | 816.74K/s |
| q13 | group by a string and top k | 44.000ms | 170.502ms | 157.604ms | 0.0% | 157.604ms | 157.604ms | 157.604ms | 157.604ms | 100.000ms | 129.98 MiB | none | 634.49K/s |
| q14 | group by a string and count distinct | 77.000ms | 176.479ms | 171.904ms | 0.0% | 171.904ms | 171.904ms | 171.904ms | 171.904ms | 230.000ms | 138.04 MiB | none | 581.71K/s |
| q15 | group by two columns and top k | 56.000ms | 154.199ms | 114.691ms | 0.0% | 114.691ms | 114.691ms | 114.691ms | 114.691ms | 140.000ms | 136.79 MiB | none | 871.89K/s |
| q16 | group by, very high card | 34.000ms | 89.462ms | 119.732ms | 0.0% | 119.732ms | 119.732ms | 119.732ms | 119.732ms | 140.000ms | 130.18 MiB | none | 835.18K/s |
| q17 | group by two, very high card | 346.000ms | 710.603ms | 407.135ms | 0.0% | 407.135ms | 407.135ms | 407.135ms | 407.135ms | 910.000ms | 155.02 MiB | none | 245.61K/s |
| q18 | group by two, no ordering | 43.000ms | 129.980ms | 116.376ms | 0.0% | 116.376ms | 116.376ms | 116.376ms | 116.376ms | 120.000ms | 148.79 MiB | none | 859.27K/s |
| q19 | group by with an extract | 62.000ms | 219.331ms | 142.644ms | 0.0% | 142.644ms | 142.644ms | 142.644ms | 142.644ms | 160.000ms | 166.39 MiB | none | 701.03K/s |
| q20 | point lookup | 11.000ms | 81.986ms | 76.824ms | 0.0% | 76.824ms | 76.824ms | 76.824ms | 76.824ms | 70.000ms | 91.71 MiB | none | 1.30M/s |
| q21 | substring scan | 48.000ms | 117.096ms | 103.019ms | 0.0% | 103.019ms | 103.019ms | 103.019ms | 103.019ms | 90.000ms | 100.45 MiB | none | 970.68K/s |
| q22 | substring scan and group by | 55.000ms | 178.521ms | 114.074ms | 0.0% | 114.074ms | 114.074ms | 114.074ms | 114.074ms | 110.000ms | 117.26 MiB | none | 876.61K/s |
| q23 | two substring scans and group by | 97.000ms | 163.713ms | 174.006ms | 0.0% | 174.006ms | 174.006ms | 174.006ms | 174.006ms | 180.000ms | 138.07 MiB | none | 574.68K/s |
| q24 | select star and top k | 953.000ms | 296.678ms | 1.011s | 0.0% | 1.011s | 1.011s | 1.011s | 1.011s | 1.010s | 178.60 MiB | none | 98.86K/s |
| q25 | top k by a date | 47.000ms | 106.844ms | 199.173ms | 0.0% | 199.173ms | 199.173ms | 199.173ms | 199.173ms | 120.000ms | 107.73 MiB | none | 502.07K/s |
| q26 | top k by a string | 25.000ms | 84.412ms | 162.437ms | 0.0% | 162.437ms | 162.437ms | 162.437ms | 162.437ms | 130.000ms | 112.90 MiB | none | 615.61K/s |
| q27 | top k by two columns | 42.000ms | 150.631ms | 100.151ms | 0.0% | 100.151ms | 100.151ms | 100.151ms | 100.151ms | 120.000ms | 111.96 MiB | none | 998.47K/s |
| q28 | group by with a string length | 109.000ms | 191.561ms | 192.900ms | 0.0% | 192.900ms | 192.900ms | 192.900ms | 192.900ms | 220.000ms | 113.76 MiB | none | 518.39K/s |
| q29 | group by a regular expression | 142.000ms | 340.452ms | 242.196ms | 0.0% | 242.196ms | 242.196ms | 242.196ms | 242.196ms | 180.000ms | 150.78 MiB | none | 412.88K/s |
| q30 | ninety sums over one column | 71.000ms | 163.634ms | 189.648ms | 0.0% | 189.648ms | 189.648ms | 189.648ms | 189.648ms | 220.000ms | 92.31 MiB | none | 527.28K/s |
| q31 | group by two and several aggregates | 86.000ms | 148.670ms | 203.531ms | 0.0% | 203.531ms | 203.531ms | 203.531ms | 203.531ms | 200.000ms | 119.18 MiB | none | 491.32K/s |
| q32 | group by a high card pair | 67.000ms | 117.139ms | 178.434ms | 0.0% | 178.434ms | 178.434ms | 178.434ms | 178.434ms | 200.000ms | 118.79 MiB | none | 560.42K/s |
| q33 | group by a high card pair, unfiltered | 51.000ms | 144.584ms | 154.017ms | 0.0% | 154.017ms | 154.017ms | 154.017ms | 154.017ms | 160.000ms | 149.53 MiB | none | 649.27K/s |
| q34 | group by a long string | 122.000ms | 522.736ms | 183.123ms | 0.0% | 183.123ms | 183.123ms | 183.123ms | 183.123ms | 220.000ms | 177.02 MiB | none | 546.07K/s |
| q35 | group by a constant and a long string | 118.000ms | 201.455ms | 211.545ms | 0.0% | 211.545ms | 211.545ms | 211.545ms | 211.545ms | 270.000ms | 182.08 MiB | none | 472.70K/s |
| q36 | group by four expressions | 36.000ms | 197.970ms | 123.076ms | 0.0% | 123.076ms | 123.076ms | 123.076ms | 123.076ms | 90.000ms | 136.79 MiB | none | 812.49K/s |
| q37 | date range and group by a URL | 54.000ms | 186.052ms | 124.415ms | 0.0% | 124.415ms | 124.415ms | 124.415ms | 124.415ms | 160.000ms | 121.96 MiB | none | 803.75K/s |
| q38 | date range and group by a title | 88.000ms | 173.239ms | 159.594ms | 0.0% | 159.594ms | 159.594ms | 159.594ms | 159.594ms | 160.000ms | 119.36 MiB | none | 626.58K/s |
| q39 | date range, group by and offset | 49.000ms | 218.910ms | 128.031ms | 0.0% | 128.031ms | 128.031ms | 128.031ms | 128.031ms | 140.000ms | 117.21 MiB | none | 781.05K/s |
| q40 | date range, a case and a wide group by | 78.000ms | 216.606ms | 165.290ms | 0.0% | 165.290ms | 165.290ms | 165.290ms | 165.290ms | 160.000ms | 134.04 MiB | none | 604.99K/s |
| q41 | date range with an IN and a hash | 21.000ms | 96.240ms | 80.577ms | 0.0% | 80.577ms | 80.577ms | 80.577ms | 80.577ms | 70.000ms | 103.82 MiB | none | 1.24M/s |
| q42 | date range and a deep offset | 37.000ms | 185.282ms | 299.684ms | 0.0% | 299.684ms | 299.684ms | 299.684ms | 299.684ms | 310.000ms | 105.11 MiB | none | 333.68K/s |
| q43 | minute buckets over a date range | 46.000ms | 204.110ms | 156.682ms | 0.0% | 156.682ms | 156.682ms | 156.682ms | 156.682ms | 140.000ms | 104.38 MiB | none | 638.22K/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 3.621s by its own clock and 7.688s by ours, 8.235s cold, 8.420s of CPU, peak 182.08 MiB, 1.19M/s and 179.48 MiB/s.

Running it cost 112% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 15.88x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 50.765ms | 470.997ms | 491.696ms | 0.0% | 491.696ms | 491.696ms | 491.696ms | 491.696ms | 450.000ms | 57.21 MiB | none | 203.37K/s |
| q2 | filtered count | 43.312ms | 620.890ms | 483.067ms | 0.0% | 483.067ms | 483.067ms | 483.067ms | 483.067ms | 460.000ms | 61.57 MiB | none | 207.01K/s |
| q3 | three aggregates | 25.724ms | 691.512ms | 463.514ms | 0.0% | 463.514ms | 463.514ms | 463.514ms | 463.514ms | 460.000ms | 61.27 MiB | none | 215.74K/s |
| q4 | average | 138.264ms | 635.106ms | 545.082ms | 0.0% | 545.082ms | 545.082ms | 545.082ms | 545.082ms | 460.000ms | 60.24 MiB | none | 183.45K/s |
| q5 | count distinct, high card | 36.673ms | 968.665ms | 448.365ms | 0.0% | 448.365ms | 448.365ms | 448.365ms | 448.365ms | 430.000ms | 68.38 MiB | none | 223.03K/s |
| q6 | count distinct, strings | 156.034ms | 551.908ms | 604.912ms | 0.0% | 604.912ms | 604.912ms | 604.912ms | 604.912ms | 750.000ms | 70.40 MiB | none | 165.31K/s |
| q7 | min and max of a date | 68.250ms | 450.347ms | 523.715ms | 0.0% | 523.715ms | 523.715ms | 523.715ms | 523.715ms | 450.000ms | 59.65 MiB | none | 190.94K/s |
| q8 | group by, low card | 34.992ms | 466.011ms | 456.878ms | 0.0% | 456.878ms | 456.878ms | 456.878ms | 456.878ms | 460.000ms | 65.36 MiB | none | 218.87K/s |
| q9 | group by and count distinct | 65.348ms | 618.346ms | 428.510ms | 0.0% | 428.510ms | 428.510ms | 428.510ms | 428.510ms | 480.000ms | 83.02 MiB | none | 233.36K/s |
| q10 | group by, several aggregates | 134.127ms | 463.589ms | 565.961ms | 0.0% | 565.961ms | 565.961ms | 565.961ms | 565.961ms | 590.000ms | 84.72 MiB | none | 176.69K/s |
| q11 | group by a string and count distinct | 70.732ms | 499.007ms | 475.024ms | 0.0% | 475.024ms | 475.024ms | 475.024ms | 475.024ms | 500.000ms | 70.42 MiB | none | 210.51K/s |
| q12 | group by two strings and count distinct | 58.629ms | 525.252ms | 473.659ms | 0.0% | 473.659ms | 473.659ms | 473.659ms | 473.659ms | 520.000ms | 70.98 MiB | none | 211.12K/s |
| q13 | group by a string and top k | 102.377ms | 527.653ms | 543.314ms | 0.0% | 543.314ms | 543.314ms | 543.314ms | 543.314ms | 560.000ms | 70.98 MiB | none | 184.05K/s |
| q14 | group by a string and count distinct | 102.786ms | 598.274ms | 507.619ms | 0.0% | 507.619ms | 507.619ms | 507.619ms | 507.619ms | 550.000ms | 79.93 MiB | none | 196.99K/s |
| q15 | group by two columns and top k | 133.577ms | 506.249ms | 518.832ms | 0.0% | 518.832ms | 518.832ms | 518.832ms | 518.832ms | 680.000ms | 72.61 MiB | none | 192.74K/s |
| q16 | group by, very high card | 68.726ms | 490.985ms | 528.129ms | 0.0% | 528.129ms | 528.129ms | 528.129ms | 528.129ms | 540.000ms | 72.10 MiB | none | 189.34K/s |
| q17 | group by two, very high card | 118.950ms | 570.244ms | 642.280ms | 0.0% | 642.280ms | 642.280ms | 642.280ms | 642.280ms | 660.000ms | 84.17 MiB | none | 155.69K/s |
| q18 | group by two, no ordering | 77.185ms | 695.980ms | 562.124ms | 0.0% | 562.124ms | 562.124ms | 562.124ms | 562.124ms | 590.000ms | 84.11 MiB | none | 177.89K/s |
| q19 | group by with an extract | 186.472ms | 945.232ms | 836.176ms | 0.0% | 836.176ms | 836.176ms | 836.176ms | 836.176ms | 1.110s | 88.74 MiB | none | 119.59K/s |
| q20 | point lookup | 38.024ms | 467.177ms | 525.986ms | 0.0% | 525.986ms | 525.986ms | 525.986ms | 525.986ms | 480.000ms | 60.38 MiB | none | 190.12K/s |
| q21 | substring scan | 79.561ms | 521.027ms | 523.678ms | 0.0% | 523.678ms | 523.678ms | 523.678ms | 523.678ms | 510.000ms | 73.38 MiB | none | 190.95K/s |
| q22 | substring scan and group by | 228.290ms | 1.348s | 1.215s | 0.0% | 1.215s | 1.215s | 1.215s | 1.215s | 720.000ms | 78.55 MiB | none | 82.33K/s |
| q23 | two substring scans and group by | 404.858ms | 1.254s | 1.333s | 0.0% | 1.333s | 1.333s | 1.333s | 1.333s | 960.000ms | 105.63 MiB | none | 75.01K/s |
| q24 | select star and top k | 484.297ms | 1.568s | 2.106s | 0.0% | 2.106s | 2.106s | 2.106s | 2.106s | 1.170s | 116.39 MiB | none | 47.48K/s |
| q25 | top k by a date | 361.675ms | 1.390s | 1.419s | 0.0% | 1.419s | 1.419s | 1.419s | 1.419s | 1.080s | 66.30 MiB | none | 70.45K/s |
| q26 | top k by a string | 79.990ms | 1.133s | 976.585ms | 0.0% | 976.585ms | 976.585ms | 976.585ms | 976.585ms | 540.000ms | 64.32 MiB | none | 102.40K/s |
| q27 | top k by two columns | 116.938ms | 1.299s | 1.125s | 0.0% | 1.125s | 1.125s | 1.125s | 1.125s | 690.000ms | 66.82 MiB | none | 88.89K/s |
| q30 | ninety sums over one column | 105.340ms | 1.177s | 1.012s | 0.0% | 1.012s | 1.012s | 1.012s | 1.012s | 670.000ms | 63.09 MiB | none | 98.82K/s |
| q31 | group by two and several aggregates | 55.722ms | 1.010s | 1.338s | 0.0% | 1.338s | 1.338s | 1.338s | 1.338s | 690.000ms | 73.18 MiB | none | 74.75K/s |
| q32 | group by a high card pair | 170.071ms | 914.708ms | 1.283s | 0.0% | 1.283s | 1.283s | 1.283s | 1.283s | 630.000ms | 74.43 MiB | none | 77.93K/s |
| q33 | group by a high card pair, unfiltered | 236.754ms | 1.932s | 1.543s | 0.0% | 1.543s | 1.543s | 1.543s | 1.543s | 810.000ms | 89.53 MiB | none | 64.82K/s |
| q34 | group by a long string | 375.904ms | 1.670s | 1.874s | 0.0% | 1.874s | 1.874s | 1.874s | 1.874s | 900.000ms | 111.10 MiB | none | 53.37K/s |
| q35 | group by a constant and a long string | 733.854ms | 3.244s | 3.184s | 0.0% | 3.184s | 3.184s | 3.184s | 3.184s | 1.050s | 120.02 MiB | 20.00 KiB | 31.41K/s |
| q37 | date range and group by a URL | 296.787ms | 1.346s | 1.544s | 0.0% | 1.544s | 1.544s | 1.544s | 1.544s | 1.070s | 91.88 MiB | 196.00 KiB | 64.78K/s |
| q38 | date range and group by a title | 264.418ms | 3.754s | 1.761s | 0.0% | 1.761s | 1.761s | 1.761s | 1.761s | 1.230s | 91.93 MiB | 16.06 MiB | 56.78K/s |
| q39 | date range, group by and offset | 97.984ms | 813.220ms | 775.181ms | 0.0% | 775.181ms | 775.181ms | 775.181ms | 775.181ms | 760.000ms | 77.84 MiB | 2.58 MiB | 129.00K/s |
| q40 | date range, a case and a wide group by | 155.452ms | 955.451ms | 887.143ms | 0.0% | 887.143ms | 887.143ms | 887.143ms | 887.143ms | 820.000ms | 85.82 MiB | 2.35 MiB | 112.72K/s |
| q41 | date range with an IN and a hash | 56.659ms | 943.486ms | 790.366ms | 0.0% | 790.366ms | 790.366ms | 790.366ms | 790.366ms | 670.000ms | 71.62 MiB | 2.47 MiB | 126.52K/s |
| q42 | date range and a deep offset | 88.632ms | 1.007s | 1.230s | 0.0% | 1.230s | 1.230s | 1.230s | 1.230s | 760.000ms | 70.00 MiB | 248.00 KiB | 81.32K/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 6.104s by its own clock and 36.543s by ours, 39.042s cold, 26.910s of CPU, peak 120.02 MiB, 638.90K/s and 96.57 MiB/s.

Running it cost 499% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 7.43x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-server in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 6.000ms | 214.321ms | 262.139ms | 0.0% | 262.139ms | 262.139ms | 262.139ms | 262.139ms | not read | not read | not read | 381.47K/s |
| q2 | filtered count | 5.000ms | 217.275ms | 327.742ms | 0.0% | 327.742ms | 327.742ms | 327.742ms | 327.742ms | not read | not read | not read | 305.11K/s |
| q3 | three aggregates | 7.000ms | 398.890ms | 188.159ms | 0.0% | 188.159ms | 188.159ms | 188.159ms | 188.159ms | not read | not read | not read | 531.45K/s |
| q4 | average | 8.000ms | 281.033ms | 200.639ms | 0.0% | 200.639ms | 200.639ms | 200.639ms | 200.639ms | not read | not read | not read | 498.40K/s |
| q5 | count distinct, high card | 25.000ms | 192.433ms | 205.308ms | 0.0% | 205.308ms | 205.308ms | 205.308ms | 205.308ms | not read | not read | not read | 487.06K/s |
| q6 | count distinct, strings | 27.000ms | 541.909ms | 284.443ms | 0.0% | 284.443ms | 284.443ms | 284.443ms | 284.443ms | not read | not read | not read | 351.56K/s |
| q7 | min and max of a date | 10.000ms | 233.473ms | 198.650ms | 0.0% | 198.650ms | 198.650ms | 198.650ms | 198.650ms | not read | not read | not read | 503.39K/s |
| q8 | group by, low card | 7.000ms | 229.300ms | 203.458ms | 0.0% | 203.458ms | 203.458ms | 203.458ms | 203.458ms | not read | not read | not read | 491.49K/s |
| q9 | group by and count distinct | 19.000ms | 230.203ms | 275.401ms | 0.0% | 275.401ms | 275.401ms | 275.401ms | 275.401ms | not read | not read | not read | 363.10K/s |
| q10 | group by, several aggregates | 73.000ms | 276.819ms | 281.638ms | 0.0% | 281.638ms | 281.638ms | 281.638ms | 281.638ms | not read | not read | not read | 355.06K/s |
| q11 | group by a string and count distinct | 8.000ms | 236.689ms | 196.137ms | 0.0% | 196.137ms | 196.137ms | 196.137ms | 196.137ms | not read | not read | not read | 509.84K/s |
| q12 | group by two strings and count distinct | 18.000ms | 220.249ms | 242.488ms | 0.0% | 242.488ms | 242.488ms | 242.488ms | 242.488ms | not read | not read | not read | 412.38K/s |
| q13 | group by a string and top k | 29.000ms | 220.821ms | 245.615ms | 0.0% | 245.615ms | 245.615ms | 245.615ms | 245.615ms | not read | not read | not read | 407.13K/s |
| q14 | group by a string and count distinct | 42.000ms | 394.901ms | 343.960ms | 0.0% | 343.960ms | 343.960ms | 343.960ms | 343.960ms | not read | not read | not read | 290.73K/s |
| q15 | group by two columns and top k | 29.000ms | 310.191ms | 326.027ms | 0.0% | 326.027ms | 326.027ms | 326.027ms | 326.027ms | not read | not read | not read | 306.72K/s |
| q16 | group by, very high card | 23.000ms | 221.694ms | 255.271ms | 0.0% | 255.271ms | 255.271ms | 255.271ms | 255.271ms | not read | not read | not read | 391.73K/s |
| q17 | group by two, very high card | 66.000ms | 274.852ms | 388.444ms | 0.0% | 388.444ms | 388.444ms | 388.444ms | 388.444ms | not read | not read | not read | 257.43K/s |
| q18 | group by two, no ordering | 13.000ms | 265.569ms | 255.391ms | 0.0% | 255.391ms | 255.391ms | 255.391ms | 255.391ms | not read | not read | not read | 391.55K/s |
| q19 | group by with an extract | 83.000ms | 353.252ms | 360.611ms | 0.0% | 360.611ms | 360.611ms | 360.611ms | 360.611ms | not read | not read | not read | 277.30K/s |
| q20 | point lookup | 5.000ms | 222.718ms | 301.106ms | 0.0% | 301.106ms | 301.106ms | 301.106ms | 301.106ms | not read | not read | not read | 332.10K/s |
| q21 | substring scan | 29.000ms | 334.783ms | 279.929ms | 0.0% | 279.929ms | 279.929ms | 279.929ms | 279.929ms | not read | not read | not read | 357.23K/s |
| q22 | substring scan and group by | 16.000ms | 273.002ms | 291.649ms | 0.0% | 291.649ms | 291.649ms | 291.649ms | 291.649ms | not read | not read | not read | 342.87K/s |
| q23 | two substring scans and group by | 33.000ms | 232.158ms | 398.398ms | 0.0% | 398.398ms | 398.398ms | 398.398ms | 398.398ms | not read | not read | not read | 251.00K/s |
| q24 | select star and top k | 264.000ms | 484.544ms | 462.577ms | 0.0% | 462.577ms | 462.577ms | 462.577ms | 462.577ms | not read | not read | not read | 216.18K/s |
| q25 | top k by a date | 8.000ms | 326.215ms | 309.001ms | 0.0% | 309.001ms | 309.001ms | 309.001ms | 309.001ms | not read | not read | not read | 323.62K/s |
| q26 | top k by a string | 7.000ms | 435.790ms | 377.441ms | 0.0% | 377.441ms | 377.441ms | 377.441ms | 377.441ms | not read | not read | not read | 264.94K/s |
| q27 | top k by two columns | 14.000ms | 396.989ms | 255.685ms | 0.0% | 255.685ms | 255.685ms | 255.685ms | 255.685ms | not read | not read | not read | 391.10K/s |
| q28 | group by with a string length | 10.000ms | 234.344ms | 286.206ms | 0.0% | 286.206ms | 286.206ms | 286.206ms | 286.206ms | not read | not read | not read | 349.39K/s |
| q29 | group by a regular expression | 204.000ms | 640.333ms | 489.817ms | 0.0% | 489.817ms | 489.817ms | 489.817ms | 489.817ms | not read | not read | not read | 204.15K/s |
| q30 | ninety sums over one column | 49.000ms | 350.897ms | 318.611ms | 0.0% | 318.611ms | 318.611ms | 318.611ms | 318.611ms | not read | not read | not read | 313.86K/s |
| q31 | group by two and several aggregates | 66.000ms | 256.385ms | 405.583ms | 0.0% | 405.583ms | 405.583ms | 405.583ms | 405.583ms | not read | not read | not read | 246.55K/s |
| q32 | group by a high card pair | 42.000ms | 245.498ms | 245.733ms | 0.0% | 245.733ms | 245.733ms | 245.733ms | 245.733ms | not read | not read | not read | 406.94K/s |
| q33 | group by a high card pair, unfiltered | 49.000ms | 529.860ms | 303.516ms | 0.0% | 303.516ms | 303.516ms | 303.516ms | 303.516ms | not read | not read | not read | 329.47K/s |
| q34 | group by a long string | 62.000ms | 361.013ms | 445.757ms | 0.0% | 445.757ms | 445.757ms | 445.757ms | 445.757ms | not read | not read | not read | 224.33K/s |
| q35 | group by a constant and a long string | 69.000ms | 398.076ms | 326.313ms | 0.0% | 326.313ms | 326.313ms | 326.313ms | 326.313ms | not read | not read | not read | 306.45K/s |
| q36 | group by four expressions | 17.000ms | 324.875ms | 249.064ms | 0.0% | 249.064ms | 249.064ms | 249.064ms | 249.064ms | not read | not read | not read | 401.50K/s |
| q37 | date range and group by a URL | 16.000ms | 347.877ms | 347.364ms | 0.0% | 347.364ms | 347.364ms | 347.364ms | 347.364ms | not read | not read | not read | 287.88K/s |
| q38 | date range and group by a title | 29.000ms | 241.817ms | 238.490ms | 0.0% | 238.490ms | 238.490ms | 238.490ms | 238.490ms | not read | not read | not read | 419.30K/s |
| q39 | date range, group by and offset | 12.000ms | 242.435ms | 302.205ms | 0.0% | 302.205ms | 302.205ms | 302.205ms | 302.205ms | not read | not read | not read | 330.89K/s |
| q40 | date range, a case and a wide group by | 26.000ms | 332.108ms | 309.156ms | 0.0% | 309.156ms | 309.156ms | 309.156ms | 309.156ms | not read | not read | not read | 323.45K/s |
| q41 | date range with an IN and a hash | 20.000ms | 320.416ms | 295.545ms | 0.0% | 295.545ms | 295.545ms | 295.545ms | 295.545ms | not read | not read | not read | 338.35K/s |
| q42 | date range and a deep offset | 14.000ms | 272.965ms | 316.034ms | 0.0% | 316.034ms | 316.034ms | 316.034ms | 316.034ms | not read | not read | not read | 316.42K/s |
| q43 | minute buckets over a date range | 13.000ms | 312.482ms | 397.119ms | 0.0% | 397.119ms | 397.119ms | 397.119ms | 397.119ms | not read | not read | not read | 251.81K/s |

clickhouse-server 26.9.1.1138 over 43 of 43 queries. Total 1.572s by its own clock and 12.994s by ours, 13.431s cold, no reading of CPU, peak not read, 2.74M/s and 413.43 MiB/s.

Running it cost 727% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.60x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 9.228ms | 6.158ms | 0.0% | 6.158ms | 6.158ms | 6.158ms | 6.158ms | 0.000us | 4.00 MiB | none | 16.24M/s |
| q2 | filtered count | 3.000ms | 13.237ms | 53.934ms | 0.0% | 53.934ms | 53.934ms | 53.934ms | 53.934ms | 0.000us | 4.88 MiB | none | 1.85M/s |
| q3 | three aggregates | 8.000ms | 15.657ms | 39.250ms | 0.0% | 39.250ms | 39.250ms | 39.250ms | 39.250ms | 0.000us | 5.60 MiB | none | 2.55M/s |
| q4 | average | 7.000ms | 48.997ms | 25.548ms | 0.0% | 25.548ms | 25.548ms | 25.548ms | 25.548ms | 0.000us | 7.50 MiB | none | 3.91M/s |
| q5 | count distinct, high card | 89.000ms | 121.945ms | 104.326ms | 0.0% | 104.326ms | 104.326ms | 104.326ms | 104.326ms | 80.000ms | 14.68 MiB | none | 958.51K/s |
| q6 | count distinct, strings | 55.000ms | 133.877ms | 109.345ms | 0.0% | 109.345ms | 109.345ms | 109.345ms | 109.345ms | 60.000ms | 7.09 MiB | none | 914.52K/s |
| q7 | min and max of a date | 10.000ms | 22.731ms | 32.746ms | 0.0% | 32.746ms | 32.746ms | 32.746ms | 32.746ms | 0.000us | 4.88 MiB | none | 3.05M/s |
| q8 | group by, low card | 6.000ms | 39.774ms | 14.336ms | 0.0% | 14.336ms | 14.336ms | 14.336ms | 14.336ms | 0.000us | 4.88 MiB | none | 6.98M/s |
| q9 | group by and count distinct | 181.000ms | 214.903ms | 207.949ms | 0.0% | 207.949ms | 207.949ms | 207.949ms | 207.949ms | 170.000ms | 16.01 MiB | none | 480.88K/s |
| q10 | group by, several aggregates | 204.000ms | 168.381ms | 224.092ms | 0.0% | 224.092ms | 224.092ms | 224.092ms | 224.092ms | 210.000ms | 16.67 MiB | none | 446.24K/s |
| q11 | group by a string and count distinct | 14.000ms | 32.772ms | 29.528ms | 0.0% | 29.528ms | 29.528ms | 29.528ms | 29.528ms | 20.000ms | 7.50 MiB | none | 3.39M/s |
| q12 | group by two strings and count distinct | 15.000ms | 68.509ms | 33.700ms | 0.0% | 33.700ms | 33.700ms | 33.700ms | 33.700ms | 10.000ms | 7.50 MiB | none | 2.97M/s |
| q13 | group by a string and top k | 43.000ms | 63.388ms | 69.162ms | 0.0% | 69.162ms | 69.162ms | 69.162ms | 69.162ms | 40.000ms | 9.53 MiB | none | 1.45M/s |
| q14 | group by a string and count distinct | 51.000ms | 98.439ms | 59.203ms | 0.0% | 59.203ms | 59.203ms | 59.203ms | 59.203ms | 50.000ms | 13.07 MiB | none | 1.69M/s |
| q15 | group by two columns and top k | 44.000ms | 60.836ms | 51.550ms | 0.0% | 51.550ms | 51.550ms | 51.550ms | 51.550ms | 30.000ms | 11.31 MiB | none | 1.94M/s |
| q16 | group by, very high card | 119.000ms | 112.824ms | 130.332ms | 0.0% | 130.332ms | 130.332ms | 130.332ms | 130.332ms | 120.000ms | 24.14 MiB | none | 767.26K/s |
| q17 | group by two, very high card | 123.000ms | 164.742ms | 141.857ms | 0.0% | 141.857ms | 141.857ms | 141.857ms | 141.857ms | 120.000ms | 33.70 MiB | none | 704.92K/s |
| q18 | group by two, no ordering | 126.000ms | 105.818ms | 136.976ms | 0.0% | 136.976ms | 136.976ms | 136.976ms | 136.976ms | 120.000ms | 33.57 MiB | none | 730.04K/s |
| q20 | point lookup | 16.000ms | 24.070ms | 37.491ms | 0.0% | 37.491ms | 37.491ms | 37.491ms | 37.491ms | 10.000ms | 7.50 MiB | none | 2.67M/s |
| q21 | substring scan | 162.000ms | 193.755ms | 206.805ms | 0.0% | 206.805ms | 206.805ms | 206.805ms | 206.805ms | 160.000ms | 24.70 MiB | none | 483.54K/s |
| q22 | substring scan and group by | 226.000ms | 197.599ms | 245.811ms | 0.0% | 245.811ms | 245.811ms | 245.811ms | 245.811ms | 220.000ms | 26.70 MiB | none | 406.81K/s |
| q23 | two substring scans and group by | 419.000ms | 311.639ms | 432.489ms | 0.0% | 432.489ms | 432.489ms | 432.489ms | 432.489ms | 410.000ms | 49.63 MiB | none | 231.22K/s |
| q24 | select star and top k | 676.000ms | 1.133s | 704.632ms | 0.0% | 704.632ms | 704.632ms | 704.632ms | 704.632ms | 690.000ms | 158.39 MiB | none | 141.92K/s |
| q25 | top k by a date | 39.000ms | 46.998ms | 50.152ms | 0.0% | 50.152ms | 50.152ms | 50.152ms | 50.152ms | 30.000ms | 8.19 MiB | none | 1.99M/s |
| q26 | top k by a string | 33.000ms | 42.862ms | 40.051ms | 0.0% | 40.051ms | 40.051ms | 40.051ms | 40.051ms | 30.000ms | 7.10 MiB | none | 2.50M/s |
| q27 | top k by two columns | 49.000ms | 55.374ms | 56.167ms | 0.0% | 56.167ms | 56.167ms | 56.167ms | 56.167ms | 40.000ms | 8.19 MiB | none | 1.78M/s |
| q28 | group by with a string length | 121.000ms | 150.469ms | 140.144ms | 0.0% | 140.144ms | 140.144ms | 140.144ms | 140.144ms | 120.000ms | 25.87 MiB | none | 713.54K/s |
| q29 | group by a regular expression | 284.000ms | 387.123ms | 300.182ms | 0.0% | 300.182ms | 300.182ms | 300.182ms | 300.182ms | 290.000ms | 20.26 MiB | none | 333.12K/s |
| q30 | ninety sums over one column | 55.000ms | 59.742ms | 63.945ms | 0.0% | 63.945ms | 63.945ms | 63.945ms | 63.945ms | 50.000ms | 5.28 MiB | none | 1.56M/s |
| q31 | group by two and several aggregates | 64.000ms | 58.402ms | 78.522ms | 0.0% | 78.522ms | 78.522ms | 78.522ms | 78.522ms | 60.000ms | 15.05 MiB | none | 1.27M/s |
| q32 | group by a high card pair | 57.000ms | 81.620ms | 67.430ms | 0.0% | 67.430ms | 67.430ms | 67.430ms | 67.430ms | 60.000ms | 15.73 MiB | none | 1.48M/s |
| q34 | group by a long string | 211.000ms | 223.742ms | 226.685ms | 0.0% | 226.685ms | 226.685ms | 226.685ms | 226.685ms | 220.000ms | 38.90 MiB | none | 441.13K/s |
| q35 | group by a constant and a long string | 245.000ms | 331.834ms | 262.973ms | 0.0% | 262.973ms | 262.973ms | 262.973ms | 262.973ms | 250.000ms | 49.52 MiB | none | 380.26K/s |
| q36 | group by four expressions | 163.000ms | 164.261ms | 173.834ms | 0.0% | 173.834ms | 173.834ms | 173.834ms | 173.834ms | 160.000ms | 40.16 MiB | none | 575.25K/s |
| q37 | date range and group by a URL | 153.000ms | 137.804ms | 162.877ms | 0.0% | 162.877ms | 162.877ms | 162.877ms | 162.877ms | 150.000ms | 28.14 MiB | none | 613.95K/s |
| q38 | date range and group by a title | 196.000ms | 132.569ms | 226.944ms | 0.0% | 226.944ms | 226.944ms | 226.944ms | 226.944ms | 190.000ms | 29.01 MiB | none | 440.63K/s |
| q39 | date range, group by and offset | 173.000ms | 340.177ms | 239.741ms | 0.0% | 239.741ms | 239.741ms | 239.741ms | 239.741ms | 170.000ms | 29.01 MiB | none | 417.11K/s |
| q40 | date range, a case and a wide group by | 288.000ms | 241.673ms | 312.739ms | 0.0% | 312.739ms | 312.739ms | 312.739ms | 312.739ms | 290.000ms | 45.77 MiB | none | 319.75K/s |
| q41 | date range with an IN and a hash | 30.000ms | 72.882ms | 47.706ms | 0.0% | 47.706ms | 47.706ms | 47.706ms | 47.706ms | 40.000ms | 10.75 MiB | none | 2.10M/s |
| q42 | date range and a deep offset | 31.000ms | 42.678ms | 49.280ms | 0.0% | 49.280ms | 49.280ms | 49.280ms | 49.280ms | 30.000ms | 10.54 MiB | none | 2.03M/s |
| q43 | minute buckets over a date range | 22.000ms | 28.503ms | 29.266ms | 0.0% | 29.266ms | 29.266ms | 29.266ms | 29.266ms | 20.000ms | 9.07 MiB | none | 3.42M/s |

rudb rudb 0.2.28 over 41 of 43 queries. Total 4.812s by its own clock and 5.626s by ours, 5.955s cold, 4.720s of CPU, peak 158.39 MiB, 852.02K/s and 128.78 MiB/s.

Running it cost 17% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 114.43x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

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

- q4: AVG(UserID) over a hundred million bigints near 10^18, where the engines disagree for two reasons. The order the partial sums are added in moves the floating point ones further apart than the one part in a billion this harness calls the same number, and DuckDB is plainly wrong: it sums the bigint column short by a multiple of 2^64 on this file, where its own hugeint and decimal paths, rudb, and adding the column up outside a database all agree. tamnd/rudb-compat#12.
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

Hot here means page cache warm and not buffer pool warm for every row but clickhouse-server, which stayed up across the whole suite with its own caches warm. That is the stronger kind of hot, so a ratio against that column is a ratio between two different quantities.

