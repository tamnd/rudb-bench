//! Making a corpus, and writing down what was made.
//!
//! [`crate::data::prepare`] finds files. This makes them, and then says what they are. The two are
//! deliberately separate commands: a harness that generated seventy gigabytes because somebody
//! typed a suite name is a harness people run once, which is the rule `data.rs` already states and
//! this one keeps.
//!
//! # Why a manifest and not eight files
//!
//! Eight Parquet files under a directory are a corpus in the sense that the queries will run
//! against them. They are not a corpus in the sense that a number measured over them means
//! anything six months later, because nothing in the directory says which generator wrote them, at
//! which version, on what day, or whether the four physical facts the graph layer is built on were
//! ever true of this copy.
//!
//! Every one of those is cheap to record at generation time and impossible to recover afterwards.
//! So the generator writes a manifest beside the files, and it records what it did rather than what
//! it intended: the row counts it observed, the bytes on disk, a SHA-256 per file, and the four
//! properties of `spec/bench/tpc-h/02-the-data.md` section 2.4 as measurements with their observed
//! values rather than as claims.
//!
//! # The four properties are checked, not assumed
//!
//! `spec/graph/03-the-file-format.md` builds a 106 MB forward link out of `lineitem` being in
//! `l_orderkey` order, and `spec/graph/02-the-data-model.md` section 2.2 picks the bitmap key map
//! over the sorted one out of `o_orderkey` being about a quarter dense. Both are properties of the
//! data and neither is enforced by anything. A regenerated corpus that lost one of them would not
//! fail; it would produce a link ten times the size and a number that looks like a regression in
//! the engine.
//!
//! So each is one pass at generation time and its answer goes in the manifest. A property that does
//! not hold is written down as not holding rather than stopping the generation, because a corpus
//! whose `o_orderkey` is dense is still a usable TPC-H corpus and is only a problem for the layer
//! that assumed otherwise. What is not acceptable is not knowing.
//!
//! # Provenance is recorded, not flattened
//!
//! `spec/bench/tpc-h/02-the-data.md` section 2.2 makes the official `dbgen` the source of truth and
//! permits DuckDB's `tpch` extension up to SF10 as a *different* provenance. It is very probably
//! the same generator compiled into a different program, and "very probably" is not a property a
//! corpus should have when the number that comes out of it is a headline. So the manifest names
//! which one wrote it and a report built on it can say so.
//!
//! # The hash is SHA-256 from a system tool
//!
//! This crate has no dependencies and one corpus is not a reason to start. The digest comes from
//! `shasum -a 256` or `sha256sum`, whichever is on the machine, and which one is recorded. It is
//! the same number `shasum -a 256 lineitem.parquet` prints, so a reader who wants to check a
//! manifest does not need this harness to do it.
//!
//! # JOB is unpacked, not generated
//!
//! The Join Order Benchmark has no generator. Its data is one snapshot of IMDb from May 2013,
//! shipped as `imdb.tgz`, and every published JOB number is over that snapshot and nothing else. So
//! `generate job` downloads the archive, loads its CSVs into a DuckDB database that is thrown away
//! afterwards, and writes one Parquet file per table in the order the CSV had its rows. The result
//! is the same shape as a TPC-H corpus, a directory of Parquet files and a manifest beside them,
//! which is what lets [`crate::data::prepare`] find it without knowing where it came from.
//!
//! The manifest records the archive it was unpacked from and that archive's SHA-256, because the
//! Parquet digests are only as good as the thing they were made out of. Two mirrors serve the
//! archive and nothing but the digest says they served the same bytes.
//!
//! The download is `curl`, for the reason the digest is `shasum`: one download is not a reason to
//! give this crate dependencies, and `curl -C -` already knows how to pick up a 1.2 GB transfer
//! where it stopped, which is the part that matters on the slower of the two mirrors.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::data::{output, root};
use crate::engine::BenchError;
use crate::suite::{JOB_SCHEMA, Scale, Size, Suite};

/// Which generator wrote a corpus.
///
/// Two values rather than one, for the reason in this module's header: they are very probably the
/// same data and a corpus does not get to be very probably anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// The official `dbgen` from the TPC-H tools distribution, built from source.
    Dbgen,
    /// DuckDB's `tpch` extension, `CALL dbgen(sf = n)`, permitted up to SF10.
    DuckdbTpch,
    /// The IMDb snapshot the JOB paper used, as the `imdb.tgz` archive its authors published.
    ///
    /// Not a generator at all, and recorded as its own provenance rather than folded into one of
    /// the two above, because a manifest that said `dbgen` about IMDb data would be the kind of
    /// flattening this enum exists to refuse.
    ImdbArchive,
}

impl Provenance {
    /// What the manifest writes and reads back.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Dbgen => "dbgen",
            Self::DuckdbTpch => "duckdb-tpch",
            Self::ImdbArchive => "imdb-archive",
        }
    }

    /// That label, read back.
    #[must_use]
    pub fn parse(given: &str) -> Option<Self> {
        match given.trim() {
            "dbgen" => Some(Self::Dbgen),
            "duckdb-tpch" => Some(Self::DuckdbTpch),
            "imdb-archive" => Some(Self::ImdbArchive),
            _ => None,
        }
    }

    /// The sentence a report carries when a result was measured over this corpus.
    ///
    /// Empty for the official generator, because a TPC-H result over `dbgen` output needs no note,
    /// and empty for the IMDb archive, because that archive is the JOB corpus rather than a second
    /// way of making it.
    #[must_use]
    pub fn note(&self) -> &'static str {
        match self {
            Self::Dbgen | Self::ImdbArchive => "",
            Self::DuckdbTpch => {
                "generated by DuckDB's tpch extension rather than by the official dbgen, which is \
                 very probably the same data and is recorded rather than assumed"
            }
        }
    }
}

/// One table, as it came out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    /// The table, as the suite names it.
    pub table: String,
    /// Rows counted in the file that was written, not the count the specification predicts.
    pub rows: u64,
    /// Bytes the Parquet file takes.
    pub bytes: u64,
    /// SHA-256 of the file, lowercase hex.
    pub sha256: String,
}

/// Where a downloaded corpus came from, and what the download was.
///
/// `None` on a manifest for a generated corpus, because there the generator and its version are the
/// source and they are already written down. A corpus unpacked from an archive has no generator to
/// name, and what stands in for one is the archive and its digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// Where the archive was fetched from, which is whichever mirror answered, or the file it was
    /// found as when it was already on disk and nothing says where it came from.
    pub archive: String,
    /// SHA-256 of the archive as it was unpacked, lowercase hex.
    ///
    /// The one fact that says two corpora unpacked from two mirrors started from the same bytes.
    pub sha256: String,
}

/// One physical property, as it was measured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property {
    /// What it is called, which is also what a report would name in a footnote.
    pub name: String,
    /// Whether it held.
    pub holds: bool,
    /// What was actually seen, which is the part worth having when it did not hold.
    ///
    /// A density of 0.25 and a density of 0.98 both answer "is it sparse" and only one of them
    /// tells the reader which key map the graph layer is going to build.
    pub observed: String,
}

/// What is known about one corpus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// The suite it belongs to.
    pub suite: String,
    /// The scale factor's label, as `--scale` takes it, or [`FIXED`] for a suite that has one
    /// corpus of one size.
    pub scale: String,
    /// Which generator wrote it.
    pub provenance: Provenance,
    /// The generator and its version, in one line somebody can read.
    pub generator: String,
    /// The DuckDB that turned the generator's output into Parquet.
    pub converter: String,
    /// Which tool produced the digests, so a reader can run the same one.
    pub hashed_by: String,
    /// The day it was written, UTC.
    pub written: String,
    /// The archive the corpus was unpacked from, for one that was downloaded rather than generated.
    pub source: Option<Source>,
    /// One per table, in the order the suite names them.
    pub tables: Vec<Recorded>,
    /// The physical properties, measured.
    pub properties: Vec<Property>,
}

impl Manifest {
    /// Where the manifest for a corpus lives.
    #[must_use]
    pub fn path(directory: &Path) -> PathBuf {
        directory.join("manifest.txt")
    }

    /// The one line a report header carries about where its data came from.
    ///
    /// Provenance first, because a number over `duckdb-tpch` and a number over `dbgen` are not
    /// quite the same claim and that is the part somebody reading a table needs to see without
    /// opening anything. The date is in it because a corpus regenerated between two engines' runs
    /// is the failure this line exists to make visible.
    ///
    /// No scale factor for the IMDb archive, because it has none and `SFfixed` in a header would be
    /// a scale factor that means nothing.
    #[must_use]
    pub fn line(&self) -> String {
        let label = match self.provenance {
            Provenance::Dbgen | Provenance::DuckdbTpch => {
                format!("{} SF{}", self.provenance.label(), self.scale)
            }
            Provenance::ImdbArchive => self.provenance.label().to_owned(),
        };
        format!("{label}, corpus {}, written {}", self.digest(), self.written)
    }

    /// The corpus as one short string a report header can carry.
    ///
    /// Sixteen hex digits folded out of the per table digests, in table order. The identity of a
    /// corpus is the eight SHA-256s in the manifest and this is not a replacement for them: it is
    /// what fits in a header line so that two tables can be told apart at a glance, and the fold is
    /// FNV-1a rather than anything stronger because nothing here is defending against somebody who
    /// wants two corpora to collide. What it has to do is come out the same for the same eight
    /// files and different for different ones, and the inputs are in the manifest for anybody who
    /// wants to redo it.
    ///
    /// Eight digests printed in full is five hundred characters and eight truncated to eight digits
    /// each is still seventy one, neither of which is a header line.
    #[must_use]
    pub fn digest(&self) -> String {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for table in &self.tables {
            for byte in table.sha256.as_bytes() {
                hash ^= u64::from(*byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        format!("{hash:016x}")
    }

    /// Total rows over every table, as counted.
    #[must_use]
    pub fn rows(&self) -> u64 {
        self.tables.iter().map(|table| table.rows).sum()
    }

    /// Total bytes over every table.
    #[must_use]
    pub fn bytes(&self) -> u64 {
        self.tables.iter().map(|table| table.bytes).sum()
    }

    /// The properties that were measured and did not hold.
    #[must_use]
    pub fn broken(&self) -> Vec<&Property> {
        self.properties.iter().filter(|property| !property.holds).collect()
    }

    /// The manifest as the file holds it.
    #[must_use]
    pub fn write(&self) -> String {
        let mut out = String::from(PREAMBLE);
        out.push_str("\n[corpus]\n");
        let _ = writeln!(out, "suite {}", self.suite);
        let _ = writeln!(out, "scale {}", self.scale);
        let _ = writeln!(out, "provenance {}", self.provenance.label());
        let _ = writeln!(out, "generator {}", self.generator);
        let _ = writeln!(out, "converter {}", self.converter);
        let _ = writeln!(out, "hashed-by {}", self.hashed_by);
        let _ = writeln!(out, "written {}", self.written);
        if let Some(source) = &self.source {
            let _ = writeln!(out, "archive {}", source.archive);
            let _ = writeln!(out, "archive-sha256 {}", source.sha256);
        }
        for table in &self.tables {
            let _ = writeln!(out, "\n[table] {}", table.table);
            let _ = writeln!(out, "rows {}", table.rows);
            let _ = writeln!(out, "bytes {}", table.bytes);
            let _ = writeln!(out, "sha256 {}", table.sha256);
        }
        for property in &self.properties {
            let _ = writeln!(out, "\n[property] {}", property.name);
            let _ = writeln!(out, "holds {}", if property.holds { "yes" } else { "no" });
            let _ = writeln!(out, "observed {}", property.observed);
        }
        out
    }

    /// The manifest as it was written, read back.
    ///
    /// # Errors
    ///
    /// When a key the manifest cannot do without is missing or does not parse. A partial manifest
    /// is refused rather than filled in, because every field here exists to be the answer to a
    /// question nobody can answer any other way, and a default is a wrong answer that reads like a
    /// right one.
    pub fn read(text: &str) -> Result<Self, String> {
        let corpus = blocks(text, "[corpus]");
        let corpus =
            corpus.first().map(|(_, body)| body).ok_or("the manifest has no [corpus] block")?;
        let provenance = need(corpus, "provenance")?;
        let provenance = Provenance::parse(provenance)
            .ok_or_else(|| format!("{provenance} is not a provenance this harness knows"))?;
        // Both or neither. An archive with no digest is a download nobody can check, and a digest
        // with no archive is a number with nothing to be the digest of, so half of one is refused
        // for the same reason a missing field is.
        let source = match (value(corpus, "archive"), value(corpus, "archive-sha256")) {
            (Some(archive), Some(sha256)) => {
                Some(Source { archive: archive.to_owned(), sha256: sha256.to_owned() })
            }
            (None, None) => None,
            (Some(_), None) => {
                return Err("the manifest has an archive and no archive-sha256".into());
            }
            (None, Some(_)) => {
                return Err("the manifest has an archive-sha256 and no archive".into());
            }
        };
        let mut tables = Vec::new();
        for (table, body) in blocks(text, "[table]") {
            tables.push(Recorded {
                table,
                rows: number(&body, "rows")?,
                bytes: number(&body, "bytes")?,
                sha256: need(&body, "sha256")?.to_owned(),
            });
        }
        let mut properties = Vec::new();
        for (name, body) in blocks(text, "[property]") {
            properties.push(Property {
                name,
                holds: need(&body, "holds")? == "yes",
                observed: need(&body, "observed")?.to_owned(),
            });
        }
        Ok(Self {
            suite: need(corpus, "suite")?.to_owned(),
            scale: need(corpus, "scale")?.to_owned(),
            provenance,
            generator: need(corpus, "generator")?.to_owned(),
            converter: need(corpus, "converter")?.to_owned(),
            hashed_by: need(corpus, "hashed-by")?.to_owned(),
            written: need(corpus, "written")?.to_owned(),
            source,
            tables,
            properties,
        })
    }

    /// Read the manifest sitting beside a corpus, when there is one.
    ///
    /// `Ok(None)` for a corpus that predates manifests, which is every corpus on every machine
    /// today. That is not an error and it is not silence either: the caller says so, because a run
    /// over a corpus nobody wrote anything down about is a run whose header cannot claim much.
    ///
    /// # Errors
    ///
    /// When there is a manifest and it does not read.
    pub fn beside(directory: &Path) -> Result<Option<Self>, String> {
        let path = Self::path(directory);
        match std::fs::read_to_string(&path) {
            Ok(text) => Self::read(&text).map(Some).map_err(|e| format!("{}: {e}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("{} is not readable: {e}", path.display())),
        }
    }
}

/// The header the manifest carries, so that somebody who opens one knows what it is for.
const PREAMBLE: &str = "\
# What this corpus is, written down at the time it was made.
#
# Written by `rudb-bench generate <suite>` and read by every run over these files. The Parquet
# files beside this one are a corpus in the sense that the queries will run against them; this file
# is what makes a number measured over them mean something later.
#
# `provenance` is which generator wrote the data. `dbgen` is the official one from the TPC-H tools
# distribution and is the source of truth. `duckdb-tpch` is DuckDB's extension, which is permitted
# up to SF10 and is recorded as a different provenance rather than treated as the same thing,
# because it is very probably identical data and a corpus does not get to be very probably
# anything. `imdb-archive` is JOB's IMDb snapshot, which has no generator: it was unpacked from the
# `archive` named here, and `archive-sha256` is the digest of that download, so two copies fetched
# from two mirrors can be told to be the same one. A corpus like that has one size and its `scale`
# says `fixed`.
#
# `rows` is counted in the file that was written and is not the count the specification predicts.
# The two agreeing is worth knowing and the two disagreeing is worth knowing sooner.
#
# `sha256` is the digest `shasum -a 256 <file>` prints, so checking a manifest needs no harness.
#
# The `[property]` blocks are the physical facts spec/bench/tpc-h/02-the-data.md section 2.4 asks
# for, measured here rather than assumed elsewhere. spec/graph/ builds a 106 MB forward link out of
# lineitem being in l_orderkey order and picks one key map over another out of o_orderkey being
# about a quarter dense. A corpus that lost one of those would not fail; it would quietly produce a
# link ten times the size and a number that reads like a regression in the engine.
#
# A property that does not hold is written down as not holding rather than refusing the corpus. It
# is still a usable TPC-H corpus and is only a problem for the layer that assumed otherwise. Not
# knowing is the thing that is not acceptable.
";

/// What [`Manifest::scale`] says for a suite that has one corpus of one size.
pub const FIXED: &str = "fixed";

/// Every `[head]` block in the file, as whatever followed the head on its own line and the lines
/// under it.
///
/// Line by line rather than by splitting on the head, because the preamble at the top of the file
/// explains what a `[property]` block is and a split would have found that sentence and tried to
/// read a property out of it. Every comment line in this file is prose somebody will edit, so the
/// parser is the thing that has to be careful.
fn blocks(text: &str, head: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut open = false;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix(head) {
            out.push((rest.trim().to_owned(), String::new()));
            open = true;
        } else if line.starts_with('[') {
            open = false;
        } else if open {
            if let Some(last) = out.last_mut() {
                last.1.push_str(line);
                last.1.push('\n');
            }
        }
    }
    out
}

/// The value of the first line with this key.
fn value<'a>(block: &'a str, key: &str) -> Option<&'a str> {
    block.lines().find_map(|line| line.strip_prefix(key)?.strip_prefix(' ').map(str::trim))
}

/// The same, and a refusal naming the key when it is not there.
fn need<'a>(block: &'a str, key: &str) -> Result<&'a str, String> {
    value(block, key).ok_or_else(|| format!("the manifest has no {key}"))
}

/// The same, as a number.
fn number(block: &str, key: &str) -> Result<u64, String> {
    need(block, key)?.parse().map_err(|e| format!("the manifest's {key} is not a number: {e}"))
}

/// What a generation is about to do, so it can be printed before it is done.
///
/// Printed rather than described, per `spec/bench/tpc-h/02-the-data.md` section 2.7: a command that
/// is about to write seventy gigabytes says so and says where, before it writes any of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// Where the files go.
    pub directory: PathBuf,
    /// Which generator will be used.
    pub provenance: Provenance,
    /// The tables, with the row count the specification predicts for each.
    pub tables: Vec<(String, u64)>,
    /// A rough number of bytes, which is a guess and says so.
    pub bytes: u64,
    /// Whether there is already a corpus there.
    pub exists: bool,
    /// Whether this scale is past what the DuckDB extension is allowed to generate.
    ///
    /// The plan carries it so that somebody who asks for SF100 is told by the plan, which is the
    /// command they run first, rather than by the generation they ran afterwards and waited on.
    pub beyond: bool,
    /// The archive that will be downloaded, for a corpus that is unpacked rather than generated.
    ///
    /// On the plan because a download is the one part of a generation that costs somebody else's
    /// bandwidth and an hour of their afternoon, and the plan is where they get to decide against it.
    pub archive: Option<&'static Archive>,
}

impl Plan {
    /// What it is about to do, as a paragraph.
    #[must_use]
    pub fn say(&self) -> String {
        let mut out = String::new();
        let _ =
            writeln!(out, "writing {} tables to {}", self.tables.len(), self.directory.display());
        let _ = writeln!(out, "generator {}", self.provenance.label());
        if let Some(archive) = self.archive {
            let _ = writeln!(
                out,
                "downloading {} from {}, about {}, unless it is already there, and unpacking about \
                 {} of CSV that is deleted once the Parquet is written",
                archive.file,
                archive.mirrors.first().copied().unwrap_or_default(),
                crate::memory::bytes(archive.bytes),
                crate::memory::bytes(archive.unpacked)
            );
        }
        // As wide as the longest name, because JOB's are twice the length of TPC-H's and a column
        // that is only straight for one suite is not a column.
        let width = self.tables.iter().map(|(name, _)| name.len()).max().unwrap_or(0).max(10);
        for (name, rows) in &self.tables {
            let _ = writeln!(out, "  {name:<width$}  {rows} rows");
        }
        // Rough on purpose and said so. The real figure goes in the manifest once the files exist,
        // and a guess printed to the byte would be read as the real one.
        let _ = writeln!(out, "about {} of Parquet, roughly", crate::memory::bytes(self.bytes));
        if self.exists {
            out.push_str("there is already a corpus there, and it will be written over\n");
        }
        if self.beyond {
            out.push_str(
                "this scale is above SF10, which is as far as DuckDB's tpch extension is \
                 permitted to go, so --yes will refuse it. Above SF10 the corpus has to come from \
                 the official dbgen and this harness does not drive it yet\n",
            );
        }
        out
    }
}

/// A corpus that is a download rather than a generator's output.
///
/// There is one today, JOB's IMDb snapshot, and it is a struct rather than a handful of constants
/// so that the plan, the download and the check at the end all read the same description of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Archive {
    /// What the archive is called, which is also what it is saved as beside the corpus.
    pub file: &'static str,
    /// Where it can be fetched from, in the order they are tried.
    pub mirrors: &'static [&'static str],
    /// Roughly how big the download is, for the plan.
    pub bytes: u64,
    /// Roughly how much CSV comes out of it, for the plan, because that is the figure that decides
    /// whether the disk has room and it is three times the download.
    pub unpacked: u64,
    /// The schema the CSVs are loaded into, which is the suite's own so that the Parquet columns
    /// have the types every engine is later handed.
    pub schema: &'static str,
    /// The tables with the row count each one has in the published snapshot, in schema order.
    pub tables: &'static [(&'static str, u64)],
}

/// JOB's IMDb snapshot.
///
/// CedarDB's mirror first, because the original at CWI is the address in the JOB paper and is slow
/// enough that a download from it takes most of a day. Both serve the same `imdb.tgz`, and the
/// digest in the manifest is what says so for any particular copy.
///
/// The per table counts are the ones every JOB paper and loader since 2015 has reported for this
/// snapshot. They add up to the 74,190,187 the suite declares, which a test checks, and the plan
/// prints them so that a table that came out short is visible against the number it should have
/// been.
pub const IMDB: Archive = Archive {
    file: "imdb.tgz",
    mirrors: &["https://bonsai.cedardb.com/job/imdb.tgz", "http://event.cwi.nl/da/job/imdb.tgz"],
    bytes: 1_200_000_000,
    unpacked: 3_700_000_000,
    schema: JOB_SCHEMA,
    tables: &[
        ("aka_name", 901_343),
        ("aka_title", 361_472),
        ("cast_info", 36_244_344),
        ("char_name", 3_140_339),
        ("comp_cast_type", 4),
        ("company_name", 234_997),
        ("company_type", 4),
        ("complete_cast", 135_086),
        ("info_type", 113),
        ("keyword", 134_170),
        ("kind_type", 7),
        ("link_type", 18),
        ("movie_companies", 2_609_129),
        ("movie_info", 14_835_720),
        ("movie_info_idx", 1_380_035),
        ("movie_keyword", 4_523_930),
        ("movie_link", 29_997),
        ("name", 4_167_491),
        ("person_info", 2_963_664),
        ("role_type", 12),
        ("title", 2_528_312),
    ],
};

/// The archive a suite's corpus is unpacked from, when it is one of those.
#[must_use]
pub fn archive(suite: &Suite) -> Option<&'static Archive> {
    (suite.name == "job").then_some(&IMDB)
}

/// The refusal for a scale factor given to a suite that has one corpus.
fn no_scale(suite: &Suite) -> String {
    format!(
        "the {} suite is one corpus of one size, so there is no scale factor to generate it at. \
         Run `rudb-bench generate {}` with no --scale",
        suite.name, suite.name
    )
}

/// Work out what generating this suite would do.
///
/// `scale` is which factor of a generated suite to write, and it has to be `None` for a suite that
/// is unpacked from an archive, because there is only one of those corpora.
///
/// # Errors
///
/// When the suite does not have a generator or an archive, does not have this scale, or was given a
/// scale it cannot have.
pub fn plan(suite: &'static Suite, scale: Option<&'static Scale>) -> Result<Plan, String> {
    if let Some(archive) = archive(suite) {
        if scale.is_some() {
            return Err(no_scale(suite));
        }
        let directory = root().join(suite.directory(None));
        let tables: Vec<(String, u64)> =
            archive.tables.iter().map(|(table, rows)| ((*table).to_owned(), *rows)).collect();
        let bytes = tables.iter().map(|(_, rows)| rows * PARQUET_BYTES_PER_ROW).sum();
        // The directory alone is not a corpus here, because a download that stopped halfway leaves
        // one behind with nothing but the partial archive in it.
        let exists =
            suite.tables.iter().any(|table| directory.join(format!("{table}.parquet")).exists());
        return Ok(Plan {
            directory,
            provenance: Provenance::ImdbArchive,
            tables,
            bytes,
            exists,
            beyond: false,
            archive: Some(archive),
        });
    }
    let Size::Generated { tables, .. } = suite.size else {
        return Err(format!(
            "the {} suite has one corpus of one size and nothing here knows how to make it, so \
             there is nothing to generate",
            suite.name
        ));
    };
    let Some(scale) = scale.or_else(|| suite.default_scale()) else {
        return Err(format!("the {} suite has no scale factor to generate at", suite.name));
    };
    if tables.is_empty() {
        return Err(format!(
            "the {} suite has no per table row counts written down yet, so a generation of it \
             cannot say what it is about to write. It needs {}",
            suite.name, suite.needs
        ));
    }
    let directory = root().join(suite.directory(Some(scale)));
    let rows: Vec<(String, u64)> =
        tables.iter().map(|table| (table.table.to_owned(), table.rows(scale))).collect();
    // Taken off the row count rather than measured, because the point of printing it is to stop
    // somebody filling a disk and for that a factor of two is plenty. The figure in the manifest is
    // the real one and is written after the files are.
    let bytes = rows.iter().map(|(_, rows)| rows * PARQUET_BYTES_PER_ROW).sum();
    Ok(Plan {
        directory,
        provenance: Provenance::DuckdbTpch,
        tables: rows,
        bytes,
        exists: root().join(suite.directory(Some(scale))).exists(),
        beyond: scale.hundredths > DUCKDB_CEILING,
        archive: None,
    })
}

/// A rough Parquet size per TPC-H row, over all eight tables.
///
/// SF1 is about 300 MB of Parquet over its roughly eight and a half million rows. This is that
/// ratio, rounded up, and it exists to keep somebody from filling a disk rather than to be right.
/// JOB uses it too, which overstates it, and an overstated disk figure is the safe direction.
const PARQUET_BYTES_PER_ROW: u64 = 40;

/// The largest scale DuckDB's extension is allowed to generate, in hundredths.
///
/// `spec/bench/tpc-h/02-the-data.md` section 2.2 permits it up to SF10 and makes `dbgen` the source
/// of truth above that, which is also where `-C` and `-S` chunking stops being optional.
const DUCKDB_CEILING: u64 = 1_000;

/// Generate a corpus and write its manifest.
///
/// Every table is written and counted, every file is hashed, and the four physical properties are
/// measured, all before anything is claimed. What comes back is the manifest that was written, so a
/// caller can print it rather than reading it back off the disk it just wrote.
///
/// For a suite that is unpacked from an archive, `scale` has to be `None` and the work is the
/// download, the unpacking and the conversion described on [`unpack`] instead of the generator.
///
/// # Errors
///
/// When DuckDB is not on the machine, when the scale is above what DuckDB's extension is permitted
/// for, when the generator or the conversion fails, or when there is no SHA-256 tool to hash the
/// result with. For an archive, also when it cannot be downloaded or unpacked, or when the rows that
/// came out of it are not the rows the suite declares.
pub fn generate(
    suite: &'static Suite,
    scale: Option<&'static Scale>,
    duckdb: &Path,
    say: &mut dyn FnMut(&str),
) -> Result<Manifest, BenchError> {
    if let Some(archive) = archive(suite) {
        if scale.is_some() {
            return Err(BenchError::new(no_scale(suite)));
        }
        return unpack(suite, archive, duckdb, say);
    }
    let Some(scale) = scale.or_else(|| suite.default_scale()) else {
        return Err(BenchError::new(format!(
            "the {} suite has no generator and no archive, so there is nothing to generate",
            suite.name
        )));
    };
    if scale.hundredths > DUCKDB_CEILING {
        return Err(BenchError::new(format!(
            "{} is above SF10, which is as far as DuckDB's tpch extension is permitted to go. \
             Above it the corpus has to come from the official dbgen with -C and -S chunking, per \
             spec/bench/tpc-h/02-the-data.md section 2.2, and this harness does not drive dbgen \
             yet",
            scale.named()
        )));
    }
    let hasher = Hasher::need()?;
    let directory = root().join(suite.directory(Some(scale)));
    std::fs::create_dir_all(&directory)
        .map_err(|e| BenchError::new(format!("cannot make {}: {e}", directory.display())))?;
    let converter = version(duckdb)?;

    // One DuckDB process generates and converts. The generator's output is a set of tables in a
    // database that is thrown away, and what survives is the Parquet, which is the form every
    // engine reads. Conversion is DuckDB's for the reason `data.rs` already gives about the smoke
    // suite: the output has nothing of DuckDB in it and the alternative is a Parquet writer this
    // project would have to prove correct before it could use it to prove anything else.
    let factor = scale.hundredths as f64 / 100.0;
    let mut sql = format!("INSTALL tpch; LOAD tpch; CALL dbgen(sf = {factor});");
    for table in suite.tables {
        let path = directory.join(format!("{table}.parquet"));
        let _ =
            write!(sql, " COPY (SELECT * FROM {table}) TO '{}' (FORMAT parquet);", path.display());
    }
    say(&format!("generating {} at {}", suite.name, scale.named()));
    ask(duckdb, &sql)?;

    let tables = record(suite, &directory, duckdb, &hasher, say)?;

    say("checking the physical properties");
    let properties = measure(suite, &directory, duckdb)?;
    for property in &properties {
        say(&format!(
            "  {:<22}  {}  {}",
            property.name,
            if property.holds { "holds" } else { "does not hold" },
            property.observed
        ));
    }

    let manifest = Manifest {
        suite: suite.name.to_owned(),
        scale: scale.label.to_owned(),
        provenance: Provenance::DuckdbTpch,
        generator: format!("duckdb tpch extension, CALL dbgen(sf = {factor})"),
        converter,
        hashed_by: hasher.said.clone(),
        written: crate::regress::today(),
        source: None,
        tables,
        properties,
    };
    keep(&manifest, &directory, say)?;
    Ok(manifest)
}

/// Count, size and hash every table a suite names, out of the Parquet files that were just written.
fn record(
    suite: &'static Suite,
    directory: &Path,
    duckdb: &Path,
    hasher: &Hasher,
    say: &mut dyn FnMut(&str),
) -> Result<Vec<Recorded>, BenchError> {
    let width = suite.tables.iter().map(|table| table.len()).max().unwrap_or(0).max(10);
    let mut tables = Vec::with_capacity(suite.tables.len());
    for table in suite.tables {
        let path = directory.join(format!("{table}.parquet"));
        let bytes = std::fs::metadata(&path)
            .map_err(|e| BenchError::new(format!("{} was not written: {e}", path.display())))?
            .len();
        // Counted out of the file rather than taken from the suite's table. The two agreeing is
        // worth knowing and the two disagreeing is worth knowing sooner.
        let counted =
            ask(duckdb, &format!("SELECT count(*) FROM read_parquet('{}')", path.display()))?;
        let rows: u64 = counted
            .trim()
            .parse()
            .map_err(|e| BenchError::new(format!("{table} did not count: {e}")))?;
        say(&format!("  {table:<width$}  {rows} rows, {}", crate::memory::bytes(bytes)));
        tables.push(Recorded {
            table: (*table).to_owned(),
            rows,
            bytes,
            sha256: hasher.of(&path)?,
        });
    }
    Ok(tables)
}

/// Write a manifest beside the corpus it describes.
fn keep(
    manifest: &Manifest,
    directory: &Path,
    say: &mut dyn FnMut(&str),
) -> Result<(), BenchError> {
    let at = Manifest::path(directory);
    std::fs::write(&at, manifest.write())
        .map_err(|e| BenchError::new(format!("cannot write {}: {e}", at.display())))?;
    say(&format!("wrote {}", at.display()));
    Ok(())
}

/// Download an archive, load its CSVs, and write the corpus and its manifest.
///
/// In order: fetch the archive unless a finished copy is already beside the corpus, hash it, unpack
/// it with `tar`, load every CSV into the suite's own schema in a DuckDB database that is thrown
/// away, write each table to Parquet in the order its rows were loaded, and then count, size and
/// hash the Parquet the same way a generated corpus is.
///
/// The archive is kept afterwards and the CSVs and the database are not. The archive is the slow
/// part to get back and the other two come out of it in minutes, and the CSVs are three times the
/// archive's size on a disk that also has to hold the Parquet.
///
/// The row total is checked against the suite's declared size before the manifest is written. A
/// truncated archive unpacks and loads without complaint, and what it produces is a corpus with a
/// short table and a manifest that describes it faithfully, which is worse than no corpus at all
/// because every number measured over it would look fine.
fn unpack(
    suite: &'static Suite,
    archive: &'static Archive,
    duckdb: &Path,
    say: &mut dyn FnMut(&str),
) -> Result<Manifest, BenchError> {
    let hasher = Hasher::need()?;
    for tool in ["curl", "tar"] {
        if !which(tool) {
            return Err(BenchError::new(format!(
                "there is no {tool} on this machine, and the {} corpus is a download that needs \
                 curl to fetch it and tar to unpack it",
                suite.name
            )));
        }
    }
    let directory = root().join(suite.directory(None));
    std::fs::create_dir_all(&directory)
        .map_err(|e| BenchError::new(format!("cannot make {}: {e}", directory.display())))?;
    let converter = version(duckdb)?;

    let (tarball, from) = fetch(archive, &directory, say)?;
    say(&format!("hashing {}", tarball.display()));
    let digest = hasher.of(&tarball)?;
    say(&format!("  sha256 {digest}"));

    let loaded = load(suite, archive, &tarball, &directory, duckdb, say);
    say("deleting the CSVs and the load database");
    tidy(&directory);
    loaded?;

    let tables = record(suite, &directory, duckdb, &hasher, say)?;
    let total: u64 = tables.iter().map(|table| table.rows).sum();
    let declared = suite.rows(None).unwrap_or(total);
    if total != declared {
        let short: Vec<String> = tables
            .iter()
            .filter_map(|table| {
                let expected = archive.tables.iter().find(|(name, _)| *name == table.table)?.1;
                (expected != table.rows).then(|| {
                    format!("{} has {} and should have {expected}", table.table, table.rows)
                })
            })
            .collect();
        return Err(BenchError::new(format!(
            "the {} corpus came out at {total} rows and the suite is {declared}, so the archive \
             at {} is not the snapshot it should be{}. No manifest was written. Delete the archive \
             and generate it again",
            suite.name,
            tarball.display(),
            if short.is_empty() { String::new() } else { format!(": {}", short.join(", ")) }
        )));
    }

    let manifest = Manifest {
        suite: suite.name.to_owned(),
        scale: FIXED.to_owned(),
        provenance: Provenance::ImdbArchive,
        generator: format!("{}, unpacked with tar and loaded into {}", archive.file, suite.name),
        converter,
        hashed_by: hasher.said.clone(),
        written: crate::regress::today(),
        source: Some(Source { archive: from, sha256: digest }),
        tables,
        properties: measure(suite, &directory, duckdb)?,
    };
    keep(&manifest, &directory, say)?;
    Ok(manifest)
}

/// Where the CSVs are unpacked, under the corpus directory.
///
/// Beside the corpus rather than in the system's temporary directory, because four gigabytes of CSV
/// is more than a small /tmp holds and the corpus directory is the one place already known to have
/// room for something this size.
const UNPACKED: &str = "csv";

/// The DuckDB database the CSVs are loaded into on their way to Parquet, which nothing keeps.
const LOADING: &str = "load.duckdb";

/// Throw away the CSVs and the load database, and whatever write-ahead log it left.
fn tidy(directory: &Path) {
    let _ = std::fs::remove_dir_all(directory.join(UNPACKED));
    let _ = std::fs::remove_file(directory.join(LOADING));
    let _ = std::fs::remove_file(directory.join(format!("{LOADING}.wal")));
}

/// Unpack the archive and turn its CSVs into Parquet, leaving the cleaning up to the caller.
fn load(
    suite: &'static Suite,
    archive: &'static Archive,
    tarball: &Path,
    directory: &Path,
    duckdb: &Path,
    say: &mut dyn FnMut(&str),
) -> Result<(), BenchError> {
    // Anything left by a run that stopped halfway is thrown out first. A stale table in the load
    // database would make every COPY after it land on top of rows that are already there.
    tidy(directory);
    let csv = directory.join(UNPACKED);
    std::fs::create_dir_all(&csv)
        .map_err(|e| BenchError::new(format!("cannot make {}: {e}", csv.display())))?;

    say(&format!("unpacking {} into {}", archive.file, csv.display()));
    let mut command = Command::new("tar");
    command.arg("-xzf").arg(tarball).arg("-C").arg(&csv);
    output(&mut command, "tar")?;
    let found = csvs(&csv, suite.tables)?;

    say(&format!("loading {} tables and writing them to Parquet", suite.tables.len()));
    let sql = load_sql(archive.schema, suite.tables, &found, directory);
    let mut command = Command::new(duckdb);
    command.arg("-batch").arg(directory.join(LOADING)).arg("-c").arg(&sql);
    output(&mut command, "duckdb")?;
    Ok(())
}

/// The one script that loads JOB's CSVs and writes them back out as Parquet.
///
/// The schema first, as it is, so the columns have the types the JOB paper gave them rather than
/// the ones DuckDB would sniff out of the text. Then a `COPY FROM` per table with the options the
/// IMDb CSVs were written with: no header, a backslash escape, a double quote, and an empty field
/// for NULL. Then a `COPY TO` per table, ordered by `rowid`, which is the order the rows were loaded
/// in and so the order the CSV had them. JOB has no physical order of its own to preserve the way
/// TPC-H does, but an engine that reads a file in the order it was published is running over the
/// same corpus as every other loader of it, and one that got whatever order a parallel writer
/// happened to produce is not.
#[must_use]
pub fn load_sql(schema: &str, tables: &[&str], csv: &Path, directory: &Path) -> String {
    let mut sql = String::from(schema.trim());
    for table in tables {
        let from = csv.join(format!("{table}.csv"));
        let _ = write!(
            sql,
            "\nCOPY {table} FROM '{}' (FORMAT csv, HEADER false, ESCAPE '\\', QUOTE '\"', NULL '');",
            from.display()
        );
    }
    for table in tables {
        let to = directory.join(format!("{table}.parquet"));
        let _ = write!(
            sql,
            "\nCOPY (SELECT * FROM {table} ORDER BY rowid) TO '{}' (FORMAT parquet);",
            to.display()
        );
    }
    sql
}

/// The directory an archive's CSVs landed in.
///
/// The directory it was unpacked into, or one level below it, because an archive is free to put its
/// files under a folder of its own and which one this snapshot does is not something the harness
/// should need to know. Every table has to be in the same place, and the refusal names the ones
/// that were not found anywhere.
fn csvs(unpacked: &Path, tables: &[&str]) -> Result<PathBuf, BenchError> {
    let complete = |at: &Path| tables.iter().all(|table| at.join(format!("{table}.csv")).is_file());
    if complete(unpacked) {
        return Ok(unpacked.to_path_buf());
    }
    if let Ok(entries) = std::fs::read_dir(unpacked) {
        for entry in entries.filter_map(Result::ok) {
            let at = entry.path();
            if at.is_dir() && complete(&at) {
                return Ok(at);
            }
        }
    }
    let missing: Vec<&str> = tables
        .iter()
        .copied()
        .filter(|table| !unpacked.join(format!("{table}.csv")).is_file())
        .collect();
    Err(BenchError::new(format!(
        "the archive unpacked into {} without {} of the CSVs the suite needs, {}",
        unpacked.display(),
        missing.len(),
        missing.join(", ")
    )))
}

/// Fetch an archive into the corpus directory, or find the one fetched before.
///
/// `curl -C -` into a `.part` file, renamed once the transfer finishes. The rename is what makes a
/// finished archive tell itself apart from a stopped one, which `curl` alone cannot: asked to resume
/// a file that is already whole, it asks the server for the bytes after the end and some servers
/// answer that with an error. So a file with the archive's own name is complete and is used as it
/// is, and a `.part` file is where a stopped download picks up from.
///
/// The mirrors are tried in order, and the second picks up from wherever the first left off. That
/// is only right if they serve the same bytes, which is the claim the digest in the manifest is
/// there to check.
///
/// Where it came from is written beside it, so that a second generation from a copy that is already
/// here can still say which mirror it was.
fn fetch(
    archive: &'static Archive,
    directory: &Path,
    say: &mut dyn FnMut(&str),
) -> Result<(PathBuf, String), BenchError> {
    let at = directory.join(archive.file);
    let origin = directory.join(format!("{}.from", archive.file));
    if at.is_file() {
        let from = std::fs::read_to_string(&origin)
            .map(|from| from.trim().to_owned())
            .ok()
            .filter(|from| !from.is_empty())
            .unwrap_or_else(|| at.display().to_string());
        say(&format!("using the {} already at {}", archive.file, at.display()));
        return Ok((at, from));
    }
    let part = directory.join(format!("{}.part", archive.file));
    let mut failures = Vec::new();
    for mirror in archive.mirrors {
        say(&format!("downloading {mirror}"));
        // Its own terminal rather than a pipe, so the progress bar reaches the person waiting on a
        // gigabyte. `output` would hold every byte of it until the transfer was over.
        let status = Command::new("curl")
            .args(["--fail", "--location", "--continue-at", "-", "--retry", "3", "--progress-bar"])
            .arg("--output")
            .arg(&part)
            .arg(mirror)
            .stdin(std::process::Stdio::null())
            .status();
        match status {
            Ok(status) if status.success() => {
                std::fs::rename(&part, &at).map_err(|e| {
                    BenchError::new(format!("cannot move {} into place: {e}", part.display()))
                })?;
                let _ = std::fs::write(&origin, format!("{mirror}\n"));
                return Ok((at, (*mirror).to_owned()));
            }
            Ok(status) => failures.push(format!("{mirror} ({status})")),
            Err(e) => failures.push(format!("{mirror} ({e})")),
        }
    }
    Err(BenchError::new(format!(
        "could not download {} from any mirror: {}. What arrived is kept at {} and the next run \
         picks up from there",
        archive.file,
        failures.join(", "),
        part.display()
    )))
}

/// The four physical properties of `spec/bench/tpc-h/02-the-data.md` section 2.4, measured.
///
/// Only for TPC-H, because they are facts about TPC-H's data and nothing here would know what to
/// ask of another suite. A suite with no checks gets an empty list, which is honest.
fn measure(
    suite: &'static Suite,
    directory: &Path,
    duckdb: &Path,
) -> Result<Vec<Property>, BenchError> {
    if suite.name != "tpch" {
        return Ok(Vec::new());
    }
    let at = |table: &str| directory.join(format!("{table}.parquet")).display().to_string();
    let mut out = Vec::new();

    // `lineitem` in `l_orderkey` order is what makes the forward link monotone and 106 MB instead
    // of 2.1 GB. `OVER ()` with no ordering walks the file in the order the file has, which is the
    // order this is asking about.
    let out_of_order = one(
        duckdb,
        &format!(
            "SELECT count(*) FROM (SELECT l_orderkey, lag(l_orderkey) OVER () AS before FROM \
             read_parquet('{}')) WHERE before IS NOT NULL AND l_orderkey < before",
            at("lineitem")
        ),
    )?;
    out.push(Property {
        name: "lineitem-ordered".to_owned(),
        holds: out_of_order == "0",
        observed: format!("{out_of_order} rows out of order in l_orderkey"),
    });

    // Sparse on purpose, so that implementations cannot assume density, and about a quarter dense
    // in practice. That quarter is why `orders` gets the bitmap key map rather than the sorted one.
    let density = one(
        duckdb,
        &format!(
            "SELECT round(count(*) / (max(o_orderkey) - min(o_orderkey) + 1), 4) FROM \
             read_parquet('{}')",
            at("orders")
        ),
    )?;
    let fraction: f64 = density.trim().parse().unwrap_or(1.0);
    out.push(Property {
        name: "orderkey-sparse".to_owned(),
        holds: fraction < 0.5,
        observed: format!("o_orderkey is {density} dense over its range"),
    });

    // Dense and ascending, which is what makes an identity key map twenty four bytes.
    for (table, key) in
        [("customer", "c_custkey"), ("part", "p_partkey"), ("supplier", "s_suppkey")]
    {
        let gaps = one(
            duckdb,
            &format!(
                "SELECT max({key}) - min({key}) + 1 - count(*) FROM read_parquet('{}')",
                at(table)
            ),
        )?;
        out.push(Property {
            name: format!("{key}-dense"),
            holds: gaps.trim() == "0",
            observed: format!("{gaps} keys missing from the range of {key}"),
        });
    }

    // Ordered by `ps_partkey` with a constant four suppliers per part, which is the second free
    // link: monotone and with a degree that needs no offsets array.
    let unordered = one(
        duckdb,
        &format!(
            "SELECT count(*) FROM (SELECT ps_partkey, lag(ps_partkey) OVER () AS before FROM \
             read_parquet('{}')) WHERE before IS NOT NULL AND ps_partkey < before",
            at("partsupp")
        ),
    )?;
    let degrees = one(
        duckdb,
        &format!(
            "SELECT min(n) || ' to ' || max(n) FROM (SELECT count(*) AS n FROM read_parquet('{}') \
             GROUP BY ps_partkey)",
            at("partsupp")
        ),
    )?;
    out.push(Property {
        name: "partsupp-ordered".to_owned(),
        holds: unordered.trim() == "0" && degrees.trim() == "4 to 4",
        observed: format!("{unordered} rows out of order, {degrees} suppliers per part"),
    });

    Ok(out)
}

/// Run a statement and give back what it said, trimmed.
fn ask(duckdb: &Path, sql: &str) -> Result<String, BenchError> {
    let mut command = Command::new(duckdb);
    command.arg("-batch").arg("-csv").arg("-noheader").arg("-c").arg(sql);
    let said = output(&mut command, "duckdb")?;
    Ok(String::from_utf8_lossy(&said).trim().to_owned())
}

/// The same, for a query whose answer is one value.
fn one(duckdb: &Path, sql: &str) -> Result<String, BenchError> {
    let said = ask(duckdb, sql)?;
    Ok(said.lines().next_back().unwrap_or_default().trim().trim_matches('"').to_owned())
}

/// What DuckDB calls itself, which is what the manifest records as the converter.
fn version(duckdb: &Path) -> Result<String, BenchError> {
    let mut command = Command::new(duckdb);
    command.arg("--version");
    let said = output(&mut command, "duckdb")?;
    Ok(format!("duckdb {}", String::from_utf8_lossy(&said).trim()))
}

/// Whatever SHA-256 tool this machine has.
///
/// Two spellings because there are two: `shasum -a 256` is what a Mac has and `sha256sum` is what
/// a Linux has. Which one ran is recorded, so a reader who wants to check a digest runs the same
/// command rather than guessing which convention the hex came from. Both print the digest first
/// and the filename after, so one parser does.
#[derive(Debug, Clone)]
struct Hasher {
    binary: String,
    args: Vec<String>,
    /// The command as the manifest writes it.
    said: String,
}

/// Whether a bare command name resolves to something on `PATH`.
fn which(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else { return false };
    std::env::split_paths(&path).any(|dir| dir.join(name).is_file())
}

impl Hasher {
    /// The same, or the refusal a generation gives when there is neither.
    fn need() -> Result<Self, BenchError> {
        Self::find().ok_or_else(|| {
            BenchError::new(
                "there is no shasum or sha256sum on this machine, and a manifest without digests \
                 is a manifest that cannot say two corpora are the same one",
            )
        })
    }

    /// The first of the two that is on this machine.
    fn find() -> Option<Self> {
        for (binary, args) in [("shasum", vec!["-a", "256"]), ("sha256sum", vec![])] {
            if which(binary) {
                return Some(Self {
                    binary: binary.to_owned(),
                    args: args.iter().map(|a| (*a).to_owned()).collect(),
                    said: if args.is_empty() {
                        binary.to_owned()
                    } else {
                        format!("{binary} {}", args.join(" "))
                    },
                });
            }
        }
        None
    }

    /// The digest of one file, lowercase hex.
    fn of(&self, path: &Path) -> Result<String, BenchError> {
        let mut command = Command::new(&self.binary);
        command.args(&self.args).arg(path);
        let said = output(&mut command, &self.binary)?;
        let text = String::from_utf8_lossy(&said);
        let digest = text.split_whitespace().next().unwrap_or_default();
        if digest.len() != 64 || !digest.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(BenchError::new(format!(
                "{} said `{}` about {}, which is not a sha256",
                self.said,
                text.trim(),
                path.display()
            )));
        }
        Ok(digest.to_ascii_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        FIXED, IMDB, Manifest, Property, Provenance, Recorded, Source, csvs, load_sql, plan,
    };
    use std::path::Path;

    fn one() -> Manifest {
        Manifest {
            suite: "tpch".to_owned(),
            scale: "1".to_owned(),
            provenance: Provenance::DuckdbTpch,
            generator: "duckdb tpch extension, CALL dbgen(sf = 1)".to_owned(),
            converter: "duckdb v1.5.5 (Variegata) d8cdaa33fd".to_owned(),
            hashed_by: "shasum -a 256".to_owned(),
            written: "2026-09-18".to_owned(),
            source: None,
            tables: vec![
                Recorded {
                    table: "lineitem".to_owned(),
                    rows: 6_001_215,
                    bytes: 233_442_121,
                    sha256: "a".repeat(64),
                },
                Recorded {
                    table: "orders".to_owned(),
                    rows: 1_500_000,
                    bytes: 56_784_112,
                    sha256: "b".repeat(64),
                },
            ],
            properties: vec![
                Property {
                    name: "lineitem-ordered".to_owned(),
                    holds: true,
                    observed: "0 rows out of order in l_orderkey".to_owned(),
                },
                Property {
                    name: "orderkey-sparse".to_owned(),
                    holds: false,
                    observed: "o_orderkey is 1.0 dense over its range".to_owned(),
                },
            ],
        }
    }

    /// A JOB corpus, which was unpacked from an archive rather than generated.
    fn unpacked() -> Manifest {
        Manifest {
            suite: "job".to_owned(),
            scale: FIXED.to_owned(),
            provenance: Provenance::ImdbArchive,
            generator: "imdb.tgz, unpacked with tar and loaded into job".to_owned(),
            converter: "duckdb v1.5.5 (Variegata) d8cdaa33fd".to_owned(),
            hashed_by: "sha256sum".to_owned(),
            written: "2026-09-24".to_owned(),
            source: Some(Source {
                archive: "https://bonsai.cedardb.com/job/imdb.tgz".to_owned(),
                sha256: "d".repeat(64),
            }),
            tables: vec![Recorded {
                table: "title".to_owned(),
                rows: 2_528_312,
                bytes: 91_000_000,
                sha256: "e".repeat(64),
            }],
            properties: Vec::new(),
        }
    }

    /// The file is the only copy of every fact in it, so a round trip that lost one would lose it
    /// permanently and would look exactly like a corpus nobody recorded anything about.
    #[test]
    fn a_manifest_read_back_is_the_manifest_that_was_written() {
        let written = one().write();
        let back = Manifest::read(&written).expect("it was just written");
        assert_eq!(back, one());

        // The archive and its digest are the only record of what an unpacked corpus was made out
        // of, so they come back too, and a generated corpus comes back with none rather than an
        // empty one.
        let written = unpacked().write();
        assert!(written.contains("archive-sha256 dddd"), "{written}");
        let back = Manifest::read(&written).expect("it was just written");
        assert_eq!(back, unpacked());
        assert!(!one().write().contains("\narchive "), "a generated corpus has no archive");
    }

    /// Half of a source is a download nobody can check or a digest of nothing, so it is refused
    /// the way any other missing field is.
    #[test]
    fn an_archive_without_its_digest_is_refused() {
        let written = unpacked().write();
        let without = written.replace(&format!("archive-sha256 {}\n", "d".repeat(64)), "");
        let why = Manifest::read(&without).expect_err("there is no digest");
        assert!(why.contains("archive-sha256"), "{why}");
    }

    /// JOB has one size, so its header line carries no scale factor for it to be misread as.
    #[test]
    fn an_unpacked_corpus_says_where_it_came_from_without_a_scale() {
        let line = unpacked().line();
        assert!(line.starts_with("imdb-archive, corpus "), "{line}");
        assert!(!line.contains("SF"), "{line}");
        assert!(one().line().starts_with("duckdb-tpch SF1, corpus "), "{}", one().line());
    }

    /// The plan is what somebody reads before a gigabyte download, so it names every table the
    /// suite will load, in the order the schema creates them, and the archive it will fetch.
    #[test]
    fn the_job_plan_names_its_21_tables_in_schema_order() {
        let job = crate::suite::find("job").expect("job is a suite");
        let plan = plan(job, None).expect("job has an archive");
        let names: Vec<&str> = plan.tables.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(names.len(), 21);
        assert_eq!(names, crate::suite::JOB_TABLES);
        assert_eq!(plan.provenance, Provenance::ImdbArchive);
        assert_eq!(plan.archive, Some(&IMDB));
        assert!(!plan.beyond);
        let said = plan.say();
        assert!(said.contains("https://bonsai.cedardb.com/job/imdb.tgz"), "{said}");
        assert!(said.contains("movie_companies"), "{said}");
    }

    /// The per table counts the plan prints and the total the generation checks are two statements
    /// of the same snapshot, so they have to agree with each other and with the suite.
    #[test]
    fn the_imdb_counts_add_up_to_the_suite() {
        let job = crate::suite::find("job").expect("job is a suite");
        let total: u64 = IMDB.tables.iter().map(|(_, rows)| rows).sum();
        assert_eq!(Some(total), job.rows(None));
        assert_eq!(total, 74_190_187);
        assert_eq!(IMDB.mirrors.len(), 2);
        assert!(IMDB.mirrors[0].contains("cedardb"), "the fast mirror goes first");
    }

    /// JOB has no scale factor, and a --scale that reached this far is refused rather than ignored.
    #[test]
    fn job_refuses_a_scale_and_tpch_still_takes_one() {
        let job = crate::suite::find("job").expect("job is a suite");
        let tpch = crate::suite::find("tpch").expect("tpch is a suite");
        let one = tpch.scale("1").expect("tpch has SF1");
        let why = plan(job, Some(one)).expect_err("job has one size");
        assert!(why.contains("no scale factor"), "{why}");
        let tpch = plan(tpch, Some(one)).expect("tpch has a generator");
        assert_eq!(tpch.provenance, Provenance::DuckdbTpch);
        assert_eq!(tpch.archive, None);
        let clickbench = crate::suite::find("clickbench").expect("clickbench is a suite");
        assert!(plan(clickbench, None).is_err(), "clickbench is neither generated nor unpacked");
    }

    /// The IMDb CSVs have no header, escape with a backslash, and write NULL as an empty field. Get
    /// any of those wrong and the load either fails halfway through `movie_info` or, worse,
    /// succeeds with the wrong values.
    #[test]
    fn the_load_script_reads_the_csvs_the_way_they_were_written() {
        let sql = load_sql(
            IMDB.schema,
            crate::suite::JOB_TABLES,
            Path::new("/data/job/csv"),
            Path::new("/data/job"),
        );
        assert!(sql.starts_with("CREATE TABLE aka_name"), "the schema comes first");
        assert!(
            sql.contains(
                "COPY cast_info FROM '/data/job/csv/cast_info.csv' (FORMAT csv, HEADER false, \
                 ESCAPE '\\', QUOTE '\"', NULL '');"
            ),
            "{sql}"
        );
        assert!(
            sql.contains(
                "COPY (SELECT * FROM title ORDER BY rowid) TO '/data/job/title.parquet' (FORMAT \
                 parquet);"
            ),
            "{sql}"
        );
        assert_eq!(sql.matches(" FROM '/data/job/csv/").count(), 21);
        assert_eq!(sql.matches("ORDER BY rowid").count(), 21);
        // Every table is loaded before any is written, so no Parquet file is a table half loaded.
        let last_load = sql.rfind(".csv'").expect("loads");
        let first_write = sql.find(".parquet'").expect("writes");
        assert!(last_load < first_write);
    }

    /// Whether the archive keeps its CSVs at its top or under a folder is not something the loader
    /// should have to know, and a missing table is named rather than found by DuckDB halfway in.
    #[test]
    fn the_csvs_are_found_at_the_top_of_the_archive_or_one_folder_down() {
        let top = std::env::temp_dir().join(format!("rudb-bench-csvs-{}", std::process::id()));
        let nested = top.join("imdb");
        std::fs::create_dir_all(&nested).expect("temp dir");
        let tables = ["title", "name"];
        for table in tables {
            std::fs::write(nested.join(format!("{table}.csv")), b"1\n").expect("write csv");
        }
        assert_eq!(csvs(&top, &tables).expect("one folder down"), nested);
        std::fs::remove_file(nested.join("name.csv")).expect("remove one");
        let why = csvs(&top, &tables).expect_err("name is missing").to_string();
        assert!(why.ends_with(", name"), "{why}");
        std::fs::remove_dir_all(&top).expect("clean up");
    }

    /// A property that did not hold is the case this file exists for, so it survives the round trip
    /// as not holding rather than as absent.
    #[test]
    fn a_property_that_did_not_hold_reads_back_as_not_holding() {
        let back = Manifest::read(&one().write()).expect("it was just written");
        let broken = back.broken();
        assert_eq!(broken.len(), 1);
        assert_eq!(broken[0].name, "orderkey-sparse");
        assert!(broken[0].observed.contains("1.0 dense"), "{:?}", broken[0]);
    }

    /// Two corpora that are the same files print the same string and two that are not do not, which
    /// is the whole of what a report header needs out of this.
    #[test]
    fn the_digest_follows_the_files_and_not_anything_else() {
        let first = one().digest();
        let mut other = one();
        other.written = "1999-01-01".to_owned();
        assert_eq!(other.digest(), first);
        other.tables[1].sha256 = "c".repeat(64);
        assert_ne!(other.digest(), first);
    }

    /// Every field here is the answer to a question nobody can answer another way once the files
    /// exist, so a missing one is a refusal rather than a default.
    #[test]
    fn a_manifest_missing_a_field_is_refused_rather_than_filled_in() {
        let written = one().write();
        let without = written.replace("provenance duckdb-tpch\n", "");
        let why = Manifest::read(&without).expect_err("there is no provenance");
        assert!(why.contains("provenance"), "{why}");

        let wrong = written.replace("duckdb-tpch", "somebody's script");
        let why = Manifest::read(&wrong).expect_err("that is not a provenance");
        assert!(why.contains("somebody's script"), "{why}");
    }

    /// The two are very probably the same data, and a corpus does not get to be very probably
    /// anything, so a result over the DuckDB one carries a sentence and a result over dbgen does
    /// not.
    #[test]
    fn only_the_unofficial_generator_puts_a_note_on_a_result() {
        assert!(Provenance::Dbgen.note().is_empty());
        assert!(Provenance::DuckdbTpch.note().contains("official dbgen"));
        assert_eq!(Provenance::parse("dbgen"), Some(Provenance::Dbgen));
        assert_eq!(Provenance::parse("duckdb-tpch"), Some(Provenance::DuckdbTpch));
        assert_eq!(Provenance::parse("imdb-archive"), Some(Provenance::ImdbArchive));
        assert!(Provenance::ImdbArchive.note().is_empty(), "the archive is the JOB corpus");
        assert_eq!(Provenance::parse("dbgen2"), None);
    }

    #[test]
    fn a_corpus_with_no_manifest_beside_it_is_not_an_error() {
        let nowhere =
            std::env::temp_dir().join(format!("rudb-bench-no-manifest-{}", std::process::id()));
        assert_eq!(Manifest::beside(&nowhere), Ok(None));
    }
}
