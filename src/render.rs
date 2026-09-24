//! Turning the board into the tables, the charts and the JSON that the README and the site carry.
//!
//! Every generated block is written between two markers and nothing else in the file is touched:
//!
//! ```text
//! <!-- rudb-bench:begin ladders -->
//! ...whatever this module produced last time...
//! <!-- rudb-bench:end ladders -->
//! ```
//!
//! [`splice`] replaces what is between them and [`stale`] says which blocks a file is out of date
//! on, which is the whole of what the pull request check runs. A README that is regenerated in one
//! job and hand written in another is a README that loses one of the two, so the markers are the
//! line between the two halves and they are checked rather than trusted.
//!
//! ## Why the numbers are here and the prose is not
//!
//! A table can be generated and a claim cannot. What goes between the markers is what was measured:
//! times, ratios, sizes, versions, and the machine and day they came from. The sentence that says
//! what a ratio means stays in the README, written by a person, outside the markers, because a
//! generated sentence is a sentence nobody reviewed and this project's whole claim is a performance
//! claim.
//!
//! ## What the charts may claim
//!
//! The same as the board: nothing publishable, per `spec/15-rudb-bench.md` rule seven. Every block
//! that carries a number carries the machine it came from in the same block, so that a table cannot
//! be lifted out of the page and read as though it came from a reporting machine.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::time::Duration;

use crate::board::{self, Rung, grouped, size_of};
use crate::measure::show;
use crate::memory::bytes;
use crate::pins::Pin;

/// The marker a generated block opens with.
#[must_use]
pub fn opener(name: &str) -> String {
    format!("<!-- rudb-bench:begin {name} -->")
}

/// The marker a generated block closes with.
#[must_use]
pub fn closer(name: &str) -> String {
    format!("<!-- rudb-bench:end {name} -->")
}

/// Every block this module knows how to generate, by the name its markers carry.
///
/// A map rather than a list of functions, because the check and the write both want to walk the
/// same set and a file that carries a marker this does not know is a file that would silently keep
/// whatever was under that marker forever.
#[must_use]
pub fn blocks(rungs: &[Rung], pins: &[Pin]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    out.insert("goal".to_owned(), goal(rungs));
    out.insert("ladders".to_owned(), ladders(rungs));
    out.insert("charts".to_owned(), charts(rungs));
    out.insert("memory".to_owned(), memory(rungs));
    out.insert("versions".to_owned(), versions(rungs));
    out.insert("pins".to_owned(), pinned(rungs, pins));
    out
}

/// What to run, and whether what is on the board was run with it.
///
/// The second half is the part worth generating. A pin file says which DuckDB the comparison is
/// against, and the one way that sentence goes wrong is for the pin to be bumped while the numbers
/// under it stay where they were. Nothing else in the repository can see that, because a report is
/// an honest record of an afternoon and the afternoon was before the bump.
#[must_use]
pub fn pinned(rungs: &[Rung], pins: &[Pin]) -> String {
    let mut out = crate::pins::table(pins);
    let behind = crate::pins::measured(pins, rungs);
    if behind.is_empty() {
        if !pins.is_empty() && !rungs.is_empty() {
            let _ = writeln!(out, "Every number on the board was measured against these versions.");
            let _ = writeln!(out);
        }
        return out;
    }
    let _ = writeln!(
        out,
        "The board has not caught up with all of these. Re-run the ladder before reading a number \
         below as a number against the pinned version:"
    );
    let _ = writeln!(out);
    for (name, found) in &behind {
        let _ = writeln!(out, "- **{name}** was measured at {}", found.join(", "));
    }
    let _ = writeln!(out);
    out
}

/// Put every generated block into a file, leaving everything outside the markers alone.
///
/// # Errors
///
/// A marker that opens and never closes, a closer before its opener, or a marker naming a block
/// this module does not generate. All three are mistakes in the file rather than in the board, and
/// all three would otherwise show up as a block that quietly stopped updating.
pub fn splice(text: &str, blocks: &BTreeMap<String, String>) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("<!-- rudb-bench:begin ") {
        let (before, from) = rest.split_at(at);
        out.push_str(before);
        let line_end = from.find('\n').ok_or("a block opens on the last line of the file")?;
        let opener = from[..line_end].trim();
        let name = opener
            .trim_start_matches("<!-- rudb-bench:begin ")
            .trim_end_matches("-->")
            .trim()
            .to_owned();
        let body =
            blocks.get(&name).ok_or(format!("`{name}` is not a block rudb-bench generates"))?;
        let closer = closer(&name);
        let close_at =
            from.find(&closer).ok_or(format!("the `{name}` block opens and never closes"))?;
        out.push_str(opener);
        out.push('\n');
        out.push_str(body);
        out.push_str(&closer);
        rest = &from[close_at + closer.len()..];
    }
    out.push_str(rest);
    Ok(out)
}

/// The blocks a file is out of date on, by name.
///
/// Empty when the file already says what the board says, which is what the pull request check
/// wants: a reviewer who changed a number in a report and forgot to re-render sees the names of the
/// blocks that moved rather than a whole file diff.
///
/// # Errors
///
/// The same three the splice refuses on, because a file that cannot be spliced cannot be checked.
pub fn stale(text: &str, blocks: &BTreeMap<String, String>) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for (name, body) in blocks {
        let opener = opener(name);
        let Some(at) = text.find(&opener) else { continue };
        let from = &text[at + opener.len()..];
        let from = from.strip_prefix('\n').unwrap_or(from);
        let closer = closer(name);
        let close_at =
            from.find(&closer).ok_or(format!("the `{name}` block opens and never closes"))?;
        if &from[..close_at] != body {
            out.push(name.clone());
        }
    }
    Ok(out)
}

/// The headline: what rudb is against the best rival at every rung, on time and on memory.
///
/// Against the best rival rather than against DuckDB, which is the ratio every other block uses,
/// because the goal is stated against every rival at once and a ratio against DuckDB alone would
/// read as met on a rung where ClickHouse was faster than both. The reference is named in the row,
/// so a rung where the best rival changed says so rather than moving the bar quietly.
#[must_use]
pub fn goal(rungs: &[Rung]) -> String {
    let mut out = String::new();
    for ((suite, machine), ladder) in board::ladders(rungs) {
        let _ = writeln!(out, "**{suite}**, on {machine}. Lower is better; the goal is 0.10x.\n");
        let _ = writeln!(
            out,
            "| size | best rival | its query time | rudb | time | its peak RSS | rudb | memory |"
        );
        let _ = writeln!(out, "| --- | --- | --- | --- | --- | --- | --- | --- |");
        for rung in &ladder {
            let Some(mine) = rung.column("rudb") else { continue };
            let rivals: Vec<_> = rung
                .columns
                .iter()
                .filter(|c| c.engine != "rudb" && c.hot > Duration::ZERO)
                .collect();
            let Some(fastest) = rivals.iter().min_by_key(|c| c.hot) else { continue };
            let leanest =
                rivals.iter().filter_map(|c| c.peak.map(|p| (p, c))).min_by_key(|(p, _)| *p);
            let (memory, lean) = match (leanest, mine.peak) {
                (Some((peak, c)), Some(ours)) => (
                    format!("{:.2}x", ours as f64 / peak as f64),
                    format!("{} ({})", bytes(peak), c.engine),
                ),
                _ => ("not read".to_owned(), "not read".to_owned()),
            };
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {:.2}x | {} | {} | {} |",
                size_of(rung),
                fastest.engine,
                show(fastest.hot),
                show(mine.hot),
                mine.hot.as_secs_f64() / fastest.hot.as_secs_f64(),
                lean,
                mine.peak.map_or_else(|| "not read".to_owned(), bytes),
                memory,
            );
        }
        let _ = writeln!(out);
        let _ = writeln!(out, "{}\n", provenance(&ladder));
    }
    if out.is_empty() {
        out.push_str("No rung has been measured yet.\n\n");
    }
    out
}

/// Every ladder as a table: one row per rung, one column per engine, times and the ratio.
#[must_use]
pub fn ladders(rungs: &[Rung]) -> String {
    let mut out = String::new();
    for ((suite, machine), ladder) in board::ladders(rungs) {
        let engines = board::engines(&ladder);
        let _ = writeln!(
            out,
            "**{suite}**, on {machine}. Hot query time, median of the runs, and the ratio against \
             duckdb underneath it.\n"
        );
        let _ = writeln!(out, "| size | {} |", engines.join(" | "));
        let _ = writeln!(out, "| --- |{}", " --- |".repeat(engines.len()));
        for rung in &ladder {
            let _ = write!(out, "| {} |", size_of(rung));
            for engine in &engines {
                match rung.column(engine) {
                    Some(column) => {
                        let base = rung.reference().map_or(0.0, |r| r.hot.as_secs_f64());
                        let ratio = if base > 0.0 {
                            format!("{:.2}x", column.hot.as_secs_f64() / base)
                        } else {
                            "-".to_owned()
                        };
                        let _ = write!(out, " {}<br>{ratio} |", show(column.hot));
                    }
                    None => {
                        let _ = write!(out, " not run |");
                    }
                }
            }
            let _ = writeln!(out);
        }
        let _ = writeln!(out);
        let _ = writeln!(out, "{}", shared_note(&ladder));
        let _ = writeln!(out, "{}\n", provenance(&ladder));
    }
    if out.is_empty() {
        out.push_str("No rung has been measured yet.\n\n");
    }
    out
}

/// Peak resident set across the ladder, which is the other half of the goal.
#[must_use]
pub fn memory(rungs: &[Rung]) -> String {
    let mut out = String::new();
    for ((suite, machine), ladder) in board::ladders(rungs) {
        let engines = board::engines(&ladder);
        let _ = writeln!(
            out,
            "**{suite}**, on {machine}. Worst peak resident set over the suite, per engine.\n"
        );
        let _ = writeln!(out, "| size | {} |", engines.join(" | "));
        let _ = writeln!(out, "| --- |{}", " --- |".repeat(engines.len()));
        for rung in &ladder {
            let _ = write!(out, "| {} |", size_of(rung));
            for engine in &engines {
                let cell = match rung.column(engine) {
                    Some(column) => column.peak.map_or_else(|| "not read".to_owned(), bytes),
                    None => "not run".to_owned(),
                };
                let _ = write!(out, " {cell} |");
            }
            let _ = writeln!(out);
        }
        let _ = writeln!(out);
        let _ = writeln!(out, "{}\n", provenance(&ladder));
    }
    if out.is_empty() {
        out.push_str("No rung has been measured yet.\n\n");
    }
    out
}

/// The charts: one block of bars per rung, and one ratio line per engine across the ladder.
#[must_use]
pub fn charts(rungs: &[Rung]) -> String {
    let mut out = String::new();
    for ((suite, machine), ladder) in board::ladders(rungs) {
        let _ = writeln!(out, "**{suite}**, on {machine}.\n");
        let _ = writeln!(out, "```text");
        for rung in &ladder {
            let _ = writeln!(out, "{} — hot query time, shorter is better", size_of(rung));
            let widest = rung.columns.iter().map(|c| c.hot).max().unwrap_or(Duration::ZERO);
            let name = rung.columns.iter().map(|c| c.engine.len()).max().unwrap_or(0);
            for column in &rung.columns {
                let _ = writeln!(
                    out,
                    "  {:name$}  {} {}",
                    column.engine,
                    bar(column.hot.as_secs_f64(), widest.as_secs_f64(), 40),
                    show(column.hot),
                );
            }
            let _ = writeln!(out);
        }
        let _ = writeln!(out, "vs duckdb, hot query time, under 1.00x is faster than duckdb");
        let sizes = ladder.iter().map(|r| r.size.len()).max().unwrap_or(0);
        let engines = board::engines(&ladder);
        // One scale for the whole block rather than one per engine. Scaling each engine to its own
        // worst rung makes every engine's longest bar the same length, so a column that is 1.43x
        // draws exactly as wide as one that is 7.28x and the chart says the opposite of the number
        // printed beside it.
        let top = engines
            .iter()
            .filter(|e| *e != "duckdb")
            .flat_map(|e| board::ratios(&ladder, e))
            .flatten()
            .fold(1.0f64, f64::max);
        for engine in &engines {
            if engine == "duckdb" {
                continue;
            }
            let _ = writeln!(out, "  {engine}");
            let ratios = board::ratios(&ladder, engine);
            for (rung, ratio) in ladder.iter().zip(&ratios) {
                match ratio {
                    Some(ratio) => {
                        let _ = writeln!(
                            out,
                            "    {:sizes$}  {} {ratio:.2}x",
                            rung.size,
                            bar(*ratio, top, 32),
                        );
                    }
                    None => {
                        let _ = writeln!(out, "    {:sizes$}  not run", rung.size);
                    }
                }
            }
        }
        let _ = writeln!(out, "```\n");
        let _ = writeln!(out, "{}\n", provenance(&ladder));
    }
    if out.is_empty() {
        out.push_str("No rung has been measured yet.\n\n");
    }
    out
}

/// One bar, in eighths of a column so that a small value is a sliver rather than nothing.
///
/// A bar that rounded to zero would print a fast engine as an absence, which is the one reading a
/// speed chart must never produce.
#[must_use]
pub fn bar(value: f64, widest: f64, width: usize) -> String {
    // A NaN falls out here too, which is what should happen to it: a bar of an unmeasurable value
    // is no bar, and the number beside it already says what it was.
    if value.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)
        || widest.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)
    {
        return String::new();
    }
    let eighths = ((value / widest) * width as f64 * 8.0).round().max(1.0) as usize;
    let full = eighths / 8;
    let rest = eighths % 8;
    let mut out = "█".repeat(full);
    if rest > 0 {
        out.push(['▏', '▏', '▎', '▍', '▌', '▋', '▊', '▉'][rest]);
    }
    out
}

/// The exact versions every rung was measured against, per reporting rule one.
///
/// The machine is a column rather than a footnote, because a version is a fact about a machine: the
/// DuckDB on one box and the DuckDB on another are two binaries, and a table that merged them would
/// be claiming they were one.
#[must_use]
pub fn versions(rungs: &[Rung]) -> String {
    let mut seen: BTreeMap<(String, String, String), usize> = BTreeMap::new();
    for rung in rungs {
        for column in &rung.columns {
            *seen
                .entry((column.engine.clone(), column.version.clone(), rung.machine.clone()))
                .or_default() += 1;
        }
    }
    if seen.is_empty() {
        return "No rung has been measured yet.\n\n".to_owned();
    }
    let mut out = String::new();
    let _ = writeln!(out, "| engine | version | machine | rungs |");
    let _ = writeln!(out, "| --- | --- | --- | --- |");
    for ((engine, version, machine), count) in &seen {
        let _ = writeln!(out, "| {engine} | {version} | {machine} | {count} |");
    }
    let _ = writeln!(out);
    // Only when it has happened. A warning printed under every table whether or not it applies is
    // a warning that stops being read by the time it does.
    let mut split: BTreeMap<(&String, &String), usize> = BTreeMap::new();
    for (engine, _, machine) in seen.keys() {
        *split.entry((engine, machine)).or_default() += 1;
    }
    for ((engine, machine), rows) in &split {
        if *rows > 1 {
            let _ = writeln!(
                out,
                "**{engine} has {rows} versions on {machine}**, so the rungs under it are not one \
                 ladder and a slope read through them is a slope through two engines. Re-run the \
                 older rungs."
            );
            let _ = writeln!(out);
        }
    }
    out
}

/// Where a ladder came from, and whether it was taken in one sitting.
fn provenance(ladder: &[Rung]) -> String {
    let Some(newest) = ladder.iter().map(|r| r.recorded.as_str()).max() else {
        return String::new();
    };
    let stale = board::stale(ladder);
    let machine = ladder.first().map_or("", |r| r.machine.as_str());
    let commit = ladder.first().map_or("", |r| r.commit.as_str());
    let mut out = format!(
        "Measured on {machine} on {newest}, by rudb-bench at {commit}. Rule seven: this machine is \
         not a reporting machine, so nothing here is comparable to a published ClickBench or \
         TPC-H number."
    );
    if !stale.is_empty() {
        let names: Vec<&str> = stale.iter().map(|r| r.size.as_str()).collect();
        let _ = write!(
            out,
            " The {} rung{} older than the rest of this ladder ({}), so the slope through it is \
             partly whatever changed in between.",
            names.join(", "),
            if names.len() == 1 { " is" } else { "s are" },
            stale.iter().map(|r| r.recorded.as_str()).collect::<Vec<_>>().join(", "),
        );
    }
    out
}

/// How many queries the ratios were taken over, when that is not all of them.
fn shared_note(ladder: &[Rung]) -> String {
    // One sentence for the ladder rather than one per rung. The reason is the same on every rung
    // and a paragraph that repeats itself four times is a paragraph nobody reads to the end of, so
    // the counts go in a list and the explanation is said once.
    let mut counts: Vec<(usize, &str)> = Vec::new();
    let mut whole = 0;
    for rung in ladder {
        let here = rung.columns.iter().map(|c| c.queries).max().unwrap_or(0);
        whole = whole.max(here);
        if rung.shared < here {
            counts.push((rung.shared, rung.size.as_str()));
        }
    }
    if counts.is_empty() {
        return String::new();
    }
    // The usual case is that every rung shares the same count, because what an engine cannot
    // express it cannot express at any size, and one number written five times reads like five
    // facts.
    let how_many = if counts.iter().all(|(n, _)| *n == counts[0].0) {
        if counts.len() > 1 {
            format!("{} on every rung", counts[0].0)
        } else {
            counts[0].0.to_string()
        }
    } else {
        counts.iter().map(|(n, size)| format!("{n} at {size}")).collect::<Vec<_>>().join(", ")
    };
    format!(
        "Ratios are over the queries every engine measured, which is {how_many} of {whole}. An \
         engine that cannot express a query, or that ran out of time on one, would otherwise be \
         compared on a different set of queries from the rest.\n",
    )
}

/// The board as JSON, which is what the charts under `docs/` read.
///
/// Written by hand, like every other format in this crate, because a serializer is a dependency
/// tree and this is eleven keys and a list.
#[must_use]
pub fn json(rungs: &[Rung]) -> String {
    let mut out = String::from("{\n  \"ladders\": [\n");
    let ladders = board::ladders(rungs);
    for (index, ((suite, machine), ladder)) in ladders.iter().enumerate() {
        let _ = writeln!(out, "    {{");
        let _ = writeln!(out, "      \"suite\": {},", quoted(suite));
        let _ = writeln!(out, "      \"machine\": {},", quoted(machine));
        let _ = writeln!(
            out,
            "      \"engines\": [{}],",
            board::engines(ladder).iter().map(|e| quoted(e)).collect::<Vec<_>>().join(", ")
        );
        let _ = writeln!(out, "      \"rungs\": [");
        for (at, rung) in ladder.iter().enumerate() {
            let _ = writeln!(out, "        {{");
            let _ = writeln!(out, "          \"size\": {},", quoted(&rung.size));
            let _ = writeln!(
                out,
                "          \"rows\": {},",
                rung.rows.map_or_else(|| "null".to_owned(), |n| n.to_string())
            );
            let _ = writeln!(out, "          \"shared\": {},", rung.shared);
            let _ = writeln!(out, "          \"recorded\": {},", quoted(&rung.recorded));
            let _ = writeln!(out, "          \"commit\": {},", quoted(&rung.commit));
            let _ = writeln!(out, "          \"columns\": [");
            for (n, column) in rung.columns.iter().enumerate() {
                let base = rung.reference().map_or(0.0, |r| r.hot.as_secs_f64());
                let ratio = if base > 0.0 { column.hot.as_secs_f64() / base } else { 0.0 };
                let _ = write!(
                    out,
                    "            {{\"engine\": {}, \"version\": {}, \"hot_ms\": {:.3}, \
                     \"own_ms\": {:.3}, \"reported\": {}, \"wall_ms\": {:.3}, \
                     \"cold_ms\": {:.3}, \"cpu_ms\": {}, \"peak\": {}, \
                     \"load_ms\": {:.3}, \"disk\": {}, \"converted\": {}, \"answered\": {}, \
                     \"queries\": {}, \"ratio\": {ratio:.4}}}",
                    quoted(&column.engine),
                    quoted(&column.version),
                    column.hot.as_secs_f64() * 1000.0,
                    column.own.as_secs_f64() * 1000.0,
                    column.reported,
                    column.wall.as_secs_f64() * 1000.0,
                    column.cold.as_secs_f64() * 1000.0,
                    column.cpu.map_or_else(
                        || "null".to_owned(),
                        |d| format!("{:.3}", d.as_secs_f64() * 1000.0)
                    ),
                    column.peak.map_or_else(|| "null".to_owned(), |n| n.to_string()),
                    column.load.as_secs_f64() * 1000.0,
                    column.disk,
                    column.converted,
                    column.answered,
                    column.queries,
                );
                let _ = writeln!(out, "{}", if n + 1 == rung.columns.len() { "" } else { "," });
            }
            let _ = writeln!(out, "          ]");
            let _ = writeln!(out, "        }}{}", if at + 1 == ladder.len() { "" } else { "," });
        }
        let _ = writeln!(out, "      ]");
        let _ = writeln!(out, "    }}{}", if index + 1 == ladders.len() { "" } else { "," });
    }
    out.push_str("  ]\n}\n");
    out
}

/// A string as JSON writes it.
fn quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A row count for a chart axis, short enough to sit under a bar.
#[must_use]
pub fn axis(rows: Option<u64>, size: &str) -> String {
    rows.map_or_else(|| size.to_owned(), grouped)
}

#[cfg(test)]
mod tests {
    use super::{bar, blocks, charts, goal, json, ladders, opener, splice, stale, versions};
    use crate::board::{Column, Rung};
    use std::collections::BTreeMap;
    use std::time::Duration;

    fn column(engine: &str, hot_ms: u64, peak: u64) -> Column {
        Column {
            engine: engine.to_owned(),
            version: format!("{engine} 1.0"),
            hot: Duration::from_millis(hot_ms),
            own: Duration::from_millis(hot_ms),
            reported: true,
            wall: Duration::from_millis(hot_ms * 2),
            cold: Duration::from_millis(hot_ms * 3),
            cpu: Some(Duration::from_millis(hot_ms * 3)),
            peak: Some(peak),
            load: Duration::from_millis(10),
            disk: 4096,
            converted: engine != "rudb",
            answered: 43,
            queries: 43,
        }
    }

    fn rung(size: &str, rows: u64, duck: u64, rudb: u64) -> Rung {
        Rung {
            suite: "clickbench".to_owned(),
            machine: "gamingpc-wsl".to_owned(),
            size: size.to_owned(),
            rows: Some(rows),
            shared: 43,
            commit: "abc1234".to_owned(),
            recorded: "2026-09-21".to_owned(),
            columns: vec![
                column("duckdb", duck, 300 * 1024 * 1024),
                column("clickhouse-local", duck * 3, 400 * 1024 * 1024),
                column("rudb", rudb, 160 * 1024 * 1024),
            ],
        }
    }

    fn ladder() -> Vec<Rung> {
        vec![rung("1k", 1_000, 100, 50), rung("1m", 1_000_000, 500, 900)]
    }

    #[test]
    fn a_spliced_file_keeps_everything_outside_the_markers() {
        let text = format!(
            "# title\n\nprose above\n\n{}\nold and wrong\n{}\n\nprose below\n",
            opener("versions"),
            super::closer("versions"),
        );
        let blocks = blocks(&ladder(), &[]);
        let out = splice(&text, &blocks).expect("splices");
        assert!(out.starts_with("# title\n\nprose above\n"), "{out}");
        assert!(out.ends_with("prose below\n"), "{out}");
        assert!(!out.contains("old and wrong"), "the block was not replaced");
        assert!(out.contains("| duckdb | duckdb 1.0 | gamingpc-wsl |"), "{out}");
        assert!(stale(&out, &blocks).expect("checks").is_empty(), "a spliced file is not fresh");
    }

    #[test]
    fn a_file_that_was_not_re_rendered_names_the_blocks_that_moved() {
        let text = format!(
            "{}\nnothing like the board\n{}\n",
            opener("versions"),
            super::closer("versions")
        );
        assert_eq!(stale(&text, &blocks(&ladder(), &[])).expect("checks"), ["versions"]);
    }

    /// A marker for a block nobody generates would sit there forever holding whatever was pasted
    /// under it, which is the one failure a generated README cannot show on its face.
    #[test]
    fn a_marker_this_does_not_generate_is_refused() {
        let text = format!("{}\n{}\n", opener("hopes"), super::closer("hopes"));
        let e = splice(&text, &blocks(&ladder(), &[])).expect_err("refused");
        assert!(e.contains("hopes"), "{e}");
    }

    #[test]
    fn a_block_that_never_closes_is_refused() {
        let text = format!("{}\nand then nothing\n", opener("versions"));
        assert!(splice(&text, &blocks(&ladder(), &[])).is_err(), "an unclosed block was spliced");
        assert!(stale(&text, &blocks(&ladder(), &[])).is_err(), "an unclosed block was checked");
    }

    #[test]
    fn every_block_names_the_machine_it_came_from() {
        for (name, body) in blocks(&ladder(), &[]) {
            if name == "pins" {
                // The pins are what to run rather than what was run, so they have no machine and
                // the block that carries a machine is the one underneath them.
                continue;
            }
            assert!(body.contains("gamingpc-wsl"), "the {name} block does not name the machine");
        }
    }

    #[test]
    fn an_empty_board_renders_rather_than_panicking() {
        for (name, body) in blocks(&[], &[]) {
            let said = if name == "pins" { "Nothing is pinned yet" } else { "No rung" };
            assert!(body.contains(said), "the {name} block said {body}");
        }
    }

    #[test]
    fn the_goal_is_against_the_fastest_rival_and_not_against_duckdb() {
        let mut only = rung("1k", 1_000, 100, 50);
        only.columns[1].hot = Duration::from_millis(40);
        let out = goal(&[only]);
        assert!(out.contains("| clickhouse-local |"), "{out}");
        assert!(out.contains("1.25x"), "50ms against a 40ms rival is 1.25x: {out}");
    }

    #[test]
    fn the_ladder_table_has_a_row_per_rung_and_a_column_per_engine() {
        let out = ladders(&ladder());
        assert!(out.contains("| size | duckdb | clickhouse-local | rudb |"), "{out}");
        assert!(out.contains("1k (1,000 rows)"), "{out}");
        assert!(out.contains("0.50x"), "50ms against 100ms is 0.50x: {out}");
        assert!(out.contains("1.80x"), "900ms against 500ms is 1.80x: {out}");
    }

    /// A fast engine printed as no bar at all is a chart that reads as a missing measurement.
    #[test]
    fn a_bar_is_never_empty_for_a_value_that_is_not_zero() {
        assert_eq!(bar(0.0, 10.0, 40), "");
        assert_eq!(bar(10.0, 0.0, 40), "");
        assert!(!bar(0.0001, 10.0, 40).is_empty());
        assert_eq!(bar(10.0, 10.0, 8).chars().count(), 8);
        assert!(bar(5.0, 10.0, 8).chars().count() <= 5);
    }

    #[test]
    fn the_chart_draws_every_rung_and_every_engine() {
        let out = charts(&ladder());
        assert_eq!(out.matches("hot query time, shorter is better").count(), 2);
        assert!(out.contains("vs duckdb"), "{out}");
        assert!(out.contains("  rudb\n"), "{out}");
        assert!(!out.contains("  duckdb\n    "), "duckdb was charted against itself");
    }

    #[test]
    fn a_ladder_assembled_over_two_days_says_so_on_the_page() {
        let mut mixed = ladder();
        mixed[0].recorded = "2026-08-01".to_owned();
        let out = ladders(&mixed);
        assert!(out.contains("older than the rest of this ladder"), "{out}");
        assert!(out.contains("2026-08-01"), "{out}");
    }

    #[test]
    fn two_versions_of_one_engine_are_two_rows_and_a_warning() {
        let mut mixed = ladder();
        mixed[1].columns[0].version = "duckdb 1.1".to_owned();
        let out = versions(&mixed);
        assert!(out.contains("| duckdb | duckdb 1.0 | gamingpc-wsl | 1 |"), "{out}");
        assert!(out.contains("| duckdb | duckdb 1.1 | gamingpc-wsl | 1 |"), "{out}");
        assert!(out.contains("not one ladder"), "{out}");
    }

    /// Hand written JSON that a browser cannot parse is a chart page that silently shows nothing,
    /// so the shape is checked here rather than in the one place it would not be noticed.
    #[test]
    fn the_json_is_balanced_and_quotes_what_it_should() {
        let out = json(&ladder());
        let opens = out.matches('{').count();
        let closes = out.matches('}').count();
        assert_eq!(opens, closes, "{out}");
        assert_eq!(out.matches('[').count(), out.matches(']').count(), "{out}");
        assert!(out.contains("\"ratio\": 1.8000"), "{out}");
        assert!(out.contains("\"engines\": [\"duckdb\", \"clickhouse-local\", \"rudb\"]"), "{out}");
        assert!(!out.contains(",\n          ]"), "a trailing comma before a close: {out}");
        assert!(!out.contains(",\n      ]"), "a trailing comma before a close: {out}");
    }

    #[test]
    fn a_file_with_no_markers_comes_back_exactly_as_it_went_in() {
        let text = "# nothing generated here\n\njust prose.\n";
        assert_eq!(splice(text, &blocks(&ladder(), &[])).expect("splices"), text);
        assert!(stale(text, &BTreeMap::new()).expect("checks").is_empty());
    }
}
