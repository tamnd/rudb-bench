# Native storage audit from Q2 through Q8

Correction: a complete low-cardinality numeric frequency table contains the full grouped count for Q8, even if the table is not stored in output order. The [grouped-count correction](q2-onward-grouped-count-correction.md) made Q8 read rows at query time, and the [partial-frequency review](q2-q8-partial-numeric-frequencies.md) removes complete multi-value numeric frequencies from newly written files. The Q8 classification below records the earlier design.

The native file should hold facts about a table or column that another query can reuse. It should not hold a saved result for one ClickBench statement. A query may recognize a narrow SQL shape and use those facts, provided it still derives the result at runtime and falls back when the facts do not prove the answer.

| Query | Stored input | Runtime work | Audit result |
| --- | --- | --- | --- |
| Q2, filtered count | Complete numeric frequencies, including nulls | Add counts for non-null values other than zero | Corrected in [RuDB #1619](https://github.com/tamnd/rudb/pull/1619) |
| Q3, sum, count, average | Per-column exact integer sums and non-null counts, plus table row count | Select two column sums and divide one by its non-null count | Reusable column statistics |
| Q4, average | Per-column exact integer sum and non-null count | Divide at query time | Reusable column statistics |
| Q5, distinct users | Per-column exact distinct-value count | Read the column statistic | Reusable cardinality statistic |
| Q6, distinct phrases | The same per-column distinct-value count | Read the column statistic | Reusable cardinality statistic |
| Q7, date bounds | Per-column exact minimum and maximum | Select the requested column bounds | Reusable column statistics |
| Q8, grouped count | Complete numeric value frequencies | Filter zero and null, then order groups by count | Stored grouped answer in practice; corrected later |

Q2 was the exception. Its catalog field stored the exact nonzero count, which was the ClickBench Q2 answer for `AdvEngineID`. New files leave that legacy field empty. The reader ignores the field in older files, and the [corrected Q2 measurements](q2-generic-frequency.md) use the query-time frequency calculation. The earlier [stored-count report](../2026-09-23/q2-fresh-process-catalog-count.md) is marked historical.

Q5 and Q6 deserve a precise boundary: an exact distinct-value count can equal a `COUNT(DISTINCT column)` result. It is still a standard property of one column, independent of a particular SQL statement, filter, grouping key, or output order. The earlier Q8 frequency table held every group key and count, so ordering at runtime did not make it acceptable. The current writer keeps multi-value numeric frequencies incomplete and Q8 reads encoded rows. Q3, Q4, and Q7 use column facts rather than a saved row for the query.

The CLI recognizes narrow SQL shapes and formats some scalar results directly. That runtime specialization does not add a stored query result. This audit checks what the file contains and how these paths use it. The later [partial-frequency review](q2-q8-partial-numeric-frequencies.md) remeasures Q2 and Q8 after the storage correction; the earlier [Q3](../2026-09-23/q3-fresh-process.md), [Q4](../2026-09-23/q4-fresh-process.md), [Q5](../2026-09-23/q5-fresh-process.md), [Q6](../2026-09-23/q6-current-main.md), and [Q7](../2026-09-23/q7-fresh-process.md) reports retain their own measured binaries and limits.
