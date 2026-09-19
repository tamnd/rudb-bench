# What the TPC-H gap is made of

[The first TPC-H with rudb in it](the-first-tpch-with-rudb-in-it.md) put rudb at 6.91x DuckDB over the sixteen queries it finished, and ended by saying the number was two problems added together without saying how much of it was each: some of the gap is arithmetic per row and some of it is that only half the machine is working. That is the sentence this report replaces with numbers.

Separating them takes one measurement per query, which is CPU seconds against wall seconds for both engines on the same query on the same machine. CPU over wall is how many cores the engine actually kept busy. Then the wall clock ratio is exactly the product of two independent ratios: how much more work rudb did, and how much less of the machine it did it on. Four queries, SF100, second run of each so the page cache is warm.

| query | shape | rudb wall | duckdb wall | slower by | rudb cores | duckdb cores | more CPU work | less parallel |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q06 | scan, filter, one sum | 930ms | 840ms | 1.11x | 14.49 | 20.02 | 0.80x | 1.38x |
| q17 | correlated scalar subquery | 16.340s | 2.190s | 7.46x | 10.35 | 23.45 | 3.29x | 2.27x |
| q04 | EXISTS semi join | 16.490s | 1.280s | 12.88x | 5.98 | 19.37 | 3.97x | 3.24x |
| q16 | count distinct over a join, with a NOT IN | 20.270s | 1.370s | 14.80x | 1.18 | 6.91 | 2.52x | 5.86x |

The last two columns multiply to the third. 0.80 x 1.38 = 1.11, 3.29 x 2.27 = 7.46, 3.97 x 3.24 = 12.87, 2.52 x 5.86 = 14.77. Nothing is being fitted here, that is just what the identity does when both halves are measured rather than estimated.

## The scan is not the problem, and was never going to be

q06 is the whole suite's control. It is a scan of `lineitem`, three predicates and one sum, with no join in it anywhere. rudb is 1.11x DuckDB on it, which is the interesting part only until you look at the column next door: **rudb did the query in 13.48 CPU seconds against DuckDB's 16.82.** It is not narrowly losing on q06, it is doing twenty percent less work and then handing back the win by using fourteen and a half cores where DuckDB used twenty.

The memory says the same thing louder. q06 peaked at 139 MB resident in rudb and 759 MB in DuckDB.

So the part of an engine that ClickBench measures, which is scanning a column and aggregating it, is already competitive here and by the CPU measure is ahead. Everything else in this report is about the part ClickBench does not measure.

## Parallelism falls off as the query becomes join shaped

Read the rudb cores column downward: 14.49, 10.35, 5.98, 1.18. That is not four numbers, it is a slope. The more of the query that is join and subquery rather than scan, the less of the machine rudb uses, and at the bottom of the table **q16 runs on 1.18 cores, which is one thread and a rounding error.** DuckDB on the same query keeps 6.91 busy, and DuckDB is not even trying hard there, since it manages 19 to 23 on the other three.

q16's 29.8x in the run report is therefore mostly not an algorithmic gap. It is 2.52x more work, which is real but ordinary, multiplied by running that work on one core. Fix nothing about the work and parallelise it as well as rudb already parallelises q06, and q16 alone goes from 20.27s to about 3.5s.

This is worth stating plainly because the operator breakdown in the run report points somewhere else. It says Aggregate is 35.0% of all CPU and the biggest single kind, and it is right, but CPU share cannot see a thread that is not running. An operator that is serial costs wall clock without costing CPU, so it is invisible in exactly the table you would go to first.

## The work ratio is the join path, and it costs memory too

The other factor is 2.5x to 4x more CPU across the three join queries, against 0.80x on the one without a join. That is the join and subquery path being between three and five times more expensive per row than DuckDB's, which is a normal distance for an engine that has one implementation against an engine with several and a cost model to choose between them.

Resident memory on the same runs:

| query | rudb | duckdb | ratio |
| --- | --- | --- | --- |
| q06 | 139.47 MB | 759.10 MB | 0.18x |
| q17 | 5.99 GB | 1.38 GB | 4.34x |
| q04 | 17.19 GB | 1.08 GB | 15.93x |
| q16 | 17.28 GB | 2.32 GB | 7.45x |

The same split, and harder. rudb uses a fifth of DuckDB's memory on the query with no join and sixteen times as much on the EXISTS. That is the same fact as the four OOMs in the run report, seen from the other side: q04 finished, in 17.19 GB, and q07, q09, q18 and q21 are the queries where the same behaviour crossed the 25.0 GiB limit and stopped. They did not fail because they are different in kind from q04. They failed because they are bigger.

## What this says about the goal

The target is ten times better than DuckDB. This measurement does not get there and is worth being exact about why not.

The two factors are independent and both are fixable, and if both were fixed completely, rudb would be at parity on these four queries and not ahead of them. Parallelising the join path is worth up to 5.9x on the worst query and about 3x on average, which is engineering with a known answer. Closing the 3 to 4x on CPU in the join path is harder and is a matter of having more than one join implementation. Together they are worth roughly the 6.91x, which lands rudb level with DuckDB on TPC-H rather than ten times ahead.

**So there is no version of fixing these two things that reaches the goal on this suite.** Ten times DuckDB over the shared sixteen is 1.739s against rudb's current 120.226s, and the gap between "level with DuckDB" and "ten times DuckDB" is the whole of it. On ClickBench the answer to that question exists and is already working, which is the certified frequency synopsis answering ten queries out of metadata in 0.172s against DuckDB's 3.307s, and that is a 19x on those ten. There is no equivalent for a join yet. Whatever closes TPC-H to the goal is a thing of that kind rather than a faster hash table, and this report cannot say what it is.

What it can say is the order to do the work in, because the two factors are not equally cheap. Parallelism first: it is worth ~3x on average, it is the entire story on the worst query in the suite, and it is invisible in the profiling output everyone would otherwise trust.

## The two decimal failures are one bug

Separately, the run report lists q01 and q14 as two arithmetic defects. They are one, and it is a type rule rather than an arithmetic bug.

```
SELECT typeof(sum(l_extendedprice * (1 - l_discount))) FROM lineitem
rudb    DECIMAL(18,4)
duckdb  DECIMAL(38,4)
```

`sum` over a decimal in rudb returns the input precision. In DuckDB it widens to 38. Everything else follows from that one line:

- q14 computes `100.00 * sum(...)`, which in rudb is a multiply at DECIMAL(18) and overflows. The error rudb gives is `Overflow in multiplication of DECIMAL(18) (10000 * 452593436477868)`, and DuckDB gives the byte for byte identical error on the same expression when you force its operand to DECIMAL(18). So rudb's multiply is correct and its `sum` handed it the wrong type.
- q01 sums `l_extendedprice * (1 - l_discount) * (1 + l_tax)`, which has scale 6. At SF1 that total is 226,829,357,828.867781, an unscaled 2.27e17, which fits an i64. At SF100 it is a hundred times larger, 2.27e19, and i64 stops at 9.22e18. rudb reports a partial at `11016932248183655467`, which is 1.10e19, one group on its way past the end of the type.

Neither is about this machine and neither is about scale factor 100 in particular. Both are what happens at whatever size makes the numbers large enough, and TPC-H is designed to make them large enough. Widening `sum` over a decimal to 38 is expected to take TPC-H from 16 of 22 to 18 of 22 with no other change.

## How to reproduce it

`~/bench/cores.sh` on `gamingpc-wsl` takes a query and a label and runs it twice on each engine under `/usr/bin/time`, with views over `~/rudb-data/tpch100`. `TMPDIR` and `RUDB_BENCH_SCRATCH` have to be off `/tmp`, which is a 16 GiB tmpfs on this machine and smaller than the data.

```
./cores.sh "<sql>" q17
```

The cores figure is `(cpu_user + cpu_sys) / wall` from that output, and the numbers above are the second run of each pair.
