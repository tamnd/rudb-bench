//! What the clients do: the keys, the values, the statements and the mix of operations.
//!
//! Keys formatted the way YCSB formats them, values a read can check for being the right row and
//! the right field, and the three ways YCSB draws a key: uniform, zipfian with the hot keys
//! scattered over the key space, and latest, where the newest rows are the hot ones.

use crate::backend::Placeholder;

/// YCSB's 64 bit FNV hash of a record number, which spreads inserted keys over the key space.
///
/// It is FNV-1a over the eight bytes of the value from the lowest, and the absolute value of the
/// result taken as a signed number, which is what `Utils.fnvhash64` in YCSB's Java does.
pub(crate) fn fnvhash64(mut value: u64) -> u64 {
    let mut hash: u64 = 0xCBF2_9CE4_8422_2325;
    for _ in 0..8 {
        let octet = value & 0xff;
        value >>= 8;
        hash ^= octet;
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    (hash as i64).unsigned_abs()
}

/// The key of record `n`.
pub(crate) fn key(n: u64) -> String {
    format!("user{}", fnvhash64(n))
}

/// How long every field is, which is YCSB's `fieldlength` default.
pub(crate) const FIELD_LENGTH: usize = 100;

/// The fields in a row.
pub(crate) const FIELDS: usize = 10;

/// The value of field `field` of the row `key`, as written by `version`.
///
/// It starts with the key and the field, so a read that returns it can say whether it got the row
/// and the field it asked for, and it is always [`FIELD_LENGTH`] bytes.
pub(crate) fn value(key: &str, field: usize, version: u64) -> String {
    const FILL: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut text = format!("{key}:{field}:{version}:");
    let mut at = 0;
    while text.len() < FIELD_LENGTH {
        text.push(char::from(FILL[at % FILL.len()]));
        at += 1;
    }
    text.truncate(FIELD_LENGTH);
    text
}

/// Whether `bytes` is a value [`value`] could have written for this key and field.
pub(crate) fn well_formed(key: &str, field: usize, bytes: &[u8]) -> bool {
    let prefix = format!("{key}:{field}:");
    bytes.len() == FIELD_LENGTH && bytes.starts_with(prefix.as_bytes())
}

/// xoshiro256**, seeded through SplitMix64, which is small, fast and the same on every platform.
#[derive(Debug, Clone)]
pub(crate) struct Rng([u64; 4]);

impl Rng {
    pub(crate) fn new(seed: u64) -> Self {
        let mut state = seed;
        let mut next = || {
            state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        };
        Self([next(), next(), next(), next()])
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        let s = &mut self.0;
        let result = s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
        result
    }

    /// A number in `[0, 1)`.
    pub(crate) fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1_u64 << 53) as f64
    }

    /// A number in `[0, below)`.
    pub(crate) fn below(&mut self, below: u64) -> u64 {
        ((u128::from(self.next_u64()) * u128::from(below)) >> 64) as u64
    }
}

/// YCSB's zipfian constant.
const THETA: f64 = 0.99;

/// The items the scrambled zipfian draws from before it hashes, and the zeta of that many at
/// [`THETA`], which are `ScrambledZipfianGenerator.ITEM_COUNT` and `ZETAN` in YCSB's Java.
const SCRAMBLED_ITEMS: u64 = 10_000_000_000;
const SCRAMBLED_ZETAN: f64 = 26.469_028_201_783_02;

/// The sum of `1 / i^theta` for `i` from `from + 1` to `to`.
fn zeta(from: u64, to: u64) -> f64 {
    (from..to).map(|i| 1.0 / ((i + 1) as f64).powf(THETA)).sum()
}

/// Gray et al.'s zipfian over `[0, items)`, YCSB's `ZipfianGenerator`, where item 0 is the most
/// often drawn. The number of items can grow, and the zeta grows with it a term at a time.
#[derive(Debug, Clone)]
pub(crate) struct Zipfian {
    items: u64,
    zetan: f64,
    eta: f64,
}

impl Zipfian {
    fn new(items: u64, zetan: f64) -> Self {
        let mut zipfian = Self { items, zetan, eta: 0.0 };
        zipfian.eta = zipfian.eta_for();
        zipfian
    }

    fn eta_for(&self) -> f64 {
        let zeta2 = zeta(0, 2);
        (1.0 - (2.0 / self.items as f64).powf(1.0 - THETA)) / (1.0 - zeta2 / self.zetan)
    }

    /// One draw over `items`, which is the count it was made for or more.
    fn next(&mut self, rng: &mut Rng, items: u64) -> u64 {
        if items > self.items {
            self.zetan += zeta(self.items, items);
            self.items = items;
            self.eta = self.eta_for();
        }
        let u = rng.next_f64();
        let uz = u * self.zetan;
        if uz < 1.0 {
            return 0;
        }
        if uz < 1.0 + 0.5_f64.powf(THETA) {
            return 1;
        }
        let drawn = self.items as f64 * (self.eta * u - self.eta + 1.0).powf(1.0 / (1.0 - THETA));
        (drawn as u64).min(self.items - 1)
    }
}

/// How a key is drawn, the YCSB spec's `requestdistribution`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Distribution {
    Uniform,
    Zipfian,
    Latest,
}

impl Distribution {
    pub(crate) fn parse(text: &str) -> Result<Self, String> {
        match text {
            "uniform" => Ok(Self::Uniform),
            "zipfian" => Ok(Self::Zipfian),
            "latest" => Ok(Self::Latest),
            _ => Err(format!("--distribution is uniform, zipfian or latest, not {text:?}")),
        }
    }

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Uniform => "uniform",
            Self::Zipfian => "zipfian",
            Self::Latest => "latest",
        }
    }
}

/// One client's way of drawing the record number of an existing row.
#[derive(Debug, Clone)]
pub(crate) struct Chooser {
    distribution: Distribution,
    zipfian: Option<Zipfian>,
}

impl Chooser {
    /// A chooser over the first `records` rows, which it is told grow as rows go in.
    pub(crate) fn new(distribution: Distribution, records: u64) -> Self {
        let zipfian = match distribution {
            Distribution::Uniform => None,
            Distribution::Zipfian => Some(Zipfian::new(SCRAMBLED_ITEMS, SCRAMBLED_ZETAN)),
            Distribution::Latest => Some(Zipfian::new(records, zeta(0, records))),
        };
        Self { distribution, zipfian }
    }

    /// A record number below `rows`, the rows every client can be sure are there.
    ///
    /// Zipfian is YCSB's scrambled zipfian: a draw over ten billion items hashed into the rows, so
    /// the hot rows are spread over the key space rather than bunched at the start of it. Latest is
    /// YCSB's skewed latest: the newest row is the most often drawn, then the one before it.
    pub(crate) fn next(&mut self, rng: &mut Rng, rows: u64) -> u64 {
        match (self.distribution, self.zipfian.as_mut()) {
            (Distribution::Zipfian, Some(zipfian)) => {
                fnvhash64(zipfian.next(rng, SCRAMBLED_ITEMS)) % rows
            }
            (Distribution::Latest, Some(zipfian)) => {
                rows - 1 - zipfian.next(rng, rows).min(rows - 1)
            }
            _ => rng.below(rows),
        }
    }
}

/// How long a scan is at most, YCSB's `maxscanlength` in workload E. The length is uniform from one
/// to this.
pub(crate) const MAX_SCAN: u64 = 100;

/// One kind of operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Op {
    Read,
    Update,
    Insert,
    Scan,
}

impl Op {
    pub(crate) const COUNT: usize = 4;

    pub(crate) const ALL: [Self; Self::COUNT] =
        [Self::Read, Self::Update, Self::Insert, Self::Scan];

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Update => "update",
            Self::Insert => "insert",
            Self::Scan => "scan",
        }
    }

    pub(crate) const fn index(self) -> usize {
        self as usize
    }
}

/// How often each operation comes up, in parts of the whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Mix {
    parts: [u32; Op::COUNT],
}

impl Mix {
    /// `read=50,update=50`, where a missing operation is zero.
    pub(crate) fn parse(text: &str) -> Result<Self, String> {
        let mut parts = [0; Op::COUNT];
        for pair in text.split(',').filter(|pair| !pair.is_empty()) {
            let (name, part) = pair
                .split_once('=')
                .ok_or_else(|| format!("--mix wants op=parts pairs, and {pair:?} is not one"))?;
            let op = Op::ALL
                .into_iter()
                .find(|op| op.name() == name)
                .ok_or_else(|| format!("--mix has no operation {name:?}"))?;
            parts[op.index()] = part.parse().map_err(|_| format!("--mix has {pair:?}"))?;
        }
        if parts.iter().sum::<u32>() == 0 {
            return Err("--mix has nothing in it".to_string());
        }
        Ok(Self { parts })
    }

    pub(crate) fn text(&self) -> String {
        Op::ALL
            .into_iter()
            .filter(|op| self.parts[op.index()] > 0)
            .map(|op| format!("{}={}", op.name(), self.parts[op.index()]))
            .collect::<Vec<_>>()
            .join(",")
    }

    pub(crate) fn has(&self, op: Op) -> bool {
        self.parts[op.index()] > 0
    }

    pub(crate) fn pick(&self, rng: &mut Rng) -> Op {
        let total: u32 = self.parts.iter().sum();
        let mut draw = rng.below(u64::from(total)) as u32;
        for op in Op::ALL {
            if draw < self.parts[op.index()] {
                return op;
            }
            draw -= self.parts[op.index()];
        }
        Op::Read
    }
}

/// The texts of the statements the clients run, in the canonical form of the YCSB spec's section
/// 3.2 with this engine's placeholders.
#[derive(Debug, Clone)]
pub(crate) struct Texts {
    pub(crate) read: String,
    pub(crate) update: Vec<String>,
    pub(crate) insert: String,
    pub(crate) scan: String,
}

impl Texts {
    pub(crate) fn new(placeholder: Placeholder) -> Self {
        let mark = |n: usize| match placeholder {
            Placeholder::Question => "?".to_string(),
            Placeholder::Dollar => format!("${n}"),
        };
        let read = format!("SELECT * FROM usertable WHERE ycsb_key = {}", mark(1));
        let update = (0..FIELDS)
            .map(|field| {
                format!(
                    "UPDATE usertable SET field{field} = {} WHERE ycsb_key = {}",
                    mark(1),
                    mark(2)
                )
            })
            .collect();
        let columns: Vec<String> = (0..FIELDS).map(|field| format!("field{field}")).collect();
        let marks: Vec<String> = (1..=FIELDS + 1).map(mark).collect();
        let insert = format!(
            "INSERT INTO usertable (ycsb_key, {}) VALUES ({})",
            columns.join(", "),
            marks.join(", ")
        );
        let scan = format!(
            "SELECT * FROM usertable WHERE ycsb_key >= {} ORDER BY ycsb_key LIMIT {}",
            mark(1),
            mark(2)
        );
        Self { read, update, insert, scan }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_the_ones_ycsb_makes() {
        // The first key of every YCSB load with the default hashed insert order.
        assert_eq!(key(0), "user6284781860667377211");
        assert!(key(1).starts_with("user"));
        assert_ne!(key(1), key(2));
    }

    #[test]
    fn a_value_is_the_right_length_and_checks_as_its_own_row() {
        let key = key(7);
        let written = value(&key, 3, 42);
        assert_eq!(written.len(), FIELD_LENGTH);
        assert!(well_formed(&key, 3, written.as_bytes()));
        assert!(!well_formed(&key, 4, written.as_bytes()));
        assert!(!well_formed(&self::key(8), 3, written.as_bytes()));
    }

    #[test]
    fn a_mix_draws_in_proportion() {
        let mix = Mix::parse("read=95,update=5").unwrap();
        assert_eq!(mix.text(), "read=95,update=5");
        let mut rng = Rng::new(1);
        let reads = (0..100_000).filter(|_| mix.pick(&mut rng) == Op::Read).count();
        assert!((94_000..96_000).contains(&reads), "{reads}");
        assert!(Mix::parse("delete=5").is_err());
        let mix = Mix::parse("scan=95,insert=5").unwrap();
        assert!(mix.has(Op::Scan) && !mix.has(Op::Read));
    }

    #[test]
    fn postgres_is_the_one_with_numbered_placeholders() {
        let texts = Texts::new(Placeholder::Dollar);
        assert_eq!(texts.update[9], "UPDATE usertable SET field9 = $1 WHERE ycsb_key = $2");
        assert!(texts.insert.ends_with("$10, $11)"));
        let texts = Texts::new(Placeholder::Question);
        assert_eq!(texts.read, "SELECT * FROM usertable WHERE ycsb_key = ?");
        assert!(texts.scan.ends_with("ORDER BY ycsb_key LIMIT ?"));
    }

    #[test]
    fn a_zipfian_draws_its_first_items_most() {
        let mut zipfian = Zipfian::new(1000, zeta(0, 1000));
        let mut rng = Rng::new(3);
        let mut counts = vec![0_u32; 1000];
        for _ in 0..200_000 {
            counts[zipfian.next(&mut rng, 1000) as usize] += 1;
        }
        // At theta 0.99 over a thousand items the first takes about 1/zeta(1000), some 13 percent.
        assert!((24_000..28_000).contains(&counts[0]), "{}", counts[0]);
        assert!(counts[0] > counts[1] && counts[1] > counts[9] && counts[9] > counts[500]);
        // Growing the items extends the zeta to what it would have been made with.
        zipfian.next(&mut rng, 2000);
        assert!((zipfian.zetan - zeta(0, 2000)).abs() < 1e-9);
    }

    #[test]
    fn the_scrambled_zeta_is_the_one_ycsb_uses() {
        // Summing ten billion terms takes too long for a test, so check the start and the shape.
        assert!((zeta(0, 2) - (1.0 + 0.5_f64.powf(THETA))).abs() < 1e-12);
        let mut chooser = Chooser::new(Distribution::Zipfian, 1000);
        let mut rng = Rng::new(5);
        let mut counts = vec![0_u32; 1000];
        for _ in 0..100_000 {
            counts[chooser.next(&mut rng, 1000) as usize] += 1;
        }
        let hottest = counts.iter().copied().max().unwrap();
        assert!(hottest > 1_000, "{hottest}");
        // Hashed, so the hottest row is not the first.
        assert_ne!(counts.iter().position(|&count| count == hottest), Some(0));
    }

    #[test]
    fn latest_draws_the_newest_rows_most_and_never_past_them() {
        let mut chooser = Chooser::new(Distribution::Latest, 100);
        let mut rng = Rng::new(9);
        let mut newest = 0;
        for at in 0..50_000_u64 {
            let rows = 100 + at / 100;
            let drawn = chooser.next(&mut rng, rows);
            assert!(drawn < rows);
            if drawn == rows - 1 {
                newest += 1;
            }
        }
        assert!(newest > 5_000, "{newest}");
        let mut uniform = Chooser::new(Distribution::Uniform, 10);
        assert!((0..1000).all(|_| uniform.next(&mut rng, 10) < 10));
    }
}
