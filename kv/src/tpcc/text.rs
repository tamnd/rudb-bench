//! The SQL of the TPC-C mode, one text per statement and sent unchanged to every engine, as the
//! spec's documents 02 and 03 write it. Parameters are `$1`, `$2` and so on, and a backend whose
//! engine numbers them some other way rewrites them through `Backend::numbered`.

/// The nine tables of TPC-C and the three CH-benCHmark adds, in the order they are created and
/// loaded.
pub(crate) const TABLES: [&str; 12] = [
    "warehouse",
    "district",
    "customer",
    "history",
    "orders",
    "new_order",
    "order_line",
    "item",
    "stock",
    "region",
    "nation",
    "supplier",
];

/// The schema of the spec's section 2.1 and of document 09 section 9.2, one statement a table, in
/// the order of [`TABLES`].
pub(crate) const SCHEMA: [&str; 12] = [
    "CREATE TABLE warehouse (
  w_id INTEGER NOT NULL, w_name VARCHAR(10), w_street_1 VARCHAR(20), w_street_2 VARCHAR(20),
  w_city VARCHAR(20), w_state CHAR(2), w_zip CHAR(9), w_tax DECIMAL(4,4), w_ytd DECIMAL(12,2),
  PRIMARY KEY (w_id))",
    "CREATE TABLE district (
  d_id INTEGER NOT NULL, d_w_id INTEGER NOT NULL, d_name VARCHAR(10), d_street_1 VARCHAR(20),
  d_street_2 VARCHAR(20), d_city VARCHAR(20), d_state CHAR(2), d_zip CHAR(9), d_tax DECIMAL(4,4),
  d_ytd DECIMAL(12,2), d_next_o_id INTEGER,
  PRIMARY KEY (d_w_id, d_id))",
    "CREATE TABLE customer (
  c_id INTEGER NOT NULL, c_d_id INTEGER NOT NULL, c_w_id INTEGER NOT NULL, c_first VARCHAR(16),
  c_middle CHAR(2), c_last VARCHAR(16), c_street_1 VARCHAR(20), c_street_2 VARCHAR(20),
  c_city VARCHAR(20), c_state CHAR(2), c_zip CHAR(9), c_phone CHAR(16), c_since TIMESTAMP,
  c_credit CHAR(2), c_credit_lim DECIMAL(12,2), c_discount DECIMAL(4,4), c_balance DECIMAL(12,2),
  c_ytd_payment DECIMAL(12,2), c_payment_cnt INTEGER, c_delivery_cnt INTEGER, c_data VARCHAR(500),
  PRIMARY KEY (c_w_id, c_d_id, c_id))",
    "CREATE TABLE history (
  h_c_id INTEGER, h_c_d_id INTEGER, h_c_w_id INTEGER, h_d_id INTEGER, h_w_id INTEGER,
  h_date TIMESTAMP, h_amount DECIMAL(6,2), h_data VARCHAR(24))",
    "CREATE TABLE orders (
  o_id INTEGER NOT NULL, o_d_id INTEGER NOT NULL, o_w_id INTEGER NOT NULL, o_c_id INTEGER,
  o_entry_d TIMESTAMP, o_carrier_id INTEGER, o_ol_cnt INTEGER, o_all_local INTEGER,
  PRIMARY KEY (o_w_id, o_d_id, o_id))",
    "CREATE TABLE new_order (
  no_o_id INTEGER NOT NULL, no_d_id INTEGER NOT NULL, no_w_id INTEGER NOT NULL,
  PRIMARY KEY (no_w_id, no_d_id, no_o_id))",
    "CREATE TABLE order_line (
  ol_o_id INTEGER NOT NULL, ol_d_id INTEGER NOT NULL, ol_w_id INTEGER NOT NULL,
  ol_number INTEGER NOT NULL, ol_i_id INTEGER, ol_supply_w_id INTEGER, ol_delivery_d TIMESTAMP,
  ol_quantity INTEGER, ol_amount DECIMAL(6,2), ol_dist_info CHAR(24),
  PRIMARY KEY (ol_w_id, ol_d_id, ol_o_id, ol_number))",
    "CREATE TABLE item (
  i_id INTEGER NOT NULL, i_im_id INTEGER, i_name VARCHAR(24), i_price DECIMAL(5,2),
  i_data VARCHAR(50),
  PRIMARY KEY (i_id))",
    "CREATE TABLE stock (
  s_i_id INTEGER NOT NULL, s_w_id INTEGER NOT NULL, s_quantity INTEGER,
  s_dist_01 CHAR(24), s_dist_02 CHAR(24), s_dist_03 CHAR(24), s_dist_04 CHAR(24),
  s_dist_05 CHAR(24), s_dist_06 CHAR(24), s_dist_07 CHAR(24), s_dist_08 CHAR(24),
  s_dist_09 CHAR(24), s_dist_10 CHAR(24), s_ytd INTEGER, s_order_cnt INTEGER,
  s_remote_cnt INTEGER, s_data VARCHAR(50),
  PRIMARY KEY (s_w_id, s_i_id))",
    "CREATE TABLE region (
  r_regionkey INTEGER PRIMARY KEY, r_name VARCHAR(55) NOT NULL, r_comment VARCHAR(152) NOT NULL)",
    "CREATE TABLE nation (
  n_nationkey INTEGER PRIMARY KEY, n_name VARCHAR(25) NOT NULL, n_regionkey INTEGER NOT NULL,
  n_comment VARCHAR(152) NOT NULL)",
    "CREATE TABLE supplier (
  su_suppkey INTEGER PRIMARY KEY, su_name VARCHAR(25) NOT NULL, su_address VARCHAR(40) NOT NULL,
  su_nationkey INTEGER NOT NULL, su_phone VARCHAR(15) NOT NULL, su_acctbal DECIMAL(12,2) NOT NULL,
  su_comment VARCHAR(101) NOT NULL)",
];

/// The two secondary indexes, each created after the load as its own timed statement.
pub(crate) const INDEXES: [(&str, &str); 2] = [
    ("customer_name", "CREATE INDEX customer_name ON customer (c_w_id, c_d_id, c_last, c_first)"),
    ("orders_customer", "CREATE INDEX orders_customer ON orders (o_w_id, o_d_id, o_c_id, o_id)"),
];

/// One statement of the five transactions, named as document 03 names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Id {
    No1,
    No2,
    No3,
    No4,
    No5,
    No6,
    No7,
    Pa1,
    Pa2,
    Pa3,
    Pa4,
    Pa5,
    Pa6,
    Os1,
    Os2,
    Os3,
    Os4,
    De1,
    De2,
    De3,
    De4,
    /// Whether a district really has no undelivered order, after `de1` deleted nothing.
    De5,
    Sl1,
    Sl2,
    /// The monitor's count of committed New-Orders, document 04 section 4.6.
    Orders,
}

impl Id {
    pub(crate) const ALL: [Self; 25] = [
        Self::No1,
        Self::No2,
        Self::No3,
        Self::No4,
        Self::No5,
        Self::No6,
        Self::No7,
        Self::Pa1,
        Self::Pa2,
        Self::Pa3,
        Self::Pa4,
        Self::Pa5,
        Self::Pa6,
        Self::Os1,
        Self::Os2,
        Self::Os3,
        Self::Os4,
        Self::De1,
        Self::De2,
        Self::De3,
        Self::De4,
        Self::De5,
        Self::Sl1,
        Self::Sl2,
        Self::Orders,
    ];

    pub(crate) const fn index(self) -> usize {
        self as usize
    }

    pub(crate) const fn text(self) -> &'static str {
        match self {
            Self::No1 => {
                "UPDATE district SET d_next_o_id = d_next_o_id + 1
 WHERE d_w_id = $1 AND d_id = $2
RETURNING d_next_o_id - 1, d_tax"
            }
            Self::No2 => "SELECT w_tax FROM warehouse WHERE w_id = $1",
            Self::No3 => {
                "SELECT c_discount, c_last, c_credit FROM customer
 WHERE c_w_id = $1 AND c_d_id = $2 AND c_id = $3"
            }
            Self::No4 => {
                "INSERT INTO orders (o_id, o_d_id, o_w_id, o_c_id, o_entry_d, o_carrier_id, o_ol_cnt, o_all_local)
VALUES ($1, $2, $3, $4, $5, NULL, $6, $7)"
            }
            Self::No5 => "INSERT INTO new_order (no_o_id, no_d_id, no_w_id) VALUES ($1, $2, $3)",
            Self::No6 => "SELECT i_price, i_name, i_data FROM item WHERE i_id = $1",
            Self::No7 => {
                "UPDATE stock
   SET s_quantity = CASE WHEN s_quantity >= $3 + 10 THEN s_quantity - $3
                         ELSE s_quantity - $3 + 91 END,
       s_ytd = s_ytd + $3,
       s_order_cnt = s_order_cnt + 1,
       s_remote_cnt = s_remote_cnt + $4
 WHERE s_w_id = $1 AND s_i_id = $2
RETURNING s_quantity, s_data,
       CASE $5 WHEN 1 THEN s_dist_01 WHEN 2 THEN s_dist_02 WHEN 3 THEN s_dist_03
               WHEN 4 THEN s_dist_04 WHEN 5 THEN s_dist_05 WHEN 6 THEN s_dist_06
               WHEN 7 THEN s_dist_07 WHEN 8 THEN s_dist_08 WHEN 9 THEN s_dist_09
               ELSE s_dist_10 END"
            }
            Self::Pa1 => {
                "UPDATE warehouse SET w_ytd = w_ytd + $2
 WHERE w_id = $1
RETURNING w_name, w_street_1, w_street_2, w_city, w_state, w_zip"
            }
            Self::Pa2 => {
                "UPDATE district SET d_ytd = d_ytd + $3
 WHERE d_w_id = $1 AND d_id = $2
RETURNING d_name, d_street_1, d_street_2, d_city, d_state, d_zip"
            }
            Self::Pa3 => {
                "SELECT c_id FROM customer
 WHERE c_w_id = $1 AND c_d_id = $2 AND c_last = $3
 ORDER BY c_first"
            }
            Self::Pa4 => {
                "UPDATE customer
   SET c_balance = c_balance - $4,
       c_ytd_payment = c_ytd_payment + $4,
       c_payment_cnt = c_payment_cnt + 1
 WHERE c_w_id = $1 AND c_d_id = $2 AND c_id = $3
RETURNING c_first, c_middle, c_last, c_street_1, c_street_2, c_city, c_state, c_zip,
          c_phone, c_since, c_credit, c_credit_lim, c_discount, c_balance"
            }
            Self::Pa5 => {
                "UPDATE customer SET c_data = substr($4 || c_data, 1, 500)
 WHERE c_w_id = $1 AND c_d_id = $2 AND c_id = $3"
            }
            Self::Pa6 => {
                "INSERT INTO history (h_c_id, h_c_d_id, h_c_w_id, h_d_id, h_w_id, h_date, h_amount, h_data)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
            }
            Self::Os1 => {
                "SELECT c_id, c_balance, c_first, c_middle, c_last FROM customer
 WHERE c_w_id = $1 AND c_d_id = $2 AND c_last = $3
 ORDER BY c_first"
            }
            Self::Os2 => {
                "SELECT c_balance, c_first, c_middle, c_last FROM customer
 WHERE c_w_id = $1 AND c_d_id = $2 AND c_id = $3"
            }
            Self::Os3 => {
                "SELECT o_id, o_entry_d, o_carrier_id FROM orders
 WHERE o_w_id = $1 AND o_d_id = $2 AND o_c_id = $3
 ORDER BY o_id DESC LIMIT 1"
            }
            Self::Os4 => {
                "SELECT ol_i_id, ol_supply_w_id, ol_quantity, ol_amount, ol_delivery_d FROM order_line
 WHERE ol_w_id = $1 AND ol_d_id = $2 AND ol_o_id = $3"
            }
            Self::De1 => {
                "DELETE FROM new_order
 WHERE no_w_id = $1 AND no_d_id = $2
   AND no_o_id = (SELECT min(no_o_id) FROM new_order WHERE no_w_id = $1 AND no_d_id = $2)
RETURNING no_o_id"
            }
            Self::De2 => {
                "UPDATE orders SET o_carrier_id = $4
 WHERE o_w_id = $1 AND o_d_id = $2 AND o_id = $3
RETURNING o_c_id"
            }
            Self::De3 => {
                "UPDATE order_line SET ol_delivery_d = $4
 WHERE ol_w_id = $1 AND ol_d_id = $2 AND ol_o_id = $3
RETURNING ol_amount"
            }
            Self::De4 => {
                "UPDATE customer
   SET c_balance = c_balance + $4, c_delivery_cnt = c_delivery_cnt + 1
 WHERE c_w_id = $1 AND c_d_id = $2 AND c_id = $3"
            }
            Self::De5 => "SELECT min(no_o_id) FROM new_order WHERE no_w_id = $1 AND no_d_id = $2",
            Self::Sl1 => "SELECT d_next_o_id FROM district WHERE d_w_id = $1 AND d_id = $2",
            Self::Sl2 => {
                "SELECT count(DISTINCT s_i_id) FROM order_line, stock
 WHERE ol_w_id = $1 AND ol_d_id = $2
   AND ol_o_id >= $3 - 20 AND ol_o_id < $3
   AND s_w_id = ol_w_id AND s_i_id = ol_i_id
   AND s_quantity < $4"
            }
            Self::Orders => "SELECT sum(d_next_o_id) FROM district",
        }
    }

    /// The statement's name in document 03, such as `no1`.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::No1 => "no1",
            Self::No2 => "no2",
            Self::No3 => "no3",
            Self::No4 => "no4",
            Self::No5 => "no5",
            Self::No6 => "no6",
            Self::No7 => "no7",
            Self::Pa1 => "pa1",
            Self::Pa2 => "pa2",
            Self::Pa3 => "pa3",
            Self::Pa4 => "pa4",
            Self::Pa5 => "pa5",
            Self::Pa6 => "pa6",
            Self::Os1 => "os1",
            Self::Os2 => "os2",
            Self::Os3 => "os3",
            Self::Os4 => "os4",
            Self::De1 => "de1",
            Self::De2 => "de2",
            Self::De3 => "de3",
            Self::De4 => "de4",
            Self::De5 => "de5",
            Self::Sl1 => "sl1",
            Self::Sl2 => "sl2",
            Self::Orders => "orders",
        }
    }

    /// How many rows and columns the statement returns in the common case, which is what the null
    /// backend hands back for it.
    pub(crate) const fn shape(self) -> (usize, usize) {
        match self {
            Self::No4 | Self::No5 | Self::Pa5 | Self::Pa6 | Self::De4 => (0, 0),
            Self::No1 => (1, 2),
            Self::No3 | Self::No6 | Self::No7 | Self::Os3 => (1, 3),
            Self::Pa1 | Self::Pa2 => (1, 6),
            Self::Pa3 => (3, 1),
            Self::Pa4 => (1, 14),
            Self::Os1 => (3, 5),
            Self::Os2 => (1, 4),
            Self::Os4 => (10, 5),
            Self::De3 => (10, 1),
            Self::No2
            | Self::De1
            | Self::De2
            | Self::De5
            | Self::Sl1
            | Self::Sl2
            | Self::Orders => (1, 1),
        }
    }

    /// The statement whose text this is, if any.
    pub(crate) fn of(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|id| id.text() == text)
    }
}

/// The fewest and the most lines an order has.
pub(crate) const MIN_LINES: usize = 5;
pub(crate) const MAX_LINES: usize = 15;

/// `no8`, the multi-row insert of an order's `lines` lines, nine parameters a line.
pub(crate) fn order_lines(lines: usize) -> String {
    let mut text = String::from(
        "INSERT INTO order_line (ol_o_id, ol_d_id, ol_w_id, ol_number, ol_i_id, ol_supply_w_id,
                        ol_delivery_d, ol_quantity, ol_amount, ol_dist_info)
VALUES ",
    );
    for line in 0..lines {
        let at = line * 9;
        if line > 0 {
            text.push_str(", ");
        }
        text.push_str(&format!(
            "(${}, ${}, ${}, ${}, ${}, ${}, NULL, ${}, ${}, ${})",
            at + 1,
            at + 2,
            at + 3,
            at + 4,
            at + 5,
            at + 6,
            at + 7,
            at + 8,
            at + 9
        ));
    }
    text
}

/// Rewrites `$1`, `$2` and so on as `?1`, `?2`, which SQLite binds by the number written. SQLite
/// reads `$1` as a parameter named `$1` and numbers those in the order they first appear, and
/// `no7` uses `$3` before `$1`.
pub(crate) fn question_numbered(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' && chars.peek().is_some_and(char::is_ascii_digit) {
            out.push('?');
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_statement_is_found_by_its_text() {
        for id in Id::ALL {
            assert_eq!(Id::of(id.text()), Some(id));
        }
        assert_eq!(Id::ALL.iter().enumerate().filter(|(at, id)| id.index() != *at).count(), 0);
        assert_eq!(Id::of("SELECT 1"), None);
    }

    #[test]
    fn an_order_of_n_lines_has_nine_parameters_a_line() {
        let text = order_lines(5);
        assert!(text.ends_with("($37, $38, $39, $40, $41, $42, NULL, $43, $44, $45)"), "{text}");
        assert_eq!(text.matches('$').count(), 45);
    }

    #[test]
    fn sqlite_gets_numbered_question_marks() {
        assert_eq!(
            question_numbered("SELECT $12 + $3, '$' FROM t WHERE a = $1"),
            "SELECT ?12 + ?3, '$' FROM t WHERE a = ?1"
        );
    }
}
