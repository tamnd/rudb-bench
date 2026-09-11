//! The machines that actually exist, as opposed to the machines results get published on.
//!
//! Reporting rule seven says never compare across machines, and rule eight says any published
//! number is reproducible by one documented command on a named machine type. The named machine type
//! for ClickBench is `c6a.4xlarge`, because that is what the public board uses and comparability is
//! worth more than picking hardware that flatters us.
//!
//! **None of the machines in this file is a `c6a.4xlarge.`** Not one of them is close. So nothing
//! measured here is ever a headline number, is ever put in a README, or is ever compared against a
//! row of the recorded board. What they are for is the other job a benchmark does, which is telling
//! you on a Tuesday that the change you merged on Monday cost eight percent. That job wants the same
//! machine over time far more than it wants the right machine, and these are the same machines over
//! time.
//!
//! The distinction matters enough to be structural rather than a note in a README somebody stops
//! reading. A result carries the machine it came from, a machine says whether it is a reporting
//! machine, and a reporting machine is the only kind a published number is allowed to come from.

/// What a machine is allowed to be used for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Numbers from this machine may be published, because it is one of the named machine types in
    /// `spec/15-rudb-bench.md`. Nothing we own is in this category yet.
    Reporting,
    /// Numbers from this machine track the engine against itself over time. Useful for catching a
    /// regression, useless for comparing against DuckDB's published board.
    Regression,
    /// Correctness only. Too small or too contended for a timing to mean anything.
    Correctness,
}

impl Role {
    /// Whether a number measured here is allowed to leave the repository as a claim.
    #[must_use]
    pub fn may_publish(self) -> bool {
        matches!(self, Self::Reporting)
    }
}

/// A machine the project actually has access to.
///
/// The numbers are as probed on 10 September 2026 and they will drift, which is fine. They are here
/// to make the shape of the fleet obvious at a glance, not to be a configuration management system.
#[derive(Debug, Clone, Copy)]
pub struct Machine {
    /// The name it answers to over ssh.
    pub name: &'static str,
    /// What it may be used for.
    pub role: Role,
    /// Operating system, as it reports itself.
    pub os: &'static str,
    /// Processor, as it reports itself.
    pub cpu: &'static str,
    /// Physical cores.
    pub cores: u16,
    /// Hardware threads.
    pub threads: u16,
    /// Installed memory in gibibytes, rounded down.
    pub memory_gib: u16,
    /// Free space on the filesystem the data would land on, in gibibytes, when probed.
    pub free_disk_gib: u16,
    /// Whether a full ClickBench run of every engine happens on this machine.
    ///
    /// Separate from [`Role`] because they are separate questions and running them together is how
    /// a machine ends up chosen for a suite it cannot hold. A machine needs three things at once
    /// here: room for the largest engine's copy of `hits`, which is DuckDB's at about 20 GB, a
    /// load average low enough that a timing is a timing, and every engine in the comparison
    /// available on it at all. Exactly one machine has all three.
    pub clickbench: bool,
    /// What this machine is good for and what it is not.
    pub note: &'static str,
}

/// The development fleet.
///
/// Ordered by how much a timing from them is worth, most first.
pub const FLEET: &[Machine] = &[
    Machine {
        name: "gamingpc-wsl",
        role: Role::Regression,
        os: "Ubuntu 26.04 LTS under WSL2, kernel 6.18.33.2-microsoft-standard-WSL2",
        cpu: "Intel Core i9-13900K",
        cores: 24,
        threads: 32,
        memory_gib: 31,
        free_disk_gib: 748,
        clickbench: true,
        note: "The Linux side of the Windows desktop, and the only machine in the fleet that can \
               run a full ClickBench comparison at all. It is the only one with room for every \
               engine's copy of hits, the only Linux with ClickHouse, DuckDB, DataFusion and \
               Polars all installed, and it sits at load 1.14 where the other box with the disk \
               sits at load 35. Four things about it have to travel with any number from it. It \
               is a virtual machine, so the disk is a virtual disk on an NVMe and the memory is \
               31 of the host's 63 GiB rather than all of it. Its /tmp is a 16 GiB tmpfs, so a \
               scratch directory left on the default lands in RAM and the run either lies or dies. \
               The cores are heterogeneous, 8 performance and 16 efficiency, so a thread on the \
               wrong kind is a 2x outlier with nothing to do with the change under test. And the \
               host is a desktop somebody uses, so a run checks that it is idle rather than \
               assuming it.",
    },
    Machine {
        name: "gamingpc",
        role: Role::Regression,
        os: "Windows 11 Pro Insider Preview, 10.0.28120",
        cpu: "Intel Core i9-13900K",
        cores: 24,
        threads: 32,
        memory_gib: 63,
        free_disk_gib: 249,
        clickbench: false,
        note: "The same hardware as the row above, seen from the other operating system, and the \
               only Windows machine there is. It matters because the storage layer has to work \
               there and nobody discovers a Windows file locking problem on a Mac, so it runs the \
               suites that are about the engine rather than about the comparison. It does not run \
               a full ClickBench even though it has the disk for one, because ClickHouse has no \
               Windows build and a four engine ClickBench with the second fastest engine on the \
               board missing is not the comparison. The hazards of the row above apply here too, \
               since it is the same 8 performance and 16 efficiency cores and the same desktop \
               somebody uses.",
    },
    Machine {
        name: "server3",
        role: Role::Regression,
        os: "Ubuntu 24.04.4 LTS, kernel 6.8.0-106",
        cpu: "AMD EPYC, virtualized",
        cores: 8,
        threads: 8,
        memory_gib: 23,
        free_disk_gib: 138,
        clickbench: false,
        note: "The quietest Linux box, at load 2.56 when this was last probed, and the default for \
               the gate and for a regression run on a suite that fits. It is where the smoke \
               baseline comes from. The disk number has been 44, then 21, and is 138 today, which \
               is the argument for probing it rather than trusting the constant, and the reason \
               this table used to say ClickBench was out of reach here on disk alone. That is no \
               longer the constraint. What is, is that the only engines installed are DuckDB and \
               Polars, and a ClickBench row that is missing ClickHouse and DataFusion is not the \
               comparison the report claims to publish. Virtualized EPYC with no stated model \
               number means the frequency policy is not ours to control, which is a real source \
               of variance and is why anything from here is a median of five with the \
               interquartile range attached.",
    },
    Machine {
        name: "server2",
        role: Role::Correctness,
        os: "Ubuntu 24.04.4 LTS, kernel 6.8.0-136",
        cpu: "AMD EPYC, virtualized",
        cores: 6,
        threads: 6,
        memory_gib: 11,
        free_disk_gib: 44,
        clickbench: false,
        note: "Correctness and compatibility runs. 44 GiB free is enough for the source file and \
               one conversion of it and nothing like enough for four, and DuckDB is the only \
               other engine installed here, so it does not try ClickBench on either count. Useful \
               for the differential harness, which needs two engines and a lot of small queries \
               rather than one engine and a lot of data.",
    },
    Machine {
        name: "server1",
        role: Role::Correctness,
        os: "Ubuntu 24.04.4 LTS, kernel 6.8.0-101",
        cpu: "AMD EPYC, virtualized",
        cores: 4,
        threads: 4,
        memory_gib: 5,
        free_disk_gib: 185,
        clickbench: false,
        note: "A Kubernetes production worker node, and that is the first thing to know about it. \
               kubelet, containerd, cilium-agent, a temporal server, a harbor registry and an \
               otelcol are resident on it alongside a long running crawler, and its load average \
               on 10 September 2026 was 51.72, 75.13 and 70.80 on four cores, and two days later \
               the one minute figure was 35.30, so the number moves but never to anything like \
               idle. Nothing timed on a machine loaded eighteen times over is a timing, at any \
               sample count, and this file \
               says so because it did not, and a smoke run from here with interquartile ranges of \
               28 to 124 percent got read as a harness problem for a while. What is genuinely \
               useful is the 5 GiB of memory with about 2 free, which is the closest thing the \
               fleet has to the small machine case in the specification, and the resource claim in \
               spec/02-the-goal.md means most exactly where memory is scarce. An out of memory on \
               server1 is a real finding even though a timing from it is not. It also has the most \
               free disk of any Linux box, which makes it the place to keep a dataset the others \
               stream from and not a reason to run anything here.",
    },
];

/// The machine type published numbers come from, which nothing in [`FLEET`] is.
pub const REPORTING_MACHINE: &str = "c6a.4xlarge, 16 vCPU, 32 GiB, gp2";

#[cfg(test)]
mod tests {
    use super::FLEET;

    #[test]
    fn nothing_we_own_may_publish_a_number() {
        // The test that has to fail before somebody adds a reporting machine, so that adding one
        // is a conversation about whether it is really a c6a.4xlarge rather than a line in a
        // table. If this test breaks, the README's claim about where numbers come from broke too.
        assert!(FLEET.iter().all(|m| !m.role.may_publish()));
    }

    #[test]
    fn every_machine_says_what_it_is_not_good_for() {
        // A machine entry that only lists its strengths is how a 4 core box with 2 GiB free ends
        // up producing a number somebody quotes.
        for machine in FLEET {
            assert!(machine.note.len() > 120, "{} needs a real note", machine.name);
        }
    }

    #[test]
    fn the_machine_clickbench_runs_on_can_actually_hold_it() {
        // DuckDB's own hits file is 20.46 GB on the recorded board, and it is the largest of the
        // three engines that convert. The harness unloads each engine as soon as it is measured,
        // so a run needs the largest copy free rather than the sum, and this is the check that
        // the machine claiming it runs ClickBench has that much. It used to be asserted over
        // every regression machine, which passed for months and was wrong the whole time, because
        // server3 is a regression machine and had 21 GiB free when that was noticed. It has 138
        // today, which changes nothing about the assertion and is the reason the assertion is on
        // the flag rather than on the role.
        for machine in FLEET.iter().filter(|m| m.clickbench) {
            assert!(machine.free_disk_gib > 21, "{} cannot hold hits", machine.name);
        }
    }

    #[test]
    fn exactly_one_machine_runs_the_comparison() {
        // Not a style rule. If this becomes zero then there is nowhere to measure and the next
        // layer of the engine has no base number to be a ratio against, and if it becomes two
        // then somebody has to say which one the series continues on, because rule seven means a
        // ledger that changes machine halfway is two ledgers.
        assert_eq!(FLEET.iter().filter(|m| m.clickbench).count(), 1);
    }
}
