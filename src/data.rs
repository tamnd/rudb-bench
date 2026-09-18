//! Where a suite's data comes from, as files every engine reads the same way.
//!
//! The first version of this harness handed each engine a list of SQL statements to build its own
//! table with. That works for one engine and falls apart at two, because `range(10000000)` is
//! DuckDB, `numbers(10000000)` is ClickHouse, and neither is DataFusion. Worse, it makes the load
//! comparison meaningless: two engines building two datasets from two different generators are not
//! being timed at the same work.
//!
//! So the data is a set of Parquet files and the load is whatever each engine does to turn those
//! bytes into its own format. That is the same shape the real suites already have, because
//! ClickBench is a Parquet file and TPC-H is eight of them, and it means the load column is
//! measuring the one thing it should be measuring: how fast this engine ingests a file everybody
//! else also had to ingest.
//!
//! A suite of one table can also be asked for fewer rows of it, which is [`Rows`] and `take`. That
//! is a development loop rather than a measurement: the full ClickBench is hours and the question
//! most changes are asking is whether they did anything at all. Every table printed from a smaller
//! run carries the sentence saying what it ran over, and nothing from one may be published.
//!
//! The smoke data is generated rather than downloaded, which makes DuckDB the generator for a
//! comparison DuckDB is in. That is worth saying out loud and it is not worth avoiding. The output
//! is a Parquet file with no DuckDB in it, every engine reads the same one, and the alternative is
//! a generator this project would then have to prove correct before it could use it to prove
//! anything else.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::corpus::Manifest;
use crate::engine::BenchError;
use crate::suite::{Scale, Suite};

/// One table of a suite, as a file on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    /// What the queries call it.
    pub name: String,
    /// The Parquet file it lives in.
    pub path: PathBuf,
    /// What that file takes, which is the number every engine's on disk size is read against.
    pub bytes: u64,
}

/// How many rows of a suite's data to run over, when that is not all of them.
///
/// ClickBench on the real `hits` is a hundred million rows, fifteen gigabytes of Parquet and hours
/// of wall clock once five engines have each loaded it and run forty three queries sixteen times.
/// That is the number that goes in a report and it is a terrible loop to develop against, because
/// the question a developer is asking most days is whether the change they just made did anything
/// at all, and waiting until tomorrow morning to find out that it did not is how a harness stops
/// being run.
///
/// So a run can ask for fewer rows and get an answer in minutes. Nothing that comes out of one is
/// comparable to anything, and [`crate::report::publishable`] says so on every table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rows {
    /// The count asked for.
    pub wanted: u64,
    /// What was typed, which is also what the smaller file is named after.
    pub label: String,
}

impl Rows {
    /// Read `1m`, `200k` or a plain number.
    ///
    /// # Errors
    ///
    /// When the text is not one of those, or when it is zero. A run over no rows would load an
    /// empty table, answer forty three queries instantly and print a table that looks like the
    /// fastest engine anybody has ever measured.
    pub fn parse(text: &str) -> Result<Self, String> {
        let label = text.trim().to_ascii_lowercase();
        let (digits, scale) = match label.strip_suffix('k') {
            Some(rest) => (rest, 1_000),
            None => match label.strip_suffix('m') {
                Some(rest) => (rest, 1_000_000),
                None => (label.as_str(), 1),
            },
        };
        let count: u64 = digits.parse().map_err(|e| {
            format!("--rows {text} is not a row count, {e}. Try 1m, 200k or 500000")
        })?;
        let wanted = count
            .checked_mul(scale)
            .ok_or_else(|| format!("--rows {text} is more rows than there are numbers"))?;
        if wanted == 0 {
            return Err("--rows 0 is a run over an empty table rather than a fast run".to_owned());
        }
        Ok(Self { wanted, label })
    }
}

/// What a smaller version of a table is, and what it is a smaller version of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sample {
    /// Rows in the file the suite would have run over.
    pub full: u64,
    /// Rows in the file it ran over instead.
    pub rows: u64,
    /// One row was kept out of every this many.
    pub every: u64,
    /// What `--rows` was asked for, which is not the same as what came out.
    ///
    /// One row in every `n` of a hundred million lands near the number somebody asked for and
    /// almost never on it. A report that told the reader to reproduce it with the count that came
    /// out would send them to a different `every` and a different file, so the command a report
    /// prints is built from this rather than from [`Self::rows`].
    pub asked: u64,
}

impl Sample {
    /// The sentence that has to travel with any number measured over it.
    #[must_use]
    pub fn sentence(&self) -> String {
        format!(
            "{} rows, one out of every {} of the {} in the full file",
            self.rows, self.every, self.full
        )
    }
}

/// Everything a suite needs before it can run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dataset {
    /// The tables, in the order the suite named them.
    pub tables: Vec<Table>,
    /// How the data was cut down, when it was.
    ///
    /// `None` is the suite as the board runs it. Anything else is a development loop and not a
    /// result, and every table printed from it says so.
    pub sample: Option<Sample>,
    /// How many rows the engines were actually handed, where that is known.
    ///
    /// The suite's declared count for a full run and the sample's for a smaller one, which is why
    /// it lives here rather than being read off the suite wherever a rate is wanted. A report that
    /// divided the suite's hundred million by the time a million row run took would be off by two
    /// orders of magnitude and would look entirely plausible.
    pub rows: Option<u64>,
    /// Which scale factor these files are, for a suite whose size is a choice.
    ///
    /// `None` for a suite that has one corpus. It is carried here rather than looked up again
    /// wherever it is printed, because the run that produced a table and the header that describes
    /// it have to be the same run, and a second lookup is a second chance to describe the wrong one.
    pub scale: Option<&'static Scale>,
    /// Whether the declared row count at this scale is the exact one.
    ///
    /// False only for a generated suite at a factor other than one, where `lineitem` is near its
    /// multiple rather than at it. A rate derived from an approximate count is still worth printing
    /// and is not worth printing to seven digits.
    pub rows_exact: bool,
    /// What the manifest beside these files says about them, when there is one.
    ///
    /// `None` for the smoke suite, which makes its own data every run, and for a corpus that was
    /// put in place by hand before this harness could write manifests. It is carried rather than
    /// read again at print time so that the run and the header describing it are the same corpus,
    /// which is the same reason [`Dataset::scale`] is carried.
    pub manifest: Option<Manifest>,
}

impl Dataset {
    /// The total size of the source files.
    #[must_use]
    pub fn bytes(&self) -> u64 {
        self.tables.iter().map(|t| t.bytes).sum()
    }
}

/// Find or make the files a suite needs.
///
/// `RUDB_BENCH_DATA` is where the downloaded and generated corpora live, and it defaults to
/// `~/rudb-data`, which is where they are on the fleet. Nothing here downloads anything. A suite
/// whose data is missing says which file is missing and where it expected it, because a harness
/// that fetched seventy gigabytes because somebody typed a suite name is a harness people run once.
///
/// `rows` asks for a smaller version of the data, which is built once and reused. See `take` for
/// what smaller means and why it is not the first `n` rows.
///
/// `scale` names which corpus of a generated suite to use, and `None` takes the suite's default.
/// It is a different question from `rows` and the two do not meet: a scale factor is a corpus the
/// generator built to a specification, and a row count is a stride over a corpus that already
/// exists.
///
/// # Errors
///
/// When a file the suite needs is not there, when the smoke generator could not run, or when a
/// smaller version was asked for and this suite is not one that can have one.
pub fn prepare(
    suite: &'static Suite,
    scratch: &Path,
    rows: Option<&Rows>,
    scale: Option<&'static Scale>,
) -> Result<Dataset, BenchError> {
    if suite.name == "smoke" {
        if rows.is_some() {
            return Err(BenchError::new(
                "the smoke suite generates its own data, so there is nothing to take a smaller \
                 version of. Its ten million rows are a constant in the generator and it already \
                 runs in under a minute",
            ));
        }
        return smoke(scratch, suite);
    }
    if suite.tables.is_empty() {
        return Err(BenchError::new(format!(
            "the {} suite has no table list yet, so there is nothing to hand an engine. It needs {}",
            suite.name, suite.needs
        )));
    }
    // One table or nothing, because the rows of two tables are not independent. Keeping every
    // hundredth `lineitem` and every hundredth `orders` keeps a hundredth of each and roughly none
    // of the pairs that join, so every join query would come back nearly empty and the column would
    // read as an engine that got a hundred times faster. Scaling TPC-H is what `dbgen -s` is for.
    if rows.is_some() && suite.tables.len() > 1 {
        return Err(BenchError::new(format!(
            "the {} suite has {} tables, and taking a fraction of the rows of each of them \
             independently keeps almost none of the rows that join, so every join query would \
             answer nothing very quickly. Run a smaller scale factor instead, which is --scale{}",
            suite.name,
            suite.tables.len(),
            match suite.scales().is_empty() {
                true => String::new(),
                false => format!(" and this suite has {}", suite.scale_labels()),
            }
        )));
    }
    let root = root();
    let directory = suite.directory(scale);
    let mut tables = Vec::with_capacity(suite.tables.len());
    let mut sample = None;
    for name in suite.tables {
        let path = root.join(&directory).join(format!("{name}.parquet"));
        let bytes = std::fs::metadata(&path)
            .map_err(|e| {
                BenchError::new(format!(
                    "the {} suite{} needs {}, which is not readable: {e}. It needs {}. Set \
                     RUDB_BENCH_DATA to where the corpora are{}",
                    suite.name,
                    scale.map(|scale| format!(" at {}", scale.named())).unwrap_or_default(),
                    path.display(),
                    suite.needs,
                    makes(suite, scale)
                ))
            })?
            .len();
        let (path, bytes) = match rows {
            None => (path, bytes),
            Some(rows) => {
                let (smaller, took) = take(suite, scratch, &path, rows)?;
                let bytes = std::fs::metadata(&smaller)
                    .map_err(|e| {
                        BenchError::new(format!("cannot size {}: {e}", smaller.display()))
                    })?
                    .len();
                sample = Some(took);
                (smaller, bytes)
            }
        };
        tables.push(Table { name: (*name).to_owned(), path, bytes });
    }
    // The sample's count when there is one, because that is the file the engines were handed, and
    // the suite's declared count at this scale otherwise.
    let rows = sample.map_or_else(|| suite.rows(scale), |s| Some(s.rows));
    let manifest = manifest(suite, &root.join(&directory))?;
    let scale = scale.or_else(|| suite.default_scale());
    Ok(Dataset { tables, sample, rows, scale, rows_exact: suite.rows_exact(scale), manifest })
}

/// The command that writes the corpus a run just failed to find, when there is one.
///
/// Empty for a suite whose data is downloaded, because pointing somebody at a generator that cannot
/// make their data wastes more of their time than saying nothing. A message that ends in the exact
/// line to type is the difference between a missing corpus costing a minute and costing an hour of
/// reading the readme, and the scale factor has to be in it: the common failure is a run at one
/// scale over a machine that only has another.
fn makes(suite: &'static Suite, scale: Option<&'static Scale>) -> String {
    if !matches!(suite.size, crate::suite::Size::Generated { .. }) {
        return String::new();
    }
    let scale = scale.or_else(|| suite.default_scale());
    match scale {
        Some(scale) => format!(
            ". This corpus is generated, so the command that makes it is `rudb-bench generate {} \
             --scale {}`",
            suite.name, scale.label
        ),
        None => format!(
            ". This corpus is generated, so the command that makes it is `rudb-bench generate {}`",
            suite.name
        ),
    }
}

/// Read the manifest beside a corpus, and check it still describes the files that are there.
///
/// The check is per table file length against what was recorded at generation time, which is not a
/// hash and is not meant to be one: rehashing eighty gigabytes before every run would cost more
/// than the run. What it catches is the case that actually happens, which is a corpus half rewritten
/// by a second generation or a file truncated by a disk that filled, and in both of those the length
/// moves. Somebody who wants the real answer has the per table SHA-256s in the manifest and
/// `shasum -a 256` on the files.
///
/// A corpus with no manifest is not an error. Manifests arrived after the first corpora did, and a
/// run over files somebody put in place by hand is still a run, it just cannot say where they came
/// from.
fn manifest(suite: &'static Suite, directory: &Path) -> Result<Option<Manifest>, BenchError> {
    let Some(manifest) = Manifest::beside(directory).map_err(BenchError::new)? else {
        return Ok(None);
    };
    for name in suite.tables {
        let Some(recorded) = manifest.tables.iter().find(|table| table.table == *name) else {
            return Err(BenchError::new(format!(
                "the manifest at {} does not record {name}, which the {} suite needs, so this \
                 directory holds some other corpus. Generate this one again",
                Manifest::path(directory).display(),
                suite.name
            )));
        };
        let path = directory.join(format!("{name}.parquet"));
        let bytes = std::fs::metadata(&path)
            .map_err(|e| BenchError::new(format!("cannot size {}: {e}", path.display())))?
            .len();
        if bytes != recorded.bytes {
            return Err(BenchError::new(format!(
                "{} is {bytes} bytes and the manifest beside it recorded {} when it was written, \
                 so the corpus changed after it was described and nothing measured over it can be \
                 compared to anything. Generate it again",
                path.display(),
                recorded.bytes
            )));
        }
    }
    Ok(Some(manifest))
}

/// Make a smaller version of one Parquet file, or find the one that was made before.
///
/// It keeps every `n`th row rather than the first `n` rows, and the difference is the whole reason
/// this is a function rather than a `LIMIT`. `hits` arrives in the order the events happened, so its
/// first million rows are one morning of one counter: `CounterID` has a handful of distinct values
/// instead of a few thousand, the date range is a day instead of a month, and the group by queries
/// that ClickBench is mostly made of would be measuring a table shape that does not exist anywhere
/// in the real file. Keeping one row in every `n` leaves the distributions roughly where they were
/// and costs one pass over the file, once, the first time a size is asked for.
///
/// The file is written next to the one it came from and named after the size, so the second run
/// finds it and starts immediately. Deleting it is how it gets rebuilt.
///
/// DuckDB does the cutting, for the same reason it generates the smoke data: the output is a
/// Parquet file with nothing of DuckDB in it, every engine reads the same one, and the alternative
/// is a Parquet writer this project would have to prove correct first. It does mean a smaller run
/// needs a DuckDB on the machine even when DuckDB is not one of the engines being measured.
///
/// # Errors
///
/// When there is no DuckDB, when the file cannot be read or written, or when the file is already
/// smaller than the size asked for.
fn take(
    suite: &'static Suite,
    scratch: &Path,
    source: &Path,
    rows: &Rows,
) -> Result<(PathBuf, Sample), BenchError> {
    let duckdb = crate::engine::Duckdb::discover(scratch, suite)?;
    let full = count(&duckdb, source)?;
    if full <= rows.wanted {
        return Err(BenchError::new(format!(
            "{} holds {full} rows, which is not more than the {} that were asked for, so there is \
             no smaller version of it to make",
            source.display(),
            rows.wanted
        )));
    }
    let every = full.div_ceil(rows.wanted);
    let stem = source.file_stem().unwrap_or_default().to_string_lossy().into_owned();
    let at = source.with_file_name(format!("{stem}-{}-snappy-rg8k.parquet", rows.label));

    if !at.is_file() {
        // `file_row_number` is the row's position in the file, which is what makes this a stride
        // over the whole thing rather than a sample that has to hold anything in memory. The
        // ClickBench's source file is Snappy. Keep that codec in the development samples: rudb
        // reads Parquet during every query while DuckDB converts it once at load time, so changing
        // the codec here changes only one side of the repeated query measurement. Eight thousand
        // rows also keeps a string page from becoming the memory of the eight readers that happen
        // to hold one. It is part of the sample's name so an older wide-row-group sample is never
        // silently reused.
        let sql = format!(
            "COPY (SELECT * EXCLUDE (file_row_number) FROM read_parquet('{}', \
             file_row_number = true) WHERE file_row_number % {every} = 0) TO '{}' \
             (FORMAT parquet, COMPRESSION snappy, ROW_GROUP_SIZE 8192)",
            source.display(),
            at.display()
        );
        duckdb.plain(&[&sql])?;
    }
    let kept = count(&duckdb, &at)?;
    Ok((at, Sample { full, rows: kept, every, asked: rows.wanted }))
}

/// How many rows a Parquet file holds, out of its footer rather than out of a scan.
fn count(duckdb: &crate::engine::Duckdb, at: &Path) -> Result<u64, BenchError> {
    let sql = format!("SELECT sum(num_rows) FROM parquet_file_metadata('{}')", at.display());
    let text = duckdb.ask(&sql)?;
    text.trim().parse().map_err(|e| {
        BenchError::new(format!("cannot read the row count of {} out of {text}: {e}", at.display()))
    })
}

/// Where the corpora live.
#[must_use]
pub fn root() -> PathBuf {
    std::env::var_os("RUDB_BENCH_DATA").map_or_else(
        || {
            std::env::var_os("HOME").map_or_else(
                || PathBuf::from("rudb-data"),
                |home| PathBuf::from(home).join("rudb-data"),
            )
        },
        PathBuf::from,
    )
}

/// Ten million rows of generated integers and strings, written once and reused.
///
/// Under the data root rather than the scratch directory, so that a second run of the suite does
/// not spend a second regenerating a file that is deterministic. Deleting it is how you regenerate
/// it, and the name says which version generated it so an old one is not silently reused after the
/// shape changes.
fn smoke(scratch: &Path, suite: &Suite) -> Result<Dataset, BenchError> {
    let root = root();
    std::fs::create_dir_all(&root)
        .map_err(|e| BenchError::new(format!("cannot make {}: {e}", root.display())))?;
    let path = root.join("smoke-1.parquet");

    if !path.is_file() {
        // The smoke suite, because that is what is being generated here and because a DuckDB
        // carries a suite now so that it can build a table the way the board's entry for it does.
        // Nothing on this path builds a table: `plain` runs statements against no database at all.
        let suite = crate::suite::find("smoke")
            .ok_or_else(|| BenchError::new("smoke is not a suite, which cannot happen"))?;
        let duckdb = crate::engine::Duckdb::discover(scratch, suite)?;
        let sql = format!(
            "COPY ({SMOKE_ROWS}) TO '{}' (FORMAT parquet, COMPRESSION zstd)",
            path.display()
        );
        duckdb.plain(&[&sql])?;
    }

    let bytes = std::fs::metadata(&path)
        .map_err(|e| BenchError::new(format!("cannot size {}: {e}", path.display())))?
        .len();
    Ok(Dataset {
        tables: vec![Table { name: "smoke".to_owned(), path, bytes }],
        sample: None,
        rows: suite.rows(None),
        scale: None,
        rows_exact: true,
        // The smoke suite writes its one file here, every run, from the rows above it. There is
        // nothing to describe that this function does not already know.
        manifest: None,
    })
}

/// The smoke rows.
///
/// Ten million is chosen so the load is a second or two and a scan is tens of milliseconds, which
/// is far enough above a process spawn to be measuring the engine and far enough below a coffee
/// break to run on every commit. The string column is deliberately low cardinality, because a
/// dictionary is the first thing any of these engines does with one and a smoke test that never
/// exercises a dictionary is not exercising much.
///
/// The arithmetic is written to give the same answers in every engine in the comparison. Integer
/// multiplication in a signed 64 bit column, a modulus, and a division by a literal with a decimal
/// point in it, which is a double everywhere rather than an integer division in some engines and a
/// double in others.
const SMOKE_ROWS: &str = "SELECT i AS id,
              (i * 2654435761) % 1000 AS k,
              (i % 97) / 7.0 AS v,
              'tag-' || ((i * 48271) % 64)::VARCHAR AS tag
       FROM range(10000000) t(i)";

/// A file's size, for the engines that report the source file as their own on disk size.
#[must_use]
pub fn size_of(path: &Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or_default()
}

/// The total size of a directory tree, which is what an engine that keeps a directory reports.
///
/// Follows nothing and counts what it can read. An engine whose data directory has something in it
/// this process cannot stat has a problem this function should report as a smaller number rather
/// than as a failure, because the number is a supporting column and the run is the result.
#[must_use]
pub fn size_of_tree(path: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else { return size_of(path) };
    let mut total = 0;
    for entry in entries.filter_map(Result::ok) {
        let Ok(kind) = entry.file_type() else { continue };
        if kind.is_dir() {
            total += size_of_tree(&entry.path());
        } else if kind.is_file() {
            total += entry.metadata().map(|m| m.len()).unwrap_or_default();
        }
    }
    total
}

/// Run a command, and turn a failure into its stderr rather than into an exit code.
///
/// Shared by every engine in [`crate::engine`], because five engines each writing their own version
/// of this is five places for a failed load to be reported as a very fast load.
pub(crate) fn output(command: &mut Command, what: &str) -> Result<Vec<u8>, BenchError> {
    Ok(both(command, what)?.0)
}

/// The same, keeping the stderr of a run that worked.
///
/// Two of the engines report what a query cost them on stderr rather than stdout, which is the
/// better of the two places for it because it keeps the timing out of the answer without anybody
/// having to strip it back out. [`output`] throws that half away, so the timed path calls this one
/// and the apparatus calls that one.
pub(crate) fn both(command: &mut Command, what: &str) -> Result<(Vec<u8>, Vec<u8>), BenchError> {
    match both_within(command, what, None)? {
        Finished::Ran { stdout, stderr } => Ok((stdout, stderr)),
        // Unreachable with no limit, and an assertion rather than a panic because the caller that
        // passed `None` is the one making the claim.
        Finished::TimedOut => Err(BenchError::new(format!(
            "{what} reported a timeout against no limit, which is a bug in the harness"
        ))),
    }
}

/// What waiting for a command produced.
#[derive(Debug)]
pub(crate) enum Finished {
    /// It exited on its own, successfully.
    Ran { stdout: Vec<u8>, stderr: Vec<u8> },
    /// The limit came first, and the process was killed.
    TimedOut,
}

/// The same as [`both`], giving up after a limit rather than waiting as long as it takes.
///
/// A query that does not finish is a fact about that query, and a harness with no limit turns it
/// into a fact about the whole run: the suite stops, and the twenty one queries after it have no
/// number for a reason that has nothing to do with them. So the limit fires, the process is killed,
/// and the caller gets a result to put in a cell.
///
/// Both pipes are drained on threads of their own. A child that fills the pipe buffer blocks in
/// `write` and never reaches its own exit, so a parent that waited first and read afterwards would
/// hang on exactly the queries this exists to catch, and would report them as timeouts whatever
/// they were really doing.
///
/// The child is put in a process group of its own, which is what makes the limit mean anything.
/// Every engine here is run under `/usr/bin/time`, so the process this harness spawns is the timer
/// and the engine is the timer's child. Killing the process would kill the timer and leave the
/// engine running, holding the write end of both pipes, and the two draining threads would then
/// wait on an engine nobody is waiting for any more. A group is the handle on all of it.
///
/// The cost of that group is Ctrl-C. A terminal sends its interrupt to the foreground group, which
/// the engine is no longer in, so a run stopped by hand leaves the query that was running to finish
/// on its own. That is a worse trade in a terminal and a better one everywhere else, because the
/// case it replaces left an engine behind on every query that ran out of its limit, and a suite at
/// SF1 has twenty two of those. A run stopped by hand leaves one, and `kill -9 -<pid>` takes it.
pub(crate) fn both_within(
    command: &mut Command,
    what: &str,
    limit: Option<Duration>,
) -> Result<Finished, BenchError> {
    command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // Zero means the group takes the child's own pid, so the group id to kill later is the pid
        // this side already has.
        command.process_group(0);
    }
    let mut child =
        command.spawn().map_err(|e| BenchError::new(format!("cannot run {what}: {e}")))?;
    let mut out = child.stdout.take().ok_or_else(|| BenchError::new("no stdout pipe"))?;
    let mut err = child.stderr.take().ok_or_else(|| BenchError::new("no stderr pipe"))?;
    let draining = std::thread::scope(|scope| {
        let reading_out = scope.spawn(move || {
            let mut bytes = Vec::new();
            out.read_to_end(&mut bytes).map(|_| bytes)
        });
        let reading_err = scope.spawn(move || {
            let mut bytes = Vec::new();
            err.read_to_end(&mut bytes).map(|_| bytes)
        });
        let status = wait_within(&mut child, limit);
        // After the wait either way. On a timeout the kill closes the pipes, which is what lets
        // these two return at all.
        let stdout = reading_out.join().unwrap_or_else(|_| Ok(Vec::new()));
        let stderr = reading_err.join().unwrap_or_else(|_| Ok(Vec::new()));
        (status, stdout, stderr)
    });
    let (status, stdout, stderr) = draining;
    let stdout = stdout.map_err(|e| BenchError::new(format!("cannot read {what} stdout: {e}")))?;
    let stderr = stderr.map_err(|e| BenchError::new(format!("cannot read {what} stderr: {e}")))?;
    let Some(status) =
        status.map_err(|e| BenchError::new(format!("cannot wait for {what}: {e}")))?
    else {
        return Ok(Finished::TimedOut);
    };
    if status.success() {
        return Ok(Finished::Ran { stdout, stderr });
    }
    let stderr = String::from_utf8_lossy(&stderr);
    let tail: Vec<&str> = stderr.lines().rev().take(6).collect();
    let mut lines: Vec<&str> = tail.into_iter().rev().collect();
    if lines.is_empty() {
        lines.push("it said nothing at all");
    }
    Err(BenchError::new(format!("{what} failed: {}", lines.join(" "))))
}

/// Waits for a child, killing it and answering `None` if the limit comes first.
///
/// Polled rather than waited on, because waiting for a child with a timeout is not in the standard
/// library and the alternative is a signal handler or a second process. The interval is a
/// compromise nobody has to think about: a limit is seconds at the smallest, so twenty milliseconds
/// of slack is under a percent of it, and a poll every twenty milliseconds costs nothing measurable
/// beside a query that is running.
fn wait_within(
    child: &mut Child,
    limit: Option<Duration>,
) -> std::io::Result<Option<std::process::ExitStatus>> {
    let Some(limit) = limit else {
        return child.wait().map(Some);
    };
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(Some(status));
        }
        if started.elapsed() >= limit {
            // Killed rather than asked politely. There is no portable way to ask, and a query that
            // ignored the request would leave the harness waiting again on the thing it gave up on.
            stop(child);
            let _ = child.wait();
            return Ok(None);
        }
        std::thread::sleep(POLL);
    }
}

/// How often a run that has a limit is asked whether it is done.
const POLL: Duration = Duration::from_millis(20);

/// Stop a child and everything it started.
///
/// The child is killed either way. On Unix the group it leads is killed first, which is the part
/// that matters: the process this harness spawned is `/usr/bin/time` and the engine underneath it
/// is a child of that, so killing the process alone stops the timer and leaves the engine to run
/// for as long as the query takes. That is not a hypothetical. It is what a sixty second limit on
/// TPC-H q02 did before this, which left one engine process on the machine for fifty minutes and a
/// harness waiting on the pipes it still had open.
///
/// A group that is already gone is not an error here. The child may have exited between the poll
/// that said it had not and this line, which is a race nothing can close and nothing needs to.
///
/// The group is killed by running `kill`, which is a subprocess to send a signal and is not how
/// anybody would write this given a free hand. The free hand is the point: this crate forbids
/// unsafe code and has no dependencies, so `kill(2)` is not reachable from it, and the choice is
/// between one short lived process on the path where a query has already spent its whole limit and
/// giving up one of the two properties everywhere. `-9` is the signal and `-<group>` is the group,
/// which is POSIX and is what both the shell builtin and `/bin/kill` do.
fn stop(child: &mut Child) {
    #[cfg(unix)]
    {
        let group = child.id();
        let _ = Command::new("sh")
            .arg("-c")
            .arg(format!("kill -9 -{group}"))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
}

#[cfg(test)]
mod tests {
    use super::{
        Dataset, Finished, Rows, Sample, Table, both_within, makes, manifest, prepare, root,
        size_of_tree,
    };
    use crate::corpus::{Manifest, Provenance, Recorded};
    use crate::suite::SUITES;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{Duration, Instant};

    /// A command that finishes inside its limit is a run, and it is not slowed down by having one.
    #[test]
    fn a_command_that_finishes_in_time_comes_back_with_what_it_said() {
        let mut command = Command::new("sh");
        command.arg("-c").arg("echo out; echo err 1>&2");
        let got = both_within(&mut command, "a shell", Some(Duration::from_secs(30)))
            .expect("a shell that echoes is not a failure");
        match got {
            Finished::Ran { stdout, stderr } => {
                assert_eq!(String::from_utf8_lossy(&stdout).trim(), "out");
                assert_eq!(String::from_utf8_lossy(&stderr).trim(), "err");
            }
            Finished::TimedOut => panic!("an echo took more than thirty seconds"),
        }
    }

    /// The whole point. A command that will not finish stops being waited for, and it stops being
    /// waited for near the limit rather than a long time after it.
    #[test]
    fn a_command_that_will_not_finish_is_stopped_at_its_limit() {
        let mut command = Command::new("sh");
        command.arg("-c").arg("sleep 60");
        let started = Instant::now();
        let got = both_within(&mut command, "a sleep", Some(Duration::from_millis(300)))
            .expect("giving up on a sleep is not a failure");
        let waited = started.elapsed();
        assert!(matches!(got, Finished::TimedOut), "{got:?}");
        assert!(waited < Duration::from_secs(10), "waited {waited:?} on a limit of 300ms");
    }

    /// The shape every engine here actually has. `/usr/bin/time` is what gets spawned and the
    /// engine is its child, so a limit that only stopped the process it spawned would stop the
    /// timer, leave the engine running, and then wait on the pipes the engine still holds. This is
    /// that arrangement in one line of shell, and the limit has to get through both of them.
    #[test]
    fn a_command_that_started_another_one_is_stopped_along_with_it() {
        let mut command = Command::new("sh");
        command.arg("-c").arg("sh -c 'sleep 30' & sleep 30");
        let started = Instant::now();
        let got =
            both_within(&mut command, "a sleep under a sleep", Some(Duration::from_millis(300)))
                .expect("giving up on a sleep is not a failure");
        let waited = started.elapsed();
        assert!(matches!(got, Finished::TimedOut), "{got:?}");
        assert!(waited < Duration::from_secs(10), "waited {waited:?} on a limit of 300ms");
    }

    /// A child that writes more than a pipe buffer holds blocks in `write` until somebody reads it.
    /// A parent that waited first and read afterwards would hang here, on exactly the runs this
    /// exists to catch, so both pipes are drained while the wait is going on.
    #[test]
    fn a_command_that_fills_the_pipe_is_read_rather_than_deadlocked_against() {
        let mut command = Command::new("sh");
        // A megabyte, which is well past the sixty four kilobytes a pipe holds on both platforms
        // this runs on.
        command.arg("-c").arg("yes abcdefghijklmnopqrstuvwxyz | head -c 1048576");
        let got = both_within(&mut command, "a lot of output", Some(Duration::from_secs(60)))
            .expect("a megabyte of output is not a failure");
        match got {
            Finished::Ran { stdout, .. } => assert_eq!(stdout.len(), 1_048_576),
            Finished::TimedOut => panic!("a megabyte took a minute"),
        }
    }

    #[test]
    fn a_corpus_that_changed_after_it_was_described_is_refused() {
        let suite = SUITES.iter().find(|suite| suite.name == "tpch").expect("tpch is a suite");
        let dir =
            std::env::temp_dir().join(format!("rudb-bench-moved-corpus-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let mut written = Manifest {
            suite: "tpch".to_owned(),
            scale: "0.01".to_owned(),
            provenance: Provenance::DuckdbTpch,
            generator: "duckdb tpch extension, CALL dbgen(sf = 0.01)".to_owned(),
            converter: "duckdb v1.5.5".to_owned(),
            hashed_by: "shasum -a 256".to_owned(),
            written: "2026-09-18".to_owned(),
            tables: Vec::new(),
            properties: Vec::new(),
        };
        for name in suite.tables {
            let path = dir.join(format!("{name}.parquet"));
            std::fs::write(&path, b"four").expect("write table");
            written.tables.push(Recorded {
                table: (*name).to_owned(),
                rows: 1,
                bytes: 4,
                sha256: "c".repeat(64),
            });
        }
        std::fs::write(Manifest::path(&dir), written.write()).expect("write manifest");
        assert!(manifest(suite, &dir).expect("a corpus that matches").is_some());

        // One byte longer than it was described as, which is what half a rewritten corpus looks
        // like from here.
        std::fs::write(dir.join("nation.parquet"), b"fives").expect("rewrite one table");
        let e = manifest(suite, &dir).expect_err("a corpus that moved");
        let said = e.to_string();
        assert!(said.contains("5 bytes"), "{said}");
        assert!(said.contains("recorded 4"), "{said}");
        assert!(said.contains("Generate it again"), "{said}");
        std::fs::remove_dir_all(&dir).expect("clean up");
    }

    #[test]
    fn a_missing_generated_corpus_says_what_makes_it() {
        let suite = SUITES.iter().find(|suite| suite.name == "tpch").expect("tpch is a suite");
        let scale = suite.scales().iter().find(|scale| scale.label == "1");
        let said = makes(suite, scale);
        assert!(said.contains("rudb-bench generate tpch --scale 1"), "{said}");

        // ClickBench is downloaded, so there is no command to offer and offering one would be a
        // lie that costs somebody a download.
        let hits = SUITES.iter().find(|suite| suite.name == "clickbench");
        if let Some(hits) = hits {
            assert_eq!(makes(hits, None), "");
        }
    }

    #[test]
    fn a_dataset_knows_what_its_files_take() {
        let set = Dataset {
            tables: vec![
                Table { name: "a".to_owned(), path: PathBuf::from("/a"), bytes: 10 },
                Table { name: "b".to_owned(), path: PathBuf::from("/b"), bytes: 32 },
            ],
            sample: None,
            rows: None,
            scale: None,
            rows_exact: true,
            manifest: None,
        };
        assert_eq!(set.bytes(), 42);
    }

    #[test]
    fn a_row_count_can_be_written_the_way_people_say_it() {
        assert_eq!(Rows::parse("1m").unwrap().wanted, 1_000_000);
        assert_eq!(Rows::parse("200k").unwrap().wanted, 200_000);
        assert_eq!(Rows::parse("500000").unwrap().wanted, 500_000);
        assert_eq!(Rows::parse("1M").unwrap().label, "1m", "the file is named after it");
    }

    #[test]
    fn a_row_count_that_is_not_one_is_refused_with_the_forms_that_work() {
        let e = Rows::parse("a few").unwrap_err();
        assert!(e.contains("1m, 200k or 500000"), "{e}");
        assert!(Rows::parse("1b").is_err(), "there is no b suffix");
        // Zero rows would load an empty table and answer every query instantly, which is the one
        // failure here that looks like a result rather than like a mistake.
        assert!(Rows::parse("0").is_err());
    }

    #[test]
    fn a_sample_says_what_it_is_a_sample_of() {
        let sample = Sample { full: 99_997_497, rows: 999_975, every: 100, asked: 1_000_000 };
        assert_eq!(
            sample.sentence(),
            "999975 rows, one out of every 100 of the 99997497 in the full file"
        );
    }

    /// Two tables cannot be cut down a table at a time, because the rows that join are not the rows
    /// that happened to survive. A run that did it anyway would answer every join query with almost
    /// nothing and report the fastest engine anybody has measured.
    #[test]
    fn a_suite_of_more_than_one_table_refuses_to_be_made_smaller() {
        let tpch = crate::suite::find("tpch").expect("tpch is a suite");
        let rows = Rows::parse("1m").unwrap();
        let e = prepare(tpch, Path::new("/no/such/scratch"), Some(&rows), None)
            .expect_err("eight tables cannot be sampled one at a time");
        assert!(e.to_string().contains("join"), "{e}");
    }

    #[test]
    fn the_smoke_suite_refuses_too_because_it_generates_its_own_data() {
        let smoke = crate::suite::find("smoke").expect("smoke is a suite");
        let rows = Rows::parse("1m").unwrap();
        let e = prepare(smoke, Path::new("/no/such/scratch"), Some(&rows), None)
            .expect_err("there is nothing to take a smaller version of");
        assert!(e.to_string().contains("generates its own data"), "{e}");
    }

    #[test]
    fn the_data_root_is_a_place_and_not_a_panic() {
        assert!(root().is_absolute() || root().as_path() == Path::new("rudb-data"));
    }

    #[test]
    fn a_tree_that_is_not_there_is_zero_rather_than_an_error() {
        assert_eq!(size_of_tree(&PathBuf::from("/no/such/directory/here")), 0);
    }
}
