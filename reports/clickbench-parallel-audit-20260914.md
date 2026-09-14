# Parallel ClickBench audit

This run measures rudb after pipeline parallelism, parallel Parquet row-group scans and mergeable grouped aggregates landed. It uses all 43 ClickBench queries at 1 thousand, 10 thousand, 100 thousand and 1 million rows. Each reported query time is the median of five hot repetitions in fresh processes after one first execution.

DuckDB native reads a loaded table. DuckDB Parquet and rudb read the same Snappy Parquet input. Query time comes from each CLI timer. Process wall time, CPU time and peak RSS come from Linux `wait4` for the child process. Peak RSS is the maximum resident set of one query process.

| Size | Engine | Queries | Query median sum | Process wall sum | CPU sum | Peak RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1k | DuckDB native | 43/43 | 0.122 s | 0.997 s | 1.216 s | 58.15 MiB |
| 1k | DuckDB Parquet | 43/43 | 0.153 s | 1.075 s | 1.273 s | 57.32 MiB |
| 1k | rudb | 43/43 | 0.022 s | 0.052 s | 0.051 s | 6.14 MiB |
| 10k | DuckDB native | 43/43 | 0.147 s | 1.024 s | 1.265 s | 57.40 MiB |
| 10k | DuckDB Parquet | 43/43 | 0.181 s | 1.110 s | 1.345 s | 58.69 MiB |
| 10k | rudb | 43/43 | 0.070 s | 0.099 s | 0.104 s | 8.64 MiB |
| 100k | DuckDB native | 43/43 | 0.327 s | 1.219 s | 1.587 s | 86.30 MiB |
| 100k | DuckDB Parquet | 43/43 | 0.452 s | 1.401 s | 1.943 s | 139.41 MiB |
| 100k | rudb | 43/43 | 0.583 s | 0.615 s | 0.655 s | 34.68 MiB |
| 1m | DuckDB native | 43/43 | 0.554 s | 1.465 s | 3.836 s | 311.91 MiB |
| 1m | DuckDB Parquet | 43/43 | 0.830 s | 1.848 s | 5.210 s | 659.07 MiB |
| 1m | rudb | 43/43 | 2.997 s | 3.046 s | 6.540 s | 248.38 MiB |

The small inputs remain dominated by startup cost. At 1 million rows, parallel execution cuts rudb's query-time sum from 5.246 seconds to 2.997 seconds, a 42.9 percent reduction. The result is still 5.4 times slower than DuckDB native and 3.6 times slower than DuckDB Parquet.

Peak RSS moves in the other direction, from 170.86 MiB before parallel aggregation to 248.38 MiB. rudb uses 20 percent less peak RSS than DuckDB native and 62 percent less than DuckDB Parquet in this run. It is still far from the target of one tenth of DuckDB native.

The speedup and memory increase have the same cause. Every worker opens an independent Parquet reader and builds a local aggregate hash table. The workers scan and aggregate concurrently, then `Aggregate::merge` serially probes the local tables into the first completed table. Duplicate group keys therefore occupy several tables during the scan, all tables are live at the merge boundary, and the merge repeats hash probes for groups already processed once.

The next architectural step is radix partitioning. Rows should be assigned by the high bits of the group hash before accumulation, with each partition owned by one aggregate task. A group will then exist in one table, local tables will not duplicate keys, and there will be no serial duplicate-group merge. The same ownership rule restores the invariant required for parallel aggregate spilling.

All 172 size and query pairs completed. The verifier compared output with DuckDB and reran ambiguous ties with deterministic ordering. Every deterministic retest matched. Differences left in the report are row-order differences in queries whose result order is not fully specified.

## Parallel aggregate merge follow-up

A second 1 million row run measures parallel `COUNT(DISTINCT)` and the tree-shaped aggregate merge. DuckDB was rerun in the same session because machine conditions changed enough that reusing its earlier number would give a misleading ratio.

| Engine | Queries | Query median sum | Process wall sum | CPU sum | Peak RSS |
| --- | ---: | ---: | ---: | ---: | ---: |
| DuckDB native | 43/43 | 0.627 s | 1.657 s | 4.402 s | 313.71 MiB |
| DuckDB Parquet | 43/43 | 0.915 s | 2.038 s | 5.936 s | 646.52 MiB |
| rudb | 43/43 | 2.782 s | 2.837 s | 8.200 s | 287.68 MiB |

The total improves by 7.2 percent from the preceding rudb run. q23 improves from 0.407 seconds to 0.084 seconds because its distinct sets now merge across workers. Its peak RSS rises from 62.8 MiB to 287.7 MiB. The suite peak follows q23 and rises by 15.8 percent.

The tree merge does not improve the high-cardinality plain groupings at this size. q34 changes from 0.193 seconds to 0.226 seconds, while q35 changes from 0.197 seconds to 0.204 seconds. These queries have nine Parquet row groups and therefore at most nine aggregate instances in this dataset. Their result confirms that rearranging the merge does not remove its work or the duplicate tables.

The next change must partition before accumulation. A worker should hash each group key once, send the row to a partition selected from the high hash bits, and let one task own that partition through aggregation and spilling. Parallel merge remains useful for plans that cannot use the exchange, but it is not the final high-cardinality architecture.

Every deterministic answer retest matched DuckDB.
