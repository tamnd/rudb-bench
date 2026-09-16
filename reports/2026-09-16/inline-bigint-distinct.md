# Inline BIGINT distinct state

Q5, Q9, Q10, and Q14 all count distinct `UserID` values. The aggregate previously created a hash set as soon as a group saw its first value. That is especially expensive for Q14, where most `SearchPhrase` groups contain one row. The query paid for more than one hundred thousand small allocations even though nearly all of those sets held one integer.

The new state has three forms: empty, one inline integer, and a hash set. It creates the set only when a second distinct integer reaches the group. Packed integers from native storage are read directly instead of passing through a general scalar conversion. A parent TopN ordered by one distinct count can also ask each radix partition for its exact local prefix, with the existing global TopN choosing the final rows.

The audit used the same strided ClickBench samples and four execution modes as the preceding fixed width radix report. Each query ran in a fresh process with six repetitions. The query median excludes the first run, while peak RSS covers the child process. The 1k and 10k audits ran before the 1m audit. All 43 queries completed in every mode. The deterministic verifier matched DuckDB, including Q14.

## Small sizes

The optimization does not materially change the small runs because they contain few distinct groups. These results provide the DuckDB comparison and show that fixed process costs still dominate at these sizes.

| Rows | Mode | DuckDB query total | rudb query total | Time comparison | DuckDB peak RSS | rudb peak RSS | Memory comparison |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | Native | 118.00 ms | 15.28 ms | rudb 7.72x faster | 54.32 MiB | 8.18 MiB | rudb 6.64x smaller |
| 1k | Parquet | 152.00 ms | 24.52 ms | rudb 6.20x faster | 53.93 MiB | 8.51 MiB | rudb 6.34x smaller |
| 10k | Native | 144.00 ms | 31.10 ms | rudb 4.63x faster | 54.25 MiB | 10.19 MiB | rudb 5.32x smaller |
| 10k | Parquet | 197.00 ms | 65.52 ms | rudb 3.01x faster | 58.20 MiB | 11.61 MiB | rudb 5.01x smaller |

## One million rows

| Mode | DuckDB query total | rudb query total | Time comparison | DuckDB peak RSS | rudb peak RSS | Memory comparison |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Native | 543.00 ms | 639.96 ms | DuckDB 1.18x faster | 308.95 MiB | 168.73 MiB | rudb 1.83x smaller |
| Parquet | 1,605.00 ms | 838.07 ms | rudb 1.92x faster | 365.61 MiB | 158.11 MiB | rudb 2.31x smaller |

The preceding rudb native total was 678.70 ms, so this change removes 38.74 ms from the suite. Rudb Parquet improves from 909.75 ms to 838.07 ms. The peak belongs to another query and remains unchanged. The native result remains behind DuckDB, and the 10x time and memory targets have not been reached at this size.

## Distinct queries

| Query | DuckDB native | rudb before | rudb after | Native change | rudb Parquet before | rudb Parquet after |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Q5 | 9.00 ms | 26.09 ms | 23.08 ms | 11.5% lower | 23.50 ms | 22.06 ms |
| Q9 | 13.00 ms | 31.30 ms | 25.98 ms | 17.0% lower | 33.25 ms | 31.34 ms |
| Q10 | 15.00 ms | 36.49 ms | 32.68 ms | 10.4% lower | 39.75 ms | 37.25 ms |
| Q14 | 12.00 ms | 31.57 ms | 21.15 ms | 33.0% lower | 26.60 ms | 21.08 ms |

Q14 benefits most because it creates the most singleton groups. Its native peak RSS falls from 65.48 MiB to 58.39 MiB. Profiles before the change placed 45.00 ms of cumulative worker time in aggregate folding and 28.79 ms in result emission. After inline state, folding fell to 38.35 ms and emission to 9.10 ms. The wall time fell from 31.57 ms to 21.15 ms.

Q5 still builds a generic grouped table whose only purpose is to deduplicate one integer column, then runs a second ungrouped aggregate to count the emitted keys. Its remaining 14.08 ms gap against DuckDB is an architectural cost. A fixed one-key radix distinct exchange can assign each integer to one owner and remove the worker tables, generic merge, and intermediate group chunks.

Q19 remains a separate cost. It groups by a BIGINT, a derived minute, and a string before ordering by count. Its rudb native median is 40.04 ms against DuckDB's 18.00 ms. The next grouped exchange should support that three-key shape rather than adding more special cases to the generic hash table.
