# reports

Runs that were taken, kept as they came out. Every file here was written by `rudb-bench run --report` and nothing in any of them was typed by hand, which is the point: a report somebody edited is a report nobody can check.

Runs are added and never replaced. A number that was true on the day it was measured stays in the directory even after a faster one exists, because a history that overwrites itself is a table rather than a history, and the interesting question is usually what changed between two of these rather than what the latest one says.

| file | suite | machine | engines | size | taken |
| --- | --- | --- | --- | --- | --- |
| [run-clickbench-vmi3391933-100k.md](run-clickbench-vmi3391933-100k.md) | ClickBench | vmi3391933 | 7 | 99998 rows, one in every 1000 | 12 September 2026 |
| [run-clickbench-vmi3391933-10k.md](run-clickbench-vmi3391933-10k.md) | ClickBench | vmi3391933 | 7 | 10000 rows, one in every 10000 | 12 September 2026 |
| [run-clickbench-vmi3391933-1k.md](run-clickbench-vmi3391933-1k.md) | ClickBench | vmi3391933 | 7 | 1000 rows, one in every 99998 | 12 September 2026 |
| [run-clickbench-server3.md](run-clickbench-server3.md) | ClickBench | vmi3391933 | 2 | 999975 rows, one in every 100 | 11 September 2026 |
| [m1-the-format-experiment.md](m1-the-format-experiment.md) | storage formats | several | n/a | n/a | September 2026 |

The three sizes are one measurement rather than three, and they are meant to be read together. A single size cannot tell a fixed cost apart from a per row cost, and which of the two an engine is paying is usually the only thing worth knowing about it. See the top of the [README](../README.md) for what this particular ladder says.

`run-clickbench-server3.md` is the odd one out and is kept because runs are kept. It predates rudb running the suite at all, so it is DuckDB against DataFusion and nothing else, and its file name says `server3` where the later ones say the host name the machine actually answers to.
