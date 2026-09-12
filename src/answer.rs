//! Whether two engines answered the same question with the same answer.
//!
//! A benchmark that does not check answers is a benchmark of how fast a program can be wrong. It is
//! also the failure mode nobody notices, because a wrong answer is usually a fast one: a filter that
//! dropped too many rows, a join that lost a side, an aggregate over a column that came back empty.
//! Once four engines are in the same table, every one of them is a check on the other three at no
//! extra cost, so this module takes it.
//!
//! ## Why the comparison is not a string comparison
//!
//! Because the text is four different things. DuckDB draws a box, ClickHouse writes tab separated,
//! DataFusion writes its own table, and Polars prints a data frame with a shape line on top. Three
//! of the four are asked for CSV here, which removes most of that, but not all of it: one writes
//! `6.8e6` where another writes `6800000.0` and a third writes `6800000`, and a column of doubles
//! summed in a different order differs in the last bit or two. The fourth prints a bordered table,
//! because its CSV writer drops rows, and a comparison that reads numbers out of the text rather
//! than fields out of a format is what makes that a footnote instead of a blocker.
//!
//! So what is compared is the multiset of numbers in the output, parsed, sorted and compared with a
//! relative tolerance. Sorted because two engines grouping the same data may emit the groups in
//! different orders, and a query with no `ORDER BY` has not asked for one. That is a real
//! weakening: this cannot catch a wrong sort. The suite's one ordered query has a `LIMIT` on it, so
//! a wrong sort changes which rows come back and the multiset changes with it.
//!
//! ## Fields, not a scan of the text
//!
//! The output is cut into fields first and each field is then either a number or it is not. This
//! used to be a scan over the whole block that took every run of digits it found, and the smoke
//! suite's only string column is `tag-12` and friends, so reading the twelve out of the tag was free
//! and nobody had to decide anything.
//!
//! ClickBench is where that stopped working. Its `Title` column is real text and real text has
//! digits in it, so q38's top ten page titles carry a `258`, a `650`, a `2008` and an `18` between
//! them, and two engines whose counts agreed exactly were reported as answering with fourteen
//! numbers against eleven. The disagreement was entirely in prose neither engine computed. That is
//! the same failure the bordered table header had, which is text being read as data, and it is
//! fixed the same way: decide what is a field, and only then decide what is a number.
//!
//! So a field whose whole text is a number is compared as a number, and a field that is not is
//! compared as text, exactly, after the quotes and the padding come off. That is stricter than what
//! it replaced rather than looser: a title that came back wrong used to be invisible unless it
//! happened to contain a different set of digits, and now it is a disagreement.

/// How far apart two numbers may be and still be the same number.
///
/// A double sum over ten million rows is associative in mathematics and not in a CPU, so two
/// engines that add the same column in a different order disagree in the last bits. That is not a
/// wrong answer and a checker that called it one would be turned off within a week, which is worse
/// than a checker with a tolerance in it.
const TOLERANCE: f64 = 1e-9;

/// Whether two answers say the same thing.
#[must_use]
pub fn same(a: &str, b: &str) -> bool {
    let (mut left, mut right) = (printed(a), printed(b));
    if left.len() != right.len() {
        return false;
    }
    left.sort_by(|x, y| f64::total_cmp(&x.0, &y.0));
    right.sort_by(|x, y| f64::total_cmp(&x.0, &y.0));
    if !left.iter().zip(&right).all(|(x, y)| close(*x, *y)) {
        return false;
    }
    // The text fields too, exactly. Sorted for the same reason the numbers are: two engines that
    // grouped the same data may hand the groups back in different orders and a query with no
    // `ORDER BY` did not ask for one.
    let (mut left, mut right) = (words(a), words(b));
    left.sort();
    right.sort();
    left == right
}

/// Two numbers that are the same number, to the last place the shorter of them was printed to.
///
/// The relative tolerance above is right when both engines printed a double. It is wrong when one of
/// them printed a decimal, and TPC-H is where that happens. `avg` over a `DECIMAL(15,2)` gives a
/// double in DuckDB, a decimal of scale six in DataFusion and a decimal of scale four in ClickHouse,
/// so q01's average quantity comes back as `25.499370423275426`, `25.499370` and `25.4993`. Dividing
/// two decimal sums in q08 does the same thing, and there the ClickHouse answer is four places wide.
/// All three engines agree on every digit any of them chose to print, and a relative tolerance of a
/// part in a billion called them three different answers for two runs in a row.
///
/// So a number printed to `f` decimal places is also allowed to differ in its last place, which is
/// `10^-f` and is an absolute quantity rather than a relative one. The wider of the two rules wins,
/// so nothing that used to pass now fails.
///
/// This is a weakening and the honest way to put it is that a digit an engine did not print is a
/// digit nobody can check. What keeps it from being a bad trade is that it applies to the decimal
/// places and not to the value: a number with no decimal point in it is compared at the relative
/// tolerance alone, so a count off by one in a hundred and forty eight million is still caught, and
/// an engine that returns a ratio of 0.041 where the rest return 0.0395 is still caught whatever
/// anybody printed.
fn close(a: (f64, Option<u32>), b: (f64, Option<u32>)) -> bool {
    if a.0 == b.0 {
        return true;
    }
    let scale = a.0.abs().max(b.0.abs());
    let mut allowed = TOLERANCE * scale;
    if let (Some(left), Some(right)) = (a.1, b.1) {
        allowed = allowed.max(10_f64.powi(-i32::try_from(left.min(right)).unwrap_or(i32::MAX)));
    }
    (a.0 - b.0).abs() <= allowed
}

/// How many decimal places a printed number carried, when it carried any.
///
/// `None` for a word with no decimal point, because an integer holds all of its digits, and for a
/// word written with an exponent, because the places after the point there are not the places of the
/// value and no engine here prints a decimal that way.
fn places(word: &str) -> Option<u32> {
    if word.contains(['e', 'E']) {
        return None;
    }
    let (_, fraction) = word.split_once('.')?;
    u32::try_from(fraction.chars().filter(char::is_ascii_digit).count()).ok()
}

/// A bordered table without its header, or the text unchanged when it is not one.
///
/// The header used to be harmless. The test below it says the borders carry no digits and neither
/// does `count(*)`, which was true of every column name in the smoke suite and is not true of
/// ClickBench. DataFusion names an unaliased expression after the expression, so
/// `SUM(ResolutionWidth + 1)` comes back as the column `sum(hits.ResolutionWidth + Int64(1))`, with
/// a sixty four in it and a one. q30 is ninety of those, so its header carries a hundred and
/// seventy eight numbers on top of the ninety in the answer, and the run reported DataFusion
/// returning 268 numbers against DuckDB's 90 and called it a disagreement. It was the harness
/// reading the column names as data.
///
/// Only the header goes. The trailing border and the row separators carry no digits, and dropping
/// everything down to the second border line is a rule that does not have to know what a cell looks
/// like.
fn body(text: &str) -> &str {
    let border = |line: &str| {
        let line = line.trim_end();
        line.starts_with('+') && line.ends_with('+') && line.chars().all(|c| c == '+' || c == '-')
    };
    let start = text.len() - text.trim_start().len();
    let table = &text[start..];
    if !border(table.lines().next().unwrap_or("")) {
        return text;
    }
    let mut seen = 0;
    let mut at = 0;
    for line in table.split_inclusive('\n') {
        at += line.len();
        if border(line) {
            seen += 1;
            if seen == 2 {
                return &table[at..];
            }
        }
    }
    text
}

/// Every field of every row, with the quoting and the padding taken off.
///
/// Three shapes of output reach this. CSV from the engines asked for it, which needs a split that
/// knows a comma inside quotes is not a separator. A bordered table from the one engine that draws
/// one, whose rows are cut by pipes and whose borders carry nothing. And a single column of values
/// with no separator at all, where the line is the field.
fn cells(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in body(text).lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // A border, which is every character being one of the two a border is drawn with.
        if trimmed.starts_with('+') && trimmed.chars().all(|c| c == '+' || c == '-') {
            continue;
        }
        if trimmed.starts_with('|') {
            out.extend(trimmed.trim_matches('|').split('|').map(tidy));
            continue;
        }
        out.extend(row(trimmed));
    }
    out.retain(|cell| !cell.is_empty());
    out
}

/// One CSV row, split on the commas that are not inside quotes.
fn row(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut rest = line.chars().peekable();
    while let Some(c) = rest.next() {
        match c {
            // A doubled quote inside a quoted field is one quote and not the end of the field.
            '"' if quoted && rest.peek() == Some(&'"') => {
                rest.next();
                field.push('"');
            }
            '"' => quoted = !quoted,
            ',' if !quoted => out.push(std::mem::take(&mut field)),
            other => field.push(other),
        }
    }
    out.push(field);
    out.into_iter().map(|f| f.trim().to_owned()).collect()
}

/// A cell out of a bordered table, without the padding or the quotes.
fn tidy(cell: &str) -> String {
    let trimmed = cell.trim();
    trimmed
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .map_or_else(|| trimmed.to_owned(), |inner| inner.replace("\"\"", "\""))
}

/// Every field that is not a number, which is the text half of an answer.
///
/// Empty fields are already gone, because an engine that prints a trailing separator and one that
/// does not are not disagreeing about anything.
#[must_use]
pub fn words(text: &str) -> Vec<String> {
    cells(text).into_iter().filter(|cell| one(cell).is_none()).collect()
}

/// Every number in a block of output, in the order it appears.
#[must_use]
pub fn numbers(text: &str) -> Vec<f64> {
    printed(text).into_iter().map(|(value, _)| value).collect()
}

/// Every number in a block of output, with how many decimal places it was printed to.
///
/// The place count is what the comparison needs and it is gone the moment the text is parsed, so it
/// comes out of the parse rather than being guessed back from the double afterwards.
#[must_use]
pub fn printed(text: &str) -> Vec<(f64, Option<u32>)> {
    cells(text).iter().filter_map(|cell| one(cell)).collect()
}

/// A field that is a number, and nothing else.
///
/// The whole field or nothing. `258` is a number and `Brooks 258 Ltd` is a name with a number inside
/// it, and the difference between those two is the whole reason this function exists.
fn one(cell: &str) -> Option<(f64, Option<u32>)> {
    let value: f64 = cell.parse().ok()?;
    // `inf` and `nan` parse and are not numbers any engine here computed, so a column of them would
    // otherwise compare equal to a column of them and hide whatever produced it.
    value.is_finite().then(|| (value, places(cell)))
}

/// Every run of digits in a block of output, wherever it sits.
///
/// What the comparison used to do. It is not what the comparison does any more and it is kept
/// because it is still the right rule for a line that is not a table: a version string, a count an
/// engine printed in a sentence, a `tag-12` in the smoke data. Nothing that compares two engines
/// goes through here, because reading a number out of the middle of a word is exactly what made
/// two agreeing engines look like they disagreed.
#[must_use]
pub fn scan(text: &str) -> Vec<(f64, Option<u32>)> {
    let chars: Vec<char> = body(text).chars().collect();
    let mut found = Vec::new();
    let mut at = 0;

    while at < chars.len() {
        if !chars[at].is_ascii_digit() {
            at += 1;
            continue;
        }
        // Back up over a sign and over a leading decimal point, so that `-.5` is one number. A sign
        // only counts when what is in front of it is not a word, because `tag-12` is a name with a
        // twelve in it and not a name and a negative twelve.
        let mut start = at;
        while start > 0 && (chars[start - 1].is_ascii_digit() || chars[start - 1] == '.') {
            start -= 1;
        }
        if start > 0 && (chars[start - 1] == '-' || chars[start - 1] == '+') {
            let before = if start >= 2 { Some(chars[start - 2]) } else { None };
            if before.is_none_or(|c| !c.is_alphanumeric() && c != '.') {
                start -= 1;
            }
        }

        let mut end = at;
        while end < chars.len() && (chars[end].is_ascii_digit() || chars[end] == '.') {
            end += 1;
        }
        // An exponent, but only a real one. `1e6` has one and `tag1e` does not.
        if end < chars.len() && (chars[end] == 'e' || chars[end] == 'E') {
            let mut after = end + 1;
            if after < chars.len() && (chars[after] == '-' || chars[after] == '+') {
                after += 1;
            }
            if after < chars.len() && chars[after].is_ascii_digit() {
                while after < chars.len() && chars[after].is_ascii_digit() {
                    after += 1;
                }
                end = after;
            }
        }

        let word: String = chars[start..end].iter().collect();
        if let Ok(value) = word.parse::<f64>() {
            found.push((value, places(&word)));
        }
        at = end.max(at + 1);
    }

    found
}

/// Which engines disagreed with the first one, given what each of them answered.
///
/// The first is the reference rather than a vote, because a majority of three engines agreeing is
/// not evidence when the fourth is the one being developed. DuckDB goes first, and the sentence
/// this produces names it, so a reader knows which side of a disagreement is the claim.
#[must_use]
pub fn disagreements(answers: &[(String, String)]) -> Vec<String> {
    let Some((reference, expected)) = answers.first() else { return Vec::new() };
    let mut out = Vec::new();
    for (engine, got) in answers.iter().skip(1) {
        if !same(expected, got) {
            out.push(format!(
                "{engine} does not agree with {reference}: {} numbers against {}",
                numbers(got).len(),
                numbers(expected).len()
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{disagreements, numbers, same, scan, words};

    #[test]
    fn a_csv_row_is_its_fields_and_not_one_long_number() {
        assert_eq!(numbers("1,2,3"), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn an_exponent_is_part_of_the_number_it_belongs_to() {
        assert_eq!(numbers("6.8e6"), vec![6_800_000.0]);
        assert_eq!(numbers("1.5e-7"), vec![1.5e-7]);
        assert_eq!(numbers("2E+3"), vec![2000.0]);
    }

    #[test]
    fn the_same_number_written_three_ways_is_the_same_number() {
        assert!(same("6800000", "6.8e6"));
        assert!(same("6800000.0", "6.8e6"));
    }

    #[test]
    fn a_dash_inside_a_word_is_not_a_minus_sign() {
        // Otherwise every tag in the smoke data flips sign and two engines that agree perfectly
        // disagree on every row, which is how a checker gets switched off. This is the scanner's
        // rule and the scanner is what reads a line that is not a table.
        assert_eq!(scan("tag-12").len(), 1);
        assert!((scan("tag-12")[0].0 - 12.0).abs() < f64::EPSILON);
        assert_eq!(numbers("count-3,-4"), vec![-4.0]);
    }

    #[test]
    fn a_number_inside_a_name_is_part_of_the_name_and_not_a_number() {
        // ClickBench q38 is the top ten page titles, and real titles have digits in them. Two
        // engines whose counts agreed exactly were reported as fourteen numbers against eleven,
        // and the whole difference was digits inside Russian prose neither engine computed.
        assert_eq!(numbers("\"Brooks 258 Ltd\",120"), vec![120.0]);
        assert_eq!(words("\"Brooks 258 Ltd\",120"), vec!["Brooks 258 Ltd".to_owned()]);
    }

    #[test]
    fn a_title_that_came_back_wrong_is_a_disagreement_now() {
        // The other half of the same change, and the reason it is not a weakening. Under the old
        // scan these two agreed, because the only digits in either of them were the counts.
        assert!(!same("\"Yandex\",120", "\"Google\",120"));
        assert!(same("\"Yandex\",120", "\"Yandex\",120"));
    }

    #[test]
    fn a_comma_inside_a_quoted_title_is_not_the_end_of_the_field() {
        // Which is most of why the split has to know about quotes at all. DuckDB quotes a title
        // with a comma in it and a naive split would make it two fields and two engines that both
        // returned it would then differ on how many fields there were.
        assert_eq!(words("\"Shop, sell and buy\",7"), vec!["Shop, sell and buy".to_owned()]);
        assert_eq!(numbers("\"Shop, sell and buy\",7"), vec![7.0]);
    }

    #[test]
    fn a_leading_minus_at_the_start_of_a_field_is_a_minus_sign() {
        assert_eq!(numbers("-7"), vec![-7.0]);
        assert_eq!(numbers("a,-7"), vec![-7.0]);
    }

    #[test]
    fn two_engines_that_grouped_in_different_orders_still_agree() {
        assert!(same("tag-1,10\ntag-2,20", "tag-2,20\ntag-1,10"));
    }

    #[test]
    fn a_bordered_table_says_the_same_thing_as_the_csv_of_it() {
        // This is why the datafusion column may print a table while the other three print CSV.
        // The borders carry no digits, the header carries no digits, and `count(*)` is not a
        // number with a star in it.
        let table = "+--------+----------+\n\
                     | tag    | count(*) |\n\
                     +--------+----------+\n\
                     | tag-1  | 10       |\n\
                     | tag-2  | 20       |\n\
                     +--------+----------+";
        assert!(same("tag-1,10\ntag-2,20", table));
    }

    #[test]
    fn a_column_name_with_a_number_in_it_is_a_name_and_not_an_answer() {
        // What q30 of ClickBench looks like out of datafusion-cli, cut down to three columns. The
        // header alone carries six numbers, two per aliasless sum, and reading them as data is how
        // the first full run reported 268 numbers against 90.
        let table = "+--------+-------------------+-------------------+\n\
                     | sum(x) | sum(x + Int64(1)) | sum(x + Int64(2)) |\n\
                     +--------+-------------------+-------------------+\n\
                     | 5      | 6                 | 7                 |\n\
                     +--------+-------------------+-------------------+";
        assert_eq!(numbers(table), vec![5.0, 6.0, 7.0]);
        assert!(same("5,6,7", table));
    }

    #[test]
    fn text_that_is_not_a_table_keeps_its_first_line() {
        // The rule is only allowed to fire on a bordered table, because every other engine here
        // writes headerless CSV and dropping two lines of that would drop two rows of the answer.
        assert_eq!(numbers("1,2\n3,4"), vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(numbers("+1,2\n3,4"), vec![1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn a_sum_that_differs_in_the_last_bits_is_not_a_wrong_answer() {
        assert!(same("6800000.0000001", "6800000.0000002"));
    }

    #[test]
    fn a_sum_that_differs_in_the_first_digits_is_a_wrong_answer() {
        assert!(!same("6800000", "6900000"));
    }

    #[test]
    fn an_engine_that_lost_half_its_rows_does_not_agree() {
        assert!(!same("1,2\n3,4", "1,2"));
    }

    #[test]
    fn no_numbers_at_all_on_both_sides_is_agreement_rather_than_a_crash() {
        assert!(same("", ""));
        assert!(same("no rows", "no rows"));
        // Two engines printing two different words for the same nothing is now a disagreement and
        // it used to be agreement. That is the strictness coming the other way and it is wanted: a
        // text column is a column, and an engine that returned different text returned a different
        // answer whether or not the difference had a digit in it.
        assert!(!same("no rows", "empty"));
    }

    #[test]
    fn a_disagreement_names_both_sides_and_the_shape_of_the_difference() {
        let answers = vec![
            ("duckdb".to_owned(), "10000000".to_owned()),
            ("polars".to_owned(), "10000000".to_owned()),
            ("clickhouse-local".to_owned(), "9999999".to_owned()),
        ];
        let found = disagreements(&answers);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("clickhouse-local"), "{}", found[0]);
        assert!(found[0].contains("duckdb"), "{}", found[0]);
    }

    /// The three TPC-H q01 answers, as the three engines actually printed them at scale factor 100.
    #[test]
    fn a_decimal_average_and_a_double_average_of_the_same_column_agree() {
        let duckdb = "A,F,25.499370423275426,38236.11698430489,0.050002243530929025";
        let datafusion = "A,F,25.499370,38236.116984,0.050002";
        let clickhouse = "A,F,25.4993,38236.1169,0.0500";
        assert!(same(duckdb, datafusion));
        assert!(same(duckdb, clickhouse));
        assert!(same(datafusion, clickhouse));
    }

    /// q08 is a division of two sums, and the three engines pick three different result scales.
    #[test]
    fn a_ratio_printed_to_four_places_agrees_with_the_same_ratio_printed_to_eighteen() {
        assert!(same("1995,0.039535108776109315", "1995,0.03953510"));
        assert!(same("1995,0.039535108776109315", "1995,0.0395"));
        // And a ratio that is genuinely a different ratio is still a different answer, however few
        // places either side printed.
        assert!(!same("1995,0.039535108776109315", "1995,0.0410"));
        assert!(!same("1995,0.0395", "1995,0.0410"));
    }

    /// The weakening applies to the decimal places and not to the value, and this is where that
    /// distinction earns its keep.
    #[test]
    fn a_count_that_is_off_by_one_is_still_a_wrong_answer_however_large_the_count() {
        assert!(!same("148047881", "148047882"));
        assert!(!same("1,148047881", "1,148047882"));
    }

    #[test]
    fn one_engine_agrees_with_itself_and_nothing_is_reported() {
        assert!(disagreements(&[("duckdb".to_owned(), "1".to_owned())]).is_empty());
        assert!(disagreements(&[]).is_empty());
    }
}
