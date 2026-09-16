# ClickBench at ten million rows and the 100M reference baseline

The ten-million-row slowdown has two compounding causes: ClickBench's high-cardinality aggregates become the critical path, and RuDB's native-table source still capped every scan at four workers.
On the 32-thread `gamingpc-wsl` host, allowing a large snapshot to use the configured 16 workers reduces the full-suite median-timer sum from 5.655 seconds to 4.540 seconds, a 19.7 percent improvement.
It raises CPU consumption by 50.7 percent, so this is a latency/parallelism fix rather than a reduction in total work.

Implementation: [tamnd/rudb#675](https://github.com/tamnd/rudb/pull/675).
ClickBench integration: [tamnd/ClickBench#1](https://github.com/tamnd/ClickBench/pull/1).

This does not meet the 10x goal.
Against the fastest previously measured 10M rival, ClickHouse at 1.552 seconds, the target is 155.2 milliseconds.
The improved result is still 29.3 times over that budget, 2.93 times slower than ClickHouse, and 1.43 times slower than DuckDB's 3.179 seconds.

## Query count and coverage

The upstream ClickBench `queries.sql` contains 43 statements, numbered Q1 through Q43.
There are no upstream Q44 or Q45.
The two queries that RuDB formerly skipped were Q19 and Q33; both now run, so these measurements cover the complete original suite, 43 of 43.
Adding two synthetic statements would no longer be an original ClickBench result.

| Query | Shape | 10M baseline | 10M adaptive | Change |
| ---: | --- | ---: | ---: | ---: |
| 19 | three high-cardinality keys and `COUNT(*)` | 508.7 ms | 355.6 ms | -30.1% |
| 33 | two keys with count, sum, and average | 289.8 ms | 249.0 ms | -14.1% |

Every one of Q1-Q43 succeeded in both sides of the A/B.
Q19's output is byte-identical.
Seven queries with tied `ORDER BY ... LIMIT` boundaries produced different row order or a different tied boundary row (Q22, Q23, Q32, Q33, Q39, Q40, and Q41); the same variation occurs between repeated runs of an unchanged binary, so these require the repository's tie-aware verifier rather than a raw byte comparison.

## Historical root-cause profile

The first complete 10M native run used RuDB 0.3.22, DuckDB 2.0.0-dev84237, and ClickHouse server 26.9.1.1162 on the same host.
The deterministic verifier found no unresolved answer difference.

| Engine | 43-query timer sum | CPU sum | Peak RSS |
| --- | ---: | ---: | ---: |
| ClickHouse server | 1.552 s | not collected | not collected |
| DuckDB native | 3.179 s | 38.320 s | 1.60 GiB |
| RuDB native | 4.769 s | 43.400 s | 1.44 GiB |

Nine grouping queries accounted for 2.694 seconds, or 56.5 percent, of RuDB's total.
Adding Q29's regular-expression grouping made the ten slowest queries 3.065 seconds, or 64.3 percent.

| Query | RuDB | DuckDB | RuDB CPU | Shape |
| ---: | ---: | ---: | ---: | --- |
| 19 | 486.1 ms | 216.0 ms | 3.630 s | three high-cardinality keys and `COUNT(*)` |
| 33 | 455.9 ms | 184.0 ms | 4.531 s | two keys and compact numeric aggregate state |
| 29 | 371.8 ms | 335.0 ms | 3.409 s | regular expression, grouping, and string minimum |
| 35 | 295.8 ms | 256.0 ms | 2.911 s | high-cardinality URL and `COUNT(*)` |
| 17 | 289.9 ms | 148.0 ms | 2.462 s | two high-cardinality keys and `COUNT(*)` |
| 34 | 287.3 ms | 231.0 ms | 2.821 s | high-cardinality URL and `COUNT(*)` |
| 18 | 284.6 ms | 148.0 ms | 0.367 s | unordered pushed limit, deliberately serial |
| 10 | 228.5 ms | 129.0 ms | 1.375 s | grouping with a distinct aggregate |
| 9 | 191.2 ms | 65.0 ms | 1.033 s | grouping with a distinct aggregate |
| 16 | 174.3 ms | 55.0 ms | 1.671 s | high-cardinality integer and `COUNT(*)` |

Stage metrics confirmed that fold, merge, and emit dominate rather than the native scan itself.
For example, Q19 spent 4.168 CPU seconds folding and 872 milliseconds emitting, while Q33 spent 1.674 seconds folding, 2.126 seconds merging, and 575 milliseconds emitting.
A shared-table Q33 experiment removed the merge but increased lock contention and regressed median wall time by 2.8 percent.
A direct typed-output experiment became neutral after the radix exchange landed on main (+0.5 percent at 1M and +0.2 percent at 10M), so neither experiment is part of the proposed change.

Q18 remains deliberately single-instance because its unordered `LIMIT 10` is pushed into the aggregate.
Independent worker-local limits can change which groups receive later rows and return wrong counts.
It needs a global bounded-group protocol.
Q29 also retains expression cost from the regular expression and string minimum after hash-table work is removed.

## Adaptive native scan A/B

The exact A/B is based on main commit `2ec428b` after the fixed aggregate radix exchange landed.
For each query and revision, five fresh processes were run in interleaved order; the table sums the median total time for each of the 43 queries.
Both binaries read the same 9,999,750-row native database and differ only by the adaptive worker commit.

| Revision | Native workers at 10M | Query timer sum | CPU sum | Peak tracked bytes |
| --- | ---: | ---: | ---: | ---: |
| main, fixed cap | 4 | 5.654901 s | 16.735039 s | 3,048.58 MiB |
| adaptive | 16 | 4.539916 s | 25.218896 s | 3,050.40 MiB |

| Largest wins | Before | After | Saved |
| ---: | ---: | ---: | ---: |
| Q29 | 711.5 ms | 397.8 ms | 313.7 ms |
| Q19 | 508.7 ms | 355.6 ms | 153.1 ms |
| Q14 | 267.8 ms | 177.3 ms | 90.4 ms |
| Q17 | 286.2 ms | 201.9 ms | 84.3 ms |
| Q36 | 203.4 ms | 124.7 ms | 78.7 ms |
| Q16 | 241.1 ms | 174.4 ms | 66.7 ms |
| Q5 | 218.9 ms | 161.6 ms | 57.3 ms |
| Q31 | 153.7 ms | 99.7 ms | 54.0 ms |
| Q15 | 164.3 ms | 113.3 ms | 51.0 ms |
| Q33 | 289.8 ms | 249.0 ms | 40.9 ms |

Short queries can regress because extra workers cost more to start and coordinate than they save.
A forced 16-worker run at 1M regressed from 660.276 milliseconds to 769.937 milliseconds, or 16.6 percent.
The selected rule therefore preserves the old choices through 2.5M rows, uses eight workers at 5M, and reaches 16 at 10M.
The 1M path is unchanged.

## Native dictionary scaling defect

Creating the 10M native database exposed an independent correctness defect.
The writer created a valid 4.6 GiB file with a whole-column lazy string dictionary larger than 256 MiB, but the reader applied the ordinary per-page limit to that dictionary and rejected its own file as having a page range outside the file.
Dictionary pages are whole-column structures and are intentionally allowed to exceed the ordinary page cap.
The fix removes that cap only for dictionary pages; ordinary data pages remain bounded.
The repaired reader reopens the file and returns all 9,999,750 rows.

## Original 100M DuckDB and ClickHouse baseline

The fork's unmodified 43 queries and common driver were run on `gpc` over all 99,997,497 rows of the official 100M Parquet dataset.
These are reference numbers for the later RuDB 100M run, not a claim about the 10M optimization.

| Engine | Version | Load | Stored data | Cold first-attempt sum | Official hot sum |
| --- | --- | ---: | ---: | ---: | ---: |
| DuckDB | 1.5.5 `d8cdaa33fd` | 54.284 s | 20,451,962,880 B | 21.882 s | 16.804 s |
| ClickHouse server | 26.9.1.1562 | 46.937 s | 9,638,208,960 B | 12.270 s | 9.618 s |

The official hot score is the sum of the better of attempts two and three for each query.
Q19 and Q33 are present: DuckDB scored 1.161 and 1.378 seconds respectively; ClickHouse scored 1.093 and 1.211 seconds.
DuckDB's original uncapped loader exceeded the available 31 GiB while unrelated host services were active, so its execution wrapper set `memory_limit='16GB'`, `threads=16`, and a 200 GB spill limit; the query file and common benchmark driver were unchanged.
ClickHouse's concurrent phase reported 3.238 QPS with a 2.2 percent error ratio under the same host contention, so that concurrent number is retained as provenance but is not used for the serial comparison.

Raw host artifacts are retained at:

- `/home/gopher/clickbench-native-audit/run-20260916-v3-direct-output/10m-native-adaptive-v2`
- `/home/gopher/tamnd-clickbench-100m/artifacts-20260916/duckdb-1.5.5-16g.log`
- `/home/gopher/tamnd-clickbench-100m/artifacts-20260916/clickhouse-fixed.log`

## Validation

- `cargo fmt --all -- --check`
- `cargo test -p rudb-native` (5 passed)
- `cargo test -p rudb-exec native_workers_grow_only_after_a_snapshot_can_amortize_them --lib`
- `cargo clippy -p rudb-native -p rudb-exec --all-targets -- -D warnings`
- complete 43-query, five-process interleaved 10M A/B on `gamingpc-wsl`
- complete original 43-query DuckDB and ClickHouse 100M runs on `gpc`

The repository-wide `cargo xtask ci` currently stops at four row-loop-policy findings in `crates/rudb-exec/src/group.rs` introduced by base commit `2ec428b`.
The identical failure was reproduced in a clean detached `origin/main` worktree; neither changed file in this work triggers that check.
