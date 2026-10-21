//! Wire container primitives. Transfer policy, chunk scheduling, reassembly,
//! timeouts and the configured total cap belong to `session::clipboard` (§2.6).
use crate::{codec::Reader, DecodeError, EncodeError};

/// Append a container in ascending format order (0 text, 1 HTML, 2 DIB).
/// No packet/string cap applies to an assembled container: session bounds it
/// before marshalling and before collecting any received DCLP data.
pub fn marshal_clipboard(formats: &[(u32, &[u8])], out: &mut Vec<u8>) -> Result<(), EncodeError> {
    let mut seen = 0u8;
    let mut len = 4usize;
    for (id, bytes) in formats {
        if *id > 2 {
            return Err(EncodeError::BadValue("unknown clipboard format"));
        }
        let bit = 1u8 << id;
        if seen & bit != 0 {
            return Err(EncodeError::BadValue("duplicate clipboard format"));
        }
        seen |= bit;
        if *id < 2 && std::str::from_utf8(bytes).is_err() {
            return Err(EncodeError::BadValue("clipboard UTF-8"));
        }
        len = len
            .checked_add(8)
            .and_then(|n| n.checked_add(bytes.len()))
            .filter(|n| *n <= u32::MAX as usize)
            .ok_or(EncodeError::BadValue("clipboard container size"))?;
    }
    out.try_reserve(len)
        .map_err(|_| EncodeError::BadValue("allocation failed"))?;
    out.extend_from_slice(&(formats.len() as u32).to_be_bytes());
    for id in 0..=2u32 {
        if let Some((_, bytes)) = formats.iter().find(|(format, _)| *format == id) {
            out.extend_from_slice(&id.to_be_bytes());
            out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
            out.extend_from_slice(bytes);
        }
    }
    Ok(())
}

/// Read a complete assembled container. Unknown format IDs are skipped with
/// checked lengths. Truncation or invalid UTF-8 rejects the whole update.
pub fn unmarshal_clipboard(bytes: &[u8]) -> Result<Vec<(u32, Vec<u8>)>, DecodeError> {
    let mut r = Reader::new(bytes);
    let count = r.u32()?;
    if count as usize > r.remaining() / 8 {
        return Err(DecodeError::Truncated);
    }
    let mut formats = Vec::new();
    let mut seen = 0u8;
    for _ in 0..count {
        let id = r.u32()?;
        let len = r.u32()?;
        let bytes = r.take(len as usize)?;
        if id <= 2 {
            let bit = 1u8 << id;
            if seen & bit != 0 {
                return Err(DecodeError::BadValue("duplicate clipboard format"));
            }
            seen |= bit;
            if id < 2 {
                std::str::from_utf8(bytes).map_err(|_| DecodeError::BadUtf8)?;
            }
            formats.push((id, bytes.to_vec()));
        }
    }
    r.end()?;
    Ok(formats)
}
