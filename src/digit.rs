//! Hex digit representation for hskii number system.
//!
//! 16 digits: 0-9, L(10), Y(11), V(12), W(13), P(14), F(15)

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Digit(pub u8);

impl Digit {
    pub const MAX: u8 = 15;

    pub fn new(value: u8) -> Result<Self, String> {
        if value <= Self::MAX {
            Ok(Digit(value))
        } else {
            Err(format!("Invalid digit: {} (max {})", value, Self::MAX))
        }
    }

    pub fn value(&self) -> u8 {
        self.0
    }

    pub fn to_char(&self) -> char {
        match self.0 {
            0..=9 => (b'0' + self.0) as char,
            10 => 'L',
            11 => 'Y',
            12 => 'V',
            13 => 'W',
            14 => 'P',
            15 => 'F',
            _ => '?',
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_uppercase() {
            '0'..='9' => Some(Digit(c as u8 - b'0')),
            'L' => Some(Digit(10)),
            'Y' => Some(Digit(11)),
            'V' => Some(Digit(12)),
            'W' => Some(Digit(13)),
            'P' => Some(Digit(14)),
            'F' => Some(Digit(15)),
            _ => None,
        }
    }
}

impl std::fmt::Display for Digit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}
