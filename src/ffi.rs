//! # FFI Safety & `unsafe` Contract
//!
//! This module exposes a C-compatible FFI interface (`extern "C"`) over `QuatBuffer` and
//! `Mp3Inspector`. All `unsafe` blocks and functions rely on the following invariants that the
//! caller (e.g., C, C++, or C#) must uphold:
//!
//! - **Pointer Validity (`*const` / `*mut`):** Pointers passed for `FfiQuatBuffer`, `out_len`,
//!   `out_kind`, and `out_result` must be non-null (where required), properly aligned, and point
//!   to initialized memory valid for at least the duration of the call. For an input `data` buffer
//!   of length `len`, the pointer must be valid for reading `len` bytes (unless `len == 0` / null
//!   is explicitly checked and handled).
//! - **Ownership & Memory Reclamation (`Box`):** `quat_buffer_new` heap-allocates an
//!   `FfiQuatBuffer` via `Box::into_raw` and transfers ownership to the caller. `quat_buffer_free`
//!   must be called at most once per pointer and only on pointers created by `quat_buffer_new`,
//!   after which the pointer must never be used again (*no use-after-free*, *no double-free*).
//! - **Source Pointers & Copying (`from_raw_parts`, `copy_nonoverlapping`):** `data` and
//!   `out_data` must not alias in an invalid manner when passed to `slice::from_raw_parts` and
//!   `ptr::copy_nonoverlapping`, and `out_data` must have capacity for at least `*out_len` bytes
//!   when calling `quat_buffer_decode`.
//! - **Exclusive / Single-Threaded Access:** The caller must ensure there are no concurrent mutable
//!   and immutable references to the same `FfiQuatBuffer` across overlapping FFI calls.

use std::os::raw::{c_uchar, c_uint, c_ushort};
use crate::buffer::QuatBuffer;
use crate::inspector::{Mp3Inspector, Mp3HeaderKind, MpegVersion, MpegLayer};
use crate::QuatError;
use crate::Quat;
use crate::inspector::VbrInfo;

#[repr(C)]
pub enum QuatFFIError {
    Success = 0,
    InvalidBufferLength = 1,
    InvalidDigitValue = 2,
    IncompleteHeader = 3,
    IoError = 4,
    NullPointer = 5,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FfiAdvancedHeaderResult {
    pub tag: FfiHeaderKindTag,
    pub version: FfiMpegVersion,
    pub layer: FfiMpegLayer,
    pub bitrate_kbps: u16,
    pub sample_rate_hz: u32,
    pub padding: u8,
    pub vbr: FfiVbrInfo,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FfiVbrInfo {
    pub is_vbr: u8, // 1 ako je VBR, 0 ako je CBR
    pub total_frames: u32,
    pub total_bytes: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum FfiMpegVersion {
    Mpeg25 = 0,
    Mpeg2 = 1,
    Mpeg1 = 2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum FfiMpegLayer {
    Layer3 = 0,
    Layer2 = 1,
    Layer1 = 2,
}

#[repr(C)]
pub struct FfiMpegHeader {
    pub version: FfiMpegVersion,
    pub layer: FfiMpegLayer,
    pub bitrate_kbps: c_ushort,
    pub sample_rate_hz: c_uint,
    pub padding: bool,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FfiParserMetrics {
    pub total_quats_inspected: usize,
    pub sync_attempts: usize,
    pub resync_skips: usize,
    pub id3v2_detected: u8,
    pub vbr_detected: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum FfiHeaderKindTag {
    Id3v2 = 0,
    MpegFrame = 1,
    Unknown = 2,
}

#[repr(C)]
pub struct FfiMp3HeaderKind {
    pub tag: FfiHeaderKindTag,
    pub size_bytes: usize, // Koristi se za Id3v2
    pub offset: usize,     // Koristi se za MpegFrame
    pub header: FfiMpegHeader, // Koristi se za MpegFrame
}

pub struct FfiQuatBuffer(QuatBuffer);

// --- Postojeće funkcije za buffer ---

#[unsafe(no_mangle)]
pub extern "C" fn quat_buffer_new() -> *mut FfiQuatBuffer {
    Box::into_raw(Box::new(FfiQuatBuffer(QuatBuffer::new())))
}

#[unsafe(no_mangle)]
pub extern "C" fn quat_buffer_free(buffer: *mut FfiQuatBuffer) {
    if !buffer.is_null() {
        unsafe { let _ = Box::from_raw(buffer); }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn quat_buffer_encode(
    buffer: *mut FfiQuatBuffer,
    data: *const c_uchar,
    len: usize,
) -> QuatFFIError {
    if buffer.is_null() || data.is_null() {
        return QuatFFIError::NullPointer;
    }
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    let buf = unsafe { &mut (*buffer).0 };
    *buf = QuatBuffer::encode(slice);
    QuatFFIError::Success
}

#[unsafe(no_mangle)]
pub extern "C" fn quat_buffer_decode(
    buffer: *const FfiQuatBuffer,
    out_data: *mut c_uchar,
    out_len: *mut usize,
) -> QuatFFIError {
    if buffer.is_null() || out_data.is_null() || out_len.is_null() {
        return QuatFFIError::NullPointer;
    }
    
    let buf = unsafe { &(*buffer).0 };
    
    match buf.decode() {
        Ok(bytes) => {
            let decoded_len = bytes.len();
            unsafe {
                if *out_len < decoded_len {
                    *out_len = decoded_len;
                    return QuatFFIError::InvalidBufferLength;
                }
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_data, decoded_len);
                *out_len = decoded_len;
            }
            QuatFFIError::Success
        }
        Err(e) => match e {
    QuatError::InvalidBufferLength { .. } => QuatFFIError::InvalidBufferLength,
    QuatError::InvalidDigitValue(_) => QuatFFIError::InvalidDigitValue,
    QuatError::IncompleteHeader => QuatFFIError::IncompleteHeader,
    QuatError::Io(_) => QuatFFIError::IoError,
    QuatError::InvalidVbrHeader => QuatFFIError::InvalidBufferLength,
    QuatError::InvalidId3v2 => QuatFFIError::IncompleteHeader,
    QuatError::CrcCheckFailed => QuatFFIError::InvalidBufferLength,
    _ => QuatFFIError::IoError, // Pokriva sve ostalo bez greške
},
    }
}

// --- Nova produktivna funkcija za Mp3Inspector ---

#[unsafe(no_mangle)]
pub extern "C" fn quat_inspect_buffer(
    buffer: *const FfiQuatBuffer,
    out_kind: *mut FfiMp3HeaderKind,
) -> QuatFFIError {
    if buffer.is_null() || out_kind.is_null() {
        return QuatFFIError::NullPointer;
    }

    let buf = unsafe { &(*buffer).0 };
    let kind = Mp3Inspector::detect_header(buf);

    unsafe {
        match kind {
            Mp3HeaderKind::Id3v2 { size_bytes } => {
                (*out_kind).tag = FfiHeaderKindTag::Id3v2;
                (*out_kind).size_bytes = size_bytes;
            }
            Mp3HeaderKind::Id3v1 { .. } => {
            (*out_kind).tag = FfiHeaderKindTag::Unknown;
        }
            Mp3HeaderKind::MpegFrame { header, offset, vbr_info: _ } => {
                (*out_kind).tag = FfiHeaderKindTag::MpegFrame;
                (*out_kind).offset = offset;
                (*out_kind).header = FfiMpegHeader {
                    version: match header.version {
                        MpegVersion::Mpeg25 => FfiMpegVersion::Mpeg25,
                        MpegVersion::Mpeg2 => FfiMpegVersion::Mpeg2,
                        MpegVersion::Mpeg1 => FfiMpegVersion::Mpeg1,
                    },
                    layer: match header.layer {
                        MpegLayer::Layer3 => FfiMpegLayer::Layer3,
                        MpegLayer::Layer2 => FfiMpegLayer::Layer2,
                        MpegLayer::Layer1 => FfiMpegLayer::Layer1,
                    },
                    bitrate_kbps: header.bitrate_kbps,
                    sample_rate_hz: header.sample_rate_hz,
                    padding: header.padding,
                };
            }
            Mp3HeaderKind::Unknown => {
                (*out_kind).tag = FfiHeaderKindTag::Unknown;
            }
        }
    }

    QuatFFIError::Success
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn quat_inspect_buffer_advanced(
    buffer: *const FfiQuatBuffer,
    out_result: *mut FfiAdvancedHeaderResult,
) -> QuatFFIError { unsafe {
    if buffer.is_null() || out_result.is_null() {
        return QuatFFIError::NullPointer;
    }

    let quat_buffer = &(*buffer).0;
    
    // Pretvaramo kvatarni bafer u niz Quat elemenata za inspektor
    let quats: Vec<Quat> = (0..quat_buffer.len())
        .filter_map(|i| quat_buffer.get(i))
        .collect();

    // Koristimo našu novu robusnu resync funkciju
    if let Some((header, _offset)) = Mp3Inspector::find_sync_frame(&quats, 0) {
    let vbr_opt = None::<VbrInfo>;
        let vbr_data = vbr_opt.unwrap_or(VbrInfo {
            is_vbr: false,
            total_frames: Some(0),
            total_bytes: Some(0),
            encoder_delay: None,
            padding: None,
        });

        (*out_result) = FfiAdvancedHeaderResult {
            tag: FfiHeaderKindTag::MpegFrame,
            version: match header.version {
                MpegVersion::Mpeg25 => FfiMpegVersion::Mpeg25,
                MpegVersion::Mpeg2 => FfiMpegVersion::Mpeg2,
                MpegVersion::Mpeg1 => FfiMpegVersion::Mpeg1,
            },
            layer: match header.layer {
                MpegLayer::Layer3 => FfiMpegLayer::Layer3,
                MpegLayer::Layer2 => FfiMpegLayer::Layer2,
                MpegLayer::Layer1 => FfiMpegLayer::Layer1,
            },
            bitrate_kbps: header.bitrate_kbps,
            sample_rate_hz: header.sample_rate_hz,
            padding: if header.padding { 1 } else { 0 },
            vbr: FfiVbrInfo {
                is_vbr: if vbr_data.is_vbr { 1 } else { 0 },
                total_frames: vbr_data.total_frames.unwrap_or(0),
                total_bytes: vbr_data.total_bytes.unwrap_or(0),
            },
        };

        QuatFFIError::Success
    } else {
        QuatFFIError::IncompleteHeader
    }
}}