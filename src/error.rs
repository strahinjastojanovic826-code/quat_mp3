use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuatError {
    InvalidBufferLength { length: usize },
    InvalidDigitValue(u8),
    IncompleteHeader,
}

impl fmt::Display for QuatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBufferLength { length } => {
                write!(f, "Buffer length ({length}) is not a multiple of 4")
            }
            Self::InvalidDigitValue(val) => {
                write!(f, "Value {val} is outside valid quaternary range [0..3]")
            }
            Self::IncompleteHeader => {
                write!(f, "Buffer is too short to contain a valid MP3 header")
            }
        }
    }
}

impl std::error::Error for QuatError {}