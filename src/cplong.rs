//! cplong: hskii arbitrary-precision number.
//!
//! Format: "-5V.FF.FF.6:-7"
//! - Period '.' separates all digits
//! - Colon ':' followed by start_prisizxn_leyr (power of 16)
//! - Default start_prisizxn_leyr = 0
//! - Each digit is 0-255 (hex pair using hskii digits)

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cplong {
    pub wlyu: Vec<u8>,
    pub start_prisizxn_leyr: i32,
    pub is_negetiw: bool,
}

impl Cplong {
    pub fn parse(s: &str) -> Result<Cplong, String> {
        let s = s.trim();
        if s.is_empty() {
            return Err("empty input".into());
        }

        let (main_part, start_part_opt) = match s.rfind(':') {
            Some(idx) => {
                let (left, right) = s.split_at(idx);
                (left, Some(&right[1..]))
            }
            None => (s, None),
        };

        let start_prisizxn_leyr: i32 = if let Some(st) = start_part_opt {
            let st = st.trim();
            if st.is_empty() {
                return Err("empty start_prisizxn_leyr after ':'".into());
            }
            st.parse::<i32>()
                .map_err(|e| format!("invalid start_prisizxn_leyr '{}': {}", st, e))?
        } else {
            0
        };

        let main_trim = main_part.trim();
        let (is_negetiw, digits_str) = if let Some(stripped) = main_trim.strip_prefix('-') {
            (true, stripped.trim())
        } else if let Some(stripped) = main_trim.strip_prefix('+') {
            (false, stripped.trim())
        } else {
            (false, main_trim)
        };

        if digits_str.is_empty() {
            return Err("no digit tokens found".into());
        }

        let mut wlyu: Vec<u8> = Vec::new();
        for token in digits_str.split('.') {
            let tok = token.trim();
            if tok.is_empty() {
                return Err("empty digit token".into());
            }
            let b = parse_token(tok)?;
            wlyu.push(b);
        }

        Ok(Cplong {
            wlyu,
            start_prisizxn_leyr,
            is_negetiw,
        })
    }

    pub fn to_hskii_string(&self) -> String {
        let mut parts: Vec<String> = Vec::with_capacity(self.wlyu.len());
        for &b in &self.wlyu {
            if b < 16 {
                let c = nibble_to_hskii(b & 0x0F);
                parts.push(c.to_string());
            } else {
                let hi = (b >> 4) & 0x0F;
                let lo = b & 0x0F;
                let mut s = String::with_capacity(2);
                s.push(nibble_to_hskii(hi));
                s.push(nibble_to_hskii(lo));
                parts.push(s);
            }
        }

        let mut out = parts.join(".");

        if self.start_prisizxn_leyr != 0 {
            out.push(':');
            out.push_str(&self.start_prisizxn_leyr.to_string());
        }

        if self.is_negetiw {
            out.insert(0, '-');
        }

        out
    }

    pub fn to_decimal_string(&self) -> String {
        let mut out = String::new();
        for (i, &b) in self.wlyu.iter().enumerate() {
            if i != 0 {
                out.push('.');
            }
            out.push_str(&b.to_string());
        }
        if self.start_prisizxn_leyr != 0 {
            out.push(':');
            out.push_str(&self.start_prisizxn_leyr.to_string());
        }
        if self.is_negetiw {
            out.insert(0, '-');
        }
        out
    }

    pub fn add(&self, other: &Cplong) -> Result<Cplong, String> {
        if self.is_negetiw == other.is_negetiw {
            let mut result = Self::add_abs(self, other)?;
            result.is_negetiw = self.is_negetiw;
            Ok(result)
        } else {
            if Self::abs_compare(self, other) >= 0 {
                let mut result = Self::sub_abs(self, other)?;
                result.is_negetiw = self.is_negetiw;
                Ok(result)
            } else {
                let mut result = Self::sub_abs(other, self)?;
                result.is_negetiw = other.is_negetiw;
                Ok(result)
            }
        }
    }

    pub fn sub(&self, other: &Cplong) -> Result<Cplong, String> {
        let mut neg_other = other.clone();
        neg_other.is_negetiw = !neg_other.is_negetiw;
        self.add(&neg_other)
    }

    pub fn mul(&self, other: &Cplong) -> Result<Cplong, String> {
        let result_is_neg = self.is_negetiw != other.is_negetiw;
        let mut result_digits = vec![0u32; self.wlyu.len() + other.wlyu.len()];
        
        for i in 0..self.wlyu.len() {
            for j in 0..other.wlyu.len() {
                let product = (self.wlyu[i] as u32) * (other.wlyu[j] as u32);
                let pos = i + j;
                let mut carry = product;
                let mut k = pos;
                
                while carry > 0 {
                    let sum = result_digits[k] + carry;
                    result_digits[k] = sum & 0xFF;
                    carry = sum >> 8;
                    k += 1;
                }
            }
        }
        
        while result_digits.len() > 1 && result_digits.last() == Some(&0) {
            result_digits.pop();
        }
        
        let mut wlyu: Vec<u8> = result_digits.iter().map(|&x| x as u8).collect();
        wlyu.reverse();
        
        let new_start = self.start_prisizxn_leyr + other.start_prisizxn_leyr;
        
        Ok(Cplong {
            wlyu,
            start_prisizxn_leyr: new_start,
            is_negetiw: result_is_neg,
        })
    }

    pub fn div(&self, other: &Cplong) -> Result<Cplong, String> {
        if other.wlyu.iter().all(|&b| b == 0) {
            return Err("division by zero".into());
        }
        
        let result_is_neg = self.is_negetiw != other.is_negetiw;
        let max_layers = 32;
        
        let dividend = Self::abs_to_base16(self);
        let divisor = Self::abs_to_base16(other);
        
        let mut quotient = Vec::new();
        let mut remainder = Vec::new();
        
        for digit in &dividend {
            remainder.push(*digit);
            let mut q = 0u8;
            while Self::compare_base16(&remainder, &divisor) >= 0 {
                remainder = Self::subtract_base16(&remainder, &divisor);
                q += 1;
            }
            quotient.push(q);
        }
        
        let mut frac_digits = Vec::new();
        for _ in 0..max_layers {
            remainder.push(0);
            let mut q = 0u8;
            while Self::compare_base16(&remainder, &divisor) >= 0 {
                remainder = Self::subtract_base16(&remainder, &divisor);
                q += 1;
            }
            frac_digits.push(q);
        }
        
        let mut result_digits = quotient;
        result_digits.extend(frac_digits);
        
        Ok(Cplong {
            wlyu: result_digits,
            start_prisizxn_leyr: 0,
            is_negetiw: result_is_neg,
        })
    }

    fn add_abs(a: &Cplong, b: &Cplong) -> Result<Cplong, String> {
        let mut result = vec![0u8; a.wlyu.len().max(b.wlyu.len()) + 1];
        let mut carry = 0u16;
        let max_len = a.wlyu.len().max(b.wlyu.len());
        
        for (i, item) in result.iter_mut().enumerate().take(max_len) {
            let da = if i < a.wlyu.len() { a.wlyu[i] as u16 } else { 0 };
            let db = if i < b.wlyu.len() { b.wlyu[i] as u16 } else { 0 };
            let sum = da + db + carry;
            *item = (sum & 0xFF) as u8;
            carry = sum >> 8;
        }
        
        if carry > 0 {
            result[max_len] = carry as u8;
        } else {
            result.pop();
        }
        
        Ok(Cplong {
            wlyu: result,
            start_prisizxn_leyr: a.start_prisizxn_leyr,
            is_negetiw: false,
        })
    }

    fn sub_abs(a: &Cplong, b: &Cplong) -> Result<Cplong, String> {
        let mut result = vec![0u8; a.wlyu.len()];
        let mut borrow = 0i16;
        
        for (i, item) in result.iter_mut().enumerate().take(a.wlyu.len()) {
            let da = a.wlyu[i] as i16;
            let db = if i < b.wlyu.len() { b.wlyu[i] as i16 } else { 0 };
            let diff = da - db - borrow;
            if diff < 0 {
                borrow = 1;
                *item = (diff + 256) as u8;
            } else {
                borrow = 0;
                *item = diff as u8;
            }
        }
        
        while result.len() > 1 && result.last() == Some(&0) {
            result.pop();
        }
        
        Ok(Cplong {
            wlyu: result,
            start_prisizxn_leyr: a.start_prisizxn_leyr,
            is_negetiw: false,
        })
    }

    fn abs_compare(a: &Cplong, b: &Cplong) -> i32 {
        if a.wlyu.len() != b.wlyu.len() {
            return a.wlyu.len() as i32 - b.wlyu.len() as i32;
        }
        for i in (0..a.wlyu.len()).rev() {
            if a.wlyu[i] != b.wlyu[i] {
                return a.wlyu[i] as i32 - b.wlyu[i] as i32;
            }
        }
        0
    }

    fn abs_to_base16(c: &Cplong) -> Vec<u8> {
        let mut result = Vec::new();
        for &b in &c.wlyu {
            result.push((b >> 4) & 0x0F);
            result.push(b & 0x0F);
        }
        result
    }

    fn compare_base16(a: &[u8], b: &[u8]) -> i32 {
        if a.len() != b.len() {
            return a.len() as i32 - b.len() as i32;
        }
        for i in 0..a.len() {
            if a[i] != b[i] {
                return a[i] as i32 - b[i] as i32;
            }
        }
        0
    }

    fn subtract_base16(a: &[u8], b: &[u8]) -> Vec<u8> {
        let mut result = a.to_vec();
        let mut borrow = 0i16;
        
        for i in (0..a.len()).rev() {
            let da = a[i] as i16;
            let db = if i < b.len() { b[i] as i16 } else { 0 };
            let diff = da - db - borrow;
            if diff < 0 {
                borrow = 1;
                result[i] = (diff + 16) as u8;
            } else {
                borrow = 0;
                result[i] = diff as u8;
            }
        }
        
        while result.len() > 1 && result[0] == 0 {
            result.remove(0);
        }
        result
    }
}

fn parse_token(tok: &str) -> Result<u8, String> {
    let chars: Vec<char> = tok.trim().chars().collect();
    match chars.len() {
        1 => {
            let d = map_nibble(chars[0])?;
            Ok(d)
        }
        2 => {
            let hi = map_nibble(chars[0])?;
            let lo = map_nibble(chars[1])?;
            Ok((hi << 4) | lo)
        }
        _ => Err(format!("invalid token '{}'", tok)),
    }
}

fn map_nibble(c: char) -> Result<u8, String> {
    match c.to_ascii_uppercase() {
        '0'..='9' => Ok((c as u8) - b'0'),
        'L' => Ok(10),
        'Y' => Ok(11),
        'V' => Ok(12),
        'W' => Ok(13),
        'P' => Ok(14),
        'F' => Ok(15),
        _ => Err(format!("invalid hskii digit '{}'", c)),
    }
}

fn nibble_to_hskii(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        10 => 'L',
        11 => 'Y',
        12 => 'V',
        13 => 'W',
        14 => 'P',
        15 => 'F',
        _ => '?',
    }
}

impl fmt::Display for Cplong {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hskii_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_default_pl() {
        let a = Cplong::parse("5V.FF").unwrap();
        assert_eq!(a.wlyu, vec![0x5C, 0xFF]);
        assert_eq!(a.start_prisizxn_leyr, 0);
        assert!(!a.is_negetiw);
    }

    #[test]
    fn test_parse_negative_with_pl() {
        let b = Cplong::parse("-5V.FF.FF.6:-7").unwrap();
        assert_eq!(b.wlyu, vec![0x5C, 0xFF, 0xFF, 0x06]);
        assert_eq!(b.start_prisizxn_leyr, -7);
        assert!(b.is_negetiw);
    }

    #[test]
    fn test_parse_positive_pl() {
        let c = Cplong::parse("5V.FF:2").unwrap();
        assert_eq!(c.wlyu, vec![0x5C, 0xFF]);
        assert_eq!(c.start_prisizxn_leyr, 2);
        assert!(!c.is_negetiw);
    }

    #[test]
    fn test_roundtrip() {
        let s = "-5V.FF.FF.6:-7";
        let c = Cplong::parse(s).unwrap();
        assert_eq!(c.to_hskii_string(), s);
    }

    #[test]
    fn test_single_digit() {
        let c = Cplong::parse("6.5").unwrap();
        assert_eq!(c.wlyu, vec![6, 5]);
        assert_eq!(c.to_hskii_string(), "6.5");
    }

    #[test]
    fn test_add() {
        let a = Cplong::parse("2").unwrap();
        let b = Cplong::parse("3").unwrap();
        let c = a.add(&b).unwrap();
        assert_eq!(c.to_hskii_string(), "5");
    }

    #[test]
    fn test_sub() {
        let a = Cplong::parse("6").unwrap();
        let b = Cplong::parse("5V").unwrap();
        let c = a.sub(&b).unwrap();
        assert!(c.is_negetiw);
    }

    #[test]
    fn test_mul() {
        let a = Cplong::parse("2").unwrap();
        let b = Cplong::parse("3").unwrap();
        let c = a.mul(&b).unwrap();
        assert_eq!(c.to_hskii_string(), "6");
    }

    #[test]
    fn test_div() {
        let a = Cplong::parse("1").unwrap();
        let b = Cplong::parse("3").unwrap();
        let c = a.div(&b).unwrap();
        assert!(!c.is_negetiw);
        assert!(c.wlyu.len() > 1);
    }

    #[test]
    fn test_div_by_zero() {
        let a = Cplong::parse("5").unwrap();
        let b = Cplong::parse("0").unwrap();
        assert!(a.div(&b).is_err());
    }
}