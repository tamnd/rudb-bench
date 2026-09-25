//! Workloads that run for a duration, beside the suites that run a query.
//!
//! A [`Suite`](crate::suite::Suite) is a list of queries, each timed once per run, and the harness
//! starts an engine's command line for every one of them. That is the wrong shape for YCSB, where
//! an operation takes a microsecond and the thing measured is a rate held for a minute. So a
//! [`Workload`] hands the whole run to a driver, the `rudb-bench-kv` program in `kv/`, and reads
//! back the lines it printed. The driver runs as a child and never inside the harness, which keeps
//! the harness free of dependencies and of `unsafe`, and keeps a crash in an engine out of it.
//!
//! The driver's output is lines of words. [`parse`] reads them into a [`WorkloadRun`]. A run without
//! the closing `end` line is a failed run whatever else it printed, because the numbers of a driver
//! that died half way are numbers about part of a window.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crate::engine::{BenchError, Outcome};
use crate::histogram::Histogram;

/// How the clients pace themselves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pace {
    /// Each client sends its next operation when the last one returns.
    Closed,
    /// Operations are due on a schedule at `rate` a second over all clients, and a late one is
    /// timed from when it was due.
    Open { rate: f64, poisson: bool },
}

/// One run of a workload: what to run it against and for how long.
#[derive(Debug, Clone, PartialEq)]
pub struct RunPlan {
    /// `null`, `sqlite`, `duckdb`, `postgres` or `rudb`.
    pub backend: String,
    /// The database file, or the conninfo for PostgreSQL.
    pub target: String,
    /// Whether to load the table first.
    pub load: bool,
    pub records: u64,
    pub clients: usize,
    /// `read=50,update=50`.
    pub mix: String,
    pub pace: Pace,
    pub warmup: Duration,
    pub window: Duration,
    pub seed: u64,
    /// `full`, `os` or `none`.
    pub level: String,
    /// The device card line, carried into the output so a number never travels without it.
    pub card: Option<String>,
}

impl RunPlan {
    /// The driver's arguments for this plan.
    #[must_use]
    pub fn arguments(&self) -> Vec<String> {
        let mut words = vec![
            "--backend".to_string(),
            self.backend.clone(),
            "--target".to_string(),
            self.target.clone(),
            "--records".to_string(),
            self.records.to_string(),
            "--clients".to_string(),
            self.clients.to_string(),
            "--mix".to_string(),
            self.mix.clone(),
            "--warmup".to_string(),
            format!("{}s", self.warmup.as_secs_f64()),
            "--window".to_string(),
            format!("{}s", self.window.as_secs_f64()),
            "--seed".to_string(),
            self.seed.to_string(),
            "--level".to_string(),
            self.level.clone(),
        ];
        if self.load {
            words.push("--load".to_string());
        }
        if let Pace::Open { rate, poisson } = self.pace {
            words.extend(["--loop".to_string(), "open".to_string(), "--rate".to_string()]);
            words.push(rate.to_string());
            if poisson {
                words.push("--poisson".to_string());
            }
        }
        if let Some(card) = &self.card {
            words.extend(["--card".to_string(), card.clone()]);
        }
        words
    }
}

/// Where a workload's database lives once it is ready to run against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prepared {
    pub target: String,
}

/// One second of one operation.
#[derive(Debug, Clone, PartialEq)]
pub struct Interval {
    pub second: u64,
    pub op: String,
    pub n: u64,
    /// Operations that started more than a millisecond after they were due.
    pub late: u64,
    pub histogram: Histogram,
}

/// One operation over the whole window, after the warmup.
#[derive(Debug, Clone, PartialEq)]
pub struct Final {
    pub op: String,
    pub n: u64,
    pub retries: u64,
    pub failed: u64,
    pub histogram: Histogram,
}

/// What the window cost the driver's process, engine included when the engine is in process.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Cost {
    pub user: Duration,
    pub system: Duration,
    pub peak_rss: u64,
    /// Bytes written to the block device, where the kernel says.
    pub written: Option<u64>,
}

/// Everything a driver printed about one run.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkloadRun {
    /// The words of the first line, `backend=sqlite` and the rest.
    pub header: BTreeMap<String, String>,
    pub notes: Vec<String>,
    pub card: Option<String>,
    /// Rows loaded and how long it took, when the run loaded.
    pub loaded: Option<(u64, Duration)>,
    pub intervals: Vec<Interval>,
    pub finals: Vec<Final>,
    /// How late operations started, on an open loop.
    pub lateness: Option<Histogram>,
    pub cost: Cost,
    /// Reads whose values were not ones the workload could have written, and reads checked.
    pub bad: u64,
    pub reads: u64,
    pub outcome: Outcome,
}

impl WorkloadRun {
    /// The final line of one operation.
    #[must_use]
    pub fn operation(&self, op: &str) -> Option<&Final> {
        self.finals.iter().find(|final_| final_.op == op)
    }
}

/// What checking a run found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checked {
    /// Whether the run's numbers may be published.
    pub passed: bool,
    /// The sentences saying what was wrong, empty when nothing was.
    pub problems: Vec<String>,
}

/// A workload the harness runs for a duration, the other half of the harness beside `Suite`.
pub trait Workload {
    /// What the report calls it.
    fn name(&self) -> &str;

    /// Get the database for `backend` ready under `at`, and say where it is.
    ///
    /// # Errors
    ///
    /// When the directory cannot be made.
    fn prepare(&mut self, backend: &str, at: &Path) -> Result<Prepared, BenchError>;

    /// Run one plan and read what came back. A run that failed is a [`WorkloadRun`] with a failed
    /// outcome, and an error is for when there is nothing to read at all.
    ///
    /// # Errors
    ///
    /// When the driver could not be started.
    fn run(&mut self, plan: &RunPlan) -> Result<WorkloadRun, BenchError>;

    /// Whether a run is one whose numbers may be published.
    ///
    /// # Errors
    ///
    /// When the check itself could not be made.
    fn check(&mut self, run: &WorkloadRun) -> Result<Checked, BenchError>;
}

/// The value of `name=` among the words of a line.
fn field<'a>(words: &[&'a str], name: &str) -> Option<&'a str> {
    words.iter().find_map(|word| word.strip_prefix(name)?.strip_prefix('='))
}

fn number<T: std::str::FromStr>(words: &[&str], name: &str, line: &str) -> Result<T, String> {
    field(words, name)
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| format!("the line {line:?} has no number {name}="))
}

fn seconds(words: &[&str], name: &str, line: &str) -> Result<Duration, String> {
    let value: f64 = number(words, name, line)?;
    Duration::try_from_secs_f64(value).map_err(|_| format!("the line {line:?} has {name}={value}"))
}

/// The histogram after `hist=`, which runs to the end of the line.
fn histogram(line: &str) -> Result<Histogram, String> {
    let (_, rest) = line.split_once(" hist=").ok_or_else(|| format!("{line:?} has no hist="))?;
    Histogram::from_line(rest)
}

/// Reads what the driver printed.
///
/// # Errors
///
/// When a line the driver prints is there and not in the form it prints it. A run that stopped
/// early is not an error here: it comes back with a failed outcome, and `stderr` is its message.
pub fn parse(stdout: &str, stderr: &str) -> Result<WorkloadRun, String> {
    let mut run = WorkloadRun {
        header: BTreeMap::new(),
        notes: Vec::new(),
        card: None,
        loaded: None,
        intervals: Vec::new(),
        finals: Vec::new(),
        lateness: None,
        cost: Cost::default(),
        bad: 0,
        reads: 0,
        outcome: Outcome::Completed,
    };
    let mut ended = false;
    let mut verified = false;
    for line in stdout.lines() {
        let words: Vec<&str> = line.split_ascii_whitespace().collect();
        match words.first().copied() {
            Some("kv") => {
                for word in &words[2..] {
                    if let Some((name, value)) = word.split_once('=') {
                        run.header.insert(name.to_string(), value.to_string());
                    }
                }
            }
            Some("note") => run.notes.push(line["note ".len()..].to_string()),
            Some("card") => {
                let card = line["card".len()..].trim();
                run.card = (card != "none").then(|| card.to_string());
            }
            Some("load") => {
                run.loaded =
                    Some((number(&words, "rows", line)?, seconds(&words, "took_s", line)?));
            }
            Some("interval") => {
                let second = words.get(1).and_then(|word| word.parse().ok());
                let (Some(second), Some(op)) = (second, words.get(2)) else {
                    return Err(format!("the line {line:?} is not an interval"));
                };
                run.intervals.push(Interval {
                    second,
                    op: (*op).to_string(),
                    n: number(&words, "n", line)?,
                    late: number(&words, "late1ms", line)?,
                    histogram: histogram(line)?,
                });
            }
            Some("final") if words.get(1) == Some(&"late") => run.lateness = Some(histogram(line)?),
            Some("final") => {
                let op = words.get(1).ok_or_else(|| format!("the line {line:?} has no op"))?;
                run.finals.push(Final {
                    op: (*op).to_string(),
                    n: number(&words, "n", line)?,
                    retries: number(&words, "retries", line)?,
                    failed: number(&words, "failed", line)?,
                    histogram: histogram(line)?,
                });
            }
            Some("cost") => {
                run.cost = Cost {
                    user: seconds(&words, "cpu_user_s", line)?,
                    system: seconds(&words, "cpu_sys_s", line)?,
                    peak_rss: number(&words, "peak_rss", line)?,
                    written: field(&words, "written").and_then(|value| value.parse().ok()),
                };
            }
            Some("verify") => {
                run.bad = number(&words, "bad", line)?;
                run.reads = number(&words, "reads", line)?;
                verified = true;
            }
            Some("end") => ended = true,
            // The summary lines are the finals read for a person, and the histograms say more.
            _ => {}
        }
    }
    if !ended || !verified {
        let said = stderr.trim();
        let message = if said.is_empty() {
            "the driver stopped before its end line and said nothing".to_string()
        } else {
            said.lines().last().unwrap_or(said).to_string()
        };
        run.outcome = Outcome::Failed { message };
    }
    Ok(run)
}

/// YCSB through the `rudb-bench-kv` driver.
#[derive(Debug, Clone)]
pub struct Kv {
    driver: PathBuf,
}

impl Kv {
    /// The driver named by `RUDB_BENCH_KV`, or the one `cargo build --release` puts in `kv/`.
    #[must_use]
    pub fn new() -> Self {
        let driver = std::env::var_os("RUDB_BENCH_KV").map_or_else(
            || Path::new(env!("CARGO_MANIFEST_DIR")).join("kv/target/release/rudb-bench-kv"),
            PathBuf::from,
        );
        Self { driver }
    }

    /// A driver somewhere else.
    #[must_use]
    pub fn at(driver: impl Into<PathBuf>) -> Self {
        Self { driver: driver.into() }
    }
}

impl Default for Kv {
    fn default() -> Self {
        Self::new()
    }
}

impl Workload for Kv {
    fn name(&self) -> &str {
        "ycsb"
    }

    fn prepare(&mut self, backend: &str, at: &Path) -> Result<Prepared, BenchError> {
        std::fs::create_dir_all(at)
            .map_err(|error| BenchError::new(format!("making {}: {error}", at.display())))?;
        let target = match backend {
            // PostgreSQL is a server, and its target is a conninfo the caller sets on the plan.
            "postgres" => String::new(),
            _ => at.join(format!("ycsb.{backend}")).display().to_string(),
        };
        Ok(Prepared { target })
    }

    fn run(&mut self, plan: &RunPlan) -> Result<WorkloadRun, BenchError> {
        let output =
            Command::new(&self.driver).args(plan.arguments()).output().map_err(|error| {
                BenchError::new(format!("starting {}: {error}", self.driver.display()))
            })?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut run = parse(&stdout, &stderr).map_err(BenchError::new)?;
        if run.outcome.measured() && !output.status.success() {
            run.outcome =
                Outcome::Failed { message: format!("the driver exited {}", output.status) };
        }
        Ok(run)
    }

    fn check(&mut self, run: &WorkloadRun) -> Result<Checked, BenchError> {
        let mut problems = Vec::new();
        if let Outcome::Failed { message } = &run.outcome {
            problems.push(format!("the run failed: {message}"));
        }
        if run.bad > 0 {
            problems
                .push(format!("{} of {} reads returned a value no write made", run.bad, run.reads));
        }
        for final_ in &run.finals {
            if final_.failed > 0 {
                problems.push(format!("{} {} operations failed", final_.failed, final_.op));
            }
            if final_.histogram.count() != final_.n {
                problems.push(format!(
                    "the {} histogram holds {} and the line says {}",
                    final_.op,
                    final_.histogram.count(),
                    final_.n
                ));
            }
        }
        if run.header.get("pinned").is_some_and(|pinned| pinned == "no") {
            problems.push("the engine is not the pinned version".to_string());
        }
        Ok(Checked { passed: problems.is_empty(), problems })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(values: &[u64]) -> String {
        let mut histogram = Histogram::new();
        for value in values {
            histogram.record(*value);
        }
        histogram.to_line()
    }

    fn output(end: bool) -> String {
        let hist = line(&[50_000, 60_000, 70_000]);
        let mut text = format!(
            "kv 1 backend=sqlite version=3.45.1 workload=read=50,update=50 records=100 clients=4 \
             mode=closed seed=0x5eed level=full warmup_s=1 window_s=5 pinned=yes\n\
             card none\n\
             load rows=100 took_s=0.250\n\
             interval 0 read n=3 late1ms=0 hist={hist}\n\
             summary read ops_per_s=0.6 p50_us=60 p99_us=70 p999_us=70 max_us=70\n\
             final read n=3 retries=1 failed=0 hist={hist}\n\
             final update n=0 retries=0 failed=0 hist={}\n\
             cost cpu_user_s=1.174 cpu_sys_s=1.969 peak_rss=9523200 written=unknown\n\
             verify values=ok bad=0 reads=3 stale=unchecked tracked=0\n",
            line(&[])
        );
        if end {
            text.push_str("end\n");
        }
        text
    }

    #[test]
    fn a_whole_run_reads_back() {
        let run = parse(&output(true), "").unwrap();
        assert_eq!(run.outcome, Outcome::Completed);
        assert_eq!(run.header["backend"], "sqlite");
        assert_eq!(run.header["workload"], "read=50,update=50");
        assert_eq!(run.card, None);
        assert_eq!(run.loaded, Some((100, Duration::from_millis(250))));
        assert_eq!(run.intervals.len(), 1);
        let read = run.operation("read").unwrap();
        assert_eq!((read.n, read.retries, read.histogram.count()), (3, 1, 3));
        assert_eq!(run.cost.written, None);
        assert_eq!(run.cost.peak_rss, 9_523_200);
        assert_eq!(run.reads, 3);
        let checked = Kv::at("unused").check(&run).unwrap();
        assert!(checked.passed, "{:?}", checked.problems);
    }

    #[test]
    fn a_run_without_its_end_line_failed() {
        let run = parse(&output(false), "BEGIN failed: database is locked\n").unwrap();
        assert_eq!(
            run.outcome,
            Outcome::Failed { message: "BEGIN failed: database is locked".to_string() }
        );
        assert!(!Kv::at("unused").check(&run).unwrap().passed);
    }

    #[test]
    fn a_bad_read_or_an_unpinned_engine_does_not_pass() {
        let text = output(true).replace("bad=0", "bad=2").replace("pinned=yes", "pinned=no");
        let checked = Kv::at("unused").check(&parse(&text, "").unwrap()).unwrap();
        assert_eq!(checked.problems.len(), 2, "{:?}", checked.problems);
    }

    #[test]
    fn a_malformed_line_is_an_error() {
        assert!(parse("interval x read n=1 late1ms=0 hist=1 0 1 1 1 0:1\nend\n", "").is_err());
        assert!(parse("final read n=1 retries=0 failed=0 hist=2 0 1 1 1 0:1\n", "").is_err());
    }

    #[test]
    fn a_plan_becomes_the_arguments_the_driver_takes() {
        let plan = RunPlan {
            backend: "null".to_string(),
            target: String::new(),
            load: true,
            records: 1000,
            clients: 2,
            mix: "read=95,update=5".to_string(),
            pace: Pace::Open { rate: 5000.0, poisson: true },
            warmup: Duration::from_secs(1),
            window: Duration::from_millis(1500),
            seed: 7,
            level: "none".to_string(),
            card: Some("card fsync_us=40".to_string()),
        };
        let words = plan.arguments().join(" ");
        assert!(words.contains("--window 1.5s"), "{words}");
        assert!(words.contains("--load"));
        assert!(words.contains("--loop open --rate 5000 --poisson"), "{words}");
        assert!(words.ends_with("--card card fsync_us=40"), "{words}");
    }
}
