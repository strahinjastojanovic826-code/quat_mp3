use crate::error::QuatError;
use crate::quat::Quat;
use std::io::{Read, Write};
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct QuatBuffer {
    quats: Vec<Quat>,
}

impl QuatBuffer {
    pub fn new() -> Self {
        Self { quats: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            quats: Vec::with_capacity(capacity),
        }
    }

    /// Encodes raw bytes into a `QuatBuffer`.
    pub fn encode(bytes: &[u8]) -> Self {
        let mut quats = Vec::with_capacity(bytes.len() * 4);
        for &byte in bytes {
            quats.push(Quat::from_bits(byte >> 6));
            quats.push(Quat::from_bits(byte >> 4));
            quats.push(Quat::from_bits(byte >> 2));
            quats.push(Quat::from_bits(byte));
        }
        Self { quats }
    }

    /// Decodes quaternary values back into raw bytes.
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

    /// Reads from a `std::io::Read` stream and encodes the contents into a `QuatBuffer`.
     pub fn read_from<R: Read>(mut reader: R) -> Result<Self, QuatError> {
        let mut buffer = Self::new();
        let mut chunk = [0u8; 1024]; // Bafer od 1 KB
        
        loop {
            let bytes_read = reader.read(&mut chunk)?;
            if bytes_read == 0 {
                break;
            }
            
            for &byte in &chunk[..bytes_read] {
                buffer.push(Quat::from_bits(byte >> 6));
                buffer.push(Quat::from_bits(byte >> 4));
                buffer.push(Quat::from_bits(byte >> 2));
                buffer.push(Quat::from_bits(byte));
            }
        }
        
        Ok(buffer)
    }

    /// Writes decoded raw bytes directly to a `std::io::Write` stream.
    pub fn write_decoded_to<W: Write>(&self, mut writer: W) -> Result<(), QuatError> {
        let bytes = self.decode()?;
        writer.write_all(&bytes)?;
        Ok(())
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

    pub fn push(&mut self, quat: Quat) {
        self.quats.push(quat);
    }
}

impl Deref for QuatBuffer {
    type Target = [Quat];

    fn deref(&self) -> &Self::Target {
        &self.quats
    }
}

impl FromIterator<Quat> for QuatBuffer {
    fn from_iter<T: IntoIterator<Item = Quat>>(iter: T) -> Self {
        Self {
            quats: iter.into_iter().collect(),
        }
    }
}
