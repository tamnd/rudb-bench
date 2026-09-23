# Q2 and Q3 small-process allocator check

The [Q2 through Q8 audit](q2-onward-fast-path-audit.md) found no newly stored query answer, but Q2 still missed the 10x peak-RSS target at 1k and 10k. A process map showed that executable, libc, and allocator pages dominate this small query. This experiment checks whether the existing system-allocator build removes enough resident memory to close that gap. It does not change the shipping allocator.

Both RuDB binaries came from source commit `8e64cf8c`. The shipping mimalloc binary has SHA-256 `052ca6ad50fafb733ed81c79b1ed8e238acfb90320e66183b7a1316b7b12054f`; the `--no-default-features` system-allocator binary has SHA-256 `cfdf3fe876cb5c2c418775b900b290f24f1d69cbfa41499062fc6eb04e2615bb`. DuckDB is v2.0.0-dev84237, SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`.

Each row is the median of 11 rotating fresh-process trials of mimalloc, system allocator, and DuckDB on the same SQL. The two RuDB builds opened the same native file; DuckDB opened its native file loaded from the same Parquet rows. Every complete CSV output matched DuckDB. The helper uses `wait4` to measure the child from spawn to exit, including startup, file open, SQL, output, and exit. The operating-system page cache was not cleared.

| Query | Rows | Mimalloc wall | System wall | DuckDB wall | Mimalloc peak RSS | System peak RSS | DuckDB peak RSS | DuckDB / system RSS |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Q2 | 1k | 4.776 ms | 4.164 ms | 210.083 ms | 3.75 MiB | 3.75 MiB | 35.14 MiB | 9.37x |
| Q2 | 10k | 4.723 ms | 3.709 ms | 195.749 ms | 3.75 MiB | 3.75 MiB | 35.27 MiB | 9.41x |
| Q2 | 1m | 4.737 ms | 3.434 ms | 213.779 ms | 3.75 MiB | 3.75 MiB | 38.27 MiB | 10.21x |
| Q2 | 10m | 5.297 ms | 3.536 ms | 218.836 ms | 3.75 MiB | 3.62 MiB | 57.68 MiB | 15.91x |
| Q3 | 1k | 5.019 ms | 4.177 ms | 205.030 ms | 4.50 MiB | 4.25 MiB | 35.64 MiB | 8.39x |
| Q3 | 10k | 6.062 ms | 4.743 ms | 170.424 ms | 4.50 MiB | 4.25 MiB | 35.52 MiB | 8.36x |
| Q3 | 1m | 4.706 ms | 3.898 ms | 194.920 ms | 4.50 MiB | 4.25 MiB | 41.15 MiB | 9.68x |
| Q3 | 10m | 4.981 ms | 3.400 ms | 452.172 ms | 4.50 MiB | 4.25 MiB | 78.43 MiB | 18.45x |

The system allocator saves no measured Q2 memory at 1k or 10k, and saves 0.25 MiB for Q3, which is still short of 10x. The shipping mimalloc build remains the appropriate default for now. Its earlier load tests found a larger cross-thread allocation benefit, and this small-query experiment provides no memory reason to give that up. The shared eight-CPU host had load averages above 40 during these runs, so the wall times and their differences are not quiet-host latency estimates. The peak-RSS result and the unchanged Q2 floor are the useful outcome here.

The [raw allocator samples](q2-allocator-process-footprint/) preserve wall, user-plus-system CPU, and peak RSS for every process. Four additional Q2 files in that directory are an 11-pair check of the shipping binary against DuckDB before the allocator build. The [rotating runner](../../scripts/allocator-fresh-process.py) preserves the method. The 1m source has 999,975 rows; the 10m source has exactly 10,000,000 rows.
