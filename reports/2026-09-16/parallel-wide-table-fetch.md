# Parallel wide native table fetch

ClickBench Q24 filters ten million URLs, orders the 1,577 matches by `EventTime`, keeps ten, and then asks for every column in those ten rows. Late materialization already keeps the roughly one hundred unneeded columns out of the scan and TopN, but the final `TableFetch` decoded those independent native columns serially in one pipeline instance.

The new path sorts and deduplicates the winning row ordinals before reading, then restores their
TopN order. A native fetch of at least sixteen columns divides the columns over at most eight scoped workers. Ordinary scans are unchanged: their pipeline already parallelizes by stripe, while a late fetch has only a handful of rows and needs columns as its parallel dimension.

Implementation: [tamnd/rudb#705](https://github.com/tamnd/rudb/pull/705).

## Targeted 10M result

Nine fresh processes per revision ran in interleaved order on `gamingpc-wsl`, with 16 query threads and a 25 GiB memory limit. Both release binaries were built from main commit `beb3582`; the candidate differed only by ordinal ordering and wide native fetch parallelism. Both read the same
9,999,750-row native database, and every candidate result was byte-identical to its baseline.

| Measurement | Baseline | Candidate | Change |
| --- | ---: | ---: | ---: |
| Q24 median query time | 210.604 ms | 196.576 ms | -6.66% |
| Q24 median child CPU | 983.467 ms | 949.933 ms | -3.41% |
| `TableFetch` median wall time | 40.189 ms | 36.470 ms | -9.25% |

The filter and scan account for most of Q24 and vary more than the fetch. The operator-local number is therefore the clearest measure: the serial tail is 3.719 ms shorter. CPU also falls rather than being traded for latency, because sorted ordinals avoid repeated stripe work.

## Full suite

The complete original 43-query suite then ran five times per binary in fresh processes, again interleaved. All 430 executions succeeded. Q24 is the only query that uses `TableFetch` in this suite.

| Measurement | Baseline | Candidate | Change |
| --- | ---: | ---: | ---: |
| Sum of 43 per-query medians | 4.279337 s | 4.115114 s | -3.84% |
| Sum of median child CPU | 23.422510 s | 22.720776 s | -3.00% |
| Q24 median query time | 197.777 ms | 193.906 ms | -1.96% |
| Suite maximum tracked peak | 3,046.764 MiB | 3,046.764 MiB | unchanged |

Only Q24's 3.871 ms belongs to this change. The rest of the apparent suite reduction is host variance, most visibly Q21 moving by 129 ms even though its code path is identical. The targeted nine-run operator measurement above is the evidence for the implementation; the full suite is the correctness and regression gate, not evidence of a 3.84 percent suite optimization.

Every deterministic output was byte-identical. Q22, Q23, Q32, Q33, Q39, Q40, and Q41 showed their already documented tied `ORDER BY ... LIMIT` variation. Q24 itself was byte-identical in the targeted and full-suite comparisons.

Raw artifacts are retained at:

- `/home/gopher/clickbench-native-audit/run-20260916-v3-direct-output/10m-native-table-fetch-parallel`
- `/home/gopher/clickbench-native-audit/run-20260916-v3-direct-output/10m-native-table-fetch-parallel-full43`

## Validation

- `cargo test -p rudb-catalog --lib` (33 passed)
- `cargo test -p rudb-exec --lib` (221 passed)
- `cargo clippy -p rudb-catalog -p rudb-exec --all-targets -- -D warnings`
- nine-process interleaved Q24 A/B at 10M
- complete 43-query, five-process interleaved A/B at 10M
