//! The index gate of W4 in `17-milestones.md`: building the key of a load of 10 million rows adds
//! at most a tenth to the same load without it.
//!
//! Each run loads `records` rows of `(id BIGINT, v BIGINT)` into a fresh table twice, once without
//! a key and once with `id` as its primary key, and drops both. The two loads take turns, so noise
//! from the machine lands on both alike, and the gate is judged on the medians.

use std::time::Instant;

use crate::backend::{Backend, Failed};

/// What the key may add to the load, as a share of the load without it.
const GATE: f64 = 0.10;

/// Runs the gate and prints a line a run and the medians.
pub(crate) fn gate(backend: &dyn Backend, records: u64, runs: usize) -> Result<(), String> {
    let rows = match backend.name() {
        "postgres" => format!("SELECT g, g * 2 FROM generate_series(0, {}) g", records - 1),
        "rudb" | "duckdb" => format!("SELECT range, range * 2 FROM range({records})"),
        other => return Err(format!("--mode index runs on rudb, duckdb or postgres, not {other}")),
    };
    println!(
        "index 1 backend={} version={} records={records} runs={runs}",
        backend.name(),
        backend.version().replace(' ', "_"),
    );
    let mut session = backend.connect()?;
    let mut load = |run: usize, key: &str| -> Result<f64, String> {
        let table = format!("gate_{}_{run}", if key.is_empty() { "plain" } else { "key" });
        let failed = |what: &'static str| {
            let table = table.clone();
            move |f: Failed| format!("{what} of {table}: {f}")
        };
        session.batch(&format!("DROP TABLE IF EXISTS {table}")).map_err(failed("the drop"))?;
        session
            .batch(&format!("CREATE TABLE {table} (id BIGINT{key}, v BIGINT)"))
            .map_err(failed("the create"))?;
        let started = Instant::now();
        session.batch(&format!("INSERT INTO {table} {rows}")).map_err(failed("the load"))?;
        let took = started.elapsed().as_secs_f64();
        session.batch(&format!("DROP TABLE {table}")).map_err(failed("the drop"))?;
        Ok(took)
    };
    let (mut plain, mut keyed) = (Vec::new(), Vec::new());
    for run in 0..runs {
        // The first load of a run alternates, so neither always meets the file just grown.
        let order = if run % 2 == 0 { [false, true] } else { [true, false] };
        for key in order {
            let took = load(run, if key { " PRIMARY KEY" } else { "" })?;
            if key { keyed.push(took) } else { plain.push(took) }
        }
        let (p, k) = (plain[run], keyed[run]);
        println!("run {run} plain_s={p:.3} key_s={k:.3} added={:.1}%", (k / p - 1.0) * 100.0);
    }
    let (p, k) = (median(&mut plain), median(&mut keyed));
    let added = k / p - 1.0;
    println!(
        "median plain_s={p:.3} key_s={k:.3} added={:.1}% gate={}",
        added * 100.0,
        if added <= GATE { "pass" } else { "fail" }
    );
    println!("end");
    Ok(())
}

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(f64::total_cmp);
    match values.len() {
        0 => f64::NAN,
        n if n % 2 == 1 => values[n / 2],
        n => (values[n / 2 - 1] + values[n / 2]) / 2.0,
    }
}

#[cfg(test)]
mod tests {
    use super::median;

    #[test]
    fn the_median_is_the_middle_or_the_mean_of_the_two_there() {
        assert_eq!(median(&mut [3.0, 1.0, 2.0]), 2.0);
        assert_eq!(median(&mut [4.0, 1.0, 3.0, 2.0]), 2.5);
        assert!(median(&mut []).is_nan());
    }
}
