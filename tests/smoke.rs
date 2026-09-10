//! The measurement path end to end, against a real DuckDB.
//!
//! Everything else in this repository is unit tested against timings that were typed rather than
//! measured, which is right for testing statistics and useless for testing whether the apparatus
//! works. This is the one that spawns a real engine, loads real data, times real runs and produces
//! the real table.
//!
//! It uses a hundred thousand rows rather than the smoke suite's ten million, because the thing
//! under test is the harness and the data size is the one parameter that costs seconds without
//! testing anything extra. It skips rather than fails when there is no DuckDB, for the reason the
//! compat harness does: a suite nobody can build without the thing it drives is a suite nobody
//! works on. CI installs one.

use rudb_bench::engine::{Duckdb, Engine};
use rudb_bench::report::{publishable, run, table};
use rudb_bench::suite::{Query, find};

const SMALL: &[&str] =
    &["CREATE TABLE t AS SELECT i AS id, i % 64 AS k, (i % 97) / 7.0 AS v FROM range(100000) s(i)"];

const QUERIES: &[Query] = &[
    Query { name: "q1", sql: "SELECT count(*) FROM t", shape: "count" },
    Query { name: "q2", sql: "SELECT k, sum(v) FROM t GROUP BY k", shape: "group by" },
];

/// A scratch directory unique to the test that asked for one.
fn scratch(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("rudb-bench-{name}-{}", std::process::id()))
}

#[test]
fn the_whole_measurement_path_produces_a_table_a_person_can_read() {
    let at = scratch("path");
    let Ok(mut duckdb) = Duckdb::discover(&at) else {
        eprintln!("skipping, no DuckDB on this machine");
        return;
    };
    let suite = find("smoke").unwrap();
    let result = run(&mut duckdb, suite, QUERIES, SMALL, 5).expect("a real engine should measure");

    assert_eq!(result.queries.len(), 2, "the whole suite, losses included");
    assert!(result.loaded.on_disk > 0, "a loaded database takes space");
    for query in &result.queries {
        assert_eq!(query.runs.hot.runs(), 5, "rule two wants at least five");
        assert!(query.runs.hot.publishable());
        assert!(query.runs.cold.as_nanos() > 0, "the cold run is a separate number");
        assert!(query.runs.hot.headline().as_nanos() > 0);
    }

    let text = table(&result);
    assert!(text.contains("q1") && text.contains("q2"), "{text}");
    assert!(text.contains("load"), "{text}");
    assert!(text.contains(duckdb.version()), "the version goes next to the number\n{text}");

    let _ = std::fs::remove_dir_all(&at);
}

#[test]
fn a_number_measured_here_can_never_be_published() {
    let at = scratch("publish");
    let Ok(mut duckdb) = Duckdb::discover(&at) else {
        eprintln!("skipping, no DuckDB on this machine");
        return;
    };
    let suite = find("smoke").unwrap();
    let result = run(&mut duckdb, suite, QUERIES, SMALL, 5).expect("a real engine should measure");

    // Two reasons at least: no machine here is a c6a.4xlarge, and smoke is comparable to nothing.
    // This assertion is the one that has to fail before anybody publishes anything, which is the
    // whole reason `publishable` returns a list of sentences rather than a bool.
    let reasons = publishable(&result);
    assert!(reasons.len() >= 2, "{reasons:?}");
    assert!(reasons.iter().any(|r| r.contains("c6a.4xlarge")), "{reasons:?}");

    let _ = std::fs::remove_dir_all(&at);
}

#[test]
fn a_query_that_does_not_run_stops_the_suite_rather_than_scoring_zero() {
    let at = scratch("bad");
    let Ok(mut duckdb) = Duckdb::discover(&at) else {
        eprintln!("skipping, no DuckDB on this machine");
        return;
    };
    let suite = find("smoke").unwrap();
    let broken: &[Query] =
        &[Query { name: "q1", sql: "SELECT nope FROM t", shape: "not a column" }];
    let got = run(&mut duckdb, suite, broken, SMALL, 5);
    assert!(got.is_err(), "a failing query is a broken run and not a fast one");

    let _ = std::fs::remove_dir_all(&at);
}
