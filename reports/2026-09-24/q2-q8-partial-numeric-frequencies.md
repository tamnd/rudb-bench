# Q2 through Q8 storage review: keep numeric frequencies partial

The Q8 correction made the query count encoded rows at runtime, but a new native file could still contain a complete column-wide value-to-count table. For low-cardinality `AdvEngineID`, that table was the Q8 grouped result before filtering and sorting. The earlier audit called it a reusable statistic; that description missed that it could reconstruct the full answer. [RuDB #1685](https://github.com/tamnd/rudb/pull/1685) makes every new multi-value numeric frequency synopsis incomplete, independent of table or column name. The writer retains at most two leading entries and sets `omitted_max` to the largest omitted count. Already partial high-cardinality synopses retain their existing entries and bound. A one-value column is already determined by its row count and extrema.

Q2 now uses row count, null count, and an exact leading zero frequency when zero is retained. If zero is omitted, it uses the regular SQL path. Q3 and Q4 still use scalar sums and non-null counts, Q5 and Q6 use distinct counts, and Q7 uses extrema. Q8 reads encoded row values and counts groups when the SQL runs. The writer does not save a complete multi-value numeric grouped result in new files. Existing files can still contain complete synopses, but Q8 does not use them.

Both engines loaded the same Parquet source with the same `CREATE TABLE`, `INSERT`, and `CHECKPOINT` SQL. These are one fresh-process load each, including startup and exit, on a shared host. The 1m and 10m database pairs were on the same temporary memory filesystem. Peak RSS is the whole child process, measured by `wait4`.

| Rows | RuDB load wall | DuckDB load wall | RuDB load CPU | DuckDB load CPU | RuDB peak RSS | DuckDB peak RSS | RuDB file | DuckDB file |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 0.620 s | 0.502 s | 0.314 s | 0.301 s | 34.21 MiB | 44.27 MiB | 1,190,638 B | 1,060,864 B |
| 10k | 1.613 s | 0.982 s | 1.810 s | 1.084 s | 81.73 MiB | 82.20 MiB | 5,104,520 B | 3,944,448 B |
| 1m | 17.789 s | 17.143 s | 48.462 s | 46.166 s | 1,286.54 MiB | 1,629.00 MiB | 212,800,662 B | 525,348,864 B |
| 10m | 78.785 s | 58.126 s | 268.309 s | 191.272 s | 1,851.85 MiB | 4,633.52 MiB | 1,233,164,113 B | 1,845,243,904 B |

At 10m, RuDB used 2.50x less memory while loading, but took 1.36x longer than DuckDB. This storage rule does not fix the load-time gap. The single load samples are not a stable throughput estimate.

All complete Q2 through Q8 CSV outputs from the newly loaded files matched DuckDB at 1k, 10k, 1m, and 10m. Q8 compared each key and count while allowing either order for equal counts. The following medians use 11 alternating fresh-process runs per engine and size. The older RuDB binary used its older native file, so its difference from the new run includes both the code and file. Each DuckDB run used a native file built from the same source rows.

| Query | Rows | Older RuDB wall | New RuDB wall | DuckDB wall | DuckDB / new wall | New RuDB CPU | DuckDB CPU | New RuDB RSS | DuckDB RSS | DuckDB / new RSS |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Q2 | 1k | 3.382 ms | 3.853 ms | 191.406 ms | 49.68x | 3.640 ms | 124.549 ms | 3.88 MiB | 35.15 MiB | 9.06x |
| Q2 | 10k | 5.119 ms | 7.836 ms | 128.793 ms | 16.44x | 7.240 ms | 116.585 ms | 4.00 MiB | 35.27 MiB | 8.82x |
| Q2 | 1m | 4.228 ms | 5.474 ms | 151.942 ms | 27.76x | 4.653 ms | 127.622 ms | 3.88 MiB | 38.40 MiB | 9.90x |
| Q2 | 10m | 3.950 ms | 5.476 ms | 225.520 ms | 41.18x | 5.171 ms | 199.608 ms | 3.88 MiB | 57.62 MiB | 14.85x |
| Q8 | 1k | 5.203 ms | 4.314 ms | 205.804 ms | 47.71x | 4.016 ms | 156.745 ms | 4.38 MiB | 38.17 MiB | 8.72x |
| Q8 | 10k | 7.011 ms | 6.464 ms | 194.473 ms | 30.09x | 6.194 ms | 176.901 ms | 4.38 MiB | 38.15 MiB | 8.71x |
| Q8 | 1m | 6.357 ms | 7.455 ms | 208.262 ms | 27.94x | 6.658 ms | 180.663 ms | 4.50 MiB | 41.41 MiB | 9.20x |
| Q8 | 10m | 42.511 ms | 12.200 ms | 290.892 ms | 23.84x | 12.024 ms | 239.399 ms | 4.88 MiB | 60.30 MiB | 12.36x |

The 10m check was repeated for 51 alternating trials because the short run's older RuDB wall median was dominated by scheduling. The longer run is the useful comparison:

| Query | Engine | Wall | CPU | Peak RSS |
| --- | --- | ---: | ---: | ---: |
| Q2 | Older RuDB | 5.375 ms | 4.430 ms | 3.75 MiB |
| Q2 | New RuDB | 6.295 ms | 5.649 ms | 3.75 MiB |
| Q2 | DuckDB | 256.614 ms | 231.953 ms | 57.68 MiB |
| Q8 | Older RuDB | 14.083 ms | 13.100 ms | 4.88 MiB |
| Q8 | New RuDB | 14.782 ms | 12.906 ms | 4.75 MiB |
| Q8 | DuckDB | 265.094 ms | 296.652 ms | 60.43 MiB |

The new file and query path make Q2 about 17% slower and Q8 about 5% slower in 10m wall time in this run. Against DuckDB, the new Q8 path is 17.93x faster in wall time and uses 12.72x less peak RSS. Those ratios moved substantially in earlier runs on the shared host, so they do not establish a stable 10x wall advantage. Q2 misses the 10x memory target through 1m; Q8 misses it through 1m. The full ClickBench goal remains open.

The release candidate was built from RuDB main `34d6572e` plus this change and has SHA-256 `d8c57dfdb0d3a59e0f6565a8ed9f7cba39285af5df940437669bda930d3d592e`. The older RuDB binary has SHA-256 `365d0cbbe81a0bec42b03595dd7c9cfb069719e19e7d5177ccfd1f7fb577f0aa`. DuckDB v2.0.0-dev84237 has SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The native, database, and CLI suites passed 186, 409, and 30 tests, respectively, and strict Clippy passed. The [raw load and query records](q2-q8-partial-frequency/) contain every process sample. The [Q2 runner](../../scripts/q2-fresh-process.py) and [Q8 runner](../../scripts/q8-fresh-process.py) preserve the measurement method. The operating-system page cache was not cleared.
