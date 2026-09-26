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
| duckdb | 23 | 0.65 | 1.78 | 1.81 | 60.001s | 22 |

The reading includes the tail of whatever ran just before, which is usually the previous engine's turn, because the one minute average takes about a minute to decay.

rudb writes summaries of each column when it loads a table, and it can answer some queries from those without reading the rows. This run turned that off with `SET stored_answers = false` before every rudb query (`RUDB_BENCH_STORED_ANSWERS=off`), so rudb read the rows the way the other engines did. Its metrics said it still answered from stored summaries for no query. The per query table marks any such query, and it also marks q1 to q7, whose shapes (counts, sums, averages, distinct counts and bounds over the whole table) are the ones a summary can answer. The headline below gives the ratios both with and without q1 to q7.

## Headline

| engine | hot total | cold total | hot geomean | total vs duckdb | geomean vs duckdb | total vs, without q1 to q7 | geomean vs, without q1 to q7 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 2.684s | 3.564s | 0.1064s | 1.000x | 1.000x | 1.000x | 1.000x |

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
| duckdb | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 7.199s | 92.690s | 2.62 GiB | its own database file | its own | 0.65 to 3.38 |
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
| duckdb | 2.684s | 3.665s | +37% | 5.695s | 28.930s | 7.89 | 1.58 GiB | 512.00 KiB | 709.94M/s | 26.28 GiB/s | 1.00x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 22 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | rudb from stored summaries |
| --- | --- | --- | --- |
| q01 | pricing summary report | 117.000ms |  |
| q02 | minimum cost supplier | 34.000ms |  |
| q03 | shipping priority | 104.000ms |  |
| q04 | order priority checking | 113.000ms |  |
| q05 | local supplier volume | 123.000ms |  |
| q06 | forecasting revenue change | 69.000ms |  |
| q07 | volume shipping | 136.000ms |  |
| q08 | national market share | 131.000ms |  |
| q09 | product type profit measure | 298.000ms |  |
| q10 | returned item reporting | 194.000ms |  |
| q11 | important stock identification | 28.000ms |  |
| q12 | shipping modes and order priority | 124.000ms |  |
| q13 | customer distribution | 197.000ms |  |
| q14 | promotion effect | 101.000ms |  |
| q15 | top supplier | 105.000ms |  |
| q16 | parts supplier relationship | 76.000ms |  |
| q17 | small quantity order revenue | 100.000ms |  |
| q18 | large volume customer | 207.000ms |  |
| q19 | discounted revenue | 109.000ms |  |
| q20 | potential part promotion | 103.000ms |  |
| q21 | suppliers who kept orders waiting | 171.000ms |  |
| q22 | global sales opportunity | 44.000ms |  |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 117.000ms | 245.821ms | 160.950ms | 0.3% | 160.950ms | 161.479ms | 160.950ms | 161.479ms | 1.150s | 633.03 MiB | 678.13 MiB | 538.13M/s |
| q02 | minimum cost supplier | 34.000ms | 123.150ms | 80.641ms | 0.1% | 80.641ms | 80.696ms | 80.641ms | 80.696ms | 150.000ms | 147.87 MiB | 131.80 MiB | 1.07G/s |
| q03 | shipping priority | 104.000ms | 284.063ms | 141.094ms | 13.4% | 141.094ms | 162.880ms | 141.094ms | 162.880ms | 1.080s | 801.98 MiB | 713.10 MiB | 613.86M/s |
| q04 | order priority checking | 113.000ms | 225.546ms | 161.123ms | 1.2% | 161.123ms | 163.092ms | 161.123ms | 163.092ms | 800.000ms | 530.80 MiB | 588.24 MiB | 537.55M/s |
| q05 | local supplier volume | 123.000ms | 263.869ms | 162.363ms | 21.5% | 162.363ms | 201.375ms | 162.363ms | 201.375ms | 1.100s | 800.16 MiB | 803.75 MiB | 533.45M/s |
| q06 | forecasting revenue change | 69.000ms | 203.500ms | 120.920ms | 1.9% | 120.920ms | 123.192ms | 120.920ms | 123.192ms | 610.000ms | 603.82 MiB | 750.07 MiB | 716.28M/s |
| q07 | volume shipping | 136.000ms | 264.848ms | 181.225ms | 0.5% | 181.225ms | 182.077ms | 181.225ms | 182.077ms | 1.220s | 1000.84 MiB | 883.44 MiB | 477.93M/s |
| q08 | national market share | 131.000ms | 304.128ms | 181.457ms | 67.4% | 181.457ms | 304.207ms | 181.457ms | 304.207ms | 1.210s | 1021.51 MiB | 1.11 GiB | 477.32M/s |
| q09 | product type profit measure | 298.000ms | 488.778ms | 353.893ms | 4.0% | 353.893ms | 368.505ms | 353.893ms | 368.505ms | 4.130s | 1.58 GiB | 1.32 GiB | 244.74M/s |
| q10 | returned item reporting | 194.000ms | 344.837ms | 243.058ms | 0.7% | 243.058ms | 244.688ms | 243.058ms | 244.688ms | 2.050s | 1.14 GiB | 825.74 MiB | 356.34M/s |
| q11 | important stock identification | 28.000ms | 123.280ms | 60.474ms | 0.1% | 60.474ms | 60.530ms | 60.474ms | 60.530ms | 180.000ms | 150.69 MiB | 134.46 MiB | 1.43G/s |
| q12 | shipping modes and order priority | 124.000ms | 243.528ms | 161.085ms | 11.6% | 161.085ms | 182.166ms | 161.085ms | 182.166ms | 990.000ms | 654.60 MiB | 750.10 MiB | 537.68M/s |
| q13 | customer distribution | 197.000ms | 327.605ms | 243.198ms | 1.6% | 243.198ms | 247.127ms | 243.198ms | 247.127ms | 2.670s | 887.33 MiB | 475.36 MiB | 356.14M/s |
| q14 | promotion effect | 101.000ms | 244.933ms | 140.931ms | 12.7% | 140.931ms | 161.498ms | 140.931ms | 161.498ms | 1.040s | 856.15 MiB | 725.70 MiB | 614.57M/s |
| q15 | top supplier | 105.000ms | 224.020ms | 141.159ms | 12.3% | 141.159ms | 160.988ms | 141.159ms | 160.988ms | 870.000ms | 777.63 MiB | 694.56 MiB | 613.58M/s |
| q16 | parts supplier relationship | 76.000ms | 165.735ms | 122.527ms | 2.9% | 122.527ms | 126.130ms | 122.527ms | 126.130ms | 610.000ms | 342.53 MiB | 103.81 MiB | 706.88M/s |
| q17 | small quantity order revenue | 100.000ms | 225.543ms | 141.286ms | 16.2% | 141.286ms | 164.191ms | 141.286ms | 164.191ms | 1.050s | 672.07 MiB | 690.48 MiB | 613.03M/s |
| q18 | large volume customer | 207.000ms | 348.213ms | 263.427ms | 0.6% | 263.427ms | 264.914ms | 263.427ms | 264.914ms | 3.180s | 1.14 GiB | 501.30 MiB | 328.79M/s |
| q19 | discounted revenue | 109.000ms | 303.792ms | 160.981ms | 0.7% | 160.981ms | 162.088ms | 160.981ms | 162.088ms | 1.260s | 942.74 MiB | 1.19 GiB | 538.03M/s |
| q20 | potential part promotion | 103.000ms | 264.553ms | 141.191ms | 13.0% | 141.191ms | 162.296ms | 141.191ms | 162.296ms | 1.110s | 846.33 MiB | 881.67 MiB | 613.44M/s |
| q21 | suppliers who kept orders waiting | 171.000ms | 331.935ms | 221.391ms | 2.9% | 221.391ms | 227.901ms | 221.391ms | 227.901ms | 1.990s | 845.43 MiB | 797.50 MiB | 391.22M/s |
| q22 | global sales opportunity | 44.000ms | 143.524ms | 80.853ms | 0.1% | 80.853ms | 80.917ms | 80.853ms | 80.917ms | 480.000ms | 231.40 MiB | 144.28 MiB | 1.07G/s |

duckdb v2.0.0-dev84237 (Development Version) cc7e7bac7f over 22 of 22 queries. Total 2.684s by its own clock and 3.665s by ours, 5.695s cold, 28.930s of CPU, peak 1.58 GiB, 709.94M/s and 26.28 GiB/s.

Running it cost 37% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 6.04x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- duckdb is a development build, v2.0.0-dev84237 (Development Version) cc7e7bac7f, and not a DuckDB release

Hot here means the tries after the first, with the page cache and any server caches left as the first try left them. The server was restarted before every query's first try, so nothing carries from one query to the next.
