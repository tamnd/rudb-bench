# ClickBench Q23 profile and RuDB versus DuckDB

The engine change is [tamnd/rudb#1438](https://github.com/tamnd/rudb/pull/1438), merged as `8c9c00d2` on 23 September 2026.
The query comparison below uses that merged RuDB build and DuckDB 1.5.5 on `gpc`.
The same 43 original SQL queries were run against 999,975 rows at 1M and 9,999,750 rows at 10M.
The two query files have the same SHA-256, `274ffe1c4f83baad2fc1773bb6773bbdab0db779faeb2970de8cc531292a5dc6`.
Both engines loaded each size from the same Parquet file with the same table schema and timestamp conversion.
Row counts and sums of `UserID` and `WatchID` match at both sizes.

## Why Q23 was slow

Callgrind attributed 87.8% of a one-thread 1M Q23 run to the pushed filter path.
The largest self-costs were FSST string decompression, string literal replay, `memset`, and `memcpy`.
The substring matcher accounted for only 3.1% of counted instructions.
The filter already carried surviving row positions from one `AND` term to the next, but a later `LIKE` function still evaluated its whole chunk before narrowing its result.
Only 3,782 of 9,999,750 rows pass `Title LIKE '%Google%'` at 10M, so doing the later `URL NOT LIKE '%.google.%'` over whole chunks wasted dictionary decoding.
The accepted change gathers a `LIKE` function's input vectors when no more than one quarter of the chunk remains live, then maps its answer back to original row positions.
Dense selections keep the old full-vector path, and gathered string vectors keep their stable dictionary identity.
On the exact-parent Callgrind pair, Q23 fell from 2,711,492,459 to 1,484,927,270 counted instructions, a 45.24% reduction.
Those Callgrind counts and its diagnostic wall times are not production benchmark timings.

The exact-parent production A/B used commit `c74a9997` against child commit `4c8de168`, with nine alternating fresh-process runs of all 43 original queries on the same native files.
At 10M, Q23 fell from 208.965 to 126.374 ms and the 43-query median sum fell from 1.174060 to 1.091479 s, a 7.03% reduction.
At 1M, Q23 fell from 40.267 to 22.201 ms and the 43-query median sum fell from 0.250664 to 0.233228 s, a 6.96% reduction.
Untied parent and feature outputs matched byte for byte, and tied outputs matched after deterministic secondary ordering.
The patch was rebased onto the rewritten latest `main` before merge and passed 476 executor tests, strict Clippy, and formatting checks.
An earlier experiment that disabled 1,024-value dictionary bulk decisions made Q21 through Q24 slower at both sizes, so branch `native-like-profile` remains unmerged.

## How the RuDB and DuckDB comparison was measured

Both engines used 16 threads, and DuckDB used a 16 GiB memory limit.
Each original query ran nine times per engine in a fresh process, with engine order alternating by repetition.
The table reports the median of those nine runs for each query, not the fastest run.
`SQL ms` is RuDB's `timing.total_ns` or DuckDB's CLI timer, and DuckDB rounds that timer to whole milliseconds.
`Process wall ms` includes opening the database, running the query, and rendering the answer, so it is the better comparison for very short queries.
`CPU ms` is process user plus system CPU, and `Peak RSS MiB` is the process maximum resident memory.
RuDB wrote a small metrics file after each query, while DuckDB printed its timer, and both operations are included in process wall time.
The page cache was not cleared between runs, and the first run was not disk cold.
In every paired cell, `R / D` means RuDB followed by DuckDB, and `Wall D/R` above 1 means RuDB was faster.

| Size | Engine | Complete queries | SQL median sum | Process wall median sum | CPU median sum | Largest observed query RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1M | RuDB | 43/43 | 0.232305 s | 0.471264 s | 1.502324 s | 186.6 MiB |
| 1M | DuckDB 1.5.5 | 43/43 | 0.512000 s | 0.982320 s | 3.761257 s | 273.7 MiB |
| 10M | RuDB | 43/43 | 0.858898 s | 1.159804 s | 9.335880 s | 792.7 MiB |
| 10M | DuckDB 1.5.5 | 43/43 | 2.856000 s | 3.515501 s | 30.299877 s | 1,789.5 MiB |

The 10M full-query lead is 3.33x by SQL median sum and 3.03x by process-wall median sum, not the requested 10x.
Q21 and Q22 are still slower than DuckDB by 10M process wall time, and Q22 uses more peak memory.
Q23's 10M process-wall lead is 1.45x after this change.

## 1M queries

| Q | SQL ms R / D | Process wall ms R / D | Wall D/R | CPU ms R / D | Peak RSS MiB R / D |
| --- | ---: | ---: | ---: | ---: | ---: |
| Q1 | 0.284 / 1.000 | 5.56 / 10.50 | 1.89x | 5.51 / 11.80 | 11.4 / 26.6 |
| Q2 | 0.299 / 1.000 | 5.56 / 11.20 | 2.02x | 5.53 / 13.71 | 11.4 / 28.2 |
| Q3 | 0.310 / 2.000 | 5.57 / 12.25 | 2.20x | 5.54 / 16.33 | 11.5 / 30.2 |
| Q4 | 0.305 / 3.000 | 5.63 / 13.27 | 2.35x | 5.58 / 19.10 | 11.4 / 35.5 |
| Q5 | 0.330 / 14.000 | 5.63 / 24.91 | 4.43x | 5.60 / 73.99 | 11.4 / 78.5 |
| Q6 | 0.333 / 8.000 | 5.83 / 18.85 | 3.23x | 5.80 / 60.22 | 11.3 / 60.0 |
| Q7 | 0.347 / 0.000 | 5.70 / 10.35 | 1.82x | 5.67 / 11.90 | 11.6 / 26.9 |
| Q8 | 0.460 / 1.000 | 5.81 / 11.64 | 2.00x | 5.79 / 15.59 | 12.4 / 30.0 |
| Q9 | 7.834 / 16.000 | 13.75 / 26.73 | 1.94x | 51.05 / 103.04 | 79.0 / 93.8 |
| Q10 | 8.544 / 16.000 | 14.25 / 28.13 | 1.97x | 63.82 / 136.76 | 87.4 / 110.3 |
| Q11 | 4.703 / 6.000 | 10.31 / 16.42 | 1.59x | 26.87 / 45.88 | 41.0 / 59.0 |
| Q12 | 4.415 / 6.000 | 10.02 / 16.63 | 1.66x | 28.14 / 46.83 | 43.9 / 58.9 |
| Q13 | 0.607 / 8.000 | 6.01 / 18.34 | 3.05x | 5.99 / 55.13 | 12.6 / 66.2 |
| Q14 | 6.174 / 11.000 | 11.97 / 22.05 | 1.84x | 38.20 / 84.37 | 56.6 / 97.8 |
| Q15 | 6.009 / 8.000 | 11.73 / 19.08 | 1.63x | 35.53 / 59.51 | 51.1 / 71.2 |
| Q16 | 0.520 / 13.000 | 5.85 / 23.89 | 4.08x | 5.79 / 87.95 | 12.4 / 93.3 |
| Q17 | 1.038 / 18.000 | 6.41 / 29.43 | 4.59x | 6.39 / 153.12 | 13.6 / 161.0 |
| Q18 | 4.643 / 16.000 | 10.34 / 27.56 | 2.66x | 34.45 / 131.21 | 42.1 / 158.6 |
| Q19 | 8.244 / 20.000 | 13.84 / 31.63 | 2.28x | 75.35 / 180.65 | 94.1 / 161.3 |
| Q20 | 2.141 / 2.000 | 8.00 / 12.01 | 1.50x | 10.65 / 18.11 | 15.4 / 35.4 |
| Q21 | 24.374 / 14.000 | 29.97 / 25.81 | 0.86x | 160.76 / 91.67 | 155.3 / 94.0 |
| Q22 | 24.516 / 14.000 | 30.27 / 25.33 | 0.84x | 173.90 / 104.05 | 178.6 / 106.4 |
| Q23 | 19.213 / 20.000 | 25.05 / 31.33 | 1.25x | 169.79 / 129.19 | 160.9 / 129.9 |
| Q24 | 32.843 / 41.000 | 38.74 / 54.05 | 1.39x | 177.27 / 212.22 | 184.4 / 208.2 |
| Q25 | 3.890 / 5.000 | 9.59 / 15.42 | 1.61x | 18.28 / 35.72 | 27.1 / 44.5 |
| Q26 | 5.163 / 4.000 | 10.91 / 14.28 | 1.31x | 27.33 / 30.12 | 42.4 / 36.9 |
| Q27 | 4.518 / 4.000 | 10.19 / 14.86 | 1.46x | 18.11 / 32.57 | 28.1 / 41.7 |
| Q28 | 10.259 / 15.000 | 15.88 / 26.48 | 1.67x | 70.45 / 105.45 | 53.4 / 103.5 |
| Q29 | 0.638 / 89.000 | 6.16 / 101.60 | 16.50x | 6.11 / 671.71 | 12.9 / 151.7 |
| Q30 | 3.766 / 5.000 | 9.27 / 14.73 | 1.59x | 15.26 / 19.93 | 22.7 / 33.0 |
| Q31 | 6.114 / 8.000 | 11.83 / 18.99 | 1.60x | 36.11 / 57.44 | 50.4 / 68.2 |
| Q32 | 6.121 / 8.000 | 11.81 / 19.25 | 1.63x | 39.67 / 62.69 | 56.8 / 77.1 |
| Q33 | 8.514 / 18.000 | 14.23 / 30.17 | 2.12x | 68.69 / 167.87 | 84.7 / 173.0 |
| Q34 | 0.540 / 29.000 | 5.91 / 41.85 | 7.08x | 5.88 / 235.39 | 12.4 / 263.0 |
| Q35 | 0.574 / 31.000 | 5.97 / 43.87 | 7.35x | 5.94 / 233.63 | 12.4 / 268.5 |
| Q36 | 0.767 / 12.000 | 6.21 / 22.98 | 3.70x | 6.18 / 99.75 | 12.6 / 101.1 |
| Q37 | 2.917 / 4.000 | 8.46 / 13.86 | 1.64x | 10.58 / 22.74 | 19.4 / 38.1 |
| Q38 | 2.870 / 3.000 | 8.38 / 13.71 | 1.64x | 9.98 / 21.08 | 18.1 / 35.7 |
| Q39 | 2.287 / 3.000 | 7.83 / 13.45 | 1.72x | 9.94 / 19.76 | 18.7 / 35.2 |
| Q40 | 9.361 / 6.000 | 14.85 / 16.51 | 1.11x | 17.01 / 26.57 | 28.6 / 44.1 |
| Q41 | 1.866 / 3.000 | 7.33 / 13.19 | 1.80x | 9.72 / 19.09 | 17.2 / 36.4 |
| Q42 | 1.806 / 3.000 | 7.33 / 12.92 | 1.76x | 9.30 / 18.82 | 16.7 / 35.2 |
| Q43 | 1.848 / 3.000 | 7.32 / 12.79 | 1.75x | 8.81 / 18.61 | 16.6 / 33.7 |

## 10M queries

| Q | SQL ms R / D | Process wall ms R / D | Wall D/R | CPU ms R / D | Peak RSS MiB R / D |
| --- | ---: | ---: | ---: | ---: | ---: |
| Q1 | 0.285 / 1.000 | 4.71 / 11.24 | 2.39x | 4.67 / 12.58 | 11.6 / 27.2 |
| Q2 | 0.312 / 7.000 | 4.75 / 17.43 | 3.67x | 4.72 / 29.25 | 11.6 / 50.9 |
| Q3 | 0.345 / 13.000 | 4.92 / 23.91 | 4.86x | 4.81 / 51.59 | 11.9 / 72.2 |
| Q4 | 0.333 / 16.000 | 4.89 / 26.47 | 5.41x | 4.87 / 77.17 | 11.9 / 98.0 |
| Q5 | 0.337 / 48.000 | 4.96 / 62.08 | 12.51x | 4.93 / 550.17 | 11.6 / 265.8 |
| Q6 | 0.334 / 42.000 | 4.97 / 55.62 | 11.19x | 4.94 / 531.96 | 11.6 / 280.0 |
| Q7 | 0.340 / 2.000 | 4.86 / 12.19 | 2.51x | 4.83 / 13.66 | 11.9 / 29.0 |
| Q8 | 0.460 / 6.000 | 5.02 / 16.69 | 3.32x | 5.00 / 34.31 | 12.6 / 53.0 |
| Q9 | 38.524 / 58.000 | 43.82 / 73.11 | 1.67x | 539.35 / 766.70 | 390.4 / 327.2 |
| Q10 | 42.397 / 76.000 | 47.32 / 91.61 | 1.94x | 584.06 / 1009.75 | 394.3 / 377.7 |
| Q11 | 10.351 / 19.000 | 15.25 / 32.33 | 2.12x | 117.09 / 215.67 | 106.5 / 180.0 |
| Q12 | 11.864 / 21.000 | 16.72 / 33.28 | 1.99x | 135.17 / 223.16 | 111.7 / 189.2 |
| Q13 | 0.630 / 36.000 | 5.25 / 49.64 | 9.45x | 5.23 / 396.88 | 13.1 / 287.2 |
| Q14 | 18.154 / 63.000 | 23.16 / 80.60 | 3.48x | 209.69 / 760.54 | 160.0 / 459.3 |
| Q15 | 15.112 / 42.000 | 20.14 / 55.85 | 2.77x | 167.69 / 455.07 | 138.4 / 327.7 |
| Q16 | 0.962 / 48.000 | 5.61 / 62.84 | 11.20x | 5.58 / 670.58 | 13.4 / 328.1 |
| Q17 | 2.401 / 102.000 | 7.21 / 122.84 | 17.04x | 7.15 / 1324.76 | 17.6 / 671.3 |
| Q18 | 14.541 / 76.000 | 19.49 / 96.89 | 4.97x | 173.92 / 942.54 | 97.6 / 640.3 |
| Q19 | 56.051 / 141.000 | 61.57 / 166.71 | 2.71x | 783.47 / 1974.80 | 515.4 / 1023.7 |
| Q20 | 2.590 / 8.000 | 9.79 / 19.10 | 1.95x | 17.25 / 67.28 | 27.8 / 96.9 |
| Q21 | 85.234 / 90.000 | 108.70 / 107.18 | 0.99x | 1120.04 / 878.69 | 487.8 / 636.2 |
| Q22 | 93.686 / 103.000 | 127.99 / 122.12 | 0.95x | 1152.10 / 868.03 | 789.6 / 707.3 |
| Q23 | 94.207 / 155.000 | 121.47 / 175.77 | 1.45x | 923.83 / 958.19 | 594.3 / 780.0 |
| Q24 | 109.589 / 149.000 | 115.10 / 172.97 | 1.50x | 887.71 / 829.89 | 537.1 / 577.4 |
| Q25 | 6.452 / 13.000 | 11.31 / 24.33 | 2.15x | 25.54 / 68.83 | 44.8 / 69.7 |
| Q26 | 15.750 / 19.000 | 20.83 / 30.31 | 1.46x | 88.81 / 164.84 | 78.9 / 96.1 |
| Q27 | 10.587 / 10.000 | 15.61 / 21.25 | 1.36x | 36.40 / 78.84 | 54.9 / 70.5 |
| Q28 | 59.903 / 82.000 | 67.65 / 99.87 | 1.48x | 715.00 / 813.97 | 133.2 / 664.7 |
| Q29 | 0.661 / 656.000 | 5.46 / 686.69 | 125.74x | 5.43 / 6726.15 | 13.1 / 829.7 |
| Q30 | 6.081 / 10.000 | 10.81 / 20.26 | 1.87x | 38.48 / 46.02 | 41.6 / 54.5 |
| Q31 | 20.476 / 47.000 | 25.48 / 61.76 | 2.42x | 235.08 / 418.42 | 145.4 / 303.6 |
| Q32 | 22.922 / 48.000 | 27.98 / 63.17 | 2.26x | 289.72 / 507.00 | 192.7 / 397.1 |
| Q33 | 59.773 / 136.000 | 82.51 / 157.41 | 1.91x | 867.50 / 1985.23 | 483.8 / 926.5 |
| Q34 | 0.558 / 185.000 | 5.17 / 215.33 | 41.63x | 5.15 / 2353.58 | 12.6 / 1558.1 |
| Q35 | 0.549 / 214.000 | 5.18 / 244.51 | 47.22x | 5.16 / 2487.59 | 12.6 / 1585.2 |
| Q36 | 0.564 / 56.000 | 5.22 / 71.29 | 13.65x | 5.20 / 753.01 | 12.6 / 362.6 |
| Q37 | 10.846 / 11.000 | 15.86 / 21.94 | 1.38x | 25.79 / 45.36 | 61.2 / 59.5 |
| Q38 | 7.196 / 6.000 | 11.93 / 16.02 | 1.34x | 19.54 / 29.34 | 47.6 / 44.0 |
| Q39 | 10.926 / 6.000 | 15.70 / 16.98 | 1.08x | 25.02 / 30.03 | 61.1 / 44.2 |
| Q40 | 18.510 / 20.000 | 23.28 / 30.43 | 1.31x | 40.91 / 68.78 | 68.9 / 84.2 |
| Q41 | 2.793 / 5.000 | 7.49 / 15.16 | 2.02x | 13.95 / 26.81 | 25.1 / 42.1 |
| Q42 | 3.040 / 5.000 | 7.69 / 15.10 | 1.96x | 12.55 / 26.21 | 25.1 / 40.5 |
| Q43 | 2.267 / 5.000 | 6.97 / 15.21 | 2.18x | 12.53 / 26.65 | 23.2 / 37.7 |

## Load and storage

Load figures are one fresh sample per engine and size, not medians.
The two engines used the same source Parquet file and `CREATE TABLE` schema at each size.
The 1M RuDB file is 207,845,397 bytes and the DuckDB file is 321,138,688 bytes.
The 10M RuDB file is 1,662,899,958 bytes and the DuckDB file is 2,936,549,376 bytes.

| Size | Engine | Load wall | Load CPU | Load peak RSS | Native file |
| --- | --- | ---: | ---: | ---: | ---: |
| 1M | RuDB | 3.755 s | 31.387 s | 2,008.1 MiB | 207,845,397 B |
| 1M | DuckDB 1.5.5 | 3.942 s | 9.193 s | 1,837.8 MiB | 321,138,688 B |
| 10M | RuDB | 21.761 s | 229.629 s | 5,955.9 MiB | 1,662,899,958 B |
| 10M | DuckDB 1.5.5 | 11.256 s | 64.399 s | 7,640.0 MiB | 2,936,549,376 B |

RuDB's 10M load wall time is 1.93x DuckDB's, despite its smaller file and lower peak RSS.
The latest RuDB loader is much faster than the older load recorded in the host-group gate, so load numbers from that older engine should not be mixed with this pair.

## Answers and raw files

The original Q23 result has a non-unique count order, and adding `SearchPhrase` as a secondary key makes the 1M and 10M RuDB and DuckDB Q23 CSV outputs byte-identical.
That diagnostic SQL was not used for any timed result.
Other original queries with a non-unique `ORDER BY` or no `ORDER BY` can return tied rows in different orders across engines, so first-run byte equality is not claimed for all 43 queries here.
The earlier ClickBench correctness reports cover those cases.
The 100M DuckDB and ClickHouse baseline remains in [the existing report](https://github.com/tamnd/rudb-bench/blob/main/reports/2026-09-16/clickbench-10m-root-cause.md).
The raw files are on `gpc` under `/home/gopher/clickbench-native-audit/20260923-frequency-pairs/duck155-rudb-merged-ab/`, with one output, metrics file, and child resource file per run.
Load resource files and native databases sit in the parent audit directory.
The two release binaries are RuDB `8c9c00d2` version 0.4.11 and DuckDB 1.5.5 `d8cdaa33fd`.
The host is an i9-13900K with 32 logical CPUs under WSL2 and 31 GiB of RAM.
