# Native substring signatures and the 1M and 10M ClickBench gap

The accepted candidate is RuDB commit `89cd2fcb` on top of `4743b737`, compared with the exact parent and DuckDB 1.5.5 on `gpc`.
This report covers all 43 original queries at 999,975 and 9,999,750 rows.
The RuDB and DuckDB query files have the same SHA-256, `274ffe1c4f83baad2fc1773bb6773bbdab0db779faeb2970de8cc531292a5dc6`.
Both engines loaded from the same Parquet source at each size, and row counts and the `UserID` and `WatchID` sums match.
The source and raw runs are under `/home/gopher/clickbench-native-audit/20260923-frequency-pairs` on `gpc`.

## Root cause and change

One-thread Callgrind attributed about 94% of Q21 and Q22 instructions to the scan and pushed filter.
Q21 counted 1,649,688,689 instructions and Q22 counted 1,654,009,076 before the change.
In Q21, `memset` cost 224,344,019 instructions, `memcpy` 180,005,254, FSST decompression 177,238,474, and LZ replay 88,487,842.
The substring matching itself was only about 1% of the count.
The native dictionary's stable `LIKE` path decoded many high-cardinality URL values before discovering that a literal was absent from nearly every 1,024-value payload block.
Only 1,577 rows match Q21 at 10M, so this is wasted decode and copy work rather than a slow substring matcher.

Native format 28 stores a 2 KiB two-hash four-byte-gram signature for each 1,024-value dictionary payload block.
For a case-sensitive literal `LIKE` containing at least four bytes, the scan lazily reads the checksummed signatures and skips a block only when a required gram is certainly absent.
Possible matches still use the existing exact matcher, and `NOT LIKE` converts a proven absent block to true.
Older native format 27 remains readable, and unsupported patterns keep the original path.
The signature section is after the lazy rank data, so unrelated queries do not read it when opening the dictionary.
A single-bit signature was too collision-prone in a URL sample: 93 of 101 blocks passed a `%google%` precheck although only six had a match.
The two-hash sample passed six of 20 blocks with three real matches; these are diagnostic samples, not measured production skip rates.
An eager placement regressed Q37 through Q40 because Q40 read 37.38 MB instead of 23.91 MB; the lazy layout removed that extra read.
A thread-local read-buffer experiment also lost on 10M Q21 and Q22 and was not merged.

The one-thread 1M Callgrind count fell from 1,645,172,533 to 1,355,175,258 for Q21 and from 1,850,217,141 to 1,557,260,085 for Q24.
That is 17.6% and 15.8% fewer diagnostic instructions, not a production latency claim.
The final rebased build passed 154 native, 288 kernel, and 478 executor tests, strict Clippy, formatting, and the original query result checks.
Tied result order can differ without changing the result; deterministic secondary ordering made Q22 and Q23 byte-identical in the checked comparison.

## Exact-parent A/B

Each original query ran nine times per build in a fresh process, alternating parent and candidate on the same host.
The table sums the per-query medians; peak RSS is the largest observed query process, not a sum.

| Rows | Build | SQL median sum | Process wall median sum | CPU median sum | Largest query peak RSS |
| --- | --- | ---: | ---: | ---: | ---: |
| 1M | Parent | 0.238469 s | 0.494390 s | 1.565480 s | 184.2 MiB |
| 1M | Signature | 0.224168 s | 0.480330 s | 1.476725 s | 172.2 MiB |
| 10M | Parent | 0.944707 s | 1.293464 s | 10.113031 s | 791.5 MiB |
| 10M | Signature | 0.928217 s | 1.251906 s | 9.674077 s | 735.7 MiB |

The 1M process-wall median sum fell 2.84%, and the 10M sum fell 3.21%.
At 10M, Q21 fell from 117.121 to 109.463 ms and Q24 from 140.049 to 117.725 ms.
Q28 regressed by 6.239 ms and Q33 by 3.150 ms at 10M, so the complete workload result matters more than either selected query.
The earlier 20% Q21 and Q24 acceptance gate is not met: Q21 improved 6.54% and Q24 improved 15.94% by 10M process wall.
This is an incremental full-suite gain, not completion of that gate or of the 10x goal.

The format 28 file is 210,036,929 bytes at 1M against 206,788,933 bytes for format 27, a 1.57% increase.
At 10M it is 1,684,821,790 against 1,662,658,931 bytes, a 1.33% increase.
Single pre-rebase native load runs were 13.101 versus 13.046 seconds at 1M and 17.602 versus 15.913 seconds at 10M, candidate versus parent.
Those single load numbers were not repeated or taken with the rebased binaries, so they indicate a possible load penalty but are not a stable load-time comparison.

## Same-host DuckDB comparison

DuckDB 1.5.5 ran the same 43 original queries nine times each in a fresh process with 16 threads and a 16 GiB memory limit.
RuDB also used 16 threads.
The candidate RuDB run above and the DuckDB run below used the same host and source data; these cross-engine runs were sequential, not alternating.
The page cache was not dropped and the first run was not disk cold.
Each table cell reports the median of nine runs, not the fastest.
`SQL ms` is RuDB's internal total or DuckDB's CLI timer; DuckDB rounds its timer to whole milliseconds.
`Process wall ms` includes process startup, database open, query, and output.
`CPU ms` is process user plus system CPU.
`Peak RSS MiB` is the per-process resident high-water mark.
In a paired cell, `R / D` means RuDB followed by DuckDB, and wall `D/R` above one means RuDB was faster.

| Rows | Engine | Completed | SQL median sum | Process wall median sum | CPU median sum | Largest query peak RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1M | RuDB | 43/43 | 0.224168 s | 0.480330 s | 1.476725 s | 172.2 MiB |
| 1M | DuckDB 1.5.5 | 43/43 | 0.509000 s | 0.965573 s | 3.827589 s | 272.2 MiB |
| 10M | RuDB | 43/43 | 0.928217 s | 1.251906 s | 9.674077 s | 735.7 MiB |
| 10M | DuckDB 1.5.5 | 43/43 | 2.985000 s | 3.679567 s | 32.840637 s | 1766.5 MiB |

At 10M, the workload lead is 3.22x by SQL median sum and 2.94x by process-wall median sum.
Q21 and Q22 remain slower than DuckDB by 10M process wall, and Q22 uses more peak RSS.
The workload-level 10x target remains open.

## 1M queries

| Q | SQL ms R / D | Process wall ms R / D | Wall D/R | CPU ms R / D | Peak RSS MiB R / D |
| --- | ---: | ---: | ---: | ---: | ---: |
| Q1 | 0.155 / 0.000 | 5.59 / 10.21 | 1.83x | 5.56 / 11.82 | 11.4 / 26.5 |
| Q2 | 0.182 / 2.000 | 5.75 / 10.95 | 1.91x | 5.72 / 13.86 | 11.4 / 28.4 |
| Q3 | 0.173 / 2.000 | 5.69 / 11.36 | 2.00x | 5.65 / 16.16 | 11.4 / 30.2 |
| Q4 | 0.177 / 2.000 | 5.74 / 11.98 | 2.09x | 5.72 / 18.52 | 11.4 / 35.7 |
| Q5 | 0.169 / 12.000 | 5.68 / 22.37 | 3.94x | 5.66 / 72.00 | 11.4 / 81.5 |
| Q6 | 0.170 / 8.000 | 5.74 / 18.68 | 3.25x | 5.71 / 58.32 | 11.4 / 61.4 |
| Q7 | 0.177 / 1.000 | 5.68 / 10.40 | 1.83x | 5.65 / 11.75 | 11.7 / 27.0 |
| Q8 | 0.366 / 2.000 | 6.04 / 11.33 | 1.88x | 5.89 / 15.84 | 12.5 / 30.0 |
| Q9 | 7.179 / 15.000 | 13.68 / 25.65 | 1.87x | 58.95 / 103.87 | 79.4 / 99.3 |
| Q10 | 8.104 / 16.000 | 14.23 / 26.92 | 1.89x | 74.25 / 136.31 | 88.0 / 110.2 |
| Q11 | 4.620 / 6.000 | 10.65 / 16.18 | 1.52x | 24.79 / 46.42 | 41.2 / 58.4 |
| Q12 | 4.830 / 6.000 | 10.92 / 16.09 | 1.47x | 27.56 / 47.93 | 44.4 / 60.7 |
| Q13 | 0.498 / 7.000 | 6.26 / 17.84 | 2.85x | 6.23 / 56.61 | 12.7 / 67.3 |
| Q14 | 6.785 / 11.000 | 13.02 / 21.59 | 1.66x | 38.86 / 85.16 | 56.9 / 99.2 |
| Q15 | 5.788 / 8.000 | 11.98 / 18.45 | 1.54x | 34.92 / 61.41 | 51.4 / 71.2 |
| Q16 | 0.427 / 11.000 | 6.16 / 21.76 | 3.53x | 6.05 / 89.75 | 12.4 / 98.9 |
| Q17 | 0.797 / 18.000 | 6.46 / 29.58 | 4.58x | 6.41 / 161.21 | 13.2 / 161.5 |
| Q18 | 5.214 / 16.000 | 11.39 / 26.95 | 2.37x | 32.47 / 134.73 | 42.2 / 159.3 |
| Q19 | 8.166 / 20.000 | 14.37 / 31.85 | 2.22x | 75.58 / 183.72 | 93.4 / 162.1 |
| Q20 | 1.906 / 2.000 | 7.87 / 11.93 | 1.52x | 10.30 / 18.42 | 14.9 / 35.1 |
| Q21 | 21.600 / 15.000 | 27.83 / 25.27 | 0.91x | 137.79 / 91.75 | 136.9 / 94.0 |
| Q22 | 22.087 / 15.000 | 28.53 / 26.56 | 0.93x | 170.83 / 103.41 | 169.6 / 106.6 |
| Q23 | 19.608 / 20.000 | 25.97 / 31.06 | 1.20x | 142.91 / 125.70 | 148.9 / 129.2 |
| Q24 | 29.671 / 41.000 | 36.16 / 53.81 | 1.49x | 152.31 / 213.32 | 164.4 / 208.2 |
| Q25 | 3.751 / 5.000 | 9.87 / 15.34 | 1.55x | 17.15 / 33.03 | 26.7 / 44.2 |
| Q26 | 4.877 / 5.000 | 11.10 / 14.35 | 1.29x | 27.03 / 31.64 | 41.2 / 37.0 |
| Q27 | 4.403 / 5.000 | 10.44 / 14.82 | 1.42x | 18.08 / 33.15 | 28.4 / 43.0 |
| Q28 | 9.883 / 17.000 | 16.26 / 28.09 | 1.73x | 94.41 / 102.00 | 53.0 / 103.9 |
| Q29 | 0.597 / 88.000 | 6.41 / 101.33 | 15.82x | 6.38 / 695.93 | 13.2 / 151.7 |
| Q30 | 3.486 / 4.000 | 9.56 / 14.70 | 1.54x | 15.75 / 20.23 | 19.4 / 32.9 |
| Q31 | 6.564 / 8.000 | 12.72 / 18.15 | 1.43x | 36.08 / 56.45 | 50.2 / 68.8 |
| Q32 | 6.900 / 8.000 | 13.18 / 18.74 | 1.42x | 41.49 / 61.21 | 57.4 / 77.0 |
| Q33 | 7.722 / 19.000 | 14.01 / 30.37 | 2.17x | 74.34 / 165.34 | 86.0 / 172.6 |
| Q34 | 0.428 / 30.000 | 6.12 / 42.09 | 6.87x | 6.10 / 238.28 | 12.7 / 263.0 |
| Q35 | 0.422 / 31.000 | 6.07 / 43.45 | 7.15x | 6.05 / 251.70 | 12.7 / 270.7 |
| Q36 | 0.627 / 12.000 | 6.22 / 23.21 | 3.73x | 6.19 / 106.44 | 12.7 / 103.6 |
| Q37 | 3.764 / 3.000 | 9.82 / 13.64 | 1.39x | 12.09 / 23.48 | 21.1 / 38.2 |
| Q38 | 3.031 / 3.000 | 8.85 / 13.11 | 1.48x | 10.87 / 21.87 | 19.5 / 36.6 |
| Q39 | 2.826 / 3.000 | 8.71 / 12.68 | 1.46x | 10.99 / 20.85 | 19.5 / 35.5 |
| Q40 | 10.819 / 5.000 | 16.85 / 15.13 | 0.90x | 18.94 / 28.99 | 29.9 / 45.2 |
| Q41 | 1.670 / 3.000 | 7.54 / 12.75 | 1.69x | 10.28 / 19.99 | 17.4 / 36.8 |
| Q42 | 1.725 / 2.000 | 7.52 / 12.46 | 1.66x | 9.80 / 19.71 | 17.2 / 35.8 |
| Q43 | 1.827 / 2.000 | 7.66 / 12.40 | 1.62x | 8.91 / 19.32 | 16.9 / 34.0 |

## 10M queries

| Q | SQL ms R / D | Process wall ms R / D | Wall D/R | CPU ms R / D | Peak RSS MiB R / D |
| --- | ---: | ---: | ---: | ---: | ---: |
| Q1 | 0.162 / 1.000 | 4.76 / 11.70 | 2.46x | 4.73 / 13.42 | 11.7 / 27.2 |
| Q2 | 0.194 / 5.000 | 4.77 / 16.11 | 3.38x | 4.75 / 31.51 | 11.7 / 50.7 |
| Q3 | 0.193 / 9.000 | 4.84 / 20.27 | 4.19x | 4.81 / 54.92 | 11.7 / 72.5 |
| Q4 | 0.170 / 12.000 | 4.80 / 23.28 | 4.85x | 4.78 / 84.30 | 11.7 / 98.0 |
| Q5 | 0.173 / 50.000 | 4.77 / 63.90 | 13.39x | 4.75 / 614.04 | 11.7 / 268.0 |
| Q6 | 0.174 / 44.000 | 4.88 / 59.54 | 12.21x | 4.85 / 565.97 | 11.7 / 277.9 |
| Q7 | 0.192 / 2.000 | 4.90 / 12.69 | 2.59x | 4.85 / 14.37 | 11.7 / 28.7 |
| Q8 | 0.355 / 5.000 | 5.15 / 15.73 | 3.05x | 5.04 / 37.56 | 12.7 / 53.2 |
| Q9 | 62.225 / 62.000 | 67.72 / 78.04 | 1.15x | 676.65 / 809.36 | 312.2 / 327.3 |
| Q10 | 67.371 / 77.000 | 83.25 / 93.38 | 1.12x | 792.27 / 1053.55 | 324.7 / 378.0 |
| Q11 | 11.325 / 23.000 | 16.64 / 35.58 | 2.14x | 122.93 / 222.81 | 106.1 / 179.7 |
| Q12 | 12.143 / 25.000 | 17.47 / 37.63 | 2.15x | 141.26 / 241.59 | 110.7 / 189.5 |
| Q13 | 0.512 / 38.000 | 5.31 / 53.69 | 10.12x | 5.28 / 436.55 | 12.9 / 292.0 |
| Q14 | 18.739 / 67.000 | 24.36 / 84.26 | 3.46x | 221.41 / 814.48 | 161.6 / 461.5 |
| Q15 | 15.825 / 42.000 | 21.32 / 56.66 | 2.66x | 177.33 / 470.24 | 139.2 / 322.7 |
| Q16 | 0.873 / 51.000 | 5.65 / 66.21 | 11.71x | 5.63 / 718.60 | 13.4 / 327.3 |
| Q17 | 2.366 / 105.000 | 7.31 / 126.53 | 17.30x | 7.29 / 1402.64 | 17.7 / 672.1 |
| Q18 | 14.920 / 82.000 | 20.24 / 102.57 | 5.07x | 180.84 / 994.90 | 97.4 / 639.1 |
| Q19 | 61.812 / 144.000 | 67.89 / 170.18 | 2.51x | 862.67 / 2058.97 | 538.7 / 1018.2 |
| Q20 | 2.509 / 8.000 | 8.34 / 19.29 | 2.31x | 17.17 / 68.18 | 27.5 / 97.0 |
| Q21 | 85.957 / 85.000 | 109.46 / 103.41 | 0.94x | 1043.84 / 941.10 | 483.0 / 636.2 |
| Q22 | 96.414 / 94.000 | 129.15 / 114.36 | 0.89x | 1125.27 / 923.57 | 726.7 / 707.3 |
| Q23 | 88.544 / 122.000 | 113.23 / 142.94 | 1.26x | 784.34 / 1053.61 | 499.2 / 771.7 |
| Q24 | 111.330 / 139.000 | 117.72 / 166.25 | 1.41x | 806.33 / 939.15 | 524.2 / 578.7 |
| Q25 | 6.611 / 12.000 | 12.21 / 23.20 | 1.90x | 30.94 / 79.02 | 45.4 / 70.2 |
| Q26 | 15.968 / 17.000 | 21.33 / 28.53 | 1.34x | 87.35 / 179.70 | 78.4 / 95.9 |
| Q27 | 10.714 / 11.000 | 16.09 / 21.95 | 1.36x | 37.13 / 89.63 | 53.5 / 70.7 |
| Q28 | 61.615 / 90.000 | 73.19 / 109.80 | 1.50x | 769.33 / 897.99 | 131.5 / 665.2 |
| Q29 | 0.567 / 765.000 | 5.53 / 798.73 | 144.45x | 5.50 / 7423.91 | 13.4 / 825.2 |
| Q30 | 5.857 / 9.000 | 11.34 / 21.30 | 1.88x | 46.55 / 49.10 | 45.9 / 54.7 |
| Q31 | 19.976 / 61.000 | 25.37 / 76.54 | 3.02x | 245.11 / 442.39 | 145.5 / 304.6 |
| Q32 | 23.748 / 50.000 | 29.31 / 66.44 | 2.27x | 302.83 / 535.79 | 191.2 / 396.8 |
| Q33 | 67.553 / 141.000 | 92.61 / 163.59 | 1.77x | 963.08 / 2057.22 | 495.9 / 927.6 |
| Q34 | 0.440 / 200.000 | 5.09 / 231.68 | 45.54x | 5.06 / 2595.27 | 12.8 / 1559.2 |
| Q35 | 0.443 / 221.000 | 5.15 / 256.39 | 49.81x | 5.11 / 2840.04 | 12.9 / 1583.9 |
| Q36 | 0.449 / 59.000 | 5.21 / 74.08 | 14.22x | 5.18 / 813.88 | 12.9 / 360.5 |
| Q37 | 12.538 / 11.000 | 17.87 / 22.19 | 1.24x | 29.38 / 49.05 | 62.7 / 59.6 |
| Q38 | 7.710 / 6.000 | 12.88 / 16.53 | 1.28x | 21.51 / 32.45 | 48.8 / 44.5 |
| Q39 | 12.130 / 6.000 | 17.34 / 17.10 | 0.99x | 26.47 / 32.18 | 61.6 / 44.5 |
| Q40 | 18.925 / 19.000 | 24.01 / 30.86 | 1.29x | 42.83 / 71.70 | 70.2 / 85.2 |
| Q41 | 2.911 / 5.000 | 7.91 / 15.72 | 1.99x | 14.78 / 29.26 | 25.4 / 43.1 |
| Q42 | 3.064 / 5.000 | 8.11 / 15.22 | 1.88x | 13.72 / 28.51 | 25.9 / 41.3 |
| Q43 | 2.518 / 5.000 | 7.44 / 15.52 | 2.09x | 13.16 / 28.17 | 24.0 / 38.5 |

## Raw runs and next bottleneck

The exact-parent run is `gram-sieve-rebased`, the DuckDB run is `gram-sieve-duck155-rebased`, and each query has its stdout, stderr, and process resource JSON.
RuDB also has a per-query metrics JSON file.
The earlier `gram-sieve-lazy` directory has the pre-rebase load records, and its Callgrind files are the diagnostic instruction evidence.
The `gram-sieve-two-hash` Q40 `pread` traces show why eager signature loading was rejected.
Q21 and Q22 still decode and copy URL blocks whose signatures pass the filter or whose patterns cannot use it.
The next profile should quantify false-positive block work, dictionary cache misses, and the remaining native scan and aggregation costs before expanding the signature or adding another fast path.
