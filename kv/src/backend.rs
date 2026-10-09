//! One trait every engine is driven through, and the engines.
//!
//! A [`Backend`] is one opened database. A [`Session`] is one connection to it, which belongs to one
//! client thread for its whole life and is never shared, pooled or handed on. Statements are
//! prepared on a session before the run starts and named by the index `prepare` returned, so the
//! loop that is timed does nothing but bind, run and read.

pub(crate) mod duckdb;
pub(crate) mod null;
pub(crate) mod postgres;
#[cfg(feature = "rudb")]
pub(crate) mod rudb;
pub(crate) mod shim;
pub(crate) mod sqlite;

use std::fmt;

/// Why a statement did not run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Failed {
    /// Something a client retries: a busy lock, a write-write conflict, a serialization failure.
    Retry(String),
    /// Anything else, which ends the run.
    Error(String),
}

impl Failed {
    pub(crate) fn message(&self) -> &str {
        match self {
            Self::Retry(message) | Self::Error(message) => message,
        }
    }
}

impl fmt::Display for Failed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

/// The values a statement returned, every column of every row in order, in one buffer.
///
/// One buffer and a list of ends rather than a `Vec<String>`, so reading a row allocates nothing
/// once the buffer has grown to the size of a row.
#[derive(Debug, Default)]
pub(crate) struct Values {
    bytes: Vec<u8>,
    ends: Vec<usize>,
}

impl Values {
    pub(crate) fn clear(&mut self) {
        self.bytes.clear();
        self.ends.clear();
    }

    pub(crate) fn push(&mut self, value: &[u8]) {
        self.bytes.extend_from_slice(value);
        self.ends.push(self.bytes.len());
    }

    pub(crate) fn len(&self) -> usize {
        self.ends.len()
    }

    pub(crate) fn get(&self, at: usize) -> &[u8] {
        let start = if at == 0 { 0 } else { self.ends[at - 1] };
        &self.bytes[start..self.ends[at]]
    }
}

/// The three durability levels of the YCSB spec's section 5.8. A table never compares two.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Level {
    /// An acknowledged commit survives power loss, as far as the device honours a flush.
    Full,
    /// An acknowledged commit survives the process being killed.
    Os,
    /// An acknowledged commit may be lost when only the process dies.
    None,
}

impl Level {
    pub(crate) fn parse(text: &str) -> Result<Self, String> {
        match text {
            "full" => Ok(Self::Full),
            "os" => Ok(Self::Os),
            "none" => Ok(Self::None),
            _ => Err(format!("--level is full, os or none, not {text:?}")),
        }
    }

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Os => "os",
            Self::None => "none",
        }
    }
}

/// How an engine writes a parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Placeholder {
    /// `?`, which rudb, SQLite and DuckDB take.
    Question,
    /// `$1`, `$2` and so on, which is PostgreSQL.
    Dollar,
}

/// One opened database.
pub(crate) trait Backend: Sync {
    /// What the report calls it.
    fn name(&self) -> &'static str;

    /// The exact version of the library or server, which goes next to every number.
    fn version(&self) -> String;

    /// How a statement reaches the engine: `in_process` for a library called on the client's
    /// thread, `unix_socket` for a server, and `unix_shim` for a library behind the shim. The
    /// report labels every number with it.
    fn boundary(&self) -> &'static str {
        "in_process"
    }

    /// How its parameters are written.
    fn placeholder(&self) -> Placeholder {
        Placeholder::Question
    }

    /// The `CREATE TABLE` for the YCSB table in this engine's dialect.
    fn create_table(&self) -> String;

    /// Whether the level asked for was set, or a sentence saying why it could not be.
    fn level(&self) -> Result<(), String>;

    /// A sentence the header carries when the level was accepted and not actually set. No backend
    /// does that now, and one that cannot set a level refuses it instead.
    fn level_note(&self) -> Option<&'static str> {
        None
    }

    /// A statement written with `$1`, `$2` parameters, as this engine wants it written. The TPC-C
    /// text is written that way, and only SQLite needs it changed.
    fn numbered(&self, text: &str) -> String {
        text.to_string()
    }

    /// Whether what is written stays written, so the checks after a run have something to check.
    /// Only the null backend keeps nothing.
    fn keeps_rows(&self) -> bool {
        true
    }

    /// The statement that lets an analytic session's queries take `threads` threads, for the
    /// CH-benCHmark streams, or `None` for an engine that has no such setting.
    fn analytic_threads(&self, _threads: usize) -> Option<String> {
        None
    }

    /// A new connection, for one client thread.
    fn connect(&self) -> Result<Box<dyn Session + '_>, String>;
}

/// One connection, owned by one client thread.
pub(crate) trait Session: Send {
    /// Prepares a statement and returns the index that names it from now on.
    fn prepare(&mut self, sql: &str) -> Result<usize, String>;

    /// Runs a prepared statement with text parameters, puts what it returned in `out`, and returns
    /// the rows it returned or changed.
    fn execute(
        &mut self,
        statement: usize,
        parameters: &[&str],
        out: &mut Values,
    ) -> Result<u64, Failed>;

    /// Runs statements that were not prepared, which is setup and transaction control.
    fn batch(&mut self, sql: &str) -> Result<(), Failed>;

    /// What a prepared statement runs as, for an engine that can say: rudb names its point plans,
    /// and the driver refuses to measure a statement that would go through its full pipeline.
    fn explain(&self, _statement: usize) -> Option<String> {
        None
    }

    fn begin(&mut self) -> Result<(), Failed> {
        self.batch("BEGIN")
    }

    /// Starts a transaction at `ISOLATION LEVEL SERIALIZABLE`, for `--isolation serializable`.
    /// `writes` says whether it is one that writes, for an engine that starts those differently.
    fn begin_serializable(&mut self, _writes: bool) -> Result<(), Failed> {
        self.batch("BEGIN TRANSACTION ISOLATION LEVEL SERIALIZABLE")
    }

    /// Starts a transaction that only reads, which only SQLite starts differently.
    fn begin_read(&mut self) -> Result<(), Failed> {
        self.begin()
    }

    /// Starts a transaction that only reads and sees one snapshot from its first statement to its
    /// last, for the freshness of document 09 section 9.6.
    fn begin_snapshot(&mut self) -> Result<(), Failed> {
        self.begin_read()
    }

    fn commit(&mut self) -> Result<(), Failed> {
        self.batch("COMMIT")
    }

    fn rollback(&mut self) -> Result<(), Failed> {
        self.batch("ROLLBACK")
    }
}

/// Where the libraries are, from the environment the harness sets.
pub(crate) fn library(variable: &str, default: &str) -> String {
    std::env::var(variable).unwrap_or_else(|_| default.to_string())
}
