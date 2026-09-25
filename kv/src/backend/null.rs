//! No engine. Everything but the call: the operations are generated, the keys formatted, a read
//! gets back a row of valid values and checks them, and the latencies are recorded.
//!
//! Closed loop it gives the driver's own ceiling, and open loop its own share of the latency and
//! the CPU. With `--service` and `--stall` it is a fake engine with a known stall, which is how the
//! driver shows it can see a stall it put there itself, the self-test of the YCSB spec's section
//! 4.12.

use std::time::{Duration, Instant};

use super::{Backend, Failed, Session, Values};
use crate::workload::value;

/// A fake engine.
#[derive(Debug)]
pub(crate) struct NullBackend {
    /// How long every call spins.
    service: Duration,
    /// How long the whole engine stops, and how often, from when it was opened.
    stall: Option<(Duration, Duration)>,
    opened: Instant,
}

impl NullBackend {
    pub(crate) fn new(service: Duration, stall: Option<(Duration, Duration)>) -> Self {
        Self { service, stall, opened: Instant::now() }
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

    fn connect(&self) -> Result<Box<dyn Session + '_>, String> {
        Ok(Box::new(NullSession { backend: self, reads: Vec::new() }))
    }
}

struct NullSession<'a> {
    backend: &'a NullBackend,
    /// For each prepared statement, whether it is a read, which gets a row back.
    reads: Vec<bool>,
}

impl Session for NullSession<'_> {
    fn prepare(&mut self, sql: &str) -> Result<usize, String> {
        self.reads.push(sql.trim_start().to_ascii_uppercase().starts_with("SELECT"));
        Ok(self.reads.len() - 1)
    }

    fn execute(
        &mut self,
        statement: usize,
        parameters: &[&str],
        out: &mut Values,
    ) -> Result<u64, Failed> {
        self.backend.serve();
        if self.reads[statement] {
            let key = parameters.first().copied().unwrap_or_default();
            out.push(key.as_bytes());
            for field in 0..10 {
                out.push(value(key, field, 0).as_bytes());
            }
        }
        Ok(1)
    }

    fn batch(&mut self, _sql: &str) -> Result<(), Failed> {
        Ok(())
    }
}
