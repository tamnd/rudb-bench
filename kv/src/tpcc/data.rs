//! The initial database of the spec's section 2.2, and the load that puts it in an engine.
//!
//! Every unit of work, one table for one warehouse and for the four tables with a row a customer
//! or an order one district of one warehouse, draws from its own xoshiro256** stream, seeded by
//! SplitMix64 over `(seed, table, warehouse, district)`. So a unit makes the same rows whichever
//! thread makes it and in whatever order, and one warehouse can be made again alone.
//!
//! The load sends the rows as literal multi-row `INSERT` statements, a transaction a unit. That is
//! the one way that works the same on all four engines today, and it is not each engine's best
//! bulk path: the CSV corpus and `COPY` or `read_csv` of section 2.6 come later.

use std::fmt::Write as _;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use super::text::{INDEXES, SCHEMA, TABLES};
use crate::backend::{Backend, Failed, Session};
use crate::workload::Rng;

/// Items, which do not grow with the warehouses.
pub(crate) const ITEMS: u64 = 100_000;
/// Districts a warehouse.
pub(crate) const DISTRICTS: u64 = 10;
/// Customers a district, and orders a district at load.
pub(crate) const CUSTOMERS: u64 = 3_000;
/// The first order of a district that is not yet delivered at load.
pub(crate) const FIRST_NEW: u64 = 2_101;
/// What `d_next_o_id` starts at.
pub(crate) const NEXT_ORDER: u64 = CUSTOMERS + 1;
/// The fixed load timestamp of section 2.4.
pub(crate) const LOADED_AT: &str = "2026-01-01 00:00:00";

/// SplitMix64's output function over `x`.
pub(crate) fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The stream of one unit of work. `table` is the table's place in [`TABLES`], or a number past
/// them for a stream that is not a table's.
pub(crate) fn stream(seed: u64, table: u64, warehouse: u64, district: u64) -> Rng {
    let mut state = splitmix64(seed);
    for part in [table, warehouse, district] {
        state = splitmix64(state ^ part);
    }
    Rng::new(state)
}

/// A number from `low` to `high`, both included.
pub(crate) fn uniform(rng: &mut Rng, low: u64, high: u64) -> u64 {
    low + rng.below(high - low + 1)
}

/// The spec's `NURand(A, x, y)` with the run's or the load's constant `c`.
pub(crate) fn nurand(rng: &mut Rng, a: u64, x: u64, y: u64, c: u64) -> u64 {
    (((uniform(rng, 0, a) | uniform(rng, x, y)) + c) % (y - x + 1)) + x
}

/// `C_LOAD`, the constant the load builds last names with, which comes from the seed so that a
/// run without the load can find it again.
pub(crate) fn c_load(seed: u64) -> u64 {
    stream(seed, 100, 0, 0).below(256)
}

/// The last name of the number `n` from 0 to 999, its three digits as syllables.
pub(crate) fn last_name(n: u64) -> String {
    const SYLLABLES: [&str; 10] =
        ["BAR", "OUGHT", "ABLE", "PRI", "PRES", "ESE", "ANTI", "CALLY", "ATION", "EING"];
    let mut name = String::with_capacity(15);
    for digit in [n / 100, n / 10 % 10, n % 10] {
        name.push_str(SYLLABLES[digit as usize]);
    }
    name
}

/// A random alphanumeric string of `low` to `high` characters.
fn alphanumeric(rng: &mut Rng, low: u64, high: u64) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let length = uniform(rng, low, high);
    (0..length).map(|_| char::from(ALPHABET[rng.below(ALPHABET.len() as u64) as usize])).collect()
}

/// A random string of `length` digits.
fn digits(rng: &mut Rng, length: u64) -> String {
    (0..length).map(|_| char::from(b'0' + rng.below(10) as u8)).collect()
}

/// A zip code, four random digits and then `11111`.
fn zip(rng: &mut Rng) -> String {
    format!("{}11111", digits(rng, 4))
}

/// `i_data` and `s_data`, 26 to 50 characters, a tenth of them with `ORIGINAL` somewhere in them.
fn data(rng: &mut Rng) -> String {
    let mut text = alphanumeric(rng, 26, 50);
    if rng.below(10) == 0 {
        let at = rng.below(text.len() as u64 - 7) as usize;
        text.replace_range(at..at + 8, "ORIGINAL");
    }
    text
}

/// One value of a generated row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Lit {
    Int(i64),
    /// A fixed point number as an integer and the digits after the point.
    Fixed(i64, u32),
    Text(String),
    Null,
    /// The load timestamp.
    Stamp,
}

impl Lit {
    /// Writes the value as SQL.
    fn write(&self, out: &mut String) {
        match self {
            Self::Int(v) => {
                let _ = write!(out, "{v}");
            }
            Self::Fixed(v, scale) => out.push_str(&fixed(*v, *scale)),
            Self::Text(v) => {
                out.push('\'');
                out.push_str(&v.replace('\'', "''"));
                out.push('\'');
            }
            Self::Null => out.push_str("NULL"),
            Self::Stamp => {
                let _ = write!(out, "'{LOADED_AT}'");
            }
        }
    }
}

/// `v` with the point `scale` digits from the right, so `fixed(-1000, 2)` is `-10.00`.
pub(crate) fn fixed(v: i64, scale: u32) -> String {
    let unit = 10_i64.pow(scale);
    let sign = if v < 0 { "-" } else { "" };
    let magnitude = v.unsigned_abs();
    let unit = unit as u64;
    format!("{sign}{}.{:0width$}", magnitude / unit, magnitude % unit, width = scale as usize)
}

fn int(v: u64) -> Lit {
    Lit::Int(v as i64)
}

fn text(v: String) -> Lit {
    Lit::Text(v)
}

/// The rows of `item`.
pub(crate) fn item(seed: u64, emit: &mut dyn FnMut(&[Lit])) {
    let mut rng = stream(seed, 7, 0, 0);
    for id in 1..=ITEMS {
        let im = uniform(&mut rng, 1, 10_000);
        let name = alphanumeric(&mut rng, 14, 24);
        let price = uniform(&mut rng, 100, 10_000) as i64;
        let data = data(&mut rng);
        emit(&[int(id), int(im), text(name), Lit::Fixed(price, 2), text(data)]);
    }
}

/// A street, a second street, a city, a state and a zip, which warehouses and districts share.
fn address(rng: &mut Rng) -> [Lit; 5] {
    [
        text(alphanumeric(rng, 10, 20)),
        text(alphanumeric(rng, 10, 20)),
        text(alphanumeric(rng, 10, 20)),
        text(alphanumeric(rng, 2, 2)),
        text(zip(rng)),
    ]
}

/// The row of warehouse `w`.
pub(crate) fn warehouse(seed: u64, w: u64, emit: &mut dyn FnMut(&[Lit])) {
    let mut rng = stream(seed, 0, w, 0);
    let name = text(alphanumeric(&mut rng, 6, 10));
    let [street_1, street_2, city, state, zip] = address(&mut rng);
    let tax = Lit::Fixed(uniform(&mut rng, 0, 2_000) as i64, 4);
    emit(&[int(w), name, street_1, street_2, city, state, zip, tax, Lit::Fixed(30_000_000, 2)]);
}

/// The stock of warehouse `w`.
pub(crate) fn stock(seed: u64, w: u64, emit: &mut dyn FnMut(&[Lit])) {
    let mut rng = stream(seed, 8, w, 0);
    let mut row = Vec::with_capacity(17);
    for id in 1..=ITEMS {
        row.clear();
        row.extend([int(id), int(w), int(uniform(&mut rng, 10, 100))]);
        row.extend((0..10).map(|_| text(alphanumeric(&mut rng, 24, 24))));
        row.extend([int(0), int(0), int(0), text(data(&mut rng))]);
        emit(&row);
    }
}

/// The ten districts of warehouse `w`.
pub(crate) fn district(seed: u64, w: u64, emit: &mut dyn FnMut(&[Lit])) {
    let mut rng = stream(seed, 1, w, 0);
    for d in 1..=DISTRICTS {
        let name = text(alphanumeric(&mut rng, 6, 10));
        let [street_1, street_2, city, state, zip] = address(&mut rng);
        let tax = Lit::Fixed(uniform(&mut rng, 0, 2_000) as i64, 4);
        emit(&[
            int(d),
            int(w),
            name,
            street_1,
            street_2,
            city,
            state,
            zip,
            tax,
            Lit::Fixed(3_000_000, 2),
            int(NEXT_ORDER),
        ]);
    }
}

/// The customers of district `d` of warehouse `w`.
pub(crate) fn customer(seed: u64, w: u64, d: u64, emit: &mut dyn FnMut(&[Lit])) {
    let c_load = c_load(seed);
    let mut rng = stream(seed, 2, w, d);
    for c in 1..=CUSTOMERS {
        let first = text(alphanumeric(&mut rng, 8, 16));
        let number = if c <= 1_000 { c - 1 } else { nurand(&mut rng, 255, 0, 999, c_load) };
        let [street_1, street_2, city, state, zip] = address(&mut rng);
        let phone = text(digits(&mut rng, 16));
        let credit = if rng.below(10) == 0 { "BC" } else { "GC" };
        let discount = Lit::Fixed(uniform(&mut rng, 0, 5_000) as i64, 4);
        let data = text(alphanumeric(&mut rng, 300, 500));
        emit(&[
            int(c),
            int(d),
            int(w),
            first,
            text("OE".to_string()),
            text(last_name(number)),
            street_1,
            street_2,
            city,
            state,
            zip,
            phone,
            Lit::Stamp,
            text(credit.to_string()),
            Lit::Fixed(5_000_000, 2),
            discount,
            Lit::Fixed(-1_000, 2),
            Lit::Fixed(1_000, 2),
            int(1),
            int(0),
            data,
        ]);
    }
}

/// The history of district `d` of warehouse `w`, a payment of 10.00 a customer.
pub(crate) fn history(seed: u64, w: u64, d: u64, emit: &mut dyn FnMut(&[Lit])) {
    let mut rng = stream(seed, 3, w, d);
    for c in 1..=CUSTOMERS {
        let data = text(alphanumeric(&mut rng, 12, 24));
        emit(&[int(c), int(d), int(w), int(d), int(w), Lit::Stamp, Lit::Fixed(1_000, 2), data]);
    }
}

/// The customer, the carrier and the line count of every order of a district at load, which
/// `orders` writes and `order_line` needs.
pub(crate) fn drawn_orders(seed: u64, w: u64, d: u64) -> Vec<(u64, Option<u64>, u64)> {
    let mut rng = stream(seed, 4, w, d);
    let mut customers: Vec<u64> = (1..=CUSTOMERS).collect();
    for at in (1..customers.len()).rev() {
        let other = rng.below(at as u64 + 1) as usize;
        customers.swap(at, other);
    }
    customers
        .into_iter()
        .enumerate()
        .map(|(at, c)| {
            let o = at as u64 + 1;
            let carrier = (o < FIRST_NEW).then(|| uniform(&mut rng, 1, 10));
            (c, carrier, uniform(&mut rng, 5, 15))
        })
        .collect()
}

/// The orders of district `d` of warehouse `w`.
pub(crate) fn orders(seed: u64, w: u64, d: u64, emit: &mut dyn FnMut(&[Lit])) {
    for (at, (c, carrier, lines)) in drawn_orders(seed, w, d).into_iter().enumerate() {
        let carrier = carrier.map_or(Lit::Null, int);
        emit(&[
            int(at as u64 + 1),
            int(d),
            int(w),
            int(c),
            Lit::Stamp,
            carrier,
            int(lines),
            int(1),
        ]);
    }
}

/// The undelivered orders of district `d` of warehouse `w`, the last 900.
pub(crate) fn new_order(w: u64, d: u64, emit: &mut dyn FnMut(&[Lit])) {
    for o in FIRST_NEW..=CUSTOMERS {
        emit(&[int(o), int(d), int(w)]);
    }
}

/// The order lines of district `d` of warehouse `w`.
pub(crate) fn order_line(seed: u64, w: u64, d: u64, emit: &mut dyn FnMut(&[Lit])) {
    let mut rng = stream(seed, 6, w, d);
    for (at, (_, _, lines)) in drawn_orders(seed, w, d).into_iter().enumerate() {
        let o = at as u64 + 1;
        for number in 1..=lines {
            let item = uniform(&mut rng, 1, ITEMS);
            let (delivered, amount) = if o < FIRST_NEW {
                (Lit::Stamp, 0)
            } else {
                (Lit::Null, uniform(&mut rng, 1, 999_999) as i64)
            };
            let info = text(alphanumeric(&mut rng, 24, 24));
            emit(&[
                int(o),
                int(d),
                int(w),
                int(number),
                int(item),
                int(w),
                delivered,
                int(5),
                Lit::Fixed(amount, 2),
                info,
            ]);
        }
    }
}

/// The order lines of `warehouses` warehouses at load, which the counting checks start from.
pub(crate) fn loaded_lines(seed: u64, warehouses: u64) -> u64 {
    let mut lines = 0;
    for w in 1..=warehouses {
        for d in 1..=DISTRICTS {
            lines += drawn_orders(seed, w, d).iter().map(|(_, _, n)| n).sum::<u64>();
        }
    }
    lines
}

/// One unit of the load: a table and the warehouse and district it is for.
#[derive(Debug, Clone, Copy)]
struct Unit {
    table: usize,
    w: u64,
    d: u64,
}

/// The units of table `table`.
fn units(table: usize, warehouses: u64) -> Vec<Unit> {
    match TABLES[table] {
        "item" => vec![Unit { table, w: 0, d: 0 }],
        "warehouse" | "district" | "stock" => {
            (1..=warehouses).map(|w| Unit { table, w, d: 0 }).collect()
        }
        _ => (1..=warehouses)
            .flat_map(|w| (1..=DISTRICTS).map(move |d| Unit { table, w, d }))
            .collect(),
    }
}

/// Makes the rows of a unit.
fn generate(seed: u64, unit: Unit, emit: &mut dyn FnMut(&[Lit])) {
    let Unit { w, d, .. } = unit;
    match TABLES[unit.table] {
        "warehouse" => warehouse(seed, w, emit),
        "district" => district(seed, w, emit),
        "customer" => customer(seed, w, d, emit),
        "history" => history(seed, w, d, emit),
        "orders" => orders(seed, w, d, emit),
        "new_order" => new_order(w, d, emit),
        "order_line" => order_line(seed, w, d, emit),
        "item" => item(seed, emit),
        _ => stock(seed, w, emit),
    }
}

/// Rows a statement of the load.
const ROWS_A_STATEMENT: usize = 500;

/// How many times a statement of the load is tried when the engine says another writer has it.
const LOAD_TRIES: u32 = 100;

/// Runs `sql`, trying again while the engine says to.
fn patiently(
    session: &mut dyn Session,
    what: &str,
    mut call: impl FnMut(&mut dyn Session) -> Result<(), Failed>,
) -> Result<(), String> {
    let mut tries = 0;
    loop {
        match call(session) {
            Ok(()) => return Ok(()),
            Err(Failed::Retry(_)) if tries < LOAD_TRIES => {
                tries += 1;
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(failed) => return Err(format!("{what} failed: {failed}")),
        }
    }
}

/// Loads one unit in one transaction and says how many rows it wrote.
fn load_unit(session: &mut dyn Session, seed: u64, unit: Unit) -> Result<u64, String> {
    let table = TABLES[unit.table];
    patiently(session, "BEGIN", |s| s.begin())?;
    let head = format!("INSERT INTO {table} VALUES ");
    let mut sql = head.clone();
    let mut in_statement = 0;
    let mut rows = 0;
    let mut failed = None;
    let flush = |sql: &mut String, session: &mut dyn Session| {
        let outcome = session.batch(sql).map_err(|f| format!("an INSERT into {table}: {f}"));
        sql.clear();
        sql.push_str(&head);
        outcome
    };
    generate(seed, unit, &mut |row: &[Lit]| {
        if failed.is_some() {
            return;
        }
        if in_statement > 0 {
            sql.push(',');
        }
        sql.push('(');
        for (at, value) in row.iter().enumerate() {
            if at > 0 {
                sql.push(',');
            }
            value.write(&mut sql);
        }
        sql.push(')');
        in_statement += 1;
        rows += 1;
        if in_statement == ROWS_A_STATEMENT {
            in_statement = 0;
            if let Err(message) = flush(&mut sql, session) {
                failed = Some(message);
            }
        }
    });
    if failed.is_none() && in_statement > 0 {
        failed = flush(&mut sql, session).err();
    }
    if let Some(message) = failed {
        let _ = session.rollback();
        return Err(message);
    }
    patiently(session, "COMMIT", |s| s.commit())?;
    Ok(rows)
}

/// What the load took, a table or an index at a time.
#[derive(Debug)]
pub(crate) struct Loaded {
    /// `(table, rows, took)`.
    pub(crate) tables: Vec<(&'static str, u64, Duration)>,
    /// `(index, took)`.
    pub(crate) indexes: Vec<(&'static str, Duration)>,
    pub(crate) create: Duration,
}

/// Creates the nine tables, loads them a table at a time over `sessions` sessions, and then
/// creates the two indexes.
pub(crate) fn load(
    backend: &dyn Backend,
    seed: u64,
    warehouses: u64,
    sessions: usize,
) -> Result<Loaded, String> {
    let mut first = backend.connect()?;
    let started = Instant::now();
    for (table, create) in TABLES.iter().zip(SCHEMA) {
        first
            .batch(&format!("DROP TABLE IF EXISTS {table}"))
            .and_then(|()| first.batch(create))
            .map_err(|failed| format!("creating {table} failed: {failed}"))?;
    }
    let create = started.elapsed();
    let mut connections = vec![first];
    for _ in 1..sessions {
        connections.push(backend.connect()?);
    }
    let mut tables = Vec::new();
    for (table, name) in TABLES.iter().enumerate() {
        let units = units(table, warehouses);
        let next = AtomicUsize::new(0);
        let started = Instant::now();
        let rows: Result<u64, String> = std::thread::scope(|scope| {
            let handles: Vec<_> = connections
                .iter_mut()
                .take(units.len())
                .map(|session| {
                    let (units, next) = (&units, &next);
                    scope.spawn(move || -> Result<u64, String> {
                        let mut rows = 0;
                        while let Some(&unit) = units.get(next.fetch_add(1, Ordering::Relaxed)) {
                            rows += load_unit(&mut **session, seed, unit)?;
                        }
                        Ok(rows)
                    })
                })
                .collect();
            handles.into_iter().try_fold(0, |sum, handle| {
                Ok(sum + handle.join().map_err(|_| "a loading session panicked".to_string())??)
            })
        });
        tables.push((*name, rows?, started.elapsed()));
    }
    let mut indexes = Vec::new();
    for (name, create) in INDEXES {
        let started = Instant::now();
        connections[0]
            .batch(create)
            .map_err(|failed| format!("creating index {name} failed: {failed}"))?;
        indexes.push((name, started.elapsed()));
    }
    Ok(Loaded { tables, indexes, create })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(make: impl FnOnce(&mut dyn FnMut(&[Lit]))) -> Vec<Vec<Lit>> {
        let mut rows = Vec::new();
        make(&mut |row: &[Lit]| rows.push(row.to_vec()));
        rows
    }

    fn texts(rows: &[Vec<Lit>], column: usize) -> Vec<String> {
        rows.iter()
            .map(|row| match &row[column] {
                Lit::Text(text) => text.clone(),
                other => panic!("{other:?} is not text"),
            })
            .collect()
    }

    #[test]
    fn names_are_the_syllables_of_their_digits() {
        assert_eq!(last_name(371), "PRICALLYOUGHT");
        assert_eq!(last_name(0), "BARBARBAR");
        assert_eq!(last_name(999), "EINGEINGEING");
    }

    #[test]
    fn a_unit_makes_the_same_rows_every_time() {
        let a = rows(|emit| customer(7, 2, 3, emit));
        assert_eq!(a, rows(|emit| customer(7, 2, 3, emit)));
        assert_ne!(a, rows(|emit| customer(7, 2, 4, emit)));
        assert_ne!(a, rows(|emit| customer(8, 2, 3, emit)));
    }

    #[test]
    fn every_name_is_in_the_first_thousand_customers_of_a_district() {
        let customers = rows(|emit| customer(1, 1, 1, emit));
        assert_eq!(customers.len(), CUSTOMERS as usize);
        let names = texts(&customers, 5);
        let mut first: Vec<_> = names[..1000].to_vec();
        first.sort();
        first.dedup();
        assert_eq!(first.len(), 1000);
        let bad = texts(&customers, 13).iter().filter(|credit| *credit == "BC").count();
        assert!((200..400).contains(&bad), "{bad}");
        assert!(texts(&customers, 10).iter().all(|zip| zip.len() == 9 && zip.ends_with("11111")));
    }

    #[test]
    fn a_tenth_of_the_items_are_original() {
        let items = rows(|emit| item(3, emit));
        assert_eq!(items.len(), ITEMS as usize);
        let data = texts(&items, 4);
        let original = data.iter().filter(|text| text.contains("ORIGINAL")).count();
        assert!((9_000..11_000).contains(&original), "{original}");
        assert!(data.iter().all(|text| (26..=50).contains(&text.len())));
    }

    #[test]
    fn orders_have_their_customers_once_and_their_lines() {
        let orders = rows(|emit| orders(5, 1, 2, emit));
        let mut customers: Vec<_> = orders.iter().map(|row| row[3].clone()).collect();
        customers.sort_by_key(|lit| match lit {
            Lit::Int(v) => *v,
            _ => -1,
        });
        assert_eq!(customers, (1..=3000).map(Lit::Int).collect::<Vec<_>>());
        let undelivered = orders.iter().filter(|row| row[5] == Lit::Null).count();
        assert_eq!(undelivered, 900);
        let lines: i64 = orders
            .iter()
            .map(|row| match row[6] {
                Lit::Int(n) => n,
                _ => 0,
            })
            .sum();
        assert_eq!(rows(|emit| order_line(5, 1, 2, emit)).len() as i64, lines);
        assert_eq!(rows(|emit| new_order(1, 2, emit)).len(), 900);
    }

    #[test]
    fn money_is_written_with_its_point() {
        assert_eq!(fixed(-1000, 2), "-10.00");
        assert_eq!(fixed(5, 4), "0.0005");
        assert_eq!(fixed(30_000_000, 2), "300000.00");
        let mut sql = String::new();
        Lit::Text("it's".to_string()).write(&mut sql);
        assert_eq!(sql, "'it''s'");
    }
}
