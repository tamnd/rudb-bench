# ClickBench rerun at main d0ff4ee6

The 1k, 10k, and 1m samples and 10m Q1 through Q18 completed with the load gate of 8. The wait expired during Q19 after new workloads saturated the host. A separate continuation reran Q19 through Q43 in full without a capacity gate. The 10m totals combine these conditions and cannot establish a controlled speedup or the 10x target. The original journal, including 31 partial Q19 readings, is retained separately; those fragments do not contribute to the tables.

RuDB was rebuilt from clean main `d0ff4ee61032215b8a469f62ffb2587be6366a3e`, version 0.8.4. DuckDB is the released v1.5.5 build `d8cdaa33fd`. Both executables stayed fixed throughout the run. The pending topology, Parquet schema index, and bound-reader changes were rebased onto this main revision but are not included in the measured binary.

All 43 canonical queries completed at 1k, 10k, 1m, and 10m in four cases: DuckDB native, RuDB native, DuckDB Parquet, and RuDB Parquet. The tables use 5,504 selected query executions and eight native loads. The original attempt measured 4,735 query children; the continuation measured 800. The 31 partial Q19 readings are excluded from the selected groups and retained in the [original attempt archive](main-d0ff4ee6-clickbench/gated-attempt.tar.gz). The target of 10x faster queries and 10x less memory remains unproven.

## Measurement protocol

Every execution starts a fresh process. One first execution is retained separately, followed by seven fresh-process repetitions whose medians form the tables. The OS page cache is not flushed. Neighboring rounds reverse execution order and rotate after each pair. The eight rounds balance positions and which engine runs first; excluding the first round leaves a small order imbalance in the seven-observation medians.

Both engines are configured with six threads and a 4GB memory limit. RuDB uses `SET stored_answers=false`; measured children receive `RUDB_PARQUET_MIRROR=0`. Ordinary reusable column statistics remain enabled. No query-specific result or optional index was introduced. Native files were loaded afresh using identical explicit schema, `INSERT INTO hits SELECT ... FROM read_parquet(...)`, and `CHECKPOINT` SQL. The same normalization projection converts raw integer dates and timestamps in both engines. Parquet cases read the same source file.

RuDB uses its default `first` engine. This report does not measure the compiled engine, including the recently merged changes to compiled group-table probes and decimal operations.

A monotonic clock measures the whole child process, including startup, file opening, parsing, execution, rendering, and exit. User plus system CPU and peak RSS come from wait4 for that child. Linux RSS is converted from KiB to bytes. Peak RSS measures resident memory, not allocated bytes or an incremental delta. Rounded CLI query timers are retained but do not drive headline ratios. Totals sum per-query medians. Maximum RSS is the largest peak among all observations, not a sum.

The shared Linux Hyper-V host exposes 32 logical CPUs on an Intel Core i9-13900K and about 31 GiB RAM. Selected pre-child load ranged from 5.22 to 21.46; post-child load ranged from 5.22 to 21.46. Each selected group records its capacity limit and source journal. A load gate does not provide exclusive CPUs or stable clocks; the ungated continuation is a contention diagnostic. Load, waiting time, context switches, faults, CPU, stdout, and stderr are retained.

The 1m and 10m samples contain 999,975 and 9,999,750 rows. Source files are the same Linux samples used in the preceding main-998cc427 report. Actual row counts and hashes are recorded.

## Query comparison

| Sample | Format | DuckDB wall total (s) | RuDB wall total (s) | DuckDB / RuDB wall | DuckDB CPU total (s) | RuDB CPU total (s) | DuckDB max RSS (MiB) | RuDB max RSS (MiB) | DuckDB / RuDB max RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | native | 0.515 | 0.094 | 5.49x | 0.628 | 0.093 | 33.74 | 13.78 | 2.45x |
| 1k | parquet | 0.536 | 0.104 | 5.17x | 0.656 | 0.102 | 32.93 | 13.21 | 2.49x |
| 10k | native | 0.534 | 0.117 | 4.58x | 0.668 | 0.118 | 37.02 | 15.19 | 2.44x |
| 10k | parquet | 0.570 | 0.155 | 3.68x | 0.712 | 0.153 | 37.73 | 16.12 | 2.34x |
| 1m | native | 1.054 | 0.283 | 3.72x | 3.081 | 0.663 | 219.20 | 115.88 | 1.89x |
| 1m | parquet | 1.342 | 1.455 | 0.92x | 4.079 | 5.580 | 359.57 | 311.36 | 1.15x |
| 10m | native | 5.522 | 1.341 | 4.12x | 25.594 | 4.938 | 1497.43 | 276.31 | 5.42x |
| 10m | parquet | 17.338 | 10.166 | 1.71x | 41.564 | 30.725 | 1240.74 | 1088.79 | 1.14x |

Ratios above one favor RuDB. All totals cover the same 43 completed queries. A ratio of totals differs from a geometric mean of per-query ratios.

The following table excludes Q1 through Q7 to show the remaining workload separately. It does not prove that every remaining query scans rows.

| Sample | Format | DuckDB Q8 to Q43 wall total (s) | RuDB Q8 to Q43 wall total (s) | DuckDB / RuDB wall |
| --- | --- | ---: | ---: | ---: |
| 1k | native | 0.444 | 0.082 | 5.43x |
| 1k | parquet | 0.460 | 0.090 | 5.12x |
| 10k | native | 0.448 | 0.100 | 4.46x |
| 10k | parquet | 0.478 | 0.137 | 3.49x |
| 1m | native | 0.961 | 0.269 | 3.57x |
| 1m | parquet | 1.238 | 1.409 | 0.88x |
| 10m | native | 5.292 | 1.318 | 4.01x |
| 10m | parquet | 15.426 | 9.232 | 1.67x |

## Query-level target

| Sample | Format | Wall-ratio geometric mean | Queries at least 10x faster / 43 | Queries with at least 10x lower peak RSS / 43 | Queries meeting both / 43 |
| --- | --- | ---: | ---: | ---: | ---: |
| 1k | native | 5.48x | 0 | 0 | 0 |
| 1k | parquet | 5.20x | 0 | 0 | 0 |
| 10k | native | 4.53x | 0 | 0 | 0 |
| 10k | parquet | 3.87x | 0 | 0 | 0 |
| 1m | native | 4.54x | 7 | 1 | 1 |
| 1m | parquet | 1.44x | 0 | 0 | 0 |
| 10m | native | 5.02x | 9 | 10 | 9 |
| 10m | parquet | 1.90x | 0 | 0 | 0 |

The 10m counts combine capacity conditions and do not establish the target under controlled conditions. These counts use per-query wall medians and the largest observed RSS in each engine. They measure performance, with answer differences qualified below.

## Q1 through Q4 at 10m

| Query | Format | DuckDB wall (ms) | RuDB wall (ms) | DuckDB / RuDB wall | DuckDB median RSS (MiB) | RuDB median RSS (MiB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Q1 | native | 11.204 | 3.199 | 3.50x | 26.74 | 12.28 |
| Q1 | parquet | 249.082 | 109.897 | 2.27x | 150.02 | 118.30 |
| Q2 | native | 14.818 | 3.317 | 4.47x | 49.99 | 12.57 |
| Q2 | parquet | 258.121 | 113.512 | 2.27x | 158.26 | 118.30 |
| Q3 | native | 18.557 | 3.338 | 5.56x | 72.22 | 12.77 |
| Q3 | parquet | 261.567 | 111.027 | 2.36x | 174.08 | 118.27 |
| Q4 | native | 23.756 | 3.237 | 7.34x | 98.02 | 12.52 |
| Q4 | parquet | 264.081 | 111.945 | 2.36x | 158.52 | 118.30 |

## Native load cost

Each load has one observation and includes building ordinary metadata and checkpointing. The 4GB engine setting is not a cap on whole-process RSS.

The six-thread setting does not cap all native commit work in this RuDB revision. Numeric summary closing and dictionary encoding choose hardware-based worker counts up to 32. The 10m RuDB load uses 7.448 average CPU core equivalents, measured as CPU seconds divided by wall seconds, compared with 2.568 for DuckDB. Its shorter wall time uses more CPU. The load wall comparison does not hold whole-process CPU concurrency equal.

| Sample | Engine | Load wall (s) | Load CPU (s) | Peak RSS (MiB) | Native file (MiB) |
| --- | --- | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | 0.053 | 0.074 | 61.39 | 1.01 |
| 1k | rudb-native | 0.078 | 0.094 | 38.73 | 1.79 |
| 10k | duckdb-native | 0.105 | 0.171 | 66.28 | 4.01 |
| 10k | rudb-native | 0.307 | 0.408 | 71.86 | 7.37 |
| 1m | duckdb-native | 3.401 | 9.236 | 1998.99 | 573.01 |
| 1m | rudb-native | 1.981 | 11.764 | 1097.77 | 235.25 |
| 10m | duckdb-native | 28.248 | 72.536 | 4771.66 | 2780.01 |
| 10m | rudb-native | 15.523 | 115.612 | 5190.93 | 1935.46 |

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
| native | Q40 | 29.328 | 22.497 | 1.30x | 75.22 | 44.60 |
| native | Q22 | 143.780 | 92.585 | 1.55x | 674.98 | 109.12 |
| native | Q39 | 17.158 | 10.164 | 1.69x | 41.92 | 27.16 |
| native | Q21 | 157.822 | 84.838 | 1.86x | 614.72 | 107.69 |
| native | Q38 | 15.923 | 7.727 | 2.06x | 41.10 | 24.59 |
| native | Q27 | 23.525 | 10.846 | 2.17x | 62.93 | 28.15 |
| native | Q37 | 21.185 | 9.749 | 2.17x | 53.48 | 29.43 |
| parquet | Q19 | 548.811 | 590.826 | 0.93x | 928.11 | 1067.35 |
| parquet | Q23 | 631.724 | 647.022 | 0.98x | 238.37 | 168.88 |
| parquet | Q34 | 574.986 | 554.755 | 1.04x | 1143.20 | 972.35 |
| parquet | Q35 | 584.041 | 529.516 | 1.10x | 1233.93 | 969.75 |
| parquet | Q17 | 433.162 | 369.477 | 1.17x | 684.44 | 603.14 |
| parquet | Q13 | 337.985 | 286.953 | 1.18x | 316.52 | 237.88 |
| parquet | Q28 | 432.412 | 355.598 | 1.22x | 253.34 | 166.57 |

## Reproduce and inspect

The [build input audit](main-d0ff4ee6-clickbench/build-input-audit.json) records a rejected build that reused the preceding 8d09c34b executable without recompiling changed d0ff4ee6 source. It was rejected before any timed child. A new target directory rebuilt the source and produced measured SHA-256 `b8e3bf1f7e3d950c695297aed1ec546f60ffb1dbe6a59b03d4190314ea996fed`. The build log is in the output archive. Use an empty target directory for a new source cutoff and verify the source, build log, and executable identity.

The [per-query tables](main-d0ff4ee6-clickbench/per-query.md), [raw repetitions](main-d0ff4ee6-clickbench/raw.jsonl.gz), [metadata](main-d0ff4ee6-clickbench/metadata.json), [answer checks](main-d0ff4ee6-clickbench/correctness.json), [native file hashes](main-d0ff4ee6-clickbench/native-files.json), and [provenance](main-d0ff4ee6-clickbench/provenance.json) retain the evidence. The [output archive](main-d0ff4ee6-clickbench/outputs.tar.gz) contains every CSV, stderr and resource record, original SQL, deterministic diagnostic SQL, script snapshots, and the build log. Binaries and data files are excluded; their hashes and paths are recorded.

Build clean main with `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$NEW_TARGET_DIR" cargo build --release -p rudb-cli --bin rudb -j8`. The repository pins Rust 1.98.0 and compiles x86_64 for x86-64-v3. The benchmark helper uses `cc -O2 -std=c11 -Wall -Wextra -Werror`. All 15 measurement, ordering, capacity, and continuation tests passed on macOS and Linux, each with one platform-specific skip.

```sh
python3 scripts/clickbench-audit.py \
  --duckdb "$DUCKDB" --rudb "$RUDB" --data "$DATA" --output "$OUTPUT" \
  --sizes 1k 10k 1m 10m --hot 7 --threads 6 --memory-limit 4GB \
  --max-load 8 --idle-timeout 600 --timeout 180
python3 scripts/verify-clickbench-audit.py "$OUTPUT"
```

If a capacity wait expires, preserve its journal and finish with a new output directory. The continuation verifies executable, Parquet, and SQL hashes, retains only complete original groups, and reruns every incomplete group across all four cases and all eight executions. Run its answer checks against the combined selected groups. Its per-group metadata makes changes in capacity conditions visible.

```sh
python3 scripts/continue-clickbench-audit.py "$OUTPUT" "$CONTINUATION" --timeout 180
python3 scripts/verify-clickbench-audit.py "$CONTINUATION"
```

The continuation snapshots are in the output archive. SQL errors emitted with exit status zero are rejected, so a timer from an earlier statement cannot make a failed query count as successful.

Export the canonical benchmark SQL into a new output directory first. Reuse the recorded normalization projection for these raw integer date and time samples. No engine optimization was merged as part of this rerun.
