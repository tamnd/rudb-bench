//! The harness command line.
//!
//! `run` works against every engine on the machine on the smoke suite, which is the suite that
//! exists so the measurement path is exercised on every commit rather than on the day somebody
//! needs a number. The suites that matter need data that has to be downloaded or generated first,
//! and they say so by name rather than failing with an error about a missing file.
//!
//! An engine that is not installed is a line saying so rather than an error, because the machine
//! that has all five is the exception. What is not allowed is a table that quietly has one fewer
//! column than the one before it.
//!
//! The subcommand names come from `spec/15-rudb-bench.md` and the nightly job in the rudb
//! repository calls them by name, so they are a decision rather than an afterthought.

#![forbid(unsafe_code)]

use std::process::ExitCode;

use rudb_bench::engine::{BenchError, ClickhouseLocal, Datafusion, Duckdb, Engine, Polars, Rudb};
use rudb_bench::machine;
use rudb_bench::report::{Abstention, comparison, table};
use rudb_bench::suite::{SUITES, queries};
use rudb_bench::{CLICKBENCH_C6A_4XLARGE, FLEET, REPORTING_MACHINE, Role, target_seconds};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--version" | "-V") => {
            println!("rudb-bench {VERSION}");
            ExitCode::SUCCESS
        }
        Some("baselines") => {
            baselines();
            ExitCode::SUCCESS
        }
        Some("fleet") => {
            fleet();
            ExitCode::SUCCESS
        }
        Some("suites") => {
            suites();
            ExitCode::SUCCESS
        }
        Some("machine") => {
            machine_record();
            ExitCode::SUCCESS
        }
        Some("run") => run(args.get(1).map_or("smoke", String::as_str)),
        Some("load" | "report") => {
            eprintln!("rudb-bench: not built yet, see spec/15-rudb-bench.md in tamnd/rudb");
            ExitCode::FAILURE
        }
        Some("--help" | "-h") | None => {
            help();
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("rudb-bench: unknown argument {other}");
            eprintln!("rudb-bench: try `rudb-bench --help`");
            ExitCode::FAILURE
        }
    }
}

/// The board as it stood when the specification was written, and the number the project has to
/// reach. Printing the target next to the leader is deliberate: it is 3.1x past the fastest thing
/// anybody has published on this machine, and that fact should be hard to avoid.
fn baselines() {
    println!("ClickBench, c6a.4xlarge, sum of the best of three per query, 43 queries");
    println!("Recomputed from the official result files on 10 September 2026.");
    println!();
    println!("system        hot total    hits on disk");
    for base in CLICKBENCH_C6A_4XLARGE {
        let disk = match base.disk_gb {
            Some(gb) => format!("{gb:.2} GB"),
            None => "not reported".to_string(),
        };
        println!("{:<12}  {:>7.2}s    {}", base.system, base.hot_total_seconds, disk);
    }

    let duckdb = CLICKBENCH_C6A_4XLARGE.iter().find(|b| b.system == "DuckDB");
    if let Some(duckdb) = duckdb {
        // Rounded away from zero rather than through the formatter, which rounds a half to even
        // and would print 2.62 for a number the specification states as 2.63. A benchmark harness
        // whose own headline number disagrees with the document by a hundredth is a harness people
        // stop trusting for good reasons.
        let target = (target_seconds(duckdb.hot_total_seconds, 10.0) * 100.0).round() / 100.0;
        println!();
        println!("Ten times DuckDB is {target:.2}s, which is past every system in that table.");
        println!("spec/02-the-goal.md is where that either closes or does not.");
    }
}

/// The machines the project actually has, and what a number from each is worth.
///
/// Printed with the reporting machine at the top rather than at the bottom, because the useful
/// fact here is the one about what these machines are not.
fn fleet() {
    println!("Published numbers come from {REPORTING_MACHINE}.");
    println!("Nothing below is that machine. Reporting rule seven: never compare across machines.");
    println!();
    println!("machine     role         cores/threads   memory   free disk");
    for machine in FLEET {
        let role = match machine.role {
            Role::Reporting => "reporting",
            Role::Regression => "regression",
            Role::Correctness => "correctness",
        };
        println!(
            "{:<10}  {:<11}  {:>6}/{:<7}  {:>4} GiB  {:>5} GiB",
            machine.name,
            role,
            machine.cores,
            machine.threads,
            machine.memory_gib,
            machine.free_disk_gib
        );
    }
    println!();
    for machine in FLEET {
        println!("{}: {}", machine.name, machine.os);
        println!("  {}", machine.cpu);
        println!("  {}", machine.note);
        println!();
    }
    println!("These track the engine against itself over time, which is a job that wants the same");
    println!("machine far more than it wants the right machine. They do not produce a number that");
    println!("goes in a README, a release note or a talk. Probed 10 September 2026.");
}

/// The suites and what each of them needs before it can run.
fn suites() {
    println!("suite       queries  comparable  needs");
    for suite in SUITES {
        let comparable = if suite.comparable { "yes" } else { "no" };
        println!("{:<10}  {:>7}  {:<10}  {}", suite.name, suite.queries, comparable, suite.needs);
    }
    println!();
    for suite in SUITES {
        println!("{}: {}", suite.name, suite.note);
        println!();
    }
    println!(
        "Only `smoke` runs today, because it is the only one whose data this harness can make"
    );
    println!("for itself. The rest need a download or a generator, and they say which.");
}

/// Everything about this machine that has to be read next to a number from it.
fn machine_record() {
    let scratch = scratch();
    println!("Probed now, for the directory the data would land in.");
    println!("{}", scratch.display());
    println!();
    for fact in machine::probe(&scratch) {
        let mark = if fact.known { " " } else { "?" };
        println!("{mark} {:<12}  {}", fact.name, fact.value);
    }
    println!();
    println!(
        "A line marked ? was not readable here, which is a fact about the machine rather than"
    );
    println!("a gap in the record. Section 15.4 wants all of this printed next to the number and");
    println!("not assumed, because the assumption that fails silently is the frequency policy.");
}

/// Run a suite against every engine that can run it, and print the table.
fn run(name: &str) -> ExitCode {
    let Some(suite) = rudb_bench::suite::find(name) else {
        eprintln!("rudb-bench: no suite called {name}");
        eprintln!("rudb-bench: try `rudb-bench suites`");
        return ExitCode::FAILURE;
    };
    let Some(queries) = queries(name) else {
        eprintln!("rudb-bench: the {name} suite needs {}", suite.needs);
        eprintln!("rudb-bench: only `smoke` runs today, see `rudb-bench suites`");
        return ExitCode::FAILURE;
    };

    let scratch = scratch();
    let (mut engines, missing) = discover(&scratch);

    // The data before the engines, because every engine gets the same files and the first thing a
    // reader of a result asks is which files those were.
    let dataset = match rudb_bench::data::prepare(suite, &scratch) {
        Ok(dataset) => dataset,
        Err(e) => {
            eprintln!("rudb-bench: {e}");
            let _ = std::fs::remove_dir_all(&scratch);
            return ExitCode::FAILURE;
        }
    };
    for file in &dataset.tables {
        println!("{:<10}  {}", file.name, file.path.display());
    }
    println!();

    let mut compared =
        rudb_bench::report::compare(&mut engines, suite, queries, &dataset.tables, 5);
    compared.skipped.extend(missing);

    for result in &compared.results {
        print!("{}", table(result));
        println!();
    }
    print!("{}", comparison(&compared));

    let _ = std::fs::remove_dir_all(&scratch);
    if compared.results.is_empty() { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

/// Every engine on this machine, and a sentence for every one that is not.
///
/// DuckDB first, because it is the reference column of the comparison and the ratio row is stated
/// against it. The order after that is the order section 15.3 names them in, and it is fixed rather
/// than discovery order so that two runs a week apart produce the same table.
fn discover(scratch: &std::path::Path) -> (Vec<Box<dyn Engine>>, Vec<Abstention>) {
    let mut engines: Vec<Box<dyn Engine>> = Vec::new();
    let mut missing = Vec::new();

    match Duckdb::discover(scratch) {
        Ok(engine) => engines.push(Box::new(engine)),
        Err(e) => missing.push(gap("duckdb", &e)),
    }
    match ClickhouseLocal::discover(scratch) {
        Ok(engine) => engines.push(Box::new(engine)),
        Err(e) => missing.push(gap("clickhouse-local", &e)),
    }
    match Datafusion::discover(scratch) {
        Ok(engine) => engines.push(Box::new(engine)),
        Err(e) => missing.push(gap("datafusion", &e)),
    }
    match Polars::discover(scratch) {
        Ok(engine) => engines.push(Box::new(engine)),
        Err(e) => missing.push(gap("polars", &e)),
    }

    // rudb is always in the list, because an engine that is missing from a comparison because it
    // could not have been built is a different thing from one that abstains, and only one of those
    // is where this project actually is.
    engines.push(Box::new(Rudb::discover(scratch)));
    (engines, missing)
}

/// An engine that is not on this machine, as a line of the report.
fn gap(what: &str, why: &BenchError) -> Abstention {
    Abstention { engine: what.to_owned(), version: "not found".to_owned(), why: why.to_string() }
}

/// Where a run puts its data.
///
/// Under the temporary directory by default and wherever `RUDB_BENCH_SCRATCH` says otherwise,
/// because on three of the four machines in the fleet the temporary directory is on the small
/// device and the data has to go somewhere else.
fn scratch() -> std::path::PathBuf {
    std::env::var_os("RUDB_BENCH_SCRATCH").map_or_else(
        || std::env::temp_dir().join(format!("rudb-bench-{}", std::process::id())),
        std::path::PathBuf::from,
    )
}

fn help() {
    println!("rudb-bench {VERSION}");
    println!("The benchmark harness for rudb.");
    println!();
    println!("Usage: rudb-bench <command> [options]");
    println!();
    println!("  baselines     print the recorded board and the number we have to reach");
    println!("  fleet         print the development machines and what a number from each is worth");
    println!("  suites        print the suites and what each of them needs before it can run");
    println!("  machine       record the machine, the frequency policy and the mount options");
    println!("  run [suite]   run a suite on every engine here and compare, defaults to smoke");
    println!("  load          load a suite's data into each engine and time it");
    println!("  report        write the published status page from the last run");
    println!("  -V, --version print the version and exit");
    println!();
    println!("  RUDB_BENCH_DUCKDB       the DuckDB binary, which is the reference column");
    println!("  RUDB_BENCH_CLICKHOUSE   the ClickHouse binary, driven as `clickhouse local`");
    println!("  RUDB_BENCH_DATAFUSION   the datafusion-cli binary");
    println!("  RUDB_BENCH_PYTHON       a python3 with polars installed");
    println!("  RUDB_BENCH_RUDB         the rudb binary, when there is one worth running");
    println!("  RUDB_BENCH_DATA         where the corpora live, default ~/rudb-data");
    println!("  RUDB_BENCH_SCRATCH      where a run puts its data");
    println!();
    println!("`run` works on the smoke suite, which generates its own data and measures nothing");
    println!("anybody should quote. The suites that matter need a download or a generator and");
    println!("`suites` says which. The design is spec/15-rudb-bench.md in");
    println!("https://github.com/tamnd/rudb.");
}
