//! Committed plan baselines, so that a plan change is a reviewed diff.
//!
//! Milestone E1 asks for this in one sentence and the sentence contains the whole argument: a plan
//! change should be a reviewed diff rather than something noticed three weeks later in a
//! performance run. An optimizer is a pile of rewrites that fire on pattern matches, and the way
//! one goes wrong is almost never a crash. It is a pass that quietly stops firing on one query
//! shape because somebody normalised an expression slightly differently three commits ago, and the
//! only evidence is that a number got worse on a suite nobody ran that week.
//!
//! So the plan is written down. `rudb-bench plans --suite <name>` asks rudb to `EXPLAIN` every
//! query in a suite and compares the answers against `baselines/plans-<suite>.txt`, which is
//! committed. A pass that stops firing is then a failed check on the commit that did it, with the
//! before and after printed, rather than an archaeology exercise later.
//!
//! # Two tiers, and they ask two different questions
//!
//! The first tier is captured against tables that have the right schema and no rows, out of
//! `fixtures/<suite>/`, which is 36 KB for the whole of TPC-H and the smoke suite together. That is
//! the decision that makes this runnable in CI at all, and the box does say "diffed in CI". The
//! alternative is a baseline that needs the real ClickBench file, which is fifteen gigabytes and is
//! on two machines in the fleet, neither of which is a GitHub runner, so a baseline built that way
//! would be a baseline that never runs where it is supposed to run.
//!
//! What that costs is stated rather than hidden. The estimates in the plan are estimates over no
//! rows, so any pass that decides something from a row count is not being exercised the way a real
//! run exercises it. That used to cost less than it sounds like, because rudb printed `[rows
//! unknown]` on almost every node of these plans instead of `[~0 rows]` and join order was not among
//! the passes it ran. Join order is among them now, which is what this module said would make a
//! second tier due.
//!
//! So the second tier is captured against the real corpus at a scale factor, with `--scale`, and it
//! lands in `baselines/plans-<suite>-sf<n>.txt`. It is the tier where a join order change is a
//! reviewed diff, and it is recorded on a machine that has the corpus rather than checked on every
//! commit, because a check that needs a corpus no runner has is a check that never runs. Every
//! baseline carries a `data` line naming its tier, and [`compare`] refuses to diff two files that
//! do not agree on it, because a plan over no rows and a plan over SF1 are two different plans of
//! the same query on purpose and diffing them would report that as a regression per query.
//!
//! What the empty tier catches on its own is everything that comes out of the pass list rather than
//! out of the row counts: a predicate that stopped moving below a join, a projection that went back
//! to reading every column, a `TopN` that turned back into a sort and a limit, a limit that stopped
//! reaching through a projection, a subquery that stopped being flattened. That is most of what E1
//! builds, and it is why the empty tier stays the one the gate runs rather than being replaced.
//!
//! # rudb only
//!
//! The same argument as the seam sweep. A baseline of DuckDB's plans would fail on the morning
//! somebody upgrades DuckDB, which is a true statement about DuckDB and not about anything this
//! project controls, and a check that goes red for reasons nobody in the repository can act on is a
//! check people learn to ignore. The other engines are a comparison column and this is not a
//! comparison.
//!
//! # What a query that will not bind does
//!
//! It gets a `refused` line naming it and the error, and the baseline records that. This was not
//! the point of the module and it is the first thing it found. Nobody had found out which of the
//! twenty two TPC-H queries rudb's binder even accepts, because the suite was declined outright.
//! Planning needs no data and no join, so this answers it on every commit for the price of eight
//! empty Parquet files: twenty one of the twenty two planned, and q11 did not, because the HAVING
//! there compares a group total against a total over the whole table.
//!
//! That makes a refusal a first class row rather than a skipped query. A query that stops binding
//! is a regression and a query that starts binding is progress, and both of them are a diff in this
//! file. q11 is the second kind, and the diff that made it one is in this repository's history.
//!
//! # The path problem
//!
//! rudb inlines a view, so a plan holds `TableFunction read_parquet args=['/some/absolute/path']`,
//! and where that path is depends on the checkout. A baseline holding it would differ on every
//! machine, which is a baseline that fails for everyone and means nothing. So every table's path is
//! replaced by `<fixture>/<file>`, or by `<corpus>/<file>` at the scale tier, before the plan is
//! written down or compared. This is the only edit made to what the engine printed, and it is made
//! in one place, [`settle`], so that the recording side and the checking side cannot drift apart.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::data::Table;
use crate::engine::Engine;
use crate::suite::{Query, Suite};

/// One query's plan, as the engine printed it and this module settled it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// The name from the suite.
    pub name: String,
    /// The plan text, with the fixture paths settled and the seam block dropped.
    pub text: String,
}

/// A query the engine would not plan, and what it said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// The name from the suite.
    pub name: String,
    /// The engine's own sentence, on one line.
    pub why: String,
}

/// What a baseline's plans were planned against.
///
/// Two tiers, and they are two different questions rather than a cheap version and a good one.
/// [`Over::Empty`] asks what comes out of the pass list, runs on a GitHub runner on every commit
/// and cannot exercise anything that reads a row count. [`Over::Scale`] asks what the optimizer
/// does when it knows how big the tables are, which is where join order lives, and it needs the
/// corpus, so it runs where the corpus is and gets recorded rather than checked on every commit.
///
/// The tier is in the file rather than only in its name, because the two files hold plans of the
/// same twenty two queries and the difference between them is invisible to a reader who does not
/// already know which is which.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Over {
    /// The zero row tables in `fixtures/<suite>`.
    Empty,
    /// The real corpus at a scale factor, by the label `--scale` takes.
    Scale(String),
}

impl Over {
    /// What the file says it was captured over.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::Empty => "no rows".to_owned(),
            Self::Scale(at) => format!("SF{at}"),
        }
    }

    /// What goes on the end of the baseline's file name, so the two tiers are two files.
    #[must_use]
    pub fn suffix(&self) -> String {
        match self {
            Self::Empty => String::new(),
            Self::Scale(at) => format!("-sf{at}"),
        }
    }

    /// The tier a `data` line names, or [`Over::Empty`] when a file has no such line.
    ///
    /// Absent means empty because the two baselines committed before there was a second tier were
    /// captured that way, and a default that quietly says otherwise would relabel them.
    #[must_use]
    pub fn read(value: &str) -> Self {
        match value.trim().strip_prefix("SF") {
            Some(at) => Self::Scale(at.to_owned()),
            None => Self::Empty,
        }
    }
}

/// Every plan in one suite, as one committed block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plans {
    /// The suite these are the plans of.
    pub suite: String,
    /// What the queries were planned against.
    pub over: Over,
    /// The engine, which is rudb and is written down anyway so the file says what it is.
    pub engine: String,
    /// The exact version, per reporting rule one.
    pub version: String,
    /// The day it was recorded, so a stale baseline is visible rather than inferred.
    pub recorded: String,
    /// One per query that planned, in suite order.
    pub plans: Vec<Plan>,
    /// One per query that did not, in suite order.
    pub refused: Vec<Refusal>,
}

impl Plans {
    /// The plan for a query, when there is one.
    #[must_use]
    pub fn find(&self, name: &str) -> Option<&Plan> {
        self.plans.iter().find(|p| p.name == name)
    }

    /// The refusal for a query, when there is one.
    #[must_use]
    pub fn refusal(&self, name: &str) -> Option<&Refusal> {
        self.refused.iter().find(|r| r.name == name)
    }
}

/// What changed between a committed baseline and a fresh capture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// The plan is not the one that was written down.
    Replanned {
        /// The query.
        name: String,
        /// What the baseline holds.
        before: String,
        /// What the engine just printed.
        after: String,
    },
    /// A query that used to plan does not any more, which is the regression worth failing on.
    Refused {
        /// The query.
        name: String,
        /// The engine's sentence.
        why: String,
    },
    /// A query that would not bind now does, which is progress and still wants the file updated.
    Bound {
        /// The query.
        name: String,
    },
    /// A query the suite has and the baseline does not.
    Appeared {
        /// The query.
        name: String,
    },
    /// A query the baseline has and the suite does not.
    Gone {
        /// The query.
        name: String,
    },
    /// The same query refused for a different reason.
    Rerefused {
        /// The query.
        name: String,
        /// What the baseline holds.
        before: String,
        /// What the engine just said.
        after: String,
    },
    /// The two sides were planned against different data, so nothing below them was compared.
    Retiered {
        /// What the baseline was captured over.
        before: String,
        /// What the capture in hand was captured over.
        after: String,
    },
}

impl Change {
    /// The query this is about.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Replanned { name, .. }
            | Self::Refused { name, .. }
            | Self::Bound { name }
            | Self::Appeared { name }
            | Self::Gone { name }
            | Self::Rerefused { name, .. } => name,
            // Not about a query. This one is about the whole file, and the alternative to a name
            // here is an Option every caller has to unwrap for a case that has one member.
            Self::Retiered { .. } => "every query",
        }
    }

    /// Whether this is the kind of change that should fail a build.
    ///
    /// All of them. There is no such thing as an expected plan change: a plan that moved either
    /// moved because somebody meant it to, in which case `--record` puts the new one in the same
    /// pull request as the change that caused it, or it moved because somebody did not mean it to,
    /// which is the entire reason this file exists. A category of change that only printed a
    /// warning would be the category people stop reading.
    #[must_use]
    pub const fn fails(&self) -> bool {
        true
    }
}

/// The checkout the committed files are read out of.
///
/// The working directory, on the same argument as [`crate::regress::path`]: these are source files
/// that get committed and reviewed as a diff, and both callers already run from the checkout.
/// `RUDB_BENCH_ROOT` is for anybody whose layout is neither.
#[must_use]
pub fn root() -> PathBuf {
    std::env::var_os("RUDB_BENCH_ROOT").map_or_else(|| PathBuf::from("."), PathBuf::from)
}

/// Where a suite's baseline lives, per tier.
///
/// The empty tier keeps the name it has had since there was only one tier, because renaming it
/// would be a diff of two files with no plan change in it and would break every link that points at
/// the old name.
#[must_use]
pub fn path(root: &Path, suite: &str, over: &Over) -> PathBuf {
    root.join("baselines").join(format!("plans-{suite}{}.txt", over.suffix()))
}

/// Where a suite's empty tables live.
#[must_use]
pub fn fixtures(root: &Path, suite: &str) -> PathBuf {
    root.join("fixtures").join(suite)
}

/// The zero row tables for a suite, in the order the suite names them.
///
/// # Errors
///
/// When a table the suite names has no fixture, with the path it was looked for at. A missing
/// fixture is the whole reason a suite has no baseline, so it says so precisely rather than
/// producing a baseline of the tables that happened to be there.
pub fn tables(root: &Path, suite: &Suite) -> Result<Vec<Table>, String> {
    let dir = fixtures(root, suite.name);
    let mut found = Vec::with_capacity(suite.tables.len());
    for name in suite.tables {
        let path = dir.join(format!("{name}.parquet"));
        let bytes = std::fs::metadata(&path)
            .map_err(|e| format!("no fixture for {name}: {} ({e})", path.display()))?
            .len();
        found.push(Table { name: (*name).to_owned(), path, bytes });
    }
    Ok(found)
}

/// Ask the engine for every plan in the suite.
///
/// # Errors
///
/// When the engine cannot be handed the tables at all. A query the engine will not plan is a
/// refusal in the result rather than an error, because which queries bind is one of the things
/// this file is for.
pub fn capture(
    engine: &mut dyn Engine,
    suite: &'static Suite,
    queries: &[Query],
    tables: &[Table],
    over: &Over,
    today: &str,
) -> Result<Plans, String> {
    // The load is the view declarations and nothing else, because rudb reads the Parquet where it
    // lies. Its timing is thrown away rather than reported: on the empty tier loading no rows takes
    // no time, and on a real corpus the number would be a load time nobody asked for in a command
    // that measures nothing.
    engine.load(tables).map_err(|e| {
        format!(
            "{} could not be given the {} {} tables: {e}",
            engine.name(),
            over.label(),
            suite.name
        )
    })?;
    let who = engine.name().to_owned();
    let mut plans = Vec::new();
    let mut refused = Vec::new();
    for query in queries {
        let Some(sql) = query.sql_for(&who) else {
            // Not a refusal. A query with no text for this engine was never asked, and recording it
            // as something the engine would not plan would be this harness's decision written down
            // as the engine's.
            continue;
        };
        match engine.plan(sql) {
            Ok(text) => {
                plans.push(Plan { name: query.name.to_owned(), text: settle(&text, tables, over) })
            }
            Err(why) => refused.push(Refusal { name: query.name.to_owned(), why: oneline(&why) }),
        }
    }
    let out = Plans {
        suite: suite.name.to_owned(),
        over: over.clone(),
        engine: who,
        version: engine.version().to_owned(),
        recorded: today.to_owned(),
        plans,
        refused,
    };
    engine.unload();
    Ok(out)
}

/// Take the machine out of a plan.
///
/// Two edits and no others. Every table path becomes `<fixture>/<file>` or `<corpus>/<file>`,
/// because rudb inlines a view and the absolute path of the machine would otherwise be in the
/// baseline. Which of the two words it is follows the tier, because a file that says `<fixture>`
/// next to a `lineitem` of six million rows is telling a reader something that is not true. And the
/// `Seams` block at the end goes, because it is a property of the build rather than of the query
/// and is byte identical in every query of the suite, so keeping it would put twenty one copies of
/// one fact in the file and make registering a seam a twenty one query diff.
///
/// The per node `[reference]` markers stay. Those are per node, they say what will actually run,
/// and a diff that says every `Get` stopped being the reference implementation is a diff somebody
/// should see exactly once.
#[must_use]
pub fn settle(text: &str, tables: &[Table], over: &Over) -> String {
    let where_from = match over {
        Over::Empty => "<fixture>",
        Over::Scale(_) => "<corpus>",
    };
    let mut out = text.to_owned();
    for table in tables {
        let from = table.path.display().to_string();
        let file = table.path.file_name().map_or_else(String::new, |f| f.to_string_lossy().into());
        out = out.replace(&from, &format!("{where_from}/{file}"));
    }
    let cut = out.find("\nSeams\n").unwrap_or(out.len());
    out[..cut].trim_end().to_owned()
}

/// An engine's error as one line, because a refusal is a line in a committed file.
fn oneline(why: &str) -> String {
    why.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The baseline file, header and all.
#[must_use]
pub fn render(plans: &Plans) -> String {
    let mut out = String::new();
    out.push_str(&header(&plans.over));
    out.push_str("\n[plans]\n");
    let _ = writeln!(out, "suite     {}", plans.suite);
    let _ = writeln!(out, "data      {}", plans.over.label());
    let _ = writeln!(out, "engine    {}", plans.engine);
    let _ = writeln!(out, "version   {}", plans.version);
    let _ = writeln!(out, "recorded  {}", plans.recorded);
    let _ = writeln!(
        out,
        "planned   {} of {}",
        plans.plans.len(),
        plans.plans.len() + plans.refused.len()
    );
    for refusal in &plans.refused {
        let _ = writeln!(out, "refused   {}  {}", refusal.name, refusal.why);
    }
    for plan in &plans.plans {
        let _ = writeln!(out, "\nquery     {}", plan.name);
        for line in plan.text.lines() {
            if line.is_empty() {
                out.push_str("  .\n");
            } else {
                let _ = writeln!(out, "  {line}");
            }
        }
    }
    out
}

/// Read a baseline back.
///
/// # Errors
///
/// When the file is not the shape [`render`] writes. A baseline this cannot read is a baseline
/// somebody edited by hand, and the useful thing to do about that is say so rather than compare
/// against half of it.
pub fn parse(text: &str) -> Result<Plans, String> {
    let mut suite = None;
    // Absent is the empty tier rather than an error, because the two baselines committed before
    // there was a second tier have no such line and they are not malformed, they are the first one.
    let mut over = Over::Empty;
    let mut engine = None;
    let mut version = None;
    let mut recorded = None;
    let mut plans: Vec<Plan> = Vec::new();
    let mut refused = Vec::new();
    let mut current: Option<(String, Vec<String>)> = None;

    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("  ") {
            match current.as_mut() {
                Some((_, body)) => {
                    body.push(if rest == "." { String::new() } else { rest.to_owned() });
                    continue;
                }
                None => return Err(format!("a plan line with no query in front of it: {line}")),
            }
        }
        if line.trim().is_empty() {
            continue;
        }
        if let Some((name, body)) = current.take() {
            plans.push(Plan { name, text: body.join("\n").trim_end().to_owned() });
        }
        if line.starts_with('#') || line == "[plans]" {
            continue;
        }
        let Some((key, value)) = line.split_once(char::is_whitespace) else {
            return Err(format!("a line with a key and nothing after it: {line}"));
        };
        let value = value.trim();
        match key {
            "suite" => suite = Some(value.to_owned()),
            "data" => over = Over::read(value),
            "engine" => engine = Some(value.to_owned()),
            "version" => version = Some(value.to_owned()),
            "recorded" => recorded = Some(value.to_owned()),
            "planned" => {}
            "refused" => match value.split_once("  ") {
                Some((name, why)) => {
                    refused
                        .push(Refusal { name: name.trim().to_owned(), why: why.trim().to_owned() });
                }
                None => return Err(format!("a refusal with no reason on it: {line}")),
            },
            "query" => current = Some((value.to_owned(), Vec::new())),
            other => return Err(format!("a key this does not know: {other}")),
        }
    }
    if let Some((name, body)) = current {
        plans.push(Plan { name, text: body.join("\n").trim_end().to_owned() });
    }

    Ok(Plans {
        suite: suite.ok_or("no suite line")?,
        over,
        engine: engine.ok_or("no engine line")?,
        version: version.ok_or("no version line")?,
        recorded: recorded.ok_or("no recorded line")?,
        plans,
        refused,
    })
}

/// What moved between the committed baseline and a fresh capture, in suite order.
///
/// The version and the date are deliberately not compared. A baseline recorded against an older
/// rudb whose plans are all still the same is not stale, it is a baseline that held, and failing on
/// the version would make every engine release a red build with no plan change in it.
///
/// The tier is compared, and it is the one difference that stops the comparison rather than adding
/// to it. A plan over no rows and a plan over a real corpus are two different plans of the same
/// query by design, so diffing them would print twenty two replanned queries and every one of them
/// would be the harness comparing the wrong pair of files.
#[must_use]
pub fn compare(before: &Plans, after: &Plans) -> Vec<Change> {
    if before.over != after.over {
        return vec![Change::Retiered { before: before.over.label(), after: after.over.label() }];
    }
    let mut changes = Vec::new();
    for plan in &after.plans {
        match (before.find(&plan.name), before.refusal(&plan.name)) {
            (Some(was), _) if was.text != plan.text => changes.push(Change::Replanned {
                name: plan.name.clone(),
                before: was.text.clone(),
                after: plan.text.clone(),
            }),
            (Some(_), _) => {}
            (None, Some(_)) => changes.push(Change::Bound { name: plan.name.clone() }),
            (None, None) => changes.push(Change::Appeared { name: plan.name.clone() }),
        }
    }
    for refusal in &after.refused {
        match (before.find(&refusal.name), before.refusal(&refusal.name)) {
            (Some(_), _) => changes
                .push(Change::Refused { name: refusal.name.clone(), why: refusal.why.clone() }),
            (None, Some(was)) if was.why != refusal.why => changes.push(Change::Rerefused {
                name: refusal.name.clone(),
                before: was.why.clone(),
                after: refusal.why.clone(),
            }),
            (None, Some(_)) => {}
            (None, None) => changes.push(Change::Appeared { name: refusal.name.clone() }),
        }
    }
    for plan in &before.plans {
        if after.find(&plan.name).is_none() && after.refusal(&plan.name).is_none() {
            changes.push(Change::Gone { name: plan.name.clone() });
        }
    }
    for refusal in &before.refused {
        if after.find(&refusal.name).is_none() && after.refusal(&refusal.name).is_none() {
            changes.push(Change::Gone { name: refusal.name.clone() });
        }
    }
    changes
}

/// What to print about a set of changes.
///
/// The whole before and the whole after for a replanned query, and not a line diff. A plan is
/// twenty lines and the interesting part is the shape, so a reader wants both trees side by side
/// rather than three `-` lines out of the middle of one. The tool that produces a good line diff is
/// `git diff` on the recorded file, and this prints what somebody needs before deciding to run it.
#[must_use]
pub fn report(suite: &str, changes: &[Change]) -> String {
    let mut out = String::new();
    if changes.is_empty() {
        let _ = writeln!(out, "every plan in {suite} is the one that was written down");
        return out;
    }
    let _ = writeln!(out, "{} plans in {suite} are not what was written down\n", changes.len());
    out.push_str(&body(changes));
    out.push_str(
        "A plan that moved on purpose wants `rudb-bench plans --suite <name> --record` in the same\n\
         pull request as the change that moved it, so the diff is reviewed next to its cause.\n",
    );
    out
}

/// What to print about one suite planned twice, once with a rule on and once with it off.
///
/// A different question from the one [`report`] asks and the same evidence, so the bodies are the
/// same function and only the sentences around them differ. This one is not about a baseline and
/// nothing about it wants recording: the answer it is looking for is that there is nothing to say,
/// and anything else is a rule that changes a plan while claiming not to.
#[must_use]
pub fn ablation(suite: &str, name: &str, changes: &[Change]) -> String {
    let mut out = String::new();
    if changes.is_empty() {
        let _ =
            writeln!(out, "every plan in {suite} is the same plan with {name} on and with it off");
        return out;
    }
    let _ = writeln!(out, "{} plans in {suite} depend on whether {name} is on", changes.len());
    let _ = writeln!(
        out,
        "The first capture is with it on and the second is with it off, so below, `was` is on and \
         `is` is off.\n"
    );
    out.push_str(&body(changes));
    let _ = writeln!(
        out,
        "A rule that is supposed to be worth nothing yet cannot move a plan. Either the rule is\n\
         doing something, in which case it is no longer the honest zero the milestone asked for,\n\
         or the plan depends on something other than the rule, which is worse."
    );
    out
}

/// The evidence itself, one paragraph per change, shared by both reports.
fn body(changes: &[Change]) -> String {
    let mut out = String::new();
    for change in changes {
        match change {
            Change::Replanned { name, before, after } => {
                let _ = writeln!(out, "{name} plans differently now");
                let _ = writeln!(out, "  was");
                for line in before.lines() {
                    let _ = writeln!(out, "    {line}");
                }
                let _ = writeln!(out, "  is");
                for line in after.lines() {
                    let _ = writeln!(out, "    {line}");
                }
                out.push('\n');
            }
            Change::Refused { name, why } => {
                let _ = writeln!(out, "{name} used to plan and now will not: {why}\n");
            }
            Change::Bound { name } => {
                let _ = writeln!(out, "{name} would not bind before and now plans\n");
            }
            Change::Rerefused { name, before, after } => {
                let _ = writeln!(out, "{name} still will not plan, for a different reason");
                let _ = writeln!(out, "  was  {before}");
                let _ = writeln!(out, "  is   {after}\n");
            }
            Change::Appeared { name } => {
                let _ = writeln!(out, "{name} is in the suite and not in the baseline\n");
            }
            Change::Gone { name } => {
                let _ = writeln!(out, "{name} is in the baseline and not in the suite\n");
            }
            Change::Retiered { before, after } => {
                let _ = writeln!(out, "these were planned against different data, so nothing was");
                let _ = writeln!(out, "compared below this line");
                let _ = writeln!(out, "  was  {before}");
                let _ = writeln!(out, "  is   {after}\n");
            }
        }
    }
    out
}

/// The sentence at the top of a baseline file, with the paragraph that says which tier it is.
fn header(over: &Over) -> String {
    let tier = match over {
        Over::Empty => EMPTY_TIER,
        Over::Scale(_) => SCALE_TIER,
    };
    format!("{HEADER}{tier}#\n{TAIL}")
}

/// The paragraph a zero row baseline carries.
const EMPTY_TIER: &str = "\
# Captured against the zero row tables in fixtures/<suite>, which have the schema and none of the
# data. That is what lets this run on a GitHub runner, and what it costs is that any pass deciding
# something from a row count is not exercised here the way a real run exercises it. rudb prints
# `rows unknown` on almost every node below rather than an estimate, which is the same fact seen
# from the other side.
";

/// The paragraph a baseline over a real corpus carries.
const SCALE_TIER: &str = "\
# Captured against the real corpus at the scale factor on the data line, so the estimates below
# are over real row counts and the passes that read one, join order first among them, are exercised
# here the way a run exercises them. What it costs is the other half of the same trade: the corpus
# is not on a GitHub runner, so this file is recorded on a machine that has the data and reviewed
# as a diff rather than checked on every commit.
#
# The two tiers are two files and they are never diffed against each other. A plan over no rows and
# a plan over this corpus are two different plans of the same query on purpose, and the data line
# is what keeps the gate from reporting that difference as twenty two regressions.
";

/// The first half of the header, above the paragraph that differs per tier.
const HEADER: &str = "\
# Committed plan baselines, read by the plan gate.
#
# Written by `rudb-bench plans --suite <name> --record` and read by `rudb-bench plans --suite
# <name>`, which fails when a query plans differently than it does here. Milestone E1 in tamnd/rudb
# asks for this so that a plan change is a reviewed diff rather than something noticed three weeks
# later in a performance run.
#
";

/// The second half of the header, below the paragraph that differs per tier.
const TAIL: &str = "\
# Nothing here is a measurement and no number in it is a time. A plan is a shape, and the only
# question this file asks is whether it is still the same shape.
#
# Every line of a plan is indented two spaces and a blank line inside one is written as a lone dot,
# so that the blank line between two queries is the only blank line in the file and a reader can
# see where one plan ends. The dot rather than two spaces and nothing, because trailing whitespace
# is the kind of thing an editor removes on save and a baseline should not depend on it surviving.
";

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use super::{
        Change, Over, Plan, Plans, Refusal, ablation, capture, compare, parse, path, render, settle,
    };
    use crate::data::Table;
    use crate::engine::{Ability, BenchError, Engine, Loaded, Ran};
    use crate::suite::{Suite, find};

    fn table(name: &str, path: &str) -> Table {
        Table { name: name.to_owned(), path: PathBuf::from(path), bytes: 0 }
    }

    /// An engine that answers questions about planning and nothing else.
    ///
    /// Here so that [`capture`] is tested without a rudb on the machine. The three methods below
    /// that are not about planning are the trait's price of admission, and this fake would be an
    /// honest engine only in the sense that it never pretends to have run anything: [`Self::run`]
    /// refuses, because a plan capture must never reach it.
    struct Fake {
        /// Which engine it is pretending to be, because a suite's query text is per engine and one
        /// of the things under test here is a query the suite declines to give one of them.
        who: &'static str,
        /// What to answer per query, in suite order, `Err` for a query that will not bind.
        answers: Vec<Result<String, String>>,
        /// How many times [`Self::plan`] was asked, so the skipped query can be counted.
        asked: std::cell::Cell<usize>,
    }

    impl Engine for Fake {
        fn name(&self) -> &str {
            self.who
        }

        fn version(&self) -> &str {
            "fake 1.0"
        }

        fn can_run(&self, _suite: &Suite) -> Ability {
            Ability::Yes
        }

        fn load(&mut self, tables: &[Table]) -> Result<Loaded, BenchError> {
            Ok(Loaded {
                took: Duration::ZERO,
                on_disk: tables.iter().map(|t| t.bytes).sum(),
                on_disk_is: "nothing at all".to_owned(),
                converted: false,
                cpu: Some(Duration::ZERO),
                ..Loaded::default()
            })
        }

        fn run(&mut self, _sql: &str, _limit: Option<Duration>) -> Result<Ran, BenchError> {
            Err(BenchError::new("a plan capture must never run a query"))
        }

        fn plan(&mut self, _sql: &str) -> Result<String, String> {
            let at = self.asked.get();
            self.asked.set(at + 1);
            self.answers.get(at).cloned().unwrap_or_else(|| Ok(format!("Get #{at}")))
        }
    }

    /// The smoke suite declares q6 absent for rudb by name, so a capture over it is the case where
    /// a suite has a query this engine was never given. It must not become a refusal: a refusal is
    /// the engine's answer and this is the harness's.
    #[test]
    fn a_query_the_suite_does_not_give_this_engine_is_not_recorded_as_one_it_refused() {
        let suite = find("smoke").expect("the smoke suite is compiled in");
        let queries = crate::suite::SMOKE;
        let mut fake = Fake { who: "rudb", answers: Vec::new(), asked: std::cell::Cell::new(0) };
        let tables = [table("smoke", "/tmp/smoke.parquet")];
        let out = capture(&mut fake, suite, queries, &tables, &Over::Empty, "2026-09-18")
            .expect("a capture");
        let given = queries.iter().filter(|q| q.sql_for("rudb").is_some()).count();
        assert_eq!(out.plans.len(), given);
        assert!(out.refused.is_empty());
        assert!(out.plans.len() < queries.len(), "q6 is the one the suite declines to give rudb");
        assert!(out.find("q6").is_none());
    }

    #[test]
    fn a_query_the_engine_will_not_bind_becomes_a_refusal_and_not_an_error() {
        let suite = find("smoke").expect("the smoke suite is compiled in");
        let mut fake = Fake {
            who: "rudb",
            answers: vec![Err("Binder Error:\n  no such column".to_owned())],
            asked: std::cell::Cell::new(0),
        };
        let out = capture(&mut fake, suite, crate::suite::SMOKE, &[], &Over::Empty, "2026-09-18")
            .expect("ok");
        assert_eq!(out.refused.len(), 1);
        // One line, because a refusal is a line in a committed file.
        assert_eq!(out.refused[0].why, "Binder Error: no such column");
    }

    fn plans(plans: Vec<Plan>, refused: Vec<Refusal>) -> Plans {
        Plans {
            suite: "tpch".to_owned(),
            over: Over::Empty,
            engine: "rudb".to_owned(),
            version: "rudb 0.3.31".to_owned(),
            recorded: "2026-09-18".to_owned(),
            plans,
            refused,
        }
    }

    fn plan(name: &str, text: &str) -> Plan {
        Plan { name: name.to_owned(), text: text.to_owned() }
    }

    /// The edit that makes a baseline portable at all. Without it the checkout's own path is in
    /// the file and the check fails for everybody who is not the machine that recorded it.
    #[test]
    fn the_absolute_path_of_the_machine_that_recorded_it_is_not_in_the_plan() {
        let tables = [table("nation", "/home/someone/rudb-bench/fixtures/tpch/nation.parquet")];
        let text =
            "Get read_parquet args=['/home/someone/rudb-bench/fixtures/tpch/nation.parquet']";
        assert_eq!(
            settle(text, &tables, &Over::Empty),
            "Get read_parquet args=['<fixture>/nation.parquet']"
        );
        // The same edit and a word that is true at the other tier.
        assert_eq!(
            settle(text, &tables, &Over::Scale("1".to_owned())),
            "Get read_parquet args=['<corpus>/nation.parquet']"
        );
    }

    /// One fact about the build, repeated once per query, would make registering a seam look like
    /// every query in the suite replanning.
    #[test]
    fn the_seam_block_is_not_part_of_the_plan() {
        let text = "Get x\n\nPipelines\n  pipeline 0\n\nSeams\n  26 seams have nothing registered";
        assert_eq!(settle(text, &[], &Over::Empty), "Get x\n\nPipelines\n  pipeline 0");
    }

    #[test]
    fn a_plan_with_a_blank_line_in_it_survives_being_written_and_read() {
        let one =
            plans(vec![plan("q1", "Get x\n\nPipelines\n  pipeline 0 waits for nothing")], vec![]);
        let back = parse(&render(&one)).expect("what render wrote");
        assert_eq!(back, one);
    }

    #[test]
    fn a_refusal_survives_being_written_and_read() {
        let one = plans(
            vec![plan("q1", "Get x")],
            vec![Refusal { name: "q11".to_owned(), why: "Binder Error: no".to_owned() }],
        );
        let back = parse(&render(&one)).expect("what render wrote");
        assert_eq!(back, one);
        assert_eq!(back.refused[0].why, "Binder Error: no");
    }

    #[test]
    fn a_baseline_that_still_holds_has_nothing_to_say() {
        let one = plans(vec![plan("q1", "Get x")], vec![]);
        assert!(compare(&one, &one).is_empty());
    }

    /// G0 wants a layer that lands worth nothing, and this is the sentence that says it did.
    #[test]
    fn a_rule_that_changes_no_plan_says_so_in_one_line() {
        let one = plans(vec![plan("q1", "Get x")], vec![]);
        let text = ablation("smoke", "statistics", &compare(&one, &one));
        assert_eq!(
            text,
            "every plan in smoke is the same plan with statistics on and with it off\n"
        );
    }

    /// And the sentence that says it did not, which is the whole reason to run the check.
    #[test]
    fn a_rule_that_moves_a_plan_is_named_along_with_both_trees() {
        let on = plans(vec![plan("q1", "Filter y\n  Get x")], vec![]);
        let off = plans(vec![plan("q1", "Get x\n  Filter y")], vec![]);
        let text = ablation("smoke", "statistics", &compare(&on, &off));
        assert!(text.contains("1 plans in smoke depend on whether statistics is on"), "{text}");
        assert!(text.contains("`was` is on and `is` is off"), "{text}");
        assert!(text.contains("Filter y\n      Get x"), "{text}");
        assert!(text.contains("no longer the honest zero"), "{text}");
        // Nothing here is a baseline, so nothing here should suggest recording one.
        assert!(!text.contains("--record"), "{text}");
    }

    /// The case the whole module is for: a pass stopped firing and nothing else says so.
    #[test]
    fn a_plan_that_moved_is_reported_with_both_trees() {
        let before = plans(vec![plan("q1", "Filter y\n  Get x")], vec![]);
        let after = plans(vec![plan("q1", "Get x\n  Filter y")], vec![]);
        let changes = compare(&before, &after);
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            Change::Replanned { name, before, after } => {
                assert_eq!(name, "q1");
                assert!(before.contains("Filter y\n  Get x"));
                assert!(after.contains("Get x\n  Filter y"));
            }
            other => panic!("wanted a replan, got {other:?}"),
        }
    }

    #[test]
    fn a_query_that_used_to_plan_and_now_will_not_is_its_own_kind_of_change() {
        let before = plans(vec![plan("q1", "Get x")], vec![]);
        let after = plans(
            vec![],
            vec![Refusal { name: "q1".to_owned(), why: "Binder Error: no".to_owned() }],
        );
        assert_eq!(
            compare(&before, &after),
            vec![Change::Refused { name: "q1".to_owned(), why: "Binder Error: no".to_owned() }]
        );
    }

    /// Progress is a diff too. A query that starts binding wants the baseline updated in the same
    /// pull request, otherwise the file quietly says the engine is worse than it is.
    #[test]
    fn a_query_that_starts_binding_is_a_change_and_not_a_silent_pass() {
        let before = plans(
            vec![],
            vec![Refusal { name: "q11".to_owned(), why: "Binder Error: no".to_owned() }],
        );
        let after = plans(vec![plan("q11", "Get x")], vec![]);
        assert_eq!(compare(&before, &after), vec![Change::Bound { name: "q11".to_owned() }]);
    }

    #[test]
    fn a_refusal_whose_reason_changed_is_not_the_same_refusal() {
        let before = plans(
            vec![],
            vec![Refusal { name: "q11".to_owned(), why: "Binder Error: a".to_owned() }],
        );
        let after = plans(
            vec![],
            vec![Refusal { name: "q11".to_owned(), why: "Parser Error: b".to_owned() }],
        );
        match &compare(&before, &after)[0] {
            Change::Rerefused { name, before, after } => {
                assert_eq!(name, "q11");
                assert_eq!(before, "Binder Error: a");
                assert_eq!(after, "Parser Error: b");
            }
            other => panic!("wanted a rerefusal, got {other:?}"),
        }
    }

    #[test]
    fn a_query_the_suite_gained_and_one_it_lost_are_both_changes() {
        let before = plans(vec![plan("q1", "Get x"), plan("q2", "Get y")], vec![]);
        let after = plans(vec![plan("q1", "Get x"), plan("q3", "Get z")], vec![]);
        let changes = compare(&before, &after);
        assert!(changes.contains(&Change::Appeared { name: "q3".to_owned() }));
        assert!(changes.contains(&Change::Gone { name: "q2".to_owned() }));
    }

    /// A baseline recorded against an older rudb whose plans all still hold is a baseline that
    /// held. Failing on the version would make every engine release a red build with no plan
    /// change in it.
    #[test]
    fn a_newer_engine_with_the_same_plans_is_not_a_change() {
        let before = plans(vec![plan("q1", "Get x")], vec![]);
        let mut after = before.clone();
        after.version = "rudb 0.4.0".to_owned();
        after.recorded = "2027-01-01".to_owned();
        assert!(compare(&before, &after).is_empty());
    }

    #[test]
    fn a_file_this_cannot_read_says_so_rather_than_comparing_against_half_of_it() {
        assert!(parse("[plans]\nsuite tpch\nnonsense here\n").is_err());
        assert!(parse("[plans]\nengine rudb\n").is_err());
    }

    /// The two tiers are two files, because they hold the plans of the same twenty two queries and
    /// one name for both would mean recording one of them deletes the other.
    #[test]
    fn the_two_tiers_land_in_two_files() {
        let root = PathBuf::from("/repo");
        assert_eq!(
            path(&root, "tpch", &Over::Empty),
            PathBuf::from("/repo/baselines/plans-tpch.txt")
        );
        assert_eq!(
            path(&root, "tpch", &Over::Scale("1".to_owned())),
            PathBuf::from("/repo/baselines/plans-tpch-sf1.txt")
        );
    }

    /// The two baselines that were committed before there was a second tier have no data line, and
    /// they were captured over no rows. A default that said anything else would relabel them.
    #[test]
    fn a_baseline_with_no_data_line_is_the_empty_tier() {
        let text = "[plans]\nsuite tpch\nengine rudb\nversion rudb 0.3.31\nrecorded 2026-09-18\n";
        assert_eq!(parse(text).expect("a baseline").over, Over::Empty);
    }

    #[test]
    fn the_tier_survives_being_written_down_and_read_back() {
        let mut one = plans(vec![plan("q1", "Get x")], vec![]);
        one.over = Over::Scale("1".to_owned());
        let back = parse(&render(&one)).expect("what render wrote");
        assert_eq!(back.over, Over::Scale("1".to_owned()));
        assert_eq!(back, one);
    }

    /// The guard that makes two tiers safe. Without it, checking an SF1 capture against the empty
    /// baseline would print every query as replanned, which is the harness comparing the wrong pair
    /// of files and saying the optimizer broke.
    #[test]
    fn two_tiers_are_never_diffed_against_each_other() {
        let before = plans(vec![plan("q1", "Get x")], vec![]);
        let mut after = plans(vec![plan("q1", "Filter y\n  Get x")], vec![]);
        after.over = Over::Scale("1".to_owned());
        let changes = compare(&before, &after);
        assert_eq!(
            changes,
            vec![Change::Retiered { before: "no rows".to_owned(), after: "SF1".to_owned() }]
        );
    }

    /// The header is the only place a reader of the committed file learns which tier it is, short
    /// of the file name, so the two headers have to differ in the paragraph that says it.
    #[test]
    fn the_header_says_which_tier_the_file_is() {
        let mut one = plans(vec![plan("q1", "Get x")], vec![]);
        let empty = render(&one);
        one.over = Over::Scale("1".to_owned());
        let scaled = render(&one);
        assert!(empty.contains("# Captured against the zero row tables"));
        assert!(scaled.contains("# Captured against the real corpus"));
        assert!(empty.contains("\ndata      no rows\n"));
        assert!(scaled.contains("\ndata      SF1\n"));
    }

    /// The committed baselines are what `--record` would write today, header and all.
    ///
    /// Reading them back and writing them out again has to produce the same bytes, because a file
    /// whose header no longer matches the one this module writes turns the next recording into a
    /// diff of prose next to a diff of plans, and that is the diff nobody reads carefully.
    #[test]
    fn the_committed_baselines_are_what_this_writes_today() {
        for name in ["smoke", "tpch"] {
            let at = path(&super::root(), name, &Over::Empty);
            let text = std::fs::read_to_string(&at).expect("a committed baseline");
            let back = parse(&text).unwrap_or_else(|e| panic!("{}: {e}", at.display()));
            assert_eq!(back.over, Over::Empty);
            assert_eq!(render(&back), text, "{} is not what render writes", at.display());
        }
    }
}
