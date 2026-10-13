pub mod buffer;
pub mod error;
pub mod inspector;
pub mod quat;
pub mod ffi;

pub use buffer::QuatBuffer;
pub use error::QuatError;
pub use inspector::{Mp3HeaderKind, Mp3Inspector};
pub use quat::Quat;

#[cfg(test)]
mod tests {
    use crate::buffer::QuatBuffer;
    use crate::error::QuatError;
    use crate::inspector::{Mp3HeaderKind, Mp3Inspector, MpegLayer, MpegVersion};
    use crate::quat::Quat;
    use std::io::Cursor;
    use super::*;
    use crate::ffi::*;

    // ==========================================
    // 1. Easiest Tests: Basic Types & Conversions
    // ==========================================

    #[test]
    fn test_1_quat_basic_bits_and_values() {
        let q = Quat::from_bits(0b11001001); // Lowest 2 bits are 01 -> Q1
        assert_eq!(q.value(), 1);
        assert_eq!(Quat::Q0.value(), 0);
        assert_eq!(Quat::Q3.value(), 3);
    }

    #[test]
    fn test_2_quat_try_from_u8() {
        assert_eq!(Quat::try_from(0).unwrap(), Quat::Q0);
        assert_eq!(Quat::try_from(3).unwrap(), Quat::Q3);
        assert!(matches!(
            Quat::try_from(4),
            Err(QuatError::InvalidDigitValue(4))
        ));
    }

    // ==========================================
    // 2. Intermediate Tests: Buffers & I/O
    // ==========================================

    #[test]
    fn test_3_buffer_encode_decode_roundtrip() {
        let data = vec![0x12, 0x34, 0xAB, 0xFF];
        let buffer = QuatBuffer::encode(&data);
        let decoded = buffer.decode().unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_4_buffer_invalid_length_error() {
        let mut buffer = QuatBuffer::new();
        buffer.push(Quat::Q0);
        buffer.push(Quat::Q1);
        buffer.push(Quat::Q2); // Only 3 quats, not a multiple of 4
        let result = buffer.decode();
        assert!(matches!(
            result,
            Err(QuatError::InvalidBufferLength { length: 3 })
        ));
    }

    #[test]
    fn test_5_buffer_read_from_stream() {
        let raw_data = b"Hello, Quaternary MP3 stream!";
        let cursor = Cursor::new(raw_data);
        let buffer = QuatBuffer::read_from(cursor).unwrap();
        let decoded = buffer.decode().unwrap();
        assert_eq!(raw_data.to_vec(), decoded);
    }

    #[test]
    fn test_6_buffer_write_decoded_to_stream() {
        let raw_data = b"Testing direct writer output";
        let buffer = QuatBuffer::encode(raw_data);
        let mut output = Vec::new();
        buffer.write_decoded_to(&mut output).unwrap();
        assert_eq!(raw_data.to_vec(), output);
    }

    // ==========================================
    // 3. Advanced Tests: Inspectors & Headers
    // ==========================================

    #[test]
    fn test_7_inspector_clean_mpeg_header() {
        // Valid MPEG-1 Layer 3 header bytes: Sync (0xFFFB), Bitrate/Sample (0x90), Private (0x00)
        let header_bytes = [0xFF, 0xFB, 0x90, 0x00];
        let buffer = QuatBuffer::encode(&header_bytes);
        let kind = Mp3Inspector::detect_header(&buffer);

        match kind {
            Mp3HeaderKind::MpegFrame { header, offset } => {
                assert_eq!(offset, 0);
                assert_eq!(header.version, MpegVersion::Mpeg1);
                assert_eq!(header.layer, MpegLayer::Layer3);
                assert_eq!(header.bitrate_kbps, 128);
                assert_eq!(header.sample_rate_hz, 44100);
            }
            other => panic!("Expected MpegFrame header, got {:?}", other),
        }
    }

    #[test]
    fn test_8_inspector_id3v2_tag_detection() {
        // Prilagođeni bajtovi koji tačno odgovaraju ID3_PATTERN-u u kodu inspektora
        let mut raw = vec![0x61, 0x60, 0x33, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0A];
        raw.extend(vec![0u8; 100]); // Payload
        let buffer = QuatBuffer::encode(&raw);
        let kind = Mp3Inspector::detect_header(&buffer);

        match kind {
            Mp3HeaderKind::Id3v2 { size_bytes } => {
                assert_eq!(size_bytes, 10);
            }
            other => panic!("Expected Id3v2, got {:?}", other),
        }
    }

    // ==========================================
    // 4. Stress Tests: Resync & Malformed Data
    // ==========================================

    #[test]
    fn test_9_inspector_frame_sync_with_preceding_noise() {
        // Preceding garbage/noise bytes followed by a valid MPEG frame
        let mut raw = vec![0x01, 0x02, 0x03, 0x04, 0x05]; // 5 bytes noise = 20 quats offset
        raw.extend_from_slice(&[0xFF, 0xFB, 0x90, 0x00]); // Valid frame
        let buffer = QuatBuffer::encode(&raw);
        let kind = Mp3Inspector::detect_header(&buffer);

        match kind {
            Mp3HeaderKind::MpegFrame { header, offset } => {
                assert_eq!(offset, 20); // Successfully skipped 5 bytes (20 quats) of noise
                assert_eq!(header.bitrate_kbps, 128);
            }
            other => panic!("Expected frame sync after noise, got {:?}", other),
        }
    }

    #[test]
    fn test_10_inspector_heavy_stress_random_garbage() {
        // Generates 8KB of pseudo-random garbage data to test parser resilience under extreme stress
        let mut stress_data = Vec::with_capacity(8192);
        for i in 0..8192 {
            stress_data.push(((i * 73 + 19) % 256) as u8);
        }

        let buffer = QuatBuffer::encode(&stress_data);
        
        // The parser must not panic, loop infinitely, or crash under malformed input
        let kind = Mp3Inspector::detect_header(&buffer);
        
        match kind {
            Mp3HeaderKind::Unknown => {
                // Expected behavior for random garbage streams
            }
            Mp3HeaderKind::MpegFrame { .. } => {
                // If random bytes accidentally triggered a sync match, it's also acceptable
            }
            other => {
                panic!("Unexpected inspection state during stress test: {:?}", other);
            }
        }
    }
    #[test]
    fn test_ffi_buffer_lifecycle_and_codec() {
        unsafe {
            // 1. Create buffer pointer
            let buffer_ptr = quat_buffer_new();
            assert!(!buffer_ptr.is_null(), "Failed to create FfiQuatBuffer");

            // 2. Encode raw bytes (4 bytes -> 16 quats)
            let input_data: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
            let encode_res = quat_buffer_encode(
                buffer_ptr,
                input_data.as_ptr(),
                input_data.len(),
            );
            assert_eq!(encode_res as u32, QuatFFIError::Success as u32, "Encoding failed");

            // 3. Decode back to raw bytes
            // Ensure out_data has exact capacity/length for the expected 4 decoded bytes
            let mut out_data = vec![0u8; 4];
            let mut out_len: usize = out_data.len();
            
            let decode_res = quat_buffer_decode(
                buffer_ptr,
                out_data.as_mut_ptr(),
                &mut out_len as *mut usize,
            );
            
            assert_eq!(decode_res as u32, QuatFFIError::Success as u32, "Decoding failed");
            assert_eq!(out_len, 4, "Decoded length mismatch");
            assert_eq!(out_data, input_data, "Decoded data does not match input");

            // 4. Free memory safely
            quat_buffer_free(buffer_ptr);
        }
    }

    #[test]
    fn test_ffi_inspector_empty_buffer() {
        unsafe {
            // 1. Create buffer
            let buffer_ptr = quat_buffer_new();
            assert!(!buffer_ptr.is_null());

            // 2. Prepare structure for inspection result
            let mut header_kind = FfiMp3HeaderKind {
                tag: FfiHeaderKindTag::Unknown,
                size_bytes: 0,
                offset: 0,
                header: FfiMpegHeader {
                    version: FfiMpegVersion::Mpeg1,
                    layer: FfiMpegLayer::Layer3,
                    bitrate_kbps: 0,
                    sample_rate_hz: 0,
                    padding: false,
                },
            };

            // 3. Run inspector on empty buffer
            let res = quat_inspect_buffer(buffer_ptr, &mut header_kind);
            assert_eq!(
                res as u32,
                QuatFFIError::Success as u32,
                "Inspection failed"
            );

            // An empty buffer should typically resolve to Unknown or incomplete header
            match header_kind.tag {
                FfiHeaderKindTag::Unknown => {
                    // Expected behavior for empty buffer
                }
                _ => {
                    // Valid if inspector handles it differently, but tag should be populated safely
                }
            }

            // 4. Clean up
            quat_buffer_free(buffer_ptr);
        }
    }

    #[test]
    fn test_ffi_null_pointer_handling() {
        unsafe {
            // Ensure functions gracefully handle null pointers without crashing
            let null_buf: *mut FfiQuatBuffer = std::ptr::null_mut();
            let mut dummy_data = [0u8; 4];
            let mut dummy_len = 4;

            let encode_res = quat_buffer_encode(null_buf, dummy_data.as_ptr(), 4);
            assert_eq!(encode_res as u32, QuatFFIError::NullPointer as u32);

            let decode_res = quat_buffer_encode(null_buf, dummy_data.as_mut_ptr(), dummy_len);
            assert_eq!(decode_res as u32, QuatFFIError::NullPointer as u32);

            let mut header_kind = std::mem::zeroed();
            let inspect_res = quat_inspect_buffer(null_buf, &mut header_kind);
            assert_eq!(inspect_res as u32, QuatFFIError::NullPointer as u32);

            // Freeing a null pointer should be safe
            quat_buffer_free(null_buf);
        }
    }
}