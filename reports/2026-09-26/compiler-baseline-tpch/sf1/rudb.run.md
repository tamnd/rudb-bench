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
| rudb | 23 | 0.37 | 0.58 | 0.58 | 129.886us | 22 |

The reading includes the tail of whatever ran just before, which is usually the previous engine's turn, because the one minute average takes about a minute to decay.

rudb writes summaries of each column when it loads a table, and it can answer some queries from those without reading the rows. This run turned that off with `SET stored_answers = false` before every rudb query (`RUDB_BENCH_STORED_ANSWERS=off`), so rudb read the rows the way the other engines did. Its metrics said it still answered from stored summaries for no query. The per query table marks any such query, and it also marks q1 to q7, whose shapes (counts, sums, averages, distinct counts and bounds over the whole table) are the ones a summary can answer. The headline below gives the ratios both with and without q1 to q7.

## Headline

| engine | hot total | cold total | hot geomean | total vs rudb | geomean vs rudb | total vs, without q1 to q7 | geomean vs, without q1 to q7 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| rudb | 0.233s | 0.976s | 0.0090s | 1.000x | 1.000x | 1.000x | 1.000x |

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
| rudb | rudb 0.6.0 | ran | 832.459ms | 6.670s | 253.10 MiB | its own database file | its own | 0.37 to 0.37 |
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
| rudb | 232.834ms | 511.626ms | +120% | 1.511s | 1.220s | 2.38 | 135.30 MiB | none | 818.38M/s | 28.66 GiB/s | 1.00x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 22 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | rudb | rudb from stored summaries |
| --- | --- | --- | --- |
| q01 | pricing summary report | 10.331ms |  |
| q02 | minimum cost supplier | 5.045ms |  |
| q03 | shipping priority | 10.616ms |  |
| q04 | order priority checking | 7.058ms |  |
| q05 | local supplier volume | 11.837ms |  |
| q06 | forecasting revenue change | 3.041ms |  |
| q07 | volume shipping | 9.742ms |  |
| q08 | national market share | 8.696ms |  |
| q09 | product type profit measure | 20.428ms |  |
| q10 | returned item reporting | 13.138ms |  |
| q11 | important stock identification | 4.988ms |  |
| q12 | shipping modes and order priority | 13.223ms |  |
| q13 | customer distribution | 12.620ms |  |
| q14 | promotion effect | 5.363ms |  |
| q15 | top supplier | 5.753ms |  |
| q16 | parts supplier relationship | 19.622ms |  |
| q17 | small quantity order revenue | 6.201ms |  |
| q18 | large volume customer | 13.874ms |  |
| q19 | discounted revenue | 4.821ms |  |
| q20 | potential part promotion | 8.645ms |  |
| q21 | suppliers who kept orders waiting | 32.110ms |  |
| q22 | global sales opportunity | 5.679ms |  |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 10.331ms | 63.358ms | 20.410ms | 0.1% | 20.410ms | 20.434ms | 20.410ms | 20.434ms | 100.000ms | 73.38 MiB | 186.50 MiB | 424.36M/s |
| q02 | minimum cost supplier | 5.045ms | 42.866ms | 20.572ms | 0.1% | 20.572ms | 20.598ms | 20.572ms | 20.598ms | 20.000ms | 34.74 MiB | 59.71 MiB | 421.03M/s |
| q03 | shipping priority | 10.616ms | 83.611ms | 20.458ms | 0.1% | 20.458ms | 20.472ms | 20.458ms | 20.472ms | 60.000ms | 84.26 MiB | 217.27 MiB | 423.36M/s |
| q04 | order priority checking | 7.058ms | 63.158ms | 20.419ms | 0.3% | 20.419ms | 20.489ms | 20.419ms | 20.489ms | 30.000ms | 60.72 MiB | 208.61 MiB | 424.17M/s |
| q05 | local supplier volume | 11.837ms | 82.819ms | 20.479ms | 0.6% | 20.479ms | 20.607ms | 20.479ms | 20.607ms | 70.000ms | 82.36 MiB | 228.98 MiB | 422.93M/s |
| q06 | forecasting revenue change | 3.041ms | 42.731ms | 20.392ms | 0.4% | 20.392ms | 20.466ms | 20.392ms | 20.466ms | 20.000ms | 54.41 MiB | 175.20 MiB | 424.75M/s |
| q07 | volume shipping | 9.742ms | 83.688ms | 20.540ms | 0.1% | 20.540ms | 20.556ms | 20.540ms | 20.556ms | 60.000ms | 85.96 MiB | 218.03 MiB | 421.67M/s |
| q08 | national market share | 8.696ms | 83.255ms | 20.542ms | 0.1% | 20.542ms | 20.567ms | 20.542ms | 20.567ms | 50.000ms | 90.57 MiB | 222.64 MiB | 421.63M/s |
| q09 | product type profit measure | 20.428ms | 102.999ms | 40.616ms | 0.1% | 40.616ms | 40.645ms | 40.616ms | 40.645ms | 140.000ms | 135.30 MiB | 249.14 MiB | 213.25M/s |
| q10 | returned item reporting | 13.138ms | 82.825ms | 20.416ms | 0.2% | 20.416ms | 20.460ms | 20.416ms | 20.460ms | 70.000ms | 99.16 MiB | 224.22 MiB | 424.24M/s |
| q11 | important stock identification | 4.988ms | 42.800ms | 20.531ms | 0.2% | 20.531ms | 20.568ms | 20.531ms | 20.568ms | 10.000ms | 40.05 MiB | 52.51 MiB | 421.86M/s |
| q12 | shipping modes and order priority | 13.223ms | 83.134ms | 20.421ms | 0.1% | 20.421ms | 20.436ms | 20.421ms | 20.436ms | 70.000ms | 105.61 MiB | 208.68 MiB | 424.14M/s |
| q13 | customer distribution | 12.620ms | 42.895ms | 20.418ms | 0.2% | 20.418ms | 20.453ms | 20.418ms | 20.453ms | 100.000ms | 79.23 MiB | 56.33 MiB | 424.19M/s |
| q14 | promotion effect | 5.363ms | 62.965ms | 20.455ms | 0.1% | 20.455ms | 20.469ms | 20.455ms | 20.469ms | 20.000ms | 75.72 MiB | 184.91 MiB | 423.42M/s |
| q15 | top supplier | 5.753ms | 63.215ms | 20.507ms | 0.2% | 20.507ms | 20.540ms | 20.507ms | 20.540ms | 30.000ms | 79.38 MiB | 176.55 MiB | 422.36M/s |
| q16 | parts supplier relationship | 19.622ms | 64.103ms | 41.497ms | 0.8% | 41.497ms | 41.810ms | 41.497ms | 41.810ms | 30.000ms | 43.48 MiB | 59.98 MiB | 208.72M/s |
| q17 | small quantity order revenue | 6.201ms | 62.775ms | 20.497ms | 0.0% | 20.497ms | 20.502ms | 20.497ms | 20.502ms | 40.000ms | 70.83 MiB | 195.86 MiB | 422.57M/s |
| q18 | large volume customer | 13.874ms | 62.765ms | 20.436ms | 0.2% | 20.436ms | 20.473ms | 20.436ms | 20.473ms | 60.000ms | 90.89 MiB | 202.56 MiB | 423.82M/s |
| q19 | discounted revenue | 4.821ms | 62.721ms | 20.449ms | 0.1% | 20.449ms | 20.473ms | 20.449ms | 20.473ms | 40.000ms | 65.70 MiB | 185.04 MiB | 423.56M/s |
| q20 | potential part promotion | 8.645ms | 84.707ms | 20.566ms | 0.2% | 20.566ms | 20.603ms | 20.566ms | 20.603ms | 50.000ms | 78.59 MiB | 217.36 MiB | 421.15M/s |
| q21 | suppliers who kept orders waiting | 32.110ms | 104.312ms | 40.535ms | 0.1% | 40.535ms | 40.589ms | 40.535ms | 40.589ms | 130.000ms | 124.47 MiB | 208.62 MiB | 213.67M/s |
| q22 | global sales opportunity | 5.679ms | 42.909ms | 20.469ms | 0.3% | 20.469ms | 20.525ms | 20.469ms | 20.525ms | 20.000ms | 30.42 MiB | 69.25 MiB | 423.13M/s |

rudb rudb 0.6.0 over 22 of 22 queries. Total 232.834ms by its own clock and 511.626ms by ours, 1.511s cold, 1.220s of CPU, peak 135.30 MiB, 818.38M/s and 28.66 GiB/s.

Running it cost 120% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.04x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | moved | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | 1.305ms | 29.304ms | 161.208ms | 161.222ms | 0.0% | 161.208ms | 140.752us | 0.000us | 25.03 KiB | 1479150.8x | 5 of 5 |
| q02 | 1.010ms | 24.142ms | 98.960ms | 99.115ms | 0.2% | 98.960ms | 250.313us | 634.376us | 10.09 MiB | 176.1x | 33 of 33 |
| q03 | 2.211ms | 53.230ms | 262.844ms | 263.034ms | 0.1% | 262.844ms | 167.749us | 0.000us | 14.33 MiB | 40867.2x | 10 of 10 |
| q04 | 1.448ms | 45.587ms | 188.930ms | 189.055ms | 0.1% | 188.930ms | 138.945us | 805.637us | 24.31 MiB | 50924.0x | 7 of 7 |
| q05 | 2.278ms | 52.680ms | 179.291ms | 179.865ms | 0.3% | 179.291ms | 190.887us | 0.000us | 14.96 MiB | 196166.0x | 19 of 19 |
| q06 | 521.813us | 26.847ms | 86.217ms | 86.235ms | 0.0% | 86.217ms | 129.322us | 3.636ms | 904 B | 114161.0x | 3 of 3 |
| q07 | 2.385ms | 50.430ms | 202.587ms | 203.098ms | 0.3% | 202.587ms | 215.567us | 0.000us | 8.75 MiB | 78822.2x | 21 of 21 |
| q08 | 2.693ms | 55.197ms | 213.799ms | 214.396ms | 0.3% | 213.799ms | 234.822us | 5.369ms | 7.42 MiB | 90880.0x | 26 of 26 |
| q09 | 2.485ms | 71.159ms | 364.238ms | 364.283ms | 0.0% | 364.238ms | 194.171us | 25.523ms | 46.01 MiB | 24271.2x | 20 of 20 |
| q10 | 2.235ms | 48.473ms | 218.830ms | 218.980ms | 0.1% | 218.830ms | 200.041us | 819.737us | 24.75 MiB | 25715.3x | 14 of 14 |
| q11 | 883.984us | 25.803ms | 98.940ms | 99.006ms | 0.1% | 98.940ms | 198.341us | 0.000us | 12.56 MiB | 177.0x | 18 of 18 |
| q12 | 1.720ms | 52.091ms | 182.912ms | 183.065ms | 0.1% | 182.912ms | 152.127us | 6.782ms | 47.49 MiB | 45539.5x | 7 of 7 |
| q13 | 1.262ms | 20.933ms | 145.853ms | 145.870ms | 0.0% | 145.853ms | 199.627us | 0.000us | 38.91 MiB | 52009.0x | 10 of 10 |
| q14 | 834.869us | 36.104ms | 120.487ms | 120.508ms | 0.0% | 120.487ms | 157.191us | 0.000us | 4.21 MiB | 215079.0x | 6 of 6 |
| q15 | 1.210ms | 29.714ms | 128.887ms | 128.910ms | 0.0% | 128.887ms | 247.017us | 843.119us | 8.63 MiB | 305958.0x | 17 of 17 |
| q16 | 1.121ms | 33.098ms | 110.892ms | 110.960ms | 0.1% | 110.892ms | 191.848us | 8.848ms | 7.76 MiB | 29.5x | 12 of 12 |
| q17 | 974.427us | 38.846ms | 129.356ms | 129.769ms | 0.3% | 129.356ms | 220.489us | 10.226us | 13.41 MiB | 31844.0x | 15 of 15 |
| q18 | 2.191ms | 43.105ms | 143.317ms | 143.565ms | 0.2% | 143.317ms | 217.212us | 0.000us | 49.98 MiB | 134253.8x | 17 of 17 |
| q19 | 1.784ms | 39.863ms | 150.713ms | 151.082ms | 0.2% | 150.713ms | 197.891us | 0.000us | 3.09 MiB | 1253.0x | 7 of 7 |
| q20 | 1.492ms | 57.634ms | 194.499ms | 194.616ms | 0.1% | 194.499ms | 236.360us | 0.000us | 4.45 MiB | 542.5x | 28 of 28 |
| q21 | 1.838ms | 72.945ms | 283.947ms | 284.526ms | 0.2% | 283.947ms | 209.149us | 5.265ms | 47.72 MiB | 17349.2x | 19 of 19 |
| q22 | 1.420ms | 18.342ms | 72.457ms | 72.610ms | 0.2% | 72.457ms | 202.437us | 7.188ms | 9.51 MiB | 49143.3x | 14 of 14 |

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
| Scan | 9.823s | 93.8% | 87 | 0 | 20222816 | handed none | 485.7ns | 87 of 87 |
| Aggregate | 301.021ms | 2.9% | 28 | 14788105 | 1714943 | 20.4ns | 175.5ns | 28 of 28 |
| Probe | 154.027ms | 1.5% | 59 | 3243880 | 3083830 | 47.5ns | 49.9ns | 59 of 59 |
| Mark | 75.089ms | 0.7% | 5 | 1592689 | 218687 | 47.1ns | 343.4ns | 5 of 5 |
| Project | 69.205ms | 0.7% | 45 | 785148 | 785148 | 88.1ns | 88.1ns | 45 of 45 |
| Gather | 29.600ms | 0.3% | 67 | 4008970 | 0 | 7.4ns | handed on none | 67 of 67 |
| Filter | 9.195ms | 0.1% | 10 | 1727441 | 151305 | 5.3ns | 60.8ns | 10 of 10 |
| Sort | 3.369ms | 0.0% | 13 | 19795 | 19795 | 170.2ns | 170.2ns | 13 of 13 |
| TopN | 2.434ms | 0.0% | 5 | 50515 | 287 | 48.2ns | 8481.8ns | 5 of 5 |
| CteScan | 42.269us | 0.0% | 4 | 0 | 79636 | handed none | 0.5ns | 4 of 4 |
| MaterializedCTE | 27.542us | 0.0% | 2 | 39818 | 0 | 0.7ns | handed on none | 2 of 2 |
| Broadcast | 13.451us | 0.0% | 3 | 81833 | 81833 | 0.2ns | 0.2ns | 3 of 3 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is Scan, and the queries where it cost the most are q09 at 749.955ms, q20 at 666.833ms, q12 at 616.467ms.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven

Hot here means the tries after the first, with the page cache and any server caches left as the first try left them. The server was restarted before every query's first try, so nothing carries from one query to the next.
