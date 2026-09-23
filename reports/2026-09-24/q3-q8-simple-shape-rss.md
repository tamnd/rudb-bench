# Q3 and Q8 cold paths without the full SQL parser

The native fast CSV paths for Q3 and Q8 already derive their answers from reusable column statistics at query time, but they called the full SQL AST parser to recognize two small statement shapes. [RuDB #1638](https://github.com/tamnd/rudb/pull/1638) replaces only that cold-path recognition with strict unquoted shape checks over the requested table and column names. Reserved words, aliases, additional clauses, different expressions, and unsupported metadata still use the regular SQL path. No query answer or new statistic is stored in the native file.

## Merged binary

The shipping RuDB binary was built from merged commit `218f9c4e`, SHA-256 `c72970f7e875051b4d7d65ca5a6631e2d21f66b86e7b3a423d45a21158701955`. Each row below is the median of 11 alternating fresh-process pairs against DuckDB on the same SQL and native files loaded from the same Parquet source. Every complete CSV result matched DuckDB. The `wait4` helper measures the child from spawn to exit, including startup, file open, SQL, output, and exit. The operating-system page cache was not cleared.

| Query | Rows | RuDB wall | DuckDB wall | DuckDB / RuDB wall | RuDB peak RSS | DuckDB peak RSS | DuckDB / RuDB RSS |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Q2 | 1k | 5.601 ms | 189.625 ms | 33.85x | 3.75 MiB | 35.64 MiB | 9.50x |
| Q2 | 10k | 4.495 ms | 191.837 ms | 42.68x | 3.75 MiB | 35.64 MiB | 9.50x |
| Q2 | 1m | 7.398 ms | 213.526 ms | 28.86x | 3.75 MiB | 38.64 MiB | 10.31x |
| Q2 | 10m | 6.706 ms | 314.047 ms | 46.83x | 3.75 MiB | 57.92 MiB | 15.45x |
| Q3 | 1k | 5.751 ms | 201.895 ms | 35.11x | 3.88 MiB | 36.02 MiB | 9.30x |
| Q3 | 10k | 7.875 ms | 167.424 ms | 21.26x | 3.88 MiB | 36.02 MiB | 9.30x |
| Q3 | 1m | 5.377 ms | 249.497 ms | 46.40x | 3.88 MiB | 41.64 MiB | 10.75x |
| Q3 | 10m | 7.169 ms | 473.429 ms | 66.03x | 3.88 MiB | 79.05 MiB | 20.40x |
| Q8 | 1k | 4.248 ms | 239.941 ms | 56.48x | 3.88 MiB | 38.52 MiB | 9.94x |
| Q8 | 10k | 4.925 ms | 203.225 ms | 41.26x | 3.88 MiB | 38.52 MiB | 9.94x |
| Q8 | 1m | 4.124 ms | 252.361 ms | 61.20x | 3.88 MiB | 41.64 MiB | 10.75x |
| Q8 | 10m | 5.029 ms | 338.565 ms | 67.33x | 3.88 MiB | 60.80 MiB | 15.69x |

All three queries still miss the 10x peak-RSS target at 1k and 10k. The eight-CPU shared host had a load average near 40 after the run, so the wall-time ratios describe that contended run, not quiet-host latency. The [merged-binary raw records](q3-q8-simple-shapes/merged/) contain all 264 child measurements, including user-plus-system CPU time. DuckDB was v2.0.0-dev84237, binary SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`.

## Isolated comparison before final rebase

The first comparison was made before the final rebase. Each row below is the median of 11 rotating fresh-process trials of the exact parent-main binary, the candidate binary, and DuckDB. All three opened native files loaded from the same Parquet rows and ran identical SQL. Every complete CSV result matched DuckDB. The same `wait4` helper and page-cache conditions apply. This run isolates the parser change from the later allocator change on main.

| Query | Rows | Parent RuDB wall | Candidate RuDB wall | DuckDB wall | Parent RuDB peak RSS | Candidate RuDB peak RSS | DuckDB peak RSS | DuckDB / candidate RSS |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Q3 | 1k | 4.483 ms | 3.943 ms | 229.754 ms | 4.50 MiB | 3.88 MiB | 36.02 MiB | 9.30x |
| Q3 | 10k | 12.913 ms | 5.245 ms | 196.960 ms | 4.38 MiB | 3.88 MiB | 35.89 MiB | 9.26x |
| Q3 | 1m | 7.479 ms | 4.422 ms | 292.048 ms | 4.38 MiB | 3.88 MiB | 41.65 MiB | 10.75x |
| Q3 | 10m | 5.089 ms | 4.339 ms | 366.641 ms | 4.50 MiB | 3.88 MiB | 78.80 MiB | 20.34x |
| Q8 | 1k | 5.218 ms | 4.338 ms | 303.719 ms | 4.38 MiB | 3.88 MiB | 38.39 MiB | 9.91x |
| Q8 | 10k | 5.891 ms | 5.376 ms | 247.495 ms | 4.38 MiB | 3.88 MiB | 38.64 MiB | 9.97x |
| Q8 | 1m | 5.779 ms | 4.003 ms | 272.783 ms | 4.38 MiB | 3.88 MiB | 41.52 MiB | 10.72x |
| Q8 | 10m | 5.357 ms | 4.236 ms | 335.193 ms | 4.38 MiB | 3.88 MiB | 60.80 MiB | 15.69x |

Avoiding the AST parser saves 0.50 to 0.62 MiB on Q3 and 0.50 MiB on Q8 in this paired run. It does not meet the 10x memory target at 1k or 10k. The host has eight CPUs and had a load average near 50 during this work, so these timings do not establish a quiet-host speed improvement. The raw records include child user-plus-system CPU time.

Q2 did not take either changed recognition path, so it ran as a control. Its candidate peak RSS was 3.75 MiB at all four sizes. This binary and host clear 10x against DuckDB at 1m and 10m:

| Rows | Parent RuDB wall | Candidate RuDB wall | DuckDB wall | Parent RuDB peak RSS | Candidate RuDB peak RSS | DuckDB peak RSS | DuckDB / candidate RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 5.977 ms | 3.940 ms | 198.124 ms | 3.88 MiB | 3.75 MiB | 35.53 MiB | 9.47x |
| 10k | 8.569 ms | 6.397 ms | 195.722 ms | 3.75 MiB | 3.75 MiB | 35.52 MiB | 9.47x |
| 1m | 4.703 ms | 4.107 ms | 251.727 ms | 3.75 MiB | 3.75 MiB | 38.65 MiB | 10.31x |
| 10m | 5.257 ms | 5.281 ms | 321.740 ms | 3.75 MiB | 3.75 MiB | 57.93 MiB | 15.45x |

The parent is source commit `b9be2042`, release binary SHA-256 `fcb749ea5a94942cc834c3c9ddd32b94de5b6a612722588cbba49c17b8920e64`. The candidate is source commit `de978fe2`, release binary SHA-256 `46afbc35e98945ffcaeda0f86d5ae8112b9edfccf551eec3b95b57761eee07fc`. The [isolated raw records](q3-q8-simple-shapes/rebased/) contain all 396 child measurements. The [pre-rebase records](q3-q8-simple-shapes/pre-rebase/) are kept separately because they measure different RuDB binaries. The [Q2](../../scripts/q2-fresh-process.py), [Q3](../../scripts/q3-fresh-process.py), and [Q8](../../scripts/q8-fresh-process.py) scripts ran the merged comparison. The [three-way Q3](../../scripts/q3-fresh-process-three-way.py) and [rotating Q2](../../scripts/allocator-fresh-process.py) scripts ran the isolated comparison. The 1m source has 999,975 rows; the 10m source has exactly 10,000,000 rows.
