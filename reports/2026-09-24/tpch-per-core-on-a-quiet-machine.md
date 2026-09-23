# TPC-H SF1 per core on a quiet machine

The measurement `reports/2026-09-24/where-tpch-actually-stands.md` said it could not make: the same paired protocol run on a box that was not busy. Two independent runs put rudb at 0.93x and 0.98x of DuckDB per core over the 22 queries, and the five queries that are slower are the same five in both runs.

## Why the earlier run has to be thrown out

The 0.85x table in the earlier report was taken on server3 while its one minute load average sat between 14 and 52 on 8 hardware threads. The report said the absolute times were inflated and that only the ratios survived. That was too generous. Running the same script on server2 at a load average under 5 on 6 threads, the same queries come back roughly 10x faster, and the per query ratios move by enough to change what the work list says.

Concretely, q12 on server3 came out at 2.25x and was written up as the worst query in TPC-H by a wide margin. On a quiet box it is 1.25x and 1.30x, and it is the fourth worst. Repeated runs of q12 alone on server3 varied between 0.21s and 0.70s with nothing changing, so the 2.25x was a draw from that spread and not a property of the engine. Pairing and alternating protects a ratio against load that drifts slowly. It does not protect it against a box where a single query's time varies by 3x run to run, because then the medians are drawn from a distribution wide enough to swamp the difference being measured.

## The setup

server2, 6 hardware threads, load average between 0.98 and 5.03 across both runs. DuckDB v2.0.0-dev84237 against its own database file, rudb 0.4.20 against its own. Both loaded from the same SF1 Parquet, both pinned to one thread, seven rounds, alternating which engine runs first each round, median per query, each engine's own reported run time. rudb's file is 250.7 MiB and DuckDB's is 266.0 MiB.

One thread rather than six because that is what removes the load sensitivity. An engine that wants every thread at once loses more to a busy run queue than one that does not, so a multi threaded comparison on a shared box measures scheduling as much as it measures the engines. What a single threaded ratio answers is how much work each engine does per row, which is the thing the 10x goal is about.

## Both runs

| query | DuckDB | rudb | ratio | DuckDB | rudb | ratio |
| --- | --- | --- | --- | --- | --- | --- |
| q01 | 353.0ms | 473.1ms | 1.34x | 304.0ms | 432.2ms | 1.42x |
| q02 | 56.0ms | 48.1ms | 0.86x | 49.0ms | 41.5ms | 0.85x |
| q03 | 282.0ms | 244.9ms | 0.87x | 226.0ms | 237.2ms | 1.05x |
| q04 | 235.0ms | 210.2ms | 0.89x | 180.0ms | 193.4ms | 1.07x |
| q05 | 331.0ms | 289.2ms | 0.87x | 252.0ms | 260.4ms | 1.03x |
| q06 | 166.0ms | 146.7ms | 0.88x | 141.0ms | 124.8ms | 0.89x |
| q07 | 315.0ms | 224.3ms | 0.71x | 289.0ms | 215.4ms | 0.75x |
| q08 | 308.0ms | 206.8ms | 0.67x | 259.0ms | 183.8ms | 0.71x |
| q09 | 831.0ms | 909.7ms | 1.09x | 726.0ms | 782.1ms | 1.08x |
| q10 | 426.0ms | 391.2ms | 0.92x | 374.0ms | 368.0ms | 0.98x |
| q11 | 70.0ms | 42.5ms | 0.61x | 63.0ms | 39.8ms | 0.63x |
| q12 | 224.0ms | 279.9ms | 1.25x | 218.0ms | 283.1ms | 1.30x |
| q13 | 468.0ms | 617.2ms | 1.32x | 479.0ms | 661.2ms | 1.38x |
| q14 | 226.0ms | 140.2ms | 0.62x | 200.0ms | 146.1ms | 0.73x |
| q15 | 194.0ms | 135.5ms | 0.70x | 183.0ms | 139.6ms | 0.76x |
| q16 | 166.0ms | 159.1ms | 0.96x | 187.0ms | 161.2ms | 0.86x |
| q17 | 218.0ms | 160.2ms | 0.73x | 213.0ms | 165.1ms | 0.77x |
| q18 | 696.0ms | 544.2ms | 0.78x | 695.0ms | 507.8ms | 0.73x |
| q19 | 301.0ms | 148.7ms | 0.49x | 268.0ms | 153.1ms | 0.57x |
| q20 | 241.0ms | 198.4ms | 0.82x | 244.0ms | 192.3ms | 0.79x |
| q21 | 420.0ms | 535.7ms | 1.28x | 390.0ms | 537.5ms | 1.38x |
| q22 | 124.0ms | 89.5ms | 0.72x | 116.0ms | 94.9ms | 0.82x |
| total | 6651.0ms | 6194.9ms | **0.93x** | 6056.0ms | 5920.3ms | **0.98x** |

The absolute times are believable now. DuckDB averages about 290ms a query over SF1 on one thread, which is the right order for this hardware, against the 1.04s a query the loaded box produced.

## The work list

Five queries are slower than DuckDB in both runs, and they are the same five. Ordered by the worse of the two ratios:

- q01 at 1.34x and 1.42x. A scan, a filter that keeps almost everything, and eight aggregates over four groups. There is no join and nothing to prune, so this is per row aggregate work and nothing else, which makes it the cleanest of the five to chase.
- q13 at 1.32x and 1.38x. A left outer join of customer against orders with a `NOT LIKE` on the comment, then a group by on the count.
- q21 at 1.28x and 1.38x. Three lineitem references, an exists and a not exists.
- q12 at 1.25x and 1.30x. Two table join, the query issue #1625 was filed about.
- q09 at 1.09x and 1.08x. Six way join, and the largest absolute time in the suite for either engine.

q03, q04 and q05 land either side of parity between the two runs and are not findings.

## What rudb is already ahead on

q19 at 0.49x and 0.57x, q11 at 0.61x and 0.63x, q14 at 0.62x and 0.73x, q08 at 0.67x and 0.71x. Nine queries are at 0.80x or better in both runs. So the 0.93x total is not an engine that is uniformly a little behind, it is an engine that is well ahead on most of the suite and loses it back on five queries.

## On q12 and zone maps

Issue #1625 proposed that q12 was slow because the lineitem scan pruned no parts on a one year range over `l_receiptdate`. That reading was wrong on both halves.

The pruning is working. A predicate nothing can satisfy skips every part of lineitem, so the zone maps are present, they are consulted and they decide. Zero parts pruned on the 1994 range is the correct answer, because TPC-H generates `o_orderdate` uniformly at random across the seven year window independently of the order key, so the dates carry no relationship to storage order and every part of lineitem holds dates spanning most of the range. No engine can prune that, DuckDB included.

The scan time that pointed at pruning was also an artifact. It came from a `--metrics` profile on the loaded box, where the lineitem scan was 88% of a 2.998s execute. On a quiet box the whole query is 283ms.

## Reproducing

Both engines single threaded, seven rounds, alternating, medians. The script is not in the harness yet, which is the thing to fix next if this measurement is going to be repeated, and it is the one recommendation the earlier report made that still stands.
