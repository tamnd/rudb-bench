# rudb-bench

The benchmark harness for [rudb](https://github.com/tamnd/rudb).

ClickBench, TPC-H, TPC-DS, JOB, CEB and H2O against DuckDB, ClickHouse, Umbra and DataFusion, and the reporting rules that decide what a number is allowed to claim.

It is a separate repository so that a result can be reproduced by someone who does not trust us, without building the engine from a specific commit of the engine's own repository. The whole project's claim is a performance claim, which means its credibility rests on the honesty of these measurements more than on any technical decision inside the engine. A benchmark number without its methodology is marketing.

The design is [`spec/15-rudb-bench.md`](https://github.com/tamnd/rudb/blob/main/spec/15-rudb-bench.md) in the rudb repository.

## Status

Early, and running. rudb cannot run a query yet, so what gets measured today is a real DuckDB on a small generated dataset. That is not a placeholder. Every reporting rule below is a property of the apparatus rather than of the engine, and every one of them is easier to build now, against an engine nobody here has a stake in, than on the afternoon somebody wants a headline.

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

### ClickBench is five query sets, not one

The official ClickBench repository keeps a `queries.sql` per engine and they are not copies of each other. Running one engine's text against all of them would be a measurement of somebody's translation, so this harness carries the official text per engine and says in the query table which engine gets which, and why.

Most of the difference is spelling. DataFusion's file quotes every identifier, because it lowercases the ones nobody quoted and every column in `hits` is camel case. Two of the forty three are not spelling. q28 and q29 average a string length, DuckDB's `STRLEN` counts characters and ClickHouse's `length` counts bytes, and `hits` is full of percent encoded UTF-8 where those are different numbers. Both boards are right about their own engine, so both texts are here and the report says the two columns disagreed on those two answers, which is the true thing to say about them.

Polars is the one engine with gaps. Its `SQLContext` accepts thirty nine of the forty three and refuses q28 for `strlen`, q29 for `regexp_replace`, q36 for the repeated `ClientIP` output name in the group by, and q43 for `date_trunc`. Those are four missing functions rather than four different opinions, and the alternative to declaring them absent is rewriting them until they ran, which would make the column a measurement of the rewrite. So they are declared absent in the query table, in a diff somebody reviewed, and never discovered at run time.

A column that is short a query is then handled everywhere it would otherwise lie. The per-query grid is looked up by name rather than by position, so a missing q28 does not slide q29's time onto q28's row. The ratio against the reference engine is taken over the queries both columns ran, and the row says how many that was, because dividing a short total by a full one hands the best number on the page to whichever engine ran the least. The report prints one line per gap naming the query and the reason. And a column with any gap in it cannot be published, since rule three asks for the whole suite.

### TPC-H is one query set, and was checked rather than assumed

The difference between TPC-H and ClickBench is that TPC-H has a specification. The business questions have one meaning, there is no per engine `queries.sql` anywhere to be faithful to, so there is one text and every engine gets it. The text is DuckDB's rendering out of `extension/tpch/dbgen/queries`, because that one is public, versioned and checkable rather than retyped by us out of a PDF.

Whether every engine actually takes that text is a question with an answer, so it was asked before the table was written, against a real SF 0.01 corpus rather than against an empty schema. DuckDB, ClickHouse and DataFusion each answered all twenty two. Polars answers nineteen: its `SQLContext` does not resolve the correlated `p_partkey` in q02 or q17, and it rejects the `NOT LIKE` inside the q13 join constraint. Those three are declared absent in the table for the same reason the four ClickBench ones are.

### ClickHouse is two rows, because it is two systems

`clickhouse local` and a ClickHouse server are not the same engine wearing different clothes. The local one reads the Parquet where it lies and has no sorting key, no primary index and no chance to have merged anything. The server owns its data in MergeTree parts ordered by the key from the suite's own `create.sql`, which for ClickBench is the key ClickHouse Inc. publishes in theirs. Measuring only the local one and calling the result ClickHouse would flatter us, and by a lot: over three runs of the smoke suite on server3 the local row totalled between 3.5s and 5.3s hot where the tuned server totalled between 1.6s and 2.3s, so the engine our headline claim is stated against is somewhere around twice as fast as the row that was standing in for it. The harness now runs both and prints both, and the sorting key costs one or two percent on disk, 50.60 MiB against 49.49 MiB for the same ten million rows.

Both ClickHouse rows get the column types out of the official `create.sql`, and only the tuned one gets the sorting key. That split is deliberate. A sorting key is tuning and belongs to the row that exists to show what tuning is worth. A schema is not tuning: asking ClickHouse to infer one from the Parquet gives every column back as `Nullable`, which costs a null map per column and turns several things off, and nobody deploys that. DuckDB never paid it, because its `CREATE TABLE AS SELECT` out of the same file gets the real non-null types, so an inferred ClickHouse schema was a handicap this project had put on ClickHouse rather than a difference between the engines. All 105 `hits` columns are copied out of `clickhouse/create.sql` down to the generic spellings it uses, since writing `Int64` where the file says `BIGINT` would be a diff against the thing this is meant to be a copy of.

Reading the server's on-disk number needs care, and getting it wrong is easy. `OPTIMIZE TABLE ... FINAL` writes a new part and leaves the parts it replaced on disk as inactive until `old_parts_lifetime` expires, which is eight minutes by default, so a directory size taken straight after the load counts the data roughly twice. That first read said 128.03 MiB and would have been published as the sorting key costing 2.6x. Both ClickHouse rows now ask ClickHouse instead, with `select sum(bytes_on_disk) from system.parts where active`, and fall back to the directory only when that answer does not parse.

### A column that is flat is not a column about the queries

The tuned server row came back with q1 count at 222ms, q2 at 268ms, q3 at 267ms, q4 at 280ms, q5 at 296ms and the q6 self join at 378ms. A count and a self join over ten million rows do not cost the same, so that column is not measuring the queries: whatever all six have in common is bigger than the difference between them, and here it is `clickhouse client` starting up and connecting. This is the same finding as the interquartile ranges below, arrived at from the other direction, and it is worth catching by machine rather than by eye.

So a result now reports its shape spread, the hot median of its slowest query over the hot median of its fastest, and a column under 2x is named in the report and refused for publication. On the run above duckdb spread over 4.61x and datafusion over 5.83x, which is what a column that is measuring the queries looks like, while clickhouse-local at 1.50x, clickhouse-server at 1.70x and polars at 1.24x were all flagged. Read a flagged column as an upper bound on the engine and not as a measurement of it. Every one of those three starts a process per query whose startup is tens or hundreds of milliseconds, which is the same cause the regression gate section below arrives at, and the fix for all of it is the in-process measurement path at sub-milestone 2b.

### Every engine gives its disk back as soon as it is measured

Three of the five engines convert the Parquet into a format of their own before they run anything, and on ClickBench those copies are roughly 20 GB for DuckDB, 13 GB for `clickhouse local` and 12 GB for the ClickHouse server. A harness that loads all of them and holds them until the report is written needs the sum of every engine's format free on the disk, about 45 GB, when the run itself never needs more than the largest one at any moment. That is not a detail. It is the difference between the suite running on the machine that is idle and the suite running on the machine that happens to have space.

So `Engine::unload` exists and the comparison calls it the moment an engine finishes, on the failure path as well as the success one. The comparison already ran one engine to completion before starting the next, and everything the report prints about an engine's size was read at load time and is sitting in its result by then, so nothing in the table changes and peak disk drops from the sum to the largest. The default implementation does nothing, which is correct for DataFusion and Polars: they read the Parquet where it lies, they never made a copy, and there is nothing of theirs to remove.

What this does cost is the afternoon where somebody wants to open an engine's database after a run and look at what it built. `RUDB_BENCH_KEEP` turns the whole thing off for that. It is set deliberately and never by default, because a debugging convenience that quietly triples the disk a run needs is a debugging convenience that stops the run.

### The regression gate, and what it does not catch yet

`rudb-bench run smoke --record` writes what just ran into `baselines/smoke.txt`, which is committed. `rudb-bench run smoke --check` runs the suite again and fails when a query's median moved past two times the recorded one and its samples no longer overlap the recorded ones. `--check-drift` is the same at ten percent and refuses to run against a record from a different machine, because ten percent between two machines is a fact about the machines. Both take fifteen hot runs per query rather than the five a person reading a table gets, since the interquartile range of five samples is the gap between the second and the fourth of them.

A query that swung wider than ten percent on the day is reported and never failed, which is section 13.8 of the engine specification and is the difference between a gate people keep and a gate people turn off. On the smoke suite today that rule declines almost everything, and the cause is measured rather than guessed. Every query here runs as a fresh subprocess on purpose, so that no engine gets a warm allocator and a warm buffer pool the others paid for. On queries of tens of milliseconds that means most of what gets timed is the process start. Fifteen runs of `duckdb -c "select 1"` on an idle server3, with no query attached to them at all, spread from 30ms to 60ms, an interquartile range of 37 percent of their own median. Fifteen runs of `clickhouse local --query "select 1"` spread from 180ms to 510ms, which is 48 percent.

So the gate prints how many queries it actually compared on every run, and says plainly that a run which compared none of them should be read as a red rather than a green. The fix is not a bigger threshold. It is to stop timing `execve`, which is the in-process measurement path at sub-milestone 2b.

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

`runs/<suite>.txt` and `baselines/<suite>.txt` look alike and are not interchangeable. A baseline record is per query and holds three quartiles, because the gate asks whether one query got twice as slow. A run is per engine and holds totals, because the ledger asks what a layer bought across a suite. Records are replaced when they are retaken and runs never are, because a history that overwrites itself is a table.

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

Those are the machines a published number comes from. They are not the machines the project owns. `rudb-bench fleet` prints what is really here, which is four boxes and none of them is a `c6a.4xlarge`.

```
$ rudb-bench fleet
Published numbers come from c6a.4xlarge, 16 vCPU, 32 GiB, gp2.
Nothing below is that machine. Reporting rule seven: never compare across machines.

machine     role         cores/threads   memory   free disk
gamingpc    regression       24/32         63 GiB    249 GiB
server3     regression        8/8          23 GiB     44 GiB
server2     correctness       6/6          11 GiB     25 GiB
server1     correctness       4/4           5 GiB    114 GiB
```

The distinction is structural rather than a note somebody stops reading. A machine carries a role, a role says whether a number from it may be published, and nothing in the fleet may. There is a test that fails if that stops being true, so adding a reporting machine is a conversation about whether it really is one.

What these are for is the other job a benchmark does, which is telling you on a Tuesday that the change you merged on Monday cost eight percent. That job wants the same machine over time much more than it wants the right machine, and these are the same machines over time.

Each one has a caveat that is worth knowing before trusting anything from it. `gamingpc` is the only machine that can hold ClickBench comfortably and the only one that can say anything about thread scaling, but its cores are heterogeneous, 8 performance and 16 efficiency, so a thread landing on the wrong kind is a 2x outlier with nothing to do with the change under test. It is also the only Windows machine, which matters because the storage layer has to work there and nobody finds a Windows file locking problem on a Mac. `server3` is the default for a Linux regression run, with 44 GiB free against DuckDB's own 20.46 GB `hits` file, which means loading it streams from Parquet and deletes the intermediate rather than keeping both. `server2` and `server1` do correctness and compatibility work. `server1` has about 2 GiB of memory free, which makes it useless for timing and genuinely useful for the resource claim, since an out of memory there is a real finding.

`server1` deserves a stronger warning than that, and it was earned the hard way. It has the most free disk of any Linux box here, 62 GiB, so it was the machine picked for the first ClickBench run, and the smoke run it produced came back with interquartile ranges between 28 and 124 percent of the medians. The cause was not the harness. `server1` is a Kubernetes production worker node: kubelet, containerd, cilium-agent, a temporal server, a harbor registry and an otelcol are all resident on it, alongside a long running crawler, and its load average on 10 September 2026 was 51.72, 75.13 and 70.80 over one, five and fifteen minutes on four cores. Nothing timed on a machine loaded eighteen times over is a timing, at any sample count, and no threshold rescues it. `server3` is idle at load 0.02 with eight cores, and it is where a Linux number comes from even though it is the tighter machine on disk, which is what the section above is about.

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
