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

    /// How its parameters are written.
    fn placeholder(&self) -> Placeholder {
        Placeholder::Question
    }

    /// The `CREATE TABLE` for the YCSB table in this engine's dialect.
    fn create_table(&self) -> String;

    /// Whether the level asked for was set, or a sentence saying why it could not be.
    fn level(&self) -> Result<(), String>;

    /// A sentence the header carries when the level was accepted and not actually set, which is
    /// rudb until it has `commit_sync`.
    fn level_note(&self) -> Option<&'static str> {
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

    fn begin(&mut self) -> Result<(), Failed> {
        self.batch("BEGIN")
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
