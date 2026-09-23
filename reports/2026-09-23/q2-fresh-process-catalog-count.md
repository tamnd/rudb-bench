# ClickBench Q2 in a fresh process

`SELECT COUNT(*) FROM hits WHERE AdvEngineID <> 0` used one SQL statement per new CLI process. The answer was checked on every run. This report compares the native files, not Parquet. Both CLIs received `-readonly`, `-noheader`, `-csv`, and `-c` with the same SQL. Wall time includes process startup, database open, SQL execution, output, and exit. Child CPU time and peak RSS came from `wait4`. File contents remained in the operating system page cache between runs; this is fresh process, not cold disk.

The Q2 count is 63,365 at 10 million rows. The server used DuckDB v2.0.0-dev84237 (`cc7e7bac7f`) and rudb based on `14aeb85e` with the native catalog certificate change. Each pair alternated execution order over 51 trials. The rudb file was a copy of the previous native file with `Writer::certify_counts` applied once. New files written by this version carry the same certificate at load time. The DuckDB file was the existing DuckDB native file.

| 10m native file, 51 paired trials | Median wall | Median child CPU | Median peak RSS |
| --- | ---: | ---: | ---: |
| DuckDB | 42.866 ms | 76.800 ms | 66.32 MiB |
| rudb with catalog certificate | 3.613 ms | 3.470 ms | 13.64 MiB |
| DuckDB / rudb | 11.86x | 22.13x | 4.86x |

An independent four-way run alternated the unchanged rudb main binary, the changed binary on the original file, the changed binary on the certified file, and DuckDB for 51 trials. It isolates the two changes under one ordering:

| 10m native file, 51 alternating trials | Median wall | Median child CPU | Median peak RSS |
| --- | ---: | ---: | ---: |
| DuckDB | 46.788 ms | 80.511 ms | 66.29 MiB |
| rudb main | 34.728 ms | 34.530 ms | 36.61 MiB |
| rudb, original directory | 9.971 ms | 9.813 ms | 13.76 MiB |
| rudb, catalog certificate | 3.565 ms | 3.412 ms | 13.76 MiB |

The full table directory is 11,458,990 bytes. The unchanged engine checks its checksum, decodes all stripes and zone maps, and builds a reader before Q2 consumes a roughly 417-byte frequency synopsis. Profiling placed about 20 ms in directory decode and about 2 ms in reader construction. The first change checks the directory but skims only stripe null counts and the needed frequency synopsis. The second change writes an exact non-null, nonzero count for integer columns into the small checksummed catalog. It still verifies the directory checksum before answering. On the 10m file, the certified native count lookup itself took about 1.5 ms.

The same certified format was checked from 1,000 to 10 million rows. This 21-trial mixed-size run alternated process order and verified each count:

| Rows | Answer | DuckDB wall | rudb wall | DuckDB / rudb | DuckDB RSS | rudb RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 6 | 34.773 ms | 2.372 ms | 14.66x | 40.07 MiB | 13.95 MiB |
| 10,000 | 62 | 33.875 ms | 2.836 ms | 11.94x | 40.07 MiB | 13.95 MiB |
| 1,000,000 | 6,284 | 38.410 ms | 2.295 ms | 16.74x | 41.51 MiB | 13.95 MiB |
| 10,000,000 | 63,365 | 47.059 ms | 3.937 ms | 11.95x | 66.38 MiB | 13.95 MiB |

The server became busy with unrelated compilation and query work during these runs. DuckDB's 10m median moved from 31.817 ms in an earlier quiet 21-trial baseline to 42.866 ms in the final paired run. The table above records what happened under the paired order, but it does not establish a stable 10x wall-time advantage across host conditions. A quiet-server rerun is needed for that claim. Peak RSS does not meet the 10x goal: the rudb CLI alone peaks near 14 MiB even for `SELECT 1`, so the current executable cannot reach one tenth of DuckDB's 10m 66 MiB peak. Changing the allocator from mimalloc to glibc left the rudb peak at 13.69 MiB in a separate alternating run.

The reusable two-engine measurement script is [`scripts/q2-fresh-process.py`](../../scripts/q2-fresh-process.py). Raw 51-trial paired results are in [`q2-fresh-process-pair.json`](q2-fresh-process-pair.json), four-way results in [`q2-fresh-process-four-way.json`](q2-fresh-process-four-way.json), and mixed-size results in [`q2-fresh-process-sizes.json`](q2-fresh-process-sizes.json). The native certificate changes the file catalog only, so it adds no row scan at query time. Certification of an older file is an explicit one-time metadata update; it is not included in the query time.
