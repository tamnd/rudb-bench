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
    /// What this machine is good for and what it is not.
    pub note: &'static str,
}

/// The development fleet.
///
/// Ordered by how much a timing from them is worth, most first.
pub const FLEET: &[Machine] = &[
    Machine {
        name: "gamingpc",
        role: Role::Regression,
        os: "Windows 11 Pro Insider Preview, 10.0.28120",
        cpu: "Intel Core i9-13900K",
        cores: 24,
        threads: 32,
        memory_gib: 63,
        free_disk_gib: 249,
        note: "The only machine with the memory and the disk to hold ClickBench hits comfortably, \
               and the only one that can say anything about thread scaling. It is also the only \
               Windows machine, which matters because the storage layer has to work there and \
               nobody discovers a Windows file locking problem on a Mac. Two hazards. The cores \
               are heterogeneous, 8 performance and 16 efficiency, so a thread that lands on the \
               wrong kind is a 2x outlier that has nothing to do with the change under test, and \
               anything measured here pins its threads or reports the spread. It is also a \
               desktop somebody uses, so a run has to check that it is idle rather than assume it.",
    },
    Machine {
        name: "server3",
        role: Role::Regression,
        os: "Ubuntu 24.04.4 LTS, kernel 6.8.0-106",
        cpu: "AMD EPYC, virtualized",
        cores: 8,
        threads: 8,
        memory_gib: 23,
        free_disk_gib: 44,
        note: "The best of the Linux boxes and the default for a regression run, because Linux is \
               where the engine will mostly be deployed and because perf works here. 44 GiB free \
               is tight for ClickBench: hits is about 70 GB as TSV and DuckDB's own file is 20.46 \
               GB, so loading it means streaming from Parquet and deleting the intermediate rather \
               than keeping both. Virtualized EPYC with no stated model number means the frequency \
               policy is not ours to control, which is a real source of variance and is why \
               anything from here is a median of five with the interquartile range attached.",
    },
    Machine {
        name: "server2",
        role: Role::Correctness,
        os: "Ubuntu 24.04.4 LTS, kernel 6.8.0-136",
        cpu: "AMD EPYC, virtualized",
        cores: 6,
        threads: 6,
        memory_gib: 11,
        free_disk_gib: 25,
        note: "Correctness and compatibility runs. 25 GiB free will not hold ClickBench in any \
               format worth measuring, so it does not try. Useful for the differential harness, \
               which needs two engines and a lot of small queries rather than one engine and a lot \
               of data.",
    },
    Machine {
        name: "server1",
        role: Role::Correctness,
        os: "Ubuntu 24.04.4 LTS, kernel 6.8.0-101",
        cpu: "AMD EPYC, virtualized",
        cores: 4,
        threads: 4,
        memory_gib: 5,
        free_disk_gib: 114,
        note: "5 GiB of memory with about 2 free, which is the interesting thing about it. It is \
               the closest thing the fleet has to the small machine case in the specification, and \
               the resource claim in spec/02-the-goal.md means most exactly where memory is \
               scarce. An out of memory on server1 is a genuine finding even though a timing from \
               it is not. It also has the most free disk of any Linux box, which makes it the \
               place to keep a dataset that the others stream from.",
    },
];

/// The machine type published numbers come from, which nothing in [`FLEET`] is.
pub const REPORTING_MACHINE: &str = "c6a.4xlarge, 16 vCPU, 32 GiB, gp2";

#[cfg(test)]
mod tests {
    use super::{FLEET, Role};

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
    fn the_regression_machines_can_actually_hold_the_data() {
        // DuckDB's own hits file is 20.46 GB on the recorded board. A machine that cannot hold
        // that is not a machine a ClickBench regression run happens on, whatever its role says.
        for machine in FLEET.iter().filter(|m| m.role == Role::Regression) {
            assert!(machine.free_disk_gib > 21, "{} cannot hold hits", machine.name);
        }
    }
}
