# Q2 onward: native statistics and fast CSV paths

The native writer must store reusable facts about columns and rows, not a saved answer for a ClickBench statement. I checked the Q2 through Q8 read paths and the writer that produces their metadata. [RuDB #1627](https://github.com/tamnd/rudb/pull/1627) removes the remaining literal ClickBench statement and column-name checks from the Q3 and Q8 fast CSV paths.

| Query | Native facts read | Work done when SQL runs |
| --- | --- | --- |
| Q2 | Complete numeric value frequencies, including nulls | Sum frequencies for non-null values other than zero |
| Q3 | Exact sums and non-null counts for two requested integer columns, plus table row count | Select the requested columns and compute the average |
| Q4 | Exact sum and non-null count for the requested integer column | Divide the sum by the count |
| Q5 and Q6 | Exact distinct-value count of the requested column | Return that one-column cardinality |
| Q7 | Exact minimum and maximum of the requested column | Select and format both bounds |
| Q8 | Complete numeric value frequencies | Remove zero and null, then sort the surviving groups by count |

The writer leaves the old filtered nonzero-count catalog field empty. It also leaves the old stored pair leaders and host aggregate fields empty. The catalog row interface ignores pair and host results in older files, so SQL execution does not use those saved answers. Q2 through Q8 use only the column and table facts listed above.

Before this change, the direct CSV method for Q3 required the exact `hits` statement with `AdvEngineID` and `ResolutionWidth`; the Q8 method required the exact `hits` statement with `AdvEngineID`. The CLI would only try Q8's path if that literal column name followed `SELECT`. The methods now parse the SQL and use their existing strict shape checks to resolve the actual table and columns. Unsupported filters, grouping changes, and limits use normal SQL execution. This changes no stored bytes.

The RuDB library passed all 395 tests, the CLI library passed all 30 tests, and strict Clippy passed for both crates. The focused tests include alternate table and column names and reject changed SQL semantics. An end-to-end CLI check over a separate `events` table returned `14,4,20` for a sum, count, and average, and `4,2` then `9,1` for filtered grouped counts.

The table below gives the median of 11 alternating fresh-process pairs at each size. Both engines ran the same Q3 or Q8 SQL against native databases loaded from the same Parquet rows. Every process returned the same complete CSV output as DuckDB. The helper measures the child with `wait4`, including process startup, file open, SQL execution, output, and exit. The operating-system page cache was not cleared. CPU is user plus system time for the child.

| Query | Rows | RuDB wall | DuckDB wall | DuckDB / RuDB wall | RuDB CPU | DuckDB CPU | RuDB peak RSS | DuckDB peak RSS | DuckDB / RuDB RSS |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Q3 | 1k | 5.275 ms | 173.616 ms | 32.91x | 4.008 ms | 124.589 ms | 4.50 MiB | 36.53 MiB | 8.12x |
| Q3 | 10k | 7.716 ms | 141.315 ms | 18.31x | 4.833 ms | 118.355 ms | 4.50 MiB | 36.52 MiB | 8.12x |
| Q3 | 1m | 6.516 ms | 189.329 ms | 29.06x | 4.457 ms | 154.449 ms | 4.50 MiB | 42.15 MiB | 9.37x |
| Q3 | 10m | 5.314 ms | 399.294 ms | 75.14x | 4.591 ms | 475.570 ms | 4.50 MiB | 79.30 MiB | 17.62x |
| Q8 | 1k | 4.031 ms | 198.861 ms | 49.33x | 3.869 ms | 159.677 ms | 4.38 MiB | 39.02 MiB | 8.92x |
| Q8 | 10k | 6.169 ms | 198.517 ms | 32.18x | 5.093 ms | 173.740 ms | 4.38 MiB | 39.02 MiB | 8.92x |
| Q8 | 1m | 4.805 ms | 238.002 ms | 49.54x | 4.158 ms | 202.862 ms | 4.38 MiB | 42.27 MiB | 9.66x |
| Q8 | 10m | 5.696 ms | 299.353 ms | 52.55x | 4.628 ms | 319.620 ms | 4.25 MiB | 61.43 MiB | 14.45x |

The host has eight CPUs and had load averages around 42 to 55 during the build and run. The alternating samples compare the two engines during the same period, but these wall times do not establish quiet-host latency. The 10m source has exactly 10,000,000 rows; the 1m source has 999,975 rows. Small-file peak RSS remains short of the 10x reduction target, even though the measured wall ratios exceed 10x.

The [raw Q3 and Q8 samples](q2-onward-fast-path-audit/) contain every child measurement. The benchmark used RuDB commit `8e64cf8c`, release binary SHA-256 `052ca6ad50fafb733ed81c79b1ed8e238acfb90320e66183b7a1316b7b12054f`, and DuckDB v2.0.0-dev84237, binary SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The [Q3 runner](../../scripts/q3-fresh-process.py) and [Q8 runner](../../scripts/q8-fresh-process.py) preserve the measurement method.
