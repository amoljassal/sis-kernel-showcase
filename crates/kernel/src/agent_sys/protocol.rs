//! AgentSys wire protocol definitions

/// Agent identifier (16-bit, embedded in token upper bits)
pub type AgentId = u32;

/// Resource reference (file, doc, audio track)
pub type ResourceRef = u32;

/// TLV tag types
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Tag {
    Path = 1,
    Offset = 2,
    Length = 3,
    Data = 4,
    Name = 5,
    Kind = 6,
    Operations = 7,
    DocRef = 8,
    TrackRef = 9,
    Level = 10,
    Duration = 11,
}

/// Parse path from TLV payload
pub fn parse_path(payload: &[u8]) -> Result<&str, &'static str> {
    if payload.len() < 2 {
        return Err("Payload too short for path");
    }

    let path_len = u16::from_le_bytes([payload[0], payload[1]]) as usize;
    if payload.len() < 2 + path_len {
        return Err("Path length exceeds payload");
    }

    let path_bytes = &payload[2..2 + path_len];
    core::str::from_utf8(path_bytes).map_err(|_| "Invalid UTF-8 in path")
}

/// Parse offset + length from TLV payload (for read/write operations)
pub fn parse_offset_len(payload: &[u8], path_len: usize) -> Result<(u64, u32), &'static str> {
    let offset_start = 2 + path_len;
    if payload.len() < offset_start + 12 {
        return Err("Payload too short for offset+len");
    }

    let offset = u64::from_le_bytes([
        payload[offset_start],
        payload[offset_start + 1],
        payload[offset_start + 2],
        payload[offset_start + 3],
        payload[offset_start + 4],
        payload[offset_start + 5],
        payload[offset_start + 6],
        payload[offset_start + 7],
    ]);

    let len = u32::from_le_bytes([
        payload[offset_start + 8],
        payload[offset_start + 9],
        payload[offset_start + 10],
        payload[offset_start + 11],
    ]);

    Ok((offset, len))
}
