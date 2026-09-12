//! The benchmark harness for rudb.
//!
//! The design is `spec/15-rudb-bench.md` in the [rudb repository]. The whole project's claim is a
//! performance claim, which means its credibility rests on the honesty of these measurements more
//! than on any technical decision in the engine. A benchmark number without its methodology is
//! marketing.
//!
//! rudb cannot run a query yet, so what is measured today is a real DuckDB on a small generated
//! dataset. That is not a placeholder. Every reporting rule in section 15.1 is a property of the
//! apparatus rather than of the engine, and every one of them is easier to build now, against an
//! engine nobody has any stake in, than on the afternoon somebody wants a headline.
//!
//! The rules that are enforced rather than remembered: the median of at least five runs with the
//! interquartile range, in [`measure`]; peak resident memory as a value that is either a number or
//! a reason, in [`memory`]; cold separated from hot, load time and on-disk size next to every
//! runtime result, and the whole suite including the losses, in [`report`]. A result that breaks
//! one of them still prints. It just prints with the reason it may not be published underneath it,
//! which today is never an empty list, because no machine this project owns is a `c6a.4xlarge`.
//!
//! The recorded baselines below were computed before there was anything to flatter, which is the
//! only time a baseline is worth anything.
//!
//! [rudb repository]: https://github.com/tamnd/rudb

#![forbid(unsafe_code)]

pub mod answer;
pub mod data;
pub mod engine;
pub mod fleet;
pub mod kernels;
pub mod ledger;
pub mod machine;
pub mod markdown;
pub mod measure;
pub mod memory;
pub mod regress;
pub mod report;
pub mod saved;
pub mod suite;

pub use fleet::{FLEET, Machine, REPORTING_MACHINE, Role};

/// A system on the board, and what it measured on ClickBench at `c6a.4xlarge`.
///
/// These are recomputed from the official result files rather than taken from anybody's slide, on
/// 10 September 2026. The hot total is the sum over the 43 queries of the best of three runs,
/// which is the board's own convention, so it is comparable to the published board and is labelled
/// ClickBench-convention wherever it is quoted. See `spec/03-baselines.md`.
#[derive(Debug, Clone, Copy)]
pub struct Baseline {
    /// The system, as the board names it.
    pub system: &'static str,
    /// Sum of the best hot run over all 43 queries, in seconds.
    pub hot_total_seconds: f64,
    /// On-disk size of `hits` in that system's own format, in gigabytes. `None` where the board
    /// does not report one.
    pub disk_gb: Option<f64>,
}

/// The systems the engine is measured against, and where they stand today.
///
/// DuckDB is the primary comparison because that is what the compatibility claim is against.
/// ClickHouse is here because any claim of being the fastest that does not beat it is not a claim
/// of being the fastest. Umbra is here because it is the current leader and because the entire
/// scenario arithmetic in `spec/02-the-goal.md` is stated relative to it. DataFusion and Polars are
/// here because they are the Rust ecosystem's answer and being ahead of them is a floor rather
/// than an achievement.
pub const CLICKBENCH_C6A_4XLARGE: &[Baseline] = &[
    Baseline { system: "Umbra", hot_total_seconds: 8.10, disk_gb: Some(8.30) },
    Baseline { system: "ClickHouse", hot_total_seconds: 18.07, disk_gb: Some(9.42) },
    Baseline { system: "DuckDB", hot_total_seconds: 26.25, disk_gb: Some(20.46) },
    Baseline { system: "Polars", hot_total_seconds: 45.35, disk_gb: None },
    Baseline { system: "DataFusion", hot_total_seconds: 45.57, disk_gb: Some(14.78) },
    Baseline { system: "Velox", hot_total_seconds: 81.51, disk_gb: None },
];

/// What the engine has to reach on a suite for a claim to hold.
///
/// Stated as a ratio against DuckDB rather than as an absolute time, because an absolute time is a
/// fact about a machine and a ratio is a fact about the engine. The absolute time is still
/// reported next to it, per rule seven: never compare across machines.
#[must_use]
pub fn target_seconds(duckdb_seconds: f64, factor: f64) -> f64 {
    duckdb_seconds / factor
}

#[cfg(test)]
mod tests {
    use super::{CLICKBENCH_C6A_4XLARGE, target_seconds};

    #[test]
    fn ten_times_duckdb_is_past_the_current_leader() {
        let duckdb = CLICKBENCH_C6A_4XLARGE.iter().find(|b| b.system == "DuckDB").unwrap();
        let umbra = CLICKBENCH_C6A_4XLARGE.iter().find(|b| b.system == "Umbra").unwrap();
        let target = target_seconds(duckdb.hot_total_seconds, 10.0);
        // 2.63 seconds, which is 3.1x past the fastest CPU engine anyone has published on this
        // machine. This test exists so that the uncomfortable number stays in front of whoever is
        // reading the code, per spec/02-the-goal.md.
        assert!(target < umbra.hot_total_seconds / 3.0);
    }

    #[test]
    fn the_board_is_ordered_fastest_first() {
        assert!(
            CLICKBENCH_C6A_4XLARGE
                .windows(2)
                .all(|w| w[0].hot_total_seconds < w[1].hot_total_seconds)
        );
    }
}
