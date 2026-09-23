# Q8 counts packed integer pages in bounded blocks

The direct encoded fold still flattened FOR+BITPACK pages before counting them. In the 10m `AdvEngineID` column, 37 such pages hold 303,104 rows. The new reader uses the existing packed-vector constructor to validate each page, then unpacks at most 64 codes into a stack buffer and visits valid values. The other 9,696,896 rows keep their existing encoded-fold path. The native writer and file bytes are unchanged, and no grouped answer is stored.

The table shows medians from 11 alternating fresh-process trials per size. Each engine opened a native file loaded from the same Parquet rows, ran the same Q8 SQL, wrote its complete CSV result, and exited. The runner checked every group key and count against DuckDB and required descending count order. SQL leaves tied-group order unspecified. Both 1m files and both 10m files were on the same temporary memory filesystem; the smaller files stayed on disk.

| Rows | Earlier RuDB wall | Packed-fold wall | DuckDB wall | DuckDB / fold wall | Earlier RuDB CPU | Fold CPU | DuckDB CPU | Fold RSS | DuckDB RSS | DuckDB / fold RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 6.442 ms | 5.744 ms | 205.661 ms | 35.80x | 4.657 ms | 4.149 ms | 194.847 ms | 4.50 MiB | 37.78 MiB | 8.40x |
| 10k | 27.830 ms | 11.507 ms | 196.917 ms | 17.11x | 7.636 ms | 6.821 ms | 187.354 ms | 4.50 MiB | 38.00 MiB | 8.44x |
| 1m | 11.294 ms | 12.392 ms | 202.935 ms | 16.38x | 6.481 ms | 6.457 ms | 213.462 ms | 4.50 MiB | 41.03 MiB | 9.12x |
| 10m | 42.889 ms | 30.114 ms | 280.987 ms | 9.33x | 23.861 ms | 14.452 ms | 298.580 ms | 4.75 MiB | 60.18 MiB | 12.67x |

The 10m comparison was repeated for 51 alternating trials because the shared host's wall times moved substantially. The earlier RuDB path used 58.112 ms wall, 17.794 ms CPU, and 4.88 MiB peak RSS; the packed fold used 52.325 ms, 15.001 ms, and 4.88 MiB; DuckDB used 475.821 ms, 302.251 ms, and 60.31 MiB. The packed fold improves RuDB's 10m wall time by 1.11x and CPU time by 1.19x. It is 9.09x faster than DuckDB in wall time and uses 12.37x less peak RSS. The 10x large-case wall-time target remains open. At 1m, RuDB CPU was essentially unchanged and wall time was slightly worse, so the large-case improvement should not be projected to every size.

A 30-process `perf` profile of the packed path no longer put full-vector unpacking among the leading symbols. Sampled cycles were led by the query counter and the native integer fold, with 5.17% in kernel memory copying and 2.48% in the 64-code unpack block. Further wall-time work should focus on the remaining row and directory work, not restore full-vector expansion. These paired medians do not establish quiet-host latency.

The [runner](../../scripts/q8-fresh-process.py) and [raw samples](q8-packed-fold/) reproduce the comparison. The earlier RuDB binary had SHA-256 `a66725bc7723064b2d95254730487e6625a9c5d086a3581e399c248dad1aa6e5`. The packed-fold binary was built from RuDB `21314a45` on main `be6e777d` and had SHA-256 `365d0cbbe81a0bec42b03595dd7c9cfb069719e19e7d5177ccfd1f7fb577f0aa`. DuckDB v2.0.0-dev84237 had SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The 1m source has 999,975 rows; the 10m source has 10,000,000.
