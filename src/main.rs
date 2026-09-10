//! The harness command line.
//!
//! At this commit it prints the recorded baselines and says plainly that it cannot run anything
//! yet. The subcommands are named now because `spec/15-rudb-bench.md` describes them and because
//! the nightly job in the rudb repository will call them by name.

#![forbid(unsafe_code)]

use std::process::ExitCode;

use rudb_bench::{CLICKBENCH_C6A_4XLARGE, target_seconds};

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
        Some("run" | "load" | "report" | "machine") => {
            eprintln!("rudb-bench: there is no engine to measure yet");
            eprintln!("rudb-bench: this arrives with M1, see spec/17-milestones.md in tamnd/rudb");
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

fn help() {
    println!("rudb-bench {VERSION}");
    println!("The benchmark harness for rudb.");
    println!();
    println!("Usage: rudb-bench <command> [options]");
    println!();
    println!("  baselines     print the recorded board and the number we have to reach");
    println!("  machine       record the machine, the frequency policy and the mount options");
    println!("  load          load a suite's data into each engine and time it");
    println!("  run           run a suite against each engine and record the distribution");
    println!("  report        write the per-query table, losses included");
    println!("  -V, --version print the version and exit");
    println!();
    println!("Only `baselines` works. The rest arrives with M1, which is the milestone where the");
    println!("apparatus is used in anger. The design is spec/15-rudb-bench.md in");
    println!("https://github.com/tamnd/rudb.");
}
