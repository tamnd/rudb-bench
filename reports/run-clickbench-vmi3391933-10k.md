# clickbench on vmi3391933

This is one run of the clickbench suite on vmi3391933, over 7 engines and 43 queries, with 1 hot run of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

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

| engine | version | state | load | load cpu | on disk | that size is | format |
| --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 631.865ms | 630.000ms | 4.26 MiB | its own database file | its own |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 625.575ms | 700.000ms | 3.76 MiB | its own database file | its own |
| clickhouse-local | 26.9.1.1138 | ran | 963.783ms | 1.650s | 2.68 MiB | its own MergeTree parts, as system.parts counts the active ones | its own |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 1.70 MiB | the source Parquet, this engine has no storage format of its own | the Parquet |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 1.70 MiB | the source Parquet, this engine has no storage format of its own | the Parquet |
| clickhouse-server | 26.9.1.1138 | ran | 2.881s | not read | 2.61 MiB | its own MergeTree parts, sorted by hits by (CounterID, EventDate, UserID, EventTime, WatchID), as system.parts counts the active ones | its own |
| rudb | rudb 0.2.28 | ran | 0.000us | 0.000us | 1.70 MiB | the source Parquet, this engine has no storage format of its own yet | the Parquet |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 561.000ms | 3.271s | +483% | 3.578s | 3.550s | 1.09 | 35.75 MiB | none | 766.49K/s | 130.03 MiB/s | 1.00x |
| duckdb-pinned | 921.000ms | 7.643s | +730% | 7.308s | 10.140s | 1.33 | 48.78 MiB | none | 466.88K/s | 79.20 MiB/s | 1.64x |
| clickhouse-local | 2.574s | 25.374s | +886% | 22.795s | 39.520s | 1.56 | 250.50 MiB | none | 167.06K/s | 28.34 MiB/s | 4.59x |
| datafusion | 3.280s | 9.743s | +197% | 9.353s | 5.830s | 0.60 | 119.00 MiB | none | 131.10K/s | 22.24 MiB/s | 5.85x |
| polars | 3.362s | 32.618s | +870% | 31.780s | 24.760s | 0.76 | 74.67 MiB | none | 115.99K/s | 19.68 MiB/s | 5.99x |
| clickhouse-server | 896.000ms | 13.273s | +1381% | 13.770s | not read | not read | not read | not read | 479.91K/s | 81.41 MiB/s | 1.60x |
| rudb | 957.000ms | 2.385s | +149% | 2.228s | 930.000ms | 0.39 | 19.45 MiB | none | 428.42K/s | 72.68 MiB/s | 1.71x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | clickhouse-server | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 2.000ms | 27.000ms | 4.000ms | 102.228ms | 3.000ms | 3.000ms |
| q2 | filtered count | 3.000ms | 6.000ms | 11.000ms | 58.000ms | 34.811ms | 8.000ms | 65.000ms |
| q3 | three aggregates | 3.000ms | 6.000ms | 18.000ms | 26.000ms | 99.883ms | 9.000ms | 2.000ms |
| q4 | average | 3.000ms | 5.000ms | 11.000ms | 86.000ms | 67.105ms | 16.000ms | 2.000ms |
| q5 | count distinct, high card | 17.000ms | 5.000ms | 20.000ms | 12.000ms | 68.045ms | 10.000ms | 13.000ms |
| q6 | count distinct, strings | 7.000ms | 6.000ms | 12.000ms | 46.000ms | 42.418ms | 21.000ms | 7.000ms |
| q7 | min and max of a date | 2.000ms | 3.000ms | 139.000ms | 4.000ms | 18.130ms | 7.000ms | 4.000ms |
| q8 | group by, low card | 4.000ms | 26.000ms | 23.000ms | 34.000ms | 63.499ms | 10.000ms | 6.000ms |
| q9 | group by and count distinct | 8.000ms | 13.000ms | 21.000ms | 259.000ms | 91.100ms | 8.000ms | 12.000ms |
| q10 | group by, several aggregates | 14.000ms | 86.000ms | 46.000ms | 31.000ms | 39.443ms | 69.000ms | 27.000ms |
| q11 | group by a string and count distinct | 21.000ms | 12.000ms | 59.000ms | 35.000ms | 71.277ms | 8.000ms | 2.000ms |
| q12 | group by two strings and count distinct | 15.000ms | 36.000ms | 172.000ms | 72.000ms | 50.712ms | 10.000ms | 3.000ms |
| q13 | group by a string and top k | 23.000ms | 27.000ms | 30.000ms | 25.000ms | 28.057ms | 6.000ms | 11.000ms |
| q14 | group by a string and count distinct | 22.000ms | 16.000ms | 29.000ms | 28.000ms | 90.036ms | 6.000ms | 17.000ms |
| q15 | group by two columns and top k | 21.000ms | 26.000ms | 51.000ms | 15.000ms | 45.879ms | 52.000ms | 8.000ms |
| q16 | group by, very high card | 12.000ms | 8.000ms | 17.000ms | 13.000ms | 60.479ms | 5.000ms | 17.000ms |
| q17 | group by two, very high card | 22.000ms | 8.000ms | 57.000ms | 116.000ms | 105.182ms | 40.000ms | 18.000ms |
| q18 | group by two, no ordering | 16.000ms | 59.000ms | 147.000ms | 14.000ms | 63.986ms | 29.000ms | 49.000ms |
| q19 | group by with an extract | 12.000ms | 11.000ms | 33.000ms | 104.000ms | 56.641ms | 22.000ms | no dialect |
| q20 | point lookup | 2.000ms | 10.000ms | 29.000ms | 44.000ms | 78.473ms | 4.000ms | 1.000ms |
| q21 | substring scan | 4.000ms | 6.000ms | 32.000ms | 100.000ms | 68.994ms | 5.000ms | 20.000ms |
| q22 | substring scan and group by | 5.000ms | 12.000ms | 103.000ms | 49.000ms | 50.914ms | 42.000ms | 26.000ms |
| q23 | two substring scans and group by | 12.000ms | 27.000ms | 74.000ms | 36.000ms | 85.085ms | 13.000ms | 67.000ms |
| q24 | select star and top k | 27.000ms | 44.000ms | 87.000ms | 129.000ms | 60.365ms | 14.000ms | 180.000ms |
| q25 | top k by a date | 19.000ms | 5.000ms | 19.000ms | 12.000ms | 60.308ms | 10.000ms | 6.000ms |
| q26 | top k by a string | 4.000ms | 9.000ms | 33.000ms | 63.000ms | 128.736ms | 6.000ms | 5.000ms |
| q27 | top k by two columns | 4.000ms | 11.000ms | 22.000ms | 51.000ms | 214.576ms | 7.000ms | 34.000ms |
| q28 | group by with a string length | 7.000ms | 11.000ms | 130.000ms | 67.000ms | no dialect | 8.000ms | 50.000ms |
| q29 | group by a regular expression | 33.000ms | 26.000ms | 54.000ms | 64.000ms | no dialect | 87.000ms | 38.000ms |
| q30 | ninety sums over one column | 18.000ms | 55.000ms | 56.000ms | 68.000ms | 77.930ms | 43.000ms | 61.000ms |
| q31 | group by two and several aggregates | 24.000ms | 39.000ms | 44.000ms | 337.000ms | 254.146ms | 38.000ms | 16.000ms |
| q32 | group by a high card pair | 19.000ms | 11.000ms | 36.000ms | 89.000ms | 241.958ms | 114.000ms | 10.000ms |
| q33 | group by a high card pair, unfiltered | 12.000ms | 56.000ms | 47.000ms | 195.000ms | 202.106ms | 15.000ms | no dialect |
| q34 | group by a long string | 15.000ms | 89.000ms | 65.000ms | 97.000ms | 186.885ms | 12.000ms | 20.000ms |
| q35 | group by a constant and a long string | 33.000ms | 13.000ms | 135.000ms | 92.000ms | 82.588ms | 15.000ms | 28.000ms |
| q36 | group by four expressions | 13.000ms | 8.000ms | 27.000ms | 229.000ms | no dialect | 12.000ms | 14.000ms |
| q37 | date range and group by a URL | 11.000ms | 11.000ms | 54.000ms | 35.000ms | 115.356ms | 11.000ms | 13.000ms |
| q38 | date range and group by a title | 22.000ms | 11.000ms | 101.000ms | 62.000ms | 56.970ms | 17.000ms | 22.000ms |
| q39 | date range, group by and offset | 14.000ms | 17.000ms | 44.000ms | 62.000ms | 49.228ms | 19.000ms | 33.000ms |
| q40 | date range, a case and a wide group by | 14.000ms | 26.000ms | 279.000ms | 105.000ms | 50.247ms | 21.000ms | 32.000ms |
| q41 | date range with an IN and a hash | 9.000ms | 23.000ms | 29.000ms | 178.000ms | 37.886ms | 14.000ms | 8.000ms |
| q42 | date range and a deep offset | 6.000ms | 30.000ms | 40.000ms | 104.000ms | 60.581ms | 10.000ms | 4.000ms |
| q43 | minute buckets over a date range | 8.000ms | 10.000ms | 111.000ms | 30.000ms | no dialect | 20.000ms | 3.000ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 92.168ms | 75.113ms | 0.0% | 75.113ms | 75.113ms | 75.113ms | 75.113ms | 60.000ms | 25.62 MiB | none | 133.13K/s |
| q2 | filtered count | 3.000ms | 80.542ms | 52.643ms | 0.0% | 52.643ms | 52.643ms | 52.643ms | 52.643ms | 40.000ms | 26.75 MiB | none | 189.96K/s |
| q3 | three aggregates | 3.000ms | 45.062ms | 52.108ms | 0.0% | 52.108ms | 52.108ms | 52.108ms | 52.108ms | 60.000ms | 27.12 MiB | none | 191.91K/s |
| q4 | average | 3.000ms | 52.917ms | 48.560ms | 0.0% | 48.560ms | 48.560ms | 48.560ms | 48.560ms | 50.000ms | 26.50 MiB | none | 205.93K/s |
| q5 | count distinct, high card | 17.000ms | 47.788ms | 80.098ms | 0.0% | 80.098ms | 80.098ms | 80.098ms | 80.098ms | 130.000ms | 29.12 MiB | none | 124.85K/s |
| q6 | count distinct, strings | 7.000ms | 65.640ms | 55.008ms | 0.0% | 55.008ms | 55.008ms | 55.008ms | 55.008ms | 50.000ms | 28.75 MiB | none | 181.79K/s |
| q7 | min and max of a date | 2.000ms | 44.590ms | 51.994ms | 0.0% | 51.994ms | 51.994ms | 51.994ms | 51.994ms | 60.000ms | 25.88 MiB | none | 192.33K/s |
| q8 | group by, low card | 4.000ms | 57.833ms | 54.812ms | 0.0% | 54.812ms | 54.812ms | 54.812ms | 54.812ms | 50.000ms | 28.50 MiB | none | 182.44K/s |
| q9 | group by and count distinct | 8.000ms | 82.262ms | 55.553ms | 0.0% | 55.553ms | 55.553ms | 55.553ms | 55.553ms | 50.000ms | 32.25 MiB | none | 180.01K/s |
| q10 | group by, several aggregates | 14.000ms | 147.925ms | 53.689ms | 0.0% | 53.689ms | 53.689ms | 53.689ms | 53.689ms | 60.000ms | 33.75 MiB | none | 186.26K/s |
| q11 | group by a string and count distinct | 21.000ms | 80.769ms | 67.609ms | 0.0% | 67.609ms | 67.609ms | 67.609ms | 67.609ms | 110.000ms | 31.00 MiB | none | 147.91K/s |
| q12 | group by two strings and count distinct | 15.000ms | 103.981ms | 57.410ms | 0.0% | 57.410ms | 57.410ms | 57.410ms | 57.410ms | 70.000ms | 32.50 MiB | none | 174.19K/s |
| q13 | group by a string and top k | 23.000ms | 46.451ms | 69.374ms | 0.0% | 69.374ms | 69.374ms | 69.374ms | 69.374ms | 90.000ms | 29.50 MiB | none | 144.15K/s |
| q14 | group by a string and count distinct | 22.000ms | 151.281ms | 188.307ms | 0.0% | 188.307ms | 188.307ms | 188.307ms | 188.307ms | 210.000ms | 32.62 MiB | none | 53.10K/s |
| q15 | group by two columns and top k | 21.000ms | 71.809ms | 81.016ms | 0.0% | 81.016ms | 81.016ms | 81.016ms | 81.016ms | 80.000ms | 29.88 MiB | none | 123.43K/s |
| q16 | group by, very high card | 12.000ms | 82.923ms | 60.526ms | 0.0% | 60.526ms | 60.526ms | 60.526ms | 60.526ms | 50.000ms | 30.88 MiB | none | 165.22K/s |
| q17 | group by two, very high card | 22.000ms | 60.416ms | 71.730ms | 0.0% | 71.730ms | 71.730ms | 71.730ms | 71.730ms | 80.000ms | 33.38 MiB | none | 139.41K/s |
| q18 | group by two, no ordering | 16.000ms | 62.713ms | 68.998ms | 0.0% | 68.998ms | 68.998ms | 68.998ms | 68.998ms | 90.000ms | 33.38 MiB | none | 144.93K/s |
| q19 | group by with an extract | 12.000ms | 64.969ms | 56.896ms | 0.0% | 56.896ms | 56.896ms | 56.896ms | 56.896ms | 60.000ms | 33.88 MiB | none | 175.76K/s |
| q20 | point lookup | 2.000ms | 57.397ms | 65.335ms | 0.0% | 65.335ms | 65.335ms | 65.335ms | 65.335ms | 60.000ms | 26.00 MiB | none | 153.06K/s |
| q21 | substring scan | 4.000ms | 73.585ms | 84.380ms | 0.0% | 84.380ms | 84.380ms | 84.380ms | 84.380ms | 90.000ms | 27.50 MiB | none | 118.51K/s |
| q22 | substring scan and group by | 5.000ms | 56.167ms | 46.698ms | 0.0% | 46.698ms | 46.698ms | 46.698ms | 46.698ms | 40.000ms | 28.50 MiB | none | 214.14K/s |
| q23 | two substring scans and group by | 12.000ms | 80.904ms | 64.562ms | 0.0% | 64.562ms | 64.562ms | 64.562ms | 64.562ms | 60.000ms | 31.38 MiB | none | 154.89K/s |
| q24 | select star and top k | 27.000ms | 86.266ms | 84.116ms | 0.0% | 84.116ms | 84.116ms | 84.116ms | 84.116ms | 80.000ms | 35.75 MiB | none | 118.88K/s |
| q25 | top k by a date | 19.000ms | 135.704ms | 92.060ms | 0.0% | 92.060ms | 92.060ms | 92.060ms | 92.060ms | 100.000ms | 29.38 MiB | 112.00 KiB | 108.62K/s |
| q26 | top k by a string | 4.000ms | 85.412ms | 142.716ms | 0.0% | 142.716ms | 142.716ms | 142.716ms | 142.716ms | 190.000ms | 26.62 MiB | none | 70.07K/s |
| q27 | top k by two columns | 4.000ms | 227.753ms | 81.048ms | 0.0% | 81.048ms | 81.048ms | 81.048ms | 81.048ms | 80.000ms | 27.50 MiB | 68.00 KiB | 123.38K/s |
| q28 | group by with a string length | 7.000ms | 71.057ms | 60.294ms | 0.0% | 60.294ms | 60.294ms | 60.294ms | 60.294ms | 50.000ms | 29.75 MiB | none | 165.85K/s |
| q29 | group by a regular expression | 33.000ms | 142.532ms | 107.027ms | 0.0% | 107.027ms | 107.027ms | 107.027ms | 107.027ms | 140.000ms | 30.88 MiB | none | 93.43K/s |
| q30 | ninety sums over one column | 18.000ms | 75.822ms | 66.335ms | 0.0% | 66.335ms | 66.335ms | 66.335ms | 66.335ms | 60.000ms | 30.25 MiB | none | 150.75K/s |
| q31 | group by two and several aggregates | 24.000ms | 63.302ms | 86.642ms | 0.0% | 86.642ms | 86.642ms | 86.642ms | 86.642ms | 80.000ms | 31.75 MiB | none | 115.42K/s |
| q32 | group by a high card pair | 19.000ms | 80.199ms | 90.701ms | 0.0% | 90.701ms | 90.701ms | 90.701ms | 90.701ms | 110.000ms | 31.75 MiB | none | 110.25K/s |
| q33 | group by a high card pair, unfiltered | 12.000ms | 66.526ms | 56.991ms | 0.0% | 56.991ms | 56.991ms | 56.991ms | 56.991ms | 70.000ms | 33.12 MiB | none | 175.47K/s |
| q34 | group by a long string | 15.000ms | 72.680ms | 63.576ms | 0.0% | 63.576ms | 63.576ms | 63.576ms | 63.576ms | 60.000ms | 32.38 MiB | none | 157.29K/s |
| q35 | group by a constant and a long string | 33.000ms | 135.836ms | 109.401ms | 0.0% | 109.401ms | 109.401ms | 109.401ms | 109.401ms | 170.000ms | 33.50 MiB | none | 91.41K/s |
| q36 | group by four expressions | 13.000ms | 102.272ms | 71.258ms | 0.0% | 71.258ms | 71.258ms | 71.258ms | 71.258ms | 70.000ms | 31.88 MiB | none | 140.34K/s |
| q37 | date range and group by a URL | 11.000ms | 56.102ms | 140.607ms | 0.0% | 140.607ms | 140.607ms | 140.607ms | 140.607ms | 170.000ms | 30.75 MiB | none | 71.12K/s |
| q38 | date range and group by a title | 22.000ms | 68.403ms | 94.157ms | 0.0% | 94.157ms | 94.157ms | 94.157ms | 94.157ms | 110.000ms | 30.62 MiB | none | 106.21K/s |
| q39 | date range, group by and offset | 14.000ms | 96.221ms | 100.679ms | 0.0% | 100.679ms | 100.679ms | 100.679ms | 100.679ms | 100.000ms | 30.12 MiB | none | 99.33K/s |
| q40 | date range, a case and a wide group by | 14.000ms | 88.751ms | 59.140ms | 0.0% | 59.140ms | 59.140ms | 59.140ms | 59.140ms | 40.000ms | 32.50 MiB | none | 169.09K/s |
| q41 | date range with an IN and a hash | 9.000ms | 69.666ms | 60.199ms | 0.0% | 60.199ms | 60.199ms | 60.199ms | 60.199ms | 50.000ms | 31.38 MiB | none | 166.12K/s |
| q42 | date range and a deep offset | 6.000ms | 91.346ms | 91.305ms | 0.0% | 91.305ms | 91.305ms | 91.305ms | 91.305ms | 80.000ms | 31.00 MiB | none | 109.52K/s |
| q43 | minute buckets over a date range | 8.000ms | 52.181ms | 50.239ms | 0.0% | 50.239ms | 50.239ms | 50.239ms | 50.239ms | 40.000ms | 29.88 MiB | none | 199.05K/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 561.000ms by its own clock and 3.271s by ours, 3.578s cold, 3.550s of CPU, peak 35.75 MiB, 766.49K/s and 130.03 MiB/s.

Running it cost 483% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 4.03x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 2.000ms | 102.801ms | 109.235ms | 0.0% | 109.235ms | 109.235ms | 109.235ms | 109.235ms | 110.000ms | 35.15 MiB | none | 91.55K/s |
| q2 | filtered count | 6.000ms | 132.468ms | 180.626ms | 0.0% | 180.626ms | 180.626ms | 180.626ms | 180.626ms | 260.000ms | 35.40 MiB | none | 55.36K/s |
| q3 | three aggregates | 6.000ms | 121.385ms | 126.941ms | 0.0% | 126.941ms | 126.941ms | 126.941ms | 126.941ms | 110.000ms | 36.15 MiB | none | 78.78K/s |
| q4 | average | 5.000ms | 119.020ms | 160.033ms | 0.0% | 160.033ms | 160.033ms | 160.033ms | 160.033ms | 280.000ms | 35.91 MiB | none | 62.49K/s |
| q5 | count distinct, high card | 5.000ms | 282.014ms | 119.453ms | 0.0% | 119.453ms | 119.453ms | 119.453ms | 119.453ms | 120.000ms | 37.78 MiB | none | 83.71K/s |
| q6 | count distinct, strings | 6.000ms | 82.611ms | 111.496ms | 0.0% | 111.496ms | 111.496ms | 111.496ms | 111.496ms | 140.000ms | 36.90 MiB | none | 89.69K/s |
| q7 | min and max of a date | 3.000ms | 141.454ms | 105.207ms | 0.0% | 105.207ms | 105.207ms | 105.207ms | 105.207ms | 110.000ms | 35.40 MiB | none | 95.05K/s |
| q8 | group by, low card | 26.000ms | 143.640ms | 295.665ms | 0.0% | 295.665ms | 295.665ms | 295.665ms | 295.665ms | 300.000ms | 38.21 MiB | none | 33.82K/s |
| q9 | group by and count distinct | 13.000ms | 103.132ms | 95.812ms | 0.0% | 95.812ms | 95.812ms | 95.812ms | 95.812ms | 100.000ms | 40.65 MiB | none | 104.37K/s |
| q10 | group by, several aggregates | 86.000ms | 205.538ms | 294.483ms | 0.0% | 294.483ms | 294.483ms | 294.483ms | 294.483ms | 640.000ms | 42.90 MiB | none | 33.96K/s |
| q11 | group by a string and count distinct | 12.000ms | 117.811ms | 136.610ms | 0.0% | 136.610ms | 136.610ms | 136.610ms | 136.610ms | 210.000ms | 39.65 MiB | none | 73.20K/s |
| q12 | group by two strings and count distinct | 36.000ms | 122.007ms | 116.964ms | 0.0% | 116.964ms | 116.964ms | 116.964ms | 116.964ms | 130.000ms | 40.27 MiB | none | 85.50K/s |
| q13 | group by a string and top k | 27.000ms | 135.470ms | 138.082ms | 0.0% | 138.082ms | 138.082ms | 138.082ms | 138.082ms | 170.000ms | 38.15 MiB | none | 72.42K/s |
| q14 | group by a string and count distinct | 16.000ms | 195.516ms | 153.799ms | 0.0% | 153.799ms | 153.799ms | 153.799ms | 153.799ms | 200.000ms | 41.78 MiB | none | 65.02K/s |
| q15 | group by two columns and top k | 26.000ms | 97.774ms | 151.302ms | 0.0% | 151.302ms | 151.302ms | 151.302ms | 151.302ms | 270.000ms | 39.02 MiB | none | 66.09K/s |
| q16 | group by, very high card | 8.000ms | 176.389ms | 122.661ms | 0.0% | 122.661ms | 122.661ms | 122.661ms | 122.661ms | 190.000ms | 39.32 MiB | none | 81.53K/s |
| q17 | group by two, very high card | 8.000ms | 207.341ms | 104.968ms | 0.0% | 104.968ms | 104.968ms | 104.968ms | 104.968ms | 150.000ms | 40.06 MiB | none | 95.27K/s |
| q18 | group by two, no ordering | 59.000ms | 118.256ms | 192.458ms | 0.0% | 192.458ms | 192.458ms | 192.458ms | 192.458ms | 250.000ms | 39.90 MiB | none | 51.96K/s |
| q19 | group by with an extract | 11.000ms | 159.703ms | 189.465ms | 0.0% | 189.465ms | 189.465ms | 189.465ms | 189.465ms | 290.000ms | 40.77 MiB | none | 52.78K/s |
| q20 | point lookup | 10.000ms | 158.366ms | 171.398ms | 0.0% | 171.398ms | 171.398ms | 171.398ms | 171.398ms | 170.000ms | 34.78 MiB | none | 58.34K/s |
| q21 | substring scan | 6.000ms | 114.774ms | 100.909ms | 0.0% | 100.909ms | 100.909ms | 100.909ms | 100.909ms | 110.000ms | 36.40 MiB | none | 99.10K/s |
| q22 | substring scan and group by | 12.000ms | 102.490ms | 105.846ms | 0.0% | 105.846ms | 105.846ms | 105.846ms | 105.846ms | 120.000ms | 37.77 MiB | none | 94.48K/s |
| q23 | two substring scans and group by | 27.000ms | 232.895ms | 124.788ms | 0.0% | 124.788ms | 124.788ms | 124.788ms | 124.788ms | 140.000ms | 43.02 MiB | none | 80.14K/s |
| q24 | select star and top k | 44.000ms | 207.417ms | 149.701ms | 0.0% | 149.701ms | 149.701ms | 149.701ms | 149.701ms | 170.000ms | 48.15 MiB | none | 66.80K/s |
| q25 | top k by a date | 5.000ms | 89.665ms | 93.737ms | 0.0% | 93.737ms | 93.737ms | 93.737ms | 93.737ms | 110.000ms | 36.15 MiB | none | 106.68K/s |
| q26 | top k by a string | 9.000ms | 219.061ms | 269.241ms | 0.0% | 269.241ms | 269.241ms | 269.241ms | 269.241ms | 220.000ms | 36.02 MiB | none | 37.14K/s |
| q27 | top k by two columns | 11.000ms | 278.397ms | 281.196ms | 0.0% | 281.196ms | 281.196ms | 281.196ms | 281.196ms | 420.000ms | 36.27 MiB | none | 35.56K/s |
| q28 | group by with a string length | 11.000ms | 171.009ms | 134.786ms | 0.0% | 134.786ms | 134.786ms | 134.786ms | 134.786ms | 130.000ms | 39.89 MiB | none | 74.19K/s |
| q29 | group by a regular expression | 26.000ms | 166.957ms | 267.686ms | 0.0% | 267.686ms | 267.686ms | 267.686ms | 267.686ms | 270.000ms | 40.40 MiB | none | 37.36K/s |
| q30 | ninety sums over one column | 55.000ms | 286.785ms | 149.646ms | 0.0% | 149.646ms | 149.646ms | 149.646ms | 149.646ms | 140.000ms | 48.78 MiB | none | 66.82K/s |
| q31 | group by two and several aggregates | 39.000ms | 107.916ms | 133.963ms | 0.0% | 133.963ms | 133.963ms | 133.963ms | 133.963ms | 170.000ms | 40.77 MiB | none | 74.65K/s |
| q32 | group by a high card pair | 11.000ms | 152.915ms | 148.924ms | 0.0% | 148.924ms | 148.924ms | 148.924ms | 148.924ms | 260.000ms | 40.78 MiB | none | 67.15K/s |
| q33 | group by a high card pair, unfiltered | 56.000ms | 130.852ms | 261.708ms | 0.0% | 261.708ms | 261.708ms | 261.708ms | 261.708ms | 350.000ms | 41.02 MiB | none | 38.21K/s |
| q34 | group by a long string | 89.000ms | 149.710ms | 352.384ms | 0.0% | 352.384ms | 352.384ms | 352.384ms | 352.384ms | 580.000ms | 40.40 MiB | none | 28.38K/s |
| q35 | group by a constant and a long string | 13.000ms | 253.946ms | 174.544ms | 0.0% | 174.544ms | 174.544ms | 174.544ms | 174.544ms | 240.000ms | 40.15 MiB | none | 57.29K/s |
| q36 | group by four expressions | 8.000ms | 184.253ms | 107.485ms | 0.0% | 107.485ms | 107.485ms | 107.485ms | 107.485ms | 80.000ms | 39.39 MiB | none | 93.04K/s |
| q37 | date range and group by a URL | 11.000ms | 106.305ms | 123.867ms | 0.0% | 123.867ms | 123.867ms | 123.867ms | 123.867ms | 120.000ms | 40.02 MiB | none | 80.73K/s |
| q38 | date range and group by a title | 11.000ms | 295.307ms | 161.950ms | 0.0% | 161.950ms | 161.950ms | 161.950ms | 161.950ms | 120.000ms | 40.15 MiB | none | 61.75K/s |
| q39 | date range, group by and offset | 17.000ms | 171.493ms | 114.592ms | 0.0% | 114.592ms | 114.592ms | 114.592ms | 114.592ms | 100.000ms | 39.77 MiB | none | 87.27K/s |
| q40 | date range, a case and a wide group by | 26.000ms | 228.869ms | 134.607ms | 0.0% | 134.607ms | 134.607ms | 134.607ms | 134.607ms | 180.000ms | 42.02 MiB | none | 74.29K/s |
| q41 | date range with an IN and a hash | 23.000ms | 262.555ms | 265.088ms | 0.0% | 265.088ms | 265.088ms | 265.088ms | 265.088ms | 210.000ms | 39.90 MiB | none | 37.72K/s |
| q42 | date range and a deep offset | 30.000ms | 201.692ms | 561.479ms | 0.0% | 561.479ms | 561.479ms | 561.479ms | 561.479ms | 1.310s | 40.14 MiB | none | 17.81K/s |
| q43 | minute buckets over a date range | 10.000ms | 270.371ms | 348.050ms | 0.0% | 348.050ms | 348.050ms | 348.050ms | 348.050ms | 390.000ms | 37.90 MiB | none | 28.73K/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 921.000ms by its own clock and 7.643s by ours, 7.308s cold, 10.140s of CPU, peak 48.78 MiB, 466.88K/s and 79.20 MiB/s.

Running it cost 730% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 5.99x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 27.000ms | 405.161ms | 403.977ms | 0.0% | 403.977ms | 403.977ms | 403.977ms | 403.977ms | 640.000ms | 237.38 MiB | none | 24.75K/s |
| q2 | filtered count | 11.000ms | 352.653ms | 774.519ms | 0.0% | 774.519ms | 774.519ms | 774.519ms | 774.519ms | 2.850s | 238.62 MiB | none | 12.91K/s |
| q3 | three aggregates | 18.000ms | 1.128s | 321.488ms | 0.0% | 321.488ms | 321.488ms | 321.488ms | 321.488ms | 440.000ms | 240.50 MiB | none | 31.11K/s |
| q4 | average | 11.000ms | 420.190ms | 380.843ms | 0.0% | 380.843ms | 380.843ms | 380.843ms | 380.843ms | 630.000ms | 240.66 MiB | none | 26.26K/s |
| q5 | count distinct, high card | 20.000ms | 507.941ms | 543.127ms | 0.0% | 543.127ms | 543.127ms | 543.127ms | 543.127ms | 1.800s | 242.12 MiB | none | 18.41K/s |
| q6 | count distinct, strings | 12.000ms | 552.826ms | 392.791ms | 0.0% | 392.791ms | 392.791ms | 392.791ms | 392.791ms | 790.000ms | 241.62 MiB | none | 25.46K/s |
| q7 | min and max of a date | 139.000ms | 363.086ms | 640.294ms | 0.0% | 640.294ms | 640.294ms | 640.294ms | 640.294ms | 1.130s | 240.34 MiB | none | 15.62K/s |
| q8 | group by, low card | 23.000ms | 292.844ms | 409.292ms | 0.0% | 409.292ms | 409.292ms | 409.292ms | 409.292ms | 590.000ms | 242.38 MiB | none | 24.43K/s |
| q9 | group by and count distinct | 21.000ms | 635.430ms | 600.142ms | 0.0% | 600.142ms | 600.142ms | 600.142ms | 600.142ms | 560.000ms | 243.50 MiB | none | 16.66K/s |
| q10 | group by, several aggregates | 46.000ms | 705.775ms | 739.062ms | 0.0% | 739.062ms | 739.062ms | 739.062ms | 739.062ms | 570.000ms | 243.88 MiB | none | 13.53K/s |
| q11 | group by a string and count distinct | 59.000ms | 609.026ms | 615.027ms | 0.0% | 615.027ms | 615.027ms | 615.027ms | 615.027ms | 420.000ms | 244.25 MiB | none | 16.26K/s |
| q12 | group by two strings and count distinct | 172.000ms | 942.363ms | 899.061ms | 0.0% | 899.061ms | 899.061ms | 899.061ms | 899.061ms | 530.000ms | 243.88 MiB | none | 11.12K/s |
| q13 | group by a string and top k | 30.000ms | 658.934ms | 840.348ms | 0.0% | 840.348ms | 840.348ms | 840.348ms | 840.348ms | 680.000ms | 243.38 MiB | none | 11.90K/s |
| q14 | group by a string and count distinct | 29.000ms | 461.312ms | 433.420ms | 0.0% | 433.420ms | 433.420ms | 433.420ms | 433.420ms | 450.000ms | 244.88 MiB | none | 23.07K/s |
| q15 | group by two columns and top k | 51.000ms | 429.256ms | 431.005ms | 0.0% | 431.005ms | 431.005ms | 431.005ms | 431.005ms | 420.000ms | 244.25 MiB | none | 23.20K/s |
| q16 | group by, very high card | 17.000ms | 592.743ms | 1.011s | 0.0% | 1.011s | 1.011s | 1.011s | 1.011s | 2.620s | 243.50 MiB | none | 9.89K/s |
| q17 | group by two, very high card | 57.000ms | 506.771ms | 442.007ms | 0.0% | 442.007ms | 442.007ms | 442.007ms | 442.007ms | 830.000ms | 246.00 MiB | none | 22.62K/s |
| q18 | group by two, no ordering | 147.000ms | 395.772ms | 1.424s | 0.0% | 1.424s | 1.424s | 1.424s | 1.424s | 2.830s | 243.88 MiB | none | 7.02K/s |
| q19 | group by with an extract | 33.000ms | 668.175ms | 451.092ms | 0.0% | 451.092ms | 451.092ms | 451.092ms | 451.092ms | 580.000ms | 247.12 MiB | none | 22.17K/s |
| q20 | point lookup | 29.000ms | 672.994ms | 527.071ms | 0.0% | 527.071ms | 527.071ms | 527.071ms | 527.071ms | 560.000ms | 240.00 MiB | none | 18.97K/s |
| q21 | substring scan | 32.000ms | 691.688ms | 415.667ms | 0.0% | 415.667ms | 415.667ms | 415.667ms | 415.667ms | 480.000ms | 243.12 MiB | none | 24.06K/s |
| q22 | substring scan and group by | 103.000ms | 718.603ms | 515.854ms | 0.0% | 515.854ms | 515.854ms | 515.854ms | 515.854ms | 520.000ms | 245.12 MiB | none | 19.39K/s |
| q23 | two substring scans and group by | 74.000ms | 564.282ms | 494.305ms | 0.0% | 494.305ms | 494.305ms | 494.305ms | 494.305ms | 970.000ms | 250.50 MiB | none | 20.23K/s |
| q24 | select star and top k | 87.000ms | 442.355ms | 547.457ms | 0.0% | 547.457ms | 547.457ms | 547.457ms | 547.457ms | 700.000ms | 244.62 MiB | none | 18.27K/s |
| q25 | top k by a date | 19.000ms | 561.986ms | 519.579ms | 0.0% | 519.579ms | 519.579ms | 519.579ms | 519.579ms | 1.000s | 242.67 MiB | none | 19.25K/s |
| q26 | top k by a string | 33.000ms | 409.074ms | 424.282ms | 0.0% | 424.282ms | 424.282ms | 424.282ms | 424.282ms | 760.000ms | 242.07 MiB | none | 23.57K/s |
| q27 | top k by two columns | 22.000ms | 412.754ms | 387.563ms | 0.0% | 387.563ms | 387.563ms | 387.563ms | 387.563ms | 510.000ms | 242.09 MiB | none | 25.80K/s |
| q28 | group by with a string length | 130.000ms | 519.570ms | 1.223s | 0.0% | 1.223s | 1.223s | 1.223s | 1.223s | 1.780s | 245.54 MiB | none | 8.18K/s |
| q29 | group by a regular expression | 54.000ms | 731.595ms | 521.964ms | 0.0% | 521.964ms | 521.964ms | 521.964ms | 521.964ms | 840.000ms | 247.38 MiB | none | 19.16K/s |
| q30 | ninety sums over one column | 56.000ms | 559.933ms | 436.771ms | 0.0% | 436.771ms | 436.771ms | 436.771ms | 436.771ms | 790.000ms | 243.79 MiB | none | 22.90K/s |
| q31 | group by two and several aggregates | 44.000ms | 485.358ms | 513.202ms | 0.0% | 513.202ms | 513.202ms | 513.202ms | 513.202ms | 1.190s | 244.50 MiB | none | 19.49K/s |
| q32 | group by a high card pair | 36.000ms | 518.708ms | 520.903ms | 0.0% | 520.903ms | 520.903ms | 520.903ms | 520.903ms | 910.000ms | 243.80 MiB | none | 19.20K/s |
| q33 | group by a high card pair, unfiltered | 47.000ms | 470.565ms | 498.037ms | 0.0% | 498.037ms | 498.037ms | 498.037ms | 498.037ms | 710.000ms | 245.35 MiB | none | 20.08K/s |
| q34 | group by a long string | 65.000ms | 498.229ms | 577.794ms | 0.0% | 577.794ms | 577.794ms | 577.794ms | 577.794ms | 1.280s | 246.12 MiB | none | 17.31K/s |
| q35 | group by a constant and a long string | 135.000ms | 437.584ms | 571.996ms | 0.0% | 571.996ms | 571.996ms | 571.996ms | 571.996ms | 500.000ms | 247.38 MiB | none | 17.48K/s |
| q36 | group by four expressions | 27.000ms | 310.094ms | 682.488ms | 0.0% | 682.488ms | 682.488ms | 682.488ms | 682.488ms | 610.000ms | 243.62 MiB | none | 14.65K/s |
| q37 | date range and group by a URL | 54.000ms | 498.811ms | 402.192ms | 0.0% | 402.192ms | 402.192ms | 402.192ms | 402.192ms | 610.000ms | 247.75 MiB | none | 24.86K/s |
| q38 | date range and group by a title | 101.000ms | 464.075ms | 656.297ms | 0.0% | 656.297ms | 656.297ms | 656.297ms | 656.297ms | 1.060s | 247.49 MiB | none | 15.24K/s |
| q39 | date range, group by and offset | 44.000ms | 402.606ms | 623.676ms | 0.0% | 623.676ms | 623.676ms | 623.676ms | 623.676ms | 830.000ms | 249.75 MiB | none | 16.03K/s |
| q40 | date range, a case and a wide group by | 279.000ms | 417.542ms | 826.566ms | 0.0% | 826.566ms | 826.566ms | 826.566ms | 826.566ms | 1.190s | 249.75 MiB | none | 12.10K/s |
| q41 | date range with an IN and a hash | 29.000ms | 446.792ms | 430.451ms | 0.0% | 430.451ms | 430.451ms | 430.451ms | 430.451ms | 700.000ms | 246.88 MiB | none | 23.23K/s |
| q42 | date range and a deep offset | 40.000ms | 394.130ms | 412.592ms | 0.0% | 412.592ms | 412.592ms | 412.592ms | 412.592ms | 840.000ms | 246.62 MiB | none | 24.24K/s |
| q43 | minute buckets over a date range | 111.000ms | 537.474ms | 892.203ms | 0.0% | 892.203ms | 892.203ms | 892.203ms | 892.203ms | 820.000ms | 245.00 MiB | none | 11.21K/s |

clickhouse-local 26.9.1.1138 over 43 of 43 queries. Total 2.574s by its own clock and 25.374s by ours, 22.795s cold, 39.520s of CPU, peak 250.50 MiB, 167.06K/s and 28.34 MiB/s.

Running it cost 886% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 4.43x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 4.000ms | 146.546ms | 138.886ms | 0.0% | 138.886ms | 138.886ms | 138.886ms | 138.886ms | 110.000ms | 77.77 MiB | none | 72.00K/s |
| q2 | filtered count | 58.000ms | 84.934ms | 138.643ms | 0.0% | 138.643ms | 138.643ms | 138.643ms | 138.643ms | 90.000ms | 85.77 MiB | none | 72.13K/s |
| q3 | three aggregates | 26.000ms | 80.816ms | 140.297ms | 0.0% | 140.297ms | 140.297ms | 140.297ms | 140.297ms | 120.000ms | 87.68 MiB | none | 71.28K/s |
| q4 | average | 86.000ms | 175.682ms | 209.550ms | 0.0% | 209.550ms | 209.550ms | 209.550ms | 209.550ms | 150.000ms | 87.16 MiB | none | 47.72K/s |
| q5 | count distinct, high card | 12.000ms | 326.538ms | 106.831ms | 0.0% | 106.831ms | 106.831ms | 106.831ms | 106.831ms | 60.000ms | 99.90 MiB | none | 93.61K/s |
| q6 | count distinct, strings | 46.000ms | 241.717ms | 129.560ms | 0.0% | 129.560ms | 129.560ms | 129.560ms | 129.560ms | 100.000ms | 101.59 MiB | none | 77.18K/s |
| q7 | min and max of a date | 4.000ms | 128.463ms | 97.063ms | 0.0% | 97.063ms | 97.063ms | 97.063ms | 97.063ms | 40.000ms | 76.89 MiB | none | 103.03K/s |
| q8 | group by, low card | 34.000ms | 113.174ms | 155.735ms | 0.0% | 155.735ms | 155.735ms | 155.735ms | 155.735ms | 130.000ms | 89.69 MiB | none | 64.21K/s |
| q9 | group by and count distinct | 259.000ms | 180.832ms | 399.165ms | 0.0% | 399.165ms | 399.165ms | 399.165ms | 399.165ms | 330.000ms | 106.72 MiB | none | 25.05K/s |
| q10 | group by, several aggregates | 31.000ms | 88.147ms | 243.751ms | 0.0% | 243.751ms | 243.751ms | 243.751ms | 243.751ms | 190.000ms | 100.57 MiB | none | 41.03K/s |
| q11 | group by a string and count distinct | 35.000ms | 88.594ms | 98.525ms | 0.0% | 98.525ms | 98.525ms | 98.525ms | 98.525ms | 80.000ms | 106.13 MiB | none | 101.50K/s |
| q12 | group by two strings and count distinct | 72.000ms | 149.984ms | 264.462ms | 0.0% | 264.462ms | 264.462ms | 264.462ms | 264.462ms | 150.000ms | 100.44 MiB | none | 37.81K/s |
| q13 | group by a string and top k | 25.000ms | 167.469ms | 109.905ms | 0.0% | 109.905ms | 109.905ms | 109.905ms | 109.905ms | 100.000ms | 110.84 MiB | none | 90.99K/s |
| q14 | group by a string and count distinct | 28.000ms | 102.387ms | 186.031ms | 0.0% | 186.031ms | 186.031ms | 186.031ms | 186.031ms | 190.000ms | 108.54 MiB | none | 53.75K/s |
| q15 | group by two columns and top k | 15.000ms | 106.357ms | 115.013ms | 0.0% | 115.013ms | 115.013ms | 115.013ms | 115.013ms | 50.000ms | 105.19 MiB | none | 86.95K/s |
| q16 | group by, very high card | 13.000ms | 94.693ms | 101.643ms | 0.0% | 101.643ms | 101.643ms | 101.643ms | 101.643ms | 70.000ms | 110.60 MiB | none | 98.38K/s |
| q17 | group by two, very high card | 116.000ms | 149.946ms | 182.067ms | 0.0% | 182.067ms | 182.067ms | 182.067ms | 182.067ms | 110.000ms | 110.33 MiB | none | 54.92K/s |
| q18 | group by two, no ordering | 14.000ms | 174.226ms | 162.019ms | 0.0% | 162.019ms | 162.019ms | 162.019ms | 162.019ms | 170.000ms | 104.98 MiB | none | 61.72K/s |
| q19 | group by with an extract | 104.000ms | 179.169ms | 305.803ms | 0.0% | 305.803ms | 305.803ms | 305.803ms | 305.803ms | 230.000ms | 109.85 MiB | none | 32.70K/s |
| q20 | point lookup | 44.000ms | 83.733ms | 117.062ms | 0.0% | 117.062ms | 117.062ms | 117.062ms | 117.062ms | 70.000ms | 85.64 MiB | none | 85.42K/s |
| q21 | substring scan | 100.000ms | 94.084ms | 167.500ms | 0.0% | 167.500ms | 167.500ms | 167.500ms | 167.500ms | 130.000ms | 90.92 MiB | none | 59.70K/s |
| q22 | substring scan and group by | 49.000ms | 147.469ms | 198.371ms | 0.0% | 198.371ms | 198.371ms | 198.371ms | 198.371ms | 100.000ms | 95.46 MiB | none | 50.41K/s |
| q23 | two substring scans and group by | 36.000ms | 125.205ms | 200.126ms | 0.0% | 200.126ms | 200.126ms | 200.126ms | 200.126ms | 130.000ms | 107.45 MiB | none | 49.97K/s |
| q24 | select star and top k | 129.000ms | 161.664ms | 347.921ms | 0.0% | 347.921ms | 347.921ms | 347.921ms | 347.921ms | 180.000ms | 115.03 MiB | none | 28.74K/s |
| q25 | top k by a date | 12.000ms | 137.438ms | 65.082ms | 0.0% | 65.082ms | 65.082ms | 65.082ms | 65.082ms | 50.000ms | 100.44 MiB | none | 153.65K/s |
| q26 | top k by a string | 63.000ms | 108.777ms | 155.357ms | 0.0% | 155.357ms | 155.357ms | 155.357ms | 155.357ms | 60.000ms | 93.20 MiB | none | 64.37K/s |
| q27 | top k by two columns | 51.000ms | 209.453ms | 249.351ms | 0.0% | 249.351ms | 249.351ms | 249.351ms | 249.351ms | 170.000ms | 94.16 MiB | none | 40.10K/s |
| q28 | group by with a string length | 67.000ms | 305.936ms | 236.866ms | 0.0% | 236.866ms | 236.866ms | 236.866ms | 236.866ms | 130.000ms | 105.29 MiB | none | 42.22K/s |
| q29 | group by a regular expression | 64.000ms | 284.136ms | 131.052ms | 0.0% | 131.052ms | 131.052ms | 131.052ms | 131.052ms | 110.000ms | 119.00 MiB | none | 76.31K/s |
| q30 | ninety sums over one column | 68.000ms | 116.618ms | 194.616ms | 0.0% | 194.616ms | 194.616ms | 194.616ms | 194.616ms | 170.000ms | 88.02 MiB | none | 51.38K/s |
| q31 | group by two and several aggregates | 337.000ms | 164.820ms | 515.329ms | 0.0% | 515.329ms | 515.329ms | 515.329ms | 515.329ms | 230.000ms | 105.52 MiB | none | 19.41K/s |
| q32 | group by a high card pair | 89.000ms | 386.815ms | 438.065ms | 0.0% | 438.065ms | 438.065ms | 438.065ms | 438.065ms | 220.000ms | 105.39 MiB | none | 22.83K/s |
| q33 | group by a high card pair, unfiltered | 195.000ms | 380.636ms | 460.321ms | 0.0% | 460.321ms | 460.321ms | 460.321ms | 460.321ms | 200.000ms | 99.61 MiB | none | 21.72K/s |
| q34 | group by a long string | 97.000ms | 828.526ms | 231.236ms | 0.0% | 231.236ms | 231.236ms | 231.236ms | 231.236ms | 150.000ms | 115.26 MiB | none | 43.25K/s |
| q35 | group by a constant and a long string | 92.000ms | 872.526ms | 210.326ms | 0.0% | 210.326ms | 210.326ms | 210.326ms | 210.326ms | 110.000ms | 116.34 MiB | none | 47.55K/s |
| q36 | group by four expressions | 229.000ms | 104.567ms | 549.450ms | 0.0% | 549.450ms | 549.450ms | 549.450ms | 549.450ms | 420.000ms | 105.71 MiB | none | 18.20K/s |
| q37 | date range and group by a URL | 35.000ms | 301.985ms | 146.330ms | 0.0% | 146.330ms | 146.330ms | 146.330ms | 146.330ms | 70.000ms | 101.29 MiB | none | 68.34K/s |
| q38 | date range and group by a title | 62.000ms | 247.736ms | 157.463ms | 0.0% | 157.463ms | 157.463ms | 157.463ms | 157.463ms | 80.000ms | 104.96 MiB | none | 63.51K/s |
| q39 | date range, group by and offset | 62.000ms | 235.686ms | 261.385ms | 0.0% | 261.385ms | 261.385ms | 261.385ms | 261.385ms | 100.000ms | 101.86 MiB | none | 38.26K/s |
| q40 | date range, a case and a wide group by | 105.000ms | 290.653ms | 408.066ms | 0.0% | 408.066ms | 408.066ms | 408.066ms | 408.066ms | 140.000ms | 98.22 MiB | none | 24.51K/s |
| q41 | date range with an IN and a hash | 178.000ms | 201.489ms | 303.452ms | 0.0% | 303.452ms | 303.452ms | 303.452ms | 303.452ms | 100.000ms | 97.35 MiB | none | 32.95K/s |
| q42 | date range and a deep offset | 104.000ms | 515.403ms | 433.365ms | 0.0% | 433.365ms | 433.365ms | 433.365ms | 433.365ms | 140.000ms | 94.70 MiB | none | 23.08K/s |
| q43 | minute buckets over a date range | 30.000ms | 267.976ms | 280.201ms | 0.0% | 280.201ms | 280.201ms | 280.201ms | 280.201ms | 100.000ms | 98.24 MiB | none | 35.69K/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 3.280s by its own clock and 9.743s by ours, 9.353s cold, 5.830s of CPU, peak 119.00 MiB, 131.10K/s and 22.24 MiB/s.

Running it cost 197% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 8.44x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 102.228ms | 1.041s | 1.650s | 0.0% | 1.650s | 1.650s | 1.650s | 1.650s | 720.000ms | 56.98 MiB | none | 6.06K/s |
| q2 | filtered count | 34.811ms | 1.418s | 1.142s | 0.0% | 1.142s | 1.142s | 1.142s | 1.142s | 700.000ms | 61.48 MiB | none | 8.76K/s |
| q3 | three aggregates | 99.883ms | 950.874ms | 1.045s | 0.0% | 1.045s | 1.045s | 1.045s | 1.045s | 650.000ms | 60.93 MiB | none | 9.57K/s |
| q4 | average | 67.105ms | 957.164ms | 821.913ms | 0.0% | 821.913ms | 821.913ms | 821.913ms | 821.913ms | 630.000ms | 58.64 MiB | none | 12.17K/s |
| q5 | count distinct, high card | 68.045ms | 596.978ms | 630.779ms | 0.0% | 630.779ms | 630.779ms | 630.779ms | 630.779ms | 560.000ms | 63.61 MiB | none | 15.85K/s |
| q6 | count distinct, strings | 42.418ms | 843.456ms | 726.783ms | 0.0% | 726.783ms | 726.783ms | 726.783ms | 726.783ms | 600.000ms | 64.01 MiB | none | 13.76K/s |
| q7 | min and max of a date | 18.130ms | 644.222ms | 749.744ms | 0.0% | 749.744ms | 749.744ms | 749.744ms | 749.744ms | 630.000ms | 59.15 MiB | none | 13.34K/s |
| q8 | group by, low card | 63.499ms | 581.156ms | 715.735ms | 0.0% | 715.735ms | 715.735ms | 715.735ms | 715.735ms | 640.000ms | 65.22 MiB | none | 13.97K/s |
| q9 | group by and count distinct | 91.100ms | 650.286ms | 636.777ms | 0.0% | 636.777ms | 636.777ms | 636.777ms | 636.777ms | 590.000ms | 69.50 MiB | none | 15.70K/s |
| q10 | group by, several aggregates | 39.443ms | 700.853ms | 556.329ms | 0.0% | 556.329ms | 556.329ms | 556.329ms | 556.329ms | 550.000ms | 71.45 MiB | none | 17.97K/s |
| q11 | group by a string and count distinct | 71.277ms | 653.378ms | 629.009ms | 0.0% | 629.009ms | 629.009ms | 629.009ms | 629.009ms | 580.000ms | 67.96 MiB | none | 15.90K/s |
| q12 | group by two strings and count distinct | 50.712ms | 616.048ms | 841.838ms | 0.0% | 841.838ms | 841.838ms | 841.838ms | 841.838ms | 640.000ms | 68.37 MiB | none | 11.88K/s |
| q13 | group by a string and top k | 28.057ms | 804.949ms | 541.491ms | 0.0% | 541.491ms | 541.491ms | 541.491ms | 541.491ms | 530.000ms | 66.46 MiB | none | 18.47K/s |
| q14 | group by a string and count distinct | 90.036ms | 662.499ms | 689.450ms | 0.0% | 689.450ms | 689.450ms | 689.450ms | 689.450ms | 620.000ms | 69.47 MiB | none | 14.50K/s |
| q15 | group by two columns and top k | 45.879ms | 628.798ms | 695.539ms | 0.0% | 695.539ms | 695.539ms | 695.539ms | 695.539ms | 550.000ms | 67.02 MiB | none | 14.38K/s |
| q16 | group by, very high card | 60.479ms | 658.184ms | 647.917ms | 0.0% | 647.917ms | 647.917ms | 647.917ms | 647.917ms | 640.000ms | 64.69 MiB | none | 15.43K/s |
| q17 | group by two, very high card | 105.182ms | 762.946ms | 780.077ms | 0.0% | 780.077ms | 780.077ms | 780.077ms | 780.077ms | 670.000ms | 67.48 MiB | none | 12.82K/s |
| q18 | group by two, no ordering | 63.986ms | 601.886ms | 727.192ms | 0.0% | 727.192ms | 727.192ms | 727.192ms | 727.192ms | 630.000ms | 65.47 MiB | none | 13.75K/s |
| q19 | group by with an extract | 56.641ms | 643.291ms | 768.268ms | 0.0% | 768.268ms | 768.268ms | 768.268ms | 768.268ms | 690.000ms | 68.92 MiB | none | 13.02K/s |
| q20 | point lookup | 78.473ms | 584.825ms | 763.703ms | 0.0% | 763.703ms | 763.703ms | 763.703ms | 763.703ms | 680.000ms | 59.61 MiB | none | 13.09K/s |
| q21 | substring scan | 68.994ms | 673.056ms | 881.084ms | 0.0% | 881.084ms | 881.084ms | 881.084ms | 881.084ms | 660.000ms | 63.84 MiB | none | 11.35K/s |
| q22 | substring scan and group by | 50.914ms | 792.271ms | 742.379ms | 0.0% | 742.379ms | 742.379ms | 742.379ms | 742.379ms | 640.000ms | 68.18 MiB | none | 13.47K/s |
| q23 | two substring scans and group by | 85.085ms | 654.720ms | 533.498ms | 0.0% | 533.498ms | 533.498ms | 533.498ms | 533.498ms | 560.000ms | 74.67 MiB | none | 18.74K/s |
| q24 | select star and top k | 60.365ms | 552.098ms | 614.409ms | 0.0% | 614.409ms | 614.409ms | 614.409ms | 614.409ms | 540.000ms | 66.97 MiB | none | 16.28K/s |
| q25 | top k by a date | 60.308ms | 507.657ms | 550.682ms | 0.0% | 550.682ms | 550.682ms | 550.682ms | 550.682ms | 590.000ms | 64.14 MiB | none | 18.16K/s |
| q26 | top k by a string | 128.736ms | 515.995ms | 592.581ms | 0.0% | 592.581ms | 592.581ms | 592.581ms | 592.581ms | 520.000ms | 62.86 MiB | none | 16.88K/s |
| q27 | top k by two columns | 214.576ms | 705.862ms | 675.033ms | 0.0% | 675.033ms | 675.033ms | 675.033ms | 675.033ms | 750.000ms | 63.84 MiB | none | 14.81K/s |
| q30 | ninety sums over one column | 77.930ms | 988.294ms | 1.197s | 0.0% | 1.197s | 1.197s | 1.197s | 1.197s | 840.000ms | 62.61 MiB | none | 8.35K/s |
| q31 | group by two and several aggregates | 254.146ms | 1.293s | 1.078s | 0.0% | 1.078s | 1.078s | 1.078s | 1.078s | 750.000ms | 68.72 MiB | none | 9.28K/s |
| q32 | group by a high card pair | 241.958ms | 1.205s | 1.612s | 0.0% | 1.612s | 1.612s | 1.612s | 1.612s | 870.000ms | 68.77 MiB | none | 6.20K/s |
| q33 | group by a high card pair, unfiltered | 202.106ms | 1.080s | 1.839s | 0.0% | 1.839s | 1.839s | 1.839s | 1.839s | 910.000ms | 69.95 MiB | none | 5.44K/s |
| q34 | group by a long string | 186.885ms | 1.374s | 1.523s | 0.0% | 1.523s | 1.523s | 1.523s | 1.523s | 830.000ms | 69.84 MiB | none | 6.57K/s |
| q35 | group by a constant and a long string | 82.588ms | 1.427s | 1.033s | 0.0% | 1.033s | 1.033s | 1.033s | 1.033s | 660.000ms | 71.09 MiB | none | 9.68K/s |
| q37 | date range and group by a URL | 115.356ms | 1.409s | 1.164s | 0.0% | 1.164s | 1.164s | 1.164s | 1.164s | 720.000ms | 71.32 MiB | none | 8.59K/s |
| q38 | date range and group by a title | 56.970ms | 1.438s | 511.998ms | 0.0% | 511.998ms | 511.998ms | 511.998ms | 511.998ms | 490.000ms | 71.45 MiB | none | 19.53K/s |
| q39 | date range, group by and offset | 49.228ms | 454.080ms | 584.174ms | 0.0% | 584.174ms | 584.174ms | 584.174ms | 584.174ms | 500.000ms | 69.52 MiB | none | 17.12K/s |
| q40 | date range, a case and a wide group by | 50.247ms | 598.852ms | 522.938ms | 0.0% | 522.938ms | 522.938ms | 522.938ms | 522.938ms | 480.000ms | 70.35 MiB | none | 19.12K/s |
| q41 | date range with an IN and a hash | 37.886ms | 517.613ms | 472.144ms | 0.0% | 472.144ms | 472.144ms | 472.144ms | 472.144ms | 450.000ms | 70.01 MiB | none | 21.18K/s |
| q42 | date range and a deep offset | 60.581ms | 591.467ms | 732.813ms | 0.0% | 732.813ms | 732.813ms | 732.813ms | 732.813ms | 500.000ms | 68.74 MiB | none | 13.65K/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 3.362s by its own clock and 32.618s by ours, 31.780s cold, 24.760s of CPU, peak 74.67 MiB, 115.99K/s and 19.68 MiB/s.

Running it cost 870% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.89x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-server in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 3.000ms | 443.763ms | 471.893ms | 0.0% | 471.893ms | 471.893ms | 471.893ms | 471.893ms | not read | not read | not read | 21.19K/s |
| q2 | filtered count | 8.000ms | 386.809ms | 382.516ms | 0.0% | 382.516ms | 382.516ms | 382.516ms | 382.516ms | not read | not read | not read | 26.14K/s |
| q3 | three aggregates | 9.000ms | 536.313ms | 371.377ms | 0.0% | 371.377ms | 371.377ms | 371.377ms | 371.377ms | not read | not read | not read | 26.93K/s |
| q4 | average | 16.000ms | 616.344ms | 797.876ms | 0.0% | 797.876ms | 797.876ms | 797.876ms | 797.876ms | not read | not read | not read | 12.53K/s |
| q5 | count distinct, high card | 10.000ms | 475.521ms | 613.855ms | 0.0% | 613.855ms | 613.855ms | 613.855ms | 613.855ms | not read | not read | not read | 16.29K/s |
| q6 | count distinct, strings | 21.000ms | 439.420ms | 616.528ms | 0.0% | 616.528ms | 616.528ms | 616.528ms | 616.528ms | not read | not read | not read | 16.22K/s |
| q7 | min and max of a date | 7.000ms | 1.088s | 549.016ms | 0.0% | 549.016ms | 549.016ms | 549.016ms | 549.016ms | not read | not read | not read | 18.21K/s |
| q8 | group by, low card | 10.000ms | 257.410ms | 225.385ms | 0.0% | 225.385ms | 225.385ms | 225.385ms | 225.385ms | not read | not read | not read | 44.37K/s |
| q9 | group by and count distinct | 8.000ms | 283.298ms | 270.119ms | 0.0% | 270.119ms | 270.119ms | 270.119ms | 270.119ms | not read | not read | not read | 37.02K/s |
| q10 | group by, several aggregates | 69.000ms | 278.286ms | 282.851ms | 0.0% | 282.851ms | 282.851ms | 282.851ms | 282.851ms | not read | not read | not read | 35.35K/s |
| q11 | group by a string and count distinct | 8.000ms | 260.976ms | 214.846ms | 0.0% | 214.846ms | 214.846ms | 214.846ms | 214.846ms | not read | not read | not read | 46.54K/s |
| q12 | group by two strings and count distinct | 10.000ms | 203.404ms | 269.399ms | 0.0% | 269.399ms | 269.399ms | 269.399ms | 269.399ms | not read | not read | not read | 37.12K/s |
| q13 | group by a string and top k | 6.000ms | 423.158ms | 248.963ms | 0.0% | 248.963ms | 248.963ms | 248.963ms | 248.963ms | not read | not read | not read | 40.17K/s |
| q14 | group by a string and count distinct | 6.000ms | 253.055ms | 235.014ms | 0.0% | 235.014ms | 235.014ms | 235.014ms | 235.014ms | not read | not read | not read | 42.55K/s |
| q15 | group by two columns and top k | 52.000ms | 201.719ms | 528.626ms | 0.0% | 528.626ms | 528.626ms | 528.626ms | 528.626ms | not read | not read | not read | 18.92K/s |
| q16 | group by, very high card | 5.000ms | 176.236ms | 228.768ms | 0.0% | 228.768ms | 228.768ms | 228.768ms | 228.768ms | not read | not read | not read | 43.71K/s |
| q17 | group by two, very high card | 40.000ms | 243.648ms | 386.733ms | 0.0% | 386.733ms | 386.733ms | 386.733ms | 386.733ms | not read | not read | not read | 25.86K/s |
| q18 | group by two, no ordering | 29.000ms | 367.648ms | 307.302ms | 0.0% | 307.302ms | 307.302ms | 307.302ms | 307.302ms | not read | not read | not read | 32.54K/s |
| q19 | group by with an extract | 22.000ms | 263.244ms | 265.560ms | 0.0% | 265.560ms | 265.560ms | 265.560ms | 265.560ms | not read | not read | not read | 37.66K/s |
| q20 | point lookup | 4.000ms | 221.157ms | 231.776ms | 0.0% | 231.776ms | 231.776ms | 231.776ms | 231.776ms | not read | not read | not read | 43.15K/s |
| q21 | substring scan | 5.000ms | 248.038ms | 334.678ms | 0.0% | 334.678ms | 334.678ms | 334.678ms | 334.678ms | not read | not read | not read | 29.88K/s |
| q22 | substring scan and group by | 42.000ms | 212.668ms | 217.432ms | 0.0% | 217.432ms | 217.432ms | 217.432ms | 217.432ms | not read | not read | not read | 45.99K/s |
| q23 | two substring scans and group by | 13.000ms | 503.372ms | 308.525ms | 0.0% | 308.525ms | 308.525ms | 308.525ms | 308.525ms | not read | not read | not read | 32.41K/s |
| q24 | select star and top k | 14.000ms | 201.409ms | 217.608ms | 0.0% | 217.608ms | 217.608ms | 217.608ms | 217.608ms | not read | not read | not read | 45.95K/s |
| q25 | top k by a date | 10.000ms | 243.978ms | 243.922ms | 0.0% | 243.922ms | 243.922ms | 243.922ms | 243.922ms | not read | not read | not read | 41.00K/s |
| q26 | top k by a string | 6.000ms | 311.692ms | 187.539ms | 0.0% | 187.539ms | 187.539ms | 187.539ms | 187.539ms | not read | not read | not read | 53.32K/s |
| q27 | top k by two columns | 7.000ms | 296.844ms | 223.946ms | 0.0% | 223.946ms | 223.946ms | 223.946ms | 223.946ms | not read | not read | not read | 44.65K/s |
| q28 | group by with a string length | 8.000ms | 207.654ms | 189.662ms | 0.0% | 189.662ms | 189.662ms | 189.662ms | 189.662ms | not read | not read | not read | 52.73K/s |
| q29 | group by a regular expression | 87.000ms | 254.497ms | 399.710ms | 0.0% | 399.710ms | 399.710ms | 399.710ms | 399.710ms | not read | not read | not read | 25.02K/s |
| q30 | ninety sums over one column | 43.000ms | 207.282ms | 286.499ms | 0.0% | 286.499ms | 286.499ms | 286.499ms | 286.499ms | not read | not read | not read | 34.90K/s |
| q31 | group by two and several aggregates | 38.000ms | 219.391ms | 256.846ms | 0.0% | 256.846ms | 256.846ms | 256.846ms | 256.846ms | not read | not read | not read | 38.93K/s |
| q32 | group by a high card pair | 114.000ms | 244.623ms | 322.010ms | 0.0% | 322.010ms | 322.010ms | 322.010ms | 322.010ms | not read | not read | not read | 31.05K/s |
| q33 | group by a high card pair, unfiltered | 15.000ms | 242.488ms | 208.720ms | 0.0% | 208.720ms | 208.720ms | 208.720ms | 208.720ms | not read | not read | not read | 47.91K/s |
| q34 | group by a long string | 12.000ms | 190.707ms | 198.760ms | 0.0% | 198.760ms | 198.760ms | 198.760ms | 198.760ms | not read | not read | not read | 50.31K/s |
| q35 | group by a constant and a long string | 15.000ms | 368.061ms | 277.456ms | 0.0% | 277.456ms | 277.456ms | 277.456ms | 277.456ms | not read | not read | not read | 36.04K/s |
| q36 | group by four expressions | 12.000ms | 507.418ms | 377.898ms | 0.0% | 377.898ms | 377.898ms | 377.898ms | 377.898ms | not read | not read | not read | 26.46K/s |
| q37 | date range and group by a URL | 11.000ms | 428.646ms | 210.537ms | 0.0% | 210.537ms | 210.537ms | 210.537ms | 210.537ms | not read | not read | not read | 47.50K/s |
| q38 | date range and group by a title | 17.000ms | 320.416ms | 191.136ms | 0.0% | 191.136ms | 191.136ms | 191.136ms | 191.136ms | not read | not read | not read | 52.32K/s |
| q39 | date range, group by and offset | 19.000ms | 248.135ms | 205.323ms | 0.0% | 205.323ms | 205.323ms | 205.323ms | 205.323ms | not read | not read | not read | 48.70K/s |
| q40 | date range, a case and a wide group by | 21.000ms | 202.937ms | 253.653ms | 0.0% | 253.653ms | 253.653ms | 253.653ms | 253.653ms | not read | not read | not read | 39.42K/s |
| q41 | date range with an IN and a hash | 14.000ms | 219.604ms | 208.603ms | 0.0% | 208.603ms | 208.603ms | 208.603ms | 208.603ms | not read | not read | not read | 47.94K/s |
| q42 | date range and a deep offset | 10.000ms | 239.692ms | 215.151ms | 0.0% | 215.151ms | 215.151ms | 215.151ms | 215.151ms | not read | not read | not read | 46.48K/s |
| q43 | minute buckets over a date range | 20.000ms | 230.950ms | 239.293ms | 0.0% | 239.293ms | 239.293ms | 239.293ms | 239.293ms | not read | not read | not read | 41.79K/s |

clickhouse-server 26.9.1.1138 over 43 of 43 queries. Total 896.000ms by its own clock and 13.273s by ours, 13.770s cold, no reading of CPU, peak not read, 479.91K/s and 81.41 MiB/s.

Running it cost 1381% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 4.25x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 3.000ms | 10.267ms | 16.417ms | 0.0% | 16.417ms | 16.417ms | 16.417ms | 16.417ms | 0.000us | 4.00 MiB | none | 609.12K/s |
| q2 | filtered count | 65.000ms | 10.799ms | 142.825ms | 0.0% | 142.825ms | 142.825ms | 142.825ms | 142.825ms | 80.000ms | 4.38 MiB | none | 70.02K/s |
| q3 | three aggregates | 2.000ms | 42.577ms | 78.677ms | 0.0% | 78.677ms | 78.677ms | 78.677ms | 78.677ms | 10.000ms | 4.38 MiB | none | 127.10K/s |
| q4 | average | 2.000ms | 62.616ms | 10.198ms | 0.0% | 10.198ms | 10.198ms | 10.198ms | 10.198ms | 0.000us | 4.38 MiB | none | 980.58K/s |
| q5 | count distinct, high card | 13.000ms | 12.243ms | 69.678ms | 0.0% | 69.678ms | 69.678ms | 69.678ms | 69.678ms | 10.000ms | 5.25 MiB | none | 143.52K/s |
| q6 | count distinct, strings | 7.000ms | 10.878ms | 26.234ms | 0.0% | 26.234ms | 26.234ms | 26.234ms | 26.234ms | 0.000us | 4.38 MiB | none | 381.18K/s |
| q7 | min and max of a date | 4.000ms | 9.553ms | 78.187ms | 0.0% | 78.187ms | 78.187ms | 78.187ms | 78.187ms | 10.000ms | 4.25 MiB | none | 127.90K/s |
| q8 | group by, low card | 6.000ms | 10.129ms | 15.239ms | 0.0% | 15.239ms | 15.239ms | 15.239ms | 15.239ms | 0.000us | 4.38 MiB | none | 656.21K/s |
| q9 | group by and count distinct | 12.000ms | 21.886ms | 25.778ms | 0.0% | 25.778ms | 25.778ms | 25.778ms | 25.778ms | 0.000us | 5.50 MiB | none | 387.93K/s |
| q10 | group by, several aggregates | 27.000ms | 14.145ms | 52.349ms | 0.0% | 52.349ms | 52.349ms | 52.349ms | 52.349ms | 20.000ms | 5.88 MiB | none | 191.03K/s |
| q11 | group by a string and count distinct | 2.000ms | 62.683ms | 14.860ms | 0.0% | 14.860ms | 14.860ms | 14.860ms | 14.860ms | 0.000us | 4.50 MiB | none | 672.95K/s |
| q12 | group by two strings and count distinct | 3.000ms | 10.186ms | 10.804ms | 0.0% | 10.804ms | 10.804ms | 10.804ms | 10.804ms | 0.000us | 4.62 MiB | none | 925.58K/s |
| q13 | group by a string and top k | 11.000ms | 40.851ms | 146.142ms | 0.0% | 146.142ms | 146.142ms | 146.142ms | 146.142ms | 10.000ms | 4.88 MiB | none | 68.43K/s |
| q14 | group by a string and count distinct | 17.000ms | 123.956ms | 62.688ms | 0.0% | 62.688ms | 62.688ms | 62.688ms | 62.688ms | 0.000us | 5.00 MiB | none | 159.52K/s |
| q15 | group by two columns and top k | 8.000ms | 116.044ms | 80.265ms | 0.0% | 80.265ms | 80.265ms | 80.265ms | 80.265ms | 10.000ms | 5.12 MiB | none | 124.59K/s |
| q16 | group by, very high card | 17.000ms | 40.586ms | 47.869ms | 0.0% | 47.869ms | 47.869ms | 47.869ms | 47.869ms | 30.000ms | 6.12 MiB | none | 208.90K/s |
| q17 | group by two, very high card | 18.000ms | 36.089ms | 39.000ms | 0.0% | 39.000ms | 39.000ms | 39.000ms | 39.000ms | 10.000ms | 7.15 MiB | none | 256.41K/s |
| q18 | group by two, no ordering | 49.000ms | 21.891ms | 76.593ms | 0.0% | 76.593ms | 76.593ms | 76.593ms | 76.593ms | 50.000ms | 7.14 MiB | none | 130.56K/s |
| q20 | point lookup | 1.000ms | 96.335ms | 11.876ms | 0.0% | 11.876ms | 11.876ms | 11.876ms | 11.876ms | 0.000us | 4.50 MiB | none | 842.03K/s |
| q21 | substring scan | 20.000ms | 25.566ms | 40.988ms | 0.0% | 40.988ms | 40.988ms | 40.988ms | 40.988ms | 20.000ms | 6.11 MiB | none | 243.97K/s |
| q22 | substring scan and group by | 26.000ms | 27.969ms | 50.928ms | 0.0% | 50.928ms | 50.928ms | 50.928ms | 50.928ms | 20.000ms | 6.29 MiB | none | 196.36K/s |
| q23 | two substring scans and group by | 67.000ms | 64.012ms | 90.627ms | 0.0% | 90.627ms | 90.627ms | 90.627ms | 90.627ms | 70.000ms | 8.48 MiB | none | 110.34K/s |
| q24 | select star and top k | 180.000ms | 293.789ms | 226.283ms | 0.0% | 226.283ms | 226.283ms | 226.283ms | 226.283ms | 180.000ms | 19.45 MiB | none | 44.19K/s |
| q25 | top k by a date | 6.000ms | 11.972ms | 15.177ms | 0.0% | 15.177ms | 15.177ms | 15.177ms | 15.177ms | 0.000us | 4.75 MiB | none | 658.89K/s |
| q26 | top k by a string | 5.000ms | 18.618ms | 31.864ms | 0.0% | 31.864ms | 31.864ms | 31.864ms | 31.864ms | 20.000ms | 4.50 MiB | none | 313.83K/s |
| q27 | top k by two columns | 34.000ms | 52.604ms | 134.867ms | 0.0% | 134.867ms | 134.867ms | 134.867ms | 134.867ms | 60.000ms | 4.75 MiB | none | 74.15K/s |
| q28 | group by with a string length | 50.000ms | 85.463ms | 76.515ms | 0.0% | 76.515ms | 76.515ms | 76.515ms | 76.515ms | 60.000ms | 6.23 MiB | none | 130.69K/s |
| q29 | group by a regular expression | 38.000ms | 77.969ms | 50.538ms | 0.0% | 50.538ms | 50.538ms | 50.538ms | 50.538ms | 30.000ms | 6.40 MiB | none | 197.87K/s |
| q30 | ninety sums over one column | 61.000ms | 93.959ms | 121.181ms | 0.0% | 121.181ms | 121.181ms | 121.181ms | 121.181ms | 100.000ms | 4.88 MiB | none | 82.52K/s |
| q31 | group by two and several aggregates | 16.000ms | 22.835ms | 56.970ms | 0.0% | 56.970ms | 56.970ms | 56.970ms | 56.970ms | 10.000ms | 5.50 MiB | none | 175.53K/s |
| q32 | group by a high card pair | 10.000ms | 182.851ms | 20.312ms | 0.0% | 20.312ms | 20.312ms | 20.312ms | 20.312ms | 0.000us | 5.38 MiB | none | 492.32K/s |
| q34 | group by a long string | 20.000ms | 96.970ms | 32.925ms | 0.0% | 32.925ms | 32.925ms | 32.925ms | 32.925ms | 20.000ms | 7.53 MiB | none | 303.72K/s |
| q35 | group by a constant and a long string | 28.000ms | 30.382ms | 35.041ms | 0.0% | 35.041ms | 35.041ms | 35.041ms | 35.041ms | 30.000ms | 8.13 MiB | none | 285.38K/s |
| q36 | group by four expressions | 14.000ms | 67.202ms | 20.627ms | 0.0% | 20.627ms | 20.627ms | 20.627ms | 20.627ms | 0.000us | 8.22 MiB | none | 484.80K/s |
| q37 | date range and group by a URL | 13.000ms | 47.964ms | 40.747ms | 0.0% | 40.747ms | 40.747ms | 40.747ms | 40.747ms | 10.000ms | 6.65 MiB | none | 245.42K/s |
| q38 | date range and group by a title | 22.000ms | 65.192ms | 44.453ms | 0.0% | 44.453ms | 44.453ms | 44.453ms | 44.453ms | 20.000ms | 6.55 MiB | none | 224.96K/s |
| q39 | date range, group by and offset | 33.000ms | 19.346ms | 47.342ms | 0.0% | 47.342ms | 47.342ms | 47.342ms | 47.342ms | 10.000ms | 6.40 MiB | none | 211.23K/s |
| q40 | date range, a case and a wide group by | 32.000ms | 74.007ms | 61.351ms | 0.0% | 61.351ms | 61.351ms | 61.351ms | 61.351ms | 20.000ms | 8.41 MiB | none | 163.00K/s |
| q41 | date range with an IN and a hash | 8.000ms | 79.436ms | 26.847ms | 0.0% | 26.847ms | 26.847ms | 26.847ms | 26.847ms | 10.000ms | 5.12 MiB | none | 372.48K/s |
| q42 | date range and a deep offset | 4.000ms | 27.786ms | 26.953ms | 0.0% | 26.953ms | 26.953ms | 26.953ms | 26.953ms | 0.000us | 4.88 MiB | none | 371.02K/s |
| q43 | minute buckets over a date range | 3.000ms | 8.802ms | 126.723ms | 0.0% | 126.723ms | 126.723ms | 126.723ms | 126.723ms | 0.000us | 4.88 MiB | none | 78.91K/s |

rudb rudb 0.2.28 over 41 of 43 queries. Total 957.000ms by its own clock and 2.385s by ours, 2.228s cold, 930.000ms of CPU, peak 19.45 MiB, 428.42K/s and 72.68 MiB/s.

Running it cost 149% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 22.19x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

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

Answers differ, so this is not a comparison: q12: clickhouse-server does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q12: rudb does not agree with duckdb: 20 numbers against 20

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

- q4: AVG(UserID) over a hundred million bigints near 10^18, where the engines disagree for two reasons. The order the partial sums are added in moves the floating point ones further apart than the one part in a billion this harness calls the same number, and DuckDB is plainly wrong: it sums the bigint column short by a multiple of 2^64 on this file, where its own hugeint and decimal paths, rudb, and adding the column up outside a database all agree. tamnd/rudb-compat#12.
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

Hot here means page cache warm and not buffer pool warm for every row but clickhouse-server, which stayed up across the whole suite with its own caches warm. That is the stronger kind of hot, so a ratio against that column is a ratio between two different quantities.

