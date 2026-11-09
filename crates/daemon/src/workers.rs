use openergy_platform::{CaptureDecision, ClipboardBackend, ClipboardData, InputEvent, InputSource};
use openergy_protocol::ClipboardId;
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        mpsc::{self, Receiver, SyncSender},
        Arc, Mutex,
    },
    time::Duration,
};

pub struct Captured {
    pub movement: Option<InputEvent>,
    pub event: InputEvent,
}
pub enum CaptureCommand {
    Enable(bool),
    Captured(bool),
    Warp(i32, i32),
    Stop,
}
pub struct Capture {
    pub commands: SyncSender<CaptureCommand>,
    pub events: Receiver<Captured>,
    pub movement: Arc<Mutex<Option<InputEvent>>>,
    pub overflow: Arc<AtomicBool>,
    pub retry: Arc<AtomicBool>,
    pub remote: Arc<AtomicBool>,
    pub thread: std::thread::JoinHandle<()>,
}
/// Same identity as the session ledger: button, or KeyID for button zero.
fn key_id(key: u16, button: u16) -> (u16, u16) {
    (button, if button == 0 { key } else { 0 })
}
pub fn capture(mut source: Box<dyn InputSource>, waker: Arc<mio::Waker>) -> Capture {
    let (commands, rx) = mpsc::sync_channel(32);
    let (tx, events) = mpsc::sync_channel(1024);
    let movement = Arc::new(Mutex::new(None::<InputEvent>));
    let moves = movement.clone();
    let remote = Arc::new(AtomicBool::new(false));
    let rem = remote.clone();
    let overflow = Arc::new(AtomicBool::new(false));
    let over = overflow.clone();
    let retry = Arc::new(AtomicBool::new(false));
    let failed = retry.clone();
    let thread = std::thread::spawn(move || {
        let mut enabled = false;
        while let Ok(cmd) = rx.recv() {
            match cmd {
                CaptureCommand::Enable(on) => {
                    if on && !enabled {
                        let tx = tx.clone();
                        let moves = moves.clone();
                        let waker = waker.clone();
                        let rem = rem.clone();
                        let over = over.clone();
                        let retry_waker = waker.clone();
                        // Keys that went down while local. Their release must
                        // reach the local OS even after the cursor left, or the
                        // OS keeps them held; their auto-repeat must not.
                        let mut local_keys = std::collections::BTreeSet::<(u16, u16)>::new();
                        match source.start(Box::new(move |event| {
                            let remote = rem.load(Ordering::Relaxed);
                            let override_decision = match event {
                                InputEvent::KeyDown { key, button, .. } => {
                                    if !remote {
                                        local_keys.insert(key_id(key, button));
                                    }
                                    None
                                }
                                InputEvent::KeyUp { key, button, .. } => (local_keys.remove(&key_id(key, button))
                                    && remote)
                                    .then_some(CaptureDecision::PassOnce),
                                InputEvent::KeyRepeat { key, button, .. } => (remote
                                    && local_keys.contains(&key_id(key, button)))
                                .then_some(CaptureDecision::Swallow),
                                _ => None,
                            };
                            // The runtime holds this lock only to take one
                            // pending move. Waiting is brief. Failing here would
                            // pass a captured event to the local OS and end the
                            // remote session for a lock collision.
                            let mut slot = moves.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                            let is_move =
                                matches!(event, InputEvent::MouseMoveAbs { .. } | InputEvent::MouseMoveRel { .. });
                            if is_move {
                                match (slot.as_mut(), &event) {
                                    (
                                        Some(InputEvent::MouseMoveRel { dx, dy }),
                                        InputEvent::MouseMoveRel { dx: x, dy: y },
                                    ) => {
                                        *dx = dx.saturating_add(*x);
                                        *dy = dy.saturating_add(*y);
                                    }
                                    (Some(InputEvent::MouseMoveAbs { .. }), InputEvent::MouseMoveAbs { .. })
                                    | (None, _) => *slot = Some(event),
                                    _ => {
                                        if let Some(previous) = slot.take() {
                                            if tx
                                                .try_send(Captured {
                                                    movement: None,
                                                    event: previous,
                                                })
                                                .is_err()
                                            {
                                                over.store(true, Ordering::Relaxed);
                                                return CaptureDecision::PassToOs;
                                            }
                                        }
                                        *slot = Some(event);
                                    }
                                }
                            } else {
                                let movement = slot.take();
                                if tx.try_send(Captured { movement, event }).is_err() {
                                    over.store(true, Ordering::Relaxed);
                                    return CaptureDecision::PassToOs;
                                }
                            }
                            drop(slot);
                            let _ = waker.wake();
                            if let Some(decision) = override_decision {
                                decision
                            } else if remote {
                                CaptureDecision::Swallow
                            } else {
                                CaptureDecision::PassToOs
                            }
                        })) {
                            Ok(()) => {
                                enabled = true;
                                failed.store(false, Ordering::Release);
                            }
                            Err(e) => {
                                log::warn!("capture unavailable: {e}");
                                failed.store(true, Ordering::Release);
                                let _ = retry_waker.wake();
                            }
                        }
                    } else if !on && enabled {
                        source.stop();
                        enabled = false;
                    }
                }
                CaptureCommand::Captured(on) => {
                    let _ = source.set_cursor_captured(on);
                }
                CaptureCommand::Warp(x, y) => {
                    let _ = source.warp_cursor(x, y);
                }
                CaptureCommand::Stop => {
                    source.stop();
                    break;
                }
            }
        }
    });
    Capture {
        commands,
        events,
        movement,
        overflow,
        retry,
        remote,
        thread,
    }
}
pub enum ClipboardCommand {
    Read {
        id: ClipboardId,
        max: usize,
    },
    Write {
        id: ClipboardId,
        data: Arc<ClipboardData>,
    },
    /// `clipboard` enables polling id 0; `primary` additionally polls id 1.
    Enabled {
        clipboard: bool,
        primary: bool,
    },
    Stop,
}
pub enum ClipboardEvent {
    Blocked {
        id: ClipboardId,
        bytes: usize,
        limit: usize,
    },
    Changed(ClipboardId),
    Payload {
        id: ClipboardId,
        data: ClipboardData,
    },
    ReadUnavailable {
        id: ClipboardId,
    },
    Failure {
        read_id: Option<ClipboardId>,
        error: String,
    },
}
pub struct ClipboardWorker {
    pub commands: SyncSender<ClipboardCommand>,
    pub events: Receiver<ClipboardEvent>,
    pub failed_reads: Arc<AtomicU8>,
    pub thread: std::thread::JoinHandle<()>,
}
pub fn clipboard(mut backend: Box<dyn ClipboardBackend>, waker: Arc<mio::Waker>) -> ClipboardWorker {
    let (commands, rx) = mpsc::sync_channel(8);
    let (tx, events) = mpsc::sync_channel(16);
    let failed_reads = Arc::new(AtomicU8::new(0));
    let failed = failed_reads.clone();
    let thread = std::thread::spawn(move || {
        let mut stamps = [backend.change_stamp_for(0), backend.change_stamp_for(1)];
        let mut enabled = false;
        let mut primary = false;
        // PRIMARY changes on every selection motion. A candidate stamp must
        // survive one more poll unchanged before it becomes a change event.
        let mut primary_candidate: Option<u64> = None;
        loop {
            match rx.recv_timeout(if enabled {
                Duration::from_millis(200)
            } else {
                Duration::from_secs(60)
            }) {
                Ok(ClipboardCommand::Read { id, max }) => {
                    let ev = match backend.read_for(id.0, max) {
                        Ok(data) => ClipboardEvent::Payload { id, data },
                        Err(openergy_platform::PlatformError::ClipboardUnavailable) => {
                            ClipboardEvent::ReadUnavailable { id }
                        }
                        Err(openergy_platform::PlatformError::ClipboardTooLarge { bytes, limit }) => {
                            ClipboardEvent::Blocked {
                                id,
                                bytes: bytes.saturating_add(max.saturating_sub(limit)),
                                limit: max,
                            }
                        }
                        Err(e) => ClipboardEvent::Failure {
                            read_id: Some(id),
                            error: e.to_string(),
                        },
                    };
                    if tx.try_send(ev).is_err() {
                        if id.0 < 2 {
                            // A dropped completion must still release the session's pending read.
                            failed.fetch_or(1 << id.0, Ordering::Release);
                        }
                        log::warn!("clipboard result queue full");
                    }
                    let _ = waker.wake();
                }
                Ok(ClipboardCommand::Write { id, data }) => {
                    if let Err(e) = backend.write_shared_for(id.0, data) {
                        let _ = tx.try_send(ClipboardEvent::Failure {
                            read_id: None,
                            error: e.to_string(),
                        });
                    }
                    // Own writes never become a new local grab. Refresh
                    // only the written id's stamp, so a remote PRIMARY
                    // write cannot re-fire as a CLIPBOARD change or the
                    // other way round.
                    if id.0 < 2 {
                        stamps[id.0 as usize] = backend.change_stamp_for(id.0);
                    }
                    if id.0 == 1 {
                        primary_candidate = None;
                    }
                    let _ = waker.wake();
                }
                Ok(ClipboardCommand::Enabled {
                    clipboard: on,
                    primary: primary_on,
                }) => {
                    enabled = on;
                    primary = on && primary_on;
                    if !primary {
                        primary_candidate = None;
                    }
                }
                Ok(ClipboardCommand::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
            if enabled {
                let current = backend.change_stamp_for(0);
                if current != stamps[0] {
                    stamps[0] = current;
                    let _ = tx.try_send(ClipboardEvent::Changed(ClipboardId::CLIPBOARD));
                    let _ = waker.wake();
                }
                if primary {
                    let current = backend.change_stamp_for(1);
                    if current == stamps[1] {
                        primary_candidate = None;
                    } else if primary_candidate == Some(current) {
                        stamps[1] = current;
                        primary_candidate = None;
                        let _ = tx.try_send(ClipboardEvent::Changed(ClipboardId::SELECTION));
                        let _ = waker.wake();
                    } else {
                        primary_candidate = Some(current);
                    }
                }
            }
        }
    });
    ClipboardWorker {
        commands,
        events,
        failed_reads,
        thread,
    }
}
