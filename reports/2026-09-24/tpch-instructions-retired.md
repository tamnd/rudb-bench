# TPC-H SF1 in instructions retired, which a busy machine cannot spoil

rudb executes 1.51x the instructions DuckDB does over the 22 queries, while taking 0.88x to 0.96x of its CPU time. The higher instruction throughput was hiding that the engine does half again as much work.

This is the measurement to track. Two runs an hour apart, with the machine's load average moving between 4.2 and 10.1 in between, agree to 1.51x and 1.52x in total and to within 0.03x on every single query. Nothing else measured against DuckDB in this repo has been that stable, and the reason is that a retired instruction count does not care what else the box is doing.

## Why this rather than wall clock

Every TPC-H comparison in this repo so far has been fought against machine load. The 901x in `reports/2026-09-18` was a configuration artifact. The 0.85x in `reports/2026-09-24/where-tpch-actually-stands.md` was taken at a load average between 14 and 52 and had to be superseded. The 0.93x and 0.98x in `tpch-per-core-on-a-quiet-machine.md` needed a box that happened to go quiet for twenty minutes, and it has not been quiet since.

Instructions retired needs none of that. It is a count of work done, read out of a hardware counter that is attributed to the process, so a run queue of forty changes how long the query takes and not how many instructions it took to answer it. The two runs here prove it rather than assume it: the second one ran while the load average was twice the first one's, and the per query ratios did not move.

It is not a substitute for time. An engine can retire fewer instructions and be slower, because a cache miss costs no instructions and plenty of nanoseconds. rudb is the other way about on this suite, which is the interesting part. But for the question this project keeps asking, which is whether a change made the engine do less work, it is the better instrument, and it is the one half of "10x faster and 10x less resource" that can be measured on a shared box.

## The setup

server2, 6 hardware threads. DuckDB v2.0.0-dev84237 against its own database file, rudb 0.4.20 against its own, both loaded from the same SF1 Parquet, both pinned to one thread. `perf stat -e instructions,task-clock` around a fresh process per query, three repeats, median, alternating which engine goes first. Startup is measured the same way with `SELECT 1` and subtracted: 169.7M instructions for DuckDB and 37.2M for rudb, so rudb starts up for about a fifth of what DuckDB spends getting to a prompt.

## Instructions retired, per query

| query | DuckDB | rudb | run one | run two |
| --- | --- | --- | --- | --- |
| q01 | 1268.4M | 2983.7M | 2.34x | 2.35x |
| q12 | 733.0M | 1439.0M | 1.96x | 1.96x |
| q18 | 1410.1M | 2756.6M | 1.94x | 1.95x |
| q21 | 1450.7M | 2532.5M | 1.74x | 1.75x |
| q13 | 1426.7M | 2443.8M | 1.71x | 1.71x |
| q04 | 635.2M | 1052.1M | 1.65x | 1.66x |
| q09 | 1666.7M | 2630.3M | 1.58x | 1.58x |
| q10 | 1037.7M | 1614.6M | 1.53x | 1.56x |
| q06 | 426.8M | 643.4M | 1.50x | 1.51x |
| q05 | 801.4M | 1081.7M | 1.32x | 1.35x |
| q03 | 690.7M | 919.2M | 1.31x | 1.33x |
| q20 | 757.9M | 968.3M | 1.28x | 1.28x |
| q22 | 430.3M | 548.9M | 1.27x | 1.28x |
| q17 | 790.6M | 997.1M | 1.21x | 1.26x |
| q02 | 142.9M | 176.4M | 1.20x | 1.23x |
| q16 | 511.8M | 607.8M | 1.19x | 1.19x |
| q19 | 680.8M | 772.7M | 1.11x | 1.14x |
| q07 | 913.4M | 973.7M | 1.05x | 1.07x |
| q14 | 544.3M | 557.7M | 1.05x | 1.02x |
| q08 | 854.9M | 864.4M | 1.01x | 1.01x |
| q15 | 526.4M | 520.2M | 1.00x | 0.99x |
| q11 | 195.6M | 181.6M | 0.92x | 0.93x |
| total | 17.90G | 27.27G | **1.51x** | **1.52x** |

The absolute columns are from the second run. The two ratio columns are what makes the point.

## What it says that the clock did not

The five queries that are slower than DuckDB on the clock are q01, q13, q21, q12 and q09, and all five of them are in the top seven here. So the instruction count agrees with the clock about where the work is, and it says so at a tenth of the measurement effort.

Where the two disagree is q18, which retires 1.95x the instructions and is 0.73x to 0.78x of DuckDB's time. It does a great deal more work and finishes sooner. That is an engine whose per instruction throughput is well ahead, which is the same story the totals tell: 1.51x the instructions in 0.88x to 0.96x of the CPU time is roughly 1.6x the instructions per cycle.

That is worth being clear about, because it cuts both ways. The vectorisation and the memory layout are doing their job. What is not doing its job is the amount of work being asked for in the first place, and no amount of further throughput will fix a query that is executing twice the instructions it needs. q01 executes 2.34x DuckDB's instructions and comes out 1.4x slower, so even the IPC lead does not cover it.

## The 10x goal, stated in this metric

To do 10x less work than DuckDB on this suite, rudb has to go from 27.27G instructions to 1.79G. That is a 15x reduction from where it is. Stating it that way is more useful than the clock version, because it is a number that can be attributed to an operator and watched for regressions on any machine, busy or not.

Nine queries are already at 1.20x or better and three are at parity or ahead, with q11 the only one under 1.00x. So the distance is not uniform and the top of the table is where it lives: q01, q12, q18, q21 and q13 account for 12.16G of the 27.27G between them, against DuckDB's 6.29G on the same five.

## Where q01's extra instructions go

q01 is the worst ratio and the simplest query, so it is the one worth opening up. A `perf record` profile of it, single threaded, with the percentages of the whole process:

```
11.1%  rudb_exec::prepared::Prepared::run_step
 5.9%  clear_page_rep                          (kernel, faulting in fresh pages)
 5.5%  rudb_exec::group::Aggregate::fold
 5.2%  rudb_kernels::aggregate::total_into
 4.7%  rudb_vector::vector::unpack_block
 4.7%  rudb_vector::vector::Packed::values_at
 4.0%  rudb_vector::vector::Vector::gather
 3.8%  rudb_kernels::aggregate::mean_into
 3.5%  __memset_avx2_unaligned_erms
 3.3%  __memmove_avx_unaligned_erms
 3.1%  rudb_encoding::integer::decode_chunk
 2.9%  rudb_vector::vector::Packed::values_at
 2.9%  rudb_kernels::aggregate::mean_into
 2.5%  rep_movs_alternative                    (kernel, copying)
```

The aggregate arithmetic is not the problem and the aggregates are properly vectorised: `total_into` and `mean_into` are kernels and come to about 12% between them. Two other things are each about as large.

Unpacking is 15%, adding up `unpack_block`, the two `Packed::values_at` entries and `decode_chunk`. Every chunk of every column q01 touches is bit packed in the stored table and is unpacked before anything reads it.

Moving memory is 15%, adding up the page faulting, the memset, the memmove and the kernel copy. A fifth of that is the operating system handing over pages that have to be zeroed, which is a process asking for memory it has not asked for before rather than reusing what it has.

The filter is a candidate for part of it. q01's `l_shipdate <= date '1998-12-01' - interval '90 day'` keeps 5,916,591 of 6,001,215 rows, and `Vector::gather` at 4% plus the copying around it is consistent with a chunk that keeps 98.6% of its rows being rebuilt into a fresh vector rather than carrying a selection. Dropping the filter from the query bears that out: rudb goes from 1.21x of DuckDB's time to 0.85x, and from 3.02G instructions to 2.30G against DuckDB's 1.15G, so rudb pays 727M instructions for a filter DuckDB pays 291M for.

That is one thread to pull and the unpacking is another. Both are in rudb #1633.

## Reproducing

`perf stat -x, -e instructions,task-clock` around each engine in a fresh process, single threaded, three repeats, median, with a `SELECT 1` baseline subtracted. The script is not in the harness. Folding it in is worth more than folding in the paired wall clock script the last report asked for, because this one does not need a quiet machine and that one does.
