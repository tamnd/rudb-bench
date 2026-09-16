# Fixed width radix exchange for high cardinality aggregates

Q32 and Q33 group by `WatchID` and `ClientIP`, then order by count and keep ten rows. At one million rows, nearly every key pair is unique. The previous aggregate built a compact hash table in every pipeline worker, divided those tables into radix partitions, and probed every group again while merging worker tables. Local TopN prefix emission bounded the output, but it did not remove that second generic hash pass.

The new path exchanges fixed width records before aggregation. Each scan worker writes 24-byte records into sixteen buffers selected by the high bits of the group hash. Combining a worker takes one lock per nonempty partition and moves the buffered rows in batches. Finalization gives each partition one owner, which builds one fixed open-addressed table and emits at most ten candidates. Equal keys always have the same hash and therefore reach the same owner. The final global TopN over the 160 candidates remains exact.

Validity is stored in a sidecar that is created only if a partition sees a null. Packed native integers are read as `base + code`, and the fixed exchange hashes keys while it writes the record. This avoids the general scalar path, which previously allocated a one-value buffer when it read a packed integer. Query memory accounting includes exchange capacity, owner buckets, aggregate states, and result chunks.

The audit used the same one million row strided ClickBench sample, DuckDB binary, native files, source Parquet files, six fresh processes per query, child RSS measurement, and deterministic verifier as the preceding audit. The 1k and 10k sizes were run first. All 43 queries completed in DuckDB native, rudb native, DuckDB Parquet, and rudb Parquet at every size. Every deterministic rerun matched DuckDB, including Q32 and Q33.

## Small sizes

| Rows | Mode | DuckDB query total | rudb query total | Time comparison | DuckDB peak RSS | rudb peak RSS | Memory comparison |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | Native | 121.00 ms | 15.18 ms | rudb 7.97x faster | 57.64 MiB | 8.20 MiB | rudb 7.03x smaller |
| 1k | Parquet | 156.00 ms | 24.46 ms | rudb 6.38x faster | 57.07 MiB | 8.49 MiB | rudb 6.72x smaller |
| 10k | Native | 147.00 ms | 30.83 ms | rudb 4.77x faster | 57.60 MiB | 10.01 MiB | rudb 5.75x smaller |
| 10k | Parquet | 201.00 ms | 65.80 ms | rudb 3.05x faster | 60.96 MiB | 11.51 MiB | rudb 5.30x smaller |

The 1k native suite is close to the 10x target but has not reached it. The ratios fall with row count, so fixed startup cost is not the remaining explanation.

## One million rows

| Mode | DuckDB query total | rudb query total | Time comparison | DuckDB peak RSS | rudb peak RSS | Memory comparison |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Native | 570.00 ms | 678.70 ms | DuckDB 1.19x faster | 309.28 MiB | 168.49 MiB | rudb 1.84x smaller |
| Parquet | 1,669.00 ms | 909.75 ms | rudb 1.83x faster | 366.11 MiB | 153.71 MiB | rudb 2.38x smaller |

The previous rudb native total was 726.93 ms, so the exchange removes 48.23 ms from the suite. The suite peak is set by another query and remains essentially unchanged. The native suite still trails DuckDB, and neither time nor memory has reached the 10x target at this size.

## Q32 and Q33

| Mode | Query | DuckDB | rudb before | rudb after | Time change | DuckDB RSS | rudb before RSS | rudb after RSS |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Native | Q32 | 12.00 ms | 16.50 ms | 9.21 ms | 44.2% lower | 104.01 MiB | 45.71 MiB | 39.55 MiB |
| Native | Q33 | 20.00 ms | 51.37 ms | 21.12 ms | 58.9% lower | 217.82 MiB | 124.19 MiB | 106.15 MiB |
| Parquet | Q32 | 39.00 ms | 18.74 ms | 14.02 ms | 25.2% lower | 158.13 MiB | 45.77 MiB | 33.77 MiB |
| Parquet | Q33 | 51.00 ms | 43.61 ms | 25.27 ms | 42.1% lower | 238.00 MiB | 167.62 MiB | 83.84 MiB |

Rudb native is 1.30 times faster than DuckDB native on Q32. Q33 is 1.12 ms slower than DuckDB native and uses 2.05 times less RSS. On Parquet, rudb is 2.78 times faster on Q32 and 2.02 times faster on Q33.

A hot Q33 profile after the change attributes 25.3 through 28.9 ms of cumulative aggregate time to scatter, 9.8 through 14.4 ms to owner table construction, and 4.8 through 5.1 ms to candidate selection and emission. Query execution is 17.0 through 21.8 ms because the stages run across workers. The second generic table merge is gone. Scatter is now the largest part of this aggregate.

## Remaining native gap

The largest positive time differences against DuckDB native now come from other query shapes.

| Query | DuckDB native | rudb native | rudb excess |
| ---: | ---: | ---: | ---: |
| Q14 | 12.00 ms | 31.57 ms | 19.57 ms |
| Q10 | 17.00 ms | 36.49 ms | 19.49 ms |
| Q19 | 20.00 ms | 38.64 ms | 18.64 ms |
| Q9 | 13.00 ms | 31.30 ms | 18.30 ms |
| Q5 | 10.00 ms | 26.09 ms | 16.09 ms |

These five queries account for 92.09 ms of the native gap. The next optimization should profile their shared scan, filter, and grouped string work rather than continue changing Q32 and Q33. The radix exchange has removed the merge architecture that dominated Q33 and brought that query to native parity.
