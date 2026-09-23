# ClickBench Q1: reuse the plan for an exact native count

ClickBench Q1 is `SELECT COUNT(*) FROM hits`. The native engine already answers it from stored row metadata. A profile of repeated Q1 statements showed that parsing, binding, and optimizing the same query cost much more than reading the count. A small plan cache now reuses the optimized plan only for a direct, unfiltered `COUNT(*)` over one immutable native table. Every call still runs the plan and returns a fresh result and metrics document. The cache key is the exact SQL text, catalog generation, and successful-setting revision. An insert changes the catalog generation, and a `SET` or `RESET` changes the setting revision.

The measured rudb build is 0.4.12 on main `c56b83bb` plus the native-count cache. Its release CLI SHA-256 is `e55c6d1cececf51904936f7ced1b73ff72d26fc5da1f79d6431586b709edfe55`. DuckDB is `v2.0.0-dev84237` (`cc7e7bac7f`), CLI SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The engine change was rebased again on `484a26a5`, which changes native load writing, after these runs; its count-cache test was rerun on that base.

## Native Q1 with an open database

The same SQL was run 10,000 times in each process. DuckDB and rudb read native databases with the same row counts on one Linux host. Both CLIs used `-readonly -noheader -csv -f`, and the output was checked against the expected count for every statement. Process wall time includes startup, opening the catalog, all SQL calls, and rendering all rows. Peak RSS is the process maximum from `wait4`. The 1k, 10k, and 1m rows have five alternating runs per engine; 10m has nine. Cells are medians, so the ratio is the ratio of medians.

| Rows | DuckDB native wall | rudb native wall | DuckDB / rudb | DuckDB peak RSS | rudb peak RSS |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 3,438 ms | 286 ms | 12.0x | 49.80 MiB | 14.26 MiB |
| 10,000 | 3,448 ms | 294 ms | 11.7x | 50.31 MiB | 16.08 MiB |
| 999,975 | 3,935 ms | 304 ms | 13.0x | 51.31 MiB | 14.26 MiB |
| 9,999,750 | 3,522 ms | 310 ms | 11.3x | 51.31 MiB | 36.32 MiB |

The repeated-Q1 time target is met at all four sizes. The process RSS target of 10x less than DuckDB is not met. The 10m rudb process used about 1.4x less RSS than DuckDB in this batch.

The batch length matters because opening a database is fixed work. At 10m rows, the nine-run medians were:

| Statements per process | DuckDB native | rudb native | DuckDB / rudb |
| ---: | ---: | ---: | ---: |
| 100 | 63.81 ms | 28.06 ms | 2.27x |
| 1,000 | 379.67 ms | 54.67 ms | 6.94x |
| 3,000 | 1,069.48 ms | 111.55 ms | 9.59x |
| 10,000 | 3,522.34 ms | 310.49 ms | 11.34x |

The hot per-statement slope is about 29 microseconds for rudb and 350 microseconds for DuckDB between 3,000 and 10,000 statements. The corresponding slope ratio is about 12x. These values describe repeated statements in one process, not a fresh process per query.

## Fresh single statements

Nine alternating fresh-process runs per engine and size gave the following medians. The cache is cold in every run, so the first query still parses and binds. Startup and catalog open dominate the wall time.

| Rows | DuckDB native wall | rudb native wall | DuckDB peak RSS | rudb peak RSS |
| ---: | ---: | ---: | ---: | ---: |
| 1,000 | 22.85 ms | 7.50 ms | 39.56 MiB | 13.83 MiB |
| 10,000 | 22.23 ms | 14.01 ms | 39.53 MiB | 15.83 MiB |
| 999,975 | 22.45 ms | 7.68 ms | 40.07 MiB | 13.70 MiB |
| 9,999,750 | 24.09 ms | 25.13 ms | 40.80 MiB | 36.08 MiB |

A fresh 10m Q1 process is roughly tied on wall time, so the 10x result does not apply to cold single-query latency. The earlier [order-summary profile](q1-order-summary.md) details that first-query path and its DuckDB comparison. The first uncached query now loads the native catalog and builds a plan; a repeated call reuses the plan. Its metrics show zero parse, bind, and optimize time on a hit because those phases did not run. Cache lookup and setting/context checks are outside those phase counters, so the whole-process wall measurement above is the fair speed comparison.

The engine regression test checks the expected count and a cache hit, then changes a setting and inserts a row. Both operations force binding again, and the count after insert changes from two to three. A filtered count and a table with uncheckpointed rows do not enter the cache.

Raw measurements: [10m batch lengths](q1-native-plan-cache/batch-10m.json), [10,000-statement size ladder](q1-native-plan-cache/batch-ladder.json), and [fresh single-query ladder](q1-native-plan-cache/single-ladder.json).
