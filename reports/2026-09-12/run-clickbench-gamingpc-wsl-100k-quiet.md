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
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 585.881ms | 660.000ms | 31.01 MiB | its own database file | its own | 3.63 to 3.66 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 699.989ms | 840.000ms | 28.76 MiB | its own database file | its own | 3.66 to 3.66 |
| clickhouse-local | 26.9.1.1162 | ran | 339.765ms | 320.000ms | 24.32 MiB | its own MergeTree parts, as system.parts counts the active ones | its own | 3.51 to 3.55 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 15.11 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 3.61 to 3.61 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 15.11 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 3.61 to 3.51 |
| clickhouse-server | 26.9.1.1162 | ran | 547.183ms | not read | 23.93 MiB | its own MergeTree parts, sorted by hits by (CounterID, EventDate, UserID, EventTime, WatchID), as system.parts counts the active ones | its own | 3.55 to 3.51 |
| rudb | rudb 0.2.31 | ran | 0.000us | 0.000us | 15.11 MiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 3.66 to 3.61 |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 314.000ms | 739.569ms | +136% | 741.458ms | 720.000ms | 0.97 | 65.04 MiB | none | 13.69M/s | 2.02 GiB/s | 1.00x |
| duckdb-pinned | 342.000ms | 1.318s | +285% | 1.329s | 1.270s | 0.96 | 81.91 MiB | none | 12.57M/s | 1.86 GiB/s | 1.09x |
| clickhouse-local | 432.000ms | 2.543s | +489% | 2.560s | 2.570s | 1.01 | 246.31 MiB | none | 9.95M/s | 1.47 GiB/s | 1.38x |
| datafusion | 589.000ms | 1.101s | +87% | 1.108s | 2.530s | 2.30 | 321.23 MiB | none | 7.30M/s | 1.08 GiB/s | 1.88x |
| polars | 784.821ms | 4.318s | +450% | 4.377s | 5.920s | 1.37 | 137.41 MiB | none | 4.97M/s | 751.06 MiB/s | 2.50x |
| clickhouse-server | 246.000ms | 1.543s | +527% | 1.532s | not read | not read | not read | not read | 17.48M/s | 2.58 GiB/s | 0.78x |
| rudb | 1.117s | 1.186s | +6% | 1.202s | 850.000ms | 0.72 | 158.62 MiB | none | 3.67M/s | 554.77 MiB/s | 3.56x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | clickhouse-server | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 1.000ms | 2.000ms | 1.000ms | 5.837ms | 1.000ms | 0.000us |
| q2 | filtered count | 1.000ms | 1.000ms | 2.000ms | 4.000ms | 7.109ms | 1.000ms | 1.000ms |
| q3 | three aggregates | 1.000ms | 1.000ms | 3.000ms | 5.000ms | 6.176ms | 2.000ms | 2.000ms |
| q4 | average | 1.000ms | 1.000ms | 3.000ms | 4.000ms | 7.423ms | 2.000ms | 2.000ms |
| q5 | count distinct, high card | 5.000ms | 5.000ms | 7.000ms | 8.000ms | 14.503ms | 6.000ms | 13.000ms |
| q6 | count distinct, strings | 5.000ms | 3.000ms | 5.000ms | 10.000ms | 13.965ms | 3.000ms | 9.000ms |
| q7 | min and max of a date | 1.000ms | 1.000ms | 3.000ms | 1.000ms | 5.776ms | 1.000ms | 1.000ms |
| q8 | group by, low card | 1.000ms | 6.000ms | 4.000ms | 5.000ms | 15.737ms | 2.000ms | 1.000ms |
| q9 | group by and count distinct | 8.000ms | 6.000ms | 6.000ms | 13.000ms | 24.419ms | 4.000ms | 18.000ms |
| q10 | group by, several aggregates | 9.000ms | 9.000ms | 6.000ms | 10.000ms | 28.214ms | 19.000ms | 21.000ms |
| q11 | group by a string and count distinct | 5.000ms | 5.000ms | 4.000ms | 9.000ms | 21.516ms | 2.000ms | 4.000ms |
| q12 | group by two strings and count distinct | 4.000ms | 5.000ms | 4.000ms | 10.000ms | 23.036ms | 3.000ms | 4.000ms |
| q13 | group by a string and top k | 5.000ms | 4.000ms | 6.000ms | 11.000ms | 17.955ms | 4.000ms | 15.000ms |
| q14 | group by a string and count distinct | 6.000ms | 7.000ms | 7.000ms | 16.000ms | 23.829ms | 5.000ms | 15.000ms |
| q15 | group by two columns and top k | 4.000ms | 5.000ms | 6.000ms | 15.000ms | 19.628ms | 4.000ms | 14.000ms |
| q16 | group by, very high card | 7.000ms | 5.000ms | 6.000ms | 10.000ms | 18.761ms | 4.000ms | 21.000ms |
| q17 | group by two, very high card | 11.000ms | 11.000ms | 13.000ms | 14.000ms | 24.096ms | 9.000ms | 35.000ms |
| q18 | group by two, no ordering | 13.000ms | 11.000ms | 6.000ms | 14.000ms | 16.357ms | 4.000ms | 26.000ms |
| q19 | group by with an extract | 11.000ms | 11.000ms | 14.000ms | 16.000ms | 26.708ms | 9.000ms | no dialect |
| q20 | point lookup | 1.000ms | 1.000ms | 4.000ms | 4.000ms | 5.434ms | 1.000ms | 2.000ms |
| q21 | substring scan | 8.000ms | 9.000ms | 9.000ms | 12.000ms | 24.257ms | 5.000ms | 35.000ms |
| q22 | substring scan and group by | 8.000ms | 10.000ms | 10.000ms | 16.000ms | 31.164ms | 3.000ms | 39.000ms |
| q23 | two substring scans and group by | 12.000ms | 12.000ms | 12.000ms | 34.000ms | 42.434ms | 5.000ms | 86.000ms |
| q24 | select star and top k | 25.000ms | 29.000ms | 116.000ms | 63.000ms | 57.852ms | 27.000ms | 197.000ms |
| q25 | top k by a date | 4.000ms | 3.000ms | 6.000ms | 7.000ms | 12.943ms | 2.000ms | 13.000ms |
| q26 | top k by a string | 2.000ms | 2.000ms | 4.000ms | 7.000ms | 11.481ms | 2.000ms | 11.000ms |
| q27 | top k by two columns | 3.000ms | 3.000ms | 5.000ms | 9.000ms | 14.171ms | 2.000ms | 14.000ms |
| q28 | group by with a string length | 9.000ms | 11.000ms | 5.000ms | 16.000ms | no dialect | 3.000ms | 35.000ms |
| q29 | group by a regular expression | 53.000ms | 55.000ms | 27.000ms | 36.000ms | no dialect | 24.000ms | 88.000ms |
| q30 | ninety sums over one column | 4.000ms | 14.000ms | 8.000ms | 12.000ms | 9.853ms | 6.000ms | 14.000ms |
| q31 | group by two and several aggregates | 4.000ms | 6.000ms | 6.000ms | 11.000ms | 21.820ms | 3.000ms | 16.000ms |
| q32 | group by a high card pair | 5.000ms | 5.000ms | 6.000ms | 11.000ms | 16.401ms | 17.000ms | 15.000ms |
| q33 | group by a high card pair, unfiltered | 9.000ms | 9.000ms | 11.000ms | 13.000ms | 18.966ms | 7.000ms | no dialect |
| q34 | group by a long string | 19.000ms | 17.000ms | 20.000ms | 24.000ms | 29.656ms | 13.000ms | 61.000ms |
| q35 | group by a constant and a long string | 19.000ms | 19.000ms | 21.000ms | 21.000ms | 32.224ms | 12.000ms | 66.000ms |
| q36 | group by four expressions | 8.000ms | 5.000ms | 6.000ms | 10.000ms | no dialect | 5.000ms | 33.000ms |
| q37 | date range and group by a URL | 3.000ms | 5.000ms | 8.000ms | 15.000ms | 27.863ms | 3.000ms | 33.000ms |
| q38 | date range and group by a title | 3.000ms | 4.000ms | 7.000ms | 25.000ms | 30.578ms | 4.000ms | 39.000ms |
| q39 | date range, group by and offset | 3.000ms | 4.000ms | 8.000ms | 17.000ms | 21.983ms | 3.000ms | 35.000ms |
| q40 | date range, a case and a wide group by | 4.000ms | 6.000ms | 9.000ms | 25.000ms | 22.746ms | 5.000ms | 62.000ms |
| q41 | date range with an IN and a hash | 3.000ms | 4.000ms | 6.000ms | 8.000ms | 15.824ms | 3.000ms | 7.000ms |
| q42 | date range and a deep offset | 3.000ms | 7.000ms | 6.000ms | 9.000ms | 16.126ms | 3.000ms | 8.000ms |
| q43 | minute buckets over a date range | 3.000ms | 4.000ms | 5.000ms | 8.000ms | no dialect | 3.000ms | 6.000ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 9.865ms | 10.602ms | 0.0% | 10.602ms | 10.602ms | 10.602ms | 10.602ms | 10.000ms | 26.79 MiB | none | 9.43M/s |
| q2 | filtered count | 1.000ms | 10.291ms | 11.016ms | 0.0% | 11.016ms | 11.016ms | 11.016ms | 11.016ms | 0.000us | 28.04 MiB | none | 9.08M/s |
| q3 | three aggregates | 1.000ms | 11.072ms | 10.259ms | 0.0% | 10.259ms | 10.259ms | 10.259ms | 10.259ms | 10.000ms | 28.29 MiB | none | 9.75M/s |
| q4 | average | 1.000ms | 10.950ms | 10.726ms | 0.0% | 10.726ms | 10.726ms | 10.726ms | 10.726ms | 0.000us | 28.54 MiB | none | 9.32M/s |
| q5 | count distinct, high card | 5.000ms | 15.069ms | 15.134ms | 0.0% | 15.134ms | 15.134ms | 15.134ms | 15.134ms | 10.000ms | 34.48 MiB | none | 6.61M/s |
| q6 | count distinct, strings | 5.000ms | 14.429ms | 14.583ms | 0.0% | 14.583ms | 14.583ms | 14.583ms | 14.583ms | 20.000ms | 32.54 MiB | none | 6.86M/s |
| q7 | min and max of a date | 1.000ms | 10.000ms | 10.162ms | 0.0% | 10.162ms | 10.162ms | 10.162ms | 10.162ms | 0.000us | 27.23 MiB | none | 9.84M/s |
| q8 | group by, low card | 1.000ms | 10.486ms | 10.177ms | 0.0% | 10.177ms | 10.177ms | 10.177ms | 10.177ms | 0.000us | 29.05 MiB | none | 9.83M/s |
| q9 | group by and count distinct | 8.000ms | 17.464ms | 17.156ms | 0.0% | 17.156ms | 17.156ms | 17.156ms | 17.156ms | 20.000ms | 44.55 MiB | none | 5.83M/s |
| q10 | group by, several aggregates | 9.000ms | 19.242ms | 18.805ms | 0.0% | 18.805ms | 18.805ms | 18.805ms | 18.805ms | 30.000ms | 44.80 MiB | none | 5.32M/s |
| q11 | group by a string and count distinct | 5.000ms | 14.466ms | 14.506ms | 0.0% | 14.506ms | 14.506ms | 14.506ms | 14.506ms | 10.000ms | 38.80 MiB | none | 6.89M/s |
| q12 | group by two strings and count distinct | 4.000ms | 14.252ms | 14.224ms | 0.0% | 14.224ms | 14.224ms | 14.224ms | 14.224ms | 10.000ms | 38.30 MiB | none | 7.03M/s |
| q13 | group by a string and top k | 5.000ms | 13.920ms | 15.267ms | 0.0% | 15.267ms | 15.267ms | 15.267ms | 15.267ms | 10.000ms | 34.55 MiB | none | 6.55M/s |
| q14 | group by a string and count distinct | 6.000ms | 16.077ms | 15.754ms | 0.0% | 15.754ms | 15.754ms | 15.754ms | 15.754ms | 20.000ms | 44.30 MiB | none | 6.35M/s |
| q15 | group by two columns and top k | 4.000ms | 14.004ms | 13.927ms | 0.0% | 13.927ms | 13.927ms | 13.927ms | 13.927ms | 10.000ms | 35.05 MiB | none | 7.18M/s |
| q16 | group by, very high card | 7.000ms | 16.330ms | 16.587ms | 0.0% | 16.587ms | 16.587ms | 16.587ms | 16.587ms | 30.000ms | 38.57 MiB | none | 6.03M/s |
| q17 | group by two, very high card | 11.000ms | 20.598ms | 20.572ms | 0.0% | 20.572ms | 20.572ms | 20.572ms | 20.572ms | 20.000ms | 47.88 MiB | none | 4.86M/s |
| q18 | group by two, no ordering | 13.000ms | 22.859ms | 22.547ms | 0.0% | 22.547ms | 22.547ms | 22.547ms | 22.547ms | 30.000ms | 56.10 MiB | none | 4.44M/s |
| q19 | group by with an extract | 11.000ms | 21.821ms | 21.652ms | 0.0% | 21.652ms | 21.652ms | 21.652ms | 21.652ms | 40.000ms | 50.27 MiB | none | 4.62M/s |
| q20 | point lookup | 1.000ms | 10.289ms | 9.959ms | 0.0% | 9.959ms | 9.959ms | 9.959ms | 9.959ms | 0.000us | 28.29 MiB | none | 10.04M/s |
| q21 | substring scan | 8.000ms | 17.398ms | 17.518ms | 0.0% | 17.518ms | 17.518ms | 17.518ms | 17.518ms | 20.000ms | 34.03 MiB | none | 5.71M/s |
| q22 | substring scan and group by | 8.000ms | 18.463ms | 18.714ms | 0.0% | 18.714ms | 18.714ms | 18.714ms | 18.714ms | 10.000ms | 37.55 MiB | none | 5.34M/s |
| q23 | two substring scans and group by | 12.000ms | 22.276ms | 21.706ms | 0.0% | 21.706ms | 21.706ms | 21.706ms | 21.706ms | 20.000ms | 43.30 MiB | none | 4.61M/s |
| q24 | select star and top k | 25.000ms | 35.683ms | 35.236ms | 0.0% | 35.236ms | 35.236ms | 35.236ms | 35.236ms | 40.000ms | 65.04 MiB | none | 2.84M/s |
| q25 | top k by a date | 4.000ms | 12.976ms | 12.705ms | 0.0% | 12.705ms | 12.705ms | 12.705ms | 12.705ms | 10.000ms | 31.52 MiB | none | 7.87M/s |
| q26 | top k by a string | 2.000ms | 11.541ms | 11.487ms | 0.0% | 11.487ms | 11.487ms | 11.487ms | 11.487ms | 0.000us | 28.54 MiB | none | 8.71M/s |
| q27 | top k by two columns | 3.000ms | 12.031ms | 12.418ms | 0.0% | 12.418ms | 12.418ms | 12.418ms | 12.418ms | 10.000ms | 29.94 MiB | none | 8.05M/s |
| q28 | group by with a string length | 9.000ms | 19.034ms | 19.351ms | 0.0% | 19.351ms | 19.351ms | 19.351ms | 19.351ms | 10.000ms | 37.88 MiB | none | 5.17M/s |
| q29 | group by a regular expression | 53.000ms | 64.924ms | 65.388ms | 0.0% | 65.388ms | 65.388ms | 65.388ms | 65.388ms | 70.000ms | 47.29 MiB | none | 1.53M/s |
| q30 | ninety sums over one column | 4.000ms | 13.679ms | 13.386ms | 0.0% | 13.386ms | 13.386ms | 13.386ms | 13.386ms | 10.000ms | 31.52 MiB | none | 7.47M/s |
| q31 | group by two and several aggregates | 4.000ms | 15.197ms | 15.019ms | 0.0% | 15.019ms | 15.019ms | 15.019ms | 15.019ms | 20.000ms | 37.13 MiB | none | 6.66M/s |
| q32 | group by a high card pair | 5.000ms | 14.568ms | 14.607ms | 0.0% | 14.607ms | 14.607ms | 14.607ms | 14.607ms | 10.000ms | 36.83 MiB | none | 6.85M/s |
| q33 | group by a high card pair, unfiltered | 9.000ms | 19.392ms | 18.327ms | 0.0% | 18.327ms | 18.327ms | 18.327ms | 18.327ms | 30.000ms | 49.27 MiB | none | 5.46M/s |
| q34 | group by a long string | 19.000ms | 30.486ms | 31.550ms | 0.0% | 31.550ms | 31.550ms | 31.550ms | 31.550ms | 70.000ms | 60.00 MiB | none | 3.17M/s |
| q35 | group by a constant and a long string | 19.000ms | 30.631ms | 31.046ms | 0.0% | 31.046ms | 31.046ms | 31.046ms | 31.046ms | 20.000ms | 58.04 MiB | none | 3.22M/s |
| q36 | group by four expressions | 8.000ms | 19.510ms | 17.765ms | 0.0% | 17.765ms | 17.765ms | 17.765ms | 17.765ms | 20.000ms | 42.57 MiB | none | 5.63M/s |
| q37 | date range and group by a URL | 3.000ms | 12.732ms | 12.441ms | 0.0% | 12.441ms | 12.441ms | 12.441ms | 12.441ms | 10.000ms | 33.49 MiB | none | 8.04M/s |
| q38 | date range and group by a title | 3.000ms | 12.404ms | 13.551ms | 0.0% | 13.551ms | 13.551ms | 13.551ms | 13.551ms | 10.000ms | 33.24 MiB | none | 7.38M/s |
| q39 | date range, group by and offset | 3.000ms | 13.507ms | 12.800ms | 0.0% | 12.800ms | 12.800ms | 12.800ms | 12.800ms | 10.000ms | 32.19 MiB | none | 7.81M/s |
| q40 | date range, a case and a wide group by | 4.000ms | 13.800ms | 13.719ms | 0.0% | 13.719ms | 13.719ms | 13.719ms | 13.719ms | 10.000ms | 36.05 MiB | none | 7.29M/s |
| q41 | date range with an IN and a hash | 3.000ms | 12.909ms | 12.278ms | 0.0% | 12.278ms | 12.278ms | 12.278ms | 12.278ms | 10.000ms | 34.09 MiB | none | 8.14M/s |
| q42 | date range and a deep offset | 3.000ms | 11.942ms | 12.724ms | 0.0% | 12.724ms | 12.724ms | 12.724ms | 12.724ms | 10.000ms | 32.05 MiB | none | 7.86M/s |
| q43 | minute buckets over a date range | 3.000ms | 12.891ms | 12.218ms | 0.0% | 12.218ms | 12.218ms | 12.218ms | 12.218ms | 10.000ms | 32.23 MiB | none | 8.18M/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 314.000ms by its own clock and 739.569ms by ours, 741.458ms cold, 720.000ms of CPU, peak 65.04 MiB, 13.69M/s and 2.02 GiB/s.

Running it cost 136% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 6.57x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 23.910ms | 22.859ms | 0.0% | 22.859ms | 22.859ms | 22.859ms | 22.859ms | 10.000ms | 40.02 MiB | none | 4.37M/s |
| q2 | filtered count | 1.000ms | 22.937ms | 23.937ms | 0.0% | 23.937ms | 23.937ms | 23.937ms | 23.937ms | 20.000ms | 40.14 MiB | none | 4.18M/s |
| q3 | three aggregates | 1.000ms | 23.973ms | 23.951ms | 0.0% | 23.951ms | 23.951ms | 23.951ms | 23.951ms | 20.000ms | 40.52 MiB | none | 4.18M/s |
| q4 | average | 1.000ms | 23.652ms | 24.028ms | 0.0% | 24.028ms | 24.028ms | 24.028ms | 24.028ms | 20.000ms | 41.14 MiB | none | 4.16M/s |
| q5 | count distinct, high card | 5.000ms | 27.216ms | 27.545ms | 0.0% | 27.545ms | 27.545ms | 27.545ms | 27.545ms | 30.000ms | 47.81 MiB | none | 3.63M/s |
| q6 | count distinct, strings | 3.000ms | 26.331ms | 26.144ms | 0.0% | 26.144ms | 26.144ms | 26.144ms | 26.144ms | 20.000ms | 44.39 MiB | none | 3.82M/s |
| q7 | min and max of a date | 1.000ms | 23.501ms | 23.488ms | 0.0% | 23.488ms | 23.488ms | 23.488ms | 23.488ms | 10.000ms | 40.27 MiB | none | 4.26M/s |
| q8 | group by, low card | 6.000ms | 29.907ms | 28.623ms | 0.0% | 28.623ms | 28.623ms | 28.623ms | 28.623ms | 30.000ms | 43.03 MiB | none | 3.49M/s |
| q9 | group by and count distinct | 6.000ms | 29.414ms | 29.512ms | 0.0% | 29.512ms | 29.512ms | 29.512ms | 29.512ms | 30.000ms | 56.04 MiB | none | 3.39M/s |
| q10 | group by, several aggregates | 9.000ms | 31.517ms | 32.914ms | 0.0% | 32.914ms | 32.914ms | 32.914ms | 32.914ms | 40.000ms | 58.84 MiB | none | 3.04M/s |
| q11 | group by a string and count distinct | 5.000ms | 26.662ms | 26.598ms | 0.0% | 26.598ms | 26.598ms | 26.598ms | 26.598ms | 20.000ms | 48.54 MiB | none | 3.76M/s |
| q12 | group by two strings and count distinct | 5.000ms | 26.840ms | 28.058ms | 0.0% | 28.058ms | 28.058ms | 28.058ms | 28.058ms | 30.000ms | 50.14 MiB | none | 3.56M/s |
| q13 | group by a string and top k | 4.000ms | 29.562ms | 26.000ms | 0.0% | 26.000ms | 26.000ms | 26.000ms | 26.000ms | 20.000ms | 45.16 MiB | none | 3.85M/s |
| q14 | group by a string and count distinct | 7.000ms | 28.985ms | 28.762ms | 0.0% | 28.762ms | 28.762ms | 28.762ms | 28.762ms | 30.000ms | 56.78 MiB | none | 3.48M/s |
| q15 | group by two columns and top k | 5.000ms | 27.140ms | 27.242ms | 0.0% | 27.242ms | 27.242ms | 27.242ms | 27.242ms | 30.000ms | 46.02 MiB | none | 3.67M/s |
| q16 | group by, very high card | 5.000ms | 27.905ms | 27.414ms | 0.0% | 27.414ms | 27.414ms | 27.414ms | 27.414ms | 20.000ms | 51.66 MiB | none | 3.65M/s |
| q17 | group by two, very high card | 11.000ms | 37.225ms | 34.640ms | 0.0% | 34.640ms | 34.640ms | 34.640ms | 34.640ms | 40.000ms | 60.05 MiB | none | 2.89M/s |
| q18 | group by two, no ordering | 11.000ms | 35.090ms | 35.010ms | 0.0% | 35.010ms | 35.010ms | 35.010ms | 35.010ms | 40.000ms | 69.33 MiB | none | 2.86M/s |
| q19 | group by with an extract | 11.000ms | 34.033ms | 32.723ms | 0.0% | 32.723ms | 32.723ms | 32.723ms | 32.723ms | 40.000ms | 61.23 MiB | none | 3.06M/s |
| q20 | point lookup | 1.000ms | 25.631ms | 26.238ms | 0.0% | 26.238ms | 26.238ms | 26.238ms | 26.238ms | 20.000ms | 40.02 MiB | none | 3.81M/s |
| q21 | substring scan | 9.000ms | 31.970ms | 30.769ms | 0.0% | 30.769ms | 30.769ms | 30.769ms | 30.769ms | 30.000ms | 46.27 MiB | none | 3.25M/s |
| q22 | substring scan and group by | 10.000ms | 32.007ms | 32.650ms | 0.0% | 32.650ms | 32.650ms | 32.650ms | 32.650ms | 30.000ms | 50.53 MiB | none | 3.06M/s |
| q23 | two substring scans and group by | 12.000ms | 36.015ms | 35.060ms | 0.0% | 35.060ms | 35.060ms | 35.060ms | 35.060ms | 30.000ms | 57.35 MiB | none | 2.85M/s |
| q24 | select star and top k | 29.000ms | 52.260ms | 53.264ms | 0.0% | 53.264ms | 53.264ms | 53.264ms | 53.264ms | 50.000ms | 81.91 MiB | none | 1.88M/s |
| q25 | top k by a date | 3.000ms | 27.091ms | 27.445ms | 0.0% | 27.445ms | 27.445ms | 27.445ms | 27.445ms | 30.000ms | 42.54 MiB | none | 3.64M/s |
| q26 | top k by a string | 2.000ms | 26.193ms | 25.396ms | 0.0% | 25.396ms | 25.396ms | 25.396ms | 25.396ms | 20.000ms | 42.39 MiB | none | 3.94M/s |
| q27 | top k by two columns | 3.000ms | 26.793ms | 24.915ms | 0.0% | 24.915ms | 24.915ms | 24.915ms | 24.915ms | 20.000ms | 43.03 MiB | none | 4.01M/s |
| q28 | group by with a string length | 11.000ms | 33.887ms | 33.803ms | 0.0% | 33.803ms | 33.803ms | 33.803ms | 33.803ms | 40.000ms | 50.49 MiB | none | 2.96M/s |
| q29 | group by a regular expression | 55.000ms | 76.398ms | 77.980ms | 0.0% | 77.980ms | 77.980ms | 77.980ms | 77.980ms | 130.000ms | 55.27 MiB | none | 1.28M/s |
| q30 | ninety sums over one column | 14.000ms | 38.706ms | 36.721ms | 0.0% | 36.721ms | 36.721ms | 36.721ms | 36.721ms | 30.000ms | 53.14 MiB | none | 2.72M/s |
| q31 | group by two and several aggregates | 6.000ms | 27.608ms | 28.299ms | 0.0% | 28.299ms | 28.299ms | 28.299ms | 28.299ms | 20.000ms | 48.80 MiB | none | 3.53M/s |
| q32 | group by a high card pair | 5.000ms | 27.739ms | 27.633ms | 0.0% | 27.633ms | 27.633ms | 27.633ms | 27.633ms | 20.000ms | 49.05 MiB | none | 3.62M/s |
| q33 | group by a high card pair, unfiltered | 9.000ms | 32.616ms | 30.265ms | 0.0% | 30.265ms | 30.265ms | 30.265ms | 30.265ms | 40.000ms | 59.72 MiB | none | 3.30M/s |
| q34 | group by a long string | 17.000ms | 40.406ms | 40.330ms | 0.0% | 40.330ms | 40.330ms | 40.330ms | 40.330ms | 40.000ms | 72.79 MiB | none | 2.48M/s |
| q35 | group by a constant and a long string | 19.000ms | 41.893ms | 42.306ms | 0.0% | 42.306ms | 42.306ms | 42.306ms | 42.306ms | 40.000ms | 71.91 MiB | none | 2.36M/s |
| q36 | group by four expressions | 5.000ms | 28.361ms | 28.182ms | 0.0% | 28.182ms | 28.182ms | 28.182ms | 28.182ms | 20.000ms | 51.30 MiB | none | 3.55M/s |
| q37 | date range and group by a URL | 5.000ms | 26.260ms | 25.637ms | 0.0% | 25.637ms | 25.637ms | 25.637ms | 25.637ms | 30.000ms | 46.02 MiB | none | 3.90M/s |
| q38 | date range and group by a title | 4.000ms | 25.996ms | 25.414ms | 0.0% | 25.414ms | 25.414ms | 25.414ms | 25.414ms | 20.000ms | 45.29 MiB | none | 3.93M/s |
| q39 | date range, group by and offset | 4.000ms | 26.013ms | 26.287ms | 0.0% | 26.287ms | 26.287ms | 26.287ms | 26.287ms | 20.000ms | 46.04 MiB | none | 3.80M/s |
| q40 | date range, a case and a wide group by | 6.000ms | 28.775ms | 28.076ms | 0.0% | 28.076ms | 28.076ms | 28.076ms | 28.076ms | 20.000ms | 48.54 MiB | none | 3.56M/s |
| q41 | date range with an IN and a hash | 4.000ms | 25.445ms | 25.292ms | 0.0% | 25.292ms | 25.292ms | 25.292ms | 25.292ms | 20.000ms | 45.80 MiB | none | 3.95M/s |
| q42 | date range and a deep offset | 7.000ms | 29.861ms | 30.622ms | 0.0% | 30.622ms | 30.622ms | 30.622ms | 30.622ms | 30.000ms | 46.16 MiB | none | 3.27M/s |
| q43 | minute buckets over a date range | 4.000ms | 25.648ms | 25.474ms | 0.0% | 25.474ms | 25.474ms | 25.474ms | 25.474ms | 20.000ms | 44.02 MiB | none | 3.93M/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 342.000ms by its own clock and 1.318s by ours, 1.329s cold, 1.270s of CPU, peak 81.91 MiB, 12.57M/s and 1.86 GiB/s.

Running it cost 285% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.41x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 2.000ms | 48.501ms | 51.620ms | 0.0% | 51.620ms | 51.620ms | 51.620ms | 51.620ms | 50.000ms | 200.72 MiB | none | 1.94M/s |
| q2 | filtered count | 2.000ms | 51.902ms | 48.157ms | 0.0% | 48.157ms | 48.157ms | 48.157ms | 48.157ms | 50.000ms | 201.13 MiB | none | 2.08M/s |
| q3 | three aggregates | 3.000ms | 49.279ms | 52.895ms | 0.0% | 52.895ms | 52.895ms | 52.895ms | 52.895ms | 50.000ms | 204.07 MiB | none | 1.89M/s |
| q4 | average | 3.000ms | 50.141ms | 49.625ms | 0.0% | 49.625ms | 49.625ms | 49.625ms | 49.625ms | 50.000ms | 204.32 MiB | none | 2.02M/s |
| q5 | count distinct, high card | 7.000ms | 53.529ms | 53.442ms | 0.0% | 53.442ms | 53.442ms | 53.442ms | 53.442ms | 50.000ms | 208.32 MiB | none | 1.87M/s |
| q6 | count distinct, strings | 5.000ms | 51.310ms | 51.490ms | 0.0% | 51.490ms | 51.490ms | 51.490ms | 51.490ms | 50.000ms | 206.57 MiB | none | 1.94M/s |
| q7 | min and max of a date | 3.000ms | 49.724ms | 52.461ms | 0.0% | 52.461ms | 52.461ms | 52.461ms | 52.461ms | 60.000ms | 203.07 MiB | none | 1.91M/s |
| q8 | group by, low card | 4.000ms | 50.288ms | 50.448ms | 0.0% | 50.448ms | 50.448ms | 50.448ms | 50.448ms | 50.000ms | 206.08 MiB | none | 1.98M/s |
| q9 | group by and count distinct | 6.000ms | 57.199ms | 53.125ms | 0.0% | 53.125ms | 53.125ms | 53.125ms | 53.125ms | 60.000ms | 209.27 MiB | none | 1.88M/s |
| q10 | group by, several aggregates | 6.000ms | 54.191ms | 52.341ms | 0.0% | 52.341ms | 52.341ms | 52.341ms | 52.341ms | 60.000ms | 211.57 MiB | none | 1.91M/s |
| q11 | group by a string and count distinct | 4.000ms | 50.036ms | 53.904ms | 0.0% | 53.904ms | 53.904ms | 53.904ms | 53.904ms | 50.000ms | 207.32 MiB | none | 1.86M/s |
| q12 | group by two strings and count distinct | 4.000ms | 55.493ms | 53.197ms | 0.0% | 53.197ms | 53.197ms | 53.197ms | 53.197ms | 60.000ms | 208.57 MiB | none | 1.88M/s |
| q13 | group by a string and top k | 6.000ms | 51.495ms | 52.571ms | 0.0% | 52.571ms | 52.571ms | 52.571ms | 52.571ms | 50.000ms | 210.93 MiB | none | 1.90M/s |
| q14 | group by a string and count distinct | 7.000ms | 58.126ms | 54.557ms | 0.0% | 54.557ms | 54.557ms | 54.557ms | 54.557ms | 60.000ms | 214.07 MiB | none | 1.83M/s |
| q15 | group by two columns and top k | 6.000ms | 56.071ms | 52.751ms | 0.0% | 52.751ms | 52.751ms | 52.751ms | 52.751ms | 50.000ms | 211.06 MiB | none | 1.90M/s |
| q16 | group by, very high card | 6.000ms | 56.073ms | 63.577ms | 0.0% | 63.577ms | 63.577ms | 63.577ms | 63.577ms | 60.000ms | 210.82 MiB | none | 1.57M/s |
| q17 | group by two, very high card | 13.000ms | 60.057ms | 63.324ms | 0.0% | 63.324ms | 63.324ms | 63.324ms | 63.324ms | 60.000ms | 222.58 MiB | none | 1.58M/s |
| q18 | group by two, no ordering | 6.000ms | 57.122ms | 55.460ms | 0.0% | 55.460ms | 55.460ms | 55.460ms | 55.460ms | 50.000ms | 210.82 MiB | none | 1.80M/s |
| q19 | group by with an extract | 14.000ms | 60.566ms | 60.610ms | 0.0% | 60.610ms | 60.610ms | 60.610ms | 60.610ms | 60.000ms | 223.36 MiB | none | 1.65M/s |
| q20 | point lookup | 4.000ms | 49.937ms | 50.365ms | 0.0% | 50.365ms | 50.365ms | 50.365ms | 50.365ms | 60.000ms | 203.73 MiB | none | 1.99M/s |
| q21 | substring scan | 9.000ms | 55.896ms | 61.448ms | 0.0% | 61.448ms | 61.448ms | 61.448ms | 61.448ms | 60.000ms | 208.34 MiB | none | 1.63M/s |
| q22 | substring scan and group by | 10.000ms | 55.795ms | 61.835ms | 0.0% | 61.835ms | 61.835ms | 61.835ms | 61.835ms | 70.000ms | 213.57 MiB | none | 1.62M/s |
| q23 | two substring scans and group by | 12.000ms | 63.327ms | 58.572ms | 0.0% | 58.572ms | 58.572ms | 58.572ms | 58.572ms | 60.000ms | 214.05 MiB | none | 1.71M/s |
| q24 | select star and top k | 116.000ms | 207.255ms | 171.758ms | 0.0% | 171.758ms | 171.758ms | 171.758ms | 171.758ms | 150.000ms | 246.31 MiB | none | 582.20K/s |
| q25 | top k by a date | 6.000ms | 53.471ms | 51.134ms | 0.0% | 51.134ms | 51.134ms | 51.134ms | 51.134ms | 50.000ms | 207.70 MiB | none | 1.96M/s |
| q26 | top k by a string | 4.000ms | 49.520ms | 50.722ms | 0.0% | 50.722ms | 50.722ms | 50.722ms | 50.722ms | 50.000ms | 205.82 MiB | none | 1.97M/s |
| q27 | top k by two columns | 5.000ms | 55.482ms | 52.841ms | 0.0% | 52.841ms | 52.841ms | 52.841ms | 52.841ms | 60.000ms | 208.30 MiB | none | 1.89M/s |
| q28 | group by with a string length | 5.000ms | 59.599ms | 59.990ms | 0.0% | 59.990ms | 59.990ms | 59.990ms | 59.990ms | 60.000ms | 208.16 MiB | none | 1.67M/s |
| q29 | group by a regular expression | 27.000ms | 82.902ms | 78.830ms | 0.0% | 78.830ms | 78.830ms | 78.830ms | 78.830ms | 80.000ms | 239.27 MiB | none | 1.27M/s |
| q30 | ninety sums over one column | 8.000ms | 55.700ms | 56.639ms | 0.0% | 56.639ms | 56.639ms | 56.639ms | 56.639ms | 60.000ms | 207.37 MiB | none | 1.77M/s |
| q31 | group by two and several aggregates | 6.000ms | 52.353ms | 53.335ms | 0.0% | 53.335ms | 53.335ms | 53.335ms | 53.335ms | 50.000ms | 210.02 MiB | none | 1.87M/s |
| q32 | group by a high card pair | 6.000ms | 52.028ms | 56.950ms | 0.0% | 56.950ms | 56.950ms | 56.950ms | 56.950ms | 60.000ms | 211.32 MiB | none | 1.76M/s |
| q33 | group by a high card pair, unfiltered | 11.000ms | 61.552ms | 64.732ms | 0.0% | 64.732ms | 64.732ms | 64.732ms | 64.732ms | 60.000ms | 220.57 MiB | none | 1.54M/s |
| q34 | group by a long string | 20.000ms | 69.697ms | 72.237ms | 0.0% | 72.237ms | 72.237ms | 72.237ms | 72.237ms | 70.000ms | 238.13 MiB | none | 1.38M/s |
| q35 | group by a constant and a long string | 21.000ms | 74.992ms | 74.214ms | 0.0% | 74.214ms | 74.214ms | 74.214ms | 74.214ms | 80.000ms | 237.47 MiB | none | 1.35M/s |
| q36 | group by four expressions | 6.000ms | 54.691ms | 51.169ms | 0.0% | 51.169ms | 51.169ms | 51.169ms | 51.169ms | 50.000ms | 210.95 MiB | none | 1.95M/s |
| q37 | date range and group by a URL | 8.000ms | 53.561ms | 57.256ms | 0.0% | 57.256ms | 57.256ms | 57.256ms | 57.256ms | 60.000ms | 211.31 MiB | none | 1.75M/s |
| q38 | date range and group by a title | 7.000ms | 53.269ms | 60.220ms | 0.0% | 60.220ms | 60.220ms | 60.220ms | 60.220ms | 60.000ms | 211.06 MiB | none | 1.66M/s |
| q39 | date range, group by and offset | 8.000ms | 55.769ms | 54.885ms | 0.0% | 54.885ms | 54.885ms | 54.885ms | 54.885ms | 60.000ms | 211.93 MiB | none | 1.82M/s |
| q40 | date range, a case and a wide group by | 9.000ms | 59.362ms | 59.521ms | 0.0% | 59.521ms | 59.521ms | 59.521ms | 59.521ms | 60.000ms | 214.82 MiB | none | 1.68M/s |
| q41 | date range with an IN and a hash | 6.000ms | 53.005ms | 52.598ms | 0.0% | 52.598ms | 52.598ms | 52.598ms | 52.598ms | 50.000ms | 209.30 MiB | none | 1.90M/s |
| q42 | date range and a deep offset | 6.000ms | 57.911ms | 53.432ms | 0.0% | 53.432ms | 53.432ms | 53.432ms | 53.432ms | 60.000ms | 209.24 MiB | none | 1.87M/s |
| q43 | minute buckets over a date range | 5.000ms | 52.130ms | 52.363ms | 0.0% | 52.363ms | 52.363ms | 52.363ms | 52.363ms | 60.000ms | 208.61 MiB | none | 1.91M/s |

clickhouse-local 26.9.1.1162 over 43 of 43 queries. Total 432.000ms by its own clock and 2.543s by ours, 2.560s cold, 2.570s of CPU, peak 246.31 MiB, 9.95M/s and 1.47 GiB/s.

Running it cost 489% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.57x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 13.682ms | 12.916ms | 0.0% | 12.916ms | 12.916ms | 12.916ms | 12.916ms | 0.000us | 80.23 MiB | none | 7.74M/s |
| q2 | filtered count | 4.000ms | 16.687ms | 16.667ms | 0.0% | 16.667ms | 16.667ms | 16.667ms | 16.667ms | 20.000ms | 106.67 MiB | none | 6.00M/s |
| q3 | three aggregates | 5.000ms | 14.587ms | 15.787ms | 0.0% | 15.787ms | 15.787ms | 15.787ms | 15.787ms | 20.000ms | 116.45 MiB | none | 6.33M/s |
| q4 | average | 4.000ms | 14.588ms | 14.720ms | 0.0% | 14.720ms | 14.720ms | 14.720ms | 14.720ms | 20.000ms | 111.05 MiB | none | 6.79M/s |
| q5 | count distinct, high card | 8.000ms | 20.338ms | 19.591ms | 0.0% | 19.591ms | 19.591ms | 19.591ms | 19.591ms | 40.000ms | 166.91 MiB | none | 5.10M/s |
| q6 | count distinct, strings | 10.000ms | 22.474ms | 21.648ms | 0.0% | 21.648ms | 21.648ms | 21.648ms | 21.648ms | 60.000ms | 181.09 MiB | none | 4.62M/s |
| q7 | min and max of a date | 1.000ms | 12.531ms | 12.209ms | 0.0% | 12.209ms | 12.209ms | 12.209ms | 12.209ms | 0.000us | 80.09 MiB | none | 8.19M/s |
| q8 | group by, low card | 5.000ms | 16.159ms | 16.656ms | 0.0% | 16.656ms | 16.656ms | 16.656ms | 16.656ms | 20.000ms | 116.12 MiB | none | 6.00M/s |
| q9 | group by and count distinct | 13.000ms | 23.544ms | 24.993ms | 0.0% | 24.993ms | 24.993ms | 24.993ms | 24.993ms | 70.000ms | 190.67 MiB | none | 4.00M/s |
| q10 | group by, several aggregates | 10.000ms | 22.790ms | 20.823ms | 0.0% | 20.823ms | 20.823ms | 20.823ms | 20.823ms | 40.000ms | 162.16 MiB | none | 4.80M/s |
| q11 | group by a string and count distinct | 9.000ms | 20.270ms | 21.485ms | 0.0% | 21.485ms | 21.485ms | 21.485ms | 21.485ms | 50.000ms | 178.82 MiB | none | 4.65M/s |
| q12 | group by two strings and count distinct | 10.000ms | 21.120ms | 23.087ms | 0.0% | 23.087ms | 23.087ms | 23.087ms | 23.087ms | 60.000ms | 178.96 MiB | none | 4.33M/s |
| q13 | group by a string and top k | 11.000ms | 23.407ms | 23.283ms | 0.0% | 23.283ms | 23.283ms | 23.283ms | 23.283ms | 70.000ms | 181.04 MiB | none | 4.29M/s |
| q14 | group by a string and count distinct | 16.000ms | 26.976ms | 27.463ms | 0.0% | 27.463ms | 27.463ms | 27.463ms | 27.463ms | 130.000ms | 224.20 MiB | none | 3.64M/s |
| q15 | group by two columns and top k | 15.000ms | 25.836ms | 26.533ms | 0.0% | 26.533ms | 26.533ms | 26.533ms | 26.533ms | 150.000ms | 199.59 MiB | none | 3.77M/s |
| q16 | group by, very high card | 10.000ms | 20.524ms | 22.764ms | 0.0% | 22.764ms | 22.764ms | 22.764ms | 22.764ms | 50.000ms | 164.60 MiB | none | 4.39M/s |
| q17 | group by two, very high card | 14.000ms | 26.869ms | 26.053ms | 0.0% | 26.053ms | 26.053ms | 26.053ms | 26.053ms | 90.000ms | 193.52 MiB | none | 3.84M/s |
| q18 | group by two, no ordering | 14.000ms | 26.389ms | 25.776ms | 0.0% | 25.776ms | 25.776ms | 25.776ms | 25.776ms | 60.000ms | 192.58 MiB | none | 3.88M/s |
| q19 | group by with an extract | 16.000ms | 28.279ms | 27.753ms | 0.0% | 27.753ms | 27.753ms | 27.753ms | 27.753ms | 70.000ms | 206.44 MiB | none | 3.60M/s |
| q20 | point lookup | 4.000ms | 15.074ms | 16.013ms | 0.0% | 16.013ms | 16.013ms | 16.013ms | 16.013ms | 40.000ms | 117.34 MiB | none | 6.24M/s |
| q21 | substring scan | 12.000ms | 23.814ms | 23.461ms | 0.0% | 23.461ms | 23.461ms | 23.461ms | 23.461ms | 40.000ms | 137.40 MiB | none | 4.26M/s |
| q22 | substring scan and group by | 16.000ms | 28.385ms | 27.611ms | 0.0% | 27.611ms | 27.611ms | 27.611ms | 27.611ms | 50.000ms | 177.14 MiB | none | 3.62M/s |
| q23 | two substring scans and group by | 34.000ms | 49.068ms | 46.033ms | 0.0% | 46.033ms | 46.033ms | 46.033ms | 46.033ms | 70.000ms | 186.63 MiB | none | 2.17M/s |
| q24 | select star and top k | 63.000ms | 78.610ms | 77.686ms | 0.0% | 77.686ms | 77.686ms | 77.686ms | 77.686ms | 80.000ms | 215.06 MiB | none | 1.29M/s |
| q25 | top k by a date | 7.000ms | 19.732ms | 19.111ms | 0.0% | 19.111ms | 19.111ms | 19.111ms | 19.111ms | 20.000ms | 123.45 MiB | none | 5.23M/s |
| q26 | top k by a string | 7.000ms | 17.913ms | 18.885ms | 0.0% | 18.885ms | 18.885ms | 18.885ms | 18.885ms | 20.000ms | 128.72 MiB | none | 5.30M/s |
| q27 | top k by two columns | 9.000ms | 20.051ms | 20.164ms | 0.0% | 20.164ms | 20.164ms | 20.164ms | 20.164ms | 20.000ms | 146.88 MiB | none | 4.96M/s |
| q28 | group by with a string length | 16.000ms | 26.299ms | 27.152ms | 0.0% | 27.152ms | 27.152ms | 27.152ms | 27.152ms | 70.000ms | 169.77 MiB | none | 3.68M/s |
| q29 | group by a regular expression | 36.000ms | 46.920ms | 47.700ms | 0.0% | 47.700ms | 47.700ms | 47.700ms | 47.700ms | 140.000ms | 219.74 MiB | none | 2.10M/s |
| q30 | ninety sums over one column | 12.000ms | 23.609ms | 23.936ms | 0.0% | 23.936ms | 23.936ms | 23.936ms | 23.936ms | 30.000ms | 110.09 MiB | none | 4.18M/s |
| q31 | group by two and several aggregates | 11.000ms | 22.967ms | 22.306ms | 0.0% | 22.306ms | 22.306ms | 22.306ms | 22.306ms | 50.000ms | 164.01 MiB | none | 4.48M/s |
| q32 | group by a high card pair | 11.000ms | 22.903ms | 22.378ms | 0.0% | 22.378ms | 22.378ms | 22.378ms | 22.378ms | 60.000ms | 166.59 MiB | none | 4.47M/s |
| q33 | group by a high card pair, unfiltered | 13.000ms | 24.783ms | 25.946ms | 0.0% | 25.946ms | 25.946ms | 25.946ms | 25.946ms | 80.000ms | 195.49 MiB | none | 3.85M/s |
| q34 | group by a long string | 24.000ms | 37.721ms | 36.365ms | 0.0% | 36.365ms | 36.365ms | 36.365ms | 36.365ms | 150.000ms | 321.23 MiB | none | 2.75M/s |
| q35 | group by a constant and a long string | 21.000ms | 35.905ms | 33.737ms | 0.0% | 33.737ms | 33.737ms | 33.737ms | 33.737ms | 60.000ms | 303.04 MiB | none | 2.96M/s |
| q36 | group by four expressions | 10.000ms | 21.397ms | 22.015ms | 0.0% | 22.015ms | 22.015ms | 22.015ms | 22.015ms | 50.000ms | 174.43 MiB | none | 4.54M/s |
| q37 | date range and group by a URL | 15.000ms | 27.514ms | 27.147ms | 0.0% | 27.147ms | 27.147ms | 27.147ms | 27.147ms | 70.000ms | 168.91 MiB | none | 3.68M/s |
| q38 | date range and group by a title | 25.000ms | 38.092ms | 36.711ms | 0.0% | 36.711ms | 36.711ms | 36.711ms | 36.711ms | 100.000ms | 177.00 MiB | none | 2.72M/s |
| q39 | date range, group by and offset | 17.000ms | 28.623ms | 28.528ms | 0.0% | 28.528ms | 28.528ms | 28.528ms | 28.528ms | 120.000ms | 164.55 MiB | none | 3.51M/s |
| q40 | date range, a case and a wide group by | 25.000ms | 38.420ms | 37.426ms | 0.0% | 37.426ms | 37.426ms | 37.426ms | 37.426ms | 110.000ms | 183.76 MiB | none | 2.67M/s |
| q41 | date range with an IN and a hash | 8.000ms | 21.432ms | 19.857ms | 0.0% | 19.857ms | 19.857ms | 19.857ms | 19.857ms | 20.000ms | 132.51 MiB | none | 5.04M/s |
| q42 | date range and a deep offset | 9.000ms | 21.002ms | 20.615ms | 0.0% | 20.615ms | 20.615ms | 20.615ms | 20.615ms | 30.000ms | 131.17 MiB | none | 4.85M/s |
| q43 | minute buckets over a date range | 8.000ms | 20.233ms | 19.795ms | 0.0% | 19.795ms | 19.795ms | 19.795ms | 19.795ms | 30.000ms | 133.33 MiB | none | 5.05M/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 589.000ms by its own clock and 1.101s by ours, 1.108s cold, 2.530s of CPU, peak 321.23 MiB, 7.30M/s and 1.08 GiB/s.

Running it cost 87% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 6.36x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 5.837ms | 107.452ms | 97.808ms | 0.0% | 97.808ms | 97.808ms | 97.808ms | 97.808ms | 130.000ms | 61.08 MiB | none | 1.02M/s |
| q2 | filtered count | 7.109ms | 95.871ms | 100.345ms | 0.0% | 100.345ms | 100.345ms | 100.345ms | 100.345ms | 120.000ms | 67.21 MiB | none | 996.54K/s |
| q3 | three aggregates | 6.176ms | 93.163ms | 91.148ms | 0.0% | 91.148ms | 91.148ms | 91.148ms | 91.148ms | 100.000ms | 66.05 MiB | none | 1.10M/s |
| q4 | average | 7.423ms | 96.806ms | 100.205ms | 0.0% | 100.205ms | 100.205ms | 100.205ms | 100.205ms | 120.000ms | 65.61 MiB | none | 997.93K/s |
| q5 | count distinct, high card | 14.503ms | 104.327ms | 105.631ms | 0.0% | 105.631ms | 105.631ms | 105.631ms | 105.631ms | 130.000ms | 80.79 MiB | none | 946.67K/s |
| q6 | count distinct, strings | 13.965ms | 104.574ms | 102.449ms | 0.0% | 102.449ms | 102.449ms | 102.449ms | 102.449ms | 120.000ms | 82.05 MiB | none | 976.08K/s |
| q7 | min and max of a date | 5.776ms | 93.095ms | 87.860ms | 0.0% | 87.860ms | 87.860ms | 87.860ms | 87.860ms | 90.000ms | 63.95 MiB | none | 1.14M/s |
| q8 | group by, low card | 15.737ms | 109.139ms | 103.812ms | 0.0% | 103.812ms | 103.812ms | 103.812ms | 103.812ms | 150.000ms | 75.54 MiB | none | 963.26K/s |
| q9 | group by and count distinct | 24.419ms | 112.291ms | 114.092ms | 0.0% | 114.092ms | 114.092ms | 114.092ms | 114.092ms | 170.000ms | 100.50 MiB | none | 876.47K/s |
| q10 | group by, several aggregates | 28.214ms | 120.277ms | 120.483ms | 0.0% | 120.483ms | 120.483ms | 120.483ms | 120.483ms | 190.000ms | 103.95 MiB | none | 829.98K/s |
| q11 | group by a string and count distinct | 21.516ms | 114.286ms | 117.932ms | 0.0% | 117.932ms | 117.932ms | 117.932ms | 117.932ms | 220.000ms | 85.30 MiB | none | 847.93K/s |
| q12 | group by two strings and count distinct | 23.036ms | 117.095ms | 114.408ms | 0.0% | 114.408ms | 114.408ms | 114.408ms | 114.408ms | 170.000ms | 86.65 MiB | none | 874.05K/s |
| q13 | group by a string and top k | 17.955ms | 109.541ms | 107.896ms | 0.0% | 107.896ms | 107.896ms | 107.896ms | 107.896ms | 140.000ms | 82.20 MiB | none | 926.80K/s |
| q14 | group by a string and count distinct | 23.829ms | 119.957ms | 116.674ms | 0.0% | 116.674ms | 116.674ms | 116.674ms | 116.674ms | 170.000ms | 97.73 MiB | none | 857.07K/s |
| q15 | group by two columns and top k | 19.628ms | 117.876ms | 114.069ms | 0.0% | 114.069ms | 114.069ms | 114.069ms | 114.069ms | 150.000ms | 83.66 MiB | none | 876.64K/s |
| q16 | group by, very high card | 18.761ms | 108.771ms | 109.512ms | 0.0% | 109.512ms | 109.512ms | 109.512ms | 109.512ms | 140.000ms | 85.45 MiB | none | 913.12K/s |
| q17 | group by two, very high card | 24.096ms | 120.044ms | 115.613ms | 0.0% | 115.613ms | 115.613ms | 115.613ms | 115.613ms | 160.000ms | 100.61 MiB | none | 864.94K/s |
| q18 | group by two, no ordering | 16.357ms | 107.654ms | 104.775ms | 0.0% | 104.775ms | 104.775ms | 104.775ms | 104.775ms | 150.000ms | 99.07 MiB | none | 954.41K/s |
| q19 | group by with an extract | 26.708ms | 117.183ms | 120.015ms | 0.0% | 120.015ms | 120.015ms | 120.015ms | 120.015ms | 190.000ms | 105.05 MiB | none | 833.21K/s |
| q20 | point lookup | 5.434ms | 94.708ms | 94.656ms | 0.0% | 94.656ms | 94.656ms | 94.656ms | 94.656ms | 100.000ms | 67.35 MiB | none | 1.06M/s |
| q21 | substring scan | 24.257ms | 112.747ms | 113.884ms | 0.0% | 113.884ms | 113.884ms | 113.884ms | 113.884ms | 140.000ms | 79.99 MiB | none | 878.07K/s |
| q22 | substring scan and group by | 31.164ms | 123.559ms | 118.600ms | 0.0% | 118.600ms | 118.600ms | 118.600ms | 118.600ms | 150.000ms | 89.19 MiB | none | 843.15K/s |
| q23 | two substring scans and group by | 42.434ms | 132.218ms | 133.302ms | 0.0% | 133.302ms | 133.302ms | 133.302ms | 133.302ms | 190.000ms | 122.57 MiB | none | 750.16K/s |
| q24 | select star and top k | 57.852ms | 150.269ms | 149.408ms | 0.0% | 149.408ms | 149.408ms | 149.408ms | 149.408ms | 340.000ms | 125.12 MiB | none | 669.29K/s |
| q25 | top k by a date | 12.943ms | 107.791ms | 109.633ms | 0.0% | 109.633ms | 109.633ms | 109.633ms | 109.633ms | 170.000ms | 75.81 MiB | none | 912.12K/s |
| q26 | top k by a string | 11.481ms | 117.030ms | 101.655ms | 0.0% | 101.655ms | 101.655ms | 101.655ms | 101.655ms | 150.000ms | 73.80 MiB | none | 983.70K/s |
| q27 | top k by two columns | 14.171ms | 100.950ms | 103.403ms | 0.0% | 103.403ms | 103.403ms | 103.403ms | 103.403ms | 120.000ms | 76.55 MiB | none | 967.07K/s |
| q30 | ninety sums over one column | 9.853ms | 103.970ms | 98.193ms | 0.0% | 98.193ms | 98.193ms | 98.193ms | 98.193ms | 100.000ms | 68.39 MiB | none | 1.02M/s |
| q31 | group by two and several aggregates | 21.820ms | 114.388ms | 114.844ms | 0.0% | 114.844ms | 114.844ms | 114.844ms | 114.844ms | 150.000ms | 85.65 MiB | none | 870.73K/s |
| q32 | group by a high card pair | 16.401ms | 106.109ms | 102.962ms | 0.0% | 102.962ms | 102.962ms | 102.962ms | 102.962ms | 140.000ms | 86.59 MiB | none | 971.21K/s |
| q33 | group by a high card pair, unfiltered | 18.966ms | 108.733ms | 109.772ms | 0.0% | 109.772ms | 109.772ms | 109.772ms | 109.772ms | 150.000ms | 105.53 MiB | none | 910.96K/s |
| q34 | group by a long string | 29.656ms | 120.016ms | 120.484ms | 0.0% | 120.484ms | 120.484ms | 120.484ms | 120.484ms | 170.000ms | 129.54 MiB | none | 829.97K/s |
| q35 | group by a constant and a long string | 32.224ms | 127.333ms | 127.345ms | 0.0% | 127.345ms | 127.345ms | 127.345ms | 127.345ms | 190.000ms | 137.41 MiB | none | 785.25K/s |
| q37 | date range and group by a URL | 27.863ms | 120.059ms | 121.151ms | 0.0% | 121.151ms | 121.151ms | 121.151ms | 121.151ms | 170.000ms | 105.59 MiB | none | 825.40K/s |
| q38 | date range and group by a title | 30.578ms | 121.272ms | 119.271ms | 0.0% | 119.271ms | 119.271ms | 119.271ms | 119.271ms | 150.000ms | 104.51 MiB | none | 838.41K/s |
| q39 | date range, group by and offset | 21.983ms | 115.126ms | 111.797ms | 0.0% | 111.797ms | 111.797ms | 111.797ms | 111.797ms | 140.000ms | 90.13 MiB | none | 894.46K/s |
| q40 | date range, a case and a wide group by | 22.746ms | 112.404ms | 111.738ms | 0.0% | 111.738ms | 111.738ms | 111.738ms | 111.738ms | 140.000ms | 97.79 MiB | none | 894.93K/s |
| q41 | date range with an IN and a hash | 15.824ms | 111.633ms | 104.065ms | 0.0% | 104.065ms | 104.065ms | 104.065ms | 104.065ms | 120.000ms | 83.47 MiB | none | 960.92K/s |
| q42 | date range and a deep offset | 16.126ms | 107.271ms | 107.505ms | 0.0% | 107.505ms | 107.505ms | 107.505ms | 107.505ms | 130.000ms | 80.78 MiB | none | 930.17K/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 784.821ms by its own clock and 4.318s by ours, 4.377s cold, 5.920s of CPU, peak 137.41 MiB, 4.97M/s and 751.06 MiB/s.

Running it cost 450% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.70x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-server in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 29.965ms | 31.418ms | 0.0% | 31.418ms | 31.418ms | 31.418ms | 31.418ms | not read | not read | not read | 3.18M/s |
| q2 | filtered count | 1.000ms | 34.109ms | 29.930ms | 0.0% | 29.930ms | 29.930ms | 29.930ms | 29.930ms | not read | not read | not read | 3.34M/s |
| q3 | three aggregates | 2.000ms | 33.683ms | 36.382ms | 0.0% | 36.382ms | 36.382ms | 36.382ms | 36.382ms | not read | not read | not read | 2.75M/s |
| q4 | average | 2.000ms | 32.788ms | 30.528ms | 0.0% | 30.528ms | 30.528ms | 30.528ms | 30.528ms | not read | not read | not read | 3.28M/s |
| q5 | count distinct, high card | 6.000ms | 35.296ms | 38.647ms | 0.0% | 38.647ms | 38.647ms | 38.647ms | 38.647ms | not read | not read | not read | 2.59M/s |
| q6 | count distinct, strings | 3.000ms | 32.474ms | 34.740ms | 0.0% | 34.740ms | 34.740ms | 34.740ms | 34.740ms | not read | not read | not read | 2.88M/s |
| q7 | min and max of a date | 1.000ms | 33.299ms | 33.464ms | 0.0% | 33.464ms | 33.464ms | 33.464ms | 33.464ms | not read | not read | not read | 2.99M/s |
| q8 | group by, low card | 2.000ms | 34.181ms | 34.021ms | 0.0% | 34.021ms | 34.021ms | 34.021ms | 34.021ms | not read | not read | not read | 2.94M/s |
| q9 | group by and count distinct | 4.000ms | 36.914ms | 36.941ms | 0.0% | 36.941ms | 36.941ms | 36.941ms | 36.941ms | not read | not read | not read | 2.71M/s |
| q10 | group by, several aggregates | 19.000ms | 33.685ms | 49.917ms | 0.0% | 49.917ms | 49.917ms | 49.917ms | 49.917ms | not read | not read | not read | 2.00M/s |
| q11 | group by a string and count distinct | 2.000ms | 31.672ms | 31.694ms | 0.0% | 31.694ms | 31.694ms | 31.694ms | 31.694ms | not read | not read | not read | 3.16M/s |
| q12 | group by two strings and count distinct | 3.000ms | 31.004ms | 31.221ms | 0.0% | 31.221ms | 31.221ms | 31.221ms | 31.221ms | not read | not read | not read | 3.20M/s |
| q13 | group by a string and top k | 4.000ms | 31.082ms | 36.268ms | 0.0% | 36.268ms | 36.268ms | 36.268ms | 36.268ms | not read | not read | not read | 2.76M/s |
| q14 | group by a string and count distinct | 5.000ms | 35.928ms | 37.003ms | 0.0% | 37.003ms | 37.003ms | 37.003ms | 37.003ms | not read | not read | not read | 2.70M/s |
| q15 | group by two columns and top k | 4.000ms | 32.622ms | 36.246ms | 0.0% | 36.246ms | 36.246ms | 36.246ms | 36.246ms | not read | not read | not read | 2.76M/s |
| q16 | group by, very high card | 4.000ms | 33.545ms | 33.199ms | 0.0% | 33.199ms | 33.199ms | 33.199ms | 33.199ms | not read | not read | not read | 3.01M/s |
| q17 | group by two, very high card | 9.000ms | 38.207ms | 37.535ms | 0.0% | 37.535ms | 37.535ms | 37.535ms | 37.535ms | not read | not read | not read | 2.66M/s |
| q18 | group by two, no ordering | 4.000ms | 31.787ms | 32.230ms | 0.0% | 32.230ms | 32.230ms | 32.230ms | 32.230ms | not read | not read | not read | 3.10M/s |
| q19 | group by with an extract | 9.000ms | 42.642ms | 38.328ms | 0.0% | 38.328ms | 38.328ms | 38.328ms | 38.328ms | not read | not read | not read | 2.61M/s |
| q20 | point lookup | 1.000ms | 34.642ms | 30.427ms | 0.0% | 30.427ms | 30.427ms | 30.427ms | 30.427ms | not read | not read | not read | 3.29M/s |
| q21 | substring scan | 5.000ms | 37.629ms | 33.499ms | 0.0% | 33.499ms | 33.499ms | 33.499ms | 33.499ms | not read | not read | not read | 2.99M/s |
| q22 | substring scan and group by | 3.000ms | 35.887ms | 32.258ms | 0.0% | 32.258ms | 32.258ms | 32.258ms | 32.258ms | not read | not read | not read | 3.10M/s |
| q23 | two substring scans and group by | 5.000ms | 41.148ms | 38.384ms | 0.0% | 38.384ms | 38.384ms | 38.384ms | 38.384ms | not read | not read | not read | 2.61M/s |
| q24 | select star and top k | 27.000ms | 57.895ms | 51.959ms | 0.0% | 51.959ms | 51.959ms | 51.959ms | 51.959ms | not read | not read | not read | 1.92M/s |
| q25 | top k by a date | 2.000ms | 34.466ms | 30.135ms | 0.0% | 30.135ms | 30.135ms | 30.135ms | 30.135ms | not read | not read | not read | 3.32M/s |
| q26 | top k by a string | 2.000ms | 32.722ms | 34.155ms | 0.0% | 34.155ms | 34.155ms | 34.155ms | 34.155ms | not read | not read | not read | 2.93M/s |
| q27 | top k by two columns | 2.000ms | 31.588ms | 31.581ms | 0.0% | 31.581ms | 31.581ms | 31.581ms | 31.581ms | not read | not read | not read | 3.17M/s |
| q28 | group by with a string length | 3.000ms | 36.073ms | 35.449ms | 0.0% | 35.449ms | 35.449ms | 35.449ms | 35.449ms | not read | not read | not read | 2.82M/s |
| q29 | group by a regular expression | 24.000ms | 52.636ms | 56.738ms | 0.0% | 56.738ms | 56.738ms | 56.738ms | 56.738ms | not read | not read | not read | 1.76M/s |
| q30 | ninety sums over one column | 6.000ms | 37.224ms | 36.147ms | 0.0% | 36.147ms | 36.147ms | 36.147ms | 36.147ms | not read | not read | not read | 2.77M/s |
| q31 | group by two and several aggregates | 3.000ms | 35.458ms | 32.043ms | 0.0% | 32.043ms | 32.043ms | 32.043ms | 32.043ms | not read | not read | not read | 3.12M/s |
| q32 | group by a high card pair | 17.000ms | 31.986ms | 45.398ms | 0.0% | 45.398ms | 45.398ms | 45.398ms | 45.398ms | not read | not read | not read | 2.20M/s |
| q33 | group by a high card pair, unfiltered | 7.000ms | 36.160ms | 36.578ms | 0.0% | 36.578ms | 36.578ms | 36.578ms | 36.578ms | not read | not read | not read | 2.73M/s |
| q34 | group by a long string | 13.000ms | 40.619ms | 41.810ms | 0.0% | 41.810ms | 41.810ms | 41.810ms | 41.810ms | not read | not read | not read | 2.39M/s |
| q35 | group by a constant and a long string | 12.000ms | 40.981ms | 42.350ms | 0.0% | 42.350ms | 42.350ms | 42.350ms | 42.350ms | not read | not read | not read | 2.36M/s |
| q36 | group by four expressions | 5.000ms | 35.244ms | 33.218ms | 0.0% | 33.218ms | 33.218ms | 33.218ms | 33.218ms | not read | not read | not read | 3.01M/s |
| q37 | date range and group by a URL | 3.000ms | 32.434ms | 33.473ms | 0.0% | 33.473ms | 33.473ms | 33.473ms | 33.473ms | not read | not read | not read | 2.99M/s |
| q38 | date range and group by a title | 4.000ms | 38.904ms | 33.267ms | 0.0% | 33.267ms | 33.267ms | 33.267ms | 33.267ms | not read | not read | not read | 3.01M/s |
| q39 | date range, group by and offset | 3.000ms | 32.946ms | 32.131ms | 0.0% | 32.131ms | 32.131ms | 32.131ms | 32.131ms | not read | not read | not read | 3.11M/s |
| q40 | date range, a case and a wide group by | 5.000ms | 33.336ms | 36.810ms | 0.0% | 36.810ms | 36.810ms | 36.810ms | 36.810ms | not read | not read | not read | 2.72M/s |
| q41 | date range with an IN and a hash | 3.000ms | 32.465ms | 32.726ms | 0.0% | 32.726ms | 32.726ms | 32.726ms | 32.726ms | not read | not read | not read | 3.06M/s |
| q42 | date range and a deep offset | 3.000ms | 30.996ms | 32.441ms | 0.0% | 32.441ms | 32.441ms | 32.441ms | 32.441ms | not read | not read | not read | 3.08M/s |
| q43 | minute buckets over a date range | 3.000ms | 33.245ms | 30.540ms | 0.0% | 30.540ms | 30.540ms | 30.540ms | 30.540ms | not read | not read | not read | 3.27M/s |

clickhouse-server 26.9.1.1162 over 43 of 43 queries. Total 246.000ms by its own clock and 1.543s by ours, 1.532s cold, no reading of CPU, peak not read, 17.48M/s and 2.58 GiB/s.

Running it cost 527% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.90x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 0.000us | 1.573ms | 1.390ms | 0.0% | 1.390ms | 1.390ms | 1.390ms | 1.390ms | 0.000us | 4.25 MiB | none | 71.94M/s |
| q2 | filtered count | 1.000ms | 1.928ms | 1.900ms | 0.0% | 1.900ms | 1.900ms | 1.900ms | 1.900ms | 0.000us | 5.00 MiB | none | 52.63M/s |
| q3 | three aggregates | 2.000ms | 3.027ms | 3.135ms | 0.0% | 3.135ms | 3.135ms | 3.135ms | 3.135ms | 0.000us | 5.92 MiB | none | 31.90M/s |
| q4 | average | 2.000ms | 3.325ms | 3.272ms | 0.0% | 3.272ms | 3.272ms | 3.272ms | 3.272ms | 0.000us | 7.76 MiB | none | 30.56M/s |
| q5 | count distinct, high card | 13.000ms | 15.810ms | 14.467ms | 0.0% | 14.467ms | 14.467ms | 14.467ms | 14.467ms | 10.000ms | 14.92 MiB | none | 6.91M/s |
| q6 | count distinct, strings | 9.000ms | 10.000ms | 9.966ms | 0.0% | 9.966ms | 9.966ms | 9.966ms | 9.966ms | 0.000us | 7.32 MiB | none | 10.03M/s |
| q7 | min and max of a date | 1.000ms | 2.299ms | 2.241ms | 0.0% | 2.241ms | 2.241ms | 2.241ms | 2.241ms | 0.000us | 5.02 MiB | none | 44.62M/s |
| q8 | group by, low card | 1.000ms | 1.980ms | 2.003ms | 0.0% | 2.003ms | 2.003ms | 2.003ms | 2.003ms | 0.000us | 5.01 MiB | none | 49.92M/s |
| q9 | group by and count distinct | 18.000ms | 18.767ms | 20.137ms | 0.0% | 20.137ms | 20.137ms | 20.137ms | 20.137ms | 10.000ms | 16.12 MiB | none | 4.97M/s |
| q10 | group by, several aggregates | 21.000ms | 23.585ms | 22.675ms | 0.0% | 22.675ms | 22.675ms | 22.675ms | 22.675ms | 10.000ms | 17.16 MiB | none | 4.41M/s |
| q11 | group by a string and count distinct | 4.000ms | 5.485ms | 5.002ms | 0.0% | 5.002ms | 5.002ms | 5.002ms | 5.002ms | 0.000us | 7.76 MiB | none | 19.99M/s |
| q12 | group by two strings and count distinct | 4.000ms | 5.944ms | 5.834ms | 0.0% | 5.834ms | 5.834ms | 5.834ms | 5.834ms | 0.000us | 7.74 MiB | none | 17.14M/s |
| q13 | group by a string and top k | 15.000ms | 15.810ms | 16.229ms | 0.0% | 16.229ms | 16.229ms | 16.229ms | 16.229ms | 10.000ms | 10.37 MiB | none | 6.16M/s |
| q14 | group by a string and count distinct | 15.000ms | 17.584ms | 17.132ms | 0.0% | 17.132ms | 17.132ms | 17.132ms | 17.132ms | 10.000ms | 13.41 MiB | none | 5.84M/s |
| q15 | group by two columns and top k | 14.000ms | 15.311ms | 15.374ms | 0.0% | 15.374ms | 15.374ms | 15.374ms | 15.374ms | 10.000ms | 11.22 MiB | none | 6.50M/s |
| q16 | group by, very high card | 21.000ms | 23.509ms | 22.641ms | 0.0% | 22.641ms | 22.641ms | 22.641ms | 22.641ms | 10.000ms | 24.66 MiB | none | 4.42M/s |
| q17 | group by two, very high card | 35.000ms | 36.507ms | 36.618ms | 0.0% | 36.618ms | 36.618ms | 36.618ms | 36.618ms | 30.000ms | 33.81 MiB | none | 2.73M/s |
| q18 | group by two, no ordering | 26.000ms | 28.161ms | 28.030ms | 0.0% | 28.030ms | 28.030ms | 28.030ms | 28.030ms | 20.000ms | 33.97 MiB | none | 3.57M/s |
| q20 | point lookup | 2.000ms | 3.270ms | 3.117ms | 0.0% | 3.117ms | 3.117ms | 3.117ms | 3.117ms | 0.000us | 7.72 MiB | none | 32.08M/s |
| q21 | substring scan | 35.000ms | 37.089ms | 36.800ms | 0.0% | 36.800ms | 36.800ms | 36.800ms | 36.800ms | 30.000ms | 24.92 MiB | none | 2.72M/s |
| q22 | substring scan and group by | 39.000ms | 41.088ms | 41.283ms | 0.0% | 41.283ms | 41.283ms | 41.283ms | 41.283ms | 30.000ms | 26.95 MiB | none | 2.42M/s |
| q23 | two substring scans and group by | 86.000ms | 88.016ms | 87.951ms | 0.0% | 87.951ms | 87.951ms | 87.951ms | 87.951ms | 70.000ms | 49.86 MiB | none | 1.14M/s |
| q24 | select star and top k | 197.000ms | 208.421ms | 202.033ms | 0.0% | 202.033ms | 202.033ms | 202.033ms | 202.033ms | 190.000ms | 158.62 MiB | none | 494.96K/s |
| q25 | top k by a date | 13.000ms | 15.310ms | 14.151ms | 0.0% | 14.151ms | 14.151ms | 14.151ms | 14.151ms | 10.000ms | 8.17 MiB | none | 7.07M/s |
| q26 | top k by a string | 11.000ms | 12.979ms | 12.322ms | 0.0% | 12.322ms | 12.322ms | 12.322ms | 12.322ms | 10.000ms | 7.32 MiB | none | 8.12M/s |
| q27 | top k by two columns | 14.000ms | 15.358ms | 16.049ms | 0.0% | 16.049ms | 16.049ms | 16.049ms | 16.049ms | 10.000ms | 8.33 MiB | none | 6.23M/s |
| q28 | group by with a string length | 35.000ms | 37.627ms | 36.794ms | 0.0% | 36.794ms | 36.794ms | 36.794ms | 36.794ms | 30.000ms | 26.16 MiB | none | 2.72M/s |
| q29 | group by a regular expression | 88.000ms | 89.978ms | 89.884ms | 0.0% | 89.884ms | 89.884ms | 89.884ms | 89.884ms | 80.000ms | 20.45 MiB | none | 1.11M/s |
| q30 | ninety sums over one column | 14.000ms | 15.423ms | 15.315ms | 0.0% | 15.315ms | 15.315ms | 15.315ms | 15.315ms | 10.000ms | 5.64 MiB | none | 6.53M/s |
| q31 | group by two and several aggregates | 16.000ms | 18.195ms | 18.063ms | 0.0% | 18.063ms | 18.063ms | 18.063ms | 18.063ms | 0.000us | 15.41 MiB | none | 5.54M/s |
| q32 | group by a high card pair | 15.000ms | 16.574ms | 16.754ms | 0.0% | 16.754ms | 16.754ms | 16.754ms | 16.754ms | 10.000ms | 15.97 MiB | none | 5.97M/s |
| q34 | group by a long string | 61.000ms | 65.536ms | 63.207ms | 0.0% | 63.207ms | 63.207ms | 63.207ms | 63.207ms | 50.000ms | 42.53 MiB | none | 1.58M/s |
| q35 | group by a constant and a long string | 66.000ms | 71.360ms | 68.133ms | 0.0% | 68.133ms | 68.133ms | 68.133ms | 68.133ms | 50.000ms | 49.75 MiB | none | 1.47M/s |
| q36 | group by four expressions | 33.000ms | 36.159ms | 34.835ms | 0.0% | 34.835ms | 34.835ms | 34.835ms | 34.835ms | 30.000ms | 40.34 MiB | none | 2.87M/s |
| q37 | date range and group by a URL | 33.000ms | 35.049ms | 34.944ms | 0.0% | 34.944ms | 34.944ms | 34.944ms | 34.944ms | 20.000ms | 28.39 MiB | none | 2.86M/s |
| q38 | date range and group by a title | 39.000ms | 39.467ms | 40.728ms | 0.0% | 40.728ms | 40.728ms | 40.728ms | 40.728ms | 30.000ms | 29.13 MiB | none | 2.46M/s |
| q39 | date range, group by and offset | 35.000ms | 36.459ms | 36.795ms | 0.0% | 36.795ms | 36.795ms | 36.795ms | 36.795ms | 20.000ms | 29.11 MiB | none | 2.72M/s |
| q40 | date range, a case and a wide group by | 62.000ms | 63.559ms | 63.814ms | 0.0% | 63.814ms | 63.814ms | 63.814ms | 63.814ms | 50.000ms | 46.10 MiB | none | 1.57M/s |
| q41 | date range with an IN and a hash | 7.000ms | 8.936ms | 7.902ms | 0.0% | 7.902ms | 7.902ms | 7.902ms | 7.902ms | 0.000us | 10.86 MiB | none | 12.65M/s |
| q42 | date range and a deep offset | 8.000ms | 8.050ms | 9.354ms | 0.0% | 9.354ms | 9.354ms | 9.354ms | 9.354ms | 0.000us | 10.93 MiB | none | 10.69M/s |
| q43 | minute buckets over a date range | 6.000ms | 7.968ms | 7.425ms | 0.0% | 7.425ms | 7.425ms | 7.425ms | 7.425ms | 0.000us | 9.17 MiB | none | 13.47M/s |

rudb rudb 0.2.31 over 41 of 43 queries. Total 1.117s by its own clock and 1.186s by ours, 1.202s cold, 850.000ms of CPU, peak 158.62 MiB, 3.67M/s and 554.77 MiB/s.

Running it cost 6% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 145.35x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

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

- polars ran every query within 1.70x of every other one, and this suite spreads over 6x on an engine it is measuring
- clickhouse-server ran every query within 1.90x of every other one, and this suite spreads over 6x on an engine it is measuring

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

