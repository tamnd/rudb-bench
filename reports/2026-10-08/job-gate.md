# JOB, the J7 gate

Step J7 of the JOB plan (tamnd/rudb#1851), the run the JOB series is for. On server2, on one load, every one of the eight conditions of `15-the-gate.md` section 15.1 holds at once. The hot total is 5,780 ms against DuckDB's 80,336, which is 13.9 times faster, and the highest per-query ratio is 0.284, on 13d.

## Setup

- Machine: server2, an AMD EPYC virtual machine with 6 cores at 2.0 GHz. The machine is shared, and the load average was between 3 and 19 during the runs.
- DuckDB: v2.0.0-dev84237 (`cc7e7bac7f`), SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`, on its own native format.
- rudb: main at `165c437c` (tamnd/rudb#2857, after the 0.8.42 release), built in release mode, SHA-256 `71a0bb1537240dde204852df7f5930756c120ba40ecafc7df06b2b0d75dc0a98`.
- Data: the May 2013 IMDb snapshot of the JOB paper, loaded fresh into both engines from the same CSV files with JOB's `schema.sql` and no indexes, which is the `plain` configuration.
- Protocol: `scripts/job-gate.py` as committed. `load plain` loads both engines and times the loads, then `run plain` runs every query in its own process per engine after dropping the page cache: one cold run, then five hot runs, of which the median is the hot time. Each engine is given six threads in the gated run.

The per-query rows are in [job-gate/](job-gate/), with the run logs next to them.

## The eight conditions

| | Condition | Result |
| --- | --- | --- |
| 1 | All 113 answers equal DuckDB's | pass |
| 2 | Hot total at most a tenth of DuckDB's | pass, 0.072 |
| 3 | Every query's hot median at most a third of DuckDB's | pass, highest 0.284 (13d) |
| 4 | No query slower than DuckDB | pass, rudb is faster on all 113 |
| 5 | Plans and counters the same across all six runs | pass |
| 6 | No file grew or appeared while the queries ran | pass |
| 7 | No `Default` provenance, every class `Dense`, the string properties present | pass, 113 reductions all dense, 21 filter columns on codes |
| 8 | The load no slower than DuckDB's | pass, 57.5 s against 97.2 s |

The line the gate prints, with the maximum per-query ratio on the same line as the total:

```
total plain threads=6: duckdb 80336 ms, rudb 5780 ms, 13.90x, max per query ratio 0.284 (13d)
```

## Time

| | DuckDB | rudb | DuckDB over rudb |
| --- | ---: | ---: | ---: |
| hot total, 6 threads | 80,336 ms | 5,780 ms | 13.90x |
| geometric mean, 6 threads | | | 13.68x |
| cold total, 6 threads | 146,352 ms | 136,553 ms | 1.07x |
| hot total, 1 thread | 99,057 ms | 6,294 ms | 15.74x |
| geometric mean, 1 thread | | | 14.63x |
| cold total, 1 thread | 261,670 ms | 154,441 ms | 1.69x |
| load | 97.2 s | 57.5 s | 1.69x |

The highest ratios at six threads are 13d at 0.284, 8a at 0.219, 27c at 0.212, 15a at 0.211 and 13b at 0.193. The lowest are 5b at 0.014, 12b at 0.017 and 17c at 0.019. At one thread every condition holds as well, and the highest ratio is 13d at 0.189.

DuckDB's hot total at six threads is only 1.23 times its total at one. The machine is shared, and at the J0 baseline a month ago, at a lower load, the same DuckDB scaled 4.4 times. The DuckDB numbers here are what it did on this machine on this day, and the one thread row is there so the claim does not rest on DuckDB having been starved of cores. rudb is 15.7 times faster there too.

The cold row is the honest weak spot. After the page cache is dropped, rudb is only 1.07 times faster in total at six threads and wins 49 of the 113 cold runs. A cold run is mostly reading the file back from disk, and rudb reads more of it on the way to the first answer than its hot runs touch. Cold time is not one of the eight conditions, and it is the first thing to look at after J7.

## The rule turned off

The `norule` configuration runs the same queries on the same load with `plan.consistent` turned off, so every query runs as an ordinary join plan. It is printed here to show how much of the result is the consistent reduction, and it is not meant to pass.

| | DuckDB | rudb, no rule | DuckDB over rudb |
| --- | ---: | ---: | ---: |
| hot total, 6 threads | 64,367 ms | 18,481 ms | 3.48x |
| geometric mean, 6 threads | | | 5.87x |

Six queries are slower than DuckDB without the rule: 13d at 1.84, 10c at 1.84, 8d at 1.56, 16b at 1.51, 7c at 1.38 and 9d at 1.32. With the rule every one of them is under 0.3. The plans without the rule carry `Default` provenance, so condition 7 fails there by design.

## Not in this report

Two of the runs that 15.1 asks to be printed alongside are not here. The `indexed` and `clustered` configurations each need a load of their own, and server2's disk was at 96 percent with other tenants' builds moving it by several gigabytes, so a third and fourth load were not attempted. The run with every graph and statistics section deleted needs a tool that strips those sections from a file, and that tool does not exist yet. Both are carried to J8 (tamnd/rudb#1852).
