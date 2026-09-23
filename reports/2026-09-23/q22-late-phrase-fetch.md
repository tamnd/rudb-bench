# Q22 selective phrase fetch at 1M and 10M

The measured engine candidate is `b4dc00af` on exact parent `c835859f` in `tamnd/rudb`.
Both CLI executables were built with `cargo build --release -p rudb-cli --bin rudb` on `gpc`.
The parent binary SHA-256 is `ae43a86cd150690b355dd6a825dd4c51ecc6eb3891869580f485d24011e5e7ec`, and the candidate binary SHA-256 is `4e8537f5fb5f8805cca096dfa78892fc9d0682f4c0fc7f724e95385e365655d2`.
Both builds read the same native format 28 database at each size; this change adds no storage metadata and does not change load or file size.
The original 43 RuDB and DuckDB SQL files have the same SHA-256, `274ffe1c4f83baad2fc1773bb6773bbdab0db779faeb2970de8cc531292a5dc6`.
The 1M and 10M tables contain 999,975 and 9,999,750 rows from the matching Parquet sources.

## Cause and change

The Q22 scan read `SearchPhrase` for every part before it evaluated the selective `URL LIKE '%google%'` term.
Only 114 of 9,999,750 rows reached its aggregate in a measured 10M run, yet a separate diagnostic run raised median peak RSS from 229 to 546 MiB merely by adding `SearchPhrase <> ''` to a URL count.
The executor now recognizes a two-column pushed `AND` with a constant, single-column case-sensitive `LIKE` and a simple comparison on the other column.
It reads the `LIKE` column first, evaluates that necessary term, and uses sparse native fetch for the other column when no more than one quarter of the part survives.
It then evaluates the original full predicate on those rows, so the first term never substitutes for the complete answer.
Dense parts and unsupported shapes use the ordinary full read.
An adaptive counter stops using the staged path after 16,384 observed rows show it is not selective enough.
Scans with a sideways join filter keep the ordinary path so row positions retain their original meaning.
The new path is an execution choice over the same native file, not a query-specific SQL rewrite.

The rebased candidate passed 41 catalog and 482 executor library tests, including a sparse, dense, and empty-part test, plus strict Clippy and formatting.
All 43 original queries exited successfully at both sizes.
Some original query outputs differ in unspecified tie order.
Adding `SearchPhrase` as a deterministic secondary ordering made Q22 output byte-identical between the final parent and candidate at both sizes; the original full-suite outputs differ only in Q22's unspecified tie order.

## Exact-parent A/B

Each original query ran nine times per build in a fresh process, with build order alternating by repetition.
The table sums per-query medians; largest query RSS is an observed process maximum, not a sum.

| Rows | Build | SQL median sum | Process wall median sum | CPU median sum | Largest query peak RSS |
| --- | --- | ---: | ---: | ---: | ---: |
| 1M | Parent | 0.226924 s | 0.473060 s | 1.398864 s | 153.2 MiB |
| 1M | Candidate | 0.227967 s | 0.474518 s | 1.419883 s | 138.3 MiB |
| 10M | Parent | 0.845103 s | 1.135482 s | 8.460430 s | 598.6 MiB |
| 10M | Candidate | 0.836077 s | 1.101710 s | 8.410122 s | 577.6 MiB |

The 10M process-wall median sum fell 2.97%, while the 1M sum rose 0.31%.
At 10M, Q22 process wall fell from 111.743 to 101.092 ms and its median peak RSS fell from 571.4 to 276.3 MiB.
Its SQL timer fell only from 87.692 to 86.009 ms, so most of the measured process-wall gain is outside that timer and is not an equivalent scan-latency gain.
At 1M, Q22 process wall was 28.312 versus 28.846 ms, while its median RSS fell from 148.9 to 123.2 MiB.
A separate 21-pair 1M Q22 check on the parent before the final load-only rebase split 11 candidate wins to 10, with a 0.014 ms median paired difference; it does not establish a 1M time gain for this final build.
The complete 10M suite gained, but the small 1M regression remains part of the result.

## Same-host DuckDB comparison

Stable DuckDB 1.5.5 ran the same 43 original queries nine times each in fresh processes with 16 threads and a 16 GiB memory limit.
RuDB also used 16 threads.
The cross-engine sweeps were sequential on the same host, not paired, and the page cache was not cleared.
`SQL ms` is the RuDB internal total or DuckDB CLI timer, which rounds to whole milliseconds.
`Process wall ms` includes startup, database open, query, and output.
`CPU ms` is user plus system CPU, and `Peak RSS MiB` is the process resident high-water mark.
In a paired cell, `R / D` means candidate RuDB followed by DuckDB, and wall `D/R` above one means RuDB was faster.

| Rows | Engine | Completed | SQL median sum | Process wall median sum | CPU median sum | Largest query peak RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1M | RuDB | 43/43 | 0.227967 s | 0.474518 s | 1.419883 s | 138.3 MiB |
| 1M | DuckDB 1.5.5 | 43/43 | 0.511000 s | 0.968606 s | 3.854918 s | 272.5 MiB |
| 10M | RuDB | 43/43 | 0.836077 s | 1.101710 s | 8.410122 s | 577.6 MiB |
| 10M | DuckDB 1.5.5 | 43/43 | 2.774000 s | 3.443129 s | 30.652972 s | 1774.8 MiB |

The 10M workload lead is 3.13x by summed process wall, not the requested 10x.
Q22 is 1.09x faster than DuckDB by 10M process wall and uses 276.3 versus 706.7 MiB of median peak RSS.
At 1M, Q22 remains slower than DuckDB by process wall.

## 1M original queries

| Q | SQL ms R / D | Process wall ms R / D | Wall D/R | CPU ms R / D | Peak RSS MiB R / D |
| --- | ---: | ---: | ---: | ---: | ---: |
| Q1 | 0.163 / 1.000 | 5.61 / 10.26 | 1.83x | 5.58 / 11.60 | 11.7 / 26.6 |
| Q2 | 0.160 / 1.000 | 5.46 / 11.01 | 2.02x | 5.43 / 13.85 | 11.7 / 28.4 |
| Q3 | 0.169 / 2.000 | 5.46 / 11.79 | 2.16x | 5.41 / 16.30 | 11.7 / 30.2 |
| Q4 | 0.145 / 3.000 | 5.48 / 12.26 | 2.24x | 5.45 / 18.22 | 11.7 / 35.7 |
| Q5 | 0.151 / 13.000 | 5.51 / 23.89 | 4.34x | 5.49 / 73.64 | 11.7 / 78.3 |
| Q6 | 0.155 / 8.000 | 5.43 / 19.04 | 3.51x | 5.34 / 59.04 | 11.7 / 60.7 |
| Q7 | 0.163 / 1.000 | 5.38 / 10.47 | 1.95x | 5.32 / 12.18 | 11.7 / 27.0 |
| Q8 | 0.338 / 2.000 | 5.67 / 11.26 | 1.99x | 5.64 / 15.63 | 12.5 / 29.9 |
| Q9 | 10.918 / 15.000 | 16.49 / 24.72 | 1.50x | 63.46 / 106.31 | 79.2 / 98.0 |
| Q10 | 8.936 / 16.000 | 14.99 / 27.78 | 1.85x | 63.91 / 136.52 | 87.6 / 110.5 |
| Q11 | 4.289 / 6.000 | 10.29 / 16.16 | 1.57x | 26.29 / 46.39 | 41.7 / 58.5 |
| Q12 | 4.723 / 6.000 | 10.70 / 16.31 | 1.52x | 28.69 / 48.31 | 44.9 / 60.4 |
| Q13 | 0.484 / 7.000 | 6.27 / 17.96 | 2.86x | 6.25 / 57.01 | 13.0 / 67.7 |
| Q14 | 6.869 / 11.000 | 12.94 / 21.69 | 1.68x | 38.71 / 87.20 | 57.6 / 98.9 |
| Q15 | 5.716 / 8.000 | 11.73 / 18.93 | 1.61x | 33.93 / 62.03 | 52.2 / 71.3 |
| Q16 | 0.441 / 11.000 | 6.10 / 21.98 | 3.60x | 6.08 / 90.34 | 12.4 / 97.5 |
| Q17 | 0.755 / 17.000 | 6.15 / 29.30 | 4.76x | 6.13 / 157.90 | 13.2 / 161.4 |
| Q18 | 6.808 / 16.000 | 12.58 / 27.50 | 2.19x | 31.43 / 136.20 | 41.2 / 159.4 |
| Q19 | 9.792 / 20.000 | 15.83 / 31.62 | 2.00x | 67.28 / 185.98 | 91.7 / 162.5 |
| Q20 | 1.865 / 2.000 | 7.82 / 11.86 | 1.52x | 10.45 / 18.43 | 15.4 / 35.2 |
| Q21 | 19.337 / 14.000 | 25.36 / 25.38 | 1.00x | 144.16 / 93.14 | 102.7 / 94.0 |
| Q22 | 22.674 / 15.000 | 28.85 / 26.12 | 0.91x | 162.96 / 107.01 | 123.2 / 107.1 |
| Q23 | 19.329 / 19.000 | 25.62 / 31.15 | 1.22x | 140.80 / 129.15 | 135.7 / 129.2 |
| Q24 | 29.036 / 42.000 | 35.21 / 54.30 | 1.54x | 162.67 / 216.31 | 130.7 / 208.5 |
| Q25 | 3.762 / 5.000 | 9.78 / 15.90 | 1.63x | 17.30 / 35.20 | 27.2 / 44.3 |
| Q26 | 5.078 / 5.000 | 11.08 / 14.25 | 1.29x | 27.08 / 29.44 | 41.5 / 36.7 |
| Q27 | 4.359 / 5.000 | 10.33 / 14.67 | 1.42x | 18.55 / 34.46 | 28.3 / 43.2 |
| Q28 | 9.479 / 17.000 | 15.43 / 27.40 | 1.78x | 57.80 / 104.83 | 55.3 / 104.0 |
| Q29 | 0.607 / 88.000 | 6.32 / 100.45 | 15.89x | 6.30 / 687.70 | 13.5 / 151.7 |
| Q30 | 3.820 / 5.000 | 9.50 / 14.63 | 1.54x | 15.32 / 20.25 | 24.3 / 33.0 |
| Q31 | 5.772 / 8.000 | 11.77 / 17.85 | 1.52x | 35.42 / 58.14 | 50.6 / 68.0 |
| Q32 | 6.441 / 9.000 | 12.38 / 19.25 | 1.55x | 39.58 / 64.17 | 56.8 / 77.8 |
| Q33 | 8.865 / 18.000 | 14.89 / 30.14 | 2.02x | 70.49 / 170.06 | 84.5 / 173.3 |
| Q34 | 0.476 / 29.000 | 6.05 / 42.61 | 7.04x | 6.03 / 242.81 | 12.5 / 264.9 |
| Q35 | 0.432 / 31.000 | 5.86 / 43.29 | 7.38x | 5.83 / 252.40 | 12.6 / 269.8 |
| Q36 | 0.608 / 12.000 | 5.97 / 23.56 | 3.95x | 5.94 / 104.34 | 13.0 / 102.6 |
| Q37 | 3.602 / 3.000 | 9.36 / 13.69 | 1.46x | 11.28 / 23.39 | 20.8 / 38.1 |
| Q38 | 3.162 / 3.000 | 8.77 / 13.06 | 1.49x | 10.60 / 21.32 | 19.5 / 36.5 |
| Q39 | 2.465 / 3.000 | 8.09 / 12.75 | 1.58x | 10.03 / 20.86 | 19.1 / 35.7 |
| Q40 | 10.426 / 5.000 | 16.19 / 15.16 | 0.94x | 18.18 / 28.56 | 29.5 / 45.1 |
| Q41 | 1.849 / 3.000 | 7.40 / 12.62 | 1.70x | 9.52 / 19.69 | 17.7 / 36.6 |
| Q42 | 1.722 / 3.000 | 7.28 / 12.39 | 1.70x | 9.04 / 19.41 | 17.2 / 35.7 |
| Q43 | 1.624 / 3.000 | 7.14 / 12.22 | 1.71x | 8.76 / 19.22 | 16.8 / 34.2 |

## 10M original queries

| Q | SQL ms R / D | Process wall ms R / D | Wall D/R | CPU ms R / D | Peak RSS MiB R / D |
| --- | ---: | ---: | ---: | ---: | ---: |
| Q1 | 0.165 / 1.000 | 4.81 / 10.98 | 2.28x | 4.79 / 12.27 | 12.0 / 27.2 |
| Q2 | 0.171 / 5.000 | 4.61 / 14.82 | 3.22x | 4.58 / 30.85 | 12.0 / 50.5 |
| Q3 | 0.177 / 9.000 | 4.68 / 18.71 | 3.99x | 4.65 / 55.25 | 12.0 / 72.2 |
| Q4 | 0.159 / 12.000 | 4.61 / 23.30 | 5.06x | 4.58 / 80.39 | 12.0 / 98.0 |
| Q5 | 0.157 / 47.000 | 4.65 / 60.76 | 13.06x | 4.62 / 559.43 | 12.0 / 267.5 |
| Q6 | 0.152 / 42.000 | 4.90 / 56.49 | 11.53x | 4.88 / 538.36 | 11.9 / 286.6 |
| Q7 | 0.171 / 2.000 | 4.64 / 12.53 | 2.70x | 4.62 / 14.35 | 12.0 / 28.9 |
| Q8 | 0.358 / 5.000 | 4.88 / 14.91 | 3.05x | 4.86 / 35.95 | 12.8 / 52.5 |
| Q9 | 38.539 / 59.000 | 43.78 / 74.53 | 1.70x | 483.42 / 745.58 | 338.2 / 327.8 |
| Q10 | 42.532 / 74.000 | 47.72 / 89.83 | 1.88x | 578.28 / 997.04 | 342.2 / 379.0 |
| Q11 | 10.551 / 20.000 | 15.83 / 32.21 | 2.03x | 119.53 / 216.28 | 107.7 / 180.2 |
| Q12 | 11.650 / 22.000 | 16.91 / 33.98 | 2.01x | 135.10 / 237.65 | 111.7 / 189.9 |
| Q13 | 0.554 / 36.000 | 5.49 / 49.68 | 9.06x | 5.46 / 413.85 | 13.2 / 290.5 |
| Q14 | 18.256 / 62.000 | 23.54 / 79.16 | 3.36x | 210.78 / 774.67 | 163.8 / 460.0 |
| Q15 | 14.793 / 39.000 | 19.98 / 54.11 | 2.71x | 165.46 / 450.26 | 140.0 / 327.4 |
| Q16 | 0.859 / 48.000 | 5.55 / 62.60 | 11.27x | 5.53 / 678.26 | 13.5 / 327.5 |
| Q17 | 2.423 / 101.000 | 7.19 / 121.93 | 16.96x | 7.16 / 1330.31 | 17.7 / 671.0 |
| Q18 | 16.689 / 77.000 | 21.99 / 97.20 | 4.42x | 187.16 / 950.31 | 97.5 / 640.8 |
| Q19 | 65.436 / 141.000 | 86.93 / 166.94 | 1.92x | 898.14 / 1984.16 | 572.6 / 1023.5 |
| Q20 | 2.850 / 8.000 | 8.41 / 18.89 | 2.25x | 15.84 / 67.93 | 28.0 / 96.9 |
| Q21 | 79.232 / 80.000 | 92.17 / 98.74 | 1.07x | 922.65 / 898.35 | 228.2 / 636.0 |
| Q22 | 86.009 / 90.000 | 101.09 / 110.04 | 1.09x | 953.87 / 897.17 | 276.3 / 706.7 |
| Q23 | 84.821 / 116.000 | 104.29 / 136.71 | 1.31x | 720.40 / 995.36 | 418.0 / 765.6 |
| Q24 | 102.192 / 131.000 | 107.73 / 155.65 | 1.44x | 674.31 / 874.49 | 225.7 / 578.0 |
| Q25 | 6.565 / 11.000 | 11.66 / 22.50 | 1.93x | 28.57 / 70.04 | 45.5 / 69.5 |
| Q26 | 15.776 / 18.000 | 21.31 / 28.63 | 1.34x | 79.03 / 166.87 | 76.5 / 95.7 |
| Q27 | 10.421 / 10.000 | 15.79 / 20.89 | 1.32x | 34.73 / 79.19 | 53.1 / 70.5 |
| Q28 | 51.720 / 85.000 | 57.23 / 102.73 | 1.80x | 538.66 / 810.38 | 144.4 / 665.2 |
| Q29 | 0.586 / 674.000 | 5.59 / 708.34 | 126.70x | 5.56 / 6753.29 | 13.8 / 830.7 |
| Q30 | 5.991 / 10.000 | 11.08 / 21.43 | 1.93x | 45.15 / 50.67 | 45.8 / 54.5 |
| Q31 | 18.438 / 58.000 | 23.90 / 73.35 | 3.07x | 231.67 / 423.79 | 146.6 / 303.6 |
| Q32 | 22.322 / 51.000 | 27.66 / 66.36 | 2.40x | 280.75 / 506.72 | 190.3 / 398.3 |
| Q33 | 61.962 / 134.000 | 68.13 / 156.18 | 2.29x | 868.24 / 1965.88 | 486.6 / 925.6 |
| Q34 | 0.473 / 189.000 | 5.29 / 220.63 | 41.71x | 5.26 / 2414.14 | 12.8 / 1557.9 |
| Q35 | 0.452 / 197.000 | 5.25 / 228.38 | 43.52x | 5.22 / 2533.13 | 13.0 / 1586.8 |
| Q36 | 0.449 / 55.000 | 5.07 / 70.79 | 13.96x | 5.05 / 779.64 | 13.2 / 361.9 |
| Q37 | 13.443 / 11.000 | 18.57 / 21.69 | 1.17x | 30.40 / 46.77 | 67.3 / 59.7 |
| Q38 | 8.328 / 5.000 | 13.51 / 15.67 | 1.16x | 22.09 / 30.92 | 48.0 / 44.2 |
| Q39 | 12.736 / 6.000 | 17.96 / 16.36 | 0.91x | 27.51 / 30.85 | 62.3 / 44.5 |
| Q40 | 19.554 / 19.000 | 24.69 / 29.81 | 1.21x | 42.29 / 70.54 | 69.5 / 85.5 |
| Q41 | 2.708 / 5.000 | 7.54 / 15.12 | 2.01x | 13.74 / 27.89 | 25.5 / 43.0 |
| Q42 | 2.934 / 4.000 | 7.84 / 14.70 | 1.87x | 12.60 / 27.05 | 26.0 / 40.9 |
| Q43 | 2.323 / 5.000 | 7.27 / 14.84 | 2.04x | 12.94 / 26.67 | 24.5 / 38.0 |

## Raw runs and next gate

The final exact-parent files are under `/home/gopher/clickbench-native-audit/20260923-frequency-pairs/q22-late-fetch-merged-parent-ab` on `gpc`.
The fresh DuckDB files are in the adjacent `q22-late-fetch-duck155` directory.
Each query has stdout, stderr, and process resource JSON, and RuDB has per-query metrics JSON.
The 21-pair 1M Q22 confirmation on the earlier parent is in `q22-late-fetch-final-1m-confirm`.
The earlier `q22-late-fetch-confirm` pre-rebase run had 19 candidate Q22 wins in 21 10M pairs and found no reproducible Q28 regression, but it is not the final exact-parent measurement.
The next work should profile Q22's remaining roughly 86 ms SQL path and the other 10M leaders rather than treating its process-wall improvement as a 10x workload result.
