# ClickBench rerun at main 0e5f97bd on Apple M4

All 43 canonical queries completed at 1k, 10k, 1m, and 10m across DuckDB native, RuDB native, DuckDB Parquet, and RuDB Parquet. This run retains 5,504 fresh-process query executions and eight fresh native loads. The complete run is uniformly ungated on a shared host. No continuation or incomplete group contributes to these tables. Contention prevents these timings from establishing a controlled revision speedup or the 10x speed and memory target.

RuDB was rebuilt from clean main `0e5f97bdf437ce41462b79441596c3d848578405`, version 0.8.5, in a new empty Cargo target directory on Apple M4. Its measured executable SHA-256 is `f308f515906c14f954158cabff5a6d28f71c6c2b140cad517b7ab75c5b8544bc`. DuckDB is released v1.5.5 `d8cdaa33fd`. Both executables stayed fixed. The rebased pending schema-index, topology, and bound-reader patches are excluded. The Linux host became read-only during the preceding run. The failure record is retained separately; its journal could not be recovered after SSH became unavailable. It contributes no readings to these tables.

## Protocol

Every query execution starts a fresh process. One first execution is retained separately; the following seven fresh-process repetitions supply per-query medians. The OS page cache is not flushed. Neighboring rounds reverse order and rotate after each pair. Eight rounds balance execution positions; excluding the first leaves a small imbalance in the seven-reading medians. Results are fully rendered.

Both engines receive six threads and a 4GB memory limit. RuDB uses its default first engine, stored answers disabled, and Parquet mirroring disabled. This does not measure the compiled engine or its recently merged execution changes. Ordinary reusable column statistics remain available; no query answer or optional query-specific index is added.

Native files are loaded afresh with the same explicit schema, INSERT INTO hits SELECT ... FROM read_parquet(...), SELECT * projection, and CHECKPOINT SQL. The Parquet cases read identical source files. These local samples already contain DATE and TIMESTAMP columns. Actual row counts and hashes are recorded below. The Linux 10m input became unreadable, so this run uses the intact local typed sample ladder. Its layout, types, row counts, and hashes differ from the raw Linux ladder. Do not compare these runs as revision speedups.

A monotonic clock measures the whole child, including startup, file opening, parsing, binding, execution, rendering, and exit. User plus system CPU and peak RSS come from wait4 for that exact child. RSS is process peak resident memory, not allocated bytes or a delta. macOS reports RSS in bytes; the helper retains those units. Rounded CLI query timers are retained but do not drive headline ratios. Totals sum per-query medians; maximum RSS is the largest process peak, not a sum.

This Mac has an Apple M4 with 10 logical CPUs and 24 GiB RAM. Pre-child load ranged from 6.20 to 29.06; post-child load ranged from 6.20 to 29.06. The prior gated Mac attempt stalled as other workloads raised load above 10. It was stopped between timed children after 3,502 readings and is archived separately. The selected rerun uses one ungated protocol from its first load through its last query. Shared CPU scheduling and clocks remain uncontrolled, so comparisons with earlier runs are not controlled revision speedups. Every waiting time, load reading, fault, context switch, stdout, stderr, and resource reading remains available.

## Source samples

| Sample | Actual rows | Parquet bytes | Footer bytes | SHA-256 |
| --- | ---: | ---: | ---: | --- |
| 1k | 1,000 | 308,393 | 13,281 | 3c6dcf052a11612f7b07573880c8d3ba08be0c349a37e523b0bd84fd46ab94e9 |
| 10k | 10,000 | 2,636,690 | 13,588 | 9d17db45885417de116d1f2bad315a85347e95de55e3fe2c0ffb03d18f8f71d9 |
| 1m | 999,975 | 229,947,121 | 107,164 | 5a6ecfc3b5e7931ddec01bb428e67acab0f30fe95b418569a7cc989e03c36168 |
| 10m | 10,000,000 | 1,480,459,152 | 1,186,538 | c721aa678f950f674b6c82a55b730a0c664804c80300e464f014f009768c1638 |

## Query comparison

| Sample | Format | DuckDB wall total (s) | RuDB wall total (s) | DuckDB / RuDB wall | DuckDB CPU total (s) | RuDB CPU total (s) | DuckDB max RSS (MiB) | RuDB max RSS (MiB) | DuckDB / RuDB max RSS |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | native | 0.765 | 0.249 | 3.08x | 0.772 | 0.210 | 33.78 | 11.95 | 2.83x |
| 1k | parquet | 0.767 | 0.251 | 3.06x | 0.777 | 0.213 | 32.45 | 11.55 | 2.81x |
| 10k | native | 0.927 | 0.310 | 2.99x | 0.956 | 0.270 | 38.28 | 14.67 | 2.61x |
| 10k | parquet | 0.947 | 0.323 | 2.93x | 0.973 | 0.279 | 37.09 | 14.75 | 2.51x |
| 1m | native | 1.837 | 0.590 | 3.11x | 4.603 | 1.122 | 271.95 | 82.62 | 3.29x |
| 1m | parquet | 2.192 | 1.548 | 1.42x | 5.709 | 4.815 | 405.28 | 339.66 | 1.19x |
| 10m | native | 7.326 | 1.602 | 4.57x | 30.012 | 5.054 | 1164.67 | 279.86 | 4.16x |
| 10m | parquet | 9.395 | 7.451 | 1.26x | 39.080 | 35.642 | 995.81 | 804.72 | 1.24x |

Ratios above one favor RuDB. Both sides cover the same 43 complete queries. A ratio of totals differs from a geometric mean of per-query ratios. Answer differences are qualified below.

## Observed query-level ratios under contention

| Sample | Format | Wall-ratio geometric mean | Queries at least 10x faster / 43 | Queries with at least 10x lower peak RSS / 43 | Queries meeting both / 43 |
| --- | --- | ---: | ---: | ---: | ---: |
| 1k | native | 3.09x | 0 | 0 | 0 |
| 1k | parquet | 3.06x | 0 | 0 | 0 |
| 10k | native | 3.00x | 0 | 0 | 0 |
| 10k | parquet | 2.95x | 0 | 0 | 0 |
| 1m | native | 3.19x | 0 | 4 | 0 |
| 1m | parquet | 1.73x | 0 | 0 | 0 |
| 10m | native | 4.07x | 7 | 7 | 6 |
| 10m | parquet | 1.49x | 0 | 0 | 0 |

## Q1 through Q4 at 10m

| Query | Format | DuckDB wall (ms) | RuDB wall (ms) | DuckDB / RuDB wall | DuckDB median RSS (MiB) | RuDB median RSS (MiB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Q1 | native | 27.733 | 10.151 | 2.73x | 19.20 | 10.94 |
| Q1 | parquet | 60.241 | 21.246 | 2.84x | 29.88 | 19.16 |
| Q2 | native | 30.604 | 10.967 | 2.79x | 27.64 | 11.22 |
| Q2 | parquet | 68.555 | 29.366 | 2.33x | 33.03 | 27.27 |
| Q3 | native | 39.313 | 10.165 | 3.87x | 35.72 | 11.12 |
| Q3 | parquet | 73.761 | 29.825 | 2.47x | 36.97 | 31.48 |
| Q4 | native | 40.923 | 10.316 | 3.97x | 41.75 | 11.27 |
| Q4 | parquet | 75.312 | 32.731 | 2.30x | 52.62 | 43.16 |

## Native load cost

Each load has one observation and includes building ordinary metadata and checkpointing. The 4GB engine setting does not cap whole-process RSS. In this revision, RuDB numeric summary closing and dictionary encoding choose hardware-based worker counts from available hardware, capped at 32, independently of the six-thread setting. This host exposes 10 logical CPUs. CPU divided by wall time exposes that concurrency; shorter load wall time can use more CPU.

| Sample | Engine | Load wall (s) | Load CPU (s) | CPU / wall | Peak RSS (MiB) | Native file (MiB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | 0.042 | 0.043 | 1.01 | 53.59 | 1.01 |
| 1k | rudb-native | 0.106 | 0.089 | 0.84 | 34.11 | 1.79 |
| 10k | duckdb-native | 0.109 | 0.170 | 1.56 | 68.98 | 4.51 |
| 10k | rudb-native | 0.390 | 0.395 | 1.01 | 68.55 | 7.37 |
| 1m | duckdb-native | 5.458 | 11.787 | 2.16 | 1262.22 | 433.76 |
| 1m | rudb-native | 3.101 | 11.249 | 3.63 | 1224.31 | 235.46 |
| 10m | duckdb-native | 58.691 | 117.736 | 2.01 | 1525.41 | 2234.01 |
| 10m | rudb-native | 25.331 | 89.534 | 3.53 | 2314.44 | 1400.79 |

## Answer checks

Every original timed output was compared across four cases and eight executions. Integers are exact, including values near 2^63; floating point uses a 1e-9 tolerance. Original order and selection differences remain visible. Separate untimed retests add output-column tie breakers without replacing timed SQL or its answers.

| Sample | Original match | Same rows, different order | Different selection, deterministic retest matches | Failed or unresolved |
| --- | ---: | ---: | ---: | ---: |
| 1k | 24 | 6 | 13 | 0 |
| 10k | 25 | 3 | 15 | 0 |
| 1m | 27 | 8 | 8 | 0 |
| 10m | 32 | 4 | 7 | 0 |

Q29 at 10m: **original match**. Matching deterministic retests do not establish that every originally selected row matches. This report does not claim all original outputs are identical.

## Remaining gaps at 10m

| Format | Query | DuckDB wall (ms) | RuDB wall (ms) | DuckDB / RuDB wall | DuckDB max RSS (MiB) | RuDB max RSS (MiB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| native | Q41 | 28.864 | 15.915 | 1.81x | 46.81 | 40.69 |
| native | Q40 | 89.683 | 46.731 | 1.92x | 240.69 | 91.30 |
| native | Q38 | 29.561 | 14.921 | 1.98x | 48.19 | 35.70 |
| native | Q8 | 30.771 | 14.676 | 2.10x | 29.75 | 27.41 |
| native | Q42 | 30.679 | 14.600 | 2.10x | 40.83 | 31.48 |
| parquet | Q23 | 493.232 | 830.038 | 0.59x | 645.97 | 477.84 |
| parquet | Q22 | 243.615 | 344.367 | 0.71x | 408.38 | 356.80 |
| parquet | Q34 | 502.197 | 642.349 | 0.78x | 936.48 | 804.72 |
| parquet | Q24 | 333.858 | 417.848 | 0.80x | 450.48 | 365.95 |
| parquet | Q35 | 521.834 | 647.437 | 0.81x | 995.81 | 793.00 |

## Reproduce and inspect

The [per-query tables](main-0e5f97bd-macos-clickbench/per-query.md), [raw repetitions](main-0e5f97bd-macos-clickbench/raw.jsonl.gz), [metadata](main-0e5f97bd-macos-clickbench/metadata.json), [answer checks](main-0e5f97bd-macos-clickbench/correctness.json), [native file hashes](main-0e5f97bd-macos-clickbench/native-files.json), and [provenance](main-0e5f97bd-macos-clickbench/provenance.json) retain the evidence. The [output archive](main-0e5f97bd-macos-clickbench/outputs.tar.gz) contains every CSV, stderr and resource record, original SQL, deterministic diagnostic SQL, measurement script snapshots, and clean-main build record. Binaries and data files are excluded; their paths and hashes are recorded.

The [build record](main-0e5f97bd-macos-clickbench/build-input-audit.json) retains the exact command, source tar hash, executable hash, compiler version, and target artifact fingerprints. Use an empty target directory for each new cutoff. The build uses Rust 1.98.0 and the default aarch64 target. All 15 measurement, capacity, order, and continuation tests passed on macOS, with one platform-specific skip.

```sh
python3 scripts/clickbench-audit.py \
  --duckdb "$DUCKDB" --rudb "$RUDB" --data "$DATA" --output "$OUTPUT" \
  --sizes 1k 10k 1m 10m --hot 7 --threads 6 --memory-limit 4GB \
  --timeout 180
python3 scripts/verify-clickbench-audit.py "$OUTPUT"
```

Export the canonical SQL into the new output directory first. For this typed input ladder, projection.sql is SELECT *. The raw Linux ladder uses the recorded date and timestamp normalization projection instead. A failed capacity wait must retain its incomplete journal; never average successful fragments into a completed group.

The [focused schema-index comparison](parquet-schema-index-c3830c12.md) uses an earlier c3830c12 Linux binary and different inputs with 60 rounds per sample and a separate unmerged candidate. That earlier focused test reuses previous native files and does not replace this fresh-load whole-suite comparison. The [gated Mac attempt](main-0e5f97bd-macos-clickbench/gated-attempt.tar.gz) retains all 3,502 readings, outputs, SQL, and the reason for stopping. The [earlier c3830c12 attempt](main-0e5f97bd-macos-clickbench/earlier-c3830c12-attempt.tar.gz) retains 4,136 readings at the preceding source cutoff. The [interrupted Linux attempt](main-0e5f97bd-macos-clickbench/linux-0e5f97bd-interrupted.tar.gz) records the filesystem failure and the last observed progress. The remote journal was not recoverable after SSH became unavailable. None of these attempts contributes to this Mac comparison. No engine optimization was merged as part of this rerun.
