# Q8 counts groups while reading native rows

After the [Q8 saved-group correction](q2-onward-grouped-count-correction.md), the regular executor read the `AdvEngineID` column through its filter and aggregate pipelines. This change adds a narrow runtime path for a small-domain numeric column grouped by its nonzero values. It reads the requested native column's rows and counts them as the SQL runs. The file's exact range and distinct-count statistics only choose a bounded counter; neither statistic supplies a group count. Changed filters, grouping, ordering, limits, unsupported types, and high-cardinality columns use the regular SQL path. The native writer and file bytes are unchanged.

The table shows medians from 11 alternating fresh-process trials per size. Each trial ran the corrected RuDB baseline, the column-count candidate, and DuckDB on native files loaded from the same Parquet rows. The wait4 helper measured process startup, file open, query, complete CSV output, and exit. Every output had the same full key/count set as DuckDB and descending counts. SQL does not specify the order of tied groups, so the runner accepts either tie order.

| Rows | Corrected RuDB wall | Column-count RuDB wall | DuckDB wall | DuckDB / candidate wall | Candidate CPU | DuckDB CPU | Corrected RuDB RSS | Candidate RSS | DuckDB RSS | DuckDB / candidate RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 47.633 ms | 42.627 ms | 441.458 ms | 10.36x | 9.951 ms | 199.137 ms | 10.25 MiB | 5.25 MiB | 37.77 MiB | 7.19x |
| 10k | 119.123 ms | 83.262 ms | 434.118 ms | 5.21x | 59.736 ms | 184.985 ms | 11.25 MiB | 5.62 MiB | 38.02 MiB | 6.76x |
| 1m | 85.579 ms | 74.340 ms | 398.237 ms | 5.36x | 22.255 ms | 189.706 ms | 15.38 MiB | 7.00 MiB | 41.20 MiB | 5.89x |
| 10m | 175.559 ms | 94.116 ms | 514.061 ms | 5.46x | 67.621 ms | 295.099 ms | 19.88 MiB | 8.00 MiB | 60.43 MiB | 7.55x |

A longer 51-trial 10m run gave 210.932 ms and 20.00 MiB for corrected RuDB, 191.090 ms and 7.88 MiB for the candidate, and 801.900 ms and 60.50 MiB for DuckDB. The candidate used 68.864 ms CPU, against 116.772 ms for corrected RuDB and 301.910 ms for DuckDB. The eight-CPU shared host was contended; wall time varied enough that the 11-trial wall gain over corrected RuDB did not hold at the same size in the longer run. The lower CPU and RSS were consistent. These runs do not establish quiet-host latency.

The candidate remains below the 10x target against DuckDB. The next bottleneck is the native integer page decoder: most `AdvEngineID` parts use sparse encoding, but the reader expands their dominant zero value into row vectors before counting. Counting sparse exceptions and their dominant run directly from encoded row data could reduce query CPU and allocations without storing an aggregate answer. A column-only directory reader would then address the remaining process RSS from loading table-wide metadata.

The [runner](../../scripts/q8-fresh-process.py) and [raw samples](q8-fused-row-scan/) reproduce the process comparison. The corrected RuDB binary had SHA-256 `7858b0c26becd2e75acca373e07003e6f18ef5f27b922dbb9e217b22186595de`. The candidate was built from RuDB `20c9e961` with the grouped-count correction and column-count change, and had SHA-256 `e3d1d2d9ff7e17f35e3219d2ace9a415cef6d544e39f5132e3c1f399a0167091`. DuckDB v2.0.0-dev84237 had SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The 1m source has 999,975 rows; the 10m source has 10,000,000.
