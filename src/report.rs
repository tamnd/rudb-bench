//! Running a suite and printing what happened.
//!
//! Rule three says the whole suite including the losses, in a table, and that a geometric mean with
//! no per-query table is not a result. So the table is the primary output and there is no mode that
//! prints only a total. Rules four, five and six say cold sits next to hot, load time and on-disk
//! size sit next to every runtime result, and peak resident memory does too, so those are columns
//! and rows in the same artifact rather than things a reader is expected to go and find.
//!
//! Rule eight says any published number is reproducible by one documented command on a named
//! machine type, and rule one says the comparison is stated exactly. Both are handled the same way:
//! [`publishable`] returns the list of reasons a result may not be published, and the table prints
//! that list under itself. Today it is never empty, because no machine this project owns is a
//! `c6a.4xlarge`. That is the point. A harness whose output says out loud what it is not is one
//! whose numbers can be trusted when it eventually says nothing.

use std::time::Duration;

use crate::engine::{BenchError, Engine, Loaded};
use crate::measure::{Distribution, Runs, show};
use crate::memory::Peak;
use crate::suite::{Query, Suite};

/// What one query cost.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryResult {
    /// The name from the suite.
    pub name: String,
    /// The shape, so a row is readable without the SQL.
    pub shape: String,
    /// The cold run and the hot distribution.
    pub runs: Runs,
    /// The largest peak resident set any of the runs reached.
    pub peak: Peak,
}

/// What a whole suite cost on one engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuiteResult {
    /// The suite that ran.
    pub suite: &'static Suite,
    /// The engine, as it names itself.
    pub engine: String,
    /// The engine's exact version, per rule one.
    pub version: String,
    /// What the load cost.
    pub loaded: Loaded,
    /// Every query, in the order they ran, losses included.
    pub queries: Vec<QueryResult>,
}

impl SuiteResult {
    /// Sum of the hot headline over every query, which is the diagnostic total.
    #[must_use]
    pub fn hot_total(&self) -> Duration {
        self.queries.iter().map(|q| q.runs.hot.headline()).sum()
    }

    /// Sum of the cold runs, which is closer to what a user's first pass at the data costs.
    #[must_use]
    pub fn cold_total(&self) -> Duration {
        self.queries.iter().map(|q| q.runs.cold).sum()
    }

    /// The largest peak any query reached, when every query reported one.
    #[must_use]
    pub fn peak(&self) -> Peak {
        let mut worst = 0u64;
        for query in &self.queries {
            match query.peak.bytes() {
                Some(n) => worst = worst.max(n),
                None => return query.peak.clone(),
            }
        }
        if self.queries.is_empty() {
            Peak::Unavailable("no queries ran".to_owned())
        } else {
            Peak::Bytes(worst)
        }
    }
}

/// Every reason this result may not be published, which is empty only when there are none.
///
/// The list is deliberately exhaustive rather than short circuiting on the first reason. A person
/// reading it is trying to work out what would have to change, and finding out one at a time over
/// three runs is how a benchmark afternoon disappears.
#[must_use]
pub fn publishable(result: &SuiteResult) -> Vec<String> {
    let mut reasons = Vec::new();

    if crate::FLEET.iter().all(|m| !m.role.may_publish()) {
        reasons.push(format!(
            "no machine this project owns is a {}, per rule seven",
            crate::REPORTING_MACHINE
        ));
    }
    if !result.suite.comparable {
        reasons.push(format!("the {} suite is not comparable to any board", result.suite.name));
    }
    for query in &result.queries {
        if !query.runs.hot.publishable() {
            reasons.push(format!(
                "{} has {} hot runs and rule two wants at least five",
                query.name,
                query.runs.hot.runs()
            ));
        }
        if !query.peak.measured() {
            reasons
                .push(format!("{} has no peak resident set, and rule six wants one", query.name));
        }
    }
    reasons
}

/// Load the data, then run every query cold once and hot `hot` times.
///
/// # Errors
///
/// When the engine cannot load the data or a query fails. A suite that reported the queries that
/// happened to work would be measuring a different suite.
pub fn run(
    engine: &mut dyn Engine,
    suite: &'static Suite,
    queries: &[Query],
    load: &[&str],
    hot: usize,
) -> Result<SuiteResult, BenchError> {
    if !engine.can_run() {
        return Err(BenchError::new(format!("{} cannot run a query yet", engine.name())));
    }
    let loaded = engine.load(load)?;

    let mut results = Vec::with_capacity(queries.len());
    for query in queries {
        let mut peaks: Vec<Peak> = Vec::with_capacity(hot + 1);
        let runs = Runs::collect(hot, || {
            peaks.push(engine.run(query.sql)?);
            Ok::<(), BenchError>(())
        })?;
        results.push(QueryResult {
            name: query.name.to_owned(),
            shape: query.shape.to_owned(),
            runs,
            peak: worst(&peaks),
        });
    }

    Ok(SuiteResult {
        suite,
        engine: engine.name().to_owned(),
        version: engine.version().to_owned(),
        loaded,
        queries: results,
    })
}

/// The largest peak over a set of runs, or the first reason there is not one.
fn worst(peaks: &[Peak]) -> Peak {
    let mut largest = None;
    for peak in peaks {
        match peak.bytes() {
            Some(n) => largest = Some(largest.map_or(n, |m: u64| m.max(n))),
            None => return peak.clone(),
        }
    }
    largest.map_or_else(|| Peak::Unavailable("nothing ran".to_owned()), Peak::Bytes)
}

/// Render the per-query table and everything that has to be read next to it.
#[must_use]
pub fn table(result: &SuiteResult) -> String {
    let mut out = String::new();
    let line = |out: &mut String, text: &str| {
        out.push_str(text);
        out.push('\n');
    };

    line(&mut out, &format!("suite    {}", result.suite.name));
    line(&mut out, &format!("engine   {} {}", result.engine, result.version));
    line(
        &mut out,
        &format!(
            "load     {} to build, {} on disk",
            show(result.loaded.took),
            crate::memory::bytes(result.loaded.on_disk)
        ),
    );
    line(&mut out, "");
    line(
        &mut out,
        "query  shape                     cold        hot median         IQR   peak RSS",
    );
    for query in &result.queries {
        let spread = query
            .runs
            .hot
            .relative_iqr()
            .map_or_else(|| "  n/a".to_owned(), |r| format!("{:.1}%", r * 100.0));
        line(
            &mut out,
            &format!(
                "{:<5}  {:<22}  {:>9}  {:>11}  {:>10}  {:>9}",
                query.name,
                query.shape,
                show(query.runs.cold),
                show(query.runs.hot.headline()),
                spread,
                peak_cell(&query.peak),
            ),
        );
    }
    line(&mut out, "");
    line(
        &mut out,
        &format!(
            "total    {} cold, {} hot, over {} queries",
            show(result.cold_total()),
            show(result.hot_total()),
            result.queries.len()
        ),
    );
    line(&mut out, &format!("peak     {}", result.peak()));
    line(&mut out, "");

    let reasons = publishable(result);
    if reasons.is_empty() {
        line(&mut out, "Nothing here blocks publication. Rule one still applies: state the");
        line(&mut out, "machine, the kernel, the filesystem and the settings next to the number.");
    } else {
        line(&mut out, "This is not a publishable number, because:");
        for reason in &reasons {
            line(&mut out, &format!("  {reason}"));
        }
    }
    line(&mut out, "");
    line(&mut out, "Hot here means page cache warm and not buffer pool warm, because every run is");
    line(&mut out, "a fresh process so that no query's number depends on the one before it.");
    out
}

/// One peak, or the short form of why there is not one, for a table cell.
fn peak_cell(peak: &Peak) -> String {
    peak.bytes().map_or_else(|| "not read".to_owned(), crate::memory::bytes)
}

/// A distribution printed the long way, for the diagnostic output.
#[must_use]
pub fn spread(d: &Distribution) -> String {
    format!(
        "{} median over {} runs, p25 {}, p75 {}, fastest {}, slowest {}",
        show(d.median_of()),
        d.runs(),
        show(d.p25()),
        show(d.p75()),
        show(d.fastest()),
        show(d.slowest())
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{QueryResult, SuiteResult, publishable, spread, table, worst};
    use crate::engine::Loaded;
    use crate::measure::{Distribution, Runs};
    use crate::memory::Peak;
    use crate::suite::find;

    fn result(peak: Peak, hot: usize) -> SuiteResult {
        let samples = vec![Duration::from_millis(10); hot];
        SuiteResult {
            suite: find("smoke").unwrap(),
            engine: "duckdb".to_owned(),
            version: "v1.5.5".to_owned(),
            loaded: Loaded { took: Duration::from_secs(1), on_disk: 1024 * 1024 },
            queries: vec![QueryResult {
                name: "q1".to_owned(),
                shape: "count".to_owned(),
                runs: Runs { cold: Duration::from_millis(40), hot: Distribution::median(samples) },
                peak,
            }],
        }
    }

    #[test]
    fn a_result_from_a_machine_we_own_can_never_be_published() {
        let reasons = publishable(&result(Peak::Bytes(1024), 5));
        assert!(reasons.iter().any(|r| r.contains("c6a.4xlarge")), "{reasons:?}");
    }

    #[test]
    fn a_missing_peak_is_its_own_reason_and_names_the_query() {
        let reasons = publishable(&result(Peak::Unavailable("no timer".to_owned()), 5));
        assert!(reasons.iter().any(|r| r.contains("q1") && r.contains("peak")), "{reasons:?}");
    }

    #[test]
    fn three_runs_is_its_own_reason() {
        let reasons = publishable(&result(Peak::Bytes(1024), 3));
        assert!(reasons.iter().any(|r| r.contains("at least five")), "{reasons:?}");
    }

    #[test]
    fn every_reason_is_listed_and_not_only_the_first() {
        let reasons = publishable(&result(Peak::Unavailable("no timer".to_owned()), 3));
        assert!(reasons.len() >= 4, "{reasons:?}");
    }

    #[test]
    fn the_table_carries_the_load_the_peak_and_the_losses() {
        let text = table(&result(Peak::Bytes(2 * 1024 * 1024), 5));
        assert!(text.contains("load"), "{text}");
        assert!(text.contains("peak"), "{text}");
        assert!(text.contains("q1"), "{text}");
        assert!(text.contains("2.00 MiB"), "{text}");
        assert!(text.contains("not a publishable number"), "{text}");
        assert!(text.contains("page cache warm"), "{text}");
    }

    #[test]
    fn the_suite_peak_is_the_worst_query_and_not_the_last_one() {
        let peaks = vec![Peak::Bytes(10), Peak::Bytes(400), Peak::Bytes(30)];
        assert_eq!(worst(&peaks), Peak::Bytes(400));
    }

    #[test]
    fn one_run_without_a_peak_makes_the_whole_thing_unmeasured() {
        let peaks =
            vec![Peak::Bytes(10), Peak::Unavailable("the timer died".to_owned()), Peak::Bytes(30)];
        assert!(!worst(&peaks).measured());
    }

    #[test]
    fn the_long_form_names_every_number_it_prints() {
        let d = Distribution::median(vec![Duration::from_millis(10); 5]);
        let text = spread(&d);
        assert!(text.contains("median"));
        assert!(text.contains("p25"));
        assert!(text.contains("slowest"));
    }
}
