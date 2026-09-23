# ClickBench Q1: avoid reading column order summaries for a row count

ClickBench Q1 is `SELECT COUNT(*) FROM hits`. On the native file, rudb already answers the count from stored row metadata. It does not scan ten million rows. The profile instead found fixed planning work: binding a table called `ascending()` on all 105 columns, opening each order summary even though Q1 has no grouping key. In a hot profile, that call cost about 41 microseconds per statement. Distinct-count summaries cost another 7 microseconds, and the rest of parsing, binding, optimization, and physical construction remains.

The engine change reads order summaries only when the statement contains a grouped select block. The sorted-key aggregate still gets its order certificate. `COUNT(*)` no longer reads 105 order summaries merely to bind `hits`.

## Paired native Q1 measurement

The old and changed rudb release binaries ran in alternating fresh processes on the same Linux host and the same native file at each size. There were nine runs per binary and size. Each cell is the median of that cell's runs. The query time is rudb's internal parse-through-execution timer; RSS is `/usr/bin/time -v` maximum resident set size for the process. Every run returned the expected count.

| Rows | Old rudb query | Changed rudb query | Old bind | Changed bind | Old peak RSS | Changed peak RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 327 µs | 271 µs | 128 µs | 76 µs | 13.19 MiB | 13.20 MiB |
| 10,000 | 374 µs | 323 µs | 135 µs | 89 µs | 15.20 MiB | 15.21 MiB |
| 1,000,000 | 363 µs | 291 µs | 152 µs | 78 µs | 12.97 MiB | 12.96 MiB |
| 9,999,750 | 545 µs | 428 µs | 231 µs | 122 µs | 35.98 MiB | 35.75 MiB |

The 10 million row binder median fell by 47%. The full query median fell by 21%. The RSS difference is too small to call a memory improvement.

## DuckDB comparison

DuckDB CLI query timing rounds to whole milliseconds for Q1, so a one-statement timer cannot resolve a 10x claim. Repeating the identical Q1 statement within one process gives enough elapsed time to compare the command paths. These are process wall times, including startup, catalog open, SQL parsing, query execution, and result rendering. The DuckDB runs and rudb runs use their respective native databases holding the same rows. Each batch result is a median of independent processes, nine for DuckDB and five for the changed rudb binary.

| Q1 statements per process | DuckDB native | Changed rudb native | DuckDB time / rudb time |
| ---: | ---: | ---: | ---: |
| 100 | 64.54 ms | 57.48 ms | 1.12x |
| 1,000 | 382.68 ms | 170.72 ms | 2.24x |
| 3,000 | 1,082.28 ms | 323.89 ms | 3.34x |

The 3,000-statement result includes fixed process cost and run-to-run noise. It supports a clear improvement but does not meet the 10x goal. A separate nine-process baseline on the old rudb binary had a 1,082.28 ms DuckDB median and a 373.88 ms rudb median at 3,000 statements. The new and old rudb binaries were also paired directly: the old median was 545.01 ms and the changed median was 323.89 ms at 3,000 statements. The separate sets should not be treated as one precisely paired three-way experiment.

For reference, fresh-process Q1 on the same host at 9,999,750 rows used 22.97 ms wall and 42.45 MiB peak RSS in DuckDB, compared with 23.87 ms wall and 35.95 MiB in the earlier rudb build. Startup and host noise dominate those single-statement wall times. DuckDB's rounded query timer reported 1 ms. The changed rudb median internal query time was 0.428 ms, and the timers have different boundaries and resolution. Neither comparison proves a 10x single-query win.

The next Q1 bottleneck is the repeated parse, bind, optimize, and physical-build path for a count that reads a stored row total. Any further shortcut needs catalog and setting invalidation so an insert, table replacement, or changed SQL setting cannot reuse an old answer or plan. That requires a separate correctness and performance change.

Raw measurements: [paired engine runs](q1-order-summary/ascending-ab.json), [DuckDB batch comparison](q1-order-summary/duckdb-batch.json), and [size ladder](q1-order-summary/latest-ladder.json).
