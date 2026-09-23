# ClickBench Q4 in fresh processes

Q4 is `SELECT AVG(UserID) FROM hits`. The same SQL runs against native rudb and native DuckDB files. Each invocation opens one database, executes one statement, prints CSV, and exits. The C resource helper measures that child with `wait4`, including wall time, user plus system CPU time, and peak resident memory. The runner alternates engine order over 51 trials per size. Every result must match the expected CSV row byte for byte.

Rudb main already has an exact sum and non-null count for signed integer columns in the small checksummed native catalog. The Q4 change reads those values directly after verifying the table directory checksum. Main instead opens and decodes the full table directory before the regular aggregate can use its stripe summaries. All three engines below ran against the same certified native rudb file or the matching DuckDB file.

| Rows | Rudb main wall ms | Rudb Q4 wall ms | DuckDB wall ms | DuckDB / Q4 | Main / Q4 | Rudb Q4 CPU ms | DuckDB CPU ms | Rudb main peak MiB | Rudb Q4 peak MiB | DuckDB peak MiB |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 19.643 | 2.323 | 68.780 | 29.6x | 8.5x | 1.608 | 46.644 | 14.63 | 5.78 | 40.27 |
| 10k | 28.033 | 2.322 | 64.855 | 27.9x | 12.1x | 1.894 | 48.079 | 16.73 | 5.79 | 40.39 |
| 1m | 25.027 | 2.453 | 86.668 | 35.3x | 10.2x | 1.656 | 59.038 | 14.73 | 5.78 | 49.50 |
| 10m | 53.267 | 3.693 | 143.605 | 38.9x | 14.4x | 2.464 | 157.360 | 37.98 | 5.78 | 113.93 |

The host was under heavy competing CPU load during the three-way run: the load average exceeded 50 on 32 CPUs. The numbers above compare binaries within the same alternating run, but the wall-time ratios may differ on a quiet host. An earlier 51-pair baseline, taken before that load increase, measured rudb main and DuckDB at 7.420 / 24.172 ms for 1k, 14.126 / 24.094 ms for 10k, 7.811 / 28.123 ms for 1m, and 29.224 / 44.767 ms for 10m. The heavy-load run is therefore evidence for the new path's resource savings, not a representative quiet-host latency claim. A quiet-host rerun remains useful.

The exact expected rows are `2.414420660257356e+18`, `2.534231104689841e+18`, `2.528885963832509e+18`, and `2.5288637482435594e+18` in size order. The new path matched DuckDB and rudb main in all 612 three-way invocations. Rudb Q4 uses 7.0x, 7.0x, 8.6x, and 19.7x less peak RSS than DuckDB by size.

The release run used rudb main `c1b88e92` and DuckDB v2.0.0-dev84237 (`cc7e7bac7f`) on `gpc`. The Q4 change was subsequently rebased onto `60c4419d`, which adds list functions, and its cold-path tests were rerun on that base. The SHA-256 hashes of the measured binaries are `d1e593c352042b7401b6d9e5f3f5bfd12e3a7ed884d9ecbeb08412682765b4e5` for rudb main, `a54532de9fcd8557f12740a77e99812063cbb6f453f6f5c5dc15ab594b047f96` for the Q4 change, and `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531` for DuckDB. The [runner](../../scripts/q4-fresh-process.py), [resource helper](../../scripts/measure-child.c), and [raw records](q4-fresh-process/) are included. `*-three-way.json` holds the paired main, Q4, and DuckDB trials; `*-baseline.json` holds the earlier main and DuckDB trials.
