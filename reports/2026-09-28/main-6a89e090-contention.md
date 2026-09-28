# ClickBench rerun at main 6a89e090

This is a contention diagnostic. The capacity-gated attempt stopped before completing the first sample because new workloads saturated the host. This separate rerun records all original SQL without a load gate. It must not replace the preceding quiet baseline or support a claimed commit speedup or achievement of the 10x target.

The [incomplete gated attempt](main-6a89e090-contention/capacity-attempt.tar.gz) retains all 915 readings, outputs, settings, and the reason the capacity wait was ended. It does not contribute to the tables below.

RuDB was rebuilt from clean main `6a89e090e227ab62864ed26952ed5a84ae63a0f8`, version 0.8.4. DuckDB is the released v1.5.5 build `d8cdaa33fd`. Both executables stayed fixed throughout the run. The pending topology, Parquet schema index, and bound-reader changes were rebased onto this main revision but are not included in the measured binary.

All 43 canonical queries completed at 1k, 10k, 1m, and 10m in four cases: DuckDB native, RuDB native, DuckDB Parquet, and RuDB Parquet. This run has 5,504 successful timed query children and eight native loads. The target of 10x faster queries and 10x less memory remains unproven.

## Measurement protocol

Every execution starts a fresh process. One first execution is retained separately, followed by seven fresh-process repetitions whose medians form the tables. The OS page cache is not flushed. Neighboring rounds reverse execution order and rotate after each pair. The eight rounds balance positions and which engine runs first; excluding the first round leaves a small order imbalance in the seven-observation medians.

Both engines use six threads and a 4GB memory limit. RuDB uses `SET stored_answers=false`; measured children receive `RUDB_PARQUET_MIRROR=0`. Ordinary reusable column statistics remain enabled. No query-specific result or optional index was introduced. Native files were loaded afresh using identical explicit schema, `INSERT INTO hits SELECT ... FROM read_parquet(...)`, and `CHECKPOINT` SQL. The same normalization projection converts raw integer dates and timestamps in both engines. Parquet cases read the same source file.

RuDB uses its default `first` engine. This report does not measure the compiled engine. Main advanced to `8d09c34b` and then `d0ff4ee6` after this run's source cutoff; those changes do not relabel this executable or its readings.

A monotonic clock measures the whole child process, including startup, file opening, parsing, execution, rendering, and exit. User plus system CPU and peak RSS come from wait4 for that child. Linux RSS is converted from KiB to bytes. Peak RSS measures resident memory, not allocated bytes or an incremental delta. Rounded CLI query timers are retained but do not drive headline ratios. Totals sum per-query medians. Maximum RSS is the largest peak among all observations, not a sum.

The shared Linux Hyper-V host exposes 32 logical CPUs on an Intel Core i9-13900K and about 31 GiB RAM. This diagnostic has no capacity gate or exclusive CPUs. Pre-child load ranged from 32.41 to 68.90; post-child load ranged from 32.41 to 68.90. Load, context switches, faults, CPU, stdout, and stderr are retained. Contention can affect engines differently, and measurements here do not establish an engine speedup.

The 1m and 10m samples contain 999,975 and 9,999,750 rows. Source files are the same Linux samples used in the preceding main-998cc427 report. Actual row counts and hashes are recorded.

## Query comparison

| Sample | Format | DuckDB wall total (s) | RuDB wall total (s) | DuckDB / RuDB wall | DuckDB CPU total (s) | RuDB CPU total (s) | DuckDB max RSS (MiB) | RuDB max RSS (MiB) | DuckDB / RuDB max RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | native | 2.614 | 0.344 | 7.61x | 1.966 | 0.237 | 34.31 | 14.34 | 2.39x |
| 1k | parquet | 2.698 | 0.364 | 7.40x | 1.988 | 0.263 | 32.48 | 13.61 | 2.39x |
| 10k | native | 3.059 | 0.552 | 5.55x | 2.005 | 0.304 | 36.79 | 15.09 | 2.44x |
| 10k | parquet | 3.300 | 0.694 | 4.76x | 2.336 | 0.448 | 37.24 | 16.29 | 2.29x |
| 1m | native | 8.934 | 1.806 | 4.95x | 10.445 | 1.635 | 235.23 | 104.94 | 2.24x |
| 1m | parquet | 10.774 | 9.837 | 1.10x | 14.902 | 16.364 | 357.64 | 294.38 | 1.21x |
| 10m | native | 30.094 | 6.021 | 5.00x | 57.943 | 10.253 | 1497.30 | 276.22 | 5.42x |
| 10m | parquet | 55.489 | 39.477 | 1.41x | 85.818 | 62.378 | 1209.98 | 1094.44 | 1.11x |

Ratios above one favor RuDB. All totals cover the same 43 completed queries. A ratio of totals differs from a geometric mean of per-query ratios.

The following table excludes Q1 through Q7 to show the remaining workload separately. It does not prove that every remaining query scans rows.

| Sample | Format | DuckDB Q8 to Q43 wall total (s) | RuDB Q8 to Q43 wall total (s) | DuckDB / RuDB wall |
| --- | --- | ---: | ---: | ---: |
| 1k | native | 2.255 | 0.311 | 7.25x |
| 1k | parquet | 2.374 | 0.320 | 7.42x |
| 10k | native | 2.650 | 0.512 | 5.18x |
| 10k | parquet | 2.892 | 0.649 | 4.46x |
| 1m | native | 8.190 | 1.722 | 4.76x |
| 1m | parquet | 9.963 | 9.448 | 1.05x |
| 10m | native | 28.775 | 5.922 | 4.86x |
| 10m | parquet | 48.625 | 35.444 | 1.37x |

## Query-level target

| Sample | Format | Wall-ratio geometric mean | Queries at least 10x faster / 43 | Queries with at least 10x lower peak RSS / 43 | Queries meeting both / 43 |
| --- | --- | ---: | ---: | ---: | ---: |
| 1k | native | 8.10x | 15 | 0 | 0 |
| 1k | parquet | 7.79x | 12 | 0 | 0 |
| 10k | native | 5.84x | 4 | 0 | 0 |
| 10k | parquet | 5.85x | 8 | 0 | 0 |
| 1m | native | 5.85x | 9 | 3 | 3 |
| 1m | parquet | 1.61x | 0 | 0 | 0 |
| 10m | native | 6.24x | 12 | 10 | 10 |
| 10m | parquet | 1.80x | 0 | 0 | 0 |

These diagnostic counts do not establish the target under controlled conditions. They use per-query wall medians and the largest observed RSS in each engine. They measure performance, with answer differences qualified below.

## Q1 through Q4 at 10m

| Query | Format | DuckDB wall (ms) | RuDB wall (ms) | DuckDB / RuDB wall | DuckDB median RSS (MiB) | RuDB median RSS (MiB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Q1 | native | 100.000 | 17.182 | 5.82x | 27.01 | 12.39 |
| Q1 | parquet | 864.328 | 400.613 | 2.16x | 150.06 | 117.95 |
| Q2 | native | 84.567 | 19.108 | 4.43x | 50.61 | 12.56 |
| Q2 | parquet | 1013.725 | 527.057 | 1.92x | 159.02 | 117.98 |
| Q3 | native | 149.517 | 8.640 | 17.30x | 72.44 | 12.55 |
| Q3 | parquet | 827.641 | 296.933 | 2.79x | 174.14 | 118.02 |
| Q4 | native | 158.047 | 10.756 | 14.69x | 98.10 | 12.56 |
| Q4 | parquet | 979.287 | 914.239 | 1.07x | 158.84 | 118.10 |

## Native load cost

Each load has one observation and includes building ordinary metadata and checkpointing. The 4GB engine setting is not a cap on whole-process RSS.

| Sample | Engine | Load wall (s) | Load CPU (s) | Peak RSS (MiB) | Native file (MiB) |
| --- | --- | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | 0.290 | 0.402 | 60.80 | 1.01 |
| 1k | rudb-native | 0.261 | 0.219 | 38.71 | 1.79 |
| 10k | duckdb-native | 0.941 | 1.534 | 72.79 | 4.01 |
| 10k | rudb-native | 1.729 | 1.919 | 81.50 | 7.37 |
| 1m | duckdb-native | 8.898 | 23.457 | 1884.60 | 571.51 |
| 1m | rudb-native | 7.629 | 20.142 | 1052.53 | 235.61 |
| 10m | duckdb-native | 72.604 | 148.976 | 4258.34 | 2774.26 |
| 10m | rudb-native | 47.724 | 173.705 | 4789.25 | 1934.75 |

## Answer checks

All original outputs were checked across four cases and eight executions. Integers are exact, including values near 2^63; floating point uses a 1e-9 tolerance. Differing order and selections remain visible. Separate untimed retests add deterministic output-column tie breakers without replacing timed SQL or its original results.

| Sample | Original match | Same rows, different order | Different selection, deterministic retest matches | Failed or unresolved |
| --- | ---: | ---: | ---: | ---: |
| 1k | 24 | 6 | 13 | 0 |
| 10k | 25 | 3 | 15 | 0 |
| 1m | 27 | 8 | 8 | 0 |
| 10m | 32 | 1 | 10 | 0 |

Q29 at 10m: **original match**. Matching deterministic retests do not establish correctness of every originally selected row. This report does not claim all original outputs are identical.

## Remaining gaps at 10m

The slowest wall ratios identify where to profile next. Timing alone does not establish the cause.

| Format | Query | DuckDB wall (ms) | RuDB wall (ms) | DuckDB / RuDB wall | DuckDB median RSS (MiB) | RuDB median RSS (MiB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| native | Q42 | 37.967 | 20.010 | 1.90x | 38.88 | 20.59 |
| native | Q39 | 28.500 | 14.748 | 1.93x | 41.98 | 26.62 |
| native | Q40 | 76.048 | 35.028 | 2.17x | 74.56 | 44.52 |
| native | Q26 | 215.019 | 96.799 | 2.22x | 88.64 | 29.54 |
| native | Q25 | 165.778 | 71.005 | 2.33x | 63.02 | 23.52 |
| native | Q27 | 144.800 | 59.832 | 2.42x | 62.95 | 27.28 |
| native | Q9 | 563.670 | 231.980 | 2.43x | 267.75 | 133.96 |
| parquet | Q35 | 1541.210 | 2442.041 | 0.63x | 1174.29 | 969.92 |
| parquet | Q34 | 2432.147 | 3545.905 | 0.69x | 1139.47 | 977.71 |
| parquet | Q6 | 782.866 | 1080.631 | 0.72x | 302.20 | 256.58 |
| parquet | Q19 | 2633.156 | 3619.459 | 0.73x | 930.02 | 1083.96 |
| parquet | Q14 | 1267.178 | 1519.099 | 0.83x | 408.20 | 321.02 |
| parquet | Q17 | 3060.075 | 3413.284 | 0.90x | 616.78 | 603.52 |
| parquet | Q23 | 1400.794 | 1461.498 | 0.96x | 235.61 | 165.69 |

## Reproduce and inspect

The [per-query tables](main-6a89e090-contention/per-query.md), [raw repetitions](main-6a89e090-contention/raw.jsonl.gz), [metadata](main-6a89e090-contention/metadata.json), [answer checks](main-6a89e090-contention/correctness.json), [native file hashes](main-6a89e090-contention/native-files.json), and [provenance](main-6a89e090-contention/provenance.json) retain the evidence. The [output archive](main-6a89e090-contention/outputs.tar.gz) contains every CSV, stderr and resource record, original SQL, deterministic diagnostic SQL, script snapshots, and the build log. Binaries and data files are excluded; their hashes and paths are recorded.

Build clean main with `CARGO_INCREMENTAL=0 cargo build --release -p rudb-cli --bin rudb -j8`. The repository pins Rust 1.98.0 and compiles x86_64 for x86-64-v3. The benchmark helper uses `cc -O2 -std=c11 -Wall -Wextra -Werror`. Measurement and ordering tests passed on macOS and Linux, each with one platform-specific skip.

```sh
python3 scripts/clickbench-audit.py \
  --duckdb "$DUCKDB" --rudb "$RUDB" --data "$DATA" --output "$OUTPUT" \
  --sizes 1k 10k 1m 10m --hot 7 --threads 6 --memory-limit 4GB \
  --timeout 180
python3 scripts/verify-clickbench-audit.py "$OUTPUT"
```

Export the canonical benchmark SQL into a new output directory first. Reuse the recorded normalization projection for these raw integer date and time samples. No engine optimization was merged as part of this rerun.
