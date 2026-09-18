# clickbench on gamingpc-wsl

This is one run of the clickbench suite on gamingpc-wsl, over 6 engines and 43 queries, with 5 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

It ran over 99998 rows, one out of every 1000 of the 99997497 in the full file, which is a development loop and not the suite. Nothing here is comparable to a full run or to anybody else's number.

## What ran

| what | it was |
| --- | --- |
| suite | clickbench |
| queries | 43 |
| tables | 22.85 MiB of Parquet in 1 table |
| rows | 99998 in the table every query reads |
| sample | 99998 rows, one out of every 1000 of the 99997497 in the full file |
| corpus | no manifest beside the data, so this run cannot say where it came from |
| summary | median with the interquartile range, per reporting rule two |
| timeout | 600s per query, and no query reached it |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --rows 100000 --engines duckdb,duckdb-pinned,clickhouse-local,datafusion,polars,rudb --runs 5 --timeout 600 --report
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
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 467.329ms | 800.000ms | 31.01 MiB | its own database file | its own | 2.60 to 4.28 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 560.344ms | 740.000ms | 28.76 MiB | its own database file | its own | 4.28 to 4.00 |
| clickhouse-local | 26.9.1.1562 | ran | 351.043ms | 390.000ms | 24.32 MiB | its own MergeTree parts, as system.parts counts the active ones | its own | 4.00 to 4.14 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 22.85 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 4.14 to 3.97 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 22.85 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 3.97 to 3.52 |
| rudb | rudb 0.3.45 | ran | 0.000us | 0.000us | 22.85 MiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 3.52 to 3.88 |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb on 39 shared |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 362.000ms | 1.221s | +237% | 1.236s | 840.000ms | 0.69 | 65.32 MiB | none | 11.88M/s | 2.65 GiB/s | 1.00x |
| duckdb-pinned | 345.000ms | 1.837s | +432% | 1.838s | 1.210s | 0.66 | 81.97 MiB | none | 12.46M/s | 2.78 GiB/s | 1.01x |
| clickhouse-local | 782.000ms | 3.823s | +389% | 3.844s | 3.450s | 0.90 | 297.00 MiB | none | 5.50M/s | 1.23 GiB/s | 2.68x |
| datafusion | 422.000ms | 1.515s | +259% | 1.476s | 3.950s | 2.61 | 401.69 MiB | none | 10.19M/s | 2.27 GiB/s | 1.42x |
| polars | 690.885ms | 4.694s | +579% | 4.714s | 6.170s | 1.31 | 132.39 MiB | none | 5.64M/s | 1.26 GiB/s | 2.59x |
| rudb | 233.610ms | 878.067ms | +276% | 878.955ms | 560.000ms | 0.64 | 46.48 MiB | none | 18.41M/s | 4.11 GiB/s | 0.78x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 1.000ms | 5.000ms | 1.000ms | 6.228ms | 2.550ms |
| q2 | filtered count | 1.000ms | 1.000ms | 5.000ms | 6.000ms | 7.228ms | 2.575ms |
| q3 | three aggregates | 1.000ms | 1.000ms | 7.000ms | 6.000ms | 7.013ms | 3.275ms |
| q4 | average | 1.000ms | 2.000ms | 7.000ms | 4.000ms | 6.866ms | 2.988ms |
| q5 | count distinct, high card | 5.000ms | 5.000ms | 11.000ms | 9.000ms | 14.262ms | 3.107ms |
| q6 | count distinct, strings | 5.000ms | 4.000ms | 9.000ms | 10.000ms | 14.686ms | 4.964ms |
| q7 | min and max of a date | 1.000ms | 1.000ms | 12.000ms | 1.000ms | 6.984ms | 2.906ms |
| q8 | group by, low card | 1.000ms | 6.000ms | 13.000ms | 7.000ms | 13.848ms | 2.785ms |
| q9 | group by and count distinct | 8.000ms | 7.000ms | 10.000ms | 11.000ms | 24.318ms | 4.322ms |
| q10 | group by, several aggregates | 10.000ms | 9.000ms | 11.000ms | 10.000ms | 27.997ms | 4.865ms |
| q11 | group by a string and count distinct | 5.000ms | 4.000ms | 8.000ms | 10.000ms | 21.230ms | 3.753ms |
| q12 | group by two strings and count distinct | 5.000ms | 5.000ms | 8.000ms | 15.000ms | 22.609ms | 3.506ms |
| q13 | group by a string and top k | 4.000ms | 4.000ms | 10.000ms | 11.000ms | 17.122ms | 5.380ms |
| q14 | group by a string and count distinct | 7.000ms | 7.000ms | 11.000ms | 17.000ms | 23.916ms | 5.662ms |
| q15 | group by two columns and top k | 5.000ms | 5.000ms | 11.000ms | 10.000ms | 18.207ms | 5.665ms |
| q16 | group by, very high card | 6.000ms | 5.000ms | 10.000ms | 10.000ms | 18.935ms | 6.580ms |
| q17 | group by two, very high card | 11.000ms | 10.000ms | 18.000ms | 12.000ms | 23.363ms | 9.293ms |
| q18 | group by two, no ordering | 12.000ms | 12.000ms | 10.000ms | 11.000ms | 15.774ms | 3.632ms |
| q19 | group by with an extract | 12.000ms | 11.000ms | 20.000ms | 12.000ms | 24.675ms | 11.127ms |
| q20 | point lookup | 1.000ms | 1.000ms | 13.000ms | 5.000ms | 6.135ms | 2.043ms |
| q21 | substring scan | 9.000ms | 9.000ms | 14.000ms | 7.000ms | 11.131ms | 5.816ms |
| q22 | substring scan and group by | 10.000ms | 11.000ms | 16.000ms | 10.000ms | 19.259ms | 6.510ms |
| q23 | two substring scans and group by | 12.000ms | 14.000ms | 18.000ms | 16.000ms | 28.815ms | 8.857ms |
| q24 | select star and top k | 25.000ms | 28.000ms | 169.000ms | 20.000ms | 43.003ms | 11.306ms |
| q25 | top k by a date | 3.000ms | 3.000ms | 14.000ms | 8.000ms | 10.354ms | 2.903ms |
| q26 | top k by a string | 2.000ms | 2.000ms | 8.000ms | 8.000ms | 9.909ms | 2.704ms |
| q27 | top k by two columns | 3.000ms | 3.000ms | 13.000ms | 8.000ms | 12.932ms | 2.966ms |
| q28 | group by with a string length | 10.000ms | 11.000ms | 8.000ms | 10.000ms | no dialect | 6.592ms |
| q29 | group by a regular expression | 73.000ms | 56.000ms | 34.000ms | 15.000ms | no dialect | 10.051ms |
| q30 | ninety sums over one column | 7.000ms | 14.000ms | 10.000ms | 13.000ms | 11.678ms | 4.110ms |
| q31 | group by two and several aggregates | 8.000ms | 5.000ms | 10.000ms | 10.000ms | 19.984ms | 4.195ms |
| q32 | group by a high card pair | 6.000ms | 5.000ms | 11.000ms | 13.000ms | 17.361ms | 4.405ms |
| q33 | group by a high card pair, unfiltered | 11.000ms | 8.000ms | 16.000ms | 10.000ms | 19.600ms | 5.293ms |
| q34 | group by a long string | 27.000ms | 18.000ms | 27.000ms | 13.000ms | 23.470ms | 13.931ms |
| q35 | group by a constant and a long string | 22.000ms | 19.000ms | 26.000ms | 15.000ms | 29.118ms | 13.699ms |
| q36 | group by four expressions | 9.000ms | 5.000ms | 11.000ms | 9.000ms | no dialect | 6.357ms |
| q37 | date range and group by a URL | 4.000ms | 4.000ms | 16.000ms | 9.000ms | 20.377ms | 5.294ms |
| q38 | date range and group by a title | 3.000ms | 4.000ms | 16.000ms | 10.000ms | 19.945ms | 5.784ms |
| q39 | date range, group by and offset | 3.000ms | 4.000ms | 48.000ms | 8.000ms | 16.914ms | 5.192ms |
| q40 | date range, a case and a wide group by | 4.000ms | 6.000ms | 47.000ms | 9.000ms | 17.093ms | 6.734ms |
| q41 | date range with an IN and a hash | 3.000ms | 4.000ms | 14.000ms | 7.000ms | 18.712ms | 3.136ms |
| q42 | date range and a deep offset | 3.000ms | 7.000ms | 14.000ms | 7.000ms | 19.834ms | 3.249ms |
| q43 | minute buckets over a date range | 3.000ms | 4.000ms | 13.000ms | 9.000ms | no dialect | 3.547ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 20.257ms | 20.274ms | 0.1% | 20.274ms | 20.295ms | 20.272ms | 20.325ms | 0.000us | 26.93 MiB | none | 4.93M/s |
| q2 | filtered count | 1.000ms | 20.307ms | 20.296ms | 0.1% | 20.287ms | 20.308ms | 20.275ms | 20.313ms | 10.000ms | 28.00 MiB | none | 4.93M/s |
| q3 | three aggregates | 1.000ms | 20.298ms | 20.292ms | 0.1% | 20.282ms | 20.307ms | 20.277ms | 20.316ms | 0.000us | 28.43 MiB | none | 4.93M/s |
| q4 | average | 1.000ms | 20.306ms | 20.282ms | 0.1% | 20.280ms | 20.291ms | 20.276ms | 20.361ms | 10.000ms | 28.50 MiB | none | 4.93M/s |
| q5 | count distinct, high card | 5.000ms | 20.276ms | 20.278ms | 0.0% | 20.278ms | 20.280ms | 20.263ms | 20.299ms | 10.000ms | 35.24 MiB | none | 4.93M/s |
| q6 | count distinct, strings | 5.000ms | 20.295ms | 20.275ms | 0.1% | 20.271ms | 20.287ms | 20.269ms | 20.295ms | 10.000ms | 33.24 MiB | none | 4.93M/s |
| q7 | min and max of a date | 1.000ms | 20.265ms | 20.291ms | 0.0% | 20.286ms | 20.295ms | 20.285ms | 20.313ms | 10.000ms | 27.25 MiB | none | 4.93M/s |
| q8 | group by, low card | 1.000ms | 20.291ms | 20.302ms | 0.1% | 20.285ms | 20.302ms | 20.282ms | 20.320ms | 0.000us | 29.69 MiB | none | 4.93M/s |
| q9 | group by and count distinct | 8.000ms | 20.299ms | 20.311ms | 0.4% | 20.278ms | 20.360ms | 20.260ms | 20.466ms | 20.000ms | 44.78 MiB | none | 4.92M/s |
| q10 | group by, several aggregates | 10.000ms | 40.389ms | 20.292ms | 0.5% | 20.271ms | 20.376ms | 20.268ms | 20.558ms | 30.000ms | 46.99 MiB | none | 4.93M/s |
| q11 | group by a string and count distinct | 5.000ms | 20.270ms | 20.286ms | 0.0% | 20.280ms | 20.288ms | 20.276ms | 20.297ms | 10.000ms | 39.00 MiB | none | 4.93M/s |
| q12 | group by two strings and count distinct | 5.000ms | 20.293ms | 20.295ms | 0.1% | 20.282ms | 20.303ms | 20.279ms | 20.311ms | 20.000ms | 38.93 MiB | none | 4.93M/s |
| q13 | group by a string and top k | 4.000ms | 20.270ms | 20.304ms | 0.1% | 20.290ms | 20.315ms | 20.284ms | 20.338ms | 10.000ms | 35.50 MiB | none | 4.93M/s |
| q14 | group by a string and count distinct | 7.000ms | 20.292ms | 20.285ms | 0.0% | 20.281ms | 20.285ms | 20.278ms | 20.312ms | 20.000ms | 45.88 MiB | none | 4.93M/s |
| q15 | group by two columns and top k | 5.000ms | 20.293ms | 20.285ms | 0.1% | 20.282ms | 20.293ms | 20.279ms | 20.302ms | 10.000ms | 36.54 MiB | none | 4.93M/s |
| q16 | group by, very high card | 6.000ms | 20.288ms | 20.291ms | 0.0% | 20.290ms | 20.294ms | 20.277ms | 20.305ms | 20.000ms | 40.79 MiB | none | 4.93M/s |
| q17 | group by two, very high card | 11.000ms | 40.381ms | 40.475ms | 0.1% | 40.461ms | 40.483ms | 40.385ms | 40.618ms | 30.000ms | 47.37 MiB | none | 2.47M/s |
| q18 | group by two, no ordering | 12.000ms | 40.388ms | 40.420ms | 0.2% | 40.397ms | 40.466ms | 40.388ms | 40.506ms | 30.000ms | 56.58 MiB | none | 2.47M/s |
| q19 | group by with an extract | 12.000ms | 40.395ms | 40.379ms | 0.0% | 40.372ms | 40.383ms | 40.371ms | 40.401ms | 30.000ms | 49.66 MiB | none | 2.48M/s |
| q20 | point lookup | 1.000ms | 20.310ms | 20.290ms | 0.0% | 20.290ms | 20.296ms | 20.287ms | 20.371ms | 10.000ms | 28.25 MiB | none | 4.93M/s |
| q21 | substring scan | 9.000ms | 20.263ms | 20.290ms | 0.1% | 20.277ms | 20.306ms | 20.254ms | 20.314ms | 10.000ms | 34.23 MiB | none | 4.93M/s |
| q22 | substring scan and group by | 10.000ms | 20.417ms | 20.299ms | 0.1% | 20.288ms | 20.308ms | 20.268ms | 40.370ms | 20.000ms | 38.53 MiB | none | 4.93M/s |
| q23 | two substring scans and group by | 12.000ms | 40.415ms | 40.395ms | 0.0% | 40.389ms | 40.402ms | 40.378ms | 40.427ms | 20.000ms | 45.28 MiB | none | 2.48M/s |
| q24 | select star and top k | 25.000ms | 40.402ms | 40.391ms | 0.1% | 40.385ms | 40.421ms | 40.367ms | 40.424ms | 50.000ms | 65.32 MiB | none | 2.48M/s |
| q25 | top k by a date | 3.000ms | 20.287ms | 20.300ms | 0.0% | 20.293ms | 20.303ms | 20.288ms | 20.303ms | 10.000ms | 31.75 MiB | none | 4.93M/s |
| q26 | top k by a string | 2.000ms | 20.301ms | 20.310ms | 0.1% | 20.306ms | 20.319ms | 20.277ms | 20.327ms | 0.000us | 28.58 MiB | none | 4.92M/s |
| q27 | top k by two columns | 3.000ms | 20.283ms | 20.329ms | 0.4% | 20.316ms | 20.390ms | 20.294ms | 20.462ms | 0.000us | 29.75 MiB | none | 4.92M/s |
| q28 | group by with a string length | 10.000ms | 40.462ms | 20.336ms | 0.3% | 20.297ms | 20.353ms | 20.276ms | 20.405ms | 10.000ms | 37.50 MiB | none | 4.92M/s |
| q29 | group by a regular expression | 73.000ms | 80.516ms | 105.657ms | 61.0% | 80.766ms | 145.268ms | 80.706ms | 163.220ms | 130.000ms | 47.93 MiB | none | 946.44K/s |
| q30 | ninety sums over one column | 7.000ms | 40.733ms | 40.691ms | 52.4% | 20.623ms | 41.964ms | 20.554ms | 48.558ms | 10.000ms | 31.74 MiB | none | 2.46M/s |
| q31 | group by two and several aggregates | 8.000ms | 40.716ms | 40.650ms | 0.2% | 40.606ms | 40.689ms | 20.455ms | 40.703ms | 30.000ms | 37.32 MiB | none | 2.46M/s |
| q32 | group by a high card pair | 6.000ms | 40.653ms | 40.526ms | 49.3% | 20.589ms | 40.582ms | 20.575ms | 40.685ms | 20.000ms | 37.59 MiB | none | 2.47M/s |
| q33 | group by a high card pair, unfiltered | 11.000ms | 40.619ms | 40.672ms | 0.3% | 40.570ms | 40.688ms | 40.569ms | 40.709ms | 30.000ms | 51.40 MiB | none | 2.46M/s |
| q34 | group by a long string | 27.000ms | 40.547ms | 60.709ms | 33.0% | 40.733ms | 60.741ms | 40.645ms | 60.763ms | 50.000ms | 59.68 MiB | none | 1.65M/s |
| q35 | group by a constant and a long string | 22.000ms | 40.484ms | 40.524ms | 0.2% | 40.461ms | 40.540ms | 40.453ms | 40.623ms | 60.000ms | 62.75 MiB | none | 2.47M/s |
| q36 | group by four expressions | 9.000ms | 40.452ms | 40.456ms | 0.1% | 40.437ms | 40.472ms | 20.388ms | 40.513ms | 30.000ms | 42.60 MiB | none | 2.47M/s |
| q37 | date range and group by a URL | 4.000ms | 40.404ms | 20.353ms | 0.2% | 20.343ms | 20.374ms | 20.335ms | 20.384ms | 10.000ms | 33.68 MiB | none | 4.91M/s |
| q38 | date range and group by a title | 3.000ms | 20.336ms | 20.366ms | 0.2% | 20.334ms | 20.374ms | 20.320ms | 20.390ms | 10.000ms | 33.00 MiB | none | 4.91M/s |
| q39 | date range, group by and offset | 3.000ms | 20.373ms | 20.368ms | 0.0% | 20.362ms | 20.372ms | 20.346ms | 20.624ms | 10.000ms | 32.50 MiB | none | 4.91M/s |
| q40 | date range, a case and a wide group by | 4.000ms | 20.347ms | 20.319ms | 0.1% | 20.298ms | 20.324ms | 20.285ms | 20.328ms | 10.000ms | 35.50 MiB | none | 4.92M/s |
| q41 | date range with an IN and a hash | 3.000ms | 20.334ms | 20.306ms | 0.0% | 20.304ms | 20.310ms | 20.301ms | 20.312ms | 10.000ms | 34.17 MiB | none | 4.92M/s |
| q42 | date range and a deep offset | 3.000ms | 20.333ms | 20.305ms | 0.1% | 20.299ms | 20.311ms | 20.285ms | 20.323ms | 10.000ms | 32.81 MiB | none | 4.92M/s |
| q43 | minute buckets over a date range | 3.000ms | 20.310ms | 20.303ms | 0.2% | 20.296ms | 20.328ms | 20.283ms | 20.425ms | 10.000ms | 32.74 MiB | none | 4.93M/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 362.000ms by its own clock and 1.221s by ours, 1.236s cold, 840.000ms of CPU, peak 65.32 MiB, 11.88M/s and 2.65 GiB/s.

Running it cost 237% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 5.21x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 40.414ms | 40.394ms | 0.2% | 40.386ms | 40.447ms | 40.384ms | 40.493ms | 20.000ms | 39.82 MiB | none | 2.48M/s |
| q2 | filtered count | 1.000ms | 40.406ms | 40.412ms | 0.0% | 40.405ms | 40.417ms | 40.394ms | 40.443ms | 20.000ms | 40.11 MiB | none | 2.47M/s |
| q3 | three aggregates | 1.000ms | 40.460ms | 40.421ms | 0.1% | 40.398ms | 40.442ms | 40.382ms | 40.474ms | 20.000ms | 40.74 MiB | none | 2.47M/s |
| q4 | average | 2.000ms | 40.432ms | 40.447ms | 0.1% | 40.430ms | 40.471ms | 40.405ms | 40.494ms | 20.000ms | 41.30 MiB | none | 2.47M/s |
| q5 | count distinct, high card | 5.000ms | 40.411ms | 40.411ms | 0.0% | 40.410ms | 40.415ms | 40.397ms | 40.463ms | 20.000ms | 50.09 MiB | none | 2.47M/s |
| q6 | count distinct, strings | 4.000ms | 40.404ms | 40.382ms | 0.0% | 40.381ms | 40.396ms | 40.361ms | 40.416ms | 20.000ms | 44.54 MiB | none | 2.48M/s |
| q7 | min and max of a date | 1.000ms | 40.398ms | 40.414ms | 0.0% | 40.404ms | 40.419ms | 40.402ms | 40.505ms | 20.000ms | 40.54 MiB | none | 2.47M/s |
| q8 | group by, low card | 6.000ms | 40.415ms | 40.411ms | 0.1% | 40.397ms | 40.421ms | 40.385ms | 40.442ms | 30.000ms | 43.09 MiB | none | 2.47M/s |
| q9 | group by and count distinct | 7.000ms | 40.423ms | 40.396ms | 0.0% | 40.396ms | 40.396ms | 40.360ms | 40.399ms | 30.000ms | 59.14 MiB | none | 2.48M/s |
| q10 | group by, several aggregates | 9.000ms | 40.403ms | 40.398ms | 0.0% | 40.393ms | 40.398ms | 40.375ms | 40.466ms | 40.000ms | 60.15 MiB | none | 2.48M/s |
| q11 | group by a string and count distinct | 4.000ms | 40.417ms | 40.428ms | 0.1% | 40.417ms | 40.464ms | 40.415ms | 40.491ms | 20.000ms | 49.40 MiB | none | 2.47M/s |
| q12 | group by two strings and count distinct | 5.000ms | 40.409ms | 40.399ms | 0.0% | 40.399ms | 40.408ms | 40.386ms | 40.410ms | 20.000ms | 49.91 MiB | none | 2.48M/s |
| q13 | group by a string and top k | 4.000ms | 40.449ms | 40.410ms | 0.1% | 40.400ms | 40.433ms | 40.400ms | 40.489ms | 20.000ms | 45.31 MiB | none | 2.47M/s |
| q14 | group by a string and count distinct | 7.000ms | 40.366ms | 40.393ms | 0.1% | 40.384ms | 40.423ms | 40.347ms | 40.467ms | 30.000ms | 54.41 MiB | none | 2.48M/s |
| q15 | group by two columns and top k | 5.000ms | 40.415ms | 40.398ms | 0.1% | 40.389ms | 40.419ms | 40.375ms | 40.445ms | 20.000ms | 46.86 MiB | none | 2.48M/s |
| q16 | group by, very high card | 5.000ms | 40.408ms | 40.393ms | 0.0% | 40.391ms | 40.408ms | 40.389ms | 40.421ms | 30.000ms | 52.60 MiB | none | 2.48M/s |
| q17 | group by two, very high card | 10.000ms | 40.405ms | 40.396ms | 0.1% | 40.374ms | 40.401ms | 40.363ms | 40.449ms | 40.000ms | 62.18 MiB | none | 2.48M/s |
| q18 | group by two, no ordering | 12.000ms | 40.394ms | 40.391ms | 0.0% | 40.382ms | 40.392ms | 40.374ms | 40.445ms | 40.000ms | 69.86 MiB | none | 2.48M/s |
| q19 | group by with an extract | 11.000ms | 40.433ms | 40.371ms | 0.0% | 40.367ms | 40.378ms | 40.366ms | 40.402ms | 40.000ms | 62.70 MiB | none | 2.48M/s |
| q20 | point lookup | 1.000ms | 40.402ms | 40.411ms | 0.2% | 40.388ms | 40.453ms | 40.381ms | 40.592ms | 20.000ms | 40.04 MiB | none | 2.47M/s |
| q21 | substring scan | 9.000ms | 40.380ms | 40.361ms | 0.0% | 40.360ms | 40.366ms | 40.357ms | 40.370ms | 30.000ms | 47.10 MiB | none | 2.48M/s |
| q22 | substring scan and group by | 11.000ms | 40.386ms | 40.365ms | 0.0% | 40.361ms | 40.375ms | 40.355ms | 40.479ms | 20.000ms | 51.30 MiB | none | 2.48M/s |
| q23 | two substring scans and group by | 14.000ms | 40.403ms | 40.350ms | 0.0% | 40.347ms | 40.357ms | 40.342ms | 40.375ms | 40.000ms | 58.20 MiB | none | 2.48M/s |
| q24 | select star and top k | 28.000ms | 60.436ms | 60.457ms | 0.1% | 60.438ms | 60.480ms | 60.437ms | 60.506ms | 50.000ms | 81.97 MiB | none | 1.65M/s |
| q25 | top k by a date | 3.000ms | 40.393ms | 40.363ms | 0.0% | 40.361ms | 40.376ms | 40.352ms | 40.392ms | 20.000ms | 42.54 MiB | none | 2.48M/s |
| q26 | top k by a string | 2.000ms | 40.379ms | 40.365ms | 0.0% | 40.360ms | 40.375ms | 40.348ms | 40.392ms | 10.000ms | 42.03 MiB | none | 2.48M/s |
| q27 | top k by two columns | 3.000ms | 40.376ms | 40.377ms | 0.1% | 40.369ms | 40.390ms | 40.359ms | 40.412ms | 20.000ms | 42.79 MiB | none | 2.48M/s |
| q28 | group by with a string length | 11.000ms | 40.364ms | 40.343ms | 0.0% | 40.340ms | 40.349ms | 40.338ms | 40.371ms | 30.000ms | 50.86 MiB | none | 2.48M/s |
| q29 | group by a regular expression | 56.000ms | 80.547ms | 80.543ms | 24.9% | 80.533ms | 100.561ms | 80.480ms | 100.563ms | 80.000ms | 54.66 MiB | none | 1.24M/s |
| q30 | ninety sums over one column | 14.000ms | 40.364ms | 40.370ms | 0.2% | 40.361ms | 40.429ms | 40.352ms | 40.436ms | 30.000ms | 52.99 MiB | none | 2.48M/s |
| q31 | group by two and several aggregates | 5.000ms | 40.344ms | 40.368ms | 0.0% | 40.364ms | 40.373ms | 40.361ms | 40.402ms | 20.000ms | 49.17 MiB | none | 2.48M/s |
| q32 | group by a high card pair | 5.000ms | 40.373ms | 40.385ms | 0.1% | 40.371ms | 40.391ms | 40.370ms | 40.403ms | 30.000ms | 49.16 MiB | none | 2.48M/s |
| q33 | group by a high card pair, unfiltered | 8.000ms | 40.495ms | 40.406ms | 0.1% | 40.369ms | 40.418ms | 40.368ms | 40.531ms | 30.000ms | 61.90 MiB | none | 2.47M/s |
| q34 | group by a long string | 18.000ms | 60.563ms | 60.455ms | 0.0% | 60.445ms | 60.473ms | 60.439ms | 60.547ms | 40.000ms | 72.61 MiB | none | 1.65M/s |
| q35 | group by a constant and a long string | 19.000ms | 60.526ms | 60.480ms | 0.0% | 60.467ms | 60.491ms | 60.464ms | 60.495ms | 50.000ms | 74.36 MiB | none | 1.65M/s |
| q36 | group by four expressions | 5.000ms | 40.371ms | 40.382ms | 0.0% | 40.380ms | 40.386ms | 40.365ms | 40.409ms | 30.000ms | 51.42 MiB | none | 2.48M/s |
| q37 | date range and group by a URL | 4.000ms | 40.365ms | 40.373ms | 0.0% | 40.364ms | 40.378ms | 40.357ms | 40.381ms | 20.000ms | 46.36 MiB | none | 2.48M/s |
| q38 | date range and group by a title | 4.000ms | 40.359ms | 40.374ms | 0.0% | 40.365ms | 40.374ms | 40.358ms | 40.392ms | 20.000ms | 45.34 MiB | none | 2.48M/s |
| q39 | date range, group by and offset | 4.000ms | 40.362ms | 40.364ms | 0.0% | 40.356ms | 40.364ms | 40.354ms | 40.368ms | 20.000ms | 45.61 MiB | none | 2.48M/s |
| q40 | date range, a case and a wide group by | 6.000ms | 40.387ms | 40.356ms | 0.0% | 40.356ms | 40.370ms | 40.351ms | 40.417ms | 30.000ms | 50.61 MiB | none | 2.48M/s |
| q41 | date range with an IN and a hash | 4.000ms | 40.367ms | 40.363ms | 0.1% | 40.360ms | 40.384ms | 40.349ms | 40.631ms | 20.000ms | 45.67 MiB | none | 2.48M/s |
| q42 | date range and a deep offset | 7.000ms | 40.368ms | 40.364ms | 0.0% | 40.359ms | 40.366ms | 40.352ms | 40.393ms | 30.000ms | 46.12 MiB | none | 2.48M/s |
| q43 | minute buckets over a date range | 4.000ms | 40.355ms | 40.369ms | 0.0% | 40.369ms | 40.371ms | 40.362ms | 40.391ms | 20.000ms | 44.12 MiB | none | 2.48M/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 345.000ms by its own clock and 1.837s by ours, 1.838s cold, 1.210s of CPU, peak 81.97 MiB, 12.46M/s and 2.78 GiB/s.

Running it cost 432% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.00x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 5.000ms | 80.490ms | 80.525ms | 0.0% | 80.505ms | 80.529ms | 80.489ms | 80.552ms | 70.000ms | 238.29 MiB | none | 1.24M/s |
| q2 | filtered count | 5.000ms | 80.556ms | 80.529ms | 0.0% | 80.515ms | 80.530ms | 80.501ms | 80.537ms | 60.000ms | 239.85 MiB | none | 1.24M/s |
| q3 | three aggregates | 7.000ms | 80.503ms | 80.540ms | 0.1% | 80.522ms | 80.569ms | 80.513ms | 80.594ms | 70.000ms | 242.47 MiB | none | 1.24M/s |
| q4 | average | 7.000ms | 80.511ms | 80.521ms | 0.0% | 80.511ms | 80.525ms | 80.500ms | 80.574ms | 70.000ms | 241.82 MiB | none | 1.24M/s |
| q5 | count distinct, high card | 11.000ms | 80.557ms | 80.517ms | 0.0% | 80.513ms | 80.536ms | 80.504ms | 80.624ms | 70.000ms | 247.32 MiB | none | 1.24M/s |
| q6 | count distinct, strings | 9.000ms | 80.518ms | 80.517ms | 0.0% | 80.511ms | 80.520ms | 80.496ms | 80.545ms | 70.000ms | 244.95 MiB | none | 1.24M/s |
| q7 | min and max of a date | 12.000ms | 80.512ms | 80.546ms | 0.1% | 80.503ms | 80.555ms | 80.498ms | 80.576ms | 70.000ms | 241.75 MiB | none | 1.24M/s |
| q8 | group by, low card | 13.000ms | 80.541ms | 80.515ms | 0.1% | 80.496ms | 80.564ms | 80.484ms | 80.660ms | 80.000ms | 244.70 MiB | none | 1.24M/s |
| q9 | group by and count distinct | 10.000ms | 80.568ms | 80.523ms | 0.1% | 80.521ms | 80.565ms | 80.500ms | 80.618ms | 70.000ms | 248.50 MiB | none | 1.24M/s |
| q10 | group by, several aggregates | 11.000ms | 80.705ms | 80.508ms | 0.0% | 80.500ms | 80.520ms | 80.487ms | 80.627ms | 80.000ms | 250.05 MiB | none | 1.24M/s |
| q11 | group by a string and count distinct | 8.000ms | 80.503ms | 80.508ms | 0.0% | 80.503ms | 80.509ms | 80.495ms | 80.509ms | 70.000ms | 246.32 MiB | none | 1.24M/s |
| q12 | group by two strings and count distinct | 8.000ms | 80.530ms | 80.551ms | 0.1% | 80.546ms | 80.624ms | 80.487ms | 81.816ms | 70.000ms | 247.07 MiB | none | 1.24M/s |
| q13 | group by a string and top k | 10.000ms | 80.510ms | 80.518ms | 0.0% | 80.502ms | 80.526ms | 80.500ms | 80.532ms | 70.000ms | 250.25 MiB | none | 1.24M/s |
| q14 | group by a string and count distinct | 11.000ms | 80.601ms | 80.520ms | 0.0% | 80.505ms | 80.540ms | 80.494ms | 80.590ms | 80.000ms | 252.98 MiB | none | 1.24M/s |
| q15 | group by two columns and top k | 11.000ms | 80.609ms | 80.525ms | 0.1% | 80.507ms | 80.575ms | 80.499ms | 80.626ms | 80.000ms | 250.44 MiB | none | 1.24M/s |
| q16 | group by, very high card | 10.000ms | 80.539ms | 80.521ms | 0.0% | 80.518ms | 80.525ms | 80.510ms | 80.598ms | 70.000ms | 249.34 MiB | none | 1.24M/s |
| q17 | group by two, very high card | 18.000ms | 100.605ms | 100.588ms | 20.0% | 80.527ms | 100.595ms | 80.505ms | 100.643ms | 80.000ms | 262.12 MiB | none | 994.14K/s |
| q18 | group by two, no ordering | 10.000ms | 80.539ms | 80.493ms | 0.0% | 80.492ms | 80.507ms | 80.429ms | 80.556ms | 80.000ms | 249.57 MiB | none | 1.24M/s |
| q19 | group by with an extract | 20.000ms | 100.611ms | 100.593ms | 0.0% | 100.576ms | 100.595ms | 80.701ms | 100.623ms | 90.000ms | 262.65 MiB | none | 994.08K/s |
| q20 | point lookup | 13.000ms | 80.542ms | 80.528ms | 0.0% | 80.521ms | 80.547ms | 80.471ms | 80.579ms | 70.000ms | 241.69 MiB | none | 1.24M/s |
| q21 | substring scan | 14.000ms | 80.523ms | 80.513ms | 0.0% | 80.509ms | 80.517ms | 80.504ms | 80.593ms | 80.000ms | 247.62 MiB | none | 1.24M/s |
| q22 | substring scan and group by | 16.000ms | 80.527ms | 80.551ms | 24.9% | 80.527ms | 100.590ms | 80.485ms | 100.593ms | 80.000ms | 252.17 MiB | none | 1.24M/s |
| q23 | two substring scans and group by | 18.000ms | 120.696ms | 100.597ms | 0.1% | 100.597ms | 100.725ms | 100.587ms | 100.729ms | 90.000ms | 252.42 MiB | none | 994.04K/s |
| q24 | select star and top k | 169.000ms | 241.131ms | 241.139ms | 0.1% | 241.102ms | 241.235ms | 241.094ms | 281.352ms | 240.000ms | 297.00 MiB | none | 414.69K/s |
| q25 | top k by a date | 14.000ms | 80.478ms | 80.478ms | 0.1% | 80.472ms | 80.536ms | 80.467ms | 80.633ms | 80.000ms | 247.25 MiB | none | 1.24M/s |
| q26 | top k by a string | 8.000ms | 80.528ms | 80.515ms | 0.1% | 80.486ms | 80.532ms | 80.482ms | 80.581ms | 70.000ms | 244.86 MiB | none | 1.24M/s |
| q27 | top k by two columns | 13.000ms | 80.504ms | 80.477ms | 0.0% | 80.474ms | 80.481ms | 80.471ms | 80.500ms | 70.000ms | 246.85 MiB | none | 1.24M/s |
| q28 | group by with a string length | 8.000ms | 80.623ms | 80.494ms | 0.0% | 80.479ms | 80.494ms | 80.462ms | 80.530ms | 70.000ms | 247.42 MiB | none | 1.24M/s |
| q29 | group by a regular expression | 34.000ms | 100.549ms | 100.577ms | 0.0% | 100.566ms | 100.613ms | 100.565ms | 100.678ms | 90.000ms | 277.17 MiB | none | 994.25K/s |
| q30 | ninety sums over one column | 10.000ms | 80.496ms | 80.510ms | 0.1% | 80.509ms | 80.554ms | 80.484ms | 80.562ms | 70.000ms | 244.96 MiB | none | 1.24M/s |
| q31 | group by two and several aggregates | 10.000ms | 80.504ms | 80.492ms | 0.1% | 80.472ms | 80.524ms | 80.468ms | 80.706ms | 70.000ms | 249.38 MiB | none | 1.24M/s |
| q32 | group by a high card pair | 11.000ms | 80.519ms | 80.497ms | 0.0% | 80.497ms | 80.502ms | 80.496ms | 80.525ms | 70.000ms | 250.00 MiB | none | 1.24M/s |
| q33 | group by a high card pair, unfiltered | 16.000ms | 80.486ms | 80.510ms | 0.1% | 80.485ms | 80.529ms | 80.476ms | 80.563ms | 80.000ms | 259.58 MiB | none | 1.24M/s |
| q34 | group by a long string | 27.000ms | 100.596ms | 100.587ms | 0.1% | 100.570ms | 100.664ms | 100.568ms | 101.190ms | 90.000ms | 277.57 MiB | none | 994.14K/s |
| q35 | group by a constant and a long string | 26.000ms | 100.595ms | 100.578ms | 0.0% | 100.576ms | 100.584ms | 100.560ms | 100.609ms | 90.000ms | 277.19 MiB | none | 994.24K/s |
| q36 | group by four expressions | 11.000ms | 80.525ms | 80.489ms | 0.0% | 80.487ms | 80.491ms | 80.485ms | 80.498ms | 70.000ms | 249.58 MiB | none | 1.24M/s |
| q37 | date range and group by a URL | 16.000ms | 80.635ms | 80.508ms | 0.0% | 80.502ms | 80.536ms | 80.488ms | 80.641ms | 80.000ms | 251.30 MiB | none | 1.24M/s |
| q38 | date range and group by a title | 16.000ms | 80.512ms | 80.503ms | 0.0% | 80.488ms | 80.506ms | 80.488ms | 80.584ms | 80.000ms | 250.80 MiB | none | 1.24M/s |
| q39 | date range, group by and offset | 48.000ms | 120.615ms | 120.593ms | 16.6% | 100.626ms | 120.653ms | 100.561ms | 140.686ms | 90.000ms | 251.60 MiB | none | 829.22K/s |
| q40 | date range, a case and a wide group by | 47.000ms | 120.652ms | 120.612ms | 0.0% | 120.612ms | 120.625ms | 100.698ms | 120.632ms | 90.000ms | 255.58 MiB | none | 829.09K/s |
| q41 | date range with an IN and a hash | 14.000ms | 80.477ms | 80.523ms | 0.1% | 80.507ms | 80.602ms | 80.476ms | 80.635ms | 80.000ms | 249.36 MiB | none | 1.24M/s |
| q42 | date range and a deep offset | 14.000ms | 80.487ms | 80.583ms | 0.2% | 80.473ms | 80.661ms | 80.464ms | 120.626ms | 80.000ms | 249.13 MiB | none | 1.24M/s |
| q43 | minute buckets over a date range | 13.000ms | 80.741ms | 80.492ms | 0.0% | 80.485ms | 80.496ms | 80.479ms | 80.500ms | 70.000ms | 247.78 MiB | none | 1.24M/s |

clickhouse-local 26.9.1.1562 over 43 of 43 queries. Total 782.000ms by its own clock and 3.823s by ours, 3.844s cold, 3.450s of CPU, peak 297.00 MiB, 5.50M/s and 1.23 GiB/s.

Running it cost 389% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.00x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 20.249ms | 20.259ms | 0.1% | 20.247ms | 20.262ms | 20.244ms | 20.272ms | 0.000us | 82.53 MiB | none | 4.94M/s |
| q2 | filtered count | 6.000ms | 20.254ms | 20.271ms | 98.9% | 20.264ms | 40.321ms | 20.256ms | 40.358ms | 50.000ms | 142.35 MiB | none | 4.93M/s |
| q3 | three aggregates | 6.000ms | 20.255ms | 20.264ms | 0.1% | 20.250ms | 20.271ms | 20.247ms | 20.273ms | 50.000ms | 158.32 MiB | none | 4.93M/s |
| q4 | average | 4.000ms | 20.256ms | 20.270ms | 0.0% | 20.266ms | 20.272ms | 20.265ms | 20.279ms | 30.000ms | 135.92 MiB | none | 4.93M/s |
| q5 | count distinct, high card | 9.000ms | 40.377ms | 40.357ms | 0.0% | 40.354ms | 40.362ms | 40.351ms | 40.363ms | 70.000ms | 236.36 MiB | none | 2.48M/s |
| q6 | count distinct, strings | 10.000ms | 40.365ms | 40.353ms | 0.0% | 40.350ms | 40.365ms | 40.349ms | 40.369ms | 70.000ms | 251.22 MiB | none | 2.48M/s |
| q7 | min and max of a date | 1.000ms | 20.283ms | 20.255ms | 0.0% | 20.250ms | 20.257ms | 20.241ms | 20.260ms | 0.000us | 82.48 MiB | none | 4.94M/s |
| q8 | group by, low card | 7.000ms | 20.268ms | 20.258ms | 0.1% | 20.252ms | 20.263ms | 20.241ms | 20.268ms | 40.000ms | 139.65 MiB | none | 4.94M/s |
| q9 | group by and count distinct | 11.000ms | 40.396ms | 40.387ms | 0.1% | 40.352ms | 40.393ms | 40.351ms | 40.396ms | 110.000ms | 277.13 MiB | none | 2.48M/s |
| q10 | group by, several aggregates | 10.000ms | 40.353ms | 40.391ms | 0.1% | 40.384ms | 40.422ms | 40.361ms | 40.437ms | 80.000ms | 246.73 MiB | none | 2.48M/s |
| q11 | group by a string and count distinct | 10.000ms | 40.416ms | 40.379ms | 0.0% | 40.376ms | 40.384ms | 40.368ms | 40.445ms | 70.000ms | 209.78 MiB | none | 2.48M/s |
| q12 | group by two strings and count distinct | 15.000ms | 40.388ms | 40.393ms | 0.2% | 40.387ms | 40.459ms | 40.385ms | 40.591ms | 250.000ms | 222.90 MiB | none | 2.48M/s |
| q13 | group by a string and top k | 11.000ms | 40.444ms | 40.520ms | 0.5% | 40.455ms | 40.653ms | 40.355ms | 40.683ms | 90.000ms | 256.21 MiB | none | 2.47M/s |
| q14 | group by a string and count distinct | 17.000ms | 40.410ms | 40.447ms | 0.1% | 40.417ms | 40.469ms | 40.357ms | 40.688ms | 250.000ms | 286.66 MiB | none | 2.47M/s |
| q15 | group by two columns and top k | 10.000ms | 40.369ms | 40.377ms | 0.3% | 40.367ms | 40.470ms | 40.357ms | 40.512ms | 80.000ms | 252.88 MiB | none | 2.48M/s |
| q16 | group by, very high card | 10.000ms | 40.459ms | 40.346ms | 0.2% | 40.344ms | 40.411ms | 40.343ms | 40.463ms | 110.000ms | 251.39 MiB | none | 2.48M/s |
| q17 | group by two, very high card | 12.000ms | 40.375ms | 40.398ms | 0.1% | 40.396ms | 40.419ms | 40.377ms | 40.436ms | 110.000ms | 310.57 MiB | none | 2.48M/s |
| q18 | group by two, no ordering | 11.000ms | 40.454ms | 40.351ms | 0.0% | 40.348ms | 40.367ms | 40.347ms | 40.428ms | 90.000ms | 317.17 MiB | none | 2.48M/s |
| q19 | group by with an extract | 12.000ms | 40.356ms | 40.391ms | 0.1% | 40.374ms | 40.414ms | 40.358ms | 40.502ms | 110.000ms | 316.55 MiB | none | 2.48M/s |
| q20 | point lookup | 5.000ms | 20.264ms | 20.273ms | 0.1% | 20.261ms | 20.275ms | 20.252ms | 20.292ms | 60.000ms | 133.91 MiB | none | 4.93M/s |
| q21 | substring scan | 7.000ms | 20.261ms | 20.262ms | 0.1% | 20.257ms | 20.268ms | 20.257ms | 20.270ms | 50.000ms | 179.27 MiB | none | 4.94M/s |
| q22 | substring scan and group by | 10.000ms | 40.360ms | 40.386ms | 0.1% | 40.356ms | 40.406ms | 40.353ms | 40.451ms | 100.000ms | 222.13 MiB | none | 2.48M/s |
| q23 | two substring scans and group by | 16.000ms | 40.383ms | 40.447ms | 0.1% | 40.419ms | 40.449ms | 40.357ms | 40.462ms | 250.000ms | 248.71 MiB | none | 2.47M/s |
| q24 | select star and top k | 20.000ms | 41.127ms | 40.402ms | 0.0% | 40.400ms | 40.411ms | 40.370ms | 40.415ms | 170.000ms | 401.69 MiB | none | 2.48M/s |
| q25 | top k by a date | 8.000ms | 20.272ms | 40.336ms | 49.8% | 20.278ms | 40.368ms | 20.254ms | 40.389ms | 100.000ms | 194.54 MiB | none | 2.48M/s |
| q26 | top k by a string | 8.000ms | 40.359ms | 40.330ms | 49.7% | 20.317ms | 40.348ms | 20.287ms | 40.354ms | 60.000ms | 195.38 MiB | none | 2.48M/s |
| q27 | top k by two columns | 8.000ms | 40.365ms | 40.414ms | 49.7% | 20.357ms | 40.427ms | 20.281ms | 40.445ms | 80.000ms | 198.30 MiB | none | 2.47M/s |
| q28 | group by with a string length | 10.000ms | 40.351ms | 40.365ms | 0.0% | 40.356ms | 40.367ms | 40.353ms | 40.409ms | 100.000ms | 254.08 MiB | none | 2.48M/s |
| q29 | group by a regular expression | 15.000ms | 40.777ms | 40.433ms | 0.2% | 40.398ms | 40.459ms | 40.372ms | 40.479ms | 180.000ms | 361.20 MiB | none | 2.47M/s |
| q30 | ninety sums over one column | 13.000ms | 40.366ms | 40.373ms | 0.1% | 40.364ms | 40.384ms | 40.360ms | 40.455ms | 40.000ms | 139.13 MiB | none | 2.48M/s |
| q31 | group by two and several aggregates | 10.000ms | 40.397ms | 40.373ms | 0.0% | 40.365ms | 40.374ms | 40.351ms | 40.386ms | 70.000ms | 213.16 MiB | none | 2.48M/s |
| q32 | group by a high card pair | 13.000ms | 40.429ms | 40.372ms | 0.0% | 40.360ms | 40.377ms | 40.353ms | 40.485ms | 200.000ms | 225.20 MiB | none | 2.48M/s |
| q33 | group by a high card pair, unfiltered | 10.000ms | 40.356ms | 40.375ms | 0.2% | 40.361ms | 40.446ms | 40.357ms | 40.460ms | 90.000ms | 262.44 MiB | none | 2.48M/s |
| q34 | group by a long string | 13.000ms | 40.363ms | 40.381ms | 0.1% | 40.350ms | 40.401ms | 40.348ms | 40.427ms | 120.000ms | 367.15 MiB | none | 2.48M/s |
| q35 | group by a constant and a long string | 15.000ms | 40.431ms | 40.422ms | 0.1% | 40.406ms | 40.446ms | 40.401ms | 40.474ms | 200.000ms | 369.59 MiB | none | 2.47M/s |
| q36 | group by four expressions | 9.000ms | 40.401ms | 40.354ms | 0.0% | 40.353ms | 40.358ms | 40.341ms | 40.369ms | 70.000ms | 251.39 MiB | none | 2.48M/s |
| q37 | date range and group by a URL | 9.000ms | 20.261ms | 40.337ms | 49.6% | 20.333ms | 40.339ms | 20.274ms | 40.347ms | 50.000ms | 166.77 MiB | none | 2.48M/s |
| q38 | date range and group by a title | 10.000ms | 40.373ms | 40.357ms | 0.0% | 40.352ms | 40.361ms | 40.340ms | 40.447ms | 70.000ms | 175.27 MiB | none | 2.48M/s |
| q39 | date range, group by and offset | 8.000ms | 40.360ms | 20.261ms | 0.0% | 20.261ms | 20.264ms | 20.250ms | 40.432ms | 30.000ms | 148.56 MiB | none | 4.94M/s |
| q40 | date range, a case and a wide group by | 9.000ms | 40.337ms | 40.355ms | 0.0% | 40.348ms | 40.359ms | 40.346ms | 40.427ms | 40.000ms | 153.91 MiB | none | 2.48M/s |
| q41 | date range with an IN and a hash | 7.000ms | 20.269ms | 20.260ms | 0.0% | 20.259ms | 20.261ms | 20.257ms | 20.265ms | 30.000ms | 137.54 MiB | none | 4.94M/s |
| q42 | date range and a deep offset | 7.000ms | 20.333ms | 20.266ms | 0.0% | 20.264ms | 20.268ms | 20.261ms | 20.279ms | 30.000ms | 137.38 MiB | none | 4.93M/s |
| q43 | minute buckets over a date range | 9.000ms | 20.267ms | 40.334ms | 49.8% | 20.272ms | 40.353ms | 20.265ms | 40.358ms | 100.000ms | 149.33 MiB | none | 2.48M/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 422.000ms by its own clock and 1.515s by ours, 1.476s cold, 3.950s of CPU, peak 401.69 MiB, 10.19M/s and 2.27 GiB/s.

Running it cost 259% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.00x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 6.228ms | 100.551ms | 100.564ms | 0.1% | 100.551ms | 100.625ms | 100.542ms | 100.851ms | 120.000ms | 63.05 MiB | none | 994.38K/s |
| q2 | filtered count | 7.228ms | 100.577ms | 100.550ms | 0.0% | 100.539ms | 100.571ms | 100.458ms | 120.620ms | 130.000ms | 67.48 MiB | none | 994.51K/s |
| q3 | three aggregates | 7.013ms | 100.568ms | 100.578ms | 0.0% | 100.553ms | 100.595ms | 100.547ms | 100.606ms | 110.000ms | 67.47 MiB | none | 994.23K/s |
| q4 | average | 6.866ms | 100.579ms | 100.568ms | 0.0% | 100.564ms | 100.571ms | 100.519ms | 100.575ms | 100.000ms | 66.32 MiB | none | 994.33K/s |
| q5 | count distinct, high card | 14.262ms | 120.643ms | 120.779ms | 0.2% | 120.659ms | 120.960ms | 120.648ms | 121.079ms | 140.000ms | 80.70 MiB | none | 827.94K/s |
| q6 | count distinct, strings | 14.686ms | 120.634ms | 120.771ms | 1.4% | 120.681ms | 122.418ms | 120.636ms | 128.344ms | 160.000ms | 82.71 MiB | none | 828.00K/s |
| q7 | min and max of a date | 6.984ms | 100.598ms | 100.568ms | 0.0% | 100.567ms | 100.572ms | 100.557ms | 100.585ms | 110.000ms | 65.09 MiB | none | 994.33K/s |
| q8 | group by, low card | 13.848ms | 121.039ms | 120.689ms | 0.3% | 120.665ms | 121.066ms | 120.645ms | 123.650ms | 150.000ms | 75.80 MiB | none | 828.56K/s |
| q9 | group by and count distinct | 24.318ms | 120.633ms | 120.718ms | 0.1% | 120.636ms | 120.732ms | 120.634ms | 120.933ms | 180.000ms | 99.57 MiB | none | 828.36K/s |
| q10 | group by, several aggregates | 27.997ms | 120.671ms | 140.796ms | 0.1% | 140.782ms | 140.869ms | 140.741ms | 141.083ms | 180.000ms | 103.82 MiB | none | 710.23K/s |
| q11 | group by a string and count distinct | 21.230ms | 120.684ms | 120.686ms | 0.1% | 120.661ms | 120.794ms | 120.636ms | 120.882ms | 160.000ms | 84.94 MiB | none | 828.58K/s |
| q12 | group by two strings and count distinct | 22.609ms | 120.652ms | 120.654ms | 0.0% | 120.651ms | 120.658ms | 120.624ms | 140.655ms | 170.000ms | 85.96 MiB | none | 828.80K/s |
| q13 | group by a string and top k | 17.122ms | 121.351ms | 120.657ms | 0.0% | 120.647ms | 120.659ms | 120.633ms | 120.729ms | 150.000ms | 83.60 MiB | none | 828.78K/s |
| q14 | group by a string and count distinct | 23.916ms | 120.769ms | 120.676ms | 0.2% | 120.666ms | 120.879ms | 120.643ms | 140.724ms | 170.000ms | 97.20 MiB | none | 828.65K/s |
| q15 | group by two columns and top k | 18.207ms | 120.659ms | 121.311ms | 0.5% | 121.078ms | 121.658ms | 120.904ms | 121.676ms | 150.000ms | 85.72 MiB | none | 824.31K/s |
| q16 | group by, very high card | 18.935ms | 121.273ms | 120.675ms | 0.0% | 120.644ms | 120.676ms | 120.597ms | 120.718ms | 140.000ms | 85.51 MiB | none | 828.66K/s |
| q17 | group by two, very high card | 23.363ms | 120.668ms | 120.640ms | 0.1% | 120.638ms | 120.702ms | 120.632ms | 120.739ms | 170.000ms | 101.40 MiB | none | 828.89K/s |
| q18 | group by two, no ordering | 15.774ms | 120.670ms | 120.668ms | 0.1% | 120.636ms | 120.757ms | 120.630ms | 121.302ms | 150.000ms | 99.27 MiB | none | 828.71K/s |
| q19 | group by with an extract | 24.675ms | 121.394ms | 120.694ms | 0.1% | 120.632ms | 120.739ms | 120.626ms | 140.827ms | 180.000ms | 104.88 MiB | none | 828.52K/s |
| q20 | point lookup | 6.135ms | 100.576ms | 100.615ms | 20.3% | 100.606ms | 121.028ms | 100.586ms | 121.320ms | 170.000ms | 66.06 MiB | none | 993.87K/s |
| q21 | substring scan | 11.131ms | 100.570ms | 103.224ms | 19.2% | 101.363ms | 121.162ms | 100.616ms | 121.739ms | 120.000ms | 83.27 MiB | none | 968.75K/s |
| q22 | substring scan and group by | 19.259ms | 122.101ms | 120.662ms | 0.0% | 120.651ms | 120.675ms | 120.639ms | 121.141ms | 160.000ms | 93.10 MiB | none | 828.75K/s |
| q23 | two substring scans and group by | 28.815ms | 141.097ms | 140.912ms | 0.2% | 140.713ms | 140.939ms | 120.693ms | 140.964ms | 210.000ms | 124.60 MiB | none | 709.65K/s |
| q24 | select star and top k | 43.003ms | 141.329ms | 140.753ms | 0.0% | 140.751ms | 140.774ms | 140.707ms | 140.813ms | 340.000ms | 120.24 MiB | none | 710.45K/s |
| q25 | top k by a date | 10.354ms | 121.617ms | 121.354ms | 2.0% | 120.952ms | 123.401ms | 120.660ms | 124.424ms | 160.000ms | 75.79 MiB | none | 824.02K/s |
| q26 | top k by a string | 9.909ms | 120.698ms | 100.575ms | 0.0% | 100.572ms | 100.608ms | 100.554ms | 120.708ms | 110.000ms | 74.18 MiB | none | 994.26K/s |
| q27 | top k by two columns | 12.932ms | 120.639ms | 100.653ms | 20.2% | 100.643ms | 120.979ms | 100.628ms | 126.657ms | 120.000ms | 75.93 MiB | none | 993.49K/s |
| q30 | ninety sums over one column | 11.678ms | 101.345ms | 100.810ms | 1.3% | 100.647ms | 101.942ms | 100.576ms | 124.887ms | 110.000ms | 69.54 MiB | none | 991.94K/s |
| q31 | group by two and several aggregates | 19.984ms | 120.657ms | 120.660ms | 0.0% | 120.655ms | 120.672ms | 120.617ms | 120.689ms | 150.000ms | 87.06 MiB | none | 828.76K/s |
| q32 | group by a high card pair | 17.361ms | 120.652ms | 120.662ms | 0.0% | 120.643ms | 120.688ms | 120.641ms | 120.811ms | 140.000ms | 87.58 MiB | none | 828.75K/s |
| q33 | group by a high card pair, unfiltered | 19.600ms | 121.183ms | 120.765ms | 0.2% | 120.692ms | 120.879ms | 120.665ms | 120.904ms | 170.000ms | 106.39 MiB | none | 828.04K/s |
| q34 | group by a long string | 23.470ms | 120.652ms | 141.050ms | 0.1% | 140.886ms | 141.087ms | 121.165ms | 141.215ms | 210.000ms | 124.95 MiB | none | 708.95K/s |
| q35 | group by a constant and a long string | 29.118ms | 161.305ms | 161.060ms | 0.0% | 161.035ms | 161.073ms | 160.975ms | 161.337ms | 240.000ms | 132.39 MiB | none | 620.87K/s |
| q37 | date range and group by a URL | 20.377ms | 141.004ms | 121.095ms | 16.5% | 120.943ms | 140.932ms | 120.686ms | 141.004ms | 150.000ms | 86.26 MiB | none | 825.78K/s |
| q38 | date range and group by a title | 19.945ms | 140.803ms | 120.700ms | 0.0% | 120.694ms | 120.708ms | 120.671ms | 120.870ms | 150.000ms | 84.54 MiB | none | 828.49K/s |
| q39 | date range, group by and offset | 16.914ms | 120.680ms | 120.831ms | 0.1% | 120.734ms | 120.854ms | 120.691ms | 120.857ms | 140.000ms | 82.43 MiB | none | 827.59K/s |
| q40 | date range, a case and a wide group by | 17.093ms | 120.785ms | 142.691ms | 13.9% | 141.132ms | 161.003ms | 141.055ms | 161.106ms | 180.000ms | 85.55 MiB | none | 700.80K/s |
| q41 | date range with an IN and a hash | 18.712ms | 120.952ms | 140.903ms | 14.3% | 120.823ms | 140.974ms | 120.742ms | 161.166ms | 160.000ms | 81.63 MiB | none | 709.69K/s |
| q42 | date range and a deep offset | 19.834ms | 161.113ms | 140.810ms | 14.3% | 120.864ms | 140.943ms | 120.691ms | 140.958ms | 160.000ms | 81.39 MiB | none | 710.16K/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 690.885ms by its own clock and 4.694s by ours, 4.714s cold, 6.170s of CPU, peak 132.39 MiB, 5.64M/s and 1.26 GiB/s.

Running it cost 579% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.60x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 2.550ms | 20.459ms | 20.366ms | 0.5% | 20.352ms | 20.453ms | 20.338ms | 20.476ms | 0.000us | 9.68 MiB | none | 4.91M/s |
| q2 | filtered count | 2.575ms | 20.504ms | 20.386ms | 0.5% | 20.357ms | 20.451ms | 20.323ms | 20.581ms | 0.000us | 10.30 MiB | none | 4.91M/s |
| q3 | three aggregates | 3.275ms | 20.398ms | 20.455ms | 0.1% | 20.443ms | 20.467ms | 20.356ms | 21.350ms | 0.000us | 9.89 MiB | none | 4.89M/s |
| q4 | average | 2.988ms | 20.388ms | 20.462ms | 0.6% | 20.397ms | 20.525ms | 20.361ms | 20.528ms | 0.000us | 10.75 MiB | none | 4.89M/s |
| q5 | count distinct, high card | 3.107ms | 20.487ms | 20.488ms | 2.3% | 20.435ms | 20.897ms | 20.367ms | 23.972ms | 0.000us | 12.19 MiB | none | 4.88M/s |
| q6 | count distinct, strings | 4.964ms | 20.472ms | 20.388ms | 0.6% | 20.369ms | 20.487ms | 20.355ms | 20.520ms | 10.000ms | 14.18 MiB | none | 4.90M/s |
| q7 | min and max of a date | 2.906ms | 20.533ms | 20.443ms | 0.5% | 20.417ms | 20.521ms | 20.403ms | 20.616ms | 0.000us | 9.96 MiB | none | 4.89M/s |
| q8 | group by, low card | 2.785ms | 20.469ms | 20.469ms | 0.1% | 20.447ms | 20.470ms | 20.401ms | 20.481ms | 0.000us | 10.34 MiB | none | 4.89M/s |
| q9 | group by and count distinct | 4.322ms | 20.492ms | 20.442ms | 0.5% | 20.369ms | 20.469ms | 20.349ms | 20.572ms | 10.000ms | 15.14 MiB | none | 4.89M/s |
| q10 | group by, several aggregates | 4.865ms | 20.473ms | 20.433ms | 0.4% | 20.379ms | 20.460ms | 20.371ms | 20.467ms | 10.000ms | 18.43 MiB | none | 4.89M/s |
| q11 | group by a string and count distinct | 3.753ms | 20.533ms | 20.436ms | 0.2% | 20.428ms | 20.465ms | 20.418ms | 20.512ms | 0.000us | 11.17 MiB | none | 4.89M/s |
| q12 | group by two strings and count distinct | 3.506ms | 20.365ms | 20.467ms | 0.4% | 20.385ms | 20.469ms | 20.355ms | 20.482ms | 0.000us | 12.15 MiB | none | 4.89M/s |
| q13 | group by a string and top k | 5.380ms | 20.365ms | 20.436ms | 0.1% | 20.412ms | 20.442ms | 20.381ms | 20.491ms | 0.000us | 14.25 MiB | none | 4.89M/s |
| q14 | group by a string and count distinct | 5.662ms | 20.536ms | 20.446ms | 0.6% | 20.400ms | 20.529ms | 20.356ms | 20.562ms | 0.000us | 17.63 MiB | none | 4.89M/s |
| q15 | group by two columns and top k | 5.665ms | 20.433ms | 20.378ms | 0.2% | 20.377ms | 20.419ms | 20.362ms | 20.481ms | 10.000ms | 15.88 MiB | none | 4.91M/s |
| q16 | group by, very high card | 6.580ms | 20.457ms | 20.523ms | 1.3% | 20.326ms | 20.587ms | 20.317ms | 20.598ms | 20.000ms | 22.59 MiB | none | 4.87M/s |
| q17 | group by two, very high card | 9.293ms | 20.413ms | 20.448ms | 0.0% | 20.440ms | 20.449ms | 20.430ms | 20.459ms | 30.000ms | 22.73 MiB | none | 4.89M/s |
| q18 | group by two, no ordering | 3.632ms | 20.449ms | 20.360ms | 0.2% | 20.345ms | 20.386ms | 20.336ms | 20.399ms | 10.000ms | 12.22 MiB | none | 4.91M/s |
| q19 | group by with an extract | 11.127ms | 20.454ms | 20.433ms | 0.1% | 20.420ms | 20.433ms | 20.359ms | 20.467ms | 40.000ms | 26.54 MiB | none | 4.89M/s |
| q20 | point lookup | 2.043ms | 20.433ms | 20.360ms | 0.3% | 20.341ms | 20.396ms | 20.334ms | 20.415ms | 0.000us | 9.88 MiB | none | 4.91M/s |
| q21 | substring scan | 5.816ms | 20.318ms | 20.362ms | 0.1% | 20.333ms | 20.362ms | 20.311ms | 20.371ms | 10.000ms | 23.78 MiB | none | 4.91M/s |
| q22 | substring scan and group by | 6.510ms | 20.374ms | 20.385ms | 0.4% | 20.347ms | 20.433ms | 20.315ms | 20.473ms | 30.000ms | 26.25 MiB | none | 4.91M/s |
| q23 | two substring scans and group by | 8.857ms | 20.393ms | 20.374ms | 0.1% | 20.364ms | 20.377ms | 20.317ms | 20.392ms | 50.000ms | 37.80 MiB | none | 4.91M/s |
| q24 | select star and top k | 11.306ms | 20.342ms | 20.369ms | 0.2% | 20.343ms | 20.380ms | 20.318ms | 20.392ms | 60.000ms | 43.39 MiB | none | 4.91M/s |
| q25 | top k by a date | 2.903ms | 20.358ms | 20.365ms | 0.2% | 20.364ms | 20.405ms | 20.345ms | 20.663ms | 0.000us | 12.86 MiB | none | 4.91M/s |
| q26 | top k by a string | 2.704ms | 20.369ms | 20.330ms | 0.2% | 20.325ms | 20.371ms | 20.317ms | 20.405ms | 10.000ms | 10.27 MiB | none | 4.92M/s |
| q27 | top k by two columns | 2.966ms | 20.355ms | 20.398ms | 0.3% | 20.351ms | 20.412ms | 20.254ms | 21.165ms | 10.000ms | 13.09 MiB | none | 4.90M/s |
| q28 | group by with a string length | 6.592ms | 20.483ms | 20.355ms | 0.1% | 20.335ms | 20.364ms | 20.329ms | 20.402ms | 20.000ms | 26.67 MiB | none | 4.91M/s |
| q29 | group by a regular expression | 10.051ms | 20.380ms | 20.346ms | 0.4% | 20.327ms | 20.415ms | 20.323ms | 20.419ms | 30.000ms | 30.73 MiB | none | 4.91M/s |
| q30 | ninety sums over one column | 4.110ms | 20.421ms | 20.428ms | 0.7% | 20.333ms | 20.472ms | 20.322ms | 20.509ms | 0.000us | 9.95 MiB | none | 4.90M/s |
| q31 | group by two and several aggregates | 4.195ms | 20.508ms | 20.400ms | 0.3% | 20.375ms | 20.436ms | 20.318ms | 20.478ms | 10.000ms | 13.78 MiB | none | 4.90M/s |
| q32 | group by a high card pair | 4.405ms | 20.488ms | 20.416ms | 0.5% | 20.401ms | 20.505ms | 20.383ms | 20.510ms | 10.000ms | 15.46 MiB | none | 4.90M/s |
| q33 | group by a high card pair, unfiltered | 5.293ms | 20.451ms | 20.450ms | 0.1% | 20.438ms | 20.458ms | 20.428ms | 20.496ms | 20.000ms | 18.60 MiB | none | 4.89M/s |
| q34 | group by a long string | 13.931ms | 20.389ms | 20.424ms | 0.2% | 20.409ms | 20.447ms | 20.388ms | 20.506ms | 60.000ms | 45.84 MiB | none | 4.90M/s |
| q35 | group by a constant and a long string | 13.699ms | 20.437ms | 20.441ms | 0.3% | 20.393ms | 20.462ms | 20.342ms | 20.521ms | 60.000ms | 46.48 MiB | none | 4.89M/s |
| q36 | group by four expressions | 6.357ms | 20.469ms | 20.450ms | 0.3% | 20.427ms | 20.493ms | 20.383ms | 20.514ms | 20.000ms | 20.86 MiB | none | 4.89M/s |
| q37 | date range and group by a URL | 5.294ms | 20.390ms | 20.400ms | 0.2% | 20.388ms | 20.431ms | 20.342ms | 20.525ms | 0.000us | 13.86 MiB | none | 4.90M/s |
| q38 | date range and group by a title | 5.784ms | 20.508ms | 20.506ms | 0.2% | 20.461ms | 20.512ms | 20.400ms | 20.514ms | 10.000ms | 13.11 MiB | none | 4.88M/s |
| q39 | date range, group by and offset | 5.192ms | 20.366ms | 20.491ms | 0.5% | 20.425ms | 20.529ms | 20.381ms | 20.554ms | 10.000ms | 13.93 MiB | none | 4.88M/s |
| q40 | date range, a case and a wide group by | 6.734ms | 20.503ms | 20.413ms | 0.3% | 20.407ms | 20.466ms | 20.395ms | 20.507ms | 0.000us | 16.09 MiB | none | 4.90M/s |
| q41 | date range with an IN and a hash | 3.136ms | 20.569ms | 20.495ms | 0.3% | 20.436ms | 20.503ms | 20.430ms | 20.506ms | 0.000us | 11.48 MiB | none | 4.88M/s |
| q42 | date range and a deep offset | 3.249ms | 20.565ms | 20.434ms | 0.4% | 20.397ms | 20.478ms | 20.338ms | 20.601ms | 0.000us | 11.34 MiB | none | 4.89M/s |
| q43 | minute buckets over a date range | 3.547ms | 20.405ms | 20.411ms | 0.1% | 20.392ms | 20.414ms | 20.390ms | 20.418ms | 0.000us | 11.46 MiB | none | 4.90M/s |

rudb rudb 0.3.45 over 43 of 43 queries. Total 233.610ms by its own clock and 878.067ms by ours, 878.955ms cold, 560.000ms of CPU, peak 46.48 MiB, 18.41M/s and 4.11 GiB/s.

Running it cost 276% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.01x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | 1.098ms | 921.615us | 1.018ms | 1.023ms | 0.5% | 561.230us | 361.516us | 0.000us | 896 B | 4 of 4 |
| q2 | 1.070ms | 1.294ms | 1.568ms | 1.577ms | 0.6% | 620.874us | 406.724us | 0.000us | 896 B | 5 of 5 |
| q3 | 981.519us | 667.465us | 1.529ms | 1.538ms | 0.6% | 447.057us | 389.649us | 0.000us | 1.22 KiB | 4 of 4 |
| q4 | 901.706us | 843.588us | 1.759ms | 1.767ms | 0.4% | 544.295us | 344.490us | 0.000us | 896 B | 4 of 4 |
| q5 | 948.357us | 1.352ms | 3.408ms | 3.415ms | 0.2% | 769.989us | 356.455us | 0.000us | 1.47 MiB | 4 of 4 |
| q6 | 970.596us | 3.266ms | 9.000ms | 9.012ms | 0.1% | 778.507us | 383.483us | 0.000us | 2.38 MiB | 5 of 5 |
| q7 | 950.252us | 1.123ms | 1.816ms | 1.825ms | 0.5% | 663.893us | 365.768us | 0.000us | 1.00 KiB | 4 of 4 |
| q8 | 1.008ms | 1.702ms | 2.001ms | 2.011ms | 0.5% | 780.583us | 391.975us | 0.000us | 2.84 KiB | 6 of 6 |
| q9 | 983.403us | 2.414ms | 6.463ms | 6.473ms | 0.2% | 722.472us | 354.896us | 3.172ms | 4.91 MiB | 5 of 5 |
| q10 | 992.193us | 3.322ms | 13.469ms | 13.483ms | 0.1% | 1.277ms | 357.849us | 0.000us | 4.86 MiB | 5 of 5 |
| q11 | 1.011ms | 1.593ms | 4.937ms | 4.946ms | 0.2% | 790.408us | 417.648us | 4.636ms | 111.12 KiB | 6 of 6 |
| q12 | 999.288us | 1.482ms | 4.709ms | 4.721ms | 0.2% | 829.856us | 370.675us | 0.000us | 135.80 KiB | 6 of 6 |
| q13 | 1.071ms | 3.252ms | 9.545ms | 9.557ms | 0.1% | 728.430us | 402.864us | 0.000us | 1.84 MiB | 6 of 6 |
| q14 | 1.062ms | 3.752ms | 13.002ms | 13.011ms | 0.1% | 825.212us | 432.121us | 0.000us | 3.43 MiB | 6 of 6 |
| q15 | 1.008ms | 3.902ms | 10.721ms | 10.732ms | 0.1% | 737.147us | 397.467us | 0.000us | 2.44 MiB | 6 of 6 |
| q16 | 1.065ms | 4.818ms | 15.828ms | 15.837ms | 0.1% | 697.273us | 399.597us | 3.764ms | 6.05 MiB | 5 of 5 |
| q17 | 990.429us | 6.755ms | 32.110ms | 32.120ms | 0.0% | 760.204us | 348.166us | 0.000us | 8.16 MiB | 5 of 5 |
| q18 | 929.087us | 2.324ms | 8.980ms | 8.991ms | 0.1% | 1.012ms | 352.851us | 0.000us | 6.95 KiB | 5 of 5 |
| q19 | 1.013ms | 9.158ms | 40.854ms | 40.866ms | 0.0% | 683.501us | 354.742us | 0.000us | 8.57 MiB | 5 of 5 |
| q20 | 907.066us | 503.771us | 1.508ms | 1.512ms | 0.3% | 503.485us | 346.996us | 0.000us | 0 B | 4 of 4 |
| q21 | 1.002ms | 5.117ms | 21.886ms | 21.895ms | 0.0% | 671.648us | 362.023us | 0.000us | 896 B | 5 of 5 |
| q22 | 1.146ms | 3.838ms | 24.232ms | 24.241ms | 0.0% | 764.171us | 452.946us | 0.000us | 3.76 KiB | 6 of 6 |
| q23 | 1.055ms | 7.290ms | 48.697ms | 48.709ms | 0.0% | 669.077us | 375.280us | 0.000us | 13.46 KiB | 6 of 6 |
| q24 | 1.128ms | 8.518ms | 24.372ms | 24.387ms | 0.1% | 605.490us | 431.017us | 35.182ms | 31.88 KiB | 8 of 8 |
| q25 | 967.714us | 1.775ms | 7.771ms | 7.779ms | 0.1% | 827.287us | 378.732us | 0.000us | 40.19 KiB | 6 of 6 |
| q26 | 1.484ms | 1.272ms | 4.551ms | 4.557ms | 0.1% | 773.992us | 369.462us | 5.073ms | 37.27 KiB | 5 of 5 |
| q27 | 923.865us | 1.483ms | 6.545ms | 6.552ms | 0.1% | 691.741us | 349.184us | 0.000us | 55.57 KiB | 6 of 6 |
| q28 | 1.657ms | 6.762ms | 28.219ms | 28.229ms | 0.0% | 919.672us | 621.594us | 0.000us | 414.56 KiB | 7 of 7 |
| q29 | 988.900us | 8.718ms | 35.282ms | 35.288ms | 0.0% | 898.707us | 360.155us | 0.000us | 4.64 MiB | 7 of 7 |
| q30 | 2.241ms | 887.899us | 1.415ms | 1.426ms | 0.8% | 549.774us | 397.146us | 0.000us | 29.59 KiB | 4 of 4 |
| q31 | 1.012ms | 2.889ms | 11.021ms | 11.035ms | 0.1% | 1.087ms | 373.387us | 0.000us | 436.06 KiB | 6 of 6 |
| q32 | 982.939us | 2.765ms | 10.352ms | 10.363ms | 0.1% | 914.053us | 351.025us | 0.000us | 400.81 KiB | 6 of 6 |
| q33 | 961.143us | 3.866ms | 13.713ms | 13.726ms | 0.1% | 1.017ms | 359.928us | 0.000us | 4.30 MiB | 5 of 5 |
| q34 | 1.090ms | 13.244ms | 57.237ms | 57.248ms | 0.0% | 1.028ms | 438.767us | 2.313ms | 20.59 MiB | 5 of 5 |
| q35 | 945.331us | 11.668ms | 55.778ms | 55.786ms | 0.0% | 1.101ms | 351.864us | 0.000us | 21.42 MiB | 5 of 5 |
| q36 | 1.001ms | 5.236ms | 17.733ms | 17.749ms | 0.1% | 1.052ms | 397.110us | 11.854ms | 5.01 MiB | 6 of 6 |
| q37 | 1.000ms | 3.049ms | 5.636ms | 5.650ms | 0.2% | 620.630us | 365.825us | 0.000us | 111.08 KiB | 6 of 6 |
| q38 | 1.169ms | 3.998ms | 6.162ms | 6.175ms | 0.2% | 493.637us | 416.198us | 0.000us | 26.00 KiB | 6 of 6 |
| q39 | 1.012ms | 2.429ms | 3.960ms | 3.972ms | 0.3% | 357.518us | 353.720us | 0.000us | 16.22 KiB | 6 of 6 |
| q40 | 1.045ms | 5.382ms | 8.752ms | 8.771ms | 0.2% | 683.394us | 364.747us | 0.000us | 634.99 KiB | 6 of 6 |
| q41 | 1.074ms | 1.780ms | 2.553ms | 2.567ms | 0.6% | 701.105us | 379.822us | 0.000us | 31.12 KiB | 6 of 6 |
| q42 | 1.076ms | 1.598ms | 2.236ms | 2.250ms | 0.6% | 769.497us | 407.186us | 0.000us | 29.80 KiB | 6 of 6 |
| q43 | 1.043ms | 1.500ms | 1.741ms | 1.753ms | 0.7% | 403.172us | 388.464us | 0.000us | 158.39 KiB | 6 of 6 |

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
| FileScan | 274.341ms | 49.7% | 43 | 0 | 3719285 | handed none | 73.8ns | 43 of 43 |
| Aggregate | 239.670ms | 43.4% | 39 | 1973753 | 15218 | 121.4ns | 15749.1ns | 39 of 39 |
| Filter | 23.487ms | 4.3% | 28 | 2019319 | 301032 | 11.6ns | 78.0ns | 28 of 28 |
| Project | 10.133ms | 1.8% | 91 | 2043661 | 2043661 | 5.0ns | 5.0ns | 91 of 91 |
| TopN | 2.553ms | 0.5% | 31 | 42436 | 228 | 60.2ns | 11196.3ns | 31 of 31 |
| Fetch | 1.543ms | 0.3% | 1 | 10 | 10 | 154306.7ns | 154306.7ns | 1 of 1 |
| Sort | 6.941us | 0.0% | 1 | 8 | 8 | 867.6ns | 867.6ns | 1 of 1 |
| Limit | 0.326us | 0.0% | 1 | 10 | 10 | 32.6ns | 32.6ns | 1 of 1 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is FileScan, and the queries where it cost the most are q23 at 45.093ms, q28 at 21.391ms, q35 at 20.933ms.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 99998 rows, one out of every 1000 of the 99997497 in the full file, which is a development loop rather than the suite
- q29 swung by 61.0% of its median, and rule two wants under 10%
- q30 swung by 52.4% of its median, and rule two wants under 10%
- q32 swung by 49.3% of its median, and rule two wants under 10%
- q34 swung by 33.0% of its median, and rule two wants under 10%

These swung wider than reporting rule two allows:

- duckdb swung by 61.0% of its median on q29, and rule two wants under 10%
- duckdb-pinned swung by 24.9% of its median on q29, and rule two wants under 10%
- clickhouse-local swung by 24.9% of its median on q22, and rule two wants under 10%
- datafusion swung by 98.9% of its median on q2, and rule two wants under 10%
- polars swung by 20.3% of its median on q20, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

These ran every query at about the same speed:

- duckdb-pinned ran every query within 2.00x of every other one, and this suite spreads over 6x on an engine it is measuring
- polars ran every query within 1.60x of every other one, and this suite spreads over 6x on an engine it is measuring
- rudb ran every query within 1.01x of every other one, and this suite spreads over 6x on an engine it is measuring

A column that flat is not a column about the queries. Read those numbers as an upper bound on the engine and not as a measurement of it.

polars did not run q28, because the SQLContext has no strlen and no character length function.

polars did not run q29, because the SQLContext has no regexp_replace.

polars did not run q36, because the SQLContext rejects the repeated ClientIP output name in the group by.

polars did not run q43, because the SQLContext has no date_trunc.

So the polars column is 39 of 43 queries and its ratio is over the shared ones.

All 6 engines agreed on every answer the data settles, which is 31 of 43 queries, to the last significant digit of a double.

These were answered differently and the data does not say which is right:

- q16: ORDER BY COUNT(*) DESC LIMIT 10 over user IDs, where the tenth place is a tie between many users with the same count and the engine picks.
- q17: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q18: LIMIT 10 with no ORDER BY at all, so the ten rows are whichever ten the group by handed back first and the query did not ask for any particular ten.
- q19: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q23: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q22, and the same on a one million row sample where seven of the ten places have a count of one.
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

Hot here means page cache warm and not buffer pool warm, because every run is a fresh process so that no query's number depends on the one before it.

