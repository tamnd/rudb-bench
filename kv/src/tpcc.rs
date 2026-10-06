//! The `tpcc` mode: the spec's TPC-C suite, documents 02 to 05, closed loop.
//!
//! It loads the nine tables if asked, opens one session per terminal with every statement of the
//! five transactions prepared, runs the terminals for a warmup and a window, counts New-Orders
//! twice (once in the driver, once from `sum(d_next_o_id)` by a monitor session), and then runs the
//! checks of document 05 against what is left in the database.

mod checks;
mod data;
mod terminal;
pub(crate) mod text;

use std::fmt::Write as _;
use std::sync::Barrier;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use rudb_bench::histogram::Histogram;

use crate::backend::{Backend, Level};
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

/// Runs the mode and prints its report.
pub(crate) fn drive(backend: &dyn Backend, settings: &Settings) -> Result<(), String> {
    let constants = Constants::new(settings.seed);
    let terminals = settings.terminals;
    let warehouses = settings.warehouses;
    println!(
        "tpcc 1 backend={} version={} warehouses={warehouses} terminals={terminals} loop=closed \
         seed={:#x} c_load={} c_run={} c_id={} c_item={} level={} warmup_s={} window_s={} pinned={} \
         boundary={}",
        backend.name(),
        backend.version().replace(' ', "_"),
        settings.seed,
        constants.c_load,
        constants.c_run,
        constants.c_id,
        constants.c_item,
        settings.level.name(),
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
        println!(
            "load method=insert statement_rows=500 transaction=unit sessions={terminals} \
             create_s={:.3}",
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

    let mut runners = Vec::with_capacity(terminals);
    for _ in 0..terminals {
        let mut session = backend.connect()?;
        let prepared = terminal::prepare(&mut *session, backend)?;
        runners.push(Runner::new(session, prepared));
    }
    let mut monitor = {
        let mut session = backend.connect()?;
        let prepared = terminal::prepare(&mut *session, backend)?;
        Runner::new(session, prepared)
    };

    let seconds = settings.window.as_secs_f64().ceil() as usize;
    let stop = AtomicBool::new(false);
    let barrier = Barrier::new(terminals + 1);
    let start = Instant::now() + Duration::from_millis(20);
    let clock = Clock::new(start, settings.window);
    let (tallies, usage, orders) = std::thread::scope(|scope| -> Result<_, String> {
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
        barrier.wait();
        sleep_until(start + settings.warmup, &stop);
        let at_window = crate::ffi::usage();
        let first = monitor.orders();
        let measured = Instant::now();
        clock.open(measured);
        sleep_until(measured + settings.window, &stop);
        let last = monitor.orders();
        let at_end = crate::ffi::usage();
        let tallies = handles
            .into_iter()
            .map(|handle| handle.join().map_err(|_| "a terminal panicked".to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok((tallies, (at_window, at_end), first.and_then(|first| last.map(|last| (first, last)))))
    })?;
    drop(monitor);

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
