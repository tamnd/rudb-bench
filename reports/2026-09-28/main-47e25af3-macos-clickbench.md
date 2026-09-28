# ClickBench rerun at main 47e25af3 on Apple M4

All 43 canonical queries completed at 1k, 10k, 1m, and 10m across DuckDB native, RuDB native, DuckDB Parquet, and RuDB Parquet. This run retains 5,504 fresh-process query executions and eight fresh native loads. The complete run is uniformly ungated on a shared host. No continuation or incomplete group contributes to these tables. Contention prevents these timings from establishing a controlled revision speedup or the 10x speed and memory target.

RuDB was rebuilt from clean main `47e25af36f0498b8f598361c9f7e9928856e0c7a`, version 0.8.5, in a new empty Cargo target directory on Apple M4. Its measured executable SHA-256 is `22e4b2a1978b9afda97f958120a1d8f56f80e4163058484df67203a2dafc86bf`. DuckDB is released v1.5.5 `d8cdaa33fd`. Both executables stayed fixed. The rebased pending schema-index, topology, and bound-reader patches are excluded. Main advanced during the preceding run. The pending branches were rebased onto this cutoff. The exact-pattern compiled regex shortcut was removed in PR 2098 before this build. This report measures the default first engine; it does not measure compiled replacement performance.

## Protocol

Every query execution starts a fresh process. One first execution is retained separately; the following seven fresh-process repetitions supply per-query medians. The OS page cache is not flushed. Neighboring rounds reverse order and rotate after each pair. Eight rounds balance execution positions; excluding the first leaves a small imbalance in the seven-reading medians. Results are fully rendered.

Both engines receive six threads and a 4GB memory limit. RuDB uses its default first engine, stored answers disabled, and Parquet mirroring disabled. This does not measure the compiled engine or its recently merged execution changes. Ordinary reusable column statistics remain available; no query answer or optional query-specific index is added.

Native files are loaded afresh with the same explicit schema, INSERT INTO hits SELECT ... FROM read_parquet(...), SELECT * projection, and CHECKPOINT SQL. The Parquet cases read identical source files. These local samples already contain DATE and TIMESTAMP columns. They are the identical typed ladder used for the completed 0e5f97bd Mac run; source hashes and canonical SQL hashes were checked. This ladder differs from the raw Linux inputs. Actual row counts, source sizes, footer sizes, and hashes are recorded below. Shared-host runs do not establish a controlled revision speedup.

A monotonic clock measures the whole child, including startup, file opening, parsing, binding, execution, rendering, and exit. User plus system CPU and peak RSS come from wait4 for that exact child. RSS is process peak resident memory, not allocated bytes or a delta. macOS reports RSS in bytes; the helper retains those units. Rounded CLI query timers are retained but do not drive headline ratios. Totals sum per-query medians; maximum RSS is the largest process peak, not a sum.

This Mac has an Apple M4 with 10 logical CPUs and 24 GiB RAM. Pre-child load ranged from 7.64 to 40.37; post-child load ranged from 7.64 to 40.37. This entire rerun uses one ungated protocol from its first load through its last query. Shared CPU scheduling and clocks remain uncontrolled, so comparisons with earlier runs are not controlled revision speedups. Every waiting time, load reading, fault, context switch, stdout, stderr, and resource reading remains available.

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
| 1k | native | 1.167 | 0.365 | 3.20x | 1.159 | 0.310 | 34.42 | 13.45 | 2.56x |
| 1k | parquet | 1.183 | 0.365 | 3.24x | 1.180 | 0.312 | 32.92 | 12.47 | 2.64x |
| 10k | native | 1.127 | 0.378 | 2.99x | 1.147 | 0.330 | 37.47 | 15.72 | 2.38x |
| 10k | parquet | 1.156 | 0.389 | 2.97x | 1.186 | 0.334 | 38.53 | 14.86 | 2.59x |
| 1m | native | 4.046 | 1.246 | 3.25x | 7.678 | 1.739 | 262.84 | 75.44 | 3.48x |
| 1m | parquet | 4.828 | 3.639 | 1.33x | 9.080 | 7.487 | 389.38 | 334.88 | 1.16x |
| 10m | native | 7.268 | 1.600 | 4.54x | 30.168 | 5.110 | 1146.69 | 282.98 | 4.05x |
| 10m | parquet | 9.451 | 7.560 | 1.25x | 39.686 | 36.093 | 994.19 | 819.11 | 1.21x |

Ratios above one favor RuDB. Both sides cover the same 43 complete queries. A ratio of totals differs from a geometric mean of per-query ratios. Answer differences are qualified below.

## Observed query-level ratios under contention

| Sample | Format | Wall-ratio geometric mean | Queries at least 10x faster / 43 | Queries with at least 10x lower peak RSS / 43 | Queries meeting both / 43 |
| --- | --- | ---: | ---: | ---: | ---: |
| 1k | native | 3.20x | 0 | 0 | 0 |
| 1k | parquet | 3.25x | 0 | 0 | 0 |
| 10k | native | 2.99x | 0 | 0 | 0 |
| 10k | parquet | 2.98x | 0 | 0 | 0 |
| 1m | native | 3.09x | 1 | 3 | 1 |
| 1m | parquet | 1.66x | 0 | 0 | 0 |
| 10m | native | 3.97x | 7 | 7 | 6 |
| 10m | parquet | 1.47x | 0 | 0 | 0 |

## Q1 through Q4 at 10m

| Query | Format | DuckDB wall (ms) | RuDB wall (ms) | DuckDB / RuDB wall | DuckDB median RSS (MiB) | RuDB median RSS (MiB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Q1 | native | 18.422 | 6.888 | 2.67x | 18.70 | 10.88 |
| Q1 | parquet | 40.402 | 14.694 | 2.75x | 29.30 | 19.89 |
| Q2 | native | 22.874 | 7.767 | 2.95x | 27.20 | 11.12 |
| Q2 | parquet | 50.276 | 22.023 | 2.28x | 33.03 | 27.00 |
| Q3 | native | 30.632 | 8.216 | 3.73x | 35.58 | 11.09 |
| Q3 | parquet | 56.505 | 23.153 | 2.44x | 36.31 | 32.25 |
| Q4 | native | 30.333 | 8.514 | 3.56x | 41.17 | 11.03 |
| Q4 | parquet | 54.421 | 24.952 | 2.18x | 50.78 | 42.02 |

## Native load cost

Each load has one observation and includes building ordinary metadata and checkpointing. The 4GB engine setting does not cap whole-process RSS. In this revision, RuDB numeric summary closing and dictionary encoding choose hardware-based worker counts from available hardware, capped at 32, independently of the six-thread setting. This host exposes 10 logical CPUs. CPU divided by wall time exposes that concurrency; shorter load wall time can use more CPU.

| Sample | Engine | Load wall (s) | Load CPU (s) | CPU / wall | Peak RSS (MiB) | Native file (MiB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1k | duckdb-native | 0.068 | 0.068 | 1.01 | 62.25 | 1.01 |
| 1k | rudb-native | 0.144 | 0.124 | 0.86 | 38.22 | 1.79 |
| 10k | duckdb-native | 0.195 | 0.283 | 1.45 | 69.39 | 4.51 |
| 10k | rudb-native | 0.574 | 0.615 | 1.07 | 73.36 | 7.37 |
| 1m | duckdb-native | 5.968 | 13.521 | 2.27 | 980.28 | 433.51 |
| 1m | rudb-native | 3.758 | 12.808 | 3.41 | 1090.94 | 235.34 |
| 10m | duckdb-native | 56.259 | 124.414 | 2.21 | 2876.78 | 2250.51 |
| 10m | rudb-native | 18.825 | 88.223 | 4.69 | 2459.38 | 1401.02 |

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
| native | Q23 | 324.422 | 199.037 | 1.63x | 859.56 | 189.16 |
| native | Q41 | 28.388 | 15.138 | 1.88x | 46.64 | 40.59 |
| native | Q42 | 28.034 | 14.804 | 1.89x | 40.55 | 31.20 |
| native | Q40 | 85.845 | 44.929 | 1.91x | 237.88 | 91.16 |
| native | Q38 | 29.404 | 15.308 | 1.92x | 47.83 | 35.88 |
| parquet | Q23 | 460.863 | 775.072 | 0.59x | 635.25 | 468.94 |
| parquet | Q22 | 240.679 | 357.686 | 0.67x | 390.38 | 354.89 |
| parquet | Q34 | 521.579 | 694.179 | 0.75x | 961.36 | 808.06 |
| parquet | Q35 | 549.310 | 691.686 | 0.79x | 994.19 | 819.11 |
| parquet | Q24 | 360.877 | 442.136 | 0.82x | 465.47 | 370.12 |

## Reproduce and inspect

The [per-query tables](main-47e25af3-macos-clickbench/per-query.md), [raw repetitions](main-47e25af3-macos-clickbench/raw.jsonl.gz), [metadata](main-47e25af3-macos-clickbench/metadata.json), [answer checks](main-47e25af3-macos-clickbench/correctness.json), [native file hashes](main-47e25af3-macos-clickbench/native-files.json), and [provenance](main-47e25af3-macos-clickbench/provenance.json) retain the evidence. The [output archive](main-47e25af3-macos-clickbench/outputs.tar.gz) contains every CSV, stderr and resource record, original SQL, deterministic diagnostic SQL, measurement script snapshots, and clean-main build record. Binaries and data files are excluded; their paths and hashes are recorded.

The [build record](main-47e25af3-macos-clickbench/build-input-audit.json) retains the exact command, source tar hash, executable hash, compiler version, and target artifact fingerprints. Use an empty target directory for each new cutoff. The build uses Rust 1.98.0 and the default aarch64 target. All 15 measurement, capacity, order, and continuation tests passed on macOS, with one platform-specific skip.

```sh
python3 scripts/clickbench-audit.py \
  --duckdb "$DUCKDB" --rudb "$RUDB" --data "$DATA" --output "$OUTPUT" \
  --sizes 1k 10k 1m 10m --hot 7 --threads 6 --memory-limit 4GB \
  --timeout 180
python3 scripts/verify-clickbench-audit.py "$OUTPUT"
```

Export the canonical SQL into the new output directory first. For these typed samples, projection.sql is SELECT *. The raw Linux ladder requires its separate normalization projection. A failed capacity wait must retain its incomplete journal; never average successful fragments into a completed group.

The [preceding Mac comparison](main-0e5f97bd-macos-clickbench.md) retains the earlier completed cutoff, the interrupted and gated attempts, and the Linux filesystem failure record. No readings from those attempts contribute to this report. The [focused parser comparison](parquet-schema-index-c3830c12.md) uses an earlier Linux cutoff and different inputs. The generic schema-index, topology, and bound-reader patches remain excluded from the selected engine.
