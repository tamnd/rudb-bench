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
| rudb | 44 | 1.21 | 1.76 | 1.98 | 270.002s | 43 |

The reading includes the tail of whatever ran just before, which is usually the previous engine's turn, because the one minute average takes about a minute to decay.

rudb writes summaries of each column when it loads a table, and it can answer some queries from those without reading the rows. This run turned that off with `SET stored_answers = false` before every rudb query (`RUDB_BENCH_STORED_ANSWERS=off`), so rudb read the rows the way the other engines did. Its metrics said it still answered from stored summaries for no query. The per query table marks any such query, and it also marks q1 to q7, whose shapes (counts, sums, averages, distinct counts and bounds over the whole table) are the ones a summary can answer. The headline below gives the ratios both with and without q1 to q7.

## Headline

| engine | hot total | cold total | hot geomean | total vs rudb | geomean vs rudb | total vs, without q1 to q7 | geomean vs, without q1 to q7 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| rudb | 4.392s | 22.527s | 0.0324s | 1.000x | 1.000x | 1.000x | 1.000x |

Over the 43 queries every engine finished, 36 of them outside q1 to q7. Hot is the best of the tries after the first and cold is the first try, both by the engine's own clock. The geometric mean is over the hot figures. A ratio under 1 means faster than rudb.

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
RUDB_BENCH_STORED_ANSWERS=off rudb-bench run clickbench --engines rudb --runs 4 --protocol upstream --timeout 600 --report
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
| rudb | rudb 0.6.0 | ran | 45.548s | 595.620s | 10.66 GiB | its own database file | its own | 1.21 to 8.65 |
| duckdb | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| duckdb-pinned | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| clickhouse-local | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| datafusion | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| polars | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

The machine load column is the one minute load average before and after that engine's suite, on a machine with 16 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is a run that had to go to the device. The page cache was dropped before each query's first run, the way official ClickBench does it.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs rudb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| rudb | 4.392s | 5.593s | +27% | 24.351s | 47.980s | 8.58 | 2.92 GiB | 12.82 MiB | 978.94M/s | 134.75 GiB/s | 1.00x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | rudb | rudb from stored summaries |
| --- | --- | --- | --- |
| q1 | count | 2.811ms | no, but the shape allows it |
| q2 | filtered count | 7.262ms | no, but the shape allows it |
| q3 | three aggregates | 21.804ms | no, but the shape allows it |
| q4 | average | 17.904ms | no, but the shape allows it |
| q5 | count distinct, high card | 154.768ms | no, but the shape allows it |
| q6 | count distinct, strings | 357.068ms | no, but the shape allows it |
| q7 | min and max of a date | 6.098ms | no, but the shape allows it |
| q8 | group by, low card | 8.324ms |  |
| q9 | group by and count distinct | 104.446ms |  |
| q10 | group by, several aggregates | 139.800ms |  |
| q11 | group by a string and count distinct | 30.656ms |  |
| q12 | group by two strings and count distinct | 37.215ms |  |
| q13 | group by a string and top k | 33.838ms |  |
| q14 | group by a string and count distinct | 84.930ms |  |
| q15 | group by two columns and top k | 76.205ms |  |
| q16 | group by, very high card | 664.773us |  |
| q17 | group by two, very high card | 19.160ms |  |
| q18 | group by two, no ordering | 34.962ms |  |
| q19 | group by with an extract | 296.833ms |  |
| q20 | point lookup | 2.606ms |  |
| q21 | substring scan | 215.760ms |  |
| q22 | substring scan and group by | 292.090ms |  |
| q23 | two substring scans and group by | 287.211ms |  |
| q24 | select star and top k | 100.506ms |  |
| q25 | top k by a date | 13.129ms |  |
| q26 | top k by a string | 61.212ms |  |
| q27 | top k by two columns | 43.317ms |  |
| q28 | group by with a string length | 213.215ms |  |
| q29 | group by a regular expression | 996.138ms |  |
| q30 | ninety sums over one column | 17.477ms |  |
| q31 | group by two and several aggregates | 92.967ms |  |
| q32 | group by a high card pair | 101.278ms |  |
| q33 | group by a high card pair, unfiltered | 280.100ms |  |
| q34 | group by a long string | 682.626us |  |
| q35 | group by a constant and a long string | 96.576ms |  |
| q36 | group by four expressions | 723.089us |  |
| q37 | date range and group by a URL | 24.208ms |  |
| q38 | date range and group by a title | 15.273ms |  |
| q39 | date range, group by and offset | 22.458ms |  |
| q40 | date range, a case and a wide group by | 62.104ms |  |
| q41 | date range with an IN and a hash | 6.420ms |  |
| q42 | date range and a deep offset | 8.437ms |  |
| q43 | minute buckets over a date range | 3.759ms |  |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 2.811ms | 44.197ms | 20.296ms | 0.2% | 20.296ms | 20.340ms | 20.296ms | 20.340ms | 30.000ms | 22.75 MiB | 24.58 MiB | 4.93G/s |
| q2 | filtered count | 7.262ms | 644.789ms | 40.409ms | 0.2% | 40.409ms | 40.508ms | 40.409ms | 40.508ms | 90.000ms | 60.15 MiB | 4.00 GiB | 2.47G/s |
| q3 | three aggregates | 21.804ms | 666.129ms | 40.503ms | 6.7% | 40.503ms | 43.215ms | 40.503ms | 43.215ms | 320.000ms | 136.90 MiB | 4.01 GiB | 2.47G/s |
| q4 | average | 17.904ms | 645.952ms | 43.042ms | 39.7% | 43.042ms | 60.516ms | 43.042ms | 60.516ms | 270.000ms | 219.42 MiB | 3.97 GiB | 2.32G/s |
| q5 | count distinct, high card | 154.768ms | 767.878ms | 203.828ms | 1.3% | 203.828ms | 206.519ms | 203.828ms | 206.519ms | 2.290s | 1.44 GiB | 3.97 GiB | 490.60M/s |
| q6 | count distinct, strings | 357.068ms | 706.659ms | 381.760ms | 5.2% | 381.760ms | 401.796ms | 381.760ms | 401.796ms | 1.500s | 398.30 MiB | 4.04 GiB | 261.94M/s |
| q7 | min and max of a date | 6.098ms | 646.169ms | 20.303ms | 0.1% | 20.303ms | 20.328ms | 20.303ms | 20.328ms | 80.000ms | 59.41 MiB | 3.95 GiB | 4.93G/s |
| q8 | group by, low card | 8.324ms | 666.389ms | 40.460ms | 2.0% | 40.460ms | 41.296ms | 40.460ms | 41.296ms | 110.000ms | 61.50 MiB | 4.01 GiB | 2.47G/s |
| q9 | group by and count distinct | 104.446ms | 705.888ms | 143.992ms | 0.4% | 143.992ms | 144.584ms | 143.992ms | 144.584ms | 1.640s | 683.25 MiB | 3.96 GiB | 694.47M/s |
| q10 | group by, several aggregates | 139.800ms | 807.311ms | 167.506ms | 8.6% | 167.506ms | 183.123ms | 167.506ms | 183.123ms | 2.170s | 832.61 MiB | 4.55 GiB | 596.98M/s |
| q11 | group by a string and count distinct | 30.656ms | 685.638ms | 62.429ms | 4.0% | 62.429ms | 64.998ms | 62.429ms | 64.998ms | 440.000ms | 294.88 MiB | 4.18 GiB | 1.60G/s |
| q12 | group by two strings and count distinct | 37.215ms | 686.664ms | 60.522ms | 5.1% | 60.522ms | 63.592ms | 60.522ms | 63.592ms | 550.000ms | 305.81 MiB | 4.18 GiB | 1.65G/s |
| q13 | group by a string and top k | 33.838ms | 685.467ms | 60.588ms | 2.8% | 60.588ms | 62.263ms | 60.588ms | 62.263ms | 350.000ms | 214.21 MiB | 4.04 GiB | 1.65G/s |
| q14 | group by a string and count distinct | 84.930ms | 786.287ms | 121.459ms | 2.1% | 121.459ms | 124.120ms | 121.459ms | 124.120ms | 1.200s | 605.55 MiB | 4.48 GiB | 823.30M/s |
| q15 | group by two columns and top k | 76.205ms | 707.764ms | 104.000ms | 3.8% | 104.000ms | 107.990ms | 104.000ms | 107.990ms | 1.060s | 456.44 MiB | 4.11 GiB | 961.51M/s |
| q16 | group by, very high card | 664.773us | 43.315ms | 20.340ms | 0.3% | 20.340ms | 20.393ms | 20.340ms | 20.393ms | 0.000us | 22.14 MiB | 24.79 MiB | 4.92G/s |
| q17 | group by two, very high card | 19.160ms | 785.258ms | 40.513ms | 0.1% | 40.513ms | 40.545ms | 40.513ms | 40.545ms | 20.000ms | 83.22 MiB | 2.46 GiB | 2.47G/s |
| q18 | group by two, no ordering | 34.962ms | 524.983ms | 60.552ms | 0.2% | 60.552ms | 60.662ms | 60.552ms | 60.662ms | 40.000ms | 105.83 MiB | 2.17 GiB | 1.65G/s |
| q19 | group by with an extract | 296.833ms | 953.283ms | 355.169ms | 4.4% | 355.169ms | 371.379ms | 355.169ms | 371.379ms | 4.570s | 2.04 GiB | 4.96 GiB | 281.55M/s |
| q20 | point lookup | 2.606ms | 83.435ms | 20.293ms | 0.1% | 20.293ms | 20.312ms | 20.293ms | 20.312ms | 30.000ms | 48.12 MiB | 320.03 MiB | 4.93G/s |
| q21 | substring scan | 215.760ms | 907.307ms | 242.994ms | 8.5% | 242.994ms | 263.920ms | 242.994ms | 263.920ms | 2.780s | 618.34 MiB | 5.07 GiB | 411.52M/s |
| q22 | substring scan and group by | 292.090ms | 988.216ms | 324.212ms | 6.8% | 324.212ms | 347.862ms | 324.212ms | 347.862ms | 2.910s | 785.83 MiB | 5.33 GiB | 308.43M/s |
| q23 | two substring scans and group by | 287.211ms | 1.269s | 323.715ms | 7.3% | 323.715ms | 347.427ms | 323.715ms | 347.427ms | 1.850s | 1.12 GiB | 6.73 GiB | 308.91M/s |
| q24 | select star and top k | 100.506ms | 366.703ms | 120.920ms | 1.2% | 120.920ms | 122.328ms | 120.920ms | 122.328ms | 390.000ms | 333.72 MiB | 1.34 GiB | 826.97M/s |
| q25 | top k by a date | 13.129ms | 183.209ms | 40.398ms | 0.1% | 40.398ms | 40.425ms | 40.398ms | 40.425ms | 30.000ms | 67.32 MiB | 603.00 MiB | 2.48G/s |
| q26 | top k by a string | 61.212ms | 765.680ms | 80.665ms | 25.5% | 80.665ms | 101.269ms | 80.665ms | 101.269ms | 360.000ms | 186.90 MiB | 4.08 GiB | 1.24G/s |
| q27 | top k by two columns | 43.317ms | 227.757ms | 60.515ms | 33.2% | 60.515ms | 80.589ms | 60.515ms | 80.589ms | 70.000ms | 90.13 MiB | 634.59 MiB | 1.65G/s |
| q28 | group by with a string length | 213.215ms | 854.013ms | 247.894ms | 6.8% | 247.894ms | 265.313ms | 247.894ms | 265.313ms | 2.260s | 512.28 MiB | 4.43 GiB | 403.39M/s |
| q29 | group by a regular expression | 996.138ms | 1.476s | 1.044s | 3.3% | 1.044s | 1.079s | 1.044s | 1.079s | 11.950s | 1.62 GiB | 5.63 GiB | 95.83M/s |
| q30 | ninety sums over one column | 17.477ms | 647.990ms | 40.423ms | 5.8% | 40.423ms | 42.829ms | 40.423ms | 42.829ms | 230.000ms | 103.33 MiB | 4.00 GiB | 2.47G/s |
| q31 | group by two and several aggregates | 92.967ms | 807.847ms | 124.074ms | 3.0% | 124.074ms | 127.817ms | 124.074ms | 127.817ms | 1.370s | 729.80 MiB | 4.75 GiB | 805.95M/s |
| q32 | group by a high card pair | 101.278ms | 1.029s | 143.962ms | 14.1% | 143.962ms | 164.245ms | 143.962ms | 164.245ms | 1.480s | 1.43 GiB | 6.15 GiB | 694.61M/s |
| q33 | group by a high card pair, unfiltered | 280.100ms | 1.153s | 343.979ms | 5.9% | 343.979ms | 364.564ms | 343.979ms | 364.564ms | 4.350s | 2.92 GiB | 5.90 GiB | 290.71M/s |
| q34 | group by a long string | 682.626us | 43.803ms | 20.328ms | 0.6% | 20.328ms | 20.442ms | 20.328ms | 20.442ms | 0.000us | 22.06 MiB | 24.91 MiB | 4.92G/s |
| q35 | group by a constant and a long string | 96.576ms | 725.029ms | 123.782ms | 30.1% | 123.782ms | 161.170ms | 123.782ms | 161.170ms | 770.000ms | 802.43 MiB | 4.04 GiB | 807.85M/s |
| q36 | group by four expressions | 723.089us | 43.379ms | 20.327ms | 0.2% | 20.327ms | 20.372ms | 20.327ms | 20.372ms | 0.000us | 22.66 MiB | 24.79 MiB | 4.92G/s |
| q37 | date range and group by a URL | 24.208ms | 102.635ms | 40.397ms | 0.3% | 40.397ms | 40.517ms | 40.397ms | 40.517ms | 50.000ms | 94.13 MiB | 148.91 MiB | 2.48G/s |
| q38 | date range and group by a title | 15.273ms | 82.854ms | 40.396ms | 0.0% | 40.396ms | 40.415ms | 40.396ms | 40.415ms | 40.000ms | 62.43 MiB | 107.06 MiB | 2.48G/s |
| q39 | date range, group by and offset | 22.458ms | 83.023ms | 40.407ms | 0.0% | 40.407ms | 40.420ms | 40.407ms | 40.420ms | 30.000ms | 82.14 MiB | 113.47 MiB | 2.47G/s |
| q40 | date range, a case and a wide group by | 62.104ms | 123.172ms | 80.549ms | 0.0% | 80.549ms | 80.574ms | 80.549ms | 80.574ms | 210.000ms | 183.54 MiB | 182.86 MiB | 1.24G/s |
| q41 | date range with an IN and a hash | 6.420ms | 82.691ms | 20.339ms | 0.3% | 20.339ms | 20.407ms | 20.339ms | 20.407ms | 30.000ms | 51.32 MiB | 115.86 MiB | 4.92G/s |
| q42 | date range and a deep offset | 8.437ms | 82.625ms | 40.395ms | 0.2% | 40.395ms | 40.466ms | 40.395ms | 40.466ms | 20.000ms | 45.90 MiB | 94.25 MiB | 2.48G/s |
| q43 | minute buckets over a date range | 3.759ms | 63.397ms | 20.387ms | 0.2% | 20.387ms | 20.418ms | 20.387ms | 20.418ms | 40.000ms | 42.34 MiB | 86.75 MiB | 4.90G/s |

rudb rudb 0.6.0 over 43 of 43 queries. Total 4.392s by its own clock and 5.593s by ours, 24.351s cold, 47.980s of CPU, peak 2.92 GiB, 978.94M/s and 134.75 GiB/s.

Running it cost 27% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 52.43x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | moved | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | 2.054ms | 7.230ms | 6.810ms | 6.889ms | 1.1% | 6.810ms | 173.268us | 2.938ms | 896 B | 99997498.0x | 3 of 3 |
| q2 | 3.162ms | 611.277ms | 930.065ms | 930.141ms | 0.0% | 930.065ms | 585.569us | 9.274ms | 896 B | 630501.0x | 3 of 3 |
| q3 | 2.408ms | 623.315ms | 1.228s | 1.228s | 0.0% | 1.228s | 180.274us | 2.227ms | 1.15 KiB | 99997498.0x | 3 of 3 |
| q4 | 2.385ms | 609.241ms | 1.213s | 1.213s | 0.0% | 1.213s | 172.292us | 7.244ms | 896 B | 99997498.0x | 3 of 3 |
| q5 | 2.775ms | 703.108ms | 1.615s | 1.615s | 0.0% | 1.615s | 172.361us | 1.335s | 1.13 GiB | 99997498.0x | 3 of 3 |
| q6 | 2.171ms | 656.554ms | 2.193s | 2.193s | 0.0% | 2.193s | 189.212us | 27.165ms | 248.21 MiB | 106016601.0x | 4 of 4 |
| q7 | 2.021ms | 603.712ms | 805.207ms | 805.281ms | 0.0% | 805.207ms | 207.619us | 14.512ms | 1.02 KiB | 99997498.0x | 3 of 3 |
| q8 | 4.510ms | 614.597ms | 1.061s | 1.061s | 0.0% | 1.061s | 199.734us | 18.978ms | 8.55 KiB | 35029.8x | 4 of 4 |
| q9 | 4.461ms | 648.923ms | 1.649s | 1.649s | 0.0% | 1.649s | 172.725us | 581.238ms | 380.48 MiB | 9999877.7x | 4 of 4 |
| q10 | 4.814ms | 744.305ms | 1.973s | 1.973s | 0.0% | 1.973s | 174.139us | 596.520ms | 416.69 MiB | 9999877.7x | 4 of 4 |
| q11 | 5.278ms | 642.138ms | 1.188s | 1.188s | 0.0% | 1.188s | 176.720us | 31.892ms | 33.27 MiB | 556354.2x | 4 of 4 |
| q12 | 5.062ms | 644.436ms | 1.336s | 1.336s | 0.0% | 1.336s | 194.610us | 33.598ms | 33.18 MiB | 556381.0x | 4 of 4 |
| q13 | 4.787ms | 635.492ms | 1.100s | 1.100s | 0.0% | 1.100s | 182.944us | 29.379ms | 97.33 MiB | 1317247.2x | 4 of 4 |
| q14 | 5.195ms | 728.942ms | 1.624s | 1.624s | 0.0% | 1.624s | 190.474us | 486.203ms | 332.60 MiB | 1317367.2x | 4 of 4 |
| q15 | 5.034ms | 653.263ms | 1.395s | 1.395s | 0.0% | 1.395s | 233.108us | 284.471ms | 329.60 MiB | 1317367.2x | 4 of 4 |
| q16 | 4.927ms | 26.238us | 21.852us | 26.223us | 16.7% | 21.852us | 220.112us | 0.000us | 5.09 KiB | 2.0x | 4 of 4 |
| q17 | 757.625ms | 32.062us | 27.125us | 32.017us | 15.3% | 27.125us | 295.689ms | 4.279ms | 6.71 KiB | 2.0x | 4 of 4 |
| q18 | 498.681ms | 19.770us | 13.427us | 19.715us | 31.9% | 13.427us | 42.288ms | 287.692ms | 1.21 KiB | 2.0x | 4 of 4 |
| q19 | 4.907ms | 872.835ms | 3.752s | 3.752s | 0.0% | 3.752s | 178.135us | 1.528s | 1.78 GiB | 9999877.7x | 4 of 4 |
| q20 | 1.994ms | 53.936ms | 122.529ms | 122.636ms | 0.1% | 122.529ms | 145.494us | 7.218ms | 216 B | 1.0x | 2 of 2 |
| q21 | 3.643ms | 856.784ms | 3.468s | 3.468s | 0.0% | 3.468s | 379.050us | 21.363ms | 896 B | 15912.0x | 3 of 3 |
| q22 | 4.603ms | 933.422ms | 3.742s | 3.742s | 0.0% | 3.742s | 312.937us | 17.272ms | 3.68 MiB | 105.8x | 4 of 4 |
| q23 | 5.173ms | 1.215s | 2.979s | 2.979s | 0.0% | 2.979s | 200.831us | 30.694ms | 4.20 MiB | 714.8x | 4 of 4 |
| q24 | 2.419ms | 321.727ms | 610.603ms | 617.210ms | 1.1% | 610.603ms | 190.353us | 82.599ms | 53.05 KiB | 114.8x | 4 of 4 |
| q25 | 2.226ms | 145.809ms | 209.874ms | 216.044ms | 2.9% | 209.874ms | 146.285us | 13.810ms | 53.77 KiB | 65888.4x | 4 of 4 |
| q26 | 2.019ms | 717.983ms | 1.355s | 1.361s | 0.5% | 1.355s | 149.545us | 8.911ms | 33.33 KiB | 2634478.4x | 3 of 3 |
| q27 | 2.316ms | 185.792ms | 313.744ms | 320.248ms | 2.0% | 313.744ms | 155.639us | 9.597ms | 63.66 KiB | 73718.4x | 4 of 4 |
| q28 | 4.936ms | 815.494ms | 3.007s | 3.007s | 0.0% | 3.007s | 199.120us | 12.355ms | 1.77 MiB | 3997201.4x | 5 of 5 |
| q29 | 5.240ms | 1.410s | 12.277s | 12.277s | 0.0% | 12.277s | 202.820us | 472.460ms | 686.05 MiB | 3241318.7x | 5 of 5 |
| q30 | 3.843ms | 612.787ms | 1.084s | 1.085s | 0.0% | 1.084s | 373.647us | 15.123ms | 13.04 KiB | 99997499.0x | 4 of 4 |
| q31 | 4.845ms | 754.140ms | 2.014s | 2.015s | 0.0% | 2.014s | 188.939us | 305.252ms | 329.24 MiB | 1317367.2x | 4 of 4 |
| q32 | 4.661ms | 966.274ms | 2.076s | 2.077s | 0.0% | 2.076s | 189.246us | 313.301ms | 321.09 MiB | 1317367.2x | 4 of 4 |
| q33 | 4.809ms | 1.062s | 3.108s | 3.108s | 0.0% | 3.108s | 170.321us | 2.242s | 1.93 GiB | 9999877.7x | 4 of 4 |
| q34 | 5.208ms | 28.151us | 23.797us | 28.149us | 15.5% | 23.797us | 262.509us | 0.000us | 6.04 KiB | 2.0x | 4 of 4 |
| q35 | 4.737ms | 675.046ms | 1.387s | 1.387s | 0.0% | 1.387s | 165.626us | 112.549ms | 522.91 MiB | 9999757.7x | 4 of 4 |
| q36 | 4.248ms | 39.500us | 35.889us | 39.519us | 9.2% | 35.889us | 236.135us | 9.724ms | 9.34 KiB | 3.0x | 5 of 5 |
| q37 | 4.454ms | 58.725ms | 220.987ms | 221.314ms | 0.1% | 220.987ms | 183.183us | 18.503ms | 143.48 MiB | 67158.5x | 4 of 4 |
| q38 | 4.908ms | 47.923ms | 200.742ms | 201.085ms | 0.2% | 200.742ms | 191.713us | 18.723ms | 75.45 MiB | 66042.3x | 4 of 4 |
| q39 | 5.120ms | 53.219ms | 188.198ms | 188.482ms | 0.2% | 188.198ms | 186.566us | 11.331ms | 140.23 MiB | 5582.0x | 4 of 4 |
| q40 | 4.919ms | 90.614ms | 274.818ms | 275.065ms | 0.1% | 274.818ms | 194.417us | 34.740ms | 87.58 MiB | 85196.8x | 4 of 4 |
| q41 | 4.828ms | 36.299ms | 141.447ms | 142.107ms | 0.5% | 141.447ms | 187.270us | 7.706ms | 3.53 MiB | 9013.4x | 4 of 4 |
| q42 | 4.904ms | 40.825ms | 117.736ms | 118.622ms | 0.7% | 117.736ms | 191.494us | 1.186ms | 5.77 MiB | 12269.6x | 4 of 4 |
| q43 | 4.148ms | 31.484ms | 185.741ms | 186.057ms | 0.2% | 185.741ms | 189.768us | 13.753ms | 1.50 MiB | 67439.9x | 4 of 4 |

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
| Scan | 283.886s | 89.2% | 43 | 0 | 1476043269 | handed none | 192.3ns | 43 of 43 |
| Aggregate | 32.067s | 10.1% | 39 | 1468191383 | 6105361 | 21.8ns | 5252.3ns | 39 of 39 |
| TopN | 2.339s | 0.7% | 31 | 13957206 | 340 | 167.6ns | 6880672.6ns | 31 of 31 |
| TableFetch | 39.916ms | 0.0% | 1 | 10 | 10 | 3991587.1ns | 3991587.1ns | 1 of 1 |
| Project | 5.000ms | 0.0% | 47 | 13957278 | 13957278 | 0.4ns | 0.4ns | 47 of 47 |
| Sort | 20.879us | 0.0% | 1 | 18 | 18 | 1159.9ns | 1159.9ns | 1 of 1 |
| Filter | 19.397us | 0.0% | 2 | 177 | 177 | 109.6ns | 109.6ns | 2 of 2 |
| Limit | 0.311us | 0.0% | 1 | 10 | 10 | 31.1ns | 31.1ns | 1 of 1 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is Scan, and the queries where it cost the most are q23 at 17.310s, q32 at 14.731s, q22 at 13.890s.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- q16 accounts for 21.852µs of cpu in its breakdown and the engine measured 26.223µs around the execution, which is 16.7% apart and the cross check wants under 5%
- q17 accounts for 27.125µs of cpu in its breakdown and the engine measured 32.017µs around the execution, which is 15.3% apart and the cross check wants under 5%
- q18 accounts for 13.427µs of cpu in its breakdown and the engine measured 19.715µs around the execution, which is 31.9% apart and the cross check wants under 5%
- q34 accounts for 23.797µs of cpu in its breakdown and the engine measured 28.149µs around the execution, which is 15.5% apart and the cross check wants under 5%
- q36 accounts for 35.889µs of cpu in its breakdown and the engine measured 39.519µs around the execution, which is 9.2% apart and the cross check wants under 5%

Hot here means the tries after the first, with the page cache and any server caches left as the first try left them. The server was restarted before every query's first try, so nothing carries from one query to the next.
