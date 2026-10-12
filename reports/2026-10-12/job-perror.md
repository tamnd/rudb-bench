# JOB, join orders against the true counts

Step J8 of the JOB plan (tamnd/rudb#1852) asks for the P-error of each engine's join order, which is how much more work the order it picked does than the best order the true counts allow. With the consistent reduction turned off, so that rudb plans ordinary join trees, the median P-error over the 113 queries is 1.54 for rudb and 8.09 for DuckDB, the 90th percentile is 17.6 against 871, and rudb's order is closer to the best one in 79 queries, as close in 8 and further in 26. Summed over the suite, the joins of rudb's trees produce 95 million rows and DuckDB's 248 million, where the best trees produce about 52 to 56 million.

## Setup

- Machine: server2, an AMD EPYC virtual machine with 6 cores at 2.0 GHz. The machine is shared, and the load average was between 1.2 and 2.4 during the runs.
- DuckDB: v2.0.0-dev84237 (`cc7e7bac7f`), SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`, on its own native format.
- rudb: 0.10.6 with tamnd/rudb#2984 on top, built in release mode, SHA-256 `c2434d377787b6598019203f5ed50b757a33ba0ecb320f31b0e5d3dd8ec1e9b3`.
- Data: the May 2013 IMDb snapshot of the JOB paper, loaded into both engines from the same CSV files with JOB's `schema.sql` and no indexes, which is the `plain` configuration.
- Protocol: `scripts/job-gate.py truecounts` and then `scripts/job-gate.py perror`, with six threads for the plans. Each query is planned once per engine outside any timing.

## How it is measured

`truecounts` counts the rows of the join of every connected set of a query's aliases, with the query's filters on, and of every alias on its own after its filters. It never runs those joins, since some of them, three fact tables on the movie and nothing else, run to trillions of rows. Every JOB query is acyclic, so a join tree over the aliases exists, and each alias counts its rows per join key and hands the counts to its parent, which multiplies them into its own. The count at the root is the exact size of the join. The suite has 71,406 such sets, and the three largest queries, 29a, 29b and 29c, have 13,248 each. 29a and 29b took about half an hour each on three threads. They are in [job-perror/counts.tsv.gz](job-perror/counts.tsv.gz), one set per line.

`perror` reads each engine's join tree off its plan. rudb's EXPLAIN names every alias. DuckDB's profile names only tables, so its scans are matched to aliases by table, then by every join in the tree joining aliases a condition connects, and where that leaves a choice, by the literals of each alias's filters. A semi join added as a reduction and an aggregation pushed under a join are passed through. The cost of a tree is C_out, the sum of the true sizes of its joins. The best cost is found by dynamic programming over the same counts, bushy and with no cross products, over the aliases the engine's tree reads. The P-error is the cost of the engine's tree over the best cost, with one row as the floor the way the q-error has it, so 1 means the engine picked a best order.

The per-query rows are in [job-perror/](job-perror/), with the log next to them. 32a was counted and graded in a second run, since the first count of it had failed, and its row is appended to `perror-norule.tsv`.

## With the rule turned off

This is the `norule` run, with `SET "plan.consistent" = false` and `SET stored_answers = false` sent to rudb before each query.

| | Queries | Median | 90th | Max | Geometric mean | Best order | Within 2x | Within 10x |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| DuckDB | 113 | 8.09 | 870.7 | 48,881 | 15.0 | 12 | 31 | 60 |
| rudb | 113 | 1.54 | 17.6 | 67,004 | 3.08 | 23 | 66 | 94 |

DuckDB's worst orders are where its estimates were worst in the q-error report. 10c, 2c, 12b, 19b, 14b and 5a are above 2,900. In 2c DuckDB's joins produce 190,392 rows where 4 would do, and rudb's produce 6.

rudb is further from the best order than DuckDB in 26 queries. Most of the gaps are small: in 14 of them both engines are within 4x of the best. The ones that matter are 10c (67,004 against 48,881), 29a (71.0 against 3.1), 28a (90.6 against 19.8), 5c (105 against 38), 28c (42.8 against 22.4), 15b (27.7 against 1.4) and 24b (16.9 against 3.2). 10c is the worst for both engines, where the best tree produces 80 rows and both pick trees that produce millions. These are the queries with several correlated filters on the dimensions of `cast_info` and `movie_companies`, where the q-error report found rudb's estimates too low, and that is the work left in the estimator.

## With the rule on

In the `plain` run the consistent reduction is on, and rudb answers every query with one reduction operator over all of its inputs, from 4 to 17 of them, and no binary join at all. There is no join order in such a plan for the P-error to grade, so the rudb columns of `perror-plain.tsv` say `error`. DuckDB's numbers are the same in both runs, with 32a graded only in the second.

## What the numbers do not say

The best cost is taken over the aliases the engine's own tree reads. rudb removes a join to a parent that is read only for its key (tamnd/rudb#2966), so in 21 queries, the 15, 16 and 17 families among them, its tree reads fewer aliases and its best cost is lower than DuckDB's. The P-error of each engine is against the best tree over the same aliases it read, so the two P-errors are fair, but the costs in the two columns are not over the same joins in those 21 queries.

C_out counts the rows the joins produce and nothing else. It does not see that both engines filter a probe side by the keys the build side found, or that rudb follows stored links instead of building a hash table for some joins, so a P-error above 1 is extra intermediate rows and not the same factor in time. It is the measure the JOB paper and the work after it use to compare join orders without the executor in the way, and that is what it is here for.
