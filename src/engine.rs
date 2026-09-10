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
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::data::{Table, output, size_of_tree};
use crate::memory::{Cost, Timer};
use crate::suite::{Fixup, Loading, Suite, loading, sorting_key};

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

    /// Whether anything this engine learned in one query is still there for the next one.
    ///
    /// False for everything driven as a fresh process per run, which is four of the five here and
    /// is a decision rather than a limitation: a buffer pool that survives between queries makes
    /// query five's number depend on query four. True for the tuned ClickHouse server, whose whole
    /// point is that it does not restart. The report reads this to decide which sentence to print
    /// about what hot means, because the two rows are hot in two different senses and one sentence
    /// covering both would be false about one of them.
    fn keeps_state(&self) -> bool {
        false
    }

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

    /// Give back the disk this engine's copy of the data is taking, now that it is measured.
    ///
    /// The default does nothing, which is right for an engine that reads the Parquet where it lies
    /// and never made a copy. The three that convert override it.
    ///
    /// This exists because the comparison runs one engine to completion before starting the next,
    /// so nothing needs two copies to be on disk at once, and a harness that kept all of them
    /// needs the sum of every engine's format free rather than the largest one. On ClickBench that
    /// is the difference between about 45 GB and about 16 GB, which is the difference between the
    /// suite running on the quiet machine and running on the loud one.
    ///
    /// Everything the report says about size was read at load time and is already in [`Loaded`],
    /// so this takes nothing away from the table. What it does take away is the ability to go and
    /// look at an engine's data after a run, which is a debugging convenience rather than a
    /// measurement, and `RUDB_BENCH_KEEP` turns it off for the afternoon somebody needs that.
    fn unload(&mut self) {}
}

/// Whether the run was asked to leave every engine's copy of the data behind.
///
/// Off by default. On, the suite needs the sum of every engine's on disk size free rather than the
/// largest one, which is a lot of disk on ClickBench and none at all on smoke.
#[must_use]
pub fn keeping() -> bool {
    std::env::var_os("RUDB_BENCH_KEEP").is_some()
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
    /// Which suite is running, so the table can be built the way this engine's own entry builds it.
    suite: &'static str,
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
    pub fn discover(scratch: &Path, suite: &'static Suite) -> Result<Self, BenchError> {
        let binary =
            std::env::var_os("RUDB_BENCH_DUCKDB").map_or_else(|| on_path("duckdb"), PathBuf::from);
        let version = version_of(&binary, &["--version"], "RUDB_BENCH_DUCKDB")?;
        make(scratch)?;
        Ok(Self {
            binary,
            version,
            database: scratch.join("bench.duckdb"),
            runner: Runner::new(scratch, "duckdb"),
            suite: suite.name,
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
        // CSV with no header, which is what three of the four engines here are asked for so that
        // their answers can be compared without a parser each. The comparison itself reads numbers
        // out of whatever text comes back, which is what lets the fourth one print a bordered table
        // for a reason of its own. See [`crate::answer`] and the note on the datafusion run.
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
        // The engine's own entry when the board publishes one, and a plain CTAS when it does not.
        // The plain CTAS is right for a suite nobody published a schema for and wrong for
        // ClickBench, where it leaves EventTime an integer and q19's extract(minute FROM EventTime)
        // has nothing to resolve against. That is not a hypothetical: it is how the first full
        // ClickBench run on gamingpc-wsl produced no DuckDB column at all.
        let mut statements: Vec<String> = Vec::with_capacity(tables.len() * 2);
        for t in tables {
            match loading(self.suite, "duckdb", &t.name) {
                Some(Loading {
                    columns: Some(schema),
                    fixup: Fixup::Select(select),
                    options,
                    ..
                }) => {
                    statements.push(format!("CREATE TABLE {} ({schema})", t.name));
                    statements.push(format!(
                        "INSERT INTO {} SELECT {select} FROM read_parquet('{}', {options})",
                        t.name,
                        t.path.display()
                    ));
                }
                _ => statements.push(format!(
                    "CREATE TABLE {} AS SELECT * FROM read_parquet('{}')",
                    t.name,
                    t.path.display()
                )),
            }
        }
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

    fn unload(&mut self) {
        if keeping() {
            return;
        }
        let _ = std::fs::remove_file(&self.database);
        // The write ahead log too. A database file removed on its own leaves a .wal beside it that
        // is most of the data on a load this size.
        let _ = std::fs::remove_file(self.database.with_extension("duckdb.wal"));
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
    /// Which suite is running, so the table can be created with that suite's published schema.
    suite: &'static str,
}

impl ClickhouseLocal {
    /// Find a ClickHouse and give it a directory.
    ///
    /// # Errors
    ///
    /// When the binary is missing or does not answer a version query.
    pub fn discover(scratch: &Path, suite: &'static Suite) -> Result<Self, BenchError> {
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
            suite: suite.name,
        })
    }

    /// What ClickHouse says its own active parts take.
    ///
    /// Asked rather than measured with `stat`, for a reason that only shows up once a table has
    /// been merged. `OPTIMIZE ... FINAL` writes a new part and leaves the parts it replaced on disk
    /// as inactive until a background thread gets to them eight minutes later, so a directory size
    /// taken straight afterwards is the data counted roughly twice. The tuned server row measured
    /// 128 MiB that way against `clickhouse local` at 49 MiB for the same ten million rows, which
    /// reads as the sorting key having cost 2.6x on disk and is not what happened.
    ///
    /// `None` rather than zero when the answer does not parse, so the caller falls back to the
    /// directory rather than publishing a table with no bytes in it.
    fn parts_bytes(&self) -> Option<u64> {
        parts_bytes(&self.exec(PARTS).ok()?.answer)
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
            // key and that belongs to the tuned server row, which is the one row here that gets it.
            //
            // The column types are a different question and this row does get those, because an
            // inferred schema is a wrong schema rather than an untuned one. See `suite::HITS`.
            let ran = match loading(self.suite, self.name(), &table.name).and_then(|l| l.columns) {
                Some(schema) => {
                    let create = format!(
                        "CREATE TABLE {} ({schema}) ENGINE = MergeTree ORDER BY tuple()",
                        table.name
                    );
                    let insert = format!(
                        "INSERT INTO {} SELECT * FROM file('{}', Parquet)",
                        table.name,
                        table.path.display()
                    );
                    let made = self.exec(&create)?;
                    cpu = add(cpu, made.cost.cpu);
                    self.exec(&insert)?
                }
                None => self.exec(&format!(
                    "CREATE TABLE {} ENGINE = MergeTree ORDER BY tuple() AS SELECT * FROM \
                     file('{}', Parquet)",
                    table.name,
                    table.path.display()
                ))?,
            };
            cpu = add(cpu, ran.cost.cpu);
        }
        let took = start.elapsed();
        Ok(Loaded {
            took,
            on_disk: self.parts_bytes().unwrap_or_else(|| size_of_tree(&self.data)),
            on_disk_is: "its own MergeTree parts, as system.parts counts the active ones"
                .to_owned(),
            converted: true,
            cpu,
        })
    }

    fn run(&mut self, sql: &str) -> Result<Ran, BenchError> {
        self.exec(sql)
    }

    fn unload(&mut self) {
        if keeping() {
            return;
        }
        let _ = std::fs::remove_dir_all(&self.data);
    }
}

/// ClickHouse as a real server, with the sorting key its own `create.sql` gives the table.
///
/// The second of the two ClickHouse rows, and it is a second row rather than a flag on the first
/// because it is a different system in two ways that both matter.
///
/// It keeps its caches. A server holds a mark cache and a primary key index in memory between
/// queries and a fresh `clickhouse local` does not, so this row's hot number is hot in the sense
/// ClickBench means and the other four rows in the table are hot only in the sense that the page
/// cache is warm. That is the stronger kind of hot and it is not comparable to the weaker kind, so
/// putting the two ClickHouses in one row would be averaging two different measurements.
///
/// It gets a sorting key, from [`crate::suite::sorting_key`]. Nothing else in this table gets one.
/// That is the point: a ClickHouse deployed without a sorting key is not the ClickHouse this
/// project's headline claim is stated against, and a comparison that only ever measured the untuned
/// one would be a comparison this project wins by picking the fight.
///
/// ## What it cannot tell you
///
/// Peak resident, CPU seconds and bytes read, and it says so per query rather than reporting a
/// number that is not the one it appears to be. `/usr/bin/time` measures the process it started,
/// and the process it would start here is `clickhouse client`, which parses a result set and prints
/// it. The work happened in a server this harness started minutes earlier and the timer never saw.
/// A peak of forty megabytes next to DuckDB's four hundred would be read as this engine winning on
/// memory, when it is the client's memory and the server is not in the column at all.
///
/// The server's own high water mark is readable, from `/proc` or from `system.asynchronous_metrics`,
/// and it is a high water mark over the whole run rather than over one query, so it does not belong
/// in a per query cell either. Rule six then says this row is not publishable, which is the correct
/// answer for a row whose hot number is a different kind of hot anyway.
#[derive(Debug)]
pub struct ClickhouseServer {
    binary: PathBuf,
    version: String,
    suite: &'static str,
    dir: PathBuf,
    port: u16,
    server: Option<Child>,
    sorted_by: String,
}

/// Why the cost columns of the tuned server row are a sentence instead of a number.
const NOT_THE_SERVER: &str =
    "the timer wraps clickhouse client and the work happens in a server process it did not start";

impl ClickhouseServer {
    /// Find a ClickHouse, pick a port for it, and give it a directory. Nothing starts yet.
    ///
    /// Deliberately nothing starts yet. A ClickHouse server resident while DuckDB is being timed is
    /// half a gigabyte of somebody else's memory and a page cache somebody else is evicting, so the
    /// server comes up inside [`Engine::load`], which runs after every other engine in the table has
    /// already finished. That ordering is in `discover` in the binary and it is a measurement
    /// decision rather than a tidiness one.
    ///
    /// # Errors
    ///
    /// When the binary is missing, when it does not answer a version query, or when there is no
    /// free port on the loopback interface.
    pub fn discover(scratch: &Path, suite: &'static Suite) -> Result<Self, BenchError> {
        let binary = std::env::var_os("RUDB_BENCH_CLICKHOUSE")
            .map_or_else(|| on_path("clickhouse"), PathBuf::from);
        let version = version_of(
            &binary,
            &["local", "--query", "SELECT version()"],
            "RUDB_BENCH_CLICKHOUSE",
        )?;
        let dir = scratch.join("clickhouse-server");
        make(&dir)?;
        Ok(Self {
            binary,
            version,
            suite: suite.name,
            dir,
            port: free_port()?,
            server: None,
            sorted_by: String::new(),
        })
    }

    /// Bring the server up and wait until it answers.
    ///
    /// The wait is a query rather than a sleep, because a sleep long enough to be safe on a loaded
    /// machine is long enough to be irritating on an idle one, and a sleep short enough to be
    /// pleasant is a flaky suite. A server that died on the way up is reported as having died, with
    /// the path to the log it wrote, rather than as sixty seconds of a query that never connected.
    fn start(&mut self) -> Result<(), BenchError> {
        if self.server.is_some() {
            return Ok(());
        }
        let config = self.dir.join("config.xml");
        let users = self.dir.join("users.xml");
        write(&users, USERS)?;
        write(
            &config,
            &CONFIG
                .replace("{dir}", &self.dir.display().to_string())
                .replace("{users}", &users.display().to_string())
                .replace("{port}", &self.port.to_string()),
        )?;

        let child = Command::new(&self.binary)
            .arg("server")
            .arg("--config-file")
            .arg(&config)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                BenchError::new(format!("cannot start {} server: {e}", self.binary.display()))
            })?;
        self.server = Some(child);

        let log = self.dir.join("server.err.log");
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            if self.exec("SELECT 1").is_ok() {
                return Ok(());
            }
            // Not a let chain, because the minimum supported Rust here is 1.85 and those landed in
            // 1.88.
            if let Some(child) = self.server.as_mut() {
                if let Ok(Some(status)) = child.try_wait() {
                    return Err(BenchError::new(format!(
                        "the clickhouse server stopped before it answered, {status}. It wrote {}",
                        log.display()
                    )));
                }
            }
            if Instant::now() > deadline {
                return Err(BenchError::new(format!(
                    "the clickhouse server did not answer on port {} within a minute. It wrote {}",
                    self.port,
                    log.display()
                )));
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    }

    /// A client pointed at our server and nobody else's.
    ///
    /// The host is spelled out rather than left to default, because a default that resolved to a
    /// ClickHouse somebody already had running on this machine would produce a table of real
    /// numbers from the wrong server.
    fn client(&self) -> Command {
        let mut command = Command::new(&self.binary);
        command
            .arg("client")
            .arg("--host")
            .arg("127.0.0.1")
            .arg("--port")
            .arg(self.port.to_string());
        command
    }

    /// Run one statement and hand back what it printed.
    fn exec(&self, sql: &str) -> Result<String, BenchError> {
        let mut command = self.client();
        command.arg("--format").arg("CSV").arg("--query").arg(sql);
        let out = output(&mut command, "clickhouse client")?;
        Ok(String::from_utf8_lossy(&out).trim().to_owned())
    }

    /// The column list a Parquet file implies, as ClickHouse would write it.
    ///
    /// Read out of the file rather than declared, because this harness runs suites whose schemas it
    /// does not have, and inferred with `schema_inference_make_columns_nullable = 0`, because a
    /// column ClickHouse decided was `Nullable(Int64)` carries a null map through every kernel it
    /// touches and none of the other engines in the table were handed one.
    fn columns_of(&self, parquet: &Path) -> Result<String, BenchError> {
        let mut command = Command::new(&self.binary);
        command.arg("local").arg("--format").arg("TSV").arg("--query").arg(format!(
            "DESCRIBE TABLE file('{}', Parquet) SETTINGS schema_inference_make_columns_nullable = 0",
            parquet.display()
        ));
        let out = output(&mut command, "clickhouse local reading a parquet schema")?;
        let text = String::from_utf8_lossy(&out);
        let columns: Vec<String> = text
            .lines()
            .filter_map(|line| {
                let mut fields = line.split('\t');
                let name = fields.next().filter(|n| !n.is_empty())?;
                let kind = fields.next().filter(|k| !k.is_empty())?;
                Some(format!("{name} {kind}"))
            })
            .collect();
        if columns.is_empty() {
            return Err(BenchError::new(format!(
                "clickhouse described no columns in {}",
                parquet.display()
            )));
        }
        Ok(columns.join(", "))
    }
}

impl Drop for ClickhouseServer {
    /// Stop the server when the engine goes away.
    ///
    /// A benchmark harness that leaves a database server running on a random port after it exits is
    /// one that makes the next run of itself slower and nobody connects the two.
    fn drop(&mut self) {
        if let Some(mut child) = self.server.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Engine for ClickhouseServer {
    fn name(&self) -> &str {
        "clickhouse-server"
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn keeps_state(&self) -> bool {
        true
    }

    fn can_run(&self, _suite: &Suite) -> Ability {
        Ability::Yes
    }

    fn load(&mut self, tables: &[Table]) -> Result<Loaded, BenchError> {
        // The server comes up before the clock starts. Bringing a database server up is not loading
        // data, and in any deployment anybody would recognise it is already up when the data
        // arrives. DuckDB's load column is a create and a checkpoint, so this one is a create, an
        // insert and a merge, and neither of them is charged for existing.
        self.start()?;
        let start = Instant::now();
        let mut keys = Vec::with_capacity(tables.len());
        for table in tables {
            // The published schema when the suite has one, and inference when it does not. A
            // column ClickHouse inferred out of Parquet comes back Nullable, which costs a null
            // map and turns several things off, and it is not what the official create.sql says.
            let described;
            let columns =
                match loading(self.suite, self.name(), &table.name).and_then(|l| l.columns) {
                    Some(schema) => schema,
                    None => {
                        described = self.columns_of(&table.path)?;
                        &described
                    }
                };
            let key = sorting_key(self.suite, &table.name);
            keys.push(format!("{} by {key}", table.name));
            self.exec(&format!(
                "CREATE TABLE {} ({columns}) ENGINE = MergeTree ORDER BY {key}",
                table.name
            ))?;

            // Through the client from a file on stdin, which is how the official ClickBench load
            // works and is also the only way that does not depend on where the file happens to be.
            // `file()` on a server only reads inside `user_files_path`, and a harness that moved
            // the corpus to satisfy that would be timing a copy of a seventy gigabyte file.
            let handle = std::fs::File::open(&table.path).map_err(|e| {
                BenchError::new(format!("cannot open {}: {e}", table.path.display()))
            })?;
            let mut command = self.client();
            command
                .arg("--query")
                .arg(format!("INSERT INTO {} FORMAT Parquet", table.name))
                .stdin(Stdio::from(handle));
            output(&mut command, "clickhouse client loading a parquet file")?;

            // Merge to one part before the clock stops, for the same reason DuckDB checkpoints
            // before its clock stops. An insert that left twenty parts to be merged in the
            // background has moved part of the load cost into whichever query runs while the merge
            // is still going, and rule five puts load time next to the runtime precisely so that
            // trade is visible rather than hidden in a query.
            self.exec(&format!("OPTIMIZE TABLE {} FINAL", table.name))?;
        }
        let took = start.elapsed();
        self.sorted_by = keys.join(", ");

        Ok(Loaded {
            took,
            on_disk: self
                .exec(PARTS)
                .ok()
                .and_then(|answer| parts_bytes(&answer))
                .unwrap_or_else(|| size_of_tree(&self.dir.join("store"))),
            on_disk_is: format!(
                "its own MergeTree parts, sorted by {}, as system.parts counts the active ones",
                self.sorted_by
            ),
            converted: true,
            // Not None because nothing was measured, but because what could be measured is the
            // client's, and a load whose CPU column was the cost of reading a file and writing it
            // to a socket would understate the real one by whatever the server spent sorting.
            cpu: None,
        })
    }

    fn run(&mut self, sql: &str) -> Result<Ran, BenchError> {
        self.start()?;
        Ok(Ran { cost: Cost::unavailable(NOT_THE_SERVER), answer: self.exec(sql)? })
    }

    fn unload(&mut self) {
        if keeping() {
            return;
        }
        // Stopped first. Removing a MergeTree directory out from under a running server leaves it
        // writing into deleted inodes, which frees nothing until the process ends and is a strange
        // thing to leave behind for whoever runs the next engine.
        if let Some(mut child) = self.server.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// The server configuration, with the three things that vary substituted in.
///
/// Written per run rather than taken from `/etc`, because the ClickHouse on the fleet is a
/// standalone binary somebody downloaded and there is no `/etc` for it, and because a run that
/// picked up a configuration somebody had edited would be a measurement of that edit.
///
/// Every system log is switched off. That is not tidiness. `metric_log` writes a row a second and
/// `query_log` writes one per query, both into MergeTree tables in the same directory the on disk
/// size is read from, so leaving them on would put tens of megabytes of the server watching itself
/// into a column that is supposed to be the size of the data. `clickhouse local` does not write
/// them either, so switching them off is also what makes the two ClickHouse rows comparable.
const CONFIG: &str = r#"<clickhouse>
    <logger>
        <level>warning</level>
        <log>{dir}/server.log</log>
        <errorlog>{dir}/server.err.log</errorlog>
        <size>50M</size>
        <count>1</count>
    </logger>
    <listen_host>127.0.0.1</listen_host>
    <tcp_port>{port}</tcp_port>
    <path>{dir}/store/</path>
    <tmp_path>{dir}/tmp/</tmp_path>
    <user_files_path>{dir}/user_files/</user_files_path>
    <format_schema_path>{dir}/format_schemas/</format_schema_path>
    <user_directories>
        <users_xml>
            <path>{users}</path>
        </users_xml>
    </user_directories>
    <query_log remove="1"/>
    <query_thread_log remove="1"/>
    <query_views_log remove="1"/>
    <query_metric_log remove="1"/>
    <part_log remove="1"/>
    <trace_log remove="1"/>
    <metric_log remove="1"/>
    <error_log remove="1"/>
    <asynchronous_metric_log remove="1"/>
    <text_log remove="1"/>
    <crash_log remove="1"/>
    <session_log remove="1"/>
    <processors_profile_log remove="1"/>
    <latency_log remove="1"/>
    <backup_log remove="1"/>
    <blob_storage_log remove="1"/>
    <opentelemetry_span_log remove="1"/>
    <s3queue_log remove="1"/>
    <asynchronous_insert_log remove="1"/>
</clickhouse>
"#;

/// One user with no password on the loopback interface, which is the whole access story this needs.
///
/// Loopback only, and the server also listens only there. A benchmark harness that opened a
/// passwordless database to the network for the duration of a run would be a harness somebody
/// eventually runs on a machine with a public address.
const USERS: &str = r#"<clickhouse>
    <profiles>
        <default/>
    </profiles>
    <users>
        <default>
            <password/>
            <networks>
                <ip>127.0.0.1</ip>
                <ip>::1</ip>
            </networks>
            <profile>default</profile>
            <quota>default</quota>
        </default>
    </users>
    <quotas>
        <default/>
    </quotas>
</clickhouse>
"#;

/// What both ClickHouse rows ask for their on disk size.
///
/// The active parts only. An inactive part is one a merge has already replaced and a cleanup thread
/// has not got to yet, and counting those is counting the same rows twice.
const PARTS: &str = "SELECT sum(bytes_on_disk) FROM system.parts WHERE active";

/// The number out of [`PARTS`], when it is a number.
///
/// Zero is refused along with everything else that does not parse. A ten million row table takes
/// space, so a zero here means the question was answered by a server that had not flushed rather
/// than by one with an empty table, and falling back to the directory is the honest answer to that.
fn parts_bytes(answer: &str) -> Option<u64> {
    answer.trim().trim_matches('"').parse::<u64>().ok().filter(|n| *n > 0)
}

/// A port nobody is listening on, found by asking the kernel for one and letting go of it.
///
/// There is a race between letting go and the server binding it, and it is the same race every
/// program that does this has. The alternative is a fixed port, which does not race and does
/// collide, with whatever ClickHouse the machine already had running, and that failure produces a
/// full table of numbers from the wrong server rather than an error.
fn free_port() -> Result<u16, BenchError> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|e| BenchError::new(format!("cannot find a free port: {e}")))?;
    let port = listener
        .local_addr()
        .map_err(|e| BenchError::new(format!("cannot read the port back: {e}")))?
        .port();
    drop(listener);
    Ok(port)
}

/// Write a file, with the failure named after the file.
fn write(path: &Path, text: &str) -> Result<(), BenchError> {
    std::fs::write(path, text)
        .map_err(|e| BenchError::new(format!("cannot write {}: {e}", path.display())))
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
    /// Which suite is running, so the table can be declared the way this engine's own entry
    /// declares it.
    suite: &'static str,
}

impl Datafusion {
    /// Find a `datafusion-cli`.
    ///
    /// # Errors
    ///
    /// When the binary is missing or does not answer `--version`.
    pub fn discover(scratch: &Path, suite: &'static Suite) -> Result<Self, BenchError> {
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
            suite: suite.name,
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
        self.ddl = Vec::with_capacity(tables.len() * 2);
        for t in tables {
            // The entry's own reader options and its own view, when the board publishes them. On
            // ClickBench that is `binary_as_string`, without which every string column arrives as
            // Binary and the queries that compare one against a literal produce nothing at all,
            // which is exactly the empty datafusion column the first full run produced.
            match loading(self.suite, "datafusion", &t.name) {
                Some(Loading { fixup: Fixup::View(view), options, .. }) => {
                    let raw = format!("{}_raw", t.name);
                    self.ddl.push(format!(
                        "CREATE EXTERNAL TABLE {raw} STORED AS PARQUET LOCATION '{}' {options}",
                        t.path.display()
                    ));
                    self.ddl.push(format!(
                        "CREATE VIEW {} AS {}",
                        t.name,
                        view.replace("{raw}", &raw)
                    ));
                }
                _ => self.ddl.push(format!(
                    "CREATE EXTERNAL TABLE {} STORED AS PARQUET LOCATION '{}'",
                    t.name,
                    t.path.display()
                )),
            }
        }
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
        //
        // The table format rather than csv, which is the odd one out among the four engines here
        // and is not a preference. `datafusion-cli` 55.0.0 truncates its streaming output formats.
        // `SELECT i FROM generate_series(1,100000)` prints 16384 rows under `--format csv` and the
        // same 16384 under `--format nd-json`, which is two batches at the default batch size, and
        // an unordered `GROUP BY` over sixteen partitions prints a different number of its 64
        // groups on every run. Under `--format table` with the row cap lifted the same queries
        // print all of it, every time, because that path collects the batches before it prints
        // instead of printing as they arrive. The answer comparison reads numbers out of the text
        // and does not care about the borders, so this costs nothing except an explanation.
        //
        // This is the answer check earning its place on the first day it existed. Without it the
        // datafusion column would have been a fast wrong answer on two of the six smoke queries
        // and nothing would have said so.
        command.arg("-q").arg("--format").arg("table").arg("--maxrows").arg("inf");
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
/// one on the out of core suites. The query goes through `SQLContext` over lazy frames.
///
/// The result is sunk rather than collected, which is what section 13.3 of the engine's
/// `13-measurement.md` asks for and is not the same thing as passing `engine="streaming"` to a
/// collect. A collect runs the streaming engine and then materializes the whole answer in memory
/// before anything is written, so the peak this harness reads includes a result set that every
/// other engine here streamed out. `sink_csv` writes as it goes and never has the answer in one
/// piece, which is the mode the out of core comparison in document 10 section 10.11 is about, and
/// it is also the mode that makes the peak resident column mean the same thing in this row as in
/// the others.
///
/// `maintain_order` stays on. Turning it off is the faster way to sink and it would make the answer
/// check meaningless, since two engines that disagree on row order are indistinguishable from two
/// engines that disagree, and the point of this row is to be comparable rather than to be quick.
#[derive(Debug, Clone)]
pub struct Polars {
    python: PathBuf,
    version: String,
    tables: Vec<Table>,
    runner: Runner,
    script: PathBuf,
    /// Which suite is running, so the scan can be built the way this engine's own entry builds it.
    suite: &'static str,
}

impl Polars {
    /// Find a Python with Polars in it.
    ///
    /// # Errors
    ///
    /// When there is no Python, or when the Python there is cannot import Polars.
    pub fn discover(scratch: &Path, suite: &'static Suite) -> Result<Self, BenchError> {
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
            // The mode goes in the version, because rule one is that the comparison is stated
            // exactly and a Polars number means a different thing in sink mode than it does out of
            // a collect. A reader of the table should not have to go and find out which one this
            // was.
            version: format!("{version} in sink mode"),
            tables: Vec::new(),
            runner: Runner::new(scratch, "polars"),
            script: scratch.join("polars-run.py"),
            suite: suite.name,
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
for triple in sys.argv[2:]:
    name, rest = triple.split("=", 1)
    path, columns = rest.split("\n", 1)
    frame = pl.scan_parquet(path)
    if columns:
        frame = frame.with_columns(eval(columns))
    ctx.register(name, frame)
ctx.execute(sql).sink_csv(
    sys.stdout,
    include_header=False,
    maintain_order=True,
    engine="streaming",
)
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
            // Name, path and the scan expressions, in one argument, separated by a newline that
            // cannot occur in any of the three. A separate argument per part would work equally
            // well and would make the argument list stop being one thing per table, which is the
            // property that makes a failing run reproducible by hand from the log.
            let columns = match loading(self.suite, "polars", &table.name) {
                Some(Loading { fixup: Fixup::Columns(text), .. }) => text,
                _ => "",
            };
            command.arg(format!("{}={}\n{columns}", table.name, table.path.display()));
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
    use super::{Ability, Duckdb, Engine, POLARS_SCRIPT, Rudb, on_path};
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
        let Ok(mut duckdb) = Duckdb::discover(&scratch, find("smoke").unwrap()) else {
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
    fn the_polars_driver_sinks_the_answer_rather_than_collecting_it() {
        // A guard rather than a measurement, because what it is guarding is easy to undo by
        // accident. `collect(engine="streaming")` and `sink_csv` both run the streaming engine and
        // the first one reads like the obvious way to get a frame out, so the difference is one
        // word and it is the difference between a peak that includes the whole answer and a peak
        // that does not. Section 13.3 asks for the second one.
        assert!(POLARS_SCRIPT.contains("sink_csv"), "{POLARS_SCRIPT}");
        assert!(!POLARS_SCRIPT.contains("collect("), "{POLARS_SCRIPT}");
        // Off is faster and would make the answer check meaningless, since a row order this
        // harness cannot predict is indistinguishable from a wrong answer.
        assert!(POLARS_SCRIPT.contains("maintain_order=True"), "{POLARS_SCRIPT}");
    }

    #[test]
    fn a_query_that_fails_is_an_error_and_not_a_fast_run() {
        let scratch = scratch("bad");
        let Ok(mut duckdb) = Duckdb::discover(&scratch, find("smoke").unwrap()) else {
            eprintln!("skipping, no DuckDB on this machine");
            return;
        };
        let got = duckdb.run("SELECT nope FROM nothing");
        assert!(got.is_err(), "a missing table should not time successfully");
        let _ = std::fs::remove_dir_all(&scratch);
    }
}
