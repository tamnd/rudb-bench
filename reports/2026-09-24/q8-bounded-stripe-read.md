# Q8 bounded stripe-page read did not improve the large case

The selected-column reader reads each encoded part separately. This experiment read one selected stripe page when it was at most 512 KiB, then counted parts from their checked spans in that buffer. Larger pages kept the part-read path. The native file and query semantics were unchanged. The candidate was tested but not merged into RuDB.

The table shows medians from 11 alternating fresh-process trials. Each process ran the same Q8 SQL against native files loaded from the same Parquet rows, wrote complete CSV output, and exited. The runner checked every group key and count against DuckDB and required descending count order. Both 1m files and both 10m files were on the same temporary memory filesystem; the smaller files stayed on disk.

| Rows | Merged RuDB wall | Stripe-page wall | DuckDB wall | Merged RuDB CPU | Stripe-page CPU | DuckDB CPU | Stripe-page RSS | DuckDB RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 5.171 ms | 4.487 ms | 94.727 ms | 4.555 ms | 4.266 ms | 122.243 ms | 4.62 MiB | 37.77 MiB |
| 10k | 6.097 ms | 5.859 ms | 92.037 ms | 5.899 ms | 5.656 ms | 112.724 ms | 4.62 MiB | 37.77 MiB |
| 1m | 6.982 ms | 6.342 ms | 104.569 ms | 6.822 ms | 5.892 ms | 153.542 ms | 4.75 MiB | 41.03 MiB |
| 10m | 16.463 ms | 16.966 ms | 98.053 ms | 16.248 ms | 16.611 ms | 194.931 ms | 5.25 MiB | 60.56 MiB |

The 10m result was repeated for 51 alternating trials. The merged part reader used 18.263 ms wall, 17.893 ms CPU, and 5.25 MiB peak RSS; the bounded stripe-page candidate used 18.060 ms, 17.722 ms, and 5.25 MiB; DuckDB used 114.858 ms, 240.415 ms, and 60.19 MiB. The 0.203 ms wall difference between RuDB paths is too small to justify another I/O strategy. The candidate was 6.36x faster than DuckDB in wall time and used 11.46x less peak RSS in that run. The 10x wall-time target remains open.

This quieter run also shows why CPU and wall time must be measured separately. DuckDB used about 13.6x as much CPU as the stripe-page candidate at 10m, but its parallel work yielded a wall-time ratio of only 6.36x. The next useful target is runtime decoding and counter updates rather than the number of selected-page reads.

The [runner](../../scripts/q8-fresh-process.py) and [raw samples](q8-stripe-read/) reproduce the comparison. The merged RuDB binary had SHA-256 `9d35ab0f541cd040301d3f1b9ed8e8bf9b84de00b2a034c87070eff01a0d9b3d`. The candidate was built from RuDB `97c113ef` and had SHA-256 `cb7822b4bea7f135d555527074308346bb042482a2aa3b185381de060e89440e`. DuckDB v2.0.0-dev84237 had SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The 1m source has 999,975 rows; the 10m source has 10,000,000.
