//! Focus Lab command adapter. LIVE serve requires an explicit pointer acknowledgement.
//! One private Unix socket service owns one real InputSink for the whole lab run.
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
static STOPPING: AtomicBool = AtomicBool::new(false);
extern "C" fn stop_signal(_: i32) {
    STOPPING.store(true, Ordering::Relaxed);
}
struct ActiveSink(Box<dyn InputSink>);
impl Drop for ActiveSink {
    fn drop(&mut self) {
        self.0.release_all();
    }
}
use openergy_platform::{InputEvent, InputSink};
use openergy_platform_macos::{self as mac, ClickNumbering, MacOptions};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
fn help() {
    println!("focuslab_inject serve --socket /private/path/input.sock --click-numbering native|legacy --i-understand-this-moves-the-pointer\nfocuslab_inject --socket /private/path/input.sock --click-numbering native|legacy move|down|up|drag left|right|middle x y\nfocuslab_inject --socket /private/path/input.sock stop");
}
fn run() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let mut socket = None;
    let mut mode = ClickNumbering::Native;
    let mut acknowledge = false;
    let mut positional = Vec::new();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => {
                help();
                return Ok(());
            }
            "--socket" => socket = Some(PathBuf::from(args.next().ok_or("--socket requires path")?)),
            "--click-numbering" => {
                mode = match args.next().as_deref() {
                    Some("native") => ClickNumbering::Native,
                    Some("legacy") => ClickNumbering::Legacy,
                    _ => return Err("--click-numbering requires native|legacy".into()),
                }
            }
            "--i-understand-this-moves-the-pointer" => acknowledge = true,
            _ => positional.push(arg),
        }
    }
    let path = socket.ok_or("--socket is required")?;
    if !path.is_absolute() {
        return Err("socket path must be absolute".into());
    }
    if positional == ["serve"] {
        if !acknowledge {
            return Err("serve requires --i-understand-this-moves-the-pointer on a separate test Mac".into());
        }
        return serve(&path, mode);
    }
    if positional == ["stop"] {
        return request(&path, "stop\n");
    }
    let action = Action::parse(&positional)?;
    request(
        &path,
        &format!(
            "{} {} {} {} {}\n",
            mode_name(mode),
            action.kind,
            action.button,
            action.x,
            action.y
        ),
    )
}
fn mode_name(mode: ClickNumbering) -> &'static str {
    match mode {
        ClickNumbering::Native => "native",
        ClickNumbering::Legacy => "legacy",
    }
}
struct Action {
    kind: String,
    button: u8,
    x: i32,
    y: i32,
}
impl Action {
    fn parse(args: &[String]) -> Result<Self> {
        if args.len() != 4 || !matches!(args[0].as_str(), "move" | "down" | "up" | "drag") {
            return Err("expected move|down|up|drag left|right|middle x y".into());
        }
        let button = match args[1].as_str() {
            "left" => 1,
            "middle" => 2,
            "right" => 3,
            _ => return Err("invalid mouse button".into()),
        };
        let coordinate = |text: &str| -> Result<i32> {
            let n: f64 = text.parse()?;
            if !n.is_finite() || n < f64::from(i32::MIN) || n > f64::from(i32::MAX) {
                return Err("invalid global coordinate".into());
            }
            // Frozen InputEvent uses integer global points. Focus Lab supplies
            // decimals; round once at this boundary (<=0.5pt, under its 2pt gate).
            Ok(n.round() as i32)
        };
        Ok(Self {
            kind: args[0].clone(),
            button,
            x: coordinate(&args[2])?,
            y: coordinate(&args[3])?,
        })
    }
}
fn request(path: &Path, message: &str) -> Result<()> {
    let mut stream = UnixStream::connect(path)?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    stream.write_all(message.as_bytes())?;
    let mut response = String::new();
    BufReader::new(stream).take(1024).read_line(&mut response)?;
    if response.trim() != "ok" {
        return Err(format!("injector service rejected request: {}", response.trim()).into());
    }
    Ok(())
}
struct SocketGuard(PathBuf);
impl Drop for SocketGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
fn serve(path: &Path, mode: ClickNumbering) -> Result<()> {
    let parent = path.parent().ok_or("socket requires parent")?;
    let metadata = parent.metadata()?;
    // Socket directory must already be a private mktemp -d directory. Never
    // unlink a stale/existing socket: a running service must not be displaced.
    // MetadataExt::uid and peer credentials restrict this to the current user.
    let uid = metadata.uid();
    if metadata.permissions().mode() & 0o077 != 0 {
        return Err("socket directory must have mode 0700".into());
    }
    let perms = mac::permissions::preflight();
    if !perms.accessibility || !perms.post {
        return Err("grant injector Accessibility/Post Event access on the TEST Mac first".into());
    }
    let listener = UnixListener::bind(path)?;
    let _guard = SocketGuard(path.to_owned());
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    let backend = mac::create(MacOptions {
        click_numbering: mode,
        ..MacOptions::default()
    })?;
    let mut sink = ActiveSink(backend.sink);
    listener.set_nonblocking(true)?;
    unsafe extern "C" {
        fn signal(sig: i32, handler: extern "C" fn(i32)) -> usize;
    }
    // Safety: handler only sets a lock-free atomic; server owns process signal handling.
    unsafe {
        signal(2, stop_signal);
        signal(15, stop_signal);
    }
    let mut held = [false; 6];
    println!(
        "ready mode={} socket={} pid={}; first synthetic down has not occurred",
        mode_name(mode),
        path.display(),
        std::process::id()
    );
    // No TCP/LAN, event tap, cursor hide/warp, or power assertion in this adapter.
    while !STOPPING.load(Ordering::Relaxed) {
        let mut stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                wait_for_client(&listener)?;
                continue;
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => break,
        };
        stream.set_read_timeout(Some(Duration::from_secs(1)))?;
        stream.set_write_timeout(Some(Duration::from_secs(1)))?;
        if peer_uid(&stream)? != uid {
            continue;
        }
        let mut line = String::new();
        let result = BufReader::new(&stream).take(256).read_line(&mut line);
        if result.is_err() || !line.ends_with('\n') {
            let _ = stream.write_all(b"error malformed request\n");
            continue;
        }
        if line == "stop\n" {
            sink.0.release_all();
            stream.write_all(b"ok\n")?;
            return Ok(());
        }
        let result = (|| -> Result<()> {
            let fields: Vec<String> = line.split_whitespace().map(str::to_owned).collect();
            if fields.len() != 5 || fields[0] != mode_name(mode) {
                return Err("click-numbering mode mismatch".into());
            }
            let button_name = match fields[2].as_str() {
                "1" => "left",
                "2" => "middle",
                "3" => "right",
                _ => return Err("invalid button".into()),
            };
            let action = Action::parse(&[
                fields[1].clone(),
                button_name.into(),
                fields[3].clone(),
                fields[4].clone(),
            ])?;
            let index = usize::from(action.button);
            match action.kind.as_str() {
                "move" if held.iter().any(|b| *b) => return Err("move while button held; use drag".into()),
                "down" if held[index] => return Err("duplicate down".into()),
                "drag" if !held[index] => return Err("drag without down".into()),
                // Failed downs can trigger an unmatched cleanup up from Focus Lab.
                "up" if !held[index] => return Ok(()),
                _ => {}
            }
            // A down/up at an already reached point does not emit another move.
            // Source mouse coordinates are in the same integer point space.
            if action.kind == "move" || action.kind == "drag" || backend.screens.cursor_pos() != (action.x, action.y) {
                sink.0.inject(&InputEvent::MouseMoveAbs {
                    x: action.x,
                    y: action.y,
                })?;
            }
            match action.kind.as_str() {
                "down" => {
                    sink.0.inject(&InputEvent::MouseDown { button: action.button })?;
                    held[index] = true;
                }
                "up" => {
                    sink.0.inject(&InputEvent::MouseUp { button: action.button })?;
                    held[index] = false;
                }
                _ => {}
            }
            Ok(())
        })();
        if result.is_ok() {
            let _ = stream.write_all(b"ok\n");
        } else {
            sink.0.release_all();
            held = [false; 6];
            let _ = stream.write_all(b"error input request rejected; held inputs released\n");
        }
    }
    sink.0.release_all();
    if STOPPING.load(Ordering::Relaxed) {
        Ok(())
    } else {
        Err("injector listener stopped".into())
    }
}
fn peer_uid(stream: &UnixStream) -> Result<u32> {
    use std::os::fd::AsRawFd;
    unsafe extern "C" {
        fn getpeereid(fd: i32, uid: *mut u32, gid: *mut u32) -> i32;
    }
    let (mut uid, mut gid) = (0, 0);
    // Safety: socket fd and both output pointers remain live for this syscall.
    if unsafe { getpeereid(stream.as_raw_fd(), &mut uid, &mut gid) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(uid)
}

fn wait_for_client(listener: &UnixListener) -> Result<()> {
    use std::os::fd::AsRawFd;
    #[repr(C)]
    struct PollFd {
        fd: i32,
        events: i16,
        revents: i16,
    }
    unsafe extern "C" {
        fn poll(fds: *mut PollFd, count: u32, timeout: i32) -> i32;
    }
    let mut fd = PollFd {
        fd: listener.as_raw_fd(),
        events: 1,
        revents: 0,
    };
    // Safety: one valid socket descriptor and one initialized output record.
    if unsafe { poll(&mut fd, 1, 1000) } < 0 {
        let err = std::io::Error::last_os_error();
        if err.kind() != std::io::ErrorKind::Interrupted {
            return Err(err.into());
        }
    }
    Ok(())
}
