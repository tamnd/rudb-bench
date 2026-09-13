# ClickBench measurement review, 13 September 2026

DuckDB v1.5.5 (`d8cdaa33fd`) and rudb 0.2.37 ran all 43 queries at each of four sample sizes on `gamingpc-wsl`, an Intel Core i9-13900K with 32 hardware threads and 31 GiB of guest RAM. Every query has one first execution and five hot repetitions, each in a fresh process. All 2,064 query executions and four DuckDB loads succeeded. The maximum observed one-minute load average during the query sweep was 1.716. Build and test jobs ran outside the validated sweep.

| Sample | Actual rows | DuckDB query seconds | rudb query seconds | DuckDB peak RSS MiB | rudb peak RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1k | 1,000 | 0.086000 | 0.029232 | 37.50 | 6.39 |
| 10k | 10,000 | 0.114000 | 0.132842 | 37.73 | 20.31 |
| 100k | 99,998 | 0.303000 | 1.128922 | 65.21 | 159.15 |
| 1m | 999,975 | 0.540000 | 10.144629 | 303.50 | 505.56 |

Query seconds are the sum of the five-run median for each of the same 43 queries. RSS is the maximum across first and hot executions of all queries. At 1m, rudb takes 18.79 times DuckDB's query time and 1.67 times its peak RSS. The samples are strided extracts of the existing ClickBench file, identified by SHA-256 and actual row count in [metadata.json](metadata.json). They are not official ClickBench scores. The full dataset has not been run in this review, following the request to start with smaller sizes.

## Measurement boundaries

The CLI query timers measure statement execution and result rendering. The native helper measures monotonic elapsed time from child launch through reaping, plus user and system CPU across the child's threads, peak resident memory, Linux block I/O accounting, page faults, and context switches. Python orchestration time is recorded separately. No per-operator metrics file is enabled during the audit. Outputs are fully produced and retained. A timeout terminates the launched process group and records unavailable engine resources rather than attributing the wrapper's resources to the engine.

First execution is explicitly unflushed, not disk-cold. Later repetitions have a warm OS page cache and a new engine process; they do not reuse a database buffer pool. Engine order alternates by query. DuckDB loads its declared native table once per size, whereas rudb uses a view over Parquet and pays decoding costs in each query. This storage-format difference is part of the measured comparison. The audit does not claim equal storage formats or equal CPU utilization. DuckDB's settings are recorded in the metadata; both engines use their default execution settings.

DuckDB's CLI timer has millisecond resolution in this release. It rounded 65 individual query executions to zero across this sweep. The small-sample query totals therefore carry quantization error; a zero is not instantaneous execution. Read the higher-resolution process wall and CPU columns beside them. Five repetitions expose variation but do not guarantee a noise-free result. Per-query median and nearest-rank IQR are in [report.md](report.md), and every sample is retained in the raw archive. These medians are not the official best-of-three convention.

## Findings and fixes

- rudb's shell rounded query time to three decimal places. It now prints nine, preserving submillisecond measurements. A shell regression test covers the output precision.
- The legacy harness could panic on negative, infinite, or NaN engine timing text. Parsing now rejects those values. It also removes stale resource reports before running another command and forces the timer's locale to C.
- The resource parser assumed BSD input-operation counts could be multiplied by 512 to obtain bytes. Those counts now produce an unavailable byte measurement. Linux byte conversions use checked arithmetic. The operating-system API describes these as input operations; a portable byte conversion cannot be assumed. See [getrusage](https://www.man7.org/linux/man-pages/man2/getrusage.2.html).
- GNU time prints CPU components to hundredths of a second, making tiny rudb queries appear to use no CPU. The audit uses microsecond CPU counters from wait4 instead. A native launcher prevents the Python parent's pre-exec resident memory from inflating small-engine peaks. A rejected preliminary run exposed that inflation; its numbers are not used here.
- The legacy suite skips q19 and q33 on rudb because of full-dataset memory limits and aborts an engine's suite on its first failure. The audit attempts all 43 queries at every size, including those two, with explicit failure and timeout records. It requires all repetitions to succeed before aggregating a query.

The [resource cross-check](crosscheck.json) measured the same workload through GNU time and the native reader: both reported 76,959,744 bytes of peak RSS. The precise CPU total was 0.068833 seconds; GNU time printed 0.06 seconds. The difference is within its component rounding plus wrapper overhead. The test suite also launches a small child after a 64 MiB child and checks that its peak does not inherit either that prior peak or Python's resident memory.

## Answer checks

Of 172 query-and-size pairs, 112 matched directly, 18 returned the same rows in a different order, and 42 matched after a separate deterministic retest added output columns as tie breakers. No difference remained unresolved. Original output and its classification remain available in [correctness.json](correctness.json). The deterministic SQL is an untimed diagnostic and never replaces the benchmark SQL. This is a check of these samples, not proof of correctness on the full dataset.

## Reproduction

The audit requires Linux, Python 3, a C compiler, the two engine binaries, and existing sample files named `hits-1k.parquet`, `hits-10k.parquet`, `hits-100k.parquet`, and `hits-1m.parquet`. Run from the rudb-bench root. Use a new output directory for each run; the raw log refuses to overwrite an existing run.

```sh
cargo build --release --example export_clickbench
cc -O2 -Wall -Wextra -Werror scripts/measure-child.c -o scripts/measure-child
./target/release/examples/export_clickbench reports/my-audit/sql
python3 scripts/clickbench-audit.py \
  --duckdb /path/to/duckdb --rudb /path/to/rudb \
  --data /path/to/samples --output reports/my-audit
python3 scripts/verify-clickbench-audit.py reports/my-audit
python3 -m unittest discover -s scripts -p 'test_clickbench_audit.py'
```

The verification command writes correctness.json and updates the report with those classifications. Exact executable hashes, command lines, environment settings, source commit IDs and working-tree patches are retained. rudb was built from the local working tree, including the four pre-existing modified files; these edits were preserved.

## Artifacts and validation

[report.md](report.md) contains all per-query time, IQR, CPU and RSS results. [summary.json](summary.json) provides those measurements in machine-readable form. [audit-20260913-raw.tar.gz](audit-20260913-raw.tar.gz) contains every stdout/stderr file, resource JSON, raw sample log, SQL, deterministic retest, cross-check, apparatus source, provenance and test logs. Load time, database size and all raw resource counters are retained in the archive.

The rudb-bench gate passed formatting, clippy with warnings denied, documentation, Rust 1.85 compatibility, and 246 Rust tests. All 29 rudb CLI shell tests passed, including the timer regression. Four Python measurement tests passed, covering isolated peak RSS, failures and timeout cleanup, exact large-integer answer comparison, and the GNU time cross-check.
