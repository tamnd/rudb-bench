# clickbench on gamingpc-wsl

This is one run of the clickbench suite on gamingpc-wsl, over 2 engines and 43 queries, with 5 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

It ran over 9999750 rows, one out of every 10 of the 99997497 in the full file, which is a development loop and not the suite. Nothing here is comparable to a full run or to anybody else's number.

## What ran

| what | it was |
| --- | --- |
| suite | clickbench |
| queries | 43 |
| tables | 1.83 GiB of Parquet in 1 table |
| rows | 9999750 in the table every query reads |
| sample | 9999750 rows, one out of every 10 of the 99997497 in the full file |
| summary | median with the interquartile range, per reporting rule two |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --rows 10000000 --engines duckdb,rudb --runs 5 --report
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
| duckdb | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 7.636s | 88.200s | 3.72 GiB | its own database file | its own | 0.89 to 3.69 |
| rudb | rudb 0.3.5 | ran | 0.000us | 0.000us | 1.83 GiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | 3.69 to 1.05 |
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
| duckdb | 3.015s | 4.224s | +40% | 4.375s | 38.040s | 9.01 | 1.63 GiB | none | 142.62M/s | 26.03 GiB/s | 1.00x |
| rudb | 51.929s | 52.204s | +1% | 52.399s | 51.720s | 0.99 | 1.27 GiB | none | 7.90M/s | 1.44 GiB/s | 19.51x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | rudb |
| --- | --- | --- | --- |
| q1 | count | 2.000ms | 13.201ms |
| q2 | filtered count | 11.000ms | 35.559ms |
| q3 | three aggregates | 15.000ms | 73.191ms |
| q4 | average | 18.000ms | 109.028ms |
| q5 | count distinct, high card | 48.000ms | 635.890ms |
| q6 | count distinct, strings | 34.000ms | 832.869ms |
| q7 | min and max of a date | 3.000ms | 45.545ms |
| q8 | group by, low card | 18.000ms | 37.316ms |
| q9 | group by and count distinct | 63.000ms | 886.307ms |
| q10 | group by, several aggregates | 129.000ms | 1.023s |
| q11 | group by a string and count distinct | 31.000ms | 195.221ms |
| q12 | group by two strings and count distinct | 34.000ms | 236.043ms |
| q13 | group by a string and top k | 50.000ms | 891.608ms |
| q14 | group by a string and count distinct | 99.000ms | 1.087s |
| q15 | group by two columns and top k | 49.000ms | 1.026s |
| q16 | group by, very high card | 56.000ms | 1.323s |
| q17 | group by two, very high card | 146.000ms | 2.628s |
| q18 | group by two, no ordering | 130.000ms | 408.218ms |
| q19 | group by with an extract | 206.000ms | no dialect |
| q20 | point lookup | 15.000ms | 105.594ms |
| q21 | substring scan | 76.000ms | 1.407s |
| q22 | substring scan and group by | 119.000ms | 1.822s |
| q23 | two substring scans and group by | 161.000ms | 3.702s |
| q24 | select star and top k | 125.000ms | 9.097s |
| q25 | top k by a date | 18.000ms | 806.251ms |
| q26 | top k by a string | 21.000ms | 811.632ms |
| q27 | top k by two columns | 15.000ms | 919.952ms |
| q28 | group by with a string length | 105.000ms | 1.403s |
| q29 | group by a regular expression | 330.000ms | 2.463s |
| q30 | ninety sums over one column | 21.000ms | 67.361ms |
| q31 | group by two and several aggregates | 88.000ms | 982.931ms |
| q32 | group by a high card pair | 87.000ms | 995.561ms |
| q33 | group by a high card pair, unfiltered | 148.000ms | no dialect |
| q34 | group by a long string | 213.000ms | 3.304s |
| q35 | group by a constant and a long string | 210.000ms | 3.340s |
| q36 | group by four expressions | 47.000ms | 1.930s |
| q37 | date range and group by a URL | 12.000ms | 1.190s |
| q38 | date range and group by a title | 7.000ms | 1.839s |
| q39 | date range, group by and offset | 9.000ms | 1.177s |
| q40 | date range, a case and a wide group by | 21.000ms | 2.298s |
| q41 | date range with an IN and a hash | 8.000ms | 303.726ms |
| q42 | date range and a deep offset | 10.000ms | 257.921ms |
| q43 | minute buckets over a date range | 7.000ms | 219.068ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 2.000ms | 23.455ms | 23.622ms | 1.6% | 23.615ms | 23.981ms | 23.197ms | 25.728ms | 20.000ms | 41.71 MiB | none | 423.32M/s |
| q2 | filtered count | 11.000ms | 32.071ms | 34.930ms | 9.1% | 32.541ms | 35.710ms | 29.632ms | 35.860ms | 40.000ms | 66.67 MiB | none | 286.28M/s |
| q3 | three aggregates | 15.000ms | 40.228ms | 36.850ms | 10.1% | 36.228ms | 39.955ms | 34.878ms | 48.410ms | 70.000ms | 92.21 MiB | none | 271.36M/s |
| q4 | average | 18.000ms | 58.098ms | 40.206ms | 5.0% | 38.319ms | 40.329ms | 37.397ms | 58.466ms | 90.000ms | 119.71 MiB | none | 248.71M/s |
| q5 | count distinct, high card | 48.000ms | 90.531ms | 74.645ms | 5.1% | 72.848ms | 76.637ms | 70.598ms | 78.089ms | 810.000ms | 372.23 MiB | none | 133.96M/s |
| q6 | count distinct, strings | 34.000ms | 58.455ms | 60.053ms | 0.9% | 59.578ms | 60.096ms | 59.147ms | 63.040ms | 400.000ms | 340.71 MiB | none | 166.51M/s |
| q7 | min and max of a date | 3.000ms | 25.313ms | 24.596ms | 1.9% | 24.482ms | 24.952ms | 24.452ms | 25.505ms | 20.000ms | 43.46 MiB | none | 406.55M/s |
| q8 | group by, low card | 18.000ms | 37.078ms | 41.539ms | 14.7% | 36.494ms | 42.603ms | 35.534ms | 42.788ms | 60.000ms | 70.48 MiB | none | 240.73M/s |
| q9 | group by and count distinct | 63.000ms | 112.878ms | 91.322ms | 1.8% | 89.829ms | 91.429ms | 89.804ms | 96.694ms | 1.010s | 438.73 MiB | none | 109.50M/s |
| q10 | group by, several aggregates | 129.000ms | 155.673ms | 158.943ms | 2.5% | 156.764ms | 160.806ms | 148.069ms | 161.766ms | 1.020s | 509.25 MiB | none | 62.91M/s |
| q11 | group by a string and count distinct | 31.000ms | 51.334ms | 55.830ms | 1.1% | 55.230ms | 55.837ms | 51.285ms | 56.038ms | 200.000ms | 220.09 MiB | none | 179.11M/s |
| q12 | group by two strings and count distinct | 34.000ms | 64.122ms | 60.167ms | 2.9% | 58.838ms | 60.559ms | 58.709ms | 70.591ms | 220.000ms | 241.18 MiB | none | 166.20M/s |
| q13 | group by a string and top k | 50.000ms | 89.957ms | 79.541ms | 17.3% | 66.181ms | 79.971ms | 64.385ms | 82.924ms | 440.000ms | 408.72 MiB | none | 125.72M/s |
| q14 | group by a string and count distinct | 99.000ms | 155.714ms | 130.828ms | 11.6% | 122.521ms | 137.656ms | 119.529ms | 154.024ms | 1.000s | 641.76 MiB | none | 76.43M/s |
| q15 | group by two columns and top k | 49.000ms | 101.907ms | 78.783ms | 16.6% | 73.330ms | 86.419ms | 70.635ms | 91.739ms | 510.000ms | 451.48 MiB | none | 126.93M/s |
| q16 | group by, very high card | 56.000ms | 89.040ms | 85.165ms | 2.3% | 83.266ms | 85.191ms | 82.862ms | 87.552ms | 990.000ms | 444.72 MiB | none | 117.42M/s |
| q17 | group by two, very high card | 146.000ms | 171.607ms | 180.660ms | 10.6% | 176.784ms | 195.912ms | 175.599ms | 200.909ms | 1.830s | 872.62 MiB | none | 55.35M/s |
| q18 | group by two, no ordering | 130.000ms | 160.808ms | 165.234ms | 3.6% | 159.318ms | 165.239ms | 151.729ms | 173.112ms | 1.210s | 822.74 MiB | none | 60.52M/s |
| q19 | group by with an extract | 206.000ms | 249.803ms | 247.332ms | 3.5% | 242.308ms | 250.958ms | 227.994ms | 258.322ms | 2.870s | 1.18 GiB | none | 40.43M/s |
| q20 | point lookup | 15.000ms | 40.548ms | 38.631ms | 3.6% | 37.426ms | 38.828ms | 34.281ms | 41.833ms | 70.000ms | 112.71 MiB | none | 258.85M/s |
| q21 | substring scan | 76.000ms | 124.079ms | 104.980ms | 19.5% | 102.306ms | 122.797ms | 99.409ms | 142.984ms | 940.000ms | 520.95 MiB | none | 95.25M/s |
| q22 | substring scan and group by | 119.000ms | 155.290ms | 149.683ms | 12.6% | 144.692ms | 163.580ms | 135.311ms | 176.135ms | 990.000ms | 638.73 MiB | none | 66.81M/s |
| q23 | two substring scans and group by | 161.000ms | 188.234ms | 193.267ms | 1.6% | 192.253ms | 195.377ms | 183.157ms | 206.298ms | 1.190s | 784.22 MiB | none | 51.74M/s |
| q24 | select star and top k | 125.000ms | 156.967ms | 155.923ms | 6.7% | 152.363ms | 162.789ms | 133.566ms | 164.127ms | 770.000ms | 552.20 MiB | none | 64.13M/s |
| q25 | top k by a date | 18.000ms | 39.980ms | 41.084ms | 29.7% | 35.746ms | 47.947ms | 35.490ms | 49.117ms | 70.000ms | 102.21 MiB | none | 243.40M/s |
| q26 | top k by a string | 21.000ms | 55.342ms | 44.738ms | 3.0% | 43.598ms | 44.951ms | 42.314ms | 50.322ms | 150.000ms | 153.21 MiB | none | 223.52M/s |
| q27 | top k by two columns | 15.000ms | 42.010ms | 39.068ms | 7.8% | 38.334ms | 41.375ms | 35.709ms | 48.453ms | 80.000ms | 102.71 MiB | none | 255.96M/s |
| q28 | group by with a string length | 105.000ms | 147.242ms | 134.876ms | 2.5% | 131.513ms | 134.909ms | 128.135ms | 138.129ms | 730.000ms | 568.47 MiB | none | 74.14M/s |
| q29 | group by a regular expression | 330.000ms | 358.385ms | 372.225ms | 3.9% | 363.108ms | 377.577ms | 348.641ms | 379.130ms | 7.910s | 1.13 GiB | none | 26.86M/s |
| q30 | ninety sums over one column | 21.000ms | 43.924ms | 43.294ms | 3.9% | 42.798ms | 44.492ms | 42.230ms | 51.710ms | 70.000ms | 83.45 MiB | none | 230.97M/s |
| q31 | group by two and several aggregates | 88.000ms | 127.054ms | 117.023ms | 1.8% | 115.585ms | 117.730ms | 115.517ms | 123.376ms | 550.000ms | 428.71 MiB | none | 85.45M/s |
| q32 | group by a high card pair | 87.000ms | 117.820ms | 115.659ms | 3.5% | 115.470ms | 119.539ms | 112.367ms | 121.584ms | 600.000ms | 507.10 MiB | none | 86.46M/s |
| q33 | group by a high card pair, unfiltered | 148.000ms | 195.629ms | 185.767ms | 0.5% | 185.437ms | 186.364ms | 183.327ms | 192.628ms | 3.120s | 1.23 GiB | none | 53.83M/s |
| q34 | group by a long string | 213.000ms | 248.522ms | 258.492ms | 6.7% | 253.544ms | 270.974ms | 253.116ms | 278.848ms | 3.300s | 1.60 GiB | none | 38.68M/s |
| q35 | group by a constant and a long string | 210.000ms | 257.092ms | 254.695ms | 8.1% | 253.642ms | 274.203ms | 252.327ms | 276.606ms | 3.550s | 1.63 GiB | none | 39.26M/s |
| q36 | group by four expressions | 47.000ms | 80.857ms | 74.483ms | 6.6% | 71.661ms | 76.539ms | 71.431ms | 77.808ms | 860.000ms | 395.27 MiB | none | 134.25M/s |
| q37 | date range and group by a URL | 12.000ms | 34.053ms | 34.788ms | 8.7% | 33.747ms | 36.768ms | 33.479ms | 38.105ms | 50.000ms | 75.36 MiB | none | 287.45M/s |
| q38 | date range and group by a title | 7.000ms | 28.991ms | 29.347ms | 1.8% | 29.152ms | 29.671ms | 28.852ms | 30.684ms | 30.000ms | 58.23 MiB | none | 340.75M/s |
| q39 | date range, group by and offset | 9.000ms | 30.380ms | 31.484ms | 1.3% | 31.255ms | 31.661ms | 30.804ms | 33.731ms | 30.000ms | 58.73 MiB | none | 317.61M/s |
| q40 | date range, a case and a wide group by | 21.000ms | 45.713ms | 44.410ms | 2.7% | 44.220ms | 45.401ms | 44.168ms | 45.756ms | 60.000ms | 100.68 MiB | none | 225.17M/s |
| q41 | date range with an IN and a hash | 8.000ms | 28.876ms | 29.973ms | 7.3% | 29.505ms | 31.698ms | 29.437ms | 32.703ms | 40.000ms | 56.16 MiB | none | 333.62M/s |
| q42 | date range and a deep offset | 10.000ms | 30.492ms | 31.444ms | 2.6% | 30.968ms | 31.789ms | 30.558ms | 32.859ms | 40.000ms | 55.42 MiB | none | 318.02M/s |
| q43 | minute buckets over a date range | 7.000ms | 29.569ms | 28.339ms | 1.5% | 28.097ms | 28.524ms | 28.017ms | 29.564ms | 30.000ms | 52.48 MiB | none | 352.87M/s |

duckdb v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 3.015s by its own clock and 4.224s by ours, 4.375s cold, 38.040s of CPU, peak 1.63 GiB, 142.62M/s and 26.03 GiB/s.

Running it cost 40% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 15.76x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 13.201ms | 17.780ms | 17.273ms | 0.4% | 17.220ms | 17.295ms | 16.972ms | 17.629ms | 10.000ms | 8.96 MiB | none | 578.92M/s |
| q2 | filtered count | 35.559ms | 39.249ms | 39.514ms | 2.2% | 38.695ms | 39.549ms | 38.464ms | 40.287ms | 30.000ms | 9.20 MiB | none | 253.07M/s |
| q3 | three aggregates | 73.191ms | 76.626ms | 77.468ms | 0.5% | 77.133ms | 77.495ms | 76.871ms | 77.645ms | 60.000ms | 9.52 MiB | none | 129.08M/s |
| q4 | average | 109.028ms | 115.579ms | 113.562ms | 1.0% | 112.978ms | 114.098ms | 112.775ms | 125.307ms | 100.000ms | 12.76 MiB | none | 88.06M/s |
| q5 | count distinct, high card | 635.890ms | 639.498ms | 640.644ms | 3.3% | 634.908ms | 656.180ms | 634.817ms | 664.052ms | 630.000ms | 120.96 MiB | none | 15.61M/s |
| q6 | count distinct, strings | 832.869ms | 841.458ms | 842.367ms | 0.5% | 839.893ms | 843.806ms | 833.759ms | 844.101ms | 830.000ms | 168.59 MiB | none | 11.87M/s |
| q7 | min and max of a date | 45.545ms | 50.371ms | 49.451ms | 1.0% | 49.310ms | 49.798ms | 49.244ms | 50.241ms | 40.000ms | 9.07 MiB | none | 202.21M/s |
| q8 | group by, low card | 37.316ms | 41.415ms | 41.721ms | 0.9% | 41.449ms | 41.830ms | 41.259ms | 41.977ms | 30.000ms | 9.71 MiB | none | 239.68M/s |
| q9 | group by and count distinct | 886.307ms | 894.969ms | 892.065ms | 1.5% | 890.585ms | 904.238ms | 888.250ms | 909.465ms | 880.000ms | 105.00 MiB | none | 11.21M/s |
| q10 | group by, several aggregates | 1.023s | 1.024s | 1.030s | 0.7% | 1.025s | 1.033s | 1.018s | 1.058s | 1.020s | 108.37 MiB | none | 9.71M/s |
| q11 | group by a string and count distinct | 195.221ms | 208.199ms | 199.821ms | 1.9% | 199.602ms | 203.445ms | 198.205ms | 207.648ms | 190.000ms | 20.58 MiB | none | 50.04M/s |
| q12 | group by two strings and count distinct | 236.043ms | 239.186ms | 240.699ms | 0.4% | 240.179ms | 241.101ms | 239.980ms | 241.339ms | 230.000ms | 20.49 MiB | none | 41.54M/s |
| q13 | group by a string and top k | 891.608ms | 905.914ms | 899.055ms | 0.6% | 899.029ms | 904.056ms | 892.900ms | 924.753ms | 890.000ms | 171.39 MiB | none | 11.12M/s |
| q14 | group by a string and count distinct | 1.087s | 1.095s | 1.096s | 0.5% | 1.093s | 1.099s | 1.092s | 1.107s | 1.080s | 210.16 MiB | none | 9.13M/s |
| q15 | group by two columns and top k | 1.026s | 1.038s | 1.034s | 0.8% | 1.026s | 1.035s | 1.025s | 1.041s | 1.020s | 207.63 MiB | none | 9.68M/s |
| q16 | group by, very high card | 1.323s | 1.348s | 1.331s | 0.2% | 1.331s | 1.334s | 1.326s | 1.340s | 1.320s | 299.28 MiB | none | 7.51M/s |
| q17 | group by two, very high card | 2.628s | 2.617s | 2.640s | 1.3% | 2.618s | 2.653s | 2.614s | 2.672s | 2.630s | 600.43 MiB | none | 3.79M/s |
| q18 | group by two, no ordering | 408.218ms | 410.771ms | 412.711ms | 2.0% | 404.993ms | 413.387ms | 402.389ms | 415.159ms | 400.000ms | 20.86 MiB | none | 24.23M/s |
| q20 | point lookup | 105.594ms | 112.954ms | 109.713ms | 1.2% | 109.595ms | 110.935ms | 108.619ms | 111.394ms | 100.000ms | 12.98 MiB | none | 91.14M/s |
| q21 | substring scan | 1.407s | 1.430s | 1.412s | 0.5% | 1.412s | 1.419s | 1.408s | 1.423s | 1.400s | 95.50 MiB | none | 7.08M/s |
| q22 | substring scan and group by | 1.822s | 1.808s | 1.829s | 1.4% | 1.813s | 1.838s | 1.807s | 1.839s | 1.810s | 107.81 MiB | none | 5.47M/s |
| q23 | two substring scans and group by | 3.702s | 3.737s | 3.709s | 0.6% | 3.700s | 3.722s | 3.696s | 3.725s | 3.700s | 123.68 MiB | none | 2.70M/s |
| q24 | select star and top k | 9.097s | 9.150s | 9.106s | 0.8% | 9.097s | 9.168s | 9.069s | 9.196s | 9.090s | 284.04 MiB | none | 1.10M/s |
| q25 | top k by a date | 806.251ms | 810.537ms | 811.030ms | 0.6% | 808.095ms | 813.186ms | 807.377ms | 815.178ms | 800.000ms | 20.93 MiB | none | 12.33M/s |
| q26 | top k by a string | 811.632ms | 820.483ms | 816.364ms | 0.2% | 814.866ms | 816.418ms | 812.569ms | 816.449ms | 800.000ms | 19.83 MiB | none | 12.25M/s |
| q27 | top k by two columns | 919.952ms | 929.979ms | 924.768ms | 0.7% | 923.837ms | 930.499ms | 922.573ms | 934.667ms | 920.000ms | 20.91 MiB | none | 10.81M/s |
| q28 | group by with a string length | 1.403s | 1.396s | 1.408s | 0.2% | 1.406s | 1.408s | 1.379s | 1.410s | 1.400s | 105.82 MiB | none | 7.10M/s |
| q29 | group by a regular expression | 2.463s | 2.464s | 2.475s | 0.2% | 2.471s | 2.475s | 2.470s | 2.479s | 2.460s | 452.50 MiB | none | 4.04M/s |
| q30 | ninety sums over one column | 67.361ms | 71.919ms | 71.351ms | 0.3% | 71.320ms | 71.507ms | 71.250ms | 72.083ms | 60.000ms | 9.27 MiB | none | 140.15M/s |
| q31 | group by two and several aggregates | 982.931ms | 992.755ms | 988.922ms | 0.6% | 987.536ms | 993.277ms | 977.667ms | 993.339ms | 980.000ms | 190.42 MiB | none | 10.11M/s |
| q32 | group by a high card pair | 995.561ms | 1.002s | 1.002s | 0.4% | 1.000s | 1.004s | 998.662ms | 1.014s | 990.000ms | 232.42 MiB | none | 9.98M/s |
| q34 | group by a long string | 3.304s | 3.428s | 3.323s | 0.6% | 3.318s | 3.337s | 3.289s | 3.354s | 3.310s | 1.27 GiB | none | 3.01M/s |
| q35 | group by a constant and a long string | 3.340s | 3.356s | 3.359s | 0.7% | 3.339s | 3.362s | 3.337s | 3.376s | 3.340s | 1.27 GiB | none | 2.98M/s |
| q36 | group by four expressions | 1.930s | 1.939s | 1.940s | 0.4% | 1.933s | 1.940s | 1.906s | 1.944s | 1.920s | 248.08 MiB | none | 5.16M/s |
| q37 | date range and group by a URL | 1.190s | 1.206s | 1.196s | 1.4% | 1.195s | 1.211s | 1.193s | 1.214s | 1.190s | 103.14 MiB | none | 8.36M/s |
| q38 | date range and group by a title | 1.839s | 1.850s | 1.845s | 2.8% | 1.838s | 1.889s | 1.827s | 1.893s | 1.830s | 55.64 MiB | none | 5.42M/s |
| q39 | date range, group by and offset | 1.177s | 1.191s | 1.183s | 0.4% | 1.179s | 1.184s | 1.178s | 1.185s | 1.170s | 97.87 MiB | none | 8.45M/s |
| q40 | date range, a case and a wide group by | 2.298s | 2.307s | 2.304s | 0.7% | 2.301s | 2.318s | 2.290s | 2.322s | 2.300s | 123.00 MiB | none | 4.34M/s |
| q41 | date range with an IN and a hash | 303.726ms | 306.848ms | 308.049ms | 0.5% | 307.746ms | 309.309ms | 307.139ms | 309.719ms | 300.000ms | 17.17 MiB | none | 32.46M/s |
| q42 | date range and a deep offset | 257.921ms | 262.602ms | 262.051ms | 0.9% | 260.784ms | 263.247ms | 260.017ms | 263.888ms | 250.000ms | 16.67 MiB | none | 38.16M/s |
| q43 | minute buckets over a date range | 219.068ms | 223.676ms | 224.105ms | 0.6% | 222.878ms | 224.136ms | 220.362ms | 227.186ms | 210.000ms | 13.12 MiB | none | 44.62M/s |

rudb rudb 0.3.5 over 41 of 43 queries. Total 51.929s by its own clock and 52.204s by ours, 52.399s cold, 51.720s of CPU, peak 1.27 GiB, 7.90M/s and 1.44 GiB/s.

Running it cost 1% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 527.19x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | execute | accounted | measured | apart | driver | build | outside | held | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | 9.012ms | 9.012ms | 9.012ms | 0.0% | 4.304ms | 1.840ms | 0.000us | 360 B | 4 of 4 |
| q2 | 31.126ms | 31.126ms | 31.126ms | 0.0% | 4.303ms | 1.891ms | 0.000us | 360 B | 5 of 5 |
| q3 | 68.742ms | 68.706ms | 68.706ms | 0.0% | 4.794ms | 1.739ms | 0.000us | 992 B | 4 of 4 |
| q4 | 107.133ms | 107.129ms | 107.129ms | 0.0% | 4.747ms | 1.827ms | 1.044ms | 360 B | 4 of 4 |
| q5 | 630.616ms | 630.599ms | 630.599ms | 0.0% | 5.519ms | 1.793ms | 0.000us | 85.55 MiB | 4 of 4 |
| q6 | 827.230ms | 827.181ms | 827.181ms | 0.0% | 5.399ms | 1.957ms | 861.975us | 141.66 MiB | 4 of 4 |
| q7 | 41.623ms | 41.623ms | 41.623ms | 0.0% | 4.687ms | 1.812ms | 0.000us | 616 B | 4 of 4 |
| q8 | 33.035ms | 33.035ms | 33.035ms | 0.0% | 4.292ms | 1.893ms | 0.000us | 4.86 KiB | 6 of 6 |
| q9 | 885.330ms | 885.244ms | 885.244ms | 0.0% | 6.103ms | 1.805ms | 0.000us | 87.07 MiB | 5 of 5 |
| q10 | 1.013s | 1.013s | 1.013s | 0.0% | 6.423ms | 1.931ms | 0.000us | 88.76 MiB | 5 of 5 |
| q11 | 199.049ms | 199.049ms | 199.049ms | 0.0% | 7.013ms | 1.925ms | 0.000us | 5.32 MiB | 6 of 6 |
| q12 | 230.193ms | 230.193ms | 230.193ms | 0.0% | 7.215ms | 1.801ms | 0.000us | 5.33 MiB | 6 of 6 |
| q13 | 892.900ms | 888.676ms | 888.676ms | 0.0% | 14.274ms | 1.872ms | 0.000us | 186.84 MiB | 6 of 6 |
| q14 | 1.083s | 1.083s | 1.083s | 0.0% | 14.200ms | 1.758ms | 5.007ms | 210.84 MiB | 6 of 6 |
| q15 | 1.026s | 1.026s | 1.026s | 0.0% | 15.163ms | 2.186ms | 1.976ms | 223.24 MiB | 6 of 6 |
| q16 | 1.334s | 1.330s | 1.330s | 0.0% | 15.819ms | 2.081ms | 0.000us | 344.11 MiB | 5 of 5 |
| q17 | 2.598s | 2.593s | 2.593s | 0.0% | 25.645ms | 1.790ms | 4.900ms | 714.22 MiB | 5 of 5 |
| q18 | 401.901ms | 401.884ms | 401.884ms | 0.0% | 5.267ms | 1.880ms | 0.000us | 2.31 KiB | 5 of 5 |
| q20 | 103.688ms | 103.688ms | 103.688ms | 0.0% | 3.273ms | 1.839ms | 0.000us | 168 B | 4 of 4 |
| q21 | 1.420s | 1.420s | 1.420s | 0.0% | 4.370ms | 1.842ms | 0.000us | 360 B | 5 of 5 |
| q22 | 1.797s | 1.797s | 1.797s | 0.0% | 5.166ms | 1.825ms | 0.000us | 92.97 KiB | 6 of 6 |
| q23 | 3.726s | 3.722s | 3.722s | 0.0% | 7.318ms | 1.863ms | 6.238ms | 645.07 KiB | 6 of 6 |
| q24 | 9.137s | 9.129s | 9.129s | 0.0% | 48.227ms | 1.790ms | 0.000us | 586.14 KiB | 5 of 5 |
| q25 | 801.182ms | 798.553ms | 798.553ms | 0.0% | 9.893ms | 1.759ms | 0.000us | 275.78 KiB | 6 of 6 |
| q26 | 811.332ms | 808.922ms | 808.922ms | 0.0% | 9.681ms | 1.771ms | 0.000us | 294.27 KiB | 5 of 5 |
| q27 | 920.702ms | 920.580ms | 920.580ms | 0.0% | 9.717ms | 1.774ms | 0.000us | 392.71 KiB | 6 of 6 |
| q28 | 1.386s | 1.386s | 1.386s | 0.0% | 7.768ms | 1.820ms | 0.000us | 859.44 KiB | 7 of 7 |
| q29 | 2.447s | 2.447s | 2.447s | 0.0% | 17.864ms | 1.894ms | 1.342ms | 390.23 MiB | 7 of 7 |
| q30 | 62.204ms | 62.198ms | 62.198ms | 0.0% | 4.715ms | 1.851ms | 0.000us | 29.59 KiB | 4 of 4 |
| q31 | 982.558ms | 982.500ms | 982.500ms | 0.0% | 12.100ms | 1.860ms | 0.000us | 218.01 MiB | 6 of 6 |
| q32 | 990.627ms | 990.530ms | 990.530ms | 0.0% | 13.289ms | 1.850ms | 0.000us | 240.65 MiB | 6 of 6 |
| q34 | 3.404s | 3.404s | 3.404s | 0.0% | 44.067ms | 1.757ms | 14.313ms | 1.81 GiB | 5 of 5 |
| q35 | 3.332s | 3.327s | 3.327s | 0.0% | 44.838ms | 1.791ms | 10.745ms | 1.81 GiB | 5 of 5 |
| q36 | 1.924s | 1.923s | 1.923s | 0.0% | 14.967ms | 1.849ms | 4.899ms | 245.71 MiB | 5 of 5 |
| q37 | 1.196s | 1.195s | 1.195s | 0.0% | 4.726ms | 1.778ms | 2.848ms | 15.59 MiB | 6 of 6 |
| q38 | 1.841s | 1.836s | 1.836s | 0.0% | 4.965ms | 1.807ms | 1.872ms | 3.00 MiB | 6 of 6 |
| q39 | 1.180s | 1.180s | 1.180s | 0.0% | 4.618ms | 1.841ms | 0.000us | 1.00 MiB | 6 of 6 |
| q40 | 2.296s | 2.296s | 2.296s | 0.0% | 6.398ms | 1.801ms | 2.107ms | 41.04 MiB | 6 of 6 |
| q41 | 297.960ms | 297.894ms | 297.894ms | 0.0% | 4.364ms | 1.836ms | 269.558us | 720.84 KiB | 6 of 6 |
| q42 | 253.518ms | 253.518ms | 253.518ms | 0.0% | 4.574ms | 1.888ms | 0.000us | 1.17 MiB | 6 of 6 |
| q43 | 215.021ms | 215.021ms | 215.021ms | 0.0% | 4.192ms | 1.872ms | 0.000us | 407.06 KiB | 6 of 6 |

Read from the breakdown the engine wrote for its cold run. `accounted` is what its pipelines charged themselves and `measured` is what it measured around running them, so `apart` is the cross check and anything over five percent is time the breakdown cannot explain. `driver` is the part of `accounted` that was not inside an operator, which is the loop that runs a pipeline rather than the operators it calls. `build` is what the engine spent putting the tree together, which is before there is a pipeline to charge and is why it is off the right hand side of the check. `outside` is the CPU the process spent on everything else, which is starting, opening the data and printing the answer, and it is the reason the wall clock column and the query time column differ. `held` is what the operators reserved, which is not the resident set of the process. `reference` is how many operators ran the reference implementation of their seam, which is the slow path kept for differential testing.

#### Where the time went

| kind | cpu | share | operators | rows in | rows out | per row in | per row out | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| FileScan | 28.198s | 54.8% | 41 | 0 | 409989750 | handed none | 68.8ns | 41 of 41 |
| Aggregate | 13.427s | 26.1% | 36 | 176215170 | 27929561 | 76.2ns | 480.7ns | 36 of 36 |
| Filter | 5.069s | 9.9% | 28 | 260603326 | 30171383 | 19.5ns | 168.0ns | 28 of 28 |
| TopN | 4.525s | 8.8% | 29 | 31272163 | 292 | 144.7ns | 15495589.7ns | 29 of 29 |
| Project | 228.614ms | 0.4% | 84 | 211439819 | 211439819 | 1.1ns | 1.1ns | 84 of 84 |
| Sort | 4.858us | 0.0% | 1 | 15 | 15 | 323.9ns | 323.9ns | 1 of 1 |
| Limit | 0.277us | 0.0% | 1 | 10 | 10 | 27.7ns | 27.7ns | 1 of 1 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is FileScan, and the queries where it cost the most are q24 at 7.495s, q23 at 3.041s, q40 at 2.163s.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 9999750 rows, one out of every 10 of the 99997497 in the full file, which is a development loop rather than the suite
- q3 swung by 10.1% of its median, and rule two wants under 10%
- q8 swung by 14.7% of its median, and rule two wants under 10%
- q13 swung by 17.3% of its median, and rule two wants under 10%
- q14 swung by 11.6% of its median, and rule two wants under 10%
- q15 swung by 16.6% of its median, and rule two wants under 10%
- q17 swung by 10.6% of its median, and rule two wants under 10%
- q21 swung by 19.5% of its median, and rule two wants under 10%
- q22 swung by 12.6% of its median, and rule two wants under 10%
- q25 swung by 29.7% of its median, and rule two wants under 10%

These swung wider than reporting rule two allows:

- duckdb swung by 29.7% of its median on q25, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

rudb did not run q19, because rudb's hash aggregate does not spill and this groups by a key that is close to unique over a hundred million rows, so the table outgrows the machine. tamnd/rudb#220, milestone E5.

rudb did not run q33, because rudb's hash aggregate does not spill and this groups by a key that is close to unique over a hundred million rows, so the table outgrows the machine. tamnd/rudb#220, milestone E5.

So the rudb column is 41 of 43 queries and its ratio is over the shared ones.

All 2 engines agreed on every answer the data settles, which is 35 of 43 queries, to the last significant digit of a double.

These were answered differently and the data does not say which is right:

- q18: LIMIT 10 with no ORDER BY at all, so the ten rows are whichever ten the group by handed back first and the query did not ask for any particular ten.
- q22: ORDER BY COUNT(*) DESC LIMIT 10 over search phrases whose URL matches, where the counts at the cut are twos and ones, so which phrases fill the ten is the engine's choice. Both engines returned 4, 3, 2, 2, 2, 2, 2, 2, 1, 1 on a ten million row sample and kept different phrases for the tied places.
- q29: ORDER BY an AVG of a length DESC LIMIT 25, which is both a tie at the cut and a double computed in a different order by each engine.
- q31: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q32: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q39: ORDER BY PageViews DESC LIMIT 10 OFFSET 1000, which is a tie at the cut a thousand rows deeper in, where the counts are smaller and the ties are denser.
- q40: ORDER BY PageViews DESC LIMIT 10 OFFSET 1000 with ties at the cut, as q39.
- q41: ORDER BY PageViews DESC LIMIT 10 OFFSET 100 with ties at the cut, as q39.

So they are not checked, and a wrong answer from any engine on one of them would go unnoticed here. Every other query in the suite is checked in full.

Hot here means page cache warm and not buffer pool warm, because every run is a fresh process so that no query's number depends on the one before it.

