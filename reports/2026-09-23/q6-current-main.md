# Q6 fresh-process check on current main

ClickBench Q6 is `SELECT COUNT(DISTINCT SearchPhrase) FROM hits`. The native catalog stores an exact non-null distinct count for this string column. The direct read-only CSV count path added for Q5 also handles Q6 without a separate engine change. This run checks the merged code on current main, since the earlier [Q6 report](q6-fresh-process.md) missed the 10x small-file memory target.

The rudb source is main `4eb19ad4`. Its rebuilt release CLI has SHA-256 `2caf274435ed63f5ccf7d7e2e4c9c9a8cb8bc96502a302b99bbf9f5b1efb0b87`. DuckDB is v2.0.0-dev84237 (`cc7e7bac7f`), SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. Both engines ran the same SQL against native files with matching row counts. Each statement used a new process. The runner alternated engine order across 51 trials per size, checked every complete CSV answer, and measured child wall time, CPU time, and peak RSS with `wait4`. Cells are medians.

| Rows | Rudb wall ms | DuckDB wall ms | DuckDB / rudb wall | Rudb CPU ms | DuckDB CPU ms | Rudb peak MiB | DuckDB peak MiB | DuckDB / rudb RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 1.149 | 22.716 | 19.8x | 1.128 | 28.191 | 4.02 | 41.36 | 10.29x |
| 10k | 1.616 | 22.616 | 14.0x | 1.594 | 27.940 | 4.02 | 41.74 | 10.38x |
| 1m | 1.101 | 30.371 | 27.6x | 1.079 | 64.226 | 4.02 | 73.01 | 18.16x |
| 10m | 2.134 | 60.871 | 28.5x | 2.109 | 496.268 | 4.02 | 299.85 | 74.60x |

The shared host had 32 logical CPUs and load average 0.98 at the end. Expected counts were `138`, `1282`, `107703`, and `892232` in size order. All 408 invocations matched the complete expected CSV row. The [runner](../../scripts/q6-fresh-process.py), [resource helper](../../scripts/measure-child.c), and [raw samples](q6-current-main/) preserve the method and observations.
