# The first TPC-H with rudb in the table

Every TPC-H run this project has published has had four engines in it and a sentence saying why the fifth was not there. The sentence was that every join in rudb is a nested loop, that TPC-H is twenty two of them, and that timing it would be a timing of a hang. That sentence was written down in [`src/engine.rs`](../../src/engine.rs), pinned by a test, repeated in three other files, and never measured.

It is false. rudb 0.3.57 answers all twenty two at SF1 in under a second each, and joining SF1's 6 million row `lineitem` to its 1.5 million row `orders` returns all 6,001,215 matching rows in 0.204s, which nine trillion pair comparisons do not do. So the refusal is gone and rudb is in the table. The run is [run-tpch-gamingpc-wsl-with-rudb.md](run-tpch-gamingpc-wsl-with-rudb.md) and the earlier four engine run on the same day and machine is still [where it was](run-tpch-gamingpc-wsl.md).

This is not a good result. It is a real one, which is the thing the refusal was preventing.

## Sixteen of twenty two

| engine | version | of 22 | query time on the shared 16 | vs duckdb | cores kept busy |
| --- | --- | --- | --- | --- | --- |
| duckdb | v1.5.5 d8cdaa33fd | 22 | 17.392s | 1.00x | 16.82 |
| duckdb-pinned | v2.0.0-dev84237 cc7e7bac7f | 22 | 16.248s | 0.93x | 12.93 |
| datafusion | datafusion-cli 55.1.0 | 22 | 42.314s | 2.43x | 18.62 |
| clickhouse-local | 26.9.1.1562 | 22 | 66.842s | 3.84x | 17.20 |
| rudb | 0.3.57 | 16 | 120.226s | 6.91x | 8.14 |

SF100, 33.20 GiB of Parquet in eight tables, five hot runs of each query after one cold one, on `gamingpc-wsl`. The totals are over the sixteen queries rudb answered, so every column is the same sixteen and none of them is the whole suite.

Two of the five engines read the Parquet in place and three convert it into a format of their own first. rudb is in the first group here, with DataFusion, and that is the comparison to make: against the other engine paying a decode inside every query, rudb is 2.84x. The three that converted paid for it once, at load, and DuckDB's load was 49.879s.

## The six it did not answer

| query | what rudb said |
| --- | --- |
| q01 | INTERNAL Error: an unscaled decimal of 11016932248183655467 does not fit the run its precision chose |
| q07 | Out of Memory Error: could not allocate 19.0 MiB (25.0 GiB/25.0 GiB used) |
| q09 | Out of Memory Error: could not allocate 3.0 GiB (22.9 GiB/25.0 GiB used) |
| q14 | Out of Range Error: Overflow in multiplication of DECIMAL(18) (10000 * 452593436477868) |
| q18 | Out of Memory Error: could not allocate 288.0 MiB (25.0 GiB/25.0 GiB used) |
| q21 | Out of Memory Error: could not allocate 4.5 GiB (22.5 GiB/25.0 GiB used) |

These are two different problems and only one of them is about this machine.

q01 and q14 are arithmetic. q01 is `sum(l_extendedprice * (1 - l_discount))` over 600 million rows and the accumulator does not hold the answer; q14 is `100.00 * sum(...)` and the multiplication overflows a `DECIMAL(18)` before the division that would bring it back. Both are the same class of defect, a decimal whose precision was chosen for the inputs and not for what the aggregate does to them, and neither is about the size of the box. They will happen at any scale factor where the numbers get large enough, and TPC-H is designed so that they do.

The other four are memory: rudb ran out against its own 25.0 GiB limit, on a box with 31 GiB and 32 threads. It is worth being careful about what that does and does not show.

These same four queries are the four most expensive in the suite for DataFusion as well, which is the other engine here reading the Parquet in place. Its peaks were q18 at 28.26 GiB, q21 at 21.44 GiB, q09 at 14.44 GiB and q07 at 11.22 GiB, and those are its four largest by a wide margin, the fifth being q04 at 6.60 GiB. DataFusion finished them because nothing stopped it going past 25 GiB, not because it was thrifty. The three engines that converted the data into a format of their own first did stay small: DuckDB peaked at 16.64 GiB over the whole suite and clickhouse-local at 17.03 GiB.

So the four OOMs are mostly the cost of reading Parquet in place, shared with the other engine that does it, and rudb's limit is what turned that cost into a failure rather than a slow query. What is rudb's own is that it stopped: q09 asked for 3.0 GiB in one go and q21 for 4.5 GiB, single allocations rather than gradual growth, which is the shape a hash build side has when it is not spilling. Nothing here is a nested loop, which is what the old refusal claimed. It is a hash join that holds everything in memory and has nowhere to put it when it will not fit.

## The sixteen it did answer, it answered correctly

The report says it in one line and it is the most important line in it: all five engines agreed on every answer the data settles, which is twenty two of twenty two queries, to the last significant digit of a double. rudb's sixteen are right. The engine that is six times slower is not six times slower because it is cutting a corner.

## Where the time goes

rudb reports its own breakdown by kind of operator, over the whole run:

| kind | cpu | share | per row |
| --- | --- | --- | --- |
| Aggregate | 319.013s | 35.0% | 211.4ns |
| FileScan | 306.044s | 33.6% | 50.0ns |
| Probe | 160.763s | 17.7% | 259.8ns |
| Filter | 94.049s | 10.3% | 18.2ns |
| Join | 18.469s | 2.0% | 918.6ns |

Two thirds of the run is aggregation and reading the file, and the aggregate is the larger half at 211 nanoseconds a row. For comparison the filter costs 18.2ns a row, so the aggregate is eleven times the cost of the cheapest thing in the pipeline and it is where a third of the suite goes. It concentrates further than that: q17 spent 132.129s in Aggregate on its own, q04 53.423s and q20 41.112s, which is three queries holding most of the largest line in the table, and all three are in the worst six below.

The cores column in the table above is the other half of the story and it is arguably the bigger one. rudb kept 8.14 of this machine's 32 threads busy. DuckDB kept 16.82, DataFusion 18.62, clickhouse-local 17.20. rudb is using about half the parallelism the other engines are on the same queries on the same machine, so some of the 6.91x is arithmetic per row and some of it is that only half the machine is working. Those are separate problems with separate fixes and this run does not separate them.

## The worst six and the best three

| query | shape | rudb | duckdb | ratio |
| --- | --- | --- | --- | --- |
| q16 | parts supplier relationship | 16.361s | 549.000ms | 29.8x |
| q22 | global sales opportunity | 7.704s | 363.000ms | 21.2x |
| q04 | order priority checking | 16.213s | 979.000ms | 16.6x |
| q17 | small quantity order revenue | 16.165s | 1.109s | 14.6x |
| q02 | minimum cost supplier | 2.349s | 251.000ms | 9.4x |
| q20 | potential part promotion | 10.269s | 1.185s | 8.7x |
| ... | | | | |
| q12 | shipping modes and order priority | 3.627s | 1.594s | 2.28x |
| q10 | returned item reporting | 6.117s | 2.289s | 2.67x |
| q06 | forecasting revenue change | 886.216ms | 744.000ms | 1.19x |

q06 is the one query in the suite that is a scan, a filter and one sum with no join at all, and rudb is within 19% of DuckDB on it. That is the floor this engine is already at when nothing else is happening. Everything above it is what joins, group bys and correlated subqueries cost on top, and the four at 14x and above are all queries with a subquery or a semi join in them.

## What this is and is not

It is a first measurement, on one machine, at one scale factor, with six queries missing. It is not comparable to anybody's published TPC-H and it is not a rule three column, because rule three asks for the whole suite and this is sixteen of twenty two. The harness says so itself: the run is not publishable, and the report lists that reason along with three queries that swung wider than rule two allows and the fact that this is not a c6a.4xlarge.

It is also nowhere near the goal. The standing target is ten times better than DuckDB and this is seven times worse, on a subset that excludes the four queries rudb cannot fit in memory. The gap on ClickBench is a different story and [today's other report](twelve-patches-later.md) tells it, but nothing there transfers to here: the certified synopsis that wins ten ClickBench queries has no equivalent in a suite that is mostly joins.

What changed today is that the number exists. It was previously unavailable by construction.

## How to reproduce it

The command the report documents is the whole sweep:

```
rudb-bench run tpch --engines duckdb,duckdb-pinned,clickhouse-local,datafusion,rudb --runs 5 --report
```

What was actually run was one engine at a time, `--save` each and `rudb-bench report tpch` at the end, so that a failure part way through did not throw away the engines already measured. The two produce the same table.

`TMPDIR` and `RUDB_BENCH_SCRATCH` have to be on the disk. `/tmp` on this machine is a 16 GiB tmpfs, SF100 is 33 GiB, and rudb spills its dictionary offsets there: left at the default it fills the tmpfs, takes the memory with it, and every query after the first reports "No space left on device" as though it were an engine defect. That is not one of the six above, and it is worth writing down because it looked exactly like one for half an hour.
