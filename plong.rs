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

    pub fn new(nmbr: i32, precision: u8) -> Self {
        Cplong { nmbr, precision }
    }

    pub fn from_digits(digits: &str, precision: u8) -> Option<Self> {
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
        Some(Cplong::new(value as i32, precision))
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
        let result = scaled_n1 / other.nmbr as i128; // truncates toward zero
        if result.unsigned_abs() > i32::MAX as u128 {
            return Err(format!(
                "quotient exceeds one cplong limb (i32 max 0x{:X}) -- needs multi-limb chaining",
                i32::MAX
            ));
        }
        Ok(Cplong::new(result as i32, m))
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
