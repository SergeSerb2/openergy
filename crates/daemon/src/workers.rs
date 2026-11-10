use openergy_platform::{CaptureDecision, ClipboardBackend, ClipboardData, InputEvent, InputSource};
use openergy_protocol::ClipboardId;
use openergy_session::file::Reason as FileReason;
use sha2::Digest;
use std::{
    collections::{BTreeMap, VecDeque},
    io::Write,
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

// ---------------------------------------------------------------------
// File worker (design §6.1): the only component that touches file
// bytes on disk. One thread; hashing is a resumable background job
// that yields to every active-transfer command first.

pub enum FileCommand {
    HashFile {
        xfer: u64,
        path: std::path::PathBuf,
    },
    ReadChunk {
        xfer: u64,
        offset: u64,
        len: u32,
    },
    DropSend {
        xfer: u64,
    },
    BeginReceive {
        xfer: u64,
        name: String,
        total_len: u64,
        sha256: [u8; 32],
        on_conflict_reject: bool,
    },
    Stage {
        xfer: u64,
        offset: u64,
        bytes: Vec<u8>,
    },
    Finish {
        xfer: u64,
    },
    DropReceive {
        xfer: u64,
        keep_part: bool,
    },
    SetRoot {
        root: std::path::PathBuf,
    },
    Stop,
}
pub enum FileEvent {
    Hashed {
        xfer: u64,
        name: String,
        total_len: u64,
        sha256: [u8; 32],
    },
    HashFailed {
        xfer: u64,
        /// Local failure names: io_error / changed_during_read / bad_name.
        reason: &'static str,
    },
    ChunkReady {
        xfer: u64,
        bytes: Vec<u8>,
    },
    ChunkFailed {
        xfer: u64,
    },
    ReceiveReady {
        xfer: u64,
        ok: bool,
        reason: openergy_session::file::Reason,
    },
    StageResult {
        xfer: u64,
        ok: bool,
        reason: openergy_session::file::Reason,
    },
    FinishResult {
        xfer: u64,
        ok: bool,
        reason: openergy_session::file::Reason,
    },
}
pub struct FileWorker {
    pub commands: SyncSender<FileCommand>,
    pub events: Receiver<FileEvent>,
    pub thread: std::thread::JoinHandle<()>,
}

struct SendFile {
    file: std::fs::File,
    len: u64,
    modified: Option<std::time::SystemTime>,
    checked: bool,
}
struct HashJob {
    xfer: u64,
    name: String,
    file: std::fs::File,
    len: u64,
    modified: Option<std::time::SystemTime>,
    offset: u64,
    hasher: sha2::Sha256,
}
struct RecvFile {
    file: std::fs::File,
    part_path: std::path::PathBuf,
    name: String,
    total_len: u64,
    sha256: [u8; 32],
    staged: u64,
    hasher: sha2::Sha256,
    on_conflict_reject: bool,
}

fn read_at(file: &std::fs::File, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileExt;
        file.read_at(buf, offset)
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::FileExt;
        file.seek_read(buf, offset)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (file, buf, offset);
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "pread unsupported",
        ))
    }
}

#[cfg(unix)]
fn free_bytes(path: &std::path::Path) -> Option<u64> {
    use std::os::unix::ffi::OsStrExt;
    let cpath = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
    // Safety: statvfs reads filesystem counters for a NUL-terminated path.
    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(cpath.as_ptr(), &mut stat) != 0 {
            return None;
        }
        let free = (stat.f_bavail as u128).saturating_mul(stat.f_frsize as u128);
        u64::try_from(free).ok()
    }
}
#[cfg(not(unix))]
fn free_bytes(_path: &std::path::Path) -> Option<u64> {
    None
}

fn is_symlink(path: &std::path::Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|meta| meta.file_type().is_symlink())
}

fn ensure_parts(root: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
    let parts = root.join(".parts");
    if is_symlink(root) || is_symlink(&parts) {
        // A planted link must never steer staging into another directory.
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "staging directory is a symlink",
        ));
    }
    let root_existed = root.exists();
    let parts_existed = parts.exists();
    std::fs::create_dir_all(&parts)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if !root_existed {
            let _ = std::fs::set_permissions(root, std::fs::Permissions::from_mode(0o700));
        }
        if !parts_existed {
            let _ = std::fs::set_permissions(&parts, std::fs::Permissions::from_mode(0o700));
        }
    }
    Ok(parts)
}

/// Milestone 1 startup sweep: delete every staged partial (resume is
/// off). Completed files in the root are never candidates.
fn sweep_parts(root: &std::path::Path) {
    let parts = root.join(".parts");
    if is_symlink(root) || is_symlink(&parts) {
        log::warn!("file partials sweep skipped: staging directory is a symlink");
        return;
    }
    let Ok(entries) = std::fs::read_dir(&parts) else { return };
    let (mut count, mut bytes) = (0u64, 0u64);
    for entry in entries.flatten() {
        // file_type does not follow links; only real staged files are removed.
        if entry.file_type().is_ok_and(|kind| kind.is_file()) {
            if let Ok(meta) = entry.metadata() {
                bytes = bytes.saturating_add(meta.len());
                if std::fs::remove_file(entry.path()).is_ok() {
                    count += 1;
                }
            }
        }
    }
    if count > 0 {
        log::info!("file partials swept: count={count} bytes={bytes}");
    }
}

fn begin_hash(
    xfer: u64,
    path: &std::path::Path,
    tx: &SyncSender<FileEvent>,
    waker: &Arc<mio::Waker>,
) -> Option<HashJob> {
    let fail = |reason: &'static str| {
        let _ = tx.try_send(FileEvent::HashFailed { xfer, reason });
        let _ = waker.wake();
        None
    };
    let Ok(link_meta) = std::fs::symlink_metadata(path) else {
        return fail("io_error");
    };
    if link_meta.file_type().is_symlink() || !link_meta.is_file() {
        return fail("io_error");
    }
    let Some(name) = path.file_name().and_then(|n| n.to_str()).map(str::to_owned) else {
        return fail("bad_name");
    };
    // Send under the validated (trimmed) name when it differs.
    let Ok(name) = openergy_session::file::validate_name(&name) else {
        return fail("bad_name");
    };
    let Ok(file) = std::fs::File::open(path) else {
        return fail("io_error");
    };
    let Ok(meta) = file.metadata() else {
        return fail("io_error");
    };
    if !meta.is_file() {
        return fail("io_error");
    }
    if meta.len() > openergy_session::file::FILE_HARD_CAP {
        return fail("io_error");
    }
    Some(HashJob {
        xfer,
        name,
        file,
        len: meta.len(),
        modified: meta.modified().ok(),
        offset: 0,
        hasher: sha2::Sha256::new(),
    })
}

pub fn file_worker(root: std::path::PathBuf, waker: Arc<mio::Waker>) -> FileWorker {
    let (commands, rx) = mpsc::sync_channel::<FileCommand>(256);
    let (tx, events) = mpsc::sync_channel::<FileEvent>(256);
    let thread = std::thread::spawn(move || {
        let emit = |tx: &SyncSender<FileEvent>, waker: &Arc<mio::Waker>, ev: FileEvent| {
            if tx.try_send(ev).is_err() {
                log::warn!("file event queue full");
            }
            let _ = waker.wake();
        };
        let mut root = root;
        let _ = ensure_parts(&root);
        sweep_parts(&root);
        let mut sends: BTreeMap<u64, SendFile> = BTreeMap::new();
        let mut recvs: BTreeMap<u64, RecvFile> = BTreeMap::new();
        let mut hash: Option<HashJob> = None;
        let mut hash_queue: VecDeque<(u64, std::path::PathBuf)> = VecDeque::new();
        let mut buf = vec![0u8; 1024 * 1024];
        'outer: loop {
            // Drain every pending command first; hash slices run last.
            // A queued hash job must be promoted before the blocking wait,
            // or it stalls until the 60 s timeout.
            let first = if hash.is_some() || !hash_queue.is_empty() {
                rx.try_recv().ok()
            } else {
                match rx.recv_timeout(Duration::from_secs(60)) {
                    Ok(cmd) => Some(cmd),
                    Err(mpsc::RecvTimeoutError::Disconnected) => break 'outer,
                    Err(mpsc::RecvTimeoutError::Timeout) => None,
                }
            };
            let mut pending: VecDeque<FileCommand> = VecDeque::new();
            if let Some(cmd) = first {
                pending.push_back(cmd);
            }
            while let Ok(cmd) = rx.try_recv() {
                pending.push_back(cmd);
            }
            for cmd in pending {
                match cmd {
                    FileCommand::Stop => break 'outer,
                    FileCommand::SetRoot { root: new_root } => {
                        root = new_root;
                        let _ = ensure_parts(&root);
                    }
                    FileCommand::HashFile { xfer, path } => {
                        if hash.is_none() {
                            hash = begin_hash(xfer, &path, &tx, &waker);
                        } else {
                            hash_queue.push_back((xfer, path));
                        }
                    }
                    FileCommand::DropSend { xfer } => {
                        sends.remove(&xfer);
                        hash_queue.retain(|(queued, _)| *queued != xfer);
                        if hash.as_ref().is_some_and(|job| job.xfer == xfer) {
                            hash = None;
                        }
                    }
                    FileCommand::ReadChunk { xfer, offset, len } => {
                        let result = (|| -> Result<Vec<u8>, ()> {
                            let send = sends.get_mut(&xfer).ok_or(())?;
                            if !send.checked {
                                // Recheck size and mtime before the
                                // first chunk (§4.6).
                                send.checked = true;
                                let meta = send.file.metadata().map_err(|_| ())?;
                                if meta.len() != send.len || meta.modified().ok() != send.modified {
                                    return Err(());
                                }
                            }
                            let mut bytes = vec![0u8; len as usize];
                            let mut done = 0usize;
                            while done < bytes.len() {
                                let n = read_at(
                                    &send.file,
                                    bytes.get_mut(done..).unwrap_or(&mut []),
                                    offset.saturating_add(done as u64),
                                )
                                .map_err(|_| ())?;
                                if n == 0 {
                                    return Err(());
                                }
                                done += n;
                            }
                            Ok(bytes)
                        })();
                        match result {
                            Ok(bytes) => emit(&tx, &waker, FileEvent::ChunkReady { xfer, bytes }),
                            Err(()) => {
                                sends.remove(&xfer);
                                emit(&tx, &waker, FileEvent::ChunkFailed { xfer });
                            }
                        }
                    }
                    FileCommand::BeginReceive {
                        xfer,
                        name,
                        total_len,
                        sha256,
                        on_conflict_reject,
                    } => {
                        let result = (|| -> Result<RecvFile, FileReason> {
                            if recvs.contains_key(&xfer) {
                                return Err(FileReason::Busy);
                            }
                            let parts = ensure_parts(&root).map_err(|_| FileReason::IoError)?;
                            if let Some(free) = free_bytes(&root) {
                                if free <= total_len.saturating_add(64 * 1024 * 1024) {
                                    return Err(FileReason::NoSpace);
                                }
                            }
                            let part_path = parts.join(xfer.to_string());
                            let mut options = std::fs::OpenOptions::new();
                            options.create_new(true).write(true);
                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::OpenOptionsExt;
                                options.mode(0o644);
                            }
                            let file = options.open(&part_path).map_err(|e| {
                                if e.kind() == std::io::ErrorKind::AlreadyExists {
                                    FileReason::Busy
                                } else {
                                    FileReason::IoError
                                }
                            })?;
                            Ok(RecvFile {
                                file,
                                part_path,
                                name,
                                total_len,
                                sha256,
                                staged: 0,
                                hasher: sha2::Sha256::new(),
                                on_conflict_reject,
                            })
                        })();
                        match result {
                            Ok(recv) => {
                                recvs.insert(xfer, recv);
                                emit(
                                    &tx,
                                    &waker,
                                    FileEvent::ReceiveReady {
                                        xfer,
                                        ok: true,
                                        reason: FileReason::None,
                                    },
                                );
                            }
                            Err(reason) => emit(
                                &tx,
                                &waker,
                                FileEvent::ReceiveReady {
                                    xfer,
                                    ok: false,
                                    reason,
                                },
                            ),
                        }
                    }
                    FileCommand::Stage { xfer, offset, bytes } => {
                        let result = (|| -> Result<(), FileReason> {
                            let recv = recvs.get_mut(&xfer).ok_or(FileReason::IoError)?;
                            if offset != recv.staged {
                                return Err(FileReason::IoError);
                            }
                            recv.file.write_all(&bytes).map_err(|_| FileReason::IoError)?;
                            recv.hasher.update(&bytes);
                            recv.staged = recv.staged.saturating_add(bytes.len() as u64);
                            Ok(())
                        })();
                        if let Err(reason) = result {
                            emit(
                                &tx,
                                &waker,
                                FileEvent::StageResult {
                                    xfer,
                                    ok: false,
                                    reason,
                                },
                            );
                        }
                    }
                    FileCommand::Finish { xfer } => {
                        let result = (|| -> Result<(), FileReason> {
                            let recv = recvs.remove(&xfer).ok_or(FileReason::IoError)?;
                            let digest: [u8; 32] = recv.hasher.finalize().into();
                            if recv.staged != recv.total_len {
                                let _ = std::fs::remove_file(&recv.part_path);
                                return Err(FileReason::IoError);
                            }
                            if digest != recv.sha256 {
                                let _ = std::fs::remove_file(&recv.part_path);
                                return Err(FileReason::HashMismatch);
                            }
                            if recv.file.sync_all().is_err() {
                                let _ = std::fs::remove_file(&recv.part_path);
                                return Err(FileReason::IoError);
                            }
                            drop(recv.file);
                            // No-replace publish: hard link fails on
                            // conflict, then the staging name is dropped.
                            let (stem, ext) = match recv.name.rfind('.') {
                                Some(dot) if dot > 0 => (
                                    recv.name.get(..dot).unwrap_or(&recv.name),
                                    recv.name.get(dot..).unwrap_or(""),
                                ),
                                _ => (recv.name.as_str(), ""),
                            };
                            let mut published = false;
                            for attempt in 0..100u32 {
                                let candidate = if attempt == 0 {
                                    recv.name.clone()
                                } else {
                                    format!("{stem} ({attempt}){ext}")
                                };
                                match std::fs::hard_link(&recv.part_path, root.join(&candidate)) {
                                    Ok(()) => {
                                        published = true;
                                        break;
                                    }
                                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                                        if recv.on_conflict_reject {
                                            let _ = std::fs::remove_file(&recv.part_path);
                                            return Err(FileReason::NameConflict);
                                        }
                                    }
                                    Err(_) => {
                                        // The receive record is already
                                        // removed, so nobody else can clean
                                        // the staged file up.
                                        let _ = std::fs::remove_file(&recv.part_path);
                                        return Err(FileReason::IoError);
                                    }
                                }
                            }
                            if !published {
                                let _ = std::fs::remove_file(&recv.part_path);
                                return Err(FileReason::NameConflict);
                            }
                            let _ = std::fs::remove_file(&recv.part_path);
                            #[cfg(unix)]
                            {
                                if let Ok(dir) = std::fs::File::open(&root) {
                                    let _ = dir.sync_all();
                                }
                            }
                            Ok(())
                        })();
                        match result {
                            Ok(()) => emit(
                                &tx,
                                &waker,
                                FileEvent::FinishResult {
                                    xfer,
                                    ok: true,
                                    reason: FileReason::None,
                                },
                            ),
                            Err(reason) => emit(
                                &tx,
                                &waker,
                                FileEvent::FinishResult {
                                    xfer,
                                    ok: false,
                                    reason,
                                },
                            ),
                        }
                    }
                    FileCommand::DropReceive { xfer, keep_part } => {
                        if let Some(recv) = recvs.remove(&xfer) {
                            drop(recv.file);
                            if !keep_part {
                                let _ = std::fs::remove_file(&recv.part_path);
                            }
                        }
                    }
                }
            }
            // At most one 1 MiB hash slice per loop turn.
            if hash.is_none() {
                if let Some((xfer, path)) = hash_queue.pop_front() {
                    hash = begin_hash(xfer, &path, &tx, &waker);
                }
            }
            if let Some(job) = &mut hash {
                let remaining = job.len.saturating_sub(job.offset);
                let want = remaining.min(buf.len() as u64) as usize;
                let mut failure: Option<&'static str> = None;
                if want > 0 {
                    let mut done = 0usize;
                    while done < want && failure.is_none() {
                        match read_at(
                            &job.file,
                            buf.get_mut(done..want).unwrap_or(&mut []),
                            job.offset.saturating_add(done as u64),
                        ) {
                            Ok(0) => failure = Some("changed_during_read"),
                            Ok(n) => done += n,
                            Err(_) => failure = Some("io_error"),
                        }
                    }
                    if failure.is_none() {
                        if let Some(slice) = buf.get(..want) {
                            job.hasher.update(slice);
                        }
                        job.offset = job.offset.saturating_add(want as u64);
                    }
                }
                let finished = failure.is_none() && job.offset >= job.len;
                if failure.is_some() || finished {
                    let job = hash.take();
                    if let Some(job) = job {
                        if let Some(reason) = failure {
                            emit(&tx, &waker, FileEvent::HashFailed { xfer: job.xfer, reason });
                        } else {
                            // A file that grew during the hash pass
                            // fails instead of sending a live tail.
                            let changed = job
                                .file
                                .metadata()
                                .map(|m| m.len() != job.len || m.modified().ok() != job.modified)
                                .unwrap_or(true);
                            if changed {
                                emit(
                                    &tx,
                                    &waker,
                                    FileEvent::HashFailed {
                                        xfer: job.xfer,
                                        reason: "changed_during_read",
                                    },
                                );
                            } else {
                                let digest: [u8; 32] = job.hasher.finalize().into();
                                sends.insert(
                                    job.xfer,
                                    SendFile {
                                        file: job.file,
                                        len: job.len,
                                        modified: job.modified,
                                        checked: false,
                                    },
                                );
                                emit(
                                    &tx,
                                    &waker,
                                    FileEvent::Hashed {
                                        xfer: job.xfer,
                                        name: job.name,
                                        total_len: job.len,
                                        sha256: digest,
                                    },
                                );
                            }
                        }
                    }
                }
            }
        }
    });
    FileWorker {
        commands,
        events,
        thread,
    }
}
