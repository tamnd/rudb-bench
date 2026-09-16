# Sixteen-byte fixed radix records

ClickBench Q33 groups ten million rows by `WatchID, ClientIP`, maintains `COUNT(*)`,
`SUM(IsRefresh)`, and `AVG(ResolutionWidth)`, then keeps ten rows ordered by count. On this sample
almost every key pair is unique. RuDB's fixed-width radix path therefore exchanges one record for
almost every input row before the sixteen owners build their aggregate tables.

The exchanged record stored an eight-byte hash beside the two keys and two `SMALLINT` aggregate
inputs. Alignment made that record 24 bytes, or about 240 MB for ten million rows. The hash is a
cheap function of the two integer keys: it is used once to choose an owner and once by that owner's
table. The new layout retains only the source values, occupies 16 bytes, and recomputes the hash at
the owner. This cuts exchanged record traffic by one third without changing the radix assignment or
group comparison.

Implementation: [tamnd/rudb#702](https://github.com/tamnd/rudb/pull/702).

## Targeted 10M result

Seven fresh processes per revision ran in interleaved order on `gamingpc-wsl`, with 16 threads and a
25 GiB memory limit. Both release binaries were built from main commit `56a378b`; the candidate
differed only by the record layout. Both read the same 9,999,750-row native database.

| Measurement | Baseline | Candidate | Change |
| --- | ---: | ---: | ---: |
| Q33 median query time | 246.761 ms | 214.362 ms | -13.13% |
| Q33 median child CPU | 801.474 ms | 731.370 ms | -8.75% |
| Q33 median tracked peak | 761.669 MiB | 666.669 MiB | -95.000 MiB |
| Q33 median process peak RSS | 896.273 MiB | 809.578 MiB | -86.695 MiB |

Q33's `COUNT(*)` is one at the TopN boundary, so the query is deliberately nondeterministic: any ten
of the tied rows are valid and repeated executions of one unchanged binary select different rows.
The radix partition unit test checks exact grouping, counts, sums, averages, collisions, and null
semantics. The result cardinality and every selected row's aggregate values are valid in every run.

## Full suite

The complete original 43-query suite then ran five times per binary in fresh processes, again
interleaved. All 430 queries succeeded. The change is selected only for Q33. The other per-query
differences change direction across repetitions and are run noise.

| Measurement | Baseline | Candidate | Change |
| --- | ---: | ---: | ---: |
| Sum of 43 per-query medians | 4.245130 s | 4.219385 s | -0.61% |
| Sum of median child CPU | 23.128365 s | 23.463525 s | +1.45% |
| Q33 median query time | 254.765 ms | 222.321 ms | -12.73% |
| Suite maximum tracked peak | 3,046.764 MiB | 3,046.764 MiB | unchanged |

Q33 saves 32.444 ms, slightly more than the suite's 25.745 ms reduction; the remainder is noise in
queries whose code did not change. The suite maximum does not move because another query, not Q33,
sets it. Q33's direct process RSS measurement is the relevant memory comparison for this change.

Raw artifacts are retained at:

- `/home/gopher/clickbench-native-audit/run-20260916-v3-direct-output/10m-native-fixed-record-16-q33`
- `/home/gopher/clickbench-native-audit/run-20260916-v3-direct-output/10m-native-fixed-record-16-q33-rss`
- `/home/gopher/clickbench-native-audit/run-20260916-v3-direct-output/10m-native-fixed-record-16-full43`

## Validation

- `cargo test -p rudb-exec --lib` (221 passed)
- `cargo clippy -p rudb-exec --all-targets -- -D warnings`
- seven-process interleaved Q33 A/B at 10M
- complete 43-query, five-process interleaved A/B at 10M
