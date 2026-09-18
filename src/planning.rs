//! What each query spent planning, against a budget somebody committed on purpose.
//!
//! Every other gate in this harness watches the part of a query that moves rows. This one watches
//! the part before any row moves, because that is the part nobody profiles. An optimizer is only
//! ever added to: each pass arrives because it paid for itself on the query somebody wrote it for,
//! and no pass ever arrives with a measurement of what it costs on the queries it does nothing for.
//! The pass that saves four hundred milliseconds on a scan of ten million rows costs its two
//! hundred microseconds on a point lookup too, and twenty passes later the point lookup is planning
//! for longer than it runs.
//!
//! Section E1 of `spec/engine/13-measurement.md` in tamnd/rudb states the case it wants caught in
//! one sentence: a query that plans for four hundred milliseconds and runs for two hundred is a
//! query the optimizer made slower. Nobody discovers that by accident, because the query still
//! returns the right answer and the profile everybody looks at starts at the first operator.
//!
//! # Why the budget is a share and not a duration
//!
//! A committed budget of nine hundred microseconds is a fact about the machine it was taken on. Run
//! the same gate on a slower box and it fails for a reason that has nothing to do with the engine,
//! which is reporting rule seven, and this project has already had one gate go red for a week
//! because a record taken on one machine was checked on another.
//!
//! A share survives the trip. Planning and execution are two spans of the same run of the same
//! query, read off the same clock, seconds apart, so the ratio between them is a property of the
//! engine in a way that neither number is on its own. It is not perfectly portable either, because
//! planning does not get faster with more cores and execution does, but it errs in the safe
//! direction: a slower machine runs the query for longer, which makes planning a smaller share, so
//! a budget carried to a slower machine gets looser rather than crying wolf.
//!
//! # Why the recorded share is written down next to the ceiling
//!
//! The ceiling is what fails a run and the recorded share is what makes the ceiling reviewable. A
//! file of ceilings alone cannot be read: nobody can tell whether four percent is a tight budget
//! that was just met or a loose one with three times the headroom it needs, so nobody can tell
//! whether a diff that raises one is somebody accepting a regression. With both numbers in the
//! line, a ceiling that has drifted away from the thing it was derived from is visible in the file
//! rather than in a run somebody has to go and do.
//!
//! # Integers
//!
//! Hundredths of a percent, as integers, for the same reason the regression records are integers:
//! a number a formatter rounded drifts by a rounding step every time the file is rewritten, and a
//! file that changes when nothing changed is a file whose diffs nobody reads.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use crate::report::{Comparison, SuiteResult};

/// How much room a recorded share is given before it becomes the ceiling.
///
/// A quarter again. Planning is a handful of microseconds on a short query and the clock around it
/// is the same clock every other span here uses, so the run-to-run swing is a real fraction of the
/// number rather than a rounding step. A tighter budget than this would fail on the weather.
pub const HEADROOM: f64 = 1.25;

/// The least room a recorded share is given, in share rather than in proportion.
///
/// Two points of percentage. Without it, a query that planned for half a percent gets a ceiling of
/// sixty two hundredths of a percent, which is twelve hundredths of a percent of headroom on a
/// number that moves by more than that between two runs on an idle machine. A proportional margin
/// on a small number is not a margin.
pub const FLOOR: f64 = 0.02;

/// What one query is allowed to spend planning, and what it spent when that was decided.
#[derive(Debug, Clone, PartialEq)]
pub struct Allowance {
    /// The query, by the name the suite gives it.
    pub name: String,
    /// The share it spent planning on the day the budget was written, in hundredths of a percent.
    pub was: u32,
    /// The most it may spend planning, in hundredths of a percent. This is the number that fails.
    pub ceiling: u32,
}

/// One engine's budget for one suite, and where it came from.
#[derive(Debug, Clone, PartialEq)]
pub struct Budget {
    /// The suite these are for.
    pub suite: String,
    /// The machine the shares were taken on.
    ///
    /// The ceiling is asserted anywhere, which is the point of it being a share. This is here
    /// because a share is still a measurement and a measurement with no machine on it is the thing
    /// rule one exists to stop, and because a budget that turns out to be wrong is argued about by
    /// somebody who needs to know where it came from.
    pub machine: String,
    /// The engine, as it names itself.
    pub engine: String,
    /// Its exact version, per rule one.
    pub version: String,
    /// The day it was written.
    pub recorded: String,
    /// One line per query, in the order the suite runs them.
    pub queries: Vec<Allowance>,
}

impl Budget {
    /// This suite's queries as they ran, turned into a budget with headroom on it.
    ///
    /// Only the queries the engine reported a breakdown for. Every other engine here is a black box
    /// with a clock on the outside and has no planning number to have a budget about, so a suite run
    /// against duckdb produces no budget rather than a budget of dashes.
    #[must_use]
    pub fn of(result: &SuiteResult, machine: &str, today: &str) -> Self {
        let queries = result
            .queries
            .iter()
            .filter_map(|query| {
                let (_, _, share) = query.planned()?;
                let was = hundredths(share);
                Some(Allowance { name: query.name.clone(), was, ceiling: allowed(share) })
            })
            .collect();
        Self {
            suite: result.suite.name.to_owned(),
            machine: machine.to_owned(),
            engine: result.engine.clone(),
            version: result.version.clone(),
            recorded: today.to_owned(),
            queries,
        }
    }

    /// What this budget allows one query, if it mentions it.
    #[must_use]
    pub fn allowance(&self, name: &str) -> Option<&Allowance> {
        self.queries.iter().find(|q| q.name == name)
    }
}

/// A share, in hundredths of a percent, saturating at everything.
fn hundredths(share: f64) -> u32 {
    let scaled = (share * 10_000.0).round();
    if scaled.is_finite() && scaled > 0.0 { scaled.min(10_000.0) as u32 } else { 0 }
}

/// The ceiling a measured share earns, in hundredths of a percent.
///
/// The larger of a proportional margin and a flat one, so that the budget is neither absurdly tight
/// on a query that planned for almost nothing nor absurdly loose on one that planned for most of
/// itself.
fn allowed(share: f64) -> u32 {
    hundredths((share * HEADROOM).max(share + FLOOR))
}

/// What one query did against what it was allowed.
#[derive(Debug, Clone, PartialEq)]
pub struct Spent {
    /// The query, by the name the suite gives it.
    pub name: String,
    /// The share it spent planning now, in hundredths of a percent.
    pub share: u32,
    /// What it was allowed, or `None` for a query the budget says nothing about.
    pub ceiling: Option<u32>,
    /// What planning and execution actually were, for the report to print next to the shares.
    ///
    /// Never asserted on and never compared to another machine's. A share with no durations beside
    /// it tells somebody a budget was missed and nothing about by how much it mattered, and eight
    /// percent of a two millisecond query is not the same news as eight percent of a two second one.
    pub planning_micros: u128,
    /// The execute span, in microseconds, for the same reason.
    pub execute_micros: u128,
}

impl Spent {
    /// Whether this is a query that has to fail the run.
    #[must_use]
    pub fn over(&self) -> bool {
        self.ceiling.is_some_and(|ceiling| self.share > ceiling)
    }
}

/// One engine's run against one engine's budget.
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    /// The engine that ran.
    pub engine: String,
    /// Every query that reported a breakdown, in the order it ran.
    pub queries: Vec<Spent>,
    /// Queries the run produced that the budget has never heard of.
    ///
    /// Reported and never failed. A query added to a suite arrives with no budget by definition, and
    /// a gate that fails on it is a gate that has to be re-recorded in the same commit that adds a
    /// query, which is how a re-record becomes a reflex instead of a decision.
    pub unbudgeted: Vec<String>,
}

impl Verdict {
    /// Whether anything here has to fail the run.
    #[must_use]
    pub fn failed(&self) -> bool {
        self.queries.iter().any(Spent::over)
    }
}

/// Check every engine that reported a breakdown against the budget written for it.
///
/// An engine with no budget is skipped rather than failed, the same as a machine with no regression
/// record. A gate nobody can introduce without introducing it everywhere at once does not get
/// introduced.
#[must_use]
pub fn check_all(budgets: &[Budget], compared: &Comparison) -> Vec<Verdict> {
    compared
        .results
        .iter()
        .filter_map(|result| {
            let budget = budgets
                .iter()
                .find(|b| b.suite == result.suite.name && b.engine == result.engine)?;
            Some(check(budget, result))
        })
        .collect()
}

/// Check one engine's run against one budget.
#[must_use]
pub fn check(budget: &Budget, result: &SuiteResult) -> Verdict {
    let mut queries = Vec::new();
    let mut unbudgeted = Vec::new();
    for query in &result.queries {
        let Some((planning, execute, share)) = query.planned() else { continue };
        let ceiling = budget.allowance(&query.name).map(|a| a.ceiling);
        if ceiling.is_none() {
            unbudgeted.push(query.name.clone());
        }
        queries.push(Spent {
            name: query.name.clone(),
            share: hundredths(share),
            ceiling,
            planning_micros: planning.as_micros(),
            execute_micros: execute.as_micros(),
        });
    }
    Verdict { engine: result.engine.clone(), queries, unbudgeted }
}

/// A share in hundredths of a percent, written the way a person reads one.
#[must_use]
pub fn percent(hundredths: u32) -> String {
    format!("{:.2}%", f64::from(hundredths) / 100.0)
}

/// What the check has to say, whether or not anything failed.
///
/// Every query, not only the ones over budget. A gate that prints nothing when it passes is a gate
/// nobody can tell is still running, and the shares next to each other are the thing worth reading
/// even on a run where none of them moved: a column that is one percent everywhere except for the
/// query that is nine is a work list.
#[must_use]
pub fn report(verdicts: &[Verdict], budgets: usize) -> String {
    let mut out = String::new();
    if verdicts.is_empty() {
        let _ = writeln!(
            out,
            "  no planning budget for anything that ran here, out of {budgets} committed"
        );
        let _ = writeln!(out, "  write one with `rudb-bench run <suite> --record`");
        return out;
    }
    for verdict in verdicts {
        let _ = writeln!(out, "  {}", verdict.engine);
        for query in &verdict.queries {
            let against = match query.ceiling {
                Some(ceiling) => format!("of {} allowed", percent(ceiling)),
                None => "not budgeted".to_owned(),
            };
            let verdict = if query.over() { "  OVER" } else { "" };
            let _ = writeln!(
                out,
                "    {:<8} {:>8} {against:<18} {}us planning, {}us running{verdict}",
                query.name,
                percent(query.share),
                query.planning_micros,
                query.execute_micros
            );
        }
        if !verdict.unbudgeted.is_empty() {
            let _ = writeln!(
                out,
                "  {} queries have no budget yet and were not failed for it: {}",
                verdict.unbudgeted.len(),
                verdict.unbudgeted.join(", ")
            );
        }
    }
    if verdicts.iter().any(Verdict::failed) {
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "A query over its budget is a query whose planning grew faster than its execution."
        );
        let _ = writeln!(
            out,
            "That is an optimizer pass that costs more than it saves here. Find out which one"
        );
        let _ = writeln!(
            out,
            "with `SET disabled_optimizers` before raising the number in the committed file."
        );
    }
    out
}

/// Where the committed budgets live.
#[must_use]
pub fn path(suite: &str) -> PathBuf {
    crate::plans::root().join("baselines").join(format!("planning-{suite}.txt"))
}

/// Read the committed budgets.
///
/// `Ok(vec![])` when there is no file, which is what a suite nobody has recorded looks like and is
/// not an error.
///
/// # Errors
///
/// When the file is there and is not a budget file.
pub fn read(at: &Path) -> Result<Vec<Budget>, String> {
    let Ok(text) = fs::read_to_string(at) else { return Ok(Vec::new()) };
    parse(&text).map_err(|why| format!("{}: {why}", at.display()))
}

/// Write the committed budgets, replacing whatever was there for the engines that ran.
///
/// # Errors
///
/// When the file cannot be written.
pub fn write(at: &Path, taken: &[Budget]) -> Result<(), String> {
    if let Some(parent) = at.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let mut kept: Vec<Budget> = read(at)?
        .into_iter()
        .filter(|old| !taken.iter().any(|new| new.suite == old.suite && new.engine == old.engine))
        .collect();
    kept.extend(taken.iter().cloned());
    kept.sort_by(|a, b| a.suite.cmp(&b.suite).then_with(|| a.engine.cmp(&b.engine)));
    fs::write(at, render(&kept)).map_err(|e| format!("{}: {e}", at.display()))
}

/// The comment block at the top of the file, which is the only place a reader is told what it is.
const HEADER: &str = "\
# Committed planning budgets, read by the planning gate.
#
# One block per suite and engine. Written by `rudb-bench run <suite> --record` and read by
# `rudb-bench run <suite> --check`, which fails a query that spent a larger share of itself planning
# than the ceiling here allows.
#
# Shares, in hundredths of a percent, as integers. The first number is what the query spent planning
# on the day this was written and the second is the ceiling derived from it. A share rather than a
# duration because a duration is a fact about a machine: planning and execution are two spans of the
# same run of the same query on the same clock, so the ratio between them travels and neither number
# does. Reporting rule seven still applies to the microseconds the gate prints beside these, which
# is why they are printed and not stored.
#
# The recorded share is kept next to the ceiling so that a diff raising a ceiling can be read as
# what it is. A file of ceilings alone cannot be reviewed, because nobody can tell a tight budget
# from one with three times the headroom it needs.
#
# Planning is everything before the first row moved: the parse, the bind, the optimizer passes and
# building the operator tree. Taken as the whole statement less the execute span rather than by
# adding the phases up, so that work moved out of the optimizer into the builder does not meet the
# budget by changing which side of a line it is on.
#
# The share is the middle one of the runs rather than the first. A query the engine answers out of
# the file's directory runs in about a millisecond some of the time, and the share is then almost
# all denominator: taking it off one run gave smoke q1 anything between 1.7% and 41.1% while its
# planning never left the range 795us to 1662us. A gate that fails one run in five is a gate
# somebody switches off.
";

/// Every budget, as the file holds them.
#[must_use]
pub fn render(budgets: &[Budget]) -> String {
    let mut out = String::from(HEADER);
    for budget in budgets {
        let _ = writeln!(out);
        let _ = writeln!(out, "[budget]");
        let _ = writeln!(out, "suite     {}", budget.suite);
        let _ = writeln!(out, "machine   {}", budget.machine);
        let _ = writeln!(out, "engine    {}", budget.engine);
        let _ = writeln!(out, "version   {}", budget.version);
        let _ = writeln!(out, "recorded  {}", budget.recorded);
        for query in &budget.queries {
            let _ =
                writeln!(out, "query     {:<12} {:>6} {:>6}", query.name, query.was, query.ceiling);
        }
    }
    out
}

/// Read that back.
///
/// # Errors
///
/// On a line this does not recognise, with the line number, because a budget file that half parsed
/// is a gate that half runs.
pub fn parse(text: &str) -> Result<Vec<Budget>, String> {
    let mut out: Vec<Budget> = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let at = index + 1;
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[budget]" {
            out.push(Budget {
                suite: String::new(),
                machine: String::new(),
                engine: String::new(),
                version: String::new(),
                recorded: String::new(),
                queries: Vec::new(),
            });
            continue;
        }
        let budget = out.last_mut().ok_or_else(|| format!("line {at} is outside any [budget]"))?;
        let (key, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
        let rest = rest.trim();
        match key {
            "suite" => budget.suite = rest.to_owned(),
            "machine" => budget.machine = rest.to_owned(),
            "engine" => budget.engine = rest.to_owned(),
            "version" => budget.version = rest.to_owned(),
            "recorded" => budget.recorded = rest.to_owned(),
            "query" => budget.queries.push(allowance(rest, at)?),
            _ => return Err(format!("line {at} starts with `{key}`, which is not a key here")),
        }
    }
    Ok(out)
}

/// One `query` line: a name, the share it had, and the ceiling that gave it.
fn allowance(rest: &str, at: usize) -> Result<Allowance, String> {
    let fields: Vec<&str> = rest.split_whitespace().collect();
    let [name, was, ceiling] = fields[..] else {
        return Err(format!("line {at} needs a name and two shares and has `{rest}`"));
    };
    let share = |field: &str| -> Result<u32, String> {
        let value: u32 = field.parse().map_err(|_| {
            format!("line {at}: `{field}` is not a share in hundredths of a percent")
        })?;
        if value > 10_000 {
            return Err(format!("line {at}: `{field}` is more than the whole of a query"));
        }
        Ok(value)
    };
    Ok(Allowance { name: name.to_owned(), was: share(was)?, ceiling: share(ceiling)? })
}

/// One line saying what a budget covers, for the record command to print back.
#[must_use]
pub fn describe(budget: &Budget) -> String {
    format!(
        "{} on {} at {}, {} queries",
        budget.engine,
        budget.suite,
        budget.machine,
        budget.queries.len()
    )
}

#[cfg(test)]
mod tests {
    use super::{Allowance, Budget, Spent, Verdict, allowed, parse, percent, render, report};

    #[test]
    fn a_query_that_planned_for_almost_nothing_still_gets_room_to_move() {
        // A proportional margin on a small number is not a margin. Half a percent times 1.25 is
        // 0.625 percent, which is a tenth of a point of headroom on a number that moves by more
        // than that between two runs on an idle machine.
        assert_eq!(allowed(0.005), 250);
        // And past the point where the flat floor stops being the larger of the two, the
        // proportional one takes over, so a query that plans for most of itself is not handed two
        // points and told that is generous.
        assert_eq!(allowed(0.40), 5_000);
    }

    #[test]
    fn a_query_that_spent_more_of_itself_planning_than_it_was_allowed_fails() {
        let over = Spent {
            name: "q1".to_owned(),
            share: 900,
            ceiling: Some(500),
            planning_micros: 900,
            execute_micros: 9_100,
        };
        assert!(over.over());
        assert!(!Spent { share: 500, ..over.clone() }.over(), "equal to the ceiling is inside it");
    }

    #[test]
    fn a_query_the_budget_has_never_heard_of_is_reported_and_not_failed() {
        // Otherwise the commit that adds a query to a suite has to re-record the budgets, and a
        // re-record that happens every time becomes something nobody reads the diff of.
        let fresh = Spent {
            name: "q23".to_owned(),
            share: 9_000,
            ceiling: None,
            planning_micros: 90,
            execute_micros: 10,
        };
        assert!(!fresh.over());
        let verdict =
            Verdict { engine: "rudb".to_owned(), queries: vec![fresh], unbudgeted: vec![] };
        assert!(!verdict.failed());
    }

    #[test]
    fn what_was_written_is_what_is_read_back() {
        let budgets = vec![Budget {
            suite: "smoke".to_owned(),
            machine: "gamingpc-wsl".to_owned(),
            engine: "rudb".to_owned(),
            version: "rudb 0.3.35".to_owned(),
            recorded: "2026-09-18".to_owned(),
            queries: vec![
                Allowance { name: "q1".to_owned(), was: 340, ceiling: 425 },
                Allowance { name: "q2".to_owned(), was: 12, ceiling: 212 },
            ],
        }];
        assert_eq!(parse(&render(&budgets)), Ok(budgets));
    }

    #[test]
    fn a_line_that_is_not_a_budget_line_says_which_line_it_was() {
        // A budget file that half parsed is a gate that half runs, and a gate that half runs
        // passes.
        let broken = "[budget]\nsuite     smoke\nquery     q1  340\n";
        assert_eq!(
            parse(broken),
            Err("line 3 needs a name and two shares and has `q1  340`".to_owned())
        );
        assert_eq!(
            parse("[budget]\nwhat      no\n"),
            Err("line 2 starts with `what`, which is not a key here".to_owned())
        );
        assert_eq!(parse("suite     smoke\n"), Err("line 1 is outside any [budget]".to_owned()));
    }

    #[test]
    fn a_share_is_printed_the_way_somebody_reads_a_percentage() {
        assert_eq!(percent(0), "0.00%");
        assert_eq!(percent(340), "3.40%");
        assert_eq!(percent(10_000), "100.00%");
    }

    #[test]
    fn the_report_says_what_to_do_about_a_query_that_went_over() {
        let verdicts = vec![Verdict {
            engine: "rudb".to_owned(),
            queries: vec![Spent {
                name: "q1".to_owned(),
                share: 900,
                ceiling: Some(500),
                planning_micros: 900,
                execute_micros: 9_100,
            }],
            unbudgeted: vec![],
        }];
        let text = report(&verdicts, 1);
        assert!(text.contains("OVER"), "{text}");
        assert!(text.contains("disabled_optimizers"), "{text}");
        // The durations are printed and never stored, because they are the half of the news that
        // says whether the share that moved was worth anybody's afternoon.
        assert!(text.contains("900us planning, 9100us running"), "{text}");
    }

    #[test]
    fn a_run_with_no_budget_for_it_says_how_to_make_one() {
        let text = report(&[], 3);
        assert!(text.contains("--record"), "{text}");
    }
}
