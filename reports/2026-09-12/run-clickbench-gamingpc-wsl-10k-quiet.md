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
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 111.849ms | 100.000ms | 4.01 MiB | its own database file | its own | 3.42 to 3.42 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 131.390ms | 120.000ms | 3.76 MiB | its own database file | its own | 3.42 to 3.42 |
| clickhouse-local | 26.9.1.1162 | ran | 145.039ms | 130.000ms | 2.68 MiB | its own MergeTree parts, as system.parts counts the active ones | its own | 3.92 to 3.77 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 1.70 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 3.42 to 3.31 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 1.70 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 3.31 to 3.92 |
| clickhouse-server | 26.9.1.1162 | ran | 202.823ms | not read | 2.61 MiB | its own MergeTree parts, sorted by hits by (CounterID, EventDate, UserID, EventTime, WatchID), as system.parts counts the active ones | its own | 3.77 to 3.63 |
| rudb | rudb 0.2.31 | ran | 0.000us | 0.000us | 1.70 MiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 3.42 to 3.42 |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 125.000ms | 528.184ms | +323% | 525.392ms | 310.000ms | 0.59 | 38.80 MiB | none | 3.44M/s | 583.84 MiB/s | 1.00x |
| duckdb-pinned | 165.000ms | 1.104s | +569% | 1.108s | 930.000ms | 0.84 | 52.77 MiB | none | 2.61M/s | 442.30 MiB/s | 1.32x |
| clickhouse-local | 179.000ms | 2.226s | +1144% | 2.226s | 2.320s | 1.04 | 213.05 MiB | none | 2.40M/s | 407.71 MiB/s | 1.43x |
| datafusion | 359.000ms | 861.364ms | +140% | 846.603ms | 2.700s | 3.13 | 188.54 MiB | none | 1.20M/s | 203.29 MiB/s | 2.87x |
| polars | 595.837ms | 4.082s | +585% | 4.091s | 5.690s | 1.39 | 89.20 MiB | none | 654.54K/s | 111.09 MiB/s | 4.77x |
| clickhouse-server | 151.000ms | 1.444s | +857% | 1.473s | not read | not read | not read | not read | 2.85M/s | 483.31 MiB/s | 1.21x |
| rudb | 123.000ms | 176.133ms | +43% | 175.214ms | 20.000ms | 0.11 | 19.78 MiB | none | 3.33M/s | 565.73 MiB/s | 0.98x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | clickhouse-server | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 0.000us | 1.000ms | 2.000ms | 1.000ms | 6.557ms | 1.000ms | 0.000us |
| q2 | filtered count | 1.000ms | 1.000ms | 2.000ms | 6.000ms | 7.541ms | 1.000ms | 0.000us |
| q3 | three aggregates | 0.000us | 1.000ms | 3.000ms | 4.000ms | 5.529ms | 1.000ms | 0.000us |
| q4 | average | 0.000us | 1.000ms | 2.000ms | 3.000ms | 6.954ms | 1.000ms | 0.000us |
| q5 | count distinct, high card | 2.000ms | 2.000ms | 3.000ms | 7.000ms | 12.103ms | 2.000ms | 1.000ms |
| q6 | count distinct, strings | 3.000ms | 2.000ms | 3.000ms | 6.000ms | 12.616ms | 2.000ms | 1.000ms |
| q7 | min and max of a date | 0.000us | 1.000ms | 3.000ms | 1.000ms | 7.295ms | 1.000ms | 0.000us |
| q8 | group by, low card | 1.000ms | 6.000ms | 4.000ms | 6.000ms | 15.033ms | 2.000ms | 0.000us |
| q9 | group by and count distinct | 3.000ms | 4.000ms | 3.000ms | 8.000ms | 22.928ms | 2.000ms | 2.000ms |
| q10 | group by, several aggregates | 5.000ms | 5.000ms | 3.000ms | 8.000ms | 25.385ms | 17.000ms | 2.000ms |
| q11 | group by a string and count distinct | 3.000ms | 4.000ms | 3.000ms | 20.000ms | 19.488ms | 2.000ms | 1.000ms |
| q12 | group by two strings and count distinct | 3.000ms | 4.000ms | 3.000ms | 8.000ms | 22.646ms | 2.000ms | 1.000ms |
| q13 | group by a string and top k | 2.000ms | 2.000ms | 3.000ms | 8.000ms | 18.452ms | 2.000ms | 1.000ms |
| q14 | group by a string and count distinct | 4.000ms | 4.000ms | 3.000ms | 11.000ms | 22.144ms | 2.000ms | 2.000ms |
| q15 | group by two columns and top k | 2.000ms | 2.000ms | 3.000ms | 10.000ms | 15.433ms | 2.000ms | 2.000ms |
| q16 | group by, very high card | 3.000ms | 3.000ms | 3.000ms | 8.000ms | 17.985ms | 2.000ms | 2.000ms |
| q17 | group by two, very high card | 3.000ms | 4.000ms | 4.000ms | 8.000ms | 22.582ms | 2.000ms | 4.000ms |
| q18 | group by two, no ordering | 4.000ms | 3.000ms | 3.000ms | 7.000ms | 13.958ms | 2.000ms | 3.000ms |
| q19 | group by with an extract | 5.000ms | 3.000ms | 4.000ms | 9.000ms | 29.462ms | 3.000ms | no dialect |
| q20 | point lookup | 0.000us | 1.000ms | 3.000ms | 4.000ms | 6.559ms | 1.000ms | 0.000us |
| q21 | substring scan | 1.000ms | 2.000ms | 4.000ms | 6.000ms | 9.910ms | 1.000ms | 4.000ms |
| q22 | substring scan and group by | 2.000ms | 2.000ms | 4.000ms | 11.000ms | 17.166ms | 3.000ms | 4.000ms |
| q23 | two substring scans and group by | 5.000ms | 7.000ms | 6.000ms | 15.000ms | 22.572ms | 4.000ms | 9.000ms |
| q24 | select star and top k | 7.000ms | 12.000ms | 8.000ms | 15.000ms | 11.515ms | 4.000ms | 22.000ms |
| q25 | top k by a date | 3.000ms | 2.000ms | 4.000ms | 5.000ms | 12.140ms | 2.000ms | 1.000ms |
| q26 | top k by a string | 1.000ms | 1.000ms | 3.000ms | 5.000ms | 12.962ms | 1.000ms | 1.000ms |
| q27 | top k by two columns | 1.000ms | 2.000ms | 4.000ms | 6.000ms | 12.536ms | 2.000ms | 1.000ms |
| q28 | group by with a string length | 8.000ms | 5.000ms | 4.000ms | 8.000ms | no dialect | 3.000ms | 4.000ms |
| q29 | group by a regular expression | 7.000ms | 9.000ms | 9.000ms | 15.000ms | no dialect | 24.000ms | 9.000ms |
| q30 | ninety sums over one column | 4.000ms | 17.000ms | 8.000ms | 12.000ms | 11.387ms | 6.000ms | 3.000ms |
| q31 | group by two and several aggregates | 3.000ms | 4.000ms | 4.000ms | 8.000ms | 15.878ms | 2.000ms | 2.000ms |
| q32 | group by a high card pair | 3.000ms | 4.000ms | 4.000ms | 7.000ms | 15.110ms | 16.000ms | 2.000ms |
| q33 | group by a high card pair, unfiltered | 3.000ms | 4.000ms | 4.000ms | 10.000ms | 16.046ms | 2.000ms | no dialect |
| q34 | group by a long string | 4.000ms | 3.000ms | 5.000ms | 10.000ms | 17.190ms | 3.000ms | 6.000ms |
| q35 | group by a constant and a long string | 4.000ms | 4.000ms | 5.000ms | 9.000ms | 18.786ms | 3.000ms | 7.000ms |
| q36 | group by four expressions | 3.000ms | 3.000ms | 3.000ms | 6.000ms | no dialect | 2.000ms | 4.000ms |
| q37 | date range and group by a URL | 3.000ms | 4.000ms | 6.000ms | 11.000ms | 16.979ms | 3.000ms | 4.000ms |
| q38 | date range and group by a title | 3.000ms | 4.000ms | 6.000ms | 15.000ms | 17.199ms | 3.000ms | 4.000ms |
| q39 | date range, group by and offset | 3.000ms | 4.000ms | 6.000ms | 8.000ms | 15.823ms | 4.000ms | 4.000ms |
| q40 | date range, a case and a wide group by | 4.000ms | 4.000ms | 7.000ms | 10.000ms | 14.216ms | 4.000ms | 7.000ms |
| q41 | date range with an IN and a hash | 3.000ms | 3.000ms | 5.000ms | 7.000ms | 14.411ms | 3.000ms | 1.000ms |
| q42 | date range and a deep offset | 3.000ms | 7.000ms | 5.000ms | 8.000ms | 13.361ms | 3.000ms | 1.000ms |
| q43 | minute buckets over a date range | 3.000ms | 4.000ms | 5.000ms | 9.000ms | no dialect | 3.000ms | 1.000ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 0.000us | 9.461ms | 9.842ms | 0.0% | 9.842ms | 9.842ms | 9.842ms | 9.842ms | 10.000ms | 26.79 MiB | none | 1.02M/s |
| q2 | filtered count | 1.000ms | 9.727ms | 10.558ms | 0.0% | 10.558ms | 10.558ms | 10.558ms | 10.558ms | 10.000ms | 28.04 MiB | none | 947.15K/s |
| q3 | three aggregates | 0.000us | 9.905ms | 9.876ms | 0.0% | 9.876ms | 9.876ms | 9.876ms | 9.876ms | 0.000us | 28.38 MiB | none | 1.01M/s |
| q4 | average | 0.000us | 10.110ms | 9.699ms | 0.0% | 9.699ms | 9.699ms | 9.699ms | 9.699ms | 0.000us | 27.79 MiB | none | 1.03M/s |
| q5 | count distinct, high card | 2.000ms | 12.183ms | 11.633ms | 0.0% | 11.633ms | 11.633ms | 11.633ms | 11.633ms | 0.000us | 30.48 MiB | none | 859.62K/s |
| q6 | count distinct, strings | 3.000ms | 11.814ms | 11.348ms | 0.0% | 11.348ms | 11.348ms | 11.348ms | 11.348ms | 10.000ms | 30.53 MiB | none | 881.21K/s |
| q7 | min and max of a date | 0.000us | 10.561ms | 9.445ms | 0.0% | 9.445ms | 9.445ms | 9.445ms | 9.445ms | 0.000us | 27.22 MiB | none | 1.06M/s |
| q8 | group by, low card | 1.000ms | 10.286ms | 10.367ms | 0.0% | 10.367ms | 10.367ms | 10.367ms | 10.367ms | 10.000ms | 29.24 MiB | none | 964.60K/s |
| q9 | group by and count distinct | 3.000ms | 15.460ms | 12.707ms | 0.0% | 12.707ms | 12.707ms | 12.707ms | 12.707ms | 10.000ms | 36.30 MiB | none | 786.97K/s |
| q10 | group by, several aggregates | 5.000ms | 14.058ms | 15.671ms | 0.0% | 15.671ms | 15.671ms | 15.671ms | 15.671ms | 10.000ms | 38.80 MiB | none | 638.12K/s |
| q11 | group by a string and count distinct | 3.000ms | 13.576ms | 12.840ms | 0.0% | 12.840ms | 12.840ms | 12.840ms | 12.840ms | 10.000ms | 35.24 MiB | none | 778.82K/s |
| q12 | group by two strings and count distinct | 3.000ms | 13.180ms | 12.671ms | 0.0% | 12.671ms | 12.671ms | 12.671ms | 12.671ms | 10.000ms | 35.49 MiB | none | 789.20K/s |
| q13 | group by a string and top k | 2.000ms | 11.659ms | 11.262ms | 0.0% | 11.262ms | 11.262ms | 11.262ms | 11.262ms | 10.000ms | 31.30 MiB | none | 887.94K/s |
| q14 | group by a string and count distinct | 4.000ms | 13.602ms | 13.135ms | 0.0% | 13.135ms | 13.135ms | 13.135ms | 13.135ms | 10.000ms | 36.80 MiB | none | 761.32K/s |
| q15 | group by two columns and top k | 2.000ms | 11.774ms | 11.547ms | 0.0% | 11.547ms | 11.547ms | 11.547ms | 11.547ms | 0.000us | 32.05 MiB | none | 866.03K/s |
| q16 | group by, very high card | 3.000ms | 12.441ms | 11.780ms | 0.0% | 11.780ms | 11.780ms | 11.780ms | 11.780ms | 0.000us | 33.44 MiB | none | 848.90K/s |
| q17 | group by two, very high card | 3.000ms | 12.297ms | 12.207ms | 0.0% | 12.207ms | 12.207ms | 12.207ms | 12.207ms | 0.000us | 35.29 MiB | none | 819.20K/s |
| q18 | group by two, no ordering | 4.000ms | 12.439ms | 13.179ms | 0.0% | 13.179ms | 13.179ms | 13.179ms | 13.179ms | 10.000ms | 36.22 MiB | none | 758.78K/s |
| q19 | group by with an extract | 5.000ms | 13.366ms | 13.815ms | 0.0% | 13.815ms | 13.815ms | 13.815ms | 13.815ms | 10.000ms | 35.62 MiB | none | 723.85K/s |
| q20 | point lookup | 0.000us | 9.729ms | 9.768ms | 0.0% | 9.768ms | 9.768ms | 9.768ms | 9.768ms | 0.000us | 27.52 MiB | none | 1.02M/s |
| q21 | substring scan | 1.000ms | 10.925ms | 10.736ms | 0.0% | 10.736ms | 10.736ms | 10.736ms | 10.736ms | 10.000ms | 28.79 MiB | none | 931.45K/s |
| q22 | substring scan and group by | 2.000ms | 10.966ms | 11.470ms | 0.0% | 11.470ms | 11.470ms | 11.470ms | 11.470ms | 0.000us | 29.80 MiB | none | 871.84K/s |
| q23 | two substring scans and group by | 5.000ms | 13.447ms | 13.583ms | 0.0% | 13.583ms | 13.583ms | 13.583ms | 13.583ms | 10.000ms | 34.69 MiB | none | 736.21K/s |
| q24 | select star and top k | 7.000ms | 15.755ms | 16.507ms | 0.0% | 16.507ms | 16.507ms | 16.507ms | 16.507ms | 10.000ms | 37.83 MiB | none | 605.80K/s |
| q25 | top k by a date | 3.000ms | 11.447ms | 11.381ms | 0.0% | 11.381ms | 11.381ms | 11.381ms | 11.381ms | 0.000us | 30.78 MiB | none | 878.66K/s |
| q26 | top k by a string | 1.000ms | 10.320ms | 9.835ms | 0.0% | 9.835ms | 9.835ms | 9.835ms | 9.835ms | 10.000ms | 28.04 MiB | none | 1.02M/s |
| q27 | top k by two columns | 1.000ms | 9.948ms | 10.691ms | 0.0% | 10.691ms | 10.691ms | 10.691ms | 10.691ms | 10.000ms | 28.69 MiB | none | 935.37K/s |
| q28 | group by with a string length | 8.000ms | 14.016ms | 18.223ms | 0.0% | 18.223ms | 18.223ms | 18.223ms | 18.223ms | 20.000ms | 32.05 MiB | none | 548.76K/s |
| q29 | group by a regular expression | 7.000ms | 16.491ms | 17.046ms | 0.0% | 17.046ms | 17.046ms | 17.046ms | 17.046ms | 10.000ms | 33.29 MiB | none | 586.65K/s |
| q30 | ninety sums over one column | 4.000ms | 13.206ms | 13.016ms | 0.0% | 13.016ms | 13.016ms | 13.016ms | 13.016ms | 0.000us | 31.54 MiB | none | 768.29K/s |
| q31 | group by two and several aggregates | 3.000ms | 13.144ms | 12.119ms | 0.0% | 12.119ms | 12.119ms | 12.119ms | 12.119ms | 10.000ms | 34.93 MiB | none | 825.15K/s |
| q32 | group by a high card pair | 3.000ms | 11.702ms | 11.995ms | 0.0% | 11.995ms | 11.995ms | 11.995ms | 11.995ms | 10.000ms | 33.35 MiB | none | 833.68K/s |
| q33 | group by a high card pair, unfiltered | 3.000ms | 12.170ms | 12.120ms | 0.0% | 12.120ms | 12.120ms | 12.120ms | 12.120ms | 10.000ms | 35.10 MiB | none | 825.08K/s |
| q34 | group by a long string | 4.000ms | 13.025ms | 13.366ms | 0.0% | 13.366ms | 13.366ms | 13.366ms | 13.366ms | 10.000ms | 35.80 MiB | none | 748.17K/s |
| q35 | group by a constant and a long string | 4.000ms | 12.941ms | 13.066ms | 0.0% | 13.066ms | 13.066ms | 13.066ms | 13.066ms | 10.000ms | 34.30 MiB | none | 765.35K/s |
| q36 | group by four expressions | 3.000ms | 12.503ms | 12.161ms | 0.0% | 12.161ms | 12.161ms | 12.161ms | 12.161ms | 0.000us | 34.10 MiB | none | 822.30K/s |
| q37 | date range and group by a URL | 3.000ms | 13.441ms | 13.589ms | 0.0% | 13.589ms | 13.589ms | 13.589ms | 13.589ms | 10.000ms | 32.80 MiB | none | 735.89K/s |
| q38 | date range and group by a title | 3.000ms | 12.224ms | 12.328ms | 0.0% | 12.328ms | 12.328ms | 12.328ms | 12.328ms | 10.000ms | 32.49 MiB | none | 811.16K/s |
| q39 | date range, group by and offset | 3.000ms | 11.918ms | 11.963ms | 0.0% | 11.963ms | 11.963ms | 11.963ms | 11.963ms | 10.000ms | 32.05 MiB | none | 835.91K/s |
| q40 | date range, a case and a wide group by | 4.000ms | 12.617ms | 12.944ms | 0.0% | 12.944ms | 12.944ms | 12.944ms | 12.944ms | 10.000ms | 34.80 MiB | none | 772.56K/s |
| q41 | date range with an IN and a hash | 3.000ms | 11.948ms | 12.958ms | 0.0% | 12.958ms | 12.958ms | 12.958ms | 12.958ms | 10.000ms | 32.37 MiB | none | 771.72K/s |
| q42 | date range and a deep offset | 3.000ms | 11.836ms | 11.739ms | 0.0% | 11.739ms | 11.739ms | 11.739ms | 11.739ms | 0.000us | 31.80 MiB | none | 851.86K/s |
| q43 | minute buckets over a date range | 3.000ms | 11.774ms | 11.998ms | 0.0% | 11.998ms | 11.998ms | 11.998ms | 11.998ms | 10.000ms | 31.79 MiB | none | 833.47K/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 125.000ms by its own clock and 528.184ms by ours, 525.392ms cold, 310.000ms of CPU, peak 38.80 MiB, 3.44M/s and 583.84 MiB/s.

Running it cost 323% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.93x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 22.611ms | 23.068ms | 0.0% | 23.068ms | 23.068ms | 23.068ms | 23.068ms | 20.000ms | 39.77 MiB | none | 433.50K/s |
| q2 | filtered count | 1.000ms | 22.639ms | 23.010ms | 0.0% | 23.010ms | 23.010ms | 23.010ms | 23.010ms | 20.000ms | 40.02 MiB | none | 434.59K/s |
| q3 | three aggregates | 1.000ms | 23.537ms | 22.989ms | 0.0% | 22.989ms | 22.989ms | 22.989ms | 22.989ms | 20.000ms | 40.77 MiB | none | 434.99K/s |
| q4 | average | 1.000ms | 22.473ms | 23.090ms | 0.0% | 23.090ms | 23.090ms | 23.090ms | 23.090ms | 10.000ms | 40.52 MiB | none | 433.09K/s |
| q5 | count distinct, high card | 2.000ms | 25.448ms | 24.005ms | 0.0% | 24.005ms | 24.005ms | 24.005ms | 24.005ms | 20.000ms | 43.40 MiB | none | 416.58K/s |
| q6 | count distinct, strings | 2.000ms | 23.826ms | 24.037ms | 0.0% | 24.037ms | 24.037ms | 24.037ms | 24.037ms | 20.000ms | 42.52 MiB | none | 416.03K/s |
| q7 | min and max of a date | 1.000ms | 23.908ms | 22.545ms | 0.0% | 22.545ms | 22.545ms | 22.545ms | 22.545ms | 10.000ms | 40.27 MiB | none | 443.56K/s |
| q8 | group by, low card | 6.000ms | 28.229ms | 27.552ms | 0.0% | 27.552ms | 27.552ms | 27.552ms | 27.552ms | 20.000ms | 42.91 MiB | none | 362.95K/s |
| q9 | group by and count distinct | 4.000ms | 25.793ms | 26.167ms | 0.0% | 26.167ms | 26.167ms | 26.167ms | 26.167ms | 30.000ms | 48.04 MiB | none | 382.16K/s |
| q10 | group by, several aggregates | 5.000ms | 27.215ms | 27.169ms | 0.0% | 27.169ms | 27.169ms | 27.169ms | 27.169ms | 30.000ms | 50.45 MiB | none | 368.07K/s |
| q11 | group by a string and count distinct | 4.000ms | 25.690ms | 26.097ms | 0.0% | 26.097ms | 26.097ms | 26.097ms | 26.097ms | 20.000ms | 47.91 MiB | none | 383.19K/s |
| q12 | group by two strings and count distinct | 4.000ms | 25.084ms | 25.634ms | 0.0% | 25.634ms | 25.634ms | 25.634ms | 25.634ms | 20.000ms | 47.04 MiB | none | 390.11K/s |
| q13 | group by a string and top k | 2.000ms | 25.034ms | 24.216ms | 0.0% | 24.216ms | 24.216ms | 24.216ms | 24.216ms | 20.000ms | 42.78 MiB | none | 412.95K/s |
| q14 | group by a string and count distinct | 4.000ms | 25.981ms | 26.912ms | 0.0% | 26.912ms | 26.912ms | 26.912ms | 26.912ms | 30.000ms | 48.96 MiB | none | 371.58K/s |
| q15 | group by two columns and top k | 2.000ms | 24.156ms | 23.779ms | 0.0% | 23.779ms | 23.779ms | 23.779ms | 23.779ms | 20.000ms | 43.78 MiB | none | 420.54K/s |
| q16 | group by, very high card | 3.000ms | 25.347ms | 24.155ms | 0.0% | 24.155ms | 24.155ms | 24.155ms | 24.155ms | 20.000ms | 44.83 MiB | none | 413.99K/s |
| q17 | group by two, very high card | 4.000ms | 24.276ms | 24.484ms | 0.0% | 24.484ms | 24.484ms | 24.484ms | 24.484ms | 20.000ms | 44.82 MiB | none | 408.43K/s |
| q18 | group by two, no ordering | 3.000ms | 25.060ms | 24.926ms | 0.0% | 24.926ms | 24.926ms | 24.926ms | 24.926ms | 20.000ms | 45.50 MiB | none | 401.19K/s |
| q19 | group by with an extract | 3.000ms | 25.042ms | 23.927ms | 0.0% | 23.927ms | 23.927ms | 23.927ms | 23.927ms | 20.000ms | 46.77 MiB | none | 417.94K/s |
| q20 | point lookup | 1.000ms | 22.409ms | 24.217ms | 0.0% | 24.217ms | 24.217ms | 24.217ms | 24.217ms | 20.000ms | 39.77 MiB | none | 412.93K/s |
| q21 | substring scan | 2.000ms | 26.511ms | 23.725ms | 0.0% | 23.725ms | 23.725ms | 23.725ms | 23.725ms | 20.000ms | 41.24 MiB | none | 421.50K/s |
| q22 | substring scan and group by | 2.000ms | 24.752ms | 24.195ms | 0.0% | 24.195ms | 24.195ms | 24.195ms | 24.195ms | 20.000ms | 42.28 MiB | none | 413.31K/s |
| q23 | two substring scans and group by | 7.000ms | 28.233ms | 28.189ms | 0.0% | 28.189ms | 28.189ms | 28.189ms | 28.189ms | 30.000ms | 48.71 MiB | none | 354.75K/s |
| q24 | select star and top k | 12.000ms | 33.392ms | 34.298ms | 0.0% | 34.298ms | 34.298ms | 34.298ms | 34.298ms | 30.000ms | 52.57 MiB | none | 291.56K/s |
| q25 | top k by a date | 2.000ms | 23.126ms | 23.566ms | 0.0% | 23.566ms | 23.566ms | 23.566ms | 23.566ms | 10.000ms | 40.92 MiB | none | 424.34K/s |
| q26 | top k by a string | 1.000ms | 23.757ms | 23.450ms | 0.0% | 23.450ms | 23.450ms | 23.450ms | 23.450ms | 20.000ms | 40.64 MiB | none | 426.44K/s |
| q27 | top k by two columns | 2.000ms | 23.439ms | 23.448ms | 0.0% | 23.448ms | 23.448ms | 23.448ms | 23.448ms | 20.000ms | 41.02 MiB | none | 426.48K/s |
| q28 | group by with a string length | 5.000ms | 27.252ms | 27.194ms | 0.0% | 27.194ms | 27.194ms | 27.194ms | 27.194ms | 30.000ms | 44.54 MiB | none | 367.73K/s |
| q29 | group by a regular expression | 9.000ms | 29.453ms | 29.997ms | 0.0% | 29.997ms | 29.997ms | 29.997ms | 29.997ms | 30.000ms | 46.76 MiB | none | 333.37K/s |
| q30 | ninety sums over one column | 17.000ms | 38.038ms | 40.166ms | 0.0% | 40.166ms | 40.166ms | 40.166ms | 40.166ms | 40.000ms | 52.77 MiB | none | 248.97K/s |
| q31 | group by two and several aggregates | 4.000ms | 25.091ms | 26.338ms | 0.0% | 26.338ms | 26.338ms | 26.338ms | 26.338ms | 20.000ms | 46.27 MiB | none | 379.68K/s |
| q32 | group by a high card pair | 4.000ms | 26.943ms | 25.109ms | 0.0% | 25.109ms | 25.109ms | 25.109ms | 25.109ms | 20.000ms | 46.47 MiB | none | 398.26K/s |
| q33 | group by a high card pair, unfiltered | 4.000ms | 25.682ms | 24.740ms | 0.0% | 24.740ms | 24.740ms | 24.740ms | 24.740ms | 20.000ms | 45.80 MiB | none | 404.20K/s |
| q34 | group by a long string | 3.000ms | 25.495ms | 25.311ms | 0.0% | 25.311ms | 25.311ms | 25.311ms | 25.311ms | 20.000ms | 45.53 MiB | none | 395.09K/s |
| q35 | group by a constant and a long string | 4.000ms | 25.438ms | 25.558ms | 0.0% | 25.558ms | 25.558ms | 25.558ms | 25.558ms | 20.000ms | 45.41 MiB | none | 391.27K/s |
| q36 | group by four expressions | 3.000ms | 24.804ms | 24.615ms | 0.0% | 24.615ms | 24.615ms | 24.615ms | 24.615ms | 20.000ms | 44.98 MiB | none | 406.26K/s |
| q37 | date range and group by a URL | 4.000ms | 26.985ms | 25.869ms | 0.0% | 25.869ms | 25.869ms | 25.869ms | 25.869ms | 20.000ms | 45.53 MiB | none | 386.56K/s |
| q38 | date range and group by a title | 4.000ms | 25.322ms | 24.896ms | 0.0% | 24.896ms | 24.896ms | 24.896ms | 24.896ms | 20.000ms | 45.16 MiB | none | 401.67K/s |
| q39 | date range, group by and offset | 4.000ms | 24.685ms | 25.397ms | 0.0% | 25.397ms | 25.397ms | 25.397ms | 25.397ms | 20.000ms | 44.29 MiB | none | 393.75K/s |
| q40 | date range, a case and a wide group by | 4.000ms | 26.152ms | 26.446ms | 0.0% | 26.446ms | 26.446ms | 26.446ms | 26.446ms | 20.000ms | 46.79 MiB | none | 378.13K/s |
| q41 | date range with an IN and a hash | 3.000ms | 26.343ms | 24.877ms | 0.0% | 24.877ms | 24.877ms | 24.877ms | 24.877ms | 20.000ms | 45.04 MiB | none | 401.98K/s |
| q42 | date range and a deep offset | 7.000ms | 27.944ms | 29.599ms | 0.0% | 29.599ms | 29.599ms | 29.599ms | 29.599ms | 30.000ms | 45.16 MiB | none | 337.85K/s |
| q43 | minute buckets over a date range | 4.000ms | 25.137ms | 25.521ms | 0.0% | 25.521ms | 25.521ms | 25.521ms | 25.521ms | 20.000ms | 43.27 MiB | none | 391.83K/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 165.000ms by its own clock and 1.104s by ours, 1.108s cold, 930.000ms of CPU, peak 52.77 MiB, 2.61M/s and 442.30 MiB/s.

Running it cost 569% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.78x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 2.000ms | 47.455ms | 47.421ms | 0.0% | 47.421ms | 47.421ms | 47.421ms | 47.421ms | 60.000ms | 200.46 MiB | none | 210.88K/s |
| q2 | filtered count | 2.000ms | 48.925ms | 50.604ms | 0.0% | 50.604ms | 50.604ms | 50.604ms | 50.604ms | 50.000ms | 200.97 MiB | none | 197.61K/s |
| q3 | three aggregates | 3.000ms | 49.364ms | 56.956ms | 0.0% | 56.956ms | 56.956ms | 56.956ms | 56.956ms | 50.000ms | 203.41 MiB | none | 175.57K/s |
| q4 | average | 2.000ms | 49.802ms | 48.912ms | 0.0% | 48.912ms | 48.912ms | 48.912ms | 48.912ms | 50.000ms | 202.95 MiB | none | 204.45K/s |
| q5 | count distinct, high card | 3.000ms | 50.048ms | 49.803ms | 0.0% | 49.803ms | 49.803ms | 49.803ms | 49.803ms | 50.000ms | 204.47 MiB | none | 200.79K/s |
| q6 | count distinct, strings | 3.000ms | 49.836ms | 53.176ms | 0.0% | 53.176ms | 53.176ms | 53.176ms | 53.176ms | 120.000ms | 204.32 MiB | none | 188.05K/s |
| q7 | min and max of a date | 3.000ms | 49.362ms | 50.996ms | 0.0% | 50.996ms | 50.996ms | 50.996ms | 50.996ms | 60.000ms | 202.57 MiB | none | 196.09K/s |
| q8 | group by, low card | 4.000ms | 49.182ms | 51.016ms | 0.0% | 51.016ms | 51.016ms | 51.016ms | 51.016ms | 50.000ms | 205.57 MiB | none | 196.02K/s |
| q9 | group by and count distinct | 3.000ms | 48.853ms | 49.234ms | 0.0% | 49.234ms | 49.234ms | 49.234ms | 49.234ms | 50.000ms | 205.70 MiB | none | 203.11K/s |
| q10 | group by, several aggregates | 3.000ms | 54.035ms | 54.949ms | 0.0% | 54.949ms | 54.949ms | 54.949ms | 54.949ms | 50.000ms | 206.52 MiB | none | 181.99K/s |
| q11 | group by a string and count distinct | 3.000ms | 52.273ms | 50.102ms | 0.0% | 50.102ms | 50.102ms | 50.102ms | 50.102ms | 50.000ms | 206.08 MiB | none | 199.59K/s |
| q12 | group by two strings and count distinct | 3.000ms | 49.871ms | 50.124ms | 0.0% | 50.124ms | 50.124ms | 50.124ms | 50.124ms | 50.000ms | 206.70 MiB | none | 199.51K/s |
| q13 | group by a string and top k | 3.000ms | 51.226ms | 49.640ms | 0.0% | 49.640ms | 49.640ms | 49.640ms | 49.640ms | 50.000ms | 206.07 MiB | none | 201.45K/s |
| q14 | group by a string and count distinct | 3.000ms | 50.701ms | 48.444ms | 0.0% | 48.444ms | 48.444ms | 48.444ms | 48.444ms | 40.000ms | 206.32 MiB | none | 206.42K/s |
| q15 | group by two columns and top k | 3.000ms | 48.314ms | 48.651ms | 0.0% | 48.651ms | 48.651ms | 48.651ms | 48.651ms | 50.000ms | 206.82 MiB | none | 205.55K/s |
| q16 | group by, very high card | 3.000ms | 49.213ms | 49.799ms | 0.0% | 49.799ms | 49.799ms | 49.799ms | 49.799ms | 50.000ms | 205.73 MiB | none | 200.81K/s |
| q17 | group by two, very high card | 4.000ms | 51.898ms | 49.773ms | 0.0% | 49.773ms | 49.773ms | 49.773ms | 49.773ms | 50.000ms | 208.57 MiB | none | 200.91K/s |
| q18 | group by two, no ordering | 3.000ms | 55.028ms | 50.007ms | 0.0% | 50.007ms | 50.007ms | 50.007ms | 50.007ms | 50.000ms | 206.57 MiB | none | 199.97K/s |
| q19 | group by with an extract | 4.000ms | 57.661ms | 50.390ms | 0.0% | 50.390ms | 50.390ms | 50.390ms | 50.390ms | 50.000ms | 209.14 MiB | none | 198.45K/s |
| q20 | point lookup | 3.000ms | 51.504ms | 50.892ms | 0.0% | 50.892ms | 50.892ms | 50.892ms | 50.892ms | 50.000ms | 203.07 MiB | none | 196.49K/s |
| q21 | substring scan | 4.000ms | 52.831ms | 53.053ms | 0.0% | 53.053ms | 53.053ms | 53.053ms | 53.053ms | 50.000ms | 206.30 MiB | none | 188.49K/s |
| q22 | substring scan and group by | 4.000ms | 51.693ms | 50.483ms | 0.0% | 50.483ms | 50.483ms | 50.483ms | 50.483ms | 50.000ms | 208.57 MiB | none | 198.09K/s |
| q23 | two substring scans and group by | 6.000ms | 53.114ms | 57.449ms | 0.0% | 57.449ms | 57.449ms | 57.449ms | 57.449ms | 60.000ms | 213.05 MiB | none | 174.07K/s |
| q24 | select star and top k | 8.000ms | 56.361ms | 53.590ms | 0.0% | 53.590ms | 53.590ms | 53.590ms | 53.590ms | 50.000ms | 207.78 MiB | none | 186.60K/s |
| q25 | top k by a date | 4.000ms | 53.663ms | 55.441ms | 0.0% | 55.441ms | 55.441ms | 55.441ms | 55.441ms | 60.000ms | 206.07 MiB | none | 180.37K/s |
| q26 | top k by a string | 3.000ms | 48.333ms | 51.957ms | 0.0% | 51.957ms | 51.957ms | 51.957ms | 51.957ms | 50.000ms | 204.75 MiB | none | 192.47K/s |
| q27 | top k by two columns | 4.000ms | 49.993ms | 51.389ms | 0.0% | 51.389ms | 51.389ms | 51.389ms | 51.389ms | 50.000ms | 205.48 MiB | none | 194.59K/s |
| q28 | group by with a string length | 4.000ms | 50.234ms | 50.595ms | 0.0% | 50.595ms | 50.595ms | 50.595ms | 50.595ms | 60.000ms | 208.03 MiB | none | 197.65K/s |
| q29 | group by a regular expression | 9.000ms | 53.816ms | 54.500ms | 0.0% | 54.500ms | 54.500ms | 54.500ms | 54.500ms | 50.000ms | 210.52 MiB | none | 183.49K/s |
| q30 | ninety sums over one column | 8.000ms | 54.426ms | 53.915ms | 0.0% | 53.915ms | 53.915ms | 53.915ms | 53.915ms | 50.000ms | 207.04 MiB | none | 185.48K/s |
| q31 | group by two and several aggregates | 4.000ms | 50.537ms | 52.175ms | 0.0% | 52.175ms | 52.175ms | 52.175ms | 52.175ms | 60.000ms | 207.55 MiB | none | 191.66K/s |
| q32 | group by a high card pair | 4.000ms | 49.857ms | 50.259ms | 0.0% | 50.259ms | 50.259ms | 50.259ms | 50.259ms | 50.000ms | 206.80 MiB | none | 198.97K/s |
| q33 | group by a high card pair, unfiltered | 4.000ms | 55.860ms | 49.861ms | 0.0% | 49.861ms | 49.861ms | 49.861ms | 49.861ms | 50.000ms | 208.17 MiB | none | 200.56K/s |
| q34 | group by a long string | 5.000ms | 51.401ms | 51.699ms | 0.0% | 51.699ms | 51.699ms | 51.699ms | 51.699ms | 50.000ms | 207.82 MiB | none | 193.43K/s |
| q35 | group by a constant and a long string | 5.000ms | 51.513ms | 59.183ms | 0.0% | 59.183ms | 59.183ms | 59.183ms | 59.183ms | 50.000ms | 208.32 MiB | none | 168.97K/s |
| q36 | group by four expressions | 3.000ms | 48.525ms | 49.812ms | 0.0% | 49.812ms | 49.812ms | 49.812ms | 49.812ms | 50.000ms | 207.27 MiB | none | 200.75K/s |
| q37 | date range and group by a URL | 6.000ms | 57.953ms | 53.099ms | 0.0% | 53.099ms | 53.099ms | 53.099ms | 53.099ms | 60.000ms | 210.43 MiB | none | 188.33K/s |
| q38 | date range and group by a title | 6.000ms | 51.921ms | 53.144ms | 0.0% | 53.144ms | 53.144ms | 53.144ms | 53.144ms | 60.000ms | 211.07 MiB | none | 188.17K/s |
| q39 | date range, group by and offset | 6.000ms | 54.492ms | 53.145ms | 0.0% | 53.145ms | 53.145ms | 53.145ms | 53.145ms | 60.000ms | 210.82 MiB | none | 188.16K/s |
| q40 | date range, a case and a wide group by | 7.000ms | 53.058ms | 53.715ms | 0.0% | 53.715ms | 53.715ms | 53.715ms | 53.715ms | 60.000ms | 212.32 MiB | none | 186.17K/s |
| q41 | date range with an IN and a hash | 5.000ms | 54.201ms | 51.300ms | 0.0% | 51.300ms | 51.300ms | 51.300ms | 51.300ms | 50.000ms | 209.81 MiB | none | 194.93K/s |
| q42 | date range and a deep offset | 5.000ms | 56.367ms | 54.097ms | 0.0% | 54.097ms | 54.097ms | 54.097ms | 54.097ms | 60.000ms | 208.07 MiB | none | 184.85K/s |
| q43 | minute buckets over a date range | 5.000ms | 50.983ms | 51.623ms | 0.0% | 51.623ms | 51.623ms | 51.623ms | 51.623ms | 50.000ms | 208.63 MiB | none | 193.71K/s |

clickhouse-local 26.9.1.1162 over 43 of 43 queries. Total 179.000ms by its own clock and 2.226s by ours, 2.226s cold, 2.320s of CPU, peak 213.05 MiB, 2.40M/s and 407.71 MiB/s.

Running it cost 1144% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.25x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 13.357ms | 12.342ms | 0.0% | 12.342ms | 12.342ms | 12.342ms | 12.342ms | 10.000ms | 79.21 MiB | none | 810.24K/s |
| q2 | filtered count | 6.000ms | 15.860ms | 16.959ms | 0.0% | 16.959ms | 16.959ms | 16.959ms | 16.959ms | 40.000ms | 115.36 MiB | none | 589.66K/s |
| q3 | three aggregates | 4.000ms | 15.003ms | 15.496ms | 0.0% | 15.496ms | 15.496ms | 15.496ms | 15.496ms | 10.000ms | 111.18 MiB | none | 645.33K/s |
| q4 | average | 3.000ms | 14.747ms | 14.832ms | 0.0% | 14.832ms | 14.832ms | 14.832ms | 14.832ms | 20.000ms | 105.84 MiB | none | 674.22K/s |
| q5 | count distinct, high card | 7.000ms | 17.143ms | 18.128ms | 0.0% | 18.128ms | 18.128ms | 18.128ms | 18.128ms | 40.000ms | 139.26 MiB | none | 551.63K/s |
| q6 | count distinct, strings | 6.000ms | 19.084ms | 18.195ms | 0.0% | 18.195ms | 18.195ms | 18.195ms | 18.195ms | 40.000ms | 153.72 MiB | none | 549.60K/s |
| q7 | min and max of a date | 1.000ms | 11.889ms | 11.780ms | 0.0% | 11.780ms | 11.780ms | 11.780ms | 11.780ms | 10.000ms | 79.97 MiB | none | 848.90K/s |
| q8 | group by, low card | 6.000ms | 16.926ms | 17.689ms | 0.0% | 17.689ms | 17.689ms | 17.689ms | 17.689ms | 40.000ms | 128.75 MiB | none | 565.32K/s |
| q9 | group by and count distinct | 8.000ms | 21.734ms | 19.485ms | 0.0% | 19.485ms | 19.485ms | 19.485ms | 19.485ms | 60.000ms | 155.84 MiB | none | 513.22K/s |
| q10 | group by, several aggregates | 8.000ms | 17.411ms | 19.762ms | 0.0% | 19.762ms | 19.762ms | 19.762ms | 19.762ms | 50.000ms | 151.77 MiB | none | 506.02K/s |
| q11 | group by a string and count distinct | 20.000ms | 20.511ms | 31.694ms | 0.0% | 31.694ms | 31.694ms | 31.694ms | 31.694ms | 90.000ms | 171.21 MiB | none | 315.52K/s |
| q12 | group by two strings and count distinct | 8.000ms | 24.323ms | 19.752ms | 0.0% | 19.752ms | 19.752ms | 19.752ms | 19.752ms | 50.000ms | 186.83 MiB | none | 506.28K/s |
| q13 | group by a string and top k | 8.000ms | 19.809ms | 19.324ms | 0.0% | 19.324ms | 19.324ms | 19.324ms | 19.324ms | 60.000ms | 163.14 MiB | none | 517.49K/s |
| q14 | group by a string and count distinct | 11.000ms | 21.490ms | 24.169ms | 0.0% | 24.169ms | 24.169ms | 24.169ms | 24.169ms | 130.000ms | 187.93 MiB | none | 413.75K/s |
| q15 | group by two columns and top k | 10.000ms | 22.396ms | 22.215ms | 0.0% | 22.215ms | 22.215ms | 22.215ms | 22.215ms | 160.000ms | 171.20 MiB | none | 450.15K/s |
| q16 | group by, very high card | 8.000ms | 19.281ms | 20.051ms | 0.0% | 20.051ms | 20.051ms | 20.051ms | 20.051ms | 110.000ms | 142.79 MiB | none | 498.73K/s |
| q17 | group by two, very high card | 8.000ms | 21.765ms | 19.004ms | 0.0% | 19.004ms | 19.004ms | 19.004ms | 19.004ms | 30.000ms | 154.72 MiB | none | 526.21K/s |
| q18 | group by two, no ordering | 7.000ms | 18.322ms | 17.392ms | 0.0% | 17.392ms | 17.392ms | 17.392ms | 17.392ms | 40.000ms | 149.60 MiB | none | 574.98K/s |
| q19 | group by with an extract | 9.000ms | 20.136ms | 19.732ms | 0.0% | 19.732ms | 19.732ms | 19.732ms | 19.732ms | 40.000ms | 160.10 MiB | none | 506.79K/s |
| q20 | point lookup | 4.000ms | 16.432ms | 15.580ms | 0.0% | 15.580ms | 15.580ms | 15.580ms | 15.580ms | 50.000ms | 123.05 MiB | none | 641.85K/s |
| q21 | substring scan | 6.000ms | 16.886ms | 17.949ms | 0.0% | 17.949ms | 17.949ms | 17.949ms | 17.949ms | 50.000ms | 123.79 MiB | none | 557.13K/s |
| q22 | substring scan and group by | 11.000ms | 21.493ms | 23.484ms | 0.0% | 23.484ms | 23.484ms | 23.484ms | 23.484ms | 170.000ms | 162.87 MiB | none | 425.82K/s |
| q23 | two substring scans and group by | 15.000ms | 25.252ms | 26.371ms | 0.0% | 26.371ms | 26.371ms | 26.371ms | 26.371ms | 200.000ms | 169.04 MiB | none | 379.20K/s |
| q24 | select star and top k | 15.000ms | 26.762ms | 27.735ms | 0.0% | 27.735ms | 27.735ms | 27.735ms | 27.735ms | 50.000ms | 142.65 MiB | none | 360.56K/s |
| q25 | top k by a date | 5.000ms | 17.160ms | 16.970ms | 0.0% | 16.970ms | 16.970ms | 16.970ms | 16.970ms | 40.000ms | 140.69 MiB | none | 589.28K/s |
| q26 | top k by a string | 5.000ms | 16.462ms | 16.848ms | 0.0% | 16.848ms | 16.848ms | 16.848ms | 16.848ms | 40.000ms | 136.41 MiB | none | 593.54K/s |
| q27 | top k by two columns | 6.000ms | 16.378ms | 17.940ms | 0.0% | 17.940ms | 17.940ms | 17.940ms | 17.940ms | 50.000ms | 143.20 MiB | none | 557.41K/s |
| q28 | group by with a string length | 8.000ms | 19.020ms | 19.641ms | 0.0% | 19.641ms | 19.641ms | 19.641ms | 19.641ms | 60.000ms | 154.45 MiB | none | 509.14K/s |
| q29 | group by a regular expression | 15.000ms | 26.996ms | 26.844ms | 0.0% | 26.844ms | 26.844ms | 26.844ms | 26.844ms | 100.000ms | 188.54 MiB | none | 372.52K/s |
| q30 | ninety sums over one column | 12.000ms | 24.000ms | 23.634ms | 0.0% | 23.634ms | 23.634ms | 23.634ms | 23.634ms | 10.000ms | 112.58 MiB | none | 423.12K/s |
| q31 | group by two and several aggregates | 8.000ms | 18.376ms | 19.722ms | 0.0% | 19.722ms | 19.722ms | 19.722ms | 19.722ms | 20.000ms | 145.91 MiB | none | 507.05K/s |
| q32 | group by a high card pair | 7.000ms | 19.009ms | 19.014ms | 0.0% | 19.014ms | 19.014ms | 19.014ms | 19.014ms | 30.000ms | 151.71 MiB | none | 525.93K/s |
| q33 | group by a high card pair, unfiltered | 10.000ms | 20.656ms | 20.949ms | 0.0% | 20.949ms | 20.949ms | 20.949ms | 20.949ms | 30.000ms | 147.47 MiB | none | 477.35K/s |
| q34 | group by a long string | 10.000ms | 20.505ms | 22.212ms | 0.0% | 22.212ms | 22.212ms | 22.212ms | 22.212ms | 60.000ms | 175.63 MiB | none | 450.21K/s |
| q35 | group by a constant and a long string | 9.000ms | 21.662ms | 22.441ms | 0.0% | 22.441ms | 22.441ms | 22.441ms | 22.441ms | 70.000ms | 181.10 MiB | none | 445.61K/s |
| q36 | group by four expressions | 6.000ms | 17.658ms | 18.016ms | 0.0% | 18.016ms | 18.016ms | 18.016ms | 18.016ms | 40.000ms | 133.38 MiB | none | 555.06K/s |
| q37 | date range and group by a URL | 11.000ms | 21.606ms | 22.090ms | 0.0% | 22.090ms | 22.090ms | 22.090ms | 22.090ms | 140.000ms | 167.70 MiB | none | 452.69K/s |
| q38 | date range and group by a title | 15.000ms | 25.292ms | 26.165ms | 0.0% | 26.165ms | 26.165ms | 26.165ms | 26.165ms | 220.000ms | 171.93 MiB | none | 382.19K/s |
| q39 | date range, group by and offset | 8.000ms | 22.555ms | 19.547ms | 0.0% | 19.547ms | 19.547ms | 19.547ms | 19.547ms | 30.000ms | 151.51 MiB | none | 511.59K/s |
| q40 | date range, a case and a wide group by | 10.000ms | 20.634ms | 20.972ms | 0.0% | 20.972ms | 20.972ms | 20.972ms | 20.972ms | 40.000ms | 139.30 MiB | none | 476.83K/s |
| q41 | date range with an IN and a hash | 7.000ms | 19.795ms | 19.321ms | 0.0% | 19.321ms | 19.321ms | 19.321ms | 19.321ms | 30.000ms | 129.37 MiB | none | 517.57K/s |
| q42 | date range and a deep offset | 8.000ms | 20.214ms | 18.949ms | 0.0% | 18.949ms | 18.949ms | 18.949ms | 18.949ms | 50.000ms | 135.90 MiB | none | 527.73K/s |
| q43 | minute buckets over a date range | 9.000ms | 20.613ms | 20.969ms | 0.0% | 20.969ms | 20.969ms | 20.969ms | 20.969ms | 90.000ms | 140.36 MiB | none | 476.89K/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 359.000ms by its own clock and 861.364ms by ours, 846.603ms cold, 2.700s of CPU, peak 188.54 MiB, 1.20M/s and 203.29 MiB/s.

Running it cost 140% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.69x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 6.557ms | 97.807ms | 93.457ms | 0.0% | 93.457ms | 93.457ms | 93.457ms | 93.457ms | 100.000ms | 61.45 MiB | none | 107.00K/s |
| q2 | filtered count | 7.541ms | 96.569ms | 95.201ms | 0.0% | 95.201ms | 95.201ms | 95.201ms | 95.201ms | 150.000ms | 67.13 MiB | none | 105.04K/s |
| q3 | three aggregates | 5.529ms | 91.442ms | 87.791ms | 0.0% | 87.791ms | 87.791ms | 87.791ms | 87.791ms | 100.000ms | 66.12 MiB | none | 113.91K/s |
| q4 | average | 6.954ms | 89.088ms | 97.988ms | 0.0% | 97.988ms | 97.988ms | 97.988ms | 97.988ms | 100.000ms | 63.37 MiB | none | 102.05K/s |
| q5 | count distinct, high card | 12.103ms | 97.539ms | 95.973ms | 0.0% | 95.973ms | 95.973ms | 95.973ms | 95.973ms | 120.000ms | 74.32 MiB | none | 104.20K/s |
| q6 | count distinct, strings | 12.616ms | 96.462ms | 95.299ms | 0.0% | 95.299ms | 95.299ms | 95.299ms | 95.299ms | 120.000ms | 75.62 MiB | none | 104.93K/s |
| q7 | min and max of a date | 7.295ms | 92.653ms | 93.955ms | 0.0% | 93.955ms | 93.955ms | 93.955ms | 93.955ms | 150.000ms | 65.09 MiB | none | 106.43K/s |
| q8 | group by, low card | 15.033ms | 105.754ms | 101.741ms | 0.0% | 101.741ms | 101.741ms | 101.741ms | 101.741ms | 130.000ms | 76.78 MiB | none | 98.29K/s |
| q9 | group by and count distinct | 22.928ms | 108.209ms | 115.129ms | 0.0% | 115.129ms | 115.129ms | 115.129ms | 115.129ms | 180.000ms | 84.02 MiB | none | 86.86K/s |
| q10 | group by, several aggregates | 25.385ms | 116.704ms | 111.883ms | 0.0% | 111.883ms | 111.883ms | 111.883ms | 111.883ms | 160.000ms | 86.71 MiB | none | 89.38K/s |
| q11 | group by a string and count distinct | 19.488ms | 111.088ms | 111.352ms | 0.0% | 111.352ms | 111.352ms | 111.352ms | 111.352ms | 270.000ms | 83.20 MiB | none | 89.81K/s |
| q12 | group by two strings and count distinct | 22.646ms | 112.895ms | 109.634ms | 0.0% | 109.634ms | 109.634ms | 109.634ms | 109.634ms | 150.000ms | 84.04 MiB | none | 91.21K/s |
| q13 | group by a string and top k | 18.452ms | 106.213ms | 113.137ms | 0.0% | 113.137ms | 113.137ms | 113.137ms | 113.137ms | 160.000ms | 77.84 MiB | none | 88.39K/s |
| q14 | group by a string and count distinct | 22.144ms | 112.837ms | 110.755ms | 0.0% | 110.755ms | 110.755ms | 110.755ms | 110.755ms | 150.000ms | 84.82 MiB | none | 90.29K/s |
| q15 | group by two columns and top k | 15.433ms | 120.727ms | 103.930ms | 0.0% | 103.930ms | 103.930ms | 103.930ms | 103.930ms | 130.000ms | 78.10 MiB | none | 96.22K/s |
| q16 | group by, very high card | 17.985ms | 110.308ms | 108.191ms | 0.0% | 108.191ms | 108.191ms | 108.191ms | 108.191ms | 170.000ms | 76.47 MiB | none | 92.43K/s |
| q17 | group by two, very high card | 22.582ms | 114.463ms | 109.227ms | 0.0% | 109.227ms | 109.227ms | 109.227ms | 109.227ms | 150.000ms | 79.25 MiB | none | 91.55K/s |
| q18 | group by two, no ordering | 13.958ms | 99.997ms | 102.609ms | 0.0% | 102.609ms | 102.609ms | 102.609ms | 102.609ms | 190.000ms | 77.35 MiB | none | 97.46K/s |
| q19 | group by with an extract | 29.462ms | 115.384ms | 126.603ms | 0.0% | 126.603ms | 126.603ms | 126.603ms | 126.603ms | 190.000ms | 81.32 MiB | none | 78.99K/s |
| q20 | point lookup | 6.559ms | 96.806ms | 96.278ms | 0.0% | 96.278ms | 96.278ms | 96.278ms | 96.278ms | 150.000ms | 64.15 MiB | none | 103.87K/s |
| q21 | substring scan | 9.910ms | 108.922ms | 102.352ms | 0.0% | 102.352ms | 102.352ms | 102.352ms | 102.352ms | 100.000ms | 69.31 MiB | none | 97.70K/s |
| q22 | substring scan and group by | 17.166ms | 106.045ms | 106.773ms | 0.0% | 106.773ms | 106.773ms | 106.773ms | 106.773ms | 130.000ms | 78.73 MiB | none | 93.66K/s |
| q23 | two substring scans and group by | 22.572ms | 124.736ms | 113.766ms | 0.0% | 113.766ms | 113.766ms | 113.766ms | 113.766ms | 290.000ms | 89.20 MiB | none | 87.90K/s |
| q24 | select star and top k | 11.515ms | 96.635ms | 98.561ms | 0.0% | 98.561ms | 98.561ms | 98.561ms | 98.561ms | 100.000ms | 76.14 MiB | none | 101.46K/s |
| q25 | top k by a date | 12.140ms | 92.432ms | 103.237ms | 0.0% | 103.237ms | 103.237ms | 103.237ms | 103.237ms | 120.000ms | 73.06 MiB | none | 96.86K/s |
| q26 | top k by a string | 12.962ms | 92.194ms | 101.579ms | 0.0% | 101.579ms | 101.579ms | 101.579ms | 101.579ms | 130.000ms | 71.57 MiB | none | 98.45K/s |
| q27 | top k by two columns | 12.536ms | 98.242ms | 105.405ms | 0.0% | 105.405ms | 105.405ms | 105.405ms | 105.405ms | 130.000ms | 72.91 MiB | none | 94.87K/s |
| q30 | ninety sums over one column | 11.387ms | 102.439ms | 98.200ms | 0.0% | 98.200ms | 98.200ms | 98.200ms | 98.200ms | 110.000ms | 68.55 MiB | none | 101.83K/s |
| q31 | group by two and several aggregates | 15.878ms | 105.072ms | 104.900ms | 0.0% | 104.900ms | 104.900ms | 104.900ms | 104.900ms | 140.000ms | 79.96 MiB | none | 95.33K/s |
| q32 | group by a high card pair | 15.110ms | 109.972ms | 105.228ms | 0.0% | 105.228ms | 105.228ms | 105.228ms | 105.228ms | 130.000ms | 80.86 MiB | none | 95.03K/s |
| q33 | group by a high card pair, unfiltered | 16.046ms | 101.271ms | 106.522ms | 0.0% | 106.522ms | 106.522ms | 106.522ms | 106.522ms | 140.000ms | 81.21 MiB | none | 93.88K/s |
| q34 | group by a long string | 17.190ms | 110.888ms | 108.207ms | 0.0% | 108.207ms | 108.207ms | 108.207ms | 108.207ms | 130.000ms | 80.76 MiB | none | 92.42K/s |
| q35 | group by a constant and a long string | 18.786ms | 112.337ms | 110.184ms | 0.0% | 110.184ms | 110.184ms | 110.184ms | 110.184ms | 150.000ms | 82.88 MiB | none | 90.76K/s |
| q37 | date range and group by a URL | 16.979ms | 106.654ms | 107.070ms | 0.0% | 107.070ms | 107.070ms | 107.070ms | 107.070ms | 140.000ms | 83.54 MiB | none | 93.40K/s |
| q38 | date range and group by a title | 17.199ms | 106.308ms | 104.341ms | 0.0% | 104.341ms | 104.341ms | 104.341ms | 104.341ms | 130.000ms | 83.41 MiB | none | 95.84K/s |
| q39 | date range, group by and offset | 15.823ms | 108.632ms | 108.165ms | 0.0% | 108.165ms | 108.165ms | 108.165ms | 108.165ms | 160.000ms | 80.05 MiB | none | 92.45K/s |
| q40 | date range, a case and a wide group by | 14.216ms | 104.157ms | 107.599ms | 0.0% | 107.599ms | 107.599ms | 107.599ms | 107.599ms | 140.000ms | 80.98 MiB | none | 92.94K/s |
| q41 | date range with an IN and a hash | 14.411ms | 104.066ms | 107.591ms | 0.0% | 107.591ms | 107.591ms | 107.591ms | 107.591ms | 170.000ms | 80.66 MiB | none | 92.94K/s |
| q42 | date range and a deep offset | 13.361ms | 107.139ms | 102.487ms | 0.0% | 102.487ms | 102.487ms | 102.487ms | 102.487ms | 130.000ms | 80.52 MiB | none | 97.57K/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 595.837ms by its own clock and 4.082s by ours, 4.091s cold, 5.690s of CPU, peak 89.20 MiB, 654.54K/s and 111.09 MiB/s.

Running it cost 585% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.44x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-server in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 30.884ms | 31.996ms | 0.0% | 31.996ms | 31.996ms | 31.996ms | 31.996ms | not read | not read | not read | 312.54K/s |
| q2 | filtered count | 1.000ms | 30.898ms | 31.637ms | 0.0% | 31.637ms | 31.637ms | 31.637ms | 31.637ms | not read | not read | not read | 316.09K/s |
| q3 | three aggregates | 1.000ms | 35.411ms | 30.401ms | 0.0% | 30.401ms | 30.401ms | 30.401ms | 30.401ms | not read | not read | not read | 328.94K/s |
| q4 | average | 1.000ms | 31.736ms | 31.730ms | 0.0% | 31.730ms | 31.730ms | 31.730ms | 31.730ms | not read | not read | not read | 315.16K/s |
| q5 | count distinct, high card | 2.000ms | 32.380ms | 32.566ms | 0.0% | 32.566ms | 32.566ms | 32.566ms | 32.566ms | not read | not read | not read | 307.07K/s |
| q6 | count distinct, strings | 2.000ms | 31.990ms | 30.866ms | 0.0% | 30.866ms | 30.866ms | 30.866ms | 30.866ms | not read | not read | not read | 323.98K/s |
| q7 | min and max of a date | 1.000ms | 31.459ms | 31.623ms | 0.0% | 31.623ms | 31.623ms | 31.623ms | 31.623ms | not read | not read | not read | 316.23K/s |
| q8 | group by, low card | 2.000ms | 35.682ms | 32.237ms | 0.0% | 32.237ms | 32.237ms | 32.237ms | 32.237ms | not read | not read | not read | 310.20K/s |
| q9 | group by and count distinct | 2.000ms | 31.820ms | 30.979ms | 0.0% | 30.979ms | 30.979ms | 30.979ms | 30.979ms | not read | not read | not read | 322.80K/s |
| q10 | group by, several aggregates | 17.000ms | 33.383ms | 46.897ms | 0.0% | 46.897ms | 46.897ms | 46.897ms | 46.897ms | not read | not read | not read | 213.23K/s |
| q11 | group by a string and count distinct | 2.000ms | 32.190ms | 32.602ms | 0.0% | 32.602ms | 32.602ms | 32.602ms | 32.602ms | not read | not read | not read | 306.73K/s |
| q12 | group by two strings and count distinct | 2.000ms | 32.140ms | 32.224ms | 0.0% | 32.224ms | 32.224ms | 32.224ms | 32.224ms | not read | not read | not read | 310.33K/s |
| q13 | group by a string and top k | 2.000ms | 30.820ms | 32.032ms | 0.0% | 32.032ms | 32.032ms | 32.032ms | 32.032ms | not read | not read | not read | 312.19K/s |
| q14 | group by a string and count distinct | 2.000ms | 33.311ms | 31.990ms | 0.0% | 31.990ms | 31.990ms | 31.990ms | 31.990ms | not read | not read | not read | 312.60K/s |
| q15 | group by two columns and top k | 2.000ms | 30.703ms | 33.148ms | 0.0% | 33.148ms | 33.148ms | 33.148ms | 33.148ms | not read | not read | not read | 301.68K/s |
| q16 | group by, very high card | 2.000ms | 31.687ms | 30.849ms | 0.0% | 30.849ms | 30.849ms | 30.849ms | 30.849ms | not read | not read | not read | 324.16K/s |
| q17 | group by two, very high card | 2.000ms | 93.096ms | 31.849ms | 0.0% | 31.849ms | 31.849ms | 31.849ms | 31.849ms | not read | not read | not read | 313.98K/s |
| q18 | group by two, no ordering | 2.000ms | 31.908ms | 31.282ms | 0.0% | 31.282ms | 31.282ms | 31.282ms | 31.282ms | not read | not read | not read | 319.67K/s |
| q19 | group by with an extract | 3.000ms | 32.333ms | 32.743ms | 0.0% | 32.743ms | 32.743ms | 32.743ms | 32.743ms | not read | not read | not read | 305.41K/s |
| q20 | point lookup | 1.000ms | 32.554ms | 31.462ms | 0.0% | 31.462ms | 31.462ms | 31.462ms | 31.462ms | not read | not read | not read | 317.84K/s |
| q21 | substring scan | 1.000ms | 31.549ms | 30.278ms | 0.0% | 30.278ms | 30.278ms | 30.278ms | 30.278ms | not read | not read | not read | 330.27K/s |
| q22 | substring scan and group by | 3.000ms | 32.655ms | 33.554ms | 0.0% | 33.554ms | 33.554ms | 33.554ms | 33.554ms | not read | not read | not read | 298.03K/s |
| q23 | two substring scans and group by | 4.000ms | 34.272ms | 33.862ms | 0.0% | 33.862ms | 33.862ms | 33.862ms | 33.862ms | not read | not read | not read | 295.32K/s |
| q24 | select star and top k | 4.000ms | 34.875ms | 34.672ms | 0.0% | 34.672ms | 34.672ms | 34.672ms | 34.672ms | not read | not read | not read | 288.42K/s |
| q25 | top k by a date | 2.000ms | 31.384ms | 32.591ms | 0.0% | 32.591ms | 32.591ms | 32.591ms | 32.591ms | not read | not read | not read | 306.83K/s |
| q26 | top k by a string | 1.000ms | 30.612ms | 31.668ms | 0.0% | 31.668ms | 31.668ms | 31.668ms | 31.668ms | not read | not read | not read | 315.78K/s |
| q27 | top k by two columns | 2.000ms | 30.781ms | 32.357ms | 0.0% | 32.357ms | 32.357ms | 32.357ms | 32.357ms | not read | not read | not read | 309.05K/s |
| q28 | group by with a string length | 3.000ms | 35.033ms | 33.457ms | 0.0% | 33.457ms | 33.457ms | 33.457ms | 33.457ms | not read | not read | not read | 298.89K/s |
| q29 | group by a regular expression | 24.000ms | 37.131ms | 54.112ms | 0.0% | 54.112ms | 54.112ms | 54.112ms | 54.112ms | not read | not read | not read | 184.80K/s |
| q30 | ninety sums over one column | 6.000ms | 40.167ms | 38.164ms | 0.0% | 38.164ms | 38.164ms | 38.164ms | 38.164ms | not read | not read | not read | 262.03K/s |
| q31 | group by two and several aggregates | 2.000ms | 32.277ms | 32.055ms | 0.0% | 32.055ms | 32.055ms | 32.055ms | 32.055ms | not read | not read | not read | 311.96K/s |
| q32 | group by a high card pair | 16.000ms | 32.075ms | 45.769ms | 0.0% | 45.769ms | 45.769ms | 45.769ms | 45.769ms | not read | not read | not read | 218.49K/s |
| q33 | group by a high card pair, unfiltered | 2.000ms | 32.427ms | 32.349ms | 0.0% | 32.349ms | 32.349ms | 32.349ms | 32.349ms | not read | not read | not read | 309.13K/s |
| q34 | group by a long string | 3.000ms | 33.548ms | 32.542ms | 0.0% | 32.542ms | 32.542ms | 32.542ms | 32.542ms | not read | not read | not read | 307.30K/s |
| q35 | group by a constant and a long string | 3.000ms | 32.595ms | 32.467ms | 0.0% | 32.467ms | 32.467ms | 32.467ms | 32.467ms | not read | not read | not read | 308.01K/s |
| q36 | group by four expressions | 2.000ms | 31.198ms | 31.429ms | 0.0% | 31.429ms | 31.429ms | 31.429ms | 31.429ms | not read | not read | not read | 318.18K/s |
| q37 | date range and group by a URL | 3.000ms | 34.228ms | 33.303ms | 0.0% | 33.303ms | 33.303ms | 33.303ms | 33.303ms | not read | not read | not read | 300.27K/s |
| q38 | date range and group by a title | 3.000ms | 33.122ms | 33.052ms | 0.0% | 33.052ms | 33.052ms | 33.052ms | 33.052ms | not read | not read | not read | 302.55K/s |
| q39 | date range, group by and offset | 4.000ms | 34.668ms | 33.720ms | 0.0% | 33.720ms | 33.720ms | 33.720ms | 33.720ms | not read | not read | not read | 296.56K/s |
| q40 | date range, a case and a wide group by | 4.000ms | 34.051ms | 33.039ms | 0.0% | 33.039ms | 33.039ms | 33.039ms | 33.039ms | not read | not read | not read | 302.67K/s |
| q41 | date range with an IN and a hash | 3.000ms | 32.740ms | 32.915ms | 0.0% | 32.915ms | 32.915ms | 32.915ms | 32.915ms | not read | not read | not read | 303.81K/s |
| q42 | date range and a deep offset | 3.000ms | 32.069ms | 33.869ms | 0.0% | 33.869ms | 33.869ms | 33.869ms | 33.869ms | not read | not read | not read | 295.26K/s |
| q43 | minute buckets over a date range | 3.000ms | 32.803ms | 32.029ms | 0.0% | 32.029ms | 32.029ms | 32.029ms | 32.029ms | not read | not read | not read | 312.22K/s |

clickhouse-server 26.9.1.1162 over 43 of 43 queries. Total 151.000ms by its own clock and 1.444s by ours, 1.473s cold, no reading of CPU, peak not read, 2.85M/s and 483.31 MiB/s.

Running it cost 857% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.79x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 0.000us | 1.535ms | 1.383ms | 0.0% | 1.383ms | 1.383ms | 1.383ms | 1.383ms | 0.000us | 4.23 MiB | none | 7.23M/s |
| q2 | filtered count | 0.000us | 1.464ms | 1.434ms | 0.0% | 1.434ms | 1.434ms | 1.434ms | 1.434ms | 0.000us | 4.50 MiB | none | 6.97M/s |
| q3 | three aggregates | 0.000us | 1.597ms | 1.568ms | 0.0% | 1.568ms | 1.568ms | 1.568ms | 1.568ms | 0.000us | 4.71 MiB | none | 6.38M/s |
| q4 | average | 0.000us | 1.533ms | 1.583ms | 0.0% | 1.583ms | 1.583ms | 1.583ms | 1.583ms | 0.000us | 4.64 MiB | none | 6.32M/s |
| q5 | count distinct, high card | 1.000ms | 2.455ms | 2.411ms | 0.0% | 2.411ms | 2.411ms | 2.411ms | 2.411ms | 0.000us | 5.47 MiB | none | 4.15M/s |
| q6 | count distinct, strings | 1.000ms | 2.242ms | 2.248ms | 0.0% | 2.248ms | 2.248ms | 2.248ms | 2.248ms | 0.000us | 4.63 MiB | none | 4.45M/s |
| q7 | min and max of a date | 0.000us | 1.498ms | 1.450ms | 0.0% | 1.450ms | 1.450ms | 1.450ms | 1.450ms | 0.000us | 4.50 MiB | none | 6.90M/s |
| q8 | group by, low card | 0.000us | 1.509ms | 1.542ms | 0.0% | 1.542ms | 1.542ms | 1.542ms | 1.542ms | 0.000us | 4.50 MiB | none | 6.49M/s |
| q9 | group by and count distinct | 2.000ms | 3.029ms | 2.992ms | 0.0% | 2.992ms | 2.992ms | 2.992ms | 2.992ms | 0.000us | 5.75 MiB | none | 3.34M/s |
| q10 | group by, several aggregates | 2.000ms | 3.324ms | 3.244ms | 0.0% | 3.244ms | 3.244ms | 3.244ms | 3.244ms | 0.000us | 6.00 MiB | none | 3.08M/s |
| q11 | group by a string and count distinct | 1.000ms | 1.946ms | 1.946ms | 0.0% | 1.946ms | 1.946ms | 1.946ms | 1.946ms | 0.000us | 4.74 MiB | none | 5.14M/s |
| q12 | group by two strings and count distinct | 1.000ms | 1.763ms | 1.724ms | 0.0% | 1.724ms | 1.724ms | 1.724ms | 1.724ms | 0.000us | 4.73 MiB | none | 5.80M/s |
| q13 | group by a string and top k | 1.000ms | 2.633ms | 2.502ms | 0.0% | 2.502ms | 2.502ms | 2.502ms | 2.502ms | 0.000us | 5.23 MiB | none | 4.00M/s |
| q14 | group by a string and count distinct | 2.000ms | 2.873ms | 2.909ms | 0.0% | 2.909ms | 2.909ms | 2.909ms | 2.909ms | 0.000us | 5.48 MiB | none | 3.44M/s |
| q15 | group by two columns and top k | 2.000ms | 2.686ms | 2.703ms | 0.0% | 2.703ms | 2.703ms | 2.703ms | 2.703ms | 0.000us | 5.49 MiB | none | 3.70M/s |
| q16 | group by, very high card | 2.000ms | 3.704ms | 3.218ms | 0.0% | 3.218ms | 3.218ms | 3.218ms | 3.218ms | 0.000us | 6.26 MiB | none | 3.11M/s |
| q17 | group by two, very high card | 4.000ms | 4.714ms | 4.816ms | 0.0% | 4.816ms | 4.816ms | 4.816ms | 4.816ms | 0.000us | 7.32 MiB | none | 2.08M/s |
| q18 | group by two, no ordering | 3.000ms | 4.057ms | 4.027ms | 0.0% | 4.027ms | 4.027ms | 4.027ms | 4.027ms | 0.000us | 7.40 MiB | none | 2.48M/s |
| q20 | point lookup | 0.000us | 1.757ms | 1.509ms | 0.0% | 1.509ms | 1.509ms | 1.509ms | 1.509ms | 0.000us | 4.74 MiB | none | 6.63M/s |
| q21 | substring scan | 4.000ms | 4.981ms | 5.137ms | 0.0% | 5.137ms | 5.137ms | 5.137ms | 5.137ms | 0.000us | 6.40 MiB | none | 1.95M/s |
| q22 | substring scan and group by | 4.000ms | 5.979ms | 5.490ms | 0.0% | 5.490ms | 5.490ms | 5.490ms | 5.490ms | 0.000us | 6.46 MiB | none | 1.82M/s |
| q23 | two substring scans and group by | 9.000ms | 10.684ms | 10.802ms | 0.0% | 10.802ms | 10.802ms | 10.802ms | 10.802ms | 10.000ms | 8.71 MiB | none | 925.75K/s |
| q24 | select star and top k | 22.000ms | 22.412ms | 23.969ms | 0.0% | 23.969ms | 23.969ms | 23.969ms | 23.969ms | 10.000ms | 19.78 MiB | none | 417.21K/s |
| q25 | top k by a date | 1.000ms | 2.876ms | 2.564ms | 0.0% | 2.564ms | 2.564ms | 2.564ms | 2.564ms | 0.000us | 5.26 MiB | none | 3.90M/s |
| q26 | top k by a string | 1.000ms | 2.307ms | 2.342ms | 0.0% | 2.342ms | 2.342ms | 2.342ms | 2.342ms | 0.000us | 4.75 MiB | none | 4.27M/s |
| q27 | top k by two columns | 1.000ms | 2.703ms | 2.590ms | 0.0% | 2.590ms | 2.590ms | 2.590ms | 2.590ms | 0.000us | 4.95 MiB | none | 3.86M/s |
| q28 | group by with a string length | 4.000ms | 5.409ms | 5.258ms | 0.0% | 5.258ms | 5.258ms | 5.258ms | 5.258ms | 0.000us | 6.54 MiB | none | 1.90M/s |
| q29 | group by a regular expression | 9.000ms | 10.265ms | 10.503ms | 0.0% | 10.503ms | 10.503ms | 10.503ms | 10.503ms | 0.000us | 6.64 MiB | none | 952.11K/s |
| q30 | ninety sums over one column | 3.000ms | 3.803ms | 4.147ms | 0.0% | 4.147ms | 4.147ms | 4.147ms | 4.147ms | 0.000us | 4.98 MiB | none | 2.41M/s |
| q31 | group by two and several aggregates | 2.000ms | 2.988ms | 2.783ms | 0.0% | 2.783ms | 2.783ms | 2.783ms | 2.783ms | 0.000us | 5.75 MiB | none | 3.59M/s |
| q32 | group by a high card pair | 2.000ms | 2.729ms | 3.246ms | 0.0% | 3.246ms | 3.246ms | 3.246ms | 3.246ms | 0.000us | 5.74 MiB | none | 3.08M/s |
| q34 | group by a long string | 6.000ms | 8.251ms | 7.641ms | 0.0% | 7.641ms | 7.641ms | 7.641ms | 7.641ms | 0.000us | 7.88 MiB | none | 1.31M/s |
| q35 | group by a constant and a long string | 7.000ms | 7.856ms | 8.267ms | 0.0% | 8.267ms | 8.267ms | 8.267ms | 8.267ms | 0.000us | 8.52 MiB | none | 1.21M/s |
| q36 | group by four expressions | 4.000ms | 4.601ms | 5.098ms | 0.0% | 5.098ms | 5.098ms | 5.098ms | 5.098ms | 0.000us | 8.25 MiB | none | 1.96M/s |
| q37 | date range and group by a URL | 4.000ms | 5.066ms | 5.197ms | 0.0% | 5.197ms | 5.197ms | 5.197ms | 5.197ms | 0.000us | 6.64 MiB | none | 1.92M/s |
| q38 | date range and group by a title | 4.000ms | 6.015ms | 5.720ms | 0.0% | 5.720ms | 5.720ms | 5.720ms | 5.720ms | 0.000us | 6.73 MiB | none | 1.75M/s |
| q39 | date range, group by and offset | 4.000ms | 5.160ms | 5.279ms | 0.0% | 5.279ms | 5.279ms | 5.279ms | 5.279ms | 0.000us | 6.70 MiB | none | 1.89M/s |
| q40 | date range, a case and a wide group by | 7.000ms | 8.212ms | 8.354ms | 0.0% | 8.354ms | 8.354ms | 8.354ms | 8.354ms | 0.000us | 8.40 MiB | none | 1.20M/s |
| q41 | date range with an IN and a hash | 1.000ms | 2.129ms | 2.143ms | 0.0% | 2.143ms | 2.143ms | 2.143ms | 2.143ms | 0.000us | 5.25 MiB | none | 4.67M/s |
| q42 | date range and a deep offset | 1.000ms | 2.435ms | 2.319ms | 0.0% | 2.319ms | 2.319ms | 2.319ms | 2.319ms | 0.000us | 5.27 MiB | none | 4.31M/s |
| q43 | minute buckets over a date range | 1.000ms | 2.034ms | 2.075ms | 0.0% | 2.075ms | 2.075ms | 2.075ms | 2.075ms | 0.000us | 5.31 MiB | none | 4.82M/s |

rudb rudb 0.2.31 over 41 of 43 queries. Total 123.000ms by its own clock and 176.133ms by ours, 175.214ms cold, 20.000ms of CPU, peak 19.78 MiB, 3.33M/s and 565.73 MiB/s.

Running it cost 43% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 17.33x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 10000 rows, one out of every 10000 of the 99997497 in the full file, which is a development loop rather than the suite
- every query here ran within 1.93x of every other one, so most of what was timed is whatever they have in common rather than the queries
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

- duckdb ran every query within 1.93x of every other one, and this suite spreads over 6x on an engine it is measuring
- duckdb-pinned ran every query within 1.78x of every other one, and this suite spreads over 6x on an engine it is measuring
- clickhouse-local ran every query within 1.25x of every other one, and this suite spreads over 6x on an engine it is measuring
- polars ran every query within 1.44x of every other one, and this suite spreads over 6x on an engine it is measuring
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

