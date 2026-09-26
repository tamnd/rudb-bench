# tpch on gamingpc-wsl

This is one run of the tpch suite on gamingpc-wsl, over 1 engine and 22 queries, with 4 tries of each query after a page cache drop, the way the upstream ClickBench driver runs it. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

## What ran

| what | it was |
| --- | --- |
| suite | tpch |
| queries | 22 |
| tables | 3.21 GiB of Parquet in 8 tables |
| rows | 86612180 in the table every query reads |
| corpus | duckdb-tpch SF10, corpus 02eb57f3a084bd10, written 2026-09-26 |
| summary | the best of the tries after the first, which is the upstream ClickBench convention |
| timeout | 600s per query, and no query reached it |
| harness | rudb-bench 0.0.1 |

## How it was measured

Every engine loaded the data first, one at a time. Then each query was run on every engine in turn, and the order rotated by one engine per query, so no engine always went first or last. Before each engine's turn the harness waited for the one minute load average to be below the 16 hardware threads, dropped the page cache (and for the ClickHouse server stopped it first and started it again after), then ran the query 4 times in a row. The first try is the cold figure and the best of the other 3 is the hot one, which is what the upstream ClickBench driver does. Every figure below is the engine's own timing where it reports one.

Every engine got the same memory budget, 20.00 GiB (21474836480 bytes, from RUDB_BENCH_MEMORY). DuckDB and rudb got it as `SET memory_limit`, the ClickHouse server as `max_server_memory_usage` and `clickhouse local` as `max_memory_usage`.

The data file, which is also written beside it as a manifest:

| path | rows | bytes | sha256 |
| --- | --- | --- | --- |
| `/home/gopher/c0-gpc/data/tpch/sf10/lineitem.parquet` | unknown | 2223320375 | `3367ae6f01bed55b66d08a490d6259495dd06373c656cde472ae776dfed08f20` |
| `/home/gopher/c0-gpc/data/tpch/sf10/orders.parquet` | unknown | 584139373 | `fe9836f7ee4d9a2a00c048953f7c2459895567b35afd2946388f7282f46eb3ef` |
| `/home/gopher/c0-gpc/data/tpch/sf10/customer.parquet` | unknown | 123774677 | `5ead990cb99ee306f9bf19661fa63e4b682a402e2369cd553c6e0d42645dc7ac` |
| `/home/gopher/c0-gpc/data/tpch/sf10/part.parquet` | unknown | 63720703 | `72ec41325e610e09b4de3ce4c5434967bb12243379121d5c2ce6be5dc1e64f63` |
| `/home/gopher/c0-gpc/data/tpch/sf10/partsupp.parquet` | unknown | 439923896 | `91b0717199118a0b65a0c24b353f69098aa5b80385d1259b7a7ece44b3e64e73` |
| `/home/gopher/c0-gpc/data/tpch/sf10/supplier.parquet` | unknown | 7904894 | `672e61a4ade385d63dd89bd98a5e816c3a66f8bea4ca7cd2457e9a72e583893c` |
| `/home/gopher/c0-gpc/data/tpch/sf10/nation.parquet` | unknown | 2311 | `2d3f229e3fa24d721df8ebf61ed05ba2b9b0a744808c857f4cad9545a1b365c8` |
| `/home/gopher/c0-gpc/data/tpch/sf10/region.parquet` | unknown | 1071 | `483f5f6a11e6bb13a3b3b3fb899df60a6418d049b75de07b15b3281867427971` |

The one minute load average at the start of each engine's turns, counting its load and every query:

| engine | readings | lowest | median | highest | held by the gate | went first |
| --- | --- | --- | --- | --- | --- | --- |
| rudb | 23 | 1.75 | 1.83 | 1.85 | 90.000s | 22 |

The reading includes the tail of whatever ran just before, which is usually the previous engine's turn, because the one minute average takes about a minute to decay.

rudb writes summaries of each column when it loads a table, and it can answer some queries from those without reading the rows. This run turned that off with `SET stored_answers = false` before every rudb query (`RUDB_BENCH_STORED_ANSWERS=off`), so rudb read the rows the way the other engines did. Its metrics said it still answered from stored summaries for no query. The per query table marks any such query, and it also marks q1 to q7, whose shapes (counts, sums, averages, distinct counts and bounds over the whole table) are the ones a summary can answer. The headline below gives the ratios both with and without q1 to q7.

## Headline

| engine | hot total | cold total | hot geomean | total vs rudb | geomean vs rudb | total vs, without q1 to q7 | geomean vs, without q1 to q7 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| rudb | 1.687s | 6.723s | 0.0578s | 1.000x | 1.000x | 1.000x | 1.000x |

Over the 22 queries every engine finished, 22 of them outside q1 to q7. Hot is the best of the tries after the first and cold is the first try, both by the engine's own clock. The geometric mean is over the hot figures. A ratio under 1 means faster than rudb.

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
RUDB_BENCH_STORED_ANSWERS=off rudb-bench run tpch --engines rudb --runs 4 --protocol upstream --timeout 600 --report
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
| rudb | rudb 0.6.0 | ran | 5.336s | 66.530s | 2.49 GiB | its own database file | its own | 1.75 to 3.06 |
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
| rudb | 1.687s | 2.200s | +30% | 7.565s | 15.960s | 7.25 | 1.16 GiB | none | 1.13G/s | 41.81 GiB/s | 1.00x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 22 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | rudb | rudb from stored summaries |
| --- | --- | --- | --- |
| q01 | pricing summary report | 70.807ms |  |
| q02 | minimum cost supplier | 16.786ms |  |
| q03 | shipping priority | 83.310ms |  |
| q04 | order priority checking | 73.206ms |  |
| q05 | local supplier volume | 77.102ms |  |
| q06 | forecasting revenue change | 15.978ms |  |
| q07 | volume shipping | 57.421ms |  |
| q08 | national market share | 71.364ms |  |
| q09 | product type profit measure | 207.481ms |  |
| q10 | returned item reporting | 93.455ms |  |
| q11 | important stock identification | 19.028ms |  |
| q12 | shipping modes and order priority | 95.310ms |  |
| q13 | customer distribution | 148.386ms |  |
| q14 | promotion effect | 36.600ms |  |
| q15 | top supplier | 31.613ms |  |
| q16 | parts supplier relationship | 50.262ms |  |
| q17 | small quantity order revenue | 34.949ms |  |
| q18 | large volume customer | 118.255ms |  |
| q19 | discounted revenue | 27.693ms |  |
| q20 | potential part promotion | 56.076ms |  |
| q21 | suppliers who kept orders waiting | 272.068ms |  |
| q22 | global sales opportunity | 29.959ms |  |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 70.807ms | 307.925ms | 83.765ms | 24.1% | 83.765ms | 103.979ms | 83.765ms | 103.979ms | 1.100s | 445.58 MiB | 1.63 GiB | 1.03G/s |
| q02 | minimum cost supplier | 16.786ms | 104.355ms | 40.654ms | 0.7% | 40.654ms | 40.935ms | 40.654ms | 40.935ms | 100.000ms | 102.35 MiB | 411.09 MiB | 2.13G/s |
| q03 | shipping priority | 83.310ms | 446.138ms | 101.245ms | 21.4% | 101.245ms | 123.122ms | 101.245ms | 123.122ms | 660.000ms | 701.05 MiB | 2.10 GiB | 855.47M/s |
| q04 | order priority checking | 73.206ms | 385.568ms | 103.984ms | 3.2% | 103.984ms | 107.325ms | 103.984ms | 107.325ms | 700.000ms | 483.66 MiB | 2.02 GiB | 832.94M/s |
| q05 | local supplier volume | 77.102ms | 425.591ms | 102.181ms | 2.9% | 102.181ms | 105.168ms | 102.181ms | 105.168ms | 810.000ms | 725.26 MiB | 2.11 GiB | 847.64M/s |
| q06 | forecasting revenue change | 15.978ms | 285.123ms | 40.564ms | 0.7% | 40.564ms | 40.864ms | 40.564ms | 40.864ms | 230.000ms | 412.66 MiB | 1.62 GiB | 2.14G/s |
| q07 | volume shipping | 57.421ms | 405.115ms | 80.864ms | 3.9% | 80.864ms | 84.086ms | 80.864ms | 84.086ms | 660.000ms | 734.38 MiB | 2.11 GiB | 1.07G/s |
| q08 | national market share | 71.364ms | 424.984ms | 103.975ms | 3.8% | 103.975ms | 107.898ms | 103.975ms | 107.898ms | 600.000ms | 759.46 MiB | 2.15 GiB | 833.01M/s |
| q09 | product type profit measure | 207.481ms | 526.961ms | 243.775ms | 0.5% | 243.775ms | 245.071ms | 243.775ms | 245.071ms | 1.950s | 1.16 GiB | 2.39 GiB | 355.30M/s |
| q10 | returned item reporting | 93.455ms | 445.464ms | 120.961ms | 3.1% | 120.961ms | 124.741ms | 120.961ms | 124.741ms | 950.000ms | 754.82 MiB | 2.11 GiB | 716.03M/s |
| q11 | important stock identification | 19.028ms | 103.627ms | 40.575ms | 0.1% | 40.575ms | 40.621ms | 40.575ms | 40.621ms | 130.000ms | 137.41 MiB | 378.48 MiB | 2.13G/s |
| q12 | shipping modes and order priority | 95.310ms | 404.457ms | 121.222ms | 15.9% | 121.222ms | 141.150ms | 121.222ms | 141.150ms | 770.000ms | 929.55 MiB | 2.02 GiB | 714.49M/s |
| q13 | customer distribution | 148.386ms | 228.047ms | 162.826ms | 3.5% | 162.826ms | 168.555ms | 162.826ms | 168.555ms | 2.100s | 244.14 MiB | 495.78 MiB | 531.93M/s |
| q14 | promotion effect | 36.600ms | 324.057ms | 60.641ms | 3.6% | 60.641ms | 62.821ms | 60.641ms | 62.821ms | 370.000ms | 613.37 MiB | 1.67 GiB | 1.43G/s |
| q15 | top supplier | 31.613ms | 304.854ms | 60.680ms | 1.7% | 60.680ms | 61.734ms | 60.680ms | 61.734ms | 380.000ms | 592.36 MiB | 1.63 GiB | 1.43G/s |
| q16 | parts supplier relationship | 50.262ms | 125.809ms | 62.512ms | 0.2% | 62.512ms | 62.651ms | 62.512ms | 62.651ms | 360.000ms | 102.59 MiB | 406.73 MiB | 1.39G/s |
| q17 | small quantity order revenue | 34.949ms | 364.347ms | 60.719ms | 2.2% | 60.719ms | 62.062ms | 60.719ms | 62.062ms | 410.000ms | 475.07 MiB | 1.68 GiB | 1.43G/s |
| q18 | large volume customer | 118.255ms | 468.895ms | 141.061ms | 16.1% | 141.061ms | 164.084ms | 141.061ms | 164.084ms | 860.000ms | 762.26 MiB | 2.11 GiB | 614.01M/s |
| q19 | discounted revenue | 27.693ms | 305.352ms | 43.929ms | 40.9% | 43.929ms | 61.895ms | 43.929ms | 61.895ms | 370.000ms | 540.28 MiB | 1.67 GiB | 1.97G/s |
| q20 | potential part promotion | 56.076ms | 387.058ms | 81.020ms | 2.5% | 81.020ms | 83.071ms | 81.020ms | 83.071ms | 550.000ms | 625.64 MiB | 1.99 GiB | 1.07G/s |
| q21 | suppliers who kept orders waiting | 272.068ms | 647.345ms | 302.418ms | 7.1% | 302.418ms | 324.147ms | 302.418ms | 324.147ms | 1.640s | 1.09 GiB | 2.02 GiB | 286.40M/s |
| q22 | global sales opportunity | 29.959ms | 143.797ms | 40.635ms | 0.3% | 40.635ms | 40.766ms | 40.635ms | 40.766ms | 260.000ms | 181.93 MiB | 513.90 MiB | 2.13G/s |

rudb rudb 0.6.0 over 22 of 22 queries. Total 1.687s by its own clock and 2.200s by ours, 7.565s cold, 15.960s of CPU, peak 1.16 GiB, 1.13G/s and 41.81 GiB/s.

Running it cost 30% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 7.52x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | moved | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | 1.297ms | 261.180ms | 1.533s | 1.533s | 0.0% | 1.533s | 183.289us | 7.047ms | 25.03 KiB | 14785655.2x | 5 of 5 |
| q02 | 1.419ms | 74.328ms | 267.462ms | 267.539ms | 0.0% | 267.462ms | 263.034us | 12.198ms | 32.53 MiB | 1799.8x | 33 of 33 |
| q03 | 2.413ms | 393.771ms | 1.463s | 1.464s | 0.0% | 1.463s | 198.251us | 76.201ms | 142.42 MiB | 405635.6x | 10 of 10 |
| q04 | 1.954ms | 345.298ms | 1.151s | 1.151s | 0.0% | 1.151s | 203.694us | 78.671ms | 243.09 MiB | 602380.6x | 7 of 7 |
| q05 | 2.809ms | 385.120ms | 1.451s | 1.451s | 0.0% | 1.451s | 227.866us | 118.733ms | 149.58 MiB | 1963000.4x | 19 of 19 |
| q06 | 844.853us | 255.627ms | 768.290ms | 768.355ms | 0.0% | 768.290ms | 165.423us | 1.480ms | 904 B | 1139265.0x | 3 of 3 |
| q07 | 2.797ms | 369.584ms | 1.537s | 1.537s | 0.0% | 1.537s | 351.312us | 62.293ms | 86.98 MiB | 784089.5x | 21 of 21 |
| q08 | 3.371ms | 385.286ms | 1.197s | 1.198s | 0.0% | 1.197s | 269.112us | 72.111ms | 61.03 MiB | 635441.5x | 26 of 26 |
| q09 | 2.922ms | 483.897ms | 2.515s | 2.515s | 0.0% | 2.515s | 329.056us | 294.298ms | 458.63 MiB | 245021.9x | 20 of 20 |
| q10 | 2.664ms | 391.424ms | 2.071s | 2.071s | 0.0% | 2.071s | 365.905us | 68.136ms | 247.45 MiB | 266408.6x | 14 of 14 |
| q11 | 1.476ms | 74.984ms | 241.847ms | 242.389ms | 0.2% | 241.847ms | 195.568us | 7.415ms | 33.19 MiB | not read | 18 of 18 |
| q12 | 1.700ms | 365.636ms | 1.157s | 1.157s | 0.0% | 1.157s | 195.673us | 62.763ms | 474.69 MiB | 533226.5x | 7 of 7 |
| q13 | 1.329ms | 186.180ms | 2.337s | 2.337s | 0.0% | 2.337s | 193.937us | 43.260ms | 76.15 MiB | 485454.2x | 10 of 10 |
| q14 | 969.774us | 276.940ms | 784.083ms | 784.142ms | 0.0% | 784.083ms | 174.101us | 95.684ms | 41.57 MiB | 2123357.0x | 6 of 6 |
| q15 | 1.999ms | 255.185ms | 815.562ms | 815.631ms | 0.0% | 815.562ms | 319.712us | 14.049ms | 30.80 MiB | 3065718.0x | 17 of 17 |
| q16 | 1.593ms | 79.156ms | 431.836ms | 431.866ms | 0.0% | 431.836ms | 184.864us | 27.949ms | 42.40 MiB | 183.2x | 12 of 12 |
| q17 | 1.403ms | 328.080ms | 825.634ms | 830.193ms | 0.5% | 825.634ms | 370.438us | 9.436ms | 30.98 MiB | 320628.0x | 15 of 15 |
| q18 | 2.867ms | 423.009ms | 1.632s | 1.634s | 0.1% | 1.632s | 342.994us | 65.745ms | 543.71 MiB | 766411.1x | 17 of 17 |
| q19 | 1.487ms | 263.005ms | 895.558ms | 895.679ms | 0.0% | 895.558ms | 223.670us | 4.097ms | 30.83 MiB | 11993.0x | 7 of 7 |
| q20 | 1.563ms | 338.122ms | 1.231s | 1.231s | 0.0% | 1.231s | 273.113us | 18.633ms | 44.12 MiB | 564.7x | 28 of 28 |
| q21 | 2.549ms | 601.052ms | 1.953s | 1.957s | 0.2% | 1.953s | 287.538us | 262.519ms | 463.65 MiB | 169462.1x | 19 of 19 |
| q22 | 1.917ms | 108.319ms | 459.832ms | 460.005ms | 0.0% | 459.832ms | 262.496us | 19.732ms | 96.12 MiB | 491870.7x | 14 of 14 |

Read from the breakdown the engine wrote for its cold run. `planning` is everything before the first row moved, which is the parse, the bind, the optimizer passes and building the tree. It is a column rather than a footnote because it is the one cost of a query nobody profiles: an optimizer only ever has passes added to it, each paying for itself on the query it was written for, and a query that plans for four hundred milliseconds to save two hundred is a query the optimizer made slower. What is gated is planning as a share of the whole statement, and that share is not in this table, because it is taken over every run rather than off the cold one the rest of these columns come from. It lives in `baselines/planning-<suite>.txt`. `accounted` is what its pipelines charged themselves and `measured` is what it measured around running them, so `apart` is the cross check and anything over five percent is time the breakdown cannot explain. `driver` is the part of `accounted` that was not inside an operator, which is the loop that runs a pipeline rather than the operators it calls. `build` is what the engine spent putting the tree together, which is before there is a pipeline to charge and is why it is off the right hand side of the check. `outside` is the CPU the process spent on everything else, which is starting, opening the data and printing the answer, and it is the reason the wall clock column and the query time column differ. `held` is what the operators reserved, which is not the resident set of the process. `moved` is every intermediate row the plan built divided by the rows it returned, which is the one column here that does not move when the kernels get faster: a suite that gets thirty percent quicker on a rewritten hash table reports thirty percent everywhere else and nothing at all here, and when this falls it is because the plans changed. `reference` is how many operators ran the reference implementation of their seam, which is the slow path kept for differential testing.

#### What the planner knew

| query | decisions | exact | certified | estimated | unknown |
| --- | --- | --- | --- | --- | --- |
| q01 | 5 | 0 | 0 | 5 | 0 |
| q02 | 33 | 6 | 0 | 18 | 9 |
| q03 | 10 | 0 | 0 | 8 | 2 |
| q04 | 7 | 0 | 0 | 6 | 1 |
| q05 | 19 | 4 | 0 | 10 | 5 |
| q06 | 3 | 2 | 0 | 1 | 0 |
| q07 | 21 | 3 | 0 | 13 | 5 |
| q08 | 26 | 5 | 0 | 14 | 7 |
| q09 | 20 | 5 | 0 | 10 | 5 |
| q10 | 14 | 2 | 0 | 9 | 3 |
| q11 | 18 | 4 | 0 | 4 | 10 |
| q12 | 7 | 1 | 0 | 5 | 1 |
| q13 | 10 | 1 | 0 | 8 | 1 |
| q14 | 6 | 3 | 0 | 2 | 1 |
| q15 | 17 | 3 | 0 | 4 | 10 |
| q16 | 12 | 1 | 0 | 9 | 2 |
| q17 | 15 | 3 | 0 | 9 | 3 |
| q18 | 17 | 4 | 0 | 10 | 3 |
| q19 | 7 | 2 | 0 | 4 | 1 |
| q20 | 28 | 3 | 0 | 19 | 6 |
| q21 | 19 | 1 | 0 | 13 | 5 |
| q22 | 14 | 2 | 0 | 10 | 2 |
| whole suite | 328 | 55 | 0 | 191 | 82 |

One row per query and one decision per operator, counted by what the planner had behind the cardinality it used. Over the whole suite that is 17% exact, 0% certified, 58% estimated and 25% unknown. `exact` is a counted number, `certified` is a number with a proven bound, `estimated` is a guess, and `unknown` is no number at all, which is an answer an operator has to handle rather than a null to paper over. Counts rather than fractions in the table because a suite's histogram is the sum of its queries' and fractions do not add.

#### Where the time went

| kind | cpu | share | operators | rows in | rows out | per row in | per row out | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Scan | 84.622s | 92.9% | 87 | 0 | 202582410 | handed none | 417.7ns | 87 of 87 |
| Aggregate | 3.357s | 3.7% | 28 | 147908794 | 16998085 | 22.7ns | 197.5ns | 28 of 28 |
| Probe | 1.832s | 2.0% | 59 | 32973651 | 30805883 | 55.5ns | 59.5ns | 59 of 59 |
| Mark | 732.710ms | 0.8% | 5 | 16039408 | 2140498 | 45.7ns | 342.3ns | 5 of 5 |
| Gather | 260.195ms | 0.3% | 67 | 39695706 | 0 | 6.6ns | handed on none | 67 of 67 |
| Project | 195.493ms | 0.2% | 45 | 7750773 | 7750773 | 25.2ns | 25.2ns | 45 of 45 |
| Filter | 58.855ms | 0.1% | 10 | 17286050 | 1506265 | 3.4ns | 39.1ns | 10 of 10 |
| TopN | 12.345ms | 0.0% | 5 | 504408 | 330 | 24.5ns | 37408.5ns | 5 of 5 |
| Sort | 4.782ms | 0.0% | 13 | 29894 | 29894 | 160.0ns | 160.0ns | 13 of 13 |
| CteScan | 1.915ms | 0.0% | 4 | 0 | 809548 | handed none | 2.4ns | 4 of 4 |
| MaterializedCTE | 294.886us | 0.0% | 2 | 404774 | 0 | 0.7ns | handed on none | 2 of 2 |
| Broadcast | 107.225us | 0.0% | 3 | 824748 | 824748 | 0.1ns | 0.1ns | 3 of 3 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is Scan, and the queries where it cost the most are q03 at 5.366s, q07 at 5.327s, q08 at 5.283s.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven

Hot here means the tries after the first, with the page cache and any server caches left as the first try left them. The server was restarted before every query's first try, so nothing carries from one query to the next.
