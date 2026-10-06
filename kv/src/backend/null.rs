//! No engine. Everything but the call: the operations are generated, the keys formatted, a read
//! gets back a row of valid values and checks them, and the latencies are recorded.
//!
//! Closed loop it gives the driver's own ceiling, and open loop its own share of the latency and
//! the CPU. With `--service` and `--stall` it is a fake engine with a known stall, which is how the
//! driver shows it can see a stall it put there itself, the self-test of the YCSB spec's section
//! 4.12.
//!
//! In the `tpcc` mode it hands back canned rows of the right shape for each statement, which is
//! the driver's own cost and ceiling for TPC-C, the spec's document 04 section 4.9.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use super::{Backend, Failed, Session, Values};
use crate::tpcc::text::Id;
use crate::tpcc::{CUSTOMERS, ITEMS};
use crate::workload::value;

/// A fake engine.
#[derive(Debug)]
pub(crate) struct NullBackend {
    /// How long every call spins.
    service: Duration,
    /// How long the whole engine stops, and how often, from when it was opened.
    stall: Option<(Duration, Duration)>,
    opened: Instant,
    /// The New-Orders committed so far, which is what `sum(d_next_o_id)` would have grown by.
    new_orders: AtomicU64,
}

impl NullBackend {
    pub(crate) fn new(service: Duration, stall: Option<(Duration, Duration)>) -> Self {
        Self { service, stall, opened: Instant::now(), new_orders: AtomicU64::new(0) }
    }

    /// Waits out a stall if one is on, then spins for the service time.
    fn serve(&self) {
        if let Some((length, every)) = self.stall {
            // The stall is the last `length` of every period, so a run sees its first one a period
            // after the engine opened and not at the start of the warmup.
            let into = self.opened.elapsed().as_nanos() % every.as_nanos();
            let quiet = every.as_nanos() - length.as_nanos();
            if into >= quiet {
                let left = u64::try_from(every.as_nanos() - into).unwrap_or(0);
                std::thread::sleep(Duration::from_nanos(left));
            }
        }
        if !self.service.is_zero() {
            let start = Instant::now();
            while start.elapsed() < self.service {
                std::hint::spin_loop();
            }
        }
    }
}

impl Backend for NullBackend {
    fn name(&self) -> &'static str {
        "null"
    }

    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn create_table(&self) -> String {
        String::new()
    }

    fn level(&self) -> Result<(), String> {
        Ok(())
    }

    fn keeps_rows(&self) -> bool {
        false
    }

    fn connect(&self) -> Result<Box<dyn Session + '_>, String> {
        Ok(Box::new(NullSession {
            backend: self,
            canned: Vec::new(),
            next_order: HashMap::new(),
            pending: 0,
        }))
    }
}

/// What a prepared statement hands back.
#[derive(Debug, Clone, Copy)]
enum Canned {
    /// A YCSB read or scan: a row of valid values for its key.
    Read,
    /// One of the TPC-C statements, as its shape says.
    Tpcc(Id),
    /// TPC-C's multi-row insert of order lines, which changes its rows and returns nothing.
    Lines,
    /// Any other `SELECT`, which gets one row of a zero.
    Select,
    /// Anything else, which changes one row and returns nothing.
    Write,
}

struct NullSession<'a> {
    backend: &'a NullBackend,
    canned: Vec<Canned>,
    /// The next order id of each district this session allocated one in, from `(w, d)`.
    next_order: HashMap<(u64, u64), u64>,
    /// New-Orders this transaction allocated, which the backend's count gets at commit.
    pending: u64,
}

impl NullSession<'_> {
    /// The rows of a TPC-C statement: an order id that counts up per district for `no1`, no item
    /// for the item that does not exist, `GC` credit, the count of committed New-Orders for the
    /// monitor, and ones everywhere else in the shape the statement returns.
    fn tpcc(&mut self, id: Id, parameters: &[&str], out: &mut Values) -> u64 {
        let one = |at: usize| parameters.get(at).and_then(|p| p.parse::<u64>().ok()).unwrap_or(1);
        match id {
            Id::No1 => {
                let next = self.next_order.entry((one(0), one(1))).or_insert(CUSTOMERS + 1);
                out.push(next.to_string().as_bytes());
                out.push(b"0.1000");
                *next += 1;
                self.pending += 1;
                return 1;
            }
            Id::No6 if one(0) > ITEMS => return 0,
            Id::No6 => {
                out.push(b"12.34");
                out.push(b"an item");
                out.push(b"ORIGINAL data");
                return 1;
            }
            Id::Pa4 => {
                for column in 0..14 {
                    out.push(if column == 10 { b"GC" } else { b"1" });
                }
                return 1;
            }
            Id::Orders => {
                out.push(self.backend.new_orders.load(Ordering::Relaxed).to_string().as_bytes());
                return 1;
            }
            _ => {}
        }
        let (rows, columns) = id.shape();
        for _ in 0..rows * columns {
            out.push(b"1");
        }
        rows.max(1) as u64
    }
}

impl Session for NullSession<'_> {
    fn prepare(&mut self, sql: &str) -> Result<usize, String> {
        let upper = sql.trim_start().to_ascii_uppercase();
        let canned = if let Some(id) = Id::of(sql) {
            Canned::Tpcc(id)
        } else if sql.contains("usertable") && upper.starts_with("SELECT") {
            Canned::Read
        } else if upper.starts_with("INSERT INTO ORDER_LINE") {
            Canned::Lines
        } else if upper.starts_with("SELECT") {
            Canned::Select
        } else {
            Canned::Write
        };
        self.canned.push(canned);
        Ok(self.canned.len() - 1)
    }

    fn execute(
        &mut self,
        statement: usize,
        parameters: &[&str],
        out: &mut Values,
    ) -> Result<u64, Failed> {
        self.backend.serve();
        match self.canned[statement] {
            Canned::Read => {
                let key = parameters.first().copied().unwrap_or_default();
                out.push(key.as_bytes());
                for field in 0..10 {
                    out.push(value(key, field, 0).as_bytes());
                }
                Ok(1)
            }
            Canned::Tpcc(id) => Ok(self.tpcc(id, parameters, out)),
            Canned::Lines => Ok(parameters.len() as u64 / 9),
            Canned::Select => {
                out.push(b"0");
                Ok(1)
            }
            Canned::Write => Ok(1),
        }
    }

    fn batch(&mut self, _sql: &str) -> Result<(), Failed> {
        Ok(())
    }

    fn begin(&mut self) -> Result<(), Failed> {
        self.pending = 0;
        Ok(())
    }

    fn commit(&mut self) -> Result<(), Failed> {
        self.backend.new_orders.fetch_add(self.pending, Ordering::Relaxed);
        self.pending = 0;
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), Failed> {
        self.pending = 0;
        Ok(())
    }
}
