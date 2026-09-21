# clickbench on gamingpc-wsl

This is one run of the clickbench suite on gamingpc-wsl, over 3 engines and 43 queries, with 5 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

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
rudb-bench run clickbench --engines datafusion,polars,rudb --runs 5 --timeout 600 --report
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
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 13.76 GiB | the source Parquet, this engine has no storage format of its own | the Parquet | 79.97 to 28.39 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 13.76 GiB | the source Parquet, this engine has no storage format of its own | the Parquet | 28.39 to 32.79 |
| rudb | rudb 0.3.45 | ran | 0.000us | 0.000us | 13.76 GiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 32.79 to 18.84 |
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | duckdb failed: it said nothing at all | n/a | n/a | n/a | n/a | n/a | n/a |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | duckdb-pinned failed: it said nothing at all | n/a | n/a | n/a | n/a | n/a | n/a |
| clickhouse-local | 26.9.1.1562 | clickhouse local failed: 12.645 Code: 243. DB::Exception: Cannot reserve 266.93 MiB, not enough space. (NOT_ENOUGH_SPACE) | n/a | n/a | n/a | n/a | n/a | n/a |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

These engines started while the machine was already above half of that, which means they were measured against somebody else's work rather than on an idle box: datafusion, polars, rudb. Their numbers are inflated by an amount nothing here can recover, and by a different amount each, depending on how much of the column was CPU bound. One case where that reading is unfair is a sweep that runs engines back to back, where the average has not had a minute to come down from the engine before.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs datafusion on 39 shared |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| datafusion | 23.555s | 25.305s | +7% | 26.018s | 561.750s | 22.20 | 9.92 GiB | 50.61 MiB | 182.55M/s | 25.13 GiB/s | 1.00x |
| polars | 22.248s | 28.568s | +28% | 29.569s | 485.170s | 16.98 | 16.66 GiB | none | 175.29M/s | 24.13 GiB/s | 1.05x |
| rudb | 32.512s | 33.795s | +4% | 33.730s | 389.630s | 11.53 | 5.73 GiB | none | 132.25M/s | 18.21 GiB/s | 1.36x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | datafusion | polars | rudb |
| --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 12.968ms | 39.381ms |
| q2 | filtered count | 20.000ms | 17.359ms | 60.445ms |
| q3 | three aggregates | 44.000ms | 44.649ms | 80.815ms |
| q4 | average | 53.000ms | 66.707ms | 67.023ms |
| q5 | count distinct, high card | 328.000ms | 282.147ms | 443.460ms |
| q6 | count distinct, strings | 368.000ms | 382.558ms | 1.355s |
| q7 | min and max of a date | 1.000ms | 35.362ms | 74.924ms |
| q8 | group by, low card | 22.000ms | 26.791ms | 63.131ms |
| q9 | group by and count distinct | 423.000ms | 707.601ms | 539.975ms |
| q10 | group by, several aggregates | 435.000ms | 730.250ms | 683.560ms |
| q11 | group by a string and count distinct | 108.000ms | 89.265ms | 311.076ms |
| q12 | group by two strings and count distinct | 116.000ms | 100.456ms | 327.450ms |
| q13 | group by a string and top k | 402.000ms | 315.606ms | 692.525ms |
| q14 | group by a string and count distinct | 638.000ms | 757.201ms | 1.371s |
| q15 | group by two columns and top k | 437.000ms | 407.215ms | 901.690ms |
| q16 | group by, very high card | 388.000ms | 344.694ms | 844.337ms |
| q17 | group by two, very high card | 844.000ms | 1.132s | 1.784s |
| q18 | group by two, no ordering | 838.000ms | 1.126s | 413.350ms |
| q19 | group by with an extract | 1.568s | 1.855s | 3.012s |
| q20 | point lookup | 45.000ms | 31.037ms | 172.867ms |
| q21 | substring scan | 545.000ms | 999.977ms | 643.581ms |
| q22 | substring scan and group by | 587.000ms | 960.149ms | 938.649ms |
| q23 | two substring scans and group by | 1.369s | 1.940s | 2.424s |
| q24 | select star and top k | 4.234s | 1.383s | 1.078s |
| q25 | top k by a date | 112.000ms | 181.565ms | 470.617ms |
| q26 | top k by a string | 168.000ms | 136.744ms | 282.618ms |
| q27 | top k by two columns | 115.000ms | 242.096ms | 508.978ms |
| q28 | group by with a string length | 571.000ms | no dialect | 830.027ms |
| q29 | group by a regular expression | 1.522s | no dialect | 2.157s |
| q30 | ninety sums over one column | 43.000ms | 113.748ms | 71.915ms |
| q31 | group by two and several aggregates | 383.000ms | 355.848ms | 581.412ms |
| q32 | group by a high card pair | 468.000ms | 455.319ms | 793.100ms |
| q33 | group by a high card pair, unfiltered | 1.689s | 2.122s | 1.816s |
| q34 | group by a long string | 1.938s | 1.900s | 2.528s |
| q35 | group by a constant and a long string | 1.876s | 2.602s | 2.580s |
| q36 | group by four expressions | 322.000ms | no dialect | 712.016ms |
| q37 | date range and group by a URL | 112.000ms | 91.612ms | 130.287ms |
| q38 | date range and group by a title | 73.000ms | 64.179ms | 101.015ms |
| q39 | date range, group by and offset | 78.000ms | 47.960ms | 80.065ms |
| q40 | date range, a case and a wide group by | 214.000ms | 131.022ms | 356.841ms |
| q41 | date range with an IN and a hash | 22.000ms | 29.992ms | 79.702ms |
| q42 | date range and a deep offset | 19.000ms | 25.937ms | 57.841ms |
| q43 | minute buckets over a date range | 16.000ms | no dialect | 52.077ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 100.702ms | 40.387ms | 0.1% | 40.382ms | 40.417ms | 40.363ms | 40.474ms | 10.000ms | 100.18 MiB | 132.50 MiB | 2.48G/s |
| q2 | filtered count | 20.000ms | 60.598ms | 60.571ms | 0.2% | 60.519ms | 60.622ms | 60.461ms | 60.631ms | 320.000ms | 257.24 MiB | 1.45 MiB | 1.65G/s |
| q3 | three aggregates | 44.000ms | 80.707ms | 80.793ms | 1.6% | 80.693ms | 81.954ms | 80.660ms | 81.978ms | 720.000ms | 720.81 MiB | 48.94 MiB | 1.24G/s |
| q4 | average | 53.000ms | 100.793ms | 81.558ms | 24.6% | 80.786ms | 100.832ms | 80.717ms | 101.201ms | 890.000ms | 688.40 MiB | 198.07 MiB | 1.23G/s |
| q5 | count distinct, high card | 328.000ms | 368.403ms | 366.239ms | 2.9% | 365.096ms | 375.566ms | 351.399ms | 389.195ms | 7.780s | 2.22 GiB | none | 273.04M/s |
| q6 | count distinct, strings | 368.000ms | 409.403ms | 408.098ms | 1.4% | 403.997ms | 409.732ms | 390.636ms | 410.863ms | 7.480s | 2.42 GiB | 240.23 MiB | 245.03M/s |
| q7 | min and max of a date | 1.000ms | 40.365ms | 40.387ms | 0.1% | 40.380ms | 40.402ms | 20.363ms | 40.517ms | 10.000ms | 101.71 MiB | 4.00 KiB | 2.48G/s |
| q8 | group by, low card | 22.000ms | 60.678ms | 60.549ms | 0.1% | 60.508ms | 60.568ms | 60.507ms | 60.608ms | 340.000ms | 279.57 MiB | none | 1.65G/s |
| q9 | group by and count distinct | 423.000ms | 473.411ms | 467.324ms | 4.8% | 449.174ms | 471.621ms | 447.553ms | 477.379ms | 9.440s | 2.38 GiB | 42.33 MiB | 213.98M/s |
| q10 | group by, several aggregates | 435.000ms | 467.380ms | 471.132ms | 4.8% | 463.316ms | 486.015ms | 451.069ms | 486.395ms | 6.790s | 2.33 GiB | none | 212.25M/s |
| q11 | group by a string and count distinct | 108.000ms | 142.764ms | 144.423ms | 2.9% | 144.306ms | 148.495ms | 141.819ms | 167.293ms | 2.130s | 1.14 GiB | 3.89 MiB | 692.39M/s |
| q12 | group by two strings and count distinct | 116.000ms | 167.499ms | 144.604ms | 14.6% | 143.102ms | 164.256ms | 142.061ms | 172.655ms | 2.440s | 1.24 GiB | 4.23 MiB | 691.52M/s |
| q13 | group by a string and top k | 402.000ms | 447.557ms | 445.163ms | 3.6% | 429.678ms | 445.588ms | 426.797ms | 475.105ms | 8.510s | 2.53 GiB | none | 224.63M/s |
| q14 | group by a string and count distinct | 638.000ms | 679.505ms | 683.516ms | 44.7% | 675.021ms | 980.420ms | 674.047ms | 1.148s | 14.130s | 3.30 GiB | none | 146.30M/s |
| q15 | group by two columns and top k | 437.000ms | 560.535ms | 472.828ms | 3.7% | 468.756ms | 486.404ms | 468.700ms | 488.545ms | 8.490s | 2.40 GiB | 12.59 MiB | 211.49M/s |
| q16 | group by, very high card | 388.000ms | 406.441ms | 426.562ms | 1.2% | 425.684ms | 431.013ms | 406.859ms | 433.019ms | 8.960s | 2.50 GiB | none | 234.43M/s |
| q17 | group by two, very high card | 844.000ms | 922.696ms | 892.346ms | 1.5% | 882.688ms | 895.894ms | 863.264ms | 936.908ms | 21.010s | 5.42 GiB | none | 112.06M/s |
| q18 | group by two, no ordering | 838.000ms | 858.829ms | 881.742ms | 1.6% | 877.655ms | 891.772ms | 861.998ms | 898.754ms | 20.730s | 5.41 GiB | none | 113.41M/s |
| q19 | group by with an extract | 1.568s | 1.704s | 1.617s | 2.2% | 1.608s | 1.643s | 1.562s | 1.663s | 42.540s | 8.54 GiB | 334.52 MiB | 61.84M/s |
| q20 | point lookup | 45.000ms | 80.977ms | 80.818ms | 0.6% | 80.720ms | 81.168ms | 80.608ms | 83.523ms | 680.000ms | 634.91 MiB | none | 1.24G/s |
| q21 | substring scan | 545.000ms | 698.411ms | 574.193ms | 3.0% | 572.796ms | 590.255ms | 565.107ms | 601.691ms | 11.220s | 1.56 GiB | 2.73 GiB | 174.15M/s |
| q22 | substring scan and group by | 587.000ms | 631.991ms | 616.146ms | 2.2% | 614.692ms | 628.516ms | 612.199ms | 648.348ms | 13.400s | 1.94 GiB | none | 162.30M/s |
| q23 | two substring scans and group by | 1.369s | 1.524s | 1.409s | 1.8% | 1.403s | 1.428s | 1.356s | 1.443s | 33.500s | 2.90 GiB | 2.05 GiB | 70.97M/s |
| q24 | select star and top k | 4.234s | 4.333s | 4.289s | 1.4% | 4.229s | 4.291s | 4.221s | 4.526s | 109.510s | 6.66 GiB | 3.67 GiB | 23.31M/s |
| q25 | top k by a date | 112.000ms | 145.364ms | 142.864ms | 1.5% | 142.238ms | 144.407ms | 140.985ms | 161.517ms | 1.480s | 882.73 MiB | none | 699.95M/s |
| q26 | top k by a string | 168.000ms | 204.655ms | 203.580ms | 0.8% | 202.246ms | 203.783ms | 201.958ms | 207.629ms | 2.820s | 1.00 GiB | none | 491.19M/s |
| q27 | top k by two columns | 115.000ms | 140.966ms | 142.752ms | 1.2% | 141.133ms | 142.808ms | 141.102ms | 161.470ms | 1.560s | 895.86 MiB | none | 700.50M/s |
| q28 | group by with a string length | 571.000ms | 596.282ms | 608.367ms | 0.4% | 608.192ms | 610.658ms | 607.841ms | 612.068ms | 12.320s | 1.74 GiB | 2.52 MiB | 164.37M/s |
| q29 | group by a regular expression | 1.522s | 1.562s | 1.574s | 2.6% | 1.564s | 1.605s | 1.534s | 1.607s | 37.800s | 2.84 GiB | 172.34 MiB | 63.53M/s |
| q30 | ninety sums over one column | 43.000ms | 80.565ms | 80.885ms | 0.5% | 80.870ms | 81.275ms | 80.528ms | 81.840ms | 500.000ms | 707.63 MiB | 52.00 KiB | 1.24G/s |
| q31 | group by two and several aggregates | 383.000ms | 430.995ms | 428.297ms | 4.0% | 411.539ms | 428.724ms | 408.574ms | 430.716ms | 8.680s | 2.28 GiB | 26.28 MiB | 233.48M/s |
| q32 | group by a high card pair | 468.000ms | 515.727ms | 503.505ms | 1.6% | 501.359ms | 509.377ms | 493.957ms | 528.123ms | 10.880s | 2.98 GiB | 24.77 MiB | 198.60M/s |
| q33 | group by a high card pair, unfiltered | 1.689s | 1.927s | 1.796s | 3.6% | 1.772s | 1.837s | 1.764s | 1.894s | 46.340s | 9.92 GiB | 508.00 KiB | 55.68M/s |
| q34 | group by a long string | 1.938s | 2.053s | 2.021s | 1.3% | 2.017s | 2.042s | 1.983s | 2.125s | 50.340s | 8.72 GiB | 3.63 MiB | 49.47M/s |
| q35 | group by a constant and a long string | 1.876s | 1.952s | 1.953s | 1.6% | 1.939s | 1.969s | 1.909s | 1.978s | 49.690s | 8.60 GiB | 8.01 MiB | 51.20M/s |
| q36 | group by four expressions | 322.000ms | 362.911ms | 363.673ms | 0.9% | 363.627ms | 366.904ms | 348.710ms | 368.359ms | 7.240s | 1.88 GiB | 3.95 MiB | 274.97M/s |
| q37 | date range and group by a URL | 112.000ms | 140.718ms | 146.559ms | 3.5% | 142.541ms | 147.670ms | 141.323ms | 148.621ms | 200.000ms | 487.49 MiB | 5.22 MiB | 682.30M/s |
| q38 | date range and group by a title | 73.000ms | 101.024ms | 101.547ms | 1.4% | 100.801ms | 102.216ms | 100.767ms | 102.292ms | 160.000ms | 237.79 MiB | none | 984.74M/s |
| q39 | date range, group by and offset | 78.000ms | 101.661ms | 101.028ms | 0.7% | 100.858ms | 101.521ms | 100.579ms | 101.731ms | 130.000ms | 247.43 MiB | none | 989.80M/s |
| q40 | date range, a case and a wide group by | 214.000ms | 241.264ms | 241.266ms | 0.1% | 241.242ms | 241.425ms | 241.204ms | 246.253ms | 400.000ms | 682.20 MiB | 8.17 MiB | 414.47M/s |
| q41 | date range with an IN and a hash | 22.000ms | 60.420ms | 60.786ms | 0.8% | 60.481ms | 60.995ms | 60.427ms | 61.766ms | 60.000ms | 199.49 MiB | 136.00 KiB | 1.65G/s |
| q42 | date range and a deep offset | 19.000ms | 40.585ms | 40.422ms | 0.5% | 40.403ms | 40.598ms | 40.350ms | 41.066ms | 70.000ms | 192.76 MiB | none | 2.47G/s |
| q43 | minute buckets over a date range | 16.000ms | 40.421ms | 40.459ms | 0.7% | 40.437ms | 40.728ms | 40.368ms | 40.968ms | 50.000ms | 181.70 MiB | none | 2.47G/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 23.555s by its own clock and 25.305s by ours, 26.018s cold, 561.750s of CPU, peak 9.92 GiB, 182.55M/s and 25.13 GiB/s.

Running it cost 7% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 106.21x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 12.968ms | 242.741ms | 100.662ms | 20.1% | 100.597ms | 120.795ms | 100.580ms | 122.142ms | 120.000ms | 74.81 MiB | 130.29 MiB | 993.40M/s |
| q2 | filtered count | 17.359ms | 123.106ms | 120.970ms | 1.2% | 120.855ms | 122.251ms | 120.747ms | 128.537ms | 130.000ms | 102.49 MiB | 1.92 MiB | 826.63M/s |
| q3 | three aggregates | 44.649ms | 140.988ms | 142.258ms | 0.3% | 141.971ms | 142.364ms | 141.167ms | 142.762ms | 510.000ms | 385.52 MiB | 800.00 KiB | 702.93M/s |
| q4 | average | 66.707ms | 185.181ms | 162.291ms | 0.9% | 161.836ms | 163.320ms | 160.937ms | 165.850ms | 890.000ms | 816.08 MiB | 231.84 MiB | 616.16M/s |
| q5 | count distinct, high card | 282.147ms | 386.232ms | 407.608ms | 4.4% | 390.513ms | 408.603ms | 383.275ms | 412.313ms | 6.530s | 1.33 GiB | none | 245.33M/s |
| q6 | count distinct, strings | 382.558ms | 556.369ms | 530.573ms | 0.4% | 530.088ms | 532.361ms | 515.440ms | 552.475ms | 9.370s | 2.58 GiB | 5.21 MiB | 188.47M/s |
| q7 | min and max of a date | 35.362ms | 121.900ms | 141.105ms | 0.2% | 141.026ms | 141.321ms | 120.759ms | 141.761ms | 280.000ms | 245.06 MiB | none | 708.67M/s |
| q8 | group by, low card | 26.791ms | 121.946ms | 121.592ms | 0.8% | 121.539ms | 122.482ms | 120.716ms | 140.742ms | 160.000ms | 111.05 MiB | none | 822.40M/s |
| q9 | group by and count distinct | 707.601ms | 831.042ms | 872.349ms | 3.7% | 857.138ms | 889.173ms | 834.288ms | 906.999ms | 18.580s | 3.47 GiB | 276.00 KiB | 114.63M/s |
| q10 | group by, several aggregates | 730.250ms | 892.270ms | 892.255ms | 0.5% | 888.235ms | 893.013ms | 870.189ms | 910.419ms | 18.740s | 3.70 GiB | none | 112.07M/s |
| q11 | group by a string and count distinct | 89.265ms | 201.205ms | 201.148ms | 7.8% | 185.504ms | 201.169ms | 180.977ms | 202.462ms | 1.280s | 668.12 MiB | 392.00 KiB | 497.13M/s |
| q12 | group by two strings and count distinct | 100.456ms | 201.302ms | 203.600ms | 0.2% | 203.531ms | 204.004ms | 201.480ms | 204.111ms | 1.460s | 681.20 MiB | 296.00 KiB | 491.15M/s |
| q13 | group by a string and top k | 315.606ms | 447.294ms | 466.107ms | 1.7% | 460.090ms | 467.933ms | 457.067ms | 468.564ms | 6.210s | 2.53 GiB | none | 214.54M/s |
| q14 | group by a string and count distinct | 757.201ms | 976.210ms | 939.930ms | 2.6% | 920.051ms | 944.869ms | 907.876ms | 947.833ms | 17.800s | 4.48 GiB | none | 106.39M/s |
| q15 | group by two columns and top k | 407.215ms | 555.231ms | 552.820ms | 2.7% | 552.664ms | 567.385ms | 547.571ms | 571.882ms | 8.840s | 2.63 GiB | 960.00 KiB | 180.89M/s |
| q16 | group by, very high card | 344.694ms | 508.398ms | 486.537ms | 0.5% | 486.083ms | 488.704ms | 479.298ms | 491.053ms | 8.110s | 1.85 GiB | none | 205.53M/s |
| q17 | group by two, very high card | 1.132s | 1.376s | 1.368s | 2.5% | 1.348s | 1.382s | 1.346s | 1.393s | 30.220s | 6.04 GiB | none | 73.10M/s |
| q18 | group by two, no ordering | 1.126s | 1.333s | 1.352s | 1.9% | 1.333s | 1.359s | 1.324s | 1.361s | 29.990s | 6.06 GiB | none | 73.96M/s |
| q19 | group by with an extract | 1.855s | 2.244s | 2.154s | 2.4% | 2.135s | 2.187s | 2.127s | 2.206s | 51.030s | 9.13 GiB | 5.65 MiB | 46.43M/s |
| q20 | point lookup | 31.037ms | 141.250ms | 121.787ms | 0.5% | 121.296ms | 121.938ms | 121.182ms | 141.685ms | 240.000ms | 421.19 MiB | 10.90 MiB | 821.09M/s |
| q21 | substring scan | 999.977ms | 1.119s | 1.128s | 6.2% | 1.060s | 1.130s | 1.055s | 1.146s | 14.070s | 2.89 GiB | 581.07 MiB | 88.65M/s |
| q22 | substring scan and group by | 960.149ms | 1.228s | 1.108s | 1.4% | 1.097s | 1.112s | 1.083s | 1.112s | 16.010s | 4.03 GiB | 384.00 KiB | 90.26M/s |
| q23 | two substring scans and group by | 1.940s | 2.253s | 2.190s | 4.7% | 2.141s | 2.244s | 1.919s | 2.271s | 36.030s | 12.09 GiB | none | 45.66M/s |
| q24 | select star and top k | 1.383s | 1.627s | 1.576s | 5.4% | 1.509s | 1.595s | 1.400s | 1.647s | 26.320s | 12.89 GiB | 616.28 MiB | 63.45M/s |
| q25 | top k by a date | 181.565ms | 285.708ms | 321.839ms | 5.7% | 304.227ms | 322.529ms | 301.787ms | 338.421ms | 2.580s | 1.72 GiB | none | 310.71M/s |
| q26 | top k by a string | 136.744ms | 242.088ms | 241.831ms | 8.5% | 241.581ms | 262.165ms | 241.265ms | 262.566ms | 1.680s | 946.81 MiB | 580.00 KiB | 413.50M/s |
| q27 | top k by two columns | 242.096ms | 387.020ms | 386.237ms | 5.2% | 368.903ms | 388.868ms | 363.671ms | 404.034ms | 4.430s | 2.26 GiB | none | 258.90M/s |
| q30 | ninety sums over one column | 113.748ms | 246.293ms | 203.927ms | 0.1% | 203.882ms | 204.141ms | 203.310ms | 224.146ms | 2.690s | 277.81 MiB | none | 490.36M/s |
| q31 | group by two and several aggregates | 355.848ms | 475.797ms | 492.564ms | 4.9% | 488.786ms | 512.799ms | 487.194ms | 523.223ms | 6.840s | 1.83 GiB | none | 203.01M/s |
| q32 | group by a high card pair | 455.319ms | 608.366ms | 621.339ms | 0.6% | 619.245ms | 622.939ms | 604.366ms | 628.580ms | 7.650s | 2.77 GiB | none | 160.94M/s |
| q33 | group by a high card pair, unfiltered | 2.122s | 2.904s | 2.548s | 5.4% | 2.512s | 2.649s | 2.349s | 2.767s | 49.770s | 13.37 GiB | 12.00 KiB | 39.24M/s |
| q34 | group by a long string | 1.900s | 2.443s | 2.383s | 0.5% | 2.372s | 2.383s | 2.299s | 2.548s | 38.040s | 16.66 GiB | none | 41.95M/s |
| q35 | group by a constant and a long string | 2.602s | 3.096s | 2.981s | 4.6% | 2.978s | 3.115s | 2.965s | 3.121s | 66.480s | 13.82 GiB | none | 33.55M/s |
| q37 | date range and group by a URL | 91.612ms | 201.189ms | 201.216ms | 0.1% | 201.158ms | 201.352ms | 201.095ms | 201.362ms | 440.000ms | 380.02 MiB | 11.25 MiB | 496.97M/s |
| q38 | date range and group by a title | 64.179ms | 160.946ms | 161.083ms | 12.4% | 161.001ms | 181.046ms | 160.950ms | 181.121ms | 260.000ms | 219.47 MiB | 16.22 MiB | 620.78M/s |
| q39 | date range, group by and offset | 47.960ms | 140.911ms | 160.914ms | 12.5% | 140.847ms | 161.033ms | 140.814ms | 161.073ms | 200.000ms | 122.43 MiB | 64.00 KiB | 621.43M/s |
| q40 | date range, a case and a wide group by | 131.022ms | 281.888ms | 261.472ms | 7.6% | 241.547ms | 261.488ms | 241.341ms | 273.578ms | 790.000ms | 787.81 MiB | 27.07 MiB | 382.44M/s |
| q41 | date range with an IN and a hash | 29.992ms | 140.748ms | 140.830ms | 0.1% | 140.733ms | 140.839ms | 127.679ms | 155.466ms | 220.000ms | 117.99 MiB | 10.42 MiB | 710.06M/s |
| q42 | date range and a deep offset | 25.937ms | 141.115ms | 123.298ms | 6.0% | 122.881ms | 130.263ms | 120.744ms | 141.758ms | 180.000ms | 106.81 MiB | 804.00 KiB | 811.02M/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 22.248s by its own clock and 28.568s by ours, 29.569s cold, 485.170s of CPU, peak 16.66 GiB, 175.29M/s and 24.13 GiB/s.

Running it cost 28% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 29.61x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 39.381ms | 60.598ms | 60.443ms | 0.1% | 60.435ms | 60.474ms | 60.427ms | 69.937ms | 270.000ms | 21.47 MiB | 9.37 MiB | 1.65G/s |
| q2 | filtered count | 60.445ms | 80.581ms | 80.828ms | 0.2% | 80.710ms | 80.844ms | 80.682ms | 82.487ms | 580.000ms | 84.41 MiB | 12.00 KiB | 1.24G/s |
| q3 | three aggregates | 80.815ms | 100.778ms | 102.296ms | 2.9% | 100.959ms | 103.889ms | 100.935ms | 104.216ms | 890.000ms | 124.50 MiB | 61.57 MiB | 977.53M/s |
| q4 | average | 67.023ms | 81.544ms | 81.620ms | 1.2% | 80.787ms | 81.755ms | 80.764ms | 84.374ms | 710.000ms | 94.11 MiB | 13.46 MiB | 1.23G/s |
| q5 | count distinct, high card | 443.460ms | 451.761ms | 468.215ms | 1.3% | 467.428ms | 473.711ms | 466.300ms | 484.677ms | 5.000s | 3.12 GiB | none | 213.57M/s |
| q6 | count distinct, strings | 1.355s | 1.447s | 1.406s | 4.3% | 1.387s | 1.447s | 1.329s | 1.467s | 8.210s | 1.12 GiB | 101.21 MiB | 71.11M/s |
| q7 | min and max of a date | 74.924ms | 100.818ms | 100.944ms | 0.2% | 100.771ms | 100.969ms | 100.728ms | 102.053ms | 810.000ms | 82.04 MiB | 10.45 MiB | 990.62M/s |
| q8 | group by, low card | 63.131ms | 81.190ms | 80.825ms | 1.2% | 80.757ms | 81.733ms | 80.680ms | 81.735ms | 650.000ms | 86.32 MiB | none | 1.24G/s |
| q9 | group by and count distinct | 539.975ms | 588.469ms | 588.127ms | 2.7% | 573.733ms | 589.735ms | 569.752ms | 592.375ms | 6.410s | 3.13 GiB | 4.66 MiB | 170.03M/s |
| q10 | group by, several aggregates | 683.560ms | 792.310ms | 723.981ms | 8.5% | 684.375ms | 745.573ms | 683.669ms | 752.086ms | 8.630s | 3.23 GiB | none | 138.12M/s |
| q11 | group by a string and count distinct | 311.076ms | 326.470ms | 327.159ms | 1.0% | 325.330ms | 328.561ms | 322.363ms | 365.413ms | 4.200s | 182.39 MiB | 1.12 MiB | 305.65M/s |
| q12 | group by two strings and count distinct | 327.450ms | 362.070ms | 345.654ms | 5.7% | 342.475ms | 362.260ms | 342.436ms | 379.530ms | 4.450s | 215.73 MiB | 252.00 KiB | 289.30M/s |
| q13 | group by a string and top k | 692.525ms | 710.233ms | 725.560ms | 2.7% | 707.750ms | 727.438ms | 705.721ms | 729.877ms | 7.800s | 830.70 MiB | none | 137.82M/s |
| q14 | group by a string and count distinct | 1.371s | 1.437s | 1.417s | 5.6% | 1.338s | 1.418s | 1.293s | 1.426s | 16.260s | 1.33 GiB | none | 70.55M/s |
| q15 | group by two columns and top k | 901.690ms | 944.454ms | 926.156ms | 2.8% | 926.037ms | 952.410ms | 924.844ms | 972.662ms | 9.820s | 1.07 GiB | 560.00 KiB | 107.97M/s |
| q16 | group by, very high card | 844.337ms | 879.596ms | 875.428ms | 0.2% | 874.711ms | 876.449ms | 872.503ms | 879.804ms | 11.140s | 1.93 GiB | none | 114.23M/s |
| q17 | group by two, very high card | 1.784s | 1.878s | 1.818s | 5.0% | 1.810s | 1.900s | 1.809s | 1.910s | 17.550s | 2.23 GiB | none | 55.02M/s |
| q18 | group by two, no ordering | 413.350ms | 422.774ms | 428.700ms | 4.9% | 425.325ms | 446.527ms | 422.521ms | 451.104ms | 6.040s | 189.75 MiB | none | 233.26M/s |
| q19 | group by with an extract | 3.012s | 3.027s | 3.110s | 1.6% | 3.066s | 3.114s | 3.037s | 3.114s | 27.510s | 4.63 GiB | 36.83 MiB | 32.15M/s |
| q20 | point lookup | 172.867ms | 181.581ms | 201.410ms | 0.7% | 201.361ms | 202.837ms | 181.840ms | 205.475ms | 2.240s | 101.70 MiB | none | 496.49M/s |
| q21 | substring scan | 643.581ms | 670.350ms | 671.718ms | 6.3% | 645.432ms | 687.493ms | 644.071ms | 711.561ms | 9.650s | 126.02 MiB | none | 148.87M/s |
| q22 | substring scan and group by | 938.649ms | 940.627ms | 956.479ms | 0.8% | 954.951ms | 962.813ms | 918.758ms | 980.875ms | 14.110s | 208.67 MiB | none | 104.55M/s |
| q23 | two substring scans and group by | 2.424s | 2.319s | 2.440s | 4.6% | 2.349s | 2.462s | 2.346s | 2.475s | 36.890s | 360.82 MiB | 1.32 GiB | 40.99M/s |
| q24 | select star and top k | 1.078s | 1.106s | 1.106s | 7.2% | 1.028s | 1.107s | 1.015s | 1.152s | 16.200s | 220.46 MiB | 144.83 MiB | 90.44M/s |
| q25 | top k by a date | 470.617ms | 503.378ms | 503.560ms | 0.2% | 503.408ms | 504.404ms | 484.143ms | 509.759ms | 6.970s | 198.97 MiB | none | 198.58M/s |
| q26 | top k by a string | 282.618ms | 307.830ms | 306.438ms | 1.3% | 305.963ms | 309.930ms | 304.454ms | 310.794ms | 4.030s | 155.88 MiB | none | 326.32M/s |
| q27 | top k by two columns | 508.978ms | 531.824ms | 530.531ms | 0.8% | 529.938ms | 534.180ms | 515.121ms | 536.793ms | 7.580s | 201.15 MiB | none | 188.49M/s |
| q28 | group by with a string length | 830.027ms | 869.616ms | 844.681ms | 2.4% | 844.609ms | 864.883ms | 827.748ms | 865.036ms | 12.510s | 153.31 MiB | none | 118.38M/s |
| q29 | group by a regular expression | 2.157s | 2.110s | 2.213s | 1.0% | 2.192s | 2.214s | 2.164s | 2.263s | 27.980s | 1.50 GiB | 2.49 GiB | 45.18M/s |
| q30 | ninety sums over one column | 71.915ms | 100.736ms | 100.712ms | 0.0% | 100.699ms | 100.744ms | 100.693ms | 101.203ms | 720.000ms | 79.72 MiB | none | 992.90M/s |
| q31 | group by two and several aggregates | 581.412ms | 623.888ms | 624.030ms | 3.0% | 605.478ms | 624.340ms | 605.290ms | 647.314ms | 8.300s | 844.04 MiB | 11.66 MiB | 160.24M/s |
| q32 | group by a high card pair | 793.100ms | 785.290ms | 819.011ms | 2.4% | 809.185ms | 828.817ms | 765.580ms | 839.105ms | 11.790s | 1.09 GiB | 98.51 MiB | 122.10M/s |
| q33 | group by a high card pair, unfiltered | 1.816s | 1.957s | 1.848s | 10.2% | 1.661s | 1.850s | 1.615s | 1.864s | 26.180s | 5.73 GiB | none | 54.10M/s |
| q34 | group by a long string | 2.528s | 2.570s | 2.567s | 2.5% | 2.515s | 2.579s | 2.497s | 2.598s | 27.730s | 4.37 GiB | none | 38.95M/s |
| q35 | group by a constant and a long string | 2.580s | 2.610s | 2.623s | 2.5% | 2.580s | 2.645s | 2.570s | 2.665s | 28.330s | 4.37 GiB | none | 38.13M/s |
| q36 | group by four expressions | 712.016ms | 762.636ms | 745.042ms | 5.0% | 744.211ms | 781.437ms | 737.630ms | 814.683ms | 9.870s | 1.25 GiB | none | 134.22M/s |
| q37 | date range and group by a URL | 130.287ms | 160.905ms | 160.966ms | 12.1% | 141.663ms | 161.140ms | 140.828ms | 161.523ms | 240.000ms | 129.64 MiB | none | 621.23M/s |
| q38 | date range and group by a title | 101.015ms | 120.759ms | 120.740ms | 0.1% | 120.738ms | 120.872ms | 120.712ms | 142.406ms | 200.000ms | 60.49 MiB | none | 828.20M/s |
| q39 | date range, group by and offset | 80.065ms | 100.787ms | 100.766ms | 0.2% | 100.671ms | 100.872ms | 100.582ms | 102.102ms | 180.000ms | 68.04 MiB | none | 992.38M/s |
| q40 | date range, a case and a wide group by | 356.841ms | 361.651ms | 382.317ms | 0.1% | 382.180ms | 382.745ms | 381.901ms | 383.300ms | 660.000ms | 279.50 MiB | none | 261.56M/s |
| q41 | date range with an IN and a hash | 79.702ms | 100.784ms | 100.814ms | 0.1% | 100.814ms | 100.880ms | 100.792ms | 100.886ms | 150.000ms | 73.94 MiB | none | 991.90M/s |
| q42 | date range and a deep offset | 57.841ms | 80.846ms | 80.689ms | 0.3% | 80.635ms | 80.859ms | 80.610ms | 80.859ms | 100.000ms | 71.92 MiB | none | 1.24G/s |
| q43 | minute buckets over a date range | 52.077ms | 80.662ms | 80.779ms | 0.1% | 80.719ms | 80.815ms | 80.674ms | 80.815ms | 90.000ms | 58.25 MiB | none | 1.24G/s |

rudb rudb 0.3.45 over 43 of 43 queries. Total 32.512s by its own clock and 33.795s by ours, 33.730s cold, 389.630s of CPU, peak 5.73 GiB, 132.25M/s and 18.21 GiB/s.

Running it cost 4% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 51.46x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | 11.781ms | 19.142ms | 211.446ms | 211.452ms | 0.0% | 90.908ms | 5.270ms | 23.278ms | 896 B | 4 of 4 |
| q2 | 12.479ms | 36.995ms | 515.441ms | 515.449ms | 0.0% | 111.875ms | 5.352ms | 19.199ms | 896 B | 5 of 5 |
| q3 | 11.810ms | 67.288ms | 889.393ms | 889.403ms | 0.0% | 93.248ms | 5.391ms | 15.206ms | 1.22 KiB | 4 of 4 |
| q4 | 11.701ms | 48.008ms | 678.349ms | 678.357ms | 0.0% | 93.505ms | 5.287ms | 26.356ms | 896 B | 4 of 4 |
| q5 | 11.781ms | 362.168ms | 2.430s | 2.430s | 0.0% | 99.482ms | 5.356ms | 2.375s | 3.03 GiB | 4 of 4 |
| q6 | 12.658ms | 1.376s | 8.217s | 8.217s | 0.0% | 129.810ms | 5.771ms | 377.201ms | 1.23 GiB | 5 of 5 |
| q7 | 11.850ms | 53.348ms | 780.014ms | 780.019ms | 0.0% | 98.860ms | 5.347ms | 14.634ms | 1.00 KiB | 4 of 4 |
| q8 | 11.677ms | 45.023ms | 626.169ms | 626.180ms | 0.0% | 118.395ms | 5.264ms | 18.556ms | 19.28 KiB | 6 of 6 |
| q9 | 11.713ms | 529.344ms | 4.050s | 4.050s | 0.0% | 105.481ms | 5.377ms | 2.435s | 3.41 GiB | 5 of 5 |
| q10 | 12.452ms | 742.040ms | 7.050s | 7.050s | 0.0% | 116.075ms | 5.940ms | 2.524s | 3.42 GiB | 5 of 5 |
| q11 | 11.757ms | 292.223ms | 4.209s | 4.209s | 0.0% | 129.103ms | 5.393ms | 15.651ms | 22.16 MiB | 6 of 6 |
| q12 | 11.497ms | 307.453ms | 4.440s | 4.440s | 0.0% | 117.326ms | 5.229ms | 24.668ms | 22.77 MiB | 6 of 6 |
| q13 | 11.753ms | 661.686ms | 7.521s | 7.521s | 0.0% | 153.233ms | 5.489ms | 93.038ms | 814.13 MiB | 6 of 6 |
| q14 | 14.480ms | 1.385s | 15.948s | 15.948s | 0.0% | 193.124ms | 7.024ms | 594.785ms | 1.41 GiB | 6 of 6 |
| q15 | 11.829ms | 884.227ms | 9.688s | 9.688s | 0.0% | 162.401ms | 5.426ms | 136.587ms | 1.07 GiB | 6 of 6 |
| q16 | 13.974ms | 814.792ms | 7.637s | 7.637s | 0.0% | 105.365ms | 7.370ms | 3.425s | 1.21 GiB | 5 of 5 |
| q17 | 12.051ms | 1.817s | 18.109s | 18.109s | 0.0% | 152.662ms | 5.563ms | 335.456ms | 2.28 GiB | 5 of 5 |
| q18 | 11.986ms | 370.162ms | 5.641s | 5.641s | 0.0% | 92.623ms | 5.443ms | 23.852ms | 17.00 KiB | 5 of 5 |
| q19 | 12.366ms | 2.947s | 26.536s | 26.536s | 0.0% | 166.784ms | 5.823ms | 737.808ms | 4.25 GiB | 5 of 5 |
| q20 | 11.955ms | 144.620ms | 2.010s | 2.010s | 0.0% | 103.735ms | 5.209ms | 44.779ms | 192 B | 4 of 4 |
| q21 | 24.275ms | 621.256ms | 9.587s | 9.587s | 0.0% | 124.671ms | 17.312ms | 25.938ms | 896 B | 5 of 5 |
| q22 | 12.551ms | 899.637ms | 14.013s | 14.013s | 0.0% | 137.641ms | 6.143ms | 20.971ms | 462.42 KiB | 6 of 6 |
| q23 | 12.345ms | 2.286s | 34.739s | 34.739s | 0.0% | 163.261ms | 5.596ms | 25.836ms | 2.63 MiB | 6 of 6 |
| q24 | 12.034ms | 1.058s | 15.433s | 15.433s | 0.0% | 187.668ms | 5.448ms | 631.220ms | 43.03 KiB | 8 of 8 |
| q25 | 16.467ms | 444.913ms | 6.875s | 6.875s | 0.0% | 153.622ms | 8.888ms | 25.820ms | 53.02 KiB | 6 of 6 |
| q26 | 11.702ms | 267.092ms | 3.982s | 3.982s | 0.0% | 148.443ms | 5.324ms | 22.438ms | 46.36 KiB | 5 of 5 |
| q27 | 12.472ms | 489.322ms | 7.485s | 7.485s | 0.0% | 169.078ms | 5.328ms | 20.105ms | 74.81 KiB | 6 of 6 |
| q28 | 12.651ms | 825.531ms | 12.653s | 12.653s | 0.0% | 134.594ms | 5.612ms | 21.792ms | 1.01 MiB | 7 of 7 |
| q29 | 11.741ms | 2.050s | 24.701s | 24.701s | 0.0% | 175.506ms | 5.256ms | 2.204s | 1.02 GiB | 7 of 7 |
| q30 | 13.330ms | 51.281ms | 697.162ms | 697.178ms | 0.0% | 91.028ms | 5.467ms | 37.355ms | 29.59 KiB | 4 of 4 |
| q31 | 11.885ms | 570.621ms | 7.109s | 7.109s | 0.0% | 133.690ms | 5.356ms | 1.365s | 787.99 MiB | 6 of 6 |
| q32 | 11.793ms | 733.517ms | 9.282s | 9.282s | 0.0% | 133.765ms | 5.230ms | 1.413s | 785.99 MiB | 6 of 6 |
| q33 | 11.912ms | 1.865s | 9.912s | 9.912s | 0.0% | 114.199ms | 5.432ms | 17.662s | 5.98 GiB | 5 of 5 |
| q34 | 11.734ms | 2.522s | 27.747s | 27.747s | 0.0% | 174.907ms | 5.258ms | 357.533ms | 6.04 GiB | 5 of 5 |
| q35 | 15.371ms | 2.559s | 27.959s | 27.959s | 0.0% | 172.593ms | 7.282ms | 353.610ms | 5.55 GiB | 5 of 5 |
| q36 | 11.809ms | 704.742ms | 8.014s | 8.014s | 0.0% | 107.274ms | 5.276ms | 2.050s | 892.84 MiB | 6 of 6 |
| q37 | 11.695ms | 112.743ms | 214.337ms | 214.345ms | 0.0% | 2.819ms | 5.203ms | 20.452ms | 69.15 MiB | 6 of 6 |
| q38 | 11.740ms | 80.413ms | 163.940ms | 163.946ms | 0.0% | 9.863ms | 5.298ms | 40.756ms | 6.47 MiB | 6 of 6 |
| q39 | 12.072ms | 58.022ms | 135.823ms | 135.829ms | 0.0% | 2.225ms | 5.278ms | 18.893ms | 3.79 MiB | 6 of 6 |
| q40 | 11.779ms | 316.675ms | 578.023ms | 578.028ms | 0.0% | 5.579ms | 5.213ms | 46.759ms | 196.36 MiB | 6 of 6 |
| q41 | 14.970ms | 60.369ms | 120.571ms | 120.583ms | 0.0% | 2.477ms | 6.579ms | 32.838ms | 3.57 MiB | 6 of 6 |
| q42 | 12.674ms | 37.361ms | 71.508ms | 71.517ms | 0.0% | 2.547ms | 5.298ms | 23.185ms | 3.33 MiB | 6 of 6 |
| q43 | 15.575ms | 31.527ms | 70.105ms | 70.114ms | 0.0% | 2.139ms | 8.962ms | 20.924ms | 407.06 KiB | 6 of 6 |

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
| Aggregate | 155.948s | 45.3% | 39 | 1968178868 | 6056627 | 79.2ns | 25748.4ns | 39 of 39 |
| FileScan | 135.824s | 39.5% | 43 | 0 | 3597060701 | handed none | 37.8ns | 43 of 43 |
| Filter | 46.540s | 13.5% | 28 | 1897103429 | 301735584 | 24.5ns | 154.2ns | 28 of 28 |
| Project | 4.989s | 1.5% | 91 | 2041263671 | 2041263671 | 2.4ns | 2.4ns | 91 of 91 |
| TopN | 637.008ms | 0.2% | 31 | 39570574 | 340 | 16.1ns | 1873553.9ns | 31 of 31 |
| Fetch | 7.962ms | 0.0% | 1 | 10 | 10 | 796246.2ns | 796246.2ns | 1 of 1 |
| Sort | 16.855us | 0.0% | 1 | 18 | 18 | 936.4ns | 936.4ns | 1 of 1 |
| Limit | 0.404us | 0.0% | 1 | 10 | 10 | 40.4ns | 40.4ns | 1 of 1 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is Aggregate, and the queries where it cost the most are q19 at 21.937s, q35 at 18.400s, q34 at 18.086s.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- the one minute load average was 79.97 before this suite started, on a machine with 32 hardware threads, so this was measured against somebody else's work
- q4 swung by 24.6% of its median, and rule two wants under 10%
- q12 swung by 14.6% of its median, and rule two wants under 10%
- q14 swung by 44.7% of its median, and rule two wants under 10%

These swung wider than reporting rule two allows:

- datafusion swung by 44.7% of its median on q14, and rule two wants under 10%
- polars swung by 20.1% of its median on q1, and rule two wants under 10%
- rudb swung by 12.1% of its median on q37, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

polars did not run q28, because the SQLContext has no strlen and no character length function.

polars did not run q29, because the SQLContext has no regexp_replace.

polars did not run q36, because the SQLContext rejects the repeated ClientIP output name in the group by.

polars did not run q43, because the SQLContext has no date_trunc.

So the polars column is 39 of 43 queries and its ratio is over the shared ones.

Answers differ, so this is not a comparison: q25: polars does not agree with datafusion: the same 0 numbers and 1 of 10 text fields different, so the disagreement is which rows came back rather than what they hold

These were answered differently and the data does not say which is right:

- q16: ORDER BY COUNT(*) DESC LIMIT 10 over user IDs, where the tenth place is a tie between many users with the same count and the engine picks.
- q17: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q18: LIMIT 10 with no ORDER BY at all, so the ten rows are whichever ten the group by handed back first and the query did not ask for any particular ten.
- q19: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q24: ORDER BY EventTime LIMIT 10, and EventTime has one second resolution over a corpus with millions of rows a second, so the tenth row is a tie.
- q31: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q32: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q33: ORDER BY COUNT(*) DESC LIMIT 10 over WatchID and ClientIP unfiltered, where almost every group has a count of one and the ten that come back are arbitrary.
- q39: ORDER BY PageViews DESC LIMIT 10 OFFSET 1000, which is a tie at the cut a thousand rows deeper in, where the counts are smaller and the ties are denser.
- q40: ORDER BY PageViews DESC LIMIT 10 OFFSET 1000 with ties at the cut, as q39.
- q41: ORDER BY PageViews DESC LIMIT 10 OFFSET 100 with ties at the cut, as q39.
- q42: ORDER BY PageViews DESC LIMIT 10 OFFSET 10000 with ties at the cut, as q39.
- q43: DATE_TRUNC on a timestamp, and ClickHouse renders a DateTime in the machine's timezone where DuckDB renders a TIMESTAMP in none, so the same epoch second prints seven hours apart on gamingpc-wsl and two hours apart on server3.

So they are not checked, and a wrong answer from any engine on one of them would go unnoticed here. Every other query in the suite is checked in full.

Hot here means page cache warm and not buffer pool warm, because every run is a fresh process so that no query's number depends on the one before it.

