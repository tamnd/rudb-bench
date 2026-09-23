# Q3 fresh-process memory after direct native output

Q3 is `SELECT SUM(AdvEngineID), COUNT(*), AVG(ResolutionWidth) FROM hits`. The native file already stores certified sums and counts, so the SQL parser and result vectors were a large share of this query's small-process memory. The candidate uses those same certified values and formats the one CSV row directly when the command is a single read-only Q3 statement with standard CSV output. Other SQL and output settings use the ordinary path.

The runner launched a new process for every query and alternated a previous rudb binary, the candidate, and DuckDB for 51 trials at each size. It checked the full CSV output against DuckDB for every run. Wall time, user plus system CPU, and peak resident memory came from `wait4` for the measured child. Cells below are medians. Both databases used native files with the same row counts, and both engines received the same SQL. The comparison binary has SHA-256 `8b9d4b97c7da981d0491dfbf0028c6bd62b8582eb3c3a475d54d1c89b20a71bd`; it uses the prior Q3 native path. The candidate was built from `9fae4249` plus this change and has SHA-256 `e0a5fe89ed49c5fef47320a625a30261d287f2af1a5304d425244ccb85bdd36b`. The engine change was subsequently rebased onto `4e6b228e`, with the focused test and Clippy rerun. DuckDB is v2.0.0-dev84237 (`cc7e7bac7f`), SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`.

| Rows | Previous rudb wall ms | Candidate rudb wall ms | DuckDB wall ms | DuckDB / candidate | Candidate CPU ms | DuckDB CPU ms | Previous rudb peak MiB | Candidate rudb peak MiB | DuckDB peak MiB | DuckDB / candidate RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 1.491 | 1.320 | 33.303 | 25.2x | 1.264 | 39.265 | 5.75 | 4.21 | 40.46 | 9.6x |
| 10k | 2.071 | 1.676 | 34.059 | 20.3x | 1.618 | 39.769 | 5.76 | 4.22 | 40.45 | 9.6x |
| 1m | 1.545 | 1.248 | 36.040 | 28.9x | 1.195 | 46.925 | 5.75 | 4.21 | 43.62 | 10.4x |
| 10m | 2.627 | 2.446 | 49.486 | 20.2x | 2.385 | 113.428 | 5.76 | 4.22 | 86.93 | 20.6x |

The run was on a 32-core shared host with load average around 19 to 23 near the end, which raised absolute times relative to the earlier Q3 report. The engines were interleaved within each trial, so the table describes this particular paired interval. Q3 exceeds 10x in wall time at all sizes and in peak RSS at 1m and 10m. The 1k and 10k peak-RSS ratios remain below 10x.

The [runner](../../scripts/q3-fresh-process-three-way.py) and [raw measurements](q3-memory/) preserve the commands and individual child measurements. The certified native files were the existing Q7 copies; the earlier Q3 native summary certificates are present in them. The lightweight path checks the table-directory checksum before using those values.
