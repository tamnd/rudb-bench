# JOB, the reduced SQL on DuckDB

Step J1 of the JOB plan (tamnd/rudb#1845). Every one of the 113 JOB queries was rewritten as a full semijoin reduction followed by a `MIN` per relation, with no join in it, and run on the pinned DuckDB next to the original. The question was whether the reduction gives the same answers on the real data. It does, on all 113. The second question was whether it is faster even on operators that were never built for it. At six threads it is not, on any of them. At one thread it is, on 23 of them.

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
| hot total, 1 thread | 153,098 ms | 280,894 ms |
| cold total, 1 thread | 364,865 ms | 596,798 ms |

At six threads the reduced SQL is slower on every one of the 113 queries. The median ratio is 11.2x slower and the geometric mean 10.6x. The closest are 5b (1.2x), 3b, 3c and 3a (1.3x to 1.5x) and 15c (1.7x). The furthest are 7a (173x), 7b (115x), 13a (85x), 12b (77x) and 8d (64x).

This does not say the reducer is slow, and it was never going to. It says what these operators cost when they are asked to do a reducer's work:
- Every first CTE copies its relation. When a relation has no filter, that is a copy of the whole table, `title` or `cast_info` or `movie_info`, before anything is reduced. The engine's reducer never copies; it keeps a bit per key.
- Every semijoin is a hash join that builds a hash table over the other side and probes it. The reducer's semijoin is a scan with a bit test.
- A relation that appears in several edges is copied again at each step, because each version is its own CTE.

The worst ratios are on the queries with the most large relations and the fewest filters on them. 7a and 7b read `cast_info`, `name`, `person_info` and `aka_name` with little in front of them, so every step copies millions of rows. The best ratios are where DuckDB's own plan is slow and the filters are selective, which is 3a to 3c, 5b and 15c. At one thread the picture is less one sided. The reduced SQL is 1.8x slower in total, the median ratio is 2.8x and the geometric mean 2.7x, and it is faster than DuckDB's own plan on 23 of the 113 queries. The best are 22d (6.7x faster), 28c (4.4x), 28a and 28b (2.6x), 23b (2.1x) and 30c (1.9x). The worst are 11a, 11b and 11c (22x to 25x slower), 16c and 7a (15x). The gap between the two thread counts is DuckDB's scaling: its own plan gets 4.4x faster on six cores and the reduced SQL only 1.6x, because a chain of materialized CTEs runs one step at a time. So on one core, with the copies and the hash tables still in it, the reduction already beats a good join order on a fifth of JOB, and those are the queries with the largest intermediate results.

So the prototype is a check of correctness, and a price for the wrong implementation. `16-open-questions.md` section 16.1 stays open, and J3's first measured ratio settles it.

## Per query

The hot medians in milliseconds. The raw rows, with every sample and the answer, are in [job-reduced-prototype/](job-reduced-prototype/).

| query | DuckDB 6 threads, ms | DuckDB 1 thread, ms | reduced SQL 6 threads, ms | reduced over DuckDB | reduced SQL 1 thread, ms | reduced over DuckDB, 1 thread |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1a | 62 | 77 | 766 | 12.4x | 311 | 4.0x |
| 1b | 26 | 37 | 867 | 33.3x | 314 | 8.5x |
| 1c | 27 | 39 | 504 | 18.7x | 239 | 6.1x |
| 1d | 23 | 33 | 849 | 36.9x | 336 | 10.2x |
| 2a | 104 | 232 | 774 | 7.4x | 374 | 1.6x |
| 2b | 93 | 246 | 753 | 8.1x | 424 | 1.7x |
| 2c | 35 | 51 | 250 | 7.1x | 147 | 2.9x |
| 2d | 266 | 267 | 929 | 3.5x | 600 | 2.2x |
| 3a | 738 | 886 | 1122 | 1.5x | 1329 | 1.5x |
| 3b | 643 | 732 | 859 | 1.3x | 1002 | 1.4x |
| 3c | 1017 | 1168 | 1299 | 1.3x | 1436 | 1.2x |
| 4a | 450 | 442 | 848 | 1.9x | 408 | 0.9x |
| 4b | 126 | 275 | 856 | 6.8x | 368 | 1.3x |
| 4c | 368 | 673 | 945 | 2.6x | 1190 | 1.8x |
| 5a | 86 | 340 | 166 | 1.9x | 182 | 0.5x |
| 5b | 645 | 151 | 759 | 1.2x | 1639 | 10.9x |
| 5c | 710 | 821 | 1434 | 2.0x | 3016 | 3.7x |
| 6a | 403 | 860 | 2135 | 5.3x | 1829 | 2.1x |
| 6b | 278 | 744 | 3852 | 13.9x | 1942 | 2.6x |
| 6c | 273 | 502 | 2056 | 7.5x | 1173 | 2.3x |
| 6d | 491 | 904 | 5704 | 11.6x | 1923 | 2.1x |
| 6e | 285 | 897 | 2529 | 8.9x | 3472 | 3.9x |
| 6f | 799 | 2102 | 8252 | 10.3x | 4953 | 2.4x |
| 7a | 119 | 180 | 20628 | 173.3x | 2720 | 15.1x |
| 7b | 91 | 252 | 10478 | 115.1x | 2201 | 8.7x |
| 7c | 404 | 977 | 13517 | 33.5x | 7869 | 8.1x |
| 8a | 143 | 351 | 3187 | 22.3x | 2816 | 8.0x |
| 8b | 134 | 341 | 2992 | 22.3x | 1359 | 4.0x |
| 8c | 737 | 7217 | 16969 | 23.0x | 6167 | 0.9x |
| 8d | 293 | 1768 | 18648 | 63.6x | 5295 | 3.0x |
| 9a | 304 | 2393 | 6495 | 21.4x | 2324 | 1.0x |
| 9b | 223 | 2040 | 3586 | 16.1x | 1543 | 0.8x |
| 9c | 396 | 1538 | 7529 | 19.0x | 2351 | 1.5x |
| 9d | 416 | 2252 | 4573 | 11.0x | 2497 | 1.1x |
| 10a | 102 | 456 | 2247 | 22.0x | 1717 | 3.8x |
| 10b | 120 | 352 | 1762 | 14.7x | 2683 | 7.6x |
| 10c | 329 | 2226 | 3671 | 11.2x | 1849 | 0.8x |
| 11a | 48 | 53 | 2032 | 42.3x | 1341 | 25.3x |
| 11b | 42 | 45 | 1116 | 26.6x | 1126 | 25.0x |
| 11c | 68 | 64 | 1625 | 23.9x | 1427 | 22.3x |
| 11d | 99 | 126 | 2772 | 28.0x | 1158 | 9.2x |
| 12a | 99 | 295 | 5442 | 55.0x | 3041 | 10.3x |
| 12b | 149 | 391 | 11525 | 77.3x | 4535 | 11.6x |
| 12c | 127 | 323 | 3082 | 24.3x | 2155 | 6.7x |
| 13a | 107 | 231 | 9129 | 85.3x | 3080 | 13.3x |
| 13b | 161 | 400 | 3070 | 19.1x | 1428 | 3.6x |
| 13c | 141 | 296 | 5813 | 41.2x | 1035 | 3.5x |
| 13d | 221 | 455 | 3182 | 14.4x | 1452 | 3.2x |
| 14a | 159 | 241 | 2604 | 16.4x | 1902 | 7.9x |
| 14b | 94 | 271 | 1717 | 18.3x | 1934 | 7.1x |
| 14c | 168 | 417 | 1720 | 10.2x | 3934 | 9.4x |
| 16a | 154 | 508 | 3691 | 24.0x | 6831 | 13.4x |
| 16b | 822 | 2840 | 5967 | 7.3x | 9101 | 3.2x |
| 16c | 255 | 682 | 5539 | 21.7x | 10550 | 15.5x |
| 16d | 215 | 670 | 5295 | 24.6x | 7171 | 10.7x |
| 17a | 360 | 1179 | 12818 | 35.6x | 4192 | 3.6x |
| 17b | 534 | 2235 | 9187 | 17.2x | 6451 | 2.9x |
| 17c | 525 | 1939 | 8100 | 15.4x | 6214 | 3.2x |
| 17d | 579 | 2022 | 7148 | 12.3x | 6216 | 3.1x |
| 17e | 469 | 1457 | 11427 | 24.4x | 5857 | 4.0x |
| 17f | 1153 | 2698 | 16243 | 14.1x | 7463 | 2.8x |
| 18a | 391 | 866 | 4989 | 12.8x | 2867 | 3.3x |
| 18b | 199 | 500 | 1660 | 8.3x | 1874 | 3.7x |
| 18c | 317 | 852 | 2033 | 6.4x | 2145 | 2.5x |
| 19a | 456 | 972 | 4315 | 9.5x | 3175 | 3.3x |
| 19b | 263 | 563 | 3384 | 12.9x | 2537 | 4.5x |
| 19c | 602 | 1791 | 6908 | 11.5x | 3893 | 2.2x |
| 19d | 604 | 1895 | 5182 | 8.6x | 2440 | 1.3x |
| 20a | 393 | 1461 | 6511 | 16.6x | 2381 | 1.6x |
| 20b | 538 | 1393 | 11208 | 20.8x | 3255 | 2.3x |
| 20c | 525 | 3552 | 11364 | 21.6x | 3034 | 0.9x |
| 21a | 102 | 414 | 1993 | 19.5x | 1405 | 3.4x |
| 21b | 74 | 94 | 2448 | 33.1x | 1314 | 14.0x |
| 21c | 108 | 231 | 2244 | 20.8x | 1476 | 6.4x |
| 22a | 244 | 557 | 2497 | 10.2x | 1636 | 2.9x |
| 22b | 212 | 845 | 2281 | 10.8x | 1991 | 2.4x |
| 22c | 401 | 3032 | 1791 | 4.5x | 1794 | 0.6x |
| 22d | 799 | 12796 | 2805 | 3.5x | 1896 | 0.1x |
| 23a | 254 | 2122 | 3116 | 12.3x | 1411 | 0.7x |
| 23b | 151 | 1604 | 1121 | 7.4x | 769 | 0.5x |
| 23c | 281 | 2073 | 1706 | 6.1x | 1143 | 0.6x |
| 24a | 387 | 3424 | 4278 | 11.1x | 2986 | 0.9x |
| 24b | 189 | 1118 | 5683 | 30.1x | 2881 | 2.6x |
| 25a | 320 | 2254 | 1862 | 5.8x | 2044 | 0.9x |
| 25b | 148 | 1335 | 1180 | 8.0x | 1921 | 1.4x |
| 25c | 374 | 2141 | 1820 | 4.9x | 2492 | 1.2x |
| 26a | 350 | 4824 | 6641 | 19.0x | 3398 | 0.7x |
| 26b | 177 | 1694 | 4784 | 27.0x | 2874 | 1.7x |
| 26c | 415 | 4537 | 4287 | 10.3x | 2895 | 0.6x |
| 27a | 70 | 228 | 1206 | 17.2x | 1341 | 5.9x |
| 27b | 69 | 255 | 1005 | 14.6x | 1131 | 4.4x |
| 27c | 78 | 167 | 836 | 10.7x | 1333 | 8.0x |
| 28a | 501 | 3887 | 863 | 1.7x | 1450 | 0.4x |
| 28b | 335 | 3514 | 885 | 2.6x | 1367 | 0.4x |
| 28c | 511 | 7162 | 901 | 1.8x | 1644 | 0.2x |
| 29a | 257 | 2106 | 2858 | 11.1x | 2675 | 1.3x |
| 29b | 268 | 1140 | 3178 | 11.9x | 2532 | 2.2x |
| 29c | 365 | 1870 | 3460 | 9.5x | 3488 | 1.9x |
| 30a | 288 | 1386 | 1128 | 3.9x | 2756 | 2.0x |
| 30b | 248 | 1238 | 1222 | 4.9x | 2241 | 1.8x |
| 30c | 414 | 4831 | 838 | 2.0x | 2536 | 0.5x |
| 31a | 327 | 3235 | 820 | 2.5x | 2138 | 0.7x |
| 31b | 259 | 1724 | 799 | 3.1x | 2291 | 1.3x |
| 31c | 443 | 3254 | 877 | 2.0x | 3019 | 0.9x |
| 32a | 24 | 43 | 159 | 6.6x | 308 | 7.2x |
| 32b | 83 | 586 | 418 | 5.0x | 585 | 1.0x |
| 33a | 60 | 129 | 445 | 7.4x | 1285 | 10.0x |
| 33b | 107 | 405 | 418 | 3.9x | 1065 | 2.6x |
| 33c | 80 | 202 | 717 | 9.0x | 1687 | 8.4x |
| 15a | 345 | 265 | 2821 | 8.2x | 1500 | 5.7x |
| 15b | 293 | 256 | 1958 | 6.7x | 2565 | 10.0x |
| 15c | 825 | 884 | 1390 | 1.7x | 1798 | 2.0x |
| 15d | 625 | 731 | 1113 | 1.8x | 2640 | 3.6x |
