use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CplongError {
    InvalidCharacter(char),
    EmptyInput,
    InvalidFormat(String),
    DivisionByZero,
    Overflow,
}

impl fmt::Display for CplongError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCharacter(c) => write!(f, "Invalid character: {}", c),
            Self::EmptyInput => write!(f, "Empty input"),
            Self::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            Self::DivisionByZero => write!(f, "Division by zero"),
            Self::Overflow => write!(f, "Overflow"),
        }
    }
}

impl std::error::Error for CplongError {}
