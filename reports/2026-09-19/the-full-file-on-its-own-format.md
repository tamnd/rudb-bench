# The full file under the official driver, with rudb ahead of DuckDB for the first time

Yesterday's run of the official driver put rudb on the real ClickBench file at 27.07s against DuckDB's 14.03s, which is [the full file](../2026-09-18/the-full-file.md). This is the same driver, the same fork, the same 99,997,497 rows out of the same 14,779,976,446 byte `hits.parquet`, on the same `gamingpc-wsl`, with rudb at 0.3.45 instead of 0.3.33 and with two more engines in the table. The driver's own `query,try,seconds` output is in [full-file-native](full-file-native) beside this.

rudb's hot total is 15.109s against DuckDB 1.5.5's 16.965s. That is the first time rudb has been ahead of a released DuckDB over the whole benchmark on this machine, and most of the rest of this report is about where the lead comes from, because the answer is one mechanism and not general speed.

## Four engines, forty three queries

| engine | version | load | size on disk | cold total | hot total |
| --- | --- | --- | --- | --- | --- |
| rudb | 0.3.45 | 100.776s | 12,220,783,789 | 18.593s | 15.109s |
| duckdb | v1.5.5 d8cdaa33fd | 41.522s | 20,464,283,648 | 19.047s | 16.965s |
| duckdb-pinned | v2.0.0-dev84237 cc7e7bac7f | 40.873s | 20,459,040,768 | 15.984s | 14.228s |
| clickhouse-local | 26.9.1.1162 | 0.009s | 14,779,976,446 | 30.007s | 17.046s |

All four answered all forty three, so unlike yesterday the totals are over the same set and nothing has been dropped. Hot is the best of three tries, cold is the first try after the driver dropped the page cache, and both are the driver's own definitions rather than this harness's.

The two DuckDB rows are the same entry run twice with a different binary on `PATH`: `duckdb` is the release the published board uses and `duckdb-pinned` is the development build this repository has been measuring against all week. `clickhouse-local` is the `clickhouse-parquet` entry, which reads `hits.parquet` in place through the File engine, so its load is a `sync` and its size on disk is the source file. That is a different storage story from the other three and its row should be read that way.

## What changed in rudb since yesterday

Over the forty two queries 0.3.33 answered:

| | 0.3.33 | 0.3.45 | |
| --- | --- | --- | --- |
| load | 324.297s | 100.776s | 3.2x faster |
| size on disk | 43,015,574,093 | 12,220,783,789 | 3.5x smaller |
| cold total | 637.899s | 18.570s | 34.4x faster |
| hot total | 27.065s | 15.092s | 1.79x faster |

The size and the load were the two things yesterday's report said were wrong at every scale, and both moved by more than three times in a day. The cold total moved by thirty four times, and the section below says why that number is not as good as it looks.

## The cold column stopped being a cold column again

Yesterday's cold total was a real disk measurement for one accidental reason: the database was 43 GB, the machine has 31 GiB, and so the Windows host underneath WSL2 could not hold the file no matter what `drop_caches` did inside the guest. The database is now 12.2 GB. It fits. The host holds it, `drop_caches` empties the guest page cache and the first try reads out of somebody else's memory again, which is the warning every run on this machine before yesterday had to carry.

So 18.593s is a floor rather than a cold number, and 34.4x is the improvement in a quantity that changed meaning in between. The honest statement is the smaller one: the file got 3.5x smaller and the cold penalty that 3.5x hides has not been measured on this machine since.

## Where the lead comes from

Ten of the forty three are `GROUP BY x ORDER BY count(*) DESC LIMIT 10`, and rudb answers them out of the frequency synopsis the writer stores per column rather than by scanning. This is the certified structure in [`spec/stats/02-the-catalogue.md`](https://github.com/tamnd/rudb/blob/main/spec/stats/02-the-catalogue.md), five hundred and twelve exact leading counts plus an `omitted_max` bound, and it answers only when it can discharge the proof that the tenth candidate beats everything omitted.

| | rudb | duckdb | duckdb-pinned | clickhouse-local |
| --- | --- | --- | --- | --- |
| the ten answered from metadata | 0.212s | 3.307s | 3.097s | 3.413s |
| the other thirty three | 14.898s | 13.658s | 11.131s | 13.633s |
| all forty three | 15.109s | 16.965s | 14.228s | 17.046s |

That is the whole lead and then some. On the other thirty three rudb is 1.09x the released DuckDB, 1.34x the development build and 1.09x clickhouse-local, which is parity rather than a lead. Counted per query rather than by total it is twenty two wins and twenty one losses against the released DuckDB.

The mechanism is general and not fitted to the benchmark, and it is worth showing that rather than asserting it. The same shape against `WatchID`, which is near unique and therefore has no heavy hitters to certify, falls back and takes 2.103s. Against `UserID` it answers at `LIMIT 16` and falls back at `LIMIT 32`, taking 0.656s; against `URL` it still answers at `LIMIT 32`, because where the certificate runs out depends on the column's distribution rather than on a number somebody chose. Adding a `WHERE` clause, changing the aggregate to `SUM(RegionID)`, or adding a tiebreaker to the `ORDER BY` all fall back to the same 0.7s. It answers what it can prove and nothing else.

It is also not free. It is built during the load, and the load is 2.4x DuckDB's.

## Time per query

| query | rudb | duckdb | duckdb-pinned | clickhouse-local | rudb over duckdb |
| --- | --- | --- | --- | --- | --- |
| q1 | 0.000286 | 0.009000 | 0.009000 | 0.021 | 0.032x |
| q2 | 0.000304 | 0.021 | 0.032 | 0.048 | 0.014x |
| q3 | 0.000390 | 0.036 | 0.035 | 0.060 | 0.011x |
| q4 | 0.000344 | 0.046 | 0.039 | 0.080 | 0.007x |
| q5 | 0.329 | 0.172 | 0.189 | 0.218 | 1.91x |
| q6 | 0.018 | 0.228 | 0.245 | 0.224 | 0.078x |
| q7 | 0.000407 | 0.015 | 0.015 | 0.047 | 0.027x |
| q8 | 0.000663 | 0.021 | 0.024 | 0.044 | 0.032x |
| q9 | 0.448 | 0.235 | 0.230 | 0.317 | 1.91x |
| q10 | 0.523 | 0.309 | 0.346 | 0.350 | 1.69x |
| q11 | 0.113 | 0.101 | 0.123 | 0.132 | 1.12x |
| q12 | 0.151 | 0.125 | 0.137 | 0.141 | 1.21x |
| q13 | 0.144 | 0.247 | 0.267 | 0.287 | 0.58x |
| q14 | 0.243 | 0.465 | 0.482 | 0.454 | 0.52x |
| q15 | 0.296 | 0.320 | 0.314 | 0.369 | 0.93x |
| q16 | 0.000657 | 0.205 | 0.190 | 0.205 | 0.003x |
| q17 | 0.799 | 0.662 | 0.668 | 0.793 | 1.21x |
| q18 | 0.148 | 0.550 | 0.542 | 0.379 | 0.27x |
| q19 | 0.947 | 1.256 | 1.254 | 1.380 | 0.75x |
| q20 | 0.022 | 0.032 | 0.033 | 0.056 | 0.67x |
| q21 | 0.865 | 0.363 | 0.330 | 0.623 | 2.38x |
| q22 | 1.046 | 0.462 | 0.474 | 0.681 | 2.26x |
| q23 | 1.164 | 0.636 | 0.594 | 1.046 | 1.83x |
| q24 | 0.979 | 0.321 | 0.125 | 0.758 | 3.05x |
| q25 | 0.125 | 0.050 | 0.066 | 0.245 | 2.51x |
| q26 | 0.207 | 0.115 | 0.094 | 0.154 | 1.80x |
| q27 | 0.116 | 0.069 | 0.059 | 0.233 | 1.68x |
| q28 | 0.974 | 0.387 | 0.405 | 0.981 | 2.52x |
| q29 | 2.231 | 4.408 | 2.011 | 1.372 | 0.51x |
| q30 | 0.050 | 0.033 | 0.055 | 0.059 | 1.52x |
| q31 | 0.401 | 0.239 | 0.266 | 0.316 | 1.68x |
| q32 | 0.425 | 0.427 | 0.409 | 0.423 | 1.00x |
| q33 | 1.544 | 1.262 | 1.222 | 1.305 | 1.22x |
| q34 | 0.105 | 1.368 | 1.301 | 1.408 | 0.077x |
| q35 | 0.103 | 1.382 | 1.291 | 1.353 | 0.074x |
| q36 | 0.000706 | 0.204 | 0.161 | 0.147 | 0.003x |
| q37 | 0.076 | 0.031 | 0.032 | 0.062 | 2.44x |
| q38 | 0.041 | 0.022 | 0.020 | 0.049 | 1.88x |
| q39 | 0.188 | 0.023 | 0.025 | 0.045 | 8.16x |
| q40 | 0.243 | 0.050 | 0.049 | 0.075 | 4.86x |
| q41 | 0.017 | 0.018 | 0.023 | 0.039 | 0.92x |
| q42 | 0.018 | 0.020 | 0.021 | 0.036 | 0.92x |
| q43 | 0.007710 | 0.020 | 0.021 | 0.031 | 0.39x |

The ten answered from the synopsis are q1, q2, q3, q4, q7, q8, q16, q34, q35 and q36. Four of those are ungrouped aggregates over the whole table rather than top-k, and they come out of the per column counts, minima, maxima and sums for the same reason. q36 is the one worth a second look: it groups by `ClientIP` and three expressions derived from `ClientIP`, and it still lands on the synopsis, which means the planner is collapsing the derived columns onto the one they depend on before the aggregate is chosen.

q34 and q35 are the two that take 0.1s rather than 0.0007s, and the difference is the column. They group by `URL`, so the ten counts come out of the synopsis but the ten strings have to be read back out of the dictionary, and that read is the whole 0.1s. The same query with `LIMIT 32` is also 0.106s, and the same query with `OFFSET 1000` falls off the synopsis and takes 0.721s.

The other end is where the work is, and it is the same list as yesterday. By ratio the worst are q39 at 8.16x, `GROUP BY URL` under five predicates with `OFFSET 1000`, which is exactly the shape the synopsis cannot answer, and q40 at 4.86x, the same thing over a wide `CASE WHEN` key. By seconds the worst are different: q24 `SELECT * ... ORDER BY EventTime LIMIT 10` at 0.658s above DuckDB, q28 `AVG(STRLEN(URL)) ... HAVING COUNT(*) > 100000` at 0.587s, and q22, q23 and q21, the `LIKE '%google%'` family, at 0.584s, 0.528s and 0.502s. Those five are 2.859s of the 4.715s rudb spends above DuckDB across every query where it is behind.

Net of the queries where it is ahead, rudb is 1.240s behind DuckDB outside the synopsis, which is the number the kernel work has to close before any of this is about the format.

## The answers, which the driver does not check

The driver times queries and never looks at what they returned, which is worth saying loudly in a report whose headline is a lead. So all forty three were run again against rudb's `hits.db` and DuckDB's, side by side, and compared.

Thirty two matched byte for byte. Of the eleven that did not:

- Ten differ only in which rows a tie at the `LIMIT` boundary picked, or in the case of q18 in which ten rows an unordered `LIMIT 10` returned. q31 is the clearest: the first nine rows are identical and the tenth is a different `ClientIP` with the same count of 1058. SQL does not specify these and neither engine is wrong.
- q24 returns the same ten rows in the same order, and differs in how the two command line interfaces print non-printable bytes in two of the hundred and five columns. `ParamCurrency` is the three bytes `4E 48 1C`; DuckDB renders the third as `\28` and rudb emits it raw. `SELECT length(ParamCurrency) ... GROUP BY 1` returns 3 for 15,911 rows on both. The stored data is identical.

So there is no answer disagreement in this run. That is a statement about these forty three queries and not a substitute for the compat corpus.

## How to reproduce it

On `gamingpc-wsl`, with a copy of the Athena `hits.parquet` already in the entry directory so the driver's `wget --continue` skips the download:

```
cd clickbench/rudb && BENCH_CONCURRENT_DURATION=0 ./benchmark.sh
```

and the same for `clickbench/duckdb` and `clickbench/clickhouse-parquet`. `rudb/install` was pinned to `v0.3.45` for this run and no-ops when a matching binary is already on `PATH`. Note that `rudb/load` and `duckdb/load` both end in `rm -f hits.parquet`, so the file in the entry directory has to be a copy rather than a link to whatever else on the machine is using it.

## What this number is not

It is not ten times anything. The goal this repository is measured against is 10x the best rival, and 1.12x the released DuckDB with the lead entirely inside one mechanism is a long way from it. The development build of DuckDB is still ahead of rudb on the same machine, 14.228s against 15.109s, and on the thirty three queries the synopsis does not touch it is ahead by 1.34x.

It is not a cold number, for the reason in the section above.

It is not comparable to anything in [`README.md`](../../README.md)'s ladder. Those columns come from `rudb-bench`, where rudb reads Parquet in place and the medians are over five runs; this is the official driver, where rudb reads its own format and the number is a best of three. The same rudb, the same machine and the same rows go from 31.361s there to 15.109s here, and the whole of that difference is the storage format.

It is not a measurement of memory. The driver does not record peak RSS and this report does not claim one.
