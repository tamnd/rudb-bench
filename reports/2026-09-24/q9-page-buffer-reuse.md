# Q9: reuse a verified page buffer per worker

The run-projection scanner read each 512 KiB extent into a new vector, then checked its checksum and decoded its rows. This change lets each query worker reuse one vector across its assigned extents. The file format, page checksums, bounds checks, and runtime distinct counting are unchanged. The projection still stores one RegionID code for every original row.

The same Q9 SQL ran in a new process each time: `SELECT RegionID, COUNT(DISTINCT UserID) AS u FROM hits GROUP BY RegionID ORDER BY u DESC LIMIT 10`. The runner checked the complete selected key/count set and descending counts, allowing either order for ties. Cases rotated order. `wait4` measured wall time, user plus system CPU, whole-process peak RSS, page faults, and process I/O counters. The operating-system page cache was warm. The 1k, 10k, and 10m results below use 51 rounds; 1m uses 11. The host was busy, with a load average near 17 when checked afterward, so small wall differences need care.

| Rows | RuDB reused wall | DuckDB native wall | DuckDB / RuDB wall | RuDB CPU | DuckDB CPU | RuDB peak RSS | DuckDB peak RSS | DuckDB / RuDB RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 11.507 ms | 101.439 ms | 8.82x | 9.526 ms | 81.995 ms | 6.25 MiB | 35.99 MiB | 5.76x |
| 10k | 34.290 ms | 97.783 ms | 2.85x | 29.796 ms | 82.631 ms | 6.75 MiB | 36.61 MiB | 5.42x |
| 1m | 37.630 ms | 267.314 ms | 7.10x | 36.697 ms | 272.264 ms | 10.25 MiB | 79.10 MiB | 7.72x |
| 10m | 52.256 ms | 480.428 ms | 9.19x | 70.005 ms | 744.510 ms | 11.12 MiB | 134.27 MiB | 12.08x |

At 10m, the same-file merged RuDB reader took 53.426 ms wall and 72.770 ms CPU in the paired run; the reusable-buffer reader took 52.256 ms wall and 70.005 ms CPU; DuckDB took 480.428 ms wall and 744.510 ms CPU. The median per-trial RuDB differences were 1.860 ms wall and 2.721 ms CPU saved, with the new reader winning 28 and 34 of 51 pairs respectively. This is an encouraging CPU reduction, but the wall result remains below 10x DuckDB and is noisy. At 1m, the new reader saved a median 3.187 ms wall and 4.324 ms CPU across 11 pairs. The 51-round small-table comparisons show no material regression.

The profiler that motivated the change found page-copy and page-fault costs, but the 10m median minor-fault count was 1,983 for **both** RuDB binaries. Median block read bytes were zero for both, consistent with the warm page cache. Reusing the buffer therefore did not measurably reduce 10m faults; the CPU gain may come from less allocation and memory management work, which the current counters do not isolate. The [raw process records](q9-page-buffer/) include every wall, CPU, RSS, fault, and I/O sample. The updated [runner](../../scripts/q9-fresh-process.py) now preserves those counters.

The optional projection's build cost is unchanged. The 10m memory target is met in this run, but wall speed remains 9.19x DuckDB, and smaller sizes still miss both 10x goals. All 194 native tests and strict native Clippy passed. The measured RuDB candidate has SHA-256 `de1d098ea1de75b7050ea857a35029d598c5d3420d8508964e0c49fe4a631679`; the same-source merged reader has `fa17c7b8548730f9411bb52c7606c447d669b689518f4da984496a7a3c2087e4`. DuckDB v2.0.0-dev84237 has SHA-256 `01b6b042bcb8ca60285da2d6e95a5619dfe16149d705794700aa135ebd1416c7`.
