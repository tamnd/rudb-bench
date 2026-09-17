# clickbench on server3-100k

This is one run of the clickbench suite on server3-100k, over 4 engines and 43 queries, with 5 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

It ran over 99998 rows, one out of every 1000 of the 99997497 in the full file, which is a development loop and not the suite. Nothing here is comparable to a full run or to anybody else's number.

## What ran

| what | it was |
| --- | --- |
| suite | clickbench |
| queries | 43 |
| tables | 15.09 MiB of Parquet in 1 table |
| rows | 99998 in the table every query reads |
| sample | 99998 rows, one out of every 1000 of the 99997497 in the full file |
| summary | median with the interquartile range, per reporting rule two |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --rows 100000 --engines duckdb,clickhouse-local,polars,clickhouse-server --runs 5 --report
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
| duckdb | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 15.545s | 5.880s | 28.76 MiB | its own database file | its own |
| clickhouse-local | 26.9.1.1225 | ran | 5.273s | 2.550s | 24.32 MiB | its own MergeTree parts, as system.parts counts the active ones | its own |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 15.09 MiB | the source Parquet, this engine has no storage format of its own | the Parquet |
| clickhouse-server | 26.9.1.1225 | ran | 3.193s | not read | 23.93 MiB | its own MergeTree parts, sorted by hits by (CounterID, EventDate, UserID, EventTime, WatchID), as system.parts counts the active ones | its own |
| duckdb-pinned | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a |
| datafusion | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a |
| rudb | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: polars. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 4.668s | 19.037s | +308% | 24.002s | 9.670s | 0.51 | 76.98 MiB | none | 921.15K/s | 139.04 MiB/s | 1.00x |
| clickhouse-local | 8.188s | 55.867s | +582% | 63.916s | 27.110s | 0.49 | 245.47 MiB | none | 525.15K/s | 79.27 MiB/s | 1.75x |
| polars | 4.005s | 28.744s | +618% | 32.661s | 28.570s | 0.99 | 119.10 MiB | 204.00 KiB | 973.83K/s | 146.99 MiB/s | 0.86x |
| clickhouse-server | 876.000ms | 7.622s | +770% | 8.562s | not read | not read | not read | not read | 4.91M/s | 740.91 MiB/s | 0.19x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | clickhouse-local | polars | clickhouse-server |
| --- | --- | --- | --- | --- | --- |
| q1 | count | 10.000ms | 49.000ms | 35.223ms | 5.000ms |
| q2 | filtered count | 14.000ms | 25.000ms | 29.001ms | 7.000ms |
| q3 | three aggregates | 31.000ms | 22.000ms | 51.044ms | 8.000ms |
| q4 | average | 42.000ms | 45.000ms | 38.645ms | 8.000ms |
| q5 | count distinct, high card | 81.000ms | 136.000ms | 49.589ms | 17.000ms |
| q6 | count distinct, strings | 41.000ms | 96.000ms | 64.849ms | 13.000ms |
| q7 | min and max of a date | 7.000ms | 93.000ms | 24.351ms | 7.000ms |
| q8 | group by, low card | 98.000ms | 77.000ms | 93.682ms | 13.000ms |
| q9 | group by and count distinct | 105.000ms | 63.000ms | 191.635ms | 20.000ms |
| q10 | group by, several aggregates | 84.000ms | 74.000ms | 109.963ms | 21.000ms |
| q11 | group by a string and count distinct | 34.000ms | 91.000ms | 118.723ms | 9.000ms |
| q12 | group by two strings and count distinct | 37.000ms | 204.000ms | 79.933ms | 9.000ms |
| q13 | group by a string and top k | 78.000ms | 124.000ms | 59.938ms | 13.000ms |
| q14 | group by a string and count distinct | 123.000ms | 186.000ms | 243.914ms | 17.000ms |
| q15 | group by two columns and top k | 46.000ms | 203.000ms | 97.057ms | 17.000ms |
| q16 | group by, very high card | 90.000ms | 125.000ms | 123.469ms | 16.000ms |
| q17 | group by two, very high card | 150.000ms | 378.000ms | 106.472ms | 36.000ms |
| q18 | group by two, no ordering | 93.000ms | 174.000ms | 98.896ms | 15.000ms |
| q19 | group by with an extract | 143.000ms | 490.000ms | 118.517ms | 44.000ms |
| q20 | point lookup | 16.000ms | 144.000ms | 46.074ms | 6.000ms |
| q21 | substring scan | 89.000ms | 167.000ms | 81.563ms | 26.000ms |
| q22 | substring scan and group by | 196.000ms | 188.000ms | 95.413ms | 11.000ms |
| q23 | two substring scans and group by | 203.000ms | 318.000ms | 155.940ms | 22.000ms |
| q24 | select star and top k | 622.000ms | 1.244s | 301.507ms | 122.000ms |
| q25 | top k by a date | 43.000ms | 125.000ms | 130.015ms | 8.000ms |
| q26 | top k by a string | 36.000ms | 138.000ms | 57.756ms | 7.000ms |
| q27 | top k by two columns | 37.000ms | 256.000ms | 89.928ms | 11.000ms |
| q28 | group by with a string length | 125.000ms | 99.000ms | no dialect | 12.000ms |
| q29 | group by a regular expression | 692.000ms | 555.000ms | no dialect | 41.000ms |
| q30 | ninety sums over one column | 237.000ms | 294.000ms | 51.087ms | 27.000ms |
| q31 | group by two and several aggregates | 70.000ms | 142.000ms | 95.233ms | 14.000ms |
| q32 | group by a high card pair | 65.000ms | 133.000ms | 64.239ms | 14.000ms |
| q33 | group by a high card pair, unfiltered | 118.000ms | 272.000ms | 104.164ms | 27.000ms |
| q34 | group by a long string | 147.000ms | 393.000ms | 199.373ms | 46.000ms |
| q35 | group by a constant and a long string | 168.000ms | 297.000ms | 193.316ms | 78.000ms |
| q36 | group by four expressions | 78.000ms | 169.000ms | no dialect | 15.000ms |
| q37 | date range and group by a URL | 52.000ms | 91.000ms | 164.961ms | 14.000ms |
| q38 | date range and group by a title | 38.000ms | 110.000ms | 99.103ms | 18.000ms |
| q39 | date range, group by and offset | 67.000ms | 73.000ms | 82.667ms | 12.000ms |
| q40 | date range, a case and a wide group by | 89.000ms | 116.000ms | 143.627ms | 15.000ms |
| q41 | date range with an IN and a hash | 35.000ms | 69.000ms | 54.546ms | 10.000ms |
| q42 | date range and a deep offset | 88.000ms | 68.000ms | 59.303ms | 13.000ms |
| q43 | minute buckets over a date range | 50.000ms | 72.000ms | no dialect | 12.000ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 10.000ms | 457.146ms | 352.569ms | 100.3% | 350.953ms | 704.746ms | 334.873ms | 778.314ms | 170.000ms | 35.67 MiB | none | 283.63K/s |
| q2 | filtered count | 14.000ms | 399.334ms | 417.647ms | 50.1% | 401.234ms | 610.420ms | 369.830ms | 783.008ms | 150.000ms | 36.03 MiB | none | 239.43K/s |
| q3 | three aggregates | 31.000ms | 405.004ms | 354.566ms | 17.0% | 328.426ms | 388.789ms | 321.483ms | 442.623ms | 160.000ms | 36.66 MiB | 308.00 KiB | 282.03K/s |
| q4 | average | 42.000ms | 535.660ms | 277.590ms | 36.7% | 260.745ms | 362.685ms | 239.964ms | 520.471ms | 120.000ms | 36.91 MiB | none | 360.24K/s |
| q5 | count distinct, high card | 81.000ms | 401.778ms | 501.004ms | 23.9% | 408.772ms | 528.540ms | 337.222ms | 528.605ms | 180.000ms | 42.91 MiB | 1.13 MiB | 199.60K/s |
| q6 | count distinct, strings | 41.000ms | 404.495ms | 498.645ms | 14.4% | 495.350ms | 567.321ms | 251.314ms | 778.150ms | 150.000ms | 39.28 MiB | 128.00 KiB | 200.54K/s |
| q7 | min and max of a date | 7.000ms | 458.494ms | 514.255ms | 12.8% | 450.352ms | 516.030ms | 329.513ms | 685.088ms | 180.000ms | 35.72 MiB | 256.00 KiB | 194.45K/s |
| q8 | group by, low card | 98.000ms | 719.333ms | 396.551ms | 6.7% | 373.652ms | 400.318ms | 327.392ms | 581.408ms | 150.000ms | 38.66 MiB | 192.00 KiB | 252.17K/s |
| q9 | group by and count distinct | 105.000ms | 433.768ms | 456.344ms | 33.8% | 340.174ms | 494.627ms | 313.813ms | 553.239ms | 230.000ms | 49.15 MiB | none | 219.13K/s |
| q10 | group by, several aggregates | 84.000ms | 806.391ms | 411.885ms | 21.5% | 386.939ms | 475.505ms | 332.804ms | 482.337ms | 170.000ms | 51.63 MiB | 212.00 KiB | 242.78K/s |
| q11 | group by a string and count distinct | 34.000ms | 343.888ms | 286.197ms | 35.4% | 284.435ms | 385.729ms | 217.702ms | 437.263ms | 170.000ms | 41.78 MiB | none | 349.40K/s |
| q12 | group by two strings and count distinct | 37.000ms | 446.708ms | 391.159ms | 37.8% | 293.210ms | 440.939ms | 264.803ms | 506.990ms | 150.000ms | 42.79 MiB | none | 255.65K/s |
| q13 | group by a string and top k | 78.000ms | 299.845ms | 365.988ms | 15.0% | 343.044ms | 397.971ms | 326.488ms | 427.700ms | 220.000ms | 41.16 MiB | 128.00 KiB | 273.23K/s |
| q14 | group by a string and count distinct | 123.000ms | 288.341ms | 438.477ms | 7.4% | 415.313ms | 447.855ms | 365.210ms | 526.755ms | 290.000ms | 48.28 MiB | none | 228.06K/s |
| q15 | group by two columns and top k | 46.000ms | 302.124ms | 265.846ms | 29.0% | 230.749ms | 307.793ms | 206.389ms | 373.819ms | 200.000ms | 42.34 MiB | none | 376.15K/s |
| q16 | group by, very high card | 90.000ms | 227.697ms | 328.029ms | 21.1% | 318.804ms | 388.172ms | 267.104ms | 498.319ms | 230.000ms | 46.29 MiB | none | 304.84K/s |
| q17 | group by two, very high card | 150.000ms | 503.645ms | 563.911ms | 54.9% | 273.626ms | 583.378ms | 252.563ms | 614.913ms | 270.000ms | 53.52 MiB | none | 177.33K/s |
| q18 | group by two, no ordering | 93.000ms | 350.130ms | 282.807ms | 28.5% | 233.177ms | 313.760ms | 209.588ms | 313.924ms | 270.000ms | 52.41 MiB | none | 353.59K/s |
| q19 | group by with an extract | 143.000ms | 287.848ms | 520.118ms | 54.3% | 457.927ms | 740.161ms | 429.411ms | 946.142ms | 360.000ms | 56.64 MiB | 256.00 KiB | 192.26K/s |
| q20 | point lookup | 16.000ms | 312.167ms | 315.000ms | 17.9% | 299.706ms | 356.233ms | 247.820ms | 374.960ms | 150.000ms | 36.13 MiB | none | 317.45K/s |
| q21 | substring scan | 89.000ms | 1.095s | 376.526ms | 23.8% | 374.328ms | 464.130ms | 342.847ms | 505.170ms | 160.000ms | 42.36 MiB | none | 265.58K/s |
| q22 | substring scan and group by | 196.000ms | 1.561s | 697.772ms | 91.2% | 398.463ms | 1.035s | 393.649ms | 1.271s | 230.000ms | 46.77 MiB | none | 143.31K/s |
| q23 | two substring scans and group by | 203.000ms | 668.335ms | 716.247ms | 20.6% | 649.299ms | 797.111ms | 636.463ms | 834.718ms | 310.000ms | 52.64 MiB | none | 139.61K/s |
| q24 | select star and top k | 622.000ms | 1.973s | 1.096s | 3.7% | 1.092s | 1.132s | 976.305ms | 1.185s | 600.000ms | 76.98 MiB | 436.00 KiB | 91.21K/s |
| q25 | top k by a date | 43.000ms | 352.656ms | 568.540ms | 30.7% | 416.132ms | 590.461ms | 285.160ms | 893.570ms | 200.000ms | 39.04 MiB | none | 175.89K/s |
| q26 | top k by a string | 36.000ms | 535.529ms | 312.848ms | 57.3% | 249.691ms | 428.825ms | 241.904ms | 463.230ms | 150.000ms | 37.77 MiB | none | 319.64K/s |
| q27 | top k by two columns | 37.000ms | 386.040ms | 497.717ms | 34.6% | 393.940ms | 566.398ms | 222.722ms | 598.578ms | 140.000ms | 38.28 MiB | none | 200.91K/s |
| q28 | group by with a string length | 125.000ms | 502.370ms | 587.000ms | 33.0% | 418.061ms | 612.028ms | 399.484ms | 799.934ms | 230.000ms | 46.03 MiB | 128.00 KiB | 170.35K/s |
| q29 | group by a regular expression | 692.000ms | 2.004s | 1.022s | 21.1% | 899.892ms | 1.116s | 691.598ms | 1.182s | 480.000ms | 50.89 MiB | 140.00 KiB | 97.87K/s |
| q30 | ninety sums over one column | 237.000ms | 898.644ms | 506.747ms | 70.3% | 402.125ms | 758.322ms | 395.292ms | 807.571ms | 280.000ms | 49.20 MiB | 204.00 KiB | 197.33K/s |
| q31 | group by two and several aggregates | 70.000ms | 641.082ms | 387.019ms | 32.1% | 366.406ms | 490.743ms | 273.934ms | 490.815ms | 200.000ms | 44.14 MiB | none | 258.38K/s |
| q32 | group by a high card pair | 65.000ms | 458.592ms | 305.223ms | 5.9% | 294.989ms | 313.126ms | 285.454ms | 328.534ms | 210.000ms | 44.40 MiB | none | 327.62K/s |
| q33 | group by a high card pair, unfiltered | 118.000ms | 502.020ms | 455.439ms | 11.7% | 435.191ms | 488.691ms | 341.247ms | 558.021ms | 250.000ms | 56.27 MiB | none | 219.56K/s |
| q34 | group by a long string | 147.000ms | 587.227ms | 395.506ms | 8.7% | 366.998ms | 401.418ms | 325.087ms | 436.599ms | 310.000ms | 63.40 MiB | none | 252.84K/s |
| q35 | group by a constant and a long string | 168.000ms | 354.818ms | 462.712ms | 29.3% | 402.894ms | 538.291ms | 289.987ms | 563.160ms | 360.000ms | 65.90 MiB | none | 216.11K/s |
| q36 | group by four expressions | 78.000ms | 349.819ms | 289.867ms | 50.8% | 277.038ms | 424.252ms | 268.617ms | 711.559ms | 280.000ms | 46.52 MiB | 196.00 KiB | 344.98K/s |
| q37 | date range and group by a URL | 52.000ms | 264.907ms | 305.197ms | 6.0% | 297.747ms | 316.170ms | 273.771ms | 328.110ms | 150.000ms | 41.90 MiB | 72.00 KiB | 327.65K/s |
| q38 | date range and group by a title | 38.000ms | 410.238ms | 350.134ms | 34.6% | 345.599ms | 466.708ms | 307.872ms | 561.160ms | 160.000ms | 40.90 MiB | none | 285.60K/s |
| q39 | date range, group by and offset | 67.000ms | 303.043ms | 324.120ms | 15.3% | 307.196ms | 356.940ms | 268.945ms | 407.650ms | 210.000ms | 41.18 MiB | none | 308.52K/s |
| q40 | date range, a case and a wide group by | 89.000ms | 246.396ms | 409.941ms | 33.6% | 322.955ms | 460.709ms | 211.383ms | 507.836ms | 230.000ms | 45.78 MiB | none | 243.93K/s |
| q41 | date range with an IN and a hash | 35.000ms | 590.981ms | 372.982ms | 26.1% | 284.062ms | 381.393ms | 229.546ms | 410.911ms | 170.000ms | 41.65 MiB | 120.00 KiB | 268.10K/s |
| q42 | date range and a deep offset | 88.000ms | 419.976ms | 359.721ms | 34.7% | 301.550ms | 426.232ms | 237.420ms | 429.713ms | 200.000ms | 40.92 MiB | 128.00 KiB | 277.99K/s |
| q43 | minute buckets over a date range | 50.000ms | 512.729ms | 299.187ms | 42.4% | 269.935ms | 396.893ms | 254.702ms | 411.374ms | 190.000ms | 39.65 MiB | 128.00 KiB | 334.23K/s |

duckdb v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 4.668s by its own clock and 19.037s by ours, 24.002s cold, 9.670s of CPU, peak 76.98 MiB, 921.15K/s and 139.04 MiB/s.

Running it cost 308% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 4.12x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 49.000ms | 1.143s | 830.102ms | 18.6% | 731.318ms | 885.531ms | 511.400ms | 935.743ms | 510.000ms | 196.00 MiB | 180.00 KiB | 120.46K/s |
| q2 | filtered count | 25.000ms | 1.140s | 642.006ms | 8.1% | 640.580ms | 692.557ms | 481.108ms | 726.878ms | 430.000ms | 195.50 MiB | 304.00 KiB | 155.76K/s |
| q3 | three aggregates | 22.000ms | 912.261ms | 609.978ms | 11.5% | 592.412ms | 662.603ms | 543.011ms | 738.073ms | 450.000ms | 198.88 MiB | 816.00 KiB | 163.94K/s |
| q4 | average | 45.000ms | 861.995ms | 1.103s | 17.4% | 1.004s | 1.195s | 547.502ms | 1.394s | 450.000ms | 198.38 MiB | none | 90.63K/s |
| q5 | count distinct, high card | 136.000ms | 891.133ms | 1.135s | 14.8% | 1.109s | 1.277s | 906.892ms | 1.470s | 490.000ms | 203.10 MiB | 600.00 KiB | 88.14K/s |
| q6 | count distinct, strings | 96.000ms | 997.058ms | 1.508s | 14.7% | 1.297s | 1.519s | 1.049s | 1.689s | 500.000ms | 201.12 MiB | 280.00 KiB | 66.29K/s |
| q7 | min and max of a date | 93.000ms | 1.119s | 1.200s | 5.0% | 1.167s | 1.226s | 1.021s | 1.411s | 530.000ms | 197.50 MiB | 604.00 KiB | 83.32K/s |
| q8 | group by, low card | 77.000ms | 2.400s | 821.124ms | 41.8% | 663.099ms | 1.006s | 615.220ms | 1.184s | 620.000ms | 199.88 MiB | 1.76 MiB | 121.78K/s |
| q9 | group by and count distinct | 63.000ms | 1.796s | 822.618ms | 5.5% | 795.354ms | 840.979ms | 708.054ms | 879.866ms | 670.000ms | 203.77 MiB | 628.00 KiB | 121.56K/s |
| q10 | group by, several aggregates | 74.000ms | 692.006ms | 895.230ms | 27.6% | 704.296ms | 951.444ms | 621.932ms | 1.166s | 1.130s | 205.62 MiB | none | 111.70K/s |
| q11 | group by a string and count distinct | 91.000ms | 750.068ms | 1.438s | 31.4% | 1.137s | 1.589s | 1.089s | 1.603s | 670.000ms | 201.62 MiB | 580.00 KiB | 69.52K/s |
| q12 | group by two strings and count distinct | 204.000ms | 1.395s | 1.122s | 37.1% | 1.005s | 1.421s | 989.657ms | 1.808s | 570.000ms | 202.12 MiB | 480.00 KiB | 89.12K/s |
| q13 | group by a string and top k | 124.000ms | 1.351s | 1.380s | 3.6% | 1.369s | 1.419s | 1.027s | 1.491s | 500.000ms | 205.50 MiB | 64.00 KiB | 72.47K/s |
| q14 | group by a string and count distinct | 186.000ms | 1.309s | 1.422s | 12.2% | 1.393s | 1.567s | 1.230s | 1.604s | 600.000ms | 208.50 MiB | none | 70.32K/s |
| q15 | group by two columns and top k | 203.000ms | 1.289s | 1.312s | 6.0% | 1.309s | 1.388s | 1.270s | 2.238s | 530.000ms | 207.00 MiB | 208.00 KiB | 76.22K/s |
| q16 | group by, very high card | 125.000ms | 2.476s | 1.307s | 3.3% | 1.278s | 1.321s | 1.222s | 1.521s | 520.000ms | 205.57 MiB | 128.00 KiB | 76.51K/s |
| q17 | group by two, very high card | 378.000ms | 1.046s | 1.572s | 16.7% | 1.440s | 1.703s | 1.336s | 1.948s | 490.000ms | 219.38 MiB | none | 63.60K/s |
| q18 | group by two, no ordering | 174.000ms | 2.659s | 1.552s | 15.1% | 1.532s | 1.766s | 1.213s | 1.800s | 500.000ms | 207.18 MiB | 208.00 KiB | 64.42K/s |
| q19 | group by with an extract | 490.000ms | 2.290s | 1.851s | 11.6% | 1.799s | 2.013s | 1.128s | 2.304s | 500.000ms | 217.38 MiB | 128.00 KiB | 54.03K/s |
| q20 | point lookup | 144.000ms | 1.450s | 1.283s | 13.5% | 1.189s | 1.363s | 1.129s | 1.399s | 400.000ms | 198.26 MiB | 212.00 KiB | 77.91K/s |
| q21 | substring scan | 167.000ms | 1.625s | 1.554s | 14.9% | 1.387s | 1.619s | 1.231s | 1.705s | 550.000ms | 203.75 MiB | none | 64.33K/s |
| q22 | substring scan and group by | 188.000ms | 2.010s | 1.478s | 39.8% | 1.014s | 1.602s | 1.004s | 1.693s | 600.000ms | 207.50 MiB | 108.00 KiB | 67.67K/s |
| q23 | two substring scans and group by | 318.000ms | 1.307s | 1.680s | 9.0% | 1.628s | 1.779s | 1.391s | 1.910s | 620.000ms | 209.75 MiB | none | 59.52K/s |
| q24 | select star and top k | 1.244s | 2.780s | 3.221s | 7.0% | 3.195s | 3.420s | 2.182s | 3.503s | 1.100s | 245.47 MiB | 900.00 KiB | 31.04K/s |
| q25 | top k by a date | 125.000ms | 1.281s | 1.612s | 8.7% | 1.567s | 1.708s | 1.483s | 2.155s | 460.000ms | 201.88 MiB | none | 62.05K/s |
| q26 | top k by a string | 138.000ms | 1.946s | 1.860s | 34.0% | 1.377s | 2.009s | 1.306s | 3.355s | 540.000ms | 200.36 MiB | 228.00 KiB | 53.76K/s |
| q27 | top k by two columns | 256.000ms | 1.510s | 2.302s | 29.8% | 2.011s | 2.697s | 1.791s | 4.226s | 750.000ms | 202.12 MiB | none | 43.43K/s |
| q28 | group by with a string length | 99.000ms | 1.405s | 1.481s | 4.9% | 1.473s | 1.546s | 1.414s | 1.586s | 630.000ms | 203.00 MiB | 128.00 KiB | 67.53K/s |
| q29 | group by a regular expression | 555.000ms | 3.579s | 2.192s | 12.1% | 2.116s | 2.381s | 2.088s | 2.932s | 830.000ms | 235.00 MiB | 15.34 MiB | 45.62K/s |
| q30 | ninety sums over one column | 294.000ms | 2.071s | 1.380s | 27.4% | 1.312s | 1.690s | 1.213s | 2.342s | 530.000ms | 202.18 MiB | 544.00 KiB | 72.47K/s |
| q31 | group by two and several aggregates | 142.000ms | 1.787s | 1.181s | 20.8% | 1.082s | 1.327s | 1.014s | 1.499s | 500.000ms | 204.75 MiB | 180.00 KiB | 84.69K/s |
| q32 | group by a high card pair | 133.000ms | 1.376s | 1.291s | 27.1% | 1.128s | 1.478s | 1.041s | 1.510s | 550.000ms | 206.88 MiB | none | 77.45K/s |
| q33 | group by a high card pair, unfiltered | 272.000ms | 1.477s | 1.492s | 29.9% | 1.280s | 1.727s | 1.196s | 1.783s | 750.000ms | 218.88 MiB | none | 67.00K/s |
| q34 | group by a long string | 393.000ms | 1.337s | 1.297s | 9.3% | 1.213s | 1.334s | 1.143s | 1.967s | 650.000ms | 232.02 MiB | none | 77.09K/s |
| q35 | group by a constant and a long string | 297.000ms | 1.929s | 1.765s | 17.0% | 1.501s | 1.800s | 1.070s | 1.957s | 750.000ms | 235.75 MiB | none | 56.65K/s |
| q36 | group by four expressions | 169.000ms | 1.173s | 1.096s | 18.0% | 986.592ms | 1.184s | 939.506ms | 1.503s | 460.000ms | 206.63 MiB | 268.00 KiB | 91.23K/s |
| q37 | date range and group by a URL | 91.000ms | 1.547s | 994.332ms | 4.2% | 958.204ms | 1.000s | 719.024ms | 1.194s | 610.000ms | 207.22 MiB | 552.00 KiB | 100.57K/s |
| q38 | date range and group by a title | 110.000ms | 1.288s | 781.166ms | 18.4% | 774.515ms | 918.066ms | 614.465ms | 956.480ms | 620.000ms | 206.62 MiB | none | 128.01K/s |
| q39 | date range, group by and offset | 73.000ms | 765.998ms | 588.390ms | 12.1% | 528.007ms | 598.996ms | 506.755ms | 719.029ms | 560.000ms | 207.25 MiB | none | 169.95K/s |
| q40 | date range, a case and a wide group by | 116.000ms | 567.357ms | 610.097ms | 27.1% | 534.444ms | 699.923ms | 476.904ms | 716.270ms | 560.000ms | 210.12 MiB | 112.00 KiB | 163.91K/s |
| q41 | date range with an IN and a hash | 69.000ms | 472.770ms | 615.480ms | 10.2% | 592.339ms | 655.221ms | 509.159ms | 720.588ms | 730.000ms | 204.75 MiB | 268.00 KiB | 162.47K/s |
| q42 | date range and a deep offset | 68.000ms | 727.887ms | 512.003ms | 5.8% | 500.438ms | 530.217ms | 499.849ms | 556.847ms | 540.000ms | 204.25 MiB | none | 195.31K/s |
| q43 | minute buckets over a date range | 72.000ms | 1.988s | 1.074s | 103.4% | 858.499ms | 1.969s | 535.467ms | 2.195s | 2.160s | 203.62 MiB | 96.00 KiB | 93.08K/s |

clickhouse-local 26.9.1.1225 over 43 of 43 queries. Total 8.188s by its own clock and 55.867s by ours, 63.916s cold, 27.110s of CPU, peak 245.47 MiB, 525.15K/s and 79.27 MiB/s.

Running it cost 582% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 6.29x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 35.223ms | 2.571s | 905.463ms | 7.9% | 889.215ms | 960.635ms | 689.022ms | 1.295s | 700.000ms | 56.73 MiB | 22.39 MiB | 110.44K/s |
| q2 | filtered count | 29.001ms | 1.313s | 844.585ms | 16.8% | 832.216ms | 973.790ms | 758.614ms | 1.070s | 720.000ms | 61.20 MiB | 5.89 MiB | 118.40K/s |
| q3 | three aggregates | 51.044ms | 818.534ms | 775.527ms | 3.0% | 752.245ms | 775.712ms | 746.656ms | 906.865ms | 690.000ms | 61.36 MiB | 1.99 MiB | 128.94K/s |
| q4 | average | 38.645ms | 805.167ms | 584.952ms | 9.1% | 533.544ms | 586.618ms | 522.051ms | 698.991ms | 560.000ms | 59.97 MiB | 236.00 KiB | 170.95K/s |
| q5 | count distinct, high card | 49.589ms | 631.369ms | 561.721ms | 3.5% | 553.630ms | 573.157ms | 518.747ms | 598.375ms | 570.000ms | 69.32 MiB | 1.24 MiB | 178.02K/s |
| q6 | count distinct, strings | 64.849ms | 685.802ms | 584.334ms | 19.6% | 551.368ms | 666.002ms | 519.272ms | 671.509ms | 600.000ms | 70.83 MiB | 224.00 KiB | 171.13K/s |
| q7 | min and max of a date | 24.351ms | 542.635ms | 644.315ms | 14.5% | 601.199ms | 694.321ms | 493.110ms | 853.346ms | 640.000ms | 59.62 MiB | 880.00 KiB | 155.20K/s |
| q8 | group by, low card | 93.682ms | 1.069s | 1.731s | 15.6% | 1.568s | 1.838s | 1.473s | 1.891s | 880.000ms | 65.10 MiB | 608.00 KiB | 57.78K/s |
| q9 | group by and count distinct | 191.635ms | 1.702s | 1.704s | 4.0% | 1.650s | 1.719s | 1.105s | 1.972s | 940.000ms | 82.39 MiB | 1.15 MiB | 58.68K/s |
| q10 | group by, several aggregates | 109.963ms | 1.498s | 1.340s | 43.7% | 777.092ms | 1.362s | 614.681ms | 1.371s | 760.000ms | 85.12 MiB | 268.00 KiB | 74.61K/s |
| q11 | group by a string and count distinct | 118.723ms | 682.698ms | 839.231ms | 11.3% | 806.276ms | 900.980ms | 763.559ms | 1.088s | 760.000ms | 70.06 MiB | 320.00 KiB | 119.15K/s |
| q12 | group by two strings and count distinct | 79.933ms | 685.572ms | 640.035ms | 18.9% | 620.921ms | 741.736ms | 614.062ms | 819.385ms | 630.000ms | 71.27 MiB | none | 156.24K/s |
| q13 | group by a string and top k | 59.938ms | 515.661ms | 527.000ms | 12.6% | 501.385ms | 567.659ms | 467.848ms | 637.072ms | 570.000ms | 71.33 MiB | none | 189.75K/s |
| q14 | group by a string and count distinct | 243.914ms | 705.375ms | 787.063ms | 28.8% | 625.805ms | 852.461ms | 586.166ms | 954.846ms | 1.130s | 81.08 MiB | none | 127.05K/s |
| q15 | group by two columns and top k | 97.057ms | 637.742ms | 727.340ms | 18.1% | 642.017ms | 773.815ms | 594.194ms | 788.756ms | 840.000ms | 72.63 MiB | none | 137.48K/s |
| q16 | group by, very high card | 123.469ms | 892.032ms | 610.784ms | 6.7% | 601.723ms | 642.780ms | 481.134ms | 649.398ms | 650.000ms | 72.34 MiB | none | 163.72K/s |
| q17 | group by two, very high card | 106.472ms | 675.826ms | 622.251ms | 5.9% | 617.497ms | 654.030ms | 593.496ms | 741.118ms | 700.000ms | 86.61 MiB | none | 160.70K/s |
| q18 | group by two, no ordering | 98.896ms | 591.618ms | 673.012ms | 11.1% | 623.542ms | 698.155ms | 494.438ms | 1.017s | 780.000ms | 84.69 MiB | none | 148.58K/s |
| q19 | group by with an extract | 118.517ms | 721.636ms | 584.553ms | 21.4% | 572.177ms | 697.033ms | 520.842ms | 1.057s | 720.000ms | 89.10 MiB | 764.00 KiB | 171.07K/s |
| q20 | point lookup | 46.074ms | 457.423ms | 509.307ms | 7.1% | 484.572ms | 520.779ms | 462.156ms | 727.765ms | 580.000ms | 60.48 MiB | 172.00 KiB | 196.34K/s |
| q21 | substring scan | 81.563ms | 593.943ms | 566.813ms | 8.2% | 549.630ms | 596.308ms | 478.998ms | 649.337ms | 570.000ms | 73.23 MiB | 1.30 MiB | 176.42K/s |
| q22 | substring scan and group by | 95.413ms | 568.165ms | 596.187ms | 8.3% | 549.784ms | 598.987ms | 533.048ms | 712.849ms | 600.000ms | 78.45 MiB | 256.00 KiB | 167.73K/s |
| q23 | two substring scans and group by | 155.940ms | 576.433ms | 601.514ms | 8.9% | 599.880ms | 653.664ms | 594.042ms | 673.234ms | 730.000ms | 105.49 MiB | none | 166.24K/s |
| q24 | select star and top k | 301.507ms | 665.374ms | 933.662ms | 48.9% | 858.876ms | 1.315s | 749.082ms | 4.041s | 1.240s | 115.85 MiB | 144.00 KiB | 107.10K/s |
| q25 | top k by a date | 130.015ms | 1.197s | 979.299ms | 49.0% | 789.576ms | 1.269s | 678.812ms | 3.138s | 1.120s | 66.30 MiB | none | 102.11K/s |
| q26 | top k by a string | 57.756ms | 617.046ms | 584.203ms | 9.5% | 560.248ms | 615.483ms | 549.614ms | 752.407ms | 590.000ms | 64.28 MiB | 588.00 KiB | 171.17K/s |
| q27 | top k by two columns | 89.928ms | 713.548ms | 698.040ms | 10.4% | 653.280ms | 725.721ms | 521.870ms | 1.046s | 620.000ms | 66.93 MiB | 280.00 KiB | 143.26K/s |
| q30 | ninety sums over one column | 51.087ms | 1.041s | 522.287ms | 26.7% | 520.808ms | 660.410ms | 504.044ms | 911.389ms | 520.000ms | 63.27 MiB | 5.97 MiB | 191.46K/s |
| q31 | group by two and several aggregates | 95.233ms | 879.743ms | 664.186ms | 4.8% | 661.681ms | 693.425ms | 638.652ms | 825.089ms | 720.000ms | 73.62 MiB | 2.59 MiB | 150.56K/s |
| q32 | group by a high card pair | 64.239ms | 600.034ms | 610.024ms | 20.4% | 568.516ms | 693.117ms | 507.856ms | 702.945ms | 640.000ms | 74.84 MiB | 784.00 KiB | 163.92K/s |
| q33 | group by a high card pair, unfiltered | 104.164ms | 779.886ms | 697.752ms | 28.5% | 615.047ms | 813.708ms | 550.542ms | 828.667ms | 800.000ms | 88.85 MiB | none | 143.31K/s |
| q34 | group by a long string | 199.373ms | 999.438ms | 779.114ms | 15.3% | 757.766ms | 876.613ms | 705.626ms | 1.200s | 990.000ms | 110.06 MiB | 2.58 MiB | 128.35K/s |
| q35 | group by a constant and a long string | 193.316ms | 922.664ms | 735.006ms | 13.3% | 725.652ms | 823.164ms | 685.008ms | 855.559ms | 1.040s | 119.10 MiB | 84.00 KiB | 136.05K/s |
| q37 | date range and group by a URL | 164.961ms | 850.844ms | 674.189ms | 28.3% | 578.826ms | 769.706ms | 567.766ms | 808.153ms | 760.000ms | 91.97 MiB | 2.44 MiB | 148.32K/s |
| q38 | date range and group by a title | 99.103ms | 658.696ms | 557.072ms | 20.1% | 539.833ms | 651.825ms | 525.597ms | 694.220ms | 580.000ms | 92.21 MiB | 1.95 MiB | 179.51K/s |
| q39 | date range, group by and offset | 82.667ms | 541.756ms | 605.708ms | 30.7% | 538.235ms | 724.461ms | 470.603ms | 1.097s | 630.000ms | 77.89 MiB | 172.00 KiB | 165.09K/s |
| q40 | date range, a case and a wide group by | 143.627ms | 1.100s | 613.053ms | 39.2% | 603.118ms | 843.589ms | 592.139ms | 1.048s | 740.000ms | 86.36 MiB | 2.20 MiB | 163.11K/s |
| q41 | date range with an IN and a hash | 54.546ms | 655.764ms | 510.279ms | 4.5% | 505.761ms | 528.523ms | 484.450ms | 580.734ms | 550.000ms | 72.12 MiB | 2.25 MiB | 195.97K/s |
| q42 | date range and a deep offset | 59.303ms | 496.825ms | 619.863ms | 6.4% | 619.503ms | 659.212ms | 608.864ms | 685.095ms | 710.000ms | 70.27 MiB | 248.00 KiB | 161.32K/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 4.005s by its own clock and 28.744s by ours, 32.661s cold, 28.570s of CPU, peak 119.10 MiB, 973.83K/s and 146.99 MiB/s.

Running it cost 618% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.40x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-server in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 5.000ms | 244.897ms | 167.645ms | 5.5% | 166.426ms | 175.637ms | 155.003ms | 192.696ms | not read | not read | not read | 596.49K/s |
| q2 | filtered count | 7.000ms | 225.829ms | 185.793ms | 4.3% | 180.433ms | 188.481ms | 174.034ms | 247.805ms | not read | not read | not read | 538.22K/s |
| q3 | three aggregates | 8.000ms | 369.719ms | 160.021ms | 59.2% | 153.253ms | 247.927ms | 149.617ms | 395.353ms | not read | not read | not read | 624.90K/s |
| q4 | average | 8.000ms | 220.003ms | 185.592ms | 26.0% | 173.479ms | 221.670ms | 171.015ms | 280.338ms | not read | not read | not read | 538.81K/s |
| q5 | count distinct, high card | 17.000ms | 248.389ms | 206.401ms | 8.2% | 190.523ms | 207.424ms | 178.419ms | 263.611ms | not read | not read | not read | 484.48K/s |
| q6 | count distinct, strings | 13.000ms | 187.128ms | 174.633ms | 8.8% | 160.235ms | 175.600ms | 159.371ms | 177.588ms | not read | not read | not read | 572.62K/s |
| q7 | min and max of a date | 7.000ms | 283.367ms | 212.314ms | 20.1% | 190.936ms | 233.513ms | 182.740ms | 245.893ms | not read | not read | not read | 470.99K/s |
| q8 | group by, low card | 13.000ms | 226.649ms | 198.135ms | 29.0% | 179.514ms | 237.018ms | 157.478ms | 262.081ms | not read | not read | not read | 504.70K/s |
| q9 | group by and count distinct | 20.000ms | 195.018ms | 211.100ms | 16.0% | 208.954ms | 242.792ms | 173.810ms | 361.481ms | not read | not read | not read | 473.70K/s |
| q10 | group by, several aggregates | 21.000ms | 240.746ms | 215.831ms | 13.7% | 188.408ms | 217.929ms | 187.392ms | 251.923ms | not read | not read | not read | 463.32K/s |
| q11 | group by a string and count distinct | 9.000ms | 169.700ms | 171.456ms | 10.2% | 165.250ms | 182.793ms | 155.678ms | 196.000ms | not read | not read | not read | 583.23K/s |
| q12 | group by two strings and count distinct | 9.000ms | 147.326ms | 163.406ms | 5.4% | 158.737ms | 167.559ms | 145.562ms | 210.489ms | not read | not read | not read | 611.96K/s |
| q13 | group by a string and top k | 13.000ms | 166.992ms | 162.784ms | 2.1% | 161.831ms | 165.184ms | 151.783ms | 174.399ms | not read | not read | not read | 614.30K/s |
| q14 | group by a string and count distinct | 17.000ms | 194.794ms | 156.799ms | 1.9% | 155.506ms | 158.452ms | 154.107ms | 168.541ms | not read | not read | not read | 637.75K/s |
| q15 | group by two columns and top k | 17.000ms | 160.917ms | 162.074ms | 11.7% | 155.152ms | 174.119ms | 148.765ms | 178.279ms | not read | not read | not read | 616.99K/s |
| q16 | group by, very high card | 16.000ms | 239.254ms | 174.360ms | 16.6% | 155.319ms | 184.322ms | 151.352ms | 253.377ms | not read | not read | not read | 573.51K/s |
| q17 | group by two, very high card | 36.000ms | 179.616ms | 201.369ms | 2.9% | 198.342ms | 204.101ms | 193.160ms | 211.328ms | not read | not read | not read | 496.59K/s |
| q18 | group by two, no ordering | 15.000ms | 164.575ms | 171.935ms | 40.6% | 169.368ms | 239.115ms | 159.695ms | 269.976ms | not read | not read | not read | 581.60K/s |
| q19 | group by with an extract | 44.000ms | 311.797ms | 199.152ms | 9.4% | 190.729ms | 209.387ms | 179.574ms | 230.587ms | not read | not read | not read | 502.12K/s |
| q20 | point lookup | 6.000ms | 149.983ms | 153.585ms | 15.5% | 151.367ms | 175.148ms | 150.822ms | 220.600ms | not read | not read | not read | 651.09K/s |
| q21 | substring scan | 26.000ms | 227.948ms | 169.399ms | 9.6% | 158.493ms | 174.699ms | 150.355ms | 193.160ms | not read | not read | not read | 590.31K/s |
| q22 | substring scan and group by | 11.000ms | 180.103ms | 165.134ms | 12.5% | 148.951ms | 169.547ms | 138.410ms | 202.965ms | not read | not read | not read | 605.56K/s |
| q23 | two substring scans and group by | 22.000ms | 199.918ms | 187.895ms | 14.2% | 169.363ms | 196.135ms | 164.201ms | 207.396ms | not read | not read | not read | 532.20K/s |
| q24 | select star and top k | 122.000ms | 308.255ms | 254.883ms | 4.5% | 246.092ms | 257.478ms | 227.906ms | 275.999ms | not read | not read | not read | 392.33K/s |
| q25 | top k by a date | 8.000ms | 139.876ms | 138.663ms | 3.0% | 137.441ms | 141.596ms | 136.990ms | 170.488ms | not read | not read | not read | 721.16K/s |
| q26 | top k by a string | 7.000ms | 149.207ms | 142.688ms | 2.2% | 141.946ms | 145.097ms | 132.671ms | 148.416ms | not read | not read | not read | 700.82K/s |
| q27 | top k by two columns | 11.000ms | 152.017ms | 168.478ms | 11.5% | 160.390ms | 179.814ms | 141.564ms | 186.220ms | not read | not read | not read | 593.54K/s |
| q28 | group by with a string length | 12.000ms | 161.964ms | 169.445ms | 25.1% | 149.848ms | 192.450ms | 138.700ms | 202.312ms | not read | not read | not read | 590.15K/s |
| q29 | group by a regular expression | 41.000ms | 307.248ms | 185.085ms | 6.9% | 183.296ms | 195.975ms | 170.916ms | 199.890ms | not read | not read | not read | 540.28K/s |
| q30 | ninety sums over one column | 27.000ms | 179.068ms | 193.949ms | 20.7% | 178.006ms | 218.206ms | 177.431ms | 1.759s | not read | not read | not read | 515.59K/s |
| q31 | group by two and several aggregates | 14.000ms | 168.839ms | 186.683ms | 5.2% | 184.378ms | 194.152ms | 151.275ms | 209.802ms | not read | not read | not read | 535.66K/s |
| q32 | group by a high card pair | 14.000ms | 166.291ms | 160.211ms | 17.2% | 157.997ms | 185.585ms | 152.374ms | 223.982ms | not read | not read | not read | 624.16K/s |
| q33 | group by a high card pair, unfiltered | 27.000ms | 175.066ms | 162.220ms | 3.5% | 157.157ms | 162.840ms | 156.280ms | 204.521ms | not read | not read | not read | 616.43K/s |
| q34 | group by a long string | 46.000ms | 204.565ms | 191.711ms | 6.4% | 180.853ms | 193.144ms | 172.785ms | 218.509ms | not read | not read | not read | 521.61K/s |
| q35 | group by a constant and a long string | 78.000ms | 191.616ms | 227.801ms | 27.1% | 206.374ms | 268.153ms | 183.567ms | 331.490ms | not read | not read | not read | 438.97K/s |
| q36 | group by four expressions | 15.000ms | 174.193ms | 157.389ms | 8.7% | 154.667ms | 168.342ms | 146.244ms | 168.727ms | not read | not read | not read | 635.36K/s |
| q37 | date range and group by a URL | 14.000ms | 170.182ms | 150.517ms | 18.5% | 146.063ms | 173.934ms | 145.524ms | 182.266ms | not read | not read | not read | 664.36K/s |
| q38 | date range and group by a title | 18.000ms | 147.789ms | 169.383ms | 7.6% | 168.643ms | 181.470ms | 163.893ms | 220.649ms | not read | not read | not read | 590.37K/s |
| q39 | date range, group by and offset | 12.000ms | 166.963ms | 161.446ms | 4.1% | 160.540ms | 167.102ms | 147.241ms | 213.645ms | not read | not read | not read | 619.39K/s |
| q40 | date range, a case and a wide group by | 15.000ms | 176.341ms | 201.034ms | 6.1% | 191.915ms | 204.144ms | 156.137ms | 217.340ms | not read | not read | not read | 497.42K/s |
| q41 | date range with an IN and a hash | 10.000ms | 158.627ms | 143.462ms | 4.3% | 140.108ms | 146.293ms | 139.760ms | 190.345ms | not read | not read | not read | 697.04K/s |
| q42 | date range and a deep offset | 13.000ms | 163.391ms | 153.203ms | 6.7% | 151.451ms | 161.684ms | 144.790ms | 167.523ms | not read | not read | not read | 652.72K/s |
| q43 | minute buckets over a date range | 12.000ms | 165.798ms | 147.099ms | 3.0% | 146.189ms | 150.608ms | 137.550ms | 161.182ms | not read | not read | not read | 679.80K/s |

clickhouse-server 26.9.1.1225 over 43 of 43 queries. Total 876.000ms by its own clock and 7.622s by ours, 8.562s cold, no reading of CPU, peak not read, 4.91M/s and 740.91 MiB/s.

Running it cost 770% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 1.84x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 99998 rows, one out of every 1000 of the 99997497 in the full file, which is a development loop rather than the suite
- q1 swung by 100.3% of its median, and rule two wants under 10%
- q2 swung by 50.1% of its median, and rule two wants under 10%
- q3 swung by 17.0% of its median, and rule two wants under 10%
- q4 swung by 36.7% of its median, and rule two wants under 10%
- q5 swung by 23.9% of its median, and rule two wants under 10%
- q6 swung by 14.4% of its median, and rule two wants under 10%
- q7 swung by 12.8% of its median, and rule two wants under 10%
- q9 swung by 33.8% of its median, and rule two wants under 10%
- q10 swung by 21.5% of its median, and rule two wants under 10%
- q11 swung by 35.4% of its median, and rule two wants under 10%
- q12 swung by 37.8% of its median, and rule two wants under 10%
- q13 swung by 15.0% of its median, and rule two wants under 10%
- q15 swung by 29.0% of its median, and rule two wants under 10%
- q16 swung by 21.1% of its median, and rule two wants under 10%
- q17 swung by 54.9% of its median, and rule two wants under 10%
- q18 swung by 28.5% of its median, and rule two wants under 10%
- q19 swung by 54.3% of its median, and rule two wants under 10%
- q20 swung by 17.9% of its median, and rule two wants under 10%
- q21 swung by 23.8% of its median, and rule two wants under 10%
- q22 swung by 91.2% of its median, and rule two wants under 10%
- q23 swung by 20.6% of its median, and rule two wants under 10%
- q25 swung by 30.7% of its median, and rule two wants under 10%
- q26 swung by 57.3% of its median, and rule two wants under 10%
- q27 swung by 34.6% of its median, and rule two wants under 10%
- q28 swung by 33.0% of its median, and rule two wants under 10%
- q29 swung by 21.1% of its median, and rule two wants under 10%
- q30 swung by 70.3% of its median, and rule two wants under 10%
- q31 swung by 32.1% of its median, and rule two wants under 10%
- q33 swung by 11.7% of its median, and rule two wants under 10%
- q35 swung by 29.3% of its median, and rule two wants under 10%
- q36 swung by 50.8% of its median, and rule two wants under 10%
- q38 swung by 34.6% of its median, and rule two wants under 10%
- q39 swung by 15.3% of its median, and rule two wants under 10%
- q40 swung by 33.6% of its median, and rule two wants under 10%
- q41 swung by 26.1% of its median, and rule two wants under 10%
- q42 swung by 34.7% of its median, and rule two wants under 10%
- q43 swung by 42.4% of its median, and rule two wants under 10%

These swung wider than reporting rule two allows:

- duckdb swung by 100.3% of its median on q1, and rule two wants under 10%
- clickhouse-local swung by 103.4% of its median on q43, and rule two wants under 10%
- polars swung by 49.0% of its median on q25, and rule two wants under 10%
- clickhouse-server swung by 59.2% of its median on q3, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

These ran every query at about the same speed:

- clickhouse-server ran every query within 1.84x of every other one, and this suite spreads over 6x on an engine it is measuring

A column that flat is not a column about the queries. Read those numbers as an upper bound on the engine and not as a measurement of it.

polars did not run q28, because the SQLContext has no strlen and no character length function.

polars did not run q29, because the SQLContext has no regexp_replace.

polars did not run q36, because the SQLContext rejects the repeated ClientIP output name in the group by.

polars did not run q43, because the SQLContext has no date_trunc.

So the polars column is 39 of 43 queries and its ratio is over the shared ones.

All 4 engines agreed on every answer the data settles, which is 32 of 43 queries, to the last significant digit of a double.

These were answered differently and the data does not say which is right:

- q4: AVG(UserID) over a hundred million bigints near 10^18, where the engines disagree for two reasons. The order the partial sums are added in moves the floating point ones further apart than the one part in a billion this harness calls the same number, and DuckDB is plainly wrong: it sums the bigint column short by a multiple of 2^64 on this file, where its own hugeint and decimal paths, rudb, and adding the column up outside a database all agree. tamnd/rudb-compat#12.
- q16: ORDER BY COUNT(*) DESC LIMIT 10 over user IDs, where the tenth place is a tie between many users with the same count and the engine picks.
- q17: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q18: LIMIT 10 with no ORDER BY at all, so the ten rows are whichever ten the group by handed back first and the query did not ask for any particular ten.
- q19: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q24: ORDER BY EventTime LIMIT 10, and EventTime has one second resolution over a corpus with millions of rows a second, so the tenth row is a tie.
- q31: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q32: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q33: ORDER BY COUNT(*) DESC LIMIT 10 over WatchID and ClientIP unfiltered, where almost every group has a count of one and the ten that come back are arbitrary.
- q36: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q38: ORDER BY PageViews DESC LIMIT 10 over page titles, a tie at the cut as q37. Seen on a hundred thousand row sample, where the tenth and eleventh titles both had thirteen views and the two engines kept a different one. Sampling makes this more likely than the full file does, because the counts are smaller and so more of them collide.

So they are not checked, and a wrong answer from any engine on one of them would go unnoticed here. Every other query in the suite is checked in full.

Hot here means page cache warm and not buffer pool warm for every row but clickhouse-server, which stayed up across the whole suite with its own caches warm. That is the stronger kind of hot, so a ratio against that column is a ratio between two different quantities.

