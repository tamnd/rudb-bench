# Q9 through a bound covering aggregate

[RuDB's planner-backed grouped-distinct change](https://github.com/tamnd/rudb/pull/1797) lets a bound aggregate use a current row-preserving native run projection. It matches stored column identities and aggregate semantics after binding. It does not inspect SQL text, aliases, ordering, or a ClickBench query number. The projection stores one covered code for every source row; each query scans those codes and counts distinct pairs. The ordinary operators above the aggregate still handle output expressions, ordering, and limits. A filtered query, incompatible type, missing projection, or table changed since the projection build uses the existing radix aggregate.

The compared RuDB binaries were built as release from the frequency-span source `58481010` plus the span patch (`f5d6942c435e8fcaada4ccc49051aace60863b750781dccccd3c386291c93931`) and from `4c7a8e54` plus the covering-source change (`b48c1993a92c0d52db30039284be13bb14a0911006a02d2a542105bd65803870`). The latter was rebased onto current main after measurement; the intervening changes affect eager aggregation, left joins, and regular expressions. Both binaries used the same native file at each size. DuckDB used its native table from the same Parquet rows. Every process executed exactly `SELECT RegionID, COUNT(DISTINCT UserID) AS u FROM hits GROUP BY RegionID ORDER BY u DESC LIMIT 10`. The RuDB commands included `--set threads=6`; DuckDB reported six default threads. The setting forces the old RuDB binary through its SQL engine rather than its former one-statement CLI shortcut. The new binary has no such shortcut.

Each number below is a median of 21 alternating fresh-process runs on a shared six-CPU Linux host. The runner checked the complete CSV answer and descending count order for all 252 executions. `wait4` captured wall time, user plus system CPU time, peak process RSS, and I/O. The page cache was warm. Host load was high and varied, so wall ratios are observations under contention; CPU times and paired wins are included to make the comparison assessable.

| Rows | Old RuDB engine wall | Covering RuDB wall | DuckDB wall | DuckDB / covering wall | Covering RuDB CPU | DuckDB CPU | Covering RuDB RSS | DuckDB RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 44.745 ms | 52.159 ms | 463.056 ms | 8.88x | 9.579 ms | 84.051 ms | 10.38 MiB | 36.87 MiB |
| 10k | 65.140 ms | 47.608 ms | 494.282 ms | 10.38x | 10.755 ms | 85.725 ms | 10.50 MiB | 37.48 MiB |
| 1m | 469.482 ms | 205.471 ms | 1,061.467 ms | 5.17x | 27.552 ms | 308.168 ms | 11.38 MiB | 81.00 MiB |
| 10m | 906.367 ms | 621.144 ms | 2,479.344 ms | 3.99x | 80.552 ms | 794.567 ms | 13.00 MiB | 134.40 MiB |

| Rows | Paired covering wall wins vs old | Paired covering CPU wins vs old | Old RuDB CPU | Old RuDB RSS | DuckDB / covering CPU | DuckDB / covering RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 7/21 | 9/21 | 9.573 ms | 10.50 MiB | 8.77x | 3.55x |
| 10k | 14/21 | 17/21 | 11.797 ms | 11.25 MiB | 7.97x | 3.57x |
| 1m | 21/21 | 21/21 | 138.198 ms | 55.75 MiB | 11.18x | 7.12x |
| 10m | 19/21 | 21/21 | 328.102 ms | 72.93 MiB | 9.86x | 10.34x |

The large-file reduction is clear: at 10m the covering path used 4.07x less CPU and 5.61x less RSS than the regular radix path on the same native file. It still missed the **10x DuckDB wall-time goal**, and the 1k wall median regressed. The operator currently scans projection pages on one engine worker to keep its CPU in the query's worker lease and metrics. A future parallel implementation must divide pages among the engine workers and merge partial group counts within that lease. Spawning an unaccounted thread pool inside the source would make the benchmark and session thread limit misleading.

The first page was previously read and checksummed once for its header and again for its rows. The new reader reuses that verified page, reducing an allocation and duplicate work for any run projection. The query measurements above used this final reader. The native file format and build command did not change. Projection build costs and Parquet-to-native load costs are in the [frequency-span load report](native-frequency-spans.md); at 10m the separate projection build took 1.566 s in that run and its bytes are included in the 1,184,871,347-byte RuDB file. DuckDB's 10m native file was 1,844,981,760 bytes. The same report records the bounded 10m load comparison for both engines.

The [raw trials](q9-covering-grouped-distinct/) and [fresh-process runner](../../scripts/q9-fresh-process.py) preserve the measurement. The engine change passed 204 native unit tests, 419 database unit tests, the focused stale-projection and filtered-query checks, and strict Clippy for native, executor, and database crates. The test uses table and column names unrelated to ClickBench and verifies the selected aggregate in engine metrics.
