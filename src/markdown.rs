//! The same run as [`crate::report`], written out as a file somebody can read a week later.
//!
//! The plain text table is what a person watching a run wants. It is fixed width, it fits a
//! terminal, and it drops everything that does not fit, which on ClickBench means five engines and
//! forty three queries reduce to one hot median a cell. That is the right trade for a terminal and
//! the wrong one for the artifact, because the numbers that got dropped are the ones somebody asks
//! about afterwards: what the spread was, how many cores it used, how much it read, what the rate
//! per row was, what the machine was at the time.
//!
//! So this module writes the whole thing. Every metric the harness measured appears here, the cross
//! engine grid is still the first table because that is the one people read, and everything the
//! plain text report says under its table in prose is said here too. Nothing is computed that is
//! not also computed for the terminal, because a report with a number the terminal cannot show is a
//! report whose number nobody ever checked.
//!
//! Markdown, and specifically markdown that passes `tests/style.rs`. Reports live in `reports/` and
//! get committed, that test walks every markdown file in the repository, and a generator that
//! emitted an em dash would produce a file that fails the build of the run after it.

use std::path::{Path, PathBuf};

use crate::machine::Fact;
use crate::measure::{Convention, Distribution, show};
use crate::memory::{Peak, bytes};
use crate::report::{Comparison, SuiteResult, publishable};

/// Where the report for one suite goes.
///
/// Named after the suite and the machine, both, because rule seven says never compare across
/// machines and two files that differ only in which box they were taken on would be compared by
/// the first person who opened them side by side. The `run-` prefix keeps the generated reports
/// apart from the written ones, which live in the same directory and are a different kind of
/// document.
///
/// `RUDB_BENCH_REPORTS` moves the directory, the way `RUDB_BENCH_BASELINE` moves the records, for
/// a run somewhere other than a checkout.
#[must_use]
pub fn path(suite: &str, machine: &str) -> PathBuf {
    let name = format!("run-{suite}-{machine}.md");
    std::env::var_os("RUDB_BENCH_REPORTS")
        .map_or_else(|| Path::new("reports").join(&name), |at| PathBuf::from(at).join(&name))
}

/// Write the report and return where it went.
///
/// # Errors
///
/// When the directory could not be made or the file could not be written.
pub fn write(
    compared: &Comparison,
    facts: &[Fact],
    machine: &str,
) -> Result<PathBuf, std::io::Error> {
    let at = path(compared.suite.name, machine);
    if let Some(parent) = at.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&at, render(compared, facts, machine))?;
    Ok(at)
}

/// The whole report as one string.
#[must_use]
pub fn render(compared: &Comparison, facts: &[Fact], machine: &str) -> String {
    let mut out = String::new();
    heading(&mut out, 1, &format!("{} on {machine}", compared.suite.name));
    out.push_str(&opening(compared, machine));

    heading(&mut out, 2, "What ran");
    out.push_str(&what_ran(compared));

    heading(&mut out, 2, "How to reproduce it");
    out.push_str(&reproduce(compared));

    heading(&mut out, 2, "The machine");
    out.push_str(&machine_table(facts));

    heading(&mut out, 2, "The engines");
    out.push_str(&engines(compared));

    heading(&mut out, 2, "Totals");
    out.push_str(&totals(compared));

    heading(&mut out, 2, "Time per query");
    out.push_str(&per_query(compared));

    for result in &compared.results {
        heading(&mut out, 2, &format!("{} in full", result.engine));
        out.push_str(&detail(result, compared.source_bytes));
    }

    heading(&mut out, 2, "What this number is not");
    out.push_str(&caveats(compared));

    out
}

/// The paragraph under the title, which says what the document is before any number appears.
fn opening(compared: &Comparison, machine: &str) -> String {
    let engines = compared.results.len();
    let queries = compared.results.first().map_or(0, |r| r.queries.len());
    let runs =
        compared.results.first().map_or(0, |r| r.queries.first().map_or(0, |q| q.runs.hot.runs()));
    let mut out = format!(
        "This is one run of the {} suite on {machine}, over {engines} engine{} and {queries} \
         quer{}, with {runs} hot run{} of each query after one cold one. It was written by \
         `rudb-bench run --report` and nothing in it was typed by hand. The command that \
         reproduces it is below.\n\n",
        compared.suite.name,
        plural(engines),
        if queries == 1 { "y" } else { "ies" },
        plural(runs),
    );
    if let Some(sample) = compared.sample {
        out.push_str(&format!(
            "It ran over {}, which is a development loop and not the suite. Nothing here is \
             comparable to a full run or to anybody else's number.\n\n",
            sample.sentence()
        ));
    }
    out
}

/// What the suite was and what it was pointed at.
fn what_ran(compared: &Comparison) -> String {
    let mut rows = vec![
        vec!["suite".to_owned(), compared.suite.name.to_owned()],
        vec!["queries".to_owned(), compared.suite.queries.to_string()],
        vec![
            "tables".to_owned(),
            format!(
                "{} of Parquet in {} table{}",
                bytes(compared.source_bytes),
                compared.suite.tables.len(),
                plural(compared.suite.tables.len())
            ),
        ],
    ];
    rows.push(vec![
        "rows".to_owned(),
        compared.rows.map_or_else(
            || "the suite does not declare one".to_owned(),
            |n| format!("{n} in the table every query reads"),
        ),
    ]);
    if let Some(sample) = compared.sample {
        rows.push(vec!["sample".to_owned(), sample.sentence()]);
    }
    rows.push(vec![
        "summary".to_owned(),
        match compared.results.first().map(|r| r.queries.first().map(|q| q.runs.hot.convention())) {
            Some(Some(Convention::Clickbench)) => {
                "best of three, which is the ClickBench board's convention".to_owned()
            }
            Some(Some(Convention::Median)) => {
                "median with the interquartile range, per reporting rule two".to_owned()
            }
            _ => "nothing ran".to_owned(),
        },
    ]);
    rows.push(vec!["harness".to_owned(), format!("rudb-bench {}", env!("CARGO_PKG_VERSION"))]);
    table(&["what", "it was"], &rows)
}

/// The one documented command, which is reporting rule eight.
fn reproduce(compared: &Comparison) -> String {
    let mut command = format!("rudb-bench run {}", compared.suite.name);
    // What was asked for rather than what came out. One row in every hundred of a hundred million
    // does not land on a round number, and asking for the number that came out would pick a
    // different stride and build a different file.
    if let Some(sample) = compared.sample {
        command.push_str(&format!(" --rows {}", sample.asked));
    }
    // Only when something was held back on purpose. An engine that is not on the machine is not on
    // it for the next run either and does not need naming, but one that was left out by the flag
    // comes back the moment the flag goes, and then the command gives a different table.
    if compared.skipped.iter().any(|s| s.unasked) {
        let asked: Vec<&str> = compared.results.iter().map(|r| r.engine.as_str()).collect();
        command.push_str(&format!(" --engines {}", asked.join(",")));
    }
    let runs = compared
        .results
        .first()
        .and_then(|r| r.queries.first().map(|q| q.runs.hot.runs()))
        .unwrap_or(0);
    if runs > 0 {
        command.push_str(&format!(" --runs {runs}"));
    }
    command.push_str(" --report");
    format!(
        "Reporting rule eight says a published number is reproducible by one documented command on \
         a named machine type. This is the command.\n\n```\n{command}\n```\n\nThe data goes under \
         `RUDB_BENCH_DATA`, which defaults to `~/rudb-data`, and the engines have to be on the \
         path. An engine that is not gets a row in the abstentions below rather than being left \
         out of the table.\n\n"
    )
}

/// Everything about the machine that has to be read next to a number.
fn machine_table(facts: &[Fact]) -> String {
    let rows: Vec<Vec<String>> = facts
        .iter()
        .map(|fact| {
            vec![
                fact.name.to_owned(),
                fact.value.clone(),
                if fact.known { "read".to_owned() } else { "not read".to_owned() },
            ]
        })
        .collect();
    let mut out = table(&["fact", "value", "source"], &rows);
    out.push_str(
        "A fact that says it was not read is a fact about this machine and not a gap in the \
         report. A field that had silently defaulted would be a lie that survived into it.\n\n",
    );
    out
}

/// One row per engine: what it is, what it did to the data, and what that cost.
fn engines(compared: &Comparison) -> String {
    let mut rows: Vec<Vec<String>> = compared
        .results
        .iter()
        .map(|r| {
            vec![
                r.engine.clone(),
                r.version.clone(),
                "ran".to_owned(),
                show(r.loaded.took),
                r.loaded.cpu.map_or_else(|| "not read".to_owned(), show),
                bytes(r.loaded.on_disk),
                r.loaded.on_disk_is.clone(),
                if r.loaded.converted { "its own".to_owned() } else { "the Parquet".to_owned() },
                r.load.map_or_else(
                    || "not read".to_owned(),
                    |(before, after)| format!("{before:.2} to {after:.2}"),
                ),
            ]
        })
        .collect();
    for skip in &compared.skipped {
        rows.push(vec![
            skip.engine.clone(),
            skip.version.clone(),
            skip.why.clone(),
            "n/a".to_owned(),
            "n/a".to_owned(),
            "n/a".to_owned(),
            "n/a".to_owned(),
            "n/a".to_owned(),
            "n/a".to_owned(),
        ]);
    }
    let mut out = table(
        &[
            "engine",
            "version",
            "state",
            "load",
            "load cpu",
            "on disk",
            "that size is",
            "format",
            "machine load",
        ],
        &rows,
    );
    let reading: Vec<&str> = compared
        .results
        .iter()
        .filter(|r| !r.loaded.converted)
        .map(|r| r.engine.as_str())
        .collect();
    if !reading.is_empty() {
        out.push_str(&format!(
            "Reading the Parquet where it lies rather than a format of its own: {}. That is an \
             empty load column and a decode inside every query, in the column being compared, \
             which the other engines paid for once at load time.\n\n",
            reading.join(", ")
        ));
    }
    let threads = crate::machine::threads_here();
    #[expect(clippy::cast_precision_loss, reason = "core counts are small integers")]
    let room = threads as f64 / 2.0;
    let busy: Vec<&str> = compared
        .results
        .iter()
        .filter(|r| r.busiest().is_some_and(|l| l > room))
        .map(|r| r.engine.as_str())
        .collect();
    if !busy.is_empty() {
        out.push_str(&format!(
            "The machine load column is the one minute load average before and after that \
             engine's suite, on a machine with {threads} hardware threads. These engines ran while \
             it was above half of that, which means they were measured against somebody else's \
             work rather than on an idle box: {}. Their numbers are inflated by an amount nothing \
             here can recover, and by a different amount each, depending on how much of the column \
             was CPU bound.\n\n",
            busy.join(", ")
        ));
    }
    // Said every time rather than only when it is the weaker of the two, because a reader who sees
    // nothing about the page cache assumes the stronger one, and for most of this project's history
    // the stronger one was not what happened.
    let forced = compared.results.iter().filter(|r| r.cold_forced).count();
    if forced == compared.results.len() && !compared.results.is_empty() {
        out.push_str(
            "The cold column is a run that had to go to the device. The page cache was dropped \
             before each query's first run, the way official ClickBench does it.\n\n",
        );
    } else if forced == 0 {
        out.push_str(
            "The cold column is the first run of each query rather than a run that had to go to \
             the device. The page cache was not dropped, so the file was still in memory from \
             whatever read it last. That makes cold a warm number taken before the others rather \
             than a measure of what a first pass off the disk costs, and the gap between the two \
             is the whole of the read. Set `RUDB_BENCH_DROP_CACHES=1` and run as root to get the \
             other one.\n\n",
        );
    } else {
        let names: Vec<&str> =
            compared.results.iter().filter(|r| r.cold_forced).map(|r| r.engine.as_str()).collect();
        out.push_str(&format!(
            "The cold column means two different things in this table, which is the one way it \
             should never be read. These engines had the page cache dropped before each query's \
             first run: {}. The rest were measured with whatever was already in memory, so their \
             cold numbers are lower for a reason that has nothing to do with the engine.\n\n",
            names.join(", ")
        ));
    }
    out
}

/// The summary row per engine, which is the table most readers stop at.
fn totals(compared: &Comparison) -> String {
    let base = compared.results.first().map(|r| r.best_total().as_secs_f64()).filter(|s| *s > 0.0);
    let reference = compared.results.first().map_or("nothing", |r| r.engine.as_str());
    let rows: Vec<Vec<String>> = compared
        .results
        .iter()
        .map(|r| {
            vec![
                r.engine.clone(),
                r.reported_total().map_or_else(|| "not read".to_owned(), show),
                show(r.hot_total()),
                r.overhead()
                    .map_or_else(|| "not read".to_owned(), |o| format!("{:+.0}%", o * 100.0)),
                show(r.cold_total()),
                r.hot_cpu().map_or_else(|| "not read".to_owned(), show),
                r.cores_used().map_or_else(|| "not read".to_owned(), |c| format!("{c:.2}")),
                peak_cell(&r.peak()),
                r.hot_read().map_or_else(|| "not read".to_owned(), read_cell),
                rows_rate(r.rows_per_second()),
                bytes_rate(r.bytes_per_second(compared.source_bytes)),
                base.map_or_else(
                    || "n/a".to_owned(),
                    |b| format!("{:.2}x", r.best_total().as_secs_f64() / b),
                ),
            ]
        })
        .collect();
    let mut out = table(
        &[
            "engine",
            "query time",
            "wall time",
            "overhead",
            "cold total",
            "hot cpu",
            "cores",
            "peak RSS",
            "hot read",
            "rows/s",
            "bytes/s",
            &format!("vs {reference}"),
        ],
        &rows,
    );
    out.push_str(
        "Query time is what the engine itself says the queries took, added up over the hot runs. \
         Every engine here was asked, each in its own way, and it is the same number the public \
         ClickBench board publishes. Wall time is the clock this harness holds around the whole \
         subprocess, so it also pays for starting a process, linking it, opening a database and \
         printing the answer. Overhead is the difference as a fraction of the query time, and it is \
         the number that says how much of the wall clock column is this harness rather than the \
         engine.\n\n",
    );
    out.push_str(
        "Read the query time column when comparing engines and the wall time column when asking \
         what the run costs to sit through. They rank engines differently and that is the point: \
         the overhead is not the same for each engine, because a DuckDB starts in about forty \
         milliseconds, a `clickhouse local` in about three hundred, and Polars has to boot a Python \
         and import itself. On a small sample the wall clock column partly ranks process startup. \
         The ratio and both throughput columns are taken against query time wherever every engine \
         reported one.\n\n",
    );
    out.push_str(&format!(
        "The two throughput columns are the whole table read once per query, so the row count and \
         the file size times the {} queries over the total. It is a rate for the run and not a rate any one query reached, and it \
         is not comparable to the same number from a suite with a different number of queries. \
         Cores is CPU seconds over wall seconds, which is how many of this machine's threads the \
         engine actually kept busy, and it is the number that says whether two wall clocks on two \
         machines can be compared at all.\n\n",
        compared.results.first().map_or(0, |r| r.queries.len())
    ));
    out.push_str(
        "Hot read is what the block layer served during the hot runs. It should be nothing on a \
         machine with room for the data, and when it is not, the hot number is not a hot number.\n\n",
    );
    out
}

/// The cross engine grid, one row per query.
fn per_query(compared: &Comparison) -> String {
    let Some(reference) = compared.results.first() else {
        return "No engine produced a number.\n\n".to_owned();
    };
    let mut header = vec!["query".to_owned(), "shape".to_owned()];
    for result in &compared.results {
        header.push(result.engine.clone());
    }
    let mut rows = Vec::with_capacity(reference.queries.len());
    // By name and never by position, because a column short a query would otherwise put its q30
    // time on the q29 row, which is a wrong number rather than a missing one.
    for query in &reference.queries {
        let mut row = vec![query.name.clone(), query.shape.clone()];
        for result in &compared.results {
            row.push(match result.find(&query.name) {
                // The engine's own, falling back to the wall clock for an engine that would not
                // say. A cell is one or the other and the per engine tables below carry both, so a
                // reader who needs to know which this was has one place to look.
                Some(q) => {
                    show(q.reported.as_ref().map_or(q.runs.hot.headline(), |r| r.hot.headline()))
                }
                None if result.missing.contains(&query.name) => "no dialect".to_owned(),
                None => "did not run".to_owned(),
            });
        }
        rows.push(row);
    }
    let head: Vec<&str> = header.iter().map(String::as_str).collect();
    let mut out = table(&head, &rows);
    out.push_str(
        "The engine's own hot figure for each query, which is the one that compares across \
         columns. The wall clock, the spread and everything else are in the per engine tables \
         below, one of which is the whole distribution for every query.\n\n",
    );
    out
}

/// One engine, every metric, every query.
fn detail(result: &SuiteResult, source_bytes: u64) -> String {
    let rows: Vec<Vec<String>> = result
        .queries
        .iter()
        .map(|q| {
            vec![
                q.name.clone(),
                q.shape.clone(),
                q.reported
                    .as_ref()
                    .map_or_else(|| "not read".to_owned(), |r| show(r.hot.headline())),
                show(q.runs.cold),
                show(q.runs.hot.headline()),
                spread_cell(&q.runs.hot),
                show(q.runs.hot.p25()),
                show(q.runs.hot.p75()),
                show(q.runs.hot.fastest()),
                show(q.runs.hot.slowest()),
                q.hot.cpu.map_or_else(|| "not read".to_owned(), show),
                peak_cell(&q.peak()),
                read_cell(q.cold.read),
                query_rate(result.rows, q.runs.hot.headline()),
            ]
        })
        .collect();
    let mut out = table(
        &[
            "query",
            "shape",
            "query time",
            "cold",
            "hot",
            "IQR",
            "p25",
            "p75",
            "fastest",
            "slowest",
            "hot cpu",
            "peak RSS",
            "cold read",
            "rows/s",
        ],
        &rows,
    );
    out.push_str(&format!(
        "{} {} over {} of {} queries. Total {} by its own clock and {} by ours, {} cold, {} of \
         CPU, peak {}, {} and {}.\n\n",
        result.engine,
        result.version,
        result.queries.len(),
        result.queries.len() + result.missing.len(),
        result.reported_total().map_or_else(|| "no reading".to_owned(), show),
        show(result.hot_total()),
        show(result.cold_total()),
        result.hot_cpu().map_or_else(|| "no reading".to_owned(), show),
        peak_cell(&result.peak()),
        rows_rate(result.rows_per_second()),
        bytes_rate(result.bytes_per_second(source_bytes)),
    ));
    if let Some(overhead) = result.overhead() {
        out.push_str(&format!(
            "Running it cost {:.0}% on top of the queries themselves. That is process start, \
             linking, opening the data and printing the answer, and it is in every wall clock \
             figure in this section.\n\n",
            overhead * 100.0
        ));
    }
    if let Some(spread) = result.shape_spread() {
        out.push_str(&format!(
            "Its slowest query is {spread:.2}x its fastest. A column much flatter than that is \
             measuring whatever every query in it has in common rather than measuring the \
             queries.\n\n"
        ));
    }
    out
}

/// Everything the terminal report says in prose under its table.
fn caveats(compared: &Comparison) -> String {
    let mut out = String::new();

    let reasons: Vec<String> = compared.results.first().map(publishable).unwrap_or_default();
    if reasons.is_empty() {
        out.push_str(
            "Nothing here blocks publication. Reporting rule one still applies: state the \
             machine, the kernel, the filesystem and the settings next to the number.\n\n",
        );
    } else {
        out.push_str("This is not a publishable number, because:\n\n");
        for reason in &reasons {
            out.push_str(&format!("- {reason}\n"));
        }
        out.push('\n');
    }

    let disturbed = compared.disturbed();
    if !disturbed.is_empty() {
        out.push_str("These swung wider than reporting rule two allows:\n\n");
        for who in &disturbed {
            out.push_str(&format!("- {who}\n"));
        }
        out.push_str(
            "\nThat is either another tenant on this machine or a query so short that starting \
             the process is most of what got timed. Either way the ratio column compares two \
             numbers whose error bars are wider than the gap between them.\n\n",
        );
    }

    let flattened = compared.flattened();
    if !flattened.is_empty() {
        out.push_str("These ran every query at about the same speed:\n\n");
        for who in &flattened {
            out.push_str(&format!("- {who}\n"));
        }
        out.push_str(
            "\nA column that flat is not a column about the queries. Read those numbers as an \
             upper bound on the engine and not as a measurement of it.\n\n",
        );
    }

    for result in compared.results.iter().filter(|r| !r.missing.is_empty()) {
        let defined = crate::suite::queries(compared.suite.name).unwrap_or(&[]);
        for name in &result.missing {
            let why = defined
                .iter()
                .find(|q| q.name == name)
                .and_then(|q| q.absent_for(&result.engine))
                .unwrap_or("no text was declared for it");
            out.push_str(&format!("{} did not run {name}, because {why}.\n\n", result.engine));
        }
        out.push_str(&format!(
            "So the {} column is {} of {} queries and its ratio is over the shared ones.\n\n",
            result.engine,
            result.queries.len(),
            result.queries.len() + result.missing.len()
        ));
    }

    let disagreements = compared.disagreements();
    let undetermined = compared.undetermined();
    let diverged = compared.diverged();
    if disagreements.is_empty() && compared.results.len() > 1 {
        let total = compared.results.first().map_or(0, |r| r.queries.len());
        out.push_str(&format!(
            "All {} engines agreed on every answer the data settles, which is {} of {total} \
             queries, to the last significant digit of a double.\n\n",
            compared.results.len(),
            total - undetermined.len() - diverged.len()
        ));
    }
    for line in &disagreements {
        out.push_str(&format!("Answers differ, so this is not a comparison: {line}\n\n"));
    }
    if !undetermined.is_empty() {
        out.push_str(
            "These were answered differently and the data does not say which is right:\n\n",
        );
        for (name, why) in &undetermined {
            out.push_str(&format!("- {name}: {why}.\n"));
        }
        out.push_str(
            "\nSo they are not checked, and a wrong answer from any engine on one of them would \
             go unnoticed here. Every other query in the suite is checked in full.\n\n",
        );
    }
    if !diverged.is_empty() {
        out.push_str("These were answered differently and which engine is wrong is settled:\n\n");
        for (name, why) in &diverged {
            out.push_str(&format!("- {name}: {why}.\n"));
        }
        out.push_str(
            "\nSo they are a known difference rather than an open one, and the write up is where \
             to go to disagree with that.\n\n",
        );
    }

    let warm: Vec<&str> =
        compared.results.iter().filter(|r| r.keeps_state).map(|r| r.engine.as_str()).collect();
    if warm.is_empty() {
        out.push_str(
            "Hot here means page cache warm and not buffer pool warm, because every run is a \
             fresh process so that no query's number depends on the one before it.\n\n",
        );
    } else {
        out.push_str(&format!(
            "Hot here means page cache warm and not buffer pool warm for every row but {}, which \
             stayed up across the whole suite with its own caches warm. That is the stronger kind \
             of hot, so a ratio against that column is a ratio between two different \
             quantities.\n\n",
            warm.join(", ")
        ));
    }
    out
}

/// A markdown table, sized by nothing because markdown does not care.
///
/// The cells are not padded. A generator that padded them would produce a diff on every run where
/// one number got a digit longer and every other row moved, and the thing being read here is the
/// rendered table rather than the source.
fn table(header: &[&str], rows: &[Vec<String>]) -> String {
    let mut out = format!("| {} |\n", header.join(" | "));
    out.push_str(&format!("|{}\n", " --- |".repeat(header.len())));
    for row in rows {
        out.push_str(&format!("| {} |\n", row.join(" | ")));
    }
    out.push('\n');
    out
}

/// A heading with the blank line after it that markdown wants.
fn heading(out: &mut String, level: usize, text: &str) {
    out.push_str(&format!("{} {text}\n\n", "#".repeat(level)));
}

/// The interquartile range as a fraction of the median, or why there is not one.
fn spread_cell(d: &Distribution) -> String {
    d.relative_iqr().map_or_else(|| "n/a".to_owned(), |r| format!("{:.1}%", r * 100.0))
}

/// One peak, or the short form of why there is not one.
fn peak_cell(peak: &Peak) -> String {
    peak.bytes().map_or_else(|| "not read".to_owned(), bytes)
}

/// Bytes read, where a zero is a result and not a gap.
fn read_cell(read: impl Into<Option<u64>>) -> String {
    match read.into() {
        Some(0) => "none".to_owned(),
        Some(n) => bytes(n),
        None => "not read".to_owned(),
    }
}

/// Rows a second, or the reason there is not a number of them.
fn rows_rate(value: Option<f64>) -> String {
    value.map_or_else(|| "no row count".to_owned(), |n| format!("{}/s", Rounded(n)))
}

/// Bytes a second, in the same units every other size in the report is in.
fn bytes_rate(value: Option<f64>) -> String {
    value.map_or_else(
        || "no timing".to_owned(),
        |n| {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a byte rate rounded to whole bytes, which the formatter then rounds again"
            )]
            let whole = n as u64;
            format!("{}/s", bytes(whole))
        },
    )
}

/// Rows per second for one query, which is the table read once.
fn query_rate(rows: Option<u64>, took: std::time::Duration) -> String {
    let Some(rows) = rows else { return "no row count".to_owned() };
    let seconds = took.as_secs_f64();
    if seconds <= 0.0 {
        return "n/a".to_owned();
    }
    #[expect(clippy::cast_precision_loss, reason = "a row count at f64 precision is exact to 9 PB")]
    let scanned = rows as f64;
    rows_rate(Some(scanned / seconds))
}

/// A count printed with a magnitude suffix, so a column of them lines up to a reader.
#[derive(Debug)]
struct Rounded(f64);

impl std::fmt::Display for Rounded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(value) = *self;
        if value >= 1e9 {
            write!(f, "{:.2}G", value / 1e9)
        } else if value >= 1e6 {
            write!(f, "{:.2}M", value / 1e6)
        } else if value >= 1e3 {
            write!(f, "{:.2}K", value / 1e3)
        } else {
            write!(f, "{value:.0}")
        }
    }
}

/// The `s` on the end of a count of things, which appears often enough here to be worth a function.
const fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Rounded, path, render, reproduce, table};
    use crate::engine::Loaded;
    use crate::machine::Fact;
    use crate::measure::{Distribution, Runs};
    use crate::memory::{Cost, Peak};
    use crate::report::{Abstention, Comparison, QueryResult, SuiteResult};
    use crate::suite::find;

    fn ms(values: &[u64]) -> Distribution {
        Distribution::median(values.iter().copied().map(Duration::from_millis).collect())
    }

    fn query(name: &str, hot: &[u64]) -> QueryResult {
        QueryResult {
            name: name.to_owned(),
            shape: "count".to_owned(),
            runs: Runs { cold: Duration::from_millis(90), hot: ms(hot) },
            // The engine's own clock, always a little under the wall clock beside it, because the
            // wall clock also paid for a process to start.
            reported: Some(Runs {
                cold: Duration::from_millis(70),
                hot: ms(&hot.iter().map(|n| n.saturating_sub(20)).collect::<Vec<_>>()),
            }),
            cold: Cost {
                peak: Peak::Bytes(1 << 20),
                cpu: Some(Duration::from_millis(120)),
                read: Some(0),
            },
            hot: Cost {
                peak: Peak::Bytes(1 << 20),
                cpu: Some(Duration::from_millis(110)),
                read: Some(0),
            },
            answer: "42".to_owned(),
        }
    }

    fn result(engine: &str, hot: &[u64]) -> SuiteResult {
        SuiteResult {
            suite: find("smoke").unwrap(),
            engine: engine.to_owned(),
            version: "v1".to_owned(),
            loaded: Loaded {
                took: Duration::from_millis(500),
                on_disk: 1 << 20,
                on_disk_is: "its own database file".to_owned(),
                converted: true,
                cpu: Some(Duration::from_millis(900)),
            },
            queries: vec![query("q1", hot), query("q2", hot)],
            missing: Vec::new(),
            sample: None,
            rows: Some(10_000_000),
            keeps_state: false,
            load: None,
            cold_forced: false,
        }
    }

    fn compared() -> Comparison {
        Comparison {
            suite: find("smoke").unwrap(),
            source_bytes: 10 << 20,
            sample: None,
            rows: Some(10_000_000),
            results: vec![
                result("duckdb", &[10, 20, 30, 40, 50]),
                result("rudb", &[5, 6, 7, 8, 9]),
            ],
            skipped: vec![Abstention {
                engine: "polars".to_owned(),
                version: "none".to_owned(),
                why: "not installed here".to_owned(),
                unasked: false,
            }],
        }
    }

    fn facts() -> Vec<Fact> {
        crate::machine::probe(std::path::Path::new("."))
    }

    /// The prose rules from `tests/style.rs`, applied to something no human is going to proofread.
    ///
    /// Reports get committed and that test walks every markdown file in the repository, so a
    /// generator that emitted an em dash would break the build of the run after the one that wrote
    /// the file. Checking it here means the failure lands on the commit that caused it.
    #[test]
    fn what_it_writes_passes_the_prose_rules() {
        let text = render(&compared(), &facts(), "testbox");
        assert!(!text.contains('\u{2014}'), "an em dash");
        assert!(!text.contains('\u{2013}'), "an en dash");
        for line in text.lines() {
            assert!(!matches!(line.trim(), "---" | "***" | "___"), "a horizontal rule: {line}");
        }
    }

    /// Every metric the harness measures has to reach the file, because the whole point of this
    /// module is that the terminal table drops most of them.
    #[test]
    fn every_metric_reaches_the_report() {
        let text = render(&compared(), &facts(), "testbox");
        for wanted in [
            "query time",
            "wall time",
            "overhead",
            "cold total",
            "hot cpu",
            "cores",
            "peak RSS",
            "hot read",
            "throughput",
            "rows/s",
            "IQR",
            "p25",
            "p75",
            "fastest",
            "slowest",
            "load",
            "on disk",
            "q1",
            "q2",
            "duckdb",
            "rudb",
        ] {
            assert!(text.contains(wanted), "{wanted} is not in the report:\n{text}");
        }
    }

    /// The two clocks are both in the file, and the ratio is taken against the engine's own.
    #[test]
    fn the_engines_own_clock_is_the_one_the_comparison_is_made_on() {
        let text = render(&compared(), &facts(), "testbox");
        // The helper builds each hot run twenty milliseconds under the wall clock, over two
        // queries, so the engine's total is forty milliseconds under and the overhead is the
        // difference over the engine's own total rather than over the wall clock.
        let duckdb = &compared().results[0];
        assert_eq!(
            duckdb.hot_total() - duckdb.reported_total().expect("a reading"),
            Duration::from_millis(40)
        );
        let overhead = duckdb.overhead().expect("a reading");
        let wanted = 0.040 / duckdb.reported_total().expect("a reading").as_secs_f64();
        assert!((overhead - wanted).abs() < 1e-9, "{overhead} against {wanted}");
        // And the report says which of the two a reader is comparing on.
        assert!(text.contains("what the engine itself says"), "{text}");
        assert!(text.contains("taken against query time"), "{text}");
    }

    /// An engine that would not say what a query cost it leaves a gap rather than a zero.
    #[test]
    fn an_engine_with_no_clock_of_its_own_falls_back_to_the_wall_clock() {
        let mut compared = compared();
        for query in &mut compared.results[1].queries {
            query.reported = None;
        }
        let quiet = &compared.results[1];
        assert_eq!(quiet.reported_total(), None);
        assert_eq!(quiet.overhead(), None);
        // A zero here would make it the fastest engine in the table. The wall clock is the honest
        // fallback, because it is a number that was actually measured.
        assert_eq!(quiet.best_total(), quiet.hot_total());
        assert!(!quiet.total_is_reported());
        let text = render(&compared, &facts(), "testbox");
        assert!(text.contains("not read"), "{text}");
    }

    /// An engine that did not run is a sentence and never a blank, in the file as in the terminal.
    #[test]
    fn an_engine_that_did_not_run_says_so_rather_than_being_absent() {
        let text = render(&compared(), &facts(), "testbox");
        assert!(text.contains("polars"), "{text}");
        assert!(text.contains("not installed here"), "{text}");
    }

    /// The sample sentence travels with a smaller run, which is the rule that stops one being read
    /// as a result.
    #[test]
    fn a_smaller_run_says_so_in_the_first_paragraph() {
        let mut compared = compared();
        compared.sample = Some(crate::data::Sample {
            full: 10_000_000,
            rows: 100_000,
            every: 100,
            asked: 100_000,
        });
        let text = render(&compared, &facts(), "testbox");
        let first = text.split("## What ran").next().unwrap();
        assert!(first.contains("one out of every 100"), "{first}");
        assert!(first.contains("development loop"), "{first}");
    }

    /// A command that runs six engines does not reproduce a report that ran two of them.
    #[test]
    fn the_reproduce_command_names_the_engines_when_some_were_held_back() {
        let mut compared = compared();
        assert!(!reproduce(&compared).contains("--engines"), "nothing was held back");
        compared.skipped[0].unasked = true;
        let text = reproduce(&compared);
        assert!(text.contains("--engines duckdb,rudb"), "{text}");
    }

    #[test]
    fn the_file_is_named_after_the_suite_and_the_machine() {
        // Rule seven is never compare across machines, and two files that differ only in which box
        // they came from are two files somebody puts side by side.
        let at = path("clickbench", "server3");
        assert!(at.ends_with("run-clickbench-server3.md"), "{}", at.display());
    }

    #[test]
    fn a_table_is_a_header_a_rule_and_the_rows() {
        let out = table(&["a", "b"], &[vec!["1".to_owned(), "2".to_owned()]]);
        assert_eq!(out, "| a | b |\n| --- | --- |\n| 1 | 2 |\n\n");
    }

    #[test]
    fn a_rate_gets_a_magnitude_so_a_column_of_them_can_be_read() {
        assert_eq!(Rounded(12_345_678.0).to_string(), "12.35M");
        assert_eq!(Rounded(1_234.0).to_string(), "1.23K");
        assert_eq!(Rounded(12.0).to_string(), "12");
    }
}
