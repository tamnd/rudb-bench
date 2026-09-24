# JOB, the reduced SQL on DuckDB

Step J1 of the JOB plan (tamnd/rudb#1845). Every one of the 113 JOB queries was rewritten as a full semijoin reduction followed by a `MIN` per relation, with no join in it, and run on the pinned DuckDB next to the original. The question was whether the reduction gives the same answers on the real data. It does, on all 113. The second question was whether it is faster even on operators that were never built for it. It is not, on any of them.

## Setup

- Machine: server2, 6 cores, shared with other work. The load average was between 6 and 25 during these runs, so the absolute times are noisy. The two columns that are compared ran on the same machine in the same hour.
- Engine: DuckDB v2.0.0-dev84237 (`cc7e7bac7f`), the pinned build.
- Data: the IMDb snapshot of May 2013 from the JOB paper, 21 tables, 74,190,187 rows, loaded with JOB's own `schema.sql` and no indexes. The database file is 2.35 GB and the load took 365 s.
- Queries: `queries/job` for the originals and `queries/job-reduced` for the rewrites, written by `scripts/job-reduce.py`.
- Protocol: `scripts/job-run.py`. For each query, drop the page cache, start one process, run the query six times. The first run is reported as cold and the median of the other five as hot. The answer is checked on every run and a query whose answers differ between runs is marked unstable. None was.

## What the rewrite is

For each relation, the first CTE is its scan with its own filters, keeping only the join columns and the columns under `MIN`. GYO finds the join tree. Each edge becomes a `WHERE EXISTS` semijoin, once from the leaves to the root and once back down. Every step is `MATERIALIZED`, so that DuckDB cannot fold it back into a join. The answer is the `MIN` of each output over what is left of its relation. Query 1a is five scans, eight semijoins and three `MIN`s.

## Answers

All 113 reduced queries return exactly the answer of the original, byte for byte in list mode. That includes the five queries that are empty on this snapshot (2c, 5a, 5b, 10b and 32a) and the long string answers of 7c. So the theorem of `05-first-principles.md` holds on the real data: no JOB query needs a join to be answered.

The original answers are committed as `answers/job/<query>.csv`, taken from the same DuckDB with `-csv`, the column names on the first line and `NULL` spelled out.

## Time

| | DuckDB's plan | the reduced SQL |
| --- | ---: | ---: |
| hot total, 6 threads | 34,657 ms | 435,440 ms |
| cold total, 6 threads | 82,916 ms | 702,205 ms |
| hot total, 1 thread | 153,098 ms | not run |

The reduced SQL is slower on every one of the 113 queries. The median ratio is 11.2x slower and the geometric mean 10.6x. The closest are 5b (1.2x), 3b, 3c and 3a (1.3x to 1.5x) and 15c (1.7x). The furthest are 7a (173x), 7b (115x), 13a (85x), 12b (77x) and 8d (64x).

This does not say the reducer is slow, and it was never going to. It says what these operators cost when they are asked to do a reducer's work:
- Every first CTE copies its relation. When a relation has no filter, that is a copy of the whole table, `title` or `cast_info` or `movie_info`, before anything is reduced. The engine's reducer never copies; it keeps a bit per key.
- Every semijoin is a hash join that builds a hash table over the other side and probes it. The reducer's semijoin is a scan with a bit test.
- A relation that appears in several edges is copied again at each step, because each version is its own CTE.

The worst ratios are on the queries with the most large relations and the fewest filters on them. 7a and 7b read `cast_info`, `name`, `person_info` and `aka_name` with little in front of them, so every step copies millions of rows. The best ratios are where DuckDB's own plan is slow and the filters are selective, which is 3a to 3c, 5b and 15c. So the prototype is a check of correctness, and a price for the wrong implementation. `16-open-questions.md` section 16.1 stays open, and J3's first measured ratio settles it.

## Per query

The hot medians in milliseconds. The raw rows, with every sample and the answer, are in [job-reduced-prototype/](job-reduced-prototype/).

| query | DuckDB 6 threads, ms | DuckDB 1 thread, ms | reduced SQL 6 threads, ms | reduced over DuckDB |
| --- | ---: | ---: | ---: | ---: |
| 1a | 62 | 77 | 766 | 12.4x |
| 1b | 26 | 37 | 867 | 33.3x |
| 1c | 27 | 39 | 504 | 18.7x |
| 1d | 23 | 33 | 849 | 36.9x |
| 2a | 104 | 232 | 774 | 7.4x |
| 2b | 93 | 246 | 753 | 8.1x |
| 2c | 35 | 51 | 250 | 7.1x |
| 2d | 266 | 267 | 929 | 3.5x |
| 3a | 738 | 886 | 1122 | 1.5x |
| 3b | 643 | 732 | 859 | 1.3x |
| 3c | 1017 | 1168 | 1299 | 1.3x |
| 4a | 450 | 442 | 848 | 1.9x |
| 4b | 126 | 275 | 856 | 6.8x |
| 4c | 368 | 673 | 945 | 2.6x |
| 5a | 86 | 340 | 166 | 1.9x |
| 5b | 645 | 151 | 759 | 1.2x |
| 5c | 710 | 821 | 1434 | 2.0x |
| 6a | 403 | 860 | 2135 | 5.3x |
| 6b | 278 | 744 | 3852 | 13.9x |
| 6c | 273 | 502 | 2056 | 7.5x |
| 6d | 491 | 904 | 5704 | 11.6x |
| 6e | 285 | 897 | 2529 | 8.9x |
| 6f | 799 | 2102 | 8252 | 10.3x |
| 7a | 119 | 180 | 20628 | 173.3x |
| 7b | 91 | 252 | 10478 | 115.1x |
| 7c | 404 | 977 | 13517 | 33.5x |
| 8a | 143 | 351 | 3187 | 22.3x |
| 8b | 134 | 341 | 2992 | 22.3x |
| 8c | 737 | 7217 | 16969 | 23.0x |
| 8d | 293 | 1768 | 18648 | 63.6x |
| 9a | 304 | 2393 | 6495 | 21.4x |
| 9b | 223 | 2040 | 3586 | 16.1x |
| 9c | 396 | 1538 | 7529 | 19.0x |
| 9d | 416 | 2252 | 4573 | 11.0x |
| 10a | 102 | 456 | 2247 | 22.0x |
| 10b | 120 | 352 | 1762 | 14.7x |
| 10c | 329 | 2226 | 3671 | 11.2x |
| 11a | 48 | 53 | 2032 | 42.3x |
| 11b | 42 | 45 | 1116 | 26.6x |
| 11c | 68 | 64 | 1625 | 23.9x |
| 11d | 99 | 126 | 2772 | 28.0x |
| 12a | 99 | 295 | 5442 | 55.0x |
| 12b | 149 | 391 | 11525 | 77.3x |
| 12c | 127 | 323 | 3082 | 24.3x |
| 13a | 107 | 231 | 9129 | 85.3x |
| 13b | 161 | 400 | 3070 | 19.1x |
| 13c | 141 | 296 | 5813 | 41.2x |
| 13d | 221 | 455 | 3182 | 14.4x |
| 14a | 159 | 241 | 2604 | 16.4x |
| 14b | 94 | 271 | 1717 | 18.3x |
| 14c | 168 | 417 | 1720 | 10.2x |
| 16a | 154 | 508 | 3691 | 24.0x |
| 16b | 822 | 2840 | 5967 | 7.3x |
| 16c | 255 | 682 | 5539 | 21.7x |
| 16d | 215 | 670 | 5295 | 24.6x |
| 17a | 360 | 1179 | 12818 | 35.6x |
| 17b | 534 | 2235 | 9187 | 17.2x |
| 17c | 525 | 1939 | 8100 | 15.4x |
| 17d | 579 | 2022 | 7148 | 12.3x |
| 17e | 469 | 1457 | 11427 | 24.4x |
| 17f | 1153 | 2698 | 16243 | 14.1x |
| 18a | 391 | 866 | 4989 | 12.8x |
| 18b | 199 | 500 | 1660 | 8.3x |
| 18c | 317 | 852 | 2033 | 6.4x |
| 19a | 456 | 972 | 4315 | 9.5x |
| 19b | 263 | 563 | 3384 | 12.9x |
| 19c | 602 | 1791 | 6908 | 11.5x |
| 19d | 604 | 1895 | 5182 | 8.6x |
| 20a | 393 | 1461 | 6511 | 16.6x |
| 20b | 538 | 1393 | 11208 | 20.8x |
| 20c | 525 | 3552 | 11364 | 21.6x |
| 21a | 102 | 414 | 1993 | 19.5x |
| 21b | 74 | 94 | 2448 | 33.1x |
| 21c | 108 | 231 | 2244 | 20.8x |
| 22a | 244 | 557 | 2497 | 10.2x |
| 22b | 212 | 845 | 2281 | 10.8x |
| 22c | 401 | 3032 | 1791 | 4.5x |
| 22d | 799 | 12796 | 2805 | 3.5x |
| 23a | 254 | 2122 | 3116 | 12.3x |
| 23b | 151 | 1604 | 1121 | 7.4x |
| 23c | 281 | 2073 | 1706 | 6.1x |
| 24a | 387 | 3424 | 4278 | 11.1x |
| 24b | 189 | 1118 | 5683 | 30.1x |
| 25a | 320 | 2254 | 1862 | 5.8x |
| 25b | 148 | 1335 | 1180 | 8.0x |
| 25c | 374 | 2141 | 1820 | 4.9x |
| 26a | 350 | 4824 | 6641 | 19.0x |
| 26b | 177 | 1694 | 4784 | 27.0x |
| 26c | 415 | 4537 | 4287 | 10.3x |
| 27a | 70 | 228 | 1206 | 17.2x |
| 27b | 69 | 255 | 1005 | 14.6x |
| 27c | 78 | 167 | 836 | 10.7x |
| 28a | 501 | 3887 | 863 | 1.7x |
| 28b | 335 | 3514 | 885 | 2.6x |
| 28c | 511 | 7162 | 901 | 1.8x |
| 29a | 257 | 2106 | 2858 | 11.1x |
| 29b | 268 | 1140 | 3178 | 11.9x |
| 29c | 365 | 1870 | 3460 | 9.5x |
| 30a | 288 | 1386 | 1128 | 3.9x |
| 30b | 248 | 1238 | 1222 | 4.9x |
| 30c | 414 | 4831 | 838 | 2.0x |
| 31a | 327 | 3235 | 820 | 2.5x |
| 31b | 259 | 1724 | 799 | 3.1x |
| 31c | 443 | 3254 | 877 | 2.0x |
| 32a | 24 | 43 | 159 | 6.6x |
| 32b | 83 | 586 | 418 | 5.0x |
| 33a | 60 | 129 | 445 | 7.4x |
| 33b | 107 | 405 | 418 | 3.9x |
| 33c | 80 | 202 | 717 | 9.0x |
| 15a | 345 | 265 | 2821 | 8.2x |
| 15b | 293 | 256 | 1958 | 6.7x |
| 15c | 825 | 884 | 1390 | 1.7x |
| 15d | 625 | 731 | 1113 | 1.8x |
