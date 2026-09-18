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
use crate::engine::{Loaded, Outcome};
use crate::measure::{Convention, Distribution, Runs};
use crate::memory::{Cost, Peak};
use crate::metrics::{Accounting, Classes, Internal, Spend};
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
#
# `inside` is what the engine said about its own execution, and only rudb says: the cpu its
# pipelines charged themselves, the cpu it measured around running them, the cpu the whole process
# was charged, the execute step, the cpu the pipelines spent outside any operator, the bytes it
# says it held at the peak, how many pipelines ran, how many operators ran, how many of those were
# reference implementations, and the cpu that went on building the tree before any pipeline
# existed. The first two are the cross check, and a query where they disagree is refused by
# `publishable` rather than dropped from the file.
#
# `spend` is the same breakdown folded by kind of operator, one line per kind: the kind, the cpu,
# the rows handed in, the rows handed on, how many operators that was, and how many of them were
# reference implementations. `inside` asks whether the breakdown adds up and this asks what to go
# and make faster, which is why both are here and neither replaces the other.
#
# `planner` is the class histogram: how many of that query's cardinality decisions rested on a
# counted number, a number with a proven bound, a guess, and nothing at all, in that order. Four
# counts rather than four fractions, because a suite's histogram is the sum of its queries' and
# fractions do not add.
#
# `timeout` and `failed` are the two ways a query can be in this file without a measurement in it.
# A query that ran out of its limit has the limit written down, because the limit is a true lower
# bound on what it would have taken. A query the engine could not answer has what the engine said
# instead, and it contributes nothing at all to any total, which is why a column with one of these
# in it prints its totals with a `>` in front of them.
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
    let corpus = one_corpus(&saved)?;
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

    // Not saved and so not restored. The limit is a property of the command that ran, and each
    // block in the file ran under its own. A query that hit one says so in its own row, which is
    // the part a reader of a restored table needs.
    Ok(Comparison {
        suite: found,
        source_bytes,
        sample,
        rows,
        corpus,
        results: saved,
        skipped,
        timeout: None,
    })
}

/// The corpus every block agrees it ran over, or the refusal naming the two that do not.
///
/// The same argument as the sample check above, one level down. Two engines saved a week apart can
/// have run over two different corpora at the same scale factor, because a corpus is regenerated by
/// a command somebody types and the row counts do not have to move for the files to. The manifest
/// digest is what makes that visible, and a table built out of two of them is a table where one
/// column answered a different question from the others.
///
/// A block with no manifest line does not match one that has a manifest line, on purpose. Not
/// knowing which files a block ran over is a different thing from knowing they were the same ones,
/// and the whole point of the check is to refuse to treat the first as the second.
fn one_corpus(saved: &[SuiteResult]) -> Result<Option<String>, String> {
    let corpus = saved.first().and_then(|first| first.corpus.clone());
    match saved.iter().find(|result| result.corpus != corpus) {
        None => Ok(corpus),
        Some(odd) => Err(format!(
            "{} ran over {} and {} ran over {}, which is not a comparison. Re-run one of them over \
             the other's corpus",
            saved[0].engine,
            corpus.as_deref().unwrap_or("a corpus with no manifest beside it"),
            odd.engine,
            odd.corpus.as_deref().unwrap_or("a corpus with no manifest beside it"),
        )),
    }
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

/// Every line with this key rather than the first, for the keys a query has several of.
fn values<'a>(block: &'a str, key: &'a str) -> impl Iterator<Item = &'a str> {
    block
        .lines()
        .filter_map(move |line| line.strip_prefix(key)?.strip_prefix(' ').map(str::trim_start))
}

/// One kind of operator and what it cost, as this file writes it.
///
/// The kind first and on its own, because it is the only field that is not a number and a kind with
/// a space in it would make every field after it land one place to the left. The engine names its
/// operators in one word today and this file does not depend on that staying true.
fn spent(s: &Spend) -> String {
    format!(
        "{} {} {} {} {} {}",
        s.kind.replace(' ', "-"),
        micros(s.cpu),
        s.rows_in,
        s.rows_out,
        s.operators,
        s.reference_impls
    )
}

/// Read that back.
fn unspend(text: &str) -> Result<Spend, String> {
    let parts: Vec<&str> = text.split_whitespace().collect();
    let [kind, cpu, rows_in, rows_out, operators, references] = parts[..] else {
        return Err(format!("a spend line needs six fields and this has `{text}`"));
    };
    let number = |field: &str, what: &str| {
        field.parse::<u64>().map_err(|_| format!("{field} is not {what}"))
    };
    Ok(Spend {
        kind: kind.to_owned(),
        cpu: Duration::from_micros(number(cpu, "a duration")?),
        rows_in: number(rows_in, "a row count")?,
        rows_out: number(rows_out, "a row count")?,
        operators: number(operators, "a count")? as usize,
        reference_impls: number(references, "a count")? as usize,
    })
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

/// What the engine said about its own execution, on one line.
///
/// Eleven fixed fields, in the order they are read back. Only rudb writes one, so the line is absent
/// for every other engine rather than being eleven dashes.
///
/// The build and the planning are last rather than beside the numbers they belong with, because
/// each was added after the line existed and a file written before it is a file somebody still
/// wants to open. Appending is what keeps that true, and it costs one arm in the reader.
fn inside(i: &Internal) -> String {
    format!(
        "{} {} {} {} {} {} {} {} {} {} {}",
        micros(i.accounting.accounted),
        micros(i.accounting.measured),
        i.accounting.process.map_or_else(|| "-".to_owned(), |d| micros(d).to_string()),
        micros(i.execute),
        micros(i.driver),
        i.peak_bytes,
        i.pipelines,
        i.operators,
        i.reference_impls,
        micros(i.accounting.build),
        micros(i.planning)
    )
}

/// Read that back.
fn uninside(text: &str) -> Result<Internal, String> {
    let parts: Vec<&str> = text.split_whitespace().collect();
    // Nine fields is a file written before the build was split out of the measured cpu, and zero is
    // what it meant at the time: everything the engine measured was on the right of the check. Ten
    // is a file written before the engine put a clock on its planner, and zero is what it meant
    // then too, because the three planning phases really did report nothing.
    let (
        [accounted, measured, process, execute, driver, peak, pipelines, operators, references],
        build,
        planning,
    ) = match parts[..] {
        [a, b, c, d, e, f, g, h, i] => ([a, b, c, d, e, f, g, h, i], "0", "0"),
        [a, b, c, d, e, f, g, h, i, j] => ([a, b, c, d, e, f, g, h, i], j, "0"),
        [a, b, c, d, e, f, g, h, i, j, k] => ([a, b, c, d, e, f, g, h, i], j, k),
        _ => {
            return Err(format!(
                "an inside line needs nine, ten or eleven fields and this has `{text}`"
            ));
        }
    };
    let time = |field: &str| -> Result<Duration, String> {
        field
            .parse::<u64>()
            .map(Duration::from_micros)
            .map_err(|_| format!("{field} is not a number of microseconds"))
    };
    let number = |field: &str| -> Result<u64, String> {
        field.parse::<u64>().map_err(|_| format!("{field} is not a number"))
    };
    Ok(Internal {
        accounting: Accounting {
            accounted: time(accounted)?,
            measured: time(measured)?,
            build: time(build)?,
            process: match maybe(process) {
                Some(field) => Some(time(field)?),
                None => None,
            },
        },
        execute: time(execute)?,
        planning: time(planning)?,
        driver: time(driver)?,
        peak_bytes: number(peak)?,
        pipelines: number(pipelines)? as usize,
        operators: number(operators)? as usize,
        reference_impls: number(references)? as usize,
    })
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
    // Which files this block was measured over, so that two blocks written a week apart can be
    // asked whether they are two columns of one table or two tables. A block written before this
    // line existed has no answer, and no answer is not the same as a matching one.
    if let Some(corpus) = &result.corpus {
        let _ = writeln!(out, "corpus    {}", escape(corpus));
    }
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
        if let Some(i) = &q.internal {
            let _ = writeln!(out, "  inside  {}", inside(i));
        }
        for s in &q.spend {
            let _ = writeln!(out, "  spend   {}", spent(s));
        }
        if let Some(c) = q.estimates {
            let _ = writeln!(
                out,
                "  planner {} {} {} {}",
                c.exact, c.certified, c.estimated, c.unknown
            );
        }
        // Only written when the limit fired. A block saved before this line existed reads back as
        // a query that finished, which is what every query in it did.
        if let Outcome::TimedOut { limit } = q.outcome {
            let _ = writeln!(out, "  timeout {}", micros(limit));
        }
        // The same idea for the other way a row can carry no measurement. Without this line a
        // failure would read back as a query that finished in no time at all, which is the one
        // reading this whole file exists to make impossible.
        if let Outcome::Failed { message } = &q.outcome {
            let _ = writeln!(out, "  failed  {}", escape(message));
        }
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
            corpus: value(block, "corpus").map(unescape),
        },
        source_bytes,
    ))
}

/// One class histogram, from the four counts on a `planner` line.
fn classes(text: &str) -> Result<Classes, String> {
    let fields: Vec<&str> = text.split_whitespace().collect();
    let [exact, certified, estimated, unknown] = fields.as_slice() else {
        return Err(format!("a planner line needs four counts and this has `{text}`"));
    };
    let number = |what: &str, field: &str| {
        field.parse::<u64>().map_err(|_| format!("{field} is not a count of {what} decisions"))
    };
    Ok(Classes {
        exact: number("exact", exact)?,
        certified: number("certified", certified)?,
        estimated: number("estimated", estimated)?,
        unknown: number("unknown", unknown)?,
    })
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

    // A limit that fired, if one did. It is kept out of the runs and put back in below, because a
    // saved sample that is really a limit has to come back out of the file counted as one. Reading
    // it as an ordinary sample would let a block that timed out be published, and the whole point
    // of writing the line was that it cannot be.
    let timeout = match value(&body, "timeout") {
        None => None,
        Some(text) => Some(Duration::from_micros(text.trim().parse::<u64>().map_err(|_| {
            format!("{engine} {name} has a timeout that is not a number of microseconds")
        })?)),
    };
    let failed = value(&body, "failed").map(unescape);
    let mut runs = split_runs(&at("wall")?)?;
    if timeout.is_some() || failed.is_some() {
        runs.hot = runs.hot.with_timeouts(1);
    }

    Ok(QueryResult {
        name: name.to_owned(),
        shape: unescape(shape),
        runs,
        // The limit first, because a query can only be one of these and a block that somehow had
        // both lines is a block whose query ran long before it fell over.
        outcome: match (timeout, failed) {
            (Some(limit), _) => Outcome::TimedOut { limit },
            (None, Some(message)) => Outcome::Failed { message },
            (None, None) => Outcome::Completed,
        },
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
        internal: match value(&body, "inside") {
            Some(text) => Some(uninside(text)?),
            None => None,
        },
        // Not saved and so not read back. A record holds one run's breakdown, and a share taken
        // over the runs cannot be rebuilt from it. Filling this with the one pair the record does
        // hold would hand a caller a sample of one wearing a distribution's name, which is the
        // thing this field was added to stop. What reads it is the budget, and the budget is
        // recorded from a run rather than from a record.
        planning: Vec::new(),
        spend: values(&body, "spend").map(unspend).collect::<Result<Vec<_>, _>>()?,
        estimates: match value(&body, "planner") {
            Some(text) => Some(classes(text)?),
            None => None,
        },
    })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{escape, one_corpus, read_one, unescape, write_one};
    use crate::engine::{Loaded, Outcome};
    use crate::measure::{Distribution, Runs};
    use crate::memory::{Cost, Peak};
    use crate::metrics::{Accounting, Classes, Internal, Spend};
    use crate::report::{QueryResult, SuiteResult};
    use crate::suite::{Query, find};

    fn ms(values: &[u64]) -> Vec<Duration> {
        values.iter().copied().map(Duration::from_millis).collect()
    }

    fn query(name: &str) -> QueryResult {
        QueryResult {
            name: name.to_owned(),
            shape: "group by, low card".to_owned(),
            outcome: Outcome::Completed,
            estimates: Some(Classes { exact: 0, certified: 0, estimated: 0, unknown: 7 }),
            // A record does not carry these, so a round trip through one comes back empty and the
            // fixture starts there. Putting a value here would make the round trip test assert that
            // a field survives a format that has no room for it.
            planning: Vec::new(),
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
            internal: Some(Internal {
                accounting: Accounting {
                    accounted: Duration::from_micros(38_000),
                    measured: Duration::from_micros(40_000),
                    build: Duration::from_micros(1_500),
                    process: Some(Duration::from_micros(95_000)),
                },
                execute: Duration::from_micros(41_000),
                planning: Duration::from_micros(2_600),
                driver: Duration::from_micros(900),
                peak_bytes: 8192,
                pipelines: 2,
                operators: 4,
                reference_impls: 4,
            }),
            spend: vec![
                Spend {
                    kind: "FileScan".to_owned(),
                    cpu: Duration::from_micros(30_000),
                    rows_in: 0,
                    rows_out: 1_000,
                    operators: 1,
                    reference_impls: 1,
                },
                Spend {
                    kind: "Filter".to_owned(),
                    cpu: Duration::from_micros(8_000),
                    rows_in: 1_000,
                    rows_out: 12,
                    operators: 2,
                    reference_impls: 2,
                },
            ],
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
            corpus: Some("duckdb-tpch SF1, corpus e02fbb7bb0145593, written 2026-09-18".to_owned()),
        }
    }

    #[test]
    fn two_engines_measured_over_two_corpora_are_not_one_table() {
        let mut first = result();
        first.engine = "duckdb".to_owned();
        let mut second = result();
        second.engine = "rudb".to_owned();
        assert_eq!(one_corpus(&[first.clone(), second.clone()]), Ok(first.corpus.clone()));

        second.corpus =
            Some("duckdb-tpch SF1, corpus 0d4c1f9a11b27e33, written 2026-09-19".to_owned());
        let e = one_corpus(&[first.clone(), second.clone()]).expect_err("two corpora");
        assert!(e.contains("e02fbb7bb0145593"), "{e}");
        assert!(e.contains("0d4c1f9a11b27e33"), "{e}");
        assert!(e.contains("not a comparison"), "{e}");

        // Not knowing is not the same as agreeing, so a block saved before manifests existed does
        // not quietly join a table built over a corpus that has one.
        second.corpus = None;
        let e = one_corpus(&[first, second]).expect_err("one of them cannot say");
        assert!(e.contains("no manifest beside it"), "{e}");
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

    /// A record written before the histogram existed is most of the history the board is built
    /// from, and reading it as a query with no decisions would be reading a measurement into a file
    /// that never took one.
    #[test]
    fn a_record_with_no_planner_line_comes_back_as_no_histogram_rather_than_zeroes() {
        let mut before = result();
        for query in &mut before.queries {
            query.estimates = None;
        }
        let text = write_one(&before, "here", 1);
        assert!(!text.contains("planner"), "{text}");
        let (after, _) = read_one("duckdb", &text).unwrap();
        assert_eq!(after.queries[0].estimates, None);
        assert_eq!(after, before);
    }

    /// Four counts, and a line that is not four counts is a file to refuse rather than a row to
    /// guess at.
    #[test]
    fn a_planner_line_that_is_not_four_counts_is_refused() {
        let text = write_one(&result(), "here", 1).replace("planner 0 0 0 7", "planner 0 0 7");
        let e = read_one("duckdb", &text).expect_err("three counts is not a histogram");
        assert!(e.contains("four counts"), "{e}");

        let text = write_one(&result(), "here", 1).replace("planner 0 0 0 7", "planner 0 0 0 lots");
        let e = read_one("duckdb", &text).expect_err("lots is not a count");
        assert!(e.contains("not a count of unknown decisions"), "{e}");
    }

    #[test]
    fn a_file_written_before_the_build_was_split_out_still_opens() {
        // Nine fields is every inside line saved up to now, and refusing to read them would throw
        // away the history the board is built from to gain a column that is zero in all of it.
        let before = result();
        let text = write_one(&before, "here", 1);
        let older: String = text
            .lines()
            .map(|line| match line.strip_prefix("  inside  ") {
                Some(fields) => {
                    let nine: Vec<&str> = fields.split_whitespace().collect();
                    format!("  inside  {}\n", nine[..9].join(" "))
                }
                None => format!("{line}\n"),
            })
            .collect();
        let after = read_one("duckdb", &older).unwrap().0;
        let inside = after.queries[0].internal.as_ref().expect("rudb wrote an inside line");
        assert_eq!(inside.accounting.build, Duration::ZERO, "no build means none of it was build");
        assert_eq!(inside.execute, before.queries[0].internal.as_ref().unwrap().execute);
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

    /// A query that ran out of time has to come back out of the file as one. Read back as an
    /// ordinary sample it would be a query that took exactly the limit, which is both wrong and
    /// publishable, and a table assembled a week later would print it next to real numbers.
    #[test]
    fn a_query_that_ran_out_of_time_comes_back_out_of_the_file_as_one() {
        let limit = Duration::from_secs(30);
        let mut before = result();
        before.queries[0].outcome = Outcome::TimedOut { limit };
        before.queries[0].runs.hot = Distribution::median(vec![limit]).with_timeouts(1);

        let text = write_one(&before, "here", 1);
        assert!(text.contains("  timeout "), "{text}");
        let after = read_one("duckdb", &text).unwrap().0;
        assert_eq!(after.queries[0].outcome, Outcome::TimedOut { limit });
        assert_eq!(after.queries[0].runs.hot.timeouts(), 1);
        assert!(!after.queries[0].runs.hot.publishable());
        assert_eq!(after.unmeasured(), 1);
    }

    /// A query the engine could not answer has to come back as one too, and for a worse reason. It
    /// has no samples of its own, so read back as an ordinary row it would be a query that finished
    /// instantly, which is the fastest number in the file.
    #[test]
    fn a_query_the_engine_could_not_answer_comes_back_out_of_the_file_as_one() {
        let said = "rudb failed: Binder Error: column must appear in the GROUP BY clause";
        let mut before = result();
        let query =
            Query { name: "q1", sql: "SELECT 1", shape: "group by, low card", dialects: &[] };
        before.queries[0] = QueryResult::that_failed(&query, said.to_owned());

        let text = write_one(&before, "here", 1);
        assert!(text.contains("  failed  "), "{text}");
        let after = read_one("duckdb", &text).unwrap().0;
        assert_eq!(after.queries[0].outcome, Outcome::Failed { message: said.to_owned() });
        assert!(!after.queries[0].runs.hot.publishable());
        assert_eq!(after.unmeasured(), 1);
        assert_eq!(after.queries[0], before.queries[0]);
    }

    /// And a block written before the line existed is a block where every query finished, which is
    /// what those runs were: a query that did not come back took the whole run with it.
    #[test]
    fn a_block_from_before_the_limit_existed_reads_as_a_run_where_everything_finished() {
        let after = read_one("duckdb", &write_one(&result(), "here", 1)).unwrap().0;
        assert_eq!(after.queries[0].outcome, Outcome::Completed);
        assert_eq!(after.unmeasured(), 0);
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
