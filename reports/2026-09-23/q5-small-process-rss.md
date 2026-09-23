# Q5 fresh-process memory and time

ClickBench Q5 is `SELECT COUNT(DISTINCT UserID) FROM hits`. Rudb already has the exact non-null distinct count in the checksummed native catalog. A fresh read-only CSV invocation still built a general one-row result to print that integer. The new path recognizes one unquoted `SELECT COUNT(DISTINCT column) FROM table` statement and prints the certified count directly. Other SQL shapes and files without the certificate use the ordinary engine path.

## Final run after rebase

The candidate was rebased onto rudb main `052f2059`, then rebuilt and measured. The release CLI SHA-256 is `2caf274435ed63f5ccf7d7e2e4c9c9a8cb8bc96502a302b99bbf9f5b1efb0b87`. DuckDB is v2.0.0-dev84237 (`cc7e7bac7f`), SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. Both engines used the same SQL and matching native row counts. The runner alternated engines for 51 fresh processes per size, checked every complete CSV answer, and used `wait4` for child wall time, CPU time, and peak RSS. Cells are medians.

| Rows | Rudb wall ms | DuckDB wall ms | DuckDB / rudb wall | Rudb CPU ms | DuckDB CPU ms | Rudb peak MiB | DuckDB peak MiB | DuckDB / rudb RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 1.127 | 22.991 | 20.4x | 1.099 | 28.163 | 4.02 | 42.59 | 10.60x |
| 10k | 1.584 | 23.094 | 14.6x | 1.550 | 28.105 | 4.02 | 42.81 | 10.65x |
| 1m | 1.115 | 32.059 | 28.7x | 1.092 | 74.566 | 4.02 | 90.56 | 22.53x |
| 10m | 2.115 | 73.800 | 34.9x | 2.092 | 855.726 | 4.02 | 365.09 | 90.83x |

The host had 32 logical CPUs and load average 4.28 at the end. The expected answers were `1000`, `9990`, `898913`, and `5606406`, respectively. All 408 invocations returned the expected complete CSV row. The final [raw samples](q5-small-process-rss/) are `q5-rebased-*.json`.

## Same-base change comparison

The unchanged rudb main was `acd46011`, release SHA-256 `f33120dff63c015f8dd02253cdf16072c84d741aa6674f0178c423329b21b8d6`. The candidate built from that same commit plus this change was SHA-256 `2b94f94cad56c2634e3cd77c3fab53ed9078daca2478219f06f8779a0465ec96`. These trials alternated unchanged rudb, the candidate, and DuckDB 51 times per size against the same native files.

| Rows | Main rudb wall ms | New rudb wall ms | DuckDB wall ms | Main rudb peak MiB | New rudb peak MiB | DuckDB peak MiB |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 1.237 | 1.053 | 22.675 | 5.11 | 4.04 | 42.57 |
| 10k | 1.702 | 1.510 | 23.036 | 5.11 | 4.04 | 42.84 |
| 1m | 1.242 | 1.040 | 31.901 | 5.11 | 4.04 | 93.09 |
| 10m | 2.248 | 2.045 | 73.048 | 5.11 | 4.04 | 363.53 |

The direct output saves about 1.07 MiB of peak process memory on that base. The shape and null tests, Clippy with warnings denied, and release build passed again after rebase. The [runner](../../scripts/q5-fresh-process.py), [resource helper](../../scripts/measure-child.c), and [raw samples](q5-small-process-rss/) preserve the method. The three-way files are `q5-three-*.json`; the earlier two-way candidate and DuckDB files are `q5-direct-*.json`.
