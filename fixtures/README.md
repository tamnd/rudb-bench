# Fixtures

Tables with the right schema and no rows, one Parquet file per table, committed.

They exist for one caller, `rudb-bench plans`, which asks rudb for the plan of every query in a suite and compares it to `baselines/plans-<suite>.txt`. A plan needs a schema to bind against and nothing else, and a zero row Parquet file carries its whole schema in the footer, so the entire TPC-H schema and the smoke suite together are 36 KB. That is what makes the plan gate something a GitHub runner can do on every commit rather than something that needs the real corpus and therefore never runs.

Nothing here is data and nothing here is measured against. A number produced by running a query over one of these files would be a measurement of an empty file, and no command in this harness will do it: `prepare` in `src/data.rs` does not look here, and `plans` never calls `run`.

## What is committed, and where it came from

`tpch/` is the eight TPC-H tables, built with DuckDB's own generator at scale factor zero, which emits the schema and no rows:

```sql
INSTALL tpch; LOAD tpch;
CALL dbgen(sf=0);
COPY (SELECT * FROM lineitem) TO 'tpch/lineitem.parquet' (FORMAT parquet, COMPRESSION zstd);
-- and the same for orders, customer, part, partsupp, supplier, nation, region
```

`smoke/` is the one table the smoke suite generates, which is a `SELECT` over `range` in `SMOKE_ROWS` in `src/data.rs`, run over `range(0)` instead of `range(10000000)`:

```sql
COPY (
  SELECT i AS id, (i * 2654435761) % 1000 AS k, (i % 97) / 7.0 AS v,
         'tag-' || ((i * 48271) % 64)::VARCHAR AS tag
  FROM range(0) t(i)
) TO 'smoke/smoke.parquet' (FORMAT parquet, COMPRESSION zstd);
```

Both were made with the pinned DuckDB, which is the same binary the comparison uses as its reference column. That matters more than it looks: the schema a fixture carries has to be the schema the suite's queries were written against, and taking it from the thing that defines the suite is how it stays that way.

## Why ClickBench is not here

The ClickBench suite has no fixture, so `rudb-bench plans` does not check it, and that is a gap rather than an oversight.

`hits` is one wide table and a fixture for it would be thirty lines of column definitions. The schema this repository has written down, in `DUCKDB_HITS` and in the converted schema under `reports/`, is the schema after the suite's own conversion: four of the columns are stored in the raw file as integers holding seconds and days, and the recorded schema has them as `TIMESTAMP` and `DATE`. rudb reads the raw file and applies that conversion itself in a view, so a fixture built from the recorded schema would be a fixture of a table that does not exist on disk anywhere, and every plan captured against it would be a plan of a query nobody runs.

The honest fix is to take the schema out of a real `hits.parquet` footer, which is a one line command on either of the two machines in the fleet that has the file. Until somebody does that, the baselines cover TPC-H and smoke, and this paragraph is here so the gap is a decision somebody can find rather than a suite that silently is not checked.

## Adding one

Put `fixtures/<suite>/<table>.parquet` there for every table the suite names in `src/suite.rs`, then `cargo run -- plans --suite <suite> --record`. Nothing in CI needs editing: the check runs over every suite that has a directory here.

Take the schema from the real file rather than from anything written down about it, for the reason the ClickBench paragraph gives. `DESCRIBE SELECT * FROM read_parquet('<the real file>')` says what it actually is, and `COPY (SELECT * FROM read_parquet('<the real file>') LIMIT 0) TO '<fixture>' (FORMAT parquet, COMPRESSION zstd)` makes the fixture out of it without anybody retyping a column list.
