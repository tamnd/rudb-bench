# Twelve patch versions later, the same driver puts rudb ahead of both DuckDB builds

[The run earlier today](the-full-file-on-its-own-format.md) put rudb 0.3.45 at 15.109s hot against a released DuckDB's 16.965s and a development DuckDB's 14.228s, so it was ahead of one and behind the other. This is the same official driver, the same fork, the same 99,997,497 rows out of the same 14,779,976,446 byte `hits.parquet`, on the same `gamingpc-wsl`, with nothing changed except which `rudb` was on `PATH`. rudb main at [db31ad8](https://github.com/tamnd/rudb/commit/db31ad8), which reports itself as 0.3.57, answers all forty three in 13.881s hot. That is ahead of both. The driver's `query,try,seconds` output is in [full-file-native/rudb-0357.csv](full-file-native/rudb-0357.csv) beside the other four.

## Four engines and two rudbs

| engine | version | load | size on disk | cold total | hot total |
| --- | --- | --- | --- | --- | --- |
| rudb | 0.3.57 | 99.341s | 11,582,927,503 | 17.258s | 13.881s |
| rudb | 0.3.45 | 100.776s | 12,220,783,789 | 18.593s | 15.109s |
| duckdb | v1.5.5 d8cdaa33fd | 41.522s | 20,464,283,648 | 19.047s | 16.965s |
| duckdb-pinned | v2.0.0-dev84237 cc7e7bac7f | 40.873s | 20,459,040,768 | 15.984s | 14.228s |
| clickhouse-local | 26.9.1.1162 | 0.009s | 14,779,976,446 | 30.007s | 17.046s |

The four rows other than the first are carried over unchanged from this morning's report, which is the whole point of running the driver again rather than a new comparison: only one variable moved.

Load is flat, 99.341s against 100.776s, which is the same number twice and not an improvement. Size on disk fell 5%, from 12.22 GB to 11.58 GB. The cold column is still a floor and not a cold measurement, for the reason [the earlier report](the-full-file-on-its-own-format.md#the-cold-column-stopped-being-a-cold-column-again) gives: 11.58 GB fits in the Windows host's cache underneath WSL2, so `drop_caches` inside the guest does not reach the blocks.

## The lead stopped being one mechanism

This morning's finding was that rudb's entire lead came from ten queries answered out of the certified frequency synopsis, and that on the other thirty three it was 1.09x the released DuckDB, which is parity rather than a lead. Splitting 0.3.57 the same way:

| | rudb 0.3.57 | rudb 0.3.45 | duckdb | duckdb-pinned | clickhouse-local |
| --- | --- | --- | --- | --- | --- |
| the ten answered from metadata | 0.172s | 0.212s | 3.307s | 3.097s | 3.413s |
| the other thirty three | 13.710s | 14.898s | 13.658s | 11.131s | 13.633s |
| all forty three | 13.881s | 15.109s | 16.965s | 14.228s | 17.046s |

The ten are the ones 0.3.45 answered from the synopsis, held fixed so the two rudb columns are over the same split. q6 joined them at 0.3.57 and is counted in the thirty three below, which understates the change rather than flattering it; the next section is about q6 on its own.

The synopsis ten barely moved, 0.212s to 0.172s, because there was nowhere for them to go. The whole 1.23s gain is in the other thirty three, 14.898s to 13.710s. That closes the gap to the released DuckDB from 1.09x to 1.00x, so the scanned part of the benchmark is now a dead heat rather than a small loss, and narrows the gap to the development build from 1.34x to 1.23x. rudb is still behind that build on the queries it actually scans for, and the total only comes out ahead because of the ten.

Counted per query rather than by total, 0.3.57 is twenty three wins and twenty losses against the released DuckDB, and twenty four wins and nineteen losses against the development build.

Thirty of the forty three got faster, thirteen got slower, and every one of the thirteen moved by 8 ms or less except q28, which went from 0.974s to 1.032s. The gains are concentrated: q17 by 0.239s, q19 by 0.211s, q23 by 0.099s, q15 by 0.098s, q33 by 0.096s.

## One query left the scan entirely

q6 is `SELECT COUNT(DISTINCT SearchPhrase) FROM hits`, and it went from 0.018s to 0.000308s. That is not a 58x speedup of a scan, it is the same kind of change as the other ten: the query stopped being executed and started being read out of the catalogue.

The count is exact and not an estimate, which matters more than the time. rudb and DuckDB both say 6,019,103 for `SearchPhrase`, 17,630,976 for `UserID`, 18,342,019 for `URL` and 9,425,424 for `Title`. The path is not restricted to the benchmark's column either: `COUNT(DISTINCT URL)` answers in 0.02s and `COUNT(DISTINCT Title)` in the same range. Adding a `WHERE CounterID = 62` falls back, as it should, since the stored count is over the whole column.

There is one sharp edge worth recording. Asking for two of them in one statement, `SELECT COUNT(DISTINCT SearchPhrase), COUNT(DISTINCT URL) FROM hits`, takes 27.75s, which is longer than the two separate queries by four orders of magnitude and longer than the entire forty three query benchmark. The metadata path appears to handle a single distinct aggregate and the fallback for two is much worse than a scan should be. No ClickBench query has that shape so it does not touch any number above, but it is a real behaviour on a query a user would write.

## Time per query

| query | rudb 0.3.45 | rudb 0.3.57 | duckdb | duckdb-pinned | 0.3.57 over pinned |
| --- | --- | --- | --- | --- | --- |
| q1 | 0.000286 | 0.000310 | 0.009000 | 0.009000 | 0.034x |
| q2 | 0.000304 | 0.000335 | 0.021 | 0.032 | 0.010x |
| q3 | 0.000390 | 0.000421 | 0.036 | 0.035 | 0.012x |
| q4 | 0.000344 | 0.000355 | 0.046 | 0.039 | 0.009x |
| q5 | 0.329 | 0.327 | 0.172 | 0.189 | 1.73x |
| q6 | 0.018 | 0.000308 | 0.228 | 0.245 | 0.001x |
| q7 | 0.000407 | 0.000383 | 0.015 | 0.015 | 0.026x |
| q8 | 0.000663 | 0.000719 | 0.021 | 0.024 | 0.030x |
| q9 | 0.448 | 0.453 | 0.235 | 0.230 | 1.97x |
| q10 | 0.523 | 0.531 | 0.309 | 0.346 | 1.54x |
| q11 | 0.113 | 0.114 | 0.101 | 0.123 | 0.93x |
| q12 | 0.151 | 0.147 | 0.125 | 0.137 | 1.07x |
| q13 | 0.144 | 0.129 | 0.247 | 0.267 | 0.48x |
| q14 | 0.243 | 0.215 | 0.465 | 0.482 | 0.45x |
| q15 | 0.296 | 0.198 | 0.320 | 0.314 | 0.63x |
| q16 | 0.000657 | 0.000694 | 0.205 | 0.190 | 0.004x |
| q17 | 0.799 | 0.559 | 0.662 | 0.668 | 0.84x |
| q18 | 0.148 | 0.119 | 0.550 | 0.542 | 0.22x |
| q19 | 0.947 | 0.736 | 1.256 | 1.254 | 0.59x |
| q20 | 0.022 | 0.004530 | 0.032 | 0.033 | 0.14x |
| q21 | 0.865 | 0.866 | 0.363 | 0.330 | 2.62x |
| q22 | 1.046 | 0.993 | 0.462 | 0.474 | 2.09x |
| q23 | 1.164 | 1.064 | 0.636 | 0.594 | 1.79x |
| q24 | 0.979 | 0.953 | 0.321 | 0.125 | 7.63x |
| q25 | 0.125 | 0.106 | 0.050 | 0.066 | 1.61x |
| q26 | 0.207 | 0.214 | 0.115 | 0.094 | 2.28x |
| q27 | 0.116 | 0.115 | 0.069 | 0.059 | 1.95x |
| q28 | 0.974 | 1.032 | 0.387 | 0.405 | 2.55x |
| q29 | 2.231 | 2.182 | 4.408 | 2.011 | 1.08x |
| q30 | 0.050 | 0.049 | 0.033 | 0.055 | 0.88x |
| q31 | 0.401 | 0.324 | 0.239 | 0.266 | 1.22x |
| q32 | 0.425 | 0.354 | 0.427 | 0.409 | 0.87x |
| q33 | 1.544 | 1.448 | 1.262 | 1.222 | 1.18x |
| q34 | 0.105 | 0.084 | 1.368 | 1.301 | 0.064x |
| q35 | 0.103 | 0.084 | 1.382 | 1.291 | 0.065x |
| q36 | 0.000706 | 0.000718 | 0.204 | 0.161 | 0.004x |
| q37 | 0.076 | 0.058 | 0.031 | 0.032 | 1.82x |
| q38 | 0.041 | 0.032 | 0.022 | 0.020 | 1.60x |
| q39 | 0.188 | 0.168 | 0.023 | 0.025 | 6.71x |
| q40 | 0.243 | 0.192 | 0.050 | 0.049 | 3.92x |
| q41 | 0.017 | 0.008526 | 0.018 | 0.023 | 0.37x |
| q42 | 0.018 | 0.012 | 0.020 | 0.021 | 0.58x |
| q43 | 0.007710 | 0.005397 | 0.020 | 0.021 | 0.26x |

Seconds, hot, best of three, from the driver's own `result.csv`. The last column is against the development build because that is the one rudb is still behind.

Where rudb loses it loses in the same places it lost this morning and the ordering has not changed. Against the development build the five worst are q24 at 0.828s above, q28 at 0.627s, q21 at 0.536s, q22 at 0.519s and q23 at 0.470s, which is 2.98s of the 4.53s rudb spends above it across every query where it is behind. The net across those thirty three is 2.58s. q24 is the single largest gap at 7.63x, and it is the one query where the development build is far ahead of its own release too, 0.125s against 0.321s, so part of that gap is a DuckDB improvement rather than a rudb regression.

## How to reproduce it

On `gamingpc-wsl`, from a copy of [tamnd/clickbench](https://github.com/tamnd/clickbench) with `hits.parquet` copied into the entry directory, with the `rudb` under test first on `PATH`:

```
BENCH_CONCURRENT_DURATION=0 ./benchmark.sh
```

`BENCH_CONCURRENT_DURATION=0` is there because rudb is a single process engine, which is [clickbench#946](https://github.com/ClickHouse/ClickBench/issues/946). The copy rather than a symlink matters: the entry's `load` ends in `rm -f hits.parquet`.

## What this number is not

It is one run of a driver that takes the best of three tries and checks no answers. This morning's report verified all forty three answers against DuckDB by hand at 0.3.45 and found no disagreement, only tie ordering under `LIMIT` and one case of CLI byte escaping; that verification has not been repeated at 0.3.57, so the correctness claim here covers q6's distinct counts and nothing else.

It is also not a 10x result and nothing in it suggests one is near. On the thirty three queries that are actually executed rather than looked up, rudb is level with a released DuckDB and 1.23x behind an unreleased one. The lead over the released build is 1.22x on the total, and it exists because eleven queries out of forty three never run.
