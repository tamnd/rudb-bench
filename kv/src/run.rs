//! The clients: one thread and one session each, a barrier, a warmup, a window, and the histograms.
//!
//! Closed loop, a client issues its next operation when the last one returns, and the latency is
//! the call. Open loop, each client has a schedule of intended starts that does not care how fast
//! the engine answers, `s_i = t0 + phase + i * c / rate`, waits for `s_i` when early, never skips
//! when late, and records the latency from `s_i`. That is the YCSB spec's section 4.4, and it is the
//! only mode whose latencies are published, because a closed loop does not send the operations a
//! stalled engine would have been sent, and so never records them.
//!
//! Every client records into its own histograms and never touches another's. Once a second of
//! intended time it writes out what it has for that second and starts again, so no operation is
//! split between two seconds and no client ever waits for another.

use std::fmt::Write as _;
use std::sync::Barrier;
use std::time::{Duration, Instant};

use rudb_bench::histogram::Histogram;

use crate::backend::{Backend, Failed, Session, Values};
use crate::workload::{FIELDS, Mix, Op, Rng, Texts, key, value, well_formed};

/// When a client starts its operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Loop {
    /// Each one when the one before returns.
    Closed,
    /// On a schedule at `rate` operations a second over all clients, with fixed gaps or, when
    /// `poisson`, exponential ones of the same mean.
    Open { rate: f64, poisson: bool },
}

/// Everything a run needs to know.
#[derive(Debug, Clone)]
pub(crate) struct Plan {
    pub(crate) records: u64,
    pub(crate) clients: usize,
    pub(crate) mix: Mix,
    pub(crate) mode: Loop,
    pub(crate) warmup: Duration,
    pub(crate) window: Duration,
    pub(crate) seed: u64,
}

/// A statement gives up after this many retries in a row and counts as failed.
const RETRIES: u32 = 100;

/// Lateness above this in an interval is what the overload rule counts.
const LATE: Duration = Duration::from_millis(1);

/// What one client measured.
#[derive(Debug)]
pub(crate) struct Client {
    /// One per operation, over the window.
    pub(crate) finals: Vec<Histogram>,
    /// Start minus intended start, over the window, open loop only.
    pub(crate) lateness: Histogram,
    pub(crate) retries: [u64; 3],
    pub(crate) failed: [u64; 3],
    /// Reads that came back with a value that was not the row or the field asked for.
    pub(crate) bad: u64,
    pub(crate) reads: u64,
    /// `(second, op, operations, late, histogram line)` for every second the client did anything.
    pub(crate) intervals: Vec<(u64, usize, u64, u64, String)>,
    /// The first error that was not a retry, if any.
    pub(crate) error: Option<String>,
}

/// The prepared statements of one session.
struct Prepared {
    read: usize,
    update: Vec<usize>,
    insert: usize,
}

fn prepare(session: &mut dyn Session, texts: &Texts) -> Result<Prepared, String> {
    let read = session.prepare(&texts.read)?;
    let update = texts.update.iter().map(|text| session.prepare(text)).collect::<Result<_, _>>()?;
    let insert = session.prepare(&texts.insert)?;
    Ok(Prepared { read, update, insert })
}

/// Runs one statement, retrying what the engine says to retry.
fn attempt(
    session: &mut dyn Session,
    statement: usize,
    parameters: &[&str],
    out: &mut Values,
    retries: &mut u64,
) -> Result<u64, Failed> {
    let mut tries = 0;
    loop {
        out.clear();
        match session.execute(statement, parameters, out) {
            Err(Failed::Retry(message)) => {
                tries += 1;
                *retries += 1;
                if tries >= RETRIES {
                    return Err(Failed::Retry(message));
                }
            }
            other => return other,
        }
    }
}

/// Starts a transaction, waiting out an engine that says another writer holds the lock.
fn begin(session: &mut dyn Session, retries: &mut u64) -> Result<(), Failed> {
    let mut tries = 0;
    loop {
        match session.begin() {
            Err(Failed::Retry(message)) => {
                tries += 1;
                *retries += 1;
                if tries >= RETRIES {
                    return Err(Failed::Retry(message));
                }
            }
            other => return other,
        }
    }
}

/// Loads `records` rows, each client inserting every `clients`th one in transactions of a thousand.
pub(crate) fn load(
    backend: &dyn Backend,
    records: u64,
    clients: usize,
) -> Result<Duration, String> {
    let mut first = backend.connect()?;
    let create = backend.create_table();
    if !create.is_empty() {
        first.batch(&create).map_err(|failed| failed.to_string())?;
    }
    drop(first);
    let texts = Texts::new(backend.placeholder());
    let mut sessions = Vec::with_capacity(clients);
    for _ in 0..clients {
        let mut session = backend.connect()?;
        let insert = session.prepare(&texts.insert)?;
        sessions.push((session, insert));
    }
    let started = Instant::now();
    let outcome: Result<(), String> = std::thread::scope(|scope| {
        let handles: Vec<_> = sessions
            .into_iter()
            .enumerate()
            .map(|(client, (mut session, insert))| {
                scope.spawn(move || -> Result<(), String> {
                    let mut out = Values::default();
                    let mut retries = 0;
                    let mut n = client as u64;
                    while n < records {
                        begin(&mut *session, &mut retries)
                            .map_err(|failed| format!("BEGIN failed: {failed}"))?;
                        let mut in_batch = 0;
                        while n < records && in_batch < 1000 {
                            let row_key = key(n);
                            let fields: Vec<String> =
                                (0..FIELDS).map(|field| value(&row_key, field, 0)).collect();
                            let mut parameters: Vec<&str> = vec![&row_key];
                            parameters.extend(fields.iter().map(String::as_str));
                            if let Err(failed) =
                                attempt(&mut *session, insert, &parameters, &mut out, &mut retries)
                            {
                                let _ = session.rollback();
                                return Err(format!("an INSERT failed: {failed}"));
                            }
                            n += clients as u64;
                            in_batch += 1;
                        }
                        session.commit().map_err(|failed| format!("COMMIT failed: {failed}"))?;
                    }
                    Ok(())
                })
            })
            .collect();
        handles.into_iter().try_for_each(|handle| {
            handle.join().map_err(|_| "a loading client panicked".to_string())?
        })
    });
    outcome?;
    Ok(started.elapsed())
}

/// Sleeps most of the way to `until` and spins the rest, because a sleep alone wakes tens of
/// microseconds late and that would be charged to the engine as latency.
fn wait_until(until: Instant) {
    loop {
        let now = Instant::now();
        if now >= until {
            return;
        }
        let left = until - now;
        if left > Duration::from_micros(200) {
            std::thread::sleep(left - Duration::from_micros(100));
        } else {
            std::hint::spin_loop();
        }
    }
}

/// One client's run.
struct Running<'p> {
    plan: &'p Plan,
    client: usize,
    rng: Rng,
    out: Values,
    result: Client,
    /// The second of intended time the interval histograms are for.
    second: u64,
    interval: Vec<Histogram>,
    interval_late: [u64; 3],
    inserted: u64,
    version: u64,
}

impl Running<'_> {
    fn flush(&mut self) {
        for op in Op::ALL {
            let histogram = &mut self.interval[op.index()];
            if histogram.count() > 0 {
                self.result.intervals.push((
                    self.second,
                    op.index(),
                    histogram.count(),
                    self.interval_late[op.index()],
                    histogram.to_line(),
                ));
                histogram.reset();
            }
        }
        self.interval_late = [0; 3];
    }

    /// Does one operation and says whether it got through.
    fn operate(&mut self, session: &mut dyn Session, prepared: &Prepared, op: Op) -> bool {
        let retries = &mut self.result.retries[op.index()];
        let outcome = match op {
            Op::Read => {
                let row_key = key(self.rng.below(self.plan.records));
                let outcome = attempt(session, prepared.read, &[&row_key], &mut self.out, retries);
                if outcome.is_ok() {
                    self.result.reads += 1;
                    let good = self.out.len() == FIELDS + 1
                        && self.out.get(0) == row_key.as_bytes()
                        && (0..FIELDS)
                            .all(|field| well_formed(&row_key, field, self.out.get(field + 1)));
                    if !good {
                        self.result.bad += 1;
                    }
                }
                outcome
            }
            Op::Update => {
                let row_key = key(self.rng.below(self.plan.records));
                let field = self.rng.below(FIELDS as u64) as usize;
                self.version += 1;
                let version = (self.client as u64) << 40 | self.version;
                let new = value(&row_key, field, version);
                attempt(session, prepared.update[field], &[&new, &row_key], &mut self.out, retries)
            }
            Op::Insert => {
                // Past the loaded records, and never the same key from two clients.
                let n = self.plan.records
                    + self.client as u64
                    + self.inserted * self.plan.clients as u64;
                self.inserted += 1;
                let row_key = key(n);
                let fields: Vec<String> =
                    (0..FIELDS).map(|field| value(&row_key, field, 0)).collect();
                let mut parameters: Vec<&str> = vec![&row_key];
                parameters.extend(fields.iter().map(String::as_str));
                attempt(session, prepared.insert, &parameters, &mut self.out, retries)
            }
        };
        match outcome {
            Ok(_) => true,
            Err(failed) => {
                self.result.failed[op.index()] += 1;
                if let Failed::Error(message) = failed {
                    self.result.error.get_or_insert(message);
                }
                false
            }
        }
    }

    fn run(mut self, session: &mut dyn Session, prepared: &Prepared, t0: Instant) -> Client {
        let plan = self.plan;
        let measured = t0 + plan.warmup;
        let end = measured + plan.window;
        let gap = match plan.mode {
            Loop::Closed => Duration::ZERO,
            Loop::Open { rate, .. } => Duration::from_secs_f64(plan.clients as f64 / rate),
        };
        let mut intended = t0 + gap.mul_f64(self.rng.next_f64());
        loop {
            let start = match plan.mode {
                Loop::Closed => {
                    intended = Instant::now();
                    intended
                }
                Loop::Open { .. } => {
                    wait_until(intended);
                    Instant::now()
                }
            };
            if intended >= end {
                break;
            }
            let op = plan.mix.pick(&mut self.rng);
            let done = self.operate(session, prepared, op);
            let finished = Instant::now();
            let latency = u64::try_from((finished - intended).as_nanos()).unwrap_or(u64::MAX);
            let late = start - intended;
            let second = (intended - t0).as_secs();
            if second != self.second {
                self.flush();
                self.second = second;
            }
            if done {
                self.interval[op.index()].record(latency);
                if late > LATE {
                    self.interval_late[op.index()] += 1;
                }
                if intended >= measured {
                    self.result.finals[op.index()].record(latency);
                    if let Loop::Open { .. } = plan.mode {
                        self.result
                            .lateness
                            .record(u64::try_from(late.as_nanos()).unwrap_or(u64::MAX));
                    }
                }
            }
            if let Loop::Open { poisson, .. } = plan.mode {
                let step = if poisson {
                    // An exponential gap of the same mean, 1 - u so the logarithm never sees zero.
                    gap.mul_f64(-(1.0 - self.rng.next_f64()).ln())
                } else {
                    gap
                };
                intended += step;
            }
        }
        self.flush();
        self.result
    }
}

/// What the whole run measured, and the process's CPU over the window.
#[derive(Debug)]
pub(crate) struct Outcome {
    pub(crate) clients: Vec<Client>,
    pub(crate) window_usage: (crate::ffi::Usage, crate::ffi::Usage),
}

/// Opens every session, prepares every statement, and then runs all the clients at once.
pub(crate) fn run(backend: &dyn Backend, plan: &Plan) -> Result<Outcome, String> {
    let texts = Texts::new(backend.placeholder());
    let mut sessions = Vec::with_capacity(plan.clients);
    for _ in 0..plan.clients {
        let mut session = backend.connect()?;
        let prepared = prepare(&mut *session, &texts)?;
        sessions.push((session, prepared));
    }
    let barrier = Barrier::new(plan.clients + 1);
    std::thread::scope(|scope| {
        let barrier = &barrier;
        let handles: Vec<_> = sessions
            .into_iter()
            .enumerate()
            .map(|(client, (mut session, prepared))| {
                let running = Running {
                    plan,
                    client,
                    rng: Rng::new(plan.seed ^ (client as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)),
                    out: Values::default(),
                    result: Client {
                        finals: vec![Histogram::new(); 3],
                        lateness: Histogram::new(),
                        retries: [0; 3],
                        failed: [0; 3],
                        bad: 0,
                        reads: 0,
                        intervals: Vec::new(),
                        error: None,
                    },
                    second: 0,
                    interval: vec![Histogram::new(); 3],
                    interval_late: [0; 3],
                    inserted: 0,
                    version: 0,
                };
                scope.spawn(move || {
                    barrier.wait();
                    // Every client reads the same start, set a little ahead so that none of them
                    // is already late for its first operation.
                    let t0 = *START.get().expect("the start is set before the barrier opens");
                    running.run(&mut *session, &prepared, t0)
                })
            })
            .collect();
        let t0 = Instant::now() + Duration::from_millis(20);
        let _ = START.set(t0);
        barrier.wait();
        std::thread::sleep((t0 + plan.warmup).saturating_duration_since(Instant::now()));
        let at_window = crate::ffi::usage();
        let clients = handles
            .into_iter()
            .map(|handle| handle.join().map_err(|_| "a client panicked".to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        let at_end = crate::ffi::usage();
        Ok(Outcome { clients, window_usage: (at_window, at_end) })
    })
}

/// When the clients start. One run per process, so a process-wide cell is enough.
static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

/// The driver's text for everything the clients measured, the YCSB spec's section 4.11.
pub(crate) fn report(outcome: &Outcome, plan: &Plan) -> String {
    let mut text = String::new();
    let mut seconds: Vec<(u64, usize, u64, u64, Histogram)> = Vec::new();
    for client in &outcome.clients {
        for (second, op, n, late, line) in &client.intervals {
            let histogram = Histogram::from_line(line).expect("a line this process wrote");
            match seconds.iter_mut().find(|(s, o, ..)| s == second && o == op) {
                Some(entry) => {
                    entry.2 += n;
                    entry.3 += late;
                    entry.4.merge(&histogram);
                }
                None => seconds.push((*second, *op, *n, *late, histogram)),
            }
        }
    }
    seconds.sort_by_key(|(second, op, ..)| (*second, *op));
    for (second, op, n, late, histogram) in &seconds {
        let name = Op::ALL[*op].name();
        let _ = writeln!(
            text,
            "interval {second} {name} n={n} late1ms={late} hist={}",
            histogram.to_line()
        );
    }
    let mut lateness = Histogram::new();
    for op in Op::ALL {
        let mut histogram = Histogram::new();
        let (mut retries, mut failed) = (0, 0);
        for client in &outcome.clients {
            histogram.merge(&client.finals[op.index()]);
            retries += client.retries[op.index()];
            failed += client.failed[op.index()];
        }
        if histogram.count() > 0 {
            let micros = |q: f64| histogram.quantile(q) as f64 / 1e3;
            let _ = writeln!(
                text,
                "summary {} ops_per_s={:.0} p50_us={:.1} p99_us={:.1} p999_us={:.1} max_us={:.1}",
                op.name(),
                histogram.count() as f64 / plan.window.as_secs_f64(),
                micros(0.5),
                micros(0.99),
                micros(0.999),
                histogram.max() as f64 / 1e3,
            );
        }
        if histogram.count() > 0 || failed > 0 {
            let _ = writeln!(
                text,
                "final {} n={} retries={retries} failed={failed} hist={}",
                op.name(),
                histogram.count(),
                histogram.to_line()
            );
        }
    }
    for client in &outcome.clients {
        lateness.merge(&client.lateness);
    }
    if let Loop::Open { .. } = plan.mode {
        let _ = writeln!(text, "final late n={} hist={}", lateness.count(), lateness.to_line());
    }
    text
}
