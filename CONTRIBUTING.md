# Contributing

This is a harness, not a product. What matters here is that a number it produces can be trusted and reproduced by somebody who does not trust us.

Read [CONTRIBUTING.md in tamnd/rudb](https://github.com/tamnd/rudb/blob/main/CONTRIBUTING.md) for the house rules on Rust style, prose and commits. They apply here unchanged. The prose rules are checked by `cargo test`, which is why they are a test and not a convention.

## What a change has to come with

**A change to how something is timed comes with the reasoning.** The difference between timing a query and timing a query plus its result materialization is a factor that shows up in a table nobody can later explain. If a measurement boundary moves, say where it moved from and to, and say which published numbers are no longer comparable.

**A change that could make us look better comes with more scrutiny than one that could make us look worse.** That asymmetry is deliberate. Nobody accidentally writes a bug that understates their own engine.

**A new suite comes with the loader, the schema and the exact command.** A suite somebody else cannot run is a suite that produces numbers nobody can check.

**Never remove a losing query from a table.** If a query is excluded there is a reason, the reason is written down next to the table, and the count of exclusions is published. A benchmark table with the regressions removed is not a benchmark table.

**A comparison against a system we do not run ourselves is labelled as a published number** with its date and its source, and never mixed into a table of numbers we measured.

## Running it

```
cargo build --release
cargo test
cargo run -- baselines
```

The rest of the commands need the engines and the datasets and neither is wired up yet. That arrives with M1.

## License

Apache-2.0, and a contribution is offered under the same terms. There is no separate agreement to sign.
