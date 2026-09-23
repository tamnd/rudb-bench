# Q9 sorted projection feasibility experiment

**This is a standalone C++ prototype, not a RuDB SQL result or a completed 10x claim.** It
tests whether storing rows in `UserID` order can remove Q9's query-time pair hash set without
storing Q9's answer. The projection contains every `UserID` and a 16-bit dictionary code for
its `RegionID`. It stores no grouped count, pair count, ranking, or TopN result.

The 9,999,750 source rows were sorted by `UserID` from the existing DuckDB native table.
At query time, sixteen workers split the sorted stream only at user boundaries. Each worker
marks the regions it sees for the current user and increments that region's count once.
The workers add their region counters after scanning and take the top ten. This is exact
for users appearing in several regions. The sample has no nulls in either column; a native
implementation must handle nulls before this path is eligible generally.

The [prototype](../../research/q9-sorted-projection/projection.cpp) and
[runner](../../research/q9-sorted-projection/bench.py) alternated fresh processes for 51
trials. Both produced the same complete Q9 output set, in descending count order, on every
trial. The `wait4` helper measured each child from spawn to exit, including output.

| Run | Program | Median wall ms | Median CPU ms | Median peak RSS MiB |
| --- | --- | ---: | ---: | ---: |
| A | Sorted projection prototype | 4.635 | 12.881 | 4.98 |
| A | DuckDB native | 93.390 | 1014.564 | 426.05 |
| B | Sorted projection prototype | 6.715 | 21.630 | 4.97 |
| B | DuckDB native | 174.756 | 1024.214 | 421.61 |

The prototype is 20.1x faster in run A and 26.0x faster in run B, with over 84x less peak
RSS in both. The shared host was heavily loaded during run B, so its wall medians are less
useful for predicting an integrated result. These prototype ratios do **not** transfer to
RuDB until a generic native projection,
catalog validation, optimizer selection, SQL execution, and update path are implemented and
measured. The current integrated [RuDB Q9 result](q9-input-row-references.md) is 50.431 ms
and 298.54 MiB, against DuckDB's 93.513 ms and 424.43 MiB in that separate run.

The packed projection is 99,997,500 bytes, ten bytes per row. Its 6,817-entry region
dictionary is 27,272 bytes. The builder assigns dictionary codes to ordinary column values;
it does not compute a query aggregate. The following one-time steps used an already loaded
DuckDB database and are **not** a Parquet-to-RuDB load benchmark:

| Build step | Wall time | Peak RSS | Output bytes |
| --- | ---: | ---: | ---: |
| DuckDB sort and CSV export | 0.26 s | 955 MiB | 234,846,171 |
| CSV to packed projection | 0.32 s | 4.5 MiB | 100,024,772 |

An integrated writer must sort directly into the native file and report its actual load
time, CPU, RSS, and file bytes. It must keep the projection correct across appends or stop
using it until rebuilt. The regular native scan remains the fallback. The optimizer should
select a projection from its declared order and covered columns, not from Q9's SQL text.

The [run A trials](q9-sorted-projection/low-load.json),
[run B trials](q9-sorted-projection/high-load.json), and
[expected CSV](../2026-09-23/q9-corrected-query-time/q9-duck-10m.csv) are retained.
