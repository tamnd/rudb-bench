# clickbench on gamingpc-wsl

This is one run of the clickbench suite on gamingpc-wsl, over 1 engine and 43 queries, with 4 tries of each query after a page cache drop, the way the upstream ClickBench driver runs it. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

## What ran

| what | it was |
| --- | --- |
| suite | clickbench |
| queries | 43 |
| tables | 13.76 GiB of Parquet in 1 table |
| rows | 99997497 in the table every query reads |
| corpus | no manifest beside the data, so this run cannot say where it came from |
| summary | the best of the tries after the first, which is the upstream ClickBench convention |
| timeout | 600s per query, and no query reached it |
| harness | rudb-bench 0.0.1 |

## How it was measured

Every engine loaded the data first, one at a time. Then each query was run on every engine in turn, and the order rotated by one engine per query, so no engine always went first or last. Before each engine's turn the harness waited for the one minute load average to be below the 16 hardware threads, dropped the page cache (and for the ClickHouse server stopped it first and started it again after), then ran the query 4 times in a row. The first try is the cold figure and the best of the other 3 is the hot one, which is what the upstream ClickBench driver does. Every figure below is the engine's own timing where it reports one.

Every engine got the same memory budget, 20.00 GiB (21474836480 bytes, from RUDB_BENCH_MEMORY). DuckDB and rudb got it as `SET memory_limit`, the ClickHouse server as `max_server_memory_usage` and `clickhouse local` as `max_memory_usage`.

The data file, which is also written beside it as a manifest:

| path | rows | bytes | sha256 |
| --- | --- | --- | --- |
| `/home/gopher/c0-gpc/data/hits.parquet` | 99997497 | 14779976446 | `a390f6cb782f6aaef278c72fc1dd86c4f30bc843ebab3c159e9bd4d45ddb079f` |

The one minute load average at the start of each engine's turns, counting its load and every query:

| engine | readings | lowest | median | highest | held by the gate | went first |
| --- | --- | --- | --- | --- | --- | --- |
| clickhouse-server | 44 | 0.34 | 1.49 | 1.85 | 660.004s | 43 |

The reading includes the tail of whatever ran just before, which is usually the previous engine's turn, because the one minute average takes about a minute to decay.

rudb writes summaries of each column when it loads a table, and it can answer some queries from those without reading the rows. This run turned that off with `SET stored_answers = false` before every rudb query (`RUDB_BENCH_STORED_ANSWERS=off`), so rudb read the rows the way the other engines did. Its metrics said it still answered from stored summaries for no query. The per query table marks any such query, and it also marks q1 to q7, whose shapes (counts, sums, averages, distinct counts and bounds over the whole table) are the ones a summary can answer. The headline below gives the ratios both with and without q1 to q7.

## Headline

| engine | hot total | cold total | hot geomean | total vs clickhouse-server | geomean vs clickhouse-server | total vs, without q1 to q7 | geomean vs, without q1 to q7 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| clickhouse-server | 10.765s | 14.914s | 0.0697s | 1.000x | 1.000x | 1.000x | 1.000x |

Over the 43 queries every engine finished, 36 of them outside q1 to q7. Hot is the best of the tries after the first and cold is the first try, both by the engine's own clock. The geometric mean is over the hot figures. A ratio under 1 means faster than clickhouse-server.

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
RUDB_BENCH_STORED_ANSWERS=off rudb-bench run clickbench --engines clickhouse-server --runs 4 --protocol upstream --timeout 600 --report
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
| page cache | droppable, cold runs are cold | read |
| timer | /usr/bin/time GNU | read |

A fact that says it was not read is a fact about this machine and not a gap in the report. A field that had silently defaulted would be a lie that survived into it.

## The engines

| engine | version | state | load | load cpu | on disk | that size is | format | machine load |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| clickhouse-server | 26.9.1.1138 | ran | 41.334s | not read | 9.37 GiB | its own MergeTree parts, sorted by hits by (CounterID, EventDate, UserID, EventTime, WatchID), as system.parts counts the active ones | its own | 0.34 to 29.42 |
| duckdb | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| duckdb-pinned | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| clickhouse-local | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| datafusion | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| polars | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| rudb | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

The machine load column is the one minute load average before and after that engine's suite, on a machine with 16 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is a run that had to go to the device. The page cache was dropped before each query's first run, the way official ClickBench does it.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs clickhouse-server |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| clickhouse-server | 10.765s | 12.674s | +18% | 16.889s | not read | not read | not read | not read | 399.43M/s | 54.98 GiB/s | 1.00x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | clickhouse-server | rudb from stored summaries |
| --- | --- | --- | --- |
| q1 | count | 1.000ms | no, but the shape allows it |
| q2 | filtered count | 1.000ms | no, but the shape allows it |
| q3 | three aggregates | 18.000ms | no, but the shape allows it |
| q4 | average | 26.000ms | no, but the shape allows it |
| q5 | count distinct, high card | 164.000ms | no, but the shape allows it |
| q6 | count distinct, strings | 230.000ms | no, but the shape allows it |
| q7 | min and max of a date | 6.000ms | no, but the shape allows it |
| q8 | group by, low card | 8.000ms |  |
| q9 | group by and count distinct | 302.000ms |  |
| q10 | group by, several aggregates | 348.000ms |  |
| q11 | group by a string and count distinct | 91.000ms |  |
| q12 | group by two strings and count distinct | 102.000ms |  |
| q13 | group by a string and top k | 230.000ms |  |
| q14 | group by a string and count distinct | 362.000ms |  |
| q15 | group by two columns and top k | 264.000ms |  |
| q16 | group by, very high card | 150.000ms |  |
| q17 | group by two, very high card | 644.000ms |  |
| q18 | group by two, no ordering | 246.000ms |  |
| q19 | group by with an extract | 1.158s |  |
| q20 | point lookup | 2.000ms |  |
| q21 | substring scan | 224.000ms |  |
| q22 | substring scan and group by | 61.000ms |  |
| q23 | two substring scans and group by | 335.000ms |  |
| q24 | select star and top k | 61.000ms |  |
| q25 | top k by a date | 36.000ms |  |
| q26 | top k by a string | 108.000ms |  |
| q27 | top k by two columns | 36.000ms |  |
| q28 | group by with a string length | 92.000ms |  |
| q29 | group by a regular expression | 985.000ms |  |
| q30 | ninety sums over one column | 23.000ms |  |
| q31 | group by two and several aggregates | 146.000ms |  |
| q32 | group by a high card pair | 204.000ms |  |
| q33 | group by a high card pair, unfiltered | 1.244s |  |
| q34 | group by a long string | 1.337s |  |
| q35 | group by a constant and a long string | 1.288s |  |
| q36 | group by four expressions | 127.000ms |  |
| q37 | date range and group by a URL | 21.000ms |  |
| q38 | date range and group by a title | 11.000ms |  |
| q39 | date range, group by and offset | 11.000ms |  |
| q40 | date range, a case and a wide group by | 43.000ms |  |
| q41 | date range with an IN and a hash | 7.000ms |  |
| q42 | date range and a deep offset | 6.000ms |  |
| q43 | minute buckets over a date range | 6.000ms |  |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## clickhouse-server in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 1.000ms | 40.366ms | 40.325ms | 0.1% | 40.325ms | 40.350ms | 40.325ms | 40.350ms | not read | not read | not read | 2.48G/s |
| q2 | filtered count | 1.000ms | 40.344ms | 40.340ms | 0.1% | 40.340ms | 40.374ms | 40.340ms | 40.374ms | not read | not read | not read | 2.48G/s |
| q3 | three aggregates | 18.000ms | 120.724ms | 61.900ms | 29.5% | 61.900ms | 80.539ms | 61.900ms | 80.539ms | not read | not read | not read | 1.62G/s |
| q4 | average | 26.000ms | 80.760ms | 64.325ms | 24.5% | 64.325ms | 84.041ms | 64.325ms | 84.041ms | not read | not read | not read | 1.55G/s |
| q5 | count distinct, high card | 164.000ms | 244.472ms | 203.081ms | 4.6% | 203.081ms | 212.450ms | 203.081ms | 212.450ms | not read | not read | not read | 492.40M/s |
| q6 | count distinct, strings | 230.000ms | 329.885ms | 269.278ms | 5.5% | 269.278ms | 285.016ms | 269.278ms | 285.016ms | not read | not read | not read | 371.35M/s |
| q7 | min and max of a date | 6.000ms | 61.098ms | 60.456ms | 33.0% | 60.456ms | 80.527ms | 60.456ms | 80.527ms | not read | not read | not read | 1.65G/s |
| q8 | group by, low card | 8.000ms | 80.501ms | 60.448ms | 0.0% | 60.448ms | 60.467ms | 60.448ms | 60.467ms | not read | not read | not read | 1.65G/s |
| q9 | group by and count distinct | 302.000ms | 405.543ms | 349.587ms | 5.6% | 349.587ms | 370.079ms | 349.587ms | 370.079ms | not read | not read | not read | 286.04M/s |
| q10 | group by, several aggregates | 348.000ms | 444.213ms | 382.732ms | 5.7% | 382.732ms | 404.910ms | 382.732ms | 404.910ms | not read | not read | not read | 261.27M/s |
| q11 | group by a string and count distinct | 91.000ms | 183.107ms | 143.305ms | 12.3% | 143.305ms | 161.072ms | 143.305ms | 161.072ms | not read | not read | not read | 697.80M/s |
| q12 | group by two strings and count distinct | 102.000ms | 182.043ms | 141.099ms | 7.1% | 141.099ms | 151.274ms | 141.099ms | 151.274ms | not read | not read | not read | 708.70M/s |
| q13 | group by a string and top k | 230.000ms | 328.912ms | 283.168ms | 1.7% | 283.168ms | 288.200ms | 283.168ms | 288.200ms | not read | not read | not read | 353.14M/s |
| q14 | group by a string and count distinct | 362.000ms | 506.916ms | 402.406ms | 5.9% | 402.406ms | 427.616ms | 402.406ms | 427.616ms | not read | not read | not read | 248.50M/s |
| q15 | group by two columns and top k | 264.000ms | 393.264ms | 308.877ms | 1.0% | 308.877ms | 311.828ms | 308.877ms | 311.828ms | not read | not read | not read | 323.75M/s |
| q16 | group by, very high card | 150.000ms | 224.807ms | 183.846ms | 11.0% | 183.846ms | 204.156ms | 183.846ms | 204.156ms | not read | not read | not read | 543.92M/s |
| q17 | group by two, very high card | 644.000ms | 754.770ms | 690.207ms | 0.5% | 690.207ms | 693.745ms | 690.207ms | 693.745ms | not read | not read | not read | 144.88M/s |
| q18 | group by two, no ordering | 246.000ms | 365.469ms | 284.673ms | 0.8% | 284.673ms | 286.977ms | 284.673ms | 286.977ms | not read | not read | not read | 351.27M/s |
| q19 | group by with an extract | 1.158s | 1.292s | 1.202s | 1.9% | 1.202s | 1.226s | 1.202s | 1.226s | not read | not read | not read | 83.16M/s |
| q20 | point lookup | 2.000ms | 100.737ms | 40.340ms | 49.8% | 40.340ms | 60.475ms | 40.340ms | 60.475ms | not read | not read | not read | 2.48G/s |
| q21 | substring scan | 224.000ms | 657.009ms | 265.734ms | 7.4% | 265.734ms | 286.853ms | 265.734ms | 286.853ms | not read | not read | not read | 376.31M/s |
| q22 | substring scan and group by | 61.000ms | 730.690ms | 106.670ms | 11.9% | 106.670ms | 120.979ms | 106.670ms | 120.979ms | not read | not read | not read | 937.44M/s |
| q23 | two substring scans and group by | 335.000ms | 895.303ms | 381.691ms | 5.5% | 381.691ms | 403.023ms | 381.691ms | 403.023ms | not read | not read | not read | 261.99M/s |
| q24 | select star and top k | 61.000ms | 242.827ms | 100.587ms | 3.2% | 100.587ms | 103.886ms | 100.587ms | 103.886ms | not read | not read | not read | 994.14M/s |
| q25 | top k by a date | 36.000ms | 140.848ms | 80.533ms | 4.6% | 80.533ms | 84.215ms | 80.533ms | 84.215ms | not read | not read | not read | 1.24G/s |
| q26 | top k by a string | 108.000ms | 203.737ms | 146.853ms | 9.2% | 146.853ms | 160.971ms | 146.853ms | 160.971ms | not read | not read | not read | 680.94M/s |
| q27 | top k by two columns | 36.000ms | 140.789ms | 80.924ms | 6.6% | 80.924ms | 86.476ms | 80.924ms | 86.476ms | not read | not read | not read | 1.24G/s |
| q28 | group by with a string length | 92.000ms | 164.627ms | 141.484ms | 15.8% | 141.484ms | 164.000ms | 141.484ms | 164.000ms | not read | not read | not read | 706.78M/s |
| q29 | group by a regular expression | 985.000ms | 1.277s | 1.028s | 4.1% | 1.028s | 1.071s | 1.028s | 1.071s | not read | not read | not read | 97.24M/s |
| q30 | ninety sums over one column | 23.000ms | 100.654ms | 60.493ms | 713.2% | 60.493ms | 584.252ms | 60.493ms | 584.252ms | not read | not read | not read | 1.65G/s |
| q31 | group by two and several aggregates | 146.000ms | 286.736ms | 184.509ms | 9.6% | 184.509ms | 204.076ms | 184.509ms | 204.076ms | not read | not read | not read | 541.97M/s |
| q32 | group by a high card pair | 204.000ms | 323.158ms | 246.404ms | 3.6% | 246.404ms | 255.407ms | 246.404ms | 255.407ms | not read | not read | not read | 405.83M/s |
| q33 | group by a high card pair, unfiltered | 1.244s | 1.364s | 1.294s | 1.3% | 1.294s | 1.310s | 1.294s | 1.310s | not read | not read | not read | 77.29M/s |
| q34 | group by a long string | 1.337s | 1.565s | 1.375s | 6.4% | 1.375s | 1.463s | 1.375s | 1.463s | not read | not read | not read | 72.74M/s |
| q35 | group by a constant and a long string | 1.288s | 1.600s | 1.341s | 3.6% | 1.341s | 1.391s | 1.341s | 1.391s | not read | not read | not read | 74.58M/s |
| q36 | group by four expressions | 127.000ms | 224.850ms | 164.356ms | 14.0% | 164.356ms | 187.654ms | 164.356ms | 187.654ms | not read | not read | not read | 608.42M/s |
| q37 | date range and group by a URL | 21.000ms | 120.773ms | 60.502ms | 33.2% | 60.502ms | 80.581ms | 60.502ms | 80.581ms | not read | not read | not read | 1.65G/s |
| q38 | date range and group by a title | 11.000ms | 121.262ms | 60.525ms | 0.1% | 60.525ms | 60.590ms | 60.525ms | 60.590ms | not read | not read | not read | 1.65G/s |
| q39 | date range, group by and offset | 11.000ms | 102.550ms | 60.486ms | 0.0% | 60.486ms | 60.516ms | 60.486ms | 60.516ms | not read | not read | not read | 1.65G/s |
| q40 | date range, a case and a wide group by | 43.000ms | 141.051ms | 100.643ms | 0.1% | 100.643ms | 100.746ms | 100.643ms | 100.746ms | not read | not read | not read | 993.59M/s |
| q41 | date range with an IN and a hash | 7.000ms | 120.817ms | 60.500ms | 0.0% | 60.500ms | 60.520ms | 60.500ms | 60.520ms | not read | not read | not read | 1.65G/s |
| q42 | date range and a deep offset | 6.000ms | 104.242ms | 60.456ms | 0.1% | 60.456ms | 60.538ms | 60.456ms | 60.538ms | not read | not read | not read | 1.65G/s |
| q43 | minute buckets over a date range | 6.000ms | 81.024ms | 60.516ms | 0.1% | 60.516ms | 60.579ms | 60.516ms | 60.579ms | not read | not read | not read | 1.65G/s |

clickhouse-server 26.9.1.1138 over 43 of 43 queries. Total 10.765s by its own clock and 12.674s by ours, 16.889s cold, no reading of CPU, peak not read, 399.43M/s and 54.98 GiB/s.

Running it cost 18% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 34.51x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- no query has a peak resident set, and rule six wants one. not measured, the timer wraps clickhouse client and the work happens in a server process it did not start

Hot here means the tries after the first, with the page cache and any server caches left as the first try left them. The server was restarted before every query's first try, so nothing carries from one query to the next.
