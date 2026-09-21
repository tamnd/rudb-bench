# clickbench on gamingpc-wsl

This is one run of the clickbench suite on gamingpc-wsl, over 6 engines and 43 queries, with 5 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

It ran over 9999750 rows, one out of every 10 of the 99997497 in the full file, which is a development loop and not the suite. Nothing here is comparable to a full run or to anybody else's number.

## What ran

| what | it was |
| --- | --- |
| suite | clickbench |
| queries | 43 |
| tables | 1.87 GiB of Parquet in 1 table |
| rows | 9999750 in the table every query reads |
| sample | 9999750 rows, one out of every 10 of the 99997497 in the full file |
| corpus | no manifest beside the data, so this run cannot say where it came from |
| summary | median with the interquartile range, per reporting rule two |
| timeout | 600s per query, and no query reached it |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --rows 10000000 --engines duckdb,duckdb-pinned,clickhouse-local,datafusion,polars,rudb --runs 5 --timeout 600 --report
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
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 11.060s | 87.710s | 2.73 GiB | its own database file | its own | 3.65 to 12.13 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 17.971s | 70.380s | 2.32 GiB | its own database file | its own | 11.32 to 14.65 |
| clickhouse-local | 26.9.1.1562 | ran | 3.557s | 29.720s | 2.00 GiB | its own MergeTree parts, as system.parts counts the active ones | its own | 14.65 to 12.47 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 1.87 GiB | the source Parquet, this engine has no storage format of its own | the Parquet | 12.47 to 16.43 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 1.87 GiB | the source Parquet, this engine has no storage format of its own | the Parquet | 16.43 to 14.10 |
| rudb | rudb 0.3.45 | ran | 0.000us | 0.000us | 1.87 GiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 14.10 to 4.39 |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

These engines started while the machine was already above half of that, which means they were measured against somebody else's work rather than on an idle box: polars. Their numbers are inflated by an amount nothing here can recover, and by a different amount each, depending on how much of the column was CPU bound. One case where that reading is unfair is a sweep that runs engines back to back, where the average has not had a minute to come down from the engine before.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb on 39 shared |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 3.902s | 5.023s | +29% | 5.251s | 42.290s | 8.42 | 1.72 GiB | none | 110.20M/s | 20.61 GiB/s | 1.00x |
| duckdb-pinned | 3.364s | 5.079s | +51% | 5.212s | 39.860s | 7.85 | 1.58 GiB | none | 127.82M/s | 23.90 GiB/s | 1.00x |
| clickhouse-local | 8.394s | 17.556s | +109% | 17.683s | 63.160s | 3.60 | 1.40 GiB | none | 51.23M/s | 9.58 GiB/s | 2.84x |
| datafusion | 13.110s | 17.026s | +30% | 16.926s | 252.610s | 14.84 | 4.55 GiB | none | 32.80M/s | 6.13 GiB/s | 4.38x |
| polars | 5.030s | 9.446s | +88% | 9.550s | 58.060s | 6.15 | 2.59 GiB | none | 77.53M/s | 14.50 GiB/s | 1.87x |
| rudb | 10.263s | 13.175s | +28% | 13.375s | 53.530s | 4.06 | 1.16 GiB | none | 41.90M/s | 7.83 GiB/s | 3.39x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 2.000ms | 6.000ms | 1.000ms | 39.278ms | 147.818ms |
| q2 | filtered count | 7.000ms | 7.000ms | 6.000ms | 253.000ms | 45.085ms | 156.011ms |
| q3 | three aggregates | 10.000ms | 10.000ms | 85.000ms | 268.000ms | 56.658ms | 164.342ms |
| q4 | average | 14.000ms | 15.000ms | 77.000ms | 255.000ms | 54.426ms | 153.940ms |
| q5 | count distinct, high card | 54.000ms | 51.000ms | 151.000ms | 294.000ms | 103.978ms | 182.859ms |
| q6 | count distinct, strings | 36.000ms | 35.000ms | 127.000ms | 296.000ms | 100.856ms | 268.683ms |
| q7 | min and max of a date | 2.000ms | 2.000ms | 230.000ms | 1.000ms | 50.918ms | 169.946ms |
| q8 | group by, low card | 7.000ms | 14.000ms | 278.000ms | 253.000ms | 52.453ms | 162.667ms |
| q9 | group by and count distinct | 69.000ms | 69.000ms | 209.000ms | 315.000ms | 163.237ms | 215.101ms |
| q10 | group by, several aggregates | 102.000ms | 122.000ms | 189.000ms | 359.000ms | 178.072ms | 224.549ms |
| q11 | group by a string and count distinct | 26.000ms | 26.000ms | 138.000ms | 291.000ms | 86.799ms | 211.495ms |
| q12 | group by two strings and count distinct | 30.000ms | 28.000ms | 134.000ms | 290.000ms | 90.017ms | 205.859ms |
| q13 | group by a string and top k | 35.000ms | 50.000ms | 130.000ms | 294.000ms | 105.529ms | 221.413ms |
| q14 | group by a string and count distinct | 60.000ms | 104.000ms | 187.000ms | 356.000ms | 149.414ms | 256.614ms |
| q15 | group by two columns and top k | 40.000ms | 48.000ms | 160.000ms | 299.000ms | 104.318ms | 236.472ms |
| q16 | group by, very high card | 65.000ms | 61.000ms | 106.000ms | 305.000ms | 122.739ms | 300.164ms |
| q17 | group by two, very high card | 164.000ms | 153.000ms | 239.000ms | 389.000ms | 201.027ms | 377.346ms |
| q18 | group by two, no ordering | 139.000ms | 128.000ms | 155.000ms | 397.000ms | 186.965ms | 211.995ms |
| q19 | group by with an extract | 229.000ms | 210.000ms | 305.000ms | 436.000ms | 235.836ms | 524.595ms |
| q20 | point lookup | 13.000ms | 14.000ms | 292.000ms | 256.000ms | 46.544ms | 144.012ms |
| q21 | substring scan | 99.000ms | 85.000ms | 180.000ms | 303.000ms | 173.493ms | 263.985ms |
| q22 | substring scan and group by | 131.000ms | 120.000ms | 211.000ms | 337.000ms | 189.969ms | 301.148ms |
| q23 | two substring scans and group by | 201.000ms | 184.000ms | 246.000ms | 417.000ms | 270.544ms | 384.834ms |
| q24 | select star and top k | 167.000ms | 135.000ms | 440.000ms | 634.000ms | 513.086ms | 299.672ms |
| q25 | top k by a date | 15.000ms | 16.000ms | 300.000ms | 257.000ms | 84.938ms | 203.648ms |
| q26 | top k by a string | 18.000ms | 24.000ms | 102.000ms | 278.000ms | 77.772ms | 177.729ms |
| q27 | top k by two columns | 13.000ms | 14.000ms | 291.000ms | 255.000ms | 90.520ms | 194.800ms |
| q28 | group by with a string length | 111.000ms | 114.000ms | 109.000ms | 315.000ms | no dialect | 286.060ms |
| q29 | group by a regular expression | 1.023s | 500.000ms | 264.000ms | 457.000ms | no dialect | 421.348ms |
| q30 | ninety sums over one column | 11.000ms | 22.000ms | 78.000ms | 261.000ms | 77.626ms | 157.424ms |
| q31 | group by two and several aggregates | 71.000ms | 88.000ms | 139.000ms | 354.000ms | 109.781ms | 217.033ms |
| q32 | group by a high card pair | 61.000ms | 96.000ms | 149.000ms | 331.000ms | 111.441ms | 207.585ms |
| q33 | group by a high card pair, unfiltered | 154.000ms | 184.000ms | 232.000ms | 425.000ms | 235.049ms | 365.701ms |
| q34 | group by a long string | 291.000ms | 242.000ms | 249.000ms | 454.000ms | 282.954ms | 472.975ms |
| q35 | group by a constant and a long string | 302.000ms | 254.000ms | 258.000ms | 465.000ms | 321.306ms | 472.535ms |
| q36 | group by four expressions | 66.000ms | 58.000ms | 97.000ms | 286.000ms | no dialect | 255.172ms |
| q37 | date range and group by a URL | 13.000ms | 15.000ms | 273.000ms | 239.000ms | 56.302ms | 155.178ms |
| q38 | date range and group by a title | 8.000ms | 7.000ms | 269.000ms | 245.000ms | 52.502ms | 145.846ms |
| q39 | date range, group by and offset | 7.000ms | 11.000ms | 262.000ms | 239.000ms | 49.602ms | 149.809ms |
| q40 | date range, a case and a wide group by | 20.000ms | 20.000ms | 276.000ms | 237.000ms | 63.073ms | 165.891ms |
| q41 | date range with an IN and a hash | 6.000ms | 9.000ms | 257.000ms | 237.000ms | 48.407ms | 147.534ms |
| q42 | date range and a deep offset | 6.000ms | 10.000ms | 253.000ms | 239.000ms | 47.583ms | 135.014ms |
| q43 | minute buckets over a date range | 5.000ms | 7.000ms | 255.000ms | 237.000ms | no dialect | 146.486ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 20.240ms | 20.251ms | 0.0% | 20.244ms | 20.253ms | 20.243ms | 20.259ms | 0.000us | 27.74 MiB | none | 493.80M/s |
| q2 | filtered count | 7.000ms | 40.334ms | 20.276ms | 0.0% | 20.270ms | 20.276ms | 20.265ms | 20.320ms | 30.000ms | 51.33 MiB | 4.32 MiB | 493.19M/s |
| q3 | three aggregates | 10.000ms | 40.374ms | 20.261ms | 99.2% | 20.253ms | 40.349ms | 20.247ms | 40.571ms | 60.000ms | 72.48 MiB | 4.30 MiB | 493.54M/s |
| q4 | average | 14.000ms | 60.805ms | 40.374ms | 0.1% | 40.351ms | 40.394ms | 40.351ms | 40.617ms | 80.000ms | 99.14 MiB | 16.21 MiB | 247.68M/s |
| q5 | count distinct, high card | 54.000ms | 81.847ms | 80.749ms | 2.0% | 80.611ms | 82.209ms | 80.564ms | 101.643ms | 990.000ms | 360.78 MiB | none | 123.84M/s |
| q6 | count distinct, strings | 36.000ms | 60.480ms | 60.508ms | 0.1% | 60.502ms | 60.554ms | 60.440ms | 61.421ms | 620.000ms | 290.48 MiB | 8.87 MiB | 165.26M/s |
| q7 | min and max of a date | 2.000ms | 20.271ms | 20.247ms | 0.0% | 20.247ms | 20.251ms | 20.237ms | 20.267ms | 0.000us | 29.49 MiB | none | 493.88M/s |
| q8 | group by, low card | 7.000ms | 20.237ms | 20.260ms | 0.0% | 20.260ms | 20.262ms | 20.247ms | 20.290ms | 30.000ms | 53.24 MiB | none | 493.56M/s |
| q9 | group by and count distinct | 69.000ms | 103.308ms | 100.817ms | 2.3% | 100.711ms | 103.051ms | 83.883ms | 104.387ms | 1.220s | 428.79 MiB | 5.05 MiB | 99.19M/s |
| q10 | group by, several aggregates | 102.000ms | 123.140ms | 122.502ms | 1.7% | 120.767ms | 122.903ms | 120.722ms | 140.863ms | 1.570s | 481.06 MiB | none | 81.63M/s |
| q11 | group by a string and count distinct | 26.000ms | 60.760ms | 40.367ms | 0.1% | 40.346ms | 40.368ms | 40.337ms | 60.430ms | 270.000ms | 196.56 MiB | 3.61 MiB | 247.72M/s |
| q12 | group by two strings and count distinct | 30.000ms | 40.383ms | 60.465ms | 0.1% | 60.451ms | 60.486ms | 60.430ms | 60.613ms | 280.000ms | 203.30 MiB | 468.00 KiB | 165.38M/s |
| q13 | group by a string and top k | 35.000ms | 61.252ms | 60.479ms | 0.0% | 60.473ms | 60.483ms | 60.468ms | 60.495ms | 450.000ms | 306.31 MiB | none | 165.34M/s |
| q14 | group by a string and count distinct | 60.000ms | 100.753ms | 83.046ms | 24.1% | 80.578ms | 100.630ms | 80.555ms | 121.507ms | 1.000s | 590.56 MiB | none | 120.41M/s |
| q15 | group by two columns and top k | 40.000ms | 60.511ms | 60.498ms | 0.1% | 60.474ms | 60.527ms | 60.451ms | 60.566ms | 550.000ms | 338.93 MiB | 1.81 MiB | 165.29M/s |
| q16 | group by, very high card | 65.000ms | 80.619ms | 100.657ms | 18.0% | 82.720ms | 100.842ms | 81.827ms | 101.714ms | 1.150s | 431.42 MiB | none | 99.34M/s |
| q17 | group by two, very high card | 164.000ms | 222.912ms | 203.354ms | 9.5% | 201.904ms | 221.234ms | 185.487ms | 225.894ms | 2.050s | 825.39 MiB | none | 49.17M/s |
| q18 | group by two, no ordering | 139.000ms | 180.993ms | 181.036ms | 11.3% | 160.985ms | 181.437ms | 160.896ms | 182.078ms | 1.510s | 766.17 MiB | none | 55.24M/s |
| q19 | group by with an extract | 229.000ms | 244.782ms | 264.429ms | 7.6% | 263.939ms | 284.015ms | 243.900ms | 284.215ms | 3.260s | 1.13 GiB | 15.55 MiB | 37.82M/s |
| q20 | point lookup | 13.000ms | 40.441ms | 40.497ms | 0.4% | 40.410ms | 40.578ms | 40.353ms | 40.700ms | 60.000ms | 98.55 MiB | none | 246.93M/s |
| q21 | substring scan | 99.000ms | 181.173ms | 120.866ms | 0.1% | 120.844ms | 120.909ms | 120.696ms | 141.262ms | 1.110s | 643.74 MiB | 103.16 MiB | 82.73M/s |
| q22 | substring scan and group by | 131.000ms | 160.971ms | 160.875ms | 0.0% | 160.866ms | 160.908ms | 160.857ms | 160.972ms | 1.130s | 721.30 MiB | none | 62.16M/s |
| q23 | two substring scans and group by | 201.000ms | 265.721ms | 241.428ms | 0.1% | 241.318ms | 241.548ms | 221.238ms | 241.789ms | 1.250s | 785.75 MiB | 120.57 MiB | 41.42M/s |
| q24 | select star and top k | 167.000ms | 221.106ms | 201.235ms | 0.1% | 201.124ms | 201.348ms | 180.929ms | 203.194ms | 1.120s | 629.53 MiB | 17.43 MiB | 49.69M/s |
| q25 | top k by a date | 15.000ms | 40.373ms | 40.545ms | 0.3% | 40.439ms | 40.569ms | 40.352ms | 40.638ms | 70.000ms | 74.70 MiB | none | 246.63M/s |
| q26 | top k by a string | 18.000ms | 61.681ms | 40.693ms | 1.0% | 40.480ms | 40.874ms | 40.363ms | 41.983ms | 200.000ms | 104.28 MiB | none | 245.74M/s |
| q27 | top k by two columns | 13.000ms | 40.376ms | 40.398ms | 0.2% | 40.352ms | 40.423ms | 40.352ms | 40.539ms | 80.000ms | 75.05 MiB | none | 247.53M/s |
| q28 | group by with a string length | 111.000ms | 140.920ms | 141.086ms | 0.5% | 140.985ms | 141.643ms | 140.795ms | 161.246ms | 970.000ms | 678.30 MiB | 508.00 KiB | 70.88M/s |
| q29 | group by a regular expression | 1.023s | 1.109s | 1.065s | 0.1% | 1.065s | 1.066s | 1.065s | 1.086s | 7.790s | 949.50 MiB | 101.95 MiB | 9.39M/s |
| q30 | ninety sums over one column | 11.000ms | 20.277ms | 40.356ms | 0.0% | 40.353ms | 40.357ms | 40.348ms | 40.520ms | 40.000ms | 56.80 MiB | none | 247.79M/s |
| q31 | group by two and several aggregates | 71.000ms | 100.988ms | 100.620ms | 0.2% | 100.619ms | 100.840ms | 100.593ms | 100.859ms | 620.000ms | 361.35 MiB | 5.50 MiB | 99.38M/s |
| q32 | group by a high card pair | 61.000ms | 80.532ms | 80.555ms | 0.0% | 80.545ms | 80.570ms | 80.542ms | 80.861ms | 640.000ms | 428.35 MiB | 15.04 MiB | 124.13M/s |
| q33 | group by a high card pair, unfiltered | 154.000ms | 202.474ms | 186.881ms | 8.7% | 185.361ms | 201.543ms | 183.046ms | 208.022ms | 3.290s | 1.21 GiB | none | 53.51M/s |
| q34 | group by a long string | 291.000ms | 327.920ms | 344.559ms | 5.4% | 326.191ms | 344.822ms | 301.658ms | 362.760ms | 3.510s | 1.69 GiB | none | 29.02M/s |
| q35 | group by a constant and a long string | 302.000ms | 323.975ms | 348.139ms | 5.2% | 344.294ms | 362.258ms | 323.708ms | 363.703ms | 3.790s | 1.72 GiB | none | 28.72M/s |
| q36 | group by four expressions | 66.000ms | 106.112ms | 86.067ms | 24.0% | 81.212ms | 101.898ms | 80.662ms | 103.705ms | 1.300s | 482.54 MiB | none | 116.19M/s |
| q37 | date range and group by a URL | 13.000ms | 40.374ms | 40.402ms | 0.1% | 40.401ms | 40.456ms | 40.392ms | 40.712ms | 30.000ms | 61.59 MiB | 216.00 KiB | 247.51M/s |
| q38 | date range and group by a title | 8.000ms | 20.291ms | 20.257ms | 0.0% | 20.251ms | 20.258ms | 20.241ms | 20.261ms | 30.000ms | 47.81 MiB | none | 493.64M/s |
| q39 | date range, group by and offset | 7.000ms | 20.398ms | 20.264ms | 0.2% | 20.254ms | 20.288ms | 20.254ms | 40.353ms | 20.000ms | 46.09 MiB | none | 493.48M/s |
| q40 | date range, a case and a wide group by | 20.000ms | 40.915ms | 40.357ms | 0.0% | 40.354ms | 40.358ms | 40.339ms | 40.393ms | 60.000ms | 89.30 MiB | none | 247.78M/s |
| q41 | date range with an IN and a hash | 6.000ms | 40.382ms | 20.291ms | 0.6% | 20.279ms | 20.398ms | 20.277ms | 40.567ms | 20.000ms | 44.64 MiB | 1.98 MiB | 492.82M/s |
| q42 | date range and a deep offset | 6.000ms | 20.283ms | 20.279ms | 0.2% | 20.272ms | 20.311ms | 20.262ms | 20.318ms | 20.000ms | 44.09 MiB | 1004.00 KiB | 493.10M/s |
| q43 | minute buckets over a date range | 5.000ms | 20.267ms | 20.264ms | 0.0% | 20.264ms | 20.264ms | 20.258ms | 20.302ms | 20.000ms | 39.81 MiB | none | 493.47M/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 3.902s by its own clock and 5.023s by ours, 5.251s cold, 42.290s of CPU, peak 1.72 GiB, 110.20M/s and 20.61 GiB/s.

Running it cost 29% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 52.61x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 2.000ms | 40.316ms | 40.357ms | 0.0% | 40.353ms | 40.372ms | 40.345ms | 40.451ms | 20.000ms | 40.67 MiB | none | 247.78M/s |
| q2 | filtered count | 7.000ms | 40.364ms | 40.333ms | 0.0% | 40.329ms | 40.337ms | 40.327ms | 40.373ms | 30.000ms | 57.57 MiB | 4.00 KiB | 247.93M/s |
| q3 | three aggregates | 10.000ms | 41.367ms | 40.355ms | 0.0% | 40.344ms | 40.358ms | 40.332ms | 40.381ms | 60.000ms | 79.68 MiB | none | 247.79M/s |
| q4 | average | 15.000ms | 40.343ms | 40.357ms | 0.1% | 40.344ms | 40.371ms | 40.337ms | 40.375ms | 100.000ms | 110.43 MiB | none | 247.78M/s |
| q5 | count distinct, high card | 51.000ms | 82.178ms | 81.173ms | 1.8% | 80.585ms | 82.085ms | 80.557ms | 100.689ms | 880.000ms | 364.94 MiB | none | 123.19M/s |
| q6 | count distinct, strings | 35.000ms | 80.512ms | 80.549ms | 24.2% | 61.030ms | 80.562ms | 60.435ms | 80.631ms | 400.000ms | 328.96 MiB | none | 124.15M/s |
| q7 | min and max of a date | 2.000ms | 40.347ms | 40.347ms | 0.1% | 40.343ms | 40.374ms | 40.332ms | 40.397ms | 20.000ms | 41.68 MiB | 428.00 KiB | 247.84M/s |
| q8 | group by, low card | 14.000ms | 40.366ms | 40.340ms | 0.0% | 40.337ms | 40.341ms | 40.328ms | 40.356ms | 50.000ms | 61.20 MiB | none | 247.88M/s |
| q9 | group by and count distinct | 69.000ms | 105.139ms | 102.058ms | 4.5% | 101.737ms | 106.372ms | 100.857ms | 121.099ms | 1.030s | 429.99 MiB | none | 97.98M/s |
| q10 | group by, several aggregates | 122.000ms | 161.091ms | 162.308ms | 0.7% | 161.248ms | 162.457ms | 160.866ms | 163.901ms | 1.170s | 480.72 MiB | none | 61.61M/s |
| q11 | group by a string and count distinct | 26.000ms | 60.447ms | 60.453ms | 0.0% | 60.448ms | 60.453ms | 60.444ms | 60.469ms | 240.000ms | 203.97 MiB | none | 165.41M/s |
| q12 | group by two strings and count distinct | 28.000ms | 80.859ms | 60.461ms | 0.0% | 60.442ms | 60.471ms | 60.430ms | 60.493ms | 260.000ms | 212.98 MiB | none | 165.39M/s |
| q13 | group by a string and top k | 50.000ms | 101.621ms | 81.402ms | 24.7% | 80.681ms | 100.796ms | 80.558ms | 120.823ms | 450.000ms | 404.30 MiB | none | 122.84M/s |
| q14 | group by a string and count distinct | 104.000ms | 144.824ms | 144.592ms | 12.2% | 143.487ms | 161.110ms | 141.060ms | 169.193ms | 1.010s | 624.18 MiB | none | 69.16M/s |
| q15 | group by two columns and top k | 48.000ms | 80.507ms | 80.609ms | 24.9% | 80.559ms | 100.635ms | 80.548ms | 105.265ms | 530.000ms | 430.75 MiB | none | 124.05M/s |
| q16 | group by, very high card | 61.000ms | 103.933ms | 102.723ms | 4.2% | 102.526ms | 106.873ms | 101.464ms | 120.835ms | 1.030s | 434.54 MiB | none | 97.35M/s |
| q17 | group by two, very high card | 153.000ms | 201.131ms | 204.122ms | 0.5% | 203.078ms | 204.140ms | 184.455ms | 204.713ms | 1.870s | 842.33 MiB | none | 48.99M/s |
| q18 | group by two, no ordering | 128.000ms | 160.943ms | 180.945ms | 0.0% | 180.912ms | 180.983ms | 160.834ms | 184.309ms | 1.270s | 801.96 MiB | none | 55.26M/s |
| q19 | group by with an extract | 210.000ms | 243.259ms | 263.918ms | 0.9% | 261.695ms | 264.118ms | 245.075ms | 265.336ms | 2.980s | 1.16 GiB | none | 37.89M/s |
| q20 | point lookup | 14.000ms | 40.345ms | 40.343ms | 0.0% | 40.342ms | 40.345ms | 40.329ms | 40.804ms | 80.000ms | 110.41 MiB | none | 247.87M/s |
| q21 | substring scan | 85.000ms | 140.795ms | 120.732ms | 0.1% | 120.720ms | 120.829ms | 120.705ms | 140.968ms | 990.000ms | 512.73 MiB | none | 82.83M/s |
| q22 | substring scan and group by | 120.000ms | 162.512ms | 160.944ms | 0.0% | 160.911ms | 160.957ms | 160.877ms | 161.051ms | 1.090s | 620.00 MiB | none | 62.13M/s |
| q23 | two substring scans and group by | 184.000ms | 221.489ms | 221.162ms | 8.8% | 201.720ms | 221.214ms | 201.210ms | 241.546ms | 1.410s | 736.11 MiB | none | 45.21M/s |
| q24 | select star and top k | 135.000ms | 162.213ms | 181.218ms | 0.3% | 181.053ms | 181.530ms | 180.949ms | 182.341ms | 960.000ms | 534.25 MiB | none | 55.18M/s |
| q25 | top k by a date | 16.000ms | 40.334ms | 60.441ms | 33.4% | 40.330ms | 60.503ms | 40.327ms | 60.593ms | 90.000ms | 98.23 MiB | none | 165.45M/s |
| q26 | top k by a string | 24.000ms | 60.544ms | 60.455ms | 0.0% | 60.435ms | 60.459ms | 60.398ms | 60.474ms | 180.000ms | 146.18 MiB | none | 165.41M/s |
| q27 | top k by two columns | 14.000ms | 40.365ms | 40.893ms | 10.5% | 40.385ms | 44.660ms | 40.351ms | 60.512ms | 90.000ms | 99.70 MiB | none | 244.53M/s |
| q28 | group by with a string length | 114.000ms | 181.256ms | 160.953ms | 12.5% | 160.947ms | 181.057ms | 140.926ms | 181.920ms | 1.080s | 555.68 MiB | none | 62.13M/s |
| q29 | group by a regular expression | 500.000ms | 591.960ms | 583.118ms | 9.8% | 528.373ms | 585.625ms | 449.047ms | 585.864ms | 7.700s | 1.12 GiB | none | 17.15M/s |
| q30 | ninety sums over one column | 22.000ms | 60.613ms | 60.618ms | 0.1% | 60.590ms | 60.652ms | 60.550ms | 60.754ms | 80.000ms | 76.48 MiB | none | 164.96M/s |
| q31 | group by two and several aggregates | 88.000ms | 120.681ms | 121.061ms | 16.4% | 121.045ms | 140.868ms | 120.868ms | 140.924ms | 610.000ms | 397.82 MiB | none | 82.60M/s |
| q32 | group by a high card pair | 96.000ms | 161.440ms | 144.819ms | 14.1% | 140.988ms | 161.476ms | 140.882ms | 221.467ms | 730.000ms | 477.00 MiB | none | 69.05M/s |
| q33 | group by a high card pair, unfiltered | 184.000ms | 363.901ms | 241.909ms | 6.5% | 228.483ms | 244.147ms | 221.505ms | 283.927ms | 3.060s | 1.18 GiB | none | 41.34M/s |
| q34 | group by a long string | 242.000ms | 300.201ms | 304.766ms | 18.7% | 302.946ms | 359.862ms | 292.234ms | 402.263ms | 3.400s | 1.58 GiB | none | 32.81M/s |
| q35 | group by a constant and a long string | 254.000ms | 308.986ms | 304.000ms | 2.7% | 303.816ms | 311.935ms | 282.293ms | 374.045ms | 3.640s | 1.58 GiB | none | 32.89M/s |
| q36 | group by four expressions | 58.000ms | 101.591ms | 101.258ms | 1.9% | 100.892ms | 102.850ms | 81.842ms | 140.931ms | 890.000ms | 379.11 MiB | none | 98.75M/s |
| q37 | date range and group by a URL | 15.000ms | 40.350ms | 60.498ms | 33.0% | 40.619ms | 60.602ms | 40.372ms | 60.669ms | 60.000ms | 75.65 MiB | none | 165.29M/s |
| q38 | date range and group by a title | 7.000ms | 40.347ms | 40.361ms | 0.1% | 40.349ms | 40.373ms | 40.342ms | 40.450ms | 40.000ms | 60.00 MiB | none | 247.76M/s |
| q39 | date range, group by and offset | 11.000ms | 40.502ms | 40.760ms | 48.5% | 40.683ms | 60.462ms | 40.585ms | 60.602ms | 60.000ms | 61.88 MiB | none | 245.33M/s |
| q40 | date range, a case and a wide group by | 20.000ms | 60.479ms | 60.438ms | 0.3% | 60.410ms | 60.596ms | 60.403ms | 60.607ms | 70.000ms | 102.37 MiB | none | 165.45M/s |
| q41 | date range with an IN and a hash | 9.000ms | 40.489ms | 40.466ms | 0.1% | 40.466ms | 40.514ms | 40.418ms | 40.588ms | 50.000ms | 58.61 MiB | none | 247.11M/s |
| q42 | date range and a deep offset | 10.000ms | 40.493ms | 40.355ms | 0.1% | 40.340ms | 40.362ms | 40.331ms | 40.384ms | 50.000ms | 56.30 MiB | none | 247.79M/s |
| q43 | minute buckets over a date range | 7.000ms | 40.457ms | 40.516ms | 0.2% | 40.513ms | 40.594ms | 40.512ms | 40.631ms | 50.000ms | 53.75 MiB | none | 246.81M/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 3.364s by its own clock and 5.079s by ours, 5.212s cold, 39.860s of CPU, peak 1.58 GiB, 127.82M/s and 23.90 GiB/s.

Running it cost 51% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 14.46x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 6.000ms | 281.435ms | 261.219ms | 7.7% | 261.207ms | 281.373ms | 261.175ms | 281.385ms | 440.000ms | 253.56 MiB | none | 38.28M/s |
| q2 | filtered count | 6.000ms | 261.135ms | 261.135ms | 0.0% | 261.132ms | 261.185ms | 261.120ms | 261.188ms | 430.000ms | 255.05 MiB | none | 38.29M/s |
| q3 | three aggregates | 85.000ms | 341.502ms | 321.512ms | 6.2% | 321.340ms | 341.431ms | 321.337ms | 341.581ms | 570.000ms | 280.41 MiB | 1.15 MiB | 31.10M/s |
| q4 | average | 77.000ms | 362.049ms | 321.352ms | 6.3% | 321.349ms | 341.512ms | 301.309ms | 342.230ms | 580.000ms | 303.71 MiB | none | 31.12M/s |
| q5 | count distinct, high card | 151.000ms | 381.772ms | 381.772ms | 0.1% | 381.645ms | 382.084ms | 364.643ms | 421.918ms | 1.460s | 854.85 MiB | none | 26.19M/s |
| q6 | count distinct, strings | 127.000ms | 381.646ms | 381.699ms | 5.4% | 361.841ms | 382.462ms | 341.601ms | 384.691ms | 1.190s | 558.54 MiB | none | 26.20M/s |
| q7 | min and max of a date | 230.000ms | 321.400ms | 342.295ms | 6.0% | 341.467ms | 361.960ms | 303.366ms | 382.608ms | 720.000ms | 346.97 MiB | 7.03 MiB | 29.21M/s |
| q8 | group by, low card | 278.000ms | 421.982ms | 401.830ms | 5.1% | 401.693ms | 422.006ms | 381.581ms | 442.376ms | 780.000ms | 365.23 MiB | 7.48 MiB | 24.89M/s |
| q9 | group by and count distinct | 209.000ms | 383.521ms | 422.160ms | 9.5% | 422.146ms | 462.268ms | 422.092ms | 504.930ms | 1.610s | 664.65 MiB | 1012.00 KiB | 23.69M/s |
| q10 | group by, several aggregates | 189.000ms | 442.668ms | 422.211ms | 9.5% | 421.991ms | 462.154ms | 402.151ms | 464.181ms | 1.840s | 657.08 MiB | none | 23.68M/s |
| q11 | group by a string and count distinct | 138.000ms | 402.466ms | 402.694ms | 10.0% | 401.874ms | 442.037ms | 383.863ms | 443.087ms | 860.000ms | 377.55 MiB | none | 24.83M/s |
| q12 | group by two strings and count distinct | 134.000ms | 442.500ms | 411.894ms | 4.6% | 403.225ms | 422.297ms | 401.926ms | 442.102ms | 940.000ms | 388.05 MiB | 8.00 KiB | 24.28M/s |
| q13 | group by a string and top k | 130.000ms | 442.278ms | 402.197ms | 9.7% | 383.106ms | 422.269ms | 381.799ms | 422.537ms | 1.150s | 564.02 MiB | none | 24.86M/s |
| q14 | group by a string and count distinct | 187.000ms | 467.545ms | 505.074ms | 8.9% | 482.402ms | 527.524ms | 445.270ms | 539.724ms | 1.620s | 628.32 MiB | none | 19.80M/s |
| q15 | group by two columns and top k | 160.000ms | 442.051ms | 422.591ms | 7.1% | 414.936ms | 444.805ms | 405.834ms | 482.319ms | 1.320s | 605.10 MiB | none | 23.66M/s |
| q16 | group by, very high card | 106.000ms | 361.894ms | 403.719ms | 3.2% | 391.143ms | 404.121ms | 373.183ms | 443.125ms | 1.350s | 595.59 MiB | none | 24.77M/s |
| q17 | group by two, very high card | 239.000ms | 543.083ms | 484.060ms | 4.2% | 483.889ms | 504.111ms | 468.109ms | 527.061ms | 3.350s | 1.02 GiB | none | 20.66M/s |
| q18 | group by two, no ordering | 155.000ms | 442.179ms | 442.169ms | 2.0% | 434.845ms | 443.878ms | 422.582ms | 465.310ms | 1.330s | 512.08 MiB | 380.00 KiB | 22.62M/s |
| q19 | group by with an extract | 305.000ms | 608.894ms | 610.071ms | 2.9% | 605.920ms | 623.734ms | 529.160ms | 626.572ms | 5.080s | 1.31 GiB | none | 16.39M/s |
| q20 | point lookup | 292.000ms | 442.523ms | 422.216ms | 15.0% | 401.923ms | 465.422ms | 401.811ms | 529.191ms | 870.000ms | 400.77 MiB | none | 23.68M/s |
| q21 | substring scan | 180.000ms | 421.855ms | 422.145ms | 4.7% | 422.054ms | 441.972ms | 401.790ms | 462.099ms | 1.610s | 485.12 MiB | none | 23.69M/s |
| q22 | substring scan and group by | 211.000ms | 463.766ms | 442.212ms | 0.0% | 442.016ms | 442.223ms | 422.402ms | 462.173ms | 1.940s | 538.74 MiB | 352.00 KiB | 22.61M/s |
| q23 | two substring scans and group by | 246.000ms | 462.143ms | 482.067ms | 4.1% | 462.381ms | 482.122ms | 442.054ms | 482.472ms | 2.270s | 575.32 MiB | none | 20.74M/s |
| q24 | select star and top k | 440.000ms | 522.391ms | 542.213ms | 4.3% | 522.880ms | 545.993ms | 522.372ms | 583.024ms | 1.740s | 596.59 MiB | 6.02 MiB | 18.44M/s |
| q25 | top k by a date | 300.000ms | 401.824ms | 427.531ms | 4.4% | 422.888ms | 441.783ms | 401.702ms | 441.879ms | 990.000ms | 439.48 MiB | none | 23.39M/s |
| q26 | top k by a string | 102.000ms | 382.091ms | 361.657ms | 5.6% | 341.492ms | 361.804ms | 341.431ms | 362.437ms | 740.000ms | 421.15 MiB | 68.00 KiB | 27.65M/s |
| q27 | top k by two columns | 291.000ms | 421.879ms | 421.748ms | 9.0% | 403.767ms | 441.843ms | 401.643ms | 442.077ms | 950.000ms | 431.75 MiB | 140.00 KiB | 23.71M/s |
| q28 | group by with a string length | 109.000ms | 341.445ms | 361.686ms | 11.2% | 341.458ms | 381.817ms | 321.389ms | 383.187ms | 730.000ms | 350.97 MiB | 2.07 MiB | 27.65M/s |
| q29 | group by a regular expression | 264.000ms | 484.545ms | 482.288ms | 8.0% | 464.708ms | 503.317ms | 442.628ms | 504.280ms | 4.040s | 1.00 GiB | 5.52 MiB | 20.73M/s |
| q30 | ninety sums over one column | 78.000ms | 301.541ms | 301.379ms | 6.7% | 301.332ms | 321.392ms | 301.317ms | 341.581ms | 510.000ms | 276.71 MiB | 2.16 MiB | 33.18M/s |
| q31 | group by two and several aggregates | 139.000ms | 401.756ms | 381.802ms | 0.4% | 381.628ms | 383.013ms | 361.561ms | 401.898ms | 1.070s | 460.15 MiB | none | 26.19M/s |
| q32 | group by a high card pair | 149.000ms | 381.816ms | 401.741ms | 5.0% | 382.129ms | 402.018ms | 381.671ms | 404.474ms | 1.280s | 515.78 MiB | none | 24.89M/s |
| q33 | group by a high card pair, unfiltered | 232.000ms | 462.263ms | 462.663ms | 4.1% | 443.800ms | 462.924ms | 442.190ms | 482.391ms | 3.660s | 1.01 GiB | none | 21.61M/s |
| q34 | group by a long string | 249.000ms | 462.366ms | 485.220ms | 3.9% | 483.616ms | 502.588ms | 471.564ms | 504.629ms | 3.810s | 1.40 GiB | none | 20.61M/s |
| q35 | group by a constant and a long string | 258.000ms | 503.579ms | 482.811ms | 5.3% | 462.689ms | 488.257ms | 444.861ms | 503.555ms | 3.870s | 1.40 GiB | none | 20.71M/s |
| q36 | group by four expressions | 97.000ms | 341.646ms | 341.720ms | 0.1% | 341.596ms | 341.787ms | 341.409ms | 342.159ms | 1.110s | 560.95 MiB | none | 29.26M/s |
| q37 | date range and group by a URL | 273.000ms | 401.696ms | 401.661ms | 5.0% | 381.743ms | 401.767ms | 381.646ms | 421.794ms | 770.000ms | 384.73 MiB | none | 24.90M/s |
| q38 | date range and group by a title | 269.000ms | 381.607ms | 401.669ms | 5.0% | 381.592ms | 401.739ms | 381.583ms | 421.726ms | 760.000ms | 375.49 MiB | none | 24.90M/s |
| q39 | date range, group by and offset | 262.000ms | 381.630ms | 381.607ms | 0.4% | 381.567ms | 383.030ms | 381.533ms | 401.966ms | 760.000ms | 373.80 MiB | none | 26.20M/s |
| q40 | date range, a case and a wide group by | 276.000ms | 421.993ms | 401.739ms | 5.1% | 401.680ms | 422.033ms | 381.610ms | 423.259ms | 830.000ms | 431.82 MiB | none | 24.89M/s |
| q41 | date range with an IN and a hash | 257.000ms | 381.640ms | 381.631ms | 5.2% | 361.647ms | 381.651ms | 361.523ms | 381.658ms | 740.000ms | 368.29 MiB | 3.48 MiB | 26.20M/s |
| q42 | date range and a deep offset | 253.000ms | 401.734ms | 381.579ms | 0.0% | 381.502ms | 381.666ms | 361.559ms | 401.681ms | 750.000ms | 365.27 MiB | none | 26.21M/s |
| q43 | minute buckets over a date range | 255.000ms | 381.567ms | 381.608ms | 0.0% | 381.589ms | 381.617ms | 361.600ms | 401.694ms | 740.000ms | 364.18 MiB | none | 26.20M/s |

clickhouse-local 26.9.1.1562 over 43 of 43 queries. Total 8.394s by its own clock and 17.556s by ours, 17.683s cold, 63.160s of CPU, peak 1.40 GiB, 51.23M/s and 9.58 GiB/s.

Running it cost 109% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.34x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 140.790ms | 100.586ms | 19.9% | 80.558ms | 100.619ms | 80.493ms | 100.633ms | 80.000ms | 182.62 MiB | 118.75 MiB | 99.42M/s |
| q2 | filtered count | 253.000ms | 347.116ms | 346.402ms | 0.8% | 345.606ms | 348.511ms | 328.740ms | 348.625ms | 5.050s | 2.38 GiB | 8.40 MiB | 28.87M/s |
| q3 | three aggregates | 268.000ms | 347.731ms | 347.763ms | 1.0% | 347.139ms | 350.479ms | 344.379ms | 365.220ms | 5.230s | 2.78 GiB | 8.00 KiB | 28.75M/s |
| q4 | average | 255.000ms | 348.259ms | 347.909ms | 1.0% | 346.606ms | 349.975ms | 346.045ms | 367.561ms | 4.890s | 2.36 GiB | none | 28.74M/s |
| q5 | count distinct, high card | 294.000ms | 386.859ms | 388.580ms | 4.6% | 370.950ms | 389.014ms | 368.287ms | 390.000ms | 5.500s | 2.60 GiB | none | 25.73M/s |
| q6 | count distinct, strings | 296.000ms | 389.020ms | 386.524ms | 0.5% | 384.808ms | 386.892ms | 373.439ms | 393.288ms | 5.740s | 2.76 GiB | none | 25.87M/s |
| q7 | min and max of a date | 1.000ms | 80.553ms | 80.527ms | 0.0% | 80.519ms | 80.551ms | 80.506ms | 80.598ms | 60.000ms | 161.53 MiB | none | 124.18M/s |
| q8 | group by, low card | 253.000ms | 325.804ms | 347.646ms | 3.3% | 337.637ms | 348.990ms | 331.604ms | 349.222ms | 4.960s | 2.34 GiB | none | 28.76M/s |
| q9 | group by and count distinct | 315.000ms | 391.887ms | 407.146ms | 0.9% | 406.612ms | 410.136ms | 386.726ms | 485.959ms | 6.020s | 2.72 GiB | none | 24.56M/s |
| q10 | group by, several aggregates | 359.000ms | 505.782ms | 450.832ms | 16.5% | 429.081ms | 503.563ms | 426.710ms | 507.828ms | 6.110s | 2.89 GiB | 4.00 KiB | 22.18M/s |
| q11 | group by a string and count distinct | 291.000ms | 368.982ms | 386.599ms | 0.7% | 384.916ms | 387.701ms | 382.685ms | 393.860ms | 5.510s | 2.84 GiB | 4.07 MiB | 25.87M/s |
| q12 | group by two strings and count distinct | 290.000ms | 371.812ms | 386.609ms | 4.4% | 369.481ms | 386.681ms | 365.856ms | 390.669ms | 5.410s | 2.82 GiB | none | 25.87M/s |
| q13 | group by a string and top k | 294.000ms | 387.960ms | 388.267ms | 0.3% | 387.394ms | 388.380ms | 385.321ms | 389.126ms | 5.560s | 2.90 GiB | none | 25.75M/s |
| q14 | group by a string and count distinct | 356.000ms | 429.089ms | 447.869ms | 0.1% | 447.725ms | 448.012ms | 439.986ms | 450.649ms | 7.080s | 3.17 GiB | none | 22.33M/s |
| q15 | group by two columns and top k | 299.000ms | 386.058ms | 386.964ms | 0.9% | 385.845ms | 389.360ms | 385.503ms | 410.219ms | 5.680s | 2.96 GiB | none | 25.84M/s |
| q16 | group by, very high card | 305.000ms | 386.559ms | 390.814ms | 5.1% | 387.012ms | 406.876ms | 386.098ms | 408.159ms | 5.880s | 2.73 GiB | none | 25.59M/s |
| q17 | group by two, very high card | 389.000ms | 493.939ms | 485.694ms | 0.7% | 483.032ms | 486.237ms | 467.943ms | 507.166ms | 7.330s | 3.68 GiB | none | 20.59M/s |
| q18 | group by two, no ordering | 397.000ms | 467.665ms | 487.198ms | 0.7% | 486.100ms | 489.277ms | 485.752ms | 489.284ms | 7.760s | 3.74 GiB | none | 20.53M/s |
| q19 | group by with an extract | 436.000ms | 530.913ms | 528.642ms | 0.3% | 527.783ms | 529.266ms | 525.914ms | 534.805ms | 8.750s | 4.22 GiB | 28.00 KiB | 18.92M/s |
| q20 | point lookup | 256.000ms | 344.614ms | 344.767ms | 0.3% | 344.358ms | 345.559ms | 328.129ms | 346.725ms | 4.970s | 2.34 GiB | none | 29.00M/s |
| q21 | substring scan | 303.000ms | 405.874ms | 387.104ms | 0.1% | 387.018ms | 387.225ms | 367.103ms | 407.521ms | 5.520s | 3.26 GiB | none | 25.83M/s |
| q22 | substring scan and group by | 337.000ms | 408.651ms | 423.629ms | 4.9% | 406.480ms | 427.040ms | 405.223ms | 429.752ms | 6.160s | 2.97 GiB | none | 23.60M/s |
| q23 | two substring scans and group by | 417.000ms | 491.126ms | 507.516ms | 1.2% | 507.468ms | 513.323ms | 491.175ms | 527.610ms | 8.350s | 2.98 GiB | none | 19.70M/s |
| q24 | select star and top k | 634.000ms | 636.366ms | 730.067ms | 2.3% | 715.936ms | 732.535ms | 701.227ms | 757.006ms | 13.820s | 3.11 GiB | none | 13.70M/s |
| q25 | top k by a date | 257.000ms | 345.874ms | 347.592ms | 0.7% | 345.593ms | 347.895ms | 331.674ms | 351.619ms | 4.970s | 2.37 GiB | none | 28.77M/s |
| q26 | top k by a string | 278.000ms | 368.160ms | 368.829ms | 1.3% | 364.954ms | 369.606ms | 364.011ms | 369.646ms | 5.330s | 2.69 GiB | none | 27.11M/s |
| q27 | top k by two columns | 255.000ms | 348.124ms | 346.523ms | 0.8% | 345.001ms | 347.653ms | 343.727ms | 348.634ms | 5.030s | 2.38 GiB | none | 28.86M/s |
| q28 | group by with a string length | 315.000ms | 427.317ms | 414.704ms | 4.2% | 411.214ms | 428.763ms | 409.442ms | 431.644ms | 6.010s | 3.34 GiB | none | 24.11M/s |
| q29 | group by a regular expression | 457.000ms | 532.664ms | 554.069ms | 2.6% | 552.113ms | 566.419ms | 544.348ms | 567.696ms | 9.360s | 4.01 GiB | none | 18.05M/s |
| q30 | ninety sums over one column | 261.000ms | 354.361ms | 348.642ms | 1.2% | 345.137ms | 349.199ms | 343.000ms | 354.913ms | 5.010s | 2.32 GiB | none | 28.68M/s |
| q31 | group by two and several aggregates | 354.000ms | 411.345ms | 452.250ms | 2.9% | 451.985ms | 465.004ms | 448.014ms | 468.026ms | 5.660s | 2.90 GiB | none | 22.11M/s |
| q32 | group by a high card pair | 331.000ms | 465.538ms | 426.432ms | 21.2% | 417.001ms | 507.591ms | 411.058ms | 515.235ms | 6.130s | 2.87 GiB | none | 23.45M/s |
| q33 | group by a high card pair, unfiltered | 425.000ms | 531.679ms | 515.899ms | 3.1% | 510.796ms | 527.016ms | 509.484ms | 532.305ms | 8.310s | 3.80 GiB | 36.00 KiB | 19.38M/s |
| q34 | group by a long string | 454.000ms | 546.323ms | 551.226ms | 0.4% | 549.396ms | 551.466ms | 527.804ms | 568.499ms | 9.210s | 4.49 GiB | none | 18.14M/s |
| q35 | group by a constant and a long string | 465.000ms | 546.910ms | 556.921ms | 3.2% | 547.704ms | 565.687ms | 547.507ms | 583.081ms | 9.300s | 4.55 GiB | none | 17.96M/s |
| q36 | group by four expressions | 286.000ms | 372.485ms | 367.903ms | 1.3% | 365.845ms | 370.536ms | 365.569ms | 370.908ms | 5.320s | 2.61 GiB | none | 27.18M/s |
| q37 | date range and group by a URL | 239.000ms | 333.598ms | 327.913ms | 0.5% | 327.823ms | 329.302ms | 325.940ms | 331.475ms | 4.590s | 2.32 GiB | none | 30.50M/s |
| q38 | date range and group by a title | 245.000ms | 328.916ms | 326.912ms | 0.4% | 325.805ms | 327.211ms | 325.435ms | 330.519ms | 4.500s | 2.35 GiB | none | 30.59M/s |
| q39 | date range, group by and offset | 239.000ms | 327.675ms | 327.459ms | 1.3% | 325.098ms | 329.389ms | 312.816ms | 334.890ms | 4.480s | 2.26 GiB | none | 30.54M/s |
| q40 | date range, a case and a wide group by | 237.000ms | 327.369ms | 329.586ms | 0.8% | 327.895ms | 330.550ms | 327.024ms | 386.673ms | 4.250s | 2.33 GiB | none | 30.34M/s |
| q41 | date range with an IN and a hash | 237.000ms | 326.144ms | 325.890ms | 0.4% | 325.127ms | 326.457ms | 323.435ms | 328.116ms | 4.580s | 2.36 GiB | none | 30.68M/s |
| q42 | date range and a deep offset | 239.000ms | 327.158ms | 326.037ms | 0.5% | 325.409ms | 327.155ms | 322.105ms | 328.975ms | 4.580s | 2.34 GiB | none | 30.67M/s |
| q43 | minute buckets over a date range | 237.000ms | 330.622ms | 325.933ms | 1.6% | 322.712ms | 327.873ms | 309.496ms | 327.990ms | 4.570s | 2.30 GiB | none | 30.68M/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 13.110s by its own clock and 17.026s by ours, 16.926s cold, 252.610s of CPU, peak 4.55 GiB, 32.80M/s and 6.13 GiB/s.

Running it cost 30% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 9.07x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 39.278ms | 241.041ms | 141.615ms | 1.0% | 141.291ms | 142.636ms | 141.113ms | 145.569ms | 150.000ms | 122.25 MiB | 115.44 MiB | 70.61M/s |
| q2 | filtered count | 45.085ms | 141.133ms | 140.850ms | 0.0% | 140.825ms | 140.879ms | 140.743ms | 141.692ms | 150.000ms | 175.12 MiB | 272.00 KiB | 71.00M/s |
| q3 | three aggregates | 56.658ms | 161.034ms | 161.504ms | 0.6% | 160.838ms | 161.866ms | 160.791ms | 162.211ms | 190.000ms | 302.65 MiB | 704.00 KiB | 61.92M/s |
| q4 | average | 54.426ms | 160.943ms | 161.035ms | 0.1% | 160.927ms | 161.060ms | 160.890ms | 161.259ms | 190.000ms | 277.52 MiB | none | 62.10M/s |
| q5 | count distinct, high card | 103.978ms | 202.693ms | 208.898ms | 6.5% | 208.029ms | 221.592ms | 203.510ms | 222.399ms | 870.000ms | 428.22 MiB | none | 47.87M/s |
| q6 | count distinct, strings | 100.856ms | 201.707ms | 202.126ms | 0.0% | 202.108ms | 202.147ms | 202.108ms | 202.186ms | 790.000ms | 459.15 MiB | none | 49.47M/s |
| q7 | min and max of a date | 50.918ms | 160.891ms | 141.535ms | 14.3% | 140.871ms | 161.121ms | 140.742ms | 167.701ms | 180.000ms | 217.37 MiB | none | 70.65M/s |
| q8 | group by, low card | 52.453ms | 140.904ms | 160.775ms | 12.3% | 141.068ms | 160.868ms | 140.937ms | 161.060ms | 190.000ms | 175.59 MiB | none | 62.20M/s |
| q9 | group by and count distinct | 163.237ms | 282.936ms | 284.976ms | 0.7% | 283.944ms | 285.833ms | 282.396ms | 305.665ms | 2.430s | 921.62 MiB | 52.00 KiB | 35.09M/s |
| q10 | group by, several aggregates | 178.072ms | 302.881ms | 304.437ms | 0.6% | 304.150ms | 306.002ms | 302.961ms | 306.869ms | 2.550s | 949.87 MiB | none | 32.85M/s |
| q11 | group by a string and count distinct | 86.799ms | 181.363ms | 181.211ms | 0.5% | 181.113ms | 181.932ms | 180.911ms | 202.195ms | 380.000ms | 391.76 MiB | none | 55.18M/s |
| q12 | group by two strings and count distinct | 90.017ms | 201.414ms | 201.482ms | 0.1% | 201.328ms | 201.533ms | 182.962ms | 201.612ms | 420.000ms | 398.20 MiB | none | 49.63M/s |
| q13 | group by a string and top k | 105.529ms | 221.162ms | 221.635ms | 0.0% | 221.593ms | 221.700ms | 221.390ms | 221.894ms | 740.000ms | 473.91 MiB | none | 45.12M/s |
| q14 | group by a string and count distinct | 149.414ms | 262.230ms | 262.836ms | 0.7% | 262.034ms | 263.878ms | 261.972ms | 263.931ms | 1.690s | 729.78 MiB | none | 38.05M/s |
| q15 | group by two columns and top k | 104.318ms | 201.410ms | 221.562ms | 0.1% | 221.464ms | 221.598ms | 221.391ms | 221.963ms | 840.000ms | 488.99 MiB | none | 45.13M/s |
| q16 | group by, very high card | 122.739ms | 244.587ms | 241.705ms | 0.0% | 241.647ms | 241.738ms | 241.271ms | 242.547ms | 1.130s | 590.08 MiB | none | 41.37M/s |
| q17 | group by two, very high card | 201.027ms | 323.512ms | 342.477ms | 5.8% | 325.013ms | 344.722ms | 323.514ms | 344.804ms | 3.070s | 1.25 GiB | none | 29.20M/s |
| q18 | group by two, no ordering | 186.965ms | 322.040ms | 322.897ms | 0.9% | 322.161ms | 325.082ms | 304.288ms | 327.187ms | 3.040s | 1.25 GiB | none | 30.97M/s |
| q19 | group by with an extract | 235.836ms | 402.361ms | 387.215ms | 0.6% | 385.703ms | 388.066ms | 382.805ms | 405.632ms | 4.270s | 1.73 GiB | none | 25.82M/s |
| q20 | point lookup | 46.544ms | 140.912ms | 141.174ms | 14.3% | 140.869ms | 161.034ms | 140.797ms | 163.367ms | 160.000ms | 209.09 MiB | 12.00 KiB | 70.83M/s |
| q21 | substring scan | 173.493ms | 283.975ms | 282.125ms | 0.1% | 281.948ms | 282.143ms | 281.611ms | 282.144ms | 1.910s | 788.96 MiB | 1.13 MiB | 35.44M/s |
| q22 | substring scan and group by | 189.969ms | 301.570ms | 305.733ms | 1.7% | 303.243ms | 308.311ms | 301.941ms | 310.176ms | 2.170s | 959.40 MiB | none | 32.71M/s |
| q23 | two substring scans and group by | 270.544ms | 385.894ms | 386.845ms | 1.0% | 384.802ms | 388.846ms | 383.574ms | 389.441ms | 4.420s | 1.70 GiB | none | 25.85M/s |
| q24 | select star and top k | 513.086ms | 644.022ms | 626.302ms | 2.7% | 625.994ms | 642.987ms | 624.486ms | 645.851ms | 6.610s | 2.40 GiB | none | 15.97M/s |
| q25 | top k by a date | 84.938ms | 181.281ms | 181.511ms | 0.3% | 181.135ms | 181.618ms | 181.076ms | 181.710ms | 430.000ms | 448.83 MiB | none | 55.09M/s |
| q26 | top k by a string | 77.772ms | 181.230ms | 181.535ms | 0.8% | 181.059ms | 182.487ms | 180.971ms | 182.626ms | 340.000ms | 345.10 MiB | none | 55.08M/s |
| q27 | top k by two columns | 90.520ms | 181.713ms | 201.756ms | 5.9% | 189.932ms | 201.913ms | 181.476ms | 201.947ms | 510.000ms | 501.10 MiB | none | 49.56M/s |
| q30 | ninety sums over one column | 77.626ms | 181.917ms | 181.458ms | 0.2% | 181.284ms | 181.727ms | 181.162ms | 181.822ms | 510.000ms | 220.05 MiB | none | 55.11M/s |
| q31 | group by two and several aggregates | 109.781ms | 221.884ms | 221.573ms | 0.4% | 221.455ms | 222.249ms | 221.423ms | 222.776ms | 750.000ms | 565.29 MiB | none | 45.13M/s |
| q32 | group by a high card pair | 111.441ms | 222.044ms | 223.082ms | 0.5% | 222.281ms | 223.503ms | 221.543ms | 223.989ms | 830.000ms | 726.13 MiB | none | 44.83M/s |
| q33 | group by a high card pair, unfiltered | 235.049ms | 403.485ms | 384.203ms | 0.1% | 384.017ms | 384.302ms | 382.831ms | 384.786ms | 4.080s | 1.83 GiB | none | 26.03M/s |
| q34 | group by a long string | 282.954ms | 467.632ms | 445.059ms | 1.5% | 443.782ms | 450.631ms | 442.894ms | 464.702ms | 4.900s | 2.59 GiB | none | 22.47M/s |
| q35 | group by a constant and a long string | 321.306ms | 490.173ms | 487.225ms | 0.2% | 487.038ms | 487.789ms | 486.869ms | 491.270ms | 6.080s | 2.55 GiB | none | 20.52M/s |
| q37 | date range and group by a URL | 56.302ms | 161.086ms | 161.051ms | 0.3% | 160.858ms | 161.366ms | 160.843ms | 161.457ms | 200.000ms | 170.98 MiB | 12.00 KiB | 62.09M/s |
| q38 | date range and group by a title | 52.502ms | 160.824ms | 160.802ms | 0.3% | 160.801ms | 161.230ms | 140.906ms | 162.121ms | 180.000ms | 146.77 MiB | none | 62.19M/s |
| q39 | date range, group by and offset | 49.602ms | 141.411ms | 141.094ms | 0.1% | 140.896ms | 141.095ms | 140.811ms | 160.931ms | 160.000ms | 139.48 MiB | none | 70.87M/s |
| q40 | date range, a case and a wide group by | 63.073ms | 160.737ms | 160.997ms | 0.3% | 160.862ms | 161.285ms | 160.757ms | 161.418ms | 230.000ms | 226.00 MiB | none | 62.11M/s |
| q41 | date range with an IN and a hash | 48.407ms | 141.142ms | 140.752ms | 0.1% | 140.736ms | 140.813ms | 140.668ms | 140.839ms | 160.000ms | 136.27 MiB | 6.77 MiB | 71.05M/s |
| q42 | date range and a deep offset | 47.583ms | 141.175ms | 140.729ms | 0.1% | 140.680ms | 140.800ms | 140.672ms | 140.815ms | 160.000ms | 133.50 MiB | none | 71.06M/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 5.030s by its own clock and 9.446s by ours, 9.550s cold, 58.060s of CPU, peak 2.59 GiB, 77.53M/s and 14.50 GiB/s.

Running it cost 88% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 4.45x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 147.818ms | 221.259ms | 221.276ms | 6.8% | 206.427ms | 221.456ms | 201.335ms | 221.513ms | 260.000ms | 76.59 MiB | 4.64 MiB | 45.19M/s |
| q2 | filtered count | 156.011ms | 226.726ms | 222.219ms | 1.0% | 221.053ms | 223.300ms | 209.479ms | 229.211ms | 290.000ms | 77.13 MiB | none | 45.00M/s |
| q3 | three aggregates | 164.342ms | 224.816ms | 226.053ms | 1.5% | 223.219ms | 226.514ms | 204.940ms | 232.178ms | 410.000ms | 77.37 MiB | none | 44.24M/s |
| q4 | average | 153.940ms | 222.358ms | 221.285ms | 3.0% | 214.683ms | 221.325ms | 206.023ms | 231.349ms | 380.000ms | 78.61 MiB | none | 45.19M/s |
| q5 | count distinct, high card | 182.859ms | 262.531ms | 243.252ms | 5.1% | 241.877ms | 254.374ms | 241.569ms | 259.626ms | 610.000ms | 442.48 MiB | none | 41.11M/s |
| q6 | count distinct, strings | 268.683ms | 342.229ms | 341.691ms | 5.1% | 324.351ms | 341.695ms | 323.041ms | 345.285ms | 1.000s | 277.29 MiB | none | 29.27M/s |
| q7 | min and max of a date | 169.946ms | 227.415ms | 230.841ms | 8.0% | 223.156ms | 241.578ms | 223.118ms | 242.870ms | 340.000ms | 76.66 MiB | none | 43.32M/s |
| q8 | group by, low card | 162.667ms | 221.182ms | 231.010ms | 4.3% | 223.230ms | 233.060ms | 221.784ms | 233.850ms | 350.000ms | 77.19 MiB | none | 43.29M/s |
| q9 | group by and count distinct | 215.101ms | 297.286ms | 303.358ms | 5.2% | 289.334ms | 305.202ms | 281.403ms | 311.881ms | 950.000ms | 584.43 MiB | none | 32.96M/s |
| q10 | group by, several aggregates | 224.549ms | 328.803ms | 306.177ms | 1.9% | 301.542ms | 307.241ms | 290.569ms | 307.656ms | 1.280s | 594.72 MiB | none | 32.66M/s |
| q11 | group by a string and count distinct | 211.495ms | 249.986ms | 281.468ms | 5.2% | 267.223ms | 281.952ms | 241.707ms | 289.813ms | 910.000ms | 97.05 MiB | none | 35.53M/s |
| q12 | group by two strings and count distinct | 205.859ms | 273.449ms | 264.205ms | 9.0% | 246.303ms | 270.155ms | 245.914ms | 276.484ms | 970.000ms | 99.16 MiB | none | 37.85M/s |
| q13 | group by a string and top k | 221.413ms | 301.403ms | 284.640ms | 0.8% | 283.867ms | 286.098ms | 282.250ms | 301.568ms | 830.000ms | 216.61 MiB | none | 35.13M/s |
| q14 | group by a string and count distinct | 256.614ms | 328.572ms | 322.498ms | 1.5% | 322.369ms | 327.267ms | 321.745ms | 327.866ms | 1.180s | 296.10 MiB | none | 31.01M/s |
| q15 | group by two columns and top k | 236.472ms | 309.418ms | 305.991ms | 0.7% | 305.604ms | 307.711ms | 301.907ms | 309.016ms | 1.000s | 259.35 MiB | none | 32.68M/s |
| q16 | group by, very high card | 300.164ms | 383.376ms | 371.040ms | 2.0% | 365.869ms | 373.247ms | 347.750ms | 409.234ms | 2.190s | 827.52 MiB | none | 26.95M/s |
| q17 | group by two, very high card | 377.346ms | 463.735ms | 443.311ms | 0.7% | 443.021ms | 445.935ms | 442.783ms | 462.691ms | 2.060s | 633.66 MiB | none | 22.56M/s |
| q18 | group by two, no ordering | 211.995ms | 291.499ms | 277.035ms | 3.1% | 271.545ms | 280.018ms | 269.246ms | 286.600ms | 1.360s | 94.70 MiB | none | 36.10M/s |
| q19 | group by with an extract | 524.595ms | 603.349ms | 605.354ms | 0.1% | 604.904ms | 605.566ms | 603.824ms | 624.408ms | 2.840s | 1.16 GiB | none | 16.52M/s |
| q20 | point lookup | 144.012ms | 206.383ms | 209.779ms | 2.6% | 205.013ms | 210.547ms | 203.456ms | 221.858ms | 370.000ms | 78.22 MiB | none | 47.67M/s |
| q21 | substring scan | 263.985ms | 341.453ms | 325.799ms | 10.9% | 311.010ms | 346.600ms | 301.671ms | 352.129ms | 2.020s | 199.14 MiB | none | 30.69M/s |
| q22 | substring scan and group by | 301.148ms | 364.491ms | 356.436ms | 11.3% | 318.790ms | 359.042ms | 315.162ms | 362.860ms | 2.820s | 195.10 MiB | none | 28.05M/s |
| q23 | two substring scans and group by | 384.834ms | 464.621ms | 453.829ms | 3.6% | 437.749ms | 454.048ms | 435.065ms | 463.790ms | 4.100s | 200.33 MiB | none | 22.03M/s |
| q24 | select star and top k | 299.672ms | 392.450ms | 368.561ms | 5.5% | 357.288ms | 377.525ms | 346.405ms | 384.369ms | 1.770s | 244.66 MiB | none | 27.13M/s |
| q25 | top k by a date | 203.648ms | 249.430ms | 258.931ms | 7.0% | 246.968ms | 265.144ms | 241.269ms | 274.096ms | 1.000s | 96.66 MiB | none | 38.62M/s |
| q26 | top k by a string | 177.729ms | 245.297ms | 234.762ms | 15.1% | 231.134ms | 266.635ms | 225.631ms | 273.451ms | 910.000ms | 93.19 MiB | none | 42.60M/s |
| q27 | top k by two columns | 194.800ms | 268.925ms | 259.926ms | 2.6% | 259.037ms | 265.879ms | 247.383ms | 271.857ms | 1.010s | 96.40 MiB | none | 38.47M/s |
| q28 | group by with a string length | 286.060ms | 350.275ms | 350.678ms | 7.6% | 344.346ms | 370.964ms | 340.462ms | 375.909ms | 2.390s | 197.78 MiB | none | 28.52M/s |
| q29 | group by a regular expression | 421.348ms | 489.362ms | 488.263ms | 1.2% | 486.629ms | 492.360ms | 485.080ms | 502.155ms | 3.400s | 414.55 MiB | none | 20.48M/s |
| q30 | ninety sums over one column | 157.424ms | 201.025ms | 222.720ms | 0.3% | 222.623ms | 223.282ms | 221.353ms | 225.052ms | 400.000ms | 76.86 MiB | none | 44.90M/s |
| q31 | group by two and several aggregates | 217.033ms | 301.161ms | 286.489ms | 12.5% | 272.994ms | 308.745ms | 255.597ms | 339.802ms | 1.090s | 198.28 MiB | none | 34.90M/s |
| q32 | group by a high card pair | 207.585ms | 292.996ms | 285.110ms | 1.1% | 284.616ms | 287.747ms | 273.983ms | 305.695ms | 930.000ms | 206.88 MiB | none | 35.07M/s |
| q33 | group by a high card pair, unfiltered | 365.701ms | 419.135ms | 442.529ms | 2.6% | 438.506ms | 450.088ms | 436.480ms | 455.958ms | 2.510s | 843.14 MiB | none | 22.60M/s |
| q34 | group by a long string | 472.975ms | 563.061ms | 543.183ms | 0.1% | 542.871ms | 543.562ms | 523.722ms | 563.550ms | 3.060s | 1011.40 MiB | none | 18.41M/s |
| q35 | group by a constant and a long string | 472.535ms | 523.857ms | 544.086ms | 0.3% | 542.849ms | 544.410ms | 523.843ms | 549.216ms | 3.070s | 999.79 MiB | none | 18.38M/s |
| q36 | group by four expressions | 255.172ms | 342.552ms | 336.059ms | 7.8% | 326.794ms | 353.038ms | 313.035ms | 365.415ms | 1.720s | 559.04 MiB | none | 29.76M/s |
| q37 | date range and group by a URL | 155.178ms | 221.149ms | 221.767ms | 0.3% | 221.365ms | 222.005ms | 221.257ms | 225.104ms | 270.000ms | 104.27 MiB | none | 45.09M/s |
| q38 | date range and group by a title | 145.846ms | 241.396ms | 206.279ms | 9.7% | 201.429ms | 221.382ms | 201.104ms | 221.636ms | 260.000ms | 82.38 MiB | none | 48.48M/s |
| q39 | date range, group by and offset | 149.809ms | 221.429ms | 221.684ms | 0.1% | 221.545ms | 221.700ms | 221.437ms | 221.953ms | 250.000ms | 85.82 MiB | none | 45.11M/s |
| q40 | date range, a case and a wide group by | 165.891ms | 271.217ms | 231.289ms | 7.9% | 228.991ms | 247.280ms | 226.747ms | 251.714ms | 350.000ms | 150.45 MiB | none | 43.23M/s |
| q41 | date range with an IN and a hash | 147.534ms | 201.782ms | 201.281ms | 9.9% | 201.060ms | 221.064ms | 200.974ms | 221.411ms | 210.000ms | 80.32 MiB | none | 49.68M/s |
| q42 | date range and a deep offset | 135.014ms | 221.337ms | 201.206ms | 0.1% | 201.075ms | 201.234ms | 201.037ms | 202.779ms | 200.000ms | 79.67 MiB | none | 49.70M/s |
| q43 | minute buckets over a date range | 146.486ms | 200.892ms | 221.240ms | 9.1% | 201.202ms | 221.430ms | 200.940ms | 221.517ms | 210.000ms | 78.22 MiB | none | 45.20M/s |

rudb rudb 0.3.45 over 43 of 43 queries. Total 10.263s by its own clock and 13.175s by ours, 13.375s cold, 53.530s of CPU, peak 1.16 GiB, 41.90M/s and 7.83 GiB/s.

Running it cost 28% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.01x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | 89.135ms | 6.251ms | 15.664ms | 15.669ms | 0.0% | 7.557ms | 39.918ms | 184.413ms | 896 B | 4 of 4 |
| q2 | 91.631ms | 15.494ms | 52.272ms | 52.276ms | 0.0% | 21.643ms | 42.556ms | 205.168ms | 896 B | 5 of 5 |
| q3 | 87.177ms | 21.455ms | 94.914ms | 94.920ms | 0.0% | 7.207ms | 42.059ms | 343.021ms | 1.22 KiB | 4 of 4 |
| q4 | 83.725ms | 20.411ms | 75.986ms | 75.991ms | 0.0% | 18.932ms | 38.339ms | 245.670ms | 896 B | 4 of 4 |
| q5 | 92.391ms | 39.969ms | 131.193ms | 131.199ms | 0.0% | 12.895ms | 42.892ms | 405.909ms | 368.19 MiB | 4 of 4 |
| q6 | 85.251ms | 135.276ms | 720.257ms | 720.265ms | 0.0% | 13.714ms | 39.740ms | 249.995ms | 152.91 MiB | 5 of 5 |
| q7 | 88.922ms | 21.617ms | 92.734ms | 92.740ms | 0.0% | 34.555ms | 42.501ms | 384.759ms | 1.00 KiB | 4 of 4 |
| q8 | 91.438ms | 15.237ms | 56.176ms | 56.183ms | 0.0% | 16.126ms | 41.562ms | 272.255ms | 14.27 KiB | 6 of 6 |
| q9 | 89.966ms | 75.622ms | 402.006ms | 402.012ms | 0.0% | 27.295ms | 41.279ms | 746.709ms | 418.98 MiB | 5 of 5 |
| q10 | 92.173ms | 87.331ms | 612.787ms | 612.794ms | 0.0% | 19.379ms | 43.156ms | 534.050ms | 416.64 MiB | 5 of 5 |
| q11 | 92.688ms | 40.669ms | 295.467ms | 295.473ms | 0.0% | 12.293ms | 43.110ms | 311.418ms | 6.57 MiB | 6 of 6 |
| q12 | 86.819ms | 69.055ms | 455.019ms | 455.027ms | 0.0% | 13.823ms | 40.086ms | 624.887ms | 6.75 MiB | 6 of 6 |
| q13 | 92.066ms | 86.618ms | 622.596ms | 622.602ms | 0.0% | 14.511ms | 41.013ms | 196.385ms | 115.60 MiB | 6 of 6 |
| q14 | 84.695ms | 125.857ms | 861.645ms | 861.653ms | 0.0% | 15.292ms | 38.965ms | 259.382ms | 194.60 MiB | 6 of 6 |
| q15 | 84.845ms | 110.269ms | 801.845ms | 801.850ms | 0.0% | 15.242ms | 39.310ms | 188.840ms | 152.09 MiB | 6 of 6 |
| q16 | 84.188ms | 173.428ms | 1.083s | 1.083s | 0.0% | 50.294ms | 38.229ms | 1.009s | 369.90 MiB | 5 of 5 |
| q17 | 83.291ms | 241.554ms | 1.842s | 1.843s | 0.0% | 18.025ms | 38.560ms | 208.937ms | 456.63 MiB | 5 of 5 |
| q18 | 89.177ms | 86.775ms | 700.570ms | 700.576ms | 0.0% | 9.828ms | 43.077ms | 736.347ms | 15.30 KiB | 5 of 5 |
| q19 | 81.881ms | 400.125ms | 2.512s | 2.512s | 0.0% | 16.288ms | 38.092ms | 260.045ms | 945.32 MiB | 5 of 5 |
| q20 | 82.713ms | 15.417ms | 44.043ms | 44.050ms | 0.0% | 11.838ms | 38.062ms | 247.888ms | 168 B | 4 of 4 |
| q21 | 82.641ms | 155.822ms | 1.847s | 1.847s | 0.0% | 41.720ms | 39.093ms | 324.297ms | 896 B | 5 of 5 |
| q22 | 83.315ms | 172.850ms | 2.111s | 2.111s | 0.0% | 19.943ms | 38.655ms | 720.588ms | 65.72 KiB | 6 of 6 |
| q23 | 84.591ms | 275.015ms | 3.787s | 3.787s | 0.0% | 18.583ms | 39.060ms | 684.395ms | 415.50 KiB | 6 of 6 |
| q24 | 82.863ms | 194.438ms | 1.737s | 1.737s | 0.0% | 38.119ms | 38.041ms | 624.460ms | 43.03 KiB | 8 of 8 |
| q25 | 82.099ms | 52.229ms | 525.568ms | 525.573ms | 0.0% | 14.620ms | 37.770ms | 206.657ms | 55.13 KiB | 6 of 6 |
| q26 | 83.118ms | 53.918ms | 477.537ms | 477.541ms | 0.0% | 24.297ms | 38.447ms | 494.012ms | 47.64 KiB | 5 of 5 |
| q27 | 82.715ms | 80.769ms | 662.287ms | 662.291ms | 0.0% | 31.702ms | 38.372ms | 719.337ms | 75.07 KiB | 6 of 6 |
| q28 | 87.390ms | 161.215ms | 2.163s | 2.163s | 0.0% | 33.014ms | 38.767ms | 278.281ms | 967.31 KiB | 7 of 7 |
| q29 | 87.783ms | 279.789ms | 2.668s | 2.668s | 0.0% | 19.963ms | 38.518ms | 503.287ms | 171.44 MiB | 7 of 7 |
| q30 | 85.701ms | 11.327ms | 60.291ms | 60.299ms | 0.0% | 9.473ms | 38.856ms | 150.845ms | 29.59 KiB | 4 of 4 |
| q31 | 84.828ms | 94.093ms | 966.196ms | 966.203ms | 0.0% | 37.288ms | 39.432ms | 314.365ms | 68.68 MiB | 6 of 6 |
| q32 | 86.424ms | 86.105ms | 785.992ms | 786.000ms | 0.0% | 11.324ms | 41.249ms | 442.751ms | 76.92 MiB | 6 of 6 |
| q33 | 88.608ms | 200.980ms | 923.300ms | 923.306ms | 0.0% | 11.518ms | 38.599ms | 1.268s | 657.17 MiB | 5 of 5 |
| q34 | 88.932ms | 331.041ms | 2.769s | 2.769s | 0.0% | 16.799ms | 41.135ms | 239.421ms | 939.27 MiB | 5 of 5 |
| q35 | 84.935ms | 321.527ms | 2.798s | 2.798s | 0.0% | 18.352ms | 38.610ms | 193.128ms | 940.77 MiB | 5 of 5 |
| q36 | 84.200ms | 131.072ms | 1.159s | 1.159s | 0.0% | 61.887ms | 38.724ms | 612.245ms | 273.20 MiB | 6 of 6 |
| q37 | 88.346ms | 8.008ms | 20.855ms | 20.860ms | 0.0% | 702.928us | 40.474ms | 158.666ms | 9.08 MiB | 6 of 6 |
| q38 | 105.797ms | 7.148ms | 12.443ms | 12.450ms | 0.1% | 1.547ms | 48.919ms | 238.632ms | 2.08 MiB | 6 of 6 |
| q39 | 89.017ms | 6.946ms | 16.371ms | 16.376ms | 0.0% | 6.162ms | 40.729ms | 172.895ms | 681.66 KiB | 6 of 6 |
| q40 | 91.494ms | 48.445ms | 112.964ms | 112.971ms | 0.0% | 739.277us | 41.180ms | 335.849ms | 39.70 MiB | 6 of 6 |
| q41 | 84.356ms | 3.118ms | 8.218ms | 8.222ms | 0.1% | 2.736ms | 38.534ms | 143.244ms | 758.62 KiB | 6 of 6 |
| q42 | 89.850ms | 4.157ms | 6.342ms | 6.345ms | 0.0% | 692.578us | 40.310ms | 153.345ms | 1.17 MiB | 6 of 6 |
| q43 | 83.118ms | 3.978ms | 7.723ms | 7.729ms | 0.1% | 3.050ms | 38.920ms | 143.351ms | 407.06 KiB | 6 of 6 |

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
| FileScan | 18.040s | 49.7% | 43 | 0 | 356378345 | handed none | 50.6ns | 43 of 43 |
| Aggregate | 15.225s | 41.9% | 39 | 197106902 | 917725 | 77.2ns | 16589.4ns | 39 of 39 |
| Filter | 2.517s | 6.9% | 28 | 186382627 | 30171383 | 13.5ns | 83.4ns | 28 of 28 |
| Project | 435.595ms | 1.2% | 91 | 204145225 | 204145225 | 2.1ns | 2.1ns | 91 of 91 |
| TopN | 52.549ms | 0.1% | 31 | 3977889 | 312 | 13.2ns | 168425.2ns | 31 of 31 |
| Fetch | 46.697ms | 0.1% | 1 | 10 | 10 | 4669722.7ns | 4669722.7ns | 1 of 1 |
| Sort | 10.460us | 0.0% | 1 | 15 | 15 | 697.3ns | 697.3ns | 1 of 1 |
| Limit | 0.386us | 0.0% | 1 | 10 | 10 | 38.6ns | 38.6ns | 1 of 1 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is FileScan, and the queries where it cost the most are q23 at 3.446s, q22 at 1.739s, q21 at 1.617s.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 9999750 rows, one out of every 10 of the 99997497 in the full file, which is a development loop rather than the suite
- q3 swung by 99.2% of its median, and rule two wants under 10%
- q14 swung by 24.1% of its median, and rule two wants under 10%
- q16 swung by 18.0% of its median, and rule two wants under 10%
- q18 swung by 11.3% of its median, and rule two wants under 10%
- q36 swung by 24.0% of its median, and rule two wants under 10%

These swung wider than reporting rule two allows:

- duckdb swung by 99.2% of its median on q3, and rule two wants under 10%
- duckdb-pinned swung by 48.5% of its median on q39, and rule two wants under 10%
- clickhouse-local swung by 15.0% of its median on q20, and rule two wants under 10%
- datafusion swung by 21.2% of its median on q32, and rule two wants under 10%
- polars swung by 14.3% of its median on q7, and rule two wants under 10%
- rudb swung by 15.1% of its median on q26, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

polars did not run q28, because the SQLContext has no strlen and no character length function.

polars did not run q29, because the SQLContext has no regexp_replace.

polars did not run q36, because the SQLContext rejects the repeated ClientIP output name in the group by.

polars did not run q43, because the SQLContext has no date_trunc.

So the polars column is 39 of 43 queries and its ratio is over the shared ones.

Answers differ, so this is not a comparison: q28: duckdb-pinned does not agree with duckdb: 63 numbers against 63

These were answered differently and the data does not say which is right:

- q16: ORDER BY COUNT(*) DESC LIMIT 10 over user IDs, where the tenth place is a tie between many users with the same count and the engine picks.
- q17: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q18: LIMIT 10 with no ORDER BY at all, so the ten rows are whichever ten the group by handed back first and the query did not ask for any particular ten.
- q19: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q22: ORDER BY COUNT(*) DESC LIMIT 10 over search phrases whose URL matches, where the counts at the cut are twos and ones, so which phrases fill the ten is the engine's choice. Both engines returned 4, 3, 2, 2, 2, 2, 2, 2, 1, 1 on a ten million row sample and kept different phrases for the tied places.
- q23: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q22, and the same on a one million row sample where seven of the ten places have a count of one.
- q24: ORDER BY EventTime LIMIT 10, and EventTime has one second resolution over a corpus with millions of rows a second, so the tenth row is a tie.
- q29: ORDER BY an AVG of a length DESC LIMIT 25, which is both a tie at the cut and a double computed in a different order by each engine.
- q31: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q32: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q33: ORDER BY COUNT(*) DESC LIMIT 10 over WatchID and ClientIP unfiltered, where almost every group has a count of one and the ten that come back are arbitrary.
- q39: ORDER BY PageViews DESC LIMIT 10 OFFSET 1000, which is a tie at the cut a thousand rows deeper in, where the counts are smaller and the ties are denser.
- q40: ORDER BY PageViews DESC LIMIT 10 OFFSET 1000 with ties at the cut, as q39.
- q41: ORDER BY PageViews DESC LIMIT 10 OFFSET 100 with ties at the cut, as q39.
- q43: DATE_TRUNC on a timestamp, and ClickHouse renders a DateTime in the machine's timezone where DuckDB renders a TIMESTAMP in none, so the same epoch second prints seven hours apart on gamingpc-wsl and two hours apart on server3.

So they are not checked, and a wrong answer from any engine on one of them would go unnoticed here. Every other query in the suite is checked in full.

These were answered differently and which engine is wrong is settled:

- q4: AVG(UserID) over a hundred million bigints, where the pinned DuckDB sums the column short by a multiple of 2^64 and rudb does not. Its own hugeint and decimal paths, the same column summed out of a table, its own per counter group sums, and adding the column up outside a database all give rudb's answer, and it reduces to two statements over a 1.3 MB one column parquet file. Written up as 14.10.1 in tamnd/rudb `spec/14-testing.md`, which is the list of places where DuckDB is wrong and we are not. tamnd/rudb-compat#12.

So they are a known difference rather than an open one, and the write up is where to go to disagree with that.

Hot here means page cache warm and not buffer pool warm, because every run is a fresh process so that no query's number depends on the one before it.

