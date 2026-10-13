use std::os::raw::{c_uchar, c_uint, c_ushort};
use crate::buffer::QuatBuffer;
use crate::inspector::{Mp3Inspector, Mp3HeaderKind, MpegVersion, MpegLayer};
use crate::QuatError;

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
pub enum FfiMpegVersion {
    Mpeg25 = 0,
    Mpeg2 = 1,
    Mpeg1 = 2,
}

#[repr(C)]
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
            Mp3HeaderKind::MpegFrame { header, offset } => {
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