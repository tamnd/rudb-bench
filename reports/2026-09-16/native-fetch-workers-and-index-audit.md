# Native fetch workers and string-index audit

This follow-up has two results: a smaller execution improvement for sparse wide fetches, and a measured requirement for the next native string index. The implementation is
[tamnd/rudb#712](https://github.com/tamnd/rudb/pull/712).

## One worker fan-out for the whole fetch

The parallel native `TableFetch` path previously launched and joined its column workers once for every stripe containing a winning row. It now resolves all requested ordinals first and keeps one fan-out alive while each column worker visits every required stripe. Arbitrary input order and duplicate ordinals remain supported; ClickBench's TopN path supplies sorted unique ordinals and restores result order afterwards.

Nine fresh processes per revision ran 10M Q24 in interleaved order on `gamingpc-wsl`, with 16 query threads and a 25 GiB memory limit. Both release binaries were built from main commit `a030eaf`; the candidate differed only in native row-fetch worker lifetime. Both read the same 9,999,750-row native database, and every paired output was byte-identical.

| Measurement | Baseline | Candidate | Change |
| --- | ---: | ---: | ---: |
| Q24 median query time | 177.358 ms | 175.217 ms | -1.21% |
| `TableFetch` median wall time | 18.216 ms | 16.417 ms | **-9.87%** |
| Median engine CPU | 972.931 ms | 964.398 ms | -0.88% |

The complete 43-query suite then ran five times per binary in fresh, interleaved processes. All 430 executions succeeded. Q24 is the only query using `TableFetch`.

| Measurement | Baseline | Candidate | Change |
| --- | ---: | ---: | ---: |
| Sum of 43 per-query medians | 4.051728 s | 4.032878 s | -0.47% |
| Sum of median engine CPU | 22.690757 s | 22.928791 s | +1.05% |
| Q24 median query time | 182.885 ms | 177.620 ms | **-2.88%** |

The suite CPU movement is host variance: the changed operator accounts for only Q24. Every deterministic output was byte-identical. Q22, Q23, Q32, Q33, Q39, Q40, and Q41 showed their already documented tied `ORDER BY ... LIMIT` variation.

## What the next native string index must preserve

Q21 and Q24 both use `URL LIKE '%google%'`. A read-only audit of the exact 10M native file walked its committed v7 directory, global URL dictionary, and URL code pages. This tests storage metadata shapes against real codes before changing the format.

| Fact | Value |
| --- | ---: |
| Rows | 9,999,750 |
| Stripes | 9,796 |
| URL dictionary values | 3,566,351 |
| URL dictionary payload | 549,124,017 bytes |
| Dictionary codes whose value contains `google` | 1,166 |
| Pages actually containing a matching code | 897 |

Exact page membership can rule out 8,899 of 9,796 URL pages, or **90.84%**. Two much smaller metadata proposals do not approximate that result:

| Persisted statistic | Pages skipped | Skip rate |
| --- | ---: | ---: |
| Page minimum/maximum global code | 653 | 6.67% |
| Per-code first/last page range | 388 | 3.96% |
| Exact code membership | 8,899 | **90.84%** |

Minimum and maximum fail because common codes widen almost every page's interval. First and last page fail because a few repeated matching values span most of the table even though the pages between them do not contain a match. Neither range belongs in native v8 as the answer to substring pruning.

The viable design is an exact, compressed code-to-page or page-to-code membership index, paired with a compact dictionary substring signature so a literal pattern does not read and search the entire 549 MB dictionary payload merely to identify its 1,166 matching codes. Both pieces must be built during load, checksummed independently, and read lazily only for a supported constant string predicate. The 90.84% figure is an upper bound until that complete consumer is benchmarked; it is not claimed as achieved performance.

Raw query artifacts are retained at:

- `/home/gopher/clickbench-native-audit/run-20260916-v3-direct-output/10m-native-fetch-groups`
- `/home/gopher/clickbench-native-audit/run-20260916-v3-direct-output/10m-native-fetch-groups-full43`

## Validation

- `cargo test -p rudb-native -p rudb-catalog -p rudb-exec --lib` (262 passed)
- warnings-as-errors Clippy for `rudb-native`, `rudb-catalog`, and `rudb-exec`
- nine-process interleaved Q24 A/B at 10M
- complete 43-query, five-process interleaved A/B at 10M
