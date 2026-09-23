# Where TPC-H actually stands against DuckDB

**The 0.85x table below is superseded and the q12 2.25x in it should not be quoted.** The same protocol on a machine that was not busy gives 0.93x and 0.98x, and q12 comes out at 1.25x rather than 2.25x, which moves it from the worst query in the suite to the fourth worst. See `reports/2026-09-24/tpch-per-core-on-a-quiet-machine.md`. The two structural findings here still hold: every query finishes and every answer agrees with DuckDB. The rest is left as it was written because it is a correct record of what a loaded box produced.

**Not a publishable number.** Every measurement here was taken on server3 while its one minute load average sat between 14 and 52 on 8 hardware threads, because no quiet machine was available at any point. The harness refused to publish all four of its runs for that reason and it was right to. What follows is worth writing down anyway, because the previous report on this comparison is wrong in a way that matters and the correction does not depend on the load.

## The number this project was measuring itself against is stale

`reports/2026-09-18/run-tpch-macbook-air-m4.md` is the standing TPC-H comparison. It says rudb timed out at 60 seconds on 17 of the 22 queries, failed q11, and came out **901.01x slower than DuckDB** on the five queries it finished. Its engine table gives the reason: rudb read "the source Parquet, this engine has no storage format of its own yet", so every query paid a full Parquet decode, while DuckDB loaded into its own file once in 2.861s.

That report is from rudb 0.3.38. None of it is true of rudb 0.4.20.

The harness now loads rudb into a native database file by default, and has done since the `RUDB_BENCH_RUDB_VIEWS` switch was added. Running the same suite at SF1 on current main:

| | DuckDB v2.0.0-dev84237 | rudb 0.4.20 |
| --- | --- | --- |
| queries that finished | 22 of 22 | 22 of 22 |
| queries that timed out | none | none |
| answers agreeing with the other engine | 22 of 22 | 22 of 22 |
| loaded into | its own database file | its own database file |
| on disk | 266.51 MiB | 250.66 MiB |

No timeouts, no failures, and the answer check passed on every query to the last significant digit of a double. The 901x is gone and so is the thing that caused it.

## rudb is at parity, not at 10x, and not at the 0.52x the suite reported

Four sequential suite runs put rudb at 0.60x, 0.52x, 1.02x and a best-of-four of 0.52x against DuckDB. Those numbers should not be believed. The spread across runs is a factor of two with nothing changing between them, and DuckDB's own reported query time averaged about 1.04s per query when an uncontended DuckDB does TPC-H SF1 in roughly 35ms a query. The box was inflating DuckDB about 23x and rudb about 5x, so the suite was measuring which engine copes better with a starved machine rather than which engine is faster.

The asymmetry is itself plausible. The suite runs one engine to completion and then the other, so the two blocks see different load, and an engine that wants all 8 threads at once loses more to a run queue of 40 than one that does not.

Pinning both engines to a single thread and alternating them query by query inside one loop removes both problems. One thread competes for one timeslice instead of needing eight simultaneously, and interleaving the arms means each engine's samples are drawn from the same load. Five rounds, alternating which engine goes first each round, median per query, each engine's own reported run time:

| query | DuckDB | rudb | ratio |
| --- | --- | --- | --- |
| q01 | 2918.0ms | 3251.1ms | 1.11x |
| q02 | 344.0ms | 278.5ms | 0.81x |
| q03 | 2626.0ms | 2581.2ms | 0.98x |
| q04 | 2040.0ms | 2774.9ms | 1.36x |
| q05 | 3074.0ms | 3101.6ms | 1.01x |
| q06 | 2198.0ms | 1308.4ms | 0.60x |
| q07 | 2983.0ms | 2071.6ms | 0.69x |
| q08 | 4108.0ms | 1762.9ms | 0.43x |
| q09 | 7175.0ms | 7436.0ms | 1.04x |
| q10 | 4341.0ms | 3518.1ms | 0.81x |
| q11 | 584.0ms | 264.0ms | 0.45x |
| q12 | 1507.0ms | 3388.6ms | 2.25x |
| q13 | 5701.0ms | 6613.7ms | 1.16x |
| q14 | 2751.0ms | 1981.2ms | 0.72x |
| q15 | 2494.0ms | 1356.4ms | 0.54x |
| q16 | 1872.0ms | 1587.1ms | 0.85x |
| q17 | 3105.0ms | 1619.0ms | 0.52x |
| q18 | 8955.0ms | 5127.1ms | 0.57x |
| q19 | 2893.0ms | 1733.6ms | 0.60x |
| q20 | 2721.0ms | 1269.8ms | 0.47x |
| q21 | 2839.0ms | 3972.0ms | 1.40x |
| q22 | 924.0ms | 610.4ms | 0.66x |
| total | 68153.0ms | 57607.1ms | **0.85x** |

Per core, the two engines are about the same speed. rudb is ahead on 14 queries and behind on 8. The goal is 10x, so the gap to close is about 8.5x and not the 1.9x the multi threaded suite suggested.

The absolute times here are still inflated, by perhaps 20x, so no cell in that table is a measurement of anything on its own. The column that survives is the ratio, because that is what the pairing protects.

## The work list this produces

Eight queries are slower than DuckDB per core, and they are where the next work goes:

- q12 at **2.25x**, the worst by a wide margin, and the query the link join work has been aimed at. Worth knowing that the recent gains there were closing a gap against rudb's own hash plan rather than against DuckDB.
- q21 at 1.40x and q04 at 1.36x
- q13 at 1.16x and q01 at 1.11x
- q09, q05 and q03 are within 5% and are not findings yet

## What has to happen before any of this is publishable

A quiet machine. server1 sat at load 177 on 4 cores, server2 at 35 on 6, server3 between 14 and 52 on 8, and gamingpc is Windows and could not be reached on these paths. Until one of them is free, or the suite runs somewhere it has the machine to itself, the only defensible statements are the two structural ones: every query finishes and every answer is right, and per core the two engines are close.

## Reproducing

```
RUDB_BENCH_RUDB=<rudb binary> RUDB_BENCH_DATA=<corpus root> \
  rudb-bench run tpch --engines duckdb,rudb --scale 1 --runs 5 --timeout 120 --report
```

The corpus goes under `tpch/sf1` as `<table>.parquet`. The paired single threaded comparison is not in the harness and was a script on the box, which is the next thing to fold in here if this measurement is going to be repeated.
