//! The kernel suite: rudb's own loops, measured in rudb's process and recorded here.
//!
//! Every other suite in this harness starts a database and sends it SQL. This one cannot. A kernel
//! is a function over a vector, it takes microseconds, and the only way to time one is to link it
//! and call it, which means the measuring code has to live in the rudb repository against the
//! working tree rather than here against the last published crate. It does: `cargo xtask kernels`
//! prints five tables and `cargo xtask kernels --json` prints the same numbers as records.
//!
//! So what is this file for. Three things that the table on its own does not do.
//!
//! It puts a machine on the numbers. A kernel number is nanoseconds per row and every machine has a
//! different answer, so a number with no machine attached is a number that cannot be compared with
//! anything, including itself next week. Rule seven is the same rule here as everywhere else.
//!
//! It commits them. `baselines/kernels.txt` is a file in a repository with a history, which is what
//! makes a regression something the next change trips over rather than something somebody notices
//! two milestones later. That is the whole reason sub-milestone 2b asks for this and not for a
//! prettier table.
//!
//! And it checks the next run against the last one, at a threshold the measurement can support.
//! There are two kinds of finding. A cell that got [`FACTOR`] slower is the ordinary one, and the
//! threshold is high because these numbers move by a fifth between two runs on a shared machine and
//! anything under that is weather. The other kind has no threshold at all: a cell that used to take
//! a specialized loop and now takes the row at a time path has lost a specialization, which is a
//! factor of fifty rather than a factor of two, and it is worth failing on the fact rather than on
//! the time it happened to produce.
//!
//! ## Rule ten still applies
//!
//! A kernel number is an explanation of a query number. Nothing in this file times a query, so
//! nothing in this file is a result on its own, and the tables the other suites produce are where
//! any number anybody quotes comes from.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::machine;
use crate::regress::{FACTOR, today};

/// One measured cell, which is one row of one of the five tables.
///
/// The same shape the JSON has, because it is the JSON. Flattened rather than a type per table for
/// the reason the producing side gives: five shapes cannot be diffed against each other and one
/// list of records with keys on it can.
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    /// Which of the five tables it came out of.
    pub table: String,
    /// What was measured: a layout name, a string case, a chunk shape, an operation.
    pub case: String,
    /// The second key: a SQL type, a selectivity, a vector size.
    pub detail: String,
    /// The form pair, or the stage of the select against compact comparison.
    pub variant: String,
    /// Percent of the rows that are null.
    pub nulls: usize,
    /// How many rows the time was divided by.
    pub rows: usize,
    /// Nanoseconds per one of those rows.
    pub nanos: f64,
    /// The interquartile range as a fraction of the median.
    pub spread: f64,
    /// Whether the kernel under this cell took the row at a time path, in which case the number is
    /// the oracle's rather than a kernel's and is a different fact entirely.
    pub fell_back: bool,
}

impl Cell {
    /// What names this cell across two runs.
    ///
    /// Everything except the numbers and the fallback flag, since those are what is being compared.
    /// Built as a string rather than compared field by field because it is also what a report line
    /// prints, and two ways of saying which cell this is would eventually disagree.
    #[must_use]
    pub fn key(&self) -> String {
        format!("{} {} {} {} {}%", self.table, self.case, self.detail, self.variant, self.nulls)
    }
}

/// One machine's recorded run of the whole table.
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    /// The machine, by the name the fleet knows it as.
    pub machine: String,
    /// What was measured, which is a rudb version and a commit rather than this harness's version.
    pub rudb: String,
    /// The day, so a record that is a year old says so.
    pub recorded: String,
    /// Every cell, in the order the tables produced them.
    pub cells: Vec<Cell>,
}

/// What happened to one cell between the recorded run and this one.
#[derive(Debug, Clone, PartialEq)]
pub enum Judgement {
    /// It took a specialized loop before and takes the row at a time path now. There is no
    /// threshold on this one: it is a fact rather than a time, it costs a factor of fifty rather
    /// than a factor of two, and it is the single most likely way for a change in this layer to go
    /// wrong without failing a test.
    LostSpecialization,
    /// It took the row at a time path before and takes a loop now, which is a loop somebody wrote.
    GainedSpecialization,
    /// It got past this cell's threshold slower. See [`threshold`].
    Slower(f64),
    /// It got past this cell's threshold faster, which is reported rather than ignored. A cell that
    /// is four times faster than the file says is usually not a cell that got faster, it is a cell
    /// whose recorded number was taken on an unlucky run, and the finding is how that gets noticed
    /// and re-recorded rather than sitting in the file waiting to fail somebody else's gate from the
    /// other side.
    Faster(f64),
    /// It moved by less than that, which on this measurement is not evidence of anything.
    Unchanged(f64),
    /// It is not in the recorded run, so there is nothing to compare it against.
    New,
}

impl Judgement {
    /// Whether this one fails the gate.
    #[must_use]
    pub fn failed(&self) -> bool {
        matches!(self, Self::LostSpecialization | Self::Slower(_))
    }
}

/// One cell's before and after.
#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    /// Which cell, as [`Cell::key`] writes it.
    pub cell: String,
    /// What it was, in nanoseconds a row, and nothing when the cell is new.
    pub was: Option<f64>,
    /// What it is now.
    pub now: f64,
    /// What that amounts to.
    pub judgement: Judgement,
}

/// How far a cell has to move before it is a finding rather than the same number again.
///
/// [`FACTOR`] for a steady cell, and wider for a cell that is not steady. Every recorded cell
/// carries the interquartile range of the measurement that produced it, which is the file's own
/// statement of how much that number moves when nothing changed, and a cell that moves by half on
/// its own cannot have a factor of two claimed about it.
///
/// The incident this exists for: `compaction 2 bigint 1 select build` recorded 341 nanoseconds with
/// an interquartile range of 58 percent, the worst in the table, and the next two runs of the same
/// binary on the same machine gave 86.3 and 86.7. A flat factor of two would have made that cell a
/// failure the first time it read high instead of low, on a change that had nothing to do with it.
#[must_use]
pub fn threshold(spread: f64) -> f64 {
    FACTOR * (1.0 + spread.max(0.0))
}

/// Compare a fresh run against a recorded one.
///
/// Cells in the record that the fresh run does not have are not reported. A table that dropped a
/// column is a change to the measuring code, which shows up as a diff in the rudb repository where
/// somebody made it, and reporting it here as though it were a result would mean every run after a
/// deliberate change printed a wall of findings about cells nobody measured.
#[must_use]
pub fn compare(record: &Record, fresh: &[Cell]) -> Vec<Change> {
    fresh
        .iter()
        .map(|cell| {
            let key = cell.key();
            let Some(was) = record.cells.iter().find(|old| old.key() == key) else {
                return Change { cell: key, was: None, now: cell.nanos, judgement: Judgement::New };
            };
            let ratio = if was.nanos == 0.0 { 1.0 } else { cell.nanos / was.nanos };
            let judgement = if cell.fell_back && !was.fell_back {
                Judgement::LostSpecialization
            } else if was.fell_back && !cell.fell_back {
                Judgement::GainedSpecialization
            } else if ratio >= threshold(was.spread) {
                Judgement::Slower(ratio)
            } else if ratio <= 1.0 / threshold(was.spread) {
                Judgement::Faster(ratio)
            } else {
                Judgement::Unchanged(ratio)
            };
            Change { cell: key, was: Some(was.nanos), now: cell.nanos, judgement }
        })
        .collect()
}

/// The lines a comparison prints, and whether the gate failed.
#[must_use]
pub fn report(changes: &[Change]) -> (String, bool) {
    let mut out = String::new();
    let mut failures = 0;
    let mut findings = 0;
    for change in changes {
        let line = match &change.judgement {
            Judgement::Unchanged(_) => continue,
            Judgement::New => format!("new       {:<52}  {:>8.2}", change.cell, change.now),
            Judgement::GainedSpecialization => format!(
                "gained    {:<52}  {:>8.2} was {:.2}, a loop now covers this pair",
                change.cell,
                change.now,
                change.was.unwrap_or_default()
            ),
            Judgement::LostSpecialization => format!(
                "LOST      {:<52}  {:>8.2} was {:.2}, this pair fell to the row at a time path",
                change.cell,
                change.now,
                change.was.unwrap_or_default()
            ),
            Judgement::Slower(ratio) => format!(
                "SLOWER    {:<52}  {:>8.2} was {:.2}, {ratio:.1}x",
                change.cell,
                change.now,
                change.was.unwrap_or_default()
            ),
            Judgement::Faster(ratio) => format!(
                "faster    {:<52}  {:>8.2} was {:.2}, {:.1}x",
                change.cell,
                change.now,
                change.was.unwrap_or_default(),
                1.0 / ratio
            ),
        };
        findings += 1;
        if change.judgement.failed() {
            failures += 1;
        }
        let _ = writeln!(out, "{line}");
    }

    let checked = changes.len();
    if findings == 0 {
        let _ = writeln!(
            out,
            "{checked} cells, none of them past {FACTOR:.0}x and none of them lost a loop"
        );
    } else {
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "{checked} cells, {findings} worth reading, {failures} that fail the gate"
        );
    }
    let _ = writeln!(
        out,
        "the threshold is {FACTOR:.0}x because these numbers move by a fifth between two runs on a \
         shared machine, widened per cell by the spread the record carries for it"
    );
    (out, failures > 0)
}

/// Run the measuring task in a rudb checkout and hand back what it measured.
///
/// The bench profile and the re-run under it are the task's own business, since a debug number is
/// not a number and it knows that better than this side does.
///
/// # Errors
///
/// A checkout that is not one, a cargo that will not build, a task that exits non zero, or output
/// that does not parse. All four name what was run.
pub fn measure(repo: &Path) -> Result<Vec<Cell>, String> {
    if !repo.join("xtask").is_dir() {
        return Err(format!(
            "{} is not a rudb checkout, it has no xtask directory. \
             Pass --repo <path> or set RUDB_BENCH_RUDB_REPO",
            repo.display()
        ));
    }
    let out = Command::new("cargo")
        .current_dir(repo)
        .args([
            "run",
            "--quiet",
            "--profile",
            "bench",
            "--package",
            "xtask",
            "--",
            "kernels",
            "--json",
        ])
        .output()
        .map_err(|e| format!("could not run cargo in {}: {e}", repo.display()))?;
    if !out.status.success() {
        return Err(format!(
            "cargo xtask kernels failed in {}:\n{}",
            repo.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    parse_json(&String::from_utf8_lossy(&out.stdout))
}

/// Which rudb this is, as a version and a short commit.
///
/// Best effort on purpose. A checkout with no git in it still measures, and a record that says the
/// version but not the commit is worth more than no record.
#[must_use]
pub fn describe(repo: &Path) -> String {
    let version = std::fs::read_to_string(repo.join("Cargo.toml"))
        .ok()
        .and_then(|text| {
            text.lines()
                .find_map(|line| line.strip_prefix("version = \""))
                .and_then(|rest| rest.split('"').next())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "unknown".to_string());
    let commit = Command::new("git")
        .current_dir(repo)
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|text| !text.is_empty());
    match commit {
        Some(commit) => format!("{version} {commit}"),
        None => version,
    }
}

/// Take a record of what was just measured.
#[must_use]
pub fn record_of(cells: Vec<Cell>, rudb: String) -> Record {
    Record { machine: machine::name_here(), rudb, recorded: today(), cells }
}

/// Where the rudb checkout is.
///
/// A flag first, then the environment, then a sibling directory, which is the layout every machine
/// in the fleet has because the two repositories are cloned next to each other.
#[must_use]
pub fn repo(flag: Option<&str>) -> PathBuf {
    if let Some(given) = flag {
        return PathBuf::from(given);
    }
    if let Some(set) = std::env::var_os("RUDB_BENCH_RUDB_REPO") {
        return PathBuf::from(set);
    }
    PathBuf::from("..").join("rudb")
}

/// Where the committed record lives.
#[must_use]
pub fn path() -> PathBuf {
    std::env::var_os("RUDB_BENCH_KERNELS")
        .map_or_else(|| Path::new("baselines").join("kernels.txt"), PathBuf::from)
}

/// The header the record file carries.
const HEADER: &str = "\
# The kernel record, read by `rudb-bench kernels --check`.
#
# One block per machine. Written by `rudb-bench kernels --record`, which runs `cargo xtask kernels
# --json` in a rudb checkout and stores what it said. The numbers are nanoseconds per row and every
# machine has its own, so a block is only ever compared against a later block from the same machine.
#
# A cell line is the table, the case, the detail, the form pair or stage, the null percentage, the
# rows the time was divided by, nanoseconds per row, the interquartile range as a percentage of the
# median, and whether a specialized loop ran or the row at a time path did. Separated by pipes
# because a case name has spaces in it and a column layout that breaks on the first one is a file
# format that works until somebody adds a case.
#
# This is also the committed form of the compaction surface, which sub-milestone 2b asks for by
# name: the rows whose table is `compaction` are it.
#
# Rule ten applies here as everywhere. Nothing in this file times a query, so nothing in it is a
# result on its own.
";

/// Render records as the file format.
#[must_use]
pub fn render(records: &[Record]) -> String {
    let mut out = String::from(HEADER);
    for record in records {
        let _ = writeln!(out);
        let _ = writeln!(out, "[kernels]");
        let _ = writeln!(out, "machine   {}", record.machine);
        let _ = writeln!(out, "rudb      {}", record.rudb);
        let _ = writeln!(out, "recorded  {}", record.recorded);
        for cell in &record.cells {
            let _ = writeln!(
                out,
                "cell      {} | {} | {} | {} | {} | {} | {:.4} | {:.1} | {}",
                cell.table,
                cell.case,
                cell.detail,
                cell.variant,
                cell.nulls,
                cell.rows,
                cell.nanos,
                cell.spread * 100.0,
                if cell.fell_back { "oracle" } else { "kernel" }
            );
        }
    }
    out
}

/// Parse the file format.
///
/// # Errors
///
/// An unknown key, a cell line without nine fields, a number that is not one, or a line before the
/// first block. Every one of them names the line it was on.
pub fn parse(text: &str) -> Result<Vec<Record>, String> {
    let mut records: Vec<Record> = Vec::new();
    for (at, line) in text.lines().enumerate() {
        let at = at + 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[kernels]" {
            records.push(Record {
                machine: String::new(),
                rudb: String::new(),
                recorded: String::new(),
                cells: Vec::new(),
            });
            continue;
        }
        let record = records.last_mut().ok_or(format!("line {at}: before the first [kernels]"))?;
        let (key, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
        let rest = rest.trim();
        match key {
            "machine" => record.machine = rest.to_string(),
            "rudb" => record.rudb = rest.to_string(),
            "recorded" => record.recorded = rest.to_string(),
            "cell" => record.cells.push(cell_line(rest, at)?),
            other => return Err(format!("line {at}: unknown key {other}")),
        }
    }
    Ok(records)
}

/// One cell line of the file format.
fn cell_line(rest: &str, at: usize) -> Result<Cell, String> {
    let fields: Vec<&str> = rest.split('|').map(str::trim).collect();
    let [table, case, detail, variant, nulls, rows, nanos, spread, path] = fields.as_slice() else {
        return Err(format!("line {at}: a cell has nine fields and this has {}", fields.len()));
    };
    let number = |text: &str, what: &str| -> Result<f64, String> {
        text.parse::<f64>().map_err(|_| format!("line {at}: {what} is not a number: {text}"))
    };
    let count = |text: &str, what: &str| -> Result<usize, String> {
        text.parse::<usize>().map_err(|_| format!("line {at}: {what} is not a count: {text}"))
    };
    let fell_back = match *path {
        "oracle" => true,
        "kernel" => false,
        other => return Err(format!("line {at}: a path is kernel or oracle, not {other}")),
    };
    Ok(Cell {
        table: (*table).to_string(),
        case: (*case).to_string(),
        detail: (*detail).to_string(),
        variant: (*variant).to_string(),
        nulls: count(nulls, "the null percentage")?,
        rows: count(rows, "the row count")?,
        nanos: number(nanos, "the time")?,
        spread: number(spread, "the spread")? / 100.0,
        fell_back,
    })
}

/// Read the committed record, or nothing at all when there is no file yet.
///
/// # Errors
///
/// A file that exists and does not parse, which is a broken commit rather than a missing one.
pub fn read(at: &Path) -> Result<Vec<Record>, String> {
    match std::fs::read_to_string(at) {
        Ok(text) => parse(&text).map_err(|e| format!("{}: {e}", at.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(format!("could not read {}: {e}", at.display())),
    }
}

/// Write a record, replacing the one for the same machine.
///
/// # Errors
///
/// Anything the filesystem says, by name.
pub fn write(at: &Path, taken: Record) -> Result<(), String> {
    let mut kept = read(at)?;
    kept.retain(|old| old.machine != taken.machine);
    kept.push(taken);
    kept.sort_by(|a, b| a.machine.cmp(&b.machine));
    if let Some(parent) = at.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("could not make {}: {e}", parent.display()))?;
    }
    std::fs::write(at, render(&kept)).map_err(|e| format!("could not write {}: {e}", at.display()))
}

/// The comma separated fields of one object, with commas inside a quoted string left alone.
///
/// The naive split is what the first version of this did and it broke on the first real run, on the
/// string table's case name `prefix decides, inline`. Quoting the comma is what JSON is for and the
/// producing side was right to write it that way, so the reading side tracks the quote. It does not
/// handle a backslash escape, because there is no escaping on the producing side at all and a test
/// there pins the character set that makes that true.
fn fields(body: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut quoted = false;
    let mut start = 0;
    for (at, byte) in body.bytes().enumerate() {
        match byte {
            b'"' => quoted = !quoted,
            b',' if !quoted => {
                out.push(&body[start..at]);
                start = at + 1;
            }
            _ => {}
        }
    }
    out.push(&body[start..]);
    out
}

/// Parse what `cargo xtask kernels --json` prints.
///
/// Hand written, and deliberately strict rather than general. This is not a JSON parser and must
/// not turn into one: it reads the exact shape the producing task writes, one object per line with
/// the keys in a fixed order and no escaping anywhere, which that task has a test pinning. A
/// tolerant parser here would mean the day the producing side starts emitting something else, this
/// one quietly reads it as something plausible.
///
/// # Errors
///
/// Anything that is not that shape, with the line number.
pub fn parse_json(text: &str) -> Result<Vec<Cell>, String> {
    let mut cells = Vec::new();
    for (at, line) in text.lines().enumerate() {
        let at = at + 1;
        let line = line.trim().trim_end_matches(',');
        if line.is_empty() || line == "[" || line == "]" {
            continue;
        }
        let body = line
            .strip_prefix('{')
            .and_then(|rest| rest.strip_suffix('}'))
            .ok_or(format!("line {at}: not one object on one line"))?;

        let mut cell = Cell {
            table: String::new(),
            case: String::new(),
            detail: String::new(),
            variant: String::new(),
            nulls: 0,
            rows: 0,
            nanos: 0.0,
            spread: 0.0,
            fell_back: false,
        };
        let mut seen = 0;
        for field in fields(body) {
            let (key, value) = field
                .split_once(':')
                .ok_or(format!("line {at}: a field is a key and a value: {field}"))?;
            let key = key.trim().trim_matches('"');
            let value = value.trim();
            let text = value.trim_matches('"');
            let number = |what: &str| -> Result<f64, String> {
                text.parse::<f64>()
                    .map_err(|_| format!("line {at}: {what} is not a number: {text}"))
            };
            let count = |what: &str| -> Result<usize, String> {
                text.parse::<usize>()
                    .map_err(|_| format!("line {at}: {what} is not a count: {text}"))
            };
            match key {
                "table" => cell.table = text.to_string(),
                "case" => cell.case = text.to_string(),
                "detail" => cell.detail = text.to_string(),
                "variant" => cell.variant = text.to_string(),
                "nulls" => cell.nulls = count("nulls")?,
                "rows" => cell.rows = count("rows")?,
                "nanos" => cell.nanos = number("nanos")?,
                "spread" => cell.spread = number("spread")?,
                "fell_back" => {
                    cell.fell_back = match text {
                        "true" => true,
                        "false" => false,
                        other => {
                            return Err(format!("line {at}: fell_back is a bool, not {other}"));
                        }
                    };
                }
                other => return Err(format!("line {at}: unknown key {other}")),
            }
            seen += 1;
        }
        if seen != 9 {
            return Err(format!("line {at}: a cell has nine fields and this has {seen}"));
        }
        cells.push(cell);
    }
    if cells.is_empty() {
        return Err("the task printed no cells, which is not a run".to_string());
    }
    Ok(cells)
}

/// The short version of what was measured, for the top of a run.
#[must_use]
pub fn summary(cells: &[Cell]) -> String {
    let oracle = cells.iter().filter(|cell| cell.fell_back).count();
    let mut tables: Vec<&str> = cells.iter().map(|cell| cell.table.as_str()).collect();
    tables.sort_unstable();
    tables.dedup();
    format!(
        "{} cells over {} tables, {oracle} of them on the row at a time path",
        cells.len(),
        tables.len()
    )
}

#[cfg(test)]
mod tests {
    use super::{Cell, Judgement, Record, compare, parse, parse_json, render, report, summary};

    /// Two lines of what the producing task prints, verbatim.
    const JSON: &str = "[\n  \
        {\"table\":\"comparison\",\"case\":\"Int32\",\"detail\":\"INTEGER\",\"variant\":\"flat/flat\",\"nulls\":0,\"rows\":1024,\"nanos\":0.4100,\"spread\":0.0120,\"fell_back\":false},\n  \
        {\"table\":\"compaction\",\"case\":\"2 bigint + varchar\",\"detail\":\"25\",\"variant\":\"select build\",\"nulls\":0,\"rows\":255,\"nanos\":10.6600,\"spread\":0.1800,\"fell_back\":false}\n]\n";

    fn cell(table: &str, case: &str, nanos: f64, fell_back: bool) -> Cell {
        Cell {
            table: table.to_string(),
            case: case.to_string(),
            detail: "INTEGER".to_string(),
            variant: "flat/flat".to_string(),
            nulls: 0,
            rows: 1024,
            nanos,
            spread: 0.1,
            fell_back,
        }
    }

    fn record(cells: Vec<Cell>) -> Record {
        Record {
            machine: "server3".to_string(),
            rudb: "0.2.2 f45d284".to_string(),
            recorded: "2026-09-11".to_string(),
            cells,
        }
    }

    #[test]
    fn the_json_the_task_prints_is_the_json_this_reads() {
        let cells = parse_json(JSON).expect("parses");
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].table, "comparison");
        assert!((cells[0].nanos - 0.41).abs() < 1e-9);
        assert!(!cells[0].fell_back);
        // The case with a space and a plus in it, which is the one a column based format loses.
        assert_eq!(cells[1].case, "2 bigint + varchar");
        assert_eq!(cells[1].rows, 255);
    }

    #[test]
    fn a_comma_inside_a_case_name_is_part_of_the_name() {
        // The string table's case names are `prefix decides, inline` and friends. The first version
        // of this parser split the object on every comma and fell over on the first real run.
        let line = "[\n  {\"table\":\"strings\",\"case\":\"prefix decides, inline\",\"detail\":\"VARCHAR\",\
            \"variant\":\"flat/flat\",\"nulls\":0,\"rows\":1024,\"nanos\":4.7500,\"spread\":0.0300,\"fell_back\":false}\n]\n";
        let cells = parse_json(line).expect("parses");
        assert_eq!(cells[0].case, "prefix decides, inline");
        assert_eq!(cells[0].variant, "flat/flat");
    }

    #[test]
    fn output_that_is_not_that_shape_is_an_error_rather_than_a_guess() {
        assert!(parse_json("[\n]\n").is_err(), "no cells is not a run");
        assert!(parse_json("{\"table\":\"x\"}").is_err(), "a cell has nine fields");
        let bad = JSON.replace("\"fell_back\":false", "\"fell_back\":maybe");
        assert!(parse_json(&bad).is_err(), "a bool is a bool");
    }

    #[test]
    fn a_record_survives_being_written_and_read() {
        let written = record(parse_json(JSON).expect("parses"));
        let read = parse(&render(std::slice::from_ref(&written))).expect("parses");
        assert_eq!(read.len(), 1);
        assert_eq!(read[0].machine, "server3");
        assert_eq!(read[0].rudb, "0.2.2 f45d284");
        // Not a formality. The case name has a space and the detail is a number that has to come
        // back as the same string, and both of those are what the pipes are there for.
        assert_eq!(read[0].cells, written.cells);
    }

    #[test]
    fn losing_a_loop_fails_whatever_it_cost() {
        // The point of this one. A cell that falls to the row at a time path is a factor of fifty,
        // and the flag says so without waiting for the time to agree.
        let was = record(vec![cell("comparison", "Int32", 0.41, false)]);
        let now = vec![cell("comparison", "Int32", 0.42, true)];
        let changes = compare(&was, &now);
        assert_eq!(changes[0].judgement, Judgement::LostSpecialization);
        assert!(changes[0].judgement.failed());
    }

    #[test]
    fn gaining_a_loop_is_reported_and_never_failed() {
        let was = record(vec![cell("comparison", "Int32", 80.0, true)]);
        let now = vec![cell("comparison", "Int32", 1.5, false)];
        let changes = compare(&was, &now);
        assert_eq!(changes[0].judgement, Judgement::GainedSpecialization);
        assert!(!changes[0].judgement.failed());
    }

    #[test]
    fn a_fifth_slower_is_weather_and_twice_is_a_finding() {
        let was = record(vec![cell("comparison", "Int32", 1.00, false)]);
        let drifted = compare(&was, &[cell("comparison", "Int32", 1.20, false)]);
        assert!(matches!(drifted[0].judgement, Judgement::Unchanged(_)));
        // Past two, and past the tenth of spread the helper gives every cell.
        let broken = compare(&was, &[cell("comparison", "Int32", 2.50, false)]);
        assert!(broken[0].judgement.failed());
    }

    #[test]
    fn a_cell_that_moves_on_its_own_gets_a_threshold_that_says_so() {
        // The real incident. `compaction 2 bigint 1 select build` recorded 341 with an
        // interquartile range of 58 percent, and the next two runs of the same binary on the same
        // machine gave 86.3 and 86.7. A flat factor of two would have failed that cell the first
        // time it landed on the high side instead of the low one.
        let mut wild = cell("compaction", "2 bigint", 100.0, false);
        wild.spread = 0.58;
        let was = record(vec![wild.clone()]);
        let mut read_high = wild.clone();
        read_high.nanos = 300.0;
        assert!(!compare(&was, &[read_high]).into_iter().next().unwrap().judgement.failed());
        // A steady cell keeps the plain factor, so the widening is a property of the cell rather
        // than a blanket loosening of the gate.
        let mut steady = wild;
        steady.spread = 0.05;
        let was = record(vec![steady.clone()]);
        let mut trebled = steady;
        trebled.nanos = 300.0;
        assert!(compare(&was, &[trebled]).into_iter().next().unwrap().judgement.failed());
    }

    #[test]
    fn a_cell_the_record_does_not_have_is_new_rather_than_a_failure() {
        let was = record(vec![cell("comparison", "Int32", 1.0, false)]);
        let changes = compare(&was, &[cell("comparison", "Int8", 1.0, false)]);
        assert_eq!(changes[0].judgement, Judgement::New);
        assert!(!changes[0].judgement.failed());
    }

    #[test]
    fn a_cell_the_fresh_run_does_not_have_is_not_reported_at_all() {
        // A dropped column is a change to the measuring code in the other repository, and reporting
        // it here would mean every run after a deliberate change printed a wall of findings.
        let was = record(vec![
            cell("comparison", "Int32", 1.0, false),
            cell("comparison", "Int8", 1.0, false),
        ]);
        let changes = compare(&was, &[cell("comparison", "Int32", 1.0, false)]);
        assert_eq!(changes.len(), 1);
    }

    #[test]
    fn a_clean_run_says_so_in_one_line_rather_than_printing_every_cell() {
        let was = record(vec![cell("comparison", "Int32", 1.0, false)]);
        let (text, failed) = report(&compare(&was, &[cell("comparison", "Int32", 1.1, false)]));
        assert!(!failed);
        assert!(text.contains("1 cells, none of them past"), "{text}");
    }

    #[test]
    fn the_threshold_is_printed_every_time_and_not_remembered() {
        let was = record(vec![cell("comparison", "Int32", 1.0, false)]);
        let (text, _) = report(&compare(&was, &[cell("comparison", "Int32", 1.0, false)]));
        assert!(text.contains("move by a fifth"), "{text}");
    }

    #[test]
    fn a_summary_counts_the_tables_and_the_fallbacks() {
        let cells =
            vec![cell("comparison", "Int32", 1.0, false), cell("sizes", "compare", 80.0, true)];
        let text = summary(&cells);
        assert!(text.contains("2 cells over 2 tables"), "{text}");
        assert!(text.contains("1 of them on the row at a time path"), "{text}");
    }
}
