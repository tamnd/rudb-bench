# JOB, the J0 baseline

Step J0 of the JOB plan (tamnd/rudb#1844). This is the denominator every later JOB ratio is taken against: the pinned DuckDB and rudb 0.4.32 on the same load on server2, every answer checked, at six threads and at one. It also holds the measurements of the data that the plan depends on.

## Setup

- Machine: server2, an AMD EPYC virtual machine with 6 cores at 2.0 GHz, 512 KB of L2 per core and 8 MB of L3. The machine is shared, and the load average was between 6 and 25 during the runs.
- DuckDB: v2.0.0-dev84237 (`cc7e7bac7f`), SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`.
- rudb: 0.4.32, built on server2 in release mode, SHA-256 `e149a2d11902d0144660901097148f4e7f2b155d437932442864fea7dbde4863`.
- Data: the May 2013 IMDb snapshot of the JOB paper, 21 tables, 74,190,187 rows, loaded with JOB's `schema.sql` and no indexes. The archive was deleted after the load to free disk, so its hash is not recorded here. `rudb-bench generate job` records it from now on.
- Protocol: `scripts/job-run.py`, as in the J1 report. For each query, drop the page cache, start one process, run the query six times. The first run is cold and the median of the other five is hot. The answer is checked on every run.

The DuckDB rows are those of [the J1 report](../2026-09-24/job-reduced-prototype/). The rudb rows are in [job-baseline/](job-baseline/).

## Answers

rudb returns DuckDB's answer on all 113 queries, byte for byte, at six threads and at one, and every query gives the same answer on all six runs. That includes the five queries that are empty on this snapshot, 2c, 5a, 5b, 10b and 32a.

## Time

| | DuckDB | rudb 0.4.32 | DuckDB over rudb |
| --- | ---: | ---: | ---: |
| hot total, 6 threads | 34,657 ms | 71,527 ms | 0.48x |
| cold total, 6 threads | 82,916 ms | 165,972 ms | 0.50x |
| geometric mean, 6 threads | | | 0.63x |
| queries rudb wins, 6 threads | | 22 of 113 | |
| hot total, 1 thread | 153,098 ms | 102,223 ms | 1.50x |
| geometric mean, 1 thread | | | 1.40x |
| queries rudb wins, 1 thread | | 77 of 113 | |

At six threads the worst for rudb are 29c (2389 ms against 365), 16a (916 against 154), then 24a, 11d, 17e, 16d, 22a, 7c, 16c, 8d, 23b and 19c, all at about a fifth of DuckDB. The best are 5b at 14x, 12b at 6x and 15d at 4.5x.

At one thread the worst are 21b (404 ms against 94), 15a, 15b, 11d, 7c, 3c and 17e. The best are 12b at 17.7x, then 33b, 32a, 32b and 28b at about 9x.

The two rows together say where rudb stands. Per core it is already ahead of DuckDB on JOB. It loses at six threads because it barely scales: DuckDB's total improves 4.4x from one thread to six and rudb's 1.43x. The families that lose most at six threads, 16, 17, 19, 22, 24 and 29, are the ones with the longest scans of `cast_info` and `movie_info`.

## Where DuckDB's time goes

Each query was profiled once at six threads after a warm run. The per-query split is in [job-baseline/duckdb-profile.tsv](job-baseline/duckdb-profile.tsv). Summed over the suite, in CPU time:

| operator | CPU, s | share |
| --- | ---: | ---: |
| table scan, with filters and dynamic filters | 62.8 | 63.7% |
| hash join | 33.9 | 34.3% |
| filter | 1.6 | 1.6% |
| aggregate | 0.3 | 0.3% |

DuckDB scanned 2.70 billion rows over the suite, out of 4.19 billion that the queries name, and its joins produced 158 million rows. Its plans carry 502 dynamic filters of the form `movie_id IN PRF(movie_id)`, so this build already pushes join keys from a build side into the probe side's scan. A scanned row costs it about 23 ns of CPU, string filters included.

## The data

[job-baseline/stats.tsv](job-baseline/stats.tsv) is the output of [stats.sql](job-baseline/stats.sql), and [zone.tsv](job-baseline/zone.tsv) of [zone.sql](job-baseline/zone.sql). The findings that change the plan:
- Every `id` runs over exactly 1 to the row count, except `aka_title.id`, which has a density of 0.956.
- Every child key has a parent row, except 93 rows of `aka_title.movie_id`.
- `cast_info` is sorted by `role_id` and grouped by `person_id` within each role. Its `movie_id` is scattered: 15.2 runs per movie, and every 122,880-row group spans 99.9 percent of the title domain. Zone maps on `cast_info.movie_id` skip nothing.
- `movie_keyword`, `movie_info_idx`, `movie_link`, `person_info` and `aka_name` are grouped by their parent key, 1.0 to 1.1 runs per value, but not sorted by it.
- `title`, `name`, `char_name`, `keyword`, `company_name` and `aka_title` are in random `id` order, about half of consecutive pairs descending.

## The load

DuckDB loads the CSVs in 365 s into a 2.36 GB file. rudb 0.4.32 has no `COPY FROM` for CSV, so it loaded Parquet written by DuckDB, into a 1.42 GB file. Loading one table per process was quadratic, 784 s for `cast_info` and 264 s for the 4 rows of `comp_cast_type`, because a keyed table committed empty made every later checkpoint rewrite the whole file. The rest was loaded in three batches of 284 s, 236 s and 277 s. That is a bug with a fix on the way, not a baseline, and the rudb load is measured again after the fix and `COPY FROM` land.
