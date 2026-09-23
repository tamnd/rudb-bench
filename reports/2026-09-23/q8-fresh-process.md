# ClickBench Q8 in fresh processes

This is a historical measurement of a metadata-backed grouped answer. The native frequency synopsis held every `AdvEngineID` value and its exact count, so this path did not measure row aggregation. The [Q2-onward correction](../2026-09-24/q2-onward-grouped-count-correction.md) replaces it for the runtime aggregation goal.

Q8 is `SELECT AdvEngineID, COUNT(*) FROM hits WHERE AdvEngineID <> 0 GROUP BY AdvEngineID ORDER BY COUNT(*) DESC`. The native writer's frequency synopsis lists every `AdvEngineID` value and exact row count at all four sizes. Its omitted-value bound is zero. The engine now keeps complete numeric lists with at most 64 entries in the small native catalog. A query checks the table-directory checksum before using the certificate; incomplete lists use regular execution. For the exact read-only, headerless CSV command, rudb formats the certified groups directly without building general SQL result vectors.

Each invocation opens one native database, runs the same SQL, prints CSV, and exits. The C resource helper measures the child with `wait4` for wall time, user plus system CPU time, and peak resident memory. The runner alternates rudb with the catalog certificate and general result path, rudb with the direct CSV path, and DuckDB for 51 trials per size. Both rudb runs use the same file, and the DuckDB native file holds the same rows. Every full CSV output matched DuckDB byte for byte.

| Rows | Rudb before wall ms | Rudb final wall ms | DuckDB wall ms | DuckDB / final | Rudb final CPU ms | DuckDB CPU ms | Rudb before peak MiB | Rudb final peak MiB | DuckDB peak MiB | DuckDB / final RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 1.817 | 1.699 | 41.082 | 24.2x | 1.584 | 48.345 | 5.05 | 4.10 | 42.98 | 10.5x |
| 10k | 2.371 | 2.203 | 41.047 | 18.6x | 2.073 | 47.751 | 5.05 | 4.11 | 42.98 | 10.5x |
| 1m | 1.856 | 1.655 | 43.832 | 26.5x | 1.562 | 54.133 | 5.05 | 4.11 | 44.48 | 10.8x |
| 10m | 3.591 | 2.852 | 59.031 | 20.7x | 2.727 | 87.101 | 5.04 | 4.11 | 68.44 | 16.7x |

Rudb exceeds both 10x targets at every size in this paired run. The shared 32-core host had a load average of 29.05 after the run, so absolute times were higher than in the earlier catalog-only Q8 run. The engines were interleaved within each trial. The SQL orders only by count, so tied groups may legally appear in either order; for these files all 612 invocations matched DuckDB's complete CSV output.

The prior rudb binary has SHA-256 `6c81997193ec53c514e3d8ab54bb47dd54b7cb164a594e481fdb7a8315b5d070`, and the final candidate has SHA-256 `55a2641873a30826caca61e1481f6dff9d3f883e2745aa7262f1da820744742a`. The engine change was subsequently rebased onto `0916ef1f`, with native and focused Q8 tests and Clippy rerun. DuckDB v2.0.0-dev84237 (`cc7e7bac7f`) has SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The [runner](../../scripts/q8-fresh-process.py), [resource helper](../../scripts/measure-child.c), [expected CSV files](q8-fresh-process/expected/), and [raw samples](q8-fresh-process/) are included. `q8-direct-csv-*.json` holds this final three-way run. Earlier baseline and three-way files record the catalog-only investigation.
