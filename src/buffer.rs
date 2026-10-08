use crate::error::QuatError;
use crate::quat::Quat;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuatBuffer {
    quats: Vec<Quat>,
}

impl QuatBuffer {
    pub fn encode(bytes: &[u8]) -> Self {
        let mut quats = Vec::with_capacity(bytes.len() * 4);
        for &byte in bytes {
            quats.push(Quat::from_bits((byte >> 6) & 0b11));
            quats.push(Quat::from_bits((byte >> 4) & 0b11));
            quats.push(Quat::from_bits((byte >> 2) & 0b11));
            quats.push(Quat::from_bits(byte & 0b11));
        }
        Self { quats }
    }

    pub fn decode(&self) -> Result<Vec<u8>, QuatError> {
        if self.quats.len() % 4 != 0 {
            return Err(QuatError::InvalidBufferLength {
                length: self.quats.len(),
            });
        }

        let mut bytes = Vec::with_capacity(self.quats.len() / 4);
        for chunk in self.quats.chunks_exact(4) {
            let byte = (chunk[0].value() << 6)
                | (chunk[1].value() << 4)
                | (chunk[2].value() << 2)
                | chunk[3].value();
            bytes.push(byte);
        }
        Ok(bytes)
    }

    pub fn as_quats(&self) -> &[Quat] {
        &self.quats
    }

    pub fn len(&self) -> usize {
        self.quats.len()
    }

    pub fn is_empty(&self) -> bool {
        self.quats.is_empty()
    }
}