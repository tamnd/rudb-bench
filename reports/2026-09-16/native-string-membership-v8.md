# Native v8 exact string-page membership

RuDB's native string pages store global dictionary codes.
Native v8 now also stores the sorted, deduplicated set of those codes beside every string page as a delta-varint stream.
The directory records each membership page's offset, length, and checksum.
A reader can therefore prove that a stripe contains none of a supplied set of dictionary codes without reading or decoding the string data page.
Native v7 remains readable and conservatively skips nothing through this API.

This is a storage primitive, not an asserted query speedup.
An eager `LIKE '%literal%'` consumer was implemented and measured, then removed after its warm-run gate failed.
The accepted change is only the v8 format, reader, corruption checks, and exact `skips_codes` API.

Implementation: [tamnd/rudb#721](https://github.com/tamnd/rudb/pull/721).

## Format cost

Both comparisons loaded the same ClickBench Parquet file into fresh database paths.
The 1M run was on `server3`; the 10M run was on `gamingpc-wsl`.
The main and candidate binaries at each size shared the same source base apart from the native format change.
`/usr/bin/time -v` measured the complete ClickBench load script, including its final `sync`.

| Rows | Format | Load wall | User CPU | System CPU | Peak RSS | Database bytes |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 999,975 | v7 main | 14.98 s | 11.50 s | 3.31 s | 459,536 KiB | 528,749,683 |
| 999,975 | v8 membership | 15.92 s | 11.62 s | 4.09 s | 456,140 KiB | 531,512,882 |
| 9,999,750 | v7 main | 25.77 s | 22.22 s | 3.14 s | 2,396,612 KiB | 4,917,514,117 |
| 9,999,750 | v8 membership | 26.89 s | 22.81 s | 3.65 s | 2,407,820 KiB | 4,944,860,926 |

At 10M, the exact index adds 27,346,809 bytes, or **0.556%**, increases complete load wall time by **4.35%**, and increases peak RSS by **0.47%**.
At 1M it adds **0.523%** storage; the single load pair moved by 6.3%, so the larger run is the acceptance measurement rather than claiming a stable small-load difference.

The 10M file has 9,796 stripes and 28 string columns.
For `URL`, 1,166 global codes contain the bytes `google`, and exact membership retains 897 pages while proving that 8,899 pages cannot match: **90.84% of URL pages are skippable**.
Across all string columns, the encoded membership payload is 22,958,313 bytes, 2.05% of the existing string-code pages and 0.47% of the v7 database.
Directory descriptors account for the rest of the measured 0.556% file growth.

## Compatibility and correctness

The candidate opened the v7 1M file and returned all 999,975 rows.
A fresh v8 file also returned 999,975 rows.
All 43 original ClickBench queries executed against both files.
Deterministic outputs matched.
Q8 changed only the order of groups tied on its sole ordering key; Q23, Q32, and Q33 chose different rows at an `ORDER BY ... LIMIT` tie, as those statements permit.
The v8 reader rejects a membership checksum mismatch and malformed, overflowing, non-monotone delta streams instead of using damaged metadata to skip data.

## Rejected eager consumer

The rejected consumer scanned a global dictionary once to find codes containing the literal, then consulted membership before each stripe read.
Seven fresh processes per side ran the four original containment-LIKE queries against the same 10M rows.
The same binary read v7 with pruning disabled and v8 with pruning enabled.

| Query | v7 hot median | Eager v8 hot median | Result |
| ---: | ---: | ---: | ---: |
| Q21 | 0.33 s | 0.77 s | 2.33x slower |
| Q22 | 0.33 s | 0.75 s | 2.27x slower |
| Q23 | 0.31 s | 0.40 s | 1.29x slower |
| Q24 | 0.33 s | 0.76 s | 2.30x slower |

Replacing 64 KiB reads and per-value searches with 8 MiB sequential reads plus one streaming substring search reduced the overhead but still failed:

| Query | v7 hot median | Batched v8 hot median | Result |
| ---: | ---: | ---: | ---: |
| Q21 | 0.32 s | 0.61 s | 1.91x slower |
| Q22 | 0.33 s | 0.57 s | 1.73x slower |
| Q23 | 0.32 s | 0.34 s | 1.06x slower |
| Q24 | 0.32 s | 0.59 s | 1.84x slower |

The root cause is discovery, not membership lookup: finding candidate codes scans the 549 MB URL dictionary before the scan can reject a page.
The existing stable-dictionary LIKE kernel instead tests only codes encountered by rows and caches each answer inside the query.
On warm files that is cheaper than eagerly classifying the complete dictionary, even when 90.84% of pages can then be skipped.
The eager planner integration and its extra dependency were reverted before the PR.

A future consumer needs a bounded persisted lookup from useful substring features to candidate codes, or adaptive feedback that learns code matches while pages flow through the filter.
It must beat the v7 hot medians above at both 1M and 10M before being merged.
Exact page membership is kept because it is small, backward compatible, independently validated, and is the required second half of either design.

Raw artifacts remain on the benchmark hosts:

- `server3:/root/native-membership-bench-20260916`
- `gpc:/home/gopher/clickbench-native-audit/native-membership-20260916`
