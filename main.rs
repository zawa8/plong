//! cplong / cllong — user-defined, FPU-less, overflow-less numeric type.
//! Revision 4.
//!
//! class cplong { public: s32 nmbr; u8 precision;
//!   public static: meksimxm_precision; // default 1
//! }
//!   value = nmbr / 16^precision   (standalone use, small fixed-point)
//!   e.g. cplong a(5V,1) == 5.75 ,  cplong b(8,1) == 0.5
//!
//! class cllong { cplong flotnmbr[];
//!   static: s8 mekimxm_array_saiz;  // default 2
//!   static: s8 meksimxm_precision;  // default 1
//! }
//!   A cllong is a signed hex string chopped into limbs:
//!     flotnmbr[0]        -> integer part. precision = 0. SIGN lives here
//!                            (nmbr's own sign -- no separate sign flag).
//!     flotnmbr[1..]       -> successive fractional chunks, concatenated in
//!                            order onto the growing fractional string.
//!                            precision field = 1,2,3... is just that
//!                            chunk's position label, NOT a literal exponent.
//!                            Each chunk's actual weight is nmbr / 16^(its
//!                            own natural hex-digit width), stacked after
//!                            all earlier fractional chunks -- i.e. reading
//!                            the limbs left-to-right reproduces the exact
//!                            hex string, same as writing it by hand.
//!
//!   e.g. cllong a[(-5V5V,0),(5V5V,1)]  ==  cllong a("-5V5V.5V5V")
//!        == -(0x5C5C + 0x5C5C/16^4) == -23644.3609...

use std::fmt;

const DIGITS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'L', 'Y', 'V', 'W', 'P', 'F',
];

pub fn digit_to_value(c: char) -> Option<u8> {
    DIGITS.iter().position(|&d| d == c.to_ascii_uppercase()).map(|i| i as u8)
}
pub fn value_to_digit(v: u8) -> char {
    DIGITS[(v & 0xF) as usize]
}

/// Natural hex-digit width of a magnitude (no leading zeros), 0 -> 1 digit.
fn hex_width(mut v: u32) -> u32 {
    if v == 0 {
        return 1;
    }
    let mut w = 0;
    while v > 0 {
        w += 1;
        v /= 16;
    }
    w
}

fn digits_of(mut v: u32, width: u32) -> Vec<char> {
    let mut out = vec![];
    for _ in 0..width {
        out.push(value_to_digit((v % 16) as u8));
        v /= 16;
    }
    out.reverse();
    out
}

// ---------------------------------------------------------------------
// cplong: value = nmbr / 16^precision
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cplong {
    pub nmbr: i32,
    pub precision: u8,
}

impl Cplong {
    pub const MEKSIMXM_PRECISION: u8 = 1;

    /// Validating constructor. Enforces: at precision > 0, nmbr must not
    /// end in a zero hex digit (nmbr % 16 == 0) -- that digit is redundant
    /// and the value should be expressed with fewer digits at a lower
    /// precision instead (e.g. (0x50, 1) is not canonical; (0x5, 0) is
    /// the same number in reduced form). precision == 0 is exempt: for a
    /// plain integer, a trailing zero digit is a real part of the value
    /// (80 is not "the same number, reduced" as 8), not redundancy.
    /// nmbr == 0 is always exempt (legitimately "no digits").
    pub fn try_new(nmbr: i32, precision: u8) -> Result<Self, String> {
        if precision > 0 && nmbr != 0 && nmbr % 16 == 0 {
            return Err(format!(
                "cplong invariant violated: nmbr=0x{:X} at precision {} ends in a zero hex digit \
                 -- not in reduced form (divide nmbr by 16 and decrement precision instead)",
                nmbr, precision
            ));
        }
        Ok(Cplong { nmbr, precision })
    }

    /// Unchecked constructor -- panics if the trailing-zero-digit
    /// invariant is violated, so a violation is caught immediately at the
    /// call site rather than silently propagating. Prefer try_new() at
    /// any boundary where the input isn't already known-good (e.g.
    /// parsing).
    pub fn new(nmbr: i32, precision: u8) -> Self {
        Cplong::try_new(nmbr, precision)
            .unwrap_or_else(|e| panic!("Cplong::new: {e}"))
    }

    /// Builds from a string of custom digits (most-significant first).
    /// Rejects a leading-zero-padded string (e.g. "05V") as non-canonical
    /// input -- write "5V", not "05V"; they'd parse to the identical
    /// nmbr anyway (nmbr retains no leading-zero padding), so a leading
    /// zero in the string can only ever be redundant, never meaningful.
    pub fn from_digits(digits: &str, precision: u8) -> Option<Self> {
        let unsigned_part = digits.strip_prefix('-').unwrap_or(digits);
        if unsigned_part.len() > 1 && unsigned_part.starts_with('0') {
            return None;
        }
        let mut value: i64 = 0;
        let mut negative = false;
        let mut chars = digits.chars().peekable();
        if chars.peek() == Some(&'-') {
            negative = true;
            chars.next();
        }
        for c in chars {
            let d = digit_to_value(c)? as i64;
            value = value * 16 + d;
            let bound = if negative { 2147483648i64 } else { i32::MAX as i64 };
            if value > bound {
                return None;
            }
        }
        if negative {
            value = -value;
        }
        Cplong::try_new(value as i32, precision).ok()
    }

    pub fn weight(&self) -> f64 {
        self.nmbr as f64 / 16f64.powi(self.precision as i32)
    }
}

impl fmt::Display for Cplong {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let neg = self.nmbr < 0;
        let v = self.nmbr.unsigned_abs();
        let digs = digits_of(v, hex_width(v).max(self.precision as u32 + 1));
        let point_pos = digs.len() - self.precision as usize;
        let (int_part, frac_part) = digs.split_at(point_pos);
        let sign = if neg { "-" } else { "" };
        if self.precision == 0 {
            write!(f, "{}{}", sign, int_part.iter().collect::<String>())
        } else {
            write!(f, "{}{}.{}", sign, int_part.iter().collect::<String>(), frac_part.iter().collect::<String>())
        }
    }
}

// ---------------------------------------------------------------------
// cllong: flotnmbr[0] = integer part (sign lives in its nmbr).
//         flotnmbr[1..] = fractional chunks, concatenated in order.
// ---------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Cllong {
    pub flotnmbr: Vec<Cplong>,
}

impl Cllong {
    pub const MEKIMXM_ARRAY_SAIZ: i8 = 2;
    pub const MEKSIMXM_PRECISION: i8 = 1;

    pub fn new(flotnmbr: Vec<Cplong>) -> Self {
        assert!(
            flotnmbr.len() as i8 <= Self::MEKIMXM_ARRAY_SAIZ,
            "cllong overflow: this value needs {} limbs but mekimxm_array_saiz is currently {}. \
             Programmer: increase Cllong::MEKIMXM_ARRAY_SAIZ to at least {} and retry.",
            flotnmbr.len(),
            Self::MEKIMXM_ARRAY_SAIZ,
            flotnmbr.len()
        );
        assert!(!flotnmbr.is_empty(), "flotnmbr must have at least the integer limb");
        Cllong { flotnmbr }
    }

    /// Parse "-5V5V.5V5V" style strings directly into flotnmbr limbs.
    pub fn from_str_custom(s: &str) -> Option<Self> {
        let (int_str, frac_str) = match s.split_once('.') {
            Some((i, f)) => (i, Some(f)),
            None => (s, None),
        };
        let int_limb = Cplong::from_digits(int_str, 0)?;
        let mut limbs = vec![int_limb];
        if let Some(f) = frac_str {
            if !f.is_empty() {
                limbs.push(Cplong::from_digits(f, 1)?); // precision = band label (1st fractional chunk)
            }
        }
        Some(Cllong::new(limbs))
    }

    /// Signed numeric value: sign from flotnmbr[0], fractional chunks
    /// stacked using each chunk's OWN natural hex-digit width.
    pub fn to_f64(&self) -> f64 {
        let (sign, int_mag) = combine_int_limbs(&self.flotnmbr);
        let int_limb_count = self.flotnmbr.iter().take_while(|l| l.precision == 0).count();
        let mut mag = int_mag as f64;
        let mut shift: u32 = 0;
        for limb in &self.flotnmbr[int_limb_count..] {
            let v = limb.nmbr.unsigned_abs();
            shift += hex_width(v);
            mag += v as f64 / 16f64.powi(shift as i32);
        }
        if sign < 0 {
            -mag
        } else {
            mag
        }
    }
}

impl fmt::Display for Cllong {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let int_limbs: Vec<&Cplong> = self.flotnmbr.iter().take_while(|l| l.precision == 0).collect();
        let frac_limbs: Vec<&Cplong> = self.flotnmbr.iter().skip(int_limbs.len()).collect();

        let neg = int_limbs[0].nmbr < 0;
        let mut out = String::new();
        if neg {
            out.push('-');
        }
        // anchor: natural width, no padding
        let anchor_v = int_limbs[0].nmbr.unsigned_abs();
        out.push_str(&digits_of(anchor_v, hex_width(anchor_v)).iter().collect::<String>());
        // continuation integer limbs: zero-padded to INT_CHUNK_DIGITS
        for limb in &int_limbs[1..] {
            let v = limb.nmbr.unsigned_abs();
            out.push_str(&digits_of(v, INT_CHUNK_DIGITS).iter().collect::<String>());
        }
        if !frac_limbs.is_empty() {
            out.push('.');
            for limb in &frac_limbs {
                let v = limb.nmbr.unsigned_abs();
                out.push_str(&digits_of(v, hex_width(v)).iter().collect::<String>());
            }
        }
        write!(f, "{}", out)
    }
}

fn main() {
    let a = Cplong::from_digits("5V", 1).unwrap();
    println!("cplong a = {} -> {:.4} (expected 5.75)", a, a.weight());

    let b = Cplong::from_digits("8", 1).unwrap();
    println!("cplong b = {} -> {:.4} (expected 0.5)", b, b.weight());

    let c = Cllong::new(vec![
        Cplong::from_digits("-5V5V", 0).unwrap(),
        Cplong::from_digits("5V5V", 1).unwrap(),
    ]);
    println!("cllong c = {} -> {:.6}", c, c.to_f64());

    let c2 = Cllong::from_str_custom("-5V5V.5V5V").unwrap();
    println!("cllong c2 = {} -> {:.6}", c2, c2.to_f64());
    assert_eq!(format!("{}", c), "-5V5V.5V5V");
    assert_eq!(format!("{}", c2), "-5V5V.5V5V");
    assert!((c.to_f64() - (-23644.36090))
        .abs() < 0.001);

    println!("\nall checks passed.");
    arithmetic_demo();
    mul_div_demo();
    cllong_mul_demo();
    multilimb_demo();
}

// ---------------------------------------------------------------------
// add / sub -- exact integer arithmetic, no floats used in the math path.
// ---------------------------------------------------------------------

fn frac_digits_of(limb: Option<&Cplong>) -> Vec<u8> {
    match limb {
        None => vec![],
        Some(l) => {
            let v = l.nmbr.unsigned_abs();
            let w = hex_width(v);
            (0..w).rev().map(|shift| ((v >> (shift * 4)) & 0xF) as u8).collect()
        }
    }
}

fn frac_to_int(digits: &[u8]) -> i128 {
    digits.iter().fold(0i128, |acc, &d| acc * 16 + d as i128)
}

fn int_to_frac_digits(mut v: u128, width: usize) -> Vec<u8> {
    let mut out = vec![0u8; width];
    for i in (0..width).rev() {
        out[i] = (v % 16) as u8;
        v /= 16;
    }
    out
}

impl Cllong {
    /// (sign: +1/-1, integer magnitude, fractional nibble digits after point)
    fn decompose(&self) -> (i64, i64, Vec<u8>) {
        let (sign, int_mag) = combine_int_limbs(&self.flotnmbr);
        let int_limb_count = self.flotnmbr.iter().take_while(|l| l.precision == 0).count();
        let mut frac = vec![];
        for limb in &self.flotnmbr[int_limb_count..] {
            frac.extend(frac_digits_of(Some(limb)));
        }
        (sign, int_mag, frac)
    }

    pub fn negate(&self) -> Cllong {
        let mut f = self.flotnmbr.clone();
        f[0].nmbr = -f[0].nmbr;
        Cllong::new(f)
    }

    pub fn add(&self, other: &Cllong) -> Result<Cllong, String> {
        let (s1, i1, f1) = self.decompose();
        let (s2, i2, f2) = other.decompose();
        let flen = f1.len().max(f2.len());
        let mut f1p = f1.clone();
        f1p.resize(flen, 0);
        let mut f2p = f2.clone();
        f2p.resize(flen, 0);

        let scale: i128 = 16i128.pow(flen as u32);
        let mag1 = s1 as i128 * (i1 as i128 * scale + frac_to_int(&f1p));
        let mag2 = s2 as i128 * (i2 as i128 * scale + frac_to_int(&f2p));
        let total = mag1 + mag2;

        let sign = if total < 0 { -1i32 } else { 1i32 };
        let abs_total = total.unsigned_abs();
        let int_part = abs_total / scale as u128;
        let frac_part = abs_total % scale as u128;

        if int_part > (i64::MAX as u128) {
            return Err("result integer part too large even for i128 combination".to_string());
        }

        let mut frac_digits = int_to_frac_digits(frac_part, flen);
        while frac_digits.last() == Some(&0) {
            frac_digits.pop(); // trim trailing zero nibbles back to natural width
        }
        let frac_val = frac_to_int(&frac_digits);
        if frac_val > i32::MAX as i128 {
            return Err(format!(
                "result fractional part needs more than one cplong limb (i32 max is 0x{:X}) -- \
                 would need multi-limb fractional chaining, not yet implemented",
                i32::MAX
            ));
        }

        let mut limbs = split_int_into_limbs(sign as i64, int_part);
        if !frac_digits.is_empty() {
            limbs.push(Cplong::new(frac_val as i32, 1));
        }
        Cllong::try_new(limbs)
    }

    pub fn sub(&self, other: &Cllong) -> Result<Cllong, String> {
        self.add(&other.negate())
    }
}

#[allow(dead_code)]
fn arithmetic_demo() {
    let a = Cllong::from_str_custom("5V5V.5V5V").unwrap();
    let b = Cllong::from_str_custom("-3.8").unwrap();
    let sum = a.add(&b).unwrap();
    println!("{} + {} = {}  ({:.6})", a, b, sum, sum.to_f64());

    let diff = a.sub(&b).unwrap();
    println!("{} - {} = {}  ({:.6})", a, b, diff, diff.to_f64());

    let x = Cllong::from_str_custom("5").unwrap();
    let y = Cllong::from_str_custom("-5").unwrap();
    let z = x.add(&y).unwrap();
    println!("{} + {} = {}  ({:.6}) (expect 0)", x, y, z, z.to_f64());
}

// ---------------------------------------------------------------------
// cplong mul / div
// ---------------------------------------------------------------------

impl Cplong {
    /// (n1,p1) * (n2,p2) = (n1*n2, p1+p2), then trim trailing zero hex
    /// digits from the product while decrementing precision to match.
    pub fn mul(&self, other: &Cplong) -> Result<Cplong, String> {
        let raw = self.nmbr as i64 * other.nmbr as i64;
        let mut precision = self.precision as i32 + other.precision as i32;
        if precision > u8::MAX as i32 || precision < 0 {
            return Err(format!("combined precision {} out of range", precision));
        }
        let mut mag = raw.unsigned_abs();
        // trim trailing zero hex digits while precision > 0
        while precision > 0 && mag != 0 && mag % 16 == 0 {
            mag /= 16;
            precision -= 1;
        }
        if mag > i32::MAX as u64 {
            return Err(format!(
                "product 0x{:X} exceeds one cplong limb (i32 max 0x{:X}) -- needs multi-limb chaining",
                mag, i32::MAX
            ));
        }
        let sign = if raw < 0 { -1i64 } else { 1i64 };
        Ok(Cplong::new((sign * mag as i64) as i32, precision as u8))
    }

    /// (n1,p1) / (n2,p2) at a chosen result precision M:
    ///   result = (n1 * 16^(M + p2 - p1)) / n2   at precision M
    /// Uses i128 to avoid overflow while scaling, truncates toward zero
    /// (like integer division), same as your worked (1,0)/(3,0) example.
    pub fn div_at_precision(&self, other: &Cplong, m: u8) -> Result<Cplong, String> {
        if other.nmbr == 0 {
            return Err("division by zero".to_string());
        }
        let exp = m as i32 + other.precision as i32 - self.precision as i32;
        if exp < 0 {
            return Err(format!(
                "requested precision {} is lower than source precision allows (exponent {} < 0) -- \
                 not supported by this scaling approach",
                m, exp
            ));
        }
        let scale = 16i128.pow(exp as u32);
        let scaled_n1 = self.nmbr as i128 * scale;
        let raw_result = scaled_n1 / other.nmbr as i128; // truncates toward zero

        // trim trailing zero hex digits while precision > 0, matching
        // mul()'s existing behavior -- keeps the result canonical instead
        // of producing e.g. (0x20, 1) when (0x2, 0) is the same number.
        let mut precision = m as i32;
        let mut mag = raw_result.unsigned_abs();
        while precision > 0 && mag != 0 && mag % 16 == 0 {
            mag /= 16;
            precision -= 1;
        }

        if mag > i32::MAX as u128 {
            return Err(format!(
                "quotient exceeds one cplong limb (i32 max 0x{:X}) -- needs multi-limb chaining",
                i32::MAX
            ));
        }
        let sign = if raw_result < 0 { -1i128 } else { 1i128 };
        Ok(Cplong::new((sign * mag as i128) as i32, precision as u8))
    }

    /// div() using the static default meksimxm_precision.
    pub fn div(&self, other: &Cplong) -> Result<Cplong, String> {
        self.div_at_precision(other, Self::MEKSIMXM_PRECISION)
    }
}

#[allow(dead_code)]
fn mul_div_demo() {
    let a = Cplong::from_digits("5V", 1).unwrap();
    let four = Cplong::new(4, 0);
    let prod = a.mul(&four).unwrap();
    println!("{} * {} = {}  ({:.4}) (expect (17,0) == 23)", a, four, prod, prod.weight());
    assert_eq!(prod, Cplong::new(0x17, 0));

    let one = Cplong::new(1, 0);
    let three = Cplong::new(3, 0);
    let q = one.div_at_precision(&three, 2).unwrap();
    println!("{} / {} @ precision 2 = {}  ({:.6}) (expect (55,2) \u{2248} 0.332)", one, three, q, q.weight());
    assert_eq!(q, Cplong::new(0x55, 2));
}

// ---------------------------------------------------------------------
// cllong mul (single-limb-integer case; detects and reports overflow
// beyond one cplong limb rather than silently producing a wrong answer)
// ---------------------------------------------------------------------

impl Cllong {
    pub fn mul(&self, other: &Cllong) -> Result<Cllong, String> {
        let (s1, i1, f1) = self.decompose();
        let (s2, i2, f2) = other.decompose();
        let flen_total = f1.len() + f2.len();
        let scale1: i128 = 16i128.pow(f1.len() as u32);
        let scale2: i128 = 16i128.pow(f2.len() as u32);
        let mag1 = i1 as i128 * scale1 + frac_to_int(&f1);
        let mag2 = i2 as i128 * scale2 + frac_to_int(&f2);
        let raw = s1 as i128 * s2 as i128 * mag1 * mag2;

        let sign = if raw < 0 { -1 } else { 1 };
        let combined_scale: i128 = 16i128.pow(flen_total as u32);
        let mut abs_val = raw.unsigned_abs();
        let mut flen = flen_total;
        while flen > 0 && abs_val % 16 == 0 {
            abs_val /= 16;
            flen -= 1;
        }
        let _ = combined_scale;

        let int_part = abs_val / 16u128.pow(flen as u32);
        let frac_part = abs_val % 16u128.pow(flen as u32);

        if int_part > i64::MAX as u128 {
            return Err("result integer part too large even for i128 combination".to_string());
        }
        if frac_part > i32::MAX as u128 {
            return Err("result fractional part exceeds one cplong limb".to_string());
        }

        let mut limbs = split_int_into_limbs(sign as i64, int_part);
        if flen > 0 {
            limbs.push(Cplong::new(frac_part as i32, 1));
        }
        Cllong::try_new(limbs)
    }
}

#[allow(dead_code)]
fn cllong_mul_demo() {
    let a = Cllong::from_str_custom("-80000000").unwrap();
    let b = Cllong::from_str_custom("2").unwrap();
    match a.mul(&b) {
        Ok(r) => println!("{} * {} = {}", a, b, r),
        Err(e) => println!("{} * {} -> correct value is -0x100000000, but: {}", a, b, e),
    }
}

// ---------------------------------------------------------------------
// Genuine multi-limb integer chaining.
// Integer limbs: all have precision == 0. Array position (not the label)
// determines significance -- most significant first. Only flotnmbr[0]
// (the anchor) carries the sign; later integer limbs are unsigned
// magnitude, zero-padded to 4 hex digits (INT_CHUNK_DIGITS).
// Fractional limbs (precision >= 1) come after all integer limbs, as before.
// ---------------------------------------------------------------------

const INT_CHUNK_DIGITS: u32 = 4; // safe width per continuation limb (max 0xFFFF)

/// Split a signed big integer magnitude into limbs: first is natural-width
/// (anchor, carries sign), rest are 4-digit zero-padded chunks.
fn split_int_into_limbs(sign: i64, mag: u128) -> Vec<Cplong> {
    if mag == 0 {
        return vec![Cplong::new(0, 0)];
    }
    // total hex digit width of mag
    let mut w = 0u32;
    let mut t = mag;
    while t > 0 {
        w += 1;
        t /= 16;
    }
    let n_chunks = ((w + INT_CHUNK_DIGITS - 1) / INT_CHUNK_DIGITS).max(1);
    let anchor_width = w - (n_chunks - 1) * INT_CHUNK_DIGITS;

    let mut chunks = vec![];
    let mut remaining = mag;
    // peel off least-significant 4-digit chunks first, then reverse
    for _ in 1..n_chunks {
        let chunk = (remaining % 16u128.pow(INT_CHUNK_DIGITS)) as i64;
        chunks.push(chunk);
        remaining /= 16u128.pow(INT_CHUNK_DIGITS);
    }
    chunks.push(remaining as i64); // anchor (most significant, width = anchor_width)
    chunks.reverse();

    let mut out = vec![];
    for (i, &c) in chunks.iter().enumerate() {
        let v = if i == 0 { sign * c } else { c };
        out.push(Cplong::new(v as i32, 0));
    }
    let _ = anchor_width;
    out
}

/// Combine consecutive precision==0 limbs (most significant first, sign
/// only on the first) back into one signed i128 magnitude.
fn combine_int_limbs(limbs: &[Cplong]) -> (i64, i64) {
    let int_limbs: Vec<&Cplong> = limbs.iter().take_while(|l| l.precision == 0).collect();
    if int_limbs.is_empty() {
        return (1, 0);
    }
    let sign = if int_limbs[0].nmbr < 0 { -1i64 } else { 1i64 };
    let mut mag: i64 = int_limbs[0].nmbr.unsigned_abs() as i64;
    for limb in &int_limbs[1..] {
        mag = mag * 16i64.pow(INT_CHUNK_DIGITS) + limb.nmbr.unsigned_abs() as i64;
    }
    (sign, mag)
}

impl Cllong {
    /// Fallible constructor -- returns Err instead of panicking when the
    /// limb count would exceed mekimxm_array_saiz.
    pub fn try_new(flotnmbr: Vec<Cplong>) -> Result<Cllong, String> {
        if flotnmbr.len() as i8 > Self::MEKIMXM_ARRAY_SAIZ {
            return Err(format!(
                "cllong overflow: this value needs {} limbs but mekimxm_array_saiz is currently {}. \
                 Programmer: increase Cllong::MEKIMXM_ARRAY_SAIZ to at least {} and retry.",
                flotnmbr.len(),
                Self::MEKIMXM_ARRAY_SAIZ,
                flotnmbr.len()
            ));
        }
        if flotnmbr.is_empty() {
            return Err("flotnmbr must have at least the integer limb".to_string());
        }
        Ok(Cllong { flotnmbr })
    }

    pub fn from_str_custom2(s: &str) -> Result<Self, String> {
        let (int_str, frac_str) = match s.split_once('.') {
            Some((i, f)) => (i, Some(f)),
            None => (s, None),
        };
        let chars = int_str.chars();
        let negative = int_str.starts_with('-');
        let digits_only: String = if negative { chars.skip(1).collect() } else { int_str.to_string() };
        let mag = digits_only.chars().try_fold(0u128, |acc, c| {
            digit_to_value(c).map(|d| acc * 16 + d as u128)
        }).ok_or("bad digit in integer part")?;
        let sign = if negative { -1i64 } else { 1i64 };
        let mut limbs = split_int_into_limbs(sign, mag);

        if let Some(f) = frac_str {
            if !f.is_empty() {
                let frac_limb = Cplong::from_digits(f, 1).ok_or("bad digit in fractional part")?;
                limbs.push(frac_limb);
            }
        }
        Cllong::try_new(limbs)
    }
}

#[allow(dead_code)]
fn multilimb_demo() {
    let a = Cllong::from_str_custom2("-80001000").unwrap();
    println!("{} -> {:?}", a, a.flotnmbr);
    assert_eq!(a.flotnmbr, vec![Cplong::new(-0x8000, 0), Cplong::new(0x1000, 0)]);

    let b = Cllong::from_str_custom2("-80000000").unwrap();
    println!("{} -> {:?}", b, b.flotnmbr);
    assert_eq!(b.flotnmbr, vec![Cplong::new(-0x8000, 0), Cplong::new(0x0000, 0)]);

    // now the earlier overflow case: with 2-limb integer support, does
    // -0x80000000 * 2 = -0x100000000 fit? it needs 3 limbs ("1","0000","0000")
    // -- still exceeds default mekimxm_array_saiz=2, so it should now fail
    // with a precise "needs 3 limbs, max is 2" message rather than a vague one.
    let x = Cllong::from_str_custom2("-80000000").unwrap();
    let (xs, xm) = combine_int_limbs(&x.flotnmbr);
    let prod_mag = (xm as i128) * 2;
    let needed = split_int_into_limbs(-xs, prod_mag.unsigned_abs() as u128);
    println!(
        "-80000000 * 2 needs {} integer limb(s): {:?}",
        needed.len(),
        needed
    );
}

// =======================================================================
// TDD test suite. Run with `cargo test`.
// =======================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------
    // Cplong: the new "no trailing zero digit at precision > 0" invariant
    // -------------------------------------------------------------

    #[test]
    fn cplong_try_new_accepts_precision_zero_with_trailing_zero() {
        // precision 0 = plain integer; 80 (0x50) is NOT redundant with 8 --
        // it's a different number. Trailing zero digits are fine here.
        assert!(Cplong::try_new(0x50, 0).is_ok());
    }

    #[test]
    fn cplong_try_new_rejects_trailing_zero_at_nonzero_precision() {
        // (0x50, 1) means 0x50/16 = 5.0 -- same number as (0x5, 0), so
        // this representation is redundant/non-canonical.
        assert!(Cplong::try_new(0x50, 1).is_err());
    }

    #[test]
    fn cplong_try_new_accepts_nonzero_last_digit_at_nonzero_precision() {
        assert!(Cplong::try_new(0x5C, 1).is_ok()); // 5V -> ends in C, fine
    }

    #[test]
    fn cplong_try_new_always_accepts_zero_regardless_of_precision() {
        assert!(Cplong::try_new(0, 0).is_ok());
        assert!(Cplong::try_new(0, 1).is_ok());
        assert!(Cplong::try_new(0, 5).is_ok());
    }

    #[test]
    #[should_panic(expected = "cplong invariant violated")]
    fn cplong_new_panics_on_trailing_zero_violation() {
        Cplong::new(0x50, 1);
    }

    #[test]
    fn cplong_from_digits_rejects_leading_zero_padded_string() {
        // "05V" and "5V" parse to the identical nmbr (leading zero
        // contributes nothing) -- the padded form is rejected as
        // non-canonical input, not because the number itself is invalid.
        assert_eq!(Cplong::from_digits("05V", 1), None);
        assert!(Cplong::from_digits("5V", 1).is_some());
    }

    #[test]
    fn cplong_from_digits_rejects_value_level_trailing_zero_even_without_leading_zero_string() {
        // "50" has no leading zero in the string, but numerically it's
        // (0x50, 1) -- still rejected, for the value-level reason.
        assert_eq!(Cplong::from_digits("50", 1), None);
    }

    #[test]
    fn cplong_from_digits_single_zero_digit_is_fine() {
        // "0" itself is length 1, so the leading-zero string check
        // (which only fires when len > 1) doesn't apply, and nmbr==0 is
        // always exempt from the trailing-zero check.
        assert_eq!(Cplong::from_digits("0", 3), Some(Cplong::new(0, 3)));
    }

    #[test]
    fn cplong_from_digits_negative_parses_correctly() {
        let c = Cplong::from_digits("-5V5V", 0).unwrap();
        assert_eq!(c.nmbr, -0x5C5C);
        assert_eq!(c.precision, 0);
    }

    // -------------------------------------------------------------
    // Cplong: basic value semantics (weight/display), from earlier work
    // -------------------------------------------------------------

    #[test]
    fn cplong_weight_matches_worked_examples() {
        let a = Cplong::from_digits("5V", 1).unwrap();
        assert!((a.weight() - 5.75).abs() < 1e-9);
        let b = Cplong::from_digits("8", 1).unwrap();
        assert!((b.weight() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn cplong_display_matches_worked_examples() {
        let a = Cplong::from_digits("5V", 1).unwrap();
        assert_eq!(format!("{}", a), "5.V");
    }

    // -------------------------------------------------------------
    // Cplong: mul
    // -------------------------------------------------------------

    #[test]
    fn cplong_mul_worked_example() {
        // (5V,1) * (4,0) = (17,0) == 23 == 5.75 * 4
        let a = Cplong::from_digits("5V", 1).unwrap();
        let four = Cplong::new(4, 0);
        let prod = a.mul(&four).unwrap();
        assert_eq!(prod, Cplong::new(0x17, 0));
    }

    #[test]
    fn cplong_mul_trims_all_trailing_zeros_down_to_precision_zero() {
        // 256 * (1/256) = 1 exactly -- trims all the way down.
        let a = Cplong::new(0x100, 0);
        let b = Cplong::new(1, 2);
        let prod = a.mul(&b).unwrap();
        assert_eq!(prod, Cplong::new(1, 0));
    }

    #[test]
    fn cplong_mul_trims_only_as_many_zeros_as_precision_budget_allows() {
        // raw product 0x1230 at precision 3: trims exactly one trailing
        // zero (budget allows 3 decrements, but only 1 zero exists) and
        // stops at precision 2 with a nonzero last digit (0x123 ends in 3).
        let a = Cplong::new(0x1230, 0);
        let b = Cplong::new(1, 3);
        let prod = a.mul(&b).unwrap();
        assert_eq!(prod, Cplong::new(0x123, 2));
    }

    #[test]
    fn cplong_mul_result_always_satisfies_the_invariant() {
        // if this didn't hold, Cplong::new inside mul() would have panicked
        let a = Cplong::new(0x50, 0);
        let b = Cplong::new(0x30, 0);
        let prod = a.mul(&b).unwrap();
        assert!(prod.precision == 0 || prod.nmbr == 0 || prod.nmbr % 16 != 0);
    }

    // -------------------------------------------------------------
    // Cplong: div_at_precision
    // -------------------------------------------------------------

    #[test]
    fn cplong_div_worked_example() {
        // (1,0) / (3,0) @ precision 2 = (0x55, 2) ~= 0.332 (approximating 1/3)
        let one = Cplong::new(1, 0);
        let three = Cplong::new(3, 0);
        let q = one.div_at_precision(&three, 2).unwrap();
        assert_eq!(q, Cplong::new(0x55, 2));
    }

    #[test]
    fn cplong_div_trims_trailing_zero_to_stay_canonical() {
        // 2 / 1 @ precision 1: raw result is (0x20, 1) -- ends in a zero
        // digit, which would violate the invariant if not trimmed.
        // Trimmed: (2, 0), the same number in reduced form.
        let two = Cplong::new(2, 0);
        let one = Cplong::new(1, 0);
        let q = two.div_at_precision(&one, 1).unwrap();
        assert_eq!(q, Cplong::new(2, 0));
    }

    #[test]
    fn cplong_div_by_zero_errs() {
        let a = Cplong::new(1, 0);
        let zero = Cplong::new(0, 0);
        assert!(a.div_at_precision(&zero, 1).is_err());
    }

    // -------------------------------------------------------------
    // Cllong: construction, sign, display round-trips
    // -------------------------------------------------------------

    #[test]
    fn cllong_sign_lives_in_anchor_limb() {
        let c = Cllong::new(vec![
            Cplong::from_digits("-5V5V", 0).unwrap(),
            Cplong::from_digits("5V5V", 1).unwrap(),
        ]);
        assert_eq!(format!("{}", c), "-5V5V.5V5V");
        assert!((c.to_f64() - (-23644.360779)).abs() < 1e-4);
    }

    #[test]
    fn cllong_from_str_custom_round_trips() {
        let c = Cllong::from_str_custom("-5V5V.5V5V").unwrap();
        assert_eq!(format!("{}", c), "-5V5V.5V5V");
    }

    #[test]
    fn cllong_multilimb_integer_split_is_correct() {
        // -0x80001000 split into anchor (-0x8000) + one 4-digit
        // continuation (0x1000), unsigned, per the corrected scheme
        // (only the anchor carries the sign).
        let a = Cllong::from_str_custom2("-80001000").unwrap();
        assert_eq!(a.flotnmbr, vec![Cplong::new(-0x8000, 0), Cplong::new(0x1000, 0)]);
        assert_eq!(format!("{}", a), "-80001000");
    }

    #[test]
    fn cllong_add_worked_example() {
        let x = Cllong::from_str_custom("5V5V.5V5V").unwrap();
        let y = Cllong::from_str_custom("-3.8").unwrap();
        let sum = x.add(&y).unwrap();
        assert!((sum.to_f64() - 23640.860779).abs() < 1e-3);
    }

    #[test]
    fn cllong_sub_worked_example() {
        let x = Cllong::from_str_custom("5V5V.5V5V").unwrap();
        let y = Cllong::from_str_custom("-3.8").unwrap();
        let diff = x.sub(&y).unwrap();
        assert!((diff.to_f64() - 23647.860779).abs() < 1e-3);
    }

    #[test]
    fn cllong_add_positive_and_negative_to_zero() {
        let x = Cllong::from_str_custom("5").unwrap();
        let y = Cllong::from_str_custom("-5").unwrap();
        let z = x.add(&y).unwrap();
        assert!((z.to_f64() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn cllong_mul_overflow_reports_exact_limb_shortfall() {
        // -0x80000000 * 2 = -0x100000000, needs 3 integer limbs; default
        // MEKIMXM_ARRAY_SAIZ is 2, so this should fail with a message
        // naming both numbers, not silently truncate or panic.
        let p = Cllong::from_str_custom2("-80000000").unwrap();
        let two = Cllong::from_str_custom2("2").unwrap();
        let result = p.mul(&two);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("3 limbs"));
        assert!(msg.contains("MEKIMXM_ARRAY_SAIZ") || msg.contains("mekimxm_array_saiz"));
    }
}
