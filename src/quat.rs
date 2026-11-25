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

impl From<Quat> for u8 {
    #[inline]
    fn from(quat: Quat) -> Self {
        quat.value()
    }
}

impl fmt::Display for Quat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl From<u8> for Quat {
    #[inline]
    fn from(bits: u8) -> Self {
        Self::from_bits(bits)
    }
}

impl AsRef<Quat> for Quat {
    #[inline]
    fn as_ref(&self) -> &Quat {
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QuatVec(pub Vec<Quat>);

impl FromIterator<u8> for QuatVec {
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        Self(iter.into_iter().map(Quat::from_bits).collect())
    }
}