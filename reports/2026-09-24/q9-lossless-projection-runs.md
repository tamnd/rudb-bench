# Q9 with lossless native run encoding

[RuDB #1816](https://github.com/tamnd/rudb/pull/1816) changes how a two-column native run projection stores repeated covered values. A run already has an order value and a source row count. If every covered value in that run is the same, the new layout stores that code once. A mixed run keeps each code in its original order. The row count lets a reader reconstruct every `(order, covered)` pair, including duplicates. The file stores no distinct-pair set, grouped count, or SQL result. The planner still selects this projection from bound columns and falls back to the ordinary aggregate for filters, stale sections, and unsupported shapes.

The 10m legacy projection has 10,000,000 source rows in 1,530,334 order-value runs. The [inspection script](q9-lossless-projection-runs/analyze-projection.py) found 1,522,750 uniform runs, or 99.5%. This distribution explains why repeating each covered code in the old 512 KiB pages cost both bytes and decode work. The new 64 KiB pages have independent checksums. Long runs that cannot fit the new first page use the legacy raw layout. The native test suite also reconstructs duplicate pairs from the encoded payload, rather than checking only a distinct aggregate.

## Fresh-process query comparison

Every process executed the same Q9 statement:

```sql
SELECT RegionID, COUNT(DISTINCT UserID) AS u
FROM hits
GROUP BY RegionID
ORDER BY u DESC
LIMIT 10;
```

The two RuDB cases used the same release binary, SHA-256 `b1db00179a369c1fd2457189cbea896ec8fdd4c5833fda5458e3def8a06976f2`, and identical base-table bytes. Only the attached projection layout differed. The binary was built from `c0040af9` plus the RLE change and later rebased onto main; the intervening changes affect general grouping and joins, not the projection reader. DuckDB was `v2.0.0-dev84237`, SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. It read native tables loaded from the same Parquet rows. Both RuDB commands included `--set threads=6`, and DuckDB reported six default query threads.

Each cell is a median of alternating fresh processes on the shared six-CPU Linux host: 31 trials at 1k, 10k, and 1m, and 51 trials at 10m. The [runner](../../scripts/q9-fresh-process.py) checked the full CSV output and descending count order for all 432 executions. Its `wait4` helper recorded wall time, user plus system CPU time, peak process RSS, and I/O. The page cache was warm. Host load varied, so CPU time and paired wall wins matter alongside the wall medians.

| Rows | Old RuDB wall | RLE RuDB wall | DuckDB wall | DuckDB / RLE wall | Paired RLE wall wins |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 17.828 ms | 14.388 ms | 155.797 ms | 10.83x | 23/31 |
| 10k | 15.708 ms | 17.475 ms | 126.393 ms | 7.23x | 21/31 |
| 1m | 42.529 ms | 41.996 ms | 304.872 ms | 7.26x | 14/31 |
| 10m | 65.180 ms | 48.171 ms | 441.216 ms | 9.16x | 47/51 |

| Rows | Old RuDB CPU | RLE RuDB CPU | DuckDB CPU | Old RuDB RSS | RLE RuDB RSS | DuckDB RSS | DuckDB / RLE RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 9.340 ms | 8.288 ms | 88.396 ms | 9.88 MiB | 9.38 MiB | 38.46 MiB | 4.10x |
| 10k | 10.612 ms | 9.992 ms | 91.386 ms | 9.88 MiB | 9.50 MiB | 39.08 MiB | 4.11x |
| 1m | 27.400 ms | 25.678 ms | 295.077 ms | 11.75 MiB | 10.75 MiB | 81.07 MiB | 7.54x |
| 10m | 76.628 ms | 39.243 ms | 678.736 ms | 15.75 MiB | 13.12 MiB | 124.87 MiB | 9.51x |

The 10m result saves 17.009 ms of median wall time, 37.385 ms of CPU time, and 2.63 MiB of peak RSS against the old projection. It **does not meet the 10x DuckDB wall-time or RSS goal in this final clean-file run**. An earlier 51-trial prototype run against another DuckDB native file from the same Parquet rows did cross both ratios; its [raw trials](q9-lossless-projection-runs/q9-rle-clean-10m-51.json) are retained to show the variation. The final clean-file result above is the headline. At 10k, the independently computed wall medians regress despite 21 of 31 paired wins, which is a reminder that the shared-host wall distribution is noisy.

The 10m old and RLE files were copied from one clean RuDB base file loaded from Parquet. The smaller query files were copied from existing native files before attaching RLE, so they contain an orphaned old projection. Their query timings are valid, but their total file sizes are not a format comparison. The [final raw trials](q9-lossless-projection-runs/) preserve every process measurement.

## Indexed load and file bytes

The 10m RuDB base table and DuckDB native table were each loaded from the same Parquet input with the exact [bounded load SQL](q9-frequency-spans/load-10m-bounded.sql), SHA-256 `32adf3666741e3cafa7873fccaaa2cf0b7725992eb6ff4ed6ab35cbd97760ecb`. The query's RLE projection was then built explicitly on a copy of the RuDB base file. The old projection was built on another copy. These are consecutive whole-process measurements, not alternating trials; the shared host was busy.

| Step | Wall | CPU | Peak RSS | Final file bytes |
| --- | ---: | ---: | ---: | ---: |
| RuDB Parquet to native base | 158.900 s | 144.611 s | 1,652.02 MiB | 1,223,167,552 |
| Old projection build | 6.764 s | 2.032 s | 198.92 MiB | 1,263,607,923 |
| RLE projection build | 3.274 s | 2.164 s | 184.62 MiB | 1,248,083,759 |
| DuckDB Parquet to native | 217.609 s | 120.567 s | 2,245.25 MiB | 1,845,243,904 |

The old projection adds 40,440,371 bytes; the RLE projection adds 24,916,207 bytes, a 38.39% reduction. Including the explicit build, the observed RuDB indexed-load wall time was 162.174 s. DuckDB's native load was 217.609 s in its consecutive run. The [load resource records](q9-lossless-projection-runs/) contain the process-level measurements. The new format reduces projection-build peak RSS; its CPU time was slightly higher in these runs, so the wall-time difference should not be read as a general build-speed claim.

Validation after the final rebase: 206 native, 527 executor, and 421 database unit tests; strict Clippy for those crates; formatting. The native tests cover lossless pair reconstruction, mixed and uniform runs, a wide dictionary, page partition merging, and legacy fallback for an oversized run.
