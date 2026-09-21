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
| timeout | 1800s per query, and no query reached it |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --rows 10000000 --engines duckdb,duckdb-pinned,clickhouse-local,datafusion,polars,rudb --runs 5 --timeout 1800 --report
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
| filesystem | tmpfs /tmp tmpfs rw,nosuid,nodev,size=16430944k,nr_inodes=1048576 0 0 | read |
| page cache | cannot open drop_caches: Permission denied (os error 13) | not read |
| timer | /usr/bin/time GNU | read |

A fact that says it was not read is a fact about this machine and not a gap in the report. A field that had silently defaulted would be a lie that survived into it.

## The engines

| engine | version | state | load | load cpu | on disk | that size is | format | machine load |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 10.550s | 80.490s | 2.72 GiB | its own database file | its own | 12.74 to 15.06 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 20.249s | 82.430s | 2.30 GiB | its own database file | its own | 15.06 to 18.26 |
| clickhouse-local | 26.9.1.1562 | ran | 3.127s | 27.290s | 2.00 GiB | its own MergeTree parts, as system.parts counts the active ones | its own | 18.26 to 14.66 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 1.87 GiB | the source Parquet, this engine has no storage format of its own | the Parquet | 14.66 to 20.28 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 1.87 GiB | the source Parquet, this engine has no storage format of its own | the Parquet | 20.28 to 15.01 |
| rudb | rudb 0.3.67 | ran | 0.000us | 0.000us | 1.87 GiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 15.01 to 10.00 |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

These engines started while the machine was already above half of that, which means they were measured against somebody else's work rather than on an idle box: clickhouse-local, polars. Their numbers are inflated by an amount nothing here can recover, and by a different amount each, depending on how much of the column was CPU bound. One case where that reading is unfair is a sweep that runs engines back to back, where the average has not had a minute to come down from the engine before.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb on 39 shared |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 3.831s | 4.932s | +29% | 5.124s | 41.020s | 8.32 | 1.73 GiB | none | 112.24M/s | 20.99 GiB/s | 1.00x |
| duckdb-pinned | 3.078s | 4.680s | +52% | 4.719s | 38.490s | 8.22 | 1.61 GiB | none | 139.70M/s | 26.12 GiB/s | 0.96x |
| clickhouse-local | 8.009s | 16.570s | +107% | 16.475s | 59.410s | 3.59 | 1.40 GiB | none | 53.69M/s | 10.04 GiB/s | 2.75x |
| datafusion | 13.518s | 17.545s | +30% | 17.357s | 239.250s | 13.64 | 4.49 GiB | none | 31.81M/s | 5.95 GiB/s | 4.60x |
| polars | 4.902s | 9.217s | +88% | 9.382s | 55.930s | 6.07 | 2.95 GiB | none | 79.56M/s | 14.88 GiB/s | 1.84x |
| rudb | 5.619s | 8.283s | +47% | 8.508s | 40.730s | 4.92 | 1.34 GiB | none | 76.53M/s | 14.31 GiB/s | 1.87x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion | polars | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 2.000ms | 5.000ms | 1.000ms | 39.361ms | 63.575ms |
| q2 | filtered count | 7.000ms | 8.000ms | 5.000ms | 247.000ms | 41.634ms | 69.781ms |
| q3 | three aggregates | 10.000ms | 10.000ms | 98.000ms | 259.000ms | 49.559ms | 73.405ms |
| q4 | average | 14.000ms | 15.000ms | 75.000ms | 245.000ms | 51.650ms | 77.170ms |
| q5 | count distinct, high card | 54.000ms | 46.000ms | 111.000ms | 291.000ms | 97.269ms | 89.045ms |
| q6 | count distinct, strings | 37.000ms | 35.000ms | 213.000ms | 283.000ms | 93.686ms | 163.802ms |
| q7 | min and max of a date | 2.000ms | 3.000ms | 258.000ms | 1.000ms | 47.719ms | 72.526ms |
| q8 | group by, low card | 8.000ms | 15.000ms | 262.000ms | 249.000ms | 50.886ms | 71.019ms |
| q9 | group by and count distinct | 69.000ms | 78.000ms | 161.000ms | 307.000ms | 163.073ms | 121.503ms |
| q10 | group by, several aggregates | 100.000ms | 129.000ms | 193.000ms | 326.000ms | 171.601ms | 130.313ms |
| q11 | group by a string and count distinct | 26.000ms | 28.000ms | 124.000ms | 545.000ms | 79.728ms | 100.480ms |
| q12 | group by two strings and count distinct | 32.000ms | 30.000ms | 119.000ms | 507.000ms | 84.248ms | 95.799ms |
| q13 | group by a string and top k | 34.000ms | 37.000ms | 120.000ms | 531.000ms | 103.936ms | 114.998ms |
| q14 | group by a string and count distinct | 62.000ms | 106.000ms | 148.000ms | 339.000ms | 140.952ms | 135.862ms |
| q15 | group by two columns and top k | 38.000ms | 63.000ms | 148.000ms | 289.000ms | 104.060ms | 125.319ms |
| q16 | group by, very high card | 56.000ms | 56.000ms | 93.000ms | 299.000ms | 122.609ms | 156.778ms |
| q17 | group by two, very high card | 156.000ms | 158.000ms | 214.000ms | 381.000ms | 192.217ms | 194.992ms |
| q18 | group by two, no ordering | 132.000ms | 136.000ms | 144.000ms | 377.000ms | 181.458ms | 116.322ms |
| q19 | group by with an extract | 226.000ms | 210.000ms | 279.000ms | 427.000ms | 240.782ms | 256.758ms |
| q20 | point lookup | 15.000ms | 14.000ms | 259.000ms | 247.000ms | 44.708ms | 73.732ms |
| q21 | substring scan | 101.000ms | 78.000ms | 175.000ms | 308.000ms | 175.396ms | 157.446ms |
| q22 | substring scan and group by | 129.000ms | 110.000ms | 195.000ms | 320.000ms | 187.117ms | 170.548ms |
| q23 | two substring scans and group by | 192.000ms | 158.000ms | 240.000ms | 408.000ms | 278.800ms | 272.534ms |
| q24 | select star and top k | 169.000ms | 109.000ms | 409.000ms | 619.000ms | 503.748ms | 224.603ms |
| q25 | top k by a date | 15.000ms | 15.000ms | 280.000ms | 244.000ms | 80.294ms | 109.636ms |
| q26 | top k by a string | 18.000ms | 21.000ms | 102.000ms | 267.000ms | 76.277ms | 97.489ms |
| q27 | top k by two columns | 21.000ms | 17.000ms | 293.000ms | 245.000ms | 88.094ms | 103.717ms |
| q28 | group by with a string length | 113.000ms | 110.000ms | 109.000ms | 320.000ms | no dialect | 178.526ms |
| q29 | group by a regular expression | 998.000ms | 358.000ms | 231.000ms | 429.000ms | no dialect | 254.323ms |
| q30 | ninety sums over one column | 11.000ms | 20.000ms | 71.000ms | 251.000ms | 77.483ms | 75.518ms |
| q31 | group by two and several aggregates | 71.000ms | 87.000ms | 134.000ms | 300.000ms | 109.037ms | 122.402ms |
| q32 | group by a high card pair | 59.000ms | 84.000ms | 139.000ms | 309.000ms | 100.158ms | 129.873ms |
| q33 | group by a high card pair, unfiltered | 148.000ms | 156.000ms | 214.000ms | 411.000ms | 234.686ms | 156.970ms |
| q34 | group by a long string | 283.000ms | 216.000ms | 242.000ms | 450.000ms | 274.914ms | 310.474ms |
| q35 | group by a constant and a long string | 306.000ms | 235.000ms | 253.000ms | 456.000ms | 311.725ms | 317.693ms |
| q36 | group by four expressions | 56.000ms | 50.000ms | 97.000ms | 293.000ms | no dialect | 135.661ms |
| q37 | date range and group by a URL | 12.000ms | 12.000ms | 257.000ms | 253.000ms | 53.680ms | 73.393ms |
| q38 | date range and group by a title | 8.000ms | 9.000ms | 258.000ms | 248.000ms | 49.735ms | 67.077ms |
| q39 | date range, group by and offset | 7.000ms | 8.000ms | 255.000ms | 243.000ms | 46.678ms | 68.775ms |
| q40 | date range, a case and a wide group by | 20.000ms | 20.000ms | 277.000ms | 248.000ms | 61.302ms | 87.363ms |
| q41 | date range with an IN and a hash | 5.000ms | 8.000ms | 253.000ms | 248.000ms | 45.651ms | 66.960ms |
| q42 | date range and a deep offset | 5.000ms | 11.000ms | 242.000ms | 250.000ms | 46.122ms | 67.554ms |
| q43 | minute buckets over a date range | 5.000ms | 7.000ms | 254.000ms | 247.000ms | no dialect | 67.057ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 20.235ms | 20.241ms | 0.0% | 20.240ms | 20.244ms | 20.236ms | 20.261ms | 0.000us | 27.79 MiB | none | 494.02M/s |
| q2 | filtered count | 7.000ms | 40.563ms | 20.286ms | 0.3% | 20.281ms | 20.338ms | 20.272ms | 20.384ms | 20.000ms | 52.04 MiB | none | 492.93M/s |
| q3 | three aggregates | 10.000ms | 40.357ms | 40.380ms | 0.2% | 40.329ms | 40.413ms | 20.262ms | 40.750ms | 50.000ms | 73.02 MiB | 4.00 KiB | 247.64M/s |
| q4 | average | 14.000ms | 40.331ms | 40.404ms | 0.2% | 40.358ms | 40.453ms | 40.349ms | 41.008ms | 80.000ms | 99.22 MiB | none | 247.49M/s |
| q5 | count distinct, high card | 54.000ms | 101.011ms | 80.747ms | 0.3% | 80.673ms | 80.939ms | 80.651ms | 100.650ms | 930.000ms | 354.48 MiB | none | 123.84M/s |
| q6 | count distinct, strings | 37.000ms | 60.804ms | 60.491ms | 0.1% | 60.489ms | 60.533ms | 60.470ms | 60.641ms | 620.000ms | 290.23 MiB | none | 165.31M/s |
| q7 | min and max of a date | 2.000ms | 20.335ms | 20.247ms | 0.3% | 20.242ms | 20.300ms | 20.224ms | 20.305ms | 0.000us | 29.48 MiB | none | 493.88M/s |
| q8 | group by, low card | 8.000ms | 40.551ms | 20.253ms | 0.0% | 20.247ms | 20.255ms | 20.246ms | 20.317ms | 20.000ms | 53.74 MiB | none | 493.75M/s |
| q9 | group by and count distinct | 69.000ms | 120.723ms | 103.119ms | 2.0% | 101.262ms | 103.293ms | 100.674ms | 104.344ms | 1.170s | 420.74 MiB | none | 96.97M/s |
| q10 | group by, several aggregates | 100.000ms | 140.786ms | 123.282ms | 1.2% | 122.617ms | 124.039ms | 120.697ms | 141.547ms | 1.500s | 483.99 MiB | none | 81.11M/s |
| q11 | group by a string and count distinct | 26.000ms | 40.363ms | 40.357ms | 0.0% | 40.356ms | 40.370ms | 40.355ms | 40.372ms | 260.000ms | 196.99 MiB | none | 247.78M/s |
| q12 | group by two strings and count distinct | 32.000ms | 40.530ms | 60.450ms | 0.1% | 60.444ms | 60.486ms | 40.346ms | 82.188ms | 240.000ms | 207.74 MiB | none | 165.42M/s |
| q13 | group by a string and top k | 34.000ms | 60.479ms | 60.545ms | 0.2% | 60.493ms | 60.620ms | 60.460ms | 60.660ms | 460.000ms | 307.99 MiB | none | 165.16M/s |
| q14 | group by a string and count distinct | 62.000ms | 80.653ms | 83.999ms | 47.6% | 80.764ms | 120.775ms | 80.697ms | 140.802ms | 1.000s | 592.68 MiB | none | 119.05M/s |
| q15 | group by two columns and top k | 38.000ms | 60.531ms | 60.489ms | 0.0% | 60.487ms | 60.498ms | 60.485ms | 82.760ms | 540.000ms | 375.19 MiB | none | 165.31M/s |
| q16 | group by, very high card | 56.000ms | 81.073ms | 82.832ms | 4.1% | 80.625ms | 84.030ms | 80.577ms | 103.906ms | 1.210s | 429.23 MiB | none | 120.72M/s |
| q17 | group by two, very high card | 156.000ms | 164.295ms | 188.956ms | 8.7% | 185.185ms | 201.635ms | 184.146ms | 203.960ms | 1.990s | 824.50 MiB | none | 52.92M/s |
| q18 | group by two, no ordering | 132.000ms | 180.885ms | 161.066ms | 1.5% | 161.050ms | 163.440ms | 160.982ms | 181.376ms | 1.470s | 773.09 MiB | none | 62.08M/s |
| q19 | group by with an extract | 226.000ms | 263.392ms | 268.101ms | 7.7% | 267.365ms | 288.074ms | 262.395ms | 289.288ms | 3.080s | 1.13 GiB | none | 37.30M/s |
| q20 | point lookup | 15.000ms | 41.461ms | 40.431ms | 0.1% | 40.422ms | 40.481ms | 40.360ms | 40.607ms | 60.000ms | 98.73 MiB | none | 247.33M/s |
| q21 | substring scan | 101.000ms | 161.497ms | 120.973ms | 16.5% | 120.898ms | 140.845ms | 120.828ms | 141.480ms | 1.060s | 643.23 MiB | none | 82.66M/s |
| q22 | substring scan and group by | 129.000ms | 181.052ms | 160.987ms | 1.0% | 160.878ms | 162.439ms | 160.870ms | 163.127ms | 1.060s | 722.48 MiB | none | 62.12M/s |
| q23 | two substring scans and group by | 192.000ms | 242.992ms | 221.199ms | 0.1% | 221.167ms | 221.282ms | 221.085ms | 221.958ms | 1.170s | 781.19 MiB | none | 45.21M/s |
| q24 | select star and top k | 169.000ms | 181.916ms | 202.391ms | 9.8% | 201.302ms | 221.119ms | 201.023ms | 242.328ms | 1.110s | 642.85 MiB | none | 49.41M/s |
| q25 | top k by a date | 15.000ms | 40.366ms | 40.365ms | 0.1% | 40.355ms | 40.398ms | 40.348ms | 40.576ms | 60.000ms | 74.73 MiB | none | 247.73M/s |
| q26 | top k by a string | 18.000ms | 60.894ms | 40.470ms | 0.2% | 40.442ms | 40.517ms | 40.422ms | 40.992ms | 200.000ms | 105.44 MiB | none | 247.09M/s |
| q27 | top k by two columns | 21.000ms | 40.362ms | 40.380ms | 0.3% | 40.361ms | 40.471ms | 40.331ms | 40.493ms | 70.000ms | 74.98 MiB | none | 247.64M/s |
| q28 | group by with a string length | 113.000ms | 161.242ms | 141.074ms | 0.5% | 140.918ms | 141.646ms | 140.761ms | 161.924ms | 970.000ms | 677.24 MiB | none | 70.88M/s |
| q29 | group by a regular expression | 998.000ms | 1.045s | 1.045s | 0.2% | 1.045s | 1.046s | 1.028s | 1.067s | 7.690s | 959.47 MiB | none | 9.57M/s |
| q30 | ninety sums over one column | 11.000ms | 40.343ms | 40.340ms | 0.1% | 40.317ms | 40.355ms | 20.259ms | 40.358ms | 40.000ms | 57.22 MiB | none | 247.89M/s |
| q31 | group by two and several aggregates | 71.000ms | 121.086ms | 100.600ms | 0.8% | 100.597ms | 101.374ms | 100.590ms | 102.117ms | 570.000ms | 360.30 MiB | none | 99.40M/s |
| q32 | group by a high card pair | 59.000ms | 80.586ms | 80.553ms | 0.1% | 80.551ms | 80.639ms | 80.491ms | 80.830ms | 620.000ms | 428.04 MiB | none | 124.14M/s |
| q33 | group by a high card pair, unfiltered | 148.000ms | 209.155ms | 187.415ms | 9.3% | 185.608ms | 203.105ms | 184.656ms | 231.747ms | 3.140s | 1.20 GiB | none | 53.36M/s |
| q34 | group by a long string | 283.000ms | 327.580ms | 327.783ms | 6.8% | 323.944ms | 346.391ms | 322.549ms | 347.152ms | 3.470s | 1.71 GiB | none | 30.51M/s |
| q35 | group by a constant and a long string | 306.000ms | 304.027ms | 342.961ms | 11.4% | 325.177ms | 364.326ms | 307.564ms | 367.826ms | 3.640s | 1.73 GiB | none | 29.16M/s |
| q36 | group by four expressions | 56.000ms | 84.268ms | 80.730ms | 0.9% | 80.615ms | 81.332ms | 80.587ms | 83.120ms | 1.270s | 481.79 MiB | none | 123.87M/s |
| q37 | date range and group by a URL | 12.000ms | 40.343ms | 40.394ms | 0.1% | 40.352ms | 40.400ms | 40.350ms | 40.578ms | 30.000ms | 62.57 MiB | none | 247.55M/s |
| q38 | date range and group by a title | 8.000ms | 40.349ms | 20.300ms | 0.0% | 20.299ms | 20.305ms | 20.249ms | 20.316ms | 20.000ms | 47.50 MiB | none | 492.61M/s |
| q39 | date range, group by and offset | 7.000ms | 20.254ms | 20.244ms | 0.1% | 20.240ms | 20.258ms | 20.240ms | 20.266ms | 20.000ms | 47.43 MiB | none | 493.97M/s |
| q40 | date range, a case and a wide group by | 20.000ms | 40.345ms | 40.363ms | 0.1% | 40.343ms | 40.394ms | 40.342ms | 40.395ms | 50.000ms | 90.76 MiB | none | 247.74M/s |
| q41 | date range with an IN and a hash | 5.000ms | 20.323ms | 20.262ms | 0.0% | 20.260ms | 20.269ms | 20.255ms | 20.282ms | 20.000ms | 44.99 MiB | none | 493.51M/s |
| q42 | date range and a deep offset | 5.000ms | 20.273ms | 20.248ms | 0.0% | 20.248ms | 20.256ms | 20.245ms | 20.261ms | 20.000ms | 43.24 MiB | none | 493.86M/s |
| q43 | minute buckets over a date range | 5.000ms | 20.253ms | 20.263ms | 0.0% | 20.261ms | 20.263ms | 20.254ms | 20.272ms | 20.000ms | 39.75 MiB | none | 493.50M/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 43 of 43 queries. Total 3.831s by its own clock and 4.932s by ours, 5.124s cold, 41.020s of CPU, peak 1.73 GiB, 112.24M/s and 20.99 GiB/s.

Running it cost 29% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 51.62x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 2.000ms | 40.345ms | 40.410ms | 0.3% | 40.377ms | 40.517ms | 40.338ms | 40.528ms | 20.000ms | 40.81 MiB | none | 247.46M/s |
| q2 | filtered count | 8.000ms | 40.523ms | 40.405ms | 0.1% | 40.394ms | 40.426ms | 40.384ms | 40.436ms | 40.000ms | 65.06 MiB | none | 247.49M/s |
| q3 | three aggregates | 10.000ms | 60.450ms | 40.350ms | 0.1% | 40.347ms | 40.369ms | 40.339ms | 40.682ms | 50.000ms | 87.05 MiB | none | 247.82M/s |
| q4 | average | 15.000ms | 64.380ms | 40.350ms | 0.1% | 40.349ms | 40.394ms | 40.332ms | 40.730ms | 100.000ms | 112.55 MiB | none | 247.82M/s |
| q5 | count distinct, high card | 46.000ms | 80.483ms | 80.621ms | 2.8% | 80.571ms | 82.816ms | 80.546ms | 83.602ms | 830.000ms | 369.55 MiB | 32.00 KiB | 124.03M/s |
| q6 | count distinct, strings | 35.000ms | 60.527ms | 80.516ms | 0.0% | 80.512ms | 80.518ms | 60.439ms | 84.595ms | 380.000ms | 329.07 MiB | none | 124.20M/s |
| q7 | min and max of a date | 3.000ms | 40.325ms | 40.339ms | 0.0% | 40.337ms | 40.357ms | 40.329ms | 40.431ms | 10.000ms | 42.38 MiB | none | 247.89M/s |
| q8 | group by, low card | 15.000ms | 40.360ms | 40.385ms | 1.7% | 40.351ms | 41.050ms | 40.336ms | 60.459ms | 50.000ms | 69.02 MiB | none | 247.61M/s |
| q9 | group by and count distinct | 78.000ms | 104.560ms | 121.540ms | 1.0% | 121.433ms | 122.611ms | 104.062ms | 125.670ms | 860.000ms | 432.44 MiB | none | 82.28M/s |
| q10 | group by, several aggregates | 129.000ms | 168.745ms | 164.229ms | 11.4% | 162.437ms | 181.113ms | 161.252ms | 181.285ms | 1.110s | 495.61 MiB | 72.00 KiB | 60.89M/s |
| q11 | group by a string and count distinct | 28.000ms | 60.753ms | 60.454ms | 0.3% | 60.443ms | 60.648ms | 60.428ms | 60.906ms | 230.000ms | 210.50 MiB | none | 165.41M/s |
| q12 | group by two strings and count distinct | 30.000ms | 60.439ms | 60.527ms | 0.3% | 60.520ms | 60.680ms | 60.483ms | 80.636ms | 250.000ms | 221.58 MiB | none | 165.21M/s |
| q13 | group by a string and top k | 37.000ms | 80.514ms | 80.736ms | 0.3% | 80.531ms | 80.764ms | 80.518ms | 81.031ms | 440.000ms | 361.13 MiB | none | 123.86M/s |
| q14 | group by a string and count distinct | 106.000ms | 100.694ms | 147.894ms | 16.1% | 142.910ms | 166.669ms | 140.901ms | 166.878ms | 1.070s | 618.10 MiB | none | 67.61M/s |
| q15 | group by two columns and top k | 63.000ms | 80.551ms | 100.621ms | 18.5% | 82.011ms | 100.662ms | 80.564ms | 100.682ms | 560.000ms | 438.50 MiB | none | 99.38M/s |
| q16 | group by, very high card | 56.000ms | 107.572ms | 100.932ms | 2.1% | 100.688ms | 102.779ms | 84.013ms | 104.239ms | 1.000s | 436.10 MiB | none | 99.07M/s |
| q17 | group by two, very high card | 158.000ms | 203.479ms | 204.024ms | 0.4% | 203.890ms | 204.647ms | 203.547ms | 224.020ms | 1.870s | 848.10 MiB | none | 49.01M/s |
| q18 | group by two, no ordering | 136.000ms | 183.917ms | 180.999ms | 1.3% | 180.978ms | 183.362ms | 161.080ms | 189.184ms | 1.170s | 806.85 MiB | none | 55.25M/s |
| q19 | group by with an extract | 210.000ms | 245.476ms | 264.940ms | 0.7% | 264.164ms | 266.023ms | 249.181ms | 266.757ms | 2.950s | 1.17 GiB | none | 37.74M/s |
| q20 | point lookup | 14.000ms | 60.508ms | 40.376ms | 0.2% | 40.354ms | 40.443ms | 40.340ms | 40.700ms | 70.000ms | 112.90 MiB | none | 247.67M/s |
| q21 | substring scan | 78.000ms | 120.923ms | 121.244ms | 0.4% | 120.810ms | 121.291ms | 120.698ms | 121.524ms | 1.000s | 514.30 MiB | none | 82.48M/s |
| q22 | substring scan and group by | 110.000ms | 140.787ms | 141.089ms | 14.3% | 140.866ms | 160.975ms | 140.795ms | 161.841ms | 1.030s | 622.32 MiB | none | 70.88M/s |
| q23 | two substring scans and group by | 158.000ms | 201.084ms | 201.511ms | 9.8% | 201.305ms | 221.131ms | 181.121ms | 221.341ms | 1.210s | 744.13 MiB | none | 49.62M/s |
| q24 | select star and top k | 109.000ms | 162.547ms | 141.279ms | 14.5% | 140.979ms | 161.415ms | 140.827ms | 161.790ms | 720.000ms | 507.91 MiB | none | 70.78M/s |
| q25 | top k by a date | 15.000ms | 40.327ms | 40.495ms | 49.6% | 40.338ms | 60.433ms | 40.329ms | 60.473ms | 70.000ms | 99.82 MiB | none | 246.94M/s |
| q26 | top k by a string | 21.000ms | 60.398ms | 60.451ms | 0.1% | 60.423ms | 60.457ms | 60.417ms | 60.595ms | 160.000ms | 147.57 MiB | none | 165.42M/s |
| q27 | top k by two columns | 17.000ms | 40.329ms | 40.336ms | 50.0% | 40.333ms | 60.501ms | 40.325ms | 62.623ms | 80.000ms | 101.32 MiB | none | 247.91M/s |
| q28 | group by with a string length | 110.000ms | 162.059ms | 141.877ms | 14.0% | 141.381ms | 161.221ms | 140.979ms | 162.326ms | 1.010s | 562.07 MiB | none | 70.48M/s |
| q29 | group by a regular expression | 358.000ms | 408.361ms | 411.427ms | 5.1% | 391.329ms | 412.309ms | 388.048ms | 415.699ms | 8.200s | 1.12 GiB | none | 24.31M/s |
| q30 | ninety sums over one column | 20.000ms | 60.465ms | 60.506ms | 0.2% | 60.421ms | 60.568ms | 60.415ms | 60.579ms | 60.000ms | 76.94 MiB | none | 165.27M/s |
| q31 | group by two and several aggregates | 87.000ms | 140.918ms | 120.725ms | 0.5% | 120.705ms | 121.284ms | 120.675ms | 121.693ms | 580.000ms | 408.40 MiB | none | 82.83M/s |
| q32 | group by a high card pair | 84.000ms | 120.734ms | 120.920ms | 1.5% | 120.703ms | 122.524ms | 120.653ms | 140.768ms | 600.000ms | 482.13 MiB | none | 82.70M/s |
| q33 | group by a high card pair, unfiltered | 156.000ms | 222.578ms | 206.168ms | 8.9% | 205.235ms | 223.577ms | 204.466ms | 244.266ms | 3.030s | 1.19 GiB | none | 48.50M/s |
| q34 | group by a long string | 216.000ms | 263.739ms | 268.191ms | 6.9% | 266.588ms | 285.073ms | 263.778ms | 286.348ms | 3.210s | 1.58 GiB | none | 37.29M/s |
| q35 | group by a constant and a long string | 235.000ms | 286.156ms | 287.877ms | 1.5% | 287.636ms | 291.993ms | 266.637ms | 305.861ms | 3.420s | 1.61 GiB | none | 34.74M/s |
| q36 | group by four expressions | 50.000ms | 80.564ms | 82.311ms | 24.3% | 80.709ms | 100.747ms | 80.703ms | 124.440ms | 760.000ms | 382.63 MiB | none | 121.49M/s |
| q37 | date range and group by a URL | 12.000ms | 40.386ms | 40.359ms | 0.1% | 40.341ms | 40.376ms | 40.332ms | 41.696ms | 50.000ms | 76.01 MiB | none | 247.77M/s |
| q38 | date range and group by a title | 9.000ms | 40.387ms | 40.424ms | 0.2% | 40.377ms | 40.447ms | 40.320ms | 63.119ms | 30.000ms | 59.77 MiB | none | 247.37M/s |
| q39 | date range, group by and offset | 8.000ms | 40.335ms | 40.383ms | 0.2% | 40.338ms | 40.431ms | 40.333ms | 40.646ms | 40.000ms | 61.01 MiB | none | 247.63M/s |
| q40 | date range, a case and a wide group by | 20.000ms | 60.630ms | 60.432ms | 0.1% | 60.424ms | 60.476ms | 60.419ms | 60.545ms | 70.000ms | 102.07 MiB | none | 165.47M/s |
| q41 | date range with an IN and a hash | 8.000ms | 40.381ms | 40.441ms | 0.2% | 40.412ms | 40.487ms | 40.326ms | 40.672ms | 30.000ms | 56.60 MiB | none | 247.27M/s |
| q42 | date range and a deep offset | 11.000ms | 60.400ms | 40.383ms | 0.3% | 40.331ms | 40.447ms | 40.320ms | 43.839ms | 40.000ms | 55.32 MiB | none | 247.62M/s |
| q43 | minute buckets over a date range | 7.000ms | 40.335ms | 40.333ms | 0.4% | 40.324ms | 40.484ms | 40.313ms | 40.871ms | 30.000ms | 52.57 MiB | none | 247.93M/s |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 3.078s by its own clock and 4.680s by ours, 4.719s cold, 38.490s of CPU, peak 1.61 GiB, 139.70M/s and 26.12 GiB/s.

Running it cost 52% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 10.20x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 5.000ms | 261.619ms | 261.258ms | 7.3% | 242.376ms | 261.383ms | 241.284ms | 263.231ms | 440.000ms | 253.22 MiB | none | 38.28M/s |
| q2 | filtered count | 5.000ms | 241.332ms | 242.353ms | 24.8% | 241.262ms | 301.423ms | 241.121ms | 362.281ms | 420.000ms | 259.00 MiB | none | 41.26M/s |
| q3 | three aggregates | 98.000ms | 343.369ms | 383.349ms | 26.4% | 321.465ms | 422.723ms | 321.390ms | 482.728ms | 680.000ms | 288.32 MiB | none | 26.09M/s |
| q4 | average | 75.000ms | 283.000ms | 321.370ms | 5.6% | 303.574ms | 321.427ms | 301.310ms | 351.494ms | 520.000ms | 304.43 MiB | none | 31.12M/s |
| q5 | count distinct, high card | 111.000ms | 362.396ms | 381.739ms | 5.0% | 363.242ms | 382.152ms | 361.922ms | 402.264ms | 1.460s | 812.22 MiB | none | 26.20M/s |
| q6 | count distinct, strings | 213.000ms | 443.743ms | 450.386ms | 31.2% | 381.989ms | 522.554ms | 361.755ms | 646.123ms | 1.140s | 556.91 MiB | none | 22.20M/s |
| q7 | min and max of a date | 258.000ms | 377.983ms | 404.770ms | 32.8% | 301.689ms | 434.605ms | 281.344ms | 488.577ms | 650.000ms | 348.89 MiB | none | 24.70M/s |
| q8 | group by, low card | 262.000ms | 362.522ms | 381.614ms | 5.3% | 361.569ms | 381.636ms | 361.555ms | 401.668ms | 750.000ms | 368.59 MiB | none | 26.20M/s |
| q9 | group by and count distinct | 161.000ms | 383.999ms | 381.775ms | 4.3% | 365.896ms | 382.279ms | 363.536ms | 383.439ms | 1.470s | 655.41 MiB | none | 26.19M/s |
| q10 | group by, several aggregates | 193.000ms | 404.102ms | 401.700ms | 3.3% | 389.060ms | 402.273ms | 385.387ms | 402.932ms | 1.920s | 654.02 MiB | none | 24.89M/s |
| q11 | group by a string and count distinct | 124.000ms | 361.520ms | 342.553ms | 0.3% | 341.983ms | 343.001ms | 341.776ms | 343.735ms | 670.000ms | 373.52 MiB | none | 29.19M/s |
| q12 | group by two strings and count distinct | 119.000ms | 349.400ms | 361.840ms | 0.1% | 361.541ms | 361.910ms | 341.434ms | 382.363ms | 820.000ms | 386.16 MiB | none | 27.64M/s |
| q13 | group by a string and top k | 120.000ms | 362.047ms | 361.840ms | 0.1% | 361.796ms | 361.979ms | 321.527ms | 363.973ms | 1.010s | 551.80 MiB | none | 27.64M/s |
| q14 | group by a string and count distinct | 148.000ms | 381.715ms | 381.772ms | 5.5% | 381.728ms | 402.629ms | 361.637ms | 405.641ms | 1.380s | 620.08 MiB | none | 26.19M/s |
| q15 | group by two columns and top k | 148.000ms | 382.039ms | 382.268ms | 6.6% | 364.000ms | 389.267ms | 361.947ms | 401.902ms | 1.160s | 603.11 MiB | none | 26.16M/s |
| q16 | group by, very high card | 93.000ms | 342.604ms | 321.984ms | 6.2% | 321.757ms | 341.781ms | 301.503ms | 361.725ms | 1.160s | 598.21 MiB | none | 31.06M/s |
| q17 | group by two, very high card | 214.000ms | 422.554ms | 442.552ms | 3.4% | 428.068ms | 442.941ms | 425.609ms | 444.657ms | 3.080s | 1018.07 MiB | none | 22.60M/s |
| q18 | group by two, no ordering | 144.000ms | 362.131ms | 384.300ms | 1.9% | 382.354ms | 389.785ms | 347.204ms | 473.887ms | 1.230s | 513.16 MiB | none | 26.02M/s |
| q19 | group by with an extract | 279.000ms | 464.698ms | 496.169ms | 7.1% | 471.990ms | 507.012ms | 468.430ms | 607.842ms | 4.690s | 1.31 GiB | none | 20.15M/s |
| q20 | point lookup | 259.000ms | 466.788ms | 381.902ms | 0.3% | 381.700ms | 382.850ms | 341.529ms | 401.908ms | 780.000ms | 394.17 MiB | none | 26.18M/s |
| q21 | substring scan | 175.000ms | 405.550ms | 402.016ms | 0.4% | 401.917ms | 403.430ms | 362.490ms | 406.270ms | 1.450s | 473.01 MiB | none | 24.87M/s |
| q22 | substring scan and group by | 195.000ms | 463.324ms | 402.562ms | 0.4% | 402.055ms | 403.561ms | 381.835ms | 442.755ms | 1.760s | 543.99 MiB | none | 24.84M/s |
| q23 | two substring scans and group by | 240.000ms | 462.454ms | 422.184ms | 5.2% | 421.980ms | 443.906ms | 421.915ms | 462.061ms | 2.230s | 568.61 MiB | none | 23.69M/s |
| q24 | select star and top k | 409.000ms | 509.852ms | 502.255ms | 4.0% | 502.186ms | 522.190ms | 482.161ms | 522.399ms | 1.630s | 578.20 MiB | none | 19.91M/s |
| q25 | top k by a date | 280.000ms | 405.076ms | 401.803ms | 0.1% | 401.794ms | 402.183ms | 401.727ms | 402.204ms | 900.000ms | 439.68 MiB | none | 24.89M/s |
| q26 | top k by a string | 102.000ms | 342.269ms | 342.297ms | 0.2% | 341.569ms | 342.365ms | 341.528ms | 342.806ms | 670.000ms | 408.08 MiB | none | 29.21M/s |
| q27 | top k by two columns | 293.000ms | 425.943ms | 421.793ms | 4.8% | 401.847ms | 421.958ms | 401.828ms | 424.952ms | 910.000ms | 440.96 MiB | none | 23.71M/s |
| q28 | group by with a string length | 109.000ms | 301.307ms | 341.600ms | 1.1% | 341.583ms | 345.234ms | 321.394ms | 361.576ms | 640.000ms | 350.70 MiB | none | 29.27M/s |
| q29 | group by a regular expression | 231.000ms | 402.827ms | 447.923ms | 1.3% | 444.132ms | 449.762ms | 442.939ms | 463.134ms | 3.790s | 1006.52 MiB | none | 22.32M/s |
| q30 | ninety sums over one column | 71.000ms | 301.602ms | 301.290ms | 0.1% | 301.279ms | 301.436ms | 301.242ms | 301.541ms | 480.000ms | 278.65 MiB | none | 33.19M/s |
| q31 | group by two and several aggregates | 134.000ms | 366.512ms | 381.943ms | 5.5% | 362.149ms | 383.007ms | 341.559ms | 383.929ms | 1.050s | 468.22 MiB | none | 26.18M/s |
| q32 | group by a high card pair | 139.000ms | 361.938ms | 369.148ms | 5.7% | 362.067ms | 383.023ms | 361.794ms | 389.928ms | 1.250s | 516.84 MiB | none | 27.09M/s |
| q33 | group by a high card pair, unfiltered | 214.000ms | 423.204ms | 443.754ms | 0.9% | 442.467ms | 446.381ms | 423.330ms | 459.846ms | 3.430s | 1016.35 MiB | none | 22.53M/s |
| q34 | group by a long string | 242.000ms | 485.526ms | 463.612ms | 0.5% | 463.342ms | 465.679ms | 444.363ms | 483.121ms | 3.680s | 1.40 GiB | none | 21.57M/s |
| q35 | group by a constant and a long string | 253.000ms | 483.383ms | 464.341ms | 4.0% | 464.298ms | 482.719ms | 445.266ms | 483.920ms | 3.690s | 1.40 GiB | none | 21.54M/s |
| q36 | group by four expressions | 97.000ms | 341.537ms | 341.547ms | 5.8% | 321.907ms | 341.820ms | 321.533ms | 362.037ms | 1.050s | 566.83 MiB | none | 29.28M/s |
| q37 | date range and group by a URL | 257.000ms | 382.693ms | 382.018ms | 0.2% | 381.752ms | 382.372ms | 381.629ms | 402.408ms | 770.000ms | 395.20 MiB | none | 26.18M/s |
| q38 | date range and group by a title | 258.000ms | 381.736ms | 381.617ms | 0.0% | 381.597ms | 381.720ms | 381.553ms | 381.794ms | 790.000ms | 378.32 MiB | none | 26.20M/s |
| q39 | date range, group by and offset | 255.000ms | 381.628ms | 381.575ms | 5.3% | 361.768ms | 381.833ms | 361.466ms | 382.123ms | 760.000ms | 375.87 MiB | none | 26.21M/s |
| q40 | date range, a case and a wide group by | 277.000ms | 381.891ms | 401.720ms | 5.0% | 381.688ms | 401.764ms | 381.672ms | 401.954ms | 850.000ms | 431.02 MiB | none | 24.89M/s |
| q41 | date range with an IN and a hash | 253.000ms | 361.562ms | 361.704ms | 5.6% | 361.619ms | 381.792ms | 341.459ms | 401.705ms | 720.000ms | 368.48 MiB | none | 27.65M/s |
| q42 | date range and a deep offset | 242.000ms | 361.742ms | 361.649ms | 0.0% | 361.508ms | 361.685ms | 341.505ms | 381.660ms | 740.000ms | 372.14 MiB | none | 27.65M/s |
| q43 | minute buckets over a date range | 254.000ms | 381.534ms | 381.756ms | 5.3% | 362.004ms | 382.132ms | 361.471ms | 385.101ms | 740.000ms | 369.25 MiB | none | 26.19M/s |

clickhouse-local 26.9.1.1562 over 43 of 43 queries. Total 8.009s by its own clock and 16.570s by ours, 16.475s cold, 59.410s of CPU, peak 1.40 GiB, 53.69M/s and 10.04 GiB/s.

Running it cost 107% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.07x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 100.707ms | 80.494ms | 0.0% | 80.492ms | 80.502ms | 80.479ms | 80.503ms | 70.000ms | 160.88 MiB | 3.62 MiB | 124.23M/s |
| q2 | filtered count | 247.000ms | 332.826ms | 325.465ms | 0.3% | 325.178ms | 326.084ms | 324.269ms | 326.497ms | 4.760s | 2.36 GiB | none | 30.72M/s |
| q3 | three aggregates | 259.000ms | 345.646ms | 346.688ms | 0.4% | 346.170ms | 347.498ms | 344.523ms | 349.032ms | 5.040s | 2.76 GiB | none | 28.84M/s |
| q4 | average | 245.000ms | 324.889ms | 328.442ms | 1.3% | 324.442ms | 328.793ms | 322.795ms | 343.682ms | 4.720s | 2.34 GiB | none | 30.45M/s |
| q5 | count distinct, high card | 291.000ms | 367.526ms | 367.943ms | 1.1% | 366.552ms | 370.432ms | 366.102ms | 389.800ms | 5.620s | 2.59 GiB | none | 27.18M/s |
| q6 | count distinct, strings | 283.000ms | 387.656ms | 366.086ms | 0.5% | 365.245ms | 367.242ms | 352.172ms | 368.028ms | 5.430s | 2.74 GiB | none | 27.32M/s |
| q7 | min and max of a date | 1.000ms | 80.495ms | 80.544ms | 0.1% | 80.508ms | 80.558ms | 80.498ms | 80.907ms | 60.000ms | 160.36 MiB | none | 124.15M/s |
| q8 | group by, low card | 249.000ms | 329.825ms | 326.189ms | 1.1% | 325.246ms | 328.778ms | 324.066ms | 329.435ms | 4.890s | 2.34 GiB | none | 30.66M/s |
| q9 | group by and count distinct | 307.000ms | 388.565ms | 387.332ms | 0.2% | 386.725ms | 387.653ms | 386.360ms | 391.955ms | 5.750s | 2.70 GiB | none | 25.82M/s |
| q10 | group by, several aggregates | 326.000ms | 408.579ms | 406.889ms | 1.1% | 406.645ms | 411.279ms | 404.688ms | 428.990ms | 5.410s | 2.91 GiB | none | 24.58M/s |
| q11 | group by a string and count distinct | 545.000ms | 367.781ms | 748.861ms | 7.2% | 726.974ms | 780.730ms | 590.135ms | 784.372ms | 4.040s | 2.82 GiB | none | 13.35M/s |
| q12 | group by two strings and count distinct | 507.000ms | 782.910ms | 733.697ms | 10.3% | 706.684ms | 782.179ms | 682.894ms | 798.968ms | 3.810s | 2.80 GiB | none | 13.63M/s |
| q13 | group by a string and top k | 531.000ms | 764.498ms | 733.106ms | 51.5% | 388.048ms | 765.583ms | 383.223ms | 777.128ms | 4.240s | 2.96 GiB | none | 13.64M/s |
| q14 | group by a string and count distinct | 339.000ms | 448.552ms | 426.235ms | 0.3% | 425.763ms | 427.185ms | 413.347ms | 428.218ms | 6.380s | 3.16 GiB | none | 23.46M/s |
| q15 | group by two columns and top k | 289.000ms | 384.334ms | 373.180ms | 3.2% | 371.028ms | 382.944ms | 364.660ms | 383.422ms | 5.330s | 2.90 GiB | none | 26.80M/s |
| q16 | group by, very high card | 299.000ms | 386.876ms | 384.974ms | 3.4% | 373.408ms | 386.358ms | 364.624ms | 390.871ms | 5.460s | 2.70 GiB | none | 25.98M/s |
| q17 | group by two, very high card | 381.000ms | 487.399ms | 467.928ms | 0.4% | 467.639ms | 469.326ms | 465.843ms | 469.387ms | 7.290s | 3.71 GiB | none | 21.37M/s |
| q18 | group by two, no ordering | 377.000ms | 467.273ms | 469.609ms | 0.9% | 466.265ms | 470.369ms | 465.242ms | 472.192ms | 7.190s | 3.65 GiB | none | 21.29M/s |
| q19 | group by with an extract | 427.000ms | 531.569ms | 513.448ms | 0.6% | 510.246ms | 513.481ms | 509.894ms | 514.679ms | 8.650s | 4.17 GiB | none | 19.48M/s |
| q20 | point lookup | 247.000ms | 345.116ms | 326.204ms | 0.3% | 325.826ms | 326.663ms | 322.634ms | 327.546ms | 4.680s | 2.36 GiB | none | 30.65M/s |
| q21 | substring scan | 308.000ms | 390.925ms | 387.663ms | 1.8% | 387.493ms | 394.581ms | 387.232ms | 406.614ms | 5.830s | 3.28 GiB | none | 25.79M/s |
| q22 | substring scan and group by | 320.000ms | 403.840ms | 409.461ms | 4.9% | 390.890ms | 410.822ms | 389.120ms | 412.900ms | 6.060s | 2.93 GiB | none | 24.42M/s |
| q23 | two substring scans and group by | 408.000ms | 496.835ms | 487.205ms | 4.7% | 486.586ms | 509.611ms | 486.458ms | 526.338ms | 8.080s | 2.91 GiB | none | 20.52M/s |
| q24 | select star and top k | 619.000ms | 649.256ms | 718.537ms | 6.3% | 675.657ms | 720.576ms | 647.161ms | 752.746ms | 12.720s | 3.14 GiB | none | 13.92M/s |
| q25 | top k by a date | 244.000ms | 326.399ms | 328.027ms | 0.4% | 327.767ms | 329.018ms | 327.327ms | 332.140ms | 4.530s | 2.36 GiB | none | 30.48M/s |
| q26 | top k by a string | 267.000ms | 362.555ms | 348.535ms | 0.5% | 347.172ms | 348.960ms | 345.716ms | 348.974ms | 5.090s | 2.66 GiB | none | 28.69M/s |
| q27 | top k by two columns | 245.000ms | 330.304ms | 327.957ms | 0.2% | 327.473ms | 328.038ms | 326.763ms | 332.408ms | 4.620s | 2.39 GiB | none | 30.49M/s |
| q28 | group by with a string length | 320.000ms | 425.474ms | 408.634ms | 1.7% | 404.616ms | 411.417ms | 390.308ms | 434.043ms | 6.100s | 3.31 GiB | none | 24.47M/s |
| q29 | group by a regular expression | 429.000ms | 532.180ms | 514.272ms | 3.3% | 511.518ms | 528.506ms | 510.209ms | 529.066ms | 8.960s | 4.02 GiB | none | 19.44M/s |
| q30 | ninety sums over one column | 251.000ms | 349.992ms | 326.988ms | 2.1% | 324.563ms | 331.564ms | 322.749ms | 346.515ms | 4.540s | 2.32 GiB | none | 30.58M/s |
| q31 | group by two and several aggregates | 300.000ms | 370.876ms | 388.068ms | 0.3% | 387.668ms | 388.738ms | 369.194ms | 390.093ms | 5.430s | 2.85 GiB | none | 25.77M/s |
| q32 | group by a high card pair | 309.000ms | 392.506ms | 390.506ms | 3.2% | 390.120ms | 402.713ms | 389.538ms | 406.179ms | 5.390s | 2.86 GiB | none | 25.61M/s |
| q33 | group by a high card pair, unfiltered | 411.000ms | 492.273ms | 505.874ms | 3.0% | 492.853ms | 508.085ms | 491.078ms | 510.666ms | 7.970s | 3.75 GiB | none | 19.77M/s |
| q34 | group by a long string | 450.000ms | 550.496ms | 535.740ms | 3.2% | 533.633ms | 550.676ms | 532.953ms | 570.462ms | 9.140s | 4.49 GiB | none | 18.67M/s |
| q35 | group by a constant and a long string | 456.000ms | 547.229ms | 547.597ms | 1.1% | 547.417ms | 553.362ms | 537.640ms | 570.107ms | 9.140s | 4.41 GiB | none | 18.26M/s |
| q36 | group by four expressions | 293.000ms | 367.541ms | 383.067ms | 3.2% | 374.607ms | 386.818ms | 372.169ms | 389.163ms | 5.590s | 2.63 GiB | none | 26.10M/s |
| q37 | date range and group by a URL | 253.000ms | 327.967ms | 349.300ms | 0.9% | 347.731ms | 350.880ms | 329.725ms | 366.946ms | 4.470s | 2.35 GiB | none | 28.63M/s |
| q38 | date range and group by a title | 248.000ms | 347.164ms | 328.705ms | 0.7% | 327.102ms | 329.391ms | 325.436ms | 346.190ms | 4.440s | 2.40 GiB | none | 30.42M/s |
| q39 | date range, group by and offset | 243.000ms | 327.433ms | 332.451ms | 2.2% | 327.792ms | 335.041ms | 307.358ms | 344.608ms | 4.470s | 2.43 GiB | none | 30.08M/s |
| q40 | date range, a case and a wide group by | 248.000ms | 346.755ms | 331.893ms | 4.6% | 330.289ms | 345.656ms | 327.166ms | 347.054ms | 4.730s | 2.41 GiB | none | 30.13M/s |
| q41 | date range with an IN and a hash | 248.000ms | 326.808ms | 327.430ms | 2.2% | 326.775ms | 334.057ms | 323.371ms | 349.972ms | 4.440s | 2.35 GiB | none | 30.54M/s |
| q42 | date range and a deep offset | 250.000ms | 327.452ms | 347.657ms | 23.6% | 327.797ms | 409.832ms | 324.620ms | 422.651ms | 4.280s | 2.29 GiB | none | 28.76M/s |
| q43 | minute buckets over a date range | 247.000ms | 331.253ms | 326.268ms | 17.2% | 326.082ms | 382.318ms | 325.432ms | 409.707ms | 4.410s | 2.27 GiB | none | 30.65M/s |

datafusion datafusion-cli 55.1.0 over 43 of 43 queries. Total 13.518s by its own clock and 17.545s by ours, 17.357s cold, 239.250s of CPU, peak 4.49 GiB, 31.81M/s and 5.95 GiB/s.

Running it cost 30% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 9.30x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 39.361ms | 228.042ms | 141.032ms | 16.0% | 140.995ms | 163.509ms | 140.963ms | 205.144ms | 180.000ms | 127.45 MiB | none | 70.90M/s |
| q2 | filtered count | 41.634ms | 140.853ms | 140.985ms | 0.6% | 140.975ms | 141.854ms | 140.791ms | 161.206ms | 170.000ms | 1.12 GiB | none | 70.93M/s |
| q3 | three aggregates | 49.559ms | 160.899ms | 141.432ms | 0.1% | 141.337ms | 141.445ms | 141.112ms | 161.036ms | 200.000ms | 1.60 GiB | none | 70.70M/s |
| q4 | average | 51.650ms | 141.473ms | 141.067ms | 14.3% | 140.921ms | 161.115ms | 140.772ms | 161.459ms | 200.000ms | 1.57 GiB | none | 70.89M/s |
| q5 | count distinct, high card | 97.269ms | 202.071ms | 203.488ms | 0.9% | 201.743ms | 203.661ms | 201.646ms | 204.634ms | 820.000ms | 1.69 GiB | none | 49.14M/s |
| q6 | count distinct, strings | 93.686ms | 201.154ms | 201.347ms | 0.2% | 201.193ms | 201.695ms | 201.033ms | 201.771ms | 770.000ms | 1.73 GiB | none | 49.66M/s |
| q7 | min and max of a date | 47.719ms | 141.510ms | 140.805ms | 0.1% | 140.798ms | 140.871ms | 140.784ms | 140.895ms | 200.000ms | 1.54 GiB | none | 71.02M/s |
| q8 | group by, low card | 50.886ms | 160.907ms | 140.901ms | 0.2% | 140.881ms | 141.209ms | 140.730ms | 160.880ms | 210.000ms | 1.12 GiB | none | 70.97M/s |
| q9 | group by and count distinct | 163.073ms | 281.992ms | 284.034ms | 0.9% | 282.617ms | 285.049ms | 282.327ms | 292.729ms | 2.380s | 1.91 GiB | none | 35.21M/s |
| q10 | group by, several aggregates | 171.601ms | 304.249ms | 284.092ms | 0.1% | 283.811ms | 284.170ms | 283.724ms | 304.121ms | 2.450s | 2.08 GiB | none | 35.20M/s |
| q11 | group by a string and count distinct | 79.728ms | 182.052ms | 181.785ms | 0.3% | 181.345ms | 181.827ms | 181.319ms | 181.909ms | 380.000ms | 1.75 GiB | none | 55.01M/s |
| q12 | group by two strings and count distinct | 84.248ms | 180.977ms | 181.131ms | 0.2% | 180.993ms | 181.352ms | 180.942ms | 181.549ms | 390.000ms | 1.76 GiB | none | 55.21M/s |
| q13 | group by a string and top k | 103.936ms | 222.716ms | 221.537ms | 0.3% | 221.416ms | 221.996ms | 221.415ms | 222.221ms | 720.000ms | 1.73 GiB | none | 45.14M/s |
| q14 | group by a string and count distinct | 140.952ms | 262.087ms | 262.537ms | 0.4% | 261.619ms | 262.572ms | 261.577ms | 262.671ms | 1.620s | 2.01 GiB | none | 38.09M/s |
| q15 | group by two columns and top k | 104.060ms | 221.519ms | 221.791ms | 0.4% | 221.246ms | 222.088ms | 221.127ms | 222.094ms | 800.000ms | 1.73 GiB | none | 45.09M/s |
| q16 | group by, very high card | 122.609ms | 243.397ms | 242.024ms | 0.5% | 241.743ms | 243.022ms | 241.460ms | 244.660ms | 1.100s | 1.76 GiB | none | 41.32M/s |
| q17 | group by two, very high card | 192.217ms | 325.385ms | 324.062ms | 0.4% | 323.979ms | 325.386ms | 323.970ms | 325.560ms | 2.920s | 2.18 GiB | none | 30.86M/s |
| q18 | group by two, no ordering | 181.458ms | 303.902ms | 306.734ms | 5.2% | 306.223ms | 322.128ms | 304.072ms | 325.367ms | 2.880s | 2.18 GiB | none | 32.60M/s |
| q19 | group by with an extract | 240.782ms | 383.475ms | 388.499ms | 4.7% | 386.091ms | 404.195ms | 382.626ms | 411.234ms | 4.150s | 2.33 GiB | none | 25.74M/s |
| q20 | point lookup | 44.708ms | 160.946ms | 141.044ms | 0.0% | 141.034ms | 141.053ms | 140.956ms | 141.217ms | 160.000ms | 1.06 GiB | none | 70.90M/s |
| q21 | substring scan | 175.396ms | 281.901ms | 281.937ms | 0.2% | 281.932ms | 282.481ms | 281.620ms | 284.141ms | 1.880s | 1.93 GiB | none | 35.47M/s |
| q22 | substring scan and group by | 187.117ms | 302.910ms | 302.483ms | 0.2% | 302.092ms | 302.567ms | 283.062ms | 305.172ms | 2.110s | 2.03 GiB | none | 33.06M/s |
| q23 | two substring scans and group by | 278.800ms | 389.656ms | 388.173ms | 0.4% | 387.135ms | 388.779ms | 387.016ms | 435.164ms | 4.320s | 2.45 GiB | none | 25.76M/s |
| q24 | select star and top k | 503.748ms | 625.591ms | 623.579ms | 0.1% | 623.412ms | 624.157ms | 543.458ms | 933.386ms | 6.010s | 2.62 GiB | none | 16.04M/s |
| q25 | top k by a date | 80.294ms | 181.365ms | 181.391ms | 1.4% | 181.157ms | 183.635ms | 180.988ms | 201.720ms | 410.000ms | 1.79 GiB | none | 55.13M/s |
| q26 | top k by a string | 76.277ms | 183.155ms | 181.335ms | 0.1% | 181.223ms | 181.494ms | 181.096ms | 181.623ms | 340.000ms | 1.66 GiB | none | 55.15M/s |
| q27 | top k by two columns | 88.094ms | 181.305ms | 181.620ms | 0.1% | 181.581ms | 181.718ms | 181.172ms | 201.329ms | 460.000ms | 1.84 GiB | none | 55.06M/s |
| q30 | ninety sums over one column | 77.483ms | 181.619ms | 181.273ms | 0.1% | 181.190ms | 181.400ms | 181.018ms | 181.716ms | 520.000ms | 1.54 GiB | none | 55.16M/s |
| q31 | group by two and several aggregates | 109.037ms | 221.601ms | 221.198ms | 6.5% | 207.132ms | 221.521ms | 201.664ms | 221.805ms | 750.000ms | 1.84 GiB | none | 45.21M/s |
| q32 | group by a high card pair | 100.158ms | 201.423ms | 202.039ms | 9.9% | 201.684ms | 221.778ms | 201.522ms | 222.611ms | 770.000ms | 1.88 GiB | none | 49.49M/s |
| q33 | group by a high card pair, unfiltered | 234.686ms | 376.020ms | 383.903ms | 5.7% | 366.227ms | 387.953ms | 365.080ms | 388.196ms | 4.160s | 2.28 GiB | none | 26.05M/s |
| q34 | group by a long string | 274.914ms | 451.329ms | 427.652ms | 4.0% | 426.675ms | 443.611ms | 424.100ms | 445.645ms | 4.580s | 2.95 GiB | none | 23.38M/s |
| q35 | group by a constant and a long string | 311.725ms | 468.359ms | 464.409ms | 1.3% | 464.139ms | 470.103ms | 463.192ms | 471.268ms | 5.730s | 2.95 GiB | none | 21.53M/s |
| q37 | date range and group by a URL | 53.680ms | 160.803ms | 161.101ms | 0.5% | 160.977ms | 161.807ms | 160.916ms | 163.568ms | 210.000ms | 172.41 MiB | none | 62.07M/s |
| q38 | date range and group by a title | 49.735ms | 141.201ms | 160.875ms | 12.5% | 141.016ms | 161.150ms | 140.725ms | 166.329ms | 180.000ms | 145.33 MiB | none | 62.16M/s |
| q39 | date range, group by and offset | 46.678ms | 140.806ms | 140.915ms | 0.2% | 140.735ms | 140.958ms | 140.698ms | 141.037ms | 170.000ms | 141.67 MiB | none | 70.96M/s |
| q40 | date range, a case and a wide group by | 61.302ms | 161.052ms | 161.140ms | 0.3% | 160.914ms | 161.322ms | 160.892ms | 161.856ms | 260.000ms | 229.97 MiB | none | 62.06M/s |
| q41 | date range with an IN and a hash | 45.651ms | 141.082ms | 140.856ms | 0.2% | 140.785ms | 141.023ms | 140.777ms | 163.760ms | 180.000ms | 136.78 MiB | none | 70.99M/s |
| q42 | date range and a deep offset | 46.122ms | 140.793ms | 141.042ms | 0.0% | 140.993ms | 141.056ms | 140.736ms | 141.384ms | 190.000ms | 133.86 MiB | none | 70.90M/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 4.902s by its own clock and 9.217s by ours, 9.382s cold, 55.930s of CPU, peak 2.95 GiB, 79.56M/s and 14.88 GiB/s.

Running it cost 88% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 4.43x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 63.575ms | 120.687ms | 121.066ms | 0.4% | 120.733ms | 121.213ms | 120.714ms | 121.281ms | 100.000ms | 109.19 MiB | none | 82.60M/s |
| q2 | filtered count | 69.781ms | 124.747ms | 123.765ms | 3.1% | 122.060ms | 125.859ms | 121.246ms | 125.860ms | 140.000ms | 117.49 MiB | none | 80.80M/s |
| q3 | three aggregates | 73.405ms | 126.805ms | 127.098ms | 8.7% | 121.027ms | 132.080ms | 120.885ms | 141.047ms | 170.000ms | 116.78 MiB | none | 78.68M/s |
| q4 | average | 77.170ms | 121.045ms | 140.817ms | 5.4% | 133.520ms | 141.075ms | 121.115ms | 141.278ms | 220.000ms | 125.83 MiB | none | 71.01M/s |
| q5 | count distinct, high card | 89.045ms | 143.733ms | 143.386ms | 16.5% | 141.334ms | 164.988ms | 141.249ms | 170.798ms | 370.000ms | 306.95 MiB | none | 69.74M/s |
| q6 | count distinct, strings | 163.802ms | 241.818ms | 222.115ms | 0.6% | 221.359ms | 222.696ms | 221.180ms | 241.204ms | 810.000ms | 346.70 MiB | none | 45.02M/s |
| q7 | min and max of a date | 72.526ms | 140.755ms | 120.988ms | 7.5% | 120.981ms | 130.051ms | 120.876ms | 141.058ms | 170.000ms | 117.39 MiB | none | 82.65M/s |
| q8 | group by, low card | 71.019ms | 120.939ms | 121.081ms | 0.1% | 121.005ms | 121.142ms | 120.902ms | 141.342ms | 150.000ms | 119.65 MiB | none | 82.59M/s |
| q9 | group by and count distinct | 121.503ms | 166.915ms | 189.283ms | 5.8% | 185.446ms | 196.444ms | 182.574ms | 204.150ms | 890.000ms | 476.52 MiB | none | 52.83M/s |
| q10 | group by, several aggregates | 130.313ms | 201.574ms | 201.733ms | 1.5% | 201.635ms | 204.757ms | 182.484ms | 206.008ms | 990.000ms | 454.06 MiB | none | 49.57M/s |
| q11 | group by a string and count distinct | 100.480ms | 171.262ms | 161.351ms | 0.0% | 161.337ms | 161.363ms | 161.262ms | 172.910ms | 320.000ms | 162.85 MiB | none | 61.97M/s |
| q12 | group by two strings and count distinct | 95.799ms | 167.661ms | 161.351ms | 0.1% | 161.322ms | 161.531ms | 141.137ms | 163.799ms | 330.000ms | 165.20 MiB | none | 61.97M/s |
| q13 | group by a string and top k | 114.998ms | 181.716ms | 181.393ms | 0.1% | 181.248ms | 181.410ms | 161.176ms | 181.444ms | 710.000ms | 306.41 MiB | none | 55.13M/s |
| q14 | group by a string and count distinct | 135.862ms | 201.720ms | 202.122ms | 0.7% | 201.517ms | 202.867ms | 201.517ms | 203.339ms | 990.000ms | 385.18 MiB | none | 49.47M/s |
| q15 | group by two columns and top k | 125.319ms | 181.386ms | 181.412ms | 4.9% | 181.400ms | 190.243ms | 181.253ms | 201.236ms | 840.000ms | 309.89 MiB | none | 55.12M/s |
| q16 | group by, very high card | 156.778ms | 221.629ms | 221.575ms | 0.1% | 221.298ms | 221.588ms | 221.288ms | 241.388ms | 1.260s | 564.65 MiB | none | 45.13M/s |
| q17 | group by two, very high card | 194.992ms | 281.693ms | 264.246ms | 7.6% | 261.727ms | 281.763ms | 261.644ms | 282.163ms | 1.930s | 714.91 MiB | none | 37.84M/s |
| q18 | group by two, no ordering | 116.322ms | 200.024ms | 169.997ms | 7.9% | 165.497ms | 178.851ms | 161.432ms | 202.167ms | 820.000ms | 183.86 MiB | none | 58.82M/s |
| q19 | group by with an extract | 256.758ms | 342.394ms | 342.351ms | 0.0% | 342.311ms | 342.373ms | 341.901ms | 343.314ms | 2.840s | 1.27 GiB | none | 29.21M/s |
| q20 | point lookup | 73.732ms | 140.914ms | 120.927ms | 16.6% | 120.913ms | 141.035ms | 120.801ms | 141.061ms | 180.000ms | 124.66 MiB | none | 82.69M/s |
| q21 | substring scan | 157.446ms | 265.081ms | 215.800ms | 8.0% | 208.206ms | 225.514ms | 203.716ms | 235.823ms | 1.390s | 303.41 MiB | none | 46.34M/s |
| q22 | substring scan and group by | 170.548ms | 242.066ms | 241.818ms | 9.9% | 221.687ms | 245.669ms | 221.403ms | 247.276ms | 1.640s | 342.58 MiB | none | 41.35M/s |
| q23 | two substring scans and group by | 272.534ms | 342.275ms | 342.202ms | 4.9% | 325.497ms | 342.325ms | 324.089ms | 344.185ms | 3.320s | 360.85 MiB | none | 29.22M/s |
| q24 | select star and top k | 224.603ms | 267.759ms | 290.252ms | 2.0% | 286.239ms | 292.023ms | 283.629ms | 313.772ms | 1.970s | 406.88 MiB | none | 34.45M/s |
| q25 | top k by a date | 109.636ms | 185.213ms | 166.370ms | 8.9% | 166.338ms | 181.108ms | 162.708ms | 190.392ms | 660.000ms | 171.96 MiB | none | 60.11M/s |
| q26 | top k by a string | 97.489ms | 171.656ms | 161.359ms | 10.0% | 148.001ms | 164.117ms | 143.602ms | 190.485ms | 530.000ms | 167.35 MiB | none | 61.97M/s |
| q27 | top k by two columns | 103.717ms | 196.020ms | 165.163ms | 1.4% | 164.210ms | 166.603ms | 161.288ms | 189.256ms | 670.000ms | 171.47 MiB | none | 60.54M/s |
| q28 | group by with a string length | 178.526ms | 269.274ms | 244.252ms | 8.4% | 226.179ms | 246.769ms | 221.768ms | 270.634ms | 1.770s | 326.10 MiB | none | 40.94M/s |
| q29 | group by a regular expression | 254.323ms | 326.138ms | 322.100ms | 1.7% | 322.094ms | 327.447ms | 321.923ms | 345.959ms | 2.790s | 571.52 MiB | none | 31.05M/s |
| q30 | ninety sums over one column | 75.518ms | 141.120ms | 141.062ms | 14.3% | 120.891ms | 141.083ms | 120.795ms | 141.190ms | 150.000ms | 117.51 MiB | none | 70.89M/s |
| q31 | group by two and several aggregates | 122.402ms | 190.942ms | 182.851ms | 10.9% | 171.856ms | 191.815ms | 165.732ms | 207.275ms | 930.000ms | 222.40 MiB | none | 54.69M/s |
| q32 | group by a high card pair | 129.873ms | 203.982ms | 193.986ms | 3.0% | 189.583ms | 195.444ms | 186.427ms | 210.810ms | 930.000ms | 229.62 MiB | none | 51.55M/s |
| q33 | group by a high card pair, unfiltered | 156.970ms | 233.000ms | 219.004ms | 6.7% | 212.849ms | 227.497ms | 209.563ms | 233.254ms | 1.320s | 512.11 MiB | none | 45.66M/s |
| q34 | group by a long string | 310.474ms | 382.335ms | 402.620ms | 0.0% | 402.577ms | 402.666ms | 402.532ms | 402.871ms | 3.230s | 1.34 GiB | none | 24.84M/s |
| q35 | group by a constant and a long string | 317.693ms | 402.733ms | 403.060ms | 1.1% | 402.803ms | 407.271ms | 402.776ms | 407.471ms | 3.240s | 1.34 GiB | none | 24.81M/s |
| q36 | group by four expressions | 135.661ms | 201.367ms | 201.224ms | 0.1% | 201.108ms | 201.351ms | 201.070ms | 201.475ms | 960.000ms | 343.73 MiB | none | 49.69M/s |
| q37 | date range and group by a URL | 73.393ms | 140.767ms | 120.675ms | 16.8% | 120.660ms | 140.906ms | 120.656ms | 141.106ms | 160.000ms | 148.13 MiB | none | 82.86M/s |
| q38 | date range and group by a title | 67.077ms | 120.774ms | 120.845ms | 1.2% | 120.792ms | 122.271ms | 120.704ms | 123.747ms | 160.000ms | 125.82 MiB | none | 82.75M/s |
| q39 | date range, group by and offset | 68.775ms | 122.710ms | 120.681ms | 0.1% | 120.669ms | 120.829ms | 120.659ms | 120.935ms | 140.000ms | 127.30 MiB | none | 82.86M/s |
| q40 | date range, a case and a wide group by | 87.363ms | 140.863ms | 147.261ms | 3.4% | 144.075ms | 149.154ms | 141.371ms | 155.378ms | 180.000ms | 221.47 MiB | none | 67.91M/s |
| q41 | date range with an IN and a hash | 66.960ms | 120.901ms | 121.055ms | 1.2% | 120.817ms | 122.264ms | 120.712ms | 122.907ms | 120.000ms | 118.24 MiB | none | 82.60M/s |
| q42 | date range and a deep offset | 67.554ms | 120.834ms | 120.830ms | 1.0% | 120.746ms | 121.900ms | 120.742ms | 122.380ms | 120.000ms | 116.77 MiB | none | 82.76M/s |
| q43 | minute buckets over a date range | 67.057ms | 120.690ms | 120.905ms | 0.2% | 120.771ms | 120.953ms | 120.710ms | 121.787ms | 120.000ms | 117.01 MiB | none | 82.71M/s |

rudb rudb 0.3.67 over 43 of 43 queries. Total 5.619s by its own clock and 8.283s by ours, 8.508s cold, 40.730s of CPU, peak 1.34 GiB, 76.53M/s and 14.31 GiB/s.

Running it cost 47% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.34x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | 61.329ms | 1.602ms | 4.867ms | 4.878ms | 0.2% | 4.867ms | 28.420ms | 76.702ms | 2.50 KiB | 4 of 4 |
| q2 | 58.518ms | 7.299ms | 30.652ms | 30.660ms | 0.0% | 30.652ms | 26.440ms | 112.900ms | 896 B | 5 of 5 |
| q3 | 58.427ms | 11.657ms | 44.851ms | 44.859ms | 0.0% | 44.851ms | 26.188ms | 108.953ms | 1.25 KiB | 4 of 4 |
| q4 | 58.551ms | 13.164ms | 54.105ms | 54.114ms | 0.0% | 54.105ms | 26.926ms | 198.959ms | 896 B | 4 of 4 |
| q5 | 59.653ms | 25.680ms | 101.133ms | 101.145ms | 0.0% | 101.133ms | 26.197ms | 162.658ms | 164.31 MiB | 4 of 4 |
| q6 | 61.607ms | 102.839ms | 629.572ms | 629.582ms | 0.0% | 629.572ms | 27.577ms | 232.841ms | 119.29 MiB | 5 of 5 |
| q7 | 62.246ms | 8.097ms | 43.043ms | 43.051ms | 0.0% | 43.043ms | 27.497ms | 99.451ms | 1.25 KiB | 4 of 4 |
| q8 | 59.375ms | 6.401ms | 35.579ms | 35.587ms | 0.0% | 35.579ms | 26.930ms | 87.483ms | 9.54 KiB | 6 of 6 |
| q9 | 59.171ms | 46.557ms | 329.843ms | 329.853ms | 0.0% | 329.843ms | 26.685ms | 303.462ms | 380.17 MiB | 5 of 5 |
| q10 | 62.097ms | 57.258ms | 559.585ms | 559.597ms | 0.0% | 559.585ms | 33.489ms | 316.914ms | 373.04 MiB | 5 of 5 |
| q11 | 63.584ms | 43.938ms | 308.162ms | 308.172ms | 0.0% | 308.162ms | 34.144ms | 197.684ms | 7.78 MiB | 6 of 6 |
| q12 | 60.985ms | 39.313ms | 263.422ms | 263.431ms | 0.0% | 263.422ms | 32.903ms | 143.666ms | 7.64 MiB | 6 of 6 |
| q13 | 59.169ms | 48.439ms | 557.938ms | 557.947ms | 0.0% | 557.938ms | 26.857ms | 95.196ms | 112.51 MiB | 6 of 6 |
| q14 | 63.229ms | 68.981ms | 797.741ms | 797.751ms | 0.0% | 797.741ms | 34.224ms | 148.025ms | 196.30 MiB | 6 of 6 |
| q15 | 62.658ms | 56.238ms | 654.054ms | 654.063ms | 0.0% | 654.054ms | 33.797ms | 132.140ms | 118.62 MiB | 6 of 6 |
| q16 | 63.040ms | 86.114ms | 1.038s | 1.038s | 0.0% | 1.038s | 34.080ms | 158.308ms | 336.27 MiB | 5 of 5 |
| q17 | 62.164ms | 131.204ms | 1.684s | 1.684s | 0.0% | 1.684s | 27.759ms | 178.725ms | 454.58 MiB | 5 of 5 |
| q18 | 62.077ms | 70.880ms | 873.571ms | 873.582ms | 0.0% | 873.571ms | 34.155ms | 262.263ms | 15.30 KiB | 5 of 5 |
| q19 | 62.246ms | 186.569ms | 2.589s | 2.589s | 0.0% | 2.589s | 33.651ms | 237.701ms | 939.36 MiB | 5 of 5 |
| q20 | 63.251ms | 5.725ms | 29.030ms | 29.040ms | 0.0% | 29.030ms | 28.409ms | 112.551ms | 168 B | 4 of 4 |
| q21 | 59.687ms | 139.226ms | 1.817s | 1.817s | 0.0% | 1.817s | 26.918ms | 425.867ms | 896 B | 5 of 5 |
| q22 | 62.741ms | 107.894ms | 1.558s | 1.558s | 0.0% | 1.558s | 33.915ms | 98.183ms | 32.74 KiB | 6 of 6 |
| q23 | 63.918ms | 200.315ms | 3.064s | 3.064s | 0.0% | 3.064s | 27.695ms | 168.088ms | 351.21 KiB | 6 of 6 |
| q24 | 62.553ms | 136.680ms | 1.433s | 1.433s | 0.0% | 1.433s | 28.421ms | 188.532ms | 43.03 KiB | 8 of 8 |
| q25 | 62.971ms | 48.235ms | 566.040ms | 566.046ms | 0.0% | 566.040ms | 34.439ms | 139.515ms | 54.93 KiB | 6 of 6 |
| q26 | 60.485ms | 51.969ms | 500.209ms | 500.217ms | 0.0% | 500.209ms | 32.598ms | 347.185ms | 47.41 KiB | 5 of 5 |
| q27 | 61.599ms | 74.745ms | 880.876ms | 880.883ms | 0.0% | 880.876ms | 33.435ms | 265.682ms | 77.38 KiB | 6 of 6 |
| q28 | 59.672ms | 142.741ms | 1.908s | 1.908s | 0.0% | 1.908s | 31.816ms | 400.485ms | 967.31 KiB | 7 of 7 |
| q29 | 62.801ms | 189.553ms | 2.591s | 2.591s | 0.0% | 2.591s | 28.755ms | 230.409ms | 175.18 MiB | 7 of 7 |
| q30 | 62.935ms | 8.515ms | 30.268ms | 30.279ms | 0.0% | 30.268ms | 33.965ms | 125.756ms | 29.59 KiB | 4 of 4 |
| q31 | 60.209ms | 65.733ms | 652.307ms | 652.317ms | 0.0% | 652.307ms | 32.396ms | 355.287ms | 47.64 MiB | 6 of 6 |
| q32 | 60.322ms | 72.486ms | 844.034ms | 844.044ms | 0.0% | 844.034ms | 32.262ms | 213.694ms | 47.82 MiB | 6 of 6 |
| q33 | 62.670ms | 108.018ms | 859.725ms | 859.736ms | 0.0% | 859.725ms | 29.026ms | 861.238ms | 365.68 MiB | 5 of 5 |
| q34 | 60.639ms | 240.838ms | 3.027s | 3.027s | 0.0% | 3.027s | 27.367ms | 175.907ms | 921.35 MiB | 5 of 5 |
| q35 | 61.883ms | 250.911ms | 3.005s | 3.005s | 0.0% | 3.005s | 27.768ms | 167.305ms | 921.35 MiB | 5 of 5 |
| q36 | 62.835ms | 62.647ms | 792.944ms | 792.954ms | 0.0% | 792.944ms | 34.165ms | 142.881ms | 154.56 MiB | 6 of 6 |
| q37 | 62.194ms | 8.271ms | 29.332ms | 29.339ms | 0.0% | 29.332ms | 27.779ms | 102.882ms | 9.61 MiB | 6 of 6 |
| q38 | 60.052ms | 6.517ms | 15.448ms | 15.456ms | 0.0% | 15.448ms | 27.279ms | 127.265ms | 2.37 MiB | 6 of 6 |
| q39 | 57.043ms | 5.485ms | 16.158ms | 16.164ms | 0.0% | 16.158ms | 25.557ms | 78.279ms | 723.66 KiB | 6 of 6 |
| q40 | 59.362ms | 30.885ms | 79.595ms | 79.604ms | 0.0% | 79.595ms | 31.530ms | 208.867ms | 19.14 MiB | 6 of 6 |
| q41 | 61.039ms | 3.330ms | 8.876ms | 8.883ms | 0.1% | 8.876ms | 26.742ms | 84.375ms | 867.00 KiB | 6 of 6 |
| q42 | 61.032ms | 4.067ms | 6.736ms | 6.742ms | 0.1% | 6.736ms | 32.601ms | 90.657ms | 1.17 MiB | 6 of 6 |
| q43 | 58.852ms | 5.053ms | 9.156ms | 9.163ms | 0.1% | 9.156ms | 26.452ms | 114.384ms | 414.78 KiB | 6 of 6 |

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
| Aggregate | 0.000us | nothing to share | 39 | 197106902 | 909666 | 0.0ns | 0.0ns | 39 of 39 |
| Fetch | 0.000us | nothing to share | 1 | 10 | 10 | 0.0ns | 0.0ns | 1 of 1 |
| FileScan | 0.000us | nothing to share | 43 | 0 | 356378345 | handed none | 0.0ns | 43 of 43 |
| Filter | 0.000us | nothing to share | 28 | 186382627 | 30171383 | 0.0ns | 0.0ns | 28 of 28 |
| Limit | 0.000us | nothing to share | 1 | 10 | 10 | 0.0ns | 0.0ns | 1 of 1 |
| Project | 0.000us | nothing to share | 91 | 204137646 | 204137646 | 0.0ns | 0.0ns | 91 of 91 |
| Sort | 0.000us | nothing to share | 1 | 15 | 15 | 0.0ns | 0.0ns | 1 of 1 |
| TopN | 0.000us | nothing to share | 31 | 3969830 | 312 | 0.0ns | 0.0ns | 31 of 31 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is Aggregate, and the queries where it cost the most are q1 at 0.000us, q2 at 0.000us, q3 at 0.000us.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 9999750 rows, one out of every 10 of the 99997497 in the full file, which is a development loop rather than the suite
- q14 swung by 47.6% of its median, and rule two wants under 10%
- q21 swung by 16.5% of its median, and rule two wants under 10%
- q35 swung by 11.4% of its median, and rule two wants under 10%

These swung wider than reporting rule two allows:

- duckdb swung by 47.6% of its median on q14, and rule two wants under 10%
- duckdb-pinned swung by 50.0% of its median on q27, and rule two wants under 10%
- clickhouse-local swung by 32.8% of its median on q7, and rule two wants under 10%
- datafusion swung by 51.5% of its median on q13, and rule two wants under 10%
- polars swung by 16.0% of its median on q1, and rule two wants under 10%
- rudb swung by 16.8% of its median on q37, and rule two wants under 10%

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

