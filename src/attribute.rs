//! What a layer was worth, one query at a time.
//!
//! The same suite, the same data, the same machine, the same process, run twice: once with a layer
//! on and once with it off. The difference is what that layer did, and the point of the table is
//! that it is stated per query rather than as a total.
//!
//! A total is the number somebody would quote and it is the least informative one available. An
//! optimizer that makes one query in forty three a hundred times faster and the other forty two
//! slightly slower has a good total and is a bad optimizer, and a total cannot tell those apart.
//! The per query column can: it is a list of what each pass was for, sorted by how much it
//! mattered, and the rows at the bottom of it are the ones worth arguing about.
//!
//! This is why the milestone calls for an attribution row rather than a claim. A claim is "the
//! optimizer makes rudb three times faster". An attribution is forty three rows, most of them
//! near one, and a handful that are not, with the query names attached. The second is what tells
//! somebody where to work, and it is also the only one of the two that cannot be produced by
//! running the suite until it flatters you.
//!
//! # The off column has to be the same answers
//!
//! The strongest property the project has is that no layer changes answers, and it is gated per
//! commit in `tamnd/rudb-compat` over the whole sqllogictest corpus. This module gets it for free
//! and checks it anyway, on the queries it happens to be timing, because a benchmark that compares
//! a right answer against a wrong one is not slower or faster, it is meaningless. A query whose two
//! runs disagree gets a row saying so and no ratio. There is no suppression and no tolerance: the
//! answers are compared by [`crate::answer::same`], which is the same comparison the cross engine
//! table uses.
//!
//! # What gets turned off
//!
//! Three things, picked with [`Ablation`], and one of them is the original.
//!
//! `optimizer` is every name `duckdb_optimizers()` gives, fed back to `SET disabled_optimizers`.
//! The list is asked of the engine rather than written down, for the reason
//! [`crate::engine::Engine::no_optimizer`] gives. Which names those were is printed under the
//! table, because an attribution against an optimizer whose contents are not stated is an
//! attribution against nothing in particular.
//!
//! The count in that line is how many names the engine gave and not how many passes it has. rudb
//! answers with 44 and runs 9, because the setting is DuckDB's and a corpus file that disables a
//! pass rudb has not written yet has to be accepted rather than rejected. So the printed list is
//! the truthful answer to "what was turned off" and is not a count of the optimizer's parts. Which
//! of the nine did the work is a different question, and it has its own command: `rudb-compat
//! sweep` in `tamnd/rudb-compat` runs the corpus with pass k on and the rest off, so a difference
//! localizes to one pass. This says what the pipeline was worth; that says which pass it was.
//!
//! It is also not the same thing as no planning at all. Binding, and whatever the engine does
//! before the pass list runs, still happen. What this measures is the passes, which is what the
//! setting turns off and what the milestone is about.
//!
//! `statistics` and `graph_sections` are rudb's own rules, and they are the two the G series is
//! built on: what the statistics a query planner reads are worth, and what the graph sections in
//! the native format are worth. They go through [`crate::engine::Engine::set_rule`] rather than
//! through `disabled_optimizers`, and an engine that has never heard of the name says so at the
//! start rather than four minutes into a suite.
//!
//! One command covers all three because they are one idea and not three. A layer is on, a layer is
//! off, the same queries run both ways, the difference is a table. Writing that three times would
//! give three tables that drift apart in their column widths, their timeout handling and their
//! answer checking, and only the first of those is cosmetic.
//!
//! # This is not a quick command
//!
//! The unoptimized half runs the plan the binder emitted, and for a join that is a cross product
//! filtered afterwards. The first run of this on the smoke suite spent over two minutes on a single
//! hot run of one join of ten million rows that takes well under a second with the passes on, and
//! there are `hot` of those plus a cold one. Each query gets the suite's default limit, so a shape
//! that does not come back costs that query and not the run, and the row it leaves says as much.
//! That is the measurement working rather than the measurement hanging, and it is still a command
//! somebody runs on purpose rather than something wired into a per commit gate.

use std::time::Duration;

use crate::data::Dataset;
use crate::engine::{BenchError, Engine};
use crate::measure::show;
use crate::report::{SuiteResult, run};
use crate::suite::{Query, Suite};

/// What the second run turns off.
///
/// Three documents ask for the same ablation, so this is one type with three values rather than
/// three commands. `spec/stats/09-measurement.md` section 9.3 asks for `statistics = off`,
/// `spec/graph/09-measurement.md` section 9.2 asks for `graph_sections = off`, and the optimizer
/// was here first. All three are the same question: run the suite twice, once with a layer and
/// once without it, and say per query what the layer was worth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ablation {
    /// Every optimizer pass the engine names, through `SET disabled_optimizers`.
    Optimizer,
    /// One named rule, through `SET <name>`.
    ///
    /// Both runs set it, one on and one off. Which way the rule's default points is a decision that
    /// moves, and a result that quietly changed meaning on the day it moved would be worse than no
    /// result.
    Rule(String),
}

impl Ablation {
    /// What `--off` takes, and what the table prints.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Optimizer => "optimizer",
            Self::Rule(name) => name,
        }
    }

    /// What `--off` takes, read back.
    ///
    /// Anything that is not `optimizer` is a rule name, passed to the engine as written. The list
    /// of rules lives in the engine and not here, for the reason [`crate::engine::Engine::set_rule`]
    /// gives: a copy of it kept in the harness goes stale the day the engine gains one, and what
    /// that produces is not an error anybody sees.
    #[must_use]
    pub fn parse(given: &str) -> Self {
        match given.trim() {
            "optimizer" => Self::Optimizer,
            other => Self::Rule(other.to_owned()),
        }
    }
}

/// One query, timed both ways.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribution {
    /// The name from the suite.
    pub name: String,
    /// The shape, so a row is readable without the SQL.
    pub shape: String,
    /// The hot headline with the engine as it comes.
    pub with: Duration,
    /// The hot headline with the layer off.
    pub without: Duration,
    /// Whether the two runs answered the same thing.
    ///
    /// False is a bug in the engine rather than a slow query, and the row prints as one.
    pub agreed: bool,
}

impl Attribution {
    /// How many times faster the run with the layer on was, or nothing when there is no ratio.
    ///
    /// `None` when the runs disagreed, because a ratio between two different answers is a number
    /// about nothing, and `None` when either side is zero, which is a query too small to time
    /// rather than an infinitely good optimizer.
    #[must_use]
    pub fn ratio(&self) -> Option<f64> {
        if !self.agreed || self.with.is_zero() || self.without.is_zero() {
            return None;
        }
        Some(self.without.as_secs_f64() / self.with.as_secs_f64())
    }

    /// What the layer saved on this query, and nothing when it cost time instead.
    #[must_use]
    pub fn saved(&self) -> Option<Duration> {
        self.without.checked_sub(self.with).filter(|d| !d.is_zero())
    }
}

/// A whole attribution, ready to print.
#[derive(Debug, Clone)]
pub struct Attributed {
    /// The suite that ran, twice.
    pub suite: &'static Suite,
    /// The engine, as it names itself.
    pub engine: String,
    /// Its exact version, per rule one.
    pub version: String,
    /// What was turned off for the second run.
    pub ablation: Ablation,
    /// The names that were turned off, in the order the engine gave them.
    ///
    /// Every optimizer pass for [`Ablation::Optimizer`], and the one rule for [`Ablation::Rule`].
    /// It is printed under the table, because an attribution against a layer whose contents are not
    /// stated is an attribution against nothing in particular.
    pub turned_off: Vec<String>,
    /// Queries this engine has no text for, with the suite's reason, and empty for most engines.
    ///
    /// Not a failure and not a timeout. The suite withholds a query from an engine when the number
    /// it would produce is about something other than the query, and the reason is written down
    /// next to the text. It is printed because the row count under the table is otherwise smaller
    /// than the suite's and nothing says why, and a reader who notices that is right to wonder.
    pub not_run: Vec<(String, String)>,
    /// One row per query the two runs have in common, in suite order.
    pub rows: Vec<Attribution>,
    /// Queries that came back from one run and not the other, with which side had them.
    ///
    /// Expected to be empty, and printed rather than dropped for when it is not. [`crate::report::run`]
    /// gives up on the whole suite when a query fails, so the case this is really about, a query
    /// that only finishes with the layer on, does not arrive here: it arrives as the whole
    /// attribution failing with the side that could not finish named in the message. That is the
    /// right trade for a suite result, and the sentence under the table says which side it was, so
    /// the finding is not lost even though the other forty two rows are.
    pub lost: Vec<(String, String)>,
}

impl Attributed {
    /// The sum over every query of the runs with the layer on.
    #[must_use]
    pub fn with_total(&self) -> Duration {
        self.rows.iter().map(|row| row.with).sum()
    }

    /// The same with the layer off.
    #[must_use]
    pub fn without_total(&self) -> Duration {
        self.rows.iter().map(|row| row.without).sum()
    }

    /// Whether every query the two runs share answered the same thing both ways.
    ///
    /// The one result here that is a property of the engine rather than of the machine, which makes
    /// it the one worth failing on.
    #[must_use]
    pub fn agreed(&self) -> bool {
        self.rows.iter().all(|row| row.agreed)
    }

    /// The queries the layer made slower, worst first.
    ///
    /// The half of the table nobody publishes and the half worth reading. A pass that pays for
    /// itself over a suite can still be a loss on a query, and the name of that query is the whole
    /// value of doing this per query.
    #[must_use]
    pub fn losses(&self) -> Vec<&Attribution> {
        let mut losses: Vec<&Attribution> =
            self.rows.iter().filter(|row| row.agreed && row.with > row.without).collect();
        losses.sort_by(|a, b| {
            let a = a.with.saturating_sub(a.without);
            let b = b.with.saturating_sub(b.without);
            b.cmp(&a)
        });
        losses
    }
}

/// Run the suite twice on one engine, with the layer and without it.
///
/// Back to back in that order, so that the machine has as little chance to change underneath the
/// two runs as this harness can arrange, and the run with the layer on is the one that goes first
/// because it is the one a reader will compare everything else to.
///
/// # Errors
///
/// When the engine has no way to turn the named layer off, or when either run of the suite fails.
/// Which side failed is in the message, because a suite that finishes with the optimizer on and
/// not with it off is the strongest attribution available and it would otherwise read as a broken
/// benchmark.
pub fn attribute(
    mut engine: Box<dyn Engine>,
    suite: &'static Suite,
    queries: &[Query],
    dataset: &Dataset,
    hot: usize,
    limit: Option<Duration>,
    ablation: Ablation,
) -> Result<Attributed, String> {
    let name = engine.name().to_owned();
    let version = engine.version().to_owned();
    if let Some(why) = engine.can_run(suite).why() {
        return Err(format!("{name} cannot run {}: {why}", suite.name));
    }

    // The on side first, because it is the one a reader compares everything else to, and set
    // explicitly rather than taken from the default for the reason `Engine::set_rule` gives.
    if let Ablation::Rule(rule) = &ablation {
        engine.set_rule(rule, true)?;
    }
    let what = ablation.name().to_owned();
    let with =
        run(engine.as_mut(), suite, queries, dataset, hot, limit).map_err(|e: BenchError| {
            format!("{name} did not finish {} with {what} on: {e}", suite.name)
        })?;
    let turned_off = match &ablation {
        Ablation::Optimizer => engine.no_optimizer()?,
        Ablation::Rule(rule) => {
            engine.set_rule(rule, false)?;
            vec![rule.clone()]
        }
    };
    let without =
        run(engine.as_mut(), suite, queries, dataset, hot, limit).map_err(|e: BenchError| {
            format!("{name} did not finish {} with {what} off: {e}", suite.name)
        })?;
    engine.unload();

    // From the run rather than from the suite, because which queries an engine has no text for is
    // the run's answer and not something this module should work out a second time. Both runs are
    // the same engine, so both skipped the same ones, and the first is enough.
    let not_run = withheld(&name, queries, &with.missing);

    let head = Head { suite, engine: name, version, ablation, turned_off, not_run };
    Ok(join(head, &with, &without))
}

/// Line the two runs up by query name.
///
/// By name and not by position, because a run that lost a query would otherwise shift every row
/// after it and the table would compare one query's time against the next query's.
/// The queries this engine had no text for, paired with the suite's reason for withholding them.
///
/// The reason comes out of the suite rather than being written again here, because it is already
/// next to the query text and two copies of a sentence like that one start disagreeing.
fn withheld(engine: &str, queries: &[Query], missing: &[String]) -> Vec<(String, String)> {
    missing
        .iter()
        .map(|name| {
            let why = queries
                .iter()
                .find(|query| query.name == name)
                .and_then(|query| {
                    query
                        .absent()
                        .into_iter()
                        .find(|(who, _)| *who == engine)
                        .map(|(_, why)| why.to_owned())
                })
                .unwrap_or_else(|| "this engine has no text for it".to_owned());
            (name.clone(), why)
        })
        .collect()
}

/// Everything about the run that is not a timing.
///
/// Gathered into one value so that [`join`] takes a run and two results rather than eight loose
/// arguments, four of which are strings that would sit next to each other unnoticed if two of them
/// ever swapped places.
struct Head {
    suite: &'static Suite,
    engine: String,
    version: String,
    ablation: Ablation,
    turned_off: Vec<String>,
    not_run: Vec<(String, String)>,
}

fn join(head: Head, with: &SuiteResult, without: &SuiteResult) -> Attributed {
    let Head { suite, engine, version, ablation, turned_off, not_run } = head;
    let mut rows = Vec::new();
    let mut lost = Vec::new();
    for on in &with.queries {
        match without.queries.iter().find(|off| off.name == on.name) {
            Some(off) => rows.push(Attribution {
                name: on.name.clone(),
                shape: on.shape.clone(),
                with: on.runs.hot.headline(),
                without: off.runs.hot.headline(),
                agreed: crate::answer::same(&on.answer, &off.answer),
            }),
            None => lost.push((
                on.name.clone(),
                format!("ran with {} on and not with it off", ablation.name()),
            )),
        }
    }
    for off in &without.queries {
        if !with.queries.iter().any(|on| on.name == off.name) {
            lost.push((
                off.name.clone(),
                format!("ran with {} off and not with it on", ablation.name()),
            ));
        }
    }
    Attributed { suite, engine, version, ablation, turned_off, not_run, rows, lost }
}

/// The attribution as a table, one row per query.
#[must_use]
pub fn table(attributed: &Attributed) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "suite       {}, {} queries\n",
        attributed.suite.name, attributed.suite.queries
    ));
    out.push_str(&format!("engine      {} {}\n", attributed.engine, attributed.version));
    out.push_str(&format!(
        "ablation    {}, {} name{} turned off\n\n",
        attributed.ablation.name(),
        attributed.turned_off.len(),
        if attributed.turned_off.len() == 1 { "" } else { "s" }
    ));

    out.push_str(&format!(
        "{:<10}  {:<18}  {:>10}  {:>10}  {}\n",
        "query", "shape", "with", "without", "what it was worth"
    ));
    for row in &attributed.rows {
        let worth = match row.ratio() {
            None if !row.agreed => "different answers".to_owned(),
            None => "-".to_owned(),
            Some(ratio) if ratio >= 1.0 => format!("{ratio:.2}x faster"),
            Some(ratio) => format!("{:.2}x slower", 1.0 / ratio),
        };
        out.push_str(&format!(
            "{:<10}  {:<18}  {:>10}  {:>10}  {worth}\n",
            row.name,
            row.shape,
            show(row.with),
            show(row.without)
        ));
    }
    for (name, why) in &attributed.lost {
        out.push_str(&format!("{:<10}  {why}\n", name));
    }

    out.push('\n');
    let with = attributed.with_total();
    let without = attributed.without_total();
    out.push_str(&format!(
        "{:>10} with {} on, over {} queries\n",
        show(with),
        attributed.ablation.name(),
        attributed.rows.len()
    ));
    out.push_str(&format!("{:>10} without it\n", show(without)));
    // Said here rather than left to arithmetic, because the header says how many queries the suite
    // has and the line above says how many ran, and a reader who spots the gap deserves the reason
    // rather than a guess.
    for (name, why) in &attributed.not_run {
        out.push_str(&format!("{name} did not run on {}: {why}\n", attributed.engine));
    }
    // A sum of attributions and never a headline. It is printed because somebody will want it and
    // leaving it out would only mean it gets computed by hand from the rows above, which is the
    // same number with nothing underneath it saying what it is.
    out.push_str(
        "this total is the sum of the rows and not a claim about the engine. The rows are the \
         result\n",
    );

    let losses = attributed.losses();
    out.push('\n');
    if losses.is_empty() {
        out.push_str(&format!("no query was slower with {} on\n", attributed.ablation.name()));
    } else {
        out.push_str(&format!(
            "{} queries {} made slower, worst first\n",
            losses.len(),
            attributed.ablation.name()
        ));
        for row in &losses {
            out.push_str(&format!(
                "    {:<10}  {} slower\n",
                row.name,
                show(row.with.saturating_sub(row.without))
            ));
        }
    }

    if !attributed.agreed() {
        out.push('\n');
        out.push_str(&format!(
            "some queries answered differently with {} off, which is a bug in the engine \
                 and\nnot a timing. A layer is allowed to make a query faster and is never \
                 allowed to change\nwhat it answers\n",
            attributed.ablation.name()
        ));
    }

    out.push('\n');
    out.push_str(&format!("turned off: {}\n", attributed.turned_off.join(", ")));
    out
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Ablation, Attributed, Attribution, table, withheld};
    use crate::suite::find;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    fn row(name: &str, with: u64, without: u64) -> Attribution {
        Attribution {
            name: name.to_owned(),
            shape: "scan".to_owned(),
            with: ms(with),
            without: ms(without),
            agreed: true,
        }
    }

    fn attributed(rows: Vec<Attribution>) -> Attributed {
        of(Ablation::Optimizer, vec!["filter_pushdown".to_owned(), "top_n".to_owned()], rows)
    }

    fn of(ablation: Ablation, turned_off: Vec<String>, rows: Vec<Attribution>) -> Attributed {
        Attributed {
            suite: find("smoke").expect("the smoke suite is compiled in"),
            engine: "rudb".to_owned(),
            version: "0.3.31".to_owned(),
            ablation,
            turned_off,
            not_run: Vec::new(),
            rows,
            lost: Vec::new(),
        }
    }

    #[test]
    fn a_query_the_optimizer_made_faster_reads_as_a_ratio_above_one() {
        let one = row("q1", 100, 400);
        assert_eq!(one.ratio(), Some(4.0));
        assert_eq!(one.saved(), Some(ms(300)));
    }

    /// The half of the table that is worth having. A pass that pays for itself over a suite can
    /// still be a loss on a query, and this is where that query's name shows up.
    #[test]
    fn a_query_the_optimizer_made_slower_is_named_under_the_table() {
        let printed = table(&attributed(vec![row("q1", 100, 400), row("q2", 500, 200)]));
        assert!(printed.contains("1 queries optimizer made slower"), "{printed}");
        assert!(printed.contains("q2"), "{printed}");
        assert!(printed.contains("2.50x slower"), "{printed}");
    }

    #[test]
    fn a_suite_the_optimizer_never_lost_on_says_so_rather_than_printing_nothing() {
        let printed = table(&attributed(vec![row("q1", 100, 400)]));
        assert!(printed.contains("no query was slower with optimizer on"), "{printed}");
    }

    /// Two different answers are not a fast run and a slow one, so the row does not get a ratio and
    /// the table says the engine is wrong rather than saying the optimizer is worth four times.
    #[test]
    fn a_query_whose_two_runs_disagree_gets_no_ratio_at_all() {
        let mut one = row("q1", 100, 400);
        one.agreed = false;
        assert_eq!(one.ratio(), None);
        let printed = table(&attributed(vec![one]));
        assert!(printed.contains("different answers"), "{printed}");
        assert!(printed.contains("a bug in the engine"), "{printed}");
        assert!(!printed.contains("4.00x"), "{printed}");
    }

    #[test]
    fn a_disagreement_anywhere_makes_the_whole_attribution_not_agreed() {
        let mut two = row("q2", 100, 100);
        two.agreed = false;
        assert!(!attributed(vec![row("q1", 100, 400), two]).agreed());
        assert!(attributed(vec![row("q1", 100, 400)]).agreed());
    }

    /// A query too small to time is not an optimizer that is infinitely good.
    #[test]
    fn a_query_that_took_no_measurable_time_has_no_ratio() {
        assert_eq!(row("q1", 0, 400).ratio(), None);
        assert_eq!(row("q1", 400, 0).ratio(), None);
    }

    /// The total is printed and then told on itself, because it is the number somebody will quote
    /// and the rows above it are the result.
    #[test]
    fn the_total_says_that_it_is_a_sum_of_rows_and_not_a_claim() {
        let printed = table(&attributed(vec![row("q1", 100, 400), row("q2", 200, 300)]));
        assert!(printed.contains("not a claim about the engine"), "{printed}");
        assert!(printed.contains("turned off: filter_pushdown, top_n"), "{printed}");
    }

    /// `optimizer` is the word the flag takes and everything else is a rule name, because the
    /// rules live in rudb and a list of them kept here would go stale without anybody noticing.
    #[test]
    fn the_word_optimizer_is_the_pass_list_and_every_other_word_is_a_rule() {
        assert_eq!(Ablation::parse("optimizer"), Ablation::Optimizer);
        assert_eq!(Ablation::parse("  optimizer  "), Ablation::Optimizer);
        assert_eq!(Ablation::parse("statistics"), Ablation::Rule("statistics".to_owned()));
        assert_eq!(Ablation::parse("graph_sections"), Ablation::Rule("graph_sections".to_owned()));
        assert_eq!(Ablation::parse("optimiser"), Ablation::Rule("optimiser".to_owned()));
        assert_eq!(Ablation::Rule("statistics".to_owned()).name(), "statistics");
    }

    /// The whole reason this is one type and not three commands: the table is the same table, and
    /// the only thing that moves is the name of what was turned off.
    #[test]
    fn a_rule_ablation_prints_the_rule_name_everywhere_the_optimizer_one_prints_its_own() {
        let one = of(
            Ablation::Rule("graph_sections".to_owned()),
            vec!["graph_sections".to_owned()],
            vec![row("q1", 100, 400), row("q2", 500, 200)],
        );
        let printed = table(&one);
        assert!(printed.contains("ablation    graph_sections, 1 name turned off"), "{printed}");
        assert!(printed.contains("with graph_sections on"), "{printed}");
        assert!(printed.contains("1 queries graph_sections made slower"), "{printed}");
        assert!(printed.contains("turned off: graph_sections"), "{printed}");
        assert!(!printed.contains("optimizer"), "{printed}");
    }

    /// Plural on two names and not on one, because the line is read by people and `1 names` is the
    /// sort of thing that makes a reader wonder what else was not looked at.
    #[test]
    fn the_count_of_what_was_turned_off_reads_as_english_either_way() {
        let many = table(&attributed(vec![row("q1", 100, 400)]));
        assert!(many.contains("ablation    optimizer, 2 names turned off"), "{many}");
        let one = of(
            Ablation::Rule("statistics".to_owned()),
            vec!["statistics".to_owned()],
            vec![row("q1", 100, 400)],
        );
        assert!(table(&one).contains("ablation    statistics, 1 name turned off"), "{one:?}");
    }

    /// The header says six and the total says five, and without this line the gap between them is
    /// left for the reader to invent an explanation for.
    #[test]
    fn a_query_the_suite_withholds_from_this_engine_is_named_with_its_reason() {
        let mut one = attributed(vec![row("q1", 100, 400)]);
        one.not_run.push(("q6".to_owned(), "every join in rudb is a nested loop".to_owned()));
        let printed = table(&one);
        assert!(printed.contains("q6 did not run on rudb: every join"), "{printed}");
    }

    /// Against the real smoke suite, because the first version of this looked the reason up under
    /// the query's name instead of the engine's and printed a fallback sentence that was true and
    /// said nothing. A fixture would have agreed with the bug.
    #[test]
    fn the_reason_a_query_was_withheld_comes_from_the_suite_and_not_from_a_fallback() {
        let smoke = crate::suite::queries("smoke").expect("the smoke suite has its queries");
        let found = withheld("rudb", smoke, &["q6".to_owned()]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, "q6");
        assert!(found[0].1.contains("nested loop"), "{:?}", found[0]);

        // duckdb has text for q6, so there is nothing to look up and the fallback is what is left.
        let other = withheld("duckdb", smoke, &["q6".to_owned()]);
        assert_eq!(other[0].1, "this engine has no text for it");
    }

    #[test]
    fn a_query_that_only_ran_one_way_is_a_row_saying_so_and_not_a_missing_row() {
        let mut one = attributed(vec![row("q1", 100, 400)]);
        one.lost.push(("q2".to_owned(), "ran with optimizer on and not with it off".to_owned()));
        let printed = table(&one);
        assert!(printed.contains("q2"), "{printed}");
        assert!(printed.contains("not with it off"), "{printed}");
    }
}
