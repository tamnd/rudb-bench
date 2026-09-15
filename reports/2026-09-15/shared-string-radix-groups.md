# Shared radix tables for string group keys

ClickBench queries 34 and 35 group one million rows by URL. Worker-local radix tables retain each string key in several tables until the final merge. The candidate sends a group with a `VARCHAR` key to shared radix partitions when partitioning starts. The tables store one packed copy of each URL key and skip the later merge of worker-local copies. Numeric group keys keep the local path.

The two rudb binaries use main revision `72f2301` with the same packed numeric aggregate state. Only the string-key routing differs. All 43 queries ran five hot repetitions in fresh processes on one Linux host using 8,192-row-group Parquet files. DuckDB native read a preloaded table. DuckDB Parquet and rudb read the same file for each query. Query time and child CPU are sums of per-query medians; peak RSS is the largest child maximum resident set. DuckDB's query timer has millisecond resolution, while rudb's has finer resolution. These are sample measurements, not official ClickBench scores.

| 1m rows | Engine | Sum of median query time (s) | Sum of median child CPU (s) | Peak child RSS (MiB) |
| --- | --- | ---: | ---: | ---: |
| Local-string run | DuckDB native | 0.548 | 3.829 | 311.0 |
| Local-string run | DuckDB Parquet | 1.622 | 8.112 | 362.4 |
| Local-string run | rudb | 0.9652 | 5.437 | 258.6 |
| Shared-string run | DuckDB native | 0.539 | 3.828 | 310.9 |
| Shared-string run | DuckDB Parquet | 1.602 | 8.047 | 375.2 |
| Shared-string run | rudb | 0.9422 | 5.238 | 235.2 |

| Query, 1m rows | Engine and run | Median query time (ms) | Median child CPU (ms) | Peak child RSS (MiB) |
| --- | --- | ---: | ---: | ---: |
| 34 | DuckDB native, local run | 26.0 | 204.2 | 305.7 |
| 34 | rudb local | 78.7 | 482.8 | 258.6 |
| 34 | DuckDB native, shared run | 26.0 | 197.8 | 302.0 |
| 34 | rudb shared | 56.7 | 375.4 | 171.2 |
| 35 | DuckDB native, local run | 28.0 | 212.1 | 311.0 |
| 35 | rudb local | 74.9 | 464.9 | 258.3 |
| 35 | DuckDB native, shared run | 27.0 | 222.9 | 310.8 |
| 35 | rudb shared | 57.2 | 375.0 | 168.4 |

Query 34's median time fell 28 percent and its peak RSS fell 34 percent. Query 35's time fell 24 percent and its peak RSS fell 35 percent. The suite query time sum fell 2 percent, child CPU fell 4 percent, and peak RSS fell 9 percent. Query 33 now sets rudb's suite peak, because numeric keys still use worker-local tables.

The candidate also completed the smaller sizes with all 43 queries and no unresolved deterministic retests.

| Size | Engine | Sum of median query time (s) | Sum of median child CPU (s) | Peak child RSS (MiB) |
| --- | --- | ---: | ---: | ---: |
| 1k | DuckDB native | 0.125 | 1.209 | 54.8 |
| 1k | DuckDB Parquet | 0.149 | 1.260 | 54.7 |
| 1k | rudb shared | 0.0231 | 0.0606 | 6.3 |
| 10k | DuckDB native | 0.145 | 1.248 | 54.9 |
| 10k | DuckDB Parquet | 0.200 | 1.396 | 58.9 |
| 10k | rudb shared | 0.0701 | 0.1264 | 9.6 |
| 100k | DuckDB native | 0.319 | 1.521 | 82.8 |
| 100k | DuckDB Parquet | 0.369 | 2.124 | 114.6 |
| 100k | rudb shared | 0.1969 | 0.7602 | 42.3 |

The shared-string build remains 1.75 times DuckDB native's 1m suite query time and uses 76 percent of its peak RSS. The requested 10 times faster and 10 times smaller target is still unmet. Query 33's numeric group state and Parquet decoding are the next large costs to investigate.
