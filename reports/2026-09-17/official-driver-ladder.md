# Three engines under the official ClickBench driver

This is rudb, DuckDB and ClickHouse measured by the official ClickBench driver rather than by this harness, over four sample sizes on `gamingpc-wsl`. It is the first time rudb has been through that driver against anything other than DuckDB, and the first time the driver has been made to run over something smaller than the full file. The raw result files the driver wrote are in [official-driver](official-driver) beside this one, twelve of them, one per engine per size, in the driver's own `query,try,seconds` format.

## Why the official driver and not this harness

This harness and the official driver measure different things and both are worth having. The harness holds a wall clock around the whole subprocess, which is the honest answer to what a run costs to sit through, and on a sample that clock partly ranks process startup rather than engines. The driver asks each engine for its own query clock, which is what the public board publishes, and it is therefore the only one of the two whose number can be put next to a number somebody else published. It also drops the page cache before every query and runs each query three times, so a cold number and a hot number come out of the same pass.

The other reason is narrower and more important. The driver is the code that produces the board, so running our fork of it is the only way to find out whether rudb can be a board entry at all, as opposed to whether rudb is fast. Those turn out to be different questions, and the answer to the first one is now yes for all forty three queries.

## What had to change to run it small

Two things in `lib/benchmark-common.sh`, both now on the fork's main.

The download is `wget --continue` against the 14 GB hits file and it is unconditional. The obvious trick of dropping a small `hits.parquet` in place before the run does not work, because `wget --continue` reads the small file as a transfer that was cut short and appends the rest of the real one to it. So the first commit adds `BENCH_SKIP_DOWNLOAD`, which leaves the dataset alone and runs against whatever the operator already put in the system directory.

The second is a floor in `bench_load` that reads a database smaller than 5 GB as a load that was killed part way through, which is sound when the driver fetched the dataset itself and is exactly backwards once the operator says it did not. Under the new knob a small database is the thing that was asked for. The `./check` half of that guard still runs, because a load that crashed is worth catching whatever the driver was pointed at, and it is the half that does not depend on knowing how big the data should be.

Neither change touches any system's entry, the measured path, or the default behaviour. With `BENCH_SKIP_DOWNLOAD` unset the driver does what it did before.

## What ran

Twelve runs, three engines at four sizes, every one of them exit zero, on an idle machine with a one minute load average under 0.4 the whole way through.

| engine | version | how it was run |
| --- | --- | --- |
| rudb | 0.3.22 | the fork's `rudb` entry, its own `.timer on` clock off stderr |
| duckdb | v2.0.0-dev84237 (Development Version) cc7e7bac7f | the fork's `duckdb` entry, unmodified |
| clickhouse | 26.9.1.1162 | the fork's `clickhouse` entry, pointed at a private server instance |

The ClickHouse entry needed one accommodation. It runs against the system daemon under `/var/lib/clickhouse` and it starts with `CREATE OR REPLACE TABLE hits`, and this machine's system daemon already holds somebody else's 14 GB `hits` table. So the entry was copied and pointed at a private server on port 9001 with its own data directory, same binary, same MergeTree, same `clickhouse-client --time`, same eager load settings the entry's `install` writes. What changed is the data path and the port. Nothing that is measured changed, and nobody's loaded table was touched.

The sizes are the strided samples already on the box, roughly one thousand, ten thousand, one hundred thousand and one million rows out of the 99,997,497 in the full file. The driver's concurrency test was switched off for all three so that the comparison is the forty three queries and nothing else.

## Hot totals, which is the sum over the forty three queries of the best of three tries

| rows | rudb | duckdb | clickhouse | rudb over duckdb | rudb over clickhouse |
| --- | --- | --- | --- | --- | --- |
| 1k | 0.0168s | 0.1230s | 0.0820s | 0.14x | 0.21x |
| 10k | 0.0575s | 0.1300s | 0.0950s | 0.44x | 0.60x |
| 100k | 0.1605s | 0.3180s | 0.1870s | 0.50x | 0.86x |
| 1m | 0.6125s | 0.5750s | 0.3930s | 1.07x | 1.56x |

## Cold totals, which is the sum of the first try after the page cache was dropped

| rows | rudb | duckdb | clickhouse |
| --- | --- | --- | --- |
| 1k | 0.0300s | 0.2240s | 0.1270s |
| 10k | 0.0930s | 0.2210s | 0.1660s |
| 100k | 0.3933s | 0.4650s | 0.4700s |
| 1m | 1.7041s | 0.9360s | 2.0010s |

## Queries won, out of forty three, on the hot number

| rows | rudb faster than duckdb | rudb faster than clickhouse |
| --- | --- | --- |
| 1k | 43 | 43 |
| 10k | 38 | 38 |
| 100k | 37 | 27 |
| 1m | 19 | 13 |

## Load and what it leaves on disk

| rows | rudb load | duckdb load | clickhouse load | rudb size | duckdb size | clickhouse size |
| --- | --- | --- | --- | --- | --- | --- |
| 1k | 0.022s | 0.061s | 0.150s | 590,212 | 1,060,864 | 306,751 |
| 10k | 0.054s | 0.122s | 0.139s | 5,783,938 | 3,944,448 | 2,636,715 |
| 100k | 0.343s | 0.627s | 0.335s | 56,604,295 | 30,158,848 | 24,978,817 |
| 1m | 3.039s | 1.944s | 0.824s | 561,185,011 | 525,611,008 | 240,376,958 |

## What the four sizes say that one size cannot

The ladder is the point of running four of them. A single size cannot tell a fixed cost per query apart from a cost per row, and for rudb against these two engines that distinction is the whole story.

At a thousand rows almost nothing is per row, so the number is close to what each engine spends getting to the first byte of work. Over forty three queries that is 0.39ms a query for rudb, 1.91ms for ClickHouse and 2.86ms for DuckDB. Between a hundred thousand rows and a million the fixed part stops mattering and the slope is visible: 11.7ns a row a query for rudb, 6.6ns for DuckDB, 5.3ns for ClickHouse.

So rudb's fixed cost is about seven times lower than DuckDB's and its per row cost is about 1.8 times higher, and the two put the crossover at roughly half a million rows. The measured crossover is between a hundred thousand rows, where rudb is at 0.50x, and a million, where it is at 1.07x, which is consistent with that. The low fixed cost is real and it is worth having, and it is also the part of the curve that a hundred million row benchmark does not care about at all.

Pushing the same two constants out to the full file says rudb would finish the suite in about 50s against DuckDB's 29s and ClickHouse's 23s. That is a projection off samples that fit in cache and not a measurement, it says nothing about IO, spilling, or how each engine's parallelism behaves two orders of magnitude further out, and it is stated here only because a projection somebody can check beats a shrug. The one weak check available on it is that DuckDB's 29s lands near the 26s DuckDB posts on the public board on a smaller machine, which at least means the shape is not nonsense. That check turned out to be wrong the next day and the projection with it, by two times for DuckDB and 2.4 times for ClickHouse, because a per row cost measured at a million rows is measured where neither engine bothers to use the machine. See [the full file under the official driver](../2026-09-18/official-driver-full-file.md) for the measurement and for what still stands from this one.

## Where the per row cost is

The worst per row queries at a million rows, rudb against DuckDB on the hot number: q7 at 5.77x, q9 at 2.43x, q2 at 2.26x, q38 at 2.20x, q19 at 2.12x, q17 at 2.06x, q10 at 1.90x. Against ClickHouse the shape is similar with q18 at 4.91x and q2 at 4.52x added to it. The group of q33, q34 and q35 sits at about 1.8x DuckDB and about 2.5x ClickHouse together, and q19 at 40.3ms is rudb's single most expensive query in the run.

rudb is already ahead at a million rows on nineteen of them, and the margins there are not small: q30 at 0.19x DuckDB, q24 at 0.25x, q42 at 0.34x, q29 at 0.46x, q11 at 0.45x, q12 at 0.53x. So the gap is not a flat constant across the suite, it is concentrated, and the concentrated part is where the kernel work pays.

## Correctness

Every one of the twelve result files has forty three queries in it and no nulls. All three engines ran the official query set unmodified at all four sizes, and rudb answered every one. This is the end to end statement the milestone asks for, made by somebody else's driver rather than by ours.

## What this does not say

A sample is not the suite. A ClickBench number is defined over the 99,997,497 row file and a run over anything else is a development run whatever it prints, which is why the knob that makes this possible says so in its own documentation.

DuckDB and ClickHouse report their query time in milliseconds and rudb reports nanoseconds, so at a thousand and ten thousand rows the other two engines are quantised at a point where rudb's whole total is sixteen milliseconds. The honest reading of the top two rows is that rudb is below the resolution the other two timers can express, not that it is precisely seven times faster.

Rule seven still holds. These numbers are from `gamingpc-wsl` and nothing in this directory taken on another machine may be read against them, including the two server3 runs added alongside this file.

## The two server3 runs in the same commit

[run-clickbench-server3-100k.md](run-clickbench-server3-100k.md) and [run-clickbench-server3-1m.md](run-clickbench-server3-1m.md) are this harness rather than the official driver, on a different machine, and they are here because they are the first runs that put `clickhouse-server` next to `clickhouse-local`, DuckDB and Polars in one table. They do not include rudb, which is not installed at a current version on that box.

Their finding is about measurement rather than about engines. At a million rows the wall clock ranks DuckDB at 9.099s ahead of `clickhouse local` at 29.958s, while each engine's own clock ranks `clickhouse-server` at 2.387s, DuckDB at 3.564s, `clickhouse local` at 7.555s and Polars at 16.052s. The overhead column says why: 155% for DuckDB against 295% to 297% for both ClickHouses and 265% for Polars, because a DuckDB starts in about forty milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python. Below roughly ten million rows the query time column is the only usable one, and that is the same conclusion the official driver reaches from the other direction by never holding a wall clock in the first place.

Polars abstains from four queries there for want of a string length, a regexp replace, a date truncation and a tolerance for the repeated output name in q36, so its column is 39 of 43. At a hundred thousand rows all four engines agree on every answer the data settles, which is 32 of 43. Every engine missed the ten percent interquartile limit at every size, which on a box sitting at a load average of 27 is what should happen and is what the report says.

## Reproducing it

The driver ladder, from a checkout of the fork with the dataset already in place:

```
export BENCH_SKIP_DOWNLOAD=yes
export BENCH_CONCURRENT_DURATION=0
cd rudb && cp ~/rudb-data/clickbench/hits-1m.parquet hits.parquet && ./benchmark.sh
```

The two server3 runs:

```
rudb-bench run clickbench --rows 1000000 --engines duckdb,clickhouse-local,polars,clickhouse-server --runs 5 --report
```
