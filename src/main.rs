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
use std::time::Duration;

use rudb_bench::attribute::Ablation;
use rudb_bench::data::Rows;
use rudb_bench::engine::{
    Ability, BenchError, ClickhouseLocal, ClickhouseServer, Datafusion, Duckdb, Engine, Polars,
    Rudb,
};
use rudb_bench::kernels;
use rudb_bench::ledger;
use rudb_bench::machine;
use rudb_bench::planning;
use rudb_bench::regress::{self, FACTOR, Watch};
use rudb_bench::report::{Abstention, comparison, table};
use rudb_bench::suite::{SUITES, Scale, Suite, queries};
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
        Some("kernels") => kernels(&args[1..]),
        Some("sweep") => sweep(&args[1..]),
        Some("attribute") => attribute(&args[1..]),
        Some("plans") => plans(&args[1..]),
        Some("seams") => seams(),
        Some("run") => match plan(&args[1..]) {
            Ok(plan) => run(&plan),
            Err(e) => {
                eprintln!("rudb-bench: {e}");
                ExitCode::FAILURE
            }
        },
        Some("report") => report_saved(args.get(1).map_or("clickbench", String::as_str)),
        Some("load") => {
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
    println!("machine       role         cores/threads   memory   free disk   clickbench");
    for machine in FLEET {
        let role = match machine.role {
            Role::Reporting => "reporting",
            Role::Regression => "regression",
            Role::Correctness => "correctness",
        };
        println!(
            "{:<12}  {:<11}  {:>6}/{:<7}  {:>4} GiB  {:>5} GiB   {}",
            machine.name,
            role,
            machine.cores,
            machine.threads,
            machine.memory_gib,
            machine.free_disk_gib,
            if machine.clickbench { "yes" } else { "no" }
        );
    }
    println!();
    for machine in FLEET {
        println!("{}: {}", machine.name, machine.os);
        println!("  answers to hostname {}", machine.hostname);
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
        if !suite.scales().is_empty() {
            let default = suite.default_scale().map_or("none", |scale| scale.label);
            println!(
                "{:<10}  scale factors {}, default {default}, under {}",
                suite.name,
                suite.scale_labels(),
                suite.directory(None)
            );
        }
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
    let scratch = match scratch() {
        Ok(at) => at,
        Err(e) => {
            eprintln!("rudb-bench: cannot make the scratch directory: {e}");
            return;
        }
    };
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
    let _ = std::fs::remove_dir_all(&scratch);
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
    /// The engines this run asked for, or every engine on the machine when it did not ask.
    ///
    /// A filter rather than a list to add to, because the reason to reach for this is always that
    /// one engine is in the way of the other four. A run of TPC-H at scale factor 100 on `gamingpc`
    /// took the machine down inside Polars, and without this flag the only way to get the other
    /// four numbers back was to uninstall Polars. An engine left out this way is an abstention with
    /// a sentence saying it was left out, not a missing row, because a table that silently has one
    /// fewer column than the one before it is the thing this harness exists to not produce.
    engines: Option<Vec<String>>,
    /// How many rows of the suite's data to run over, when that is not all of them.
    ///
    /// The development loop. A full ClickBench is hours and most changes are asking whether they did
    /// anything at all, which a million rows answers in minutes. Nothing measured this way is
    /// comparable to anything and every table printed from it says so.
    rows: Option<Rows>,
    /// Which scale factor of a generated suite to run, when the default is not the one wanted.
    ///
    /// A different question from [`Plan::rows`] and not a smaller version of it. A scale factor is a
    /// corpus the generator built to a specification, with the joins between its tables intact, and
    /// every number out of one is comparable to every other number at the same scale. `--rows` is a
    /// stride over a corpus that already exists and is a development loop.
    scale: Option<&'static Scale>,
    /// Whether to write the run out as markdown as well as printing it.
    ///
    /// The terminal table is for the person watching the run and it drops most of what was
    /// measured, because five engines and forty three queries do not fit in eighty columns any
    /// other way. The file keeps everything, and it keeps it somewhere that can be committed and
    /// read next week, which is the difference between a measurement and a number somebody
    /// remembers.
    report: bool,
    /// Whether to add what this run measured to the saved file for this machine.
    ///
    /// The flag that makes one engine at a time a workable way to benchmark. A full ClickBench is
    /// hours per engine and six engines in one command is a day that fails on the fifth one and
    /// leaves nothing behind. With this, each engine is measured on its own and written down, and
    /// `rudb-bench report <suite>` builds the cross engine table afterwards out of everything that
    /// has been saved so far.
    save: bool,
    /// How long any one query is given before the harness stops waiting for it.
    ///
    /// The suite's own default unless `--timeout` says otherwise, and `--timeout 0` means wait as
    /// long as it takes. A limit is not a nuisance to be tuned away: it is what lets a suite nobody
    /// has run before produce a table on the first attempt, with a cell per query saying which of
    /// them finished, instead of one engine's first hang standing in for the whole run.
    timeout: Option<Duration>,
}

/// Every engine this harness knows how to drive, in the order [`discover`] builds them.
const ENGINES: [&str; 7] = [
    "duckdb",
    "duckdb-pinned",
    "clickhouse-local",
    "datafusion",
    "polars",
    "clickhouse-server",
    "rudb",
];

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
    let mut engines = None;
    let mut rows = None;
    let mut scale = None;
    let mut report = false;
    let mut save = false;
    let mut timeout = None;
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
                // One is allowed, and it is not a hole in reporting rule two. A distribution under
                // five samples already says no to `Distribution::publishable`, so a run this small
                // cannot reach a published table, cannot be stored by `--store` and carries the
                // reason in its own report. What it can do is finish in a fifth of the time, which
                // is what the development loop wants and what a rule that refused it outright was
                // quietly costing.
                if n == 0 {
                    return Err("--runs 0 would measure nothing".to_owned());
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
            "--rows" => {
                let given =
                    rest.next().ok_or("--rows wants a row count after it, such as `--rows 1m`")?;
                rows = Some(Rows::parse(given)?);
                continue;
            }
            "--scale" => {
                let given = rest
                    .next()
                    .ok_or("--scale wants a scale factor after it, such as `--scale 1`")?;
                scale = Some(given.trim().trim_start_matches("sf").to_owned());
                continue;
            }
            "--timeout" => {
                let given = rest.next().ok_or(
                    "--timeout wants a number of seconds after it, such as `--timeout 120`",
                )?;
                let seconds: u64 = given
                    .parse()
                    .map_err(|e| format!("--timeout {given} is not a number of seconds, {e}"))?;
                // Zero is the way to ask for no limit at all, which is what somebody debugging a
                // query that takes twenty minutes wants. It is spelled as a value rather than as a
                // second flag because the two are the same decision and a run should not be able to
                // say both.
                timeout = Some(Duration::from_secs(seconds));
                continue;
            }
            "--report" => {
                report = true;
                continue;
            }
            "--save" => {
                save = true;
                continue;
            }
            "--engines" => {
                let given = rest.next().ok_or(
                    "--engines wants a comma separated list after it, such as \
                     `--engines duckdb,clickhouse-local`",
                )?;
                let wanted: Vec<String> = given
                    .split(',')
                    .map(str::trim)
                    .filter(|name| !name.is_empty())
                    .map(str::to_owned)
                    .collect();
                if wanted.is_empty() {
                    return Err(format!(
                        "--engines {given} names no engine, and a run of nothing is not a run"
                    ));
                }
                for name in &wanted {
                    if !ENGINES.contains(&name.as_str()) {
                        return Err(format!(
                            "--engines {name} is not an engine here, the ones there are: {}",
                            ENGINES.join(", ")
                        ));
                    }
                }
                engines = Some(wanted);
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
    // A committed record or a ledger row taken over a smaller version of the data would be a bar
    // every later full run clears by an order of magnitude and every later small run holds to a
    // number from a different file. Neither file says which it was, because a record is per query
    // and a run is per engine, so the two would sit next to each other looking comparable. The
    // refusal is here rather than in the files, where it would have to be a third shape of record.
    if rows.is_some() && (gate != Gate::Nothing || store.is_some()) {
        return Err("--rows is a development loop and --check, --check-drift, --record and \
                    --store are the committed history, so they do not go together. A record \
                    taken over a million rows is a bar that every full run clears without \
                    trying"
            .to_owned());
    }
    let suite = suite.unwrap_or_else(|| "smoke".to_owned());
    let scale = chosen_scale(&suite, scale.as_deref())?;
    // Worked out here rather than in the initializer, because the suite's own default needs the
    // suite and the suite has been moved into the plan by the time the field comes round.
    let timeout = match timeout {
        Some(given) if given.is_zero() => None,
        Some(given) => Some(given),
        None => rudb_bench::suite::find(&suite).map(|found| found.timeout(scale)),
    };
    Ok(Plan {
        suite,
        gate,
        runs: runs.unwrap_or_else(|| gate.runs()),
        store,
        engines,
        rows,
        scale,
        report,
        save,
        timeout,
    })
}

/// Turn what `--scale` was given into the scale the suite has by that name.
///
/// A suite this harness does not have is not an error here. `run` already says which suites there
/// are and says it in one place, and a second sentence about an unknown suite written here is a
/// second sentence to keep in step with the first.
fn chosen_scale(suite: &str, asked: Option<&str>) -> Result<Option<&'static Scale>, String> {
    let Some(asked) = asked else { return Ok(None) };
    let Some(suite) = rudb_bench::suite::find(suite) else { return Ok(None) };
    if suite.scales().is_empty() {
        return Err(format!(
            "the {} suite is one corpus of one size, so it takes no --scale. The suites that do \
             are the generated ones, which `rudb-bench suites` lists",
            suite.name
        ));
    }
    suite.scale(asked).map(Some).ok_or_else(|| {
        format!(
            "the {} suite has no scale factor {asked}, the ones it has are {}",
            suite.name,
            suite.scale_labels()
        )
    })
}

/// Build the cross engine table out of the runs saved on this machine.
///
/// The other half of `--save`. Each engine is measured on its own, which for ClickBench is the
/// difference between a command that finishes and one that falls over four hours in, and this is
/// what puts the columns back in one table afterwards. It measures nothing itself, so an engine that
/// is not saved is an abstention naming the command that would save it rather than a missing column.
fn report_saved(suite: &str) -> ExitCode {
    let here = machine::name_here();
    let compared = match rudb_bench::saved::restore(suite, &here, &ENGINES) {
        Ok(compared) => compared,
        Err(e) => {
            eprintln!("rudb-bench: {e}");
            return ExitCode::FAILURE;
        }
    };
    print!("{}", comparison(&compared));
    // No machine facts. Those are read off the machine as it is now and the saved runs were taken
    // over days, so printing today's governor next to Tuesday's numbers would be a claim about
    // Tuesday that nobody made. The per run reports keep theirs.
    match rudb_bench::markdown::write(&compared, &[], &here) {
        Ok(at) => {
            println!("\nwrote {}", at.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("rudb-bench: could not write the report: {e}");
            ExitCode::FAILURE
        }
    }
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

    let scratch = match scratch() {
        Ok(at) => at,
        Err(e) => {
            eprintln!("rudb-bench: cannot make the scratch directory: {e}");
            return ExitCode::FAILURE;
        }
    };
    let (mut engines, missing) = discover(&scratch, suite, plan.engines.as_deref());

    // The data before the engines, because every engine gets the same files and the first thing a
    // reader of a result asks is which files those were.
    let dataset = match rudb_bench::data::prepare(suite, &scratch, plan.rows.as_ref(), plan.scale) {
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

    let mut compared = rudb_bench::report::compare(
        &mut engines,
        suite,
        queries,
        &dataset,
        plan.runs,
        plan.timeout,
    );
    compared.skipped.extend(missing);

    for result in &compared.results {
        print!("{}", table(result));
        println!();
    }
    print!("{}", comparison(&compared));

    // Saved before the report is written, so that a run which saved and then could not write its
    // markdown has still kept the hours of measurement it just did.
    if plan.save {
        let here = machine::name_here();
        for result in &compared.results {
            match rudb_bench::saved::save(result, &here, compared.source_bytes) {
                Ok(at) => println!("saved {} to {}", result.engine, at.display()),
                Err(e) => eprintln!("rudb-bench: could not save {}: {e}", result.engine),
            }
        }
    }

    // Before the scratch directory goes, because the filesystem fact the machine record wants is
    // the one under the data and not the one under the binary, and on most of the fleet those are
    // different devices.
    if plan.report {
        let facts = machine::probe(&scratch);
        let here = machine::name_here();
        match rudb_bench::markdown::write(&compared, &facts, &here) {
            Ok(at) => println!("\nwrote {}", at.display()),
            // A report that could not be written is worth saying and is not worth failing a run
            // over. The numbers are already on the terminal and the alternative is an hour of
            // ClickBench thrown away because a directory was read only.
            Err(e) => eprintln!("\nrudb-bench: could not write the report: {e}"),
        }
    }

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
    let slower = verdicts.iter().any(regress::Verdict::failed);
    // Both gates run and both are reported even when the first one failed, because a run that is
    // slower and planning for longer is one finding and the second half of it is the explanation.
    // Stopping at the first failure would hide it behind the thing it caused.
    let over = budgets(compared);
    if slower || over { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

/// Compare what each query spent planning against the committed budget for it.
///
/// Its own gate rather than a column in the one above, because the two ask different questions and
/// only one of them can be asked anywhere. The regression gate compares this machine's run against
/// this machine's record, so it says nothing on a machine with no record. A planning budget is a
/// share of a run against itself, so it means the same thing on every machine and can be checked on
/// whatever ran.
fn budgets(compared: &rudb_bench::report::Comparison) -> bool {
    let at = planning::path(compared.suite.name);
    let budgets = match planning::read(&at) {
        Ok(budgets) => budgets,
        Err(e) => {
            eprintln!("rudb-bench: {e}");
            return true;
        }
    };
    let verdicts = planning::check_all(&budgets, compared);
    println!();
    println!("{}", at.display());
    print!("{}", planning::report(&verdicts, budgets.len()));
    verdicts.iter().any(planning::Verdict::failed)
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
    // The planning budgets come off the same run, because the alternative is two commands somebody
    // has to remember to run together and a budget file that is a month older than the records
    // beside it.
    let budgets: Vec<planning::Budget> = compared
        .results
        .iter()
        .map(|r| planning::Budget::of(r, &here, &today))
        .filter(|b| !b.queries.is_empty())
        .collect();
    if !budgets.is_empty() {
        let at = planning::path(compared.suite.name);
        if let Err(e) = planning::write(&at, &budgets) {
            eprintln!("rudb-bench: {e}");
            return ExitCode::FAILURE;
        }
        println!("{}", at.display());
        for budget in &budgets {
            println!("  {}", planning::describe(budget));
        }
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

/// Measure rudb's kernels in rudb's own process, and check or record what they cost.
///
/// The one subcommand here that does not start an engine and send it SQL, because a kernel cannot be
/// reached that way. It runs `cargo xtask kernels --json` in a rudb checkout and keeps the answer.
/// Everything it adds over reading that task's table by eye is in [`rudb_bench::kernels`]: a machine
/// name on the numbers, a committed file, and a comparison against the last run at a threshold the
/// measurement can support.
///
/// It is not a [`Suite`], and that is a decision rather than an omission. A suite is queries over
/// tables against every engine on the machine, all three of which this is not, and forcing it into
/// that shape would mean a `queries: 0` entry that the report code has to keep making exceptions
/// for. The `micro` suite entry stays what it is, a description of the category.
fn kernels(args: &[String]) -> ExitCode {
    let mut record = false;
    let mut check = false;
    let mut repo = None;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--record" => record = true,
            "--check" => check = true,
            "--repo" => match rest.next() {
                Some(path) => repo = Some(path.clone()),
                None => {
                    eprintln!("rudb-bench: --repo needs a path");
                    return ExitCode::FAILURE;
                }
            },
            other => {
                eprintln!("rudb-bench: unknown argument {other}");
                return ExitCode::FAILURE;
            }
        }
    }

    let repo = kernels::repo(repo.as_deref());
    println!("measuring the kernels in {}", repo.display());
    println!("this builds rudb under the bench profile first, so the first run is a few minutes");
    println!();
    let cells = match kernels::measure(&repo) {
        Ok(cells) => cells,
        Err(e) => {
            eprintln!("rudb-bench: {e}");
            return ExitCode::FAILURE;
        }
    };
    let rudb = kernels::describe(&repo);
    println!("rudb {rudb} on {}", machine::name_here());
    println!("{}", kernels::summary(&cells));
    println!();

    let at = kernels::path();
    if check {
        let records = match kernels::read(&at) {
            Ok(records) => records,
            Err(e) => {
                eprintln!("rudb-bench: {e}");
                return ExitCode::FAILURE;
            }
        };
        let here = machine::name_here();
        let Some(against) = records.iter().find(|old| old.machine == here) else {
            // Not a failure. The first run on a machine has nothing to compare against, and a gate
            // that fails on a machine nobody has recorded yet is a gate that stops the first person
            // to try it rather than the change that broke something.
            println!("no record for {here} in {}, so there is nothing to check", at.display());
            println!("take one with `rudb-bench kernels --record`");
            return ExitCode::SUCCESS;
        };
        println!("against {} measured on {}", against.rudb, against.recorded);
        let (text, failed) = kernels::report(&kernels::compare(against, &cells));
        print!("{text}");
        if failed {
            return ExitCode::FAILURE;
        }
    }

    if record {
        let taken = kernels::record_of(cells, rudb);
        if let Err(e) = kernels::write(&at, taken) {
            eprintln!("rudb-bench: {e}");
            return ExitCode::FAILURE;
        }
        println!("recorded in {}", at.display());
    }
    ExitCode::SUCCESS
}

/// Where the rudb binary is, or the sentence naming the override.
fn rudb_binary() -> Result<std::path::PathBuf, String> {
    let found =
        std::env::var_os("RUDB_BENCH_RUDB").map(std::path::PathBuf::from).or_else(|| which("rudb"));
    found.ok_or_else(|| "no rudb on PATH, set RUDB_BENCH_RUDB".to_owned())
}

/// The first entry of `PATH` that holds an executable of this name.
fn which(name: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).map(|dir| dir.join(name)).find(|at| at.is_file())
}

/// Every seam the engine has, and what is registered at each of them.
///
/// A listing rather than a measurement, and the thing somebody runs before a sweep to find out what
/// there is to sweep. It asks the engine rather than holding a copy of the list, because a harness
/// with its own idea of what the seams are is a harness that sweeps a seam the engine renamed.
fn seams() -> ExitCode {
    let binary = match rudb_binary() {
        Ok(at) => at,
        Err(e) => {
            eprintln!("rudb-bench: {e}");
            return ExitCode::FAILURE;
        }
    };
    let all = match rudb_bench::sweep::seams(&binary) {
        Ok(all) => all,
        Err(e) => {
            eprintln!("rudb-bench: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("seam                  milestone  registered  what it is");
    for seam in &all {
        let registered = if seam.implementations.is_empty() {
            "reference".to_owned()
        } else {
            seam.implementations.iter().map(|i| i.name.as_str()).collect::<Vec<_>>().join(", ")
        };
        println!(
            "{:<20}  {:<9}  {:<10}  {}",
            seam.id, seam.milestone, registered, seam.description
        );
    }
    println!();
    println!(
        "`reference` in the registered column means nothing has been registered there yet and"
    );
    println!(
        "the engine runs the slow implementation kept for differential testing. A sweep of one"
    );
    println!(
        "of those prints one row, which is the apparatus working rather than a missing result."
    );
    println!();
    println!("Sweep one with `rudb-bench sweep --seam <seam> --suite <suite>`.");
    ExitCode::SUCCESS
}

/// Run one suite once per registered implementation of one seam, with everything else held fixed.
///
/// The command the strategy registry exists for. Every other subcommand here compares engines, and
/// this one compares the engine against itself with a single decision moved, which is the only shape
/// of measurement that can say what a decision was worth.
///
/// rudb only, and that is not a gap. No other engine here has a seam to move, and a sweep that ran
/// DuckDB alongside would be running a column that is the same number in every row.
fn sweep(args: &[String]) -> ExitCode {
    let mut seam = None;
    let mut suite_name = "smoke".to_owned();
    let mut runs = 5usize;
    let mut rows = None;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--seam" => match rest.next() {
                Some(name) => seam = Some(name.clone()),
                None => {
                    eprintln!(
                        "rudb-bench: --seam wants a seam name after it, try `rudb-bench seams`"
                    );
                    return ExitCode::FAILURE;
                }
            },
            "--suite" => match rest.next() {
                Some(name) => suite_name = name.clone(),
                None => {
                    eprintln!("rudb-bench: --suite wants a suite name after it");
                    return ExitCode::FAILURE;
                }
            },
            "--runs" => match rest.next().map(|n| n.parse::<usize>()) {
                Some(Ok(n)) if n > 0 => runs = n,
                _ => {
                    eprintln!("rudb-bench: --runs wants a number above zero after it");
                    return ExitCode::FAILURE;
                }
            },
            "--rows" => match rest.next().map(|given| Rows::parse(given)) {
                Some(Ok(parsed)) => rows = Some(parsed),
                Some(Err(e)) => {
                    eprintln!("rudb-bench: {e}");
                    return ExitCode::FAILURE;
                }
                None => {
                    eprintln!("rudb-bench: --rows wants a row count after it, such as `--rows 1m`");
                    return ExitCode::FAILURE;
                }
            },
            other => {
                eprintln!("rudb-bench: unknown argument {other}");
                eprintln!("rudb-bench: sweep --seam <seam> --suite <suite> [--runs n] [--rows n]");
                return ExitCode::FAILURE;
            }
        }
    }
    let Some(wanted) = seam else {
        eprintln!("rudb-bench: sweep needs --seam, and `rudb-bench seams` lists them");
        return ExitCode::FAILURE;
    };
    let binary = match rudb_binary() {
        Ok(at) => at,
        Err(e) => {
            eprintln!("rudb-bench: {e}");
            return ExitCode::FAILURE;
        }
    };
    let all = match rudb_bench::sweep::seams(&binary) {
        Ok(all) => all,
        Err(e) => {
            eprintln!("rudb-bench: {e}");
            return ExitCode::FAILURE;
        }
    };
    let found = match rudb_bench::sweep::find(&all, &wanted) {
        Ok(found) => found.clone(),
        Err(e) => {
            eprintln!("rudb-bench: {e}");
            return ExitCode::FAILURE;
        }
    };
    let Some(suite) = rudb_bench::suite::find(&suite_name) else {
        eprintln!("rudb-bench: no suite called {suite_name}");
        eprintln!("rudb-bench: try `rudb-bench suites`");
        return ExitCode::FAILURE;
    };
    let Some(queries) = queries(&suite_name) else {
        eprintln!("rudb-bench: the {suite_name} suite needs {}", suite.needs);
        return ExitCode::FAILURE;
    };
    let scratch = match scratch() {
        Ok(at) => at,
        Err(e) => {
            eprintln!("rudb-bench: cannot make the scratch directory: {e}");
            return ExitCode::FAILURE;
        }
    };
    // One dataset for every row, made once and handed to each of them. Making it per row would be
    // the same bytes in a different order on the device, which is exactly the kind of difference a
    // sweep is supposed to be free of.
    let dataset = match rudb_bench::data::prepare(suite, &scratch, rows.as_ref(), None) {
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
    // The suite's own limit. This command has no flag for it, because a sweep is a comparison
    // between variants of one engine and a variant that hangs is the answer rather than a run to
    // abandon.
    let limit = Some(suite.timeout(None));
    let swept = rudb_bench::sweep::sweep(&found, suite, queries, &dataset, &scratch, runs, limit);
    print!("{}", rudb_bench::sweep::table(&swept));
    let _ = std::fs::remove_dir_all(&scratch);
    if swept.rows.iter().all(|row| row.result.is_err()) {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// Run one suite twice on one engine, with one layer on and then off, per query.
///
/// The sibling of `sweep` and the same idea one level up. `sweep` moves a single decision inside the
/// engine and holds the rest fixed; this turns a whole layer off and holds the rest fixed. What it
/// produces is an attribution and not a claim, for the reason [`rudb_bench::attribute`] gives at
/// length: the per query column is what says where the layer is worth something and, more usefully,
/// where it is not.
///
/// Which layer is `--off`: the optimizer by default, and otherwise a rudb rule by name, which is
/// how `statistics` and `graph_sections` get the same treatment without a second command that would
/// slowly stop agreeing with this one.
///
/// One engine rather than every engine, because the two runs have to be the same engine for the
/// difference between them to mean anything, and a table with a column per engine would invite
/// exactly the comparison this does not make. Which engine is a flag, so that DuckDB can be
/// attributed the same way, which is the only way to find out whether a number this prints about
/// rudb is a number about rudb or a number about the apparatus.
///
/// It exits non-zero when the two runs answered differently, which is the one result here that is
/// a property of the engine rather than of the machine and the only one worth failing a job over.
fn attribute(args: &[String]) -> ExitCode {
    let mut engine_name = "rudb".to_owned();
    let mut suite_name = "smoke".to_owned();
    let mut hot = 5usize;
    let mut rows = None;
    let mut ablation = Ablation::Optimizer;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--engine" => match rest.next() {
                Some(name) => engine_name = name.clone(),
                None => {
                    eprintln!("rudb-bench: --engine wants an engine name after it");
                    return ExitCode::FAILURE;
                }
            },
            "--suite" => match rest.next() {
                Some(name) => suite_name = name.clone(),
                None => {
                    eprintln!("rudb-bench: --suite wants a suite name after it");
                    return ExitCode::FAILURE;
                }
            },
            // Not checked against a list here. Which rules exist is the engine's to say, and a copy
            // of that list in the harness would be wrong on the day rudb gains one.
            "--off" => match rest.next() {
                Some(what) => ablation = Ablation::parse(what),
                None => {
                    eprintln!(
                        "rudb-bench: --off wants something to turn off after it, such as \
                         `--off statistics`"
                    );
                    return ExitCode::FAILURE;
                }
            },
            "--hot" => match rest.next().map(|n| n.parse::<usize>()) {
                Some(Ok(n)) if n > 0 => hot = n,
                _ => {
                    eprintln!("rudb-bench: --hot wants a number above zero after it");
                    return ExitCode::FAILURE;
                }
            },
            "--rows" => match rest.next().map(|given| Rows::parse(given)) {
                Some(Ok(parsed)) => rows = Some(parsed),
                Some(Err(e)) => {
                    eprintln!("rudb-bench: {e}");
                    return ExitCode::FAILURE;
                }
                None => {
                    eprintln!("rudb-bench: --rows wants a row count after it, such as `--rows 1m`");
                    return ExitCode::FAILURE;
                }
            },
            other => {
                eprintln!("rudb-bench: unknown argument {other}");
                eprintln!(
                    "rudb-bench: attribute [--engine e] [--suite s] [--hot n] [--rows n] \
                     [--off what]"
                );
                return ExitCode::FAILURE;
            }
        }
    }
    let Some(suite) = rudb_bench::suite::find(&suite_name) else {
        eprintln!("rudb-bench: no suite called {suite_name}");
        eprintln!("rudb-bench: try `rudb-bench suites`");
        return ExitCode::FAILURE;
    };
    let Some(queries) = queries(&suite_name) else {
        eprintln!("rudb-bench: the {suite_name} suite needs {}", suite.needs);
        return ExitCode::FAILURE;
    };
    let scratch = match scratch() {
        Ok(at) => at,
        Err(e) => {
            eprintln!("rudb-bench: cannot make the scratch directory: {e}");
            return ExitCode::FAILURE;
        }
    };
    let asked = [engine_name.clone()];
    let (mut engines, missing) = discover(&scratch, suite, Some(&asked));
    let Some(engine) = engines.pop() else {
        // The abstention rather than a shorter sentence of this command's own, because the reason
        // an engine is not here has already been worked out once and saying it differently in two
        // places is how the two start disagreeing.
        match missing.iter().find(|one| one.engine == engine_name && !one.unasked) {
            Some(one) => eprintln!("rudb-bench: {engine_name} is not here: {}", one.why),
            None => eprintln!("rudb-bench: no engine called {engine_name}"),
        }
        let _ = std::fs::remove_dir_all(&scratch);
        return ExitCode::FAILURE;
    };
    let dataset = match rudb_bench::data::prepare(suite, &scratch, rows.as_ref(), None) {
        Ok(dataset) => dataset,
        Err(e) => {
            eprintln!("rudb-bench: {e}");
            let _ = std::fs::remove_dir_all(&scratch);
            return ExitCode::FAILURE;
        }
    };
    let limit = Some(suite.timeout(None));
    let attributed =
        rudb_bench::attribute::attribute(engine, suite, queries, &dataset, hot, limit, ablation);
    let _ = std::fs::remove_dir_all(&scratch);
    match attributed {
        Ok(attributed) => {
            print!("{}", rudb_bench::attribute::table(&attributed));
            if attributed.agreed() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
        }
        Err(e) => {
            eprintln!("rudb-bench: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Check every committed plan baseline, or record one.
///
/// The one subcommand here that measures nothing. It asks rudb to `EXPLAIN` each query in a suite
/// against the zero row tables in `fixtures/`, settles the machine out of what came back, and
/// compares it to `baselines/plans-<suite>.txt`. Milestone E1 asks for it so that a plan change is a
/// reviewed diff rather than something noticed three weeks later in a performance run, and
/// [`rudb_bench::plans`] makes the argument at length.
///
/// No `--suite` means every suite that has fixtures committed for it, which is what CI runs and
/// what makes adding a suite a matter of committing its empty tables rather than of editing a
/// workflow. `--record` writes the file instead of checking it, and takes one suite at a time,
/// because recording everything at once is how a plan change in the suite nobody was looking at
/// gets committed along with the one somebody meant.
///
/// It exits non-zero on any change at all, including a query that started binding. The reasoning is
/// in [`rudb_bench::plans::Change::fails`]: a category of plan change that only printed a warning
/// would be the category people stop reading.
fn plans(args: &[String]) -> ExitCode {
    let mut wanted: Option<String> = None;
    let mut record = false;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--record" => record = true,
            "--suite" => match rest.next() {
                Some(name) => wanted = Some(name.clone()),
                None => {
                    eprintln!("rudb-bench: --suite wants a suite name after it");
                    return ExitCode::FAILURE;
                }
            },
            other => {
                eprintln!("rudb-bench: unknown argument {other}");
                eprintln!("rudb-bench: plans [--suite s] [--record]");
                return ExitCode::FAILURE;
            }
        }
    }
    let root = rudb_bench::plans::root();
    let chosen: Vec<&'static Suite> = match wanted.as_deref() {
        Some(name) => match rudb_bench::suite::find(name) {
            Some(suite) => vec![suite],
            None => {
                eprintln!("rudb-bench: no suite called {name}");
                eprintln!("rudb-bench: try `rudb-bench suites`");
                return ExitCode::FAILURE;
            }
        },
        // Every suite whose empty tables are committed, which today is two of the five. The other
        // three have no fixtures because nobody has written down a schema for them that is the
        // schema of the real file, and a baseline captured against a guessed schema would be worse
        // than no baseline: it would pass, and it would be a plan of a query over a table that does
        // not exist anywhere.
        None => SUITES
            .iter()
            .filter(|suite| rudb_bench::plans::fixtures(&root, suite.name).is_dir())
            .collect(),
    };
    if chosen.is_empty() {
        eprintln!("rudb-bench: no suite here has fixtures committed, so there is nothing to check");
        eprintln!("rudb-bench: see fixtures/README.md for what one is and how it was made");
        return ExitCode::FAILURE;
    }
    if record && chosen.len() > 1 {
        eprintln!("rudb-bench: --record takes one suite at a time, so say which with --suite");
        return ExitCode::FAILURE;
    }

    let scratch = match scratch() {
        Ok(at) => at,
        Err(e) => {
            eprintln!("rudb-bench: cannot make the scratch directory: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut worst = ExitCode::SUCCESS;
    for suite in chosen {
        // A fresh engine per suite, because the view declarations a capture leaves behind are the
        // previous suite's tables and a plan over those is a plan of a different query.
        let mut engine = Rudb::discover(&scratch, suite);
        // Asked about the smoke suite and not about this one, which is deliberate. rudb declines
        // TPC-H because every join in it is a nested loop, and that is a statement about execution:
        // planning executes nothing, and that refusal is exactly why nobody has ever found out
        // which of the twenty two queries rudb can bind. Smoke is the suite whose only reason to be
        // declined is that rudb is not built, which is the one refusal that does apply here.
        if let Ability::No(why) = engine.can_run(built()) {
            eprintln!("rudb-bench: {why}");
            let _ = std::fs::remove_dir_all(&scratch);
            return ExitCode::FAILURE;
        }
        match one_suite(&root, &mut engine, suite, record) {
            Ok(false) => worst = ExitCode::FAILURE,
            Ok(true) => {}
            Err(e) => {
                eprintln!("rudb-bench: {e}");
                worst = ExitCode::FAILURE;
            }
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
    worst
}

/// The smoke suite, which is the one rudb declines only when it is not built.
fn built() -> &'static Suite {
    rudb_bench::suite::find("smoke").expect("the smoke suite is compiled in")
}

/// Capture one suite's plans, and either write them down or say what moved.
///
/// `Ok(true)` when there was nothing to say. Separate from [`plans`] so that a suite failing is one
/// suite failing: a check over two suites should report both rather than stop at the first, because
/// the second one's diff is the thing somebody is about to need.
fn one_suite(
    root: &std::path::Path,
    engine: &mut dyn Engine,
    suite: &'static Suite,
    record: bool,
) -> Result<bool, String> {
    let Some(queries) = queries(suite.name) else {
        return Err(format!("the {} suite needs {}", suite.name, suite.needs));
    };
    let tables = rudb_bench::plans::tables(root, suite)?;
    let today = regress::today();
    let captured = rudb_bench::plans::capture(engine, suite, queries, &tables, &today)?;
    let at = rudb_bench::plans::path(root, suite.name);

    if record {
        let text = rudb_bench::plans::render(&captured);
        if let Some(under) = at.parent() {
            std::fs::create_dir_all(under)
                .map_err(|e| format!("cannot make {}: {e}", under.display()))?;
        }
        std::fs::write(&at, text).map_err(|e| format!("cannot write {}: {e}", at.display()))?;
        println!(
            "wrote {}: {} plans, {} refused, against {}",
            at.display(),
            captured.plans.len(),
            captured.refused.len(),
            captured.version
        );
        return Ok(true);
    }

    let text = std::fs::read_to_string(&at).map_err(|e| {
        format!(
            "{}: {e}. Record it with `rudb-bench plans --suite {} --record`",
            at.display(),
            suite.name
        )
    })?;
    let committed =
        rudb_bench::plans::parse(&text).map_err(|e| format!("{}: {e}", at.display()))?;
    let changes = rudb_bench::plans::compare(&committed, &captured);
    print!("{}", rudb_bench::plans::report(suite.name, &changes));
    Ok(changes.iter().all(|change| !change.fails()))
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
///
/// `asked` is the `--engines` filter and `None` means every engine here. An engine the filter left
/// out comes back as an abstention rather than as nothing at all, so that a table produced by a
/// focused run cannot be mistaken for a table produced by a full one.
fn discover(
    scratch: &std::path::Path,
    suite: &'static Suite,
    asked: Option<&[String]>,
) -> (Vec<Box<dyn Engine>>, Vec<Abstention>) {
    let mut engines: Vec<Box<dyn Engine>> = Vec::new();
    let mut missing = Vec::new();
    let wanted = |name: &str| asked.is_none_or(|list| list.iter().any(|one| one == name));

    macro_rules! consider {
        ($name:literal, $found:expr) => {
            if wanted($name) {
                match $found {
                    Ok(engine) => engines.push(Box::new(engine)),
                    Err(e) => missing.push(gap($name, &e)),
                }
            } else {
                missing.push(unasked($name));
            }
        };
    }

    consider!("duckdb", Duckdb::discover(scratch, suite));
    consider!("duckdb-pinned", Duckdb::discover_pinned(scratch, suite));
    consider!("clickhouse-local", ClickhouseLocal::discover(scratch, suite));
    consider!("datafusion", Datafusion::discover(scratch, suite));
    consider!("polars", Polars::discover(scratch, suite));
    consider!("clickhouse-server", ClickhouseServer::discover(scratch, suite));

    // rudb is always in the list, because an engine that is missing from a comparison because it
    // could not have been built is a different thing from one that abstains, and only one of those
    // is where this project actually is. The filter still applies to it, because a run that asked
    // for four rivals and got five rows would be a filter that does not mean what it says.
    if wanted("rudb") {
        engines.push(Box::new(Rudb::discover(scratch, suite)));
    } else {
        missing.push(unasked("rudb"));
    }
    (engines, missing)
}

/// An engine that is not on this machine, as a line of the report.
fn gap(what: &str, why: &BenchError) -> Abstention {
    Abstention {
        engine: what.to_owned(),
        version: "not found".to_owned(),
        why: why.to_string(),
        unasked: false,
    }
}

/// An engine that is on this machine and was left out of this run on purpose.
fn unasked(what: &str) -> Abstention {
    Abstention {
        engine: what.to_owned(),
        version: "not asked for".to_owned(),
        why: "left out of this run by --engines".to_owned(),
        unasked: true,
    }
}

/// Where a run puts its data, which is a directory of its own that it makes and then removes.
///
/// Under the temporary directory by default and under `RUDB_BENCH_SCRATCH` when that names a place,
/// because on three of the four machines in the fleet the temporary directory is on the small device
/// and the data has to go somewhere else. Either way the directory is named after this process, for
/// two reasons. Two runs on one machine do not write over each other, and the directory removed at
/// the end of a run is one this made rather than one somebody pointed at, which matters because the
/// removal is a `remove_dir_all` and the variable is set by hand.
///
/// It is made here rather than by whoever writes into it first. Nothing was making it when the
/// suite's data was already on the machine, because the only `create_dir_all` on that path is in
/// the generator, so every engine failed with `No such file or directory` on a run that had nothing
/// to generate.
fn scratch() -> Result<std::path::PathBuf, std::io::Error> {
    let under = std::env::var_os("RUDB_BENCH_SCRATCH")
        .map_or_else(std::env::temp_dir, std::path::PathBuf::from);
    scratch_under(&under)
}

/// The directory a run works in, under the place it was told to work, made if it is not there.
///
/// Split out from [`scratch`] so a test can ask for one without setting an environment variable,
/// which is unsafe in this edition and is shared with every other test in the process anyway.
fn scratch_under(under: &std::path::Path) -> Result<std::path::PathBuf, std::io::Error> {
    let at = under.join(format!("rudb-bench-{}", std::process::id()));
    std::fs::create_dir_all(&at)?;
    Ok(at)
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
    println!("                    and the planning budgets, for the engine that reports one");
    println!("                    (--check also fails a query that spent a larger share of");
    println!("                    itself planning than baselines/planning-<suite>.txt allows,");
    println!("                    which is a share rather than a time and so travels)");
    println!("    --store <layer> add this run to the ledger as the row that closes a layer");
    println!("    --engines a,b   run only these, the rest abstain saying they were left out");
    println!("    --runs n        hot runs per query, default five and fifteen. Under five is a");
    println!("                    development number and cannot be published or stored");
    println!("                    for anything that reads or writes a record");
    println!("    --rows n        run over this many rows instead of the whole table, as 1m or");
    println!("                    200k, for the development loop. Not comparable to anything,");
    println!("                    and not allowed with the flags that write a record");
    println!("    --scale n       which scale factor of a generated suite to run, such as 1 or");
    println!("                    100. A corpus the generator built, so a number out of one is");
    println!("                    comparable to every other number at the same scale");
    println!("    --timeout n     seconds one query gets before the harness stops waiting for");
    println!("                    it, default the suite's own. A query that runs out of it is a");
    println!("                    row in the table saying so and the rest of the suite carries");
    println!("                    on, and it is left out of the ratio. 0 waits as long as it");
    println!("                    takes, which is what a query being debugged wants");
    println!("    --save          add what each engine measured to reports/saved-<suite>-");
    println!("                    <machine>.txt, so that engines run on separate days end up");
    println!("                    in one table. Re-running an engine replaces its block");
    println!(
        "    --report        also write the whole run to reports/<date>/run-<suite>-<machine>.md,"
    );
    println!("                    which keeps every metric the terminal table has to drop");
    println!("  ledger        print what each layer bought, from the committed runs");
    println!("  seams         print rudb's seams and what is registered at each of them");
    println!("  sweep         run a suite once per implementation of one seam, rudb only");
    println!("    --seam <seam>   the seam to move, from `rudb-bench seams`");
    println!("    --suite <name>  the suite to hold fixed, default smoke");
    println!("    --runs n        hot runs per query, default five");
    println!("    --rows n        run over this many rows instead of the whole table");
    println!("  attribute     run a suite twice on one engine, with one layer on and then off,");
    println!("                and print what that layer was worth per query. Exits non-zero");
    println!("                when the two runs answered differently, which is an engine bug");
    println!("    --engine <name> the engine to attribute, default rudb");
    println!("    --suite <name>  the suite to run twice, default smoke");
    println!("    --hot n         hot runs per query, default five");
    println!("    --rows n        run over this many rows instead of the whole table");
    println!("    --off <what>    the layer to turn off, default optimizer. Anything else is a");
    println!("                    rudb rule name, such as statistics or graph_sections, and both");
    println!("                    runs set it rather than leaning on whichever way it defaults");
    println!("  plans         check the committed plan baselines, rudb only. Asks for the plan of");
    println!("                every query in a suite against the zero row tables in fixtures/ and");
    println!("                fails when one is not the plan that was written down. Measures");
    println!("                nothing and needs no data");
    println!("    --suite <name>  one suite, default every suite that has fixtures committed");
    println!("    --record        write baselines/plans-<suite>.txt instead of checking it");
    println!("  kernels       measure rudb's own loops in rudb's process, per row");
    println!("    --repo <path>   the rudb checkout, default ../rudb");
    println!("    --record        replace this machine's block in baselines/kernels.txt");
    println!(
        "    --check         fail on a cell {FACTOR:.0}x slower, and on any cell that lost its loop"
    );
    println!("  load          load a suite's data into each engine and time it");
    println!("  report [suite] build the cross engine table out of the saved runs on this");
    println!("                machine, for the engines measured one at a time with --save");
    println!("  -V, --version print the version and exit");
    println!();
    println!("  RUDB_BENCH_DUCKDB       the DuckDB binary, which is the reference column");
    println!("  RUDB_BENCH_DUCKDB_PINNED  the DuckDB the grammar is vendored from, which is the");
    println!("                          compatibility target and is a second row. There is no");
    println!("                          release at that commit, so scripts/oracle in the rudb");
    println!("                          checkout is what installs it");
    println!("  RUDB_BENCH_CLICKHOUSE   the ClickHouse binary, driven both as `local` and as a");
    println!("                          real server with a sorting key, which are two rows");
    println!("  RUDB_BENCH_DATAFUSION   the datafusion-cli binary");
    println!("  RUDB_BENCH_PYTHON       a python3 with polars installed");
    println!("  RUDB_BENCH_RUDB         the rudb binary, when there is one worth running");
    println!("  RUDB_BENCH_DATA         where the corpora live, default ~/rudb-data");
    println!("  RUDB_BENCH_SCRATCH      where a run makes the directory it works in and removes");
    println!("                          afterwards, default the temporary directory");
    println!("  RUDB_BENCH_MACHINE      what to call this machine in a committed record, needed");
    println!("                          only on a machine the fleet table does not know");
    println!("  RUDB_BENCH_PROGRESS     say which query is running, on stderr, for the suites");
    println!("                          that take hours and otherwise say nothing until the end");
    println!("  RUDB_BENCH_BASELINE     the records file, default baselines/<suite>.txt");
    println!("  RUDB_BENCH_RUDB_REPO    the rudb checkout the kernel suite measures");
    println!("  RUDB_BENCH_KERNELS      the kernel records file, default baselines/kernels.txt");
    println!("  RUDB_BENCH_ROOT         the checkout the plan baselines and fixtures are read out");
    println!("                          of, default the working directory");
    println!();
    println!("`run` works on the smoke suite, which generates its own data and measures nothing");
    println!("anybody should quote. The suites that matter need a download or a generator and");
    println!("`suites` says which. The design is spec/15-rudb-bench.md in");
    println!("https://github.com/tamnd/rudb.");
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Gate, plan, scratch_under};
    use rudb_bench::measure::Distribution;
    use rudb_bench::regress::Watch;

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_owned).collect()
    }

    #[test]
    fn a_run_that_says_nothing_about_a_limit_gets_the_suite_s_own() {
        let got = plan(&args("tpch")).expect("tpch is a suite");
        let suite = rudb_bench::suite::find("tpch").expect("tpch is a suite");
        assert_eq!(got.timeout, Some(suite.timeout(None)));
    }

    #[test]
    fn a_limit_given_in_seconds_is_the_limit() {
        let got = plan(&args("tpch --timeout 45")).expect("tpch is a suite");
        assert_eq!(got.timeout, Some(Duration::from_secs(45)));
    }

    /// Zero is how a query being debugged asks to be waited for however long it takes. It is a
    /// value rather than a second flag, because the two are one decision and a run should not be
    /// able to say both.
    #[test]
    fn a_limit_of_zero_is_no_limit_at_all() {
        let got = plan(&args("tpch --timeout 0")).expect("tpch is a suite");
        assert_eq!(got.timeout, None);
    }

    #[test]
    fn a_limit_that_is_not_a_number_of_seconds_says_so() {
        let got = plan(&args("tpch --timeout soon"));
        let why = got.expect_err("soon is not a number");
        assert!(why.contains("--timeout soon"), "{why}");
        let got = plan(&args("tpch --timeout"));
        assert!(got.expect_err("nothing follows it").contains("--timeout"));
    }

    /// The scale moves the data and the limit is taken off the data, so the two move together.
    #[test]
    fn the_default_limit_follows_the_scale_the_run_asked_for() {
        let small = plan(&args("tpch --scale 1")).expect("tpch runs at scale 1");
        let large = plan(&args("tpch --scale 100")).expect("tpch runs at scale 100");
        assert!(large.timeout > small.timeout, "{:?} then {:?}", small.timeout, large.timeout);
    }

    /// The directory is made, and it is under the place rather than being the place.
    ///
    /// Both halves matter. Nothing made it before, so a run with nothing to generate had every
    /// engine fail with `No such file or directory`, and the end of a run removes it with a
    /// `remove_dir_all`, which is not something to point at a directory somebody named by hand.
    #[test]
    fn the_scratch_directory_is_made_under_the_place_it_was_given() {
        let under = std::env::temp_dir().join("rudb-bench-scratch-test");
        let at = scratch_under(&under).expect("a directory under the temporary directory");
        assert!(at.is_dir(), "{} was not made", at.display());
        assert_eq!(at.parent(), Some(under.as_path()));
        assert!(scratch_under(&under).is_ok(), "asking twice is not an error");
        std::fs::remove_dir_all(&under).expect("the test cleans up after itself");
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
    fn a_run_smaller_than_rule_two_wants_is_allowed_and_cannot_be_published() {
        // Allowed, because the development loop wants an answer in a fifth of the time and the
        // rule is enforced where it matters rather than at the flag. A single run produces a
        // distribution that says no to `publishable`, so it cannot reach a published table and
        // cannot be stored, and its own report carries the reason.
        assert_eq!(plan(&args("smoke --runs 1")).unwrap().runs, 1);
        assert_eq!(plan(&args("smoke --runs 3")).unwrap().runs, 3);
        assert!(!Distribution::median(vec![Duration::from_millis(1)]).publishable());
        // Zero is still refused, because it is not a fast measurement, it is no measurement.
        assert!(plan(&args("smoke --runs 0")).is_err());
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

    #[test]
    fn asking_for_nothing_in_particular_asks_for_every_engine() {
        assert_eq!(plan(&args("tpch")).unwrap().engines, None);
    }

    #[test]
    fn the_engine_filter_keeps_the_order_and_the_spelling_it_was_given() {
        let got = plan(&args("tpch --engines duckdb,clickhouse-local,datafusion")).unwrap();
        assert_eq!(
            got.engines,
            Some(vec!["duckdb".to_owned(), "clickhouse-local".to_owned(), "datafusion".to_owned()])
        );
        assert_eq!(got.suite, "tpch");
    }

    #[test]
    fn an_engine_nobody_here_has_heard_of_is_refused_with_the_list_of_the_ones_there_are() {
        // Typing `clickhouse` when the engine is called `clickhouse-local` is the way this gets
        // used wrong, and silently running five engines instead of one would hide it for an hour.
        let e = plan(&args("tpch --engines clickhouse")).unwrap_err();
        assert!(e.contains("clickhouse-local"), "{e}");
        assert!(plan(&args("tpch --engines")).is_err());
        assert!(plan(&args("tpch --engines ,,")).is_err());
    }

    #[test]
    fn a_row_count_is_read_the_way_people_write_one() {
        assert_eq!(plan(&args("clickbench --rows 1m")).unwrap().rows.unwrap().wanted, 1_000_000);
        assert_eq!(plan(&args("clickbench")).unwrap().rows, None);
        assert!(plan(&args("clickbench --rows")).is_err());
        assert!(plan(&args("clickbench --rows soon")).is_err());
    }

    #[test]
    fn a_scale_factor_is_read_as_the_scale_the_suite_has() {
        let got = plan(&args("tpch --scale 1")).expect("tpch has an SF1");
        assert_eq!(got.scale.map(|scale| scale.label), Some("1"));
        // Typing what the directory is called rather than what the flag takes is the obvious near
        // miss, so it is taken rather than refused.
        assert_eq!(plan(&args("tpch --scale sf10")).unwrap().scale.map(|s| s.label), Some("10"));
        // Nothing said is the suite's own default, which is what the corpus on the fleet is.
        assert_eq!(plan(&args("tpch")).unwrap().scale, None);
        assert!(plan(&args("tpch --scale")).is_err());
    }

    #[test]
    fn a_scale_nobody_generated_is_refused_with_the_ones_there_are() {
        let e = plan(&args("tpch --scale 50")).unwrap_err();
        assert!(e.contains("0.01, 1, 10, 100, 1000"), "{e}");
        // ClickBench is one file of one size, so a scale factor is a question it does not answer.
        let e = plan(&args("clickbench --scale 10")).unwrap_err();
        assert!(e.contains("one corpus of one size"), "{e}");
    }

    /// The committed record and the ledger are a history, and a row of either taken over a million
    /// rows would sit next to rows taken over a hundred million looking like the same measurement.
    #[test]
    fn a_smaller_run_cannot_write_itself_into_the_committed_history() {
        for line in [
            "clickbench --rows 1m --record",
            "clickbench --rows 1m --check",
            "clickbench --rows 1m --check-drift",
            "clickbench --rows 1m --store 2b",
        ] {
            let e = plan(&args(line)).unwrap_err();
            assert!(e.contains("development loop"), "{line}: {e}");
        }
        assert!(plan(&args("clickbench --rows 1m")).is_ok());
    }

    /// The markdown file is asked for and never written by default, because a run that dirtied the
    /// checkout every time it was invoked is a run people stop invoking from the checkout.
    #[test]
    fn the_markdown_report_is_off_until_it_is_asked_for() {
        assert!(!plan(&args("clickbench")).unwrap().report);
        assert!(plan(&args("clickbench --report")).unwrap().report);
        // It says nothing about the numbers, so it goes with anything, including a smaller run and
        // the gate flags that a smaller run is refused with.
        assert!(plan(&args("clickbench --rows 1m --report")).unwrap().report);
        assert!(plan(&args("smoke --record --report")).unwrap().report);
    }

    #[test]
    fn every_name_the_filter_accepts_is_a_name_discovery_actually_builds() {
        // The two lists are written in two places, and the day one of them gains an engine is the
        // day the other one has to. This is the test that says so.
        for name in super::ENGINES {
            assert!(plan(&args(&format!("tpch --engines {name}"))).is_ok(), "{name}");
        }
        assert_eq!(super::ENGINES.len(), 7);
    }

    #[test]
    fn duckdb_is_two_rows_because_the_released_one_is_not_the_one_we_match() {
        // The released DuckDB is the rival. The pinned one is the compatibility target, which is a
        // build off the v2.0 branch with a different grammar. A table with only the first in it is
        // comparing against something other than the thing being matched, so both are askable and
        // both are askable on their own.
        assert!(plan(&args("clickbench --engines duckdb")).is_ok());
        assert!(plan(&args("clickbench --engines duckdb-pinned")).is_ok());
        let both = plan(&args("clickbench --engines duckdb,duckdb-pinned")).unwrap();
        assert_eq!(both.engines.unwrap().len(), 2);
    }
}
