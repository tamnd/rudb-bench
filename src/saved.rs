//! Keeping a run, so that engines measured on different days end up in one table.
//!
//! A full ClickBench over fourteen gigabytes takes hours per engine, and six engines in one command
//! is a day that fails on the fifth one and leaves nothing behind. So a run can be told to save what
//! it measured, one engine at a time, and the table is built afterwards from whatever has been saved
//! so far. The file is text, one block per engine, and it is meant to be read by a person and
//! committed, which is the same decision the records file in `baselines/` already made.
//!
//! What is saved is the measurement and not a summary of it. Every sample is written out, so a
//! result that comes back out of this file has the same interquartile range it had going in, and the
//! publication rules apply to it exactly as they did on the day. Nothing here can turn a number that
//! could not be published into one that can: a small run says so in its own block and says so again
//! when it is read back.
//!
//! The one thing this file cannot do is make two blocks comparable that were not. Rule seven is
//! machine by machine, so the machine is part of the filename rather than a column, and a block that
//! ran over a sample of the data carries the sample it ran over. Two engines saved against different
//! samples are refused rather than printed side by side, because a table like that looks exactly
//! like a table where one engine is eight times faster.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::data::Sample;
use crate::engine::Loaded;
use crate::measure::{Convention, Distribution, Runs};
use crate::memory::{Cost, Peak};
use crate::report::{Abstention, Comparison, QueryResult, SuiteResult};
use crate::suite::find;

/// Where the saved runs for one suite on one machine live.
///
/// The machine is in the name rather than in a column, because rule seven says a number from one
/// machine is never compared against a number from another and a single file holding both would be
/// one `--engines` flag away from doing exactly that.
#[must_use]
pub fn path(suite: &str, machine: &str) -> PathBuf {
    let name = format!("saved-{suite}-{machine}.txt");
    std::env::var_os("RUDB_BENCH_REPORTS")
        .map_or_else(|| Path::new("reports").join(&name), |at| PathBuf::from(at).join(&name))
}

/// The header the file carries, so that somebody who opens it knows what they are looking at.
const PREAMBLE: &str = "\
# Saved benchmark runs, one block per engine.
#
# Written by `rudb-bench run <suite> --save` and read by `rudb-bench report <suite>`, which builds
# the cross engine table out of whatever is in here. A full ClickBench is hours per engine, so the
# engines are measured one at a time and the table is assembled afterwards from this file.
#
# Times are microseconds, as integers, and every sample is here rather than a median of them. A
# saved median could not be recombined with another engine's run without losing the spread, and the
# spread is what says whether the median means anything.
#
# Re-running an engine replaces its block. Nothing else in the file is touched, so the engine that
# ran this morning keeps the number it got this morning.
#
# Reporting rule seven still applies, which is why the machine is in the filename. These are not
# comparable to anybody's board and they are not comparable to the same file from another machine.
#
# `loadavg` is the one minute load average before and after that engine's suite, on a shared fleet
# where the usual reason two runs disagree is that one of them had the machine to itself. `load` a
# few lines further down is a different thing, the time the load step took.
#
# `cold-is` says whether the cold number is a run that had to go to the device, which is `dropped`,
# or a run that was merely the first one, which is `first`. Dropping the page cache needs root and
# throws away every other process's working set, so it is asked for rather than assumed.
";

/// Add this result to the file, replacing whatever that engine had there before.
///
/// # Errors
///
/// When the directory could not be made, or the existing file could not be read or rewritten.
pub fn save(
    result: &SuiteResult,
    machine: &str,
    source_bytes: u64,
) -> Result<PathBuf, std::io::Error> {
    let at = path(result.suite.name, machine);
    if let Some(parent) = at.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // Read, replace, write. Not append, because the whole point of saving per engine is that an
    // engine gets re-run after somebody changes it, and a file that appended would grow a second
    // rudb block and then have to guess which one the table meant.
    let mut blocks: BTreeMap<String, String> = match std::fs::read_to_string(&at) {
        Ok(text) => split(&text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => BTreeMap::new(),
        Err(e) => return Err(e),
    };
    blocks.insert(result.engine.clone(), write_one(result, machine, source_bytes));

    let mut out = String::from(PREAMBLE);
    for block in blocks.values() {
        out.push_str("\n[result]\n");
        out.push_str(block);
    }
    std::fs::write(&at, out)?;
    Ok(at)
}

/// Every engine saved for this suite on this machine, in the order the report wants them.
///
/// # Errors
///
/// When the file is missing, unreadable, or holds a block this cannot make sense of.
pub fn read(suite: &str, machine: &str) -> Result<Vec<(SuiteResult, u64)>, String> {
    let at = path(suite, machine);
    let text = std::fs::read_to_string(&at)
        .map_err(|e| format!("cannot read {}: {e}. Run with --save first", at.display()))?;
    let mut results = Vec::new();
    for (engine, block) in split(&text) {
        results.push(read_one(&engine, &block).map_err(|e| format!("{}: {e}", at.display()))?);
    }
    Ok(results)
}

/// Build the cross engine comparison out of the saved blocks.
///
/// The order is the order the harness discovers engines in, so the reference column of a table built
/// from this file is the same column it would have been in a single run. An engine that was never
/// saved becomes a sentence under the table rather than a gap in it, which is the same treatment an
/// engine that is not installed already gets.
///
/// # Errors
///
/// When nothing is saved, when the suite is not one this harness knows, or when two blocks ran over
/// different amounts of data and so are not a comparison.
pub fn restore(suite: &str, machine: &str, order: &[&str]) -> Result<Comparison, String> {
    let Some(found) = find(suite) else {
        return Err(format!("no suite called {suite}"));
    };
    let read_back = read(suite, machine)?;
    if read_back.is_empty() {
        return Err(format!("nothing saved for {suite} on {machine}"));
    }
    // The size of what everything read is a fact about the suite rather than about an engine, so it
    // is written into every block and the first one is taken. A block that disagreed with the rest
    // ran over a different cut of the data, and the check below is the one that catches that.
    let source_bytes = read_back[0].1;
    let mut saved: Vec<SuiteResult> = read_back.into_iter().map(|(r, _)| r).collect();
    // Two engines that ran over different cuts of the data are not two columns of one table. The
    // smaller one comes out four to eight times faster and nothing in the table says why, so this
    // is refused rather than footnoted.
    let sample = saved[0].sample;
    let rows = saved[0].rows;
    if let Some(odd) = saved.iter().find(|r| r.sample != sample) {
        return Err(format!(
            "{} ran over {} and {} ran over {}, which is not a comparison. Re-run one of them",
            saved[0].engine,
            describe(sample.as_ref()),
            odd.engine,
            describe(odd.sample.as_ref()),
        ));
    }
    saved.sort_by_key(|r| order.iter().position(|n| *n == r.engine).unwrap_or(usize::MAX));

    let skipped = order
        .iter()
        .filter(|n| !saved.iter().any(|r| r.engine == **n))
        .map(|n| Abstention {
            engine: (*n).to_owned(),
            version: "unknown".to_owned(),
            why: "nothing saved for it on this machine yet".to_owned(),
            unasked: true,
        })
        .collect();

    Ok(Comparison { suite: found, source_bytes, sample, rows, results: saved, skipped })
}

/// How much data a block ran over, in the words the refusal above needs.
fn describe(sample: Option<&Sample>) -> String {
    sample.map_or_else(
        || "the whole table".to_owned(),
        |s| format!("{} rows, one in every {}", s.rows, s.every),
    )
}

/// Cut a file into one block per engine, keyed by engine name.
fn split(text: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for block in text.split("[result]\n").skip(1) {
        let body = block.trim_end_matches('\n');
        if let Some(engine) = value(body, "engine") {
            out.insert(engine.to_owned(), format!("{body}\n"));
        }
    }
    out
}

/// The value of the first line with this key, where the value is the rest of the line.
fn value<'a>(block: &'a str, key: &str) -> Option<&'a str> {
    block.lines().find_map(|line| line.strip_prefix(key)?.strip_prefix(' ').map(str::trim_start))
}

/// The two load readings out of the line that holds them.
///
/// Both or neither. Half a pair would have to be written down as a load that was read once, and
/// the whole reason there are two is that one of them cannot tell a run that got busy halfway from
/// a run that was busy throughout.
fn loadavg(text: &str) -> Option<(f64, f64)> {
    let (before, after) = text.split_once(' ')?;
    Some((before.trim().parse().ok()?, after.trim().parse().ok()?))
}

/// Microseconds, as this file writes them.
fn micros(d: Duration) -> u128 {
    d.as_micros()
}

/// A list of durations, space separated, as microseconds.
fn list(every: &[Duration]) -> String {
    every.iter().map(|d| micros(*d).to_string()).collect::<Vec<_>>().join(" ")
}

/// Read that back.
fn unlist(text: &str) -> Result<Vec<Duration>, String> {
    text.split_whitespace()
        .map(|n| {
            n.parse::<u64>()
                .map(Duration::from_micros)
                .map_err(|_| format!("{n} is not a number of microseconds"))
        })
        .collect()
}

/// A newline in a value would be a new key, so it is spelled out and so is the escape that spells it.
fn escape(text: &str) -> String {
    text.replace('\\', "\\\\").replace('\n', "\\n")
}

/// The other half of [`escape`].
fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text.chars();
    while let Some(c) = rest.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match rest.next() {
            Some('n') => out.push('\n'),
            Some('\\') => out.push('\\'),
            // Anything else was not written by `escape`, so it is kept as it was found rather than
            // dropped. A saved answer that lost a character would fail the agreement check and send
            // somebody looking for a bug in an engine.
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// A peak, which is either a number of bytes or the sentence saying why there is not one.
fn peak(p: &Peak) -> String {
    match p {
        Peak::Bytes(n) => n.to_string(),
        Peak::Unavailable(why) => format!("- {}", escape(why)),
    }
}

/// Read that back.
fn unpeak(text: &str) -> Peak {
    text.strip_prefix('-').map_or_else(
        || {
            text.parse::<u64>().map_or_else(
                |_| Peak::Unavailable(format!("the saved run wrote {text}, which is not a size")),
                Peak::Bytes,
            )
        },
        |why| Peak::Unavailable(unescape(why.trim_start())),
    )
}

/// One cost, on one line, as `peak cpu read`.
fn cost(c: &Cost) -> String {
    let cpu = c.cpu.map_or_else(|| "-".to_owned(), |d| micros(d).to_string());
    let read = c.read.map_or_else(|| "-".to_owned(), |n| n.to_string());
    // The peak goes last, because it is the only one of the three that can be a sentence with
    // spaces in it and a field of unknown width has to be the last field on the line.
    format!("{cpu} {read} {}", peak(&c.peak))
}

/// Read that back.
fn uncost(text: &str) -> Result<Cost, String> {
    let mut parts = text.splitn(3, ' ');
    let (Some(cpu), Some(read), Some(rest)) = (parts.next(), parts.next(), parts.next()) else {
        return Err(format!("a cost line needs three fields and this has `{text}`"));
    };
    Ok(Cost {
        cpu: maybe(cpu).map(|n| Duration::from_micros(n.parse().unwrap_or(0))),
        read: maybe(read).and_then(|n| n.parse().ok()),
        peak: unpeak(rest),
    })
}

/// A field spelled `-` is an absence rather than a value.
fn maybe(text: &str) -> Option<&str> {
    (text != "-").then_some(text)
}

/// One result, as the text that goes between two `[result]` markers.
fn write_one(result: &SuiteResult, machine: &str, source_bytes: u64) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "suite     {}", result.suite.name);
    let _ = writeln!(out, "machine   {machine}");
    let _ = writeln!(out, "engine    {}", result.engine);
    let _ = writeln!(out, "version   {}", escape(&result.version));
    let _ = writeln!(out, "saved     {}", crate::regress::today());
    let _ = writeln!(out, "source    {source_bytes}");
    let _ = writeln!(out, "state     {}", if result.keeps_state { "kept" } else { "fresh" });
    // Not `load`, which is the load time. This is how busy the machine was, and it is written next
    // to the engine's name rather than with the timings because it qualifies all of them at once.
    if let Some((before, after)) = result.load {
        let _ = writeln!(out, "loadavg   {before} {after}");
    }
    // Whether the cold column is a run that went to the device or a run that was merely first. A
    // block written before this line existed reads back as `first`, which is what those runs were.
    let _ = writeln!(out, "cold-is   {}", if result.cold_forced { "dropped" } else { "first" });
    if let Some(rows) = result.rows {
        let _ = writeln!(out, "rows      {rows}");
    }
    if let Some(s) = &result.sample {
        let _ = writeln!(out, "sample    {} {} {} {}", s.full, s.rows, s.every, s.asked);
    }
    if !result.missing.is_empty() {
        let _ = writeln!(out, "missing   {}", result.missing.join(" "));
    }

    let l = &result.loaded;
    let _ = writeln!(out, "load      {}", micros(l.took));
    let _ = writeln!(
        out,
        "load-cpu  {}",
        l.cpu.map_or_else(|| "-".to_owned(), |d| micros(d).to_string())
    );
    let _ = writeln!(out, "load-disk {}", l.on_disk);
    let _ = writeln!(out, "load-conv {}", if l.converted { "yes" } else { "no" });
    let _ = writeln!(out, "load-what {}", escape(&l.on_disk_is));

    for q in &result.queries {
        let _ = writeln!(out, "query     {} {}", q.name, escape(&q.shape));
        let _ = writeln!(
            out,
            "  by      {}",
            match q.runs.hot.convention() {
                Convention::Median => "median",
                Convention::Clickbench => "clickbench",
            }
        );
        let _ = writeln!(out, "  wall    {} {}", micros(q.runs.cold), list(q.runs.hot.samples()));
        if let Some(said) = &q.reported {
            let _ = writeln!(out, "  said    {} {}", micros(said.cold), list(said.hot.samples()));
        }
        let _ = writeln!(out, "  cold    {}", cost(&q.cold));
        let _ = writeln!(out, "  hot     {}", cost(&q.hot));
        let _ = writeln!(out, "  answer  {}", escape(&q.answer));
    }
    out
}

/// One result, from the text between two `[result]` markers.
fn read_one(engine: &str, block: &str) -> Result<(SuiteResult, u64), String> {
    let need = |key: &str| {
        value(block, key).ok_or_else(|| format!("the {engine} block has no `{key}` line"))
    };
    let suite_name = need("suite")?;
    let Some(suite) = find(suite_name) else {
        return Err(format!("the {engine} block names a suite called {suite_name}"));
    };

    let sample = match value(block, "sample") {
        None => None,
        Some(text) => {
            let f: Vec<&str> = text.split_whitespace().collect();
            let [full, rows, every, asked] = f.as_slice() else {
                return Err(format!("a sample line needs four fields and {engine} has `{text}`"));
            };
            Some(Sample {
                full: full.parse().map_err(|_| format!("{full} is not a row count"))?,
                rows: rows.parse().map_err(|_| format!("{rows} is not a row count"))?,
                every: every.parse().map_err(|_| format!("{every} is not a stride"))?,
                asked: asked.parse().map_err(|_| format!("{asked} is not a row count"))?,
            })
        }
    };

    let loaded = Loaded {
        took: Duration::from_micros(
            need("load")?.parse().map_err(|_| format!("{engine} has an unreadable load time"))?,
        ),
        cpu: value(block, "load-cpu")
            .and_then(maybe)
            .and_then(|n| n.parse().ok())
            .map(Duration::from_micros),
        on_disk: need("load-disk")?
            .parse()
            .map_err(|_| format!("{engine} has an unreadable load size"))?,
        on_disk_is: unescape(need("load-what")?),
        converted: need("load-conv")? == "yes",
    };

    // The query lines are one key and the five lines under each are five more, so the block is cut
    // at every `query` and each piece read on its own. Reading by key over the whole block would
    // find the first query's wall clock forty three times.
    let mut queries = Vec::new();
    for piece in block.split("query     ").skip(1) {
        queries.push(one_query(engine, piece)?);
    }

    let source_bytes =
        need("source")?.parse().map_err(|_| format!("{engine} has an unreadable source size"))?;
    Ok((
        SuiteResult {
            suite,
            engine: engine.to_owned(),
            version: unescape(need("version")?),
            loaded,
            queries,
            missing: value(block, "missing")
                .map(|m| m.split_whitespace().map(str::to_owned).collect())
                .unwrap_or_default(),
            sample,
            rows: value(block, "rows").and_then(|r| r.parse().ok()),
            keeps_state: need("state")? == "kept",
            load: value(block, "loadavg").and_then(loadavg),
            cold_forced: value(block, "cold-is").is_some_and(|how| how.trim() == "dropped"),
        },
        source_bytes,
    ))
}

/// One query, from the piece of a block that starts at its name.
fn one_query(engine: &str, piece: &str) -> Result<QueryResult, String> {
    let mut lines = piece.lines();
    let head = lines.next().unwrap_or_default();
    let (name, shape) = head.split_once(' ').unwrap_or((head, ""));
    let body: String = lines.map(|l| format!("{}\n", l.trim_start())).collect();

    let at = |key: &str| {
        value(&body, key)
            .map(str::to_owned)
            .ok_or_else(|| format!("{engine} {name} has no `{key}` line"))
    };
    let clickbench = value(&body, "by") == Some("clickbench");
    let summarize = |samples: Vec<Duration>| {
        if clickbench { Distribution::clickbench(samples) } else { Distribution::median(samples) }
    };
    let split_runs = |text: &str| -> Result<Runs, String> {
        let every = unlist(text)?;
        let Some((cold, hot)) = every.split_first() else {
            return Err(format!("{engine} {name} saved no runs at all"));
        };
        if hot.is_empty() {
            return Err(format!("{engine} {name} saved a cold run and no hot one"));
        }
        Ok(Runs { cold: *cold, hot: summarize(hot.to_vec()) })
    };

    Ok(QueryResult {
        name: name.to_owned(),
        shape: unescape(shape),
        runs: split_runs(&at("wall")?)?,
        reported: match value(&body, "said") {
            Some(text) => Some(split_runs(text)?),
            None => None,
        },
        cold: uncost(&at("cold")?)?,
        hot: uncost(&at("hot")?)?,
        // An empty answer is a real answer, so this is not `need`. A query whose result set is empty
        // writes an empty line and reading it back as missing would turn every one of them into a
        // disagreement the first time two engines were compared out of this file.
        answer: value(&body, "answer").map(unescape).unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{escape, read_one, unescape, write_one};
    use crate::engine::Loaded;
    use crate::measure::{Distribution, Runs};
    use crate::memory::{Cost, Peak};
    use crate::report::{QueryResult, SuiteResult};
    use crate::suite::find;

    fn ms(values: &[u64]) -> Vec<Duration> {
        values.iter().copied().map(Duration::from_millis).collect()
    }

    fn query(name: &str) -> QueryResult {
        QueryResult {
            name: name.to_owned(),
            shape: "group by, low card".to_owned(),
            runs: Runs {
                cold: Duration::from_millis(90),
                hot: Distribution::median(ms(&[10, 20, 30])),
            },
            reported: Some(Runs {
                cold: Duration::from_millis(70),
                hot: Distribution::median(ms(&[5, 15, 25])),
            }),
            cold: Cost {
                peak: Peak::Bytes(1024),
                cpu: Some(Duration::from_millis(40)),
                read: Some(7),
            },
            hot: Cost {
                peak: Peak::Unavailable("this is not the server".to_owned()),
                cpu: None,
                read: None,
            },
            answer: "1,2\n3,4".to_owned(),
        }
    }

    fn result() -> SuiteResult {
        SuiteResult {
            suite: find("smoke").unwrap(),
            engine: "duckdb".to_owned(),
            version: "v1.5.5 (Variegata) d8cdaa33fd".to_owned(),
            loaded: Loaded {
                took: Duration::from_millis(4757),
                on_disk: 35_399_680,
                on_disk_is: "its own database file".to_owned(),
                converted: true,
                cpu: Some(Duration::from_millis(3560)),
            },
            queries: vec![query("q1"), query("q2")],
            missing: vec!["q19".to_owned()],
            sample: None,
            rows: Some(99_998),
            keeps_state: false,
            load: None,
            cold_forced: false,
        }
    }

    #[test]
    fn a_result_that_went_through_the_file_is_the_result_that_went_in() {
        // The whole point of saving per engine is that the table is built later, so anything this
        // loses is a column that will be wrong in a report nobody can re-run cheaply.
        let before = result();
        let text = write_one(&before, "vmi3391933", 15_848_298);
        let (after, bytes) = read_one("duckdb", &text).unwrap();
        assert_eq!(before, after);
        assert_eq!(bytes, 15_848_298);
    }

    #[test]
    fn how_busy_the_machine_was_survives_the_file() {
        // It qualifies every number in the block, so losing it in the round trip would mean the
        // assembled table silently drops the one thing that says a column is not comparable.
        let mut before = result();
        before.load = Some((0.31, 18.75));
        let after = read_one("duckdb", &write_one(&before, "here", 1)).unwrap().0;
        assert_eq!(after.load, Some((0.31, 18.75)));

        // And a machine that does not publish one comes back as not published rather than as idle.
        let mut quiet = result();
        quiet.load = None;
        assert_eq!(read_one("duckdb", &write_one(&quiet, "here", 1)).unwrap().0.load, None);
    }

    #[test]
    fn what_the_cold_column_means_survives_the_file() {
        // A table assembled a week later has to know whether cold meant off the device or merely
        // first, and the two differ by the whole cost of reading the file.
        let mut dropped = result();
        dropped.cold_forced = true;
        assert!(read_one("duckdb", &write_one(&dropped, "here", 1)).unwrap().0.cold_forced);

        let warm = result();
        assert!(!read_one("duckdb", &write_one(&warm, "here", 1)).unwrap().0.cold_forced);

        // A block written before the line existed is read as the weaker of the two, which is what
        // those runs actually were.
        let old = write_one(&warm, "here", 1).replace("cold-is   first\n", "");
        assert!(!read_one("duckdb", &old).unwrap().0.cold_forced);
    }

    #[test]
    fn the_samples_survive_and_not_just_the_median() {
        // A saved median cannot be recombined with anything, and rule two says the spread travels
        // with the median. So the file keeps every run.
        let after = read_one("duckdb", &write_one(&result(), "here", 1)).unwrap().0;
        assert_eq!(after.queries[0].runs.hot.runs(), 3);
        assert_eq!(after.queries[0].runs.hot.samples(), ms(&[10, 20, 30]));
        assert_eq!(after.queries[0].reported.as_ref().unwrap().hot.samples(), ms(&[5, 15, 25]));
    }

    #[test]
    fn a_peak_that_was_a_sentence_comes_back_as_that_sentence() {
        // Rule six says a peak is either a number or the reason there is not one. A file that read
        // the reason back as a zero would be publishing a memory result of nothing at all.
        let after = read_one("duckdb", &write_one(&result(), "here", 1)).unwrap().0;
        assert_eq!(
            after.queries[0].hot.peak,
            Peak::Unavailable("this is not the server".to_owned())
        );
    }

    #[test]
    fn an_answer_with_newlines_in_it_does_not_become_five_new_keys() {
        // Every answer in the suite has newlines in it, and the file is one key per line, so this is
        // the failure the format would have by default rather than an edge case.
        let text = write_one(&result(), "here", 1);
        assert!(text.contains("1,2\\n3,4"), "{text}");
        assert_eq!(read_one("duckdb", &text).unwrap().0.queries[0].answer, "1,2\n3,4");
    }

    #[test]
    fn a_backslash_in_an_answer_is_not_a_newline_next_time() {
        let awkward = "a\\nb\nc\\\\";
        assert_eq!(unescape(&escape(awkward)), awkward);
    }

    #[test]
    fn an_empty_answer_reads_back_as_empty_and_not_as_missing() {
        // A query with no rows is a real answer. Reading it back as absent would make every engine
        // disagree with every other engine about it.
        let mut one = result();
        one.queries[0].answer = String::new();
        let after = read_one("duckdb", &write_one(&one, "here", 1)).unwrap().0;
        assert_eq!(after.queries[0].answer, "");
    }
}
