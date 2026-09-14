# Why rudb falls behind DuckDB at one million rows

The large-data gap is mainly an execution architecture gap. It is not a general quadratic-growth bug.

The audit ran all 43 ClickBench queries with one first execution and five hot repetitions in fresh processes. DuckDB native used a loaded table. DuckDB Parquet and rudb scanned the same Snappy Parquet file. Query time came from each CLI timer. Process wall time, CPU time, and peak RSS came from Linux `wait4` for the exact child process.

## Scaling evidence

| Rows | DuckDB native query sum | DuckDB Parquet query sum | rudb query sum | DuckDB native peak RSS | DuckDB Parquet peak RSS | rudb peak RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 0.124 s | 0.153 s | 0.024 s | 53.19 MiB | 52.86 MiB | 6.33 MiB |
| 10,000 | 0.150 s | 0.181 s | 0.087 s | 53.11 MiB | 55.27 MiB | 15.91 MiB |
| 100,000 | 0.321 s | 0.453 s | 0.708 s | 82.82 MiB | 135.84 MiB | 102.81 MiB |
| 1,000,000 before the architectural fixes | 0.565 s | 0.846 s | 6.143 s | 307.37 MiB | 639.39 MiB | 170.77 MiB |
| 1,000,000 after the architectural fixes | 0.538 s | 0.808 s | 5.246 s | 312.72 MiB | 643.26 MiB | 170.86 MiB |

rudb grows roughly eight to ten times when the input grows ten times from 100,000 to 1 million rows. That is the shape of linear scan and hash work. DuckDB grows much less over the same interval because its fixed process cost is still visible, its native table avoids Parquet decoding, and its execution uses several cores.

## Primary root cause: the execution engine is serial

The benchmark host has 32 logical CPUs. DuckDB's 1 million row run used 3.861 seconds of CPU in 1.498 seconds of process wall time. rudb used 5.288 seconds of CPU in 5.315 seconds of process wall time. DuckDB overlaps work across cores. rudb consumes almost exactly one CPU at a time.

This follows directly from the current code:

- The database drains a pull-based operator tree on the caller's thread.
- `rudb-pipeline` provides `run_serial`; the parallel F4 driver described in the source comments is not implemented.
- A Parquet `FileScan` creates one morsel for the entire file and protects one sequential reader with a mutex.
- Grouped aggregation rejects a second pipeline instance because aggregate-state merging is not implemented.
- Query metrics record one execution thread even when the session setting requests more.

Adding worker threads around the current file scan would only make workers wait on the reader mutex. Useful parallelism requires independent Parquet row-group readers, a pipeline scheduler that runs several instances, and mergeable aggregate states.

## Secondary root cause: too much work per row

DuckDB native finishes the 43 query timers in 0.538 seconds. rudb still needs 5.246 seconds after removing two large sources of wasted work. Parallelism alone cannot make rudb ten times faster than DuckDB because rudb currently spends more total CPU than DuckDB while DuckDB already runs that CPU concurrently.

The remaining per-row costs are concentrated in these areas:

| Query | DuckDB native | DuckDB Parquet | rudb | rudb peak RSS | Dominant shape |
| --- | ---: | ---: | ---: | ---: | --- |
| q23 | 0.026 s | 0.036 s | 0.436 s | 57.60 MiB | Five-column Parquet scan, two text predicates, high-cardinality aggregate |
| q34 | 0.029 s | 0.040 s | 0.307 s | 170.86 MiB | One million URL group probes, then `LIMIT 10` |
| q35 | 0.031 s | 0.041 s | 0.319 s | 170.86 MiB | Same high-cardinality URL aggregation with a constant group key |
| q29 | 0.025 s | 0.031 s | 0.278 s | 77.16 MiB | Regular-expression host extraction and grouping |
| q40 | 0.007 s | 0.026 s | 0.243 s | 47.74 MiB | Filter, five-column grouping, and `OFFSET 1000` TopN |

The grouped path still hashes and stores values row by row. High-cardinality text grouping retains almost one key per input row. The sort specification already calls for normalized byte keys, but TopN and grouping still carry general `Value` representations in important paths. The Parquet reader also decodes requested columns eagerly into vectors before later operators can eliminate rows.

## Confirmed architecture mistakes already fixed

The first benchmark compared DuckDB native storage with rudb decoding Parquet for every query. The audit now reports DuckDB native and DuckDB Parquet separately. This makes storage-format cost visible instead of attributing it to the SQL engine.

Small TopN bounds used to materialize a heap-allocated payload row for every candidate before checking whether it could enter `LIMIT 10`. Comparing the key first reduced the 1 million row suite from 6.143 seconds to 5.696 seconds. The largest affected queries improved by 17% to 31%.

Late materialization initially worked only when every view projection was a bare file column. The ClickBench view converts dates and timestamps, so the optimizer missed the measured plan and q24 still decoded all 105 columns. Replaying computed projections after sparse fetch reduced q24 from 1.029 seconds to 0.552 seconds and reduced its peak RSS from 167.95 MiB to 42.07 MiB.

## Work required to reach the target

The next changes need to follow dependency order because each one enables the next:

1. Make a Parquet row group an independently readable morsel. Each worker needs its own reader state, and output order must be restored only for queries that promise an order.
2. Replace the pull-tree driver with a pipeline graph runner. It must honor pipeline dependencies, run several local stream and sink states, and report actual worker counts and blocked time.
3. Add aggregate-state combine operations. Start with count, integer sum, min, max, and average, which cover most ClickBench aggregates, then add distinct-state merging.
4. Replace general text group keys with normalized keys stored in compact arenas. Store one hash and one offset per group instead of a general recursive value.
5. Push eligible predicates into Parquet row-group and page pruning, then evaluate supported predicates on encoded vectors when decoding can be avoided.
6. Repeat the 1,000, 10,000, 100,000, and 1 million row audits after each architecture change. A result only counts when all 43 queries complete and deterministic correctness checks pass.

At the current 1 million row result, rudb is 9.8 times slower than DuckDB native and uses 1.8 times less peak memory. Reaching the requested target means reducing the query sum below 0.0538 seconds and peak RSS below 31.27 MiB on this host. The serial scheduler, sequential Parquet reader, and high-cardinality aggregate representation prevent both thresholds today.
