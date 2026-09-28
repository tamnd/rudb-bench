# ClickBench rerun at main 998cc427

RuDB was rebuilt from main `998cc427fc33d9f96d1f9aebd213954f9e309b14`. The optimization worktree was rebased onto the same revision. This run measures clean main, without the pending bound-reader patch. DuckDB is the released v1.5.5 build `d8cdaa33fd`. Both executables were frozen for the entire run.

All 43 canonical queries completed at four sample sizes in four cases. There are 5,504 successful timed query executions and eight native loads. This is a new Linux baseline. It must not be compared with the preceding saturated macOS run as evidence of a commit improvement. The 10x time and memory target remains unproven.

## Protocol

Every execution starts a fresh process. One first execution is retained separately, followed by seven fresh-process repetitions whose medians form the headline tables. The OS page cache is not flushed. Neighboring rounds run in reversed orders; the rotation advances after each pair. All eight rounds together balance positions and which engine runs first. Removing the first round leaves a small order imbalance in the seven-observation medians; raw readings retain all rounds.

Both engines use six threads and a 4GB memory limit. RuDB receives `SET stored_answers=false`, and every measured child receives `RUDB_PARQUET_MIRROR=0`. The four cases query the same rows. Native tables are loaded afresh using the same explicit schema, `INSERT INTO hits SELECT ... FROM read_parquet(...)`, and `CHECKPOINT` commands. Raw integer dates and times use the same normalization projection in both engines. No optional index or projection is built. Ordinary reusable column statistics remain enabled. No new query-specific result was introduced for this rerun.

Process wall time includes startup, opening files, SQL parsing, execution, rendering, and exit. A monotonic clock surrounds the native child; user plus system CPU and peak RSS come from wait4 for that child. Linux RSS is converted from KiB to bytes. Peak RSS is resident memory, not allocated bytes or a delta from startup. The separate CLI timer is retained, but its rounded values do not drive headline ratios. Query totals sum per-query medians; maximum query RSS is the largest peak among all eight executions, not a sum.

The host is a shared Linux Hyper-V guest on an Intel Core i9-13900K, with 32 visible logical CPUs and about 31 GiB RAM. It is not an exclusive benchmark machine. Before every load and query child, the harness waits until the one-minute load is at most 8, with a 600-second capacity timeout. Observed pre-child load ranged from 4.93 to 7.96; post-child load ranged from 4.93 to 10.15. This gate limits existing contention; it cannot prove stable CPU clocks or isolation. CPU, context switches, page faults, load, waiting time, stdout, and stderr are retained for every measured child.

The labels 1m and 10m name samples with 999,975 and 9,999,750 rows. They are not exact one-million and ten-million-row tables. The Linux source files also differ from the preceding macOS samples. Dataset hashes and actual row counts are recorded.

## Query comparison

| Sample | Format | DuckDB wall total (s) | RuDB wall total (s) | DuckDB / RuDB wall | DuckDB CPU total (s) | RuDB CPU total (s) | DuckDB max RSS (MiB) | RuDB max RSS (MiB) | DuckDB / RuDB max RSS |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1k | native | 0.462 | 0.091 | 5.07x | 0.569 | 0.090 | 33.30 | 14.27 | 2.33x |
| 1k | parquet | 0.481 | 0.090 | 5.34x | 0.590 | 0.089 | 32.93 | 13.80 | 2.39x |
| 10k | native | 0.507 | 0.124 | 4.08x | 0.648 | 0.127 | 36.70 | 15.86 | 2.31x |
| 10k | parquet | 0.545 | 0.150 | 3.64x | 0.693 | 0.149 | 37.39 | 15.68 | 2.38x |
| 1m | native | 1.083 | 0.272 | 3.99x | 3.195 | 0.652 | 219.51 | 115.47 | 1.90x |
| 1m | parquet | 1.375 | 1.519 | 0.91x | 4.214 | 5.771 | 360.91 | 310.05 | 1.16x |
| 10m | native | 4.565 | 1.101 | 4.14x | 22.571 | 4.293 | 1499.73 | 293.71 | 5.11x |
| 10m | parquet | 16.009 | 9.322 | 1.72x | 37.550 | 27.523 | 1238.29 | 1087.82 | 1.14x |

A ratio above one favors RuDB. All totals cover the same 43 completed queries. Ratios of totals differ from the geometric mean of per-query ratios. Memory during loading is separate and appears below.

The following sensitivity check excludes scalar Q1 through Q7, so metadata-friendly aggregates do not hide the remaining workload. This exclusion does not prove that every remaining query scans rows.

| Sample | Format | DuckDB Q8 to Q43 wall total (s) | RuDB Q8 to Q43 wall total (s) | DuckDB / RuDB wall |
| --- | --- | --- | --- | --- |
| 1k | native | 0.392 | 0.078 | 5.01x |
| 1k | parquet | 0.406 | 0.078 | 5.24x |
| 10k | native | 0.421 | 0.107 | 3.94x |
| 10k | parquet | 0.455 | 0.134 | 3.40x |
| 1m | native | 0.991 | 0.254 | 3.90x |
| 1m | parquet | 1.272 | 1.473 | 0.86x |
| 10m | native | 4.333 | 1.074 | 4.04x |
| 10m | parquet | 14.008 | 8.376 | 1.67x |

## Query-level target

| Sample | Format | Wall-ratio geometric mean | Queries at least 10x faster / 43 | Queries with at least 10x lower RSS / 43 | Queries meeting both / 43 |
| --- | --- | --- | --- | --- | --- |
| 1k | native | 5.08x | 0 | 0 | 0 |
| 1k | parquet | 5.38x | 0 | 0 | 0 |
| 10k | native | 4.10x | 0 | 0 | 0 |
| 10k | parquet | 3.94x | 0 | 0 | 0 |
| 1m | native | 4.17x | 3 | 1 | 1 |
| 1m | parquet | 1.41x | 0 | 0 | 0 |
| 10m | native | 4.47x | 9 | 9 | 8 |
| 10m | parquet | 1.89x | 0 | 0 | 0 |

These are performance counts only. Original answer differences are qualified below.

## Q1 through Q4 at 10m

| Query | Format | DuckDB wall (ms) | RuDB wall (ms) | DuckDB / RuDB wall | DuckDB median RSS (MiB) | RuDB median RSS (MiB) |
| --- | --- | --- | --- | --- | --- | --- |
| Q1 | native | 10.906 | 3.859 | 2.83x | 26.89 | 13.21 |
| Q1 | parquet | 248.294 | 106.779 | 2.33x | 150.17 | 118.88 |
| Q2 | native | 14.278 | 3.970 | 3.60x | 50.28 | 13.40 |
| Q2 | parquet | 267.876 | 106.567 | 2.51x | 158.63 | 118.84 |
| Q3 | native | 19.357 | 4.042 | 4.79x | 72.42 | 13.51 |
| Q3 | parquet | 341.408 | 122.746 | 2.78x | 174.16 | 118.87 |
| Q4 | native | 23.761 | 4.033 | 5.89x | 98.06 | 13.27 |
| Q4 | parquet | 263.176 | 116.246 | 2.26x | 158.91 | 118.86 |

## Native load cost

Each load has one observation. The same SQL builds the native database before query timing. Metadata construction is included. The 4GB engine memory setting is not a cap on whole-process RSS.

| Sample | Engine | Load wall (s) | Load CPU (s) | Peak RSS (MiB) | Native file (MiB) |
| --- | --- | --- | --- | --- | --- |
| 1k | duckdb-native | 0.082 | 0.068 | 59.90 | 1.01 |
| 1k | rudb-native | 0.134 | 0.154 | 41.29 | 1.79 |
| 10k | duckdb-native | 0.106 | 0.159 | 72.50 | 4.26 |
| 10k | rudb-native | 0.302 | 0.436 | 77.46 | 7.37 |
| 1m | duckdb-native | 2.807 | 10.314 | 1810.40 | 575.76 |
| 1m | rudb-native | 2.366 | 12.989 | 1132.38 | 235.44 |
| 10m | duckdb-native | 22.980 | 64.131 | 4637.11 | 2771.76 |
| 10m | rudb-native | 13.738 | 105.101 | 5083.52 | 1935.08 |

## Answer checks

Every original output was checked across all four cases and all eight repetitions. Integers are compared exactly, including values near 2^63. Floating point uses a 1e-9 tolerance. Equal rows with different order are reported separately. Differing selections are rerun with deterministic output-column tie breakers in separate untimed diagnostics; these do not replace the timed SQL or hide its original differences.

| Sample | Original match | Same rows, different order | Different selection, deterministic retest matches | Failed or unresolved |
| --- | --- | --- | --- | --- |
| 1k | 24 | 6 | 13 | 0 |
| 10k | 25 | 3 | 15 | 0 |
| 1m | 27 | 8 | 8 | 0 |
| 10m | 32 | 1 | 10 | 0 |

Q29 at 10m: **original match**. A matching diagnostic does not prove every originally selected row is correct; this report makes no blanket claim that all original answers are identical.

## Remaining gaps at 10m

This rerun establishes a baseline for profiling. It does not attribute a root cause from timing alone. The lowest wall ratios below identify queries needing attention.

| Format | Query | DuckDB wall (ms) | RuDB wall (ms) | DuckDB / RuDB wall | DuckDB median RSS (MiB) | RuDB median RSS (MiB) |
| --- | --- | --- | --- | --- | --- | --- |
| native | Q40 | 29.626 | 22.120 | 1.34x | 75.13 | 44.96 |
| native | Q39 | 16.172 | 10.365 | 1.56x | 41.69 | 26.95 |
| native | Q22 | 149.808 | 86.724 | 1.73x | 682.12 | 227.12 |
| native | Q27 | 22.772 | 11.908 | 1.91x | 63.15 | 27.87 |
| native | Q37 | 21.828 | 10.637 | 2.05x | 53.41 | 29.44 |
| native | Q38 | 18.352 | 8.657 | 2.12x | 42.30 | 24.33 |
| native | Q23 | 162.240 | 75.458 | 2.15x | 831.68 | 153.11 |
| parquet | Q23 | 444.798 | 520.282 | 0.85x | 238.72 | 169.26 |
| parquet | Q19 | 507.340 | 558.087 | 0.91x | 930.23 | 1076.61 |
| parquet | Q34 | 570.936 | 537.632 | 1.06x | 1143.20 | 955.59 |
| parquet | Q35 | 592.808 | 541.639 | 1.09x | 1234.79 | 962.77 |
| parquet | Q17 | 415.955 | 372.760 | 1.12x | 684.52 | 600.87 |
| parquet | Q22 | 413.541 | 317.311 | 1.30x | 247.36 | 163.59 |
| parquet | Q14 | 360.642 | 262.305 | 1.37x | 409.53 | 322.59 |

## Reproduce and inspect

The [full per-query tables](main-998cc427-clickbench/per-query.md), [raw repetitions](main-998cc427-clickbench/raw.jsonl.gz), [metadata](main-998cc427-clickbench/metadata.json), [correctness checks](main-998cc427-clickbench/correctness.json), and [provenance](main-998cc427-clickbench/provenance.json) retain the measured inputs and readings. The [output archive](main-998cc427-clickbench/outputs.tar.gz) retains CSV outputs, stderr, per-child resource records, canonical SQL, deterministic diagnostic SQL, and script snapshots. Database files, source Parquet files, and executable binaries are excluded. Their identities and paths are recorded, including [native file hashes](main-998cc427-clickbench/native-files.json). The [earlier bound-reader experiment](bound-header-diagnostic.md) is separate and does not contribute to these tables.

The source build used `CARGO_INCREMENTAL=0 cargo build --release -p rudb-cli --bin rudb -j8` on the isolated Linux source archive. The benchmark starts from clean main, not another worktree's executable. Use a new output directory and the exported normalization projection for these raw integer date/time samples.

```sh
cc -O2 -std=c11 -Wall -Wextra -Werror scripts/measure-child.c -o scripts/measure-child
cargo run --release --example export_clickbench -- "$OUTPUT/sql"
python3 scripts/clickbench-audit.py \
  --duckdb "$DUCKDB" --rudb "$RUDB" --data "$DATA" --output "$OUTPUT" \
  --sizes 1k 10k 1m 10m --hot 7 --threads 6 --memory-limit 4GB \
  --max-load 8 --idle-timeout 600 --timeout 120
python3 scripts/verify-clickbench-audit.py "$OUTPUT"
```

The measurement and ordering tests passed on macOS and Linux, including per-child RSS, failures, timeout handling, exact integers, mirroring suppression, and positional and pairwise order balance. The capacity gate and its timeout are tested separately. No engine change was merged for this rerun.
