# Q9 region-first row projection experiment

The merged Q9 projection keeps every `(UserID, RegionID)` source row in user order and counts distinct pairs while scanning. This experiment kept the same rows and ten-byte record width but sorted by `RegionID` and then `UserID`. The reader counted a pair when it differed from the previous pair. It stored no distinct pairs, group counts, or run lengths. [RuDB commit `fb63c085`](https://github.com/tamnd/rudb/commit/fb63c08507c8f512b380cf4e5b28515eebb2f6be) contains the prototype on a separate branch. The slower layout was not merged.

The builder used `rudb-projection --cluster-covered FILE hits UserID RegionID`. It made a separate copy of each native file and attached only the region-first projection. The existing user-first projection was built from the same base file. Both indexed files had the same byte length at every size. Every process ran the same Q9 SQL and returned the same complete selected key/count set as DuckDB. Tied counts may appear in either order under the SQL, so the runner checks the set and descending counts. The [fresh-process runner](../../scripts/q9-fresh-process.py) rotated all three cases; 1k, 10k, and 1m use 11 rounds, and 10m uses 51.

| Rows | RuDB user-first wall | RuDB region-first wall | DuckDB native wall | DuckDB / region-first wall | User-first CPU | Region-first CPU | DuckDB CPU | User-first RSS | Region-first RSS | DuckDB RSS |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1k | 21.428 ms | 18.362 ms | 188.198 ms | 10.25x | 8.409 ms | 8.328 ms | 84.547 ms | 5.38 MiB | 5.38 MiB | 38.21 MiB |
| 10k | 50.059 ms | 55.042 ms | 158.920 ms | 2.89x | 30.186 ms | 31.624 ms | 90.940 ms | 5.88 MiB | 5.88 MiB | 39.58 MiB |
| 1m | 44.192 ms | 42.057 ms | 295.868 ms | 7.03x | 23.858 ms | 25.959 ms | 259.541 ms | 8.00 MiB | 8.00 MiB | 81.96 MiB |
| 10m | 91.181 ms | 96.513 ms | 589.543 ms | 6.11x | 103.513 ms | 104.524 ms | 667.926 ms | 11.25 MiB | 11.00 MiB | 135.24 MiB |

At 10m, region-first was 5.8% slower in wall time and 1.0% slower in CPU time than user-first. The small RSS reduction does not compensate for that loss. Build wall time was 3.689 s, peak RSS was 255.32 MiB, and file growth was 101,570,819 bytes, the same file growth as user-first. The [raw query samples](q9-region-cluster/q9-cluster-vs-sorted-10m.json) and [build measurement](q9-region-cluster/cluster-build-10m.json) retain the process records.

The six-CPU host had a load average near 30 during the 10m run. These wall values should not be compared to the earlier Q9 report as if the host were unchanged. The rotating comparison and nearly equal CPU times do support the narrower conclusion: changing row order alone did not remove the main cost. `strace` showed six worker threads on the existing 10m user-first scan, so the remaining time is not caused by an accidental single-worker path. The next work should target the bytes read, copied, checksummed, and decoded for every source row.

The prototype release binaries had SHA-256 `7b9be8ce3753985c076ab05f9e3c0feb6e8244ce943e4cf858be1cc71f8789b4` for RuDB and `9eed187ab4715eb8582e4760472d80fa1a4cdb578ba6460599f8d1240333e241` for the projection builder. DuckDB v2.0.0-dev84237 had SHA-256 `bb7b276fa5805c257becbb0e7238297d45aa4ea6d33ad933637cc0fa0a7d2531`. The focused native and database tests passed. This experiment is not a new 10x result.
