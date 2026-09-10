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
use crate::memory::{Cost, Peak};
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
    /// What the cold run cost besides time, which is where the bytes read number is worth reading.
    pub cold: Cost,
    /// The hot runs together: the worst peak, the median CPU and the median bytes read.
    pub hot: Cost,
}

impl QueryResult {
    /// The largest peak resident set any run of this query reached.
    ///
    /// Over cold and hot together, because rule six is about what the query costs and a peak that
    /// only happened on the first run is still a peak the machine had to have.
    #[must_use]
    pub fn peak(&self) -> Peak {
        match (self.cold.peak.bytes(), self.hot.peak.bytes()) {
            (Some(cold), Some(hot)) => Peak::Bytes(cold.max(hot)),
            _ if self.cold.peak.measured() => self.hot.peak.clone(),
            _ => self.cold.peak.clone(),
        }
    }
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
            match query.peak().bytes() {
                Some(n) => worst = worst.max(n),
                None => return query.peak(),
            }
        }
        if self.queries.is_empty() {
            Peak::Unavailable("no queries ran".to_owned())
        } else {
            Peak::Bytes(worst)
        }
    }

    /// CPU seconds over the hot run of every query, when every one of them reported it.
    ///
    /// The hot run and not the cold one, so that it pairs with [`Self::hot_total`]. The two
    /// together are the ratio that says how many cores the engine actually used, which is the
    /// question a wall clock on a four core machine and a wall clock on an eight core machine
    /// cannot be compared without.
    #[must_use]
    pub fn hot_cpu(&self) -> Option<Duration> {
        let mut total = Duration::ZERO;
        for query in &self.queries {
            total += query.hot.cpu?;
        }
        Some(total)
    }

    /// Bytes read at the block layer over the hot run of every query.
    ///
    /// Expected to be zero on a machine with enough memory for the dataset, and the reason it is in
    /// the table is that when it is not zero the hot number is not a hot number.
    #[must_use]
    pub fn hot_read(&self) -> Option<u64> {
        let mut total = 0;
        for query in &self.queries {
            total += query.hot.read?;
        }
        Some(total)
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
        if !query.peak().measured() {
            reasons
                .push(format!("{} has no peak resident set, and rule six wants one", query.name));
        }
        if query.hot.implausible(query.runs.hot.headline(), crate::machine::threads_here()) {
            reasons.push(format!(
                "{} reports more CPU seconds than this machine could have given it, which is a \
                 measurement fault and not a result",
                query.name
            ));
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
        let mut costs: Vec<Cost> = Vec::with_capacity(hot + 1);
        let runs = Runs::collect(hot, || {
            costs.push(engine.run(query.sql)?);
            Ok::<(), BenchError>(())
        })?;
        // The first entry is the cold run, by the order `Runs::collect` calls the closure in.
        let (cold, rest) = costs.split_first().ok_or_else(|| BenchError::new("nothing ran"))?;
        results.push(QueryResult {
            name: query.name.to_owned(),
            shape: query.shape.to_owned(),
            runs,
            cold: cold.clone(),
            hot: together(rest),
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

/// A set of runs of one query as one cost.
///
/// Three fields and three different summaries, because they answer three different questions. The
/// peak is the worst, because the machine had to have it. The CPU is the median, so that it pairs
/// with the median wall clock next to it in the table rather than being a total of a different set
/// of runs. The bytes read is the median for the same reason, and because the question it answers
/// is whether a typical hot run touched the disk, which a total over five runs would blur.
fn together(costs: &[Cost]) -> Cost {
    Cost {
        peak: worst(&costs.iter().map(|c| c.peak.clone()).collect::<Vec<_>>()),
        cpu: middle(costs.iter().map(|c| c.cpu).collect()),
        read: middle(costs.iter().map(|c| c.read).collect()),
    }
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

/// The median of a set of measurements, and nothing at all when one of them is missing.
///
/// One missing sample makes the whole thing missing rather than making the median a median of the
/// rest, because a column that silently changes what it is a median of between rows is worse than
/// a column with a gap in it.
fn middle<T: Copy + Ord>(values: Vec<Option<T>>) -> Option<T> {
    let mut got: Vec<T> = Vec::with_capacity(values.len());
    for value in values {
        got.push(value?);
    }
    got.sort_unstable();
    got.get(got.len() / 2).copied()
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
            "load     {} to build, {} of CPU, {} on disk",
            show(result.loaded.took),
            result.loaded.cpu.map_or_else(|| "no reading".to_owned(), show),
            crate::memory::bytes(result.loaded.on_disk)
        ),
    );
    line(&mut out, "");
    // The header goes through the same format string as the rows, because a header written out by
    // hand is a header that drifts by one space the first time a column gets wider, and a column
    // that has drifted by one space is read as the column next to it.
    line(
        &mut out,
        &row("query", "shape", "cold", "hot median", "IQR", "hot cpu", "peak RSS", "cold read"),
    );
    for query in &result.queries {
        let spread = query
            .runs
            .hot
            .relative_iqr()
            .map_or_else(|| "n/a".to_owned(), |r| format!("{:.1}%", r * 100.0));
        line(
            &mut out,
            &row(
                &query.name,
                &query.shape,
                &show(query.runs.cold),
                &show(query.runs.hot.headline()),
                &spread,
                &query.hot.cpu.map_or_else(|| "not read".to_owned(), show),
                &peak_cell(&query.peak()),
                &read_cell(query.cold.read),
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
    line(
        &mut out,
        &format!(
            "cpu      {} over the hot runs",
            result.hot_cpu().map_or_else(|| "not read".to_owned(), show)
        ),
    );
    line(&mut out, &format!("peak     {}", result.peak()));
    if let Some(read) = result.hot_read() {
        if read > 0 {
            line(
                &mut out,
                &format!(
                    "read     {} at the block layer during the hot runs, so hot was not warm",
                    crate::memory::bytes(read)
                ),
            );
        }
    }
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

/// One line of the per query table, header included.
#[expect(clippy::too_many_arguments, reason = "eight columns, and they are the eight columns")]
fn row(
    name: &str,
    shape: &str,
    cold: &str,
    hot: &str,
    iqr: &str,
    cpu: &str,
    peak: &str,
    read: &str,
) -> String {
    format!(
        "{name:<5}  {shape:<22}  {cold:>10}  {hot:>10}  {iqr:>7}  {cpu:>9}  {peak:>10}  {read:>10}"
    )
}

/// One peak, or the short form of why there is not one, for a table cell.
fn peak_cell(peak: &Peak) -> String {
    peak.bytes().map_or_else(|| "not read".to_owned(), crate::memory::bytes)
}

/// Bytes read, where a zero is a result and not a gap.
///
/// Zero is the expected answer for a hot run and it is the whole reason the column is there, so it
/// prints as `none` rather than as `0 B`, which reads like a missing number in a column of sizes.
fn read_cell(read: Option<u64>) -> String {
    match read {
        Some(0) => "none".to_owned(),
        Some(n) => crate::memory::bytes(n),
        None => "not read".to_owned(),
    }
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

    use super::{QueryResult, SuiteResult, middle, publishable, spread, table, together, worst};
    use crate::engine::Loaded;
    use crate::measure::{Distribution, Runs};
    use crate::memory::{Cost, Peak};
    use crate::suite::find;

    fn cost(peak: Peak, read: Option<u64>) -> Cost {
        Cost { peak, cpu: Some(Duration::from_millis(30)), read }
    }

    fn result(peak: Peak, hot: usize) -> SuiteResult {
        let samples = vec![Duration::from_millis(10); hot];
        SuiteResult {
            suite: find("smoke").unwrap(),
            engine: "duckdb".to_owned(),
            version: "v1.5.5".to_owned(),
            loaded: Loaded {
                took: Duration::from_secs(1),
                on_disk: 1024 * 1024,
                cpu: Some(Duration::from_secs(3)),
            },
            queries: vec![QueryResult {
                name: "q1".to_owned(),
                shape: "count".to_owned(),
                runs: Runs { cold: Duration::from_millis(40), hot: Distribution::median(samples) },
                cold: cost(peak.clone(), Some(4096)),
                hot: cost(peak, Some(0)),
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
    fn the_table_carries_the_cpu_and_what_the_cold_run_read() {
        let text = table(&result(Peak::Bytes(2 * 1024 * 1024), 5));
        assert!(text.contains("hot cpu"), "{text}");
        assert!(text.contains("cold read"), "{text}");
        assert!(text.contains("4.00 KiB"), "{text}");
        assert!(text.contains("3.000s of CPU"), "{text}");
    }

    #[test]
    fn a_hot_run_that_touched_the_disk_says_so_under_the_table() {
        let mut result = result(Peak::Bytes(1024), 5);
        result.queries[0].hot.read = Some(8 * 1024 * 1024);
        let text = table(&result);
        assert!(text.contains("hot was not warm"), "{text}");
    }

    #[test]
    fn a_hot_run_that_read_nothing_says_nothing() {
        let text = table(&result(Peak::Bytes(1024), 5));
        assert!(!text.contains("hot was not warm"), "{text}");
    }

    #[test]
    fn a_cpu_number_the_machine_could_not_have_given_blocks_publication() {
        let mut result = result(Peak::Bytes(1024), 5);
        // Ten seconds of CPU out of a ten millisecond query, which no machine has.
        result.queries[0].hot.cpu = Some(Duration::from_secs(10));
        let reasons = publishable(&result);
        assert!(reasons.iter().any(|r| r.contains("more CPU seconds")), "{reasons:?}");
    }

    #[test]
    fn the_hot_summary_takes_the_worst_peak_and_the_median_of_the_rest() {
        let runs = vec![
            Cost { peak: Peak::Bytes(10), cpu: Some(Duration::from_secs(1)), read: Some(0) },
            Cost { peak: Peak::Bytes(90), cpu: Some(Duration::from_secs(3)), read: Some(512) },
            Cost { peak: Peak::Bytes(30), cpu: Some(Duration::from_secs(2)), read: Some(0) },
        ];
        let got = together(&runs);
        assert_eq!(got.peak, Peak::Bytes(90));
        assert_eq!(got.cpu, Some(Duration::from_secs(2)));
        assert_eq!(got.read, Some(0));
    }

    #[test]
    fn one_run_without_a_number_makes_the_median_missing_rather_than_shorter() {
        assert_eq!(middle(vec![Some(1), None, Some(3)]), None);
        assert_eq!(middle(vec![Some(3), Some(1), Some(2)]), Some(2));
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
