# Sparse overflow storage for compact numeric groups

ClickBench query 33 groups almost every input row by `WatchID` and `ClientIP` while computing `COUNT(*)`, `SUM(IsRefresh)` and `AVG(ResolutionWidth)`. The compact group state previously held two 128-bit exact totals in every group. SMALLINT totals normally fit in 64 bits. The candidate holds two 64-bit totals in each group and allocates a sparse wide entry only when either total crosses the 64-bit range. Null SUM and AVG inputs still count independently, and a focused test covers exact totals above and below the 64-bit range through both updates and merges.

Both rudb builds include the shared-string radix behavior from PR 645 and differ only in numeric state layout. All 43 ClickBench queries ran five hot repetitions in fresh processes on the same Linux host. DuckDB native read a preloaded table. DuckDB Parquet and rudb read the same Parquet file per query. Query time and child CPU sum each query's median; peak RSS is the largest child maximum resident set. The 1k to 1m files use 8,192-row Parquet groups. The 10m file is the existing Snappy sample with 9,999,750 rows. These are sample measurements, not official ClickBench scores.

| 1m rows | Engine and run | Sum of median query time (s) | Sum of median child CPU (s) | Peak child RSS (MiB) |
| --- | --- | ---: | ---: | ---: |
| Wide-state run | DuckDB native | 0.539 | 3.828 | 310.9 |
| Wide-state run | DuckDB Parquet | 1.602 | 8.047 | 375.2 |
| Wide-state run | rudb | 0.9422 | 5.238 | 235.2 |
| Sparse-state run | DuckDB native | 0.542 | 3.813 | 310.8 |
| Sparse-state run | DuckDB Parquet | 1.617 | 8.182 | 383.4 |
| Sparse-state run | rudb | 0.9423 | 5.170 | 190.9 |

| 10m rows | Engine and run | Sum of median query time (s) | Sum of median child CPU (s) | Peak child RSS (MiB) |
| --- | --- | ---: | ---: | ---: |
| Wide-state run | DuckDB native | 2.982 | 37.834 | 1,659.8 |
| Wide-state run | DuckDB Parquet | 4.047 | 45.123 | 2,738.2 |
| Wide-state run | rudb | 5.8244 | 49.218 | 1,633.8 |
| Sparse-state run | DuckDB native | 2.919 | 37.672 | 1,649.6 |
| Sparse-state run | DuckDB Parquet | 4.015 | 45.021 | 2,740.3 |
| Sparse-state run | rudb | 5.7398 | 48.815 | 1,364.2 |

| Query 33 | Engine and run | Median query time (ms) | Median child CPU (ms) | Peak child RSS (MiB) |
| --- | --- | ---: | ---: | ---: |
| 1m | DuckDB native, wide run | 19.0 | 179.9 | 216.0 |
| 1m | rudb wide | 49.0 | 334.1 | 235.2 |
| 1m | DuckDB native, sparse run | 19.0 | 174.0 | 214.8 |
| 1m | rudb sparse | 47.0 | 301.4 | 190.9 |
| 10m | DuckDB native, wide run | 143.0 | 3,078.4 | 1,274.8 |
| 10m | rudb wide | 441.1 | 4,340.5 | 1,633.8 |
| 10m | DuckDB native, sparse run | 142.0 | 3,109.9 | 1,257.7 |
| 10m | rudb sparse | 417.4 | 4,108.3 | 1,364.2 |

At 1m rows the suite time is unchanged within measurement variation and peak RSS fell 19 percent. At 10m rows the suite time fell 1 percent and peak RSS fell 17 percent. Query 33's 10m median time fell 5 percent and its peak RSS fell 17 percent. The sparse build completed all 43 timed queries at every size. Deterministic correctness retests matched at 1k, 10k, 100k and 1m rows. At 10m, query 29 remains unresolved on both unchanged rudb main and the sparse build; it groups a regular-expression result and does not use the compact numeric state.

| Smaller size | Engine | Sum of median query time (s) | Sum of median child CPU (s) | Peak child RSS (MiB) |
| --- | --- | ---: | ---: | ---: |
| 1k | DuckDB native | 0.123 | 1.226 | 54.0 |
| 1k | DuckDB Parquet | 0.159 | 1.286 | 54.2 |
| 1k | rudb sparse | 0.0233 | 0.0603 | 6.4 |
| 10k | DuckDB native | 0.148 | 1.259 | 54.2 |
| 10k | DuckDB Parquet | 0.196 | 1.408 | 57.7 |
| 10k | rudb sparse | 0.0705 | 0.1271 | 9.5 |
| 100k | DuckDB native | 0.326 | 1.565 | 83.5 |
| 100k | DuckDB Parquet | 0.374 | 2.167 | 115.3 |
| 100k | rudb sparse | 0.1960 | 0.7429 | 42.6 |

The 10m sparse suite still takes 1.97 times DuckDB native's query time and uses 83 percent of its peak RSS. The requested 10 times faster and 10 times smaller target remains unmet. A rejected bounded radix handoff cut memory but slowed query 33 at 10m because it repeatedly scattered group state. The faster local merge path remains in place, with a smaller per-group state.

Profiling query 33 on the 10m sample showed the aggregate dominated the work. In the unchanged build, aggregate fold and final merge accumulated 2,020 and 2,230 ms across worker threads, while the file scan accumulated 216 ms. The fixed 16k-group handoff trial accumulated 4,940 ms in scatter and raised query 33's median time from 441 to 544 ms. These stage totals overlap across threads and are diagnostic work readings, not process elapsed time. A futex trace on the 1m query counted 751 calls in the local build, 1,646 in a fully shared numeric trial, and 1,046 with fixed 16k handoffs. The trace adds overhead, so its syscall counts are used to locate contention rather than to report benchmark time. Neither handoff trial was merged.
