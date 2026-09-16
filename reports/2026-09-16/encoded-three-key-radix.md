# Encoded three-key radix count

Q19 groups by `UserID`, the minute extracted from `EventTime`, and `SearchPhrase`, then orders by count and keeps ten rows. On native storage those keys are a packed BIGINT, a derived BIGINT, and a stable global string code. The generic aggregate did not preserve that shape. It built a table in each worker, scattered the completed groups, probed the same keys into owner tables, and decoded string keys before the local TopN removed nearly all of them.

The new path exchanges 24-byte records before aggregation. A record holds two integers, one string code, and a 32-bit hash. The high hash bits choose one of sixteen owners. Each owner builds one open-addressed table, keeps at most ten local candidates, and decodes only those candidates. A validity sidecar is allocated only if a key is null. The ordinary global TopN still chooses the final ten rows, so the reduction is exact.

The path requires one stable string dictionary. Native storage provides that code space across every row group. Parquet dictionaries are local to pages or row groups, so Parquet keeps the general path. This is a storage contract used by execution rather than a query-specific interpretation of `SearchPhrase`.

The audit used the same strided ClickBench samples, DuckDB binary, native files, source Parquet files, six fresh processes per query, child RSS measurement, and deterministic verifier as the preceding reports. The 1k and 10k sizes ran first. All 43 queries completed in every mode at every size, and every deterministic rerun matched DuckDB.

## Small sizes

| Rows | Mode | DuckDB query total | rudb query total | Time comparison | DuckDB peak RSS | rudb peak RSS | Memory comparison |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | Native | 119.00 ms | 15.07 ms | rudb 7.90x faster | 56.56 MiB | 8.31 MiB | rudb 6.81x smaller |
| 1k | Parquet | 154.00 ms | 24.62 ms | rudb 6.25x faster | 55.82 MiB | 8.51 MiB | rudb 6.56x smaller |
| 10k | Native | 148.00 ms | 30.41 ms | rudb 4.87x faster | 56.38 MiB | 10.01 MiB | rudb 5.63x smaller |
| 10k | Parquet | 199.00 ms | 64.85 ms | rudb 3.07x faster | 59.85 MiB | 11.61 MiB | rudb 5.16x smaller |

Native Q19 is unchanged at 0.35 ms for 1k rows and improves from 1.24 ms to 0.84 ms at 10k rows. Finalization uses one worker below 65,536 exchanged rows, so the small runs do not pay for sixteen finishing threads.

## One million rows

| Mode | DuckDB query total | rudb query total | Time comparison | DuckDB peak RSS | rudb peak RSS | Memory comparison |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Native | 545.00 ms | 614.73 ms | DuckDB 1.13x faster | 311.59 MiB | 168.40 MiB | rudb 1.85x smaller |
| Parquet | 1,621.00 ms | 832.12 ms | rudb 1.95x faster | 396.25 MiB | 155.07 MiB | rudb 2.56x smaller |

Run variance hides the local gain in the suite total, so Q19 is the useful comparison.

| Mode | Engine | Q19 time | Q19 peak RSS | Comparison with DuckDB |
| --- | --- | ---: | ---: | --- |
| Native | DuckDB | 18.00 ms | 199.47 MiB | reference |
| Native | rudb before | 38.81 ms | 93.50 MiB | 2.16x slower, 2.13x smaller |
| Native | rudb after | 20.55 ms | 86.23 MiB | 1.14x slower, 2.31x smaller |
| Parquet | DuckDB | 51.00 ms | 273.50 MiB | reference |
| Parquet | rudb | 41.92 ms | 108.09 MiB | 1.22x faster, 2.53x smaller |

Native Q19 is 1.89 times faster than the preceding rudb build. Earlier profiling attributed about 121 ms of cumulative worker time to generic aggregate folding. The owner tables now take 8.0 ms. Record scatter is the largest aggregate stage at 24.2 ms of cumulative worker time, while the query completes in about 20.6 ms because scan and aggregate work run across four pipeline workers.

Q19 is now within 2.55 ms of DuckDB and uses less than half its process memory. Q9, Q10, and Q14 remain the clearest shared aggregate gap. They group rows while maintaining distinct BIGINT values per group, so they still use the generic group table and per-group distinct state.
