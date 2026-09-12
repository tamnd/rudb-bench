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
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 69.630ms | 40.000ms | 1.01 MiB | its own database file | its own | 3.64 to 3.64 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 70.079ms | 60.000ms | 1.01 MiB | its own database file | its own | 3.59 to 3.59 |
| clickhouse-local | 26.9.1.1162 | ran | 156.355ms | 120.000ms | 362.94 KiB | its own MergeTree parts, as system.parts counts the active ones | its own | 3.50 to 3.46 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 225.88 KiB | the source Parquet, this engine has no storage format of its own | the Parquet | 3.59 to 3.59 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 225.88 KiB | the source Parquet, this engine has no storage format of its own | the Parquet | 3.59 to 3.50 |
| clickhouse-server | 26.9.1.1162 | ran | 168.879ms | not read | 356.92 KiB | its own MergeTree parts, sorted by hits by (CounterID, EventDate, UserID, EventTime, WatchID), as system.parts counts the active ones | its own | 3.46 to 3.42 |
| rudb | rudb 0.2.31 | ran | 0.000us | 0.000us | 225.88 KiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 3.59 to 3.59 |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 101.000ms | 499.047ms | +394% | 508.647ms | 320.000ms | 0.64 | 37.55 MiB | none | 425.74K/s | 93.91 MiB/s | 1.00x |
| duckdb-pinned | 136.000ms | 1.088s | +700% | 1.090s | 900.000ms | 0.83 | 53.02 MiB | none | 316.18K/s | 69.74 MiB/s | 1.35x |
| clickhouse-local | 149.000ms | 2.170s | +1356% | 2.157s | 2.120s | 0.98 | 208.08 MiB | none | 288.59K/s | 63.66 MiB/s | 1.48x |
| datafusion | 218.000ms | 710.714ms | +226% | 752.496ms | 770.000ms | 1.08 | 173.04 MiB | none | 197.25K/s | 43.51 MiB/s | 2.16x |
| polars | 536.773ms | 3.989s | +643% | 4.028s | 5.400s | 1.35 | 84.05 MiB | none | 72.66K/s | 16.03 MiB/s | 5.31x |
| clickhouse-server | 135.000ms | 1.376s | +920% | 1.357s | not read | not read | not read | not read | 318.52K/s | 70.26 MiB/s | 1.34x |
| rudb | 22.000ms | 74.763ms | +240% | 76.120ms | 0.000us | 0.00 | 6.00 MiB | none | 1.86M/s | 411.09 MiB/s | 0.22x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | clickhouse-server | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 1.000ms | 2.000ms | 1.000ms | 5.190ms | 1.000ms | 0.000us |
| q2 | filtered count | 0.000us | 1.000ms | 2.000ms | 3.000ms | 6.472ms | 1.000ms | 0.000us |
| q3 | three aggregates | 1.000ms | 1.000ms | 3.000ms | 2.000ms | 6.127ms | 1.000ms | 0.000us |
| q4 | average | 0.000us | 1.000ms | 2.000ms | 1.000ms | 6.744ms | 1.000ms | 0.000us |
| q5 | count distinct, high card | 2.000ms | 2.000ms | 2.000ms | 5.000ms | 12.721ms | 1.000ms | 0.000us |
| q6 | count distinct, strings | 2.000ms | 2.000ms | 2.000ms | 3.000ms | 12.337ms | 1.000ms | 0.000us |
| q7 | min and max of a date | 1.000ms | 1.000ms | 3.000ms | 1.000ms | 5.838ms | 1.000ms | 0.000us |
| q8 | group by, low card | 1.000ms | 6.000ms | 4.000ms | 4.000ms | 13.983ms | 2.000ms | 0.000us |
| q9 | group by and count distinct | 5.000ms | 3.000ms | 3.000ms | 6.000ms | 19.971ms | 2.000ms | 0.000us |
| q10 | group by, several aggregates | 4.000ms | 5.000ms | 3.000ms | 5.000ms | 25.331ms | 16.000ms | 1.000ms |
| q11 | group by a string and count distinct | 4.000ms | 3.000ms | 3.000ms | 8.000ms | 20.085ms | 2.000ms | 0.000us |
| q12 | group by two strings and count distinct | 4.000ms | 4.000ms | 3.000ms | 8.000ms | 20.187ms | 2.000ms | 0.000us |
| q13 | group by a string and top k | 3.000ms | 3.000ms | 3.000ms | 6.000ms | 14.886ms | 2.000ms | 0.000us |
| q14 | group by a string and count distinct | 4.000ms | 4.000ms | 3.000ms | 8.000ms | 20.140ms | 2.000ms | 0.000us |
| q15 | group by two columns and top k | 3.000ms | 3.000ms | 3.000ms | 6.000ms | 15.437ms | 2.000ms | 0.000us |
| q16 | group by, very high card | 2.000ms | 2.000ms | 3.000ms | 3.000ms | 17.565ms | 2.000ms | 1.000ms |
| q17 | group by two, very high card | 3.000ms | 3.000ms | 3.000ms | 4.000ms | 20.570ms | 2.000ms | 1.000ms |
| q18 | group by two, no ordering | 3.000ms | 3.000ms | 3.000ms | 4.000ms | 11.510ms | 2.000ms | 1.000ms |
| q19 | group by with an extract | 3.000ms | 3.000ms | 3.000ms | 5.000ms | 22.975ms | 2.000ms | no dialect |
| q20 | point lookup | 0.000us | 1.000ms | 3.000ms | 2.000ms | 6.220ms | 1.000ms | 0.000us |
| q21 | substring scan | 1.000ms | 1.000ms | 3.000ms | 4.000ms | 6.594ms | 1.000ms | 1.000ms |
| q22 | substring scan and group by | 1.000ms | 2.000ms | 3.000ms | 5.000ms | 13.102ms | 3.000ms | 1.000ms |
| q23 | two substring scans and group by | 1.000ms | 2.000ms | 4.000ms | 5.000ms | 19.399ms | 3.000ms | 1.000ms |
| q24 | select star and top k | 5.000ms | 9.000ms | 6.000ms | 8.000ms | 9.829ms | 4.000ms | 3.000ms |
| q25 | top k by a date | 2.000ms | 1.000ms | 3.000ms | 3.000ms | 11.252ms | 2.000ms | 0.000us |
| q26 | top k by a string | 1.000ms | 1.000ms | 2.000ms | 3.000ms | 10.506ms | 1.000ms | 0.000us |
| q27 | top k by two columns | 1.000ms | 2.000ms | 4.000ms | 4.000ms | 11.348ms | 1.000ms | 0.000us |
| q28 | group by with a string length | 2.000ms | 3.000ms | 3.000ms | 6.000ms | no dialect | 2.000ms | 1.000ms |
| q29 | group by a regular expression | 3.000ms | 4.000ms | 4.000ms | 7.000ms | no dialect | 21.000ms | 1.000ms |
| q30 | ninety sums over one column | 4.000ms | 14.000ms | 8.000ms | 10.000ms | 10.760ms | 6.000ms | 1.000ms |
| q31 | group by two and several aggregates | 2.000ms | 4.000ms | 3.000ms | 7.000ms | 15.378ms | 2.000ms | 1.000ms |
| q32 | group by a high card pair | 3.000ms | 4.000ms | 3.000ms | 7.000ms | 15.424ms | 14.000ms | 0.000us |
| q33 | group by a high card pair, unfiltered | 3.000ms | 3.000ms | 3.000ms | 5.000ms | 15.781ms | 2.000ms | no dialect |
| q34 | group by a long string | 3.000ms | 2.000ms | 3.000ms | 4.000ms | 14.552ms | 2.000ms | 1.000ms |
| q35 | group by a constant and a long string | 3.000ms | 2.000ms | 3.000ms | 5.000ms | 17.071ms | 2.000ms | 1.000ms |
| q36 | group by four expressions | 3.000ms | 4.000ms | 3.000ms | 4.000ms | no dialect | 2.000ms | 1.000ms |
| q37 | date range and group by a URL | 3.000ms | 3.000ms | 5.000ms | 7.000ms | 14.899ms | 3.000ms | 1.000ms |
| q38 | date range and group by a title | 3.000ms | 3.000ms | 5.000ms | 7.000ms | 13.867ms | 3.000ms | 1.000ms |
| q39 | date range, group by and offset | 1.000ms | 1.000ms | 5.000ms | 6.000ms | 11.584ms | 3.000ms | 1.000ms |
| q40 | date range, a case and a wide group by | 2.000ms | 4.000ms | 5.000ms | 7.000ms | 14.599ms | 3.000ms | 1.000ms |
| q41 | date range with an IN and a hash | 3.000ms | 4.000ms | 5.000ms | 7.000ms | 13.760ms | 3.000ms | 0.000us |
| q42 | date range and a deep offset | 2.000ms | 8.000ms | 5.000ms | 6.000ms | 12.779ms | 3.000ms | 1.000ms |
| q43 | minute buckets over a date range | 3.000ms | 3.000ms | 5.000ms | 6.000ms | no dialect | 3.000ms | 0.000us |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 10.409ms | 9.828ms | 0.0% | 9.828ms | 9.828ms | 9.828ms | 9.828ms | 10.000ms | 26.80 MiB | none | 101.75K/s |
| q2 | filtered count | 0.000us | 10.704ms | 9.999ms | 0.0% | 9.999ms | 9.999ms | 9.999ms | 9.999ms | 0.000us | 28.04 MiB | none | 100.01K/s |
| q3 | three aggregates | 1.000ms | 10.181ms | 10.021ms | 0.0% | 10.021ms | 10.021ms | 10.021ms | 10.021ms | 0.000us | 28.04 MiB | none | 99.79K/s |
| q4 | average | 0.000us | 9.817ms | 9.713ms | 0.0% | 9.713ms | 9.713ms | 9.713ms | 9.713ms | 10.000ms | 27.78 MiB | none | 102.95K/s |
| q5 | count distinct, high card | 2.000ms | 11.616ms | 11.522ms | 0.0% | 11.522ms | 11.522ms | 11.522ms | 11.522ms | 10.000ms | 30.04 MiB | none | 86.79K/s |
| q6 | count distinct, strings | 2.000ms | 11.157ms | 11.629ms | 0.0% | 11.629ms | 11.629ms | 11.629ms | 11.629ms | 0.000us | 29.79 MiB | none | 85.99K/s |
| q7 | min and max of a date | 1.000ms | 9.879ms | 9.810ms | 0.0% | 9.810ms | 9.810ms | 9.810ms | 9.810ms | 10.000ms | 27.05 MiB | none | 101.94K/s |
| q8 | group by, low card | 1.000ms | 10.628ms | 10.389ms | 0.0% | 10.389ms | 10.389ms | 10.389ms | 10.389ms | 0.000us | 29.05 MiB | none | 96.26K/s |
| q9 | group by and count distinct | 5.000ms | 13.132ms | 13.852ms | 0.0% | 13.852ms | 13.852ms | 13.852ms | 13.852ms | 20.000ms | 34.30 MiB | none | 72.19K/s |
| q10 | group by, several aggregates | 4.000ms | 13.494ms | 13.352ms | 0.0% | 13.352ms | 13.352ms | 13.352ms | 13.352ms | 20.000ms | 37.55 MiB | none | 74.90K/s |
| q11 | group by a string and count distinct | 4.000ms | 13.069ms | 13.051ms | 0.0% | 13.051ms | 13.051ms | 13.051ms | 13.051ms | 10.000ms | 35.05 MiB | none | 76.62K/s |
| q12 | group by two strings and count distinct | 4.000ms | 12.728ms | 13.080ms | 0.0% | 13.080ms | 13.080ms | 13.080ms | 13.080ms | 10.000ms | 34.76 MiB | none | 76.45K/s |
| q13 | group by a string and top k | 3.000ms | 11.528ms | 11.447ms | 0.0% | 11.447ms | 11.447ms | 11.447ms | 11.447ms | 10.000ms | 30.70 MiB | none | 87.36K/s |
| q14 | group by a string and count distinct | 4.000ms | 12.997ms | 13.204ms | 0.0% | 13.204ms | 13.204ms | 13.204ms | 13.204ms | 10.000ms | 36.05 MiB | none | 75.73K/s |
| q15 | group by two columns and top k | 3.000ms | 11.723ms | 12.004ms | 0.0% | 12.004ms | 12.004ms | 12.004ms | 12.004ms | 10.000ms | 31.99 MiB | none | 83.31K/s |
| q16 | group by, very high card | 2.000ms | 11.796ms | 11.526ms | 0.0% | 11.526ms | 11.526ms | 11.526ms | 11.526ms | 10.000ms | 32.38 MiB | none | 86.76K/s |
| q17 | group by two, very high card | 3.000ms | 12.138ms | 11.850ms | 0.0% | 11.850ms | 11.850ms | 11.850ms | 11.850ms | 10.000ms | 32.34 MiB | none | 84.39K/s |
| q18 | group by two, no ordering | 3.000ms | 12.851ms | 12.101ms | 0.0% | 12.101ms | 12.101ms | 12.101ms | 12.101ms | 10.000ms | 34.58 MiB | none | 82.64K/s |
| q19 | group by with an extract | 3.000ms | 11.914ms | 11.989ms | 0.0% | 11.989ms | 11.989ms | 11.989ms | 11.989ms | 10.000ms | 34.10 MiB | none | 83.41K/s |
| q20 | point lookup | 0.000us | 10.711ms | 9.975ms | 0.0% | 9.975ms | 9.975ms | 9.975ms | 9.975ms | 0.000us | 27.54 MiB | none | 100.25K/s |
| q21 | substring scan | 1.000ms | 9.900ms | 9.875ms | 0.0% | 9.875ms | 9.875ms | 9.875ms | 9.875ms | 10.000ms | 28.04 MiB | none | 101.27K/s |
| q22 | substring scan and group by | 1.000ms | 10.500ms | 11.261ms | 0.0% | 11.261ms | 11.261ms | 11.261ms | 11.261ms | 0.000us | 28.80 MiB | none | 88.80K/s |
| q23 | two substring scans and group by | 1.000ms | 10.598ms | 10.591ms | 0.0% | 10.591ms | 10.591ms | 10.591ms | 10.591ms | 0.000us | 28.55 MiB | none | 94.42K/s |
| q24 | select star and top k | 5.000ms | 13.938ms | 13.941ms | 0.0% | 13.941ms | 13.941ms | 13.941ms | 13.941ms | 10.000ms | 34.06 MiB | none | 71.73K/s |
| q25 | top k by a date | 2.000ms | 11.555ms | 11.292ms | 0.0% | 11.292ms | 11.292ms | 11.292ms | 11.292ms | 0.000us | 30.53 MiB | none | 88.56K/s |
| q26 | top k by a string | 1.000ms | 10.034ms | 10.082ms | 0.0% | 10.082ms | 10.082ms | 10.082ms | 10.082ms | 0.000us | 27.54 MiB | none | 99.19K/s |
| q27 | top k by two columns | 1.000ms | 10.174ms | 10.189ms | 0.0% | 10.189ms | 10.189ms | 10.189ms | 10.189ms | 0.000us | 28.00 MiB | none | 98.15K/s |
| q28 | group by with a string length | 2.000ms | 11.784ms | 11.858ms | 0.0% | 11.858ms | 11.858ms | 11.858ms | 11.858ms | 0.000us | 31.55 MiB | none | 84.33K/s |
| q29 | group by a regular expression | 3.000ms | 12.836ms | 12.342ms | 0.0% | 12.342ms | 12.342ms | 12.342ms | 12.342ms | 10.000ms | 31.79 MiB | none | 81.02K/s |
| q30 | ninety sums over one column | 4.000ms | 13.353ms | 13.352ms | 0.0% | 13.352ms | 13.352ms | 13.352ms | 13.352ms | 10.000ms | 31.54 MiB | none | 74.90K/s |
| q31 | group by two and several aggregates | 2.000ms | 12.210ms | 11.896ms | 0.0% | 11.896ms | 11.896ms | 11.896ms | 11.896ms | 0.000us | 33.07 MiB | none | 84.06K/s |
| q32 | group by a high card pair | 3.000ms | 11.758ms | 11.688ms | 0.0% | 11.688ms | 11.688ms | 11.688ms | 11.688ms | 10.000ms | 33.60 MiB | none | 85.56K/s |
| q33 | group by a high card pair, unfiltered | 3.000ms | 12.330ms | 11.730ms | 0.0% | 11.730ms | 11.730ms | 11.730ms | 11.730ms | 10.000ms | 33.34 MiB | none | 85.25K/s |
| q34 | group by a long string | 3.000ms | 11.966ms | 11.800ms | 0.0% | 11.800ms | 11.800ms | 11.800ms | 11.800ms | 10.000ms | 31.30 MiB | none | 84.75K/s |
| q35 | group by a constant and a long string | 3.000ms | 12.367ms | 11.849ms | 0.0% | 11.849ms | 11.849ms | 11.849ms | 11.849ms | 10.000ms | 31.55 MiB | none | 84.40K/s |
| q36 | group by four expressions | 3.000ms | 12.036ms | 11.955ms | 0.0% | 11.955ms | 11.955ms | 11.955ms | 11.955ms | 20.000ms | 32.31 MiB | none | 83.65K/s |
| q37 | date range and group by a URL | 3.000ms | 20.345ms | 11.767ms | 0.0% | 11.767ms | 11.767ms | 11.767ms | 11.767ms | 10.000ms | 30.99 MiB | none | 84.98K/s |
| q38 | date range and group by a title | 3.000ms | 11.395ms | 11.932ms | 0.0% | 11.932ms | 11.932ms | 11.932ms | 11.932ms | 10.000ms | 30.94 MiB | none | 83.81K/s |
| q39 | date range, group by and offset | 1.000ms | 10.598ms | 10.265ms | 0.0% | 10.265ms | 10.265ms | 10.265ms | 10.265ms | 0.000us | 28.74 MiB | none | 97.42K/s |
| q40 | date range, a case and a wide group by | 2.000ms | 11.780ms | 12.370ms | 0.0% | 12.370ms | 12.370ms | 12.370ms | 12.370ms | 10.000ms | 32.80 MiB | none | 80.84K/s |
| q41 | date range with an IN and a hash | 3.000ms | 11.549ms | 11.708ms | 0.0% | 11.708ms | 11.708ms | 11.708ms | 11.708ms | 10.000ms | 32.35 MiB | none | 85.41K/s |
| q42 | date range and a deep offset | 2.000ms | 11.557ms | 11.232ms | 0.0% | 11.232ms | 11.232ms | 11.232ms | 11.232ms | 0.000us | 30.55 MiB | none | 89.03K/s |
| q43 | minute buckets over a date range | 3.000ms | 11.612ms | 13.730ms | 0.0% | 13.730ms | 13.730ms | 13.730ms | 13.730ms | 10.000ms | 31.02 MiB | none | 72.83K/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 101.000ms by its own clock and 499.047ms by ours, 508.647ms cold, 320.000ms of CPU, peak 37.55 MiB, 425.74K/s and 93.91 MiB/s.

Running it cost 394% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.44x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 23.889ms | 23.218ms | 0.0% | 23.218ms | 23.218ms | 23.218ms | 23.218ms | 10.000ms | 39.70 MiB | none | 43.07K/s |
| q2 | filtered count | 1.000ms | 22.664ms | 23.633ms | 0.0% | 23.633ms | 23.633ms | 23.633ms | 23.633ms | 20.000ms | 40.57 MiB | none | 42.31K/s |
| q3 | three aggregates | 1.000ms | 23.369ms | 24.187ms | 0.0% | 24.187ms | 24.187ms | 24.187ms | 24.187ms | 20.000ms | 40.82 MiB | none | 41.34K/s |
| q4 | average | 1.000ms | 22.894ms | 22.813ms | 0.0% | 22.813ms | 22.813ms | 22.813ms | 22.813ms | 10.000ms | 40.70 MiB | none | 43.83K/s |
| q5 | count distinct, high card | 2.000ms | 24.193ms | 24.145ms | 0.0% | 24.145ms | 24.145ms | 24.145ms | 24.145ms | 20.000ms | 42.81 MiB | 32.00 KiB | 41.42K/s |
| q6 | count distinct, strings | 2.000ms | 23.752ms | 24.177ms | 0.0% | 24.177ms | 24.177ms | 24.177ms | 24.177ms | 20.000ms | 42.02 MiB | none | 41.36K/s |
| q7 | min and max of a date | 1.000ms | 24.442ms | 23.229ms | 0.0% | 23.229ms | 23.229ms | 23.229ms | 23.229ms | 10.000ms | 40.14 MiB | none | 43.05K/s |
| q8 | group by, low card | 6.000ms | 28.014ms | 27.610ms | 0.0% | 27.610ms | 27.610ms | 27.610ms | 27.610ms | 30.000ms | 43.28 MiB | none | 36.22K/s |
| q9 | group by and count distinct | 3.000ms | 25.263ms | 26.484ms | 0.0% | 26.484ms | 26.484ms | 26.484ms | 26.484ms | 30.000ms | 47.23 MiB | none | 37.76K/s |
| q10 | group by, several aggregates | 5.000ms | 27.401ms | 26.513ms | 0.0% | 26.513ms | 26.513ms | 26.513ms | 26.513ms | 20.000ms | 48.80 MiB | 72.00 KiB | 37.72K/s |
| q11 | group by a string and count distinct | 3.000ms | 25.436ms | 25.460ms | 0.0% | 25.460ms | 25.460ms | 25.460ms | 25.460ms | 20.000ms | 46.54 MiB | none | 39.28K/s |
| q12 | group by two strings and count distinct | 4.000ms | 25.805ms | 26.847ms | 0.0% | 26.847ms | 26.847ms | 26.847ms | 26.847ms | 20.000ms | 46.57 MiB | none | 37.25K/s |
| q13 | group by a string and top k | 3.000ms | 24.630ms | 24.691ms | 0.0% | 24.691ms | 24.691ms | 24.691ms | 24.691ms | 20.000ms | 42.28 MiB | none | 40.50K/s |
| q14 | group by a string and count distinct | 4.000ms | 25.179ms | 25.745ms | 0.0% | 25.745ms | 25.745ms | 25.745ms | 25.745ms | 20.000ms | 47.29 MiB | none | 38.84K/s |
| q15 | group by two columns and top k | 3.000ms | 23.992ms | 24.484ms | 0.0% | 24.484ms | 24.484ms | 24.484ms | 24.484ms | 20.000ms | 43.04 MiB | none | 40.84K/s |
| q16 | group by, very high card | 2.000ms | 25.542ms | 24.295ms | 0.0% | 24.295ms | 24.295ms | 24.295ms | 24.295ms | 20.000ms | 43.17 MiB | none | 41.16K/s |
| q17 | group by two, very high card | 3.000ms | 25.016ms | 25.191ms | 0.0% | 25.191ms | 25.191ms | 25.191ms | 25.191ms | 20.000ms | 44.42 MiB | none | 39.70K/s |
| q18 | group by two, no ordering | 3.000ms | 24.902ms | 24.074ms | 0.0% | 24.074ms | 24.074ms | 24.074ms | 24.074ms | 20.000ms | 43.86 MiB | none | 41.54K/s |
| q19 | group by with an extract | 3.000ms | 25.917ms | 25.184ms | 0.0% | 25.184ms | 25.184ms | 25.184ms | 25.184ms | 30.000ms | 45.28 MiB | none | 39.71K/s |
| q20 | point lookup | 1.000ms | 22.707ms | 23.704ms | 0.0% | 23.704ms | 23.704ms | 23.704ms | 23.704ms | 10.000ms | 39.73 MiB | none | 42.19K/s |
| q21 | substring scan | 1.000ms | 22.972ms | 23.546ms | 0.0% | 23.546ms | 23.546ms | 23.546ms | 23.546ms | 20.000ms | 40.27 MiB | none | 42.47K/s |
| q22 | substring scan and group by | 2.000ms | 24.470ms | 23.691ms | 0.0% | 23.691ms | 23.691ms | 23.691ms | 23.691ms | 20.000ms | 41.53 MiB | none | 42.21K/s |
| q23 | two substring scans and group by | 2.000ms | 25.088ms | 25.344ms | 0.0% | 25.344ms | 25.344ms | 25.344ms | 25.344ms | 20.000ms | 43.09 MiB | none | 39.46K/s |
| q24 | select star and top k | 9.000ms | 31.430ms | 31.224ms | 0.0% | 31.224ms | 31.224ms | 31.224ms | 31.224ms | 30.000ms | 48.09 MiB | none | 32.03K/s |
| q25 | top k by a date | 1.000ms | 24.386ms | 23.749ms | 0.0% | 23.749ms | 23.749ms | 23.749ms | 23.749ms | 20.000ms | 40.77 MiB | none | 42.11K/s |
| q26 | top k by a string | 1.000ms | 23.034ms | 23.697ms | 0.0% | 23.697ms | 23.697ms | 23.697ms | 23.697ms | 20.000ms | 40.52 MiB | none | 42.20K/s |
| q27 | top k by two columns | 2.000ms | 24.044ms | 23.327ms | 0.0% | 23.327ms | 23.327ms | 23.327ms | 23.327ms | 20.000ms | 41.02 MiB | none | 42.87K/s |
| q28 | group by with a string length | 3.000ms | 25.155ms | 25.772ms | 0.0% | 25.772ms | 25.772ms | 25.772ms | 25.772ms | 30.000ms | 42.79 MiB | none | 38.80K/s |
| q29 | group by a regular expression | 4.000ms | 26.213ms | 26.169ms | 0.0% | 26.169ms | 26.169ms | 26.169ms | 26.169ms | 30.000ms | 43.99 MiB | none | 38.21K/s |
| q30 | ninety sums over one column | 14.000ms | 37.435ms | 36.542ms | 0.0% | 36.542ms | 36.542ms | 36.542ms | 36.542ms | 30.000ms | 53.02 MiB | none | 27.37K/s |
| q31 | group by two and several aggregates | 4.000ms | 25.438ms | 25.273ms | 0.0% | 25.273ms | 25.273ms | 25.273ms | 25.273ms | 20.000ms | 45.32 MiB | none | 39.57K/s |
| q32 | group by a high card pair | 4.000ms | 24.751ms | 25.183ms | 0.0% | 25.183ms | 25.183ms | 25.183ms | 25.183ms | 20.000ms | 45.07 MiB | none | 39.71K/s |
| q33 | group by a high card pair, unfiltered | 3.000ms | 24.398ms | 26.240ms | 0.0% | 26.240ms | 26.240ms | 26.240ms | 26.240ms | 20.000ms | 44.82 MiB | none | 38.11K/s |
| q34 | group by a long string | 2.000ms | 25.101ms | 23.886ms | 0.0% | 23.886ms | 23.886ms | 23.886ms | 23.886ms | 20.000ms | 42.77 MiB | none | 41.87K/s |
| q35 | group by a constant and a long string | 2.000ms | 24.816ms | 23.846ms | 0.0% | 23.846ms | 23.846ms | 23.846ms | 23.846ms | 20.000ms | 43.03 MiB | none | 41.94K/s |
| q36 | group by four expressions | 4.000ms | 24.512ms | 25.042ms | 0.0% | 25.042ms | 25.042ms | 25.042ms | 25.042ms | 20.000ms | 45.48 MiB | none | 39.93K/s |
| q37 | date range and group by a URL | 3.000ms | 25.869ms | 24.804ms | 0.0% | 24.804ms | 24.804ms | 24.804ms | 24.804ms | 20.000ms | 43.16 MiB | none | 40.32K/s |
| q38 | date range and group by a title | 3.000ms | 26.222ms | 25.163ms | 0.0% | 25.163ms | 25.163ms | 25.163ms | 25.163ms | 20.000ms | 43.53 MiB | none | 39.74K/s |
| q39 | date range, group by and offset | 1.000ms | 25.180ms | 24.038ms | 0.0% | 24.038ms | 24.038ms | 24.038ms | 24.038ms | 20.000ms | 41.03 MiB | none | 41.60K/s |
| q40 | date range, a case and a wide group by | 4.000ms | 25.295ms | 25.807ms | 0.0% | 25.807ms | 25.807ms | 25.807ms | 25.807ms | 20.000ms | 44.79 MiB | none | 38.75K/s |
| q41 | date range with an IN and a hash | 4.000ms | 25.693ms | 24.942ms | 0.0% | 24.942ms | 24.942ms | 24.942ms | 24.942ms | 20.000ms | 45.05 MiB | none | 40.09K/s |
| q42 | date range and a deep offset | 8.000ms | 28.777ms | 29.570ms | 0.0% | 29.570ms | 29.570ms | 29.570ms | 29.570ms | 30.000ms | 44.79 MiB | none | 33.82K/s |
| q43 | minute buckets over a date range | 3.000ms | 25.013ms | 25.217ms | 0.0% | 25.217ms | 25.217ms | 25.217ms | 25.217ms | 20.000ms | 42.27 MiB | none | 39.66K/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 136.000ms by its own clock and 1.088s by ours, 1.090s cold, 900.000ms of CPU, peak 53.02 MiB, 316.18K/s and 69.74 MiB/s.

Running it cost 700% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.60x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 2.000ms | 65.577ms | 52.247ms | 0.0% | 52.247ms | 52.247ms | 52.247ms | 52.247ms | 40.000ms | 199.66 MiB | none | 19.14K/s |
| q2 | filtered count | 2.000ms | 47.279ms | 46.846ms | 0.0% | 46.846ms | 46.846ms | 46.846ms | 46.846ms | 40.000ms | 200.68 MiB | none | 21.35K/s |
| q3 | three aggregates | 3.000ms | 47.827ms | 47.666ms | 0.0% | 47.666ms | 47.666ms | 47.666ms | 47.666ms | 50.000ms | 202.48 MiB | none | 20.98K/s |
| q4 | average | 2.000ms | 47.760ms | 52.389ms | 0.0% | 52.389ms | 52.389ms | 52.389ms | 52.389ms | 50.000ms | 202.22 MiB | none | 19.09K/s |
| q5 | count distinct, high card | 2.000ms | 52.621ms | 48.457ms | 0.0% | 48.457ms | 48.457ms | 48.457ms | 48.457ms | 50.000ms | 202.16 MiB | none | 20.64K/s |
| q6 | count distinct, strings | 2.000ms | 48.215ms | 48.051ms | 0.0% | 48.051ms | 48.051ms | 48.051ms | 48.051ms | 50.000ms | 202.46 MiB | none | 20.81K/s |
| q7 | min and max of a date | 3.000ms | 47.848ms | 50.936ms | 0.0% | 50.936ms | 50.936ms | 50.936ms | 50.936ms | 50.000ms | 201.74 MiB | none | 19.63K/s |
| q8 | group by, low card | 4.000ms | 50.682ms | 52.906ms | 0.0% | 52.906ms | 52.906ms | 52.906ms | 52.906ms | 50.000ms | 204.47 MiB | 1.09 MiB | 18.90K/s |
| q9 | group by and count distinct | 3.000ms | 48.116ms | 47.370ms | 0.0% | 47.370ms | 47.370ms | 47.370ms | 47.370ms | 40.000ms | 203.60 MiB | none | 21.11K/s |
| q10 | group by, several aggregates | 3.000ms | 49.564ms | 46.704ms | 0.0% | 46.704ms | 46.704ms | 46.704ms | 46.704ms | 50.000ms | 204.70 MiB | none | 21.41K/s |
| q11 | group by a string and count distinct | 3.000ms | 48.316ms | 47.917ms | 0.0% | 47.917ms | 47.917ms | 47.917ms | 47.917ms | 50.000ms | 205.01 MiB | none | 20.87K/s |
| q12 | group by two strings and count distinct | 3.000ms | 47.887ms | 49.061ms | 0.0% | 49.061ms | 49.061ms | 49.061ms | 49.061ms | 50.000ms | 205.38 MiB | none | 20.38K/s |
| q13 | group by a string and top k | 3.000ms | 48.926ms | 46.785ms | 0.0% | 46.785ms | 46.785ms | 46.785ms | 46.785ms | 50.000ms | 204.89 MiB | none | 21.37K/s |
| q14 | group by a string and count distinct | 3.000ms | 52.391ms | 48.484ms | 0.0% | 48.484ms | 48.484ms | 48.484ms | 48.484ms | 50.000ms | 205.13 MiB | none | 20.63K/s |
| q15 | group by two columns and top k | 3.000ms | 49.093ms | 48.212ms | 0.0% | 48.212ms | 48.212ms | 48.212ms | 48.212ms | 40.000ms | 205.73 MiB | none | 20.74K/s |
| q16 | group by, very high card | 3.000ms | 48.124ms | 49.330ms | 0.0% | 49.330ms | 49.330ms | 49.330ms | 49.330ms | 50.000ms | 204.14 MiB | none | 20.27K/s |
| q17 | group by two, very high card | 3.000ms | 55.265ms | 49.595ms | 0.0% | 49.595ms | 49.595ms | 49.595ms | 49.595ms | 50.000ms | 204.38 MiB | none | 20.16K/s |
| q18 | group by two, no ordering | 3.000ms | 48.439ms | 49.190ms | 0.0% | 49.190ms | 49.190ms | 49.190ms | 49.190ms | 50.000ms | 204.11 MiB | none | 20.33K/s |
| q19 | group by with an extract | 3.000ms | 47.547ms | 47.378ms | 0.0% | 47.378ms | 47.378ms | 47.378ms | 47.378ms | 50.000ms | 205.14 MiB | none | 21.11K/s |
| q20 | point lookup | 3.000ms | 49.833ms | 51.223ms | 0.0% | 51.223ms | 51.223ms | 51.223ms | 51.223ms | 50.000ms | 202.25 MiB | none | 19.52K/s |
| q21 | substring scan | 3.000ms | 52.838ms | 83.544ms | 0.0% | 83.544ms | 83.544ms | 83.544ms | 83.544ms | 50.000ms | 203.40 MiB | none | 11.97K/s |
| q22 | substring scan and group by | 3.000ms | 49.129ms | 48.430ms | 0.0% | 48.430ms | 48.430ms | 48.430ms | 48.430ms | 50.000ms | 205.07 MiB | none | 20.65K/s |
| q23 | two substring scans and group by | 4.000ms | 49.239ms | 52.159ms | 0.0% | 52.159ms | 52.159ms | 52.159ms | 52.159ms | 50.000ms | 205.23 MiB | none | 19.17K/s |
| q24 | select star and top k | 6.000ms | 57.886ms | 53.408ms | 0.0% | 53.408ms | 53.408ms | 53.408ms | 53.408ms | 60.000ms | 204.52 MiB | 4.00 KiB | 18.72K/s |
| q25 | top k by a date | 3.000ms | 48.872ms | 48.602ms | 0.0% | 48.602ms | 48.602ms | 48.602ms | 48.602ms | 40.000ms | 204.11 MiB | none | 20.58K/s |
| q26 | top k by a string | 2.000ms | 47.459ms | 46.734ms | 0.0% | 46.734ms | 46.734ms | 46.734ms | 46.734ms | 50.000ms | 203.06 MiB | none | 21.40K/s |
| q27 | top k by two columns | 4.000ms | 48.653ms | 48.723ms | 0.0% | 48.723ms | 48.723ms | 48.723ms | 48.723ms | 40.000ms | 204.36 MiB | none | 20.52K/s |
| q28 | group by with a string length | 3.000ms | 48.277ms | 47.408ms | 0.0% | 47.408ms | 47.408ms | 47.408ms | 47.408ms | 50.000ms | 205.14 MiB | none | 21.09K/s |
| q29 | group by a regular expression | 4.000ms | 49.370ms | 49.359ms | 0.0% | 49.359ms | 49.359ms | 49.359ms | 49.359ms | 50.000ms | 206.14 MiB | none | 20.26K/s |
| q30 | ninety sums over one column | 8.000ms | 51.950ms | 59.066ms | 0.0% | 59.066ms | 59.066ms | 59.066ms | 59.066ms | 60.000ms | 205.59 MiB | none | 16.93K/s |
| q31 | group by two and several aggregates | 3.000ms | 50.509ms | 51.150ms | 0.0% | 51.150ms | 51.150ms | 51.150ms | 51.150ms | 50.000ms | 205.80 MiB | none | 19.55K/s |
| q32 | group by a high card pair | 3.000ms | 47.747ms | 53.546ms | 0.0% | 53.546ms | 53.546ms | 53.546ms | 53.546ms | 60.000ms | 206.39 MiB | none | 18.68K/s |
| q33 | group by a high card pair, unfiltered | 3.000ms | 48.491ms | 48.322ms | 0.0% | 48.322ms | 48.322ms | 48.322ms | 48.322ms | 50.000ms | 205.31 MiB | none | 20.69K/s |
| q34 | group by a long string | 3.000ms | 46.401ms | 48.037ms | 0.0% | 48.037ms | 48.037ms | 48.037ms | 48.037ms | 50.000ms | 204.39 MiB | none | 20.82K/s |
| q35 | group by a constant and a long string | 3.000ms | 47.246ms | 49.668ms | 0.0% | 49.668ms | 49.668ms | 49.668ms | 49.668ms | 50.000ms | 204.89 MiB | none | 20.13K/s |
| q36 | group by four expressions | 3.000ms | 51.989ms | 48.987ms | 0.0% | 48.987ms | 48.987ms | 48.987ms | 48.987ms | 50.000ms | 204.39 MiB | none | 20.41K/s |
| q37 | date range and group by a URL | 5.000ms | 50.174ms | 50.550ms | 0.0% | 50.550ms | 50.550ms | 50.550ms | 50.550ms | 50.000ms | 207.14 MiB | none | 19.78K/s |
| q38 | date range and group by a title | 5.000ms | 50.409ms | 49.789ms | 0.0% | 49.789ms | 49.789ms | 49.789ms | 49.789ms | 50.000ms | 207.32 MiB | none | 20.08K/s |
| q39 | date range, group by and offset | 5.000ms | 50.058ms | 49.639ms | 0.0% | 49.639ms | 49.639ms | 49.639ms | 49.639ms | 50.000ms | 205.82 MiB | none | 20.15K/s |
| q40 | date range, a case and a wide group by | 5.000ms | 50.997ms | 51.656ms | 0.0% | 51.656ms | 51.656ms | 51.656ms | 51.656ms | 50.000ms | 207.45 MiB | none | 19.36K/s |
| q41 | date range with an IN and a hash | 5.000ms | 52.352ms | 49.736ms | 0.0% | 49.736ms | 49.736ms | 49.736ms | 49.736ms | 50.000ms | 208.08 MiB | 3.41 MiB | 20.11K/s |
| q42 | date range and a deep offset | 5.000ms | 55.902ms | 50.723ms | 0.0% | 50.723ms | 50.723ms | 50.723ms | 50.723ms | 50.000ms | 207.37 MiB | none | 19.71K/s |
| q43 | minute buckets over a date range | 5.000ms | 49.537ms | 50.108ms | 0.0% | 50.108ms | 50.108ms | 50.108ms | 50.108ms | 50.000ms | 206.62 MiB | none | 19.96K/s |

clickhouse-local 26.9.1.1162 over 43 of 43 queries. Total 149.000ms by its own clock and 2.170s by ours, 2.157s cold, 2.120s of CPU, peak 208.08 MiB, 288.59K/s and 63.66 MiB/s.

Running it cost 1356% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.79x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 49.803ms | 12.354ms | 0.0% | 12.354ms | 12.354ms | 12.354ms | 12.354ms | 10.000ms | 80.32 MiB | 71.41 MiB | 80.95K/s |
| q2 | filtered count | 3.000ms | 14.773ms | 14.401ms | 0.0% | 14.401ms | 14.401ms | 14.401ms | 14.401ms | 10.000ms | 92.48 MiB | 32.00 KiB | 69.44K/s |
| q3 | three aggregates | 2.000ms | 14.582ms | 13.201ms | 0.0% | 13.201ms | 13.201ms | 13.201ms | 13.201ms | 10.000ms | 81.79 MiB | 12.00 KiB | 75.75K/s |
| q4 | average | 1.000ms | 12.124ms | 12.595ms | 0.0% | 12.595ms | 12.595ms | 12.595ms | 12.595ms | 10.000ms | 80.30 MiB | none | 79.40K/s |
| q5 | count distinct, high card | 5.000ms | 15.384ms | 17.491ms | 0.0% | 17.491ms | 17.491ms | 17.491ms | 17.491ms | 20.000ms | 113.88 MiB | none | 57.17K/s |
| q6 | count distinct, strings | 3.000ms | 15.274ms | 14.502ms | 0.0% | 14.502ms | 14.502ms | 14.502ms | 14.502ms | 10.000ms | 117.29 MiB | none | 68.96K/s |
| q7 | min and max of a date | 1.000ms | 12.440ms | 12.038ms | 0.0% | 12.038ms | 12.038ms | 12.038ms | 12.038ms | 10.000ms | 79.50 MiB | 4.00 KiB | 83.07K/s |
| q8 | group by, low card | 4.000ms | 15.170ms | 16.176ms | 0.0% | 16.176ms | 16.176ms | 16.176ms | 16.176ms | 10.000ms | 98.60 MiB | none | 61.82K/s |
| q9 | group by and count distinct | 6.000ms | 16.535ms | 17.661ms | 0.0% | 17.661ms | 17.661ms | 17.661ms | 17.661ms | 30.000ms | 123.44 MiB | none | 56.62K/s |
| q10 | group by, several aggregates | 5.000ms | 16.520ms | 15.876ms | 0.0% | 15.876ms | 15.876ms | 15.876ms | 15.876ms | 20.000ms | 119.13 MiB | none | 62.99K/s |
| q11 | group by a string and count distinct | 8.000ms | 18.394ms | 20.377ms | 0.0% | 20.377ms | 20.377ms | 20.377ms | 20.377ms | 30.000ms | 142.65 MiB | none | 49.07K/s |
| q12 | group by two strings and count distinct | 8.000ms | 21.171ms | 21.250ms | 0.0% | 21.250ms | 21.250ms | 21.250ms | 21.250ms | 30.000ms | 149.50 MiB | none | 47.06K/s |
| q13 | group by a string and top k | 6.000ms | 18.981ms | 17.221ms | 0.0% | 17.221ms | 17.221ms | 17.221ms | 17.221ms | 30.000ms | 134.40 MiB | none | 58.07K/s |
| q14 | group by a string and count distinct | 8.000ms | 20.803ms | 19.920ms | 0.0% | 19.920ms | 19.920ms | 19.920ms | 19.920ms | 60.000ms | 173.04 MiB | none | 50.20K/s |
| q15 | group by two columns and top k | 6.000ms | 19.134ms | 17.734ms | 0.0% | 17.734ms | 17.734ms | 17.734ms | 17.734ms | 30.000ms | 127.03 MiB | none | 56.39K/s |
| q16 | group by, very high card | 3.000ms | 14.209ms | 13.919ms | 0.0% | 13.919ms | 13.919ms | 13.919ms | 13.919ms | 10.000ms | 103.87 MiB | none | 71.84K/s |
| q17 | group by two, very high card | 4.000ms | 15.861ms | 15.515ms | 0.0% | 15.515ms | 15.515ms | 15.515ms | 15.515ms | 10.000ms | 116.98 MiB | none | 64.45K/s |
| q18 | group by two, no ordering | 4.000ms | 15.826ms | 14.538ms | 0.0% | 14.538ms | 14.538ms | 14.538ms | 14.538ms | 20.000ms | 115.05 MiB | none | 68.79K/s |
| q19 | group by with an extract | 5.000ms | 16.409ms | 16.154ms | 0.0% | 16.154ms | 16.154ms | 16.154ms | 16.154ms | 20.000ms | 117.74 MiB | 32.00 KiB | 61.90K/s |
| q20 | point lookup | 2.000ms | 13.956ms | 13.995ms | 0.0% | 13.995ms | 13.995ms | 13.995ms | 13.995ms | 10.000ms | 93.32 MiB | none | 71.45K/s |
| q21 | substring scan | 4.000ms | 14.428ms | 15.422ms | 0.0% | 15.422ms | 15.422ms | 15.422ms | 15.422ms | 20.000ms | 105.86 MiB | none | 64.84K/s |
| q22 | substring scan and group by | 5.000ms | 15.736ms | 16.420ms | 0.0% | 16.420ms | 16.420ms | 16.420ms | 16.420ms | 20.000ms | 115.23 MiB | none | 60.90K/s |
| q23 | two substring scans and group by | 5.000ms | 18.100ms | 17.008ms | 0.0% | 17.008ms | 17.008ms | 17.008ms | 17.008ms | 10.000ms | 120.62 MiB | none | 58.80K/s |
| q24 | select star and top k | 8.000ms | 20.166ms | 19.678ms | 0.0% | 19.678ms | 19.678ms | 19.678ms | 19.678ms | 10.000ms | 115.93 MiB | none | 50.82K/s |
| q25 | top k by a date | 3.000ms | 14.528ms | 14.562ms | 0.0% | 14.562ms | 14.562ms | 14.562ms | 14.562ms | 10.000ms | 102.14 MiB | none | 68.67K/s |
| q26 | top k by a string | 3.000ms | 14.328ms | 14.508ms | 0.0% | 14.508ms | 14.508ms | 14.508ms | 14.508ms | 20.000ms | 99.61 MiB | none | 68.93K/s |
| q27 | top k by two columns | 4.000ms | 14.398ms | 14.365ms | 0.0% | 14.365ms | 14.365ms | 14.365ms | 14.365ms | 10.000ms | 103.00 MiB | none | 69.61K/s |
| q28 | group by with a string length | 6.000ms | 17.535ms | 17.765ms | 0.0% | 17.765ms | 17.765ms | 17.765ms | 17.765ms | 20.000ms | 125.84 MiB | none | 56.29K/s |
| q29 | group by a regular expression | 7.000ms | 20.350ms | 19.308ms | 0.0% | 19.308ms | 19.308ms | 19.308ms | 19.308ms | 20.000ms | 145.23 MiB | none | 51.79K/s |
| q30 | ninety sums over one column | 10.000ms | 22.520ms | 21.278ms | 0.0% | 21.278ms | 21.278ms | 21.278ms | 21.278ms | 10.000ms | 88.86 MiB | 52.00 KiB | 47.00K/s |
| q31 | group by two and several aggregates | 7.000ms | 17.178ms | 18.565ms | 0.0% | 18.565ms | 18.565ms | 18.565ms | 18.565ms | 20.000ms | 129.52 MiB | none | 53.86K/s |
| q32 | group by a high card pair | 7.000ms | 19.245ms | 18.040ms | 0.0% | 18.040ms | 18.040ms | 18.040ms | 18.040ms | 20.000ms | 131.96 MiB | none | 55.43K/s |
| q33 | group by a high card pair, unfiltered | 5.000ms | 15.846ms | 15.545ms | 0.0% | 15.545ms | 15.545ms | 15.545ms | 15.545ms | 20.000ms | 119.49 MiB | none | 64.33K/s |
| q34 | group by a long string | 4.000ms | 16.464ms | 15.781ms | 0.0% | 15.781ms | 15.781ms | 15.781ms | 15.781ms | 10.000ms | 126.44 MiB | none | 63.37K/s |
| q35 | group by a constant and a long string | 5.000ms | 17.058ms | 15.775ms | 0.0% | 15.775ms | 15.775ms | 15.775ms | 15.775ms | 10.000ms | 123.59 MiB | none | 63.39K/s |
| q36 | group by four expressions | 4.000ms | 15.312ms | 15.497ms | 0.0% | 15.497ms | 15.497ms | 15.497ms | 15.497ms | 20.000ms | 106.77 MiB | none | 64.53K/s |
| q37 | date range and group by a URL | 7.000ms | 17.009ms | 18.748ms | 0.0% | 18.748ms | 18.748ms | 18.748ms | 18.748ms | 30.000ms | 128.98 MiB | none | 53.34K/s |
| q38 | date range and group by a title | 7.000ms | 17.841ms | 17.339ms | 0.0% | 17.339ms | 17.339ms | 17.339ms | 17.339ms | 20.000ms | 122.55 MiB | none | 57.67K/s |
| q39 | date range, group by and offset | 6.000ms | 16.498ms | 17.550ms | 0.0% | 17.550ms | 17.550ms | 17.550ms | 17.550ms | 20.000ms | 117.53 MiB | none | 56.98K/s |
| q40 | date range, a case and a wide group by | 7.000ms | 18.210ms | 18.369ms | 0.0% | 18.369ms | 18.369ms | 18.369ms | 18.369ms | 20.000ms | 123.52 MiB | none | 54.44K/s |
| q41 | date range with an IN and a hash | 7.000ms | 16.854ms | 17.863ms | 0.0% | 17.863ms | 17.863ms | 17.863ms | 17.863ms | 20.000ms | 115.86 MiB | none | 55.98K/s |
| q42 | date range and a deep offset | 6.000ms | 17.699ms | 17.435ms | 0.0% | 17.435ms | 17.435ms | 17.435ms | 17.435ms | 10.000ms | 108.98 MiB | none | 57.36K/s |
| q43 | minute buckets over a date range | 6.000ms | 17.844ms | 16.975ms | 0.0% | 16.975ms | 16.975ms | 16.975ms | 16.975ms | 10.000ms | 114.97 MiB | none | 58.91K/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 218.000ms by its own clock and 710.714ms by ours, 752.496ms cold, 770.000ms of CPU, peak 173.04 MiB, 197.25K/s and 43.51 MiB/s.

Running it cost 226% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.77x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 5.190ms | 113.736ms | 90.127ms | 0.0% | 90.127ms | 90.127ms | 90.127ms | 90.127ms | 120.000ms | 61.41 MiB | 34.15 MiB | 11.10K/s |
| q2 | filtered count | 6.472ms | 94.180ms | 111.276ms | 0.0% | 111.276ms | 111.276ms | 111.276ms | 111.276ms | 360.000ms | 66.09 MiB | 312.00 KiB | 8.99K/s |
| q3 | three aggregates | 6.127ms | 93.008ms | 94.907ms | 0.0% | 94.907ms | 94.907ms | 94.907ms | 94.907ms | 90.000ms | 65.30 MiB | 736.00 KiB | 10.54K/s |
| q4 | average | 6.744ms | 100.320ms | 91.400ms | 0.0% | 91.400ms | 91.400ms | 91.400ms | 91.400ms | 90.000ms | 63.37 MiB | none | 10.94K/s |
| q5 | count distinct, high card | 12.721ms | 100.828ms | 102.060ms | 0.0% | 102.060ms | 102.060ms | 102.060ms | 102.060ms | 130.000ms | 73.75 MiB | none | 9.80K/s |
| q6 | count distinct, strings | 12.337ms | 102.007ms | 100.695ms | 0.0% | 100.695ms | 100.695ms | 100.695ms | 100.695ms | 150.000ms | 73.49 MiB | none | 9.93K/s |
| q7 | min and max of a date | 5.838ms | 89.919ms | 91.262ms | 0.0% | 91.262ms | 91.262ms | 91.262ms | 91.262ms | 90.000ms | 64.05 MiB | none | 10.96K/s |
| q8 | group by, low card | 13.983ms | 99.984ms | 98.859ms | 0.0% | 98.859ms | 98.859ms | 98.859ms | 98.859ms | 110.000ms | 76.04 MiB | none | 10.12K/s |
| q9 | group by and count distinct | 19.971ms | 110.046ms | 110.145ms | 0.0% | 110.145ms | 110.145ms | 110.145ms | 110.145ms | 150.000ms | 80.91 MiB | none | 9.08K/s |
| q10 | group by, several aggregates | 25.331ms | 115.201ms | 113.727ms | 0.0% | 113.727ms | 113.727ms | 113.727ms | 113.727ms | 170.000ms | 83.38 MiB | none | 8.79K/s |
| q11 | group by a string and count distinct | 20.085ms | 108.297ms | 111.646ms | 0.0% | 111.646ms | 111.646ms | 111.646ms | 111.646ms | 140.000ms | 82.27 MiB | none | 8.96K/s |
| q12 | group by two strings and count distinct | 20.187ms | 106.519ms | 110.986ms | 0.0% | 110.986ms | 110.986ms | 110.986ms | 110.986ms | 160.000ms | 82.35 MiB | none | 9.01K/s |
| q13 | group by a string and top k | 14.886ms | 107.109ms | 104.022ms | 0.0% | 104.022ms | 104.022ms | 104.022ms | 104.022ms | 130.000ms | 76.27 MiB | none | 9.61K/s |
| q14 | group by a string and count distinct | 20.140ms | 110.320ms | 107.132ms | 0.0% | 107.132ms | 107.132ms | 107.132ms | 107.132ms | 140.000ms | 82.19 MiB | none | 9.33K/s |
| q15 | group by two columns and top k | 15.437ms | 101.602ms | 105.900ms | 0.0% | 105.900ms | 105.900ms | 105.900ms | 105.900ms | 170.000ms | 77.93 MiB | none | 9.44K/s |
| q16 | group by, very high card | 17.565ms | 105.153ms | 103.553ms | 0.0% | 103.553ms | 103.553ms | 103.553ms | 103.553ms | 160.000ms | 75.04 MiB | none | 9.66K/s |
| q17 | group by two, very high card | 20.570ms | 108.486ms | 110.513ms | 0.0% | 110.513ms | 110.513ms | 110.513ms | 110.513ms | 160.000ms | 76.24 MiB | none | 9.05K/s |
| q18 | group by two, no ordering | 11.510ms | 101.625ms | 99.976ms | 0.0% | 99.976ms | 99.976ms | 99.976ms | 99.976ms | 130.000ms | 73.23 MiB | none | 10.00K/s |
| q19 | group by with an extract | 22.975ms | 111.526ms | 111.510ms | 0.0% | 111.510ms | 111.510ms | 111.510ms | 111.510ms | 150.000ms | 78.60 MiB | none | 8.97K/s |
| q20 | point lookup | 6.220ms | 94.152ms | 93.571ms | 0.0% | 93.571ms | 93.571ms | 93.571ms | 93.571ms | 90.000ms | 62.64 MiB | 12.00 KiB | 10.69K/s |
| q21 | substring scan | 6.594ms | 98.639ms | 91.715ms | 0.0% | 91.715ms | 91.715ms | 91.715ms | 91.715ms | 100.000ms | 67.48 MiB | 8.80 MiB | 10.90K/s |
| q22 | substring scan and group by | 13.102ms | 121.074ms | 103.472ms | 0.0% | 103.472ms | 103.472ms | 103.472ms | 103.472ms | 120.000ms | 78.52 MiB | none | 9.66K/s |
| q23 | two substring scans and group by | 19.399ms | 109.412ms | 108.686ms | 0.0% | 108.686ms | 108.686ms | 108.686ms | 108.686ms | 140.000ms | 84.05 MiB | none | 9.20K/s |
| q24 | select star and top k | 9.829ms | 96.931ms | 103.274ms | 0.0% | 103.274ms | 103.274ms | 103.274ms | 103.274ms | 130.000ms | 74.80 MiB | none | 9.68K/s |
| q25 | top k by a date | 11.252ms | 99.445ms | 100.065ms | 0.0% | 100.065ms | 100.065ms | 100.065ms | 100.065ms | 120.000ms | 72.72 MiB | none | 9.99K/s |
| q26 | top k by a string | 10.506ms | 100.266ms | 100.546ms | 0.0% | 100.546ms | 100.546ms | 100.546ms | 100.546ms | 140.000ms | 70.75 MiB | none | 9.95K/s |
| q27 | top k by two columns | 11.348ms | 101.290ms | 99.207ms | 0.0% | 99.207ms | 99.207ms | 99.207ms | 99.207ms | 110.000ms | 71.52 MiB | none | 10.08K/s |
| q30 | ninety sums over one column | 10.760ms | 98.334ms | 99.949ms | 0.0% | 99.949ms | 99.949ms | 99.949ms | 99.949ms | 110.000ms | 67.45 MiB | none | 10.01K/s |
| q31 | group by two and several aggregates | 15.378ms | 104.916ms | 102.896ms | 0.0% | 102.896ms | 102.896ms | 102.896ms | 102.896ms | 140.000ms | 78.75 MiB | none | 9.72K/s |
| q32 | group by a high card pair | 15.424ms | 101.773ms | 99.413ms | 0.0% | 99.413ms | 99.413ms | 99.413ms | 99.413ms | 130.000ms | 78.92 MiB | none | 10.06K/s |
| q33 | group by a high card pair, unfiltered | 15.781ms | 102.027ms | 103.108ms | 0.0% | 103.108ms | 103.108ms | 103.108ms | 103.108ms | 130.000ms | 77.91 MiB | none | 9.70K/s |
| q34 | group by a long string | 14.552ms | 101.320ms | 99.734ms | 0.0% | 99.734ms | 99.734ms | 99.734ms | 99.734ms | 130.000ms | 74.64 MiB | none | 10.03K/s |
| q35 | group by a constant and a long string | 17.071ms | 107.988ms | 104.135ms | 0.0% | 104.135ms | 104.135ms | 104.135ms | 104.135ms | 140.000ms | 75.39 MiB | none | 9.60K/s |
| q37 | date range and group by a URL | 14.899ms | 104.542ms | 105.770ms | 0.0% | 105.770ms | 105.770ms | 105.770ms | 105.770ms | 190.000ms | 79.91 MiB | 20.00 KiB | 9.45K/s |
| q38 | date range and group by a title | 13.867ms | 101.048ms | 103.215ms | 0.0% | 103.215ms | 103.215ms | 103.215ms | 103.215ms | 130.000ms | 80.39 MiB | none | 9.69K/s |
| q39 | date range, group by and offset | 11.584ms | 101.207ms | 101.021ms | 0.0% | 101.021ms | 101.021ms | 101.021ms | 101.021ms | 130.000ms | 77.18 MiB | none | 9.90K/s |
| q40 | date range, a case and a wide group by | 14.599ms | 104.469ms | 101.591ms | 0.0% | 101.591ms | 101.591ms | 101.591ms | 101.591ms | 160.000ms | 80.12 MiB | none | 9.84K/s |
| q41 | date range with an IN and a hash | 13.760ms | 101.578ms | 97.463ms | 0.0% | 97.463ms | 97.463ms | 97.463ms | 97.463ms | 130.000ms | 81.04 MiB | 120.00 KiB | 10.26K/s |
| q42 | date range and a deep offset | 12.779ms | 97.527ms | 100.482ms | 0.0% | 100.482ms | 100.482ms | 100.482ms | 100.482ms | 130.000ms | 79.26 MiB | none | 9.95K/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 536.773ms by its own clock and 3.989s by ours, 4.028s cold, 5.400s of CPU, peak 84.05 MiB, 72.66K/s and 16.03 MiB/s.

Running it cost 643% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.26x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-server in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 30.786ms | 29.590ms | 0.0% | 29.590ms | 29.590ms | 29.590ms | 29.590ms | not read | not read | not read | 33.80K/s |
| q2 | filtered count | 1.000ms | 29.428ms | 28.907ms | 0.0% | 28.907ms | 28.907ms | 28.907ms | 28.907ms | not read | not read | not read | 34.59K/s |
| q3 | three aggregates | 1.000ms | 30.589ms | 30.277ms | 0.0% | 30.277ms | 30.277ms | 30.277ms | 30.277ms | not read | not read | not read | 33.03K/s |
| q4 | average | 1.000ms | 29.787ms | 29.923ms | 0.0% | 29.923ms | 29.923ms | 29.923ms | 29.923ms | not read | not read | not read | 33.42K/s |
| q5 | count distinct, high card | 1.000ms | 32.093ms | 29.447ms | 0.0% | 29.447ms | 29.447ms | 29.447ms | 29.447ms | not read | not read | not read | 33.96K/s |
| q6 | count distinct, strings | 1.000ms | 29.383ms | 29.199ms | 0.0% | 29.199ms | 29.199ms | 29.199ms | 29.199ms | not read | not read | not read | 34.25K/s |
| q7 | min and max of a date | 1.000ms | 29.878ms | 30.426ms | 0.0% | 30.426ms | 30.426ms | 30.426ms | 30.426ms | not read | not read | not read | 32.87K/s |
| q8 | group by, low card | 2.000ms | 31.926ms | 30.170ms | 0.0% | 30.170ms | 30.170ms | 30.170ms | 30.170ms | not read | not read | not read | 33.15K/s |
| q9 | group by and count distinct | 2.000ms | 30.271ms | 30.773ms | 0.0% | 30.773ms | 30.773ms | 30.773ms | 30.773ms | not read | not read | not read | 32.50K/s |
| q10 | group by, several aggregates | 16.000ms | 31.100ms | 44.995ms | 0.0% | 44.995ms | 44.995ms | 44.995ms | 44.995ms | not read | not read | not read | 22.22K/s |
| q11 | group by a string and count distinct | 2.000ms | 31.300ms | 30.782ms | 0.0% | 30.782ms | 30.782ms | 30.782ms | 30.782ms | not read | not read | not read | 32.49K/s |
| q12 | group by two strings and count distinct | 2.000ms | 30.828ms | 30.866ms | 0.0% | 30.866ms | 30.866ms | 30.866ms | 30.866ms | not read | not read | not read | 32.40K/s |
| q13 | group by a string and top k | 2.000ms | 30.709ms | 30.711ms | 0.0% | 30.711ms | 30.711ms | 30.711ms | 30.711ms | not read | not read | not read | 32.56K/s |
| q14 | group by a string and count distinct | 2.000ms | 30.783ms | 30.116ms | 0.0% | 30.116ms | 30.116ms | 30.116ms | 30.116ms | not read | not read | not read | 33.20K/s |
| q15 | group by two columns and top k | 2.000ms | 30.294ms | 31.473ms | 0.0% | 31.473ms | 31.473ms | 31.473ms | 31.473ms | not read | not read | not read | 31.77K/s |
| q16 | group by, very high card | 2.000ms | 30.549ms | 30.199ms | 0.0% | 30.199ms | 30.199ms | 30.199ms | 30.199ms | not read | not read | not read | 33.11K/s |
| q17 | group by two, very high card | 2.000ms | 31.094ms | 30.626ms | 0.0% | 30.626ms | 30.626ms | 30.626ms | 30.626ms | not read | not read | not read | 32.65K/s |
| q18 | group by two, no ordering | 2.000ms | 44.874ms | 30.001ms | 0.0% | 30.001ms | 30.001ms | 30.001ms | 30.001ms | not read | not read | not read | 33.33K/s |
| q19 | group by with an extract | 2.000ms | 33.405ms | 31.131ms | 0.0% | 31.131ms | 31.131ms | 31.131ms | 31.131ms | not read | not read | not read | 32.12K/s |
| q20 | point lookup | 1.000ms | 30.470ms | 29.275ms | 0.0% | 29.275ms | 29.275ms | 29.275ms | 29.275ms | not read | not read | not read | 34.16K/s |
| q21 | substring scan | 1.000ms | 30.598ms | 31.008ms | 0.0% | 31.008ms | 31.008ms | 31.008ms | 31.008ms | not read | not read | not read | 32.25K/s |
| q22 | substring scan and group by | 3.000ms | 33.228ms | 32.602ms | 0.0% | 32.602ms | 32.602ms | 32.602ms | 32.602ms | not read | not read | not read | 30.67K/s |
| q23 | two substring scans and group by | 3.000ms | 30.916ms | 32.011ms | 0.0% | 32.011ms | 32.011ms | 32.011ms | 32.011ms | not read | not read | not read | 31.24K/s |
| q24 | select star and top k | 4.000ms | 33.637ms | 33.224ms | 0.0% | 33.224ms | 33.224ms | 33.224ms | 33.224ms | not read | not read | not read | 30.10K/s |
| q25 | top k by a date | 2.000ms | 30.725ms | 31.210ms | 0.0% | 31.210ms | 31.210ms | 31.210ms | 31.210ms | not read | not read | not read | 32.04K/s |
| q26 | top k by a string | 1.000ms | 30.405ms | 29.764ms | 0.0% | 29.764ms | 29.764ms | 29.764ms | 29.764ms | not read | not read | not read | 33.60K/s |
| q27 | top k by two columns | 1.000ms | 30.228ms | 30.179ms | 0.0% | 30.179ms | 30.179ms | 30.179ms | 30.179ms | not read | not read | not read | 33.14K/s |
| q28 | group by with a string length | 2.000ms | 31.080ms | 31.169ms | 0.0% | 31.169ms | 31.169ms | 31.169ms | 31.169ms | not read | not read | not read | 32.08K/s |
| q29 | group by a regular expression | 21.000ms | 31.398ms | 50.269ms | 0.0% | 50.269ms | 50.269ms | 50.269ms | 50.269ms | not read | not read | not read | 19.89K/s |
| q30 | ninety sums over one column | 6.000ms | 36.123ms | 34.971ms | 0.0% | 34.971ms | 34.971ms | 34.971ms | 34.971ms | not read | not read | not read | 28.60K/s |
| q31 | group by two and several aggregates | 2.000ms | 31.566ms | 30.674ms | 0.0% | 30.674ms | 30.674ms | 30.674ms | 30.674ms | not read | not read | not read | 32.60K/s |
| q32 | group by a high card pair | 14.000ms | 30.532ms | 43.556ms | 0.0% | 43.556ms | 43.556ms | 43.556ms | 43.556ms | not read | not read | not read | 22.96K/s |
| q33 | group by a high card pair, unfiltered | 2.000ms | 31.144ms | 30.144ms | 0.0% | 30.144ms | 30.144ms | 30.144ms | 30.144ms | not read | not read | not read | 33.17K/s |
| q34 | group by a long string | 2.000ms | 31.084ms | 30.330ms | 0.0% | 30.330ms | 30.330ms | 30.330ms | 30.330ms | not read | not read | not read | 32.97K/s |
| q35 | group by a constant and a long string | 2.000ms | 30.998ms | 30.526ms | 0.0% | 30.526ms | 30.526ms | 30.526ms | 30.526ms | not read | not read | not read | 32.76K/s |
| q36 | group by four expressions | 2.000ms | 30.631ms | 30.242ms | 0.0% | 30.242ms | 30.242ms | 30.242ms | 30.242ms | not read | not read | not read | 33.07K/s |
| q37 | date range and group by a URL | 3.000ms | 32.069ms | 32.659ms | 0.0% | 32.659ms | 32.659ms | 32.659ms | 32.659ms | not read | not read | not read | 30.62K/s |
| q38 | date range and group by a title | 3.000ms | 31.437ms | 33.221ms | 0.0% | 33.221ms | 33.221ms | 33.221ms | 33.221ms | not read | not read | not read | 30.10K/s |
| q39 | date range, group by and offset | 3.000ms | 31.595ms | 31.843ms | 0.0% | 31.843ms | 31.843ms | 31.843ms | 31.843ms | not read | not read | not read | 31.40K/s |
| q40 | date range, a case and a wide group by | 3.000ms | 32.588ms | 31.818ms | 0.0% | 31.818ms | 31.818ms | 31.818ms | 31.818ms | not read | not read | not read | 31.43K/s |
| q41 | date range with an IN and a hash | 3.000ms | 31.285ms | 31.200ms | 0.0% | 31.200ms | 31.200ms | 31.200ms | 31.200ms | not read | not read | not read | 32.05K/s |
| q42 | date range and a deep offset | 3.000ms | 32.068ms | 32.507ms | 0.0% | 32.507ms | 32.507ms | 32.507ms | 32.507ms | not read | not read | not read | 30.76K/s |
| q43 | minute buckets over a date range | 3.000ms | 31.913ms | 32.379ms | 0.0% | 32.379ms | 32.379ms | 32.379ms | 32.379ms | not read | not read | not read | 30.88K/s |

clickhouse-server 26.9.1.1162 over 43 of 43 queries. Total 135.000ms by its own clock and 1.376s by ours, 1.357s cold, no reading of CPU, peak not read, 318.52K/s and 70.26 MiB/s.

Running it cost 920% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.74x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 0.000us | 1.663ms | 1.503ms | 0.0% | 1.503ms | 1.503ms | 1.503ms | 1.503ms | 0.000us | 4.25 MiB | none | 665.34K/s |
| q2 | filtered count | 0.000us | 1.419ms | 1.411ms | 0.0% | 1.411ms | 1.411ms | 1.411ms | 1.411ms | 0.000us | 4.48 MiB | none | 708.72K/s |
| q3 | three aggregates | 0.000us | 1.385ms | 1.382ms | 0.0% | 1.382ms | 1.382ms | 1.382ms | 1.382ms | 0.000us | 4.43 MiB | none | 723.59K/s |
| q4 | average | 0.000us | 1.382ms | 1.349ms | 0.0% | 1.349ms | 1.349ms | 1.349ms | 1.349ms | 0.000us | 4.41 MiB | none | 741.29K/s |
| q5 | count distinct, high card | 0.000us | 1.453ms | 1.463ms | 0.0% | 1.463ms | 1.463ms | 1.463ms | 1.463ms | 0.000us | 4.25 MiB | none | 683.53K/s |
| q6 | count distinct, strings | 0.000us | 1.450ms | 1.430ms | 0.0% | 1.430ms | 1.430ms | 1.430ms | 1.430ms | 0.000us | 4.26 MiB | none | 699.30K/s |
| q7 | min and max of a date | 0.000us | 1.353ms | 1.398ms | 0.0% | 1.398ms | 1.398ms | 1.398ms | 1.398ms | 0.000us | 4.50 MiB | none | 715.31K/s |
| q8 | group by, low card | 0.000us | 1.537ms | 1.416ms | 0.0% | 1.416ms | 1.416ms | 1.416ms | 1.416ms | 0.000us | 4.47 MiB | none | 706.21K/s |
| q9 | group by and count distinct | 0.000us | 1.662ms | 1.517ms | 0.0% | 1.517ms | 1.517ms | 1.517ms | 1.517ms | 0.000us | 4.71 MiB | none | 659.20K/s |
| q10 | group by, several aggregates | 1.000ms | 1.705ms | 1.671ms | 0.0% | 1.671ms | 1.671ms | 1.671ms | 1.671ms | 0.000us | 4.76 MiB | none | 598.44K/s |
| q11 | group by a string and count distinct | 0.000us | 1.452ms | 1.537ms | 0.0% | 1.537ms | 1.537ms | 1.537ms | 1.537ms | 0.000us | 4.45 MiB | none | 650.62K/s |
| q12 | group by two strings and count distinct | 0.000us | 1.513ms | 1.484ms | 0.0% | 1.484ms | 1.484ms | 1.484ms | 1.484ms | 0.000us | 4.39 MiB | none | 673.85K/s |
| q13 | group by a string and top k | 0.000us | 1.543ms | 1.472ms | 0.0% | 1.472ms | 1.472ms | 1.472ms | 1.472ms | 0.000us | 4.48 MiB | none | 679.35K/s |
| q14 | group by a string and count distinct | 0.000us | 1.575ms | 1.520ms | 0.0% | 1.520ms | 1.520ms | 1.520ms | 1.520ms | 0.000us | 4.40 MiB | none | 657.89K/s |
| q15 | group by two columns and top k | 0.000us | 1.585ms | 1.674ms | 0.0% | 1.674ms | 1.674ms | 1.674ms | 1.674ms | 0.000us | 4.48 MiB | none | 597.37K/s |
| q16 | group by, very high card | 1.000ms | 1.731ms | 1.661ms | 0.0% | 1.661ms | 1.661ms | 1.661ms | 1.661ms | 0.000us | 4.66 MiB | none | 602.05K/s |
| q17 | group by two, very high card | 1.000ms | 1.845ms | 1.874ms | 0.0% | 1.874ms | 1.874ms | 1.874ms | 1.874ms | 0.000us | 4.75 MiB | none | 533.62K/s |
| q18 | group by two, no ordering | 1.000ms | 1.670ms | 1.653ms | 0.0% | 1.653ms | 1.653ms | 1.653ms | 1.653ms | 0.000us | 4.82 MiB | none | 604.96K/s |
| q20 | point lookup | 0.000us | 1.584ms | 1.416ms | 0.0% | 1.416ms | 1.416ms | 1.416ms | 1.416ms | 0.000us | 4.48 MiB | none | 706.21K/s |
| q21 | substring scan | 1.000ms | 1.914ms | 1.821ms | 0.0% | 1.821ms | 1.821ms | 1.821ms | 1.821ms | 0.000us | 4.64 MiB | none | 549.15K/s |
| q22 | substring scan and group by | 1.000ms | 1.901ms | 1.876ms | 0.0% | 1.876ms | 1.876ms | 1.876ms | 1.876ms | 0.000us | 4.75 MiB | none | 533.05K/s |
| q23 | two substring scans and group by | 1.000ms | 2.482ms | 2.475ms | 0.0% | 2.475ms | 2.475ms | 2.475ms | 2.475ms | 0.000us | 4.98 MiB | none | 404.04K/s |
| q24 | select star and top k | 3.000ms | 4.382ms | 4.233ms | 0.0% | 4.233ms | 4.233ms | 4.233ms | 4.233ms | 0.000us | 6.00 MiB | none | 236.24K/s |
| q25 | top k by a date | 0.000us | 1.637ms | 1.578ms | 0.0% | 1.578ms | 1.578ms | 1.578ms | 1.578ms | 0.000us | 4.70 MiB | none | 633.71K/s |
| q26 | top k by a string | 0.000us | 1.510ms | 1.486ms | 0.0% | 1.486ms | 1.486ms | 1.486ms | 1.486ms | 0.000us | 4.50 MiB | none | 672.95K/s |
| q27 | top k by two columns | 0.000us | 1.665ms | 1.719ms | 0.0% | 1.719ms | 1.719ms | 1.719ms | 1.719ms | 0.000us | 4.46 MiB | none | 581.73K/s |
| q28 | group by with a string length | 1.000ms | 1.899ms | 2.068ms | 0.0% | 2.068ms | 2.068ms | 2.068ms | 2.068ms | 0.000us | 4.75 MiB | none | 483.56K/s |
| q29 | group by a regular expression | 1.000ms | 2.482ms | 2.440ms | 0.0% | 2.440ms | 2.440ms | 2.440ms | 2.440ms | 0.000us | 4.75 MiB | none | 409.84K/s |
| q30 | ninety sums over one column | 1.000ms | 2.708ms | 2.535ms | 0.0% | 2.535ms | 2.535ms | 2.535ms | 2.535ms | 0.000us | 5.01 MiB | none | 394.48K/s |
| q31 | group by two and several aggregates | 1.000ms | 1.659ms | 1.611ms | 0.0% | 1.611ms | 1.611ms | 1.611ms | 1.611ms | 0.000us | 4.51 MiB | none | 620.73K/s |
| q32 | group by a high card pair | 0.000us | 1.720ms | 1.582ms | 0.0% | 1.582ms | 1.582ms | 1.582ms | 1.582ms | 0.000us | 4.57 MiB | none | 632.11K/s |
| q34 | group by a long string | 1.000ms | 2.218ms | 2.048ms | 0.0% | 2.048ms | 2.048ms | 2.048ms | 2.048ms | 0.000us | 4.74 MiB | none | 488.28K/s |
| q35 | group by a constant and a long string | 1.000ms | 2.235ms | 2.265ms | 0.0% | 2.265ms | 2.265ms | 2.265ms | 2.265ms | 0.000us | 4.96 MiB | none | 441.50K/s |
| q36 | group by four expressions | 1.000ms | 1.822ms | 1.823ms | 0.0% | 1.823ms | 1.823ms | 1.823ms | 1.823ms | 0.000us | 4.91 MiB | none | 548.55K/s |
| q37 | date range and group by a URL | 1.000ms | 1.852ms | 2.493ms | 0.0% | 2.493ms | 2.493ms | 2.493ms | 2.493ms | 0.000us | 4.74 MiB | none | 401.12K/s |
| q38 | date range and group by a title | 1.000ms | 2.198ms | 2.160ms | 0.0% | 2.160ms | 2.160ms | 2.160ms | 2.160ms | 0.000us | 4.75 MiB | none | 462.96K/s |
| q39 | date range, group by and offset | 1.000ms | 2.010ms | 1.970ms | 0.0% | 1.970ms | 1.970ms | 1.970ms | 1.970ms | 0.000us | 4.75 MiB | none | 507.61K/s |
| q40 | date range, a case and a wide group by | 1.000ms | 2.542ms | 2.869ms | 0.0% | 2.869ms | 2.869ms | 2.869ms | 2.869ms | 0.000us | 5.00 MiB | none | 348.55K/s |
| q41 | date range with an IN and a hash | 0.000us | 1.682ms | 1.674ms | 0.0% | 1.674ms | 1.674ms | 1.674ms | 1.674ms | 0.000us | 4.51 MiB | none | 597.37K/s |
| q42 | date range and a deep offset | 1.000ms | 2.504ms | 1.647ms | 0.0% | 1.647ms | 1.647ms | 1.647ms | 1.647ms | 0.000us | 4.51 MiB | none | 607.16K/s |
| q43 | minute buckets over a date range | 0.000us | 1.591ms | 1.559ms | 0.0% | 1.559ms | 1.559ms | 1.559ms | 1.559ms | 0.000us | 4.82 MiB | none | 641.44K/s |

rudb rudb 0.2.31 over 41 of 43 queries. Total 22.000ms by its own clock and 74.763ms by ours, 76.120ms cold, 0.000us of CPU, peak 6.00 MiB, 1.86M/s and 411.09 MiB/s.

Running it cost 240% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.14x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 1000 rows, one out of every 99998 of the 99997497 in the full file, which is a development loop rather than the suite
- every query here ran within 1.44x of every other one, so most of what was timed is whatever they have in common rather than the queries
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

- duckdb ran every query within 1.44x of every other one, and this suite spreads over 6x on an engine it is measuring
- duckdb-pinned ran every query within 1.60x of every other one, and this suite spreads over 6x on an engine it is measuring
- clickhouse-local ran every query within 1.79x of every other one, and this suite spreads over 6x on an engine it is measuring
- datafusion ran every query within 1.77x of every other one, and this suite spreads over 6x on an engine it is measuring
- polars ran every query within 1.26x of every other one, and this suite spreads over 6x on an engine it is measuring
- clickhouse-server ran every query within 1.74x of every other one, and this suite spreads over 6x on an engine it is measuring

A column that flat is not a column about the queries. Read those numbers as an upper bound on the engine and not as a measurement of it.

polars did not run q28, because the SQLContext has no strlen and no character length function.

polars did not run q29, because the SQLContext has no regexp_replace.

polars did not run q36, because the SQLContext rejects the repeated ClientIP output name in the group by.

polars did not run q43, because the SQLContext has no date_trunc.

So the polars column is 39 of 43 queries and its ratio is over the shared ones.

rudb did not run q19, because rudb's hash aggregate does not spill and this groups by a key that is close to unique over a hundred million rows, so the table outgrows the machine. tamnd/rudb#220, milestone E5.

rudb did not run q33, because rudb's hash aggregate does not spill and this groups by a key that is close to unique over a hundred million rows, so the table outgrows the machine. tamnd/rudb#220, milestone E5.

So the rudb column is 41 of 43 queries and its ratio is over the shared ones.

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

Answers differ, so this is not a comparison: q34: clickhouse-local does not agree with duckdb: 10 numbers against 10

Answers differ, so this is not a comparison: q34: polars does not agree with duckdb: 10 numbers against 10

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

