# Q2 from generic column frequencies

The earlier Q2 fast path stored a `nonzero` count in the native catalog. That value was the exact result of `AdvEngineID <> 0` on this data. It does not meet the rule that the file may store reusable column statistics but not a query answer. [RuDB PR #1619](https://github.com/tamnd/rudb/pull/1619) stops writing that count and ignores it in existing files. For a column with a complete numeric frequency table, Q2 now sums the counts for non-null values other than zero at query time. If that table is incomplete, the existing directory synopsis path derives the count, or regular execution handles it.

Each row below is the median of 51 alternating fresh-process pairs on the same SQL, `SELECT COUNT(*) FROM hits WHERE AdvEngineID <> 0`. Both engines read their native files. Every output was checked against DuckDB's answer. Wall time includes process startup, file open, query execution, output, and exit. Peak RSS comes from the SQL child through `wait4`. The operating system page cache was not cleared.

| Rows | Answer | RuDB wall | DuckDB wall | DuckDB / RuDB | RuDB RSS | DuckDB RSS | DuckDB / RuDB |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 6 | 1.145 ms | 23.348 ms | 20.39x | 5.41 MiB | 40.07 MiB | 7.41x |
| 10k | 62 | 1.552 ms | 24.724 ms | 15.93x | 5.41 MiB | 40.07 MiB | 7.41x |
| 1m | 6,284 | 1.150 ms | 24.739 ms | 21.51x | 5.41 MiB | 41.82 MiB | 7.73x |
| 10m | 63,365 | 2.063 ms | 32.764 ms | 15.88x | 5.41 MiB | 65.33 MiB | 12.08x |

The time target remains met at every size. The 10x memory target is met at 10m, but not at the three smaller sizes. Child CPU time below one millisecond is too coarse in these records to interpret, so the comparison uses wall time and RSS. This is a Q2 result only, not a full ClickBench result.

The RuDB binary SHA-256 is `ed9be7df9ce8aa3436fba7bff78f5fc0b2735b486d7523c576b45cdb77c26537`. The test used the existing certified native files, which still contain the legacy count. The new binary ignores that field. A native writer test checks that a newly written file leaves it empty while the derived answer remains correct. The [raw records](q2-generic-frequency/) and [fresh-process runner](../../scripts/q2-fresh-process.py) preserve the measurements.
