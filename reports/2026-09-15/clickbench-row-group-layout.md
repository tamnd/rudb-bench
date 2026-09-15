# ClickBench sample row-group layout

The 1 million row development sample previously had nine Parquet row groups, with up to 124,263 rows in one group. q23 keeps decoded string pages from several groups live at once. Rewriting the same rows with smaller Snappy row groups tests the storage boundary while all engines read the same source file. This is a development sample comparison, not an engine-only speedup against an unchanged file.

Ten fresh-process q23 runs at each layout showed a speed and memory knee near 8,192 rows per group.

| Row-group target | rudb q23 median wall | Median CPU | Peak RSS |
| ---: | ---: | ---: | ---: |
| Original, up to 124k | 70.8 ms | 0.477 s | 284.7 MiB |
| 32k | 38.6 ms | 0.612 s | 265.5 MiB |
| 16k | 35.1 ms | 0.587 s | 163.2 MiB |
| 8k | 33.8 ms | 0.546 s | 120.1 MiB |
| 4k | 43.6 ms | 0.522 s | 96.6 MiB |
| 2k | 79.0 ms | 0.653 s | 88.2 MiB |

The full 43-query audit used five hot runs per query in fresh processes. The table reports the sum of per-query medians and the highest per-query peak RSS.

| Rows | Engine | Query sum | CPU sum | Peak RSS |
| ---: | --- | ---: | ---: | ---: |
| 1k | DuckDB native | 0.132 s | 1.297 s | 53.27 MiB |
| 1k | DuckDB Parquet | 0.171 s | 1.405 s | 52.99 MiB |
| 1k | rudb | 0.0237 s | 0.0555 s | 6.20 MiB |
| 10k | DuckDB native | 0.147 s | 1.287 s | 53.36 MiB |
| 10k | DuckDB Parquet | 0.202 s | 1.452 s | 57.75 MiB |
| 10k | rudb | 0.0756 s | 0.126 s | 8.92 MiB |
| 100k | DuckDB native | 0.324 s | 1.521 s | 82.79 MiB |
| 100k | DuckDB Parquet | 0.376 s | 2.157 s | 113.71 MiB |
| 100k | rudb | 0.2265 s | 0.804 s | 42.66 MiB |
| 1m | DuckDB native | 0.544 s | 3.812 s | 306.04 MiB |
| 1m | DuckDB Parquet | 1.633 s | 8.078 s | 358.76 MiB |
| 1m | rudb | 1.0122 s | 6.814 s | 210.84 MiB |

All deterministic checks matched DuckDB. At 1 million rows, rudb is 1.61 times faster than DuckDB reading the same Parquet file and uses 1.70 times less peak memory. It is still 1.86 times slower than DuckDB native and uses 1.45 times less peak memory. The requested 10x target is not reached.

Compared with the earlier wide-row-group file, rudb's query sum drops from 1.417 to 1.012 seconds and its peak RSS drops from 282.5 to 210.8 MiB. Its CPU sum rises from 6.140 to 6.814 seconds, because more groups require more metadata and reader setup. DuckDB Parquet also changes substantially on the new file, rising from 0.877 to 1.633 seconds. DuckDB native is loaded once and therefore sees a different storage path.

The benchmark generator now names these samples `*-snappy-rg8k.parquet` and writes `ROW_GROUP_SIZE 8192`, so it cannot silently reuse older wide-group samples. The result is a format/layout finding. Further engine optimization should measure against both the original and the 8k file rather than crediting a source-file change to rudb alone.
