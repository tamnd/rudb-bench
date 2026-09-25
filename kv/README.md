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
