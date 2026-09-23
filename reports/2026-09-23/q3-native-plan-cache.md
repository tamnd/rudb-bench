# ClickBench Q3: reuse the native summary aggregate plan

ClickBench Q3 is `SELECT SUM(AdvEngineID), COUNT(*), AVG(ResolutionWidth) FROM hits`. Rudb already answers all three aggregates from native metadata without scanning rows. The 10m profile showed an aggregate with `detail=stored summary` and `rows_in=0`. A repeated statement still parsed, bound, and optimized the same SQL because the existing native plan cache admitted only Q1 and Q2 counts. The engine now reuses this direct three-aggregate plan while the table and settings are unchanged.

The comparison main binary is commit `97a146a2`, SHA-256 `2483b895b75d03f77509472b4a28d330b40b3c8f61aae641a94485415e63339f`. The measured patch is that same commit plus the Q3 change, release CLI SHA-256 `a8661df960e044c7785a70c7554c60da6238cd7741c5bd430864610cb7cb7dd6`. DuckDB is `v2.0.0-dev84237` (`cc7e7bac7f`), CLI SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The engine change was rebased onto `9bf580db`, which changes native substring `LIKE` pruning, after these runs; the aggregate-cache test was rerun on that base.

## Repeated Q3 statements

Each process ran 3,000 copies of the exact same Q3 SQL against a native database. The 1k, 10k, and 1m cases used five alternating independent processes per engine; 10m used nine. Both CLIs ran with `-readonly -noheader -csv -f`. Every CSV output was checked byte for byte against DuckDB's answer. The count fields were 1,000, 10,000, 999,975, and 9,999,750 respectively. Wall time includes process startup, catalog open, all SQL calls, and output. CPU is user plus system time from `wait4`, and RSS is its peak resident set. Cells are medians; speedups divide the wall-time medians.

| Rows | DuckDB wall | rudb main wall | rudb patch wall | DuckDB / patch | DuckDB CPU | rudb patch CPU | DuckDB peak RSS | rudb patch peak RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 2,008 ms | 288 ms | 102.2 ms | 19.7x | 2,295 ms | 102.1 ms | 63.07 MiB | 13.95 MiB |
| 10,000 | 2,641 ms | 321 ms | 140.0 ms | 18.9x | 3,094 ms | 139.9 ms | 67.97 MiB | 15.78 MiB |
| 999,975 | 4,073 ms | 346 ms | 120.6 ms | 33.8x | 9,752 ms | 120.4 ms | 66.04 MiB | 14.45 MiB |
| 9,999,750 | 5,335 ms | 391 ms | 182.2 ms | 29.3x | 70,155 ms | 182.1 ms | 104.09 MiB | 36.25 MiB |

The repeated-statement time target is met at all four sizes. The peak RSS target is not met: rudb uses about 2.9x to 4.6x less RSS than DuckDB in these runs, short of 10x. The native summary makes the answer independent of row count, but opening the file and starting the process are still fixed costs.

At 10m rows, nine alternating runs per engine and batch length show the cold limit:

| Statements per process | DuckDB wall | rudb main wall | rudb patch wall | DuckDB / patch | DuckDB peak RSS | rudb patch peak RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 38.33 ms | 26.28 ms | 26.28 ms | 1.46x | 86.57 MiB | 36.00 MiB |
| 100 | 245.32 ms | 38.79 ms | 31.00 ms | 7.91x | 91.55 MiB | 36.02 MiB |
| 1,000 | 1,848.18 ms | 151.79 ms | 79.97 ms | 23.1x | 100.56 MiB | 36.01 MiB |
| 3,000 | 5,335.27 ms | 391.00 ms | 182.25 ms | 29.3x | 104.09 MiB | 36.25 MiB |

The first patched 10m statement took 89 microseconds to parse, 76 to bind, 43 to optimize, 126 to build the physical plan, and 27 to execute in one instrumented run. The second statement reported zero parse, bind, and optimize time, 23 microseconds for physical planning, and 5 for execution. Both used the stored summary and scanned zero rows. These per-statement counters exclude process startup and catalog open, so the wall table is the full-process comparison.

The new cache entry has the same exact-SQL, catalog-generation, and setting-revision checks as Q1 and Q2. It accepts only direct `SUM(column), COUNT(*), AVG(column)` over one immutable native table, with no filter, grouping, distinct aggregate, or computed aggregate argument. The regression test checks a cache hit, rejects a computed `SUM(i + 1)`, and checks setting and insert invalidation. The Q1 and Q2 cache-hit checks still pass. Rust library tests, formatting, and Clippy with warnings denied pass.

Raw per-process measurements and checked answers: [paired Q3 runs](q3-native-plan-cache/paired.json).
