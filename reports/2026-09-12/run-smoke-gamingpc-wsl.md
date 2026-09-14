# smoke on gamingpc-wsl

This is one run of the smoke suite on gamingpc-wsl, over 6 engines and 6 queries, with 15 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

## What ran

| what | it was |
| --- | --- |
| suite | smoke |
| queries | 6 |
| tables | 10.41 MiB of Parquet in 1 table |
| rows | 10000000 in the table every query reads |
| summary | median with the interquartile range, per reporting rule two |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run smoke --runs 15 --report
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
| filesystem | tmpfs /tmp tmpfs rw,nosuid,nodev,size=16430952k,nr_inodes=1048576 0 0 | read |
| page cache | cannot open drop_caches: Permission denied (os error 13) | not read |
| timer | /usr/bin/time GNU | read |

A fact that says it was not read is a fact about this machine and not a gap in the report. A field that had silently defaulted would be a lie that survived into it.

## The engines

| engine | version | state | load | load cpu | on disk | that size is | format | machine load |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 157.183ms | 1.820s | 92.01 MiB | its own database file | its own | 1.33 to 1.33 |
| clickhouse-local | 26.9.1.1162 | ran | 265.133ms | 1.010s | 49.53 MiB | its own MergeTree parts, as system.parts counts the active ones | its own | 1.33 to 8.05 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 10.41 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 8.05 to 8.05 |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 10.41 MiB | the source Parquet, this engine has no storage format of its own | the Parquet | 8.05 to 7.27 |
| clickhouse-server | 26.9.1.1162 | ran | 1.394s | not read | 51.31 MiB | its own MergeTree parts, sorted by smoke by (k, tag), as system.parts counts the active ones | its own | 7.27 to 7.10 |
| rudb | rudb 0.2.36 | ran | 0.000us | 0.000us | 10.41 MiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 7.10 to 6.04 |
| duckdb-pinned | not found | set RUDB_BENCH_DUCKDB_PINNED to the DuckDB the grammar is vendored from. `scripts/oracle` in the rudb checkout installs it, and there is no release at that commit to look up | n/a | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: datafusion, polars, rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 77.000ms | 205.744ms | +167% | 226.953ms | 440.000ms | 2.14 | 153.93 MiB | none | 779.22M/s | 811.56 MiB/s | 1.00x |
| clickhouse-local | 499.000ms | 944.452ms | +89% | 942.949ms | 1.580s | 1.67 | 435.89 MiB | none | 120.24M/s | 125.23 MiB/s | 6.48x |
| datafusion | 139.000ms | 200.751ms | +44% | 200.756ms | 1.290s | 6.43 | 433.70 MiB | none | 431.65M/s | 449.57 MiB/s | 1.81x |
| polars | 161.379ms | 684.125ms | +324% | 695.064ms | 1.500s | 2.19 | 292.84 MiB | none | 371.80M/s | 387.22 MiB/s | 2.10x |
| clickhouse-server | 68.000ms | 242.620ms | +257% | 269.171ms | not read | not read | not read | not read | 882.35M/s | 918.97 MiB/s | 0.88x |
| rudb | 1.019s | 1.025s | +1% | 1.026s | 970.000ms | 0.95 | 7.73 MiB | none | 49.07M/s | 51.10 MiB/s | 13.23x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 6 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | clickhouse-local | datafusion | polars | clickhouse-server | rudb |
| --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 2.000ms | 1.000ms | 5.585ms | 1.000ms | 10.000ms |
| q2 | filter and sum | 17.000ms | 104.000ms | 15.000ms | 14.516ms | 3.000ms | 108.000ms |
| q3 | group by, low card | 9.000ms | 91.000ms | 18.000ms | 32.532ms | 8.000ms | 288.000ms |
| q4 | group by and top k | 22.000ms | 90.000ms | 18.000ms | 33.336ms | 7.000ms | 318.000ms |
| q5 | count distinct | 23.000ms | 119.000ms | 20.000ms | 33.674ms | 10.000ms | 295.000ms |
| q6 | join and group by | 5.000ms | 93.000ms | 67.000ms | 41.736ms | 39.000ms | no dialect |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 22.604ms | 21.472ms | 3.3% | 21.062ms | 21.772ms | 20.876ms | 23.913ms | 10.000ms | 39.51 MiB | none | 465.73M/s |
| q2 | filter and sum | 17.000ms | 48.139ms | 38.145ms | 4.6% | 37.462ms | 39.232ms | 36.707ms | 45.013ms | 90.000ms | 128.12 MiB | none | 262.16M/s |
| q3 | group by, low card | 9.000ms | 35.861ms | 30.324ms | 12.2% | 28.936ms | 32.625ms | 27.780ms | 36.903ms | 50.000ms | 66.02 MiB | none | 329.77M/s |
| q4 | group by and top k | 22.000ms | 49.064ms | 43.279ms | 12.3% | 42.675ms | 47.990ms | 41.390ms | 63.777ms | 120.000ms | 130.55 MiB | none | 231.06M/s |
| q5 | count distinct | 23.000ms | 45.582ms | 45.753ms | 10.1% | 43.896ms | 48.497ms | 43.347ms | 65.903ms | 140.000ms | 153.93 MiB | none | 218.57M/s |
| q6 | join and group by | 5.000ms | 25.703ms | 26.772ms | 8.4% | 26.154ms | 28.416ms | 25.574ms | 30.305ms | 30.000ms | 52.46 MiB | none | 373.53M/s |

duckdb v2.0.0-dev84237 (Development Version) cc7e7bac7f over 6 of 6 queries. Total 77.000ms by its own clock and 205.744ms by ours, 226.953ms cold, 440.000ms of CPU, peak 153.93 MiB, 779.22M/s and 811.56 MiB/s.

Running it cost 167% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.13x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 2.000ms | 46.317ms | 42.803ms | 8.0% | 40.502ms | 43.933ms | 40.385ms | 47.831ms | 50.000ms | 198.88 MiB | none | 233.63M/s |
| q2 | filter and sum | 104.000ms | 192.234ms | 183.861ms | 12.4% | 173.788ms | 196.665ms | 162.476ms | 207.239ms | 260.000ms | 375.45 MiB | none | 54.39M/s |
| q3 | group by, low card | 91.000ms | 162.161ms | 172.506ms | 15.7% | 158.034ms | 185.059ms | 135.570ms | 196.177ms | 210.000ms | 384.61 MiB | none | 57.97M/s |
| q4 | group by and top k | 90.000ms | 170.100ms | 171.283ms | 12.2% | 163.024ms | 183.920ms | 122.700ms | 190.484ms | 240.000ms | 392.74 MiB | none | 58.38M/s |
| q5 | count distinct | 119.000ms | 205.173ms | 202.451ms | 21.6% | 185.482ms | 229.271ms | 179.079ms | 258.297ms | 590.000ms | 435.89 MiB | none | 49.39M/s |
| q6 | join and group by | 93.000ms | 166.964ms | 171.547ms | 20.9% | 159.529ms | 195.331ms | 151.807ms | 218.427ms | 230.000ms | 369.82 MiB | none | 58.29M/s |

clickhouse-local 26.9.1.1162 over 6 of 6 queries. Total 499.000ms by its own clock and 944.452ms by ours, 942.949ms cold, 1.580s of CPU, peak 435.89 MiB, 120.24M/s and 125.23 MiB/s.

Running it cost 89% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 4.73x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 11.401ms | 9.948ms | 7.7% | 9.383ms | 10.146ms | 8.819ms | 11.179ms | 0.000us | 77.30 MiB | none | 1.01G/s |
| q2 | filter and sum | 15.000ms | 23.367ms | 25.637ms | 16.8% | 24.187ms | 28.481ms | 23.567ms | 29.519ms | 260.000ms | 290.02 MiB | none | 390.06M/s |
| q3 | group by, low card | 18.000ms | 28.207ms | 27.991ms | 8.0% | 27.501ms | 29.736ms | 25.885ms | 31.857ms | 300.000ms | 351.02 MiB | none | 357.26M/s |
| q4 | group by and top k | 18.000ms | 28.383ms | 28.334ms | 3.8% | 27.872ms | 28.944ms | 24.900ms | 29.430ms | 310.000ms | 367.79 MiB | none | 352.94M/s |
| q5 | count distinct | 20.000ms | 29.955ms | 31.230ms | 7.8% | 29.690ms | 32.132ms | 29.120ms | 34.107ms | 340.000ms | 433.70 MiB | none | 320.20M/s |
| q6 | join and group by | 67.000ms | 79.444ms | 77.612ms | 2.3% | 76.294ms | 78.110ms | 75.312ms | 78.965ms | 80.000ms | 170.07 MiB | none | 128.85M/s |

datafusion datafusion-cli 55.1.0 over 6 of 6 queries. Total 139.000ms by its own clock and 200.751ms by ours, 200.756ms cold, 1.290s of CPU, peak 433.70 MiB, 431.65M/s and 449.57 MiB/s.

Running it cost 44% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 7.80x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 5.585ms | 94.202ms | 90.708ms | 4.1% | 89.029ms | 92.768ms | 87.348ms | 102.337ms | 150.000ms | 61.75 MiB | none | 110.24M/s |
| q2 | filter and sum | 14.516ms | 102.654ms | 97.782ms | 4.2% | 96.174ms | 100.246ms | 92.332ms | 102.760ms | 130.000ms | 94.00 MiB | none | 102.27M/s |
| q3 | group by, low card | 32.532ms | 119.869ms | 121.651ms | 2.8% | 119.755ms | 123.105ms | 113.305ms | 129.882ms | 290.000ms | 258.39 MiB | none | 82.20M/s |
| q4 | group by and top k | 33.336ms | 120.513ms | 120.822ms | 2.4% | 119.664ms | 122.504ms | 118.656ms | 125.966ms | 270.000ms | 240.62 MiB | none | 82.77M/s |
| q5 | count distinct | 33.674ms | 122.345ms | 122.822ms | 1.5% | 121.558ms | 123.430ms | 119.793ms | 125.734ms | 290.000ms | 279.68 MiB | none | 81.42M/s |
| q6 | join and group by | 41.736ms | 135.480ms | 130.339ms | 3.1% | 128.920ms | 132.895ms | 127.002ms | 137.931ms | 370.000ms | 292.84 MiB | none | 76.72M/s |

polars 1.44.2 in sink mode over 6 of 6 queries. Total 161.379ms by its own clock and 684.125ms by ours, 695.064ms cold, 1.500s of CPU, peak 292.84 MiB, 371.80M/s and 387.22 MiB/s.

Running it cost 324% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.44x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-server in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 31.107ms | 28.668ms | 6.4% | 28.089ms | 29.936ms | 27.084ms | 32.352ms | not read | not read | not read | 348.82M/s |
| q2 | filter and sum | 3.000ms | 33.112ms | 30.892ms | 4.0% | 30.595ms | 31.842ms | 30.126ms | 44.512ms | not read | not read | not read | 323.71M/s |
| q3 | group by, low card | 8.000ms | 37.547ms | 36.446ms | 2.5% | 35.887ms | 36.795ms | 35.191ms | 37.822ms | not read | not read | not read | 274.38M/s |
| q4 | group by and top k | 7.000ms | 35.365ms | 36.312ms | 3.2% | 35.787ms | 36.940ms | 35.213ms | 39.833ms | not read | not read | not read | 275.39M/s |
| q5 | count distinct | 10.000ms | 39.908ms | 39.144ms | 5.7% | 37.835ms | 40.067ms | 36.898ms | 41.005ms | not read | not read | not read | 255.47M/s |
| q6 | join and group by | 39.000ms | 92.132ms | 71.158ms | 10.3% | 68.140ms | 75.458ms | 67.330ms | 80.638ms | not read | not read | not read | 140.53M/s |

clickhouse-server 26.9.1.1162 over 6 of 6 queries. Total 68.000ms by its own clock and 242.620ms by ours, 269.171ms cold, no reading of CPU, peak not read, 882.35M/s and 918.97 MiB/s.

Running it cost 257% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.48x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 10.000ms | 11.216ms | 10.808ms | 4.3% | 10.606ms | 11.066ms | 10.426ms | 14.913ms | 0.000us | 4.67 MiB | none | 925.22M/s |
| q2 | filter and sum | 108.000ms | 108.318ms | 109.567ms | 1.0% | 109.118ms | 110.224ms | 108.164ms | 110.903ms | 100.000ms | 7.30 MiB | none | 91.27M/s |
| q3 | group by, low card | 288.000ms | 290.744ms | 289.404ms | 1.1% | 288.198ms | 291.403ms | 286.173ms | 293.521ms | 280.000ms | 6.08 MiB | none | 34.55M/s |
| q4 | group by and top k | 318.000ms | 315.263ms | 319.630ms | 2.4% | 314.013ms | 321.571ms | 307.943ms | 324.035ms | 310.000ms | 7.70 MiB | none | 31.29M/s |
| q5 | count distinct | 295.000ms | 300.499ms | 295.995ms | 3.7% | 287.356ms | 298.254ms | 286.161ms | 301.574ms | 280.000ms | 7.73 MiB | none | 33.78M/s |

rudb rudb 0.2.36 over 5 of 6 queries. Total 1.019s by its own clock and 1.025s by ours, 1.026s cold, 970.000ms of CPU, peak 7.73 MiB, 49.07M/s and 51.10 MiB/s.

Running it cost 1% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 29.57x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | execute | accounted | measured | apart | driver | build | outside | held | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | 9.592ms | 9.592ms | 9.592ms | 0.0% | 4.611ms | 86.025us | 0.000us | 0 B | 4 of 4 |
| q2 | 106.460ms | 106.460ms | 106.460ms | 0.0% | 6.618ms | 94.968us | 0.000us | 0 B | 5 of 5 |
| q3 | 289.027ms | 289.022ms | 289.022ms | 0.0% | 5.223ms | 96.627us | 0.000us | 0 B | 4 of 4 |
| q4 | 313.551ms | 313.539ms | 313.539ms | 0.0% | 5.416ms | 85.755us | 0.000us | 0 B | 5 of 5 |
| q5 | 298.688ms | 298.688ms | 298.688ms | 0.0% | 7.115ms | 87.574us | 0.000us | 0 B | 5 of 5 |

Read from the breakdown the engine wrote for its cold run. `accounted` is what its pipelines charged themselves and `measured` is what it measured around running them, so `apart` is the cross check and anything over five percent is time the breakdown cannot explain. `driver` is the part of `accounted` that was not inside an operator, which is the loop that runs a pipeline rather than the operators it calls. `build` is what the engine spent putting the tree together, which is before there is a pipeline to charge and is why it is off the right hand side of the check. `outside` is the CPU the process spent on everything else, which is starting, opening the data and printing the answer, and it is the reason the wall clock column and the query time column differ. `held` is what the operators reserved, which is not the resident set of the process. `reference` is how many operators ran the reference implementation of their seam, which is the slow path kept for differential testing.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- the smoke suite is not comparable to any board
- q3 swung by 12.2% of its median, and rule two wants under 10%
- q4 swung by 12.3% of its median, and rule two wants under 10%
- q5 swung by 10.1% of its median, and rule two wants under 10%

These swung wider than reporting rule two allows:

- duckdb swung by 12.3% of its median on q4, and rule two wants under 10%
- clickhouse-local swung by 21.6% of its median on q5, and rule two wants under 10%
- datafusion swung by 16.8% of its median on q2, and rule two wants under 10%
- clickhouse-server swung by 10.3% of its median on q6, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

These ran every query at about the same speed:

- polars ran every query within 1.44x of every other one, and this suite spreads over 6x on an engine it is measuring

A column that flat is not a column about the queries. Read those numbers as an upper bound on the engine and not as a measurement of it.

rudb did not run q6, because every join in rudb is a nested loop, so this is a hundred thousand rows against a hundred thousand rows and the number would be about the loop rather than about the join. spec/07-execution.md section 7.4, milestone E3.

So the rudb column is 5 of 6 queries and its ratio is over the shared ones.

All 6 engines agreed on every answer the data settles, which is 6 of 6 queries, to the last significant digit of a double.

Hot here means page cache warm and not buffer pool warm for every row but clickhouse-server, which stayed up across the whole suite with its own caches warm. That is the stronger kind of hot, so a ratio against that column is a ratio between two different quantities.

