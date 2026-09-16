# Radix BIGINT distinct count

Q5 is `SELECT COUNT(DISTINCT UserID) FROM hits`. The optimizer previously turned it into a grouping by `UserID` and an ungrouped count over the grouped rows. At one million rows that plan built and merged general group tables, materialized 898,913 rows, and scanned the intermediate chunks to produce one integer.

The new path keeps the original aggregate. Scan workers write 16-byte hash and integer records into sixteen radix buffers. Equal integers always reach the same owner. Each owner builds one compact open-addressed set, returns its distinct count, and releases its input without constructing result rows. Nulls are skipped before the exchange, and packed native integers stay encoded. Finalization uses one worker below 65,536 input rows and scales to sixteen workers, which keeps the small ladder from paying for threads it cannot use.

The audit used the same strided ClickBench data, DuckDB binary, native files, source Parquet files, six fresh processes per query, child RSS measurement, and deterministic verifier as the preceding reports. The 1k and 10k sizes ran first. All 43 queries completed in DuckDB native, rudb native, DuckDB Parquet, and rudb Parquet at every size. Every deterministic rerun matched DuckDB.

## Small sizes

| Rows | Mode | DuckDB query total | rudb query total | Time comparison | DuckDB peak RSS | rudb peak RSS | Memory comparison |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | Native | 126.00 ms | 15.38 ms | rudb 8.19x faster | 53.45 MiB | 8.23 MiB | rudb 6.49x smaller |
| 1k | Parquet | 156.00 ms | 24.87 ms | rudb 6.27x faster | 53.74 MiB | 8.48 MiB | rudb 6.34x smaller |
| 10k | Native | 145.00 ms | 30.79 ms | rudb 4.71x faster | 53.70 MiB | 10.00 MiB | rudb 5.37x smaller |
| 10k | Parquet | 200.00 ms | 64.94 ms | rudb 3.08x faster | 57.80 MiB | 11.71 MiB | rudb 4.94x smaller |

Q5 itself takes 0.20 ms in rudb native at 1k and 0.40 ms at 10k. The owner count threshold removed the 0.3 ms thread startup cost measured in the first implementation.

## One million rows

| Mode | DuckDB query total | rudb query total | Time comparison | DuckDB peak RSS | rudb peak RSS | Memory comparison |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Native | 538.00 ms | 619.21 ms | DuckDB 1.15x faster | 305.45 MiB | 168.69 MiB | rudb 1.81x smaller |
| Parquet | 1,578.00 ms | 834.91 ms | rudb 1.89x faster | 361.61 MiB | 155.98 MiB | rudb 2.32x smaller |

The preceding rudb native total was 639.96 ms, so the new audit is 20.75 ms lower. Rudb Parquet changes from 838.07 ms to 834.91 ms. Only Q5 uses the new path. Differences in the other queries are run variance, and the Q5 result below is the direct measure of the change.

## Q5

| Mode | Engine | Query time | Peak RSS | Comparison with DuckDB |
| --- | --- | ---: | ---: | --- |
| Native | DuckDB | 9.00 ms | 95.93 MiB | reference |
| Native | rudb before | 23.08 ms | 70.73 MiB | 2.56x slower, 1.36x smaller |
| Native | rudb after | 9.80 ms | 61.21 MiB | 1.09x slower, 1.57x smaller |
| Parquet | DuckDB | 31.00 ms | 130.09 MiB | reference |
| Parquet | rudb before | 22.06 ms | 100.70 MiB | 1.41x faster, 1.29x smaller |
| Parquet | rudb after | 16.68 ms | 52.54 MiB | 1.86x faster, 2.48x smaller |

Native Q5 is 2.35 times faster than the preceding rudb build and its process peak falls by 9.52 MiB. It is within 0.80 ms of DuckDB. The Parquet path is 1.32 times faster than the preceding rudb build, and its process peak falls by 48.16 MiB.

The rewrite was the root cause rather than a slow instruction inside the old hash table. Removing the intermediate relation accounts for most of the gain. The remaining native suite gap is spread across other query shapes. Q19 is the clearest next aggregation target: it groups by one BIGINT, one derived integer minute, and one string, then orders by count.
