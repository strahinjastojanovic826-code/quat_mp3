# QuatMP3

A lightweight, experimental Rust library for processing MP3 audio metadata and frame headers using **quaternary (base-4) logic**. 

Instead of operating on standard octets directly, `quat_mp3` maps binary data into 2-bit quaternary digits (**quats**), enabling native quaternary operations, byte-to-quat encoding/decoding, and direct pattern matching over base-4 representations.

---

## Key Features

- **Base-4 Bit Packing**: Directly maps pairs of bits (`00`, `01`, `10`, `11`) into 4-value quaternary digits (`Q0`, `Q1`, `Q2`, `Q3`).
- **Zero-Copy & Standard Conversions**: Convert back and forth between byte slices and quaternary buffers safely.
- **Header Inspection**: Detect ID3v2 metadata and MPEG Frame Sync markers directly from quaternary buffers without full byte decoding.
- **Idiomatic Error Handling**: Strong typing with `std::error::Error` support and `TryFrom` trait implementations.

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

## Usage

Add `quat_mp3` to your `Cargo.toml`:

```toml
[dependencies]
quat_mp3 = { path = "." }
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
use quat_mp3::{QuatBuffer, Mp3Inspector, Mp3HeaderKind};

fn main() {
    // ID3v2 header magic bytes
    let id3_bytes = vec![0x49, 0x44, 0x33, 0x03, 0x00];
    let id3_buffer = QuatBuffer::encode(&id3_bytes);

    assert_eq!(
        Mp3Inspector::detect_header(&id3_buffer),
        Mp3HeaderKind::Id3v2
    );

    // MPEG Frame Sync (11 bits set to 1)
    let sync_bytes = vec![0xFF, 0xFB, 0x90, 0x64];
    let sync_buffer = QuatBuffer::encode(&sync_bytes);

    assert_eq!(
        Mp3Inspector::detect_header(&sync_buffer),
        Mp3HeaderKind::MpegFrameSync
    );
}
```

---

## Running Tests

To run the complete unit and integration test suite:

```bash
cargo test
```

---
