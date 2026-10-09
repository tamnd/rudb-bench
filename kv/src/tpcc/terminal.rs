//! One terminal: one thread, one session, its own deck and random stream, and the five
//! transactions of the spec's document 03, each sent statement by statement in that document's
//! order, retried by document 04 section 4.5's rules, and counted.

use std::fmt::{Display, Write as _};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use rudb_bench::histogram::Histogram;

use super::data::{self, CUSTOMERS, DISTRICTS, ITEMS, nurand, uniform};
use super::fresh::Acks;
use super::text::{Id, MAX_LINES, MIN_LINES, order_lines};
use crate::backend::{Backend, Failed, Session, Values};
use crate::workload::Rng;

/// The five transaction types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    NewOrder,
    Payment,
    OrderStatus,
    Delivery,
    StockLevel,
}

impl Kind {
    pub(crate) const COUNT: usize = 5;
    pub(crate) const ALL: [Self; Self::COUNT] =
        [Self::NewOrder, Self::Payment, Self::OrderStatus, Self::Delivery, Self::StockLevel];

    pub(crate) const fn index(self) -> usize {
        self as usize
    }

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::NewOrder => "new_order",
            Self::Payment => "payment",
            Self::OrderStatus => "order_status",
            Self::Delivery => "delivery",
            Self::StockLevel => "stock_level",
        }
    }

    /// How many of the hundred cards in the deck are this type.
    pub(crate) const fn cards(self) -> usize {
        match self {
            Self::NewOrder => 45,
            Self::Payment => 43,
            Self::OrderStatus | Self::Delivery | Self::StockLevel => 4,
        }
    }

    /// Whether the transaction writes, which decides how SQLite begins it.
    const fn writes(self) -> bool {
        !matches!(self, Self::OrderStatus | Self::StockLevel)
    }
}

/// The deck of a hundred cards of document 01 section 1.3, dealt in a shuffled order and shuffled
/// again from the terminal's stream each time it runs out.
#[derive(Debug)]
pub(crate) struct Deck {
    cards: Vec<Kind>,
    at: usize,
}

impl Deck {
    pub(crate) fn new() -> Self {
        let cards: Vec<Kind> =
            Kind::ALL.iter().flat_map(|kind| std::iter::repeat_n(*kind, kind.cards())).collect();
        let at = cards.len();
        Self { cards, at }
    }

    pub(crate) fn deal(&mut self, rng: &mut Rng) -> Kind {
        if self.at == self.cards.len() {
            for at in (1..self.cards.len()).rev() {
                let other = rng.below(at as u64 + 1) as usize;
                self.cards.swap(at, other);
            }
            self.at = 0;
        }
        self.at += 1;
        self.cards[self.at - 1]
    }
}

/// The run's NURand constants, which the header prints.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Constants {
    pub(crate) c_load: u64,
    pub(crate) c_run: u64,
    pub(crate) c_id: u64,
    pub(crate) c_item: u64,
}

impl Constants {
    /// `C_LOAD` from the seed as the load drew it, and `C_RUN` drawn so that its distance from
    /// `C_LOAD` is one the specification allows: 65 to 119, and not 96 or 112.
    pub(crate) fn new(seed: u64) -> Self {
        let c_load = data::c_load(seed);
        let mut rng = data::stream(seed, 101, 0, 0);
        let c_run = loop {
            let c = rng.below(256);
            let delta = c.abs_diff(c_load);
            if (65..=119).contains(&delta) && delta != 96 && delta != 112 {
                break c;
            }
        };
        Self { c_load, c_run, c_id: rng.below(1024), c_item: rng.below(8192) }
    }
}

/// Where a terminal works, document 04 section 4.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Home {
    first: u64,
    last: u64,
    /// The district its Stock-Level always reads.
    district: u64,
}

impl Home {
    /// With at least as many terminals as warehouses, terminal `t` has warehouse `1 + t mod W` and
    /// district `1 + (t div W) mod 10`. With fewer, it owns a contiguous slice of the warehouses
    /// and picks its warehouse in the slice for each transaction, and its district is
    /// `1 + t mod 10`.
    pub(crate) fn of(terminal: u64, terminals: u64, warehouses: u64) -> Self {
        if terminals >= warehouses {
            let w = 1 + terminal % warehouses;
            Self { first: w, last: w, district: 1 + (terminal / warehouses) % DISTRICTS }
        } else {
            Self {
                first: terminal * warehouses / terminals + 1,
                last: (terminal + 1) * warehouses / terminals,
                district: 1 + terminal % DISTRICTS,
            }
        }
    }

    fn warehouse(&self, rng: &mut Rng) -> u64 {
        if self.first == self.last { self.first } else { uniform(rng, self.first, self.last) }
    }
}

/// A customer picked by last name or by id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Customer {
    Id(u64),
    Name(String),
}

/// One line of a New-Order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Line {
    item: u64,
    supply: u64,
    quantity: u64,
}

/// A transaction's inputs, drawn once and kept for every retry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Inputs {
    NewOrder { w: u64, d: u64, c: u64, lines: Vec<Line> },
    Payment { w: u64, d: u64, c_w: u64, c_d: u64, customer: Customer, cents: i64 },
    OrderStatus { w: u64, d: u64, customer: Customer },
    Delivery { w: u64, carrier: u64 },
    StockLevel { w: u64, d: u64, threshold: u64 },
}

/// A customer by last name 60% of the time and by id 40%.
fn customer(rng: &mut Rng, constants: &Constants) -> Customer {
    if rng.below(100) < 60 {
        Customer::Name(data::last_name(nurand(rng, 255, 0, 999, constants.c_run)))
    } else {
        Customer::Id(nurand(rng, 1023, 1, CUSTOMERS, constants.c_id))
    }
}

/// A warehouse other than `w`, uniformly.
fn other(rng: &mut Rng, w: u64, warehouses: u64) -> u64 {
    let other = 1 + rng.below(warehouses - 1);
    if other >= w { other + 1 } else { other }
}

/// Draws the inputs of a transaction of type `kind`, document 03 sections 3.2 to 3.6.
pub(crate) fn draw(
    kind: Kind,
    rng: &mut Rng,
    home: &Home,
    warehouses: u64,
    constants: &Constants,
) -> Inputs {
    let w = home.warehouse(rng);
    match kind {
        Kind::NewOrder => {
            let d = uniform(rng, 1, DISTRICTS);
            let c = nurand(rng, 1023, 1, CUSTOMERS, constants.c_id);
            let count = uniform(rng, MIN_LINES as u64, MAX_LINES as u64);
            let rollback = rng.below(100) == 0;
            let mut lines: Vec<Line> = (0..count)
                .map(|_| {
                    let item = nurand(rng, 8191, 1, ITEMS, constants.c_item);
                    let remote = warehouses > 1 && rng.below(100) == 0;
                    let supply = if remote { other(rng, w, warehouses) } else { w };
                    Line { item, supply, quantity: uniform(rng, 1, 10) }
                })
                .collect();
            if rollback {
                lines.last_mut().expect("an order has lines").item = ITEMS + 1;
            }
            Inputs::NewOrder { w, d, c, lines }
        }
        Kind::Payment => {
            let d = uniform(rng, 1, DISTRICTS);
            let (c_w, c_d) = if warehouses > 1 && rng.below(100) < 15 {
                (other(rng, w, warehouses), uniform(rng, 1, DISTRICTS))
            } else {
                (w, d)
            };
            let customer = customer(rng, constants);
            Inputs::Payment { w, d, c_w, c_d, customer, cents: uniform(rng, 100, 500_000) as i64 }
        }
        Kind::OrderStatus => {
            let d = uniform(rng, 1, DISTRICTS);
            Inputs::OrderStatus { w, d, customer: customer(rng, constants) }
        }
        Kind::Delivery => Inputs::Delivery { w, carrier: uniform(rng, 1, 10) },
        Kind::StockLevel => {
            Inputs::StockLevel { w, d: home.district, threshold: uniform(rng, 10, 20) }
        }
    }
}

/// The date of the day `days` after 1970-01-01, Howard Hinnant's `civil_from_days`.
fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + i64::from(month <= 2), month, day)
}

/// The driver's wall clock as a timestamp, the same text for every engine.
pub(crate) fn stamp(at: SystemTime) -> String {
    let since = at.duration_since(UNIX_EPOCH).unwrap_or_default();
    let seconds = since.as_secs();
    let (year, month, day) = civil((seconds / 86_400) as i64);
    let second = seconds % 86_400;
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02}.{:06}",
        second / 3_600,
        second / 60 % 60,
        second % 60,
        since.subsec_micros()
    )
}

/// A number the engine returned as text, as an integer scaled by `10^scale`. Through a float,
/// because SQLite hands back a decimal column as whichever of an integer or a double it stored.
/// Every amount in TPC-C is far below where a double stops being exact in cents.
pub(crate) fn scaled(text: &str, scale: i32) -> Option<i64> {
    let value: f64 = text.trim().parse().ok()?;
    Some((value * 10_f64.powi(scale)).round() as i64)
}

/// What a committed transaction did, which the counting checks of document 05 section 5.2 add up.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Effects {
    /// Order lines written.
    pub(crate) lines: u64,
    /// The sum of their quantities.
    pub(crate) quantity: u64,
    /// The lines supplied by another warehouse.
    pub(crate) remote: u64,
    /// Orders delivered.
    pub(crate) delivered: u64,
    /// Districts a Delivery found empty.
    pub(crate) skipped: u64,
}

impl Effects {
    pub(crate) fn add(&mut self, other: &Self) {
        self.lines += other.lines;
        self.quantity += other.quantity;
        self.remote += other.remote;
        self.delivered += other.delivered;
        self.skipped += other.skipped;
    }
}

/// How a transaction ended without an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ended {
    Committed(Effects),
    /// The New-Order whose last item does not exist, rolled back on purpose.
    RolledBack,
}

/// The statements of one session: every [`Id`], and `no8` for each line count.
#[derive(Debug)]
pub(crate) struct Prepared {
    statements: Vec<usize>,
    lines: Vec<usize>,
}

/// Prepares every statement of the five transactions on a session.
pub(crate) fn prepare(
    session: &mut dyn Session,
    backend: &dyn Backend,
) -> Result<Prepared, String> {
    let mut statements = Vec::with_capacity(Id::ALL.len());
    for id in Id::ALL {
        let statement = session
            .prepare(&backend.numbered(id.text()))
            .map_err(|message| format!("preparing {} failed: {message}", id.name()))?;
        statements.push(statement);
    }
    let mut lines = Vec::new();
    for count in MIN_LINES..=MAX_LINES {
        let statement = session
            .prepare(&backend.numbered(&order_lines(count)))
            .map_err(|message| format!("preparing no8 of {count} lines failed: {message}"))?;
        lines.push(statement);
    }
    Ok(Prepared { statements, lines })
}

/// A session with its statements, and the buffers each call reuses.
pub(crate) struct Runner<'s> {
    session: Box<dyn Session + 's>,
    prepared: Prepared,
    serializable: bool,
    out: Values,
    texts: Vec<String>,
    /// Where each New-Order's acknowledgement is noted, when analytic streams judge freshness.
    acks: Option<&'s Acks>,
}

impl std::fmt::Debug for Runner<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runner").field("prepared", &self.prepared).finish_non_exhaustive()
    }
}

impl<'s> Runner<'s> {
    pub(crate) fn new(
        session: Box<dyn Session + 's>,
        prepared: Prepared,
        serializable: bool,
    ) -> Self {
        Self {
            session,
            prepared,
            serializable,
            out: Values::default(),
            texts: Vec::new(),
            acks: None,
        }
    }

    /// Notes each New-Order's acknowledgement in `acks`.
    pub(crate) fn acking(mut self, acks: Option<&'s Acks>) -> Self {
        self.acks = acks;
        self
    }

    fn begin(&mut self, kind: Kind) -> Result<(), Failed> {
        if self.serializable {
            self.session.begin_serializable(kind.writes())
        } else if kind.writes() {
            self.session.begin()
        } else {
            self.session.begin_read()
        }
    }

    fn execute(&mut self, statement: usize, parameters: &[&dyn Display]) -> Result<u64, Failed> {
        if self.texts.len() < parameters.len() {
            self.texts.resize(parameters.len(), String::new());
        }
        for (text, value) in self.texts.iter_mut().zip(parameters) {
            text.clear();
            let _ = write!(text, "{value}");
        }
        let texts: Vec<&str> = self.texts[..parameters.len()].iter().map(String::as_str).collect();
        self.out.clear();
        self.session.execute(statement, &texts, &mut self.out)
    }

    fn run(&mut self, id: Id, parameters: &[&dyn Display]) -> Result<u64, Failed> {
        self.execute(self.prepared.statements[id.index()], parameters)
    }

    /// Runs a statement that must return at least one row of `columns` columns.
    fn row(&mut self, id: Id, parameters: &[&dyn Display], columns: usize) -> Result<(), Failed> {
        self.run(id, parameters)?;
        if self.out.len() < columns {
            return Err(Failed::Error(format!(
                "{} returned {} values and not a row of {columns}",
                id.name(),
                self.out.len()
            )));
        }
        Ok(())
    }

    fn text(&self, at: usize) -> &str {
        std::str::from_utf8(self.out.get(at)).unwrap_or("")
    }

    fn number(&self, id: Id, at: usize, scale: i32) -> Result<i64, Failed> {
        scaled(self.text(at), scale).ok_or_else(|| {
            Failed::Error(format!(
                "{} returned {:?} where a number belongs",
                id.name(),
                self.text(at)
            ))
        })
    }

    /// Runs one transaction once, from `BEGIN` to `COMMIT`. A failure leaves the transaction open
    /// and the caller rolls it back.
    fn attempt(&mut self, inputs: &Inputs, stamp: &str) -> Result<Ended, Failed> {
        match inputs {
            Inputs::NewOrder { w, d, c, lines } => self.new_order(*w, *d, *c, lines, stamp),
            Inputs::Payment { w, d, c_w, c_d, customer, cents } => {
                self.payment(*w, *d, *c_w, *c_d, customer, *cents, stamp)
            }
            Inputs::OrderStatus { w, d, customer } => self.order_status(*w, *d, customer),
            Inputs::Delivery { w, carrier } => self.delivery(*w, *carrier, stamp),
            Inputs::StockLevel { w, d, threshold } => self.stock_level(*w, *d, *threshold),
        }
    }

    /// Document 03 section 3.2.
    fn new_order(
        &mut self,
        w: u64,
        d: u64,
        c: u64,
        lines: &[Line],
        stamp: &str,
    ) -> Result<Ended, Failed> {
        self.begin(Kind::NewOrder)?;
        self.row(Id::No1, &[&w, &d], 2)?;
        let o_id = self.number(Id::No1, 0, 0)?;
        let d_tax = self.number(Id::No1, 1, 4)?;
        self.row(Id::No2, &[&w], 1)?;
        let w_tax = self.number(Id::No2, 0, 4)?;
        self.row(Id::No3, &[&w, &d, &c], 3)?;
        let discount = self.number(Id::No3, 0, 4)?;
        let all_local = u8::from(lines.iter().all(|line| line.supply == w));
        self.run(Id::No4, &[&o_id, &d, &w, &c, &stamp, &lines.len(), &all_local])?;
        self.run(Id::No5, &[&o_id, &d, &w])?;
        let mut values: Vec<String> = Vec::with_capacity(lines.len() * 9);
        let mut effects = Effects::default();
        let mut total = 0_i64;
        for (at, line) in lines.iter().enumerate() {
            self.run(Id::No6, &[&line.item])?;
            if self.out.len() == 0 {
                self.session.rollback()?;
                return Ok(Ended::RolledBack);
            }
            if self.out.len() < 3 {
                return Err(Failed::Error(format!("no6 returned {} values", self.out.len())));
            }
            let price = self.number(Id::No6, 0, 2)?;
            let original = self.text(2).contains("ORIGINAL");
            let remote = u8::from(line.supply != w);
            self.row(Id::No7, &[&line.supply, &line.item, &line.quantity, &remote, &d], 3)?;
            let brand = if original && self.text(1).contains("ORIGINAL") { 'B' } else { 'G' };
            std::hint::black_box(brand);
            let amount = price * line.quantity as i64;
            total += amount;
            values.extend([
                o_id.to_string(),
                d.to_string(),
                w.to_string(),
                (at + 1).to_string(),
                line.item.to_string(),
                line.supply.to_string(),
                line.quantity.to_string(),
                data::fixed(amount, 2),
                self.text(2).to_string(),
            ]);
            effects.lines += 1;
            effects.quantity += line.quantity;
            effects.remote += u64::from(remote);
        }
        let parameters: Vec<&dyn Display> = values.iter().map(|v| v as &dyn Display).collect();
        self.execute(self.prepared.lines[lines.len() - MIN_LINES], &parameters)?;
        self.session.commit()?;
        if let (Some(acks), Ok(o_id)) = (self.acks, u64::try_from(o_id)) {
            acks.acked(w, d, o_id, Instant::now());
        }
        // The order total of the terminal's output, in ten-thousandths of cents times cents.
        let total =
            i128::from(total) * i128::from(10_000 - discount) * i128::from(10_000 + w_tax + d_tax)
                / 100_000_000;
        std::hint::black_box(total);
        Ok(Ended::Committed(effects))
    }

    /// Document 03 section 3.3.
    #[allow(clippy::too_many_arguments)]
    fn payment(
        &mut self,
        w: u64,
        d: u64,
        c_w: u64,
        c_d: u64,
        customer: &Customer,
        cents: i64,
        stamp: &str,
    ) -> Result<Ended, Failed> {
        let amount = data::fixed(cents, 2);
        self.begin(Kind::Payment)?;
        self.row(Id::Pa1, &[&w, &amount], 6)?;
        let w_name = self.text(0).to_string();
        self.row(Id::Pa2, &[&w, &d, &amount], 6)?;
        let d_name = self.text(0).to_string();
        let c_id = match customer {
            Customer::Id(c) => *c as i64,
            Customer::Name(name) => {
                self.run(Id::Pa3, &[&c_w, &c_d, name])?;
                let found = self.out.len();
                if found == 0 {
                    return Err(Failed::Error(format!("pa3 found no customer named {name}")));
                }
                self.number(Id::Pa3, found.div_ceil(2) - 1, 0)?
            }
        };
        self.row(Id::Pa4, &[&c_w, &c_d, &c_id, &amount], 14)?;
        if self.text(10) == "BC" {
            let data = format!("{c_id} {c_d} {c_w} {d} {w} {amount}");
            self.run(Id::Pa5, &[&c_w, &c_d, &c_id, &data])?;
        }
        let h_data = format!("{w_name}    {d_name}");
        self.run(Id::Pa6, &[&c_id, &c_d, &c_w, &d, &w, &stamp, &amount, &h_data])?;
        self.session.commit()?;
        Ok(Ended::Committed(Effects::default()))
    }

    /// Document 03 section 3.4.
    fn order_status(&mut self, w: u64, d: u64, customer: &Customer) -> Result<Ended, Failed> {
        self.begin(Kind::OrderStatus)?;
        let c_id = match customer {
            Customer::Id(c) => {
                self.row(Id::Os2, &[&w, &d, c], 4)?;
                *c as i64
            }
            Customer::Name(name) => {
                self.run(Id::Os1, &[&w, &d, name])?;
                let found = self.out.len() / 5;
                if found == 0 {
                    return Err(Failed::Error(format!("os1 found no customer named {name}")));
                }
                self.number(Id::Os1, (found.div_ceil(2) - 1) * 5, 0)?
            }
        };
        self.row(Id::Os3, &[&w, &d, &c_id], 3)?;
        let o_id = self.number(Id::Os3, 0, 0)?;
        self.run(Id::Os4, &[&w, &d, &o_id])?;
        std::hint::black_box(self.out.len());
        self.session.commit()?;
        Ok(Ended::Committed(Effects::default()))
    }

    /// Document 03 section 3.5, all ten districts in one transaction.
    fn delivery(&mut self, w: u64, carrier: u64, stamp: &str) -> Result<Ended, Failed> {
        /// How many times `de1` is run again for a district whose oldest order another Delivery
        /// took first.
        const RERUNS: u32 = 10;
        self.begin(Kind::Delivery)?;
        let mut effects = Effects::default();
        for d in 1..=DISTRICTS {
            let mut reruns = 0;
            let o_id = loop {
                self.run(Id::De1, &[&w, &d])?;
                if self.out.len() > 0 {
                    break Some(self.number(Id::De1, 0, 0)?);
                }
                self.run(Id::De5, &[&w, &d])?;
                if self.out.len() == 0 || self.out.get(0).is_empty() {
                    break None;
                }
                reruns += 1;
                if reruns > RERUNS {
                    return Err(Failed::Retry(format!(
                        "de1 deleted nothing {RERUNS} times in district {d} of warehouse {w}"
                    )));
                }
            };
            let Some(o_id) = o_id else {
                effects.skipped += 1;
                continue;
            };
            self.row(Id::De2, &[&w, &d, &o_id, &carrier], 1)?;
            let c_id = self.number(Id::De2, 0, 0)?;
            self.run(Id::De3, &[&w, &d, &o_id, &stamp])?;
            let mut cents = 0;
            for at in 0..self.out.len() {
                cents += self.number(Id::De3, at, 2)?;
            }
            self.run(Id::De4, &[&w, &d, &c_id, &data::fixed(cents, 2)])?;
            effects.delivered += 1;
        }
        self.session.commit()?;
        Ok(Ended::Committed(effects))
    }

    /// Document 03 section 3.6.
    fn stock_level(&mut self, w: u64, d: u64, threshold: u64) -> Result<Ended, Failed> {
        self.begin(Kind::StockLevel)?;
        self.row(Id::Sl1, &[&w, &d], 1)?;
        let next = self.number(Id::Sl1, 0, 0)?;
        self.row(Id::Sl2, &[&w, &d, &next, &threshold], 1)?;
        std::hint::black_box(self.number(Id::Sl2, 0, 0)?);
        self.session.commit()?;
        Ok(Ended::Committed(Effects::default()))
    }

    /// `SELECT sum(d_next_o_id) FROM district`, outside a transaction, for the monitor.
    pub(crate) fn orders(&mut self) -> Result<i64, String> {
        self.row(Id::Orders, &[], 1).map_err(|failed| format!("the monitor failed: {failed}"))?;
        self.number(Id::Orders, 0, 0).map_err(|failed| failed.to_string())
    }
}

/// Attempts a transaction gets before it counts as failed.
const ATTEMPTS: u64 = 100;

/// The longest backoff between two attempts.
const MOST_BACKOFF: Duration = Duration::from_millis(10);

/// What one terminal counted.
#[derive(Debug, Clone)]
pub(crate) struct Tally {
    /// Commits over the whole run, warmup included, which the counting checks compare against.
    pub(crate) committed: [u64; Kind::COUNT],
    /// Intended rollbacks over the whole run.
    pub(crate) rolled_back: [u64; Kind::COUNT],
    /// Transactions that gave up or hit an error, over the whole run.
    pub(crate) failed: [u64; Kind::COUNT],
    /// What the transactions whose commit returned inside the window did.
    pub(crate) window: [u64; Kind::COUNT],
    pub(crate) window_rolled_back: [u64; Kind::COUNT],
    pub(crate) window_attempts: [u64; Kind::COUNT],
    /// Service time from the first attempt's start to the final commit, for the window's commits.
    pub(crate) service: Vec<Histogram>,
    /// Commits by type in each second of the window.
    pub(crate) seconds: Vec<[u64; Kind::COUNT]>,
    pub(crate) effects: Effects,
    pub(crate) error: Option<String>,
}

impl Tally {
    pub(crate) fn new(seconds: usize) -> Self {
        Self {
            committed: [0; Kind::COUNT],
            rolled_back: [0; Kind::COUNT],
            failed: [0; Kind::COUNT],
            window: [0; Kind::COUNT],
            window_rolled_back: [0; Kind::COUNT],
            window_attempts: [0; Kind::COUNT],
            service: vec![Histogram::new(); Kind::COUNT],
            seconds: vec![[0; Kind::COUNT]; seconds],
            effects: Effects::default(),
            error: None,
        }
    }

    pub(crate) fn merge(&mut self, other: &Self) {
        for at in 0..Kind::COUNT {
            self.committed[at] += other.committed[at];
            self.rolled_back[at] += other.rolled_back[at];
            self.failed[at] += other.failed[at];
            self.window[at] += other.window[at];
            self.window_rolled_back[at] += other.window_rolled_back[at];
            self.window_attempts[at] += other.window_attempts[at];
            self.service[at].merge(&other.service[at]);
        }
        for (mine, theirs) in self.seconds.iter_mut().zip(&other.seconds) {
            for at in 0..Kind::COUNT {
                mine[at] += theirs[at];
            }
        }
        self.effects.add(&other.effects);
        if self.error.is_none() {
            self.error.clone_from(&other.error);
        }
    }
}

/// When the run starts and when its window opens and closes. The window opens when the monitor's
/// first read of `sum(d_next_o_id)` has returned, not at a set instant, so the driver and the
/// database count New-Orders over the same span however late the monitor gets to run.
#[derive(Debug)]
pub(crate) struct Clock {
    start: Instant,
    window: Duration,
    /// Nanoseconds after `start` that the window opened, `u64::MAX` while it is still shut.
    opened: AtomicU64,
}

impl Clock {
    pub(crate) fn new(start: Instant, window: Duration) -> Self {
        Self { start, window, opened: AtomicU64::new(u64::MAX) }
    }

    /// Opens the window at `at`.
    pub(crate) fn open(&self, at: Instant) {
        let nanos = u64::try_from(at.saturating_duration_since(self.start).as_nanos());
        self.opened.store(nanos.unwrap_or(u64::MAX - 1), Ordering::Release);
    }

    /// When the window opened, if it has.
    pub(crate) fn measured(&self) -> Option<Instant> {
        match self.opened.load(Ordering::Acquire) {
            u64::MAX => None,
            nanos => Some(self.start + Duration::from_nanos(nanos)),
        }
    }

    /// When the window closes, if it has opened.
    pub(crate) fn end(&self) -> Option<Instant> {
        self.measured().map(|measured| measured + self.window)
    }
}

/// One terminal's whole run.
#[derive(Debug)]
pub(crate) struct Terminal<'s> {
    pub(crate) runner: Runner<'s>,
    pub(crate) rng: Rng,
    /// Backoff draws come from their own stream, so a retry does not change the inputs that follow.
    pub(crate) backoff: Rng,
    pub(crate) deck: Deck,
    pub(crate) home: Home,
    pub(crate) warehouses: u64,
    pub(crate) constants: Constants,
}

impl Terminal<'_> {
    /// Runs transactions until the window closes or another terminal hit an error, and counts.
    pub(crate) fn run(mut self, clock: &Clock, seconds: usize, stop: &AtomicBool) -> Tally {
        let mut tally = Tally::new(seconds);
        while !stop.load(Ordering::Relaxed) && clock.end().is_none_or(|end| Instant::now() < end) {
            let kind = self.deck.deal(&mut self.rng);
            let inputs = draw(kind, &mut self.rng, &self.home, self.warehouses, &self.constants);
            let started = Instant::now();
            let at = stamp(SystemTime::now());
            let mut attempts = 0;
            let outcome = loop {
                attempts += 1;
                match self.runner.attempt(&inputs, &at) {
                    Ok(ended) => break Ok(ended),
                    Err(failed) => {
                        let _ = self.runner.session.rollback();
                        match failed {
                            Failed::Retry(_) if attempts < ATTEMPTS => self.wait(attempts),
                            Failed::Retry(_) => break Err(None),
                            Failed::Error(message) => {
                                break Err(Some(format!("{}: {message}", kind.name())));
                            }
                        }
                    }
                }
            };
            let finished = Instant::now();
            let k = kind.index();
            let measured = clock.measured();
            let inside = measured
                .is_some_and(|measured| finished >= measured && finished < measured + clock.window);
            if inside {
                tally.window_attempts[k] += attempts;
            }
            match outcome {
                Ok(Ended::Committed(effects)) => {
                    tally.committed[k] += 1;
                    tally.effects.add(&effects);
                    if inside {
                        tally.window[k] += 1;
                        let took =
                            u64::try_from((finished - started).as_nanos()).unwrap_or(u64::MAX);
                        tally.service[k].record(took);
                        if let Some(measured) = measured {
                            let second = (finished - measured).as_secs() as usize;
                            if let Some(counts) = tally.seconds.get_mut(second) {
                                counts[k] += 1;
                            }
                        }
                    }
                }
                Ok(Ended::RolledBack) => {
                    tally.rolled_back[k] += 1;
                    if inside {
                        tally.window_rolled_back[k] += 1;
                    }
                }
                Err(None) => tally.failed[k] += 1,
                Err(Some(message)) => {
                    tally.failed[k] += 1;
                    tally.error = Some(message);
                    stop.store(true, Ordering::Relaxed);
                }
            }
        }
        tally
    }

    /// Waits before the next attempt: nothing before the first retry, then a random time up to
    /// `2^k × 100µs` before the `k`th, never more than 10ms.
    fn wait(&mut self, retry: u64) {
        if retry < 2 {
            return;
        }
        let most = Duration::from_micros(100 << retry.min(20)).min(MOST_BACKOFF);
        std::thread::sleep(most.mul_f64(self.backoff.next_f64()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_hundred_cards_are_the_mix() {
        let mut deck = Deck::new();
        let mut rng = Rng::new(1);
        for _ in 0..3 {
            let mut counts = [0; Kind::COUNT];
            for _ in 0..100 {
                counts[deck.deal(&mut rng).index()] += 1;
            }
            assert_eq!(counts, [45, 43, 4, 4, 4]);
        }
    }

    #[test]
    fn the_run_constant_keeps_its_distance_from_the_load_constant() {
        for seed in 0..200 {
            let constants = Constants::new(seed);
            let delta = constants.c_run.abs_diff(constants.c_load);
            assert!((65..=119).contains(&delta) && delta != 96 && delta != 112, "{seed}");
            assert!(constants.c_id < 1024 && constants.c_item < 8192);
        }
    }

    #[test]
    fn terminals_share_warehouses_or_own_slices() {
        assert_eq!(Home::of(0, 8, 1), Home { first: 1, last: 1, district: 1 });
        assert_eq!(Home::of(5, 8, 2), Home { first: 2, last: 2, district: 3 });
        let slices: Vec<_> = (0..3).map(|t| Home::of(t, 3, 10)).collect();
        assert_eq!(
            slices.iter().map(|h| (h.first, h.last)).collect::<Vec<_>>(),
            [(1, 3), (4, 6), (7, 10)]
        );
    }

    #[test]
    fn inputs_follow_the_rules() {
        let constants = Constants::new(9);
        let home = Home::of(0, 1, 1);
        let mut rng = Rng::new(3);
        let mut rolled_back = 0;
        for _ in 0..10_000 {
            match draw(Kind::NewOrder, &mut rng, &home, 1, &constants) {
                Inputs::NewOrder { w, lines, .. } => {
                    assert_eq!(w, 1);
                    assert!((MIN_LINES..=MAX_LINES).contains(&lines.len()));
                    assert!(lines.iter().all(|line| line.supply == 1));
                    rolled_back += usize::from(lines.last().unwrap().item > ITEMS);
                }
                other => panic!("{other:?}"),
            }
        }
        assert!((50..150).contains(&rolled_back), "{rolled_back}");
        let mut remote = 0;
        for _ in 0..10_000 {
            if let Inputs::Payment { w, c_w, cents, .. } =
                draw(Kind::Payment, &mut rng, &home, 4, &constants)
            {
                remote += usize::from(c_w != w);
                assert!((100..=500_000).contains(&cents));
            }
        }
        assert!((1_200..1_800).contains(&remote), "{remote}");
    }

    #[test]
    fn stamps_are_civil_time() {
        let at = UNIX_EPOCH + Duration::from_micros(20_454 * 86_400_000_000 + 3_723_000_042);
        assert_eq!(stamp(at), "2026-01-01 01:02:03.000042");
        assert_eq!(
            stamp(UNIX_EPOCH + Duration::from_secs(951_782_400)),
            "2000-02-29 00:00:00.000000"
        );
    }

    #[test]
    fn numbers_come_back_scaled() {
        assert_eq!(scaled("-10", 2), Some(-1_000));
        assert_eq!(scaled("1224.56", 2), Some(122_456));
        assert_eq!(scaled("0.0005", 4), Some(5));
        assert_eq!(scaled("4999.990000000001", 2), Some(499_999));
        assert_eq!(scaled("", 2), None);
    }
}
