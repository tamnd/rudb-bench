# Q8 folds encoded counts into the query counter

The selected-column reader previously built a count map for each encoded part, merged those maps into a table-wide count map, then copied the result into the query's bounded counter. The new path visits `(value, count)` pairs from encoded row data and adds them directly to the query counter. Sparse chunks with sorted exception positions avoid a position map; repeated or unordered positions keep the decoder's last-write-wins rule. Nullable and other pages visit valid decoded rows. The native writer and file bytes are unchanged, and no grouped query answer is stored.

The table shows medians from 11 alternating fresh-process trials at each size. Each process opened a native file, ran the same Q8 SQL, wrote complete CSV output, and exited. The runner checked every group key and count against DuckDB and required descending count order. SQL does not specify tie order. Both native files came from the same Parquet rows. The 1m and 10m file pairs were on the same temporary memory filesystem; the smaller pairs were on disk.

| Rows | Earlier RuDB wall | Direct-fold wall | DuckDB wall | DuckDB / fold wall | Earlier RuDB CPU | Fold CPU | DuckDB CPU | Fold RSS | DuckDB RSS | DuckDB / fold RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 4.506 ms | 4.608 ms | 145.043 ms | 31.48x | 4.308 ms | 4.390 ms | 159.326 ms | 4.50 MiB | 38.14 MiB | 8.48x |
| 10k | 6.951 ms | 7.274 ms | 132.012 ms | 18.15x | 6.759 ms | 6.660 ms | 145.055 ms | 4.50 MiB | 38.14 MiB | 8.48x |
| 1m | 6.975 ms | 7.525 ms | 140.701 ms | 18.70x | 6.684 ms | 7.217 ms | 157.525 ms | 4.50 MiB | 41.41 MiB | 9.20x |
| 10m | 18.927 ms | 15.833 ms | 163.890 ms | 10.35x | 18.633 ms | 15.572 ms | 226.851 ms | 5.00 MiB | 60.44 MiB | 12.09x |

The 11-trial 10m result crossed both 10x targets, but a longer check did not sustain the wall-time result. In 51 alternating 10m trials, the earlier RuDB path used 18.482 ms wall, 18.190 ms CPU, and 5.25 MiB peak RSS; the direct fold used 15.981 ms, 15.222 ms, and 5.00 MiB; DuckDB used 127.323 ms, 216.112 ms, and 60.19 MiB. The direct fold improves RuDB's 10m wall time by 1.16x and CPU time by 1.19x. Against DuckDB it is 7.97x faster in wall time and uses 12.04x less peak RSS. The 10x large-case wall-time target remains open. At smaller sizes, the direct fold's wall time was flat or slightly worse than the earlier RuDB path, while its peak RSS fell by 0.12 to 0.25 MiB.

A 30-process `perf` profile of the new 10m path assigned 9.74% of sampled cycles to vector unpacking, 8.22% to the native integer fold itself, and 7.69% to kernel memory copying. The path still decodes some nullable or non-cascade parts into vectors. Eliminating unnecessary row expansion there is the next measured target. The host was shared, and paired medians do not establish quiet-host latency.

The [runner](../../scripts/q8-fresh-process.py) and [raw samples](q8-direct-fold/) reproduce the comparison. The earlier RuDB binary had SHA-256 `9d35ab0f541cd040301d3f1b9ed8e8bf9b84de00b2a034c87070eff01a0d9b3d`. The direct-fold binary was built from RuDB `3da5774b` on main `002d3018` and had SHA-256 `a66725bc7723064b2d95254730487e6625a9c5d086a3581e399c248dad1aa6e5`. DuckDB v2.0.0-dev84237 had SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The 1m source has 999,975 rows; the 10m source has 10,000,000.
