use thiserror::Error;

#[derive(Debug, Error)]
pub enum QuatError {
    #[error("Buffer length ({length}) is not a multiple of 4")]
    InvalidBufferLength { length: usize },

    #[error("Value {0} is outside valid quaternary range [0..3]")]
    InvalidDigitValue(u8),

    #[error("Buffer is too short to contain a valid MP3 header")]
    IncompleteHeader,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}