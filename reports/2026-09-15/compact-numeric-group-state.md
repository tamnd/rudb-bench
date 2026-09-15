# Compact numeric group state on rudb 0.3.20

ClickBench queries 31 to 33 group rows with `COUNT(*)`, `SUM(IsRefresh)` and `AVG(ResolutionWidth)`. Both arguments are `SMALLINT`. The general aggregate holds three tagged accumulator objects in every group, repeating the operation and return type beside each state. The compact path keeps one count, an exact sum and its valid flag, and an exact average sum and count. Result types stay in the plan. Null SUM and AVG inputs remain independent of COUNT(*).

The comparison used two rudb builds from the same `main` revision, `7294b12`, after worker-local radix tables were reenabled in PR 621. The binaries had different SHA-256 hashes and were pinned before timing. All 43 ClickBench queries ran five hot repetitions in fresh processes on the same Linux host and 8,192-row-group Parquet files. DuckDB native opened a table loaded once before timing. DuckDB Parquet and rudb read the same file per query. Suite time and CPU sum each query's median. Peak RSS is the largest child resident memory across the suite. These are sample measurements, not official ClickBench scores.

| 1m row run | Engine | Sum of median query time (s) | Sum of median child CPU (s) | Peak child RSS (MiB) |
| --- | --- | ---: | ---: | ---: |
| General-state run | DuckDB native | 0.547 | 3.817 | 313.8 |
| General-state run | DuckDB Parquet | 1.579 | 7.932 | 364.3 |
| General-state run | rudb | 0.9767 | 5.675 | 291.2 |
| Compact-state run | DuckDB native | 0.548 | 3.829 | 311.0 |
| Compact-state run | DuckDB Parquet | 1.622 | 8.112 | 362.4 |
| Compact-state run | rudb | 0.9652 | 5.437 | 258.6 |

The compact build also completed the smaller size ladder with all 43 queries and no unresolved correctness retests.

| Size | Engine | Sum of median query time (s) | Sum of median child CPU (s) | Peak child RSS (MiB) |
| --- | --- | ---: | ---: | ---: |
| 1k | DuckDB native | 0.118 | 1.203 | 55.0 |
| 1k | DuckDB Parquet | 0.150 | 1.263 | 54.9 |
| 1k | rudb compact | 0.0232 | 0.0598 | 6.4 |
| 10k | DuckDB native | 0.142 | 1.247 | 55.0 |
| 10k | DuckDB Parquet | 0.194 | 1.382 | 58.9 |
| 10k | rudb compact | 0.0697 | 0.1254 | 9.7 |
| 100k | DuckDB native | 0.321 | 1.499 | 82.5 |
| 100k | DuckDB Parquet | 0.371 | 2.125 | 117.3 |
| 100k | rudb compact | 0.1953 | 0.7593 | 40.2 |

| Query on 1m rows | Engine and run | Median query time (ms) | Median child CPU (ms) | Peak child RSS (MiB) |
| --- | --- | ---: | ---: | ---: |
| 31 | DuckDB native, general run | 12.0 | 69.0 | 91.7 |
| 31 | rudb general | 22.4 | 126.6 | 68.7 |
| 31 | DuckDB native, compact run | 13.0 | 70.8 | 94.3 |
| 31 | rudb compact | 18.7 | 113.9 | 57.8 |
| 32 | DuckDB native, general run | 15.0 | 73.9 | 104.3 |
| 32 | rudb general | 20.9 | 120.3 | 65.5 |
| 32 | DuckDB native, compact run | 11.0 | 73.2 | 105.9 |
| 32 | rudb compact | 18.7 | 101.1 | 53.7 |
| 33 | DuckDB native, general run | 19.0 | 184.3 | 218.6 |
| 33 | rudb general | 55.6 | 427.2 | 291.2 |
| 33 | DuckDB native, compact run | 19.0 | 176.1 | 218.8 |
| 33 | rudb compact | 48.0 | 335.2 | 229.1 |

Query 33's peak fell 21 percent and its median time fell 14 percent. The full suite peak fell 11 percent because query 34 became its largest memory user at 259 MiB. The full suite query sum improved about one percent and child CPU fell four percent. All deterministic correctness retests matched at 1k, 10k, 100k and 1m rows. A grouped test covers null inputs, including a group where both SUM and AVG are null while COUNT(*) is two.

The compact build still takes 1.76 times the DuckDB native suite query time and 83 percent of its peak RSS at 1 million rows. Worker-local copies, URL key storage and Parquet decoding keep it far from the requested 10 times faster and 10 times smaller target.
