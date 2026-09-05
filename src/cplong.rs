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
        let (is_negetiw, digits_str) = if main_trim.starts_with('-') {
            (true, main_trim[1..].trim())
        } else if main_trim.starts_with('+') {
            (false, main_trim[1..].trim())
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
                let c = nibble_to_hskii((b & 0x0F) as u8);
                parts.push(c.to_string());
            } else {
                let hi = (b >> 4) & 0x0F;
                let lo = b & 0x0F;
                let mut s = String::with_capacity(2);
                s.push(nibble_to_hskii(hi as u8));
                s.push(nibble_to_hskii(lo as u8));
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
}