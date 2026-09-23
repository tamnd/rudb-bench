# ClickBench Q5 in fresh processes

Q5 is `SELECT COUNT(DISTINCT UserID) FROM hits`. Each invocation opens one native database, runs this SQL, prints CSV, and exits. The C resource helper measures the child process with `wait4` for wall time, user plus system CPU time, and peak resident memory. The runner alternates three engines over 51 trials per size and checks every CSV row against the exact expected value.

The native writer already records an exact non-null distinct count for `UserID` in the table directory. The prior rudb binary decodes that large directory to get it. The Q5 change copies exact counts into the small checksummed catalog at commit. Existing files can append a certified catalog slot without rewriting table pages. A query verifies the table directory checksum before using the count. If a count is absent, it uses regular execution. The two rudb files in this comparison hold the same rows; the Q5 file has the appended catalog certificate, while the prior binary uses the prior file it can read.

| Rows | Prior rudb wall ms | Rudb Q5 wall ms | DuckDB wall ms | DuckDB / Q5 | Prior / Q5 | Rudb Q5 CPU ms | DuckDB CPU ms | Prior rudb peak MiB | Rudb Q5 peak MiB | DuckDB peak MiB | DuckDB / Q5 RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 11.007 | 1.439 | 36.306 | 25.2x | 7.7x | 1.391 | 40.305 | 13.97 | 5.23 | 43.24 | 8.3x |
| 10k | 21.802 | 1.793 | 41.345 | 23.1x | 12.2x | 1.755 | 42.854 | 15.98 | 5.23 | 43.88 | 8.4x |
| 1m | 14.095 | 2.115 | 77.226 | 36.5x | 6.7x | 1.793 | 130.653 | 14.21 | 5.24 | 90.11 | 17.2x |
| 10m | 36.563 | 2.651 | 196.241 | 74.0x | 13.8x | 2.474 | 882.578 | 36.46 | 5.23 | 357.77 | 68.4x |

The shared host was busy during these trials. Its load average was 34 at the end of the run on 32 logical CPUs, and the earlier two-way baseline has different wall times. These are paired comparisons under contention, not quiet-host latency estimates. The Q5 path meets the 10x wall-time target against DuckDB in the measured run. Its peak RSS is at least 10x lower from 1m rows upward, while the 1k and 10k ratios are 8.3x and 8.4x.

Expected results by size were `1000`, `9990`, `898913`, and `5606406`. All 612 three-way invocations matched. A new rudb binary also returned `5606406` from the prior 10m file, which has no distinct certificate, through normal execution.

The prior binary is the Q4 feature build measured before its final rebase onto main. The Q5 binary was built from the Q5 change on `4ccffcf3`, then rebased onto `5f5a208c` for merge and retested. The measured prior rudb binary has SHA-256 `a54532de9fcd8557f12740a77e99812063cbb6f453f6f5c5dc15ab594b047f96`, the Q5 binary has `1921cf0a1eb0f89235eb32ad38f3f72fd308a9b5e7f4e02c889e9dd719e9ff44`, and DuckDB v2.0.0-dev84237 (`cc7e7bac7f`) has `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The [runner](../../scripts/q5-fresh-process.py), [resource helper](../../scripts/measure-child.c), and [raw records](q5-fresh-process/) are included. The `*-baseline.json` files contain the earlier two-way prior rudb and DuckDB runs.
