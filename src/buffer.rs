use crate::error::QuatError;
use crate::quat::Quat;
use std::io::{Read, Write};

#[derive(Debug, Clone, Default)]
pub struct QuatBuffer {
    data: Vec<u8>,   // Ovde čuvamo zbijene bajtove (4 kvarta po bajtu)
    len: usize,      // Ukupan broj pojedinačnih kvartova
}

impl QuatBuffer {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            // Pošto 4 kvarta staju u 1 bajt, delimo kapacitet sa 4
            data: Vec::with_capacity((capacity + 3) / 4),
            len: 0,
        }
    }

    /// Dodaje kvart i automatski ga pakuje na odgovarajuća 2 bita u bajtu
    pub fn push(&mut self, quat: Quat) {
        let val = quat.value();
        let position = self.len % 4;       // Pozicija u trenutnom bajtu (0, 1, 2 ili 3)
        let shift = (3 - position) * 2;    // Izračunavanje bitovskog pomeraja (6, 4, 2, 0)

        if position == 0 {
            self.data.push(val << shift);
        } else {
            let last_idx = self.data.len() - 1;
            self.data[last_idx] |= val << shift;
        }

        self.len += 1;
    }

    /// Čita kvart sa zadatog indeksa bez dodatne alokacije memorije
    pub fn get(&self, index: usize) -> Option<Quat> {
        if index >= self.len {
            return None;
        }

        let byte_idx = index / 4;
        let position = index % 4;
        let shift = (3 - position) * 2;

        let byte = self.data[byte_idx];
        let val = (byte >> shift) & 0b11;

        Some(Quat::from_bits(val))
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Vraća sirove spakovane bajtove spremne za čuvanje ili slanje
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Enkodira obične sirove bajtove u zbijeni kvartarni format (4x manja memorija za čuvanje)
    pub fn encode(bytes: &[u8]) -> Self {
        let mut buffer = Self::with_capacity(bytes.len() * 4);
        for &byte in bytes {
            buffer.push(Quat::from_bits(byte >> 6));
            buffer.push(Quat::from_bits(byte >> 4));
            buffer.push(Quat::from_bits(byte >> 2));
            buffer.push(Quat::from_bits(byte >> 0));
        }
        buffer
    }

    pub fn encode_bulk(bytes: &[u8]) -> Self {
        let mut data = Vec::with_capacity((bytes.len() + 3) / 4);
        let mut total_quats = 0;

        for chunk in bytes.chunks(4) {
            let mut packed_byte: u8 = 0;
            for (i, &b) in chunk.iter().enumerate() {
                let quat_val = (b >> 6) & 0b11;
                packed_byte |= quat_val << ((3 - i) * 2);
            }
            data.push(packed_byte);
            total_quats += chunk.len() * 4;
        }

        Self {
            data,
            len: total_quats,
        }
    }

    /// Dekodira zbijene kvartove nazad u originalne bajtove
    pub fn decode(&self) -> Result<Vec<u8>, QuatError> {
        if self.len % 4 != 0 {
            return Err(QuatError::InvalidBufferLength { length: self.len });
        }

        let mut bytes = Vec::with_capacity(self.len / 4);
        for chunk_idx in 0..(self.len / 4) {
            let base_idx = chunk_idx * 4;
            let q0 = self.get(base_idx).unwrap().value();
            let q1 = self.get(base_idx + 1).unwrap().value();
            let q2 = self.get(base_idx + 2).unwrap().value();
            let q3 = self.get(base_idx + 3).unwrap().value();

            let byte = (q0 << 6) | (q1 << 4) | (q2 << 2) | q3;
            bytes.push(byte);
        }
        Ok(bytes)
    }
    pub fn write_decoded_to<W: Write>(&self, mut writer: W) -> Result<(), QuatError> {
        let bytes = self.decode()?;
        writer.write_all(&bytes)?;
        Ok(())
    }

}

/// Omogućava kreiranje PackedQuatBuffer-a direktno iz iteratora kvartova
impl FromIterator<Quat> for QuatBuffer {
    fn from_iter<T: IntoIterator<Item = Quat>>(iter: T) -> Self {
        let mut buffer = Self::new();
        for quat in iter {
            buffer.push(quat);
        }
        buffer
    }
}

pub struct QuatStreamReader<R: Read> {
    inner: R,
}

impl<R: Read> QuatStreamReader<R> {
    pub fn new(inner: R) -> Self {
        Self { inner }
    }

    /// Reads a chunk of raw bytes from the stream and encodes them into a PackedQuatBuffer on-the-fly
    pub fn read_chunk(&mut self, chunk_size: usize) -> std::io::Result<Option<QuatBuffer>> {
        let mut raw_buf = vec![0u8; chunk_size];
        let bytes_read = self.inner.read(&mut raw_buf)?;
        
        if bytes_read == 0 {
            return Ok(None);
        }

        raw_buf.truncate(bytes_read);
        Ok(Some(QuatBuffer::encode(&raw_buf)))
    }
}

pub struct QuatStreamWriter<W: Write> {
    inner: W,
}

impl<W: Write> QuatStreamWriter<W> {
    pub fn new(inner: W) -> Self {
        Self { inner }
    }

    /// Decodes a PackedQuatBuffer and writes the raw bytes directly to the underlying writer stream
    pub fn write_buffer(&mut self, buffer: &QuatBuffer) -> Result<(), QuatError> {
        let decoded_bytes = buffer.decode()?;
        self.inner.write_all(&decoded_bytes)?;
        Ok(())
    }
}