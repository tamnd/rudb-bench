# Two-key encoded radix count

The implementation is [tamnd/rudb#726](https://github.com/tamnd/rudb/pull/726). It extends the native encoded count path from `(BIGINT, BIGINT, VARCHAR)` to `(BIGINT, VARCHAR)` group keys under a count-descending TopN.

Q17 groups by `(UserID, SearchPhrase)`. At one million rows it has about 902,000 distinct pairs. The previous general aggregate copied variable-width strings into its group table. The new path keeps `SearchPhrase` as a stable native dictionary code, scatters fixed 24-byte records by radix partition, and materializes strings only for bounded partition winners.

## Q17 comparison

| Rows | DuckDB native | rudb native before | rudb native after | After vs DuckDB |
| ---: | ---: | ---: | ---: | ---: |
| 1,000 | 3.000 ms | 0.305 ms | 0.353 ms | 8.50x faster |
| 10,000 | 3.000 ms | 1.053 ms | 0.666 ms | 4.50x faster |
| 1,000,000 | 18.000 ms | 31.486 ms | 16.701 ms | 1.08x faster |

At one million rows, rudb Q17 peak RSS is 86.18 MiB and DuckDB peak RSS is 197.66 MiB. The new rudb path is 1.88x faster than the previous rudb path and uses 2.29x less memory than DuckDB. It does not reach the 10x target because it still writes and aggregates one fixed record for every input row.

## Complete one million row audit

| Engine and format | Suite time | Peak RSS | Completed |
| --- | ---: | ---: | ---: |
| DuckDB native | 567.000 ms | 313.44 MiB | 43/43 |
| rudb native | 491.367 ms | 134.11 MiB | 43/43 |
| DuckDB Parquet | 1649.000 ms | 362.18 MiB | 43/43 |
| rudb Parquet | 811.067 ms | 152.15 MiB | 43/43 |

The native suite is 1.15x faster than DuckDB and uses 2.34x less peak memory. These suite figures include run-to-run movement outside Q17 and remain far from the project-wide target.

## Next factor

The exact scalar synopsis for `UserID` already proves an upper bound for every omitted user. Native storage can add a bounded occurrence list for the stored leaders. Q17 can then fetch only the `SearchPhrase` values at those row ordinals, aggregate candidate pairs, and accept the answer when the tenth pair count is strictly greater than the omitted-user bound. A failed proof must run the ordinary full aggregate.

This avoids pairwise storage metadata and keeps file growth bounded. The next benchmark must include load wall time, load peak RSS, file size, fetched row count, proof acceptance, and fallback behavior.

## Artifacts

- Small audit: `/home/gopher/gate/two-key-encoded-small-v2`
- One million row audit: `/home/gopher/gate/two-key-encoded-1m`

## Validation

- `cargo test --workspace`
- warnings-as-errors Clippy for `rudb-exec`
- Four-mode ClickBench verification at 1,000, 10,000, and 1,000,000 rows
