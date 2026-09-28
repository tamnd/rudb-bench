# Bound-reader experiment, not merged

The pending native bound-reader patch was tested on main `79446a75`, before the latest-main rerun at `998cc427`. It reads a byte bound's encoded length before requesting its payload, avoiding repeated truncated-decoder attempts. It changes the shared codec and native directory reader, without changing the file format, building a saved query result, recognizing SQL text, or introducing an optional index. The optimization worktree has since been rebased to `1a3e8247`, but that rebased candidate was not measured in this experiment.

The large-sample native gain is modest, small-sample native gains are inconclusive, and the 10k Parquet control regresses. This patch remains unmerged. It is not part of the clean-main results.

## Evidence and limits

At earlier main `cd9d493c`, an untimed macOS sample of repeated open and COUNT operations found 2441 of 3704 main-thread samples under Catalog::table. This points to eager directory decoding. It is a repeated-open profile, not a fresh-process benchmark. The patch addresses only one bound-decoding cost; it does not defer directory opening or solve startup architecture. A fresh-process profile is needed before choosing the next architectural change.

The Linux experiment compared clean main `79446a75`, its patched build `bba64b01`, and released DuckDB v1.5.5. All three opened the same corresponding loaded files; native files were built by clean main, not by the candidate. Each query was COUNT(*) over the full table. Both engines used six threads and 4GB, RuDB stored answers were disabled, and Parquet mirroring was disabled. Every execution used a fresh process and retained exact expected output, wall, CPU, RSS, load, and command. No OS page-cache flushing was attempted. These raw samples contain 1000, 10000, 999975, and 9999750 rows.

The first diagnostic used rotating orders with one first execution plus 11 repetitions for the first three sizes and 51 at 10m. Its relative before/after order was imbalanced. The repeat used reversed neighboring rounds and one first execution plus 101 repetitions for each size, producing 2448 successful child executions. The summaries below use the repeat. Both experiments are retained, and the first is not presented as evidence of a gain. The host was shared, with pre-child load capped at 8.

## Fresh-process readings

| Sample | Format | Main wall (ms) | Candidate wall (ms) | DuckDB wall (ms) | Main RSS (MiB) | Candidate RSS (MiB) | DuckDB RSS (MiB) |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | native | 1.805 | 1.791 | 9.449 | 10.81 | 11.04 | 26.16 |
| 1k | parquet | 1.688 | 1.701 | 9.936 | 11.06 | 11.25 | 26.12 |
| 10k | native | 3.097 | 3.158 | 14.768 | 10.80 | 11.04 | 26.16 |
| 10k | parquet | 2.534 | 2.603 | 15.280 | 11.05 | 11.26 | 26.18 |
| 1m | native | 2.371 | 2.359 | 9.648 | 11.06 | 11.30 | 26.37 |
| 1m | parquet | 2.532 | 2.549 | 11.121 | 11.81 | 12.00 | 26.82 |
| 10m | native | 4.118 | 4.062 | 11.375 | 12.56 | 12.79 | 26.90 |
| 10m | parquet | 120.330 | 134.003 | 275.848 | 118.03 | 118.23 | 150.16 |

The table contains independent medians over the 101 subsequent fresh processes. Changing load or clocks can make a ratio of those medians disagree with the median ratio from matched rounds. Use the matched-round table below to assess this small patch, rather than selecting whichever summary is favorable. RSS medians show no 10x memory reduction.

## Matched-round comparison

| Sample | Format | Main / candidate wall, median of paired ratios | Bootstrap 95% interval | Candidate wins / 101 |
| --- | --- | ---: | --- | ---: |
| 1k | native | 0.996x | 0.981 to 1.021 | 49 |
| 1k | parquet | 0.990x | 0.978 to 1.009 | 47 |
| 10k | native | 0.997x | 0.958 to 1.059 | 49 |
| 10k | parquet | 0.945x | 0.925 to 0.978 | 36 |
| 1m | native | 1.002x | 0.971 to 1.023 | 51 |
| 1m | parquet | 0.986x | 0.976 to 1.000 | 43 |
| 10m | native | 1.030x | 1.009 to 1.051 | 61 |
| 10m | parquet | 1.004x | 0.986 to 1.013 | 54 |

The interval uses 3000 deterministic resamples of matched-round ratios with replacement, seed 19. It describes these observed rounds; it does not remove shared-host dependence or adjust for testing several sizes and formats. At 10m, the paired native median favors the candidate by about 3%, while the Parquet control is inconclusive. At 10k the Parquet control favors main. Small native effects do not establish a useful general improvement.

## Verification and next step

The common and native unit suites passed, including long bounds, truncated headers and payloads, unknown tags, a u32-max length claim, cursor position on errors, and bounds crossing tiny and 64 KiB windows. Strict Clippy and formatting checks passed before the rebase. A full candidate SQL differential was not completed because this performance result is insufficient for merging the change. Successful tests alone do not justify calling the patch an optimization.

The engine still eagerly opens every table directory. The next investigation should profile fresh-process startup and separate file opening, directory validation, schema setup, machine topology discovery, parsing, and query execution. A generic lazy table reader requires careful error propagation and consistent schema and constraint handling; this experiment does not implement it.

The [raw diagnostic archive](bound-header-diagnostic/q1-bound-header-diagnostics.tar.gz) retains both runs, metadata, every expected-output case, output and resource files, and original load observations. The [summaries and paired estimates](bound-header-diagnostic/analysis.json) retain the values in these tables. These files are excluded from every table in the [latest clean-main report](main-998cc427-clickbench.md).
