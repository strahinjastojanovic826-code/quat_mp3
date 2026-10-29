use crate::buffer::QuatBuffer;
use crate::quat::Quat;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mp3HeaderKind {
    Id3v2 { size_bytes: usize },
    MpegFrame { header: MpegHeader, offset: usize },
    Id3v1 { title: String, artist: String, album: String, year: u16 },
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MpegVersion {
    Mpeg25,
    Mpeg2,
    Mpeg1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MpegLayer {
    Layer3,
    Layer2,
    Layer1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MpegHeader {
    pub version: MpegVersion,
    pub layer: MpegLayer,
    pub bitrate_kbps: u16,
    pub sample_rate_hz: u32,
    pub padding: bool,
}

pub struct Mp3Inspector;

impl Mp3Inspector {
    pub const ID3_PATTERN: [Quat; 12] = [
        Quat::Q1, Quat::Q2, Quat::Q0, Quat::Q1, // 'I'
        Quat::Q1, Quat::Q2, Quat::Q0, Quat::Q0, // 'D'
        Quat::Q0, Quat::Q3, Quat::Q0, Quat::Q3, // '3'
    ];

    /// Inspects the buffer, handles ID3v2 tag skipping, and searches for a valid MPEG audio frame.
    pub fn detect_header(buffer: &QuatBuffer) -> Mp3HeaderKind {
    // Pretvaramo zbijeni bafer u niz kvartova da bi ostatak inspektora radio bez izmena
    let quats: Vec<Quat> = (0..buffer.len()).filter_map(|i| buffer.get(i)).collect();

    if let Some(id3v1) = Self::detect_id3v1(&quats) {
        return id3v1;
    }

    // 1. Check for ID3v2 Tag Header at the start
    if quats.starts_with(&Self::ID3_PATTERN) {
        if quats.len() >= 40 {
            let id3_size = Self::parse_id3_size(&quats[24..40]);
            // Total ID3v2 tag size = 10-byte header + id3_size payload. In quats: * 4.
            let total_id3_quats = (10 + id3_size) * 4;

            // Search for the MPEG frame after skipping the ID3v2 tag
            if let Some((header, offset)) = Self::find_sync_frame(&quats, total_id3_quats) {
                return Mp3HeaderKind::MpegFrame { header, offset };
            }
        }
    }

    // 2. Search for MPEG Frame Header from the beginning of the buffer
    if let Some((header, offset)) = Self::find_sync_frame(&quats, 0) {
        return Mp3HeaderKind::MpegFrame { header, offset };
    }

    Mp3HeaderKind::Unknown
}

    pub fn detect_id3v1(quats: &[Quat]) -> Option<Mp3HeaderKind> {
        if quats.len() >= 512 {
            let start_idx = quats.len() - 512;
            let mut decoded_bytes = Vec::with_capacity(128);
            let mut valid = true;
            for chunk_idx in 0..128 {
                let base = start_idx + (chunk_idx * 4);
                if let (Some(q0), Some(q1), Some(q2), Some(q3)) = (
                    quats.get(base), quats.get(base + 1), quats.get(base + 2), quats.get(base + 3)
                ) {
                    let byte = (q0.value() << 6) | (q1.value() << 4) | (q2.value() << 2) | q3.value();
                    decoded_bytes.push(byte);
                } else {
                    valid = false;
                    break;
                }
            }
            if valid && decoded_bytes.len() >= 128 && &decoded_bytes[0..3] == b"TAG" {
                let parse_string = |bytes: &[u8]| -> String {
                    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
                    String::from_utf8_lossy(&bytes[0..len]).trim().to_string()
                };
                let title = parse_string(&decoded_bytes[3..33]);
                let artist = parse_string(&decoded_bytes[33..63]);
                let album = parse_string(&decoded_bytes[63..93]);
                let year = parse_string(&decoded_bytes[93..97]).parse().unwrap_or(0);
                return Some(Mp3HeaderKind::Id3v1 { title, artist, album, year });
            }
        }
        None
    }

    /// Scans through quats starting from `start_offset` to find a valid MPEG sync word.
    pub fn find_sync_frame(quats: &[Quat], start_offset: usize) -> Option<(MpegHeader, usize)> {
        if quats.len() < start_offset + 16 {
            return None;
        }

        let mut i = start_offset;
        // Advance byte by byte (4 quats per byte) until a valid header is parsed
        while i + 16 <= quats.len() {
            if let Some(header) = Self::parse_mpeg_header(&quats[i..i + 16]) {
                return Some((header, i));
            }
            i += 4;
        }
        None
    }

    fn parse_mpeg_header(quats: &[Quat]) -> Option<MpegHeader> {
        let mut b = [0u8; 4];
        for j in 0..4 {
            b[j] = (quats[j * 4].value() << 6)
                | (quats[j * 4 + 1].value() << 4)
                | (quats[j * 4 + 2].value() << 2)
                | quats[j * 4 + 3].value();
        }

        // Check frame sync bits (11 bits set to 1: 0xFF followed by top 3 bits of second byte)
        if b[0] != 0xFF || (b[1] & 0xE0) != 0xE0 {
            return None;
        }

        let version = match (b[1] >> 3) & 0x03 {
            0 => MpegVersion::Mpeg25,
            2 => MpegVersion::Mpeg2,
            3 => MpegVersion::Mpeg1,
            _ => return None,
        };

        let layer = match (b[1] >> 1) & 0x03 {
            1 => MpegLayer::Layer3,
            2 => MpegLayer::Layer2,
            3 => MpegLayer::Layer1,
            _ => return None,
        };

        let bitrate_idx = (b[2] >> 4) & 0x0F;
        let sample_idx = (b[2] >> 2) & 0x03;

        if bitrate_idx == 0 || bitrate_idx == 15 || sample_idx == 3 {
            return None;
        }

        let bitrate_kbps = Self::get_bitrate(version, layer, bitrate_idx)?;
        let sample_rate_hz = Self::get_sample_rate(version, sample_idx)?;
        let padding = ((b[2] >> 1) & 0x01) == 1;

        Some(MpegHeader {
            version,
            layer,
            bitrate_kbps,
            sample_rate_hz,
            padding,
        })
    }

    fn get_bitrate(version: MpegVersion, layer: MpegLayer, idx: u8) -> Option<u16> {
        match (version, layer) {
            (MpegVersion::Mpeg1, MpegLayer::Layer1) => {
                const T: [u16; 15] = [0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448];
                T.get(idx as usize).copied()
            }
            (MpegVersion::Mpeg1, MpegLayer::Layer2) => {
                const T: [u16; 15] = [0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384];
                T.get(idx as usize).copied()
            }
            (MpegVersion::Mpeg1, MpegLayer::Layer3) => {
                const T: [u16; 15] = [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320];
                T.get(idx as usize).copied()
            }
            (MpegVersion::Mpeg2 | MpegVersion::Mpeg25, MpegLayer::Layer1) => {
                const T: [u16; 15] = [0, 32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256];
                T.get(idx as usize).copied()
            }
            (MpegVersion::Mpeg2 | MpegVersion::Mpeg25, MpegLayer::Layer2 | MpegLayer::Layer3) => {
                const T: [u16; 15] = [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160];
                T.get(idx as usize).copied()
            }
        }
    }

    fn get_sample_rate(version: MpegVersion, idx: u8) -> Option<u32> {
        match version {
            MpegVersion::Mpeg1 => match idx {
                0 => Some(44100),
                1 => Some(48000),
                2 => Some(32000),
                _ => None,
            },
            MpegVersion::Mpeg2 => match idx {
                0 => Some(22050),
                1 => Some(24000),
                2 => Some(16000),
                _ => None,
            },
            MpegVersion::Mpeg25 => match idx {
                0 => Some(11025),
                1 => Some(12000),
                2 => Some(8000),
                _ => None,
            },
        }
    }

    fn parse_id3_size(quats: &[Quat]) -> usize {
        let mut bytes = [0u8; 4];
        for i in 0..4 {
            bytes[i] = (quats[i * 4].value() << 6)
                | (quats[i * 4 + 1].value() << 4)
                | (quats[i * 4 + 2].value() << 2)
                | quats[i * 4 + 3].value();
        }
        ((bytes[0] as usize & 0x7F) << 21)
            | ((bytes[1] as usize & 0x7F) << 14)
            | ((bytes[2] as usize & 0x7F) << 7)
            | (bytes[3] as usize & 0x7F)
    }
}