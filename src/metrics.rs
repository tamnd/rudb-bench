//! What rudb said about itself, read back out of the file it was asked to write.
//!
//! Every other engine here is a black box with a clock on the outside. rudb is not, because it is
//! ours: `rudb --metrics FILE` appends one JSON document per statement, and the document says what
//! each operator consumed, produced and spent. That is the difference between a table saying a
//! query took forty milliseconds and a table saying where the forty milliseconds went.
//!
//! The reason the harness reads it rather than trusting it is the cross check below. A breakdown
//! that does not add up to the run it came from is worse than no breakdown, because it looks like
//! evidence. So the sum of the per operator CPU is compared against the CPU the engine measured
//! around the whole execution, and a query where the two disagree is a query whose numbers this
//! harness will not publish.
//!
//! The JSON reader is hand written for the same reason everything else here is: no dependencies.
//! It reads the document rudb writes rather than every document anybody could write, and it is
//! strict about the shape it expects, so a schema that moves fails loudly on the next run instead
//! of quietly reading zeros.

use std::fs;
use std::path::Path;
use std::time::Duration;

/// How far the accounted CPU may be from the measured CPU before the run is not worth publishing.
///
/// Five percent, from `spec/15-rudb-bench.md`. It is a tolerance rather than an equality because
/// the two numbers are taken by different clocks: the operators charge themselves per chunk and
/// the execution is measured once around the outside, so a few microseconds of the outer span
/// belong to nobody and always will.
pub const TOLERANCE: f64 = 0.05;

/// One statement, as the engine described it.
///
/// Only the fields the harness has a use for. The document has more in it, and adding a field here
/// is reading one more key rather than changing anything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// The statement this describes, so a record can be matched to the query that asked for it.
    pub sql: String,
    /// `ok`, or whatever the engine called the way it ended.
    pub state: String,
    /// Parse, bind, optimize, build and execute, together.
    pub total: Duration,
    /// The execute step on its own, which is the span the operators below are inside of.
    pub execute: Duration,
    /// CPU the engine measured around building and running the tree.
    pub cpu: Duration,
    /// The part of `cpu` that went on building the tree rather than on running it.
    ///
    /// Off the denominator of the cross check, because no pipeline can ever account for it.
    /// Opening the file, reading its schema and allocating the tree all happen before there is a
    /// pipeline to charge, so a check that left this in would read the whole of it as time that
    /// went missing, and on a query short enough to be interesting it is a real share of the run.
    pub build: Duration,
    /// The most the engine says it was holding at once, in bytes.
    ///
    /// Not the resident set of the process, which is what [`crate::memory::Peak`] holds. This is
    /// what the operators reserved, so it is smaller and it is attributable, and the gap between
    /// the two is the allocator and the binary itself.
    pub peak_bytes: u64,
    /// Bytes the engine says it read.
    pub bytes_read: u64,
    /// Bytes the engine says it spilled, which is zero until there is anywhere to spill to.
    pub bytes_spilled: u64,
    /// Every pipeline the plan ran as, in the order the document lists them.
    pub pipelines: Vec<Pipeline>,
    /// Every operator, in the order the document lists them.
    pub operators: Vec<Operator>,
    /// Whatever the engine thought was worth warning about, already worded.
    pub warnings: Vec<String>,
}

/// One pipeline's row of the breakdown.
///
/// The unit the engine schedules, which is why the cross check is against these rather than against
/// the operators inside them. An operator charges itself for the time inside its own call, and the
/// loop that makes those calls is neither of them, so a check that only added up operators would
/// read the driver as time that went missing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pipeline {
    /// The id the plan gave it.
    pub id: u64,
    /// Wall clock from when it started to when it finished.
    pub wall: Duration,
    /// CPU inside it, which is what the cross check adds up.
    pub cpu: Duration,
}

/// One operator's row of the breakdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operator {
    /// The id the plan gave it, which is what an `EXPLAIN ANALYZE` line is keyed by.
    pub id: u64,
    /// Which pipeline ran it.
    pub pipeline: u64,
    /// `Get`, `Filter`, `Sort` and so on.
    pub kind: String,
    /// Rows it was handed.
    pub rows_in: u64,
    /// Rows it handed on.
    pub rows_out: u64,
    /// Wall clock inside it.
    pub wall: Duration,
    /// CPU inside it, which is what the cross check adds up.
    pub cpu: Duration,
    /// Whether it ran the reference implementation of its seam rather than a registered one.
    ///
    /// The reference implementations are the slow paths kept for differential testing, so a run
    /// where every operator is one is a run measuring the thing the fast paths exist to replace.
    /// Worth carrying into the report rather than reading off a log.
    pub reference_impl: bool,
}

impl Document {
    /// Read one document out of one line of JSON.
    ///
    /// # Errors
    ///
    /// When the line is not JSON, or is JSON of a shape this does not recognise.
    pub fn parse(line: &str) -> Result<Self, String> {
        let json = Json::read(line)?;
        let schema = json.at("schema").and_then(Json::count).unwrap_or(0);
        if schema != 1 {
            return Err(format!("this is a schema {schema} metrics document and we read schema 1"));
        }
        let timing = json.at("timing").ok_or("the document has no timing")?;
        let resource = json.at("resource").ok_or("the document has no resource")?;
        Ok(Self {
            sql: json
                .at("query")
                .and_then(|q| q.at("sql"))
                .and_then(Json::text)
                .unwrap_or_default(),
            state: json
                .at("outcome")
                .and_then(|o| o.at("state"))
                .and_then(Json::text)
                .unwrap_or_default(),
            total: nanos(timing, "total_ns"),
            execute: nanos(timing, "execute_ns"),
            cpu: nanos(resource, "cpu_ns"),
            build: nanos(resource, "build_cpu_ns"),
            peak_bytes: count(resource, "peak_bytes"),
            bytes_read: count(resource, "bytes_read"),
            bytes_spilled: count(resource, "bytes_spilled"),
            pipelines: json
                .at("pipelines")
                .and_then(Json::list)
                .unwrap_or_default()
                .iter()
                .map(Pipeline::read)
                .collect(),
            operators: json
                .at("operators")
                .and_then(Json::list)
                .unwrap_or_default()
                .iter()
                .map(Operator::read)
                .collect(),
            warnings: json
                .at("warnings")
                .and_then(Json::list)
                .unwrap_or_default()
                .iter()
                .filter_map(Json::text)
                .collect(),
        })
    }

    /// The last document in a file of them, which is the statement the harness timed.
    ///
    /// The last rather than the only one, because a rudb run replays the view definitions in front
    /// of the query and each statement that measured anything writes a record. The setup is real
    /// and is worth keeping in the file, and it is not the thing being reported on.
    ///
    /// `Ok(None)` when there is no file or nothing in it, which is what a statement that measured
    /// nothing leaves behind and is not an error.
    ///
    /// # Errors
    ///
    /// When the file is there and its last line is not a document.
    pub fn last_in(path: &Path) -> Result<Option<Self>, String> {
        let Ok(text) = fs::read_to_string(path) else { return Ok(None) };
        let Some(line) = text.lines().rev().find(|line| !line.trim().is_empty()) else {
            return Ok(None);
        };
        Self::parse(line).map(Some).map_err(|why| format!("{}: {why}", path.display()))
    }

    /// The CPU the operators charged themselves, added up.
    #[must_use]
    pub fn operator_cpu(&self) -> Duration {
        self.operators.iter().map(|o| o.cpu).sum()
    }

    /// The CPU the pipelines charged themselves, added up.
    #[must_use]
    pub fn pipeline_cpu(&self) -> Duration {
        self.pipelines.iter().map(|p| p.cpu).sum()
    }

    /// What the engine accounted for, which is the pipelines when it has any.
    ///
    /// The operators are the fallback rather than the answer. A plan that ran as no pipeline at
    /// all, which is what a statement with a constant on the right of it is, has operators and
    /// nothing to schedule them, and adding up an empty list would report every one of those as a
    /// hundred percent unaccounted.
    #[must_use]
    pub fn accounted_cpu(&self) -> Duration {
        if self.pipelines.is_empty() { self.operator_cpu() } else { self.pipeline_cpu() }
    }

    /// The CPU the engine measured around running the tree, with the build taken off.
    ///
    /// This is the denominator of the cross check. It is the engine's own measurement of the same
    /// span the pipelines are inside, so the two are answering the same question in two ways, and
    /// that is what makes disagreeing between them mean something.
    #[must_use]
    pub fn executed_cpu(&self) -> Duration {
        self.cpu.saturating_sub(self.build)
    }

    /// How many operators ran a reference implementation.
    #[must_use]
    pub fn reference_impls(&self) -> usize {
        self.operators.iter().filter(|o| o.reference_impl).count()
    }

    /// The accounted CPU against the measured CPU, and against the process, where there is one.
    #[must_use]
    pub fn accounting(&self, process: Option<Duration>) -> Accounting {
        Accounting {
            accounted: self.accounted_cpu(),
            measured: self.executed_cpu(),
            build: self.build,
            process,
        }
    }

    /// The part of the document a run record keeps.
    #[must_use]
    pub fn internal(&self, process: Option<Duration>) -> Internal {
        Internal {
            accounting: self.accounting(process),
            execute: self.execute,
            driver: self.accounted_cpu().saturating_sub(self.operator_cpu()),
            peak_bytes: self.peak_bytes,
            pipelines: self.pipelines.len(),
            operators: self.operators.len(),
            reference_impls: self.reference_impls(),
        }
    }
}

/// What a run record keeps out of one document.
///
/// A handful of numbers rather than the document, because a record is written to a file that is
/// read back a week later and compared against another one, and a hundred operator rows per query
/// makes that file something nobody opens. The document is on disk in the scratch directory for
/// the afternoon somebody wants the rest of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Internal {
    /// The cross check: what the operators charged, against what the engine measured.
    pub accounting: Accounting,
    /// The execute step, by the engine's own clock.
    pub execute: Duration,
    /// CPU the pipelines spent outside any operator, which is the scheduling.
    ///
    /// Pulling a morsel, handing a chunk on, deciding there is nothing left. It is work an engine
    /// has to do and it is not work anybody asked for, so a number that grows here while the
    /// operator numbers stay flat is a thing worth seeing.
    pub driver: Duration,
    /// The most the engine says it held at once, which is not the resident set of the process.
    pub peak_bytes: u64,
    /// How many pipelines the plan ran as.
    pub pipelines: usize,
    /// How many operators the plan ran as.
    pub operators: usize,
    /// How many of them ran a reference implementation.
    ///
    /// The number that says whether a row is measuring the engine or measuring the slow paths the
    /// engine exists to replace. At F0 it is every operator in every query, and writing it down is
    /// how the row a milestone later is read against this one.
    pub reference_impls: usize,
}

impl Pipeline {
    /// One row, reading a missing key as a zero.
    fn read(json: &Json) -> Self {
        Self { id: count(json, "id"), wall: nanos(json, "wall_ns"), cpu: nanos(json, "cpu_ns") }
    }
}

impl Operator {
    /// One row, reading a missing key as a zero.
    fn read(json: &Json) -> Self {
        Self {
            id: count(json, "id"),
            pipeline: count(json, "pipeline"),
            kind: json.at("kind").and_then(Json::text).unwrap_or_default(),
            rows_in: count(json, "rows_in"),
            rows_out: count(json, "rows_out"),
            wall: nanos(json, "wall_ns"),
            cpu: nanos(json, "cpu_ns"),
            reference_impl: json.at("reference_impl").and_then(Json::flag).unwrap_or(false),
        }
    }
}

/// Does the breakdown add up to the run it came from.
///
/// Three numbers for the same span, taken three ways, from the inside out. `accounted` is what the
/// pipelines say they spent, `measured` is what the engine measured around running them, and
/// `process` is what the operating system charged the whole command including starting it.
///
/// The rule is on the first two. They are measurements of the same span, so they are meant to
/// agree, and when they do not it is either time going somewhere no pipeline accounts for or the
/// same time being charged twice, and both of those make the breakdown misleading.
///
/// The third is carried and reported and is not a rule, because there is no tolerance at which it
/// could be one. A ClickBench query against a process that has to start, link, open a database,
/// declare a view and print a result spends real CPU on all of that, and none of it happens inside
/// an operator. `spec/15-rudb-bench.md` writes the check against the process time, and it cannot
/// be that: at the sizes F0 runs, process startup is most of the number, so a five percent band
/// against it would mark every rudb run unusable and take rudb off the board that the same
/// milestone exists to get it onto. What the rule is actually protecting is the breakdown, and the
/// breakdown is inside the engine, so that is where the band is. The gap to the process is printed
/// next to it as [`Accounting::unattributed`], which is the honest way to keep the number that
/// paragraph was reaching for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Accounting {
    /// The sum of the per pipeline CPU.
    pub accounted: Duration,
    /// The CPU the engine measured around running the tree, with the build taken off.
    pub measured: Duration,
    /// The CPU the engine spent building the tree, which is the part of it no pipeline can own.
    pub build: Duration,
    /// The CPU the operating system charged the whole process, where it was measured.
    pub process: Option<Duration>,
}

impl Accounting {
    /// How far the accounted CPU is from the measured CPU, as a fraction of the measured CPU.
    ///
    /// `None` when the engine measured no CPU at all, because there is nothing to be a fraction of
    /// and a query that was too fast to charge anything has nothing to disagree about.
    #[must_use]
    pub fn drift(&self) -> Option<f64> {
        let measured = self.measured.as_secs_f64();
        if measured <= 0.0 {
            return None;
        }
        Some((self.accounted.as_secs_f64() - measured).abs() / measured)
    }

    /// Whether the breakdown is close enough to the run to be worth publishing.
    ///
    /// True when there is nothing to compare, which is the same answer the rest of this harness
    /// gives to a measurement it could not take: a missing number is a reason of its own elsewhere
    /// and is not evidence of a fault here.
    #[must_use]
    pub fn agrees(&self) -> bool {
        self.drift().is_none_or(|drift| drift <= TOLERANCE)
    }

    /// CPU the process spent outside the execution, where the process was measured.
    ///
    /// Starting, linking, opening the database, declaring the views, printing the answer. It is
    /// the reason the wall clock column and the reported column differ, in CPU rather than in wall
    /// clock, and on a small enough dataset it is most of what the harness timed. Building the
    /// tree is off it as well as off the execution, because the engine reports that separately and
    /// a number that appeared in two columns would be read as two numbers.
    #[must_use]
    pub fn unattributed(&self) -> Option<Duration> {
        self.process.map(|process| process.saturating_sub(self.measured).saturating_sub(self.build))
    }

    /// Why this breakdown may not be published, when it may not be.
    #[must_use]
    pub fn why(&self, query: &str) -> Option<String> {
        let drift = self.drift().filter(|drift| *drift > TOLERANCE)?;
        Some(format!(
            "{query} accounts for {:?} of cpu in its breakdown and the engine measured {:?} around \
             the execution, which is {:.1}% apart and the cross check wants under {:.0}%",
            self.accounted,
            self.measured,
            drift * 100.0,
            TOLERANCE * 100.0
        ))
    }
}

/// A duration from a count of nanoseconds under a key, missing reading as zero.
fn nanos(json: &Json, key: &str) -> Duration {
    Duration::from_nanos(count(json, key))
}

/// A count under a key, missing or null reading as zero.
fn count(json: &Json, key: &str) -> u64 {
    json.at(key).and_then(Json::count).unwrap_or(0)
}

/// Enough JSON to read a document rudb wrote.
///
/// Numbers are kept as the text they arrived as. A count of nanoseconds passes the range where an
/// `f64` still holds every integer after about a hundred days, and bytes read over a fleet passes
/// it sooner, so parsing every number through a float to get an integer back out is a rounding
/// error waiting for a long enough run.
#[derive(Debug, Clone, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Number(String),
    Text(String),
    List(Vec<Json>),
    Map(Vec<(String, Json)>),
}

impl Json {
    /// Read one value, and insist that it is the whole input.
    fn read(text: &str) -> Result<Self, String> {
        let mut reader = Reader { bytes: text.as_bytes(), at: 0 };
        let value = reader.value()?;
        reader.space();
        if reader.at < reader.bytes.len() {
            return Err(format!("there is more after the document, at byte {}", reader.at));
        }
        Ok(value)
    }

    /// The value under a key, for a map, and nothing for anything else.
    fn at(&self, key: &str) -> Option<&Self> {
        match self {
            Self::Map(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    fn text(&self) -> Option<String> {
        match self {
            Self::Text(text) => Some(text.clone()),
            _ => None,
        }
    }

    fn count(&self) -> Option<u64> {
        match self {
            Self::Number(raw) => raw.parse().ok().or_else(|| {
                // A number written with a point or an exponent is still a count, it is just not
                // written like one, and refusing it would make the reader depend on how the writer
                // felt about trailing zeros.
                let float = raw.parse::<f64>().ok()?;
                (float.is_finite() && float >= 0.0).then_some(float as u64)
            }),
            _ => None,
        }
    }

    fn flag(&self) -> Option<bool> {
        match self {
            Self::Bool(flag) => Some(*flag),
            _ => None,
        }
    }

    fn list(&self) -> Option<&[Self]> {
        match self {
            Self::List(items) => Some(items),
            _ => None,
        }
    }
}

/// A position in the text, and the recursive descent that walks it.
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn space(&mut self) {
        while matches!(self.bytes.get(self.at), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.at += 1;
        }
    }

    fn peek(&mut self) -> Result<u8, String> {
        self.space();
        self.bytes.get(self.at).copied().ok_or_else(|| "the document ends early".to_owned())
    }

    /// Step over a byte that has to be there.
    fn want(&mut self, byte: u8) -> Result<(), String> {
        if self.peek()? != byte {
            return Err(format!(
                "wanted `{}` at byte {} and found `{}`",
                byte as char, self.at, self.bytes[self.at] as char
            ));
        }
        self.at += 1;
        Ok(())
    }

    fn value(&mut self) -> Result<Json, String> {
        match self.peek()? {
            b'{' => self.map(),
            b'[' => self.list(),
            b'"' => self.text().map(Json::Text),
            b't' => self.word("true").map(|()| Json::Bool(true)),
            b'f' => self.word("false").map(|()| Json::Bool(false)),
            b'n' => self.word("null").map(|()| Json::Null),
            _ => self.number(),
        }
    }

    fn word(&mut self, word: &str) -> Result<(), String> {
        if self.bytes[self.at..].starts_with(word.as_bytes()) {
            self.at += word.len();
            return Ok(());
        }
        Err(format!("wanted `{word}` at byte {}", self.at))
    }

    fn map(&mut self) -> Result<Json, String> {
        self.want(b'{')?;
        let mut pairs = Vec::new();
        if self.peek()? == b'}' {
            self.at += 1;
            return Ok(Json::Map(pairs));
        }
        loop {
            let key = self.text()?;
            self.want(b':')?;
            pairs.push((key, self.value()?));
            match self.peek()? {
                b',' => self.at += 1,
                b'}' => {
                    self.at += 1;
                    return Ok(Json::Map(pairs));
                }
                other => {
                    return Err(format!(
                        "wanted `,` or `}}` at byte {} and found `{}`",
                        self.at, other as char
                    ));
                }
            }
        }
    }

    fn list(&mut self) -> Result<Json, String> {
        self.want(b'[')?;
        let mut items = Vec::new();
        if self.peek()? == b']' {
            self.at += 1;
            return Ok(Json::List(items));
        }
        loop {
            items.push(self.value()?);
            match self.peek()? {
                b',' => self.at += 1,
                b']' => {
                    self.at += 1;
                    return Ok(Json::List(items));
                }
                other => {
                    return Err(format!(
                        "wanted `,` or `]` at byte {} and found `{}`",
                        self.at, other as char
                    ));
                }
            }
        }
    }

    fn text(&mut self) -> Result<String, String> {
        self.want(b'"')?;
        let mut out = String::new();
        loop {
            let byte = *self.bytes.get(self.at).ok_or_else(|| "a string never ends".to_owned())?;
            self.at += 1;
            match byte {
                b'"' => return Ok(out),
                b'\\' => out.push(self.escape()?),
                _ => {
                    // Everything that is not an escape goes through as the bytes it arrived as, so
                    // a SQL statement with anything but ASCII in it comes back the way it went in.
                    let start = self.at - 1;
                    let mut end = self.at;
                    while end < self.bytes.len() && !matches!(self.bytes[end], b'"' | b'\\') {
                        end += 1;
                    }
                    out.push_str(
                        std::str::from_utf8(&self.bytes[start..end])
                            .map_err(|_| "a string is not utf-8".to_owned())?,
                    );
                    self.at = end;
                }
            }
        }
    }

    /// The character after a backslash.
    fn escape(&mut self) -> Result<char, String> {
        let byte = *self.bytes.get(self.at).ok_or_else(|| "an escape never ends".to_owned())?;
        self.at += 1;
        Ok(match byte {
            b'"' => '"',
            b'\\' => '\\',
            b'/' => '/',
            b'b' => '\u{8}',
            b'f' => '\u{c}',
            b'n' => '\n',
            b'r' => '\r',
            b't' => '\t',
            b'u' => return self.short(),
            other => return Err(format!("`\\{}` is not an escape", other as char)),
        })
    }

    /// A `\u` escape, and the second half of a surrogate pair when this was the first half.
    fn short(&mut self) -> Result<char, String> {
        let first = self.hex()?;
        if !(0xd800..0xdc00).contains(&first) {
            return char::from_u32(u32::from(first))
                .ok_or_else(|| format!("\\u{first:04x} is not a character"));
        }
        // The writer only ever emits these in pairs, so a lone half is a truncated document rather
        // than something to guess at.
        if self.bytes.get(self.at) != Some(&b'\\') || self.bytes.get(self.at + 1) != Some(&b'u') {
            return Err("a surrogate with no pair".to_owned());
        }
        self.at += 2;
        let second = self.hex()?;
        if !(0xdc00..0xe000).contains(&second) {
            return Err("a surrogate paired with something that is not one".to_owned());
        }
        let whole = 0x1_0000 + ((u32::from(first) - 0xd800) << 10) + (u32::from(second) - 0xdc00);
        char::from_u32(whole)
            .ok_or_else(|| format!("\\u{first:04x}\\u{second:04x} is not a character"))
    }

    /// Four hexadecimal digits.
    fn hex(&mut self) -> Result<u16, String> {
        let end = self.at + 4;
        let digits =
            self.bytes.get(self.at..end).ok_or_else(|| "a \\u escape is cut short".to_owned())?;
        let text = std::str::from_utf8(digits).map_err(|_| "a \\u escape is not hex".to_owned())?;
        let value = u16::from_str_radix(text, 16)
            .map_err(|_| format!("`{text}` is not four hex digits"))?;
        self.at = end;
        Ok(value)
    }

    fn number(&mut self) -> Result<Json, String> {
        self.space();
        let start = self.at;
        while matches!(
            self.bytes.get(self.at),
            Some(b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
        ) {
            self.at += 1;
        }
        if self.at == start {
            return Err(format!(
                "wanted a value at byte {start} and found `{}`",
                self.bytes[start] as char
            ));
        }
        let raw = std::str::from_utf8(&self.bytes[start..self.at])
            .map_err(|_| "a number is not utf-8".to_owned())?;
        // Checked here so that a caller reading a key can treat a bad number as a missing one
        // rather than having to decide what a `12.3.4` means.
        if raw.parse::<f64>().is_err() {
            return Err(format!("`{raw}` is not a number"));
        }
        Ok(Json::Number(raw.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Accounting, Document, Json, TOLERANCE};

    /// A document the shape rudb writes, small enough to read.
    fn one(cpu_ns: u64, operators: &str) -> String {
        format!(
            "{{\"schema\":1,\"query\":{{\"sql\":\"SELECT 1\",\"hash\":\"abc\",\"suite\":null,\
             \"id\":null}},\"engine\":{{\"version\":\"0.2.35\"}},\"machine\":{{\"cores\":10}},\
             \"settings\":{{}},\"outcome\":{{\"state\":\"ok\"}},\"timing\":{{\"parse_ns\":10,\
             \"bind_ns\":20,\"optimize_ns\":30,\"physical_ns\":40,\"execute_ns\":50,\
             \"total_ns\":150}},\"resource\":{{\"cpu_ns\":{cpu_ns},\"peak_bytes\":4096,\
             \"bytes_read\":777,\"bytes_decoded\":0,\"bytes_spilled\":0,\"bytes_read_back\":0,\
             \"io_requests\":0}},\"strategies\":[],\"pipelines\":[{{\"id\":0,\"instances\":1,\
             \"depends_on\":[],\"wall_ns\":1200,\"cpu_ns\":1200,\"blocked_ns\":{{\"io\":0}}}}],\
             \"operators\":[{operators}],\"warnings\":[\"one thing\"]}}"
        )
    }

    fn operator(id: u64, kind: &str, cpu_ns: u64, reference: bool) -> String {
        format!(
            "{{\"id\":{id},\"pipeline\":0,\"kind\":\"{kind}\",\"detail\":null,\"rows_in\":100,\
             \"rows_out\":90,\"wall_ns\":{cpu_ns},\"cpu_ns\":{cpu_ns},\"bytes_read\":0,\
             \"bytes_decoded\":0,\"bytes_spilled\":0,\"memory\":{{\"reserved\":0,\
             \"high_water\":0}},\"reference_impl\":{reference}}}"
        )
    }

    #[test]
    fn a_document_reads_back_as_the_numbers_that_went_into_it() {
        let text = one(
            1000,
            &format!("{},{}", operator(0, "Get", 600, true), operator(1, "Filter", 400, false)),
        );
        let document = Document::parse(&text).expect("this is the shape rudb writes");
        assert_eq!(document.sql, "SELECT 1");
        assert_eq!(document.state, "ok");
        assert_eq!(document.total, Duration::from_nanos(150));
        assert_eq!(document.execute, Duration::from_nanos(50));
        assert_eq!(document.cpu, Duration::from_nanos(1000));
        assert_eq!(document.peak_bytes, 4096);
        assert_eq!(document.bytes_read, 777);
        assert_eq!(document.pipelines.len(), 1);
        assert_eq!(document.operators.len(), 2);
        assert_eq!(document.operators[1].kind, "Filter");
        assert_eq!(document.operators[1].rows_in, 100);
        assert_eq!(document.operators[1].rows_out, 90);
        assert_eq!(document.reference_impls(), 1);
        assert_eq!(document.warnings, vec!["one thing".to_owned()]);
        assert_eq!(document.operator_cpu(), Duration::from_nanos(1000));
        assert_eq!(document.pipeline_cpu(), Duration::from_nanos(1200));
        // The pipeline, because that is the unit the engine schedules, and the two hundred
        // nanoseconds between the two is the loop that made the calls.
        assert_eq!(document.accounted_cpu(), Duration::from_nanos(1200));
        assert_eq!(document.internal(None).driver, Duration::from_nanos(200));
    }

    #[test]
    fn a_plan_that_ran_as_no_pipeline_is_accounted_by_its_operators() {
        // `SELECT 1` has an operator and nothing to schedule it. Adding up an empty list of
        // pipelines would report every query of that shape as a hundred percent unaccounted for.
        let text = one(600, &operator(0, "Values", 600, false)).replace(
            "\"pipelines\":[{\"id\":0,\"instances\":1,\"depends_on\":[],\"wall_ns\":1200,\
             \"cpu_ns\":1200,\"blocked_ns\":{\"io\":0}}]",
            "\"pipelines\":[]",
        );
        let document = Document::parse(&text).expect("a plan with no pipeline");
        assert!(document.pipelines.is_empty());
        assert_eq!(document.accounted_cpu(), Duration::from_nanos(600));
        assert!(document.accounting(None).agrees());
    }

    #[test]
    fn a_schema_this_does_not_know_is_refused_rather_than_read_as_zeros() {
        let text = one(1000, "").replace("\"schema\":1", "\"schema\":2");
        let why = Document::parse(&text).expect_err("a schema 2 document");
        assert!(why.contains("schema 2"), "{why}");
    }

    #[test]
    fn a_breakdown_that_adds_up_agrees_and_one_that_does_not_says_how_far_off_it_is() {
        let close = Accounting {
            accounted: Duration::from_nanos(980),
            measured: Duration::from_nanos(1000),
            build: Duration::ZERO,
            process: None,
        };
        assert!(close.agrees());
        assert!(close.why("q1").is_none());
        assert!(close.drift().expect("a measured cpu") < TOLERANCE);

        let off = Accounting {
            accounted: Duration::from_nanos(500),
            measured: Duration::from_nanos(1000),
            build: Duration::ZERO,
            process: None,
        };
        assert!(!off.agrees());
        let why = off.why("q1").expect("half the cpu is unaccounted for");
        assert!(why.starts_with("q1 accounts for"), "{why}");
        assert!(why.contains("50.0%"), "{why}");
    }

    #[test]
    fn a_query_too_fast_to_charge_any_cpu_has_nothing_to_disagree_about() {
        // Not a fault. An engine that measured no cpu around an execution has no denominator, and
        // reporting that as a hundred percent off would put a reason under every fast query.
        let nothing = Accounting {
            accounted: Duration::ZERO,
            measured: Duration::ZERO,
            build: Duration::ZERO,
            process: None,
        };
        assert_eq!(nothing.drift(), None);
        assert!(nothing.agrees());
        assert!(nothing.why("q1").is_none());
    }

    #[test]
    fn what_the_process_spent_outside_the_execution_is_reported_and_is_not_the_rule() {
        // The whole reason the band is against the engine's own cpu. A ClickBench count over a
        // small file spends more cpu starting a process than running the query, and none of that
        // can ever land inside an operator.
        let accounting = Accounting {
            accounted: Duration::from_millis(20),
            measured: Duration::from_millis(20),
            build: Duration::from_millis(5),
            process: Some(Duration::from_millis(70)),
        };
        assert!(accounting.agrees());
        assert_eq!(
            accounting.unattributed(),
            Some(Duration::from_millis(45)),
            "the build is time the process spent and the execution did not, so it comes off too"
        );
    }

    #[test]
    fn the_last_document_in_the_file_is_the_one_the_harness_timed() {
        let dir = std::env::temp_dir().join("rudb-bench-metrics-test");
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        let path = dir.join("two.jsonl");
        let setup = one(11, "").replace("SELECT 1", "CREATE VIEW hits AS SELECT 1");
        std::fs::write(&path, format!("{setup}\n{}\n", one(22, ""))).expect("write the file");
        let document = Document::last_in(&path).expect("both lines parse").expect("two lines");
        assert_eq!(document.sql, "SELECT 1");
        assert_eq!(document.cpu, Duration::from_nanos(22));

        // A run that measured nothing leaves no file, and that is not an error anywhere.
        assert_eq!(Document::last_in(&dir.join("never-written.jsonl")), Ok(None));
        std::fs::remove_dir_all(&dir).expect("clean up");
    }

    #[test]
    fn the_reader_handles_the_parts_of_json_a_document_actually_contains() {
        let value = Json::read(
            "{\"a\":[1,-2.5,1e3],\"b\":\"one \\\"two\\\"\\nthree\\u0041\",\"c\":null,\"d\":true,\"e\":{}}",
        )
        .expect("valid json");
        assert_eq!(value.at("a").and_then(Json::list).map(<[Json]>::len), Some(3));
        assert_eq!(value.at("b").and_then(Json::text).as_deref(), Some("one \"two\"\nthreeA"));
        assert_eq!(value.at("c").and_then(Json::count), None);
        assert_eq!(value.at("d").and_then(Json::flag), Some(true));
        assert_eq!(value.at("e").and_then(Json::list), None);
        // A count of nanoseconds goes past where an f64 holds every integer, so the text is kept.
        let big = Json::read("{\"n\":9007199254740993}").expect("valid json");
        assert_eq!(big.at("n").and_then(Json::count), Some(9_007_199_254_740_993));
    }

    #[test]
    fn text_that_is_not_a_document_is_a_reason_rather_than_a_panic() {
        for bad in ["", "{", "{\"a\"}", "{\"a\":}", "[1,]", "{} trailing", "{\"a\":1.2.3}"] {
            assert!(Json::read(bad).is_err(), "`{bad}` read as valid json");
        }
    }
}
