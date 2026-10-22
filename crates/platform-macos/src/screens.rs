//! Geometry, permission preflight, and opt-in lifecycle notifications.
use crate::{displays, error, ffi, permissions, MacClipboard, MacNotice, Notices, Result};
use openergy_platform::{ClipboardBackend, Grant, PermissionKind, Permissions, ScreenRect, ScreenSystem};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc, Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Instant;

type DisplayCallback = Box<dyn FnMut() + Send>;

pub struct MacScreens {
    clipboard: MacClipboard,
    notices: Notices,
    monitor: Option<Monitor>,
    assertion: Option<u32>,
    layout_dirty: Arc<AtomicBool>,
}
impl MacScreens {
    pub(crate) fn new(notices: Notices, clipboard: MacClipboard, layout_dirty: Arc<AtomicBool>) -> Self {
        Self {
            clipboard,
            notices,
            monitor: None,
            assertion: None,
            layout_dirty,
        }
    }
    fn set_prevent_sleep(&mut self, on: bool) -> Result<()> {
        match (on, self.assertion) {
            (true, None) => {
                let _pool = ffi::Pool::new();
                let kind = ffi::ns_string("PreventUserIdleSystemSleep")?;
                let reason = ffi::ns_string("Openergy remote input is active")?;
                let mut assertion = 0;
                // Safety: explicit remote-active request only; strings and out pointer are live.
                let status = unsafe {
                    ffi::IOPMAssertionCreateWithName(kind.0.cast_const(), 255, reason.0.cast_const(), &mut assertion)
                };
                check(status, "create sleep assertion")?;
                self.assertion = Some(assertion);
            }
            (false, Some(assertion)) => {
                // Safety: release exactly the assertion created and owned by this instance.
                check(
                    unsafe { ffi::IOPMAssertionRelease(assertion) },
                    "release sleep assertion",
                )?;
                self.assertion = None;
            }
            _ => {}
        }
        Ok(())
    }
}
impl ScreenSystem for MacScreens {
    fn screens(&self) -> Vec<ScreenRect> {
        match displays::active() {
            Ok(list) => list
                .active
                .into_iter()
                .map(|d| ScreenRect {
                    x: d.bounds.x.round() as i32,
                    y: d.bounds.y.round() as i32,
                    w: d.bounds.width.round() as i32,
                    h: d.bounds.height.round() as i32,
                    scale: d.scale_x,
                })
                .collect(),
            Err(_) => {
                log::error!("macOS display enumeration failed");
                Vec::new()
            }
        }
    }
    fn cursor_pos(&self) -> (i32, i32) {
        // Safety: creates only a metadata snapshot; it never posts input.
        let value = unsafe {
            ffi::OwnedCf::new(ffi::CGEventCreate(ffi::null()).cast_const(), "cursor query")
                .map(|e| ffi::CGEventGetLocation(e.0))
        };
        match value {
            Ok(p) => (p.x.round() as i32, p.y.round() as i32),
            Err(_) => {
                log::error!("macOS cursor query failed");
                (0, 0)
            }
        }
    }
    fn clipboard(&mut self) -> &mut dyn ClipboardBackend {
        &mut self.clipboard
    }
    fn permissions(&self) -> Permissions {
        permissions::platform()
    }
    /// Explicit TCC request only. Construction and safe probes never call it.
    fn request_permission(&mut self, kind: PermissionKind) -> Result<Grant> {
        let granted = permissions::request(kind)?;
        Ok(if granted { Grant::Granted } else { Grant::Denied })
    }
    fn on_display_changed(&mut self, cb: DisplayCallback) {
        // One lifecycle thread, no polling when idle. Replacing callback joins
        // the old observer before installing the new one.
        self.monitor.take();
        match Monitor::start(self.notices.clone(), cb, self.layout_dirty.clone()) {
            Ok(monitor) => self.monitor = Some(monitor),
            Err(_) => {
                self.notices.send(MacNotice::NotificationSetupFailed);
                log::error!("macOS notification setup failed");
            }
        }
    }
    fn prevent_sleep(&mut self, on: bool) {
        if self.set_prevent_sleep(on).is_err() {
            log::error!("macOS sleep assertion operation failed");
        }
    }
    fn secure_input_app(&self) -> Option<String> {
        let secure = displays::secure_input();
        secure
            .enabled
            .then(|| secure.owner_app.unwrap_or_else(|| "unknown".into()))
    }
}
impl Drop for MacScreens {
    fn drop(&mut self) {
        self.monitor.take();
        if self.assertion.is_some() {
            let _ = self.set_prevent_sleep(false);
        }
    }
}
fn check(status: i32, operation: &str) -> Result<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(error(format!("{operation}: {status}")))
    }
}

struct Monitor {
    wake: ffi::RunLoopWake,
    stopping: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}
impl Monitor {
    fn start(notices: Notices, mut callback: DisplayCallback, layout_dirty: Arc<AtomicBool>) -> Result<Self> {
        let stopping = Arc::new(AtomicBool::new(false));
        let thread_stopping = stopping.clone();
        let (tx, rx) = mpsc::sync_channel(1);
        let thread = thread::Builder::new()
            .name("openergy-system-events".into())
            .spawn(move || {
                let _pool = ffi::Pool::new();
                let setup = (|| -> Result<_> {
                    let (wake, source) = ffi::RunLoopWake::new()?;
                    let mut state = Box::new(ObserverState {
                        display: Box::new(DisplaySignal {
                            epoch: Instant::now(),
                            display_millis: AtomicU64::new(0),
                            loop_ref: wake.loop_ref,
                            wake_source: wake.source,
                        }),
                        loop_ref: wake.loop_ref,
                        notices: notices.clone(),
                        power_connection: 0,
                        port: std::ptr::null_mut(),
                        notifier: 0,
                        display_registered: false,
                        layout_dirty,
                    });
                    let user = (&mut *state as *mut ObserverState).cast();
                    // Safety: stable ObserverState outlives all registered callbacks.
                    unsafe {
                        check(
                            ffi::CGDisplayRegisterReconfigurationCallback(
                                display_changed,
                                (&*state.display as *const DisplaySignal).cast_mut().cast(),
                            ),
                            "display observer",
                        )?;
                        state.display_registered = true;
                        state.power_connection =
                            ffi::IORegisterForSystemPower(user, &mut state.port, power_changed, &mut state.notifier);
                        if state.power_connection == 0 {
                            return Err(error("IORegisterForSystemPower failed"));
                        }
                        let power_source = ffi::IONotificationPortGetRunLoopSource(state.port);
                        if power_source.is_null() {
                            return Err(error("missing power notification source"));
                        }
                        ffi::CFRunLoopAddSource(wake.loop_ref as ffi::Ref, power_source, ffi::kCFRunLoopCommonModes);
                    }
                    let locked = ffi::ns_string("com.apple.screenIsLocked")?;
                    let unlocked = ffi::ns_string("com.apple.screenIsUnlocked")?;
                    // Safety: names are copied by the center; observer remains alive until removal.
                    unsafe {
                        let center = ffi::CFNotificationCenterGetDistributedCenter();
                        ffi::CFNotificationCenterAddObserver(
                            center,
                            user,
                            keyboard_layout_changed,
                            ffi::kTISNotifySelectedKeyboardInputSourceChanged,
                            ffi::null(),
                            4,
                        );
                        ffi::CFNotificationCenterAddObserver(
                            center,
                            user,
                            lock_changed,
                            locked.0.cast_const(),
                            ffi::null(),
                            4,
                        );
                        ffi::CFNotificationCenterAddObserver(
                            center,
                            user,
                            lock_changed,
                            unlocked.0.cast_const(),
                            ffi::null(),
                            4,
                        );
                    }
                    Ok((wake, source, state))
                })();
                match setup {
                    Err(err) => {
                        let _ = tx.send(Err(err));
                    }
                    Ok((wake, _source, state)) => {
                        if tx.send(Ok(wake)).is_ok() {
                            let mut delivered = 0;
                            while !thread_stopping.load(Ordering::Acquire) {
                                let stamp = state.display.display_millis.load(Ordering::Acquire);
                                let now = state.display.epoch.elapsed().as_secs_f64();
                                let mut timeout = 86400.0;
                                if stamp != 0 && stamp != delivered {
                                    let remaining = (stamp as f64 / 1000.0 + 0.250) - now;
                                    if remaining <= 0.0 {
                                        delivered = stamp;
                                        notices.send(MacNotice::DisplaysChanged);
                                        callback();
                                    } else {
                                        timeout = remaining;
                                    }
                                }
                                let _iteration = ffi::Pool::new();
                                // Safety: current loop owns the registered sources until state drops.
                                if unsafe { ffi::CFRunLoopRunInMode(ffi::kCFRunLoopDefaultMode, timeout, true) } == 1 {
                                    break;
                                }
                            }
                        }
                        drop(state);
                    }
                }
            })
            .map_err(|e| error(format!("system notification thread: {e}")))?;
        match rx.recv() {
            Ok(Ok(wake)) => Ok(Self {
                wake,
                stopping,
                thread: Some(thread),
            }),
            Ok(Err(e)) => {
                let _ = thread.join();
                Err(e)
            }
            Err(_) => {
                let _ = thread.join();
                Err(error("system observer setup channel closed"))
            }
        }
    }
}
impl Drop for Monitor {
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::Release);
        self.wake.wake();
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

struct DisplaySignal {
    epoch: Instant,
    display_millis: AtomicU64,
    loop_ref: usize,
    wake_source: usize,
}
struct ObserverState {
    display: Box<DisplaySignal>,
    loop_ref: usize,
    notices: Notices,
    power_connection: u32,
    port: ffi::MutRef,
    notifier: u32,
    display_registered: bool,
    layout_dirty: Arc<AtomicBool>,
}
impl Drop for ObserverState {
    fn drop(&mut self) {
        let user = (self as *mut Self).cast();
        // Safety: unregister before releasing state; power delivery occurs on this thread's loop.
        unsafe {
            ffi::CFNotificationCenterRemoveEveryObserver(ffi::CFNotificationCenterGetDistributedCenter(), user);
            if self.display_registered {
                ffi::CGDisplayRemoveReconfigurationCallback(
                    display_changed,
                    (&*self.display as *const DisplaySignal).cast_mut().cast(),
                );
            }
            if self.notifier != 0 {
                ffi::IODeregisterForSystemPower(&mut self.notifier);
            }
            if !self.port.is_null() {
                let source = ffi::IONotificationPortGetRunLoopSource(self.port);
                if !source.is_null() {
                    ffi::CFRunLoopRemoveSource(self.loop_ref as ffi::Ref, source, ffi::kCFRunLoopCommonModes);
                }
                ffi::IONotificationPortDestroy(self.port);
            }
            if self.power_connection != 0 {
                ffi::IOServiceClose(self.power_connection);
            }
        }
    }
}
unsafe extern "C" fn display_changed(_: u32, flags: u32, user: ffi::MutRef) {
    if user.is_null() || flags & 1 != 0 {
        return;
    }
    // Safety: registration retains the stable allocation until callback removal.
    let state = unsafe { &*user.cast::<DisplaySignal>() };
    state
        .display_millis
        .store(state.epoch.elapsed().as_millis().max(1) as u64, Ordering::Release);
    // Safety: retained run-loop/source references remain live through callback removal.
    unsafe {
        ffi::CFRunLoopSourceSignal(state.wake_source as ffi::Ref);
        ffi::CFRunLoopWakeUp(state.loop_ref as ffi::Ref);
    }
}
unsafe extern "C" fn power_changed(user: ffi::MutRef, _: u32, message: u32, argument: ffi::MutRef) {
    if user.is_null() {
        return;
    }
    // Safety: IOKit invokes this on our loop while ObserverState is registered.
    let state = unsafe { &*user.cast::<ObserverState>() };
    // IOKit's iokit_common_msg values from the public IOMessage.h.
    match message {
        0xE000_0270 | 0xE000_0280 => {
            // Safety: acknowledge immediately, before notifying Rust consumers; never veto sleep.
            unsafe {
                ffi::IOAllowPowerChange(state.power_connection, argument as isize);
            }
            if message == 0xE000_0280 {
                state.notices.send(MacNotice::Sleeping);
            }
        }
        0xE000_0300 => state.notices.send(MacNotice::Woke),
        _ => {}
    }
}
unsafe extern "C" fn lock_changed(_: ffi::Ref, user: ffi::MutRef, name: ffi::Ref, _: ffi::Ref, _: ffi::Ref) {
    if user.is_null() {
        return;
    }
    // Safety: observer and notification name remain live for this callback.
    let state = unsafe { &*user.cast::<ObserverState>() };
    if let Ok(name) = ffi::cf_string(name) {
        match name.as_str() {
            "com.apple.screenIsLocked" => state.notices.send(MacNotice::ScreenLocked),
            "com.apple.screenIsUnlocked" => state.notices.send(MacNotice::ScreenUnlocked),
            _ => {}
        }
    }
}

unsafe extern "C" fn keyboard_layout_changed(
    _: ffi::Ref,
    observer: ffi::MutRef,
    _: ffi::Ref,
    _: ffi::Ref,
    _: ffi::Ref,
) {
    if !observer.is_null() {
        // Safety: observer is the registered stable ObserverState until removal.
        unsafe { &*observer.cast::<ObserverState>() }
            .layout_dirty
            .store(true, Ordering::Release);
    }
}
