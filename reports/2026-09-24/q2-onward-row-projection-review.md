# Q2 onward: stored facts and Q9 row projection

This review applies the rule that a native file may store reusable column facts and row-valued access paths, but not a saved ClickBench result. The earlier Q8 complete frequency list crossed that boundary. New files keep multi-value numeric frequencies partial, and Q8 counts encoded rows when its SQL runs. The [Q2 through Q8 correction](q2-q8-partial-numeric-frequencies.md) contains the paired measurements on files written with that rule.

| Query | Stored input | Work when SQL runs |
| --- | --- | --- |
| Q2 | Table rows, null count, and an exact leading zero frequency if retained | Derive the filtered scalar count, or scan when the zero frequency is absent |
| Q3 and Q4 | Per-column sum and non-null count | Combine scalar facts and divide for average |
| Q5 and Q6 | Per-column distinct-value count | Return the requested column's cardinality |
| Q7 | Per-column minimum and maximum | Return the requested bounds |
| Q8 | Encoded column values and a partial frequency synopsis | Read rows, group, count, and sort |
| Q9 | Optional projection with one original `UserID` and `RegionID` code per row | Count distinct user/region pairs and sort the groups |

Q5 and Q6 use exact column cardinalities, which are standard reusable statistics but equal the result of these unfiltered scalar queries. Q9's projection stores every source row in user order, not distinct pairs or region counts. Its builder is an explicit extra storage operation. An insert makes it stale through the table generation; a query without a current matching projection falls back to regular execution. The SQL text is identical for RuDB and DuckDB.

As a regression check, Q2 through Q8 returned the same complete CSV on projected RuDB, unindexed RuDB, and DuckDB at 1k and 10m. Q8 compared key/count rows without requiring an order for ties. The earlier [partial-frequency review](q2-q8-partial-numeric-frequencies.md) checked these queries on files written by the corrected writer at all four sizes.

Q9 is `SELECT RegionID, COUNT(DISTINCT UserID) AS u FROM hits GROUP BY RegionID ORDER BY u DESC LIMIT 10`. Both engines returned the same complete selected key/count set at 1k, 10k, 1m, and 10m. Tied counts may appear in either order under this SQL, so the runner checks the selected set and descending counts. It runs each engine in a new process and measures wall time, user plus system CPU, and peak child RSS with `wait4`. The operating-system page cache is warm; it is not cleared. The three cases rotate order in each round. The 10m result uses 51 rounds, and the other sizes use 11.

| Rows | RuDB unindexed wall | RuDB indexed wall | DuckDB native wall | DuckDB / indexed wall | Indexed CPU | DuckDB CPU | Indexed peak RSS | DuckDB peak RSS | DuckDB / indexed RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 24.089 ms | 13.050 ms | 118.306 ms | 9.07x | 8.532 ms | 86.415 ms | 5.50 MiB | 38.33 MiB | 6.97x |
| 10k | 83.843 ms | 50.870 ms | 128.685 ms | 2.53x | 30.816 ms | 93.871 ms | 6.00 MiB | 39.21 MiB | 6.54x |
| 1m | 128.880 ms | 41.039 ms | 261.949 ms | 6.38x | 25.234 ms | 255.569 ms | 8.25 MiB | 81.71 MiB | 9.90x |
| 10m | 140.014 ms | 60.417 ms | 368.027 ms | 6.09x | 100.305 ms | 656.268 ms | 11.50 MiB | 134.02 MiB | 11.65x |

The optional projection improved the same RuDB binary's 10m wall median by 2.32x and reduced its peak RSS from 76.88 MiB to 11.50 MiB. It still misses the 10x wall target at every size and the 10x memory target below 10m. The server had six available CPUs and load averages above six during some runs, so wall time is subject to scheduling noise. The 51-round alternating run is the stronger 10m estimate; it is not a quiet-host latency claim. The complete [raw clean-process records](q9-row-projection/q9-final-clean-10m.json) preserve every sample, with the [other sizes in the same directory](q9-row-projection/). The [runner](../../scripts/q9-fresh-process.py) rejects wrong output.

The projection build is a separate fresh process after the native load. This table shows the measured extra cost and final file size. It must be included when comparing full load workflows. The existing paired Parquet-to-native load measurement in the [Q2 through Q8 report](q2-q8-partial-numeric-frequencies.md) ran on another host and another file revision, so adding those times to this table would not be a valid paired load comparison.

| Rows | Projection build wall | Build peak RSS | Unindexed native file | Indexed native file | Added bytes |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 0.049 s | 5.38 MiB | 1,325,569 B | 2,080,389 B | 754,820 B |
| 10k | 0.089 s | 15.12 MiB | 5,533,695 B | 8,303,199 B | 2,769,504 B |
| 1m | 0.446 s | 34.40 MiB | 219,201,943 B | 230,673,534 B | 11,471,591 B |
| 10m | 3.407 s | 254.97 MiB | 1,177,015,790 B | 1,278,586,609 B | 101,570,819 B |

The 10m native source in this Q9 comparison predates the partial-frequency writer correction and contains legacy synopses. Q9 reads its row projection or the ordinary input rows; it does not consume those synopses. A full same-host Parquet load comparison with projection construction remains to be measured. The [raw build records](q9-row-projection/projection-build-v2-10m.json) give every measured process resource, and the [expected CSV files](q9-row-projection/) preserve correctness output.

The release candidate is based on RuDB main `dacbb603`, with binary SHA-256 `51bd4c2f0800081c4f62d2baf553685ef67f92501e425d96296398912934951d`. DuckDB v2.0.0-dev84237 has SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The native, database, and CLI library suites passed 188, 410, and 30 tests. Strict Clippy passed for the changed crates. The Q9 parallel scan was profiled on the 10m projection; row deduplication, extent copies, and checksums were the largest sampled costs. A per-user region bit-mask experiment was slower than epoch marks in the [paired trial](q9-row-projection/q9-bitmask-10m.json) and was removed.
