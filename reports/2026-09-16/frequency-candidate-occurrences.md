# Frequency candidate occurrence lists

The implementation is [tamnd/rudb#728](https://github.com/tamnd/rudb/pull/728). It stores bounded row ordinals for numeric Misra-Gries candidates during the exact recount that already runs at load time. The `RUDBFQ2` extension delta encodes sorted table-wide ordinals, caps each column at 65,536 rows, and keeps `RUDBFQ1` readable.

## One million row metadata

| Column | Candidate rows | Share of table | Omitted frequency bound |
| --- | ---: | ---: | ---: |
| `ClientIP` | 60,418 | 6.04% | 31 |
| `UserID` | 25,455 | 2.55% | 30 |

Q17's tenth `(UserID, SearchPhrase)` pair occurs 51 times. Since 51 is strictly greater than 30, an aggregate over the stored `UserID` candidate rows can prove that no omitted user pair enters the result. The engine consumer is separate work and is not included in these query numbers.

## Load comparison

| Native load at 1M rows | Wall time | Peak RSS | Database bytes |
| --- | ---: | ---: | ---: |
| DuckDB | 3.323 s | 2064.07 MiB | 281,030,656 |
| rudb | 3.029 s | 434.89 MiB | 531,903,998 |

Rudb loads 1.10x faster than DuckDB and uses 4.75x less peak memory. The occurrence lists reuse the numeric recount, so they add no page pass.

## Complete suite baseline

| Engine and format | Suite time | Peak RSS | Completed |
| --- | ---: | ---: | ---: |
| DuckDB native | 538.000 ms | 310.98 MiB | 43/43 |
| rudb native | 479.038 ms | 137.73 MiB | 43/43 |
| DuckDB Parquet | 1622.000 ms | 350.62 MiB | 43/43 |
| rudb Parquet | 796.273 ms | 156.66 MiB | 43/43 |

## Artifacts

- Small audit: `/home/gopher/gate/frequency-occurrences-small`
- One million row audit: `/home/gopher/gate/frequency-occurrences-1m`

## Validation

- `cargo test --workspace`
- warnings-as-errors Clippy for `rudb-native` and `rudb-catalog`
- Four-mode ClickBench audits at 1,000, 10,000, and 1,000,000 rows
