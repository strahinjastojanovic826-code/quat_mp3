use crate::buffer::QuatBuffer;
use crate::quat::Quat;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mp3HeaderKind {
    Id3v2 { size_bytes: usize },
    MpegFrame { header: MpegHeader },
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MpegVersion {
    Mpeg1,
    Mpeg2,
    Mpeg2_5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MpegLayer {
    Layer1,
    Layer2,
    Layer3,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MpegHeader {
    pub version: MpegVersion,
    pub layer: MpegLayer,
    pub bitrate_kbps: u16,
    pub sample_rate_hz: u32,
    pub padding: bool,
}

pub struct Mp3Inspector;

impl Mp3Inspector {
    // Quat representation for ASCII "ID3" pattern
    const ID3_PATTERN: [Quat; 12] = [
        Quat::Q1, Quat::Q0, Quat::Q2, Quat::Q1, // 'I'
        Quat::Q1, Quat::Q0, Quat::Q1, Quat::Q0, // 'D'
        Quat::Q0, Quat::Q3, Quat::Q0, Quat::Q3, // '3'
    ];

    /// Inspects the buffer's start and returns the detected header structure.
    pub fn detect_header(buffer: &QuatBuffer) -> Mp3HeaderKind {
        let quats = buffer.as_quats();
        if quats.len() < 12 {
            return Mp3HeaderKind::Unknown;
        }

        // 1. Check for ID3v2 Tag Header
        if quats.starts_with(&Self::ID3_PATTERN) {
            let size = if quats.len() >= 40 {
                Self::parse_id3_size(&quats[24..40])
            } else {
                0
            };
            return Mp3HeaderKind::Id3v2 { size_bytes: size };
        }

        // 2. Check for MPEG Frame Header (requires at least 4 bytes = 16 quats)
        if quats.len() >= 16 {
            if let Some(header) = Self::parse_mpeg_header(&quats[0..16]) {
                return Mp3HeaderKind::MpegFrame { header };
            }
        }

        Mp3HeaderKind::Unknown
    }

    fn parse_mpeg_header(quats: &[Quat]) -> Option<MpegHeader> {
        let mut b = [0u8; 4];
        for i in 0..4 {
            b[i] = (quats[i * 4].value() << 6)
                | (quats[i * 4 + 1].value() << 4)
                | (quats[i * 4 + 2].value() << 2)
                | quats[i * 4 + 3].value();
        }

        if b[0] != 0xFF || (b[1] & 0xE0) != 0xE0 {
            return None;
        }

        let version = match (b[1] >> 3) & 0x03 {
            0 => MpegVersion::Mpeg2_5,
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

        let bitrate_idx = ((b[2] >> 4) & 0x0F) as usize;
        let sample_idx = ((b[2] >> 2) & 0x03) as usize;

        if bitrate_idx == 0 || bitrate_idx == 15 || sample_idx == 3 {
            return None; // Nevažeći bitovi u zaglavlju
        }

        // Standardna tabela protoka za MPEG-1 Layer 3 (u kbps)
        const BITRATES_MPEG1_L3: [u16; 15] = [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320];
        // Standardna tabela frekvencija odabiranja za MPEG-1 (u Hz)
        const SAMPLE_RATES_MPEG1: [u32; 3] = [44100, 48000, 32000];

        let bitrate_kbps = BITRATES_MPEG1_L3[bitrate_idx];
        let sample_rate_hz = SAMPLE_RATES_MPEG1[sample_idx];
        let padding = ((b[2] >> 1) & 0x01) == 1;

        Some(MpegHeader {
            version,
            layer,
            bitrate_kbps,
            sample_rate_hz,
            padding,
        })
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