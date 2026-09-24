# clickbench on server3

This is one run of the clickbench suite on server3, over 3 engines and 43 queries, with 5 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

It ran over 9999750 rows, one out of every 10 of the 99997497 in the full file, which is a development loop and not the suite. Nothing here is comparable to a full run or to anybody else's number.

## What ran

| what | it was |
| --- | --- |
| suite | clickbench |
| queries | 43 |
| tables | 1.40 GiB of Parquet in 1 table |
| rows | 9999750 in the table every query reads |
| sample | 9999750 rows, one out of every 10 of the 99997497 in the full file |
| corpus | no manifest beside the data, so this run cannot say where it came from |
| summary | median with the interquartile range, per reporting rule two |
| timeout | 600s per query, which one query reached |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --rows 10000000 --engines duckdb,clickhouse-local,rudb --runs 5 --timeout 600 --report
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

| engine | version | state | load | load cpu | on disk | that size is | format | machine load |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 381.281s | 300.470s | 1.71 GiB | its own database file | its own | 43.00 to 26.61 |
| clickhouse-local | 26.9.1.1138 | ran | 185.197s | 155.730s | 1.03 GiB | its own MergeTree parts, as system.parts counts the active ones | its own | 26.61 to 22.15 |
| rudb | rudb 0.4.31 | ran | 162.019s | 231.050s | 1.06 GiB | its own database file | its own | 22.06 to 24.28 |
| duckdb-pinned | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| datafusion | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| polars | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

The machine load column is the one minute load average before and after that engine's suite, on a machine with 8 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

These engines started while the machine was already above half of that, which means they were measured against somebody else's work rather than on an idle box: duckdb, clickhouse-local, rudb. Their numbers are inflated by an amount nothing here can recover, and by a different amount each, depending on how much of the column was CPU bound. One case where that reading is unfair is a sweep that runs engines back to back, where the average has not had a minute to come down from the engine before.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb on 42 shared |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 138.398s | 185.340s | +34% | 220.458s | 105.060s | 0.57 | 1.01 GiB | 69.82 MiB | 3.11M/s | 444.14 MiB/s | 1.00x |
| clickhouse-local | not read | >287.360s | not read | >335.897s | not read | not read | not read | not read | 1.50M/s | 213.91 MiB/s | 1.44x |
| rudb | 14.684s | 20.246s | +38% | 24.463s | 21.580s | 1.07 | 247.38 MiB | 176.00 KiB | 29.28M/s | 4.09 GiB/s | 0.11x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | clickhouse-local | rudb |
| --- | --- | --- | --- | --- |
| q1 | count | 6.000ms | 158.000ms | 2.112ms |
| q2 | filtered count | 118.000ms | 292.000ms | 28.366ms |
| q3 | three aggregates | 308.000ms | 483.000ms | 2.596ms |
| q4 | average | 347.000ms | 677.000ms | 2.135ms |
| q5 | count distinct, high card | 799.000ms | 1.450s | 2.286ms |
| q6 | count distinct, strings | 1.587s | 1.795s | 1.917ms |
| q7 | min and max of a date | 16.000ms | 877.000ms | 2.105ms |
| q8 | group by, low card | 196.000ms | 772.000ms | 67.096ms |
| q9 | group by and count distinct | 979.000ms | 2.483s | 618.607ms |
| q10 | group by, several aggregates | 2.105s | 1.745s | 706.356ms |
| q11 | group by a string and count distinct | 597.000ms | 826.000ms | 114.024ms |
| q12 | group by two strings and count distinct | 1.075s | 1.315s | 180.989ms |
| q13 | group by a string and top k | 1.708s | 4.046s | 217.312ms |
| q14 | group by a string and count distinct | 2.401s | 5.095s | 364.782ms |
| q15 | group by two columns and top k | 1.908s | 4.153s | 341.536ms |
| q16 | group by, very high card | 1.043s | 2.808s | 148.767ms |
| q17 | group by two, very high card | 4.267s | 6.586s | 549.497ms |
| q18 | group by two, no ordering | 3.514s | 2.666s | 206.814ms |
| q19 | group by with an extract | 10.376s | 11.911s | 1.016s |
| q20 | point lookup | 128.000ms | 1.234s | 9.290ms |
| q21 | substring scan | 6.511s | 4.683s | 284.210ms |
| q22 | substring scan and group by | 5.857s | 4.823s | 435.747ms |
| q23 | two substring scans and group by | 9.597s | 11.165s | 1.052s |
| q24 | select star and top k | 5.114s | 7.433s | 518.203ms |
| q25 | top k by a date | 265.000ms | 4.450s | 52.297ms |
| q26 | top k by a string | 1.392s | 2.685s | 306.910ms |
| q27 | top k by two columns | 252.000ms | 2.159s | 201.262ms |
| q28 | group by with a string length | 6.341s | 2.372s | 629.762ms |
| q29 | group by a regular expression | 39.731s | failed | 3.829s |
| q30 | ninety sums over one column | 1.581s | 969.000ms | 72.938ms |
| q31 | group by two and several aggregates | 3.193s | 2.670s | 306.622ms |
| q32 | group by a high card pair | 3.391s | 6.602s | 336.786ms |
| q33 | group by a high card pair, unfiltered | 6.009s | 13.514s | 801.773ms |
| q34 | group by a long string | 7.301s | 11.349s | 204.357ms |
| q35 | group by a constant and a long string | 5.341s | 5.044s | 153.495ms |
| q36 | group by four expressions | 652.000ms | 689.000ms | 126.888ms |
| q37 | date range and group by a URL | 495.000ms | 1.419s | 91.372ms |
| q38 | date range and group by a title | 156.000ms | 1.173s | 44.561ms |
| q39 | date range, group by and offset | 253.000ms | 2.001s | 71.293ms |
| q40 | date range, a case and a wide group by | 993.000ms | 3.410s | 401.839ms |
| q41 | date range with an IN and a hash | 197.000ms | 692.000ms | 104.932ms |
| q42 | date range and a deep offset | 153.000ms | 795.000ms | 47.646ms |
| q43 | minute buckets over a date range | 145.000ms | 814.000ms | 27.144ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 6.000ms | 747.548ms | 320.739ms | 23.7% | 282.032ms | 357.932ms | 181.191ms | 384.440ms | 140.000ms | 35.94 MiB | 228.00 KiB | 31.18M/s |
| q2 | filtered count | 118.000ms | 469.449ms | 549.201ms | 37.2% | 359.562ms | 563.986ms | 305.981ms | 691.445ms | 170.000ms | 56.31 MiB | 10.07 MiB | 18.21M/s |
| q3 | three aggregates | 308.000ms | 1.143s | 676.047ms | 13.4% | 620.066ms | 710.747ms | 340.001ms | 1.188s | 300.000ms | 76.98 MiB | 11.25 MiB | 14.79M/s |
| q4 | average | 347.000ms | 774.770ms | 528.056ms | 83.9% | 525.963ms | 969.034ms | 486.878ms | 1.008s | 370.000ms | 65.71 MiB | 17.06 MiB | 18.94M/s |
| q5 | count distinct, high card | 799.000ms | 1.280s | 1.092s | 30.7% | 1.047s | 1.383s | 1.034s | 1.384s | 930.000ms | 118.31 MiB | 2.04 MiB | 9.16M/s |
| q6 | count distinct, strings | 1.587s | 2.500s | 2.371s | 14.9% | 2.156s | 2.511s | 1.814s | 2.788s | 1.340s | 259.93 MiB | 27.41 MiB | 4.22M/s |
| q7 | min and max of a date | 16.000ms | 401.199ms | 391.131ms | 45.0% | 255.031ms | 430.985ms | 219.086ms | 546.214ms | 110.000ms | 37.05 MiB | 256.00 KiB | 25.57M/s |
| q8 | group by, low card | 196.000ms | 565.619ms | 512.101ms | 37.5% | 407.033ms | 599.060ms | 313.994ms | 1.128s | 260.000ms | 59.06 MiB | 292.00 KiB | 19.53M/s |
| q9 | group by and count distinct | 979.000ms | 2.131s | 1.400s | 9.1% | 1.319s | 1.447s | 1.203s | 1.553s | 1.190s | 146.81 MiB | 8.73 MiB | 7.14M/s |
| q10 | group by, several aggregates | 2.105s | 1.730s | 2.488s | 23.4% | 2.298s | 2.880s | 1.791s | 3.123s | 1.820s | 194.55 MiB | 204.00 KiB | 4.02M/s |
| q11 | group by a string and count distinct | 597.000ms | 927.098ms | 1.169s | 41.7% | 902.661ms | 1.390s | 704.971ms | 1.872s | 580.000ms | 104.41 MiB | 8.64 MiB | 8.55M/s |
| q12 | group by two strings and count distinct | 1.075s | 1.863s | 1.664s | 53.6% | 1.544s | 2.436s | 1.501s | 3.056s | 780.000ms | 108.57 MiB | 2.00 MiB | 6.01M/s |
| q13 | group by a string and top k | 1.708s | 3.313s | 2.586s | 27.5% | 2.145s | 2.857s | 1.968s | 3.385s | 1.770s | 274.66 MiB | 128.00 KiB | 3.87M/s |
| q14 | group by a string and count distinct | 2.401s | 3.596s | 3.334s | 6.9% | 3.327s | 3.556s | 3.233s | 3.983s | 2.490s | 360.28 MiB | none | 3.00M/s |
| q15 | group by two columns and top k | 1.908s | 1.816s | 3.083s | 16.4% | 2.967s | 3.473s | 2.307s | 3.626s | 1.840s | 287.82 MiB | 3.25 MiB | 3.24M/s |
| q16 | group by, very high card | 1.043s | 1.600s | 1.984s | 7.8% | 1.943s | 2.097s | 1.507s | 2.116s | 1.110s | 139.86 MiB | none | 5.04M/s |
| q17 | group by two, very high card | 4.267s | 3.792s | 5.609s | 7.5% | 5.361s | 5.783s | 3.661s | 6.894s | 3.120s | 344.94 MiB | none | 1.78M/s |
| q18 | group by two, no ordering | 3.514s | 4.754s | 5.431s | 35.3% | 3.856s | 5.772s | 3.583s | 6.642s | 2.420s | 333.83 MiB | 496.00 KiB | 1.84M/s |
| q19 | group by with an extract | 10.376s | 12.961s | 13.980s | 19.4% | 11.858s | 14.576s | 10.400s | 16.125s | 5.200s | 632.07 MiB | 58.36 MiB | 715.27K/s |
| q20 | point lookup | 128.000ms | 1.723s | 813.031ms | 77.0% | 696.922ms | 1.323s | 372.975ms | 2.112s | 160.000ms | 51.31 MiB | 128.00 KiB | 12.30M/s |
| q21 | substring scan | 6.511s | 6.912s | 8.876s | 17.6% | 8.772s | 10.335s | 7.981s | 11.197s | 3.020s | 327.07 MiB | 279.18 MiB | 1.13M/s |
| q22 | substring scan and group by | 5.857s | 9.625s | 7.750s | 11.7% | 7.629s | 8.538s | 7.267s | 10.887s | 3.210s | 396.25 MiB | 43.05 MiB | 1.29M/s |
| q23 | two substring scans and group by | 9.597s | 15.949s | 12.860s | 4.5% | 12.508s | 13.091s | 11.734s | 14.355s | 5.020s | 636.62 MiB | 326.21 MiB | 777.61K/s |
| q24 | select star and top k | 5.114s | 7.835s | 7.411s | 20.7% | 6.487s | 8.018s | 6.298s | 10.098s | 2.720s | 356.33 MiB | 97.54 MiB | 1.35M/s |
| q25 | top k by a date | 265.000ms | 1.376s | 654.942ms | 38.9% | 492.025ms | 746.893ms | 300.022ms | 1.637s | 210.000ms | 54.69 MiB | 2.50 MiB | 15.27M/s |
| q26 | top k by a string | 1.392s | 2.787s | 2.454s | 27.3% | 2.102s | 2.772s | 1.237s | 3.194s | 710.000ms | 108.03 MiB | 256.00 KiB | 4.07M/s |
| q27 | top k by two columns | 252.000ms | 420.146ms | 776.914ms | 51.2% | 439.874ms | 837.436ms | 240.143ms | 1.194s | 220.000ms | 55.06 MiB | 2.50 MiB | 12.87M/s |
| q28 | group by with a string length | 6.341s | 10.298s | 8.539s | 4.7% | 8.505s | 8.910s | 5.654s | 13.736s | 2.720s | 342.61 MiB | 79.35 MiB | 1.17M/s |
| q29 | group by a regular expression | 39.731s | 59.296s | 43.019s | 12.6% | 40.376s | 45.815s | 39.365s | 50.404s | 29.200s | 681.22 MiB | 293.75 MiB | 232.45K/s |
| q30 | ninety sums over one column | 1.581s | 3.393s | 2.876s | 13.6% | 2.500s | 2.891s | 1.699s | 3.738s | 450.000ms | 68.68 MiB | 18.70 MiB | 3.48M/s |
| q31 | group by two and several aggregates | 3.193s | 8.322s | 4.610s | 0.0% | 4.610s | 4.612s | 3.712s | 4.842s | 1.870s | 258.17 MiB | 89.07 MiB | 2.17M/s |
| q32 | group by a high card pair | 3.391s | 4.826s | 5.502s | 23.1% | 4.277s | 5.547s | 3.757s | 6.178s | 2.090s | 383.60 MiB | 80.95 MiB | 1.82M/s |
| q33 | group by a high card pair, unfiltered | 6.009s | 13.600s | 8.415s | 7.9% | 8.154s | 8.818s | 8.044s | 8.909s | 5.870s | 862.74 MiB | 14.77 MiB | 1.19M/s |
| q34 | group by a long string | 7.301s | 13.246s | 9.312s | 53.3% | 8.238s | 13.200s | 7.644s | 15.243s | 7.580s | 1011.09 MiB | 272.53 MiB | 1.07M/s |
| q35 | group by a constant and a long string | 5.341s | 8.376s | 6.646s | 5.5% | 6.310s | 6.673s | 6.094s | 9.273s | 9.330s | 1.01 GiB | 140.00 KiB | 1.50M/s |
| q36 | group by four expressions | 652.000ms | 915.305ms | 1.053s | 9.1% | 973.423ms | 1.069s | 690.931ms | 1.441s | 1.200s | 129.18 MiB | 520.00 KiB | 9.50M/s |
| q37 | date range and group by a URL | 495.000ms | 1.393s | 1.157s | 41.2% | 821.376ms | 1.297s | 658.045ms | 1.526s | 720.000ms | 145.05 MiB | 1.95 MiB | 8.64M/s |
| q38 | date range and group by a title | 156.000ms | 479.896ms | 364.005ms | 40.3% | 317.531ms | 464.370ms | 306.643ms | 515.533ms | 300.000ms | 63.68 MiB | 2.62 MiB | 27.47M/s |
| q39 | date range, group by and offset | 253.000ms | 552.268ms | 483.604ms | 8.3% | 459.775ms | 499.732ms | 439.860ms | 797.229ms | 450.000ms | 79.80 MiB | 1.50 MiB | 20.68M/s |
| q40 | date range, a case and a wide group by | 993.000ms | 1.210s | 1.358s | 11.8% | 1.342s | 1.503s | 1.202s | 1.651s | 1.300s | 235.50 MiB | 19.80 MiB | 7.36M/s |
| q41 | date range with an IN and a hash | 197.000ms | 594.165ms | 412.935ms | 25.1% | 405.351ms | 508.801ms | 292.518ms | 545.205ms | 270.000ms | 61.80 MiB | 10.45 MiB | 24.22M/s |
| q42 | date range and a deep offset | 153.000ms | 621.604ms | 393.395ms | 22.3% | 336.432ms | 424.320ms | 322.411ms | 476.623ms | 250.000ms | 56.68 MiB | 1.44 MiB | 25.42M/s |
| q43 | minute buckets over a date range | 145.000ms | 341.671ms | 433.351ms | 61.3% | 318.606ms | 584.201ms | 299.754ms | 1.083s | 250.000ms | 51.30 MiB | 3.15 MiB | 23.08M/s |

duckdb v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 138.398s by its own clock and 185.340s by ours, 220.458s cold, 105.060s of CPU, peak 1.01 GiB, 3.11M/s and 444.14 MiB/s.

Running it cost 34% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 134.13x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 158.000ms | 1.735s | 2.527s | 24.9% | 2.181s | 2.811s | 2.072s | 3.491s | 1.260s | 249.75 MiB | 208.00 KiB | 3.96M/s |
| q2 | filtered count | 292.000ms | 4.608s | 3.118s | 13.9% | 3.059s | 3.492s | 2.714s | 4.120s | 1.460s | 330.25 MiB | 276.00 KiB | 3.21M/s |
| q3 | three aggregates | 483.000ms | 3.686s | 3.105s | 16.9% | 2.969s | 3.495s | 2.921s | 4.378s | 1.330s | 260.50 MiB | 1.30 MiB | 3.22M/s |
| q4 | average | 677.000ms | 5.625s | 3.061s | 9.2% | 2.999s | 3.282s | 2.596s | 3.370s | 1.540s | 263.25 MiB | 7.11 MiB | 3.27M/s |
| q5 | count distinct, high card | 1.450s | 4.852s | 4.166s | 28.6% | 3.412s | 4.602s | 3.220s | 6.240s | 2.710s | 467.88 MiB | 544.00 KiB | 2.40M/s |
| q6 | count distinct, strings | 1.795s | 3.817s | 3.611s | 3.6% | 3.525s | 3.655s | 3.460s | 4.208s | 3.530s | 545.09 MiB | 26.65 MiB | 2.77M/s |
| q7 | min and max of a date | 877.000ms | 3.124s | 2.831s | 12.2% | 2.791s | 3.136s | 2.544s | 4.203s | 1.550s | 341.29 MiB | 480.00 KiB | 3.53M/s |
| q8 | group by, low card | 772.000ms | 2.776s | 2.327s | 16.3% | 2.254s | 2.633s | 1.826s | 4.893s | 1.540s | 265.88 MiB | 2.07 MiB | 4.30M/s |
| q9 | group by and count distinct | 2.483s | 11.520s | 5.495s | 2.3% | 5.472s | 5.597s | 3.406s | 5.861s | 2.970s | 425.02 MiB | 8.04 MiB | 1.82M/s |
| q10 | group by, several aggregates | 1.745s | 4.563s | 4.418s | 22.8% | 3.647s | 4.655s | 3.560s | 5.702s | 3.010s | 410.79 MiB | 3.10 MiB | 2.26M/s |
| q11 | group by a string and count distinct | 826.000ms | 4.978s | 3.675s | 27.7% | 3.084s | 4.102s | 2.437s | 4.683s | 1.920s | 361.38 MiB | 1.27 MiB | 2.72M/s |
| q12 | group by two strings and count distinct | 1.315s | 2.794s | 3.756s | 34.6% | 3.682s | 4.983s | 3.291s | 8.197s | 1.880s | 363.07 MiB | 992.00 KiB | 2.66M/s |
| q13 | group by a string and top k | 4.046s | 11.180s | 9.539s | 39.9% | 7.444s | 11.254s | 7.262s | 20.617s | 2.880s | 484.33 MiB | 28.01 MiB | 1.05M/s |
| q14 | group by a string and count distinct | 5.095s | 11.967s | 7.821s | 21.5% | 7.503s | 9.184s | 6.700s | 9.832s | 2.960s | 474.20 MiB | 27.65 MiB | 1.28M/s |
| q15 | group by two columns and top k | 4.153s | 6.619s | 7.476s | 115.0% | 6.475s | 15.069s | 6.151s | 28.635s | 4.120s | 507.68 MiB | 4.14 MiB | 1.34M/s |
| q16 | group by, very high card | 2.808s | 7.958s | 6.721s | 50.6% | 5.879s | 9.279s | 5.020s | 14.926s | 2.220s | 352.62 MiB | 1.58 MiB | 1.49M/s |
| q17 | group by two, very high card | 6.586s | 10.206s | 9.108s | 27.7% | 7.898s | 10.423s | 7.556s | 15.183s | not read | not read | 14.25 MiB | 1.10M/s |
| q18 | group by two, no ordering | 2.666s | 3.139s | 7.437s | 40.5% | 5.002s | 8.016s | 4.791s | 10.348s | 3.120s | 345.43 MiB | 1.03 MiB | 1.34M/s |
| q19 | group by with an extract | 11.911s | 18.277s | 16.001s | 23.0% | 12.996s | 16.682s | 12.888s | 19.836s | 6.460s | 646.81 MiB | 39.38 MiB | 624.95K/s |
| q20 | point lookup | 1.234s | 4.397s | 6.290s | 72.0% | 4.793s | 9.321s | 3.410s | 10.414s | 1.700s | 334.23 MiB | 260.00 KiB | 1.59M/s |
| q21 | substring scan | 4.683s | 20.780s | 8.651s | 33.3% | 7.529s | 10.412s | 7.191s | 10.889s | 4.210s | 322.34 MiB | 159.18 MiB | 1.16M/s |
| q22 | substring scan and group by | 4.823s | 9.986s | 7.676s | 27.9% | 5.833s | 7.976s | 5.260s | 8.173s | 4.670s | 335.13 MiB | 880.00 KiB | 1.30M/s |
| q23 | two substring scans and group by | 11.165s | 10.796s | 16.206s | 37.3% | 13.410s | 19.460s | 11.326s | 20.389s | 6.960s | 357.42 MiB | 168.39 MiB | 617.04K/s |
| q24 | select star and top k | 7.433s | 15.094s | 15.102s | 25.1% | 13.616s | 17.412s | 12.906s | 18.673s | 4.790s | 433.30 MiB | 33.06 MiB | 662.14K/s |
| q25 | top k by a date | 4.450s | 12.405s | 11.205s | 2.7% | 11.016s | 11.316s | 8.799s | 12.089s | 2.360s | 344.50 MiB | 6.04 MiB | 892.45K/s |
| q26 | top k by a string | 2.685s | 12.839s | 9.928s | 10.0% | 9.462s | 10.459s | 7.482s | 15.592s | 2.970s | 376.55 MiB | 17.43 MiB | 1.01M/s |
| q27 | top k by two columns | 2.159s | 7.076s | 6.364s | 42.4% | 4.422s | 7.121s | 3.753s | 10.482s | 2.260s | 375.07 MiB | none | 1.57M/s |
| q28 | group by with a string length | 2.372s | 8.285s | 6.694s | 33.6% | 6.465s | 8.712s | 5.593s | 10.170s | 2.080s | 333.90 MiB | 10.22 MiB | 1.49M/s |
| q29 | group by a regular expression | not read | 0.000us | 0.000us | n/a | 0.000us | 0.000us | 0.000us | 0.000us | not read | not read | not read | n/a |
| q30 | ninety sums over one column | 969.000ms | 16.900s | 4.632s | 38.7% | 4.147s | 5.941s | 3.072s | 6.453s | 1.440s | 334.00 MiB | 153.34 MiB | 2.16M/s |
| q31 | group by two and several aggregates | 2.670s | 6.458s | 5.728s | 61.4% | 5.359s | 8.878s | 4.961s | 11.200s | 2.780s | 404.19 MiB | 25.40 MiB | 1.75M/s |
| q32 | group by a high card pair | 6.602s | 8.074s | 11.865s | 22.8% | 10.640s | 13.344s | 7.282s | 20.878s | 2.410s | 440.09 MiB | 55.41 MiB | 842.76K/s |
| q33 | group by a high card pair, unfiltered | 13.514s | 12.194s | 20.860s | 26.9% | 16.638s | 22.245s | 13.835s | 29.355s | 5.930s | 622.82 MiB | 264.00 KiB | 479.37K/s |
| q34 | group by a long string | 11.349s | 29.040s | 14.704s | 20.2% | 12.947s | 15.920s | 11.591s | 16.338s | 9.140s | 841.05 MiB | 210.36 MiB | 680.05K/s |
| q35 | group by a constant and a long string | 5.044s | 9.620s | 6.503s | 54.7% | 5.539s | 9.094s | 4.680s | 9.404s | 8.900s | 868.16 MiB | none | 1.54M/s |
| q36 | group by four expressions | 689.000ms | 2.601s | 2.960s | 18.2% | 2.503s | 3.042s | 1.918s | 3.161s | 2.010s | 385.87 MiB | 224.00 KiB | 3.38M/s |
| q37 | date range and group by a URL | 1.419s | 4.476s | 3.665s | 18.0% | 3.380s | 4.039s | 3.175s | 4.545s | 2.410s | 403.21 MiB | 1.40 MiB | 2.73M/s |
| q38 | date range and group by a title | 1.173s | 3.425s | 3.098s | 39.7% | 3.012s | 4.242s | 2.309s | 4.272s | 1.850s | 356.88 MiB | 3.08 MiB | 3.23M/s |
| q39 | date range, group by and offset | 2.001s | 3.838s | 5.535s | 39.9% | 5.025s | 7.234s | 4.617s | 7.355s | 2.250s | 375.65 MiB | 284.00 KiB | 1.81M/s |
| q40 | date range, a case and a wide group by | 3.410s | 6.334s | 5.528s | 15.9% | 5.245s | 6.122s | 4.101s | 6.158s | 3.780s | 555.98 MiB | 12.02 MiB | 1.81M/s |
| q41 | date range with an IN and a hash | 692.000ms | 2.395s | 2.322s | 4.5% | 2.227s | 2.332s | 2.060s | 2.439s | 1.910s | 347.00 MiB | 6.81 MiB | 4.31M/s |
| q42 | date range and a deep offset | 795.000ms | 2.894s | 2.806s | 10.0% | 2.552s | 2.831s | 2.503s | 3.101s | 2.170s | 362.00 MiB | 580.00 KiB | 3.56M/s |
| q43 | minute buckets over a date range | 814.000ms | 2.567s | 2.845s | 29.4% | 2.475s | 3.311s | 2.119s | 3.590s | 1.850s | 350.67 MiB | 1.96 MiB | 3.52M/s |

clickhouse-local 26.9.1.1138 over 43 of 43 queries. Total no reading by its own clock and 287.360s by ours, 335.897s cold, no reading of CPU, peak not read, 1.50M/s and 213.91 MiB/s.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 2.112ms | 83.789ms | 133.065ms | 14.3% | 120.966ms | 139.970ms | 55.205ms | 165.966ms | 10.000ms | 11.12 MiB | 72.00 KiB | 75.15M/s |
| q2 | filtered count | 28.366ms | 307.191ms | 183.796ms | 31.0% | 141.943ms | 198.936ms | 84.106ms | 209.218ms | 30.000ms | 15.00 MiB | 920.00 KiB | 54.41M/s |
| q3 | three aggregates | 2.596ms | 59.261ms | 58.811ms | 19.5% | 54.457ms | 65.954ms | 43.165ms | 77.890ms | 0.000us | 11.62 MiB | none | 170.03M/s |
| q4 | average | 2.135ms | 44.262ms | 61.133ms | 79.3% | 46.989ms | 95.483ms | 42.918ms | 167.900ms | 10.000ms | 11.62 MiB | none | 163.57M/s |
| q5 | count distinct, high card | 2.286ms | 101.466ms | 91.955ms | 32.5% | 64.082ms | 93.977ms | 45.944ms | 96.030ms | 10.000ms | 11.25 MiB | none | 108.75M/s |
| q6 | count distinct, strings | 1.917ms | 82.001ms | 155.416ms | 22.9% | 124.976ms | 160.587ms | 81.001ms | 177.000ms | 10.000ms | 11.25 MiB | none | 64.34M/s |
| q7 | min and max of a date | 2.105ms | 104.890ms | 75.511ms | 37.3% | 57.910ms | 86.093ms | 43.067ms | 162.566ms | 10.000ms | 11.38 MiB | none | 132.43M/s |
| q8 | group by, low card | 67.096ms | 356.796ms | 195.959ms | 94.4% | 125.161ms | 310.174ms | 100.005ms | 385.968ms | 40.000ms | 15.75 MiB | 68.00 KiB | 51.03M/s |
| q9 | group by and count distinct | 618.607ms | 690.889ms | 962.797ms | 8.7% | 900.503ms | 984.056ms | 619.930ms | 1.008s | 460.000ms | 62.75 MiB | 2.10 MiB | 10.39M/s |
| q10 | group by, several aggregates | 706.356ms | 716.741ms | 913.417ms | 27.7% | 774.918ms | 1.028s | 680.988ms | 1.104s | 800.000ms | 66.50 MiB | 444.00 KiB | 10.95M/s |
| q11 | group by a string and count distinct | 114.024ms | 299.395ms | 242.749ms | 18.4% | 231.782ms | 276.461ms | 211.851ms | 276.595ms | 160.000ms | 27.38 MiB | 180.00 KiB | 41.19M/s |
| q12 | group by two strings and count distinct | 180.989ms | 502.110ms | 313.851ms | 29.9% | 303.130ms | 396.857ms | 284.019ms | 400.130ms | 180.000ms | 28.50 MiB | 168.00 KiB | 31.86M/s |
| q13 | group by a string and top k | 217.312ms | 439.487ms | 312.224ms | 10.2% | 299.987ms | 331.960ms | 216.377ms | 375.220ms | 190.000ms | 28.88 MiB | 2.34 MiB | 32.03M/s |
| q14 | group by a string and count distinct | 364.782ms | 424.787ms | 445.062ms | 31.1% | 441.275ms | 579.758ms | 434.183ms | 748.883ms | 480.000ms | 58.50 MiB | 388.00 KiB | 22.47M/s |
| q15 | group by two columns and top k | 341.536ms | 532.625ms | 553.747ms | 29.6% | 470.011ms | 634.137ms | 319.763ms | 726.322ms | 530.000ms | 71.62 MiB | 1.39 MiB | 18.06M/s |
| q16 | group by, very high card | 148.767ms | 489.999ms | 241.894ms | 23.7% | 209.229ms | 266.677ms | 199.638ms | 408.028ms | 270.000ms | 53.62 MiB | none | 41.34M/s |
| q17 | group by two, very high card | 549.497ms | 838.313ms | 680.516ms | 23.2% | 645.763ms | 803.809ms | 574.105ms | 840.338ms | 1.240s | 152.75 MiB | none | 14.69M/s |
| q18 | group by two, no ordering | 206.814ms | 258.728ms | 248.769ms | 28.6% | 239.595ms | 310.715ms | 200.308ms | 397.058ms | 380.000ms | 30.25 MiB | none | 40.20M/s |
| q19 | group by with an extract | 1.016s | 1.155s | 1.169s | 13.2% | 1.136s | 1.291s | 1.045s | 1.424s | 2.010s | 247.38 MiB | 1.08 MiB | 8.55M/s |
| q20 | point lookup | 9.290ms | 96.997ms | 80.126ms | 29.1% | 76.608ms | 99.948ms | 43.266ms | 122.776ms | 10.000ms | 15.25 MiB | 776.00 KiB | 124.80M/s |
| q21 | substring scan | 284.210ms | 621.638ms | 384.276ms | 7.8% | 379.779ms | 409.921ms | 352.642ms | 411.643ms | 480.000ms | 42.50 MiB | 8.63 MiB | 26.02M/s |
| q22 | substring scan and group by | 435.747ms | 484.494ms | 550.803ms | 14.3% | 536.156ms | 615.002ms | 491.842ms | 844.040ms | 700.000ms | 57.12 MiB | 12.00 KiB | 18.15M/s |
| q23 | two substring scans and group by | 1.052s | 1.495s | 1.350s | 29.2% | 1.091s | 1.486s | 941.263ms | 1.554s | 1.100s | 90.62 MiB | 3.66 MiB | 7.40M/s |
| q24 | select star and top k | 518.203ms | 720.151ms | 678.016ms | 13.1% | 589.920ms | 678.880ms | 556.369ms | 884.958ms | 470.000ms | 71.75 MiB | 784.00 KiB | 14.75M/s |
| q25 | top k by a date | 52.297ms | 191.639ms | 133.957ms | 54.2% | 103.412ms | 175.980ms | 86.715ms | 259.652ms | 70.000ms | 20.62 MiB | none | 74.65M/s |
| q26 | top k by a string | 306.910ms | 419.359ms | 536.198ms | 50.2% | 287.039ms | 555.982ms | 201.966ms | 656.826ms | 150.000ms | 29.50 MiB | 604.00 KiB | 18.65M/s |
| q27 | top k by two columns | 201.262ms | 284.971ms | 333.987ms | 66.2% | 168.999ms | 389.938ms | 95.423ms | 477.987ms | 50.000ms | 24.12 MiB | none | 29.94M/s |
| q28 | group by with a string length | 629.762ms | 929.684ms | 865.137ms | 30.7% | 665.839ms | 931.274ms | 480.026ms | 945.950ms | 380.000ms | 50.00 MiB | 68.00 KiB | 11.56M/s |
| q29 | group by a regular expression | 3.829s | 6.773s | 4.013s | 19.5% | 3.858s | 4.642s | 3.397s | 5.310s | 6.140s | 227.12 MiB | 46.08 MiB | 2.49M/s |
| q30 | ninety sums over one column | 72.938ms | 221.692ms | 162.870ms | 12.9% | 156.779ms | 177.721ms | 120.053ms | 280.447ms | 130.000ms | 16.88 MiB | 84.00 KiB | 61.40M/s |
| q31 | group by two and several aggregates | 306.622ms | 679.970ms | 496.846ms | 11.1% | 465.061ms | 520.153ms | 419.652ms | 602.376ms | 570.000ms | 67.75 MiB | 1.18 MiB | 20.13M/s |
| q32 | group by a high card pair | 336.786ms | 595.852ms | 457.215ms | 18.2% | 442.757ms | 526.170ms | 417.454ms | 596.912ms | 640.000ms | 96.88 MiB | 3.89 MiB | 21.87M/s |
| q33 | group by a high card pair, unfiltered | 801.773ms | 898.875ms | 981.873ms | 4.6% | 978.799ms | 1.024s | 709.605ms | 1.076s | 1.770s | 231.25 MiB | none | 10.18M/s |
| q34 | group by a long string | 204.357ms | 435.947ms | 300.055ms | 7.7% | 278.176ms | 301.307ms | 199.881ms | 394.541ms | 310.000ms | 78.50 MiB | none | 33.33M/s |
| q35 | group by a constant and a long string | 153.495ms | 320.021ms | 300.053ms | 40.5% | 212.268ms | 333.894ms | 198.756ms | 354.977ms | 370.000ms | 77.00 MiB | none | 33.33M/s |
| q36 | group by four expressions | 126.888ms | 179.981ms | 216.193ms | 17.8% | 182.192ms | 220.620ms | 163.983ms | 319.177ms | 330.000ms | 54.00 MiB | none | 46.25M/s |
| q37 | date range and group by a URL | 91.372ms | 197.748ms | 179.863ms | 55.2% | 120.459ms | 219.729ms | 120.086ms | 240.115ms | 100.000ms | 37.88 MiB | 136.00 KiB | 55.60M/s |
| q38 | date range and group by a title | 44.561ms | 197.814ms | 171.946ms | 20.3% | 139.951ms | 174.889ms | 138.413ms | 185.040ms | 60.000ms | 30.00 MiB | none | 58.16M/s |
| q39 | date range, group by and offset | 71.293ms | 111.059ms | 125.205ms | 4.0% | 122.020ms | 127.024ms | 112.294ms | 235.163ms | 50.000ms | 26.25 MiB | 92.00 KiB | 79.87M/s |
| q40 | date range, a case and a wide group by | 401.839ms | 600.304ms | 497.762ms | 19.8% | 430.962ms | 529.441ms | 379.042ms | 564.054ms | 620.000ms | 76.50 MiB | 280.00 KiB | 20.09M/s |
| q41 | date range with an IN and a hash | 104.932ms | 203.692ms | 171.466ms | 44.7% | 151.185ms | 227.760ms | 148.889ms | 274.375ms | 110.000ms | 41.88 MiB | 384.00 KiB | 58.32M/s |
| q42 | date range and a deep offset | 47.646ms | 213.323ms | 119.148ms | 50.4% | 114.897ms | 174.999ms | 110.958ms | 188.889ms | 100.000ms | 32.12 MiB | 500.00 KiB | 83.93M/s |
| q43 | minute buckets over a date range | 27.144ms | 101.681ms | 99.296ms | 41.7% | 97.586ms | 138.992ms | 64.356ms | 155.765ms | 50.000ms | 24.25 MiB | 128.00 KiB | 100.71M/s |

rudb rudb 0.4.31 over 43 of 43 queries. Total 14.684s by its own clock and 20.246s by ours, 24.463s cold, 21.580s of CPU, peak 247.38 MiB, 29.28M/s and 4.09 GiB/s.

Running it cost 38% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 68.24x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | moved | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | 2.603ms | 168.154us | 122.560us | 168.185us | 27.1% | 122.560us | 534.943us | 9.297ms | 192 B | 1.0x | 2 of 2 |
| q2 | 14.673ms | 101.042ms | 22.120ms | 22.535ms | 1.8% | 22.120ms | 1.927ms | 45.538ms | 896 B | 202135.0x | 3 of 3 |
| q3 | 1.469ms | 108.021us | 82.475us | 108.053us | 23.7% | 82.475us | 382.056us | 9.510ms | 472 B | 1.0x | 2 of 2 |
| q4 | 1.065ms | 99.686us | 74.209us | 99.727us | 25.6% | 74.209us | 302.367us | 9.598ms | 192 B | 1.0x | 2 of 2 |
| q5 | 959.992us | 105.838us | 81.873us | 105.858us | 22.7% | 81.873us | 248.135us | 39.646ms | 192 B | 1.0x | 2 of 2 |
| q6 | 1.202ms | 104.224us | 77.876us | 104.235us | 25.3% | 77.876us | 262.893us | 9.633ms | 192 B | 1.0x | 2 of 2 |
| q7 | 1.283ms | 69.280us | 44.444us | 69.300us | 35.9% | 44.444us | 327.414us | 0.000us | 320 B | 1.0x | 2 of 2 |
| q8 | 22.775ms | 114.538ms | 19.333ms | 19.430ms | 0.5% | 19.333ms | 497.653us | 10.072ms | 2.62 KiB | 25268.8x | 4 of 4 |
| q9 | 143.038ms | 429.980ms | 280.878ms | 280.964ms | 0.0% | 280.878ms | 476.814us | 138.559ms | 46.72 MiB | 1000103.0x | 4 of 4 |
| q10 | 4.990ms | 625.858ms | 634.821ms | 634.932ms | 0.0% | 634.821ms | 579.066us | 134.489ms | 45.78 MiB | 1000103.0x | 4 of 4 |
| q11 | 5.250ms | 197.035ms | 195.531ms | 195.629ms | 0.1% | 195.531ms | 792.506us | 33.579ms | 2.88 MiB | 35001.1x | 4 of 4 |
| q12 | 2.044ms | 216.929ms | 254.338ms | 254.440ms | 0.0% | 254.338ms | 635.090us | 74.925ms | 2.98 MiB | 35013.5x | 4 of 4 |
| q13 | 2.568ms | 113.135ms | 108.626ms | 108.683ms | 0.1% | 108.626ms | 842.079us | 50.475ms | 12.08 MiB | 137414.9x | 4 of 4 |
| q14 | 4.267ms | 370.448ms | 390.629ms | 390.726ms | 0.0% | 390.629ms | 523.391us | 198.751ms | 33.23 MiB | 137534.9x | 4 of 4 |
| q15 | 2.151ms | 419.799ms | 349.546ms | 349.645ms | 0.0% | 349.546ms | 596.709us | 159.759ms | 52.08 MiB | 137534.9x | 4 of 4 |
| q16 | 6.463ms | 270.655ms | 244.438ms | 244.656ms | 0.1% | 244.438ms | 1.151ms | 164.193ms | 31.54 MiB | 1000103.0x | 4 of 4 |
| q17 | 1.789ms | 617.878ms | 1.006s | 1.006s | 0.0% | 1.006s | 532.639us | 183.514ms | 194.78 MiB | 1000103.0x | 4 of 4 |
| q18 | 1.533ms | 193.851ms | 303.587ms | 303.649ms | 0.0% | 303.587ms | 414.867us | 15.936ms | 388.35 KiB | 999977.0x | 4 of 4 |
| q19 | 2.130ms | 928.374ms | 1.441s | 1.441s | 0.0% | 1.441s | 758.262us | 478.422ms | 333.64 MiB | 1000103.0x | 4 of 4 |
| q20 | 989.788us | 20.291ms | 9.426ms | 9.506ms | 0.9% | 9.426ms | 308.007us | 40.186ms | 0 B | not read | 2 of 2 |
| q21 | 1.559ms | 567.684ms | 770.640ms | 770.705ms | 0.0% | 770.640ms | 670.267us | 18.625ms | 896 B | 647.0x | 3 of 3 |
| q22 | 2.210ms | 439.162ms | 613.360ms | 613.416ms | 0.0% | 613.360ms | 549.129us | 16.035ms | 502.95 KiB | 6.4x | 4 of 4 |
| q23 | 2.354ms | 1.270s | 1.330s | 1.330s | 0.0% | 1.330s | 720.320us | 99.496ms | 656.47 KiB | 111.1x | 4 of 4 |
| q24 | 3.260ms | 603.808ms | 377.557ms | 380.702ms | 0.8% | 377.557ms | 461.555us | 188.836ms | 30.18 KiB | 64.4x | 4 of 4 |
| q25 | 4.215ms | 51.220ms | 16.715ms | 17.535ms | 4.7% | 16.715ms | 357.941us | 42.107ms | 28.00 KiB | 12926.6x | 4 of 4 |
| q26 | 1.347ms | 238.462ms | 198.936ms | 203.468ms | 2.2% | 198.936ms | 404.759us | 6.127ms | 17.28 KiB | 274813.8x | 3 of 3 |
| q27 | 1.166ms | 104.623ms | 30.294ms | 31.113ms | 2.6% | 30.294ms | 368.881us | 38.518ms | 33.00 KiB | 13738.0x | 4 of 4 |
| q28 | 69.244ms | 573.812ms | 363.867ms | 363.919ms | 0.0% | 363.867ms | 539.030us | 35.542ms | 21.03 KiB | 833261.8x | 5 of 5 |
| q29 | 2.327ms | 6.212s | 6.798s | 6.798s | 0.0% | 6.798s | 881.262us | 250.738ms | 78.94 MiB | 578849.7x | 5 of 5 |
| q30 | 47.058ms | 39.232ms | 44.739ms | 44.909ms | 0.4% | 44.739ms | 38.843ms | 16.247ms | 29.59 KiB | 9999751.0x | 3 of 3 |
| q31 | 2.325ms | 509.877ms | 695.183ms | 695.283ms | 0.0% | 695.183ms | 577.182us | 154.140ms | 39.67 MiB | 137534.9x | 4 of 4 |
| q32 | 2.081ms | 425.865ms | 786.572ms | 786.642ms | 0.0% | 786.572ms | 585.968us | 132.772ms | 40.60 MiB | 137534.9x | 4 of 4 |
| q33 | 1.769ms | 705.953ms | 988.687ms | 988.779ms | 0.0% | 988.687ms | 427.842us | 650.793ms | 205.74 MiB | 1000103.0x | 4 of 4 |
| q34 | 1.920ms | 259.344ms | 247.217ms | 247.282ms | 0.0% | 247.217ms | 673.983us | 82.044ms | 58.61 MiB | 999983.0x | 4 of 4 |
| q35 | 1.498ms | 261.144ms | 263.607ms | 263.696ms | 0.0% | 263.607ms | 436.428us | 85.868ms | 58.58 MiB | 999983.0x | 4 of 4 |
| q36 | 2.029ms | 101.322ms | 127.227ms | 127.299ms | 0.1% | 127.227ms | 580.739us | 52.121ms | 36.04 MiB | 1000167.0x | 5 of 5 |
| q37 | 2.404ms | 81.117ms | 75.950ms | 77.412ms | 1.9% | 75.950ms | 452.859us | 62.135ms | 22.53 MiB | 55817.7x | 4 of 4 |
| q38 | 2.537ms | 90.639ms | 80.335ms | 80.545ms | 0.3% | 80.335ms | 522.018us | 18.933ms | 14.70 MiB | 54895.3x | 4 of 4 |
| q39 | 1.976ms | 72.271ms | 62.756ms | 63.623ms | 1.4% | 62.756ms | 527.679us | 15.849ms | 20.20 MiB | 4734.2x | 4 of 4 |
| q40 | 5.115ms | 452.485ms | 652.971ms | 653.144ms | 0.0% | 652.971ms | 563.807us | 86.293ms | 23.88 MiB | 72956.8x | 4 of 4 |
| q41 | 6.133ms | 111.423ms | 182.283ms | 185.841ms | 1.9% | 182.283ms | 935.524us | 23.223ms | 3.53 MiB | 7512.5x | 4 of 4 |
| q42 | 1.908ms | 46.656ms | 46.207ms | 48.541ms | 4.8% | 46.207ms | 490.089us | 50.969ms | 5.50 MiB | not read | 4 of 4 |
| q43 | 1.768ms | 17.543ms | 39.445ms | 39.615ms | 0.4% | 39.445ms | 485.871us | 9.899ms | 770.00 KiB | 56098.5x | 4 of 4 |

Read from the breakdown the engine wrote for its cold run. `planning` is everything before the first row moved, which is the parse, the bind, the optimizer passes and building the tree. It is a column rather than a footnote because it is the one cost of a query nobody profiles: an optimizer only ever has passes added to it, each paying for itself on the query it was written for, and a query that plans for four hundred milliseconds to save two hundred is a query the optimizer made slower. What is gated is planning as a share of the whole statement, and that share is not in this table, because it is taken over every run rather than off the cold one the rest of these columns come from. It lives in `baselines/planning-<suite>.txt`. `accounted` is what its pipelines charged themselves and `measured` is what it measured around running them, so `apart` is the cross check and anything over five percent is time the breakdown cannot explain. `driver` is the part of `accounted` that was not inside an operator, which is the loop that runs a pipeline rather than the operators it calls. `build` is what the engine spent putting the tree together, which is before there is a pipeline to charge and is why it is off the right hand side of the check. `outside` is the CPU the process spent on everything else, which is starting, opening the data and printing the answer, and it is the reason the wall clock column and the query time column differ. `held` is what the operators reserved, which is not the resident set of the process. `moved` is every intermediate row the plan built divided by the rows it returned, which is the one column here that does not move when the kernels get faster: a suite that gets thirty percent quicker on a rewritten hash table reports thirty percent everywhere else and nothing at all here, and when this falls it is because the plans changed. `reference` is how many operators ran the reference implementation of their seam, which is the slow path kept for differential testing.

#### What the planner knew

| query | decisions | exact | certified | estimated | unknown |
| --- | --- | --- | --- | --- | --- |
| q1 | 2 | 2 | 0 | 0 | 0 |
| q2 | 3 | 2 | 0 | 1 | 0 |
| q3 | 2 | 2 | 0 | 0 | 0 |
| q4 | 2 | 2 | 0 | 0 | 0 |
| q5 | 2 | 2 | 0 | 0 | 0 |
| q6 | 2 | 2 | 0 | 0 | 0 |
| q7 | 2 | 2 | 0 | 0 | 0 |
| q8 | 4 | 0 | 0 | 4 | 0 |
| q9 | 4 | 1 | 0 | 3 | 0 |
| q10 | 4 | 1 | 0 | 3 | 0 |
| q11 | 4 | 0 | 0 | 4 | 0 |
| q12 | 4 | 0 | 0 | 4 | 0 |
| q13 | 4 | 0 | 0 | 4 | 0 |
| q14 | 4 | 0 | 0 | 4 | 0 |
| q15 | 4 | 0 | 0 | 4 | 0 |
| q16 | 4 | 1 | 0 | 3 | 0 |
| q17 | 4 | 1 | 0 | 3 | 0 |
| q18 | 4 | 1 | 0 | 3 | 0 |
| q19 | 4 | 1 | 0 | 3 | 0 |
| q20 | 2 | 0 | 0 | 2 | 0 |
| q21 | 3 | 2 | 0 | 1 | 0 |
| q22 | 4 | 0 | 0 | 4 | 0 |
| q23 | 4 | 0 | 0 | 4 | 0 |
| q24 | 4 | 0 | 0 | 4 | 0 |
| q25 | 4 | 0 | 0 | 4 | 0 |
| q26 | 3 | 0 | 0 | 3 | 0 |
| q27 | 4 | 0 | 0 | 4 | 0 |
| q28 | 5 | 0 | 0 | 5 | 0 |
| q29 | 5 | 0 | 0 | 5 | 0 |
| q30 | 3 | 3 | 0 | 0 | 0 |
| q31 | 4 | 0 | 0 | 4 | 0 |
| q32 | 4 | 0 | 0 | 4 | 0 |
| q33 | 4 | 1 | 0 | 3 | 0 |
| q34 | 4 | 1 | 0 | 3 | 0 |
| q35 | 4 | 1 | 0 | 3 | 0 |
| q36 | 5 | 1 | 0 | 4 | 0 |
| q37 | 4 | 0 | 0 | 4 | 0 |
| q38 | 4 | 0 | 0 | 4 | 0 |
| q39 | 4 | 0 | 0 | 4 | 0 |
| q40 | 4 | 0 | 0 | 4 | 0 |
| q41 | 4 | 0 | 0 | 4 | 0 |
| q42 | 4 | 0 | 0 | 4 | 0 |
| q43 | 4 | 0 | 0 | 4 | 0 |
| whole suite | 157 | 29 | 0 | 128 | 0 |

One row per query and one decision per operator, counted by what the planner had behind the cardinality it used. Over the whole suite that is 18% exact, 0% certified, 82% estimated and 0% unknown. `exact` is a counted number, `certified` is a number with a proven bound, `estimated` is a guess, and `unknown` is no number at all, which is an answer an operator has to handle rather than a null to paper over. Counts rather than fractions in the table because a suite's histogram is the sum of its queries' and fractions do not add.

#### Where the time went

| kind | cpu | share | operators | rows in | rows out | per row in | per row out | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Aggregate | 65.055s | 57.8% | 37 | 139120580 | 87321 | 467.6ns | 745012.6ns | 37 of 37 |
| Scan | 46.413s | 41.2% | 37 | 0 | 140628279 | handed none | 330.0ns | 37 of 37 |
| TopN | 855.648ms | 0.8% | 31 | 1594994 | 307 | 536.5ns | 2787128.2ns | 31 of 31 |
| TableFetch | 189.255ms | 0.2% | 1 | 10 | 10 | 18925515.1ns | 18925515.1ns | 1 of 1 |
| Project | 18.194ms | 0.0% | 46 | 1595681 | 1595681 | 11.4ns | 11.4ns | 46 of 46 |
| Sort | 6.159ms | 0.0% | 1 | 8 | 8 | 769818.0ns | 769818.0ns | 1 of 1 |
| Filter | 88.174us | 0.0% | 2 | 27 | 27 | 3265.7ns | 3265.7ns | 2 of 2 |
| Limit | 13.335us | 0.0% | 1 | 10 | 10 | 1333.5ns | 1333.5ns | 1 of 1 |
| Values | 11.191us | 0.0% | 1 | 0 | 1 | handed none | 11191.0ns | 1 of 1 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is Aggregate, and the queries where it cost the most are q29 at 43.684s, q19 at 3.886s, q28 at 2.477s.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 9999750 rows, one out of every 10 of the 99997497 in the full file, which is a development loop rather than the suite
- the one minute load average was 43.00 before this suite started, on a machine with 8 hardware threads, so this was measured against somebody else's work
- q1 swung by 23.7% of its median, and rule two wants under 10%
- q2 swung by 37.2% of its median, and rule two wants under 10%
- q3 swung by 13.4% of its median, and rule two wants under 10%
- q4 swung by 83.9% of its median, and rule two wants under 10%
- q5 swung by 30.7% of its median, and rule two wants under 10%
- q6 swung by 14.9% of its median, and rule two wants under 10%
- q7 swung by 45.0% of its median, and rule two wants under 10%
- q8 swung by 37.5% of its median, and rule two wants under 10%
- q10 swung by 23.4% of its median, and rule two wants under 10%
- q11 swung by 41.7% of its median, and rule two wants under 10%
- q12 swung by 53.6% of its median, and rule two wants under 10%
- q13 swung by 27.5% of its median, and rule two wants under 10%
- q15 swung by 16.4% of its median, and rule two wants under 10%
- q18 swung by 35.3% of its median, and rule two wants under 10%
- q19 swung by 19.4% of its median, and rule two wants under 10%
- q20 swung by 77.0% of its median, and rule two wants under 10%
- q21 swung by 17.6% of its median, and rule two wants under 10%
- q22 swung by 11.7% of its median, and rule two wants under 10%
- q24 swung by 20.7% of its median, and rule two wants under 10%
- q25 swung by 38.9% of its median, and rule two wants under 10%
- q26 swung by 27.3% of its median, and rule two wants under 10%
- q27 swung by 51.2% of its median, and rule two wants under 10%
- q29 swung by 12.6% of its median, and rule two wants under 10%
- q30 swung by 13.6% of its median, and rule two wants under 10%
- q32 swung by 23.1% of its median, and rule two wants under 10%
- q34 swung by 53.3% of its median, and rule two wants under 10%
- q37 swung by 41.2% of its median, and rule two wants under 10%
- q38 swung by 40.3% of its median, and rule two wants under 10%
- q40 swung by 11.8% of its median, and rule two wants under 10%
- q41 swung by 25.1% of its median, and rule two wants under 10%
- q42 swung by 22.3% of its median, and rule two wants under 10%
- q43 swung by 61.3% of its median, and rule two wants under 10%

These swung wider than reporting rule two allows:

- duckdb swung by 83.9% of its median on q4, and rule two wants under 10%
- clickhouse-local swung by 115.0% of its median on q15, and rule two wants under 10%
- rudb swung by 94.4% of its median on q8, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

All 3 engines agreed on every answer the data settles, which is 32 of 43 queries, to the last significant digit of a double.

These were answered differently and the data does not say which is right:

- q18: LIMIT 10 with no ORDER BY at all, so the ten rows are whichever ten the group by handed back first and the query did not ask for any particular ten.
- q22: ORDER BY COUNT(*) DESC LIMIT 10 over search phrases whose URL matches, where the counts at the cut are twos and ones, so which phrases fill the ten is the engine's choice. Both engines returned 4, 3, 2, 2, 2, 2, 2, 2, 1, 1 on a ten million row sample and kept different phrases for the tied places.
- q24: ORDER BY EventTime LIMIT 10, and EventTime has one second resolution over a corpus with millions of rows a second, so the tenth row is a tie.
- q29: ORDER BY an AVG of a length DESC LIMIT 25, which is both a tie at the cut and a double computed in a different order by each engine.
- q32: ORDER BY COUNT(*) DESC LIMIT 10 with ties at the cut, as q16.
- q33: ORDER BY COUNT(*) DESC LIMIT 10 over WatchID and ClientIP unfiltered, where almost every group has a count of one and the ten that come back are arbitrary.
- q39: ORDER BY PageViews DESC LIMIT 10 OFFSET 1000, which is a tie at the cut a thousand rows deeper in, where the counts are smaller and the ties are denser.
- q40: ORDER BY PageViews DESC LIMIT 10 OFFSET 1000 with ties at the cut, as q39.
- q41: ORDER BY PageViews DESC LIMIT 10 OFFSET 100 with ties at the cut, as q39.
- q43: DATE_TRUNC on a timestamp, and ClickHouse renders a DateTime in the machine's timezone where DuckDB renders a TIMESTAMP in none, so the same epoch second prints seven hours apart on gamingpc-wsl and two hours apart on server3.

So they are not checked, and a wrong answer from any engine on one of them would go unnoticed here. Every other query in the suite is checked in full.

These agreed on the rows and not on the order they came back in:

- q19: rudb
- q23: clickhouse-local, rudb

An ORDER BY that does not totally order its rows lets two correct engines answer this way, so it is not a failure. It is also not a full check: an engine that returned the right rows in the wrong order passes one of these and would fail every other query in the suite.

These were answered differently and which engine is wrong is settled:

- q4: AVG(UserID) over a hundred million bigints, where the pinned DuckDB sums the column short by a multiple of 2^64 and rudb does not. Its own hugeint and decimal paths, the same column summed out of a table, its own per counter group sums, and adding the column up outside a database all give rudb's answer, and it reduces to two statements over a 1.3 MB one column parquet file. Written up as 14.10.1 in tamnd/rudb `spec/14-testing.md`, which is the list of places where DuckDB is wrong and we are not. tamnd/rudb-compat#12.

So they are a known difference rather than an open one, and the write up is where to go to disagree with that.

Hot here means page cache warm and not buffer pool warm, because every run is a fresh process so that no query's number depends on the one before it.

