//! Running one suite once per registered implementation of one seam, everything else held fixed.
//!
//! This is the apparatus the whole strategy registry exists for. A seam is a place in the engine
//! where more than one implementation is defensible, and the argument for having a registry at all
//! is that the choice becomes a measurement rather than an opinion. That argument only cashes out
//! if there is a command that runs the same suite over the same data on the same machine with one
//! seam moved and nothing else, so that the difference between two columns is the thing that was
//! changed.
//!
//! At F0 there is nothing registered at any seam, so every sweep prints one row: the reference. That
//! is the correct output rather than a stub. The point of building it now is that the day somebody
//! lands a second hash table, the way to find out whether it is faster already exists and is the
//! same command it will always be, instead of being an afternoon of shell scripts whose results
//! nobody can reproduce.
//!
//! The engine is asked what its seams are rather than being told. `rudb_strategies()` is the public
//! list and `EXPLAIN` reads the same one, so a seam that exists is sweepable without anything here
//! being changed, and a seam name this harness knows but the engine does not is impossible rather
//! than being a silent run of the default.

use std::path::Path;
use std::process::Command;

use crate::engine::{BenchError, Engine, Rudb};
use crate::measure::show;
use crate::report::{SuiteResult, run};
use crate::suite::{Query, Suite};

/// One registered implementation of one seam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Implementation {
    /// What to pin the seam to, which is what `SET seam.<id> = '<name>'` takes.
    pub name: String,
    /// The one line the registry carries, printed under the table.
    pub description: String,
    /// Where it came from: a paper, another engine, or this project.
    pub provenance: String,
    /// Whether this is the reference, which is the slow one kept for differential testing.
    pub is_reference: bool,
    /// Whether this is what an unpinned session gets.
    pub is_default: bool,
}

/// One seam and everything registered at it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seam {
    /// The dotted name, which is what the flag and the `SET` both take.
    pub id: String,
    /// The milestone that fills it, from the design documents.
    pub milestone: String,
    /// What the seam is, in one line.
    pub description: String,
    /// Everything registered here, in the order the engine lists it.
    pub implementations: Vec<Implementation>,
}

impl Seam {
    /// What a sweep of this seam runs, in the order it runs them.
    ///
    /// The reference first, because the ratio column is against the first row and a ratio against
    /// the thing every implementation has to agree with is the one worth reading. A seam with
    /// nothing registered sweeps as one unpinned run, which is the reference by definition since
    /// there is nothing else for the policy to pick.
    #[must_use]
    pub fn variants(&self) -> Vec<Variant> {
        if self.implementations.is_empty() {
            return vec![Variant {
                label: "reference".to_owned(),
                pin: None,
                provenance: "nothing is registered here yet".to_owned(),
            }];
        }
        let mut variants: Vec<Variant> = self
            .implementations
            .iter()
            .map(|i| Variant {
                label: i.name.clone(),
                pin: Some((self.id.clone(), i.name.clone())),
                provenance: i.provenance.clone(),
            })
            .collect();
        let reference = self.implementations.iter().position(|i| i.is_reference);
        if let Some(at) = reference {
            variants.swap(0, at);
        }
        variants
    }
}

/// One column of a sweep: the engine with the seam set one way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    /// The implementation name, which is what the row is labelled with.
    pub label: String,
    /// What to pin, or nothing when the run is the engine as it comes.
    pub pin: Option<(String, String)>,
    /// Where the implementation came from, printed under the table.
    pub provenance: String,
}

/// One row of a sweep: what the suite cost with the seam set one way.
#[derive(Debug, Clone)]
pub struct Row {
    /// Which setting this row is.
    pub variant: Variant,
    /// What the suite measured, or why it did not run.
    pub result: Result<SuiteResult, String>,
}

/// A whole sweep, ready to print.
#[derive(Debug, Clone)]
pub struct Sweep {
    /// The seam that was moved.
    pub seam: Seam,
    /// The suite that was held fixed.
    pub suite: &'static Suite,
    /// The engine version every row ran under, since one binary runs all of them.
    pub version: String,
    /// One row per implementation, in the order they ran.
    pub rows: Vec<Row>,
}

/// Every seam the engine has, with everything registered at each of them.
///
/// # Errors
///
/// When there is no rudb to ask, when it will not run, or when it answers with something that is
/// not the nine column shape `rudb_strategies()` is documented to have.
pub fn seams(binary: &Path) -> Result<Vec<Seam>, String> {
    let out = Command::new(binary)
        .args(["-batch", "-csv", "-noheader", "-c"])
        .arg(
            "SELECT seam, milestone, seam_description, implementation, implementation_description, \
             provenance, is_reference, is_default FROM rudb_strategies()",
        )
        .output()
        .map_err(|e| format!("cannot run {}: {e}", binary.display()))?;
    if !out.status.success() {
        let why = String::from_utf8_lossy(&out.stderr);
        return Err(format!("{} would not list its seams: {}", binary.display(), why.trim()));
    }
    read_seams(&String::from_utf8_lossy(&out.stdout))
}

/// Read what `rudb_strategies()` answered, with the process left outside.
///
/// Split out so that the grouping can be tested against the shape the engine actually writes,
/// without a build of rudb being a thing this crate's test suite needs.
fn read_seams(text: &str) -> Result<Vec<Seam>, String> {
    let mut seams: Vec<Seam> = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let row = fields(line);
        let [id, milestone, description, name, of_name, provenance, is_reference, is_default] =
            &row[..]
        else {
            return Err(format!(
                "rudb_strategies() answered with {} columns and this reads eight, so the engine \
                 and the harness disagree about what that table is: `{line}`",
                row.len()
            ));
        };
        // One row per seam per implementation, so a seam with three of them arrives three times and
        // a seam with none arrives once with every implementation column null.
        if seams.last().map(|seam| seam.id.as_str()) != Some(id.as_str()) {
            seams.push(Seam {
                id: id.clone(),
                milestone: milestone.clone(),
                description: description.clone(),
                implementations: Vec::new(),
            });
        }
        if name == "NULL" {
            continue;
        }
        if let Some(seam) = seams.last_mut() {
            seam.implementations.push(Implementation {
                name: name.clone(),
                description: of_name.clone(),
                provenance: provenance.clone(),
                is_reference: is_reference == "true",
                is_default: is_default == "true",
            });
        }
    }
    if seams.is_empty() {
        return Err("rudb_strategies() is empty, which means this is not a build with seams in it"
            .to_owned());
    }
    Ok(seams)
}

/// The seam with this name, or the list of the ones there are.
///
/// The prefix is accepted because `seam.hash.table` is what a script writes and `hash.table` is
/// what a person says, and the engine takes both for the same reason.
///
/// # Errors
///
/// When nothing is called that. The message is the list, because a mistyped seam name is the
/// commonest way to get a sweep that measured the default twice.
pub fn find<'a>(seams: &'a [Seam], name: &str) -> Result<&'a Seam, String> {
    // Twice, because the underscored spelling carries the prefix as `seam_` and only becomes
    // `seam.` once the underscores are dots, which is the same two steps the engine takes.
    let dotted = name.strip_prefix("seam.").unwrap_or(name).replace('_', ".");
    let wanted = dotted.strip_prefix("seam.").unwrap_or(&dotted);
    seams.iter().find(|seam| seam.id == wanted).ok_or_else(|| {
        let all: Vec<&str> = seams.iter().map(|seam| seam.id.as_str()).collect();
        format!("no seam called {name}, the ones there are: {}", all.join(", "))
    })
}

/// Run the suite once per implementation of this seam.
///
/// Everything outside the seam is the same on every row by construction: one binary, one dataset,
/// one machine, one number of runs, and the rows run back to back so that the machine has as little
/// chance to change underneath them as this harness can arrange. That is the whole claim a sweep
/// makes and it is worth being boring about.
///
/// A variant that will not run is a row saying so rather than an abandoned sweep, because the
/// interesting case is exactly the one where a new implementation is wrong on one query, and a
/// sweep that threw away the reference row in response would be a sweep that hid the comparison.
pub fn sweep(
    seam: &Seam,
    suite: &'static Suite,
    queries: &[Query],
    dataset: &crate::data::Dataset,
    scratch: &Path,
    hot: usize,
) -> Sweep {
    let mut rows = Vec::new();
    let mut version = String::new();
    for variant in seam.variants() {
        let mut engine = Rudb::discover(scratch, suite);
        if let Some((key, value)) = &variant.pin {
            engine.pin(key, value);
        }
        if version.is_empty() {
            version = engine.version().to_owned();
        }
        let result = match engine.can_run(suite).why() {
            Some(why) => Err(why.to_owned()),
            None => run(&mut engine, suite, queries, dataset, hot)
                .map_err(|e: BenchError| e.to_string()),
        };
        engine.unload();
        rows.push(Row { variant, result });
    }
    Sweep { seam: seam.clone(), suite, version, rows }
}

impl Sweep {
    /// The row every ratio is against, which is the first one that produced a number.
    fn reference(&self) -> Option<&SuiteResult> {
        self.rows.iter().find_map(|row| row.result.as_ref().ok())
    }
}

/// The sweep as a table, one row per implementation.
#[must_use]
pub fn table(swept: &Sweep) -> String {
    let mut out = String::new();
    out.push_str(&format!("seam      {} ({})\n", swept.seam.id, swept.seam.milestone));
    out.push_str(&format!("          {}\n", swept.seam.description));
    out.push_str(&format!("suite     {}, {} queries\n", swept.suite.name, swept.suite.queries));
    out.push_str(&format!("engine    {}\n\n", swept.version));

    let base = swept.reference().map(SuiteResult::hot_total);
    out.push_str(
        "implementation        hot total     hot cpu     peak RSS   worst IQR   vs first\n",
    );
    for row in &swept.rows {
        let Ok(result) = &row.result else {
            out.push_str(&format!("{:<20}  did not run\n", row.variant.label));
            continue;
        };
        let total = result.hot_total();
        let ratio = match base {
            Some(base) if !base.is_zero() && !total.is_zero() => {
                format!("{:.2}x", base.as_secs_f64() / total.as_secs_f64())
            }
            _ => "-".to_owned(),
        };
        out.push_str(&format!(
            "{:<20}  {:>9}  {:>10}  {:>11}  {:>9}  {:>9}\n",
            row.variant.label,
            show(total),
            result.hot_cpu().map_or_else(|| "not read".to_owned(), show),
            result.peak().bytes().map_or_else(|| "not read".to_owned(), crate::memory::bytes),
            result.worst_iqr().map_or_else(|| "-".to_owned(), |iqr| format!("{:.1}%", iqr * 100.0)),
            ratio
        ));
    }
    out.push('\n');
    for row in &swept.rows {
        out.push_str(&format!("{}: {}\n", row.variant.label, row.variant.provenance));
        if let Err(why) = &row.result {
            out.push_str(&format!("  did not run: {why}\n"));
        }
    }
    out.push('\n');
    if swept.rows.len() < 2 {
        out.push_str(
            "One row, because nothing is registered at this seam yet and the reference is\n",
        );
        out.push_str(
            "the only thing there is to run. That is the apparatus working rather than a\n",
        );
        out.push_str(
            "missing result. The second row appears on the day somebody registers a second\n",
        );
        out.push_str("implementation, and nothing here has to change for it to.\n");
    } else {
        out.push_str("Every row is the same binary over the same files on this machine, back to\n");
        out.push_str(
            "back, with this seam moved and nothing else. The ratio is against the first\n",
        );
        out.push_str("row, which is the reference, so over one is faster than the thing every\n");
        out.push_str("implementation at this seam has to agree with.\n");
    }
    out.push('\n');
    out.push_str("These are not publishable numbers. Reporting rule seven still applies and a\n");
    out.push_str(
        "sweep is a comparison of the engine against itself rather than against a board.\n",
    );
    out
}

/// Split one CSV line the way RFC 4180 says, which is the way rudb's shell writes one.
///
/// Hand written because this crate has no dependencies and because the input is one program's
/// output rather than a file somebody typed. What it has to get right is a quoted field with a
/// comma in it, since a seam description is prose, and a doubled quote inside one.
fn fields(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut rest = line.chars().peekable();
    while let Some(c) = rest.next() {
        match c {
            '"' if quoted && rest.peek() == Some(&'"') => {
                field.push('"');
                rest.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => out.push(std::mem::take(&mut field)),
            _ => field.push(c),
        }
    }
    out.push(field);
    out
}

#[cfg(test)]
mod tests {
    use super::{Implementation, Seam, fields, find, read_seams};

    #[test]
    fn a_seam_with_three_implementations_arrives_as_three_lines_and_becomes_one_seam() {
        // The shape rudb_strategies() has: one row per seam per implementation, so the grouping is
        // the harness's job and getting it wrong would put the same seam in the list twice.
        let text = "\
hash.table,F5,the hash table itself,chained,a bucket per hash,this project,true,false\n\
hash.table,F5,the hash table itself,unchained,\"tags, in the pointer\",Birler 2024,false,true\n\
sort,F9,sorting rows,NULL,NULL,NULL,NULL,NULL\n";
        let seams = read_seams(text).expect("three well formed lines");
        assert_eq!(seams.len(), 2);
        assert_eq!(seams[0].id, "hash.table");
        assert_eq!(seams[0].implementations.len(), 2);
        assert!(seams[0].implementations[0].is_reference);
        assert!(seams[0].implementations[1].is_default);
        assert_eq!(seams[0].implementations[1].description, "tags, in the pointer");
        assert_eq!(seams[1].id, "sort");
        assert!(seams[1].implementations.is_empty(), "every column null means nothing registered");
    }

    #[test]
    fn a_table_of_the_wrong_width_is_refused_rather_than_read_as_a_short_seam() {
        // The one thing that has to fail loudly. A harness that read seven columns as eight would
        // sweep the wrong seam and produce a table that looks entirely reasonable.
        let why = read_seams("hash.table,F5,the hash table itself,NULL,NULL,NULL,NULL\n")
            .expect_err("seven columns");
        assert!(why.contains("7 columns"), "{why}");
    }

    #[test]
    fn an_engine_with_no_seams_says_so_rather_than_sweeping_nothing() {
        let why = read_seams("").expect_err("no rows at all");
        assert!(why.contains("not a build with seams in it"), "{why}");
    }

    fn seam(id: &str, implementations: Vec<Implementation>) -> Seam {
        Seam {
            id: id.to_owned(),
            milestone: "F5".to_owned(),
            description: "the hash table itself".to_owned(),
            implementations,
        }
    }

    fn of(name: &str, is_reference: bool) -> Implementation {
        Implementation {
            name: name.to_owned(),
            description: String::new(),
            provenance: "this project".to_owned(),
            is_reference,
            is_default: false,
        }
    }

    #[test]
    fn a_seam_with_nothing_registered_sweeps_as_one_unpinned_run() {
        // The F0 case, and the one the whole command has to get right before it is ever useful. A
        // sweep of a seam nobody has filled is one row and no flag, not an error and not a run of
        // something called reference that does not exist.
        let variants = seam("hash.table", Vec::new()).variants();
        assert_eq!(variants.len(), 1);
        assert_eq!(variants[0].label, "reference");
        assert_eq!(variants[0].pin, None);
    }

    #[test]
    fn the_reference_runs_first_whatever_order_the_engine_lists_it_in() {
        // The ratio column is against the first row, and a ratio against whichever implementation
        // happened to be registered first is a number that changes meaning when somebody reorders a
        // registration file.
        let variants =
            seam("hash.table", vec![of("unchained", false), of("chained", true)]).variants();
        assert_eq!(variants[0].label, "chained");
        assert_eq!(variants[1].label, "unchained");
        assert_eq!(variants[0].pin, Some(("hash.table".to_owned(), "chained".to_owned())));
    }

    #[test]
    fn all_three_spellings_of_a_seam_name_find_the_same_seam() {
        // The engine takes all three and a harness that took one of them would be the reason
        // somebody's sweep failed with a name they had just copied out of an EXPLAIN.
        let all = vec![seam("hash.table", Vec::new())];
        assert_eq!(find(&all, "hash.table").unwrap().id, "hash.table");
        assert_eq!(find(&all, "seam.hash.table").unwrap().id, "hash.table");
        assert_eq!(find(&all, "seam_hash_table").unwrap().id, "hash.table");
    }

    #[test]
    fn a_seam_nobody_has_heard_of_comes_back_with_the_list_of_the_ones_there_are() {
        let all = vec![seam("hash.table", Vec::new()), seam("sort", Vec::new())];
        let why = find(&all, "hash.tabel").expect_err("a typo");
        assert!(why.contains("hash.table"), "{why}");
        assert!(why.contains("sort"), "{why}");
    }

    #[test]
    fn a_quoted_field_with_a_comma_in_it_is_one_field() {
        // Seam descriptions are prose and prose has commas in it, so a splitter that split on every
        // comma would read one row as nine columns and refuse the whole table.
        assert_eq!(fields("a,b,c"), vec!["a", "b", "c"]);
        assert_eq!(fields("a,\"b,c\",d"), vec!["a", "b,c", "d"]);
        assert_eq!(fields("\"say \"\"hi\"\"\",2"), vec!["say \"hi\"", "2"]);
        assert_eq!(fields("a,,b"), vec!["a", "", "b"]);
    }
}
