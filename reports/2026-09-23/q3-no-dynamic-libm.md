# Q3 fresh-process memory without dynamic libm

Q3 is `SELECT SUM(AdvEngineID), COUNT(*), AVG(ResolutionWidth) FROM hits`. The native catalog answers this query without scanning rows. After the direct native result change, the remaining small-file memory gap came from a shared math library loaded at process startup. The rudb CLI had two unresolved math symbols, `log` for the storage distinct counter and `pow` for an optimizer estimate. Neither is used by Q3. Replacing those two calls with the pure Rust `libm` implementation removed `libm.so` from the CLI's dynamic dependencies. A live process map had shown about 0.4 MiB resident in that library.

The runner alternated an exact-main rudb build, the candidate built from that same commit plus the math change, and DuckDB. It launched a fresh process for each statement, with 51 trials per size, and checked every complete CSV answer against DuckDB. Both engines ran the same SQL over native files with matching row counts. The resource helper used `wait4` to collect child wall time, user plus system CPU, and peak RSS. All cells are medians.

| Rows | Main rudb wall ms | New rudb wall ms | DuckDB wall ms | DuckDB / new | New rudb CPU ms | DuckDB CPU ms | Main rudb peak MiB | New rudb peak MiB | DuckDB peak MiB | DuckDB / new RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 1.112 | 1.014 | 22.570 | 22.3x | 0.988 | 25.981 | 4.22 | 3.80 | 40.43 | 10.6x |
| 10k | 1.498 | 1.389 | 22.596 | 16.3x | 1.367 | 26.055 | 4.22 | 3.80 | 40.44 | 10.6x |
| 1m | 1.101 | 1.005 | 24.614 | 24.5x | 0.974 | 30.599 | 4.22 | 3.80 | 43.71 | 11.5x |
| 10m | 2.181 | 2.082 | 35.221 | 16.9x | 2.062 | 70.833 | 4.22 | 3.79 | 86.84 | 22.9x |

Q3 now exceeds the 10x wall-time and 10x peak-RSS targets at each tested size. These results are for a shared 32-core host whose load average was 2.27 at the end of the run. The exact-main rudb release binary was built from `8b8885a1` and has SHA-256 `400048e42193aa0507c669cccb1d6d82a8b40953dceed2812eadbb2b92c9f55a`. The candidate has SHA-256 `5a55b0aa5fc2552e036d7dc12e004e1df9ab354172ad5f6b1b5ddc52c76f615d`. The engine change was subsequently rebased onto `d35a1158`, and the relevant tests and Clippy were rerun. DuckDB is v2.0.0-dev84237 (`cc7e7bac7f`) with SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`.

The [runner](../../scripts/q3-fresh-process-three-way.py) and [raw samples](q3-no-libm/) preserve every measurement. The native inputs are the certified Q7 files used in the prior Q3 report; no table data was rewritten for this comparison.
