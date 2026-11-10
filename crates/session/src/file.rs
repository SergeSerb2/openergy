//! Openergy explicit file transfer state (design §4, §6, §7).
//!
//! Session holds metadata only, never file bytes beyond one chunk in
//! flight inside an action. Disk work lives in the daemon file worker.
use crate::{notify, Action, ConnId};
use openergy_config::Config;
use openergy_protocol::{FileDataChunk, Message};
use std::collections::BTreeSet;

/// Hard cap for one file, enforced in session like clipboard `HARD_CAP`.
pub const FILE_HARD_CAP: u64 = openergy_config::MAX_FILE_BYTES;
// `OFTR` has 13 payload header bytes plus the four-byte frame prefix.
pub const FILE_CHUNK_BYTES: usize = 32 * 1024 - 17;

pub const CAP_EXPLICIT: u32 = 1;
pub const CAP_DRAG: u32 = 2;
pub const CAP_CLASSIC: u32 = 4;

/// Wire bytes of one `OFTR` frame, for the daemon's bulk gate.
pub fn file_chunk_wire_bytes(chunk: &FileDataChunk) -> usize {
    chunk.payload_len().saturating_add(17)
}

/// One shared reason enum for `OFAN`, `OFTR` cancel, and IPC mirrors.
/// `PeerChanged` is daemon-local only: it never travels on the wire.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Reason {
    #[default]
    None,
    UserRejected,
    TooLarge,
    Disabled,
    BadName,
    Busy,
    NoSpace,
    HashMismatch,
    Cancelled,
    Stalled,
    IoError,
    Unsupported,
    TlsRequired,
    Timeout,
    PeerChanged,
    NameConflict,
}
impl Reason {
    pub fn code(self) -> u8 {
        match self {
            Self::None => 0,
            Self::UserRejected => 1,
            Self::TooLarge => 2,
            Self::Disabled => 3,
            Self::BadName => 4,
            Self::Busy => 5,
            Self::NoSpace => 6,
            Self::HashMismatch => 7,
            Self::Cancelled => 8,
            Self::Stalled => 9,
            Self::IoError => 10,
            Self::Unsupported => 11,
            Self::TlsRequired => 12,
            Self::Timeout => 13,
            // Local only. This code must never be sent in `OFAN`/cancel.
            Self::PeerChanged => 14,
            Self::NameConflict => 15,
        }
    }
    pub fn from_code(code: u8) -> Self {
        match code {
            1 => Self::UserRejected,
            2 => Self::TooLarge,
            3 => Self::Disabled,
            4 => Self::BadName,
            5 => Self::Busy,
            6 => Self::NoSpace,
            7 => Self::HashMismatch,
            8 => Self::Cancelled,
            9 => Self::Stalled,
            10 => Self::IoError,
            11 => Self::Unsupported,
            12 => Self::TlsRequired,
            13 => Self::Timeout,
            15 => Self::NameConflict,
            _ => Self::IoError,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::UserRejected => "user_rejected",
            Self::TooLarge => "too_large",
            Self::Disabled => "disabled",
            Self::BadName => "bad_name",
            Self::Busy => "busy",
            Self::NoSpace => "no_space",
            Self::HashMismatch => "hash_mismatch",
            Self::Cancelled => "cancelled",
            Self::Stalled => "stalled",
            Self::IoError => "io_error",
            Self::Unsupported => "unsupported",
            Self::TlsRequired => "tls_required",
            Self::Timeout => "timeout",
            Self::PeerChanged => "peer_changed",
            Self::NameConflict => "name_conflict",
        }
    }
}

/// Validate an offered base name (design §7.2). Returns the stored name,
/// with trailing dots and spaces trimmed for Windows safety.
pub fn validate_name(name: &str) -> Result<String, Reason> {
    let bad = Err(Reason::BadName);
    if name.is_empty() || name.len() > 255 {
        return bad;
    }
    if name
        .chars()
        .any(|c| c == '/' || c == '\\' || c == ':' || c.is_control())
    {
        return bad;
    }
    let trimmed = name.trim_end_matches(['.', ' ']);
    if trimmed.is_empty() || trimmed == "." || trimmed == ".." {
        return bad;
    }
    if trimmed == ".parts" || trimmed.starts_with(".part-") {
        return bad;
    }
    let stem = trimmed.split('.').next().unwrap_or_default();
    const RESERVED: [&str; 22] = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1",
        "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if RESERVED.iter().any(|r| stem.eq_ignore_ascii_case(r)) {
        return bad;
    }
    Ok(trimmed.to_owned())
}

/// This daemon's receive-side capability mask for `OFIL`/`OFCP`.
pub fn local_cap_mask(cfg: &Config) -> u32 {
    if !cfg.file.enabled {
        return 0;
    }
    let mut mask = 0;
    if cfg.file.receive {
        mask |= CAP_EXPLICIT;
    }
    if cfg.file.drag_receive {
        mask |= CAP_DRAG;
    }
    if cfg.file.classic_send {
        mask |= CAP_CLASSIC;
    }
    mask
}

/// This daemon's receive cap in KiB for `OFSZ`/`OFCP`.
pub fn local_max_kib(cfg: &Config) -> u32 {
    (cfg.file.max_bytes.min(FILE_HARD_CAP) / 1024).min(u32::MAX as u64) as u32
}

pub fn inbound_expiry_ms(heartbeat_ms: u64) -> u64 {
    30_000u64.max(heartbeat_ms.saturating_mul(3))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OutState {
    AwaitingAnswer,
    Sending,
    AwaitingStored,
}
#[derive(Clone)]
pub(crate) struct OutTransfer {
    pub xfer: u64,
    pub transfer: u32,
    pub kind: u8,
    pub total_len: u64,
    state: OutState,
    since_ms: u64,
    /// Next byte offset to request from the worker.
    request_offset: u64,
    /// Bytes emitted on the wire so far.
    sent: u64,
    credit: u32,
    outstanding: bool,
    requested_len: usize,
    held: Option<Vec<u8>>,
    started: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InState {
    AwaitingDecision,
    Receiving,
    Finishing,
}
#[derive(Clone)]
pub(crate) struct InTransfer {
    pub xfer: u64,
    pub transfer: u32,
    pub kind: u8,
    pub total_len: u64,
    state: InState,
    since_ms: u64,
    last_data_ms: u64,
    staged: u64,
    started: bool,
}
#[derive(Default, Clone)]
pub(crate) struct FileTransfers {
    pub out: Option<OutTransfer>,
    pub inn: Option<InTransfer>,
    /// Peer capability from `OFCP` (server side). Client side reads `DSOP`.
    pub cap_mask: Option<u32>,
    pub cap_max_kib: Option<u32>,
    dead_in: BTreeSet<u32>,
    counted_unknown: BTreeSet<u32>,
}
impl FileTransfers {
    pub fn out_xfer(&self) -> Option<u64> {
        self.out.as_ref().map(|o| o.xfer)
    }
    pub fn in_xfer(&self) -> Option<u64> {
        self.inn.as_ref().map(|i| i.xfer)
    }
    pub fn has_xfer(&self, xfer: u64) -> bool {
        self.out_xfer() == Some(xfer) || self.in_xfer() == Some(xfer)
    }
    fn note_unknown(&mut self, transfer: u32) {
        if self.counted_unknown.len() < 64 && self.counted_unknown.insert(transfer) {
            log::warn!("file data for unknown transfer discarded");
        }
    }
    fn mark_dead_in(&mut self, transfer: u32) {
        if self.dead_in.len() < 256 {
            self.dead_in.insert(transfer);
        }
        self.note_unknown(transfer);
    }
}

fn send_msg(conn: ConnId, msg: Message, out: &mut Vec<Action>) {
    out.push(Action::Send { conn, msg });
}
fn answer(conn: ConnId, transfer: u32, accept: u8, reason: Reason, resume_offset: u64, out: &mut Vec<Action>) {
    send_msg(
        conn,
        Message::FileAnswer {
            transfer,
            accept,
            reason: reason.code(),
            resume_offset,
        },
        out,
    );
}
fn cancel_frame(conn: ConnId, transfer: u32, reason: Reason, out: &mut Vec<Action>) {
    // Cancel travels at control priority through the plain Send route.
    send_msg(
        conn,
        Message::FileData {
            transfer,
            chunk: FileDataChunk::Cancel { reason: reason.code() },
        },
        out,
    );
}
pub(crate) fn failed(xfer: u64, peer: &str, direction: &str, reason: Reason, out: &mut Vec<Action>) {
    log::warn!(
        "file transfer failed: xfer={xfer} direction={direction} reason={}",
        reason.as_str()
    );
    notify(
        "file.failed",
        serde_json::json!({"xfer":xfer,"peer":peer,"direction":direction,"reason":reason.as_str(),"unix_ms":null}),
        out,
    );
}
pub(crate) fn blocked(peer: &str, bytes: u64, limit: u64, reason: Reason, out: &mut Vec<Action>) {
    log::warn!("file blocked: bytes={bytes} limit={limit} reason={}", reason.as_str());
    notify(
        "file.blocked",
        serde_json::json!({"peer":peer,"bytes":bytes,"limit":limit,"reason":reason.as_str(),"unix_ms":null}),
        out,
    );
}
fn completed(xfer: u64, peer: &str, direction: &str, bytes: u64, kind: u8, out: &mut Vec<Action>) {
    notify(
        "file.completed",
        serde_json::json!({"xfer":xfer,"peer":peer,"direction":direction,"bytes":bytes,"kind":kind,"verified":true,"unix_ms":null}),
        out,
    );
}
fn progress(xfer: u64, peer: &str, direction: &str, done: u64, total: u64, out: &mut Vec<Action>) {
    notify(
        "file.progress",
        serde_json::json!({"xfer":xfer,"peer":peer,"direction":direction,"bytes_done":done,"bytes_total":total}),
        out,
    );
}

/// Context session dispatch passes alongside the peer record.
#[derive(Clone, Copy)]
pub(crate) struct Ctx {
    pub tls_ok: bool,
    /// Peer's receive cap in bytes, when capability is known.
    pub peer_cap: Option<u64>,
    /// Peer can receive explicit Openergy sends.
    pub peer_explicit: bool,
}

impl FileTransfers {
    pub fn deadline(&self, now: u64, cfg: &Config, bulk_ready_at: u64, heartbeat_ms: u64) -> Option<u64> {
        let mut best: Option<u64> = None;
        let mut consider = |t: u64| best = Some(best.map_or(t, |b: u64| b.min(t)));
        if let Some(o) = &self.out {
            match o.state {
                OutState::AwaitingAnswer => consider(o.since_ms.saturating_add(cfg.file.offer_timeout())),
                OutState::Sending => {
                    if !o.started || o.held.is_some() || (o.sent >= o.total_len && !o.outstanding) {
                        consider(bulk_ready_at.max(now));
                    }
                }
                OutState::AwaitingStored => consider(o.since_ms.saturating_add(inbound_expiry_ms(heartbeat_ms))),
            }
        }
        if let Some(i) = &self.inn {
            match i.state {
                InState::AwaitingDecision => consider(i.since_ms.saturating_add(cfg.file.offer_timeout())),
                InState::Receiving => consider(i.last_data_ms.saturating_add(inbound_expiry_ms(heartbeat_ms))),
                InState::Finishing => {}
            }
        }
        best
    }

    /// Daemon asks to offer after the worker hashed the file. The peer
    /// binding was captured at `file.send` call time and is re-validated
    /// by the caller before this runs.
    #[allow(clippy::too_many_arguments)]
    pub fn send_requested(
        &mut self,
        conn: ConnId,
        peer_name: &str,
        now: u64,
        cfg: &Config,
        ctx: Ctx,
        xfer: u64,
        transfer: u32,
        kind: u8,
        name: String,
        total_len: u64,
        sha256: [u8; 32],
        out: &mut Vec<Action>,
    ) {
        let fail = |reason: Reason, out: &mut Vec<Action>| failed(xfer, peer_name, "out", reason, out);
        if transfer == 0 {
            fail(Reason::IoError, out);
            return;
        }
        if !cfg.file.enabled || !cfg.file.send {
            fail(Reason::Disabled, out);
            return;
        }
        if !ctx.tls_ok {
            fail(Reason::TlsRequired, out);
            return;
        }
        if !ctx.peer_explicit {
            fail(Reason::Unsupported, out);
            return;
        }
        if kind != 0 {
            fail(Reason::Unsupported, out);
            return;
        }
        if validate_name(&name).is_err() {
            fail(Reason::BadName, out);
            return;
        }
        let limit = cfg
            .file
            .max_bytes
            .min(FILE_HARD_CAP)
            .min(ctx.peer_cap.unwrap_or(FILE_HARD_CAP));
        if total_len > limit {
            blocked(peer_name, total_len, limit, Reason::TooLarge, out);
            fail(Reason::TooLarge, out);
            return;
        }
        if self.out.is_some() {
            fail(Reason::Busy, out);
            return;
        }
        self.out = Some(OutTransfer {
            xfer,
            transfer,
            kind,
            total_len,
            state: OutState::AwaitingAnswer,
            since_ms: now,
            request_offset: 0,
            sent: 0,
            credit: 0,
            outstanding: false,
            requested_len: 0,
            held: None,
            started: false,
        });
        send_msg(
            conn,
            Message::FileOffer {
                transfer,
                kind,
                name,
                total_len,
                sha256,
            },
            out,
        );
    }

    pub fn on_credit(&mut self, xfer: u64, chunks: u32, out: &mut Vec<Action>) {
        if let Some(o) = self.out.as_mut().filter(|o| o.xfer == xfer) {
            o.credit = o.credit.saturating_add(chunks).min(64);
        }
        self.maybe_request(out);
    }

    fn maybe_request(&mut self, out: &mut Vec<Action>) {
        let Some(o) = self.out.as_mut() else { return };
        if o.state != OutState::Sending
            || !o.started
            || o.held.is_some()
            || o.outstanding
            || o.credit == 0
            || o.request_offset >= o.total_len
        {
            return;
        }
        let remaining = o.total_len - o.request_offset;
        let len = remaining.min(FILE_CHUNK_BYTES as u64) as u32;
        o.outstanding = true;
        o.requested_len = len as usize;
        out.push(Action::ReadFileChunk {
            xfer: o.xfer,
            offset: o.request_offset,
            len,
        });
    }

    pub fn on_chunk_ready(&mut self, conn: ConnId, peer_name: &str, xfer: u64, bytes: Vec<u8>, out: &mut Vec<Action>) {
        let Some(o) = self.out.as_mut().filter(|o| o.xfer == xfer) else {
            return;
        };
        if o.state != OutState::Sending || !o.outstanding {
            return;
        }
        o.outstanding = false;
        if bytes.len() != o.requested_len {
            let (xfer, transfer) = (o.xfer, o.transfer);
            self.out = None;
            out.push(Action::PurgeFileBulk { conn, xfer });
            cancel_frame(conn, transfer, Reason::IoError, out);
            failed(xfer, peer_name, "out", Reason::IoError, out);
            return;
        }
        o.request_offset = o.request_offset.saturating_add(bytes.len() as u64);
        o.held = Some(bytes);
    }

    pub fn on_chunk_failed(&mut self, conn: ConnId, peer_name: &str, xfer: u64, reason: Reason, out: &mut Vec<Action>) {
        let Some(o) = self.out.take_if(|o| o.xfer == xfer) else {
            return;
        };
        out.push(Action::PurgeFileBulk { conn, xfer });
        if o.state != OutState::AwaitingAnswer {
            cancel_frame(conn, o.transfer, Reason::IoError, out);
        }
        failed(xfer, peer_name, "out", reason, out);
    }

    pub fn on_bulk_aborted(&mut self, conn: ConnId, peer_name: &str, xfer: u64, out: &mut Vec<Action>) {
        let Some(o) = self.out.take_if(|o| o.xfer == xfer) else {
            return;
        };
        out.push(Action::PurgeFileBulk { conn, xfer });
        if o.state != OutState::AwaitingAnswer {
            cancel_frame(conn, o.transfer, Reason::Stalled, out);
        }
        failed(xfer, peer_name, "out", Reason::Stalled, out);
    }

    /// One inbound `OFOF` from the peer. `xfer` is a pre-minted
    /// daemon-unique handle, burned unused when the offer is rejected
    /// on the spot. Every well-formed offer is answered once: immediate
    /// reject here, or accept/reject after daemon consent.
    #[allow(clippy::too_many_arguments)]
    pub fn on_offer(
        &mut self,
        conn: ConnId,
        peer_name: &str,
        now: u64,
        cfg: &Config,
        ctx: Ctx,
        xfer: u64,
        transfer: u32,
        kind: u8,
        name: String,
        total_len: u64,
        sha256: [u8; 32],
        out: &mut Vec<Action>,
    ) {
        let limit = cfg.file.max_bytes.min(FILE_HARD_CAP);
        let stored_name = match validate_name(&name) {
            Ok(n) => n,
            Err(reason) => {
                answer(conn, transfer, 0, reason, 0, out);
                blocked(peer_name, total_len, limit, reason, out);
                return;
            }
        };
        if kind != 0 {
            answer(conn, transfer, 0, Reason::Unsupported, 0, out);
            blocked(peer_name, total_len, limit, Reason::Unsupported, out);
            return;
        }
        if !cfg.file.enabled || !cfg.file.receive {
            answer(conn, transfer, 0, Reason::Disabled, 0, out);
            blocked(peer_name, total_len, limit, Reason::Disabled, out);
            return;
        }
        if !ctx.tls_ok {
            answer(conn, transfer, 0, Reason::TlsRequired, 0, out);
            blocked(peer_name, total_len, limit, Reason::TlsRequired, out);
            return;
        }
        if total_len > limit {
            answer(conn, transfer, 0, Reason::TooLarge, 0, out);
            blocked(peer_name, total_len, limit, Reason::TooLarge, out);
            return;
        }
        if self.inn.is_some() {
            answer(conn, transfer, 0, Reason::Busy, 0, out);
            blocked(peer_name, total_len, limit, Reason::Busy, out);
            return;
        }
        self.inn = Some(InTransfer {
            xfer,
            transfer,
            kind,
            total_len,
            state: InState::AwaitingDecision,
            since_ms: now,
            last_data_ms: now,
            staged: 0,
            started: false,
        });
        out.push(Action::FileOffered {
            xfer,
            conn,
            kind,
            name: stored_name,
            total_len,
            sha256,
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub fn on_decided(
        &mut self,
        conn: ConnId,
        peer_name: &str,
        now: u64,
        xfer: u64,
        accept: bool,
        reason: Reason,
        resume_offset: u64,
        out: &mut Vec<Action>,
    ) {
        let Some(i) = self.inn.as_mut().filter(|i| i.xfer == xfer) else {
            return;
        };
        if i.state != InState::AwaitingDecision {
            return;
        }
        if accept {
            // Milestone 1 always starts at zero. A nonzero offset from a
            // future peer is refused rather than silently mishandled.
            if resume_offset != 0 {
                let (transfer, total) = (i.transfer, i.total_len);
                self.inn = None;
                answer(conn, transfer, 0, Reason::Unsupported, 0, out);
                blocked(peer_name, total, total, Reason::Unsupported, out);
                out.push(Action::DropFile { xfer, keep_part: false });
                return;
            }
            i.state = InState::Receiving;
            i.last_data_ms = now;
            answer(conn, i.transfer, 1, Reason::None, 0, out);
        } else {
            let (transfer, total) = (i.transfer, i.total_len);
            self.inn = None;
            answer(conn, transfer, 0, reason, 0, out);
            blocked(peer_name, total, total, reason, out);
            out.push(Action::DropFile { xfer, keep_part: false });
        }
    }

    /// One inbound `OFAN` for a transfer we originated.
    #[allow(clippy::too_many_arguments)]
    pub fn on_answer(
        &mut self,
        conn: ConnId,
        peer_name: &str,
        now: u64,
        transfer: u32,
        accept: u8,
        reason_code: u8,
        resume_offset: u64,
        out: &mut Vec<Action>,
    ) {
        let Some(o) = self.out.as_mut().filter(|o| o.transfer == transfer) else {
            return;
        };
        let reason = Reason::from_code(reason_code);
        match (o.state, accept) {
            (OutState::AwaitingAnswer, 1) => {
                if resume_offset > o.total_len {
                    let xfer = o.xfer;
                    self.out = None;
                    failed(xfer, peer_name, "out", Reason::IoError, out);
                    return;
                }
                o.state = OutState::Sending;
                o.since_ms = now;
                o.request_offset = resume_offset;
                o.sent = resume_offset;
            }
            (OutState::AwaitingAnswer, 0) => {
                let xfer = o.xfer;
                self.out = None;
                failed(xfer, peer_name, "out", reason, out);
            }
            (OutState::AwaitingStored, 2) => {
                let (xfer, total, kind) = (o.xfer, o.total_len, o.kind);
                self.out = None;
                completed(xfer, peer_name, "out", total, kind, out);
            }
            (_, 0) => {
                let xfer = o.xfer;
                self.out = None;
                out.push(Action::PurgeFileBulk { conn, xfer });
                failed(xfer, peer_name, "out", reason, out);
            }
            (_, 1) | (_, 2) => {
                log::warn!("file answer out of phase discarded");
            }
            _ => {
                let xfer = o.xfer;
                self.out = None;
                out.push(Action::PurgeFileBulk { conn, xfer });
                failed(xfer, peer_name, "out", Reason::Unsupported, out);
            }
        }
    }

    fn abort_inbound(
        &mut self,
        conn: ConnId,
        peer_name: &str,
        reason: Reason,
        keep_part: bool,
        final_answer: bool,
        out: &mut Vec<Action>,
    ) {
        let Some(i) = self.inn.take() else { return };
        self.mark_dead_in(i.transfer);
        if final_answer {
            answer(conn, i.transfer, 0, reason, 0, out);
        }
        out.push(Action::DropFile {
            xfer: i.xfer,
            keep_part,
        });
        failed(i.xfer, peer_name, "in", reason, out);
    }

    /// One inbound `OFTR` frame. Data marks resolve to the inbound
    /// record. A cancel resolves to the sender-originated (inbound)
    /// record first; see the namespace note in the design §4.1.
    #[allow(clippy::too_many_arguments)]
    pub fn on_data(
        &mut self,
        conn: ConnId,
        peer_name: &str,
        now: u64,
        cfg: &Config,
        transfer: u32,
        chunk: FileDataChunk,
        out: &mut Vec<Action>,
    ) {
        if let FileDataChunk::Cancel { reason } = &chunk {
            let reason = Reason::from_code(*reason);
            if self.inn.as_ref().is_some_and(|i| i.transfer == transfer) {
                let keep = reason != Reason::Cancelled;
                self.abort_inbound(conn, peer_name, reason, keep, false, out);
                return;
            }
            if let Some(o) = self.out.take_if(|o| o.transfer == transfer) {
                out.push(Action::PurgeFileBulk { conn, xfer: o.xfer });
                failed(o.xfer, peer_name, "out", reason, out);
                return;
            }
            self.note_unknown(transfer);
            return;
        }
        let Some(i) = self.inn.as_mut().filter(|i| i.transfer == transfer) else {
            self.note_unknown(transfer);
            return;
        };
        if i.state != InState::Receiving {
            self.note_unknown(transfer);
            return;
        }
        match chunk {
            FileDataChunk::Start { total_len } => {
                if i.started || total_len != i.total_len {
                    self.abort_inbound(conn, peer_name, Reason::IoError, false, true, out);
                    return;
                }
                i.started = true;
                i.last_data_ms = now;
            }
            FileDataChunk::Data(bytes) => {
                if !i.started {
                    self.abort_inbound(conn, peer_name, Reason::IoError, false, true, out);
                    return;
                }
                // Streaming length enforcement, before staging (§4.4).
                let limit = i.total_len.min(cfg.file.max_bytes).min(FILE_HARD_CAP);
                if i.staged.saturating_add(bytes.len() as u64) > limit {
                    self.abort_inbound(conn, peer_name, Reason::TooLarge, false, true, out);
                    return;
                }
                let (xfer, offset, total) = (i.xfer, i.staged, i.total_len);
                i.staged = i.staged.saturating_add(bytes.len() as u64);
                i.last_data_ms = now;
                let done = i.staged;
                out.push(Action::StageFileChunk { xfer, offset, bytes });
                progress(xfer, peer_name, "in", done, total, out);
            }
            FileDataChunk::End => {
                if !i.started || i.staged != i.total_len {
                    self.abort_inbound(conn, peer_name, Reason::IoError, false, true, out);
                    return;
                }
                i.state = InState::Finishing;
                out.push(Action::FinishFile { xfer: i.xfer });
            }
            FileDataChunk::Cancel { .. } => {}
        }
    }

    pub fn on_stage_result(
        &mut self,
        conn: ConnId,
        peer_name: &str,
        xfer: u64,
        ok: bool,
        reason: Reason,
        out: &mut Vec<Action>,
    ) {
        if ok {
            return;
        }
        if self.inn.as_ref().is_some_and(|i| i.xfer == xfer) {
            self.abort_inbound(conn, peer_name, reason, false, true, out);
        }
    }

    pub fn on_finish_result(
        &mut self,
        conn: ConnId,
        peer_name: &str,
        xfer: u64,
        ok: bool,
        reason: Reason,
        out: &mut Vec<Action>,
    ) {
        let Some(i) = self.inn.take_if(|i| i.xfer == xfer) else {
            return;
        };
        if ok {
            answer(conn, i.transfer, 2, Reason::None, 0, out);
            completed(xfer, peer_name, "in", i.total_len, i.kind, out);
        } else {
            self.mark_dead_in(i.transfer);
            answer(conn, i.transfer, 0, reason, 0, out);
            out.push(Action::DropFile { xfer, keep_part: false });
            failed(xfer, peer_name, "in", reason, out);
        }
    }

    /// Local user cancel through IPC, either direction.
    pub fn on_cancel_requested(&mut self, conn: ConnId, peer_name: &str, xfer: u64, out: &mut Vec<Action>) -> bool {
        if let Some(o) = self.out.take_if(|o| o.xfer == xfer) {
            out.push(Action::PurgeFileBulk { conn, xfer });
            cancel_frame(conn, o.transfer, Reason::Cancelled, out);
            failed(xfer, peer_name, "out", Reason::Cancelled, out);
            return true;
        }
        if let Some(i) = self.inn.take_if(|i| i.xfer == xfer) {
            self.mark_dead_in(i.transfer);
            if i.state == InState::AwaitingDecision {
                answer(conn, i.transfer, 0, Reason::Cancelled, 0, out);
            } else {
                cancel_frame(conn, i.transfer, Reason::Cancelled, out);
            }
            out.push(Action::DropFile { xfer, keep_part: false });
            failed(xfer, peer_name, "in", Reason::Cancelled, out);
            return true;
        }
        false
    }

    /// Timer work: expiries plus at most one paced bulk emission.
    /// `bulk_ready` is the shared clipboard/file pacing gate for this
    /// millisecond (clipboard ran first). Returns true when a bulk
    /// frame was emitted, so the caller can advance the shared gate.
    #[allow(clippy::too_many_arguments)]
    pub fn tick(
        &mut self,
        conn: ConnId,
        peer_name: &str,
        now: u64,
        cfg: &Config,
        heartbeat_ms: u64,
        bulk_ready: bool,
        out: &mut Vec<Action>,
    ) -> bool {
        // Offer expiry, sender side (§4.2).
        if self.out.as_ref().is_some_and(|o| {
            o.state == OutState::AwaitingAnswer && now.saturating_sub(o.since_ms) >= cfg.file.offer_timeout()
        }) {
            let o = self.out.take();
            if let Some(o) = o {
                failed(o.xfer, peer_name, "out", Reason::Timeout, out);
            }
        }
        // Final-result expiry after End (§6.2 inbound budget shape).
        if self.out.as_ref().is_some_and(|o| {
            o.state == OutState::AwaitingStored && now.saturating_sub(o.since_ms) >= inbound_expiry_ms(heartbeat_ms)
        }) {
            let o = self.out.take();
            if let Some(o) = o {
                failed(o.xfer, peer_name, "out", Reason::Stalled, out);
            }
        }
        // Prompt expiry backstop, receiver side (§7.1). The daemon owns
        // the prompt and normally decides first with the same bound.
        if self.inn.as_ref().is_some_and(|i| {
            i.state == InState::AwaitingDecision && now.saturating_sub(i.since_ms) >= cfg.file.offer_timeout()
        }) {
            if let Some(i) = self.inn.take() {
                self.mark_dead_in(i.transfer);
                answer(conn, i.transfer, 0, Reason::Timeout, 0, out);
                blocked(peer_name, i.total_len, i.total_len, Reason::Timeout, out);
                out.push(Action::DropFile {
                    xfer: i.xfer,
                    keep_part: false,
                });
            }
        }
        // Inbound inactivity expiry (§6.2). Stall keeps the partial.
        if self.inn.as_ref().is_some_and(|i| {
            i.state == InState::Receiving && now.saturating_sub(i.last_data_ms) >= inbound_expiry_ms(heartbeat_ms)
        }) {
            self.abort_inbound(conn, peer_name, Reason::Stalled, true, true, out);
        }
        self.maybe_request(out);
        if !bulk_ready {
            return false;
        }
        let Some(o) = self.out.as_mut() else { return false };
        if o.state != OutState::Sending {
            return false;
        }
        if !o.started {
            o.started = true;
            let (transfer, total) = (o.transfer, o.total_len);
            out.push(Action::SendFileChunk {
                conn,
                transfer,
                chunk: FileDataChunk::Start { total_len: total },
            });
            self.maybe_request(out);
            return true;
        }
        if let Some(bytes) = o.held.take() {
            o.credit = o.credit.saturating_sub(1);
            o.sent = o.sent.saturating_add(bytes.len() as u64);
            let (xfer, transfer, done, total) = (o.xfer, o.transfer, o.sent, o.total_len);
            out.push(Action::SendFileChunk {
                conn,
                transfer,
                chunk: FileDataChunk::Data(bytes),
            });
            progress(xfer, peer_name, "out", done, total, out);
            self.maybe_request(out);
            return true;
        }
        if o.sent >= o.total_len && !o.outstanding {
            o.state = OutState::AwaitingStored;
            o.since_ms = now;
            let transfer = o.transfer;
            out.push(Action::SendFileChunk {
                conn,
                transfer,
                chunk: FileDataChunk::End,
            });
            return true;
        }
        false
    }

    /// Config change: drop all file state (§7.3). Capability learned
    /// from the peer survives; live transfers do not.
    pub fn reset(&mut self, conn: ConnId, peer_name: &str, out: &mut Vec<Action>) {
        if let Some(o) = self.out.take() {
            out.push(Action::PurgeFileBulk { conn, xfer: o.xfer });
            cancel_frame(conn, o.transfer, Reason::Disabled, out);
            failed(o.xfer, peer_name, "out", Reason::Disabled, out);
        }
        if let Some(i) = self.inn.take() {
            self.mark_dead_in(i.transfer);
            if i.state == InState::AwaitingDecision {
                answer(conn, i.transfer, 0, Reason::Disabled, 0, out);
            } else {
                cancel_frame(conn, i.transfer, Reason::Disabled, out);
            }
            out.push(Action::DropFile {
                xfer: i.xfer,
                keep_part: false,
            });
            failed(i.xfer, peer_name, "in", Reason::Disabled, out);
        }
    }

    /// Peer disconnect: fail live transfers and drop staged bytes.
    /// Milestone 1 deletes partials on disconnect (§4.5).
    pub fn drop_all(&mut self, peer_name: &str, out: &mut Vec<Action>) {
        if let Some(o) = self.out.take() {
            failed(o.xfer, peer_name, "out", Reason::IoError, out);
        }
        if let Some(i) = self.inn.take() {
            out.push(Action::DropFile {
                xfer: i.xfer,
                keep_part: false,
            });
            failed(i.xfer, peer_name, "in", Reason::IoError, out);
        }
    }
}
