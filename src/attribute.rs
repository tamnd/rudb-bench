//! What the optimizer was worth, one query at a time.
//!
//! The same suite, the same data, the same machine, the same process, run twice: once with the
//! engine as it comes, and once with every optimizer it has turned off. The difference is what the
//! optimizer did, and the point of the table is that it is stated per query rather than as a total.
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
//! The strongest property the project has is that the optimizer does not change answers, and it is
//! gated per commit in `tamnd/rudb-compat` over the whole sqllogictest corpus. This module gets it
//! for free and checks it anyway, on the queries it happens to be timing, because a benchmark that
//! compares a right answer against a wrong one is not slower or faster, it is meaningless. A query
//! whose two runs disagree gets a row saying so and no ratio. There is no suppression and no
//! tolerance: the answers are compared by [`crate::answer::same`], which is the same comparison the
//! cross engine table uses.
//!
//! # What "off" means
//!
//! Every name `duckdb_optimizers()` gives, fed back to `SET disabled_optimizers`. The list is asked
//! of the engine rather than written down, for the reason [`crate::engine::Engine::no_optimizer`]
//! gives. Which names those were is printed under the table, because an attribution against an
//! optimizer whose contents are not stated is an attribution against nothing in particular.
//!
//! It is also not the same thing as no planning at all. Binding, and whatever the engine does
//! before the pass list runs, still happen. What this measures is the passes, which is what the
//! setting turns off and what the milestone is about.
//!
//! # This is not a quick command
//!
//! The unoptimized half runs the plan the binder emitted, and for a join that is a cross product
//! filtered afterwards. The first run of this on the smoke suite spent over two minutes on a single
//! hot run of one join of ten million rows that takes well under a second with the passes on, and
//! there are `hot` of those plus a cold one. Nothing here has a clock on it, in keeping with the
//! rest of the harness, so a suite whose unoptimized shape does not finish will not finish. That is
//! the measurement working rather than the measurement hanging, and it is the reason this is a
//! command somebody runs on purpose rather than something wired into a per commit gate.

use std::time::Duration;

use crate::data::Dataset;
use crate::engine::{BenchError, Engine};
use crate::measure::show;
use crate::report::{SuiteResult, run};
use crate::suite::{Query, Suite};

/// One query, timed both ways.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribution {
    /// The name from the suite.
    pub name: String,
    /// The shape, so a row is readable without the SQL.
    pub shape: String,
    /// The hot headline with the engine as it comes.
    pub with: Duration,
    /// The hot headline with every optimizer off.
    pub without: Duration,
    /// Whether the two runs answered the same thing.
    ///
    /// False is a bug in the engine rather than a slow query, and the row prints as one.
    pub agreed: bool,
}

impl Attribution {
    /// How many times faster the optimized run was, or nothing when there is no ratio to take.
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

    /// What the optimizer saved on this query, and nothing when it cost time instead.
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
    /// Every optimizer that was turned off, in the order the engine named them.
    pub optimizers: Vec<String>,
    /// One row per query the two runs have in common, in suite order.
    pub rows: Vec<Attribution>,
    /// Queries that came back from one run and not the other, with which side had them.
    ///
    /// Expected to be empty, and printed rather than dropped for when it is not. [`crate::report::run`]
    /// gives up on the whole suite when a query fails, so the case this is really about, a query
    /// that only finishes with the optimizer on, does not arrive here: it arrives as the whole
    /// attribution failing with the side that could not finish named in the message. That is the
    /// right trade for a suite result, and the sentence under the table says which side it was, so
    /// the finding is not lost even though the other forty two rows are.
    pub lost: Vec<(String, String)>,
}

impl Attributed {
    /// The sum over every query of the optimized runs.
    #[must_use]
    pub fn with_total(&self) -> Duration {
        self.rows.iter().map(|row| row.with).sum()
    }

    /// The same with every optimizer off.
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

    /// The queries the optimizer made slower, worst first.
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

/// Run the suite twice on one engine, with the optimizer and without it.
///
/// Back to back in that order, so that the machine has as little chance to change underneath the
/// two runs as this harness can arrange, and the optimized run is the one that goes first because
/// it is the one a reader will compare everything else to.
///
/// # Errors
///
/// When the engine has no way to turn its optimizer off, or when either run of the suite fails.
/// Which side failed is in the message, because a suite that finishes with the optimizer on and
/// not with it off is the strongest attribution available and it would otherwise read as a broken
/// benchmark.
pub fn attribute(
    mut engine: Box<dyn Engine>,
    suite: &'static Suite,
    queries: &[Query],
    dataset: &Dataset,
    hot: usize,
) -> Result<Attributed, String> {
    let name = engine.name().to_owned();
    let version = engine.version().to_owned();
    if let Some(why) = engine.can_run(suite).why() {
        return Err(format!("{name} cannot run {}: {why}", suite.name));
    }

    let with = run(engine.as_mut(), suite, queries, dataset, hot).map_err(|e: BenchError| {
        format!("{name} did not finish {} with its optimizer on: {e}", suite.name)
    })?;
    let optimizers = engine.no_optimizer()?;
    let without = run(engine.as_mut(), suite, queries, dataset, hot).map_err(|e: BenchError| {
        format!("{name} did not finish {} with its optimizer off: {e}", suite.name)
    })?;
    engine.unload();

    Ok(join(suite, name, version, optimizers, &with, &without))
}

/// Line the two runs up by query name.
///
/// By name and not by position, because a run that lost a query would otherwise shift every row
/// after it and the table would compare one query's time against the next query's.
fn join(
    suite: &'static Suite,
    engine: String,
    version: String,
    optimizers: Vec<String>,
    with: &SuiteResult,
    without: &SuiteResult,
) -> Attributed {
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
                "ran with the optimizer on and not with it off".to_owned(),
            )),
        }
    }
    for off in &without.queries {
        if !with.queries.iter().any(|on| on.name == off.name) {
            lost.push((
                off.name.clone(),
                "ran with the optimizer off and not with it on".to_owned(),
            ));
        }
    }
    Attributed { suite, engine, version, optimizers, rows, lost }
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
    out.push_str(&format!("optimizers  {} turned off\n\n", attributed.optimizers.len()));

    out.push_str("query       shape                 with        without     what it was worth\n");
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
        "{:>10} with the optimizer, over {} queries\n",
        show(with),
        attributed.rows.len()
    ));
    out.push_str(&format!("{:>10} without it\n", show(without)));
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
        out.push_str("no query was slower with the optimizer on\n");
    } else {
        out.push_str(&format!("{} queries the optimizer made slower, worst first\n", losses.len()));
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
        out.push_str(
            "some queries answered differently with the optimizer off, which is a bug in the \
             engine and\nnot a timing. The corpus gate in tamnd/rudb-compat is where that gets \
             chased down\n",
        );
    }

    out.push('\n');
    out.push_str(&format!("turned off: {}\n", attributed.optimizers.join(", ")));
    out
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Attributed, Attribution, table};
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
        Attributed {
            suite: find("smoke").expect("the smoke suite is compiled in"),
            engine: "rudb".to_owned(),
            version: "0.3.31".to_owned(),
            optimizers: vec!["filter_pushdown".to_owned(), "top_n".to_owned()],
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
        assert!(printed.contains("1 queries the optimizer made slower"), "{printed}");
        assert!(printed.contains("q2"), "{printed}");
        assert!(printed.contains("2.50x slower"), "{printed}");
    }

    #[test]
    fn a_suite_the_optimizer_never_lost_on_says_so_rather_than_printing_nothing() {
        let printed = table(&attributed(vec![row("q1", 100, 400)]));
        assert!(printed.contains("no query was slower with the optimizer on"), "{printed}");
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

    #[test]
    fn a_query_that_only_ran_one_way_is_a_row_saying_so_and_not_a_missing_row() {
        let mut one = attributed(vec![row("q1", 100, 400)]);
        one.lost
            .push(("q2".to_owned(), "ran with the optimizer on and not with it off".to_owned()));
        let printed = table(&one);
        assert!(printed.contains("q2"), "{printed}");
        assert!(printed.contains("not with it off"), "{printed}");
    }
}
