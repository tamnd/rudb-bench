//! The board: every rung of every ladder, and the tables and charts made out of it.
//!
//! A ladder is one suite run at several sizes on one machine, and it is a different question from
//! everything else this harness keeps. `baselines/<suite>.txt` asks whether one query got twice as
//! slow since yesterday. `runs/<suite>.txt` asks what a layer of the engine bought. Neither of them
//! can answer the question a reader of the README actually arrives with, which is whether the gap
//! against DuckDB grows or shrinks with the data, because both of them hold one size.
//!
//! That question needs the sizes side by side, and it needs them to have been measured close
//! enough together that the engine did not move underneath them. So a board file holds one rung per
//! size, each rung holds what every engine totalled at that size, and `rudb-bench render` turns the
//! file into the tables and the charts that go in the README and on the site.
//!
//! ## Why a rung is replaced and a run is not
//!
//! The ledger is a history and appends forever, because the interesting thing about it is what
//! changed between two rows. The board is the opposite: it is what is true now, and a ladder with
//! last week's 1m rung next to this week's 10m rung is a ladder whose slope is partly the release
//! that happened in between. So re-running a size replaces that rung, and the history of the sizes
//! lives where it already lived, in `reports/<date>/` as the full run the rung was taken from.
//!
//! Every rung says which harness commit and which day it came from, and [`stale`] is what stops the
//! replacement rule from quietly producing a ladder assembled over a month.
//!
//! ## What a rung may and may not claim
//!
//! Nothing on the board is publishable in the sense of `spec/15-rudb-bench.md` rule seven, and the
//! renderers say so on every table they produce. A ClickBench rung under the full hundred million
//! rows is a strided sample rather than the benchmark, a TPC-H rung is a generated corpus at a
//! scale factor, and none of the machines in [`crate::machine`] is a `c6a.4xlarge`. What the board
//! is for is the shape: five sizes of the same suite measured the same way on the same afternoon,
//! which is the only way to tell a fixed cost from a per row one.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::measure::show;
use crate::memory::bytes;
use crate::report::{Comparison, SuiteResult};

/// What one engine totalled at one rung.
///
/// Times by the engine's own clock wherever the engine has one, which is the number the public
/// ClickBench board publishes and the number every ratio in a report is taken against. The harness
/// clock is kept beside it in [`Column::wall`] rather than instead of it, because the difference
/// between the two is what this harness costs to ask a question, and on a thousand row rung that
/// difference is most of the wall clock: forty three processes start, link, open a database and
/// print an answer, and none of that is the engine.
///
/// Two of the engine's own totals rather than one. [`Column::hot`] is over the queries every engine
/// at this rung measured, which is the only total a ratio may be taken over, and [`Column::own`] is
/// over everything this engine answered, which is what its own row in a report says. They are equal
/// for most columns and they are not equal for Polars on ClickBench, which cannot express four of
/// the queries, so keeping one of them would either lose the ratio or lose the engine's real total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    /// The engine, as it names itself.
    pub engine: String,
    /// Its exact version, per rule one.
    pub version: String,
    /// Hot total over the queries every engine at this rung measured.
    pub hot: Duration,
    /// Hot total over every query this engine measured.
    pub own: Duration,
    /// Whether the two above are the engine's own clock or this harness's.
    ///
    /// False for an engine that reports no per query time of its own, whose total is therefore a
    /// process lifetime and is being compared against other engines' query times. That is a
    /// difference worth a column rather than a footnote, and today it is true of every engine here.
    pub reported: bool,
    /// The same hot runs by this harness's clock, around the whole subprocess.
    pub wall: Duration,
    /// The first run of each query, after the page cache was dropped where the machine allows it.
    pub cold: Duration,
    /// CPU seconds over the hot runs, when the machine would say.
    pub cpu: Option<Duration>,
    /// Worst peak resident set over the suite, when every query reported one.
    pub peak: Option<u64>,
    /// What the load cost in wall clock, and zero for an engine that reads the Parquet where it
    /// lies.
    pub load: Duration,
    /// What the data takes on disk in this engine's format, or the source Parquet for an engine
    /// with no format of its own.
    pub disk: u64,
    /// Whether the load converted the data into a format of the engine's own.
    ///
    /// Carried next to [`Column::disk`] rather than left to a footnote, because the two numbers
    /// mean different things and a size chart that does not separate them reads as a compression
    /// result. An engine that did not convert is showing the size of the Parquet every other column
    /// also read, and it is paying for the decode again inside every query time beside it.
    pub converted: bool,
    /// How many queries it answered.
    pub answered: usize,
    /// How many there were.
    pub queries: usize,
}

impl Column {
    /// Take one engine's totals out of a finished run, over the shared queries named.
    #[must_use]
    pub fn of(result: &SuiteResult, shared: &[String]) -> Self {
        Self {
            engine: result.engine.clone(),
            version: result.version.clone(),
            hot: result.best_total_over(shared).unwrap_or_else(|| result.best_total()),
            own: result.best_total(),
            reported: result.total_is_reported(),
            wall: result.hot_total(),
            cold: result.cold_total(),
            cpu: result.hot_cpu(),
            peak: result.peak().bytes(),
            load: result.loaded.took,
            disk: result.loaded.on_disk,
            converted: result.loaded.converted,
            answered: result.queries.len() - result.unmeasured(),
            queries: result.queries.len() + result.missing.len(),
        }
    }

    /// Whether this column answered everything the suite asked.
    #[must_use]
    pub fn whole(&self) -> bool {
        self.answered == self.queries
    }
}

/// One rung: a suite at one size, on one machine, on one day.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rung {
    /// The suite that ran.
    pub suite: String,
    /// The machine it ran on. Rule seven: rungs from two machines are two ladders.
    pub machine: String,
    /// What the size is called, which is `--rows` as it was typed or `sf` and the scale factor.
    pub size: String,
    /// How many rows were behind it, when the run knew.
    pub rows: Option<u64>,
    /// How many queries every engine here measured, which is what the ratios are taken over.
    pub shared: usize,
    /// This harness's commit, short.
    pub commit: String,
    /// The day it was taken, so that a ladder assembled over a month can be told from one taken in
    /// an afternoon.
    pub recorded: String,
    /// Every engine that produced numbers, in the order the run gave them.
    pub columns: Vec<Column>,
}

impl Rung {
    /// Take a whole comparison as one rung.
    #[must_use]
    pub fn of(compared: &Comparison, machine: &str, size: &str, commit: &str, today: &str) -> Self {
        let shared = shared_queries(compared);
        Self {
            suite: compared.suite.name.to_owned(),
            machine: machine.to_owned(),
            size: size.to_owned(),
            rows: compared.rows,
            shared: shared.len(),
            commit: commit.to_owned(),
            recorded: today.to_owned(),
            columns: compared.results.iter().map(|r| Column::of(r, &shared)).collect(),
        }
    }

    /// What one engine did at this rung, by name.
    #[must_use]
    pub fn column(&self, engine: &str) -> Option<&Column> {
        self.columns.iter().find(|c| c.engine == engine)
    }

    /// The engine every ratio at this rung is taken against.
    ///
    /// DuckDB by construction, because it is the system compatibility is claimed against and the
    /// one the goal is stated in terms of. A reference column that moved depending on what happened
    /// to be installed would make two rungs incomparable without either of them saying so.
    #[must_use]
    pub fn reference(&self) -> Option<&Column> {
        self.column("duckdb").or_else(|| self.columns.first())
    }

    /// This rung's size as a number, for sorting a ladder into order.
    ///
    /// The row count when the run knew it, and otherwise whatever the label parses to. A label that
    /// parses to nothing sorts last rather than first, because an unreadable size at the small end
    /// of a chart is an unreadable size that changes every ratio printed beside it.
    #[must_use]
    pub fn magnitude(&self) -> u64 {
        self.rows.unwrap_or_else(|| parse_size(&self.size).unwrap_or(u64::MAX))
    }
}

/// The queries every engine in a comparison measured.
///
/// Finished in every column rather than merely present in it, which is the rule
/// [`crate::report::table`] already applies to its own ratio row and is applied here for the same
/// reason: a query one engine gave up on contributes that engine's timeout rather than its time,
/// and a ratio over that is a ratio over a number the flag chose.
fn shared_queries(compared: &Comparison) -> Vec<String> {
    let Some(reference) = compared.results.first() else { return Vec::new() };
    reference
        .queries
        .iter()
        .map(|q| q.name.clone())
        .filter(|name| {
            compared.results.iter().all(|r| r.find(name).is_some_and(|q| q.outcome.measured()))
        })
        .collect()
}

/// Read `1k`, `10m` or `sf0.1` back to the number it means.
///
/// Only for ordering, so a scale factor is multiplied by the row count of TPC-H at factor one
/// rather than being given a unit of its own. The two suites never share a ladder, so the two
/// scales never have to be comparable with each other.
fn parse_size(label: &str) -> Option<u64> {
    if let Some(factor) = label.strip_prefix("sf") {
        let scaled = factor.parse::<f64>().ok()? * 8_661_245.0;
        return Some(scaled as u64);
    }
    let text = label.trim().to_ascii_lowercase();
    let (digits, unit) = match text.strip_suffix('k') {
        Some(rest) => (rest, 1_000),
        None => match text.strip_suffix('m') {
            Some(rest) => (rest, 1_000_000),
            None => (text.as_str(), 1),
        },
    };
    digits.parse::<u64>().ok()?.checked_mul(unit)
}

/// Every ladder on the board, keyed by suite and machine, each sorted small end first.
///
/// Two machines are two ladders and they are never merged, because rule seven says a number from
/// one machine is never compared against a number from another and a chart with both in it is that
/// comparison whether or not anybody meant it.
#[must_use]
pub fn ladders(rungs: &[Rung]) -> BTreeMap<(String, String), Vec<Rung>> {
    let mut out: BTreeMap<(String, String), Vec<Rung>> = BTreeMap::new();
    for rung in rungs {
        out.entry((rung.suite.clone(), rung.machine.clone())).or_default().push(rung.clone());
    }
    for ladder in out.values_mut() {
        ladder.sort_by_key(Rung::magnitude);
    }
    out
}

/// The rungs of a ladder that were not taken on the day the newest one was.
///
/// A ladder is read as a slope, and a slope is only a fact about the engine when every rung on it
/// measured the same engine. Two rungs a fortnight apart are two engines, and the chart drawn
/// through them has a release in the middle of it that nothing on the page mentions.
#[must_use]
pub fn stale(ladder: &[Rung]) -> Vec<&Rung> {
    let Some(newest) = ladder.iter().map(|r| r.recorded.as_str()).max() else {
        return Vec::new();
    };
    ladder.iter().filter(|r| r.recorded != newest).collect()
}

/// Where a board file lives.
///
/// One file per suite and machine rather than one file for everything, so that a run on one box
/// cannot rewrite the lines another box wrote and a ladder is one reviewable diff.
/// `RUDB_BENCH_BOARD` moves the directory, the way `RUDB_BENCH_RUNS` moves the ledger's.
#[must_use]
pub fn path(suite: &str, machine: &str) -> PathBuf {
    let root =
        std::env::var_os("RUDB_BENCH_BOARD").map_or_else(|| PathBuf::from("board"), PathBuf::from);
    root.join(format!("{suite}-{machine}.txt"))
}

/// Every board file in the board directory, read into one list.
///
/// # Errors
///
/// A file that exists and does not parse, with its name and the line number.
pub fn read_all() -> Result<Vec<Rung>, String> {
    let root =
        std::env::var_os("RUDB_BENCH_BOARD").map_or_else(|| PathBuf::from("board"), PathBuf::from);
    let entries = match std::fs::read_dir(&root) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("could not read {}: {e}", root.display())),
    };
    // Sorted, because a directory hands its files back in whatever order the filesystem holds them
    // and a README that changed every time it was rendered on a different machine is a README with
    // a permanent diff in it.
    let mut files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|at| at.extension().is_some_and(|ext| ext == "txt"))
        .collect();
    files.sort();
    let mut out = Vec::new();
    for at in files {
        out.extend(read(&at)?);
    }
    Ok(out)
}

/// Read one board file, or nothing at all when there is no file yet.
///
/// # Errors
///
/// A file that exists and does not parse, with the line number.
pub fn read(at: &Path) -> Result<Vec<Rung>, String> {
    match std::fs::read_to_string(at) {
        Ok(text) => parse(&text).map_err(|e| format!("{}: {e}", at.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(format!("could not read {}: {e}", at.display())),
    }
}

/// Put a rung on the board, replacing the one at the same size.
///
/// Replacing rather than appending, for the reason in the module header. What the replaced rung was
/// is still in `reports/<date>/`, which is where the run it came from was written.
///
/// # Errors
///
/// Anything the filesystem says, including the directory not existing, which it says by name.
pub fn write(at: &Path, rung: &Rung) -> Result<(), String> {
    let mut kept = read(at)?;
    match kept.iter_mut().find(|k| k.suite == rung.suite && k.size == rung.size) {
        Some(slot) => *slot = rung.clone(),
        None => kept.push(rung.clone()),
    }
    kept.sort_by_key(Rung::magnitude);
    if let Some(parent) = at.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("could not make {}: {e}", parent.display()))?;
    }
    std::fs::write(at, render(&kept)).map_err(|e| format!("could not write {}: {e}", at.display()))
}

/// The header every board file carries.
const HEADER: &str = "\
# One ladder, read by `rudb-bench render`.
#
# One block per rung, holding what every engine totalled over the suite at that size. Written by
# `rudb-bench run <suite> --rows <n> --board` and rendered into the tables and the charts in the
# README and under docs/.
#
# A rung is replaced when the size is measured again, unlike runs/<suite>.txt, which appends
# forever. The board is what is true now and the ledger is a history, and a ladder holding last
# week's small end next to this week's large end has a release drawn into its slope. What a rung
# used to say is still in reports/<date>/, as the run it was taken from.
#
# Times are microseconds and sizes are bytes, both as integers, and a measure the machine would not
# give is a single dash rather than a zero. `hot` is over the queries every engine at the rung
# measured and `own` is over everything that engine measured, and the two differ for a column that
# cannot express the whole suite. Both are the engine's own clock where `reported` is true, which is
# the number the public ClickBench board publishes; `wall` is the same runs by this harness's clock,
# around the whole subprocess, and the gap between them is what asking the question costs.
#
# These are regression numbers from a machine this project owns. Reporting rule seven still applies
# and nothing here is comparable to anybody's board.
";

/// Render a ladder as the file format.
#[must_use]
pub fn render(rungs: &[Rung]) -> String {
    let mut out = String::from(HEADER);
    for rung in rungs {
        let _ = writeln!(out);
        let _ = writeln!(out, "[rung]");
        let _ = writeln!(out, "suite     {}", rung.suite);
        let _ = writeln!(out, "machine   {}", rung.machine);
        let _ = writeln!(out, "size      {}", rung.size);
        let _ = writeln!(out, "rows      {}", count(rung.rows));
        let _ = writeln!(out, "shared    {}", rung.shared);
        let _ = writeln!(out, "commit    {}", rung.commit);
        let _ = writeln!(out, "recorded  {}", rung.recorded);
        for column in &rung.columns {
            let _ = writeln!(out, "[engine]");
            let _ = writeln!(out, "name      {}", column.engine);
            let _ = writeln!(out, "version   {}", column.version);
            let _ = writeln!(out, "hot       {}", column.hot.as_micros());
            let _ = writeln!(out, "own       {}", column.own.as_micros());
            let _ = writeln!(out, "reported  {}", column.reported);
            let _ = writeln!(out, "wall      {}", column.wall.as_micros());
            let _ = writeln!(out, "cold      {}", column.cold.as_micros());
            let _ = writeln!(out, "cpu       {}", micros(column.cpu));
            let _ = writeln!(out, "peak      {}", count(column.peak));
            let _ = writeln!(out, "load      {}", column.load.as_micros());
            let _ = writeln!(out, "disk      {}", column.disk);
            let _ = writeln!(out, "converted {}", column.converted);
            let _ = writeln!(out, "answered  {}", column.answered);
            let _ = writeln!(out, "queries   {}", column.queries);
        }
    }
    out
}

/// A duration that may not have been read, as the file writes it.
fn micros(value: Option<Duration>) -> String {
    value.map_or_else(|| "-".to_owned(), |d| d.as_micros().to_string())
}

/// A count that may not have been read, as the file writes it.
fn count(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_owned(), |n| n.to_string())
}

/// Parse the file format.
///
/// Hand written, like every other reader in this crate, because a harness whose credibility is its
/// whole point does not grow a dependency tree for a file with eleven keys in it.
///
/// # Errors
///
/// A line that is not a key it knows, a key outside the block it belongs to, a number that is not
/// one, or a rung missing something a rung has to have. Every one of them names the line.
pub fn parse(text: &str) -> Result<Vec<Rung>, String> {
    let mut out: Vec<Rung> = Vec::new();
    let mut inside_engine = false;
    for (index, line) in text.lines().enumerate() {
        let at = index + 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[rung]" {
            out.push(Rung {
                suite: String::new(),
                machine: String::new(),
                size: String::new(),
                rows: None,
                shared: 0,
                commit: String::new(),
                recorded: String::new(),
                columns: Vec::new(),
            });
            inside_engine = false;
            continue;
        }
        if line == "[engine]" {
            let rung = out.last_mut().ok_or(format!("line {at}: an engine before any rung"))?;
            rung.columns.push(Column {
                engine: String::new(),
                version: String::new(),
                hot: Duration::ZERO,
                own: Duration::ZERO,
                reported: false,
                wall: Duration::ZERO,
                cold: Duration::ZERO,
                cpu: None,
                peak: None,
                load: Duration::ZERO,
                disk: 0,
                converted: false,
                answered: 0,
                queries: 0,
            });
            inside_engine = true;
            continue;
        }
        let (key, value) = line
            .split_once(char::is_whitespace)
            .ok_or(format!("line {at}: `{line}` is not a key and a value"))?;
        let value = value.trim();
        let rung = out.last_mut().ok_or(format!("line {at}: `{key}` before any rung"))?;
        if inside_engine {
            let column =
                rung.columns.last_mut().ok_or(format!("line {at}: `{key}` before any engine"))?;
            match key {
                "name" => column.engine = value.to_owned(),
                "version" => column.version = value.to_owned(),
                "hot" => column.hot = duration(value, at)?,
                "own" => column.own = duration(value, at)?,
                "reported" => column.reported = flag(value, at)?,
                "wall" => column.wall = duration(value, at)?,
                "cold" => column.cold = duration(value, at)?,
                "cpu" => column.cpu = maybe_duration(value, at)?,
                "peak" => column.peak = maybe_number(value, at)?,
                "load" => column.load = duration(value, at)?,
                "disk" => column.disk = number(value, at)?,
                "converted" => column.converted = flag(value, at)?,
                "answered" => column.answered = number(value, at)? as usize,
                "queries" => column.queries = number(value, at)? as usize,
                _ => return Err(format!("line {at}: an engine has no `{key}`")),
            }
            continue;
        }
        match key {
            "suite" => rung.suite = value.to_owned(),
            "machine" => rung.machine = value.to_owned(),
            "size" => rung.size = value.to_owned(),
            "rows" => rung.rows = maybe_number(value, at)?,
            "shared" => rung.shared = number(value, at)? as usize,
            "commit" => rung.commit = value.to_owned(),
            "recorded" => rung.recorded = value.to_owned(),
            _ => return Err(format!("line {at}: a rung has no `{key}`")),
        }
    }
    for rung in &out {
        if rung.suite.is_empty() || rung.machine.is_empty() || rung.size.is_empty() {
            return Err(format!("a rung is missing its suite, its machine or its size: {rung:?}"));
        }
    }
    Ok(out)
}

/// A `true` or a `false`, or the line it was neither on.
fn flag(text: &str, at: usize) -> Result<bool, String> {
    match text {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("line {at}: `{text}` is not true or false")),
    }
}

/// A number, or the line it was not on.
fn number(text: &str, at: usize) -> Result<u64, String> {
    text.parse::<u64>().map_err(|e| format!("line {at}: `{text}` is not a number, {e}"))
}

/// A number that may be a dash.
fn maybe_number(text: &str, at: usize) -> Result<Option<u64>, String> {
    if text == "-" { Ok(None) } else { number(text, at).map(Some) }
}

/// Microseconds as a duration.
fn duration(text: &str, at: usize) -> Result<Duration, String> {
    Ok(Duration::from_micros(number(text, at)?))
}

/// Microseconds that may be a dash.
fn maybe_duration(text: &str, at: usize) -> Result<Option<Duration>, String> {
    if text == "-" { Ok(None) } else { duration(text, at).map(Some) }
}

/// How a size is written in a table, which is the label with a thousands separated row count.
#[must_use]
pub fn size_of(rung: &Rung) -> String {
    match rung.rows {
        Some(rows) => format!("{} ({} rows)", rung.size, grouped(rows)),
        None => rung.size.clone(),
    }
}

/// A count with a thousands separator, since a row count without one is a count nobody reads.
#[must_use]
pub fn grouped(value: u64) -> String {
    let digits = value.to_string();
    let mut out = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// One engine's line across a ladder, as a ratio against the reference at each rung.
///
/// `None` at a rung the engine did not run, which prints as a gap rather than as a one, because an
/// engine that was not there is not an engine that tied.
#[must_use]
pub fn ratios(ladder: &[Rung], engine: &str) -> Vec<Option<f64>> {
    ladder
        .iter()
        .map(|rung| {
            let base = rung.reference()?.hot.as_secs_f64();
            let mine = rung.column(engine)?.hot.as_secs_f64();
            if base <= 0.0 { None } else { Some(mine / base) }
        })
        .collect()
}

/// Every engine that appears anywhere on a ladder, reference first and the rest in the order they
/// were first seen.
#[must_use]
pub fn engines(ladder: &[Rung]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for rung in ladder {
        for column in &rung.columns {
            if !out.iter().any(|seen| seen == &column.engine) {
                out.push(column.engine.clone());
            }
        }
    }
    if let Some(at) = out.iter().position(|name| name == "duckdb") {
        out.swap(0, at);
    }
    out
}

/// A duration as the tables print it.
#[must_use]
pub fn time(value: Duration) -> String {
    show(value)
}

/// A size as the tables print it.
#[must_use]
pub fn size(value: u64) -> String {
    bytes(value)
}

#[cfg(test)]
mod tests {
    use super::{Column, Rung, engines, grouped, ladders, parse, ratios, render, stale, write};
    use std::time::Duration;

    fn column(engine: &str, hot_ms: u64) -> Column {
        Column {
            engine: engine.to_owned(),
            version: format!("{engine} 1.0"),
            hot: Duration::from_millis(hot_ms),
            own: Duration::from_millis(hot_ms),
            reported: true,
            wall: Duration::from_millis(hot_ms * 2),
            cold: Duration::from_millis(hot_ms * 3),
            cpu: Some(Duration::from_millis(hot_ms * 3)),
            peak: Some(1024 * 1024),
            load: Duration::from_millis(10),
            disk: 4096,
            converted: true,
            answered: 43,
            queries: 43,
        }
    }

    fn rung(size: &str, rows: u64, duck: u64, rudb: u64) -> Rung {
        Rung {
            suite: "clickbench".to_owned(),
            machine: "gamingpc-wsl".to_owned(),
            size: size.to_owned(),
            rows: Some(rows),
            shared: 43,
            commit: "abc1234".to_owned(),
            recorded: "2026-09-21".to_owned(),
            columns: vec![column("duckdb", duck), column("rudb", rudb)],
        }
    }

    #[test]
    fn a_rung_written_is_the_rung_read_back() {
        let written = vec![rung("1k", 1000, 100, 50), rung("1m", 1_000_000, 500, 900)];
        let read = parse(&render(&written)).expect("parses");
        assert_eq!(read, written);
    }

    #[test]
    fn a_ladder_comes_back_small_end_first_whatever_order_it_was_written() {
        let out = ladders(&[rung("1m", 1_000_000, 500, 900), rung("1k", 1000, 100, 50)]);
        let ladder = out.values().next().expect("one ladder");
        assert_eq!(ladder.iter().map(|r| r.size.as_str()).collect::<Vec<_>>(), ["1k", "1m"]);
    }

    /// Rule seven, as a property of the renderer rather than a sentence under it.
    #[test]
    fn two_machines_are_two_ladders() {
        let mut other = rung("1k", 1000, 100, 50);
        other.machine = "server3".to_owned();
        let out = ladders(&[rung("1k", 1000, 100, 50), other]);
        assert_eq!(out.len(), 2, "two machines came back as one ladder");
    }

    #[test]
    fn the_ratio_is_against_duckdb_and_a_missing_engine_is_a_gap() {
        let mut short = rung("1m", 1_000_000, 500, 900);
        short.columns.pop();
        let ladder = vec![rung("1k", 1000, 100, 50), short];
        assert_eq!(ratios(&ladder, "rudb"), vec![Some(0.5), None]);
        assert_eq!(ratios(&ladder, "duckdb"), vec![Some(1.0), Some(1.0)]);
    }

    /// The reference is DuckDB wherever DuckDB ran, and not whichever engine happens to be first in
    /// the file, because a ratio whose base moved between two rungs is two ratios.
    #[test]
    fn duckdb_is_the_reference_even_when_it_is_not_first() {
        let mut moved = rung("1k", 1000, 100, 50);
        moved.columns.reverse();
        assert_eq!(moved.reference().map(|c| c.engine.as_str()), Some("duckdb"));
        assert_eq!(engines(&[moved])[0], "duckdb");
    }

    #[test]
    fn a_rung_taken_on_another_day_is_named_as_stale() {
        let mut old = rung("1k", 1000, 100, 50);
        old.recorded = "2026-09-01".to_owned();
        let ladder = vec![old, rung("1m", 1_000_000, 500, 900)];
        assert_eq!(stale(&ladder).len(), 1);
        assert_eq!(stale(&ladder)[0].size, "1k");
        assert!(stale(&ladder[1..]).is_empty(), "one rung is never stale against itself");
    }

    #[test]
    fn measuring_a_size_again_replaces_that_rung_and_leaves_the_others() {
        let dir = std::env::temp_dir().join(format!("rudb-bench-board-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let at = dir.join("clickbench-gamingpc-wsl.txt");
        write(&at, &rung("1k", 1000, 100, 50)).expect("writes");
        write(&at, &rung("1m", 1_000_000, 500, 900)).expect("writes");
        let mut again = rung("1k", 1000, 100, 25);
        again.recorded = "2026-09-22".to_owned();
        write(&at, &again).expect("writes");
        let read = super::read(&at).expect("reads");
        assert_eq!(read.len(), 2, "the second 1k made a third rung");
        assert_eq!(read[0].column("rudb").expect("rudb").hot, Duration::from_millis(25));
        assert_eq!(read[1].size, "1m", "replacing a rung disturbed another one");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_line_that_is_not_the_format_says_which_line_it_was() {
        let e = parse("[rung]\nsuite clickbench\nwidth 7\n").expect_err("refused");
        assert!(e.contains("line 3"), "{e}");
        let e = parse("[rung]\nsuite clickbench\n[engine]\nname duckdb\nhot later\n")
            .expect_err("refused");
        assert!(e.contains("line 5"), "{e}");
        assert!(parse("name duckdb\n").is_err(), "an engine key before any rung");
    }

    #[test]
    fn a_row_count_is_written_so_a_person_can_read_it() {
        assert_eq!(grouped(999_975), "999,975");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(1_000), "1,000");
        assert_eq!(grouped(99_997_497), "99,997,497");
    }
}
