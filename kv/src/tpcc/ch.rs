//! CH-benCHmark, the spec's document 09: the 22 analytic queries over the TPC-C tables and the
//! three tables they add, run by analytic streams beside the terminals or on their own.
//!
//! The texts are BenchBase's, from revision [`BENCHBASE`], with `oorder` named `orders` as the
//! TPC-C mode names it and every date moved on by [`shifted`]. Nothing else is changed but the
//! name of the view Q15 reads, which is each stream's own.

use std::fmt::Write as _;
use std::io::Write as _;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use rudb_bench::histogram::Histogram;

use super::data::{self, LOADED_AT};
use super::fresh::{self, Acks, Judged};
use super::terminal::Clock;
use crate::backend::{Backend, Session, Values};
use crate::workload::Rng;

/// The BenchBase revision the texts and the rows of `region` and `nation` come from.
pub(crate) const BENCHBASE: &str = "33c00473807ebd49304d114a6d769d2d2b2bbb34";

/// The 22 queries as BenchBase writes them, Q15 being the `SELECT` over the view that
/// [`Q15_VIEW`] makes.
pub(crate) const QUERIES: [&str; 22] = [
    r#"SELECT ol_number, sum(ol_quantity) AS sum_qty, sum(ol_amount) AS sum_amount, avg(ol_quantity) AS avg_qty, avg(ol_amount) AS avg_amount, count(*) AS count_order FROM order_line WHERE ol_delivery_d > '2007-01-02 00:00:00.000000' GROUP BY ol_number ORDER BY ol_number"#,
    r#"SELECT su_suppkey, su_name, n_name, i_id, i_name, su_address, su_phone, su_comment FROM item, supplier, stock, nation, region, (SELECT s_i_id AS m_i_id, MIN(s_quantity) AS m_s_quantity FROM stock, supplier, nation, region WHERE MOD((s_w_id*s_i_id), 10000)=su_suppkey AND su_nationkey=n_nationkey AND n_regionkey=r_regionkey AND r_name LIKE 'Europ%' GROUP BY s_i_id) m WHERE i_id = s_i_id AND MOD((s_w_id * s_i_id), 10000) = su_suppkey AND su_nationkey = n_nationkey AND n_regionkey = r_regionkey AND i_data LIKE '%b' AND r_name LIKE 'Europ%' AND i_id=m_i_id AND s_quantity = m_s_quantity ORDER BY n_name, su_name, i_id"#,
    r#"SELECT ol_o_id, ol_w_id, ol_d_id, sum(ol_amount) AS revenue, o_entry_d FROM customer, new_order, orders, order_line WHERE c_state LIKE 'A%' AND c_id = o_c_id AND c_w_id = o_w_id AND c_d_id = o_d_id AND no_w_id = o_w_id AND no_d_id = o_d_id AND no_o_id = o_id AND ol_w_id = o_w_id AND ol_d_id = o_d_id AND ol_o_id = o_id AND o_entry_d > '2007-01-02 00:00:00.000000' GROUP BY ol_o_id, ol_w_id, ol_d_id, o_entry_d ORDER BY revenue DESC , o_entry_d"#,
    r#"SELECT o_ol_cnt, count(*) AS order_count FROM orders WHERE exists (SELECT * FROM order_line WHERE o_id = ol_o_id AND o_w_id = ol_w_id AND o_d_id = ol_d_id AND ol_delivery_d >= o_entry_d) GROUP BY o_ol_cnt ORDER BY o_ol_cnt"#,
    r#"SELECT n_name, sum(ol_amount) AS revenue FROM customer, orders, order_line, stock, supplier, nation, region WHERE c_id = o_c_id AND c_w_id = o_w_id AND c_d_id = o_d_id AND ol_o_id = o_id AND ol_w_id = o_w_id AND ol_d_id=o_d_id AND ol_w_id = s_w_id AND ol_i_id = s_i_id AND MOD((s_w_id * s_i_id), 10000) = su_suppkey AND ascii(substring(c_state from  1  for  1)) = su_nationkey AND su_nationkey = n_nationkey AND n_regionkey = r_regionkey AND r_name = 'Europe' AND o_entry_d >= '2007-01-02 00:00:00.000000' GROUP BY n_name ORDER BY revenue DESC"#,
    r#"SELECT sum(ol_amount) AS revenue FROM order_line WHERE ol_delivery_d >= '1999-01-01 00:00:00.000000' AND ol_delivery_d < '2020-01-01 00:00:00.000000' AND ol_quantity BETWEEN 1 AND 100000"#,
    r#"SELECT su_nationkey AS supp_nation, substring(c_state from 1 for 1) AS cust_nation, extract(YEAR FROM o_entry_d) AS l_year, sum(ol_amount) AS revenue FROM supplier, stock, order_line, orders, customer, nation n1, nation n2 WHERE ol_supply_w_id = s_w_id AND ol_i_id = s_i_id AND MOD ((s_w_id * s_i_id), 10000) = su_suppkey AND ol_w_id = o_w_id AND ol_d_id = o_d_id AND ol_o_id = o_id AND c_id = o_c_id AND c_w_id = o_w_id AND c_d_id = o_d_id AND su_nationkey = n1.n_nationkey AND ascii(substring(c_state from  1  for  1)) = n2.n_nationkey AND ((n1.n_name = 'Germany' AND n2.n_name = 'Cambodia') OR (n1.n_name = 'Cambodia' AND n2.n_name = 'Germany')) GROUP BY su_nationkey, cust_nation, l_year ORDER BY su_nationkey, cust_nation, l_year"#,
    r#"SELECT extract(YEAR FROM o_entry_d) AS l_year, sum(CASE WHEN n2.n_name = 'Germany' THEN ol_amount ELSE 0 END) / sum(ol_amount) AS mkt_share FROM item, supplier, stock, order_line, orders, customer, nation n1, nation n2, region WHERE i_id = s_i_id AND ol_i_id = s_i_id AND ol_supply_w_id = s_w_id AND MOD ((s_w_id * s_i_id), 10000) = su_suppkey AND ol_w_id = o_w_id AND ol_d_id = o_d_id AND ol_o_id = o_id AND c_id = o_c_id AND c_w_id = o_w_id AND c_d_id = o_d_id AND n1.n_nationkey = ascii(substring(c_state from  1  for  1)) AND n1.n_regionkey = r_regionkey AND ol_i_id < 1000 AND r_name = 'Europe' AND su_nationkey = n2.n_nationkey AND i_data LIKE '%b' AND i_id = ol_i_id GROUP BY l_year ORDER BY l_year"#,
    r#"SELECT n_name, extract(YEAR FROM o_entry_d) AS l_year, sum(ol_amount) AS sum_profit FROM item, stock, supplier, order_line, orders, nation WHERE ol_i_id = s_i_id AND ol_supply_w_id = s_w_id AND MOD ((s_w_id * s_i_id), 10000) = su_suppkey AND ol_w_id = o_w_id AND ol_d_id = o_d_id AND ol_o_id = o_id AND ol_i_id = i_id AND su_nationkey = n_nationkey AND i_data LIKE '%bb' GROUP BY n_name, l_year ORDER BY n_name, l_year DESC"#,
    r#"SELECT c_id, c_last, sum(ol_amount) AS revenue, c_city, c_phone, n_name FROM customer, orders, order_line, nation WHERE c_id = o_c_id AND c_w_id = o_w_id AND c_d_id = o_d_id AND ol_w_id = o_w_id AND ol_d_id = o_d_id AND ol_o_id = o_id AND o_entry_d >= '2007-01-02 00:00:00.000000' AND o_entry_d <= ol_delivery_d AND n_nationkey = ascii(substring(c_state from  1  for  1)) GROUP BY c_id, c_last, c_city, c_phone, n_name ORDER BY revenue DESC"#,
    r#"SELECT s_i_id, sum(s_order_cnt) AS ordercount FROM stock, supplier, nation WHERE mod((s_w_id * s_i_id), 10000) = su_suppkey AND su_nationkey = n_nationkey AND n_name = 'Germany' GROUP BY s_i_id HAVING sum(s_order_cnt) > (SELECT sum(s_order_cnt) * .005 FROM stock, supplier, nation WHERE mod((s_w_id * s_i_id), 10000) = su_suppkey AND su_nationkey = n_nationkey AND n_name = 'Germany') ORDER BY ordercount DESC"#,
    r#"SELECT o_ol_cnt, sum(CASE WHEN o_carrier_id = 1 OR o_carrier_id = 2 THEN 1 ELSE 0 END) AS high_line_count, sum(CASE WHEN o_carrier_id <> 1 AND o_carrier_id <> 2 THEN 1 ELSE 0 END) AS low_line_count FROM orders, order_line WHERE ol_w_id = o_w_id AND ol_d_id = o_d_id AND ol_o_id = o_id AND o_entry_d <= ol_delivery_d AND ol_delivery_d < '2020-01-01 00:00:00.000000' GROUP BY o_ol_cnt ORDER BY o_ol_cnt"#,
    r#"SELECT c_count, count(*) AS custdist FROM (SELECT c_id, count(o_id) AS c_count FROM customer LEFT OUTER JOIN orders ON (c_w_id = o_w_id AND c_d_id = o_d_id AND c_id = o_c_id AND o_carrier_id > 8) GROUP BY c_id) AS c_orders GROUP BY c_count ORDER BY custdist DESC, c_count DESC"#,
    r#"SELECT (100.00 * sum(CASE WHEN i_data LIKE 'PR%' THEN ol_amount ELSE 0 END) / (1 + sum(ol_amount))) AS promo_revenue FROM order_line, item WHERE ol_i_id = i_id AND ol_delivery_d >= '2007-01-02 00:00:00.000000' AND ol_delivery_d < '2020-01-02 00:00:00.000000'"#,
    r#"SELECT su_suppkey, su_name, su_address, su_phone, total_revenue FROM supplier, revenue0 WHERE su_suppkey = supplier_no AND total_revenue = (select max(total_revenue) from revenue0) ORDER BY su_suppkey"#,
    r#"SELECT i_name, substring(i_data from  1 for 3) AS brand, i_price, count(DISTINCT (mod((s_w_id * s_i_id),10000))) AS supplier_cnt FROM stock, item WHERE i_id = s_i_id AND i_data NOT LIKE 'zz%' AND (mod((s_w_id * s_i_id),10000) NOT IN (SELECT su_suppkey FROM supplier WHERE su_comment LIKE '%bad%')) GROUP BY i_name, brand, i_price ORDER BY supplier_cnt DESC"#,
    r#"SELECT SUM(ol_amount) / 2.0 AS avg_yearly FROM order_line, (SELECT i_id, AVG (ol_quantity) AS a FROM item, order_line WHERE i_data LIKE '%b' AND ol_i_id = i_id GROUP BY i_id) t WHERE ol_i_id = t.i_id AND ol_quantity < t.a"#,
    r#"SELECT c_last, c_id, o_id, o_entry_d, o_ol_cnt, sum(ol_amount) AS amount_sum FROM customer, orders, order_line WHERE c_id = o_c_id AND c_w_id = o_w_id AND c_d_id = o_d_id AND ol_w_id = o_w_id AND ol_d_id = o_d_id AND ol_o_id = o_id GROUP BY o_id, o_w_id, o_d_id, c_id, c_last, o_entry_d, o_ol_cnt HAVING sum(ol_amount) > 200 ORDER BY amount_sum DESC, o_entry_d"#,
    r#"SELECT sum(ol_amount) AS revenue FROM order_line, item WHERE (ol_i_id = i_id AND i_data LIKE '%a' AND ol_quantity >= 1 AND ol_quantity <= 10 AND i_price BETWEEN 1 AND 400000 AND ol_w_id IN (1, 2, 3)) OR (ol_i_id = i_id AND i_data LIKE '%b' AND ol_quantity >= 1 AND ol_quantity <= 10 AND i_price BETWEEN 1 AND 400000 AND ol_w_id IN (1, 2, 4)) OR (ol_i_id = i_id AND i_data LIKE '%c' AND ol_quantity >= 1 AND ol_quantity <= 10 AND i_price BETWEEN 1 AND 400000 AND ol_w_id IN (1, 5, 3))"#,
    r#"SELECT su_name, su_address FROM supplier, nation WHERE su_suppkey IN (SELECT mod(s_i_id * s_w_id, 10000) FROM stock INNER JOIN item ON i_id = s_i_id INNER JOIN order_line ON ol_i_id = s_i_id WHERE ol_delivery_d > '2010-05-23 12:00:00' AND i_data LIKE 'co%' GROUP BY s_i_id, s_w_id, s_quantity HAVING 2*s_quantity > sum(ol_quantity)) AND su_nationkey = n_nationkey AND n_name = 'Germany' ORDER BY su_name"#,
    r#"SELECT su_name, count(*) AS numwait FROM supplier, order_line l1, orders, stock, nation WHERE ol_o_id = o_id AND ol_w_id = o_w_id AND ol_d_id = o_d_id AND ol_w_id = s_w_id AND ol_i_id = s_i_id AND mod((s_w_id * s_i_id),10000) = su_suppkey AND l1.ol_delivery_d > o_entry_d AND NOT EXISTS (SELECT * FROM order_line l2 WHERE l2.ol_o_id = l1.ol_o_id AND l2.ol_w_id = l1.ol_w_id AND l2.ol_d_id = l1.ol_d_id AND l2.ol_delivery_d > l1.ol_delivery_d) AND su_nationkey = n_nationkey AND n_name = 'Germany' GROUP BY su_name ORDER BY numwait DESC, su_name"#,
    r#"SELECT substring(c_state from 1 for 1) AS country, count(*) AS numcust, sum(c_balance) AS totacctbal FROM customer WHERE substring(c_phone from 1 for 1) IN ('1', '2', '3', '4', '5', '6', '7') AND c_balance > (SELECT avg(c_balance) FROM customer WHERE c_balance > 0.00 AND substring(c_phone from 1 for 1) IN ('1', '2', '3', '4', '5', '6', '7')) AND NOT EXISTS (SELECT * FROM orders WHERE o_c_id = c_id AND o_w_id = c_w_id AND o_d_id = c_d_id) GROUP BY substring(c_state from 1 for 1) ORDER BY substring(c_state,1,1)"#,
];

/// The view Q15 reads, which each stream makes once under a name of its own in place of
/// `revenue0`, so two streams do not make and drop one view between them.
pub(crate) const Q15_VIEW: &str = r#"CREATE view revenue0 (supplier_no, total_revenue) AS SELECT mod((s_w_id * s_i_id),10000) as supplier_no, sum(ol_amount) as total_revenue FROM order_line, stock WHERE ol_i_id = s_i_id AND ol_supply_w_id = s_w_id AND ol_delivery_d >= '2007-01-02 00:00:00.000000' GROUP BY supplier_no"#;

/// The read that tells a snapshot's freshness, section 9.6.
const DISTRICT_READ: &str = "SELECT d_w_id, d_id, d_next_o_id FROM district";

/// The year the benchmark's dates were written against, section 9.2.
const REFERENCE_YEAR: i32 = 2012;

/// How many years every date of the texts moves on: from [`REFERENCE_YEAR`] to the year of the
/// load timestamp.
pub(crate) fn shift_years() -> i32 {
    LOADED_AT[..4].parse::<i32>().expect("the load timestamp starts with its year") - REFERENCE_YEAR
}

/// `text` with every date literal in it moved on by `years`. A date literal is a quote, four
/// digits and a dash, and all of the texts are ASCII.
pub(crate) fn shifted(text: &str, years: i32) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while at < bytes.len() {
        let date = bytes[at] == b'\''
            && bytes.len() > at + 5
            && bytes[at + 1..at + 5].iter().all(u8::is_ascii_digit)
            && bytes[at + 5] == b'-';
        if date {
            let year: i32 = text[at + 1..at + 5].parse().expect("four digits");
            let _ = write!(out, "'{:04}", year + years);
            at += 5;
        } else {
            out.push(char::from(bytes[at]));
            at += 1;
        }
    }
    out
}

/// The name of stream `stream`'s view in place of `revenue0`.
fn view_name(stream: usize) -> String {
    format!("revenue{stream}")
}

/// The 22 texts as one stream runs them, shifted and with its own view, and the view.
fn texts(stream: usize) -> (String, Vec<String>) {
    let (years, view) = (shift_years(), view_name(stream));
    let named = |text: &str| shifted(text, years).replace("revenue0", &view);
    (named(Q15_VIEW), QUERIES.iter().map(|text| named(text)).collect())
}

/// The SHA-256 of the 22 shifted texts as one file, Q15 as its three statements the way BenchBase
/// writes them, or `unknown` when neither `sha256sum` nor `shasum` runs.
pub(crate) fn texts_sha256() -> String {
    let years = shift_years();
    let mut file = String::new();
    for (at, text) in QUERIES.iter().enumerate() {
        if at == 14 {
            let _ = writeln!(file, "{};", shifted(Q15_VIEW, years));
            let _ = writeln!(file, "{};", shifted(text, years));
            file.push_str("DROP VIEW revenue0;\n");
        } else {
            let _ = writeln!(file, "{};", shifted(text, years));
        }
    }
    let hashed = |program: &str, arguments: &[&str]| -> Option<String> {
        let mut child = Command::new(program)
            .args(arguments)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        child.stdin.take()?.write_all(file.as_bytes()).ok()?;
        let output = child.wait_with_output().ok()?;
        let text = String::from_utf8(output.stdout).ok()?;
        text.split_whitespace().next().map(str::to_string)
    };
    hashed("sha256sum", &[])
        .or_else(|| hashed("shasum", &["-a", "256"]))
        .unwrap_or_else(|| "unknown".to_string())
}

/// What one stream did.
#[derive(Debug)]
pub(crate) struct Ran {
    /// The time of each query's runs that started inside the window, in microseconds, so a query
    /// of up to a day fits the histogram.
    pub(crate) times: Vec<Histogram>,
    /// The rows each query returned the last time it ran.
    pub(crate) rows: Vec<u64>,
    /// The passes over all 22 that finished.
    pub(crate) passes: u64,
    /// The first error of each query that failed, and how many times it did.
    pub(crate) errors: Vec<(usize, String, u64)>,
    /// The freshness of each query judged, section 9.6, in microseconds.
    pub(crate) stale: Histogram,
    /// The queries judged that missed an order acknowledged before their snapshot started.
    pub(crate) late: u64,
    /// The queries judged that saw an order acknowledged after their snapshot started.
    pub(crate) ahead: u64,
    /// The orders those saw early, summed over them.
    pub(crate) ahead_orders: u64,
    /// The most by which such an order's acknowledgement came after the snapshot started.
    pub(crate) lead: Duration,
}

impl Ran {
    fn new() -> Self {
        Self {
            times: (0..QUERIES.len()).map(|_| Histogram::new()).collect(),
            rows: vec![0; QUERIES.len()],
            passes: 0,
            errors: Vec::new(),
            stale: Histogram::new(),
            late: 0,
            ahead: 0,
            ahead_orders: 0,
            lead: Duration::ZERO,
        }
    }

    fn judged(&mut self, judged: Judged) {
        self.stale.record(judged.stale.as_micros().try_into().unwrap_or(u64::MAX));
        self.late += u64::from(judged.stale > Duration::ZERO);
        self.ahead += u64::from(judged.ahead > 0);
        self.ahead_orders += judged.ahead;
        self.lead = self.lead.max(judged.lead);
    }

    fn failed(&mut self, query: usize, message: String) {
        match self.errors.iter_mut().find(|(at, _, _)| *at == query) {
            Some((_, _, times)) => *times += 1,
            None => self.errors.push((query, message, 1)),
        }
    }
}

/// An analytic session with the 22 queries prepared, and its view made.
pub(crate) struct Stream<'s> {
    session: Box<dyn Session + 's>,
    /// Each query's statement, or why it could not be prepared.
    statements: Vec<Result<usize, String>>,
    view: String,
    rng: Rng,
    out: Values,
    /// The acknowledgements and the district read, when the stream judges freshness.
    fresh: Option<(&'s Acks, usize)>,
}

impl std::fmt::Debug for Stream<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stream").field("view", &self.view).finish_non_exhaustive()
    }
}

impl<'s> Stream<'s> {
    /// Connects stream `stream`, tells the session how many threads a query may take, makes its
    /// view and prepares the queries. A query the engine cannot prepare is reported and left out
    /// rather than ending the run.
    pub(crate) fn open(
        backend: &'s dyn Backend,
        stream: usize,
        threads: usize,
        seed: u64,
    ) -> Result<Self, String> {
        let mut session = backend.connect()?;
        if let Some(set) = backend.analytic_threads(threads) {
            session.batch(&set).map_err(|failed| format!("{set} failed: {failed}"))?;
        }
        let (view, queries) = texts(stream);
        let name = view_name(stream);
        let _ = session.batch(&format!("DROP VIEW IF EXISTS {name}"));
        session
            .batch(&view)
            .map_err(|failed| format!("making the view {name} failed: {failed}"))?;
        let statements = queries.iter().map(|text| session.prepare(text)).collect();
        let rng = data::stream(seed, 300, stream as u64, 0);
        Ok(Self { session, statements, view: name, rng, out: Values::default(), fresh: None })
    }

    /// Judges the freshness of each query [`Self::run`] runs against `acks`, section 9.6, and
    /// tells `acks` the orders there already. An error leaves the stream as it was.
    pub(crate) fn judging(&mut self, acks: &'s Acks) -> Result<(), String> {
        let read = self.session.prepare(DISTRICT_READ)?;
        acks.there(&self.districts(read)?);
        self.fresh = Some((acks, read));
        Ok(())
    }

    /// Each district's `(d_w_id, d_id, d_next_o_id)`.
    fn districts(&mut self, read: usize) -> Result<Vec<(u64, u64, u64)>, String> {
        self.out.clear();
        self.session
            .execute(read, &[], &mut self.out)
            .map_err(|failed| format!("reading the districts failed: {failed}"))?;
        let values: Vec<&[u8]> = (0..self.out.len()).map(|at| self.out.get(at)).collect();
        fresh::rows(&values).ok_or_else(|| {
            format!("the district read returned {} values, not three numbers a row", values.len())
        })
    }

    /// Why each query that could not be prepared could not.
    pub(crate) fn unprepared(&self) -> Vec<(usize, &str)> {
        self.statements
            .iter()
            .enumerate()
            .filter_map(|(at, statement)| statement.as_ref().err().map(|why| (at, why.as_str())))
            .collect()
    }

    /// Runs query `query` once and, if `counted`, records how long it took, and says whether it
    /// ran. A query that could not be prepared is not run.
    fn once(&mut self, query: usize, counted: bool, ran: &mut Ran) -> bool {
        let Ok(statement) = self.statements[query] else {
            return false;
        };
        self.out.clear();
        let started = Instant::now();
        let outcome = self.session.execute(statement, &[], &mut self.out);
        let took = started.elapsed();
        match outcome {
            Ok(rows) if counted => {
                ran.times[query].record(took.as_micros().try_into().unwrap_or(u64::MAX));
                ran.rows[query] = rows;
            }
            Ok(_) => {}
            Err(failed) => {
                ran.failed(query, failed.message().to_string());
                return false;
            }
        }
        true
    }

    /// Runs query `query` once as [`Self::once`] does, and when the stream judges freshness, in a
    /// transaction of its own that reads the districts first, section 9.6.
    fn judged(&mut self, query: usize, counted: bool, ran: &mut Ran) {
        let Some((acks, read)) = self.fresh else {
            self.once(query, counted, ran);
            return;
        };
        if self.statements[query].is_err() {
            return;
        }
        if let Err(failed) = self.session.begin_snapshot() {
            ran.failed(query, format!("starting the snapshot failed: {failed}"));
            return;
        }
        let sent = Instant::now();
        let rows = self.districts(read);
        let got = Instant::now();
        let judged = rows.map(|rows| acks.judge(&rows, sent, got, Instant::now()));
        let ended = if self.once(query, counted, ran) && judged.is_ok() {
            self.session.commit()
        } else {
            self.session.rollback()
        };
        match (judged, ended) {
            (Err(why), _) => ran.failed(query, why),
            (_, Err(failed)) => ran.failed(query, format!("ending the snapshot failed: {failed}")),
            (Ok(judged), Ok(())) if counted => ran.judged(judged),
            (Ok(_), Ok(())) => {}
        }
    }

    /// Runs the 22 over and over, each pass in an order drawn from the stream's own seed, until the
    /// window closes or a terminal stops the run. Only runs that start inside the window count.
    pub(crate) fn run(mut self, clock: &Clock, stop: &AtomicBool) -> Ran {
        let mut ran = Ran::new();
        let over =
            || stop.load(Ordering::Relaxed) || clock.end().is_some_and(|end| Instant::now() >= end);
        let mut order: Vec<usize> = (0..QUERIES.len()).collect();
        'passes: loop {
            for at in (1..order.len()).rev() {
                order.swap(at, self.rng.below(at as u64 + 1) as usize);
            }
            for &query in &order {
                if over() {
                    break 'passes;
                }
                let counted = clock.measured().is_some_and(|opened| Instant::now() >= opened);
                self.judged(query, counted, &mut ran);
            }
            if clock.measured().is_some() {
                ran.passes += 1;
            }
        }
        ran
    }

    /// Runs each query `runs` times in turn, for the analytic reference.
    pub(crate) fn each(mut self, runs: usize) -> Ran {
        let mut ran = Ran::new();
        for query in 0..QUERIES.len() {
            for _ in 0..runs {
                let _ = self.once(query, true, &mut ran);
            }
        }
        ran.passes = runs as u64;
        ran
    }
}

impl Drop for Stream<'_> {
    fn drop(&mut self) {
        let _ = self.session.batch(&format!("DROP VIEW IF EXISTS {}", self.view));
    }
}

/// The geometric mean of `values`, all of them above zero, or zero when there are none.
fn geometric_mean(values: &[f64]) -> f64 {
    if values.is_empty() || values.iter().any(|value| *value <= 0.0) {
        return 0.0;
    }
    (values.iter().map(|value| value.ln()).sum::<f64>() / values.len() as f64).exp()
}

/// The report lines of the analytic side, a `ch query` line a query, a `ch error` line for each
/// query that failed, a `ch summary`, and a `ch freshness` when queries were judged. `window` is
/// how long the streams ran, for the queries an hour, or `None` for the reference.
pub(crate) fn report(streams: &[Ran], window: Option<Duration>, out: &mut String) {
    let mut medians = Vec::new();
    let mut hourly = Vec::new();
    for query in 0..QUERIES.len() {
        let mut times = Histogram::new();
        for ran in streams {
            times.merge(&ran.times[query]);
        }
        let errors: u64 = streams
            .iter()
            .flat_map(|ran| ran.errors.iter())
            .filter(|(at, _, _)| *at == query)
            .map(|(_, _, times)| times)
            .sum();
        let rows = streams.iter().map(|ran| ran.rows[query]).max().unwrap_or(0);
        let ms = |micros: u64| micros as f64 / 1e3;
        let median = ms(times.quantile(0.5));
        if times.count() > 0 {
            medians.push(median);
        }
        if let Some(window) = window {
            hourly.push(times.count() as f64 * 3600.0 / window.as_secs_f64());
        }
        let _ = writeln!(
            out,
            "ch query=Q{} n={} median_ms={median:.2} q1_ms={:.2} q3_ms={:.2} min_ms={:.2} \
             max_ms={:.2} rows={rows} errors={errors} hist_us={}",
            query + 1,
            times.count(),
            ms(times.quantile(0.25)),
            ms(times.quantile(0.75)),
            ms(times.min()),
            ms(times.max()),
            times.to_line(),
        );
    }
    let mut errors: Vec<(usize, &str, u64)> = Vec::new();
    for (query, message, times) in streams.iter().flat_map(|ran| ran.errors.iter()) {
        match errors.iter_mut().find(|(at, _, _)| at == query) {
            Some((_, _, seen)) => *seen += times,
            None => errors.push((*query, message, *times)),
        }
    }
    errors.sort_unstable_by_key(|(query, _, _)| *query);
    for (query, message, times) in errors {
        let _ = writeln!(out, "ch error query=Q{} times={times} message={message:?}", query + 1);
    }
    let _ = write!(
        out,
        "ch summary streams={} passes={} answered={} geomean_ms={:.2}",
        streams.len(),
        streams.iter().map(|ran| ran.passes).sum::<u64>(),
        medians.len(),
        geometric_mean(&medians),
    );
    if window.is_some() {
        let qph = geometric_mean(&hourly);
        let _ =
            write!(out, " qph={qph:.1} qph_per_stream={:.1}", qph / streams.len().max(1) as f64);
    }
    out.push('\n');
    let mut stale = Histogram::new();
    for ran in streams {
        stale.merge(&ran.stale);
    }
    if stale.count() > 0 {
        let ms = |micros: u64| micros as f64 / 1e3;
        let _ = writeln!(
            out,
            "ch freshness judged={} late={} stale_median_ms={:.2} stale_p99_ms={:.2} \
             stale_max_ms={:.2} ahead={} ahead_orders={} lead_max_ms={:.2}",
            stale.count(),
            streams.iter().map(|ran| ran.late).sum::<u64>(),
            ms(stale.quantile(0.5)),
            ms(stale.quantile(0.99)),
            ms(stale.max()),
            streams.iter().map(|ran| ran.ahead).sum::<u64>(),
            streams.iter().map(|ran| ran.ahead_orders).sum::<u64>(),
            streams.iter().map(|ran| ran.lead).max().unwrap_or_default().as_secs_f64() * 1e3,
        );
    }
}

/// The rows of `region` as BenchBase's `region_gen.tbl` has them, with the blanks that pad each
/// field to its column's width taken off, as a `CHAR` column would compare them.
const REGIONS: [(i64, &str, &str); 5] = [
    (
        1,
        "Africa",
        "bIKaGP4VDy4kV8z5GsyCvoC2SFL8BBb9lI5hJx284oWL2fhPNqV3XeRTJM2U2IhodH4wID9cDCr5wQSZJ0RQ0qP3bx99bR4L7QC13dYq0kWwmfTa5NMA",
    ),
    (
        2,
        "America",
        "NmdHQaMgd9G4OopGEQ5mQGp6o79hI8NL0BobmVWEu29MuJpQVLQrUhvLi0JQhyY0Lkt0KYDbx1S8FAKcMJLF0eDQPenevRCAj5Z",
    ),
    (
        3,
        "Asia",
        "bIDtqrZXraFtliGTl7NIz2RtLBavLE2cEVgW9iPEZi3WL59LEBXNk4pScy96SYU7Ti0SYSi7oebrakOD5tuUUdLkbztPU8GpD8pYvZ889eGLWu0mYi3XD4iCw0M",
    ),
    (
        4,
        "Australia",
        "6pRd6jAeKGj2vWYQqHtpPUupR5Sc1SkQGVDg2ZGueffAZaCufbx7YKUohHsPyz7pMfL4fEOS1wsOjGwDlamI8cZDvNOdQn4m9J2NKOHBMZ1tTuDplR0nKu8d1V2lB1Pw4elmyE3NJqtMWthf36R2pk",
    ),
    (
        5,
        "Europe",
        "1oi2r0Es7qYdGyYVTaRJlxzbcghww6cbQ2eTXilhXo5P5cjWwAzZnsE4I5XTIExBXnHnMQLcIMaHJjdOS0uOOFW24djaeT4918wiulgjyv0wiKOO3ci0Who7CsOOASLeBG",
    ),
];

/// The rows of `nation` as BenchBase's `nation_gen.tbl` has them, padding taken off the same way.
const NATIONS: [(i64, &str, i64, &str); 62] = [
    (
        48,
        "Australia",
        4,
        "ImYA3dlu79hFqWejPPZF9FecMt9Ly6QQ71MQYCPanCCOaCTo3nGWvKIAdc6Ltbp3KOTw8mILwgdPcJA2MedBw7jbxfAxQ0DXKN5zPBHbsyGw1",
    ),
    (49, "Belgium", 5, "PwMGK3g3FFlHQEYPgJUSbLwagROjSddIuzcF6qxtwCGsyTLYeuyjd7QB"),
    (50, "Cameroon", 1, "hNuqDIdJz52cLzGRAHWHVAlu"),
    (51, "Denmark", 5, "7nuyXxx1yGrZ9zVicLJlDfKEdVKXk7G"),
    (
        52,
        "Ecuador",
        2,
        "rbCqamFknsRfsCXM1ckEmHDA7FBbqSykNr16xKLKmn4YSm3wbUbDUGi61giUEOlgTvIe0AxgcmfWL5FFi6XMVGnMHj9LpCHHZXOuSrzsy",
    ),
    (
        53,
        "France",
        5,
        "cC1VvW0URAvH9HpXxu7rmwWpvjmapq52uFW9VovhbfxSlaMoO3btHsDYHOj8l7tnHqSVx4w5pOCxtcoQW1O",
    ),
    (
        54,
        "Germany",
        5,
        "NURjbfJXaBbpH0tRswofPfVqvgUZJmEhR9p4QvFFl1FSeDjM9DkxOwHxnqO7qnH2pIBw9XMUUwHMmq2A1lnACNUnmG6TCh",
    ),
    (55, "Hungary", 5, "IHzD3VnQNtoIS6MdHXv15avj6elCNnxoBFoQdwoPcxaofwbt2RM14dAViehcq81WRykh1Ao"),
    (
        56,
        "Italy",
        5,
        "KERvGkQ5V9h5kZ39XMiV8fY0dhBxKLBphnd5VedUKsUgLGjDBeBPBXFv94MUj4kHTVunxAWZ4CeZzKW7",
    ),
    (57, "Japan", 3, "BugljuTxD7OvuBjxZ34"),
    (
        65,
        "Kenya",
        1,
        "r5qawQKz8nGugl8W5Wp8FqmkkpZkaq2iheisO7C7qXVD5rAX1gNpt9HUEYNQYnBt3HwBESdhyrAsIkRrLJ6mKsAe0m89keyo",
    ),
    (66, "Lithuania", 5, "T5om5nUhCLg"),
    (67, "Mexico", 2, "ELuxhMrRKy0Hk"),
    (68, "Netherlands", 5, "tiIKiIsAKe4T21hxdCjcTBKM"),
    (
        69,
        "Oman",
        1,
        "hIimZrqu4iJUJZV9oXCxoesJTXSCQJccW696JnfPX3AN6pE0Ws3YlJnqBwVo4tovKzzI59izWDjgvm42gAqOIeFCffdcIy8OrtplwhsMVN14w0epe8uhTVLu",
    ),
    (70, "Portugal", 5, "S0HVjn45jpkhnL1n1iMcmbSyHvZXC2w5Bkcf6rJKsJR6aOd"),
    (
        71,
        "Qatar",
        1,
        "IOTzgo5Pl501f8C1jxSATVw1J4BSHqQJxDGQAQikzT2NQuZAmyaVfGHxGP8mjkGCxC6aUo8UjUPdjbnXK0lciljmLL6smzJu3t7CP7T",
    ),
    (72, "Rwanda", 1, "V5X74pR9XuZt7slAvolM5IAvuufXtbbzqCwKex"),
    (73, "Serbia", 5, "yciUkPbmR6dcFS7NwwZmIqnjoQsHGBMgbvLEg2AEYULsccz4"),
    (74, "Togo", 1, "hejrh8otzRReyPdeBLI8Q2QRxzaJrPPX0Nm62zDqDO1E73USsBXR"),
    (
        75,
        "United States",
        2,
        "XzpCgCmTiYWwWlKS9BZEBK6wRgwVRa9uTwfT2SPTnmKIqyL9qVxH83NPr3ZKYMY8Kwc9YuqD3NqUR4AqFMOUcTInTEGs9lhA09tE",
    ),
    (76, "Vietnam", 3, "oky6mS1P6Tkulb8QVj5aYATUPi2K6DY9QobV"),
    (
        77,
        "Singapore",
        3,
        "ZLUa27OFYDjSBqkoRmyoNQZet17zCJ7YlcWqZ8Qz9g11YU0MZl5sLIRLinhcjl6JlVS6lHmbHEMc54",
    ),
    (
        78,
        "Cambodia",
        3,
        "Rfy2xDF5koJIHlos5IGpWY30VNNlZYSeojQRFb1OGXxYvWzj2SZ4G9x4qwpUV8sC0L5ufyQkB7",
    ),
    (79, "Yemen", 1, "bPG1FRMq8PyKYpAk2nxWOpwCyjDw0yjPahOd1T9uSN00WpAylglt"),
    (
        80,
        "Zimbabwe",
        1,
        "y0eUHlWs0JX2WqR5ulbFhCsthvmpCzjiEkotFxk1qn6AYTaJWkrNFKPFhsCpWxVHGP1cFKX7qt4tUP6NUy3j",
    ),
    (81, "Argentina", 2, "yrgra42JDGRqJ27j"),
    (82, "Bolivia", 2, "mp6KDXsdjoIPpJeYeZA0ziCYZ1p3Bpd85QU6SIAZrwN4lZuZWERD7Pl5jrC9c"),
    (
        83,
        "Canada",
        2,
        "N5gZGOoMSOeOWE3mFCRZpws38KKOS8i86XvROexKI3DeqzabrrYwLYUHGEw3YZiGLdBRIRWoTiTCW6nVLIIv5dIZN1BNiZuJc6tUN2vOLy",
    ),
    (84, "Dominican Republic", 2, "pFR3SUfPS7si4MwUCvGPvsCMTrequ8J0m4Lu"),
    (85, "Egypt", 1, "NHPzECTo0jOXc6QsfAHRQmoKlo31xNqsDzE1wAcNduijHqxGsGAsfncY9"),
    (86, "Finnland", 5, "mw31VU3cV2qWlQvtKIn6pbFxeo1TyGHrS1xk8LBQswXsNkEvXzG1R"),
    (
        87,
        "Ghana",
        1,
        "m329BZJXqa556yROCdiLRocWUV6mmbusNkAFCKLxJjAWvTH8q1yTWl8vCPRUBdkzeR7yLgypwnq7EGyl5L",
    ),
    (
        88,
        "Haiti",
        2,
        "L34nIapyZ08tgv3liaEhqmypxiuzFzGG8EJIQAJsozLmnFOcUtw7A4lUThjIbgM8uoD9iBEognaYqCj9Tsu1OeiUBZfTVz1gV6y",
    ),
    (89, "India", 3, "OPnfUAAVAUMErDP1O8p2J4nRQfGaamS0Nq6pbidpuIVhqhj6AFFrwnzelQn"),
    (90, "Jamaica", 4, "ywH9riXfBu5dp3lwgC1fTMnxOlScaeW4MjyjBP8mYlSp"),
    (
        97,
        "Kazakhstan",
        3,
        "qHb04sdUyr25So0A7Fwz4iEbQtXzp2xd4k5KJPQWB0s366BNnEjFqaRqZgZji3ewI6CdyanMfDmr3wmuGYGu4VffcXfG2xffS1s7G5DLX",
    ),
    (
        98,
        "Luxembourg",
        5,
        "XNOi8zXKdppOztQT18xFPCcmNButvzjbuv61XVRT4DLTnk4rQYM0IJNaavgPwm9Zq0bsDT82J0blTpmOBaTjIq6HGPCDHvqOpy6YBW",
    ),
    (99, "Morocco", 1, "Po1NXucw7yj9nE2jSUgZ7GY0U97E3hxt"),
    (
        100,
        "Norway",
        5,
        "K01kE1rBmjazFDMJqiWVxkMFpXS2HPJUvi4KHqLsA1hlkZSJK0mdjifqLeH9TrhPbnQBTrH6V7vawXr5FluMjHeYDft24NnLNxIUx2bs4Uor4eqCfhqW",
    ),
    (
        101,
        "Poland",
        5,
        "6oUdNwfbV62yeSBQoGeE3mKy2jWm5Nlvif51XEEQlX9gW9cTmSevhLuaNMRqzY6ScGDzu7A3H2z8",
    ),
    (
        102,
        "Peru",
        2,
        "XNQLktpC5kJd9IBxbgXtuV5NHv4bSbpCkkw2wJeBE3DGPwfQegBP2lJ0tyJgYgd7TUWeZk4zKbDlJxlFJ7qrOJL",
    ),
    (103, "Nicaragua", 2, "56TozdJ9J04"),
    (104, "Romania", 5, "AXmE73xrnFqhWZM6CIcS8hsg569aIy"),
    (105, "South Africa", 1, "NIP7RHwtZXGs0orkiVKqo1jDxr5yjHjyw9H77h"),
    (
        106,
        "Thailand",
        3,
        "jBzOO2ft7hiORxUWpO5A3HNIbcCXD1USm4sa7nFF41bregt3b3ojfXv1YVVP4bag9thJgF3CnXXp4BRr1jBitKF",
    ),
    (107, "United Kingdom", 5, "OgeQYnr1I2prusdk1jLCwTT1xNA3MBCFIJkEq0E44y"),
    (
        108,
        "Venezuela",
        2,
        "dtmad27sf7Gj7Cnxj0bBMYodBa8YJQ2j9Ozl3n3A5BFKBxJvDsAwTwrtyIenvc2MLJ0EmvoGDptotIMLUUaMUnLC1pnvuqHS10EBe",
    ),
    (109, "Liechtenstein", 5, "2LNt79yezuSyjyXUubEuzyVno"),
    (110, "Austria", 5, "we0ueB1CSM6JIFgwEpn4X7b5jc3XAq82dSoL3KLM1VhqjHk"),
    (111, "Laos", 3, "0UdhTYwBmN8KCg8UFmgTIiziU9n7GeFLVNr"),
    (112, "Zambia", 1, "QiTRFmSLlFcv3UjM74bPT95KXsY8aBZZ6ya0"),
    (113, "Switzerland", 5, "958ci6lBtkmimOUgeX"),
    (114, "China", 3, "7iOGly6tStkOr1fxkwEYg55doV0l420WY5u6TXz9zG68rmZ3lBK4sxw4kDi1s"),
    (
        115,
        "Papua New Guinea",
        4,
        "rAoMqeT2PSzYDxekB3bRXspPpr8KFLJv4fD7ONtEAVc3FLE7ca1cWgtwf2mJaOZj0NQiTCGlYsN8QdAYmMZYdl6M4QOO5ZejNx5YEslUFQHXL8xZAx",
    ),
    (116, "East Timor", 4, "fAXZ6jQSYgsloUho8bxwPzrlRROXRZ615vU1QwpCujz9fZGzmxKQA"),
    (117, "Bulgaria", 5, "UsWwdKSwk68neapI7qGt2WvmxurU1"),
    (118, "Brazil", 2, "UHIrnQjiqxuXHpgCpYWBUhhIFlaOwXQdfN9VP0iIfe5NWK56HjaAOhVbzHYVFZ7Oz"),
    (119, "Albania", 5, "xENQx1ftvJOiiF"),
    (
        120,
        "Andorra",
        5,
        "thvD2MadZTLZuBddFXtDd0Zf8cHaEDVJdiCXIoWQIUX8oM4SOgnDnrtwemUSYj4oXauMCY7mud2ILfuiSAPdCdobwhwvgG27ZjdBSE5c",
    ),
    (
        121,
        "Belize",
        2,
        "jKRlI0Qnv0sR6GPeH1ln6WyhaKODfNHCB4yb4TOFlxFt1cOKjho5RQl0KOMBr5BAWAQaR6dszP2bcQW6lX4ObH5WCsNt",
    ),
    (122, "Botswana", 1, "0AexXDO3UdP6hfDyq2pfwYBewB4YKkLG1ey5l7YxFvc7b"),
];

/// The rows of `region`.
pub(crate) fn region(emit: &mut dyn FnMut(&[data::Lit])) {
    for (key, name, comment) in REGIONS {
        emit(&[data::Lit::Int(key), text(name), text(comment)]);
    }
}

/// The rows of `nation`.
pub(crate) fn nation(emit: &mut dyn FnMut(&[data::Lit])) {
    for (key, name, region, comment) in NATIONS {
        emit(&[data::Lit::Int(key), text(name), data::Lit::Int(region), text(comment)]);
    }
}

/// The 10,000 rows of `supplier`, made as BenchBase makes them from the stream of its place in
/// the tables.
pub(crate) fn supplier(seed: u64, emit: &mut dyn FnMut(&[data::Lit])) {
    let mut rng = data::stream(seed, SUPPLIER, 0, 0);
    for key in 1..=SUPPLIERS {
        let name = data::alphanumeric(&mut rng, 25, 25);
        let address = data::alphanumeric(&mut rng, 20, 40);
        let nation = NATIONS[rng.below(NATIONS.len() as u64) as usize].0;
        let phone = data::digits(&mut rng, 15);
        let balance = data::uniform(&mut rng, 1_000_000, 100_000_000_000);
        let comment = data::alphanumeric(&mut rng, 51, 101);
        emit(&[
            data::Lit::Int(key),
            data::Lit::Text(name),
            data::Lit::Text(address),
            data::Lit::Int(nation),
            data::Lit::Text(phone),
            data::Lit::Fixed(balance as i64, 2),
            data::Lit::Text(comment),
        ]);
    }
}

/// The place of `supplier` in [`TABLES`](super::text::TABLES).
const SUPPLIER: u64 = 11;

/// The rows of `supplier`, which do not scale with the warehouses.
pub(crate) const SUPPLIERS: i64 = 10_000;

fn text(value: &str) -> data::Lit {
    data::Lit::Text(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_date_moves_on_fourteen_years_and_nothing_else_moves() {
        assert_eq!(shift_years(), 14);
        let text =
            "WHERE d > '2007-01-02 00:00:00.000000' AND d < '2020-01-01' AND n IN ('1', '2')";
        assert_eq!(
            shifted(text, 14),
            "WHERE d > '2021-01-02 00:00:00.000000' AND d < '2034-01-01' AND n IN ('1', '2')"
        );
        let (_, queries) = texts(0);
        assert!(queries[19].contains("'2024-05-23 12:00:00'"), "{}", queries[19]);
        assert!(queries.iter().all(|text| !text.contains("oorder")));
    }

    #[test]
    fn each_stream_reads_a_view_of_its_own() {
        let (view, queries) = texts(3);
        assert!(view.starts_with("CREATE view revenue3 "), "{view}");
        assert!(queries[14].contains("FROM supplier, revenue3 "), "{}", queries[14]);
        assert!(!queries.iter().any(|text| text.contains("revenue0")));
    }

    #[test]
    fn the_added_tables_have_their_rows() {
        let mut regions = 0;
        region(&mut |_| regions += 1);
        let mut keys = Vec::new();
        nation(&mut |row| {
            if let data::Lit::Int(key) = row[0] {
                keys.push(key);
            }
        });
        let mut wanted: Vec<i64> =
            (b'0'..=b'9').chain(b'A'..=b'Z').chain(b'a'..=b'z').map(i64::from).collect();
        wanted.sort_unstable();
        keys.sort_unstable();
        assert_eq!((regions, keys), (5, wanted));
        let mut suppliers = Vec::new();
        supplier(9, &mut |row| suppliers.push(row.to_vec()));
        assert_eq!(super::super::text::TABLES[SUPPLIER as usize], "supplier");
        assert_eq!(suppliers.len(), SUPPLIERS as usize);
        assert!(
            suppliers.iter().all(|row| matches!(row[5], data::Lit::Fixed(v, 2) if v >= 1_000_000))
        );
    }

    #[test]
    fn the_report_has_a_line_a_query_and_the_errors_once() {
        assert!((geometric_mean(&[1.0, 4.0]) - 2.0).abs() < 1e-9);
        assert_eq!(geometric_mean(&[]), 0.0);
        let mut one = Ran::new();
        let mut two = Ran::new();
        one.times[0].record(2_000);
        two.times[0].record(2_000);
        one.failed(3, "no ascii".to_string());
        two.failed(3, "no ascii".to_string());
        two.failed(3, "no ascii".to_string());
        let mut out = String::new();
        report(&[one, two], Some(Duration::from_secs(3600)), &mut out);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 22 + 2, "{out}");
        assert!(lines[0].starts_with("ch query=Q1 n=2 median_ms=2.00 "), "{}", lines[0]);
        assert!(lines[1].starts_with("ch query=Q2 n=0 "), "{}", lines[1]);
        assert_eq!(lines[22], "ch error query=Q4 times=3 message=\"no ascii\"");
        assert!(lines[23].starts_with("ch summary streams=2 passes=0 answered=1 geomean_ms=2.00 "));
    }

    #[test]
    fn the_freshness_line_comes_once_a_query_was_judged() {
        let mut one = Ran::new();
        let mut two = Ran::new();
        one.judged(Judged::default());
        two.judged(Judged {
            stale: Duration::from_millis(4),
            ahead: 3,
            lead: Duration::from_micros(1_500),
        });
        two.judged(Judged::default());
        let mut out = String::new();
        report(&[one, two], Some(Duration::from_secs(60)), &mut out);
        let last = out.lines().last().unwrap_or_default();
        assert!(last.starts_with("ch freshness judged=3 late=1 stale_median_ms=0.00 "), "{last}");
        assert!(last.ends_with(" ahead=1 ahead_orders=3 lead_max_ms=1.50"), "{last}");
        let mut out = String::new();
        report(&[Ran::new()], None, &mut out);
        assert!(!out.contains("ch freshness"), "{out}");
    }
}
