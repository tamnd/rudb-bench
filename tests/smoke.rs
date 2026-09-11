//! The measurement path end to end, against whatever engines this machine has.
//!
//! Everything else in this repository is unit tested against timings that were typed rather than
//! measured, which is right for testing statistics and useless for testing whether the apparatus
//! works. This is the one that spawns real engines, loads real data, times real runs and produces
//! the real table.
//!
//! It uses a hundred thousand rows rather than the smoke suite's ten million, because the thing
//! under test is the harness and the data size is the one parameter that costs seconds without
//! testing anything extra. It skips rather than fails when there is no DuckDB, for the reason the
//! compat harness does: a suite nobody can build without the thing it drives is a suite nobody
//! works on. CI installs one.
//!
//! The engines other than DuckDB are not installed everywhere, so the tests that use them take
//! whatever is there and say what they took. A test that demanded all five would be a test that is
//! skipped on every laptop and therefore never run.

use rudb_bench::data::Table;
use rudb_bench::engine::{ClickhouseLocal, Datafusion, Duckdb, Engine, Polars, Rudb};
use rudb_bench::report::{compare, comparison, publishable, run, table};
use rudb_bench::suite::{Query, find};

const QUERIES: &[Query] = &[
    Query { name: "q1", sql: "SELECT count(*) FROM t", shape: "count", dialects: &[] },
    Query {
        name: "q2",
        sql: "SELECT k, sum(v) AS total FROM t GROUP BY k",
        shape: "group by",
        dialects: &[],
    },
];

/// A scratch directory unique to the test that asked for one.
fn scratch(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("rudb-bench-{name}-{}", std::process::id()))
}

/// A hundred thousand rows in a Parquet file, which is what every engine here is handed.
///
/// Written by DuckDB, which is also one of the engines being compared. That is worth saying and it
/// is not worth avoiding: the output is a Parquet file with no DuckDB left in it.
fn small(duckdb: &Duckdb, at: &std::path::Path) -> Vec<Table> {
    std::fs::create_dir_all(at).unwrap();
    let path = at.join("t.parquet");
    duckdb
        .plain(&[&format!(
            "COPY (SELECT i AS id, i % 64 AS k, (i % 97) / 7.0 AS v FROM range(100000) s(i)) TO \
             '{}' (FORMAT parquet)",
            path.display()
        )])
        .expect("writing a parquet should work");
    let bytes = std::fs::metadata(&path).unwrap().len();
    vec![Table { name: "t".to_owned(), path, bytes }]
}

#[test]
fn the_whole_measurement_path_produces_a_table_a_person_can_read() {
    let at = scratch("path");
    let Ok(mut duckdb) = Duckdb::discover(&at, find("smoke").unwrap()) else {
        eprintln!("skipping, no DuckDB on this machine");
        return;
    };
    let tables = small(&duckdb, &at);
    let suite = find("smoke").unwrap();
    let result =
        run(&mut duckdb, suite, QUERIES, &tables, 5).expect("a real engine should measure");

    assert_eq!(result.queries.len(), 2, "the whole suite, losses included");
    assert!(result.loaded.on_disk > 0, "a loaded database takes space");
    for query in &result.queries {
        assert_eq!(query.runs.hot.runs(), 5, "rule two wants at least five");
        assert!(query.runs.hot.publishable());
        assert!(query.runs.cold.as_nanos() > 0, "the cold run is a separate number");
        assert!(query.runs.hot.headline().as_nanos() > 0);
        assert!(!query.answer.is_empty(), "{} answered nothing at all", query.name);
    }
    assert!(result.queries[0].answer.contains("100000"), "{}", result.queries[0].answer);

    let text = table(&result);
    assert!(text.contains("q1") && text.contains("q2"), "{text}");
    assert!(text.contains("load"), "{text}");
    assert!(text.contains(duckdb.version()), "the version goes next to the number\n{text}");

    let _ = std::fs::remove_dir_all(&at);
}

#[test]
fn a_number_measured_here_can_never_be_published() {
    let at = scratch("publish");
    let Ok(mut duckdb) = Duckdb::discover(&at, find("smoke").unwrap()) else {
        eprintln!("skipping, no DuckDB on this machine");
        return;
    };
    let tables = small(&duckdb, &at);
    let suite = find("smoke").unwrap();
    let result =
        run(&mut duckdb, suite, QUERIES, &tables, 5).expect("a real engine should measure");

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
    let Ok(mut duckdb) = Duckdb::discover(&at, find("smoke").unwrap()) else {
        eprintln!("skipping, no DuckDB on this machine");
        return;
    };
    let tables = small(&duckdb, &at);
    let suite = find("smoke").unwrap();
    let broken: &[Query] =
        &[Query { name: "q1", sql: "SELECT nope FROM t", shape: "not a column", dialects: &[] }];
    let got = run(&mut duckdb, suite, broken, &tables, 5);
    assert!(got.is_err(), "a failing query is a broken run and not a fast one");

    let _ = std::fs::remove_dir_all(&at);
}

#[test]
fn every_engine_on_this_machine_gets_the_same_file_and_answers_the_same_thing() {
    let at = scratch("compare");
    let Ok(duckdb) = Duckdb::discover(&at, find("smoke").unwrap()) else {
        eprintln!("skipping, no DuckDB on this machine");
        return;
    };
    let tables = small(&duckdb, &at);

    let mut engines: Vec<Box<dyn Engine>> = vec![Box::new(duckdb)];
    let smoke = find("smoke").expect("smoke is a suite");
    if let Ok(engine) = ClickhouseLocal::discover(&at, smoke) {
        engines.push(Box::new(engine));
    }
    if let Ok(engine) = Datafusion::discover(&at, smoke) {
        engines.push(Box::new(engine));
    }
    if let Ok(engine) = Polars::discover(&at, smoke) {
        engines.push(Box::new(engine));
    }
    engines.push(Box::new(Rudb::discover(&at, smoke)));
    eprintln!("comparing {} engines", engines.len() - 1);

    let suite = find("smoke").unwrap();
    // Three hot runs rather than five, because this is a test of the comparison and not a
    // measurement, and the publication rules already refuse anything measured here.
    let compared = compare(&mut engines, suite, QUERIES, &tables, 3);

    assert!(!compared.results.is_empty(), "DuckDB at least should have produced numbers");
    assert_eq!(compared.results[0].engine, "duckdb", "the reference column is DuckDB");
    assert!(
        compared.disagreements().is_empty(),
        "engines that disagree are not a comparison: {:?}",
        compared.disagreements()
    );

    // rudb abstains rather than being absent, which is the property that has to survive until the
    // day it stops abstaining.
    assert!(compared.skipped.iter().any(|s| s.engine == "rudb"), "{:?}", compared.skipped);

    let text = comparison(&compared);
    assert!(text.contains("vs duckdb"), "{text}");
    assert!(text.contains("q1") && text.contains("q2"), "{text}");
    assert!(text.contains("rudb"), "{text}");

    let _ = std::fs::remove_dir_all(&at);
}
