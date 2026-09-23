# ClickBench Q4: reuse the native average plan

ClickBench Q4 is `SELECT AVG(UserID) FROM hits`. Rudb already answers it from native column metadata without scanning rows. The 10m profile showed `Aggregate stored summary` with `rows_in=0`. Repeated statements still parsed, bound, and optimized the same SQL because the native plan cache did not admit a single `AVG(column)`. The engine now reuses that direct aggregate plan while the table and settings are unchanged.

The comparison main binary is commit `4cfca64b`, SHA-256 `64fc7a5932110de2072cc307056919d76a91f1cf556ce62f7ae591f5237a39a2`. The measured patch is that same commit plus the Q4 change, release CLI SHA-256 `9492d963c63bbc84695778485d085e07f162662ce89876be3b4ae99d078fb746`. DuckDB is `v2.0.0-dev84237` (`cc7e7bac7f`), CLI SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The engine change was rebased onto `074a14a7`, which changes report printing, after these runs; the aggregate-cache test was rerun on that base.

## Repeated Q4 statements

Each process ran 3,000 copies of the exact same Q4 SQL against a native database. The 1k, 10k, and 1m cases used five alternating independent processes per engine; 10m used nine. Both CLIs ran with `-readonly -noheader -csv -f`. Every CSV output was checked byte for byte against DuckDB's answer. Wall time includes process startup, catalog open, all SQL calls, and output. CPU is user plus system time from `wait4`, and RSS is its peak resident set. Cells are medians; speedups divide the wall-time medians.

| Rows | DuckDB wall | rudb main wall | rudb patch wall | DuckDB / patch | DuckDB CPU | rudb patch CPU | DuckDB peak RSS | rudb patch peak RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 2,220 ms | 242 ms | 133.3 ms | 16.7x | 2,595 ms | 132.9 ms | 61.68 MiB | 13.89 MiB |
| 10,000 | 1,535 ms | 364 ms | 117.0 ms | 13.1x | 2,089 ms | 116.8 ms | 63.88 MiB | 16.10 MiB |
| 999,975 | 2,176 ms | 231 ms | 96.3 ms | 22.6x | 5,582 ms | 96.2 ms | 66.21 MiB | 14.73 MiB |
| 9,999,750 | 7,839 ms | 282 ms | 137.1 ms | 57.2x | 135,233 ms | 136.9 ms | 126.99 MiB | 36.35 MiB |

The repeated-statement time target is met at all four sizes. The peak RSS target is not met: rudb uses about 3.5x to 4.5x less RSS than DuckDB in these runs, short of 10x. DuckDB's CPU time is much higher than its wall time at 10m because it uses multiple cores. Rudb answers Q4 from stored metadata, so its measured CPU and wall times are close.

At 10m rows, nine alternating runs per engine and batch length show the cold limit:

| Statements per process | DuckDB wall | rudb main wall | rudb patch wall | DuckDB / patch | DuckDB peak RSS | rudb patch peak RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 50.91 ms | 24.70 ms | 24.96 ms | 2.04x | 111.74 MiB | 36.16 MiB |
| 100 | 322.20 ms | 33.57 ms | 29.02 ms | 11.1x | 118.70 MiB | 36.36 MiB |
| 1,000 | 2,661.59 ms | 112.60 ms | 63.24 ms | 42.1x | 125.79 MiB | 36.16 MiB |
| 3,000 | 7,839.47 ms | 282.36 ms | 137.08 ms | 57.2x | 126.99 MiB | 36.35 MiB |

The first patched 10m statement took 82 microseconds to parse, 78 to bind, 40 to optimize, 96 to build the physical plan, and 28 to execute in one instrumented run. The second statement reported zero parse, bind, and optimize time, 18 microseconds for physical planning, and 4 for execution. Both used the stored summary and scanned zero rows. These per-statement counters exclude process startup and catalog open, so the wall table is the full-process comparison.

The new cache entry has the same exact-SQL, catalog-generation, and setting-revision checks as Q1 through Q3. It accepts only direct `AVG(column)` over one immutable native table, with no filter, grouping, distinct aggregate, or computed argument. The regression test checks null handling, a cache hit, rejection of `AVG(i + 1)`, and setting and insert invalidation. The Q1 through Q3 cache checks still pass. Formatting and Clippy with warnings denied pass.

Raw per-process measurements and checked answers: [paired Q4 runs](q4-native-plan-cache/paired.json).
