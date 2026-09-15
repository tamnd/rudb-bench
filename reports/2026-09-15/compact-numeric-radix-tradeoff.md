# Compact numeric groups and radix contention

After string keys moved to shared radix partitions, ClickBench query 33 set rudb's one million row suite peak. It groups `WatchID` and `ClientIP` with a packed count, sum and average state. Worker-local tables duplicate those groups before merging, so two trials routed packed numeric groups to shared partitions. The second trial increased the radix fanout from 16 to 64 partitions. Neither source change was merged.

All three builds include the shared-string behavior from rudb PR 645. Each ran all 43 queries for five hot repetitions in fresh processes on one Linux host. DuckDB native used a preloaded table. DuckDB Parquet and rudb read the same 8,192-row-group Parquet file per query. Time and child CPU sum per-query medians; peak RSS is the largest child resident maximum. Correctness retests matched in both trials. These are sample measurements, not official ClickBench scores.

| 1m rows | Engine and run | Sum of median query time (s) | Sum of median child CPU (s) | Peak child RSS (MiB) |
| --- | --- | ---: | ---: | ---: |
| Worker-local numeric | DuckDB native | 0.539 | 3.828 | 310.9 |
| Worker-local numeric | DuckDB Parquet | 1.602 | 8.047 | 375.2 |
| Worker-local numeric | rudb | 0.9422 | 5.238 | 235.2 |
| Shared numeric, 16 partitions | DuckDB native | 0.550 | 3.826 | 311.8 |
| Shared numeric, 16 partitions | DuckDB Parquet | 1.636 | 8.373 | 360.4 |
| Shared numeric, 16 partitions | rudb | 0.9524 | 5.198 | 170.3 |
| Shared numeric, 64 partitions | DuckDB native | 0.540 | 3.836 | 310.8 |
| Shared numeric, 64 partitions | DuckDB Parquet | 1.613 | 8.191 | 365.7 |
| Shared numeric, 64 partitions | rudb | 1.0256 | 5.730 | 214.9 |

| Query 33, 1m rows | Engine and run | Median query time (ms) | Median child CPU (ms) | Peak child RSS (MiB) |
| --- | --- | ---: | ---: | ---: |
| Worker-local numeric | DuckDB native | 19.0 | 179.9 | 216.0 |
| Worker-local numeric | rudb | 49.0 | 334.1 | 235.2 |
| Shared numeric, 16 partitions | DuckDB native | 19.0 | 177.0 | 215.8 |
| Shared numeric, 16 partitions | rudb | 56.2 | 308.8 | 144.7 |
| Shared numeric, 64 partitions | DuckDB native | 19.0 | 184.0 | 216.7 |
| Shared numeric, 64 partitions | rudb | 62.2 | 334.0 | 153.5 |

Shared numeric groups with 16 partitions cut the suite peak by 28 percent but increased query 33's elapsed time by 15 percent and the suite time sum by one percent. Query 33's CPU fell while its elapsed time rose, which is consistent with workers waiting to fold into shared tables. At 64 partitions, both time and RSS regressed: each partition carries its own table and buffers, and routing groups through more partitions adds work. Query 34 rose from 56.7 to 74.8 ms and query 35 from 57.2 to 71.8 ms in that trial.

The current worker-local numeric path remains the better time choice. A bounded radix exchange with a single owner for each partition could remove duplicate group state without locking a shared table on each worker fold. It needs a pipeline-level handoff and a measured implementation. The present suite is still 1.75 times DuckDB native's query time and uses 76 percent of its peak RSS, far from the requested 10 times faster and 10 times smaller target.
