//! Whether two engines answered the same question with the same answer.
//!
//! A benchmark that does not check answers is a benchmark of how fast a program can be wrong. It is
//! also the failure mode nobody notices, because a wrong answer is usually a fast one: a filter that
//! dropped too many rows, a join that lost a side, an aggregate over a column that came back empty.
//! Once four engines are in the same table, every one of them is a check on the other three at no
//! extra cost, so this module takes it.
//!
//! ## Why numbers and not text
//!
//! Because the text is four different things. DuckDB draws a box, ClickHouse writes tab separated,
//! DataFusion writes its own table, and Polars prints a data frame with a shape line on top. All
//! four are asked for CSV here, which removes most of that, but not all of it: one writes `6.8e6`
//! where another writes `6800000.0` and a third writes `6800000`, and a column of doubles summed in
//! a different order differs in the last bit or two.
//!
//! So what is compared is the multiset of numbers in the output, parsed, sorted and compared with a
//! relative tolerance. Sorted because two engines grouping the same data may emit the groups in
//! different orders, and a query with no `ORDER BY` has not asked for one. That is a real
//! weakening: this cannot catch a wrong sort. The suite's one ordered query has a `LIMIT` on it, so
//! a wrong sort changes which rows come back and the multiset changes with it.
//!
//! Strings come through as the numbers inside them, because the smoke data's only string column is
//! `tag-12` and friends. That is not a principled decision, it is a cheap one that happens to work
//! on the data at hand, and a suite with real text in it will need the comparison to grow.

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
    let (mut left, mut right) = (numbers(a), numbers(b));
    if left.len() != right.len() {
        return false;
    }
    left.sort_by(f64::total_cmp);
    right.sort_by(f64::total_cmp);
    left.iter().zip(&right).all(|(x, y)| close(*x, *y))
}

/// Two numbers that are the same number to within the tolerance.
fn close(a: f64, b: f64) -> bool {
    if a == b {
        return true;
    }
    let scale = a.abs().max(b.abs());
    (a - b).abs() <= TOLERANCE * scale
}

/// Every number in a block of output, in the order it appears.
///
/// Written as a scanner rather than as a split on punctuation, because the two things that break a
/// split are the two things every engine prints: an exponent, so that `1.5e-7` is one number and
/// not three, and a comma between fields, so that a CSV row is not one very long number.
#[must_use]
pub fn numbers(text: &str) -> Vec<f64> {
    let chars: Vec<char> = text.chars().collect();
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
            found.push(value);
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
    use super::{disagreements, numbers, same};

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
        // disagree on every row, which is how a checker gets switched off.
        assert_eq!(numbers("tag-12"), vec![12.0]);
        assert_eq!(numbers("count-3,-4"), vec![3.0, -4.0]);
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
        assert!(same("no rows", "empty"));
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

    #[test]
    fn one_engine_agrees_with_itself_and_nothing_is_reported() {
        assert!(disagreements(&[("duckdb".to_owned(), "1".to_owned())]).is_empty());
        assert!(disagreements(&[]).is_empty());
    }
}
