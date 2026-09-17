# Sparse native page fetch

ClickBench Q24 filters ten million URLs, keeps ten rows with TopN, and then fetches all 105 columns for those rows. The native directory already records the exact offset, length, and checksum of every column page, but `TableFetch` used the sequential scan reader. That reader deliberately prefetches up to 32 adjacent 1,024-row stripe pages as one column extent. Fetching a few final rows therefore read as much as 32 times the required encoded data for each projected column.

The native reader now has a sparse path that reads exactly one directory page and verifies the same page checksum. Only late `TableFetch` uses it. Sequential scans retain extent prefetch and its bounded cache.

Implementation: [tamnd/rudb#709](https://github.com/tamnd/rudb/pull/709).

## Targeted 10M result

Nine fresh processes per revision ran in interleaved order on `gamingpc-wsl`, with 16 query threads and a 25 GiB memory limit. The baseline is the parallel wide-fetch binary from
[tamnd/rudb#705](https://github.com/tamnd/rudb/pull/705); the candidate differs only in choosing an
exact page read for sparse native fetches. Both binaries read the same 9,999,750-row native file.
All nine candidate results were byte-identical to their paired baseline.

| Measurement | Baseline | Candidate | Change |
| --- | ---: | ---: | ---: |
| Q24 median query time | 211.364 ms | 192.053 ms | **-9.14%** |
| `TableFetch` median wall time | 45.487 ms | 24.529 ms | **-46.07%** |
| Median engine CPU | 1,081.447 ms | 1,099.255 ms | +1.65% |

The CPU movement is scan variance rather than work moved into the sparse fetch: median operator CPU was 2.388 ms before and 2.444 ms after for `TableFetch`, while Scan and Filter together moved by about five milliseconds. The removed cost is scan-oriented page prefetch and waiting, visible in the 20.957 ms reduction in fetch wall time.

## Full suite

The complete original 43-query suite then ran five times per binary in fresh processes, again interleaved. All 430 executions succeeded. Q24 is the only query that uses `TableFetch` in this suite.

| Measurement | Baseline | Candidate | Change |
| --- | ---: | ---: | ---: |
| Sum of 43 per-query medians | 4.087222 s | 4.084065 s | -0.08% |
| Sum of median engine CPU | 22.440790 s | 22.673313 s | +1.04% |
| Q24 median query time | 195.808 ms | 181.135 ms | **-7.49%** |

The suite total and CPU difference are within host variance; only Q24 takes the changed path. Every deterministic output was byte-identical. Q22, Q23, Q32, Q33, Q39, Q40, and Q41 showed their already documented tied `ORDER BY ... LIMIT` variation. Q24 was byte-identical in the targeted and full-suite comparisons.

The remaining fetch cost is now page decoding rather than 32-page read amplification. A useful next native-format step is selected-row decoding: the directory or page header can expose codec-specific offsets/checkpoints so ten row ordinals do not require materializing every value in each 1,024-row page. This keeps format metadata tied to a measured consumer instead of adding statistics that no operator reads.

Raw artifacts are retained at:

- `/home/gopher/clickbench-native-audit/run-20260916-v3-direct-output/10m-native-sparse-fetch`
- `/home/gopher/clickbench-native-audit/run-20260916-v3-direct-output/10m-native-sparse-fetch-full43`

## Validation

- `cargo test -p rudb-native -p rudb-catalog` (38 passed)
- `cargo test -p rudb-exec --lib` (221 passed)
- warnings-as-errors Clippy for `rudb-native`, `rudb-catalog`, and `rudb-exec`
- nine-process interleaved Q24 A/B at 10M
- complete 43-query, five-process interleaved A/B at 10M
