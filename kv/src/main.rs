//! rudb-bench-kv: the driver the harness starts for the workloads that run for a duration.
//!
//! The harness drives every engine as a command line started once per query, which is right for a
//! query that takes a second and measures the process start forty thousand times over for a point
//! read that takes a microsecond. So a workload like YCSB is run by this program instead. It opens
//! the database once, gives each client thread its own connection with its statements prepared,
//! runs them all at once for a warmup and a window, and prints what it measured as lines of text
//! the harness reads without any dependency. The design is `bench/ycsb/04-the-driver.md` in the
//! spec.
//!
//! It is its own package so the harness keeps zero dependencies and no `unsafe`. The only `unsafe`
//! here is in `ffi.rs`, which loads SQLite, DuckDB and libpq at run time.

#![deny(unsafe_code)]

mod backend;
mod ffi;
mod run;
mod tpcc;
mod workload;

use std::process::ExitCode;
use std::time::Duration;

use backend::duckdb::DuckdbBackend;
use backend::null::NullBackend;
use backend::postgres::PostgresBackend;
use backend::shim::Shim;
use backend::sqlite::SqliteBackend;
use backend::{Backend, Level};
use run::{Loop, Plan};
use workload::{Distribution, Mix};

const USAGE: &str = "\
usage: rudb-bench-kv [--mode ycsb|tpcc|ch] --backend null|sqlite|duckdb|postgres|rudb --target <file or conninfo>
                     [--load] [--warehouses N] [--terminals N] [--records N] [--mix read=50,update=50] [--clients N]
                     [--distribution uniform|zipfian|latest] [--loop closed|open] [--rate OPS] [--poisson] [--warmup 10s] [--window 60s]
                     [--seed N] [--level full|os|none] [--card <card line>] [--unpinned]
                     [--service 100us] [--stall 100ms/10s] [--shim none|unix] [--isolation default|serializable]
                     [--analytic N] [--analytic-threads N] [--forgotten] [--runs N] [--visibility durable|committed]

The libraries come from RUDB_BENCH_LIBSQLITE, RUDB_BENCH_LIBDUCKDB and RUDB_BENCH_LIBPQ.
--service and --stall make the null backend a fake engine with a known stall.
--shim unix puts rudb or the null backend behind a socketpair and a thread per session, so every
statement pays the round trip PostgreSQL pays.
--unpinned lets DuckDB run with a library that is not the pin, and marks the run pinned=no.
--mode tpcc runs the TPC-C suite on --warehouses warehouses with --terminals terminals (the same as
--clients), closed loop. --load creates and loads its twelve tables first, the nine of TPC-C and the
three CH-benCHmark adds. --isolation serializable begins each of its transactions ISOLATION LEVEL
SERIALIZABLE, and SQLite, which is serializable anyway, as it begins them otherwise.
--analytic N runs N CH-benCHmark analytic streams beside the terminals, each letting its queries
take --analytic-threads threads, 2 unless set.
--forgotten opens a transaction that reads once as the window opens and leaves it open to the end,
and prints the throughput of each ten seconds, to see what the writers do while it holds undo.
--mode ch runs each of the 22 CH-benCHmark queries --runs times, 5 unless set, on one session over
the tables as they are, or as --load loads them, which is the reference a mixed run is compared with.
--visibility committed has rudb show a commit's rows to other sessions once it is ordered, before it
is durable. durable, the default, is when every engine shows them, and only rudb takes the other.";

/// The two suites the driver runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Ycsb,
    Tpcc,
    Ch,
}

/// Everything on the command line.
#[derive(Debug)]
struct Arguments {
    mode: Mode,
    warehouses: u64,
    backend: String,
    target: String,
    load: bool,
    plan: Plan,
    level: Level,
    card: Option<String>,
    unpinned: bool,
    service: Duration,
    stall: Option<(Duration, Duration)>,
    shim: bool,
    serializable: bool,
    analytic: usize,
    analytic_threads: usize,
    forgotten: bool,
    runs: usize,
    /// Whether `--visibility committed` asked for a commit's rows to show before it is durable.
    committed: bool,
}

/// `100us`, `5ms`, `10s`, or a bare number of seconds.
fn duration(text: &str) -> Result<Duration, String> {
    let (number, unit) = match text.find(|c: char| c.is_ascii_alphabetic()) {
        Some(at) => text.split_at(at),
        None => (text, "s"),
    };
    let number: f64 = number.parse().map_err(|_| format!("{text:?} is not a duration"))?;
    let seconds = match unit {
        "ns" => number / 1e9,
        "us" => number / 1e6,
        "ms" => number / 1e3,
        "s" => number,
        _ => return Err(format!("{text:?} has a unit that is not ns, us, ms or s")),
    };
    Ok(Duration::from_secs_f64(seconds))
}

fn parse(mut words: impl Iterator<Item = String>) -> Result<Arguments, String> {
    let mut arguments = Arguments {
        mode: Mode::Ycsb,
        warehouses: 1,
        backend: String::new(),
        target: String::new(),
        load: false,
        plan: Plan {
            records: 10_000,
            clients: 1,
            mix: Mix::parse("read=50,update=50")?,
            distribution: Distribution::Uniform,
            mode: Loop::Closed,
            warmup: Duration::from_secs(1),
            window: Duration::from_secs(5),
            seed: 0x5eed,
        },
        level: Level::Full,
        card: None,
        unpinned: false,
        service: Duration::ZERO,
        stall: None,
        shim: false,
        serializable: false,
        analytic: 0,
        analytic_threads: 2,
        committed: false,
        forgotten: false,
        runs: 5,
    };
    let mut rate = None;
    let mut open = false;
    let mut poisson = false;
    while let Some(word) = words.next() {
        let mut value = || words.next().ok_or_else(|| format!("{word} wants a value"));
        match word.as_str() {
            "--mode" => {
                arguments.mode = match value()?.as_str() {
                    "ycsb" => Mode::Ycsb,
                    "tpcc" => Mode::Tpcc,
                    "ch" => Mode::Ch,
                    other => return Err(format!("--mode is ycsb, tpcc or ch, not {other:?}")),
                };
            }
            "--warehouses" => {
                arguments.warehouses = value()?.parse().map_err(|_| "--warehouses is a count")?;
            }
            "--backend" => arguments.backend = value()?,
            "--target" => arguments.target = value()?,
            "--load" => arguments.load = true,
            "--records" => {
                arguments.plan.records = value()?.parse().map_err(|_| "--records is a count")?;
            }
            "--mix" => arguments.plan.mix = Mix::parse(&value()?)?,
            "--distribution" => {
                arguments.plan.distribution = Distribution::parse(&value()?)?;
            }
            "--clients" | "--terminals" => {
                arguments.plan.clients = value()?.parse().map_err(|_| "--clients is a count")?;
            }
            "--loop" => {
                open = match value()?.as_str() {
                    "open" => true,
                    "closed" => false,
                    other => return Err(format!("--loop is open or closed, not {other:?}")),
                };
            }
            "--rate" => {
                rate = Some(value()?.parse::<f64>().map_err(|_| "--rate is operations a second")?);
            }
            "--poisson" => poisson = true,
            "--warmup" => arguments.plan.warmup = duration(&value()?)?,
            "--window" => arguments.plan.window = duration(&value()?)?,
            "--seed" => {
                let text = value()?;
                let parsed = match text.strip_prefix("0x") {
                    Some(hex) => u64::from_str_radix(hex, 16),
                    None => text.parse(),
                };
                arguments.plan.seed = parsed.map_err(|_| "--seed is a number")?;
            }
            "--level" => arguments.level = Level::parse(&value()?)?,
            "--card" => arguments.card = Some(value()?),
            "--unpinned" => arguments.unpinned = true,
            "--service" => arguments.service = duration(&value()?)?,
            "--stall" => {
                let text = value()?;
                let (length, every) = text
                    .split_once('/')
                    .ok_or("--stall is a length and a period, like 100ms/10s")?;
                let (length, every) = (duration(length)?, duration(every)?);
                if length >= every {
                    return Err("--stall has to be shorter than its period".to_string());
                }
                arguments.stall = Some((length, every));
            }
            "--shim" => {
                arguments.shim = match value()?.as_str() {
                    "unix" => true,
                    "none" => false,
                    other => return Err(format!("--shim is none or unix, not {other:?}")),
                };
            }
            "--isolation" => {
                arguments.serializable = match value()?.as_str() {
                    "serializable" => true,
                    "default" => false,
                    other => {
                        return Err(format!(
                            "--isolation is default or serializable, not {other:?}"
                        ));
                    }
                };
            }
            "--analytic" => {
                arguments.analytic = value()?.parse().map_err(|_| "--analytic is a count")?;
            }
            "--analytic-threads" => {
                arguments.analytic_threads =
                    value()?.parse().map_err(|_| "--analytic-threads is a count")?;
            }
            "--forgotten" => arguments.forgotten = true,
            "--visibility" => {
                arguments.committed = match value()?.as_str() {
                    "committed" => true,
                    "durable" => false,
                    other => {
                        return Err(format!("--visibility is durable or committed, not {other:?}"));
                    }
                };
            }
            "--runs" => arguments.runs = value()?.parse().map_err(|_| "--runs is a count")?,
            "--help" | "-h" => return Err(USAGE.to_string()),
            other => return Err(format!("{other:?} is not an option\n\n{USAGE}")),
        }
    }
    if arguments.backend.is_empty() {
        return Err(USAGE.to_string());
    }
    if arguments.plan.clients == 0 || arguments.plan.records == 0 || arguments.warehouses == 0 {
        return Err("--clients, --records and --warehouses have to be at least one".to_string());
    }
    if arguments.runs == 0 || arguments.analytic_threads == 0 {
        return Err("--runs and --analytic-threads have to be at least one".to_string());
    }
    if open && arguments.mode != Mode::Ycsb {
        return Err("the tpcc and ch modes run a closed loop only for now".to_string());
    }
    if arguments.analytic > 0 && arguments.mode != Mode::Tpcc {
        return Err("--analytic runs streams beside the tpcc mode's terminals".to_string());
    }
    if arguments.forgotten && arguments.mode != Mode::Tpcc {
        return Err(
            "--forgotten holds a transaction open beside the tpcc mode's terminals".to_string()
        );
    }
    if open {
        let rate = rate.filter(|rate| *rate > 0.0).ok_or("an open loop wants --rate")?;
        arguments.plan.mode = Loop::Open { rate, poisson };
    }
    Ok(arguments)
}

/// Bytes this process has had written to the block device, where Linux says.
fn written() -> Option<u64> {
    let io = std::fs::read_to_string("/proc/self/io").ok()?;
    io.lines().find_map(|line| line.strip_prefix("write_bytes: ")?.trim().parse().ok())
}

/// Opens the backend, loads if asked, runs, and prints.
fn drive(arguments: &Arguments) -> Result<(), String> {
    let mut pinned = true;
    let backend: Box<dyn Backend> = match arguments.backend.as_str() {
        "null" if arguments.shim => {
            Box::new(Shim::new(NullBackend::new(arguments.service, arguments.stall)))
        }
        #[cfg(feature = "rudb")]
        "rudb" if arguments.shim => Box::new(Shim::new(backend::rudb::RudbBackend::open(
            &arguments.target,
            arguments.level,
        )?)),
        _ if arguments.shim => {
            return Err(
                "--shim unix is for the rudb and null backends, which run in process".to_string()
            );
        }
        "null" => Box::new(NullBackend::new(arguments.service, arguments.stall)),
        "sqlite" => {
            let backend = SqliteBackend::open(&arguments.target, arguments.level)?;
            eprintln!("sqlite library {}", backend.library());
            Box::new(backend)
        }
        "duckdb" => {
            let (backend, is_pin) =
                DuckdbBackend::open(&arguments.target, arguments.level, arguments.unpinned)?;
            pinned = is_pin;
            Box::new(backend)
        }
        "postgres" => Box::new(PostgresBackend::open(&arguments.target, arguments.level)?),
        #[cfg(feature = "rudb")]
        "rudb" => Box::new(backend::rudb::RudbBackend::open(&arguments.target, arguments.level)?),
        #[cfg(not(feature = "rudb"))]
        "rudb" => return Err("this driver was built without --features rudb".to_string()),
        other => return Err(format!("there is no backend {other:?}")),
    };
    backend.level()?;
    backend.visibility(arguments.committed)?;
    let plan = &arguments.plan;
    if arguments.mode != Mode::Ycsb {
        if arguments.backend == "duckdb" && arguments.mode == Mode::Tpcc {
            return Err(
                "the duckdb backend reads VARCHAR results only, so tpcc does not run on it \
                        yet"
                .to_string(),
            );
        }
        let settings = tpcc::Settings {
            warehouses: arguments.warehouses,
            terminals: plan.clients,
            warmup: plan.warmup,
            window: plan.window,
            seed: plan.seed,
            load: arguments.load,
            level: arguments.level,
            pinned,
            card: arguments.card.clone(),
            serializable: arguments.serializable,
            analytic: arguments.analytic,
            analytic_threads: arguments.analytic_threads,
            forgotten: arguments.forgotten,
            committed: arguments.committed,
        };
        if arguments.mode == Mode::Ch {
            return tpcc::reference(&*backend, &settings, arguments.runs);
        }
        return tpcc::drive(&*backend, &settings);
    }
    let mode = match plan.mode {
        Loop::Closed => "mode=closed".to_string(),
        Loop::Open { rate, poisson } => {
            format!("mode=open rate={rate} arrivals={}", if poisson { "poisson" } else { "fixed" })
        }
    };
    println!(
        "kv 1 backend={} version={} workload={} distribution={} records={} clients={} {mode} \
         seed={:#x} level={} warmup_s={} window_s={} pinned={} boundary={}",
        backend.name(),
        backend.version().replace(' ', "_"),
        plan.mix.text(),
        plan.distribution.name(),
        plan.records,
        plan.clients,
        plan.seed,
        arguments.level.name(),
        plan.warmup.as_secs_f64(),
        plan.window.as_secs_f64(),
        if pinned { "yes" } else { "no" },
        backend.boundary(),
    );
    if let Some(note) = backend.level_note() {
        println!("note {note}");
    }
    match &arguments.card {
        Some(card) => println!("card {}", card.strip_prefix("card ").unwrap_or(card)),
        None => println!("card none"),
    }
    if arguments.load {
        let took = run::load(&*backend, plan.records, plan.clients)?;
        println!("load rows={} took_s={:.3}", plan.records, took.as_secs_f64());
    }
    let written_before = written();
    let outcome = run::run(&*backend, plan)?;
    print!("{}", run::report(&outcome, plan));
    let (start, end) = outcome.window_usage;
    let written = written_before.zip(written()).map(|(before, after)| after.saturating_sub(before));
    println!(
        "cost cpu_user_s={:.3} cpu_sys_s={:.3} peak_rss={} written={}",
        (end.user.saturating_sub(start.user)).as_secs_f64(),
        (end.system.saturating_sub(start.system)).as_secs_f64(),
        end.peak,
        written.map_or_else(|| "unknown".to_string(), |bytes| bytes.to_string()),
    );
    let bad: u64 = outcome.clients.iter().map(|client| client.bad).sum();
    let reads: u64 = outcome.clients.iter().map(|client| client.reads).sum();
    println!(
        "verify values={} bad={bad} reads={reads} stale=unchecked tracked=0",
        if bad == 0 { "ok" } else { "bad" }
    );
    if let Some(error) = outcome.clients.iter().find_map(|client| client.error.as_ref()) {
        eprintln!("an operation failed: {error}");
    }
    println!("end");
    Ok(())
}

fn main() -> ExitCode {
    let arguments = match parse(std::env::args().skip(1)) {
        Ok(arguments) => arguments,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    match drive(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(text: &str) -> impl Iterator<Item = String> + '_ {
        text.split_whitespace().map(str::to_string)
    }

    #[test]
    fn durations_take_the_units_a_person_writes() {
        assert_eq!(duration("100us").unwrap(), Duration::from_micros(100));
        assert_eq!(duration("5ms").unwrap(), Duration::from_millis(5));
        assert_eq!(duration("10").unwrap(), Duration::from_secs(10));
        assert!(duration("10m").is_err());
    }

    #[test]
    fn an_open_loop_needs_a_rate() {
        assert!(parse(words("--backend null --loop open")).is_err());
        let arguments =
            parse(words("--backend null --loop open --rate 5000 --stall 100ms/10s")).unwrap();
        assert_eq!(arguments.plan.mode, Loop::Open { rate: 5000.0, poisson: false });
        assert_eq!(arguments.stall, Some((Duration::from_millis(100), Duration::from_secs(10))));
        assert!(parse(words("--backend null --stall 10s/1s")).is_err());
    }

    #[test]
    fn the_tpcc_mode_takes_warehouses_and_terminals() {
        let arguments =
            parse(words("--mode tpcc --backend null --warehouses 4 --terminals 8")).unwrap();
        assert_eq!(arguments.mode, Mode::Tpcc);
        assert_eq!((arguments.warehouses, arguments.plan.clients), (4, 8));
        assert!(parse(words("--mode tpcc --backend null --warehouses 0")).is_err());
        assert!(parse(words("--mode tpcc --backend null --loop open --rate 10")).is_err());
        assert!(parse(words("--mode tpch --backend null")).is_err());
        assert!(parse(words("--backend null --shim unix")).unwrap().shim);
        assert!(parse(words("--backend null --shim tcp")).is_err());
        assert!(
            parse(words("--mode tpcc --backend null --isolation serializable"))
                .unwrap()
                .serializable
        );
        assert!(!parse(words("--mode tpcc --backend null")).unwrap().serializable);
        assert!(parse(words("--backend null --isolation snapshot")).is_err());
        assert!(
            parse(words("--mode tpcc --backend null --visibility committed")).unwrap().committed
        );
        assert!(
            !parse(words("--mode tpcc --backend null --visibility durable")).unwrap().committed
        );
        assert!(parse(words("--backend null --visibility eventual")).is_err());
    }

    #[test]
    fn the_analytic_streams_run_beside_the_terminals_and_the_reference_alone() {
        let arguments =
            parse(words("--mode tpcc --backend null --analytic 2 --analytic-threads 4")).unwrap();
        assert_eq!((arguments.analytic, arguments.analytic_threads), (2, 4));
        let arguments = parse(words("--mode ch --backend null --runs 3")).unwrap();
        assert_eq!((arguments.mode, arguments.runs, arguments.analytic_threads), (Mode::Ch, 3, 2));
        assert!(parse(words("--mode ch --backend null --runs 0")).is_err());
        assert!(parse(words("--mode tpcc --backend null --analytic-threads 0")).is_err());
        assert!(parse(words("--mode ch --backend null --analytic 1")).is_err());
        assert!(parse(words("--backend null --analytic 1")).is_err());
        assert!(parse(words("--mode tpcc --backend null --forgotten")).unwrap().forgotten);
        assert!(parse(words("--mode ch --backend null --forgotten")).is_err());
    }
}
