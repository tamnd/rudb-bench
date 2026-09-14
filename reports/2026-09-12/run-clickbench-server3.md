# clickbench on server3

This is one run of the clickbench suite on server3, over 2 engines and 43 queries, with 5 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

It ran over 999975 rows, one out of every 100 of the 99997497 in the full file, which is a development loop and not the suite. Nothing here is comparable to a full run or to anybody else's number.

## What ran

| what | it was |
| --- | --- |
| suite | clickbench |
| queries | 43 |
| tables | 138.65 MiB of Parquet in 1 table |
| rows | 999975 in the table every query reads |
| sample | 999975 rows, one out of every 100 of the 99997497 in the full file |
| summary | median with the interquartile range, per reporting rule two |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --rows 1000000 --engines duckdb,datafusion --runs 5 --report
```

The data goes under `RUDB_BENCH_DATA`, which defaults to `~/rudb-data`, and the engines have to be on the path. An engine that is not gets a row in the abstentions below rather than being left out of the table.

## The machine

| fact | value | source |
| --- | --- | --- |
| host | vmi3391933 | read |
| os | Linux 6.8.0-106-generic x86_64 | read |
| cpu | AMD EPYC Processor (with IBPB) | read |
| threads | 8 | read |
| memory | 23.47 GiB | read |
| governor | no cpufreq here, the policy is not ours to see | not read |
| turbo | no intel_pstate here, the state is not ours to see | not read |
| filesystem | /dev/sda1 / ext4 rw,relatime,discard,errors=remount-ro,commit=30 0 0 | read |
| page cache | droppable, cold runs are cold | read |
| timer | /usr/bin/time GNU | read |

A fact that says it was not read is a fact about this machine and not a gap in the report. A field that had silently defaulted would be a lie that survived into it.

## The engines

| engine | version | state | load | load cpu | on disk | that size is | format |
| --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 9.872s | 43.140s | 501.26 MiB | its own database file | its own |
| datafusion | datafusion-cli 55.0.0 | ran | 0.000us | 0.000us | 138.65 MiB | the source Parquet, this engine has no storage format of its own | the Parquet |
| clickhouse-local | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a |
| polars | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a |
| rudb | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: datafusion. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

## Totals

| engine | hot total | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 10.968s | 11.243s | 32.160s | 2.93 | 233.41 MiB | none | 3.92M/s | 543.55 MiB/s | 1.00x |
| datafusion | 8.131s | 9.231s | 21.950s | 2.70 | 903.03 MiB | none | 5.29M/s | 733.25 MiB/s | 0.74x |

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the hot total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | datafusion |
| --- | --- | --- | --- |
| q1 | count | 108.079ms | 80.073ms |
| q2 | filtered count | 128.848ms | 130.337ms |
| q3 | three aggregates | 123.372ms | 117.347ms |
| q4 | average | 138.554ms | 84.570ms |
| q5 | count distinct, high card | 134.442ms | 179.510ms |
| q6 | count distinct, strings | 182.333ms | 125.124ms |
| q7 | min and max of a date | 139.063ms | 75.038ms |
| q8 | group by, low card | 189.364ms | 94.834ms |
| q9 | group by and count distinct | 347.815ms | 166.830ms |
| q10 | group by, several aggregates | 585.943ms | 204.778ms |
| q11 | group by a string and count distinct | 201.183ms | 164.603ms |
| q12 | group by two strings and count distinct | 251.858ms | 183.159ms |
| q13 | group by a string and top k | 345.798ms | 119.083ms |
| q14 | group by a string and count distinct | 263.558ms | 148.463ms |
| q15 | group by two columns and top k | 246.971ms | 156.731ms |
| q16 | group by, very high card | 276.757ms | 136.486ms |
| q17 | group by two, very high card | 366.298ms | 175.193ms |
| q18 | group by two, no ordering | 319.522ms | 200.562ms |
| q19 | group by with an extract | 409.194ms | 258.511ms |
| q20 | point lookup | 118.227ms | 104.870ms |
| q21 | substring scan | 189.355ms | 180.255ms |
| q22 | substring scan and group by | 222.045ms | 268.347ms |
| q23 | two substring scans and group by | 278.496ms | 328.159ms |
| q24 | select star and top k | 521.407ms | 702.610ms |
| q25 | top k by a date | 197.664ms | 206.131ms |
| q26 | top k by a string | 176.702ms | 133.391ms |
| q27 | top k by two columns | 138.912ms | 132.425ms |
| q28 | group by with a string length | 369.623ms | 247.781ms |
| q29 | group by a regular expression | 739.505ms | 432.152ms |
| q30 | ninety sums over one column | 210.782ms | 168.568ms |
| q31 | group by two and several aggregates | 292.343ms | 209.089ms |
| q32 | group by a high card pair | 210.767ms | 169.921ms |
| q33 | group by a high card pair, unfiltered | 273.814ms | 244.792ms |
| q34 | group by a long string | 495.433ms | 298.443ms |
| q35 | group by a constant and a long string | 380.734ms | 275.074ms |
| q36 | group by four expressions | 219.327ms | 143.781ms |
| q37 | date range and group by a URL | 126.862ms | 190.124ms |
| q38 | date range and group by a title | 122.017ms | 169.963ms |
| q39 | date range, group by and offset | 162.449ms | 170.435ms |
| q40 | date range, a case and a wide group by | 159.376ms | 202.072ms |
| q41 | date range with an IN and a hash | 165.014ms | 95.759ms |
| q42 | date range and a deep offset | 185.311ms | 118.150ms |
| q43 | minute buckets over a date range | 253.340ms | 137.344ms |

The hot figure for each query. The spread that belongs next to it is in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 94.930ms | 108.079ms | 21.3% | 98.518ms | 121.513ms | 90.252ms | 122.949ms | 130.000ms | 35.41 MiB | none | 9.25M/s |
| q2 | filtered count | 117.156ms | 128.848ms | 32.9% | 117.476ms | 159.827ms | 100.776ms | 184.930ms | 190.000ms | 38.66 MiB | none | 7.76M/s |
| q3 | three aggregates | 106.539ms | 123.372ms | 9.9% | 113.989ms | 126.231ms | 108.067ms | 151.106ms | 190.000ms | 41.91 MiB | none | 8.11M/s |
| q4 | average | 125.918ms | 138.554ms | 37.2% | 109.315ms | 160.813ms | 100.325ms | 188.367ms | 300.000ms | 45.41 MiB | none | 7.22M/s |
| q5 | count distinct, high card | 126.765ms | 134.442ms | 6.4% | 129.272ms | 137.927ms | 129.165ms | 176.638ms | 290.000ms | 80.03 MiB | none | 7.44M/s |
| q6 | count distinct, strings | 202.567ms | 182.333ms | 24.2% | 140.348ms | 184.480ms | 131.275ms | 199.891ms | 480.000ms | 72.40 MiB | none | 5.48M/s |
| q7 | min and max of a date | 159.077ms | 139.063ms | 34.2% | 105.371ms | 152.917ms | 89.284ms | 159.974ms | 180.000ms | 35.78 MiB | none | 7.19M/s |
| q8 | group by, low card | 169.787ms | 189.364ms | 10.6% | 169.748ms | 189.913ms | 140.850ms | 285.984ms | 340.000ms | 41.66 MiB | none | 5.28M/s |
| q9 | group by and count distinct | 296.264ms | 347.815ms | 30.1% | 303.696ms | 408.509ms | 198.215ms | 535.241ms | 1.260s | 88.91 MiB | none | 2.88M/s |
| q10 | group by, several aggregates | 347.889ms | 585.943ms | 46.7% | 470.293ms | 743.779ms | 418.720ms | 950.439ms | 2.100s | 100.28 MiB | none | 1.71M/s |
| q11 | group by a string and count distinct | 248.294ms | 201.183ms | 17.2% | 187.763ms | 222.442ms | 140.362ms | 371.805ms | 470.000ms | 63.41 MiB | none | 4.97M/s |
| q12 | group by two strings and count distinct | 379.075ms | 251.858ms | 26.3% | 192.548ms | 258.909ms | 140.470ms | 264.762ms | 490.000ms | 66.03 MiB | none | 3.97M/s |
| q13 | group by a string and top k | 176.280ms | 345.798ms | 20.8% | 301.863ms | 373.727ms | 179.291ms | 405.872ms | 1.260s | 77.91 MiB | none | 2.89M/s |
| q14 | group by a string and count distinct | 482.598ms | 263.558ms | 21.0% | 227.723ms | 282.941ms | 221.277ms | 378.885ms | 730.000ms | 103.91 MiB | none | 3.79M/s |
| q15 | group by two columns and top k | 153.019ms | 246.971ms | 7.8% | 242.104ms | 261.445ms | 209.197ms | 343.664ms | 600.000ms | 83.48 MiB | none | 4.05M/s |
| q16 | group by, very high card | 290.317ms | 276.757ms | 22.9% | 249.967ms | 313.324ms | 238.853ms | 315.735ms | 890.000ms | 92.91 MiB | none | 3.61M/s |
| q17 | group by two, very high card | 317.990ms | 366.298ms | 63.4% | 346.703ms | 578.963ms | 334.479ms | 619.262ms | 1.440s | 147.78 MiB | none | 2.73M/s |
| q18 | group by two, no ordering | 310.752ms | 319.522ms | 5.0% | 318.025ms | 333.985ms | 268.659ms | 362.108ms | 1.160s | 142.53 MiB | none | 3.13M/s |
| q19 | group by with an extract | 458.035ms | 409.194ms | 54.4% | 349.188ms | 571.871ms | 308.379ms | 762.881ms | 1.560s | 162.40 MiB | none | 2.44M/s |
| q20 | point lookup | 137.690ms | 118.227ms | 21.3% | 112.094ms | 137.231ms | 98.911ms | 177.066ms | 190.000ms | 44.15 MiB | none | 8.46M/s |
| q21 | substring scan | 262.872ms | 189.355ms | 15.2% | 179.693ms | 208.401ms | 172.586ms | 320.468ms | 490.000ms | 96.53 MiB | none | 5.28M/s |
| q22 | substring scan and group by | 178.037ms | 222.045ms | 10.4% | 212.520ms | 235.633ms | 207.441ms | 295.932ms | 620.000ms | 114.53 MiB | none | 4.50M/s |
| q23 | two substring scans and group by | 386.676ms | 278.496ms | 14.5% | 256.073ms | 296.378ms | 244.663ms | 370.460ms | 710.000ms | 147.91 MiB | none | 3.59M/s |
| q24 | select star and top k | 441.061ms | 521.407ms | 17.6% | 432.610ms | 524.613ms | 394.530ms | 733.422ms | 1.790s | 210.41 MiB | none | 1.92M/s |
| q25 | top k by a date | 150.068ms | 197.664ms | 40.7% | 158.684ms | 239.181ms | 129.394ms | 286.849ms | 530.000ms | 57.40 MiB | none | 5.06M/s |
| q26 | top k by a string | 225.653ms | 176.702ms | 16.8% | 165.357ms | 195.092ms | 143.794ms | 323.638ms | 390.000ms | 52.03 MiB | none | 5.66M/s |
| q27 | top k by two columns | 125.648ms | 138.912ms | 5.9% | 138.230ms | 146.477ms | 117.579ms | 156.198ms | 270.000ms | 57.78 MiB | none | 7.20M/s |
| q28 | group by with a string length | 237.055ms | 369.623ms | 36.7% | 250.091ms | 385.614ms | 214.999ms | 436.983ms | 860.000ms | 108.54 MiB | none | 2.71M/s |
| q29 | group by a regular expression | 816.336ms | 739.505ms | 6.5% | 717.122ms | 764.897ms | 644.531ms | 960.228ms | 3.630s | 168.78 MiB | none | 1.35M/s |
| q30 | ninety sums over one column | 212.147ms | 210.782ms | 5.1% | 207.187ms | 217.907ms | 195.470ms | 333.060ms | 380.000ms | 52.15 MiB | none | 4.74M/s |
| q31 | group by two and several aggregates | 322.147ms | 292.343ms | 8.3% | 276.636ms | 300.832ms | 242.604ms | 464.458ms | 640.000ms | 86.78 MiB | none | 3.42M/s |
| q32 | group by a high card pair | 651.314ms | 210.767ms | 21.1% | 190.821ms | 235.191ms | 184.580ms | 308.081ms | 450.000ms | 97.03 MiB | none | 4.74M/s |
| q33 | group by a high card pair, unfiltered | 335.252ms | 273.814ms | 26.7% | 214.903ms | 287.960ms | 198.168ms | 390.218ms | 850.000ms | 150.79 MiB | none | 3.65M/s |
| q34 | group by a long string | 363.715ms | 495.433ms | 37.3% | 452.745ms | 637.550ms | 307.213ms | 902.177ms | 1.970s | 233.41 MiB | none | 2.02M/s |
| q35 | group by a constant and a long string | 391.644ms | 380.734ms | 25.4% | 364.883ms | 461.640ms | 318.981ms | 704.083ms | 1.320s | 232.15 MiB | none | 2.63M/s |
| q36 | group by four expressions | 170.975ms | 219.327ms | 118.9% | 185.071ms | 445.884ms | 168.094ms | 689.375ms | 550.000ms | 84.16 MiB | none | 4.56M/s |
| q37 | date range and group by a URL | 155.621ms | 126.862ms | 20.6% | 113.598ms | 139.686ms | 112.747ms | 151.334ms | 200.000ms | 45.53 MiB | none | 7.88M/s |
| q38 | date range and group by a title | 124.002ms | 122.017ms | 12.4% | 113.400ms | 128.512ms | 111.875ms | 136.548ms | 220.000ms | 43.78 MiB | none | 8.20M/s |
| q39 | date range, group by and offset | 173.963ms | 162.449ms | 22.3% | 140.267ms | 176.535ms | 127.317ms | 355.807ms | 200.000ms | 43.65 MiB | none | 6.16M/s |
| q40 | date range, a case and a wide group by | 387.661ms | 159.376ms | 36.2% | 142.222ms | 199.988ms | 121.827ms | 256.176ms | 390.000ms | 53.28 MiB | none | 6.27M/s |
| q41 | date range with an IN and a hash | 121.402ms | 165.014ms | 11.9% | 162.237ms | 181.910ms | 151.376ms | 182.629ms | 350.000ms | 44.16 MiB | none | 6.06M/s |
| q42 | date range and a deep offset | 159.091ms | 185.311ms | 79.4% | 145.772ms | 292.912ms | 139.481ms | 567.216ms | 430.000ms | 43.91 MiB | none | 5.40M/s |
| q43 | minute buckets over a date range | 149.818ms | 253.340ms | 15.6% | 248.870ms | 288.497ms | 179.402ms | 362.845ms | 670.000ms | 42.28 MiB | none | 3.95M/s |

duckdb v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 10.968s hot and 11.243s cold, 32.160s of CPU, peak 233.41 MiB, 3.92M/s and 543.55 MiB/s.

Its slowest query is 6.84x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 140.083ms | 80.073ms | 9.4% | 75.253ms | 82.786ms | 56.575ms | 87.542ms | 90.000ms | 73.90 MiB | none | 12.49M/s |
| q2 | filtered count | 138.819ms | 130.337ms | 36.3% | 104.030ms | 151.282ms | 86.770ms | 182.912ms | 200.000ms | 104.73 MiB | none | 7.67M/s |
| q3 | three aggregates | 114.391ms | 117.347ms | 83.3% | 85.013ms | 182.727ms | 83.787ms | 185.689ms | 180.000ms | 131.27 MiB | none | 8.52M/s |
| q4 | average | 110.444ms | 84.570ms | 18.5% | 81.468ms | 97.135ms | 79.396ms | 185.392ms | 100.000ms | 107.34 MiB | none | 11.82M/s |
| q5 | count distinct, high card | 173.839ms | 179.510ms | 23.4% | 148.141ms | 190.073ms | 122.294ms | 201.406ms | 430.000ms | 249.53 MiB | none | 5.57M/s |
| q6 | count distinct, strings | 159.478ms | 125.124ms | 71.0% | 115.407ms | 204.258ms | 109.082ms | 219.482ms | 290.000ms | 277.77 MiB | none | 7.99M/s |
| q7 | min and max of a date | 55.440ms | 75.038ms | 36.0% | 68.488ms | 95.501ms | 60.890ms | 101.423ms | 80.000ms | 77.31 MiB | none | 13.33M/s |
| q8 | group by, low card | 81.341ms | 94.834ms | 10.3% | 88.092ms | 97.870ms | 86.024ms | 111.653ms | 120.000ms | 110.58 MiB | none | 10.54M/s |
| q9 | group by and count distinct | 149.767ms | 166.830ms | 5.4% | 166.300ms | 175.229ms | 135.410ms | 295.422ms | 460.000ms | 282.43 MiB | none | 5.99M/s |
| q10 | group by, several aggregates | 157.629ms | 204.778ms | 59.5% | 175.813ms | 297.578ms | 158.604ms | 341.501ms | 450.000ms | 245.80 MiB | none | 4.88M/s |
| q11 | group by a string and count distinct | 168.402ms | 164.603ms | 86.5% | 143.093ms | 285.423ms | 101.879ms | 313.969ms | 250.000ms | 175.91 MiB | none | 6.08M/s |
| q12 | group by two strings and count distinct | 120.817ms | 183.159ms | 22.4% | 165.090ms | 206.202ms | 145.248ms | 316.362ms | 360.000ms | 188.02 MiB | none | 5.46M/s |
| q13 | group by a string and top k | 178.849ms | 119.083ms | 16.5% | 116.521ms | 136.122ms | 106.493ms | 143.649ms | 280.000ms | 293.37 MiB | none | 8.40M/s |
| q14 | group by a string and count distinct | 191.776ms | 148.463ms | 35.2% | 142.235ms | 194.431ms | 131.431ms | 209.618ms | 390.000ms | 322.04 MiB | none | 6.74M/s |
| q15 | group by two columns and top k | 131.511ms | 156.731ms | 22.0% | 141.702ms | 176.226ms | 125.060ms | 184.674ms | 360.000ms | 319.70 MiB | none | 6.38M/s |
| q16 | group by, very high card | 172.580ms | 136.486ms | 27.3% | 115.177ms | 152.459ms | 108.959ms | 155.563ms | 390.000ms | 279.16 MiB | none | 7.33M/s |
| q17 | group by two, very high card | 235.591ms | 175.193ms | 7.7% | 164.723ms | 178.132ms | 159.874ms | 234.056ms | 610.000ms | 428.18 MiB | none | 5.71M/s |
| q18 | group by two, no ordering | 203.251ms | 200.562ms | 17.8% | 192.055ms | 227.756ms | 169.137ms | 334.866ms | 680.000ms | 404.40 MiB | none | 4.99M/s |
| q19 | group by with an extract | 255.240ms | 258.511ms | 21.4% | 236.111ms | 291.385ms | 206.666ms | 478.699ms | 960.000ms | 456.95 MiB | none | 3.87M/s |
| q20 | point lookup | 90.159ms | 104.870ms | 68.7% | 88.660ms | 160.674ms | 86.819ms | 162.854ms | 240.000ms | 106.62 MiB | none | 9.54M/s |
| q21 | substring scan | 252.892ms | 180.255ms | 20.7% | 176.108ms | 213.369ms | 162.620ms | 223.849ms | 490.000ms | 226.22 MiB | none | 5.55M/s |
| q22 | substring scan and group by | 187.675ms | 268.347ms | 4.1% | 259.613ms | 270.630ms | 230.161ms | 288.559ms | 900.000ms | 286.01 MiB | none | 3.73M/s |
| q23 | two substring scans and group by | 481.739ms | 328.159ms | 20.7% | 311.763ms | 379.842ms | 306.954ms | 478.912ms | 1.310s | 419.53 MiB | none | 3.05M/s |
| q24 | select star and top k | 1.297s | 702.610ms | 20.8% | 688.535ms | 834.643ms | 666.702ms | 907.079ms | 3.030s | 903.03 MiB | none | 1.42M/s |
| q25 | top k by a date | 281.135ms | 206.131ms | 19.8% | 182.943ms | 223.654ms | 124.070ms | 261.502ms | 440.000ms | 241.12 MiB | none | 4.85M/s |
| q26 | top k by a string | 127.270ms | 133.391ms | 18.8% | 129.955ms | 155.073ms | 123.561ms | 157.163ms | 260.000ms | 216.48 MiB | none | 7.50M/s |
| q27 | top k by two columns | 118.113ms | 132.425ms | 44.7% | 131.904ms | 191.144ms | 128.913ms | 202.936ms | 310.000ms | 248.75 MiB | none | 7.55M/s |
| q28 | group by with a string length | 192.572ms | 247.781ms | 26.7% | 241.164ms | 307.300ms | 155.855ms | 327.232ms | 700.000ms | 266.55 MiB | none | 4.04M/s |
| q29 | group by a regular expression | 370.108ms | 432.152ms | 33.5% | 366.441ms | 511.283ms | 364.750ms | 611.423ms | 1.520s | 389.14 MiB | none | 2.31M/s |
| q30 | ninety sums over one column | 250.655ms | 168.568ms | 26.0% | 132.239ms | 176.095ms | 117.965ms | 215.453ms | 210.000ms | 126.58 MiB | none | 5.93M/s |
| q31 | group by two and several aggregates | 147.880ms | 209.089ms | 43.1% | 157.737ms | 247.860ms | 152.802ms | 302.882ms | 450.000ms | 271.55 MiB | none | 4.78M/s |
| q32 | group by a high card pair | 148.650ms | 169.921ms | 25.0% | 151.266ms | 193.790ms | 137.453ms | 232.264ms | 440.000ms | 288.16 MiB | none | 5.88M/s |
| q33 | group by a high card pair, unfiltered | 333.848ms | 244.792ms | 14.7% | 220.629ms | 256.697ms | 210.350ms | 340.836ms | 840.000ms | 402.77 MiB | none | 4.08M/s |
| q34 | group by a long string | 368.955ms | 298.443ms | 9.1% | 285.521ms | 312.650ms | 231.693ms | 321.718ms | 1.210s | 473.54 MiB | none | 3.35M/s |
| q35 | group by a constant and a long string | 561.436ms | 275.074ms | 7.7% | 258.772ms | 279.953ms | 194.225ms | 299.272ms | 980.000ms | 482.37 MiB | none | 3.64M/s |
| q36 | group by four expressions | 118.057ms | 143.781ms | 6.1% | 138.408ms | 147.221ms | 115.029ms | 155.507ms | 370.000ms | 266.89 MiB | none | 6.95M/s |
| q37 | date range and group by a URL | 203.784ms | 190.124ms | 45.0% | 156.879ms | 242.447ms | 133.425ms | 280.742ms | 280.000ms | 157.42 MiB | none | 5.26M/s |
| q38 | date range and group by a title | 182.709ms | 169.963ms | 16.5% | 158.049ms | 186.165ms | 148.171ms | 257.290ms | 260.000ms | 150.29 MiB | none | 5.88M/s |
| q39 | date range, group by and offset | 137.549ms | 170.435ms | 35.3% | 129.888ms | 190.112ms | 109.055ms | 245.451ms | 250.000ms | 145.34 MiB | none | 5.87M/s |
| q40 | date range, a case and a wide group by | 149.909ms | 202.072ms | 4.3% | 196.174ms | 204.934ms | 151.674ms | 409.199ms | 310.000ms | 204.18 MiB | none | 4.95M/s |
| q41 | date range with an IN and a hash | 91.842ms | 95.759ms | 46.0% | 80.891ms | 124.939ms | 78.065ms | 181.341ms | 120.000ms | 113.65 MiB | none | 10.44M/s |
| q42 | date range and a deep offset | 98.318ms | 118.150ms | 18.8% | 110.664ms | 132.889ms | 100.446ms | 133.108ms | 160.000ms | 115.23 MiB | none | 8.46M/s |
| q43 | minute buckets over a date range | 98.608ms | 137.344ms | 34.1% | 101.415ms | 148.258ms | 95.311ms | 150.615ms | 190.000ms | 111.74 MiB | none | 7.28M/s |

datafusion datafusion-cli 55.0.0 over 43 of 43 queries. Total 8.131s hot and 9.231s cold, 21.950s of CPU, peak 903.03 MiB, 5.29M/s and 733.25 MiB/s.

Its slowest query is 9.36x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 999975 rows, one out of every 100 of the 99997497 in the full file, which is a development loop rather than the suite
- q1 swung by 21.3% of its median, and rule two wants under 10%
- q2 swung by 32.9% of its median, and rule two wants under 10%
- q4 swung by 37.2% of its median, and rule two wants under 10%
- q6 swung by 24.2% of its median, and rule two wants under 10%
- q7 swung by 34.2% of its median, and rule two wants under 10%
- q8 swung by 10.6% of its median, and rule two wants under 10%
- q9 swung by 30.1% of its median, and rule two wants under 10%
- q10 swung by 46.7% of its median, and rule two wants under 10%
- q11 swung by 17.2% of its median, and rule two wants under 10%
- q12 swung by 26.3% of its median, and rule two wants under 10%
- q13 swung by 20.8% of its median, and rule two wants under 10%
- q14 swung by 21.0% of its median, and rule two wants under 10%
- q16 swung by 22.9% of its median, and rule two wants under 10%
- q17 swung by 63.4% of its median, and rule two wants under 10%
- q19 swung by 54.4% of its median, and rule two wants under 10%
- q20 swung by 21.3% of its median, and rule two wants under 10%
- q21 swung by 15.2% of its median, and rule two wants under 10%
- q22 swung by 10.4% of its median, and rule two wants under 10%
- q23 swung by 14.5% of its median, and rule two wants under 10%
- q24 swung by 17.6% of its median, and rule two wants under 10%
- q25 swung by 40.7% of its median, and rule two wants under 10%
- q26 swung by 16.8% of its median, and rule two wants under 10%
- q28 swung by 36.7% of its median, and rule two wants under 10%
- q32 swung by 21.1% of its median, and rule two wants under 10%
- q33 swung by 26.7% of its median, and rule two wants under 10%
- q34 swung by 37.3% of its median, and rule two wants under 10%
- q35 swung by 25.4% of its median, and rule two wants under 10%
- q36 swung by 118.9% of its median, and rule two wants under 10%
- q37 swung by 20.6% of its median, and rule two wants under 10%
- q38 swung by 12.4% of its median, and rule two wants under 10%
- q39 swung by 22.3% of its median, and rule two wants under 10%
- q40 swung by 36.2% of its median, and rule two wants under 10%
- q41 swung by 11.9% of its median, and rule two wants under 10%
- q42 swung by 79.4% of its median, and rule two wants under 10%
- q43 swung by 15.6% of its median, and rule two wants under 10%

These swung wider than reporting rule two allows:

- duckdb swung by 118.9% of its median on q36, and rule two wants under 10%
- datafusion swung by 86.5% of its median on q11, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

Answers differ, so this is not a comparison: q23: datafusion does not agree with duckdb: 57 numbers against 78

These were answered differently and the data does not say which is right:

- q16: ORDER BY COUNT(*) DESC LIMIT 10 over user IDs, where the tenth place is a tie between many users with the same count and the engine picks.
- q18: LIMIT 10 with no ORDER BY at all, so the ten rows are whichever ten the group by handed back first and the query did not ask for any particular ten.
- q19: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q24: ORDER BY EventTime LIMIT 10, and EventTime has one second resolution over a corpus with millions of rows a second, so the tenth row is a tie.
- q32: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q33: ORDER BY COUNT(*) DESC LIMIT 10 over WatchID and ClientIP unfiltered, where almost every group has a count of one and the ten that come back are arbitrary.
- q40: ORDER BY PageViews DESC LIMIT 10 OFFSET 1000 with ties at the cut, as q39.
- q41: ORDER BY PageViews DESC LIMIT 10 OFFSET 100 with ties at the cut, as q39.

So they are not checked, and a wrong answer from any engine on one of them would go unnoticed here. Every other query in the suite is checked in full.

Hot here means page cache warm and not buffer pool warm, because every run is a fresh process so that no query's number depends on the one before it.

