//! Bounded Synergy clipboard containers and paced, serialized chunk transfers.
use crate::{notify, Action, ConnId};
use openergy_platform::{ClipboardData, ClipboardFormat, FormatKind};
use openergy_protocol::{Chunk, ClipboardId};
use std::collections::VecDeque;
use std::sync::Arc;

pub const CHUNK_BYTES: usize = 32 * 1024;
pub const HARD_CAP: usize = 64 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardError {
    TooLarge,
    Malformed,
    InvalidUtf8,
}

pub fn marshal(data: &ClipboardData, cap: usize) -> Result<Vec<u8>, ClipboardError> {
    let mut formats = data
        .formats
        .iter()
        .filter_map(|f| f.kind.wire_id().map(|id| (id, &f.bytes)))
        .collect::<Vec<_>>();
    formats.sort_by_key(|(id, _)| *id);
    formats.dedup_by_key(|(id, _)| *id);
    let total = formats
        .iter()
        .try_fold(4usize, |n, (_, b)| n.checked_add(8)?.checked_add(b.len()))
        .ok_or(ClipboardError::TooLarge)?;
    if total > cap.min(HARD_CAP) {
        return Err(ClipboardError::TooLarge);
    }
    let mut bytes = Vec::with_capacity(total);
    bytes.extend_from_slice(&(formats.len() as u32).to_be_bytes());
    for (id, b) in formats {
        if id < 2 && std::str::from_utf8(b).is_err() {
            return Err(ClipboardError::InvalidUtf8);
        }
        bytes.extend_from_slice(&id.to_be_bytes());
        bytes.extend_from_slice(&(b.len() as u32).to_be_bytes());
        bytes.extend_from_slice(b);
    }
    Ok(bytes)
}
pub fn unmarshal(bytes: &[u8]) -> Result<ClipboardData, ClipboardError> {
    fn take_u32(bytes: &[u8], p: &mut usize) -> Result<u32, ClipboardError> {
        let end = p.checked_add(4).ok_or(ClipboardError::Malformed)?;
        let b: [u8; 4] = bytes
            .get(*p..end)
            .ok_or(ClipboardError::Malformed)?
            .try_into()
            .map_err(|_| ClipboardError::Malformed)?;
        *p = end;
        Ok(u32::from_be_bytes(b))
    }
    let mut p = 0;
    let count = take_u32(bytes, &mut p)?;
    if count as usize > bytes.len().saturating_sub(4) / 8 {
        return Err(ClipboardError::Malformed);
    }
    let mut data = ClipboardData::default();
    for _ in 0..count {
        let id = take_u32(bytes, &mut p)?;
        let len = take_u32(bytes, &mut p)? as usize;
        let end = p.checked_add(len).ok_or(ClipboardError::Malformed)?;
        let b = bytes.get(p..end).ok_or(ClipboardError::Malformed)?;
        p = end;
        if let Some(kind) = FormatKind::from_wire_id(id) {
            if id < 2 && std::str::from_utf8(b).is_err() {
                return Err(ClipboardError::InvalidUtf8);
            }
            data.formats.push(ClipboardFormat {
                kind,
                bytes: b.to_vec(),
            });
        }
    }
    if p != bytes.len() {
        return Err(ClipboardError::Malformed);
    }
    Ok(data)
}

struct Incoming {
    id: ClipboardId,
    sequence: u32,
    total: usize,
    bytes: Vec<u8>,
    since: u64,
}
struct Outgoing {
    id: ClipboardId,
    sequence: u32,
    bytes: Arc<Vec<u8>>,
    offset: usize,
    started: bool,
}
#[derive(Default)]
pub(crate) struct Transfers {
    incoming: Option<Incoming>,
    outgoing: VecDeque<Outgoing>,
}
impl Transfers {
    pub fn cancel(&mut self, id: ClipboardId) {
        if self.incoming.as_ref().is_some_and(|i| i.id == id) {
            self.incoming = None;
        }
        // Already-started wire transfers must finish before a newer transfer starts.
        self.outgoing.retain(|o| o.id != id || o.started);
    }
    pub fn enqueue(&mut self, id: ClipboardId, sequence: u32, bytes: Arc<Vec<u8>>) {
        self.outgoing.retain(|o| o.id != id || o.started);
        if self.outgoing.len() < 3 {
            self.outgoing.push_back(Outgoing {
                id,
                sequence,
                bytes,
                offset: 0,
                started: false,
            });
        }
    }
    pub fn tick(&mut self, conn: ConnId, now: u64, out: &mut Vec<Action>) {
        if self
            .incoming
            .as_ref()
            .is_some_and(|i| now.saturating_sub(i.since) >= 10_000)
        {
            self.incoming = None;
        }
        let Some(o) = self.outgoing.front_mut() else { return };
        let chunk = if !o.started {
            o.started = true;
            Chunk::Start {
                total_len: o.bytes.len() as u32,
            }
        } else if o.offset < o.bytes.len() {
            let end = (o.offset + CHUNK_BYTES).min(o.bytes.len());
            let chunk = Chunk::Data(o.bytes.get(o.offset..end).unwrap_or_default().to_vec());
            o.offset = end;
            chunk
        } else {
            Chunk::End
        };
        let done = matches!(chunk, Chunk::End);
        out.push(Action::SendClipboardChunk {
            conn,
            id: o.id,
            sequence: o.sequence,
            chunk,
        });
        if done {
            self.outgoing.pop_front();
        }
    }
    pub fn receive(
        &mut self,
        id: ClipboardId,
        sequence: u32,
        chunk: Chunk,
        cap: usize,
        now: u64,
    ) -> Result<Option<ClipboardData>, ClipboardError> {
        if id.0 > 1 {
            return Ok(None);
        }
        match chunk {
            Chunk::Start { total_len } => {
                self.incoming = None;
                if total_len as usize > cap.min(HARD_CAP) {
                    return Err(ClipboardError::TooLarge);
                }
                self.incoming = Some(Incoming {
                    id,
                    sequence,
                    total: total_len as usize,
                    bytes: Vec::new(),
                    since: now,
                });
            }
            Chunk::Data(bytes) => {
                let Some(i) = self.incoming.as_mut() else {
                    return Ok(None);
                };
                if i.id != id
                    || i.sequence != sequence
                    || bytes.len() > i.total.saturating_sub(i.bytes.len())
                    || now.saturating_sub(i.since) >= 10_000
                {
                    self.incoming = None;
                    return Err(ClipboardError::Malformed);
                }
                i.bytes.extend(bytes);
            }
            Chunk::End => {
                let Some(i) = self.incoming.take() else { return Ok(None) };
                if i.id != id
                    || i.sequence != sequence
                    || i.bytes.len() != i.total
                    || now.saturating_sub(i.since) >= 10_000
                {
                    return Err(ClipboardError::Malformed);
                }
                return unmarshal(&i.bytes).map(Some);
            }
        }
        Ok(None)
    }
}
pub(crate) fn blocked(bytes: usize, limit: usize, peer: &str, out: &mut Vec<Action>) {
    log::warn!("clipboard blocked: bytes={bytes}, limit={limit}, peer={peer}");
    notify(
        "clipboard.blocked",
        serde_json::json!({"bytes":bytes,"limit":limit,"peer":peer,"unix_ms":null}),
        out,
    );
}
pub(crate) fn transferred(data: &ClipboardData, direction: &str, out: &mut Vec<Action>) {
    notify(
        "clipboard.transfer",
        serde_json::json!({"direction":direction,"bytes":data.total_bytes(),
        "formats":data.formats.iter().map(|f|f.kind.as_str()).collect::<Vec<_>>(),"unix_ms":null}),
        out,
    );
}
