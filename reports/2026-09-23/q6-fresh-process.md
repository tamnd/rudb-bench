# ClickBench Q6 in fresh processes

Q6 is `SELECT COUNT(DISTINCT SearchPhrase) FROM hits`. The Q5 native-catalog change also certifies exact distinct counts for string columns. `SearchPhrase` has an exact count at all four measured sizes, so Q6 uses the same small-catalog read path without another engine change. The prior rudb binary decodes the full table directory to reach the count.

Each invocation opens one native database, runs the same SQL, prints CSV, and exits. The C resource helper measures the child with `wait4` for wall time, user plus system CPU time, and peak resident memory. The runner alternates the prior rudb binary, the certified-catalog rudb binary, and DuckDB for 51 trials at each size. The two rudb files contain the same rows; the newer file has an appended catalog certificate. Every output was checked byte for byte against the expected count.

| Rows | Prior rudb wall ms | Certified rudb wall ms | DuckDB wall ms | DuckDB / rudb | Prior / rudb | Rudb CPU ms | DuckDB CPU ms | Prior peak MiB | Rudb peak MiB | DuckDB peak MiB | DuckDB / rudb RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 9.697 | 2.296 | 39.952 | 17.4x | 4.2x | 2.201 | 49.606 | 13.84 | 5.23 | 41.82 | 8.0x |
| 10k | 17.152 | 2.493 | 35.836 | 14.4x | 6.9x | 2.408 | 44.551 | 15.85 | 5.23 | 42.29 | 8.1x |
| 1m | 9.480 | 1.980 | 42.894 | 21.7x | 4.8x | 1.933 | 97.185 | 14.10 | 5.23 | 74.30 | 14.2x |
| 10m | 26.096 | 2.262 | 88.232 | 39.0x | 11.5x | 2.216 | 602.117 | 36.34 | 5.23 | 300.55 | 57.5x |

The shared host was still busy during this run, with a load average of 23.69 on 32 logical CPUs at the end. The table is a paired comparison under contention, not a quiet-host latency estimate. The measured Q6 path meets the 10x wall-time target at every size. Its peak RSS is 8.0x and 8.1x below DuckDB at 1k and 10k, and more than 10x below from 1m onward.

The exact counts by size were `138`, `1282`, `107703`, and `892232`. All 612 invocations matched. The measured prior binary is the Q4 feature build with SHA-256 `a54532de9fcd8557f12740a77e99812063cbb6f453f6f5c5dc15ab594b047f96`. The certified rudb binary has SHA-256 `1921cf0a1eb0f89235eb32ad38f3f72fd308a9b5e7f4e02c889e9dd719e9ff44`; this is the Q5 change before its final rebase onto main. DuckDB v2.0.0-dev84237 (`cc7e7bac7f`) has SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`.

The [runner](../../scripts/q6-fresh-process.py), [resource helper](../../scripts/measure-child.c), and [raw records](q6-fresh-process/) are included.
