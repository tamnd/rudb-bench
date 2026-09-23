# Native substring signatures and the 1M and 10M ClickBench gap

The measured RuDB candidate is commit `8a62e7cd` on parent `97a146a2`, compared with that exact parent and DuckDB 1.5.5 on `gpc`.
This report covers all 43 original queries at 999,975 and 9,999,750 rows.
The RuDB and DuckDB query files have the same SHA-256, `274ffe1c4f83baad2fc1773bb6773bbdab0db779faeb2970de8cc531292a5dc6`.
Both engines loaded from the same Parquet source at each size, and row counts and the `UserID` and `WatchID` sums match.
The raw runs are under `/home/gopher/clickbench-native-audit/20260923-frequency-pairs` on `gpc`.

## Root cause and change

One-thread Callgrind attributed about 94% of Q21 and Q22 instructions to the scan and pushed filter on the pre-rebase parent.
Q21 counted 1,649,688,689 instructions and Q22 counted 1,654,009,076 before the change.
In Q21, `memset` cost 224,344,019 instructions, `memcpy` 180,005,254, FSST decompression 177,238,474, and LZ replay 88,487,842.
The substring matching itself was only about 1% of the count.
The native dictionary's stable `LIKE` path decoded high-cardinality URL values before discovering that a required literal was absent from nearly every 1,024-value payload block.
Only 1,577 rows match Q21 at 10M, so this is wasted decode and copy work rather than a slow substring matcher.

Native format 28 stores a 2 KiB two-hash four-byte-gram signature for each 1,024-value dictionary payload block.
For a case-sensitive literal `LIKE` containing at least four bytes, the scan lazily reads the checksummed signatures and skips a block only when a required gram is certainly absent.
Possible matches still use the exact matcher, and `NOT LIKE` converts a proven absent block to true.
Older native format 27 remains readable, and unsupported patterns keep the original path.
The signature section is after the lazy rank data, so unrelated queries do not read it when opening the dictionary.
A single-bit signature was too collision-prone in a URL sample: 93 of 101 blocks passed a `%google%` precheck although only six had a match.
The two-hash sample passed six of 20 blocks with three real matches; these are diagnostic samples, not production skip rates.
An eager placement regressed Q37 through Q40 because Q40 read 37.38 MB instead of 23.91 MB; the lazy layout removed that extra read.
A thread-local read-buffer experiment also lost on 10M Q21 and Q22 and was not merged.

The pre-rebase one-thread 1M Callgrind count fell from 1,645,172,533 to 1,355,175,258 for Q21 and from 1,850,217,141 to 1,557,260,085 for Q24.
That is 17.6% and 15.8% fewer diagnostic instructions, not production latency.
The final rebase includes the parent's dictionary-block cache change and a checked-subtraction fallback for an inherited extreme-integer count test failure.
The final build passed 154 native, 288 kernel, and 478 executor tests, strict Clippy, formatting, and all 43 original queries at both sizes.
Some untied result renderings differ only in tied order; the earlier deterministic secondary-order check made Q22 and Q23 byte-identical.

## Exact-parent A/B

Each original query ran nine times per build in a fresh process, alternating parent and candidate on the same host.
The table sums per-query medians; peak RSS is the largest observed query process, not a sum.

| Rows | Build | SQL median sum | Process wall median sum | CPU median sum | Largest query peak RSS |
| --- | --- | ---: | ---: | ---: | ---: |
| 1M | Parent | 0.244215 s | 0.503239 s | 1.620215 s | 184.2 MiB |
| 1M | Signature | 0.228075 s | 0.487572 s | 1.522877 s | 172.3 MiB |
| 10M | Parent | 0.958989 s | 1.316347 s | 10.397641 s | 791.1 MiB |
| 10M | Signature | 0.932051 s | 1.280028 s | 9.853118 s | 734.7 MiB |

The 1M process-wall median sum fell 3.11%, and the 10M sum fell 2.76%.
At 10M, Q21 fell from 116.145 to 109.613 ms and Q24 from 119.640 to 116.729 ms.
The new parent cache had already cut Q24 to 119.640 ms, so the signature's incremental Q24 gain is only 2.43%.
The earlier 20% Q21 and Q24 acceptance gate is not met; Q21 improved 5.62% against this parent.
This is an incremental full-suite gain, not completion of that gate or of the 10x goal.

The format 28 file is 210,036,929 bytes at 1M against 206,788,933 bytes for format 27, a 1.57% increase.
At 10M it is 1,684,821,790 against 1,662,658,931 bytes, a 1.33% increase.
Single pre-rebase native load runs were 13.101 versus 13.046 seconds at 1M and 17.602 versus 15.913 seconds at 10M, candidate versus parent.
Those single load numbers were not repeated or taken with the final rebased binaries, so they indicate a possible load penalty but are not a stable load-time comparison.

## Same-host DuckDB comparison

DuckDB 1.5.5 ran the same 43 original queries nine times each in a fresh process with 16 threads and a 16 GiB memory limit.
RuDB also used 16 threads.
The final RuDB candidate and DuckDB runs used the same host and source data; these cross-engine runs were sequential, not alternating.
The page cache was not dropped and the first run was not disk cold.
Each table cell reports the median of nine runs, not the fastest.
`SQL ms` is RuDB's internal total or DuckDB's CLI timer; DuckDB rounds its timer to whole milliseconds.
`Process wall ms` includes process startup, database open, query, and output.
`CPU ms` is process user plus system CPU.
`Peak RSS MiB` is the per-process resident high-water mark.
In a paired cell, `R / D` means RuDB followed by DuckDB, and wall `D/R` above one means RuDB was faster.

| Rows | Engine | Completed | SQL median sum | Process wall median sum | CPU median sum | Largest query peak RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1M | RuDB | 43/43 | 0.228075 s | 0.487572 s | 1.522877 s | 172.3 MiB |
| 1M | DuckDB 1.5.5 | 43/43 | 0.509000 s | 0.965573 s | 3.827589 s | 272.2 MiB |
| 10M | RuDB | 43/43 | 0.932051 s | 1.280028 s | 9.853118 s | 734.7 MiB |
| 10M | DuckDB 1.5.5 | 43/43 | 2.985000 s | 3.679567 s | 32.840637 s | 1766.5 MiB |

At 10M, the workload lead is 3.20x by SQL median sum and 2.87x by process-wall median sum.
Q21 and Q22 remain slower than DuckDB by 10M process wall, and Q22 uses more peak RSS.
The workload-level 10x target remains open.

## 1M queries

| Q | SQL ms R / D | Process wall ms R / D | Wall D/R | CPU ms R / D | Peak RSS MiB R / D |
| --- | ---: | ---: | ---: | ---: | ---: |
| Q1 | 0.173 / 0.000 | 5.96 / 10.21 | 1.71x | 5.93 / 11.82 | 11.4 / 26.5 |
| Q2 | 0.197 / 2.000 | 5.90 / 10.95 | 1.86x | 5.87 / 13.86 | 11.4 / 28.4 |
| Q3 | 0.200 / 2.000 | 5.85 / 11.36 | 1.94x | 5.82 / 16.16 | 11.4 / 30.2 |
| Q4 | 0.187 / 2.000 | 5.82 / 11.98 | 2.06x | 5.79 / 18.52 | 11.4 / 35.7 |
| Q5 | 0.176 / 12.000 | 5.95 / 22.37 | 3.76x | 5.88 / 72.00 | 11.4 / 81.5 |
| Q6 | 0.189 / 8.000 | 5.82 / 18.68 | 3.21x | 5.80 / 58.32 | 11.4 / 61.4 |
| Q7 | 0.202 / 1.000 | 5.84 / 10.40 | 1.78x | 5.81 / 11.75 | 11.4 / 27.0 |
| Q8 | 0.391 / 2.000 | 6.16 / 11.33 | 1.84x | 6.13 / 15.84 | 12.4 / 30.0 |
| Q9 | 8.160 / 15.000 | 14.44 / 25.65 | 1.78x | 54.61 / 103.87 | 78.6 / 99.3 |
| Q10 | 8.652 / 16.000 | 15.08 / 26.92 | 1.78x | 73.97 / 136.31 | 88.0 / 110.2 |
| Q11 | 4.496 / 6.000 | 10.71 / 16.18 | 1.51x | 29.41 / 46.42 | 41.2 / 58.4 |
| Q12 | 4.945 / 6.000 | 11.06 / 16.09 | 1.45x | 31.43 / 47.93 | 44.1 / 60.7 |
| Q13 | 0.515 / 7.000 | 6.46 / 17.84 | 2.76x | 6.43 / 56.61 | 12.7 / 67.3 |
| Q14 | 6.768 / 11.000 | 13.24 / 21.59 | 1.63x | 43.00 / 85.16 | 57.2 / 99.2 |
| Q15 | 6.115 / 8.000 | 12.28 / 18.45 | 1.50x | 38.69 / 61.41 | 51.2 / 71.2 |
| Q16 | 0.460 / 11.000 | 6.33 / 21.76 | 3.44x | 6.30 / 89.75 | 12.4 / 98.9 |
| Q17 | 0.813 / 18.000 | 6.58 / 29.58 | 4.50x | 6.55 / 161.21 | 13.2 / 161.5 |
| Q18 | 5.874 / 16.000 | 11.86 / 26.95 | 2.27x | 33.58 / 134.73 | 41.2 / 159.3 |
| Q19 | 8.922 / 20.000 | 15.25 / 31.85 | 2.09x | 75.25 / 183.72 | 93.0 / 162.1 |
| Q20 | 1.931 / 2.000 | 8.08 / 11.93 | 1.48x | 10.74 / 18.42 | 14.9 / 35.1 |
| Q21 | 21.274 / 15.000 | 27.46 / 25.27 | 0.92x | 143.78 / 91.75 | 136.4 / 94.0 |
| Q22 | 22.673 / 15.000 | 29.11 / 26.56 | 0.91x | 173.37 / 103.41 | 168.2 / 106.6 |
| Q23 | 19.235 / 20.000 | 25.58 / 31.06 | 1.21x | 148.53 / 125.70 | 149.2 / 129.2 |
| Q24 | 30.113 / 41.000 | 36.44 / 53.81 | 1.48x | 159.49 / 213.32 | 165.9 / 208.2 |
| Q25 | 3.729 / 5.000 | 9.93 / 15.34 | 1.54x | 18.51 / 33.03 | 26.5 / 44.2 |
| Q26 | 5.136 / 5.000 | 11.32 / 14.35 | 1.27x | 28.30 / 31.64 | 41.4 / 37.0 |
| Q27 | 4.580 / 5.000 | 10.81 / 14.82 | 1.37x | 19.76 / 33.15 | 28.4 / 43.0 |
| Q28 | 10.548 / 17.000 | 16.80 / 28.09 | 1.67x | 86.47 / 102.00 | 52.9 / 103.9 |
| Q29 | 0.624 / 88.000 | 6.61 / 101.33 | 15.33x | 6.58 / 695.93 | 12.9 / 151.7 |
| Q30 | 3.782 / 4.000 | 9.74 / 14.70 | 1.51x | 16.44 / 20.23 | 22.2 / 32.9 |
| Q31 | 6.128 / 8.000 | 12.46 / 18.15 | 1.46x | 40.83 / 56.45 | 50.5 / 68.8 |
| Q32 | 6.432 / 8.000 | 12.76 / 18.74 | 1.47x | 45.78 / 61.21 | 56.7 / 77.0 |
| Q33 | 8.219 / 19.000 | 14.56 / 30.37 | 2.09x | 78.70 / 165.34 | 85.2 / 172.6 |
| Q34 | 0.458 / 30.000 | 6.42 / 42.09 | 6.56x | 6.38 / 238.28 | 12.4 / 263.0 |
| Q35 | 0.440 / 31.000 | 6.19 / 43.45 | 7.02x | 6.13 / 251.70 | 12.7 / 270.7 |
| Q36 | 0.624 / 12.000 | 6.29 / 23.21 | 3.69x | 6.26 / 106.44 | 12.6 / 103.6 |
| Q37 | 3.344 / 3.000 | 9.24 / 13.64 | 1.48x | 11.52 / 23.48 | 20.4 / 38.2 |
| Q38 | 3.261 / 3.000 | 9.20 / 13.11 | 1.42x | 11.34 / 21.87 | 18.9 / 36.6 |
| Q39 | 2.363 / 3.000 | 8.35 / 12.68 | 1.52x | 10.36 / 20.85 | 18.7 / 35.5 |
| Q40 | 10.468 / 5.000 | 16.54 / 15.13 | 0.91x | 18.69 / 28.99 | 29.6 / 45.2 |
| Q41 | 1.750 / 3.000 | 7.75 / 12.75 | 1.64x | 10.06 / 19.99 | 17.3 / 36.8 |
| Q42 | 1.652 / 2.000 | 7.62 / 12.46 | 1.64x | 9.44 / 19.71 | 17.2 / 35.8 |
| Q43 | 1.875 / 2.000 | 7.74 / 12.40 | 1.60x | 9.18 / 19.32 | 16.5 / 34.0 |

## 10M queries

| Q | SQL ms R / D | Process wall ms R / D | Wall D/R | CPU ms R / D | Peak RSS MiB R / D |
| --- | ---: | ---: | ---: | ---: | ---: |
| Q1 | 0.184 / 1.000 | 5.10 / 11.70 | 2.29x | 5.07 / 13.42 | 11.7 / 27.2 |
| Q2 | 0.197 / 5.000 | 4.98 / 16.11 | 3.23x | 4.95 / 31.51 | 11.7 / 50.7 |
| Q3 | 0.211 / 9.000 | 5.08 / 20.27 | 3.99x | 5.06 / 54.92 | 11.7 / 72.5 |
| Q4 | 0.184 / 12.000 | 4.99 / 23.28 | 4.67x | 4.96 / 84.30 | 11.7 / 98.0 |
| Q5 | 0.198 / 50.000 | 4.98 / 63.90 | 12.84x | 4.95 / 614.04 | 11.7 / 268.0 |
| Q6 | 0.202 / 44.000 | 5.05 / 59.54 | 11.79x | 5.02 / 565.97 | 11.6 / 277.9 |
| Q7 | 0.222 / 2.000 | 4.99 / 12.69 | 2.55x | 4.96 / 14.37 | 11.7 / 28.7 |
| Q8 | 0.406 / 5.000 | 5.42 / 15.73 | 2.90x | 5.39 / 37.56 | 12.7 / 53.2 |
| Q9 | 59.879 / 62.000 | 71.29 / 78.04 | 1.09x | 689.69 / 809.36 | 310.7 / 327.3 |
| Q10 | 69.274 / 77.000 | 81.43 / 93.38 | 1.15x | 807.25 / 1053.55 | 323.7 / 378.0 |
| Q11 | 11.413 / 23.000 | 17.20 / 35.58 | 2.07x | 126.77 / 222.81 | 107.0 / 179.7 |
| Q12 | 12.424 / 25.000 | 17.92 / 37.63 | 2.10x | 145.17 / 241.59 | 111.2 / 189.5 |
| Q13 | 0.553 / 38.000 | 5.68 / 53.69 | 9.46x | 5.65 / 436.55 | 13.0 / 292.0 |
| Q14 | 19.184 / 67.000 | 24.40 / 84.26 | 3.45x | 227.24 / 814.48 | 162.2 / 461.5 |
| Q15 | 16.232 / 42.000 | 21.85 / 56.66 | 2.59x | 182.39 / 470.24 | 138.4 / 322.7 |
| Q16 | 0.928 / 51.000 | 6.02 / 66.21 | 11.01x | 5.99 / 718.60 | 13.4 / 327.3 |
| Q17 | 2.370 / 105.000 | 7.52 / 126.53 | 16.82x | 7.48 / 1402.64 | 17.7 / 672.1 |
| Q18 | 15.556 / 82.000 | 21.02 / 102.57 | 4.88x | 185.46 / 994.90 | 97.9 / 639.1 |
| Q19 | 61.625 / 144.000 | 85.82 / 170.18 | 1.98x | 878.89 / 2058.97 | 521.9 / 1018.2 |
| Q20 | 2.498 / 8.000 | 8.32 / 19.29 | 2.32x | 16.15 / 68.18 | 27.2 / 97.0 |
| Q21 | 85.452 / 85.000 | 109.61 / 103.41 | 0.94x | 1052.73 / 941.10 | 482.1 / 636.2 |
| Q22 | 92.883 / 94.000 | 126.88 / 114.36 | 0.90x | 1141.17 / 923.57 | 730.7 / 707.3 |
| Q23 | 89.934 / 122.000 | 114.63 / 142.94 | 1.25x | 798.96 / 1053.61 | 499.5 / 771.7 |
| Q24 | 110.993 / 139.000 | 116.73 / 166.25 | 1.42x | 816.45 / 939.15 | 523.4 / 578.7 |
| Q25 | 6.681 / 12.000 | 12.12 / 23.20 | 1.91x | 28.94 / 79.02 | 44.4 / 70.2 |
| Q26 | 15.343 / 17.000 | 21.06 / 28.53 | 1.35x | 94.36 / 179.70 | 78.1 / 95.9 |
| Q27 | 11.319 / 11.000 | 16.91 / 21.95 | 1.30x | 38.19 / 89.63 | 53.9 / 70.7 |
| Q28 | 62.737 / 90.000 | 72.85 / 109.80 | 1.51x | 771.75 / 897.99 | 132.5 / 665.2 |
| Q29 | 0.611 / 765.000 | 5.77 / 798.73 | 138.39x | 5.74 / 7423.91 | 13.4 / 825.2 |
| Q30 | 5.944 / 9.000 | 11.31 / 21.30 | 1.88x | 46.38 / 49.10 | 45.2 / 54.7 |
| Q31 | 19.753 / 61.000 | 25.44 / 76.54 | 3.01x | 252.14 / 442.39 | 145.2 / 304.6 |
| Q32 | 23.616 / 50.000 | 29.25 / 66.44 | 2.27x | 310.79 / 535.79 | 192.2 / 396.8 |
| Q33 | 69.368 / 141.000 | 93.38 / 163.59 | 1.75x | 992.32 / 2057.22 | 492.4 / 927.6 |
| Q34 | 0.522 / 200.000 | 5.53 / 231.68 | 41.88x | 5.49 / 2595.27 | 12.7 / 1559.2 |
| Q35 | 0.489 / 221.000 | 5.42 / 256.39 | 47.30x | 5.39 / 2840.04 | 12.7 / 1583.9 |
| Q36 | 0.501 / 59.000 | 5.40 / 74.08 | 13.73x | 5.37 / 813.88 | 12.9 / 360.5 |
| Q37 | 12.596 / 11.000 | 17.99 / 22.19 | 1.23x | 29.61 / 49.05 | 62.5 / 59.6 |
| Q38 | 8.269 / 6.000 | 13.54 / 16.53 | 1.22x | 22.90 / 32.45 | 48.2 / 44.5 |
| Q39 | 12.733 / 6.000 | 17.87 / 17.10 | 0.96x | 28.30 / 32.18 | 62.0 / 44.5 |
| Q40 | 19.802 / 19.000 | 25.09 / 30.86 | 1.23x | 45.15 / 71.70 | 69.6 / 85.2 |
| Q41 | 2.925 / 5.000 | 8.11 / 15.72 | 1.94x | 15.33 / 29.26 | 25.5 / 43.1 |
| Q42 | 3.200 / 5.000 | 8.30 / 15.22 | 1.83x | 13.33 / 28.51 | 25.7 / 41.3 |
| Q43 | 2.641 / 5.000 | 7.80 / 15.52 | 1.99x | 13.82 / 28.17 | 24.3 / 38.5 |

## Raw runs and next bottleneck

The final exact-parent run is `gram-sieve-latest-main`, the DuckDB run is `gram-sieve-duck155-rebased`, and each query has stdout, stderr, and process resource JSON.
RuDB also has a per-query metrics JSON file.
The earlier `gram-sieve-lazy` directory has the pre-rebase load records and Callgrind diagnostic files.
The `gram-sieve-two-hash` Q40 `pread` traces show why eager signature loading was rejected.
Q21 and Q22 still decode and copy URL blocks whose signatures pass the filter or whose patterns cannot use it.
The next profile should quantify false-positive block work, dictionary cache misses, and the remaining native scan and aggregation costs before expanding the signature or adding another fast path.
