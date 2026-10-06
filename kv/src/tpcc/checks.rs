//! The checks of the spec's document 05: the consistency conditions of section 5.1, which check
//! the database against itself, and the counting checks of section 5.2, which check it against what
//! the driver saw commit. Every one is SQL sent through the same backend as the run, and every one
//! must come out as expected.

use std::fmt::Write as _;
use std::time::Instant;

use super::data::{CUSTOMERS, DISTRICTS, NEXT_ORDER};
use super::terminal::scaled;
use crate::backend::{Backend, Session, Values};

/// The conditions, by number. A condition with two queries passes when both count nothing.
///
/// Condition 11 is left out: it needs the driver's deliveries a district in a temporary table.
const CONDITIONS: [(&str, &[&str]); 11] = [
    (
        "c1",
        &["SELECT count(*) FROM warehouse w
  JOIN (SELECT d_w_id, sum(d_ytd) AS s FROM district GROUP BY d_w_id) d ON d.d_w_id = w.w_id
 WHERE round(w.w_ytd * 100) <> round(d.s * 100)"],
    ),
    (
        "c2",
        &["SELECT count(*) FROM district d
  LEFT JOIN (SELECT o_w_id, o_d_id, max(o_id) AS m FROM orders GROUP BY o_w_id, o_d_id) o
    ON o.o_w_id = d.d_w_id AND o.o_d_id = d.d_id
  LEFT JOIN (SELECT no_w_id, no_d_id, max(no_o_id) AS m FROM new_order GROUP BY no_w_id, no_d_id) n
    ON n.no_w_id = d.d_w_id AND n.no_d_id = d.d_id
 WHERE o.m IS NULL OR n.m IS NULL OR d.d_next_o_id - 1 <> o.m OR d.d_next_o_id - 1 <> n.m"],
    ),
    (
        "c3",
        &["SELECT count(*) FROM
  (SELECT max(no_o_id) - min(no_o_id) + 1 AS span, count(*) AS n
     FROM new_order GROUP BY no_w_id, no_d_id) q
 WHERE q.span <> q.n"],
    ),
    (
        "c4",
        &["SELECT count(*) FROM
  (SELECT o_w_id, o_d_id, sum(o_ol_cnt) AS s FROM orders GROUP BY o_w_id, o_d_id) o
  FULL JOIN (SELECT ol_w_id, ol_d_id, count(*) AS n FROM order_line GROUP BY ol_w_id, ol_d_id) l
    ON l.ol_w_id = o.o_w_id AND l.ol_d_id = o.o_d_id
 WHERE o.s IS NULL OR l.n IS NULL OR o.s <> l.n"],
    ),
    (
        "c5",
        &[
            "SELECT count(*) FROM orders o
  LEFT JOIN new_order n ON n.no_w_id = o.o_w_id AND n.no_d_id = o.o_d_id AND n.no_o_id = o.o_id
 WHERE (o.o_carrier_id IS NULL) <> (n.no_o_id IS NOT NULL)",
            "SELECT count(*) FROM new_order n
  LEFT JOIN orders o ON o.o_w_id = n.no_w_id AND o.o_d_id = n.no_d_id AND o.o_id = n.no_o_id
 WHERE o.o_id IS NULL",
        ],
    ),
    (
        "c6",
        &["SELECT count(*) FROM orders o
  LEFT JOIN (SELECT ol_w_id, ol_d_id, ol_o_id, count(*) AS n FROM order_line
              GROUP BY ol_w_id, ol_d_id, ol_o_id) l
    ON l.ol_w_id = o.o_w_id AND l.ol_d_id = o.o_d_id AND l.ol_o_id = o.o_id
 WHERE l.n IS NULL OR l.n <> o.o_ol_cnt"],
    ),
    (
        "c7",
        &["SELECT count(*) FROM order_line l
  JOIN orders o ON o.o_w_id = l.ol_w_id AND o.o_d_id = l.ol_d_id AND o.o_id = l.ol_o_id
 WHERE (l.ol_delivery_d IS NULL) <> (o.o_carrier_id IS NULL)"],
    ),
    (
        "c8",
        &["SELECT count(*) FROM warehouse w
  LEFT JOIN (SELECT h_w_id, sum(h_amount) AS s FROM history GROUP BY h_w_id) h ON h.h_w_id = w.w_id
 WHERE h.s IS NULL OR round(w.w_ytd * 100) <> round(h.s * 100)"],
    ),
    (
        "c9",
        &["SELECT count(*) FROM district d
  LEFT JOIN (SELECT h_w_id, h_d_id, sum(h_amount) AS s FROM history GROUP BY h_w_id, h_d_id) h
    ON h.h_w_id = d.d_w_id AND h.h_d_id = d.d_id
 WHERE h.s IS NULL OR round(d.d_ytd * 100) <> round(h.s * 100)"],
    ),
    (
        "c10",
        &["SELECT count(*) FROM customer c
  LEFT JOIN (SELECT o.o_w_id, o.o_d_id, o.o_c_id, sum(l.ol_amount) AS s
               FROM orders o JOIN order_line l
                 ON l.ol_w_id = o.o_w_id AND l.ol_d_id = o.o_d_id AND l.ol_o_id = o.o_id
              WHERE l.ol_delivery_d IS NOT NULL
              GROUP BY o.o_w_id, o.o_d_id, o.o_c_id) dl
    ON dl.o_w_id = c.c_w_id AND dl.o_d_id = c.c_d_id AND dl.o_c_id = c.c_id
  LEFT JOIN (SELECT h_c_w_id, h_c_d_id, h_c_id, sum(h_amount) AS s FROM history
              GROUP BY h_c_w_id, h_c_d_id, h_c_id) h
    ON h.h_c_w_id = c.c_w_id AND h.h_c_d_id = c.c_d_id AND h.h_c_id = c.c_id
 WHERE round(c.c_balance * 100) <> round(coalesce(dl.s, 0) * 100) - round(coalesce(h.s, 0) * 100)"],
    ),
    (
        "c12",
        &["SELECT count(*) FROM customer c
  LEFT JOIN (SELECT o.o_w_id, o.o_d_id, o.o_c_id, sum(l.ol_amount) AS s
               FROM orders o JOIN order_line l
                 ON l.ol_w_id = o.o_w_id AND l.ol_d_id = o.o_d_id AND l.ol_o_id = o.o_id
              WHERE l.ol_delivery_d IS NOT NULL
              GROUP BY o.o_w_id, o.o_d_id, o.o_c_id) dl
    ON dl.o_w_id = c.c_w_id AND dl.o_d_id = c.c_d_id AND dl.o_c_id = c.c_id
 WHERE round((c.c_balance + c.c_ytd_payment) * 100) <> round(coalesce(dl.s, 0) * 100)"],
    ),
];

/// What the driver knows the database holds since the load, for the counting checks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Since {
    pub(crate) warehouses: u64,
    /// Order lines at load.
    pub(crate) loaded_lines: u64,
    /// Committed New-Orders, `N_no`.
    pub(crate) new_orders: u64,
    /// Their lines, quantities and remote lines, `L`, `Q` and `R`.
    pub(crate) lines: u64,
    pub(crate) quantity: u64,
    pub(crate) remote: u64,
    /// Committed Payments, `N_pay`.
    pub(crate) payments: u64,
    /// Orders delivered, `D`.
    pub(crate) delivered: u64,
}

/// The counting checks: a query and what it must return. Without `since` only the two that hold
/// whatever ran since the load.
fn counts(since: Option<&Since>) -> Vec<(&'static str, &'static str, i64)> {
    let mut checks = vec![
        (
            "payments",
            "SELECT (SELECT sum(c_payment_cnt) FROM customer) - (SELECT count(*) FROM history)",
            0,
        ),
        ("stock_range", "SELECT count(*) FROM stock WHERE s_quantity NOT BETWEEN 10 AND 100", 0),
    ];
    if let Some(since) = since {
        let orders = (since.warehouses * DISTRICTS * CUSTOMERS) as i64;
        let next = (since.warehouses * DISTRICTS * NEXT_ORDER) as i64;
        let n = |v: u64| v as i64;
        checks.extend([
            ("orders", "SELECT count(*) FROM orders", orders + n(since.new_orders)),
            ("next_order", "SELECT sum(d_next_o_id) FROM district", next + n(since.new_orders)),
            (
                "order_lines",
                "SELECT count(*) FROM order_line",
                n(since.loaded_lines) + n(since.lines),
            ),
            ("history", "SELECT count(*) FROM history", orders + n(since.payments)),
            ("deliveries", "SELECT sum(c_delivery_cnt) FROM customer", n(since.delivered)),
            ("stock_orders", "SELECT sum(s_order_cnt) FROM stock", n(since.lines)),
            ("stock_ytd", "SELECT sum(s_ytd) FROM stock", n(since.quantity)),
            ("stock_remote", "SELECT sum(s_remote_cnt) FROM stock", n(since.remote)),
        ]);
    }
    checks
}

/// Runs one query that returns one number, an empty sum being zero.
fn number(session: &mut dyn Session, sql: &str) -> Result<i64, String> {
    let statement = session.prepare(sql)?;
    let mut out = Values::default();
    session.execute(statement, &[], &mut out).map_err(|failed| failed.to_string())?;
    if out.len() == 0 || out.get(0).is_empty() {
        return Ok(0);
    }
    let text = String::from_utf8_lossy(out.get(0));
    scaled(&text, 0).ok_or_else(|| format!("{text:?} is not a number"))
}

/// Runs every check on a fresh session, writes a `check` line for each, and says whether all
/// passed.
pub(crate) fn run(
    backend: &dyn Backend,
    after: &str,
    since: Option<&Since>,
    report: &mut String,
) -> Result<bool, String> {
    let mut session = backend.connect()?;
    let mut passed = true;
    for (name, queries) in CONDITIONS {
        let started = Instant::now();
        let mut violations = 0;
        let mut error = None;
        for sql in queries {
            match number(&mut *session, sql) {
                Ok(count) => violations += count,
                Err(message) => error = Some(message),
            }
        }
        let took = started.elapsed().as_secs_f64();
        match error {
            Some(message) => {
                passed = false;
                let _ = writeln!(
                    report,
                    "check {name} after={after} result=error took_s={took:.3} error={message:?}"
                );
            }
            None => {
                let result = if violations == 0 { "pass" } else { "fail" };
                passed &= violations == 0;
                let _ = writeln!(
                    report,
                    "check {name} after={after} result={result} violations={violations} \
                     took_s={took:.3}"
                );
            }
        }
    }
    for (name, sql, expected) in counts(since) {
        let started = Instant::now();
        let got = number(&mut *session, sql);
        let took = started.elapsed().as_secs_f64();
        match got {
            Ok(got) => {
                let result = if got == expected { "pass" } else { "fail" };
                passed &= got == expected;
                let _ = writeln!(
                    report,
                    "check {name} after={after} result={result} expected={expected} got={got} \
                     took_s={took:.3}"
                );
            }
            Err(message) => {
                passed = false;
                let _ = writeln!(
                    report,
                    "check {name} after={after} result=error took_s={took:.3} error={message:?}"
                );
            }
        }
    }
    Ok(passed)
}
