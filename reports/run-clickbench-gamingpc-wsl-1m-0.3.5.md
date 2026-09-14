# clickbench on gamingpc-wsl

This is one run of the clickbench suite on gamingpc-wsl, over 2 engines and 43 queries, with 5 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

It ran over 999975 rows, one out of every 100 of the 99997497 in the full file, which is a development loop and not the suite. Nothing here is comparable to a full run or to anybody else's number.

## What ran

| what | it was |
| --- | --- |
| suite | clickbench |
| queries | 43 |
| tables | 211.28 MiB of Parquet in 1 table |
| rows | 999975 in the table every query reads |
| sample | 999975 rows, one out of every 100 of the 99997497 in the full file |
| summary | median with the interquartile range, per reporting rule two |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --rows 1000000 --engines duckdb,rudb --runs 5 --report
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
| duckdb | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 1.994s | 8.790s | 501.01 MiB | its own database file | its own | 0.12 to 0.48 |
| rudb | rudb 0.3.5 | ran | 0.000us | 0.000us | 211.28 MiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 0.48 to 0.89 |
| duckdb-pinned | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| clickhouse-local | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| datafusion | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| polars | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb on 41 shared |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 556.000ms | 1.515s | +173% | 1.555s | 3.410s | 2.25 | 307.60 MiB | none | 77.34M/s | 15.96 GiB/s | 1.00x |
| rudb | 5.854s | 5.944s | +2% | 5.959s | 5.560s | 0.94 | 172.14 MiB | none | 7.00M/s | 1.45 GiB/s | 11.28x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | rudb |
| --- | --- | --- | --- |
| q1 | count | 1.000ms | 1.682ms |
| q2 | filtered count | 1.000ms | 4.382ms |
| q3 | three aggregates | 3.000ms | 9.754ms |
| q4 | average | 3.000ms | 11.924ms |
| q5 | count distinct, high card | 10.000ms | 50.711ms |
| q6 | count distinct, strings | 8.000ms | 72.772ms |
| q7 | min and max of a date | 1.000ms | 5.274ms |
| q8 | group by, low card | 6.000ms | 4.451ms |
| q9 | group by and count distinct | 13.000ms | 78.434ms |
| q10 | group by, several aggregates | 16.000ms | 92.805ms |
| q11 | group by a string and count distinct | 8.000ms | 20.623ms |
| q12 | group by two strings and count distinct | 9.000ms | 24.799ms |
| q13 | group by a string and top k | 9.000ms | 100.770ms |
| q14 | group by a string and count distinct | 13.000ms | 117.994ms |
| q15 | group by two columns and top k | 10.000ms | 112.386ms |
| q16 | group by, very high card | 10.000ms | 138.710ms |
| q17 | group by two, very high card | 17.000ms | 256.877ms |
| q18 | group by two, no ordering | 17.000ms | 44.981ms |
| q19 | group by with an extract | 19.000ms | no dialect |
| q20 | point lookup | 3.000ms | 11.370ms |
| q21 | substring scan | 14.000ms | 160.694ms |
| q22 | substring scan and group by | 20.000ms | 209.346ms |
| q23 | two substring scans and group by | 22.000ms | 459.936ms |
| q24 | select star and top k | 40.000ms | 1.106s |
| q25 | top k by a date | 6.000ms | 92.610ms |
| q26 | top k by a string | 5.000ms | 94.915ms |
| q27 | top k by two columns | 6.000ms | 102.135ms |
| q28 | group by with a string length | 17.000ms | 158.942ms |
| q29 | group by a regular expression | 90.000ms | 267.995ms |
| q30 | ninety sums over one column | 14.000ms | 8.633ms |
| q31 | group by two and several aggregates | 12.000ms | 107.371ms |
| q32 | group by a high card pair | 12.000ms | 107.628ms |
| q33 | group by a high card pair, unfiltered | 18.000ms | no dialect |
| q34 | group by a long string | 28.000ms | 376.798ms |
| q35 | group by a constant and a long string | 29.000ms | 378.060ms |
| q36 | group by four expressions | 11.000ms | 211.588ms |
| q37 | date range and group by a URL | 4.000ms | 141.628ms |
| q38 | date range and group by a title | 4.000ms | 214.762ms |
| q39 | date range, group by and offset | 4.000ms | 141.262ms |
| q40 | date range, a case and a wide group by | 7.000ms | 266.625ms |
| q41 | date range with an IN and a hash | 4.000ms | 34.012ms |
| q42 | date range and a deep offset | 8.000ms | 29.607ms |
| q43 | minute buckets over a date range | 4.000ms | 22.653ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 22.011ms | 22.096ms | 3.6% | 21.848ms | 22.654ms | 21.651ms | 23.158ms | 20.000ms | 39.96 MiB | none | 45.26M/s |
| q2 | filtered count | 1.000ms | 25.837ms | 23.620ms | 1.2% | 23.503ms | 23.796ms | 23.402ms | 24.387ms | 20.000ms | 42.72 MiB | none | 42.34M/s |
| q3 | three aggregates | 3.000ms | 23.526ms | 24.578ms | 2.4% | 24.015ms | 24.605ms | 23.737ms | 24.912ms | 20.000ms | 45.98 MiB | none | 40.69M/s |
| q4 | average | 3.000ms | 25.622ms | 24.700ms | 0.4% | 24.661ms | 24.771ms | 23.814ms | 27.489ms | 20.000ms | 49.71 MiB | none | 40.48M/s |
| q5 | count distinct, high card | 10.000ms | 34.439ms | 31.965ms | 4.3% | 31.484ms | 32.872ms | 30.965ms | 36.771ms | 70.000ms | 101.42 MiB | none | 31.28M/s |
| q6 | count distinct, strings | 8.000ms | 30.149ms | 29.898ms | 2.5% | 29.225ms | 29.971ms | 28.213ms | 30.208ms | 40.000ms | 80.97 MiB | none | 33.45M/s |
| q7 | min and max of a date | 1.000ms | 22.676ms | 22.236ms | 0.8% | 22.200ms | 22.384ms | 21.666ms | 22.474ms | 20.000ms | 40.62 MiB | none | 44.97M/s |
| q8 | group by, low card | 6.000ms | 28.343ms | 28.718ms | 6.2% | 27.359ms | 29.131ms | 26.881ms | 29.331ms | 20.000ms | 45.98 MiB | none | 34.82M/s |
| q9 | group by and count distinct | 13.000ms | 38.487ms | 35.295ms | 2.2% | 34.771ms | 35.563ms | 34.242ms | 42.443ms | 80.000ms | 118.98 MiB | none | 28.33M/s |
| q10 | group by, several aggregates | 16.000ms | 37.358ms | 38.568ms | 7.6% | 37.900ms | 40.814ms | 37.688ms | 42.524ms | 100.000ms | 134.40 MiB | none | 25.93M/s |
| q11 | group by a string and count distinct | 8.000ms | 28.500ms | 30.305ms | 1.1% | 30.214ms | 30.562ms | 29.614ms | 32.095ms | 40.000ms | 72.99 MiB | none | 33.00M/s |
| q12 | group by two strings and count distinct | 9.000ms | 33.397ms | 32.202ms | 5.3% | 31.215ms | 32.926ms | 30.486ms | 32.975ms | 40.000ms | 75.85 MiB | none | 31.05M/s |
| q13 | group by a string and top k | 9.000ms | 30.779ms | 31.417ms | 6.9% | 30.972ms | 33.148ms | 30.625ms | 34.666ms | 60.000ms | 87.23 MiB | none | 31.83M/s |
| q14 | group by a string and count distinct | 13.000ms | 34.862ms | 34.870ms | 0.7% | 34.729ms | 34.986ms | 34.720ms | 35.007ms | 90.000ms | 137.24 MiB | none | 28.68M/s |
| q15 | group by two columns and top k | 10.000ms | 31.934ms | 31.809ms | 12.5% | 30.905ms | 34.868ms | 30.779ms | 37.797ms | 60.000ms | 92.71 MiB | none | 31.44M/s |
| q16 | group by, very high card | 10.000ms | 39.186ms | 32.546ms | 3.6% | 32.144ms | 33.317ms | 32.140ms | 41.431ms | 80.000ms | 112.27 MiB | none | 30.72M/s |
| q17 | group by two, very high card | 17.000ms | 38.947ms | 40.084ms | 2.5% | 39.198ms | 40.184ms | 38.764ms | 41.415ms | 140.000ms | 196.52 MiB | none | 24.95M/s |
| q18 | group by two, no ordering | 17.000ms | 38.609ms | 39.991ms | 2.3% | 39.883ms | 40.796ms | 38.762ms | 42.592ms | 130.000ms | 195.76 MiB | none | 25.01M/s |
| q19 | group by with an extract | 19.000ms | 54.255ms | 42.935ms | 1.8% | 42.568ms | 43.344ms | 41.160ms | 51.787ms | 160.000ms | 225.43 MiB | none | 23.29M/s |
| q20 | point lookup | 3.000ms | 23.665ms | 24.309ms | 4.0% | 23.671ms | 24.645ms | 23.483ms | 25.048ms | 20.000ms | 48.70 MiB | none | 41.14M/s |
| q21 | substring scan | 14.000ms | 40.514ms | 36.562ms | 4.7% | 36.225ms | 37.927ms | 35.820ms | 39.100ms | 100.000ms | 102.21 MiB | none | 27.35M/s |
| q22 | substring scan and group by | 20.000ms | 37.573ms | 42.220ms | 16.1% | 42.020ms | 48.829ms | 39.701ms | 51.904ms | 100.000ms | 121.97 MiB | none | 23.68M/s |
| q23 | two substring scans and group by | 22.000ms | 46.395ms | 44.357ms | 1.3% | 44.310ms | 44.907ms | 43.753ms | 46.470ms | 130.000ms | 155.48 MiB | none | 22.54M/s |
| q24 | select star and top k | 40.000ms | 59.801ms | 63.993ms | 4.3% | 62.589ms | 65.325ms | 61.825ms | 67.380ms | 180.000ms | 219.43 MiB | none | 15.63M/s |
| q25 | top k by a date | 6.000ms | 26.224ms | 26.882ms | 11.1% | 26.288ms | 29.264ms | 25.960ms | 32.702ms | 30.000ms | 63.12 MiB | none | 37.20M/s |
| q26 | top k by a string | 5.000ms | 27.534ms | 26.640ms | 3.6% | 25.868ms | 26.824ms | 25.559ms | 28.550ms | 30.000ms | 56.98 MiB | none | 37.54M/s |
| q27 | top k by two columns | 6.000ms | 28.199ms | 27.694ms | 1.5% | 27.558ms | 27.965ms | 26.874ms | 28.754ms | 30.000ms | 63.22 MiB | none | 36.11M/s |
| q28 | group by with a string length | 17.000ms | 40.171ms | 39.325ms | 6.1% | 38.112ms | 40.493ms | 37.682ms | 56.576ms | 90.000ms | 116.73 MiB | none | 25.43M/s |
| q29 | group by a regular expression | 90.000ms | 112.000ms | 114.345ms | 2.8% | 111.728ms | 114.938ms | 111.405ms | 116.223ms | 510.000ms | 178.94 MiB | none | 8.75M/s |
| q30 | ninety sums over one column | 14.000ms | 36.625ms | 36.372ms | 1.1% | 36.333ms | 36.724ms | 36.162ms | 36.857ms | 30.000ms | 56.23 MiB | none | 27.49M/s |
| q31 | group by two and several aggregates | 12.000ms | 36.391ms | 34.444ms | 5.0% | 33.455ms | 35.173ms | 33.420ms | 44.572ms | 60.000ms | 93.27 MiB | none | 29.03M/s |
| q32 | group by a high card pair | 12.000ms | 35.983ms | 34.020ms | 1.1% | 33.661ms | 34.043ms | 32.366ms | 34.555ms | 70.000ms | 102.48 MiB | none | 29.39M/s |
| q33 | group by a high card pair, unfiltered | 18.000ms | 47.952ms | 41.968ms | 4.1% | 41.457ms | 43.195ms | 41.359ms | 44.158ms | 170.000ms | 216.49 MiB | none | 23.83M/s |
| q34 | group by a long string | 28.000ms | 52.420ms | 51.426ms | 1.4% | 51.407ms | 52.134ms | 50.893ms | 52.510ms | 200.000ms | 301.85 MiB | none | 19.45M/s |
| q35 | group by a constant and a long string | 29.000ms | 66.605ms | 53.183ms | 1.3% | 53.095ms | 53.802ms | 52.175ms | 56.813ms | 200.000ms | 307.60 MiB | none | 18.80M/s |
| q36 | group by four expressions | 11.000ms | 31.904ms | 32.656ms | 0.9% | 32.414ms | 32.705ms | 31.669ms | 32.924ms | 80.000ms | 107.72 MiB | none | 30.62M/s |
| q37 | date range and group by a URL | 4.000ms | 27.631ms | 25.810ms | 1.1% | 25.752ms | 26.035ms | 25.602ms | 26.499ms | 20.000ms | 50.22 MiB | none | 38.74M/s |
| q38 | date range and group by a title | 4.000ms | 25.061ms | 25.532ms | 1.4% | 25.347ms | 25.706ms | 25.252ms | 25.831ms | 20.000ms | 49.47 MiB | none | 39.17M/s |
| q39 | date range, group by and offset | 4.000ms | 25.390ms | 25.707ms | 1.1% | 25.707ms | 25.983ms | 25.390ms | 27.120ms | 30.000ms | 48.72 MiB | none | 38.90M/s |
| q40 | date range, a case and a wide group by | 7.000ms | 28.376ms | 28.802ms | 2.1% | 28.544ms | 29.152ms | 28.185ms | 29.235ms | 30.000ms | 57.37 MiB | none | 34.72M/s |
| q41 | date range with an IN and a hash | 4.000ms | 25.332ms | 25.538ms | 3.6% | 25.068ms | 25.999ms | 24.783ms | 26.234ms | 20.000ms | 50.51 MiB | none | 39.16M/s |
| q42 | date range and a deep offset | 8.000ms | 29.293ms | 30.176ms | 6.0% | 30.035ms | 31.849ms | 29.450ms | 32.097ms | 30.000ms | 49.23 MiB | none | 33.14M/s |
| q43 | minute buckets over a date range | 4.000ms | 25.459ms | 25.665ms | 4.4% | 24.864ms | 26.006ms | 24.474ms | 26.052ms | 20.000ms | 48.21 MiB | none | 38.96M/s |

duckdb v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 556.000ms by its own clock and 1.515s by ours, 1.555s cold, 3.410s of CPU, peak 307.60 MiB, 77.34M/s and 15.96 GiB/s.

Running it cost 173% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 5.17x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.682ms | 3.463ms | 3.134ms | 2.0% | 3.134ms | 3.196ms | 3.107ms | 3.256ms | 0.000us | 5.28 MiB | none | 319.09M/s |
| q2 | filtered count | 4.382ms | 5.807ms | 5.821ms | 0.7% | 5.803ms | 5.846ms | 5.782ms | 5.849ms | 0.000us | 6.17 MiB | none | 171.78M/s |
| q3 | three aggregates | 9.754ms | 11.393ms | 11.293ms | 1.6% | 11.190ms | 11.367ms | 11.122ms | 11.472ms | 10.000ms | 6.60 MiB | none | 88.55M/s |
| q4 | average | 11.924ms | 13.782ms | 13.411ms | 4.2% | 12.910ms | 13.472ms | 12.826ms | 13.501ms | 0.000us | 9.18 MiB | none | 74.56M/s |
| q5 | count distinct, high card | 50.711ms | 52.095ms | 52.420ms | 0.9% | 52.390ms | 52.880ms | 51.324ms | 53.393ms | 40.000ms | 22.46 MiB | none | 19.08M/s |
| q6 | count distinct, strings | 72.772ms | 71.089ms | 75.162ms | 2.7% | 73.372ms | 75.378ms | 71.490ms | 76.108ms | 60.000ms | 26.74 MiB | none | 13.30M/s |
| q7 | min and max of a date | 5.274ms | 7.297ms | 6.754ms | 1.3% | 6.680ms | 6.770ms | 6.653ms | 6.931ms | 0.000us | 6.02 MiB | none | 148.07M/s |
| q8 | group by, low card | 4.451ms | 5.950ms | 5.945ms | 3.5% | 5.903ms | 6.114ms | 5.823ms | 6.221ms | 0.000us | 6.17 MiB | none | 168.19M/s |
| q9 | group by and count distinct | 78.434ms | 80.785ms | 80.157ms | 0.8% | 79.646ms | 80.304ms | 79.185ms | 80.410ms | 70.000ms | 21.25 MiB | none | 12.48M/s |
| q10 | group by, several aggregates | 92.805ms | 93.816ms | 94.705ms | 1.9% | 93.116ms | 94.932ms | 91.947ms | 95.353ms | 80.000ms | 23.94 MiB | none | 10.56M/s |
| q11 | group by a string and count distinct | 20.623ms | 22.444ms | 22.126ms | 1.3% | 22.035ms | 22.319ms | 21.677ms | 22.737ms | 10.000ms | 10.78 MiB | none | 45.19M/s |
| q12 | group by two strings and count distinct | 24.799ms | 26.638ms | 26.627ms | 2.2% | 26.203ms | 26.778ms | 26.031ms | 26.814ms | 20.000ms | 11.61 MiB | none | 37.56M/s |
| q13 | group by a string and top k | 100.770ms | 102.310ms | 102.986ms | 0.9% | 102.255ms | 103.150ms | 101.724ms | 107.168ms | 90.000ms | 25.84 MiB | none | 9.71M/s |
| q14 | group by a string and count distinct | 117.994ms | 121.550ms | 120.224ms | 0.6% | 120.106ms | 120.861ms | 118.909ms | 122.465ms | 110.000ms | 36.15 MiB | none | 8.32M/s |
| q15 | group by two columns and top k | 112.386ms | 113.377ms | 114.612ms | 1.1% | 114.197ms | 115.458ms | 113.852ms | 115.597ms | 110.000ms | 29.81 MiB | none | 8.72M/s |
| q16 | group by, very high card | 138.710ms | 140.020ms | 140.766ms | 1.6% | 140.640ms | 142.832ms | 139.976ms | 145.218ms | 130.000ms | 50.23 MiB | none | 7.10M/s |
| q17 | group by two, very high card | 256.877ms | 253.061ms | 259.571ms | 2.6% | 256.381ms | 263.081ms | 254.559ms | 263.183ms | 250.000ms | 86.59 MiB | none | 3.85M/s |
| q18 | group by two, no ordering | 44.981ms | 47.128ms | 46.543ms | 5.4% | 45.550ms | 48.059ms | 45.097ms | 48.650ms | 40.000ms | 11.34 MiB | none | 21.49M/s |
| q20 | point lookup | 11.370ms | 12.992ms | 12.834ms | 0.9% | 12.749ms | 12.862ms | 12.684ms | 13.275ms | 0.000us | 9.48 MiB | none | 77.92M/s |
| q21 | substring scan | 160.694ms | 163.066ms | 163.442ms | 0.8% | 162.543ms | 163.912ms | 162.428ms | 165.335ms | 150.000ms | 38.52 MiB | none | 6.12M/s |
| q22 | substring scan and group by | 209.346ms | 214.273ms | 211.752ms | 0.8% | 211.512ms | 213.275ms | 210.650ms | 217.873ms | 200.000ms | 40.50 MiB | none | 4.72M/s |
| q23 | two substring scans and group by | 459.936ms | 462.883ms | 462.177ms | 1.7% | 459.051ms | 466.972ms | 454.346ms | 468.008ms | 450.000ms | 53.00 MiB | none | 2.16M/s |
| q24 | select star and top k | 1.106s | 1.115s | 1.112s | 0.8% | 1.111s | 1.119s | 1.097s | 1.131s | 1.100s | 153.40 MiB | none | 899.45K/s |
| q25 | top k by a date | 92.610ms | 96.691ms | 94.330ms | 1.6% | 93.364ms | 94.907ms | 92.665ms | 99.406ms | 90.000ms | 13.67 MiB | none | 10.60M/s |
| q26 | top k by a string | 94.915ms | 95.168ms | 96.553ms | 0.4% | 96.172ms | 96.563ms | 95.528ms | 97.658ms | 90.000ms | 11.97 MiB | none | 10.36M/s |
| q27 | top k by two columns | 102.135ms | 109.171ms | 103.806ms | 2.0% | 103.393ms | 105.431ms | 102.688ms | 105.491ms | 100.000ms | 13.36 MiB | none | 9.63M/s |
| q28 | group by with a string length | 158.942ms | 161.470ms | 161.396ms | 1.5% | 160.209ms | 162.701ms | 159.355ms | 163.362ms | 150.000ms | 40.97 MiB | none | 6.20M/s |
| q29 | group by a regular expression | 267.995ms | 276.943ms | 271.125ms | 0.4% | 270.400ms | 271.545ms | 270.035ms | 273.048ms | 260.000ms | 76.76 MiB | none | 3.69M/s |
| q30 | ninety sums over one column | 8.633ms | 10.942ms | 10.139ms | 2.0% | 10.079ms | 10.281ms | 10.021ms | 10.303ms | 0.000us | 6.21 MiB | none | 98.63M/s |
| q31 | group by two and several aggregates | 107.371ms | 107.936ms | 109.362ms | 0.2% | 109.225ms | 109.495ms | 108.967ms | 110.378ms | 100.000ms | 30.74 MiB | none | 9.14M/s |
| q32 | group by a high card pair | 107.628ms | 111.207ms | 109.693ms | 0.1% | 109.666ms | 109.735ms | 109.509ms | 109.781ms | 100.000ms | 31.30 MiB | none | 9.12M/s |
| q34 | group by a long string | 376.798ms | 380.980ms | 381.924ms | 0.5% | 380.830ms | 382.799ms | 378.897ms | 382.936ms | 370.000ms | 171.86 MiB | none | 2.62M/s |
| q35 | group by a constant and a long string | 378.060ms | 383.526ms | 382.989ms | 1.4% | 380.957ms | 386.448ms | 380.800ms | 388.500ms | 370.000ms | 172.14 MiB | none | 2.61M/s |
| q36 | group by four expressions | 211.588ms | 216.050ms | 214.133ms | 3.1% | 209.622ms | 216.161ms | 207.183ms | 217.972ms | 210.000ms | 56.28 MiB | none | 4.67M/s |
| q37 | date range and group by a URL | 141.628ms | 143.371ms | 144.006ms | 3.1% | 143.890ms | 148.352ms | 143.356ms | 149.053ms | 140.000ms | 40.57 MiB | none | 6.94M/s |
| q38 | date range and group by a title | 214.762ms | 215.022ms | 217.049ms | 0.4% | 216.299ms | 217.165ms | 216.207ms | 218.511ms | 210.000ms | 29.35 MiB | none | 4.61M/s |
| q39 | date range, group by and offset | 141.262ms | 144.854ms | 143.712ms | 2.1% | 141.354ms | 144.353ms | 139.940ms | 145.394ms | 130.000ms | 40.32 MiB | none | 6.96M/s |
| q40 | date range, a case and a wide group by | 266.625ms | 272.090ms | 268.662ms | 0.2% | 268.476ms | 269.000ms | 262.924ms | 276.414ms | 260.000ms | 46.74 MiB | none | 3.72M/s |
| q41 | date range with an IN and a hash | 34.012ms | 37.254ms | 35.664ms | 1.4% | 35.413ms | 35.898ms | 35.285ms | 36.039ms | 30.000ms | 13.26 MiB | none | 28.04M/s |
| q42 | date range and a deep offset | 29.607ms | 31.463ms | 31.201ms | 1.8% | 31.102ms | 31.661ms | 30.809ms | 31.701ms | 20.000ms | 12.82 MiB | none | 32.05M/s |
| q43 | minute buckets over a date range | 22.653ms | 24.202ms | 24.115ms | 0.2% | 24.082ms | 24.135ms | 23.771ms | 24.220ms | 10.000ms | 9.94 MiB | none | 41.47M/s |

rudb rudb 0.3.5 over 41 of 43 queries. Total 5.854s by its own clock and 5.944s by ours, 5.959s cold, 5.560s of CPU, peak 172.14 MiB, 7.00M/s and 1.45 GiB/s.

Running it cost 2% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 354.76x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | execute | accounted | measured | apart | driver | build | outside | held | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | 967.493us | 967.473us | 967.473us | 0.0% | 451.103us | 272.084us | 0.000us | 360 B | 4 of 4 |
| q2 | 3.574ms | 3.574ms | 3.574ms | 0.0% | 461.289us | 254.475us | 0.000us | 360 B | 5 of 5 |
| q3 | 9.210ms | 9.210ms | 9.210ms | 0.0% | 502.939us | 256.278us | 0.000us | 992 B | 4 of 4 |
| q4 | 11.428ms | 11.428ms | 11.428ms | 0.0% | 497.477us | 258.637us | 0.000us | 360 B | 4 of 4 |
| q5 | 49.724ms | 49.724ms | 49.724ms | 0.0% | 543.826us | 244.988us | 0.000us | 13.72 MiB | 4 of 4 |
| q6 | 68.217ms | 68.217ms | 68.217ms | 0.0% | 515.816us | 245.268us | 0.000us | 16.83 MiB | 4 of 4 |
| q7 | 4.852ms | 4.849ms | 4.849ms | 0.0% | 467.569us | 258.333us | 0.000us | 616 B | 4 of 4 |
| q8 | 3.717ms | 3.717ms | 3.717ms | 0.0% | 451.726us | 248.710us | 0.000us | 4.33 KiB | 6 of 6 |
| q9 | 78.471ms | 78.471ms | 78.471ms | 0.0% | 574.528us | 238.824us | 0.000us | 14.13 MiB | 5 of 5 |
| q10 | 91.159ms | 91.159ms | 91.159ms | 0.0% | 623.803us | 246.536us | 0.000us | 14.97 MiB | 5 of 5 |
| q11 | 20.007ms | 20.006ms | 20.006ms | 0.0% | 693.728us | 258.441us | 0.000us | 808.20 KiB | 6 of 6 |
| q12 | 24.351ms | 24.351ms | 24.351ms | 0.0% | 720.586us | 247.740us | 0.000us | 828.25 KiB | 6 of 6 |
| q13 | 99.720ms | 99.720ms | 99.720ms | 0.0% | 1.274ms | 246.226us | 0.000us | 22.24 MiB | 6 of 6 |
| q14 | 118.308ms | 118.308ms | 118.308ms | 0.0% | 1.417ms | 565.481us | 0.000us | 25.24 MiB | 6 of 6 |
| q15 | 110.595ms | 110.595ms | 110.595ms | 0.0% | 1.379ms | 258.681us | 0.000us | 26.89 MiB | 6 of 6 |
| q16 | 136.987ms | 136.977ms | 136.977ms | 0.0% | 2.160ms | 249.790us | 0.000us | 46.15 MiB | 5 of 5 |
| q17 | 249.326ms | 249.326ms | 249.326ms | 0.0% | 3.590ms | 262.582us | 0.000us | 92.18 MiB | 5 of 5 |
| q18 | 44.591ms | 44.591ms | 44.591ms | 0.0% | 527.655us | 254.224us | 0.000us | 2.34 KiB | 5 of 5 |
| q20 | 10.728ms | 10.723ms | 10.723ms | 0.0% | 325.897us | 245.668us | 0.000us | 0 B | 4 of 4 |
| q21 | 160.188ms | 160.188ms | 160.188ms | 0.0% | 430.241us | 247.829us | 0.000us | 360 B | 5 of 5 |
| q22 | 210.797ms | 210.785ms | 210.785ms | 0.0% | 552.503us | 357.652us | 0.000us | 12.31 KiB | 6 of 6 |
| q23 | 459.947ms | 459.921ms | 459.921ms | 0.0% | 758.299us | 273.357us | 0.000us | 73.55 KiB | 6 of 6 |
| q24 | 1.109s | 1.106s | 1.106s | 0.0% | 5.599ms | 318.141us | 0.000us | 181.90 KiB | 5 of 5 |
| q25 | 93.982ms | 93.982ms | 93.982ms | 0.0% | 1.059ms | 253.080us | 0.000us | 218.26 KiB | 6 of 6 |
| q26 | 92.790ms | 92.790ms | 92.790ms | 0.0% | 1.014ms | 247.515us | 0.000us | 226.53 KiB | 5 of 5 |
| q27 | 106.287ms | 106.282ms | 106.282ms | 0.0% | 1.040ms | 253.267us | 0.000us | 306.53 KiB | 6 of 6 |
| q28 | 158.224ms | 158.224ms | 158.224ms | 0.0% | 780.150us | 260.048us | 0.000us | 821.53 KiB | 7 of 7 |
| q29 | 272.605ms | 272.588ms | 272.588ms | 0.0% | 2.197ms | 274.176us | 0.000us | 64.51 MiB | 7 of 7 |
| q30 | 6.829ms | 6.829ms | 6.829ms | 0.0% | 468.690us | 269.186us | 2.902ms | 29.59 KiB | 4 of 4 |
| q31 | 105.376ms | 105.373ms | 105.373ms | 0.0% | 1.270ms | 258.395us | 0.000us | 27.12 MiB | 6 of 6 |
| q32 | 108.103ms | 108.103ms | 108.103ms | 0.0% | 1.264ms | 332.906us | 0.000us | 28.72 MiB | 6 of 6 |
| q34 | 375.256ms | 375.256ms | 375.256ms | 0.0% | 5.310ms | 248.935us | 0.000us | 235.43 MiB | 5 of 5 |
| q35 | 377.748ms | 377.733ms | 377.733ms | 0.0% | 5.818ms | 258.707us | 0.000us | 235.52 MiB | 5 of 5 |
| q36 | 212.612ms | 212.612ms | 212.612ms | 0.0% | 2.464ms | 255.078us | 0.000us | 56.74 MiB | 5 of 5 |
| q37 | 140.004ms | 140.004ms | 140.004ms | 0.0% | 487.139us | 286.635us | 0.000us | 1.58 MiB | 6 of 6 |
| q38 | 211.879ms | 211.874ms | 211.874ms | 0.0% | 477.349us | 272.708us | 0.000us | 515.67 KiB | 6 of 6 |
| q39 | 141.425ms | 141.425ms | 141.425ms | 0.0% | 480.735us | 267.485us | 0.000us | 112.07 KiB | 6 of 6 |
| q40 | 269.064ms | 269.050ms | 269.050ms | 0.0% | 616.663us | 284.744us | 0.000us | 4.97 MiB | 6 of 6 |
| q41 | 34.552ms | 34.499ms | 34.499ms | 0.0% | 436.406us | 262.419us | 0.000us | 272.08 KiB | 6 of 6 |
| q42 | 28.894ms | 28.891ms | 28.891ms | 0.0% | 482.158us | 254.466us | 0.000us | 234.22 KiB | 6 of 6 |
| q43 | 21.835ms | 21.835ms | 21.835ms | 0.0% | 481.489us | 261.972us | 0.000us | 402.55 KiB | 6 of 6 |

Read from the breakdown the engine wrote for its cold run. `accounted` is what its pipelines charged themselves and `measured` is what it measured around running them, so `apart` is the cross check and anything over five percent is time the breakdown cannot explain. `driver` is the part of `accounted` that was not inside an operator, which is the loop that runs a pipeline rather than the operators it calls. `build` is what the engine spent putting the tree together, which is before there is a pipeline to charge and is why it is off the right hand side of the check. `outside` is the CPU the process spent on everything else, which is starting, opening the data and printing the answer, and it is the reason the wall clock column and the query time column differ. `held` is what the operators reserved, which is not the resident set of the process. `reference` is how many operators ran the reference implementation of their seam, which is the slow path kept for differential testing.

#### Where the time went

| kind | cpu | share | operators | rows in | rows out | per row in | per row out | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| FileScan | 3.319s | 57.4% | 41 | 0 | 40998975 | handed none | 80.9ns | 41 of 41 |
| Aggregate | 1.207s | 20.9% | 36 | 17619574 | 4224428 | 68.5ns | 285.7ns | 36 of 36 |
| Filter | 655.150ms | 11.3% | 28 | 26099067 | 3014046 | 25.1ns | 217.4ns | 28 of 28 |
| TopN | 572.562ms | 9.9% | 29 | 4518776 | 250 | 126.7ns | 2290248.0ns | 29 of 29 |
| Project | 26.159ms | 0.5% | 84 | 22532499 | 22532499 | 1.2ns | 1.2ns | 84 of 84 |
| Sort | 4.322us | 0.0% | 1 | 13 | 13 | 332.5ns | 332.5ns | 1 of 1 |
| Limit | 0.354us | 0.0% | 1 | 10 | 10 | 35.4ns | 35.4ns | 1 of 1 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is FileScan, and the queries where it cost the most are q24 at 855.010ms, q23 at 399.121ms, q40 at 261.448ms.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 999975 rows, one out of every 100 of the 99997497 in the full file, which is a development loop rather than the suite
- q15 swung by 12.5% of its median, and rule two wants under 10%
- q22 swung by 16.1% of its median, and rule two wants under 10%
- q25 swung by 11.1% of its median, and rule two wants under 10%

These swung wider than reporting rule two allows:

- duckdb swung by 16.1% of its median on q22, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

rudb did not run q19, because rudb's hash aggregate does not spill and this groups by a key that is close to unique over a hundred million rows, so the table outgrows the machine. tamnd/rudb#220, milestone E5.

rudb did not run q33, because rudb's hash aggregate does not spill and this groups by a key that is close to unique over a hundred million rows, so the table outgrows the machine. tamnd/rudb#220, milestone E5.

So the rudb column is 41 of 43 queries and its ratio is over the shared ones.

All 2 engines agreed on every answer the data settles, which is 37 of 43 queries, to the last significant digit of a double.

These were answered differently and the data does not say which is right:

- q16: ORDER BY COUNT(*) DESC LIMIT 10 over user IDs, where the tenth place is a tie between many users with the same count and the engine picks.
- q18: LIMIT 10 with no ORDER BY at all, so the ten rows are whichever ten the group by handed back first and the query did not ask for any particular ten.
- q23: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q22, and the same on a one million row sample where seven of the ten places have a count of one.
- q32: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q40: ORDER BY PageViews DESC LIMIT 10 OFFSET 1000 with ties at the cut, as q39.
- q41: ORDER BY PageViews DESC LIMIT 10 OFFSET 100 with ties at the cut, as q39.

So they are not checked, and a wrong answer from any engine on one of them would go unnoticed here. Every other query in the suite is checked in full.

Hot here means page cache warm and not buffer pool warm, because every run is a fresh process so that no query's number depends on the one before it.

