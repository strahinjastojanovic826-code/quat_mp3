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

    #[error("Invalid Xing/VBRI VBR header")]
    InvalidVbrHeader,

    #[error("Corrupted or unsupported ID3v2 tag")]
    InvalidId3v2,

    #[error("MPEG CRC verification failed")]
    CrcCheckFailed,

    #[error("Invalid UTF-8 or text encoding in metadata: {0}")]
    InvalidMetadataEncoding(String),
}

impl QuatError {
    pub fn is_recoverable_stream_error(&self) -> bool {
        matches!(
            self,
            Self::IncompleteHeader
                | Self::InvalidVbrHeader
                | Self::InvalidId3v2
                | Self::CrcCheckFailed
        )
    }
}