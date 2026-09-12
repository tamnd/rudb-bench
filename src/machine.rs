//! What the machine was, recorded rather than assumed.
//!
//! Section 15.4 ends with measurement hygiene: frequency pinned where possible, turbo state
//! recorded, page cache dropped before cold runs, filesystem and mount options recorded, and every
//! one of those facts printed in the result artifact rather than assumed. The last clause is the
//! one that does the work. A harness that assumes the governor was `performance` reports a twelve
//! percent regression on the morning somebody's laptop went into low power mode, and the two days
//! spent bisecting it are the reason this module exists.
//!
//! Everything here is best effort and everything that could not be read says so in words. A field
//! that reads "not readable here" is a fact about the machine. A field that silently defaulted is a
//! lie that survives into a report.

use std::path::Path;
use std::process::Command;

use crate::memory::Timer;

/// One line of the machine record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fact {
    /// What it is.
    pub name: &'static str,
    /// What it was, or why it could not be read.
    pub value: String,
    /// Whether the value is a reading rather than an explanation of a missing reading.
    pub known: bool,
}

impl Fact {
    /// A fact that was read.
    fn known(name: &'static str, value: impl Into<String>) -> Self {
        Self { name, value: value.into(), known: true }
    }

    /// A fact that was not.
    fn unknown(name: &'static str, why: impl Into<String>) -> Self {
        Self { name, value: why.into(), known: false }
    }
}

/// Probe the machine.
///
/// `scratch` is the directory the benchmark data would land in, because the filesystem that matters
/// is the one under the data and not the one under the binary, and on most of the fleet those are
/// different devices.
#[must_use]
pub fn probe(scratch: &Path) -> Vec<Fact> {
    vec![
        host(),
        os(),
        cpu(),
        threads(),
        memory(),
        governor(),
        turbo(),
        filesystem(scratch),
        page_cache(),
        timer(),
    ]
}

/// The name it answers to.
fn host() -> Fact {
    read_command("hostname", &[]).map_or_else(
        || Fact::unknown("host", "hostname did not answer"),
        |name| Fact::known("host", name),
    )
}

/// Kernel and architecture, as the system reports itself.
fn os() -> Fact {
    read_command("uname", &["-srm"])
        .map_or_else(|| Fact::unknown("os", "uname did not answer"), |name| Fact::known("os", name))
}

/// The processor, by whichever of the two places has it.
fn cpu() -> Fact {
    if let Some(line) = field_in_file("/proc/cpuinfo", "model name") {
        return Fact::known("cpu", line);
    }
    if let Some(brand) = read_command("sysctl", &["-n", "machdep.cpu.brand_string"]) {
        return Fact::known("cpu", brand);
    }
    Fact::unknown("cpu", "no /proc/cpuinfo and no sysctl brand string")
}

/// Hardware threads, which is the number that matters for a parallel engine.
fn threads() -> Fact {
    if let Ok(text) = std::fs::read_to_string("/proc/cpuinfo") {
        let n = text.lines().filter(|l| l.starts_with("processor")).count();
        if n > 0 {
            return Fact::known("threads", n.to_string());
        }
    }
    if let Some(n) = read_command("sysctl", &["-n", "hw.logicalcpu"]) {
        return Fact::known("threads", n);
    }
    Fact::unknown("threads", "neither /proc/cpuinfo nor hw.logicalcpu answered")
}

/// Hardware threads as a number, for the code that has to do arithmetic with it.
///
/// [`std::thread::available_parallelism`] rather than the `/proc/cpuinfo` count above, because it
/// takes the affinity mask and the cgroup quota into account and those are the ceiling that
/// actually applies. The two disagree inside a container, and inside a container the smaller one is
/// right. One when the system will not say, which makes the CPU plausibility check strict rather
/// than absent, and a strict check that fires is easier to notice than a check that quietly did
/// not run.
#[must_use]
pub fn threads_here() -> usize {
    std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get)
}

/// The one minute load average, where the system publishes one.
///
/// This exists because the fleet is shared. Three of the four machines have other people's work on
/// them at any hour, and a suite measured while a compiler is using every core is a measurement of
/// the compiler. That has already happened here often enough to throw a full sweep away over, and
/// the failure mode is the bad one: the table comes out looking exactly like a table, every column
/// is inflated by a different amount depending on how much of it was CPU bound, and there is
/// nothing in the artifact that says so. So the load is read before and after every engine's suite
/// and travels with the result, and [`crate::report::publishable`] refuses a run taken while the
/// machine was busy.
///
/// One minute rather than five or fifteen, because the useful question is what was happening during
/// this run rather than during the afternoon. `None` on a system with no `/proc/loadavg`, which
/// includes macOS, and a missing reading is reported as missing rather than as zero.
///
/// The reading taken before a suite is the one that answers the question. The one taken after it
/// counts the engine's own threads, and an engine using every core is the thing being measured
/// rather than a reason to distrust the measurement. See [`crate::report::SuiteResult::foreign_load`].
#[must_use]
pub fn load_now() -> Option<f64> {
    let text = std::fs::read_to_string("/proc/loadavg").ok()?;
    text.split_whitespace().next()?.parse().ok()
}

/// Wait for the machine to go quiet, when the run asked to be measured on a quiet one.
///
/// `RUDB_BENCH_SETTLE` is the one minute load average to wait for, and nothing waits without it. A
/// harness that refuses a result for a reason sixty seconds of patience would have removed is a
/// harness that wastes an afternoon per sweep, and the reason is usually not somebody else at all:
/// the one minute average decays over a minute, so a sweep that runs seven engines back to back
/// reads the tail of its own previous column as foreign work.
///
/// Returns what it waited for, so the caller can say so. `None` where there is no load to read, and
/// where nothing was asked for. Gives up after an hour and returns the reading anyway rather than
/// sitting there forever, because a machine that has been busy for an hour is not about to stop and
/// the run should go ahead and be refused with the number attached.
pub fn settle(say: impl Fn(&str)) -> Option<f64> {
    let want: f64 = std::env::var("RUDB_BENCH_SETTLE").ok()?.trim().parse().ok()?;
    let mut waited = 0;
    loop {
        let now = load_now()?;
        if now <= want || waited >= 60 {
            if waited > 0 {
                say(&format!("starting at load {now:.2} after waiting {waited} minutes"));
            }
            return Some(now);
        }
        say(&format!("waiting for the machine, load is {now:.2} and this run wants {want:.2}"));
        std::thread::sleep(std::time::Duration::from_secs(60));
        waited += 1;
    }
}

/// Whether this run was asked to make its cold runs actually cold.
///
/// Off unless `RUDB_BENCH_DROP_CACHES` is set to something other than `0` or `no`, which is the
/// wrong default for a benchmark harness and the right one for this fleet. Dropping the page cache
/// throws away every other process's working set, and three of the four machines have somebody
/// else's build on them at any hour. Turning it on for a run that is going to be published is one
/// environment variable, and the report says which of the two it was either way, so nothing silently
/// claims a cold number it did not take.
#[must_use]
pub fn forcing_cold() -> bool {
    match std::env::var("RUDB_BENCH_DROP_CACHES") {
        Ok(set) => !matches!(set.trim(), "" | "0" | "no"),
        Err(_) => false,
    }
}

/// Drop the page cache, so that the next read of the data comes off the device.
///
/// `sync` first, because dirty pages cannot be dropped and a write that has not reached the device
/// stays in memory and gets read back out of it. Then `3` into `drop_caches`, which is the page
/// cache plus the dentry and inode caches, and it is the whole set because the cost of opening and
/// stat-ing a file is part of what a first read costs.
///
/// # Errors
///
/// When there is no `drop_caches`, which is every system that is not Linux, or when this process
/// cannot write to it, which is every user that is not root. Both come back as a sentence rather
/// than as a silent warm run under a cold heading.
pub fn drop_caches() -> Result<(), String> {
    let synced = Command::new("sync").status().map_err(|e| format!("sync did not run: {e}"))?;
    if !synced.success() {
        return Err("sync failed, so dirty pages would have stayed in memory".to_owned());
    }
    std::fs::write("/proc/sys/vm/drop_caches", "3\n")
        .map_err(|e| format!("cannot write to /proc/sys/vm/drop_caches: {e}"))
}

/// What to call this machine in a file that gets committed.
///
/// The fleet name when the hostname belongs to a machine in [`crate::fleet::FLEET`], the hostname
/// otherwise, and `RUDB_BENCH_MACHINE` ahead of both. The override exists for CI, where the
/// hostname is a different random string on every run and the useful name is the runner image. It
/// is not a way to pretend one machine is another: the committed record carries whatever this
/// returned, the scheduled check refuses to compare two records whose names differ, and somebody
/// who lies to it here is lying to a file with their name on the commit.
///
/// The fleet lookup is here because the ledger is keyed by this string and rule seven turns a name
/// that moves into two series that may not be compared. Three of the four machines print a hostname
/// nobody would recognise, so before this the only thing standing between a run and a row filed
/// under `vmi3391933` was remembering to export a variable.
///
/// `unknown` when the system will not say, which is a name like any other and sorts to one place,
/// rather than an empty string that would silently match the next machine that also would not say.
#[must_use]
pub fn name_here() -> String {
    if let Some(named) = std::env::var_os("RUDB_BENCH_MACHINE") {
        let named = named.to_string_lossy().trim().to_owned();
        if !named.is_empty() {
            return named;
        }
    }
    let Some(host) = read_command("hostname", &[]) else {
        return "unknown".to_owned();
    };
    crate::fleet::name_for(&host, under_wsl()).map_or(host, ToOwned::to_owned)
}

/// Whether this is a Linux guest under WSL rather than the Windows side of the same desktop.
///
/// The two are one machine to the person sitting at it and two machines to this harness, because
/// the guest sees 31 of the 63 GiB and its /tmp is a tmpfs. `/proc/version` carries `microsoft` in
/// the kernel string there and nowhere else, which is the same test everything else uses.
fn under_wsl() -> bool {
    std::fs::read_to_string("/proc/version")
        .is_ok_and(|text| text.to_ascii_lowercase().contains("microsoft"))
}

/// Installed memory.
fn memory() -> Fact {
    if let Some(line) = field_in_file("/proc/meminfo", "MemTotal") {
        let kib = line.split_whitespace().next().and_then(|n| n.parse::<u64>().ok());
        if let Some(kib) = kib {
            return Fact::known("memory", crate::memory::bytes(kib * 1024));
        }
    }
    if let Some(n) = read_command("sysctl", &["-n", "hw.memsize"]) {
        if let Ok(n) = n.parse::<u64>() {
            return Fact::known("memory", crate::memory::bytes(n));
        }
    }
    Fact::unknown("memory", "neither MemTotal nor hw.memsize answered")
}

/// The frequency policy, which is the single most common explanation of a regression that is not
/// one.
fn governor() -> Fact {
    match std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor") {
        Ok(text) => Fact::known("governor", text.trim().to_owned()),
        Err(_) => Fact::unknown("governor", "no cpufreq here, the policy is not ours to see"),
    }
}

/// Turbo, recorded rather than controlled, because on most of the fleet it is not ours to control.
fn turbo() -> Fact {
    match std::fs::read_to_string("/sys/devices/system/cpu/intel_pstate/no_turbo") {
        Ok(text) if text.trim() == "1" => Fact::known("turbo", "off"),
        Ok(_) => Fact::known("turbo", "on"),
        Err(_) => Fact::unknown("turbo", "no intel_pstate here, the state is not ours to see"),
    }
}

/// The filesystem under the data, and how it was mounted.
fn filesystem(scratch: &Path) -> Fact {
    // The scratch directory usually does not exist yet when the record is taken, and `df` on a path
    // that is not there answers nothing. The nearest ancestor that does exist is on the same device
    // the directory would be created on, which is the fact being recorded.
    let mut at = scratch;
    while !at.exists() {
        match at.parent() {
            Some(parent) => at = parent,
            None => return Fact::unknown("filesystem", "no part of the scratch path exists"),
        }
    }
    let Some(device) = read_command("df", &["-P", &at.display().to_string()]) else {
        return Fact::unknown("filesystem", "df did not answer for the scratch directory");
    };
    let Some(device) = device.lines().nth(1).and_then(|l| l.split_whitespace().next()) else {
        return Fact::unknown("filesystem", "df said nothing about the scratch directory");
    };

    if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
        for line in mounts.lines() {
            let mut parts = line.split_whitespace();
            if parts.next() == Some(device) {
                let rest: Vec<&str> = parts.collect();
                return Fact::known("filesystem", format!("{device} {}", rest.join(" ")));
            }
        }
    }
    if let Some(mounted) = read_command("mount", &[]) {
        for line in mounted.lines() {
            if line.starts_with(device) {
                return Fact::known("filesystem", line.trim().to_owned());
            }
        }
    }
    Fact::known("filesystem", format!("{device}, mount options not readable here"))
}

/// Whether a cold run on this machine is actually cold.
///
/// Dropping the page cache needs root, and a harness that said cold when it could not drop it would
/// be reporting a warm number under a cold heading, which is worse than reporting no cold number.
fn page_cache() -> Fact {
    let path = Path::new("/proc/sys/vm/drop_caches");
    if !path.exists() {
        return Fact::unknown("page cache", "no /proc/sys/vm/drop_caches, cold cannot be forced");
    }
    match std::fs::OpenOptions::new().write(true).open(path) {
        Ok(_) => Fact::known("page cache", "droppable, cold runs are cold"),
        Err(e) => Fact::unknown("page cache", format!("cannot open drop_caches: {e}")),
    }
}

/// Which timer will be reading peak resident memory.
fn timer() -> Fact {
    match Timer::find() {
        Ok(timer) => {
            Fact::known("timer", format!("{} {}", timer.binary().display(), timer.flavour()))
        }
        Err(why) => Fact::unknown("timer", why),
    }
}

/// Read the first line of a command's output, or nothing.
fn read_command(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    if text.is_empty() { None } else { Some(text) }
}

/// Pull one colon separated field out of a `/proc` file.
fn field_in_file(path: &str, name: &str) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix(name) {
            let rest = rest.trim_start_matches([' ', '\t', ':']).trim();
            if !rest.is_empty() {
                return Some(rest.to_owned());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{Fact, field_in_file, probe, read_command};

    #[test]
    fn every_field_is_either_a_reading_or_a_sentence() {
        let facts = probe(&std::env::temp_dir());
        assert_eq!(facts.len(), 10);
        for fact in &facts {
            assert!(!fact.value.is_empty(), "{} said nothing at all", fact.name);
        }
    }

    #[test]
    fn the_machine_this_runs_on_at_least_knows_its_own_cpu_and_memory() {
        let facts = probe(&std::env::temp_dir());
        let named = |name: &str| facts.iter().find(|f| f.name == name).unwrap().clone();
        assert!(named("cpu").known, "{}", named("cpu").value);
        assert!(named("memory").known, "{}", named("memory").value);
        assert!(named("threads").known, "{}", named("threads").value);
    }

    #[test]
    fn a_command_that_does_not_exist_is_none_rather_than_a_panic() {
        assert!(read_command("this-is-not-a-program-anybody-has", &[]).is_none());
    }

    #[test]
    fn a_field_in_a_file_that_is_not_there_is_none() {
        assert!(field_in_file("/proc/definitely-not-a-file", "MemTotal").is_none());
    }

    #[test]
    fn an_unknown_fact_still_carries_a_reason_somebody_can_act_on() {
        let fact = Fact::unknown("governor", "no cpufreq here");
        assert!(!fact.known);
        assert!(fact.value.contains("cpufreq"));
    }
}
