//! The engines a suite is run against.
//!
//! Section 15.3 names five systems plus the Vortex-backed builds, and this trait is the seam all of
//! them go behind. There are two of them here today: a real DuckDB, which is the primary
//! comparison because it is what the compatibility claim is against, and rudb, which cannot run a
//! query yet and says so rather than being left out of the table.
//!
//! Leaving it out of the table would be the wrong shape. A benchmark harness whose own engine is a
//! special case is one where the day it starts working, the reporting path is written in a hurry
//! by somebody who wants a number.
//!
//! ## Why every run is a fresh process
//!
//! Because a buffer pool that survives between queries makes query five's number depend on query
//! four, and a suite where reordering the queries changes the answer is not measuring the engine.
//! It costs a process spawn per run, which is single digit milliseconds and is why nothing in the
//! smoke suite is smaller than tens of milliseconds. The cost is in the cold number and the hot
//! number equally, so it does not move a ratio, and it is stated in the report rather than netted
//! out, because subtracting an estimate of overhead from a measurement is how a harness starts
//! reporting what its author expected.
//!
//! It does mean hot here is page cache warm rather than buffer pool warm. That is a weaker kind of
//! hot than ClickBench's and the report says so.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use crate::memory::{Cost, Timer};

/// Something went wrong with the apparatus, as opposed to a query being slow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchError(pub String);

impl BenchError {
    /// Build one.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for BenchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for BenchError {}

/// What loading a suite's data cost.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loaded {
    /// Wall clock time to build the dataset in this engine's own format.
    pub took: Duration,
    /// What it takes on disk afterwards, in bytes.
    pub on_disk: u64,
    /// CPU seconds the load spent, where the machine will say.
    ///
    /// Separate from the wall clock because a load is the most parallel thing any of these engines
    /// does, and on a fleet of four, six and eight core machines the wall clock of a load says as
    /// much about the machine as about the engine. M1 measured rudb encoding at 5 MB/s of values a
    /// core, and that is a sentence about CPU seconds that a wall clock cannot make.
    pub cpu: Option<Duration>,
}

/// An engine a suite can be run against.
pub trait Engine {
    /// What the table calls it.
    fn name(&self) -> &str;

    /// The exact version, which rule one says appears next to every number.
    fn version(&self) -> &str;

    /// Whether this engine can run a query at all today.
    ///
    /// rudb cannot, and a harness that discovered that by timing a failure would report a very
    /// good number for a query that did nothing.
    fn can_run(&self) -> bool;

    /// Build the dataset.
    ///
    /// # Errors
    ///
    /// When the engine could not be started or a statement failed.
    fn load(&mut self, statements: &[&str]) -> Result<Loaded, BenchError>;

    /// Run one query once, and say what it cost besides time.
    ///
    /// The timing is the caller's, taken around this call, because the caller is the one that knows
    /// whether this run is the cold one.
    ///
    /// # Errors
    ///
    /// When the engine could not be started or the query failed.
    fn run(&mut self, sql: &str) -> Result<Cost, BenchError>;
}

/// A real DuckDB, driven as a subprocess.
#[derive(Debug, Clone)]
pub struct Duckdb {
    binary: PathBuf,
    version: String,
    database: PathBuf,
    timer: Result<Timer, String>,
    scratch: PathBuf,
}

impl Duckdb {
    /// Find a DuckDB and set up a database file for it under `scratch`.
    ///
    /// `RUDB_BENCH_DUCKDB` names the binary when it is set, and otherwise `duckdb` is looked up on
    /// `PATH` and resolved to a file, so the report names which of the three DuckDBs on a developer
    /// machine actually ran.
    ///
    /// # Errors
    ///
    /// When the binary is missing or does not answer `--version`.
    pub fn discover(scratch: &Path) -> Result<Self, BenchError> {
        let binary =
            std::env::var_os("RUDB_BENCH_DUCKDB").map_or_else(|| on_path("duckdb"), PathBuf::from);
        let out = Command::new(&binary).arg("--version").output().map_err(|e| {
            BenchError::new(format!(
                "cannot run {}: {e}. Set RUDB_BENCH_DUCKDB, or put a duckdb on PATH",
                binary.display()
            ))
        })?;
        if !out.status.success() {
            return Err(BenchError::new(format!("{} --version failed", binary.display())));
        }
        Ok(Self {
            binary,
            version: String::from_utf8_lossy(&out.stdout).trim().to_owned(),
            database: scratch.join("bench.duckdb"),
            timer: Timer::find(),
            scratch: scratch.to_path_buf(),
        })
    }

    /// The binary being driven, for the machine record.
    #[must_use]
    pub fn binary(&self) -> &Path {
        &self.binary
    }

    /// Run statements in one process, under the timer, and say what they cost.
    ///
    /// Under the timer even during a load, because the CPU seconds a load spends are the number
    /// that separates an engine that is slow from an engine that is single threaded, and those are
    /// different problems with different fixes.
    fn exec(&self, statements: &[&str]) -> Result<Cost, BenchError> {
        let report = self.scratch.join("load-time.txt");
        let mut command = match &self.timer {
            Ok(timer) => timer.wrap(&self.binary, &report),
            Err(_) => Command::new(&self.binary),
        };
        command.arg("-batch").arg(&self.database);
        for statement in statements {
            command.arg("-c").arg(statement);
        }
        let out = command
            .output()
            .map_err(|e| BenchError::new(format!("cannot run {}: {e}", self.binary.display())))?;
        if !out.status.success() {
            return Err(BenchError::new(String::from_utf8_lossy(&out.stderr).trim().to_owned()));
        }
        Ok(match &self.timer {
            Ok(_) => Timer::read(&report),
            Err(why) => Cost::unavailable(why.clone()),
        })
    }
}

impl Engine for Duckdb {
    fn name(&self) -> &str {
        "duckdb"
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn can_run(&self) -> bool {
        true
    }

    fn load(&mut self, statements: &[&str]) -> Result<Loaded, BenchError> {
        std::fs::create_dir_all(&self.scratch)
            .map_err(|e| BenchError::new(format!("cannot make {}: {e}", self.scratch.display())))?;
        let _ = std::fs::remove_file(&self.database);

        let start = Instant::now();
        let build = self.exec(statements)?;
        // CHECKPOINT before the clock stops, because a load that left the write ahead log to be
        // replayed later is a load whose cost has been moved into the first query. Rule five puts
        // load time next to every runtime result precisely so that trade shows up, and it cannot
        // show up if the load stops timing before the data is durable.
        let checkpoint = self.exec(&["CHECKPOINT"])?;
        let took = start.elapsed();

        let on_disk = std::fs::metadata(&self.database).map(|m| m.len()).map_err(|e| {
            BenchError::new(format!("cannot size {}: {e}", self.database.display()))
        })?;
        Ok(Loaded { took, on_disk, cpu: add(build.cpu, checkpoint.cpu) })
    }

    fn run(&mut self, sql: &str) -> Result<Cost, BenchError> {
        let report = self.scratch.join("time.txt");
        let mut command = match &self.timer {
            Ok(timer) => timer.wrap(&self.binary, &report),
            Err(_) => Command::new(&self.binary),
        };
        command.arg("-batch").arg(&self.database).arg("-c").arg(sql);

        let out = command
            .output()
            .map_err(|e| BenchError::new(format!("cannot run {}: {e}", self.binary.display())))?;
        if !out.status.success() {
            return Err(BenchError::new(String::from_utf8_lossy(&out.stderr).trim().to_owned()));
        }
        Ok(match &self.timer {
            Ok(_) => Timer::read(&report),
            Err(why) => Cost::unavailable(why.clone()),
        })
    }
}

/// Two CPU totals added, where both of them are numbers.
///
/// A load is more than one process here and its CPU seconds are the sum, but a sum that treated a
/// missing half as zero would under report by however much that half cost, which is the direction
/// that flatters. So one missing half makes the whole thing missing.
fn add(a: Option<Duration>, b: Option<Duration>) -> Option<Duration> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a + b),
        _ => None,
    }
}

/// rudb, which is in every table and cannot run anything yet.
#[derive(Debug, Clone, Default)]
pub struct Rudb;

impl Engine for Rudb {
    fn name(&self) -> &str {
        "rudb"
    }

    fn version(&self) -> &str {
        option_env!("RUDB_VERSION").unwrap_or("not built")
    }

    fn can_run(&self) -> bool {
        false
    }

    fn load(&mut self, _statements: &[&str]) -> Result<Loaded, BenchError> {
        Err(BenchError::new("rudb has no storage layer yet, see M1 in spec/17-milestones.md"))
    }

    fn run(&mut self, _sql: &str) -> Result<Cost, BenchError> {
        Err(BenchError::new("rudb has no executor yet, see M0 and M1 in spec/17-milestones.md"))
    }
}

/// Resolve a bare command name against `PATH`.
fn on_path(name: &str) -> PathBuf {
    let Some(path) = std::env::var_os("PATH") else { return PathBuf::from(name) };
    std::env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| PathBuf::from(name))
}

#[cfg(test)]
mod tests {
    use super::{Duckdb, Engine, Rudb, on_path};

    #[test]
    fn rudb_says_it_cannot_run_rather_than_failing_fast_and_looking_quick() {
        let mut rudb = Rudb;
        assert!(!rudb.can_run());
        assert!(rudb.run("SELECT 1").is_err());
        assert!(rudb.load(&["CREATE TABLE t (a INT)"]).is_err());
    }

    #[test]
    fn a_command_on_the_path_resolves_to_the_file_it_is() {
        let got = on_path("sh");
        assert!(got.is_absolute(), "{} should have resolved", got.display());
        assert!(got.is_file());
    }

    #[test]
    fn a_duckdb_on_this_machine_loads_and_runs_and_reports_a_size() {
        let scratch = std::env::temp_dir().join(format!("rudb-bench-test-{}", std::process::id()));
        let Ok(mut duckdb) = Duckdb::discover(&scratch) else {
            eprintln!("skipping, no DuckDB on this machine");
            return;
        };
        assert!(duckdb.version().starts_with('v'), "{}", duckdb.version());

        let loaded = duckdb
            .load(&["CREATE TABLE t AS SELECT i FROM range(100000) t(i)"])
            .expect("a hundred thousand integers should load");
        assert!(loaded.on_disk > 0, "a loaded database takes space");
        assert!(loaded.took.as_nanos() > 0);

        let cost = duckdb.run("SELECT count(*) FROM t").expect("counting should work");
        // Either a number or a reason. On a machine with a /usr/bin/time it is a number, and
        // asserting that here would make the test a fact about the runner rather than the harness.
        assert!(cost.peak.measured() || cost.peak.bytes().is_none());
        if let Some(cpu) = cost.cpu {
            assert!(cpu.as_secs_f64() < 600.0, "counting a hundred thousand rows took {cpu:?}");
        }

        let _ = std::fs::remove_dir_all(&scratch);
    }

    #[test]
    fn a_query_that_fails_is_an_error_and_not_a_fast_run() {
        let scratch = std::env::temp_dir().join(format!("rudb-bench-bad-{}", std::process::id()));
        let Ok(mut duckdb) = Duckdb::discover(&scratch) else {
            eprintln!("skipping, no DuckDB on this machine");
            return;
        };
        duckdb.load(&["CREATE TABLE t AS SELECT 1 AS a"]).unwrap();
        let got = duckdb.run("SELECT nope FROM t");
        assert!(got.is_err(), "a missing column should not time successfully");
        let _ = std::fs::remove_dir_all(&scratch);
    }
}
