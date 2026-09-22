//! The answers TPC-H publishes for its own queries at scale factor one.
//!
//! Every other correctness check in this harness compares one engine against another. That catches
//! a lot, and it cannot catch the thing it is worst placed to catch: two engines that are wrong in
//! the same way. Decimal scale is the obvious candidate, because every money column in TPC-H goes
//! through the same three multiplications and any engine that copied DuckDB's rules copied whatever
//! is wrong with them, and a comparison between two copies of the same rule agrees loudly.
//!
//! So this is the one reference here that is not an engine's output. TPC-H defines a qualification
//! database at scale factor one with the substitution parameters fixed, and publishes the answer to
//! each of the twenty two queries. The numbers do not depend on anybody's implementation, which is
//! the whole point of having them, and they are useless at any other scale, which is why nothing
//! here runs unless the corpus says SF1.
//!
//! ## What is committed and where it came from
//!
//! One file per query under `answers/tpch-sf1/`, with the column names on the first line and the
//! rows under it. The specification prints them pipe separated and they are committed as CSV,
//! because every engine in this harness is asked for CSV with no header and a reference in a
//! different format would be comparing the separator as well as the answer. Nothing else was
//! touched: the fields are the exact text the published answer carries, trailing zeros and all,
//! which is why `53758257134.8700` is in there rather than what a double prints as.
//!
//! They were taken out of DuckDB's `tpch` extension, which ships the published set as
//! `tpch_answers()`, rather than retyped out of the specification's appendix. That is worth being
//! plain about, because it weakens the independence this module exists for: DuckDB validates itself
//! against these files, so a mistake in DuckDB's copy of them would be a mistake nobody here would
//! see either. Two things make it worth having anyway. The values match the ones the specification
//! prints, digit for digit as far as the specification prints them, which is what a reader can
//! check by hand against the appendix for as many of the twenty two as they have patience for. And
//! the failure this is meant to catch is a rudb bug, which is caught whatever the provenance of the
//! file, because rudb had no part in producing it.
//!
//! The honest fix, for whoever wants it, is to retype the appendix or to take the answer set the
//! TPC's own toolkit ships. The files here have the same shape either way, so it is a replacement
//! of twenty two files and nothing else.
//!
//! ## The ties
//!
//! Q2, Q3, Q10, Q18 and Q21 order by keys that do not totally order the rows and then cut with a
//! `LIMIT`. The published answer is one correct choice of rows at that cut and an engine that chose
//! differently is not wrong. Settling that needs the query run again without its `LIMIT`, which is
//! `boundary` in [`crate::answer`] and the plumbing for it is not built, so a disagreement on one
//! of those five is reported with that sentence attached rather than called a wrong answer.

use crate::answer::Agreement;

/// The published answer for each of the twenty two, in query order.
///
/// Compiled in rather than read at run time. A correctness reference that a run can be missing is
/// a correctness reference that quietly is not checked, and these are 700 KB of text that never
/// change.
const ANSWERS: &[(&str, &str)] = &[
    ("q01", include_str!("../answers/tpch-sf1/q01.csv")),
    ("q02", include_str!("../answers/tpch-sf1/q02.csv")),
    ("q03", include_str!("../answers/tpch-sf1/q03.csv")),
    ("q04", include_str!("../answers/tpch-sf1/q04.csv")),
    ("q05", include_str!("../answers/tpch-sf1/q05.csv")),
    ("q06", include_str!("../answers/tpch-sf1/q06.csv")),
    ("q07", include_str!("../answers/tpch-sf1/q07.csv")),
    ("q08", include_str!("../answers/tpch-sf1/q08.csv")),
    ("q09", include_str!("../answers/tpch-sf1/q09.csv")),
    ("q10", include_str!("../answers/tpch-sf1/q10.csv")),
    ("q11", include_str!("../answers/tpch-sf1/q11.csv")),
    ("q12", include_str!("../answers/tpch-sf1/q12.csv")),
    ("q13", include_str!("../answers/tpch-sf1/q13.csv")),
    ("q14", include_str!("../answers/tpch-sf1/q14.csv")),
    ("q15", include_str!("../answers/tpch-sf1/q15.csv")),
    ("q16", include_str!("../answers/tpch-sf1/q16.csv")),
    ("q17", include_str!("../answers/tpch-sf1/q17.csv")),
    ("q18", include_str!("../answers/tpch-sf1/q18.csv")),
    ("q19", include_str!("../answers/tpch-sf1/q19.csv")),
    ("q20", include_str!("../answers/tpch-sf1/q20.csv")),
    ("q21", include_str!("../answers/tpch-sf1/q21.csv")),
    ("q22", include_str!("../answers/tpch-sf1/q22.csv")),
];

/// The five queries whose `ORDER BY` does not settle which rows the `LIMIT` keeps.
///
/// Document 04 section 4.4 names them and they are hard coded rather than worked out, because the
/// query set is fixed by the specification and will not grow a twenty third.
const TIED_AT_THE_CUT: &[&str] = &["q02", "q03", "q10", "q18", "q21"];

/// The published answer for a TPC-H query, ready to compare.
///
/// The column names come off here. The file keeps them because a fixture nobody can read is a
/// fixture nobody will check, and every engine in this harness is asked for CSV with no header, so
/// leaving the line on would make the published answer one row longer than any engine's.
#[must_use]
pub fn answer(query: &str) -> Option<&'static str> {
    let text = ANSWERS.iter().find(|(name, _)| *name == query).map(|(_, text)| *text)?;
    text.split_once('\n').map(|(_, rows)| rows)
}

/// The scale factor a corpus line names.
///
/// Read out of the line rather than carried beside it, because a comparison restored from saved
/// blocks has the line and does not have the dataset that produced it, and a check that works on a
/// fresh run and not on a restored one is a check somebody will be surprised by.
#[must_use]
pub fn scale(corpus: &str) -> Option<&str> {
    let (_, rest) = corpus.split_once(" SF")?;
    Some(rest.split(',').next().unwrap_or(rest).trim())
}

/// Whether these answers are the right reference for a run.
///
/// Two conditions and both are necessary. The wrong suite has no answers here at all. The right
/// suite at the wrong scale has answers that are all wrong, which is worse, because a check that
/// fails on every query is a check that gets turned off rather than read.
#[must_use]
pub fn applies(suite: &str, corpus: Option<&str>) -> bool {
    suite == "tpch" && corpus.and_then(scale) == Some("1")
}

/// Whether a query's `LIMIT` can cut a tie, so that a difference in it is not settled.
#[must_use]
pub fn ties(query: &str) -> bool {
    TIED_AT_THE_CUT.contains(&query)
}

/// How an engine's answer stands against the published one, or nothing when they disagree.
#[must_use]
pub fn agrees(query: &str, got: &str) -> Option<Agreement> {
    crate::answer::agreement(answer(query)?, got)
}

#[cfg(test)]
mod tests {
    use super::{ANSWERS, agrees, answer, applies, scale, ties};
    use crate::answer::Agreement;
    use crate::suite::TPCH;

    /// The cardinalities the specification prints, which is the cheapest check there is on twenty
    /// two files of data entry. A file that lost its last row, gained a stray line or kept a header
    /// it should not have is a file with the wrong number of rows in it, and every one of those is
    /// a mistake somebody could make while replacing these with a set typed from the appendix.
    const ROWS: &[(&str, usize)] = &[
        ("q01", 4),
        ("q02", 100),
        ("q03", 10),
        ("q04", 5),
        ("q05", 5),
        ("q06", 1),
        ("q07", 4),
        ("q08", 2),
        ("q09", 175),
        ("q10", 20),
        ("q11", 1048),
        ("q12", 2),
        ("q13", 42),
        ("q14", 1),
        ("q15", 1),
        ("q16", 18314),
        ("q17", 1),
        ("q18", 57),
        ("q19", 1),
        ("q20", 186),
        ("q21", 100),
        ("q22", 7),
    ];

    fn rows(text: &str) -> usize {
        text.lines().filter(|line| !line.trim().is_empty()).count()
    }

    /// How many fields a CSV line holds, counting only the separators outside quotes, because two
    /// of the twenty two have a comma inside a supplier's address and one has one in a comment.
    fn fields(line: &str) -> usize {
        let mut quoted = false;
        let mut count = 1;
        for c in line.chars() {
            match c {
                '"' => quoted = !quoted,
                ',' if !quoted => count += 1,
                _ => {}
            }
        }
        count
    }

    #[test]
    fn every_query_in_the_suite_has_a_published_answer() {
        for query in TPCH {
            assert!(answer(query.name).is_some(), "{} has no answer committed", query.name);
        }
        assert_eq!(ANSWERS.len(), TPCH.len());
    }

    #[test]
    fn each_answer_has_the_number_of_rows_the_specification_prints() {
        for (query, expected) in ROWS {
            let text = answer(query).expect("committed");
            assert_eq!(rows(text), *expected, "{query}");
        }
    }

    #[test]
    fn the_column_names_come_off_and_the_first_row_does_not() {
        let q01 = answer("q01").expect("committed");
        assert!(!q01.starts_with("l_returnflag"), "the header is not a row");
        assert!(q01.starts_with("A,F,37734107,"), "and the first row still is one");
    }

    /// A row short of a field or carrying an extra one is the mistake a hand edit makes and the one
    /// the comparison would report as a wrong answer rather than as a broken fixture. The column
    /// names are the width, so the header earns its place twice.
    #[test]
    fn every_row_is_as_wide_as_the_column_names() {
        for (query, whole) in ANSWERS {
            let mut lines = whole.lines();
            let wide = fields(lines.next().expect("a header"));
            for (at, line) in lines.enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                assert_eq!(fields(line), wide, "{query} row {}", at + 1);
            }
        }
    }

    #[test]
    fn an_answer_agrees_with_itself_row_for_row() {
        for (query, _) in ANSWERS {
            let text = answer(query).expect("committed");
            assert_eq!(agrees(query, text), Some(Agreement::Ordered), "{query}");
        }
    }

    /// Not a tautology. The published answers print decimals to a fixed scale and the engines print
    /// whatever their writer prints, so `53758257134.8700` has to compare equal to
    /// `53758257134.87`, and it is the field by field number parsing rather than the text that
    /// makes it so.
    #[test]
    fn a_decimal_printed_to_a_different_scale_is_the_same_number() {
        let got = concat!(
            "A,F,37734107,56586554400.73,53758257134.87,55909065222.827692,",
            "25.522005853257337,38273.129734621674,0.049985295838397614,1478493\n",
            "N,F,991417,1487504710.38,1413082168.0541,1469649223.194375,",
            "25.516471920522985,38284.4677608483,0.0500934266742163,38854\n",
            "N,O,74476040,111701729697.74,106118230307.6056,110367043872.49701,",
            "25.50222676958499,38249.11798890827,0.04999658605370408,2920374\n",
            "R,F,37719753,56568041380.9,53741292684.604,55889619119.831932,",
            "25.50579361269077,38250.85462609966,0.05000940583012706,1478870\n",
        );
        assert_eq!(agrees("q01", got), Some(Agreement::Ordered));
    }

    #[test]
    fn a_wrong_number_anywhere_in_the_answer_is_a_disagreement() {
        let text = answer("q06").expect("committed");
        let wrong = text.replace('1', "2");
        assert_eq!(agrees("q06", &wrong), None);
    }

    #[test]
    fn a_query_nobody_published_an_answer_for_is_not_checked() {
        assert_eq!(answer("q23"), None);
        assert_eq!(agrees("q23", "anything"), None);
    }

    #[test]
    fn the_scale_comes_out_of_the_corpus_line() {
        let line = "duckdb-tpch SF1, corpus e02fbb7bb0145593, written 2026-09-18";
        assert_eq!(scale(line), Some("1"));
        assert_eq!(scale("dbgen SF100, corpus abc, written 2026-09-18"), Some("100"));
        assert_eq!(scale("nothing about a scale here"), None);
    }

    #[test]
    fn the_answers_are_used_on_tpch_at_sf1_and_nowhere_else() {
        let sf1 = Some("duckdb-tpch SF1, corpus e02fbb7bb0145593, written 2026-09-18");
        let sf100 = Some("duckdb-tpch SF100, corpus e02fbb7bb0145593, written 2026-09-18");
        assert!(applies("tpch", sf1));
        assert!(!applies("tpch", sf100), "the numbers are different at every other scale");
        assert!(!applies("clickbench", sf1), "and they are not about any other suite");
        assert!(!applies("tpch", None), "a run that does not say what it ran over is not checked");
    }

    #[test]
    fn the_five_queries_that_can_tie_at_their_limit_are_named() {
        for query in ["q02", "q03", "q10", "q18", "q21"] {
            assert!(ties(query), "{query} orders by keys that do not settle the cut");
        }
        assert!(!ties("q01"), "a query with no limit cannot tie at one");
        assert!(!ties("q05"));
    }
}
