//! Export the harness's DuckDB SQL for the independent measurement audit.
use rudb_bench::suite::{CLICKBENCH, DUCKDB_HITS, Fixup, divergence, loading, unsettled};

fn main() -> std::io::Result<()> {
    let root = std::path::PathBuf::from(std::env::args().nth(1).expect("output directory"));
    std::fs::create_dir_all(&root)?;
    std::fs::write(root.join("schema.sql"), DUCKDB_HITS)?;
    let recipe = loading("clickbench", "duckdb", "hits").unwrap();
    let Fixup::Select(select) = recipe.fixup else { panic!("expected projection") };
    std::fs::write(root.join("projection.sql"), select)?;
    for query in CLICKBENCH {
        std::fs::write(root.join(format!("{}.sql", query.name)), query.sql_for("duckdb").unwrap())?;
        if let Some(why) =
            unsettled("clickbench", query.name).or_else(|| divergence("clickbench", query.name))
        {
            std::fs::write(root.join(format!("{}.caveat", query.name)), why)?;
        }
    }
    Ok(())
}
