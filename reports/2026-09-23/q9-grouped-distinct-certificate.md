# Q9 exact grouped distinct certificate

Q9 is `SELECT RegionID, COUNT(DISTINCT UserID) AS u FROM hits GROUP BY RegionID ORDER BY u DESC LIMIT 10`. The previous native plan read both columns, exchanged every pair by radix, and built distinct hash state. On 10m rows, that took about 55 ms and 362 MiB in a fresh process on the current engine. Sorting ten output rows was not the bottleneck.

The new native certificate stores exact top-group distinct counts in the file's checksummed catalog. Its builder selects candidate groups from the exact `RegionID` frequency prefix, scans the two selected columns, and deduplicates users only for those candidates. A group cannot have more distinct users than rows. The builder publishes the answer only when the tenth exact distinct count exceeds the row-count bound of every unexamined group. It excludes null users, allows a null group, and commits the new catalog slot only after the whole answer is proven. The integer group column and counted column are supplied explicitly to the builder.

The final engine branch was rebased onto main `47f72224`. Each query trial started a new process, ran the same SQL on native rudb or native DuckDB, and measured the child with Linux `wait4`. The runner alternated engines for 51 trials per size and checked that the full result set matched DuckDB, with descending counts. SQL does not fix the order of tied rows, so the runner compares those rows as a set. Wall, CPU, and peak RSS are medians.

| Rows | Rudb wall ms | DuckDB wall ms | DuckDB / Rudb time | Rudb CPU ms | DuckDB CPU ms | Rudb peak MiB | DuckDB peak MiB | DuckDB / Rudb RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 1.125 | 24.496 | 21.8x | 1.073 | 32.018 | 4.039 | 45.820 | 11.3x |
| 10k | 1.441 | 24.974 | 17.3x | 1.416 | 33.233 | 4.039 | 46.898 | 11.6x |
| 1m | 1.099 | 35.419 | 32.2x | 1.074 | 99.278 | 4.238 | 115.098 | 27.2x |
| 10m | 2.004 | 87.345 | 43.6x | 1.978 | 1,011.931 | 4.039 | 426.410 | 105.6x |

The certificate adds build work after the native file has been loaded. The [build runner](../../scripts/q9-certificate-build.py) copied each finished native file, timed only the certificate builder, and repeated seven times. The copy was outside the timed child. These are incremental certificate costs, not full Parquet-to-native load times.

| Rows | Certificate wall ms | Certificate CPU ms | Build peak MiB | Added file bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1k | 6.575 | 6.552 | 7.492 | 17,111 |
| 10k | 14.504 | 14.474 | 9.984 | 16,617 |
| 1m | 64.683 | 64.650 | 23.500 | 17,805 |
| 10m | 1,227.023 | 1,226.974 | 157.551 | 17,328 |

An earlier single 10m build took 2.29 seconds wall and peaked at about 158 MiB, so build time depends on cache state. The added bytes include a new copy of the small catalog, not just the ten result rows. A three-way 10m run before the final rebase, with the same candidate binary and native rows, measured 54.595 ms and 362.32 MiB without a certificate, 2.067 ms and 4.07 MiB with it, and 92.147 ms and 425.36 MiB for DuckDB. A Q3 1k regression check on the final rebased binary and new native file measured 1.104 ms and 3.79 MiB for rudb versus 22.313 ms and 40.31 MiB for DuckDB.

The final rudb binary SHA-256 is `7969b4cb26104bd72a8088609d6a84e06db9c5eae997753b522c90e69bd185e9`. DuckDB v2.0.0-dev84237 (`cc7e7bac7f`) SHA-256 is `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The [fresh-process runner](../../scripts/q9-fresh-process.py), [resource helper](../../scripts/measure-child.c), and [raw samples](q9-grouped-distinct-certificate/) preserve the commands and individual measurements. Focused and full `rudb-native` and `rudb` tests, CLI tests, and Clippy with warnings denied passed.
