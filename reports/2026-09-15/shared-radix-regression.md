# Shared radix table regression on rudb 0.3.16

The worker-local aggregate tables added in rudb PR 603 duplicate high-cardinality groups and lose counts during the final merge. On the 1 million row ClickBench sample with 8,192-row Parquet groups, disabling worker-local tables reduced the suite peak resident memory from 297 to 215 MiB and restored correct results for queries 14, 15 and 35. Query 34 fell from 95 to 63 ms. The full suite remains slower than DuckDB native storage.

The audit ran all 43 queries five hot times in fresh processes on the same Linux host. The table sums each query's median engine time and median child CPU time, and takes the maximum child peak RSS. DuckDB native used a table loaded once before timing; DuckDB Parquet and rudb read the same re-encoded Parquet file per query. The native DuckDB and Parquet controls were rerun alongside each rudb binary. These are sample measurements, not official ClickBench scores.

| Size | Engine and build | Sum of median query time (s) | Sum of median child CPU (s) | Peak child RSS (MiB) |
| --- | --- | ---: | ---: | ---: |
| 1k | DuckDB native | 0.121 | 1.206 | 56.8 |
| 1k | DuckDB Parquet | 0.150 | 1.259 | 56.4 |
| 1k | rudb shared | 0.0229 | 0.0525 | 6.2 |
| 10k | DuckDB native | 0.146 | 1.256 | 56.8 |
| 10k | DuckDB Parquet | 0.196 | 1.394 | 60.4 |
| 10k | rudb shared | 0.0757 | 0.1241 | 9.7 |
| 100k | DuckDB native | 0.327 | 1.543 | 85.3 |
| 100k | DuckDB Parquet | 0.380 | 2.137 | 116.3 |
| 100k | rudb shared | 0.2231 | 0.8175 | 43.1 |
| 1m, baseline run | DuckDB native | 0.537 | 3.860 | 311.3 |
| 1m, baseline run | DuckDB Parquet | 1.607 | 8.033 | 369.8 |
| 1m, baseline run | rudb worker-local | 1.0453 | 7.139 | 296.9 |
| 1m, shared run | DuckDB native | 0.557 | 3.874 | 312.8 |
| 1m, shared run | DuckDB Parquet | 1.646 | 8.229 | 366.6 |
| 1m, shared run | rudb shared | 1.0323 | 7.033 | 214.7 |

| Query on 1m rows | rudb worker-local time (ms) | rudb shared time (ms) | rudb worker-local RSS (MiB) | rudb shared RSS (MiB) |
| --- | ---: | ---: | ---: | ---: |
| 14 | 29.2 | 28.4 | 83.8 | 77.7 |
| 15 | 23.3 | 24.1 | 64.5 | 62.1 |
| 23 | 29.5 | 29.0 | 126.0 | 129.8 |
| 33 | 57.1 | 64.5 | 275.5 | 186.8 |
| 34 | 95.2 | 63.3 | 293.0 | 214.7 |
| 35 | 81.3 | 64.9 | 296.9 | 206.5 |

The original worker-local run returned wrong aggregate counts after adding deterministic tie breakers to queries 14, 15 and 35. For example, query 14 returned 629 distinct users for the top phrase while DuckDB returned 667. Query 35 returned 16,102 visits for `http://kinopoisk.ru` while DuckDB returned 16,109. The shared run had no unresolved answer differences on the 1k, 10k, 100k or 1m audits. Original SQL stayed unchanged in timed runs; tie breakers were used only in untimed correctness retests.

The shared path avoids storing a separate hash table for every worker and avoids merging those copies at the end. The 1m suite still takes 1.85 times the DuckDB native query time and 68.6 percent of its peak RSS. This fixes the regression but does not meet the 10 times faster and 10 times smaller goal. Row-group size and aggregate table layout remain the next large factors to address.
