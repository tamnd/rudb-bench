# Native frequency spans and Q9 startup

**Correction:** The first Q9 table below used RuDB’s one-statement native CSV route. That route recognizes this SQL shape and computes grouped distinct counts directly from a row-preserving projection, bypassing the normal parser, planner, and executor. The file does not store the grouped answer, but this path is specific to this query shape. The original comparison must not be used as evidence that the regular SQL engine is 8x to 15x faster than DuckDB. The regular-engine measurements follow immediately below.

Opening a native table used to decode every column's frequency synopsis to find where the next one began. Q9 does not ask for those statistics, but on a 10k file the directory walk was a large part of its fresh-process cost. [RuDB #1786](https://github.com/tamnd/rudb/pull/1786) writes a checked byte length and entry count before each synopsis. A table open can skip unused payloads; a request for that column decodes and validates its complete payload. This works for any table and any query. It stores reusable column facts, not a SQL result or grouped answer. The reader still opens older files with the previous directory walk.

The query comparison used binaries built from the same RuDB 0.4.30 commit, one with the span change and one without it. Both loaded the same Parquet export using the same `CREATE TABLE`, `INSERT`, and `CHECKPOINT` SQL, then built the existing row-preserving Q9 run projection. The source export came from the corresponding DuckDB native table and was not timed. The 1m label contains 999,975 rows; the other labels contain 1,000, 10,000, and 10,000,000 rows. Every Q9 trial started a new process, alternated order, and checked the complete answer against DuckDB. `wait4` recorded wall time, user plus system CPU time, and whole-process peak RSS. The OS page cache was warm and not cleared. This was a busy shared six-CPU host, so wall medians are observations under contention, not quiet-host guarantees.

For the corrected run, both RuDB binaries received `--set threads=6`. This equals the six-CPU host's default thread count and prevents the CLI one-statement route; DuckDB reported its default `threads` setting as 6. The **SQL statement was identical** in all three cases. Each of the following values is the median of 21 fresh processes. The same native files, DuckDB files, expected answers, warm page cache, process resource collector, and alternating order were used. All 252 answers matched DuckDB in full and were sorted by descending count.

| Rows | Old RuDB engine wall | Span RuDB engine wall | DuckDB wall | DuckDB / span wall | Span RuDB CPU | DuckDB CPU | Span RuDB RSS | DuckDB RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 40.868 ms | 41.505 ms | 273.853 ms | 6.60x | 10.769 ms | 87.995 ms | 10.50 MiB | 36.99 MiB |
| 10k | 118.386 ms | 52.813 ms | 262.720 ms | 4.97x | 14.185 ms | 91.624 ms | 11.25 MiB | 37.86 MiB |
| 1m | 229.191 ms | 195.245 ms | 462.907 ms | 2.37x | 131.621 ms | 289.726 ms | 56.00 MiB | 80.75 MiB |
| 10m | 320.855 ms | 328.837 ms | 807.515 ms | 2.46x | 311.674 ms | 746.362 ms | 73.12 MiB | 135.50 MiB |

The span file beat the old file on wall time in 12/21, 21/21, 13/21, and 9/21 paired trials at 1k, 10k, 1m, and 10m rows. Only the 10k gain is stable. At 10m the candidate's median wall time and CPU time were slightly worse than the old format. RuDB's normal SQL path remains far short of the 10x speed and 10x RSS targets against DuckDB. The shared host was heavily contended: total CPU time is less sensitive to scheduling than wall time, but it still includes runtime overhead. The [corrected raw trials](q9-frequency-spans/) retain all per-process wall, CPU, RSS, and I/O records.

The earlier one-statement CSV measurements remain below as a record of the file-open experiment. They exercise a shape-specific CLI optimization and are **not** the regular-engine comparison.

| Rows | Trials | RuDB old wall | RuDB spans wall | DuckDB wall | DuckDB / spans wall | RuDB spans RSS | DuckDB RSS | DuckDB / spans RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 21 | 10.622 ms | 7.792 ms | 115.113 ms | 14.77x | 6.250 MiB | 37.859 MiB | 6.06x |
| 10k | 51 | 33.601 ms | 9.345 ms | 119.813 ms | 12.82x | 6.375 MiB | 38.734 MiB | 6.08x |
| 1m | 21 | 28.849 ms | 23.544 ms | 287.821 ms | 12.22x | 8.875 MiB | 81.863 MiB | 9.22x |
| 10m | 51 | 48.699 ms | 47.961 ms | 405.196 ms | 8.45x | 11.625 MiB | 133.523 MiB | 11.49x |

| Rows | RuDB old CPU | RuDB spans CPU | DuckDB CPU | Paired span minus old wall | Paired span minus old CPU | Span wall wins |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 9.603 ms | 7.187 ms | 84.872 ms | -3.094 ms | -2.238 ms | 18/21 |
| 10k | 27.826 ms | 8.422 ms | 91.422 ms | -23.840 ms | -19.606 ms | 51/51 |
| 1m | 32.965 ms | 28.578 ms | 294.063 ms | -3.533 ms | -3.891 ms | 13/21 |
| 10m | 79.847 ms | 75.267 ms | 743.125 ms | -1.674 ms | -2.328 ms | 29/51 |

For the direct CLI route, the 10k gain was strong across all paired runs. The 10m wall gain was small and did not meet the 10x DuckDB target. The 1k, 10k, and 1m RSS ratios also remain below 10x. The new 10k native file is 1,680 bytes larger than the old file, which is the added span headers. The 1m and 10m files differ slightly in size because parallel loads can place rows differently; those results should not be read as byte-for-byte format-only comparisons. Native sizes include the Q9 projection; DuckDB sizes are native tables loaded from the same Parquet rows.

| Rows | Old RuDB file | Span RuDB file | DuckDB file |
| ---: | ---: | ---: | ---: |
| 1k | 2,327,304 B | 2,328,984 B | 1,060,864 B |
| 10k | 7,895,271 B | 7,896,951 B | 3,944,448 B |
| 1m | 226,369,381 B | 226,129,722 B | 397,422,592 B |
| 10m | 1,185,405,381 B | 1,184,871,347 B | 1,845,243,904 B |

Parquet-to-native load used the same SQL file for RuDB and DuckDB at each size. RuDB's optional Q9 projection build is shown separately and belongs in any end-to-end load comparison. These are single fresh-process samples; the 10m RuDB writer comparison is repeated below. Peak RSS is for the individual process, so the full RuDB build's peak is the larger of its load and projection peaks.

| Rows | RuDB old load | RuDB spans load | RuDB projection build | DuckDB load | RuDB spans load RSS | DuckDB load RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 0.373 s | 0.331 s | 0.023 s | 0.322 s | 39.05 MiB | 45.28 MiB |
| 10k | 1.132 s | 1.057 s | 0.053 s | 0.687 s | 78.20 MiB | 83.53 MiB |
| 1m | 10.729 s | 10.538 s | 0.237 s | 10.927 s | 1,150.26 MiB | 1,293.03 MiB |
| 10m | 52.040 s | 57.014 s | 1.566 s | Killed at 30.320 s | 2,094.12 MiB | 5,173.29 MiB before kill |

The default-thread DuckDB 10m load was killed by the host's memory limit, so it has no completed load time. Repeating 10m with the **same** `SET threads=1; SET memory_limit='2GB'` SQL prefix for both engines completed: RuDB loaded in 85.151 s at 1,624.19 MiB peak RSS, then built the projection in 1.933 s at 199.27 MiB; DuckDB loaded in 107.342 s at 2,228.98 MiB. Both bounded files contained 10,000,000 rows. This measures a different resource setting from the default query table above.

I repeated the default RuDB 10m load in reverse order because the first candidate load was 4.974 s slower than the old writer. On the second pair, the old writer took 75.373 s and the span writer took 67.470 s; their CPU times were 171.723 and 173.839 s. The host load average was above 17. These two single-process pairs do not support a stable load-throughput gain or regression from the format change.

Q2 and Q8, which exercise frequency metadata and normal encoded-row grouping, matched DuckDB at every size. So did `count(*)`. Q9 matched in every timed process. The source files came from DuckDB's Parquet export of each existing ClickBench native table; their SHA-256 hashes were `3c6dcf052a11612f7b07573880c8d3ba08be0c349a37e523b0bd84fd46ab94e9` (1k), `9d17db45885417de116d1f2bad315a85347e95de55e3fe2c0ffb03d18f8f71d9` (10k), `5a6ecfc3b5e7931ddec01bb428e67acab0f30fe95b418569a7cc989e03c36168` (1m), and `c721aa678f950f674b6c82a55b730a0c664804c80300e464f014f009768c1638` (10m).

The [raw query trials](q9-frequency-spans/), [load resource records](q9-frequency-spans/), [correctness script](q9-frequency-spans/verify.py), [correctness output](q9-frequency-spans/q9-span-verify.json), and [10k load SQL](q9-frequency-spans/load-10k.sql) are retained. The bounded 10m SQL is [here](q9-frequency-spans/load-10m-bounded.sql). The baseline RuDB binary SHA-256 is `dd5e252e1300a54e337232f75f0ebda648c9323c6d6ab2addca00d3432a67901`; the span binary is `f5d6942c435e8fcaada4ccc49051aace60863b750781dccccd3c386291c93931`. Both were built from RuDB 0.4.30 at `58481010` with the span patch applied only to the latter. The branch passed 204 native unit tests, 418 database unit tests, and strict native Clippy. Native tests and Clippy passed again after the final rebase.
