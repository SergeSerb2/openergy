//! Local blocking IPC clients submit all mutations to the single session owner.
use crate::{events::EventBus, runtime::IoCommand};
use openergy_ipc::{write_frame, Request, Response};
#[cfg(unix)]
use std::{
    fs::{File, OpenOptions},
    os::{
        fd::AsRawFd,
        unix::{
            fs::OpenOptionsExt,
            net::{UnixListener, UnixStream},
        },
    },
};
use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender},
        Arc,
    },
    time::Duration,
};

#[cfg(unix)]
pub struct SocketLock {
    path: PathBuf,
    _file: File,
}
#[cfg(unix)]
impl Drop for SocketLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
#[cfg(unix)]
pub fn bind(path: &Path) -> Result<(UnixListener, SocketLock), String> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(parent).map_err(|e| e.to_string())?;
        // Shared sticky directories such as /tmp must retain their system mode.
        if metadata.permissions().mode() & 0o1000 == 0 {
            std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700)).map_err(|e| e.to_string())?;
        }
    }
    let lock_path = path.with_extension("sock.lock");
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(&lock_path)
        .map_err(|e| e.to_string())?;
    // Safety: flock operates on our open lock file and does not affect other descriptors.
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        return Err(format!("another daemon owns {}", path.display()));
    }
    if path.exists() {
        if UnixStream::connect(path).is_ok() {
            return Err(format!("another daemon is listening on {}", path.display()));
        }
        std::fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    let listener = UnixListener::bind(path).map_err(|e| format!("bind {}: {e}", path.display()))?;
    mode(path, 0o600);
    Ok((
        listener,
        SocketLock {
            path: path.to_path_buf(),
            _file: file,
        },
    ))
}
#[cfg(unix)]
fn mode(path: &Path, n: u32) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(n));
}
#[cfg(unix)]
pub fn serve(
    listener: UnixListener,
    path: PathBuf,
    tx: SyncSender<IoCommand>,
    waker: Arc<mio::Waker>,
    bus: Arc<EventBus>,
    stop: Arc<AtomicBool>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut clients = Vec::new();
        for conn in listener.incoming() {
            if stop.load(Ordering::Relaxed) {
                break;
            }
            clients.retain(|h: &std::thread::JoinHandle<()>| !h.is_finished());
            let Ok(stream) = conn else { continue };
            if clients.len() >= 64 {
                continue;
            }
            let tx = tx.clone();
            let waker = waker.clone();
            let bus = bus.clone();
            let stop = stop.clone();
            clients.push(std::thread::spawn(move || client(stream, tx, waker, bus, stop)));
        }
        drop(listener);
        let _ = std::fs::remove_file(path);
        for h in clients {
            let _ = h.join();
        }
    })
}
#[cfg(unix)]
fn client(
    mut stream: UnixStream,
    tx: SyncSender<IoCommand>,
    waker: Arc<mio::Waker>,
    bus: Arc<EventBus>,
    stop: Arc<AtomicBool>,
) {
    if stream.set_nonblocking(false).is_err() {
        return;
    }
    let _ = stream.set_read_timeout(Some(Duration::from_millis(50)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
    let queue = bus.client();
    let mut buf = Vec::new();
    let mut scratch = [0u8; 8192];
    while !stop.load(Ordering::Relaxed) {
        match stream.read(&mut scratch) {
            Ok(0) => return,
            Ok(n) => {
                buf.extend_from_slice(&scratch[..n]);
            }
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut | std::io::ErrorKind::Interrupted
                ) => {}
            Err(_) => return,
        }
        loop {
            let Some(h) = buf.get(..4) else { break };
            let Ok(header) = <[u8; 4]>::try_from(h) else { return };
            let n = u32::from_be_bytes(header) as usize;
            if n > 16 * 1024 * 1024 {
                return;
            }
            if buf.len() < n + 4 {
                break;
            }
            let request = serde_json::from_slice::<Request>(&buf[4..n + 4]);
            buf.drain(..n + 4);
            let Ok(req) = request else { return };
            let (reply, answer) = mpsc::channel::<Response>();
            if tx
                .try_send(IoCommand::Ipc {
                    req,
                    reply,
                    events: queue.clone(),
                })
                .is_err()
            {
                return;
            }
            let _ = waker.wake();
            let Ok(response) = answer.recv_timeout(Duration::from_secs(10)) else {
                return;
            };
            let Ok(value) = serde_json::to_value(response) else {
                return;
            };
            if write_frame(&mut stream, &value).is_err() {
                return;
            }
        }
        for ev in queue.drain() {
            let Ok(value) = serde_json::to_value(ev) else { return };
            if write_frame(&mut stream, &value).is_err() {
                return;
            }
        }
    }
}
