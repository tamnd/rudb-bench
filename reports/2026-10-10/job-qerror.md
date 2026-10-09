# JOB, row estimates against DuckDB's

Step J8 of the JOB plan (tamnd/rudb#1852) asks for the q-error of rudb's estimates to be reported beside DuckDB's on the same queries. With the consistent reduction turned off, so that every query runs as an ordinary join plan, the median q-error of rudb's top join is 12.4 against DuckDB's 141.8, the median over every join is 13.3 against 93.0, and rudb's worst join estimate is closer to the truth than DuckDB's in 89 of the 113 queries.

## Setup

- Machine: server2, an AMD EPYC virtual machine with 6 cores at 2.0 GHz. The machine is shared, and the load average was between 8 and 11 during the runs.
- DuckDB: v2.0.0-dev84237 (`cc7e7bac7f`), SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`, on its own native format.
- rudb: main at `9b1bff08` with tamnd/rudb#2964, #2965, #2966 and #2967 on top, which is the J8 estimator as merged, built in release mode, SHA-256 `2a791cf37d60984c95b74de9e80cdf4e0c4fdbca71717b4932602512de2dfcc2`. The binary still calls itself 0.10.3.
- Data: the May 2013 IMDb snapshot of the JOB paper, loaded into both engines from the same CSV files with JOB's `schema.sql` and no indexes, which is the `plain` configuration.
- Protocol: `scripts/job-gate.py qerror` as committed at `858814b2`, with six threads. Each query runs once per engine outside any timing. rudb writes its metrics document and DuckDB writes its JSON profile, and every operator with an estimate is compared with the rows it produced. The q-error is the larger of the estimate and the true count over the smaller, with one row as the floor.

The per-query rows are in [job-qerror/](job-qerror/), with the run logs next to them. Each row gives, per engine, the number of joins with an estimate, the median and the largest q-error over those joins, the largest over every operator, and the q-error of the top join.

## With the rule turned off

This is the `norule` run, with `SET "plan.consistent" = false` and `SET stored_answers = false` sent to rudb before each query. Both engines plan the same join trees from their own estimates, so the numbers compare like with like.

| | Estimates | Median | 90th | 99th | Max |
| --- | ---: | ---: | ---: | ---: | ---: |
| DuckDB, top join | 113 | 141.8 | 46,281 | 670,390 | 1,855,296 |
| rudb, top join | 113 | 12.4 | 1,469 | 28,647 | 55,247 |
| DuckDB, every join | 910 | 93.0 | 11,855 | 350,933 | 2,967,144 |
| rudb, every join | 856 | 13.3 | 1,273 | 45,102 | 107,406 |
| DuckDB, every operator | 2,824 | 62.5 | 58,749 | 2,083,746 | 18,122,172 |
| rudb, every operator | 2,089 | 8.9 | 16,864 | 1,098,314 | 7,417,860 |

The top join of 52 queries is within a factor of 10 in rudb, against 15 in DuckDB, and 16 are within a factor of 2, against 5.

The 24 queries where rudb's worst join is not closer than DuckDB's are worth naming, because they are where the estimator has the most left to do. The largest gaps are 31c (107,406 against 1,454), 27b (67,543 against 342), 31a (58,294 against 1,778), 10c (46,711 against 1,348), 20a and 20b (33,772 and 45,029 against 211 and 88) and 27a (33,772 against 581). These are the queries that join `cast_info` or `movie_companies` under several correlated dimension filters, where treating the filters as independent puts the join too low. The same correlation is behind the underestimate on the `cast_info` side of 16b and 17e. It is not in this step and is left for later.

## With the rule on

In the `plain` run the consistent reduction is on, and rudb answers every query with a reduction plan that has no join operators, so it has no join estimates to grade. Over its other operators it has 1,179 estimates with a median q-error of 1.0, a 90th percentile of 46,048, a 99th of 1,349,923 and a largest of 7,417,860. DuckDB's numbers are the same as in the table above.

## What the numbers do not say

Both engines filter a scan under a hash join by the keys the other side found, so an operator under a join can produce fewer rows than the subplan it is the root of would, and its q-error is partly that filter working and not only the estimate being wrong. This is why the every operator rows have a long tail in both engines. Nothing filters the top join, so its q-error is the estimate against the true size of the whole join, and that is the row to read first.

rudb grades 856 joins where DuckDB grades 910. rudb removes a join to a parent that is read only for its key (tamnd/rudb#2966) and so has fewer joins in some plans, and the two engines do not always join the same tables in the same order, so the every join rows are not over the same set of joins. The top join rows are, since both have one per query.

This report is about the q-error. The P-error, which asks how much slower the plan chosen on the estimates is than the plan chosen on the true counts, is the other half of the J8 report and is not here yet.
