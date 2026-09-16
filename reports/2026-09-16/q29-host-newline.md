# ClickBench query 29 host extraction and embedded newlines

ClickBench query 29 extracts a host from `Referer` with `regexp_replace(Referer, '^https?://(?:www\.)?([^/]+)/.*$', '\1')`. Rudb has a fast path for this fixed pattern because it runs on millions of rows. That path took the bytes before the first slash after the scheme but did not check the anchored `.*$` after the slash. The general regex does not let a dot consume a newline by default. On the 10m sample, the fast path put URLs with an embedded newline into a host group that DuckDB left as the original string. The `Referer <> ''` input count was identical in DuckDB Parquet and rudb. For `google.ru`, DuckDB grouped 216,193 rows and rudb grouped 216,205 before the fix. All 12 extra rows contained a newline after the host slash.

The corrected shortcut uses a byte search for a newline in the suffix. It still extracts the host directly for ordinary URLs and lets an empty `www.` host fall back to the capture the general regex would make. A differential test checks the shortcut against rudb's regex machine on normal, newline, and boundary cases. On the 10m sample, deterministic query 29 now agrees with DuckDB on all 11 grouped keys, counts, averages, and minimum strings.

The benchmark ran all 43 ClickBench queries for five hot repetitions in fresh processes on one Linux host. DuckDB native read a preloaded table. DuckDB Parquet and rudb read the same file per query. Query time and child CPU sum per-query medians; peak RSS is the largest child maximum resident set. The final binary had its own SHA-256 hash and was pinned before timing. These are sample measurements, not official ClickBench scores.

| 10m query 29 | Engine and run | Median query time (ms) | Median child CPU (ms) | Peak child RSS (MiB) | Deterministic answer |
| --- | --- | ---: | ---: | ---: | --- |
| Before | DuckDB native | 319.0 | 7,852.8 | 1,157.7 | Reference |
| Before | DuckDB Parquet | 349.0 | 8,111.2 | 1,658.3 | Reference |
| Before | rudb | 384.3 | 4,049.2 | 610.8 | Different group counts |
| After | DuckDB native | 316.0 | 7,810.6 | 1,152.4 | Reference |
| After | DuckDB Parquet | 374.0 | 8,176.7 | 1,648.0 | Reference |
| After | rudb | 386.9 | 4,126.7 | 595.4 | Match |

The time difference between rudb's two runs is less than one percent. DuckDB Parquet varied more between those runs, so the table does not support attributing a small timing change to this fix. Rudb's corrected 10m query uses about half DuckDB Parquet's child CPU and 36 percent of its peak RSS. It is 3 percent slower than DuckDB Parquet's query timer in the final run. At 1m rows, corrected rudb runs query 29 in 48.5 ms, against 83 ms for DuckDB native and 90 ms for DuckDB Parquet.

| 10m suite | Engine and run | Sum of median query time (s) | Sum of median child CPU (s) | Peak child RSS (MiB) | Complete queries |
| --- | --- | ---: | ---: | ---: | ---: |
| Before | DuckDB native | 2.919 | 37.672 | 1,649.6 | 43 |
| Before | DuckDB Parquet | 4.015 | 45.021 | 2,740.3 | 43 |
| Before | rudb | 5.7398 | 48.815 | 1,364.2 | 43 |
| After | DuckDB native | 2.986 | 37.570 | 1,667.1 | 43 |
| After | DuckDB Parquet | 4.050 | 45.035 | 2,732.5 | 43 |
| After | rudb | 5.7638 | 48.948 | 1,361.2 | 43 |

All 43 timed queries completed at 1k, 10k, 100k, 1m, and 10m rows. Every deterministic correctness retest matched in the final 10m audit, including query 29. The earlier 10m audit left query 29 unresolved. The source gate passed on server3, and the focused differential regex test passed.

| Smaller size | Engine | Sum of median query time (s) | Sum of median child CPU (s) | Peak child RSS (MiB) |
| --- | --- | ---: | ---: | ---: |
| 1k | DuckDB native | 0.124 | 1.215 | 53.5 |
| 1k | DuckDB Parquet | 0.151 | 1.270 | 52.8 |
| 1k | rudb corrected | 0.0233 | 0.0597 | 6.4 |
| 10k | DuckDB native | 0.145 | 1.268 | 53.2 |
| 10k | DuckDB Parquet | 0.198 | 1.399 | 57.3 |
| 10k | rudb corrected | 0.0699 | 0.1263 | 9.6 |
| 100k | DuckDB native | 0.326 | 1.520 | 81.3 |
| 100k | DuckDB Parquet | 0.372 | 2.152 | 114.5 |
| 100k | rudb corrected | 0.1959 | 0.7214 | 42.9 |
| 1m | DuckDB native | 0.549 | 3.837 | 305.6 |
| 1m | DuckDB Parquet | 1.600 | 7.998 | 396.5 |
| 1m | rudb corrected | 0.9294 | 5.227 | 196.3 |

A stage profile on the 10m query spent only about 8 ms in the projection around host extraction, while the aggregate accumulated about 3,517 ms and Parquet scan about 1,145 ms across workers. Those sums overlap across threads and are diagnostic work readings, not process elapsed time. The host shortcut remains cheap. The aggregate and decoding paths control the remaining 10m elapsed time and the distance from the requested 10 times faster and 10 times smaller target.
