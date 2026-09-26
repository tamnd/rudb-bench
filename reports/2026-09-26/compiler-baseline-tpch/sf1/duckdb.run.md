# tpch on gamingpc-wsl

This is one run of the tpch suite on gamingpc-wsl, over 1 engine and 22 queries, with 4 tries of each query after a page cache drop, the way the upstream ClickBench driver runs it. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

## What ran

| what | it was |
| --- | --- |
| suite | tpch |
| queries | 22 |
| tables | 310.61 MiB of Parquet in 8 tables |
| rows | 8661245 in the table every query reads |
| corpus | duckdb-tpch SF1, corpus 5739c03f28043b03, written 2026-09-26 |
| summary | the best of the tries after the first, which is the upstream ClickBench convention |
| timeout | 600s per query, and no query reached it |
| harness | rudb-bench 0.0.1 |

## How it was measured

Every engine loaded the data first, one at a time. Then each query was run on every engine in turn, and the order rotated by one engine per query, so no engine always went first or last. Before each engine's turn the harness waited for the one minute load average to be below the 16 hardware threads, dropped the page cache (and for the ClickHouse server stopped it first and started it again after), then ran the query 4 times in a row. The first try is the cold figure and the best of the other 3 is the hot one, which is what the upstream ClickBench driver does. Every figure below is the engine's own timing where it reports one.

Every engine got the same memory budget, 20.00 GiB (21474836480 bytes, from RUDB_BENCH_MEMORY). DuckDB and rudb got it as `SET memory_limit`, the ClickHouse server as `max_server_memory_usage` and `clickhouse local` as `max_memory_usage`.

The data file, which is also written beside it as a manifest:

| path | rows | bytes | sha256 |
| --- | --- | --- | --- |
| `/home/gopher/c0-gpc/data/tpch/sf1/lineitem.parquet` | unknown | 207130015 | `a66b99aafb9882125a1a48e26a8dfdc881c31a0b48bde529e766f0970a2d3704` |
| `/home/gopher/c0-gpc/data/tpch/sf1/orders.parquet` | unknown | 56375002 | `33f9e7e14255888e66d65e437cc65f6fff0e3f9416edf3c49525d4a686259469` |
| `/home/gopher/c0-gpc/data/tpch/sf1/customer.parquet` | unknown | 12388477 | `4478dabc6ae820bdedb1274651c92012795aa98e75f23fc5c58f414625d05a25` |
| `/home/gopher/c0-gpc/data/tpch/sf1/part.parquet` | unknown | 6363519 | `91189368144f954cc593383bfb1d71ccb236b38322504062e2453295edc04806` |
| `/home/gopher/c0-gpc/data/tpch/sf1/partsupp.parquet` | unknown | 42643732 | `b8155f44b46d1e8f2e5ee80ed3df9e0ba23b486f0a27c968d7ca6895d7ed5cb4` |
| `/home/gopher/c0-gpc/data/tpch/sf1/supplier.parquet` | unknown | 793644 | `0febe07e859b36473b0904328438f09604c5d1c237112d3983b1853ffc05e177` |
| `/home/gopher/c0-gpc/data/tpch/sf1/nation.parquet` | unknown | 2311 | `2d3f229e3fa24d721df8ebf61ed05ba2b9b0a744808c857f4cad9545a1b365c8` |
| `/home/gopher/c0-gpc/data/tpch/sf1/region.parquet` | unknown | 1071 | `483f5f6a11e6bb13a3b3b3fb899df60a6418d049b75de07b15b3281867427971` |

The one minute load average at the start of each engine's turns, counting its load and every query:

| engine | readings | lowest | median | highest | held by the gate | went first |
| --- | --- | --- | --- | --- | --- | --- |
| duckdb | 23 | 0.32 | 0.32 | 0.37 | 150.360us | 22 |

The reading includes the tail of whatever ran just before, which is usually the previous engine's turn, because the one minute average takes about a minute to decay.

rudb writes summaries of each column when it loads a table, and it can answer some queries from those without reading the rows. This run turned that off with `SET stored_answers = false` before every rudb query (`RUDB_BENCH_STORED_ANSWERS=off`), so rudb read the rows the way the other engines did. Its metrics said it still answered from stored summaries for no query. The per query table marks any such query, and it also marks q1 to q7, whose shapes (counts, sums, averages, distinct counts and bounds over the whole table) are the ones a summary can answer. The headline below gives the ratios both with and without q1 to q7.

## Headline

| engine | hot total | cold total | hot geomean | total vs duckdb | geomean vs duckdb | total vs, without q1 to q7 | geomean vs, without q1 to q7 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 0.549s | 0.666s | 0.0230s | 1.000x | 1.000x | 1.000x | 1.000x |

Over the 22 queries every engine finished, 22 of them outside q1 to q7. Hot is the best of the tries after the first and cold is the first try, both by the engine's own clock. The geometric mean is over the hot figures. A ratio under 1 means faster than duckdb.

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
RUDB_BENCH_STORED_ANSWERS=off rudb-bench run tpch --engines duckdb --runs 4 --protocol upstream --timeout 600 --report
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
| duckdb | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 1.271s | 8.530s | 268.76 MiB | its own database file | its own | 0.35 to 0.35 |
| duckdb-pinned | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| clickhouse-local | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| datafusion | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| polars | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| rudb | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

The machine load column is the one minute load average before and after that engine's suite, on a machine with 16 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is a run that had to go to the device. The page cache was dropped before each query's first run, the way official ClickBench does it.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 549.000ms | 1.375s | +150% | 2.491s | 3.200s | 2.33 | 224.12 MiB | none | 347.08M/s | 12.16 GiB/s | 1.00x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 22 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | rudb from stored summaries |
| --- | --- | --- | --- |
| q01 | pricing summary report | 35.000ms |  |
| q02 | minimum cost supplier | 10.000ms |  |
| q03 | shipping priority | 23.000ms |  |
| q04 | order priority checking | 17.000ms |  |
| q05 | local supplier volume | 21.000ms |  |
| q06 | forecasting revenue change | 9.000ms |  |
| q07 | volume shipping | 34.000ms |  |
| q08 | national market share | 33.000ms |  |
| q09 | product type profit measure | 42.000ms |  |
| q10 | returned item reporting | 41.000ms |  |
| q11 | important stock identification | 10.000ms |  |
| q12 | shipping modes and order priority | 22.000ms |  |
| q13 | customer distribution | 36.000ms |  |
| q14 | promotion effect | 20.000ms |  |
| q15 | top supplier | 25.000ms |  |
| q16 | parts supplier relationship | 28.000ms |  |
| q17 | small quantity order revenue | 18.000ms |  |
| q18 | large volume customer | 28.000ms |  |
| q19 | discounted revenue | 25.000ms |  |
| q20 | potential part promotion | 22.000ms |  |
| q21 | suppliers who kept orders waiting | 28.000ms |  |
| q22 | global sales opportunity | 22.000ms |  |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 35.000ms | 124.463ms | 60.498ms | 25.0% | 60.498ms | 80.669ms | 60.498ms | 80.669ms | 170.000ms | 111.93 MiB | 127.84 MiB | 143.17M/s |
| q02 | minimum cost supplier | 10.000ms | 103.080ms | 40.465ms | 0.1% | 40.465ms | 40.521ms | 40.465ms | 40.521ms | 40.000ms | 64.83 MiB | 67.53 MiB | 214.04M/s |
| q03 | shipping priority | 23.000ms | 123.644ms | 60.531ms | 0.1% | 60.531ms | 60.601ms | 60.531ms | 60.601ms | 120.000ms | 131.45 MiB | 121.53 MiB | 143.09M/s |
| q04 | order priority checking | 17.000ms | 62.869ms | 60.498ms | 0.1% | 60.498ms | 60.560ms | 60.498ms | 60.560ms | 100.000ms | 100.93 MiB | 67.50 MiB | 143.17M/s |
| q05 | local supplier volume | 21.000ms | 103.111ms | 60.504ms | 0.2% | 60.504ms | 60.603ms | 60.504ms | 60.603ms | 130.000ms | 139.62 MiB | 129.77 MiB | 143.15M/s |
| q06 | forecasting revenue change | 9.000ms | 103.064ms | 40.409ms | 0.1% | 40.409ms | 40.462ms | 40.409ms | 40.462ms | 80.000ms | 104.68 MiB | 132.74 MiB | 214.34M/s |
| q07 | volume shipping | 34.000ms | 82.955ms | 80.610ms | 0.1% | 80.610ms | 80.687ms | 80.610ms | 80.687ms | 170.000ms | 148.79 MiB | 106.72 MiB | 107.45M/s |
| q08 | national market share | 33.000ms | 123.108ms | 80.619ms | 0.1% | 80.619ms | 80.662ms | 80.619ms | 80.662ms | 150.000ms | 146.63 MiB | 172.44 MiB | 107.43M/s |
| q09 | product type profit measure | 42.000ms | 143.099ms | 80.685ms | 0.0% | 80.685ms | 80.722ms | 80.685ms | 80.722ms | 330.000ms | 224.12 MiB | 193.11 MiB | 107.35M/s |
| q10 | returned item reporting | 41.000ms | 143.305ms | 80.644ms | 0.3% | 80.644ms | 80.852ms | 80.644ms | 80.852ms | 210.000ms | 193.43 MiB | 149.09 MiB | 107.40M/s |
| q11 | important stock identification | 10.000ms | 102.871ms | 40.429ms | 0.3% | 40.429ms | 40.531ms | 40.429ms | 40.531ms | 50.000ms | 63.56 MiB | 66.78 MiB | 214.24M/s |
| q12 | shipping modes and order priority | 22.000ms | 102.878ms | 60.514ms | 0.1% | 60.514ms | 60.553ms | 60.514ms | 60.553ms | 130.000ms | 118.85 MiB | 122.52 MiB | 143.13M/s |
| q13 | customer distribution | 36.000ms | 123.116ms | 80.668ms | 0.0% | 80.668ms | 80.700ms | 80.668ms | 80.700ms | 270.000ms | 163.50 MiB | 107.02 MiB | 107.37M/s |
| q14 | promotion effect | 20.000ms | 123.544ms | 60.569ms | 0.0% | 60.569ms | 60.582ms | 60.569ms | 60.582ms | 100.000ms | 134.46 MiB | 133.21 MiB | 143.00M/s |
| q15 | top supplier | 25.000ms | 102.936ms | 60.485ms | 0.1% | 60.485ms | 60.541ms | 60.485ms | 60.541ms | 110.000ms | 124.84 MiB | 128.09 MiB | 143.20M/s |
| q16 | parts supplier relationship | 28.000ms | 123.974ms | 61.086ms | 0.5% | 61.086ms | 61.415ms | 61.086ms | 61.415ms | 110.000ms | 104.98 MiB | 64.03 MiB | 141.79M/s |
| q17 | small quantity order revenue | 18.000ms | 103.168ms | 60.511ms | 0.1% | 60.511ms | 60.581ms | 60.511ms | 60.581ms | 120.000ms | 121.51 MiB | 125.45 MiB | 143.13M/s |
| q18 | large volume customer | 28.000ms | 123.223ms | 60.560ms | 6.0% | 60.560ms | 64.183ms | 60.560ms | 64.183ms | 280.000ms | 204.74 MiB | 114.38 MiB | 143.02M/s |
| q19 | discounted revenue | 25.000ms | 123.172ms | 60.558ms | 0.2% | 60.558ms | 60.692ms | 60.558ms | 60.692ms | 140.000ms | 142.24 MiB | 210.00 MiB | 143.02M/s |
| q20 | potential part promotion | 22.000ms | 123.698ms | 60.566ms | 0.1% | 60.566ms | 60.608ms | 60.566ms | 60.608ms | 130.000ms | 134.26 MiB | 135.43 MiB | 143.00M/s |
| q21 | suppliers who kept orders waiting | 28.000ms | 122.919ms | 62.844ms | 1.0% | 62.844ms | 63.460ms | 62.844ms | 63.460ms | 190.000ms | 130.05 MiB | 122.47 MiB | 137.82M/s |
| q22 | global sales opportunity | 22.000ms | 103.086ms | 60.506ms | 0.1% | 60.506ms | 60.571ms | 60.506ms | 60.571ms | 70.000ms | 75.07 MiB | 69.97 MiB | 143.15M/s |

duckdb v2.0.0-dev84237 (Development Version) cc7e7bac7f over 22 of 22 queries. Total 549.000ms by its own clock and 1.375s by ours, 2.491s cold, 3.200s of CPU, peak 224.12 MiB, 347.08M/s and 12.16 GiB/s.

Running it cost 150% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.00x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- duckdb is a development build, v2.0.0-dev84237 (Development Version) cc7e7bac7f, and not a DuckDB release
- every query here ran within 2.00x of every other one, so most of what was timed is whatever they have in common rather than the queries

These ran every query at about the same speed:

- duckdb ran every query within 2.00x of every other one, and this suite spreads over 6x on an engine it is measuring

A column that flat is not a column about the queries. Read those numbers as an upper bound on the engine and not as a measurement of it.

Hot here means the tries after the first, with the page cache and any server caches left as the first try left them. The server was restarted before every query's first try, so nothing carries from one query to the next.
