use std::fmt;

#[derive(Debug)]
pub enum QuatError {
    InvalidBufferLength { length: usize },
    InvalidDigitValue(u8),
    IncompleteHeader,
    Io(std::io::Error),
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
            Self::Io(err) => write!(f, "I/O error: {err}"),
        }
    }
}

impl std::error::Error for QuatError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for QuatError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}