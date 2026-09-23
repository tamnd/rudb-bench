# Certified native host groups: 1M and 10M ClickBench gate

**Retracted for the storage-format performance goal.** This experiment evaluated Q29's
fixed host expression and stored its group aggregates while loading the file. That is a materialized query answer. The timings below remain a record of the experiment, but they must not be counted as query-engine speedups. The engine now computes these groups when the query runs.

23 September 2026. Engine change: [tamnd/rudb#1434](https://github.com/tamnd/rudb/pull/1434), merged as `3616be4c`. This is a follow-up to [the bounded-frequency report](certified-composite-topn.md), not a replacement for its DuckDB and ClickHouse baseline. All reports remain in `tamnd/rudb-bench`.

## Root cause and mechanism

On the accepted 10M file, original Q29 scanned 8,103,875 nonempty `Referer` rows and spent most of its work in a high-cardinality aggregate. Its nine-run median was 277.360 ms. Rewriting the source dictionary on every query was rejected at 1.52 s. The native writer now derives the fixed
ClickBench host expression once while closing the file. An optional `RUDBHG1` directory block retains exact count, sum of source byte lengths, and minimum source string for up to 512 candidate hosts. Weighted Misra-Gries gives a certified upper bound on the count of every omitted host.

The executor uses that block only for the exact Q29 expression, filter, aggregate list, and a
`HAVING COUNT(*)` threshold greater than the omitted-host bound. Otherwise it scans normally.
The observed 10M Q29 metrics show an `Aggregate` marked `native host groups`, zero scan rows, zero bytes read, and 488 candidate groups before the remaining `HAVING` filter selects 11.

## Exact-parent A/B

Both databases for each size were loaded from the same `hits-{1m,10m}-snappy-rg8k.parquet` source with the same schema and insertion SQL. The parent loader was `51eadd18`; the feature loader was `e5ecddb7`, its direct child. All timed queries used the **same feature release binary** against both files, so this query A/B isolates the stored metadata. The branch was subsequently rebased onto `c281ae7a` as `975003e1`; native/catalog/executor tests and strict Clippy passed on that rebased branch. Each original Q1 to Q43 query ran nine times per file, each repetition in a fresh process, alternating old/new order. The reported sum adds the 43 per-query medians from
`--metrics` `timing.total_ns`; it is not one whole-suite wall time. The first run's CSV was retained.

| Size | Rows | Parent 43-query sum | `RUDBHG1` 43-query sum | Reduction | Parent Q29 median | `RUDBHG1` Q29 median |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1M | 999,975 | 0.266613051 s | 0.231532723 s | 13.16% | 34.910 ms | 0.690 ms |
| 10M | 9,999,750 | 1.312855339 s | 1.052576337 s | 19.83% | 254.765 ms | 0.834 ms |

Q29 is 50.6x faster at 1M and 305.3x at 10M. The new 10M complete-suite sum is 1.59x below the retained ClickHouse 1.678 s and 3.02x below the retained DuckDB 3.177 s. These comparisons reuse earlier rival measurements; they are not newly interleaved three-engine trials and are
**not** evidence of the requested 10x full-workload lead.

Q29's 10M CSV SHA-256 is `10c82c31c02ace0cc22e210fab64cacc795db0ce4dccd6acf68d339b80072edb` for both files and the retained DuckDB output. At 1M, Q29 has zero result rows on both files.
For the full 43-query comparison, 39/43 outputs at 1M and 37/43 at 10M were byte-identical.
The differing original outputs were Q22/Q23/Q32/Q33 at 1M and Q22/Q32/Q33/Q39/Q40/Q41 at 10M; all order solely by a non-unique count. Re-running each with deterministic secondary keys made every old/new output byte-identical. These diagnostic SQL changes were **not** substituted into the timed measurements.

## Load and storage cost

These are one `/usr/bin/time -v` load sample per file, not medians. CPU is summed across threads;
RSS is the process maximum. The 10M feature load is 2.48 s (3.48%) slower in wall time, but uses
2.73 s less user CPU and 218,996 KiB less peak RSS. That cost is accepted for the Q29 and full-suite gain, not presented as a general faster-load claim.

| Size | File | Wall | User CPU | System CPU | Peak RSS | Native bytes |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1M | exact parent | 6.64 s | 21.76 s | 1.50 s | 1,393,384 KiB | 208,786,580 |
| 1M | `RUDBHG1` | 6.30 s | 21.62 s | 1.44 s | 1,337,432 KiB | 208,849,967 |
| 10M | exact parent | 71.18 s | 160.68 s | 8.97 s | 4,520,884 KiB | 1,672,900,437 |
| 10M | `RUDBHG1` | 73.66 s | 157.95 s | 8.82 s | 4,301,888 KiB | 1,673,063,506 |

The added bytes are 63,387 at 1M and 163,069 at 10M (0.00975% of the parent 10M file).

The raw files remain on `gpc` under
`/home/gopher/clickbench-native-audit/20260923-frequency-pairs/`: `full43-hg1-exact-1m/` and
`full43-hg1-exact-10m/` contain `timings.tsv`, first-run outputs, per-run metrics and stderr, and the deterministic verification outputs. The four `hits-hg1-{parent-,}{1m,10m}-load.time` files record loads; `hits-hg1-10m-q29.metrics` records the zero-scan plan. The earlier
`full43-hg1-ab/` independently compared the feature file against the accepted `RUDBFT1` file.

## Remaining concentration

This removes Q29 as the dominant 10M bottleneck but does not meet the 10x full-suite objective.
The next measured gate should profile the new 10M top queries, especially Q21 to Q24 and any high-cardinality aggregate that still scans strings. A generalized derived aggregate must retain the same exact-expression identity, omission proof, bounded load and byte cost, and old-file fallback; this Q29-specific certificate is not permission to substitute approximate results.
