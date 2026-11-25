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
- **ID3v1 Tag Support**: Automatically detects and parses ID3v1 tags from the end of files (title, artist, album, and year).
- **Idiomatic Iterators Support**: Full support for standard Rust iterators (`IntoIterator`, `FromIterator`, `iter()`) for clean and ergonomic quat traversal.
- **Multi-language FFI Integration**: Fast and safe FFI layer allowing seamless integration into C, C++ (with RAII smart pointers), and C# (.NET) applications.
- **VBR Info Parsing**: Automatic detection and parsing of variable bitrate (Xing/Info) headers with preserved tuple structure and the `vbr_opt` parameter.
- **Custom Benchmarking Suite**: Built-in performance measurement for new features (resync, VBR, and ID3v1) based on `std::time::Instant` without requiring external dependencies like `criterion`.
- **FFI & Pattern Matching Sync**: Synchronized 3-element tuples and pattern matchings in `ffi.rs` and `lib.rs` (`Mp3HeaderKind::MpegFrame` with `_vbr_info`) for stable cross-language communication.
- **Incremental Streaming Parser & Metrics**: Feeds encoded data incrementally and tracks parsing telemetry (`ParserMetrics`, sync attempts, and inspected quats).
- **Bounds-Safe Parsing**: Adds length and bounds checks when reading buffer elements and VBR/tag headers to avoid out-of-bounds panics on short or partial inputs.
- **Expanded Test & Timing Coverage**: Tests `QuatVec`, stream chunking, recoverable stream errors, incremental parsing, and VBR parsing with built-in `std::time::Instant` execution timing.

---

## Benchmark

| Test Name | Module / Area | Status | Execution Time | Notes |
| :--- | :--- | :--- | :--- | :--- |
| `test_1_quat_basic_bits_and_values` | `quat` | OK | `100ns` | Basic quaternary bit values and representation |
| `test_2_quat_try_from_u8` | `quat` | OK | `300ns` | Fallible conversion from `u8` to quaternary values |
| `test_3_buffer_encode_decode_roundtrip` | `buffer` | OK | `3µs` | Full encode and decode roundtrip on `QuatBuffer` |
| `test_4_buffer_invalid_length_error` | `buffer` | OK | `400ns` | Error handling on invalid buffer lengths |
| `test_5_buffer_read_from_stream` | `buffer` | OK | `7.9µs` | Reading raw data from an input stream into buffer |
| `test_6_buffer_write_decoded_to_stream` | `buffer` | OK | `9µs` | Writing decoded output to a stream |
| `test_7_inspector_clean_mpeg_header` | `inspector` | OK | `17.2µs` | Detection of a clean MPEG frame header |
| `test_9_inspector_frame_sync_with_preceding_noise` | `inspector` | OK | `18µs` | Frame resynchronization after leading noise bytes |
| `test_10_inspector_heavy_stress_random_garbage` | `inspector` | OK | `3.5994ms` | Heavy stress test against random garbage input |
| `test_11_quat_vec_from_iterator_and_error_helpers` | `quat` / `error` | OK | `29.8µs` | `FromIterator` collection for `QuatVec` and recoverable error helpers |
| `test_12_stream_reader_read_encoded_into_incremental` | `buffer` | OK | `26.5µs` | Incremental chunked reading via `QuatStreamReader::read_encoded_into` |
| `test_13_incremental_parser_id3v2_and_metrics` | `inspector` | OK | `67µs` | Incremental feeding with ID3v2 data and parser metrics tracking |
| `test_14_parse_real_xing_vbr_header` | `inspector` | OK | `42.1µs` | Xing VBR header parsing over quaternary input |
| `test_inspector_vbr_detection` | `inspector` | OK | `177.1µs` | VBR header detection (`VBR Info detected successfully: None` on sample) |
| `test_inspector_id3v1_tag_detection` | `inspector` | OK | `242.2µs` | ID3v1 tag detection and field parsing |
| `test_quat_streaming_api` | `buffer` / `quat` | OK | `21.8µs` | Streaming API workflow verification |
| `test_ffi_buffer_lifecycle_and_codec` | `ffi` | OK | `4µs` | FFI buffer allocation, codec operations, and cleanup |
| `test_ffi_inspector_empty_buffer` | `ffi` | OK | `2µs` | FFI inspection behavior on an empty buffer |
| `test_ffi_null_pointer_handling` | `ffi` | OK | `500ns` | Safe handling of null pointers across FFI boundary |

## Licensing

This project is dual-licensed:

* **Open Source (AGPLv3):** Free to use for open-source projects. If you use or modify this library in your software or network service, your source code must also be open-source under AGPLv3.
* **Commercial License:** Required if you want to use `quat_mp3` in closed-source or proprietary commercial software without AGPLv3 restrictions.

For commercial licensing, contact: **strahinjastojanovic826@gmail.com**

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
quat_mp3 = "1.1.0"
```

### Basic Encoding and Decoding

```rust
use quat_mp3::{QuatBuffer, Quat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw_bytes = vec![0x49, 0x44, 0x33]; // ASCII "ID3"

    // Enkodiranje bajtova u kvartarni bafer
    let buffer = QuatBuffer::encode(&raw_bytes);
    assert_eq!(buffer.len(), 12);

    // Korišćenje iteratora umesto stare as_quats() metode
    for quat in &buffer {
        print!("{quat}"); // Štampa kvartarne cifre (0-3)
    }
    println!();

    // Dekodiranje nazad u sirove bajtove
    let decoded = buffer.decode()?;
    assert_eq!(decoded, raw_bytes);

    Ok(())
}
```

### Incremental Parsing, Stream Chunking & `QuatVec`

```rust
use quat_mp3::buffer::{QuatBuffer, QuatStreamReader};
use quat_mp3::inspector::IncrementalMp3Parser;
use quat_mp3::quat::QuatVec;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build QuatVec via FromIterator<u8>
    let quat_vec: QuatVec = [0u8, 1u8, 2u8, 3u8].into_iter().collect();
    assert_eq!(quat_vec.0.len(), 4);

    // Stream input into QuatBuffer in chunks
    let stream_data = b"QuatMP3 streaming chunk example";
    let mut reader = QuatStreamReader::new(Cursor::new(stream_data.to_vec()));
    let mut buf = QuatBuffer::new();
    while reader.read_encoded_into(&mut buf, 8)? > 0 {}

    // Feed buffer into incremental parser and inspect metrics
    let mut parser = IncrementalMp3Parser::new();
    parser.feed(&buf);
    println!(
        "Inspected quats: {}, sync attempts: {}",
        parser.metrics.total_quats_inspected,
        parser.metrics.sync_attempts
    );

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

    // 2. MPEG Frame detection with sync search, offset tracking & VBR info
    let mut raw_stream = vec![0x00, 0x01, 0x02]; // Noise bytes
    raw_stream.extend_from_slice(&[0xFF, 0xFB, 0x90, 0x00]);

    let stream_buffer = QuatBuffer::encode(&raw_stream);

    match Mp3Inspector::detect_header(&stream_buffer) {
        Mp3HeaderKind::MpegFrame { header, offset, vbr_info } => {
            println!("Found MPEG Frame at quat offset: {}, VBR Info: {:?}", offset, vbr_info);
            println!("Bitrate: {} kbps, Sample Rate: {} Hz", header.bitrate_kbps, header.sample_rate_hz);
            assert_eq!(header.version, MpegVersion::Mpeg1);
            assert_eq!(header.layer, MpegLayer::Layer3);
        }
        other => panic!("Expected MpegFrame, got: {:?}", other),
    }

    // 3. ID3v1 Tag Detection Example
    let mut id3v1_raw = vec![0u8; 100];
    let mut id3v1_tag = vec![0u8; 128];
    id3v1_tag[0..3].copy_from_slice(b"TAG");
    id3v1_tag[3..18].copy_from_slice(b"Sample Title");
    id3v1_tag[93..97].copy_from_slice(b"2026");
    id3v1_raw.extend(&id3v1_tag);

    let id3v1_buffer = QuatBuffer::encode(&id3v1_raw);
    match Mp3Inspector::detect_header(&id3v1_buffer) {
        Mp3HeaderKind::Id3v1 { title, artist, album, year } => {
            println!("ID3v1 Title: {}, Year: {}", title, year);
            assert_eq!(title, "Sample Title");
            assert_eq!(year, 2026);
        }
        other => panic!("Expected Id3v1 tag, got: {:?}", other),
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

# quat_mp3 FFI Integration Guide

This guide demonstrates how you can easily import and use the `quat_mp3` FFI library for working with quaternary buffers (`QuatBuffer`)[cite: 7, 8] and inspecting MP3 headers (`Mp3Inspector`)[cite: 3, 4] in **C**, **C++**, and **C#**.

---

## 1. C Integration (`main.c`)

In C, you include the header (`quat_mp3.h`) and directly call functions with manual memory management.

```c
#include <stdio.h>
#include <stdint.h>
#include "quat_mp3.h"

int main() {
    // 1. Create buffer (FfiQuatBuffer)
    FfiQuatBuffer* buffer = quat_buffer_new();
    if (!buffer) {
        printf("Failed to allocate buffer.\n");
        return 1;
    }

    // 2. Encode raw bytes (e.g., MPEG header with VBR or ID3v1)
    uint8_t data[] = { 0xFF, 0xFB, 0x90, 0x00 };
    QuatFFIError err = quat_buffer_encode(buffer, data, sizeof(data));
    if (err != QUAT_SUCCESS) {
        printf("Encoding failed with error code: %d\n", err);
        quat_buffer_free(buffer);
        return 1;
    }

    // 3. Inspect MP3 header (Mp3Inspector)
    FfiAdvancedHeaderResult headerKind;
    quat_inspect_buffer(buffer, &headerKind);

    if (headerKind.tag == QUAT_TAG_MPEG_FRAME) {
        printf("MPEG Frame found! Bitrate: %d kbps, Sample Rate: %d Hz\n",
               headerKind.header.bitrate_kbps,
               headerKind.header.sample_rate_hz);
        
        // Checking VBR info if available
        if (headerKind.vbr_data.is_vbr) {
            printf("VBR detected! Total frames: %u, Total bytes: %u\n",
                   headerKind.vbr_data.total_frames,
                   headerKind.vbr_data.total_bytes);
        }
    } else if (headerKind.tag == QUAT_TAG_ID3V1) {
        printf("ID3v1 tag detected!\n");
    } else {
        printf("No valid MPEG frame or ID3v1 tag found.\n");
    }

    // 4. Free memory
    quat_buffer_free(buffer);
    return 0;
} 
```

---

## 2. C++ Integration (`main.cpp`)

In C++, you can use the RAII principle (e.g., smart pointers) to automatically release memory when the object goes out of scope.

```cpp
#include <iostream>
#include <vector>
#include <memory>
#include "quat_mp3.h"

// RAII Deleter for FfiQuatBuffer
struct QuatBufferDeleter {
    void operator()(FfiQuatBuffer* ptr) const {
        quat_buffer_free(ptr);
    }
};

using SafeQuatBuffer = std::unique_ptr<FfiQuatBuffer, QuatBufferDeleter>;

int main() {
    // 1. Safe allocation using unique_ptr for QuatBuffer[cite: 7, 8]
    SafeQuatBuffer buffer(quat_buffer_new());
    if (!buffer) {
        std::cerr << "Failed to create FfiQuatBuffer.\n";
        return 1;
    }

    // 2. Encode bytes
    std::vector<uint8_t> rawData = { 0x12, 0x34, 0x56, 0x78 };
    QuatFFIError err = quat_buffer_encode(buffer.get(), rawData.data(), rawData.size());
    
    if (err != QUAT_SUCCESS) {
        std::cerr << "Encoding failed.\n";
        return 1;
    }

    // 3. Inspect buffer via Mp3Inspector[cite: 3, 4]
    FfiMp3HeaderKind headerKind{};
    quat_inspect_buffer(buffer.get(), &headerKind);

    std::cout << "Inspection completed. Header tag type: " << static_cast<int>(headerKind.tag) << "\n";

    // Memory is automatically freed here via quat_buffer_free when `buffer` goes out of scope
    return 0;
}
```

---

## 3. C# Integration (`Program.cs`)

In C#, use the defined `QuatNative` class with `DllImport` attributes to interact with the dynamic library (`.dll` on Windows or `.so` on Linux).

```csharp
using System;

namespace QuatMp3App
{
    class Program
    {
        static void Main(string[] args)
        {
            // 1. Create a new buffer pointer[cite: 7, 8]
            IntPtr bufferPtr = QuatNative.quat_buffer_new();
            if (bufferPtr == IntPtr.Zero)
            {
                Console.WriteLine("Failed to create native buffer.");
                return;
            }

            try
            {
                // 2. Encode data
                byte[] data = { 0x12, 0x34, 0x56, 0x78 };
                QuatFFIError err = QuatNative.quat_buffer_encode(bufferPtr, data, (IntPtr)data.Length);

                if (err != QuatFFIError.Success)
                {
                    Console.WriteLine($"Encoding failed with error: {err}");
                    return;
                }

                // 3. Decode data back to verify[cite: 7, 8]
                byte[] decodedData = new byte[4];
                IntPtr outLen = (IntPtr)decodedData.Length;
                err = QuatNative.quat_buffer_decode(bufferPtr, decodedData, ref outLen);

                if (err == QuatFFIError.Success)
                {
                    Console.WriteLine($"Successfully decoded {outLen} bytes!");
                    Console.WriteLine($"First byte: {decodedData[0]}");
                }
            }
            finally
            {
                // 4. Always free native memory
                QuatNative.quat_buffer_free(bufferPtr);
            }
        }
    }
}
```
