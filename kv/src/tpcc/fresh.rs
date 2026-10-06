//! Freshness, the spec's document 09 section 9.6: how long an order the driver was told had
//! committed stays out of the analytic snapshots, and how many orders a snapshot sees before the
//! driver is told they committed.
//!
//! The terminals note the order number and the time of each New-Order's acknowledgement, a list a
//! district. Before each query an analytic stream reads `d_next_o_id` of every district in the
//! query's own transaction, so the newest order a district shows is `d_next_o_id - 1`. The read
//! is sent at one instant and answered at a later one, and the snapshot is taken somewhere in
//! between, so an order counts as missed only if its acknowledgement came before the read was sent,
//! and as seen ahead only if it came after the answer, or had not come when the snapshot is judged.

use std::collections::BTreeMap;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use super::data::DISTRICTS;

/// How long an order is kept after a snapshot saw it, for a stream whose snapshot started before
/// its acknowledgement and is judged a little later.
const KEPT: Duration = Duration::from_secs(10);

/// The acknowledged New-Orders of every district that no snapshot has seen for long.
#[derive(Debug)]
pub(crate) struct Acks {
    districts: Vec<Mutex<District>>,
}

#[derive(Debug, Default)]
struct District {
    /// Every order up to it was seen by a snapshot more than [`KEPT`] ago, or was there before
    /// the run.
    floor: u64,
    /// The orders past `floor` acknowledged, by number.
    acked: BTreeMap<u64, Instant>,
}

/// What one snapshot saw against the acknowledgements.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Judged {
    /// From the acknowledgement of the oldest order acknowledged before the snapshot started and
    /// not seen in it, to the start, or zero when every one was seen.
    pub(crate) stale: Duration,
    /// Orders seen whose acknowledgement came after the snapshot started.
    pub(crate) ahead: u64,
    /// The most by which one of those came after it, at the least.
    pub(crate) lead: Duration,
}

impl Acks {
    pub(crate) fn new(warehouses: u64) -> Self {
        let districts = (0..warehouses * DISTRICTS).map(|_| Mutex::default()).collect();
        Self { districts }
    }

    fn district(&self, w: u64, d: u64) -> Option<&Mutex<District>> {
        let at = w.checked_sub(1)? * DISTRICTS + d.checked_sub(1)?;
        (d <= DISTRICTS).then(|| self.districts.get(at as usize)).flatten()
    }

    /// Notes that New-Order `o_id` of district `d` of warehouse `w` was acknowledged `at`.
    pub(crate) fn acked(&self, w: u64, d: u64, o_id: u64, at: Instant) {
        if let Some(district) = self.district(w, d) {
            district.lock().unwrap_or_else(PoisonError::into_inner).acked.insert(o_id, at);
        }
    }

    /// Takes `rows`, each district's `(d_w_id, d_id, d_next_o_id)` read before the run, as the
    /// orders there already.
    pub(crate) fn there(&self, rows: &[(u64, u64, u64)]) {
        for &(w, d, next) in rows {
            if let Some(district) = self.district(w, d) {
                let mut district = district.lock().unwrap_or_else(PoisonError::into_inner);
                district.floor = district.floor.max(next.saturating_sub(1));
            }
        }
    }

    /// Judges a snapshot that read `rows`, each district's `(d_w_id, d_id, d_next_o_id)`, from a
    /// read sent at `sent` and answered at `got`, at `now`. Orders seen more than [`KEPT`] before
    /// `sent` are let go.
    pub(crate) fn judge(
        &self,
        rows: &[(u64, u64, u64)],
        sent: Instant,
        got: Instant,
        now: Instant,
    ) -> Judged {
        let mut judged = Judged::default();
        let mut oldest: Option<Instant> = None;
        for &(w, d, next) in rows {
            let Some(district) = self.district(w, d) else { continue };
            let mut guard = district.lock().unwrap_or_else(PoisonError::into_inner);
            let district = &mut *guard;
            let seen = next.saturating_sub(1);
            for (_, &at) in district.acked.range(seen + 1..) {
                if at < sent && oldest.is_none_or(|oldest| at < oldest) {
                    oldest = Some(at);
                }
            }
            if seen > district.floor {
                let mut listed = 0;
                for (_, &at) in district.acked.range(district.floor + 1..=seen) {
                    listed += 1;
                    if at > got {
                        judged.ahead += 1;
                        judged.lead = judged.lead.max(at - got);
                    }
                }
                // Seen and not acknowledged yet, so acknowledged after the snapshot started.
                let unacknowledged = seen - district.floor - listed;
                if unacknowledged > 0 {
                    judged.ahead += unacknowledged;
                    judged.lead = judged.lead.max(now.saturating_duration_since(got));
                }
            }
            while let Some((&o_id, &at)) = district.acked.first_key_value() {
                if o_id > seen || at + KEPT >= sent {
                    break;
                }
                district.acked.pop_first();
                district.floor = district.floor.max(o_id);
            }
        }
        judged.stale = oldest.map_or(Duration::ZERO, |oldest| sent - oldest);
        judged
    }
}

/// Each district's `(d_w_id, d_id, d_next_o_id)` from the values of the district read, three a
/// row, or `None` when they are not that.
pub(crate) fn rows(values: &[&[u8]]) -> Option<Vec<(u64, u64, u64)>> {
    if values.len() % 3 != 0 {
        return None;
    }
    let number = |value: &[u8]| std::str::from_utf8(value).ok()?.trim().parse::<u64>().ok();
    values.chunks(3).map(|row| Some((number(row[0])?, number(row[1])?, number(row[2])?))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_snapshot_is_late_by_the_oldest_order_it_missed_and_ahead_by_what_it_saw_early() {
        let acks = Acks::new(2);
        acks.there(&[(1, 1, 3001), (1, 2, 3001), (2, 10, 3001)]);
        let base = Instant::now();
        let ms = |n: u64| base + Duration::from_millis(n);
        acks.acked(1, 1, 3001, ms(10));
        acks.acked(1, 1, 3002, ms(20));
        acks.acked(1, 2, 3001, ms(5));
        acks.acked(2, 10, 3001, ms(150));

        // Sent at 100 and answered at 110: 3002 of (1, 1) and 3001 of (1, 2) were missed, the
        // oldest acknowledged at 5. 3001 of (2, 10) was seen and acknowledged at 150, 40 after.
        let judged =
            acks.judge(&[(1, 1, 3002), (1, 2, 3001), (2, 10, 3002)], ms(100), ms(110), ms(200));
        assert_eq!(
            judged,
            Judged { stale: Duration::from_millis(95), ahead: 1, lead: Duration::from_millis(40) }
        );

        // Everything seen, and 3002 of (2, 10) seen though not acknowledged when judged.
        let judged =
            acks.judge(&[(1, 1, 3003), (1, 2, 3002), (2, 10, 3003)], ms(300), ms(310), ms(330));
        assert_eq!(
            judged,
            Judged { stale: Duration::ZERO, ahead: 1, lead: Duration::from_millis(20) }
        );
    }

    #[test]
    fn orders_seen_long_ago_are_let_go() {
        let acks = Acks::new(1);
        acks.there(&[(1, 1, 3001)]);
        let base = Instant::now();
        for o_id in 3001..3101 {
            acks.acked(1, 1, o_id, base);
        }
        let later = base + KEPT + Duration::from_secs(1);
        let judged = acks.judge(&[(1, 1, 3051)], later, later, later);
        assert_eq!(judged.stale, KEPT + Duration::from_secs(1));
        let district = acks.districts[0].lock().unwrap();
        assert_eq!((district.floor, district.acked.len()), (3050, 50));
    }

    #[test]
    fn the_district_read_is_three_numbers_a_row() {
        assert_eq!(
            rows(&[b"1", b"2", b"3001", b"1", b"3", b" 3005"]),
            Some(vec![(1, 2, 3001), (1, 3, 3005)])
        );
        assert_eq!(rows(&[b"0"]), None);
        assert_eq!(rows(&[b"1", b"x", b"3"]), None);
    }
}
