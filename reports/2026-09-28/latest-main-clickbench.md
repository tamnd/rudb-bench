# Latest main ClickBench rerun, 2026-09-28

RuDB was rebased onto main `cd9d493c0df4053cb0e34fc07cd1e8652b77fa76` and rebuilt before this run. The measured binary stayed fixed throughout. DuckDB is the released v1.5.5 build `d8cdaa33fd`. This is a diagnostic baseline. The host was heavily saturated during this final run, with load above 50 on 10 hardware threads. Wall ratios from this run cannot establish an improvement or verify the 10x goal. Do not compare them against earlier reports as a commit effect.

In this saturated-host run at 10m rows, the sum of per-query process-wall medians is 9.705 seconds for DuckDB native and 1.760 seconds for RuDB native, a 5.51x ratio. For Parquet it is 12.675 seconds and 8.814 seconds, a 1.44x ratio. These are observed diagnostic totals, not reliable quiet-host speedups. The 10x time and memory goal has not been demonstrated.

## Measurement protocol

All 43 canonical ClickBench queries ran on the same 1k, 10k, 1m, and 10m samples in four cases: DuckDB native, RuDB native, DuckDB Parquet, and RuDB Parquet. There are 4,128 successful timed query executions and eight native loads. Each query has one first execution and five subsequent repetitions. Every repetition starts a fresh process, and the four cases rotate order on each query and repetition. The OS page cache is not flushed. These are warm-cache sample measurements, not official ClickBench scores.

Both engines receive `SET threads=6; SET memory_limit='4GB'` for loads and queries. Every RuDB query also receives `SET stored_answers=false`, and every measured child receives `RUDB_PARQUET_MIRROR=0`. The settings are read back and retained in the metadata. Native files were loaded afresh using the same `CREATE TABLE`, `INSERT INTO ... SELECT * FROM read_parquet(...)`, and `CHECKPOINT` SQL. The samples already contain typed DATE and TIMESTAMP columns, so both engines use an identity projection. No optional projection or index was built after the native load.

Reusable column statistics remain enabled. Some scalar queries can therefore use native metadata rather than scan rows. The current writer does not save pair leaders or derived host-group query results, and the reader ignores legacy results of those kinds. The table below also gives totals excluding Q1 through Q7 so their scalar shapes do not hide the rest of the workload. This exclusion is a sensitivity check, not proof that every other query scans rows.

Process wall time is measured with a monotonic clock around the native child. CPU is user plus system time, and peak RSS comes from `wait4` for that exact child. The macOS RSS reading is already in bytes; Linux converts KiB to bytes. macOS I/O byte readings are unavailable and stored as null. The CLI query timer, including result rendering, is retained separately. Headline totals sum the five-repetition median for each query, including process startup. Peak RSS in the summary is the largest observed query-process peak, not a sum, allocation count, or average.

The host is an Apple M4 Mac16,13 with 10 hardware threads and 24 GiB RAM, running macOS 15.8.1. It was shared with other work and had substantial existing swap use. Per-execution query load averages ranged from 18.03 to 45.94; a separate observation during native loading reached 53.73. Timing spread is retained. Rotating order reduces ordering bias, but these results do not establish quiet-host latency. Loads have one observation each and ran sequentially under changing contention, so their ratios cannot establish a load-speed improvement.

## Query comparison

| Rows | Format | DuckDB wall total (s) | RuDB wall total (s) | DuckDB / RuDB wall | DuckDB CPU total (s) | RuDB CPU total (s) | DuckDB max RSS (MiB) | RuDB max RSS (MiB) | DuckDB / RuDB max RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | native | 1.630 | 0.563 | 2.89x | 1.468 | 0.452 | 35.66 | 13.34 | 2.67x |
| 1k | parquet | 1.582 | 0.537 | 2.94x | 1.476 | 0.433 | 33.02 | 13.11 | 2.52x |
| 10k | native | 3.952 | 1.165 | 3.39x | 2.059 | 0.727 | 38.61 | 16.50 | 2.34x |
| 10k | parquet | 4.292 | 1.054 | 4.07x | 2.112 | 0.697 | 40.77 | 16.06 | 2.54x |
| 1m | native | 3.036 | 0.914 | 3.32x | 5.665 | 1.398 | 259.69 | 82.20 | 3.16x |
| 1m | parquet | 3.895 | 2.790 | 1.40x | 7.022 | 5.657 | 385.80 | 333.89 | 1.16x |
| 10m | native | 9.705 | 1.760 | 5.51x | 29.653 | 5.054 | 1155.86 | 282.33 | 4.09x |
| 10m | parquet | 12.675 | 8.814 | 1.44x | 39.139 | 34.239 | 948.27 | 829.09 | 1.14x |

All cases completed the same 43 queries. A ratio above one favors RuDB. Ratios of totals are different from the geometric mean of query ratios.

| Rows | Format | DuckDB Q8 to Q43 wall total (s) | RuDB Q8 to Q43 wall total (s) | DuckDB / RuDB wall |
| --- | --- | ---: | ---: | ---: |
| 1k | native | 1.396 | 0.476 | 2.93x |
| 1k | parquet | 1.342 | 0.451 | 2.97x |
| 10k | native | 3.616 | 1.052 | 3.44x |
| 10k | parquet | 3.974 | 0.956 | 4.16x |
| 1m | native | 2.758 | 0.826 | 3.34x |
| 1m | parquet | 3.583 | 2.650 | 1.35x |
| 10m | native | 9.280 | 1.666 | 5.57x |
| 10m | parquet | 12.007 | 8.352 | 1.44x |

For the query-level target, both wall and RSS use medians of the five subsequent fresh processes. The counts below cover timed performance only. Original tied row selections remain qualified by the correctness checks.

| Rows | Format | Wall-ratio geometric mean | Queries at least 10x faster / 43 | Queries with at least 10x lower RSS / 43 | Queries meeting both / 43 |
| --- | --- | ---: | ---: | ---: | ---: |
| 1k | native | 2.88x | 0 | 0 | 0 |
| 1k | parquet | 2.98x | 0 | 0 | 0 |
| 10k | native | 3.02x | 0 | 0 | 0 |
| 10k | parquet | 3.24x | 3 | 0 | 0 |
| 1m | native | 2.97x | 0 | 3 | 0 |
| 1m | parquet | 1.73x | 0 | 0 | 0 |
| 10m | native | 4.00x | 6 | 6 | 6 |
| 10m | parquet | 1.49x | 0 | 0 | 0 |

## Native load cost

| Rows | Engine | Load wall (s) | Load CPU (s) | Load peak RSS (MiB) | Native file (MiB) |
| --- | --- | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | 0.085 | 0.083 | 61.53 | 1.01 |
| 1k | rudb-native | 0.183 | 0.164 | 41.14 | 1.79 |
| 10k | duckdb-native | 0.302 | 0.326 | 77.36 | 4.76 |
| 10k | rudb-native | 1.041 | 0.955 | 72.19 | 7.37 |
| 1m | duckdb-native | 13.297 | 13.940 | 968.94 | 430.76 |
| 1m | rudb-native | 5.922 | 13.409 | 1096.81 | 235.22 |
| 10m | duckdb-native | 286.881 | 120.874 | 820.33 | 2257.26 |
| 10m | rudb-native | 31.599 | 85.186 | 1492.08 | 1400.52 |

Load memory is reported separately from query memory. Native metadata construction is included in the RuDB load.

## Answer checks

Every retained output was checked across all four cases and all six repetitions, using exact integer comparisons and a 1e-9 tolerance for floating point. Q29 at 10m matches on all 24 original executions.

| Rows | Original answers match | Same rows, different order | Different selection, deterministic retest matches | Failed or unresolved |
| --- | ---: | ---: | ---: | ---: |
| 1k | 24 | 6 | 13 | 0 |
| 10k | 25 | 3 | 15 | 0 |
| 1m | 27 | 8 | 8 | 0 |
| 10m | 32 | 4 | 7 | 0 |

Of 172 size/query combinations, {original} original outputs match and {order} contain the same rows in a different order. The remaining {selection} select different rows. All {selection} match after adding deterministic output-column tie breakers in separate untimed queries. The original SQL, outputs, and differences remain intact. A matching diagnostic does not prove that each originally selected row was correct; no blanket claim of identical original answers is made.

## Remaining performance gaps at 10m

This run gives a baseline for profiling, not a new root-cause diagnosis. The query-level table above shows how few queries meet both 10x targets. The weakest wall ratios below show where the goal is still furthest away.

| Format | Query | DuckDB wall (ms) | RuDB wall (ms) | DuckDB / RuDB wall | DuckDB median RSS (MiB) | RuDB median RSS (MiB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| native | Q41 | 29.903 | 16.842 | 1.78x | 46.81 | 38.88 |
| native | Q40 | 93.577 | 51.025 | 1.83x | 236.23 | 90.12 |
| native | Q42 | 29.891 | 16.171 | 1.85x | 40.88 | 30.73 |
| native | Q43 | 36.293 | 19.355 | 1.88x | 37.11 | 26.59 |
| native | Q38 | 35.484 | 18.470 | 1.92x | 46.98 | 35.39 |
| native | Q27 | 31.603 | 16.185 | 1.95x | 39.62 | 24.34 |
| native | Q8 | 30.299 | 14.710 | 2.06x | 29.67 | 20.09 |
| parquet | Q23 | 483.717 | 826.235 | 0.59x | 637.09 | 445.44 |
| parquet | Q34 | 720.114 | 944.076 | 0.76x | 914.94 | 811.52 |
| parquet | Q35 | 634.299 | 830.221 | 0.76x | 926.81 | 801.83 |
| parquet | Q27 | 100.825 | 128.054 | 0.79x | 106.78 | 155.17 |
| parquet | Q6 | 181.003 | 223.239 | 0.81x | 245.89 | 200.62 |
| parquet | Q17 | 265.302 | 315.630 | 0.84x | 319.34 | 357.28 |
| parquet | Q24 | 355.306 | 414.716 | 0.86x | 451.56 | 349.11 |

Q9 here opens the ordinary freshly loaded native table. It is not the optional indexed Q9 experiment in the earlier reports.

## Reproduce and inspect

The [per-query tables](latest-main-clickbench/per-query.md), [raw repetitions](latest-main-clickbench/raw.jsonl.gz), [correctness results](latest-main-clickbench/correctness.json), [metadata](latest-main-clickbench/metadata.json), and [provenance](latest-main-clickbench/provenance.json) retain the measurements. The [output archive](latest-main-clickbench/outputs.tar.gz) contains every original CSV, stderr, resource record, SQL file, deterministic diagnostic, and the measured script snapshots. Native databases and source Parquet files are excluded from the archive. Their local paths and source hashes are recorded.

The first run allowed RuDB's default Parquet mirroring, which writes and reuses a native file for sufficiently large Parquet inputs. Its 10m RuDB Parquet case is not a pure Parquet comparison. That run is withdrawn as a four-way comparison and excluded from every table above. Its [diagnostic archive](latest-main-clickbench/mirror-diagnostic.tar.gz) is retained with a warning. The corrected run rebuilt all native files in a new directory and explicitly disabled mirroring for every measured child.

The harness tests passed on macOS and Linux, including per-child RSS, child failures, timeouts, exact large integers, and the mirroring environment override. The benchmark Rust suite passed 395 library, 23 binary, four integration, and one additional test; strict all-target Clippy passed. No engine source changes were made for this rerun.

```sh
cc -O2 -Wall -Wextra -Werror scripts/measure-child.c -o scripts/measure-child
cargo run --release --example export_clickbench -- "$OUTPUT/sql"
# For these already typed samples, set projection.sql to a single *.
printf "*\n" > "$OUTPUT/sql/projection.sql"
python3 scripts/clickbench-audit.py \
  --duckdb "$DUCKDB" --rudb "$RUDB" --data "$DATA" --output "$OUTPUT" \
  --sizes 1k 10k 1m 10m --hot 5 --threads 6 --memory-limit 4GB --timeout 600
python3 scripts/verify-clickbench-audit.py "$OUTPUT"
```

Use a new output directory and the exact source samples. Upstream raw integer date/time files require the exported normalization projection instead of this identity projection. Binary and source SHA-256 values are in the metadata. RuDB executable SHA-256 is `22594531a28dbf8841dd10fe0690facff5a15200d33c54808863186b7ccfed31`. The benchmark base revision is `0f2b0af5817fef1c4417a98909e09327bb1df675`, with the script changes in this report's PR.

Main changed while the first corrected run was in progress. Aggregate merge PR #2068 landed, followed by release 0.8.2. The engine was rebased and rebuilt again before the final run above. The [preceding corrected baseline](latest-main-clickbench/before-aggregate-merge-0fc61d7e.tar.gz) preserves the earlier binary and its full report separately. Successive shared-host runs do not isolate the effect of that commit.
