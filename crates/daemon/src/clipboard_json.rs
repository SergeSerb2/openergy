//! Compact IPC representation keeps large virtual fixtures within the local frame cap.
use openergy_ipc::IpcError;
use openergy_platform::virtual_backend::ClipboardSnapshot;
use openergy_platform::{ClipboardData, ClipboardFormat, FormatKind};
use serde_json::{json, Value};
const CAP: usize = 64 * 1024 * 1024;
const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
fn invalid() -> IpcError {
    IpcError::new("invalid_params", "invalid or oversized clipboard encoding")
}
pub fn decode(value: &Value) -> Result<ClipboardData, IpcError> {
    let items = value.get("formats").and_then(Value::as_array).ok_or_else(invalid)?;
    if items.len() > 4 {
        return Err(invalid());
    }
    let mut formats = Vec::new();
    let mut total = 0usize;
    for item in items {
        let kind: FormatKind =
            serde_json::from_value(item.get("kind").cloned().ok_or_else(invalid)?).map_err(|_| invalid())?;
        if item.get("bytes").is_some() == item.get("bytes_base64").is_some() {
            return Err(invalid());
        }
        let bytes = if let Some(text) = item.get("bytes_base64").and_then(Value::as_str) {
            unbase64(text, CAP.saturating_sub(total))?
        } else {
            serde_json::from_value::<Vec<u8>>(item.get("bytes").cloned().ok_or_else(invalid)?).map_err(|_| invalid())?
        };
        total = total.checked_add(bytes.len()).ok_or_else(invalid)?;
        if total > CAP {
            return Err(invalid());
        }
        formats.push(ClipboardFormat { kind, bytes });
    }
    Ok(ClipboardData { formats })
}
pub fn encode(snapshot: ClipboardSnapshot) -> Value {
    json!({"stamp":snapshot.stamp,"own_write":snapshot.own_write,"last_own_write_stamp":snapshot.last_own_write_stamp,
        "data":{"formats":snapshot.data.formats.iter().map(|f|json!({"kind":f.kind,"bytes_base64":base64(&f.bytes)})).collect::<Vec<_>>()}})
}
fn base64(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let a = chunk.first().copied().unwrap_or(0);
        let b = chunk.get(1).copied().unwrap_or(0);
        let c = chunk.get(2).copied().unwrap_or(0);
        for index in [a >> 2, ((a & 3) << 4) | (b >> 4)] {
            output.push(TABLE.get(index as usize).copied().unwrap_or(b'?') as char);
        }
        output.push(if chunk.len() > 1 {
            TABLE
                .get((((b & 15) << 2) | (c >> 6)) as usize)
                .copied()
                .unwrap_or(b'?') as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            TABLE.get((c & 63) as usize).copied().unwrap_or(b'?') as char
        } else {
            '='
        });
    }
    output
}
fn digit(b: u8) -> Option<u8> {
    match b {
        b'A'..=b'Z' => Some(b - b'A'),
        b'a'..=b'z' => Some(b - b'a' + 26),
        b'0'..=b'9' => Some(b - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}
fn unbase64(text: &str, cap: usize) -> Result<Vec<u8>, IpcError> {
    if text.len() % 4 != 0 || text.len() / 4 * 3 > cap.saturating_add(2) {
        return Err(invalid());
    }
    let mut out = Vec::with_capacity((text.len() / 4 * 3).min(cap));
    for (index, chunk) in text.as_bytes().chunks_exact(4).enumerate() {
        let a = digit(*chunk.first().ok_or_else(invalid)?).ok_or_else(invalid)?;
        let b = digit(*chunk.get(1).ok_or_else(invalid)?).ok_or_else(invalid)?;
        let cb = *chunk.get(2).ok_or_else(invalid)?;
        let db = *chunk.get(3).ok_or_else(invalid)?;
        let cp = cb == b'=';
        let dp = db == b'=';
        if (cp && !dp) || ((cp || dp) && index + 1 != text.len() / 4) {
            return Err(invalid());
        }
        let c = if cp { 0 } else { digit(cb).ok_or_else(invalid)? };
        let d = if dp { 0 } else { digit(db).ok_or_else(invalid)? };
        if (cp && b & 15 != 0) || (dp && !cp && c & 3 != 0) {
            return Err(invalid());
        }
        out.push((a << 2) | (b >> 4));
        if !cp {
            out.push((b << 4) | (c >> 2));
        }
        if !dp {
            out.push((c << 6) | d);
        }
        if out.len() > cap {
            return Err(invalid());
        }
    }
    Ok(out)
}
