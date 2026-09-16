# Borrowed string minima in grouped ClickBench queries

Query 29 computes `MIN(Referer)` after grouping by the host extracted from `Referer`. Its aggregate kernel previously built an owned `Value` for every valid string row before comparing it with the group's current minimum. The 10m sample has 8,103,875 nonempty `Referer` rows. The new kernel compares borrowed bytes from flat, dictionary, string view, and run length vectors. It owns a string only when that group gets a new minimum or maximum. Other vector forms keep the existing general path.

The source gate passed on server3. The existing scattered aggregate differential test checks grouped `MIN` and `MAX` of strings against one accumulator per group, including nulls and dictionary input. The 1m and 10m audit ran all 43 queries, and every original answer or deterministic tie retest matched DuckDB. The source branch starts from main after the query 29 newline correction.

Each timing row uses five hot repetitions in fresh processes on the same Linux host. Query time is the CLI timer; child CPU and peak RSS come from Linux `wait4`. DuckDB native uses a preloaded table. DuckDB Parquet and rudb read the same Parquet file per query. The binary was copied and pinned before the audit. The first query 29 table shows separate baseline and candidate sweeps on the same host. Timing drift between sweeps is possible.

| 10m query 29 | DuckDB native | DuckDB Parquet | rudb |
| --- | ---: | ---: | ---: |
| Baseline median time (ms) | 320.0 | 369.0 | 385.6 |
| Candidate median time (ms) | 333.0 | 366.0 | 354.9 |
| Baseline child CPU (ms) | 7,946.3 | 8,094.4 | 4,168.2 |
| Candidate child CPU (ms) | 7,806.2 | 8,099.0 | 3,969.9 |
| Baseline peak RSS (MiB) | 1,163.4 | 1,657.8 | 606.5 |
| Candidate peak RSS (MiB) | 1,152.0 | 1,616.3 | 611.0 |

Rudb's query timer improved by 8 percent in the paired sweeps. Its candidate time is 3 percent faster than DuckDB Parquet's candidate time, with about half the child CPU and 38 percent of the peak RSS. The small RSS difference between rudb runs is within the variation seen across these sweeps; this change is a CPU optimization, not a claim of reduced peak memory.

The complete audit below is a separate candidate sweep. Query 29 measured 354.8 ms for rudb, 336.0 ms for DuckDB native, and 369.0 ms for DuckDB Parquet at 10m. At 1m, the corresponding times were 47.0, 83.0, and 87.0 ms.

| Size | Engine | Complete queries | Sum of median query time (s) | Sum of median child CPU (s) | Peak child RSS (MiB) |
| --- | --- | ---: | ---: | ---: | ---: |
| 1m | DuckDB native | 43 | 0.539 | 3.814 | 309.1 |
| 1m | DuckDB Parquet | 43 | 1.599 | 8.019 | 377.0 |
| 1m | rudb | 43 | 0.9356 | 5.184 | 191.9 |
| 10m | DuckDB native | 43 | 2.974 | 37.786 | 1,654.8 |
| 10m | DuckDB Parquet | 43 | 4.108 | 45.513 | 2,724.5 |
| 10m | rudb | 43 | 5.7648 | 48.876 | 1,349.6 |

The 10m suite remains around 1.4 times DuckDB Parquet's total query time and about half its peak RSS. Saving per-row string ownership makes query 29 faster, but it does not move the complete suite toward the requested ten times speed and memory target by itself. The aggregate's key probes, other aggregate calls, and Parquet decoding remain important at this size.
