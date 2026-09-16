# Mixed aggregate radix owners

Q10 combines four different aggregate behaviors for each `RegionID`: an exact `SUM(SMALLINT)`, `COUNT(*)`, an exact `AVG(SMALLINT)`, and `COUNT(DISTINCT BIGINT)`. The general operator built worker-local group tables and one distinct set per group, then merged every numeric state and distinct set after the scan. Current main took 32.45 ms on the native file and 35.20 ms on the source Parquet file at one million rows.

[rudb #708](https://github.com/tamnd/rudb/pull/708) gives this exact typed shape one radix owner per group. Scan workers decode the four integer columns into 24-byte temporary rows and hand off 32K-row batches. Sixteen owners maintain one compact numeric state per region and one flat structure-of-arrays set for distinct `(RegionID, UserID)` pairs. The pair table uses a two-byte control value, an eight-byte user array, and a four-byte region array. The owner table is released before output vectors are built, and the output reservation stays live with the result.

The planner previously propagated a TopN count bound only when the count was the first aggregate call. Q10 orders by its second call, so the new path also resolves the ordered output column to the exact aggregate call. Each owner emits at most ten candidates, and the existing global TopN makes the final choice.

The first prototype buffered every 24-byte row until finalization. It took about 43 ms and peaked near 95 MiB because the retained rows and pair buckets overlapped. Folding into shared owners for every scan chunk reduced RSS to about 53 MiB but lock queues kept time near 33 ms. Batching 32K rows reduced lock acquisitions without retaining the full input. Reading native packed integers and flat Parquet integers directly then removed repeated vector form and validity checks. The final native median is 15.18 ms.

The audit used the same strided ClickBench samples, DuckDB binary, source Parquet files, six fresh processes per query, Linux child RSS measurement, and deterministic verifier as the preceding reports. The 1K and 10K sizes ran first. The implementation was then rebased onto current main and the complete ladder was repeated. All 43 queries completed in DuckDB native, rudb native, DuckDB Parquet, and rudb Parquet at every size.

## Q10 at one million rows

Five hot repetitions ran in fresh processes. Time is the median CLI query time. RSS is the largest Linux `wait4` peak among the hot repetitions.

| Mode | DuckDB time | rudb main time | rudb after | Improvement from main | DuckDB RSS | rudb after RSS |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Native | 17.00 ms | 32.45 ms | 15.18 ms | 2.14x faster | 136.09 MiB | 57.25 MiB |
| Parquet | 49.00 ms | 35.20 ms | 19.39 ms | 1.82x faster | 181.32 MiB | 65.33 MiB |

The flat pair table and bounded handoff buffer increase Q10 RSS from 53.48 MiB to 57.25 MiB natively and from 51.02 MiB to 65.33 MiB on Parquet. The completed operator remains 2.38 times smaller than DuckDB natively and 2.78 times smaller on Parquet. The whole-suite RSS peak remains below the preceding audit.

## Small sizes

| Rows | Mode | DuckDB query total | rudb query total | Time comparison | DuckDB peak RSS | rudb peak RSS | Memory comparison |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1K | Native | 122.00 ms | 15.37 ms | rudb 7.94x faster | 54.01 MiB | 8.27 MiB | rudb 6.53x smaller |
| 1K | Parquet | 153.00 ms | 24.71 ms | rudb 6.19x faster | 53.74 MiB | 8.52 MiB | rudb 6.31x smaller |
| 10K | Native | 145.00 ms | 29.60 ms | rudb 4.90x faster | 54.05 MiB | 9.69 MiB | rudb 5.58x smaller |
| 10K | Parquet | 197.00 ms | 63.56 ms | rudb 3.10x faster | 57.15 MiB | 11.73 MiB | rudb 4.87x smaller |

At 10K rows, native Q10 is 0.91 ms against DuckDB's 5.00 ms. Parquet Q10 is 1.27 ms against DuckDB's 7.00 ms.

## One million rows

| Mode | DuckDB query total | rudb query total | Time comparison | DuckDB peak RSS | rudb peak RSS | Memory comparison |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Native | 544.00 ms | 572.91 ms | DuckDB 1.05x faster | 307.53 MiB | 165.68 MiB | rudb 1.86x smaller |
| Parquet | 1,584.00 ms | 804.65 ms | rudb 1.97x faster | 389.32 MiB | 157.48 MiB | rudb 2.47x smaller |

Q10 removes 17.26 ms from the native query median and 15.81 ms from the Parquet median. The native suite is now within five percent of DuckDB while retaining a 1.86 times memory advantage. The Parquet suite is almost twice as fast as DuckDB and uses less than half its peak RSS.

The remaining native gap is spread across Q16, Q17, Q36, Q28, Q15, and Q14. Their shared cost is high-cardinality grouping and TopN rather than this mixed aggregate shape, so the next change should target their key exchange and table layout instead of adding more cases to this operator.
