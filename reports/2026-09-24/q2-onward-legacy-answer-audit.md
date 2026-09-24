# Q2 onward: check native facts against saved answers

The native storage rule is that a file may keep reusable column facts and row-valued indexes. It must not supply a stored grouped SQL answer. This review starts at Q2 and follows both the current writer and the older-file reader. The SQL is unchanged from ClickBench.

| Query | Native input used | Work when the query runs | Finding |
| --- | --- | --- | --- |
| Q2 | Row count, null count, and an exact retained zero frequency | Subtract null and zero rows; scan if the zero count is unknown | No saved filtered count is written. The old `nonzero` field is ignored. |
| Q3 and Q4 | Per-column integer sums and non-null counts | Combine facts and divide for averages | Reusable scalar statistics. |
| Q5 and Q6 | Exact per-column distinct counts | Return the requested column cardinality | Reusable scalar statistics, although each equals this unfiltered query's answer. |
| Q7 | Per-column minimum and maximum | Return the selected bounds | Reusable scalar statistics. |
| Q8 | Encoded column values | Count each group and sort while SQL runs | New multi-value numeric frequency lists are partial. No complete grouped answer is used. |
| Q9 | Optional sorted projection retaining one region code for every original row | Deduplicate user/region pairs, count regions, and sort | The projection keeps duplicate rows. Its build cost is extra load work. |

The Q2 shortcut is exact only when the zero frequency is present, or a complete synopsis proves zero absent. Otherwise `quick_nonzero` returns no answer and the normal query reads rows. The writer keeps at most two leaders in a multi-value numeric frequency list and records an omission bound. Neither the writer nor the reader uses the old query-specific `nonzero` catalog field. The fast CSV recognizer now also rejects reserved SQL words as unquoted table or column names.

The older-file check found a gap in the grouped-answer rule. Current files leave the pair leader and fixed host aggregate sections empty, but the reader could still return those blocks from old files for a two-key TopN or the host-expression grouped query. [RuDB #1724](https://github.com/tamnd/rudb/pull/1724) makes the reader parse those blocks for file compatibility and return no grouped answer from them. The ordinary row path or query-time candidate path remains available. A regression test injects bogus legacy counts into the reader and checks that neither is returned.

This change does not alter the Q2 ClickBench statement or the native file writer. It does not create a new Q2 speedup. For context, the prior corrected-file, alternating fresh-process comparison reported these Q2 medians on the same Parquet rows and SQL. These are historical measurements from the [partial-frequency review](q2-q8-partial-numeric-frequencies.md), not measurements of this reader fix.

| Rows | RuDB wall | DuckDB wall | DuckDB / RuDB wall | RuDB peak RSS | DuckDB peak RSS | DuckDB / RuDB RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 3.853 ms | 191.406 ms | 49.68x | 3.88 MiB | 35.15 MiB | 9.06x |
| 10k | 7.836 ms | 128.793 ms | 16.44x | 4.00 MiB | 35.27 MiB | 8.82x |
| 1m | 5.474 ms | 151.942 ms | 27.76x | 3.88 MiB | 38.40 MiB | 9.90x |
| 10m | 5.476 ms | 225.520 ms | 41.18x | 3.88 MiB | 57.62 MiB | 14.85x |

Those small-size RSS ratios remain below the 10x goal. The shared host was contended, so the wall ratios are not quiet-host guarantees. The [row-projection review](q2-onward-row-projection-review.md) and [Q9 run report](q9-row-preserving-run-projection.md) contain the corresponding grouped-query results and build costs. Future reports should use files written by the current partial-frequency writer and identify any older native files used for read-path comparisons.
