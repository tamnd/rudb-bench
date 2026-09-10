# rudb-bench

The benchmark harness for [rudb](https://github.com/tamnd/rudb).

ClickBench, TPC-H, TPC-DS, JOB, CEB and H2O against DuckDB, ClickHouse, Umbra and DataFusion, and the reporting rules that decide what a number is allowed to claim.

It is a separate repository so that a result can be reproduced by someone who does not trust us, without building the engine from a specific commit of the engine's own repository. The whole project's claim is a performance claim, which means its credibility rests on the honesty of these measurements more than on any technical decision inside the engine. A benchmark number without its methodology is marketing.

The design is [`spec/15-rudb-bench.md`](https://github.com/tamnd/rudb/blob/main/spec/15-rudb-bench.md) in the rudb repository.

## Status

Early. There is no engine to measure yet. What exists is the recorded board and the shape of a result, and the board was recorded before there was anything to flatter, which is the only time a baseline is worth recording.

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

## Who we measure against

DuckDB at the version compatibility is claimed against, in its native format. This is the primary comparison.

ClickHouse, because it is the fastest widely deployed single node analytical engine. Any claim of being the fastest that does not beat ClickHouse is not a claim of being the fastest.

Umbra where a binary is obtainable, because it is the current leader. Where a binary is not obtainable its published board numbers are cited as published board numbers with the date, and never mixed into a table of numbers we measured.

DataFusion and Polars, because they are the Rust ecosystem's answer and because being ahead of them is a floor rather than an achievement.

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
