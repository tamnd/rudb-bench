# clickbench on server2

This is one run of the clickbench suite on server2, over 3 engines and 43 queries, with 3 tries of each query after a page cache drop, the way the upstream ClickBench driver runs it. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

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
| summary | the best of the tries after the first, which is the upstream ClickBench convention |
| timeout | 600s per query, and no query reached it |
| harness | rudb-bench 0.0.1 |

## How it was measured

Every engine loaded the data first, one at a time. Then each query was run on every engine in turn, and the order rotated by one engine per query, so no engine always went first or last. Before each engine's turn the harness waited for the one minute load average to be below the 6 hardware threads, dropped the page cache (and for the ClickHouse server stopped it first and started it again after), then ran the query 3 times in a row. The first try is the cold figure and the best of the other 2 is the hot one, which is what the upstream ClickBench driver does. Every figure below is the engine's own timing where it reports one.

Every engine got the same memory budget, 9.34 GiB (10033194592 bytes, from 80% of MemTotal). DuckDB and rudb got it as `SET memory_limit`, the ClickHouse server as `max_server_memory_usage` and `clickhouse local` as `max_memory_usage`.

The data file, which is also written beside it as a manifest:

| path | rows | bytes | sha256 |
| --- | --- | --- | --- |
| `/root/rudb-data/hits-10m.parquet` | 9999750 | 1498936215 | `988229f0b8f3f2e2ce901ceffb61e6ae0eb0d2cecca354f5c8c71815b2395edd` |

The one minute load average at the start of each engine's turns, counting its load and every query:

| engine | readings | lowest | median | highest | held by the gate | went first |
| --- | --- | --- | --- | --- | --- | --- |
| duckdb | 44 | 3.00 | 4.68 | 5.95 | 270.015s | 15 |
| clickhouse-server | 44 | 2.98 | 4.66 | 5.88 | 330.017s | 14 |
| rudb | 44 | 2.98 | 4.68 | 5.88 | 240.008s | 14 |

The reading includes the tail of whatever ran just before, which is usually the previous engine's turn, because the one minute average takes about a minute to decay.

rudb writes summaries of each column when it loads a table, and it can answer some queries from those without reading the rows. This run turned that off with `SET stored_answers = false` before every rudb query (`RUDB_BENCH_STORED_ANSWERS=off`), so rudb read the rows the way the other engines did. Its metrics said it still answered from stored summaries for no query. The per query table marks any such query, and it also marks q1 to q7, whose shapes (counts, sums, averages, distinct counts and bounds over the whole table) are the ones a summary can answer. The headline below gives the ratios both with and without q1 to q7.

## Headline

| engine | hot total | cold total | hot geomean | total vs duckdb | geomean vs duckdb | total vs, without q1 to q7 | geomean vs, without q1 to q7 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 12.581s | 19.652s | 0.1172s | 1.000x | 1.000x | 1.000x | 1.000x |
| clickhouse-server | 7.079s | 17.475s | 0.0863s | 0.563x | 0.737x | 0.550x | 0.728x |
| rudb | 3.235s | 10.481s | 0.0396s | 0.257x | 0.338x | 0.241x | 0.294x |

Over the 43 queries every engine finished, 36 of them outside q1 to q7. Hot is the best of the tries after the first and cold is the first try, both by the engine's own clock. The geometric mean is over the hot figures. A ratio under 1 means faster than duckdb.

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
RUDB_BENCH_STORED_ANSWERS=off rudb-bench run clickbench --rows 10000000 --engines duckdb,clickhouse-server,rudb --runs 3 --protocol upstream --sample-file /root/rudb-data/hits-10m.parquet --timeout 600 --report
```

The data goes under `RUDB_BENCH_DATA`, which defaults to `~/rudb-data`, and the engines have to be on the path. An engine that is not gets a row in the abstentions below rather than being left out of the table.

## The machine

| fact | value | source |
| --- | --- | --- |
| host | vmi3112167 | read |
| os | Linux 6.8.0-136-generic x86_64 | read |
| cpu | AMD EPYC Processor (with IBPB) | read |
| threads | 6 | read |
| memory | 11.68 GiB | read |
| governor | no cpufreq here, the policy is not ours to see | not read |
| turbo | no intel_pstate here, the state is not ours to see | not read |
| filesystem | /dev/sda1 / ext4 rw,relatime,discard,errors=remount-ro,commit=30 0 0 | read |
| page cache | droppable, cold runs are cold | read |
| timer | /usr/bin/time GNU | read |

A fact that says it was not read is a fact about this machine and not a gap in the report. A field that had silently defaulted would be a lie that survived into it.

## The engines

| engine | version | state | load | load cpu | on disk | that size is | format | machine load |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | v1.4.1 (Andium) b390a7c376 | ran | 63.763s | 205.810s | 1.71 GiB | its own database file | its own | 5.20 to 7.25 |
| clickhouse-server | 26.10.1.642 | ran | 49.905s | not read | 1.03 GiB | its own MergeTree parts, sorted by hits by (CounterID, EventDate, UserID, EventTime, WatchID), as system.parts counts the active ones | its own | 5.42 to 7.06 |
| rudb | rudb 0.4.34 | ran | 40.209s | 146.160s | 1.07 GiB | its own database file | its own | 5.61 to 8.77 |
| duckdb-pinned | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| clickhouse-local | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| datafusion | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| polars | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

The machine load column is the one minute load average before and after that engine's suite, on a machine with 6 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

These engines started while the machine was already above half of that, which means they were measured against somebody else's work rather than on an idle box: duckdb, clickhouse-server, rudb. Their numbers are inflated by an amount nothing here can recover, and by a different amount each, depending on how much of the column was CPU bound. One case where that reading is unfair is a sweep that runs engines back to back, where the average has not had a minute to come down from the engine before.

The cold column is a run that had to go to the device. The page cache was dropped before each query's first run, the way official ClickBench does it.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 12.581s | 16.717s | +33% | 30.950s | 67.420s | 4.03 | 950.99 MiB | 49.25 MiB | 34.18M/s | 4.77 GiB/s | 1.00x |
| clickhouse-server | 7.079s | 14.600s | +106% | 24.783s | not read | not read | not read | not read | 60.74M/s | 8.48 GiB/s | 0.56x |
| rudb | 3.235s | 4.827s | +49% | 14.734s | 14.860s | 3.08 | 219.00 MiB | 3.07 MiB | 132.91M/s | 18.55 GiB/s | 0.26x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | clickhouse-server | rudb | rudb from stored summaries |
| --- | --- | --- | --- | --- | --- |
| q1 | count | 3.000ms | 3.000ms | 3.164ms | no, but the shape allows it |
| q2 | filtered count | 21.000ms | 3.000ms | 6.042ms | no, but the shape allows it |
| q3 | three aggregates | 26.000ms | 27.000ms | 15.009ms | no, but the shape allows it |
| q4 | average | 24.000ms | 22.000ms | 14.452ms | no, but the shape allows it |
| q5 | count distinct, high card | 137.000ms | 98.000ms | 80.067ms | no, but the shape allows it |
| q6 | count distinct, strings | 187.000ms | 214.000ms | 166.548ms | no, but the shape allows it |
| q7 | min and max of a date | 10.000ms | 16.000ms | 13.210ms | no, but the shape allows it |
| q8 | group by, low card | 20.000ms | 13.000ms | 6.857ms |  |
| q9 | group by and count distinct | 180.000ms | 127.000ms | 46.415ms |  |
| q10 | group by, several aggregates | 299.000ms | 152.000ms | 64.001ms |  |
| q11 | group by a string and count distinct | 64.000ms | 41.000ms | 18.448ms |  |
| q12 | group by two strings and count distinct | 67.000ms | 53.000ms | 22.055ms |  |
| q13 | group by a string and top k | 184.000ms | 180.000ms | 22.634ms |  |
| q14 | group by a string and count distinct | 308.000ms | 217.000ms | 58.604ms |  |
| q15 | group by two columns and top k | 226.000ms | 182.000ms | 55.239ms |  |
| q16 | group by, very high card | 150.000ms | 95.000ms | 33.265ms |  |
| q17 | group by two, very high card | 286.000ms | 278.000ms | 84.837ms |  |
| q18 | group by two, no ordering | 263.000ms | 170.000ms | 45.920ms |  |
| q19 | group by with an extract | 502.000ms | 418.000ms | 128.787ms |  |
| q20 | point lookup | 15.000ms | 4.000ms | 3.570ms |  |
| q21 | substring scan | 220.000ms | 110.000ms | 74.429ms |  |
| q22 | substring scan and group by | 350.000ms | 63.000ms | 126.132ms |  |
| q23 | two substring scans and group by | 1.313s | 476.000ms | 289.478ms |  |
| q24 | select star and top k | 839.000ms | 260.000ms | 174.131ms |  |
| q25 | top k by a date | 79.000ms | 41.000ms | 13.330ms |  |
| q26 | top k by a string | 47.000ms | 83.000ms | 36.932ms |  |
| q27 | top k by two columns | 44.000ms | 37.000ms | 23.032ms |  |
| q28 | group by with a string length | 213.000ms | 55.000ms | 106.679ms |  |
| q29 | group by a regular expression | 3.527s | 718.000ms | 722.751ms |  |
| q30 | ninety sums over one column | 40.000ms | 43.000ms | 14.694ms |  |
| q31 | group by two and several aggregates | 189.000ms | 115.000ms | 58.277ms |  |
| q32 | group by a high card pair | 203.000ms | 119.000ms | 63.065ms |  |
| q33 | group by a high card pair, unfiltered | 525.000ms | 468.000ms | 164.457ms |  |
| q34 | group by a long string | 643.000ms | 568.000ms | 51.526ms |  |
| q35 | group by a constant and a long string | 699.000ms | 584.000ms | 56.107ms |  |
| q36 | group by four expressions | 194.000ms | 92.000ms | 36.736ms |  |
| q37 | date range and group by a URL | 86.000ms | 312.000ms | 41.641ms |  |
| q38 | date range and group by a title | 33.000ms | 132.000ms | 25.922ms |  |
| q39 | date range, group by and offset | 65.000ms | 66.000ms | 29.732ms |  |
| q40 | date range, a case and a wide group by | 171.000ms | 262.000ms | 173.173ms |  |
| q41 | date range with an IN and a hash | 44.000ms | 35.000ms | 20.749ms |  |
| q42 | date range and a deep offset | 44.000ms | 51.000ms | 27.834ms |  |
| q43 | minute buckets over a date range | 41.000ms | 76.000ms | 15.360ms |  |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 3.000ms | 254.110ms | 42.176ms | 48.0% | 42.176ms | 62.433ms | 42.176ms | 62.433ms | 50.000ms | 24.88 MiB | 22.64 MiB | 237.09M/s |
| q2 | filtered count | 21.000ms | 315.670ms | 82.815ms | 3.8% | 82.815ms | 85.942ms | 82.815ms | 85.942ms | 110.000ms | 48.12 MiB | 45.69 MiB | 120.75M/s |
| q3 | three aggregates | 26.000ms | 316.699ms | 83.892ms | 1.7% | 83.892ms | 85.357ms | 83.892ms | 85.357ms | 170.000ms | 70.12 MiB | 67.21 MiB | 119.20M/s |
| q4 | average | 24.000ms | 271.183ms | 81.626ms | 1.9% | 81.626ms | 83.164ms | 81.626ms | 83.164ms | 180.000ms | 55.38 MiB | 54.05 MiB | 122.51M/s |
| q5 | count distinct, high card | 137.000ms | 454.770ms | 202.584ms | 0.4% | 202.584ms | 203.430ms | 202.584ms | 203.430ms | 800.000ms | 117.98 MiB | 53.94 MiB | 49.36M/s |
| q6 | count distinct, strings | 187.000ms | 715.412ms | 287.463ms | 17.3% | 287.463ms | 337.089ms | 287.463ms | 337.089ms | 1.210s | 260.41 MiB | 80.39 MiB | 34.79M/s |
| q7 | min and max of a date | 10.000ms | 299.633ms | 61.843ms | 0.4% | 61.843ms | 62.062ms | 61.843ms | 62.062ms | 70.000ms | 31.38 MiB | 29.33 MiB | 161.70M/s |
| q8 | group by, low card | 20.000ms | 271.053ms | 62.111ms | 3.4% | 62.111ms | 64.217ms | 62.111ms | 64.217ms | 120.000ms | 49.75 MiB | 47.18 MiB | 161.00M/s |
| q9 | group by and count distinct | 180.000ms | 492.746ms | 243.346ms | 17.2% | 243.346ms | 285.196ms | 243.346ms | 285.196ms | 1.090s | 146.52 MiB | 74.57 MiB | 41.09M/s |
| q10 | group by, several aggregates | 299.000ms | 742.036ms | 385.670ms | 5.4% | 385.670ms | 406.686ms | 385.670ms | 406.686ms | 1.700s | 193.38 MiB | 113.25 MiB | 25.93M/s |
| q11 | group by a string and count distinct | 64.000ms | 334.638ms | 143.750ms | 0.2% | 143.750ms | 143.975ms | 143.750ms | 143.975ms | 350.000ms | 94.00 MiB | 76.14 MiB | 69.56M/s |
| q12 | group by two strings and count distinct | 67.000ms | 291.644ms | 125.941ms | 16.3% | 125.941ms | 146.506ms | 125.941ms | 146.506ms | 320.000ms | 97.75 MiB | 79.39 MiB | 79.40M/s |
| q13 | group by a string and top k | 184.000ms | 563.916ms | 288.246ms | 14.9% | 288.246ms | 331.121ms | 288.246ms | 331.121ms | 1.210s | 270.79 MiB | 80.52 MiB | 34.69M/s |
| q14 | group by a string and count distinct | 308.000ms | 869.212ms | 426.946ms | 12.0% | 426.946ms | 478.199ms | 426.946ms | 478.199ms | 1.870s | 377.78 MiB | 108.40 MiB | 23.42M/s |
| q15 | group by two columns and top k | 226.000ms | 560.757ms | 321.251ms | 3.1% | 321.251ms | 331.099ms | 321.251ms | 331.099ms | 1.230s | 290.62 MiB | 94.80 MiB | 31.13M/s |
| q16 | group by, very high card | 150.000ms | 453.698ms | 223.770ms | 22.0% | 223.770ms | 272.992ms | 223.770ms | 272.992ms | 1.050s | 133.85 MiB | 55.94 MiB | 44.69M/s |
| q17 | group by two, very high card | 286.000ms | 617.697ms | 385.640ms | 5.9% | 385.640ms | 408.569ms | 385.640ms | 408.569ms | 1.650s | 330.58 MiB | 110.02 MiB | 25.93M/s |
| q18 | group by two, no ordering | 263.000ms | 619.916ms | 364.736ms | 6.0% | 364.736ms | 386.602ms | 364.736ms | 386.602ms | 1.620s | 325.95 MiB | 109.75 MiB | 27.42M/s |
| q19 | group by with an extract | 502.000ms | 951.098ms | 671.033ms | 21.8% | 671.033ms | 817.470ms | 671.033ms | 817.470ms | 3.510s | 600.75 MiB | 174.08 MiB | 14.90M/s |
| q20 | point lookup | 15.000ms | 228.447ms | 62.244ms | 0.7% | 62.244ms | 62.687ms | 62.244ms | 62.687ms | 80.000ms | 39.88 MiB | 38.63 MiB | 160.65M/s |
| q21 | substring scan | 220.000ms | 760.302ms | 325.414ms | 0.2% | 325.414ms | 326.039ms | 325.414ms | 326.039ms | 1.380s | 313.50 MiB | 300.57 MiB | 30.73M/s |
| q22 | substring scan and group by | 350.000ms | 1.113s | 470.257ms | 8.4% | 470.257ms | 509.949ms | 470.257ms | 509.949ms | 1.870s | 380.88 MiB | 348.44 MiB | 21.26M/s |
| q23 | two substring scans and group by | 1.313s | 2.126s | 1.498s | 10.7% | 1.498s | 1.659s | 1.498s | 1.659s | 3.290s | 652.57 MiB | 544.20 MiB | 6.67M/s |
| q24 | select star and top k | 839.000ms | 1.642s | 978.924ms | 10.5% | 978.924ms | 1.082s | 978.924ms | 1.082s | 1.770s | 393.25 MiB | 335.06 MiB | 10.22M/s |
| q25 | top k by a date | 79.000ms | 442.844ms | 165.423ms | 2.5% | 165.423ms | 169.612ms | 165.423ms | 169.612ms | 390.000ms | 92.50 MiB | 78.86 MiB | 60.45M/s |
| q26 | top k by a string | 47.000ms | 311.504ms | 102.923ms | 19.9% | 102.923ms | 123.398ms | 102.923ms | 123.398ms | 310.000ms | 92.50 MiB | 79.23 MiB | 97.16M/s |
| q27 | top k by two columns | 44.000ms | 312.441ms | 103.447ms | 0.0% | 103.447ms | 103.483ms | 103.447ms | 103.483ms | 250.000ms | 92.00 MiB | 79.21 MiB | 96.67M/s |
| q28 | group by with a string length | 213.000ms | 827.776ms | 326.004ms | 6.9% | 326.004ms | 348.588ms | 326.004ms | 348.588ms | 1.390s | 324.62 MiB | 304.99 MiB | 30.67M/s |
| q29 | group by a regular expression | 3.527s | 4.567s | 3.725s | 2.2% | 3.725s | 3.808s | 3.725s | 3.808s | 19.890s | 586.87 MiB | 324.89 MiB | 2.68M/s |
| q30 | ninety sums over one column | 40.000ms | 373.624ms | 103.606ms | 2.2% | 103.606ms | 105.859ms | 103.606ms | 105.859ms | 180.000ms | 52.00 MiB | 47.08 MiB | 96.52M/s |
| q31 | group by two and several aggregates | 189.000ms | 647.817ms | 285.899ms | 7.4% | 285.899ms | 307.108ms | 285.899ms | 307.108ms | 1.080s | 265.04 MiB | 163.33 MiB | 34.98M/s |
| q32 | group by a high card pair | 203.000ms | 699.011ms | 306.896ms | 6.3% | 306.896ms | 326.240ms | 306.896ms | 326.240ms | 1.200s | 349.75 MiB | 232.48 MiB | 32.58M/s |
| q33 | group by a high card pair, unfiltered | 525.000ms | 1.132s | 750.362ms | 8.4% | 750.362ms | 813.701ms | 750.362ms | 813.701ms | 3.480s | 846.74 MiB | 176.98 MiB | 13.33M/s |
| q34 | group by a long string | 643.000ms | 1.492s | 871.904ms | 8.8% | 871.904ms | 948.431ms | 871.904ms | 948.431ms | 4.290s | 927.18 MiB | 299.02 MiB | 11.47M/s |
| q35 | group by a constant and a long string | 699.000ms | 1.832s | 933.541ms | 16.5% | 933.541ms | 1.088s | 933.541ms | 1.088s | 4.790s | 950.99 MiB | 297.62 MiB | 10.71M/s |
| q36 | group by four expressions | 194.000ms | 508.043ms | 265.684ms | 15.7% | 265.684ms | 307.504ms | 265.684ms | 307.504ms | 1.240s | 140.62 MiB | 53.09 MiB | 37.64M/s |
| q37 | date range and group by a URL | 86.000ms | 441.362ms | 167.422ms | 2.0% | 167.422ms | 170.688ms | 167.422ms | 170.688ms | 440.000ms | 132.38 MiB | 49.16 MiB | 59.73M/s |
| q38 | date range and group by a title | 33.000ms | 250.588ms | 82.888ms | 25.6% | 82.888ms | 104.075ms | 82.888ms | 104.075ms | 200.000ms | 51.12 MiB | 32.41 MiB | 120.64M/s |
| q39 | date range, group by and offset | 65.000ms | 302.616ms | 125.305ms | 15.0% | 125.305ms | 144.067ms | 125.305ms | 144.067ms | 310.000ms | 68.62 MiB | 48.91 MiB | 79.80M/s |
| q40 | date range, a case and a wide group by | 171.000ms | 553.819ms | 271.533ms | 6.8% | 271.533ms | 289.887ms | 271.533ms | 289.887ms | 830.000ms | 216.38 MiB | 68.60 MiB | 36.83M/s |
| q41 | date range with an IN and a hash | 44.000ms | 315.230ms | 103.555ms | 3.4% | 103.555ms | 107.076ms | 103.555ms | 107.076ms | 140.000ms | 51.75 MiB | 42.47 MiB | 96.56M/s |
| q42 | date range and a deep offset | 44.000ms | 314.991ms | 102.330ms | 2.0% | 102.330ms | 104.421ms | 102.330ms | 104.421ms | 150.000ms | 45.88 MiB | 38.40 MiB | 97.72M/s |
| q43 | minute buckets over a date range | 41.000ms | 357.950ms | 103.497ms | 7.2% | 103.497ms | 110.989ms | 103.497ms | 110.989ms | 160.000ms | 40.50 MiB | 33.32 MiB | 96.62M/s |

duckdb v1.4.1 (Andium) b390a7c376 over 43 of 43 queries. Total 12.581s by its own clock and 16.717s by ours, 30.950s cold, 67.420s of CPU, peak 950.99 MiB, 34.18M/s and 4.77 GiB/s.

Running it cost 33% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 88.32x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-server in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 3.000ms | 183.969ms | 163.989ms | 0.4% | 163.989ms | 164.711ms | 163.989ms | 164.711ms | not read | not read | not read | 60.98M/s |
| q2 | filtered count | 3.000ms | 183.415ms | 163.417ms | 24.1% | 163.417ms | 202.876ms | 163.417ms | 202.876ms | not read | not read | not read | 61.19M/s |
| q3 | three aggregates | 27.000ms | 285.204ms | 247.101ms | 20.3% | 247.101ms | 297.220ms | 247.101ms | 297.220ms | not read | not read | not read | 40.47M/s |
| q4 | average | 22.000ms | 203.078ms | 163.258ms | 0.7% | 163.258ms | 164.374ms | 163.258ms | 164.374ms | not read | not read | not read | 61.25M/s |
| q5 | count distinct, high card | 98.000ms | 441.174ms | 306.443ms | 2.9% | 306.443ms | 315.276ms | 306.443ms | 315.276ms | not read | not read | not read | 32.63M/s |
| q6 | count distinct, strings | 214.000ms | 569.632ms | 393.522ms | 14.4% | 393.522ms | 450.007ms | 393.522ms | 450.007ms | not read | not read | not read | 25.41M/s |
| q7 | min and max of a date | 16.000ms | 227.768ms | 184.397ms | 10.6% | 184.397ms | 203.896ms | 184.397ms | 203.896ms | not read | not read | not read | 54.23M/s |
| q8 | group by, low card | 13.000ms | 204.895ms | 163.594ms | 13.4% | 163.594ms | 185.501ms | 163.594ms | 185.501ms | not read | not read | not read | 61.13M/s |
| q9 | group by and count distinct | 127.000ms | 431.999ms | 328.631ms | 30.3% | 328.631ms | 428.168ms | 328.631ms | 428.168ms | not read | not read | not read | 30.43M/s |
| q10 | group by, several aggregates | 152.000ms | 396.287ms | 287.215ms | 14.7% | 287.215ms | 329.446ms | 287.215ms | 329.446ms | not read | not read | not read | 34.82M/s |
| q11 | group by a string and count distinct | 41.000ms | 245.070ms | 188.297ms | 10.4% | 188.297ms | 207.790ms | 188.297ms | 207.790ms | not read | not read | not read | 53.11M/s |
| q12 | group by two strings and count distinct | 53.000ms | 265.344ms | 224.365ms | 1.6% | 224.365ms | 228.007ms | 224.365ms | 228.007ms | not read | not read | not read | 44.57M/s |
| q13 | group by a string and top k | 180.000ms | 364.096ms | 325.085ms | 0.9% | 325.085ms | 328.011ms | 325.085ms | 328.011ms | not read | not read | not read | 30.76M/s |
| q14 | group by a string and count distinct | 217.000ms | 449.059ms | 349.872ms | 16.3% | 349.872ms | 406.815ms | 349.872ms | 406.815ms | not read | not read | not read | 28.58M/s |
| q15 | group by two columns and top k | 182.000ms | 522.152ms | 364.282ms | 8.3% | 364.282ms | 394.576ms | 364.282ms | 394.576ms | not read | not read | not read | 27.45M/s |
| q16 | group by, very high card | 95.000ms | 265.448ms | 265.695ms | 15.0% | 265.695ms | 305.578ms | 265.695ms | 305.578ms | not read | not read | not read | 37.64M/s |
| q17 | group by two, very high card | 278.000ms | 530.305ms | 409.397ms | 20.5% | 409.397ms | 493.398ms | 409.397ms | 493.398ms | not read | not read | not read | 24.43M/s |
| q18 | group by two, no ordering | 170.000ms | 450.974ms | 306.115ms | 1.6% | 306.115ms | 311.077ms | 306.115ms | 311.077ms | not read | not read | not read | 32.67M/s |
| q19 | group by with an extract | 418.000ms | 750.938ms | 553.281ms | 18.6% | 553.281ms | 656.361ms | 553.281ms | 656.361ms | not read | not read | not read | 18.07M/s |
| q20 | point lookup | 4.000ms | 182.714ms | 141.810ms | 0.6% | 141.810ms | 142.703ms | 141.810ms | 142.703ms | not read | not read | not read | 70.52M/s |
| q21 | substring scan | 110.000ms | 726.930ms | 268.367ms | 13.7% | 268.367ms | 305.105ms | 268.367ms | 305.105ms | not read | not read | not read | 37.26M/s |
| q22 | substring scan and group by | 63.000ms | 895.221ms | 247.560ms | 28.7% | 247.560ms | 318.616ms | 247.560ms | 318.616ms | not read | not read | not read | 40.39M/s |
| q23 | two substring scans and group by | 476.000ms | 2.967s | 693.625ms | 4.0% | 693.625ms | 721.361ms | 693.625ms | 721.361ms | not read | not read | not read | 14.42M/s |
| q24 | select star and top k | 260.000ms | 2.107s | 448.769ms | 45.6% | 448.769ms | 653.415ms | 448.769ms | 653.415ms | not read | not read | not read | 22.28M/s |
| q25 | top k by a date | 41.000ms | 306.350ms | 225.200ms | 21.1% | 225.200ms | 272.715ms | 225.200ms | 272.715ms | not read | not read | not read | 44.40M/s |
| q26 | top k by a string | 83.000ms | 289.788ms | 243.137ms | 10.0% | 243.137ms | 267.502ms | 243.137ms | 267.502ms | not read | not read | not read | 41.13M/s |
| q27 | top k by two columns | 37.000ms | 347.403ms | 206.960ms | 18.4% | 206.960ms | 245.026ms | 206.960ms | 245.026ms | not read | not read | not read | 48.32M/s |
| q28 | group by with a string length | 55.000ms | 266.462ms | 204.009ms | 11.8% | 204.009ms | 228.024ms | 204.009ms | 228.024ms | not read | not read | not read | 49.02M/s |
| q29 | group by a regular expression | 718.000ms | 1.685s | 915.124ms | 6.4% | 915.124ms | 973.730ms | 915.124ms | 973.730ms | not read | not read | not read | 10.93M/s |
| q30 | ninety sums over one column | 43.000ms | 309.393ms | 226.531ms | 26.3% | 226.531ms | 286.135ms | 226.531ms | 286.135ms | not read | not read | not read | 44.14M/s |
| q31 | group by two and several aggregates | 115.000ms | 390.099ms | 318.347ms | 28.9% | 318.347ms | 410.441ms | 318.347ms | 410.441ms | not read | not read | not read | 31.41M/s |
| q32 | group by a high card pair | 119.000ms | 451.615ms | 264.453ms | 8.9% | 264.453ms | 287.886ms | 264.453ms | 287.886ms | not read | not read | not read | 37.81M/s |
| q33 | group by a high card pair, unfiltered | 468.000ms | 799.875ms | 676.796ms | 10.5% | 676.796ms | 747.794ms | 676.796ms | 747.794ms | not read | not read | not read | 14.78M/s |
| q34 | group by a long string | 568.000ms | 1.610s | 752.169ms | 5.4% | 752.169ms | 792.509ms | 752.169ms | 792.509ms | not read | not read | not read | 13.29M/s |
| q35 | group by a constant and a long string | 584.000ms | 1.100s | 729.144ms | 0.7% | 729.144ms | 734.153ms | 729.144ms | 734.153ms | not read | not read | not read | 13.71M/s |
| q36 | group by four expressions | 92.000ms | 310.547ms | 244.977ms | 51.2% | 244.977ms | 370.305ms | 244.977ms | 370.305ms | not read | not read | not read | 40.82M/s |
| q37 | date range and group by a URL | 312.000ms | 730.617ms | 567.916ms | 43.2% | 567.916ms | 813.252ms | 567.916ms | 813.252ms | not read | not read | not read | 17.61M/s |
| q38 | date range and group by a title | 132.000ms | 367.483ms | 384.176ms | 6.3% | 384.176ms | 408.463ms | 384.176ms | 408.463ms | not read | not read | not read | 26.03M/s |
| q39 | date range, group by and offset | 66.000ms | 286.363ms | 225.321ms | 18.3% | 225.321ms | 266.470ms | 225.321ms | 266.470ms | not read | not read | not read | 44.38M/s |
| q40 | date range, a case and a wide group by | 262.000ms | 729.302ms | 407.344ms | 15.4% | 407.344ms | 470.012ms | 407.344ms | 470.012ms | not read | not read | not read | 24.55M/s |
| q41 | date range with an IN and a hash | 35.000ms | 264.430ms | 184.116ms | 10.2% | 184.116ms | 202.890ms | 184.116ms | 202.890ms | not read | not read | not read | 54.31M/s |
| q42 | date range and a deep offset | 51.000ms | 303.514ms | 268.543ms | 11.0% | 268.543ms | 298.201ms | 268.543ms | 298.201ms | not read | not read | not read | 37.24M/s |
| q43 | minute buckets over a date range | 76.000ms | 385.086ms | 367.171ms | 21.4% | 367.171ms | 445.564ms | 367.171ms | 445.564ms | not read | not read | not read | 27.23M/s |

clickhouse-server 26.10.1.642 over 43 of 43 queries. Total 7.079s by its own clock and 14.600s by ours, 24.783s cold, no reading of CPU, peak not read, 60.74M/s and 8.48 GiB/s.

Running it cost 106% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 6.45x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 3.164ms | 111.807ms | 21.343ms | 94.9% | 21.343ms | 41.588ms | 21.343ms | 41.588ms | 10.000ms | 13.25 MiB | 11.07 MiB | 468.53M/s |
| q2 | filtered count | 6.042ms | 146.380ms | 41.548ms | 3.2% | 41.548ms | 42.863ms | 41.548ms | 42.863ms | 30.000ms | 14.88 MiB | 13.58 MiB | 240.68M/s |
| q3 | three aggregates | 15.009ms | 253.331ms | 43.050ms | 1.7% | 43.050ms | 43.769ms | 43.050ms | 43.769ms | 80.000ms | 15.00 MiB | 22.00 MiB | 232.28M/s |
| q4 | average | 14.452ms | 213.639ms | 42.101ms | 47.9% | 42.101ms | 62.265ms | 42.101ms | 62.265ms | 100.000ms | 15.25 MiB | 35.87 MiB | 237.52M/s |
| q5 | count distinct, high card | 80.067ms | 291.770ms | 123.855ms | 16.8% | 123.855ms | 144.722ms | 123.855ms | 144.722ms | 460.000ms | 131.12 MiB | 34.79 MiB | 80.74M/s |
| q6 | count distinct, strings | 166.548ms | 376.048ms | 206.983ms | 21.3% | 206.983ms | 251.059ms | 206.983ms | 251.059ms | 810.000ms | 51.62 MiB | 31.91 MiB | 48.31M/s |
| q7 | min and max of a date | 13.210ms | 152.104ms | 42.144ms | 4.7% | 42.144ms | 44.146ms | 42.144ms | 44.146ms | 70.000ms | 14.00 MiB | 12.41 MiB | 237.27M/s |
| q8 | group by, low card | 6.857ms | 131.205ms | 41.636ms | 12.5% | 41.636ms | 46.852ms | 41.636ms | 46.852ms | 60.000ms | 16.00 MiB | 13.48 MiB | 240.17M/s |
| q9 | group by and count distinct | 46.415ms | 231.480ms | 83.364ms | 8.4% | 83.364ms | 90.355ms | 83.364ms | 90.355ms | 280.000ms | 49.75 MiB | 43.40 MiB | 119.95M/s |
| q10 | group by, several aggregates | 64.001ms | 362.257ms | 102.815ms | 1.3% | 102.815ms | 104.193ms | 102.815ms | 104.193ms | 340.000ms | 51.62 MiB | 54.80 MiB | 97.26M/s |
| q11 | group by a string and count distinct | 18.448ms | 194.117ms | 41.771ms | 47.9% | 41.771ms | 61.773ms | 41.771ms | 61.773ms | 90.000ms | 19.12 MiB | 37.98 MiB | 239.39M/s |
| q12 | group by two strings and count distinct | 22.055ms | 189.248ms | 64.099ms | 0.5% | 64.099ms | 64.411ms | 64.099ms | 64.411ms | 110.000ms | 20.12 MiB | 39.58 MiB | 156.00M/s |
| q13 | group by a string and top k | 22.634ms | 213.489ms | 63.111ms | 0.5% | 63.111ms | 63.417ms | 63.111ms | 63.417ms | 100.000ms | 26.50 MiB | 32.09 MiB | 158.45M/s |
| q14 | group by a string and count distinct | 58.604ms | 382.748ms | 111.272ms | 13.1% | 111.272ms | 125.881ms | 111.272ms | 125.881ms | 420.000ms | 44.88 MiB | 59.42 MiB | 89.87M/s |
| q15 | group by two columns and top k | 55.239ms | 324.308ms | 83.227ms | 28.7% | 83.227ms | 107.080ms | 83.227ms | 107.080ms | 280.000ms | 55.38 MiB | 42.68 MiB | 120.15M/s |
| q16 | group by, very high card | 33.265ms | 193.555ms | 66.011ms | 26.4% | 66.011ms | 83.410ms | 66.011ms | 83.410ms | 190.000ms | 50.38 MiB | 35.46 MiB | 151.49M/s |
| q17 | group by two, very high card | 84.837ms | 315.655ms | 123.847ms | 0.3% | 123.847ms | 124.192ms | 123.847ms | 124.192ms | 450.000ms | 84.75 MiB | 54.96 MiB | 80.74M/s |
| q18 | group by two, no ordering | 45.920ms | 212.772ms | 84.016ms | 1.1% | 84.016ms | 84.917ms | 84.016ms | 84.917ms | 260.000ms | 19.12 MiB | 54.14 MiB | 119.02M/s |
| q19 | group by with an extract | 128.787ms | 397.828ms | 164.911ms | 12.3% | 164.911ms | 185.186ms | 164.911ms | 185.186ms | 760.000ms | 156.75 MiB | 84.45 MiB | 60.64M/s |
| q20 | point lookup | 3.570ms | 130.045ms | 22.026ms | 94.8% | 22.026ms | 42.898ms | 22.026ms | 42.898ms | 10.000ms | 14.12 MiB | 12.82 MiB | 453.99M/s |
| q21 | substring scan | 74.429ms | 353.860ms | 102.841ms | 0.5% | 102.841ms | 103.381ms | 102.841ms | 103.381ms | 360.000ms | 40.88 MiB | 91.63 MiB | 97.23M/s |
| q22 | substring scan and group by | 126.132ms | 515.414ms | 165.039ms | 14.4% | 165.039ms | 188.869ms | 165.039ms | 188.869ms | 520.000ms | 52.25 MiB | 111.21 MiB | 60.59M/s |
| q23 | two substring scans and group by | 289.478ms | 1.297s | 331.978ms | 0.9% | 331.978ms | 335.095ms | 331.978ms | 335.095ms | 960.000ms | 66.38 MiB | 227.34 MiB | 30.12M/s |
| q24 | select star and top k | 174.131ms | 872.543ms | 228.142ms | 1.0% | 228.142ms | 230.403ms | 228.142ms | 230.403ms | 420.000ms | 59.75 MiB | 121.03 MiB | 43.83M/s |
| q25 | top k by a date | 13.330ms | 231.157ms | 45.412ms | 3.9% | 45.412ms | 47.189ms | 45.412ms | 47.189ms | 50.000ms | 17.88 MiB | 18.17 MiB | 220.20M/s |
| q26 | top k by a string | 36.932ms | 250.163ms | 62.598ms | 31.8% | 62.598ms | 82.493ms | 62.598ms | 82.493ms | 120.000ms | 23.38 MiB | 37.86 MiB | 159.75M/s |
| q27 | top k by two columns | 23.032ms | 168.588ms | 62.110ms | 0.3% | 62.110ms | 62.288ms | 62.110ms | 62.288ms | 50.000ms | 21.12 MiB | 21.70 MiB | 161.00M/s |
| q28 | group by with a string length | 106.679ms | 314.901ms | 143.014ms | 1.4% | 143.014ms | 145.029ms | 143.014ms | 145.029ms | 440.000ms | 39.00 MiB | 54.23 MiB | 69.92M/s |
| q29 | group by a regular expression | 722.751ms | 1.544s | 792.251ms | 3.1% | 792.251ms | 817.083ms | 792.251ms | 817.083ms | 3.600s | 219.00 MiB | 218.04 MiB | 12.62M/s |
| q30 | ninety sums over one column | 14.694ms | 234.413ms | 43.054ms | 1.1% | 43.054ms | 43.548ms | 43.054ms | 43.548ms | 60.000ms | 15.62 MiB | 22.60 MiB | 232.26M/s |
| q31 | group by two and several aggregates | 58.277ms | 417.082ms | 105.198ms | 1.1% | 105.198ms | 106.312ms | 105.198ms | 106.312ms | 320.000ms | 53.25 MiB | 68.75 MiB | 95.06M/s |
| q32 | group by a high card pair | 63.065ms | 525.509ms | 105.354ms | 1.3% | 105.354ms | 106.716ms | 105.354ms | 106.716ms | 360.000ms | 56.88 MiB | 149.58 MiB | 94.92M/s |
| q33 | group by a high card pair, unfiltered | 164.457ms | 498.450ms | 224.259ms | 10.9% | 224.259ms | 248.653ms | 224.259ms | 248.653ms | 990.000ms | 196.75 MiB | 129.77 MiB | 44.59M/s |
| q34 | group by a long string | 51.526ms | 259.750ms | 82.987ms | 2.6% | 82.987ms | 85.119ms | 82.987ms | 85.119ms | 220.000ms | 70.62 MiB | 54.15 MiB | 120.50M/s |
| q35 | group by a constant and a long string | 56.107ms | 323.775ms | 104.571ms | 6.4% | 104.571ms | 111.299ms | 104.571ms | 111.299ms | 330.000ms | 70.38 MiB | 53.70 MiB | 95.63M/s |
| q36 | group by four expressions | 36.736ms | 256.587ms | 85.737ms | 0.2% | 85.737ms | 85.947ms | 85.737ms | 85.947ms | 210.000ms | 50.12 MiB | 30.07 MiB | 116.63M/s |
| q37 | date range and group by a URL | 41.641ms | 369.238ms | 84.523ms | 30.4% | 84.523ms | 110.260ms | 84.523ms | 110.260ms | 80.000ms | 34.12 MiB | 23.00 MiB | 118.31M/s |
| q38 | date range and group by a title | 25.922ms | 239.809ms | 61.816ms | 3.8% | 61.816ms | 64.187ms | 61.816ms | 64.187ms | 70.000ms | 29.88 MiB | 20.08 MiB | 161.77M/s |
| q39 | date range, group by and offset | 29.732ms | 192.522ms | 64.917ms | 27.5% | 64.917ms | 82.771ms | 64.917ms | 82.771ms | 70.000ms | 24.25 MiB | 23.12 MiB | 154.04M/s |
| q40 | date range, a case and a wide group by | 173.173ms | 446.830ms | 207.337ms | 18.6% | 207.337ms | 245.875ms | 207.337ms | 245.875ms | 540.000ms | 69.38 MiB | 35.03 MiB | 48.23M/s |
| q41 | date range with an IN and a hash | 20.749ms | 228.535ms | 43.944ms | 42.1% | 43.944ms | 62.453ms | 43.944ms | 62.453ms | 70.000ms | 24.25 MiB | 24.50 MiB | 227.56M/s |
| q42 | date range and a deep offset | 27.834ms | 194.891ms | 64.456ms | 0.7% | 64.456ms | 64.932ms | 64.456ms | 64.932ms | 50.000ms | 24.62 MiB | 19.92 MiB | 155.14M/s |
| q43 | minute buckets over a date range | 15.360ms | 175.608ms | 42.129ms | 0.1% | 42.129ms | 42.188ms | 42.129ms | 42.188ms | 60.000ms | 21.12 MiB | 15.31 MiB | 237.36M/s |

rudb rudb 0.4.34 over 43 of 43 queries. Total 3.235s by its own clock and 4.827s by ours, 14.734s cold, 14.860s of CPU, peak 219.00 MiB, 132.91M/s and 18.55 GiB/s.

Running it cost 49% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 37.12x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | moved | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | 18.123ms | 10.065ms | 6.053ms | 6.557ms | 7.7% | 6.053ms | 3.004ms | 30.439ms | 1.25 KiB | 9999751.0x | 3 of 3 |
| q2 | 28.169ms | 38.936ms | 53.036ms | 53.975ms | 1.7% | 53.036ms | 3.384ms | 32.641ms | 896 B | 202135.0x | 3 of 3 |
| q3 | 21.451ms | 145.500ms | 239.611ms | 240.932ms | 0.5% | 239.611ms | 3.067ms | 26.001ms | 1.15 KiB | 9999751.0x | 3 of 3 |
| q4 | 28.860ms | 79.210ms | 155.861ms | 156.609ms | 0.5% | 155.861ms | 2.916ms | 40.475ms | 896 B | 9999751.0x | 3 of 3 |
| q5 | 24.369ms | 144.824ms | 415.780ms | 416.613ms | 0.2% | 415.780ms | 4.814ms | 228.573ms | 98.79 MiB | 9999751.0x | 3 of 3 |
| q6 | 24.108ms | 247.677ms | 799.858ms | 800.787ms | 0.1% | 799.858ms | 3.510ms | 55.702ms | 29.46 MiB | 10834794.0x | 4 of 4 |
| q7 | 20.159ms | 39.446ms | 87.692ms | 88.236ms | 0.6% | 87.692ms | 2.519ms | 29.245ms | 1.02 KiB | 9999751.0x | 3 of 3 |
| q8 | 29.068ms | 25.453ms | 49.153ms | 49.624ms | 0.9% | 49.153ms | 2.445ms | 27.931ms | 2.77 KiB | 25268.8x | 4 of 4 |
| q9 | 17.733ms | 140.454ms | 381.721ms | 383.372ms | 0.4% | 381.721ms | 1.913ms | 64.715ms | 29.35 MiB | 1000103.0x | 4 of 4 |
| q10 | 33.146ms | 226.518ms | 573.318ms | 574.635ms | 0.2% | 573.318ms | 3.727ms | 91.638ms | 29.87 MiB | 1000103.0x | 4 of 4 |
| q11 | 29.069ms | 85.259ms | 189.307ms | 189.923ms | 0.3% | 189.307ms | 3.537ms | 36.540ms | 2.05 MiB | 35001.1x | 4 of 4 |
| q12 | 29.704ms | 84.873ms | 189.610ms | 190.052ms | 0.2% | 189.610ms | 2.633ms | 37.315ms | 2.00 MiB | 35013.5x | 4 of 4 |
| q13 | 32.189ms | 86.542ms | 166.799ms | 167.901ms | 0.7% | 166.799ms | 2.954ms | 29.146ms | 12.09 MiB | 137414.9x | 4 of 4 |
| q14 | 36.005ms | 234.493ms | 405.961ms | 406.826ms | 0.2% | 405.961ms | 4.348ms | 188.826ms | 33.71 MiB | 137534.9x | 4 of 4 |
| q15 | 41.574ms | 175.193ms | 350.983ms | 351.689ms | 0.2% | 350.983ms | 4.707ms | 173.604ms | 33.37 MiB | 137534.9x | 4 of 4 |
| q16 | 26.376ms | 73.563ms | 184.543ms | 185.142ms | 0.3% | 184.543ms | 1.890ms | 62.969ms | 28.28 MiB | 1000103.0x | 4 of 4 |
| q17 | 33.188ms | 180.581ms | 528.393ms | 528.850ms | 0.1% | 528.393ms | 3.017ms | 158.133ms | 75.65 MiB | 1000103.0x | 4 of 4 |
| q18 | 23.026ms | 117.307ms | 346.944ms | 347.567ms | 0.2% | 346.944ms | 2.351ms | 30.082ms | 388.35 KiB | 999977.0x | 4 of 4 |
| q19 | 32.157ms | 267.915ms | 867.499ms | 868.246ms | 0.1% | 867.499ms | 2.656ms | 219.098ms | 169.50 MiB | 1000103.0x | 4 of 4 |
| q20 | 23.473ms | 20.679ms | 22.264ms | 22.931ms | 2.9% | 22.264ms | 2.903ms | 34.167ms | 0 B | not read | 2 of 2 |
| q21 | 18.805ms | 250.337ms | 578.193ms | 578.685ms | 0.1% | 578.193ms | 3.081ms | 28.235ms | 896 B | 647.0x | 3 of 3 |
| q22 | 35.008ms | 377.266ms | 845.885ms | 846.506ms | 0.1% | 845.885ms | 5.601ms | 37.893ms | 503.48 KiB | 6.4x | 4 of 4 |
| q23 | 45.123ms | 1.136s | 2.200s | 2.202s | 0.1% | 2.200s | 8.658ms | 59.587ms | 655.59 KiB | 111.1x | 4 of 4 |
| q24 | 36.885ms | 680.662ms | 488.196ms | 501.402ms | 2.6% | 488.196ms | 3.043ms | 175.555ms | 30.18 KiB | 64.0x | 4 of 4 |
| q25 | 26.234ms | 101.230ms | 64.831ms | 84.517ms | 23.3% | 64.831ms | 5.364ms | 40.119ms | 21.44 KiB | 15058.2x | 4 of 4 |
| q26 | 22.709ms | 149.166ms | 208.573ms | 220.371ms | 5.4% | 208.573ms | 3.702ms | 25.927ms | 13.22 KiB | 274813.8x | 3 of 3 |
| q27 | 17.852ms | 73.760ms | 53.092ms | 63.025ms | 15.8% | 53.092ms | 2.460ms | 24.515ms | 25.19 KiB | 14210.0x | 4 of 4 |
| q28 | 40.269ms | 182.106ms | 454.299ms | 455.405ms | 0.2% | 454.299ms | 4.525ms | 60.069ms | 18.37 KiB | 833261.8x | 5 of 5 |
| q29 | 28.855ms | 1.375s | 4.760s | 4.761s | 0.0% | 4.760s | 4.399ms | 134.948ms | 78.06 MiB | 578849.7x | 5 of 5 |
| q30 | 45.065ms | 82.342ms | 135.990ms | 137.293ms | 0.9% | 135.990ms | 6.743ms | 45.965ms | 13.04 KiB | 9999752.0x | 4 of 4 |
| q31 | 35.961ms | 272.792ms | 572.188ms | 572.889ms | 0.1% | 572.188ms | 4.251ms | 142.860ms | 34.91 MiB | 137534.9x | 4 of 4 |
| q32 | 37.820ms | 393.353ms | 872.774ms | 873.468ms | 0.1% | 872.774ms | 5.955ms | 110.577ms | 36.28 MiB | 137534.9x | 4 of 4 |
| q33 | 26.514ms | 344.533ms | 990.841ms | 991.590ms | 0.1% | 990.841ms | 3.209ms | 455.201ms | 206.20 MiB | 1000103.0x | 4 of 4 |
| q34 | 31.831ms | 121.083ms | 272.870ms | 273.424ms | 0.2% | 272.870ms | 3.591ms | 72.986ms | 58.54 MiB | 999983.0x | 4 of 4 |
| q35 | 27.871ms | 177.999ms | 425.173ms | 425.932ms | 0.2% | 425.173ms | 2.256ms | 91.812ms | 58.55 MiB | 999983.0x | 4 of 4 |
| q36 | 40.182ms | 112.660ms | 297.147ms | 297.995ms | 0.3% | 297.147ms | 6.306ms | 95.699ms | 31.05 MiB | 1000167.0x | 5 of 5 |
| q37 | 72.682ms | 151.693ms | 79.791ms | 81.138ms | 1.7% | 79.791ms | 3.748ms | 65.114ms | 22.51 MiB | 55817.7x | 4 of 4 |
| q38 | 40.287ms | 100.308ms | 152.528ms | 153.843ms | 0.9% | 152.528ms | 6.783ms | 59.374ms | 14.79 MiB | 54895.3x | 4 of 4 |
| q39 | 27.005ms | 74.009ms | 72.297ms | 72.799ms | 0.7% | 72.297ms | 3.175ms | 44.026ms | 20.20 MiB | 4734.2x | 4 of 4 |
| q40 | 48.843ms | 265.325ms | 511.792ms | 513.014ms | 0.2% | 511.792ms | 6.279ms | 100.707ms | 23.64 MiB | 72956.8x | 4 of 4 |
| q41 | 35.983ms | 71.900ms | 117.550ms | 118.759ms | 1.0% | 117.550ms | 4.713ms | 56.528ms | 3.53 MiB | 7512.5x | 4 of 4 |
| q42 | 34.785ms | 75.515ms | 97.581ms | 98.439ms | 0.9% | 97.581ms | 6.655ms | 34.906ms | 5.50 MiB | not read | 4 of 4 |
| q43 | 32.272ms | 42.684ms | 84.684ms | 85.281ms | 0.7% | 84.684ms | 3.445ms | 51.274ms | 684.80 KiB | 56098.5x | 4 of 4 |

Read from the breakdown the engine wrote for its cold run. `planning` is everything before the first row moved, which is the parse, the bind, the optimizer passes and building the tree. It is a column rather than a footnote because it is the one cost of a query nobody profiles: an optimizer only ever has passes added to it, each paying for itself on the query it was written for, and a query that plans for four hundred milliseconds to save two hundred is a query the optimizer made slower. What is gated is planning as a share of the whole statement, and that share is not in this table, because it is taken over every run rather than off the cold one the rest of these columns come from. It lives in `baselines/planning-<suite>.txt`. `accounted` is what its pipelines charged themselves and `measured` is what it measured around running them, so `apart` is the cross check and anything over five percent is time the breakdown cannot explain. `driver` is the part of `accounted` that was not inside an operator, which is the loop that runs a pipeline rather than the operators it calls. `build` is what the engine spent putting the tree together, which is before there is a pipeline to charge and is why it is off the right hand side of the check. `outside` is the CPU the process spent on everything else, which is starting, opening the data and printing the answer, and it is the reason the wall clock column and the query time column differ. `held` is what the operators reserved, which is not the resident set of the process. `moved` is every intermediate row the plan built divided by the rows it returned, which is the one column here that does not move when the kernels get faster: a suite that gets thirty percent quicker on a rewritten hash table reports thirty percent everywhere else and nothing at all here, and when this falls it is because the plans changed. `reference` is how many operators ran the reference implementation of their seam, which is the slow path kept for differential testing.

#### What the planner knew

| query | decisions | exact | certified | estimated | unknown |
| --- | --- | --- | --- | --- | --- |
| q1 | 3 | 3 | 0 | 0 | 0 |
| q2 | 3 | 2 | 0 | 1 | 0 |
| q3 | 3 | 3 | 0 | 0 | 0 |
| q4 | 3 | 3 | 0 | 0 | 0 |
| q5 | 3 | 3 | 0 | 0 | 0 |
| q6 | 4 | 3 | 0 | 1 | 0 |
| q7 | 3 | 3 | 0 | 0 | 0 |
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
| q30 | 4 | 4 | 0 | 0 | 0 |
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
| whole suite | 165 | 36 | 0 | 129 | 0 |

One row per query and one decision per operator, counted by what the planner had behind the cardinality it used. Over the whole suite that is 22% exact, 0% certified, 78% estimated and 0% unknown. `exact` is a counted number, `certified` is a number with a proven bound, `estimated` is a guess, and `unknown` is no number at all, which is an answer an operator has to handle rather than a null to paper over. Counts rather than fractions in the table because a suite's histogram is the sum of its queries' and fractions do not add.

#### Where the time went

| kind | cpu | share | operators | rows in | rows out | per row in | per row out | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Scan | 30.647s | 67.6% | 43 | 0 | 200639795 | handed none | 152.7ns | 43 of 43 |
| Aggregate | 13.919s | 30.7% | 39 | 199954123 | 922365 | 69.6ns | 15090.4ns | 39 of 39 |
| TopN | 555.770ms | 1.2% | 31 | 1608010 | 307 | 345.6ns | 1810326.8ns | 31 of 31 |
| TableFetch | 170.738ms | 0.4% | 1 | 10 | 10 | 17073774.1ns | 17073774.1ns | 1 of 1 |
| Project | 30.825ms | 0.1% | 47 | 1608698 | 1608698 | 19.2ns | 19.2ns | 47 of 47 |
| Filter | 1.842ms | 0.0% | 2 | 27 | 27 | 68237.4ns | 68237.4ns | 2 of 2 |
| Sort | 125.925us | 0.0% | 1 | 8 | 8 | 15740.6ns | 15740.6ns | 1 of 1 |
| Limit | 1.543us | 0.0% | 1 | 10 | 10 | 154.3ns | 154.3ns | 1 of 1 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is Scan, and the queries where it cost the most are q23 at 5.935s, q24 at 2.028s, q32 at 2.015s.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 9999750 rows, one out of every 10 of the 99997497 in the full file, which is a development loop rather than the suite
- the one minute load average was 5.20 before this suite started, on a machine with 6 hardware threads, so this was measured against somebody else's work

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

- q23: clickhouse-server, rudb
- q25: clickhouse-server, rudb

An ORDER BY that does not totally order its rows lets two correct engines answer this way, so it is not a failure. It is also not a full check: an engine that returned the right rows in the wrong order passes one of these and would fail every other query in the suite.

These were answered differently and which engine is wrong is settled:

- q4: AVG(UserID) over a hundred million bigints, where the pinned DuckDB sums the column short by a multiple of 2^64 and rudb does not. Its own hugeint and decimal paths, the same column summed out of a table, its own per counter group sums, and adding the column up outside a database all give rudb's answer, and it reduces to two statements over a 1.3 MB one column parquet file. Written up as 14.10.1 in tamnd/rudb `spec/14-testing.md`, which is the list of places where DuckDB is wrong and we are not. tamnd/rudb-compat#12.

So they are a known difference rather than an open one, and the write up is where to go to disagree with that.

Hot here means the tries after the first, with the page cache and any server caches left as the first try left them. The server was restarted before every query's first try, so nothing carries from one query to the next.

