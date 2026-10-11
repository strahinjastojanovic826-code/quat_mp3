# QuatMP3

A lightweight, experimental Rust library for processing MP3 audio metadata and frame headers using **quaternary (base-4) logic**. 

Instead of operating on standard octets directly, `quat_mp3` maps binary data into 2-bit quaternary digits (**quats**), enabling native quaternary operations, byte-to-quat encoding/decoding, and direct pattern matching over base-4 representations.

---

## Key Features

- **Base-4 Bit Packing**: Directly maps pairs of bits (`'00'`, `'01'`, `'10'`, `'11'`) into quaternary digits (`Quat::Q0` to `Quat::Q3`)[cite: 10].
- **Stream & I/O Support**: Seamlessly reads raw data directly from `std::io::Read` and writes decoded bytes to `std::io::Write`[cite: 10].
- **Robust Frame Resynchronization (Sync Search)**: Scans buffers byte-by-byte to skip preceding noise or corrupted data, successfully locating valid MPEG sync words.
- **Full MPEG & Layer Compatibility**: Comprehensive bitrate and sample rate lookup tables supporting all versions (**MPEG-1, MPEG-2, MPEG-2.5**) and layers (**Layer 1, Layer 2, Layer 3**).
- **Advanced ID3v2 Handling**: Automatically parses ID3v2 tag sizes and skips them to correctly locate underlying audio frames.
- **Standard & Safe Conversions**: Full support for standard Rust traits including `TryFrom`, `From`, `Deref`, and `FromIterator`[cite: 10].
- **Idiomatic Error Handling**: Comprehensive error management using a custom `QuatError` enum powered by `thiserror`[cite: 10].

---

## Quaternary Bit Mapping

Each byte (8 bits) is sliced from Most Significant Bit (MSB) to Least Significant Bit (LSB) into 4 quaternary digits:

| Binary Pair | Base-10 | Base-4 Digit (`Quat`) |
| :---: | :---: | :---: |
| `00` | 0 | `Quat::Q0` |
| `01` | 1 | `Quat::Q1` |
| `10` | 2 | `Quat::Q2` |
| `11` | 3 | `Quat::Q3` |

### Example
The byte `0xE4` (`0b11_10_01_00`) translates directly to:
```text
[ Quat::Q3, Quat::Q2, Quat::Q1, Quat::Q0 ]  // Value sequence: 3, 2, 1, 0
```

## Usage

Add `quat_mp3` to your `Cargo.toml`:

```toml
[dependencies]
quat_mp3 = "0.3.0"
```

### Basic Encoding and Decoding

```rust
use quat_mp3::{QuatBuffer, Quat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw_bytes = vec![0x49, 0x44, 0x33]; // ASCII "ID3"
    
    // Encode bytes to 4 quats per byte (12 quats total)
    let buffer = QuatBuffer::encode(&raw_bytes);
    assert_eq!(buffer.len(), 12);

    // Inspect quaternary representation
    for quat in buffer.as_quats() {
        print!("{quat}"); // Prints quaternary digits (0-3)
    }
    println!();

    // Decode back to raw bytes
    let decoded = buffer.decode()?;
    assert_eq!(decoded, raw_bytes);

    Ok(())
}
```

### MP3 Header Detection

```rust
use quat_mp3::buffer::QuatBuffer;
use quat_mp3::inspector::{Mp3Inspector, Mp3HeaderKind, MpegVersion, MpegLayer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. ID3v2 tag detection and size parsing
    let id3_bytes = vec![b'I', b'D', b'3', 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0A];
    let id3_buffer = QuatBuffer::encode(&id3_bytes);

    assert!(matches!(
        Mp3Inspector::detect_header(&id3_buffer),
        Mp3HeaderKind::Id3v2 { size_bytes: 10 }
    ));

    // 2. MPEG Frame detection with sync search & offset tracking
    // Preceding noise followed by a valid MPEG-1 Layer 3 frame header (0xFFFB9000)
    let mut raw_stream = vec![0x00, 0x01, 0x02]; // Noise bytes
    raw_stream.extend_from_slice(&[0xFF, 0xFB, 0x90, 0x00]);
    
    let stream_buffer = QuatBuffer::encode(&raw_stream);
    
    match Mp3Inspector::detect_header(&stream_buffer) {
        Mp3HeaderKind::MpegFrame { header, offset } => {
            println!("Found MPEG Frame at quat offset: {}", offset);
            println!("Bitrate: {} kbps, Sample Rate: {} Hz", header.bitrate_kbps, header.sample_rate_hz);
            assert_eq!(header.version, MpegVersion::Mpeg1);
            assert_eq!(header.layer, MpegLayer::Layer3);
        }
        other => panic!("Expected MpegFrame, got {:?}", other),
    }

    Ok(())
}
```

---

## Running Tests

To run the complete unit and integration test suite:

```bash
cargo test
```

---
