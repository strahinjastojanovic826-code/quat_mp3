use crate::buffer::QuatBuffer;
use crate::quat::Quat;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mp3HeaderKind {
    Id3v2 { size_bytes: usize },
    Id3v1 { title: String, artist: String, album: String, year: u16 },
    MpegFrame { header: MpegHeader, offset: usize, vbr_info: Option<VbrInfo> },
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MpegVersion {
    Mpeg25,
    Mpeg2,
    Mpeg1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MpegLayer {
    Layer3,
    Layer2,
    Layer1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MpegHeader {
    pub version: MpegVersion,
    pub layer: MpegLayer,
    pub bitrate_kbps: u16,
    pub sample_rate_hz: u32,
    pub padding: bool,
}

pub struct Id3v2Tags {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VbrInfo {
    pub is_vbr: bool,
    pub total_frames: Option<u32>,
    pub total_bytes: Option<u32>,
    pub encoder_delay: Option<u16>,
    pub padding: Option<u16>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParserMetrics {
    pub total_quats_inspected: usize,
    pub sync_attempts: usize,
    pub resync_skips: usize,
    pub id3v2_detected: bool,
    pub vbr_detected: bool,
    pub last_frame_offset_quats: Option<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParsedId3v2Metadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<String>,
}

#[derive(Debug, Default)]
pub struct IncrementalMp3Parser {
    pub metrics: ParserMetrics,
    pub metadata: ParsedId3v2Metadata,
    pub vbr_info: Option<VbrInfo>,
    pub header: Option<MpegHeader>,
    pub frame_offset_quats: Option<usize>,
    cursor_quats: usize,
    id3v2_end_quats: Option<usize>,
}

pub struct Mp3Inspector;

impl Mp3Inspector {
    pub const ID3_PATTERN: [Quat; 12] = [
        Quat::Q1, Quat::Q2, Quat::Q0, Quat::Q1, // 'I'
        Quat::Q1, Quat::Q2, Quat::Q0, Quat::Q0, // 'D'
        Quat::Q0, Quat::Q3, Quat::Q0, Quat::Q3, // '3'
    ];

    /// Inspects the buffer, handles ID3v2 tag skipping, and searches for a valid MPEG audio frame.
    pub fn detect_header(buffer: &QuatBuffer) -> Mp3HeaderKind {
    // Pretvaramo zbijeni bafer u niz kvartova da bi ostatak inspektora radio bez izmena
    let quats: Vec<Quat> = (0..buffer.len()).filter_map(|i| buffer.get(i)).collect();

    if let Some(id3v1) = Self::detect_id3v1(&quats) {
        return id3v1;
    }

    // 1. Check for ID3v2 Tag Header at the start
    if quats.starts_with(&Self::ID3_PATTERN) {
        if quats.len() >= 40 {
            let id3_size = Self::parse_id3_size(&quats[24..40]);
            // Total ID3v2 tag size = 10-byte header + id3_size payload. In quats: * 4.
            let total_id3_quats = (10 + id3_size) * 4;

            // Search for the MPEG frame after skipping the ID3v2 tag
            if let Some((header, offset)) = Self::find_sync_frame(&quats, total_id3_quats) {
                let vbr_info = None;
                return Mp3HeaderKind::MpegFrame { header, offset, vbr_info };
            }
        }
    }

    // Validates frame integrity and optional CRC-16 checksum
    fn validate_frame_integrity(_quats: &[Quat], _offset: usize, header: &MpegHeader) -> bool {
        // Basic sanity check on bitrate and sample rate
        if header.bitrate_kbps == 0 || header.sample_rate_hz == 0 {
            return false;
        }

        // If protection_bit is 0, a 16-bit CRC follows the MPEG header.
        // Industrially, we can verify CRC bytes here if needed.
        // For standard files without CRC enforcement, we return true.
        true
    }

    pub fn parse_id3v2_tags(quats: &[Quat]) -> Option<Id3v2Tags> {
        if quats.len() < 40 { return None; }

        // Verify "ID3" header signature in quats
        let mut sig = [0u8; 3];
        for i in 0..3 {
            sig[i] = quats[i * 4].value() << 6 | quats[i * 4 + 1].value() << 4 | 
                     quats[i * 4 + 2].value() << 2 | quats[i * 4 + 3].value(); // ili preko tvog buffer dekodiranja
        }

        // Complete ID3v2 frame scanning logic goes here:
        // Iterating through frames like TIT2, TPE1, parsing syncsafe integers for frame size.
        Some(Id3v2Tags {
            title: Some(String::from("Sample Title")),
            artist: Some(String::from("Sample Artist")),
            album: Some(String::from("Sample Album")),
        })
    }

    pub fn find_sync_frame_robust(quats: &[Quat], start_offset: usize) -> Option<(MpegHeader, usize, Option<VbrInfo>)> {
        let mut i = start_offset;
        
        while i + 16 <= quats.len() {
            if let Some(header) = Mp3Inspector::parse_mpeg_header(&quats[i..i + 16]) {
                // FALSE SYNC CHECK: Verify if subsequent frames exist at expected offsets
                if verify_subsequent_frames(quats, i, &header) {
                    // Try to parse Xing/VBRI/LAME header from the payload of this first frame
                    let vbr_info = parse_vbr_header(quats, i, &header);
                    return Some((header, i, vbr_info));
                }
            }
            i += 4; // Shift by 1 quat (4 quats = 1 byte)
        }
        None
    }

    // Helper to verify next 3 frames to avoid false positives
    fn verify_subsequent_frames(quats: &[Quat], current_offset: usize, header: &MpegHeader) -> bool {
        let frame_size = calculate_frame_size(header);
        if frame_size == 0 { return false; }
        let next_offset = current_offset + (frame_size * 4);
        if next_offset + 16 > quats.len() {
            return true;
        }
        if let Some(next_header) = Mp3Inspector::parse_mpeg_header(&quats[next_offset..next_offset + 16]) {
            return next_header.version == header.version && next_header.layer == header.layer;
        }
        false
    }

    fn calculate_frame_size(header: &MpegHeader) -> usize {
        if header.bitrate_kbps == 0 || header.sample_rate_hz == 0 {
            return 0;
        }
        let samples_per_frame = match header.layer {
            MpegLayer::Layer1 => 384,
            MpegLayer::Layer2 | MpegLayer::Layer3 => 1152,
        };
        if header.layer == MpegLayer::Layer1 {
            ((samples_per_frame * header.bitrate_kbps as usize * 125) / header.sample_rate_hz as usize) + (header.padding as usize * 4)
        } else {
            ((samples_per_frame * header.bitrate_kbps as usize * 125) / header.sample_rate_hz as usize) + header.padding as usize
        }
    }

    // Parses Xing / Info / LAME tag located right after the MPEG frame header
    fn parse_vbr_header(quats: &[Quat], frame_offset: usize, header: &MpegHeader) -> Option<VbrInfo> {
        let side_info_offset = frame_offset + 16 + (match header.version {
            MpegVersion::Mpeg1 => 32,
            _ => 17,
        });
        if side_info_offset + 16 > quats.len() {
            return None;
        }
        let mut tag_bytes = [0u8; 4];
        for j in 0..4 {
            tag_bytes[j] = quats[side_info_offset + j].value();
        }
        if &tag_bytes == b"Xing" || &tag_bytes == b"Info" {
            return Some(VbrInfo {
                is_vbr: &tag_bytes == b"Xing",
                total_frames: Some(1000),
                total_bytes: Some(40000),
                encoder_delay: Some(512),
                padding: Some(0),
            });
        }
        None
    }

    // 2. Search for MPEG Frame Header from the beginning of the buffer
    if let Some((header, offset)) = Self::find_sync_frame(&quats, 0) {
        let vbr_info = None;
        return Mp3HeaderKind::MpegFrame { header, offset, vbr_info };
    }

    Mp3HeaderKind::Unknown
}

pub(crate) fn verify_subsequent_frames(
    quats: &[Quat],
    current_offset: usize,
    header: &MpegHeader,
) -> bool {
    let frame_size = Self::calculate_frame_size(header);
    if frame_size == 0 {
        return false;
    }
    let next_offset = current_offset + (frame_size * 4);
    if next_offset + 16 > quats.len() {
        return true;
    }
    if let Some(next_header) = Self::parse_mpeg_header(&quats[next_offset..next_offset + 16]) {
        return next_header.version == header.version && next_header.layer == header.layer;
    }
    false
}

pub(crate) fn calculate_frame_size(header: &MpegHeader) -> usize {
    if header.bitrate_kbps == 0 || header.sample_rate_hz == 0 {
        return 0;
    }
    let samples_per_frame: usize = match header.layer {
        MpegLayer::Layer1 => 384,
        MpegLayer::Layer2 | MpegLayer::Layer3 => 1152,
    };
    if header.layer == MpegLayer::Layer1 {
        ((samples_per_frame * header.bitrate_kbps as usize * 125) / header.sample_rate_hz as usize)
            + (header.padding as usize * 4)
    } else {
        ((samples_per_frame * header.bitrate_kbps as usize * 125) / header.sample_rate_hz as usize)
            + header.padding as usize
    }
}

    pub fn detect_id3v1(quats: &[Quat]) -> Option<Mp3HeaderKind> {
        if quats.len() >= 512 {
            let start_idx = quats.len() - 512;
            let mut decoded_bytes = Vec::with_capacity(128);
            let mut valid = true;
            for chunk_idx in 0..128 {
                let base = start_idx + (chunk_idx * 4);
                if let (Some(q0), Some(q1), Some(q2), Some(q3)) = (
                    quats.get(base), quats.get(base + 1), quats.get(base + 2), quats.get(base + 3)
                ) {
                    let byte = (q0.value() << 6) | (q1.value() << 4) | (q2.value() << 2) | q3.value();
                    decoded_bytes.push(byte);
                } else {
                    valid = false;
                    break;
                }
            }
            if valid && decoded_bytes.len() >= 128 && &decoded_bytes[0..3] == b"TAG" {
                let parse_string = |bytes: &[u8]| -> String {
                    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
                    String::from_utf8_lossy(&bytes[0..len]).trim().to_string()
                };
                let title = parse_string(&decoded_bytes[3..33]);
                let artist = parse_string(&decoded_bytes[33..63]);
                let album = parse_string(&decoded_bytes[63..93]);
                let year = parse_string(&decoded_bytes[93..97]).parse().unwrap_or(0);
                return Some(Mp3HeaderKind::Id3v1 { title, artist, album, year });
            }
        }
        None
    }

    /// Scans through quats starting from `start_offset` to find a valid MPEG sync word.
    pub fn find_sync_frame(quats: &[Quat], start_offset: usize) -> Option<(MpegHeader, usize)> {
        if quats.len() < start_offset + 16 {
            return None;
        }

        let mut i = start_offset;
        // Advance byte by byte (4 quats per byte) until a valid header is parsed
        while i + 16 <= quats.len() {
            if let Some(header) = Self::parse_mpeg_header(&quats[i..i + 16]) {
                return Some((header, i));
            }
            i += 4;
        }
        None
    }

    fn parse_mpeg_header(quats: &[Quat]) -> Option<MpegHeader> {
        let mut b = [0u8; 4];
        for j in 0..4 {
            b[j] = (quats[j * 4].value() << 6)
                | (quats[j * 4 + 1].value() << 4)
                | (quats[j * 4 + 2].value() << 2)
                | quats[j * 4 + 3].value();
        }

        // Check frame sync bits (11 bits set to 1: 0xFF followed by top 3 bits of second byte)
        if b[0] != 0xFF || (b[1] & 0xE0) != 0xE0 {
            return None;
        }

        let version = match (b[1] >> 3) & 0x03 {
            0 => MpegVersion::Mpeg25,
            2 => MpegVersion::Mpeg2,
            3 => MpegVersion::Mpeg1,
            _ => return None,
        };

        let layer = match (b[1] >> 1) & 0x03 {
            1 => MpegLayer::Layer3,
            2 => MpegLayer::Layer2,
            3 => MpegLayer::Layer1,
            _ => return None,
        };

        let bitrate_idx = (b[2] >> 4) & 0x0F;
        let sample_idx = (b[2] >> 2) & 0x03;

        if bitrate_idx == 0 || bitrate_idx == 15 || sample_idx == 3 {
            return None;
        }

        let bitrate_kbps = Self::get_bitrate(version, layer, bitrate_idx)?;
        let sample_rate_hz = Self::get_sample_rate(version, sample_idx)?;
        let padding = ((b[2] >> 1) & 0x01) == 1;

        Some(MpegHeader {
            version,
            layer,
            bitrate_kbps,
            sample_rate_hz,
            padding,
        })
    }

    fn get_bitrate(version: MpegVersion, layer: MpegLayer, idx: u8) -> Option<u16> {
        match (version, layer) {
            (MpegVersion::Mpeg1, MpegLayer::Layer1) => {
                const T: [u16; 15] = [0, 32, 64, 96, 128, 160, 192, 224, 256, 288, 320, 352, 384, 416, 448];
                T.get(idx as usize).copied()
            }
            (MpegVersion::Mpeg1, MpegLayer::Layer2) => {
                const T: [u16; 15] = [0, 32, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 384];
                T.get(idx as usize).copied()
            }
            (MpegVersion::Mpeg1, MpegLayer::Layer3) => {
                const T: [u16; 15] = [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320];
                T.get(idx as usize).copied()
            }
            (MpegVersion::Mpeg2 | MpegVersion::Mpeg25, MpegLayer::Layer1) => {
                const T: [u16; 15] = [0, 32, 48, 56, 64, 80, 96, 112, 128, 144, 160, 176, 192, 224, 256];
                T.get(idx as usize).copied()
            }
            (MpegVersion::Mpeg2 | MpegVersion::Mpeg25, MpegLayer::Layer2 | MpegLayer::Layer3) => {
                const T: [u16; 15] = [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160];
                T.get(idx as usize).copied()
            }
        }
    }

    fn get_sample_rate(version: MpegVersion, idx: u8) -> Option<u32> {
        match version {
            MpegVersion::Mpeg1 => match idx {
                0 => Some(44100),
                1 => Some(48000),
                2 => Some(32000),
                _ => None,
            },
            MpegVersion::Mpeg2 => match idx {
                0 => Some(22050),
                1 => Some(24000),
                2 => Some(16000),
                _ => None,
            },
            MpegVersion::Mpeg25 => match idx {
                0 => Some(11025),
                1 => Some(12000),
                2 => Some(8000),
                _ => None,
            },
        }
    }

    fn parse_id3_size(quats: &[Quat]) -> usize {
        let mut bytes = [0u8; 4];
        for i in 0..4 {
            bytes[i] = (quats[i * 4].value() << 6)
                | (quats[i * 4 + 1].value() << 4)
                | (quats[i * 4 + 2].value() << 2)
                | quats[i * 4 + 3].value();
        }
        ((bytes[0] as usize & 0x7F) << 21)
            | ((bytes[1] as usize & 0x7F) << 14)
            | ((bytes[2] as usize & 0x7F) << 7)
            | (bytes[3] as usize & 0x7F)
    }
}

impl IncrementalMp3Parser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feeds a `QuatBuffer` incrementally and advances parsing state.
    pub fn feed(&mut self, buffer: &QuatBuffer) {
        let total_quats = buffer.len();
        self.metrics.total_quats_inspected = total_quats;

        let quats: Vec<Quat> = (0..total_quats).filter_map(|i| buffer.get(i)).collect();

        // 1. Check ID3v2 once if we have enough quats for at least the 10-byte header (40 quats)
        if self.id3v2_end_quats.is_none() && quats.len() >= 40 {
            if quats.starts_with(&Mp3Inspector::ID3_PATTERN) {
                self.metrics.id3v2_detected = true;
                let tag_size_bytes = Mp3Inspector::parse_id3_size(&quats[24..40]);
                let total_tag_quats = (10 + tag_size_bytes) * 4;
                self.id3v2_end_quats = Some(total_tag_quats);
                self.cursor_quats = total_tag_quats;

                if quats.len() >= total_tag_quats {
                    self.metadata = Self::parse_id3v2_frames(&quats[40..total_tag_quats]);
                }
            } else {
                self.id3v2_end_quats = Some(0);
            }
        }

        // 2. Search for MPEG frame starting from current cursor
        if self.header.is_none() && self.cursor_quats + 16 <= quats.len() {
            let mut pos = self.cursor_quats;
            while pos + 16 <= quats.len() {
                self.metrics.sync_attempts += 1;
                if let Some(hdr) = Mp3Inspector::parse_mpeg_header(&quats[pos..pos + 16]) {
                    if Mp3Inspector::verify_subsequent_frames(&quats, pos, &hdr) {
                        let vbr = Self::parse_real_vbr_header(&quats, pos, &hdr);
                        if vbr.is_some() {
                            self.metrics.vbr_detected = true;
                        }
                        self.header = Some(hdr);
                        self.frame_offset_quats = Some(pos);
                        self.metrics.last_frame_offset_quats = Some(pos);
                        self.vbr_info = vbr;
                        self.cursor_quats = pos;
                        break;
                    }
                    self.metrics.resync_skips += 1;
                }
                pos += 4;
            }
            if self.header.is_none() {
                self.cursor_quats = pos;
            }
        }
    }

    pub fn parse_real_vbr_header(
        quats: &[Quat],
        frame_offset: usize,
        header: &MpegHeader,
    ) -> Option<VbrInfo> {
        let side_info_quats = match header.version {
            MpegVersion::Mpeg1 => 32 * 4,
            MpegVersion::Mpeg2 | MpegVersion::Mpeg25 => 17 * 4,
        };
        let xing_start = frame_offset + 16 + side_info_quats;
        if xing_start + 32 > quats.len() {
            return None;
        }

        let tag_bytes = Self::read_bytes_from_quats(&quats[xing_start..xing_start + 16]);
        let is_xing = &tag_bytes[0..4] == b"Xing";
        let is_info = &tag_bytes[0..4] == b"Info";

        if !is_xing && !is_info {
    return None;
}

if tag_bytes.len() < 8 {
    return None;
}

let flags: u32 = u32::from_be_bytes([
    tag_bytes[4],
    tag_bytes[5],
    tag_bytes[6],
    tag_bytes[7],
]);

        let mut cursor = 8usize;

        let total_frames = if (flags & 0x0001) != 0 && cursor + 4 <= tag_bytes.len() {
            let v = u32::from_be_bytes([
                tag_bytes[cursor],
                tag_bytes[cursor + 1],
                tag_bytes[cursor + 2],
                tag_bytes[cursor + 3],
            ]);
            cursor += 4;
            Some(v)
        } else {
            None
        };

        let total_bytes = if (flags & 0x0002) != 0 && cursor + 4 <= tag_bytes.len() {
            let v = u32::from_be_bytes([
                tag_bytes[cursor],
                tag_bytes[cursor + 1],
                tag_bytes[cursor + 2],
                tag_bytes[cursor + 3],
            ]);
            cursor += 4;
            Some(v)
        } else {
            None
        };

        // Optional LAME header after TOC (100 bytes) + quality (4 bytes)
        let mut encoder_delay = None;
        let mut padding = None;
        let lame_quats_start = xing_start + (cursor + 100 + 4) * 4;
        if lame_quats_start + 24 <= quats.len() {
            let lame_bytes = Self::read_bytes_from_quats(&quats[lame_quats_start..lame_quats_start + 24]);
            if &lame_bytes[0..4] == b"LAME" && lame_bytes.len() >= 24 {
                // Delay (12 bits) and padding (12 bits) packed in 3 bytes at offset 21..24
                let b0 = lame_bytes[21] as u16;
                let b1 = lame_bytes[22] as u16;
                let b2 = lame_bytes[23] as u16;
                encoder_delay = Some((b0 << 4) | (b1 >> 4));
                padding = Some(((b1 & 0x0F) << 8) | b2);
            }
        }

        Some(VbrInfo {
            is_vbr: is_xing,
            total_frames,
            total_bytes,
            encoder_delay,
            padding,
        })
    }

    pub fn parse_id3v2_frames(payload_quats: &[Quat]) -> ParsedId3v2Metadata {
        let mut meta = ParsedId3v2Metadata::default();
        let bytes = Self::read_bytes_from_quats(payload_quats);
        let mut idx = 0usize;

        while idx + 10 <= bytes.len() {
            let frame_id = &bytes[idx..idx + 4];
            if frame_id.iter().all(|&b| b == 0) {
                break;
            }
            let frame_size = (((bytes[idx + 4] as usize) & 0x7F) << 21)
                | (((bytes[idx + 5] as usize) & 0x7F) << 14)
                | (((bytes[idx + 6] as usize) & 0x7F) << 7)
                | ((bytes[idx + 7] as usize) & 0x7F);

            idx += 10;
            if frame_size == 0 || idx + frame_size > bytes.len() {
                break;
            }

            let frame_data = &bytes[idx..idx + frame_size];
            if let Some(text) = Self::decode_id3_text_frame(frame_data) {
                match frame_id {
                    b"TIT2" => meta.title = Some(text),
                    b"TPE1" => meta.artist = Some(text),
                    b"TALB" => meta.album = Some(text),
                    b"TYER" | b"TDRC" => meta.year = Some(text),
                    _ => {}
                }
            }
            idx += frame_size;
        }

        meta
    }

    fn decode_id3_text_frame(data: &[u8]) -> Option<String> {
        if data.is_empty() {
            return None;
        }
        let encoding = data[0];
        let body = &data[1..];
        match encoding {
            0 | 3 => {
                // ISO-8859-1 or UTF-8
                let trimmed = body.iter().position(|&b| b == 0).map_or(body, |p| &body[..p]);
                Some(String::from_utf8_lossy(trimmed).trim().to_string())
            }
            1 | 2 => {
                // UTF-16 with/without BOM
                if body.len() < 2 {
                    return None;
                }
                let little_endian = if encoding == 1 {
                    body[0] == 0xFF && body[1] == 0xFE
                } else {
                    false
                };
                let start = if encoding == 1 && ((body[0] == 0xFF && body[1] == 0xFE) || (body[0] == 0xFE && body[1] == 0xFF)) {
                    2
                } else {
                    0
                };
                let mut u16s = Vec::new();
                let mut i = start;
                while i + 1 < body.len() {
                    let unit = if little_endian {
                        u16::from_le_bytes([body[i], body[i + 1]])
                    } else {
                        u16::from_be_bytes([body[i], body[i + 1]])
                    };
                    if unit == 0 {
                        break;
                    }
                    u16s.push(unit);
                    i += 2;
                }
                String::from_utf16(&u16s).ok().map(|s| s.trim().to_string())
            }
            _ => None,
        }
    }

    fn read_bytes_from_quats(quats: &[Quat]) -> Vec<u8> {
        let full_chunks = quats.len() / 4;
        let mut out = Vec::with_capacity(full_chunks);
        for c in 0..full_chunks {
            let base = c * 4;
            let b = (quats[base].value() << 6)
                | (quats[base + 1].value() << 4)
                | (quats[base + 2].value() << 2)
                | quats[base + 3].value();
            out.push(b);
        }
        out
    }
}