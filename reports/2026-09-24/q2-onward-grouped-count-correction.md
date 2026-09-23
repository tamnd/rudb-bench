# Q2 onward: grouped counts must read rows

The earlier Q8 runs answered `GROUP BY AdvEngineID` from a complete native frequency synopsis. That synopsis contained each key and its exact count. It is a reusable, bounded column statistic, but for Q8 it was already the grouped result. Those runs measured metadata lookup and formatting, not query-time aggregation. The Q8 speed and memory claims in the earlier reports should be read in that context.

The engine now uses column statistics for scalar Q2 through Q7 results and reads rows to compute a grouped count. It no longer formats Q8 directly from the small native catalog, returns grouped rows from the one-shot native API, or substitutes a complete frequency list for a single-column aggregate in the general executor. The native file format and load path are unchanged. Frequency synopses remain available for selectivity estimates and scalar row counts; they are not grouped output.

| Query | Stored facts used | Query-time work |
| --- | --- | --- |
| Q2 | Table cardinality and numeric zero/null frequencies | Count qualifying rows from column statistics |
| Q3 | Column sums, non-null counts, table cardinality | Combine scalar statistics and divide for average |
| Q4 | Column sum and non-null count | Divide for average |
| Q5 and Q6 | Column distinct count | Return scalar cardinality |
| Q7 | Column minimum and maximum | Return scalar bounds |
| Q8 | Encoded column values | Filter, group, count, and sort when SQL runs |

The table is the median of 11 alternating fresh-process pairs for the same Q8 SQL and the same Parquet-derived native tables. The wait4 helper measures startup, file open, query, output, and exit. Every result had the same complete key/count set as DuckDB and was sorted by descending count. SQL leaves the order of tied counts unspecified, so the runner accepts either tie order. The host has eight CPUs and had a load average of about 18 to 29 during the run. These wall times describe that contention, not quiet-host latency. The native writer did not change; load time and file bytes are the same as before this correction.

| Rows | RuDB wall | DuckDB wall | DuckDB / RuDB wall | RuDB CPU | DuckDB CPU | RuDB peak RSS | DuckDB peak RSS | DuckDB / RuDB RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 44.383 ms | 305.489 ms | 6.88x | 15.596 ms | 154.505 ms | 10.25 MiB | 38.02 MiB | 3.71x |
| 10k | 92.839 ms | 372.716 ms | 4.01x | 42.062 ms | 166.390 ms | 11.12 MiB | 38.02 MiB | 3.42x |
| 1m | 106.121 ms | 422.368 ms | 3.98x | 44.163 ms | 211.740 ms | 14.88 MiB | 41.40 MiB | 2.78x |
| 10m | 196.189 ms | 517.343 ms | 2.64x | 118.145 ms | 339.507 ms | 20.25 MiB | 60.43 MiB | 2.98x |

Q8 no longer meets either 10x target. The 10m profile puts most sampled CPU in comparison, selection, and native narrowing, while the engine metrics record about 15 ms in aggregation and about 200 ms for the scan pipeline. This points to filtering and decoding the column, not the eight-group count, as the next bottleneck. A row-level column scan that fuses filtering and counting can address it without storing group counts.

The [runner](../../scripts/q8-fresh-process.py) and [raw records](q8-runtime/) reproduce the process comparison. The source was based on RuDB `3b5990c3` with the grouped-count correction, built as release binary SHA-256 `7858b0c26becd2e75acca373e07003e6f18ef5f27b922dbb9e217b22186595de`. DuckDB v2.0.0-dev84237 had binary SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The 1m source has 999,975 rows and the 10m source has 10,000,000 rows.
