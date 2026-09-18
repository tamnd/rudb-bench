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

use rudb_bench::data::{Dataset, Table};
use rudb_bench::engine::{ClickhouseLocal, Datafusion, Duckdb, Engine, Outcome, Polars, Rudb};
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
fn small(duckdb: &Duckdb, at: &std::path::Path) -> Dataset {
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
    Dataset {
        tables: vec![Table { name: "t".to_owned(), path, bytes }],
        sample: None,
        rows: Some(100_000),
        scale: None,
        rows_exact: true,
        manifest: None,
    }
}

#[test]
fn the_whole_measurement_path_produces_a_table_a_person_can_read() {
    let at = scratch("path");
    let Ok(mut duckdb) = Duckdb::discover(&at, find("smoke").unwrap()) else {
        eprintln!("skipping, no DuckDB on this machine");
        return;
    };
    let dataset = small(&duckdb, &at);
    let suite = find("smoke").unwrap();
    let result =
        run(&mut duckdb, suite, QUERIES, &dataset, 5, None).expect("a real engine should measure");

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
    let dataset = small(&duckdb, &at);
    let suite = find("smoke").unwrap();
    let result =
        run(&mut duckdb, suite, QUERIES, &dataset, 5, None).expect("a real engine should measure");

    // Two reasons at least: no machine here is a c6a.4xlarge, and smoke is comparable to nothing.
    // This assertion is the one that has to fail before anybody publishes anything, which is the
    // whole reason `publishable` returns a list of sentences rather than a bool.
    let reasons = publishable(&result);
    assert!(reasons.len() >= 2, "{reasons:?}");
    assert!(reasons.iter().any(|r| r.contains("c6a.4xlarge")), "{reasons:?}");

    let _ = std::fs::remove_dir_all(&at);
}

/// A query the engine cannot answer is one row of the table and not the end of the column.
///
/// It used to stop the whole run, on the reasoning that a query which is not valid SQL is a fact
/// about the run rather than about the query. That reasoning holds for an engine that cannot start
/// and does not hold for one that started, loaded the data and then objected to query eleven. An
/// SF1 run here came back with no rudb column at all because of a HAVING the engine could not bind
/// yet, and the twenty one queries it did answer went in the bin with it.
///
/// What it must never become is a zero that reads as a fast query, which is what the rest of this
/// checks.
#[test]
fn a_query_that_does_not_run_is_a_row_saying_so_rather_than_a_fast_one() {
    let at = scratch("bad");
    let Ok(mut duckdb) = Duckdb::discover(&at, find("smoke").unwrap()) else {
        eprintln!("skipping, no DuckDB on this machine");
        return;
    };
    let dataset = small(&duckdb, &at);
    let suite = find("smoke").unwrap();
    let broken: &[Query] =
        &[Query { name: "q1", sql: "SELECT nope FROM t", shape: "not a column", dialects: &[] }];
    let result = run(&mut duckdb, suite, broken, &dataset, 5, None)
        .expect("one query the engine cannot answer is a row and not the end of the run");

    assert_eq!(result.queries.len(), 1);
    let query = &result.queries[0];
    let Outcome::Failed { message } = &query.outcome else {
        panic!("{:?} is not a failure", query.outcome);
    };
    assert!(message.contains("nope"), "{message}");
    assert_eq!(query.outcome.cell(), "failed");

    // No number anywhere on it, and nothing downstream may read one.
    assert!(!query.outcome.measured());
    assert!(!query.runs.hot.publishable());
    assert!(!query.cold.peak.measured());
    assert_eq!(result.unmeasured(), 1);

    // And the column says so in words, because a total of zero over one query is the most
    // flattering number this harness could print.
    let reasons = publishable(&result);
    assert!(reasons.iter().any(|r| r.contains("q1 failed")), "{reasons:?}");
    assert!(reasons.iter().any(|r| r.contains("floor")), "{reasons:?}");

    let _ = std::fs::remove_dir_all(&at);
}

#[test]
fn every_engine_on_this_machine_gets_the_same_file_and_answers_the_same_thing() {
    let at = scratch("compare");
    let Ok(duckdb) = Duckdb::discover(&at, find("smoke").unwrap()) else {
        eprintln!("skipping, no DuckDB on this machine");
        return;
    };
    let dataset = small(&duckdb, &at);

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
    // One hot run rather than five, because this is a test of the comparison and not a
    // measurement, and the publication rules already refuse anything measured here. Five engines
    // times two queries times a cold run and the hot ones is the longest thing the gate does, and
    // every run after the first tests the same code with a different number in it. The test above
    // is the one that holds rule two, and it asks for five.
    let compared = compare(&mut engines, suite, QUERIES, &dataset, 1, None);

    assert!(!compared.results.is_empty(), "DuckDB at least should have produced numbers");
    assert_eq!(compared.results[0].engine, "duckdb", "the reference column is DuckDB");
    assert!(
        compared.disagreements().is_empty(),
        "engines that disagree are not a comparison: {:?}",
        compared.disagreements()
    );

    // rudb produces numbers on this suite, which is what E0e is for. It is asserted rather than
    // hoped for, because the failure mode is a row quietly going missing and a table that still
    // looks complete. A machine with no rudb built on it is a skip with a reason and not a pass.
    let rudb = compared.results.iter().find(|r| r.engine == "rudb");
    match rudb {
        Some(rudb) => {
            let ran: Vec<&str> = rudb.queries.iter().map(|q| q.name.as_str()).collect();
            assert_eq!(ran, ["q1", "q2"], "both of the queries this file asks for");
            assert!(rudb.queries[0].answer.contains("100000"), "{}", rudb.queries[0].answer);
            assert_eq!(rudb.loaded.took, std::time::Duration::ZERO, "a view is not a load");
            assert_eq!(rudb.loaded.on_disk, dataset.tables[0].bytes, "the source file is the size");
        }
        None => assert!(
            compared.skipped.iter().any(|s| s.engine == "rudb"),
            "rudb is either a row or an abstention with a reason, never absent: {:?}",
            compared.skipped
        ),
    }

    let text = comparison(&compared);
    assert!(text.contains("vs duckdb"), "{text}");
    assert!(text.contains("q1") && text.contains("q2"), "{text}");
    assert!(text.contains("rudb"), "{text}");

    let _ = std::fs::remove_dir_all(&at);
}
