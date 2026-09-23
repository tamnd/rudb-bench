# Q8 counts sparse integer parts from encoded rows

The Q8 runtime counter previously expanded every sparse `AdvEngineID` part into a row vector. The new reader counts the dominant value and final exception values from the encoded rows as the query runs. Constant and run-length parts also avoid row expansion. Nullable pages use the ordinary reader. The file still stores row values and reusable column statistics, not a grouped query answer. The writer and native file bytes are unchanged.

The table shows medians from 11 alternating fresh-process trials at each size. Each process opened a native file, ran the same Q8 SQL, wrote its complete CSV result, and exited. The runner compared every key and count with DuckDB and checked descending count order. SQL does not specify the order of tied groups, so either tie order is accepted. Both native files were loaded from the same Parquet rows.

| Rows | RuDB before wall | RuDB encoded fold wall | DuckDB wall | DuckDB / fold wall | RuDB before CPU | Fold CPU | DuckDB CPU | Fold RSS | DuckDB RSS | DuckDB / fold RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 41.951 ms | 20.291 ms | 307.069 ms | 15.13x | 11.856 ms | 9.956 ms | 164.355 ms | 5.25 MiB | 38.14 MiB | 7.27x |
| 10k | 104.995 ms | 110.983 ms | 421.730 ms | 3.80x | 31.929 ms | 33.887 ms | 170.681 ms | 5.62 MiB | 37.96 MiB | 6.75x |
| 1m | 78.841 ms | 62.191 ms | 424.008 ms | 6.82x | 40.220 ms | 20.634 ms | 197.753 ms | 7.00 MiB | 41.15 MiB | 5.88x |
| 10m | 171.534 ms | 72.525 ms | 451.919 ms | 6.23x | 67.243 ms | 26.926 ms | 333.439 ms | 8.00 MiB | 60.44 MiB | 7.55x |

The 10k result varies with host load. A longer 51-trial 10k run measured 104.058 ms wall and 56.708 ms CPU for the earlier RuDB path, 100.554 ms and 39.858 ms for the encoded fold, and 372.069 ms and 206.699 ms for DuckDB. Both RuDB paths used 5.62 MiB peak RSS; DuckDB used 37.91 MiB. This gives no reliable 10k wall-time improvement.

A 51-trial 10m run measured 136.249 ms wall, 69.054 ms CPU, and 8.00 MiB peak RSS for the earlier RuDB path; 76.345 ms, 26.658 ms, and 8.00 MiB for the encoded fold; and 518.781 ms, 311.569 ms, and 60.31 MiB for DuckDB. The fold is 1.78x faster in wall time and 2.59x faster in CPU time than the earlier RuDB path. Against DuckDB it is 6.80x faster in wall time and uses 7.54x less peak RSS. The host was shared and contended, so these are paired observations rather than quiet-host latency estimates.

The 10x wall-time and peak-memory targets remain unmet. The remaining native memory cost includes opening the table-wide directory even though Q8 reads one column. A column-only directory reader should retain just that column's page and part spans. The result still needs a fresh-process measurement that includes file open and complete output. The encoded fold's 10m CPU time is already more than 10x below DuckDB's, so file open, process startup, and scheduling now matter more to wall time.

The [runner](../../scripts/q8-fresh-process.py) and [raw samples](q8-sparse-fold/) reproduce the process comparison. The earlier RuDB binary had SHA-256 `e3d1d2d9ff7e17f35e3219d2ace9a415cef6d544e39f5132e3c1f399a0167091`. The encoded-fold binary was built from RuDB `d098b53b` and had SHA-256 `e07cd066d99b7c5fcafe9e6518c5bae309b813f934f1f5138de56a38b8884738`. DuckDB v2.0.0-dev84237 had SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The 1m source has 999,975 rows; the 10m source has 10,000,000.
