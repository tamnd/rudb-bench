# Answers

What a query is supposed to return, committed, so that a run can be checked against something that is not another engine.

Every other correctness check in this harness is one engine against another, which is a strong check with one blind spot: it cannot see two engines that are wrong in the same way. That is not a theoretical worry on TPC-H, where every column is money and where the engines have mostly copied each other's decimal rules. So a reference that nobody in the table produced is worth having, and this is where it lives.

## `tpch-sf1/`

The answers TPC-H publishes for its own twenty two queries against the qualification database at scale factor one, one file per query, `q01.csv` through `q22.csv`.

The specification prints them pipe separated with the column names on top. They are committed as CSV with the column names still on top, because every engine here is asked for CSV with no header, and a reference in a different format would be comparing the separator as well as the answer. The fields are the exact text the published answer carries and nothing was reformatted, which is why `53758257134.8700` is in `q01.csv` rather than what a double prints as. `src/qualified.rs` takes the header line off before comparing and everything else goes through as it is.

They apply at SF1 and nowhere else. A TPC-H run at any other scale is a run against a different database with different answers, so `qualified::applies` reads the scale out of the corpus line and this reference stays out of the way unless it is the right one.

### Where they came from, and what is weak about that

They were taken out of DuckDB's `tpch` extension, which ships the published set as the `tpch_answers()` table function:

```sql
INSTALL tpch; LOAD tpch;
SELECT answer FROM tpch_answers() WHERE scale_factor = 1 AND query_nr = 1;
```

That is worth being plain about, because it weakens the independence the directory exists for. DuckDB validates itself against these same files, so a mistake in DuckDB's copy of them is a mistake nobody here would catch either.

Two things make it worth having anyway. The values match the ones the specification prints, digit for digit as far as the specification prints them, which anybody can check by hand against the appendix for as many of the twenty two as they have patience for. And the failure this is meant to catch is a rudb bug, which is caught whatever the provenance of the file, because rudb had no part in producing it.

The honest fix, for whoever wants it, is to retype the appendix or to take the answer set the TPC's own toolkit ships. The files have the same shape either way, so it is a replacement of twenty two files and no code.

### The five that can tie

Q2, Q3, Q10, Q18 and Q21 order by keys that do not totally order the rows and then cut with a `LIMIT`. The published answer is one correct choice of rows at that cut, and an engine that kept a different set of tied rows is not wrong. Settling that needs the query run again without its `LIMIT`, which is `answer::boundary` and issue #148 is the plumbing for it. Until then a difference in one of those five is reported with that sentence next to it rather than called a wrong answer.

## Adding a set

Put the files under `answers/<suite>-<scale>/`, one per query, named for the query as `src/suite.rs` names it, with the column names on the first line. Then add the directory to `ANSWERS` in `src/qualified.rs` and say in the module doc where the numbers came from, because a reference whose provenance is not written down is a reference nobody can weigh.
