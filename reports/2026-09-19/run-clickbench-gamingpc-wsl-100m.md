# clickbench on gamingpc-wsl

This is one run of the clickbench suite on gamingpc-wsl, over 6 engines and 43 queries, with 5 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

## What ran

| what | it was |
| --- | --- |
| suite | clickbench |
| queries | 43 |
| tables | 13.76 GiB of Parquet in 1 table |
| rows | 99997497 in the table every query reads |
| corpus | no manifest beside the data, so this run cannot say where it came from |
| summary | median with the interquartile range, per reporting rule two |
| timeout | 600s per query, and no query reached it |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --engines duckdb,duckdb-pinned,clickhouse-local,datafusion,polars,rudb --runs 5 --timeout 600 --report
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
| filesystem | /dev/sdd / ext4 rw,relatime,discard,errors=remount-ro,data=ordered 0 0 | read |
| page cache | cannot open drop_caches: Permission denied (os error 13) | not read |
| timer | /usr/bin/time GNU | read |

A fact that says it was not read is a fact about this machine and not a gap in the report. A field that had silently defaulted would be a lie that survived into it.

## The engines

| engine | version | state | load | load cpu | on disk | that size is | format | machine load |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 42.612s | 573.090s | 24.98 GiB | its own database file | its own | 1.51 to 27.66 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 38.626s | 568.160s | 19.05 GiB | its own database file | its own | 27.66 to 28.19 |
| clickhouse-local | 26.9.1.1562 | ran | 26.467s | 509.380s | 10.42 GiB | its own MergeTree parts, as system.parts counts the active ones | its own | 28.19 to 14.92 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 13.76 GiB | the source Parquet, this engine has no storage format of its own | the Parquet | 14.92 to 23.61 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 13.76 GiB | the source Parquet, this engine has no storage format of its own | the Parquet | 23.61 to 17.37 |
| rudb | rudb 0.3.45 | ran | 0.000us | 0.000us | 13.76 GiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 17.37 to 11.44 |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

These engines started while the machine was already above half of that, which means they were measured against somebody else's work rather than on an idle box: duckdb-pinned, clickhouse-local, polars, rudb. Their numbers are inflated by an amount nothing here can recover, and by a different amount each, depending on how much of the column was CPU bound. One case where that reading is unfair is a sweep that runs engines back to back, where the average has not had a minute to come down from the engine before.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb on 39 shared |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 24.116s | 26.790s | +11% | 27.719s | 316.630s | 11.82 | 10.81 GiB | none | 178.30M/s | 24.54 GiB/s | 1.00x |
| duckdb-pinned | 15.416s | 19.136s | +24% | 19.952s | 280.320s | 14.65 | 9.56 GiB | none | 278.92M/s | 38.39 GiB/s | 0.80x |
| clickhouse-local | 16.396s | 26.703s | +63% | 27.091s | 366.320s | 13.72 | 7.53 GiB | 29.16 MiB | 262.25M/s | 36.10 GiB/s | 0.95x |
| datafusion | 23.356s | 25.055s | +7% | 25.848s | 563.710s | 22.50 | 10.90 GiB | 30.97 MiB | 184.10M/s | 25.34 GiB/s | 1.34x |
| polars | 21.432s | 27.665s | +29% | 28.322s | 506.770s | 18.32 | 16.68 GiB | 84.00 KiB | 181.97M/s | 25.05 GiB/s | 1.37x |
| rudb | 31.361s | 32.658s | +4% | 32.870s | 373.900s | 11.45 | 5.69 GiB | none | 137.11M/s | 18.87 GiB/s | 1.78x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 9.000ms | 10.000ms | 24.000ms | 1.000ms | 13.709ms | 38.350ms |
| q2 | filtered count | 22.000ms | 29.000ms | 33.000ms | 19.000ms | 16.986ms | 58.430ms |
| q3 | three aggregates | 37.000ms | 40.000ms | 82.000ms | 42.000ms | 42.345ms | 77.091ms |
| q4 | average | 43.000ms | 45.000ms | 92.000ms | 50.000ms | 65.666ms | 66.456ms |
| q5 | count distinct, high card | 199.000ms | 188.000ms | 213.000ms | 322.000ms | 280.349ms | 421.776ms |
| q6 | count distinct, strings | 347.000ms | 252.000ms | 298.000ms | 363.000ms | 382.860ms | 1.292s |
| q7 | min and max of a date | 17.000ms | 16.000ms | 169.000ms | 1.000ms | 31.475ms | 75.224ms |
| q8 | group by, low card | 23.000ms | 23.000ms | 164.000ms | 20.000ms | 25.309ms | 63.461ms |
| q9 | group by and count distinct | 258.000ms | 230.000ms | 415.000ms | 399.000ms | 686.685ms | 515.923ms |
| q10 | group by, several aggregates | 350.000ms | 356.000ms | 439.000ms | 419.000ms | 722.493ms | 741.048ms |
| q11 | group by a string and count distinct | 125.000ms | 97.000ms | 185.000ms | 102.000ms | 90.706ms | 302.935ms |
| q12 | group by two strings and count distinct | 136.000ms | 107.000ms | 209.000ms | 113.000ms | 96.577ms | 345.054ms |
| q13 | group by a string and top k | 342.000ms | 289.000ms | 337.000ms | 412.000ms | 330.616ms | 666.523ms |
| q14 | group by a string and count distinct | 617.000ms | 543.000ms | 436.000ms | 616.000ms | 759.430ms | 1.318s |
| q15 | group by two columns and top k | 375.000ms | 310.000ms | 365.000ms | 421.000ms | 405.022ms | 936.879ms |
| q16 | group by, very high card | 232.000ms | 218.000ms | 225.000ms | 383.000ms | 373.037ms | 802.444ms |
| q17 | group by two, very high card | 774.000ms | 700.000ms | 794.000ms | 822.000ms | 1.178s | 1.821s |
| q18 | group by two, no ordering | 667.000ms | 581.000ms | 367.000ms | 823.000ms | 1.115s | 426.113ms |
| q19 | group by with an extract | 1.428s | 1.332s | 1.340s | 1.576s | 1.856s | 2.919s |
| q20 | point lookup | 31.000ms | 35.000ms | 175.000ms | 46.000ms | 30.221ms | 168.213ms |
| q21 | substring scan | 711.000ms | 362.000ms | 395.000ms | 567.000ms | 917.314ms | 623.762ms |
| q22 | substring scan and group by | 794.000ms | 510.000ms | 491.000ms | 616.000ms | 958.891ms | 869.253ms |
| q23 | two substring scans and group by | 1.008s | 646.000ms | 643.000ms | 1.407s | 1.639s | 2.271s |
| q24 | select star and top k | 410.000ms | 120.000ms | 301.000ms | 4.305s | 1.167s | 987.283ms |
| q25 | top k by a date | 54.000ms | 59.000ms | 197.000ms | 111.000ms | 170.285ms | 483.724ms |
| q26 | top k by a string | 108.000ms | 108.000ms | 170.000ms | 174.000ms | 137.995ms | 273.267ms |
| q27 | top k by two columns | 55.000ms | 61.000ms | 201.000ms | 117.000ms | 240.901ms | 484.086ms |
| q28 | group by with a string length | 747.000ms | 477.000ms | 132.000ms | 577.000ms | no dialect | 820.873ms |
| q29 | group by a regular expression | 7.520s | 2.243s | 1.137s | 1.532s | no dialect | 2.077s |
| q30 | ninety sums over one column | 32.000ms | 49.000ms | 80.000ms | 42.000ms | 112.709ms | 71.335ms |
| q31 | group by two and several aggregates | 359.000ms | 294.000ms | 251.000ms | 389.000ms | 344.402ms | 591.052ms |
| q32 | group by a high card pair | 588.000ms | 438.000ms | 309.000ms | 482.000ms | 410.682ms | 746.865ms |
| q33 | group by a high card pair, unfiltered | 1.330s | 1.300s | 1.425s | 1.523s | 2.064s | 1.603s |
| q34 | group by a long string | 1.965s | 1.497s | 1.388s | 1.849s | 1.798s | 2.462s |
| q35 | group by a constant and a long string | 1.968s | 1.474s | 1.363s | 1.853s | 2.587s | 2.456s |
| q36 | group by four expressions | 239.000ms | 178.000ms | 181.000ms | 327.000ms | no dialect | 667.922ms |
| q37 | date range and group by a URL | 36.000ms | 32.000ms | 219.000ms | 112.000ms | 87.805ms | 130.362ms |
| q38 | date range and group by a title | 20.000ms | 22.000ms | 199.000ms | 73.000ms | 61.785ms | 98.142ms |
| q39 | date range, group by and offset | 24.000ms | 28.000ms | 187.000ms | 79.000ms | 44.374ms | 76.446ms |
| q40 | date range, a case and a wide group by | 58.000ms | 57.000ms | 231.000ms | 214.000ms | 132.119ms | 331.810ms |
| q41 | date range with an IN and a hash | 19.000ms | 19.000ms | 179.000ms | 22.000ms | 30.457ms | 72.738ms |
| q42 | date range and a deep offset | 19.000ms | 21.000ms | 191.000ms | 19.000ms | 23.997ms | 55.356ms |
| q43 | minute buckets over a date range | 20.000ms | 20.000ms | 164.000ms | 16.000ms | no dialect | 51.453ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 9.000ms | 40.534ms | 40.372ms | 0.1% | 40.361ms | 40.387ms | 40.333ms | 40.440ms | 20.000ms | 35.57 MiB | none | 2.48G/s |
| q2 | filtered count | 22.000ms | 60.723ms | 40.350ms | 0.0% | 40.350ms | 40.359ms | 40.349ms | 40.517ms | 120.000ms | 121.57 MiB | 53.00 MiB | 2.48G/s |
| q3 | three aggregates | 37.000ms | 80.528ms | 60.478ms | 0.0% | 60.471ms | 60.492ms | 60.432ms | 81.324ms | 380.000ms | 240.32 MiB | 106.23 MiB | 1.65G/s |
| q4 | average | 43.000ms | 80.574ms | 60.466ms | 33.2% | 60.463ms | 80.521ms | 60.446ms | 80.598ms | 570.000ms | 318.47 MiB | 246.00 MiB | 1.65G/s |
| q5 | count distinct, high card | 199.000ms | 246.303ms | 243.030ms | 7.9% | 225.048ms | 244.143ms | 224.976ms | 246.882ms | 4.650s | 937.02 MiB | none | 411.46M/s |
| q6 | count distinct, strings | 347.000ms | 390.613ms | 402.233ms | 3.6% | 389.109ms | 403.499ms | 388.472ms | 416.601ms | 5.720s | 1.60 GiB | 430.76 MiB | 248.61M/s |
| q7 | min and max of a date | 17.000ms | 40.349ms | 40.341ms | 0.0% | 40.340ms | 40.345ms | 40.334ms | 40.354ms | 20.000ms | 52.83 MiB | none | 2.48G/s |
| q8 | group by, low card | 23.000ms | 40.354ms | 40.372ms | 0.2% | 40.363ms | 40.438ms | 40.345ms | 40.468ms | 130.000ms | 123.34 MiB | none | 2.48G/s |
| q9 | group by and count distinct | 258.000ms | 305.191ms | 304.804ms | 3.6% | 296.097ms | 307.061ms | 287.612ms | 308.053ms | 5.880s | 1.15 GiB | 134.50 MiB | 328.07M/s |
| q10 | group by, several aggregates | 350.000ms | 408.378ms | 393.711ms | 4.3% | 388.523ms | 405.352ms | 388.221ms | 420.376ms | 7.690s | 1.35 GiB | none | 253.99M/s |
| q11 | group by a string and count distinct | 125.000ms | 140.835ms | 161.137ms | 11.8% | 142.138ms | 161.192ms | 140.871ms | 161.352ms | 1.500s | 640.08 MiB | 83.50 MiB | 620.58M/s |
| q12 | group by two strings and count distinct | 136.000ms | 161.253ms | 163.679ms | 24.5% | 140.895ms | 180.989ms | 140.831ms | 201.047ms | 1.640s | 702.33 MiB | 51.00 MiB | 610.94M/s |
| q13 | group by a string and top k | 342.000ms | 382.454ms | 403.974ms | 0.5% | 403.349ms | 405.429ms | 385.757ms | 406.387ms | 5.340s | 1.67 GiB | none | 247.53M/s |
| q14 | group by a string and count distinct | 617.000ms | 672.185ms | 688.038ms | 3.7% | 687.610ms | 712.845ms | 655.251ms | 713.711ms | 10.530s | 2.85 GiB | none | 145.34M/s |
| q15 | group by two columns and top k | 375.000ms | 427.012ms | 431.153ms | 5.5% | 428.037ms | 451.773ms | 425.191ms | 463.425ms | 5.950s | 1.84 GiB | 83.12 MiB | 231.93M/s |
| q16 | group by, very high card | 232.000ms | 271.525ms | 282.569ms | 5.6% | 268.736ms | 284.458ms | 267.943ms | 284.742ms | 5.500s | 1.08 GiB | none | 353.89M/s |
| q17 | group by two, very high card | 774.000ms | 852.548ms | 867.720ms | 1.9% | 853.280ms | 869.887ms | 852.153ms | 870.119ms | 13.630s | 3.24 GiB | none | 115.24M/s |
| q18 | group by two, no ordering | 667.000ms | 720.364ms | 757.463ms | 8.6% | 729.894ms | 795.218ms | 711.100ms | 808.576ms | 9.780s | 3.18 GiB | none | 132.02M/s |
| q19 | group by with an extract | 1.428s | 1.640s | 1.582s | 0.4% | 1.580s | 1.587s | 1.548s | 1.622s | 24.850s | 6.10 GiB | 501.20 MiB | 63.22M/s |
| q20 | point lookup | 31.000ms | 60.647ms | 60.451ms | 0.1% | 60.442ms | 60.496ms | 60.437ms | 60.520ms | 190.000ms | 235.81 MiB | none | 1.65G/s |
| q21 | substring scan | 711.000ms | 1.032s | 805.210ms | 2.4% | 786.898ms | 805.928ms | 784.770ms | 828.349ms | 8.870s | 4.66 GiB | 5.39 GiB | 124.19M/s |
| q22 | substring scan and group by | 794.000ms | 916.007ms | 887.732ms | 2.4% | 887.537ms | 908.693ms | 885.560ms | 927.584ms | 9.150s | 5.14 GiB | none | 112.64M/s |
| q23 | two substring scans and group by | 1.008s | 1.320s | 1.131s | 3.9% | 1.087s | 1.132s | 1.033s | 1.207s | 12.070s | 6.70 GiB | 3.78 GiB | 88.42M/s |
| q24 | select star and top k | 410.000ms | 523.596ms | 504.108ms | 0.2% | 503.017ms | 504.114ms | 502.859ms | 505.320ms | 3.710s | 729.29 MiB | 134.51 MiB | 198.37M/s |
| q25 | top k by a date | 54.000ms | 80.579ms | 80.948ms | 1.0% | 80.781ms | 81.620ms | 80.667ms | 82.207ms | 290.000ms | 184.28 MiB | none | 1.24G/s |
| q26 | top k by a string | 108.000ms | 141.563ms | 140.846ms | 15.0% | 121.402ms | 142.575ms | 120.713ms | 142.908ms | 1.770s | 549.76 MiB | none | 709.98M/s |
| q27 | top k by two columns | 55.000ms | 100.779ms | 80.710ms | 0.4% | 80.529ms | 80.827ms | 80.514ms | 100.675ms | 310.000ms | 191.32 MiB | none | 1.24G/s |
| q28 | group by with a string length | 747.000ms | 810.917ms | 824.813ms | 4.1% | 811.445ms | 845.300ms | 805.680ms | 845.775ms | 7.600s | 4.72 GiB | 26.75 MiB | 121.24M/s |
| q29 | group by a regular expression | 7.520s | 8.061s | 7.679s | 0.3% | 7.677s | 7.703s | 7.674s | 7.835s | 56.670s | 5.18 GiB | 3.77 GiB | 13.02M/s |
| q30 | ninety sums over one column | 32.000ms | 60.570ms | 60.457ms | 0.1% | 60.453ms | 60.507ms | 60.451ms | 60.650ms | 310.000ms | 178.91 MiB | none | 1.65G/s |
| q31 | group by two and several aggregates | 359.000ms | 388.138ms | 403.766ms | 5.2% | 403.384ms | 424.320ms | 387.690ms | 427.592ms | 5.000s | 1.74 GiB | 176.10 MiB | 247.66M/s |
| q32 | group by a high card pair | 588.000ms | 644.801ms | 655.972ms | 3.5% | 645.990ms | 669.058ms | 645.461ms | 670.002ms | 7.310s | 2.99 GiB | 694.89 MiB | 152.44M/s |
| q33 | group by a high card pair, unfiltered | 1.330s | 1.575s | 1.525s | 0.6% | 1.518s | 1.527s | 1.500s | 1.580s | 30.990s | 9.51 GiB | 1.50 MiB | 65.56M/s |
| q34 | group by a long string | 1.965s | 2.169s | 2.184s | 0.9% | 2.169s | 2.189s | 2.145s | 2.196s | 30.150s | 10.56 GiB | none | 45.79M/s |
| q35 | group by a constant and a long string | 1.968s | 2.246s | 2.172s | 1.4% | 2.169s | 2.199s | 2.156s | 2.227s | 31.350s | 10.81 GiB | none | 46.04M/s |
| q36 | group by four expressions | 239.000ms | 284.635ms | 287.250ms | 1.3% | 284.611ms | 288.468ms | 267.032ms | 296.605ms | 6.040s | 1.48 GiB | none | 348.12M/s |
| q37 | date range and group by a URL | 36.000ms | 60.455ms | 60.459ms | 0.1% | 60.435ms | 60.470ms | 60.430ms | 60.523ms | 200.000ms | 215.34 MiB | 2.65 MiB | 1.65G/s |
| q38 | date range and group by a title | 20.000ms | 40.402ms | 40.373ms | 0.1% | 40.361ms | 40.409ms | 40.357ms | 40.500ms | 70.000ms | 86.83 MiB | none | 2.48G/s |
| q39 | date range, group by and offset | 24.000ms | 40.368ms | 40.365ms | 49.8% | 40.352ms | 60.437ms | 40.343ms | 60.438ms | 100.000ms | 118.09 MiB | 960.00 KiB | 2.48G/s |
| q40 | date range, a case and a wide group by | 58.000ms | 80.820ms | 80.618ms | 0.3% | 80.590ms | 80.867ms | 80.582ms | 81.194ms | 370.000ms | 402.09 MiB | 768.00 KiB | 1.24G/s |
| q41 | date range with an IN and a hash | 19.000ms | 40.381ms | 40.370ms | 0.0% | 40.368ms | 40.377ms | 40.362ms | 40.393ms | 70.000ms | 85.66 MiB | 15.63 MiB | 2.48G/s |
| q42 | date range and a deep offset | 19.000ms | 40.361ms | 40.358ms | 0.1% | 40.355ms | 40.395ms | 40.351ms | 40.400ms | 60.000ms | 76.78 MiB | 1.91 MiB | 2.48G/s |
| q43 | minute buckets over a date range | 20.000ms | 40.355ms | 40.356ms | 0.0% | 40.352ms | 40.356ms | 40.349ms | 40.384ms | 80.000ms | 70.45 MiB | none | 2.48G/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 24.116s by its own clock and 26.790s by ours, 27.719s cold, 316.630s of CPU, peak 10.81 GiB, 178.30M/s and 24.54 GiB/s.

Running it cost 11% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 190.36x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 10.000ms | 60.578ms | 40.369ms | 50.0% | 40.365ms | 60.547ms | 40.362ms | 60.637ms | 30.000ms | 46.63 MiB | 68.00 KiB | 2.48G/s |
| q2 | filtered count | 29.000ms | 80.532ms | 60.432ms | 0.0% | 60.429ms | 60.438ms | 60.421ms | 60.545ms | 110.000ms | 131.38 MiB | 59.86 MiB | 1.65G/s |
| q3 | three aggregates | 40.000ms | 80.705ms | 80.949ms | 0.3% | 80.799ms | 81.080ms | 80.725ms | 81.508ms | 420.000ms | 249.81 MiB | 116.25 MiB | 1.24G/s |
| q4 | average | 45.000ms | 100.652ms | 80.888ms | 0.2% | 80.793ms | 80.984ms | 80.578ms | 100.791ms | 590.000ms | 328.15 MiB | 253.25 MiB | 1.24G/s |
| q5 | count distinct, high card | 188.000ms | 244.484ms | 243.433ms | 7.2% | 226.263ms | 243.831ms | 225.672ms | 244.243ms | 4.210s | 949.22 MiB | none | 410.78M/s |
| q6 | count distinct, strings | 252.000ms | 344.871ms | 311.249ms | 5.4% | 307.366ms | 324.108ms | 304.338ms | 324.843ms | 4.370s | 1.65 GiB | 428.49 MiB | 321.28M/s |
| q7 | min and max of a date | 16.000ms | 40.352ms | 60.420ms | 33.2% | 40.358ms | 60.420ms | 40.343ms | 60.455ms | 30.000ms | 62.76 MiB | none | 1.66G/s |
| q8 | group by, low card | 23.000ms | 60.590ms | 60.442ms | 0.0% | 60.441ms | 60.468ms | 60.434ms | 60.473ms | 150.000ms | 134.33 MiB | none | 1.65G/s |
| q9 | group by and count distinct | 230.000ms | 286.862ms | 286.849ms | 1.0% | 285.188ms | 287.970ms | 284.118ms | 289.155ms | 5.070s | 1.16 GiB | 143.77 MiB | 348.61M/s |
| q10 | group by, several aggregates | 356.000ms | 429.868ms | 412.853ms | 3.5% | 412.562ms | 427.192ms | 404.930ms | 427.200ms | 6.570s | 1.35 GiB | none | 242.21M/s |
| q11 | group by a string and count distinct | 97.000ms | 140.862ms | 141.749ms | 14.2% | 141.250ms | 161.405ms | 141.219ms | 161.932ms | 1.030s | 639.45 MiB | 86.50 MiB | 705.45M/s |
| q12 | group by two strings and count distinct | 107.000ms | 140.945ms | 161.312ms | 0.1% | 161.305ms | 161.456ms | 140.747ms | 161.858ms | 1.260s | 705.20 MiB | 56.25 MiB | 619.90M/s |
| q13 | group by a string and top k | 289.000ms | 363.955ms | 363.897ms | 6.5% | 348.047ms | 371.831ms | 344.033ms | 384.084ms | 5.000s | 1.72 GiB | none | 274.80M/s |
| q14 | group by a string and count distinct | 543.000ms | 712.623ms | 708.365ms | 3.4% | 688.088ms | 712.334ms | 671.780ms | 751.810ms | 9.290s | 2.87 GiB | none | 141.17M/s |
| q15 | group by two columns and top k | 310.000ms | 387.550ms | 382.921ms | 3.1% | 372.080ms | 383.993ms | 364.718ms | 387.783ms | 5.740s | 1.92 GiB | 85.49 MiB | 261.14M/s |
| q16 | group by, very high card | 218.000ms | 264.168ms | 271.848ms | 1.0% | 269.783ms | 272.400ms | 263.785ms | 286.708ms | 4.990s | 1.09 GiB | none | 367.84M/s |
| q17 | group by two, very high card | 700.000ms | 892.576ms | 861.017ms | 2.5% | 851.837ms | 873.404ms | 831.563ms | 889.436ms | 11.460s | 3.30 GiB | none | 116.14M/s |
| q18 | group by two, no ordering | 581.000ms | 731.314ms | 728.162ms | 0.6% | 724.571ms | 729.076ms | 724.200ms | 746.332ms | 7.610s | 3.25 GiB | none | 137.33M/s |
| q19 | group by with an extract | 1.332s | 1.577s | 1.573s | 1.9% | 1.550s | 1.580s | 1.533s | 1.593s | 20.980s | 6.12 GiB | 508.02 MiB | 63.58M/s |
| q20 | point lookup | 35.000ms | 80.558ms | 80.542ms | 0.1% | 80.509ms | 80.612ms | 80.508ms | 80.974ms | 190.000ms | 249.13 MiB | none | 1.24G/s |
| q21 | substring scan | 362.000ms | 684.171ms | 430.325ms | 4.4% | 426.259ms | 445.020ms | 425.892ms | 445.422ms | 7.510s | 2.99 GiB | 2.94 GiB | 232.38M/s |
| q22 | substring scan and group by | 510.000ms | 647.166ms | 645.012ms | 0.2% | 644.985ms | 646.542ms | 643.635ms | 665.516ms | 7.640s | 3.45 GiB | none | 155.03M/s |
| q23 | two substring scans and group by | 646.000ms | 865.429ms | 786.500ms | 2.4% | 785.706ms | 804.221ms | 783.933ms | 807.038ms | 8.490s | 3.78 GiB | 2.08 GiB | 127.14M/s |
| q24 | select star and top k | 120.000ms | 201.389ms | 161.873ms | 12.1% | 161.551ms | 181.109ms | 160.919ms | 181.297ms | 920.000ms | 459.60 MiB | 150.16 MiB | 617.75M/s |
| q25 | top k by a date | 59.000ms | 101.153ms | 100.802ms | 0.1% | 100.757ms | 100.814ms | 100.732ms | 100.864ms | 220.000ms | 223.10 MiB | none | 992.02M/s |
| q26 | top k by a string | 108.000ms | 141.115ms | 141.655ms | 14.0% | 141.591ms | 161.353ms | 141.097ms | 161.901ms | 1.190s | 592.38 MiB | none | 705.92M/s |
| q27 | top k by two columns | 61.000ms | 100.767ms | 100.901ms | 0.3% | 100.673ms | 101.019ms | 100.612ms | 101.638ms | 250.000ms | 227.09 MiB | none | 991.04M/s |
| q28 | group by with a string length | 477.000ms | 548.677ms | 549.775ms | 10.5% | 547.295ms | 604.842ms | 546.692ms | 627.431ms | 7.670s | 3.07 GiB | 29.75 MiB | 181.89M/s |
| q29 | group by a regular expression | 2.243s | 2.539s | 2.436s | 1.5% | 2.429s | 2.465s | 2.428s | 2.471s | 59.460s | 5.70 GiB | 2.92 GiB | 41.06M/s |
| q30 | ninety sums over one column | 49.000ms | 100.874ms | 80.717ms | 0.3% | 80.536ms | 80.770ms | 80.488ms | 80.928ms | 330.000ms | 191.38 MiB | none | 1.24G/s |
| q31 | group by two and several aggregates | 294.000ms | 362.431ms | 363.118ms | 0.5% | 362.465ms | 364.375ms | 349.567ms | 366.352ms | 4.790s | 1.81 GiB | 214.12 MiB | 275.39M/s |
| q32 | group by a high card pair | 438.000ms | 526.757ms | 525.629ms | 4.4% | 508.025ms | 530.914ms | 507.532ms | 545.541ms | 7.440s | 3.05 GiB | 975.40 MiB | 190.24M/s |
| q33 | group by a high card pair, unfiltered | 1.300s | 1.736s | 1.637s | 2.6% | 1.614s | 1.657s | 1.595s | 1.676s | 28.070s | 9.56 GiB | 3.00 MiB | 61.09M/s |
| q34 | group by a long string | 1.497s | 1.866s | 1.800s | 8.5% | 1.736s | 1.888s | 1.724s | 1.983s | 25.130s | 8.94 GiB | none | 55.54M/s |
| q35 | group by a constant and a long string | 1.474s | 1.796s | 1.777s | 0.7% | 1.769s | 1.781s | 1.767s | 1.825s | 27.040s | 9.19 GiB | none | 56.27M/s |
| q36 | group by four expressions | 178.000ms | 250.264ms | 226.052ms | 1.5% | 224.251ms | 227.713ms | 224.241ms | 242.837ms | 4.100s | 989.16 MiB | none | 442.36M/s |
| q37 | date range and group by a URL | 32.000ms | 60.514ms | 60.445ms | 0.0% | 60.442ms | 60.452ms | 60.438ms | 60.459ms | 200.000ms | 217.65 MiB | 768.00 KiB | 1.65G/s |
| q38 | date range and group by a title | 22.000ms | 60.447ms | 60.517ms | 0.1% | 60.459ms | 60.529ms | 60.448ms | 60.594ms | 80.000ms | 98.84 MiB | none | 1.65G/s |
| q39 | date range, group by and offset | 28.000ms | 60.454ms | 60.452ms | 0.0% | 60.440ms | 60.464ms | 60.434ms | 60.664ms | 110.000ms | 120.15 MiB | 1.00 MiB | 1.65G/s |
| q40 | date range, a case and a wide group by | 57.000ms | 100.619ms | 100.670ms | 0.6% | 100.621ms | 101.186ms | 100.609ms | 101.323ms | 350.000ms | 395.42 MiB | 512.00 KiB | 993.32M/s |
| q41 | date range with an IN and a hash | 19.000ms | 60.537ms | 60.454ms | 0.0% | 60.438ms | 60.464ms | 60.435ms | 60.555ms | 80.000ms | 94.89 MiB | 14.49 MiB | 1.65G/s |
| q42 | date range and a deep offset | 21.000ms | 60.449ms | 60.447ms | 0.0% | 60.446ms | 60.459ms | 60.432ms | 60.585ms | 80.000ms | 87.65 MiB | 2.24 MiB | 1.65G/s |
| q43 | minute buckets over a date range | 20.000ms | 60.446ms | 60.458ms | 0.0% | 60.447ms | 60.460ms | 60.445ms | 90.365ms | 70.000ms | 80.13 MiB | none | 1.65G/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 15.416s by its own clock and 19.136s by ours, 19.952s cold, 280.320s of CPU, peak 9.56 GiB, 278.92M/s and 38.39 GiB/s.

Running it cost 24% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 60.33x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 24.000ms | 286.206ms | 282.464ms | 0.5% | 281.539ms | 283.027ms | 281.533ms | 301.562ms | 1.230s | 377.64 MiB | 9.45 MiB | 354.02M/s |
| q2 | filtered count | 33.000ms | 281.657ms | 261.374ms | 0.0% | 261.324ms | 261.407ms | 261.223ms | 281.648ms | 1.240s | 296.57 MiB | 7.25 MiB | 382.58M/s |
| q3 | three aggregates | 82.000ms | 422.183ms | 321.599ms | 6.9% | 321.479ms | 343.695ms | 301.573ms | 347.373ms | 1.590s | 341.62 MiB | 25.63 MiB | 310.94M/s |
| q4 | average | 92.000ms | 341.641ms | 325.599ms | 6.3% | 321.532ms | 341.887ms | 302.016ms | 342.150ms | 1.620s | 421.61 MiB | 125.07 MiB | 307.12M/s |
| q5 | count distinct, high card | 213.000ms | 482.908ms | 466.336ms | 0.1% | 466.107ms | 466.370ms | 462.730ms | 502.708ms | 5.700s | 1.88 GiB | 45.43 MiB | 214.43M/s |
| q6 | count distinct, strings | 298.000ms | 523.699ms | 526.557ms | 0.6% | 524.515ms | 527.598ms | 522.503ms | 542.916ms | 6.920s | 1.37 GiB | 226.46 MiB | 189.91M/s |
| q7 | min and max of a date | 169.000ms | 361.749ms | 362.477ms | 0.7% | 362.106ms | 364.818ms | 341.664ms | 382.520ms | 1.620s | 439.21 MiB | 240.00 KiB | 275.87M/s |
| q8 | group by, low card | 164.000ms | 382.721ms | 362.497ms | 5.6% | 361.785ms | 382.163ms | 361.771ms | 403.466ms | 1.760s | 445.84 MiB | 10.16 MiB | 275.86M/s |
| q9 | group by and count distinct | 415.000ms | 648.937ms | 663.964ms | 2.7% | 647.799ms | 665.466ms | 647.166ms | 687.386ms | 9.370s | 1.65 GiB | 45.31 MiB | 150.61M/s |
| q10 | group by, several aggregates | 439.000ms | 703.858ms | 684.522ms | 3.0% | 684.054ms | 704.475ms | 667.342ms | 708.240ms | 10.030s | 1.67 GiB | none | 146.08M/s |
| q11 | group by a string and count distinct | 185.000ms | 402.126ms | 383.905ms | 4.9% | 383.197ms | 402.105ms | 362.092ms | 403.197ms | 2.570s | 682.38 MiB | 4.52 MiB | 260.47M/s |
| q12 | group by two strings and count distinct | 209.000ms | 401.989ms | 422.773ms | 0.3% | 422.424ms | 423.535ms | 362.014ms | 442.169ms | 2.870s | 728.32 MiB | 5.13 MiB | 236.53M/s |
| q13 | group by a string and top k | 337.000ms | 568.038ms | 604.614ms | 7.1% | 563.354ms | 606.113ms | 545.319ms | 647.237ms | 7.700s | 1.49 GiB | none | 165.39M/s |
| q14 | group by a string and count distinct | 436.000ms | 686.346ms | 705.291ms | 1.9% | 692.861ms | 706.179ms | 690.353ms | 727.300ms | 10.900s | 2.00 GiB | none | 141.78M/s |
| q15 | group by two columns and top k | 365.000ms | 604.728ms | 604.547ms | 0.3% | 603.649ms | 605.370ms | 603.444ms | 623.610ms | 8.330s | 1.67 GiB | 12.91 MiB | 165.41M/s |
| q16 | group by, very high card | 225.000ms | 462.608ms | 443.071ms | 0.5% | 442.867ms | 445.070ms | 442.238ms | 483.450ms | 5.290s | 1.21 GiB | none | 225.69M/s |
| q17 | group by two, very high card | 794.000ms | 1.092s | 1.070s | 0.3% | 1.069s | 1.072s | 1.040s | 1.073s | 20.240s | 2.68 GiB | 62.05 MiB | 93.50M/s |
| q18 | group by two, no ordering | 367.000ms | 584.203ms | 585.409ms | 4.0% | 563.811ms | 587.083ms | 549.869ms | 607.527ms | 8.940s | 686.77 MiB | 3.83 MiB | 170.82M/s |
| q19 | group by with an extract | 1.340s | 1.725s | 1.672s | 2.9% | 1.648s | 1.697s | 1.619s | 1.699s | 34.760s | 4.61 GiB | 213.05 MiB | 59.80M/s |
| q20 | point lookup | 175.000ms | 362.000ms | 382.973ms | 5.2% | 382.196ms | 401.965ms | 361.848ms | 402.428ms | 2.130s | 478.84 MiB | none | 261.11M/s |
| q21 | substring scan | 395.000ms | 666.207ms | 609.061ms | 0.6% | 605.868ms | 609.409ms | 603.395ms | 625.410ms | 9.870s | 690.22 MiB | 1.12 GiB | 164.18M/s |
| q22 | substring scan and group by | 491.000ms | 664.524ms | 705.462ms | 5.5% | 688.617ms | 727.208ms | 685.988ms | 748.436ms | 11.620s | 683.65 MiB | 600.00 KiB | 141.75M/s |
| q23 | two substring scans and group by | 643.000ms | 907.770ms | 866.994ms | 2.3% | 849.207ms | 869.242ms | 845.770ms | 873.869ms | 15.120s | 728.50 MiB | 544.39 MiB | 115.34M/s |
| q24 | select star and top k | 301.000ms | 563.546ms | 522.841ms | 0.0% | 522.811ms | 522.844ms | 502.579ms | 543.625ms | 3.850s | 655.31 MiB | 14.70 MiB | 191.26M/s |
| q25 | top k by a date | 197.000ms | 423.127ms | 422.780ms | 0.5% | 422.624ms | 424.924ms | 382.437ms | 462.407ms | 2.500s | 522.95 MiB | none | 236.52M/s |
| q26 | top k by a string | 170.000ms | 384.178ms | 402.166ms | 1.0% | 402.104ms | 406.321ms | 362.138ms | 425.556ms | 3.860s | 616.81 MiB | none | 248.65M/s |
| q27 | top k by two columns | 201.000ms | 402.770ms | 422.219ms | 4.7% | 402.393ms | 422.260ms | 401.981ms | 422.808ms | 2.390s | 515.55 MiB | none | 236.84M/s |
| q28 | group by with a string length | 132.000ms | 382.228ms | 344.233ms | 5.6% | 343.170ms | 362.600ms | 341.655ms | 365.665ms | 2.960s | 489.61 MiB | 992.00 KiB | 290.49M/s |
| q29 | group by a regular expression | 1.137s | 1.493s | 1.430s | 0.8% | 1.423s | 1.434s | 1.413s | 1.436s | 29.520s | 2.89 GiB | 896.17 MiB | 69.94M/s |
| q30 | ninety sums over one column | 80.000ms | 321.802ms | 321.735ms | 12.1% | 302.872ms | 341.852ms | 281.354ms | 342.124ms | 1.500s | 391.96 MiB | 16.00 KiB | 310.81M/s |
| q31 | group by two and several aggregates | 251.000ms | 465.749ms | 466.132ms | 4.1% | 463.818ms | 482.722ms | 443.743ms | 503.608ms | 4.930s | 1.03 GiB | 78.81 MiB | 214.53M/s |
| q32 | group by a high card pair | 309.000ms | 546.041ms | 543.404ms | 0.1% | 543.335ms | 544.132ms | 526.145ms | 563.263ms | 7.080s | 1.22 GiB | 34.32 MiB | 184.02M/s |
| q33 | group by a high card pair, unfiltered | 1.425s | 1.717s | 1.742s | 2.7% | 1.718s | 1.764s | 1.713s | 1.778s | 36.160s | 4.50 GiB | none | 57.41M/s |
| q34 | group by a long string | 1.388s | 1.832s | 1.754s | 3.2% | 1.736s | 1.793s | 1.712s | 1.815s | 35.800s | 7.53 GiB | none | 57.02M/s |
| q35 | group by a constant and a long string | 1.363s | 1.713s | 1.734s | 2.7% | 1.692s | 1.740s | 1.676s | 1.772s | 35.530s | 7.42 GiB | none | 57.66M/s |
| q36 | group by four expressions | 181.000ms | 426.313ms | 403.109ms | 4.9% | 402.300ms | 422.246ms | 402.126ms | 422.401ms | 4.570s | 1.04 GiB | 5.05 MiB | 248.07M/s |
| q37 | date range and group by a URL | 219.000ms | 403.216ms | 445.214ms | 4.2% | 443.859ms | 462.531ms | 442.384ms | 482.769ms | 2.370s | 596.34 MiB | 568.00 KiB | 224.61M/s |
| q38 | date range and group by a title | 199.000ms | 401.981ms | 404.623ms | 4.9% | 402.277ms | 422.140ms | 402.090ms | 422.762ms | 1.910s | 518.22 MiB | none | 247.14M/s |
| q39 | date range, group by and offset | 187.000ms | 382.250ms | 402.668ms | 5.1% | 401.990ms | 422.487ms | 362.838ms | 423.102ms | 1.930s | 511.24 MiB | 240.00 KiB | 248.34M/s |
| q40 | date range, a case and a wide group by | 231.000ms | 442.743ms | 442.693ms | 4.5% | 442.596ms | 462.428ms | 442.282ms | 462.512ms | 2.640s | 785.76 MiB | 1.02 MiB | 225.88M/s |
| q41 | date range with an IN and a hash | 179.000ms | 401.985ms | 402.023ms | 5.2% | 382.038ms | 402.810ms | 361.692ms | 422.190ms | 1.770s | 481.20 MiB | 25.73 MiB | 248.74M/s |
| q42 | date range and a deep offset | 191.000ms | 402.005ms | 401.993ms | 9.9% | 362.075ms | 402.004ms | 361.603ms | 422.426ms | 1.850s | 487.03 MiB | 7.16 MiB | 248.75M/s |
| q43 | minute buckets over a date range | 164.000ms | 422.501ms | 382.282ms | 0.5% | 382.028ms | 383.952ms | 362.380ms | 402.081ms | 1.780s | 482.97 MiB | 232.00 KiB | 261.58M/s |

clickhouse-local 26.9.1.1562 over 43 of 43 queries. Total 16.396s by its own clock and 26.703s by ours, 27.091s cold, 366.320s of CPU, peak 7.53 GiB, 262.25M/s and 36.10 GiB/s.

Running it cost 63% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 6.71x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 100.634ms | 40.384ms | 0.1% | 40.380ms | 40.403ms | 40.378ms | 40.406ms | 20.000ms | 101.37 MiB | 130.17 MiB | 2.48G/s |
| q2 | filtered count | 19.000ms | 61.176ms | 60.533ms | 30.8% | 41.878ms | 60.552ms | 40.442ms | 60.564ms | 290.000ms | 255.30 MiB | 1.44 MiB | 1.65G/s |
| q3 | three aggregates | 42.000ms | 80.725ms | 80.784ms | 0.3% | 80.643ms | 80.881ms | 80.580ms | 80.896ms | 690.000ms | 736.93 MiB | 52.71 MiB | 1.24G/s |
| q4 | average | 50.000ms | 105.379ms | 80.825ms | 0.3% | 80.698ms | 80.948ms | 80.666ms | 80.976ms | 840.000ms | 664.13 MiB | 207.84 MiB | 1.24G/s |
| q5 | count distinct, high card | 322.000ms | 363.853ms | 367.013ms | 0.4% | 366.844ms | 368.412ms | 365.045ms | 384.897ms | 7.710s | 2.25 GiB | none | 272.46M/s |
| q6 | count distinct, strings | 363.000ms | 433.541ms | 407.872ms | 1.3% | 403.961ms | 409.115ms | 390.159ms | 411.544ms | 7.460s | 2.41 GiB | 352.07 MiB | 245.17M/s |
| q7 | min and max of a date | 1.000ms | 20.458ms | 40.371ms | 49.7% | 20.309ms | 40.372ms | 20.301ms | 40.405ms | 10.000ms | 101.82 MiB | 4.00 KiB | 2.48G/s |
| q8 | group by, low card | 20.000ms | 61.543ms | 60.452ms | 29.6% | 42.710ms | 60.578ms | 41.662ms | 60.666ms | 320.000ms | 275.86 MiB | none | 1.65G/s |
| q9 | group by and count distinct | 399.000ms | 467.160ms | 439.738ms | 4.1% | 428.676ms | 446.657ms | 427.752ms | 452.849ms | 9.510s | 2.45 GiB | 44.32 MiB | 227.40M/s |
| q10 | group by, several aggregates | 419.000ms | 446.423ms | 463.544ms | 3.7% | 448.897ms | 466.236ms | 448.028ms | 466.498ms | 6.810s | 2.30 GiB | none | 215.72M/s |
| q11 | group by a string and count distinct | 102.000ms | 142.976ms | 142.416ms | 0.4% | 142.187ms | 142.750ms | 141.905ms | 144.166ms | 2.020s | 1.15 GiB | 4.13 MiB | 702.15M/s |
| q12 | group by two strings and count distinct | 113.000ms | 161.149ms | 144.046ms | 0.9% | 142.796ms | 144.081ms | 142.153ms | 145.041ms | 2.250s | 1.24 GiB | 4.55 MiB | 694.21M/s |
| q13 | group by a string and top k | 412.000ms | 463.265ms | 470.775ms | 7.1% | 454.939ms | 488.312ms | 438.690ms | 495.584ms | 8.890s | 2.53 GiB | none | 212.41M/s |
| q14 | group by a string and count distinct | 616.000ms | 651.738ms | 656.506ms | 1.7% | 651.433ms | 662.347ms | 649.186ms | 667.788ms | 14.410s | 3.36 GiB | none | 152.32M/s |
| q15 | group by two columns and top k | 421.000ms | 508.913ms | 465.510ms | 1.7% | 457.936ms | 465.789ms | 455.779ms | 467.722ms | 9.110s | 2.38 GiB | 13.96 MiB | 214.81M/s |
| q16 | group by, very high card | 383.000ms | 407.436ms | 429.165ms | 1.3% | 424.605ms | 430.044ms | 422.369ms | 430.696ms | 9.400s | 2.46 GiB | none | 233.00M/s |
| q17 | group by two, very high card | 822.000ms | 899.119ms | 870.175ms | 2.3% | 859.504ms | 879.465ms | 857.303ms | 893.186ms | 21.660s | 5.44 GiB | none | 114.92M/s |
| q18 | group by two, no ordering | 823.000ms | 922.094ms | 871.092ms | 2.0% | 856.408ms | 874.028ms | 852.977ms | 875.607ms | 21.490s | 5.47 GiB | none | 114.80M/s |
| q19 | group by with an extract | 1.576s | 1.712s | 1.627s | 3.8% | 1.609s | 1.671s | 1.596s | 1.708s | 42.740s | 8.20 GiB | 343.28 MiB | 61.47M/s |
| q20 | point lookup | 46.000ms | 80.606ms | 80.988ms | 0.7% | 80.692ms | 81.297ms | 80.608ms | 82.106ms | 760.000ms | 621.17 MiB | none | 1.23G/s |
| q21 | substring scan | 567.000ms | 735.199ms | 594.305ms | 1.0% | 589.658ms | 595.869ms | 572.929ms | 611.719ms | 11.720s | 1.51 GiB | 3.09 GiB | 168.26M/s |
| q22 | substring scan and group by | 616.000ms | 636.696ms | 652.033ms | 2.5% | 636.167ms | 652.595ms | 631.055ms | 662.437ms | 13.820s | 1.88 GiB | none | 153.36M/s |
| q23 | two substring scans and group by | 1.407s | 1.545s | 1.439s | 1.3% | 1.436s | 1.454s | 1.433s | 1.462s | 33.850s | 2.86 GiB | 2.12 GiB | 69.47M/s |
| q24 | select star and top k | 4.305s | 4.410s | 4.357s | 0.7% | 4.336s | 4.366s | 4.271s | 4.390s | 112.900s | 6.64 GiB | 4.09 GiB | 22.95M/s |
| q25 | top k by a date | 111.000ms | 161.600ms | 142.160ms | 1.2% | 141.201ms | 142.962ms | 141.185ms | 143.266ms | 1.560s | 879.75 MiB | none | 703.41M/s |
| q26 | top k by a string | 174.000ms | 204.321ms | 202.648ms | 0.5% | 202.058ms | 203.147ms | 201.593ms | 205.524ms | 2.840s | 1.01 GiB | none | 493.46M/s |
| q27 | top k by two columns | 117.000ms | 141.417ms | 161.309ms | 12.3% | 141.580ms | 161.370ms | 141.331ms | 161.404ms | 1.590s | 929.50 MiB | none | 619.91M/s |
| q28 | group by with a string length | 577.000ms | 592.766ms | 614.539ms | 3.6% | 613.275ms | 635.553ms | 611.389ms | 655.215ms | 12.500s | 1.75 GiB | 2.52 MiB | 162.72M/s |
| q29 | group by a regular expression | 1.532s | 1.621s | 1.578s | 1.9% | 1.559s | 1.589s | 1.558s | 1.665s | 37.960s | 2.90 GiB | 175.39 MiB | 63.38M/s |
| q30 | ninety sums over one column | 42.000ms | 80.582ms | 80.605ms | 0.1% | 80.589ms | 80.636ms | 80.572ms | 80.645ms | 550.000ms | 734.45 MiB | 52.00 KiB | 1.24G/s |
| q31 | group by two and several aggregates | 389.000ms | 464.310ms | 427.023ms | 1.0% | 426.793ms | 430.986ms | 424.487ms | 450.215ms | 8.790s | 2.31 GiB | 25.30 MiB | 234.17M/s |
| q32 | group by a high card pair | 482.000ms | 508.780ms | 517.505ms | 0.2% | 516.927ms | 517.731ms | 509.850ms | 522.633ms | 10.940s | 2.97 GiB | 21.50 MiB | 193.23M/s |
| q33 | group by a high card pair, unfiltered | 1.523s | 1.740s | 1.576s | 0.7% | 1.568s | 1.579s | 1.550s | 1.604s | 41.860s | 10.90 GiB | none | 63.46M/s |
| q34 | group by a long string | 1.849s | 1.928s | 1.913s | 0.6% | 1.907s | 1.919s | 1.892s | 1.935s | 49.180s | 8.63 GiB | none | 52.26M/s |
| q35 | group by a constant and a long string | 1.853s | 1.910s | 1.909s | 2.1% | 1.905s | 1.945s | 1.896s | 2.005s | 48.900s | 8.80 GiB | none | 52.39M/s |
| q36 | group by four expressions | 327.000ms | 351.627ms | 364.675ms | 4.8% | 351.098ms | 368.755ms | 348.070ms | 369.310ms | 7.280s | 1.95 GiB | none | 274.21M/s |
| q37 | date range and group by a URL | 112.000ms | 141.571ms | 141.042ms | 0.2% | 140.857ms | 141.124ms | 140.771ms | 142.095ms | 200.000ms | 429.75 MiB | none | 708.99M/s |
| q38 | date range and group by a title | 73.000ms | 100.604ms | 100.617ms | 0.1% | 100.591ms | 100.645ms | 100.552ms | 100.688ms | 170.000ms | 257.68 MiB | none | 993.85M/s |
| q39 | date range, group by and offset | 79.000ms | 100.586ms | 101.533ms | 0.7% | 100.896ms | 101.592ms | 100.600ms | 120.685ms | 130.000ms | 243.48 MiB | none | 984.87M/s |
| q40 | date range, a case and a wide group by | 214.000ms | 241.140ms | 241.280ms | 0.2% | 241.159ms | 241.544ms | 241.084ms | 261.288ms | 410.000ms | 637.99 MiB | none | 414.45M/s |
| q41 | date range with an IN and a hash | 22.000ms | 60.527ms | 60.436ms | 30.9% | 41.743ms | 60.443ms | 41.298ms | 60.715ms | 60.000ms | 206.20 MiB | none | 1.65G/s |
| q42 | date range and a deep offset | 19.000ms | 41.598ms | 40.882ms | 1.7% | 40.843ms | 41.523ms | 40.836ms | 60.427ms | 60.000ms | 196.47 MiB | none | 2.45G/s |
| q43 | minute buckets over a date range | 16.000ms | 40.413ms | 40.993ms | 2.4% | 40.361ms | 41.346ms | 40.359ms | 41.429ms | 50.000ms | 164.69 MiB | none | 2.44G/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 23.356s by its own clock and 25.055s by ours, 25.848s cold, 563.710s of CPU, peak 10.90 GiB, 184.10M/s and 25.34 GiB/s.

Running it cost 7% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 107.94x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 13.709ms | 241.096ms | 120.713ms | 16.6% | 100.639ms | 120.718ms | 100.598ms | 120.739ms | 120.000ms | 75.56 MiB | 129.26 MiB | 828.39M/s |
| q2 | filtered count | 16.986ms | 120.658ms | 121.001ms | 0.2% | 120.992ms | 121.231ms | 120.766ms | 122.684ms | 130.000ms | 105.95 MiB | 304.00 KiB | 826.42M/s |
| q3 | three aggregates | 42.345ms | 141.984ms | 141.105ms | 0.4% | 140.944ms | 141.487ms | 140.765ms | 142.921ms | 480.000ms | 384.96 MiB | 744.00 KiB | 708.68M/s |
| q4 | average | 65.666ms | 161.371ms | 164.050ms | 12.3% | 161.291ms | 181.505ms | 161.169ms | 221.828ms | 960.000ms | 827.00 MiB | none | 609.55M/s |
| q5 | count distinct, high card | 280.349ms | 407.575ms | 402.451ms | 4.9% | 388.307ms | 407.920ms | 387.710ms | 408.420ms | 6.610s | 1.35 GiB | none | 248.47M/s |
| q6 | count distinct, strings | 382.860ms | 548.564ms | 530.767ms | 0.8% | 528.816ms | 532.806ms | 525.899ms | 537.402ms | 9.260s | 2.62 GiB | none | 188.40M/s |
| q7 | min and max of a date | 31.475ms | 120.724ms | 140.746ms | 14.2% | 120.995ms | 140.964ms | 120.721ms | 147.291ms | 280.000ms | 227.97 MiB | none | 710.48M/s |
| q8 | group by, low card | 25.309ms | 120.702ms | 121.108ms | 0.2% | 120.949ms | 121.242ms | 120.712ms | 121.882ms | 170.000ms | 111.91 MiB | none | 825.69M/s |
| q9 | group by and count distinct | 686.685ms | 853.716ms | 836.800ms | 2.4% | 833.178ms | 853.540ms | 831.872ms | 853.670ms | 18.200s | 3.48 GiB | none | 119.50M/s |
| q10 | group by, several aggregates | 722.493ms | 905.819ms | 888.458ms | 2.6% | 882.894ms | 905.883ms | 870.582ms | 956.351ms | 18.940s | 3.71 GiB | none | 112.55M/s |
| q11 | group by a string and count distinct | 90.706ms | 201.186ms | 201.582ms | 0.6% | 201.557ms | 202.715ms | 182.068ms | 203.400ms | 1.270s | 664.21 MiB | none | 496.06M/s |
| q12 | group by two strings and count distinct | 96.577ms | 201.267ms | 202.384ms | 1.2% | 201.255ms | 203.719ms | 201.220ms | 205.239ms | 1.540s | 695.29 MiB | none | 494.10M/s |
| q13 | group by a string and top k | 330.616ms | 469.564ms | 469.452ms | 4.5% | 469.018ms | 490.338ms | 463.199ms | 495.840ms | 6.240s | 2.53 GiB | none | 213.01M/s |
| q14 | group by a string and count distinct | 759.430ms | 977.333ms | 940.208ms | 1.3% | 939.851ms | 952.200ms | 935.472ms | 983.691ms | 18.330s | 4.49 GiB | none | 106.36M/s |
| q15 | group by two columns and top k | 405.022ms | 569.597ms | 554.509ms | 40.8% | 549.783ms | 775.881ms | 549.604ms | 1.056s | 8.450s | 2.63 GiB | none | 180.34M/s |
| q16 | group by, very high card | 373.037ms | 635.853ms | 514.641ms | 7.4% | 508.503ms | 546.377ms | 506.480ms | 568.932ms | 8.130s | 1.85 GiB | none | 194.31M/s |
| q17 | group by two, very high card | 1.178s | 1.416s | 1.387s | 3.4% | 1.381s | 1.428s | 1.367s | 1.436s | 29.530s | 5.95 GiB | none | 72.10M/s |
| q18 | group by two, no ordering | 1.115s | 1.364s | 1.325s | 2.0% | 1.315s | 1.341s | 1.281s | 1.361s | 30.060s | 6.04 GiB | none | 75.48M/s |
| q19 | group by with an extract | 1.856s | 2.165s | 2.134s | 1.6% | 2.120s | 2.155s | 2.090s | 2.158s | 50.080s | 9.12 GiB | none | 46.87M/s |
| q20 | point lookup | 30.221ms | 140.764ms | 121.189ms | 0.1% | 121.046ms | 121.223ms | 120.737ms | 121.625ms | 250.000ms | 425.97 MiB | 16.00 KiB | 825.14M/s |
| q21 | substring scan | 917.314ms | 1.069s | 1.037s | 2.3% | 1.031s | 1.055s | 1.029s | 1.055s | 13.990s | 2.90 GiB | 8.80 MiB | 96.45M/s |
| q22 | substring scan and group by | 958.891ms | 1.092s | 1.107s | 0.2% | 1.106s | 1.109s | 1.084s | 1.114s | 16.040s | 4.01 GiB | none | 90.29M/s |
| q23 | two substring scans and group by | 1.639s | 1.950s | 1.887s | 0.5% | 1.884s | 1.892s | 1.867s | 1.900s | 37.090s | 12.07 GiB | none | 52.98M/s |
| q24 | select star and top k | 1.167s | 1.391s | 1.346s | 1.3% | 1.342s | 1.360s | 1.326s | 1.369s | 27.120s | 12.99 GiB | 262.34 MiB | 74.31M/s |
| q25 | top k by a date | 170.285ms | 307.524ms | 282.095ms | 0.7% | 282.029ms | 283.926ms | 281.683ms | 303.260ms | 2.600s | 1.72 GiB | none | 354.48M/s |
| q26 | top k by a string | 137.995ms | 242.225ms | 242.065ms | 8.2% | 241.450ms | 261.367ms | 241.446ms | 262.927ms | 1.650s | 941.80 MiB | none | 413.10M/s |
| q27 | top k by two columns | 240.901ms | 365.200ms | 382.329ms | 3.3% | 369.753ms | 382.443ms | 362.674ms | 384.232ms | 4.680s | 2.25 GiB | none | 261.55M/s |
| q30 | ninety sums over one column | 112.709ms | 202.444ms | 223.891ms | 8.9% | 204.189ms | 224.016ms | 201.328ms | 224.026ms | 2.750s | 279.55 MiB | none | 446.63M/s |
| q31 | group by two and several aggregates | 344.402ms | 464.205ms | 482.931ms | 3.3% | 471.950ms | 488.006ms | 471.928ms | 492.161ms | 7.010s | 1.82 GiB | none | 207.06M/s |
| q32 | group by a high card pair | 410.682ms | 580.910ms | 569.911ms | 3.0% | 553.415ms | 570.678ms | 552.149ms | 587.206ms | 8.730s | 2.76 GiB | none | 175.46M/s |
| q33 | group by a high card pair, unfiltered | 2.064s | 2.566s | 2.510s | 2.0% | 2.484s | 2.534s | 2.478s | 2.546s | 55.920s | 14.01 GiB | 132.00 KiB | 39.83M/s |
| q34 | group by a long string | 1.798s | 2.315s | 2.289s | 0.9% | 2.282s | 2.303s | 2.257s | 2.470s | 45.490s | 16.68 GiB | 1.68 MiB | 43.68M/s |
| q35 | group by a constant and a long string | 2.587s | 2.984s | 2.983s | 0.2% | 2.978s | 2.985s | 2.947s | 2.995s | 72.730s | 13.78 GiB | none | 33.52M/s |
| q37 | date range and group by a URL | 87.805ms | 221.599ms | 201.034ms | 0.0% | 201.017ms | 201.054ms | 200.989ms | 201.913ms | 400.000ms | 383.80 MiB | 30.68 MiB | 497.42M/s |
| q38 | date range and group by a title | 61.785ms | 162.845ms | 160.932ms | 12.4% | 160.899ms | 180.907ms | 160.865ms | 180.911ms | 250.000ms | 220.59 MiB | 9.51 MiB | 621.37M/s |
| q39 | date range, group by and offset | 44.374ms | 140.766ms | 140.813ms | 0.0% | 140.776ms | 140.838ms | 140.758ms | 161.032ms | 190.000ms | 122.62 MiB | none | 710.15M/s |
| q40 | date range, a case and a wide group by | 132.119ms | 261.530ms | 261.404ms | 7.4% | 242.109ms | 261.418ms | 241.331ms | 261.432ms | 780.000ms | 784.95 MiB | 260.00 KiB | 382.54M/s |
| q41 | date range with an IN and a hash | 30.457ms | 120.632ms | 120.668ms | 0.1% | 120.652ms | 120.822ms | 120.646ms | 120.909ms | 170.000ms | 119.22 MiB | 3.64 MiB | 828.70M/s |
| q42 | date range and a deep offset | 23.997ms | 120.730ms | 120.677ms | 0.0% | 120.668ms | 120.699ms | 120.646ms | 121.043ms | 150.000ms | 113.70 MiB | none | 828.63M/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 21.432s by its own clock and 27.665s by ours, 28.322s cold, 506.770s of CPU, peak 16.68 GiB, 181.97M/s and 25.05 GiB/s.

Running it cost 29% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 24.72x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 38.350ms | 66.995ms | 60.574ms | 0.1% | 60.564ms | 60.605ms | 60.453ms | 72.808ms | 260.000ms | 21.69 MiB | 9.50 MiB | 1.65G/s |
| q2 | filtered count | 58.430ms | 80.764ms | 80.792ms | 0.1% | 80.727ms | 80.810ms | 80.634ms | 80.822ms | 560.000ms | 81.49 MiB | 1.74 MiB | 1.24G/s |
| q3 | three aggregates | 77.091ms | 100.728ms | 100.803ms | 0.0% | 100.786ms | 100.822ms | 100.727ms | 100.984ms | 830.000ms | 117.46 MiB | none | 992.00M/s |
| q4 | average | 66.456ms | 120.893ms | 80.706ms | 0.3% | 80.594ms | 80.854ms | 80.584ms | 100.590ms | 660.000ms | 93.19 MiB | 288.52 MiB | 1.24G/s |
| q5 | count distinct, high card | 421.776ms | 422.543ms | 446.897ms | 1.3% | 442.601ms | 448.427ms | 442.467ms | 473.419ms | 4.730s | 3.15 GiB | none | 223.76M/s |
| q6 | count distinct, strings | 1.292s | 1.366s | 1.346s | 0.0% | 1.346s | 1.346s | 1.346s | 1.372s | 8.040s | 1.17 GiB | 358.86 MiB | 74.28M/s |
| q7 | min and max of a date | 75.224ms | 101.265ms | 100.962ms | 1.5% | 100.855ms | 102.363ms | 100.839ms | 121.341ms | 750.000ms | 88.04 MiB | 12.00 KiB | 990.45M/s |
| q8 | group by, low card | 63.461ms | 100.691ms | 80.736ms | 0.1% | 80.640ms | 80.754ms | 80.631ms | 80.773ms | 600.000ms | 84.22 MiB | none | 1.24G/s |
| q9 | group by and count distinct | 515.923ms | 563.019ms | 563.408ms | 2.7% | 548.437ms | 563.681ms | 547.075ms | 602.399ms | 6.160s | 3.09 GiB | 46.66 MiB | 177.49M/s |
| q10 | group by, several aggregates | 741.048ms | 756.032ms | 767.916ms | 1.0% | 763.983ms | 771.496ms | 751.703ms | 784.862ms | 9.420s | 3.24 GiB | none | 130.22M/s |
| q11 | group by a string and count distinct | 302.935ms | 331.474ms | 322.409ms | 7.1% | 321.801ms | 344.593ms | 301.884ms | 348.155ms | 3.980s | 182.27 MiB | 6.54 MiB | 310.16M/s |
| q12 | group by two strings and count distinct | 345.054ms | 362.064ms | 371.697ms | 2.2% | 363.766ms | 371.961ms | 344.427ms | 393.842ms | 4.490s | 204.43 MiB | 25.37 MiB | 269.03M/s |
| q13 | group by a string and top k | 666.523ms | 726.099ms | 703.650ms | 0.8% | 703.324ms | 709.283ms | 683.454ms | 719.502ms | 7.540s | 852.92 MiB | none | 142.11M/s |
| q14 | group by a string and count distinct | 1.318s | 1.387s | 1.353s | 1.5% | 1.348s | 1.368s | 1.328s | 1.434s | 15.500s | 1.35 GiB | none | 73.91M/s |
| q15 | group by two columns and top k | 936.879ms | 975.081ms | 970.542ms | 0.4% | 968.005ms | 971.671ms | 956.242ms | 972.035ms | 10.330s | 1.07 GiB | 788.00 KiB | 103.03M/s |
| q16 | group by, very high card | 802.444ms | 945.233ms | 824.712ms | 0.2% | 824.509ms | 826.366ms | 804.428ms | 904.810ms | 10.240s | 2.26 GiB | none | 121.25M/s |
| q17 | group by two, very high card | 1.821s | 1.790s | 1.880s | 3.7% | 1.810s | 1.880s | 1.789s | 1.882s | 18.150s | 2.25 GiB | none | 53.20M/s |
| q18 | group by two, no ordering | 426.113ms | 451.744ms | 455.753ms | 1.2% | 452.134ms | 457.472ms | 444.042ms | 462.329ms | 6.250s | 195.57 MiB | none | 219.41M/s |
| q19 | group by with an extract | 2.919s | 3.027s | 3.017s | 2.7% | 3.002s | 3.082s | 2.956s | 3.129s | 26.800s | 4.66 GiB | 368.09 MiB | 33.15M/s |
| q20 | point lookup | 168.213ms | 182.282ms | 183.897ms | 10.9% | 181.222ms | 201.340ms | 181.176ms | 202.581ms | 2.180s | 101.79 MiB | none | 543.77M/s |
| q21 | substring scan | 623.762ms | 643.843ms | 643.399ms | 0.4% | 643.345ms | 646.146ms | 643.303ms | 648.653ms | 9.280s | 129.63 MiB | none | 155.42M/s |
| q22 | substring scan and group by | 869.253ms | 886.456ms | 885.468ms | 0.1% | 885.200ms | 886.324ms | 884.897ms | 886.983ms | 13.100s | 207.04 MiB | none | 112.93M/s |
| q23 | two substring scans and group by | 2.271s | 2.375s | 2.294s | 0.1% | 2.294s | 2.296s | 2.280s | 2.335s | 34.660s | 357.41 MiB | 1.56 GiB | 43.58M/s |
| q24 | select star and top k | 987.283ms | 1.009s | 1.006s | 0.0% | 1.006s | 1.006s | 1.006s | 1.007s | 14.910s | 220.59 MiB | none | 99.40M/s |
| q25 | top k by a date | 483.724ms | 504.768ms | 507.158ms | 0.4% | 505.244ms | 507.486ms | 503.379ms | 513.891ms | 7.120s | 198.49 MiB | none | 197.17M/s |
| q26 | top k by a string | 273.267ms | 302.040ms | 304.055ms | 1.4% | 301.799ms | 306.008ms | 289.415ms | 306.522ms | 3.840s | 160.25 MiB | none | 328.88M/s |
| q27 | top k by two columns | 484.086ms | 503.036ms | 504.174ms | 0.8% | 503.090ms | 507.267ms | 503.034ms | 531.575ms | 7.080s | 198.16 MiB | none | 198.34M/s |
| q28 | group by with a string length | 820.873ms | 828.275ms | 844.312ms | 2.3% | 825.466ms | 844.552ms | 824.911ms | 844.578ms | 12.220s | 157.71 MiB | none | 118.44M/s |
| q29 | group by a regular expression | 2.077s | 2.111s | 2.132s | 0.2% | 2.132s | 2.137s | 2.110s | 2.138s | 26.930s | 1.49 GiB | none | 46.90M/s |
| q30 | ninety sums over one column | 71.335ms | 100.683ms | 101.148ms | 0.9% | 100.805ms | 101.718ms | 100.733ms | 101.934ms | 710.000ms | 81.14 MiB | none | 988.62M/s |
| q31 | group by two and several aggregates | 591.052ms | 623.523ms | 623.959ms | 0.2% | 623.903ms | 625.080ms | 623.651ms | 629.422ms | 8.480s | 816.53 MiB | none | 160.26M/s |
| q32 | group by a high card pair | 746.865ms | 795.621ms | 784.745ms | 0.0% | 784.669ms | 784.912ms | 784.496ms | 785.012ms | 10.900s | 1.00 GiB | 68.00 KiB | 127.43M/s |
| q33 | group by a high card pair, unfiltered | 1.603s | 1.669s | 1.629s | 1.6% | 1.612s | 1.637s | 1.609s | 1.670s | 22.530s | 5.69 GiB | none | 61.40M/s |
| q34 | group by a long string | 2.462s | 2.476s | 2.493s | 0.8% | 2.492s | 2.512s | 2.472s | 2.517s | 27.010s | 4.36 GiB | none | 40.12M/s |
| q35 | group by a constant and a long string | 2.456s | 2.493s | 2.494s | 0.1% | 2.492s | 2.494s | 2.472s | 2.512s | 27.030s | 4.37 GiB | none | 40.09M/s |
| q36 | group by four expressions | 667.922ms | 686.214ms | 695.021ms | 2.4% | 688.998ms | 705.826ms | 684.223ms | 706.263ms | 9.100s | 1.08 GiB | none | 143.88M/s |
| q37 | date range and group by a URL | 130.362ms | 160.911ms | 160.895ms | 0.1% | 160.881ms | 160.998ms | 160.850ms | 161.508ms | 240.000ms | 130.39 MiB | none | 621.51M/s |
| q38 | date range and group by a title | 98.142ms | 120.850ms | 120.733ms | 0.1% | 120.720ms | 120.785ms | 120.713ms | 120.795ms | 180.000ms | 60.40 MiB | none | 828.25M/s |
| q39 | date range, group by and offset | 76.446ms | 100.641ms | 100.659ms | 0.0% | 100.644ms | 100.666ms | 100.642ms | 100.669ms | 160.000ms | 67.54 MiB | none | 993.43M/s |
| q40 | date range, a case and a wide group by | 331.810ms | 361.658ms | 361.675ms | 0.0% | 361.673ms | 361.714ms | 361.638ms | 361.805ms | 620.000ms | 277.80 MiB | none | 276.48M/s |
| q41 | date range with an IN and a hash | 72.738ms | 100.655ms | 100.740ms | 0.2% | 100.707ms | 100.932ms | 100.627ms | 100.932ms | 140.000ms | 73.91 MiB | none | 992.63M/s |
| q42 | date range and a deep offset | 55.356ms | 80.558ms | 80.595ms | 0.1% | 80.574ms | 80.674ms | 80.571ms | 81.069ms | 100.000ms | 71.86 MiB | none | 1.24G/s |
| q43 | minute buckets over a date range | 51.453ms | 80.585ms | 80.567ms | 0.1% | 80.562ms | 80.614ms | 80.553ms | 80.676ms | 90.000ms | 58.12 MiB | none | 1.24G/s |

rudb rudb 0.3.45 over 43 of 43 queries. Total 31.361s by its own clock and 32.658s by ours, 32.870s cold, 373.900s of CPU, peak 5.69 GiB, 137.11M/s and 18.87 GiB/s.

Running it cost 4% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 49.80x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | 12.166ms | 20.142ms | 181.685ms | 181.692ms | 0.0% | 87.751ms | 5.656ms | 42.652ms | 896 B | 4 of 4 |
| q2 | 11.599ms | 41.802ms | 537.308ms | 537.314ms | 0.0% | 111.192ms | 5.209ms | 17.478ms | 896 B | 5 of 5 |
| q3 | 12.039ms | 55.005ms | 803.675ms | 803.685ms | 0.0% | 89.499ms | 5.437ms | 20.879ms | 1.22 KiB | 4 of 4 |
| q4 | 11.948ms | 74.714ms | 719.786ms | 719.794ms | 0.0% | 78.099ms | 5.372ms | 14.835ms | 896 B | 4 of 4 |
| q5 | 11.639ms | 333.066ms | 2.285s | 2.285s | 0.0% | 93.352ms | 5.307ms | 2.260s | 3.03 GiB | 4 of 4 |
| q6 | 12.130ms | 1.294s | 7.776s | 7.776s | 0.0% | 114.676ms | 5.341ms | 348.971ms | 1.19 GiB | 5 of 5 |
| q7 | 11.585ms | 53.359ms | 688.294ms | 688.299ms | 0.0% | 86.629ms | 5.314ms | 36.387ms | 1.00 KiB | 4 of 4 |
| q8 | 11.865ms | 50.412ms | 535.567ms | 535.574ms | 0.0% | 105.679ms | 5.260ms | 29.166ms | 18.90 KiB | 6 of 6 |
| q9 | 12.903ms | 499.551ms | 3.773s | 3.773s | 0.0% | 96.954ms | 5.963ms | 2.331s | 3.37 GiB | 5 of 5 |
| q10 | 13.018ms | 703.215ms | 6.851s | 6.851s | 0.0% | 114.203ms | 5.145ms | 2.453s | 3.29 GiB | 5 of 5 |
| q11 | 13.808ms | 290.474ms | 4.063s | 4.063s | 0.0% | 131.157ms | 5.366ms | 12.111ms | 22.38 MiB | 6 of 6 |
| q12 | 11.886ms | 324.737ms | 4.613s | 4.613s | 0.0% | 124.001ms | 5.393ms | 21.987ms | 22.49 MiB | 6 of 6 |
| q13 | 12.409ms | 672.285ms | 7.687s | 7.687s | 0.0% | 155.429ms | 6.098ms | 97.149ms | 827.27 MiB | 6 of 6 |
| q14 | 12.125ms | 1.333s | 15.194s | 15.194s | 0.0% | 186.443ms | 5.624ms | 650.259ms | 1.41 GiB | 6 of 6 |
| q15 | 12.505ms | 913.902ms | 10.244s | 10.244s | 0.0% | 170.313ms | 5.258ms | 130.718ms | 1.06 GiB | 6 of 6 |
| q16 | 12.294ms | 883.157ms | 7.759s | 7.759s | 0.0% | 106.705ms | 5.718ms | 3.105s | 1.25 GiB | 5 of 5 |
| q17 | 11.680ms | 1.735s | 17.153s | 17.153s | 0.0% | 147.242ms | 5.320ms | 331.714ms | 2.24 GiB | 5 of 5 |
| q18 | 12.681ms | 407.586ms | 6.233s | 6.233s | 0.0% | 98.686ms | 5.292ms | 11.491ms | 17.00 KiB | 5 of 5 |
| q19 | 13.063ms | 2.926s | 26.075s | 26.075s | 0.0% | 161.657ms | 5.194ms | 669.348ms | 4.32 GiB | 5 of 5 |
| q20 | 11.792ms | 145.753ms | 2.074s | 2.074s | 0.0% | 96.606ms | 5.224ms | 20.756ms | 192 B | 4 of 4 |
| q21 | 11.977ms | 597.565ms | 9.238s | 9.238s | 0.0% | 122.643ms | 5.337ms | 16.670ms | 896 B | 5 of 5 |
| q22 | 11.616ms | 842.684ms | 13.066s | 13.066s | 0.0% | 127.822ms | 5.364ms | 28.462ms | 399.17 KiB | 6 of 6 |
| q23 | 12.076ms | 2.335s | 35.516s | 35.516s | 0.0% | 167.341ms | 5.344ms | 18.437ms | 2.75 MiB | 6 of 6 |
| q24 | 11.777ms | 971.494ms | 14.397s | 14.397s | 0.0% | 169.877ms | 5.286ms | 558.014ms | 43.03 KiB | 8 of 8 |
| q25 | 11.966ms | 468.506ms | 7.196s | 7.196s | 0.0% | 151.252ms | 5.505ms | 28.767ms | 52.57 KiB | 6 of 6 |
| q26 | 11.835ms | 253.044ms | 3.799s | 3.799s | 0.0% | 144.292ms | 5.453ms | 25.850ms | 47.14 KiB | 5 of 5 |
| q27 | 11.817ms | 452.465ms | 7.013s | 7.013s | 0.0% | 148.169ms | 5.280ms | 21.966ms | 72.67 KiB | 6 of 6 |
| q28 | 11.830ms | 777.668ms | 12.041s | 12.041s | 0.0% | 132.754ms | 5.295ms | 23.387ms | 1.01 MiB | 7 of 7 |
| q29 | 11.855ms | 2.038s | 24.589s | 24.589s | 0.0% | 183.252ms | 5.305ms | 2.276s | 1.02 GiB | 7 of 7 |
| q30 | 12.852ms | 51.684ms | 680.399ms | 680.412ms | 0.0% | 89.225ms | 5.299ms | 24.289ms | 29.59 KiB | 4 of 4 |
| q31 | 12.589ms | 565.344ms | 7.014s | 7.014s | 0.0% | 127.718ms | 5.777ms | 1.431s | 801.99 MiB | 6 of 6 |
| q32 | 12.023ms | 727.211ms | 9.419s | 9.419s | 0.0% | 133.061ms | 5.401ms | 1.536s | 777.99 MiB | 6 of 6 |
| q33 | 11.946ms | 1.584s | 8.758s | 8.758s | 0.0% | 104.045ms | 5.254ms | 14.716s | 5.98 GiB | 5 of 5 |
| q34 | 11.568ms | 2.432s | 26.677s | 26.677s | 0.0% | 162.079ms | 5.150ms | 328.115ms | 5.55 GiB | 5 of 5 |
| q35 | 11.762ms | 2.432s | 26.677s | 26.677s | 0.0% | 166.266ms | 5.231ms | 327.642ms | 5.82 GiB | 5 of 5 |
| q36 | 11.978ms | 642.360ms | 7.063s | 7.063s | 0.0% | 96.275ms | 5.231ms | 1.892s | 855.71 MiB | 6 of 6 |
| q37 | 12.056ms | 112.889ms | 214.752ms | 214.761ms | 0.0% | 2.414ms | 5.566ms | 19.672ms | 69.14 MiB | 6 of 6 |
| q38 | 11.597ms | 78.720ms | 154.347ms | 154.356ms | 0.0% | 2.186ms | 5.300ms | 20.344ms | 6.47 MiB | 6 of 6 |
| q39 | 12.236ms | 58.038ms | 136.237ms | 136.244ms | 0.0% | 2.622ms | 5.191ms | 48.564ms | 3.79 MiB | 6 of 6 |
| q40 | 11.708ms | 311.361ms | 564.256ms | 564.262ms | 0.0% | 2.682ms | 5.267ms | 50.470ms | 196.41 MiB | 6 of 6 |
| q41 | 11.684ms | 53.540ms | 112.969ms | 112.978ms | 0.0% | 2.338ms | 5.190ms | 11.832ms | 3.57 MiB | 6 of 6 |
| q42 | 11.911ms | 35.286ms | 68.428ms | 68.437ms | 0.0% | 2.350ms | 5.373ms | 16.190ms | 3.33 MiB | 6 of 6 |
| q43 | 11.738ms | 31.717ms | 65.922ms | 65.929ms | 0.0% | 1.685ms | 5.166ms | 8.905ms | 407.06 KiB | 6 of 6 |

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
| Aggregate | 150.867s | 45.0% | 39 | 1968178868 | 6056627 | 76.7ns | 24909.4ns | 39 of 39 |
| FileScan | 133.075s | 39.7% | 43 | 0 | 3597060701 | handed none | 37.0ns | 43 of 43 |
| Filter | 45.794s | 13.7% | 28 | 1897103429 | 301735584 | 24.1ns | 151.8ns | 28 of 28 |
| Project | 4.774s | 1.4% | 91 | 2041263671 | 2041263671 | 2.3ns | 2.3ns | 91 of 91 |
| TopN | 587.339ms | 0.2% | 31 | 39570574 | 340 | 14.8ns | 1727467.9ns | 31 of 31 |
| Fetch | 8.805ms | 0.0% | 1 | 10 | 10 | 880531.7ns | 880531.7ns | 1 of 1 |
| Sort | 12.160us | 0.0% | 1 | 18 | 18 | 675.6ns | 675.6ns | 1 of 1 |
| Limit | 0.352us | 0.0% | 1 | 10 | 10 | 35.2ns | 35.2ns | 1 of 1 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is Aggregate, and the queries where it cost the most are q19 at 21.488s, q35 at 17.550s, q34 at 17.423s.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- q4 swung by 33.2% of its median, and rule two wants under 10%
- q11 swung by 11.8% of its median, and rule two wants under 10%
- q12 swung by 24.5% of its median, and rule two wants under 10%
- q26 swung by 15.0% of its median, and rule two wants under 10%
- q39 swung by 49.8% of its median, and rule two wants under 10%

These swung wider than reporting rule two allows:

- duckdb swung by 49.8% of its median on q39, and rule two wants under 10%
- duckdb-pinned swung by 50.0% of its median on q1, and rule two wants under 10%
- clickhouse-local swung by 12.1% of its median on q30, and rule two wants under 10%
- datafusion swung by 49.7% of its median on q7, and rule two wants under 10%
- polars swung by 40.8% of its median on q15, and rule two wants under 10%
- rudb swung by 10.9% of its median on q20, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

polars did not run q28, because the SQLContext has no strlen and no character length function.

polars did not run q29, because the SQLContext has no regexp_replace.

polars did not run q36, because the SQLContext rejects the repeated ClientIP output name in the group by.

polars did not run q43, because the SQLContext has no date_trunc.

So the polars column is 39 of 43 queries and its ratio is over the shared ones.

Answers differ, so this is not a comparison: q25: duckdb-pinned does not agree with duckdb: the same 0 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q25: clickhouse-local does not agree with duckdb: the same 0 numbers and 1 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q25: datafusion does not agree with duckdb: the same 0 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q25: polars does not agree with duckdb: the same 0 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q25: rudb does not agree with duckdb: the same 0 numbers and 2 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q28: duckdb-pinned does not agree with duckdb: 75 numbers against 75

These were answered differently and the data does not say which is right:

- q16: ORDER BY COUNT(*) DESC LIMIT 10 over user IDs, where the tenth place is a tie between many users with the same count and the engine picks.
- q17: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q18: LIMIT 10 with no ORDER BY at all, so the ten rows are whichever ten the group by handed back first and the query did not ask for any particular ten.
- q19: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q24: ORDER BY EventTime LIMIT 10, and EventTime has one second resolution over a corpus with millions of rows a second, so the tenth row is a tie.
- q29: ORDER BY an AVG of a length DESC LIMIT 25, which is both a tie at the cut and a double computed in a different order by each engine.
- q31: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q32: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q33: ORDER BY COUNT(*) DESC LIMIT 10 over WatchID and ClientIP unfiltered, where almost every group has a count of one and the ten that come back are arbitrary.
- q39: ORDER BY PageViews DESC LIMIT 10 OFFSET 1000, which is a tie at the cut a thousand rows deeper in, where the counts are smaller and the ties are denser.
- q40: ORDER BY PageViews DESC LIMIT 10 OFFSET 1000 with ties at the cut, as q39.
- q41: ORDER BY PageViews DESC LIMIT 10 OFFSET 100 with ties at the cut, as q39.
- q42: ORDER BY PageViews DESC LIMIT 10 OFFSET 10000 with ties at the cut, as q39.
- q43: DATE_TRUNC on a timestamp, and ClickHouse renders a DateTime in the machine's timezone where DuckDB renders a TIMESTAMP in none, so the same epoch second prints seven hours apart on gamingpc-wsl and two hours apart on server3.

So they are not checked, and a wrong answer from any engine on one of them would go unnoticed here. Every other query in the suite is checked in full.

These were answered differently and which engine is wrong is settled:

- q4: AVG(UserID) over a hundred million bigints, where the pinned DuckDB sums the column short by a multiple of 2^64 and rudb does not. Its own hugeint and decimal paths, the same column summed out of a table, its own per counter group sums, and adding the column up outside a database all give rudb's answer, and it reduces to two statements over a 1.3 MB one column parquet file. Written up as 14.10.1 in tamnd/rudb `spec/14-testing.md`, which is the list of places where DuckDB is wrong and we are not. tamnd/rudb-compat#12.

So they are a known difference rather than an open one, and the write up is where to go to disagree with that.

Hot here means page cache warm and not buffer pool warm, because every run is a fresh process so that no query's number depends on the one before it.

