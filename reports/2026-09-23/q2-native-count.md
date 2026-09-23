# ClickBench Q2: cache the count plan and native frequency summary

ClickBench Q2 is `SELECT COUNT(*) FROM hits WHERE AdvEngineID <> 0`. The native file already has a frequency summary that lets rudb answer this predicate without scanning rows. Profiling showed two repeated costs: each statement parsed and planned the same SQL, and each use of a stored frequency summary read and decoded its file section again. The engine now reuses the plan for this narrow literal predicate and decodes each stored frequency section once per open native table.

The final measured rudb build is commit `e944dd48` on main `5d199918`, release CLI SHA-256 `b673aa3326c67a7bf9d67fd01be5799628d19daf9044cc62eff2cb5b3b8c7d1f`. The comparison main binary is from `923dd55c`, SHA-256 `7fec21cc3ae487283cfc2e39d68d3b1e1d99a02984fccd261314695915574e38`. DuckDB is `v2.0.0-dev84237` (`cc7e7bac7f`), CLI SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`.

## Repeated Q2 statements

Each process ran 3,000 copies of the same SQL against a native database. The 1k, 10k, and 1m cases used five alternating independent processes per engine; 10m used nine. Both CLIs ran with `-readonly -noheader -csv -f`. Every output row was checked: 6, 62, 6,284, and 63,365 respectively. Wall time includes process startup, catalog open, all SQL calls, and output. CPU time is user plus system time from `wait4`, and RSS is its peak resident set. Cells are medians; speedups divide the wall-time medians.

| Rows | DuckDB wall | rudb main wall | rudb patch wall | DuckDB / patch | DuckDB CPU | rudb patch CPU | DuckDB peak RSS | rudb patch peak RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 1,541 ms | 305 ms | 99.5 ms | 15.5x | 1,886 ms | 99.3 ms | 57.11 MiB | 13.76 MiB |
| 10,000 | 1,570 ms | 703 ms | 106.5 ms | 14.7x | 1,897 ms | 106.3 ms | 57.00 MiB | 15.99 MiB |
| 999,975 | 1,914 ms | 263 ms | 104.7 ms | 18.3x | 3,629 ms | 104.5 ms | 54.14 MiB | 13.76 MiB |
| 9,999,750 | 2,891 ms | 342 ms | 161.1 ms | 17.9x | 17,114 ms | 160.9 ms | 83.41 MiB | 35.86 MiB |

The repeated-statement time target is met at all four sizes. The memory target is not met: rudb's peak RSS is 2.3x to 4.2x lower than DuckDB's in these runs, short of 10x. This result applies to repeated SQL in one process, not to cold single-query latency.

At 10m rows, shorter batches show how startup and catalog open affect the ratio:

| Statements per process | DuckDB wall | rudb main wall | rudb patch wall | DuckDB / patch | DuckDB peak RSS | rudb patch peak RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 34.06 ms | 24.90 ms | 24.78 ms | 1.37x | 64.38 MiB | 35.84 MiB |
| 100 | 139.54 ms | 35.06 ms | 28.88 ms | 4.83x | 73.64 MiB | 36.00 MiB |
| 1,000 | 1,440.84 ms | 165.65 ms | 68.77 ms | 21.0x | 82.29 MiB | 36.00 MiB |
| 3,000 | 2,890.64 ms | 342.19 ms | 161.10 ms | 17.9x | 83.41 MiB | 35.86 MiB |

## Cause and ablation

The 10m profiler reported a stored-summary aggregate with `rows_in=0`: the execution path was already constant in table size. Parse, bind, optimize, and physical-plan construction dominated its first statement. The Q1 cache did not admit a filter, so Q2 repeated those phases on every call. The new cache only admits a direct native-table `COUNT(*)` with a written `column <> numeric literal` filter. It still checks catalog generation and setting revision, and every call executes the plan.

At 10k rows, the native reader also reread and decoded a 30,192-byte frequency section during physical planning and execution. A three-statement `strace -e pread64` showed 12 reads at the section offset before the reader cache and one after it. A four-way ablation with seven alternating independent processes per engine separates the two fixes:

| 10k rows, 3,000 statements | Median wall | Median peak RSS |
| --- | ---: | ---: |
| rudb main | 684.7 ms | 16.66 MiB |
| rudb with plan reuse only | 512.2 ms | 16.09 MiB |
| rudb with plan reuse and summary cache | 102.6 ms | 16.06 MiB |
| DuckDB native | 1,518.2 ms | 56.88 MiB |

The reader cache is shared by clones of one open table. Its regression test checks that a stored synopsis is absent from the cache before the first request, present afterward, and shared by a clone. The filtered-count regression checks a cache hit, setting invalidation, and a changed count after an insert. The native library tests and Rust lint checks pass.

Raw `wait4` measurements: [size ladder](q2-native-count/size-ladder.json), [10m batch lengths](q2-native-count/ten-million-batches.json), and [10k ablation](q2-native-count/ten-thousand-ablation.json).
