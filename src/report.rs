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

use crate::data::Table;
use crate::engine::{BenchError, Engine, Loaded};
use crate::measure::{Distribution, Runs, show};
use crate::memory::{Cost, Peak};
use crate::suite::{Query, Suite};

/// How wide a spread has to be before a run is called disturbed rather than measured.
///
/// Ten percent, which is the figure `cargo xtask bench` in the rudb repository already prints under
/// its own table as the point where the machine was busy and the run should be taken again. It is
/// not derived from anything. It is a round number that has held up on this fleet, and the reason
/// it is a named constant with this paragraph attached is so that the day somebody wants to move it
/// they have to say why in a commit rather than in a magic number.
///
/// The consequence of crossing it is a sentence and never a refusal to print. Section 13.8 of
/// `spec/engine/13-measurement.md` is explicit that a run too noisy to be published cannot fail a
/// build either, because a gate that fires randomly is a gate that gets disabled, and the same
/// argument applies to a report that hides its own output.
pub const NOISY: f64 = 0.10;

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
    /// What the engine answered, kept so that two engines answering differently is visible.
    ///
    /// From the cold run, which is the same answer as every other run unless something is very
    /// wrong, and taking it from the first one means the comparison holds even when a later run
    /// fails and the suite stops.
    pub answer: String,
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

    /// The widest spread any query showed, as a fraction of its own median.
    ///
    /// The worst rather than the average, because the question this answers is whether anything in
    /// the run was disturbed, and one query that swung by half while the other five were steady is
    /// a run where something else was using the machine. An average over six would hide it.
    ///
    /// `None` when a query was too fast for the clock to have a ratio about, which is a different
    /// thing from a steady one and prints differently.
    #[must_use]
    pub fn worst_iqr(&self) -> Option<f64> {
        if self.queries.is_empty() {
            return None;
        }
        self.queries
            .iter()
            .map(|q| q.runs.hot.relative_iqr())
            .try_fold(0.0_f64, |worst, spread| Some(worst.max(spread?)))
    }

    /// The query that produced [`Self::worst_iqr`], for a sentence that has to name one.
    #[must_use]
    pub fn noisiest(&self) -> Option<(&str, f64)> {
        self.queries
            .iter()
            .filter_map(|q| q.runs.hot.relative_iqr().map(|r| (q.name.as_str(), r)))
            .max_by(|a, b| a.1.total_cmp(&b.1))
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
        if let Some(spread) = query.runs.hot.relative_iqr().filter(|s| *s > NOISY) {
            // Named per query rather than summarised, because the person reading this is deciding
            // whether to rerun the suite or to go and find what else is on the machine, and which
            // query swung is the thing that tells them apart.
            reasons.push(format!(
                "{} swung by {:.1}% of its median, and rule two wants under {:.0}%",
                query.name,
                spread * 100.0,
                NOISY * 100.0
            ));
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
    tables: &[Table],
    hot: usize,
) -> Result<SuiteResult, BenchError> {
    let ability = engine.can_run(suite);
    if let Some(why) = ability.why() {
        return Err(BenchError::new(format!(
            "{} is not running {}: {why}",
            engine.name(),
            suite.name
        )));
    }
    let loaded = engine.load(tables)?;

    let mut results = Vec::with_capacity(queries.len());
    for query in queries {
        let mut costs: Vec<Cost> = Vec::with_capacity(hot + 1);
        let mut answer = String::new();
        let runs = Runs::collect(hot, || {
            let ran = engine.run(query.sql)?;
            if answer.is_empty() {
                answer = ran.answer;
            }
            costs.push(ran.cost);
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
            answer,
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
    line(&mut out, &format!("on disk  is {}", result.loaded.on_disk_is));
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

/// An engine that did not produce a row, and why not.
///
/// A missing row is information. An engine that has not been built, an engine that cannot read the
/// format, and an engine that crashed halfway through the fourth query are three different states
/// and a blank space in a table is none of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Abstention {
    /// The engine, as it names itself.
    pub engine: String,
    /// Its version, where it has one.
    pub version: String,
    /// The sentence that goes where its numbers would have been.
    pub why: String,
}

/// One suite, run on every engine that could run it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comparison {
    /// The suite that ran.
    pub suite: &'static Suite,
    /// What the source Parquet takes, which every engine's on disk number is read against.
    pub source_bytes: u64,
    /// The engines that produced numbers, in the order they were given.
    pub results: Vec<SuiteResult>,
    /// The engines that did not, and why.
    pub skipped: Vec<Abstention>,
}

impl Comparison {
    /// Every disagreement about an answer, per query.
    ///
    /// The first engine is the reference. A benchmark comparing engines that do not agree about
    /// what the answer is is comparing how fast they are wrong, so this runs on every result and
    /// the sentences it produces sit under the table rather than in a log nobody reads.
    #[must_use]
    pub fn disagreements(&self) -> Vec<String> {
        let mut out = Vec::new();
        let Some(reference) = self.results.first() else { return out };
        for (at, query) in reference.queries.iter().enumerate() {
            let answers: Vec<(String, String)> = self
                .results
                .iter()
                .filter_map(|r| r.queries.get(at).map(|q| (r.engine.clone(), q.answer.clone())))
                .collect();
            for line in crate::answer::disagreements(&answers) {
                out.push(format!("{}: {line}", query.name));
            }
        }
        out
    }

    /// Every engine whose worst query swung wider than [`NOISY`], and which query it was.
    ///
    /// This is the one caveat the cross engine grid could not carry until now. The single engine
    /// table has an IQR column against every query and the grid dropped it, so the grid read as the
    /// more precise of the two while being the one made of the same numbers. Reporting rule two
    /// says the spread travels with the median, and a table where it travelled with only one of the
    /// two views was a table that satisfied the rule on a technicality.
    #[must_use]
    pub fn disturbed(&self) -> Vec<String> {
        self.results
            .iter()
            .filter_map(|result| {
                let (query, spread) = result.noisiest()?;
                (spread > NOISY).then(|| {
                    format!(
                        "{} swung by {:.1}% of its median on {query}, and rule two wants under \
                         {:.0}%",
                        result.engine,
                        spread * 100.0,
                        NOISY * 100.0
                    )
                })
            })
            .collect()
    }
}

/// Run one suite on every engine, keeping the ones that cannot as abstentions rather than as
/// failures.
///
/// One engine being broken does not stop the others, because the afternoon somebody is comparing
/// four engines is not the afternoon to find out that a missing `datafusion-cli` means no numbers
/// at all. What it must not do is hide it, so every one of them ends up in the report either as a
/// row or as a sentence.
pub fn compare(
    engines: &mut [Box<dyn Engine>],
    suite: &'static Suite,
    queries: &[Query],
    tables: &[Table],
    hot: usize,
) -> Comparison {
    let mut results = Vec::new();
    let mut skipped = Vec::new();

    for engine in engines.iter_mut() {
        let ability = engine.can_run(suite);
        if let Some(why) = ability.why() {
            skipped.push(Abstention {
                engine: engine.name().to_owned(),
                version: engine.version().to_owned(),
                why: why.to_owned(),
            });
            continue;
        }
        match run(engine.as_mut(), suite, queries, tables, hot) {
            Ok(result) => results.push(result),
            Err(e) => skipped.push(Abstention {
                engine: engine.name().to_owned(),
                version: engine.version().to_owned(),
                why: e.to_string(),
            }),
        }
    }

    Comparison { suite, source_bytes: tables.iter().map(|t| t.bytes).sum(), results, skipped }
}

/// The cross engine table: one column per engine, one row per query, and the supporting rows.
///
/// The first engine is the reference for the ratio row. That is DuckDB by construction, because
/// the compatibility claim is against DuckDB and so is the performance claim, and a table whose
/// reference column moved depending on what happened to be installed would not be comparable to
/// last week's.
#[must_use]
pub fn comparison(compared: &Comparison) -> String {
    let mut out = String::new();
    let line = |out: &mut String, text: &str| {
        out.push_str(text.trim_end());
        out.push('\n');
    };

    line(&mut out, &format!("suite    {}", compared.suite.name));
    line(
        &mut out,
        &format!(
            "data     {} of Parquet in {} table{}",
            crate::memory::bytes(compared.source_bytes),
            compared.suite.tables.len(),
            if compared.suite.tables.len() == 1 { "" } else { "s" }
        ),
    );
    for result in &compared.results {
        line(&mut out, &format!("engine   {} {}", result.engine, result.version));
    }
    line(&mut out, "");

    if compared.results.is_empty() {
        line(&mut out, "No engine produced a number.");
    } else {
        line(&mut out, &grid(compared));
    }

    for skip in &compared.skipped {
        line(&mut out, &format!("{} {} did not run: {}", skip.engine, skip.version, skip.why));
    }
    if !compared.skipped.is_empty() {
        line(&mut out, "");
    }

    let reading: Vec<&str> = compared
        .results
        .iter()
        .filter(|r| !r.loaded.converted)
        .map(|r| r.engine.as_str())
        .collect();
    if !reading.is_empty() {
        line(
            &mut out,
            &format!(
                "Reading the Parquet directly rather than a format of its own: {}.",
                reading.join(", ")
            ),
        );
        line(
            &mut out,
            "That is an empty load column and a decode inside every query, in the column",
        );
        line(&mut out, "being compared, which the other engines paid for once at load time.");
        line(&mut out, "");
    }

    let disturbed = compared.disturbed();
    if !disturbed.is_empty() {
        line(&mut out, "Something else was using this machine while this ran:");
        for who in &disturbed {
            line(&mut out, &format!("  {who}"));
        }
        line(
            &mut out,
            "So the ratio row is a ratio of two disturbed numbers. Take the run again on",
        );
        line(&mut out, "a quiet machine before quoting anything out of it.");
        line(&mut out, "");
    }

    let disagreements = compared.disagreements();
    if disagreements.is_empty() && compared.results.len() > 1 {
        line(
            &mut out,
            &format!(
                "All {} engines agreed on every answer, to the last significant digit of a double.",
                compared.results.len()
            ),
        );
    }
    for line_of in &disagreements {
        line(&mut out, &format!("Answers differ, so this is not a comparison: {line_of}"));
    }
    line(&mut out, "");
    line(&mut out, "Hot here means page cache warm and not buffer pool warm, because every run is");
    line(&mut out, "a fresh process so that no query's number depends on the one before it.");
    out
}

/// The body of the cross engine table, sized to whatever is actually in it.
///
/// Column widths come from the contents rather than from a constant, because the number of engines
/// is not known here and a fixed width table with five engines in it wraps, and a wrapped table is
/// read as two tables.
/// How many rows under the queries are summaries rather than queries.
///
/// A constant because the blank line that separates the two halves of the table is placed by
/// counting back from the end, and a table where somebody added a row and the blank line landed in
/// the middle of the totals is a table nobody trusts.
const SUMMARY_ROWS: usize = 9;

fn grid(compared: &Comparison) -> String {
    let reference = &compared.results[0];
    let mut header = vec!["query".to_owned(), "shape".to_owned()];
    for result in &compared.results {
        header.push(result.engine.clone());
    }

    let mut rows = vec![header];
    for (at, query) in reference.queries.iter().enumerate() {
        let mut row = vec![query.name.clone(), query.shape.clone()];
        for result in &compared.results {
            row.push(
                result
                    .queries
                    .get(at)
                    .map_or_else(|| "did not run".to_owned(), |q| show(q.runs.hot.headline())),
            );
        }
        rows.push(row);
    }

    rows.push(summary("total hot", compared, |r| show(r.hot_total())));
    // Directly under the total it qualifies, because a spread printed three rows away from the
    // number it belongs to is a spread that gets read as its own fact rather than as a caveat.
    rows.push(summary("worst IQR", compared, |r| {
        r.worst_iqr().map_or_else(|| "n/a".to_owned(), |r| format!("{:.1}%", r * 100.0))
    }));
    rows.push(summary("total cold", compared, |r| show(r.cold_total())));
    rows.push(summary("hot cpu", compared, |r| {
        r.hot_cpu().map_or_else(|| "not read".to_owned(), show)
    }));
    rows.push(summary("peak RSS", compared, |r| peak_cell(&r.peak())));
    rows.push(summary("load", compared, |r| show(r.loaded.took)));
    rows.push(summary("on disk", compared, |r| crate::memory::bytes(r.loaded.on_disk)));
    rows.push(summary("storage", compared, |r| {
        if r.loaded.converted { "own format".to_owned() } else { "the Parquet".to_owned() }
    }));
    // The ratio last, because it is the one number a reader takes away, and because it means
    // nothing without the six rows above it that say what was measured.
    let base = reference.hot_total().as_secs_f64();
    rows.push(summary(&format!("vs {}", reference.engine), compared, move |r| {
        if base <= 0.0 {
            return "n/a".to_owned();
        }
        format!("{:.2}x", r.hot_total().as_secs_f64() / base)
    }));

    let mut widths = vec![0usize; rows[0].len()];
    for row in &rows {
        for (at, cell) in row.iter().enumerate() {
            widths[at] = widths[at].max(cell.chars().count());
        }
    }

    let mut out = String::new();
    for (at, row) in rows.iter().enumerate() {
        let mut line = String::new();
        for (column, cell) in row.iter().enumerate() {
            if column > 0 {
                line.push_str("  ");
            }
            let width = widths[column];
            if column < 2 {
                line.push_str(&format!("{cell:<width$}"));
            } else {
                line.push_str(&format!("{cell:>width$}"));
            }
        }
        out.push_str(line.trim_end());
        out.push('\n');
        // A blank line between the queries and the totals, because the totals are a different kind
        // of row and a reader scanning down a column should not add one of them into a sum.
        if at == rows.len() - SUMMARY_ROWS - 1 {
            out.push('\n');
        }
    }
    out
}

/// One supporting row of the cross engine table.
fn summary(name: &str, compared: &Comparison, of: impl Fn(&SuiteResult) -> String) -> Vec<String> {
    let mut row = vec![name.to_owned(), String::new()];
    for result in &compared.results {
        row.push(of(result));
    }
    row
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

    use super::{
        Abstention, Comparison, QueryResult, SuiteResult, comparison, middle, publishable, spread,
        table, together, worst,
    };
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
                on_disk_is: "its own database file".to_owned(),
                converted: true,
                cpu: Some(Duration::from_secs(3)),
            },
            queries: vec![QueryResult {
                name: "q1".to_owned(),
                shape: "count".to_owned(),
                runs: Runs { cold: Duration::from_millis(40), hot: Distribution::median(samples) },
                cold: cost(peak.clone(), Some(4096)),
                hot: cost(peak, Some(0)),
                answer: "10000000".to_owned(),
            }],
        }
    }

    /// The same result with a different name, a different speed and a different answer, for the
    /// comparison tests.
    fn rival(name: &str, millis: u64, answer: &str) -> SuiteResult {
        let mut result = result(Peak::Bytes(1024), 5);
        result.engine = name.to_owned();
        result.version = "1.0".to_owned();
        result.queries[0].runs.hot = Distribution::median(vec![Duration::from_millis(millis); 5]);
        result.queries[0].answer = answer.to_owned();
        result
    }

    fn compared(results: Vec<SuiteResult>, skipped: Vec<Abstention>) -> Comparison {
        Comparison {
            suite: find("smoke").unwrap(),
            source_bytes: 92 * 1024 * 1024,
            results,
            skipped,
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
    fn the_comparison_puts_every_engine_in_one_column_each_and_ratios_against_the_first() {
        let text = comparison(&compared(
            vec![result(Peak::Bytes(1024), 5), rival("clickhouse-local", 5, "10000000")],
            vec![],
        ));
        assert!(text.contains("duckdb"), "{text}");
        assert!(text.contains("clickhouse-local"), "{text}");
        assert!(text.contains("vs duckdb"), "{text}");
        // Five milliseconds against ten is half the time, and the reference column is 1.00x.
        assert!(text.contains("1.00x"), "{text}");
        assert!(text.contains("0.50x"), "{text}");
        assert!(text.contains("92.00 MiB of Parquet"), "{text}");
    }

    #[test]
    fn an_engine_reading_the_parquet_is_marked_as_reading_the_parquet() {
        let mut polars = rival("polars", 5, "10000000");
        polars.loaded.converted = false;
        polars.loaded.on_disk_is = "the source Parquet".to_owned();
        let text = comparison(&compared(vec![result(Peak::Bytes(1024), 5), polars], vec![]));
        assert!(text.contains("storage"), "{text}");
        assert!(text.contains("own format"), "{text}");
        assert!(text.contains("the Parquet"), "{text}");
        assert!(text.contains("Reading the Parquet directly"), "{text}");
    }

    #[test]
    fn nothing_is_said_about_parquet_when_every_engine_converted() {
        let text = comparison(&compared(
            vec![result(Peak::Bytes(1024), 5), rival("clickhouse-local", 5, "10000000")],
            vec![],
        ));
        assert!(!text.contains("Reading the Parquet directly"), "{text}");
    }

    #[test]
    fn the_blank_line_lands_between_the_queries_and_the_totals() {
        let text = comparison(&compared(vec![result(Peak::Bytes(1024), 5)], vec![]));
        let lines: Vec<&str> = text.lines().collect();
        let at = lines.iter().position(|l| l.starts_with("total hot")).expect("a totals row");
        assert!(lines[at - 1].is_empty(), "the totals should be separated\n{text}");
        assert!(lines[at - 2].starts_with("q1"), "the queries should be above\n{text}");
    }

    #[test]
    fn an_engine_that_could_not_run_is_a_sentence_and_not_a_gap() {
        let text = comparison(&compared(
            vec![result(Peak::Bytes(1024), 5)],
            vec![Abstention {
                engine: "rudb".to_owned(),
                version: "0.1.0".to_owned(),
                why: "no path from a file into a chunk yet".to_owned(),
            }],
        ));
        assert!(text.contains("rudb 0.1.0 did not run"), "{text}");
        assert!(text.contains("into a chunk yet"), "{text}");
    }

    #[test]
    fn two_engines_that_disagree_about_the_answer_are_not_a_comparison() {
        let text = comparison(&compared(
            vec![result(Peak::Bytes(1024), 5), rival("polars", 5, "9999999")],
            vec![],
        ));
        assert!(text.contains("Answers differ"), "{text}");
        assert!(text.contains("q1"), "{text}");
    }

    #[test]
    fn engines_that_agree_are_said_to_agree_rather_than_being_left_silent() {
        let text = comparison(&compared(
            vec![result(Peak::Bytes(1024), 5), rival("polars", 5, "10000000")],
            vec![],
        ));
        assert!(text.contains("agreed on every answer"), "{text}");
    }

    #[test]
    fn a_comparison_with_nothing_in_it_says_so_rather_than_printing_an_empty_table() {
        let text = comparison(&compared(vec![], vec![]));
        assert!(text.contains("No engine produced a number"), "{text}");
    }

    /// A result whose one query swung, for the noise tests. The samples are chosen so the
    /// interquartile range is a known fraction of the median rather than whatever a random spread
    /// happens to be.
    fn swung(name: &str, samples: &[u64]) -> SuiteResult {
        let mut result = result(Peak::Bytes(1024), 5);
        result.engine = name.to_owned();
        result.queries[0].runs.hot =
            Distribution::median(samples.iter().map(|&m| Duration::from_millis(m)).collect());
        result
    }

    #[test]
    fn the_worst_spread_is_the_worst_one_and_not_an_average_of_six() {
        // The failure this is written against: five steady queries and one that swung by half,
        // averaged, look like a run that was slightly disturbed rather than a run where something
        // else grabbed the machine for one query.
        let mut result = swung("duckdb", &[10, 10, 10, 10, 10]);
        let mut wild = result.queries[0].clone();
        wild.name = "q2".to_owned();
        wild.runs.hot = Distribution::median(vec![
            Duration::from_millis(10),
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(30),
            Duration::from_millis(40),
        ]);
        result.queries.push(wild);
        let worst = result.worst_iqr().expect("both queries have a median");
        assert!(worst > 0.5, "{worst}");
        assert_eq!(result.noisiest().expect("one of them is worst").0, "q2");
    }

    #[test]
    fn a_query_that_swung_wide_is_a_reason_not_to_publish() {
        let result = swung("duckdb", &[10, 10, 20, 30, 40]);
        let reasons = publishable(&result);
        assert!(reasons.iter().any(|r| r.contains("swung by") && r.contains("q1")), "{reasons:?}");
    }

    #[test]
    fn a_steady_query_is_not_a_reason_not_to_publish() {
        let reasons = publishable(&result(Peak::Bytes(1024), 5));
        assert!(!reasons.iter().any(|r| r.contains("swung by")), "{reasons:?}");
    }

    #[test]
    fn the_grid_carries_the_spread_directly_under_the_total_it_qualifies() {
        // Rule two says the spread travels with the median. It travelled with the single engine
        // table and not with the grid, which made the grid look like the more precise of two views
        // built out of the same numbers.
        let text = comparison(&compared(vec![swung("duckdb", &[10, 10, 20, 30, 40])], vec![]));
        let lines: Vec<&str> = text.lines().collect();
        let at = lines.iter().position(|l| l.starts_with("worst IQR")).expect("a spread row");
        assert!(lines[at - 1].starts_with("total hot"), "{text}");
    }

    #[test]
    fn a_disturbed_run_names_the_engine_and_the_query_and_refuses_to_stand_behind_the_ratio() {
        let text = comparison(&compared(
            vec![result(Peak::Bytes(1024), 5), swung("polars", &[10, 10, 20, 30, 40])],
            vec![],
        ));
        assert!(text.contains("Something else was using this machine"), "{text}");
        assert!(text.contains("polars swung by"), "{text}");
        assert!(text.contains("q1"), "{text}");
        assert!(text.contains("Take the run again"), "{text}");
        // The steady engine is not accused of anything.
        assert!(!text.contains("duckdb swung by"), "{text}");
    }

    #[test]
    fn a_quiet_machine_gets_no_sentence_about_noise() {
        let text = comparison(&compared(
            vec![result(Peak::Bytes(1024), 5), rival("polars", 5, "10000000")],
            vec![],
        ));
        assert!(!text.contains("Something else was using this machine"), "{text}");
    }

    #[test]
    fn a_query_too_fast_for_the_clock_is_not_called_noisy() {
        // Zero median means no ratio exists, which is a different thing from a steady one, and
        // calling it either would be inventing a fact about a measurement that did not happen.
        let result = swung("duckdb", &[0, 0, 0, 0, 0]);
        assert_eq!(result.worst_iqr(), None);
        assert!(!publishable(&result).iter().any(|r| r.contains("swung by")));
    }

    #[test]
    fn the_table_says_what_the_on_disk_number_is_a_size_of() {
        let text = table(&result(Peak::Bytes(1024), 5));
        assert!(text.contains("its own database file"), "{text}");
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
