# Q2 direct CSV from complete native frequencies

[RuDB PR #1622](https://github.com/tamnd/rudb/pull/1622) adds nonzero entries from a complete numeric column frequency table when Q2 runs and prints the count directly. The native file stores no filtered count. A file without a complete table, or SQL outside the simple filtered-count shape, uses the regular execution path. The change removes the general one-row result from this fresh-process CSV case.

The 51 alternating fresh-process pairs at each size used the same SQL, `SELECT COUNT(*) FROM hits WHERE AdvEngineID <> 0`, against native files loaded from the same Parquet source with the same `CREATE TABLE`, `INSERT`, and `CHECKPOINT` SQL. Every complete CSV answer matched DuckDB. Q3 ran as a control because both statements share the CLI binary. The helper measures the SQL child's wall time and peak RSS with `wait4`; the operating system page cache was not cleared.

| Query | Rows | RuDB wall | DuckDB wall | DuckDB / RuDB wall | RuDB peak RSS | DuckDB peak RSS | DuckDB / RuDB RSS |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Q2 | 1k | 24.341 ms | 541.673 ms | 22.25x | 3.88 MiB | 35.15 MiB | 9.07x |
| Q2 | 10k | 12.511 ms | 346.777 ms | 27.72x | 3.75 MiB | 35.15 MiB | 9.37x |
| Q2 | 1m | 10.022 ms | 528.269 ms | 52.71x | 3.88 MiB | 38.28 MiB | 9.88x |
| Q2 | 10m | 14.339 ms | 676.734 ms | 47.20x | 3.88 MiB | 57.91 MiB | 14.94x |
| Q3 | 1k | 14.553 ms | 425.151 ms | 29.21x | 3.75 MiB | 35.66 MiB | 9.51x |
| Q3 | 10k | 12.015 ms | 319.485 ms | 26.59x | 3.75 MiB | 35.65 MiB | 9.51x |
| Q3 | 1m | 9.923 ms | 466.824 ms | 47.05x | 3.75 MiB | 41.40 MiB | 11.04x |
| Q3 | 10m | 13.292 ms | 939.383 ms | 70.68x | 3.75 MiB | 79.18 MiB | 21.11x |

These wall times were recorded on an eight-CPU shared host with load averages far above eight. The alternating runs compare the two children under the same period of contention, but their absolute latency and ratios are not a quiet-host result. The RSS numbers identify the remaining small-file gap: Q2 is 9.07x to 9.88x below DuckDB through 1m, short of the 10x memory target. Q3 also remains short at 1k and 10k on this host. A profile at process exit found roughly 1.3 MiB of RuDB executable pages, 1.4 MiB of libc pages, and 0.3 MiB of allocator pages in the Q2 path. The remaining small-file memory cost is mostly mapped code and runtime, not data rows.

The answers for Q2 were 6, 62, 6,284, and 202,134; Q3 returned `37,1000,1503.928`, `627,10000,1517.7663`, `74434,999975,1514.0342458561463`, and `3152344,10000000,1508.8046441`. This host's 10m Parquet file has exactly 10,000,000 rows and differs from the earlier 9,999,750-row native file. Results from those two 10m files should not be pooled.

The measured RuDB binary is commit `bee49934`, SHA-256 `c7523a2f82e49ab08da1438ba99be0bb74c2fb4b9b287d3556d67f8142758c6a`; DuckDB is v2.0.0-dev84237, SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The engine branch was rebased on `27faf28b` after these runs. The [raw Q2 and Q3 records](q2-direct-csv/) preserve every process sample. The [Q2 runner](../../scripts/q2-fresh-process.py) and [Q3 runner](../../scripts/q3-fresh-process.py) preserve the measurement method.
