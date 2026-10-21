use openergy_platform::{CaptureDecision, ClipboardBackend, ClipboardData, InputEvent, InputSource};
use openergy_protocol::ClipboardId;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
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
    pub remote: Arc<AtomicBool>,
    pub thread: std::thread::JoinHandle<()>,
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
                        match source.start(Box::new(move |event| {
                            let Ok(mut slot) = moves.try_lock() else {
                                over.store(true, Ordering::Relaxed);
                                return CaptureDecision::PassToOs;
                            };
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
                            if rem.load(Ordering::Relaxed) {
                                CaptureDecision::Swallow
                            } else {
                                CaptureDecision::PassToOs
                            }
                        })) {
                            Ok(()) => enabled = true,
                            Err(e) => log::warn!("capture unavailable: {e}"),
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
        remote,
        thread,
    }
}
pub enum ClipboardCommand {
    Read { id: ClipboardId, max: usize },
    Write { data: Arc<ClipboardData> },
    Enabled(bool),
    Stop,
}
pub enum ClipboardEvent {
    Blocked { bytes: usize, limit: usize },
    Changed,
    Payload { id: ClipboardId, data: ClipboardData },
    Failure(String),
}
pub struct ClipboardWorker {
    pub commands: SyncSender<ClipboardCommand>,
    pub events: Receiver<ClipboardEvent>,
    pub thread: std::thread::JoinHandle<()>,
}
pub fn clipboard(mut backend: Box<dyn ClipboardBackend>, waker: Arc<mio::Waker>) -> ClipboardWorker {
    let (commands, rx) = mpsc::sync_channel(8);
    let (tx, events) = mpsc::sync_channel(16);
    let thread = std::thread::spawn(move || {
        let mut stamp = backend.change_stamp();
        let mut enabled = false;
        loop {
            match rx.recv_timeout(if enabled {
                Duration::from_millis(200)
            } else {
                Duration::from_secs(60)
            }) {
                Ok(ClipboardCommand::Read { id, max }) => {
                    let ev = match backend.read(max) {
                        Ok(data) => ClipboardEvent::Payload { id, data },
                        Err(openergy_platform::PlatformError::ClipboardTooLarge { bytes, limit }) => {
                            ClipboardEvent::Blocked {
                                bytes: bytes.saturating_add(max.saturating_sub(limit)),
                                limit: max,
                            }
                        }
                        Err(e) => ClipboardEvent::Failure(e.to_string()),
                    };
                    if tx.try_send(ev).is_err() {
                        log::warn!("clipboard result queue full");
                    }
                    let _ = waker.wake();
                }
                Ok(ClipboardCommand::Write { data }) => {
                    if let Err(e) = backend.write_shared(data) {
                        let _ = tx.try_send(ClipboardEvent::Failure(e.to_string()));
                    }
                    // Own writes never become a new local grab.
                    stamp = backend.change_stamp();
                    let _ = waker.wake();
                }
                Ok(ClipboardCommand::Enabled(on)) => {
                    enabled = on;
                }
                Ok(ClipboardCommand::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
            if enabled {
                let current = backend.change_stamp();
                if current != stamp {
                    stamp = current;
                    let _ = tx.try_send(ClipboardEvent::Changed);
                    let _ = waker.wake();
                }
            }
        }
    });
    ClipboardWorker {
        commands,
        events,
        thread,
    }
}
