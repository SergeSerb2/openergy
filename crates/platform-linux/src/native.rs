use crate::clipboard::X11Clipboard;
use openergy_platform::{ClipboardBackend, Permissions, PlatformError, ScreenRect, ScreenSystem};
use std::{
    io::{Read, Write},
    os::fd::AsRawFd,
    os::unix::net::UnixStream,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::JoinHandle,
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
pub struct Screens {
    c: RustConnection,
    root: u32,
    clipboard: X11Clipboard,
    display: Option<String>,
    monitor: Option<(Arc<AtomicBool>, UnixStream, JoinHandle<()>)>,
}
impl Screens {
    pub fn new(display: Option<&str>, clipboard: X11Clipboard) -> Result<Self, PlatformError> {
        let (c, root) = connect(display)?;
        Ok(Self {
            c,
            root,
            clipboard,
            display: display.map(str::to_owned),
            monitor: None,
        })
    }
    fn stop_monitor(&mut self) {
        if let Some((stop, w, h)) = self.monitor.take() {
            stop.store(true, Ordering::Release);
            wake(&w);
            let _ = h.join();
        }
    }
}
impl ScreenSystem for Screens {
    fn screens(&self) -> Vec<ScreenRect> {
        if let Ok(cookie) = self.c.randr_get_monitors(self.root, true) {
            if let Ok(reply) = cookie.reply() {
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
                    return screens;
                }
            }
        }
        self.c
            .get_geometry(self.root)
            .ok()
            .and_then(|r| r.reply().ok())
            .map(|r| {
                vec![ScreenRect {
                    x: r.x.into(),
                    y: r.y.into(),
                    w: r.width.into(),
                    h: r.height.into(),
                    scale: 1.0,
                }]
            })
            .unwrap_or_default()
    }
    fn cursor_pos(&self) -> (i32, i32) {
        self.c
            .query_pointer(self.root)
            .ok()
            .and_then(|r| r.reply().ok())
            .map(|r| (r.root_x.into(), r.root_y.into()))
            .unwrap_or((0, 0))
    }
    fn clipboard(&mut self) -> &mut dyn ClipboardBackend {
        &mut self.clipboard
    }
    fn permissions(&self) -> Permissions {
        Permissions::not_required()
    }
    fn on_display_changed(&mut self, mut cb: Box<dyn FnMut() + Send>) {
        self.stop_monitor();
        let result = (|| -> Result<_, PlatformError> {
            let (c, root) = connect(self.display.as_deref())?;
            c.randr_select_input(
                root,
                NotifyMask::SCREEN_CHANGE | NotifyMask::CRTC_CHANGE | NotifyMask::OUTPUT_CHANGE,
            )
            .map_err(error)?
            .check()
            .map_err(error)?;
            c.flush().map_err(error)?;
            let (w, mut r) = UnixStream::pair().map_err(error)?;
            w.set_nonblocking(true).map_err(error)?;
            r.set_nonblocking(true).map_err(error)?;
            let stop = Arc::new(AtomicBool::new(false));
            let flag = stop.clone();
            let h = std::thread::Builder::new()
                .name("oe-x11-displays".into())
                .spawn(move || {
                    'monitor: while !flag.load(Ordering::Acquire) {
                        loop {
                            let ev = match c.poll_for_event() {
                                Ok(Some(ev)) => ev,
                                Ok(None) => break,
                                Err(_) => break 'monitor,
                            };
                            if matches!(ev, Event::RandrScreenChangeNotify(_) | Event::RandrNotify(_)) {
                                cb();
                            }
                        }
                        if wait(&c, &r, -1).is_err() {
                            break;
                        }
                        drain(&mut r);
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
        let result = self
            .c
            .screensaver_suspend(u32::from(on))
            .map_err(error)
            .and_then(|c| c.check().map_err(error));
        if let Err(e) = result {
            log::warn!("X11 screensaver inhibition failed: {e}");
        }
    }
}
impl Drop for Screens {
    fn drop(&mut self) {
        self.stop_monitor();
    }
}
