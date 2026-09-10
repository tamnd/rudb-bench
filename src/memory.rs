//! What a run cost besides wall clock: peak resident memory, CPU seconds and bytes read.
//!
//! A query that is fast because it used 30 GB is not fast, it is expensive, and axis 4 of
//! `spec/02-the-goal.md` makes the resource claim first class rather than a footnote. So the type a
//! result carries is not a `u64`. It is a [`Peak`], which is either a number of bytes or a reason
//! there is no number, and a result whose peak is a reason is not publishable. That way an engine
//! measured on a machine where the peak could not be read produces a table that says so instead of
//! a table with a blank column somebody fills in from memory later.
//!
//! CPU seconds and bytes read are in a [`Cost`] next to it, for two different reasons. CPU seconds
//! is the number that says whether a win is an engine or a thread count, and on a fleet where one
//! machine has four cores and another has eight it is the only cross machine number in the table
//! that means anything. Bytes read is the number that says whether hot was actually hot. It is
//! block layer reads, so a run the page cache served reads zero, and a hot run that did not read
//! zero is a hot run that was not warm.
//!
//! ## How it is read
//!
//! For a subprocess, `/usr/bin/time` with its output sent to a file by `-o`, which both the GNU and
//! the BSD flavours support. The file matters: `time` writes to stderr by default and the engines
//! here are driven by parsing their stderr, so letting the two share a stream would turn a resource
//! measurement into a spurious engine error.
//!
//! The two flavours differ in the flag, the wording and the unit. GNU wants `-v` and reports
//! kibibytes. BSD wants `-l` and reports bytes. Both are parsed, chosen by which line actually
//! appeared, rather than by deciding from `cfg!(target_os)` what should have appeared.
//!
//! ## Why not `getrusage`
//!
//! `spec/engine/13-measurement.md` asks for `getrusage` with `RUSAGE_CHILDREN` as the source and
//! `/usr/bin/time` as the cross check. This is the other way round, and the reason is that this
//! crate has no dependencies and `getrusage` needs either `libc` or an `unsafe` block declaring the
//! symbol by hand, for a number `/usr/bin/time` already reads out of the same field of the same
//! struct. The cross check that is left is a bound rather than a second reading:
//! [`Cost::implausible`] says so when CPU seconds exceed wall clock times the thread count, which
//! is the shape a misparsed unit takes. On Windows there is no `/usr/bin/time` and the honest
//! answer is a reason, which is what a missing timer produces everywhere else too.
//!
//! For this process, `/proc/self/status` on Linux and nothing anywhere else.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// Peak resident memory, or why there is not a number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Peak {
    /// High water mark of resident set size, in bytes.
    Bytes(u64),
    /// It could not be read here, and this is why.
    Unavailable(String),
}

impl Peak {
    /// The number, when there is one.
    #[must_use]
    pub const fn bytes(&self) -> Option<u64> {
        match self {
            Self::Bytes(n) => Some(*n),
            Self::Unavailable(_) => None,
        }
    }

    /// Whether a result carrying this peak satisfies rule six.
    #[must_use]
    pub const fn measured(&self) -> bool {
        matches!(self, Self::Bytes(_))
    }
}

impl std::fmt::Display for Peak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bytes(n) => write!(f, "{}", bytes(*n)),
            Self::Unavailable(why) => write!(f, "not measured, {why}"),
        }
    }
}

/// What one run cost, other than the wall clock the caller timed.
///
/// Three fields with three different absence stories, which is why they are not one `Option`. The
/// peak carries its own reason because rule six blocks publication without it. CPU seconds and
/// bytes read are plain options, because a machine without a timer has neither and saying the same
/// sentence three times in one table row helps nobody.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cost {
    /// High water mark of resident set size.
    pub peak: Peak,
    /// User plus system time, over the process and everything it waited for.
    pub cpu: Option<Duration>,
    /// Bytes read at the block layer, which is zero when the page cache served the read.
    pub read: Option<u64>,
}

impl Cost {
    /// Nothing was measured, and this is why.
    #[must_use]
    pub fn unavailable(why: impl Into<String>) -> Self {
        Self { peak: Peak::Unavailable(why.into()), cpu: None, read: None }
    }

    /// Whether the CPU number is impossible on a machine with this many threads.
    ///
    /// The bound is generous on purpose. Wall clock times threads is the ceiling, and a run that
    /// exceeds it has a parsing bug rather than a fast engine, because no scheduler hands out more
    /// CPU seconds than it has. Half a second of slack covers the timer's own start up, which is
    /// charged to the child on both flavours.
    #[must_use]
    pub fn implausible(&self, wall: Duration, threads: usize) -> bool {
        #[expect(clippy::cast_precision_loss, reason = "a thread count, not a measurement")]
        let ceiling = wall.as_secs_f64() * threads as f64 + 0.5;
        self.cpu.is_some_and(|cpu| cpu.as_secs_f64() > ceiling)
    }
}

/// The `/usr/bin/time` on this machine, and which flavour it is.
///
/// Found once and reused, because probing it per query would be a subprocess per query for a fact
/// that does not change, and because a machine where the answer changes halfway through a suite has
/// a bigger problem than this harness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timer {
    binary: PathBuf,
    flag: &'static str,
}

impl Timer {
    /// Find a `time` that can report a peak, or say what is missing.
    ///
    /// # Errors
    ///
    /// When there is no `/usr/bin/time`, or when it does not answer either flag, which is what a
    /// busybox `time` does.
    pub fn find() -> Result<Self, String> {
        let binary = PathBuf::from("/usr/bin/time");
        if !binary.is_file() {
            return Err("there is no /usr/bin/time on this machine".to_owned());
        }
        for flag in ["-v", "-l"] {
            if probe(&binary, flag) {
                return Ok(Self { binary, flag });
            }
        }
        Err("/usr/bin/time answers neither -v nor -l".to_owned())
    }

    /// The binary, for the machine record.
    #[must_use]
    pub fn binary(&self) -> &Path {
        &self.binary
    }

    /// Which flavour, for the machine record.
    #[must_use]
    pub fn flavour(&self) -> &'static str {
        if self.flag == "-v" { "GNU" } else { "BSD" }
    }

    /// Build the command that runs `program` under this timer, writing the resource report to
    /// `report`.
    ///
    /// The caller gets a [`Command`] rather than a finished run, because the engines here already
    /// know how to build their own argument lists and how to read their own output, and a wrapper
    /// that took that over would have to reimplement both.
    #[must_use]
    pub fn wrap(&self, program: &Path, report: &Path) -> Command {
        let mut command = Command::new(&self.binary);
        command.arg(self.flag).arg("-o").arg(report).arg(program);
        command
    }

    /// Read back what the timer wrote.
    #[must_use]
    pub fn read(report: &Path) -> Cost {
        match std::fs::read_to_string(report) {
            Ok(text) => parse(&text),
            Err(e) => Cost::unavailable(format!("cannot read the timer report: {e}")),
        }
    }
}

/// Whether `/usr/bin/time` accepts a flag, decided by running it on something that always works.
fn probe(binary: &Path, flag: &str) -> bool {
    Command::new(binary).arg(flag).arg("/usr/bin/true").output().is_ok_and(|out| {
        out.status.success() && parse(&String::from_utf8_lossy(&out.stderr)).peak.measured()
    })
}

/// Pull the peak, the CPU seconds and the blocks read out of a `time` report, whichever flavour
/// wrote it.
///
/// GNU writes one field per line with a name in front of the number. BSD writes `real user sys` on
/// one line and then one field per line with the name behind the number. Both are parsed by what
/// actually appeared rather than by what `cfg!(target_os)` says should have appeared, because a GNU
/// `time` installed on a Mac is a thing that happens.
///
/// The units differ the same way. GNU's peak is kibibytes and BSD's is bytes, and reading one as
/// the other is out by a factor of 1024. Both count reads in 512 byte blocks, which is the kernel's
/// unit for `ru_inblock` and not a choice either of them made.
fn parse(text: &str) -> Cost {
    const BLOCK: u64 = 512;
    let mut peak = None;
    let mut user = None;
    let mut system = None;
    let mut blocks = None;

    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Maximum resident set size (kbytes):") {
            peak = rest.trim().parse::<u64>().ok().map(|kib| kib * 1024);
        } else if let Some(rest) = line.strip_prefix("User time (seconds):") {
            user = seconds(rest);
        } else if let Some(rest) = line.strip_prefix("System time (seconds):") {
            system = seconds(rest);
        } else if let Some(rest) = line.strip_prefix("File system inputs:") {
            blocks = rest.trim().parse::<u64>().ok();
        } else if line.ends_with("maximum resident set size") {
            peak = leading(line);
        } else if line.ends_with("block input operations") {
            blocks = leading(line);
        } else if line.contains(" real ") && line.contains(" user ") {
            // BSD puts all three on one line as `0.01 real 0.00 user 0.00 sys`, so the number in
            // front of each name is the one that belongs to it.
            let words: Vec<&str> = line.split_whitespace().collect();
            for pair in words.windows(2) {
                match pair[1] {
                    "user" => user = seconds(pair[0]),
                    "sys" => system = seconds(pair[0]),
                    _ => {}
                }
            }
        }
    }

    // Either half of the CPU total on its own would be a number that looks like CPU seconds and is
    // not, so a report with only one of them has none.
    let cpu = match (user, system) {
        (Some(u), Some(s)) => Some(u + s),
        _ => None,
    };
    Cost {
        peak: peak.map_or_else(
            || Peak::Unavailable("the timer report had no peak in it".to_owned()),
            Peak::Bytes,
        ),
        cpu,
        read: blocks.map(|n| n * BLOCK),
    }
}

/// A count of seconds written as a decimal, as both flavours write one.
fn seconds(text: &str) -> Option<Duration> {
    Duration::try_from_secs_f64(text.trim().parse::<f64>().ok()?).ok()
}

/// The number at the front of a BSD style line.
fn leading(line: &str) -> Option<u64> {
    line.split_whitespace().next()?.parse::<u64>().ok()
}

/// This process's own high water mark, where the system will say.
///
/// Used for in-process micro-benchmarks, where there is no child to wrap. On anything but Linux
/// this is a reason rather than a number, and that is the honest answer rather than a zero.
#[must_use]
pub fn own_peak() -> Peak {
    let Ok(status) = std::fs::read_to_string("/proc/self/status") else {
        return Peak::Unavailable("no /proc/self/status on this system".to_owned());
    };
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("VmHWM:") {
            let kib = rest.split_whitespace().next().and_then(|n| n.parse::<u64>().ok());
            if let Some(kib) = kib {
                return Peak::Bytes(kib * 1024);
            }
        }
    }
    Peak::Unavailable("/proc/self/status has no VmHWM line".to_owned())
}

/// Format a byte count the way every table here formats one.
///
/// Binary units, because that is what the kernel reports and converting to powers of ten to make a
/// number look smaller is the sort of thing this document exists to prevent.
#[must_use]
pub fn bytes(n: u64) -> String {
    #[expect(clippy::cast_precision_loss, reason = "three significant figures of a byte count")]
    let value = n as f64;
    const KIB: f64 = 1024.0;
    if value >= KIB * KIB * KIB {
        format!("{:.2} GiB", value / (KIB * KIB * KIB))
    } else if value >= KIB * KIB {
        format!("{:.2} MiB", value / (KIB * KIB))
    } else if value >= KIB {
        format!("{:.2} KiB", value / KIB)
    } else {
        format!("{n} B")
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Cost, Peak, Timer, bytes, own_peak, parse};

    const GNU: &str = "\tCommand being timed: \"duckdb\"
\tUser time (seconds): 3.25
\tSystem time (seconds): 0.75
\tPercent of CPU this job got: 380%
\tMaximum resident set size (kbytes): 2048
\tFile system inputs: 4096
\tFile system outputs: 0
";

    const BSD: &str = "        1.20 real         0.90 user         0.10 sys
             1245184  maximum resident set size
                 128  block input operations
";

    #[test]
    fn a_gnu_report_is_kibibytes_and_its_cpu_is_user_plus_system() {
        let cost = parse(GNU);
        assert_eq!(cost.peak, Peak::Bytes(2048 * 1024));
        assert_eq!(cost.cpu, Some(Duration::from_millis(4000)));
        assert_eq!(cost.read, Some(4096 * 512));
    }

    #[test]
    fn a_bsd_report_is_bytes_and_its_three_numbers_are_on_one_line() {
        let cost = parse(BSD);
        assert_eq!(cost.peak, Peak::Bytes(1_245_184));
        assert_eq!(cost.cpu, Some(Duration::from_millis(1000)));
        assert_eq!(cost.read, Some(128 * 512));
    }

    #[test]
    fn a_report_with_no_peak_in_it_is_a_reason_rather_than_zero() {
        let cost = parse("        0.00 real         0.00 user         0.00 sys\n");
        assert!(!cost.peak.measured());
        assert_eq!(cost.read, None);
    }

    #[test]
    fn half_a_cpu_total_is_no_cpu_total() {
        // A report with user and no system would otherwise print a number that looks like CPU
        // seconds, is smaller than the real one, and has nothing marking it as partial.
        let cost = parse("\tUser time (seconds): 3.25\n\tMaximum resident set size (kbytes): 8\n");
        assert_eq!(cost.cpu, None);
        assert!(cost.peak.measured());
    }

    #[test]
    fn a_hot_run_that_read_from_the_disk_is_visible_as_a_number_that_is_not_zero() {
        assert_eq!(parse(BSD).read, Some(65536));
        assert_eq!(parse("             0  block input operations\n").read, Some(0));
    }

    #[test]
    fn more_cpu_seconds_than_the_machine_has_is_a_parsing_bug_and_says_so() {
        let cost =
            Cost { peak: Peak::Bytes(1), cpu: Some(Duration::from_secs(100)), read: Some(0) };
        assert!(cost.implausible(Duration::from_secs(1), 8));
        assert!(!cost.implausible(Duration::from_secs(20), 8));
    }

    #[test]
    fn a_peak_that_is_a_reason_does_not_satisfy_the_rule() {
        let peak = Peak::Unavailable("no timer here".to_owned());
        assert!(!peak.measured());
        assert_eq!(peak.bytes(), None);
        assert_eq!(peak.to_string(), "not measured, no timer here");
    }

    #[test]
    fn byte_counts_are_binary_and_to_two_places() {
        assert_eq!(bytes(512), "512 B");
        assert_eq!(bytes(1024), "1.00 KiB");
        assert_eq!(bytes(3 * 1024 * 1024), "3.00 MiB");
        assert_eq!(bytes(5 * 1024 * 1024 * 1024), "5.00 GiB");
    }

    #[test]
    fn the_answer_here_is_either_a_number_or_a_sentence_and_never_a_panic() {
        // This is deliberately not asserting a platform. It runs on a Mac in development and on
        // Linux in CI and the point of the type is that both answers are usable.
        match own_peak() {
            Peak::Bytes(n) => assert!(n > 0, "this process has resident memory"),
            Peak::Unavailable(why) => assert!(!why.is_empty()),
        }
    }

    #[test]
    fn finding_a_timer_either_works_or_explains_itself() {
        match Timer::find() {
            Ok(timer) => {
                assert!(timer.binary().is_file());
                assert!(matches!(timer.flavour(), "GNU" | "BSD"));
            }
            Err(why) => assert!(why.contains("time"), "{why}"),
        }
    }
}
