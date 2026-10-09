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
    use std::io::Cursor;
    use crate::buffer::QuatBuffer;
    use crate::error::QuatError;
    use crate::inspector::{Mp3HeaderKind, Mp3Inspector, MpegLayer, MpegVersion};
    use crate::quat::Quat;

    // Helper function to render visual progress reports in tests
    fn print_test_report(test_name: &str, level: &str, details: &str) {
        println!("\n==================================================");
        println!("[TEST REPORT] Name: {test_name}");
        println!("[DIFFICULTY ] Level: {level}");
        println!("[DETAILS    ] {details}");
        println!("==================================================");
    }

    // -------------------------------------------------------------------------
    // 1. EASY: Single Quat conversions and bit extraction
    // -------------------------------------------------------------------------
    #[test]
    fn test_1_easy_quat_conversions() {
        print_test_report(
            "test_1_easy_quat_conversions",
            "1/8 - Easy",
            "Testing basic bit-to-Quat conversion and TryFrom/From traits."
        );

        assert_eq!(Quat::from_bits(0b00), Quat::Q0);
        assert_eq!(Quat::from_bits(0b01), Quat::Q1);
        assert_eq!(Quat::from_bits(0b10), Quat::Q2);
        assert_eq!(Quat::from_bits(0b11), Quat::Q3);

        // Masking higher bits check (e.g., 0b1101 & 0b11 = 0b01 -> Q1)
        assert_eq!(Quat::from_bits(0b1101), Quat::Q1);

        // TryFrom boundary checks
        assert_eq!(Quat::try_from(0).unwrap(), Quat::Q0);
        assert_eq!(Quat::try_from(3).unwrap(), Quat::Q3);
        assert!(Quat::try_from(4).is_err());

        println!("[SUCCESS] All basic Quat conversions passed as expected.");
    }

    // -------------------------------------------------------------------------
    // 2. EASY: Small Buffer Encode and Decode Roundtrip
    // -------------------------------------------------------------------------
    #[test]
    fn test_2_easy_buffer_encode_decode() {
        print_test_report(
            "test_2_easy_buffer_encode_decode",
            "2/8 - Easy",
            "Testing small byte payload encoding to QuatBuffer and decoding back."
        );

        let input_bytes = vec![0x12, 0xAB, 0xFF, 0x00];
        println!("[REPORT] Encoding {} bytes: {:?}", input_bytes.len(), input_bytes);

        let buffer = QuatBuffer::encode(&input_bytes);
        assert_eq!(buffer.len(), input_bytes.len() * 4);

        let decoded_bytes = buffer.decode().expect("Decoding should succeed");
        assert_eq!(input_bytes, decoded_bytes);

        println!("[SUCCESS] Roundtrip matched perfectly. Buffer length: {} quats.", buffer.len());
    }

    // -------------------------------------------------------------------------
    // 3. MEDIUM: Invalid Buffer Length Handling
    // -------------------------------------------------------------------------
    #[test]
    fn test_3_medium_invalid_buffer_length_error() {
        print_test_report(
            "test_3_medium_invalid_buffer_length_error",
            "3/8 - Medium",
            "Verifying error handling when decoding incomplete Quat sequences (not multiple of 4)."
        );

        let mut buffer = QuatBuffer::new();
        buffer.push(Quat::Q0);
        buffer.push(Quat::Q1);
        buffer.push(Quat::Q2); // Only 3 quats (requires 4 for 1 byte)

        println!("[REPORT] Attempting to decode buffer of size: {}", buffer.len());

        match buffer.decode() {
            Err(QuatError::InvalidBufferLength { length }) => {
                println!("[SUCCESS] Correctly caught InvalidBufferLength error with length: {length}");
                assert_eq!(length, 3);
            }
            Ok(_) => panic!("Decoding should have failed due to invalid buffer length!"),
            Err(other) => panic!("Unexpected error type received: {:?}", other),
        }
    }

    // -------------------------------------------------------------------------
    // 4. MEDIUM: ID3v2 Header Detection & Synchsafe Size Parsing
    // -------------------------------------------------------------------------
    #[test]
    fn test_4_medium_id3v2_header_detection() {
        print_test_report(
            "test_4_medium_id3v2_header_detection",
            "4/8 - Medium",
            "Constructing a valid 10-byte ID3v2 header and checking inspector parsing accuracy."
        );

        // ID3 header: 'I', 'D', '3', ver 4.0, flags 0x00, size bytes [0x00, 0x00, 0x02, 0x01] -> 257 bytes
        let mut id3_bytes = vec![b'I', b'D', b'3', 0x04, 0x00, 0x00, 0x00, 0x00, 0x02, 0x01];
        id3_bytes.resize(10, 0);

        let buffer = QuatBuffer::encode(&id3_bytes);
        println!("[REPORT] Encoded ID3 header buffer size: {} quats", buffer.len());

        let header_kind = Mp3Inspector::detect_header(&buffer);
        println!("[REPORT] Detected Header: {:?}", header_kind);

        if let Mp3HeaderKind::Id3v2 { size_bytes } = header_kind {
            assert_eq!(size_bytes, 257);
            println!("[SUCCESS] ID3v2 header recognized with parsed payload size: {size_bytes} bytes.");
        } else {
            panic!("Failed to identify valid ID3v2 header!");
        }
    }

    // -------------------------------------------------------------------------
    // 5. HARD: MPEG Frame Sync Detection & Header Parsing
    // -------------------------------------------------------------------------
    #[test]
    fn test_5_hard_mpeg_frame_header_parsing() {
        print_test_report(
            "test_5_hard_mpeg_frame_header_parsing",
            "5/8 - Hard",
            "Validating MPEG-1 Layer 3 sync header extraction (128 kbps, 44100 Hz)."
        );

        // MPEG-1 Layer 3 Header: 0xFF, 0xFB (sync + MPEG1 + Layer3), 0x90 (128kbps, 44100Hz), 0x00
        let mpeg_bytes = vec![0xFF, 0xFB, 0x90, 0x00];
        let buffer = QuatBuffer::encode(&mpeg_bytes);

        let header_kind = Mp3Inspector::detect_header(&buffer);
        println!("[REPORT] Parsed Header Result: {:?}", header_kind);

        if let Mp3HeaderKind::MpegFrame { header } = header_kind {
            assert_eq!(header.version, MpegVersion::Mpeg1);
            assert_eq!(header.layer, MpegLayer::Layer3);
            assert_eq!(header.bitrate_kbps, 128);
            assert_eq!(header.sample_rate_hz, 44100);
            assert!(!header.padding);
            println!("[SUCCESS] MPEG Frame parameters match spec (128 kbps / 44.1 kHz).");
        } else {
            panic!("Failed to recognize valid MPEG Frame sync header!");
        }
    }

    // -------------------------------------------------------------------------
    // 6. HARD: Chunked Stream Reading via std::io::Read
    // -------------------------------------------------------------------------
    #[test]
    fn test_6_hard_stream_reading_chunked() {
        print_test_report(
            "test_6_hard_stream_reading_chunked",
            "6/8 - Hard",
            "Testing 8 KB streamed reading through reader chunks into QuatBuffer."
        );

        let raw_stream_data: Vec<u8> = (0..8192).map(|i| (i % 256) as u8).collect();
        let reader = Cursor::new(raw_stream_data.clone());

        let buffer = QuatBuffer::read_from(reader).expect("Stream reading should succeed");
        println!("[REPORT] Stream processed. Generated quats: {}", buffer.len());

        assert_eq!(buffer.len(), raw_stream_data.len() * 4);
        let decoded = buffer.decode().expect("Decoding stream buffer failed");
        assert_eq!(decoded, raw_stream_data);

        println!("[SUCCESS] Stream read and re-decoded 8192 bytes with 100% fidelity.");
    }

    // -------------------------------------------------------------------------
    // 7. STRESS: Large Payload Performance (10 MB Data Processing)
    // -------------------------------------------------------------------------
    #[test]
    fn test_7_stress_large_data_processing() {
        print_test_report(
            "test_7_stress_large_data_processing",
            "7/8 - Stress",
            "Stress testing encoder/decoder with a 10 MB payload (40,000,000 Quat units)."
        );

        let payload_size = 10 * 1024 * 1024; // 10 MB
        println!("[REPORT] Generating {payload_size} bytes pseudo-random payload...");
        let large_payload: Vec<u8> = (0..payload_size).map(|x| (x ^ (x >> 3)) as u8).collect();

        let start_time = std::time::Instant::now();
        let buffer = QuatBuffer::encode(&large_payload);
        let encode_duration = start_time.elapsed();

        println!(
            "[REPORT] Encoding finished in {:?}. Quat buffer count: {}",
            encode_duration,
            buffer.len()
        );

        let decode_start = std::time::Instant::now();
        let decoded_payload = buffer.decode().expect("Stress decode failed");
        let decode_duration = decode_start.elapsed();

        println!("[REPORT] Decoding finished in {:?}", decode_duration);

        assert_eq!(large_payload.len(), decoded_payload.len());
        assert_eq!(large_payload, decoded_payload);

        println!(
            "[SUCCESS] 10 MB Stress test passed! Total time: {:?}",
            encode_duration + decode_duration
        );
    }

    // -------------------------------------------------------------------------
    // 8. STRESS / FUZZ: Corrupted MP3 Bitstream Header Search
    // -------------------------------------------------------------------------
    #[test]
    fn test_8_stress_corrupted_bitstream_fuzzing() {
        print_test_report(
            "test_8_stress_corrupted_bitstream_fuzzing",
            "8/8 - Stress & Noise",
            "Feeding 1 MB of corrupted noise data and ensuring parser resilience without panics."
        );

        // Generate 1 MB of invalid noise pattern
        let mut noisy_data: Vec<u8> = (0..1_000_000).map(|i| ((i * 37) % 251) as u8).collect();

        // Inject a valid frame header deep in the noise at byte 500,000
        let target_offset = 500_000;
        noisy_data[target_offset] = 0xFF;
        noisy_data[target_offset + 1] = 0xFB;
        noisy_data[target_offset + 2] = 0x90;
        noisy_data[target_offset + 3] = 0x00;

        let buffer = QuatBuffer::encode(&noisy_data);
        println!("[REPORT] Created 4,000,000 quat noisy buffer. Scanning headers...");

        // Ensure parser gracefully returns Unknown or handles random bytes without crashing
        let initial_header = Mp3Inspector::detect_header(&buffer);
        println!("[REPORT] Header check on noise offset 0: {:?}", initial_header);

        // Slice directly to injected header position (500,000 bytes = 2,000,000 quats)
        let slice_buffer = QuatBuffer::from_iter(
            buffer[target_offset * 4..(target_offset + 4) * 4].iter().copied()
        );

        let injected_header = Mp3Inspector::detect_header(&slice_buffer);
        println!("[REPORT] Header check on injected noise offset: {:?}", injected_header);

        if let Mp3HeaderKind::MpegFrame { header } = injected_header {
            assert_eq!(header.bitrate_kbps, 128);
            println!("[SUCCESS] Deeply embedded MPEG sync header correctly extracted from noise stream!");
        } else {
            panic!("Parser failed to extract valid frame header embedded inside noise!");
        }
    }
}