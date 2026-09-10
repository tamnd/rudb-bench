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
//!
//! `run` also carries the regression gate, in [`rudb_bench::regress`]. `--check` is what CI runs
//! and `--record` is what puts a run into the committed file. They are flags on `run` rather than
//! subcommands of their own because the gate compares a run that just happened, and a `check`
//! subcommand that secretly ran the suite first would be a command whose name hid the expensive
//! half of what it does.

#![forbid(unsafe_code)]

use std::process::ExitCode;

use rudb_bench::engine::{
    BenchError, ClickhouseLocal, ClickhouseServer, Datafusion, Duckdb, Engine, Polars, Rudb,
};
use rudb_bench::ledger;
use rudb_bench::machine;
use rudb_bench::regress::{self, FACTOR, Watch};
use rudb_bench::report::{Abstention, comparison, table};
use rudb_bench::suite::{SUITES, Suite, queries};
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
        Some("ledger") => ledger(),
        Some("run") => match plan(&args[1..]) {
            Ok(plan) => run(&plan),
            Err(e) => {
                eprintln!("rudb-bench: {e}");
                ExitCode::FAILURE
            }
        },
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

/// What `run` does with the committed records afterwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Gate {
    /// Print the table and stop, which is what a person asking for numbers wants.
    Nothing,
    /// Compare against the records and fail on a regression.
    Check(Watch),
    /// Replace the records for the engines that ran here.
    Record,
}

impl Gate {
    /// How many hot runs this job wants.
    ///
    /// Five for a person asking for a table, which is the floor rule two sets and the number that
    /// gets an answer back in a minute. Fifteen for anything that writes or reads a committed
    /// record, because the interquartile range of five samples is the gap between the second and
    /// the fourth of them, and that is a crude estimate of a spread rather than a measurement of
    /// one. On `smoke`, where a query is under a second, five samples put every engine over the ten
    /// percent line on an idle machine, which would make a gate built on the spread useless before
    /// it ever caught anything. More samples cost seconds and buy the difference between a gate and
    /// a decoration.
    const fn runs(self) -> usize {
        match self {
            Self::Nothing => 5,
            Self::Check(_) | Self::Record => 15,
        }
    }
}

/// What a `run` invocation asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Plan {
    /// The suite.
    suite: String,
    /// What to do with the committed records afterwards.
    gate: Gate,
    /// How many hot runs per query.
    runs: usize,
    /// The layer to store this run under in the attribution ledger, when it closes one.
    ///
    /// Separate from [`Gate`] because it is a separate question. `--record` moves the bar the
    /// regression gate compares against, and `--store` adds a row to the history of what each layer
    /// bought. A run that closes a layer usually wants both and neither implies the other.
    store: Option<String>,
}

/// Read the arguments after `run`.
///
/// Written out rather than reached for a parser, because the whole argument surface of this program
/// is one positional and four flags, and an unknown flag has to be an error rather than a suite name
/// with two dashes in front of it. The three job flags are spelled differently instead of being one
/// flag with a value: `--check` answers whether something got twice as slow and is safe anywhere,
/// `--check-drift` answers by how much and is only meaningful on the machine the record came from,
/// and somebody who types the wrong one gets the wrong question answered rather than a warning.
fn plan(args: &[String]) -> Result<Plan, String> {
    let mut suite = None;
    let mut gate = Gate::Nothing;
    let mut runs = None;
    let mut store = None;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        let wanted = match arg.as_str() {
            "--check" => Gate::Check(Watch::Ci),
            "--check-drift" => Gate::Check(Watch::Scheduled),
            "--record" => Gate::Record,
            "--runs" => {
                let given = rest.next().ok_or("--runs wants a number after it")?;
                let n: usize =
                    given.parse().map_err(|e| format!("--runs {given} is not a number, {e}"))?;
                if n < 5 {
                    return Err(format!(
                        "--runs {n} is under the five that reporting rule two requires"
                    ));
                }
                runs = Some(n);
                continue;
            }
            "--store" => {
                let given = rest.next().ok_or(
                    "--store wants the layer this run closes after it, such as `--store 2a`",
                )?;
                if given.starts_with("--") {
                    return Err(format!(
                        "--store wants a layer name after it and got {given}, which is a flag"
                    ));
                }
                store = Some(given.clone());
                continue;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown option {other}, try `rudb-bench --help`"));
            }
            other => {
                suite = Some(other.to_owned());
                continue;
            }
        };
        if gate != Gate::Nothing && gate != wanted {
            return Err("--check, --check-drift and --record are three different jobs, so run \
                        one of them at a time"
                .to_owned());
        }
        gate = wanted;
    }
    Ok(Plan {
        suite: suite.unwrap_or_else(|| "smoke".to_owned()),
        gate,
        runs: runs.unwrap_or_else(|| gate.runs()),
        store,
    })
}

/// Run a suite against every engine that can run it, and print the table.
fn run(plan: &Plan) -> ExitCode {
    let name = plan.suite.as_str();
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
    let (mut engines, missing) = discover(&scratch, suite);

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
        rudb_bench::report::compare(&mut engines, suite, queries, &dataset.tables, plan.runs);
    compared.skipped.extend(missing);

    for result in &compared.results {
        print!("{}", table(result));
        println!();
    }
    print!("{}", comparison(&compared));

    let _ = std::fs::remove_dir_all(&scratch);
    if compared.results.is_empty() {
        return ExitCode::FAILURE;
    }
    // Before the gate, so that a run which closes a layer and fails its own regression check still
    // leaves the row behind. The ledger is a history and the afternoon something got slower is
    // exactly the afternoon a history is worth having.
    if let Some(layer) = plan.store.as_deref() {
        if store(&compared, layer) == ExitCode::FAILURE {
            return ExitCode::FAILURE;
        }
    }
    match plan.gate {
        Gate::Nothing => ExitCode::SUCCESS,
        Gate::Check(watch) => check(&compared, watch),
        Gate::Record => record(&compared),
    }
}

/// Compare what just ran against the committed records.
fn check(compared: &rudb_bench::report::Comparison, watch: Watch) -> ExitCode {
    let at = regress::path(compared.suite.name);
    let records = match regress::read(&at) {
        Ok(records) => records,
        Err(e) => {
            eprintln!("rudb-bench: {e}");
            return ExitCode::FAILURE;
        }
    };
    let here = machine::name_here();
    let verdicts = regress::check_all(&records, compared, &here, watch);
    println!("{}", at.display());
    print!("{}", regress::report(&verdicts, watch, records.len()));
    // A machine with no record passes. The alternative is a gate that cannot be introduced without
    // being introduced on every machine at once, and the sentence above says how to make it a gate
    // here, which is the part that stops it being a hole nobody notices.
    if verdicts.iter().any(regress::Verdict::failed) {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Write what just ran into the committed records.
fn record(compared: &rudb_bench::report::Comparison) -> ExitCode {
    let at = regress::path(compared.suite.name);
    let here = machine::name_here();
    let today = regress::today();
    let taken: Vec<regress::Record> =
        compared.results.iter().map(|r| regress::Record::of(r, &here, &today)).collect();
    if let Err(e) = regress::write(&at, &taken) {
        eprintln!("rudb-bench: {e}");
        return ExitCode::FAILURE;
    }
    println!("{}", at.display());
    for record in &taken {
        println!("  {}", regress::describe_record(record));
    }
    println!();
    println!("Read the diff before committing it. A record taken on a machine somebody else was");
    println!("using is a record that raises the bar the gate has to clear, quietly and forever.");
    ExitCode::SUCCESS
}

/// Add what just ran to the committed runs, as the row that closes a layer.
fn store(compared: &rudb_bench::report::Comparison, layer: &str) -> ExitCode {
    let at = ledger::path(compared.suite.name);
    let run = ledger::Stored::of(
        compared,
        &machine::name_here(),
        layer,
        &ledger::commit_here(),
        &regress::today(),
    );
    if let Err(e) = ledger::write(&at, &run) {
        eprintln!("rudb-bench: {e}");
        return ExitCode::FAILURE;
    }
    println!("{}", at.display());
    println!("  stored as the run closing {layer} on {}", run.machine);
    println!();
    ExitCode::SUCCESS
}

/// Print the attribution ledger over every suite that has stored runs.
fn ledger() -> ExitCode {
    let mut stored = Vec::new();
    for suite in SUITES {
        match ledger::read(&ledger::path(suite.name)) {
            Ok(runs) => stored.extend(runs),
            Err(e) => {
                eprintln!("rudb-bench: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
    print!("{}", ledger::report(&ledger::rows(&stored)));
    ExitCode::SUCCESS
}

/// Every engine on this machine, and a sentence for every one that is not.
///
/// DuckDB first, because it is the reference column of the comparison and the ratio row is stated
/// against it. The order after that is the order section 15.3 names them in, and it is fixed rather
/// than discovery order so that two runs a week apart produce the same table.
///
/// The one exception to that order is the tuned ClickHouse server, which goes last among the
/// engines that measure anything. It is the only engine here that leaves a process running after
/// its own load, and half a gigabyte of resident ClickHouse next to a DuckDB being timed is
/// somebody else's memory and somebody else's page cache in a column that claims to be DuckDB's.
/// Since the comparison runs one engine to completion before starting the next, putting it last
/// means it comes up after everything it could disturb has already been measured.
fn discover(
    scratch: &std::path::Path,
    suite: &'static Suite,
) -> (Vec<Box<dyn Engine>>, Vec<Abstention>) {
    let mut engines: Vec<Box<dyn Engine>> = Vec::new();
    let mut missing = Vec::new();

    match Duckdb::discover(scratch) {
        Ok(engine) => engines.push(Box::new(engine)),
        Err(e) => missing.push(gap("duckdb", &e)),
    }
    match ClickhouseLocal::discover(scratch, suite) {
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
    match ClickhouseServer::discover(scratch, suite) {
        Ok(engine) => engines.push(Box::new(engine)),
        Err(e) => missing.push(gap("clickhouse-server", &e)),
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
    println!(
        "    --check         fail when a query got {FACTOR:.0}x slower and left its recorded range"
    );
    println!(
        "    --check-drift   the same at {:.0}%, and only on the machine the record came from",
        (regress::DRIFT - 1.0) * 100.0
    );
    println!("    --record        replace the committed records for the engines that ran here");
    println!("    --store <layer> add this run to the ledger as the row that closes a layer");
    println!("    --runs n        hot runs per query, five at least, default five and fifteen");
    println!("                    for anything that reads or writes a record");
    println!("  ledger        print what each layer bought, from the committed runs");
    println!("  load          load a suite's data into each engine and time it");
    println!("  report        write the published status page from the last run");
    println!("  -V, --version print the version and exit");
    println!();
    println!("  RUDB_BENCH_DUCKDB       the DuckDB binary, which is the reference column");
    println!("  RUDB_BENCH_CLICKHOUSE   the ClickHouse binary, driven both as `local` and as a");
    println!("                          real server with a sorting key, which are two rows");
    println!("  RUDB_BENCH_DATAFUSION   the datafusion-cli binary");
    println!("  RUDB_BENCH_PYTHON       a python3 with polars installed");
    println!("  RUDB_BENCH_RUDB         the rudb binary, when there is one worth running");
    println!("  RUDB_BENCH_DATA         where the corpora live, default ~/rudb-data");
    println!("  RUDB_BENCH_SCRATCH      where a run puts its data");
    println!("  RUDB_BENCH_MACHINE      what to call this machine in a committed record");
    println!("  RUDB_BENCH_BASELINE     the records file, default baselines/<suite>.txt");
    println!();
    println!("`run` works on the smoke suite, which generates its own data and measures nothing");
    println!("anybody should quote. The suites that matter need a download or a generator and");
    println!("`suites` says which. The design is spec/15-rudb-bench.md in");
    println!("https://github.com/tamnd/rudb.");
}

#[cfg(test)]
mod tests {
    use super::{Gate, plan};
    use rudb_bench::regress::Watch;

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_owned).collect()
    }

    #[test]
    fn no_arguments_at_all_is_the_smoke_suite_and_no_gate() {
        let got = plan(&[]).expect("nothing is a valid thing to ask for");
        assert_eq!(got.suite, "smoke");
        assert_eq!(got.gate, Gate::Nothing);
        assert_eq!(got.runs, 5);
    }

    #[test]
    fn a_gate_takes_more_samples_than_a_person_looking_at_a_table_does() {
        // The reason this is not one number: five samples put every engine on this suite over the
        // ten percent line on an idle server, so a gate built on the spread at five samples would
        // report noise on every run and catch nothing.
        assert_eq!(plan(&args("smoke --check")).unwrap().runs, 15);
        assert_eq!(plan(&args("smoke --record")).unwrap().runs, 15);
        assert_eq!(plan(&args("smoke")).unwrap().runs, 5);
    }

    #[test]
    fn the_two_checks_are_two_different_questions_and_are_spelled_differently() {
        assert_eq!(plan(&args("smoke --check")).unwrap().gate, Gate::Check(Watch::Ci));
        assert_eq!(plan(&args("smoke --check-drift")).unwrap().gate, Gate::Check(Watch::Scheduled));
    }

    #[test]
    fn the_number_after_runs_is_not_mistaken_for_the_suite() {
        // The bug this is written against: a positional suite name and a flag that takes a value,
        // where the naive read of "the first argument without dashes" makes 15 the suite.
        let got = plan(&args("--runs 15 tpch")).expect("a flag with a value and a positional");
        assert_eq!(got.suite, "tpch");
        assert_eq!(got.runs, 15);
    }

    #[test]
    fn fewer_than_five_runs_is_refused_by_name() {
        let e = plan(&args("smoke --runs 3")).unwrap_err();
        assert!(e.contains("rule two"), "{e}");
        assert!(plan(&args("smoke --runs")).is_err());
        assert!(plan(&args("smoke --runs many")).is_err());
    }

    #[test]
    fn two_jobs_at_once_is_refused_rather_than_silently_being_the_last_one() {
        let e = plan(&args("smoke --record --check")).unwrap_err();
        assert!(e.contains("one of them at a time"), "{e}");
        // The same one twice is fine, because it is not ambiguous.
        assert!(plan(&args("smoke --check --check")).is_ok());
    }

    #[test]
    fn an_unknown_flag_is_an_error_and_not_a_suite_with_dashes_on_it() {
        let e = plan(&args("smoke --fast")).unwrap_err();
        assert!(e.contains("--fast"), "{e}");
    }
}
