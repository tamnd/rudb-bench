# ClickBench Q3 in fresh processes

Q3 is `SELECT SUM(AdvEngineID), COUNT(*), AVG(ResolutionWidth) FROM hits`. Each invocation opens one native database, executes this one statement, prints CSV, and exits. Rudb and DuckDB use the same SQL and row counts. The resource helper starts and waits for the measured child with `wait4`, so the reported wall time, CPU time, and peak resident memory belong to that child process. The benchmark alternates engine order for 51 runs per size and reports medians. Every run must return the expected CSV row.

The native rudb file now carries exact signed-integer sums and non-null counts in its small checksummed catalog. The query verifies the table directory checksum, reads those values and the table row count, and computes `AVG` from the exact integer sum and non-null count. It does not decode the large table directory or scan column pages. Existing native files were copied and certified without rewriting table data. The pre-change baseline uses the same source files before the new certificate and the same measurement method.

| Rows | Rudb wall ms | DuckDB wall ms | DuckDB / rudb | Rudb CPU ms | DuckDB CPU ms | Rudb peak MiB | DuckDB peak MiB | DuckDB / rudb RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 1.270 | 23.586 | 18.6x | 1.246 | 27.834 | 5.79 | 40.52 | 7.0x |
| 10k | 1.660 | 23.686 | 14.3x | 1.623 | 27.930 | 5.79 | 40.52 | 7.0x |
| 1m | 1.236 | 25.140 | 20.3x | 1.206 | 32.649 | 5.66 | 43.84 | 7.8x |
| 10m | 2.166 | 35.037 | 16.2x | 2.140 | 83.795 | 5.79 | 87.00 | 15.0x |

| Rows | Rudb wall before ms | Rudb wall after ms | Rudb speedup | Rudb peak before MiB | Rudb peak after MiB | Rudb RSS reduction |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 6.987 | 1.270 | 5.5x | 14.85 | 5.79 | 2.6x |
| 10k | 13.643 | 1.660 | 8.2x | 16.62 | 5.79 | 2.9x |
| 1m | 7.336 | 1.236 | 5.9x | 14.85 | 5.66 | 2.6x |
| 10m | 24.241 | 2.166 | 11.2x | 37.11 | 5.79 | 6.4x |

Rudb meets the 10x wall-time target at every size and the 10x peak-RSS target at 10m. The small-file RSS ratios are 7.0x to 7.8x; their roughly 5.7 MiB rudb process footprint is the remaining limit for this query.

Expected CSV rows were `37,1000,1503.928`, `627,10000,1517.7663`, `74434,999975,1514.0342458561463`, and `732801,9999750,1513.4569152228805` in size order. Both engines returned the same row on every run.

The release build ran on `gpc` with DuckDB v2.0.0-dev84237 (`cc7e7bac7f`). The [runner](../../scripts/q3-fresh-process.py) and [resource helper](../../scripts/measure-child.c) reproduce the measurement. Raw samples are in [q3-fresh-process](q3-fresh-process/), with the earlier samples in its `baseline` subdirectory. The source databases are native rudb and native DuckDB files, not Parquet scans.
