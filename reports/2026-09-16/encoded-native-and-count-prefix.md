# Encoded native strings and count prefix aggregation

The native storage experiment now keeps one code space for each string column and exposes storage backed text vectors to execution. String comparisons, LIKE, regular expressions, length, casts, sorting, TopN, and grouping can operate without decoding every row into an owned string. Grouped counts over one stable dictionary use dense code indexed counters.

The same run includes direct aggregates over packed numeric pages and a count prefix rule for high cardinality grouping. When the parent asks for `ORDER BY COUNT(*) DESC LIMIT k`, each radix partition emits its local top `k`. The ordinary TopN receives that union and makes the final global choice. This is exact because a group behind `k` groups in its own partition cannot enter the first `k` overall.

The audit used the same one million row strided ClickBench sample, DuckDB binary, rudb native file, source Parquet file, six executions per query, child RSS measurement, and deterministic verifier as the earlier storage audit. All 43 queries completed in all four modes. Every deterministic rerun matched DuckDB.

## Suite result

| Mode | DuckDB query total | rudb query total | DuckDB peak RSS | rudb peak RSS | Time ratio | RSS ratio |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Native | 0.574 s | 0.727 s | 313.78 MiB | 168.46 MiB | rudb 1.27x slower | rudb 1.86x smaller |
| Parquet | 1.580 s | 0.910 s | 364.70 MiB | 167.62 MiB | rudb 1.74x faster | rudb 2.18x smaller |

The native suite is still behind DuckDB. The Parquet suite is faster and smaller, but neither path has reached the 10x target.

## Query comparison

| Query | DuckDB native | rudb native | DuckDB Parquet | rudb Parquet |
| --- | ---: | ---: | ---: | ---: |
| Q3 | 3.000 ms | 1.980 ms | 25.000 ms | 9.034 ms |
| Q7 | 1.000 ms | 2.159 ms | 21.000 ms | 8.863 ms |
| Q13 | 8.000 ms | 14.447 ms | 32.000 ms | 16.391 ms |
| Q19 | 20.000 ms | 39.817 ms | 45.000 ms | 42.462 ms |
| Q28 | 14.000 ms | 29.548 ms | 54.000 ms | 21.272 ms |
| Q29 | 101.000 ms | 84.860 ms | 86.000 ms | 46.654 ms |
| Q33 | 23.000 ms | 51.374 ms | 46.000 ms | 43.609 ms |
| Q34 | 31.000 ms | 6.594 ms | 63.000 ms | 48.482 ms |
| Q35 | 31.000 ms | 7.414 ms | 67.000 ms | 59.335 ms |

## Q33 stage result

Before count prefix emission, Q33 produced 999,975 aggregate rows. Afterwards it produced 160 candidates, ten from each of sixteen radix partitions.

| Measurement | Before | After | Change |
| --- | ---: | ---: | ---: |
| Native query time | 57.646 ms | 51.374 ms | 10.9% lower |
| Native peak RSS | 132.2 MiB | 124.2 MiB | 6.1% lower |
| Parquet query time | 47.875 ms | 43.609 ms | 8.9% lower |
| Parquet peak RSS | 198.4 MiB | 167.6 MiB | 15.5% lower |
| Aggregate emit stage | about 42 ms | 6.6 ms | 84% lower |
| Aggregate output rows | 999,975 | 160 | 6,250x fewer |

The output materialization problem is now bounded by the requested prefix. Q33 remains 2.23 times slower than DuckDB native in the same run. Its profile points to two remaining costs: building worker local compact hash tables and probing every local group again during the final merge. Sending full gathered vectors directly to shared partitions was measured and rejected because lock waits and row gathering raised Q33 to 66 through 70 ms. The next design needs a compact fixed width radix exchange that transfers key and state records in batches, with one owner table per partition.

The 1k and 10k audits were run before the one million row audit. Both completed all 43 queries in all four modes and passed deterministic verification.
