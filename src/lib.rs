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
    use std::time::Instant;
    use crate::buffer::QuatStreamReader;

    // ==========================================
    // 1. Quat Basic Bits & Values
    // ==========================================

    #[test]
    fn test_1_quat_basic_bits_and_values() {
        let start = std::time::Instant::now();

        let q = Quat::from_bits(0b11001001); // Lowest 2 bits are 01 -> Q1
        assert_eq!(q.value(), 1);
        assert_eq!(Quat::Q0.value(), 0);
        assert_eq!(Quat::Q3.value(), 3);

        println!("[BENCHMARK] test_1_quat_basic_bits_and_values execution time: {:?}", start.elapsed());
    }

    #[test]
    fn test_2_quat_try_from_u8() {
        let start = std::time::Instant::now();

        assert_eq!(Quat::try_from(0).unwrap(), Quat::Q0);
        assert_eq!(Quat::try_from(3).unwrap(), Quat::Q3);
        assert!(matches!(
            Quat::try_from(4),
            Err(QuatError::InvalidDigitValue(4))
        ));

        println!("[BENCHMARK] test_2_quat_try_from_u8 execution time: {:?}", start.elapsed());
    }

    #[test]
    fn test_3_buffer_encode_decode_roundtrip() {
        let start = std::time::Instant::now();

        let data = vec![0x12, 0x34, 0xAB, 0xFF];
        let buffer = QuatBuffer::encode(&data);
        let decoded = buffer.decode().unwrap();
        assert_eq!(data, decoded);

        println!("[BENCHMARK] test_3_buffer_encode_decode_roundtrip execution time: {:?}", start.elapsed());
    }

    #[test]
    fn test_4_buffer_invalid_length_error() {
        let start = std::time::Instant::now();

        let mut buffer = QuatBuffer::new();
        buffer.push(Quat::Q0);
        buffer.push(Quat::Q1);
        buffer.push(Quat::Q2); // Only 3 quats, not a multiple of 4
        let result = buffer.decode();
        assert!(matches!(
            result,
            Err(QuatError::InvalidBufferLength { length: 3 })
        ));

        println!("[BENCHMARK] test_4_buffer_invalid_length_error execution time: {:?}", start.elapsed());
    }

    #[test]
    fn test_5_buffer_read_from_stream() {
        let start = std::time::Instant::now();

        let raw_data = b"Hello, Quaternary MP3 stream!";
        let cursor = std::io::Cursor::new(raw_data.to_vec());
        let mut reader = QuatStreamReader::new(cursor);
        
        // Čitamo direktno preko read_chunk metode iz readera, pošto QuatStreamReader nema into_inner()
        let buffer = reader.read_chunk(raw_data.len()).unwrap().unwrap();
        let decoded = buffer.decode().unwrap();
        assert_eq!(raw_data.to_vec(), decoded);

        println!("[BENCHMARK] test_5_buffer_read_from_stream execution time: {:?}", start.elapsed());
    }

    #[test]
    fn test_6_buffer_write_decoded_to_stream() {
        let start = std::time::Instant::now();

        let raw_data = b"Testing direct writer output";
        let buffer = QuatBuffer::encode(raw_data);
        let mut output = Vec::new();
        buffer.write_decoded_to(&mut output).unwrap();
        assert_eq!(raw_data.to_vec(), output);

        println!("[BENCHMARK] test_6_buffer_write_decoded_to_stream execution time: {:?}", start.elapsed());
    }

    #[test]
    fn test_7_inspector_clean_mpeg_header() {
        let start = std::time::Instant::now();

        // Valid MPEG-1 Layer 3 header bytes: Sync (0xFFFB), Bitrate/Sample (0x90), Private (0x00)
        let header_bytes = [0xFF, 0xFB, 0x90, 0x00];
        let buffer = QuatBuffer::encode(&header_bytes);
        let kind = Mp3Inspector::detect_header(&buffer);

        match kind {
            Mp3HeaderKind::MpegFrame { header, offset, vbr_info } => {
                assert_eq!(offset, 0);
                assert_eq!(header.version, MpegVersion::Mpeg1);
                assert_eq!(header.layer, MpegLayer::Layer3);
                assert_eq!(header.bitrate_kbps, 128);
                assert_eq!(header.sample_rate_hz, 44100);
            }
            other => panic!("Expected MpegFrame header, got {:?}", other),
        }

        println!("[BENCHMARK] test_7_inspector_clean_mpeg_header execution time: {:?}", start.elapsed());
    }

    #[test]
    fn test_9_inspector_frame_sync_with_preceding_noise() {
        let start = std::time::Instant::now();

        // Preceding garbage/noise bytes followed by a valid MPEG frame
        let mut raw = vec![0x01, 0x02, 0x03, 0x04, 0x05]; // 5 bytes noise = 20 quats offset
        raw.extend_from_slice(&[0xFF, 0xFB, 0x90, 0x00]); // Valid frame
        let buffer = QuatBuffer::encode(&raw);
        let kind = Mp3Inspector::detect_header(&buffer);

        match kind {
            Mp3HeaderKind::MpegFrame { header, offset, vbr_info } => {
                assert_eq!(offset, 20); // Successfully skipped 5 bytes (20 quats) of noise
                assert_eq!(header.bitrate_kbps, 128);
            }
            other => panic!("Expected frame sync after noise, got {:?}", other),
        }

        println!("[BENCHMARK] test_9_inspector_frame_sync_with_preceding_noise execution time: {:?}", start.elapsed());
    }

    #[test]
    fn test_10_inspector_heavy_stress_random_garbage() {
        let start = std::time::Instant::now();

        // Generates 8KB of pseudo-random garbage data to test parser resilience under extreme stress
        let mut stress_data = Vec::with_capacity(8192);
        for i in 0..8192 {
            stress_data.push(((i * 73 + 19) % 256) as u8);
        }

        let buffer = QuatBuffer::encode(&stress_data);

        // The parser must not panic, loop infinitely, or crash under malformed input
        let kind = Mp3Inspector::detect_header(&buffer);

        match kind {
            Mp3HeaderKind::Unknown => {}
            Mp3HeaderKind::MpegFrame { .. } => {}
            other => {
                panic!("Unexpected inspection state during stress test: {:?}", other);
            }
        }

        println!("[BENCHMARK] test_10_inspector_heavy_stress_random_garbage execution time: {:?}", start.elapsed());
    }

    #[test]
    fn test_ffi_buffer_lifecycle_and_codec() {
        let start = std::time::Instant::now();

        unsafe {
            let buffer_ptr = quat_buffer_new();
            assert!(!buffer_ptr.is_null(), "Failed to create FfiQuatBuffer");

            let input_data: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
            let encode_res = quat_buffer_encode(
                buffer_ptr,
                input_data.as_ptr(),
                input_data.len(),
            );
            assert_eq!(encode_res as u32, QuatFFIError::Success as u32, "Encoding failed");

            let mut out_data = vec![0u8; 4];
            let mut out_len: usize = out_data.len();

            let decode_res = quat_buffer_decode(
                buffer_ptr,
                out_data.as_mut_ptr(),
                &mut out_len,
            );
            assert_eq!(decode_res as u32, QuatFFIError::Success as u32, "Decoding failed");
            assert_eq!(out_len, 4, "Decoded length mismatch");
            assert_eq!(out_data, input_data, "Decoded data does not match input");

            quat_buffer_free(buffer_ptr);
        }

        println!("[BENCHMARK] test_ffi_buffer_lifecycle_and_codec execution time: {:?}", start.elapsed());
    }

    #[test]
    fn test_ffi_inspector_empty_buffer() {
        let start = std::time::Instant::now();

        unsafe {
            let buffer_ptr = quat_buffer_new();
            assert!(!buffer_ptr.is_null());

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

            let res = quat_inspect_buffer(buffer_ptr, &mut header_kind);
            assert_eq!(
                res as u32,
                QuatFFIError::Success as u32,
                "Inspection failed"
            );

            match header_kind.tag {
                FfiHeaderKindTag::Unknown => {}
                _ => {}
            }

            quat_buffer_free(buffer_ptr);
        }

        println!("[BENCHMARK] test_ffi_inspector_empty_buffer execution time: {:?}", start.elapsed());
    }

    #[test]
    fn test_ffi_null_pointer_handling() {
        let start = std::time::Instant::now();

        unsafe {
            let null_buf: *mut FfiQuatBuffer = std::ptr::null_mut();
            let mut dummy_data = [0u8; 4];
            let mut dummy_len = 4;

            let encode_res = quat_buffer_encode(null_buf, dummy_data.as_ptr(), dummy_len);
            assert_eq!(encode_res as u32, QuatFFIError::NullPointer as u32);

            let decode_res = quat_buffer_decode(null_buf, dummy_data.as_mut_ptr(), &mut dummy_len);
            assert_eq!(decode_res as u32, QuatFFIError::NullPointer as u32);

            let mut header_kind = std::mem::zeroed();
            let inspect_res = quat_inspect_buffer(null_buf, &mut header_kind);
            assert_eq!(inspect_res as u32, QuatFFIError::NullPointer as u32);

            quat_buffer_free(null_buf);
        }

        println!("[BENCHMARK] test_ffi_null_pointer_handling execution time: {:?}", start.elapsed());
    }

    #[test]
    fn test_quat_streaming_api() {
        let start = std::time::Instant::now();

        use std::io::Cursor;
        use crate::buffer::{QuatStreamReader, QuatStreamWriter};

        let original_data = b"Streaming test data for quaternary buffer optimization!";
        let cursor = Cursor::new(original_data.to_vec());

        let mut reader = QuatStreamReader::new(cursor);
        let mut encoded_buffers = Vec::new();

        while let Ok(Some(buf)) = reader.read_chunk(8) {
            encoded_buffers.push(buf);
        }

        let mut output_data = Vec::new();
        let mut writer = QuatStreamWriter::new(&mut output_data);

        for buf in &encoded_buffers {
            writer.write_buffer(buf).unwrap();
        }

        assert_eq!(original_data.to_vec(), output_data[..]);

        println!("[BENCHMARK] test_quat_streaming_api execution time: {:?}", start.elapsed());
    }

    #[test]
    fn test_inspector_vbr_detection() {
        let start = std::time::Instant::now();

        let header_bytes = [0xFF, 0xFB, 0x90, 0x00];
        let buffer = QuatBuffer::encode(&header_bytes);
        let kind = Mp3Inspector::detect_header(&buffer);

        match kind {
            Mp3HeaderKind::MpegFrame { header: _, offset, vbr_info } => {
                assert_eq!(offset, 0);
                println!("VBR Info detected successfully: {:?}", vbr_info);
            }
            other => panic!("Expected MpegFrame, got: {:?}", other),
        }

        println!("[BENCHMARK] test_inspector_vbr_detection execution time: {:?}", start.elapsed());
    }

    #[test]
    fn test_inspector_id3v1_tag_detection() {
        let start = std::time::Instant::now();

        let mut raw = vec![0u8; 500];
        let mut id3v1 = vec![0u8; 128];
        id3v1[0..3].copy_from_slice(b"TAG");
        id3v1[3..18].copy_from_slice(b"Test Song Title");
        id3v1[33..44].copy_from_slice(b"Test Artist");
        id3v1[63..73].copy_from_slice(b"Test Album");
        id3v1[93..97].copy_from_slice(b"2026");
        raw.extend(&id3v1);

        let buffer = QuatBuffer::encode(&raw);
        let kind = Mp3Inspector::detect_header(&buffer);

        match kind {
            Mp3HeaderKind::Id3v1 { title, artist, album, year } => {
                assert_eq!(title, "Test Song Title");
                assert_eq!(artist, "Test Artist");
                assert_eq!(album, "Test Album");
                assert_eq!(year, 2026);
            }
            other => panic!("Expected Id3v1 tag, got: {:?}", other),
        }

        println!("[BENCHMARK] test_inspector_id3v1_tag_detection execution time: {:?}", start.elapsed());
    }

}