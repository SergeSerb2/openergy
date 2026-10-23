//! Local blocking IPC clients submit all mutations to the single session owner.
#[cfg(windows)]
pub use crate::windows_pipe::bind;
#[cfg(windows)]
use crate::windows_pipe::Stream as UnixStream;
use crate::{events::EventBus, runtime::IoCommand};
use openergy_ipc::{write_frame, IpcError, Request, Response, MAX_FRAME_LEN};
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::{
    fs::{File, OpenOptions},
    os::{
        fd::AsRawFd,
        unix::{
            fs::{FileTypeExt, MetadataExt, OpenOptionsExt},
            net::{UnixListener, UnixStream},
        },
    },
};
use std::{
    io::Read,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender},
        Arc,
    },
    time::{Duration, Instant},
};

const REQUEST_READ_TIMEOUT: Duration = Duration::from_secs(10);

#[cfg(unix)]
pub struct SocketLock {
    path: PathBuf,
    device: u64,
    inode: u64,
    _file: File,
}
#[cfg(unix)]
pub struct Listener {
    socket: UnixListener,
    shutdown_read: UnixStream,
    shutdown_write: UnixStream,
}
#[cfg(unix)]
pub struct ServerThread {
    thread: std::thread::JoinHandle<()>,
    shutdown: UnixStream,
}
#[cfg(unix)]
impl ServerThread {
    pub fn join(self) -> std::thread::Result<()> {
        // Closing the private peer wakes poll without a timer or a pathname
        // lookup. No connection can be sent to a replaced control socket.
        drop(self.shutdown);
        self.thread.join()
    }
}
#[cfg(unix)]
impl Drop for SocketLock {
    fn drop(&mut self) {
        if std::fs::symlink_metadata(&self.path)
            .is_ok_and(|m| m.file_type().is_socket() && m.dev() == self.device && m.ino() == self.inode)
        {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}
#[cfg(unix)]
pub fn bind(path: &Path) -> Result<(Listener, SocketLock), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
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
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(&lock_path)
        .map_err(|e| e.to_string())?;
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    // Safety: geteuid only queries this process's effective user ID.
    let uid = unsafe { libc::geteuid() };
    if !metadata.is_file() || metadata.uid() != uid || metadata.nlink() != 1 {
        return Err("IPC lock must be an owned regular file with one link".into());
    }
    // Safety: flock operates on our open lock file and does not affect other descriptors.
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        return Err(format!("another daemon owns {}", path.display()));
    }
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.file_type().is_socket() || metadata.uid() != uid {
                return Err(format!("refusing to replace a non-owned socket at {}", path.display()));
            }
            match UnixStream::connect(path) {
                Ok(_) => return Err(format!("another daemon is listening on {}", path.display())),
                Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => {}
                Err(e) => return Err(format!("check socket {}: {e}", path.display())),
            }
            std::fs::remove_file(path).map_err(|e| e.to_string())?;
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.to_string()),
    }
    let listener = UnixListener::bind(path).map_err(|e| format!("bind {}: {e}", path.display()))?;
    let metadata = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    let lock = SocketLock {
        path: path.to_path_buf(),
        device: metadata.dev(),
        inode: metadata.ino(),
        _file: file,
    };
    mode(path, 0o600)?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    // Allocate the shutdown channel during bind, before platform workers start.
    let (shutdown_read, shutdown_write) = UnixStream::pair().map_err(|e| e.to_string())?;
    Ok((
        Listener {
            socket: listener,
            shutdown_read,
            shutdown_write,
        },
        lock,
    ))
}
#[cfg(unix)]
fn mode(path: &Path, n: u32) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(n)).map_err(|e| e.to_string())
}
#[cfg(unix)]
pub fn serve(
    listener: Listener,
    _path: PathBuf,
    tx: SyncSender<IoCommand>,
    waker: Arc<mio::Waker>,
    bus: Arc<EventBus>,
    stop: Arc<AtomicBool>,
) -> ServerThread {
    let Listener {
        socket: listener,
        shutdown_read,
        shutdown_write,
    } = listener;
    let thread = std::thread::spawn(move || {
        let mut clients = Vec::new();
        loop {
            if stop.load(Ordering::Relaxed) {
                break;
            }
            let mut fds = [
                libc::pollfd {
                    fd: listener.as_raw_fd(),
                    events: libc::POLLIN,
                    revents: 0,
                },
                libc::pollfd {
                    fd: shutdown_read.as_raw_fd(),
                    events: libc::POLLIN,
                    revents: 0,
                },
            ];
            // Safety: both descriptors remain owned by this thread. Input or
            // closure of the private shutdown peer wakes the blocking poll.
            let ready = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as _, -1) };
            if ready < 0 && std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            if fds[1].revents != 0 {
                stop.store(true, Ordering::Relaxed);
                let _ = waker.wake();
                break;
            }
            if ready < 0 || fds[0].revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 {
                log::error!("IPC listener failed");
                stop.store(true, Ordering::Relaxed);
                let _ = waker.wake();
                break;
            }
            let stream = match listener.accept() {
                Ok((stream, _)) => stream,
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                    ) =>
                {
                    continue
                }
                Err(e) => {
                    log::error!("IPC accept: {e}");
                    stop.store(true, Ordering::Relaxed);
                    let _ = waker.wake();
                    break;
                }
            };
            if stop.load(Ordering::Relaxed) {
                break;
            }
            clients.retain(|h: &std::thread::JoinHandle<()>| !h.is_finished());
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
        for h in clients {
            let _ = h.join();
        }
    });
    ServerThread {
        thread,
        shutdown: shutdown_write,
    }
}
#[cfg(windows)]
pub fn serve(
    mut listener: crate::windows_pipe::Listener,
    _path: PathBuf,
    tx: SyncSender<IoCommand>,
    waker: Arc<mio::Waker>,
    bus: Arc<EventBus>,
    stop: Arc<AtomicBool>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut clients = Vec::new();
        while !stop.load(Ordering::Relaxed) {
            let stream = match listener.accept(&stop) {
                Ok(stream) => stream,
                Err(_) if stop.load(Ordering::Relaxed) => break,
                Err(e) => {
                    log::error!("IPC accept: {e}");
                    stop.store(true, Ordering::Relaxed);
                    break;
                }
            };
            clients.retain(|h: &std::thread::JoinHandle<()>| !h.is_finished());
            if clients.len() >= 64 {
                continue;
            }
            let (tx, waker, bus, stop) = (tx.clone(), waker.clone(), bus.clone(), stop.clone());
            clients.push(std::thread::spawn(move || client(stream, tx, waker, bus, stop)));
        }
        for h in clients {
            let _ = h.join();
        }
    })
}
fn client(
    mut stream: UnixStream,
    tx: SyncSender<IoCommand>,
    waker: Arc<mio::Waker>,
    bus: Arc<EventBus>,
    stop: Arc<AtomicBool>,
) {
    #[cfg(unix)]
    {
        if stream.set_nonblocking(false).is_err() {
            return;
        }
        let _ = stream.set_read_timeout(Some(Duration::from_millis(50)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
    }
    let queue = bus.client();
    let mut buf = Vec::new();
    let mut scratch = [0u8; 8192];
    // Empty first requests and incomplete frames must not hold all 64 client
    // slots forever. Completed requests may idle indefinitely for subscriptions.
    // The deadline is absolute: additional partial bytes do not extend it.
    let mut request_deadline = Some(Instant::now() + REQUEST_READ_TIMEOUT);
    while !stop.load(Ordering::Relaxed) {
        if request_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            return;
        }
        match stream.read(&mut scratch) {
            Ok(0) => return,
            Ok(n) => {
                request_deadline.get_or_insert_with(|| Instant::now() + REQUEST_READ_TIMEOUT);
                buf.extend_from_slice(&scratch[..n]);
            }
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut | std::io::ErrorKind::Interrupted
                ) => {}
            Err(_) => return,
        }
        while let Some(h) = buf.get(..4) {
            let Ok(header) = <[u8; 4]>::try_from(h) else { return };
            let n = u32::from_be_bytes(header) as usize;
            if n > MAX_FRAME_LEN {
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
            let response = {
                let started = std::time::Instant::now();
                loop {
                    let result = answer.recv_timeout(Duration::from_millis(50));
                    if !matches!(result, Err(mpsc::RecvTimeoutError::Timeout))
                        || stop.load(Ordering::Relaxed)
                        || started.elapsed() >= Duration::from_secs(10)
                    {
                        break result;
                    }
                }
            };
            let Ok(response) = response else {
                return;
            };
            let Ok(value) = serde_json::to_value(&response) else {
                return;
            };
            if let Err(error) = write_frame(&mut stream, &value) {
                if error.kind() != std::io::ErrorKind::InvalidData {
                    return;
                }
                let error = Response::err(
                    response.id,
                    IpcError::new(
                        "response_too_large",
                        "IPC response exceeds the frame limit; request a smaller result or a summary",
                    ),
                );
                let Ok(value) = serde_json::to_value(error) else { return };
                if write_frame(&mut stream, &value).is_err() {
                    return;
                }
            }
            // Start the next frame's read budget after the preceding response;
            // time spent handling that request is not charged to its successor.
            request_deadline = (!buf.is_empty()).then(|| Instant::now() + REQUEST_READ_TIMEOUT);
        }
        for ev in queue.drain() {
            let Ok(value) = serde_json::to_value(ev) else { return };
            if write_frame(&mut stream, &value).is_err() {
                return;
            }
        }
    }
}
