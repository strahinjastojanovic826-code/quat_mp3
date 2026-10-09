use crate::error::QuatError;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Quat {
    Q0 = 0,
    Q1 = 1,
    Q2 = 2,
    Q3 = 3,
}

impl Quat {
    /// Constructs a `Quat` value using the lowest 2 bits.
    #[inline]
    pub const fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            0 => Self::Q0,
            1 => Self::Q1,
            2 => Self::Q2,
            _ => Self::Q3,
        }
    }

    #[inline]
    pub const fn value(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for Quat {
    type Error = QuatError;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Q0),
            1 => Ok(Self::Q1),
            2 => Ok(Self::Q2),
            3 => Ok(Self::Q3),
            val => Err(QuatError::InvalidDigitValue(val)),
        }
    }
}

impl From<Quat> for u8 {
    #[inline]
    fn from(q: Quat) -> Self {
        q.value()
    }
}

impl fmt::Display for Quat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}