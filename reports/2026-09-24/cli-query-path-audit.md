# One SQL path for the CLI

RuDB's command line had several early answer routes for one read-only statement. They recognized SQL text and opened native statistics or a projection directly, before constructing a `Database` and running the parser, binder, optimizer, and executor. The statistics are reusable column facts, and the Q9 projection retains source rows, but a CLI-only SQL matcher makes a benchmark depend on command flags. `--set threads=6` was enough to send the same Q9 SQL through a different path and change its measured cost. [RuDB #1792](https://github.com/tamnd/rudb/pull/1792) removed the Q9 route. The follow-up engine change removes the remaining early CLI routes so a one-statement CSV command uses the same SQL path as any other command.

The engine can still use column statistics when its parsed and bound plan permits it. This keeps Q2 and Q3 efficient without a CLI exception. The native file continues to store column statistics and source rows, not SQL answers.

The measurements below use the pre-cleanup release binary with `--set threads=6` to force the ordinary SQL path. The new CLI behavior is equivalent because the early return has been removed. DuckDB reported six default threads on the same host. The SQL was identical between engines. Each value is the median of 11 alternating fresh-process trials on a busy shared six-CPU host. `wait4` measured whole-process wall time, user plus system CPU time, and peak RSS; the complete output matched DuckDB on every run. The page cache was warm. The 10k and 10m native files are the same Parquet-derived files as the [Q9 audit](native-frequency-spans.md).

| Query | Rows | RuDB engine wall | DuckDB wall | DuckDB / RuDB wall | RuDB CPU | DuckDB CPU | RuDB RSS | DuckDB RSS |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Q2 | 10k | 58.792 ms | 418.201 ms | 7.11x | 11.588 ms | 84.437 ms | 11.00 MiB | 33.24 MiB |
| Q2 | 10m | 149.481 ms | 586.372 ms | 3.92x | 36.559 ms | 157.775 ms | 16.50 MiB | 55.78 MiB |
| Q3 | 10k | 65.102 ms | 355.379 ms | 5.46x | 11.295 ms | 89.061 ms | 9.75 MiB | 33.83 MiB |
| Q3 | 10m | 109.077 ms | 830.284 ms | 7.61x | 18.732 ms | 244.599 ms | 12.25 MiB | 77.02 MiB |

The gaps between wall and CPU time are large because the host was busy. These results do not establish quiet-host speed ratios. They do show that both scalar queries remain fast on 10m rows through the engine's statistics path. Neither query meets both 10x targets in these runs. [The corrected Q9 report](native-frequency-spans.md) gives the grouped-distinct regular-engine measurements: at 10m, 328.837 ms and 73.12 MiB for RuDB against 807.515 ms and 135.50 MiB for DuckDB.

The [raw Q2 and Q3 records](cli-query-path/) keep each trial. The [Q2 runner](../../scripts/q2-fresh-process.py) and [Q3 runner](../../scripts/q3-fresh-process.py) now accept `--rudb-set threads=6` to reproduce this pre-cleanup comparison. The one-shot CLI numbers in older Q2 and Q3 reports measured a different route and should not be treated as ordinary engine latency. The native load process, source rows, and SQL did not change in this audit.
