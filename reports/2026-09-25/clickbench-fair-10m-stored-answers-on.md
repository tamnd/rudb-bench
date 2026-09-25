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
| duckdb | 44 | 3.27 | 5.10 | 5.94 | 1110.075s | 15 |
| clickhouse-server | 44 | 3.96 | 5.00 | 5.94 | 2010.183s | 14 |
| rudb | 44 | 4.21 | 5.07 | 5.76 | 960.267s | 14 |

The reading includes the tail of whatever ran just before, which is usually the previous engine's turn, because the one minute average takes about a minute to decay.

rudb writes summaries of each column when it loads a table, and it can answer some queries from those without reading the rows. Its metrics said it did that for q1, q3, q4, q5, q6, q7, q30. Those times are lookups, not scans, and the other engines read the data for the same queries. The per query table marks them, and it also marks q1 to q7, whose shapes (counts, sums, averages, distinct counts and bounds over the whole table) are the ones a summary can answer. The headline below gives the ratios both with and without q1 to q7.

## Headline

| engine | hot total | cold total | hot geomean | total vs duckdb | geomean vs duckdb | total vs, without q1 to q7 | geomean vs, without q1 to q7 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 15.507s | 22.313s | 0.1506s | 1.000x | 1.000x | 1.000x | 1.000x |
| clickhouse-server | 10.852s | 20.744s | 0.1022s | 0.700x | 0.679x | 0.695x | 0.682x |
| rudb | 3.952s | 10.380s | 0.0346s | 0.255x | 0.229x | 0.262x | 0.303x |

Over the 43 queries every engine finished, 36 of them outside q1 to q7. Hot is the best of the tries after the first and cold is the first try, both by the engine's own clock. The geometric mean is over the hot figures. A ratio under 1 means faster than duckdb.

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run clickbench --rows 10000000 --engines duckdb,clickhouse-server,rudb --runs 3 --protocol upstream --sample-file /root/rudb-data/hits-10m.parquet --timeout 600 --report
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
| duckdb | v1.4.1 (Andium) b390a7c376 | ran | 51.636s | 192.540s | 1.71 GiB | its own database file | its own | 3.27 to 4.55 |
| clickhouse-server | 26.10.1.642 | ran | 40.682s | not read | 1.03 GiB | its own MergeTree parts, sorted by hits by (CounterID, EventDate, UserID, EventTime, WatchID), as system.parts counts the active ones | its own | 4.55 to 4.36 |
| rudb | rudb 0.4.33 | ran | 33.324s | 151.770s | 1.06 GiB | its own database file | its own | 4.36 to 5.87 |
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
| duckdb | 15.507s | 20.003s | +29% | 34.555s | 65.220s | 3.26 | 949.72 MiB | 142.20 MiB | 27.73M/s | 3.87 GiB/s | 1.00x |
| clickhouse-server | 10.852s | 18.907s | +74% | 28.434s | not read | not read | not read | not read | 39.62M/s | 5.53 GiB/s | 0.70x |
| rudb | 3.952s | 5.708s | +44% | 15.092s | 13.730s | 2.41 | 235.50 MiB | 13.59 MiB | 108.81M/s | 15.19 GiB/s | 0.25x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | clickhouse-server | rudb | rudb from stored summaries |
| --- | --- | --- | --- | --- | --- |
| q1 | count | 4.000ms | 6.000ms | 1.490ms | yes, per its metrics |
| q2 | filtered count | 21.000ms | 3.000ms | 6.296ms | no, but the shape allows it |
| q3 | three aggregates | 42.000ms | 22.000ms | 1.359ms | yes, per its metrics |
| q4 | average | 32.000ms | 19.000ms | 1.512ms | yes, per its metrics |
| q5 | count distinct, high card | 172.000ms | 91.000ms | 1.423ms | yes, per its metrics |
| q6 | count distinct, strings | 198.000ms | 256.000ms | 1.432ms | yes, per its metrics |
| q7 | min and max of a date | 10.000ms | 12.000ms | 1.516ms | yes, per its metrics |
| q8 | group by, low card | 48.000ms | 12.000ms | 21.165ms |  |
| q9 | group by and count distinct | 223.000ms | 157.000ms | 67.664ms |  |
| q10 | group by, several aggregates | 447.000ms | 197.000ms | 183.982ms |  |
| q11 | group by a string and count distinct | 145.000ms | 108.000ms | 55.813ms |  |
| q12 | group by two strings and count distinct | 161.000ms | 124.000ms | 51.625ms |  |
| q13 | group by a string and top k | 424.000ms | 381.000ms | 41.247ms |  |
| q14 | group by a string and count distinct | 320.000ms | 233.000ms | 59.088ms |  |
| q15 | group by two columns and top k | 245.000ms | 238.000ms | 82.959ms |  |
| q16 | group by, very high card | 234.000ms | 176.000ms | 93.648ms |  |
| q17 | group by two, very high card | 607.000ms | 704.000ms | 312.921ms |  |
| q18 | group by two, no ordering | 464.000ms | 362.000ms | 102.464ms |  |
| q19 | group by with an extract | 1.284s | 844.000ms | 354.486ms |  |
| q20 | point lookup | 16.000ms | 5.000ms | 6.129ms |  |
| q21 | substring scan | 399.000ms | 246.000ms | 103.778ms |  |
| q22 | substring scan and group by | 348.000ms | 56.000ms | 144.975ms |  |
| q23 | two substring scans and group by | 899.000ms | 505.000ms | 204.982ms |  |
| q24 | select star and top k | 1.018s | 125.000ms | 198.435ms |  |
| q25 | top k by a date | 62.000ms | 37.000ms | 9.495ms |  |
| q26 | top k by a string | 59.000ms | 84.000ms | 32.819ms |  |
| q27 | top k by two columns | 51.000ms | 40.000ms | 22.323ms |  |
| q28 | group by with a string length | 261.000ms | 93.000ms | 112.911ms |  |
| q29 | group by a regular expression | 2.999s | 2.520s | 727.963ms |  |
| q30 | ninety sums over one column | 32.000ms | 33.000ms | 6.771ms | yes, per its metrics |
| q31 | group by two and several aggregates | 206.000ms | 154.000ms | 108.407ms |  |
| q32 | group by a high card pair | 339.000ms | 179.000ms | 108.144ms |  |
| q33 | group by a high card pair, unfiltered | 829.000ms | 608.000ms | 273.635ms |  |
| q34 | group by a long string | 941.000ms | 779.000ms | 63.327ms |  |
| q35 | group by a constant and a long string | 1.055s | 692.000ms | 65.531ms |  |
| q36 | group by four expressions | 437.000ms | 105.000ms | 43.511ms |  |
| q37 | date range and group by a URL | 99.000ms | 136.000ms | 25.471ms |  |
| q38 | date range and group by a title | 30.000ms | 74.000ms | 24.396ms |  |
| q39 | date range, group by and offset | 43.000ms | 59.000ms | 21.287ms |  |
| q40 | date range, a case and a wide group by | 181.000ms | 257.000ms | 138.852ms |  |
| q41 | date range with an IN and a hash | 39.000ms | 63.000ms | 27.660ms |  |
| q42 | date range and a deep offset | 44.000ms | 27.000ms | 24.935ms |  |
| q43 | minute buckets over a date range | 39.000ms | 30.000ms | 13.914ms |  |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 4.000ms | 189.011ms | 41.585ms | 0.2% | 41.585ms | 41.671ms | 41.585ms | 41.671ms | 30.000ms | 24.88 MiB | 24.17 MiB | 240.46M/s |
| q2 | filtered count | 21.000ms | 271.630ms | 82.761ms | 0.8% | 82.761ms | 83.415ms | 82.761ms | 83.415ms | 100.000ms | 48.12 MiB | 47.09 MiB | 120.83M/s |
| q3 | three aggregates | 42.000ms | 230.339ms | 103.019ms | 3.2% | 103.019ms | 106.319ms | 103.019ms | 106.319ms | 240.000ms | 69.88 MiB | 68.10 MiB | 97.07M/s |
| q4 | average | 32.000ms | 335.513ms | 84.186ms | 21.3% | 84.186ms | 102.097ms | 84.186ms | 102.097ms | 190.000ms | 55.50 MiB | 54.60 MiB | 118.78M/s |
| q5 | count distinct, high card | 172.000ms | 398.814ms | 246.497ms | 1.1% | 246.497ms | 249.151ms | 246.497ms | 249.151ms | 850.000ms | 117.12 MiB | 54.82 MiB | 40.57M/s |
| q6 | count distinct, strings | 198.000ms | 526.122ms | 311.992ms | 11.5% | 311.992ms | 348.016ms | 311.992ms | 348.016ms | 1.050s | 265.89 MiB | 82.15 MiB | 32.05M/s |
| q7 | min and max of a date | 10.000ms | 245.584ms | 62.585ms | 0.9% | 62.585ms | 63.129ms | 62.585ms | 63.129ms | 50.000ms | 31.75 MiB | 30.59 MiB | 159.78M/s |
| q8 | group by, low card | 48.000ms | 407.823ms | 112.151ms | 78.6% | 112.151ms | 200.344ms | 112.151ms | 200.344ms | 410.000ms | 49.88 MiB | 48.58 MiB | 89.16M/s |
| q9 | group by and count distinct | 223.000ms | 510.159ms | 310.535ms | 10.0% | 310.535ms | 341.627ms | 310.535ms | 341.627ms | 1.190s | 146.95 MiB | 76.60 MiB | 32.20M/s |
| q10 | group by, several aggregates | 447.000ms | 902.932ms | 570.891ms | 0.2% | 570.891ms | 571.793ms | 570.891ms | 571.793ms | 1.890s | 195.00 MiB | 114.38 MiB | 17.52M/s |
| q11 | group by a string and count distinct | 145.000ms | 563.064ms | 244.867ms | 18.1% | 244.867ms | 289.253ms | 244.867ms | 289.253ms | 290.000ms | 93.75 MiB | 78.28 MiB | 40.84M/s |
| q12 | group by two strings and count distinct | 161.000ms | 494.197ms | 249.837ms | 0.9% | 249.837ms | 252.071ms | 249.837ms | 252.071ms | 380.000ms | 98.75 MiB | 81.62 MiB | 40.03M/s |
| q13 | group by a string and top k | 424.000ms | 931.063ms | 550.834ms | 17.5% | 550.834ms | 647.158ms | 550.834ms | 647.158ms | 1.020s | 265.48 MiB | 80.67 MiB | 18.15M/s |
| q14 | group by a string and count distinct | 320.000ms | 706.825ms | 446.117ms | 0.3% | 446.117ms | 447.587ms | 446.117ms | 447.587ms | 1.600s | 380.56 MiB | 109.53 MiB | 22.42M/s |
| q15 | group by two columns and top k | 245.000ms | 594.767ms | 345.742ms | 1.3% | 345.742ms | 350.268ms | 345.742ms | 350.268ms | 1.110s | 278.12 MiB | 96.61 MiB | 28.92M/s |
| q16 | group by, very high card | 234.000ms | 610.178ms | 307.498ms | 19.5% | 307.498ms | 367.543ms | 307.498ms | 367.543ms | 1.000s | 133.14 MiB | 57.53 MiB | 32.52M/s |
| q17 | group by two, very high card | 607.000ms | 1.279s | 731.531ms | 28.0% | 731.531ms | 936.613ms | 731.531ms | 936.613ms | 1.830s | 331.79 MiB | 111.71 MiB | 13.67M/s |
| q18 | group by two, no ordering | 464.000ms | 1.050s | 589.589ms | 11.1% | 589.589ms | 654.765ms | 589.589ms | 654.765ms | 1.610s | 320.72 MiB | 111.60 MiB | 16.96M/s |
| q19 | group by with an extract | 1.284s | 2.120s | 1.525s | 10.7% | 1.525s | 1.688s | 1.525s | 1.688s | 3.830s | 600.11 MiB | 176.05 MiB | 6.56M/s |
| q20 | point lookup | 16.000ms | 283.088ms | 63.368ms | 31.7% | 63.368ms | 83.432ms | 63.368ms | 83.432ms | 80.000ms | 40.12 MiB | 40.25 MiB | 157.80M/s |
| q21 | substring scan | 399.000ms | 1.318s | 521.800ms | 4.5% | 521.800ms | 545.317ms | 521.800ms | 545.317ms | 1.720s | 313.62 MiB | 300.58 MiB | 19.16M/s |
| q22 | substring scan and group by | 348.000ms | 1.102s | 468.907ms | 9.6% | 468.907ms | 513.746ms | 468.907ms | 513.746ms | 1.840s | 380.75 MiB | 351.11 MiB | 21.33M/s |
| q23 | two substring scans and group by | 899.000ms | 1.502s | 1.070s | 60.3% | 1.070s | 1.715s | 1.070s | 1.715s | 3.080s | 637.65 MiB | 556.23 MiB | 9.34M/s |
| q24 | select star and top k | 1.018s | 1.731s | 1.222s | 6.4% | 1.222s | 1.300s | 1.222s | 1.300s | 1.670s | 390.12 MiB | 329.00 MiB | 8.19M/s |
| q25 | top k by a date | 62.000ms | 390.805ms | 125.407ms | 8.5% | 125.407ms | 136.080ms | 125.407ms | 136.080ms | 410.000ms | 92.12 MiB | 80.51 MiB | 79.74M/s |
| q26 | top k by a string | 59.000ms | 351.879ms | 123.649ms | 19.3% | 123.649ms | 147.538ms | 123.649ms | 147.538ms | 410.000ms | 92.38 MiB | 80.87 MiB | 80.87M/s |
| q27 | top k by two columns | 51.000ms | 311.417ms | 123.073ms | 1.3% | 123.073ms | 124.624ms | 123.073ms | 124.624ms | 300.000ms | 91.12 MiB | 79.97 MiB | 81.25M/s |
| q28 | group by with a string length | 261.000ms | 1.113s | 378.822ms | 5.5% | 378.822ms | 399.559ms | 378.822ms | 399.559ms | 1.490s | 324.62 MiB | 307.16 MiB | 26.40M/s |
| q29 | group by a regular expression | 2.999s | 4.262s | 3.163s | 6.5% | 3.163s | 3.368s | 3.163s | 3.368s | 18.000s | 586.35 MiB | 324.87 MiB | 3.16M/s |
| q30 | ninety sums over one column | 32.000ms | 248.760ms | 82.264ms | 0.2% | 82.264ms | 82.426ms | 82.264ms | 82.426ms | 140.000ms | 51.88 MiB | 48.75 MiB | 121.56M/s |
| q31 | group by two and several aggregates | 206.000ms | 576.663ms | 289.979ms | 0.5% | 289.979ms | 291.298ms | 289.979ms | 291.298ms | 940.000ms | 261.41 MiB | 164.53 MiB | 34.48M/s |
| q32 | group by a high card pair | 339.000ms | 829.744ms | 456.241ms | 3.3% | 456.241ms | 471.320ms | 456.241ms | 471.320ms | 1.220s | 351.50 MiB | 236.17 MiB | 21.92M/s |
| q33 | group by a high card pair, unfiltered | 829.000ms | 1.507s | 1.043s | 40.5% | 1.043s | 1.465s | 1.043s | 1.465s | 3.900s | 841.00 MiB | 180.67 MiB | 9.59M/s |
| q34 | group by a long string | 941.000ms | 1.941s | 1.162s | 3.8% | 1.162s | 1.206s | 1.162s | 1.206s | 3.870s | 925.52 MiB | 297.82 MiB | 8.61M/s |
| q35 | group by a constant and a long string | 1.055s | 1.743s | 1.332s | 0.6% | 1.332s | 1.340s | 1.332s | 1.340s | 4.480s | 949.72 MiB | 297.34 MiB | 7.51M/s |
| q36 | group by four expressions | 437.000ms | 555.018ms | 560.730ms | 5.9% | 560.730ms | 593.701ms | 560.730ms | 593.701ms | 1.010s | 139.43 MiB | 53.06 MiB | 17.83M/s |
| q37 | date range and group by a URL | 99.000ms | 340.221ms | 163.644ms | 12.8% | 163.644ms | 184.546ms | 163.644ms | 184.546ms | 410.000ms | 129.88 MiB | 50.80 MiB | 61.11M/s |
| q38 | date range and group by a title | 30.000ms | 251.137ms | 91.094ms | 12.1% | 91.094ms | 102.111ms | 91.094ms | 102.111ms | 130.000ms | 51.00 MiB | 33.93 MiB | 109.77M/s |
| q39 | date range, group by and offset | 43.000ms | 289.519ms | 103.543ms | 15.2% | 103.543ms | 119.298ms | 103.543ms | 119.298ms | 230.000ms | 68.62 MiB | 50.91 MiB | 96.58M/s |
| q40 | date range, a case and a wide group by | 181.000ms | 516.064ms | 247.530ms | 11.5% | 247.530ms | 275.939ms | 247.530ms | 275.939ms | 740.000ms | 214.12 MiB | 69.66 MiB | 40.40M/s |
| q41 | date range with an IN and a hash | 39.000ms | 383.119ms | 83.858ms | 26.7% | 83.858ms | 106.266ms | 83.858ms | 106.266ms | 120.000ms | 50.88 MiB | 41.97 MiB | 119.25M/s |
| q42 | date range and a deep offset | 44.000ms | 303.276ms | 105.281ms | 0.4% | 105.281ms | 105.668ms | 105.281ms | 105.668ms | 110.000ms | 45.50 MiB | 37.33 MiB | 94.98M/s |
| q43 | minute buckets over a date range | 39.000ms | 338.842ms | 105.369ms | 20.6% | 105.369ms | 127.029ms | 105.369ms | 127.029ms | 250.000ms | 40.38 MiB | 34.86 MiB | 94.90M/s |

duckdb v1.4.1 (Andium) b390a7c376 over 43 of 43 queries. Total 15.507s by its own clock and 20.003s by ours, 34.555s cold, 65.220s of CPU, peak 949.72 MiB, 27.73M/s and 3.87 GiB/s.

Running it cost 29% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 76.05x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-server in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 6.000ms | 187.873ms | 186.862ms | 7.6% | 186.862ms | 201.042ms | 186.862ms | 201.042ms | not read | not read | not read | 53.51M/s |
| q2 | filtered count | 3.000ms | 141.719ms | 142.197ms | 0.5% | 142.197ms | 142.916ms | 142.197ms | 142.916ms | not read | not read | not read | 70.32M/s |
| q3 | three aggregates | 22.000ms | 203.418ms | 204.667ms | 0.0% | 204.667ms | 204.723ms | 204.667ms | 204.723ms | not read | not read | not read | 48.86M/s |
| q4 | average | 19.000ms | 246.627ms | 182.868ms | 10.9% | 182.868ms | 202.752ms | 182.868ms | 202.752ms | not read | not read | not read | 54.68M/s |
| q5 | count distinct, high card | 91.000ms | 269.211ms | 246.881ms | 0.6% | 246.881ms | 248.453ms | 246.881ms | 248.453ms | not read | not read | not read | 40.50M/s |
| q6 | count distinct, strings | 256.000ms | 586.593ms | 403.940ms | 11.9% | 403.940ms | 452.162ms | 403.940ms | 452.162ms | not read | not read | not read | 24.76M/s |
| q7 | min and max of a date | 12.000ms | 162.521ms | 163.660ms | 12.6% | 163.660ms | 184.324ms | 163.660ms | 184.324ms | not read | not read | not read | 61.10M/s |
| q8 | group by, low card | 12.000ms | 184.312ms | 162.323ms | 13.3% | 162.323ms | 183.887ms | 162.323ms | 183.887ms | not read | not read | not read | 61.60M/s |
| q9 | group by and count distinct | 157.000ms | 421.314ms | 325.637ms | 7.3% | 325.637ms | 349.250ms | 325.637ms | 349.250ms | not read | not read | not read | 30.71M/s |
| q10 | group by, several aggregates | 197.000ms | 491.787ms | 384.612ms | 0.7% | 384.612ms | 387.279ms | 384.612ms | 387.279ms | not read | not read | not read | 26.00M/s |
| q11 | group by a string and count distinct | 108.000ms | 431.305ms | 486.247ms | 0.9% | 486.247ms | 490.557ms | 486.247ms | 490.557ms | not read | not read | not read | 20.57M/s |
| q12 | group by two strings and count distinct | 124.000ms | 307.493ms | 331.489ms | 3.9% | 331.489ms | 344.329ms | 331.489ms | 344.329ms | not read | not read | not read | 30.17M/s |
| q13 | group by a string and top k | 381.000ms | 1.060s | 690.399ms | 31.2% | 690.399ms | 905.919ms | 690.399ms | 905.919ms | not read | not read | not read | 14.48M/s |
| q14 | group by a string and count distinct | 233.000ms | 447.174ms | 365.777ms | 12.5% | 365.777ms | 411.585ms | 365.777ms | 411.585ms | not read | not read | not read | 27.34M/s |
| q15 | group by two columns and top k | 238.000ms | 581.896ms | 386.690ms | 5.9% | 386.690ms | 409.503ms | 386.690ms | 409.503ms | not read | not read | not read | 25.86M/s |
| q16 | group by, very high card | 176.000ms | 587.011ms | 465.804ms | 0.5% | 465.804ms | 468.075ms | 465.804ms | 468.075ms | not read | not read | not read | 21.47M/s |
| q17 | group by two, very high card | 704.000ms | 942.289ms | 932.593ms | 1.3% | 932.593ms | 944.783ms | 932.593ms | 944.783ms | not read | not read | not read | 10.72M/s |
| q18 | group by two, no ordering | 362.000ms | 757.313ms | 714.111ms | 26.2% | 714.111ms | 901.423ms | 714.111ms | 901.423ms | not read | not read | not read | 14.00M/s |
| q19 | group by with an extract | 844.000ms | 1.421s | 990.103ms | 13.1% | 990.103ms | 1.120s | 990.103ms | 1.120s | not read | not read | not read | 10.10M/s |
| q20 | point lookup | 5.000ms | 318.053ms | 222.228ms | 2.8% | 222.228ms | 228.339ms | 222.228ms | 228.339ms | not read | not read | not read | 45.00M/s |
| q21 | substring scan | 246.000ms | 1.544s | 511.180ms | 9.0% | 511.180ms | 557.416ms | 511.180ms | 557.416ms | not read | not read | not read | 19.56M/s |
| q22 | substring scan and group by | 56.000ms | 1.085s | 233.005ms | 31.4% | 233.005ms | 306.100ms | 233.005ms | 306.100ms | not read | not read | not read | 42.92M/s |
| q23 | two substring scans and group by | 505.000ms | 2.062s | 690.757ms | 5.6% | 690.757ms | 729.313ms | 690.757ms | 729.313ms | not read | not read | not read | 14.48M/s |
| q24 | select star and top k | 125.000ms | 1.231s | 265.428ms | 74.1% | 265.428ms | 462.088ms | 265.428ms | 462.088ms | not read | not read | not read | 37.67M/s |
| q25 | top k by a date | 37.000ms | 328.048ms | 205.239ms | 48.0% | 205.239ms | 303.660ms | 205.239ms | 303.660ms | not read | not read | not read | 48.72M/s |
| q26 | top k by a string | 84.000ms | 330.001ms | 228.259ms | 16.0% | 228.259ms | 264.841ms | 228.259ms | 264.841ms | not read | not read | not read | 43.81M/s |
| q27 | top k by two columns | 40.000ms | 245.166ms | 185.208ms | 56.2% | 185.208ms | 289.366ms | 185.208ms | 289.366ms | not read | not read | not read | 53.99M/s |
| q28 | group by with a string length | 93.000ms | 284.801ms | 287.709ms | 22.5% | 287.709ms | 352.309ms | 287.709ms | 352.309ms | not read | not read | not read | 34.76M/s |
| q29 | group by a regular expression | 2.520s | 3.275s | 2.969s | 3.5% | 2.969s | 3.072s | 2.969s | 3.072s | not read | not read | not read | 3.37M/s |
| q30 | ninety sums over one column | 33.000ms | 283.236ms | 162.050ms | 13.0% | 162.050ms | 183.062ms | 162.050ms | 183.062ms | not read | not read | not read | 61.71M/s |
| q31 | group by two and several aggregates | 154.000ms | 427.826ms | 308.768ms | 13.0% | 308.768ms | 348.898ms | 308.768ms | 348.898ms | not read | not read | not read | 32.39M/s |
| q32 | group by a high card pair | 179.000ms | 610.415ms | 349.469ms | 0.9% | 349.469ms | 352.512ms | 349.469ms | 352.512ms | not read | not read | not read | 28.61M/s |
| q33 | group by a high card pair, unfiltered | 608.000ms | 1.337s | 757.071ms | 10.6% | 757.071ms | 837.221ms | 757.071ms | 837.221ms | not read | not read | not read | 13.21M/s |
| q34 | group by a long string | 779.000ms | 1.510s | 957.824ms | 2.8% | 957.824ms | 984.969ms | 957.824ms | 984.969ms | not read | not read | not read | 10.44M/s |
| q35 | group by a constant and a long string | 692.000ms | 1.320s | 836.005ms | 16.3% | 836.005ms | 972.489ms | 836.005ms | 972.489ms | not read | not read | not read | 11.96M/s |
| q36 | group by four expressions | 105.000ms | 284.354ms | 266.568ms | 1.2% | 266.568ms | 269.852ms | 266.568ms | 269.852ms | not read | not read | not read | 37.51M/s |
| q37 | date range and group by a URL | 136.000ms | 326.262ms | 284.394ms | 1.8% | 284.394ms | 289.580ms | 284.394ms | 289.580ms | not read | not read | not read | 35.16M/s |
| q38 | date range and group by a title | 74.000ms | 268.195ms | 246.049ms | 1.6% | 246.049ms | 249.919ms | 246.049ms | 249.919ms | not read | not read | not read | 40.64M/s |
| q39 | date range, group by and offset | 59.000ms | 388.253ms | 204.005ms | 20.3% | 204.005ms | 245.427ms | 204.005ms | 245.427ms | not read | not read | not read | 49.02M/s |
| q40 | date range, a case and a wide group by | 257.000ms | 739.334ms | 394.668ms | 4.0% | 394.668ms | 410.375ms | 394.668ms | 410.375ms | not read | not read | not read | 25.34M/s |
| q41 | date range with an IN and a hash | 63.000ms | 316.693ms | 224.774ms | 25.2% | 224.774ms | 281.437ms | 224.774ms | 281.437ms | not read | not read | not read | 44.49M/s |
| q42 | date range and a deep offset | 27.000ms | 224.566ms | 164.686ms | 24.2% | 164.686ms | 204.614ms | 164.686ms | 204.614ms | not read | not read | not read | 60.72M/s |
| q43 | minute buckets over a date range | 30.000ms | 264.566ms | 186.130ms | 20.1% | 186.130ms | 223.460ms | 186.130ms | 223.460ms | not read | not read | not read | 53.72M/s |

clickhouse-server 26.10.1.642 over 43 of 43 queries. Total 10.852s by its own clock and 18.907s by ours, 28.434s cold, no reading of CPU, peak not read, 39.62M/s and 5.53 GiB/s.

Running it cost 74% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 20.88x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.490ms | 107.898ms | 21.248ms | 1.6% | 21.248ms | 21.598ms | 21.248ms | 21.598ms | 10.000ms | 11.38 MiB | 9.87 MiB | 470.63M/s |
| q2 | filtered count | 6.296ms | 149.858ms | 41.631ms | 4.0% | 41.631ms | 43.294ms | 41.631ms | 43.294ms | 30.000ms | 14.62 MiB | 12.49 MiB | 240.20M/s |
| q3 | three aggregates | 1.359ms | 88.179ms | 21.384ms | 1.9% | 21.384ms | 21.784ms | 21.384ms | 21.784ms | 0.000us | 11.62 MiB | 10.28 MiB | 467.62M/s |
| q4 | average | 1.512ms | 111.775ms | 21.656ms | 4.2% | 21.656ms | 22.572ms | 21.656ms | 22.572ms | 0.000us | 11.50 MiB | 10.00 MiB | 461.75M/s |
| q5 | count distinct, high card | 1.423ms | 89.531ms | 21.204ms | 3.2% | 21.204ms | 21.880ms | 21.204ms | 21.880ms | 0.000us | 11.25 MiB | 9.87 MiB | 471.59M/s |
| q6 | count distinct, strings | 1.432ms | 88.107ms | 21.477ms | 4.5% | 21.477ms | 22.453ms | 21.477ms | 22.453ms | 10.000ms | 11.38 MiB | 9.86 MiB | 465.61M/s |
| q7 | min and max of a date | 1.516ms | 130.732ms | 21.401ms | 1.5% | 21.401ms | 21.719ms | 21.401ms | 21.719ms | 0.000us | 11.50 MiB | 10.02 MiB | 467.25M/s |
| q8 | group by, low card | 21.165ms | 265.062ms | 76.713ms | 46.6% | 76.713ms | 112.432ms | 76.713ms | 112.432ms | 110.000ms | 15.50 MiB | 12.69 MiB | 130.35M/s |
| q9 | group by and count distinct | 67.664ms | 275.754ms | 106.734ms | 20.5% | 106.734ms | 128.656ms | 106.734ms | 128.656ms | 360.000ms | 53.75 MiB | 28.23 MiB | 93.69M/s |
| q10 | group by, several aggregates | 183.982ms | 555.348ms | 254.433ms | 3.2% | 254.433ms | 262.506ms | 254.433ms | 262.506ms | 390.000ms | 56.50 MiB | 32.98 MiB | 39.30M/s |
| q11 | group by a string and count distinct | 55.813ms | 324.107ms | 101.795ms | 6.7% | 101.795ms | 108.658ms | 101.795ms | 108.658ms | 90.000ms | 25.12 MiB | 24.92 MiB | 98.23M/s |
| q12 | group by two strings and count distinct | 51.625ms | 256.765ms | 92.112ms | 20.2% | 92.112ms | 110.694ms | 92.112ms | 110.694ms | 110.000ms | 24.88 MiB | 25.85 MiB | 108.56M/s |
| q13 | group by a string and top k | 41.247ms | 567.043ms | 66.419ms | 28.9% | 66.419ms | 85.606ms | 66.419ms | 85.606ms | 100.000ms | 27.38 MiB | 22.89 MiB | 150.56M/s |
| q14 | group by a string and count distinct | 59.088ms | 235.920ms | 82.674ms | 5.3% | 82.674ms | 87.044ms | 82.674ms | 87.044ms | 260.000ms | 48.00 MiB | 39.68 MiB | 120.95M/s |
| q15 | group by two columns and top k | 82.959ms | 229.060ms | 122.694ms | 5.8% | 122.694ms | 129.793ms | 122.694ms | 129.793ms | 370.000ms | 55.62 MiB | 30.16 MiB | 81.50M/s |
| q16 | group by, very high card | 93.648ms | 322.047ms | 154.760ms | 6.1% | 154.760ms | 164.262ms | 154.760ms | 164.262ms | 230.000ms | 51.25 MiB | 23.23 MiB | 64.61M/s |
| q17 | group by two, very high card | 312.921ms | 735.012ms | 389.666ms | 6.2% | 389.666ms | 413.656ms | 389.666ms | 413.656ms | 720.000ms | 102.12 MiB | 34.62 MiB | 25.66M/s |
| q18 | group by two, no ordering | 102.464ms | 367.893ms | 135.852ms | 9.6% | 135.852ms | 148.905ms | 135.852ms | 148.905ms | 320.000ms | 27.12 MiB | 34.02 MiB | 73.61M/s |
| q19 | group by with an extract | 354.486ms | 671.225ms | 432.002ms | 9.2% | 432.002ms | 471.562ms | 432.002ms | 471.562ms | 1.190s | 235.50 MiB | 56.15 MiB | 23.15M/s |
| q20 | point lookup | 6.129ms | 174.070ms | 42.774ms | 0.2% | 42.774ms | 42.852ms | 42.774ms | 42.852ms | 30.000ms | 15.00 MiB | 13.71 MiB | 233.78M/s |
| q21 | substring scan | 103.778ms | 663.746ms | 142.855ms | 3.6% | 142.855ms | 147.948ms | 142.855ms | 147.948ms | 400.000ms | 45.12 MiB | 83.80 MiB | 70.00M/s |
| q22 | substring scan and group by | 144.975ms | 535.399ms | 190.750ms | 10.1% | 190.750ms | 209.960ms | 190.750ms | 209.960ms | 510.000ms | 56.00 MiB | 97.22 MiB | 52.42M/s |
| q23 | two substring scans and group by | 204.982ms | 825.199ms | 245.455ms | 3.0% | 245.455ms | 252.746ms | 245.455ms | 252.746ms | 710.000ms | 84.00 MiB | 188.49 MiB | 40.74M/s |
| q24 | select star and top k | 198.435ms | 990.216ms | 270.396ms | 12.3% | 270.396ms | 303.550ms | 270.396ms | 303.550ms | 340.000ms | 69.12 MiB | 115.98 MiB | 36.98M/s |
| q25 | top k by a date | 9.495ms | 210.618ms | 42.356ms | 46.9% | 42.356ms | 62.201ms | 42.356ms | 62.201ms | 50.000ms | 20.12 MiB | 18.79 MiB | 236.09M/s |
| q26 | top k by a string | 32.819ms | 354.240ms | 63.177ms | 30.6% | 63.177ms | 82.524ms | 63.177ms | 82.524ms | 120.000ms | 24.12 MiB | 27.73 MiB | 158.28M/s |
| q27 | top k by two columns | 22.323ms | 210.968ms | 61.907ms | 34.6% | 61.907ms | 83.303ms | 61.907ms | 83.303ms | 80.000ms | 23.25 MiB | 22.47 MiB | 161.53M/s |
| q28 | group by with a string length | 112.911ms | 298.451ms | 147.642ms | 27.6% | 147.642ms | 188.337ms | 147.642ms | 188.337ms | 350.000ms | 45.00 MiB | 40.63 MiB | 67.73M/s |
| q29 | group by a regular expression | 727.963ms | 1.416s | 799.374ms | 0.2% | 799.374ms | 800.770ms | 799.374ms | 800.770ms | 3.530s | 222.38 MiB | 205.36 MiB | 12.51M/s |
| q30 | ninety sums over one column | 6.771ms | 110.936ms | 41.805ms | 0.6% | 41.805ms | 42.051ms | 41.805ms | 42.051ms | 10.000ms | 12.88 MiB | 11.30 MiB | 239.20M/s |
| q31 | group by two and several aggregates | 108.407ms | 363.869ms | 144.286ms | 3.2% | 144.286ms | 148.918ms | 144.286ms | 148.918ms | 340.000ms | 60.50 MiB | 39.39 MiB | 69.31M/s |
| q32 | group by a high card pair | 108.144ms | 358.088ms | 147.883ms | 0.0% | 147.883ms | 147.932ms | 147.883ms | 147.932ms | 420.000ms | 80.12 MiB | 112.13 MiB | 67.62M/s |
| q33 | group by a high card pair, unfiltered | 273.635ms | 500.520ms | 329.530ms | 11.9% | 329.530ms | 368.652ms | 329.530ms | 368.652ms | 1.030s | 215.38 MiB | 99.44 MiB | 30.35M/s |
| q34 | group by a long string | 63.327ms | 232.275ms | 103.956ms | 22.9% | 103.956ms | 127.722ms | 103.956ms | 127.722ms | 230.000ms | 74.00 MiB | 39.79 MiB | 96.19M/s |
| q35 | group by a constant and a long string | 65.531ms | 319.562ms | 104.544ms | 66.8% | 104.544ms | 174.414ms | 104.544ms | 174.414ms | 340.000ms | 74.25 MiB | 39.77 MiB | 95.65M/s |
| q36 | group by four expressions | 43.511ms | 273.580ms | 84.285ms | 1.5% | 84.285ms | 85.519ms | 84.285ms | 85.519ms | 200.000ms | 52.00 MiB | 20.03 MiB | 118.64M/s |
| q37 | date range and group by a URL | 25.471ms | 187.589ms | 61.565ms | 0.4% | 61.565ms | 61.785ms | 61.565ms | 61.785ms | 70.000ms | 36.88 MiB | 22.10 MiB | 162.43M/s |
| q38 | date range and group by a title | 24.396ms | 212.735ms | 62.013ms | 1.4% | 62.013ms | 62.889ms | 62.013ms | 62.889ms | 60.000ms | 30.25 MiB | 19.61 MiB | 161.25M/s |
| q39 | date range, group by and offset | 21.287ms | 170.635ms | 82.671ms | 6.1% | 82.671ms | 87.735ms | 82.671ms | 87.735ms | 70.000ms | 25.12 MiB | 22.12 MiB | 120.96M/s |
| q40 | date range, a case and a wide group by | 138.852ms | 358.989ms | 183.371ms | 0.9% | 183.371ms | 185.007ms | 183.371ms | 185.007ms | 360.000ms | 73.62 MiB | 34.13 MiB | 54.53M/s |
| q41 | date range with an IN and a hash | 27.660ms | 386.325ms | 64.890ms | 7.7% | 64.890ms | 69.892ms | 64.890ms | 69.892ms | 70.000ms | 36.00 MiB | 25.39 MiB | 154.10M/s |
| q42 | date range and a deep offset | 24.935ms | 218.819ms | 63.338ms | 4.3% | 63.338ms | 66.060ms | 63.338ms | 66.060ms | 60.000ms | 30.50 MiB | 20.14 MiB | 157.88M/s |
| q43 | minute buckets over a date range | 13.914ms | 148.171ms | 41.653ms | 6.5% | 41.653ms | 44.378ms | 41.653ms | 44.378ms | 50.000ms | 21.62 MiB | 14.80 MiB | 240.07M/s |

rudb rudb 0.4.33 over 43 of 43 queries. Total 3.952s by its own clock and 5.708s by ours, 15.092s cold, 13.730s of CPU, peak 235.50 MiB, 108.81M/s and 15.19 GiB/s.

Running it cost 44% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 37.70x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | moved | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | 23.957ms | 2.363ms | 762.982us | 925.720us | 17.6% | 762.982us | 4.602ms | 14.472ms | 192 B | 1.0x | 2 of 2 |
| q2 | 26.542ms | 42.691ms | 56.941ms | 57.740ms | 1.4% | 56.941ms | 3.336ms | 28.924ms | 896 B | 202135.0x | 3 of 3 |
| q3 | 18.368ms | 1.535ms | 462.303us | 635.440us | 27.2% | 462.303us | 3.460ms | 25.905ms | 472 B | 1.0x | 2 of 2 |
| q4 | 21.999ms | 2.843ms | 618.439us | 853.573us | 27.5% | 618.439us | 3.408ms | 25.739ms | 192 B | 1.0x | 2 of 2 |
| q5 | 16.935ms | 1.896ms | 597.159us | 847.351us | 29.5% | 597.159us | 2.756ms | 16.396ms | 192 B | 1.0x | 2 of 2 |
| q6 | 16.464ms | 2.085ms | 535.010us | 664.014us | 19.4% | 535.010us | 2.165ms | 27.171ms | 192 B | 1.0x | 2 of 2 |
| q7 | 28.347ms | 2.471ms | 288.355us | 1.226ms | 76.5% | 288.355us | 3.984ms | 44.790ms | 320 B | 1.0x | 2 of 2 |
| q8 | 47.919ms | 73.751ms | 127.044ms | 127.467ms | 0.3% | 127.044ms | 8.087ms | 64.446ms | 2.77 KiB | 25268.8x | 4 of 4 |
| q9 | 25.502ms | 154.193ms | 359.689ms | 360.407ms | 0.2% | 359.689ms | 3.711ms | 125.882ms | 44.38 MiB | 1000103.0x | 4 of 4 |
| q10 | 67.553ms | 271.715ms | 405.032ms | 405.552ms | 0.1% | 405.032ms | 2.906ms | 101.543ms | 44.81 MiB | 1000103.0x | 4 of 4 |
| q11 | 54.089ms | 117.021ms | 120.195ms | 120.606ms | 0.3% | 120.195ms | 2.716ms | 36.678ms | 2.96 MiB | 35001.1x | 4 of 4 |
| q12 | 37.828ms | 95.623ms | 129.994ms | 130.742ms | 0.6% | 129.994ms | 3.380ms | 25.879ms | 3.04 MiB | 35013.3x | 4 of 4 |
| q13 | 146.116ms | 260.488ms | 135.468ms | 136.341ms | 0.6% | 135.468ms | 10.212ms | 73.447ms | 12.02 MiB | 137414.9x | 4 of 4 |
| q14 | 27.126ms | 126.374ms | 252.144ms | 253.168ms | 0.4% | 252.144ms | 4.502ms | 172.329ms | 32.80 MiB | 137534.9x | 4 of 4 |
| q15 | 22.750ms | 107.047ms | 243.974ms | 244.446ms | 0.2% | 243.974ms | 2.903ms | 132.651ms | 30.74 MiB | 137534.9x | 4 of 4 |
| q16 | 41.617ms | 151.600ms | 203.507ms | 204.737ms | 0.6% | 203.507ms | 3.230ms | 122.034ms | 28.19 MiB | 1000103.0x | 4 of 4 |
| q17 | 83.766ms | 447.282ms | 702.700ms | 703.633ms | 0.1% | 702.700ms | 3.000ms | 153.368ms | 133.57 MiB | 1000103.0x | 4 of 4 |
| q18 | 39.549ms | 197.936ms | 443.382ms | 444.327ms | 0.2% | 443.382ms | 7.016ms | 48.657ms | 388.35 KiB | 999977.0x | 4 of 4 |
| q19 | 35.972ms | 497.172ms | 1.120s | 1.121s | 0.0% | 1.120s | 3.861ms | 275.632ms | 350.76 MiB | 1000103.0x | 4 of 4 |
| q20 | 27.397ms | 33.226ms | 48.305ms | 49.358ms | 2.1% | 48.305ms | 9.155ms | 51.488ms | 0 B | not read | 2 of 2 |
| q21 | 29.087ms | 546.639ms | 796.875ms | 797.531ms | 0.1% | 796.875ms | 4.153ms | 38.316ms | 896 B | 647.0x | 3 of 3 |
| q22 | 36.173ms | 393.186ms | 841.248ms | 842.029ms | 0.1% | 841.248ms | 5.101ms | 42.871ms | 503.83 KiB | 6.4x | 4 of 4 |
| q23 | 43.311ms | 679.050ms | 1.326s | 1.326s | 0.0% | 1.326s | 4.407ms | 49.336ms | 655.74 KiB | 111.1x | 4 of 4 |
| q24 | 46.681ms | 783.969ms | 427.838ms | 439.589ms | 2.7% | 427.838ms | 2.687ms | 137.724ms | 30.18 KiB | 69.4x | 4 of 4 |
| q25 | 21.191ms | 102.341ms | 43.950ms | 61.084ms | 28.1% | 43.950ms | 2.611ms | 26.305ms | 21.44 KiB | 14210.0x | 4 of 4 |
| q26 | 22.674ms | 244.597ms | 281.922ms | 304.290ms | 7.4% | 281.922ms | 4.458ms | 41.252ms | 13.22 KiB | 274813.8x | 3 of 3 |
| q27 | 21.803ms | 99.788ms | 75.924ms | 94.651ms | 19.8% | 75.924ms | 4.801ms | 40.548ms | 25.19 KiB | 14879.0x | 4 of 4 |
| q28 | 40.566ms | 168.804ms | 417.228ms | 417.775ms | 0.1% | 417.228ms | 4.176ms | 38.049ms | 18.93 KiB | 833261.8x | 5 of 5 |
| q29 | 31.727ms | 1.254s | 4.647s | 4.649s | 0.0% | 4.647s | 5.240ms | 135.881ms | 78.06 MiB | 578849.7x | 5 of 5 |
| q30 | 24.485ms | 6.318ms | 2.108ms | 2.252ms | 6.4% | 2.108ms | 3.210ms | 24.538ms | 12.71 KiB | 2.0x | 3 of 3 |
| q31 | 34.061ms | 224.527ms | 380.511ms | 380.979ms | 0.1% | 380.511ms | 3.742ms | 105.279ms | 34.83 MiB | 137534.9x | 4 of 4 |
| q32 | 27.428ms | 234.179ms | 541.482ms | 542.185ms | 0.1% | 541.482ms | 3.710ms | 114.105ms | 38.78 MiB | 137534.9x | 4 of 4 |
| q33 | 36.126ms | 343.130ms | 805.786ms | 807.020ms | 0.2% | 805.786ms | 3.220ms | 439.760ms | 204.32 MiB | 1000103.0x | 4 of 4 |
| q34 | 33.240ms | 111.119ms | 239.878ms | 240.543ms | 0.3% | 239.878ms | 3.872ms | 65.585ms | 58.45 MiB | 999983.0x | 4 of 4 |
| q35 | 42.376ms | 157.643ms | 303.134ms | 304.548ms | 0.5% | 303.134ms | 3.602ms | 91.851ms | 58.55 MiB | 999983.0x | 4 of 4 |
| q36 | 46.613ms | 82.300ms | 189.713ms | 190.149ms | 0.2% | 189.713ms | 3.927ms | 85.924ms | 33.30 MiB | 1000167.0x | 5 of 5 |
| q37 | 39.291ms | 55.880ms | 66.654ms | 67.382ms | 1.1% | 66.654ms | 4.441ms | 48.177ms | 22.51 MiB | 55817.7x | 4 of 4 |
| q38 | 42.970ms | 70.537ms | 81.956ms | 82.874ms | 1.1% | 81.956ms | 6.044ms | 51.082ms | 14.69 MiB | 54895.3x | 4 of 4 |
| q39 | 25.859ms | 59.103ms | 57.743ms | 58.480ms | 1.3% | 57.743ms | 4.814ms | 36.706ms | 20.20 MiB | 4734.2x | 4 of 4 |
| q40 | 31.842ms | 205.874ms | 386.422ms | 386.870ms | 0.1% | 386.422ms | 4.428ms | 78.702ms | 24.00 MiB | 72956.8x | 4 of 4 |
| q41 | 145.950ms | 80.289ms | 99.013ms | 99.476ms | 0.5% | 99.013ms | 5.905ms | 54.619ms | 3.53 MiB | 7512.5x | 4 of 4 |
| q42 | 33.949ms | 58.124ms | 86.210ms | 86.960ms | 0.9% | 86.210ms | 3.796ms | 49.244ms | 5.50 MiB | not read | 4 of 4 |
| q43 | 34.045ms | 35.192ms | 71.633ms | 72.217ms | 0.8% | 71.633ms | 3.954ms | 33.829ms | 684.80 KiB | 56098.5x | 4 of 4 |

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
| Scan | 27.771s | 66.0% | 36 | 0 | 130640676 | handed none | 212.6ns | 36 of 36 |
| Aggregate | 13.276s | 31.5% | 37 | 129120830 | 87320 | 102.8ns | 152042.1ns | 37 of 37 |
| TopN | 811.789ms | 1.9% | 31 | 1607140 | 307 | 505.1ns | 2644262.8ns | 31 of 31 |
| TableFetch | 198.448ms | 0.5% | 1 | 10 | 10 | 19844792.3ns | 19844792.3ns | 1 of 1 |
| Project | 26.997ms | 0.1% | 47 | 1607828 | 1607828 | 16.8ns | 16.8ns | 47 of 47 |
| Filter | 2.008ms | 0.0% | 2 | 27 | 27 | 74373.7ns | 74373.7ns | 2 of 2 |
| Sort | 1.551ms | 0.0% | 1 | 8 | 8 | 193922.8ns | 193922.8ns | 1 of 1 |
| Values | 10.780us | 0.0% | 1 | 0 | 1 | handed none | 10780.0ns | 1 of 1 |
| Limit | 1.432us | 0.0% | 1 | 10 | 10 | 143.2ns | 143.2ns | 1 of 1 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is Scan, and the queries where it cost the most are q23 at 3.466s, q21 at 3.224s, q24 at 2.390s.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- this ran over 9999750 rows, one out of every 10 of the 99997497 in the full file, which is a development loop rather than the suite
- the one minute load average was 3.27 before this suite started, on a machine with 6 hardware threads, so this was measured against somebody else's work

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
- q23: clickhouse-server, rudb
- q25: clickhouse-server, rudb
- q31: clickhouse-server, rudb

An ORDER BY that does not totally order its rows lets two correct engines answer this way, so it is not a failure. It is also not a full check: an engine that returned the right rows in the wrong order passes one of these and would fail every other query in the suite.

These were answered differently and which engine is wrong is settled:

- q4: AVG(UserID) over a hundred million bigints, where the pinned DuckDB sums the column short by a multiple of 2^64 and rudb does not. Its own hugeint and decimal paths, the same column summed out of a table, its own per counter group sums, and adding the column up outside a database all give rudb's answer, and it reduces to two statements over a 1.3 MB one column parquet file. Written up as 14.10.1 in tamnd/rudb `spec/14-testing.md`, which is the list of places where DuckDB is wrong and we are not. tamnd/rudb-compat#12.

So they are a known difference rather than an open one, and the write up is where to go to disagree with that.

Hot here means the tries after the first, with the page cache and any server caches left as the first try left them. The server was restarted before every query's first try, so nothing carries from one query to the next.

