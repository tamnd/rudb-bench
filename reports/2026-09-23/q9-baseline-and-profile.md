# ClickBench Q9 baseline and aggregate profile

Q9 is `SELECT RegionID, COUNT(DISTINCT UserID) AS u FROM hits GROUP BY RegionID ORDER BY u DESC LIMIT 10`. Each run opens one native database, executes this SQL, prints CSV, and exits. Rudb and DuckDB use the same SQL and row counts. The resource helper uses `wait4` to measure child wall time, user plus system CPU, and peak resident memory, including process startup. The runner alternates engines for 51 fresh processes per size and reports medians.

The SQL orders rows by count only, so tied rows can appear in either order. The runner checks that counts descend and that the complete set of output rows matches DuckDB's output. This comparison passed for every run. The 1k outputs differed only in the order of three rows tied at 17.

| Rows | Rudb wall ms | DuckDB wall ms | DuckDB / rudb | Rudb CPU ms | DuckDB CPU ms | Rudb peak MiB | DuckDB peak MiB | DuckDB / rudb RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 11.844 | 38.233 | 3.2x | 11.771 | 51.960 | 14.61 | 46.15 | 3.2x |
| 10k | 22.409 | 38.887 | 1.7x | 22.133 | 54.892 | 17.61 | 47.52 | 2.7x |
| 1m | 21.510 | 54.408 | 2.5x | 84.012 | 165.619 | 81.85 | 124.19 | 1.5x |
| 10m | 78.965 | 159.572 | 2.0x | 626.717 | 989.189 | 377.86 | 422.02 | 1.1x |

The shared 32-core host had load average 20.24 after the run. The [10m analyzed plan](q9-fresh-process/explain-10m.txt) attributes about 139 ms CPU to reading `RegionID` and `UserID`, and 246 ms CPU to the grouped distinct aggregate. It reports 247.8 MiB held at its peak. Rudb already uses a radix exchange for this shape; the exchange buffers wide `(group, user)` records and deduplicates them in partition hash tables. The ten-row TopN is a small part of the cost. `UserID` has 5,606,406 distinct values over a broad signed 64-bit range on this file, so a dense bitmap is not a fit. The 10m DuckDB result sets a target below 16 ms wall and about 42 MiB peak RSS, which requires a different way to provide or stream exact pair counts.

The rudb release binary has SHA-256 `55a2641873a30826caca61e1481f6dff9d3f883e2745aa7262f1da820744742a`; DuckDB v2.0.0-dev84237 (`cc7e7bac7f`) has SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The [runner](../../scripts/q9-fresh-process.py), [expected CSV files](q9-fresh-process/expected/), and [raw samples](q9-fresh-process/) reproduce the comparison.
