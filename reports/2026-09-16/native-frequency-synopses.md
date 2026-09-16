# Native frequency synopses

The implementation is [tamnd/rudb#719](https://github.com/tamnd/rudb/pull/719). Native snapshots now store an exact leading frequency list and an upper bound for every omitted value. The engine uses this metadata for count-descending grouped TopN only when the requested boundary is strictly above that bound. Otherwise it keeps the ordinary scan and aggregate path.

Numeric columns use a bounded 32,768-entry Misra-Gries candidate pass. A second pass computes exact candidate counts only when the lower bound can certify a useful TopN result. String columns reuse exact global dictionary counts. Each column stores at most 512 leaders, and old version 7 directories without the optional extension remain readable.

## Query results

Q16 groups by `UserID`. Q36 groups by deterministic expressions derived from `ClientIP`, which the planner reduces to one independent key before reading the synopsis.

| Query at 1M rows | DuckDB native | rudb native before | rudb native after | After vs DuckDB |
| --- | ---: | ---: | ---: | ---: |
| Q16 | 12.000 ms | 19.920 ms | 0.317 ms | 37.85x faster |
| Q36 | 12.000 ms | 24.605 ms | 0.395 ms | 30.38x faster |

## Complete suite

Each suite value is the sum of 43 per-query medians. Peak RSS is the maximum measured fresh-process RSS among those queries. All deterministic verification runs matched.

| Rows | Engine and format | Suite time | Peak RSS | Completed |
| ---: | --- | ---: | ---: | ---: |
| 1,000 | DuckDB native | 124.000 ms | 57.50 MiB | 43/43 |
| 1,000 | rudb native | 15.093 ms | 8.95 MiB | 43/43 |
| 1,000 | DuckDB Parquet | 149.000 ms | 57.30 MiB | 43/43 |
| 1,000 | rudb Parquet | 25.068 ms | 8.52 MiB | 43/43 |
| 10,000 | DuckDB native | 145.000 ms | 57.44 MiB | 43/43 |
| 10,000 | rudb native | 28.822 ms | 10.62 MiB | 43/43 |
| 10,000 | DuckDB Parquet | 199.000 ms | 60.18 MiB | 43/43 |
| 10,000 | rudb Parquet | 64.193 ms | 11.91 MiB | 43/43 |
| 1,000,000 | DuckDB native | 557.000 ms | 313.77 MiB | 43/43 |
| 1,000,000 | rudb native | 486.800 ms | 133.18 MiB | 43/43 |
| 1,000,000 | DuckDB Parquet | 1590.000 ms | 356.24 MiB | 43/43 |
| 1,000,000 | rudb Parquet | 783.943 ms | 158.02 MiB | 43/43 |

At 1 million rows, rudb native is 1.14x faster than DuckDB native and uses 2.36x less peak memory. The complete suite has not reached the 10x speed or memory target.

## Load and file cost

| Native load at 1M rows | Wall time | Peak RSS | File write bytes |
| --- | ---: | ---: | ---: |
| rudb before | 2.508 s | 317.44 MiB | 528,625,664 |
| rudb after | 4.662 s | 352.16 MiB | 529,059,840 |
| DuckDB | 3.224 s | 1995.08 MiB | n/a |

The metadata increases native write volume by about 0.08 percent. The current numeric construction makes rudb load 1.45x slower than DuckDB, while rudb uses 5.66x less peak load memory. This load regression is the main cost of the change and should be removed by a later writer change that avoids rescanning viable numeric candidates.

## Remaining limit

Q17 groups by `(UserID, SearchPhrase)` and still takes 31.486 ms in rudb native against 19.000 ms in DuckDB native. Independent scalar frequencies do not determine joint frequencies. Storing every pair would make metadata grow quadratically, so the next engine path should aggregate only rows whose anchor value is in a certified scalar leader set, then accept the result only when its boundary proves that omitted anchors cannot win.

## Artifacts

- Small audit: `/home/gopher/gate/native-frequency-selective-small`
- One million row audit: `/home/gopher/gate/native-frequency-selective-1m`

## Validation

- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy -p rudb-native -p rudb-catalog -p rudb-exec --all-targets -- -D warnings`
- Four-mode ClickBench verification at 1,000, 10,000, and 1,000,000 rows
