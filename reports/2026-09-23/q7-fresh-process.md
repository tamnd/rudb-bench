# ClickBench Q7 in fresh processes

Q7 is `SELECT MIN(EventDate), MAX(EventDate) FROM hits`. The native table directory already has exact bounds for this column. A fresh process used to decode the full directory before it could answer. The Q7 change copies exact integer and date bounds into the small checksummed catalog. It verifies the referenced directory checksum before using the certificate. A column without exact bounds uses regular execution; an empty or all-NULL column has a separate certified NULL result.

Each invocation opens one native database, runs the same SQL, prints CSV, and exits. The C resource helper measures that child with `wait4` for wall time, user plus system CPU time, and peak resident memory. The runner alternates prior rudb, certified rudb, and DuckDB for 51 trials per size. The two rudb files contain the same rows; the newer one has the appended catalog certificate. Every result matched the expected CSV row byte for byte.

| Rows | Prior rudb wall ms | Rudb Q7 wall ms | DuckDB wall ms | DuckDB / Q7 | Prior / Q7 | Rudb Q7 CPU ms | DuckDB CPU ms | Prior peak MiB | Rudb Q7 peak MiB | DuckDB peak MiB | DuckDB / Q7 RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 7.447 | 1.370 | 22.971 | 16.8x | 5.4x | 1.343 | 26.754 | 13.77 | 5.31 | 40.09 | 7.6x |
| 10k | 14.167 | 1.785 | 23.194 | 13.0x | 7.9x | 1.750 | 27.134 | 16.02 | 5.32 | 40.19 | 7.6x |
| 1m | 7.808 | 1.334 | 23.610 | 17.7x | 5.9x | 1.309 | 27.601 | 14.01 | 5.32 | 40.71 | 7.6x |
| 10m | 24.856 | 2.443 | 25.533 | 10.5x | 10.2x | 2.415 | 29.426 | 36.27 | 5.33 | 42.46 | 8.0x |

A second independent 51-trial 10m run measured 2.440 ms for rudb and 25.583 ms for DuckDB, again 10.5x. The host load average fell from 13.40 to 6.50 on 32 logical CPUs across these runs. The 10m time margin is narrow but repeated. Peak RSS is 7.6x to 8.0x below DuckDB, short of the 10x memory goal.

The 1k and 10k native files store `EventDate` as USMALLINT and returned `15888,15917`. The 1m and 10m files store it as DATE and returned `2013-07-02,2013-07-31`. All 612 three-way invocations matched DuckDB. A new rudb binary also answered the older 10m file without an extrema certificate through normal execution.

The prior measured binary is the Q4 feature build, SHA-256 `a54532de9fcd8557f12740a77e99812063cbb6f453f6f5c5dc15ab594b047f96`. The measured Q7 release binary has SHA-256 `44a5c148ac599810963a0ad01208b7463d76e761bfc65a53ef1daf1f1410a718`. DuckDB v2.0.0-dev84237 (`cc7e7bac7f`) has SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The [runner](../../scripts/q7-fresh-process.py), [resource helper](../../scripts/measure-child.c), and [raw samples](q7-fresh-process/) are included. `*-baseline.json` holds an earlier two-way measurement; `10m-repeat.json` holds the independent repeat.
