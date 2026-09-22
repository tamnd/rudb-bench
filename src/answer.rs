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
//!
//! ## The three steps, in that order and only in that order
//!
//! `spec/bench/tpc-h/04-the-answers.md` section 4.4 sets the rule, and it is a ladder rather than a
//! choice. Compare the rows in the order both engines produced them. If that fails, sort both and
//! compare as multisets, which is a tie broken two ways and is recorded as [`Agreement::Tied`]
//! rather than as a failure. If that fails, look at the boundary: two engines that cut a tie at a
//! `LIMIT` kept different rows, and what can still be checked is the part the tie did not reach.
//! Anything else is a failure.
//!
//! The order is the whole point. Sorting first would hide a wrong sort, which is a real class of
//! bug and one rudb has a `TopN` operator for, so the ordered comparison has to be tried and has to
//! be tried first. Going straight to the boundary would call a wrong answer a tie.
//!
//! Row by row, and not the flat multiset of fields it used to be. Two engines that returned the
//! same values arranged into different rows have not returned the same answer, and under the flat
//! comparison they did.

use std::cmp::Ordering;

/// How far apart two numbers may be and still be the same number.
///
/// A double sum over ten million rows is associative in mathematics and not in a CPU, so two
/// engines that add the same column in a different order disagree in the last bits. That is not a
/// wrong answer and a checker that called it one would be turned off within a week, which is worse
/// than a checker with a tolerance in it.
const TOLERANCE: f64 = 1e-9;

/// How two answers agreed, when they agreed.
///
/// Three states rather than a boolean, because the three are not the same claim. The first says the
/// engines produced the same rows in the same order, which is everything the query asked for. The
/// second says they produced the same rows and put them in a different order, which an `ORDER BY`
/// that does not totally order the rows permits and which nobody should be failed for. The third
/// says a `LIMIT` cut a tie and the engines kept different halves of it, so what was checked is the
/// part before the cut and how many rows came back, and a reader is owed that sentence rather than
/// a tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Agreement {
    /// Row for row, in the order both of them produced them.
    Ordered,
    /// The same rows in a different order.
    Tied,
    /// The rows the tie did not reach, and the count, with a `LIMIT` cutting the rest.
    Boundary,
}

impl Agreement {
    /// The word a report prints for it.
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Self::Ordered => "ordered",
            Self::Tied => "tied",
            Self::Boundary => "boundary",
        }
    }
}

/// Whether two answers say the same thing.
///
/// The first two steps of the ladder. The third needs the same query run again without its `LIMIT`,
/// which is a thing only the harness can do, so it is [`boundary`] and the caller reaches for it
/// when this says no.
#[must_use]
pub fn agreement(a: &str, b: &str) -> Option<Agreement> {
    let (mut left, mut right) = (table(a), table(b));
    // Different row counts is not a tie about anything. Two engines that cut the same `LIMIT` at
    // the same place return the same number of rows whichever rows they chose.
    if left.len() != right.len() {
        return None;
    }
    if left.iter().zip(&right).all(|(x, y)| alike(x, y)) {
        return Some(Agreement::Ordered);
    }
    left.sort_by(|x, y| order(x, y));
    right.sort_by(|x, y| order(x, y));
    left.iter().zip(&right).all(|(x, y)| alike(x, y)).then_some(Agreement::Tied)
}

/// Whether two answers say the same thing, by any of the rules that count as saying it.
#[must_use]
pub fn same(a: &str, b: &str) -> bool {
    agreement(a, b).is_some()
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

/// Every row of an answer, as its fields, with the quoting and the padding taken off.
///
/// Three shapes of output reach this. CSV from the engines asked for it, which needs a split that
/// knows a comma inside quotes is not a separator. A bordered table from the one engine that draws
/// one, whose rows are cut by pipes and whose borders carry nothing. And a single column of values
/// with no separator at all, where the line is the field.
///
/// Empty fields go, which is one engine printing a trailing separator and another not, and a row
/// left with nothing in it goes with them.
fn table(text: &str) -> Vec<Vec<String>> {
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
        let mut fields: Vec<String> = if trimmed.starts_with('|') {
            trimmed.trim_matches('|').split('|').map(tidy).collect()
        } else {
            row(trimmed)
        };
        fields.retain(|field| !field.is_empty());
        if !fields.is_empty() {
            out.push(fields);
        }
    }
    out
}

/// Every field of every row, in the order they appear.
fn cells(text: &str) -> Vec<String> {
    table(text).into_iter().flatten().collect()
}

/// Whether two fields say the same thing, which is by value when both are numbers and exactly when
/// neither is.
///
/// An engine that printed a number where another printed a word disagrees whatever the two say. The
/// one case that reaches this is a null rendered as a word by one engine and as nothing by the
/// other, and the nothing is already gone by the time anything gets here, so the rows are a
/// different width and never reach the field comparison at all.
fn matches(a: &str, b: &str) -> bool {
    match (one(a), one(b)) {
        (Some(x), Some(y)) => close(x, y),
        (None, None) => a == b,
        _ => false,
    }
}

/// Whether two rows say the same thing, field for field.
fn alike(a: &[String], b: &[String]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| matches(x, y))
}

/// A total order over rows, by every column in turn, which is what the multiset step sorts by.
///
/// Numbers before words at the same position, and numbers by value rather than by their text, so
/// that `6.8e6` and `6800000` land next to each other rather than a page apart.
fn order(a: &[String], b: &[String]) -> Ordering {
    for (x, y) in a.iter().zip(b) {
        let by = match (one(x), one(y)) {
            (Some(p), Some(q)) => f64::total_cmp(&p.0, &q.0),
            (None, None) => x.cmp(y),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
        };
        if by != Ordering::Equal {
            return by;
        }
    }
    a.len().cmp(&b.len())
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

/// What a query asked for at the end, which is what decides whether a tie can change the rows.
///
/// Read out of the SQL rather than configured per query, because the suites are somebody else's
/// query sets and a list of which of their queries have a `LIMIT` is a list that goes stale on the
/// day they change one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cut {
    /// How many expressions the `ORDER BY` names, which is how many leading fields of a row are the
    /// sort key.
    ///
    /// Zero when there is no `ORDER BY`, and a query with no `ORDER BY` has no boundary: it did not
    /// ask for an order, so there is nothing for a tie to be a tie in.
    pub keys: usize,
    /// How many rows the `LIMIT` asked for, when it asked for any.
    pub limit: Option<usize>,
}

/// The `ORDER BY` width and the `LIMIT` of a statement.
///
/// Only the outermost ones. An `ORDER BY` inside a window frame or a subquery is inside brackets,
/// so counting bracket depth is enough to tell the query's own from everything that looks like it,
/// and both of the suites here put theirs at the end of the statement where this expects it.
#[must_use]
pub fn cut(sql: &str) -> Cut {
    let lower = sql.to_lowercase();
    let text: Vec<char> = lower.chars().collect();
    let mut depth = 0i32;
    let mut order_at = None;
    let mut limit_at = None;
    let mut at = 0;
    while at < text.len() {
        match text[at] {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ if depth == 0 => {
                if word_at(&text, at, "order") && after(&text, at + 5, "by").is_some() {
                    order_at = after(&text, at + 5, "by");
                    limit_at = None;
                } else if word_at(&text, at, "limit") {
                    limit_at = Some(at + 5);
                }
            }
            _ => {}
        }
        at += 1;
    }
    let keys = order_at.map_or(0, |from| {
        let to = limit_at.map_or(text.len(), |limit| limit.saturating_sub(5));
        1 + commas(&text[from..to.max(from)])
    });
    Cut { keys, limit: limit_at.and_then(|from| count(&text[from..])) }
}

/// Whether the word `want` starts at `at` and is a word rather than part of one.
fn word_at(text: &[char], at: usize, want: &str) -> bool {
    if at > 0 && (text[at - 1].is_alphanumeric() || text[at - 1] == '_') {
        return false;
    }
    let end = at + want.len();
    if end > text.len() || !text[at..end].iter().copied().eq(want.chars()) {
        return false;
    }
    text.get(end).is_none_or(|&c| !c.is_alphanumeric() && c != '_')
}

/// Where the text after `want` starts, when `want` is the next word after `at`.
fn after(text: &[char], at: usize, want: &str) -> Option<usize> {
    let start = at + text[at.min(text.len())..].iter().take_while(|c| c.is_whitespace()).count();
    word_at(text, start, want).then_some(start + want.len())
}

/// How many commas the text holds outside brackets, which is one less than the number of
/// expressions in a list.
fn commas(text: &[char]) -> usize {
    let mut depth = 0i32;
    let mut found = 0;
    for &c in text {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => found += 1,
            _ => {}
        }
    }
    found
}

/// The first whole number in the text, which is what follows a `LIMIT`.
fn count(text: &[char]) -> Option<usize> {
    let digits: String =
        text.iter().skip_while(|c| c.is_whitespace()).take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

/// The third step, which needs the same two queries run again without their `LIMIT`.
///
/// Only reached when [`agreement`] has already said no, which is what makes it affordable: running
/// a TPC-H query at SF100 without its `LIMIT` is expensive and it should never happen.
///
/// The question it asks is whether both engines were forced to choose. Take the sort key of the
/// last row each of them returned. If the two keys differ, the engines stopped at different values
/// and that is a wrong answer rather than a tie. If they agree, count how many rows of each
/// engine's unlimited result carry that key: when more of them carry it than there was room for
/// under the `LIMIT`, that engine had more candidates than places and picked. Both having picked is
/// what a boundary tie is.
///
/// What is then checked is what the tie did not touch: the rows whose key is not the boundary key,
/// as a multiset, and the number of rows that came back. That is the deterministic part of the
/// answer, and it is the whole answer for every query where the boundary is not tied.
#[must_use]
pub fn boundary(a: &str, b: &str, whole_a: &str, whole_b: &str, cut: Cut) -> Option<Agreement> {
    let limit = cut.limit?;
    if cut.keys == 0 {
        return None;
    }
    let (left, right) = (table(a), table(b));
    if left.len() != right.len() || left.is_empty() {
        return None;
    }
    let (edge_left, edge_right) = (key(left.last()?, cut.keys), key(right.last()?, cut.keys));
    if !alike(&edge_left, &edge_right) {
        return None;
    }
    // The places the tie had to fill, which is the `LIMIT` less the rows that got in on their own.
    let settled = |rows: &[Vec<String>], edge: &[String]| -> Vec<Vec<String>> {
        rows.iter().filter(|row| !alike(&key(row, cut.keys), edge)).cloned().collect()
    };
    let (before_left, before_right) = (settled(&left, &edge_left), settled(&right, &edge_right));
    let room = limit.saturating_sub(before_left.len());
    if sharing(whole_a, &edge_left, cut.keys) <= room
        || sharing(whole_b, &edge_right, cut.keys) <= room
    {
        return None;
    }
    let (mut before_left, mut before_right) = (before_left, before_right);
    before_left.sort_by(|x, y| order(x, y));
    before_right.sort_by(|x, y| order(x, y));
    let settled_agrees = before_left.len() == before_right.len()
        && before_left.iter().zip(&before_right).all(|(x, y)| alike(x, y));
    settled_agrees.then_some(Agreement::Boundary)
}

/// The leading `keys` fields of a row, which is its sort key.
fn key(row: &[String], keys: usize) -> Vec<String> {
    row.iter().take(keys).cloned().collect()
}

/// How many rows of an unlimited answer carry this sort key.
fn sharing(whole: &str, edge: &[String], keys: usize) -> usize {
    table(whole).iter().filter(|row| alike(&key(row, keys), edge)).count()
}

/// Which engines disagreed with the first one, given what each of them answered.
///
/// The first is the reference rather than a vote, because a majority of three engines agreeing is
/// not evidence when the fourth is the one being developed. DuckDB goes first, and the sentence
/// this produces names it, so a reader knows which side of a disagreement is the claim.
///
/// The sentence says which half disagreed. It used to print the two number counts whatever the
/// disagreement was, and for a group by that ties at its `LIMIT` the counts are equal and the text
/// is not, so the sentence read "20 numbers against 20" and left a reader to work out that the
/// numbers were never the problem. Two engines that computed the same counts and kept different
/// tied rows is a different thing from two engines that computed different counts, and the first
/// is usually the query rather than the engine.
#[must_use]
pub fn disagreements(answers: &[(String, String)]) -> Vec<String> {
    let Some((reference, expected)) = answers.first() else { return Vec::new() };
    let mut out = Vec::new();
    for (engine, got) in answers.iter().skip(1) {
        if agreement(expected, got).is_none() {
            out.push(format!("{engine} does not agree with {reference}: {}", how(expected, got)));
        }
    }
    out
}

/// Which engines agreed with the first one only after both answers were sorted.
///
/// Recorded rather than passed over. A query whose `ORDER BY` does not totally order its rows was
/// checked less thoroughly than one that does, and a report that ticks both the same way is a
/// report claiming a check it did not make. It is also the sentence that says which queries the
/// suite would want a tiebreaking column on.
#[must_use]
pub fn ties(answers: &[(String, String)]) -> Vec<String> {
    let Some((_, expected)) = answers.first() else { return Vec::new() };
    answers
        .iter()
        .skip(1)
        .filter(|(_, got)| agreement(expected, got) == Some(Agreement::Tied))
        .map(|(engine, _)| engine.clone())
        .collect()
}

/// Which half of an answer the disagreement is in, for the sentence above.
fn how(expected: &str, got: &str) -> String {
    let (mut mine, mut theirs) = (printed(got), printed(expected));
    mine.sort_by(|x, y| f64::total_cmp(&x.0, &y.0));
    theirs.sort_by(|x, y| f64::total_cmp(&x.0, &y.0));
    let same_numbers =
        mine.len() == theirs.len() && mine.iter().zip(&theirs).all(|(x, y)| close(*x, *y));
    if !same_numbers {
        return format!("{} numbers against {}", mine.len(), theirs.len());
    }
    let counted = theirs.len();
    let (mine, theirs) = (words(got), words(expected));
    let common = mine.iter().filter(|word| theirs.contains(word)).count();
    format!(
        "the same {counted} numbers and {} of {} text fields different, so the disagreement is \
         which rows came back rather than what they hold",
        theirs.len().saturating_sub(common),
        theirs.len()
    )
}

#[cfg(test)]
mod tests {
    use super::{
        Agreement, Cut, agreement, boundary, cut, disagreements, numbers, same, scan, ties, words,
    };

    #[test]
    fn two_engines_that_produced_the_same_rows_in_the_same_order_agree_outright() {
        assert_eq!(agreement("a,1\nb,2", "a,1\nb,2"), Some(Agreement::Ordered));
    }

    /// The step that exists because sorting first would hide a wrong sort, which is a real bug in a
    /// `TopN` operator and not a hypothetical one.
    #[test]
    fn the_same_rows_in_a_different_order_are_a_tie_and_not_an_agreement_outright() {
        assert_eq!(agreement("a,1\nb,2", "b,2\na,1"), Some(Agreement::Tied));
        assert_eq!(ties(&engines("a,1\nb,2", "b,2\na,1")), vec!["rudb".to_owned()]);
        assert!(ties(&engines("a,1\nb,2", "a,1\nb,2")).is_empty(), "an ordered match is not a tie");
    }

    /// Two engines that returned the same values arranged into different rows have not returned the
    /// same answer, and under the flat comparison this replaced they did.
    #[test]
    fn the_same_values_in_different_rows_is_a_disagreement() {
        assert_eq!(agreement("a,1\nb,2", "a,2\nb,1"), None);
    }

    #[test]
    fn a_word_where_another_engine_printed_a_number_is_a_disagreement() {
        assert_eq!(agreement("a,1", "a,x"), None);
    }

    #[test]
    fn the_order_by_width_and_the_limit_come_out_of_the_query() {
        assert_eq!(cut("SELECT 1"), Cut { keys: 0, limit: None });
        assert_eq!(cut("SELECT x FROM t ORDER BY x LIMIT 10"), Cut { keys: 1, limit: Some(10) });
        assert_eq!(
            cut("SELECT x FROM t ORDER BY a, b DESC, c LIMIT 100"),
            Cut { keys: 3, limit: Some(100) }
        );
        // The commas that belong to a function call are not the commas that separate the keys.
        assert_eq!(
            cut("SELECT x FROM t ORDER BY coalesce(a, b), c LIMIT 5"),
            Cut { keys: 2, limit: Some(5) }
        );
    }

    #[test]
    fn an_order_by_inside_brackets_is_not_the_query_s_own() {
        // A window frame carries one and a subquery carries one, and neither of them decides what
        // the rows that come back are ordered by.
        assert_eq!(
            cut("SELECT row_number() OVER (ORDER BY a) FROM t LIMIT 3"),
            Cut { keys: 0, limit: Some(3) }
        );
        assert_eq!(
            cut("SELECT * FROM (SELECT x FROM t ORDER BY x, y) ORDER BY z LIMIT 3"),
            Cut { keys: 1, limit: Some(3) }
        );
    }

    /// Q2, Q3, Q10, Q18 and Q21 of TPC-H in miniature. Two engines order by one column, limit to
    /// three, and the value at the cut is held by four rows, so each of them keeps two of the four
    /// and they do not keep the same two.
    #[test]
    fn a_limit_that_cut_a_tie_is_checked_on_the_part_the_tie_did_not_reach() {
        let asked = Cut { keys: 1, limit: Some(3) };
        let duckdb = "9,a\n5,b\n5,c";
        let rudb = "9,a\n5,d\n5,b";
        let whole = "9,a\n5,b\n5,c\n5,d\n5,e\n1,f";
        assert_eq!(agreement(duckdb, rudb), None, "the rows differ and the sorted rows differ");
        assert_eq!(boundary(duckdb, rudb, whole, whole, asked), Some(Agreement::Boundary));
    }

    #[test]
    fn a_boundary_the_engines_stopped_at_different_values_of_is_a_wrong_answer() {
        let asked = Cut { keys: 1, limit: Some(3) };
        let whole = "9,a\n5,b\n5,c\n5,d\n1,f";
        assert_eq!(boundary("9,a\n5,b\n5,c", "9,a\n5,b\n1,f", whole, whole, asked), None);
    }

    #[test]
    fn a_boundary_that_had_room_for_every_row_holding_it_is_not_a_tie() {
        // Three rows carry the cut value and there are three places for them, so neither engine
        // chose anything and a difference here is a difference about the rows.
        let asked = Cut { keys: 1, limit: Some(4) };
        let whole = "9,a\n5,b\n5,c\n5,d\n1,f";
        assert_eq!(boundary("9,a\n5,b\n5,c\n5,d", "9,a\n5,d\n5,c\n5,e", whole, whole, asked), None);
    }

    #[test]
    fn a_difference_before_the_boundary_is_not_excused_by_the_boundary() {
        let asked = Cut { keys: 1, limit: Some(3) };
        let whole = "9,a\n5,b\n5,c\n5,d\n5,e";
        // The nine is not tied with anything and the two engines returned different nines.
        assert_eq!(boundary("9,a\n5,b\n5,c", "9,z\n5,b\n5,d", whole, whole, asked), None);
    }

    #[test]
    fn a_query_with_no_limit_and_no_order_by_has_no_boundary_to_check() {
        assert_eq!(boundary("a,1", "a,2", "a,1", "a,2", Cut { keys: 1, limit: None }), None);
        assert_eq!(boundary("a,1", "a,2", "a,1", "a,2", Cut { keys: 0, limit: Some(1) }), None);
    }

    /// Two answers, as the first engine and the second gave them.
    fn engines(first: &str, second: &str) -> Vec<(String, String)> {
        vec![("duckdb".to_owned(), first.to_owned()), ("rudb".to_owned(), second.to_owned())]
    }

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

    /// q22 and q23 on ClickBench, in miniature. The counts matched and the tied rows did not, and
    /// the sentence used to say "numbers against" a number equal to itself.
    #[test]
    fn a_tie_kept_differently_is_reported_as_text_rather_than_as_numbers() {
        let answers = vec![
            ("duckdb".to_owned(), "a,2\nb,1\nc,1".to_owned()),
            ("rudb".to_owned(), "a,2\nb,1\nd,1".to_owned()),
        ];
        let found = disagreements(&answers);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("the same 3 numbers"), "{}", found[0]);
        assert!(found[0].contains("1 of 3 text fields"), "{}", found[0]);
        assert!(found[0].contains("which rows came back"), "{}", found[0]);
    }

    #[test]
    fn a_count_that_is_different_is_still_reported_as_numbers() {
        let answers = vec![
            ("duckdb".to_owned(), "a,2\nb,1".to_owned()),
            ("rudb".to_owned(), "a,2\nb,1\nc,1".to_owned()),
        ];
        let found = disagreements(&answers);
        assert!(found[0].contains("3 numbers against 2"), "{}", found[0]);
    }

    #[test]
    fn one_engine_agrees_with_itself_and_nothing_is_reported() {
        assert!(disagreements(&[("duckdb".to_owned(), "1".to_owned())]).is_empty());
        assert!(disagreements(&[]).is_empty());
    }
}
