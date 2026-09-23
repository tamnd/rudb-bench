# tpch on macbook-air-m4

**Superseded. Do not quote the 901.01x.** This run is rudb 0.3.38 reading the source Parquet through a view, so every query paid a full Parquet decode while DuckDB queried a file it had loaded once. That is why 17 of the 22 queries ran out of their 60 seconds. The harness now loads rudb into a native database file by default and rudb has had one since 0.4, so neither the timeouts nor the ratio describes the engine any more. On rudb 0.4.20 all 22 queries finish and all 22 answers agree with DuckDB. See `reports/2026-09-24/where-tpch-actually-stands.md`. The run below is left as it was written, because it is a correct record of what that version did.

This is one run of the tpch suite on macbook-air-m4, over 2 engines and 22 queries, with 5 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

## What ran

| what | it was |
| --- | --- |
| suite | tpch |
| queries | 22 |
| tables | 309.23 MiB of Parquet in 8 tables |
| rows | 8661245 in the table every query reads |
| corpus | duckdb-tpch SF1, corpus e02fbb7bb0145593, written 2026-09-18 |
| summary | median with the interquartile range, per reporting rule two |
| timeout | 60s per query, which 17 queries reached |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run tpch --engines duckdb,rudb --runs 5 --timeout 60 --report
```

The data goes under `RUDB_BENCH_DATA`, which defaults to `~/rudb-data`, and the engines have to be on the path. An engine that is not gets a row in the abstentions below rather than being left out of the table.

## The machine

| fact | value | source |
| --- | --- | --- |
| host | USERnoMacBook-Air.local | read |
| os | Darwin 24.6.0 arm64 | read |
| cpu | Apple M4 | read |
| threads | 10 | read |
| memory | 24.00 GiB | read |
| governor | no cpufreq here, the policy is not ours to see | not read |
| turbo | no intel_pstate here, the state is not ours to see | not read |
| filesystem | /dev/disk3s1 on /System/Volumes/Data (apfs, local, journaled, nobrowse, protect, root data) | read |
| page cache | no /proc/sys/vm/drop_caches, cold cannot be forced | not read |
| timer | /usr/bin/time BSD | read |

A fact that says it was not read is a fact about this machine and not a gap in the report. A field that had silently defaulted would be a lie that survived into it.

## The engines

| engine | version | state | load | load cpu | on disk | that size is | format | machine load |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 2.861s | 11.970s | 253.01 MiB | its own database file | its own | not read |
| rudb | rudb 0.3.38 | ran | 0.000us | 0.000us | 309.23 MiB | the source Parquet, this engine has no storage format of its own yet | the Parquet | not read |
| duckdb-pinned | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| clickhouse-local | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| datafusion | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| polars | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: rudb. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb on 5 shared |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 588.000ms | 1.434s | +144% | 1.439s | 3.450s | 2.41 | 274.78 MiB | not read | 324.06M/s | 11.30 GiB/s | 1.00x |
| rudb | not read | >1048.487s | not read | >1013.973s | not read | not read | not read | not read | 181.74K/s | 6.49 MiB/s | 901.01x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 22 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | rudb |
| --- | --- | --- | --- |
| q01 | pricing summary report | 35.000ms | 222.961ms |
| q02 | minimum cost supplier | 11.000ms | timeout at 60s |
| q03 | shipping priority | 19.000ms | timeout at 60s |
| q04 | order priority checking | 19.000ms | timeout at 60s |
| q05 | local supplier volume | 21.000ms | timeout at 60s |
| q06 | forecasting revenue change | 9.000ms | 27.802ms |
| q07 | volume shipping | 21.000ms | timeout at 60s |
| q08 | national market share | 20.000ms | timeout at 60s |
| q09 | product type profit measure | 51.000ms | timeout at 60s |
| q10 | returned item reporting | 36.000ms | timeout at 60s |
| q11 | important stock identification | 6.000ms | failed |
| q12 | shipping modes and order priority | 18.000ms | timeout at 60s |
| q13 | customer distribution | 68.000ms | timeout at 60s |
| q14 | promotion effect | 20.000ms | 34.632s |
| q15 | top supplier | 13.000ms | 951.462ms |
| q16 | parts supplier relationship | 29.000ms | timeout at 60s |
| q17 | small quantity order revenue | 17.000ms | timeout at 60s |
| q18 | large volume customer | 43.000ms | timeout at 60s |
| q19 | discounted revenue | 29.000ms | timeout at 60s |
| q20 | potential part promotion | 22.000ms | timeout at 60s |
| q21 | suppliers who kept orders waiting | 60.000ms | timeout at 60s |
| q22 | global sales opportunity | 21.000ms | 52.465s |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 35.000ms | 86.769ms | 76.628ms | 20.3% | 64.315ms | 79.869ms | 60.904ms | 83.159ms | 270.000ms | 91.02 MiB | not read | 113.03M/s |
| q02 | minimum cost supplier | 11.000ms | 59.096ms | 56.111ms | 18.0% | 46.221ms | 56.326ms | 44.052ms | 58.951ms | 50.000ms | 69.05 MiB | not read | 154.36M/s |
| q03 | shipping priority | 19.000ms | 56.553ms | 55.143ms | 16.0% | 47.418ms | 56.224ms | 47.315ms | 60.068ms | 120.000ms | 116.72 MiB | not read | 157.07M/s |
| q04 | order priority checking | 19.000ms | 59.710ms | 57.602ms | 4.3% | 56.562ms | 59.052ms | 53.072ms | 63.652ms | 110.000ms | 87.50 MiB | not read | 150.36M/s |
| q05 | local supplier volume | 21.000ms | 50.360ms | 58.418ms | 11.2% | 55.204ms | 61.757ms | 50.407ms | 67.241ms | 130.000ms | 116.08 MiB | not read | 148.26M/s |
| q06 | forecasting revenue change | 9.000ms | 52.691ms | 53.556ms | 7.2% | 51.721ms | 55.556ms | 32.100ms | 59.234ms | 70.000ms | 82.14 MiB | not read | 161.72M/s |
| q07 | volume shipping | 21.000ms | 57.091ms | 52.143ms | 7.6% | 50.994ms | 54.974ms | 49.214ms | 75.317ms | 140.000ms | 135.56 MiB | not read | 166.11M/s |
| q08 | national market share | 20.000ms | 47.304ms | 54.577ms | 10.3% | 52.542ms | 58.169ms | 51.216ms | 58.252ms | 110.000ms | 130.27 MiB | not read | 158.70M/s |
| q09 | product type profit measure | 51.000ms | 104.217ms | 86.446ms | 1.4% | 85.363ms | 86.546ms | 83.608ms | 95.187ms | 330.000ms | 274.78 MiB | not read | 100.19M/s |
| q10 | returned item reporting | 36.000ms | 86.392ms | 79.884ms | 4.0% | 78.614ms | 81.836ms | 76.185ms | 90.648ms | 190.000ms | 167.25 MiB | not read | 108.42M/s |
| q11 | important stock identification | 6.000ms | 29.328ms | 48.489ms | 46.9% | 30.617ms | 53.346ms | 30.500ms | 55.542ms | 30.000ms | 48.20 MiB | not read | 178.62M/s |
| q12 | shipping modes and order priority | 18.000ms | 53.308ms | 52.815ms | 11.2% | 50.166ms | 56.101ms | 45.184ms | 56.193ms | 130.000ms | 94.20 MiB | not read | 163.99M/s |
| q13 | customer distribution | 68.000ms | 95.832ms | 113.047ms | 1.7% | 112.563ms | 114.510ms | 110.399ms | 133.446ms | 390.000ms | 133.80 MiB | not read | 76.62M/s |
| q14 | promotion effect | 20.000ms | 59.394ms | 56.552ms | 28.7% | 55.631ms | 71.887ms | 49.898ms | 72.202ms | 110.000ms | 117.61 MiB | not read | 153.16M/s |
| q15 | top supplier | 13.000ms | 64.484ms | 51.165ms | 5.2% | 51.040ms | 53.692ms | 47.959ms | 55.537ms | 90.000ms | 101.02 MiB | not read | 169.28M/s |
| q16 | parts supplier relationship | 29.000ms | 74.449ms | 58.556ms | 13.3% | 54.482ms | 62.282ms | 52.720ms | 72.987ms | 90.000ms | 85.38 MiB | not read | 147.91M/s |
| q17 | small quantity order revenue | 17.000ms | 54.843ms | 56.165ms | 8.4% | 54.330ms | 59.049ms | 49.695ms | 60.014ms | 110.000ms | 108.73 MiB | not read | 154.21M/s |
| q18 | large volume customer | 43.000ms | 84.022ms | 89.467ms | 6.8% | 85.398ms | 91.523ms | 75.543ms | 97.959ms | 310.000ms | 160.58 MiB | not read | 96.81M/s |
| q19 | discounted revenue | 29.000ms | 51.945ms | 60.288ms | 14.9% | 56.042ms | 65.015ms | 54.545ms | 71.240ms | 180.000ms | 143.56 MiB | not read | 143.66M/s |
| q20 | potential part promotion | 22.000ms | 51.603ms | 57.977ms | 10.8% | 54.332ms | 60.580ms | 52.606ms | 75.618ms | 130.000ms | 117.48 MiB | not read | 149.39M/s |
| q21 | suppliers who kept orders waiting | 60.000ms | 104.616ms | 102.227ms | 7.9% | 100.533ms | 108.635ms | 93.650ms | 115.769ms | 310.000ms | 145.56 MiB | not read | 84.73M/s |
| q22 | global sales opportunity | 21.000ms | 55.096ms | 56.946ms | 5.7% | 55.055ms | 58.327ms | 47.105ms | 70.462ms | 50.000ms | 46.97 MiB | not read | 152.10M/s |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 22 of 22 queries. Total 588.000ms by its own clock and 1.434s by ours, 1.439s cold, 3.450s of CPU, peak 274.78 MiB, 324.06M/s and 11.30 GiB/s.

Running it cost 144% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 2.33x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## rudb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 222.961ms | 274.212ms | 250.603ms | 3.1% | 243.854ms | 251.594ms | 234.111ms | 275.810ms | 1.650s | 146.52 MiB | not read | 34.56M/s |
| q02 | minimum cost supplier | not read | 60.000s | 60.000s | 0.0% | 60.000s | 60.000s | 60.000s | 60.000s | not read | not read | not read | 144.35K/s |
| q03 | shipping priority | not read | 60.000s | 60.000s | 0.0% | 60.000s | 60.000s | 60.000s | 60.000s | not read | not read | not read | 144.35K/s |
| q04 | order priority checking | not read | 60.000s | 60.000s | 0.0% | 60.000s | 60.000s | 60.000s | 60.000s | not read | not read | not read | 144.35K/s |
| q05 | local supplier volume | not read | 60.000s | 60.000s | 0.0% | 60.000s | 60.000s | 60.000s | 60.000s | not read | not read | not read | 144.35K/s |
| q06 | forecasting revenue change | 27.802ms | 64.900ms | 59.826ms | 6.8% | 55.884ms | 59.960ms | 51.909ms | 61.750ms | 200.000ms | 78.95 MiB | not read | 144.77M/s |
| q07 | volume shipping | not read | 60.000s | 60.000s | 0.0% | 60.000s | 60.000s | 60.000s | 60.000s | not read | not read | not read | 144.35K/s |
| q08 | national market share | not read | 60.000s | 60.000s | 0.0% | 60.000s | 60.000s | 60.000s | 60.000s | not read | not read | not read | 144.35K/s |
| q09 | product type profit measure | not read | 60.000s | 60.000s | 0.0% | 60.000s | 60.000s | 60.000s | 60.000s | not read | not read | not read | 144.35K/s |
| q10 | returned item reporting | not read | 60.000s | 60.000s | 0.0% | 60.000s | 60.000s | 60.000s | 60.000s | not read | not read | not read | 144.35K/s |
| q11 | important stock identification | not read | 0.000us | 0.000us | n/a | 0.000us | 0.000us | 0.000us | 0.000us | not read | not read | not read | n/a |
| q12 | shipping modes and order priority | not read | 60.000s | 60.000s | 0.0% | 60.000s | 60.000s | 60.000s | 60.000s | not read | not read | not read | 144.35K/s |
| q13 | customer distribution | not read | 60.000s | 60.000s | 0.0% | 60.000s | 60.000s | 60.000s | 60.000s | not read | not read | not read | 144.35K/s |
| q14 | promotion effect | 34.632s | 33.524s | 34.670s | 3.6% | 33.494s | 34.740s | 31.109s | 37.125s | 161.560s | 158.17 MiB | not read | 249.82K/s |
| q15 | top supplier | 951.462ms | 1.011s | 986.022ms | 2.7% | 962.947ms | 989.372ms | 952.153ms | 1.115s | 1.460s | 136.44 MiB | not read | 8.78M/s |
| q16 | parts supplier relationship | not read | 60.000s | 60.000s | 0.0% | 60.000s | 60.000s | 60.000s | 60.000s | not read | not read | not read | 144.35K/s |
| q17 | small quantity order revenue | not read | 60.000s | 60.000s | 0.0% | 60.000s | 60.000s | 60.000s | 60.000s | not read | not read | not read | 144.35K/s |
| q18 | large volume customer | not read | 60.000s | 60.000s | 0.0% | 60.000s | 60.000s | 60.000s | 60.000s | not read | not read | not read | 144.35K/s |
| q19 | discounted revenue | not read | 60.000s | 60.000s | 0.0% | 60.000s | 60.000s | 60.000s | 60.000s | not read | not read | not read | 144.35K/s |
| q20 | potential part promotion | not read | 60.000s | 60.000s | 0.0% | 60.000s | 60.000s | 60.000s | 60.000s | not read | not read | not read | 144.35K/s |
| q21 | suppliers who kept orders waiting | not read | 60.000s | 60.000s | 0.0% | 60.000s | 60.000s | 60.000s | 60.000s | not read | not read | not read | 144.35K/s |
| q22 | global sales opportunity | 52.465s | 19.099s | 52.520s | 11.1% | 51.466s | 57.306s | 48.699s | 57.309s | 26.580s | 130.72 MiB | not read | 164.91K/s |

rudb rudb 0.3.38 over 22 of 22 queries. Total no reading by its own clock and 1048.487s by ours, 1013.973s cold, no reading of CPU, peak not read, 181.74K/s and 6.49 MiB/s.

#### Inside the engine

| query | planning | execute | accounted | measured | apart | driver | build | outside | held | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | 3.424ms | 233.621ms | 1.615s | 1.615s | 0.0% | 8.406ms | 801.417us | 3.809ms | 15.80 KiB | 6 of 6 |
| q06 | 984.375us | 29.778ms | 181.959ms | 181.969ms | 0.0% | 8.996ms | 413.500us | 7.617ms | 896 B | 5 of 5 |
| q14 | 954.959us | 33.504s | 192.505s | 192.505s | 0.0% | 15.850s | 412.125us | 4.497ms | 3.51 MiB | 10 of 10 |
| q15 | 2.570ms | 956.916ms | 1.464s | 1.464s | 0.0% | 140.890ms | 1.214ms | 14.833ms | 10.52 MiB | 22 of 22 |
| q22 | 661.332us | 19.066s | 18.751s | 18.751s | 0.0% | 2.383ms | 224.000us | 9.189ms | 40.41 MiB | 22 of 22 |

Read from the breakdown the engine wrote for its cold run. `planning` is everything before the first row moved, which is the parse, the bind, the optimizer passes and building the tree. It is a column rather than a footnote because it is the one cost of a query nobody profiles: an optimizer only ever has passes added to it, each paying for itself on the query it was written for, and a query that plans for four hundred milliseconds to save two hundred is a query the optimizer made slower. What is gated is planning as a share of the whole statement, and that share is not in this table, because it is taken over every run rather than off the cold one the rest of these columns come from. It lives in `baselines/planning-<suite>.txt`. `accounted` is what its pipelines charged themselves and `measured` is what it measured around running them, so `apart` is the cross check and anything over five percent is time the breakdown cannot explain. `driver` is the part of `accounted` that was not inside an operator, which is the loop that runs a pipeline rather than the operators it calls. `build` is what the engine spent putting the tree together, which is before there is a pipeline to charge and is why it is off the right hand side of the check. `outside` is the CPU the process spent on everything else, which is starting, opening the data and printing the answer, and it is the reason the wall clock column and the query time column differ. `held` is what the operators reserved, which is not the resident set of the process. `reference` is how many operators ran the reference implementation of their seam, which is the slow path kept for differential testing.

#### What the planner knew

| query | decisions | exact | certified | estimated | unknown |
| --- | --- | --- | --- | --- | --- |
| q01 | 6 | 0 | 0 | 0 | 6 |
| q06 | 5 | 2 | 0 | 0 | 3 |
| q14 | 10 | 2 | 0 | 0 | 8 |
| q15 | 22 | 2 | 0 | 0 | 20 |
| q22 | 22 | 2 | 0 | 0 | 20 |
| whole suite | 65 | 8 | 0 | 0 | 57 |

One row per query and one decision per operator, counted by what the planner had behind the cardinality it used. Over the whole suite that is 12% exact, 0% certified, 0% estimated and 88% unknown. `exact` is a counted number, `certified` is a number with a proven bound, `estimated` is a guess, and `unknown` is no number at all, which is an answer an operator has to handle rather than a null to paper over. Counts rather than fractions in the table because a suite's histogram is the sum of its queries' and fractions do not add.

#### Where the time went

| kind | cpu | share | operators | rows in | rows out | per row in | per row out | reference |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| CrossProduct | 102.716s | 51.7% | 2 | 171957 | 15296600000 | 597336.3ns | 6.7ns | 2 of 2 |
| Filter | 49.204s | 24.8% | 12 | 15326977090 | 6750145 | 3.2ns | 7289.4ns | 12 of 12 |
| Aggregate | 27.193s | 13.7% | 9 | 8113146 | 120011 | 3351.8ns | 226591.7ns | 9 of 9 |
| Join | 18.608s | 9.4% | 3 | 71015 | 71015 | 262029.0ns | 262029.0ns | 3 of 3 |
| FileScan | 741.286ms | 0.4% | 10 | 0 | 32016075 | handed none | 23.2ns | 10 of 10 |
| Project | 39.223ms | 0.0% | 21 | 8475173 | 8475173 | 4.6ns | 4.6ns | 21 of 21 |
| Gather | 3.435ms | 0.0% | 3 | 99998 | 0 | 34.4ns | handed on none | 3 of 3 |
| Keep | 110.090us | 0.0% | 2 | 210000 | 0 | 0.5ns | handed on none | 2 of 2 |
| Sort | 50.542us | 0.0% | 3 | 12 | 12 | 4211.8ns | 4211.8ns | 3 of 3 |

Every operator the engine ran over the whole suite, added up by kind, off the cold run of each query. `share` is of what the operators charged rather than of the wall clock, so the driver and the process startup are not in the denominator and the column adds to a hundred percent. `per row in` is the number to compare across kinds, and it is missing for a scan because a scan is handed nothing, so read `per row out` for that one. `reference` is how many of them ran the reference implementation of their seam, which is the slow path kept for differential testing, and a kind that is all reference is a kind whose number is about the slow path rather than about the engine.

The most expensive kind is CrossProduct, and the queries where it cost the most are q14 at 102.382s, q15 at 333.807ms.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- q01 swung by 20.3% of its median, and rule two wants under 10%
- q02 swung by 18.0% of its median, and rule two wants under 10%
- q03 swung by 16.0% of its median, and rule two wants under 10%
- q05 swung by 11.2% of its median, and rule two wants under 10%
- q08 swung by 10.3% of its median, and rule two wants under 10%
- q11 swung by 46.9% of its median, and rule two wants under 10%
- q12 swung by 11.2% of its median, and rule two wants under 10%
- q14 swung by 28.7% of its median, and rule two wants under 10%
- q16 swung by 13.3% of its median, and rule two wants under 10%
- q19 swung by 14.9% of its median, and rule two wants under 10%
- q20 swung by 10.8% of its median, and rule two wants under 10%

These swung wider than reporting rule two allows:

- duckdb swung by 46.9% of its median on q11, and rule two wants under 10%
- rudb swung by 11.1% of its median on q22, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

Answers differ, so this is not a comparison: q02: rudb does not agree with duckdb: 0 numbers against 200

Answers differ, so this is not a comparison: q03: rudb does not agree with duckdb: 0 numbers against 30

Answers differ, so this is not a comparison: q04: rudb does not agree with duckdb: 0 numbers against 5

Answers differ, so this is not a comparison: q05: rudb does not agree with duckdb: 0 numbers against 5

Answers differ, so this is not a comparison: q07: rudb does not agree with duckdb: 0 numbers against 8

Answers differ, so this is not a comparison: q08: rudb does not agree with duckdb: 0 numbers against 4

Answers differ, so this is not a comparison: q09: rudb does not agree with duckdb: 0 numbers against 350

Answers differ, so this is not a comparison: q10: rudb does not agree with duckdb: 0 numbers against 60

Answers differ, so this is not a comparison: q11: rudb does not agree with duckdb: 0 numbers against 2096

Answers differ, so this is not a comparison: q12: rudb does not agree with duckdb: 0 numbers against 4

Answers differ, so this is not a comparison: q13: rudb does not agree with duckdb: 0 numbers against 84

Answers differ, so this is not a comparison: q16: rudb does not agree with duckdb: 0 numbers against 36628

Answers differ, so this is not a comparison: q17: rudb does not agree with duckdb: 0 numbers against 1

Answers differ, so this is not a comparison: q18: rudb does not agree with duckdb: 0 numbers against 228

Answers differ, so this is not a comparison: q19: rudb does not agree with duckdb: 0 numbers against 1

Answers differ, so this is not a comparison: q20: rudb does not agree with duckdb: the same 0 numbers and 372 of 372 text fields different, so the disagreement is which rows came back rather than what they hold

Answers differ, so this is not a comparison: q21: rudb does not agree with duckdb: 0 numbers against 100

Hot here means page cache warm and not buffer pool warm, because every run is a fresh process so that no query's number depends on the one before it.

