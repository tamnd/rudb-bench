//! The engines a suite is run against.
//!
//! Section 15.3 names five systems plus the Vortex-backed builds, and this trait is the seam all of
//! them go behind. There are five here: DuckDB, which is the primary comparison because it is what
//! the compatibility claim is against, ClickHouse, which is the fastest thing on the board this
//! project's headline number is stated against, DataFusion, which is the control for whether a
//! difference is Rust or is rudb, Polars, which is the out of core comparison, and rudb, which
//! cannot run a query yet and says so rather than being left out of the table.
//!
//! Leaving rudb out of the table would be the wrong shape. A benchmark harness whose own engine is
//! a special case is one where the day it starts working, the reporting path is written in a hurry
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
//!
//! ## Why every engine is a subprocess
//!
//! Including rudb, which this harness could link against directly. Running rudb in process while
//! running DuckDB as a subprocess would hand rudb a free process start, a warm allocator and a warm
//! buffer pool on every hot run, which is between one and ten milliseconds of advantage that no
//! reader of the table would know was there.
//!
//! ## What a load is
//!
//! Each suite is a set of Parquet files and a load is whatever an engine does to turn those bytes
//! into whatever it queries. For DuckDB and ClickHouse that is a real conversion into a real
//! storage format and the on disk number means something. For DataFusion and Polars there is no
//! storage format, the load is a table definition pointing at the file, and the on disk number is
//! the source Parquet. Both are reported, and [`Loaded::on_disk_is`] says which of the two a
//! reader is looking at, because a table where 92 MiB and 92 MiB mean different things without
//! saying so is worse than one with a gap.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use crate::data::{Table, output, size_of_tree};
use crate::memory::{Cost, Timer};
use crate::suite::Suite;

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

/// Whether an engine can run a suite, and when it cannot, the sentence the table prints instead.
///
/// A reason rather than a boolean, because the useful thing to know about a missing row is which of
/// the several different missing things it is. rudb cannot run any suite yet for one reason,
/// DataFusion cannot run TPC-H for a different one, and a blank cell says neither.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ability {
    /// It can.
    Yes,
    /// It cannot, and this is why.
    No(String),
}

impl Ability {
    /// It cannot, for this reason.
    #[must_use]
    pub fn no(why: impl Into<String>) -> Self {
        Self::No(why.into())
    }

    /// Whether it can.
    #[must_use]
    pub const fn yes(&self) -> bool {
        matches!(self, Self::Yes)
    }

    /// Why not, when there is a why not.
    #[must_use]
    pub fn why(&self) -> Option<&str> {
        match self {
            Self::Yes => None,
            Self::No(why) => Some(why),
        }
    }
}

/// What loading a suite's data cost.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loaded {
    /// Wall clock time to build the dataset in this engine's own format.
    pub took: Duration,
    /// What it takes on disk afterwards, in bytes.
    pub on_disk: u64,
    /// What that on disk number is a size of.
    ///
    /// An engine with a storage format reports its own files. An engine without one reports the
    /// Parquet it reads, and a reader who does not know which is which will read the second as a
    /// compression result.
    pub on_disk_is: String,
    /// Whether the load actually converted the data into something else.
    ///
    /// The single most important thing to know about a query column, and it belongs next to the
    /// number rather than in a footnote. An engine that converted paid for the decode once, at load
    /// time, in a column a reader can see. An engine that did not is paying for it again in every
    /// query, in the column being compared, and a table that did not say so would be reporting the
    /// second engine as slow when part of what it is doing is the work the first one already did.
    pub converted: bool,
    /// CPU seconds the load spent, where the machine will say.
    ///
    /// Separate from the wall clock because a load is the most parallel thing any of these engines
    /// does, and on a fleet of four, six and eight core machines the wall clock of a load says as
    /// much about the machine as about the engine. M1 measured rudb encoding at 5 MB/s of values a
    /// core, and that is a sentence about CPU seconds that a wall clock cannot make.
    pub cpu: Option<Duration>,
}

/// One run of one query: what it cost, and what it answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ran {
    /// Peak, CPU and bytes read.
    pub cost: Cost,
    /// Whatever the engine printed, kept so that two engines answering differently is visible.
    pub answer: String,
}

/// An engine a suite can be run against.
pub trait Engine {
    /// What the table calls it.
    fn name(&self) -> &str;

    /// The exact version, which rule one says appears next to every number.
    fn version(&self) -> &str;

    /// Whether this engine can run this suite, and when not, why not.
    ///
    /// Per suite rather than a constant, because the answer is per suite. rudb will be able to run
    /// `smoke` a long time before it can run ClickBench, and an engine that reported one answer for
    /// both would either be left out of a table it belongs in or timed on a failure.
    fn can_run(&self, suite: &Suite) -> Ability;

    /// Build the dataset from the suite's Parquet files.
    ///
    /// # Errors
    ///
    /// When the engine could not be started or a statement failed.
    fn load(&mut self, tables: &[Table]) -> Result<Loaded, BenchError>;

    /// Run one query once, and say what it cost and what it answered.
    ///
    /// The timing is the caller's, taken around this call, because the caller is the one that knows
    /// whether this run is the cold one.
    ///
    /// # Errors
    ///
    /// When the engine could not be started or the query failed.
    fn run(&mut self, sql: &str) -> Result<Ran, BenchError>;
}

/// A subprocess under the timer, which is what all five of these are.
///
/// The engines differ in their arguments and in their dialect and in nothing else, so this holds
/// the part that is the same: find the timer once, wrap the command in it, run it, read the report
/// back. An engine on a machine with no `/usr/bin/time` still runs and still answers, and its cost
/// is a reason rather than a number.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Runner {
    timer: Result<Timer, String>,
    report: PathBuf,
}

impl Runner {
    fn new(scratch: &Path, name: &str) -> Self {
        Self { timer: Timer::find(), report: scratch.join(format!("{name}-time.txt")) }
    }

    /// A command that will be timed, or a plain one when there is no timer here.
    fn command(&self, program: &Path) -> Command {
        match &self.timer {
            Ok(timer) => timer.wrap(program, &self.report),
            Err(_) => Command::new(program),
        }
    }

    /// Run it, and read back what it cost and what it printed.
    fn go(&self, mut command: Command, what: &str) -> Result<Ran, BenchError> {
        let stdout = output(&mut command, what)?;
        let cost = match &self.timer {
            Ok(_) => Timer::read(&self.report),
            Err(why) => Cost::unavailable(why.clone()),
        };
        Ok(Ran { cost, answer: String::from_utf8_lossy(&stdout).trim().to_owned() })
    }
}

/// A real DuckDB, driven as a subprocess.
#[derive(Debug, Clone)]
pub struct Duckdb {
    binary: PathBuf,
    version: String,
    database: PathBuf,
    runner: Runner,
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
        let version = version_of(&binary, &["--version"], "RUDB_BENCH_DUCKDB")?;
        make(scratch)?;
        Ok(Self {
            binary,
            version,
            database: scratch.join("bench.duckdb"),
            runner: Runner::new(scratch, "duckdb"),
        })
    }

    /// The binary being driven, for the machine record.
    #[must_use]
    pub fn binary(&self) -> &Path {
        &self.binary
    }

    /// Run statements with no database and no timer, for the jobs that are apparatus rather than
    /// measurement, like generating the smoke Parquet.
    ///
    /// # Errors
    ///
    /// When DuckDB could not be started or a statement failed.
    pub fn plain(&self, statements: &[&str]) -> Result<(), BenchError> {
        let mut command = Command::new(&self.binary);
        command.arg("-batch");
        for statement in statements {
            command.arg("-c").arg(statement);
        }
        output(&mut command, "duckdb")?;
        Ok(())
    }

    /// Run statements against the database, under the timer.
    fn exec(&self, statements: &[&str]) -> Result<Ran, BenchError> {
        let mut command = self.runner.command(&self.binary);
        // CSV with no header, which is what every engine here is asked for, so that four answers
        // can be compared without four output parsers. See [`crate::answer`].
        command.arg("-batch").arg("-csv").arg("-noheader").arg(&self.database);
        for statement in statements {
            command.arg("-c").arg(statement);
        }
        self.runner.go(command, "duckdb")
    }
}

impl Engine for Duckdb {
    fn name(&self) -> &str {
        "duckdb"
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn can_run(&self, _suite: &Suite) -> Ability {
        Ability::Yes
    }

    fn load(&mut self, tables: &[Table]) -> Result<Loaded, BenchError> {
        let _ = std::fs::remove_file(&self.database);
        let statements: Vec<String> = tables
            .iter()
            .map(|t| {
                format!(
                    "CREATE TABLE {} AS SELECT * FROM read_parquet('{}')",
                    t.name,
                    t.path.display()
                )
            })
            .collect();
        let refs: Vec<&str> = statements.iter().map(String::as_str).collect();

        let start = Instant::now();
        let build = self.exec(&refs)?;
        // CHECKPOINT before the clock stops, because a load that left the write ahead log to be
        // replayed later is a load whose cost has been moved into the first query. Rule five puts
        // load time next to every runtime result precisely so that trade shows up, and it cannot
        // show up if the load stops timing before the data is durable.
        let checkpoint = self.exec(&["CHECKPOINT"])?;
        let took = start.elapsed();

        let on_disk = std::fs::metadata(&self.database).map(|m| m.len()).map_err(|e| {
            BenchError::new(format!("cannot size {}: {e}", self.database.display()))
        })?;
        Ok(Loaded {
            took,
            on_disk,
            on_disk_is: "its own database file".to_owned(),
            converted: true,
            cpu: add(build.cost.cpu, checkpoint.cost.cpu),
        })
    }

    fn run(&mut self, sql: &str) -> Result<Ran, BenchError> {
        self.exec(&[sql])
    }
}

/// ClickHouse, run as `clickhouse local` against a directory it keeps its own data in.
///
/// `local` and not a server, which is a real difference and not a convenience. The server has a
/// page cache of its own that survives between queries and `local` does not, so this row is the
/// one that is comparable to the other four here, all of which are also a fresh process per query.
/// The tuned server row that section 15.3 asks for is a second row and not this one.
#[derive(Debug, Clone)]
pub struct ClickhouseLocal {
    binary: PathBuf,
    version: String,
    data: PathBuf,
    runner: Runner,
}

impl ClickhouseLocal {
    /// Find a ClickHouse and give it a directory.
    ///
    /// # Errors
    ///
    /// When the binary is missing or does not answer a version query.
    pub fn discover(scratch: &Path) -> Result<Self, BenchError> {
        let binary = std::env::var_os("RUDB_BENCH_CLICKHOUSE")
            .map_or_else(|| on_path("clickhouse"), PathBuf::from);
        let version = version_of(
            &binary,
            &["local", "--query", "SELECT version()"],
            "RUDB_BENCH_CLICKHOUSE",
        )?;
        make(scratch)?;
        Ok(Self {
            binary,
            version,
            data: scratch.join("clickhouse"),
            runner: Runner::new(scratch, "clickhouse"),
        })
    }

    fn exec(&self, sql: &str) -> Result<Ran, BenchError> {
        let mut command = self.runner.command(&self.binary);
        command
            .arg("local")
            .arg("--path")
            .arg(&self.data)
            .arg("--format")
            .arg("CSV")
            .arg("--query")
            .arg(sql);
        self.runner.go(command, "clickhouse local")
    }
}

impl Engine for ClickhouseLocal {
    fn name(&self) -> &str {
        "clickhouse-local"
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn can_run(&self, _suite: &Suite) -> Ability {
        Ability::Yes
    }

    fn load(&mut self, tables: &[Table]) -> Result<Loaded, BenchError> {
        let _ = std::fs::remove_dir_all(&self.data);
        std::fs::create_dir_all(&self.data)
            .map_err(|e| BenchError::new(format!("cannot make {}: {e}", self.data.display())))?;

        let start = Instant::now();
        let mut cpu = Some(Duration::ZERO);
        for table in tables {
            // ORDER BY tuple() means no sorting key, which is the honest default for a comparison
            // where nobody else was given one either. ClickBench's own create.sql picks a sorting
            // key and that belongs to the ClickBench row, where every engine gets the tuning its
            // published result was measured with.
            let ran = self.exec(&format!(
                "CREATE TABLE {} ENGINE = MergeTree ORDER BY tuple() AS SELECT * FROM file('{}', \
                 Parquet)",
                table.name,
                table.path.display()
            ))?;
            cpu = add(cpu, ran.cost.cpu);
        }
        let took = start.elapsed();
        Ok(Loaded {
            took,
            on_disk: size_of_tree(&self.data),
            on_disk_is: "its own MergeTree parts".to_owned(),
            converted: true,
            cpu,
        })
    }

    fn run(&mut self, sql: &str) -> Result<Ran, BenchError> {
        self.exec(sql)
    }
}

/// DataFusion through its own command line.
///
/// The control for the question the other rows cannot answer. DuckDB and ClickHouse are C++, so a
/// difference against them is a difference between two projects and two languages at once.
/// DataFusion is Rust with an Arrow memory format and a vectorized executor, which is the same set
/// of decisions rudb made, so a gap against DataFusion is a gap in rudb and not in the language.
#[derive(Debug, Clone)]
pub struct Datafusion {
    binary: PathBuf,
    version: String,
    ddl: Vec<String>,
    source_bytes: u64,
    runner: Runner,
}

impl Datafusion {
    /// Find a `datafusion-cli`.
    ///
    /// # Errors
    ///
    /// When the binary is missing or does not answer `--version`.
    pub fn discover(scratch: &Path) -> Result<Self, BenchError> {
        let binary = std::env::var_os("RUDB_BENCH_DATAFUSION")
            .map_or_else(|| on_path("datafusion-cli"), PathBuf::from);
        let version = version_of(&binary, &["--version"], "RUDB_BENCH_DATAFUSION")?;
        make(scratch)?;
        Ok(Self {
            binary,
            version,
            ddl: Vec::new(),
            source_bytes: 0,
            runner: Runner::new(scratch, "datafusion"),
        })
    }
}

impl Engine for Datafusion {
    fn name(&self) -> &str {
        "datafusion"
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn can_run(&self, _suite: &Suite) -> Ability {
        Ability::Yes
    }

    fn load(&mut self, tables: &[Table]) -> Result<Loaded, BenchError> {
        // Nothing is converted, so nothing is timed. The table definitions are kept and replayed in
        // front of every query, because `datafusion-cli` has no catalog that outlives a process and
        // this harness starts a fresh one per run on purpose.
        self.ddl = tables
            .iter()
            .map(|t| {
                format!(
                    "CREATE EXTERNAL TABLE {} STORED AS PARQUET LOCATION '{}'",
                    t.name,
                    t.path.display()
                )
            })
            .collect();
        self.source_bytes = tables.iter().map(|t| t.bytes).sum();
        Ok(Loaded {
            took: Duration::ZERO,
            on_disk: self.source_bytes,
            on_disk_is: "the source Parquet, this engine has no storage format of its own"
                .to_owned(),
            converted: false,
            cpu: Some(Duration::ZERO),
        })
    }

    fn run(&mut self, sql: &str) -> Result<Ran, BenchError> {
        let mut command = self.runner.command(&self.binary);
        // Quiet, because without it every result is followed by a line saying how many rows were
        // fetched and how long it took, and those digits land in the answer comparison as data.
        command.arg("-q").arg("--format").arg("csv");
        for statement in &self.ddl {
            command.arg("-c").arg(statement);
        }
        command.arg("-c").arg(sql);
        self.runner.go(command, "datafusion-cli")
    }
}

/// Polars, driven through a Python that has it installed.
///
/// Through Python because that is where Polars is, and a comparison that left it out because its
/// natural interface is not a SQL shell would be leaving out the engine most likely to be the fast
/// one on the out of core suites. The query goes through `SQLContext` over lazy frames and the
/// collect is the streaming engine, which is the mode section 15.3 asks for.
#[derive(Debug, Clone)]
pub struct Polars {
    python: PathBuf,
    version: String,
    tables: Vec<Table>,
    runner: Runner,
    script: PathBuf,
}

impl Polars {
    /// Find a Python with Polars in it.
    ///
    /// # Errors
    ///
    /// When there is no Python, or when the Python there is cannot import Polars.
    pub fn discover(scratch: &Path) -> Result<Self, BenchError> {
        let python =
            std::env::var_os("RUDB_BENCH_PYTHON").map_or_else(|| on_path("python3"), PathBuf::from);
        let version = version_of(
            &python,
            &["-c", "import polars; print(polars.__version__)"],
            "RUDB_BENCH_PYTHON",
        )?;
        make(scratch)?;
        Ok(Self {
            python,
            version,
            tables: Vec::new(),
            runner: Runner::new(scratch, "polars"),
            script: scratch.join("polars-run.py"),
        })
    }
}

/// What the Polars subprocess runs.
///
/// A file rather than `-c`, because the argument list of a run is already the timer, the
/// interpreter and the query, and putting a program in there too makes a failure impossible to
/// reproduce by hand from the log.
const POLARS_SCRIPT: &str = r#"import sys
import polars as pl

sql = sys.argv[1]
ctx = pl.SQLContext()
for pair in sys.argv[2:]:
    name, path = pair.split("=", 1)
    ctx.register(name, pl.scan_parquet(path))
frame = ctx.execute(sql).collect(engine="streaming")
sys.stdout.write(frame.write_csv(include_header=False))
"#;

impl Engine for Polars {
    fn name(&self) -> &str {
        "polars"
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn can_run(&self, _suite: &Suite) -> Ability {
        Ability::Yes
    }

    fn load(&mut self, tables: &[Table]) -> Result<Loaded, BenchError> {
        std::fs::write(&self.script, POLARS_SCRIPT)
            .map_err(|e| BenchError::new(format!("cannot write {}: {e}", self.script.display())))?;
        self.tables = tables.to_vec();
        Ok(Loaded {
            took: Duration::ZERO,
            on_disk: tables.iter().map(|t| t.bytes).sum(),
            on_disk_is: "the source Parquet, this engine has no storage format of its own"
                .to_owned(),
            converted: false,
            cpu: Some(Duration::ZERO),
        })
    }

    fn run(&mut self, sql: &str) -> Result<Ran, BenchError> {
        let mut command = self.runner.command(&self.python);
        command.arg(&self.script).arg(sql);
        for table in &self.tables {
            command.arg(format!("{}={}", table.name, table.path.display()));
        }
        self.runner.go(command, "polars")
    }
}

/// rudb, driven through `rudb-cli` the same way everything else here is driven.
///
/// It abstains rather than losing, which is a distinction worth keeping. There is no path from a
/// file on disk into a chunk yet, so a rudb row on any of these suites would be a timing of an
/// error message. When that path exists, [`Engine::can_run`] is the only thing here that changes.
#[derive(Debug, Clone)]
pub struct Rudb {
    binary: Option<PathBuf>,
    version: String,
    runner: Runner,
}

impl Rudb {
    /// Find a `rudb` command line, and carry on without one.
    ///
    /// Not an error when it is missing, because rudb is in the table either way and the difference
    /// between a rudb that is not built and a rudb that cannot read a Parquet file is a difference
    /// the report should be able to state.
    #[must_use]
    pub fn discover(scratch: &Path) -> Self {
        let binary =
            std::env::var_os("RUDB_BENCH_RUDB").map_or_else(|| on_path("rudb"), PathBuf::from);
        match version_of(&binary, &["--version"], "RUDB_BENCH_RUDB") {
            Ok(version) => {
                Self { binary: Some(binary), version, runner: Runner::new(scratch, "rudb") }
            }
            Err(_) => Self {
                binary: None,
                version: "not built".to_owned(),
                runner: Runner::new(scratch, "rudb"),
            },
        }
    }
}

impl Engine for Rudb {
    fn name(&self) -> &str {
        "rudb"
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn can_run(&self, suite: &Suite) -> Ability {
        match &self.binary {
            None => Ability::no("no rudb on PATH, set RUDB_BENCH_RUDB"),
            Some(_) => Ability::no(format!(
                "no path from a file on disk into a chunk yet, so {} would be a timing of an error \
                 message. spec/engine/05-scan.md, sub-milestone 2e",
                suite.name
            )),
        }
    }

    fn load(&mut self, _tables: &[Table]) -> Result<Loaded, BenchError> {
        Err(BenchError::new("rudb has no storage layer yet, see M1 in spec/17-milestones.md"))
    }

    fn run(&mut self, sql: &str) -> Result<Ran, BenchError> {
        let Some(binary) = self.binary.clone() else {
            return Err(BenchError::new("there is no rudb to run"));
        };
        let mut command = self.runner.command(&binary);
        command.arg("-c").arg(sql);
        self.runner.go(command, "rudb")
    }
}

/// Ask a binary what it is, and turn every way that can fail into one sentence naming the override.
fn version_of(binary: &Path, args: &[&str], env: &str) -> Result<String, BenchError> {
    let out = Command::new(binary).args(args).output().map_err(|e| {
        BenchError::new(format!("cannot run {}: {e}. Set {env} to a binary", binary.display()))
    })?;
    if !out.status.success() {
        return Err(BenchError::new(format!(
            "{} is there but did not answer a version query. Set {env} to one that does",
            binary.display()
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

/// Make a directory, with the failure named after the directory rather than after the syscall.
fn make(path: &Path) -> Result<(), BenchError> {
    std::fs::create_dir_all(path)
        .map_err(|e| BenchError::new(format!("cannot make {}: {e}", path.display())))
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
    use super::{Ability, Duckdb, Engine, Rudb, on_path};
    use crate::data::Table;
    use crate::suite::find;

    fn scratch(what: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("rudb-bench-{what}-{}", std::process::id()))
    }

    #[test]
    fn rudb_abstains_with_a_reason_rather_than_failing_fast_and_looking_quick() {
        let scratch = scratch("rudb");
        let mut rudb = Rudb::discover(&scratch);
        let smoke = find("smoke").unwrap();
        assert!(!rudb.can_run(smoke).yes());
        assert!(rudb.can_run(smoke).why().is_some_and(|why| !why.is_empty()));
        assert!(rudb.load(&[]).is_err());
        let _ = std::fs::remove_dir_all(&scratch);
    }

    #[test]
    fn an_ability_that_is_yes_has_nothing_to_explain() {
        assert!(Ability::Yes.yes());
        assert_eq!(Ability::Yes.why(), None);
        assert_eq!(Ability::no("because").why(), Some("because"));
    }

    #[test]
    fn a_command_on_the_path_resolves_to_the_file_it_is() {
        let got = on_path("sh");
        assert!(got.is_absolute(), "{} should have resolved", got.display());
        assert!(got.is_file());
    }

    #[test]
    fn a_duckdb_on_this_machine_loads_a_parquet_and_runs_and_reports_a_size() {
        let scratch = scratch("duck");
        let Ok(mut duckdb) = Duckdb::discover(&scratch) else {
            eprintln!("skipping, no DuckDB on this machine");
            return;
        };
        assert!(duckdb.version().starts_with('v'), "{}", duckdb.version());

        let parquet = scratch.join("t.parquet");
        duckdb
            .plain(&[&format!(
                "COPY (SELECT i FROM range(100000) t(i)) TO '{}' (FORMAT parquet)",
                parquet.display()
            )])
            .expect("writing a parquet should work");
        let bytes = std::fs::metadata(&parquet).unwrap().len();
        let tables = vec![Table { name: "t".to_owned(), path: parquet, bytes }];

        let loaded = duckdb.load(&tables).expect("a hundred thousand integers should load");
        assert!(loaded.on_disk > 0, "a loaded database takes space");
        assert!(loaded.took.as_nanos() > 0);
        assert!(loaded.on_disk_is.contains("database file"), "{}", loaded.on_disk_is);

        let ran = duckdb.run("SELECT count(*) FROM t").expect("counting should work");
        assert!(ran.answer.contains("100000"), "{}", ran.answer);
        // Either a number or a reason. On a machine with a /usr/bin/time it is a number, and
        // asserting that here would make the test a fact about the runner rather than the harness.
        assert!(ran.cost.peak.measured() || ran.cost.peak.bytes().is_none());

        let _ = std::fs::remove_dir_all(&scratch);
    }

    #[test]
    fn a_query_that_fails_is_an_error_and_not_a_fast_run() {
        let scratch = scratch("bad");
        let Ok(mut duckdb) = Duckdb::discover(&scratch) else {
            eprintln!("skipping, no DuckDB on this machine");
            return;
        };
        let got = duckdb.run("SELECT nope FROM nothing");
        assert!(got.is_err(), "a missing table should not time successfully");
        let _ = std::fs::remove_dir_all(&scratch);
    }
}
