# Grouped distinct radix pairs

Q9 groups by `RegionID`, counts distinct `UserID` values, orders by that count, and keeps ten groups. The general aggregate held one distinct set per group in every scan worker. Combining workers merged the group tables and then walked the arriving sets into the retained sets. A logical `(RegionID, UserID)` pair could therefore be hashed in the group table, in a worker's distinct set, and again while the sets merged.

The new operator is isolated in `group_distinct.rs`. A scan worker writes a 16-byte record containing one 32-bit region, one 64-bit user, and one 32-bit pair hash. The region hash assigns all values for one group to one of sixteen owners. Each owner deduplicates pairs in one flat open-addressed table, counts the surviving pairs in a compact group table, and emits at most ten candidates. The final global TopN still chooses the answer. Null distinct values are skipped, while a null region remains a group.

An earlier 24-byte record also stored the region hash. That layout ran native Q9 in about 18.2 ms and peaked near 87 MiB. Recomputing the cheap region hash during group counting reduced the record to 16 bytes, Q9 to 13.9 ms, and peak RSS to 64.4 MiB.

The audit used the same strided ClickBench samples, DuckDB binary, native files, source Parquet files, six fresh processes per query, child RSS measurement, and deterministic verifier as the preceding reports. The 1k and 10k sizes ran first. All 43 queries completed in every mode at every size, and every deterministic rerun matched DuckDB.

## Small sizes

| Rows | Mode | DuckDB query total | rudb query total | Time comparison | DuckDB peak RSS | rudb peak RSS | Memory comparison |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | Native | 125.00 ms | 15.50 ms | rudb 8.06x faster | 54.04 MiB | 8.23 MiB | rudb 6.57x smaller |
| 1k | Parquet | 157.00 ms | 24.94 ms | rudb 6.29x faster | 53.56 MiB | 8.54 MiB | rudb 6.27x smaller |
| 10k | Native | 147.00 ms | 30.18 ms | rudb 4.87x faster | 53.77 MiB | 10.04 MiB | rudb 5.36x smaller |
| 10k | Parquet | 199.00 ms | 64.71 ms | rudb 3.08x faster | 57.62 MiB | 11.77 MiB | rudb 4.90x smaller |

At 10k rows, native Q9 improves from 0.72 ms to 0.64 ms and Parquet Q9 improves from 1.24 ms to 0.98 ms. At 1k rows the differences are below a tenth of a millisecond.

## One million rows

| Mode | DuckDB query total | rudb query total | Time comparison | DuckDB peak RSS | rudb peak RSS | Memory comparison |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Native | 561.00 ms | 590.66 ms | DuckDB 1.05x faster | 309.27 MiB | 168.26 MiB | rudb 1.84x smaller |
| Parquet | 1,624.00 ms | 813.14 ms | rudb 2.00x faster | 419.11 MiB | 155.34 MiB | rudb 2.70x smaller |

Q9 shows the local result without suite run variance.

| Mode | Engine | Q9 time | Q9 peak RSS | Comparison with DuckDB |
| --- | --- | ---: | ---: | --- |
| Native | DuckDB | 12.00 ms | 123.47 MiB | reference |
| Native | rudb before | 28.66 ms | 49.75 MiB | 2.39x slower, 2.48x smaller |
| Native | rudb after | 13.91 ms | 64.42 MiB | 1.16x slower, 1.92x smaller |
| Parquet | DuckDB | 36.00 ms | 140.78 MiB | reference |
| Parquet | rudb before | 34.32 ms | 42.81 MiB | 1.05x faster, 3.29x smaller |
| Parquet | rudb after | 18.13 ms | 52.02 MiB | 1.99x faster, 2.71x smaller |

The flat pair exchange trades memory for fewer probes. Native Q9 uses 14.67 MiB more RSS than the inline-set path, but it remains 1.92 times smaller than DuckDB and the suite peak does not move. Native Q9 is 2.06 times faster than the preceding rudb build. Parquet Q9 is 1.89 times faster.

The native suite is now within five percent of DuckDB at one million rows. Q10 still combines SUM, COUNT, AVG, and distinct count for the same integer groups. It cannot use this single-result operator yet. Q14 groups stable string codes, where singleton groups make the inline distinct state as fast as the tested pair exchange.
