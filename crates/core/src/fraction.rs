//! Exact numbers written by students: whole numbers, fractions, mixed numbers
//! and decimals, held as reduced integer ratios. Integers only (rule e).

/// A reduced ratio: `den` is always positive and shares no factor with `num`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ratio {
    pub num: i64,
    pub den: i64,
}

/// What a student wrote, read as a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Written {
    pub value: Ratio,
    /// Written in simplest form: a whole number, or a fraction (or the
    /// fraction part of a mixed number) whose top and bottom share no factor
    /// and whose bottom is not 1. Decimals are never simplest form.
    pub simplest: bool,
    pub decimal: bool,
}

/// The longest run of digits accepted in one part, so no sum can overflow.
const MAX_DIGITS: usize = 9;

/// Trim, collapse runs of spaces to one, drop spaces next to `/`, lower case.
pub fn normalize(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let joined = words.join(" ");
    joined.replace(" /", "/").replace("/ ", "/").to_lowercase()
}

fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

impl Ratio {
    /// Build a reduced ratio. Returns None when `den` is zero.
    pub fn new(num: i64, den: i64) -> Option<Ratio> {
        if den == 0 {
            return None;
        }
        let g = gcd(num, den);
        let sign = if den < 0 { -1 } else { 1 };
        Some(Ratio {
            num: sign * num / g,
            den: sign * den / g,
        })
    }
}

fn digits(part: &str) -> Option<i64> {
    if part.is_empty() || part.len() > MAX_DIGITS || !part.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    part.parse::<i64>().ok()
}

fn parse_fraction(part: &str) -> Option<(i64, i64)> {
    let (top, bottom) = part.split_once('/')?;
    let top = digits(top)?;
    let bottom = digits(bottom)?;
    if bottom == 0 {
        return None;
    }
    Some((top, bottom))
}

/// Read a student's answer as a number. Returns None when it is not one.
pub fn parse(text: &str) -> Option<Written> {
    let s = normalize(text);
    let (negative, rest) = match s.strip_prefix('-') {
        Some(r) => (true, r.trim_start()),
        None => (false, s.as_str()),
    };
    let sign = if negative { -1 } else { 1 };
    if let Some((whole, frac)) = rest.split_once(' ') {
        // Mixed number: whole part, one space, proper fraction part.
        let whole = digits(whole)?;
        let (top, bottom) = parse_fraction(frac)?;
        if top == 0 || top >= bottom {
            return None;
        }
        let value = Ratio::new(sign * (whole * bottom + top), bottom)?;
        let simplest = gcd(top, bottom) == 1;
        return Some(Written { value, simplest, decimal: false });
    }
    if rest.contains('/') {
        let (top, bottom) = parse_fraction(rest)?;
        let value = Ratio::new(sign * top, bottom)?;
        let simplest = gcd(top, bottom) == 1 && bottom != 1;
        return Some(Written { value, simplest, decimal: false });
    }
    if let Some((whole, frac)) = rest.split_once('.') {
        let whole = if whole.is_empty() { 0 } else { digits(whole)? };
        let top = digits(frac)?;
        let mut bottom: i64 = 1;
        for _ in 0..frac.len() {
            bottom *= 10;
        }
        let value = Ratio::new(sign * (whole * bottom + top), bottom)?;
        return Some(Written { value, simplest: false, decimal: true });
    }
    let whole = digits(rest)?;
    let value = Ratio::new(sign * whole, 1)?;
    Some(Written { value, simplest: true, decimal: false })
}
