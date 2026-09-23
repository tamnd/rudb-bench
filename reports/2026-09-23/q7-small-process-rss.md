# Q7 fresh-process time and memory

Q7 runs `SELECT MIN(EventDate), MAX(EventDate) FROM hits`. The native catalog already contains exact, checksummed extrema. Before this change, a fresh CLI process built a general query result for two values and loaded code that this query never uses. The CLI now reads the certified bounds as primitive values and prints the CSV row directly. Its own release crate is optimized for size. Engine crates retain the existing release optimization and unwind behavior.

Each trial launched a new process, executed the same SQL on a native file, checked the complete CSV output byte for byte, and measured the child with Linux `wait4`. The runner alternated rudb and DuckDB over 51 trials per size. Wall, CPU, and peak RSS below are medians. These are query process measurements, not load times.

| Rows | Rudb wall ms | DuckDB wall ms | DuckDB / Rudb time | Rudb CPU ms | DuckDB CPU ms | Rudb peak MiB | DuckDB peak MiB | DuckDB / Rudb RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 1.124 | 21.934 | 19.5x | 1.098 | 25.100 | 3.980 | 39.895 | 10.0x |
| 10k | 1.585 | 21.687 | 13.7x | 1.555 | 24.815 | 3.980 | 39.895 | 10.0x |
| 1m | 1.089 | 22.527 | 20.7x | 1.050 | 25.660 | 3.980 | 40.395 | 10.2x |
| 10m | 2.101 | 24.509 | 11.7x | 2.068 | 28.020 | 3.980 | 42.395 | 10.7x |

The 1k and 10k files store `EventDate` as USMALLINT and both engines returned `15888,15917`. The 1m and 10m files store it as DATE and both returned `2013-07-02,2013-07-31`. The native database files were unchanged. A missing or unsupported certificate falls back to regular execution.

The first candidate was also measured against unchanged engine main (`4eb19ad4`) and DuckDB in 51 alternating three-way trials per size. Its 10m wall time fell from 2.257 to 2.040 ms, and peak RSS fell from 5.344 to 3.844 MiB. DuckDB measured 24.348 ms and 42.395 MiB in the same series. Rebase onto `0104ff97` changed the binary and raised its peak RSS to 3.980 MiB, so the table above uses fresh measurements of that final rebased binary. The 1k and 10k RSS ratios have little margin above 10x.

Before the rebase, the same candidate ran Q3, Q4, Q5, Q6, and Q8 at all four sizes, with 51 fresh-process trials per engine and exact CSV checks. Every time and RSS ratio remained above 10x. Those regression samples are included, but are not measurements of the final rebased binary. The final rebased build passed `cargo test -p rudb-cli`, the focused extrema tests in `rudb`, and Clippy with warnings denied.

The final rudb binary SHA-256 is `182e62c021f40edabda0f7f11a84bddea743088f0fba892d7b529757c8bcd646`. DuckDB v2.0.0-dev84237 (`cc7e7bac7f`) SHA-256 is `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The [runner](../../scripts/q7-fresh-process.py), [resource helper](../../scripts/measure-child.c), and [raw measurements](q7-small-process-rss/) make the comparisons reproducible.
