# clickbench on server3-1m

This is one run of the clickbench suite on server3-1m, over 4 engines and 43 queries, with 5 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

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
rudb-bench run clickbench --rows 1000000 --engines duckdb,clickhouse-local,polars,clickhouse-server --runs 5 --report
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
| duckdb | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 9.362s | 39.710s | 501.01 MiB | its own database file | its own |
| clickhouse-local | 26.9.1.1225 | ran | 3.814s | 13.570s | 230.50 MiB | its own MergeTree parts, as system.parts counts the active ones | its own |
| polars | 1.44.2 in sink mode | ran | 0.000us | 0.000us | 138.65 MiB | the source Parquet, this engine has no storage format of its own | the Parquet |
| clickhouse-server | 26.9.1.1225 | ran | 31.310s | not read | 145.41 MiB | its own MergeTree parts, sorted by hits by (CounterID, EventDate, UserID, EventTime, WatchID), as system.parts counts the active ones | its own |
| duckdb-pinned | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a |
| datafusion | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a |
| rudb | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: polars. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 3.564s | 9.099s | +155% | 9.057s | 23.980s | 2.64 | 235.16 MiB | none | 12.06M/s | 1.63 GiB/s | 1.00x |
| clickhouse-local | 7.555s | 29.958s | +297% | 27.651s | 61.600s | 2.06 | 368.00 MiB | none | 5.69M/s | 789.14 MiB/s | 2.12x |
| polars | 16.052s | 58.546s | +265% | 65.110s | 49.970s | 0.85 | 452.75 MiB | none | 2.43M/s | 336.87 MiB/s | 4.50x |
| clickhouse-server | 2.387s | 9.439s | +295% | 10.064s | not read | not read | not read | not read | 18.01M/s | 2.44 GiB/s | 0.67x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | clickhouse-local | polars | clickhouse-server |
| --- | --- | --- | --- | --- | --- |
| q1 | count | 3.000ms | 14.000ms | 50.015ms | 5.000ms |
| q2 | filtered count | 9.000ms | 15.000ms | 40.426ms | 3.000ms |
| q3 | three aggregates | 13.000ms | 66.000ms | 36.117ms | 19.000ms |
| q4 | average | 12.000ms | 59.000ms | 173.083ms | 25.000ms |
| q5 | count distinct, high card | 43.000ms | 168.000ms | 508.701ms | 88.000ms |
| q6 | count distinct, strings | 38.000ms | 78.000ms | 183.226ms | 71.000ms |
| q7 | min and max of a date | 5.000ms | 35.000ms | 36.019ms | 10.000ms |
| q8 | group by, low card | 34.000ms | 59.000ms | 185.420ms | 18.000ms |
| q9 | group by and count distinct | 73.000ms | 108.000ms | 1.121s | 106.000ms |
| q10 | group by, several aggregates | 76.000ms | 165.000ms | 1.203s | 109.000ms |
| q11 | group by a string and count distinct | 31.000ms | 168.000ms | 295.161ms | 28.000ms |
| q12 | group by two strings and count distinct | 36.000ms | 135.000ms | 162.124ms | 33.000ms |
| q13 | group by a string and top k | 40.000ms | 146.000ms | 199.331ms | 63.000ms |
| q14 | group by a string and count distinct | 48.000ms | 123.000ms | 384.070ms | 62.000ms |
| q15 | group by two columns and top k | 40.000ms | 85.000ms | 257.467ms | 66.000ms |
| q16 | group by, very high card | 42.000ms | 98.000ms | 374.244ms | 64.000ms |
| q17 | group by two, very high card | 78.000ms | 259.000ms | 739.943ms | 133.000ms |
| q18 | group by two, no ordering | 94.000ms | 95.000ms | 652.897ms | 41.000ms |
| q19 | group by with an extract | 139.000ms | 218.000ms | 1.214s | 122.000ms |
| q20 | point lookup | 17.000ms | 48.000ms | 52.212ms | 4.000ms |
| q21 | substring scan | 62.000ms | 70.000ms | 317.281ms | 57.000ms |
| q22 | substring scan and group by | 105.000ms | 127.000ms | 257.402ms | 32.000ms |
| q23 | two substring scans and group by | 130.000ms | 116.000ms | 1.039s | 61.000ms |
| q24 | select star and top k | 196.000ms | 520.000ms | 1.146s | 94.000ms |
| q25 | top k by a date | 29.000ms | 63.000ms | 214.090ms | 21.000ms |
| q26 | top k by a string | 32.000ms | 185.000ms | 138.143ms | 25.000ms |
| q27 | top k by two columns | 66.000ms | 271.000ms | 86.179ms | 22.000ms |
| q28 | group by with a string length | 66.000ms | 135.000ms | no dialect | 27.000ms |
| q29 | group by a regular expression | 495.000ms | 379.000ms | no dialect | 132.000ms |
| q30 | ninety sums over one column | 84.000ms | 65.000ms | 74.612ms | 49.000ms |
| q31 | group by two and several aggregates | 183.000ms | 178.000ms | 179.381ms | 44.000ms |
| q32 | group by a high card pair | 151.000ms | 186.000ms | 227.126ms | 51.000ms |
| q33 | group by a high card pair, unfiltered | 203.000ms | 313.000ms | 737.920ms | 115.000ms |
| q34 | group by a long string | 307.000ms | 393.000ms | 615.461ms | 187.000ms |
| q35 | group by a constant and a long string | 220.000ms | 656.000ms | 1.789s | 229.000ms |
| q36 | group by four expressions | 105.000ms | 401.000ms | no dialect | 55.000ms |
| q37 | date range and group by a URL | 44.000ms | 265.000ms | 531.690ms | 17.000ms |
| q38 | date range and group by a title | 29.000ms | 310.000ms | 301.211ms | 18.000ms |
| q39 | date range, group by and offset | 38.000ms | 192.000ms | 120.015ms | 16.000ms |
| q40 | date range, a case and a wide group by | 39.000ms | 188.000ms | 235.147ms | 26.000ms |
| q41 | date range with an IN and a hash | 34.000ms | 120.000ms | 103.402ms | 13.000ms |
| q42 | date range and a deep offset | 49.000ms | 124.000ms | 70.340ms | 13.000ms |
| q43 | minute buckets over a date range | 26.000ms | 156.000ms | no dialect | 13.000ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 3.000ms | 101.844ms | 95.626ms | 7.9% | 90.600ms | 98.182ms | 90.544ms | 104.561ms | 120.000ms | 35.29 MiB | 228.00 KiB | 10.46M/s |
| q2 | filtered count | 9.000ms | 93.097ms | 96.600ms | 2.5% | 94.611ms | 97.011ms | 93.762ms | 97.469ms | 120.000ms | 38.41 MiB | 128.00 KiB | 10.35M/s |
| q3 | three aggregates | 13.000ms | 113.427ms | 97.018ms | 0.9% | 96.254ms | 97.120ms | 93.785ms | 108.622ms | 130.000ms | 41.53 MiB | 320.00 KiB | 10.31M/s |
| q4 | average | 12.000ms | 109.835ms | 99.565ms | 1.1% | 98.573ms | 99.659ms | 98.435ms | 100.103ms | 130.000ms | 45.16 MiB | none | 10.04M/s |
| q5 | count distinct, high card | 43.000ms | 142.114ms | 133.351ms | 12.8% | 125.500ms | 142.550ms | 123.189ms | 145.071ms | 310.000ms | 80.79 MiB | 2.17 MiB | 7.50M/s |
| q6 | count distinct, strings | 38.000ms | 132.295ms | 135.384ms | 12.2% | 124.368ms | 140.856ms | 122.467ms | 155.020ms | 290.000ms | 72.66 MiB | 128.00 KiB | 7.39M/s |
| q7 | min and max of a date | 5.000ms | 107.131ms | 129.135ms | 16.4% | 127.315ms | 148.462ms | 111.942ms | 159.850ms | 190.000ms | 35.54 MiB | 256.00 KiB | 7.74M/s |
| q8 | group by, low card | 34.000ms | 168.707ms | 127.278ms | 2.7% | 126.780ms | 130.236ms | 124.081ms | 156.933ms | 200.000ms | 41.28 MiB | 212.00 KiB | 7.86M/s |
| q9 | group by and count distinct | 73.000ms | 169.006ms | 182.071ms | 5.6% | 175.408ms | 185.639ms | 174.389ms | 367.618ms | 510.000ms | 89.03 MiB | none | 5.49M/s |
| q10 | group by, several aggregates | 76.000ms | 228.249ms | 181.965ms | 10.3% | 165.949ms | 184.735ms | 161.137ms | 192.880ms | 470.000ms | 101.02 MiB | 204.00 KiB | 5.50M/s |
| q11 | group by a string and count distinct | 31.000ms | 126.688ms | 129.709ms | 16.9% | 123.060ms | 145.040ms | 111.153ms | 157.594ms | 240.000ms | 63.65 MiB | none | 7.71M/s |
| q12 | group by two strings and count distinct | 36.000ms | 120.084ms | 121.887ms | 45.8% | 120.796ms | 176.581ms | 118.625ms | 204.572ms | 260.000ms | 66.40 MiB | 4.00 KiB | 8.20M/s |
| q13 | group by a string and top k | 40.000ms | 152.424ms | 153.108ms | 13.2% | 134.139ms | 154.286ms | 130.470ms | 154.724ms | 330.000ms | 78.03 MiB | 128.00 KiB | 6.53M/s |
| q14 | group by a string and count distinct | 48.000ms | 164.323ms | 144.751ms | 11.3% | 141.200ms | 157.618ms | 137.954ms | 200.528ms | 370.000ms | 101.78 MiB | none | 6.91M/s |
| q15 | group by two columns and top k | 40.000ms | 127.812ms | 133.905ms | 25.5% | 124.220ms | 158.301ms | 123.515ms | 160.912ms | 310.000ms | 83.17 MiB | none | 7.47M/s |
| q16 | group by, very high card | 42.000ms | 131.645ms | 137.607ms | 3.3% | 136.468ms | 140.964ms | 135.109ms | 158.073ms | 350.000ms | 92.28 MiB | none | 7.27M/s |
| q17 | group by two, very high card | 78.000ms | 167.794ms | 187.154ms | 5.5% | 179.994ms | 190.201ms | 176.659ms | 194.340ms | 620.000ms | 156.41 MiB | none | 5.34M/s |
| q18 | group by two, no ordering | 94.000ms | 188.454ms | 216.450ms | 9.8% | 203.303ms | 224.435ms | 192.793ms | 304.438ms | 620.000ms | 143.65 MiB | none | 4.62M/s |
| q19 | group by with an extract | 139.000ms | 216.851ms | 255.216ms | 15.4% | 238.035ms | 277.379ms | 223.911ms | 319.683ms | 790.000ms | 166.78 MiB | 300.00 KiB | 3.92M/s |
| q20 | point lookup | 17.000ms | 105.329ms | 111.751ms | 16.9% | 104.562ms | 123.442ms | 103.347ms | 128.615ms | 170.000ms | 44.04 MiB | none | 8.95M/s |
| q21 | substring scan | 62.000ms | 166.515ms | 154.316ms | 6.6% | 152.402ms | 162.595ms | 133.746ms | 172.257ms | 420.000ms | 96.16 MiB | none | 6.48M/s |
| q22 | substring scan and group by | 105.000ms | 176.893ms | 215.408ms | 26.0% | 175.042ms | 230.985ms | 166.507ms | 243.027ms | 660.000ms | 114.53 MiB | none | 4.64M/s |
| q23 | two substring scans and group by | 130.000ms | 259.695ms | 227.145ms | 24.9% | 182.567ms | 239.160ms | 180.296ms | 308.278ms | 590.000ms | 145.40 MiB | none | 4.40M/s |
| q24 | select star and top k | 196.000ms | 391.535ms | 320.251ms | 12.8% | 306.825ms | 347.872ms | 293.217ms | 390.045ms | 850.000ms | 209.66 MiB | 588.00 KiB | 3.12M/s |
| q25 | top k by a date | 29.000ms | 119.852ms | 126.563ms | 4.9% | 121.363ms | 127.520ms | 118.867ms | 136.430ms | 210.000ms | 57.40 MiB | none | 7.90M/s |
| q26 | top k by a string | 32.000ms | 145.908ms | 123.413ms | 6.6% | 117.486ms | 125.623ms | 114.550ms | 154.410ms | 230.000ms | 51.78 MiB | none | 8.10M/s |
| q27 | top k by two columns | 66.000ms | 156.921ms | 188.243ms | 30.4% | 146.391ms | 203.569ms | 130.088ms | 221.403ms | 440.000ms | 57.65 MiB | none | 5.31M/s |
| q28 | group by with a string length | 66.000ms | 208.853ms | 173.795ms | 7.9% | 162.562ms | 176.338ms | 156.242ms | 176.339ms | 450.000ms | 108.41 MiB | 128.00 KiB | 5.75M/s |
| q29 | group by a regular expression | 495.000ms | 727.321ms | 635.931ms | 6.2% | 609.551ms | 648.965ms | 517.556ms | 1.048s | 3.000s | 168.13 MiB | 44.00 KiB | 1.57M/s |
| q30 | ninety sums over one column | 84.000ms | 262.572ms | 279.018ms | 41.7% | 200.506ms | 316.768ms | 199.796ms | 367.312ms | 440.000ms | 51.29 MiB | 204.00 KiB | 3.58M/s |
| q31 | group by two and several aggregates | 183.000ms | 215.524ms | 467.616ms | 75.3% | 396.875ms | 748.907ms | 198.833ms | 1.024s | 1.180s | 87.77 MiB | none | 2.14M/s |
| q32 | group by a high card pair | 151.000ms | 407.625ms | 434.256ms | 28.7% | 332.379ms | 456.981ms | 259.553ms | 839.384ms | 950.000ms | 94.90 MiB | none | 2.30M/s |
| q33 | group by a high card pair, unfiltered | 203.000ms | 243.562ms | 443.429ms | 28.8% | 357.614ms | 485.501ms | 318.561ms | 570.797ms | 1.590s | 148.52 MiB | none | 2.26M/s |
| q34 | group by a long string | 307.000ms | 543.476ms | 489.705ms | 41.8% | 432.129ms | 637.009ms | 383.949ms | 950.801ms | 1.970s | 230.78 MiB | none | 2.04M/s |
| q35 | group by a constant and a long string | 220.000ms | 516.363ms | 479.053ms | 57.8% | 317.845ms | 594.617ms | 316.916ms | 668.733ms | 1.490s | 235.16 MiB | none | 2.09M/s |
| q36 | group by four expressions | 105.000ms | 384.605ms | 267.638ms | 64.1% | 222.581ms | 394.157ms | 199.266ms | 411.360ms | 820.000ms | 84.66 MiB | 260.00 KiB | 3.74M/s |
| q37 | date range and group by a URL | 44.000ms | 146.261ms | 211.872ms | 12.1% | 195.748ms | 221.353ms | 166.417ms | 314.575ms | 390.000ms | 45.40 MiB | none | 4.72M/s |
| q38 | date range and group by a title | 29.000ms | 139.725ms | 140.150ms | 29.0% | 130.808ms | 171.503ms | 126.089ms | 300.432ms | 270.000ms | 43.52 MiB | none | 7.14M/s |
| q39 | date range, group by and offset | 38.000ms | 172.557ms | 184.605ms | 10.5% | 173.991ms | 193.372ms | 143.571ms | 276.491ms | 310.000ms | 43.03 MiB | none | 5.42M/s |
| q40 | date range, a case and a wide group by | 39.000ms | 123.485ms | 149.603ms | 15.7% | 146.750ms | 170.193ms | 126.815ms | 330.971ms | 270.000ms | 53.03 MiB | none | 6.68M/s |
| q41 | date range with an IN and a hash | 34.000ms | 296.573ms | 179.392ms | 12.7% | 157.366ms | 180.220ms | 139.315ms | 204.741ms | 290.000ms | 43.90 MiB | 120.00 KiB | 5.57M/s |
| q42 | date range and a deep offset | 49.000ms | 244.787ms | 193.446ms | 18.9% | 182.398ms | 218.877ms | 168.660ms | 279.124ms | 370.000ms | 43.78 MiB | 128.00 KiB | 5.17M/s |
| q43 | minute buckets over a date range | 26.000ms | 138.926ms | 145.004ms | 28.1% | 143.277ms | 184.075ms | 143.007ms | 208.851ms | 260.000ms | 41.65 MiB | 308.00 KiB | 6.90M/s |

duckdb v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 3.564s by its own clock and 9.099s by ours, 9.057s cold, 23.980s of CPU, peak 235.16 MiB, 12.06M/s and 1.63 GiB/s.

Running it cost 155% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 6.65x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 14.000ms | 363.689ms | 607.227ms | 26.5% | 510.385ms | 671.048ms | 305.206ms | 707.855ms | 2.120s | 202.75 MiB | none | 1.65M/s |
| q2 | filtered count | 15.000ms | 467.190ms | 464.595ms | 3.1% | 458.477ms | 472.796ms | 312.869ms | 610.665ms | 1.240s | 203.59 MiB | none | 2.15M/s |
| q3 | three aggregates | 66.000ms | 394.815ms | 365.704ms | 9.9% | 359.751ms | 395.914ms | 298.985ms | 401.297ms | 960.000ms | 211.00 MiB | none | 2.73M/s |
| q4 | average | 59.000ms | 291.134ms | 441.968ms | 1.7% | 440.683ms | 448.239ms | 416.772ms | 638.705ms | 1.430s | 212.00 MiB | none | 2.26M/s |
| q5 | count distinct, high card | 168.000ms | 602.069ms | 781.392ms | 30.1% | 700.535ms | 936.018ms | 544.191ms | 1.358s | 3.170s | 309.82 MiB | none | 1.28M/s |
| q6 | count distinct, strings | 78.000ms | 490.709ms | 423.982ms | 15.7% | 417.273ms | 484.015ms | 360.670ms | 485.048ms | 1.150s | 241.25 MiB | none | 2.36M/s |
| q7 | min and max of a date | 35.000ms | 254.931ms | 333.433ms | 2.5% | 329.508ms | 337.734ms | 301.193ms | 542.380ms | 740.000ms | 205.75 MiB | none | 3.00M/s |
| q8 | group by, low card | 59.000ms | 299.176ms | 343.053ms | 2.1% | 340.122ms | 347.299ms | 294.975ms | 388.837ms | 660.000ms | 211.62 MiB | none | 2.91M/s |
| q9 | group by and count distinct | 108.000ms | 511.732ms | 382.231ms | 23.3% | 355.141ms | 444.092ms | 337.116ms | 564.115ms | 820.000ms | 271.56 MiB | none | 2.62M/s |
| q10 | group by, several aggregates | 165.000ms | 643.347ms | 475.717ms | 13.3% | 458.094ms | 521.562ms | 446.774ms | 566.782ms | 1.290s | 271.75 MiB | none | 2.10M/s |
| q11 | group by a string and count distinct | 168.000ms | 392.536ms | 803.508ms | 60.1% | 766.031ms | 1.249s | 325.522ms | 1.649s | 3.600s | 225.07 MiB | none | 1.24M/s |
| q12 | group by two strings and count distinct | 135.000ms | 1.074s | 844.742ms | 59.0% | 615.737ms | 1.114s | 548.584ms | 1.188s | 3.720s | 227.88 MiB | none | 1.18M/s |
| q13 | group by a string and top k | 146.000ms | 865.433ms | 1.107s | 42.1% | 742.065ms | 1.208s | 690.627ms | 1.379s | 5.120s | 252.93 MiB | none | 903.43K/s |
| q14 | group by a string and count distinct | 123.000ms | 639.924ms | 558.121ms | 8.8% | 510.186ms | 559.066ms | 509.591ms | 686.251ms | 1.820s | 260.50 MiB | none | 1.79M/s |
| q15 | group by two columns and top k | 85.000ms | 418.403ms | 356.392ms | 4.3% | 354.754ms | 370.074ms | 319.622ms | 398.503ms | 790.000ms | 259.87 MiB | 36.00 KiB | 2.81M/s |
| q16 | group by, very high card | 98.000ms | 400.481ms | 434.469ms | 39.1% | 371.524ms | 541.191ms | 359.131ms | 554.870ms | 980.000ms | 267.42 MiB | none | 2.30M/s |
| q17 | group by two, very high card | 259.000ms | 487.423ms | 688.897ms | 10.5% | 642.568ms | 714.713ms | 562.116ms | 756.318ms | 1.600s | 336.00 MiB | none | 1.45M/s |
| q18 | group by two, no ordering | 95.000ms | 390.924ms | 523.666ms | 15.0% | 471.591ms | 550.115ms | 374.616ms | 563.944ms | 1.130s | 246.12 MiB | none | 1.91M/s |
| q19 | group by with an extract | 218.000ms | 506.953ms | 548.532ms | 5.8% | 524.383ms | 556.002ms | 522.969ms | 590.818ms | 1.490s | 337.32 MiB | none | 1.82M/s |
| q20 | point lookup | 48.000ms | 458.357ms | 346.066ms | 9.5% | 335.909ms | 368.815ms | 328.342ms | 479.985ms | 660.000ms | 212.34 MiB | none | 2.89M/s |
| q21 | substring scan | 70.000ms | 331.486ms | 347.077ms | 3.9% | 344.659ms | 358.297ms | 342.142ms | 362.296ms | 680.000ms | 242.74 MiB | none | 2.88M/s |
| q22 | substring scan and group by | 127.000ms | 451.432ms | 443.845ms | 56.5% | 404.183ms | 654.814ms | 382.508ms | 790.365ms | 950.000ms | 254.80 MiB | none | 2.25M/s |
| q23 | two substring scans and group by | 116.000ms | 374.164ms | 388.950ms | 9.0% | 373.112ms | 407.942ms | 354.978ms | 416.740ms | 950.000ms | 265.89 MiB | none | 2.57M/s |
| q24 | select star and top k | 520.000ms | 837.066ms | 1.096s | 5.2% | 1.080s | 1.137s | 978.318ms | 1.300s | 1.630s | 339.12 MiB | none | 912.19K/s |
| q25 | top k by a date | 63.000ms | 379.631ms | 465.465ms | 27.5% | 349.784ms | 477.727ms | 295.057ms | 519.973ms | 590.000ms | 226.88 MiB | none | 2.15M/s |
| q26 | top k by a string | 185.000ms | 341.381ms | 976.985ms | 52.7% | 618.923ms | 1.134s | 551.821ms | 1.195s | 590.000ms | 227.00 MiB | none | 1.02M/s |
| q27 | top k by two columns | 271.000ms | 997.321ms | 1.197s | 55.4% | 724.333ms | 1.388s | 504.697ms | 1.469s | 920.000ms | 224.86 MiB | none | 835.20K/s |
| q28 | group by with a string length | 135.000ms | 652.476ms | 584.231ms | 28.8% | 537.654ms | 705.679ms | 528.151ms | 948.977ms | 1.690s | 222.62 MiB | none | 1.71M/s |
| q29 | group by a regular expression | 379.000ms | 501.173ms | 895.518ms | 7.9% | 862.660ms | 933.754ms | 703.131ms | 992.161ms | 1.750s | 324.38 MiB | none | 1.12M/s |
| q30 | ninety sums over one column | 65.000ms | 448.930ms | 434.744ms | 14.7% | 424.518ms | 488.402ms | 381.710ms | 673.508ms | 770.000ms | 212.27 MiB | none | 2.30M/s |
| q31 | group by two and several aggregates | 178.000ms | 431.961ms | 625.909ms | 30.4% | 506.367ms | 696.771ms | 479.414ms | 751.378ms | 770.000ms | 241.88 MiB | none | 1.60M/s |
| q32 | group by a high card pair | 186.000ms | 761.564ms | 751.182ms | 26.0% | 561.560ms | 757.232ms | 559.515ms | 835.128ms | 950.000ms | 251.50 MiB | none | 1.33M/s |
| q33 | group by a high card pair, unfiltered | 313.000ms | 1.035s | 1.068s | 40.6% | 863.403ms | 1.297s | 728.728ms | 1.816s | 1.370s | 304.50 MiB | none | 936.38K/s |
| q34 | group by a long string | 393.000ms | 897.479ms | 1.028s | 19.4% | 899.408ms | 1.099s | 881.331ms | 1.284s | 2.810s | 366.83 MiB | none | 972.30K/s |
| q35 | group by a constant and a long string | 656.000ms | 885.315ms | 1.255s | 30.0% | 1.231s | 1.606s | 1.094s | 1.636s | 1.950s | 368.00 MiB | none | 796.94K/s |
| q36 | group by four expressions | 401.000ms | 900.665ms | 1.183s | 18.4% | 1.053s | 1.270s | 883.170ms | 1.493s | 1.150s | 261.57 MiB | none | 845.63K/s |
| q37 | date range and group by a URL | 265.000ms | 1.590s | 1.423s | 19.4% | 1.165s | 1.442s | 1.095s | 1.682s | 910.000ms | 225.88 MiB | 128.00 KiB | 702.72K/s |
| q38 | date range and group by a title | 310.000ms | 2.283s | 1.777s | 51.1% | 1.066s | 1.973s | 804.399ms | 2.621s | 870.000ms | 224.62 MiB | none | 562.87K/s |
| q39 | date range, group by and offset | 192.000ms | 1.007s | 810.336ms | 17.8% | 762.849ms | 907.331ms | 580.317ms | 1.132s | 2.600s | 226.88 MiB | none | 1.23M/s |
| q40 | date range, a case and a wide group by | 188.000ms | 753.894ms | 584.231ms | 10.3% | 581.625ms | 641.706ms | 525.153ms | 719.220ms | 950.000ms | 234.25 MiB | none | 1.71M/s |
| q41 | date range with an IN and a hash | 120.000ms | 477.582ms | 490.683ms | 43.0% | 479.439ms | 690.277ms | 478.329ms | 780.901ms | 770.000ms | 223.62 MiB | none | 2.04M/s |
| q42 | date range and a deep offset | 124.000ms | 695.399ms | 632.275ms | 31.7% | 612.607ms | 812.762ms | 580.352ms | 861.141ms | 740.000ms | 221.50 MiB | none | 1.58M/s |
| q43 | minute buckets over a date range | 156.000ms | 663.628ms | 669.742ms | 7.3% | 635.054ms | 683.726ms | 607.794ms | 700.320ms | 750.000ms | 222.00 MiB | none | 1.49M/s |

clickhouse-local 26.9.1.1225 over 43 of 43 queries. Total 7.555s by its own clock and 29.958s by ours, 27.651s cold, 61.600s of CPU, peak 368.00 MiB, 5.69M/s and 789.14 MiB/s.

Running it cost 297% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 5.33x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## polars in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 50.015ms | 753.218ms | 533.422ms | 20.1% | 507.427ms | 614.722ms | 501.304ms | 716.528ms | 560.000ms | 57.86 MiB | none | 1.87M/s |
| q2 | filtered count | 40.426ms | 692.016ms | 881.512ms | 14.1% | 797.206ms | 921.193ms | 699.213ms | 1.409s | 770.000ms | 62.19 MiB | none | 1.13M/s |
| q3 | three aggregates | 36.117ms | 783.049ms | 733.690ms | 81.5% | 717.433ms | 1.315s | 662.191ms | 2.018s | 700.000ms | 66.90 MiB | none | 1.36M/s |
| q4 | average | 173.083ms | 1.700s | 1.999s | 47.8% | 1.442s | 2.398s | 1.146s | 2.410s | 1.010s | 80.87 MiB | none | 500.29K/s |
| q5 | count distinct, high card | 508.701ms | 2.169s | 2.084s | 6.1% | 2.008s | 2.136s | 1.701s | 3.195s | 1.270s | 118.72 MiB | 480.00 KiB | 479.85K/s |
| q6 | count distinct, strings | 183.226ms | 2.311s | 876.111ms | 11.0% | 840.556ms | 936.594ms | 792.055ms | 1.105s | 1.030s | 116.82 MiB | none | 1.14M/s |
| q7 | min and max of a date | 36.019ms | 688.471ms | 735.369ms | 24.7% | 634.984ms | 816.930ms | 590.775ms | 991.744ms | 680.000ms | 66.23 MiB | 512.00 KiB | 1.36M/s |
| q8 | group by, low card | 185.420ms | 2.105s | 2.511s | 22.6% | 2.279s | 2.846s | 1.827s | 3.387s | 760.000ms | 65.93 MiB | 16.00 KiB | 398.26K/s |
| q9 | group by and count distinct | 1.121s | 4.689s | 3.301s | 6.5% | 3.096s | 3.311s | 2.652s | 3.752s | 1.760s | 201.38 MiB | 392.00 KiB | 302.96K/s |
| q10 | group by, several aggregates | 1.203s | 4.067s | 2.873s | 40.5% | 2.332s | 3.497s | 1.966s | 3.914s | 2.000s | 204.19 MiB | none | 348.09K/s |
| q11 | group by a string and count distinct | 295.161ms | 2.384s | 2.019s | 42.6% | 1.527s | 2.387s | 1.410s | 2.851s | 890.000ms | 84.70 MiB | none | 495.21K/s |
| q12 | group by two strings and count distinct | 162.124ms | 1.469s | 1.392s | 18.0% | 1.258s | 1.508s | 1.167s | 1.768s | 920.000ms | 86.28 MiB | none | 718.30K/s |
| q13 | group by a string and top k | 199.331ms | 1.034s | 1.066s | 11.3% | 958.134ms | 1.078s | 885.329ms | 1.212s | 1.010s | 105.27 MiB | none | 938.27K/s |
| q14 | group by a string and count distinct | 384.070ms | 1.697s | 1.294s | 17.5% | 1.077s | 1.304s | 1.011s | 1.410s | 1.270s | 142.14 MiB | none | 772.59K/s |
| q15 | group by two columns and top k | 257.467ms | 819.148ms | 998.419ms | 3.1% | 994.372ms | 1.025s | 988.556ms | 1.085s | 1.050s | 115.03 MiB | none | 1.00M/s |
| q16 | group by, very high card | 374.244ms | 1.477s | 1.166s | 12.2% | 1.141s | 1.283s | 965.756ms | 1.486s | 1.190s | 140.16 MiB | none | 857.24K/s |
| q17 | group by two, very high card | 739.943ms | 1.758s | 1.728s | 19.6% | 1.495s | 1.834s | 1.482s | 2.135s | 2.230s | 241.33 MiB | none | 578.83K/s |
| q18 | group by two, no ordering | 652.897ms | 2.129s | 1.629s | 12.3% | 1.536s | 1.735s | 1.165s | 1.958s | 1.860s | 243.05 MiB | none | 613.69K/s |
| q19 | group by with an extract | 1.214s | 2.576s | 2.969s | 34.2% | 2.821s | 3.837s | 2.806s | 4.100s | 2.110s | 257.94 MiB | 436.00 KiB | 336.86K/s |
| q20 | point lookup | 52.212ms | 1.790s | 1.787s | 38.3% | 1.497s | 2.182s | 1.051s | 2.996s | 780.000ms | 73.16 MiB | 128.00 KiB | 559.49K/s |
| q21 | substring scan | 317.281ms | 2.243s | 991.726ms | 4.4% | 956.069ms | 999.923ms | 930.109ms | 1.424s | 1.440s | 175.48 MiB | 476.00 KiB | 1.01M/s |
| q22 | substring scan and group by | 257.402ms | 1.031s | 719.195ms | 1.7% | 716.423ms | 728.609ms | 670.945ms | 973.020ms | 1.400s | 192.61 MiB | 320.00 KiB | 1.39M/s |
| q23 | two substring scans and group by | 1.039s | 1.609s | 2.093s | 33.3% | 1.798s | 2.495s | 1.704s | 2.787s | 2.830s | 359.93 MiB | none | 477.67K/s |
| q24 | select star and top k | 1.146s | 2.509s | 2.404s | 3.9% | 2.325s | 2.417s | 1.876s | 2.470s | 3.320s | 452.75 MiB | 128.00 KiB | 415.99K/s |
| q25 | top k by a date | 214.090ms | 1.504s | 1.590s | 12.1% | 1.425s | 1.618s | 1.405s | 1.668s | 910.000ms | 92.18 MiB | none | 628.75K/s |
| q26 | top k by a string | 138.143ms | 1.093s | 1.110s | 12.0% | 1.043s | 1.176s | 1.023s | 1.328s | 860.000ms | 80.71 MiB | none | 900.48K/s |
| q27 | top k by two columns | 86.179ms | 811.167ms | 527.007ms | 6.6% | 526.674ms | 561.248ms | 460.926ms | 577.711ms | 590.000ms | 93.38 MiB | none | 1.90M/s |
| q30 | ninety sums over one column | 74.612ms | 481.208ms | 549.225ms | 17.6% | 506.761ms | 603.573ms | 495.659ms | 635.415ms | 560.000ms | 68.55 MiB | none | 1.82M/s |
| q31 | group by two and several aggregates | 179.381ms | 551.491ms | 818.629ms | 23.0% | 810.483ms | 999.073ms | 594.880ms | 1.204s | 940.000ms | 99.73 MiB | none | 1.22M/s |
| q32 | group by a high card pair | 227.126ms | 916.522ms | 1.025s | 9.0% | 959.885ms | 1.052s | 776.263ms | 1.089s | 900.000ms | 108.61 MiB | none | 975.29K/s |
| q33 | group by a high card pair, unfiltered | 737.920ms | 2.273s | 2.009s | 20.5% | 1.992s | 2.403s | 1.792s | 4.094s | 1.890s | 258.57 MiB | none | 497.80K/s |
| q34 | group by a long string | 615.461ms | 1.693s | 1.424s | 24.4% | 1.373s | 1.721s | 1.275s | 4.048s | 2.470s | 442.52 MiB | none | 702.19K/s |
| q35 | group by a constant and a long string | 1.789s | 3.513s | 3.218s | 11.6% | 3.201s | 3.574s | 3.091s | 4.087s | 2.750s | 438.35 MiB | none | 310.71K/s |
| q37 | date range and group by a URL | 531.690ms | 3.423s | 2.436s | 41.1% | 1.779s | 2.780s | 1.269s | 4.087s | 1.070s | 137.48 MiB | none | 410.45K/s |
| q38 | date range and group by a title | 301.211ms | 1.152s | 1.419s | 20.3% | 1.189s | 1.477s | 1.033s | 1.485s | 1.000s | 129.07 MiB | none | 704.75K/s |
| q39 | date range, group by and offset | 120.015ms | 941.726ms | 729.295ms | 31.1% | 693.459ms | 920.561ms | 669.689ms | 952.283ms | 740.000ms | 98.52 MiB | none | 1.37M/s |
| q40 | date range, a case and a wide group by | 235.147ms | 686.774ms | 1.162s | 38.7% | 982.563ms | 1.432s | 970.379ms | 1.714s | 970.000ms | 127.82 MiB | none | 860.38K/s |
| q41 | date range with an IN and a hash | 103.402ms | 900.216ms | 937.106ms | 7.8% | 922.049ms | 994.708ms | 881.095ms | 1.222s | 720.000ms | 75.54 MiB | none | 1.07M/s |
| q42 | date range and a deep offset | 70.340ms | 684.548ms | 805.593ms | 8.1% | 775.167ms | 840.095ms | 699.006ms | 1.080s | 760.000ms | 72.45 MiB | none | 1.24M/s |

polars 1.44.2 in sink mode over 39 of 43 queries. Total 16.052s by its own clock and 58.546s by ours, 65.110s cold, 49.970s of CPU, peak 452.75 MiB, 2.43M/s and 336.87 MiB/s.

Running it cost 265% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 6.26x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-server in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 5.000ms | 588.640ms | 404.637ms | 34.8% | 402.472ms | 543.111ms | 272.322ms | 686.332ms | not read | not read | not read | 2.47M/s |
| q2 | filtered count | 3.000ms | 219.406ms | 183.341ms | 8.5% | 177.757ms | 193.362ms | 155.463ms | 198.125ms | not read | not read | not read | 5.45M/s |
| q3 | three aggregates | 19.000ms | 191.984ms | 203.059ms | 22.3% | 184.259ms | 229.457ms | 173.865ms | 250.684ms | not read | not read | not read | 4.92M/s |
| q4 | average | 25.000ms | 186.650ms | 214.884ms | 16.1% | 181.461ms | 216.134ms | 171.986ms | 222.406ms | not read | not read | not read | 4.65M/s |
| q5 | count distinct, high card | 88.000ms | 259.797ms | 265.829ms | 6.0% | 255.865ms | 271.808ms | 208.713ms | 284.957ms | not read | not read | not read | 3.76M/s |
| q6 | count distinct, strings | 71.000ms | 190.984ms | 234.016ms | 17.3% | 219.791ms | 260.287ms | 210.278ms | 283.917ms | not read | not read | not read | 4.27M/s |
| q7 | min and max of a date | 10.000ms | 255.732ms | 181.229ms | 9.8% | 163.450ms | 181.250ms | 159.379ms | 239.573ms | not read | not read | not read | 5.52M/s |
| q8 | group by, low card | 18.000ms | 160.038ms | 193.173ms | 4.6% | 191.152ms | 200.109ms | 160.478ms | 200.673ms | not read | not read | not read | 5.18M/s |
| q9 | group by and count distinct | 106.000ms | 304.520ms | 263.316ms | 12.5% | 260.621ms | 293.447ms | 256.255ms | 320.346ms | not read | not read | not read | 3.80M/s |
| q10 | group by, several aggregates | 109.000ms | 271.433ms | 325.266ms | 35.8% | 272.651ms | 389.139ms | 232.455ms | 395.607ms | not read | not read | not read | 3.07M/s |
| q11 | group by a string and count distinct | 28.000ms | 234.666ms | 186.800ms | 16.3% | 173.861ms | 204.242ms | 167.946ms | 208.807ms | not read | not read | not read | 5.35M/s |
| q12 | group by two strings and count distinct | 33.000ms | 192.693ms | 194.310ms | 20.2% | 182.560ms | 221.724ms | 181.483ms | 229.892ms | not read | not read | not read | 5.15M/s |
| q13 | group by a string and top k | 63.000ms | 187.733ms | 219.357ms | 11.6% | 212.256ms | 237.785ms | 203.522ms | 244.938ms | not read | not read | not read | 4.56M/s |
| q14 | group by a string and count distinct | 62.000ms | 190.447ms | 212.209ms | 7.6% | 210.300ms | 226.529ms | 203.797ms | 302.719ms | not read | not read | not read | 4.71M/s |
| q15 | group by two columns and top k | 66.000ms | 206.429ms | 224.901ms | 7.9% | 221.855ms | 239.515ms | 193.156ms | 264.505ms | not read | not read | not read | 4.45M/s |
| q16 | group by, very high card | 64.000ms | 195.649ms | 233.216ms | 13.8% | 209.085ms | 241.380ms | 205.074ms | 342.282ms | not read | not read | not read | 4.29M/s |
| q17 | group by two, very high card | 133.000ms | 269.242ms | 264.398ms | 8.5% | 249.022ms | 271.403ms | 237.710ms | 273.554ms | not read | not read | not read | 3.78M/s |
| q18 | group by two, no ordering | 41.000ms | 266.253ms | 203.689ms | 4.6% | 196.080ms | 205.506ms | 173.770ms | 260.182ms | not read | not read | not read | 4.91M/s |
| q19 | group by with an extract | 122.000ms | 279.095ms | 301.799ms | 29.1% | 253.654ms | 341.608ms | 241.092ms | 355.797ms | not read | not read | not read | 3.31M/s |
| q20 | point lookup | 4.000ms | 162.964ms | 132.349ms | 7.8% | 129.560ms | 139.887ms | 127.306ms | 152.068ms | not read | not read | not read | 7.56M/s |
| q21 | substring scan | 57.000ms | 272.002ms | 216.140ms | 5.0% | 206.631ms | 217.469ms | 199.070ms | 285.152ms | not read | not read | not read | 4.63M/s |
| q22 | substring scan and group by | 32.000ms | 278.252ms | 174.481ms | 7.1% | 172.550ms | 185.023ms | 162.349ms | 213.117ms | not read | not read | not read | 5.73M/s |
| q23 | two substring scans and group by | 61.000ms | 229.439ms | 230.423ms | 22.4% | 193.226ms | 244.905ms | 191.379ms | 311.434ms | not read | not read | not read | 4.34M/s |
| q24 | select star and top k | 94.000ms | 253.333ms | 215.651ms | 12.6% | 212.439ms | 239.662ms | 207.620ms | 408.942ms | not read | not read | not read | 4.64M/s |
| q25 | top k by a date | 21.000ms | 139.398ms | 170.792ms | 8.0% | 158.583ms | 172.285ms | 149.931ms | 206.510ms | not read | not read | not read | 5.85M/s |
| q26 | top k by a string | 25.000ms | 176.350ms | 182.809ms | 25.8% | 166.678ms | 213.881ms | 164.943ms | 262.918ms | not read | not read | not read | 5.47M/s |
| q27 | top k by two columns | 22.000ms | 169.983ms | 175.866ms | 5.8% | 169.792ms | 179.945ms | 164.079ms | 282.111ms | not read | not read | not read | 5.69M/s |
| q28 | group by with a string length | 27.000ms | 266.541ms | 183.656ms | 26.7% | 153.370ms | 202.456ms | 152.424ms | 223.171ms | not read | not read | not read | 5.44M/s |
| q29 | group by a regular expression | 132.000ms | 319.742ms | 256.074ms | 3.6% | 253.521ms | 262.702ms | 237.575ms | 282.762ms | not read | not read | not read | 3.91M/s |
| q30 | ninety sums over one column | 49.000ms | 186.007ms | 258.591ms | 52.7% | 172.970ms | 309.198ms | 170.048ms | 1.613s | not read | not read | not read | 3.87M/s |
| q31 | group by two and several aggregates | 44.000ms | 212.107ms | 193.969ms | 12.8% | 187.155ms | 211.904ms | 186.349ms | 213.425ms | not read | not read | not read | 5.16M/s |
| q32 | group by a high card pair | 51.000ms | 216.148ms | 204.826ms | 17.2% | 203.486ms | 238.641ms | 201.361ms | 376.374ms | not read | not read | not read | 4.88M/s |
| q33 | group by a high card pair, unfiltered | 115.000ms | 306.683ms | 299.090ms | 34.8% | 248.200ms | 352.290ms | 235.078ms | 360.820ms | not read | not read | not read | 3.34M/s |
| q34 | group by a long string | 187.000ms | 507.995ms | 425.544ms | 25.2% | 344.963ms | 452.210ms | 310.352ms | 917.277ms | not read | not read | not read | 2.35M/s |
| q35 | group by a constant and a long string | 229.000ms | 334.812ms | 360.886ms | 21.4% | 291.301ms | 368.397ms | 240.198ms | 403.289ms | not read | not read | not read | 2.77M/s |
| q36 | group by four expressions | 55.000ms | 259.231ms | 206.820ms | 39.0% | 198.794ms | 279.417ms | 187.422ms | 359.406ms | not read | not read | not read | 4.84M/s |
| q37 | date range and group by a URL | 17.000ms | 183.460ms | 141.974ms | 23.4% | 138.449ms | 171.642ms | 132.365ms | 191.224ms | not read | not read | not read | 7.04M/s |
| q38 | date range and group by a title | 18.000ms | 149.290ms | 154.302ms | 11.3% | 138.149ms | 155.641ms | 137.017ms | 175.741ms | not read | not read | not read | 6.48M/s |
| q39 | date range, group by and offset | 16.000ms | 141.874ms | 152.330ms | 6.1% | 144.231ms | 153.503ms | 141.981ms | 154.251ms | not read | not read | not read | 6.56M/s |
| q40 | date range, a case and a wide group by | 26.000ms | 159.566ms | 163.923ms | 10.2% | 160.177ms | 176.977ms | 140.647ms | 188.721ms | not read | not read | not read | 6.10M/s |
| q41 | date range with an IN and a hash | 13.000ms | 203.400ms | 149.108ms | 3.1% | 148.018ms | 152.696ms | 138.723ms | 177.350ms | not read | not read | not read | 6.71M/s |
| q42 | date range and a deep offset | 13.000ms | 148.428ms | 142.651ms | 5.2% | 140.305ms | 147.792ms | 134.584ms | 151.020ms | not read | not read | not read | 7.01M/s |
| q43 | minute buckets over a date range | 13.000ms | 135.541ms | 137.468ms | 4.5% | 131.329ms | 137.533ms | 130.144ms | 147.218ms | not read | not read | not read | 7.27M/s |

clickhouse-server 26.9.1.1225 over 43 of 43 queries. Total 2.387s by its own clock and 9.439s by ours, 10.064s cold, no reading of CPU, peak not read, 18.01M/s and 2.44 GiB/s.

Running it cost 295% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 3.22x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 999975 rows, one out of every 100 of the 99997497 in the full file, which is a development loop rather than the suite
- q5 swung by 12.8% of its median, and rule two wants under 10%
- q6 swung by 12.2% of its median, and rule two wants under 10%
- q7 swung by 16.4% of its median, and rule two wants under 10%
- q10 swung by 10.3% of its median, and rule two wants under 10%
- q11 swung by 16.9% of its median, and rule two wants under 10%
- q12 swung by 45.8% of its median, and rule two wants under 10%
- q13 swung by 13.2% of its median, and rule two wants under 10%
- q14 swung by 11.3% of its median, and rule two wants under 10%
- q15 swung by 25.5% of its median, and rule two wants under 10%
- q19 swung by 15.4% of its median, and rule two wants under 10%
- q20 swung by 16.9% of its median, and rule two wants under 10%
- q22 swung by 26.0% of its median, and rule two wants under 10%
- q23 swung by 24.9% of its median, and rule two wants under 10%
- q24 swung by 12.8% of its median, and rule two wants under 10%
- q27 swung by 30.4% of its median, and rule two wants under 10%
- q30 swung by 41.7% of its median, and rule two wants under 10%
- q31 swung by 75.3% of its median, and rule two wants under 10%
- q32 swung by 28.7% of its median, and rule two wants under 10%
- q33 swung by 28.8% of its median, and rule two wants under 10%
- q34 swung by 41.8% of its median, and rule two wants under 10%
- q35 swung by 57.8% of its median, and rule two wants under 10%
- q36 swung by 64.1% of its median, and rule two wants under 10%
- q37 swung by 12.1% of its median, and rule two wants under 10%
- q38 swung by 29.0% of its median, and rule two wants under 10%
- q39 swung by 10.5% of its median, and rule two wants under 10%
- q40 swung by 15.7% of its median, and rule two wants under 10%
- q41 swung by 12.7% of its median, and rule two wants under 10%
- q42 swung by 18.9% of its median, and rule two wants under 10%
- q43 swung by 28.1% of its median, and rule two wants under 10%

These swung wider than reporting rule two allows:

- duckdb swung by 75.3% of its median on q31, and rule two wants under 10%
- clickhouse-local swung by 60.1% of its median on q11, and rule two wants under 10%
- polars swung by 81.5% of its median on q3, and rule two wants under 10%
- clickhouse-server swung by 52.7% of its median on q30, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

polars did not run q28, because the SQLContext has no strlen and no character length function.

polars did not run q29, because the SQLContext has no regexp_replace.

polars did not run q36, because the SQLContext rejects the repeated ClientIP output name in the group by.

polars did not run q43, because the SQLContext has no date_trunc.

So the polars column is 39 of 43 queries and its ratio is over the shared ones.

Answers differ, so this is not a comparison: q23: clickhouse-local does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q23: polars does not agree with duckdb: 20 numbers against 20

Answers differ, so this is not a comparison: q23: clickhouse-server does not agree with duckdb: 20 numbers against 20

These were answered differently and the data does not say which is right:

- q4: AVG(UserID) over a hundred million bigints near 10^18, where the engines disagree for two reasons. The order the partial sums are added in moves the floating point ones further apart than the one part in a billion this harness calls the same number, and DuckDB is plainly wrong: it sums the bigint column short by a multiple of 2^64 on this file, where its own hugeint and decimal paths, rudb, and adding the column up outside a database all agree. tamnd/rudb-compat#12.
- q16: ORDER BY COUNT(*) DESC LIMIT 10 over user IDs, where the tenth place is a tie between many users with the same count and the engine picks.
- q17: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q18: LIMIT 10 with no ORDER BY at all, so the ten rows are whichever ten the group by handed back first and the query did not ask for any particular ten.
- q19: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q24: ORDER BY EventTime LIMIT 10, and EventTime has one second resolution over a corpus with millions of rows a second, so the tenth row is a tie.
- q32: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q33: ORDER BY COUNT(*) DESC LIMIT 10 over WatchID and ClientIP unfiltered, where almost every group has a count of one and the ten that come back are arbitrary.
- q40: ORDER BY PageViews DESC LIMIT 10 OFFSET 1000 with ties at the cut, as q39.
- q41: ORDER BY PageViews DESC LIMIT 10 OFFSET 100 with ties at the cut, as q39.
- q43: DATE_TRUNC on a timestamp, and ClickHouse renders a DateTime in the machine's timezone where DuckDB renders a TIMESTAMP in none, so the same epoch second prints seven hours apart on gamingpc-wsl and two hours apart on server3.

So they are not checked, and a wrong answer from any engine on one of them would go unnoticed here. Every other query in the suite is checked in full.

Hot here means page cache warm and not buffer pool warm for every row but clickhouse-server, which stayed up across the whole suite with its own caches warm. That is the stronger kind of hot, so a ratio against that column is a ratio between two different quantities.

