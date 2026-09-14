# clickbench on vmi3391933

This is one run of the clickbench suite on vmi3391933, over 7 engines and 43 queries, with 1 hot run of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

It ran over 1000 rows, one out of every 99998 of the 99997497 in the full file, which is a development loop and not the suite. Nothing here is comparable to a full run or to anybody else's number.

## What ran

| what | it was |
| --- | --- |
| suite | clickbench |
| queries | 43 |
| tables | 225.95 KiB of Parquet in 1 table |
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

| engine | version | state | load | load cpu | on disk | that size is | format |
| --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 972.761ms | 500.000ms | 1.01 MiB | its own database file | its own |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 518.921ms | 440.000ms | 1.01 MiB | its own database file | its own |
| clickhouse-local | 26.9.1.1138 | ran | 1.748s | 2.990s | 362.94 KiB | its own MergeTree parts, as system.parts counts the active ones | its own |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 225.95 KiB | the source Parquet, this engine has no storage format of its own | the Parquet |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 225.95 KiB | the source Parquet, this engine has no storage format of its own | the Parquet |
| clickhouse-server | 26.9.1.1138 | ran | 2.324s | not read | 356.92 KiB | its own MergeTree parts, sorted by hits by (CounterID, EventDate, UserID, EventTime, WatchID), as system.parts counts the active ones | its own |
| rudb | rudb 0.2.28 | ran | 0.000us | 0.000us | 225.95 KiB | the source Parquet, this engine has no storage format of its own yet | the Parquet |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 835.000ms | 4.405s | +428% | 4.083s | 3.460s | 0.79 | 33.50 MiB | none | 51.50K/s | 11.36 MiB/s | 1.00x |
| duckdb-pinned | 895.000ms | 6.827s | +663% | 7.028s | 7.190s | 1.05 | 47.91 MiB | none | 48.04K/s | 10.60 MiB/s | 1.07x |
| clickhouse-local | 2.979s | 30.403s | +921% | 29.249s | 37.200s | 1.22 | 246.75 MiB | 128.00 KiB | 14.43K/s | 3.19 MiB/s | 3.57x |
| datafusion | 1.508s | 6.043s | +301% | 5.322s | 5.880s | 0.97 | 114.16 MiB | none | 28.51K/s | 6.29 MiB/s | 1.81x |
| polars | 3.208s | 25.119s | +683% | 26.251s | 25.920s | 1.03 | 69.74 MiB | none | 12.16K/s | 2.68 MiB/s | 3.84x |
| clickhouse-server | 854.000ms | 14.047s | +1545% | 13.496s | not read | not read | not read | not read | 50.35K/s | 11.11 MiB/s | 1.02x |
| rudb | 151.000ms | 808.161ms | +435% | 825.630ms | 160.000ms | 0.20 | 5.75 MiB | none | 271.52K/s | 59.91 MiB/s | 0.18x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | clickhouse-server | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 4.000ms | 3.000ms | 12.000ms | 6.000ms | 135.414ms | 8.000ms | 1.000ms |
| q2 | filtered count | 38.000ms | 3.000ms | 8.000ms | 24.000ms | 23.290ms | 23.000ms | 1.000ms |
| q3 | three aggregates | 6.000ms | 4.000ms | 25.000ms | 7.000ms | 66.421ms | 55.000ms | 1.000ms |
| q4 | average | 55.000ms | 49.000ms | 456.000ms | 8.000ms | 21.220ms | 7.000ms | 27.000ms |
| q5 | count distinct, high card | 24.000ms | 24.000ms | 15.000ms | 16.000ms | 57.054ms | 7.000ms | 1.000ms |
| q6 | count distinct, strings | 47.000ms | 9.000ms | 70.000ms | 13.000ms | 161.458ms | 8.000ms | 1.000ms |
| q7 | min and max of a date | 3.000ms | 3.000ms | 26.000ms | 5.000ms | 26.469ms | 36.000ms | 1.000ms |
| q8 | group by, low card | 34.000ms | 27.000ms | 22.000ms | 15.000ms | 95.968ms | 8.000ms | 1.000ms |
| q9 | group by and count distinct | 121.000ms | 10.000ms | 35.000ms | 21.000ms | 114.957ms | 5.000ms | 2.000ms |
| q10 | group by, several aggregates | 13.000ms | 17.000ms | 56.000ms | 16.000ms | 77.383ms | 97.000ms | 2.000ms |
| q11 | group by a string and count distinct | 13.000ms | 69.000ms | 17.000ms | 35.000ms | 99.270ms | 48.000ms | 1.000ms |
| q12 | group by two strings and count distinct | 10.000ms | 52.000ms | 18.000ms | 22.000ms | 46.593ms | 63.000ms | 1.000ms |
| q13 | group by a string and top k | 12.000ms | 11.000ms | 31.000ms | 13.000ms | 51.495ms | 15.000ms | 2.000ms |
| q14 | group by a string and count distinct | 17.000ms | 10.000ms | 17.000ms | 16.000ms | 67.571ms | 9.000ms | 2.000ms |
| q15 | group by two columns and top k | 8.000ms | 8.000ms | 48.000ms | 19.000ms | 49.685ms | 6.000ms | 2.000ms |
| q16 | group by, very high card | 9.000ms | 18.000ms | 36.000ms | 10.000ms | 112.829ms | 6.000ms | 2.000ms |
| q17 | group by two, very high card | 10.000ms | 11.000ms | 19.000ms | 33.000ms | 101.044ms | 6.000ms | 3.000ms |
| q18 | group by two, no ordering | 6.000ms | 9.000ms | 16.000ms | 28.000ms | 121.051ms | 12.000ms | 3.000ms |
| q19 | group by with an extract | 7.000ms | 18.000ms | 25.000ms | 49.000ms | 96.179ms | 16.000ms | no dialect |
| q20 | point lookup | 3.000ms | 2.000ms | 15.000ms | 77.000ms | 24.173ms | 12.000ms | 1.000ms |
| q21 | substring scan | 2.000ms | 3.000ms | 14.000ms | 10.000ms | 30.523ms | 5.000ms | 3.000ms |
| q22 | substring scan and group by | 19.000ms | 7.000ms | 67.000ms | 36.000ms | 75.193ms | 23.000ms | 3.000ms |
| q23 | two substring scans and group by | 5.000ms | 10.000ms | 20.000ms | 58.000ms | 98.360ms | 10.000ms | 6.000ms |
| q24 | select star and top k | 85.000ms | 79.000ms | 79.000ms | 40.000ms | 34.685ms | 18.000ms | 13.000ms |
| q25 | top k by a date | 7.000ms | 10.000ms | 36.000ms | 66.000ms | 85.051ms | 5.000ms | 17.000ms |
| q26 | top k by a string | 2.000ms | 8.000ms | 26.000ms | 11.000ms | 90.070ms | 10.000ms | 1.000ms |
| q27 | top k by two columns | 4.000ms | 4.000ms | 57.000ms | 15.000ms | 25.478ms | 5.000ms | 2.000ms |
| q28 | group by with a string length | 21.000ms | 42.000ms | 43.000ms | 27.000ms | no dialect | 7.000ms | 3.000ms |
| q29 | group by a regular expression | 8.000ms | 32.000ms | 22.000ms | 88.000ms | no dialect | 95.000ms | 5.000ms |
| q30 | ninety sums over one column | 22.000ms | 88.000ms | 44.000ms | 41.000ms | 36.377ms | 23.000ms | 5.000ms |
| q31 | group by two and several aggregates | 27.000ms | 30.000ms | 90.000ms | 25.000ms | 109.810ms | 6.000ms | 2.000ms |
| q32 | group by a high card pair | 8.000ms | 14.000ms | 18.000ms | 23.000ms | 51.487ms | 42.000ms | 3.000ms |
| q33 | group by a high card pair, unfiltered | 18.000ms | 13.000ms | 51.000ms | 130.000ms | 81.260ms | 10.000ms | no dialect |
| q34 | group by a long string | 16.000ms | 30.000ms | 71.000ms | 18.000ms | 67.800ms | 11.000ms | 4.000ms |
| q35 | group by a constant and a long string | 9.000ms | 11.000ms | 79.000ms | 55.000ms | 44.601ms | 8.000ms | 9.000ms |
| q36 | group by four expressions | 13.000ms | 31.000ms | 79.000ms | 39.000ms | no dialect | 10.000ms | 3.000ms |
| q37 | date range and group by a URL | 9.000ms | 12.000ms | 178.000ms | 46.000ms | 50.205ms | 14.000ms | 3.000ms |
| q38 | date range and group by a title | 14.000ms | 41.000ms | 148.000ms | 27.000ms | 213.317ms | 15.000ms | 3.000ms |
| q39 | date range, group by and offset | 4.000ms | 8.000ms | 270.000ms | 21.000ms | 60.800ms | 11.000ms | 2.000ms |
| q40 | date range, a case and a wide group by | 12.000ms | 11.000ms | 165.000ms | 23.000ms | 73.534ms | 18.000ms | 4.000ms |
| q41 | date range with an IN and a hash | 28.000ms | 10.000ms | 52.000ms | 33.000ms | 146.779ms | 37.000ms | 1.000ms |
| q42 | date range and a deep offset | 14.000ms | 26.000ms | 28.000ms | 196.000ms | 283.141ms | 14.000ms | 2.000ms |
| q43 | minute buckets over a date range | 48.000ms | 18.000ms | 375.000ms | 47.000ms | no dialect | 10.000ms | 2.000ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 4.000ms | 115.564ms | 96.410ms | 0.0% | 96.410ms | 96.410ms | 96.410ms | 96.410ms | 40.000ms | 25.62 MiB | none | 10.37K/s |
| q2 | filtered count | 38.000ms | 175.310ms | 133.705ms | 0.0% | 133.705ms | 133.705ms | 133.705ms | 133.705ms | 80.000ms | 26.75 MiB | 12.00 KiB | 7.48K/s |
| q3 | three aggregates | 6.000ms | 106.346ms | 165.201ms | 0.0% | 165.201ms | 165.201ms | 165.201ms | 165.201ms | 60.000ms | 26.88 MiB | 84.00 KiB | 6.05K/s |
| q4 | average | 55.000ms | 84.014ms | 132.799ms | 0.0% | 132.799ms | 132.799ms | 132.799ms | 132.799ms | 90.000ms | 26.50 MiB | none | 7.53K/s |
| q5 | count distinct, high card | 24.000ms | 192.708ms | 123.572ms | 0.0% | 123.572ms | 123.572ms | 123.572ms | 123.572ms | 60.000ms | 28.00 MiB | 656.00 KiB | 8.09K/s |
| q6 | count distinct, strings | 47.000ms | 152.760ms | 205.147ms | 0.0% | 205.147ms | 205.147ms | 205.147ms | 205.147ms | 90.000ms | 28.38 MiB | none | 4.87K/s |
| q7 | min and max of a date | 3.000ms | 137.458ms | 95.889ms | 0.0% | 95.889ms | 95.889ms | 95.889ms | 95.889ms | 70.000ms | 26.00 MiB | 64.00 KiB | 10.43K/s |
| q8 | group by, low card | 34.000ms | 169.885ms | 213.584ms | 0.0% | 213.584ms | 213.584ms | 213.584ms | 213.584ms | 90.000ms | 28.50 MiB | 848.00 KiB | 4.68K/s |
| q9 | group by and count distinct | 121.000ms | 123.488ms | 310.792ms | 0.0% | 310.792ms | 310.792ms | 310.792ms | 310.792ms | 240.000ms | 31.88 MiB | 120.00 KiB | 3.22K/s |
| q10 | group by, several aggregates | 13.000ms | 147.238ms | 117.968ms | 0.0% | 117.968ms | 117.968ms | 117.968ms | 117.968ms | 80.000ms | 33.50 MiB | none | 8.48K/s |
| q11 | group by a string and count distinct | 13.000ms | 98.640ms | 90.423ms | 0.0% | 90.423ms | 90.423ms | 90.423ms | 90.423ms | 80.000ms | 31.50 MiB | 72.00 KiB | 11.06K/s |
| q12 | group by two strings and count distinct | 10.000ms | 79.238ms | 54.803ms | 0.0% | 54.803ms | 54.803ms | 54.803ms | 54.803ms | 60.000ms | 31.25 MiB | none | 18.25K/s |
| q13 | group by a string and top k | 12.000ms | 96.942ms | 66.815ms | 0.0% | 66.815ms | 66.815ms | 66.815ms | 66.815ms | 70.000ms | 29.25 MiB | none | 14.97K/s |
| q14 | group by a string and count distinct | 17.000ms | 60.523ms | 119.361ms | 0.0% | 119.361ms | 119.361ms | 119.361ms | 119.361ms | 130.000ms | 31.50 MiB | none | 8.38K/s |
| q15 | group by two columns and top k | 8.000ms | 106.308ms | 86.452ms | 0.0% | 86.452ms | 86.452ms | 86.452ms | 86.452ms | 80.000ms | 29.62 MiB | none | 11.57K/s |
| q16 | group by, very high card | 9.000ms | 116.202ms | 68.449ms | 0.0% | 68.449ms | 68.449ms | 68.449ms | 68.449ms | 50.000ms | 30.38 MiB | 1.64 MiB | 14.61K/s |
| q17 | group by two, very high card | 10.000ms | 54.701ms | 130.252ms | 0.0% | 130.252ms | 130.252ms | 130.252ms | 130.252ms | 120.000ms | 31.12 MiB | none | 7.68K/s |
| q18 | group by two, no ordering | 6.000ms | 56.743ms | 59.441ms | 0.0% | 59.441ms | 59.441ms | 59.441ms | 59.441ms | 50.000ms | 31.25 MiB | none | 16.82K/s |
| q19 | group by with an extract | 7.000ms | 81.590ms | 71.610ms | 0.0% | 71.610ms | 71.610ms | 71.610ms | 71.610ms | 60.000ms | 31.38 MiB | 124.00 KiB | 13.96K/s |
| q20 | point lookup | 3.000ms | 69.503ms | 49.802ms | 0.0% | 49.802ms | 49.802ms | 49.802ms | 49.802ms | 40.000ms | 26.12 MiB | none | 20.08K/s |
| q21 | substring scan | 2.000ms | 70.647ms | 60.793ms | 0.0% | 60.793ms | 60.793ms | 60.793ms | 60.793ms | 30.000ms | 26.88 MiB | none | 16.45K/s |
| q22 | substring scan and group by | 19.000ms | 60.207ms | 71.825ms | 0.0% | 71.825ms | 71.825ms | 71.825ms | 71.825ms | 90.000ms | 27.62 MiB | none | 13.92K/s |
| q23 | two substring scans and group by | 5.000ms | 98.706ms | 69.519ms | 0.0% | 69.519ms | 69.519ms | 69.519ms | 69.519ms | 60.000ms | 28.00 MiB | none | 14.38K/s |
| q24 | select star and top k | 85.000ms | 85.846ms | 179.553ms | 0.0% | 179.553ms | 179.553ms | 179.553ms | 179.553ms | 260.000ms | 32.88 MiB | 120.00 KiB | 5.57K/s |
| q25 | top k by a date | 7.000ms | 83.679ms | 61.139ms | 0.0% | 61.139ms | 61.139ms | 61.139ms | 61.139ms | 60.000ms | 29.25 MiB | none | 16.36K/s |
| q26 | top k by a string | 2.000ms | 56.894ms | 69.226ms | 0.0% | 69.226ms | 69.226ms | 69.226ms | 69.226ms | 60.000ms | 26.50 MiB | 108.00 KiB | 14.45K/s |
| q27 | top k by two columns | 4.000ms | 60.503ms | 80.588ms | 0.0% | 80.588ms | 80.588ms | 80.588ms | 80.588ms | 60.000ms | 27.00 MiB | none | 12.41K/s |
| q28 | group by with a string length | 21.000ms | 89.586ms | 62.289ms | 0.0% | 62.289ms | 62.289ms | 62.289ms | 62.289ms | 60.000ms | 29.50 MiB | 96.00 KiB | 16.05K/s |
| q29 | group by a regular expression | 8.000ms | 56.721ms | 76.421ms | 0.0% | 76.421ms | 76.421ms | 76.421ms | 76.421ms | 70.000ms | 29.38 MiB | 192.00 KiB | 13.09K/s |
| q30 | ninety sums over one column | 22.000ms | 94.715ms | 68.084ms | 0.0% | 68.084ms | 68.084ms | 68.084ms | 68.084ms | 70.000ms | 30.12 MiB | 156.00 KiB | 14.69K/s |
| q31 | group by two and several aggregates | 27.000ms | 118.830ms | 96.470ms | 0.0% | 96.470ms | 96.470ms | 96.470ms | 96.470ms | 140.000ms | 31.25 MiB | none | 10.37K/s |
| q32 | group by a high card pair | 8.000ms | 51.783ms | 88.394ms | 0.0% | 88.394ms | 88.394ms | 88.394ms | 88.394ms | 100.000ms | 30.50 MiB | none | 11.31K/s |
| q33 | group by a high card pair, unfiltered | 18.000ms | 61.968ms | 83.377ms | 0.0% | 83.377ms | 83.377ms | 83.377ms | 83.377ms | 90.000ms | 30.88 MiB | none | 11.99K/s |
| q34 | group by a long string | 16.000ms | 59.840ms | 84.838ms | 0.0% | 84.838ms | 84.838ms | 84.838ms | 84.838ms | 90.000ms | 29.50 MiB | none | 11.79K/s |
| q35 | group by a constant and a long string | 9.000ms | 56.933ms | 54.000ms | 0.0% | 54.000ms | 54.000ms | 54.000ms | 54.000ms | 50.000ms | 30.00 MiB | none | 18.52K/s |
| q36 | group by four expressions | 13.000ms | 59.980ms | 64.447ms | 0.0% | 64.447ms | 64.447ms | 64.447ms | 64.447ms | 60.000ms | 31.12 MiB | 24.00 KiB | 15.52K/s |
| q37 | date range and group by a URL | 9.000ms | 60.925ms | 54.494ms | 0.0% | 54.494ms | 54.494ms | 54.494ms | 54.494ms | 50.000ms | 29.25 MiB | none | 18.35K/s |
| q38 | date range and group by a title | 14.000ms | 49.346ms | 90.146ms | 0.0% | 90.146ms | 90.146ms | 90.146ms | 90.146ms | 120.000ms | 29.50 MiB | none | 11.09K/s |
| q39 | date range, group by and offset | 4.000ms | 65.470ms | 54.726ms | 0.0% | 54.726ms | 54.726ms | 54.726ms | 54.726ms | 40.000ms | 27.62 MiB | none | 18.27K/s |
| q40 | date range, a case and a wide group by | 12.000ms | 183.887ms | 162.742ms | 0.0% | 162.742ms | 162.742ms | 162.742ms | 162.742ms | 80.000ms | 30.88 MiB | 160.00 KiB | 6.14K/s |
| q41 | date range with an IN and a hash | 28.000ms | 111.851ms | 117.932ms | 0.0% | 117.932ms | 117.932ms | 117.932ms | 117.932ms | 70.000ms | 30.88 MiB | none | 8.48K/s |
| q42 | date range and a deep offset | 14.000ms | 93.601ms | 162.205ms | 0.0% | 162.205ms | 162.205ms | 162.205ms | 162.205ms | 60.000ms | 29.00 MiB | none | 6.17K/s |
| q43 | minute buckets over a date range | 48.000ms | 125.469ms | 99.471ms | 0.0% | 99.471ms | 99.471ms | 99.471ms | 99.471ms | 50.000ms | 29.38 MiB | none | 10.05K/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 835.000ms by its own clock and 4.405s by ours, 4.083s cold, 3.460s of CPU, peak 33.50 MiB, 51.50K/s and 11.36 MiB/s.

Running it cost 428% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 6.24x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 3.000ms | 128.167ms | 160.419ms | 0.0% | 160.419ms | 160.419ms | 160.419ms | 160.419ms | 150.000ms | 35.28 MiB | none | 6.23K/s |
| q2 | filtered count | 3.000ms | 102.445ms | 194.763ms | 0.0% | 194.763ms | 194.763ms | 194.763ms | 194.763ms | 190.000ms | 35.46 MiB | none | 5.13K/s |
| q3 | three aggregates | 4.000ms | 122.075ms | 121.872ms | 0.0% | 121.872ms | 121.872ms | 121.872ms | 121.872ms | 110.000ms | 36.03 MiB | none | 8.21K/s |
| q4 | average | 49.000ms | 99.432ms | 171.105ms | 0.0% | 171.105ms | 171.105ms | 171.105ms | 171.105ms | 190.000ms | 35.91 MiB | none | 5.84K/s |
| q5 | count distinct, high card | 24.000ms | 230.792ms | 293.594ms | 0.0% | 293.594ms | 293.594ms | 293.594ms | 293.594ms | 260.000ms | 38.16 MiB | none | 3.41K/s |
| q6 | count distinct, strings | 9.000ms | 192.908ms | 122.559ms | 0.0% | 122.559ms | 122.559ms | 122.559ms | 122.559ms | 90.000ms | 36.40 MiB | none | 8.16K/s |
| q7 | min and max of a date | 3.000ms | 148.393ms | 122.523ms | 0.0% | 122.523ms | 122.523ms | 122.523ms | 122.523ms | 120.000ms | 35.52 MiB | none | 8.16K/s |
| q8 | group by, low card | 27.000ms | 200.180ms | 164.712ms | 0.0% | 164.712ms | 164.712ms | 164.712ms | 164.712ms | 160.000ms | 38.53 MiB | none | 6.07K/s |
| q9 | group by and count distinct | 10.000ms | 206.441ms | 183.364ms | 0.0% | 183.364ms | 183.364ms | 183.364ms | 183.364ms | 130.000ms | 39.52 MiB | none | 5.45K/s |
| q10 | group by, several aggregates | 17.000ms | 130.295ms | 183.171ms | 0.0% | 183.171ms | 183.171ms | 183.171ms | 183.171ms | 170.000ms | 43.02 MiB | none | 5.46K/s |
| q11 | group by a string and count distinct | 69.000ms | 423.873ms | 204.494ms | 0.0% | 204.494ms | 204.494ms | 204.494ms | 204.494ms | 240.000ms | 40.25 MiB | none | 4.89K/s |
| q12 | group by two strings and count distinct | 52.000ms | 182.356ms | 231.987ms | 0.0% | 231.987ms | 231.987ms | 231.987ms | 231.987ms | 240.000ms | 40.77 MiB | none | 4.31K/s |
| q13 | group by a string and top k | 11.000ms | 265.441ms | 136.380ms | 0.0% | 136.380ms | 136.380ms | 136.380ms | 136.380ms | 130.000ms | 37.90 MiB | none | 7.33K/s |
| q14 | group by a string and count distinct | 10.000ms | 151.457ms | 130.295ms | 0.0% | 130.295ms | 130.295ms | 130.295ms | 130.295ms | 120.000ms | 40.41 MiB | none | 7.67K/s |
| q15 | group by two columns and top k | 8.000ms | 205.185ms | 148.862ms | 0.0% | 148.862ms | 148.862ms | 148.862ms | 148.862ms | 150.000ms | 38.27 MiB | none | 6.72K/s |
| q16 | group by, very high card | 18.000ms | 112.750ms | 105.492ms | 0.0% | 105.492ms | 105.492ms | 105.492ms | 105.492ms | 90.000ms | 38.77 MiB | none | 9.48K/s |
| q17 | group by two, very high card | 11.000ms | 161.742ms | 121.987ms | 0.0% | 121.987ms | 121.987ms | 121.987ms | 121.987ms | 110.000ms | 39.40 MiB | none | 8.20K/s |
| q18 | group by two, no ordering | 9.000ms | 108.201ms | 159.045ms | 0.0% | 159.045ms | 159.045ms | 159.045ms | 159.045ms | 160.000ms | 38.77 MiB | none | 6.29K/s |
| q19 | group by with an extract | 18.000ms | 222.264ms | 190.986ms | 0.0% | 190.986ms | 190.986ms | 190.986ms | 190.986ms | 220.000ms | 40.27 MiB | none | 5.24K/s |
| q20 | point lookup | 2.000ms | 153.359ms | 114.690ms | 0.0% | 114.690ms | 114.690ms | 114.690ms | 114.690ms | 110.000ms | 35.03 MiB | none | 8.72K/s |
| q21 | substring scan | 3.000ms | 115.460ms | 170.445ms | 0.0% | 170.445ms | 170.445ms | 170.445ms | 170.445ms | 210.000ms | 35.68 MiB | none | 5.87K/s |
| q22 | substring scan and group by | 7.000ms | 110.931ms | 117.260ms | 0.0% | 117.260ms | 117.260ms | 117.260ms | 117.260ms | 110.000ms | 36.64 MiB | none | 8.53K/s |
| q23 | two substring scans and group by | 10.000ms | 180.639ms | 136.896ms | 0.0% | 136.896ms | 136.896ms | 136.896ms | 136.896ms | 140.000ms | 39.02 MiB | none | 7.30K/s |
| q24 | select star and top k | 79.000ms | 186.436ms | 197.014ms | 0.0% | 197.014ms | 197.014ms | 197.014ms | 197.014ms | 210.000ms | 43.40 MiB | none | 5.08K/s |
| q25 | top k by a date | 10.000ms | 162.346ms | 189.390ms | 0.0% | 189.390ms | 189.390ms | 189.390ms | 189.390ms | 230.000ms | 36.03 MiB | none | 5.28K/s |
| q26 | top k by a string | 8.000ms | 143.887ms | 118.864ms | 0.0% | 118.864ms | 118.864ms | 118.864ms | 118.864ms | 120.000ms | 35.77 MiB | none | 8.41K/s |
| q27 | top k by two columns | 4.000ms | 146.166ms | 105.657ms | 0.0% | 105.657ms | 105.657ms | 105.657ms | 105.657ms | 100.000ms | 36.02 MiB | none | 9.46K/s |
| q28 | group by with a string length | 42.000ms | 130.308ms | 167.462ms | 0.0% | 167.462ms | 167.462ms | 167.462ms | 167.462ms | 190.000ms | 38.14 MiB | none | 5.97K/s |
| q29 | group by a regular expression | 32.000ms | 172.696ms | 132.705ms | 0.0% | 132.705ms | 132.705ms | 132.705ms | 132.705ms | 120.000ms | 39.40 MiB | none | 7.54K/s |
| q30 | ninety sums over one column | 88.000ms | 207.625ms | 242.481ms | 0.0% | 242.481ms | 242.481ms | 242.481ms | 242.481ms | 270.000ms | 47.91 MiB | none | 4.12K/s |
| q31 | group by two and several aggregates | 30.000ms | 114.801ms | 121.485ms | 0.0% | 121.485ms | 121.485ms | 121.485ms | 121.485ms | 110.000ms | 40.27 MiB | none | 8.23K/s |
| q32 | group by a high card pair | 14.000ms | 129.087ms | 123.535ms | 0.0% | 123.535ms | 123.535ms | 123.535ms | 123.535ms | 120.000ms | 40.27 MiB | none | 8.09K/s |
| q33 | group by a high card pair, unfiltered | 13.000ms | 124.547ms | 117.252ms | 0.0% | 117.252ms | 117.252ms | 117.252ms | 117.252ms | 150.000ms | 40.27 MiB | none | 8.53K/s |
| q34 | group by a long string | 30.000ms | 98.260ms | 163.437ms | 0.0% | 163.437ms | 163.437ms | 163.437ms | 163.437ms | 210.000ms | 37.48 MiB | none | 6.12K/s |
| q35 | group by a constant and a long string | 11.000ms | 203.857ms | 95.279ms | 0.0% | 95.279ms | 95.279ms | 95.279ms | 95.279ms | 110.000ms | 38.40 MiB | none | 10.50K/s |
| q36 | group by four expressions | 31.000ms | 104.943ms | 203.212ms | 0.0% | 203.212ms | 203.212ms | 203.212ms | 203.212ms | 250.000ms | 39.15 MiB | none | 4.92K/s |
| q37 | date range and group by a URL | 12.000ms | 138.755ms | 134.835ms | 0.0% | 134.835ms | 134.835ms | 134.835ms | 134.835ms | 140.000ms | 38.27 MiB | none | 7.42K/s |
| q38 | date range and group by a title | 41.000ms | 118.618ms | 145.729ms | 0.0% | 145.729ms | 145.729ms | 145.729ms | 145.729ms | 190.000ms | 38.02 MiB | none | 6.86K/s |
| q39 | date range, group by and offset | 8.000ms | 109.461ms | 230.644ms | 0.0% | 230.644ms | 230.644ms | 230.644ms | 230.644ms | 370.000ms | 36.77 MiB | none | 4.34K/s |
| q40 | date range, a case and a wide group by | 11.000ms | 190.468ms | 158.927ms | 0.0% | 158.927ms | 158.927ms | 158.927ms | 158.927ms | 150.000ms | 39.64 MiB | none | 6.29K/s |
| q41 | date range with an IN and a hash | 10.000ms | 174.692ms | 147.191ms | 0.0% | 147.191ms | 147.191ms | 147.191ms | 147.191ms | 120.000ms | 40.15 MiB | none | 6.79K/s |
| q42 | date range and a deep offset | 26.000ms | 211.790ms | 196.066ms | 0.0% | 196.066ms | 196.066ms | 196.066ms | 196.066ms | 230.000ms | 39.40 MiB | none | 5.10K/s |
| q43 | minute buckets over a date range | 18.000ms | 204.847ms | 148.967ms | 0.0% | 148.967ms | 148.967ms | 148.967ms | 148.967ms | 200.000ms | 37.77 MiB | none | 6.71K/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 895.000ms by its own clock and 6.827s by ours, 7.028s cold, 7.190s of CPU, peak 47.91 MiB, 48.04K/s and 10.60 MiB/s.

Running it cost 663% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.08x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 12.000ms | 941.811ms | 470.543ms | 0.0% | 470.543ms | 470.543ms | 470.543ms | 470.543ms | 880.000ms | 237.25 MiB | none | 2.13K/s |
| q2 | filtered count | 8.000ms | 447.095ms | 457.740ms | 0.0% | 457.740ms | 457.740ms | 457.740ms | 457.740ms | 950.000ms | 238.48 MiB | none | 2.18K/s |
| q3 | three aggregates | 25.000ms | 380.968ms | 784.582ms | 0.0% | 784.582ms | 784.582ms | 784.582ms | 784.582ms | 2.260s | 240.88 MiB | none | 1.27K/s |
| q4 | average | 456.000ms | 711.195ms | 1.051s | 0.0% | 1.051s | 1.051s | 1.051s | 1.051s | 1.600s | 240.88 MiB | none | 951/s |
| q5 | count distinct, high card | 15.000ms | 554.357ms | 369.543ms | 0.0% | 369.543ms | 369.543ms | 369.543ms | 369.543ms | 410.000ms | 240.97 MiB | none | 2.71K/s |
| q6 | count distinct, strings | 70.000ms | 414.995ms | 570.561ms | 0.0% | 570.561ms | 570.561ms | 570.561ms | 570.561ms | 440.000ms | 240.62 MiB | none | 1.75K/s |
| q7 | min and max of a date | 26.000ms | 942.667ms | 767.184ms | 0.0% | 767.184ms | 767.184ms | 767.184ms | 767.184ms | 2.820s | 240.38 MiB | none | 1.30K/s |
| q8 | group by, low card | 22.000ms | 494.165ms | 542.670ms | 0.0% | 542.670ms | 542.670ms | 542.670ms | 542.670ms | 720.000ms | 242.50 MiB | none | 1.84K/s |
| q9 | group by and count distinct | 35.000ms | 600.520ms | 498.418ms | 0.0% | 498.418ms | 498.418ms | 498.418ms | 498.418ms | 470.000ms | 242.62 MiB | none | 2.01K/s |
| q10 | group by, several aggregates | 56.000ms | 484.468ms | 772.631ms | 0.0% | 772.631ms | 772.631ms | 772.631ms | 772.631ms | 600.000ms | 243.35 MiB | none | 1.29K/s |
| q11 | group by a string and count distinct | 17.000ms | 698.117ms | 886.045ms | 0.0% | 886.045ms | 886.045ms | 886.045ms | 886.045ms | 2.260s | 243.71 MiB | none | 1.13K/s |
| q12 | group by two strings and count distinct | 18.000ms | 952.654ms | 561.110ms | 0.0% | 561.110ms | 561.110ms | 561.110ms | 561.110ms | 520.000ms | 243.38 MiB | none | 1.78K/s |
| q13 | group by a string and top k | 31.000ms | 505.873ms | 494.435ms | 0.0% | 494.435ms | 494.435ms | 494.435ms | 494.435ms | 660.000ms | 242.38 MiB | none | 2.02K/s |
| q14 | group by a string and count distinct | 17.000ms | 515.769ms | 475.242ms | 0.0% | 475.242ms | 475.242ms | 475.242ms | 475.242ms | 580.000ms | 244.00 MiB | none | 2.10K/s |
| q15 | group by two columns and top k | 48.000ms | 491.407ms | 415.158ms | 0.0% | 415.158ms | 415.158ms | 415.158ms | 415.158ms | 440.000ms | 243.38 MiB | none | 2.41K/s |
| q16 | group by, very high card | 36.000ms | 535.657ms | 783.821ms | 0.0% | 783.821ms | 783.821ms | 783.821ms | 783.821ms | 1.260s | 241.44 MiB | none | 1.28K/s |
| q17 | group by two, very high card | 19.000ms | 604.619ms | 491.249ms | 0.0% | 491.249ms | 491.249ms | 491.249ms | 491.249ms | 530.000ms | 242.88 MiB | none | 2.04K/s |
| q18 | group by two, no ordering | 16.000ms | 452.571ms | 549.661ms | 0.0% | 549.661ms | 549.661ms | 549.661ms | 549.661ms | 450.000ms | 242.38 MiB | none | 1.82K/s |
| q19 | group by with an extract | 25.000ms | 485.732ms | 527.471ms | 0.0% | 527.471ms | 527.471ms | 527.471ms | 527.471ms | 410.000ms | 243.14 MiB | none | 1.90K/s |
| q20 | point lookup | 15.000ms | 591.857ms | 549.769ms | 0.0% | 549.769ms | 549.769ms | 549.769ms | 549.769ms | 490.000ms | 239.75 MiB | none | 1.82K/s |
| q21 | substring scan | 14.000ms | 604.461ms | 510.021ms | 0.0% | 510.021ms | 510.021ms | 510.021ms | 510.021ms | 1.100s | 240.75 MiB | none | 1.96K/s |
| q22 | substring scan and group by | 67.000ms | 416.480ms | 494.056ms | 0.0% | 494.056ms | 494.056ms | 494.056ms | 494.056ms | 700.000ms | 243.88 MiB | none | 2.02K/s |
| q23 | two substring scans and group by | 20.000ms | 407.598ms | 468.365ms | 0.0% | 468.365ms | 468.365ms | 468.365ms | 468.365ms | 720.000ms | 243.00 MiB | none | 2.14K/s |
| q24 | select star and top k | 79.000ms | 427.099ms | 428.866ms | 0.0% | 428.866ms | 428.866ms | 428.866ms | 428.866ms | 470.000ms | 242.38 MiB | none | 2.33K/s |
| q25 | top k by a date | 36.000ms | 411.500ms | 475.562ms | 0.0% | 475.562ms | 475.562ms | 475.562ms | 475.562ms | 690.000ms | 241.38 MiB | none | 2.10K/s |
| q26 | top k by a string | 26.000ms | 358.780ms | 363.571ms | 0.0% | 363.571ms | 363.571ms | 363.571ms | 363.571ms | 380.000ms | 240.64 MiB | none | 2.75K/s |
| q27 | top k by two columns | 57.000ms | 507.579ms | 530.057ms | 0.0% | 530.057ms | 530.057ms | 530.057ms | 530.057ms | 780.000ms | 241.75 MiB | none | 1.89K/s |
| q28 | group by with a string length | 43.000ms | 538.987ms | 404.306ms | 0.0% | 404.306ms | 404.306ms | 404.306ms | 404.306ms | 490.000ms | 243.25 MiB | none | 2.47K/s |
| q29 | group by a regular expression | 22.000ms | 454.885ms | 371.071ms | 0.0% | 371.071ms | 371.071ms | 371.071ms | 371.071ms | 440.000ms | 244.62 MiB | none | 2.69K/s |
| q30 | ninety sums over one column | 44.000ms | 355.025ms | 614.703ms | 0.0% | 614.703ms | 614.703ms | 614.703ms | 614.703ms | 1.530s | 244.25 MiB | none | 1.63K/s |
| q31 | group by two and several aggregates | 90.000ms | 433.309ms | 882.060ms | 0.0% | 882.060ms | 882.060ms | 882.060ms | 882.060ms | 2.650s | 243.50 MiB | none | 1.13K/s |
| q32 | group by a high card pair | 18.000ms | 641.887ms | 454.828ms | 0.0% | 454.828ms | 454.828ms | 454.828ms | 454.828ms | 660.000ms | 244.00 MiB | none | 2.20K/s |
| q33 | group by a high card pair, unfiltered | 51.000ms | 1.079s | 797.146ms | 0.0% | 797.146ms | 797.146ms | 797.146ms | 797.146ms | 730.000ms | 242.12 MiB | none | 1.25K/s |
| q34 | group by a long string | 71.000ms | 849.739ms | 925.217ms | 0.0% | 925.217ms | 925.217ms | 925.217ms | 925.217ms | 730.000ms | 243.12 MiB | none | 1.08K/s |
| q35 | group by a constant and a long string | 79.000ms | 824.427ms | 691.598ms | 0.0% | 691.598ms | 691.598ms | 691.598ms | 691.598ms | 470.000ms | 243.50 MiB | none | 1.45K/s |
| q36 | group by four expressions | 79.000ms | 943.861ms | 1.333s | 0.0% | 1.333s | 1.333s | 1.333s | 1.333s | 680.000ms | 243.12 MiB | none | 750/s |
| q37 | date range and group by a URL | 178.000ms | 1.639s | 1.477s | 0.0% | 1.477s | 1.477s | 1.477s | 1.477s | 660.000ms | 245.00 MiB | none | 677/s |
| q38 | date range and group by a title | 148.000ms | 1.559s | 1.370s | 0.0% | 1.370s | 1.370s | 1.370s | 1.370s | 670.000ms | 244.75 MiB | none | 730/s |
| q39 | date range, group by and offset | 270.000ms | 1.022s | 1.126s | 0.0% | 1.126s | 1.126s | 1.126s | 1.126s | 810.000ms | 245.00 MiB | none | 888/s |
| q40 | date range, a case and a wide group by | 165.000ms | 920.705ms | 1.034s | 0.0% | 1.034s | 1.034s | 1.034s | 1.034s | 910.000ms | 245.25 MiB | none | 967/s |
| q41 | date range with an IN and a hash | 52.000ms | 945.171ms | 1.436s | 0.0% | 1.436s | 1.436s | 1.436s | 1.436s | 940.000ms | 246.75 MiB | none | 697/s |
| q42 | date range and a deep offset | 28.000ms | 1.103s | 928.684ms | 0.0% | 928.684ms | 928.684ms | 928.684ms | 928.684ms | 470.000ms | 245.00 MiB | none | 1.08K/s |
| q43 | minute buckets over a date range | 375.000ms | 999.596ms | 1.269s | 0.0% | 1.269s | 1.269s | 1.269s | 1.269s | 770.000ms | 243.38 MiB | none | 788/s |

clickhouse-local 26.9.1.1138 over 43 of 43 queries. Total 2.979s by its own clock and 30.403s by ours, 29.249s cold, 37.200s of CPU, peak 246.75 MiB, 14.43K/s and 3.19 MiB/s.

Running it cost 921% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 4.06x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 6.000ms | 104.964ms | 66.663ms | 0.0% | 66.663ms | 66.663ms | 66.663ms | 66.663ms | 70.000ms | 77.06 MiB | none | 15.00K/s |
| q2 | filtered count | 24.000ms | 107.826ms | 143.338ms | 0.0% | 143.338ms | 143.338ms | 143.338ms | 143.338ms | 180.000ms | 87.49 MiB | none | 6.98K/s |
| q3 | three aggregates | 7.000ms | 115.815ms | 79.488ms | 0.0% | 79.488ms | 79.488ms | 79.488ms | 79.488ms | 80.000ms | 78.94 MiB | none | 12.58K/s |
| q4 | average | 8.000ms | 81.244ms | 186.695ms | 0.0% | 186.695ms | 186.695ms | 186.695ms | 186.695ms | 170.000ms | 77.55 MiB | none | 5.36K/s |
| q5 | count distinct, high card | 16.000ms | 61.308ms | 60.656ms | 0.0% | 60.656ms | 60.656ms | 60.656ms | 60.656ms | 50.000ms | 86.89 MiB | none | 16.49K/s |
| q6 | count distinct, strings | 13.000ms | 230.631ms | 82.433ms | 0.0% | 82.433ms | 82.433ms | 82.433ms | 82.433ms | 60.000ms | 94.21 MiB | none | 12.13K/s |
| q7 | min and max of a date | 5.000ms | 69.811ms | 61.695ms | 0.0% | 61.695ms | 61.695ms | 61.695ms | 61.695ms | 50.000ms | 76.68 MiB | none | 16.21K/s |
| q8 | group by, low card | 15.000ms | 119.847ms | 120.615ms | 0.0% | 120.615ms | 120.615ms | 120.615ms | 120.615ms | 160.000ms | 89.04 MiB | none | 8.29K/s |
| q9 | group by and count distinct | 21.000ms | 120.553ms | 100.242ms | 0.0% | 100.242ms | 100.242ms | 100.242ms | 100.242ms | 100.000ms | 94.60 MiB | none | 9.98K/s |
| q10 | group by, several aggregates | 16.000ms | 104.242ms | 131.441ms | 0.0% | 131.441ms | 131.441ms | 131.441ms | 131.441ms | 130.000ms | 96.61 MiB | none | 7.61K/s |
| q11 | group by a string and count distinct | 35.000ms | 98.902ms | 117.853ms | 0.0% | 117.853ms | 117.853ms | 117.853ms | 117.853ms | 140.000ms | 107.81 MiB | none | 8.49K/s |
| q12 | group by two strings and count distinct | 22.000ms | 90.572ms | 91.811ms | 0.0% | 91.811ms | 91.811ms | 91.811ms | 91.811ms | 80.000ms | 98.40 MiB | none | 10.89K/s |
| q13 | group by a string and top k | 13.000ms | 90.422ms | 63.229ms | 0.0% | 63.229ms | 63.229ms | 63.229ms | 63.229ms | 50.000ms | 96.18 MiB | none | 15.82K/s |
| q14 | group by a string and count distinct | 16.000ms | 86.095ms | 73.715ms | 0.0% | 73.715ms | 73.715ms | 73.715ms | 73.715ms | 60.000ms | 114.16 MiB | none | 13.57K/s |
| q15 | group by two columns and top k | 19.000ms | 89.949ms | 97.900ms | 0.0% | 97.900ms | 97.900ms | 97.900ms | 97.900ms | 100.000ms | 98.52 MiB | none | 10.21K/s |
| q16 | group by, very high card | 10.000ms | 90.407ms | 74.725ms | 0.0% | 74.725ms | 74.725ms | 74.725ms | 74.725ms | 50.000ms | 88.66 MiB | none | 13.38K/s |
| q17 | group by two, very high card | 33.000ms | 76.231ms | 111.860ms | 0.0% | 111.860ms | 111.860ms | 111.860ms | 111.860ms | 130.000ms | 96.42 MiB | none | 8.94K/s |
| q18 | group by two, no ordering | 28.000ms | 128.095ms | 129.324ms | 0.0% | 129.324ms | 129.324ms | 129.324ms | 129.324ms | 160.000ms | 91.96 MiB | none | 7.73K/s |
| q19 | group by with an extract | 49.000ms | 265.465ms | 186.205ms | 0.0% | 186.205ms | 186.205ms | 186.205ms | 186.205ms | 160.000ms | 97.05 MiB | none | 5.37K/s |
| q20 | point lookup | 77.000ms | 298.035ms | 159.171ms | 0.0% | 159.171ms | 159.171ms | 159.171ms | 159.171ms | 150.000ms | 86.16 MiB | none | 6.28K/s |
| q21 | substring scan | 10.000ms | 57.930ms | 118.766ms | 0.0% | 118.766ms | 118.766ms | 118.766ms | 118.766ms | 140.000ms | 87.40 MiB | none | 8.42K/s |
| q22 | substring scan and group by | 36.000ms | 72.809ms | 418.367ms | 0.0% | 418.367ms | 418.367ms | 418.367ms | 418.367ms | 430.000ms | 95.75 MiB | none | 2.39K/s |
| q23 | two substring scans and group by | 58.000ms | 179.366ms | 170.852ms | 0.0% | 170.852ms | 170.852ms | 170.852ms | 170.852ms | 180.000ms | 98.05 MiB | none | 5.85K/s |
| q24 | select star and top k | 40.000ms | 94.216ms | 162.003ms | 0.0% | 162.003ms | 162.003ms | 162.003ms | 162.003ms | 130.000ms | 99.94 MiB | none | 6.17K/s |
| q25 | top k by a date | 66.000ms | 256.305ms | 430.454ms | 0.0% | 430.454ms | 430.454ms | 430.454ms | 430.454ms | 460.000ms | 92.43 MiB | none | 2.32K/s |
| q26 | top k by a string | 11.000ms | 102.528ms | 90.089ms | 0.0% | 90.089ms | 90.089ms | 90.089ms | 90.089ms | 100.000ms | 89.76 MiB | none | 11.10K/s |
| q27 | top k by two columns | 15.000ms | 73.417ms | 108.747ms | 0.0% | 108.747ms | 108.747ms | 108.747ms | 108.747ms | 100.000ms | 92.47 MiB | none | 9.20K/s |
| q28 | group by with a string length | 27.000ms | 108.971ms | 88.846ms | 0.0% | 88.846ms | 88.846ms | 88.846ms | 88.846ms | 80.000ms | 98.86 MiB | none | 11.26K/s |
| q29 | group by a regular expression | 88.000ms | 111.498ms | 151.935ms | 0.0% | 151.935ms | 151.935ms | 151.935ms | 151.935ms | 130.000ms | 106.33 MiB | none | 6.58K/s |
| q30 | ninety sums over one column | 41.000ms | 121.742ms | 107.837ms | 0.0% | 107.837ms | 107.837ms | 107.837ms | 107.837ms | 100.000ms | 85.03 MiB | none | 9.27K/s |
| q31 | group by two and several aggregates | 25.000ms | 124.383ms | 111.752ms | 0.0% | 111.752ms | 111.752ms | 111.752ms | 111.752ms | 100.000ms | 99.24 MiB | none | 8.95K/s |
| q32 | group by a high card pair | 23.000ms | 78.320ms | 82.862ms | 0.0% | 82.862ms | 82.862ms | 82.862ms | 82.862ms | 80.000ms | 99.16 MiB | none | 12.07K/s |
| q33 | group by a high card pair, unfiltered | 130.000ms | 165.888ms | 241.832ms | 0.0% | 241.832ms | 241.832ms | 241.832ms | 241.832ms | 340.000ms | 94.89 MiB | none | 4.14K/s |
| q34 | group by a long string | 18.000ms | 177.852ms | 104.268ms | 0.0% | 104.268ms | 104.268ms | 104.268ms | 104.268ms | 90.000ms | 102.10 MiB | none | 9.59K/s |
| q35 | group by a constant and a long string | 55.000ms | 120.858ms | 301.970ms | 0.0% | 301.970ms | 301.970ms | 301.970ms | 301.970ms | 220.000ms | 104.10 MiB | none | 3.31K/s |
| q36 | group by four expressions | 39.000ms | 97.264ms | 139.746ms | 0.0% | 139.746ms | 139.746ms | 139.746ms | 139.746ms | 100.000ms | 89.53 MiB | none | 7.16K/s |
| q37 | date range and group by a URL | 46.000ms | 146.080ms | 107.658ms | 0.0% | 107.658ms | 107.658ms | 107.658ms | 107.658ms | 80.000ms | 105.49 MiB | none | 9.29K/s |
| q38 | date range and group by a title | 27.000ms | 140.319ms | 146.416ms | 0.0% | 146.416ms | 146.416ms | 146.416ms | 146.416ms | 140.000ms | 99.29 MiB | none | 6.83K/s |
| q39 | date range, group by and offset | 21.000ms | 123.660ms | 93.337ms | 0.0% | 93.337ms | 93.337ms | 93.337ms | 93.337ms | 80.000ms | 98.21 MiB | none | 10.71K/s |
| q40 | date range, a case and a wide group by | 23.000ms | 104.704ms | 88.402ms | 0.0% | 88.402ms | 88.402ms | 88.402ms | 88.402ms | 80.000ms | 102.18 MiB | none | 11.31K/s |
| q41 | date range with an IN and a hash | 33.000ms | 187.650ms | 140.910ms | 0.0% | 140.910ms | 140.910ms | 140.910ms | 140.910ms | 120.000ms | 95.63 MiB | none | 7.10K/s |
| q42 | date range and a deep offset | 196.000ms | 119.594ms | 306.653ms | 0.0% | 306.653ms | 306.653ms | 306.653ms | 306.653ms | 250.000ms | 93.42 MiB | none | 3.26K/s |
| q43 | minute buckets over a date range | 47.000ms | 126.030ms | 190.067ms | 0.0% | 190.067ms | 190.067ms | 190.067ms | 190.067ms | 220.000ms | 93.52 MiB | none | 5.26K/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 1.508s by its own clock and 6.043s by ours, 5.322s cold, 5.880s of CPU, peak 114.16 MiB, 28.51K/s and 6.29 MiB/s.

Running it cost 301% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 7.10x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 135.414ms | 432.992ms | 583.374ms | 0.0% | 583.374ms | 583.374ms | 583.374ms | 583.374ms | 860.000ms | 56.60 MiB | none | 1.71K/s |
| q2 | filtered count | 23.290ms | 399.133ms | 410.294ms | 0.0% | 410.294ms | 410.294ms | 410.294ms | 410.294ms | 420.000ms | 61.23 MiB | none | 2.44K/s |
| q3 | three aggregates | 66.421ms | 496.866ms | 456.505ms | 0.0% | 456.505ms | 456.505ms | 456.505ms | 456.505ms | 490.000ms | 60.85 MiB | none | 2.19K/s |
| q4 | average | 21.220ms | 822.235ms | 435.924ms | 0.0% | 435.924ms | 435.924ms | 435.924ms | 435.924ms | 420.000ms | 58.36 MiB | none | 2.29K/s |
| q5 | count distinct, high card | 57.054ms | 567.211ms | 580.439ms | 0.0% | 580.439ms | 580.439ms | 580.439ms | 580.439ms | 690.000ms | 62.38 MiB | none | 1.72K/s |
| q6 | count distinct, strings | 161.458ms | 463.465ms | 545.395ms | 0.0% | 545.395ms | 545.395ms | 545.395ms | 545.395ms | 1.070s | 63.26 MiB | none | 1.83K/s |
| q7 | min and max of a date | 26.469ms | 456.224ms | 452.421ms | 0.0% | 452.421ms | 452.421ms | 452.421ms | 452.421ms | 470.000ms | 59.23 MiB | none | 2.21K/s |
| q8 | group by, low card | 95.968ms | 680.655ms | 875.782ms | 0.0% | 875.782ms | 875.782ms | 875.782ms | 875.782ms | 620.000ms | 64.74 MiB | none | 1.14K/s |
| q9 | group by and count distinct | 114.957ms | 878.922ms | 728.063ms | 0.0% | 728.063ms | 728.063ms | 728.063ms | 728.063ms | 550.000ms | 66.39 MiB | none | 1.37K/s |
| q10 | group by, several aggregates | 77.383ms | 1.010s | 880.725ms | 0.0% | 880.725ms | 880.725ms | 880.725ms | 880.725ms | 700.000ms | 68.23 MiB | none | 1.14K/s |
| q11 | group by a string and count distinct | 99.270ms | 882.062ms | 923.215ms | 0.0% | 923.215ms | 923.215ms | 923.215ms | 923.215ms | 620.000ms | 67.52 MiB | none | 1.08K/s |
| q12 | group by two strings and count distinct | 46.593ms | 978.298ms | 894.565ms | 0.0% | 894.565ms | 894.565ms | 894.565ms | 894.565ms | 850.000ms | 67.85 MiB | none | 1.12K/s |
| q13 | group by a string and top k | 51.495ms | 897.836ms | 706.239ms | 0.0% | 706.239ms | 706.239ms | 706.239ms | 706.239ms | 590.000ms | 65.09 MiB | none | 1.42K/s |
| q14 | group by a string and count distinct | 67.571ms | 1.048s | 697.478ms | 0.0% | 697.478ms | 697.478ms | 697.478ms | 697.478ms | 580.000ms | 67.45 MiB | none | 1.43K/s |
| q15 | group by two columns and top k | 49.685ms | 825.050ms | 624.012ms | 0.0% | 624.012ms | 624.012ms | 624.012ms | 624.012ms | 560.000ms | 65.68 MiB | none | 1.60K/s |
| q16 | group by, very high card | 112.829ms | 1.017s | 753.531ms | 0.0% | 753.531ms | 753.531ms | 753.531ms | 753.531ms | 770.000ms | 63.50 MiB | none | 1.33K/s |
| q17 | group by two, very high card | 101.044ms | 1.037s | 957.852ms | 0.0% | 957.852ms | 957.852ms | 957.852ms | 957.852ms | 730.000ms | 64.40 MiB | none | 1.04K/s |
| q18 | group by two, no ordering | 121.051ms | 1.092s | 740.485ms | 0.0% | 740.485ms | 740.485ms | 740.485ms | 740.485ms | 710.000ms | 62.87 MiB | none | 1.35K/s |
| q19 | group by with an extract | 96.179ms | 1.016s | 1.094s | 0.0% | 1.094s | 1.094s | 1.094s | 1.094s | 750.000ms | 65.96 MiB | none | 914/s |
| q20 | point lookup | 24.173ms | 797.349ms | 638.551ms | 0.0% | 638.551ms | 638.551ms | 638.551ms | 638.551ms | 590.000ms | 59.01 MiB | none | 1.57K/s |
| q21 | substring scan | 30.523ms | 878.823ms | 604.978ms | 0.0% | 604.978ms | 604.978ms | 604.978ms | 604.978ms | 580.000ms | 62.53 MiB | none | 1.65K/s |
| q22 | substring scan and group by | 75.193ms | 574.112ms | 736.976ms | 0.0% | 736.976ms | 736.976ms | 736.976ms | 736.976ms | 630.000ms | 66.69 MiB | none | 1.36K/s |
| q23 | two substring scans and group by | 98.360ms | 631.565ms | 717.155ms | 0.0% | 717.155ms | 717.155ms | 717.155ms | 717.155ms | 640.000ms | 69.74 MiB | none | 1.39K/s |
| q24 | select star and top k | 34.685ms | 561.744ms | 440.052ms | 0.0% | 440.052ms | 440.052ms | 440.052ms | 440.052ms | 430.000ms | 65.33 MiB | none | 2.27K/s |
| q25 | top k by a date | 85.051ms | 611.412ms | 698.712ms | 0.0% | 698.712ms | 698.712ms | 698.712ms | 698.712ms | 690.000ms | 63.73 MiB | none | 1.43K/s |
| q26 | top k by a string | 90.070ms | 545.016ms | 615.401ms | 0.0% | 615.401ms | 615.401ms | 615.401ms | 615.401ms | 780.000ms | 62.48 MiB | none | 1.62K/s |
| q27 | top k by two columns | 25.478ms | 474.097ms | 417.360ms | 0.0% | 417.360ms | 417.360ms | 417.360ms | 417.360ms | 420.000ms | 63.57 MiB | none | 2.40K/s |
| q30 | ninety sums over one column | 36.377ms | 421.144ms | 458.580ms | 0.0% | 458.580ms | 458.580ms | 458.580ms | 458.580ms | 470.000ms | 62.66 MiB | none | 2.18K/s |
| q31 | group by two and several aggregates | 109.810ms | 478.498ms | 706.434ms | 0.0% | 706.434ms | 706.434ms | 706.434ms | 706.434ms | 880.000ms | 67.36 MiB | none | 1.42K/s |
| q32 | group by a high card pair | 51.487ms | 542.750ms | 506.707ms | 0.0% | 506.707ms | 506.707ms | 506.707ms | 506.707ms | 510.000ms | 67.61 MiB | none | 1.97K/s |
| q33 | group by a high card pair, unfiltered | 81.260ms | 504.815ms | 590.825ms | 0.0% | 590.825ms | 590.825ms | 590.825ms | 590.825ms | 620.000ms | 66.39 MiB | none | 1.69K/s |
| q34 | group by a long string | 67.800ms | 540.108ms | 564.306ms | 0.0% | 564.306ms | 564.306ms | 564.306ms | 564.306ms | 660.000ms | 64.61 MiB | none | 1.77K/s |
| q35 | group by a constant and a long string | 44.601ms | 557.637ms | 486.212ms | 0.0% | 486.212ms | 486.212ms | 486.212ms | 486.212ms | 480.000ms | 64.73 MiB | none | 2.06K/s |
| q37 | date range and group by a URL | 50.205ms | 486.114ms | 516.912ms | 0.0% | 516.912ms | 516.912ms | 516.912ms | 516.912ms | 600.000ms | 68.98 MiB | none | 1.93K/s |
| q38 | date range and group by a title | 213.317ms | 503.004ms | 606.383ms | 0.0% | 606.383ms | 606.383ms | 606.383ms | 606.383ms | 1.290s | 69.07 MiB | none | 1.65K/s |
| q39 | date range, group by and offset | 60.800ms | 456.729ms | 493.458ms | 0.0% | 493.458ms | 493.458ms | 493.458ms | 493.458ms | 530.000ms | 66.36 MiB | none | 2.03K/s |
| q40 | date range, a case and a wide group by | 73.534ms | 521.418ms | 490.764ms | 0.0% | 490.764ms | 490.764ms | 490.764ms | 490.764ms | 540.000ms | 68.86 MiB | none | 2.04K/s |
| q41 | date range with an IN and a hash | 146.779ms | 585.231ms | 628.787ms | 0.0% | 628.787ms | 628.787ms | 628.787ms | 628.787ms | 960.000ms | 69.68 MiB | none | 1.59K/s |
| q42 | date range and a deep offset | 283.141ms | 578.179ms | 860.796ms | 0.0% | 860.796ms | 860.796ms | 860.796ms | 860.796ms | 1.170s | 68.10 MiB | none | 1.16K/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 3.208s by its own clock and 25.119s by ours, 26.251s cold, 25.920s of CPU, peak 69.74 MiB, 12.16K/s and 2.68 MiB/s.

Running it cost 683% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.67x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-server in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 8.000ms | 582.055ms | 487.151ms | 0.0% | 487.151ms | 487.151ms | 487.151ms | 487.151ms | not read | not read | not read | 2.05K/s |
| q2 | filtered count | 23.000ms | 521.941ms | 791.675ms | 0.0% | 791.675ms | 791.675ms | 791.675ms | 791.675ms | not read | not read | not read | 1.26K/s |
| q3 | three aggregates | 55.000ms | 614.725ms | 790.879ms | 0.0% | 790.879ms | 790.879ms | 790.879ms | 790.879ms | not read | not read | not read | 1.26K/s |
| q4 | average | 7.000ms | 593.443ms | 555.224ms | 0.0% | 555.224ms | 555.224ms | 555.224ms | 555.224ms | not read | not read | not read | 1.80K/s |
| q5 | count distinct, high card | 7.000ms | 724.975ms | 323.105ms | 0.0% | 323.105ms | 323.105ms | 323.105ms | 323.105ms | not read | not read | not read | 3.09K/s |
| q6 | count distinct, strings | 8.000ms | 496.215ms | 611.501ms | 0.0% | 611.501ms | 611.501ms | 611.501ms | 611.501ms | not read | not read | not read | 1.64K/s |
| q7 | min and max of a date | 36.000ms | 465.447ms | 466.255ms | 0.0% | 466.255ms | 466.255ms | 466.255ms | 466.255ms | not read | not read | not read | 2.14K/s |
| q8 | group by, low card | 8.000ms | 804.714ms | 822.253ms | 0.0% | 822.253ms | 822.253ms | 822.253ms | 822.253ms | not read | not read | not read | 1.22K/s |
| q9 | group by and count distinct | 5.000ms | 508.750ms | 378.321ms | 0.0% | 378.321ms | 378.321ms | 378.321ms | 378.321ms | not read | not read | not read | 2.64K/s |
| q10 | group by, several aggregates | 97.000ms | 388.320ms | 552.202ms | 0.0% | 552.202ms | 552.202ms | 552.202ms | 552.202ms | not read | not read | not read | 1.81K/s |
| q11 | group by a string and count distinct | 48.000ms | 314.718ms | 296.767ms | 0.0% | 296.767ms | 296.767ms | 296.767ms | 296.767ms | not read | not read | not read | 3.37K/s |
| q12 | group by two strings and count distinct | 63.000ms | 238.698ms | 338.516ms | 0.0% | 338.516ms | 338.516ms | 338.516ms | 338.516ms | not read | not read | not read | 2.95K/s |
| q13 | group by a string and top k | 15.000ms | 182.782ms | 654.977ms | 0.0% | 654.977ms | 654.977ms | 654.977ms | 654.977ms | not read | not read | not read | 1.53K/s |
| q14 | group by a string and count distinct | 9.000ms | 354.616ms | 211.736ms | 0.0% | 211.736ms | 211.736ms | 211.736ms | 211.736ms | not read | not read | not read | 4.72K/s |
| q15 | group by two columns and top k | 6.000ms | 257.117ms | 183.251ms | 0.0% | 183.251ms | 183.251ms | 183.251ms | 183.251ms | not read | not read | not read | 5.46K/s |
| q16 | group by, very high card | 6.000ms | 206.294ms | 169.382ms | 0.0% | 169.382ms | 169.382ms | 169.382ms | 169.382ms | not read | not read | not read | 5.90K/s |
| q17 | group by two, very high card | 6.000ms | 170.665ms | 178.292ms | 0.0% | 178.292ms | 178.292ms | 178.292ms | 178.292ms | not read | not read | not read | 5.61K/s |
| q18 | group by two, no ordering | 12.000ms | 393.134ms | 392.601ms | 0.0% | 392.601ms | 392.601ms | 392.601ms | 392.601ms | not read | not read | not read | 2.55K/s |
| q19 | group by with an extract | 16.000ms | 229.137ms | 235.376ms | 0.0% | 235.376ms | 235.376ms | 235.376ms | 235.376ms | not read | not read | not read | 4.25K/s |
| q20 | point lookup | 12.000ms | 206.934ms | 293.629ms | 0.0% | 293.629ms | 293.629ms | 293.629ms | 293.629ms | not read | not read | not read | 3.41K/s |
| q21 | substring scan | 5.000ms | 258.192ms | 187.038ms | 0.0% | 187.038ms | 187.038ms | 187.038ms | 187.038ms | not read | not read | not read | 5.35K/s |
| q22 | substring scan and group by | 23.000ms | 419.217ms | 265.190ms | 0.0% | 265.190ms | 265.190ms | 265.190ms | 265.190ms | not read | not read | not read | 3.77K/s |
| q23 | two substring scans and group by | 10.000ms | 269.098ms | 190.808ms | 0.0% | 190.808ms | 190.808ms | 190.808ms | 190.808ms | not read | not read | not read | 5.24K/s |
| q24 | select star and top k | 18.000ms | 214.928ms | 189.579ms | 0.0% | 189.579ms | 189.579ms | 189.579ms | 189.579ms | not read | not read | not read | 5.27K/s |
| q25 | top k by a date | 5.000ms | 308.667ms | 190.260ms | 0.0% | 190.260ms | 190.260ms | 190.260ms | 190.260ms | not read | not read | not read | 5.26K/s |
| q26 | top k by a string | 10.000ms | 196.064ms | 315.151ms | 0.0% | 315.151ms | 315.151ms | 315.151ms | 315.151ms | not read | not read | not read | 3.17K/s |
| q27 | top k by two columns | 5.000ms | 194.649ms | 215.024ms | 0.0% | 215.024ms | 215.024ms | 215.024ms | 215.024ms | not read | not read | not read | 4.65K/s |
| q28 | group by with a string length | 7.000ms | 173.160ms | 178.342ms | 0.0% | 178.342ms | 178.342ms | 178.342ms | 178.342ms | not read | not read | not read | 5.61K/s |
| q29 | group by a regular expression | 95.000ms | 218.488ms | 342.569ms | 0.0% | 342.569ms | 342.569ms | 342.569ms | 342.569ms | not read | not read | not read | 2.92K/s |
| q30 | ninety sums over one column | 23.000ms | 249.391ms | 197.143ms | 0.0% | 197.143ms | 197.143ms | 197.143ms | 197.143ms | not read | not read | not read | 5.07K/s |
| q31 | group by two and several aggregates | 6.000ms | 257.381ms | 223.705ms | 0.0% | 223.705ms | 223.705ms | 223.705ms | 223.705ms | not read | not read | not read | 4.47K/s |
| q32 | group by a high card pair | 42.000ms | 211.094ms | 256.175ms | 0.0% | 256.175ms | 256.175ms | 256.175ms | 256.175ms | not read | not read | not read | 3.90K/s |
| q33 | group by a high card pair, unfiltered | 10.000ms | 188.288ms | 194.947ms | 0.0% | 194.947ms | 194.947ms | 194.947ms | 194.947ms | not read | not read | not read | 5.13K/s |
| q34 | group by a long string | 11.000ms | 173.859ms | 292.287ms | 0.0% | 292.287ms | 292.287ms | 292.287ms | 292.287ms | not read | not read | not read | 3.42K/s |
| q35 | group by a constant and a long string | 8.000ms | 199.801ms | 196.125ms | 0.0% | 196.125ms | 196.125ms | 196.125ms | 196.125ms | not read | not read | not read | 5.10K/s |
| q36 | group by four expressions | 10.000ms | 212.401ms | 226.390ms | 0.0% | 226.390ms | 226.390ms | 226.390ms | 226.390ms | not read | not read | not read | 4.42K/s |
| q37 | date range and group by a URL | 14.000ms | 193.933ms | 239.147ms | 0.0% | 239.147ms | 239.147ms | 239.147ms | 239.147ms | not read | not read | not read | 4.18K/s |
| q38 | date range and group by a title | 15.000ms | 265.964ms | 286.842ms | 0.0% | 286.842ms | 286.842ms | 286.842ms | 286.842ms | not read | not read | not read | 3.49K/s |
| q39 | date range, group by and offset | 11.000ms | 210.459ms | 186.170ms | 0.0% | 186.170ms | 186.170ms | 186.170ms | 186.170ms | not read | not read | not read | 5.37K/s |
| q40 | date range, a case and a wide group by | 18.000ms | 194.561ms | 208.879ms | 0.0% | 208.879ms | 208.879ms | 208.879ms | 208.879ms | not read | not read | not read | 4.79K/s |
| q41 | date range with an IN and a hash | 37.000ms | 196.127ms | 293.170ms | 0.0% | 293.170ms | 293.170ms | 293.170ms | 293.170ms | not read | not read | not read | 3.41K/s |
| q42 | date range and a deep offset | 14.000ms | 207.460ms | 230.039ms | 0.0% | 230.039ms | 230.039ms | 230.039ms | 230.039ms | not read | not read | not read | 4.35K/s |
| q43 | minute buckets over a date range | 10.000ms | 228.458ms | 208.729ms | 0.0% | 208.729ms | 208.729ms | 208.729ms | 208.729ms | not read | not read | not read | 4.79K/s |

clickhouse-server 26.9.1.1138 over 43 of 43 queries. Total 854.000ms by its own clock and 14.047s by ours, 13.496s cold, no reading of CPU, peak not read, 50.35K/s and 11.11 MiB/s.

Running it cost 1545% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 4.85x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 13.943ms | 10.325ms | 0.0% | 10.325ms | 10.325ms | 10.325ms | 10.325ms | 0.000us | 4.00 MiB | none | 96.85K/s |
| q2 | filtered count | 1.000ms | 23.330ms | 27.023ms | 0.0% | 27.023ms | 27.023ms | 27.023ms | 27.023ms | 0.000us | 4.25 MiB | none | 37.01K/s |
| q3 | three aggregates | 1.000ms | 26.447ms | 11.148ms | 0.0% | 11.148ms | 11.148ms | 11.148ms | 11.148ms | 0.000us | 4.12 MiB | none | 89.70K/s |
| q4 | average | 27.000ms | 17.495ms | 138.955ms | 0.0% | 138.955ms | 138.955ms | 138.955ms | 138.955ms | 80.000ms | 4.00 MiB | none | 7.20K/s |
| q5 | count distinct, high card | 1.000ms | 37.003ms | 21.981ms | 0.0% | 21.981ms | 21.981ms | 21.981ms | 21.981ms | 0.000us | 4.12 MiB | none | 45.49K/s |
| q6 | count distinct, strings | 1.000ms | 74.495ms | 10.484ms | 0.0% | 10.484ms | 10.484ms | 10.484ms | 10.484ms | 0.000us | 4.12 MiB | none | 95.38K/s |
| q7 | min and max of a date | 1.000ms | 24.329ms | 11.095ms | 0.0% | 11.095ms | 11.095ms | 11.095ms | 11.095ms | 0.000us | 4.12 MiB | none | 90.13K/s |
| q8 | group by, low card | 1.000ms | 21.799ms | 13.897ms | 0.0% | 13.897ms | 13.897ms | 13.897ms | 13.897ms | 0.000us | 4.12 MiB | none | 71.96K/s |
| q9 | group by and count distinct | 2.000ms | 11.946ms | 12.018ms | 0.0% | 12.018ms | 12.018ms | 12.018ms | 12.018ms | 0.000us | 4.38 MiB | none | 83.21K/s |
| q10 | group by, several aggregates | 2.000ms | 8.472ms | 16.557ms | 0.0% | 16.557ms | 16.557ms | 16.557ms | 16.557ms | 0.000us | 4.50 MiB | none | 60.40K/s |
| q11 | group by a string and count distinct | 1.000ms | 10.369ms | 30.377ms | 0.0% | 30.377ms | 30.377ms | 30.377ms | 30.377ms | 10.000ms | 4.25 MiB | none | 32.92K/s |
| q12 | group by two strings and count distinct | 1.000ms | 8.744ms | 9.389ms | 0.0% | 9.389ms | 9.389ms | 9.389ms | 9.389ms | 0.000us | 4.25 MiB | none | 106.51K/s |
| q13 | group by a string and top k | 2.000ms | 12.795ms | 19.747ms | 0.0% | 19.747ms | 19.747ms | 19.747ms | 19.747ms | 0.000us | 4.25 MiB | none | 50.64K/s |
| q14 | group by a string and count distinct | 2.000ms | 17.599ms | 20.411ms | 0.0% | 20.411ms | 20.411ms | 20.411ms | 20.411ms | 0.000us | 4.38 MiB | none | 48.99K/s |
| q15 | group by two columns and top k | 2.000ms | 42.396ms | 25.859ms | 0.0% | 25.859ms | 25.859ms | 25.859ms | 25.859ms | 0.000us | 4.38 MiB | none | 38.67K/s |
| q16 | group by, very high card | 2.000ms | 9.296ms | 9.394ms | 0.0% | 9.394ms | 9.394ms | 9.394ms | 9.394ms | 0.000us | 4.38 MiB | none | 106.45K/s |
| q17 | group by two, very high card | 3.000ms | 13.800ms | 13.885ms | 0.0% | 13.885ms | 13.885ms | 13.885ms | 13.885ms | 0.000us | 4.62 MiB | none | 72.02K/s |
| q18 | group by two, no ordering | 3.000ms | 9.115ms | 10.645ms | 0.0% | 10.645ms | 10.645ms | 10.645ms | 10.645ms | 0.000us | 4.38 MiB | none | 93.94K/s |
| q20 | point lookup | 1.000ms | 12.617ms | 8.712ms | 0.0% | 8.712ms | 8.712ms | 8.712ms | 8.712ms | 0.000us | 4.12 MiB | none | 114.78K/s |
| q21 | substring scan | 3.000ms | 7.794ms | 9.675ms | 0.0% | 9.675ms | 9.675ms | 9.675ms | 9.675ms | 0.000us | 4.38 MiB | none | 103.36K/s |
| q22 | substring scan and group by | 3.000ms | 41.361ms | 12.378ms | 0.0% | 12.378ms | 12.378ms | 12.378ms | 12.378ms | 0.000us | 4.38 MiB | none | 80.79K/s |
| q23 | two substring scans and group by | 6.000ms | 17.887ms | 31.770ms | 0.0% | 31.770ms | 31.770ms | 31.770ms | 31.770ms | 20.000ms | 4.62 MiB | none | 31.48K/s |
| q24 | select star and top k | 13.000ms | 47.608ms | 22.983ms | 0.0% | 22.983ms | 22.983ms | 22.983ms | 22.983ms | 10.000ms | 5.75 MiB | none | 43.51K/s |
| q25 | top k by a date | 17.000ms | 17.208ms | 44.211ms | 0.0% | 44.211ms | 44.211ms | 44.211ms | 44.211ms | 30.000ms | 4.38 MiB | none | 22.62K/s |
| q26 | top k by a string | 1.000ms | 11.889ms | 9.745ms | 0.0% | 9.745ms | 9.745ms | 9.745ms | 9.745ms | 0.000us | 4.38 MiB | none | 102.62K/s |
| q27 | top k by two columns | 2.000ms | 6.924ms | 7.879ms | 0.0% | 7.879ms | 7.879ms | 7.879ms | 7.879ms | 0.000us | 4.38 MiB | none | 126.92K/s |
| q28 | group by with a string length | 3.000ms | 12.153ms | 9.010ms | 0.0% | 9.010ms | 9.010ms | 9.010ms | 9.010ms | 0.000us | 4.50 MiB | none | 110.99K/s |
| q29 | group by a regular expression | 5.000ms | 39.235ms | 14.749ms | 0.0% | 14.749ms | 14.749ms | 14.749ms | 14.749ms | 0.000us | 4.62 MiB | none | 67.80K/s |
| q30 | ninety sums over one column | 5.000ms | 18.558ms | 17.707ms | 0.0% | 17.707ms | 17.707ms | 17.707ms | 17.707ms | 0.000us | 4.88 MiB | none | 56.47K/s |
| q31 | group by two and several aggregates | 2.000ms | 10.035ms | 17.606ms | 0.0% | 17.606ms | 17.606ms | 17.606ms | 17.606ms | 0.000us | 4.25 MiB | none | 56.80K/s |
| q32 | group by a high card pair | 3.000ms | 7.344ms | 21.126ms | 0.0% | 21.126ms | 21.126ms | 21.126ms | 21.126ms | 10.000ms | 4.25 MiB | none | 47.34K/s |
| q34 | group by a long string | 4.000ms | 12.451ms | 19.047ms | 0.0% | 19.047ms | 19.047ms | 19.047ms | 19.047ms | 0.000us | 4.50 MiB | none | 52.50K/s |
| q35 | group by a constant and a long string | 9.000ms | 28.919ms | 17.168ms | 0.0% | 17.168ms | 17.168ms | 17.168ms | 17.168ms | 0.000us | 4.75 MiB | none | 58.25K/s |
| q36 | group by four expressions | 3.000ms | 17.425ms | 15.512ms | 0.0% | 15.512ms | 15.512ms | 15.512ms | 15.512ms | 0.000us | 4.75 MiB | none | 64.47K/s |
| q37 | date range and group by a URL | 3.000ms | 10.523ms | 12.955ms | 0.0% | 12.955ms | 12.955ms | 12.955ms | 12.955ms | 0.000us | 4.50 MiB | none | 77.19K/s |
| q38 | date range and group by a title | 3.000ms | 19.306ms | 9.416ms | 0.0% | 9.416ms | 9.416ms | 9.416ms | 9.416ms | 0.000us | 4.62 MiB | none | 106.20K/s |
| q39 | date range, group by and offset | 2.000ms | 19.743ms | 9.399ms | 0.0% | 9.399ms | 9.399ms | 9.399ms | 9.399ms | 0.000us | 4.62 MiB | none | 106.39K/s |
| q40 | date range, a case and a wide group by | 4.000ms | 18.394ms | 19.834ms | 0.0% | 19.834ms | 19.834ms | 19.834ms | 19.834ms | 0.000us | 4.75 MiB | none | 50.42K/s |
| q41 | date range with an IN and a hash | 1.000ms | 18.938ms | 11.808ms | 0.0% | 11.808ms | 11.808ms | 11.808ms | 11.808ms | 0.000us | 4.25 MiB | none | 84.69K/s |
| q42 | date range and a deep offset | 2.000ms | 38.978ms | 45.232ms | 0.0% | 45.232ms | 45.232ms | 45.232ms | 45.232ms | 0.000us | 4.38 MiB | none | 22.11K/s |
| q43 | minute buckets over a date range | 2.000ms | 6.967ms | 7.049ms | 0.0% | 7.049ms | 7.049ms | 7.049ms | 7.049ms | 0.000us | 4.50 MiB | none | 141.86K/s |

rudb rudb 0.2.28 over 41 of 43 queries. Total 151.000ms by its own clock and 808.161ms by ours, 825.630ms cold, 160.000ms of CPU, peak 5.75 MiB, 271.52K/s and 59.91 MiB/s.

Running it cost 435% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 19.71x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 1000 rows, one out of every 99998 of the 99997497 in the full file, which is a development loop rather than the suite
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

Answers differ, so this is not a comparison: q34: clickhouse-local does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q34: datafusion does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q34: polars does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q34: clickhouse-server does not agree with duckdb: 10 numbers against 10

These were answered differently and the data does not say which is right:

- q4: AVG(UserID) over a hundred million bigints near 10^18, where the engines disagree for two reasons. The order the partial sums are added in moves the floating point ones further apart than the one part in a billion this harness calls the same number, and DuckDB is plainly wrong: it sums the bigint column short by a multiple of 2^64 on this file, where its own hugeint and decimal paths, rudb, and adding the column up outside a database all agree. tamnd/rudb-compat#12.
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

Hot here means page cache warm and not buffer pool warm for every row but clickhouse-server, which stayed up across the whole suite with its own caches warm. That is the stronger kind of hot, so a ratio against that column is a ratio between two different quantities.

