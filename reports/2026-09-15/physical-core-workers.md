# Physical-core worker count on ClickBench

On the 32-logical-CPU gaming PC, rudb used 32 aggregate workers by default even though Linux reports 16 physical cores with two hardware threads each. High-cardinality grouping gained no query-time throughput from the second hardware thread. It increased aggregate scratch space, child CPU time and peak resident memory. A rudb build that defaults to the 16 physical cores cut the 1 million row suite's peak RSS from 214 to 175 MiB and child CPU from 6.85 to 5.72 seconds. Query time changed from 1.024 to 1.008 seconds.

Both rudb builds came from the same rudb `main` revision, `d2c98fe`, with only the core-count default changed. All 43 ClickBench queries ran five hot repetitions in fresh processes against the same 8,192-row-group Parquet files. DuckDB native and DuckDB Parquet were measured in both runs as controls. The table sums each query's median engine time and child CPU time. Peak RSS is the maximum child resident memory, not an allocation counter. DuckDB native opened a table loaded once before timing. DuckDB Parquet and rudb decoded the same source file per query. These are sample measurements, not official ClickBench scores.

| Size | Engine and run | Sum of median query time (s) | Sum of median child CPU (s) | Peak child RSS (MiB) |
| --- | --- | ---: | ---: | ---: |
| 1k, physical run | DuckDB native | 0.125 | 1.239 | 56.8 |
| 1k, physical run | DuckDB Parquet | 0.159 | 1.299 | 56.7 |
| 1k, physical run | rudb physical | 0.0228 | 0.0588 | 6.3 |
| 10k, physical run | DuckDB native | 0.142 | 1.258 | 56.6 |
| 10k, physical run | DuckDB Parquet | 0.194 | 1.392 | 60.2 |
| 10k, physical run | rudb physical | 0.0749 | 0.1321 | 9.7 |
| 100k, physical run | DuckDB native | 0.321 | 1.533 | 83.8 |
| 100k, physical run | DuckDB Parquet | 0.375 | 2.134 | 116.6 |
| 100k, physical run | rudb physical | 0.2153 | 0.8007 | 43.4 |
| 1m, logical run | DuckDB native | 0.547 | 3.832 | 313.5 |
| 1m, logical run | DuckDB Parquet | 1.653 | 8.068 | 363.3 |
| 1m, logical run | rudb 32 logical workers | 1.0239 | 6.854 | 214.3 |
| 1m, physical run | DuckDB native | 0.537 | 3.821 | 310.8 |
| 1m, physical run | DuckDB Parquet | 1.624 | 8.176 | 360.4 |
| 1m, physical run | rudb 16 physical workers | 1.0076 | 5.723 | 175.1 |

The diagnostic thread sweep used the shared-radix build on the same 1 million row file. Each entry is the median of five fresh-process runs for that query. RSS is the largest peak among those runs.

| Query | Workers | Median query time (ms) | Median child CPU (ms) | Largest child RSS (MiB) |
| --- | ---: | ---: | ---: | ---: |
| 33, WatchID and ClientIP | 16 | 63.0 | 376.9 | 171.9 |
| 33, WatchID and ClientIP | 32 | 64.1 | 461.3 | 189.6 |
| 34, URL | 16 | 60.7 | 425.7 | 172.8 |
| 34, URL | 32 | 61.0 | 519.1 | 205.4 |
| 35, constant and URL | 16 | 62.3 | 434.8 | 173.0 |
| 35, constant and URL | 32 | 61.2 | 526.4 | 204.2 |

No deterministic correctness retest was unresolved in either full 1 million row audit. The physical-core build also had no unresolved retest at 1k, 10k or 100k rows. The change reduces worker copies and contention, but the full 1 million row query sum is still 1.88 times DuckDB native and its peak RSS is 56 percent of DuckDB native. It does not meet the requested 10 times faster and 10 times smaller goal. The 8,192-row Parquet layout and the large per-group aggregate state remain the dominant costs.
