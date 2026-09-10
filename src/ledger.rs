//! The attribution ledger: what each layer of the engine actually bought.
//!
//! Section 2.8 of `spec/engine/02-baseline.md` in tamnd/rudb asks for a series rather than a table.
//! Each layer closes with a row, the row carries the before and the after on total time, CPU
//! seconds, peak resident and bytes read, on the same machine, and the versions of the comparison
//! engines are held fixed across a layer or restated when they move. The release notes are then
//! written from the ledger, on the grounds that a release note claiming the hash table made joins
//! faster and unable to point at a row is a release note that is guessing.
//!
//! The ledger is generated and never written by hand, for the reason the specification gives: a
//! hand maintained ledger is one that acquires a good number nobody can reproduce. What is
//! committed is the runs, in `runs/<suite>.txt`, and `rudb-bench ledger` renders them.
//!
//! ## Why this is not the regression records
//!
//! `baselines/<suite>.txt` and `runs/<suite>.txt` look similar and answer different questions, and
//! keeping one file for both would make each of them worse. A record is per query and holds three
//! quartiles, because the gate's question is whether one query got twice as slow and its samples no
//! longer overlap. A run is per engine and holds totals, because the ledger's question is what a
//! layer bought across a whole suite. A record is replaced every time it is taken, since the gate
//! only ever compares against the current bar. A run is appended and never replaced, since the
//! ledger is a history and a history that overwrites itself is a table.
//!
//! ## What a row is a row about
//!
//! rudb, mostly. The rivals are in every run so that a layer which appears to have bought something
//! can be checked against the possibility that DuckDB got slower on the same machine on the same
//! afternoon, which is the failure this whole file exists to make visible. When a rival's version
//! moves between two rows the ledger says so on the row, because a ratio across that boundary is a
//! ratio between two comparisons.
//!
//! The commit in a run is this harness's commit and not rudb's. It has to be, since the harness is
//! what is running, and it is worth having: a change to the measurement path moves a number exactly
//! the way a change to the engine does, and the ledger should not be able to hide that. rudb's own
//! version travels in its engine block like every other engine's does.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crate::measure::show;
use crate::memory::bytes;
use crate::report::{Comparison, SuiteResult};

/// What one engine totalled in one stored run.
///
/// Totals rather than per query numbers, because a layer is judged on a suite. The per query detail
/// of any given afternoon is in the table that run printed, and storing it here as well would make
/// the ledger a second copy of the regression records that drifts from the first one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Standing {
    /// The engine, as it names itself.
    pub engine: String,
    /// Its exact version, per rule one, which is what tells a later row that a rival moved.
    pub version: String,
    /// Total hot wall clock over every query in the suite.
    pub hot: Duration,
    /// Total CPU seconds over the hot runs, when the machine would say.
    ///
    /// This is the axis section 2.5 calls the one that decides whether the project is real, since a
    /// wall clock win bought with threads is a scheduling result and not an engine result.
    pub cpu: Option<Duration>,
    /// Worst peak resident set over the suite, when every query reported one.
    pub peak: Option<u64>,
    /// Bytes read at the block layer over the hot runs, when the machine would say.
    pub read: Option<u64>,
    /// What the load cost in wall clock.
    pub load: Duration,
    /// What the data takes on disk in this engine's format afterwards.
    pub disk: u64,
}

impl Standing {
    /// Take an engine's totals out of a finished suite result.
    #[must_use]
    pub fn of(result: &SuiteResult) -> Self {
        Self {
            engine: result.engine.clone(),
            version: result.version.clone(),
            hot: result.hot_total(),
            cpu: result.hot_cpu(),
            peak: result.peak().bytes(),
            read: result.hot_read(),
            load: result.loaded.took,
            disk: result.loaded.on_disk,
        }
    }
}

/// One stored run: a suite, on a machine, at a commit, closing a layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stored {
    /// The suite that ran.
    pub suite: String,
    /// The machine it ran on. Rows only ever pair runs from the same one.
    pub machine: String,
    /// The layer this run closes, first token being the layer name and the rest being free text.
    pub layer: String,
    /// This harness's commit, short.
    pub commit: String,
    /// The date, so that a row can be read against what else was happening that week.
    pub recorded: String,
    /// Every engine that ran, in the order the report put them.
    pub standings: Vec<Standing>,
}

impl Stored {
    /// Take a whole comparison as one stored run.
    #[must_use]
    pub fn of(
        compared: &Comparison,
        machine: &str,
        layer: &str,
        commit: &str,
        today: &str,
    ) -> Self {
        Self {
            suite: compared.suite.name.to_owned(),
            machine: machine.to_owned(),
            layer: layer.to_owned(),
            commit: commit.to_owned(),
            recorded: today.to_owned(),
            standings: compared.results.iter().map(Standing::of).collect(),
        }
    }

    /// The layer name on its own, which is the first token of the layer field.
    ///
    /// Free text after it so that a run can be stored as `2a the baseline` and read by somebody who
    /// does not have the plan document open. Grouping on the first token means the descriptive part
    /// can be improved later without splitting a layer into two rows.
    #[must_use]
    pub fn layer_name(&self) -> &str {
        self.layer.split_whitespace().next().unwrap_or(&self.layer)
    }

    /// What one engine did in this run, by name.
    #[must_use]
    pub fn standing(&self, engine: &str) -> Option<&Standing> {
        self.standings.iter().find(|s| s.engine == engine)
    }
}

/// One line of the ledger: a layer, on a suite, on a machine, with what it closed and what it
/// followed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// The run that closed this layer.
    pub after: Stored,
    /// The run that closed the layer before it on the same suite and machine, when there was one.
    ///
    /// `None` on the first row of a series, which is not a defect and prints as a sentence saying
    /// this is where the series starts.
    pub before: Option<Stored>,
}

impl Row {
    /// The layer name.
    #[must_use]
    pub fn layer(&self) -> &str {
        self.after.layer_name()
    }

    /// Every engine whose version is not the same as it was on the previous row.
    ///
    /// The sentence this produces is the one that stops a layer taking credit for a rival's
    /// release. An engine that is new in the after, or gone from it, is reported the same way and
    /// for the same reason.
    #[must_use]
    pub fn moved(&self) -> Vec<String> {
        let Some(before) = self.before.as_ref() else {
            return Vec::new();
        };
        let mut moved = Vec::new();
        for now in &self.after.standings {
            match before.standing(&now.engine) {
                Some(was) if was.version == now.version => {}
                Some(was) => moved
                    .push(format!("{} moved from {} to {}", now.engine, was.version, now.version)),
                None => moved.push(format!("{} was not in the row before this one", now.engine)),
            }
        }
        for was in &before.standings {
            if self.after.standing(&was.engine).is_none() {
                moved.push(format!(
                    "{} ran on the row before this one and not on this one",
                    was.engine
                ));
            }
        }
        moved
    }
}

/// Build the ledger out of stored runs.
///
/// Grouped by suite and machine, because a row that paired two machines would be a row about the
/// machines. Within a group the layers come in the order they were first stored, and a layer stored
/// more than once is represented by its last run, which is what closing a layer means: the run that
/// was true when the work was done, not the one taken halfway through it.
#[must_use]
pub fn rows(stored: &[Stored]) -> Vec<Row> {
    let mut groups: BTreeMap<(&str, &str), Vec<&Stored>> = BTreeMap::new();
    for run in stored {
        groups.entry((run.suite.as_str(), run.machine.as_str())).or_default().push(run);
    }

    let mut out = Vec::new();
    for runs in groups.values() {
        // A list rather than a map keyed by layer, because the order matters and it is the order
        // the layers were first stored in. Overwriting in place keeps that order while taking the
        // last run of a layer as the one that closed it.
        let mut closing: Vec<(&str, &Stored)> = Vec::new();
        for run in runs {
            let name = run.layer_name();
            match closing.iter_mut().find(|(seen, _)| *seen == name) {
                Some(slot) => slot.1 = run,
                None => closing.push((name, run)),
            }
        }
        let mut previous: Option<Stored> = None;
        for (_, after) in closing {
            out.push(Row { after: after.clone(), before: previous.replace(after.clone()) });
        }
    }
    out
}

/// Render the ledger for a reader.
#[must_use]
pub fn report(rows: &[Row]) -> String {
    let mut out = String::new();
    if rows.is_empty() {
        out.push_str("No runs are stored yet, so there is no ledger.\n");
        out.push_str("`rudb-bench run <suite> --store <layer>` writes one.\n");
        return out;
    }
    for row in rows {
        let _ = writeln!(out, "[{}] {} on {}", row.after.layer, row.after.suite, row.after.machine);
        let _ = writeln!(out, "closed by harness {} on {}", row.after.commit, row.after.recorded);
        match row.before.as_ref() {
            Some(before) => {
                let _ = writeln!(
                    out,
                    "against [{}], harness {} on {}",
                    before.layer_name(),
                    before.commit,
                    before.recorded
                );
            }
            None => {
                let _ = writeln!(
                    out,
                    "against nothing, this is where the series starts on this suite and machine"
                );
            }
        }
        out.push('\n');
        out.push_str(&grid(row));
        for note in row.moved() {
            let _ = writeln!(out, "  {note}");
        }
        if !row.moved().is_empty() {
            let _ = writeln!(
                out,
                "A ratio across that boundary is a ratio between two different comparisons."
            );
        }
        out.push('\n');
    }
    out
}

/// The table of standings for one row, with the change against the row before it under each one.
fn grid(row: &Row) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{:<20} {:>12} {:>12} {:>12} {:>12} {:>12} {:>12}",
        "engine", "hot", "hot cpu", "peak RSS", "read", "load", "on disk"
    );
    for now in &row.after.standings {
        let _ = writeln!(
            out,
            "{:<20} {:>12} {:>12} {:>12} {:>12} {:>12} {:>12}",
            now.engine,
            show(now.hot),
            now.cpu.map_or_else(|| "not read".to_owned(), show),
            now.peak.map_or_else(|| "not read".to_owned(), bytes),
            now.read.map_or_else(|| "not read".to_owned(), bytes),
            show(now.load),
            bytes(now.disk),
        );
        let was = row.before.as_ref().and_then(|b| b.standing(&now.engine));
        let Some(was) = was else {
            if row.before.is_some() {
                let _ = writeln!(out, "{:<20} {:>12}", "", "new");
            }
            continue;
        };
        let _ = writeln!(
            out,
            "{:<20} {:>12} {:>12} {:>12} {:>12} {:>12} {:>12}",
            "",
            times(now.hot.as_secs_f64(), was.hot.as_secs_f64()),
            pair(now.cpu, was.cpu, |d| d.as_secs_f64()),
            pair(now.peak, was.peak, |n| n as f64),
            pair(now.read, was.read, |n| n as f64),
            times(now.load.as_secs_f64(), was.load.as_secs_f64()),
            times(now.disk as f64, was.disk as f64),
        );
    }
    out
}

/// Now over then, as the ratio the report already uses everywhere else.
///
/// Under one is an improvement, which is the same direction as the `vs duckdb` row and is worth
/// keeping consistent even though a reader's first instinct is that bigger is better. A then of
/// zero has no ratio, and a measurement too small for the clock is not an infinite speedup.
fn times(now: f64, then: f64) -> String {
    if then <= 0.0 { "n/a".to_owned() } else { format!("{:.2}x", now / then) }
}

/// The same for a measure that either side may not have.
///
/// A measure missing on one side is not a change of zero, and printing it as one is how a ledger
/// ends up claiming a layer bought a hundred megabytes of peak on the afternoon `/usr/bin/time`
/// stopped answering.
fn pair<T: Copy>(now: Option<T>, then: Option<T>, as_f64: impl Fn(T) -> f64) -> String {
    match (now, then) {
        (Some(now), Some(then)) => times(as_f64(now), as_f64(then)),
        _ => "n/a".to_owned(),
    }
}

/// Where the committed runs live.
///
/// Beside the baselines and not in them, for the reason in the module doc. `RUDB_BENCH_RUNS`
/// overrides it, which is what the tests use so that they never touch the committed file.
#[must_use]
pub fn path(suite: &str) -> PathBuf {
    std::env::var_os("RUDB_BENCH_RUNS")
        .map_or_else(|| Path::new("runs").join(format!("{suite}.txt")), PathBuf::from)
}

/// This harness's commit, short, or a sentence saying why there is not one.
///
/// A run with no commit is still worth storing, since a number taken from a tarball is still a
/// number, and it is worth being obvious about, since a ledger row nobody can get back to is a row
/// that can only be trusted.
#[must_use]
pub fn commit_here() -> String {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map_or_else(|| "unknown".to_owned(), |text| text.trim().to_owned())
}

/// Read the stored runs, or nothing at all when there is no file yet.
///
/// # Errors
///
/// A file that exists and does not parse, with the line number.
pub fn read(at: &Path) -> Result<Vec<Stored>, String> {
    match std::fs::read_to_string(at) {
        Ok(text) => parse(&text).map_err(|e| format!("{}: {e}", at.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(format!("could not read {}: {e}", at.display())),
    }
}

/// Append a run to the file.
///
/// Appended and never replaced, unlike the regression records. A run that is superseded is still
/// part of the history, and a ledger that lets a rerun overwrite the run a claim was made from is a
/// ledger whose old rows quietly change.
///
/// # Errors
///
/// Anything the filesystem says, including the directory not existing, which it says by name.
pub fn write(at: &Path, run: &Stored) -> Result<(), String> {
    let mut kept = read(at)?;
    kept.push(run.clone());
    if let Some(parent) = at.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("could not make {}: {e}", parent.display()))?;
    }
    std::fs::write(at, render(&kept)).map_err(|e| format!("could not write {}: {e}", at.display()))
}

/// The header every runs file carries.
const HEADER: &str = "\
# Committed benchmark runs, read by `rudb-bench ledger`.
#
# One block per run, holding what each engine totalled over a whole suite. Written by
# `rudb-bench run <suite> --store <layer>` and rendered into the attribution ledger of section 2.8
# of spec/engine/02-baseline.md in tamnd/rudb, which is the document that says what each layer of
# the engine bought.
#
# This is not baselines/<suite>.txt and the two are not interchangeable. A baseline record is per
# query and holds three quartiles, because the regression gate asks whether one query got twice as
# slow. A run here is per engine and holds totals, because the ledger asks what a layer bought
# across a suite. Records get replaced when they are retaken and runs never do, because a history
# that overwrites itself is a table.
#
# Times are microseconds and sizes are bytes, both as integers, and a measure the machine would not
# give is a single dash rather than a zero. The commit is this harness's commit, since the harness
# is what ran, and rudb's own version is in its engine block like every other engine's.
#
# These are regression numbers from machines this project owns. Reporting rule seven still applies
# and nothing here is comparable to anybody's board.
";

/// Render stored runs as the file format.
#[must_use]
pub fn render(stored: &[Stored]) -> String {
    let mut out = String::from(HEADER);
    for run in stored {
        let _ = writeln!(out);
        let _ = writeln!(out, "[run]");
        let _ = writeln!(out, "suite     {}", run.suite);
        let _ = writeln!(out, "machine   {}", run.machine);
        let _ = writeln!(out, "layer     {}", run.layer);
        let _ = writeln!(out, "commit    {}", run.commit);
        let _ = writeln!(out, "recorded  {}", run.recorded);
        for standing in &run.standings {
            let _ = writeln!(out, "[engine]");
            let _ = writeln!(out, "name      {}", standing.engine);
            let _ = writeln!(out, "version   {}", standing.version);
            let _ = writeln!(out, "hot       {}", standing.hot.as_micros());
            let _ = writeln!(out, "cpu       {}", micros(standing.cpu));
            let _ = writeln!(out, "peak      {}", count(standing.peak));
            let _ = writeln!(out, "read      {}", count(standing.read));
            let _ = writeln!(out, "load      {}", standing.load.as_micros());
            let _ = writeln!(out, "disk      {}", standing.disk);
        }
    }
    out
}

/// A duration that may not have been read, as the file writes it.
fn micros(value: Option<Duration>) -> String {
    value.map_or_else(|| "-".to_owned(), |d| d.as_micros().to_string())
}

/// A size that may not have been read, as the file writes it.
fn count(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_owned(), |n| n.to_string())
}

/// Parse the file format.
///
/// Hand written, like everything else here, because a harness whose credibility is its whole point
/// does not get to grow a dependency tree for a file with eight keys in it.
///
/// # Errors
///
/// An unknown key, a number that is not a number, a key outside the block it belongs to, or a line
/// before the first `[run]`. Every one of them names the line.
pub fn parse(text: &str) -> Result<Vec<Stored>, String> {
    let mut runs: Vec<Stored> = Vec::new();
    for (at, line) in text.lines().enumerate() {
        let at = at + 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[run]" {
            runs.push(Stored {
                suite: String::new(),
                machine: String::new(),
                layer: String::new(),
                commit: String::new(),
                recorded: String::new(),
                standings: Vec::new(),
            });
            continue;
        }
        let Some(run) = runs.last_mut() else {
            return Err(format!("line {at}: {line}, before any [run] block"));
        };
        if line == "[engine]" {
            run.standings.push(Standing {
                engine: String::new(),
                version: String::new(),
                hot: Duration::ZERO,
                cpu: None,
                peak: None,
                read: None,
                load: Duration::ZERO,
                disk: 0,
            });
            continue;
        }
        let (key, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
        let rest = rest.trim();
        match key {
            "suite" => run.suite = rest.to_owned(),
            "machine" => run.machine = rest.to_owned(),
            "layer" => run.layer = rest.to_owned(),
            "commit" => run.commit = rest.to_owned(),
            "recorded" => run.recorded = rest.to_owned(),
            other => {
                let Some(standing) = run.standings.last_mut() else {
                    return Err(format!(
                        "line {at}: {other} belongs to an engine and there is no \
                                        [engine] block open"
                    ));
                };
                match other {
                    "name" => standing.engine = rest.to_owned(),
                    "version" => standing.version = rest.to_owned(),
                    "hot" => standing.hot = Duration::from_micros(number(rest, at, other)?),
                    "cpu" => standing.cpu = maybe(rest, at, other)?.map(Duration::from_micros),
                    "peak" => standing.peak = maybe(rest, at, other)?,
                    "read" => standing.read = maybe(rest, at, other)?,
                    "load" => standing.load = Duration::from_micros(number(rest, at, other)?),
                    "disk" => standing.disk = number(rest, at, other)?,
                    _ => return Err(format!("line {at}: no key called {other}")),
                }
            }
        }
    }
    Ok(runs)
}

/// A number, or the line it was not on.
fn number(text: &str, at: usize, key: &str) -> Result<u64, String> {
    text.parse().map_err(|e| format!("line {at}: {key} {text} is not a number, {e}"))
}

/// A number or the dash that means the machine would not say.
fn maybe(text: &str, at: usize, key: &str) -> Result<Option<u64>, String> {
    if text == "-" { Ok(None) } else { number(text, at, key).map(Some) }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Row, Standing, Stored, parse, render, report, rows};

    fn standing(engine: &str, version: &str, hot: u64) -> Standing {
        Standing {
            engine: engine.to_owned(),
            version: version.to_owned(),
            hot: Duration::from_millis(hot),
            cpu: Some(Duration::from_millis(hot * 2)),
            peak: Some(128 * 1024 * 1024),
            read: Some(0),
            load: Duration::from_secs(1),
            disk: 92 * 1024 * 1024,
        }
    }

    fn stored(layer: &str, machine: &str, engines: Vec<Standing>) -> Stored {
        Stored {
            suite: "smoke".to_owned(),
            machine: machine.to_owned(),
            layer: layer.to_owned(),
            commit: "abc1234".to_owned(),
            recorded: "2026-09-11".to_owned(),
            standings: engines,
        }
    }

    #[test]
    fn a_run_survives_being_written_and_read_back() {
        let run = stored("2a the baseline", "server3", vec![standing("duckdb", "v1.5.5", 767)]);
        let got = parse(&render(std::slice::from_ref(&run)))
            .expect("what this module writes it can read");
        assert_eq!(got, vec![run]);
    }

    #[test]
    fn a_measure_the_machine_would_not_give_comes_back_missing_and_not_as_zero() {
        // The tuned ClickHouse server row is the case in the table today: no peak and no CPU,
        // because the timer wraps a client and the work is in a server it did not start. A round
        // trip that turned those into zeroes would make that row the most efficient engine on the
        // board.
        let mut only = standing("clickhouse-server", "26.9.1.1138", 1710);
        only.cpu = None;
        only.peak = None;
        only.read = None;
        let run = stored("2a", "server3", vec![only]);
        let got = parse(&render(std::slice::from_ref(&run))).expect("missing measures round trip");
        assert_eq!(got, vec![run]);
    }

    #[test]
    fn the_first_row_of_a_series_has_nothing_before_it_and_says_so() {
        let built = rows(&[stored("2a", "server3", vec![standing("duckdb", "v1.5.5", 767)])]);
        assert_eq!(built.len(), 1);
        assert!(built[0].before.is_none());
        let text = report(&built);
        assert!(text.contains("this is where the series starts"), "{text}");
    }

    #[test]
    fn a_layer_stored_twice_is_closed_by_the_last_run_and_not_by_both() {
        // Storing again after fixing something in the same layer is the ordinary case, and two rows
        // for one layer would make the ledger claim a layer bought whatever the rerun's noise was.
        let built = rows(&[
            stored("2a", "server3", vec![standing("duckdb", "v1.5.5", 900)]),
            stored("2a", "server3", vec![standing("duckdb", "v1.5.5", 767)]),
        ]);
        assert_eq!(built.len(), 1);
        assert_eq!(built[0].after.standings[0].hot, Duration::from_millis(767));
    }

    #[test]
    fn two_machines_are_two_series_and_never_one_row() {
        // A row pairing server1's before with server3's after would be a row about the machines,
        // and it would be the most flattering row in the ledger every time, since server3 has twice
        // the cores.
        let built = rows(&[
            stored("2a", "server1", vec![standing("duckdb", "v1.5.5", 2000)]),
            stored("2b", "server3", vec![standing("duckdb", "v1.5.5", 767)]),
        ]);
        assert_eq!(built.len(), 2);
        assert!(built.iter().all(|r| r.before.is_none()), "{built:?}");
    }

    #[test]
    fn a_rival_that_released_between_two_rows_is_named_on_the_row() {
        // The failure this exists to catch: DuckDB ships a faster version during a layer, every
        // ratio moves, and the layer takes the credit.
        let row = Row {
            after: stored("2b", "server3", vec![standing("duckdb", "v1.6.0", 600)]),
            before: Some(stored("2a", "server3", vec![standing("duckdb", "v1.5.5", 767)])),
        };
        let moved = row.moved();
        assert_eq!(moved.len(), 1, "{moved:?}");
        assert!(moved[0].contains("v1.5.5") && moved[0].contains("v1.6.0"), "{moved:?}");
        assert!(report(&[row]).contains("two different comparisons"));
    }

    #[test]
    fn a_rival_that_stayed_where_it_was_is_not_mentioned() {
        let row = Row {
            after: stored("2b", "server3", vec![standing("duckdb", "v1.5.5", 600)]),
            before: Some(stored("2a", "server3", vec![standing("duckdb", "v1.5.5", 767)])),
        };
        assert!(row.moved().is_empty(), "{:?}", row.moved());
    }

    #[test]
    fn an_engine_that_starts_running_partway_through_is_not_a_speedup() {
        // rudb is the case. It abstains until 2e gives it a scan, so its first row has no before,
        // and a ratio against nothing would be the most impressive number in the ledger.
        let row = Row {
            after: stored(
                "2f",
                "server3",
                vec![standing("duckdb", "v1.5.5", 767), standing("rudb", "0.1.0", 400)],
            ),
            before: Some(stored("2e", "server3", vec![standing("duckdb", "v1.5.5", 767)])),
        };
        let text = report(std::slice::from_ref(&row));
        assert!(text.contains("new"), "{text}");
        assert!(row.moved().iter().any(|m| m.contains("rudb")), "{:?}", row.moved());
    }

    #[test]
    fn the_ledger_says_it_is_empty_rather_than_printing_nothing() {
        let text = report(&[]);
        assert!(text.contains("no ledger"), "{text}");
        assert!(text.contains("--store"), "{text}");
    }

    #[test]
    fn a_key_outside_its_block_names_the_line_rather_than_being_ignored() {
        let got = parse("[run]\nname      duckdb\n");
        assert!(got.is_err(), "{got:?}");
        let message = got.unwrap_err();
        assert!(message.contains("line 2"), "{message}");
        assert!(message.contains("[engine]"), "{message}");
    }

    #[test]
    fn a_layer_name_is_the_first_word_so_the_rest_can_be_improved_later() {
        let built = rows(&[
            stored("2a the baseline", "server3", vec![standing("duckdb", "v1.5.5", 900)]),
            stored(
                "2a a better sentence about it",
                "server3",
                vec![standing("duckdb", "v1.5.5", 767)],
            ),
        ]);
        assert_eq!(built.len(), 1, "{built:?}");
    }
}
