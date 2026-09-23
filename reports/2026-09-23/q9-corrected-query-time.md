# Q9 after removing stored query answers

Q9 groups by `RegionID`, counts distinct `UserID` values, orders by that count, and takes ten rows. This run uses the corrected RuDB release build from [rudb#1604](https://github.com/tamnd/rudb/pull/1604).
The native files contain reusable column statistics but no Q9 grouped-distinct certificate.
The corrected engine also ignores query-answer blocks that may exist in older files.

Each invocation starts a new process, opens a native database, runs the same SQL in RuDB or
DuckDB, prints CSV, and exits. The `wait4` helper measures child wall time, user plus system
CPU time, and peak resident memory. The runner alternates engines for 51 trials at each size.
It checks every output against DuckDB's complete row set and checks descending counts. Ties may appear in a different order because the SQL has no secondary ordering key. All 408 invocations passed.

| Rows | RuDB wall ms | DuckDB wall ms | DuckDB / RuDB time | RuDB CPU ms | DuckDB CPU ms | RuDB peak MiB | DuckDB peak MiB | DuckDB / RuDB RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 7.627 | 25.009 | 3.3x | 7.591 | 33.667 | 13.91 | 46.20 | 3.3x |
| 10k | 14.570 | 25.753 | 1.8x | 14.505 | 34.284 | 16.91 | 46.96 | 2.8x |
| 1m | 14.831 | 36.606 | 2.5x | 62.321 | 109.800 | 80.41 | 117.68 | 1.5x |
| 10m | 78.220 | 156.843 | 2.0x | 534.065 | 1017.584 | 361.90 | 421.50 | 1.2x |

The 10m result is well short of the goal of ten times lower query time and peak RSS than
DuckDB. The aggregate still has to scan columns and count distinct `(RegionID, UserID)` pairs at query time. This is the performance baseline for further engine work, not a success claim.

The RuDB release binary SHA-256 is
`7ba7c5a2dcf04e53442b9980972f044f661d3bf0430e3ab67eae4d567fb8a116`. DuckDB is v2.0.0-dev84237 (`cc7e7bac7f`), SHA-256
`bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`.
The [runner](../../scripts/q9-fresh-process.py),
[raw trial records](q9-corrected-query-time/), and expected CSV files reproduce these medians.
