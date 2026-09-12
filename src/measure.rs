//! Timings, and the statistics the reporting rules require of them.
//!
//! Reporting rule two says the median of at least five runs with the interquartile range, never a
//! minimum and never a single run. That rule is the difference between a measurement and a
//! screenshot, and it is easy to write down and easy to forget under deadline, so it is enforced
//! here rather than remembered: a [`Distribution`] built from fewer than five samples exists and is
//! printable and says no to [`Distribution::publishable`].
//!
//! ClickBench's own convention is the best of three, and rule two allows it explicitly because
//! comparability with the public board is worth more than rigour on that one suite. That is a
//! different number and it has a different constructor, [`Distribution::clickbench`], so a
//! ClickBench-convention number cannot be produced by accident and cannot be mixed into a table of
//! medians without somebody typing the word.

use std::time::Duration;

/// A set of timings for one thing, and what they say.
///
/// Percentiles are the nearest rank on the sorted samples, which is the definition that does not
/// invent a value that was never measured. With five samples the quartiles are the second and
/// fourth, and the interquartile range is the distance between them. Interpolating between samples
/// would give a smoother number and a less true one, and at these sample counts the smoothness
/// would be an illusion anyway.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Distribution {
    samples: Vec<Duration>,
    convention: Convention,
}

/// Which of the two allowed summaries this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Convention {
    /// Median of at least five, with the interquartile range. The default, and the only one a
    /// number outside a ClickBench table is allowed to use.
    Median,
    /// Best of three, which is what the ClickBench board does. Comparable to the board and not
    /// comparable to anything else, so it is labelled everywhere it appears.
    Clickbench,
}

impl Distribution {
    /// Summarize a set of runs by the median.
    ///
    /// # Panics
    ///
    /// When there are no samples at all. Zero runs is a bug in the caller rather than a result with
    /// a wide error bar, and returning an empty distribution would let it reach a table.
    #[must_use]
    pub fn median(samples: Vec<Duration>) -> Self {
        assert!(!samples.is_empty(), "a distribution of nothing is not a measurement");
        let mut samples = samples;
        samples.sort_unstable();
        Self { samples, convention: Convention::Median }
    }

    /// Summarize a set of runs the way the ClickBench board does, by the best.
    ///
    /// # Panics
    ///
    /// When there are no samples.
    #[must_use]
    pub fn clickbench(samples: Vec<Duration>) -> Self {
        assert!(!samples.is_empty(), "a distribution of nothing is not a measurement");
        let mut samples = samples;
        samples.sort_unstable();
        Self { samples, convention: Convention::Clickbench }
    }

    /// Which summary this is, for the label.
    #[must_use]
    pub const fn convention(&self) -> Convention {
        self.convention
    }

    /// How many runs went into it.
    #[must_use]
    pub fn runs(&self) -> usize {
        self.samples.len()
    }

    /// Every run, sorted.
    ///
    /// For the one caller that has to write the measurement down rather than summarize it. A saved
    /// run that kept only the median could never be recombined with another engine's run without
    /// losing the interquartile range, and rule two says the spread travels with the median.
    #[must_use]
    pub fn samples(&self) -> &[Duration] {
        &self.samples
    }

    /// The headline number, which depends on the convention.
    #[must_use]
    pub fn headline(&self) -> Duration {
        match self.convention {
            Convention::Median => self.percentile(50),
            Convention::Clickbench => self.samples[0],
        }
    }

    /// The median, whatever the convention says the headline is.
    #[must_use]
    pub fn median_of(&self) -> Duration {
        self.percentile(50)
    }

    /// The lower quartile.
    #[must_use]
    pub fn p25(&self) -> Duration {
        self.percentile(25)
    }

    /// The upper quartile.
    #[must_use]
    pub fn p75(&self) -> Duration {
        self.percentile(75)
    }

    /// The interquartile range, which is the number that says whether the median means anything.
    #[must_use]
    pub fn iqr(&self) -> Duration {
        self.p75().saturating_sub(self.p25())
    }

    /// The fastest run. Present for diagnosis and deliberately not the headline of anything except
    /// a ClickBench-convention number.
    #[must_use]
    pub fn fastest(&self) -> Duration {
        self.samples[0]
    }

    /// The slowest run.
    #[must_use]
    pub fn slowest(&self) -> Duration {
        self.samples[self.samples.len() - 1]
    }

    /// The spread as a fraction of the median, which is what a per-benchmark regression threshold
    /// is derived from per section 15.5.
    ///
    /// Returns `None` when the median is zero, which means the thing being timed is below the
    /// clock's resolution and no ratio computed from it would mean anything.
    #[must_use]
    pub fn relative_iqr(&self) -> Option<f64> {
        let median = self.median_of().as_secs_f64();
        if median <= 0.0 { None } else { Some(self.iqr().as_secs_f64() / median) }
    }

    /// Whether a number out of this is allowed to be published as a median.
    ///
    /// Five runs is the floor in rule two. A ClickBench-convention number is never publishable
    /// under this name, because it is a best of three and calling it a median would be a lie about
    /// the method rather than about the number.
    #[must_use]
    pub fn publishable(&self) -> bool {
        self.convention == Convention::Median && self.samples.len() >= 5
    }

    /// The sample at the nearest rank to a percentile.
    fn percentile(&self, p: usize) -> Duration {
        // Nearest rank: ceil(p/100 * n), clamped into the array. Integer arithmetic throughout so
        // that a percentile never lands one index off because of a float that was 2.9999999.
        let n = self.samples.len();
        let rank = (p * n).div_ceil(100).max(1);
        self.samples[rank - 1]
    }
}

/// Run something enough times to have a distribution, and separate the first run from the rest.
///
/// Rule four says cold and hot are reported separately and a hot number never appears without the
/// cold one next to it. The first run is not a sample of the same thing as the rest, so it is
/// returned on its own rather than averaged in, which is the mistake that makes a cold cost
/// disappear into a wide interquartile range.
///
/// The caller is responsible for whatever makes cold actually cold. Dropping the page cache needs
/// privileges this process does not assume it has, and a run that quietly did not drop it and said
/// cold anyway is worse than one that says it could not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Runs {
    /// The first run, on a state nobody warmed.
    pub cold: Duration,
    /// Every run after it.
    pub hot: Distribution,
}

impl Runs {
    /// Time `once` one cold run and `hot` hot runs.
    ///
    /// # Errors
    ///
    /// Whatever the closure returns, at the first run that fails. A benchmark that averages over
    /// the runs that happened to succeed is measuring the wrong thing.
    ///
    /// # Panics
    ///
    /// When asked for zero hot runs.
    pub fn collect<E>(hot: usize, mut once: impl FnMut() -> Result<(), E>) -> Result<Self, E> {
        assert!(hot > 0, "a hot distribution needs at least one run");
        let start = std::time::Instant::now();
        once()?;
        let cold = start.elapsed();

        let mut samples = Vec::with_capacity(hot);
        for _ in 0..hot {
            let start = std::time::Instant::now();
            once()?;
            samples.push(start.elapsed());
        }
        Ok(Self { cold, hot: Distribution::median(samples) })
    }
}

/// Format a duration the way every table in this harness formats one.
///
/// Milliseconds below a second and seconds above, both to three significant places, because a table
/// where one row is in microseconds and the next is in seconds is a table nobody reads correctly.
#[must_use]
pub fn show(duration: Duration) -> String {
    let seconds = duration.as_secs_f64();
    if seconds >= 1.0 {
        format!("{seconds:.3}s")
    } else if seconds >= 0.001 {
        format!("{:.3}ms", seconds * 1000.0)
    } else {
        format!("{:.3}us", seconds * 1_000_000.0)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Convention, Distribution, Runs, show};

    fn ms(values: &[u64]) -> Vec<Duration> {
        values.iter().copied().map(Duration::from_millis).collect()
    }

    #[test]
    fn the_median_of_five_is_the_third_one_sorted() {
        let d = Distribution::median(ms(&[50, 10, 30, 20, 40]));
        assert_eq!(d.median_of(), Duration::from_millis(30));
        assert_eq!(d.fastest(), Duration::from_millis(10));
        assert_eq!(d.slowest(), Duration::from_millis(50));
    }

    #[test]
    fn the_quartiles_are_samples_and_not_interpolations_between_them() {
        // Nearest rank on five samples puts p25 at the second and p75 at the fourth, so both are
        // numbers that were actually measured. An interpolating definition would report 17.5 and
        // 42.5 here, neither of which any run produced.
        let d = Distribution::median(ms(&[10, 20, 30, 40, 50]));
        assert_eq!(d.p25(), Duration::from_millis(20));
        assert_eq!(d.p75(), Duration::from_millis(40));
        assert_eq!(d.iqr(), Duration::from_millis(20));
    }

    #[test]
    fn four_runs_is_not_enough_to_publish_and_five_is() {
        assert!(!Distribution::median(ms(&[1, 2, 3, 4])).publishable());
        assert!(Distribution::median(ms(&[1, 2, 3, 4, 5])).publishable());
    }

    #[test]
    fn a_clickbench_number_is_the_best_of_the_runs_and_never_publishable_as_a_median() {
        let d = Distribution::clickbench(ms(&[30, 10, 20]));
        assert_eq!(d.headline(), Duration::from_millis(10));
        assert_eq!(d.convention(), Convention::Clickbench);
        // Even with fifty runs behind it. The bar is not the sample count, it is that a best of n
        // and a median are different claims.
        assert!(!Distribution::clickbench(ms(&[1; 50])).publishable());
    }

    #[test]
    fn the_relative_spread_is_none_when_the_thing_is_too_fast_to_time() {
        let d = Distribution::median(vec![Duration::ZERO; 5]);
        assert!(d.relative_iqr().is_none());
    }

    #[test]
    fn the_relative_spread_is_the_iqr_over_the_median() {
        let d = Distribution::median(ms(&[10, 20, 30, 40, 50]));
        let got = d.relative_iqr().unwrap();
        assert!((got - 20.0 / 30.0).abs() < 1e-9, "{got}");
    }

    #[test]
    fn the_first_run_is_kept_apart_from_the_rest() {
        let mut n = 0u32;
        let runs = Runs::collect::<()>(5, || {
            n += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(n, 6, "one cold run and five hot ones");
        assert_eq!(runs.hot.runs(), 5);
        assert!(runs.hot.publishable());
    }

    #[test]
    fn a_run_that_fails_stops_the_measurement_rather_than_being_skipped() {
        let mut n = 0u32;
        let got = Runs::collect::<&str>(5, || {
            n += 1;
            if n == 3 { Err("the engine fell over") } else { Ok(()) }
        });
        assert_eq!(got, Err("the engine fell over"));
    }

    #[test]
    fn durations_are_printed_in_a_unit_a_reader_can_compare() {
        assert_eq!(show(Duration::from_secs(2)), "2.000s");
        assert_eq!(show(Duration::from_millis(250)), "250.000ms");
        assert_eq!(show(Duration::from_micros(7)), "7.000us");
    }
}
