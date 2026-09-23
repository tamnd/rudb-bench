# Q4 fresh-process check on the current native path

Q4 is `SELECT AVG(UserID) FROM hits`. RuDB reads the column's exact integer sum and non-null count from the native catalog, checks the table directory, and divides when SQL runs. The file stores column statistics, not the Q4 result. This run checks the release binary rebuilt after the Q2 direct-count change and its rebase on main.

Each row is the median of 51 alternating fresh-process pairs. Both engines read native databases loaded with the same `CREATE TABLE`, `INSERT` from Parquet, and `CHECKPOINT` SQL. Every complete CSV answer matched DuckDB. The C helper measures only the SQL child with `wait4`, including process startup, file open, output, and exit. The operating system page cache was not cleared.

| Rows | RuDB wall | DuckDB wall | DuckDB / RuDB wall | RuDB peak RSS | DuckDB peak RSS | DuckDB / RuDB RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 21.701 ms | 758.234 ms | 34.94x | 3.75 MiB | 35.40 MiB | 9.44x |
| 10k | 36.738 ms | 725.319 ms | 19.74x | 3.75 MiB | 35.28 MiB | 9.41x |
| 1m | 24.390 ms | 614.853 ms | 25.21x | 3.75 MiB | 45.28 MiB | 12.07x |
| 10m | 29.213 ms | 1007.853 ms | 34.50x | 3.75 MiB | 65.18 MiB | 17.38x |

The eight-CPU shared host had load averages far above eight during these runs. The alternating comparison records what happened under that contention; the wall times and ratios are not quiet-host latency estimates. Q4 clears the measured 10x wall-time ratio at every size, but peak RSS remains below the 10x reduction target at 1k and 10k. The same fixed executable, libc, and allocator pages limit Q2 and Q3 on this host. Changing `UserID` statistics or the native row encoding will not remove those process pages.

The expected answers were `2.414420660257356e+18`, `2.534231104689841e+18`, `2.528885963832509e+18`, and `2.5131007489380997e+18` in size order. The 10m Parquet source on this host contains exactly 10,000,000 rows and differs from the earlier 9,999,750-row file. RuDB source commit `b1d1a0f6` produced binary SHA-256 `86df52f51e61f1b5fb4edf84172c11b8335b22eb8b7a9a6ac4273cc432d0c05f`. DuckDB is v2.0.0-dev84237, SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The [raw records](q4-current-main/) and [runner](../../scripts/q4-fresh-process.py) preserve the comparison.
