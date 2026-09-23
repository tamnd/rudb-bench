# Native substring signatures and the 1M and 10M ClickBench gap

Correction on 23 September 2026: the first version of this report measured stale RuDB CLI executables after a build command rebuilt only the `rudb` library package.
Those timing and memory tables are withdrawn.
This version uses executables built explicitly with `cargo build --release -p rudb-cli --bin rudb` from parent `97a146a2` and candidate `8a62e7cd`.
The verified binary SHA-256 values are `2483b895b75d03f77509472b4a28d330b40b3c8f61aae641a94485415e63339f` for the parent and `91c2f8efe1dc906115bc499e6e6d745e94c8293a85719024b2637d7aa0723e0d` for the candidate.
The earlier raw runs remain on `gpc` for audit but must not be used for performance claims.

The corrected run covers all 43 original queries at 999,975 and 9,999,750 rows on `gpc`.
The RuDB and DuckDB query files have the same SHA-256, `274ffe1c4f83baad2fc1773bb6773bbdab0db779faeb2970de8cc531292a5dc6`.
The two engines loaded from the same Parquet source at each size, and row counts and the `UserID` and `WatchID` sums match.

## Root cause and change

The verified one-thread 1M Callgrind pair counted 1,647,661,714 instructions for parent Q21 and 1,358,020,810 for the candidate, a 17.58% reduction.
Q22 fell from 1,664,712,070 to 1,478,003,022 instructions, an 11.22% reduction.
FSST decompression alone accounted for roughly 20% of both Q21 runs and fell from 329,648,905 to 265,876,833 counted instructions.
These counts diagnose work inside a one-thread run and are not production latency.
The native dictionary's stable `LIKE` path still decoded many high-cardinality URL values when a required literal was absent from a whole 1,024-value payload block.
Only 1,577 rows match Q21 at 10M.
The remaining decoder cost is why this change does not reach the 10x target.

Native format 28 stores a 2 KiB two-hash four-byte-gram signature for each 1,024-value dictionary payload block.
For a case-sensitive literal `LIKE` containing at least four bytes, the scan lazily reads the checksummed signatures and skips a block only when a required gram is certainly absent.
Possible matches still use the exact matcher, and `NOT LIKE` converts a proven absent block to true.
Older native format 27 remains readable, and unsupported patterns use the old path.
The signature section follows the lazy rank section, so unrelated dictionary opens do not read it.
A one-hash prototype had too many possible matches in a small URL sample, and eagerly reading the two-hash extent added I/O to unrelated string queries; those experiments were not merged.
The candidate also includes a checked-subtraction fallback for an extreme-integer count test failure inherited from its parent.
The final branch passed 154 native, 288 kernel, and 478 executor tests, strict Clippy, and formatting checks.

## Exact-parent A/B

Each original query ran nine times per build in a fresh process, alternating parent and candidate on the same host.
The table sums per-query medians; peak RSS is the largest observed query process, not a sum.

| Rows | Build | SQL median sum | Process wall median sum | CPU median sum | Largest query peak RSS |
| --- | --- | ---: | ---: | ---: | ---: |
| 1M | Parent | 0.249564 s | 0.531648 s | 1.771385 s | 153.6 MiB |
| 1M | Signature | 0.235530 s | 0.506847 s | 1.660933 s | 153.7 MiB |
| 10M | Parent | 1.133655 s | 1.524990 s | 12.183148 s | 579.9 MiB |
| 10M | Signature | 1.097387 s | 1.476451 s | 11.571551 s | 582.9 MiB |

The 1M process-wall median sum fell 4.66%, and the 10M sum fell 3.18%.
At 10M, Q21 fell from 115.757 to 109.856 ms, Q22 from 145.067 to 142.845 ms, and Q24 from 137.594 to 126.703 ms.
The candidate's largest 10M query RSS was 582.9 MiB against the parent's 579.9 MiB, so the corrected run does not show a memory improvement.
The earlier 20% Q21 and Q24 acceptance gate remains unmet: those 10M process-wall reductions are 5.10% and 7.92%.
The full-suite gain is incremental, not completion of that gate or of the 10x goal.

The format 28 native file is 210,036,929 bytes at 1M against 206,788,933 bytes for format 27, a 1.57% increase.
At 10M it is 1,684,821,790 against 1,662,658,931 bytes, a 1.33% increase.
The earlier load-time measurements used unverified executables and are withdrawn.
The load penalty, if any, needs a repeated run with verified executables before another claim.

## Same-host DuckDB comparison

DuckDB 1.5.5 ran the same 43 original queries nine times each in a fresh process with 16 threads and a 16 GiB memory limit.
RuDB also used 16 threads.
The verified RuDB and DuckDB runs used the same host and source data; these cross-engine runs were sequential, not alternating.
The page cache was not dropped and the first run was not disk cold.
Each table cell reports the median of nine runs, not the fastest.
`SQL ms` is RuDB's internal total or DuckDB's CLI timer; DuckDB rounds its timer to whole milliseconds.
`Process wall ms` includes process startup, database open, query, and output.
`CPU ms` is process user plus system CPU.
`Peak RSS MiB` is the per-process resident high-water mark.
In a paired cell, `R / D` means RuDB followed by DuckDB, and wall `D/R` above one means RuDB was faster.

| Rows | Engine | Completed | SQL median sum | Process wall median sum | CPU median sum | Largest query peak RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1M | RuDB | 43/43 | 0.235530 s | 0.506847 s | 1.660933 s | 153.7 MiB |
| 1M | DuckDB 1.5.5 | 43/43 | 0.545000 s | 1.067317 s | 4.372068 s | 275.0 MiB |
| 10M | RuDB | 43/43 | 1.097387 s | 1.476451 s | 11.571551 s | 582.9 MiB |
| 10M | DuckDB 1.5.5 | 43/43 | 2.960000 s | 3.729897 s | 37.472858 s | 1788.0 MiB |

At 10M, the workload lead is 2.70x by SQL median sum and 2.53x by process-wall median sum.
Q21 and Q22 remain slower than DuckDB by 10M process wall, and Q22 uses less peak RSS than DuckDB in this corrected run.
The workload-level 10x target remains open.

## 1M queries

| Q | SQL ms R / D | Process wall ms R / D | Wall D/R | CPU ms R / D | Peak RSS MiB R / D |
| --- | ---: | ---: | ---: | ---: | ---: |
| Q1 | 0.211 / 1.000 | 6.34 / 11.75 | 1.85x | 6.30 / 13.40 | 11.6 / 26.5 |
| Q2 | 0.240 / 1.000 | 6.49 / 12.57 | 1.94x | 6.45 / 16.63 | 11.6 / 28.2 |
| Q3 | 0.282 / 2.000 | 6.34 / 13.01 | 2.05x | 6.29 / 18.52 | 11.6 / 30.2 |
| Q4 | 0.233 / 2.000 | 6.32 / 13.54 | 2.14x | 6.28 / 21.38 | 11.6 / 35.7 |
| Q5 | 0.241 / 10.000 | 6.39 / 22.72 | 3.56x | 6.35 / 85.69 | 11.5 / 83.5 |
| Q6 | 0.250 / 9.000 | 6.43 / 20.43 | 3.17x | 6.40 / 64.56 | 11.3 / 62.5 |
| Q7 | 0.268 / 1.000 | 6.42 / 11.79 | 1.84x | 6.36 / 13.60 | 11.6 / 27.0 |
| Q8 | 0.477 / 2.000 | 6.71 / 13.10 | 1.95x | 6.67 / 18.25 | 12.4 / 30.0 |
| Q9 | 7.322 / 14.000 | 13.76 / 26.17 | 1.90x | 61.76 / 128.14 | 79.6 / 102.2 |
| Q10 | 8.605 / 17.000 | 15.10 / 29.47 | 1.95x | 81.58 / 156.20 | 88.5 / 110.9 |
| Q11 | 4.619 / 6.000 | 10.90 / 18.01 | 1.65x | 29.90 / 50.87 | 41.3 / 59.2 |
| Q12 | 4.982 / 6.000 | 11.40 / 18.19 | 1.60x | 33.39 / 54.50 | 44.1 / 60.8 |
| Q13 | 0.582 / 8.000 | 6.85 / 20.28 | 2.96x | 6.80 / 62.50 | 12.8 / 67.6 |
| Q14 | 6.797 / 12.000 | 13.19 / 24.05 | 1.82x | 44.32 / 98.69 | 56.6 / 100.0 |
| Q15 | 6.430 / 9.000 | 12.64 / 20.97 | 1.66x | 41.10 / 70.20 | 51.3 / 72.2 |
| Q16 | 0.514 / 12.000 | 6.70 / 24.08 | 3.60x | 6.66 / 109.03 | 12.3 / 101.8 |
| Q17 | 0.919 / 19.000 | 7.16 / 31.71 | 4.43x | 7.12 / 178.88 | 13.1 / 161.7 |
| Q18 | 5.244 / 17.000 | 11.53 / 30.53 | 2.65x | 37.40 / 163.04 | 41.9 / 159.9 |
| Q19 | 8.828 / 22.000 | 15.24 / 35.12 | 2.31x | 82.85 / 216.24 | 93.0 / 162.8 |
| Q20 | 2.196 / 2.000 | 8.72 / 13.30 | 1.53x | 11.62 / 20.58 | 15.1 / 35.5 |
| Q21 | 21.344 / 16.000 | 27.62 / 28.21 | 1.02x | 166.90 / 100.75 | 102.1 / 93.7 |
| Q22 | 24.187 / 15.000 | 30.83 / 27.76 | 0.90x | 185.77 / 112.03 | 150.3 / 107.0 |
| Q23 | 21.687 / 20.000 | 28.19 / 33.05 | 1.17x | 158.39 / 136.43 | 134.9 / 129.6 |
| Q24 | 31.464 / 46.000 | 38.15 / 59.47 | 1.56x | 182.23 / 233.72 | 131.6 / 208.3 |
| Q25 | 3.859 / 5.000 | 10.52 / 16.94 | 1.61x | 19.84 / 38.55 | 26.6 / 44.4 |
| Q26 | 5.208 / 4.000 | 11.78 / 15.73 | 1.34x | 32.89 / 33.75 | 41.3 / 37.0 |
| Q27 | 4.773 / 5.000 | 11.15 / 16.49 | 1.48x | 21.28 / 38.29 | 28.1 / 43.4 |
| Q28 | 9.917 / 17.000 | 16.30 / 28.53 | 1.75x | 90.63 / 113.21 | 53.0 / 104.0 |
| Q29 | 0.671 / 95.000 | 6.97 / 111.48 | 16.01x | 6.82 / 775.44 | 13.0 / 152.2 |
| Q30 | 3.872 / 5.000 | 10.34 / 16.66 | 1.61x | 17.21 / 22.88 | 22.7 / 32.7 |
| Q31 | 5.985 / 9.000 | 12.42 / 20.05 | 1.62x | 44.79 / 63.77 | 50.7 / 68.8 |
| Q32 | 6.440 / 9.000 | 12.84 / 20.51 | 1.60x | 48.13 / 67.20 | 57.1 / 78.8 |
| Q33 | 8.635 / 19.000 | 15.05 / 32.04 | 2.13x | 85.71 / 193.55 | 85.6 / 173.2 |
| Q34 | 0.617 / 34.000 | 7.00 / 48.30 | 6.90x | 6.96 / 286.86 | 12.6 / 266.5 |
| Q35 | 0.522 / 35.000 | 6.79 / 49.93 | 7.36x | 6.73 / 292.24 | 12.6 / 272.2 |
| Q36 | 0.708 / 13.000 | 6.92 / 25.92 | 3.75x | 6.88 / 124.13 | 12.6 / 104.1 |
| Q37 | 3.623 / 4.000 | 9.76 / 15.46 | 1.58x | 12.10 / 26.47 | 19.9 / 38.2 |
| Q38 | 3.326 / 4.000 | 9.37 / 14.92 | 1.59x | 11.35 / 25.04 | 18.7 / 36.2 |
| Q39 | 2.523 / 3.000 | 8.70 / 14.56 | 1.67x | 10.78 / 24.05 | 18.9 / 36.0 |
| Q40 | 11.163 / 6.000 | 17.27 / 17.36 | 1.01x | 19.76 / 33.49 | 30.0 / 45.7 |
| Q41 | 1.885 / 3.000 | 8.04 / 14.70 | 1.83x | 10.37 / 22.68 | 17.3 / 37.1 |
| Q42 | 1.904 / 3.000 | 8.13 / 14.37 | 1.77x | 10.14 / 23.93 | 17.1 / 36.2 |
| Q43 | 1.973 / 3.000 | 8.10 / 14.05 | 1.74x | 9.68 / 22.70 | 16.6 / 34.1 |

## 10M queries

| Q | SQL ms R / D | Process wall ms R / D | Wall D/R | CPU ms R / D | Peak RSS MiB R / D |
| --- | ---: | ---: | ---: | ---: | ---: |
| Q1 | 0.219 / 1.000 | 5.64 / 12.94 | 2.29x | 5.60 / 15.17 | 11.6 / 27.2 |
| Q2 | 0.254 / 5.000 | 5.73 / 16.23 | 2.83x | 5.67 / 36.28 | 11.8 / 51.0 |
| Q3 | 0.257 / 8.000 | 5.88 / 19.30 | 3.28x | 5.84 / 62.36 | 11.8 / 72.2 |
| Q4 | 0.233 / 10.000 | 5.60 / 22.84 | 4.08x | 5.53 / 91.00 | 11.8 / 98.1 |
| Q5 | 0.237 / 53.000 | 5.54 / 68.70 | 12.40x | 5.50 / 737.49 | 11.8 / 271.4 |
| Q6 | 0.245 / 45.000 | 5.62 / 60.73 | 10.80x | 5.58 / 602.11 | 11.7 / 277.7 |
| Q7 | 0.292 / 2.000 | 5.69 / 14.13 | 2.48x | 5.65 / 15.91 | 11.8 / 28.8 |
| Q8 | 0.478 / 5.000 | 5.96 / 16.66 | 2.79x | 5.92 / 38.42 | 12.6 / 52.9 |
| Q9 | 74.309 / 68.000 | 81.66 / 84.96 | 1.04x | 862.38 / 942.81 | 308.6 / 328.8 |
| Q10 | 80.321 / 87.000 | 94.65 / 104.38 | 1.10x | 976.35 / 1265.84 | 316.8 / 380.3 |
| Q11 | 13.480 / 21.000 | 19.45 / 35.06 | 1.80x | 149.73 / 234.80 | 107.1 / 180.4 |
| Q12 | 14.384 / 23.000 | 20.38 / 37.07 | 1.82x | 182.24 / 258.43 | 112.6 / 190.1 |
| Q13 | 0.660 / 39.000 | 6.84 / 54.21 | 7.92x | 6.79 / 496.48 | 13.1 / 300.0 |
| Q14 | 24.187 / 74.000 | 30.26 / 93.45 | 3.09x | 270.21 / 976.10 | 161.3 / 469.1 |
| Q15 | 18.256 / 43.000 | 24.14 / 59.93 | 2.48x | 207.08 / 552.42 | 138.4 / 334.0 |
| Q16 | 0.998 / 59.000 | 6.55 / 76.06 | 11.61x | 6.51 / 870.48 | 13.5 / 332.8 |
| Q17 | 2.919 / 113.000 | 8.76 / 137.17 | 15.65x | 8.71 / 1668.33 | 17.6 / 681.7 |
| Q18 | 17.998 / 83.000 | 25.14 / 106.40 | 4.23x | 212.43 / 1134.71 | 98.8 / 648.0 |
| Q19 | 85.956 / 173.000 | 118.95 / 204.95 | 1.72x | 1271.42 / 2606.84 | 548.1 / 1043.5 |
| Q20 | 2.829 / 8.000 | 9.10 / 20.65 | 2.27x | 18.53 / 70.50 | 27.5 / 97.0 |
| Q21 | 94.639 / 87.000 | 109.86 / 108.18 | 0.98x | 1134.05 / 998.50 | 224.4 / 637.2 |
| Q22 | 105.567 / 93.000 | 142.84 / 114.04 | 0.80x | 1234.76 / 996.19 | 573.8 / 707.7 |
| Q23 | 103.480 / 118.000 | 131.57 / 140.80 | 1.07x | 878.38 / 1114.21 | 418.1 / 765.7 |
| Q24 | 120.602 / 134.000 | 126.70 / 161.89 | 1.28x | 812.93 / 931.99 | 226.4 / 577.0 |
| Q25 | 7.880 / 11.000 | 13.66 / 23.63 | 1.73x | 34.21 / 76.49 | 46.8 / 70.0 |
| Q26 | 18.302 / 16.000 | 24.52 / 28.27 | 1.15x | 121.36 / 187.41 | 80.6 / 96.2 |
| Q27 | 12.419 / 10.000 | 18.29 / 22.65 | 1.24x | 43.14 / 88.56 | 56.3 / 70.7 |
| Q28 | 61.310 / 86.000 | 68.39 / 107.75 | 1.58x | 675.33 / 895.41 | 134.1 / 665.7 |
| Q29 | 0.803 / 633.000 | 6.88 / 675.26 | 98.11x | 6.41 / 8244.69 | 13.3 / 830.0 |
| Q30 | 6.886 / 9.000 | 12.77 / 20.71 | 1.62x | 54.56 / 54.30 | 46.3 / 54.4 |
| Q31 | 24.893 / 50.000 | 30.85 / 65.39 | 2.12x | 305.11 / 490.39 | 145.1 / 306.6 |
| Q32 | 28.575 / 50.000 | 34.45 / 67.39 | 1.96x | 371.51 / 583.48 | 188.9 / 405.6 |
| Q33 | 100.227 / 174.000 | 132.90 / 200.46 | 1.51x | 1471.66 / 2653.76 | 498.6 / 946.0 |
| Q34 | 0.599 / 220.000 | 6.34 / 257.92 | 40.66x | 6.30 / 3034.82 | 12.8 / 1569.2 |
| Q35 | 0.615 / 224.000 | 6.32 / 262.91 | 41.62x | 6.24 / 3173.54 | 12.8 / 1597.7 |
| Q36 | 0.630 / 66.000 | 6.34 / 82.65 | 13.03x | 6.29 / 980.01 | 12.8 / 369.2 |
| Q37 | 14.653 / 12.000 | 20.86 / 23.99 | 1.15x | 34.75 / 50.66 | 62.7 / 59.5 |
| Q38 | 9.097 / 5.000 | 14.78 / 17.75 | 1.20x | 25.44 / 34.17 | 48.1 / 44.5 |
| Q39 | 14.723 / 7.000 | 21.14 / 18.66 | 0.88x | 33.03 / 35.19 | 61.9 / 44.4 |
| Q40 | 23.184 / 20.000 | 28.87 / 33.18 | 1.15x | 50.82 / 79.54 | 69.3 / 85.2 |
| Q41 | 3.290 / 5.000 | 8.79 / 16.96 | 1.93x | 16.76 / 31.64 | 25.4 / 43.1 |
| Q42 | 3.675 / 5.000 | 9.38 / 16.47 | 1.76x | 15.28 / 30.51 | 25.6 / 41.5 |
| Q43 | 2.828 / 5.000 | 8.39 / 17.16 | 2.05x | 15.56 / 30.93 | 23.9 / 37.8 |

## Raw runs and next bottleneck

The verified exact-parent files are in `gram-sieve-cli-verified`, and the verified DuckDB files are in `gram-sieve-cli-verified-duck` under `/home/gopher/clickbench-native-audit/20260923-frequency-pairs`.
Each query has stdout, stderr, and process resource JSON, and RuDB also has per-query metrics JSON.
The Q21 and Q22 Callgrind files are in `gram-sieve-cli-verified` with the same base and feature labels.
The pre-correction `gram-sieve-latest-main` and `gram-sieve-duck155-rebased` timing files are retained but withdrawn from claims made here.
The next profile should quantify which signature-positive dictionary blocks really contain a match and how much time remains in FSST decode, payload copying, and aggregation.
A new metadata field needs a measured 1M and 10M query benefit and a verified repeated load-cost check.
