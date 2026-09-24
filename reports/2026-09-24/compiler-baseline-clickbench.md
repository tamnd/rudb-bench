# ClickBench compiler baseline

This is the ClickBench half of the C0 baseline for tamnd/rudb#1828: what every ClickBench query costs in rudb today, next to DuckDB and ClickHouse, and how long rudb's frontend (parse, bind, optimize) takes on each one. It is a measurement and changes nothing in rudb.

## Read this first

The numbers come from server3, a shared machine that was busy with other people's work for the whole run. The one minute load average on its 8 threads was 43.00 when the DuckDB suite started, 26.61 for ClickHouse and 22.06 for rudb, and between 26 and 65 during the instruction and frontend passes. Instructions retired is the primary number in this report because load does not change it much. Wall time and the engine-reported query times are secondary: they are real, but a quiet machine would give smaller and steadier ones. The rudb hot runs have an interquartile range between 4% and 94% of the median (about 23% typical), which is well outside the harness's own 10% rule.

The data is the 10M sample, not the full table: 9,999,750 rows, one in every ten of the 99,997,497 in `hits.parquet`, snappy with 8192 row groups. The full table did not fit. server3 had between 0.1 and 16 GB free while this ran, and three engines each loading 100M rows needs far more than that. server1 has 4 cores and was not usable for this, gamingpc was not needed and server2 is unreachable. Nothing here is comparable to a full ClickBench run.

## What ran

| what | value |
| --- | --- |
| machine | server3, AMD EPYC, 8 threads, 23.5 GiB, Linux 6.8.0 |
| data | 10M sample, 9,999,750 rows, 1.40 GiB of Parquet |
| rudb | 0.4.31, tamnd/rudb main at 78e1c934e605e909590542df8bea71610412e031 (#1841), built with `cargo build --release -p rudb-cli` |
| DuckDB | v2.0.0-dev84237 cc7e7bac7f |
| ClickHouse | clickhouse-local 26.9.1.1138 |
| harness | rudb-bench abbbc47, 5 hot runs after one cold run, 600 s timeout |
| date | 2026-09-24, harness run 18:20 to 19:31 CEST, passes to 21:36 |

Each engine loaded the sample into its own format first:

| engine | load | on disk |
| --- | --- | --- |
| DuckDB | 381 s | 1.71 GiB |
| ClickHouse-local | 185 s | 1.03 GiB |
| rudb | 162 s | 1.06 GiB |

## How the numbers were taken

Query time is the harness rule: the median of the five hot runs of what each engine reports as its own query time, the same figure the public ClickBench board uses. The harness report for this run is next to this file as `compiler-baseline-clickbench-run.md`.

Instructions retired are counted with `perf stat -e instructions` using the protocol from `tpch-instructions-retired.md`: one query per fresh process, `threads=1`, the median of 3 rounds, with the count of `SELECT 1` on the same binary subtracted (36.4M for rudb, 170.7M for DuckDB). ClickHouse was not counted. Both databases were rebuilt from the sample with the same load statements the harness uses, one at a time because the disk could not hold both.

The frontend is read from the timing block rudb writes with `--metrics`: `parse_ns`, `bind_ns` and `optimize_ns`, one query per fresh process, `threads=1`, 9 rounds. The columns are the per-phase medians, the median of parse plus bind plus optimize (frontend), and the fastest round of that sum. The fastest round is shown because these are wall clock spans on a loaded machine, and the fastest is the one least inflated by preemption. The harness planning column is a different number: it is everything before execute on the cold run with all threads, so it also includes the physical plan build and any cold reads.

## How to reproduce it

On a Linux machine with the ClickBench data under `RUDB_BENCH_DATA` (default `~/rudb-data`). The harness writes the 10M sample from `hits.parquet` when it is missing. It deletes its databases at the end, so the two scripts need a rudb and a DuckDB database rebuilt from the sample with the load statements in `src/suite.rs`.

```
RUDB_BENCH_RUDB=/path/to/rudb/target/release/rudb RUDB_BENCH_CLICKHOUSE=/path/to/clickhouse rudb-bench run clickbench --rows 10000000 --engines duckdb,clickhouse-local,rudb --runs 5 --timeout 600 --report

cargo run --example export_queries -- clickbench rudb /tmp/cb-rudb.sql
cargo run --example export_queries -- clickbench duckdb /tmp/cb-duckdb.sql
scripts/instructions-per-query.py --engine rudb --binary /path/to/rudb --database hits.rudb --queries /tmp/cb-rudb.sql --rounds 3
scripts/instructions-per-query.py --engine duckdb --binary /usr/local/bin/duckdb --database hits.duckdb --queries /tmp/cb-duckdb.sql --rounds 3
scripts/frontend-latency.py --rudb /path/to/rudb --database hits.rudb --queries /tmp/cb-rudb.sql --rounds 9
```

`frontend-latency.py` now prints the fastest round beside the median; that change is part of this PR.

## Summary

| measure | DuckDB | ClickHouse-local | rudb | rudb/DuckDB | rudb/ClickHouse |
| --- | --- | --- | --- | --- | --- |
| instructions, total over 43 | 188.56G | not counted | 46.38G | 0.246x | |
| instructions, geomean | 928.1M | not counted | 218.6M | 0.236x | |
| query time, total over 43 | 138.398 s | | 14.684 s | 0.106x | |
| query time, geomean over 43 | 0.989 s | | 0.107 s | 0.108x | |
| query time, total over the 42 ClickHouse finished | 98.667 s | 142.283 s | 10.855 s | 0.110x | 0.076x |
| query time, geomean over the 42 | 0.906 s | 2.109 s | 0.098 s | 0.108x | 0.046x |
| wall time, total over 43 | 185.34 s | over 287.36 s | 20.25 s | 0.109x | |

On instructions, rudb does about a quarter of DuckDB's work over the suite. The query time ratio is closer to a tenth because the time runs used every thread and rudb kept more of them busy (1.07 cores of CPU per wall second against DuckDB's 0.57), and because DuckDB ran first, when the load was highest (43 against 22). Treat the time ratios as rough. rudb retires more instructions than DuckDB on two queries, q17 (1.13x) and q43 (1.06x), and more than 0.8x on q19, q38, q40 and q41.

32 of the 43 answers agreed across all three engines. Of the other 11, ten are ties at a LIMIT or a timezone rendering that the data does not settle, and q4 is the known DuckDB AVG(UserID) overflow, where rudb is right. The harness report lists them.

## Per query

Times are the harness hot medians. Instructions are single threaded with `SELECT 1` subtracted. Parse, bind, optimize, frontend and fastest frontend are from the 9 round frontend pass. Harness planning is the harness's cold "before execute" span.

| query | shape | duckdb | clickhouse-local | rudb | rudb/duckdb | rudb instructions | duckdb instructions | instructions ratio | parse | bind | optimize | frontend | fastest frontend | harness planning |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 6.0 ms | 158.0 ms | 2.1 ms | 0.35x | 1.0M | 20.3M | 0.05x | 95 µs | 503 µs | 143 µs | 730 µs | 522 µs | 2.6 ms |
| q2 | filtered count | 118.0 ms | 292.0 ms | 28.4 ms | 0.24x | 41.5M | 112.9M | 0.37x | 144 µs | 490 µs | 177 µs | 864 µs | 650 µs | 14.7 ms |
| q3 | three aggregates | 308.0 ms | 483.0 ms | 2.6 ms | 0.01x | 1.6M | 267.4M | 0.01x | 148 µs | 406 µs | 174 µs | 728 µs | 667 µs | 1.5 ms |
| q4 | average | 347.0 ms | 677.0 ms | 2.1 ms | 0.01x | 1.1M | 385.6M | 0.00x | 91 µs | 380 µs | 139 µs | 629 µs | 584 µs | 1.1 ms |
| q5 | count distinct, high card | 799.0 ms | 1.450 s | 2.3 ms | 0.00x | 0.8M | 1.22G | 0.00x | 95 µs | 370 µs | 141 µs | 618 µs | 566 µs | 960 µs |
| q6 | count distinct, strings | 1.587 s | 1.795 s | 1.9 ms | 0.00x | 1.1M | 1.17G | 0.00x | 98 µs | 365 µs | 177 µs | 635 µs | 591 µs | 1.2 ms |
| q7 | min and max of a date | 16.0 ms | 877.0 ms | 2.1 ms | 0.13x | 1.2M | 22.9M | 0.05x | 145 µs | 492 µs | 225 µs | 803 µs | 633 µs | 1.3 ms |
| q8 | group by, low card | 196.0 ms | 772.0 ms | 67.1 ms | 0.34x | 94.9M | 192.3M | 0.49x | 170 µs | 655 µs | 261 µs | 1189 µs | 997 µs | 22.8 ms |
| q9 | group by and count distinct | 979.0 ms | 2.483 s | 618.6 ms | 0.63x | 1.00G | 1.65G | 0.61x | 150 µs | 776 µs | 204 µs | 1272 µs | 912 µs | 143.0 ms |
| q10 | group by, several aggregates | 2.105 s | 1.745 s | 706.4 ms | 0.34x | 1.49G | 2.74G | 0.54x | 206 µs | 769 µs | 235 µs | 1313 µs | 870 µs | 5.0 ms |
| q11 | group by a string and count distinct | 597.0 ms | 826.0 ms | 114.0 ms | 0.19x | 334.6M | 573.6M | 0.58x | 167 µs | 723 µs | 286 µs | 1174 µs | 911 µs | 5.2 ms |
| q12 | group by two strings and count distinct | 1.075 s | 1.315 s | 181.0 ms | 0.17x | 382.8M | 647.0M | 0.59x | 205 µs | 785 µs | 270 µs | 1304 µs | 1111 µs | 2.0 ms |
| q13 | group by a string and top k | 1.708 s | 4.046 s | 217.3 ms | 0.13x | 373.2M | 1.43G | 0.26x | 188 µs | 735 µs | 459 µs | 4353 µs | 1219 µs | 2.6 ms |
| q14 | group by a string and count distinct | 2.401 s | 5.095 s | 364.8 ms | 0.15x | 1.01G | 2.04G | 0.50x | 170 µs | 818 µs | 480 µs | 1781 µs | 1171 µs | 4.3 ms |
| q15 | group by two columns and top k | 1.908 s | 4.153 s | 341.5 ms | 0.18x | 1.03G | 1.63G | 0.63x | 186 µs | 828 µs | 513 µs | 1713 µs | 1146 µs | 2.2 ms |
| q16 | group by, very high card | 1.043 s | 2.808 s | 148.8 ms | 0.14x | 491.0M | 1.37G | 0.36x | 144 µs | 984 µs | 187 µs | 1298 µs | 1017 µs | 6.5 ms |
| q17 | group by two, very high card | 4.267 s | 6.586 s | 549.5 ms | 0.13x | 3.07G | 2.72G | 1.13x | 170 µs | 1261 µs | 181 µs | 2141 µs | 991 µs | 1.8 ms |
| q18 | group by two, no ordering | 3.514 s | 2.666 s | 206.8 ms | 0.06x | 1.71G | 2.58G | 0.66x | 158 µs | 679 µs | 153 µs | 1015 µs | 828 µs | 1.5 ms |
| q19 | group by with an extract | 10.376 s | 11.911 s | 1.016 s | 0.10x | 4.06G | 4.70G | 0.86x | 331 µs | 4086 µs | 304 µs | 4484 µs | 1030 µs | 2.1 ms |
| q20 | point lookup | 128.0 ms | 1.234 s | 9.3 ms | 0.07x | 9.8M | 81.3M | 0.12x | 98 µs | 331 µs | 199 µs | 665 µs | 604 µs | 990 µs |
| q21 | substring scan | 6.511 s | 4.683 s | 284.2 ms | 0.04x | 1.61G | 5.14G | 0.31x | 115 µs | 406 µs | 179 µs | 750 µs | 592 µs | 1.6 ms |
| q22 | substring scan and group by | 5.857 s | 4.823 s | 435.7 ms | 0.07x | 1.83G | 5.48G | 0.33x | 224 µs | 1136 µs | 576 µs | 1929 µs | 1220 µs | 2.2 ms |
| q23 | two substring scans and group by | 9.597 s | 11.165 s | 1.052 s | 0.11x | 2.88G | 8.61G | 0.33x | 272 µs | 784 µs | 554 µs | 1798 µs | 1520 µs | 2.4 ms |
| q24 | select star and top k | 5.114 s | 7.433 s | 518.2 ms | 0.10x | 1.65G | 3.74G | 0.44x | 129 µs | 606 µs | 310 µs | 1099 µs | 939 µs | 3.3 ms |
| q25 | top k by a date | 265.0 ms | 4.450 s | 52.3 ms | 0.20x | 42.6M | 144.0M | 0.30x | 114 µs | 438 µs | 155 µs | 737 µs | 544 µs | 4.2 ms |
| q26 | top k by a string | 1.392 s | 2.685 s | 306.9 ms | 0.22x | 388.7M | 947.9M | 0.41x | 102 µs | 407 µs | 148 µs | 695 µs | 553 µs | 1.3 ms |
| q27 | top k by two columns | 252.0 ms | 2.159 s | 201.3 ms | 0.80x | 76.8M | 149.0M | 0.52x | 125 µs | 429 µs | 152 µs | 706 µs | 638 µs | 1.2 ms |
| q28 | group by with a string length | 6.341 s | 2.372 s | 629.8 ms | 0.10x | 1.11G | 4.36G | 0.25x | 226 µs | 758 µs | 694 µs | 4610 µs | 1516 µs | 69.2 ms |
| q29 | group by a regular expression | 39.731 s | failed | 3.829 s | 0.10x | 9.86G | 105.76G | 0.09x | 265 µs | 793 µs | 266 µs | 1392 µs | 1011 µs | 2.3 ms |
| q30 | ninety sums over one column | 1.581 s | 969.0 ms | 72.9 ms | 0.05x | 231.2M | 386.0M | 0.60x | 2120 µs | 1080 µs | 2246 µs | 6519 µs | 3886 µs | 47.1 ms |
| q31 | group by two and several aggregates | 3.193 s | 2.670 s | 306.6 ms | 0.10x | 1.39G | 1.97G | 0.70x | 221 µs | 855 µs | 503 µs | 1826 µs | 1193 µs | 2.3 ms |
| q32 | group by a high card pair | 3.391 s | 6.602 s | 336.8 ms | 0.10x | 1.58G | 2.27G | 0.70x | 225 µs | 760 µs | 547 µs | 1643 µs | 1098 µs | 2.1 ms |
| q33 | group by a high card pair, unfiltered | 6.009 s | 13.514 s | 801.8 ms | 0.13x | 4.50G | 6.29G | 0.72x | 194 µs | 733 µs | 165 µs | 1132 µs | 947 µs | 1.8 ms |
| q34 | group by a long string | 7.301 s | 11.349 s | 204.4 ms | 0.03x | 896.8M | 6.52G | 0.14x | 137 µs | 1056 µs | 222 µs | 1563 µs | 885 µs | 1.9 ms |
| q35 | group by a constant and a long string | 5.341 s | 5.044 s | 153.5 ms | 0.03x | 896.0M | 7.25G | 0.12x | 146 µs | 694 µs | 143 µs | 1114 µs | 895 µs | 1.5 ms |
| q36 | group by four expressions | 652.0 ms | 689.0 ms | 126.9 ms | 0.19x | 480.7M | 1.14G | 0.42x | 221 µs | 817 µs | 218 µs | 1372 µs | 1045 µs | 2.0 ms |
| q37 | date range and group by a URL | 495.0 ms | 1.419 s | 91.4 ms | 0.18x | 170.7M | 584.8M | 0.29x | 203 µs | 735 µs | 856 µs | 1884 µs | 1666 µs | 2.4 ms |
| q38 | date range and group by a title | 156.0 ms | 1.173 s | 44.6 ms | 0.29x | 146.8M | 171.3M | 0.86x | 205 µs | 739 µs | 838 µs | 3539 µs | 1420 µs | 2.5 ms |
| q39 | date range, group by and offset | 253.0 ms | 2.001 s | 71.3 ms | 0.28x | 72.1M | 373.2M | 0.19x | 237 µs | 772 µs | 326 µs | 2612 µs | 1078 µs | 2.0 ms |
| q40 | date range, a case and a wide group by | 993.0 ms | 3.410 s | 401.8 ms | 0.40x | 1.05G | 1.22G | 0.86x | 292 µs | 696 µs | 302 µs | 1332 µs | 1141 µs | 5.1 ms |
| q41 | date range with an IN and a hash | 197.0 ms | 692.0 ms | 104.9 ms | 0.53x | 114.1M | 135.0M | 0.85x | 243 µs | 707 µs | 458 µs | 3431 µs | 1279 µs | 6.1 ms |
| q42 | date range and a deep offset | 153.0 ms | 795.0 ms | 47.6 ms | 0.31x | 110.4M | 181.4M | 0.61x | 271 µs | 772 µs | 434 µs | 1524 µs | 1203 µs | 1.9 ms |
| q43 | minute buckets over a date range | 145.0 ms | 814.0 ms | 27.1 ms | 0.19x | 174.7M | 165.5M | 1.06x | 270 µs | 730 µs | 260 µs | 1280 µs | 1133 µs | 1.8 ms |

## Where rudb's time goes

The ten slowest rudb queries by query time are 69.2% of its 14.684 s total.

| query | shape | query time | share of time | instructions | share of instructions |
| --- | --- | --- | --- | --- | --- |
| q29 | group by a regular expression | 3.829 s | 26.1% | 9.86G | 21.3% |
| q23 | two substring scans and group by | 1.052 s | 7.2% | 2.88G | 6.2% |
| q19 | group by with an extract | 1.016 s | 6.9% | 4.06G | 8.8% |
| q33 | group by a high card pair, unfiltered | 801.8 ms | 5.5% | 4.50G | 9.7% |
| q10 | group by, several aggregates | 706.4 ms | 4.8% | 1.49G | 3.2% |
| q28 | group by with a string length | 629.8 ms | 4.3% | 1.11G | 2.4% |
| q9 | group by and count distinct | 618.6 ms | 4.2% | 1.00G | 2.2% |
| q17 | group by two, very high card | 549.5 ms | 3.7% | 3.07G | 6.6% |
| q24 | select star and top k | 518.2 ms | 3.5% | 1.65G | 3.6% |
| q22 | substring scan and group by | 435.7 ms | 3.0% | 1.83G | 3.9% |

By instructions the top ten are q29, q33, q19, q17, q23, q22, q18, q24, q21 and q32, together 70.6% of the 46.38G. q29 alone is a fifth of the suite on either measure. The harness splits rudb's execute time as Aggregate 57.8%, Scan 41.2% and TopN 0.8%, so the group by is where the work is.

## Frontend against the budget

The budget in tamnd/rudb#1828 is a frontend median of 0.3 ms and a max of 2 ms.

| measure | median | max |
| --- | --- | --- |
| frontend, median of 9 rounds per query | 1.304 ms | 6.519 ms (q30) |
| frontend, fastest of 9 rounds per query | 0.997 ms | 3.886 ms (q30) |
| parse, per query medians | 0.171 ms | 2.120 ms (q30) |
| bind, per query medians | 0.735 ms | 4.086 ms (q19) |
| optimize, per query medians | 0.260 ms | 2.246 ms (q30) |
| harness planning, cold, all threads, physical build included | 2.151 ms | 143.0 ms (q9) |

Every one of the 43 queries is over 0.3 ms, even taking the fastest round. On the medians eight queries are over 2 ms (q13, q17, q19, q28, q30, q38, q39, q41). On the fastest rounds only q30 is, so most of those eight are load and not the compiler. The slowest single run was 151 ms, which is the machine and not rudb. An earlier 5 round pass gave a median of 1.253 ms, so the median is stable at about 1 to 1.3 ms whichever way it is read. It is 3 to 4 times the budget, and load cannot explain that much of it.

Bind is the largest phase, more than half of the frontend on a typical query. Even q1, `SELECT COUNT(*) FROM hits`, spends about 0.5 ms in bind, and the simplest queries (q1 to q7, q20) come to 0.6 to 0.9 ms in all. That cost does not grow with the query, so it looks like a fixed per statement price for the 105 column `hits` table: opening and resolving the catalog and the table's schema in a fresh process. q30, ninety `SUM(ResolutionWidth + n)` terms, is the one query where parse (2.1 ms) and optimize (2.2 ms) are large. Optimize is also 0.4 to 0.9 ms on q13 to q15, q22, q23, q28, q31, q32, q37, q38, q41 and q42, the queries with a WHERE on strings or dates or several expressions.

The harness planning column is much larger than the frontend on a few queries (q9 143 ms, q28 69 ms, q30 47 ms, q8 23 ms, q2 15 ms). That span is the cold run and includes the physical build, which is 38.8 ms of q30's 47 ms. It is outside the frontend budget but worth knowing about, because it is paid on every cold statement.

## Plan for the frontend

1. Measure it properly before changing it. Add a way to count instructions for parse, bind and optimize alone, and to repeat one statement in one process so the warm cost is separate from first touch in a fresh process. Rerun on a quiet machine. The budget should be checked against that, not against wall clock under a load of 30.
2. Bind first, since it is the biggest phase and the same on every query. Profile q1 to see how much of the 0.5 ms is catalog open, schema resolution for 105 columns, or name lookup, then keep the resolved table in a form the binder can reuse instead of rebuilding it per statement.
3. q30 parse and optimize. Ninety similar expressions should cost tens of microseconds, not 2 ms each phase. Look for per-term copying or rules that walk the whole expression list per term.
4. The optimizer on the filtered queries (q28, q37, q38 and the rest above 0.4 ms). Check how many passes the rules take and whether the plan is copied on each.
5. Separately from the budget, find out what the physical build does for 38.8 ms on cold q30 and what makes q9's cold planning 143 ms.

## What went wrong

- The full 100M table did not fit on server3, so this is the 10M sample. The disk fell to about 100 MB free more than once because of other tenants, and one 10M attempt was stopped and restarted when it did.
- ClickHouse-local ran out of memory on q29 (limit 4.57 GiB), so its totals cover 42 queries and the ClickHouse ratios use the 42 it finished.
- The first DuckDB rebuild for the instruction pass was killed by the OOM killer and left an empty table. It was rebuilt with a 3 GB memory limit and 4 threads and checked at 9,999,750 rows before counting.
- The load was high all the way through, and the hot run spread is well above the harness's 10% rule on most queries. The time columns in this report should be read with that in mind.
- The harness sample was not written by the harness here: an existing stride-10 sample on server3 (`hits-10m.parquet`, 9,999,750 rows, snappy, 8192 row groups) was linked under the name the harness expects to save 1.4 GB of disk. Everything created on server3 has been removed.
