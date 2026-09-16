# Reports

Reports are grouped by the UTC date when the measurement was taken or the analysis was committed. Generated runs and written investigations share the same date directory. Supporting JSON files live in a named subdirectory beside their Markdown report.

| Date | Reports |
| --- | --- |
| 2026-09-16 | [native storage v3](2026-09-16/native-storage-v3.md), [native streaming load](2026-09-16/native-streaming-load.md), [native checksums](2026-09-16/native-fast-checksum.md), [native late materialization](2026-09-16/native-late-materialization.md), [native zone maps](2026-09-16/native-zone-maps-and-dependent-groups.md), [global string codes and radix count](2026-09-16/global-string-codes-and-radix-count.md), [shared aggregate string output](2026-09-16/shared-aggregate-string-output.md), [compact numeric state](2026-09-16/sparse-compact-numeric-state.md), [borrowed string minima](2026-09-16/borrowed-string-minima.md), [Q19 and Q33 completion](2026-09-16/q19-q33-completion.md), [Q29 newline host](2026-09-16/q29-host-newline.md) |
| 2026-09-15 | [row-group layout](2026-09-15/clickbench-row-group-layout.md), [zone maps and the view](2026-09-15/zone-maps-and-the-view.md), [q23 memory scaling](2026-09-15/q23-memory-scaling.md), [q23 staged scan](2026-09-15/q23-staged-scan-experiment.md), [radix table follow-up](2026-09-15/radix-table-follow-up.md) |
| 2026-09-14 | [root cause](2026-09-14/rudb-duckdb-root-cause-20260914.md), [parallel audit](2026-09-14/clickbench-parallel-audit-20260914.md), [radix and fetch follow-up](2026-09-14/clickbench-radix-and-fetch-20260914.md) |
| 2026-09-13 | [measurement audit](2026-09-13/clickbench-audit.md), [per-query details](2026-09-13/clickbench-audit-details.md) |
| 2026-09-12 | ClickBench and smoke harness runs |
| 2026-09-11 | [storage format experiment](2026-09-11/m1-the-format-experiment.md) |

Generated runs are kept as they came out. Every run file was written by `rudb-bench run --report` and nothing in it was typed by hand, which is the point: a report somebody edited is a report nobody can check.

Runs are added and never replaced. A number that was true on the day it was measured stays in the directory even after a faster one exists, because a history that overwrites itself is a table rather than a history, and the interesting question is usually what changed between two of these rather than what the latest one says.

| file | suite | machine | engines | size | taken |
| --- | --- | --- | --- | --- | --- |
| [run-clickbench-gamingpc-wsl-1m-quiet.md](2026-09-12/run-clickbench-gamingpc-wsl-1m-quiet.md) | ClickBench | gamingpc-wsl | 7 | 999975 rows, one in every 100 | 12 September 2026 |
| [run-clickbench-gamingpc-wsl-100k-quiet.md](2026-09-12/run-clickbench-gamingpc-wsl-100k-quiet.md) | ClickBench | gamingpc-wsl | 7 | 99998 rows, one in every 1000 | 12 September 2026 |
| [run-clickbench-gamingpc-wsl-10k-quiet.md](2026-09-12/run-clickbench-gamingpc-wsl-10k-quiet.md) | ClickBench | gamingpc-wsl | 7 | 10000 rows, one in every 10000 | 12 September 2026 |
| [run-clickbench-gamingpc-wsl-1k-quiet.md](2026-09-12/run-clickbench-gamingpc-wsl-1k-quiet.md) | ClickBench | gamingpc-wsl | 7 | 1000 rows, one in every 99998 | 12 September 2026 |
| [run-clickbench-gamingpc-wsl-1m.md](2026-09-12/run-clickbench-gamingpc-wsl-1m.md) | ClickBench | gamingpc-wsl | 7 | 999975 rows, one in every 100 | 12 September 2026 |
| [run-clickbench-gamingpc-wsl-100k.md](2026-09-12/run-clickbench-gamingpc-wsl-100k.md) | ClickBench | gamingpc-wsl | 7 | 99998 rows, one in every 1000 | 12 September 2026 |
| [run-clickbench-gamingpc-wsl-10k.md](2026-09-12/run-clickbench-gamingpc-wsl-10k.md) | ClickBench | gamingpc-wsl | 7 | 10000 rows, one in every 10000 | 12 September 2026 |
| [run-clickbench-gamingpc-wsl-1k.md](2026-09-12/run-clickbench-gamingpc-wsl-1k.md) | ClickBench | gamingpc-wsl | 7 | 1000 rows, one in every 99998 | 12 September 2026 |
| [run-clickbench-vmi3391933-100k.md](2026-09-12/run-clickbench-vmi3391933-100k.md) | ClickBench | vmi3391933 | 7 | 99998 rows, one in every 1000 | 12 September 2026 |
| [run-clickbench-vmi3391933-10k.md](2026-09-12/run-clickbench-vmi3391933-10k.md) | ClickBench | vmi3391933 | 7 | 10000 rows, one in every 10000 | 12 September 2026 |
| [run-clickbench-vmi3391933-1k.md](2026-09-12/run-clickbench-vmi3391933-1k.md) | ClickBench | vmi3391933 | 7 | 1000 rows, one in every 99998 | 12 September 2026 |
| [run-clickbench-server3.md](2026-09-12/run-clickbench-server3.md) | ClickBench | vmi3391933 | 2 | 999975 rows, one in every 100 | 11 September 2026 |
| [m1-the-format-experiment.md](2026-09-11/m1-the-format-experiment.md) | storage formats | several | n/a | n/a | September 2026 |

A ladder is one measurement rather than three or four, and the sizes are meant to be read together. A single size cannot tell a fixed cost apart from a per row cost, and which of the two an engine is paying is usually the only thing worth knowing about it. See the top of the [README](../README.md) for what these ladders say.

There are ladders from two machines here, and rule seven says a number from one machine is never compared against a number from another. `gamingpc-wsl` has 32 hardware threads and reaches a million rows. `vmi3391933` has 8 and stops at a hundred thousand. Reading a row off one and a row off the other is the mistake the rule exists to prevent.

The four files with `quiet` in the name are a second sweep of the same ladder on `gamingpc-wsl`, taken a few hours after the first. Every engine in them started with the one minute load average under four on a machine with 32 threads, which each report records per engine. The first sweep ran while another agent was building DuckDB from source on the same box, and reading the two against each other is the interesting thing in this directory right now: contention cost DuckDB roughly three times its query time at a million rows and cost rudb less than twice, because DuckDB was trying to use three and a half cores and rudb was using one. The busy sweep is kept rather than replaced, both because runs are always kept and because it is the evidence for that.

`run-clickbench-server3.md` is the odd one out and is kept because runs are kept. It predates rudb running the suite at all, so it is DuckDB against DataFusion and nothing else, and its file name says `server3` where the later ones say the host name the machine actually answers to.
