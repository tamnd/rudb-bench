# tpch on gamingpc-wsl

This is one run of the tpch suite on gamingpc-wsl, over 4 engines and 22 queries, with 5 hot runs of each query after one cold one. It was written by `rudb-bench run --report` and nothing in it was typed by hand. The command that reproduces it is below.

## What ran

| what | it was |
| --- | --- |
| suite | tpch |
| queries | 22 |
| tables | 33.20 GiB of Parquet in 8 tables |
| rows | the suite does not declare one |
| summary | median with the interquartile range, per reporting rule two |
| harness | rudb-bench 0.0.1 |

## How to reproduce it

Reporting rule eight says a published number is reproducible by one documented command on a named machine type. This is the command.

```
rudb-bench run tpch --engines duckdb,duckdb-pinned,clickhouse-local,datafusion --runs 5 --report
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
| page cache | cannot open drop_caches: Permission denied (os error 13) | not read |
| timer | /usr/bin/time GNU | read |

A fact that says it was not read is a fact about this machine and not a gap in the report. A field that had silently defaulted would be a lie that survived into it.

## The engines

| engine | version | state | load | load cpu | on disk | that size is | format | machine load |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | v1.5.5 (Variegata) d8cdaa33fd | ran | 49.976s | 1144.180s | 26.24 GiB | its own database file | its own | 1.46 to 31.40 |
| duckdb-pinned | v2.0.0-dev84237 (Development Version) cc7e7bac7f | ran | 56.701s | 1350.500s | 27.96 GiB | its own database file | its own | 31.40 to 26.88 |
| clickhouse-local | 26.9.1.1562 | ran | 84.779s | 1291.550s | 30.23 GiB | its own MergeTree parts, as system.parts counts the active ones | its own | 26.88 to 20.85 |
| datafusion | datafusion-cli 55.1.0 | ran | 0.000us | 0.000us | 33.20 GiB | the source Parquet, this engine has no storage format of its own | the Parquet | 20.85 to 26.92 |
| polars | 1.44.2 in sink mode | polars failed: /home/gopher/rudb-data/scratch/rudb-bench-2912067/polars-run.py:18: DeprecationWarning: Casting from String to Date is deprecated and will be removed in Polars 2.0. Use `str.to_date()` instead.   ctx.execute(sql).sink_csv( | n/a | n/a | n/a | n/a | n/a | n/a |
| rudb | rudb 0.3.45 | every join in rudb is a nested loop and TPC-H is twenty two of them, so this would be a timing of a hang. spec/07-execution.md section 7.4, milestone E3 | n/a | n/a | n/a | n/a | n/a | n/a |
| clickhouse-server | not asked for | left out of this run by --engines | n/a | n/a | n/a | n/a | n/a | n/a |

Reading the Parquet where it lies rather than a format of its own: datafusion. That is an empty load column and a decode inside every query, in the column being compared, which the other engines paid for once at load time.

The machine load column is the one minute load average before and after that engine's suite, on a machine with 32 hardware threads. Only the first of the two says how busy the machine was with work that was not this run's. The second one counts the engine's own threads, and an engine that uses every core is supposed to use every core, so a high number there is the measurement rather than a problem with it.

These engines started while the machine was already above half of that, which means they were measured against somebody else's work rather than on an idle box: duckdb-pinned, clickhouse-local, datafusion. Their numbers are inflated by an amount nothing here can recover, and by a different amount each, depending on how much of the column was CPU bound. One case where that reading is unfair is a sweep that runs engines back to back, where the average has not had a minute to come down from the engine before.

The cold column is the first run of each query rather than a run that had to go to the device. The page cache was not dropped, so the file was still in memory from whatever read it last. That makes cold a warm number taken before the others rather than a measure of what a first pass off the disk costs, and the gap between the two is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the other one.

## Totals

| engine | query time | wall time | overhead | cold total | hot cpu | cores | peak RSS | hot read | rows/s | bytes/s | vs duckdb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb | 27.416s | 29.975s | +9% | 32.679s | 538.140s | 17.95 | 16.65 GiB | 434.35 MiB | no row count | 26.64 GiB/s | 1.00x |
| duckdb-pinned | 25.676s | 30.436s | +19% | 32.721s | 414.130s | 13.61 | 16.82 GiB | 853.99 MiB | no row count | 28.45 GiB/s | 0.94x |
| clickhouse-local | 123.414s | 129.134s | +5% | 142.254s | 2158.370s | 16.71 | 16.99 GiB | 9.83 GiB | no row count | 5.92 GiB/s | 4.50x |
| datafusion | 103.139s | 105.621s | +2% | 145.738s | 2281.770s | 21.60 | 29.22 GiB | 12.04 GiB | no row count | 7.08 GiB/s | 3.76x |

Query time is what the engine itself says the queries took, added up over the hot runs. Every engine here was asked, each in its own way, and it is the same number the public ClickBench board publishes. Wall time is the clock this harness holds around the whole subprocess, so it also pays for starting a process, linking it, opening a database and printing the answer. Overhead is the difference as a fraction of the query time, and it is the number that says how much of the wall clock column is this harness rather than the engine.

Read the query time column when comparing engines and the wall time column when asking what the run costs to sit through. They rank engines differently and that is the point: the overhead is not the same for each engine, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself. On a small sample the wall clock column partly ranks process startup. The ratio and both throughput columns are taken against query time wherever every engine reported one.

The two throughput columns are the whole table read once per query, so the row count and the file size times the 22 queries over the total. It is a rate for the run and not a rate any one query reached, and it is not comparable to the same number from a suite with a different number of queries. Cores is CPU seconds over wall seconds, which is how many of this machine's threads the engine actually kept busy, and it is the number that says whether two wall clocks on two machines can be compared at all.

Hot read is what the block layer served during the hot runs. It should be nothing on a machine with room for the data, and when it is not, the hot number is not a hot number.

## Time per query

| query | shape | duckdb | duckdb-pinned | clickhouse-local | datafusion |
| --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 813.000ms | 1.064s | 3.777s | 3.577s |
| q02 | minimum cost supplier | 279.000ms | 200.000ms | 27.637s | 1.005s |
| q03 | shipping priority | 999.000ms | 1.148s | 2.612s | 2.315s |
| q04 | order priority checking | 979.000ms | 1.110s | 2.026s | 1.053s |
| q05 | local supplier volume | 1.095s | 1.114s | 2.290s | 4.176s |
| q06 | forecasting revenue change | 762.000ms | 870.000ms | 1.371s | 850.000ms |
| q07 | volume shipping | 1.373s | 1.307s | 2.545s | 7.875s |
| q08 | national market share | 1.458s | 1.440s | 2.958s | 4.034s |
| q09 | product type profit measure | 2.970s | 2.833s | 34.418s | 8.498s |
| q10 | returned item reporting | 1.577s | 1.512s | 3.042s | 3.516s |
| q11 | important stock identification | 199.000ms | 201.000ms | 421.000ms | 928.000ms |
| q12 | shipping modes and order priority | 1.596s | 1.270s | 1.916s | 1.263s |
| q13 | customer distribution | 2.204s | 1.574s | 3.520s | 2.712s |
| q14 | promotion effect | 1.050s | 1.143s | 1.292s | 1.369s |
| q15 | top supplier | 950.000ms | 921.000ms | 2.823s | 2.492s |
| q16 | parts supplier relationship | 482.000ms | 512.000ms | 508.000ms | 2.301s |
| q17 | small quantity order revenue | 1.128s | 1.010s | 12.643s | 8.193s |
| q18 | large volume customer | 2.074s | 2.068s | 4.762s | 21.463s |
| q19 | discounted revenue | 1.459s | 1.304s | 2.244s | 2.132s |
| q20 | potential part promotion | 1.115s | 1.065s | 1.536s | 2.623s |
| q21 | suppliers who kept orders waiting | 2.447s | 1.683s | 8.450s | 20.423s |
| q22 | global sales opportunity | 407.000ms | 327.000ms | 623.000ms | 341.000ms |

The engine's own hot figure for each query, which is the one that compares across columns. The wall clock, the spread and everything else are in the per engine tables below, one of which is the whole distribution for every query.

## duckdb in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 813.000ms | 1.309s | 915.858ms | 3.5% | 910.612ms | 942.509ms | 902.839ms | 953.685ms | 18.130s | 5.07 GiB | 5.04 GiB | no row count |
| q02 | minimum cost supplier | 279.000ms | 280.913ms | 309.771ms | 4.0% | 305.666ms | 318.196ms | 299.403ms | 320.014ms | 2.450s | 1020.72 MiB | 444.09 MiB | no row count |
| q03 | shipping priority | 999.000ms | 1.331s | 1.121s | 4.5% | 1.107s | 1.157s | 1.103s | 1.162s | 23.380s | 6.84 GiB | 2.55 GiB | no row count |
| q04 | order priority checking | 979.000ms | 1.321s | 1.064s | 2.4% | 1.051s | 1.076s | 1.049s | 1.102s | 16.850s | 5.11 GiB | 4.34 GiB | no row count |
| q05 | local supplier volume | 1.095s | 1.433s | 1.227s | 4.1% | 1.219s | 1.269s | 1.165s | 1.279s | 25.380s | 7.61 GiB | 3.14 GiB | no row count |
| q06 | forecasting revenue change | 762.000ms | 910.862ms | 848.283ms | 8.8% | 823.153ms | 897.981ms | 765.651ms | 949.276ms | 7.410s | 5.02 GiB | none | no row count |
| q07 | volume shipping | 1.373s | 1.620s | 1.510s | 3.6% | 1.503s | 1.557s | 1.473s | 1.625s | 25.620s | 9.23 GiB | none | no row count |
| q08 | national market share | 1.458s | 1.588s | 1.611s | 0.4% | 1.608s | 1.614s | 1.543s | 1.666s | 21.180s | 8.85 GiB | 1.36 GiB | no row count |
| q09 | product type profit measure | 2.970s | 4.050s | 3.176s | 11.0% | 3.082s | 3.433s | 2.962s | 4.181s | 71.420s | 16.65 GiB | 273.24 MiB | no row count |
| q10 | returned item reporting | 1.577s | 1.906s | 1.783s | 2.9% | 1.773s | 1.825s | 1.751s | 1.830s | 35.170s | 9.96 GiB | 1.88 GiB | no row count |
| q11 | important stock identification | 199.000ms | 271.617ms | 228.937ms | 1.7% | 228.505ms | 232.410ms | 226.629ms | 241.633ms | 2.260s | 1.15 GiB | 202.78 MiB | no row count |
| q12 | shipping modes and order priority | 1.596s | 1.872s | 1.699s | 2.0% | 1.670s | 1.704s | 1.549s | 1.706s | 14.230s | 5.71 GiB | 5.42 GiB | no row count |
| q13 | customer distribution | 2.204s | 2.369s | 2.342s | 2.9% | 2.338s | 2.407s | 2.186s | 2.420s | 59.980s | 7.84 GiB | 2.70 GiB | no row count |
| q14 | promotion effect | 1.050s | 1.223s | 1.167s | 3.2% | 1.154s | 1.192s | 1.108s | 1.238s | 15.890s | 7.86 GiB | 71.02 MiB | no row count |
| q15 | top supplier | 950.000ms | 1.095s | 1.061s | 1.3% | 1.055s | 1.069s | 1.044s | 1.093s | 13.790s | 7.42 GiB | 1.75 MiB | no row count |
| q16 | parts supplier relationship | 482.000ms | 583.703ms | 537.093ms | 2.4% | 533.799ms | 546.508ms | 530.949ms | 546.755ms | 9.790s | 2.25 GiB | 70.95 MiB | no row count |
| q17 | small quantity order revenue | 1.128s | 1.084s | 1.230s | 1.9% | 1.207s | 1.231s | 1.198s | 1.279s | 16.270s | 6.32 GiB | 3.50 MiB | no row count |
| q18 | large volume customer | 2.074s | 2.318s | 2.255s | 0.6% | 2.245s | 2.259s | 2.237s | 2.279s | 52.030s | 9.31 GiB | 437.28 MiB | no row count |
| q19 | discounted revenue | 1.459s | 1.696s | 1.618s | 4.9% | 1.584s | 1.663s | 1.551s | 1.687s | 24.990s | 10.10 GiB | 255.43 MiB | no row count |
| q20 | potential part promotion | 1.115s | 1.330s | 1.235s | 7.3% | 1.221s | 1.311s | 1.195s | 1.313s | 15.820s | 7.23 GiB | 1.58 GiB | no row count |
| q21 | suppliers who kept orders waiting | 2.447s | 2.655s | 2.598s | 3.4% | 2.556s | 2.643s | 2.532s | 2.644s | 56.070s | 9.17 GiB | 4.36 MiB | no row count |
| q22 | global sales opportunity | 407.000ms | 430.910ms | 438.046ms | 1.6% | 437.973ms | 445.054ms | 416.761ms | 458.117ms | 10.030s | 1.09 GiB | 5.78 MiB | no row count |

duckdb v1.5.5 (Variegata) d8cdaa33fd over 22 of 22 queries. Total 27.416s by its own clock and 29.975s by ours, 32.679s cold, 538.140s of CPU, peak 16.65 GiB, no row count and 26.64 GiB/s.

Running it cost 9% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 13.87x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## duckdb-pinned in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 1.064s | 1.324s | 1.275s | 0.2% | 1.273s | 1.275s | 1.226s | 1.303s | 13.070s | 5.01 GiB | 4.96 GiB | no row count |
| q02 | minimum cost supplier | 200.000ms | 305.003ms | 244.786ms | 7.7% | 235.388ms | 254.177ms | 232.544ms | 262.619ms | 1.200s | 958.81 MiB | 498.92 MiB | no row count |
| q03 | shipping priority | 1.148s | 1.367s | 1.382s | 2.7% | 1.362s | 1.400s | 1.354s | 1.405s | 19.770s | 6.87 GiB | 2.61 GiB | no row count |
| q04 | order priority checking | 1.110s | 1.311s | 1.288s | 0.4% | 1.285s | 1.290s | 1.238s | 1.311s | 14.210s | 4.76 GiB | 4.31 GiB | no row count |
| q05 | local supplier volume | 1.114s | 1.570s | 1.372s | 2.0% | 1.361s | 1.388s | 1.352s | 1.412s | 21.180s | 7.80 GiB | 3.33 GiB | no row count |
| q06 | forecasting revenue change | 870.000ms | 1.083s | 1.052s | 1.0% | 1.051s | 1.062s | 1.034s | 1.070s | 6.060s | 4.97 GiB | none | no row count |
| q07 | volume shipping | 1.307s | 1.665s | 1.586s | 6.3% | 1.534s | 1.634s | 1.445s | 1.694s | 15.880s | 9.47 GiB | none | no row count |
| q08 | national market share | 1.440s | 1.769s | 1.709s | 3.5% | 1.682s | 1.742s | 1.659s | 1.745s | 14.390s | 9.03 GiB | 1.48 GiB | no row count |
| q09 | product type profit measure | 2.833s | 4.646s | 3.225s | 7.6% | 3.204s | 3.449s | 3.131s | 4.890s | 57.860s | 16.82 GiB | 241.14 MiB | no row count |
| q10 | returned item reporting | 1.512s | 1.903s | 1.852s | 2.6% | 1.821s | 1.870s | 1.806s | 1.880s | 33.800s | 10.00 GiB | 1.62 GiB | no row count |
| q11 | important stock identification | 201.000ms | 252.856ms | 239.634ms | 5.3% | 228.994ms | 241.762ms | 222.250ms | 254.988ms | 2.330s | 1.04 GiB | 205.30 MiB | no row count |
| q12 | shipping modes and order priority | 1.270s | 1.522s | 1.474s | 0.6% | 1.470s | 1.478s | 1.466s | 1.487s | 11.420s | 5.76 GiB | 3.95 GiB | no row count |
| q13 | customer distribution | 1.574s | 2.002s | 1.855s | 4.1% | 1.805s | 1.880s | 1.764s | 1.896s | 44.130s | 7.86 GiB | 3.10 GiB | no row count |
| q14 | promotion effect | 1.143s | 1.358s | 1.361s | 1.9% | 1.341s | 1.366s | 1.331s | 1.380s | 11.520s | 7.87 GiB | 46.15 MiB | no row count |
| q15 | top supplier | 921.000ms | 1.221s | 1.141s | 2.8% | 1.112s | 1.144s | 1.098s | 1.166s | 11.340s | 7.39 GiB | 132.09 MiB | no row count |
| q16 | parts supplier relationship | 512.000ms | 656.230ms | 613.838ms | 1.2% | 610.154ms | 617.357ms | 594.437ms | 621.243ms | 7.940s | 2.24 GiB | 63.45 MiB | no row count |
| q17 | small quantity order revenue | 1.010s | 1.182s | 1.196s | 5.5% | 1.164s | 1.230s | 1.127s | 1.235s | 12.790s | 6.39 GiB | 3.82 MiB | no row count |
| q18 | large volume customer | 2.068s | 2.433s | 2.439s | 1.1% | 2.423s | 2.450s | 2.373s | 2.519s | 49.750s | 10.54 GiB | 470.48 MiB | no row count |
| q19 | discounted revenue | 1.304s | 1.578s | 1.558s | 0.7% | 1.555s | 1.566s | 1.542s | 1.584s | 14.580s | 8.76 GiB | 701.49 MiB | no row count |
| q20 | potential part promotion | 1.065s | 1.383s | 1.286s | 1.7% | 1.281s | 1.303s | 1.238s | 1.304s | 13.750s | 7.18 GiB | 243.80 MiB | no row count |
| q21 | suppliers who kept orders waiting | 1.683s | 1.832s | 1.916s | 5.1% | 1.862s | 1.960s | 1.852s | 2.082s | 30.290s | 8.00 GiB | 31.08 MiB | no row count |
| q22 | global sales opportunity | 327.000ms | 357.759ms | 369.991ms | 1.6% | 365.145ms | 371.220ms | 356.150ms | 389.601ms | 6.870s | 1.85 GiB | 760.00 KiB | no row count |

duckdb-pinned v2.0.0-dev84237 (Development Version) cc7e7bac7f over 22 of 22 queries. Total 25.676s by its own clock and 30.436s by ours, 32.721s cold, 414.130s of CPU, peak 16.82 GiB, no row count and 28.45 GiB/s.

Running it cost 19% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 13.46x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## clickhouse-local in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 3.777s | 4.490s | 3.987s | 1.0% | 3.983s | 4.024s | 3.976s | 4.051s | 115.610s | 1.99 GiB | 4.31 GiB | no row count |
| q02 | minimum cost supplier | 27.637s | 32.388s | 29.260s | 6.2% | 28.472s | 30.297s | 26.899s | 40.230s | 253.840s | 14.38 GiB | 12.03 GiB | no row count |
| q03 | shipping priority | 2.612s | 3.066s | 2.823s | 3.2% | 2.816s | 2.906s | 2.801s | 3.001s | 74.880s | 2.53 GiB | 3.29 GiB | no row count |
| q04 | order priority checking | 2.026s | 2.231s | 2.203s | 1.4% | 2.181s | 2.212s | 2.076s | 2.236s | 55.280s | 1.55 GiB | 2.16 GiB | no row count |
| q05 | local supplier volume | 2.290s | 2.956s | 2.475s | 1.5% | 2.453s | 2.491s | 2.410s | 2.608s | 65.820s | 1.79 GiB | 1.12 GiB | no row count |
| q06 | forecasting revenue change | 1.371s | 1.486s | 1.546s | 3.8% | 1.510s | 1.568s | 1.490s | 1.592s | 39.010s | 604.27 MiB | 728.04 MiB | no row count |
| q07 | volume shipping | 2.545s | 2.713s | 2.756s | 9.3% | 2.717s | 2.972s | 2.638s | 3.151s | 71.600s | 1.96 GiB | 30.75 MiB | no row count |
| q08 | national market share | 2.958s | 3.321s | 3.176s | 0.7% | 3.156s | 3.178s | 3.147s | 3.205s | 86.270s | 3.43 GiB | 1.52 GiB | no row count |
| q09 | product type profit measure | 34.418s | 41.404s | 34.760s | 1.5% | 34.625s | 35.164s | 34.083s | 36.655s | 265.790s | 16.99 GiB | 6.47 GiB | no row count |
| q10 | returned item reporting | 3.042s | 3.277s | 3.296s | 1.6% | 3.293s | 3.346s | 3.281s | 3.359s | 83.600s | 5.25 GiB | 1.33 GiB | no row count |
| q11 | important stock identification | 421.000ms | 609.760ms | 542.620ms | 3.0% | 530.300ms | 546.453ms | 530.067ms | 564.764ms | 9.820s | 702.37 MiB | 184.94 MiB | no row count |
| q12 | shipping modes and order priority | 1.916s | 2.194s | 2.058s | 0.7% | 2.052s | 2.065s | 2.033s | 2.218s | 52.080s | 1.17 GiB | 3.70 GiB | no row count |
| q13 | customer distribution | 3.520s | 4.057s | 3.803s | 1.0% | 3.796s | 3.836s | 3.767s | 4.006s | 98.100s | 12.15 GiB | 1.75 GiB | no row count |
| q14 | promotion effect | 1.292s | 1.486s | 1.433s | 1.1% | 1.422s | 1.438s | 1.413s | 1.483s | 34.540s | 1.31 GiB | 73.98 MiB | no row count |
| q15 | top supplier | 2.823s | 2.961s | 2.999s | 1.1% | 2.991s | 3.024s | 2.964s | 3.076s | 76.060s | 3.20 GiB | 78.51 MiB | no row count |
| q16 | parts supplier relationship | 508.000ms | 659.725ms | 650.398ms | 1.0% | 645.581ms | 652.308ms | 645.065ms | 652.584ms | 12.130s | 1.54 GiB | 75.40 MiB | no row count |
| q17 | small quantity order revenue | 12.643s | 13.783s | 12.834s | 1.2% | 12.743s | 12.894s | 12.739s | 12.943s | 340.110s | 15.59 GiB | 3.62 GiB | no row count |
| q18 | large volume customer | 4.762s | 5.179s | 5.066s | 1.5% | 5.038s | 5.112s | 5.035s | 5.184s | 133.200s | 13.49 GiB | 1.27 GiB | no row count |
| q19 | discounted revenue | 2.244s | 2.453s | 2.362s | 1.1% | 2.359s | 2.386s | 2.350s | 2.392s | 62.210s | 518.64 MiB | 1.13 GiB | no row count |
| q20 | potential part promotion | 1.536s | 1.760s | 1.668s | 2.5% | 1.634s | 1.675s | 1.627s | 1.682s | 40.480s | 954.61 MiB | 2.02 GiB | no row count |
| q21 | suppliers who kept orders waiting | 8.450s | 9.018s | 8.693s | 0.5% | 8.658s | 8.698s | 8.626s | 8.739s | 176.680s | 7.33 GiB | 1.00 GiB | no row count |
| q22 | global sales opportunity | 623.000ms | 760.094ms | 742.223ms | 1.9% | 740.533ms | 754.920ms | 737.432ms | 772.786ms | 11.260s | 701.55 MiB | 68.09 MiB | no row count |

clickhouse-local 26.9.1.1562 over 22 of 22 queries. Total 123.414s by its own clock and 129.134s by ours, 142.254s cold, 2158.370s of CPU, peak 16.99 GiB, no row count and 5.92 GiB/s.

Running it cost 5% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 64.06x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## datafusion in full

| query | shape | query time | cold | hot | IQR | p25 | p75 | fastest | slowest | hot cpu | peak RSS | cold read | rows/s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| q01 | pricing summary report | 3.577s | 3.811s | 3.632s | 2.6% | 3.601s | 3.695s | 3.600s | 3.804s | 101.940s | 747.30 MiB | 5.14 GiB | no row count |
| q02 | minimum cost supplier | 1.005s | 1.210s | 1.066s | 0.5% | 1.065s | 1.071s | 1.061s | 1.076s | 25.480s | 2.35 GiB | 1.13 GiB | no row count |
| q03 | shipping priority | 2.315s | 2.588s | 2.374s | 0.9% | 2.372s | 2.392s | 2.371s | 2.393s | 61.000s | 2.14 GiB | 3.31 GiB | no row count |
| q04 | order priority checking | 1.053s | 1.789s | 1.127s | 0.8% | 1.127s | 1.136s | 1.121s | 1.206s | 26.730s | 6.94 GiB | 9.19 GiB | no row count |
| q05 | local supplier volume | 4.176s | 4.719s | 4.239s | 0.9% | 4.211s | 4.248s | 4.205s | 4.248s | 109.960s | 2.91 GiB | 5.93 GiB | no row count |
| q06 | forecasting revenue change | 850.000ms | 910.814ms | 904.640ms | 0.7% | 900.246ms | 906.737ms | 894.224ms | 911.738ms | 21.500s | 741.33 MiB | none | no row count |
| q07 | volume shipping | 7.875s | 8.439s | 8.004s | 1.3% | 8.003s | 8.106s | 7.997s | 8.383s | 222.540s | 11.28 GiB | 1.83 MiB | no row count |
| q08 | national market share | 4.034s | 4.137s | 4.104s | 0.9% | 4.102s | 4.137s | 4.096s | 4.177s | 102.560s | 4.93 GiB | 13.66 MiB | no row count |
| q09 | product type profit measure | 8.498s | 9.721s | 8.722s | 5.2% | 8.670s | 9.120s | 8.614s | 10.474s | 214.720s | 14.42 GiB | 3.43 GiB | no row count |
| q10 | returned item reporting | 3.516s | 3.901s | 3.620s | 2.1% | 3.579s | 3.656s | 3.544s | 3.695s | 92.410s | 6.71 GiB | 9.59 GiB | no row count |
| q11 | important stock identification | 928.000ms | 1.001s | 982.122ms | 0.2% | 981.705ms | 983.903ms | 974.837ms | 986.721ms | 24.110s | 1.30 GiB | 122.95 MiB | no row count |
| q12 | shipping modes and order priority | 1.263s | 1.550s | 1.324s | 0.7% | 1.318s | 1.328s | 1.316s | 1.515s | 32.570s | 1.70 GiB | 5.16 GiB | no row count |
| q13 | customer distribution | 2.712s | 2.904s | 2.775s | 2.6% | 2.766s | 2.838s | 2.761s | 2.851s | 79.240s | 2.81 GiB | 1.59 GiB | no row count |
| q14 | promotion effect | 1.369s | 1.478s | 1.425s | 1.1% | 1.410s | 1.425s | 1.409s | 1.430s | 35.180s | 1.99 GiB | 973.81 MiB | no row count |
| q15 | top supplier | 2.492s | 2.573s | 2.548s | 0.5% | 2.548s | 2.560s | 2.530s | 2.612s | 64.480s | 1.40 GiB | 42.99 MiB | no row count |
| q16 | parts supplier relationship | 2.301s | 3.352s | 2.400s | 8.9% | 2.291s | 2.504s | 2.206s | 2.922s | 13.730s | 5.76 GiB | 154.13 MiB | no row count |
| q17 | small quantity order revenue | 8.193s | 8.443s | 8.266s | 0.7% | 8.237s | 8.294s | 8.198s | 8.497s | 231.800s | 3.13 GiB | 3.33 GiB | no row count |
| q18 | large volume customer | 21.463s | 48.061s | 22.180s | 7.1% | 21.178s | 22.758s | 19.818s | 26.079s | 352.360s | 29.22 GiB | 7.77 GiB | no row count |
| q19 | discounted revenue | 2.132s | 2.903s | 2.189s | 1.4% | 2.187s | 2.218s | 2.152s | 2.227s | 55.400s | 1.33 GiB | 15.85 GiB | no row count |
| q20 | potential part promotion | 2.623s | 2.846s | 2.687s | 0.5% | 2.685s | 2.699s | 2.672s | 2.730s | 70.560s | 4.84 GiB | 900.26 MiB | no row count |
| q21 | suppliers who kept orders waiting | 20.423s | 28.963s | 20.657s | 9.2% | 20.253s | 22.143s | 17.212s | 30.001s | 334.970s | 23.61 GiB | 16.01 GiB | no row count |
| q22 | global sales opportunity | 341.000ms | 436.530ms | 396.058ms | 1.5% | 394.738ms | 400.701ms | 392.359ms | 421.802ms | 8.530s | 1.31 GiB | 399.09 MiB | no row count |

datafusion datafusion-cli 55.1.0 over 22 of 22 queries. Total 103.139s by its own clock and 105.621s by ours, 145.738s cold, 2281.770s of CPU, peak 29.22 GiB, no row count and 7.08 GiB/s.

Running it cost 2% on top of the queries themselves. That is process start, linking, opening the data and printing the answer, and it is in every wall clock figure in this section.

Its slowest query is 56.00x its fastest. A column much flatter than that is measuring whatever every query in it has in common rather than measuring the queries.

## What this number is not

This is not a publishable number, because:

- no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
- q09 swung by 11.0% of its median, and rule two wants under 10%

These swung wider than reporting rule two allows:

- duckdb swung by 11.0% of its median on q09, and rule two wants under 10%

That is either another tenant on this machine or a query so short that starting the process is most of what got timed. Either way the ratio column compares two numbers whose error bars are wider than the gap between them.

All 4 engines agreed on every answer the data settles, which is 22 of 22 queries, to the last significant digit of a double.

Hot here means page cache warm and not buffer pool warm, because every run is a fresh process so that no query's number depends on the one before it.

