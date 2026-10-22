use crate::{
    keymap::Keymap,
    native::{connect, drain, error, wait, wake},
};
use openergy_platform::{CaptureDecision, InputEvent, InputSource, PlatformError};
use std::{
    collections::BTreeMap,
    os::unix::net::UnixStream,
    sync::mpsc::{self, SyncSender},
    thread::JoinHandle,
};
use x11rb::{
    connection::Connection,
    protocol::{
        xfixes::ConnectionExt as _,
        xinput::{self, ConnectionExt as _},
        xkb::{self, ConnectionExt as _},
        xproto::{self, ConnectionExt as _},
        Event,
    },
    rust_connection::RustConnection,
    CURRENT_TIME, NONE,
};
type Callback = Box<dyn FnMut(InputEvent) -> CaptureDecision + Send>;
enum Command {
    Capture(bool, SyncSender<Result<(), PlatformError>>),
    Warp(i32, i32, SyncSender<Result<(), PlatformError>>),
    Stop,
}
pub struct Source {
    display: Option<String>,
    worker: Option<(SyncSender<Command>, UnixStream, JoinHandle<()>)>,
    captured: bool,
}
impl Source {
    pub fn new(display: Option<&str>) -> Result<Self, PlatformError> {
        let _ = connect(display)?;
        Ok(Self {
            display: display.map(str::to_owned),
            worker: None,
            captured: false,
        })
    }
    fn command(
        &self,
        build: impl FnOnce(SyncSender<Result<(), PlatformError>>) -> Command,
    ) -> Result<(), PlatformError> {
        let (tx, w, _) = self
            .worker
            .as_ref()
            .ok_or_else(|| error("X11 capture is not started"))?;
        let (reply, rx) = mpsc::sync_channel(1);
        tx.send(build(reply)).map_err(error)?;
        wake(w);
        rx.recv().map_err(error)?
    }
}
impl InputSource for Source {
    fn start(&mut self, callback: Callback) -> Result<(), PlatformError> {
        self.stop();
        let (c, root) = connect(self.display.as_deref())?;
        c.xinput_xi_query_version(2, 3).map_err(error)?.reply().map_err(error)?;
        c.xfixes_query_version(4, 0).map_err(error)?.reply().map_err(error)?;
        c.xinput_xi_select_events(
            root,
            &[xinput::EventMask {
                deviceid: 1,
                mask: vec![
                    xinput::XIEventMask::RAW_MOTION
                        | xinput::XIEventMask::RAW_KEY_PRESS
                        | xinput::XIEventMask::RAW_KEY_RELEASE
                        | xinput::XIEventMask::RAW_BUTTON_PRESS
                        | xinput::XIEventMask::RAW_BUTTON_RELEASE,
                ],
            }],
        )
        .map_err(error)?
        .check()
        .map_err(error)?;
        c.xkb_use_extension(1, 0).map_err(error)?.reply().map_err(error)?;
        c.xkb_per_client_flags(
            xkb::ID::USE_CORE_KBD.into(),
            xkb::PerClientFlag::DETECTABLE_AUTO_REPEAT,
            xkb::PerClientFlag::DETECTABLE_AUTO_REPEAT,
            xkb::BoolCtrl::default(),
            xkb::BoolCtrl::default(),
            xkb::BoolCtrl::default(),
        )
        .map_err(error)?
        .reply()
        .map_err(error)?;
        let state = c
            .query_pointer(root)
            .map_err(error)?
            .reply()
            .map_err(error)?
            .mask
            .into();
        let map = Keymap::load(&c)?;
        let (tx, rx) = mpsc::sync_channel(32);
        let (w, mut r) = UnixStream::pair().map_err(error)?;
        w.set_nonblocking(true).map_err(error)?;
        r.set_nonblocking(true).map_err(error)?;
        let initial = self.captured;
        let (ready, ready_rx) = mpsc::sync_channel(1);
        let h = std::thread::Builder::new()
            .name("oe-x11-capture".into())
            .spawn(move || {
                let mut capture = Capture {
                    c,
                    root,
                    map,
                    callback,
                    captured: false,
                    hidden: false,
                    pressed: BTreeMap::new(),
                    state,
                    relative: [0.0; 2],
                    last_pos: (0, 0),
                    last_raw_time: None,
                };
                let result = capture.grab(initial);
                let failed = result.is_err();
                let _ = ready.send(result);
                if failed {
                    return;
                }
                let result = (|| -> Result<(), PlatformError> {
                    loop {
                        while let Ok(cmd) = rx.try_recv() {
                            match cmd {
                                Command::Stop => return Ok(()),
                                Command::Capture(value, reply) => {
                                    let _ = reply.send(capture.grab(value));
                                }
                                Command::Warp(x, y, reply) => {
                                    let x = x.clamp(-32768, 32767) as i16;
                                    let y = y.clamp(-32768, 32767) as i16;
                                    let result = capture
                                        .c
                                        .warp_pointer(NONE, root, 0, 0, 0, 0, x, y)
                                        .map_err(error)
                                        .and_then(|r| r.check().map_err(error));
                                    if result.is_ok() {
                                        capture.last_pos = (i32::from(x), i32::from(y));
                                    }
                                    let _ = reply.send(result);
                                }
                            }
                        }
                        while let Some(event) = capture.c.poll_for_event().map_err(error)? {
                            capture.event(event)?;
                        }
                        capture.c.flush().map_err(error)?;
                        wait(&capture.c, &r, -1)?;
                        drain(&mut r);
                    }
                })();
                let _ = capture.grab(false);
                if let Err(e) = result {
                    log::error!("X11 capture stopped: {e}");
                }
            })
            .map_err(error)?;
        if let Err(e) = ready_rx.recv().map_err(error)? {
            let _ = h.join();
            return Err(e);
        }
        self.worker = Some((tx, w, h));
        Ok(())
    }
    fn stop(&mut self) {
        if let Some((tx, w, h)) = self.worker.take() {
            let _ = tx.send(Command::Stop);
            wake(&w);
            let _ = h.join();
        }
    }
    fn set_cursor_captured(&mut self, captured: bool) -> Result<(), PlatformError> {
        if self.worker.is_some() {
            self.command(|reply| Command::Capture(captured, reply))?;
        }
        self.captured = captured;
        Ok(())
    }
    fn warp_cursor(&mut self, x: i32, y: i32) -> Result<(), PlatformError> {
        self.command(|reply| Command::Warp(x, y, reply))
    }
}
impl Drop for Source {
    fn drop(&mut self) {
        self.stop();
    }
}
struct Capture {
    c: RustConnection,
    root: u32,
    map: Keymap,
    callback: Callback,
    captured: bool,
    hidden: bool,
    pressed: BTreeMap<u8, u16>,
    state: u16,
    relative: [f64; 2],
    last_pos: (i32, i32),
    last_raw_time: Option<u32>,
}
impl Capture {
    fn grab(&mut self, on: bool) -> Result<(), PlatformError> {
        if on == self.captured {
            return Ok(());
        }
        if on {
            let pointer = self.c.query_pointer(self.root).map_err(error)?.reply().map_err(error)?;
            self.last_pos = (pointer.root_x.into(), pointer.root_y.into());
            let keyboard = self
                .c
                .grab_keyboard(
                    false,
                    self.root,
                    CURRENT_TIME,
                    xproto::GrabMode::ASYNC,
                    xproto::GrabMode::ASYNC,
                )
                .map_err(error)?
                .reply()
                .map_err(error)?;
            if keyboard.status != xproto::GrabStatus::SUCCESS {
                return Err(error(format!("XGrabKeyboard refused: {:?}", keyboard.status)));
            }
            // From this point every failure must release the keyboard, even if
            // the pointer grab or cursor request fails before capture is ready.
            self.captured = true;
            let result = (|| {
                let pointer = self
                    .c
                    .grab_pointer(
                        false,
                        self.root,
                        xproto::EventMask::BUTTON_PRESS
                            | xproto::EventMask::BUTTON_RELEASE
                            | xproto::EventMask::POINTER_MOTION,
                        xproto::GrabMode::ASYNC,
                        xproto::GrabMode::ASYNC,
                        NONE,
                        NONE,
                        CURRENT_TIME,
                    )
                    .map_err(error)?
                    .reply()
                    .map_err(error)?;
                if pointer.status != xproto::GrabStatus::SUCCESS {
                    return Err(error(format!("XGrabPointer refused: {:?}", pointer.status)));
                }
                self.c
                    .xfixes_hide_cursor(self.root)
                    .map_err(error)?
                    .check()
                    .map_err(error)?;
                self.hidden = true;
                Ok(())
            })();
            if let Err(e) = result {
                let _ = self.grab(false);
                return Err(e);
            }
            self.relative = [0.0; 2];
            self.last_raw_time = None;
        } else {
            // Check each release independently. A failed request must not skip
            // the other grab or leave the cursor hidden. Checks also flush the
            // requests before acknowledging the transition to another client.
            let keyboard = self
                .c
                .ungrab_keyboard(CURRENT_TIME)
                .map_err(error)
                .and_then(|c| c.check().map_err(error));
            let pointer = self
                .c
                .ungrab_pointer(CURRENT_TIME)
                .map_err(error)
                .and_then(|c| c.check().map_err(error));
            let cursor = if self.hidden {
                self.c
                    .xfixes_show_cursor(self.root)
                    .map_err(error)
                    .and_then(|c| c.check().map_err(error))
            } else {
                Ok(())
            };
            if cursor.is_ok() {
                self.hidden = false;
            }
            keyboard?;
            pointer?;
            cursor?;
        }
        self.captured = on;
        self.c.flush().map_err(error)
    }
    fn key(&mut self, code: u8, down: bool, state: u16) -> Option<InputEvent> {
        let previous = self.pressed.get(&code).copied();
        let key = previous.or_else(|| self.map.logical(code, state))?;
        self.state = state;
        if previous.is_none() || !down {
            for (index, codes) in self
                .map
                .modifier_codes
                .chunks((self.map.modifier_codes.len() / 8).max(1))
                .enumerate()
            {
                if codes.contains(&code) {
                    let lock = self.map.masks[index]
                        & (openergy_platform::Modifiers::CAPS_LOCK
                            | openergy_platform::Modifiers::NUM_LOCK
                            | openergy_platform::Modifiers::SCROLL_LOCK)
                        != 0;
                    if lock {
                        if down {
                            self.state ^= 1 << index;
                        }
                    } else if down {
                        self.state |= 1 << index;
                    } else if !codes.iter().any(|k| *k != code && self.pressed.contains_key(k)) {
                        self.state &= !(1 << index);
                    }
                }
            }
        }
        let modifiers = self.map.modifiers(self.state);
        let button = code.into();
        Some(if down {
            self.pressed.insert(code, key);
            if previous.is_none() {
                InputEvent::KeyDown { key, modifiers, button }
            } else {
                InputEvent::KeyRepeat {
                    key,
                    modifiers,
                    count: 1,
                    button,
                }
            }
        } else {
            self.pressed.remove(&code);
            InputEvent::KeyUp { key, modifiers, button }
        })
    }
    fn forward(&self, event: Event) -> Result<(), PlatformError> {
        let focus = self.c.get_input_focus().map_err(error)?.reply().map_err(error)?.focus;
        let focus = if focus <= 1 { self.root } else { focus };
        match event {
            Event::KeyPress(mut e) | Event::KeyRelease(mut e) => {
                e.event = focus;
                e.child = NONE;
                self.c
                    .send_event(
                        false,
                        focus,
                        xproto::EventMask::KEY_PRESS | xproto::EventMask::KEY_RELEASE,
                        e,
                    )
                    .map_err(error)?
                    .check()
                    .map_err(error)?;
            }
            Event::ButtonPress(mut e) | Event::ButtonRelease(mut e) => {
                let mut target = self.root;
                loop {
                    let child = self
                        .c
                        .query_pointer(target)
                        .map_err(error)?
                        .reply()
                        .map_err(error)?
                        .child;
                    if child == NONE {
                        break;
                    }
                    target = child;
                }
                let position = self
                    .c
                    .translate_coordinates(self.root, target, e.root_x, e.root_y)
                    .map_err(error)?
                    .reply()
                    .map_err(error)?;
                e.event = target;
                e.child = NONE;
                e.event_x = position.dst_x;
                e.event_y = position.dst_y;
                self.c
                    .send_event(
                        false,
                        target,
                        xproto::EventMask::BUTTON_PRESS | xproto::EventMask::BUTTON_RELEASE,
                        e,
                    )
                    .map_err(error)?
                    .check()
                    .map_err(error)?;
            }
            _ => {}
        }
        self.c.flush().map_err(error)
    }
    fn event(&mut self, event: Event) -> Result<(), PlatformError> {
        let replay = if self.captured
            && matches!(
                event,
                Event::KeyPress(_) | Event::KeyRelease(_) | Event::ButtonPress(_) | Event::ButtonRelease(_)
            ) {
            Some(event.clone())
        } else {
            None
        };
        let ev = match event {
            Event::MappingNotify(_) => {
                self.map = Keymap::load(&self.c)?;
                None
            }
            Event::XinputRawMotion(e) => {
                if self.captured {
                    self.last_raw_time = Some(e.time);
                    let mut values = e.axisvalues_raw.iter();
                    let mut delta = [0.0; 2];
                    for (word, mask) in e.valuator_mask.iter().enumerate() {
                        for bit in 0..32 {
                            if mask & (1 << bit) != 0 {
                                if let Some(value) = values.next() {
                                    if let Some(slot) = delta.get_mut(word * 32 + bit) {
                                        *slot = f64::from(value.integral) + f64::from(value.frac) / 4294967296.0;
                                    }
                                }
                            }
                        }
                    }
                    for (a, b) in self.relative.iter_mut().zip(delta) {
                        *a += b;
                    }
                    let dx = self.relative[0].trunc() as i32;
                    let dy = self.relative[1].trunc() as i32;
                    self.relative[0] -= f64::from(dx);
                    self.relative[1] -= f64::from(dy);
                    if dx != 0 || dy != 0 {
                        Some(InputEvent::MouseMoveRel { dx, dy })
                    } else {
                        None
                    }
                } else {
                    let p = self.c.query_pointer(self.root).map_err(error)?.reply().map_err(error)?;
                    Some(InputEvent::MouseMoveAbs {
                        x: p.root_x.into(),
                        y: p.root_y.into(),
                    })
                }
            }
            Event::XinputRawKeyPress(e) if !self.captured => self.key(e.detail as u8, true, self.state),
            Event::XinputRawKeyRelease(e) if !self.captured => self.key(e.detail as u8, false, self.state),
            Event::XinputRawButtonPress(e) if !self.captured => button(e.detail as u8, true),
            Event::XinputRawButtonRelease(e) if !self.captured => button(e.detail as u8, false),
            Event::MotionNotify(e) if self.captured => {
                let position = (i32::from(e.root_x), i32::from(e.root_y));
                let delta = (position.0 - self.last_pos.0, position.1 - self.last_pos.1);
                self.last_pos = position;
                if self.last_raw_time != Some(e.time) && delta != (0, 0) {
                    Some(InputEvent::MouseMoveRel {
                        dx: delta.0,
                        dy: delta.1,
                    })
                } else {
                    None
                }
            }
            Event::KeyPress(e) if self.captured => self.key(e.detail, true, e.state.into()),
            Event::KeyRelease(e) if self.captured => self.key(e.detail, false, e.state.into()),
            Event::ButtonPress(e) if self.captured => button(e.detail, true),
            Event::ButtonRelease(e) if self.captured => button(e.detail, false),
            _ => None,
        };
        if let Some(ev) = ev {
            let decision = (self.callback)(ev);
            if decision == CaptureDecision::Swallow {
                self.grab(true)?;
            } else if self.captured {
                self.grab(false)?;
                // The server already applied the physical state. Forward the current event
                // to the focused application rather than posting another physical transition.
                if let Some(event) = replay {
                    self.forward(event)?;
                }
            }
        }
        Ok(())
    }
}
fn button(code: u8, down: bool) -> Option<InputEvent> {
    if (4..=7).contains(&code) {
        return if down {
            Some(InputEvent::MouseWheel {
                dx: match code {
                    6 => -120,
                    7 => 120,
                    _ => 0,
                },
                dy: match code {
                    4 => 120,
                    5 => -120,
                    _ => 0,
                },
            })
        } else {
            None
        };
    }
    let button = match code {
        1..=3 => code,
        8 => 4,
        9 => 5,
        _ => return None,
    };
    Some(if down {
        InputEvent::MouseDown { button }
    } else {
        InputEvent::MouseUp { button }
    })
}
