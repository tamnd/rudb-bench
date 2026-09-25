//! PostgreSQL through libpq, loaded from `RUDB_BENCH_LIBPQ`, one backend process per client
//! thread over the Unix socket the connection string names.

use super::{Backend, Failed, Level, Placeholder, Session, Values, library};
use crate::ffi::{Pq, PqConnection};

/// The library and the connection string every client uses.
#[derive(Debug)]
pub(crate) struct PostgresBackend {
    pq: Pq,
    conninfo: String,
    level: Level,
    server: i32,
}

impl PostgresBackend {
    pub(crate) fn open(conninfo: &str, level: Level) -> Result<Self, String> {
        let default = if cfg!(target_os = "macos") { "libpq.dylib" } else { "libpq.so.5" };
        let pq = Pq::load(&library("RUDB_BENCH_LIBPQ", default))?;
        let server = pq.connect(conninfo)?.server_version();
        Ok(Self { pq, conninfo: conninfo.to_string(), level, server })
    }
}

impl Backend for PostgresBackend {
    fn name(&self) -> &'static str {
        "postgres"
    }

    /// The server's version, which is the engine, and libpq's after it.
    fn version(&self) -> String {
        let dotted = |v: i32| format!("{}.{}", v / 10000, v % 10000);
        format!("{} libpq={}", dotted(self.server), dotted(self.pq.library_version()))
    }

    fn placeholder(&self) -> Placeholder {
        Placeholder::Dollar
    }

    /// `COLLATE "C"` is for correctness: in any other collation workload E's `ORDER BY` returns
    /// rows in a different order from every engine that compares bytes.
    fn create_table(&self) -> String {
        "CREATE TABLE usertable (ycsb_key VARCHAR(255) COLLATE \"C\" PRIMARY KEY, field0 TEXT, \
         field1 TEXT, field2 TEXT, field3 TEXT, field4 TEXT, field5 TEXT, field6 TEXT, \
         field7 TEXT, field8 TEXT, field9 TEXT)"
            .to_string()
    }

    fn level(&self) -> Result<(), String> {
        if self.level == Level::Os {
            return Err("PostgreSQL has no level that survives a killed process and not a power \
                        loss, so it runs at full or none"
                .to_string());
        }
        Ok(())
    }

    fn connect(&self) -> Result<Box<dyn Session + '_>, String> {
        let mut connection = self.pq.connect(&self.conninfo)?;
        let synchronous = if self.level == Level::None { "off" } else { "on" };
        connection
            .batch(&format!("SET synchronous_commit = {synchronous}"))
            .map_err(|failed| failed.to_string())?;
        Ok(Box::new(PostgresSession(connection)))
    }
}

struct PostgresSession<'a>(PqConnection<'a>);

impl Session for PostgresSession<'_> {
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
