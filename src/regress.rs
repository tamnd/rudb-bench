//! The regression gate: today's run against a committed one.
//!
//! Section 13.8 of `spec/engine/13-measurement.md` asks for `smoke` on every commit, failing the
//! build when a query regresses rather than printing a number nobody reads. It also says exactly
//! how, and the how is the interesting half. A gate that fires on noise is a gate somebody disables
//! in the second week, so a run too noisy to be published cannot fail a build either. The rule is
//! that a regression fails when the distributions do not overlap and the median moved past the
//! threshold, and a noisy query is reported and not failed.
//!
//! The overlap test is why the committed record stores three numbers per query rather than one. A
//! median on its own can only be compared by a percentage, and a percentage on a shared runner is a
//! coin flip. Two quartiles from then and two from now say whether the two sets of samples are even
//! describing different things, and that question has an answer that does not depend on how busy
//! either machine was.
//!
//! Two thresholds, because there are two jobs. On a GitHub runner the gate catches a factor: the
//! hardware underneath varies between runs, so anything smaller than [`FACTOR`] is not evidence.
//! On `server3`, on a schedule, the gate catches a percentage, because that machine is the same
//! machine every week and [`DRIFT`] is a real signal there. Reporting rule seven applies to the
//! second one and not to the first: a drift check against a record from another machine is refused,
//! while a factor of two is a code change on any machine anybody could plausibly run this on.
//!
//! The record is a text file, committed, one block per suite and machine and engine. It is written
//! by `rudb-bench run <suite> --record` and read by `--check` and `--check-drift`, and the reason
//! it is text with one query per line is that the useful review of a baseline update is the diff.
//!
//! # What this does not check yet, measured rather than guessed
//!
//! On the `smoke` suite this gate declines almost every query, and the cause is not the machine.
//! Every query here is a fresh subprocess, by design, so that no engine gets a warm allocator and a
//! warm buffer pool that the others paid for. On a suite whose queries are tens of milliseconds
//! that means most of what gets timed is the process start, and a process start is not a steady
//! thing. Timed on an idle `server3` with no query attached, fifteen runs of `duckdb -c "select 1"`
//! spread from 30ms to 60ms, an interquartile range of 37% of their own median, and fifteen runs of
//! `clickhouse local --query "select 1"` spread from 180ms to 510ms, which is 48%. Both are several
//! times [`crate::report::NOISY`].
//!
//! So the noise rule fires first on nearly every `smoke` query and nearly nothing gets compared.
//! That is the correct behaviour of the rule and the wrong outcome for a gate, and the fix is not
//! to move the threshold. It is to stop timing `execve`, which is the in-process measurement path
//! at sub-milestone 2b. Until then [`Verdict::checked`] counts what was actually compared and the
//! report prints that count on every run, because a gate that declined everything and then said
//! nothing regressed is indistinguishable from a gate nobody wired up.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::measure::{Distribution, show};
use crate::report::{Comparison, NOISY, SuiteResult};

/// How far a median has to move on a shared runner before it is evidence of anything.
///
/// A factor of two. GitHub gives a different physical machine on every run and the spread between
/// the fast ones and the slow ones is comfortably tens of percent, so a ten percent gate there
/// would fire on Tuesdays. What survives that noise is the class of change this gate is actually
/// for: an accidental clone in a loop, a scan that stopped being vectorised, a lock somebody put
/// back. Those are factors and not percentages.
pub const FACTOR: f64 = 2.0;

/// How far a median has to move on a quiet machine that has run this every week.
///
/// Ten percent, matching [`NOISY`], and the two matching is not a coincidence. A movement smaller
/// than the run to run spread cannot be told apart from the spread, so the smallest threshold worth
/// having is the spread itself. Below that the gate would be reporting its own measurement error.
pub const DRIFT: f64 = 1.10;

/// Which of the two jobs a check is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Watch {
    /// Every commit, on whatever hardware the runner handed out.
    Ci,
    /// On a schedule, on the machine the record was taken on.
    Scheduled,
}

impl Watch {
    /// How far the median has to move.
    #[must_use]
    pub const fn threshold(self) -> f64 {
        match self {
            Self::Ci => FACTOR,
            Self::Scheduled => DRIFT,
        }
    }

    /// Whether the record has to have come from this machine.
    ///
    /// Rule seven says never compare across machines, and the CI check is the one exception the
    /// rules can carry: it does not produce a number, it answers yes or no to whether something got
    /// twice as slow, and no machine in this fleet is twice as fast as another at running `smoke`.
    /// The scheduled check does produce a number and gets no exception.
    #[must_use]
    pub const fn same_machine(self) -> bool {
        matches!(self, Self::Scheduled)
    }

    /// What to call it in a sentence.
    #[must_use]
    pub const fn what(self) -> &'static str {
        match self {
            Self::Ci => "the per commit check",
            Self::Scheduled => "the scheduled check",
        }
    }
}

/// What one query cost when the record was taken.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timing {
    /// The query, by the name the suite gives it.
    pub name: String,
    /// The lower quartile of the hot runs.
    pub p25: Duration,
    /// The median of the hot runs, which is the number the threshold applies to.
    pub median: Duration,
    /// The upper quartile of the hot runs.
    pub p75: Duration,
}

impl Timing {
    /// Take the three numbers out of a distribution.
    #[must_use]
    pub fn of(name: &str, hot: &Distribution) -> Self {
        Self { name: name.to_owned(), p25: hot.p25(), median: hot.median_of(), p75: hot.p75() }
    }
}

/// One engine on one machine on one suite, as it stood when somebody committed it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// The suite.
    pub suite: String,
    /// The machine, by the name [`crate::machine::name_here`] gives it.
    pub machine: String,
    /// The engine, as it names itself.
    pub engine: String,
    /// The engine's exact version, because a record against a version nobody wrote down is a record
    /// that silently becomes a comparison against a different program.
    pub version: String,
    /// The day it was taken, as a date somebody can read.
    pub recorded: String,
    /// Every query in the suite, in the order it ran.
    pub queries: Vec<Timing>,
}

impl Record {
    /// Take a record of a result that just ran.
    #[must_use]
    pub fn of(result: &SuiteResult, machine: &str, today: &str) -> Self {
        Self {
            suite: result.suite.name.to_owned(),
            machine: machine.to_owned(),
            engine: result.engine.clone(),
            version: result.version.clone(),
            recorded: today.to_owned(),
            queries: result.queries.iter().map(|q| Timing::of(&q.name, &q.runs.hot)).collect(),
        }
    }

    /// The recorded timing for a query, if it was in the suite when this was taken.
    #[must_use]
    pub fn query(&self, name: &str) -> Option<&Timing> {
        self.queries.iter().find(|q| q.name == name)
    }
}

/// What happened to one query since the record was taken.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Judgement {
    /// Within the threshold, or overlapping the record, or both.
    Steady {
        /// Today's median over the recorded one.
        ratio: f64,
    },
    /// Past the threshold, and the two sets of samples do not overlap.
    Regressed {
        /// Today's median over the recorded one.
        ratio: f64,
    },
    /// Past the threshold the other way, which is a reason to take the record again.
    Improved {
        /// Today's median over the recorded one.
        ratio: f64,
    },
    /// Too wide a spread today to say anything, which is never a failure.
    Noisy {
        /// Today's median over the recorded one, for the record.
        ratio: f64,
        /// How far today's samples swung, as a fraction of today's median.
        spread: f64,
    },
    /// Below the clock's resolution on one side or the other, so no ratio exists.
    TooFast,
    /// In the suite today and not in the record.
    Added,
    /// In the record and not in the suite today.
    Missing,
}

impl Judgement {
    /// Whether this one fails a build. Only a regression does.
    #[must_use]
    pub const fn fails(self) -> bool {
        matches!(self, Self::Regressed { .. })
    }
}

/// One query, and what happened to it.
#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    /// The query.
    pub query: String,
    /// What happened.
    pub judgement: Judgement,
}

/// One engine's run against one record.
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    /// The engine.
    pub engine: String,
    /// The machine the record came from.
    pub recorded_on: String,
    /// The machine this ran on.
    pub ran_on: String,
    /// Which job this was.
    pub watch: Watch,
    /// Every query, in the order the suite ran them.
    pub changes: Vec<Change>,
    /// Set when the check refused to run at all, which is a machine mismatch on a drift check.
    pub refused: Option<String>,
}

impl Verdict {
    /// Whether this fails the build.
    #[must_use]
    pub fn failed(&self) -> bool {
        self.changes.iter().any(|c| c.judgement.fails())
    }

    /// How many queries this actually compared, as opposed to declined to compare.
    ///
    /// A query that was too noisy, too fast, new or gone was not checked. Counting them matters
    /// because a gate that declined every query passes, and a pass that means nothing looks exactly
    /// like a pass that means something unless somebody counts.
    #[must_use]
    pub fn checked(&self) -> usize {
        self.changes
            .iter()
            .filter(|c| {
                matches!(
                    c.judgement,
                    Judgement::Steady { .. }
                        | Judgement::Regressed { .. }
                        | Judgement::Improved { .. }
                )
            })
            .count()
    }
}

/// Compare one engine's run against the record of it.
///
/// The order of the tests is the whole rule and it is worth stating in one place. Noise first, so
/// that a query which swung wide today is reported and can never fail, whatever its median did.
/// Then the overlap, so that two sets of samples describing the same thing are steady even when
/// their medians happen to sit far apart. Only a query that is quiet, moved past the threshold and
/// no longer overlaps its record is a regression.
#[must_use]
pub fn check(record: &Record, result: &SuiteResult, ran_on: &str, watch: Watch) -> Verdict {
    let mut verdict = Verdict {
        engine: result.engine.clone(),
        recorded_on: record.machine.clone(),
        ran_on: ran_on.to_owned(),
        watch,
        changes: Vec::new(),
        refused: None,
    };
    if watch.same_machine() && record.machine != ran_on {
        verdict.refused = Some(format!(
            "the record was taken on {} and this ran on {}, and rule seven says never compare \
             across machines",
            record.machine, ran_on
        ));
        return verdict;
    }

    for query in &result.queries {
        let judgement = record
            .query(&query.name)
            .map_or(Judgement::Added, |was| judge(was, &query.runs.hot, watch.threshold()));
        verdict.changes.push(Change { query: query.name.clone(), judgement });
    }
    for was in &record.queries {
        if !result.queries.iter().any(|q| q.name == was.name) {
            verdict.changes.push(Change { query: was.name.clone(), judgement: Judgement::Missing });
        }
    }
    verdict
}

/// One query, against one recorded timing.
fn judge(was: &Timing, now: &Distribution, threshold: f64) -> Judgement {
    let before = was.median.as_secs_f64();
    let after = now.median_of().as_secs_f64();
    if before <= 0.0 || after <= 0.0 {
        return Judgement::TooFast;
    }
    let ratio = after / before;

    // Noise first, and unconditionally. A query that swung by a third today has no median worth
    // comparing to anything, and a gate that fails on one is a gate whose next red is ignored.
    if let Some(spread) = now.relative_iqr().filter(|s| *s > NOISY) {
        return Judgement::Noisy { ratio, spread };
    }
    // Then overlap. Two sets of samples that share any of their middle half are two views of the
    // same thing however far apart their medians landed, which is the case a percentage alone gets
    // wrong on every run where the machine was busy for one of them.
    if ratio > threshold && now.p25() > was.p75 {
        return Judgement::Regressed { ratio };
    }
    if ratio < 1.0 / threshold && now.p75() < was.p25 {
        return Judgement::Improved { ratio };
    }
    Judgement::Steady { ratio }
}

/// Compare a whole comparison against whatever records exist for it.
///
/// An engine with no record is not in the output, because the alternative is a gate that turns red
/// the day somebody installs Polars. A record with no engine is also not in the output, for the
/// same reason from the other direction: the machine that has all five engines is the exception and
/// the harness already says which ones abstained.
#[must_use]
pub fn check_all(
    records: &[Record],
    compared: &Comparison,
    ran_on: &str,
    watch: Watch,
) -> Vec<Verdict> {
    compared
        .results
        .iter()
        .filter_map(|result| {
            let record = records
                .iter()
                .find(|r| r.suite == compared.suite.name && r.engine == result.engine)?;
            Some(check(record, result, ran_on, watch))
        })
        .collect()
}

/// The whole check, as the lines a build log prints.
///
/// A steady query gets a line the same as a regressed one. A gate whose green is silent is a gate
/// nobody can tell apart from a gate that did not run, and the second of those is the one that
/// happens by accident.
#[must_use]
pub fn report(verdicts: &[Verdict], watch: Watch, records: usize) -> String {
    let mut out = String::new();
    if verdicts.is_empty() {
        let _ = writeln!(
            out,
            "Nothing to check: {} record(s) committed, none of them for an engine that ran here.",
            records
        );
        let _ = writeln!(
            out,
            "Take one with `rudb-bench run <suite> --record` and commit it, and this becomes a gate."
        );
        return out;
    }

    let _ = writeln!(out, "{}, against a record taken earlier.", watch.what());
    let _ = writeln!(
        out,
        "A query fails when its median moved past {:.2}x and its samples no longer overlap the",
        watch.threshold()
    );
    let _ = writeln!(
        out,
        "recorded ones. A query that swung by more than {:.0}% today is reported and never failed.",
        NOISY * 100.0
    );
    let _ = writeln!(out);

    for verdict in verdicts {
        if let Some(why) = &verdict.refused {
            let _ = writeln!(out, "{}: not checked, {why}", verdict.engine);
            let _ = writeln!(out);
            continue;
        }
        let _ = writeln!(
            out,
            "{} on {}, against the record from {}",
            verdict.engine, verdict.ran_on, verdict.recorded_on
        );
        for change in &verdict.changes {
            let _ = writeln!(out, "  {:<10}  {}", change.query, describe(change.judgement));
        }
        let _ = writeln!(out);
    }

    // The count before the verdict, because a gate that declined every query and then said nothing
    // regressed is a gate that reads exactly like a working one. On the smoke suite today this is
    // the common case and it is not the machine's fault: every query is a fresh subprocess, so a
    // query of forty milliseconds is mostly a measurement of `execve`, and the bare startup of
    // duckdb and of clickhouse swing by 37% and 48% of their own medians on an idle server3 with no
    // query attached at all. Until the harness can time a query without timing a process start,
    // this gate will decline most of what it is shown, and it has to say so every time.
    let checked: usize = verdicts.iter().map(Verdict::checked).sum();
    let queries: usize = verdicts.iter().map(|v| v.changes.len()).sum();
    let _ = writeln!(out, "Compared {checked} of {queries} queries, and declined the rest.");
    if checked == 0 {
        let _ = writeln!(
            out,
            "Which means this run checked nothing. A gate that declines everything and passes is"
        );
        let _ = writeln!(
            out,
            "indistinguishable from a gate that is not wired up, so read that as a red rather than"
        );
        let _ = writeln!(out, "a green until the numbers underneath it hold still.");
    }
    let _ = writeln!(out);

    let failed: Vec<&Verdict> = verdicts.iter().filter(|v| v.failed()).collect();
    if failed.is_empty() {
        let _ = writeln!(out, "Nothing regressed past the threshold.");
    } else {
        for verdict in failed {
            let names: Vec<&str> = verdict
                .changes
                .iter()
                .filter(|c| c.judgement.fails())
                .map(|c| c.query.as_str())
                .collect();
            let _ = writeln!(out, "{} regressed on {}.", verdict.engine, names.join(" "));
        }
        let _ = writeln!(
            out,
            "Take the run again before believing it. If it holds, either the change is the cause"
        );
        let _ =
            writeln!(out, "or the record is stale, and only one of those is fixed by `--record`.");
    }
    out
}

/// One judgement, as a person reads it.
fn describe(judgement: Judgement) -> String {
    match judgement {
        Judgement::Steady { ratio } => format!("steady, {ratio:.2}x"),
        Judgement::Regressed { ratio } => format!("REGRESSED, {ratio:.2}x and no overlap"),
        Judgement::Improved { ratio } => {
            format!("faster, {ratio:.2}x and no overlap, so the record is worth taking again")
        }
        Judgement::Noisy { ratio, spread } => {
            format!("not checked, swung by {:.1}% today, and it was {ratio:.2}x", spread * 100.0)
        }
        Judgement::TooFast => {
            "not checked, too fast for the clock to have a ratio about".to_owned()
        }
        Judgement::Added => "new since the record, so there is nothing to compare it to".to_owned(),
        Judgement::Missing => "in the record and not in the suite today".to_owned(),
    }
}

/// Where the committed records live.
///
/// Under the working directory rather than under the binary, because the file is a source file that
/// gets committed and reviewed as a diff, and because both callers already run from the checkout:
/// CI runs `cargo run` from the root and `cargo xtask bench` sets the harness checkout as the
/// working directory. `RUDB_BENCH_BASELINE` overrides it for anybody whose layout is neither.
#[must_use]
pub fn path(suite: &str) -> PathBuf {
    std::env::var_os("RUDB_BENCH_BASELINE")
        .map_or_else(|| Path::new("baselines").join(format!("{suite}.txt")), PathBuf::from)
}

/// Read the records out of a file, or nothing at all when there is no file yet.
///
/// A missing file is `Ok(vec![])` and not an error. The first run on a new machine has nothing to
/// compare against and saying so is the correct outcome, whereas failing would mean a gate that
/// cannot be introduced without being introduced everywhere at once.
///
/// # Errors
///
/// A file that exists and does not parse, which is a broken commit rather than a missing one, with
/// the line number in the message.
pub fn read(at: &Path) -> Result<Vec<Record>, String> {
    match std::fs::read_to_string(at) {
        Ok(text) => parse(&text).map_err(|e| format!("{}: {e}", at.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(format!("could not read {}: {e}", at.display())),
    }
}

/// Write records to a file, replacing the ones for the same suite, machine and engine.
///
/// # Errors
///
/// Anything the filesystem says, including the directory not existing, which it says by name.
pub fn write(at: &Path, taken: &[Record]) -> Result<(), String> {
    let mut kept = read(at)?;
    kept.retain(|old| {
        !taken.iter().any(|new| {
            new.suite == old.suite && new.machine == old.machine && new.engine == old.engine
        })
    });
    kept.extend(taken.iter().cloned());
    kept.sort_by(|a, b| (&a.suite, &a.machine, &a.engine).cmp(&(&b.suite, &b.machine, &b.engine)));
    if let Some(parent) = at.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("could not make {}: {e}", parent.display()))?;
    }
    std::fs::write(at, render(&kept)).map_err(|e| format!("could not write {}: {e}", at.display()))
}

/// The header every record file carries, so that somebody who opens one knows what it is for.
const HEADER: &str = "\
# Committed benchmark records, read by the regression gate.
#
# One block per suite, machine and engine. Written by `rudb-bench run <suite> --record` and read by
# `rudb-bench run <suite> --check`, which fails when a query's median moved past the threshold and
# its samples no longer overlap the ones here. A query that swung wide on the day is reported and
# never failed, per section 13.8 of spec/engine/13-measurement.md in tamnd/rudb.
#
# Times are microseconds, as integers, in the order lower quartile, median, upper quartile. Integers
# because a number a formatter rounded is a number that drifts by a rounding step every time this
# file is rewritten, and three of them because a median alone can only be compared by a percentage
# and a percentage on a shared runner is a coin flip.
#
# These are not published numbers and nothing here is comparable to anybody's board. They exist to
# be compared against themselves on the same machine. Reporting rule seven still applies.
";

/// Render records as the file format.
#[must_use]
pub fn render(records: &[Record]) -> String {
    let mut out = String::from(HEADER);
    for record in records {
        let _ = writeln!(out);
        let _ = writeln!(out, "[record]");
        let _ = writeln!(out, "suite     {}", record.suite);
        let _ = writeln!(out, "machine   {}", record.machine);
        let _ = writeln!(out, "engine    {}", record.engine);
        let _ = writeln!(out, "version   {}", record.version);
        let _ = writeln!(out, "recorded  {}", record.recorded);
        for query in &record.queries {
            let _ = writeln!(
                out,
                "query     {:<10} {:>10} {:>10} {:>10}",
                query.name,
                query.p25.as_micros(),
                query.median.as_micros(),
                query.p75.as_micros()
            );
        }
    }
    out
}

/// Parse the file format.
///
/// Hand written, like everything else in this repository, because a harness whose credibility is
/// its whole point does not get to grow a dependency tree for a file with five keys in it.
///
/// # Errors
///
/// An unknown key, a query line that is not four fields, a number that is not a number, or a line
/// before the first `[record]`. Every one of them names the line.
pub fn parse(text: &str) -> Result<Vec<Record>, String> {
    let mut records: Vec<Record> = Vec::new();
    for (at, line) in text.lines().enumerate() {
        let at = at + 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[record]" {
            records.push(Record {
                suite: String::new(),
                machine: String::new(),
                engine: String::new(),
                version: String::new(),
                recorded: String::new(),
                queries: Vec::new(),
            });
            continue;
        }
        let Some(record) = records.last_mut() else {
            return Err(format!("line {at}: {line}, before any [record] block"));
        };
        let (key, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
        let rest = rest.trim();
        match key {
            "suite" => record.suite = rest.to_owned(),
            "machine" => record.machine = rest.to_owned(),
            "engine" => record.engine = rest.to_owned(),
            "version" => record.version = rest.to_owned(),
            "recorded" => record.recorded = rest.to_owned(),
            "query" => record.queries.push(timing(rest, at)?),
            other => return Err(format!("line {at}: no key called {other}")),
        }
    }
    Ok(records)
}

/// One query line: a name and three microsecond counts.
fn timing(rest: &str, at: usize) -> Result<Timing, String> {
    let fields: Vec<&str> = rest.split_whitespace().collect();
    let [name, p25, median, p75] = fields.as_slice() else {
        return Err(format!(
            "line {at}: a query is a name and three microsecond counts, got {} fields",
            fields.len()
        ));
    };
    Ok(Timing {
        name: (*name).to_owned(),
        p25: micros(p25, at)?,
        median: micros(median, at)?,
        p75: micros(p75, at)?,
    })
}

fn micros(field: &str, at: usize) -> Result<Duration, String> {
    field
        .parse::<u64>()
        .map(Duration::from_micros)
        .map_err(|e| format!("line {at}: {field} is not a microsecond count, {e}"))
}

/// Today, as `YYYY-MM-DD` in UTC.
///
/// Computed here rather than taken from a date library, because this repository has no dependencies
/// and one date is not a reason to start. UTC rather than local, because the record is committed and
/// two people in two timezones writing two different days for the same run is a diff that confuses
/// whoever reads it later.
///
/// The conversion is Howard Hinnant's `civil_from_days`, which is the standard one and is correct
/// for every date this project will ever write. A clock set before 1970 gives `unknown`, which is a
/// fact about that machine worth carrying into the file.
#[must_use]
pub fn today() -> String {
    let Ok(since) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) else {
        return "unknown".to_owned();
    };
    let (year, month, day) = civil(i64::try_from(since.as_secs() / 86_400).unwrap_or(0));
    format!("{year:04}-{month:02}-{day:02}")
}

/// Days since the Unix epoch, as a year, a month and a day.
fn civil(days: i64) -> (i64, u64, u64) {
    // Shifts the era so that the leap day is the last day of the year, which is what removes every
    // special case from the arithmetic below.
    let shifted = days + 719_468;
    let era = if shifted >= 0 { shifted } else { shifted - 146_096 } / 146_097;
    let day_of_era = u64::try_from(shifted - era * 146_097).unwrap_or(0);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_position = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_position + 2) / 5 + 1;
    let month = if month_position < 10 { month_position + 3 } else { month_position - 9 };
    let year = i64::try_from(year_of_era).unwrap_or(0) + era * 400;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// One line summarising a record, for the output of `--record`.
#[must_use]
pub fn describe_record(record: &Record) -> String {
    let total: Duration = record.queries.iter().map(|q| q.median).sum();
    format!(
        "{} on {}, {} queries, {} of medians, version {}",
        record.engine,
        record.machine,
        record.queries.len(),
        show(total),
        record.version
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        Judgement, Record, Timing, Watch, check, civil, judge, parse, path, render, report, today,
    };
    use crate::engine::Loaded;
    use crate::measure::{Distribution, Runs};
    use crate::memory::{Cost, Peak};
    use crate::report::{QueryResult, SuiteResult};

    fn ms(values: &[u64]) -> Distribution {
        Distribution::median(values.iter().copied().map(Duration::from_millis).collect())
    }

    /// A result of one query at a known speed, which is everything these tests need out of a run.
    fn ran(engine: &str, millis: &[u64]) -> SuiteResult {
        let cost = || Cost {
            peak: Peak::Bytes(1024),
            cpu: Some(Duration::from_millis(30)),
            read: Some(0),
        };
        SuiteResult {
            suite: crate::suite::find("smoke").expect("smoke is a suite"),
            engine: engine.to_owned(),
            version: "v1.5.5".to_owned(),
            loaded: Loaded {
                took: Duration::from_secs(1),
                on_disk: 1024 * 1024,
                on_disk_is: "its own database file".to_owned(),
                converted: true,
                cpu: Some(Duration::from_secs(3)),
            },
            queries: vec![QueryResult {
                reported: None,
                name: "q1".to_owned(),
                shape: "count".to_owned(),
                runs: Runs { cold: Duration::from_millis(40), hot: ms(millis) },
                cold: cost(),
                hot: cost(),
                answer: "10000000".to_owned(),
                internal: None,
            }],
            sample: None,
            rows: None,
            keeps_state: false,
            missing: Vec::new(),
            load: None,
            cold_forced: false,
        }
    }

    fn was(name: &str, p25: u64, median: u64, p75: u64) -> Timing {
        Timing {
            name: name.to_owned(),
            p25: Duration::from_millis(p25),
            median: Duration::from_millis(median),
            p75: Duration::from_millis(p75),
        }
    }

    #[test]
    fn a_query_that_doubled_and_left_the_old_range_behind_is_a_regression() {
        let judgement = judge(&was("q1", 99, 100, 101), &ms(&[299, 300, 300, 300, 301]), 2.0);
        assert!(matches!(judgement, Judgement::Regressed { .. }), "{judgement:?}");
        assert!(judgement.fails());
    }

    #[test]
    fn a_query_that_doubled_while_swinging_wildly_is_reported_and_not_failed() {
        // The whole reason the gate is written this way. A shared runner produces this run several
        // times a week and a gate that goes red on it is a gate somebody deletes.
        let judgement = judge(&was("q1", 99, 100, 101), &ms(&[100, 200, 300, 400, 500]), 2.0);
        assert!(matches!(judgement, Judgement::Noisy { .. }), "{judgement:?}");
        assert!(!judgement.fails());
    }

    #[test]
    fn a_median_past_the_threshold_whose_samples_still_overlap_is_not_a_regression() {
        // Recorded quiet with a wide upper quartile, today quiet and slower, but today's lower
        // quartile is inside yesterday's range, so the two are describing the same thing.
        let judgement = judge(&was("q1", 50, 100, 400), &ms(&[290, 300, 300, 300, 310]), 2.0);
        assert!(matches!(judgement, Judgement::Steady { .. }), "{judgement:?}");
    }

    #[test]
    fn the_shared_runner_threshold_is_a_factor_and_the_quiet_one_is_a_percentage() {
        let now = ms(&[119, 120, 120, 120, 121]);
        // Twenty percent slower. Nothing on a GitHub runner, a real signal on server3.
        assert!(matches!(
            judge(&was("q1", 99, 100, 101), &now, Watch::Ci.threshold()),
            Judgement::Steady { .. }
        ));
        assert!(matches!(
            judge(&was("q1", 99, 100, 101), &now, Watch::Scheduled.threshold()),
            Judgement::Regressed { .. }
        ));
    }

    #[test]
    fn getting_faster_says_the_record_is_worth_taking_again_and_does_not_fail() {
        let judgement = judge(&was("q1", 299, 300, 301), &ms(&[99, 100, 100, 100, 101]), 2.0);
        assert!(matches!(judgement, Judgement::Improved { .. }), "{judgement:?}");
        assert!(!judgement.fails());
    }

    #[test]
    fn a_query_below_the_clock_is_not_a_gate() {
        let judgement = judge(&was("q1", 0, 0, 0), &ms(&[1, 1, 1, 1, 1]), 2.0);
        assert_eq!(judgement, Judgement::TooFast);
    }

    #[test]
    fn a_drift_check_against_another_machine_is_refused_rather_than_answered() {
        // Rule seven. The scheduled check produces a percentage, and a percentage between two
        // machines is not a number about the code.
        let record = Record {
            suite: "smoke".to_owned(),
            machine: "server3".to_owned(),
            engine: "duckdb".to_owned(),
            version: "v1.5.5".to_owned(),
            recorded: "2026-09-11".to_owned(),
            queries: vec![was("q1", 99, 100, 101)],
        };
        let result = ran("duckdb", &[900, 900, 900, 900, 900]);
        let verdict = check(&record, &result, "server1", Watch::Scheduled);
        assert!(verdict.refused.is_some(), "{verdict:?}");
        assert!(!verdict.failed(), "a refusal is not a failure");
        // The same run on the per commit check is answered, because a factor of nine is not a
        // difference between two machines that both run this suite in under a second.
        let verdict = check(&record, &result, "server1", Watch::Ci);
        assert!(verdict.refused.is_none());
        assert!(verdict.failed());
    }

    #[test]
    fn a_query_the_record_never_had_is_new_and_one_it_lost_is_missing() {
        let record = Record {
            suite: "smoke".to_owned(),
            machine: "here".to_owned(),
            engine: "duckdb".to_owned(),
            version: "v1".to_owned(),
            recorded: "2026-09-11".to_owned(),
            queries: vec![was("q9", 99, 100, 101)],
        };
        let result = ran("duckdb", &[100, 100, 100, 100, 100]);
        let verdict = check(&record, &result, "here", Watch::Ci);
        let judged: Vec<Judgement> = verdict.changes.iter().map(|c| c.judgement).collect();
        assert!(judged.contains(&Judgement::Added), "{judged:?}");
        assert!(judged.contains(&Judgement::Missing), "{judged:?}");
        assert!(!verdict.failed(), "neither of those is a regression");
    }

    #[test]
    fn a_file_survives_being_written_and_read_back() {
        let record = Record {
            suite: "smoke".to_owned(),
            machine: "server3".to_owned(),
            engine: "duckdb".to_owned(),
            version: "v1.5.5 (Variegata) d8cdaa33fd".to_owned(),
            recorded: "2026-09-11".to_owned(),
            queries: vec![was("q1", 44, 45, 46), was("q2", 145, 146, 148)],
        };
        let back =
            parse(&render(std::slice::from_ref(&record))).expect("what this wrote, it can read");
        assert_eq!(back, vec![record]);
    }

    #[test]
    fn a_version_with_spaces_in_it_survives_the_round_trip() {
        // DuckDB names itself "v1.5.5 (Variegata) d8cdaa33fd", so a parser that took the second
        // field and stopped would record a different program from the one that ran.
        let back = parse("[record]\nversion   v1.5.5 (Variegata) d8cdaa33fd\n").unwrap();
        assert_eq!(back[0].version, "v1.5.5 (Variegata) d8cdaa33fd");
    }

    #[test]
    fn a_broken_line_names_the_line_it_is_on() {
        let e = parse("[record]\nsuite smoke\nnonsense here\n").unwrap_err();
        assert!(e.contains("line 3"), "{e}");
        let e = parse("[record]\nquery q1 1 2\n").unwrap_err();
        assert!(e.contains("line 2") && e.contains("three microsecond counts"), "{e}");
        let e = parse("suite smoke\n").unwrap_err();
        assert!(e.contains("before any [record]"), "{e}");
    }

    #[test]
    fn a_comment_and_a_blank_line_are_not_data() {
        assert!(parse("# nothing here\n\n").unwrap().is_empty());
    }

    #[test]
    fn nothing_to_check_says_how_to_make_it_a_gate_instead_of_passing_quietly() {
        let text = report(&[], Watch::Ci, 0);
        assert!(text.contains("--record"), "{text}");
        assert!(text.contains("Nothing to check"), "{text}");
    }

    #[test]
    fn a_gate_that_declined_every_query_says_so_rather_than_reporting_green() {
        // This is the state the smoke suite is actually in, because every query is a fresh
        // subprocess and a forty millisecond query is mostly a timing of the process start. A
        // report that printed "nothing regressed" here would be a report that lied by omission.
        let record = Record {
            suite: "smoke".to_owned(),
            machine: "here".to_owned(),
            engine: "duckdb".to_owned(),
            version: "v1".to_owned(),
            recorded: "2026-09-11".to_owned(),
            queries: vec![was("q1", 99, 100, 101)],
        };
        let noisy = ran("duckdb", &[100, 200, 300, 400, 500]);
        let verdict = check(&record, &noisy, "here", Watch::Ci);
        assert_eq!(verdict.checked(), 0);
        let text = report(&[verdict], Watch::Ci, 1);
        assert!(text.contains("Compared 0 of 1 queries"), "{text}");
        assert!(text.contains("checked nothing"), "{text}");
        assert!(text.contains("read that as a red"), "{text}");
    }

    #[test]
    fn a_gate_that_did_compare_something_counts_it_and_does_not_cry_wolf() {
        let record = Record {
            suite: "smoke".to_owned(),
            machine: "here".to_owned(),
            engine: "duckdb".to_owned(),
            version: "v1".to_owned(),
            recorded: "2026-09-11".to_owned(),
            queries: vec![was("q1", 99, 100, 101)],
        };
        let steady = ran("duckdb", &[100, 100, 100, 100, 100]);
        let verdict = check(&record, &steady, "here", Watch::Ci);
        assert_eq!(verdict.checked(), 1);
        let text = report(&[verdict], Watch::Ci, 1);
        assert!(text.contains("Compared 1 of 1 queries"), "{text}");
        assert!(!text.contains("checked nothing"), "{text}");
        assert!(text.contains("Nothing regressed"), "{text}");
    }

    #[test]
    fn the_date_in_a_record_is_the_date_and_not_an_off_by_one() {
        assert_eq!(civil(0), (1970, 1, 1));
        assert_eq!(civil(-1), (1969, 12, 31));
        // A leap day, and the day after it, which is where a wrong month boundary shows up first.
        assert_eq!(civil(20_513), (2026, 3, 1));
        assert_eq!(civil(19_782), (2024, 2, 29));
        assert_eq!(civil(19_783), (2024, 3, 1));
        // 2100 is divisible by four and is not a leap year, which the naive rule gets wrong.
        assert_eq!(civil(47_540), (2100, 2, 28));
        assert_eq!(civil(47_541), (2100, 3, 1));
        let now = today();
        assert_eq!(now.len(), 10, "{now}");
        assert!(now.starts_with("20"), "{now}");
    }

    #[test]
    fn the_record_file_is_a_path_under_the_checkout_and_not_under_the_binary() {
        // A file meant to be reviewed as a diff has to be somewhere `git` can see it.
        assert!(path("smoke").ends_with("baselines/smoke.txt"));
    }
}
