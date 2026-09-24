# Q9 row-preserving run projection

Q9 counts distinct `UserID` values within each `RegionID`. A sorted projection already keeps every source row, but repeats the eight-byte user value on every row. The new optional run format stores one user value and run length for consecutive equal users, followed by one region code for **each original row**. It retains duplicate pairs. No distinct-pair list, region count, rank, or SQL result is stored. Query workers read the codes and count distinct pairs when the SQL runs.

This follows the [Q2 onward storage review](q2-onward-row-projection-review.md). Q2 derives its filtered count from generic row, null, and retained-value frequency statistics; Q3 and Q4 use sums and non-null counts; Q5 and Q6 use exact column cardinality; Q7 uses extrema. Q8 reads encoded rows and counts groups at query time. The Q9 run projection is an optional row-valued access path, not another precomputed answer. A table append invalidates it through the table generation.

The SQL was identical for both engines: `SELECT RegionID, COUNT(DISTINCT UserID) AS u FROM hits GROUP BY RegionID ORDER BY u DESC LIMIT 10`. The runner checked the complete selected key/count set and descending counts, allowing either order for tied counts. Every query ran in a new process; `wait4` measured wall time, user plus system CPU, and whole-child peak RSS. The operating-system page cache was warm and not cleared. Cases rotated order. The 10m result uses 51 rounds; smaller sizes use 11. These are shared-host medians, not quiet-host latency claims.

| Rows | RuDB run wall | DuckDB native wall | DuckDB / RuDB wall | RuDB CPU | DuckDB CPU | RuDB peak RSS | DuckDB peak RSS | DuckDB / RuDB RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 15.647 ms | 129.009 ms | 8.25x | 9.760 ms | 79.450 ms | 6.38 MiB | 37.49 MiB | 5.88x |
| 10k | 43.275 ms | 131.325 ms | 3.03x | 32.911 ms | 85.959 ms | 6.88 MiB | 38.12 MiB | 5.54x |
| 1m | 45.929 ms | 306.143 ms | 6.67x | 38.888 ms | 267.697 ms | 10.38 MiB | 80.75 MiB | 7.78x |
| 10m | 51.752 ms | 446.263 ms | 8.62x | 71.652 ms | 681.028 ms | 11.38 MiB | 133.78 MiB | 11.76x |

The 10m comparison also ran the previous fixed-width sorted projection with the **same RuDB binary** in each rotating round. It took 64.282 ms wall, 100.379 ms CPU, and 11.38 MiB peak RSS. The run format took 51.752 ms wall and 71.652 ms CPU; DuckDB took 446.263 ms wall and 681.028 ms CPU in those same rounds. That is a 1.24x RuDB wall improvement, but still short of the 10x wall goal against DuckDB. The [three-way samples](q9-run-projection/q9-runs-rebased-vs-sorted-10m.json) and [all-size samples](q9-run-projection/) preserve every fresh-process measurement. Small-table results are poor enough that this format should stay opt-in.

Building the 10m run projection is a separate step after native load. With the rebased builder it took 1.949 s wall, 1.657 s CPU, and 198.47 MiB peak RSS in one fresh process. It added 40,350,489 bytes to a 1,177,015,790-byte native file. The resulting file was 1,217,366,279 bytes and byte-identical to the file used for the query runs. This build cost must be included in any full load comparison. A same-host paired DuckDB and RuDB Parquet-to-native load run with this projection is still needed; query timing alone does not cover it. The [raw build record](q9-run-projection/runs-rebased-build-10m.json) contains the process counters.

The 10m source native file predates the corrected partial-frequency writer used in the Q2–Q8 review. Q9 reads original rows from the projection and does not consume that legacy synopsis. The rebased release RuDB binary has SHA-256 `bf623a684a34ef4627db0785b2fb16ff1714d988e2c9c797a60ae954a9f491bc`. All 193 native and 411 database unit tests passed. Strict Clippy passed for the changed native and projection crates; the database crate passed with two existing `explicit_auto_deref` warnings in unchanged `rudb-exec/src/group.rs` excluded.
