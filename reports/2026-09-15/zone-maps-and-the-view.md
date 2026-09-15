# What zone maps bought, and what the harness is measuring instead

Two findings from the same run. The first is that per chunk zone maps work and are worth much less than they look. The second is that this harness builds rudb's ClickBench table differently from DuckDB's, and that difference is larger than the first finding.

Everything here is `gamingpc-wsl`, `hits-1m-snappy.parquet` at 999,975 rows, rudb pinned to one thread with `SET threads = 1`, medians of nine interleaved runs for the single query numbers and of five for the suite numbers. DuckDB is 2.0.0-dev84237 at its own default thread count. These were taken by hand against two rudb binaries built from the same tree with and without the change, not by `rudb-bench run`, so they are not comparable against the `run-clickbench-gamingpc-wsl-1m` files in this directory: those are unpinned and this is one thread. Rule seven still holds, it is one machine.

## Zone maps

rudb now keeps the smallest value, the largest value and the null count of every column of every chunk of a table in memory, and a scan skips a chunk whose bounds rule out the filter. Chunks are 2048 rows, so at a million rows there are 489 of them.

| | no zone maps | zone maps |
| --- | ---: | ---: |
| q37 query time | 11.05 ms | 4.36 ms |
| q37 rows leaving the scan | 999,975 | 247,265 |
| load, `CREATE TABLE AS SELECT` | 0.93 s | 1.02 s |
| peak RSS | 898 MB | 898 MB |

Six of the forty three queries have a filter the maps can answer. All six skip 75.3 percent of their chunks and run between two and two and a half times faster. q19 skips 37.8 percent and gains a little. The other thirty six are unchanged. Output is byte for byte identical with and without, across all forty three.

## Why that is not a win yet

The suite is the number, not the query.

| | query total over 41 queries |
| --- | ---: |
| no zone maps | 1604.4 ms |
| zone maps | 1567.7 ms |

Thirty seven milliseconds saved against sixty to ninety milliseconds spent building the maps during the load. One pass over ClickBench is a wash. It pays from the second pass on, and the published ClickBench protocol runs each query three times, so it does pay, by about seventy milliseconds over a full three pass run.

The reason it is not more is not the maps. It is that `hits` is not clustered on `CounterID`. The 7381 rows q37 asks for are spread across 121 of the 489 chunks, so three quarters of the chunks can be ruled out and no more. Sorted on `CounterID` those rows would fit in four chunks and the same maps would skip 99 percent. The sort key and the zone map are one lever and only half of it exists.

Forty one and not forty three because both engines reject the same two queries. `hits-1m-snappy.parquet` stores `EventDate` as days since the epoch and `EventTime` as seconds since the epoch, so the `date_part` and `date_trunc` queries have an integer where they want a timestamp, and rudb and DuckDB produce the same two binder errors.

## What the harness builds

`src/engine.rs` loads ClickBench for DuckDB with `CREATE TABLE ... AS SELECT * FROM read_parquet(...)` followed by `CHECKPOINT`, into a database file that survives between queries. It loads it for rudb with `CREATE VIEW ... AS SELECT * FROM read_parquet(...)`, replayed in front of every query, because rudb has no storage format to persist into and a CTAS in a process that is thrown away after one query would be a load timed once per query.

That is a defensible choice and the comment in the file explains it. Every generated report already says so in its own words: the engine table has a format column that reads "the Parquet" for rudb and "its own" for DuckDB, and the paragraph under it says that is an empty load column and a decode inside every query. So nothing is hidden. What is missing is the size of it, because a reader who knows a decode is in there still has no way to know whether it is five percent of the query column or half.

Here is the size of that, measured by running the whole suite in one rudb process so the CTAS is paid once.

| | load | suite wall | query total | peak RSS |
| --- | ---: | ---: | ---: | ---: |
| DuckDB, its own database file | 1.18 s | 2.19 s | 1.01 s | 1035 MB |
| rudb, table, zone maps | 1.02 s | 2.61 s | 1.59 s | 1006 MB |
| rudb, table, no zone maps | 0.93 s | 2.58 s | 1.60 s | 1006 MB |
| rudb, view over `read_parquet` | 0.00 s | 3.16 s | 3.16 s | 347 MB |

The view path is what this harness measures today. Reading the first and last rows against each other puts rudb 1.44 times behind DuckDB on the suite. Reading the first and second rows, which is the like for like comparison once rudb has a format, puts it 1.19 times behind. Neither number is wrong and they answer different questions.

Two things fall out of it that are worth saying plainly.

rudb's load is faster than DuckDB's. 1.02 seconds against 1.18 for the same statement over the same file, while building a full set of zone maps, and 0.93 without them. That is through the Parquet reader and it is not the native encoder, which is a different piece of code and is eighty times away from its own target. The F2 milestone currently treats those as one number and should not.

rudb's memory is three times DuckDB's advantage today, not a disadvantage: 347 MB against 1035. All of that advantage comes from not holding the table. Holding it costs 1006 MB, which is level with DuckDB, so on the resource half of the goal a format that is smaller in memory than a `Vec<Chunk>` of decoded vectors is where the remaining room is.

## What to do about the harness

Not a straight switch to CTAS. That would triple rudb's reported memory and would move a load into a query, which is the thing rule five exists to prevent. The honest fix is for rudb to have a storage format it can persist into and reopen, which is F2, and for this harness to then load it the way it loads DuckDB's.

Until then the number in this file is the one to quote when somebody asks how much of the gap is the decode. On ClickBench at a million rows on one thread it is 1.57 seconds of the 3.16, which is half the query column, and it is the single largest item in the gap between rudb and DuckDB on this suite.
