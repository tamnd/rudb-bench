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
| timeout | 900s per query, and no query reached it |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --rows 1000 --engines duckdb,duckdb-pinned,clickhouse-local,datafusion,polars,rudb --runs 5 --timeout 900 --report
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
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 48.901ms | 30.000ms | 1.01 MiB | its own database file | its own | 10.21 to 9.39 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 56.465ms | 40.000ms | 1.01 MiB | its own database file | its own | 9.39 to 8.66 |
| clickhouse-local | 26.9.1.1562 | ran | 135.498ms | 140.000ms | 362.94 KiB | its own MergeTree parts, as system.parts counts the active ones | its own | 8.66 to 6.96 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 293.41 KiB | the source Parquet, this engine has no storage format of its own | the Parquet | 6.96 to 6.48 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 293.41 KiB | the source Parquet, this engine has no storage format of its own | the Parquet | 6.48 to 4.76 |
| rudb | rudb 0.3.67 | ran | 0.000us | 0.000us | 293.41 KiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 4.76 to 4.17 |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb on 39 shared |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 88.000ms | 870.850ms | +890% | 871.157ms | 210.000ms | 0.24 | 40.39 MiB | none | 488.64K/s | 140.01 MiB/s | 1.00x |
| duckdb-pinned | 127.000ms | 1.740s | +1270% | 1.741s | 780.000ms | 0.45 | 53.02 MiB | none | 338.58K/s | 97.01 MiB/s | 1.49x |
| clickhouse-local | 319.000ms | 3.083s | +867% | 3.112s | 2.830s | 0.92 | 247.78 MiB | none | 134.80K/s | 38.62 MiB/s | 3.69x |
| datafusion | 187.000ms | 872.859ms | +367% | 894.350ms | 460.000ms | 0.53 | 159.01 MiB | none | 229.95K/s | 65.89 MiB/s | 2.14x |
| polars | 567.734ms | 4.112s | +624% | 4.130s | 5.170s | 1.26 | 83.24 MiB | none | 68.69K/s | 19.68 MiB/s | 7.28x |
| rudb | 28.817ms | 880.945ms | +2957% | 880.453ms | 0.000us | 0.00 | 11.23 MiB | none | 1.49M/s | 427.55 MiB/s | 0.33x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 0.000us | 1.000ms | 5.000ms | 1.000ms | 5.627ms | 480.672us |
| q2 | filtered count | 1.000ms | 1.000ms | 5.000ms | 2.000ms | 6.559ms | 492.156us |
| q3 | three aggregates | 1.000ms | 1.000ms | 5.000ms | 1.000ms | 6.107ms | 483.874us |
| q4 | average | 1.000ms | 1.000ms | 5.000ms | 1.000ms | 4.977ms | 485.343us |
| q5 | count distinct, high card | 2.000ms | 2.000ms | 5.000ms | 3.000ms | 12.118ms | 488.483us |
| q6 | count distinct, strings | 2.000ms | 2.000ms | 5.000ms | 3.000ms | 11.218ms | 562.545us |
| q7 | min and max of a date | 1.000ms | 1.000ms | 9.000ms | 1.000ms | 6.196ms | 510.220us |
| q8 | group by, low card | 1.000ms | 6.000ms | 7.000ms | 4.000ms | 12.549ms | 509.939us |
| q9 | group by and count distinct | 3.000ms | 4.000ms | 6.000ms | 6.000ms | 22.953ms | 567.942us |
| q10 | group by, several aggregates | 3.000ms | 5.000ms | 6.000ms | 6.000ms | 26.951ms | 809.265us |
| q11 | group by a string and count distinct | 3.000ms | 4.000ms | 6.000ms | 6.000ms | 22.142ms | 576.152us |
| q12 | group by two strings and count distinct | 3.000ms | 4.000ms | 6.000ms | 7.000ms | 22.030ms | 584.404us |
| q13 | group by a string and top k | 2.000ms | 2.000ms | 6.000ms | 5.000ms | 16.390ms | 562.923us |
| q14 | group by a string and count distinct | 3.000ms | 4.000ms | 6.000ms | 7.000ms | 22.349ms | 599.180us |
| q15 | group by two columns and top k | 2.000ms | 3.000ms | 6.000ms | 5.000ms | 15.986ms | 664.785us |
| q16 | group by, very high card | 2.000ms | 2.000ms | 6.000ms | 3.000ms | 19.274ms | 547.637us |
| q17 | group by two, very high card | 3.000ms | 2.000ms | 6.000ms | 4.000ms | 22.344ms | 609.995us |
| q18 | group by two, no ordering | 3.000ms | 3.000ms | 6.000ms | 3.000ms | 11.345ms | 514.308us |
| q19 | group by with an extract | 2.000ms | 3.000ms | 6.000ms | 4.000ms | 24.231ms | 749.098us |
| q20 | point lookup | 0.000us | 1.000ms | 10.000ms | 2.000ms | 5.354ms | 417.569us |
| q21 | substring scan | 1.000ms | 1.000ms | 6.000ms | 3.000ms | 6.695ms | 640.095us |
| q22 | substring scan and group by | 1.000ms | 2.000ms | 6.000ms | 4.000ms | 15.171ms | 724.903us |
| q23 | two substring scans and group by | 1.000ms | 3.000ms | 7.000ms | 5.000ms | 22.079ms | 967.949us |
| q24 | select star and top k | 4.000ms | 9.000ms | 13.000ms | 7.000ms | 9.430ms | 798.771us |
| q25 | top k by a date | 2.000ms | 2.000ms | 10.000ms | 3.000ms | 12.672ms | 482.298us |
| q26 | top k by a string | 0.000us | 1.000ms | 5.000ms | 3.000ms | 10.822ms | 482.722us |
| q27 | top k by two columns | 1.000ms | 2.000ms | 10.000ms | 3.000ms | 12.580ms | 516.309us |
| q28 | group by with a string length | 2.000ms | 3.000ms | 6.000ms | 5.000ms | no dialect | 732.565us |
| q29 | group by a regular expression | 3.000ms | 4.000ms | 7.000ms | 7.000ms | no dialect | 800.654us |
| q30 | ninety sums over one column | 4.000ms | 14.000ms | 9.000ms | 10.000ms | 12.044ms | 1.771ms |
| q31 | group by two and several aggregates | 3.000ms | 3.000ms | 6.000ms | 5.000ms | 15.712ms | 789.107us |
| q32 | group by a high card pair | 2.000ms | 2.000ms | 6.000ms | 6.000ms | 16.904ms | 732.208us |
| q33 | group by a high card pair, unfiltered | 3.000ms | 3.000ms | 6.000ms | 4.000ms | 16.531ms | 771.485us |
| q34 | group by a long string | 3.000ms | 2.000ms | 6.000ms | 4.000ms | 17.097ms | 731.945us |
| q35 | group by a constant and a long string | 3.000ms | 3.000ms | 6.000ms | 4.000ms | 17.090ms | 770.761us |
| q36 | group by four expressions | 3.000ms | 2.000ms | 6.000ms | 3.000ms | no dialect | 621.533us |
| q37 | date range and group by a URL | 2.000ms | 2.000ms | 12.000ms | 6.000ms | 16.357ms | 743.786us |
| q38 | date range and group by a title | 2.000ms | 2.000ms | 12.000ms | 5.000ms | 15.476ms | 853.130us |
| q39 | date range, group by and offset | 1.000ms | 1.000ms | 11.000ms | 5.000ms | 13.529ms | 770.621us |
| q40 | date range, a case and a wide group by | 3.000ms | 3.000ms | 11.000ms | 6.000ms | 14.172ms | 895.871us |
| q41 | date range with an IN and a hash | 2.000ms | 3.000ms | 12.000ms | 5.000ms | 13.646ms | 645.386us |
| q42 | date range and a deep offset | 2.000ms | 6.000ms | 12.000ms | 5.000ms | 13.027ms | 730.745us |
| q43 | minute buckets over a date range | 2.000ms | 2.000ms | 12.000ms | 5.000ms | no dialect | 626.474us |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 0.000us | 20.200ms | 20.350ms | 0.3% | 20.286ms | 20.351ms | 20.202ms | 20.357ms | 0.000us | 30.62 MiB | none | 49.14K/s |
| q2 | filtered count | 1.000ms | 20.435ms | 20.270ms | 0.3% | 20.206ms | 20.273ms | 20.200ms | 20.343ms | 0.000us | 31.81 MiB | none | 49.34K/s |
| q3 | three aggregates | 1.000ms | 20.199ms | 20.356ms | 0.7% | 20.205ms | 20.357ms | 20.199ms | 20.393ms | 0.000us | 31.88 MiB | none | 49.13K/s |
| q4 | average | 1.000ms | 20.351ms | 20.217ms | 0.1% | 20.215ms | 20.237ms | 20.204ms | 20.274ms | 0.000us | 31.43 MiB | none | 49.46K/s |
| q5 | count distinct, high card | 2.000ms | 20.195ms | 20.272ms | 0.7% | 20.213ms | 20.349ms | 20.205ms | 20.363ms | 10.000ms | 33.75 MiB | none | 49.33K/s |
| q6 | count distinct, strings | 2.000ms | 20.379ms | 20.359ms | 0.5% | 20.261ms | 20.363ms | 20.222ms | 20.412ms | 0.000us | 34.49 MiB | none | 49.12K/s |
| q7 | min and max of a date | 1.000ms | 20.209ms | 20.212ms | 0.1% | 20.210ms | 20.225ms | 20.207ms | 20.351ms | 0.000us | 31.00 MiB | none | 49.48K/s |
| q8 | group by, low card | 1.000ms | 20.203ms | 20.357ms | 0.1% | 20.347ms | 20.360ms | 20.209ms | 20.396ms | 0.000us | 34.20 MiB | none | 49.12K/s |
| q9 | group by and count distinct | 3.000ms | 20.367ms | 20.235ms | 0.2% | 20.226ms | 20.258ms | 20.222ms | 20.367ms | 10.000ms | 38.27 MiB | none | 49.42K/s |
| q10 | group by, several aggregates | 3.000ms | 20.360ms | 20.238ms | 0.7% | 20.224ms | 20.364ms | 20.216ms | 20.370ms | 10.000ms | 40.39 MiB | none | 49.41K/s |
| q11 | group by a string and count distinct | 3.000ms | 20.219ms | 20.304ms | 0.3% | 20.294ms | 20.357ms | 20.216ms | 20.443ms | 10.000ms | 39.26 MiB | none | 49.25K/s |
| q12 | group by two strings and count distinct | 3.000ms | 20.218ms | 20.237ms | 0.7% | 20.222ms | 20.362ms | 20.220ms | 20.427ms | 10.000ms | 40.26 MiB | none | 49.41K/s |
| q13 | group by a string and top k | 2.000ms | 20.360ms | 20.225ms | 0.7% | 20.223ms | 20.362ms | 20.219ms | 20.410ms | 0.000us | 35.12 MiB | none | 49.44K/s |
| q14 | group by a string and count distinct | 3.000ms | 20.274ms | 20.228ms | 0.7% | 20.225ms | 20.361ms | 20.215ms | 20.375ms | 10.000ms | 39.45 MiB | none | 49.44K/s |
| q15 | group by two columns and top k | 2.000ms | 20.366ms | 20.233ms | 0.4% | 20.229ms | 20.311ms | 20.225ms | 20.311ms | 0.000us | 35.20 MiB | none | 49.42K/s |
| q16 | group by, very high card | 2.000ms | 20.320ms | 20.230ms | 0.1% | 20.226ms | 20.238ms | 20.219ms | 20.292ms | 10.000ms | 37.26 MiB | none | 49.43K/s |
| q17 | group by two, very high card | 3.000ms | 20.220ms | 20.221ms | 0.1% | 20.218ms | 20.232ms | 20.217ms | 20.425ms | 0.000us | 37.88 MiB | none | 49.45K/s |
| q18 | group by two, no ordering | 3.000ms | 20.216ms | 20.227ms | 0.1% | 20.222ms | 20.246ms | 20.219ms | 20.270ms | 10.000ms | 39.88 MiB | none | 49.44K/s |
| q19 | group by with an extract | 2.000ms | 20.222ms | 20.221ms | 0.1% | 20.216ms | 20.230ms | 20.209ms | 20.445ms | 0.000us | 37.76 MiB | none | 49.45K/s |
| q20 | point lookup | 0.000us | 20.209ms | 20.204ms | 0.0% | 20.203ms | 20.205ms | 20.199ms | 20.206ms | 0.000us | 31.19 MiB | none | 49.50K/s |
| q21 | substring scan | 1.000ms | 20.198ms | 20.210ms | 0.2% | 20.201ms | 20.250ms | 20.201ms | 20.307ms | 10.000ms | 31.75 MiB | none | 49.48K/s |
| q22 | substring scan and group by | 1.000ms | 20.206ms | 20.205ms | 0.0% | 20.198ms | 20.207ms | 20.196ms | 20.335ms | 0.000us | 32.69 MiB | none | 49.49K/s |
| q23 | two substring scans and group by | 1.000ms | 20.196ms | 20.210ms | 0.3% | 20.208ms | 20.270ms | 20.199ms | 20.321ms | 10.000ms | 32.70 MiB | none | 49.48K/s |
| q24 | select star and top k | 4.000ms | 20.213ms | 20.219ms | 0.0% | 20.216ms | 20.224ms | 20.207ms | 20.247ms | 10.000ms | 38.31 MiB | none | 49.46K/s |
| q25 | top k by a date | 2.000ms | 20.224ms | 20.211ms | 0.0% | 20.210ms | 20.216ms | 20.209ms | 20.218ms | 10.000ms | 34.11 MiB | none | 49.48K/s |
| q26 | top k by a string | 0.000us | 20.209ms | 20.211ms | 0.0% | 20.209ms | 20.216ms | 20.199ms | 20.221ms | 0.000us | 31.38 MiB | none | 49.48K/s |
| q27 | top k by two columns | 1.000ms | 20.198ms | 20.265ms | 0.2% | 20.236ms | 20.275ms | 20.216ms | 20.405ms | 0.000us | 31.69 MiB | none | 49.35K/s |
| q28 | group by with a string length | 2.000ms | 20.377ms | 20.340ms | 0.3% | 20.294ms | 20.346ms | 20.211ms | 20.353ms | 10.000ms | 35.39 MiB | none | 49.16K/s |
| q29 | group by a regular expression | 3.000ms | 20.356ms | 20.229ms | 0.6% | 20.224ms | 20.353ms | 20.222ms | 20.381ms | 10.000ms | 35.43 MiB | none | 49.43K/s |
| q30 | ninety sums over one column | 4.000ms | 20.220ms | 20.211ms | 0.0% | 20.210ms | 20.214ms | 20.206ms | 20.351ms | 10.000ms | 35.38 MiB | none | 49.48K/s |
| q31 | group by two and several aggregates | 3.000ms | 20.213ms | 20.352ms | 0.6% | 20.307ms | 20.424ms | 20.212ms | 20.442ms | 0.000us | 37.64 MiB | none | 49.13K/s |
| q32 | group by a high card pair | 2.000ms | 20.223ms | 20.245ms | 0.1% | 20.231ms | 20.261ms | 20.214ms | 20.359ms | 0.000us | 37.82 MiB | none | 49.39K/s |
| q33 | group by a high card pair, unfiltered | 3.000ms | 20.217ms | 20.228ms | 0.7% | 20.216ms | 20.360ms | 20.216ms | 20.365ms | 0.000us | 37.57 MiB | none | 49.44K/s |
| q34 | group by a long string | 3.000ms | 20.355ms | 20.234ms | 0.3% | 20.226ms | 20.293ms | 20.223ms | 20.373ms | 0.000us | 34.88 MiB | none | 49.42K/s |
| q35 | group by a constant and a long string | 3.000ms | 20.224ms | 20.224ms | 0.6% | 20.224ms | 20.351ms | 20.221ms | 20.354ms | 10.000ms | 36.07 MiB | none | 49.45K/s |
| q36 | group by four expressions | 3.000ms | 20.349ms | 20.262ms | 0.6% | 20.221ms | 20.351ms | 20.217ms | 20.384ms | 10.000ms | 37.51 MiB | none | 49.35K/s |
| q37 | date range and group by a URL | 2.000ms | 20.217ms | 20.219ms | 0.0% | 20.215ms | 20.219ms | 20.213ms | 20.269ms | 10.000ms | 35.14 MiB | none | 49.46K/s |
| q38 | date range and group by a title | 2.000ms | 20.211ms | 20.357ms | 0.5% | 20.261ms | 20.365ms | 20.206ms | 20.377ms | 10.000ms | 35.14 MiB | none | 49.12K/s |
| q39 | date range, group by and offset | 1.000ms | 20.370ms | 20.225ms | 0.1% | 20.212ms | 20.236ms | 20.210ms | 20.311ms | 0.000us | 32.39 MiB | none | 49.44K/s |
| q40 | date range, a case and a wide group by | 3.000ms | 20.205ms | 20.217ms | 0.0% | 20.217ms | 20.220ms | 20.208ms | 20.320ms | 10.000ms | 36.64 MiB | none | 49.46K/s |
| q41 | date range with an IN and a hash | 2.000ms | 20.224ms | 20.258ms | 0.7% | 20.223ms | 20.359ms | 20.217ms | 20.373ms | 0.000us | 37.70 MiB | none | 49.36K/s |
| q42 | date range and a deep offset | 2.000ms | 20.212ms | 20.221ms | 0.0% | 20.217ms | 20.226ms | 20.211ms | 20.324ms | 10.000ms | 35.20 MiB | none | 49.45K/s |
| q43 | minute buckets over a date range | 2.000ms | 20.216ms | 20.302ms | 0.6% | 20.227ms | 20.358ms | 20.224ms | 20.359ms | 0.000us | 35.12 MiB | none | 49.26K/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 88.000ms by its own clock and 870.850ms by ours, 871.157ms cold, 210.000ms of CPU, peak 40.39 MiB, 488.64K/s and 140.01 MiB/s.

Running it cost 890% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.01x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 40.297ms | 40.352ms | 0.3% | 40.314ms | 40.416ms | 40.299ms | 40.472ms | 20.000ms | 39.58 MiB | none | 24.78K/s |
| q2 | filtered count | 1.000ms | 40.531ms | 40.461ms | 0.3% | 40.377ms | 40.487ms | 40.338ms | 40.501ms | 10.000ms | 40.08 MiB | none | 24.72K/s |
| q3 | three aggregates | 1.000ms | 40.848ms | 40.470ms | 0.3% | 40.430ms | 40.545ms | 40.411ms | 41.264ms | 20.000ms | 40.57 MiB | none | 24.71K/s |
| q4 | average | 1.000ms | 40.998ms | 40.479ms | 0.5% | 40.308ms | 40.519ms | 40.292ms | 40.549ms | 10.000ms | 40.58 MiB | none | 24.70K/s |
| q5 | count distinct, high card | 2.000ms | 40.334ms | 40.342ms | 0.1% | 40.313ms | 40.343ms | 40.303ms | 40.446ms | 20.000ms | 43.02 MiB | none | 24.79K/s |
| q6 | count distinct, strings | 2.000ms | 40.550ms | 40.375ms | 0.1% | 40.367ms | 40.405ms | 40.315ms | 40.557ms | 20.000ms | 41.84 MiB | none | 24.77K/s |
| q7 | min and max of a date | 1.000ms | 40.503ms | 40.442ms | 1.2% | 40.404ms | 40.900ms | 40.384ms | 40.948ms | 20.000ms | 40.14 MiB | none | 24.73K/s |
| q8 | group by, low card | 6.000ms | 40.359ms | 40.358ms | 5.5% | 40.330ms | 42.555ms | 40.325ms | 42.804ms | 20.000ms | 42.85 MiB | none | 24.78K/s |
| q9 | group by and count distinct | 4.000ms | 40.317ms | 40.507ms | 0.4% | 40.352ms | 40.509ms | 40.304ms | 40.514ms | 20.000ms | 47.12 MiB | none | 24.69K/s |
| q10 | group by, several aggregates | 5.000ms | 40.319ms | 40.506ms | 0.2% | 40.425ms | 40.525ms | 40.325ms | 40.613ms | 20.000ms | 48.43 MiB | none | 24.69K/s |
| q11 | group by a string and count distinct | 4.000ms | 40.355ms | 40.489ms | 0.3% | 40.426ms | 40.535ms | 40.319ms | 40.706ms | 20.000ms | 46.87 MiB | none | 24.70K/s |
| q12 | group by two strings and count distinct | 4.000ms | 40.531ms | 40.401ms | 0.1% | 40.398ms | 40.442ms | 40.363ms | 40.480ms | 20.000ms | 48.18 MiB | none | 24.75K/s |
| q13 | group by a string and top k | 2.000ms | 40.516ms | 40.428ms | 0.2% | 40.386ms | 40.466ms | 40.297ms | 40.634ms | 20.000ms | 42.32 MiB | none | 24.74K/s |
| q14 | group by a string and count distinct | 4.000ms | 40.470ms | 40.592ms | 0.2% | 40.510ms | 40.599ms | 40.502ms | 41.050ms | 20.000ms | 47.64 MiB | none | 24.64K/s |
| q15 | group by two columns and top k | 3.000ms | 40.355ms | 40.326ms | 0.5% | 40.320ms | 40.529ms | 40.316ms | 41.025ms | 20.000ms | 44.02 MiB | none | 24.80K/s |
| q16 | group by, very high card | 2.000ms | 40.306ms | 40.300ms | 0.2% | 40.287ms | 40.379ms | 40.270ms | 41.207ms | 20.000ms | 43.63 MiB | none | 24.81K/s |
| q17 | group by two, very high card | 2.000ms | 40.293ms | 40.436ms | 0.2% | 40.419ms | 40.506ms | 40.387ms | 40.506ms | 20.000ms | 44.16 MiB | none | 24.73K/s |
| q18 | group by two, no ordering | 3.000ms | 40.661ms | 40.531ms | 0.0% | 40.523ms | 40.538ms | 40.344ms | 40.546ms | 20.000ms | 43.87 MiB | none | 24.67K/s |
| q19 | group by with an extract | 3.000ms | 40.442ms | 40.387ms | 0.3% | 40.360ms | 40.469ms | 40.308ms | 40.731ms | 10.000ms | 45.12 MiB | none | 24.76K/s |
| q20 | point lookup | 1.000ms | 40.297ms | 40.498ms | 1.3% | 40.480ms | 41.024ms | 40.315ms | 41.207ms | 10.000ms | 39.33 MiB | none | 24.69K/s |
| q21 | substring scan | 1.000ms | 40.487ms | 40.587ms | 0.4% | 40.448ms | 40.619ms | 40.379ms | 41.462ms | 10.000ms | 40.58 MiB | none | 24.64K/s |
| q22 | substring scan and group by | 2.000ms | 40.315ms | 40.467ms | 1.1% | 40.399ms | 40.849ms | 40.291ms | 41.342ms | 10.000ms | 41.33 MiB | none | 24.71K/s |
| q23 | two substring scans and group by | 3.000ms | 40.562ms | 40.395ms | 0.2% | 40.372ms | 40.450ms | 40.348ms | 40.482ms | 20.000ms | 43.36 MiB | none | 24.76K/s |
| q24 | select star and top k | 9.000ms | 40.430ms | 40.488ms | 0.1% | 40.434ms | 40.493ms | 40.371ms | 40.567ms | 30.000ms | 47.74 MiB | none | 24.70K/s |
| q25 | top k by a date | 2.000ms | 40.304ms | 40.564ms | 0.8% | 40.490ms | 40.807ms | 40.356ms | 40.977ms | 20.000ms | 40.33 MiB | none | 24.65K/s |
| q26 | top k by a string | 1.000ms | 40.626ms | 40.668ms | 0.6% | 40.531ms | 40.777ms | 40.351ms | 40.908ms | 10.000ms | 40.52 MiB | none | 24.59K/s |
| q27 | top k by two columns | 2.000ms | 40.424ms | 40.559ms | 0.8% | 40.322ms | 40.650ms | 40.316ms | 40.687ms | 10.000ms | 40.58 MiB | none | 24.66K/s |
| q28 | group by with a string length | 3.000ms | 40.706ms | 40.438ms | 0.2% | 40.359ms | 40.443ms | 40.317ms | 40.464ms | 20.000ms | 43.08 MiB | none | 24.73K/s |
| q29 | group by a regular expression | 4.000ms | 40.910ms | 40.460ms | 0.0% | 40.445ms | 40.462ms | 40.295ms | 40.668ms | 20.000ms | 44.10 MiB | none | 24.72K/s |
| q30 | ninety sums over one column | 14.000ms | 40.400ms | 40.414ms | 0.1% | 40.410ms | 40.443ms | 40.380ms | 40.564ms | 30.000ms | 53.02 MiB | none | 24.74K/s |
| q31 | group by two and several aggregates | 3.000ms | 40.420ms | 40.482ms | 0.3% | 40.413ms | 40.540ms | 40.355ms | 40.617ms | 20.000ms | 45.84 MiB | none | 24.70K/s |
| q32 | group by a high card pair | 2.000ms | 40.516ms | 40.538ms | 0.1% | 40.533ms | 40.576ms | 40.421ms | 40.640ms | 10.000ms | 44.77 MiB | none | 24.67K/s |
| q33 | group by a high card pair, unfiltered | 3.000ms | 40.302ms | 40.491ms | 0.2% | 40.454ms | 40.527ms | 40.305ms | 40.542ms | 20.000ms | 44.77 MiB | none | 24.70K/s |
| q34 | group by a long string | 2.000ms | 40.439ms | 40.381ms | 0.5% | 40.360ms | 40.548ms | 40.322ms | 40.870ms | 20.000ms | 43.02 MiB | none | 24.76K/s |
| q35 | group by a constant and a long string | 3.000ms | 40.382ms | 40.380ms | 0.2% | 40.348ms | 40.432ms | 40.319ms | 40.572ms | 20.000ms | 43.14 MiB | none | 24.76K/s |
| q36 | group by four expressions | 2.000ms | 40.313ms | 40.537ms | 0.6% | 40.523ms | 40.750ms | 40.382ms | 40.778ms | 20.000ms | 44.15 MiB | none | 24.67K/s |
| q37 | date range and group by a URL | 2.000ms | 40.697ms | 40.527ms | 0.4% | 40.455ms | 40.630ms | 40.410ms | 40.796ms | 10.000ms | 42.82 MiB | none | 24.67K/s |
| q38 | date range and group by a title | 2.000ms | 40.754ms | 40.458ms | 0.7% | 40.454ms | 40.722ms | 40.415ms | 41.344ms | 20.000ms | 42.58 MiB | none | 24.72K/s |
| q39 | date range, group by and offset | 1.000ms | 40.569ms | 40.511ms | 0.9% | 40.502ms | 40.884ms | 40.444ms | 41.654ms | 20.000ms | 41.26 MiB | none | 24.68K/s |
| q40 | date range, a case and a wide group by | 3.000ms | 40.337ms | 40.315ms | 0.1% | 40.315ms | 40.355ms | 40.313ms | 40.432ms | 20.000ms | 44.41 MiB | none | 24.80K/s |
| q41 | date range with an IN and a hash | 3.000ms | 40.437ms | 40.357ms | 0.2% | 40.353ms | 40.452ms | 40.339ms | 40.568ms | 20.000ms | 44.74 MiB | none | 24.78K/s |
| q42 | date range and a deep offset | 6.000ms | 40.351ms | 40.470ms | 0.5% | 40.315ms | 40.536ms | 40.308ms | 40.748ms | 20.000ms | 44.33 MiB | none | 24.71K/s |
| q43 | minute buckets over a date range | 2.000ms | 40.648ms | 40.356ms | 0.2% | 40.309ms | 40.387ms | 40.288ms | 40.395ms | 20.000ms | 42.56 MiB | none | 24.78K/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 127.000ms by its own clock and 1.740s by ours, 1.741s cold, 780.000ms of CPU, peak 53.02 MiB, 338.58K/s and 97.01 MiB/s.

Running it cost 1270% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.01x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 5.000ms | 60.519ms | 60.704ms | 33.0% | 60.427ms | 80.488ms | 60.384ms | 80.591ms | 60.000ms | 238.99 MiB | none | 16.47K/s |
| q2 | filtered count | 5.000ms | 60.587ms | 60.395ms | 0.1% | 60.387ms | 60.445ms | 60.383ms | 60.684ms | 60.000ms | 239.46 MiB | none | 16.56K/s |
| q3 | three aggregates | 5.000ms | 60.452ms | 60.441ms | 0.1% | 60.432ms | 60.499ms | 60.413ms | 61.284ms | 60.000ms | 241.20 MiB | none | 16.55K/s |
| q4 | average | 5.000ms | 60.390ms | 80.497ms | 24.8% | 60.568ms | 80.561ms | 60.444ms | 99.517ms | 60.000ms | 241.13 MiB | none | 12.42K/s |
| q5 | count distinct, high card | 5.000ms | 60.362ms | 60.516ms | 0.3% | 60.383ms | 60.547ms | 60.372ms | 90.172ms | 100.000ms | 241.50 MiB | none | 16.52K/s |
| q6 | count distinct, strings | 5.000ms | 80.760ms | 60.480ms | 0.2% | 60.401ms | 60.531ms | 60.381ms | 80.439ms | 80.000ms | 241.01 MiB | none | 16.53K/s |
| q7 | min and max of a date | 9.000ms | 60.441ms | 80.490ms | 24.9% | 60.620ms | 80.689ms | 60.460ms | 81.108ms | 60.000ms | 240.88 MiB | none | 12.42K/s |
| q8 | group by, low card | 7.000ms | 81.136ms | 80.618ms | 0.1% | 80.506ms | 80.622ms | 80.449ms | 80.698ms | 70.000ms | 244.26 MiB | none | 12.40K/s |
| q9 | group by and count distinct | 6.000ms | 60.426ms | 60.376ms | 0.3% | 60.374ms | 60.579ms | 60.369ms | 80.610ms | 60.000ms | 243.27 MiB | none | 16.56K/s |
| q10 | group by, several aggregates | 6.000ms | 80.660ms | 80.436ms | 24.8% | 60.500ms | 80.453ms | 60.476ms | 80.703ms | 70.000ms | 243.72 MiB | none | 12.43K/s |
| q11 | group by a string and count distinct | 6.000ms | 60.523ms | 60.522ms | 33.2% | 60.456ms | 80.560ms | 60.453ms | 80.590ms | 60.000ms | 244.85 MiB | none | 16.52K/s |
| q12 | group by two strings and count distinct | 6.000ms | 80.515ms | 60.450ms | 0.4% | 60.396ms | 60.611ms | 60.388ms | 60.942ms | 60.000ms | 244.76 MiB | none | 16.54K/s |
| q13 | group by a string and top k | 6.000ms | 60.378ms | 80.508ms | 26.5% | 60.490ms | 81.857ms | 60.483ms | 82.349ms | 60.000ms | 243.52 MiB | none | 12.42K/s |
| q14 | group by a string and count distinct | 6.000ms | 80.568ms | 80.525ms | 24.9% | 60.508ms | 80.555ms | 60.378ms | 80.639ms | 90.000ms | 244.76 MiB | none | 12.42K/s |
| q15 | group by two columns and top k | 6.000ms | 80.534ms | 60.402ms | 33.3% | 60.375ms | 80.491ms | 60.371ms | 80.900ms | 60.000ms | 244.73 MiB | none | 16.56K/s |
| q16 | group by, very high card | 6.000ms | 80.513ms | 60.412ms | 0.1% | 60.384ms | 60.423ms | 60.380ms | 80.465ms | 60.000ms | 243.51 MiB | none | 16.55K/s |
| q17 | group by two, very high card | 6.000ms | 80.571ms | 60.582ms | 0.5% | 60.400ms | 60.688ms | 60.396ms | 61.302ms | 60.000ms | 244.01 MiB | none | 16.51K/s |
| q18 | group by two, no ordering | 6.000ms | 60.543ms | 80.465ms | 24.6% | 60.644ms | 80.470ms | 60.393ms | 80.486ms | 60.000ms | 244.22 MiB | none | 12.43K/s |
| q19 | group by with an extract | 6.000ms | 81.288ms | 80.475ms | 0.2% | 80.464ms | 80.589ms | 60.370ms | 80.703ms | 70.000ms | 244.53 MiB | none | 12.43K/s |
| q20 | point lookup | 10.000ms | 80.632ms | 80.613ms | 0.0% | 80.577ms | 80.616ms | 60.670ms | 80.807ms | 60.000ms | 241.48 MiB | none | 12.40K/s |
| q21 | substring scan | 6.000ms | 80.536ms | 60.751ms | 33.1% | 60.497ms | 80.621ms | 60.383ms | 81.105ms | 60.000ms | 241.78 MiB | none | 16.46K/s |
| q22 | substring scan and group by | 6.000ms | 80.459ms | 80.457ms | 0.2% | 80.447ms | 80.632ms | 60.372ms | 80.682ms | 60.000ms | 243.50 MiB | none | 12.43K/s |
| q23 | two substring scans and group by | 7.000ms | 60.373ms | 60.583ms | 0.3% | 60.399ms | 60.602ms | 60.386ms | 80.489ms | 70.000ms | 243.36 MiB | none | 16.51K/s |
| q24 | select star and top k | 13.000ms | 80.558ms | 80.707ms | 0.1% | 80.682ms | 80.797ms | 80.476ms | 81.238ms | 70.000ms | 243.35 MiB | none | 12.39K/s |
| q25 | top k by a date | 10.000ms | 80.604ms | 80.547ms | 0.0% | 80.543ms | 80.556ms | 60.499ms | 80.617ms | 60.000ms | 242.98 MiB | none | 12.42K/s |
| q26 | top k by a string | 5.000ms | 60.381ms | 60.559ms | 33.2% | 60.549ms | 80.665ms | 60.384ms | 88.711ms | 60.000ms | 241.59 MiB | none | 16.51K/s |
| q27 | top k by two columns | 10.000ms | 80.570ms | 80.542ms | 0.2% | 80.541ms | 80.687ms | 80.497ms | 80.952ms | 80.000ms | 243.16 MiB | none | 12.42K/s |
| q28 | group by with a string length | 6.000ms | 60.388ms | 60.932ms | 32.9% | 60.496ms | 80.519ms | 60.442ms | 80.533ms | 60.000ms | 245.30 MiB | none | 16.41K/s |
| q29 | group by a regular expression | 7.000ms | 60.551ms | 80.458ms | 24.9% | 60.486ms | 80.491ms | 60.379ms | 80.597ms | 60.000ms | 245.35 MiB | none | 12.43K/s |
| q30 | ninety sums over one column | 9.000ms | 65.297ms | 80.459ms | 0.1% | 80.442ms | 80.502ms | 60.391ms | 80.907ms | 70.000ms | 245.67 MiB | none | 12.43K/s |
| q31 | group by two and several aggregates | 6.000ms | 80.495ms | 80.510ms | 0.3% | 80.477ms | 80.715ms | 60.395ms | 80.765ms | 70.000ms | 244.57 MiB | none | 12.42K/s |
| q32 | group by a high card pair | 6.000ms | 80.434ms | 60.479ms | 0.6% | 60.442ms | 60.792ms | 60.433ms | 80.440ms | 60.000ms | 244.51 MiB | none | 16.53K/s |
| q33 | group by a high card pair, unfiltered | 6.000ms | 63.656ms | 60.404ms | 0.2% | 60.395ms | 60.522ms | 60.389ms | 60.583ms | 60.000ms | 243.47 MiB | none | 16.56K/s |
| q34 | group by a long string | 6.000ms | 60.398ms | 60.484ms | 33.0% | 60.471ms | 80.436ms | 60.382ms | 81.110ms | 60.000ms | 243.76 MiB | none | 16.53K/s |
| q35 | group by a constant and a long string | 6.000ms | 60.437ms | 80.451ms | 24.0% | 61.265ms | 80.541ms | 60.654ms | 80.569ms | 60.000ms | 243.97 MiB | none | 12.43K/s |
| q36 | group by four expressions | 6.000ms | 80.466ms | 60.390ms | 0.5% | 60.388ms | 60.709ms | 60.369ms | 80.587ms | 60.000ms | 244.36 MiB | none | 16.56K/s |
| q37 | date range and group by a URL | 12.000ms | 81.129ms | 80.594ms | 0.1% | 80.535ms | 80.605ms | 80.482ms | 80.974ms | 70.000ms | 247.26 MiB | none | 12.41K/s |
| q38 | date range and group by a title | 12.000ms | 80.460ms | 80.844ms | 0.7% | 80.440ms | 80.999ms | 80.438ms | 81.242ms | 60.000ms | 247.78 MiB | none | 12.37K/s |
| q39 | date range, group by and offset | 11.000ms | 80.987ms | 80.981ms | 0.9% | 80.475ms | 81.224ms | 80.457ms | 81.334ms | 70.000ms | 245.79 MiB | none | 12.35K/s |
| q40 | date range, a case and a wide group by | 11.000ms | 80.444ms | 80.501ms | 0.0% | 80.481ms | 80.518ms | 80.440ms | 80.659ms | 70.000ms | 246.54 MiB | none | 12.42K/s |
| q41 | date range with an IN and a hash | 12.000ms | 80.506ms | 80.509ms | 0.1% | 80.471ms | 80.554ms | 80.446ms | 80.824ms | 80.000ms | 246.86 MiB | none | 12.42K/s |
| q42 | date range and a deep offset | 12.000ms | 80.462ms | 80.536ms | 0.1% | 80.511ms | 80.597ms | 80.450ms | 80.736ms | 70.000ms | 245.79 MiB | none | 12.42K/s |
| q43 | minute buckets over a date range | 12.000ms | 80.523ms | 80.636ms | 0.5% | 80.495ms | 80.858ms | 80.450ms | 80.987ms | 70.000ms | 246.55 MiB | none | 12.40K/s |

clickhouse-local 26.9.1.1562 over 43 of 43 queries. Total 319.000ms by its own clock and 3.083s by ours, 3.112s cold, 2.830s of CPU, peak 247.78 MiB, 134.80K/s and 38.62 MiB/s.

Running it cost 867% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.34x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 20.227ms | 20.312ms | 0.5% | 20.245ms | 20.350ms | 20.238ms | 20.371ms | 10.000ms | 80.36 MiB | none | 49.23K/s |
| q2 | filtered count | 2.000ms | 20.384ms | 20.371ms | 0.7% | 20.237ms | 20.382ms | 20.228ms | 20.396ms | 10.000ms | 95.63 MiB | none | 49.09K/s |
| q3 | three aggregates | 1.000ms | 20.241ms | 20.224ms | 0.0% | 20.220ms | 20.224ms | 20.220ms | 20.233ms | 0.000us | 81.77 MiB | none | 49.45K/s |
| q4 | average | 1.000ms | 20.367ms | 20.365ms | 0.7% | 20.233ms | 20.367ms | 20.217ms | 20.368ms | 10.000ms | 80.49 MiB | none | 49.10K/s |
| q5 | count distinct, high card | 3.000ms | 20.288ms | 20.245ms | 0.6% | 20.244ms | 20.370ms | 20.234ms | 20.606ms | 10.000ms | 107.90 MiB | none | 49.39K/s |
| q6 | count distinct, strings | 3.000ms | 20.236ms | 20.381ms | 0.1% | 20.364ms | 20.388ms | 20.240ms | 20.663ms | 10.000ms | 112.82 MiB | none | 49.07K/s |
| q7 | min and max of a date | 1.000ms | 20.689ms | 20.224ms | 0.0% | 20.220ms | 20.226ms | 20.217ms | 20.230ms | 0.000us | 80.00 MiB | none | 49.45K/s |
| q8 | group by, low card | 4.000ms | 20.234ms | 20.374ms | 0.6% | 20.253ms | 20.374ms | 20.245ms | 20.380ms | 10.000ms | 98.29 MiB | none | 49.08K/s |
| q9 | group by and count distinct | 6.000ms | 20.250ms | 20.251ms | 0.1% | 20.249ms | 20.263ms | 20.246ms | 20.266ms | 20.000ms | 124.76 MiB | none | 49.38K/s |
| q10 | group by, several aggregates | 6.000ms | 20.262ms | 20.389ms | 0.7% | 20.249ms | 20.389ms | 20.246ms | 20.392ms | 10.000ms | 116.93 MiB | none | 49.05K/s |
| q11 | group by a string and count distinct | 6.000ms | 20.243ms | 20.396ms | 0.0% | 20.390ms | 20.399ms | 20.378ms | 21.427ms | 10.000ms | 128.67 MiB | none | 49.03K/s |
| q12 | group by two strings and count distinct | 7.000ms | 20.273ms | 20.271ms | 0.0% | 20.269ms | 20.276ms | 20.257ms | 20.324ms | 20.000ms | 142.75 MiB | none | 49.33K/s |
| q13 | group by a string and top k | 5.000ms | 20.270ms | 20.257ms | 0.0% | 20.256ms | 20.265ms | 20.256ms | 20.270ms | 20.000ms | 128.21 MiB | none | 49.37K/s |
| q14 | group by a string and count distinct | 7.000ms | 20.254ms | 20.382ms | 0.7% | 20.257ms | 20.407ms | 20.257ms | 20.554ms | 20.000ms | 159.01 MiB | none | 49.06K/s |
| q15 | group by two columns and top k | 5.000ms | 21.607ms | 20.389ms | 0.6% | 20.266ms | 20.393ms | 20.265ms | 20.403ms | 10.000ms | 127.20 MiB | none | 49.04K/s |
| q16 | group by, very high card | 3.000ms | 20.386ms | 20.246ms | 0.0% | 20.243ms | 20.247ms | 20.229ms | 20.414ms | 10.000ms | 104.32 MiB | none | 49.39K/s |
| q17 | group by two, very high card | 4.000ms | 20.242ms | 20.260ms | 0.0% | 20.255ms | 20.264ms | 20.251ms | 20.312ms | 10.000ms | 115.21 MiB | none | 49.36K/s |
| q18 | group by two, no ordering | 3.000ms | 20.387ms | 20.379ms | 0.6% | 20.249ms | 20.381ms | 20.239ms | 20.400ms | 10.000ms | 106.47 MiB | none | 49.07K/s |
| q19 | group by with an extract | 4.000ms | 20.240ms | 20.380ms | 0.0% | 20.377ms | 20.382ms | 20.377ms | 20.516ms | 10.000ms | 117.30 MiB | none | 49.07K/s |
| q20 | point lookup | 2.000ms | 20.369ms | 20.231ms | 0.0% | 20.230ms | 20.237ms | 20.229ms | 20.371ms | 0.000us | 87.86 MiB | none | 49.43K/s |
| q21 | substring scan | 3.000ms | 20.248ms | 20.265ms | 0.1% | 20.248ms | 20.272ms | 20.242ms | 20.378ms | 10.000ms | 100.39 MiB | none | 49.35K/s |
| q22 | substring scan and group by | 4.000ms | 20.248ms | 20.296ms | 0.7% | 20.251ms | 20.391ms | 20.245ms | 20.400ms | 10.000ms | 113.70 MiB | none | 49.27K/s |
| q23 | two substring scans and group by | 5.000ms | 20.258ms | 20.257ms | 0.0% | 20.248ms | 20.257ms | 20.247ms | 20.262ms | 10.000ms | 116.26 MiB | none | 49.37K/s |
| q24 | select star and top k | 7.000ms | 20.256ms | 20.361ms | 0.6% | 20.270ms | 20.389ms | 20.263ms | 20.639ms | 10.000ms | 114.93 MiB | none | 49.11K/s |
| q25 | top k by a date | 3.000ms | 20.314ms | 20.248ms | 0.0% | 20.246ms | 20.253ms | 20.245ms | 20.255ms | 10.000ms | 102.24 MiB | none | 49.39K/s |
| q26 | top k by a string | 3.000ms | 20.234ms | 20.242ms | 0.0% | 20.241ms | 20.243ms | 20.236ms | 20.371ms | 10.000ms | 101.71 MiB | none | 49.40K/s |
| q27 | top k by two columns | 3.000ms | 20.245ms | 20.377ms | 0.8% | 20.324ms | 20.478ms | 20.249ms | 20.529ms | 10.000ms | 100.74 MiB | none | 49.08K/s |
| q28 | group by with a string length | 5.000ms | 20.286ms | 20.257ms | 0.6% | 20.253ms | 20.384ms | 20.246ms | 20.400ms | 10.000ms | 120.84 MiB | none | 49.37K/s |
| q29 | group by a regular expression | 7.000ms | 20.382ms | 20.269ms | 0.2% | 20.266ms | 20.305ms | 20.264ms | 20.400ms | 20.000ms | 139.67 MiB | none | 49.34K/s |
| q30 | ninety sums over one column | 10.000ms | 40.374ms | 20.253ms | 99.1% | 20.252ms | 40.327ms | 20.234ms | 40.430ms | 10.000ms | 88.72 MiB | none | 49.37K/s |
| q31 | group by two and several aggregates | 5.000ms | 20.365ms | 20.393ms | 0.7% | 20.268ms | 20.415ms | 20.250ms | 20.496ms | 10.000ms | 128.44 MiB | none | 49.04K/s |
| q32 | group by a high card pair | 6.000ms | 20.240ms | 20.252ms | 0.7% | 20.250ms | 20.383ms | 20.249ms | 20.390ms | 10.000ms | 127.82 MiB | none | 49.38K/s |
| q33 | group by a high card pair, unfiltered | 4.000ms | 20.382ms | 20.255ms | 0.0% | 20.253ms | 20.255ms | 20.247ms | 20.585ms | 10.000ms | 117.14 MiB | none | 49.37K/s |
| q34 | group by a long string | 4.000ms | 20.365ms | 20.267ms | 0.7% | 20.253ms | 20.386ms | 20.249ms | 20.394ms | 10.000ms | 119.30 MiB | none | 49.34K/s |
| q35 | group by a constant and a long string | 4.000ms | 20.395ms | 20.258ms | 0.1% | 20.252ms | 20.282ms | 20.248ms | 20.336ms | 10.000ms | 118.95 MiB | none | 49.36K/s |
| q36 | group by four expressions | 3.000ms | 20.250ms | 20.246ms | 0.0% | 20.241ms | 20.247ms | 20.236ms | 20.256ms | 10.000ms | 105.41 MiB | none | 49.39K/s |
| q37 | date range and group by a URL | 6.000ms | 20.374ms | 20.382ms | 0.6% | 20.262ms | 20.389ms | 20.259ms | 20.409ms | 20.000ms | 125.91 MiB | none | 49.06K/s |
| q38 | date range and group by a title | 5.000ms | 20.251ms | 20.263ms | 0.0% | 20.263ms | 20.267ms | 20.254ms | 20.394ms | 10.000ms | 121.16 MiB | none | 49.35K/s |
| q39 | date range, group by and offset | 5.000ms | 20.391ms | 20.257ms | 0.0% | 20.249ms | 20.257ms | 20.248ms | 20.316ms | 10.000ms | 116.61 MiB | none | 49.37K/s |
| q40 | date range, a case and a wide group by | 6.000ms | 20.274ms | 20.253ms | 0.0% | 20.251ms | 20.257ms | 20.248ms | 20.286ms | 10.000ms | 124.08 MiB | none | 49.38K/s |
| q41 | date range with an IN and a hash | 5.000ms | 20.263ms | 20.377ms | 0.6% | 20.254ms | 20.382ms | 20.250ms | 20.384ms | 10.000ms | 111.28 MiB | none | 49.08K/s |
| q42 | date range and a deep offset | 5.000ms | 20.250ms | 20.253ms | 0.2% | 20.249ms | 20.295ms | 20.247ms | 20.301ms | 10.000ms | 110.92 MiB | none | 49.38K/s |
| q43 | minute buckets over a date range | 5.000ms | 20.254ms | 20.256ms | 0.0% | 20.254ms | 20.256ms | 20.245ms | 20.261ms | 10.000ms | 113.59 MiB | none | 49.37K/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 187.000ms by its own clock and 872.859ms by ours, 894.350ms cold, 460.000ms of CPU, peak 159.01 MiB, 229.95K/s and 65.89 MiB/s.

Running it cost 367% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.01x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 5.627ms | 100.512ms | 100.599ms | 0.2% | 100.551ms | 100.725ms | 100.534ms | 100.849ms | 110.000ms | 61.48 MiB | none | 9.94K/s |
| q2 | filtered count | 6.559ms | 100.527ms | 100.622ms | 0.1% | 100.598ms | 100.689ms | 100.573ms | 100.703ms | 180.000ms | 65.84 MiB | none | 9.94K/s |
| q3 | three aggregates | 6.107ms | 100.678ms | 100.606ms | 0.1% | 100.567ms | 100.685ms | 100.567ms | 100.744ms | 110.000ms | 65.07 MiB | none | 9.94K/s |
| q4 | average | 4.977ms | 100.572ms | 100.752ms | 0.2% | 100.614ms | 100.842ms | 100.572ms | 103.827ms | 210.000ms | 63.54 MiB | none | 9.93K/s |
| q5 | count distinct, high card | 12.118ms | 100.608ms | 100.734ms | 0.1% | 100.695ms | 100.788ms | 100.524ms | 100.982ms | 110.000ms | 72.80 MiB | none | 9.93K/s |
| q6 | count distinct, strings | 11.218ms | 100.567ms | 100.633ms | 0.2% | 100.607ms | 100.838ms | 100.585ms | 127.680ms | 170.000ms | 72.97 MiB | none | 9.94K/s |
| q7 | min and max of a date | 6.196ms | 100.494ms | 100.689ms | 0.1% | 100.621ms | 100.765ms | 100.577ms | 101.303ms | 90.000ms | 63.82 MiB | none | 9.93K/s |
| q8 | group by, low card | 12.549ms | 100.528ms | 100.818ms | 1.3% | 100.633ms | 101.968ms | 100.525ms | 120.682ms | 120.000ms | 74.34 MiB | none | 9.92K/s |
| q9 | group by and count distinct | 22.953ms | 120.902ms | 121.628ms | 0.7% | 120.819ms | 121.715ms | 120.626ms | 122.150ms | 140.000ms | 80.41 MiB | none | 8.22K/s |
| q10 | group by, several aggregates | 26.951ms | 121.174ms | 120.905ms | 0.3% | 120.632ms | 120.977ms | 120.630ms | 141.118ms | 160.000ms | 82.64 MiB | none | 8.27K/s |
| q11 | group by a string and count distinct | 22.142ms | 121.963ms | 121.242ms | 2.1% | 120.813ms | 123.408ms | 120.773ms | 126.336ms | 150.000ms | 80.47 MiB | none | 8.25K/s |
| q12 | group by two strings and count distinct | 22.030ms | 121.724ms | 121.630ms | 0.6% | 121.592ms | 122.337ms | 120.991ms | 123.062ms | 160.000ms | 82.36 MiB | none | 8.22K/s |
| q13 | group by a string and top k | 16.390ms | 100.602ms | 100.689ms | 0.2% | 100.607ms | 100.793ms | 100.579ms | 120.881ms | 160.000ms | 74.88 MiB | none | 9.93K/s |
| q14 | group by a string and count distinct | 22.349ms | 120.673ms | 120.967ms | 0.1% | 120.921ms | 120.984ms | 120.878ms | 123.867ms | 150.000ms | 82.07 MiB | none | 8.27K/s |
| q15 | group by two columns and top k | 15.986ms | 100.579ms | 100.865ms | 21.9% | 100.581ms | 122.698ms | 100.530ms | 122.914ms | 130.000ms | 76.46 MiB | none | 9.91K/s |
| q16 | group by, very high card | 19.274ms | 100.818ms | 120.957ms | 1.1% | 120.747ms | 122.020ms | 120.640ms | 125.126ms | 140.000ms | 73.06 MiB | none | 8.27K/s |
| q17 | group by two, very high card | 22.344ms | 120.875ms | 120.758ms | 0.3% | 120.728ms | 121.104ms | 100.570ms | 121.573ms | 150.000ms | 74.90 MiB | none | 8.28K/s |
| q18 | group by two, no ordering | 11.345ms | 100.724ms | 100.767ms | 20.0% | 100.570ms | 120.760ms | 100.530ms | 120.771ms | 110.000ms | 72.65 MiB | none | 9.92K/s |
| q19 | group by with an extract | 24.231ms | 120.820ms | 120.795ms | 0.1% | 120.668ms | 120.829ms | 120.663ms | 122.486ms | 140.000ms | 76.28 MiB | none | 8.28K/s |
| q20 | point lookup | 5.354ms | 100.868ms | 100.936ms | 0.4% | 100.625ms | 101.046ms | 80.462ms | 103.724ms | 120.000ms | 63.78 MiB | none | 9.91K/s |
| q21 | substring scan | 6.695ms | 100.749ms | 100.669ms | 0.1% | 100.573ms | 100.682ms | 100.564ms | 100.919ms | 80.000ms | 67.33 MiB | none | 9.93K/s |
| q22 | substring scan and group by | 15.171ms | 100.518ms | 100.672ms | 0.2% | 100.567ms | 100.800ms | 100.553ms | 121.024ms | 130.000ms | 76.66 MiB | none | 9.93K/s |
| q23 | two substring scans and group by | 22.079ms | 120.703ms | 122.149ms | 0.7% | 121.478ms | 122.297ms | 120.827ms | 124.217ms | 140.000ms | 83.24 MiB | none | 8.19K/s |
| q24 | select star and top k | 9.430ms | 101.229ms | 100.679ms | 0.2% | 100.580ms | 100.772ms | 100.563ms | 100.779ms | 120.000ms | 74.01 MiB | none | 9.93K/s |
| q25 | top k by a date | 12.672ms | 100.670ms | 100.568ms | 0.1% | 100.515ms | 100.591ms | 100.496ms | 120.636ms | 120.000ms | 72.41 MiB | none | 9.94K/s |
| q26 | top k by a string | 10.822ms | 100.622ms | 100.566ms | 0.2% | 100.536ms | 100.725ms | 100.527ms | 100.749ms | 100.000ms | 70.80 MiB | none | 9.94K/s |
| q27 | top k by two columns | 12.580ms | 100.693ms | 100.693ms | 0.1% | 100.664ms | 100.781ms | 100.598ms | 100.783ms | 150.000ms | 71.10 MiB | none | 9.93K/s |
| q30 | ninety sums over one column | 12.044ms | 100.653ms | 100.731ms | 0.1% | 100.676ms | 100.761ms | 100.669ms | 100.768ms | 150.000ms | 67.41 MiB | none | 9.93K/s |
| q31 | group by two and several aggregates | 15.712ms | 100.569ms | 100.676ms | 20.0% | 100.662ms | 120.800ms | 100.602ms | 121.561ms | 140.000ms | 78.21 MiB | none | 9.93K/s |
| q32 | group by a high card pair | 16.904ms | 100.755ms | 100.670ms | 0.1% | 100.592ms | 100.690ms | 100.525ms | 101.006ms | 140.000ms | 77.40 MiB | none | 9.93K/s |
| q33 | group by a high card pair, unfiltered | 16.531ms | 101.107ms | 100.638ms | 0.1% | 100.627ms | 100.688ms | 100.595ms | 100.836ms | 120.000ms | 76.41 MiB | none | 9.94K/s |
| q34 | group by a long string | 17.097ms | 101.171ms | 100.637ms | 20.0% | 100.627ms | 120.761ms | 100.530ms | 120.937ms | 140.000ms | 74.10 MiB | none | 9.94K/s |
| q35 | group by a constant and a long string | 17.090ms | 100.522ms | 100.645ms | 0.1% | 100.579ms | 100.666ms | 100.563ms | 100.805ms | 120.000ms | 74.52 MiB | none | 9.94K/s |
| q37 | date range and group by a URL | 16.357ms | 100.569ms | 100.913ms | 19.9% | 100.801ms | 120.926ms | 100.616ms | 122.619ms | 120.000ms | 79.29 MiB | none | 9.91K/s |
| q38 | date range and group by a title | 15.476ms | 120.856ms | 100.833ms | 19.8% | 100.701ms | 120.670ms | 100.689ms | 120.857ms | 130.000ms | 80.07 MiB | none | 9.92K/s |
| q39 | date range, group by and offset | 13.529ms | 100.662ms | 100.581ms | 0.3% | 100.524ms | 100.823ms | 100.512ms | 100.891ms | 120.000ms | 76.68 MiB | none | 9.94K/s |
| q40 | date range, a case and a wide group by | 14.172ms | 100.610ms | 100.639ms | 0.1% | 100.590ms | 100.672ms | 100.571ms | 100.709ms | 110.000ms | 77.93 MiB | none | 9.94K/s |
| q41 | date range with an IN and a hash | 13.646ms | 120.689ms | 100.606ms | 0.0% | 100.602ms | 100.620ms | 100.518ms | 100.690ms | 120.000ms | 79.93 MiB | none | 9.94K/s |
| q42 | date range and a deep offset | 13.027ms | 100.629ms | 100.664ms | 0.1% | 100.583ms | 100.682ms | 100.524ms | 100.685ms | 110.000ms | 78.11 MiB | none | 9.93K/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 567.734ms by its own clock and 4.112s by ours, 4.130s cold, 5.170s of CPU, peak 83.24 MiB, 68.69K/s and 19.68 MiB/s.

Running it cost 624% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.21x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 480.672us | 20.274ms | 20.411ms | 0.3% | 20.364ms | 20.430ms | 20.249ms | 20.619ms | 0.000us | 10.35 MiB | none | 48.99K/s |
| q2 | filtered count | 492.156us | 20.467ms | 20.430ms | 0.1% | 20.418ms | 20.433ms | 20.413ms | 20.459ms | 0.000us | 10.60 MiB | none | 48.95K/s |
| q3 | three aggregates | 483.874us | 20.587ms | 20.580ms | 0.2% | 20.573ms | 20.619ms | 20.432ms | 21.612ms | 0.000us | 10.47 MiB | none | 48.59K/s |
| q4 | average | 485.343us | 20.443ms | 20.561ms | 0.4% | 20.491ms | 20.565ms | 20.411ms | 22.380ms | 0.000us | 10.45 MiB | none | 48.64K/s |
| q5 | count distinct, high card | 488.483us | 20.426ms | 20.428ms | 0.6% | 20.416ms | 20.540ms | 20.410ms | 20.553ms | 0.000us | 10.21 MiB | none | 48.95K/s |
| q6 | count distinct, strings | 562.545us | 20.445ms | 20.653ms | 1.9% | 20.442ms | 20.841ms | 20.311ms | 22.503ms | 0.000us | 10.27 MiB | none | 48.42K/s |
| q7 | min and max of a date | 510.220us | 20.435ms | 20.550ms | 0.6% | 20.437ms | 20.559ms | 20.418ms | 20.565ms | 0.000us | 10.67 MiB | none | 48.66K/s |
| q8 | group by, low card | 509.939us | 20.593ms | 20.445ms | 0.3% | 20.416ms | 20.474ms | 20.414ms | 22.447ms | 0.000us | 10.66 MiB | none | 48.91K/s |
| q9 | group by and count distinct | 567.942us | 20.437ms | 20.486ms | 0.4% | 20.482ms | 20.571ms | 20.460ms | 20.587ms | 0.000us | 10.27 MiB | none | 48.81K/s |
| q10 | group by, several aggregates | 809.265us | 20.578ms | 20.439ms | 0.2% | 20.430ms | 20.460ms | 20.419ms | 20.570ms | 0.000us | 10.93 MiB | none | 48.93K/s |
| q11 | group by a string and count distinct | 576.152us | 20.442ms | 20.568ms | 0.0% | 20.559ms | 20.569ms | 20.423ms | 20.604ms | 0.000us | 10.71 MiB | none | 48.62K/s |
| q12 | group by two strings and count distinct | 584.404us | 20.443ms | 20.480ms | 0.5% | 20.476ms | 20.579ms | 20.423ms | 20.601ms | 0.000us | 10.70 MiB | none | 48.83K/s |
| q13 | group by a string and top k | 562.923us | 20.602ms | 20.453ms | 0.7% | 20.425ms | 20.568ms | 20.419ms | 20.571ms | 0.000us | 10.64 MiB | none | 48.89K/s |
| q14 | group by a string and count distinct | 599.180us | 20.656ms | 20.647ms | 0.2% | 20.627ms | 20.674ms | 20.614ms | 20.689ms | 0.000us | 10.70 MiB | none | 48.43K/s |
| q15 | group by two columns and top k | 664.785us | 20.673ms | 20.648ms | 0.1% | 20.624ms | 20.654ms | 20.596ms | 20.777ms | 0.000us | 10.70 MiB | none | 48.43K/s |
| q16 | group by, very high card | 547.637us | 20.660ms | 20.632ms | 0.2% | 20.601ms | 20.644ms | 20.589ms | 20.698ms | 0.000us | 10.25 MiB | none | 48.47K/s |
| q17 | group by two, very high card | 609.995us | 20.653ms | 20.645ms | 0.0% | 20.642ms | 20.650ms | 20.638ms | 20.712ms | 0.000us | 10.27 MiB | none | 48.44K/s |
| q18 | group by two, no ordering | 514.308us | 20.503ms | 20.632ms | 0.4% | 20.567ms | 20.645ms | 20.531ms | 20.671ms | 0.000us | 10.02 MiB | none | 48.47K/s |
| q19 | group by with an extract | 749.098us | 20.610ms | 20.447ms | 0.7% | 20.337ms | 20.475ms | 20.312ms | 20.575ms | 0.000us | 10.70 MiB | none | 48.91K/s |
| q20 | point lookup | 417.569us | 20.246ms | 20.386ms | 0.8% | 20.232ms | 20.403ms | 20.231ms | 20.441ms | 0.000us | 10.41 MiB | none | 49.05K/s |
| q21 | substring scan | 640.095us | 20.441ms | 20.427ms | 0.5% | 20.345ms | 20.444ms | 20.237ms | 20.640ms | 0.000us | 10.49 MiB | none | 48.95K/s |
| q22 | substring scan and group by | 724.903us | 20.241ms | 20.404ms | 1.0% | 20.397ms | 20.600ms | 20.251ms | 20.615ms | 0.000us | 10.70 MiB | none | 49.01K/s |
| q23 | two substring scans and group by | 967.949us | 20.626ms | 20.577ms | 0.4% | 20.503ms | 20.579ms | 20.427ms | 20.588ms | 0.000us | 10.73 MiB | none | 48.60K/s |
| q24 | select star and top k | 798.771us | 20.600ms | 20.521ms | 0.3% | 20.462ms | 20.522ms | 20.449ms | 22.816ms | 0.000us | 10.96 MiB | none | 48.73K/s |
| q25 | top k by a date | 482.298us | 20.445ms | 20.578ms | 0.1% | 20.562ms | 20.580ms | 20.432ms | 20.584ms | 0.000us | 10.72 MiB | none | 48.60K/s |
| q26 | top k by a string | 482.722us | 20.469ms | 20.247ms | 0.1% | 20.238ms | 20.263ms | 20.236ms | 20.394ms | 0.000us | 10.45 MiB | none | 49.39K/s |
| q27 | top k by two columns | 516.309us | 20.387ms | 20.463ms | 0.6% | 20.434ms | 20.561ms | 20.253ms | 20.598ms | 0.000us | 10.69 MiB | none | 48.87K/s |
| q28 | group by with a string length | 732.565us | 20.300ms | 20.464ms | 0.2% | 20.428ms | 20.469ms | 20.340ms | 20.522ms | 0.000us | 11.01 MiB | none | 48.87K/s |
| q29 | group by a regular expression | 800.654us | 20.420ms | 20.256ms | 0.8% | 20.248ms | 20.420ms | 20.195ms | 20.435ms | 0.000us | 10.96 MiB | none | 49.37K/s |
| q30 | ninety sums over one column | 1.771ms | 20.238ms | 20.594ms | 0.1% | 20.585ms | 20.599ms | 20.430ms | 20.627ms | 0.000us | 10.81 MiB | none | 48.56K/s |
| q31 | group by two and several aggregates | 789.107us | 20.526ms | 20.569ms | 0.4% | 20.500ms | 20.578ms | 20.459ms | 20.591ms | 0.000us | 10.90 MiB | none | 48.62K/s |
| q32 | group by a high card pair | 732.208us | 20.607ms | 20.389ms | 0.2% | 20.389ms | 20.435ms | 20.238ms | 20.440ms | 0.000us | 10.71 MiB | none | 49.05K/s |
| q33 | group by a high card pair, unfiltered | 771.485us | 20.600ms | 20.446ms | 0.1% | 20.440ms | 20.457ms | 20.431ms | 20.583ms | 0.000us | 10.68 MiB | none | 48.91K/s |
| q34 | group by a long string | 731.945us | 20.459ms | 20.476ms | 0.7% | 20.459ms | 20.598ms | 20.459ms | 21.028ms | 0.000us | 10.49 MiB | none | 48.84K/s |
| q35 | group by a constant and a long string | 770.761us | 20.509ms | 20.443ms | 0.0% | 20.442ms | 20.443ms | 20.438ms | 20.447ms | 0.000us | 10.65 MiB | none | 48.92K/s |
| q36 | group by four expressions | 621.533us | 20.480ms | 20.459ms | 0.5% | 20.457ms | 20.566ms | 20.443ms | 20.605ms | 0.000us | 10.52 MiB | none | 48.88K/s |
| q37 | date range and group by a URL | 743.786us | 20.489ms | 20.467ms | 0.1% | 20.464ms | 20.485ms | 20.437ms | 20.517ms | 0.000us | 11.16 MiB | none | 48.86K/s |
| q38 | date range and group by a title | 853.130us | 20.467ms | 20.575ms | 0.7% | 20.450ms | 20.587ms | 20.446ms | 20.589ms | 0.000us | 11.20 MiB | none | 48.60K/s |
| q39 | date range, group by and offset | 770.621us | 20.510ms | 20.500ms | 0.5% | 20.489ms | 20.583ms | 20.440ms | 20.590ms | 0.000us | 11.17 MiB | none | 48.78K/s |
| q40 | date range, a case and a wide group by | 895.871us | 20.505ms | 20.410ms | 1.1% | 20.264ms | 20.484ms | 20.244ms | 20.513ms | 0.000us | 11.23 MiB | none | 49.00K/s |
| q41 | date range with an IN and a hash | 645.386us | 20.284ms | 20.460ms | 0.2% | 20.424ms | 20.464ms | 20.252ms | 20.480ms | 0.000us | 11.00 MiB | none | 48.88K/s |
| q42 | date range and a deep offset | 730.745us | 20.395ms | 20.310ms | 0.9% | 20.276ms | 20.450ms | 20.235ms | 20.503ms | 0.000us | 10.99 MiB | none | 49.24K/s |
| q43 | minute buckets over a date range | 626.474us | 20.282ms | 20.388ms | 0.2% | 20.386ms | 20.429ms | 20.242ms | 20.605ms | 0.000us | 11.18 MiB | none | 49.05K/s |

rudb rudb 0.3.67 over 43 of 43 queries. Total 28.817ms by its own clock and 880.945ms by ours, 880.453ms cold, 0.000us of CPU, peak 11.23 MiB, 1.49M/s and 427.55 MiB/s.

Running it cost 2957% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.02x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | 375.370us | 71.175us | 62.851us | 71.177us | 11.7% | 62.851us | 135.486us | 0.000us | 360 B | 4 of 4 |
| q2 | 313.008us | 66.676us | 59.123us | 66.685us | 11.3% | 59.123us | 98.638us | 0.000us | 360 B | 5 of 5 |
| q3 | 372.966us | 109.718us | 94.645us | 105.200us | 10.0% | 94.645us | 98.454us | 0.000us | 992 B | 4 of 4 |
| q4 | 349.210us | 48.254us | 39.577us | 48.254us | 18.0% | 39.577us | 94.466us | 0.000us | 360 B | 4 of 4 |
| q5 | 350.225us | 63.428us | 56.424us | 63.422us | 11.0% | 56.424us | 98.554us | 0.000us | 12.01 KiB | 4 of 4 |
| q6 | 326.754us | 156.307us | 147.282us | 156.296us | 5.8% | 147.282us | 102.325us | 0.000us | 39.97 KiB | 5 of 5 |
| q7 | 327.652us | 64.762us | 56.751us | 64.770us | 12.4% | 56.751us | 99.361us | 0.000us | 616 B | 4 of 4 |
| q8 | 378.260us | 66.780us | 60.095us | 66.789us | 10.0% | 60.095us | 111.706us | 0.000us | 1.17 KiB | 6 of 6 |
| q9 | 372.734us | 113.530us | 106.687us | 113.529us | 6.0% | 106.687us | 99.481us | 0.000us | 57.72 KiB | 5 of 5 |
| q10 | 357.929us | 356.121us | 345.479us | 356.120us | 3.0% | 345.479us | 89.897us | 0.000us | 238.00 KiB | 5 of 5 |
| q11 | 345.862us | 74.440us | 67.019us | 74.428us | 10.0% | 67.019us | 105.322us | 0.000us | 1.84 KiB | 6 of 6 |
| q12 | 403.559us | 92.334us | 83.118us | 92.339us | 10.0% | 83.118us | 167.071us | 0.000us | 4.96 KiB | 6 of 6 |
| q13 | 377.352us | 111.234us | 104.053us | 111.241us | 6.5% | 104.053us | 94.618us | 0.000us | 22.00 KiB | 6 of 6 |
| q14 | 476.100us | 108.389us | 99.537us | 108.394us | 8.2% | 99.537us | 128.635us | 0.000us | 38.19 KiB | 6 of 6 |
| q15 | 384.286us | 109.025us | 100.815us | 109.028us | 7.5% | 100.815us | 104.587us | 0.000us | 22.69 KiB | 6 of 6 |
| q16 | 363.001us | 106.236us | 99.940us | 106.221us | 5.9% | 99.940us | 100.143us | 0.000us | 41.28 KiB | 5 of 5 |
| q17 | 372.331us | 215.951us | 208.224us | 215.961us | 3.6% | 208.224us | 92.429us | 0.000us | 56.78 KiB | 5 of 5 |
| q18 | 413.685us | 133.934us | 125.072us | 133.946us | 6.6% | 125.072us | 128.165us | 0.000us | 2.33 KiB | 5 of 5 |
| q19 | 400.462us | 197.633us | 189.933us | 197.626us | 3.9% | 189.933us | 153.466us | 0.000us | 65.16 KiB | 5 of 5 |
| q20 | 301.845us | 59.140us | 51.899us | 59.138us | 12.2% | 51.899us | 87.629us | 0.000us | 0 B | 4 of 4 |
| q21 | 359.366us | 205.864us | 199.118us | 205.861us | 3.3% | 199.118us | 108.966us | 0.000us | 360 B | 5 of 5 |
| q22 | 354.767us | 202.726us | 196.305us | 202.713us | 3.2% | 196.305us | 113.315us | 0.000us | 512 B | 6 of 6 |
| q23 | 389.608us | 486.179us | 478.648us | 486.175us | 1.5% | 478.648us | 113.977us | 0.000us | 512 B | 6 of 6 |
| q24 | 493.053us | 175.508us | 170.172us | 175.494us | 3.0% | 170.172us | 152.820us | 0.000us | 0 B | 8 of 8 |
| q25 | 360.933us | 87.485us | 80.714us | 87.490us | 7.7% | 80.714us | 83.866us | 0.000us | 4.91 KiB | 6 of 6 |
| q26 | 305.732us | 78.531us | 72.094us | 78.533us | 8.2% | 72.094us | 78.816us | 0.000us | 4.81 KiB | 5 of 5 |
| q27 | 328.972us | 98.026us | 90.700us | 98.018us | 7.5% | 90.700us | 89.466us | 0.000us | 6.21 KiB | 6 of 6 |
| q28 | 369.586us | 266.527us | 260.998us | 266.509us | 2.1% | 260.998us | 112.203us | 0.000us | 23.03 KiB | 7 of 7 |
| q29 | 784.679us | 342.561us | 335.401us | 342.550us | 2.1% | 335.401us | 131.354us | 0.000us | 71.06 KiB | 7 of 7 |
| q30 | 1.540ms | 77.413us | 67.994us | 77.430us | 12.2% | 67.994us | 127.208us | 0.000us | 29.59 KiB | 4 of 4 |
| q31 | 373.878us | 230.942us | 219.330us | 230.952us | 5.0% | 219.330us | 103.954us | 0.000us | 53.05 KiB | 6 of 6 |
| q32 | 369.835us | 208.544us | 198.789us | 208.537us | 4.7% | 198.789us | 105.199us | 0.000us | 53.59 KiB | 6 of 6 |
| q33 | 347.628us | 510.747us | 500.985us | 510.741us | 1.9% | 500.985us | 100.845us | 0.000us | 108.56 KiB | 5 of 5 |
| q34 | 336.633us | 344.825us | 338.197us | 344.834us | 1.9% | 338.197us | 102.414us | 0.000us | 173.91 KiB | 5 of 5 |
| q35 | 325.468us | 277.520us | 270.464us | 277.515us | 2.5% | 270.464us | 97.309us | 0.000us | 174.09 KiB | 5 of 5 |
| q36 | 380.202us | 111.936us | 103.849us | 111.932us | 7.2% | 103.849us | 120.163us | 0.000us | 37.22 KiB | 6 of 6 |
| q37 | 381.124us | 309.598us | 298.810us | 309.593us | 3.5% | 298.810us | 110.065us | 0.000us | 8.44 KiB | 6 of 6 |
| q38 | 421.744us | 364.786us | 355.185us | 364.779us | 2.6% | 355.185us | 163.502us | 0.000us | 3.51 KiB | 6 of 6 |
| q39 | 376.450us | 226.052us | 216.332us | 226.058us | 4.3% | 216.332us | 113.625us | 0.000us | 512 B | 6 of 6 |
| q40 | 429.903us | 396.047us | 383.130us | 396.042us | 3.3% | 383.130us | 116.765us | 0.000us | 10.79 KiB | 6 of 6 |
| q41 | 451.532us | 147.277us | 128.207us | 147.276us | 12.9% | 128.207us | 138.728us | 0.000us | 1.33 KiB | 6 of 6 |
| q42 | 439.995us | 117.320us | 106.797us | 117.310us | 9.0% | 106.797us | 124.210us | 0.000us | 1.10 KiB | 6 of 6 |
| q43 | 473.695us | 136.893us | 126.534us | 136.895us | 7.6% | 126.534us | 169.316us | 0.000us | 2.53 KiB | 6 of 6 |

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
| Aggregate | 0.000us | nothing to share | 39 | 19785 | 1448 | 0.0ns | 0.0ns | 39 of 39 |
| Fetch | 0.000us | nothing to share | 1 | 0 | 0 | handed none | handed on none | 1 of 1 |
| FileScan | 0.000us | nothing to share | 43 | 0 | 43000 | handed none | 0.0ns | 43 of 43 |
| Filter | 0.000us | nothing to share | 28 | 26000 | 3061 | 0.0ns | 0.0ns | 28 of 28 |
| Limit | 0.000us | nothing to share | 1 | 10 | 10 | 0.0ns | 0.0ns | 1 of 1 |
| Project | 0.000us | nothing to share | 91 | 21815 | 21815 | 0.0ns | 0.0ns | 91 of 91 |
| Sort | 0.000us | nothing to share | 1 | 2 | 2 | 0.0ns | 0.0ns | 1 of 1 |
| TopN | 0.000us | nothing to share | 31 | 1703 | 197 | 0.0ns | 0.0ns | 31 of 31 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is Aggregate, and the queries where it cost the most are q1 at 0.000us, q2 at 0.000us, q3 at 0.000us.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 1000 rows, one out of every 99998 of the 99997497 in the full file, which is a development loop rather than the suite
- every query here ran within 1.01x of every other one, so most of what was timed is whatever they have in common rather than the queries

These swung wider than reporting rule two allows:

- clickhouse-local swung by 33.3% of its median on q15, and rule two wants under 10%
- datafusion swung by 99.1% of its median on q30, and rule two wants under 10%
- polars swung by 21.9% of its median on q15, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

These ran every query at about the same speed:

- duckdb ran every query within 1.01x of every other one, and this suite spreads over 6x on an engine it is measuring
- duckdb-pinned ran every query within 1.01x of every other one, and this suite spreads over 6x on an engine it is measuring
- clickhouse-local ran every query within 1.34x of every other one, and this suite spreads over 6x on an engine it is measuring
- datafusion ran every query within 1.01x of every other one, and this suite spreads over 6x on an engine it is measuring
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

Answers differ, so this is not a comparison: q14: duckdb-pinned does not agree with duckdb: the same 10 numbers and 9 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q14: clickhouse-local does not agree with duckdb: the same 10 numbers and 9 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q14: datafusion does not agree with duckdb: the same 10 numbers and 8 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q14: polars does not agree with duckdb: the same 10 numbers and 8 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q14: rudb does not agree with duckdb: the same 10 numbers and 9 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q15: clickhouse-local does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q15: datafusion does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q15: polars does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q15: rudb does not agree with duckdb: 20 numbers against 20

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

