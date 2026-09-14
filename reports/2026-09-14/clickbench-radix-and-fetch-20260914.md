# Radix aggregation and wide fetch follow-up

This follow-up measures rudb after shared radix aggregate tables, hash reuse, parallel wide-row Parquet fetches and compressed-byte balancing. It uses the same gamingpc-wsl host, Snappy Parquet inputs and measurement method as the parallel audit. Each query median comes from five hot executions in fresh processes.

## Size ladder

| Rows | Engine | Completed | Query median sum | CPU sum | Peak RSS |
| ---: | --- | ---: | ---: | ---: | ---: |
| 1k | DuckDB native | 43/43 | 0.124 s | 1.240 s | 56.12 MiB |
| 1k | DuckDB Parquet | 43/43 | 0.157 s | 1.302 s | 55.79 MiB |
| 1k | rudb | 43/43 | 0.032 s | 0.075 s | 6.22 MiB |
| 10k | DuckDB native | 43/43 | 0.144 s | 1.274 s | 56.85 MiB |
| 10k | DuckDB Parquet | 43/43 | 0.188 s | 1.365 s | 57.69 MiB |
| 10k | rudb | 43/43 | 0.083 s | 0.129 s | 8.65 MiB |
| 100k | DuckDB native | 43/43 | 0.328 s | 1.527 s | 86.32 MiB |
| 100k | DuckDB Parquet | 43/43 | 0.455 s | 1.969 s | 139.02 MiB |
| 100k | rudb | 43/43 | 0.577 s | 0.708 s | 77.58 MiB |
| 1m | DuckDB native | 43/43 | 0.557 s | 3.838 s | 313.61 MiB |
| 1m | DuckDB Parquet | 43/43 | 0.827 s | 5.207 s | 649.88 MiB |
| 1m | rudb | 43/43 | 1.757 s | 6.891 s | 286.06 MiB |

All deterministic tie-breaker retests matched DuckDB.

## Change history at 1 million rows

| rudb implementation | Query median sum | Peak RSS | Change from parallel baseline |
| --- | ---: | ---: | ---: |
| Per-worker aggregate tables and serial merge | 2.997 s | 248.38 MiB | baseline |
| Shared radix tables and hash reuse | 2.090 s | 286.43 MiB | 30.3% faster |
| Parallel wide Parquet fetch | 1.791 s | 287.39 MiB | 40.2% faster |
| Byte-balanced wide Parquet fetch | 1.757 s | 286.06 MiB | 41.4% faster |

The radix exchange removes the algorithmic cost of keeping the same group in several worker tables and probing those duplicate groups again during a final merge. Wide fetch parallelism addresses a separate serial phase exposed by query 24. That query selects ten row ordinals with a narrow scan and then reads 105 result columns. Its fetch phase previously decoded the selected column chunks in one loop.

| Query 24 engine | Query time |
| --- | ---: |
| DuckDB native | 0.039 s |
| DuckDB Parquet | 0.087 s |
| rudb before parallel fetch | 0.445 s |
| rudb after parallel fetch | 0.178 s |
| rudb after byte balancing | 0.173 s |

The remaining fetch limit is inside a column. ClickBench has nine row groups, and the largest compressed column chunk is about 8 MiB. Assigning whole columns cannot divide that work, so page-level tasks are required to use more cores once the other columns finish.

The target is not reached at large sizes. At 1 million rows rudb is 3.2 times slower than DuckDB native and uses 8.8 percent less peak RSS. The 1k suite is 3.9 times faster and uses 9.0 times less peak RSS, while the 10k suite is 1.7 times faster and uses 6.6 times less peak RSS. Growth between 10k and 1m remains the main problem.
