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

/// Everything a suite needs before it can run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dataset {
    /// The tables, in the order the suite named them.
    pub tables: Vec<Table>,
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
/// # Errors
///
/// When a file the suite needs is not there, or when the smoke generator could not run.
pub fn prepare(suite: &Suite, scratch: &Path) -> Result<Dataset, BenchError> {
    if suite.name == "smoke" {
        return smoke(scratch);
    }
    if suite.tables.is_empty() {
        return Err(BenchError::new(format!(
            "the {} suite has no table list yet, so there is nothing to hand an engine. It needs {}",
            suite.name, suite.needs
        )));
    }
    let root = root();
    let mut tables = Vec::with_capacity(suite.tables.len());
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
        tables.push(Table { name: (*name).to_owned(), path, bytes });
    }
    Ok(Dataset { tables })
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
    Ok(Dataset { tables: vec![Table { name: "smoke".to_owned(), path, bytes }] })
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
    use super::{Dataset, Table, root, size_of_tree};
    use std::path::{Path, PathBuf};

    #[test]
    fn a_dataset_knows_what_its_files_take() {
        let set = Dataset {
            tables: vec![
                Table { name: "a".to_owned(), path: PathBuf::from("/a"), bytes: 10 },
                Table { name: "b".to_owned(), path: PathBuf::from("/b"), bytes: 32 },
            ],
        };
        assert_eq!(set.bytes(), 42);
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
