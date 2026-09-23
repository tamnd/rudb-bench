# Certified composite TopN at ten million rows

RuDB now answers ClickBench Q17 from certified native metadata in a median **2.600 ms** over
9,999,750 rows. On the same `gamingpc-wsl` host and source rows, fresh-process medians were
101 ms for ClickHouse and 161 ms for DuckDB. RuDB is therefore **38.9x faster than ClickHouse**
and **61.9x faster than DuckDB** for this query. All three engines returned the same ten rows and
counts.

Implementation: [tamnd/rudb#1416](https://github.com/tamnd/rudb/pull/1416), merged as
`eb5a40fdd150d26fa3f30486c346105b24d661e8` after rebasing onto the 23 September 2026 main.

This is an individual-query result, not a claim that the complete ClickBench workload is ten times
faster. The complete-workload target still requires a new 43-query run after the remaining engine
work and the 100M confirmation described in the existing baseline report.

## Root cause and progression

Q17 is:

```sql
SELECT UserID, SearchPhrase, COUNT(*)
FROM hits
GROUP BY UserID, SearchPhrase
ORDER BY COUNT(*) DESC
LIMIT 10;
```

The original 10M path scanned two columns, exchanged rows, and built a high-cardinality composite
group table. Persisting the leading `UserID` values alone did not solve the problem: marginal
frequency bounds cannot prove the correlated `(UserID, SearchPhrase)` winners, because the empty
phrase dominates the string column.

The FQ3 precursor persisted the row ordinals belonging to bounded numeric heavy hitters. At query
time RuDB fetched stable `SearchPhrase` codes at those ordinals, counted compact `(u16, u32)` pairs,
and accepted the result only when the TopN boundary beat the omitted-`UserID` maximum. That reduced
the nine-process median to 13.023 ms, but resolving sparse string code pages remained on the query
path and left RuDB only 7.76x faster than ClickHouse.

The accepted change performs that bounded join once while the native file closes. The optional
`RUDBPF1` directory block stores at most 512 exact leading pairs per eligible column pair and one
upper bound covering every omitted pair. Q17 now opens the directory, proves the tenth count is
strictly greater than the bound, resolves only the ten winning dictionary strings, and emits them.
`EXPLAIN ANALYZE` reports the `Get` operator producing zero rows and zero scan time.

The proof is:

```text
count(a, b) <= count(a) <= numeric_omitted_max
```

for every omitted numeric key, while pairs below the stored composite prefix are bounded by the
largest dropped pair count. The stored bound is the maximum of those two quantities. A result is
accepted only when the requested boundary is strictly greater than it. Equality falls back because
an omitted pair could tie and a later ordering key could select it.

## Q17 measurement

Nine fresh processes were run for each embedded engine. ClickHouse stayed in its existing server
process, matching the public ClickBench lifecycle. The table reports the median of the nine engine
timers. The comparison engines and source table had already been loaded from the same Parquet file.

| Engine/path | Nine-run median | RuDB speedup |
| --- | ---: | ---: |
| RuDB, persisted composite leaders | 2.600 ms | 1.0x |
| RuDB, FQ3 sparse string-code fetch | 13.023 ms | 5.01x |
| ClickHouse 26.9.1.1562 | 101 ms | 38.9x |
| DuckDB 1.5.5 `d8cdaa33fd` | 161 ms | 61.9x |

The nine accepted RuDB times were 2.144, 2.972, 2.534, 2.885, 2.600, 1.943, 1.827, 2.792, and
2.691 ms. An additional `EXPLAIN ANALYZE` run measured 230.994 us planning, 1.290 ms building the
tree, 45.067 us executing the pipelines, and 1.566 ms total. Its scan produced zero rows.

## Load and storage cost

Both RuDB files were loaded from
`/home/gopher/rudb-data/hits-10m-snappy-rg8k.parquet`, which contains 9,999,750 sampled original
ClickBench rows. Loads used the official RuDB entry's four date/timestamp conversions.

| Native metadata | Load wall | User CPU | System CPU | Peak RSS | File bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| FQ3 sparse occurrences | 72.40 s | 178.61 s | 20.51 s | 2,218,748 KiB | 1,672,300,101 |
| FQ3 + `RUDBPF1` leaders | 92.03 s | 199.11 s | 22.39 s | 2,276,372 KiB | 1,672,576,864 |

The composite block adds 276,763 bytes, or 0.01655% of the file. Peak RSS rises 56.27 MiB, or
2.60%. Load wall rises 19.63 seconds, or 27.11%, because this first implementation derives every
eligible bounded numeric/stable-string pair by reading its code pages at close. That overhead is
real and is the next storage-side target: construction should reuse codes while encoding or apply a
measured synopsis budget without weakening the proof.

For context, the same source loaded into DuckDB in 25.19 seconds with a 2.4 GB file and 7.0 GiB peak
RSS. ClickHouse loaded its isolated `hits_10m_rudb_audit` table in 14.24 seconds. Those load numbers
do not change the Q17 query result, but they prevent treating metadata construction as free.

## Current complete-suite concentration

After accepting the Q17 path, all three engines ran nine repetitions of every original query over
the same 9,999,750 rows. RuDB and DuckDB started a fresh process for every repetition. ClickHouse
used a fresh client against its retained server table. The sums below add the 43 per-query medians;
they are a development comparison under that lifecycle, not a public-board score.

| Engine | 43-query median sum | Relative to RuDB |
| --- | ---: | ---: |
| RuDB `4db823f3` / merged `eb5a40fd` | 1.580063 s | 1.00x |
| ClickHouse 26.9.1.1562 | 1.678000 s | 1.06x slower |
| DuckDB 1.5.5 `d8cdaa33fd` | 3.177000 s | 2.01x slower |

This replaces the stale claim that the old 1.552-second ClickHouse total still represented the
current concentration. RuDB is now slightly ahead of ClickHouse and about twice as fast as DuckDB,
but it is not ten times faster than either. The measured workload-level target remains open.

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

Q17 no longer appears in the fifteen slowest RuDB queries. The remaining concentration is not the
general composite hash table that explained the first 10M result. It is repeated string work:
dictionary searches for substring predicates, decoding/fetch around selected strings, per-row
string length, and a derived regex key whose code space is not yet snapshot-wide. The next engine
work should make those attributes and derived mappings part of native storage, with the same
consumer-and-proof discipline as the accepted pair synopsis.

Raw timing artifacts beside the databases are:

- `full43-fq4-timings.tsv` and `full43-fq4-medians.tsv`
- `full43-duckdb-timings.tsv` and `full43-duckdb-medians.tsv`
- `full43-clickhouse-timings.tsv` and `full43-clickhouse-medians.tsv`
- `full43-comparison.tsv`

## Complete original-query validation

The new native file and the existing DuckDB file each ran the original Q1 through Q43 once in a
fresh process per query. There are 43 original ClickBench queries, not 45; the formerly missing Q19
and Q33 are already part of the suite.

| Outcome | Queries |
| --- | --- |
| Byte-identical output | 32 |
| Same rows, different order | Q19, Q23, Q25 |
| Different tied `LIMIT` selection; deterministic retest matched | Q18, Q22, Q31, Q32, Q33, Q39, Q40, Q41 |
| Execution failures | none |

The deterministic retest appends every output column as an untimed ordering key, following the
repository verifier. All eight retests matched byte for byte. Q42 and Q43, the final two original
queries, both matched DuckDB exactly.

## Validation and artifacts

Local validation after the final rebase:

- `cargo test -p rudb-native -p rudb-catalog -p rudb-vector -p rudb-exec`
- 143 native, 41 catalog, 466 executor, and 242 vector tests, plus doctests
- `cargo clippy -p rudb-native -p rudb-catalog -p rudb-vector -p rudb-exec --all-targets -- -D warnings`
- FQ2/FQ3 reader compatibility and proof-failure fallback tests

Host artifacts:

- run root: `/home/gopher/clickbench-native-audit/20260923-frequency-pairs`
- accepted native file: `hits-fq4.db`
- FQ3 comparison file: `hits-fq3.db`
- DuckDB comparison file: `hits.duckdb`
- full-query outputs and deterministic retests: `full43-fq4/`
- isolated ClickHouse table: `hits_10m_rudb_audit`

The failed first load command created a 33,990-byte database with an empty `hits` table because it
omitted the required time conversions. Its zero row count was verified and that exact failed file
was removed before the successful load; the source Parquet file was not modified.
