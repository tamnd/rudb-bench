//! Write a suite's queries out as one file, one query a line under a `-- qNN` comment.
//!
//! `scripts/tpch-instructions-ab.py` and the other things that measure a single engine outside the
//! harness need the query text and have no way to read a Rust constant. Rather than a second copy of
//! it checked in beside this, they get a file written from the constant, so there is still one text
//! and the file is a rendering of it.
//!
//! The dialect is the one an engine was given, because a query a suite rewrote for an engine is the
//! query that engine was measured on. `rudb` has no rewrites in either suite, so its text is the
//! upstream text.
//!
//! Usage: `cargo run --example export_queries -- tpch rudb /tmp/tpch.sql`
use rudb_bench::suite::queries;

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1);
    let suite = args.next().expect("a suite name, tpch or clickbench");
    let engine = args.next().expect("an engine name, rudb or duckdb");
    let into = args.next().expect("a file to write");
    let listed = queries(&suite).unwrap_or_else(|| panic!("no such suite: {suite}"));

    let mut out = format!(
        "-- The {suite} queries as the suite in tamnd/rudb-bench runs them against {engine}.\n\
         --\n\
         -- Written by `cargo run --example export_queries`. The text lives in `src/suite.rs` and\n\
         -- this is a rendering of it, so edit that and write this again rather than editing here.\n\
         --\n\
         -- The newlines are gone and the indentation with them, so a query is one line and a\n\
         -- reader can tell where one ends without counting semicolons.\n"
    );
    for query in listed {
        let Some(sql) = query.sql_for(&engine) else { continue };
        out.push_str(&format!("\n-- {}\n{sql};\n", query.name));
    }
    std::fs::write(into, out)
}
