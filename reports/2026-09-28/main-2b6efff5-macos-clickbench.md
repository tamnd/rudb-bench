# ClickBench rerun at main 2b6efff5 on Apple M4

All 43 canonical queries completed at 1k, 10k, 1m, and 10m across DuckDB native, RuDB native, DuckDB Parquet, and RuDB Parquet. This run retains 5,504 fresh-process query executions and eight fresh native loads. The complete run is uniformly ungated on a shared host. No continuation or incomplete group contributes to these tables. Contention prevents these timings from establishing a controlled revision speedup or the 10x speed and memory target.

RuDB was rebuilt from clean main `2b6efff5e9260f7018c6f5c57d46296f41e15283`, version 0.8.5, in a new empty Cargo target directory on Apple M4. Its measured executable SHA-256 is `f9b283bac1b07acf6cec5857f1be70e3de9f9e16bd0304799a997cf13ff26b37`. DuckDB is released v1.5.5 `d8cdaa33fd`. Both executables stayed fixed. The rebased pending schema-index, topology, and bound-reader patches are excluded. Main advanced to this cutoff during the initial build. That unmeasured build was stopped. A new empty target was built for this revision before any selected measurement began. The pending branches were rebased onto this cutoff. The exact-pattern compiled regex shortcut was removed in PR 2098 before this build. This report measures the default first engine; it does not measure compiled replacement performance.

## Protocol

Every query execution starts a fresh process. One first execution is retained separately; the following seven fresh-process repetitions supply per-query medians. The OS page cache is not flushed. Neighboring rounds reverse order and rotate after each pair. Eight rounds balance execution positions; excluding the first leaves a small imbalance in the seven-reading medians. Results are fully rendered.

Both engines receive six threads and a 4GB memory limit. RuDB uses its default first engine, stored answers disabled, and Parquet mirroring disabled. This does not measure the compiled engine or its recently merged execution changes. Ordinary reusable column statistics remain available; no query answer or optional query-specific index is added.

Native files are loaded afresh with the same explicit schema, INSERT INTO hits SELECT ... FROM read_parquet(...), SELECT * projection, and CHECKPOINT SQL. The Parquet cases read identical source files. These local samples already contain DATE and TIMESTAMP columns. They are the identical typed ladder used for the completed 0e5f97bd Mac run; source hashes and canonical SQL hashes were checked. This ladder differs from the raw Linux inputs. Actual row counts, source sizes, footer sizes, and hashes are recorded below. Shared-host runs do not establish a controlled revision speedup.

A monotonic clock measures the whole child, including startup, file opening, parsing, binding, execution, rendering, and exit. User plus system CPU and peak RSS come from wait4 for that exact child. RSS is process peak resident memory, not allocated bytes or a delta. macOS reports RSS in bytes; the helper retains those units. Rounded CLI query timers are retained but do not drive headline ratios. Totals sum per-query medians; maximum RSS is the largest process peak, not a sum.

This Mac has an Apple M4 with 10 logical CPUs and 24 GiB RAM. Pre-child load ranged from 13.29 to 32.23; post-child load ranged from 13.29 to 32.23. This entire rerun uses one ungated protocol from its first load through its last query. Shared CPU scheduling and clocks remain uncontrolled, so comparisons with earlier runs are not controlled revision speedups. Every waiting time, load reading, fault, context switch, stdout, stderr, and resource reading remains available.

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
| 1k | native | 0.918 | 0.300 | 3.06x | 0.907 | 0.254 | 35.88 | 11.89 | 3.02x |
| 1k | parquet | 0.921 | 0.302 | 3.04x | 0.918 | 0.255 | 34.05 | 11.61 | 2.93x |
| 10k | native | 0.866 | 0.305 | 2.84x | 0.872 | 0.264 | 38.27 | 14.25 | 2.69x |
| 10k | parquet | 0.879 | 0.314 | 2.80x | 0.895 | 0.268 | 39.14 | 14.94 | 2.62x |
| 1m | native | 1.885 | 0.637 | 2.96x | 4.670 | 1.157 | 232.33 | 83.33 | 2.79x |
| 1m | parquet | 2.237 | 1.530 | 1.46x | 5.772 | 4.814 | 399.03 | 334.78 | 1.19x |
| 10m | native | 9.300 | 2.026 | 4.59x | 32.098 | 5.487 | 1144.39 | 286.94 | 3.99x |
| 10m | parquet | 12.403 | 9.297 | 1.33x | 41.923 | 37.262 | 1013.48 | 816.97 | 1.24x |

Ratios above one favor RuDB. Both sides cover the same 43 complete queries. A ratio of totals differs from a geometric mean of per-query ratios. Answer differences are qualified below.

## Observed query-level ratios under contention

| Sample | Format | Wall-ratio geometric mean | Queries at least 10x faster / 43 | Queries with at least 10x lower peak RSS / 43 | Queries meeting both / 43 |
| --- | --- | ---: | ---: | ---: | ---: |
| 1k | native | 3.06x | 0 | 0 | 0 |
| 1k | parquet | 3.04x | 0 | 0 | 0 |
| 10k | native | 2.84x | 0 | 0 | 0 |
| 10k | parquet | 2.82x | 0 | 0 | 0 |
| 1m | native | 3.08x | 0 | 3 | 0 |
| 1m | parquet | 1.74x | 0 | 0 | 0 |
| 10m | native | 4.01x | 6 | 7 | 6 |
| 10m | parquet | 1.49x | 0 | 0 | 0 |

## Q1 through Q4 at 10m

| Query | Format | DuckDB wall (ms) | RuDB wall (ms) | DuckDB / RuDB wall | DuckDB median RSS (MiB) | RuDB median RSS (MiB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Q1 | native | 22.830 | 8.951 | 2.55x | 19.03 | 11.14 |
| Q1 | parquet | 49.099 | 17.299 | 2.84x | 30.34 | 19.14 |
| Q2 | native | 26.048 | 9.224 | 2.82x | 27.02 | 11.12 |
| Q2 | parquet | 57.343 | 24.826 | 2.31x | 34.03 | 27.48 |
| Q3 | native | 35.993 | 9.658 | 3.73x | 34.83 | 11.12 |
| Q3 | parquet | 66.233 | 27.952 | 2.37x | 35.03 | 32.62 |
| Q4 | native | 36.358 | 9.568 | 3.80x | 41.45 | 11.02 |
| Q4 | parquet | 67.797 | 29.696 | 2.28x | 47.64 | 41.95 |

## Native load cost

Each load has one observation and includes building ordinary metadata and checkpointing. The 4GB engine setting does not cap whole-process RSS. In this revision, RuDB numeric summary closing and dictionary encoding choose hardware-based worker counts from available hardware, capped at 32, independently of the six-thread setting. This host exposes 10 logical CPUs. CPU divided by wall time exposes that concurrency; shorter load wall time can use more CPU.

| Sample | Engine | Load wall (s) | Load CPU (s) | CPU / wall | Peak RSS (MiB) | Native file (MiB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | 0.047 | 0.046 | 0.97 | 47.27 | 1.26 |
| 1k | rudb-native | 0.117 | 0.091 | 0.78 | 34.62 | 1.79 |
| 10k | duckdb-native | 0.127 | 0.130 | 1.02 | 65.48 | 4.26 |
| 10k | rudb-native | 0.409 | 0.458 | 1.12 | 70.47 | 7.37 |
| 1m | duckdb-native | 5.651 | 13.309 | 2.36 | 1106.48 | 431.26 |
| 1m | rudb-native | 3.061 | 11.813 | 3.86 | 1438.95 | 235.37 |
| 10m | duckdb-native | 59.336 | 126.042 | 2.12 | 2527.08 | 2242.51 |
| 10m | rudb-native | 23.155 | 92.590 | 4.00 | 2147.77 | 1400.06 |

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
| native | Q41 | 38.261 | 22.275 | 1.72x | 46.34 | 41.02 |
| native | Q42 | 39.016 | 22.038 | 1.77x | 41.44 | 31.36 |
| native | Q40 | 123.326 | 64.617 | 1.91x | 240.70 | 92.22 |
| native | Q38 | 48.455 | 25.162 | 1.93x | 48.48 | 36.36 |
| native | Q39 | 48.545 | 24.997 | 1.94x | 68.67 | 28.95 |
| parquet | Q23 | 523.368 | 860.971 | 0.61x | 652.59 | 494.39 |
| parquet | Q22 | 321.381 | 496.417 | 0.65x | 405.83 | 348.78 |
| parquet | Q34 | 697.050 | 1021.480 | 0.68x | 952.33 | 813.58 |
| parquet | Q17 | 218.833 | 279.949 | 0.78x | 331.34 | 400.50 |
| parquet | Q24 | 358.918 | 458.608 | 0.78x | 458.59 | 366.33 |

## Reproduce and inspect

The [per-query tables](main-2b6efff5-macos-clickbench/per-query.md), [raw repetitions](main-2b6efff5-macos-clickbench/raw.jsonl.gz), [metadata](main-2b6efff5-macos-clickbench/metadata.json), [answer checks](main-2b6efff5-macos-clickbench/correctness.json), [native file hashes](main-2b6efff5-macos-clickbench/native-files.json), and [provenance](main-2b6efff5-macos-clickbench/provenance.json) retain the evidence. The [output archive](main-2b6efff5-macos-clickbench/outputs.tar.gz) contains every CSV, stderr and resource record, original SQL, deterministic diagnostic SQL, measurement script snapshots, and clean-main build record. Binaries and data files are excluded; their paths and hashes are recorded.

The [build record](main-2b6efff5-macos-clickbench/build-input-audit.json) retains the exact command, source tar hash, executable hash, compiler version, and target artifact fingerprints. Use an empty target directory for each new cutoff. The build uses Rust 1.98.0 and the default aarch64 target. The unchanged measurement scripts previously passed all 15 measurement, capacity, order, and continuation tests on macOS, with one platform-specific skip.

```sh
python3 scripts/clickbench-audit.py \
  --duckdb "$DUCKDB" --rudb "$RUDB" --data "$DATA" --output "$OUTPUT" \
  --sizes 1k 10k 1m 10m --hot 7 --threads 6 --memory-limit 4GB \
  --timeout 180
python3 scripts/verify-clickbench-audit.py "$OUTPUT"
```

Export the canonical SQL into the new output directory first. For these typed samples, projection.sql is SELECT *. The raw Linux ladder requires its separate normalization projection. A failed capacity wait must retain its incomplete journal; never average successful fragments into a completed group.

The [preceding Mac comparison](main-47e25af3-macos-clickbench.md) retains the earlier completed cutoff, the interrupted and gated attempts, and the Linux filesystem failure record. No readings from those attempts contribute to this report. The [focused parser comparison](parquet-schema-index-c3830c12.md) uses an earlier Linux cutoff and different inputs. The generic schema-index, topology, and bound-reader patches remain excluded from the selected engine.
