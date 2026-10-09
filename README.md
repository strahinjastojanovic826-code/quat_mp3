# QuatMP3

A lightweight, experimental Rust library for processing MP3 audio metadata and frame headers using **quaternary (base-4) logic**. 

Instead of operating on standard octets directly, `quat_mp3` maps binary data into 2-bit quaternary digits (**quats**), enabling native quaternary operations, byte-to-quat encoding/decoding, and direct pattern matching over base-4 representations.

---

## Key Features

- **Base-4 Bit Packing**: Directly maps pairs of bits (`00`, `01`, `10`, `11`) into quaternary digits (`Quat::Q0` through `Quat::Q3`).
- **Stream & I/O Support**: Seamlessly reads raw data directly from `std::io::Read` streams and writes decoded bytes to `std::io::Write` streams.
- **Header Inspection**: Inspects and detects ID3v2 metadata and MPEG-1/2/2.5 frame headers straight from quaternary buffers.
- **Standard & Safe Conversions**: Full support for standard Rust traits including `TryFrom`, `From`, `FromIterator`, and `Deref` for slice-like manipulation.
- **Idiomatic Error Handling**: Comprehensive error management using a custom `QuatError` enum implementing `std::error::Error` and `Display`.

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

---

# Test Results

The test suite evaluates the library across multiple difficulty levels, ranging from basic conversions and buffer handling to heavy fuzzing and large-scale stress tests.

## Summary
* **Status**: 8 passed, 0 failed, 0 ignored[cite: 9]
* **Execution Time**: 1.03 seconds[cite: 9]

## Test Suite Breakdown

* **ID3v2 Header Inspection**
  * Successfully constructed a valid 10-byte ID3v2 header totaling 40 quats[cite: 9].
  * Accurately detected and parsed the payload size to 257 bytes[cite: 9].

* **Test 1: Easy Quat Conversions**
  * Successfully processed the stream and generated 32,768 quats[cite: 9].

* **Test 2: Easy Buffer Encode / Decode (Level 2/8)**
  * Tested small byte payload encoding (`[18, 171, 255, 0]`) into 16 quats with 100% roundtrip fidelity[cite: 9].

* **Test 3: Medium Invalid Buffer Length Error (Level 3/8)**
  * Verified proper error handling for incomplete quat sequences that are not multiples of 4[cite: 9].

* **Test 4: Medium ID3v2 Header Detection (Level 4/8)**
  * Validated robust ID3v2 header recognition and parsing accuracy[cite: 9].

* **Test 5: Hard MPEG Frame Header Parsing (Level 5/8)**
  * Successfully parsed and validated MPEG frame parameters[cite: 9].

* **Test 6: Hard Stream Reading Chunked (Level 6/8)**
  * Read and re-decoded 8,192 bytes with complete fidelity[cite: 9].
  * Successfully scanned a 4,000,000-quat noisy buffer and correctly extracted a deeply embedded MPEG sync header (MPEG-1, Layer 3, 128 kbps, 44100 Hz, padding: false)[cite: 9].

* **Test 7: Stress Large Data Processing (Level 7/8)**
  * Encoded large data in 403.74 ms producing 41,943,040 quats[cite: 9].
  * Decoded the data in 514.63 ms[cite: 9].
  * Completed a 10 MB stress test with a total execution time of 918.37 ms[cite: 9].

* **Test 8: Stress Corrupted Bitstream Fuzzing (Level 8/8 - Stress & Noise)**
  * Fed 1 MB of corrupted noise data into the parser to test resilience[cite: 9].
  * Verified complete parser stability and recovery without panics[cite: 9].

---

## Usage

Add `quat_mp3` to your `Cargo.toml`:

```toml
[dependencies]
quat_mp3 = "0.2.0"
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
use quat_mp3::inspector::{Mp3Inspector, Mp3HeaderKind};

fn main() {
    // ID3v2 header magic bytes
    let id3_bytes = vec![0x49, 0x44, 0x33, 0x03, 0x00];
    let id3_buffer = QuatBuffer::encode(&id3_bytes);

    // Koristimo matches! zbog { size_bytes } polja unutar Id3v2 varijanta
    assert!(matches!(
        Mp3Inspector::detect_header(&id3_buffer),
        Mp3HeaderKind::Id3v2 { .. }
    ));

    // MPEG Frame (11 bits set to 1)
    let sync_bytes = vec![0xFF, 0xFB, 0x90, 0x64];
    let sync_buffer = QuatBuffer::encode(&sync_bytes);

    // Koristimo matches! umesto MpegFrameSync i zbog { header } polja
    assert!(matches!(
        Mp3Inspector::detect_header(&sync_buffer),
        Mp3HeaderKind::MpegFrame { .. }
    ));
}
```

---

## Running Tests

To run the complete unit and integration test suite:

```bash
cargo test
```

---
