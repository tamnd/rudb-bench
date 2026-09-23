# Q3 fresh-process check on current main

ClickBench Q3 is `SELECT SUM(AdvEngineID), COUNT(*), AVG(ResolutionWidth) FROM hits`. Rudb answers the three aggregates from certified native metadata. This run checks that later engine changes have not lost the fresh-process result.

The rudb source is main commit `20c27ba9`. Its rebuilt release CLI has SHA-256 `3a62e35475884be3842a0c42e1aeb1782df105760d76a3a779f235335b5459d7`. DuckDB is v2.0.0-dev84237 (`cc7e7bac7f`), SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. Both engines ran the same SQL against native files with the same row counts. Each statement used a new process. The runner alternated engine order across 51 trials per size, checked the complete CSV answer on every trial, and used `wait4` for child wall time, CPU time, and peak RSS. Values below are medians.

| Rows | Rudb wall ms | DuckDB wall ms | DuckDB / rudb wall | Rudb CPU ms | DuckDB CPU ms | Rudb peak MiB | DuckDB peak MiB | DuckDB / rudb RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 2.048 | 66.395 | 32.4x | 1.531 | 45.674 | 4.02 | 40.71 | 10.1x |
| 10k | 4.750 | 69.805 | 14.7x | 2.119 | 45.374 | 4.01 | 40.69 | 10.2x |
| 1m | 2.562 | 76.786 | 30.0x | 1.631 | 56.364 | 4.02 | 44.19 | 11.0x |
| 10m | 4.035 | 137.337 | 34.0x | 2.887 | 114.187 | 4.01 | 88.19 | 22.0x |

The shared 32-logical-CPU host was busy. Load average reached 44.03 at the end of the run, so wall times are less stable than in the earlier low-load [Q3 run](q3-no-dynamic-libm.md). The CPU and RSS medians remain below one tenth of DuckDB's at every size. This run verifies the current build; it does not introduce an engine change.

The [runner](../../scripts/q3-fresh-process.py), [resource helper](../../scripts/measure-child.c), and [raw samples](q3-current-main/) retain the measurement details. The checked CSV rows, in size order, were `37,1000,1503.928`, `627,10000,1517.7663`, `74434,999975,1514.0342458561463`, and `732801,9999750,1513.4569152228805`.
