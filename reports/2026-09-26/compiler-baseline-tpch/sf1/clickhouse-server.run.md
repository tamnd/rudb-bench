# tpch on gamingpc-wsl

This is one run of the tpch suite on gamingpc-wsl, over 0 engines and 0 queries, with 4 tries of each query after a page cache drop, the way the upstream ClickBench driver runs it. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

## What ran

| what | it was |
| --- | --- |
| suite | tpch |
| queries | 22 |
| tables | 310.61 MiB of Parquet in 8 tables |
| rows | 8661245 in the table every query reads |
| corpus | duckdb-tpch SF1, corpus 5739c03f28043b03, written 2026-09-26 |
| summary | nothing ran |
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

rudb writes summaries of each column when it loads a table, and it can answer some queries from those without reading the rows. This run turned that off with `SET stored_answers = false` before every rudb query (`RUDB_BENCH_STORED_ANSWERS=off`), so rudb read the rows the way the other engines did. Its metrics said it still answered from stored summaries for no query. The per query table marks any such query, and it also marks q1 to q7, whose shapes (counts, sums, averages, distinct counts and bounds over the whole table) are the ones a summary can answer. The headline below gives the ratios both with and without q1 to q7.

## Headline

No engine produced a number.

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
RUDB_BENCH_STORED_ANSWERS=off rudb-bench run tpch --engines  --runs 4 --protocol upstream --timeout 600 --report
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
| clickhouse-server | 26.9.1.1138 | clickhouse client loading a parquet file failed: Error on processing query: Code: 108. DB::Exception: No data to insert. (NO_DATA_TO_INSERT) (version 26.9.1.1138 (official build)) (query: INSERT INTO lineitem FORMAT Parquet) | n/a | n/a | n/a | n/a | n/a | n/a |
| duckdb | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| duckdb-pinned | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| clickhouse-local | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| datafusion | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| polars | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| rudb | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs nothing |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 0 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

No engine produced a number.

## What this number is not

Nothing here blocks publication. Reporting rule one still applies: state the machine, the kernel, the filesystem and the settings next to the number.

Hot here means the tries after the first, with the page cache and any server caches left as the first try left them. The server was restarted before every query's first try, so nothing carries from one query to the next.
