# Q9 with engine-leased projection page scans

[The RuDB change](https://github.com/tamnd/rudb/pull/1805) divides a current covering run projection into whole-page ranges and scans them on the query engine's leased workers. The native file still stores one covered code for every source row, including duplicates. It stores no distinct counts or query results. The planner selects the source from bound column identities and aggregate semantics, as described in the [earlier Q9 report](q9-covering-grouped-distinct.md). Filters, stale projections, and queries without a matching projection use the normal aggregate. Small projections keep one worker.

The comparison used release binaries built from main `5aa4fdfb` (`66427c29482722d9780b4d9dde8e5d4bed8dbc0119f21243650d82f377284e90`) and that main plus the page-scan change (`09cfe50a61d418b56724c1d9525fca92d65d12355c006830f537acc52dc55e20`). DuckDB was `v2.0.0-dev84237`, binary SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. Both RuDB binaries read the exact same native file at each size. DuckDB read its native table loaded from the same Parquet rows. The SQL was identical in every process:

```sql
SELECT RegionID, COUNT(DISTINCT UserID) AS u
FROM hits
GROUP BY RegionID
ORDER BY u DESC
LIMIT 10;
```

Both RuDB commands used `--set threads=6`; DuckDB reported six default threads. Each cell is the median of 31 alternating fresh-process trials on a shared six-CPU Linux host. The [runner](../../scripts/q9-fresh-process.py) checked the complete CSV answer and descending count order for all 372 executions. Its `wait4` helper measured process wall time, user plus system CPU time, and peak RSS. The page cache was warm. Other jobs ran on the host, so the paired wins and CPU time help interpret wall-time variation.

| Rows | Main RuDB wall | Page-scan RuDB wall | DuckDB wall | Main / page-scan wall | DuckDB / page-scan wall | Paired page-scan wall wins |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 18.054 ms | 19.364 ms | 166.880 ms | 0.93x | 8.62x | 11/31 |
| 10k | 20.820 ms | 19.938 ms | 163.662 ms | 1.04x | 8.21x | 17/31 |
| 1m | 52.845 ms | 41.271 ms | 318.176 ms | 1.28x | 7.71x | 26/31 |
| 10m | 132.243 ms | 68.603 ms | 511.307 ms | 1.93x | 7.45x | 31/31 |

| Rows | Main RuDB CPU | Page-scan RuDB CPU | DuckDB CPU | Main RuDB RSS | Page-scan RuDB RSS | DuckDB RSS | DuckDB / page-scan RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 9.509 ms | 9.456 ms | 95.182 ms | 10.12 MiB | 9.88 MiB | 39.21 MiB | 3.97x |
| 10k | 10.206 ms | 10.046 ms | 93.187 ms | 10.12 MiB | 9.88 MiB | 39.70 MiB | 4.02x |
| 1m | 26.294 ms | 27.984 ms | 308.011 ms | 10.62 MiB | 11.62 MiB | 81.46 MiB | 7.01x |
| 10m | 65.511 ms | 72.921 ms | 693.368 ms | 12.88 MiB | 15.75 MiB | 135.57 MiB | 8.61x |

At 10m the page-scan source saves 63.640 ms of median wall time, but spends 7.410 ms more CPU and raises peak RSS by 2.87 MiB. The gain is parallel latency, not less total work. The **10x DuckDB wall-time and RSS targets remain unmet**. At 1k, the median wall time is 1.310 ms slower, while CPU and RSS are effectively flat against main. A single-worker path avoids the extra scanner state and first-page copy for small projections.

The native format, source rows, projection build command, and load process did not change. Indexed-load cost and file sizes are recorded in the [run-projection load report](native-frequency-spans.md). The [raw final trials](q9-engine-page-partitions/) include the three engines at each size. A sorted-code file layout and a one-byte short-run decoder were also tested, then dropped because neither showed a reliable CPU benefit; their raw trials are retained in the same directory. `perf record` on the 10m query identified the native page decoder as the largest sampled user-space symbol, while file-read and thread activity also appeared in the profile. The next work should reduce the per-worker page-buffer and stack footprint without returning to a single-worker scan.

Validation after rebasing onto `5aa4fdfb`: 205 native unit tests, 524 executor unit tests, 420 database unit tests, strict Clippy for all three crates, and formatting. A native test forces multiple projection pages and checks that two independent page ranges merge to the exact distinct counts.
