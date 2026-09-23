# Audit of native metadata used by ClickBench

Native loading may store reusable statistics about the data. It must not run a ClickBench expression or aggregate in advance and store its answer. Query time includes any work needed to compute the requested groups, distinct counts, measures, and ordering from those statistics and the stored columns.

| Query | Stored data | Audit result |
| --- | --- | --- |
| Q2 | Column non-null and nonzero counts | Reusable column statistics |
| Q3 | Column sum, non-null count, and table row count | Reusable column and table statistics |
| Q4 | Column sum and non-null count | Reusable column statistics |
| Q5, Q6 | Per-column distinct count | Reusable column statistics |
| Q7 | Per-column minimum and maximum | Reusable column statistics |
| Q8 | Per-column value frequencies | Reusable column statistics |
| Q9 | Top groups with distinct counts for a selected column pair | Query answer, removed |
| Q10 | Top groups with count, sum, average, and distinct count for selected columns | Query answer, removed |
| Q11 experiment | Top nonempty string groups with distinct counts | Query answer, abandoned before merge |
| Q17 | Top `(UserID, SearchPhrase)` pairs and counts | Query answer, disabled for reads and new writes |
| Q29 | Groups from the fixed host expression with counts and measures | Query answer, disabled for reads and new writes |

The Q9 and Q10 candidate reports and their result files were removed. The older Q17 and Q29 reports have retraction notices because they also contain useful historical load measurements.
Their claimed query speedups are not evidence for the 10x goal. Existing native files containing these optional blocks remain readable, but query execution no longer uses the blocks. The Q11 certificate branch was not merged and its candidate numbers are not published.

No new DuckDB comparison is asserted here. The valid per-query comparisons for Q2 through Q8 remain in their individual reports. Q9, Q10, Q17, and Q29 need fresh-process measurements after the engine computes their answers at query time.
