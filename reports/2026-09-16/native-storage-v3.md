# Native storage v3 baseline

## Result

This change adds the first complete rudb native storage path and an apples to apples ClickBench audit. The audit loads the same Parquet projection with the same SQL into DuckDB and rudb, then runs all 43 queries through four paths:

1. DuckDB native
2. rudb native
3. DuckDB Parquet
4. rudb Parquet

The native reader is already useful. At one million rows, rudb loads 25 percent faster than DuckDB and uses 52 percent less peak memory. Across the 43 queries, rudb native uses 42 percent less peak memory and 35 percent less process wall time than DuckDB native. The sum of engine timers is still 43 percent slower, and the rudb file is 98 percent larger. The 10x target has not been reached.

## Measurement contract

Both native engines run this exact statement, with the same schema and projection substituted in place:

```sql
CREATE TABLE hits (...);
INSERT INTO hits
SELECT <projection>
FROM read_parquet('<same file>', binary_as_string = true);
CHECKPOINT;
```

Each query run starts a fresh process. Five measured hot repetitions follow one first run. The query timer includes result rendering. Process wall time, CPU time, peak RSS, I/O, faults, context switches, commands, outputs, and exit status come from a Linux `wait4` wrapper for the exact child process. Peak RSS is an absolute process high water mark.

The original ClickBench SQL can return tied rows in a different order. The audit first compares the original output. It reports the same rows in a different order separately. If a tied `GROUP BY` and `LIMIT` result selects different rows, an untimed diagnostic adds deterministic tie breakers. The diagnostic never replaces the timed SQL.

## One million rows

| Engine | Load wall | Load CPU | Load peak RSS | Native file |
| --- | ---: | ---: | ---: | ---: |
| DuckDB native | 3.471 s | 6.776 s | 1,931 MiB | 281.8 MB |
| rudb native | 2.603 s | 4.059 s | 926 MiB | 557.1 MB |
| rudb relative to DuckDB | **25.0% faster** | **40.1% less** | **52.0% less** | 97.7% larger |

| Engine | Complete | Query median sum | Process wall median sum | CPU median sum | Peak RSS |
| --- | ---: | ---: | ---: | ---: | ---: |
| DuckDB native | 43 / 43 | 0.546 s | 1.465 s | 3.836 s | 309.6 MiB |
| rudb native | 43 / 43 | 0.782 s | 0.953 s | 6.551 s | 181.0 MiB |
| DuckDB Parquet | 43 / 43 | 1.645 s | 3.295 s | 8.124 s | 353.8 MiB |
| rudb Parquet | 43 / 43 | 0.938 s | 1.188 s | 5.186 s | 187.9 MiB |

All 43 original results either match, contain the same rows in a different order, or match after the separate deterministic tie diagnostic. Query 29 matches on all four paths.

## Size ladder

These points use the same harness and establish the small input behavior. The 1,000 and 10,000 row measurements predate integer bit packing. The 100,000 and one million row measurements use the packed format. Only the one million row load includes the corrected single checkpoint write.

| Rows | Engine | Load wall | Load peak RSS | Query median sum | Query peak RSS |
| ---: | --- | ---: | ---: | ---: | ---: |
| 1,000 | DuckDB native | 0.045 s | 52.3 MiB | 0.122 s | 53.5 MiB |
| 1,000 | rudb native | 0.011 s | 7.6 MiB | 0.016 s | 6.8 MiB |
| 10,000 | DuckDB native | 0.109 s | 90.8 MiB | 0.146 s | 53.0 MiB |
| 10,000 | rudb native | 0.052 s | 23.9 MiB | 0.057 s | 13.9 MiB |
| 100,000 | DuckDB native | 0.511 s | 371.8 MiB | 0.327 s | 82.4 MiB |
| 100,000 | rudb native | 0.508 s | 160.1 MiB | 0.191 s | 37.8 MiB |

## Root cause

The current checkpoint first materializes the complete `INSERT ... SELECT` result in the in-memory table. It then encodes that table to a second set of buffers and writes it. This explains the roughly linear load RSS and why a ten million row load would require far more memory than the final file. Streaming the pipeline into a storage sink is the main memory change still required.

The physical page is currently one 1,024-row execution vector. This is too small for storage decisions. It repeats page metadata and prevents a string encoder from learning across a useful sample. An FSST experiment at this granularity reduced the one million row file from 557 MB to 519 MB, but increased rudb load wall time from 2.60 seconds to 20.76 seconds. Training one symbol table per vector was the cost. That implementation was discarded.

The next storage layer must separate execution vectors from storage pages. A storage page should hold several execution vectors, train compression once, and let the reader expose 1,024-row views without copying or reopening the page. This also enables one checksum per larger page, shared dictionaries, useful min and max metadata, and page pruning. The loader can keep one bounded page builder per column and publish completed stripes directly to the file.

The format follows conclusions that recur in recent analytical storage work: no single encoding wins for every workload, structural encodings should remain available to execution, and late materialization needs page metadata that can decide whether decoding is useful. The design references the [2024 analytical format study](https://arxiv.org/abs/2411.14331), the [2025 Lance structural encoding paper](https://arxiv.org/abs/2504.15247), the [Parquet late materialization implementation](https://arrow.apache.org/blog/2025/12/11/parquet-late-materialization-deep-dive/), and [DuckDB storage internals](https://duckdb.org/docs/current/internals/storage) as evidence. The rudb layout and commit protocol are designed independently around its vector engine and benchmark results.

## Implemented format

The first format version provides:

- one native file with a versioned header and committed directory
- independent projected column page reads through one shared file handle
- page and directory checksums with bounds validation
- immutable row stripes
- all-valid, all-null, and bitmap validity
- adaptive string dictionaries
- frame-of-reference bit packing retained as packed vectors in execution
- atomic snapshot publication through a temporary file and rename
- native catalog scans through the existing parallel scan operator

The header reserves two directory slots, but this milestone publishes a fresh snapshot file and uses the first slot. Incremental generations, page statistics, multiple tables, schema evolution, and direct pipeline sinks remain follow-up work.
