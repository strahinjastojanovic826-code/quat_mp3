pub mod buffer;
pub mod error;
pub mod inspector;
pub mod quat;

pub use buffer::QuatBuffer;
pub use error::QuatError;
pub use inspector::{Mp3HeaderKind, Mp3Inspector};
pub use quat::Quat;

#[cfg(test)]
mod tests {
    use super::*;

    // ==========================================
    // Quat Type Tests
    // ==========================================

    #[test]
    fn test_quat_from_bits() {
        assert_eq!(Quat::from_bits(0b00), Quat::Q0);
        assert_eq!(Quat::from_bits(0b01), Quat::Q1);
        assert_eq!(Quat::from_bits(0b10), Quat::Q2);
        assert_eq!(Quat::from_bits(0b11), Quat::Q3);

        // Masks out higher bits automatically
        assert_eq!(Quat::from_bits(0b1111_1101), Quat::Q1);
    }

    #[test]
    fn test_quat_try_from() {
        assert_eq!(Quat::try_from(0), Ok(Quat::Q0));
        assert_eq!(Quat::try_from(1), Ok(Quat::Q1));
        assert_eq!(Quat::try_from(2), Ok(Quat::Q2));
        assert_eq!(Quat::try_from(3), Ok(Quat::Q3));

        assert_eq!(Quat::try_from(4), Err(QuatError::InvalidDigitValue(4)));
        assert_eq!(Quat::try_from(255), Err(QuatError::InvalidDigitValue(255)));
    }

    #[test]
    fn test_quat_values_and_display() {
        assert_eq!(Quat::Q0.value(), 0);
        assert_eq!(Quat::Q1.value(), 1);
        assert_eq!(Quat::Q2.value(), 2);
        assert_eq!(Quat::Q3.value(), 3);

        assert_eq!(format!("{}", Quat::Q2), "2");
    }

    // ==========================================
    // QuatBuffer Tests
    // ==========================================

    #[test]
    fn test_buffer_encode_decode_roundtrip() {
        let original_data = vec![0x00, 0xFF, 0x49, 0x44, 0x33, 0xAA, 0x55];
        let buffer = QuatBuffer::encode(&original_data);

        // Each byte produces 4 quats
        assert_eq!(buffer.len(), original_data.len() * 4);

        let decoded = buffer.decode().expect("Decoding valid buffer should succeed");
        assert_eq!(decoded, original_data);
    }

    #[test]
    fn test_empty_buffer() {
        let buffer = QuatBuffer::encode(&[]);
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);

        let decoded = buffer.decode().unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_invalid_buffer_length_decode() {
        let raw_data = vec![0x12, 0x34];
        let mut buffer = QuatBuffer::encode(&raw_data);

        // Manually corrupting quat length to simulate unaligned buffer
        // Note: as_quats returns slice, so we test via custom scenario if buffer is unaligned
        let quats = buffer.as_quats();
        let unaligned_quats = quats[..quats.len() - 1].to_vec();

        // Constructing unaligned QuatBuffer directly or testing length error
        let unaligned_buffer = unsafe {
            // Safe in test context for structural testing
            std::mem::transmute::<Vec<Quat>, QuatBuffer>(unaligned_quats)
        };

        match unaligned_buffer.decode() {
            Err(QuatError::InvalidBufferLength { length }) => {
                assert_eq!(length, 7);
            }
            _ => panic!("Expected InvalidBufferLength error"),
        }
    }

    // ==========================================
    // Mp3Inspector Tests
    // ==========================================

    #[test]
    fn test_detect_id3v2_header() {
        // ASCII "ID3" (0x49, 0x44, 0x33) + v2.3 version + flags
        let id3_bytes = vec![0x49, 0x44, 0x33, 0x03, 0x00];
        let buffer = QuatBuffer::encode(&id3_bytes);

        assert_eq!(Mp3Inspector::detect_header(&buffer), Mp3HeaderKind::Id3v2);
    }

    #[test]
    fn test_detect_mpeg_frame_sync() {
        // Frame Sync: 11 bits set to 1 -> 0xFF, 0xFB (MPEG-1 Layer 3, no CRC)
        let sync_bytes = vec![0xFF, 0xFB, 0x90, 0x64];
        let buffer = QuatBuffer::encode(&sync_bytes);

        assert_eq!(
            Mp3Inspector::detect_header(&buffer),
            Mp3HeaderKind::MpegFrameSync
        );
    }

    #[test]
    fn test_detect_unknown_header() {
        let random_bytes = vec![0x12, 0x34, 0x56, 0x78, 0x90];
        let buffer = QuatBuffer::encode(&random_bytes);

        assert_eq!(
            Mp3Inspector::detect_header(&buffer),
            Mp3HeaderKind::Unknown
        );
    }

    #[test]
    fn test_detect_too_short_buffer() {
        let short_bytes = vec![0x49, 0x44]; // Less than 12 quats (3 bytes)
        let buffer = QuatBuffer::encode(&short_bytes);

        assert_eq!(
            Mp3Inspector::detect_header(&buffer),
            Mp3HeaderKind::Unknown
        );
    }

    // ==========================================
    // Error Formatting Tests
    // ==========================================

    #[test]
    fn test_error_display() {
        let err1 = QuatError::InvalidBufferLength { length: 5 };
        assert_eq!(format!("{err1}"), "Buffer length (5) is not a multiple of 4");

        let err2 = QuatError::InvalidDigitValue(9);
        assert_eq!(format!("{err2}"), "Value 9 is outside valid quaternary range [0..3]");

        let err3 = QuatError::IncompleteHeader;
        assert_eq!(format!("{err3}"), "Buffer is too short to contain a valid MP3 header");
    }
}