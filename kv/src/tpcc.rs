//! The `tpcc` mode: the spec's TPC-C suite, documents 02 to 05, closed loop.
//!
//! It loads the twelve tables if asked, opens one session per terminal with every statement of the
//! five transactions prepared, runs the terminals for a warmup and a window, counts New-Orders
//! twice (once in the driver, once from `sum(d_next_o_id)` by a monitor session), and then runs the
//! checks of document 05 against what is left in the database.
//!
//! With `--analytic N` it is the mixed run of CH-benCHmark, document 09 section 9.5: `N` streams run
//! the 22 analytic queries over and over beside the terminals. [`reference`] is the analytic run on
//! its own, section 9.4. With `--forgotten` a session holds one transaction open through the
//! window, the run that decides what writers do while undo is held past its share.

mod ch;
mod checks;
mod data;
mod fresh;
mod terminal;
pub(crate) mod text;

use std::fmt::Write as _;
use std::sync::Barrier;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use rudb_bench::histogram::Histogram;

use crate::backend::{Backend, Level, Session, Values};
pub(crate) use data::{CUSTOMERS, ITEMS};
use terminal::{Clock, Constants, Deck, Home, Kind, Runner, Tally, Terminal};

/// Everything the mode is run with.
#[derive(Debug, Clone)]
pub(crate) struct Settings {
    pub(crate) warehouses: u64,
    pub(crate) terminals: usize,
    pub(crate) warmup: Duration,
    pub(crate) window: Duration,
    pub(crate) seed: u64,
    pub(crate) load: bool,
    pub(crate) level: Level,
    pub(crate) pinned: bool,
    pub(crate) card: Option<String>,
    /// Whether every transaction begins `ISOLATION LEVEL SERIALIZABLE` rather than at the
    /// engine's default level.
    pub(crate) serializable: bool,
    /// How many CH-benCHmark analytic streams run beside the terminals.
    pub(crate) analytic: usize,
    /// How many threads each analytic stream's queries may take.
    pub(crate) analytic_threads: usize,
    /// Whether a session holds a transaction open from the start of the window to its end.
    pub(crate) forgotten: bool,
    /// Whether rudb shows a commit's rows once it is ordered, before it is durable.
    pub(crate) committed: bool,
}

/// The transaction `--forgotten` leaves open: it reads once, so it holds a snapshot, and nothing
/// after that until the window ends.
struct Forgotten<'s> {
    session: Box<dyn Session + 's>,
    read: usize,
}

impl<'s> Forgotten<'s> {
    fn open(backend: &'s dyn Backend) -> Result<Self, String> {
        let mut session = backend.connect()?;
        let read = session.prepare("SELECT count(*) FROM warehouse")?;
        Ok(Self { session, read })
    }

    /// Begins it and reads, and says when.
    fn begin(&mut self) -> Result<Instant, String> {
        self.session.begin_snapshot().map_err(|failed| format!("BEGIN failed: {failed}"))?;
        self.session
            .execute(self.read, &[], &mut Values::default())
            .map_err(|failed| format!("the read failed: {failed}"))?;
        Ok(Instant::now())
    }

    /// Rolls it back, and says how long it was held.
    fn end(&mut self, began: Instant) -> Result<Duration, String> {
        let held = began.elapsed();
        self.session.rollback().map_err(|failed| format!("ROLLBACK failed: {failed}"))?;
        Ok(held)
    }
}

/// The mean transactions a second over each `every` seconds of `per_second`, the last span
/// perhaps shorter.
fn timeline(per_second: &[u64], every: usize) -> Vec<f64> {
    per_second
        .chunks(every.max(1))
        .map(|span| span.iter().sum::<u64>() as f64 / span.len() as f64)
        .collect()
}

/// The share of any type's transactions that may fail before the run is invalid, 0.1%.
const FAILED_SHARE: f64 = 0.001;

/// Sleeps until `until`, or until a terminal has stopped the run.
fn sleep_until(until: Instant, stop: &AtomicBool) {
    while !stop.load(std::sync::atomic::Ordering::Relaxed) {
        let now = Instant::now();
        if now >= until {
            return;
        }
        std::thread::sleep((until - now).min(Duration::from_millis(50)));
    }
}

/// The mean and the coefficient of variation of `values`.
fn spread(values: &[u64]) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0);
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<u64>() as f64 / n;
    let variance = values.iter().map(|v| (*v as f64 - mean).powi(2)).sum::<f64>() / n;
    let cv = if mean > 0.0 { variance.sqrt() / mean } else { 0.0 };
    (mean, cv)
}

/// The line that says how the analytic side runs and with which texts.
fn ch_line(streams: usize, threads: usize) -> String {
    format!(
        "ch streams={streams} threads={threads} texts_sha256={} benchbase={} shift_years={}",
        ch::texts_sha256(),
        ch::BENCHBASE,
        ch::shift_years(),
    )
}

/// Prints what the load took.
fn print_load(loaded: &data::Loaded, sessions: usize) {
    println!(
        "load method=insert statement_rows=500 transaction=unit sessions={sessions} create_s={:.3}",
        loaded.create.as_secs_f64()
    );
    let mut total = loaded.create;
    for (table, rows, took) in &loaded.tables {
        total += *took;
        println!("load table={table} rows={rows} took_s={:.3}", took.as_secs_f64());
    }
    for (index, took) in &loaded.indexes {
        total += *took;
        println!("load index={index} took_s={:.3}", took.as_secs_f64());
    }
    println!("load total_s={:.3}", total.as_secs_f64());
}

/// Opens `count` analytic streams and prints the queries the engine could not prepare, once.
fn open_streams<'b>(
    backend: &'b dyn Backend,
    count: usize,
    settings: &Settings,
) -> Result<Vec<ch::Stream<'b>>, String> {
    let mut streams = Vec::with_capacity(count);
    for at in 0..count {
        let stream = ch::Stream::open(backend, at, settings.analytic_threads, settings.seed)?;
        if at == 0 {
            for (query, why) in stream.unprepared() {
                println!("ch unprepared query=Q{} message={why:?}", query + 1);
            }
        }
        streams.push(stream);
    }
    Ok(streams)
}

/// The analytic reference of document 09 section 9.4: each of the 22 queries `runs` times in turn
/// on one session, over the tables as they are, after loading them if asked.
pub(crate) fn reference(
    backend: &dyn Backend,
    settings: &Settings,
    runs: usize,
) -> Result<(), String> {
    println!(
        "ch 1 backend={} version={} warehouses={} runs={runs} seed={:#x} level={} boundary={}",
        backend.name(),
        backend.version().replace(' ', "_"),
        settings.warehouses,
        settings.seed,
        settings.level.name(),
        backend.boundary(),
    );
    match &settings.card {
        Some(card) => println!("card {}", card.strip_prefix("card ").unwrap_or(card)),
        None => println!("card none"),
    }
    if settings.load {
        let loaded = data::load(backend, settings.seed, settings.warehouses, settings.terminals)?;
        print_load(&loaded, settings.terminals);
    }
    println!("{}", ch_line(1, settings.analytic_threads));
    let mut streams = open_streams(backend, 1, settings)?;
    let started = Instant::now();
    let ran = streams.remove(0).each(runs);
    let mut report = String::new();
    ch::report(&[ran], None, &mut report);
    print!("{report}");
    println!("ch took_s={:.3}", started.elapsed().as_secs_f64());
    println!("end");
    Ok(())
}

/// Runs the mode and prints its report.
pub(crate) fn drive(backend: &dyn Backend, settings: &Settings) -> Result<(), String> {
    let constants = Constants::new(settings.seed);
    let terminals = settings.terminals;
    let warehouses = settings.warehouses;
    println!(
        "tpcc 1 backend={} version={} warehouses={warehouses} terminals={terminals} loop=closed \
         seed={:#x} c_load={} c_run={} c_id={} c_item={} level={} isolation={} visibility={} warmup_s={} window_s={} pinned={} \
         boundary={}",
        backend.name(),
        backend.version().replace(' ', "_"),
        settings.seed,
        constants.c_load,
        constants.c_run,
        constants.c_id,
        constants.c_item,
        settings.level.name(),
        if settings.serializable { "serializable" } else { "default" },
        if settings.committed { "committed" } else { "durable" },
        settings.warmup.as_secs_f64(),
        settings.window.as_secs_f64(),
        if settings.pinned { "yes" } else { "no" },
        backend.boundary(),
    );
    if let Some(note) = backend.level_note() {
        println!("note {note}");
    }
    match &settings.card {
        Some(card) => println!("card {}", card.strip_prefix("card ").unwrap_or(card)),
        None => println!("card none"),
    }
    let keeps_rows = backend.keeps_rows();
    let mut verdicts: Vec<String> = Vec::new();
    let mut loaded_lines = None;
    if settings.load {
        let loaded = data::load(backend, settings.seed, warehouses, terminals)?;
        print_load(&loaded, terminals);
        let lines = data::loaded_lines(settings.seed, warehouses);
        loaded_lines = Some(lines);
        if keeps_rows {
            let since =
                checks::Since { warehouses, loaded_lines: lines, ..checks::Since::default() };
            let mut report = String::new();
            let passed = checks::run(backend, "load", Some(&since), &mut report)?;
            print!("{report}");
            if !passed {
                verdicts.push("load_incorrect".to_string());
            }
        }
    }

    // The streams judge each query's freshness against the New-Orders the terminals were told
    // had committed, document 09 section 9.6.
    let acks = (settings.analytic > 0).then(|| fresh::Acks::new(warehouses));
    let mut runners = Vec::with_capacity(terminals);
    for _ in 0..terminals {
        let mut session = backend.connect()?;
        let prepared = terminal::prepare(&mut *session, backend)?;
        runners.push(Runner::new(session, prepared, settings.serializable).acking(acks.as_ref()));
    }
    let mut monitor = {
        let mut session = backend.connect()?;
        let prepared = terminal::prepare(&mut *session, backend)?;
        Runner::new(session, prepared, settings.serializable)
    };
    if settings.analytic > 0 {
        println!("{}", ch_line(settings.analytic, settings.analytic_threads));
    }
    let mut streams = open_streams(backend, settings.analytic, settings)?;
    let mut forgotten = settings.forgotten.then(|| Forgotten::open(backend)).transpose()?;
    if let Some(acks) = &acks {
        for (at, stream) in streams.iter_mut().enumerate() {
            if let Err(why) = stream.judging(acks)
                && at == 0
            {
                println!("ch unjudged message={why:?}");
            }
        }
    }

    let seconds = settings.window.as_secs_f64().ceil() as usize;
    let stop = AtomicBool::new(false);
    let barrier = Barrier::new(terminals + streams.len() + 1);
    let start = Instant::now() + Duration::from_millis(20);
    let clock = Clock::new(start, settings.window);
    let (tallies, usage, orders, analytic, held) =
        std::thread::scope(|scope| -> Result<_, String> {
            let handles: Vec<_> = runners
                .into_iter()
                .enumerate()
                .map(|(t, runner)| {
                    let t = t as u64;
                    let terminal = Terminal {
                        runner,
                        rng: data::stream(settings.seed, 200, t, 0),
                        backoff: data::stream(settings.seed, 201, t, 0),
                        deck: Deck::new(),
                        home: Home::of(t, terminals as u64, warehouses),
                        warehouses,
                        constants,
                    };
                    let (stop, barrier, clock) = (&stop, &barrier, &clock);
                    scope.spawn(move || {
                        barrier.wait();
                        terminal.run(clock, seconds, stop)
                    })
                })
                .collect();
            let streams: Vec<_> = streams
                .into_iter()
                .map(|stream| {
                    let (stop, barrier, clock) = (&stop, &barrier, &clock);
                    scope.spawn(move || {
                        barrier.wait();
                        stream.run(clock, stop)
                    })
                })
                .collect();
            barrier.wait();
            sleep_until(start + settings.warmup, &stop);
            let at_window = crate::ffi::usage();
            let first = monitor.orders();
            let measured = Instant::now();
            clock.open(measured);
            let began = forgotten.as_mut().map(Forgotten::begin);
            sleep_until(measured + settings.window, &stop);
            let held = forgotten
                .as_mut()
                .zip(began)
                .map(|(forgotten, began)| began.and_then(|began| forgotten.end(began)));
            let last = monitor.orders();
            let at_end = crate::ffi::usage();
            let tallies = handles
                .into_iter()
                .map(|handle| handle.join().map_err(|_| "a terminal panicked".to_string()))
                .collect::<Result<Vec<_>, _>>()?;
            let analytic = streams
                .into_iter()
                .map(|handle| handle.join().map_err(|_| "an analytic stream panicked".to_string()))
                .collect::<Result<Vec<_>, _>>()?;
            let orders = first.and_then(|first| last.map(|last| (first, last)));
            Ok((tallies, (at_window, at_end), orders, analytic, held))
        })?;
    drop(monitor);
    drop(forgotten);

    let mut tally = Tally::new(seconds);
    for one in &tallies {
        tally.merge(one);
    }
    let window_s = settings.window.as_secs_f64();
    let mut report = String::new();
    for (second, counts) in tally.seconds.iter().enumerate() {
        let _ = write!(report, "interval {second}");
        for kind in Kind::ALL {
            let _ = write!(report, " {}={}", kind.name(), counts[kind.index()]);
        }
        report.push('\n');
    }
    for kind in Kind::ALL {
        let k = kind.index();
        let histogram: &Histogram = &tally.service[k];
        let committed = tally.window[k];
        let retries =
            tally.window_attempts[k].saturating_sub(committed + tally.window_rolled_back[k]);
        let retry_rate = if committed > 0 { retries as f64 / committed as f64 } else { 0.0 };
        let micros = |q: f64| histogram.quantile(q) as f64 / 1e3;
        let _ = writeln!(
            report,
            "final {} n={committed} attempts={} rolled_back={} retries={retries} \
             retry_rate={retry_rate:.4} failed={} per_s={:.1} p50_us={:.1} p99_us={:.1} \
             max_us={:.1} hist={}",
            kind.name(),
            tally.window_attempts[k],
            tally.window_rolled_back[k],
            tally.failed[k],
            committed as f64 / window_s,
            micros(0.5),
            micros(0.99),
            histogram.max() as f64 / 1e3,
            histogram.to_line(),
        );
        let ended = tally.committed[k] + tally.rolled_back[k] + tally.failed[k];
        if tally.failed[k] as f64 > FAILED_SHARE * ended as f64 {
            verdicts.push(format!("{}_failed={}", kind.name(), tally.failed[k]));
        }
    }
    let new_orders = tally.window[Kind::NewOrder.index()];
    match &orders {
        Ok((first, last)) => {
            let database = last - first;
            let differ = database.abs_diff(new_orders as i64);
            let limit = 2 * terminals as u64;
            let agree = differ <= limit;
            let _ = writeln!(
                report,
                "monitor start={first} end={last} database={database} driver={new_orders} \
                 differ={differ} limit={limit} agree={}",
                if agree { "yes" } else { "no" }
            );
            if !agree {
                verdicts.push("nopm_disagree".to_string());
            }
        }
        Err(message) => {
            let _ = writeln!(report, "monitor error={message:?}");
            verdicts.push("monitor_failed".to_string());
        }
    }
    let per_second: Vec<u64> =
        tally.seconds.iter().map(|counts| counts.iter().sum::<u64>()).collect();
    let (mean, cv) = spread(&per_second);
    let committed: u64 = tally.window.iter().sum();
    let dealt: u64 = committed + tally.window_rolled_back.iter().sum::<u64>();
    let mix: Vec<String> = Kind::ALL
        .iter()
        .map(|kind| {
            let k = kind.index();
            let share =
                (tally.window[k] + tally.window_rolled_back[k]) as f64 / dealt.max(1) as f64;
            format!("{}={:.1}", kind.name(), share * 100.0)
        })
        .collect();
    let _ = writeln!(
        report,
        "summary nopm={:.0} nopm_database={} tx_per_s={:.1} mean_tx_per_s={mean:.1} \
         min_tx_per_s={} cv={cv:.3} mix={}",
        new_orders as f64 * 60.0 / window_s,
        orders.as_ref().map_or_else(
            |_| "unknown".to_string(),
            |(first, last)| format!("{:.0}", (last - first) as f64 * 60.0 / window_s)
        ),
        committed as f64 / window_s,
        per_second.iter().min().copied().unwrap_or(0),
        mix.join(","),
    );
    match held {
        Some(Ok(held)) => {
            let spans: Vec<String> =
                timeline(&per_second, 10).iter().map(|tx| format!("{tx:.1}")).collect();
            let _ = writeln!(
                report,
                "forgotten held_s={:.3} every_s=10 tx_per_s={}",
                held.as_secs_f64(),
                spans.join(",")
            );
        }
        Some(Err(message)) => {
            let _ = writeln!(report, "forgotten error={message:?}");
            verdicts.push("forgotten_failed".to_string());
        }
        None => {}
    }
    let effects = tally.effects;
    let since = checks::Since {
        warehouses,
        loaded_lines: loaded_lines.unwrap_or(0),
        new_orders: tally.committed[Kind::NewOrder.index()],
        lines: effects.lines,
        quantity: effects.quantity,
        remote: effects.remote,
        payments: tally.committed[Kind::Payment.index()],
        delivered: effects.delivered,
    };
    let _ = writeln!(
        report,
        "counts n_no={} l={} q={} r={} n_pay={} n_del={} d={} skipped={} rolled_back={}",
        since.new_orders,
        since.lines,
        since.quantity,
        since.remote,
        since.payments,
        tally.committed[Kind::Delivery.index()],
        since.delivered,
        effects.skipped,
        tally.rolled_back[Kind::NewOrder.index()],
    );
    if !analytic.is_empty() {
        ch::report(&analytic, Some(settings.window), &mut report);
    }
    print!("{report}");
    let (at_window, at_end) = usage;
    let cpu = (at_end.user.saturating_sub(at_window.user)
        + at_end.system.saturating_sub(at_window.system))
    .as_secs_f64();
    println!(
        "cost cpu_user_s={:.3} cpu_sys_s={:.3} peak_rss={} cpu_us_per_tx={:.1}",
        at_end.user.saturating_sub(at_window.user).as_secs_f64(),
        at_end.system.saturating_sub(at_window.system).as_secs_f64(),
        at_end.peak,
        if committed > 0 { cpu * 1e6 / committed as f64 } else { 0.0 },
    );
    if let Some(error) = &tally.error {
        eprintln!("a transaction failed: {error}");
        verdicts.push("error".to_string());
    }
    let checked = if keeps_rows {
        let mut report = String::new();
        let since = loaded_lines.map(|_| since);
        let passed = checks::run(backend, "run", since.as_ref(), &mut report)?;
        print!("{report}");
        if passed {
            "pass"
        } else {
            verdicts.push("check_failed".to_string());
            "fail"
        }
    } else {
        "skipped"
    };
    println!(
        "verify run={} checks={checked} reasons={}",
        if verdicts.is_empty() { "valid" } else { "invalid" },
        if verdicts.is_empty() { "none".to_string() } else { verdicts.join(",") },
    );
    println!("end");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::timeline;

    #[test]
    fn the_timeline_is_the_mean_of_each_span_and_the_last_may_be_short() {
        let per_second = [10, 20, 30, 40, 50, 0, 0];
        assert_eq!(timeline(&per_second, 3), vec![20.0, 30.0, 0.0]);
        assert_eq!(timeline(&[], 10), Vec::<f64>::new());
    }
}
