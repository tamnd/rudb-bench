# Q19 and Q33 completion

ClickBench defines 43 queries. Q19 and Q33 were the last two that the regular rudb benchmark harness marked unavailable. The engine now completes both, so rudb runs 43 of 43 canonical ClickBench queries. There are no canonical Q44 or Q45 queries.

The audit ran the smaller samples first. Every entry below completed all six timed executions. The differential verifier compared deterministic reruns against DuckDB and found no mismatch. Query time is the median time reported by the engine. Peak resident memory is the largest measured child RSS among the timed executions.

## Native storage

| Rows | Query | DuckDB time | rudb time | DuckDB / rudb | DuckDB RSS | rudb RSS | DuckDB / rudb RSS |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | Q19 | 3.000 ms | 0.407 ms | 7.37x | 48.67 MiB | 7.76 MiB | 6.27x |
| 1k | Q33 | 3.000 ms | 0.467 ms | 6.42x | 49.09 MiB | 7.76 MiB | 6.33x |
| 10k | Q19 | 4.000 ms | 1.694 ms | 2.36x | 50.08 MiB | 9.20 MiB | 5.44x |
| 10k | Q33 | 3.000 ms | 2.012 ms | 1.49x | 50.04 MiB | 9.00 MiB | 5.56x |
| 100k | Q19 | 11.000 ms | 8.030 ms | 1.37x | 65.82 MiB | 17.18 MiB | 3.83x |
| 100k | Q33 | 9.000 ms | 18.011 ms | 0.50x | 66.07 MiB | 21.88 MiB | 3.02x |
| 1m | Q19 | 19.000 ms | 42.434 ms | 0.45x | 206.29 MiB | 95.21 MiB | 2.17x |
| 1m | Q33 | 19.000 ms | 58.060 ms | 0.33x | 217.85 MiB | 129.89 MiB | 1.68x |

## Parquet

| Rows | Query | DuckDB time | rudb time | DuckDB / rudb | DuckDB RSS | rudb RSS | DuckDB / rudb RSS |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | Q19 | 3.000 ms | 0.612 ms | 4.90x | 48.07 MiB | 8.00 MiB | 6.01x |
| 1k | Q33 | 4.000 ms | 0.637 ms | 6.28x | 48.70 MiB | 7.75 MiB | 6.28x |
| 10k | Q19 | 5.000 ms | 3.012 ms | 1.66x | 52.49 MiB | 9.49 MiB | 5.53x |
| 10k | Q33 | 4.000 ms | 2.519 ms | 1.59x | 52.40 MiB | 9.25 MiB | 5.66x |
| 100k | Q19 | 11.000 ms | 8.628 ms | 1.27x | 82.32 MiB | 25.28 MiB | 3.26x |
| 100k | Q33 | 12.000 ms | 8.884 ms | 1.35x | 83.13 MiB | 27.82 MiB | 2.99x |
| 1m | Q19 | 62.000 ms | 48.901 ms | 1.27x | 290.70 MiB | 125.79 MiB | 2.31x |
| 1m | Q33 | 54.000 ms | 49.030 ms | 1.10x | 243.76 MiB | 192.54 MiB | 1.27x |

The small inputs confirm that the queries are wired through every storage path. The one million row results expose the remaining scaling problem. Native Q19 is 2.23 times slower than DuckDB and native Q33 is 3.06 times slower. Q33 creates almost one group per row, so its local hash tables, merge, and row materialization dominate the query. The next aggregate change should remove the local table merge through owner partitioned radix exchange.

The harness change removes the stale rudb exclusions for Q19 and Q33 and includes a test that fails if any canonical ClickBench query is omitted for rudb.
