//! The answers TPC-H publishes for its own queries at scale factor one, and the JOB answers the
//! pinned DuckDB gives on the IMDb snapshot.
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

/// The answer to each of the 113 JOB queries on the May 2013 IMDb snapshot, in query order.
///
/// These are not published by anybody. They are what the pinned DuckDB (v2.0.0-dev84237) returned
/// on server2 on 24 September 2026 over a load of `imdb.tgz` from JOB's own `schema.sql`, and
/// `answers/README.md` says more. So this is one engine's answer rather than an
/// independent one, the same weakness the TPC-H set has, and it earns its place the same way: rudb
/// had no part in producing it. JOB is kind to a stored answer, because every query returns one row
/// with no `LIMIT` and no floating point aggregate, so there is nothing to tie and nothing to round.
///
/// A `NULL` is spelled out in the files so that a reader can see it. The engines print it as an
/// empty field, so [`answer`] turns it back into one.
const JOB_ANSWERS: &[(&str, &str)] = &[
    ("1a", include_str!("../answers/job/1a.csv")),
    ("1b", include_str!("../answers/job/1b.csv")),
    ("1c", include_str!("../answers/job/1c.csv")),
    ("1d", include_str!("../answers/job/1d.csv")),
    ("2a", include_str!("../answers/job/2a.csv")),
    ("2b", include_str!("../answers/job/2b.csv")),
    ("2c", include_str!("../answers/job/2c.csv")),
    ("2d", include_str!("../answers/job/2d.csv")),
    ("3a", include_str!("../answers/job/3a.csv")),
    ("3b", include_str!("../answers/job/3b.csv")),
    ("3c", include_str!("../answers/job/3c.csv")),
    ("4a", include_str!("../answers/job/4a.csv")),
    ("4b", include_str!("../answers/job/4b.csv")),
    ("4c", include_str!("../answers/job/4c.csv")),
    ("5a", include_str!("../answers/job/5a.csv")),
    ("5b", include_str!("../answers/job/5b.csv")),
    ("5c", include_str!("../answers/job/5c.csv")),
    ("6a", include_str!("../answers/job/6a.csv")),
    ("6b", include_str!("../answers/job/6b.csv")),
    ("6c", include_str!("../answers/job/6c.csv")),
    ("6d", include_str!("../answers/job/6d.csv")),
    ("6e", include_str!("../answers/job/6e.csv")),
    ("6f", include_str!("../answers/job/6f.csv")),
    ("7a", include_str!("../answers/job/7a.csv")),
    ("7b", include_str!("../answers/job/7b.csv")),
    ("7c", include_str!("../answers/job/7c.csv")),
    ("8a", include_str!("../answers/job/8a.csv")),
    ("8b", include_str!("../answers/job/8b.csv")),
    ("8c", include_str!("../answers/job/8c.csv")),
    ("8d", include_str!("../answers/job/8d.csv")),
    ("9a", include_str!("../answers/job/9a.csv")),
    ("9b", include_str!("../answers/job/9b.csv")),
    ("9c", include_str!("../answers/job/9c.csv")),
    ("9d", include_str!("../answers/job/9d.csv")),
    ("10a", include_str!("../answers/job/10a.csv")),
    ("10b", include_str!("../answers/job/10b.csv")),
    ("10c", include_str!("../answers/job/10c.csv")),
    ("11a", include_str!("../answers/job/11a.csv")),
    ("11b", include_str!("../answers/job/11b.csv")),
    ("11c", include_str!("../answers/job/11c.csv")),
    ("11d", include_str!("../answers/job/11d.csv")),
    ("12a", include_str!("../answers/job/12a.csv")),
    ("12b", include_str!("../answers/job/12b.csv")),
    ("12c", include_str!("../answers/job/12c.csv")),
    ("13a", include_str!("../answers/job/13a.csv")),
    ("13b", include_str!("../answers/job/13b.csv")),
    ("13c", include_str!("../answers/job/13c.csv")),
    ("13d", include_str!("../answers/job/13d.csv")),
    ("14a", include_str!("../answers/job/14a.csv")),
    ("14b", include_str!("../answers/job/14b.csv")),
    ("14c", include_str!("../answers/job/14c.csv")),
    ("15a", include_str!("../answers/job/15a.csv")),
    ("15b", include_str!("../answers/job/15b.csv")),
    ("15c", include_str!("../answers/job/15c.csv")),
    ("15d", include_str!("../answers/job/15d.csv")),
    ("16a", include_str!("../answers/job/16a.csv")),
    ("16b", include_str!("../answers/job/16b.csv")),
    ("16c", include_str!("../answers/job/16c.csv")),
    ("16d", include_str!("../answers/job/16d.csv")),
    ("17a", include_str!("../answers/job/17a.csv")),
    ("17b", include_str!("../answers/job/17b.csv")),
    ("17c", include_str!("../answers/job/17c.csv")),
    ("17d", include_str!("../answers/job/17d.csv")),
    ("17e", include_str!("../answers/job/17e.csv")),
    ("17f", include_str!("../answers/job/17f.csv")),
    ("18a", include_str!("../answers/job/18a.csv")),
    ("18b", include_str!("../answers/job/18b.csv")),
    ("18c", include_str!("../answers/job/18c.csv")),
    ("19a", include_str!("../answers/job/19a.csv")),
    ("19b", include_str!("../answers/job/19b.csv")),
    ("19c", include_str!("../answers/job/19c.csv")),
    ("19d", include_str!("../answers/job/19d.csv")),
    ("20a", include_str!("../answers/job/20a.csv")),
    ("20b", include_str!("../answers/job/20b.csv")),
    ("20c", include_str!("../answers/job/20c.csv")),
    ("21a", include_str!("../answers/job/21a.csv")),
    ("21b", include_str!("../answers/job/21b.csv")),
    ("21c", include_str!("../answers/job/21c.csv")),
    ("22a", include_str!("../answers/job/22a.csv")),
    ("22b", include_str!("../answers/job/22b.csv")),
    ("22c", include_str!("../answers/job/22c.csv")),
    ("22d", include_str!("../answers/job/22d.csv")),
    ("23a", include_str!("../answers/job/23a.csv")),
    ("23b", include_str!("../answers/job/23b.csv")),
    ("23c", include_str!("../answers/job/23c.csv")),
    ("24a", include_str!("../answers/job/24a.csv")),
    ("24b", include_str!("../answers/job/24b.csv")),
    ("25a", include_str!("../answers/job/25a.csv")),
    ("25b", include_str!("../answers/job/25b.csv")),
    ("25c", include_str!("../answers/job/25c.csv")),
    ("26a", include_str!("../answers/job/26a.csv")),
    ("26b", include_str!("../answers/job/26b.csv")),
    ("26c", include_str!("../answers/job/26c.csv")),
    ("27a", include_str!("../answers/job/27a.csv")),
    ("27b", include_str!("../answers/job/27b.csv")),
    ("27c", include_str!("../answers/job/27c.csv")),
    ("28a", include_str!("../answers/job/28a.csv")),
    ("28b", include_str!("../answers/job/28b.csv")),
    ("28c", include_str!("../answers/job/28c.csv")),
    ("29a", include_str!("../answers/job/29a.csv")),
    ("29b", include_str!("../answers/job/29b.csv")),
    ("29c", include_str!("../answers/job/29c.csv")),
    ("30a", include_str!("../answers/job/30a.csv")),
    ("30b", include_str!("../answers/job/30b.csv")),
    ("30c", include_str!("../answers/job/30c.csv")),
    ("31a", include_str!("../answers/job/31a.csv")),
    ("31b", include_str!("../answers/job/31b.csv")),
    ("31c", include_str!("../answers/job/31c.csv")),
    ("32a", include_str!("../answers/job/32a.csv")),
    ("32b", include_str!("../answers/job/32b.csv")),
    ("33a", include_str!("../answers/job/33a.csv")),
    ("33b", include_str!("../answers/job/33b.csv")),
    ("33c", include_str!("../answers/job/33c.csv")),
];

/// The five queries whose `ORDER BY` does not settle which rows the `LIMIT` keeps.
///
/// Document 04 section 4.4 names them and they are hard coded rather than worked out, because the
/// query set is fixed by the specification and will not grow a twenty third.
const TIED_AT_THE_CUT: &[&str] = &["q02", "q03", "q10", "q18", "q21"];

/// The committed answer for a query, ready to compare.
///
/// The column names come off here. The file keeps them because a fixture nobody can read is a
/// fixture nobody will check, and every engine in this harness is asked for CSV with no header, so
/// leaving the line on would make the reference one row longer than any engine's.
///
/// The two suites share one lookup because their query names cannot collide: TPC-H's are `q01` to
/// `q22` and JOB's a number and a letter.
#[must_use]
pub fn answer(query: &str) -> Option<String> {
    if let Some((_, text)) = ANSWERS.iter().find(|(name, _)| *name == query) {
        return text.split_once('\n').map(|(_, rows)| rows.to_owned());
    }
    let (_, text) = JOB_ANSWERS.iter().find(|(name, _)| *name == query)?;
    let (_, rows) = text.split_once('\n')?;
    Some(rows.lines().map(unspell).collect::<Vec<_>>().join("\n") + "\n")
}

/// A JOB answer line with each unquoted `NULL` field printed the way `-csv` prints a null, which is
/// as nothing at all.
fn unspell(line: &str) -> String {
    let mut out = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    for c in line.chars() {
        match c {
            '"' => {
                quoted = !quoted;
                field.push(c);
            }
            ',' if !quoted => out.push(std::mem::take(&mut field)),
            _ => field.push(c),
        }
    }
    out.push(field);
    out.iter().map(|f| if f == "NULL" { "" } else { f.as_str() }).collect::<Vec<_>>().join(",")
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
/// For TPC-H two conditions and both are necessary. The wrong suite has no answers here at all. The
/// right suite at the wrong scale has answers that are all wrong, which is worse, because a check
/// that fails on every query is a check that gets turned off rather than read.
///
/// For JOB the condition is that the corpus was unpacked from the IMDb archive, which is the one
/// thing a JOB corpus can be. A run that does not say what it ran over is not checked.
#[must_use]
pub fn applies(suite: &str, corpus: Option<&str>) -> bool {
    match suite {
        "tpch" => corpus.and_then(scale) == Some("1"),
        "job" => corpus.is_some_and(|line| line.starts_with("imdb-archive,")),
        _ => false,
    }
}

/// What the reference is, for the sentence that introduces a query that does not match it.
#[must_use]
pub fn reference(suite: &str) -> &'static str {
    if suite == "job" {
        "the answer committed in answers/job, which the pinned DuckDB gave on the IMDb snapshot"
    } else {
        "the answer TPC-H publishes for them at SF1"
    }
}

/// Whether a query's `LIMIT` can cut a tie, so that a difference in it is not settled.
#[must_use]
pub fn ties(query: &str) -> bool {
    TIED_AT_THE_CUT.contains(&query)
}

/// How an engine's answer stands against the published one, or nothing when they disagree.
#[must_use]
pub fn agrees(query: &str, got: &str) -> Option<Agreement> {
    crate::answer::agreement(&answer(query)?, got)
}

#[cfg(test)]
mod tests {
    use super::{ANSWERS, JOB_ANSWERS, agrees, answer, applies, scale, ties};
    use crate::answer::Agreement;
    use crate::suite::{JOB, TPCH};

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
            assert_eq!(rows(&text), *expected, "{query}");
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
        for (query, _) in ANSWERS.iter().chain(JOB_ANSWERS) {
            let text = answer(query).expect("committed");
            assert_eq!(agrees(query, &text), Some(Agreement::Ordered), "{query}");
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

    #[test]
    fn every_job_query_has_one_answer_row_under_its_column_names() {
        for query in JOB {
            let whole = JOB_ANSWERS
                .iter()
                .find(|(name, _)| *name == query.name)
                .map(|(_, text)| *text)
                .unwrap_or_else(|| panic!("{} has no answer committed", query.name));
            let mut lines = whole.lines().filter(|line| !line.trim().is_empty());
            let wide = fields(lines.next().expect("a header"));
            // 7c's answer is a biography with commas inside quotes, which is why this counts
            // separators outside quotes rather than splitting.
            assert_eq!(lines.clone().count(), 1, "{}", query.name);
            assert_eq!(fields(lines.next().expect("a row")), wide, "{}", query.name);
        }
        assert_eq!(JOB_ANSWERS.len(), JOB.len());
    }

    /// The five empty queries answer one row of nulls, which `-csv` prints as nothing but commas,
    /// and an engine that did that has to agree with a file that spells the nulls out.
    #[test]
    fn a_row_of_nulls_is_what_the_engines_print_for_one() {
        assert_eq!(answer("2c").as_deref(), Some("\n"));
        assert_eq!(agrees("2c", "\n"), Some(Agreement::Ordered));
        assert_eq!(agrees("5a", ",,\n"), Some(Agreement::Ordered));
        assert_eq!(agrees("5a", "x,,\n"), None, "a value where the answer has a null is wrong");
    }

    #[test]
    fn a_job_answer_is_checked_only_over_the_imdb_archive() {
        let unpacked = Some("imdb-archive, corpus 0123456789abcdef, written 2026-09-25");
        assert!(applies("job", unpacked));
        assert!(!applies("job", None));
        assert!(!applies(
            "job",
            Some("duckdb-tpch SF1, corpus e02fbb7bb0145593, written 2026-09-18")
        ));
        assert!(!applies("tpch", unpacked), "the TPC-H answers are not about IMDb");
    }
}
