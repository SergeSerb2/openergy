use crate::clipboard::X11Clipboard;
use openergy_platform::{ClipboardBackend, Permissions, PlatformError, ScreenRect, ScreenSystem};
use std::{
    io::{Read, Write},
    os::fd::AsRawFd,
    os::unix::net::UnixStream,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, MutexGuard,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
use x11rb::{
    connection::Connection,
    protocol::{
        randr::{ConnectionExt as _, NotifyMask},
        screensaver::ConnectionExt as _,
        xproto::ConnectionExt as _,
        Event,
    },
    rust_connection::RustConnection,
};
pub fn error(e: impl std::fmt::Display) -> PlatformError {
    PlatformError::Failed(e.to_string())
}
pub fn connect(display: Option<&str>) -> Result<(RustConnection, u32), PlatformError> {
    let (c, index) = x11rb::connect(display).map_err(error)?;
    let root = c
        .setup()
        .roots
        .get(index)
        .ok_or_else(|| error("X11 screen is missing"))?
        .root;
    Ok((c, root))
}
pub fn wake(w: &UnixStream) {
    let _ = (&*w).write(&[1]);
}
pub fn drain(w: &mut UnixStream) {
    let mut b = [0; 64];
    while w.read(&mut b).is_ok_and(|n| n > 0) {}
}
/// Every x11rb connection error except a parse error means the server
/// connection itself is closed (the `XCB_CONN_CLOSED_*` family plus I/O).
/// A parse error consumed one bad event and the stream stays usable.
/// Workers must route fatal errors into reconnect handling: a dead
/// TCP-backed DISPLAY can stay readable with EOF, so `wait()` below may
/// never observe POLLHUP for it.
pub fn conn_fatal(e: &x11rb::errors::ConnectionError) -> bool {
    !matches!(e, x11rb::errors::ConnectionError::ParseError(_))
}
pub fn wait(c: &RustConnection, w: &UnixStream, timeout: i32) -> Result<(), PlatformError> {
    let mut fds = [
        libc::pollfd {
            fd: c.stream().as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: w.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        },
    ];
    loop {
        // Safety: fds points to two valid pollfd values; poll does not retain the pointer.
        let result = unsafe { libc::poll(fds.as_mut_ptr(), 2, timeout) };
        if result >= 0 {
            if fds
                .iter()
                .any(|fd| fd.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0)
            {
                return Err(error("X11 worker connection closed"));
            }
            return Ok(());
        }
        let e = std::io::Error::last_os_error();
        if e.kind() != std::io::ErrorKind::Interrupted {
            return Err(error(e));
        }
    }
}
/// True when the X socket reports hangup/error. Lets synchronous users
/// (the sink) distinguish a dead server from an ordinary X error.
pub fn conn_dead(c: &RustConnection) -> bool {
    let mut fd = libc::pollfd {
        fd: c.stream().as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    // Safety: fd is valid for the duration of this synchronous poll call.
    let result = unsafe { libc::poll(&mut fd, 1, 0) };
    result > 0 && fd.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0
}
/// Wait only on the wake fd (used while the X connection is down).
/// Command senders wake this through the paired stream, so Stop and
/// state changes stay responsive during reconnect backoff.
pub fn wait_wake(w: &UnixStream, timeout: Duration) {
    let mut fd = libc::pollfd {
        fd: w.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    let ms = timeout.as_millis().min(i32::MAX as u128) as i32;
    loop {
        // Safety: fd is valid for the duration of this synchronous poll call.
        let result = unsafe { libc::poll(&mut fd, 1, ms) };
        if result >= 0 || std::io::Error::last_os_error().kind() != std::io::ErrorKind::Interrupted {
            return;
        }
    }
}
/// Bounded exponential backoff for supervised X reconnects.
pub struct Backoff {
    delay: Duration,
}
impl Backoff {
    pub fn new() -> Self {
        Self {
            delay: Duration::from_millis(100),
        }
    }
    pub fn next_delay(&mut self) -> Duration {
        let delay = self.delay;
        self.delay = (self.delay * 2).min(Duration::from_secs(2));
        delay
    }
    pub fn reset(&mut self) {
        self.delay = Duration::from_millis(100);
    }
}
impl Default for Backoff {
    fn default() -> Self {
        Self::new()
    }
}
// None means the query failed outright (dead connection), as opposed to a
// successful reply. Callers serve last-known values instead of masking a
// dead server as an empty display set or a (0, 0) cursor.
fn query_screens(c: &RustConnection, root: u32) -> Option<Vec<ScreenRect>> {
    if let Some(reply) = c.randr_get_monitors(root, true).ok().and_then(|r| r.reply().ok()) {
        let screens: Vec<_> = reply
            .monitors
            .iter()
            .filter(|m| m.width > 0 && m.height > 0)
            .map(|m| ScreenRect {
                x: m.x.into(),
                y: m.y.into(),
                w: m.width.into(),
                h: m.height.into(),
                scale: 1.0,
            })
            .collect();
        if !screens.is_empty() {
            return Some(screens);
        }
    }
    c.get_geometry(root).ok().and_then(|r| r.reply().ok()).map(|r| {
        vec![ScreenRect {
            x: r.x.into(),
            y: r.y.into(),
            w: r.width.into(),
            h: r.height.into(),
            scale: 1.0,
        }]
    })
}
fn query_cursor(c: &RustConnection, root: u32) -> Option<(i32, i32)> {
    c.query_pointer(root)
        .ok()
        .and_then(|r| r.reply().ok())
        .map(|r| (r.root_x.into(), r.root_y.into()))
}
struct ConnState {
    c: RustConnection,
    root: u32,
    last_screens: Vec<ScreenRect>,
    last_cursor: (i32, i32),
    alive: bool,
    retry_at: Instant,
    backoff: Backoff,
}
impl ConnState {
    fn note_alive(&mut self) {
        if !self.alive {
            log::info!("X11 screens connection recovered");
        }
        self.alive = true;
        self.backoff.reset();
    }
    fn note_dead(&mut self) {
        // Schedule the retry only on the alive -> dead transition. Doing it
        // on every failed poll would keep pushing retry_at forward and a
        // frequently polled Screens could starve its own reconnect forever.
        if self.alive {
            log::error!("X11 screens connection lost; serving last-known displays until reconnect");
            self.alive = false;
            self.retry_at = Instant::now() + self.backoff.next_delay();
        }
    }
    // Reconnect when due. Returns true when a fresh connection is installed.
    fn try_reconnect(&mut self, display: Option<&str>) -> bool {
        if self.alive || Instant::now() < self.retry_at {
            return false;
        }
        match connect(display) {
            Ok((c, root)) => {
                self.c = c;
                self.root = root;
                log::info!("X11 screens reconnected");
                true
            }
            Err(e) => {
                log::warn!("X11 screens reconnect failed: {e}");
                self.retry_at = Instant::now() + self.backoff.next_delay();
                false
            }
        }
    }
}
pub struct Screens {
    state: Mutex<ConnState>,
    clipboard: X11Clipboard,
    display: Option<String>,
    monitor: Option<(Arc<AtomicBool>, UnixStream, JoinHandle<()>)>,
}
impl Screens {
    pub fn new(display: Option<&str>, clipboard: X11Clipboard) -> Result<Self, PlatformError> {
        let (c, root) = connect(display)?;
        // Seed last-known values so a later outage never has to invent an
        // empty display set or a (0, 0) cursor.
        let last_screens = query_screens(&c, root).unwrap_or_default();
        let last_cursor = query_cursor(&c, root).unwrap_or((0, 0));
        Ok(Self {
            state: Mutex::new(ConnState {
                c,
                root,
                last_screens,
                last_cursor,
                alive: true,
                retry_at: Instant::now(),
                backoff: Backoff::new(),
            }),
            clipboard,
            display: display.map(str::to_owned),
            monitor: None,
        })
    }
    fn lock(&self) -> MutexGuard<'_, ConnState> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }
    fn stop_monitor(&mut self) {
        if let Some((stop, w, h)) = self.monitor.take() {
            stop.store(true, Ordering::Release);
            wake(&w);
            let _ = h.join();
        }
    }
}
fn monitor_connect(display: Option<&str>) -> Result<(RustConnection, u32), PlatformError> {
    let (c, root) = connect(display)?;
    c.randr_select_input(
        root,
        NotifyMask::SCREEN_CHANGE | NotifyMask::CRTC_CHANGE | NotifyMask::OUTPUT_CHANGE,
    )
    .map_err(error)?
    .check()
    .map_err(error)?;
    c.flush().map_err(error)?;
    Ok((c, root))
}
impl ScreenSystem for Screens {
    fn screens(&self) -> Vec<ScreenRect> {
        let mut state = self.lock();
        if let Some(screens) = query_screens(&state.c, state.root) {
            state.note_alive();
            state.last_screens = screens.clone();
            return screens;
        }
        state.note_dead();
        if state.try_reconnect(self.display.as_deref()) {
            if let Some(screens) = query_screens(&state.c, state.root) {
                state.note_alive();
                state.last_screens = screens.clone();
                return screens;
            }
            state.retry_at = Instant::now() + state.backoff.next_delay();
        }
        // The trait has no error surface. Last-known displays are honest
        // stale data; an empty set would pose as a real headless server.
        state.last_screens.clone()
    }
    fn cursor_pos(&self) -> (i32, i32) {
        let mut state = self.lock();
        if let Some(pos) = query_cursor(&state.c, state.root) {
            state.note_alive();
            state.last_cursor = pos;
            return pos;
        }
        state.note_dead();
        if state.try_reconnect(self.display.as_deref()) {
            if let Some(pos) = query_cursor(&state.c, state.root) {
                state.note_alive();
                state.last_cursor = pos;
                return pos;
            }
            state.retry_at = Instant::now() + state.backoff.next_delay();
        }
        state.last_cursor
    }
    fn clipboard(&mut self) -> &mut dyn ClipboardBackend {
        &mut self.clipboard
    }
    fn permissions(&self) -> Permissions {
        Permissions::not_required()
    }
    fn on_display_changed(&mut self, mut cb: Box<dyn FnMut() + Send>) {
        self.stop_monitor();
        let display = self.display.clone();
        let result = (|| -> Result<_, PlatformError> {
            let (c, _) = monitor_connect(display.as_deref())?;
            let (w, mut r) = UnixStream::pair().map_err(error)?;
            w.set_nonblocking(true).map_err(error)?;
            r.set_nonblocking(true).map_err(error)?;
            let stop = Arc::new(AtomicBool::new(false));
            let flag = stop.clone();
            let h = std::thread::Builder::new()
                .name("oe-x11-displays".into())
                .spawn(move || {
                    let mut c = c;
                    'run: while !flag.load(Ordering::Acquire) {
                        loop {
                            let mut dead = false;
                            loop {
                                match c.poll_for_event() {
                                    Ok(Some(ev)) => {
                                        if matches!(ev, Event::RandrScreenChangeNotify(_) | Event::RandrNotify(_)) {
                                            cb();
                                        }
                                    }
                                    Ok(None) => break,
                                    Err(e) => {
                                        // A parse error consumed one bad
                                        // event; anything else is a dead
                                        // connection (see conn_fatal).
                                        if conn_fatal(&e) {
                                            dead = true;
                                            break;
                                        }
                                    }
                                }
                            }
                            if dead || wait(&c, &r, -1).is_err() {
                                break;
                            }
                            drain(&mut r);
                            if flag.load(Ordering::Acquire) {
                                break 'run;
                            }
                        }
                        if flag.load(Ordering::Acquire) {
                            break;
                        }
                        log::error!("X11 display monitor connection lost; reconnecting");
                        let mut backoff = Backoff::new();
                        loop {
                            wait_wake(&r, backoff.next_delay());
                            drain(&mut r);
                            if flag.load(Ordering::Acquire) {
                                break 'run;
                            }
                            match monitor_connect(display.as_deref()) {
                                Ok((next, _)) => {
                                    c = next;
                                    log::info!("X11 display monitor reconnected");
                                    // The display set may differ across a
                                    // server restart; notify once.
                                    cb();
                                    continue 'run;
                                }
                                Err(e) => log::warn!("X11 display monitor reconnect failed: {e}"),
                            }
                        }
                    }
                })
                .map_err(error)?;
            Ok((stop, w, h))
        })();
        match result {
            Ok(m) => self.monitor = Some(m),
            Err(e) => log::warn!("X11 display notifications unavailable: {e}"),
        }
    }
    fn prevent_sleep(&mut self, on: bool) {
        let mut state = self.lock();
        let result = state
            .c
            .screensaver_suspend(u32::from(on))
            .map_err(error)
            .and_then(|c| c.check().map_err(error));
        if let Err(e) = result {
            log::warn!("X11 screensaver inhibition failed: {e}");
            state.note_dead();
        }
    }
}
impl Drop for Screens {
    fn drop(&mut self) {
        self.stop_monitor();
    }
}
