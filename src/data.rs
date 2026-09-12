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

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::engine::BenchError;
use crate::suite::Suite;

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
/// # Errors
///
/// When a file the suite needs is not there, when the smoke generator could not run, or when a
/// smaller version was asked for and this suite is not one that can have one.
pub fn prepare(
    suite: &'static Suite,
    scratch: &Path,
    rows: Option<&Rows>,
) -> Result<Dataset, BenchError> {
    if suite.name == "smoke" {
        if rows.is_some() {
            return Err(BenchError::new(
                "the smoke suite generates its own data, so there is nothing to take a smaller \
                 version of. Its ten million rows are a constant in the generator and it already \
                 runs in under a minute",
            ));
        }
        return smoke(scratch);
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
             answer nothing very quickly. Generate a smaller scale factor instead",
            suite.name,
            suite.tables.len()
        )));
    }
    let root = root();
    let mut tables = Vec::with_capacity(suite.tables.len());
    let mut sample = None;
    for name in suite.tables {
        let path = root.join(suite.directory).join(format!("{name}.parquet"));
        let bytes = std::fs::metadata(&path)
            .map_err(|e| {
                BenchError::new(format!(
                    "the {} suite needs {}, which is not readable: {e}. It needs {}. Set \
                     RUDB_BENCH_DATA to where the corpora are",
                    suite.name,
                    path.display(),
                    suite.needs
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
    Ok(Dataset { tables, sample })
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
    let at = source.with_file_name(format!("{stem}-{}.parquet", rows.label));

    if !at.is_file() {
        // `file_row_number` is the row's position in the file, which is what makes this a stride
        // over the whole thing rather than a sample that has to hold anything in memory. The
        // compression is zstd rather than whatever the source used, which is one more reason a
        // number measured over this file is not comparable to a number measured over that one.
        let sql = format!(
            "COPY (SELECT * EXCLUDE (file_row_number) FROM read_parquet('{}', \
             file_row_number = true) WHERE file_row_number % {every} = 0) TO '{}' \
             (FORMAT parquet, COMPRESSION zstd)",
            source.display(),
            at.display()
        );
        duckdb.plain(&[&sql])?;
    }
    let kept = count(&duckdb, &at)?;
    Ok((at, Sample { full, rows: kept, every }))
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
fn smoke(scratch: &Path) -> Result<Dataset, BenchError> {
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
    Ok(Dataset { tables: vec![Table { name: "smoke".to_owned(), path, bytes }], sample: None })
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
    let out = command.output().map_err(|e| BenchError::new(format!("cannot run {what}: {e}")))?;
    if out.status.success() {
        return Ok(out.stdout);
    }
    let stderr = String::from_utf8_lossy(&out.stderr);
    let tail: Vec<&str> = stderr.lines().rev().take(6).collect();
    let mut lines: Vec<&str> = tail.into_iter().rev().collect();
    if lines.is_empty() {
        lines.push("it said nothing at all");
    }
    Err(BenchError::new(format!("{what} failed: {}", lines.join(" "))))
}

#[cfg(test)]
mod tests {
    use super::{Dataset, Rows, Sample, Table, prepare, root, size_of_tree};
    use std::path::{Path, PathBuf};

    #[test]
    fn a_dataset_knows_what_its_files_take() {
        let set = Dataset {
            tables: vec![
                Table { name: "a".to_owned(), path: PathBuf::from("/a"), bytes: 10 },
                Table { name: "b".to_owned(), path: PathBuf::from("/b"), bytes: 32 },
            ],
            sample: None,
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
        let sample = Sample { full: 99_997_497, rows: 999_975, every: 100 };
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
        let e = prepare(tpch, Path::new("/no/such/scratch"), Some(&rows))
            .expect_err("eight tables cannot be sampled one at a time");
        assert!(e.to_string().contains("join"), "{e}");
    }

    #[test]
    fn the_smoke_suite_refuses_too_because_it_generates_its_own_data() {
        let smoke = crate::suite::find("smoke").expect("smoke is a suite");
        let rows = Rows::parse("1m").unwrap();
        let e = prepare(smoke, Path::new("/no/such/scratch"), Some(&rows))
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
