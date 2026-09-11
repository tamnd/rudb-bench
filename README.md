# rudb-bench

The benchmark harness for [rudb](https://github.com/tamnd/rudb).

ClickBench, TPC-H, TPC-DS, JOB, CEB and H2O against DuckDB, ClickHouse, Umbra and DataFusion, and the reporting rules that decide what a number is allowed to claim.

It is a separate repository so that a result can be reproduced by someone who does not trust us, without building the engine from a specific commit of the engine's own repository. The whole project's claim is a performance claim, which means its credibility rests on the honesty of these measurements more than on any technical decision inside the engine. A benchmark number without its methodology is marketing.

The design is [`spec/15-rudb-bench.md`](https://github.com/tamnd/rudb/blob/main/spec/15-rudb-bench.md) in the rudb repository.

## Status

Early, and running. rudb cannot run a query yet, so what gets measured today is five rivals against each other: DuckDB, ClickHouse twice, DataFusion and Polars, on a generated smoke dataset, on the real ClickBench `hits` and on TPC-H at scale factor 100. That is not a placeholder. Every reporting rule below is a property of the apparatus rather than of the engine, and every one of them is easier to build now, against engines nobody here has a stake in, than on the afternoon somebody wants a headline.

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

The last two are more interesting than they look. q4 is `AVG(UserID)` over a hundred million bigints near ten to the eighteenth, where the order the partial sums are added in moves the result further apart than the one part in a billion this harness calls the same number, so ClickHouse disagrees with DuckDB about an average that is not wrong on either side. q43 is a timezone. ClickHouse renders a `DateTime` in the machine's timezone and DuckDB renders a `TIMESTAMP` in none, so the same epoch second prints as `2013-06-23 22:06:40` on `gamingpc-wsl`, as `2013-06-23 17:06:40` on `server3` and as `2013-06-23 15:06:40` out of DuckDB anywhere. That is not only a comparison problem. It means the ClickHouse column's answer to q43 depends on which machine ran it, which is worth knowing before somebody writes a compatibility test against one.

So seventeen of the forty three are named in `CLICKBENCH_UNSETTLED` with the reason, and they are reported under the table as answered differently with the data not saying which is right, rather than as disagreements. The cost is real and the README should say it plainly: a wrong answer from rudb on any of those seventeen would go unnoticed here. Twenty six are still checked to the last significant digit of a double, the whole smoke suite is still checked because it was written with a total order on every query, and catching a wrong q33 is a job for the differential harness in `tamnd/rudb-compat`, which can compare against one engine on data it controls, rather than for a benchmark comparing five.

### ClickBench is five query sets, not one

The official ClickBench repository keeps a `queries.sql` per engine and they are not copies of each other. Running one engine's text against all of them would be a measurement of somebody's translation, so this harness carries the official text per engine and says in the query table which engine gets which, and why.

Most of the difference is spelling. DataFusion's file quotes every identifier, because it lowercases the ones nobody quoted and every column in `hits` is camel case. Two of the forty three are not spelling. q28 and q29 average a string length, DuckDB's `STRLEN` counts characters and ClickHouse's `length` counts bytes, and `hits` is full of percent encoded UTF-8 where those are different numbers. Both boards are right about their own engine, so both texts are here and the report says the two columns disagreed on those two answers, which is the true thing to say about them.

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

### Three engines, three opinions about how wide a decimal is

The first two TPC-H runs both reported q01 and q08 as disagreements, thirty two numbers against thirty two and four against four, which is the shape of a report saying every row is there and every number is wrong. Running the two queries by hand against all three engines says otherwise. Every engine agrees on the answer. They disagree about the result type.

`avg` over a `DECIMAL(15,2)` is a double in DuckDB, a decimal of scale six in DataFusion and a decimal of scale four in ClickHouse. So the average quantity in q01 comes back as `25.499370423275426`, `25.499370` and `25.4993`, which are the same number printed to three widths. q08 divides two decimal sums and does the same thing, worse: DuckDB says `0.039535108776109315`, DataFusion says `0.03953510` and ClickHouse says `0.0395`. A relative tolerance of a part in a billion calls all three pairs different, correctly, because they are different to a part in a billion and there is nothing in the text that says they should not be.

So the comparison now allows two numbers to differ in the last place either of them was printed to, which is `10^-f` for `f` decimal places and is absolute rather than relative. The wider of that and the old relative tolerance wins, so nothing that used to pass now fails. The weakening is real and applies only to the decimal places: a number printed with no decimal point at all is compared at the relative tolerance alone, so a count off by one in a hundred and forty eight million is still caught, and a ratio of 0.041 against everybody else's 0.0395 is still caught however few places anybody printed.

This is a different kind of concession from the seventeen unsettled ClickBench queries and it is worth keeping the two apart. Those are queries whose answer the data does not determine, so no checker can ever check them. These two are queries whose answer every engine got right and the checker was reading the printing rather than the value.

### ClickHouse is two rows, because it is two systems

`clickhouse local` and a ClickHouse server are not the same engine wearing different clothes. The local one reads the Parquet where it lies and has no sorting key, no primary index and no chance to have merged anything. The server owns its data in MergeTree parts ordered by the key from the suite's own `create.sql`, which for ClickBench is the key ClickHouse Inc. publishes in theirs. Measuring only the local one and calling the result ClickHouse would flatter us, and by a lot: over three runs of the smoke suite on server3 the local row totalled between 3.5s and 5.3s hot where the tuned server totalled between 1.6s and 2.3s, so the engine our headline claim is stated against is somewhere around twice as fast as the row that was standing in for it. The harness now runs both and prints both, and the sorting key costs one or two percent on disk, 50.60 MiB against 49.49 MiB for the same ten million rows.

Both ClickHouse rows get the column types out of the official `create.sql`, and only the tuned one gets the sorting key. That split is deliberate. A sorting key is tuning and belongs to the row that exists to show what tuning is worth. A schema is not tuning: asking ClickHouse to infer one from the Parquet gives every column back as `Nullable`, which costs a null map per column and turns several things off, and nobody deploys that. DuckDB never paid it, because its `CREATE TABLE AS SELECT` out of the same file gets the real non-null types, so an inferred ClickHouse schema was a handicap this project had put on ClickHouse rather than a difference between the engines. All 105 `hits` columns are copied out of `clickhouse/create.sql` down to the generic spellings it uses, since writing `Int64` where the file says `BIGINT` would be a diff against the thing this is meant to be a copy of.

Reading the server's on-disk number needs care, and getting it wrong is easy. `OPTIMIZE TABLE ... FINAL` writes a new part and leaves the parts it replaced on disk as inactive until `old_parts_lifetime` expires, which is eight minutes by default, so a directory size taken straight after the load counts the data roughly twice. That first read said 128.03 MiB and would have been published as the sorting key costing 2.6x. Both ClickHouse rows now ask ClickHouse instead, with `select sum(bytes_on_disk) from system.parts where active`, and fall back to the directory only when that answer does not parse.

### A column that is flat is not a column about the queries

The tuned server row came back with q1 count at 222ms, q2 at 268ms, q3 at 267ms, q4 at 280ms, q5 at 296ms and the q6 self join at 378ms. A count and a self join over ten million rows do not cost the same, so that column is not measuring the queries: whatever all six have in common is bigger than the difference between them, and here it is `clickhouse client` starting up and connecting. This is the same finding as the interquartile ranges below, arrived at from the other direction, and it is worth catching by machine rather than by eye.

So a result now reports its shape spread, the hot median of its slowest query over the hot median of its fastest, and a column under 2x is named in the report and refused for publication. On the run above duckdb spread over 4.61x and datafusion over 5.83x, which is what a column that is measuring the queries looks like, while clickhouse-local at 1.50x, clickhouse-server at 1.70x and polars at 1.24x were all flagged. Read a flagged column as an upper bound on the engine and not as a measurement of it. Every one of those three starts a process per query whose startup is tens or hundreds of milliseconds, which is the same cause the regression gate section below arrives at, and the fix for all of it is the in-process measurement path at sub-milestone 2b.

### Every engine gives its disk back as soon as it is measured

Three of the five engines convert the Parquet into a format of their own before they run anything, and on ClickBench those copies are roughly 20 GB for DuckDB, 13 GB for `clickhouse local` and 12 GB for the ClickHouse server. A harness that loads all of them and holds them until the report is written needs the sum of every engine's format free on the disk, about 45 GB, when the run itself never needs more than the largest one at any moment. That is not a detail. It decided which machine the suite runs on, and it is still worth having on the machine with 501 GiB free, because a run that needs the largest engine's copy rather than the sum of all of them is a run that can be reproduced somewhere other than here.

So `Engine::unload` exists and the comparison calls it the moment an engine finishes, on the failure path as well as the success one. The comparison already ran one engine to completion before starting the next, and everything the report prints about an engine's size was read at load time and is sitting in its result by then, so nothing in the table changes and peak disk drops from the sum to the largest. The default implementation does nothing, which is correct for DataFusion and Polars: they read the Parquet where it lies, they never made a copy, and there is nothing of theirs to remove.

What this does cost is the afternoon where somebody wants to open an engine's database after a run and look at what it built. `RUDB_BENCH_KEEP` turns the whole thing off for that. It is set deliberately and never by default, because a debugging convenience that quietly triples the disk a run needs is a debugging convenience that stops the run.

### The regression gate, and what it does not catch yet

`rudb-bench run smoke --record` writes what just ran into `baselines/smoke.txt`, which is committed. `rudb-bench run smoke --check` runs the suite again and fails when a query's median moved past two times the recorded one and its samples no longer overlap the recorded ones. `--check-drift` is the same at ten percent and refuses to run against a record from a different machine, because ten percent between two machines is a fact about the machines. Both take fifteen hot runs per query rather than the five a person reading a table gets, since the interquartile range of five samples is the gap between the second and the fourth of them.

A query that swung wider than ten percent on the day is reported and never failed, which is section 13.8 of the engine specification and is the difference between a gate people keep and a gate people turn off. On the smoke suite today that rule declines almost everything, and the cause is measured rather than guessed. Every query here runs as a fresh subprocess on purpose, so that no engine gets a warm allocator and a warm buffer pool the others paid for. On queries of tens of milliseconds that means most of what gets timed is the process start. Fifteen runs of `duckdb -c "select 1"` on an idle server3, with no query attached to them at all, spread from 30ms to 60ms, an interquartile range of 37 percent of their own median. Fifteen runs of `clickhouse local --query "select 1"` spread from 180ms to 510ms, which is 48 percent.

So the gate prints how many queries it actually compared on every run, and says plainly that a run which compared none of them should be read as a red rather than a green. The fix is not a bigger threshold. It is to stop timing `execve`, which is the in-process measurement path at sub-milestone 2b.

### The kernel record

`rudb-bench kernels` is the in-process measurement the paragraph above says is the fix. It does not start an engine and it does not send anybody SQL, because a kernel is a function over a vector that takes microseconds and the only way to time one is to link it and call it. The measuring code therefore lives in the rudb repository, as `cargo xtask kernels`, where it can be compiled against the working tree instead of against the last published crate. This subcommand runs that task with `--json`, puts a machine name on what comes back, writes it to `baselines/kernels.txt`, and compares the next run against it.

The record is 654 cells over six tables: comparison and arithmetic for every physical layout in every form pair at zero, one and fifty percent nulls, string comparison where the prefix decides against where the prefix ties, the select against compact surface swept over selectivity by chunk shape, the vector size sweep from 256 to 4096, and the expression table, which is the one thing in the record that is not a kernel. That last one times a bound expression walked as a tree against the same expression prepared once, at a full chunk and at sixty four rows, which is what a chunk looks like after a selective filter. On `gamingpc-wsl` at rudb 0.2.3, comparison of two flat columns with no nulls costs 0.055 nanoseconds a row on a boolean, 0.054 on an 8 bit integer, 0.13 on a 32 bit one, 0.46 on a 64 bit one and 1.20 on a double. Those are per row costs of one loop and they are not a query result, so reporting rule ten applies to every one of them: a kernel number explains a query number and is never quoted as one.

The reason to commit them rather than read the table and move on is the fallback column, which says for every cell whether a specialized loop ran or whether the kernel fell through to the row at a time path. The first record had 129 of its 618 cells falling through, and the difference is not a percentage. Comparing a dictionary column against a flat one had no loop at all and cost 83 nanoseconds a row against 1.2 for the dictionary against constant pair next to it, on the same data and the same operator. Both of the pairs that were missing have loops now, in tamnd/rudb#93 and tamnd/rudb#94, and this record is the first one where the column is empty on both machines. That is what the column is for in both directions: it found two loops worth forty times their cells, and a change that loses a specialization which already exists looks exactly the same and would be caught the same way, on the fact rather than on the time, because the answer stays right and only the time is wrong.

The vector size sweep is in the record from two machines now, and the second one changes the answer. `server3` is a virtualized EPYC at 2.0 GHz with 32 KiB of L1d a core, and its sweep from 256 to 4096 gets monotonically cheaper and never turns back up, which says nothing about L1d residency at all. `gamingpc-wsl` is an i9-13900K with 48 KiB of L1d on a performance core, it is three to four times faster on every row, and its interquartile ranges are under five percent against `server3`'s ten to twenty. On that machine the two kernel rows still do not turn: comparison of two flat columns goes 0.53, 0.49, 0.46, 0.47, 0.43 nanoseconds a row across the five sizes and addition goes 0.59, 0.48, 0.43, 0.40, 0.43. The two pipeline rows do turn. Select then one pass goes 3.09, 2.23, 1.97, 1.66 and then back up to 2.51 at 4096, and compact then one pass goes 5.18, 4.41, 4.06, 3.72 and then back up to 4.12.

That is the difference between a kernel and a pipeline stated in a measurement. One loop over one column streams and does not care how long the column is. A stage that holds a chunk, the rows it kept and the thing it compares them against does care, and the size where it starts caring is between 2048 and 4096 on a machine with 48 KiB of L1d. `server3` could not resolve that, not because its cache is smaller but because a ten to twenty percent spread cannot see a difference of a fifth. Neither machine is the 128 KiB L1d the original argument for a 1024 row vector was written against, so what this pair of records supports is the shape of the curve rather than the constant.

So the check has two kinds of finding and only one of them has a threshold. A cell that got twice as slow is the ordinary one, and the threshold is that high because these numbers move by a fifth between two runs of the same binary on the same machine. It is widened per cell by the interquartile range the record carries for that cell, because a cell that moves by half on its own cannot have a factor of two claimed about it. That rule is not a precaution: `compaction 2 bigint 1 select build` recorded 341 nanoseconds with a spread of 58 percent and the next two runs of the same binary gave 86.3 and 86.7, and a flat factor of two would have failed it the first time it landed on the high side. The other kind of finding is a cell that used to take a loop and now takes the row at a time path, and that fails on the fact rather than on the time, because it is worth catching on the commit that did it rather than on whichever later run happened to be slow enough.

Which machine the check runs on matters more than the threshold rule does, and the record now says so with a number. Over the 654 cells the mean interquartile range is 1.1 percent on `gamingpc-wsl` and 19.7 percent on `server3`, the worst cell is 8.7 percent against 1533.3, and no cell on `gamingpc-wsl` is over 50 percent where 35 cells on `server3` are. A threshold widened by a spread of 1533 percent is a threshold of thirty times, which is not a gate, it is a cell the gate has given up on. So the widening keeps a bad measurement from failing an innocent change, and running the check where the measurements are not bad is what makes it catch anything. `server3` is the gate machine because it is the one that is always free, and it stays the machine the full build runs on, but the kernel check belongs on `gamingpc-wsl`.

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

[`reports/m1-the-format-experiment.md`](reports/m1-the-format-experiment.md) is the first one. It is the storage format measurement from M1: a standalone encoder over ClickBench `hits`, TPC-H at scale factor 100 and three real Parquet files off the internet, answering whether shared dictionaries, shared symbol tables and recomputation rules deliver the ratios the specification assumed. The target was `hits` under 4 GB and the line at which the resource claim was declared wrong was 6 GB. It came out at 9.65 GB, the claim was wrong, and the specification was amended. The two techniques with no equivalent in DuckDB were worth 4 MB of it, and front coding a sorted dictionary, which any format could adopt tomorrow, was worth 2 GB.

## The reporting rules

These apply to the README, release notes, the dashboard, any talk, any post, and any conversation. They are in the engine's specification in full and this is the short version.

**State the comparison exactly.** Which rudb commit, which DuckDB version, which ClickHouse version, which machine, which kernel, which filesystem, which settings. "10x faster than DuckDB" is not a claim, it is a mood.

**Report the distribution, not the best run.** Median of at least five runs with the interquartile range. Never a minimum, never a single run. ClickBench's own convention of best-of-three is used for ClickBench because comparability with the public board matters more than rigor there, and any number in that form is labelled as ClickBench-convention.

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
gamingpc-wsl  regression       24/32         31 GiB    501 GiB   yes
gamingpc      regression       24/32         63 GiB    249 GiB   no
server3       regression        8/8          23 GiB     21 GiB   no
server2       correctness       6/6          11 GiB     28 GiB   no
server1       correctness       4/4           5 GiB     63 GiB   no
```

The last column is a separate question from the role and it is separate on purpose, because running the two together is how a machine gets picked for a suite it cannot hold. Running a full ClickBench needs three things at once: room for the largest engine's copy of `hits`, which is DuckDB's at about 20 GB, a load average low enough that a timing is a timing, and all five engines available on the machine at all. Exactly one row has all three, and there is a test that fails if that count ever stops being one, because zero means the next layer of the engine has no base number to be a ratio against and two means somebody has to say which of them the series continues on.

The distinction is structural rather than a note somebody stops reading. A machine carries a role, a role says whether a number from it may be published, and nothing in the fleet may. There is a test that fails if that stops being true, so adding a reporting machine is a conversation about whether it really is one.

What these are for is the other job a benchmark does, which is telling you on a Tuesday that the change you merged on Monday cost eight percent. That job wants the same machine over time much more than it wants the right machine, and these are the same machines over time.

Each one has a caveat that is worth knowing before trusting anything from it. `gamingpc-wsl` and `gamingpc` are one desktop seen from its two operating systems, and they are separate rows because a number from one is not a number from the other. The Linux side is where a full ClickBench happens, since ClickHouse has no Windows build and a four engine ClickBench missing the second fastest engine on the board is not the comparison. Four things travel with any number from it: it is a virtual machine, so the disk is virtual and the memory is 31 of the host's 63 GiB, its `/tmp` is a 16 GiB tmpfs so a scratch directory left on the default lands in RAM, its cores are heterogeneous at 8 performance and 16 efficiency so a thread on the wrong kind is a 2x outlier with nothing to do with the change under test, and the host is a desktop somebody uses. The Windows side runs the suites that are about the engine rather than about the comparison, which matters because the storage layer has to work there and nobody finds a Windows file locking problem on a Mac. `server3` is the quietest Linux box and the default for the gate and for a suite that fits, which ClickBench does not: 21 GiB free against DuckDB's own 20.46 GB `hits` file leaves no room to load it and none at all for anything after it. That figure was 44 GiB when this file was first written, which is the argument for probing it rather than trusting the table. `server2` and `server1` do correctness and compatibility work. `server1` has about 2 GiB of memory free, which makes it useless for timing and genuinely useful for the resource claim, since an out of memory there is a real finding.

`server1` deserves a stronger warning than that, and it was earned the hard way. It has the most free disk of any Linux box here, 62 GiB, so it was the machine picked for the first ClickBench run, and the smoke run it produced came back with interquartile ranges between 28 and 124 percent of the medians. The cause was not the harness. `server1` is a Kubernetes production worker node: kubelet, containerd, cilium-agent, a temporal server, a harbor registry and an otelcol are all resident on it, alongside a long running crawler, and its load average on 10 September 2026 was 51.72, 75.13 and 70.80 over one, five and fifteen minutes on four cores. Nothing timed on a machine loaded eighteen times over is a timing, at any sample count, and no threshold rescues it. Looking properly at the rest of the fleet afterwards produced the second finding, which is that `server3` is idle but has 21 GiB free against a 20.46 GB DuckDB `hits` file, so the quiet machine cannot hold the suite either. What can is the Linux side of `gamingpc`, at 32 threads, 31 GiB and 501 GiB free with every engine on it, and that is where ClickBench runs.

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
