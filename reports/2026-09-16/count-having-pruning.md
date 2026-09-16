# Prune groups below a COUNT HAVING bound

ClickBench Q29 builds 604,336 host groups at ten million rows, but only 11 of them satisfy `HAVING COUNT(*) > 100000`. RuDB previously materialized the keys and all four aggregate results for every completed group, then let the ordinary filter discard 604,325 rows. The new aggregate path recognizes a direct lower bound on an output `COUNT(*)` and selects the qualifying completed groups before it creates result vectors. The downstream filter remains in the pipeline and checks the predicate again.

Implementation: [tamnd/rudb#681](https://github.com/tamnd/rudb/pull/681).

The exact A/B used two release binaries built from the same main revision. The candidate differed only by the count-bound change. Each of the 43 original ClickBench queries ran five times in a fresh process for each binary, interleaved on `gamingpc-wsl`, with 16 threads and a 25 GiB memory limit. Both binaries read the same 9,999,750-row native database. All 430 executions succeeded.

| Measurement | Baseline | Candidate | Change |
| --- | ---: | ---: | ---: |
| Sum of 43 per-query medians | 4.511656 s | 4.479847 s | -0.71% |
| Sum of median child CPU | 25.139014 s | 25.362923 s | +0.89% |
| Q29 median query time | 401.983 ms | 373.171 ms | -7.17% |
| Q29 completed groups materialized | 604,336 | 11 | -99.998% |

The full-suite difference is 31.8 milliseconds, of which Q29 accounts for 28.8 milliseconds. Changes in other queries are run noise: this optimization is selected only for Q29 in the measured suite. Its benefit is limited because it acts after grouping. Q29 still scans 9,999,750 rows, extracts a host for every nonempty referrer, probes the group table, and maintains `AVG`, `COUNT`, and `MIN(Referer)` for every group before the result can be tested.

Q29's output was byte-identical between revisions. Q22, Q23, Q32, Q33, Q39, Q40, and Q41 showed the same tied `ORDER BY ... LIMIT` variation already documented in the root-cause report; repeated runs of one unchanged binary also vary at those boundaries. Q19 and Q33 both completed, so this is the full upstream suite: ClickBench has Q1 through Q43, not 45 queries.

The implementation accepts only lower bounds that are safe for early discard: `COUNT(*) >` a BIGINT constant and `COUNT(*) >=` a BIGINT constant. Upper bounds remain ordinary filters. Unit tests cover the accepted and refused plan shapes, and both compact and general count states use their exact completed count.

Validation:

- `cargo test -p rudb-exec --lib` (216 passed)
- `cargo clippy -p rudb-exec --all-targets -- -D warnings`
- complete 43-query, five-process interleaved 10M A/B

Raw artifacts are retained at `/home/gopher/clickbench-native-audit/run-20260916-v3-direct-output/10m-native-having-full`.
