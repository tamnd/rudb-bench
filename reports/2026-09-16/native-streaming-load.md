# Native streaming load

## Change

A file-backed initial `INSERT ... SELECT` now connects the producing pipeline directly to the native writer. The root no longer queues the complete result, the catalog no longer holds a second in-memory copy, and `CHECKPOINT` no longer walks and encodes that copy after the insert.

The sink writes chunks as they arrive. Each chunk carries its source morsel and position into the native directory, so physical completion order does not change scan order. The current writer serializes page encoding and file writes. Running several Parquet decoder instances in front of that lock raised memory without reducing wall time, so this sink deliberately uses one pipeline instance until page encoding is moved outside the file lock.

## One million ClickBench rows

Both engines ran the same SQL against the same Parquet file:

```sql
CREATE TABLE hits (...);
INSERT INTO hits
SELECT <projection>
FROM read_parquet('<same file>', binary_as_string = true);
CHECKPOINT;
```

| Engine | Load wall | Load CPU | Load peak RSS | Native file |
| --- | ---: | ---: | ---: | ---: |
| DuckDB native | 3.529 s | 6.901 s | 1,953.1 MiB | 281.8 MB |
| rudb native, previous | 2.603 s | 4.059 s | 926.0 MiB | 557.1 MB |
| rudb native, streaming parallel source | 2.068 s | 2.748 s | 209.4 MiB | 557.1 MB |
| rudb native, streaming single writer | **2.620 s** | **2.548 s** | **33.6 MiB** | 557.1 MB |

The single-writer streaming path uses 58.1x less peak RSS than DuckDB and loads 25.8 percent faster. It uses 27.6x less peak RSS than the previous rudb checkpoint path.

The parallel streaming run also executed all 43 ClickBench queries through DuckDB native, rudb native, DuckDB Parquet, and rudb Parquet. Every result matched directly, contained the same rows in a different order, or matched under the separate deterministic tie diagnostic. Query 29 matched directly on all four paths. Native query performance and native file size were unchanged because this change only replaces the load path.

## Next bottleneck

The file is still 1.98x the size of DuckDB's native file. Storage pages are still tied to 1,024-row execution vectors, so dictionaries and other encodings see too little data and page metadata repeats too often. The next format change must build larger column pages from several execution vectors, encode those pages outside the file lock, and expose 1,024-row views when scanning. That will make page compression parallel without returning to an unbounded load buffer.
