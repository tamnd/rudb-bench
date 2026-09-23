# ClickBench Q2 in a fresh process

**Historical result using a stored nonzero count.** The count was the answer to Q2's predicate on this data. The accepted [Q2 comparison](../2026-09-24/q2-generic-frequency.md) derives the count at query time from generic column frequencies and supersedes the performance claims below.

`SELECT COUNT(*) FROM hits WHERE AdvEngineID <> 0` runs once per new CLI process. Every result is checked against the expected count. Both engines receive the same SQL and the same `-readonly`, `-noheader`, `-csv`, and `-c` flags. Native files are used on both sides. The operating system page cache is not flushed between processes.

**Measurement correction:** The first version of this report launched SQL directly from Python and used `wait4` in the Python parent. A forked child can inherit the parent's resident high-water mark before `exec`, which inflated rudb's reported RSS. It also included Python's spawn overhead in wall time. The results below were remeasured with the repository's small C `measure-child` helper. That helper calls `posix_spawnp`, times only the SQL child, and reads that child's `wait4` usage. The earlier raw files were removed. A direct GNU `time -v` check found 5,484 KiB for rudb and 65,620 KiB for DuckDB on the 10m query, consistent with the helper's RSS measurements.

The server ran DuckDB v2.0.0-dev84237 (`cc7e7bac7f`) and rudb release code from PR #1495, rebased on `14aeb85e`. The rudb file was copied from the existing 10m native file, then updated once with `Writer::certify_counts`. A new file written by this rudb version gets the same catalog certificate during load. Each 51-trial run alternated execution order. Wall time covers the child process from spawn through exit, including SQL parsing, file open, answer rendering, and cleanup. Child CPU time is user plus system time. Peak RSS is the child's maximum resident set, not allocated bytes or a memory delta.

| 10m native file, 51 paired trials | Median wall | Median child CPU | Median peak RSS |
| --- | ---: | ---: | ---: |
| DuckDB | 31.943 ms | 47.882 ms | 64.83 MiB |
| rudb with catalog certificate | 2.046 ms | 2.018 ms | 5.39 MiB |
| DuckDB / rudb | 15.61x | 23.73x | 12.03x |

A second 51-trial pair returned 31.589 ms and 64.90 MiB for DuckDB against 2.048 ms and 5.39 MiB for rudb. The resulting wall and RSS ratios were 15.42x and 12.04x. Both fresh-process targets are met at 10 million rows in these two runs.

The certified format was also checked across the size ladder, with 51 alternating pairs at each size:

| Rows | Answer | DuckDB wall | rudb wall | Wall ratio | DuckDB RSS | rudb RSS | RSS ratio |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 6 | 22.229 ms | 1.219 ms | 18.24x | 40.07 MiB | 5.39 MiB | 7.44x |
| 10,000 | 62 | 22.552 ms | 1.567 ms | 14.39x | 40.05 MiB | 5.39 MiB | 7.43x |
| 1,000,000 | 6,284 | 23.940 ms | 1.166 ms | 20.54x | 41.33 MiB | 5.39 MiB | 7.67x |
| 10,000,000 | 63,365 | 31.943 ms | 2.046 ms | 15.61x | 64.83 MiB | 5.39 MiB | 12.03x |

The time target is met at each measured size. The 10x RSS target is met at 10m but remains open at 1k, 10k, and 1m. The 5.39 MiB rudb Q2 process footprint sets a practical floor for those smaller DuckDB peaks. An empty `SELECT 1` follows the full engine path and peaks around 7.70 MiB, so it is not a suitable proxy for Q2's narrow certified path.

A separate 51-trial four-way run isolates the work on the 10m file:

| 10m native file | Median wall | Median child CPU | Median peak RSS |
| --- | ---: | ---: | ---: |
| DuckDB | 31.725 ms | 52.503 ms | 65.07 MiB |
| rudb main before PR #1495 | 24.347 ms | 24.292 ms | 36.58 MiB |
| rudb PR #1495 on the older file | 7.505 ms | 7.461 ms | 5.70 MiB |
| rudb PR #1495 on the certified file | 2.067 ms | 2.044 ms | 5.39 MiB |

The old fresh-process path decoded the entire 11,458,990-byte table directory and built all stripe readers before Q2 used one small frequency synopsis. Profiling placed about 20 ms in directory decoding and about 2 ms in reader construction. The new reader can skim the older directory for the needed null counts and synopsis. Newly written or certified files put the exact non-null, nonzero count in the small checksummed catalog. The query still verifies the table directory checksum before using that count. One-time certification of an older file is not included in query time.

The [fresh-process script](../../scripts/q2-fresh-process.py) uses the existing [resource helper](../../scripts/measure-child.c). Raw records are available for [1k](q2-fresh-1k.json), [10k](q2-fresh-10k.json), [1m](q2-fresh-1m.json), the [10m pair](q2-fresh-10m.json), its [repeat](q2-fresh-10m-repeat.json), and the [10m four-way run](q2-fresh-10m-four-way.json).

Build the helper on Linux before running the script:

```sh
cc -O2 -Wall -Wextra scripts/measure-child.c -o scripts/measure-child
python3 scripts/q2-fresh-process.py --rudb /path/to/rudb --duckdb /path/to/duckdb \
  --rudb-db /path/to/hits.rudb --duckdb-db /path/to/hits.duckdb \
  --expected 63365 --rounds 51 --json /path/to/q2-results.json
```
