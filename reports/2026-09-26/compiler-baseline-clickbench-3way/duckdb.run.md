# clickbench on gamingpc-wsl

This is one run of the clickbench suite on gamingpc-wsl, over 1 engine and 43 queries, with 4 tries of each query after a page cache drop, the way the upstream ClickBench driver runs it. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

## What ran

| what | it was |
| --- | --- |
| suite | clickbench |
| queries | 43 |
| tables | 13.76 GiB of Parquet in 1 table |
| rows | 99997497 in the table every query reads |
| corpus | no manifest beside the data, so this run cannot say where it came from |
| summary | the best of the tries after the first, which is the upstream ClickBench convention |
| timeout | 600s per query, and no query reached it |
| harness | rudb-bench 0.0.1 |

## How it was measured

Every engine loaded the data first, one at a time. Then each query was run on every engine in turn, and the order rotated by one engine per query, so no engine always went first or last. Before each engine's turn the harness waited for the one minute load average to be below the 16 hardware threads, dropped the page cache (and for the ClickHouse server stopped it first and started it again after), then ran the query 4 times in a row. The first try is the cold figure and the best of the other 3 is the hot one, which is what the upstream ClickBench driver does. Every figure below is the engine's own timing where it reports one.

Every engine got the same memory budget, 20.00 GiB (21474836480 bytes, from RUDB_BENCH_MEMORY). DuckDB and rudb got it as `SET memory_limit`, the ClickHouse server as `max_server_memory_usage` and `clickhouse local` as `max_memory_usage`.

The data file, which is also written beside it as a manifest:

| path | rows | bytes | sha256 |
| --- | --- | --- | --- |
| `/home/gopher/c0-gpc/data/hits.parquet` | 99997497 | 14779976446 | `a390f6cb782f6aaef278c72fc1dd86c4f30bc843ebab3c159e9bd4d45ddb079f` |

The one minute load average at the start of each engine's turns, counting its load and every query:

| engine | readings | lowest | median | highest | held by the gate | went first |
| --- | --- | --- | --- | --- | --- | --- |
| duckdb | 44 | 0.29 | 1.58 | 1.81 | 420.002s | 43 |

The reading includes the tail of whatever ran just before, which is usually the previous engine's turn, because the one minute average takes about a minute to decay.

rudb writes summaries of each column when it loads a table, and it can answer some queries from those without reading the rows. This run turned that off with `SET stored_answers = false` before every rudb query (`RUDB_BENCH_STORED_ANSWERS=off`), so rudb read the rows the way the other engines did. Its metrics said it still answered from stored summaries for no query. The per query table marks any such query, and it also marks q1 to q7, whose shapes (counts, sums, averages, distinct counts and bounds over the whole table) are the ones a summary can answer. The headline below gives the ratios both with and without q1 to q7.

## Headline

| engine | hot total | cold total | hot geomean | total vs duckdb | geomean vs duckdb | total vs, without q1 to q7 | geomean vs, without q1 to q7 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 14.381s | 18.178s | 0.1225s | 1.000x | 1.000x | 1.000x | 1.000x |

Over the 43 queries every engine finished, 36 of them outside q1 to q7. Hot is the best of the tries after the first and cold is the first try, both by the engine's own clock. The geometric mean is over the hot figures. A ratio under 1 means faster than duckdb.

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
RUDB_BENCH_STORED_ANSWERS=off rudb-bench run clickbench --engines duckdb --runs 4 --protocol upstream --timeout 600 --report
```

The data goes under `RUDB_BENCH_DATA`, which defaults to `~/rudb-data`, and the engines have to be on the path. An engine that is not gets a row in the abstentions below rather than being left out of the table.

## The machine

| fact | value | source |
| --- | --- | --- |
| host | GamingPC | read |
| os | Linux 6.18.33.2-microsoft-standard-WSL2 x86_64 | read |
| cpu | 13th Gen Intel(R) Core(TM) i9-13900K | read |
| threads | 32 | read |
| memory | 31.34 GiB | read |
| governor | no cpufreq here, the policy is not ours to see | not read |
| turbo | no intel_pstate here, the state is not ours to see | not read |
| filesystem | /dev/sdd / ext4 rw,relatime,discard,errors=remount-ro,data=ordered 0 0 | read |
| page cache | droppable, cold runs are cold | read |
| timer | /usr/bin/time GNU | read |

A fact that says it was not read is a fact about this machine and not a gap in the report. A field that had silently defaulted would be a lie that survived into it.

## The engines

| engine | version | state | load | load cpu | on disk | that size is | format | machine load |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 42.066s | 508.270s | 19.05 GiB | its own database file | its own | 0.29 to 6.31 |
| duckdb-pinned | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| clickhouse-local | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| datafusion | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| polars | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |
| rudb | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

The machine load column is the one minute load average before and after that engine's suite, on a machine with 16 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

The cold column is a run that had to go to the device. The page cache was dropped before each query's first run, the way official ClickBench does it.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 14.381s | 17.562s | +22% | 23.502s | 211.260s | 12.03 | 8.86 GiB | 187.51 MiB | 299.00M/s | 41.16 GiB/s | 1.00x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 43 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | rudb from stored summaries |
| --- | --- | --- | --- |
| q1 | count | 9.000ms | no, but the shape allows it |
| q2 | filtered count | 23.000ms | no, but the shape allows it |
| q3 | three aggregates | 46.000ms | no, but the shape allows it |
| q4 | average | 46.000ms | no, but the shape allows it |
| q5 | count distinct, high card | 183.000ms | no, but the shape allows it |
| q6 | count distinct, strings | 212.000ms | no, but the shape allows it |
| q7 | min and max of a date | 16.000ms | no, but the shape allows it |
| q8 | group by, low card | 22.000ms |  |
| q9 | group by and count distinct | 231.000ms |  |
| q10 | group by, several aggregates | 317.000ms |  |
| q11 | group by a string and count distinct | 83.000ms |  |
| q12 | group by two strings and count distinct | 101.000ms |  |
| q13 | group by a string and top k | 241.000ms |  |
| q14 | group by a string and count distinct | 460.000ms |  |
| q15 | group by two columns and top k | 278.000ms |  |
| q16 | group by, very high card | 218.000ms |  |
| q17 | group by two, very high card | 539.000ms |  |
| q18 | group by two, no ordering | 383.000ms |  |
| q19 | group by with an extract | 987.000ms |  |
| q20 | point lookup | 29.000ms |  |
| q21 | substring scan | 399.000ms |  |
| q22 | substring scan and group by | 439.000ms |  |
| q23 | two substring scans and group by | 483.000ms |  |
| q24 | select star and top k | 88.000ms |  |
| q25 | top k by a date | 38.000ms |  |
| q26 | top k by a string | 86.000ms |  |
| q27 | top k by two columns | 39.000ms |  |
| q28 | group by with a string length | 350.000ms |  |
| q29 | group by a regular expression | 3.484s |  |
| q30 | ninety sums over one column | 43.000ms |  |
| q31 | group by two and several aggregates | 248.000ms |  |
| q32 | group by a high card pair | 350.000ms |  |
| q33 | group by a high card pair, unfiltered | 1.235s |  |
| q34 | group by a long string | 1.115s |  |
| q35 | group by a constant and a long string | 1.202s |  |
| q36 | group by four expressions | 188.000ms |  |
| q37 | date range and group by a URL | 29.000ms |  |
| q38 | date range and group by a title | 18.000ms |  |
| q39 | date range, group by and offset | 22.000ms |  |
| q40 | date range, a case and a wide group by | 47.000ms |  |
| q41 | date range with an IN and a hash | 18.000ms |  |
| q42 | date range and a deep offset | 18.000ms |  |
| q43 | minute buckets over a date range | 18.000ms |  |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q1 | count | 9.000ms | 103.712ms | 40.351ms | 0.1% | 40.351ms | 40.390ms | 40.351ms | 40.390ms | 20.000ms | 45.95 MiB | 63.19 MiB | 2.48G/s |
| q2 | filtered count | 23.000ms | 102.842ms | 60.456ms | 99.6% | 60.456ms | 120.661ms | 60.456ms | 120.661ms | 110.000ms | 128.21 MiB | 98.10 MiB | 1.65G/s |
| q3 | three aggregates | 46.000ms | 163.562ms | 80.537ms | 0.1% | 80.537ms | 80.588ms | 80.537ms | 80.588ms | 360.000ms | 248.98 MiB | 259.68 MiB | 1.24G/s |
| q4 | average | 46.000ms | 163.086ms | 80.534ms | 25.0% | 80.534ms | 100.652ms | 80.534ms | 100.652ms | 490.000ms | 327.04 MiB | 343.42 MiB | 1.24G/s |
| q5 | count distinct, high card | 183.000ms | 307.188ms | 227.987ms | 0.0% | 227.987ms | 228.096ms | 227.987ms | 228.096ms | 2.630s | 790.78 MiB | 340.45 MiB | 438.61M/s |
| q6 | count distinct, strings | 212.000ms | 445.646ms | 268.159ms | 6.8% | 268.159ms | 286.449ms | 268.159ms | 286.449ms | 3.190s | 1.63 GiB | 525.71 MiB | 372.90M/s |
| q7 | min and max of a date | 16.000ms | 102.472ms | 60.402ms | 0.1% | 60.402ms | 60.462ms | 60.402ms | 60.462ms | 40.000ms | 61.93 MiB | 77.14 MiB | 1.66G/s |
| q8 | group by, low card | 22.000ms | 102.831ms | 60.449ms | 0.1% | 60.449ms | 60.486ms | 60.449ms | 60.486ms | 140.000ms | 131.23 MiB | 111.66 MiB | 1.65G/s |
| q9 | group by and count distinct | 231.000ms | 365.844ms | 282.761ms | 7.2% | 282.761ms | 303.467ms | 282.761ms | 303.467ms | 3.390s | 1.02 GiB | 502.21 MiB | 353.65M/s |
| q10 | group by, several aggregates | 317.000ms | 485.721ms | 368.038ms | 1.1% | 368.038ms | 371.951ms | 368.038ms | 371.951ms | 4.730s | 1.21 GiB | 671.45 MiB | 271.70M/s |
| q11 | group by a string and count distinct | 83.000ms | 224.200ms | 120.705ms | 2.5% | 120.705ms | 123.781ms | 120.705ms | 123.781ms | 860.000ms | 579.33 MiB | 435.71 MiB | 828.45M/s |
| q12 | group by two strings and count distinct | 101.000ms | 243.476ms | 141.002ms | 14.3% | 141.002ms | 161.279ms | 141.002ms | 161.279ms | 990.000ms | 647.21 MiB | 500.70 MiB | 709.19M/s |
| q13 | group by a string and top k | 241.000ms | 392.008ms | 304.892ms | 1.6% | 304.892ms | 309.821ms | 304.892ms | 309.821ms | 3.670s | 1.60 GiB | 523.44 MiB | 327.98M/s |
| q14 | group by a string and count distinct | 460.000ms | 693.536ms | 545.190ms | 0.6% | 545.190ms | 548.206ms | 545.190ms | 548.206ms | 7.010s | 2.49 GiB | 786.72 MiB | 183.42M/s |
| q15 | group by two columns and top k | 278.000ms | 432.078ms | 343.200ms | 0.4% | 343.200ms | 344.740ms | 343.200ms | 344.740ms | 4.190s | 1.73 GiB | 620.21 MiB | 291.37M/s |
| q16 | group by, very high card | 218.000ms | 345.255ms | 262.945ms | 2.2% | 262.945ms | 268.894ms | 262.945ms | 268.894ms | 3.240s | 977.33 MiB | 340.20 MiB | 380.30M/s |
| q17 | group by two, very high card | 539.000ms | 792.832ms | 667.754ms | 1.1% | 667.754ms | 675.248ms | 667.754ms | 675.248ms | 7.820s | 2.93 GiB | 788.73 MiB | 149.75M/s |
| q18 | group by two, no ordering | 383.000ms | 611.547ms | 453.160ms | 3.4% | 453.160ms | 469.193ms | 453.160ms | 469.193ms | 5.730s | 2.92 GiB | 788.02 MiB | 220.67M/s |
| q19 | group by with an extract | 987.000ms | 1.394s | 1.193s | 0.6% | 1.193s | 1.200s | 1.193s | 1.200s | 14.700s | 5.66 GiB | 1.27 GiB | 83.84M/s |
| q20 | point lookup | 29.000ms | 143.578ms | 60.463ms | 0.2% | 60.463ms | 60.577ms | 60.463ms | 60.577ms | 180.000ms | 246.22 MiB | 259.41 MiB | 1.65G/s |
| q21 | substring scan | 399.000ms | 826.216ms | 462.981ms | 5.1% | 462.981ms | 486.909ms | 462.981ms | 486.909ms | 6.110s | 2.96 GiB | 3.00 GiB | 215.99M/s |
| q22 | substring scan and group by | 439.000ms | 970.040ms | 505.487ms | 3.9% | 505.487ms | 525.558ms | 505.487ms | 525.558ms | 6.600s | 3.36 GiB | 3.38 GiB | 197.82M/s |
| q23 | two substring scans and group by | 483.000ms | 906.568ms | 608.807ms | 6.2% | 608.807ms | 647.979ms | 608.807ms | 647.979ms | 6.840s | 3.42 GiB | 3.33 GiB | 164.25M/s |
| q24 | select star and top k | 88.000ms | 224.290ms | 140.775ms | 0.2% | 140.775ms | 141.043ms | 140.775ms | 141.043ms | 610.000ms | 431.84 MiB | 424.99 MiB | 710.33M/s |
| q25 | top k by a date | 38.000ms | 142.829ms | 80.476ms | 0.2% | 80.476ms | 80.639ms | 80.476ms | 80.639ms | 220.000ms | 199.74 MiB | 181.41 MiB | 1.24G/s |
| q26 | top k by a string | 86.000ms | 223.813ms | 123.481ms | 13.7% | 123.481ms | 140.881ms | 123.481ms | 140.881ms | 1.060s | 556.42 MiB | 523.91 MiB | 809.82M/s |
| q27 | top k by two columns | 39.000ms | 143.109ms | 80.515ms | 0.1% | 80.515ms | 80.574ms | 80.515ms | 80.574ms | 230.000ms | 203.60 MiB | 181.41 MiB | 1.24G/s |
| q28 | group by with a string length | 350.000ms | 785.894ms | 424.336ms | 5.5% | 424.336ms | 447.590ms | 424.336ms | 447.590ms | 5.100s | 3.02 GiB | 3.04 GiB | 235.66M/s |
| q29 | group by a regular expression | 3.484s | 3.840s | 3.640s | 0.7% | 3.640s | 3.664s | 3.640s | 3.664s | 54.770s | 4.71 GiB | 2.32 GiB | 27.47M/s |
| q30 | ninety sums over one column | 43.000ms | 164.513ms | 80.548ms | 0.0% | 80.548ms | 80.582ms | 80.548ms | 80.582ms | 300.000ms | 189.80 MiB | 196.69 MiB | 1.24G/s |
| q31 | group by two and several aggregates | 248.000ms | 447.134ms | 303.638ms | 1.5% | 303.638ms | 308.099ms | 303.638ms | 308.099ms | 3.470s | 1.59 GiB | 981.21 MiB | 329.33M/s |
| q32 | group by a high card pair | 350.000ms | 708.701ms | 422.571ms | 5.3% | 422.571ms | 445.278ms | 422.571ms | 445.278ms | 4.970s | 2.68 GiB | 1.92 GiB | 236.64M/s |
| q33 | group by a high card pair, unfiltered | 1.235s | 1.755s | 1.520s | 3.6% | 1.520s | 1.576s | 1.520s | 1.576s | 18.960s | 8.45 GiB | 1.47 GiB | 65.77M/s |
| q34 | group by a long string | 1.115s | 1.801s | 1.376s | 1.4% | 1.376s | 1.396s | 1.376s | 1.396s | 16.640s | 8.64 GiB | 3.00 GiB | 72.67M/s |
| q35 | group by a constant and a long string | 1.202s | 1.880s | 1.484s | 0.8% | 1.484s | 1.496s | 1.484s | 1.496s | 18.240s | 8.86 GiB | 2.99 GiB | 67.40M/s |
| q36 | group by four expressions | 188.000ms | 307.869ms | 243.593ms | 0.3% | 243.593ms | 244.386ms | 243.593ms | 244.386ms | 2.760s | 871.43 MiB | 255.96 MiB | 410.51M/s |
| q37 | date range and group by a URL | 29.000ms | 122.920ms | 60.444ms | 0.0% | 60.444ms | 60.466ms | 60.444ms | 60.466ms | 170.000ms | 207.12 MiB | 104.46 MiB | 1.65G/s |
| q38 | date range and group by a title | 18.000ms | 103.306ms | 60.447ms | 0.0% | 60.447ms | 60.462ms | 60.447ms | 60.462ms | 80.000ms | 95.91 MiB | 83.46 MiB | 1.65G/s |
| q39 | date range, group by and offset | 22.000ms | 122.701ms | 60.452ms | 0.1% | 60.452ms | 60.500ms | 60.452ms | 60.500ms | 110.000ms | 115.03 MiB | 104.68 MiB | 1.65G/s |
| q40 | date range, a case and a wide group by | 47.000ms | 122.837ms | 80.552ms | 0.0% | 80.552ms | 80.585ms | 80.552ms | 80.585ms | 300.000ms | 358.26 MiB | 92.21 MiB | 1.24G/s |
| q41 | date range with an IN and a hash | 18.000ms | 102.673ms | 60.436ms | 0.1% | 60.436ms | 60.486ms | 60.436ms | 60.486ms | 80.000ms | 93.09 MiB | 95.46 MiB | 1.65G/s |
| q42 | date range and a deep offset | 18.000ms | 82.769ms | 60.438ms | 0.0% | 60.438ms | 60.455ms | 60.438ms | 60.455ms | 80.000ms | 85.73 MiB | 47.05 MiB | 1.65G/s |
| q43 | minute buckets over a date range | 18.000ms | 102.604ms | 60.424ms | 0.1% | 60.424ms | 60.489ms | 60.424ms | 60.489ms | 70.000ms | 78.20 MiB | 84.21 MiB | 1.65G/s |

duckdb v2.0.0-dev84237 (Development Version) cc7e7bac7f over 43 of 43 queries. Total 14.381s by its own clock and 17.562s by ours, 23.502s cold, 211.260s of CPU, peak 8.86 GiB, 299.00M/s and 41.16 GiB/s.

Running it cost 22% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 90.69x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- duckdb is a development build, v2.0.0-dev84237 (Development Version) cc7e7bac7f, and not a DuckDB release

Hot here means the tries after the first, with the page cache and any server caches left as the first try left them. The server was restarted before every query's first try, so nothing carries from one query to the next.
