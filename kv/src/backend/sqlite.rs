//! SQLite, loaded from `RUDB_BENCH_LIBSQLITE`, in WAL mode with one connection per thread.

use super::{Backend, Failed, Level, Session, Values, library};
use crate::ffi::{Sqlite, SqliteConnection};

/// The library and the file every connection opens.
#[derive(Debug)]
pub(crate) struct SqliteBackend {
    sqlite: Sqlite,
    file: String,
    level: Level,
}

impl SqliteBackend {
    pub(crate) fn open(file: &str, level: Level) -> Result<Self, String> {
        let default =
            if cfg!(target_os = "macos") { "libsqlite3.dylib" } else { "libsqlite3.so.0" };
        let sqlite = Sqlite::load(&library("RUDB_BENCH_LIBSQLITE", default))?;
        let backend = Self { sqlite, file: file.to_string(), level };
        // WAL mode is a property of the file, so it is set once here and every connection sees it.
        let mut first = backend.sqlite.open(file)?;
        first.batch("PRAGMA journal_mode = WAL").map_err(|failed| failed.to_string())?;
        drop(first);
        Ok(backend)
    }

    /// The library file, for the report.
    pub(crate) fn library(&self) -> &str {
        self.sqlite.path()
    }
}

impl Backend for SqliteBackend {
    fn name(&self) -> &'static str {
        "sqlite"
    }

    fn version(&self) -> String {
        self.sqlite.version()
    }

    fn create_table(&self) -> String {
        "CREATE TABLE usertable (ycsb_key TEXT PRIMARY KEY, field0 TEXT, field1 TEXT, field2 TEXT, \
         field3 TEXT, field4 TEXT, field5 TEXT, field6 TEXT, field7 TEXT, field8 TEXT, field9 TEXT) \
         WITHOUT ROWID"
            .to_string()
    }

    fn level(&self) -> Result<(), String> {
        Ok(())
    }

    fn connect(&self) -> Result<Box<dyn Session + '_>, String> {
        let mut connection = self.sqlite.open(&self.file)?;
        let synchronous = match self.level {
            Level::Full => "FULL",
            Level::Os => "NORMAL",
            Level::None => "OFF",
        };
        // On macOS a FULL without fullfsync calls fsync, which Apple's drives do not take to the
        // medium, so it would be `os` claiming to be `full`.
        let fullfsync =
            if cfg!(target_os = "macos") && self.level == Level::Full { "ON" } else { "OFF" };
        let setup = format!(
            "PRAGMA synchronous = {synchronous}; PRAGMA fullfsync = {fullfsync}; \
             PRAGMA busy_timeout = 5000"
        );
        connection.batch(&setup).map_err(|failed| failed.to_string())?;
        Ok(Box::new(SqliteSession(connection)))
    }
}

struct SqliteSession<'a>(SqliteConnection<'a>);

impl Session for SqliteSession<'_> {
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

    /// `BEGIN IMMEDIATE`, which takes the write lock at the start, as SQLite's documentation says
    /// to for a transaction known to write. A deferred one that reads and then writes can fail its
    /// write when another connection wrote in between.
    fn begin(&mut self) -> Result<(), Failed> {
        self.0.batch("BEGIN IMMEDIATE")
    }
}
