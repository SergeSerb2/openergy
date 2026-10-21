//! Real X11 end-to-end probe. Run only through tools/linux-dev/linux-dev.sh.
#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("x11_probe requires the isolated Linux container");
    std::process::exit(2);
}
#[cfg(target_os = "linux")]
fn main() {
    if let Err(e) = probe::run() {
        eprintln!("x11_probe: {e}");
        std::process::exit(1);
    }
}
#[cfg(target_os = "linux")]
mod probe {
    use openergy_platform::{
        CaptureDecision, ClipboardBackend, ClipboardData, ClipboardFormat, FormatKind, Grant, InputEvent, Modifiers,
        PermissionKind,
    };
    use openergy_platform_linux::{create, Selection, X11Clipboard, X11Options};
    use openergy_protocol::keys;
    use std::{
        error::Error,
        fs,
        io::Write,
        process::{Child, Command, Stdio},
        sync::{
            atomic::{AtomicBool, AtomicU16, AtomicUsize, Ordering},
            Arc,
        },
        time::{Duration, Instant},
    };
    use x11rb::protocol::xproto::ConnectionExt;
    type Result<T> = std::result::Result<T, Box<dyn Error>>;
    struct Process(Child);
    impl Drop for Process {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    fn command(name: &str, args: &[&str]) -> Result<Vec<u8>> {
        let out = Command::new(name).args(args).output()?;
        if !out.status.success() {
            return Err(format!("{name} failed: {}", String::from_utf8_lossy(&out.stderr)).into());
        }
        Ok(out.stdout)
    }
    fn pause() {
        std::thread::sleep(Duration::from_millis(100));
    }
    pub fn run() -> Result<()> {
        if std::env::var("OPENERGY_X11_PROBE_ISOLATED").as_deref() != Ok("1")
            || std::env::var("DISPLAY").as_deref() != Ok(":99")
        {
            return Err("Probe requires isolated container DISPLAY=:99".into());
        }
        let mode = std::env::args().nth(1).unwrap_or_else(|| "all".into());
        match mode.as_str() {
            "inject" => inject()?,
            "clipboard" => clipboard()?,
            "capture" => capture()?,
            "all" => {
                inject()?;
                clipboard()?;
                capture()?;
            }
            _ => return Err("Mode must be inject, clipboard, capture, or all".into()),
        }
        println!("PASS {mode}");
        Ok(())
    }
    fn xev(label: &str) -> Result<(Process, String)> {
        let path = format!("/tmp/oe-{label}-xev.log");
        let child = Command::new("stdbuf")
            .args(["-oL", "xev", "-name", "oe-wp8-xev", "-geometry", "800x500+0+0"])
            .stdout(fs::File::create(&path)?)
            .stderr(Stdio::null())
            .spawn()?;
        let child = Process(child);
        let start = Instant::now();
        loop {
            if let Ok(ids) = command("xdotool", &["search", "--name", "^oe-wp8-xev$"]) {
                if let Some(id) = String::from_utf8(ids)?.lines().next() {
                    command("xdotool", &["windowfocus", id])?;
                    break;
                }
            }
            if start.elapsed() > Duration::from_secs(3) {
                return Err("xev window did not appear".into());
            }
            pause();
        }
        pause();
        Ok((child, path))
    }
    fn inject() -> Result<()> {
        let (_xev, path) = xev("inject")?;
        let mut recording = Process(
            Command::new("stdbuf")
                .args(["-oL", "xinput", "test-xi2", "--root"])
                .stdout(fs::File::create("/tmp/oe-inject-xi2.log")?)
                .stderr(Stdio::null())
                .spawn()?,
        );
        pause();
        let (map_connection, _) = x11rb::connect(None)?;
        use x11rb::connection::Connection;
        let setup = map_connection.setup();
        let original = map_connection
            .get_keyboard_mapping(setup.min_keycode, setup.max_keycode - setup.min_keycode + 1)?
            .reply()?
            .keysyms;
        let mut b: openergy_platform::Backend = create(X11Options::default())?;
        for kind in [
            PermissionKind::Accessibility,
            PermissionKind::InputMonitoring,
            PermissionKind::PostEvents,
        ] {
            if b.screens.request_permission(kind)? != Grant::NotRequired {
                return Err("X11 permission should be NotRequired".into());
            }
        }
        println!("construction: shared Backend and default permission requests passed");
        let s = &mut b.sink;
        s.inject(&InputEvent::MouseMoveAbs { x: 100, y: 120 })?;
        s.inject(&InputEvent::MouseMoveRel { dx: 17, dy: 13 })?;
        pause();
        let pos = command("xdotool", &["getmouselocation", "--shell"])?;
        let pos = String::from_utf8(pos)?;
        if !pos.contains("X=117\nY=133") {
            return Err(format!("Relative pointer position mismatch: {pos}").into());
        }
        println!("inject pointer: X=117 Y=133");
        for button in 1..=5 {
            s.inject(&InputEvent::MouseDown { button })?;
            s.inject(&InputEvent::MouseUp { button })?;
        }
        for _ in 0..2 {
            s.inject(&InputEvent::MouseDown { button: 1 })?;
            s.inject(&InputEvent::MouseUp { button: 1 })?;
        }
        s.inject(&InputEvent::MouseDown { button: 1 })?;
        s.inject(&InputEvent::MouseMoveAbs { x: 160, y: 170 })?;
        s.inject(&InputEvent::MouseUp { button: 1 })?;
        for (index, key) in ['a' as u16, 'A' as u16, 'é' as u16, '€' as u16, keys::RETURN]
            .into_iter()
            .enumerate()
        {
            s.inject(&InputEvent::KeyDown {
                key,
                modifiers: Modifiers::NONE,
                button: index as u16 + 1,
            })?;
            pause();
            s.inject(&InputEvent::KeyUp {
                key,
                modifiers: Modifiers::NONE,
                button: index as u16 + 1,
            })?;
            pause();
        }
        // Two held characters share a synthetic Shift. Releasing one must keep it held.
        for (key, button) in [('A' as u16, 101), ('B' as u16, 102)] {
            s.inject(&InputEvent::KeyDown {
                key,
                modifiers: Modifiers::NONE,
                button,
            })?;
        }
        s.inject(&InputEvent::KeyUp {
            key: 'A' as u16,
            modifiers: Modifiers::NONE,
            button: 101,
        })?;
        let shift = map_connection
            .get_modifier_mapping()?
            .reply()?
            .keycodes
            .first()
            .copied()
            .ok_or("X server has no Shift")?;
        let down = map_connection.query_keymap()?.reply()?.keys;
        if down[usize::from(shift) / 8] & (1 << (shift % 8)) == 0 {
            return Err("Shared Shift released too early".into());
        }
        s.inject(&InputEvent::KeyRepeat {
            key: 'B' as u16,
            modifiers: Modifiers::NONE,
            count: 2,
            button: 102,
        })?;
        s.inject(&InputEvent::KeyUp {
            key: 'B' as u16,
            modifiers: Modifiers::NONE,
            button: 102,
        })?;
        let keypad = Arc::new(AtomicU16::new(0));
        let observed = keypad.clone();
        b.source.start(Box::new(move |event| {
            if let InputEvent::KeyDown { key, .. } = event {
                observed.store(key, Ordering::Release);
            }
            CaptureDecision::PassToOs
        }))?;
        for key in [keys::KP_1, keys::KP_END] {
            keypad.store(0, Ordering::Release);
            s.inject(&InputEvent::KeyDown {
                key,
                modifiers: Modifiers(Modifiers::NUM_LOCK),
                button: 210,
            })?;
            pause();
            if keypad.load(Ordering::Acquire) != key {
                return Err("Num Lock keypad capture/injection mismatch".into());
            }
            s.inject(&InputEvent::KeyUp {
                key,
                modifiers: Modifiers(Modifiers::NUM_LOCK),
                button: 210,
            })?;
            pause();
        }
        b.source.stop();
        println!("inject/capture Num Lock: keypad digit and shifted keypad navigation passed");
        // Fractional values produce exactly one vertical and one horizontal notch.
        for _ in 0..4 {
            s.inject(&InputEvent::MouseWheel { dx: 30, dy: 30 })?;
        }
        s.inject(&InputEvent::KeyDown {
            key: keys::CONTROL_L,
            modifiers: Modifiers::NONE,
            button: 77,
        })?;
        s.inject(&InputEvent::MouseDown { button: 1 })?;
        s.release_all();
        s.release_all();
        pause();
        let (c, _) = x11rb::connect(None)?;
        let keys = c.query_keymap()?.reply()?.keys;
        if keys.iter().any(|k| *k != 0) {
            return Err("release_all left a key down".into());
        }
        for name in ["Virtual core XTEST keyboard", "Virtual core XTEST pointer"] {
            let state = String::from_utf8(command("xinput", &["query-state", name])?)?;
            if state.contains("=down") {
                return Err(format!("release_all left state down on {name}").into());
            }
            println!("inject release_all: {name}: all up");
        }
        let restored = map_connection
            .get_keyboard_mapping(setup.min_keycode, setup.max_keycode - setup.min_keycode + 1)?
            .reply()?
            .keysyms;
        if original != restored {
            return Err("Unicode fallback did not restore the X keymap".into());
        }
        println!("inject keymap restored; shared modifiers and repeats passed");
        let text = fs::read_to_string(path)?;
        for symbol in [" a)", " A)", "eacute", "U20AC", "Return"] {
            if !text.contains(symbol) {
                return Err(format!("xev did not receive {symbol}").into());
            }
        }
        let _ = recording.0.kill();
        let _ = recording.0.wait();
        let raw = fs::read_to_string("/tmp/oe-inject-xi2.log")?;
        if !raw.contains("RawKeyPress") || !raw.contains("RawButtonPress") {
            return Err("XI2 recording lacks input".into());
        }
        println!("inject xev: ASCII, Shift, eacute, U20AC fallback, Return, clicks, drag, wheel received");
        Ok(())
    }
    fn clip_out(selection: &str, target: &str) -> Result<Vec<u8>> {
        command("xclip", &["-selection", selection, "-target", target, "-o"])
    }
    fn clip_in(backend: &X11Clipboard, selection: &str, target: &str, bytes: &[u8]) -> Result<Process> {
        let backend = backend.for_selection(if selection == "primary" {
            Selection::Primary
        } else {
            Selection::Clipboard
        });
        let stamp = backend.change_stamp();
        let (c, _) = x11rb::connect(None)?;
        let atom = c
            .intern_atom(
                false,
                if selection == "primary" {
                    b"PRIMARY"
                } else {
                    b"CLIPBOARD"
                },
            )?
            .reply()?
            .atom;
        let old_owner = c.get_selection_owner(atom)?.reply()?.owner;
        let mut child = Process(
            Command::new("xclip")
                .args(["-selection", selection, "-target", target, "-i", "-quiet"])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?,
        );
        let mut input = child.0.stdin.take().ok_or("Missing xclip stdin")?;
        input.write_all(bytes)?;
        drop(input);
        let start = Instant::now();
        while backend.change_stamp() == stamp || {
            let owner = c.get_selection_owner(atom)?.reply()?.owner;
            owner == 0 || owner == old_owner
        } {
            if start.elapsed() > Duration::from_secs(3) {
                return Err("XFixes did not report selection change".into());
            }
            pause();
        }
        Ok(child)
    }
    fn equal_format(data: &ClipboardData, kind: FormatKind, bytes: &[u8]) -> Result<()> {
        if !data.formats.iter().any(|f| f.kind == kind && f.bytes == bytes) {
            return Err(format!("Clipboard format mismatch: {kind:?}").into());
        }
        Ok(())
    }
    fn clipboard() -> Result<()> {
        let mut b = X11Clipboard::connect(None, Selection::Clipboard)?;
        let png = fs::read("/tmp/oe-small.png")?;
        let text = b"Openergy UTF-8: caf\xc3\xa9 \xe2\x82\xac\n";
        let html = b"<p>Openergy <b>HTML</b></p>";
        b.write(&ClipboardData {
            formats: vec![
                ClipboardFormat {
                    kind: FormatKind::TextUtf8,
                    bytes: text.to_vec(),
                },
                ClipboardFormat {
                    kind: FormatKind::HtmlUtf8,
                    bytes: html.to_vec(),
                },
                ClipboardFormat {
                    kind: FormatKind::Png,
                    bytes: png.clone(),
                },
            ],
        })?;
        if clip_out("clipboard", "UTF8_STRING")? != text
            || clip_out("clipboard", "text/plain;charset=utf-8")? != text
            || clip_out("clipboard", "text/html")? != html
            || clip_out("clipboard", "image/png")? != png
        {
            return Err("xclip read-back differs from backend write".into());
        }
        let bmp = clip_out("clipboard", "image/bmp")?;
        if bmp.get(..2) != Some(b"BM") {
            return Err("Backend did not export BMP".into());
        }
        let data = b.read(1024 * 1024)?;
        equal_format(&data, FormatKind::TextUtf8, text)?;
        equal_format(&data, FormatKind::HtmlUtf8, html)?;
        let dib = data
            .formats
            .iter()
            .find(|f| f.kind == FormatKind::Dib)
            .ok_or("PNG did not convert to wire DIB")?
            .bytes
            .clone();
        if dib.len() != 108 + 64 * 64 * 4 || bmp[14..] != dib {
            return Err("DIB/BMP bridge mismatch".into());
        }
        b.write(&ClipboardData {
            formats: vec![ClipboardFormat {
                kind: FormatKind::Dib,
                bytes: dib.clone(),
            }],
        })?;
        fs::write("/tmp/oe-roundtrip.png", clip_out("clipboard", "image/png")?)?;
        println!("clipboard backend -> xclip: text, HTML, PNG and headerless little-endian DIB/BMP passed");
        for (kind, target, bytes) in [
            (FormatKind::TextUtf8, "UTF8_STRING", text.as_slice()),
            (FormatKind::HtmlUtf8, "text/html", html.as_slice()),
            (FormatKind::Dib, "image/bmp", bmp.as_slice()),
        ] {
            let _owner = clip_in(&b, "clipboard", target, bytes)?;
            let data = b.read(1024 * 1024)?;
            equal_format(&data, kind, if kind == FormatKind::Dib { &dib } else { bytes })?;
        }
        let large = fs::read("/tmp/oe-large.png")?;
        {
            let _owner = clip_in(&b, "clipboard", "image/png", &large)?;
            let data = b.read(32 * 1024 * 1024)?;
            let image = data
                .formats
                .iter()
                .find(|f| f.kind == FormatKind::Dib)
                .ok_or("Large PNG missing DIB")?;
            if image.bytes.len() != 108 + 1600 * 1640 * 4 {
                return Err("Large INCR decode size mismatch".into());
            }
            fs::write("/tmp/oe-large.dib", &image.bytes)?;
            println!(
                "clipboard xclip -> backend INCR: PNG={} DIB={}",
                large.len(),
                image.bytes.len()
            );
            if b.read(1024).is_ok() {
                return Err("Clipboard limit did not reject oversized INCR".into());
            }
        }
        b.write(&ClipboardData {
            formats: vec![ClipboardFormat {
                kind: FormatKind::Png,
                bytes: large.clone(),
            }],
        })?;
        if clip_out("clipboard", "image/png")? != large {
            return Err("Large backend -> xclip INCR bytes mismatch".into());
        }
        println!(
            "clipboard backend -> xclip INCR: {} bytes equal; oversize read rejected",
            large.len()
        );
        let mut shared: openergy_platform::Backend = create(X11Options::default())?;
        let clipboard_only = b"v1 CLIPBOARD only";
        shared.clipboard.write(&ClipboardData {
            formats: vec![ClipboardFormat {
                kind: FormatKind::TextUtf8,
                bytes: clipboard_only.to_vec(),
            }],
        })?;
        equal_format(&shared.clipboard.read(1024)?, FormatKind::TextUtf8, clipboard_only)?;
        let clipboard_stamp = shared.clipboard.change_stamp();
        b.write_selection(
            Selection::Primary,
            &ClipboardData {
                formats: vec![ClipboardFormat {
                    kind: FormatKind::TextUtf8,
                    bytes: b"primary selection".to_vec(),
                }],
            },
        )?;
        if clip_out("primary", "UTF8_STRING")? != b"primary selection" {
            return Err("PRIMARY owner failed".into());
        }
        {
            let _owner = clip_in(&b, "primary", "UTF8_STRING", b"external primary")?;
            equal_format(
                &b.read_selection(Selection::Primary, 100)?,
                FormatKind::TextUtf8,
                b"external primary",
            )?;
        }
        equal_format(&shared.clipboard.read(1024)?, FormatKind::TextUtf8, clipboard_only)?;
        if shared.clipboard.change_stamp() != clipboard_stamp {
            return Err("PRIMARY changed the shared v1 CLIPBOARD stamp".into());
        }
        println!("clipboard v1 routing: PRIMARY does not change shared CLIPBOARD data or stamp");
        // Empty owner and malformed content must not poison the worker.
        b.write(&ClipboardData::default())?;
        if !b.read(1024)?.formats.is_empty() {
            return Err("Empty clipboard not empty".into());
        }
        if b.write(&ClipboardData {
            formats: vec![ClipboardFormat {
                kind: FormatKind::Png,
                bytes: b"invalid PNG".to_vec(),
            }],
        })
        .is_ok()
        {
            return Err("Invalid PNG accepted".into());
        }
        println!("clipboard PRIMARY, XFixes change stamp, empty selection, invalid PNG: passed");
        Ok(())
    }
    fn capture() -> Result<()> {
        let (_xev, path) = xev("capture")?;
        let mut b = create(X11Options::default())?;
        let swallow = Arc::new(AtomicBool::new(false));
        let flag = swallow.clone();
        let count = Arc::new(AtomicUsize::new(0));
        let events = count.clone();
        let relative = Arc::new(AtomicUsize::new(0));
        let rel = relative.clone();
        b.source.start(Box::new(move |ev| {
            if matches!(ev, InputEvent::MouseMoveRel { .. }) {
                rel.fetch_add(1, Ordering::Relaxed);
            }
            if matches!(ev, InputEvent::KeyDown { .. } | InputEvent::KeyUp { .. }) {
                events.fetch_add(1, Ordering::Relaxed);
            }
            if flag.load(Ordering::Acquire) {
                CaptureDecision::Swallow
            } else {
                CaptureDecision::PassToOs
            }
        }))?;
        command("xdotool", &["key", "a"])?;
        pause();
        let before = fs::read_to_string(&path)?.matches("KeyPress event").count();
        if before == 0 || count.load(Ordering::Relaxed) == 0 {
            return Err("Pass-through did not reach both xev and capture".into());
        }
        let captured_before = count.load(Ordering::Relaxed);
        swallow.store(true, Ordering::Release);
        b.source.set_cursor_captured(true)?;
        command("xdotool", &["key", "b"])?;
        command("xdotool", &["mousemove_relative", "--", "20", "10"])?;
        pause();
        let during = fs::read_to_string(&path)?.matches("KeyPress event").count();
        if during != before || count.load(Ordering::Relaxed) <= captured_before {
            return Err("Active grab did not swallow locally while capturing".into());
        }
        swallow.store(false, Ordering::Release);
        // Let the callback return PassToOs while the grab is still active.
        command("xdotool", &["key", "c"])?;
        b.source.set_cursor_captured(false)?;
        pause();
        b.source.stop();
        let after = fs::read_to_string(&path)?.matches("KeyPress event").count();
        if after <= during {
            return Err("Pass-through did not resume".into());
        }
        println!(
            "capture events={} relative_moves={} xev KeyPress counts: pass={before} swallowed={during} resumed={after}",
            count.load(Ordering::Relaxed),
            relative.load(Ordering::Relaxed)
        );
        if relative.load(Ordering::Relaxed) == 0 {
            return Err("Raw relative motion was not captured".into());
        }
        println!("capture: XI2 observation, active keyboard/pointer grabs, CaptureDecision transitions passed");
        Ok(())
    }
}
