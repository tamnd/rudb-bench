# Global string codes and radix count

The native format prototype assigns one stable `u32` code to each distinct string in a column.
Stripe pages store those codes. The dictionary payload stays in the file and is read in 64 KiB blocks only when a query needs string bytes. Grouped `count(*)` over one varying stable string key uses the code directly, exchanges codes into four radix owners, and counts with ordinary integers.
Constant grouping columns are reconstructed beside the varying key.

The main costs were architectural:

1. Every stripe previously built and merged a high cardinality string hash table.
2. Loading a global dictionary eagerly raised startup time and resident memory.
3. Loading individual strings caused one `pread` per distinct string in broad predicates.
4. `LIKE` did not recognize storage backed dictionary values and fell back to row value creation.
5. Small native scans and their downstream buffered pipelines launched more workers than the work
   could amortize.

The current reader keeps the offset index resident, caches payload blocks, and lets byte oriented kernels read external values without constructing `Value` objects. The scheduler caps small native and buffered pipelines by their row count.

## Complete 100k audit

Every query ran in a fresh process. The first run was retained separately and five following runs formed each hot median. All 43 queries completed in all four modes. The deterministic verifier matched every query that needed a stable tie breaker.

| Engine | Complete | Query median sum | Process wall sum | CPU sum | Peak RSS |
| --- | ---: | ---: | ---: | ---: | ---: |
| DuckDB native | 43 / 43 | 0.324 s | 1.212 s | 1.531 s | 85.91 MiB |
| rudb native | 43 / 43 | 0.204 s | 0.269 s | 0.426 s | 22.94 MiB |
| DuckDB Parquet | 43 / 43 | 0.374 s | 1.370 s | 2.146 s | 116.81 MiB |
| rudb Parquet | 43 / 43 | 0.185 s | 0.248 s | 0.716 s | 42.70 MiB |

| Native load | Wall | CPU | Peak RSS | File size |
| --- | ---: | ---: | ---: | ---: |
| DuckDB | 0.552 s | 0.740 s | 384.36 MiB | 30.16 MB |
| rudb | 0.278 s | 0.267 s | 56.64 MiB | 55.09 MB |

## Queries affected by the new representation

| Query | DuckDB native | rudb native | Speedup |
| --- | ---: | ---: | ---: |
| Q21, URL contains `google` | 8.00 ms | 4.05 ms | 1.98x |
| Q22, filtered grouped strings | 10.00 ms | 5.33 ms | 1.88x |
| Q23, two string predicates and grouped strings | 14.00 ms | 5.76 ms | 2.43x |
| Q24, filtered ordered rows | 27.00 ms | 5.86 ms | 4.61x |
| Q34, URL count TopN | 16.00 ms | 1.56 ms | 10.25x |
| Q35, constant plus URL count TopN | 19.00 ms | 1.52 ms | 12.48x |

Before block caching and the external byte kernel, rudb native took 31.88 ms, 35.05 ms,
36.68 ms, and 33.76 ms on Q21 through Q24. The final times are 81 to 84 percent lower.

## Limits still open

At one million rows Q34 is about 5 to 7 ms with four useful workers, compared with 26 ms for
DuckDB. The file has roughly one 1,024 row stripe per input chunk, so the query still performs about
981 small code page reads. Larger physical pages with vector sized logical chunks are the next storage change.

The 100k native suite is 1.59x faster than DuckDB and uses 3.74x less peak query memory. Native load uses 6.78x less peak memory. The whole suite has not reached the 10x target.

The compact arena writer and streamed dictionary finalization reduce the later 1m load peak from
431 MiB to 295 MiB. DuckDB uses 1,932.7 MiB for the same load, so the current ratio is 6.5x. The remaining dictionary membership indexes and payload arenas stay resident together until commit.

After rebasing to main and limiting both scan and buffered stages to four workers at this scale, the version 7 1m measurements are:

| Query | DuckDB native | rudb native | Speedup |
| --- | ---: | ---: | ---: |
| Q21 | 15.0 ms | 20.6 ms | 0.73x |
| Q34 | 26.0 ms | 5.41 ms | 4.81x |
| Q35 | 26.0 ms | 5.41 ms | 4.81x |

Q34 peak RSS is 39.7 MiB versus DuckDB's 301.0 MiB. The remaining query cost is the 1,024 row physical stripe layout. A 1m scan performs about 981 small code page reads before radix counting.

Massif on the compact writer showed that finalization duplicated the complete dictionary payload.
Writing the authenticated index followed by the existing arena reduced 1m load peak RSS from
339 MiB to 295 MiB. The result is 6.5x below DuckDB's 1,932.7 MiB load peak.

The lazy dictionary format needs independently authenticated indexes and payload blocks. Version 7 adds an index checksum and one checksum per 64 KiB payload block. Error propagation from a lazy block read through every vector consumer remains part of the storage contract and must be complete before this work is merged.
