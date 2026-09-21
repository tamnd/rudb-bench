//! What to run, and what it is pinned to.
//!
//! Reporting rule one says a number carries the exact version of everything that produced it, and
//! the reports already do that: they write down what the binary said about itself on the day. That
//! answers what *was* run and it does not answer what *should* be, which is a different question
//! with a different failure. A rival that shipped two releases since the last measurement makes
//! every ratio in the README quietly wrong, and nothing in a report can notice, because a report is
//! a record of an afternoon.
//!
//! So the versions to run are written down here, in one file, and a scheduled job compares them
//! against what each project has released. When one moves, the job opens a pull request that bumps
//! the line and says which command re-measures it. Nothing is ever timed by that job: no machine a
//! shared CI runner hands out is a machine this project may publish a number from, and pretending
//! otherwise would put a number into the README that rule seven forbids.
//!
//! The file is also the answer to the question somebody arriving at the repository actually asks,
//! which is which DuckDB the comparison is against. That answer belongs in one place rather than in
//! a workflow file, a README paragraph and a shell history that disagree.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// One thing that gets run, and the version it is pinned to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    /// The engine, spelled the way `--engines` spells it, or `rudb` for the engine itself.
    pub name: String,
    /// The exact version to run, as the project spells it.
    pub version: String,
    /// Where the scheduled check asks what the newest version is.
    ///
    /// Three kinds because these five projects ship three different ways, and a pin file that only
    /// knew about GitHub releases would have to mark DataFusion and Polars unchecked for a reason
    /// that is about this file rather than about them. Nothing at all is still allowed, for
    /// something installed by hand: a pin nobody can check automatically is still a pin, and saying
    /// so is better than a check that quietly never looks at it.
    pub source: Option<Source>,
    /// Which release channel a bump follows.
    pub channel: Channel,
    /// Why this version and not another, in one sentence, for the reader rather than the machine.
    pub why: String,
}

/// Where to ask what the newest version is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// A GitHub repository's releases, by `owner/name`.
    GitHub(String),
    /// A crate on crates.io, by name.
    Crate(String),
    /// A package on PyPI, by name.
    PyPi(String),
}

impl Source {
    /// The key this is written under.
    #[must_use]
    pub const fn key(&self) -> &'static str {
        match self {
            Self::GitHub(_) => "github",
            Self::Crate(_) => "crate",
            Self::PyPi(_) => "pypi",
        }
    }

    /// What it names.
    #[must_use]
    pub fn what(&self) -> &str {
        match self {
            Self::GitHub(at) | Self::Crate(at) | Self::PyPi(at) => at,
        }
    }

    /// Where a person goes to see the releases for themselves.
    #[must_use]
    pub fn link(&self) -> String {
        match self {
            Self::GitHub(at) => format!("https://github.com/{at}/releases"),
            Self::Crate(at) => format!("https://crates.io/crates/{at}"),
            Self::PyPi(at) => format!("https://pypi.org/project/{at}/"),
        }
    }
}

/// Which releases a pin follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    /// The newest release the project marks as stable. What almost every pin follows, because a
    /// rival measured at a prerelease is a rival nobody else is running.
    Stable,
    /// The newest release including prereleases. For the DuckDB the compatibility target follows,
    /// which is a development build on purpose.
    Any,
    /// Never bumped by the scheduled check. For anything installed by hand.
    Held,
}

impl Channel {
    /// The word the file uses.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Any => "any",
            Self::Held => "held",
        }
    }

    /// Read the word back.
    fn parse(word: &str) -> Option<Self> {
        match word {
            "stable" => Some(Self::Stable),
            "any" => Some(Self::Any),
            "held" => Some(Self::Held),
            _ => None,
        }
    }
}

/// Where the pins live.
#[must_use]
pub fn path() -> PathBuf {
    std::env::var_os("RUDB_BENCH_PINS").map_or_else(|| PathBuf::from("pins.txt"), PathBuf::from)
}

/// Read the pins, or nothing at all when the file is not there.
///
/// # Errors
///
/// A file that exists and does not parse, with the line number.
pub fn read(at: &Path) -> Result<Vec<Pin>, String> {
    match std::fs::read_to_string(at) {
        Ok(text) => parse(&text).map_err(|e| format!("{}: {e}", at.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(format!("could not read {}: {e}", at.display())),
    }
}

/// The header the file carries.
const HEADER: &str = "\
# What to run, and what it is pinned to.
#
# One block per engine. Read by `rudb-bench pins`, rendered into the README, and bumped by the
# scheduled version check, which opens a pull request when a project releases something newer than
# the line here. That job never times anything: no shared CI runner is a machine this project may
# publish a number from. It says what moved and what command re-measures it, and a person runs that
# command on a machine in `rudb-bench fleet`.
#
# channel stable  follow the newest release the project marks stable
# channel any     follow the newest release, prereleases included
# channel held    never bumped automatically, installed by hand here
#
# Changing a version here does not change any number in this repository. The numbers move when
# somebody re-runs the ladder and commits the board, which is the whole point of keeping the two
# apart: a README that bumped its versions without re-measuring would be attributing this month's
# numbers to next month's DuckDB.
";

/// Write the pins back out.
///
/// # Errors
///
/// Anything the filesystem says.
pub fn write(at: &Path, pins: &[Pin]) -> Result<(), String> {
    std::fs::write(at, render(pins)).map_err(|e| format!("could not write {}: {e}", at.display()))
}

/// The file, as it is written.
#[must_use]
pub fn render(pins: &[Pin]) -> String {
    let mut out = String::from(HEADER);
    for pin in pins {
        let _ = writeln!(out);
        let _ = writeln!(out, "[engine]");
        let _ = writeln!(out, "name      {}", pin.name);
        let _ = writeln!(out, "version   {}", pin.version);
        if let Some(source) = &pin.source {
            let _ = writeln!(out, "{:<10}{}", source.key(), source.what());
        }
        let _ = writeln!(out, "channel   {}", pin.channel.word());
        let _ = writeln!(out, "why       {}", pin.why);
    }
    out
}

/// Read the file.
///
/// # Errors
///
/// A key outside a block, a key nothing has, a channel that is not one of the three, or a block
/// missing its name or its version. Every one of them names the line.
pub fn parse(text: &str) -> Result<Vec<Pin>, String> {
    let mut out: Vec<Pin> = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let at = index + 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[engine]" {
            out.push(Pin {
                name: String::new(),
                version: String::new(),
                source: None,
                channel: Channel::Stable,
                why: String::new(),
            });
            continue;
        }
        let (key, value) = line
            .split_once(char::is_whitespace)
            .ok_or(format!("line {at}: `{line}` is not a key and a value"))?;
        let value = value.trim().to_owned();
        let pin = out.last_mut().ok_or(format!("line {at}: `{key}` before any engine"))?;
        match key {
            "name" => pin.name = value,
            "version" => pin.version = value,
            "github" => pin.source = Some(Source::GitHub(value)),
            "crate" => pin.source = Some(Source::Crate(value)),
            "pypi" => pin.source = Some(Source::PyPi(value)),
            "channel" => {
                pin.channel = Channel::parse(&value)
                    .ok_or(format!("line {at}: `{value}` is not stable, any or held"))?;
            }
            "why" => pin.why = value,
            _ => return Err(format!("line {at}: an engine has no `{key}`")),
        }
    }
    for pin in &out {
        if pin.name.is_empty() || pin.version.is_empty() {
            return Err(format!("an engine is missing its name or its version: {pin:?}"));
        }
    }
    Ok(out)
}

/// Move one pin to a new version, leaving every other line in the file alone.
///
/// The scheduled check edits through this rather than through a regular expression in a workflow,
/// so that a malformed bump fails here with a sentence rather than silently rewriting a version
/// into the middle of a `why`.
///
/// # Errors
///
/// A name that is not pinned, which is what a typo in the workflow looks like.
pub fn set(pins: &mut [Pin], name: &str, version: &str) -> Result<bool, String> {
    // The list of names is built before the search, because the error wants it and the search holds
    // the whole slice for as long as its answer lives.
    let known = pins.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(", ");
    let pin = pins
        .iter_mut()
        .find(|p| p.name == name)
        .ok_or_else(|| format!("{name} is not pinned here, the ones that are: {known}"))?;
    if pin.version == version {
        return Ok(false);
    }
    pin.version = version.to_owned();
    Ok(true)
}

/// The pins as a markdown table, for the block in the README.
#[must_use]
pub fn table(pins: &[Pin]) -> String {
    if pins.is_empty() {
        return "Nothing is pinned yet.\n\n".to_owned();
    }
    let mut out = String::new();
    let _ = writeln!(out, "| what | version to run | follows | why |");
    let _ = writeln!(out, "| --- | --- | --- | --- |");
    for pin in pins {
        let follows = match (&pin.source, pin.channel) {
            (Some(source), Channel::Held) => {
                format!("[{}]({}), held here", source.what(), source.link())
            }
            (None, Channel::Held) => "held here, installed by hand".to_owned(),
            (Some(source), channel) => {
                format!("[{}]({}), {}", source.what(), source.link(), channel.word())
            }
            (None, channel) => channel.word().to_owned(),
        };
        let _ = writeln!(out, "| {} | `{}` | {follows} | {} |", pin.name, pin.version, pin.why);
    }
    let _ = writeln!(out);
    out
}

/// What each pin says versus what each rung on the board actually ran.
///
/// The one check worth running without a machine: it catches the README that claims DuckDB v1.5.5
/// while every number under it came from v1.4.1, which is the failure that a pin file makes
/// possible and that nothing else in the repository would notice.
///
/// Substring rather than equality, because the pin is a release tag and the engine says something
/// longer with a codename and a hash in it. A tag that is not in what the binary said is a mismatch
/// worth a sentence; the other direction is normal.
#[must_use]
pub fn measured(pins: &[Pin], rungs: &[crate::board::Rung]) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let named: Vec<&str> = pins.iter().map(|p| p.name.as_str()).collect();
    for pin in pins {
        for rung in rungs {
            for column in &rung.columns {
                if column.engine != pin.name
                    && !(matches(&pin.name, &column.engine)
                        && !named.contains(&column.engine.as_str()))
                {
                    continue;
                }
                if !column.version.contains(pin.version.trim_start_matches('v'))
                    && !out.entry(pin.name.clone()).or_default().contains(&column.version)
                {
                    out.entry(pin.name.clone()).or_default().push(column.version.clone());
                }
            }
        }
    }
    out
}

/// Whether a pin covers an engine that spells itself differently.
///
/// `clickhouse` is one release and two engines here, `clickhouse-local` and `clickhouse-server`,
/// which is a property of ClickHouse rather than a naming accident worth two pins. The rule only
/// applies to an engine with no pin of its own: `duckdb-pinned` is spelled like a variant of
/// `duckdb` and is a separate build at a separate version, which is the whole reason it exists.
fn matches(pin: &str, engine: &str) -> bool {
    engine.strip_prefix(pin).is_some_and(|rest| rest.starts_with('-'))
}

#[cfg(test)]
mod tests {
    use super::{Channel, Pin, measured, parse, render, set, table};
    use crate::board::{Column, Rung};
    use std::time::Duration;

    fn pins() -> Vec<Pin> {
        vec![
            Pin {
                name: "duckdb".to_owned(),
                version: "v1.5.5".to_owned(),
                source: Some(super::Source::GitHub("duckdb/duckdb".to_owned())),
                channel: Channel::Stable,
                why: "The newest stable release.".to_owned(),
            },
            Pin {
                name: "clickhouse".to_owned(),
                version: "26.9.1.1562".to_owned(),
                source: None,
                channel: Channel::Held,
                why: "Installed from the distribution's package.".to_owned(),
            },
        ]
    }

    fn rung(columns: &[(&str, &str)]) -> Rung {
        Rung {
            suite: "clickbench".to_owned(),
            machine: "gamingpc-wsl".to_owned(),
            size: "1k".to_owned(),
            rows: Some(1000),
            shared: 1,
            commit: "abc1234".to_owned(),
            recorded: "2026-09-21".to_owned(),
            columns: columns
                .iter()
                .map(|(engine, version)| Column {
                    engine: (*engine).to_owned(),
                    version: (*version).to_owned(),
                    hot: Duration::from_millis(1),
                    own: Duration::from_millis(1),
                    reported: true,
                    wall: Duration::from_millis(1),
                    cold: Duration::from_millis(1),
                    cpu: None,
                    peak: None,
                    load: Duration::ZERO,
                    disk: 0,
                    converted: false,
                    answered: 1,
                    queries: 1,
                })
                .collect(),
        }
    }

    #[test]
    fn what_is_written_is_what_is_read_back() {
        assert_eq!(parse(&render(&pins())).expect("parses"), pins());
    }

    #[test]
    fn a_bump_moves_one_line_and_says_when_it_moved_nothing() {
        let mut pins = pins();
        assert!(set(&mut pins, "duckdb", "v1.6.0").expect("set"));
        assert_eq!(pins[0].version, "v1.6.0");
        assert!(
            !set(&mut pins, "duckdb", "v1.6.0").expect("set"),
            "a no-op bump reported a change"
        );
        assert_eq!(pins[1], self::pins()[1], "another pin moved");
    }

    #[test]
    fn a_name_nobody_pinned_is_refused_with_the_ones_there_are() {
        let e = set(&mut pins(), "sqlite", "3.50").expect_err("refused");
        assert!(e.contains("duckdb, clickhouse"), "{e}");
    }

    #[test]
    fn a_channel_that_is_not_one_of_the_three_names_its_line() {
        let e = parse("[engine]\nname duckdb\nversion v1\nchannel someday\n").expect_err("refused");
        assert!(e.contains("line 4"), "{e}");
    }

    /// `duckdb-pinned` reads like a variant of `duckdb` and is a different build at a different
    /// version. A prefix rule that claimed it would report the deliberate prerelease as a stale pin
    /// on every run.
    #[test]
    fn an_engine_with_a_pin_of_its_own_is_not_swept_up_by_a_prefix() {
        let mut pins = pins();
        pins.push(Pin {
            name: "duckdb-pinned".to_owned(),
            version: "v2.0.0-dev1".to_owned(),
            source: None,
            channel: Channel::Any,
            why: "The compatibility target.".to_owned(),
        });
        let rung = rung(&[
            ("duckdb", "v1.5.5 (Variegata) d8cdaa33fd"),
            ("duckdb-pinned", "v2.0.0-dev1 (Development Version) cc7e7bac7f"),
        ]);
        assert!(measured(&pins, &[rung]).is_empty(), "a pin matched another engine's column");
    }

    #[test]
    fn the_table_links_what_can_be_checked_and_says_so_when_it_cannot() {
        let out = table(&pins());
        assert!(out.contains("[duckdb/duckdb](https://github.com/duckdb/duckdb/releases), stable"));
        assert!(
            out.contains("| clickhouse | `26.9.1.1562` | held here, installed by hand |"),
            "{out}"
        );
    }

    /// The failure a pin file makes possible: a README that names one version over numbers that
    /// came from another.
    #[test]
    fn a_pin_that_is_not_what_ran_is_reported_and_a_pin_that_is_is_not() {
        let rung =
            rung(&[("duckdb", "v1.4.1 (Andium) 1a2b3c"), ("clickhouse-local", "26.9.1.1562")]);
        let found = measured(&pins(), &[rung]);
        assert_eq!(found.get("duckdb").map(Vec::len), Some(1), "the stale duckdb was not caught");
        assert!(!found.contains_key("clickhouse"), "clickhouse-local did not match its pin");
    }
}
