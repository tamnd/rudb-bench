# Radix aggregate table follow-up

This follow-up measured three changes against the 1 million row ClickBench workload. Every engine and query ran in a fresh process. Each table reports the sum of 43 per-query medians from five hot runs, the sum of median child CPU time, and the largest per-query peak RSS. Deterministic result checks matched DuckDB for all audited queries.

## Current main

The repository advanced from the earlier balanced-fetch baseline while these experiments were running. A fresh audit of current main was required before making another performance decision.

| Engine | 43-query median sum | CPU sum | Peak RSS |
| --- | ---: | ---: | ---: |
| DuckDB native | 0.580 s | 3.957 s | 311.79 MiB |
| DuckDB Parquet | 0.838 s | 5.375 s | 643.81 MiB |
| rudb main | 1.390 s | 5.920 s | 282.31 MiB |

The earlier rudb result was 1.757 seconds, 6.891 CPU seconds, and 286.06 MiB. Current main is 20.9 percent faster by query time and uses 14.1 percent less CPU. It is still 2.40 times DuckDB native query time and 1.66 times DuckDB Parquet query time. The requested 10x advantage is not reached.

## Direct selected-row folding

This experiment removed radix partition gathers and passed source row indexes directly into table probing and aggregate scatter kernels. It reduced peak RSS slightly but made source access noncontiguous. The gather already copies each row once across all partitions, not once per partition, and the contiguous partition vectors improve cache locality.

| Engine | 43-query median sum | CPU sum | Peak RSS |
| --- | ---: | ---: | ---: |
| DuckDB native | 0.596 s | 4.237 s | 310.38 MiB |
| DuckDB Parquet | 0.894 s | 5.797 s | 661.87 MiB |
| rudb direct selection | 1.862 s | 7.331 s | 281.81 MiB |

A same-window unchanged rudb baseline measured 1.743 seconds, 6.823 CPU seconds, and 286.91 MiB. Direct selection was rejected because the 5.1 MiB peak reduction did not justify 6.8 percent more query time and 7.4 percent more CPU.

## Thirty-two radix partitions

This experiment matched the partition count to the host's 32 workers. Sixteen partitions already expose enough concurrent owners for this workload. The extra partitions add exchange and table overhead.

| Engine | 43-query median sum | CPU sum | Peak RSS |
| --- | ---: | ---: | ---: |
| DuckDB native | 0.555 s | 3.848 s | 307.99 MiB |
| DuckDB Parquet | 0.831 s | 5.280 s | 646.99 MiB |
| rudb with 32 partitions | 1.774 s | 6.964 s | 288.64 MiB |

The same-window 16-partition rudb baseline was 1.743 seconds, 6.823 CPU seconds, and 286.91 MiB. The 32-partition version was rejected on all three measures.

## Compact string-key offsets

Each string group key stored a machine-word end offset. Radix partitions bound their payload independently, so a 32-bit offset covers 4 GiB inside each partition and halves the index width. The implementation returns a memory error before exceeding that bound.

| Engine | 43-query median sum | CPU sum | Peak RSS |
| --- | ---: | ---: | ---: |
| DuckDB native | 0.597 s | 4.136 s | 312.53 MiB |
| DuckDB Parquet | 0.877 s | 5.516 s | 653.07 MiB |
| rudb with compact offsets | 1.417 s | 6.140 s | 282.48 MiB |

Host load was higher during this run, as shown by both DuckDB controls. The targeted memory comparisons remain stable and improve on the string-key queries.

| Query | Main RSS | Compact offset RSS | Change |
| --- | ---: | ---: | ---: |
| q17 | 92.5 MiB | 88.8 MiB | -4.0% |
| q19 | 117.5 MiB | 112.9 MiB | -3.9% |
| q33 | 169.8 MiB | 163.1 MiB | -3.9% |
| q34 | 212.7 MiB | 210.0 MiB | -1.3% |

This change was merged as rudb PR #579. The suite peak remains q23. Its `COUNT(DISTINCT UserID)` and parallel scan lifetime need separate measurement before changing the distinct table layout.
