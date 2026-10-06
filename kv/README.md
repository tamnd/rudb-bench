# rudb-bench-kv

The driver for workloads that run for a duration, like YCSB. The harness starts it as a child process and reads back what it prints. It has its own package so the harness keeps zero dependencies and no `unsafe`.

It opens the database once and gives each client thread its own connection with its statements prepared. Then it runs a warmup and a window, and prints per-second and whole-window histograms, the CPU and memory the window cost, and whether every value read back was one the workload wrote. A run that does not print `end` failed.

```
cargo build --release
./target/release/rudb-bench-kv --backend sqlite --target /tmp/y.db --load --records 100000 --clients 4 --mix read=50,update=50 --level full --window 60s
```

Backends:

- `null` does everything but the call, which gives the driver's own ceiling. With `--service` and `--stall` it becomes a fake engine with a known stall, so you can check that the driver sees a stall that was put there on purpose.
- `sqlite`, `duckdb` and `postgres` load their libraries at run time from `RUDB_BENCH_LIBSQLITE`, `RUDB_BENCH_LIBDUCKDB` and `RUDB_BENCH_LIBPQ`. DuckDB refuses to run unless the library is the pinned v2.0 build, and `--unpinned` lets it run anyway but marks the output `pinned=no`.
- `rudb` needs `--features rudb`. To build against a checkout instead of crates.io, add `--config 'patch.crates-io.rudb.path="<checkout>/crates/rudb"'`.

`--level full|os|none` picks the durability level. A backend that cannot set the level refuses, and results are only ever compared at the same level.

`--mix` takes `read`, `update`, `insert` and `scan` parts, so workload E is `--mix scan=95,insert=5`. A scan starts at a drawn key and reads 1 to 100 rows in key order, and every row it returns is checked like a read. `--distribution uniform|zipfian|latest` picks how keys are drawn: zipfian is YCSB's scrambled zipfian, and latest makes the newest rows the hot ones, as in workload D. Inserts during a run take the next record number, and reads only draw rows whose inserts have finished.

## TPC-C

`--mode tpcc` runs the TPC-C suite of the spec's `bench/tpc-c` documents, closed loop:

```
./target/release/rudb-bench-kv --mode tpcc --backend sqlite --target /tmp/c.db --load --warehouses 1 --terminals 4 --warmup 10s --window 60s
```

`--load` drops and creates the nine tables, then fills them one table at a time with literal multi-row `INSERT` statements, one transaction for each warehouse or district, spread over the terminals' sessions, and then creates the two indexes. Every table and index prints its own time. The rows come from a seeded generator, so the same `--seed` gives the same database on every engine. A run without `--load` runs on whatever the target already holds.

Each terminal deals the 45/43/4/4/4 deck of New-Order, Payment, Order-Status, Delivery and Stock-Level, and sends each transaction's statements as document 03 writes them. A conflict is retried from `BEGIN` with the same inputs, and the 1% of New-Orders whose last item does not exist are rolled back on purpose and counted apart. NOPM is counted twice, by the driver and by `sum(d_next_o_id)` at the edges of the window, and a run whose counts differ by more than twice the terminals is invalid. The window opens when the first of those reads returns rather than at a set instant, so a late monitor thread does not leave New-Orders that only the driver saw. After the run the consistency conditions of document 05 run through the same backend, and when the run also loaded, so do the counting checks against what the driver saw commit. The `verify` line says whether the run is valid and why not.

The null backend hands back canned rows of the right shape for each statement, which is the driver's own cost and ceiling, and it skips the checks because it keeps nothing. DuckDB does not run TPC-C yet, because its backend reads only VARCHAR results.
