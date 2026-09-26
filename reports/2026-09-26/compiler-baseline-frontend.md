# Compiler baseline: frontend latency per suite

This is the frontend latency box of milestone C0 in tamnd/rudb#1828. It gives the median and the max frontend time per suite, single threaded, for ClickBench, TPC-H and JOB, cold and warm. The frontend is parse, bind and optimize, where optimize includes the rewrite passes. The budget from the 2026-09-24 report is a median of 0.3 ms and a max of 2 ms per suite. Every suite is over the median budget, so the plan for cutting it is at the end.

## Summary

Times are in microseconds, taken from the per query medians.

| Suite | Queries | Cold median | Cold max | Warm median | Warm max | Over 300 us warm |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| ClickBench (10M sample) | 43 | 1027.1 | 4978.5 (q30) | 418.5 | 4187.2 (q30) | 38 of 43 |
| TPC-H SF1 | 22 | 1045.3 | 1525.9 (q02) | 701.0 | 1007.6 (q08) | 22 of 22 |
| JOB | 113 | 1199.5 | 2250.8 (29b) | 912.0 | 1951.3 (29a) | 113 of 113 |

The same over the fastest round of each query, which is the least disturbed by other work on the machine:

| Suite | Cold median | Cold max | Warm median | Warm max |
| --- | ---: | ---: | ---: | ---: |
| ClickBench | 901.1 | 4199.4 (q30) | 355.9 | 3754.2 (q30) |
| TPC-H SF1 | 951.4 | 1347.1 (q08) | 568.3 | 850.9 (q21) |
| JOB | 1054.4 | 1972.4 (29a) | 758.7 | 1487.8 (29c) |

Cold is the first statement in a fresh process. Warm is the same statement again in the same process with the plan cache bypassed, so it is the frontend with the catalog, the table readers and the allocator already warm. Warm is the number a long lived server or a repeated query pays, and it is the one the budget should be held to. Cold is the one the command line pays on every run.

Only ClickBench q30 is over the 2 ms max, cold and warm. JOB is over 2 ms cold on 4 queries (29a, 29b, 29c and 24b) and under it warm. The median is the problem: no suite is near 0.3 ms, and the cheapest warm query of all, ClickBench q1 (`SELECT COUNT(*) FROM hits`), takes 186 us.

## Where the time goes

Each phase's share of the frontend, summed over the per query medians:

| Suite | Run | Parse | Bind | Rewrite | Optimizer |
| --- | --- | ---: | ---: | ---: | ---: |
| ClickBench | cold | 18% | 56% | 15% | 11% |
| ClickBench | warm | 40% | 35% | 15% | 10% |
| TPC-H SF1 | cold | 31% | 34% | 21% | 14% |
| TPC-H SF1 | warm | 45% | 25% | 15% | 15% |
| JOB | cold | 39% | 33% | 24% | 4% |
| JOB | warm | 48% | 27% | 21% | 4% |

Medians of each phase across the queries, in microseconds:

| Suite | Run | Parse | Bind | Rewrite | Optimizer |
| --- | --- | ---: | ---: | ---: | ---: |
| ClickBench | cold | 149.7 | 626.9 | 113.6 | 82.6 |
| ClickBench | warm | 164.4 | 161.2 | 37.4 | 43.4 |
| TPC-H SF1 | cold | 329.6 | 346.4 | 222.8 | 118.0 |
| TPC-H SF1 | warm | 320.8 | 171.6 | 94.8 | 91.3 |
| JOB | cold | 462.8 | 404.3 | 277.5 | 39.0 |
| JOB | warm | 442.2 | 248.0 | 184.8 | 27.8 |

What this says:

- Parse is the biggest phase warm in every suite, and it does not get cheaper warm because it does not touch the catalog. It grows with the length of the text: about 160 us for a ClickBench query, 320 us for TPC-H and 440 us for JOB. ClickBench q30, ninety `SUM(ResolutionWidth + N)` terms, parses in 1.67 ms.
- Bind is the biggest phase cold. On ClickBench cold bind is 627 us against 161 us warm, so about 465 us of a cold ClickBench statement is first touch of the 105 column table: opening it, reading its statistics and building the scope. That work is repeated by every fresh process.
- Rewrite is small on ClickBench but 185 us warm on JOB, where the queries have many joins and filters and each rewrite rule walks the whole plan.
- The optimizer is under 100 us warm on every suite median, but reaches 200 to 480 us cold on the filtered ClickBench queries (q13, q14, q15, q22, q23, q28, q38).
- q30 is quadratic, not slow per term. Its warm rewrite (1.61 ms) is as large as its parse, and its bind is 789 us, against 30 to 50 us of rewrite and 120 to 250 us of bind for the other ClickBench queries.

## How this compares with the 2026-09-24 numbers

The 2026-09-24 ClickBench report measured a frontend median of 1.304 ms and a max of 6.519 ms (q30) on server3 at a load around 30, over 9 rounds taken through the harness, with each round a fresh process. That is the cold number. On the quieter server2 the cold ClickBench median is 1.027 ms and the max 4.979 ms, so about a fifth of the old number was the load. Most of it is real.

A first pass of this run was taken on server2 at a load of 5 to 6 before the machine went quiet. It gave a ClickBench cold median of 1107.2 us and warm median of 454.0 us, and a TPC-H cold median of 1168.5 us and warm median of 726.6 us. The quiet numbers above are 3 to 10 percent lower, and the ranking of the queries is the same.

## Machine, data and protocol

- Machine: server2, 6 vCPU AMD EPYC (with IBPB) at 2.0 GHz, 11 GB of memory, shared with other projects. The one minute load at the start of each process was a median of 1.42 and a max of 1.99 during ClickBench, a median of 1.14 and a max of 1.18 during TPC-H, and a median of 1.15 and a max of 5.71 during JOB. No process started above the gate.
- rudb: 0.5.0 built in release mode from tamnd/rudb at 22ffd0b.
- ClickBench: the 10M row sample (9,999,750 rows) of hits, loaded from `hits-10m.parquet` with the same `CREATE TABLE hits AS SELECT * REPLACE (...)` the harness uses. The frontend does not read the data, so the row count does not change these numbers, but the 105 column schema does. `stored_answers` is off so that no query is answered from a stored result, which would skip the frontend work being measured.
- TPC-H: scale factor 1 from dbgen, the 22 queries as the suite runs them.
- JOB: the 113 queries in `queries/job` against the full IMDb database.
- Protocol: for each query, 5 fresh processes, each running the query once cold and then 3 more times warm, single threaded (`threads=1`). Each warm repeat adds spaces after the first keyword so the plan cache does not answer it. A query's cold number is the median of its 5 cold runs, its warm number the median of its 15 warm runs, and its fastest number the smallest of those. The phase times are the `parse_ns`, `bind_ns` and `optimize_ns` (which includes `rewrite_ns`) that rudb writes to its metrics file for each statement. Before each process the script waits for the one minute load to fall under 6 and for at least 2 GB free on the disk it writes to.
- The per query tables, with every phase including the physical build and execution, are in `compiler-baseline-frontend/{clickbench,tpch,job}.tsv`, and the full script output in the `.txt` files next to them.

## Reproduce

The script is `scripts/frontend-latency.py` in this repository, as changed in this pull request. The ClickBench and TPC-H query files are written from the suite text with the export example:

```
cargo run --release --example export_queries -- clickbench rudb clickbench.sql
cargo run --release --example export_queries -- tpch rudb tpch-q.sql
```

Each suite is then one command:

```
python3 scripts/frontend-latency.py --rudb rudb --database hits.rudb --queries clickbench.sql --processes 5 --repeats 3 --gate 6 --patience 600 --set stored_answers=false --tsv clickbench.tsv
python3 scripts/frontend-latency.py --rudb rudb --database tpch1.rudb --queries tpch-q.sql --processes 5 --repeats 3 --gate 6 --patience 600 --tsv tpch.tsv
python3 scripts/frontend-latency.py --rudb rudb --database imdb.rudb --queries queries/job --processes 5 --repeats 3 --gate 6 --patience 600 --tsv job.tsv
```

## Script changes in this pull request

- A metrics document that was cut off part way now fails that query with the exit status and the end of stderr, instead of stopping the whole run with a JSON error. A JOB pass did stop that way on 5c when server2's disk ran out.
- Before each process the script now also waits while the disk under its scratch directory has less than 2 GB free, printing how much is free every 5 minutes, so a full disk pauses the run instead of cutting lines off.

## Plan for the frontend

Every suite is over 0.3 ms, so this plan covers all three. The steps are in the order of what they are worth across the suites, with the source paths as of 22ffd0b. Each step should be checked against this report with the same commands, warm medians first.

1. Parse, the biggest warm phase everywhere (40 to 48 percent). The grammar is a PEG with 1,088 rules, of which 22 are memoized, and each expression goes down about 20 precedence levels before it reaches a column name or a literal. Add a fast path for the common expression shapes (a column, a literal, a function call, a binary comparison) that skips the precedence ladder, and memoize the rules that the profile shows are retried after a failed alternative. The target is under 60 us for a ClickBench query and under 150 us for a JOB query.
2. Bind on a cold statement. Cache the per table statistics the binder reads (the distinct counts, the widths and the held summary) on the table reader, so a statement reads them once per table rather than per column reference, and they survive to the next statement in the same process. This is most of the 465 us gap between cold and warm ClickBench bind.
3. Linear scans that make wide queries quadratic. Column lookup in `crates/rudb-bind/src/scope.rs` (`self.columns.iter().filter(...)`) walks every visible column per reference; the aggregate dedup in `crates/rudb-bind/src/binder.rs` compares each new aggregate against every earlier one with `same_expr`; and `Plan::intern` in `crates/rudb-plan/src/plan.rs` finds a string with a linear `position`. Replace each with a hash map keyed the same way. This is what q30 pays for with its 90 aggregates, and it matters for every JOB query with a dozen tables in scope. The constant folding in `crates/rudb-opt/src/fold.rs` and the shared expression pass in `crates/rudb-opt/src/shared.rs` have the same shape and should be checked in the same change.
4. Rewrite. There are nine rewrite rules plus unnesting, and each walks the whole plan. Fuse the ones that look at one node at a time into one bottom up walk, and skip a rule when the plan has no node of the kind it rewrites. On JOB this is 185 us warm.
5. The optimizer. It runs 21 passes with a fixed cost of about 19 us before any of them does work. Skip the passes that cannot apply (join ordering on a one table query, filter pushdown with no filter) and look at the cold cost on the filtered ClickBench queries, 200 to 480 us, which is most likely first touch of the statistics again and is covered by step 2.

The physical build is outside the frontend and outside the budget, but it is 451 us cold on ClickBench and 716 us cold on TPC-H at the median, more than any single frontend phase, so it is the next thing to measure after the frontend is in budget.
