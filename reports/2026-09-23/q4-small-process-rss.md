# Q4 fresh-process memory and time

ClickBench Q4 is `SELECT AVG(UserID) FROM hits`. Rudb already reads the exact integer sum and non-null count from the checksummed native catalog. The regular one-statement result still built a query result vector and used the general CSV renderer. A narrow read-only CSV path now reads the same certified values and formats the answer directly. It recognizes a single unquoted `SELECT AVG(column) FROM table` statement. Other SQL uses the ordinary engine path, as do empty or all-null averages.

## Final run after rebase

The candidate was rebased onto rudb main `70f05cc1`, then rebuilt and measured. The release CLI SHA-256 is `22d000200011dd5d474d14f25e6a3429a223f4d900bda9b44122e490445d5046`. DuckDB is v2.0.0-dev84237 (`cc7e7bac7f`), SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. Each invocation used a new process. The runner alternated the engines over 51 trials per size, checked every complete CSV answer byte for byte, and measured child wall time, CPU time, and peak RSS with `wait4`. All cells are medians.

| Rows | Rudb wall ms | DuckDB wall ms | DuckDB / rudb wall | Rudb CPU ms | DuckDB CPU ms | Rudb peak MiB | DuckDB peak MiB | DuckDB / rudb RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 1.103 | 21.748 | 19.7x | 1.079 | 26.137 | 4.03 | 40.34 | 10.007x |
| 10k | 1.479 | 22.507 | 15.2x | 1.443 | 25.908 | 4.03 | 40.34 | 10.007x |
| 1m | 1.118 | 25.217 | 22.6x | 1.093 | 33.102 | 4.03 | 49.21 | 12.21x |
| 10m | 2.014 | 39.165 | 19.4x | 1.970 | 106.496 | 4.03 | 112.72 | 27.96x |

The shared 32-logical-CPU host had a load average of 4.58 at the end. The 1k and 10k memory ratios exceed 10x by a very small margin. The [final raw samples](q4-small-process-rss/) are named `q4-head-*.json`. An earlier full run on main `c9961865` is retained as `q4-rebased-*.json`.

The engine patch was subsequently rebased onto the 0.4.18 release commit `9080ce16`. The focused tests and release build passed again. That release binary has SHA-256 `f33120dff63c015f8dd02253cdf16072c84d741aa6674f0178c423329b21b8d6`. A new 51-pair 1k run, stored in `q4-018-1k.json`, measured 2.345 ms and 4.031 MiB for rudb against DuckDB's 42.798 ms and 40.719 MiB. The memory ratio was 10.10x. This interval had higher CPU times for both engines than the full run above.

## Same-base change comparison

Before the rebase, the unchanged rudb main was `43239e57`, SHA-256 `1e25ede046dc9a21609fefc4ef2587b6f46a59552bd547a1462537c43d7fff98`. The candidate on that base was SHA-256 `0f2033c3f6212466493b7aecce0fffdf3c04c70342f34e01f1be50e7f4bd7806`. These three-way trials alternated unchanged main, the candidate, and DuckDB 51 times per size against the same native files.

| Rows | Main rudb wall ms | New rudb wall ms | DuckDB wall ms | Main rudb peak MiB | New rudb peak MiB | DuckDB peak MiB |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 1.227 | 1.076 | 21.970 | 5.39 | 3.96 | 40.43 |
| 10k | 1.591 | 1.414 | 22.451 | 5.39 | 3.96 | 40.44 |
| 1m | 1.236 | 1.071 | 24.445 | 5.39 | 3.96 | 49.46 |
| 10m | 2.166 | 1.985 | 39.172 | 5.39 | 3.96 | 112.46 |

The direct value path removes about 1.43 MiB of peak process memory on that base. Formatting a large DOUBLE through the tagged `Value` still touched extra pages; the final formatter derives DuckDB's decimal or exponent spelling from the shortest ordinary double representation. A unit test compares that formatter with the existing SQL value printer for 100,000 generated bit patterns, and the four benchmark answers matched DuckDB in every run. The relevant average and null tests, Clippy with warnings denied, and release build passed after the rebase.

The [runner](../../scripts/q4-fresh-process.py), [resource helper](../../scripts/measure-child.c), and [raw samples](q4-small-process-rss/) preserve the commands and observations. The same-base three-way samples are named `q4-paired-final-*.json`. All tests used native rudb and native DuckDB files, with row counts of 1,000, 10,000, 999,975, and 9,999,750 respectively.
