//! DuckDB, loaded from `RUDB_BENCH_LIBDUCKDB`, one database and one connection per thread.
//!
//! The library has to be built from the pin's commit. The driver asks it for its version before it
//! opens anything and refuses any other, because the Python wheel and the release libraries are the
//! same line and a different build, and a number from one of them is not a number for the pin.
//! `--unpinned` runs anyway, for trying the driver out, and the header says `pinned=no` so the
//! harness never publishes it.

use super::{Backend, Failed, Level, Session, Values, library};
use crate::ffi::{DuckConnection, Duckdb, duckdb_version};

/// The version string the pin's library reports.
pub(crate) const PIN: &str = "v2.0.0-dev84237";

/// The opened database.
#[derive(Debug)]
pub(crate) struct DuckdbBackend {
    duckdb: Duckdb,
    level: Level,
}

impl DuckdbBackend {
    /// Opens `file` with the library the environment names, after checking the library is the pin.
    pub(crate) fn open(file: &str, level: Level, unpinned: bool) -> Result<(Self, bool), String> {
        let default = if cfg!(target_os = "macos") { "libduckdb.dylib" } else { "libduckdb.so" };
        let path = library("RUDB_BENCH_LIBDUCKDB", default);
        let version = duckdb_version(&path)?;
        let pinned = version == PIN;
        if !pinned && !unpinned {
            return Err(format!(
                "{path} is DuckDB {version} and the pin is {PIN}. Build libduckdb from the pin's \
                 commit, or pass --unpinned to try the driver out with a number nobody publishes"
            ));
        }
        Ok((Self { duckdb: Duckdb::open(&path, file)?, level }, pinned))
    }
}

impl Backend for DuckdbBackend {
    fn name(&self) -> &'static str {
        "duckdb"
    }

    fn version(&self) -> String {
        self.duckdb.version()
    }

    fn create_table(&self) -> String {
        "CREATE TABLE usertable (ycsb_key VARCHAR PRIMARY KEY, field0 VARCHAR, field1 VARCHAR, \
         field2 VARCHAR, field3 VARCHAR, field4 VARCHAR, field5 VARCHAR, field6 VARCHAR, \
         field7 VARCHAR, field8 VARCHAR, field9 VARCHAR)"
            .to_string()
    }

    /// DuckDB syncs its WAL at commit on Linux, which is `full`. On macOS we have not checked which
    /// call it makes, so it is `os` there until we have. It has no level below its default.
    fn level(&self) -> Result<(), String> {
        let own = if cfg!(target_os = "macos") { Level::Os } else { Level::Full };
        if self.level == own {
            Ok(())
        } else {
            Err(format!(
                "DuckDB has one durability level here, {}, and the run asked for {}",
                own.name(),
                self.level.name()
            ))
        }
    }

    fn connect(&self) -> Result<Box<dyn Session + '_>, String> {
        Ok(Box::new(DuckdbSession(self.duckdb.connect()?)))
    }
}

struct DuckdbSession<'a>(DuckConnection<'a>);

impl Session for DuckdbSession<'_> {
    fn prepare(&mut self, sql: &str) -> Result<usize, String> {
        self.0.prepare(sql)
    }

    fn execute(
        &mut self,
        statement: usize,
        parameters: &[&str],
        out: &mut Values,
    ) -> Result<u64, Failed> {
        self.0.execute(statement, parameters, out)
    }

    fn batch(&mut self, sql: &str) -> Result<(), Failed> {
        self.0.batch(sql)
    }
}
