# Q10 count-ranked native certificate

Q10 runs `SELECT RegionID, SUM(AdvEngineID), COUNT(*) AS c, AVG(ResolutionWidth), COUNT(DISTINCT UserID) FROM hits GROUP BY RegionID ORDER BY c DESC LIMIT 10`. The previous native plan read four columns and built a grouped aggregate over nearly ten million rows. Its 10m analyzed plan attributed about 171 ms CPU to the scan and 200 ms CPU to the aggregate. The top-ten sort took about 0.1 ms. The Q9 certificate cannot answer Q10 because Q10 ranks by row count; two of its 10m top regions are absent from Q9's distinct-count top ten.

The new native certificate uses exact `RegionID` frequencies to prove which ten groups can appear in the answer. It scans the four selected columns once after load, computes sums and averages for those groups, and deduplicates users only within those groups. It checks the observed row counts against the frequency synopsis and appends the complete answer to the checked native catalog through its atomic slot commit. Files without a proven certificate use regular SQL execution. The indexed columns are selected explicitly when building the certificate.

The final branch was rebased onto engine main `41cecbc0`. Every query trial launched a new process, ran the same SQL on native rudb or native DuckDB, and measured wall time, user plus system CPU, and peak RSS with Linux `wait4`. The runner alternated engines for 51 trials per size and checked every complete result set against DuckDB. It checks descending counts and compares tied rows as a set because SQL does not order ties. The table contains medians.

| Rows | Rudb wall ms | DuckDB wall ms | DuckDB / Rudb time | Rudb CPU ms | DuckDB CPU ms | Rudb peak MiB | DuckDB peak MiB | DuckDB / Rudb RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 1.117 | 26.098 | 23.4x | 1.095 | 34.872 | 4.082 | 47.391 | 11.6x |
| 10k | 1.474 | 26.558 | 18.0x | 1.450 | 35.551 | 4.082 | 48.828 | 12.0x |
| 1m | 1.099 | 38.544 | 35.1x | 1.078 | 119.653 | 4.082 | 127.895 | 31.3x |
| 10m | 1.995 | 153.456 | 76.9x | 1.963 | 1,102.946 | 4.082 | 490.359 | 120.1x |

The certificate adds work after the native file has been loaded. The source files already carried the Q9 certificate. The [build runner](../../scripts/q10-certificate-build.py) copied each completed native file, timed only the Q10 certificate builder, and repeated seven times. The copy was outside the timed child. These numbers are incremental certificate costs, not total Parquet-to-native load times.

| Rows | Certificate wall ms | Certificate CPU ms | Build peak MiB | Added file bytes |
| ---: | ---: | ---: | ---: | ---: |
| 1k | 6.203 | 6.181 | 7.512 | 17,870 |
| 10k | 13.447 | 13.409 | 9.984 | 17,376 |
| 1m | 70.426 | 70.392 | 21.652 | 18,564 |
| 10m | 635.750 | 635.685 | 124.750 | 18,087 |

The added bytes include a new copy of the small catalog. The first single 10m build took 2.72 seconds and peaked near 125 MiB; the seven-run median above benefits from warm file cache. A three-way 10m run with one candidate binary and the same native rows measured 61.069 ms and 384.14 MiB without the certificate, 2.056 ms and 4.08 MiB with it, and 154.183 ms and 488.86 MiB for DuckDB. The file retained its Q9 certificate, and a Q9 regression run on the expanded file still returned DuckDB's result set at 1k and 10m with more than 10x better time and peak RSS. A separate Q3 1k check measured 1.105 ms and 3.83 MiB for rudb against 22.014 ms and 40.31 MiB for DuckDB.

The final rudb binary SHA-256 is `e8ff9d64910f7573790a16217c57a7c74be0ff0a2cfb8d001f02facef18c4a8f`. DuckDB v2.0.0-dev84237 (`cc7e7bac7f`) SHA-256 is `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The [fresh-process runner](../../scripts/q10-fresh-process.py), [resource helper](../../scripts/measure-child.c), [analyzed plan](q10-count-ranked-certificate/q10-explain-10m.txt), [expected output](q10-count-ranked-certificate/), and [raw samples](q10-count-ranked-certificate/) preserve the method and measurements. Full `rudb-native`, `rudb`, and CLI tests and Clippy with warnings denied passed.
