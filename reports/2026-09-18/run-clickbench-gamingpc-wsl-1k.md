# clickbench on gamingpc-wsl

This is one run of the clickbench suite on gamingpc-wsl, over 6 engines and 43 queries, with 5 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

It ran over 1000 rows, one out of every 99998 of the 99997497 in the full file, which is a development loop and not the suite. Nothing here is comparable to a full run or to anybody else's number.

## What ran

| what | it was |
| --- | --- |
| suite | clickbench |
| queries | 43 |
| tables | 293.41 KiB of Parquet in 1 table |
| rows | 1000 in the table every query reads |
| sample | 1000 rows, one out of every 99998 of the 99997497 in the full file |
| corpus | no manifest beside the data, so this run cannot say where it came from |
| summary | median with the interquartile range, per reporting rule two |
| timeout | 600s per query, and no query reached it |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --rows 1000 --engines duckdb,duckdb-pinned,clickhouse-local,datafusion,polars,rudb --runs 5 --timeout 600 --report
```

The data goes under `RUDB_BENCH_DATA`, which defaults to `~/rudb-data`, and the engines have to be on the path. An engine that is not gets a row in the abstentions below rather than being left out of the table.

## The machine

| fact | value | source |
| --- | --- | --- |
| host | GamingPC | read |
| os | Linux 6.18.33.2-microsoft-standard-WSL2 x86_64 | read |
| cpu | 13th Gen Intel(R) Core(TM) i9-13900K | read |
| threads | 32 | read |
| memory | 31.34 GiB | read |
| governor | no cpufreq here, the policy is not ours to see | not read |
| turbo | no intel_pstate here, the state is not ours to see | not read |
| filesystem | tmpfs /tmp tmpfs rw,nosuid,nodev,nr_inodes=1048576 0 0 | read |
| page cache | cannot open drop_caches: Permission denied (os error 13) | not read |
| timer | /usr/bin/time GNU | read |

A fact that says it was not read is a fact about this machine and not a gap in the report. A field that had silently defaulted would be a lie that survived into it.

## The engines

| engine | version | state | load | load cpu | on disk | that size is | format | machine load |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 52.568ms | 40.000ms | 1.01 MiB | its own database file | its own | 5.52 to 5.24 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 59.631ms | 50.000ms | 1.01 MiB | its own database file | its own | 5.24 to 4.59 |
| clickhouse-local | 26.9.1.1562 | ran | 156.354ms | 120.000ms | 362.94 KiB | its own MergeTree parts, as system.parts counts the active ones | its own | 4.59 to 3.77 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 293.41 KiB | the source Parquet, this engine has no storage format of its own | the Parquet | 3.77 to 3.70 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 293.41 KiB | the source Parquet, this engine has no storage format of its own | the Parquet | 3.70 to 2.99 |
| rudb | rudb 0.3.45 | ran | 0.000us | 0.000us | 293.41 KiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 2.99 to 2.83 |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb on 39 shared |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 98.000ms | 872.739ms | +791% | 873.336ms | 290.000ms | 0.33 | 38.00 MiB | none | 438.78K/s | 125.72 MiB/s | 1.00x |
| duckdb-pinned | 130.000ms | 1.736s | +1236% | 1.736s | 880.000ms | 0.51 | 53.36 MiB | none | 330.77K/s | 94.78 MiB/s | 1.34x |
| clickhouse-local | 364.000ms | 3.402s | +835% | 3.379s | 2.950s | 0.87 | 247.34 MiB | none | 118.13K/s | 33.85 MiB/s | 3.76x |
| datafusion | 203.000ms | 892.572ms | +340% | 933.233ms | 620.000ms | 0.69 | 156.72 MiB | none | 211.82K/s | 60.69 MiB/s | 2.05x |
| polars | 569.005ms | 4.429s | +678% | 4.371s | 5.680s | 1.28 | 83.79 MiB | none | 68.54K/s | 19.64 MiB/s | 6.47x |
| rudb | 33.153ms | 874.295ms | +2537% | 875.200ms | 0.000us | 0.00 | 10.24 MiB | none | 1.30M/s | 371.63 MiB/s | 0.34x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 1.000ms | 5.000ms | 1.000ms | 6.622ms | 517.674us |
| q2 | filtered count | 1.000ms | 1.000ms | 5.000ms | 3.000ms | 6.320ms | 580.130us |
| q3 | three aggregates | 1.000ms | 1.000ms | 6.000ms | 2.000ms | 6.671ms | 567.292us |
| q4 | average | 1.000ms | 1.000ms | 6.000ms | 1.000ms | 6.211ms | 520.615us |
| q5 | count distinct, high card | 2.000ms | 2.000ms | 6.000ms | 3.000ms | 12.590ms | 555.807us |
| q6 | count distinct, strings | 3.000ms | 2.000ms | 6.000ms | 4.000ms | 13.465ms | 596.592us |
| q7 | min and max of a date | 0.000us | 1.000ms | 11.000ms | 1.000ms | 6.340ms | 540.331us |
| q8 | group by, low card | 1.000ms | 5.000ms | 12.000ms | 4.000ms | 13.538ms | 605.392us |
| q9 | group by and count distinct | 4.000ms | 3.000ms | 6.000ms | 6.000ms | 21.086ms | 611.188us |
| q10 | group by, several aggregates | 4.000ms | 4.000ms | 7.000ms | 5.000ms | 26.629ms | 782.474us |
| q11 | group by a string and count distinct | 3.000ms | 4.000ms | 6.000ms | 9.000ms | 19.590ms | 597.764us |
| q12 | group by two strings and count distinct | 4.000ms | 4.000ms | 7.000ms | 8.000ms | 20.558ms | 675.009us |
| q13 | group by a string and top k | 2.000ms | 2.000ms | 7.000ms | 5.000ms | 15.130ms | 618.312us |
| q14 | group by a string and count distinct | 4.000ms | 4.000ms | 7.000ms | 7.000ms | 20.318ms | 647.822us |
| q15 | group by two columns and top k | 2.000ms | 3.000ms | 7.000ms | 5.000ms | 15.865ms | 676.878us |
| q16 | group by, very high card | 2.000ms | 2.000ms | 6.000ms | 3.000ms | 19.867ms | 620.951us |
| q17 | group by two, very high card | 3.000ms | 2.000ms | 6.000ms | 4.000ms | 20.706ms | 698.910us |
| q18 | group by two, no ordering | 3.000ms | 2.000ms | 6.000ms | 4.000ms | 12.802ms | 619.124us |
| q19 | group by with an extract | 3.000ms | 3.000ms | 7.000ms | 5.000ms | 22.952ms | 746.850us |
| q20 | point lookup | 1.000ms | 1.000ms | 12.000ms | 2.000ms | 6.561ms | 513.548us |
| q21 | substring scan | 1.000ms | 1.000ms | 6.000ms | 3.000ms | 7.366ms | 666.823us |
| q22 | substring scan and group by | 1.000ms | 2.000ms | 7.000ms | 5.000ms | 14.202ms | 763.259us |
| q23 | two substring scans and group by | 2.000ms | 3.000ms | 7.000ms | 5.000ms | 19.936ms | 1.010ms |
| q24 | select star and top k | 4.000ms | 9.000ms | 15.000ms | 7.000ms | 10.249ms | 958.894us |
| q25 | top k by a date | 2.000ms | 1.000ms | 12.000ms | 3.000ms | 12.714ms | 625.196us |
| q26 | top k by a string | 1.000ms | 1.000ms | 6.000ms | 3.000ms | 11.086ms | 569.143us |
| q27 | top k by two columns | 2.000ms | 2.000ms | 12.000ms | 3.000ms | 12.024ms | 635.579us |
| q28 | group by with a string length | 2.000ms | 2.000ms | 7.000ms | 6.000ms | no dialect | 763.590us |
| q29 | group by a regular expression | 3.000ms | 4.000ms | 7.000ms | 7.000ms | no dialect | 945.355us |
| q30 | ninety sums over one column | 4.000ms | 14.000ms | 10.000ms | 10.000ms | 11.427ms | 1.959ms |
| q31 | group by two and several aggregates | 3.000ms | 3.000ms | 7.000ms | 6.000ms | 17.821ms | 1.390ms |
| q32 | group by a high card pair | 3.000ms | 3.000ms | 7.000ms | 6.000ms | 17.874ms | 1.249ms |
| q33 | group by a high card pair, unfiltered | 3.000ms | 3.000ms | 6.000ms | 4.000ms | 17.687ms | 1.332ms |
| q34 | group by a long string | 3.000ms | 3.000ms | 6.000ms | 4.000ms | 15.727ms | 826.143us |
| q35 | group by a constant and a long string | 2.000ms | 3.000ms | 7.000ms | 4.000ms | 18.361ms | 865.955us |
| q36 | group by four expressions | 3.000ms | 3.000ms | 6.000ms | 4.000ms | no dialect | 686.243us |
| q37 | date range and group by a URL | 2.000ms | 3.000ms | 14.000ms | 6.000ms | 15.805ms | 837.161us |
| q38 | date range and group by a title | 2.000ms | 3.000ms | 14.000ms | 6.000ms | 16.482ms | 890.024us |
| q39 | date range, group by and offset | 1.000ms | 2.000ms | 13.000ms | 6.000ms | 14.332ms | 776.633us |
| q40 | date range, a case and a wide group by | 3.000ms | 4.000ms | 14.000ms | 6.000ms | 14.326ms | 958.881us |
| q41 | date range with an IN and a hash | 2.000ms | 3.000ms | 14.000ms | 6.000ms | 14.352ms | 712.338us |
| q42 | date range and a deep offset | 2.000ms | 7.000ms | 13.000ms | 5.000ms | 13.413ms | 716.313us |
| q43 | minute buckets over a date range | 2.000ms | 3.000ms | 13.000ms | 6.000ms | no dialect | 722.568us |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 20.266ms | 20.320ms | 0.1% | 20.312ms | 20.328ms | 20.279ms | 20.409ms | 0.000us | 26.93 MiB | none | 49.21K/s |
| q2 | filtered count | 1.000ms | 20.288ms | 20.292ms | 0.1% | 20.285ms | 20.311ms | 20.264ms | 20.314ms | 0.000us | 27.99 MiB | none | 49.28K/s |
| q3 | three aggregates | 1.000ms | 20.290ms | 20.278ms | 0.1% | 20.273ms | 20.299ms | 20.269ms | 20.305ms | 0.000us | 28.23 MiB | none | 49.31K/s |
| q4 | average | 1.000ms | 20.320ms | 20.297ms | 0.1% | 20.296ms | 20.322ms | 20.286ms | 20.359ms | 10.000ms | 27.75 MiB | none | 49.27K/s |
| q5 | count distinct, high card | 2.000ms | 20.306ms | 20.274ms | 0.0% | 20.267ms | 20.275ms | 20.255ms | 20.286ms | 0.000us | 29.74 MiB | none | 49.32K/s |
| q6 | count distinct, strings | 3.000ms | 20.275ms | 20.303ms | 1.3% | 20.280ms | 20.552ms | 20.269ms | 20.982ms | 0.000us | 30.19 MiB | none | 49.25K/s |
| q7 | min and max of a date | 0.000us | 20.391ms | 20.319ms | 0.4% | 20.317ms | 20.401ms | 20.298ms | 20.446ms | 0.000us | 27.25 MiB | none | 49.21K/s |
| q8 | group by, low card | 1.000ms | 20.359ms | 20.340ms | 0.2% | 20.312ms | 20.355ms | 20.299ms | 20.363ms | 0.000us | 29.50 MiB | none | 49.16K/s |
| q9 | group by and count distinct | 4.000ms | 20.322ms | 20.314ms | 0.0% | 20.312ms | 20.317ms | 20.301ms | 20.395ms | 10.000ms | 35.93 MiB | none | 49.23K/s |
| q10 | group by, several aggregates | 4.000ms | 20.297ms | 20.284ms | 0.1% | 20.281ms | 20.299ms | 20.279ms | 20.306ms | 10.000ms | 38.00 MiB | none | 49.30K/s |
| q11 | group by a string and count distinct | 3.000ms | 20.317ms | 20.283ms | 0.2% | 20.278ms | 20.314ms | 20.273ms | 20.330ms | 10.000ms | 35.75 MiB | none | 49.30K/s |
| q12 | group by two strings and count distinct | 4.000ms | 20.337ms | 20.320ms | 0.1% | 20.293ms | 20.321ms | 20.292ms | 20.357ms | 10.000ms | 35.69 MiB | none | 49.21K/s |
| q13 | group by a string and top k | 2.000ms | 20.293ms | 20.301ms | 0.1% | 20.299ms | 20.321ms | 20.282ms | 20.340ms | 10.000ms | 31.54 MiB | none | 49.26K/s |
| q14 | group by a string and count distinct | 4.000ms | 20.291ms | 20.310ms | 0.0% | 20.307ms | 20.312ms | 20.305ms | 20.313ms | 10.000ms | 35.93 MiB | none | 49.24K/s |
| q15 | group by two columns and top k | 2.000ms | 20.306ms | 20.300ms | 0.1% | 20.286ms | 20.304ms | 20.286ms | 20.306ms | 10.000ms | 32.00 MiB | none | 49.26K/s |
| q16 | group by, very high card | 2.000ms | 20.293ms | 20.294ms | 0.1% | 20.292ms | 20.312ms | 20.286ms | 20.337ms | 10.000ms | 32.86 MiB | none | 49.28K/s |
| q17 | group by two, very high card | 3.000ms | 20.386ms | 20.291ms | 0.1% | 20.280ms | 20.291ms | 20.261ms | 20.300ms | 10.000ms | 33.86 MiB | none | 49.28K/s |
| q18 | group by two, no ordering | 3.000ms | 20.269ms | 20.309ms | 0.2% | 20.296ms | 20.339ms | 20.277ms | 20.418ms | 10.000ms | 34.86 MiB | none | 49.24K/s |
| q19 | group by with an extract | 3.000ms | 20.281ms | 20.297ms | 0.0% | 20.294ms | 20.300ms | 20.288ms | 20.324ms | 10.000ms | 33.92 MiB | none | 49.27K/s |
| q20 | point lookup | 1.000ms | 20.312ms | 20.298ms | 0.1% | 20.284ms | 20.305ms | 20.270ms | 20.346ms | 0.000us | 27.49 MiB | none | 49.27K/s |
| q21 | substring scan | 1.000ms | 20.320ms | 20.310ms | 0.2% | 20.283ms | 20.323ms | 20.248ms | 20.508ms | 10.000ms | 27.99 MiB | none | 49.24K/s |
| q22 | substring scan and group by | 1.000ms | 20.273ms | 20.298ms | 0.1% | 20.281ms | 20.301ms | 20.276ms | 20.351ms | 0.000us | 28.75 MiB | none | 49.27K/s |
| q23 | two substring scans and group by | 2.000ms | 20.288ms | 20.329ms | 0.2% | 20.301ms | 20.332ms | 20.285ms | 20.411ms | 0.000us | 29.45 MiB | none | 49.19K/s |
| q24 | select star and top k | 4.000ms | 20.273ms | 20.272ms | 0.0% | 20.272ms | 20.276ms | 20.256ms | 20.292ms | 0.000us | 34.10 MiB | none | 49.33K/s |
| q25 | top k by a date | 2.000ms | 20.314ms | 20.293ms | 0.0% | 20.285ms | 20.294ms | 20.279ms | 20.356ms | 0.000us | 30.50 MiB | none | 49.28K/s |
| q26 | top k by a string | 1.000ms | 20.613ms | 20.288ms | 0.1% | 20.285ms | 20.306ms | 20.274ms | 20.367ms | 10.000ms | 27.74 MiB | none | 49.29K/s |
| q27 | top k by two columns | 2.000ms | 20.297ms | 20.292ms | 0.1% | 20.289ms | 20.306ms | 20.284ms | 20.309ms | 0.000us | 28.24 MiB | none | 49.28K/s |
| q28 | group by with a string length | 2.000ms | 20.266ms | 20.282ms | 0.1% | 20.280ms | 20.296ms | 20.265ms | 20.303ms | 10.000ms | 31.49 MiB | none | 49.31K/s |
| q29 | group by a regular expression | 3.000ms | 20.338ms | 20.308ms | 0.7% | 20.290ms | 20.434ms | 20.286ms | 20.441ms | 10.000ms | 32.48 MiB | none | 49.24K/s |
| q30 | ninety sums over one column | 4.000ms | 20.308ms | 20.282ms | 0.1% | 20.275ms | 20.295ms | 20.272ms | 20.341ms | 10.000ms | 31.49 MiB | none | 49.31K/s |
| q31 | group by two and several aggregates | 3.000ms | 20.300ms | 20.315ms | 0.1% | 20.307ms | 20.320ms | 20.285ms | 20.326ms | 10.000ms | 33.86 MiB | none | 49.22K/s |
| q32 | group by a high card pair | 3.000ms | 20.289ms | 20.305ms | 0.1% | 20.297ms | 20.310ms | 20.296ms | 20.361ms | 10.000ms | 33.36 MiB | none | 49.25K/s |
| q33 | group by a high card pair, unfiltered | 3.000ms | 20.306ms | 20.309ms | 0.1% | 20.304ms | 20.326ms | 20.287ms | 20.452ms | 10.000ms | 33.39 MiB | none | 49.24K/s |
| q34 | group by a long string | 3.000ms | 20.396ms | 20.278ms | 0.0% | 20.278ms | 20.286ms | 20.275ms | 20.288ms | 10.000ms | 31.98 MiB | none | 49.31K/s |
| q35 | group by a constant and a long string | 2.000ms | 20.268ms | 20.282ms | 0.0% | 20.276ms | 20.282ms | 20.271ms | 20.294ms | 10.000ms | 32.75 MiB | none | 49.31K/s |
| q36 | group by four expressions | 3.000ms | 20.259ms | 20.278ms | 0.1% | 20.277ms | 20.292ms | 20.276ms | 20.349ms | 10.000ms | 32.89 MiB | none | 49.32K/s |
| q37 | date range and group by a URL | 2.000ms | 20.266ms | 20.278ms | 0.0% | 20.276ms | 20.279ms | 20.267ms | 20.297ms | 10.000ms | 31.28 MiB | none | 49.32K/s |
| q38 | date range and group by a title | 2.000ms | 20.290ms | 20.292ms | 0.1% | 20.275ms | 20.299ms | 20.272ms | 20.334ms | 0.000us | 31.50 MiB | none | 49.28K/s |
| q39 | date range, group by and offset | 1.000ms | 20.301ms | 20.317ms | 0.2% | 20.302ms | 20.339ms | 20.291ms | 20.383ms | 10.000ms | 28.75 MiB | none | 49.22K/s |
| q40 | date range, a case and a wide group by | 3.000ms | 20.267ms | 20.276ms | 0.1% | 20.273ms | 20.286ms | 20.268ms | 20.297ms | 10.000ms | 32.50 MiB | none | 49.32K/s |
| q41 | date range with an IN and a hash | 2.000ms | 20.291ms | 20.285ms | 0.1% | 20.281ms | 20.297ms | 20.277ms | 20.309ms | 10.000ms | 32.92 MiB | none | 49.30K/s |
| q42 | date range and a deep offset | 2.000ms | 20.342ms | 20.267ms | 0.1% | 20.265ms | 20.277ms | 20.265ms | 20.286ms | 10.000ms | 31.00 MiB | none | 49.34K/s |
| q43 | minute buckets over a date range | 2.000ms | 20.273ms | 20.280ms | 0.0% | 20.278ms | 20.284ms | 20.274ms | 20.359ms | 10.000ms | 31.23 MiB | none | 49.31K/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 98.000ms by its own clock and 872.739ms by ours, 873.336ms cold, 290.000ms of CPU, peak 38.00 MiB, 438.78K/s and 125.72 MiB/s.

Running it cost 791% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.00x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 40.350ms | 40.420ms | 0.1% | 40.390ms | 40.426ms | 40.358ms | 40.862ms | 20.000ms | 40.00 MiB | none | 24.74K/s |
| q2 | filtered count | 1.000ms | 40.376ms | 40.373ms | 0.0% | 40.366ms | 40.374ms | 40.365ms | 40.431ms | 20.000ms | 40.10 MiB | none | 24.77K/s |
| q3 | three aggregates | 1.000ms | 40.384ms | 40.375ms | 0.0% | 40.368ms | 40.383ms | 40.363ms | 40.400ms | 20.000ms | 40.54 MiB | none | 24.77K/s |
| q4 | average | 1.000ms | 40.374ms | 40.360ms | 0.0% | 40.359ms | 40.363ms | 40.338ms | 40.438ms | 20.000ms | 40.55 MiB | none | 24.78K/s |
| q5 | count distinct, high card | 2.000ms | 40.456ms | 40.382ms | 0.0% | 40.371ms | 40.390ms | 40.364ms | 40.423ms | 20.000ms | 43.35 MiB | none | 24.76K/s |
| q6 | count distinct, strings | 2.000ms | 40.357ms | 40.385ms | 0.0% | 40.379ms | 40.388ms | 40.378ms | 40.452ms | 20.000ms | 41.54 MiB | none | 24.76K/s |
| q7 | min and max of a date | 1.000ms | 40.361ms | 40.405ms | 0.1% | 40.376ms | 40.419ms | 40.364ms | 40.439ms | 20.000ms | 40.11 MiB | none | 24.75K/s |
| q8 | group by, low card | 5.000ms | 40.411ms | 40.358ms | 0.0% | 40.355ms | 40.362ms | 40.352ms | 40.365ms | 30.000ms | 43.11 MiB | none | 24.78K/s |
| q9 | group by and count distinct | 3.000ms | 40.353ms | 40.382ms | 0.0% | 40.382ms | 40.385ms | 40.366ms | 40.427ms | 20.000ms | 48.80 MiB | none | 24.76K/s |
| q10 | group by, several aggregates | 4.000ms | 40.371ms | 40.379ms | 0.0% | 40.367ms | 40.382ms | 40.364ms | 40.393ms | 30.000ms | 48.87 MiB | none | 24.77K/s |
| q11 | group by a string and count distinct | 4.000ms | 40.441ms | 40.390ms | 0.1% | 40.381ms | 40.415ms | 40.372ms | 40.539ms | 20.000ms | 46.85 MiB | none | 24.76K/s |
| q12 | group by two strings and count distinct | 4.000ms | 40.371ms | 40.357ms | 0.1% | 40.356ms | 40.381ms | 40.355ms | 40.396ms | 20.000ms | 48.41 MiB | none | 24.78K/s |
| q13 | group by a string and top k | 2.000ms | 40.414ms | 40.414ms | 0.2% | 40.380ms | 40.442ms | 40.370ms | 40.467ms | 20.000ms | 42.80 MiB | none | 24.74K/s |
| q14 | group by a string and count distinct | 4.000ms | 40.392ms | 40.404ms | 0.1% | 40.365ms | 40.408ms | 40.357ms | 40.435ms | 20.000ms | 47.59 MiB | none | 24.75K/s |
| q15 | group by two columns and top k | 3.000ms | 40.360ms | 40.443ms | 0.3% | 40.394ms | 40.531ms | 40.358ms | 40.590ms | 20.000ms | 43.59 MiB | none | 24.73K/s |
| q16 | group by, very high card | 2.000ms | 40.354ms | 40.369ms | 0.1% | 40.367ms | 40.411ms | 40.354ms | 40.574ms | 20.000ms | 43.89 MiB | none | 24.77K/s |
| q17 | group by two, very high card | 2.000ms | 40.372ms | 40.378ms | 0.0% | 40.372ms | 40.384ms | 40.369ms | 40.401ms | 20.000ms | 44.17 MiB | none | 24.77K/s |
| q18 | group by two, no ordering | 2.000ms | 40.377ms | 40.369ms | 0.1% | 40.354ms | 40.379ms | 40.353ms | 40.389ms | 20.000ms | 44.42 MiB | none | 24.77K/s |
| q19 | group by with an extract | 3.000ms | 40.370ms | 40.387ms | 0.1% | 40.381ms | 40.403ms | 40.376ms | 40.413ms | 20.000ms | 44.95 MiB | none | 24.76K/s |
| q20 | point lookup | 1.000ms | 40.604ms | 40.403ms | 0.1% | 40.366ms | 40.410ms | 40.353ms | 40.425ms | 20.000ms | 39.54 MiB | none | 24.75K/s |
| q21 | substring scan | 1.000ms | 40.387ms | 40.353ms | 0.0% | 40.344ms | 40.362ms | 40.335ms | 40.382ms | 20.000ms | 40.60 MiB | none | 24.78K/s |
| q22 | substring scan and group by | 2.000ms | 40.366ms | 40.362ms | 0.0% | 40.358ms | 40.376ms | 40.350ms | 40.378ms | 20.000ms | 41.17 MiB | none | 24.78K/s |
| q23 | two substring scans and group by | 3.000ms | 40.353ms | 40.376ms | 0.0% | 40.371ms | 40.382ms | 40.358ms | 40.383ms | 20.000ms | 43.08 MiB | none | 24.77K/s |
| q24 | select star and top k | 9.000ms | 40.354ms | 40.359ms | 0.0% | 40.353ms | 40.363ms | 40.344ms | 40.400ms | 30.000ms | 48.16 MiB | none | 24.78K/s |
| q25 | top k by a date | 1.000ms | 40.403ms | 40.368ms | 0.0% | 40.363ms | 40.371ms | 40.358ms | 40.386ms | 10.000ms | 40.57 MiB | none | 24.77K/s |
| q26 | top k by a string | 1.000ms | 40.367ms | 40.369ms | 0.0% | 40.363ms | 40.378ms | 40.362ms | 40.396ms | 10.000ms | 40.54 MiB | none | 24.77K/s |
| q27 | top k by two columns | 2.000ms | 40.363ms | 40.376ms | 0.1% | 40.364ms | 40.391ms | 40.356ms | 40.431ms | 10.000ms | 40.79 MiB | none | 24.77K/s |
| q28 | group by with a string length | 2.000ms | 40.389ms | 40.375ms | 0.1% | 40.365ms | 40.410ms | 40.357ms | 40.430ms | 20.000ms | 43.16 MiB | none | 24.77K/s |
| q29 | group by a regular expression | 4.000ms | 40.359ms | 40.363ms | 0.0% | 40.359ms | 40.363ms | 40.336ms | 40.368ms | 20.000ms | 44.11 MiB | none | 24.78K/s |
| q30 | ninety sums over one column | 14.000ms | 40.377ms | 40.342ms | 0.0% | 40.341ms | 40.351ms | 40.335ms | 40.398ms | 30.000ms | 53.36 MiB | none | 24.79K/s |
| q31 | group by two and several aggregates | 3.000ms | 40.361ms | 40.362ms | 0.0% | 40.361ms | 40.375ms | 40.348ms | 40.379ms | 20.000ms | 45.16 MiB | none | 24.78K/s |
| q32 | group by a high card pair | 3.000ms | 40.364ms | 40.373ms | 0.1% | 40.370ms | 40.403ms | 40.363ms | 40.426ms | 20.000ms | 45.17 MiB | none | 24.77K/s |
| q33 | group by a high card pair, unfiltered | 3.000ms | 40.362ms | 40.368ms | 0.0% | 40.362ms | 40.379ms | 40.361ms | 40.398ms | 20.000ms | 45.17 MiB | none | 24.77K/s |
| q34 | group by a long string | 3.000ms | 40.478ms | 40.382ms | 0.1% | 40.363ms | 40.387ms | 40.363ms | 40.436ms | 20.000ms | 42.86 MiB | none | 24.76K/s |
| q35 | group by a constant and a long string | 3.000ms | 40.362ms | 40.370ms | 0.0% | 40.370ms | 40.374ms | 40.358ms | 40.486ms | 20.000ms | 42.87 MiB | none | 24.77K/s |
| q36 | group by four expressions | 3.000ms | 40.388ms | 40.357ms | 0.1% | 40.350ms | 40.397ms | 40.349ms | 40.421ms | 20.000ms | 44.61 MiB | none | 24.78K/s |
| q37 | date range and group by a URL | 3.000ms | 40.371ms | 40.360ms | 0.0% | 40.356ms | 40.360ms | 40.354ms | 40.363ms | 20.000ms | 43.36 MiB | none | 24.78K/s |
| q38 | date range and group by a title | 3.000ms | 40.376ms | 40.363ms | 0.0% | 40.359ms | 40.364ms | 40.342ms | 40.367ms | 20.000ms | 43.05 MiB | none | 24.78K/s |
| q39 | date range, group by and offset | 2.000ms | 40.354ms | 40.374ms | 0.0% | 40.363ms | 40.374ms | 40.362ms | 40.462ms | 20.000ms | 41.61 MiB | none | 24.77K/s |
| q40 | date range, a case and a wide group by | 4.000ms | 40.363ms | 40.367ms | 0.1% | 40.366ms | 40.399ms | 40.351ms | 40.403ms | 20.000ms | 44.61 MiB | none | 24.77K/s |
| q41 | date range with an IN and a hash | 3.000ms | 40.375ms | 40.365ms | 0.0% | 40.363ms | 40.375ms | 40.361ms | 40.384ms | 20.000ms | 45.18 MiB | none | 24.77K/s |
| q42 | date range and a deep offset | 7.000ms | 40.353ms | 40.364ms | 0.1% | 40.359ms | 40.401ms | 40.342ms | 40.568ms | 30.000ms | 44.39 MiB | none | 24.77K/s |
| q43 | minute buckets over a date range | 3.000ms | 40.372ms | 40.388ms | 0.0% | 40.380ms | 40.390ms | 40.353ms | 40.396ms | 20.000ms | 42.54 MiB | none | 24.76K/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 130.000ms by its own clock and 1.736s by ours, 1.736s cold, 880.000ms of CPU, peak 53.36 MiB, 330.77K/s and 94.78 MiB/s.

Running it cost 1236% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.00x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 5.000ms | 60.443ms | 80.500ms | 24.9% | 60.482ms | 80.510ms | 60.451ms | 80.547ms | 60.000ms | 238.85 MiB | none | 12.42K/s |
| q2 | filtered count | 5.000ms | 80.515ms | 80.588ms | 0.1% | 80.525ms | 80.616ms | 80.515ms | 80.666ms | 70.000ms | 239.78 MiB | none | 12.41K/s |
| q3 | three aggregates | 6.000ms | 80.520ms | 80.518ms | 0.0% | 80.517ms | 80.529ms | 60.447ms | 80.543ms | 70.000ms | 241.76 MiB | none | 12.42K/s |
| q4 | average | 6.000ms | 60.433ms | 80.504ms | 0.0% | 80.504ms | 80.521ms | 80.502ms | 80.531ms | 60.000ms | 241.68 MiB | none | 12.42K/s |
| q5 | count distinct, high card | 6.000ms | 80.557ms | 80.500ms | 24.8% | 60.545ms | 80.502ms | 60.432ms | 80.531ms | 60.000ms | 240.79 MiB | none | 12.42K/s |
| q6 | count distinct, strings | 6.000ms | 80.499ms | 60.433ms | 33.2% | 60.421ms | 80.490ms | 60.418ms | 80.505ms | 70.000ms | 240.85 MiB | none | 16.55K/s |
| q7 | min and max of a date | 11.000ms | 80.486ms | 80.556ms | 0.1% | 80.538ms | 80.584ms | 80.508ms | 80.610ms | 70.000ms | 241.34 MiB | none | 12.41K/s |
| q8 | group by, low card | 12.000ms | 80.700ms | 80.533ms | 0.0% | 80.517ms | 80.550ms | 80.508ms | 80.563ms | 70.000ms | 243.77 MiB | none | 12.42K/s |
| q9 | group by and count distinct | 6.000ms | 80.512ms | 80.527ms | 0.0% | 80.523ms | 80.558ms | 80.503ms | 80.575ms | 70.000ms | 242.94 MiB | none | 12.42K/s |
| q10 | group by, several aggregates | 7.000ms | 80.886ms | 80.521ms | 0.0% | 80.518ms | 80.522ms | 80.516ms | 80.571ms | 70.000ms | 243.16 MiB | none | 12.42K/s |
| q11 | group by a string and count distinct | 6.000ms | 80.543ms | 80.522ms | 0.0% | 80.507ms | 80.547ms | 60.426ms | 80.640ms | 70.000ms | 244.33 MiB | none | 12.42K/s |
| q12 | group by two strings and count distinct | 7.000ms | 80.547ms | 80.514ms | 0.0% | 80.506ms | 80.523ms | 80.502ms | 80.565ms | 70.000ms | 244.18 MiB | none | 12.42K/s |
| q13 | group by a string and top k | 7.000ms | 80.501ms | 80.523ms | 0.1% | 80.512ms | 80.571ms | 80.507ms | 83.182ms | 70.000ms | 244.61 MiB | none | 12.42K/s |
| q14 | group by a string and count distinct | 7.000ms | 80.734ms | 80.524ms | 0.0% | 80.510ms | 80.526ms | 80.503ms | 80.543ms | 70.000ms | 243.86 MiB | none | 12.42K/s |
| q15 | group by two columns and top k | 7.000ms | 80.528ms | 80.548ms | 0.1% | 80.501ms | 80.554ms | 80.485ms | 80.635ms | 60.000ms | 244.43 MiB | none | 12.42K/s |
| q16 | group by, very high card | 6.000ms | 80.535ms | 80.525ms | 0.0% | 80.518ms | 80.553ms | 80.502ms | 80.567ms | 70.000ms | 243.05 MiB | none | 12.42K/s |
| q17 | group by two, very high card | 6.000ms | 80.537ms | 80.541ms | 0.1% | 80.522ms | 80.598ms | 80.503ms | 80.626ms | 60.000ms | 244.33 MiB | none | 12.42K/s |
| q18 | group by two, no ordering | 6.000ms | 60.477ms | 60.445ms | 33.2% | 60.427ms | 80.478ms | 60.423ms | 80.548ms | 60.000ms | 243.68 MiB | none | 16.54K/s |
| q19 | group by with an extract | 7.000ms | 80.509ms | 60.430ms | 0.0% | 60.423ms | 60.435ms | 60.421ms | 80.498ms | 60.000ms | 243.93 MiB | none | 16.55K/s |
| q20 | point lookup | 12.000ms | 80.520ms | 80.504ms | 0.0% | 80.497ms | 80.534ms | 80.490ms | 80.730ms | 70.000ms | 241.09 MiB | none | 12.42K/s |
| q21 | substring scan | 6.000ms | 80.563ms | 80.508ms | 0.0% | 80.503ms | 80.508ms | 80.499ms | 80.538ms | 70.000ms | 242.15 MiB | none | 12.42K/s |
| q22 | substring scan and group by | 7.000ms | 60.440ms | 80.503ms | 0.0% | 80.491ms | 80.520ms | 60.410ms | 80.566ms | 70.000ms | 243.20 MiB | none | 12.42K/s |
| q23 | two substring scans and group by | 7.000ms | 60.503ms | 80.517ms | 0.0% | 80.515ms | 80.522ms | 80.512ms | 80.532ms | 70.000ms | 244.42 MiB | none | 12.42K/s |
| q24 | select star and top k | 15.000ms | 80.515ms | 80.518ms | 0.0% | 80.494ms | 80.524ms | 80.485ms | 80.605ms | 70.000ms | 243.30 MiB | none | 12.42K/s |
| q25 | top k by a date | 12.000ms | 80.594ms | 80.511ms | 0.0% | 80.511ms | 80.540ms | 80.509ms | 80.554ms | 70.000ms | 243.47 MiB | none | 12.42K/s |
| q26 | top k by a string | 6.000ms | 96.713ms | 80.506ms | 24.8% | 60.507ms | 80.507ms | 60.438ms | 80.533ms | 60.000ms | 242.70 MiB | none | 12.42K/s |
| q27 | top k by two columns | 12.000ms | 80.521ms | 80.515ms | 0.0% | 80.508ms | 80.548ms | 80.491ms | 80.571ms | 80.000ms | 243.18 MiB | none | 12.42K/s |
| q28 | group by with a string length | 7.000ms | 80.498ms | 80.539ms | 0.0% | 80.523ms | 80.555ms | 60.423ms | 80.637ms | 70.000ms | 243.98 MiB | none | 12.42K/s |
| q29 | group by a regular expression | 7.000ms | 80.484ms | 80.524ms | 0.0% | 80.518ms | 80.548ms | 80.507ms | 80.565ms | 70.000ms | 244.84 MiB | none | 12.42K/s |
| q30 | ninety sums over one column | 10.000ms | 80.531ms | 80.505ms | 0.0% | 80.496ms | 80.517ms | 80.455ms | 82.755ms | 80.000ms | 244.33 MiB | none | 12.42K/s |
| q31 | group by two and several aggregates | 7.000ms | 80.549ms | 80.574ms | 0.2% | 80.521ms | 80.690ms | 80.513ms | 83.449ms | 80.000ms | 244.68 MiB | none | 12.41K/s |
| q32 | group by a high card pair | 7.000ms | 80.573ms | 80.534ms | 0.0% | 80.514ms | 80.552ms | 80.511ms | 80.573ms | 70.000ms | 245.41 MiB | none | 12.42K/s |
| q33 | group by a high card pair, unfiltered | 6.000ms | 80.545ms | 80.539ms | 0.1% | 80.525ms | 80.573ms | 80.499ms | 80.585ms | 60.000ms | 244.20 MiB | none | 12.42K/s |
| q34 | group by a long string | 6.000ms | 80.534ms | 80.514ms | 0.1% | 80.498ms | 80.583ms | 60.417ms | 80.595ms | 70.000ms | 244.23 MiB | none | 12.42K/s |
| q35 | group by a constant and a long string | 7.000ms | 80.513ms | 80.549ms | 0.1% | 80.508ms | 80.575ms | 80.497ms | 80.587ms | 70.000ms | 244.27 MiB | none | 12.41K/s |
| q36 | group by four expressions | 6.000ms | 80.538ms | 80.566ms | 0.1% | 80.509ms | 80.574ms | 80.494ms | 80.888ms | 70.000ms | 244.15 MiB | none | 12.41K/s |
| q37 | date range and group by a URL | 14.000ms | 80.609ms | 80.517ms | 0.0% | 80.514ms | 80.549ms | 80.506ms | 80.550ms | 70.000ms | 246.36 MiB | none | 12.42K/s |
| q38 | date range and group by a title | 14.000ms | 80.578ms | 80.562ms | 0.1% | 80.513ms | 80.572ms | 80.509ms | 80.642ms | 70.000ms | 246.20 MiB | none | 12.41K/s |
| q39 | date range, group by and offset | 13.000ms | 80.493ms | 80.504ms | 0.0% | 80.502ms | 80.507ms | 80.490ms | 80.516ms | 70.000ms | 246.07 MiB | none | 12.42K/s |
| q40 | date range, a case and a wide group by | 14.000ms | 80.507ms | 80.511ms | 0.0% | 80.510ms | 80.529ms | 80.477ms | 80.555ms | 70.000ms | 247.11 MiB | none | 12.42K/s |
| q41 | date range with an IN and a hash | 14.000ms | 80.566ms | 80.536ms | 0.1% | 80.501ms | 80.558ms | 80.489ms | 80.661ms | 70.000ms | 247.34 MiB | none | 12.42K/s |
| q42 | date range and a deep offset | 13.000ms | 80.517ms | 80.538ms | 0.0% | 80.506ms | 80.541ms | 80.498ms | 80.551ms | 70.000ms | 246.95 MiB | none | 12.42K/s |
| q43 | minute buckets over a date range | 13.000ms | 80.523ms | 80.527ms | 0.0% | 80.514ms | 80.535ms | 80.510ms | 80.607ms | 70.000ms | 246.34 MiB | none | 12.42K/s |

clickhouse-local 26.9.1.1562 over 43 of 43 queries. Total 364.000ms by its own clock and 3.402s by ours, 3.379s cold, 2.950s of CPU, peak 247.34 MiB, 118.13K/s and 33.85 MiB/s.

Running it cost 835% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.33x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 20.290ms | 20.272ms | 0.0% | 20.272ms | 20.275ms | 20.270ms | 20.281ms | 10.000ms | 80.40 MiB | none | 49.33K/s |
| q2 | filtered count | 3.000ms | 20.321ms | 20.281ms | 0.0% | 20.272ms | 20.281ms | 20.270ms | 20.302ms | 10.000ms | 102.39 MiB | none | 49.31K/s |
| q3 | three aggregates | 2.000ms | 20.301ms | 20.276ms | 0.1% | 20.274ms | 20.289ms | 20.260ms | 20.306ms | 10.000ms | 81.90 MiB | none | 49.32K/s |
| q4 | average | 1.000ms | 20.265ms | 20.274ms | 0.1% | 20.267ms | 20.288ms | 20.264ms | 20.291ms | 10.000ms | 80.75 MiB | none | 49.32K/s |
| q5 | count distinct, high card | 3.000ms | 20.294ms | 20.289ms | 0.0% | 20.288ms | 20.291ms | 20.280ms | 20.306ms | 20.000ms | 115.07 MiB | none | 49.29K/s |
| q6 | count distinct, strings | 4.000ms | 20.275ms | 20.283ms | 0.1% | 20.280ms | 20.295ms | 20.268ms | 20.296ms | 20.000ms | 129.83 MiB | none | 49.30K/s |
| q7 | min and max of a date | 1.000ms | 20.283ms | 20.270ms | 0.1% | 20.265ms | 20.281ms | 20.230ms | 20.326ms | 0.000us | 80.02 MiB | none | 49.33K/s |
| q8 | group by, low card | 4.000ms | 40.426ms | 20.292ms | 0.1% | 20.289ms | 20.307ms | 20.280ms | 20.322ms | 10.000ms | 98.42 MiB | none | 49.28K/s |
| q9 | group by and count distinct | 6.000ms | 20.281ms | 20.299ms | 0.0% | 20.296ms | 20.299ms | 20.292ms | 20.307ms | 30.000ms | 126.03 MiB | none | 49.26K/s |
| q10 | group by, several aggregates | 5.000ms | 20.301ms | 20.294ms | 0.1% | 20.290ms | 20.303ms | 20.289ms | 20.321ms | 10.000ms | 120.28 MiB | none | 49.28K/s |
| q11 | group by a string and count distinct | 9.000ms | 20.282ms | 20.296ms | 0.1% | 20.282ms | 20.304ms | 20.279ms | 40.358ms | 40.000ms | 150.17 MiB | none | 49.27K/s |
| q12 | group by two strings and count distinct | 8.000ms | 20.311ms | 20.289ms | 0.1% | 20.289ms | 20.303ms | 20.286ms | 40.397ms | 40.000ms | 150.75 MiB | none | 49.29K/s |
| q13 | group by a string and top k | 5.000ms | 20.286ms | 20.286ms | 0.0% | 20.286ms | 20.289ms | 20.286ms | 20.372ms | 10.000ms | 129.09 MiB | none | 49.29K/s |
| q14 | group by a string and count distinct | 7.000ms | 20.287ms | 20.289ms | 0.1% | 20.285ms | 20.311ms | 20.282ms | 20.339ms | 20.000ms | 156.72 MiB | none | 49.29K/s |
| q15 | group by two columns and top k | 5.000ms | 20.283ms | 20.326ms | 0.2% | 20.293ms | 20.329ms | 20.292ms | 20.336ms | 10.000ms | 129.49 MiB | none | 49.20K/s |
| q16 | group by, very high card | 3.000ms | 20.310ms | 20.284ms | 0.2% | 20.280ms | 20.319ms | 20.276ms | 20.335ms | 10.000ms | 109.13 MiB | none | 49.30K/s |
| q17 | group by two, very high card | 4.000ms | 20.284ms | 20.297ms | 0.9% | 20.287ms | 20.476ms | 20.285ms | 40.380ms | 10.000ms | 115.13 MiB | none | 49.27K/s |
| q18 | group by two, no ordering | 4.000ms | 20.291ms | 20.287ms | 0.1% | 20.284ms | 20.304ms | 20.283ms | 20.337ms | 10.000ms | 117.04 MiB | none | 49.29K/s |
| q19 | group by with an extract | 5.000ms | 20.289ms | 20.295ms | 0.0% | 20.294ms | 20.300ms | 20.290ms | 20.366ms | 20.000ms | 119.95 MiB | none | 49.27K/s |
| q20 | point lookup | 2.000ms | 20.284ms | 20.283ms | 0.0% | 20.279ms | 20.285ms | 20.272ms | 20.299ms | 0.000us | 89.87 MiB | none | 49.30K/s |
| q21 | substring scan | 3.000ms | 20.350ms | 20.295ms | 0.1% | 20.293ms | 20.321ms | 20.277ms | 20.364ms | 10.000ms | 100.10 MiB | none | 49.27K/s |
| q22 | substring scan and group by | 5.000ms | 20.283ms | 20.281ms | 0.1% | 20.279ms | 20.295ms | 20.275ms | 20.320ms | 10.000ms | 115.87 MiB | none | 49.31K/s |
| q23 | two substring scans and group by | 5.000ms | 20.288ms | 20.286ms | 0.1% | 20.280ms | 20.294ms | 20.245ms | 20.346ms | 10.000ms | 122.77 MiB | none | 49.29K/s |
| q24 | select star and top k | 7.000ms | 20.283ms | 20.277ms | 0.0% | 20.273ms | 20.281ms | 20.271ms | 20.294ms | 10.000ms | 113.22 MiB | none | 49.32K/s |
| q25 | top k by a date | 3.000ms | 20.593ms | 20.303ms | 0.1% | 20.291ms | 20.314ms | 20.279ms | 20.325ms | 10.000ms | 103.95 MiB | none | 49.25K/s |
| q26 | top k by a string | 3.000ms | 20.334ms | 20.277ms | 0.0% | 20.276ms | 20.285ms | 20.269ms | 20.303ms | 10.000ms | 100.98 MiB | none | 49.32K/s |
| q27 | top k by two columns | 3.000ms | 20.332ms | 20.293ms | 0.1% | 20.286ms | 20.301ms | 20.282ms | 20.317ms | 10.000ms | 102.56 MiB | none | 49.28K/s |
| q28 | group by with a string length | 6.000ms | 20.291ms | 20.295ms | 0.0% | 20.288ms | 20.297ms | 20.282ms | 20.321ms | 10.000ms | 125.56 MiB | none | 49.27K/s |
| q29 | group by a regular expression | 7.000ms | 40.367ms | 20.302ms | 0.3% | 20.295ms | 20.354ms | 20.286ms | 20.400ms | 20.000ms | 144.68 MiB | none | 49.26K/s |
| q30 | ninety sums over one column | 10.000ms | 40.372ms | 40.376ms | 0.0% | 40.368ms | 40.381ms | 40.364ms | 40.386ms | 10.000ms | 88.56 MiB | none | 24.77K/s |
| q31 | group by two and several aggregates | 6.000ms | 20.293ms | 20.289ms | 0.2% | 20.285ms | 20.324ms | 20.285ms | 20.363ms | 20.000ms | 128.07 MiB | none | 49.29K/s |
| q32 | group by a high card pair | 6.000ms | 20.292ms | 20.290ms | 0.0% | 20.289ms | 20.294ms | 20.288ms | 20.368ms | 20.000ms | 131.30 MiB | none | 49.29K/s |
| q33 | group by a high card pair, unfiltered | 4.000ms | 20.295ms | 20.287ms | 0.0% | 20.281ms | 20.289ms | 20.280ms | 20.291ms | 10.000ms | 118.22 MiB | none | 49.29K/s |
| q34 | group by a long string | 4.000ms | 20.283ms | 20.298ms | 0.0% | 20.293ms | 20.299ms | 20.269ms | 20.309ms | 10.000ms | 123.64 MiB | none | 49.26K/s |
| q35 | group by a constant and a long string | 4.000ms | 20.298ms | 20.291ms | 0.0% | 20.291ms | 20.300ms | 20.290ms | 20.311ms | 20.000ms | 122.03 MiB | none | 49.28K/s |
| q36 | group by four expressions | 4.000ms | 20.302ms | 20.306ms | 0.0% | 20.300ms | 20.308ms | 20.282ms | 20.421ms | 10.000ms | 108.11 MiB | none | 49.25K/s |
| q37 | date range and group by a URL | 6.000ms | 20.307ms | 20.292ms | 0.0% | 20.289ms | 20.292ms | 20.288ms | 20.292ms | 20.000ms | 128.24 MiB | none | 49.28K/s |
| q38 | date range and group by a title | 6.000ms | 20.289ms | 20.302ms | 0.0% | 20.294ms | 20.303ms | 20.293ms | 20.305ms | 10.000ms | 122.84 MiB | none | 49.26K/s |
| q39 | date range, group by and offset | 6.000ms | 20.288ms | 20.297ms | 0.1% | 20.288ms | 20.303ms | 20.280ms | 20.310ms | 20.000ms | 120.05 MiB | none | 49.27K/s |
| q40 | date range, a case and a wide group by | 6.000ms | 20.284ms | 20.300ms | 0.1% | 20.294ms | 20.317ms | 20.286ms | 20.321ms | 10.000ms | 123.58 MiB | none | 49.26K/s |
| q41 | date range with an IN and a hash | 6.000ms | 20.279ms | 20.290ms | 0.1% | 20.284ms | 20.297ms | 20.276ms | 20.305ms | 20.000ms | 115.34 MiB | none | 49.29K/s |
| q42 | date range and a deep offset | 5.000ms | 20.284ms | 20.295ms | 0.1% | 20.290ms | 20.301ms | 20.279ms | 20.315ms | 20.000ms | 111.38 MiB | none | 49.27K/s |
| q43 | minute buckets over a date range | 6.000ms | 20.302ms | 20.290ms | 0.0% | 20.283ms | 20.291ms | 20.281ms | 20.296ms | 20.000ms | 116.65 MiB | none | 49.28K/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 203.000ms by its own clock and 892.572ms by ours, 933.233ms cold, 620.000ms of CPU, peak 156.72 MiB, 211.82K/s and 60.69 MiB/s.

Running it cost 340% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.99x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 6.622ms | 100.597ms | 100.626ms | 0.1% | 100.589ms | 100.647ms | 100.585ms | 100.666ms | 120.000ms | 61.77 MiB | none | 9.94K/s |
| q2 | filtered count | 6.320ms | 100.937ms | 100.606ms | 0.0% | 100.589ms | 100.610ms | 100.581ms | 100.639ms | 110.000ms | 65.87 MiB | none | 9.94K/s |
| q3 | three aggregates | 6.671ms | 100.606ms | 100.633ms | 20.1% | 100.611ms | 120.870ms | 100.580ms | 124.751ms | 100.000ms | 65.38 MiB | none | 9.94K/s |
| q4 | average | 6.211ms | 100.620ms | 120.739ms | 16.8% | 100.601ms | 120.890ms | 100.584ms | 121.188ms | 340.000ms | 63.39 MiB | none | 8.28K/s |
| q5 | count distinct, high card | 12.590ms | 120.694ms | 100.686ms | 0.1% | 100.630ms | 100.726ms | 100.607ms | 120.999ms | 120.000ms | 73.39 MiB | none | 9.93K/s |
| q6 | count distinct, strings | 13.465ms | 120.728ms | 120.678ms | 16.3% | 101.078ms | 120.768ms | 100.765ms | 120.856ms | 140.000ms | 73.78 MiB | none | 8.29K/s |
| q7 | min and max of a date | 6.340ms | 100.629ms | 100.599ms | 0.1% | 100.593ms | 100.656ms | 100.588ms | 100.662ms | 90.000ms | 64.16 MiB | none | 9.94K/s |
| q8 | group by, low card | 13.538ms | 100.816ms | 120.676ms | 0.0% | 120.671ms | 120.683ms | 100.685ms | 120.836ms | 140.000ms | 75.54 MiB | none | 8.29K/s |
| q9 | group by and count distinct | 21.086ms | 120.701ms | 120.880ms | 0.2% | 120.854ms | 121.060ms | 120.841ms | 121.419ms | 150.000ms | 81.41 MiB | none | 8.27K/s |
| q10 | group by, several aggregates | 26.629ms | 120.718ms | 120.741ms | 0.1% | 120.687ms | 120.762ms | 120.647ms | 121.048ms | 180.000ms | 82.80 MiB | none | 8.28K/s |
| q11 | group by a string and count distinct | 19.590ms | 120.761ms | 120.704ms | 0.0% | 120.659ms | 120.716ms | 120.636ms | 120.785ms | 150.000ms | 82.55 MiB | none | 8.28K/s |
| q12 | group by two strings and count distinct | 20.558ms | 120.688ms | 120.707ms | 0.2% | 120.662ms | 120.959ms | 120.660ms | 121.579ms | 140.000ms | 82.59 MiB | none | 8.28K/s |
| q13 | group by a string and top k | 15.130ms | 120.839ms | 120.905ms | 0.1% | 120.764ms | 120.913ms | 120.647ms | 121.446ms | 140.000ms | 75.78 MiB | none | 8.27K/s |
| q14 | group by a string and count distinct | 20.318ms | 120.676ms | 120.677ms | 0.2% | 120.668ms | 120.879ms | 120.657ms | 120.957ms | 170.000ms | 82.52 MiB | none | 8.29K/s |
| q15 | group by two columns and top k | 15.865ms | 120.869ms | 120.800ms | 0.1% | 120.762ms | 120.876ms | 120.730ms | 121.211ms | 140.000ms | 76.51 MiB | none | 8.28K/s |
| q16 | group by, very high card | 19.867ms | 120.685ms | 120.750ms | 0.1% | 120.723ms | 120.836ms | 120.689ms | 121.213ms | 140.000ms | 75.11 MiB | none | 8.28K/s |
| q17 | group by two, very high card | 20.706ms | 120.693ms | 120.769ms | 0.0% | 120.764ms | 120.781ms | 120.663ms | 120.974ms | 150.000ms | 76.13 MiB | none | 8.28K/s |
| q18 | group by two, no ordering | 12.802ms | 100.603ms | 100.706ms | 20.0% | 100.656ms | 120.762ms | 100.591ms | 120.848ms | 130.000ms | 73.26 MiB | none | 9.93K/s |
| q19 | group by with an extract | 22.952ms | 120.759ms | 120.727ms | 0.0% | 120.690ms | 120.743ms | 120.644ms | 120.934ms | 170.000ms | 77.84 MiB | none | 8.28K/s |
| q20 | point lookup | 6.561ms | 100.568ms | 120.748ms | 18.3% | 100.615ms | 122.711ms | 100.606ms | 123.168ms | 270.000ms | 64.00 MiB | none | 8.28K/s |
| q21 | substring scan | 7.366ms | 100.577ms | 100.601ms | 0.0% | 100.597ms | 100.607ms | 100.592ms | 100.609ms | 100.000ms | 68.20 MiB | none | 9.94K/s |
| q22 | substring scan and group by | 14.202ms | 120.787ms | 120.687ms | 16.5% | 100.724ms | 120.691ms | 100.662ms | 120.806ms | 130.000ms | 77.27 MiB | none | 8.29K/s |
| q23 | two substring scans and group by | 19.936ms | 120.685ms | 120.691ms | 0.0% | 120.660ms | 120.694ms | 120.646ms | 120.702ms | 150.000ms | 83.79 MiB | none | 8.29K/s |
| q24 | select star and top k | 10.249ms | 100.601ms | 100.602ms | 0.0% | 100.588ms | 100.615ms | 100.581ms | 120.733ms | 150.000ms | 73.23 MiB | none | 9.94K/s |
| q25 | top k by a date | 12.714ms | 100.678ms | 100.646ms | 0.0% | 100.639ms | 100.653ms | 100.600ms | 120.687ms | 120.000ms | 71.77 MiB | none | 9.94K/s |
| q26 | top k by a string | 11.086ms | 100.581ms | 100.596ms | 0.0% | 100.576ms | 100.600ms | 100.570ms | 100.607ms | 110.000ms | 70.99 MiB | none | 9.94K/s |
| q27 | top k by two columns | 12.024ms | 100.630ms | 100.647ms | 0.0% | 100.633ms | 100.651ms | 100.574ms | 130.595ms | 120.000ms | 71.97 MiB | none | 9.94K/s |
| q30 | ninety sums over one column | 11.427ms | 100.650ms | 100.560ms | 0.0% | 100.556ms | 100.577ms | 100.552ms | 100.607ms | 120.000ms | 67.41 MiB | none | 9.94K/s |
| q31 | group by two and several aggregates | 17.821ms | 120.975ms | 120.681ms | 16.3% | 100.978ms | 120.708ms | 100.721ms | 121.022ms | 130.000ms | 77.82 MiB | none | 8.29K/s |
| q32 | group by a high card pair | 17.874ms | 120.922ms | 121.586ms | 0.1% | 121.544ms | 121.630ms | 120.742ms | 124.485ms | 170.000ms | 78.71 MiB | none | 8.22K/s |
| q33 | group by a high card pair, unfiltered | 17.687ms | 122.060ms | 120.859ms | 0.2% | 120.744ms | 121.021ms | 120.712ms | 121.194ms | 160.000ms | 78.02 MiB | none | 8.27K/s |
| q34 | group by a long string | 15.727ms | 143.080ms | 100.686ms | 20.0% | 100.642ms | 120.736ms | 100.555ms | 121.915ms | 130.000ms | 74.63 MiB | none | 9.93K/s |
| q35 | group by a constant and a long string | 18.361ms | 120.769ms | 120.775ms | 0.8% | 120.762ms | 121.751ms | 120.647ms | 123.153ms | 150.000ms | 75.57 MiB | none | 8.28K/s |
| q37 | date range and group by a URL | 15.805ms | 100.601ms | 120.798ms | 0.1% | 120.736ms | 120.822ms | 101.189ms | 121.502ms | 150.000ms | 80.03 MiB | none | 8.28K/s |
| q38 | date range and group by a title | 16.482ms | 120.754ms | 120.743ms | 0.1% | 120.704ms | 120.807ms | 120.693ms | 121.080ms | 140.000ms | 78.59 MiB | none | 8.28K/s |
| q39 | date range, group by and offset | 14.332ms | 120.820ms | 120.871ms | 0.2% | 120.717ms | 120.928ms | 100.592ms | 121.095ms | 130.000ms | 76.98 MiB | none | 8.27K/s |
| q40 | date range, a case and a wide group by | 14.326ms | 100.598ms | 120.637ms | 16.6% | 100.626ms | 120.661ms | 100.619ms | 134.316ms | 150.000ms | 79.77 MiB | none | 8.29K/s |
| q41 | date range with an IN and a hash | 14.352ms | 100.587ms | 100.787ms | 19.9% | 100.640ms | 120.687ms | 100.589ms | 120.739ms | 130.000ms | 80.39 MiB | none | 9.92K/s |
| q42 | date range and a deep offset | 13.413ms | 100.593ms | 120.757ms | 0.1% | 120.688ms | 120.819ms | 100.572ms | 129.281ms | 150.000ms | 79.02 MiB | none | 8.28K/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 569.005ms by its own clock and 4.429s by ours, 4.371s cold, 5.680s of CPU, peak 83.79 MiB, 68.54K/s and 19.64 MiB/s.

Running it cost 678% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.21x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 517.674us | 20.398ms | 20.326ms | 0.1% | 20.312ms | 20.335ms | 20.310ms | 20.434ms | 0.000us | 9.29 MiB | none | 49.20K/s |
| q2 | filtered count | 580.130us | 20.368ms | 20.333ms | 0.1% | 20.304ms | 20.334ms | 20.300ms | 20.367ms | 0.000us | 9.73 MiB | none | 49.18K/s |
| q3 | three aggregates | 567.292us | 20.437ms | 20.297ms | 0.0% | 20.296ms | 20.304ms | 20.289ms | 20.325ms | 0.000us | 9.48 MiB | none | 49.27K/s |
| q4 | average | 520.615us | 20.308ms | 20.307ms | 0.1% | 20.302ms | 20.317ms | 20.288ms | 20.367ms | 0.000us | 9.30 MiB | none | 49.24K/s |
| q5 | count distinct, high card | 555.807us | 20.299ms | 20.308ms | 0.1% | 20.303ms | 20.315ms | 20.299ms | 20.339ms | 0.000us | 9.36 MiB | none | 49.24K/s |
| q6 | count distinct, strings | 596.592us | 20.304ms | 20.366ms | 0.2% | 20.317ms | 20.367ms | 20.305ms | 20.542ms | 0.000us | 9.40 MiB | none | 49.10K/s |
| q7 | min and max of a date | 540.331us | 20.315ms | 20.352ms | 0.3% | 20.324ms | 20.381ms | 20.308ms | 20.512ms | 0.000us | 9.41 MiB | none | 49.14K/s |
| q8 | group by, low card | 605.392us | 20.334ms | 20.618ms | 1.3% | 20.358ms | 20.623ms | 20.320ms | 20.638ms | 0.000us | 9.62 MiB | none | 48.50K/s |
| q9 | group by and count distinct | 611.188us | 20.322ms | 20.302ms | 0.1% | 20.301ms | 20.319ms | 20.276ms | 20.332ms | 0.000us | 9.33 MiB | none | 49.26K/s |
| q10 | group by, several aggregates | 782.474us | 20.317ms | 20.351ms | 0.2% | 20.331ms | 20.371ms | 20.319ms | 20.410ms | 0.000us | 9.55 MiB | none | 49.14K/s |
| q11 | group by a string and count distinct | 597.764us | 20.334ms | 20.311ms | 0.1% | 20.305ms | 20.325ms | 20.302ms | 20.330ms | 0.000us | 9.61 MiB | none | 49.23K/s |
| q12 | group by two strings and count distinct | 675.009us | 20.585ms | 20.306ms | 0.1% | 20.301ms | 20.316ms | 20.298ms | 20.329ms | 0.000us | 9.63 MiB | none | 49.25K/s |
| q13 | group by a string and top k | 618.312us | 20.311ms | 20.313ms | 0.2% | 20.305ms | 20.351ms | 20.297ms | 20.359ms | 0.000us | 9.54 MiB | none | 49.23K/s |
| q14 | group by a string and count distinct | 647.822us | 20.310ms | 20.332ms | 0.1% | 20.320ms | 20.339ms | 20.300ms | 20.351ms | 0.000us | 9.62 MiB | none | 49.18K/s |
| q15 | group by two columns and top k | 676.878us | 20.328ms | 20.327ms | 0.1% | 20.311ms | 20.334ms | 20.305ms | 20.344ms | 0.000us | 9.50 MiB | none | 49.20K/s |
| q16 | group by, very high card | 620.951us | 20.312ms | 20.315ms | 0.1% | 20.311ms | 20.321ms | 20.296ms | 20.325ms | 0.000us | 9.29 MiB | none | 49.22K/s |
| q17 | group by two, very high card | 698.910us | 20.289ms | 20.318ms | 0.1% | 20.312ms | 20.326ms | 20.296ms | 20.327ms | 0.000us | 9.38 MiB | none | 49.22K/s |
| q18 | group by two, no ordering | 619.124us | 20.296ms | 20.306ms | 0.1% | 20.304ms | 20.332ms | 20.301ms | 20.356ms | 0.000us | 9.25 MiB | none | 49.25K/s |
| q19 | group by with an extract | 746.850us | 20.397ms | 20.318ms | 0.2% | 20.306ms | 20.355ms | 20.299ms | 20.371ms | 0.000us | 9.62 MiB | none | 49.22K/s |
| q20 | point lookup | 513.548us | 20.322ms | 20.313ms | 0.0% | 20.310ms | 20.317ms | 20.285ms | 20.320ms | 0.000us | 9.44 MiB | none | 49.23K/s |
| q21 | substring scan | 666.823us | 20.297ms | 20.556ms | 0.4% | 20.533ms | 20.620ms | 20.533ms | 20.623ms | 0.000us | 9.67 MiB | none | 48.65K/s |
| q22 | substring scan and group by | 763.259us | 20.310ms | 20.316ms | 0.1% | 20.315ms | 20.332ms | 20.305ms | 20.350ms | 0.000us | 9.52 MiB | none | 49.22K/s |
| q23 | two substring scans and group by | 1.010ms | 20.390ms | 20.383ms | 0.4% | 20.338ms | 20.414ms | 20.312ms | 20.464ms | 0.000us | 9.75 MiB | none | 49.06K/s |
| q24 | select star and top k | 958.894us | 20.338ms | 20.322ms | 0.1% | 20.319ms | 20.335ms | 20.318ms | 20.351ms | 0.000us | 9.98 MiB | none | 49.21K/s |
| q25 | top k by a date | 625.196us | 20.356ms | 20.320ms | 0.1% | 20.306ms | 20.328ms | 20.299ms | 20.334ms | 0.000us | 9.70 MiB | none | 49.21K/s |
| q26 | top k by a string | 569.143us | 20.342ms | 20.306ms | 0.1% | 20.305ms | 20.330ms | 20.304ms | 20.612ms | 0.000us | 9.50 MiB | none | 49.25K/s |
| q27 | top k by two columns | 635.579us | 20.531ms | 20.302ms | 0.1% | 20.302ms | 20.314ms | 20.298ms | 20.339ms | 0.000us | 9.56 MiB | none | 49.26K/s |
| q28 | group by with a string length | 763.590us | 20.331ms | 20.316ms | 0.1% | 20.316ms | 20.332ms | 20.304ms | 20.342ms | 0.000us | 9.92 MiB | none | 49.22K/s |
| q29 | group by a regular expression | 945.355us | 20.325ms | 20.325ms | 0.1% | 20.314ms | 20.337ms | 20.305ms | 20.346ms | 0.000us | 10.01 MiB | none | 49.20K/s |
| q30 | ninety sums over one column | 1.959ms | 20.305ms | 20.315ms | 0.0% | 20.312ms | 20.321ms | 20.302ms | 20.322ms | 0.000us | 9.64 MiB | none | 49.22K/s |
| q31 | group by two and several aggregates | 1.390ms | 20.283ms | 20.338ms | 0.3% | 20.317ms | 20.374ms | 20.291ms | 20.386ms | 0.000us | 9.62 MiB | none | 49.17K/s |
| q32 | group by a high card pair | 1.249ms | 20.317ms | 20.312ms | 0.0% | 20.309ms | 20.318ms | 20.302ms | 20.337ms | 0.000us | 9.63 MiB | none | 49.23K/s |
| q33 | group by a high card pair, unfiltered | 1.332ms | 20.320ms | 20.313ms | 0.1% | 20.306ms | 20.324ms | 20.295ms | 20.338ms | 0.000us | 9.40 MiB | none | 49.23K/s |
| q34 | group by a long string | 826.143us | 20.602ms | 20.305ms | 0.1% | 20.297ms | 20.318ms | 20.295ms | 20.339ms | 0.000us | 9.67 MiB | none | 49.25K/s |
| q35 | group by a constant and a long string | 865.955us | 20.374ms | 20.306ms | 0.0% | 20.303ms | 20.310ms | 20.300ms | 20.316ms | 0.000us | 9.62 MiB | none | 49.25K/s |
| q36 | group by four expressions | 686.243us | 20.345ms | 20.310ms | 0.1% | 20.306ms | 20.323ms | 20.305ms | 20.449ms | 0.000us | 9.57 MiB | none | 49.24K/s |
| q37 | date range and group by a URL | 837.161us | 20.495ms | 20.324ms | 1.4% | 20.306ms | 20.597ms | 20.300ms | 20.709ms | 0.000us | 10.03 MiB | none | 49.20K/s |
| q38 | date range and group by a title | 890.024us | 20.491ms | 20.313ms | 0.1% | 20.312ms | 20.338ms | 20.300ms | 20.372ms | 0.000us | 10.09 MiB | none | 49.23K/s |
| q39 | date range, group by and offset | 776.633us | 20.292ms | 20.303ms | 0.4% | 20.300ms | 20.387ms | 20.299ms | 20.549ms | 0.000us | 10.11 MiB | none | 49.25K/s |
| q40 | date range, a case and a wide group by | 958.881us | 20.308ms | 20.338ms | 0.2% | 20.316ms | 20.347ms | 20.306ms | 20.380ms | 0.000us | 10.24 MiB | none | 49.17K/s |
| q41 | date range with an IN and a hash | 712.338us | 20.302ms | 20.323ms | 1.2% | 20.316ms | 20.557ms | 20.312ms | 20.760ms | 0.000us | 9.88 MiB | none | 49.20K/s |
| q42 | date range and a deep offset | 716.313us | 20.320ms | 20.314ms | 0.1% | 20.308ms | 20.319ms | 20.304ms | 20.347ms | 0.000us | 9.73 MiB | none | 49.23K/s |
| q43 | minute buckets over a date range | 722.568us | 20.340ms | 20.315ms | 0.1% | 20.310ms | 20.323ms | 20.299ms | 20.472ms | 0.000us | 9.88 MiB | none | 49.23K/s |

rudb rudb 0.3.45 over 43 of 43 queries. Total 33.153ms by its own clock and 874.295ms by ours, 875.200ms cold, 0.000us of CPU, peak 10.24 MiB, 1.30M/s and 371.63 MiB/s.

Running it cost 2537% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.02x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | 366.942us | 36.904us | 32.919us | 36.886us | 10.8% | 15.796us | 133.845us | 0.000us | 360 B | 4 of 4 |
| q2 | 328.812us | 58.957us | 54.092us | 58.939us | 8.2% | 14.066us | 110.168us | 0.000us | 360 B | 5 of 5 |
| q3 | 351.382us | 56.034us | 50.346us | 56.017us | 10.1% | 13.603us | 79.419us | 0.000us | 992 B | 4 of 4 |
| q4 | 349.065us | 52.375us | 47.630us | 52.371us | 9.1% | 14.429us | 95.246us | 0.000us | 360 B | 4 of 4 |
| q5 | 311.225us | 57.606us | 53.496us | 57.592us | 7.1% | 15.134us | 90.442us | 0.000us | 13.53 KiB | 4 of 4 |
| q6 | 312.236us | 380.061us | 373.398us | 380.053us | 1.8% | 166.257us | 97.801us | 0.000us | 39.97 KiB | 5 of 5 |
| q7 | 329.293us | 57.357us | 52.594us | 57.342us | 8.3% | 11.851us | 114.466us | 0.000us | 616 B | 4 of 4 |
| q8 | 329.308us | 96.101us | 90.523us | 96.089us | 5.8% | 17.040us | 84.007us | 0.000us | 1.17 KiB | 6 of 6 |
| q9 | 324.210us | 132.687us | 109.774us | 132.685us | 17.3% | 16.453us | 73.598us | 0.000us | 47.19 KiB | 5 of 5 |
| q10 | 366.603us | 240.751us | 209.602us | 240.747us | 12.9% | 26.248us | 82.505us | 0.000us | 105.14 KiB | 5 of 5 |
| q11 | 312.211us | 86.159us | 80.961us | 86.140us | 6.0% | 15.376us | 79.745us | 0.000us | 1.84 KiB | 6 of 6 |
| q12 | 330.044us | 88.761us | 82.774us | 88.744us | 6.7% | 16.315us | 85.793us | 0.000us | 4.96 KiB | 6 of 6 |
| q13 | 310.170us | 110.601us | 106.196us | 110.584us | 4.0% | 22.580us | 80.170us | 0.000us | 22.00 KiB | 6 of 6 |
| q14 | 317.698us | 134.428us | 119.673us | 126.350us | 5.3% | 15.466us | 80.122us | 0.000us | 38.19 KiB | 6 of 6 |
| q15 | 317.679us | 128.239us | 122.935us | 128.234us | 4.1% | 15.988us | 83.803us | 0.000us | 30.19 KiB | 6 of 6 |
| q16 | 301.473us | 119.018us | 105.291us | 118.997us | 11.5% | 14.203us | 77.012us | 0.000us | 41.28 KiB | 5 of 5 |
| q17 | 311.064us | 181.017us | 173.423us | 180.994us | 4.2% | 20.599us | 79.093us | 0.000us | 56.78 KiB | 5 of 5 |
| q18 | 327.886us | 118.280us | 111.673us | 118.250us | 5.6% | 12.069us | 76.260us | 0.000us | 2.33 KiB | 5 of 5 |
| q19 | 335.042us | 211.727us | 203.711us | 211.718us | 3.8% | 15.738us | 84.767us | 0.000us | 65.16 KiB | 5 of 5 |
| q20 | 309.830us | 30.244us | 27.374us | 30.217us | 9.4% | 6.302us | 81.700us | 0.000us | 0 B | 4 of 4 |
| q21 | 350.859us | 201.638us | 197.064us | 201.628us | 2.3% | 12.198us | 98.943us | 0.000us | 360 B | 5 of 5 |
| q22 | 369.340us | 262.666us | 257.438us | 262.648us | 2.0% | 14.503us | 119.280us | 0.000us | 512 B | 6 of 6 |
| q23 | 385.190us | 446.662us | 438.941us | 446.639us | 1.7% | 15.774us | 100.313us | 0.000us | 512 B | 6 of 6 |
| q24 | 541.753us | 217.683us | 213.421us | 217.677us | 2.0% | 8.313us | 201.690us | 0.000us | 0 B | 8 of 8 |
| q25 | 295.826us | 98.091us | 93.461us | 98.077us | 4.7% | 7.791us | 80.660us | 0.000us | 4.91 KiB | 6 of 6 |
| q26 | 294.730us | 81.756us | 77.490us | 81.736us | 5.2% | 8.601us | 78.205us | 0.000us | 4.81 KiB | 5 of 5 |
| q27 | 311.911us | 95.531us | 90.852us | 95.517us | 4.9% | 8.807us | 86.308us | 0.000us | 6.21 KiB | 6 of 6 |
| q28 | 391.797us | 240.319us | 236.064us | 240.301us | 1.8% | 14.409us | 121.561us | 0.000us | 23.03 KiB | 7 of 7 |
| q29 | 397.215us | 340.835us | 336.179us | 340.811us | 1.4% | 16.120us | 93.997us | 0.000us | 71.06 KiB | 7 of 7 |
| q30 | 1.590ms | 78.897us | 71.286us | 78.883us | 9.6% | 22.314us | 101.488us | 0.000us | 29.59 KiB | 4 of 4 |
| q31 | 352.087us | 760.270us | 559.404us | 567.938us | 1.5% | 36.758us | 89.558us | 0.000us | 23.41 KiB | 6 of 6 |
| q32 | 377.262us | 697.627us | 545.225us | 554.896us | 1.7% | 28.338us | 113.107us | 0.000us | 24.45 KiB | 6 of 6 |
| q33 | 330.121us | 625.007us | 564.739us | 591.675us | 4.6% | 32.782us | 80.389us | 0.000us | 47.81 KiB | 5 of 5 |
| q34 | 298.467us | 374.716us | 369.606us | 374.699us | 1.4% | 16.309us | 75.896us | 0.000us | 173.91 KiB | 5 of 5 |
| q35 | 326.090us | 394.014us | 387.833us | 394.016us | 1.6% | 14.091us | 76.475us | 0.000us | 174.09 KiB | 5 of 5 |
| q36 | 342.411us | 148.873us | 143.277us | 148.852us | 3.7% | 14.934us | 87.145us | 0.000us | 37.22 KiB | 6 of 6 |
| q37 | 389.704us | 259.275us | 251.367us | 259.274us | 3.0% | 13.812us | 96.905us | 0.000us | 8.44 KiB | 6 of 6 |
| q38 | 366.925us | 315.299us | 307.891us | 315.277us | 2.3% | 13.384us | 98.479us | 0.000us | 3.51 KiB | 6 of 6 |
| q39 | 366.536us | 220.348us | 212.444us | 220.337us | 3.6% | 11.450us | 95.927us | 0.000us | 512 B | 6 of 6 |
| q40 | 422.817us | 350.307us | 342.257us | 350.279us | 2.3% | 14.775us | 114.210us | 0.000us | 10.79 KiB | 6 of 6 |
| q41 | 395.828us | 115.504us | 107.486us | 115.467us | 6.9% | 12.972us | 104.006us | 0.000us | 1.33 KiB | 6 of 6 |
| q42 | 377.900us | 101.426us | 93.411us | 101.413us | 7.9% | 11.986us | 102.948us | 0.000us | 1.33 KiB | 6 of 6 |
| q43 | 391.525us | 100.113us | 94.031us | 100.087us | 6.1% | 12.300us | 106.062us | 0.000us | 2.53 KiB | 6 of 6 |

Read from the breakdown the engine wrote for its cold run. `planning` is everything before the first row moved, which is the parse, the bind, the optimizer passes and building the tree. It is a column rather than a footnote because it is the one cost of a query nobody profiles: an optimizer only ever has passes added to it, each paying for itself on the query it was written for, and a query that plans for four hundred milliseconds to save two hundred is a query the optimizer made slower. What is gated is planning as a share of the whole statement, and that share is not in this table, because it is taken over every run rather than off the cold one the rest of these columns come from. It lives in `baselines/planning-<suite>.txt`. `accounted` is what its pipelines charged themselves and `measured` is what it measured around running them, so `apart` is the cross check and anything over five percent is time the breakdown cannot explain. `driver` is the part of `accounted` that was not inside an operator, which is the loop that runs a pipeline rather than the operators it calls. `build` is what the engine spent putting the tree together, which is before there is a pipeline to charge and is why it is off the right hand side of the check. `outside` is the CPU the process spent on everything else, which is starting, opening the data and printing the answer, and it is the reason the wall clock column and the query time column differ. `held` is what the operators reserved, which is not the resident set of the process. `reference` is how many operators ran the reference implementation of their seam, which is the slow path kept for differential testing.

#### What the planner knew

| query | decisions | exact | certified | estimated | unknown |
| --- | --- | --- | --- | --- | --- |
| q1 | 4 | 4 | 0 | 0 | 0 |
| q2 | 5 | 3 | 0 | 2 | 0 |
| q3 | 4 | 4 | 0 | 0 | 0 |
| q4 | 4 | 4 | 0 | 0 | 0 |
| q5 | 4 | 4 | 0 | 0 | 0 |
| q6 | 5 | 4 | 0 | 1 | 0 |
| q7 | 4 | 4 | 0 | 0 | 0 |
| q8 | 6 | 1 | 0 | 5 | 0 |
| q9 | 5 | 2 | 0 | 3 | 0 |
| q10 | 5 | 2 | 0 | 3 | 0 |
| q11 | 6 | 1 | 0 | 5 | 0 |
| q12 | 6 | 1 | 0 | 5 | 0 |
| q13 | 6 | 1 | 0 | 5 | 0 |
| q14 | 6 | 1 | 0 | 5 | 0 |
| q15 | 6 | 1 | 0 | 5 | 0 |
| q16 | 5 | 2 | 0 | 3 | 0 |
| q17 | 5 | 2 | 0 | 3 | 0 |
| q18 | 5 | 2 | 0 | 3 | 0 |
| q19 | 5 | 2 | 0 | 3 | 0 |
| q20 | 4 | 1 | 0 | 3 | 0 |
| q21 | 5 | 3 | 0 | 2 | 0 |
| q22 | 6 | 1 | 0 | 5 | 0 |
| q23 | 6 | 1 | 0 | 5 | 0 |
| q24 | 8 | 1 | 0 | 7 | 0 |
| q25 | 6 | 1 | 0 | 5 | 0 |
| q26 | 5 | 1 | 0 | 4 | 0 |
| q27 | 6 | 1 | 0 | 5 | 0 |
| q28 | 7 | 1 | 0 | 6 | 0 |
| q29 | 7 | 1 | 0 | 6 | 0 |
| q30 | 4 | 4 | 0 | 0 | 0 |
| q31 | 6 | 1 | 0 | 5 | 0 |
| q32 | 6 | 1 | 0 | 5 | 0 |
| q33 | 5 | 2 | 0 | 3 | 0 |
| q34 | 5 | 2 | 0 | 3 | 0 |
| q35 | 5 | 2 | 0 | 3 | 0 |
| q36 | 6 | 2 | 0 | 4 | 0 |
| q37 | 6 | 1 | 0 | 5 | 0 |
| q38 | 6 | 1 | 0 | 5 | 0 |
| q39 | 6 | 1 | 0 | 5 | 0 |
| q40 | 6 | 1 | 0 | 5 | 0 |
| q41 | 6 | 1 | 0 | 5 | 0 |
| q42 | 6 | 1 | 0 | 5 | 0 |
| q43 | 6 | 1 | 0 | 5 | 0 |
| whole suite | 235 | 78 | 0 | 157 | 0 |

One row per query and one decision per operator, counted by what the planner had behind the cardinality it used. Over the whole suite that is 33% exact, 0% certified, 67% estimated and 0% unknown. `exact` is a counted number, `certified` is a number with a proven bound, `estimated` is a guess, and `unknown` is no number at all, which is an answer an operator has to handle rather than a null to paper over. Counts rather than fractions in the table because a suite's histogram is the sum of its queries' and fractions do not add.

#### Where the time went

| kind | cpu | share | operators | rows in | rows out | per row in | per row out | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| FileScan | 3.383ms | 45.9% | 43 | 0 | 43000 | handed none | 78.7ns | 43 of 43 |
| Aggregate | 3.024ms | 41.0% | 39 | 19785 | 877 | 152.8ns | 3448.1ns | 39 of 39 |
| Filter | 465.102us | 6.3% | 28 | 26000 | 3061 | 17.9ns | 151.9ns | 28 of 28 |
| TopN | 277.362us | 3.8% | 31 | 1132 | 197 | 245.0ns | 1407.9ns | 31 of 31 |
| Project | 217.187us | 2.9% | 91 | 21244 | 21244 | 10.2ns | 10.2ns | 91 of 91 |
| Sort | 4.092us | 0.1% | 1 | 2 | 2 | 2046.0ns | 2046.0ns | 1 of 1 |
| Limit | 0.273us | 0.0% | 1 | 10 | 10 | 27.3ns | 27.3ns | 1 of 1 |
| Fetch | 0.000us | 0.0% | 1 | 0 | 0 | handed none | handed on none | 1 of 1 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is FileScan, and the queries where it cost the most are q23 at 387.382us, q40 at 267.111us, q38 at 251.580us.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 1000 rows, one out of every 99998 of the 99997497 in the full file, which is a development loop rather than the suite
- every query here ran within 1.00x of every other one, so most of what was timed is whatever they have in common rather than the queries

These swung wider than reporting rule two allows:

- clickhouse-local swung by 33.2% of its median on q6, and rule two wants under 10%
- polars swung by 20.1% of its median on q3, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

These ran every query at about the same speed:

- duckdb ran every query within 1.00x of every other one, and this suite spreads over 6x on an engine it is measuring
- duckdb-pinned ran every query within 1.00x of every other one, and this suite spreads over 6x on an engine it is measuring
- clickhouse-local ran every query within 1.33x of every other one, and this suite spreads over 6x on an engine it is measuring
- datafusion ran every query within 1.99x of every other one, and this suite spreads over 6x on an engine it is measuring
- polars ran every query within 1.21x of every other one, and this suite spreads over 6x on an engine it is measuring
- rudb ran every query within 1.02x of every other one, and this suite spreads over 6x on an engine it is measuring

A column that flat is not a column about the queries. Read those numbers as an upper bound on the engine and not as a measurement of it.

polars did not run q28, because the SQLContext has no strlen and no character length function.

polars did not run q29, because the SQLContext has no regexp_replace.

polars did not run q36, because the SQLContext rejects the repeated ClientIP output name in the group by.

polars did not run q43, because the SQLContext has no date_trunc.

So the polars column is 39 of 43 queries and its ratio is over the shared ones.

Answers differ, so this is not a comparison: q13: duckdb-pinned does not agree with duckdb: the same 10 numbers and 9 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q13: clickhouse-local does not agree with duckdb: the same 10 numbers and 8 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q13: datafusion does not agree with duckdb: the same 10 numbers and 9 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q13: polars does not agree with duckdb: the same 10 numbers and 9 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q13: rudb does not agree with duckdb: the same 10 numbers and 8 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q14: duckdb-pinned does not agree with duckdb: the same 10 numbers and 3 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q14: clickhouse-local does not agree with duckdb: the same 10 numbers and 9 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q14: datafusion does not agree with duckdb: the same 10 numbers and 9 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q14: polars does not agree with duckdb: the same 10 numbers and 9 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q14: rudb does not agree with duckdb: the same 10 numbers and 9 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q15: duckdb-pinned does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q15: clickhouse-local does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q15: datafusion does not agree with duckdb: the same 20 numbers and 9 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q15: polars does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q15: rudb does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q34: duckdb-pinned does not agree with duckdb: the same 10 numbers and 1 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q34: clickhouse-local does not agree with duckdb: the same 10 numbers and 1 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q34: polars does not agree with duckdb: the same 10 numbers and 1 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q34: rudb does not agree with duckdb: the same 10 numbers and 1 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

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

Hot here means page cache warm and not buffer pool warm, because every run is a fresh process so that no query's number depends on the one before it.

