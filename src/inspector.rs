use crate::buffer::QuatBuffer;
use crate::quat::Quat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mp3HeaderKind {
    Id3v2,
    MpegFrameSync,
    Unknown,
}

pub struct Mp3Inspector;

impl Mp3Inspector {
    /// Evaluates headers directly from the quaternary representation.
    pub fn detect_header(buffer: &QuatBuffer) -> Mp3HeaderKind {
        let quats = buffer.as_quats();
        if quats.len() < 12 {
            return Mp3HeaderKind::Unknown;
        }

        // Quat sequence for ASCII "ID3": [1, 0, 2, 1], [1, 0, 1, 0], [0, 3, 0, 3]
        const ID3_PATTERN: [Quat; 12] = [
            Quat::Q1, Quat::Q0, Quat::Q2, Quat::Q1,
            Quat::Q1, Quat::Q0, Quat::Q1, Quat::Q0,
            Quat::Q0, Quat::Q3, Quat::Q0, Quat::Q3,
        ];

        if quats.starts_with(&ID3_PATTERN) {
            return Mp3HeaderKind::Id3v2;
        }

        // MPEG Frame Sync (11 bits set to 1)
        if quats[0] == Quat::Q3 && quats[1] == Quat::Q3 && quats[2] == Quat::Q3 && quats[3] == Quat::Q3 {
            if quats[4] == Quat::Q3 && quats[5].value() >= 2 {
                return Mp3HeaderKind::MpegFrameSync;
            }
        }

        Mp3HeaderKind::Unknown
    }
}