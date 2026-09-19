# rudb-bench

The benchmark harness for [rudb](https://github.com/tamnd/rudb).

ClickBench, TPC-H, TPC-DS, JOB, CEB and H2O against DuckDB, ClickHouse, Umbra and DataFusion, and the reporting rules that decide what a number is allowed to claim.

It is a separate repository so that a result can be reproduced by someone who does not trust us, without building the engine from a specific commit of the engine's own repository. The whole project's claim is a performance claim, which means its credibility rests on the honesty of these measurements more than on any technical decision inside the engine. A benchmark number without its methodology is marketing.

The design is [`spec/15-rudb-bench.md`](https://github.com/tamnd/rudb/blob/main/spec/15-rudb-bench.md) in the rudb repository.

The [13 September measurement audit](reports/2026-09-13/clickbench-audit.md) runs DuckDB 1.5.5 and rudb 0.2.37 through all 43 ClickBench queries at 1k, 10k, 100k and 1m rows, with five hot repetitions, per-child CPU and RSS checks, and retained raw results. It includes q19 and q33 on the samples. The earlier measurements below retain their original versions and methodology.

rudb now exposes all 43 ClickBench queries through the regular harness. Q19 and Q33 were the last two exclusions. Both complete and match DuckDB at 1k, 10k, 100k, and 1m rows in the four way native and Parquet audit. ClickBench has 43 queries, so the complete result is 43 of 43.

## Where rudb is, today

rudb 0.3.45, six engines, six sizes, on `gamingpc-wsl`, which is 32 hardware threads. Five hot runs of each query after a cold one, all 43 queries, one report per size. Query time is what each engine itself reports, which is the number the public ClickBench board publishes. The ratio is against DuckDB v1.5.5 over the 39 queries every engine in the run answered.

| engine | 1k rows | 10k rows | 100k rows | 1m rows | 10m rows | 100m rows |
| --- | --- | --- | --- | --- | --- | --- |
| duckdb v1.5.5 d8cdaa33fd | 98ms | 120ms | 362ms | 577ms | 3.902s | 24.116s |
| duckdb-pinned v2.0.0-dev84237 cc7e7bac7f | 130ms | 148ms | 345ms | 565ms | 3.364s | 15.416s |
| clickhouse-local 26.9.1.1562 | 364ms | 398ms | 782ms | 3.005s | 8.394s | 16.396s |
| datafusion 55.1.0 | 203ms | 300ms | 422ms | 1.199s | 13.110s | 23.356s |
| polars 1.44.2 | 569.005ms | 585.086ms | 690.885ms | 1.222s | 5.030s | 21.432s |
| rudb 0.3.45 | 33.153ms | 77.588ms | 233.610ms | 927.438ms | 10.263s | 31.361s |
| rudb against duckdb | 0.34x | 0.66x | 0.78x | 1.78x | 3.39x | 1.78x |

The last column is the only one that is not a sample. It is the real `hits`, all 99,997,497 rows and 13.76 GiB of it, which is the file the public board runs, and it is the first size in this ladder that carries no development loop caveat.

rudb is ahead of every engine in the table at a thousand, ten thousand and a hundred thousand rows, 1.78x behind DuckDB at a million, 3.39x behind at ten million, and 1.78x behind on the full file. A ratio that gets worse at every step and then gets better again at the last one is not a shape any per-row cost has, and chasing that turned out to be the most useful thing in the run. It is in [the ten million column was mostly row groups](#the-ten-million-column-was-mostly-row-groups) below, and the short version is that the 10m column is measuring the harness's own sampling rather than rudb at ten million rows.

Read against the file the board actually publishes, then, rudb 0.3.45 is 1.78x DuckDB v1.5.5, 2.03x the pinned DuckDB, 1.91x `clickhouse-local`, and within half of DataFusion and Polars, while holding the smallest memory of the six and the lowest harness overhead of the six. The previous entries in this file, further down, had rudb 16.59x behind DuckDB at a million rows at 0.2.31, and 11.28x at a million and 19.51x at ten million at 0.3.5, doing less work than it does now: 0.3.5 ran 41 of 43 queries and excluded Q19 and Q33, and 0.3.45 runs all 43.

None of which is 10x ahead, which is the goal, and 1.78x behind is the honest distance to it. But the gap at the full file is per-query work in a handful of identifiable shapes rather than a floor under everything, which is the more tractable of the two problems to have.

The cores column is the one that changed most, and it changes what the remaining gap can be blamed on, differently at each end.

| cores used, hot | 1k rows | 10k rows | 100k rows | 1m rows | 10m rows | 100m rows |
| --- | --- | --- | --- | --- | --- | --- |
| duckdb | 0.33 | 0.41 | 0.69 | 2.61 | 8.42 | 11.82 |
| rudb | 0.00 | 0.00 | 0.64 | 2.65 | 4.06 | 11.45 |

At 0.3.5 rudb never got above 1.00 core on a 32 thread box while DuckDB kept 2.25 busy at a million rows and 9.01 at ten million. At 0.3.45 rudb keeps 2.65 busy at a million, marginally more than DuckDB's 2.61, and is still 1.78x behind, so the gap at that size is per-core work rather than a thread count. At ten million the two come apart, DuckDB reaching 8.42 cores against rudb's 4.06, and on the full file they close up again at 11.82 and 11.45. That the two ends of the ladder agree and only the middle disagrees is the second thing pointing at the 10m column rather than at rudb, and the read that survives it is that rudb scales its threads about as far as DuckDB does on this machine and loses on what each thread is doing. The two zeros at 1k and 10k are not rudb using no CPU; they are the process finishing inside the 10ms granularity of the CPU accounting, so there is nothing to divide.

Nobody in the run is close to the 32 threads the box has. DataFusion gets furthest at 22.50 and is still 1.34x DuckDB, so on this suite at this size the thread count is not what separates the engines.

Memory still goes rudb's way at every size, which matters because a time bought with twice the memory is not the same result.

| peak RSS | 1k rows | 10k rows | 100k rows | 1m rows | 10m rows | 100m rows |
| --- | --- | --- | --- | --- | --- | --- |
| duckdb | 38.00 MiB | 40.25 MiB | 65.32 MiB | 307.69 MiB | 1.72 GiB | 10.81 GiB |
| datafusion | 156.72 MiB | 201.09 MiB | 401.69 MiB | 1.16 GiB | 4.55 GiB | 10.90 GiB |
| rudb | 10.24 MiB | 13.24 MiB | 46.48 MiB | 162.24 MiB | 1.16 GiB | 5.69 GiB |

rudb is the smallest engine in the run at every size. On the full file it is 5.69 GiB against DuckDB's 10.81 and DataFusion's 10.90, and against Polars at 16.68 GiB, which is a little over half the memory of the box for a suite rudb ran in under six. It also has the lowest harness overhead of the six at 1m, 10m and 100m, at +75%, +28% and +4%, and +4% is the number that says the 31.361s is the engine and not the measurement: 1.297s of the 32.658s wall clock was everything this harness does around 43 subprocesses.

Memory is worth as much as the time here. An engine that answers in 24s and needs 10.81 GiB and an engine that answers in 31s and needs 5.69 GiB are not ranked by the time column alone, and on a machine sized like the board's c6a.4xlarge, which has 32 GiB, Polars' 16.68 GiB is half the machine.

### The ten million column was mostly row groups

The most useful thing in the 10m report was not the total. It was that the harness computes, for each engine, how much its slowest query beats its fastest, and the answer for rudb was 3.01x where DuckDB's was 52.61x.

| engine | slowest query over fastest, 10m |
| --- | --- |
| duckdb | 52.61x |
| duckdb-pinned | 14.46x |
| datafusion | 9.07x |
| polars | 4.45x |
| rudb | 3.01x |
| clickhouse-local | 2.34x |

A column that flat is measuring whatever every query in it has in common rather than measuring the queries. Reading down rudb's per-query times at ten million rows there was a floor at about 135ms that nothing got under: `SELECT COUNT(*)` took 147.818ms, the min and max of a date took 169.946ms, and the seven date-range queries q37 to q43 took between 135.014ms and 165.891ms each. This file previously read that as a full decode of the file paid once per query, about 5.8s of rudb's 10.263s.

That reading was wrong about the cause, and the 100m run is what caught it. On the full file, ten times the rows, the same rudb binary answers `SELECT COUNT(*)` in **38.350ms**, the min and max of a date in 75.224ms, and q37 to q43 in between 51.453ms and 331.810ms. Ten times the data, and the floor went down rather than up. No per-row cost does that.

The difference is not the rows, it is how the rows are packed. The harness writes its sampled Parquet with `ROW_GROUP_SIZE 8192` (`src/data.rs:287`), and the real `hits` is not packed anything like that.

| file | rows | row groups | rows per group |
| --- | --- | --- | --- |
| `hits-10m-snappy-rg8k.parquet`, what the 10m column read | 9,999,750 | 1204 | 8305 |
| `hits.parquet`, what the 100m column read | 99,997,497 | 226 | 442,467 |

The sampled 10m file has **5.3x more row groups than the file ten times its size**. Divide the counts through and rudb's `COUNT(*)` costs 0.123ms per row group at 10m and 0.170ms per row group at 100m, against a row count that differs by a factor of ten. rudb's floor tracks row groups and does not track rows. The 135ms was about twelve hundred row group headers at roughly a tenth of a millisecond each, and the 3.01x flatness was every query paying that same opening bill before it started.

Three things follow, and they are worth keeping apart.

The first is about rudb: a tenth of a millisecond per row group is too much, and DuckDB reads the same 1204 row groups for a `COUNT(*)` in 1ms total. That is a real cost in rudb's Parquet path and it is not the Parquet format's fault. It is much less visible on the full file because a realistic file has few, large row groups, but a 13.76 GiB Parquet written by something else with 8k row groups is not a hypothetical.

The second is about the harness, and it is a measurement bug rather than an engine one. A sampled file at `ROW_GROUP_SIZE 8192` does not resemble the file it was sampled from, and the whole 1k to 10m ladder is therefore biased against whichever engine is most sensitive to row group count, which on this evidence is rudb. Every sampled column in the table above is pessimistic for rudb by an amount only a rerun can say. The runs stay as they are, because runs here are kept and not replaced, and the fix is a later run against files packed like the original rather than an edit to these.

The third is what survives the correction. rudb still does not use the Parquet footer statistics: `COUNT(*)` should be a footer read and it is not, at either size, and the date range queries should prune whole row groups on a min and max they already have. That was true when it was written and it is still true. What changed is its share, which at 100m is small.

### Where the full file actually goes

At 100m rudb's flatness is 49.80x, so the full file column is measuring the queries rather than a floor, and it can be read query by query. rudb beats DuckDB v1.5.5 on two of them and loses badly on four.

| query | shape | rudb 0.3.45 | duckdb v1.5.5 | |
| --- | --- | --- | --- | --- |
| q29 | group by a regular expression | 2.077s | 7.520s | 3.62x faster |
| q18 | group by two, no ordering | 426.113ms | 667.000ms | 1.57x faster |
| q20 | point lookup | 168.213ms | 31.000ms | 5.4x slower |
| q40 | date range, a case and a wide group by | 331.810ms | 58.000ms | 5.7x slower |
| q27 | top k by two columns | 484.086ms | 55.000ms | 8.8x slower |
| q25 | top k by a date | 483.724ms | 54.000ms | 9.0x slower |

The four losses are one shape, not four. Each is a query that touches few rows or wants only the top ten of them, and each is a query DuckDB answers in under 60ms because it never reads most of the file. rudb reads it. q20 is a single point lookup, q25 and q27 are `ORDER BY ... LIMIT 10`, and q40 is a date range: all four are prunes and early exits rather than faster arithmetic, and all four are the same missing thing as the footer statistics above.

rudb's four most expensive queries in absolute terms are a different problem and a more ordinary one: q19 at 2.919s, q34 at 2.462s, q35 at 2.456s and q23 at 2.271s, against DuckDB's 1.428s, 1.965s, 1.968s and 1.008s. Those are large group-bys over long strings and substring scans, rudb is between 1.25x and 2.25x on them, and closing that is per-core work with nothing clever hiding in it. Together they are 10.1s of rudb's 31.361s against 6.4s of DuckDB's 24.116s, so more than half of the 7.2s gap on the whole suite is those four queries.

So the 31.361s on the full file is roughly: about 1.1s of prune that is not happening on four queries, something under a second of footer reads that are not happening on nine more, and the rest spread evenly across large aggregations where rudb is between 1.2x and 2x DuckDB. The first two are bounded and identified. The third is the milestone work.

Five things travel with all of that or it is worth nothing.

The first five columns are strided samples of the real `hits`, one row in every 99998, 10000, 1000, 100 and 10, so nothing in them is comparable to the public board or to anybody else's number, and as the row group section above says they are packed unlike the file they came from. Only the 100m column is the file itself. rudb, DataFusion and Polars read the source Parquet where it lies and pay to decode it inside every query, which is in the column being compared; DuckDB and ClickHouse paid once at load time and are being timed on a format of their own, and on the full file that load was 42.612s for DuckDB, 38.626s for the pinned one and 26.467s for `clickhouse-local` against rudb's nothing, for a database 24.98 GiB, 19.05 GiB and 10.42 GiB on disk against the 13.76 GiB of Parquet the other three read in place. Polars cannot run q28, q29, q36 or q43 in its SQLContext, so it is 39 of 43 and every ratio in the table is taken over the shared set. `clickhouse-server` is absent from all six runs: its loader fails with `NO_DATA_TO_INSERT` on this build and the runs were taken without it rather than with a row nobody could trust. And the page cache was warm and not dropped, so the cold column is a first pass rather than a first pass off the device.

The load column is worth a second look before the query times are read as the whole story. DuckDB spends 42.612s and 24.98 GiB to make the file fast, which is more than the entire 31.361s rudb takes to answer all 43 queries off the Parquet without spending it. Which of those matters depends entirely on how many times the questions get asked, and the ClickBench board's answer is that they get asked a lot, so the load is amortised and the query column is the one to compare. It is still the case that rudb has no storage format of its own yet, and the day it does is the day this table changes shape.

One number in the reports wants reading carefully rather than at face value. rudb's worst interquartile range at a million rows is 99.2% of the median on q39, which rule two would normally treat as a measurement that did not settle. It did not fail to settle: q37 and q39 are bimodal between about 20.28ms and about 40.4ms, which is one tick and two ticks of a 20ms clock. That is timer granularity on a query too short to resolve, not variance in the engine, and the fix is a longer query rather than more runs. At ten million rows, where the queries are long enough to resolve, rudb's worst is 15.1% on q26 and the 99.2% swing belongs to DuckDB on q3.

On the full file the same artefact has changed hands entirely, and it is now DuckDB's. Every engine in the 100m run trips rule two somewhere, and rudb trips it least: its worst is 10.9% on q20 against DuckDB's 49.8% on q39, the pinned DuckDB's 50.0% on q1, DataFusion's 49.7% on q7 and Polars' 40.8% on q15. Those are the 20ms clock again, not four engines behaving badly. DuckDB's flagged q4 swings between 60.466ms and 80.521ms and its q39 between 40.365ms and 60ms, which is one tick in both cases, and they read as 33.2% and 49.8% only because the queries underneath them are short enough for one tick to be a third of the answer. It is the engines that finish these queries fastest that the timer punishes hardest, which is the opposite of what an IQR column is usually read to mean.

None of this is the goal. The goal is the full file on a c6a.4xlarge, where ten times DuckDB is 2.63s, and no machine this project owns is one, which is the rule seven blocker on every number here.

There is a second measurement of rudb on the same file, and it now runs the same version, which makes it worth putting beside this one. [The full file under the official driver](reports/2026-09-19/the-full-file-on-its-own-format.md) got 15.109s hot for rudb 0.3.45 against DuckDB 1.5.5's 16.965s, on this machine, over the same 99,997,497 rows. This harness says 31.361s against 24.116s. The engine, the machine and the rows are identical and the numbers are half; the difference is that the driver loads the file into rudb's native format first and this harness reads the source Parquet in place. Two smaller differences go the same way: the driver is a best of three where this is a median of five, and the driver has no sampling step at all.

That lead is one mechanism rather than general speed, and the report says where it is. Ten of the forty three queries are `GROUP BY x ORDER BY count(*) DESC LIMIT 10`, rudb answers them from the frequency synopsis its writer stores per column instead of scanning, and those ten are 0.212s against DuckDB's 3.307s. On the other thirty three rudb is 1.09x the released DuckDB and 1.34x the development build, which is parity. The development build is still ahead over the whole benchmark, 14.228s against 15.109s.

Running the same driver again on rudb main, [twelve patch versions later](reports/2026-09-19/twelve-patches-later.md), moves that. 0.3.57 answers all forty three in 13.881s, ahead of both DuckDB builds for the first time, and the whole 1.23s gain is in the thirty three queries that scan rather than in the ten that do not. That closes the scanned part from 1.09x behind the released DuckDB to level, and narrows the development build from 1.34x to 1.23x, so rudb is still behind an unreleased DuckDB everywhere it does real work and ahead on the total only because eleven queries never run.

The older official driver run, [rudb on the full ClickBench file](reports/2026-09-18/the-full-file.md), is a day old and a different engine: 0.3.33, 27.07s hot, 324.297s to load, 43.0 GB on disk. Over the forty two queries that run answered, 0.3.45 is 1.79x faster hot, loads 3.2x faster and writes a file 3.5x smaller.

The load asymmetry in the table above is worth one more line because of it. rudb has a native format and the official driver uses it: that entry's `load` builds a `hits.db` and its own comment calls it native columnar storage. This harness does not use it, and runs rudb the way it runs DataFusion and Polars, off the source Parquet. So the rudb row here is a decode inside every query and the 1.78x is that mode, not the fastest mode rudb has. The two numbers above put a size on that: 31.361s in this mode and 15.109s in the other one.

Which makes one piece of this harness's own prose wrong, and it is wrong in every report above. The engines table prints `the source Parquet, this engine has no storage format of its own yet` against the rudb row, and the `yet` stopped being true somewhere around the native storage work in [the 2026-09-16 reports](reports/README.md). What is accurate is that this harness does not use it. The generated sentence is left alone here rather than quietly corrected, because it is in six committed reports that were written by the harness and are not edited by hand, and changing the text without rerunning them would make the README and the reports disagree about what the reports say.

The six runs in full, written by the harness with nothing typed into them by hand, are [1k](reports/2026-09-18/run-clickbench-gamingpc-wsl-1k.md), [10k](reports/2026-09-18/run-clickbench-gamingpc-wsl-10k.md), [100k](reports/2026-09-18/run-clickbench-gamingpc-wsl-100k.md), [1m](reports/2026-09-18/run-clickbench-gamingpc-wsl-1m.md), [10m](reports/2026-09-18/run-clickbench-gamingpc-wsl-10m.md) and [100m](reports/2026-09-19/run-clickbench-gamingpc-wsl-100m.md). At a million rows all six engines agreed on every answer the data settles, which is 31 of 43 queries, to the last significant digit of a double. At ten million the data settles fewer of them, 27, because more of the `LIMIT 10` cuts land in ties, and the six agreed on all of those. The one settled disagreement at every size is q4, where the pinned DuckDB's `AVG(UserID)` is short by a multiple of 2^64 and rudb's is not, and on the full hundred million bigints that is the same bug at ten times the scale rather than a new one.

There is a seventh file, [100m on a tmpfs scratch](reports/2026-09-18/run-clickbench-gamingpc-wsl-100m-tmpfs-scratch.md), and it is kept because runs are kept even when the run is the mistake. Three of its six engines are missing: DuckDB and the pinned DuckDB died without saying anything and `clickhouse-local` said `NOT_ENOUGH_SPACE`. The harness puts its scratch directory under `std::env::temp_dir()`, `/tmp` on this box is a 16 GB tmpfs, and the three engines that load into a format of their own were therefore writing a 25 GiB database into RAM on a 31 GiB machine. The fix was `RUDB_BENCH_SCRATCH=$HOME/rudb-data/scratch` and the engines were then left entirely uncapped, which is what the 100m column above is. It is worth keeping because the failure mode is silent, it only appears at the size where it matters, and an engine dying with no message is very easy to write up as an engine that cannot do the work.

### What the earlier ladders said

The table below records rudb 0.3.5, when rudb ran 41 of 43 queries and still excluded Q19 and Q33. It is retained because benchmark history should not be rewritten after the engine improves. These measurements compare DuckDB and rudb on `gamingpc-wsl`, which is 32 hardware threads, at one million and ten million rows, with five hot runs of each query after a cold one. Query time is what each engine reports. The ratio covers the 41 queries both engines ran at that revision.

| | 1m rows | 10m rows |
| --- | --- | --- |
| duckdb v2.0.0-dev84237 cc7e7bac7f | 556ms | 3.015s |
| rudb 0.3.5 | 5.854s | 51.929s |
| rudb against duckdb | 11.28x | 19.51x |

The direction of that line is the whole story and it is not a flattering one. Ten times the rows cost DuckDB five and a half times the time and cost rudb nine, so the gap grows with the data. A gap that grows with the rows is a per-row cost rather than a startup cost, and per-row cost is what the F milestones are for.

The column that says most of why is the one nobody puts in a headline, which is how many cores each engine actually used.

| cores used, hot | 1m rows | 10m rows |
| --- | --- | --- |
| duckdb | 2.25 | 9.01 |
| rudb | 0.94 | 0.99 |

rudb never gets above one. It is a single threaded engine on a box with 32 threads, and at ten million rows DuckDB is keeping nine of them busy while rudb keeps one. That is nine of the nineteen. The rest, a factor of about two, is the loop, and the loop is the part that has to come down first because it is also what every added thread would be running.

The memory half of the claim goes the other way, which is the one piece of good news in here. It is worth putting next to the time, because a time that came out of twice the memory is not the same result.

| peak RSS | 1m rows | 10m rows |
| --- | --- | --- |
| duckdb | 307.60 MiB | 1.63 GiB |
| rudb | 172.14 MiB | 1.27 GiB |

Every operator of every query writes what it cost, so the harness can say where the time goes rather than only how much of it there is. At ten million rows, over the whole suite: FileScan 28.198s and 54.8 percent of it, Aggregate 13.427s and 26.1 percent, Filter 5.069s and 9.9 percent, TopN 4.525s and 8.8 percent, Project 228ms and 0.4 percent. Every one of those operators ran the reference implementation of its seam, which is the slow path kept for differential testing, so more than half of this is a Parquet decode that has not been specialised yet.

Four things travel with all of that or it is worth nothing. These are strided samples of the real `hits`, not the file, so nothing here is comparable to the board or to anybody else's number. rudb reads the Parquet where it lies and pays to decode it inside every query, where DuckDB paid once at load time and is being timed on a format of its own. The rudb column is 41 queries against DuckDB's 43 and the ratio is taken over the shared ones. And both runs were taken with the page cache warm and not dropped, so the cold column is a first pass rather than a first pass off the device.

The two runs in full, written by the harness with nothing typed into them by hand, are [1m](reports/2026-09-14/run-clickbench-gamingpc-wsl-1m-0.3.5.md) and [10m](reports/2026-09-14/run-clickbench-gamingpc-wsl-10m-0.3.5.md). Next up the ladder is the file itself.

The earlier sweep on the same machine, seven engines at 1k, 10k, 100k and 1m, is kept below because runs are added and never replaced.

| engine | 1k rows | 10k rows | 100k rows | 1m rows |
| --- | --- | --- | --- | --- |
| duckdb v1.5.5 | 101ms | 125ms | 314ms | 594ms |
| duckdb-pinned v2.0.0-dev84237 cc7e7bac7f | 136ms | 165ms | 342ms | 593ms |
| clickhouse-server 26.9.1.1162 | 135ms | 151ms | 246ms | 449ms |
| clickhouse-local 26.9.1.1162 | 149ms | 179ms | 432ms | 2.866s |
| datafusion 55.1.0 | 218ms | 359ms | 589ms | 970ms |
| polars 1.44.2 | 537ms | 596ms | 785ms | 1.216s |
| rudb 0.2.31 | 22ms | 123ms | 1.117s | 9.854s |

Against DuckDB that was 0.22x, 0.98x, 3.56x and 16.59x. rudb won at a thousand rows because at a thousand rows almost nothing is being measured except what it costs to start, and rudb starts in nearly no memory at all. Those are one hot run each rather than the five rule two asks for, and `clickhouse-server` stayed up across the whole suite with its own caches warm where every other row is a fresh process, so that row is a different quantity from the rest. The four are [1k](reports/2026-09-12/run-clickbench-gamingpc-wsl-1k-quiet.md), [10k](reports/2026-09-12/run-clickbench-gamingpc-wsl-10k-quiet.md), [100k](reports/2026-09-12/run-clickbench-gamingpc-wsl-100k-quiet.md) and [1m](reports/2026-09-12/run-clickbench-gamingpc-wsl-1m-quiet.md). The same ladder on `vmi3391933`, which is eight threads, is in [reports/](reports/), and the two are not comparable to each other because rule seven says they are not.

## Status

Early, and running. Every reporting rule below is a property of the apparatus rather than of the engine, and every one of them was easier to build early, against engines nobody here has a stake in, than on the afternoon somebody wants a headline.

What exists is the recorded board, which was recorded before there was anything to flatter, and a measurement path that runs end to end: load, on-disk size, a cold run, five hot runs, the median and the interquartile range, peak resident memory, and a per-query table with a list of the reasons the number may not be published underneath it. Today that list is never empty.

```
$ rudb-bench baselines
ClickBench, c6a.4xlarge, sum of the best of three per query, 43 queries
Recomputed from the official result files on 10 September 2026.

system        hot total    hits on disk
Umbra            8.10s    8.30 GB
ClickHouse      18.07s    9.42 GB
DuckDB          26.25s    20.46 GB
Polars          45.35s    not reported
DataFusion      45.57s    14.78 GB
Velox           81.51s    not reported

Ten times DuckDB is 2.63s, which is past every system in that table.
spec/02-the-goal.md is where that either closes or does not.
```

That last pair of lines is the point of this repository existing. The target is 3.1x past the fastest CPU engine anybody has published on that machine, and the Rust and Arrow stack is currently 1.7x behind DuckDB rather than ahead of it. Any harness that makes that easy to forget is doing harm.

### What runs today

`rudb-bench run` measures the smoke suite, which is not one of the seven suites in the specification and is not a benchmark. It generates ten million rows of its own in a few seconds and runs six queries over them that between them touch a scan, a filter, a sum, a group by, a top k, a count distinct and a join. It exists so that the measurement path is exercised on every commit rather than on the day somebody needs a real number and finds out the apparatus rotted.

```
$ rudb-bench run
suite    smoke
engine   duckdb v1.5.5 (Variegata) d8cdaa33fd
load     6.261s to build, 92.01 MiB on disk

query  shape                     cold        hot median         IQR   peak RSS
q1     count                    42.641ms     37.046ms        4.6%  19.75 MiB
q2     filter and sum           59.964ms     57.244ms        8.2%  105.06 MiB
q3     group by, low card       50.402ms     49.107ms       14.8%  43.34 MiB
q4     group by and top k       78.783ms     52.696ms       13.5%  105.58 MiB
q5     count distinct           53.982ms     58.553ms        9.5%  125.52 MiB
q6     join and group by       145.637ms    128.908ms       12.2%  34.61 MiB

total    431.410ms cold, 383.553ms hot, over 6 queries
peak     125.52 MiB

This is not a publishable number, because:
  no machine this project owns is a c6a.4xlarge, 16 vCPU, 32 GiB, gp2, per rule seven
  the smoke suite is not comparable to any board
```

The run above was taken on a laptop that was busy doing something else at the time, which is why the load took six seconds and the interquartile ranges are in double figures. That is the ordinary state of a developer machine and it is exactly why rule seven exists.

That last block is the part worth looking at. A result carries the list of reasons it may not be published, the list is computed rather than written, and it is printed under every table. The rules are enforced in the types: a peak resident set is either a number of bytes or a sentence explaining why there is not one, a distribution built from four runs says no when asked whether it may be published, and a best of three is a different constructor from a median so a ClickBench-convention number cannot be produced by accident.

`rudb-bench machine` records what has to be read next to a number: the processor, the thread count, the memory, the frequency governor, the turbo state, the filesystem under the data and its mount options, whether the page cache can actually be dropped, and which `/usr/bin/time` will be reading the peak. Anything that could not be read is marked and says why, because a field that silently defaulted is a lie that survives into a report.

`rudb-bench suites` lists the seven suites from the specification with what each one needs before it can run, which is a download or a generator in every case.

### The first full ClickBench, and what it says

ClickBench has now run end to end on `gamingpc-wsl` against all five rivals, on the real hundred million row `hits.parquet`, with each engine loaded the way its own entry on the official board loads it. It is committed as the `2a the baseline` row of `runs/clickbench.txt` and `rudb-bench ledger` renders it.

```
engine              hot total   hot cpu   peak RSS      load    on disk   vs duckdb
duckdb                25.932s   313.590s  10.82 GiB    57.865s  24.95 GiB     1.00x
clickhouse-local      25.619s   393.530s   7.64 GiB    36.493s   9.69 GiB     1.32x
datafusion            23.716s   555.330s  10.83 GiB   no load   13.76 GiB     1.21x
polars                25.947s   507.320s  17.04 GiB   no load   13.76 GiB     1.48x
clickhouse-server     10.329s  not read   not read   206.791s    8.77 GiB     0.53x
```

The ratio column is over the thirty nine queries every column ran, not over the totals, which is why DataFusion's total is the smallest on the page and its ratio is still 1.21x. DuckDB's q29 alone is 7.293s of its 25.932s, and q29 is one of the four Polars cannot express, so the shared set is the set where DuckDB is not carrying its single worst query.

Four things have to travel with that table or it is worth nothing. The machine is not a `c6a.4xlarge` and nothing here is comparable to the recorded board. The `clickhouse-server` row is the only one whose engine stayed up across the whole suite, so its hot is buffer pool warm where every other row's hot is only page cache warm, and a ratio against it is a ratio between two different quantities. DataFusion and Polars have no storage format, so their empty load column is a decode inside every query rather than a decode they avoided, and their on disk figure is the source Parquet. And four engines each blew past the ten percent spread rule on at least one query, worst of all DuckDB at 31.1 percent on q27, which is a 65ms query where most of what was timed is starting a process.

What it is useful for is the shape rather than the ranking. Three of the five engines land within nine percent of each other on total hot time, which is close enough that the ordering between them on this machine is noise. The one that does not is `clickhouse-server`, at 10.329s, and the difference between it and `clickhouse-local` at 25.619s is not the engine, since it is the same binary and the same version. It is the sorting key and a process that stayed up. That is a 2.48x gap between two runs of one engine over one dataset, which is a larger effect than anything separating the four engines above it, and it is the single most useful number this run produced for deciding what the layers of rudb should be.

The target is unchanged and now has a real second reference. Ten times DuckDB's recorded board number is 2.63s. The fastest thing measured on this machine is 10.329s. The distance is not a tuning distance.

### What the answer check can and cannot catch

Five engines running the same forty three queries check each other for free, and the first full run turned that check into forty eight lines of disagreement across seventeen queries. None of them was a wrong answer, and finding that out is worth writing down because the obvious reading of forty eight disagreements is that something is badly broken.

One of them was this harness. DataFusion prints a bordered table rather than CSV, because its CSV writer truncates streaming output at one batch, and the answer comparison reads the numbers out of whatever text an engine printed. That was safe while the column names were `count(*)`. On ClickBench, DataFusion names an unaliased expression after the expression, so `SUM(ResolutionWidth + 1)` comes back under the heading `sum(hits.ResolutionWidth + Int64(1))`, and q30 is ninety of those. Its header carries a hundred and seventy eight numbers on top of the ninety in the answer, and the run duly reported DataFusion returning 268 numbers against DuckDB's 90. The comparison now drops a bordered table's header before it reads anything, and q30 agrees.

Fourteen more are one sentence written fourteen times. ClickBench is full of `GROUP BY` with `ORDER BY COUNT(*) DESC LIMIT 10` and nothing after it, and at the tenth place thousands of groups have the same count. Which ten come back is whichever ten the engine's hash table reached first, and every one of them is a correct answer to the query as written. q33 is the extreme case, grouping by `WatchID` and `ClientIP` unfiltered where almost every group has a count of one. q18 does not even have an `ORDER BY`. Rule three says the official text is run as written or the comparison is not one, so rewriting them until they were deterministic was never available.

The last two are more interesting than they look. q4 is `AVG(UserID)` over a hundred million bigints near ten to the eighteenth, where the order the partial sums are added in moves the result further apart than the one part in a billion this harness calls the same number, so ClickHouse disagrees with DuckDB about an average. That one was read as nobody being wrong for as long as there was nothing to arbitrate with, and it was read wrong. The differential harness has since put rudb and DuckDB to the same file, and DuckDB is short by a multiple of two to the sixty fourth on the sum behind that average, where its own hugeint and decimal paths, rudb, and adding the column up outside a database all agree. It is [tamnd/rudb-compat#12](https://github.com/tamnd/rudb-compat/issues/12), it is the reference column of this board that has it wrong, and it is the reason a benchmark comparing engines is not a substitute for a checker that knows what the right answer is. q43 is a timezone. ClickHouse renders a `DateTime` in the machine's timezone and DuckDB renders a `TIMESTAMP` in none, so the same epoch second prints as `2013-06-23 22:06:40` on `gamingpc-wsl`, as `2013-06-23 17:06:40` on `server3` and as `2013-06-23 15:06:40` out of DuckDB anywhere. That is not only a comparison problem. It means the ClickHouse column's answer to q43 depends on which machine ran it, which is worth knowing before somebody writes a compatibility test against one.

Then rudb joined the table and the check found one more of its own bugs, which was again the harness. The comparison used to read an answer by scanning the whole text for runs of digits, which works on a suite whose strings are `tag-12` and falls apart on ClickBench, where the answers are Russian page titles with years and model numbers in them. q38 came back as rudb returning 14 numbers against DuckDB's 11 when both engines had returned exactly ten rows with the same ten counts. The difference was digits inside the titles. The comparison now cuts the output into fields first, reads a field as a number only when the whole field is one, and compares the rest as text.

That is stricter rather than looser, and it changed what the suite knows about itself. A title that came back wrong used to be invisible, because only the counts were ever compared. With the titles in the comparison, q37 and q38 turn out to be the same tie at a `LIMIT 10` as the other fourteen, on a page title and a URL rather than on a user ID. q38 fails it outright on a hundred thousand row sample, where the tenth and eleventh titles both have thirteen views and the two engines keep a different one. q37 passes only because both engines happened to keep the same ten URLs in a different order, which is luck rather than agreement. Both are on the list now.

So nineteen of the forty three are named in `CLICKBENCH_UNSETTLED` with the reason, and they are reported under the table as answered differently with the data not saying which is right, rather than as disagreements. The cost is real and the README should say it plainly: a wrong answer from rudb on any of those nineteen would go unnoticed here. Twenty four are still checked to the last significant digit of a double, the whole smoke suite is still checked because it was written with a total order on every query, and catching a wrong q33 is a job for the differential harness in `tamnd/rudb-compat`, which can compare against one engine on data it controls, rather than for a benchmark comparing five.

### ClickBench is five query sets, not one

The official ClickBench repository keeps a `queries.sql` per engine and they are not copies of each other. Running one engine's text against all of them would be a measurement of somebody's translation, so this harness carries the official text per engine and says in the query table which engine gets which, and why.

Most of the difference is spelling. DataFusion's file quotes every identifier, because it lowercases the ones nobody quoted and every column in `hits` is camel case. q28 and q29 average a string length, and there the difference is which function means what. ClickHouse's `length` counts bytes and DuckDB's counts characters, so DuckDB's file says `STRLEN`, which is DuckDB's byte counting function. Measured on `héllo`, ClickHouse's `length` is 6 and its `lengthUTF8` is 5, DuckDB's `strlen` is 6 and its `length` is 5. The two texts differ so that the two boards ask the same question, which matters here because `hits` is full of percent encoded UTF-8 and the two counts are not the same number over it.

Polars is the one engine with gaps. Its `SQLContext` accepts thirty nine of the forty three and refuses q28 for `strlen`, q29 for `regexp_replace`, q36 for the repeated `ClientIP` output name in the group by, and q43 for `date_trunc`. Those are four missing functions rather than four different opinions, and the alternative to declaring them absent is rewriting them until they ran, which would make the column a measurement of the rewrite. So they are declared absent in the query table, in a diff somebody reviewed, and never discovered at run time.

A column that is short a query is then handled everywhere it would otherwise lie. The per-query grid is looked up by name rather than by position, so a missing q28 does not slide q29's time onto q28's row. The ratio against the reference engine is taken over the queries both columns ran, and the row says how many that was, because dividing a short total by a full one hands the best number on the page to whichever engine ran the least. The report prints one line per gap naming the query and the reason. And a column with any gap in it cannot be published, since rule three asks for the whole suite.

### TPC-H is one query set, and was checked rather than assumed

The difference between TPC-H and ClickBench is that TPC-H has a specification. The business questions have one meaning, there is no per engine `queries.sql` anywhere to be faithful to, so there is one text and every engine gets it. The text is DuckDB's rendering out of `extension/tpch/dbgen/queries`, because that one is public, versioned and checkable rather than retyped by us out of a PDF.

Whether every engine actually takes that text is a question with an answer, so it was asked before the table was written, against a real SF 0.01 corpus rather than against an empty schema. DuckDB, ClickHouse and DataFusion each answered all twenty two. Polars answers nineteen: its `SQLContext` does not resolve the correlated `p_partkey` in q02 or q17, and it rejects the `NOT LIKE` inside the q13 join constraint. Those three are declared absent in the table for the same reason the four ClickBench ones are.

### The first full TPC-H at scale factor 100, and what it says

TPC-H has now run end to end on `gamingpc-wsl` over the real 22.50 GiB corpus, all twenty two queries, five runs each, with DuckDB and `clickhouse local` each loading into a format of their own and DataFusion reading the Parquet where it lies. It is committed as the `2a` row of `runs/tpch.txt`.

```
engine              hot total   hot cpu   peak RSS      load    on disk   vs duckdb   worst IQR   shape spread
duckdb                30.936s   536.480s  16.65 GiB    53.524s  25.90 GiB     1.00x       10.4%         15.29x
datafusion           106.241s  2209.510s  28.14 GiB   no load   22.50 GiB     3.43x       34.7%         64.06x
clickhouse-local     130.813s  2269.040s  17.36 GiB   248.100s  25.92 GiB     4.23x       65.9%         66.35x
```

The first thing this says is that TPC-H is not ClickBench, and it is worth saying loudly because the two tables come from the same harness on the same machine a week apart. On ClickBench three of the five engines landed within nine percent of each other on total hot time and the ordering between them was noise. On TPC-H the second engine is three and a half times the first and the third is four and a quarter. Nothing about the machine changed. What changed is that ClickBench is forty three scans of one table with a `GROUP BY` on the end and TPC-H is twenty two join plans over eight tables, and joins are where the engines are actually different.

The shape spread column is the compact way to see it. That is an engine's slowest query over its fastest, and DuckDB's is 15.29x where the other two are 64.06x and 66.35x. An engine whose worst query is fifteen times its best has a plan for everything in the suite. An engine whose worst is sixty five times its best has a handful of queries it falls off a cliff on, and it is the same handful in both columns: q09, the product type profit measure, is 3.105s for DuckDB and 37.064s for `clickhouse local`, and q02 is 302ms against 25.719s. Those are the correlated subquery and the six way join, not the scan.

The `clickhouse-server` row is missing and the reason is recorded rather than glossed. Building the MergeTree table needs `OPTIMIZE TABLE lineitem FINAL` over six hundred million rows, and `clickhouse client` gives up on it at its three hundred second default. That is a harness bug and not a ClickHouse limit, since the server was still working when the client stopped waiting. Until it is fixed the TPC-H table has no tuned ClickHouse column, which matters because ClickBench showed the tuned server at 2.48x the local one over the same data, so the gap between DuckDB and ClickHouse here is an upper bound on the real one rather than the real one.

The target does not move. Ten times DuckDB over this suite on this machine is 3.09s hot over twenty two queries, against a current fastest of 30.936s. That is the second number of its kind, after ClickBench's, and it points at the joins rather than at the scan.

### TPC-H at SF100 again, and the row rudb cannot fill yet

The run above is kept and this is a second one beside it rather than a replacement, because the corpus is not the same corpus: this one is 33.20 GiB of Snappy Parquet against that one's 22.50 GiB, generated fresh by `dbgen` at SF100 into the eight tables the suite wants. Row counts are exact, `lineitem` at 600,037,902. It adds the pinned DuckDB as a fourth column and it is the first TPC-H run in this file that asked rudb.

| engine | query time | cores | peak RSS | overhead | load | on disk | slowest over fastest | vs duckdb |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| duckdb v1.5.5 | 27.416s | 17.95 | 16.65 GiB | +9% | 49.976s | 26.24 GiB | 13.87x | 1.00x |
| duckdb-pinned v2.0.0-dev84237 | 25.676s | 13.61 | 16.82 GiB | +19% | 56.701s | 27.96 GiB | 13.46x | 0.94x |
| datafusion 55.1.0 | 103.139s | 21.60 | 29.22 GiB | +2% | no load | 33.20 GiB | 56.00x | 3.76x |
| clickhouse-local 26.9.1.1562 | 123.414s | 16.71 | 16.99 GiB | +5% | 84.779s | 30.23 GiB | 64.06x | 4.50x |

**rudb has no row here, and the harness is what refused to give it one.** The engine did not fail and was not left out; the suite declines to time it, saying that every join in rudb is a nested loop and TPC-H is twenty two of them, so what got measured would be a hang rather than a number. `spec/07-execution.md` section 7.4 specifies a partitioned radix hash join switched at runtime on observed build cardinality, and that is milestone E3 and not yet built. rudb was 0.3.45 for this run and main is 0.3.49, and the guard still fires on both.

That is the single most important fact in this file about the ten times goal. ClickBench is forty three scans of one table and rudb is 1.78x off DuckDB on the real one. TPC-H is twenty two join plans over eight tables and rudb cannot start. The two suites are not two difficulties of the same task, and a ClickBench number says nothing about this column. Ten times the faster DuckDB here is **2.57s over twenty two queries**, and the distance to it is currently not measurable rather than large.

What the rival columns say is the same thing the earlier run said, which is worth something because the corpus changed underneath it and the shape did not. The two DuckDBs are within six percent of each other and both have a plan for the whole suite, at 13.87x and 13.46x slowest over fastest. The other two fall off a cliff on a handful of queries and it is the same handful: `clickhouse-local` takes 27.637s on q02 against DuckDB's 279ms and 34.418s on q09 against 2.970s, and DataFusion takes 21.463s on q18 against 2.074s and 20.423s on q21 against 2.447s. Those are the correlated subquery, the six way join and the large aggregation, not the scan. All four engines agreed on every answer the data settles, which here is 22 of 22 queries, to the last significant digit of a double, and the only rule two flag in the whole run is DuckDB swinging 11.0% on q09.

Two caveats travel with this table and both matter more than they would at ClickBench sizes. The hot column is not fully hot: the block layer served 9.83 GiB to `clickhouse-local` and 12.04 GiB to DataFusion during the hot runs, because SF100 does not fit in this machine's 31.34 GiB alongside an engine, so those two numbers include disk in a way DuckDB's mostly do not. And DataFusion peaked at 29.22 GiB of resident memory on a 31.34 GiB box, which is close enough to the edge that its number is partly a number about surviving. The suite's own description says SF1000 is the scale that measures spilling; at SF100 on this machine it is already measuring a little of it.

Polars has no column because the kernel killed it. `dmesg` records the run's `python3` going down at `anon-rss:30808780kB`, which is 29.4 GiB on a box with 31.34 GiB, and two reruns since went the same way at 30.8 and 31.1 GiB. So the missing column is Polars not fitting SF100 in this machine in streaming mode, which is a fact about the pair and not a defect in either. It belongs beside DataFusion peaking at 29.22 GiB and surviving: these two engines are both at the edge here and one of them went over it.

What the report says about that failure is the wrong sentence, and the harness is why. An OOM kill leaves nothing on stderr, so the wrapper printed the last six lines it found, which happened to be a `DeprecationWarning` about casting a String to a Date. That reads as though a warning stopped the run. It did not: the same cast runs to completion by hand. `both` in `src/data.rs` only fails a run on a non-zero exit, and it should say when the exit was a signal rather than quoting whatever was last on stderr, because the current message sends a reader after the wrong thing. This paragraph is that reader, an hour later.

### Three engines, three opinions about how wide a decimal is

The first two TPC-H runs both reported q01 and q08 as disagreements, thirty two numbers against thirty two and four against four, which is the shape of a report saying every row is there and every number is wrong. Running the two queries by hand against all three engines says otherwise. Every engine agrees on the answer. They disagree about the result type.

`avg` over a `DECIMAL(15,2)` is a double in DuckDB, a decimal of scale six in DataFusion and a decimal of scale four in ClickHouse. So the average quantity in q01 comes back as `25.499370423275426`, `25.499370` and `25.4993`, which are the same number printed to three widths. q08 divides two decimal sums and does the same thing, worse: DuckDB says `0.039535108776109315`, DataFusion says `0.03953510` and ClickHouse says `0.0395`. A relative tolerance of a part in a billion calls all three pairs different, correctly, because they are different to a part in a billion and there is nothing in the text that says they should not be.

So the comparison now allows two numbers to differ in the last place either of them was printed to, which is `10^-f` for `f` decimal places and is absolute rather than relative. The wider of that and the old relative tolerance wins, so nothing that used to pass now fails. The weakening is real and applies only to the decimal places: a number printed with no decimal point at all is compared at the relative tolerance alone, so a count off by one in a hundred and forty eight million is still caught, and a ratio of 0.041 against everybody else's 0.0395 is still caught however few places anybody printed.

This is a different kind of concession from the nineteen unsettled ClickBench queries and it is worth keeping the two apart. Those are queries whose answer the data does not determine, so no checker can ever check them. These two are queries whose answer every engine got right and the checker was reading the printing rather than the value.

### ClickHouse is two rows, because it is two systems

`clickhouse local` and a ClickHouse server are not the same engine wearing different clothes. The local one reads the Parquet where it lies and has no sorting key, no primary index and no chance to have merged anything. The server owns its data in MergeTree parts ordered by the key from the suite's own `create.sql`, which for ClickBench is the key ClickHouse Inc. publishes in theirs. Measuring only the local one and calling the result ClickHouse would flatter us, and by a lot: over three runs of the smoke suite on server3 the local row totalled between 3.5s and 5.3s hot where the tuned server totalled between 1.6s and 2.3s, so the engine our headline claim is stated against is somewhere around twice as fast as the row that was standing in for it. The harness now runs both and prints both, and the sorting key costs one or two percent on disk, 50.60 MiB against 49.49 MiB for the same ten million rows.

Both ClickHouse rows get the column types out of the official `create.sql`, and only the tuned one gets the sorting key. That split is deliberate. A sorting key is tuning and belongs to the row that exists to show what tuning is worth. A schema is not tuning: asking ClickHouse to infer one from the Parquet gives every column back as `Nullable`, which costs a null map per column and turns several things off, and nobody deploys that. DuckDB never paid it, because its `CREATE TABLE AS SELECT` out of the same file gets the real non-null types, so an inferred ClickHouse schema was a handicap this project had put on ClickHouse rather than a difference between the engines. All 105 `hits` columns are copied out of `clickhouse/create.sql` down to the generic spellings it uses, since writing `Int64` where the file says `BIGINT` would be a diff against the thing this is meant to be a copy of.

Reading the server's on-disk number needs care, and getting it wrong is easy. `OPTIMIZE TABLE ... FINAL` writes a new part and leaves the parts it replaced on disk as inactive until `old_parts_lifetime` expires, which is eight minutes by default, so a directory size taken straight after the load counts the data roughly twice. That first read said 128.03 MiB and would have been published as the sorting key costing 2.6x. Both ClickHouse rows now ask ClickHouse instead, with `select sum(bytes_on_disk) from system.parts where active`, and fall back to the directory only when that answer does not parse.

### DuckDB is two rows, for a different reason

There are two DuckDBs in this table and they are not the same program. The `duckdb` row is whatever DuckDB is released, which is what a person would install and is the rival on the public board. The `duckdb-pinned` row is the build rudb's compatibility is actually stated against, which is a commit on the v2.0 development branch recorded in `crates/rudb-parse/grammar/VENDOR` in the rudb checkout.

That distinction is not a detail. The two builds speak different languages: the pinned one accepts `ORDER BY x ASCENDING`, which a released 1.5 rejects, and a released 1.5 reads `[1, 2] <-> [3, 4]` as a single token, which the pinned one does not. Every compatibility number this project publishes is against the pinned build. A performance table that only ever measured the released one would be reporting how we do against something other than the thing we are matching, and the gap between the two is ours to know rather than ours to assume.

The pinned row has to be pointed at its binary with `RUDB_BENCH_DUCKDB_PINNED`. There is no release at that commit, so nothing on a `PATH` is reliably it, and a row that guessed would sooner or later measure the released DuckDB twice and print the second column as if it meant something. `scripts/oracle` in the rudb checkout is what puts the binary on a machine. When the variable is not set the row abstains and says so, and when it resolves to the same build as the `duckdb` row the run is refused, because two identical columns is not a comparison.

### A column that is flat is not a column about the queries

The tuned server row came back with q1 count at 222ms, q2 at 268ms, q3 at 267ms, q4 at 280ms, q5 at 296ms and the q6 self join at 378ms. A count and a self join over ten million rows do not cost the same, so that column is not measuring the queries: whatever all six have in common is bigger than the difference between them, and here it is `clickhouse client` starting up and connecting. This is the same finding as the interquartile ranges below, arrived at from the other direction, and it is worth catching by machine rather than by eye.

So a result now reports its shape spread, the hot median of its slowest query over the hot median of its fastest, and a column under 2x is named in the report and refused for publication. On the run above duckdb spread over 4.61x and datafusion over 5.83x, which is what a column that is measuring the queries looks like, while clickhouse-local at 1.50x, clickhouse-server at 1.70x and polars at 1.24x were all flagged. Read a flagged column as an upper bound on the engine and not as a measurement of it. Every one of those three starts a process per query whose startup is tens or hundreds of milliseconds, which is the same cause the regression gate section below arrives at, and the fix for all of it is the in-process measurement path at sub-milestone 2b.

### Every engine gives its disk back as soon as it is measured

Three of the five engines convert the Parquet into a format of their own before they run anything, and on ClickBench those copies are roughly 20 GB for DuckDB, 13 GB for `clickhouse local` and 12 GB for the ClickHouse server. A harness that loads all of them and holds them until the report is written needs the sum of every engine's format free on the disk, about 45 GB, when the run itself never needs more than the largest one at any moment. That is not a detail. It decided which machine the suite runs on, and it is still worth having on the machine with 748 GiB free, because a run that needs the largest engine's copy rather than the sum of all of them is a run that can be reproduced somewhere other than here.

So `Engine::unload` exists and the comparison calls it the moment an engine finishes, on the failure path as well as the success one. The comparison already ran one engine to completion before starting the next, and everything the report prints about an engine's size was read at load time and is sitting in its result by then, so nothing in the table changes and peak disk drops from the sum to the largest. The default implementation does nothing, which is correct for DataFusion and Polars: they read the Parquet where it lies, they never made a copy, and there is nothing of theirs to remove.

What this does cost is the afternoon where somebody wants to open an engine's database after a run and look at what it built. `RUDB_BENCH_KEEP` turns the whole thing off for that. It is set deliberately and never by default, because a debugging convenience that quietly triples the disk a run needs is a debugging convenience that stops the run.

### A smaller ClickBench, for the loop rather than for the board

A full ClickBench is hours, and most of the changes that want one are asking whether they did anything at all. `rudb-bench run clickbench --rows 1m` answers that question in minutes. It writes a smaller copy of `hits` next to the original once, reuses it on every later run, and prints exactly the same table over it. On `server3` the copy is 146 MB against 14.8 GB, DuckDB loads it in 14 seconds against forty minutes, and the whole 43 query suite is 16 seconds of hot time. The row count is written the way people say it, so `1m`, `200k` and `500000` all work.

The rows it keeps are every hundredth one and not the first million. That distinction is the whole reason this is a feature rather than a `head`. `hits` arrives in event order, so its first million rows are one morning of a handful of counters, and the group by queries that most of ClickBench is made of would be measuring a table shape that does not exist in the real file. Taking one row in every hundred keeps the cardinality of every column and the full date range, at a hundredth of the size.

Nothing measured this way may be published, and the harness enforces that rather than trusting anybody to remember. The sample sentence saying how many rows out of how many goes on the table and on the comparison header, `publishable` returns it as a reason, and `--rows` is refused outright together with `--record`, `--check`, `--check-drift` and `--store`, because neither the baseline file nor the runs file has a field saying which size it was taken at and the two would sit next to each other looking comparable. It is also refused on any suite of more than one table, since sampling `lineitem` and `orders` a table at a time keeps almost none of the rows that join, and a suite of joins that answer nothing is a suite that looks a hundred times faster.

### The regression gate, and what it does not catch yet

`rudb-bench run smoke --record` writes what just ran into `baselines/smoke.txt`, which is committed. `rudb-bench run smoke --check` runs the suite again and fails when a query's median moved past two times the recorded one and its samples no longer overlap the recorded ones. `--check-drift` is the same at ten percent and refuses to run against a record from a different machine, because ten percent between two machines is a fact about the machines. Both take fifteen hot runs per query rather than the five a person reading a table gets, since the interquartile range of five samples is the gap between the second and the fourth of them.

A query that swung wider than ten percent on the day is reported and never failed, which is section 13.8 of the engine specification and is the difference between a gate people keep and a gate people turn off. On the smoke suite today that rule declines almost everything, and the cause is measured rather than guessed. Every query here runs as a fresh subprocess on purpose, so that no engine gets a warm allocator and a warm buffer pool the others paid for. On queries of tens of milliseconds that means most of what gets timed is the process start. Fifteen runs of `duckdb -c "select 1"` on an idle server3, with no query attached to them at all, spread from 30ms to 60ms, an interquartile range of 37 percent of their own median. Fifteen runs of `clickhouse local --query "select 1"` spread from 180ms to 510ms, which is 48 percent.

So the gate prints how many queries it actually compared on every run, and says plainly that a run which compared none of them should be read as a red rather than a green. The fix is not a bigger threshold. It is to stop timing `execve`, which is the in-process measurement path at sub-milestone 2b.

### The kernel record

`rudb-bench kernels` is the in-process measurement the paragraph above says is the fix. It does not start an engine and it does not send anybody SQL, because a kernel is a function over a vector that takes microseconds and the only way to time one is to link it and call it. The measuring code therefore lives in the rudb repository, as `cargo xtask kernels`, where it can be compiled against the working tree instead of against the last published crate. This subcommand runs that task with `--json`, puts a machine name on what comes back, writes it to `baselines/kernels.txt`, and compares the next run against it.

The record is 682 cells over seven tables: comparison and arithmetic for every physical layout in every form pair at zero, one and fifty percent nulls, string comparison where the prefix decides against where the prefix ties, the select against compact surface swept over selectivity by chunk shape, the vector size sweep from 256 to 4096, the expression table and the filter table. The last two are the things in the record that are not kernels. The expression table times a bound expression walked as a tree against the same expression prepared once, at a full chunk and at sixty four rows, which is what a chunk looks like after a selective filter. The filter table times a whole conjunction evaluated as one boolean expression against the same conjunction run a conjunct at a time over the rows the earlier conjuncts left, which is the selection threading in tamnd/rudb#98. On `gamingpc-wsl` at rudb 0.2.3, comparison of two flat columns with no nulls costs 0.055 nanoseconds a row on a boolean, 0.054 on an 8 bit integer, 0.13 on a 32 bit one, 0.46 on a 64 bit one and 1.20 on a double. Those are per row costs of one loop and they are not a query result, so reporting rule ten applies to every one of them: a kernel number explains a query number and is never quoted as one.

The reason to commit them rather than read the table and move on is the fallback column, which says for every cell whether a specialized loop ran or whether the kernel fell through to the row at a time path. The first record had 129 of its 618 cells falling through, and the difference is not a percentage. Comparing a dictionary column against a flat one had no loop at all and cost 83 nanoseconds a row against 1.2 for the dictionary against constant pair next to it, on the same data and the same operator. Both of the pairs that were missing have loops now, in tamnd/rudb#93 and tamnd/rudb#94, and this record is the first one where the column is empty on both machines. That is what the column is for in both directions: it found two loops worth forty times their cells, and a change that loses a specialization which already exists looks exactly the same and would be caught the same way, on the fact rather than on the time, because the answer stays right and only the time is wrong.

The filter table is the one that says a change was worth landing and where it cost something. On `gamingpc-wsl` at a full chunk, a predicate of one comparison is 1.00 times, which is what it should be because there is nothing before it to narrow what it reads. Two conjuncts that each keep three rows in four is 0.94 times, and that is a real regression rather than noise: the second conjunct saves a quarter of one comparison and that does not pay for the selection the threaded path builds and the indirection it then reads through. Four conjuncts at a fifth each is 2.34 times, and a predicate whose first conjunct rejects the chunk outright is 3.09 times, because the other three conjuncts never run at all. The shape of that is the whole argument for ordering the conjuncts by what they throw away, which is the next box on tamnd/rudb#57, and the 0.94 is the cell that will say whether the ordering is worth its own bookkeeping.

The vector size sweep is in the record from two machines now, and the second one changes the answer. `server3` is a virtualized EPYC at 2.0 GHz with 32 KiB of L1d a core, and its sweep from 256 to 4096 gets monotonically cheaper and never turns back up, which says nothing about L1d residency at all. `gamingpc-wsl` is an i9-13900K with 48 KiB of L1d on a performance core, it is three to four times faster on every row, and its interquartile ranges are under five percent against `server3`'s ten to twenty. On that machine the two kernel rows still do not turn: comparison of two flat columns goes 0.53, 0.49, 0.46, 0.47, 0.43 nanoseconds a row across the five sizes and addition goes 0.59, 0.48, 0.43, 0.40, 0.43. The two pipeline rows do turn. Select then one pass goes 3.09, 2.23, 1.97, 1.66 and then back up to 2.51 at 4096, and compact then one pass goes 5.18, 4.41, 4.06, 3.72 and then back up to 4.12.

That is the difference between a kernel and a pipeline stated in a measurement. One loop over one column streams and does not care how long the column is. A stage that holds a chunk, the rows it kept and the thing it compares them against does care, and the size where it starts caring is between 2048 and 4096 on a machine with 48 KiB of L1d. `server3` could not resolve that, not because its cache is smaller but because a ten to twenty percent spread cannot see a difference of a fifth. Neither machine is the 128 KiB L1d the original argument for a 1024 row vector was written against, so what this pair of records supports is the shape of the curve rather than the constant.

So the check has two kinds of finding and only one of them has a threshold. A cell that got twice as slow is the ordinary one, and the threshold is that high because these numbers move by a fifth between two runs of the same binary on the same machine. It is widened per cell by the interquartile range the record carries for that cell, because a cell that moves by half on its own cannot have a factor of two claimed about it. That rule is not a precaution: `compaction 2 bigint 1 select build` recorded 341 nanoseconds with a spread of 58 percent and the next two runs of the same binary gave 86.3 and 86.7, and a flat factor of two would have failed it the first time it landed on the high side. The other kind of finding is a cell that used to take a loop and now takes the row at a time path, and that fails on the fact rather than on the time, because it is worth catching on the commit that did it rather than on whichever later run happened to be slow enough.

Which machine the check runs on matters more than the threshold rule does, and the record now says so with a number. Over the 682 cells the mean interquartile range is 1.8 percent on `gamingpc-wsl` and 17.3 percent on `server3`, the worst cell is 54.4 percent against 235.3, and one cell on `gamingpc-wsl` is over 50 percent where 38 cells on `server3` are. A threshold widened by a spread of 235 percent is a threshold of six times, which is not a gate, it is a cell the gate has given up on. The one bad cell on `gamingpc-wsl` is the first filter cell that runs, the whole variant of the one conjunct predicate, and it is bad on both machines, which is what a cell that is paying for a cold start rather than measuring a loop looks like. So the widening keeps a bad measurement from failing an innocent change, and running the check where the measurements are not bad is what makes it catch anything. `server3` is the gate machine because it is the one that is always free, and it stays the machine the full build runs on, but the kernel check belongs on `gamingpc-wsl`.

### Sweeping a seam

A seam is a place in rudb where more than one implementation is defensible. The hash table, the sort, the way a column carries its values, the scheduler. There are twenty seven of them and `rudb_strategies()` is the engine's own list, which is also what `EXPLAIN` reads when it marks a node. The argument for having a registry of them at all is that the choice becomes a measurement rather than an opinion, and that argument only cashes out if there is a command that runs the same suite over the same files on the same machine with one seam moved and nothing else.

That command is `rudb-bench sweep --seam <seam> --suite <suite>`, and `rudb-bench seams` prints what there is to sweep. It is rudb only, which is not a gap: no other engine here has a seam to move and a column that is the same number in every row is not a column.

Today every sweep prints one row. Nothing has been registered at any of the twenty seven seams yet, so the only thing to run is the reference implementation, which is the slow one kept for differential testing. That is the apparatus working rather than a missing result, and it is the reason to build it now instead of on the afternoon somebody needs it. The day a second hash table lands, finding out whether it is faster is a command that already exists and already holds everything else fixed, rather than an afternoon of shell scripts whose results nobody else can reproduce.

```
$ rudb-bench sweep --seam hash.table --suite smoke --runs 3

seam      hash.table (F5)
          the hash table itself
suite     smoke, 6 queries
engine    rudb 0.2.36

implementation        hot total     hot cpu     peak RSS   worst IQR   vs first
reference                1.375s      1.240s    35.31 MiB      10.5%      1.00x
```

The reference always runs first, whatever order the engine lists the implementations in, because the ratio column is against the first row and a ratio against whichever implementation happened to be registered first would change meaning when somebody reorders a registration file. A variant that will not run is a row saying so rather than an abandoned sweep, since the interesting case is exactly the one where a new implementation is wrong on one query.

### What the optimizer was worth

`rudb-bench attribute --engine <name> --suite <name>` runs one suite twice on one engine, once as it comes and once with every optimizer it has turned off, and prints one row per query with the two times and the ratio. It is the sibling of the sweep and the same idea one level up: the sweep moves a single decision inside the engine and holds the rest fixed, and this turns the whole rewrite pipeline off and holds the rest fixed. Milestone E1 asks for it as an attribution row rather than as a claim, and the difference between those two is the whole point of the command.

A claim is "the optimizer makes rudb three times faster". An attribution is forty three rows, most of them near one and a handful that are not, with the query names attached. The second is the one that says where to work, and it is also the only one of the two that cannot be produced by running the suite until it flatters you. So the total prints with a sentence under it saying that it is a sum of the rows and not a claim about the engine, and the queries the optimizer made *slower* get a list of their own under the table, because that is the half nobody publishes and the half worth reading.

Which passes were turned off is asked of the engine through `duckdb_optimizers()` and never written down here, and the names are printed under the table. A harness holding its own copy of the list would leave a pass on the day the engine added one, and would report the optimizer as worth slightly less than it is with nothing in the output saying so. Both DuckDB and rudb answer that table function and both take the names back through `SET disabled_optimizers`, which is why `--engine` is a flag: attributing DuckDB the same way is the only way to find out whether a number this prints about rudb is a number about rudb or a number about the apparatus.

The command exits non-zero when a query answered differently with the passes off. That is the one result here that is a property of the engine rather than of the machine, and it is a bug rather than a timing, so the row says so instead of printing a ratio between two different answers. The gate that actually chases it down is the whole corpus run in `tamnd/rudb-compat`, and this only checks the handful of queries it happens to be timing, which is worth having anyway because a benchmark comparing a right answer against a wrong one is not slower or faster, it is meaningless.

This is not a quick command and it is not wired into any gate. The unoptimized half runs the plan the binder emitted, which for a join is a cross product filtered afterwards, and nothing in this harness has a clock on it. The first run of it, on `mba-m2`, took half an hour, and all but a minute of that was six runs of one query.

```
$ rudb-bench attribute --engine duckdb --suite smoke --hot 5

suite       smoke, 6 queries
engine      duckdb v1.5.5 (Variegata) d8cdaa33fd
optimizers  33 turned off

query       shape                     with     without  what it was worth
q1          count                 19.448ms    19.476ms  1.00x faster
q2          filter and sum        30.953ms    32.569ms  1.05x faster
q3          group by, low card    24.495ms    25.486ms  1.04x faster
q4          group by and top k    33.273ms    46.294ms  1.39x faster
q5          count distinct        36.806ms    38.212ms  1.04x faster
q6          join and group by     81.712ms    269.045s  3292.61x faster

 226.686ms with the optimizer, over 6 queries
  269.207s without it
this total is the sum of the rows and not a claim about the engine. The rows are the result
```

That is the argument for the per query column in one table. Five of the six rows are between 1.00 and 1.39, which is to say the optimizer is worth nothing measurable on a scan or an aggregate, and the sixth is 3292x. A total would have reported the optimizer as worth 1188x on the smoke suite, which is a number about q6 wearing a suite's name, and a suite with one more scan in it would have reported a different one for no reason that has anything to do with the optimizer. The rows do not have that problem: q6 is 3292x and the rest are noise, on this suite and on any other.

The same command on rudb, which is the engine this is for, runs five queries rather than six because rudb declines the join on the smoke suite until it has something better than a nested loop. The five it does run agree both ways, which is the exit code, and the spread is 1.53x to 49.99x:

```
$ rudb-bench attribute --engine rudb --suite smoke --hot 3

suite       smoke, 6 queries
engine      rudb rudb 0.3.31
optimizers  44 turned off

query       shape                     with     without  what it was worth
q1          count                 45.206ms      2.260s  49.99x faster
q2          filter and sum       501.877ms      1.655s  3.30x faster
q3          group by, low card      1.105s      2.466s  2.23x faster
q4          group by and top k      1.107s      2.578s  2.33x faster
q5          count distinct          1.650s      2.522s  1.53x faster

    4.409s with the optimizer, over 5 queries
   11.480s without it
```

Forty four names against DuckDB's thirty three, and rudb has nine passes. That is not a discrepancy, it is what the setting is: a corpus file that disables a pass rudb has not written yet has to be accepted rather than rejected, so `duckdb_optimizers()` answers with the whole DuckDB list. The printed line is therefore the truthful answer to what was turned off and is not a count of the optimizer's parts. Which of the nine did the work is a different question with its own command, `rudb-compat sweep` in `tamnd/rudb-compat`, which runs the corpus with pass k on and the rest off so that a difference localizes to one pass. This says what the pipeline was worth and that says which pass it was.

The 49.99x on q1 is the interesting row and it is the one to be suspicious of, because `SELECT count(*)` should not need an optimizer. What it says is that without projection pushdown rudb reads every column of ten million rows to count them, which is the pass that landed first in E1 and is doing exactly what it was landed to do. It is also a measurement of how much rudb's scan layer costs when nothing prunes it, which is milestone E2, and reading it as a fact about the optimizer rather than about the scan would be the wrong lesson from a right number.

### The plan gate

Everything above this line is a measurement, and the trouble with catching an optimizer regression by measurement is that a measurement is slow, noisy and run on the day somebody needs a number. A pass that quietly stops firing on one query shape does not crash. It shows up weeks later as a number that got worse, on a machine that is busy, in a run that touched forty commits.

So the plan is written down. `rudb-bench plans` asks rudb to `EXPLAIN` every query in a suite and compares what comes back to `baselines/plans-<suite>.txt`, which is committed and reviewed as a diff. It measures nothing, it runs in under a second, and it runs on every commit.

```
$ rudb-bench plans
every plan in smoke is the one that was written down
every plan in tpch is the one that was written down
```

It needs no data. A plan needs a schema to bind against and a zero row Parquet file carries its whole schema in the footer, so the tables it plans against are the empty ones in `fixtures/`, which are 36 KB for the whole of TPC-H and the smoke suite together. That is the decision that lets this be a CI job rather than a thing that needs the fifteen gigabyte corpus and therefore never runs where it was supposed to. What it costs is stated in `fixtures/README.md` and in the header of every baseline file: a pass that decides something from a row count is not exercised here the way a real run exercises it. Today that costs less than it sounds like, because rudb prints `rows unknown` on almost every node of these plans rather than an estimate, and join order is not among the nine passes it runs yet.

What is in the file is the shape, and the shape is where a pass shows up. This is q1 of the smoke suite, which is `SELECT count(*)`:

```
query     q1
  Project #3 [#2.0::BIGINT AS "count_star()"]  [~1 rows] [pipeline 0] [reference]
    Aggregate #2 groups=[] aggregates=[count_star()::BIGINT]  [~1 rows] [pipeline 1] [reference]
      Project #1 []  [rows unknown] [pipeline 1] [reference]
        TableFunction read_parquet args=['<fixture>/smoke.parquet'::VARCHAR] #0 []  [rows unknown] [pipeline 1] [reference]
```

The empty column list on the `TableFunction` is projection pushdown, and it is the same pass the attribution above measured as 49.99x on this query. The point of having both is that the attribution says what it is worth and needs ten million rows and a quiet machine to say it, and this says whether it is still happening and needs neither.

The absolute path of the checkout is settled out before anything is written down, because rudb inlines a view and a baseline holding one machine's path is a baseline that fails for everybody else. The trailing seam block is dropped too: it is byte identical in every query, so keeping it would put twenty one copies of one fact about the build in a file and make registering a seam look like every query in the suite replanning.

A query that will not bind is a line rather than an absence, and that turned out to be the first thing the gate found. rudb declines to run TPC-H, because every join in it is a nested loop and timing that would be timing a hang, so nobody had ever found out which of the twenty two queries rudb can bind. Planning executes nothing, so the answer is now in the file and is checked on every commit:

```
planned   21 of 22
refused   q11  Binder Error: column a column must appear in the GROUP BY clause or must be part of an aggregate function
```

Twenty one of twenty two, for the price of eight empty Parquet files. A query that stops binding is a regression and a query that starts binding is progress, and both of them are a diff here. The doubled word in that error is rudb's, and it is a second thing this found: the message substitutes an empty column name into a sentence that already says "column".

The gate fails on any change at all, including a change somebody meant. A plan that moved on purpose wants `rudb-bench plans --suite <name> --record` in the same pull request as the change that moved it, which is the whole point: the diff gets reviewed next to its cause. A category of plan change that only printed a warning would be the category people stop reading.

ClickBench has no baseline yet and that is a gap rather than an oversight. The only `hits` schema this repository has written down is the one after the suite's own conversion, and rudb reads the raw file and applies that conversion in a view, so a fixture built from what is written down would be a table that does not exist on disk anywhere. `fixtures/README.md` says what the honest fix is and it is one command on either of the two machines that has the file.

### The planning budget

The gate above watches what the optimizer decided. This one watches what deciding it cost.

Every other number in this harness is about the part of a query that moves rows. Planning is the part before any row moves, and until rudb 0.3.36 it was not measured at all: parsing, binding and optimizing happen above the code that writes the metrics document, nothing up there was on a clock, and all three fields were zero in every document the engine had ever written. The total was the physical build plus the run, so planning could grow without a single number going up anywhere.

That matters because of how an optimizer changes. It is only ever added to. Each pass arrives with a measurement showing it paid for itself on the query somebody wrote it for, and no pass arrives with a measurement of what it costs on the queries it does nothing for. The pass that saves four hundred milliseconds on a scan of ten million rows costs its two hundred microseconds on a point lookup too. Twenty passes later the point lookup plans for longer than it runs, and nobody has found out, because the query still returns the right answer and every profile anybody opens starts at the first operator.

So planning is a column of its own next to the runtime, and `run <suite> --check` fails a query that spent a larger share of itself planning than `baselines/planning-<suite>.txt` allows.

```
baselines/planning-smoke.txt
  rudb
    q1          2.66% of 3.50% allowed   1109us planning, 40639us running
    q2          0.57% of 2.34% allowed   1465us planning, 256651us running
    q3          0.37% of 2.23% allowed   1177us planning, 320015us running
    q4          0.26% of 2.15% allowed   1082us planning, 421870us running
    q5          0.35% of 2.31% allowed   1458us planning, 412884us running
```

The microseconds are the more interesting half of that. Planning is about a millisecond whatever the query is, because the work it does is proportional to the size of the statement and these five statements are all about the same size, while what they run over differs by a factor of fifteen. That is the shape of the problem: the cost of planning does not shrink when the query gets small, so the share is largest exactly where the absolute number matters least, and a pass added for the benefit of q5 is paid for by q1.

The budget is a share and not a duration, and that is the whole design. A committed budget of nine hundred microseconds is a fact about the machine it was taken on, and checking it somewhere else breaks reporting rule seven. A share survives the trip, because planning and execution are two spans of the same run of the same query read off the same clock seconds apart, so the ratio between them is a property of the engine in a way that neither number is on its own. It is not perfectly portable, since planning does not get faster with more cores and execution does, but it errs the safe way: a slower machine runs the query for longer, which makes planning a smaller share, so a budget carried somewhere slower gets looser rather than crying wolf. The microseconds are printed beside the shares because eight percent of a two millisecond query is not the same news as eight percent of a two second one, and they are printed rather than stored for exactly the reason the share is stored rather than them.

The share is taken over every run of the query and the middle one is the one recorded, which is the only part of this that had to be found by measuring rather than by arguing. The obvious thing to do is read the breakdown the harness already keeps, which is the cold run's, and five recordings of smoke q1 done that way gave 1.7%, 1.7%, 2.0%, 14.9% and 41.1%. Planning had not moved: it was between 795 and 1662 microseconds in every one of them, as it was for every other query. What moved was the denominator. q1 is a count that the engine answers out of the file's directory, so its execute span is sometimes a millisecond and the ratio is then almost entirely noise. A gate recorded off a sample of one fails one run in five for a reason that has nothing to do with the optimizer, and a gate like that gets switched off. The median is used rather than the worst because the worst of sixteen runs is a sample of one again, picked adversarially, and a ceiling recorded off it is a ceiling nothing ever comes near.

Planning here is the whole statement less the execute span, rather than the three planning phases added together. The difference is the physical build, and leaving the build in is deliberate: work moved out of the optimizer and into the builder would otherwise meet the budget by changing which side of a line it sits on. What this measures is every nanosecond before the first row moved, and the only way to make it smaller is to do less.

The recorded share is kept in the file next to the ceiling it produced. A file of ceilings alone cannot be reviewed, because nobody reading four percent can tell a budget that was just met from one with three times the headroom it needs, and so nobody can tell whether a diff raising one is somebody accepting a regression. The ceiling is the recorded share plus a quarter again, or plus two points of percentage, whichever is larger, because a proportional margin on a number that is half a percent is not a margin.

A query the budget has never heard of is reported and never failed. Otherwise the commit that adds a query to a suite has to re-record the budgets, and a re-record that happens every time is a diff nobody reads.

### The attribution ledger

Section 2.8 of the engine specification asks for a series rather than a table. Each layer of the engine closes with a row, the row carries the before and the after on total time, CPU seconds, peak resident and bytes read on the same machine, and the release notes are written from it. The argument for it is that a release note claiming the hash table made joins faster and unable to point at a row is a release note that is guessing.

`rudb-bench run smoke --store "2a the baseline"` appends what just ran to `runs/smoke.txt`, which is committed, and `rudb-bench ledger` renders it. Nothing in the ledger is written by hand, because a hand maintained ledger is one that acquires a good number nobody can reproduce. The first row is in:

```
$ rudb-bench ledger
[2a the baseline] smoke on server3
closed by harness 598e065 on 2026-09-10
against nothing, this is where the series starts on this suite and machine

engine                        hot      hot cpu     peak RSS         read         load      on disk
duckdb                  766.122ms       2.110s   125.62 MiB          0 B       1.458s    92.01 MiB
clickhouse-local           4.445s      13.000s   379.86 MiB          0 B       1.957s    49.49 MiB
datafusion              951.643ms       2.140s   172.48 MiB          0 B      0.000us    10.41 MiB
polars                     3.007s       5.570s   190.71 MiB          0 B      0.000us    10.41 MiB
clickhouse-server          1.896s     not read     not read     not read       6.678s    50.44 MiB
```

rudb is not in it, because rudb cannot run a query yet, and the harness says not built did not run rather than printing a blank. That is what the first row of a series looks like and it is worth committing anyway: every later row is a ratio against something, and this is the something.

Later rows print the change under each number. What matters more than the change is the line above it: when a rival's version moved between two rows the ledger names it and says that a ratio across that boundary is a ratio between two different comparisons. That is the failure this file exists to catch, which is a layer taking the credit for DuckDB shipping a release in the middle of it.

`runs/<suite>.txt` and `baselines/<suite>.txt` look alike and are not interchangeable. A baseline record is per query and holds three quartiles, because the gate asks whether one query got twice as slow. A run is per engine and holds totals, because the ledger asks what a layer bought across a suite. Records are replaced when they are retaken and runs never are, because a history that overwrites itself is a table. `baselines/kernels.txt` is a third shape again and is one block per machine rather than one per query, because a kernel has no query to be per.

## Reports

A milestone whose exit criterion is a measurement publishes the measurement here, in `reports/`, rather than leaving it in a comment thread on the issue. The rule is that a report names the number it was supposed to hit before it gives the number it hit.

[`reports/2026-09-11/m1-the-format-experiment.md`](reports/2026-09-11/m1-the-format-experiment.md) is the first one. It is the storage format measurement from M1: a standalone encoder over ClickBench `hits`, TPC-H at scale factor 100 and three real Parquet files off the internet, answering whether shared dictionaries, shared symbol tables and recomputation rules deliver the ratios the specification assumed. The target was `hits` under 4 GB and the line at which the resource claim was declared wrong was 6 GB. It came out at 9.65 GB, the claim was wrong, and the specification was amended. The two techniques with no equivalent in DuckDB were worth 4 MB of it, and front coding a sorted dictionary, which any format could adopt tomorrow, was worth 2 GB.

The other kind of file in `reports/` is written by the harness rather than by a person. `rudb-bench run clickbench --report` prints the usual table and then writes `reports/<yyyy-mm-dd>/run-<suite>-<machine>.md`, which holds everything the table had to drop. The terminal gets one hot median a cell because five engines and 43 queries have to fit a terminal. The file gets the cold time next to the hot one, the interquartile range, the first and third quartiles, the fastest and slowest run, CPU seconds, cores used, peak resident, bytes read, rows a second and bytes a second, per engine and then per query, under the machine record that was taken at the time. The name carries the machine because reporting rule seven says never compare across machines and two files that differ only in which box they were taken on would be compared by the first person who opened them side by side.

It is a flag and not the default. A run that wrote into the checkout every time is a run people stop starting from the checkout, and `RUDB_BENCH_REPORTS` moves the directory for a run somewhere without one. A report that cannot be written warns and does not fail the run, because the numbers are already on the terminal by then and the alternative is throwing away an hour of ClickBench over a directory that was read only. The generated markdown is checked against the same prose rules as everything else here, in a unit test over the generator rather than only in the test that walks the committed files, so a generator that started emitting em dashes fails on the commit that changed it rather than on the build after somebody committed a report.

### One engine at a time, and the table afterwards

A full ClickBench over the fourteen gigabyte file is hours per engine. Six engines in one command is a day that falls over on the fifth one and leaves nothing behind, and a machine that has to hold six engines' storage formats at once needs a lot more disk than the largest of them. So the engines are measured one at a time.

`rudb-bench run clickbench --engines rudb --save` measures one engine and writes what it measured into `reports/saved-<suite>-<machine>.txt`. Run it again tomorrow for the next engine and that engine is added. Re-run one that has already been measured and its block is replaced, so an engine that got faster overnight gets its new number and the four that did not move keep the numbers they earned. `rudb-bench report clickbench` then builds the cross engine table out of everything saved so far, and it measures nothing itself, so it takes a second rather than a day.

What is saved is the measurement rather than a summary of it. Every sample of every run is in the file, so a result that comes back out of it has the interquartile range it had going in and every publication rule applies to it exactly as it did on the day. A saved median would have been smaller and would also have been the end of rule two, which says the spread travels with the median.

Two things this refuses to do. It will not put two engines in one table when they ran over different cuts of the data, because the smaller one comes back four to eight times faster and a table like that reads exactly like a table where one engine is much better. And the machine is in the filename rather than in a column, because rule seven says never compare across machines and one file holding two of them would be one flag away from doing it. An engine with nothing saved is a sentence under the table naming what would save it, not a gap in the row.

The cross engine table built this way carries no machine record, and that is deliberate. The machine facts are read off the box as it is now and the saved runs were taken over days, so printing today's governor next to Tuesday's numbers would be a claim about Tuesday that nobody made. The per run report each engine wrote on its own day keeps the machine it ran on.

### Two clocks, and which one to read

Every number in a report comes from one of two clocks and the report always shows both.

The wall clock is this harness's. It starts before the engine's process does and stops after the process has exited, so it pays for the fork, the dynamic linker, opening the database, running the query and printing the answer. The query time is the engine's own. Each engine is asked in its own way: DuckDB and rudb get `.timer on` and print `Run Time (s): real`, both ClickHouses get `--time`, `datafusion-cli` prints `Elapsed` when it is not told to be quiet, and the Polars script times itself around the execute and the sink because Polars has no shell to ask. That is the number the public ClickBench board publishes, and asking for it is what makes a column here comparable to a column there.

Asking is one line of code an engine at a time and getting it wrong is silent, which is worth a warning. DuckDB and rudb print the same `Run Time (s): real` line under the same `.timer on`, and DuckDB prints it on stdout while rudb prints it on stderr. Nothing else about the two differs. The first version of this read stdout only, so rudb had no clock of its own and fell back to the wall clock while DuckDB did not, and the table then compared a DuckDB query against a rudb query plus a process start. That is the same unfairness this whole path exists to remove, pointing at us instead of at them, and it made rudb look better than it is by most of a factor. Both streams are read now and there is a test that says so.

They can be very far apart. A ClickBench `COUNT(*)` over a hundred thousand rows is two milliseconds by DuckDB's own clock and about fifty by ours, so 96 percent of the wall clock is a process starting. That would be harmless if it were the same for everybody, and it is not: DuckDB starts in about forty milliseconds, `clickhouse local` in about three hundred, and Polars has to boot a Python and import itself before it can look at the query. A table of wall clocks over a small sample partly ranks process startup and calls it a ranking of query engines, which is the specific mistake this pair of columns exists to stop.

So compare engines on query time and read the wall clock to know what a run costs to sit through. Every ratio and both throughput columns are taken against query time wherever every engine reported one. The `overhead` column is the gap as a fraction of the query time, and it is the number that tells you how much of a wall clock you are allowed to believe: under about a tenth it does not matter, over one and most of the wall clock column is this harness.

Nothing is netted out. Subtracting an estimate of overhead from a measurement is how a harness starts reporting what its author expected, so both clocks are printed and neither is adjusted by the other.

## The reporting rules

These apply to the README, release notes, the dashboard, any talk, any post, and any conversation. They are in the engine's specification in full and this is the short version.

**State the comparison exactly.** Which rudb commit, which DuckDB version, which ClickHouse version, which machine, which kernel, which filesystem, which settings. "10x faster than DuckDB" is not a claim, it is a mood.

**Report the distribution, not the best run.** Median of at least five runs with the interquartile range. Never a minimum, never a single run. ClickBench's own convention of best-of-three is used for ClickBench because comparability with the public board matters more than rigor there, and any number in that form is labelled as ClickBench-convention.

`--runs 1` exists and does not break that rule. The rule is enforced where the number would be published rather than at the flag: a distribution of fewer than five samples says no to `publishable`, so a run that small cannot go into a record, cannot be stored in the ledger, and carries the reason in its own report. What it can do is finish in a fifth of the time, which is what you want while you are changing the engine and not what you want on the day you write something down.

**Report the whole suite including the losses.** Every query, in a table, including the ones where we are slower. A geometric mean with no per-query table is not a result.

**Report cold and hot separately.** Cold is what a user's first query does.

**Report load time and on-disk size with every runtime result.** A runtime win paid for with a 10x load time is a different product than it appears.

**Report peak resident memory with every runtime result.** A query that is fast because it used 30 GB is not fast, it is expensive.

**Never compare across machines**, and never compare a number measured today against one measured on different hardware six months ago.

**Any published number is reproducible by one documented command** on a named machine type, and the command lives in the same artifact as the number.

**State the mode.** Native format, attached DuckDB file and Parquet in place are three different products with three different performance profiles. A number without its mode is not interpretable.

**A micro-benchmark number never appears without the end-to-end number it is supposed to explain.** A 20x kernel improvement that moves the query by 3 percent is an engineering note, not a result.

## The machines

The primary reporting machine is `c6a.4xlarge`, 16 vCPU and 32 GiB, because that is what the ClickBench board uses and comparability is worth more than picking a machine that flatters us. A large machine of at least 64 cores is reported alongside, because an engine that is fast at 16 threads and does not scale to 128 is a different product. A small machine of 4 cores and 8 GiB is reported alongside, because the embedded case is frequently a laptop and because the resource claim means most where resources are scarce. ARM is reported, on Graviton and on Apple silicon, because a SIMD story that only works on x86 is half a story.

Frequency policy, turbo state, page cache handling, filesystem and mount options are recorded in the result artifact rather than assumed. Nothing is measured on a shared CI runner, ever, and the CI in this repository deliberately runs no benchmarks: a timing from a shared virtual machine is not a timing.

### The machines we actually have

Those are the machines a published number comes from. They are not the machines the project owns. `rudb-bench fleet` prints what is really here, which is four boxes across three operating systems and none of them is a `c6a.4xlarge`.

```
$ rudb-bench fleet
Published numbers come from c6a.4xlarge, 16 vCPU, 32 GiB, gp2.
Nothing below is that machine. Reporting rule seven: never compare across machines.

machine       role         cores/threads   memory   free disk   clickbench
gamingpc-wsl  regression       24/32         31 GiB    748 GiB   yes
gamingpc      regression       24/32         63 GiB    249 GiB   no
server3       regression        8/8          23 GiB    138 GiB   no
server2       correctness       6/6          11 GiB     44 GiB   no
server1       correctness       4/4           5 GiB    185 GiB   no
```

The last column is a separate question from the role and it is separate on purpose, because running the two together is how a machine gets picked for a suite it cannot hold. Running a full ClickBench needs three things at once: room for the largest engine's copy of `hits`, which is DuckDB's at about 20 GB, a load average low enough that a timing is a timing, and all five engines available on the machine at all. Exactly one row has all three, and there is a test that fails if that count ever stops being one, because zero means the next layer of the engine has no base number to be a ratio against and two means somebody has to say which of them the series continues on.

The distinction is structural rather than a note somebody stops reading. A machine carries a role, a role says whether a number from it may be published, and nothing in the fleet may. There is a test that fails if that stops being true, so adding a reporting machine is a conversation about whether it really is one.

What these are for is the other job a benchmark does, which is telling you on a Tuesday that the change you merged on Monday cost eight percent. That job wants the same machine over time much more than it wants the right machine, and these are the same machines over time.

Each one has a caveat that is worth knowing before trusting anything from it. `gamingpc-wsl` and `gamingpc` are one desktop seen from its two operating systems, and they are separate rows because a number from one is not a number from the other. The Linux side is where a full ClickBench happens, since ClickHouse has no Windows build and a four engine ClickBench missing the second fastest engine on the board is not the comparison. Four things travel with any number from it: it is a virtual machine, so the disk is virtual and the memory is 31 of the host's 63 GiB, its `/tmp` is a 16 GiB tmpfs so a scratch directory left on the default lands in RAM, its cores are heterogeneous at 8 performance and 16 efficiency so a thread on the wrong kind is a 2x outlier with nothing to do with the change under test, and the host is a desktop somebody uses. The Windows side runs the suites that are about the engine rather than about the comparison, which matters because the storage layer has to work there and nobody finds a Windows file locking problem on a Mac. `server3` is the quietest Linux box and the default for the gate and for a suite that fits, which ClickBench does not, though the reason moved. Its free disk has read 44 GiB, then 21, and 138 today, and for a while the table said the 21 was what ruled ClickBench out, which is the argument for probing these numbers rather than trusting the table. The disk is no longer the constraint. The engines are: `server3` has DuckDB and Polars and neither ClickHouse nor DataFusion, and a ClickBench row missing two of the four engines is not the comparison this report publishes. `server2` and `server1` do correctness and compatibility work, and `server2` has DuckDB alone on it, which settles the same question there. `server1` has about 2 GiB of memory free, which makes it useless for timing and genuinely useful for the resource claim, since an out of memory there is a real finding.

`server1` deserves a stronger warning than that, and it was earned the hard way. It has the most free disk of any Linux box here, 185 GiB, and it is the only Linux box other than `gamingpc-wsl` with all four comparison engines installed, so on paper it is the obvious place to run ClickBench and it was the machine picked for the first run. The smoke run it produced came back with interquartile ranges between 28 and 124 percent of the medians. The cause was not the harness. `server1` is a Kubernetes production worker node: kubelet, containerd, cilium-agent, a temporal server, a harbor registry and an otelcol are all resident on it, alongside a long running crawler, and its load average on 10 September 2026 was 51.72, 75.13 and 70.80 over one, five and fifteen minutes on four cores, with the one minute figure still at 35.30 two days later. Nothing timed on a machine loaded eighteen times over is a timing, at any sample count, and no threshold rescues it. That is the whole reason `server1` is disqualified, and it is a load problem rather than a capacity one, which is worth saying plainly because the capacity looks fine. What is left is the Linux side of `gamingpc`, at 32 threads, 31 GiB and 748 GiB free with every engine on it and a load average near one, and that is where ClickBench runs.

## Who we measure against

DuckDB at the version compatibility is claimed against, in its native format. This is the primary comparison.

ClickHouse, because it is the fastest widely deployed single node analytical engine. Any claim of being the fastest that does not beat ClickHouse is not a claim of being the fastest.

Umbra where a binary is obtainable, because it is the current leader. Where a binary is not obtainable its published board numbers are cited as published board numbers with the date, and never mixed into a table of numbers we measured.

DataFusion and Polars, because they are the Rust ecosystem's answer and because being ahead of them is a floor rather than an achievement. Polars is measured in `sink` mode rather than out of a collect, so the query writes its answer as it goes and never holds it in one piece. That is what the out of core comparison is about, and it is also what makes the peak resident column in that row mean the same thing it means in the others. On smoke it is worth about 14 MiB of peak and nothing outside the noise on time, because these six answers are small. On a suite whose answer does not fit it is the difference between a measurement and an out of memory.

Vortex-backed DuckDB and DataFusion specifically, because they are the closest published thing to this design and their result is negative: DuckDB with Vortex measured 40.99 seconds against DuckDB's own 26.25. Tracking them is how we find out whether we have avoided their failure mode or reproduced it.

## Building

```
git clone https://github.com/tamnd/rudb-bench
cd rudb-bench
cargo build --release
cargo test
```

## License

Apache-2.0. See [LICENSE-APACHE](LICENSE-APACHE).

ClickBench is by ClickHouse Inc. TPC-H and TPC-DS are by the Transaction Processing Performance Council. Numbers produced here are not audited TPC results and are not claimed to be.
