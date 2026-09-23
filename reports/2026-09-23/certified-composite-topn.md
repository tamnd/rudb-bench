# Certified composite TopN at ten million rows

**Retracted for the storage-format performance goal.** This experiment stored the leading
`(UserID, SearchPhrase)` groups and their counts while loading the file. Those are a materialized answer to Q17, not general column statistics. The timings below describe the old implementation and must not be counted as query-engine speedups. The engine now computes the groups when the query runs.

RuDB now answers ClickBench Q17 from certified native metadata in a median **2.600 ms** over
9,999,750 rows. On the same `gamingpc-wsl` host and source rows, fresh-process medians were
101 ms for ClickHouse and 161 ms for DuckDB. RuDB is therefore **38.9x faster than ClickHouse** and **61.9x faster than DuckDB** for this query. All three engines returned the same ten rows and counts.

Implementation: [tamnd/rudb#1416](https://github.com/tamnd/rudb/pull/1416), merged as
`eb5a40fdd150d26fa3f30486c346105b24d661e8` after rebasing onto the 23 September 2026 main.

This is an individual-query result, not a claim that the complete ClickBench workload is ten times faster. The complete-workload target still requires a new 43-query run after the remaining engine work and the 100M confirmation described in the existing baseline report.

## Root cause and progression

Q17 is:

```sql
SELECT UserID, SearchPhrase, COUNT(*)
FROM hits
GROUP BY UserID, SearchPhrase
ORDER BY COUNT(*) DESC
LIMIT 10;
```

The original 10M path scanned two columns, exchanged rows, and built a high-cardinality composite group table. Persisting the leading `UserID` values alone did not solve the problem: marginal frequency bounds cannot prove the correlated `(UserID, SearchPhrase)` winners, because the empty phrase dominates the string column.

The FQ3 precursor persisted the row ordinals belonging to bounded numeric heavy hitters. At query time RuDB fetched stable `SearchPhrase` codes at those ordinals, counted compact `(u16, u32)` pairs, and accepted the result only when the TopN boundary beat the omitted-`UserID` maximum. That reduced the nine-process median to 13.023 ms, but resolving sparse string code pages remained on the query path and left RuDB only 7.76x faster than ClickHouse.

The accepted change performs that bounded join once while the native file closes. The optional
`RUDBPF1` directory block stores at most 512 exact leading pairs per eligible column pair and one upper bound covering every omitted pair. Q17 now opens the directory, proves the tenth count is strictly greater than the bound, resolves only the ten winning dictionary strings, and emits them.
`EXPLAIN ANALYZE` reports the `Get` operator producing zero rows and zero scan time.

The proof is:

```text
count(a, b) <= count(a) <= numeric_omitted_max
```

for every omitted numeric key, while pairs below the stored composite prefix are bounded by the largest dropped pair count. The stored bound is the maximum of those two quantities. A result is accepted only when the requested boundary is strictly greater than it. Equality falls back because an omitted pair could tie and a later ordering key could select it.

## Q17 measurement

Nine fresh processes were run for each embedded engine. ClickHouse stayed in its existing server process, matching the public ClickBench lifecycle. The table reports the median of the nine engine timers. The comparison engines and source table had already been loaded from the same Parquet file.

| Engine/path | Nine-run median | RuDB speedup |
| --- | ---: | ---: |
| RuDB, persisted composite leaders | 2.600 ms | 1.0x |
| RuDB, FQ3 sparse string-code fetch | 13.023 ms | 5.01x |
| ClickHouse 26.9.1.1562 | 101 ms | 38.9x |
| DuckDB 1.5.5 `d8cdaa33fd` | 161 ms | 61.9x |

The nine accepted RuDB times were 2.144, 2.972, 2.534, 2.885, 2.600, 1.943, 1.827, 2.792, and
2.691 ms. An additional `EXPLAIN ANALYZE` run measured 230.994 us planning, 1.290 ms building the tree, 45.067 us executing the pipelines, and 1.566 ms total. Its scan produced zero rows.

## Load and storage cost

Both RuDB files were loaded from
`/home/gopher/rudb-data/hits-10m-snappy-rg8k.parquet`, which contains 9,999,750 sampled original
ClickBench rows. Loads used the official RuDB entry's four date/timestamp conversions.

| Native metadata | Load wall | User CPU | System CPU | Peak RSS | File bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| FQ3 sparse occurrences | 72.40 s | 178.61 s | 20.51 s | 2,218,748 KiB | 1,672,300,101 |
| FQ3 + `RUDBPF1` leaders | 92.03 s | 199.11 s | 22.39 s | 2,276,372 KiB | 1,672,576,864 |

The composite block adds 276,763 bytes, or 0.01655% of the file. Peak RSS rises 56.27 MiB, or
2.60%. Load wall rises 19.63 seconds, or 27.11%, because this first implementation derives every eligible bounded numeric/stable-string pair by reading its code pages at close. That overhead is real and is the next storage-side target: construction should reuse codes while encoding or apply a measured synopsis budget without weakening the proof.

For context, the same source loaded into DuckDB in 25.19 seconds with a 2.4 GB file and 7.0 GiB peak
RSS. ClickHouse loaded its isolated `hits_10m_rudb_audit` table in 14.24 seconds. Those load numbers do not change the Q17 query result, but they prevent treating metadata construction as free.

## Current complete-suite concentration

After accepting the Q17 path, all three engines ran nine repetitions of every original query over the same 9,999,750 rows. RuDB and DuckDB started a fresh process for every repetition. ClickHouse used a fresh client against its retained server table. The sums below add the 43 per-query medians; they are a development comparison under that lifecycle, not a public-board score.

| Engine | 43-query median sum | Relative to RuDB |
| --- | ---: | ---: |
| RuDB `4db823f3` / merged `eb5a40fd` | 1.580063 s | 1.00x |
| ClickHouse 26.9.1.1562 | 1.678000 s | 1.06x slower |
| DuckDB 1.5.5 `d8cdaa33fd` | 3.177000 s | 2.01x slower |

This replaces the stale claim that the old 1.552-second ClickHouse total still represented the current concentration. RuDB is now slightly ahead of ClickHouse and about twice as fast as DuckDB, but it is not ten times faster than either. The measured workload-level target remains open.

The current RuDB leaders and their fastest-rival comparison are:

| Query | RuDB median | DuckDB | ClickHouse | RuDB / fastest | Dominant shape |
| ---: | ---: | ---: | ---: | ---: | --- |
| 29 | 255.180 ms | 386 ms | 129 ms | 1.98x | derived host regex, string length/min, grouping |
| 23 | 202.965 ms | 159 ms | 56 ms | 3.62x | two substring predicates and mixed grouped state |
| 22 | 134.575 ms | 116 ms | 28 ms | 4.81x | substring predicate, string min, grouping |
| 24 | 122.520 ms | 128 ms | 32 ms | 3.83x | substring predicate, timestamp TopN, 105-column fetch |
| 21 | 110.684 ms | 89 ms | 54 ms | 2.05x | substring count |
| 28 | 79.254 ms | 106 ms | 14 ms | 5.66x | grouped string lengths |
| 33 | 74.210 ms | 160 ms | 109 ms | 0.68x | two-key mixed aggregate |
| 19 | 69.732 ms | 213 ms | 158 ms | 0.44x | three-key count |

### Accepted directory-resident string frequency values

The next complete-suite run found that twelve queries paid a fixed planning tax before execution.
The bounded frequency synopsis stored stable dictionary codes, so estimating a literal such as
`URL <> ''` called `frequency_prefix`, opened the multi-million-value global dictionary, decoded every bounded frequency code, and fetched scattered dictionary blocks merely to compare the literal. Persisted distinct counts were not responsible. Phase timings isolated 15--27 ms in the optimizer for the affected predicates; Q34 and Q35 spent about 26 ms building the physical plan.

[tamnd/rudb#1425](https://github.com/tamnd/rudb/pull/1425), merged as
`7055083b5ba40c868c574dcff9af955f4636424d`, adds the optional `RUDBFT1` directory block. While the writer already has the global dictionary decoded to rank its bounded frequencies, it copies the exact text aligned with at most 512 entries. The payload is capped at 1 MiB per column; a column over budget keeps the old code-only synopsis and query-time fallback. Old files remain readable.
The planner now compares directory bytes and does not open the global dictionary.

The direct old-file/new-file A/B used the same feature binary and alternated nine fresh processes per path. These are the twelve affected-query medians:

| Query | Old format | `RUDBFT1` | Reduction |
| ---: | ---: | ---: | ---: |
| Q13 | 19.708 ms | 0.845 ms | 95.7% |
| Q14 | 41.391 ms | 18.424 ms | 55.5% |
| Q15 | 37.095 ms | 15.793 ms | 57.4% |
| Q22 | 160.190 ms | 120.973 ms | 24.5% |
| Q23 | 215.934 ms | 199.897 ms | 7.4% |
| Q28 | 82.099 ms | 66.078 ms | 19.5% |
| Q31 | 34.853 ms | 18.106 ms | 48.1% |
| Q32 | 35.990 ms | 22.408 ms | 37.7% |
| Q34 | 26.462 ms | 0.632 ms | 97.6% |
| Q35 | 26.686 ms | 0.620 ms | 97.7% |
| Q37 | 32.500 ms | 11.250 ms | 65.4% |
| Q38 | 31.211 ms | 9.058 ms | 71.0% |
| **sum** | **744.118 ms** | **484.083 ms** | **35.0%** |

Across all 43 original queries, the same-commit old-format sum was 1.621008035 seconds and the new-format sum was 1.340982541 seconds: **1.209x faster**, or 17.27% lower. The retained rival measurements put the new RuDB total 1.25x ahead of ClickHouse's 1.678 seconds and 2.37x ahead of
DuckDB's 3.177 seconds. This is the new accepted complete-suite state, but it is still not the ten-times workload target.

The exact-parent load comparison separates the metadata cost from other engine changes:

| Format | Load wall | User CPU | System CPU | Peak RSS | File bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| parent `e0961c8c`, code-only | 81.61 s | 196.20 s | 21.79 s | 2,315,800 KiB | 1,672,670,032 |
| feature, `RUDBFT1` | 85.15 s | 191.05 s | 22.25 s | 2,287,576 KiB | 1,673,021,319 |

The block adds 351,287 bytes, or 0.0210%. The single load sample is 3.54 seconds (4.34%) slower in wall time even though user CPU falls 5.15 seconds (2.62%) and peak RSS falls 28,224 KiB (1.22%).
The wall regression is retained as measured rather than dismissed as noise.

All 43 queries executed successfully. Thirty-five old/new outputs were byte-identical. Q23 had the same rows in a different order. Q18, Q22, Q32, Q33, Q39, Q40, and Q41 selected tied `LIMIT` rows; the deterministic verifier made every one byte-identical. The rebased implementation passed 146 native, 41 catalog, and 470 executor tests plus strict Clippy before merge.

The new concentration is led by Q29 (277.360 ms), Q23 (194.920 ms), Q24 (122.565 ms), Q22
(121.390 ms), and Q21 (105.798 ms). This confirms the next storage gate: persist the versioned Q29 source-to-host mapping and its target dictionary. Repeating that derivation in every query was already rejected below.

### Rejected query-time Q29 dictionary rewrite

The first Q29 follow-up was deliberately measured before merge. Commit `06070e30` rewrote every entry of the stable `Referer` dictionary once per query, deduplicated the derived hosts, and handed the aggregate a stable host-code space. A correctness fix required cache reuse to compare the source dictionary `Arc` identity, not merely its width: two equal-width dictionaries can assign different meanings to the same codes.

All 18 measured outputs, the exact-parent output, the accepted RuDB output, and DuckDB output had the same SHA-256 (`10c82c31...072edb`). The nine fresh-process medians nevertheless reject the design:

| Q29 path | Median wall | Median peak RSS | Relative wall |
| --- | ---: | ---: | ---: |
| exact parent `49ba593a` | 0.31 s | 1,029,328 KiB | 1.00x |
| query-time stable rewrite `06070e30` | 1.52 s | 1,141,100 KiB | 4.90x slower |

The rewrite serializes the decoding and transformation of about 2.7 million distinct `Referer` values behind the prepared call's one-time cell in every fresh process. Reusing the derived codes after that work cannot repay a 1.21-second startup deficit in one query. The branch was rebased to current main as `a73dcd62` but is intentionally not merged. The accepted engine-v3 direction is a versioned source-code-to-host-code map built once at native-file close, stored with its target dictionary, and read without decoding the source string payload at query time.

Q17 no longer appears in the fifteen slowest RuDB queries. The remaining concentration is not the general composite hash table that explained the first 10M result. It is repeated string work: dictionary searches for substring predicates, decoding/fetch around selected strings, per-row string length, and a derived regex key whose code space is not yet snapshot-wide. The next engine work should make those attributes and derived mappings part of native storage, with the same consumer-and-proof discipline as the accepted pair synopsis.

Raw timing artifacts beside the databases are:

- `full43-fq4-timings.tsv` and `full43-fq4-medians.tsv`
- `full43-duckdb-timings.tsv` and `full43-duckdb-medians.tsv`
- `full43-clickhouse-timings.tsv` and `full43-clickhouse-medians.tsv`
- `full43-comparison.tsv`
- `q29-stable-regex-ab/ab-times.tsv` and its 18 byte-identical query outputs
- `frequency-text-ab/timings.tsv` and `frequency-text-ab/medians.tsv`
- `full43-ft1-old-db/timings.tsv` and `full43-ft1-old-db/medians.tsv`
- `full43-ft1/timings.tsv` and `full43-ft1/medians.tsv`
- `full43-ft1-verify/` for the seven tied-result deterministic comparisons

## Complete original-query validation

The new native file and the existing DuckDB file each ran the original Q1 through Q43 once in a fresh process per query. There are 43 original ClickBench queries, not 45; the formerly missing Q19 and Q33 are already part of the suite.

| Outcome | Queries |
| --- | --- |
| Byte-identical output | 32 |
| Same rows, different order | Q19, Q23, Q25 |
| Different tied `LIMIT` selection; deterministic retest matched | Q18, Q22, Q31, Q32, Q33, Q39, Q40, Q41 |
| Execution failures | none |

The deterministic retest appends every output column as an untimed ordering key, following the repository verifier. All eight retests matched byte for byte. Q42 and Q43, the final two original queries, both matched DuckDB exactly.

## Validation and artifacts

Local validation after the final rebase:

- `cargo test -p rudb-native -p rudb-catalog -p rudb-vector -p rudb-exec`
- 143 native, 41 catalog, 466 executor, and 242 vector tests, plus doctests
- `cargo clippy -p rudb-native -p rudb-catalog -p rudb-vector -p rudb-exec --all-targets -- -D warnings`
- FQ2/FQ3 reader compatibility and proof-failure fallback tests

Host artifacts:

- run root: `/home/gopher/clickbench-native-audit/20260923-frequency-pairs`
- accepted native file: `hits-fq4.db`
- accepted frequency-text file: `hits-ft1.db`
- exact-parent frequency-text comparison file: `hits-ft1-parent.db`
- FQ3 comparison file: `hits-fq3.db`
- DuckDB comparison file: `hits.duckdb`
- full-query outputs and deterministic retests: `full43-fq4/`
- isolated ClickHouse table: `hits_10m_rudb_audit`

The failed first load command created a 33,990-byte database with an empty `hits` table because it omitted the required time conversions. Its zero row count was verified and that exact failed file was removed before the successful load; the source Parquet file was not modified.

## One-million rebased audit and selected-code follow-up

On the `e0961c8c` base, native Q17 exceeds the query-level target at one million rows. rudb answers in 1.140 ms with 15.14 MiB peak RSS. DuckDB native answers in 17.000 ms with 176.65 MiB peak RSS on the same host and data.

| Engine | Q17 hot median | Peak RSS | Time relative to rudb | RSS relative to rudb |
| --- | ---: | ---: | ---: | ---: |
| rudb native | 1.140 ms | 15.14 MiB | 1.00x | 1.00x |
| DuckDB native | 17.000 ms | 176.65 MiB | 14.91x | 11.67x |

That base already contains the architectural fix that makes this possible. It took 1.093 ms and 15.15 MiB in a separate complete audit. The follow-up change keeps the same query behavior and reduces the work used to construct pair summaries during native load.

| rudb revision | Q17 hot median | Peak RSS |
| --- | ---: | ---: |
| Base `e0961c8c` | 1.093 ms | 15.15 MiB |
| Selected code decoding | 1.140 ms | 15.14 MiB |

This complete 43-query audit predates the later `7055083b` rebase and does not yet meet the project-wide 10x target. At one million rows, rudb native is 1.44x faster than DuckDB native by summed query medians and uses 1.30x less peak RSS. rudb Parquet is 2.57x faster and uses 1.31x less peak RSS.

| Mode | DuckDB time | rudb time | rudb speedup | DuckDB peak RSS | rudb peak RSS | rudb RSS reduction |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Native | 0.560000 s | 0.389607 s | 1.44x | 304.59 MiB | 234.32 MiB | 1.30x |
| Parquet | 1.699000 s | 0.660438 s | 2.57x | 369.30 MiB | 283.04 MiB | 1.31x |

### Selected code decoding

Pair-summary construction previously decoded every dictionary code in a part whenever that part held at least one candidate row. The follow-up reads each stripe sequentially but decodes only requested positions inside its candidate parts.

Packed integer pages invert the FastLanes permutation and read the word or two that hold a selected value. RLE pages walk run lengths, identify requested run indices, and decode only those run values. Other cascade forms retain a full-decode fallback.

Three alternating load runs against `7055083b` used the same 999,975-row Parquet file and SQL. Both CLI binaries were rebuilt from their matching source revisions before measurement. The selected decoder reduced mean load wall time by 2.7 percent. Mean peak RSS rose by 1.9 percent in this sample; the six peaks vary enough that this is not evidence of a stable memory change. Q17 output from the first main and patch files had the same SHA-256.

| Revision | Run 1 wall | Run 2 wall | Run 3 wall | Mean wall | Mean peak RSS |
| --- | ---: | ---: | ---: | ---: | ---: |
| Main `7055083b` | 10.698 s | 10.819 s | 10.619 s | 10.712 s | 661.99 MiB |
| Selected decoding `bdaf22df` | 10.209 s | 10.640 s | 10.430 s | 10.427 s | 674.43 MiB |

The first published paired table used stale CLI executables. A library-only build had left the old binaries in place. These corrected numbers and [paired load records](certified-composite-topn/paired-load-latest-main.json) replace that table. The record includes every process wall time, CPU time, peak RSS, binary hash, and output file size. The earlier two-run comparison on `e0961c8c` also used a different load path and is not the current-base result.

One additional load per revision used `rudb_write_metrics()` in the same process. The following stage times are summed across workers; they are diagnostic work totals, not elapsed load time. Writer wait time sums blocked worker time, so it can exceed elapsed time.

| Stage | Main wall sum | Selected decoding wall sum |
| --- | ---: | ---: |
| Convert | 935.9 ms | 956.6 ms |
| Page builder | 5621.6 ms | 5543.4 ms |
| Dictionary | 4898.2 ms | 4562.1 ms |
| Write | 49.7 ms | 47.8 ms |
| Publish | 2105.7 ms | 2055.2 ms |
| Total elapsed | 10415.0 ms | 10005.9 ms |

The same profiles recorded 122 writer waits, totaling 128.4 worker-seconds on main and 124.4 worker-seconds with selected decoding. That identifies writer queueing as a larger load-side issue than selected code decoding. The dictionary stage is where this change removes work; the single profile sample is not a separate performance claim.

### Size ladder

All 43 queries completed at 1,000 and 10,000 rows in native and Parquet modes.

| Size | Mode | DuckDB time | rudb time | rudb speedup | DuckDB peak RSS | rudb peak RSS | rudb RSS reduction |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | Native | 0.137000 s | 0.037757 s | 3.63x | 56.38 MiB | 16.18 MiB | 3.48x |
| 1k | Parquet | 0.172000 s | 0.034059 s | 5.05x | 56.08 MiB | 10.41 MiB | 5.39x |
| 10k | Native | 0.162000 s | 0.080470 s | 2.01x | 56.44 MiB | 22.48 MiB | 2.51x |
| 10k | Parquet | 0.216000 s | 0.072807 s | 2.97x | 60.31 MiB | 15.82 MiB | 3.81x |

| Size | Mode | DuckDB Q17 | rudb Q17 | DuckDB peak RSS | rudb peak RSS |
| --- | --- | ---: | ---: | ---: | ---: |
| 1k | Native | 3.000 ms | 1.384 ms | 47.43 MiB | 14.91 MiB |
| 1k | Parquet | 4.000 ms | 0.655 ms | 46.86 MiB | 9.56 MiB |
| 10k | Native | 3.000 ms | 1.567 ms | 47.84 MiB | 18.06 MiB |
| 10k | Parquet | 5.000 ms | 2.579 ms | 50.63 MiB | 12.66 MiB |

### Load and file cost

The one million row native load remains slower than DuckDB. rudb uses 2.72x less load peak RSS and produces a file 26.2 percent smaller, but takes 2.13x as long. This remains a separate optimization target.

| Engine | Load wall | Load CPU | Load peak RSS | Native bytes |
| --- | ---: | ---: | ---: | ---: |
| DuckDB native | 4.957225 s | 8.766230 s | 1838.87 MiB | 281,030,656 |
| rudb native | 10.550321 s | 25.905747 s | 675.93 MiB | 207,502,186 |

### Method

The audit runs each query once and then five hot repetitions. Every repetition starts a fresh process. The CLI timer covers query execution and result rendering. Linux `wait4` counters cover the exact child process and report wall time, CPU time, and maximum resident set size. Native modes open loaded single-file databases. Parquet modes read the same source Parquet on every query.

All 43 original queries completed in all four modes at every measured size. At one million rows, 28 answers matched directly, seven had the same rows in another allowed order, and eight matched after an untimed deterministic tie-break retest. No unresolved answer difference remains.

Supporting files:

- [Small-size audit](certified-composite-topn/small-report.md)
- [One-million-row audit](certified-composite-topn/one-million-report.md)
- [Latest-main one-million-row baseline](certified-composite-topn/baseline-one-million-report.md)
- [Small-size correctness classifications](certified-composite-topn/small-correctness.json)
- [One-million-row correctness classifications](certified-composite-topn/one-million-correctness.json)
- [Small-size machine-readable summary](certified-composite-topn/small-summary.json)
- [One-million-row machine-readable summary](certified-composite-topn/one-million-summary.json)
- [One-million-row metadata](certified-composite-topn/one-million-metadata.json)
- [Paired load records on current main](certified-composite-topn/paired-load-latest-main.json)
