//! rudb in process, through the Rust API: one `Database`, and one `Connection` per client thread
//! with its statements prepared on it.
//!
//! The Rust API and not the C one, because `rudb-c-api` is still a stub, and because the point path
//! spec (engine-v4 section 13.8) names the Rust API as how rudb is driven in process.

use rudb::{Connection, Database, Prepared, QueryResult, Value};

use super::{Backend, Failed, Session, Values};

/// The opened database.
#[derive(Debug)]
pub(crate) struct RudbBackend {
    database: Database,
}

impl RudbBackend {
    pub(crate) fn open(file: &str) -> Result<Self, String> {
        let database = Database::open(file).map_err(|error| error.to_string())?;
        Ok(Self { database })
    }
}

/// Whether an error is a write-write conflict a client retries.
fn failed(error: impl ToString) -> Failed {
    let message = error.to_string();
    if message.contains("onflict") { Failed::Retry(message) } else { Failed::Error(message) }
}

impl Backend for RudbBackend {
    fn name(&self) -> &'static str {
        "rudb"
    }

    fn version(&self) -> String {
        self.database.value("SELECT version()").map(|v| v.to_string()).unwrap_or_default()
    }

    fn create_table(&self) -> String {
        "CREATE TABLE usertable (ycsb_key VARCHAR PRIMARY KEY, field0 VARCHAR, field1 VARCHAR, \
         field2 VARCHAR, field3 VARCHAR, field4 VARCHAR, field5 VARCHAR, field6 VARCHAR, \
         field7 VARCHAR, field8 VARCHAR, field9 VARCHAR)"
            .to_string()
    }

    fn level(&self) -> Result<(), String> {
        Ok(())
    }

    fn level_note(&self) -> Option<&'static str> {
        Some("rudb has no commit_sync setting before W3, so the run is at its default")
    }

    fn connect(&self) -> Result<Box<dyn Session + '_>, String> {
        Ok(Box::new(RudbSession { connection: self.database.connect(), statements: Vec::new() }))
    }
}

struct RudbSession {
    connection: Connection,
    statements: Vec<Prepared>,
}

fn read(result: &QueryResult, out: &mut Values) -> u64 {
    for row in 0..result.len() {
        for column in 0..result.width() {
            match result.value_at(row, column) {
                Value::Varchar(text) => out.push(text.as_bytes()),
                Value::Null => out.push(b""),
                other => out.push(other.to_string().as_bytes()),
            }
        }
    }
    result.len() as u64
}

impl Session for RudbSession {
    fn prepare(&mut self, sql: &str) -> Result<usize, String> {
        let prepared = self.connection.prepare(sql).map_err(|error| error.to_string())?;
        self.statements.push(prepared);
        Ok(self.statements.len() - 1)
    }

    fn execute(
        &mut self,
        statement: usize,
        parameters: &[&str],
        out: &mut Values,
    ) -> Result<u64, Failed> {
        let values: Vec<Value> =
            parameters.iter().map(|text| Value::Varchar((*text).to_string())).collect();
        let result = self.statements[statement].execute(&values).map_err(failed)?;
        if result.width() > 0 && result.changes().is_none() {
            Ok(read(&result, out))
        } else {
            Ok(result.changes().unwrap_or(0) as u64)
        }
    }

    fn batch(&mut self, sql: &str) -> Result<(), Failed> {
        self.connection.execute(sql).map(|_| ()).map_err(failed)
    }
}
