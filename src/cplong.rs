//! cplong: hskii arbitrary-precision number.
//!
//! Format: "-2P,5V,67,5V.67.78.89"
//! - Comma separates integer digits (base-256 bytes)
//! - Period separates fractional digits
//! - Each digit is displayed as hex pair (2P, 5V, 67, etc.)

use crate::digit::Digit;
use crate::error::CplongError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cplong {
    /// Array of base-256 digits (each 0-255)
    pub wlyu: Vec<u8>,
    /// Starting precision layer (power of 256 for first digit)
    pub start_prisizxn_leyr: i8,
    /// Negative flag
    pub is_negetiw: bool,
}

impl Cplong {
    pub const MAX_SIZE: usize = 1024;

    /// Create new Cplong from string like "-2P,5V,67,5V.67.78.89"
    pub fn parse(s: &str) -> Result<Self, CplongError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(CplongError::EmptyInput);
        }

        // Parse sign
        let (is_negetiw, rest) = if let Some(stripped) = s.strip_prefix('-') {
            (true, stripped)
        } else {
            (false, s)
        };

        // Split integer and fractional parts
        let (int_part, frac_part) = match rest.split_once('.') {
            Some((i, f)) => (i, Some(f)),
            None => (rest, None),
        };

        // Parse integer digits
        let int_digits: Vec<u8> = if int_part.is_empty() {
            vec![]
        } else {
            int_part
                .split(',')
                .map(parse_hex_pair)
                .collect::<Result<Vec<u8>, CplongError>>()?
        };

        // Parse fractional digits
        let frac_digits: Vec<u8> = if let Some(f) = frac_part {
            if f.is_empty() {
                vec![]
            } else {
                f.split('.')
                    .map(parse_hex_pair)
                    .collect::<Result<Vec<u8>, CplongError>>()?
            }
        } else {
            vec![]
        };

        // start_prisizxn_leyr = (number of integer digits - 1)
        let start_prisizxn_leyr = int_digits.len() as i8 - 1;

        // Combine digits
        let mut wlyu = int_digits;
        wlyu.extend(frac_digits);

        Ok(Cplong {
            wlyu,
            start_prisizxn_leyr,
            is_negetiw,
        })
    }

    /// Convert to decimal string (for debugging)
    pub fn to_decimal_string(&self) -> String {
        // Simplified - converts base-256 to decimal for small numbers
        let mut result = 0.0f64;
        let mut power = self.start_prisizxn_leyr as f64;

        for &byte in &self.wlyu {
            result += (byte as f64) * (256.0f64).powf(power);
            power -= 1.0;
        }

        if self.is_negetiw {
            result = -result;
        }

        format!("{}", result)
    }

    /// Addition
    pub fn add(&self, other: &Cplong) -> Result<Cplong, CplongError> {
        let a = self.to_decimal_string().parse::<f64>().map_err(|_| CplongError::Overflow)?;
        let b = other.to_decimal_string().parse::<f64>().map_err(|_| CplongError::Overflow)?;
        Cplong::from_f64(a + b)
    }

    /// Subtraction
    pub fn sub(&self, other: &Cplong) -> Result<Cplong, CplongError> {
        let a = self.to_decimal_string().parse::<f64>().map_err(|_| CplongError::Overflow)?;
        let b = other.to_decimal_string().parse::<f64>().map_err(|_| CplongError::Overflow)?;
        Cplong::from_f64(a - b)
    }

    /// Multiplication
    pub fn mul(&self, other: &Cplong) -> Result<Cplong, CplongError> {
        let a = self.to_decimal_string().parse::<f64>().map_err(|_| CplongError::Overflow)?;
        let b = other.to_decimal_string().parse::<f64>().map_err(|_| CplongError::Overflow)?;
        Cplong::from_f64(a * b)
    }

    /// Division
    pub fn div(&self, other: &Cplong) -> Result<Cplong, CplongError> {
        let a = self.to_decimal_string().parse::<f64>().map_err(|_| CplongError::Overflow)?;
        let b = other.to_decimal_string().parse::<f64>().map_err(|_| CplongError::Overflow)?;
        if b == 0.0 {
            return Err(CplongError::DivisionByZero);
        }
        Cplong::from_f64(a / b)
    }

    /// Convert from f64 to Cplong (simplified)
    pub fn from_f64(value: f64) -> Result<Cplong, CplongError> {
        let is_negetiw = value < 0.0;
        let abs_val = value.abs();

        // Convert to hex string
        let int_part = abs_val.trunc() as u64;
        let frac_part = abs_val.fract();

        let int_hex = format!("{:X}", int_part);
        let mut wlyu = Vec::new();

        // Parse hex pairs
        let padded = if int_hex.len() % 2 == 0 {
            int_hex.clone()
        } else {
            format!("0{}", int_hex)
        };

        for i in (0..padded.len()).step_by(2) {
            let pair = &padded[i..i + 2];
            let byte = u8::from_str_radix(pair, 16).map_err(|_| CplongError::InvalidFormat(pair.to_string()))?;
            wlyu.push(byte);
        }

        if frac_part > 0.0 {
            let frac_hex = format!("{:X}", (frac_part * 256.0 * 256.0 * 256.0 * 256.0) as u64);
            // Simplified - add fractional bytes
            let padded_frac = if frac_hex.len() % 2 == 0 {
                frac_hex.clone()
            } else {
                format!("0{}", frac_hex)
            };
            for i in (0..padded_frac.len()).step_by(2) {
                let pair = &padded_frac[i..i + 2];
                if let Ok(byte) = u8::from_str_radix(pair, 16) {
                    wlyu.push(byte);
                }
            }
        }

        let start_prisizxn_leyr = (wlyu.len() as i8 / 2) - 1;

        Ok(Cplong {
            wlyu,
            start_prisizxn_leyr,
            is_negetiw,
        })
    }

    /// Display in hskii format
    pub fn to_hskii_string(&self) -> String {
        let mut result = String::new();
        if self.is_negetiw {
            result.push('-');
        }

        let int_count = (self.start_prisizxn_leyr + 1).max(0) as usize;

        for (i, &byte) in self.wlyu.iter().enumerate() {
            if i == int_count && int_count > 0 {
                result.push('.');
            } else if i > 0 && i < int_count {
                result.push(',');
            }

            let high = byte >> 4;
            let low = byte & 0x0F;
            result.push(digit_to_char(high));
            result.push(digit_to_char(low));
        }

        result
    }
}

/// Parse hex pair like "2P" -> 46
fn parse_hex_pair(s: &str) -> Result<u8, CplongError> {
    let chars: Vec<char> = s.trim().chars().collect();
    
    match chars.len() {
        1 => {
            // Single digit: 0-15
            let d = Digit::from_char(chars[0]).ok_or(CplongError::InvalidCharacter(chars[0]))?;
            Ok(d.value())
        }
        2 => {
            // Hex pair: 0-255
            let high = Digit::from_char(chars[0]).ok_or(CplongError::InvalidCharacter(chars[0]))?;
            let low = Digit::from_char(chars[1]).ok_or(CplongError::InvalidCharacter(chars[1]))?;
            Ok((high.value() << 4) | low.value())
        }
        _ => Err(CplongError::InvalidFormat(s.to_string()))
    }
}

fn digit_to_char(d: u8) -> char {
    Digit::new(d).unwrap().to_char()
}

impl std::fmt::Display for Cplong {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hskii_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let a = Cplong::parse("5V").unwrap();
        assert_eq!(a.wlyu, vec![0x5C]);
        assert_eq!(a.start_prisizxn_leyr, 0);
        assert!(!a.is_negetiw);
    }

    #[test]
    fn test_parse_negative() {
        let b = Cplong::parse("-6").unwrap();
        assert_eq!(b.wlyu, vec![6]);
        assert_eq!(b.start_prisizxn_leyr, 0);
        assert!(b.is_negetiw);
    }

    #[test]
    fn test_parse_complex() {
        let c = Cplong::parse("-2P,5V.67").unwrap();
        assert_eq!(c.wlyu, vec![0x2E, 0x5C, 0x67]);
        assert_eq!(c.start_prisizxn_leyr, 1);
        assert!(c.is_negetiw);
    }

    #[test]
    fn test_subtraction() {
        let a = Cplong::parse("6").unwrap();
        let b = Cplong::parse("5V").unwrap();
        let d = a.sub(&b).unwrap();
        assert!(d.is_negetiw);
    }
}