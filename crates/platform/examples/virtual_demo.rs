//! Real, headless product backend demonstration. No native input or sockets.
//! Allocation instrumentation is confined to this executable, never the library.
use openergy_platform::virtual_backend::{create, create_with_config, RecordKind, VirtualOptions};
use openergy_platform::{
    Backend, CaptureDecision, ClipboardData, ClipboardFormat, FormatKind, Grant, InjectRequest, InputEvent, Modifiers,
    PermissionKind, Permissions, PlatformError, RecordedEvent, ScreenRect, VirtualConfig,
};
use std::alloc::{GlobalAlloc, Layout, System};
use std::error::Error;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

type Outcome = Result<(), Box<dyn Error>>;
struct CountedSystem;
static COUNTING: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
#[global_allocator]
static ALLOCATOR: CountedSystem = CountedSystem;
// Safety: All allocation contracts and pointer ownership are delegated unchanged to System.
unsafe impl GlobalAlloc for CountedSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count_allocation();
        // Safety: The caller supplies the valid allocation layout required by GlobalAlloc.
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        count_allocation();
        // Safety: The caller supplies the valid allocation layout required by GlobalAlloc.
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        count_allocation();
        // Safety: The pointer, old layout, and new size retain the caller's GlobalAlloc contract.
        unsafe { System.realloc(pointer, layout, size) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // Safety: The pointer and layout retain the caller's matching allocation contract.
        unsafe { System.dealloc(pointer, layout) }
    }
}
fn count_allocation() {
    if COUNTING.load(Ordering::Relaxed) {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    }
}
fn check(label: &str, passed: bool) -> Outcome {
    println!("{} {label}", if passed { "PASS" } else { "FAIL" });
    if passed {
        Ok(())
    } else {
        Err(label.to_owned().into())
    }
}
fn parse(json: &str) -> Result<Vec<InputEvent>, Box<dyn Error>> {
    Ok(serde_json::from_str::<InjectRequest>(json)?.into_events())
}
fn fixture() -> ClipboardData {
    ClipboardData {
        formats: vec![
            ClipboardFormat {
                kind: FormatKind::TextUtf8,
                bytes: vec![65, 10, 206, 187],
            },
            ClipboardFormat {
                kind: FormatKind::HtmlUtf8,
                bytes: vec![60, 98, 62, 65, 60, 47, 98, 62],
            },
            ClipboardFormat {
                kind: FormatKind::Dib,
                bytes: vec![40, 0, 0, 0],
            },
        ],
    }
}

fn main() -> Outcome {
    // This is the exact factory and split ownership contract used by the daemon.
    let (
        Backend {
            name,
            mut source,
            mut sink,
            mut screens,
            mut clipboard,
        },
        control,
    ) = create(VirtualOptions {
        hid_seed: 1000,
        ..VirtualOptions::default()
    });
    check(
        "split backend name and default screen",
        name == "virtual" && screens.screens()[0].w == 1920,
    )?;
    check("empty recorded ledger", control.recorded(0).is_empty())?;
    check(
        "capture refuses injection before start",
        control.inject(Vec::new()).is_err(),
    )?;

    let events = parse(
        r#"{"events":[
        {"move_abs":[10,20]},
        {"key_down":{"key":"Shift_L","button":57,"modifiers":["shift"]}},
        {"key_down":{"key":"a","button":1,"modifiers":["shift"]}},
        {"key_repeat":{"key":97,"button":1,"modifiers":1,"count":2}},
        {"key_up":{"key":"a","button":1,"modifiers":1}},
        {"key_up":{"key":"Shift_L","button":57}},
        {"wheel":[0,-120]}, {"mouse_down":1}, {"move_rel":[3,2]}, {"mouse_up":1}
    ]}"#,
    )?;
    let expected = events.clone();
    let (observed_tx, observed_rx) = mpsc::sync_channel(32);
    let producer_control = control.clone();
    let capture = thread::spawn(move || -> Result<_, PlatformError> {
        source.start(Box::new(move |event| {
            let swallow = matches!(event, InputEvent::MouseMoveRel { .. });
            let _ = observed_tx.send(event);
            if swallow {
                CaptureDecision::Swallow
            } else {
                CaptureDecision::PassToOs
            }
        }))?;
        let report = producer_control.inject(events)?;
        if report.delivered != 10 || report.passed != 9 || report.swallowed != 1 {
            return Err(PlatformError::Failed("capture decision count mismatch".into()));
        }
        source.set_cursor_captured(true)?;
        source.warp_cursor(50, 60)?;
        producer_control.inject(vec![InputEvent::MouseMoveAbs { x: 900, y: 800 }])?;
        source.set_cursor_captured(false)?;
        source.stop();
        source.stop();
        // Prove restart and rejection of a second owner without leaking workers.
        source.start(Box::new(|_| CaptureDecision::PassToOs))?;
        let report = producer_control.inject(Vec::new())?;
        if report.delivered != 0 {
            return Err(PlatformError::Failed("empty batch delivered input".into()));
        }
        source.stop();
        Ok(source)
    });
    let mut source = capture.join().map_err(|_| "capture owner thread failed")??;
    let mut observed = Vec::new();
    for _ in 0..11 {
        observed.push(observed_rx.recv_timeout(Duration::from_secs(2))?);
    }
    check(
        "capture callback received 10 scripted events in order on a real thread",
        observed[..10] == expected,
    )?;
    check(
        "swallow and captured cursor preserve local position",
        screens.cursor_pos() == (50, 60),
    )?;
    let audit = control.ledger(0);
    check(
        "capture decision, cursor capture, and warp markers recorded",
        audit.events.iter().any(|entry| {
            matches!(
                entry.detail,
                RecordKind::Capture {
                    decision: openergy_platform::virtual_backend::RecordedDecision::Swallow
                }
            )
        }) && audit
            .events
            .iter()
            .any(|entry| matches!(entry.detail, RecordKind::CursorWarp { x: 50, y: 60 })),
    )?;
    check(
        "capture events stay out of the sink ledger",
        control.recorded(0).is_empty(),
    )?;

    let sink_control = control.clone();
    let sink_thread = thread::spawn(move || -> Result<_, PlatformError> {
        sink.inject(&InputEvent::MouseMoveAbs { x: 100, y: 100 })?;
        sink.inject(&InputEvent::MouseDown { button: 1 })?;
        sink.inject(&InputEvent::MouseMoveRel { dx: 2, dy: 0 })?;
        sink.inject(&InputEvent::MouseUp { button: 1 })?;
        for _ in 0..2 {
            sink.inject(&InputEvent::MouseDown { button: 1 })?;
            sink.inject(&InputEvent::MouseUp { button: 1 })?;
        }
        sink.inject(&InputEvent::MouseMoveAbs { x: 200, y: 200 })?;
        sink.inject(&InputEvent::MouseDown { button: 1 })?;
        sink.inject(&InputEvent::MouseUp { button: 1 })?;
        sink_control.set_hid_seed(2000);
        sink.inject(&InputEvent::MouseDown { button: 3 })?;
        sink.inject(&InputEvent::MouseUp { button: 3 })?;
        thread::sleep(Duration::from_millis(550));
        sink.inject(&InputEvent::MouseDown { button: 1 })?;
        sink.inject(&InputEvent::MouseUp { button: 1 })?;
        // A far drag resets the following click, even when it returns to the origin.
        sink.inject(&InputEvent::MouseDown { button: 1 })?;
        sink.inject(&InputEvent::MouseMoveRel { dx: 8, dy: 0 })?;
        sink.inject(&InputEvent::MouseMoveRel { dx: -8, dy: 0 })?;
        sink.inject(&InputEvent::MouseUp { button: 1 })?;
        sink.inject(&InputEvent::MouseDown { button: 1 })?;
        sink.inject(&InputEvent::MouseUp { button: 1 })?;
        Ok(sink)
    });
    let mut sink = sink_thread.join().map_err(|_| "sink thread failed")??;
    let records = control.recorded(0);
    check_numbering(&records)?;
    println!(
        "LEDGER {}",
        serde_json::to_string(&control.ledger(audit.stats.last_sequence))?
    );
    check(
        "recorded(since) is exclusive and supports an empty future cursor",
        control.recorded(u64::MAX).is_empty() && control.recorded(records[0].sequence).len() + 1 == records.len(),
    )?;

    let since = control.stats().last_sequence;
    sink.inject(&InputEvent::KeyDown {
        key: 0xEFE1,
        modifiers: Modifiers(1),
        button: 57,
    })?;
    sink.inject(&InputEvent::KeyDown {
        key: 97,
        modifiers: Modifiers(1),
        button: 1,
    })?;
    // Duplicate key down must not create a second outstanding press.
    sink.inject(&InputEvent::KeyDown {
        key: 97,
        modifiers: Modifiers(1),
        button: 1,
    })?;
    sink.inject(&InputEvent::MouseDown { button: 1 })?;
    let mut denied = Permissions::not_required();
    denied.post_events = Grant::Denied;
    control.set_permissions(denied);
    check(
        "sink permission revocation returns typed error",
        matches!(
            sink.inject(&InputEvent::MouseWheel { dx: 0, dy: 120 }),
            Err(PlatformError::PermissionDenied(PermissionKind::PostEvents))
        ),
    )?;
    sink.release_all();
    let release = control.recorded(since);
    check(
        "release_all emits reverse press order and marker despite revoked permission",
        release.len() == 8
            && matches!(release[4].event, Some(InputEvent::MouseUp { button: 1 }))
            && matches!(release[5].event, Some(InputEvent::KeyUp { key: 97, .. }))
            && matches!(release[6].event, Some(InputEvent::KeyUp { key: 0xEFE1, .. }))
            && release[7].release_all,
    )?;
    let sequence = control.stats().last_sequence;
    sink.release_all();
    check(
        "release_all is idempotent with an empty pressed ledger",
        sequence == control.stats().last_sequence
            && control.stats().pressed_keys == 0
            && control.stats().pressed_buttons == 0,
    )?;
    println!("RELEASE {}", serde_json::to_string(&release)?);
    control.set_permissions(Permissions::not_required());
    sink.set_cursor_visible(false);
    sink.set_cursor_visible(true);
    screens.prevent_sleep(true);
    check(
        "cursor visibility and sleep state are shared",
        control.stats().cursor_visible && control.stats().prevent_sleep,
    )?;
    screens.prevent_sleep(false);

    let external = control.clipboard_set(fixture())?;
    check(
        "external clipboard set increments stamp without own-write flag",
        external == 1 && !control.clipboard_get().own_write,
    )?;
    let clipboard_control = control.clone();
    let clipboard_thread = thread::spawn(move || -> Result<_, PlatformError> {
        let data = clipboard.read(1024)?;
        if data.formats.len() != 3 || clipboard.change_stamp() != 1 {
            return Err(PlatformError::Failed("clipboard read mismatch".into()));
        }
        clipboard.write(&data)?;
        clipboard.write(&data)?;
        if clipboard.change_stamp() != 3 || !clipboard_control.clipboard_get().own_write {
            return Err(PlatformError::Failed("own-write stamp mismatch".into()));
        }
        Ok(clipboard)
    });
    let mut clipboard = clipboard_thread.join().map_err(|_| "clipboard thread failed")??;
    check(
        "clipboard worker sees shared formats and every own write advances stamp",
        control.clipboard_get().data == fixture() && control.clipboard_get().last_own_write_stamp == Some(3),
    )?;
    let snapshot = control.clipboard_get();
    check(
        "clipboard read byte cap leaves data and stamp unchanged",
        clipboard.read(1).is_err() && control.clipboard_get().stamp == snapshot.stamp,
    )?;
    control.clipboard_set(ClipboardData::default())?;
    check(
        "external empty clipboard breaks own-write loop marker",
        !control.clipboard_get().own_write && clipboard.read(0)?.formats.is_empty() && clipboard.change_stamp() == 4,
    )?;
    // ScreenSystem's frozen clipboard accessor remains independently usable.
    screens.clipboard().write(&fixture())?;
    check(
        "legacy screen clipboard accessor shares independent worker state",
        clipboard.change_stamp() == 5,
    )?;
    println!(
        "CLIPBOARD stamp={} formats={} bytes={} own_write={}",
        clipboard.change_stamp(),
        snapshot.data.formats.len(),
        snapshot.data.total_bytes(),
        control.clipboard_get().own_write
    );

    let displays = Arc::new(AtomicU64::new(0));
    let count = Arc::clone(&displays);
    let display_control = control.clone();
    screens.on_display_changed(Box::new(move || {
        let _ = display_control.stats(); // Callback can query state without deadlock.
        count.fetch_add(1, Ordering::Relaxed);
    }));
    let display_thread = thread::spawn(move || -> Result<_, PlatformError> {
        let rects = vec![ScreenRect {
            x: -1280,
            y: 0,
            w: 1280,
            h: 720,
            scale: 2.0,
        }];
        control.set_screens(rects.clone())?;
        control.set_screens(rects)?;
        Ok(control)
    });
    let control = display_thread.join().map_err(|_| "display thread failed")??;
    check(
        "display callback fires once for a changed geometry on a real thread",
        displays.load(Ordering::Relaxed) == 1 && screens.screens()[0].x == -1280,
    )?;
    check(
        "invalid screen geometry is rejected atomically",
        control
            .set_screens(vec![ScreenRect {
                x: 0,
                y: 0,
                w: 0,
                h: 10,
                scale: 1.0,
            }])
            .is_err()
            && displays.load(Ordering::Relaxed) == 1,
    )?;
    control.set_screens(Vec::new())?;
    check(
        "empty display list supports a headless machine",
        screens.screens().is_empty() && displays.load(Ordering::Relaxed) == 2,
    )?;
    control.set_secure_input(Some("scripted-owner".into()));
    check(
        "secure input owner is shared",
        screens.secure_input_app().as_deref() == Some("scripted-owner"),
    )?;
    control.set_secure_input(None);
    let mut permissions = Permissions::not_required();
    permissions.input_monitoring = Grant::Denied;
    control.set_permissions(permissions.clone());
    check("permission state is shared", screens.permissions() == permissions)?;
    check(
        "capture revocation returns typed error",
        matches!(
            control.inject(Vec::new()),
            Err(PlatformError::PermissionDenied(PermissionKind::InputMonitoring))
        ),
    )?;
    check(
        "capture start is refused while permission is denied",
        matches!(
            source.start(Box::new(|_| CaptureDecision::PassToOs)),
            Err(PlatformError::PermissionDenied(PermissionKind::InputMonitoring))
        ),
    )?;
    control.set_permissions(Permissions::not_required());
    let callback_control = control.clone();
    let reentry_rejected = Arc::new(AtomicBool::new(false));
    let reentry = Arc::clone(&reentry_rejected);
    source.start(Box::new(move |_| {
        reentry.store(callback_control.inject(Vec::new()).is_err(), Ordering::Relaxed);
        CaptureDecision::PassToOs
    }))?;
    check(
        "second capture owner is refused",
        source.start(Box::new(|_| CaptureDecision::Swallow)).is_err(),
    )?;
    source.set_cursor_captured(true)?;
    let mut revoked = Permissions::not_required();
    revoked.input_monitoring = Grant::Denied;
    control.set_permissions(revoked);
    check(
        "running capture refuses events after revocation",
        control.inject(vec![InputEvent::MouseWheel { dx: 0, dy: 120 }]).is_err(),
    )?;
    source.set_cursor_captured(false)?;
    check(
        "cursor can always be uncaptured after revocation",
        !control.stats().cursor_captured && control.stats().cursor_visible,
    )?;
    control.set_permissions(Permissions::not_required());
    check(
        "running capture resumes after grant without restart",
        control
            .inject(vec![InputEvent::MouseWheel { dx: 0, dy: 120 }])?
            .delivered
            == 1,
    )?;
    check(
        "callback reentry returns an error without deadlock",
        reentry_rejected.load(Ordering::Relaxed),
    )?;
    source.set_cursor_captured(true)?;
    source.stop();
    check(
        "capture stop restores a pinned hidden cursor",
        !control.stats().cursor_captured && control.stats().cursor_visible,
    )?;
    drop(source);

    let permission_since = control.stats().last_sequence;
    let permission_script = serde_json::from_str(
        r#"{
        "accessibility":"denied", "input_monitoring":"unknown", "post_events":"denied",
        "on_request":{"accessibility":"granted","input_monitoring":"granted","post_events":"granted"}
    }"#,
    )?;
    control.configure_permissions(permission_script);
    check(
        "scripted grants stay unchanged until a permission request",
        screens.permissions().accessibility == Grant::Denied
            && screens.permissions().input_monitoring == Grant::Unknown,
    )?;
    check(
        "virtual permission requests apply scripted outcomes",
        screens.request_permission(PermissionKind::Accessibility)? == Grant::Granted
            && screens.request_permission(PermissionKind::InputMonitoring)? == Grant::Granted
            && screens.request_permission(PermissionKind::PostEvents)? == Grant::Granted
            && screens.permissions().post_events == Grant::Granted,
    )?;
    let requests = control.ledger(permission_since);
    check(
        "permission requests are recorded with previous and resulting grants",
        requests.events.len() == 3
            && requests.events.iter().all(|entry| {
                matches!(
                    entry.detail,
                    RecordKind::PermissionRequest {
                        grant: Grant::Granted,
                        ..
                    }
                )
            }),
    )?;
    control.set_permission_request_result(PermissionKind::Accessibility, Some(Grant::Denied));
    check(
        "a scripted denial is returned as grant state",
        screens.request_permission(PermissionKind::Accessibility)? == Grant::Denied,
    )?;
    check(
        "unscripted requests return current grant",
        screens.request_permission(PermissionKind::Accessibility)? == Grant::Denied,
    )?;
    control.set_permissions(Permissions::not_required());
    check(
        "permission setters clear pending prompt outcomes",
        screens.request_permission(PermissionKind::PostEvents)? == Grant::NotRequired,
    )?;

    for bad in [
        r#"{"events":[{"mouse_down":0}]}"#,
        r#"{"events":[{"move_abs":[1]}]}"#,
        r#"{"events":[{"key_down":"unknown-key"}]}"#,
        r#"{"events":[{"key_repeat":{"key":"a","count":0}}]}"#,
        r#"{"events":[{"key_down":"a","mouse_down":1}]}"#,
        r#"{"events":[{"key_down":"😀"}]}"#,
    ] {
        check("malformed script returns an error", parse(bad).is_err())?;
    }
    check(
        "frozen tagged events remain accepted",
        parse(r#"{"events":[{"type":"key_down","key":97,"modifiers":0,"button":1}]}"#)?
            == vec![InputEvent::KeyDown {
                key: 97,
                modifiers: Modifiers::NONE,
                button: 1,
            }],
    )?;
    check(
        "invalid native event returns an error",
        sink.inject(&InputEvent::MouseDown { button: 255 }).is_err(),
    )?;
    let sequence = control.stats().last_sequence;
    control.set_hid_seed(i64::MAX);
    check(
        "number exhaustion rejects down without ledger or pressed-state mutation",
        sink.inject(&InputEvent::MouseDown { button: 1 }).is_err()
            && control.stats().last_sequence == sequence
            && control.stats().pressed_buttons == 0,
    )?;
    drop(sink);
    let callback_lifetime = Arc::new(());
    let callback_weak = Arc::downgrade(&callback_lifetime);
    let held_control = control.clone();
    screens.on_display_changed(Box::new(move || {
        let _ = (&callback_lifetime, held_control.stats());
    }));
    drop(screens);
    check(
        "dropping screen owner releases callback and its captured control",
        callback_weak.upgrade().is_none(),
    )?;
    drop(clipboard);
    measure_bounded_hot_path()?;
    println!("PASS virtual_demo completed; no native input, clipboard, permissions, network, or Docker used");
    Ok(())
}

fn check_numbering(records: &[RecordedEvent]) -> Outcome {
    let mut previous = 1000;
    let mut pressed = [None; 8];
    let mut counts = Vec::new();
    let mut matched = true;
    for entry in records {
        match entry.event {
            Some(InputEvent::MouseDown { button }) => {
                let number = entry.click_number.ok_or("down lacks click number")?;
                matched &= number > previous;
                previous = number;
                pressed[usize::from(button)] = Some((entry.click_number, entry.click_state));
                counts.push(entry.click_state);
                if button == 3 {
                    matched &= number > 2000;
                }
            }
            Some(InputEvent::MouseUp { button }) => {
                matched &= pressed[usize::from(button)].take() == Some((entry.click_number, entry.click_state));
            }
            Some(InputEvent::MouseMoveAbs { .. } | InputEvent::MouseMoveRel { .. }) => {
                if let Some(assignment) = pressed.iter().flatten().last() {
                    matched &= *assignment == (entry.click_number, entry.click_state);
                } else {
                    matched &= entry.click_number.is_none() && entry.click_state.is_none();
                }
            }
            _ => {}
        }
    }
    check(
        "S18 strictly increasing down numbers, first above seed, drag/up reuse, right-button re-seed",
        matched,
    )?;
    check(
        "S18 click states 1,2,3; distance/time/drag reset; independent right button",
        counts == vec![Some(1), Some(2), Some(3), Some(1), Some(1), Some(1), Some(2), Some(1)],
    )?;
    Ok(())
}

fn measure_bounded_hot_path() -> Outcome {
    let (mut backend, control) = create_with_config(
        VirtualOptions::default(),
        VirtualConfig {
            ledger_capacity: 128,
            pressed_capacity: 8,
            max_clipboard_bytes: 32,
            ..VirtualConfig::default()
        },
    )?;
    let before = control.stats();
    ALLOCATIONS.store(0, Ordering::Relaxed);
    COUNTING.store(true, Ordering::Relaxed);
    let start = Instant::now();
    let mut outcome = Ok(());
    for index in 0..100_000 {
        let event = match index % 10 {
            0 => InputEvent::KeyDown {
                key: 97,
                modifiers: Modifiers::NONE,
                button: 1,
            },
            1 => InputEvent::KeyRepeat {
                key: 97,
                modifiers: Modifiers::NONE,
                button: 1,
                count: 1,
            },
            2 => InputEvent::KeyUp {
                key: 97,
                modifiers: Modifiers::NONE,
                button: 1,
            },
            3 => InputEvent::MouseDown { button: 1 },
            4 => InputEvent::MouseMoveRel { dx: 1, dy: 0 },
            5 => InputEvent::MouseUp { button: 1 },
            6 => InputEvent::MouseWheel { dx: 0, dy: -120 },
            _ => InputEvent::MouseMoveAbs { x: index, y: 0 },
        };
        if let Err(error) = backend.sink.inject(&event) {
            outcome = Err(error);
            break;
        }
    }
    let elapsed = start.elapsed();
    COUNTING.store(false, Ordering::Relaxed);
    outcome?;
    let allocations = ALLOCATIONS.load(Ordering::Relaxed);
    let after = control.stats();
    println!("MEASURE events={} elapsed_us={} ns_per_event={} allocations={} ring_len={} ring_cap={} reserved_bytes={} dropped={}",
        after.last_sequence, elapsed.as_micros(), elapsed.as_nanos() / 100_000, allocations,
        after.ledger_len, after.ledger_capacity, after.ledger_reserved_bytes, after.dropped);
    check(
        "100k real sink events allocate zero times in the hot path",
        allocations == 0 && after.last_sequence == 100_000,
    )?;
    check(
        "ring stays bounded and reports exact dropped count",
        after.ledger_len == 128
            && after.dropped == 99_872
            && after.ledger_reserved_bytes == before.ledger_reserved_bytes
            && after.pressed_keys == 0
            && after.pressed_buttons == 0,
    )?;
    let ledger = control.ledger(0);
    check(
        "retained records have strict sequences and monotonic timestamps",
        ledger.events.windows(2).all(|pair| {
            pair[0].recorded.sequence + 1 == pair[1].recorded.sequence && pair[0].timestamp_us <= pair[1].timestamp_us
        }),
    )?;
    // A small configured pressed ledger returns an error before mutating state.
    for button in 1..=8 {
        backend.sink.inject(&InputEvent::KeyDown {
            key: button,
            modifiers: Modifiers::NONE,
            button,
        })?;
    }
    check(
        "pressed capacity failure preserves existing state",
        backend
            .sink
            .inject(&InputEvent::KeyDown {
                key: 9,
                modifiers: Modifiers::NONE,
                button: 9,
            })
            .is_err()
            && control.stats().pressed_keys == 8,
    )?;
    backend.sink.release_all();
    check(
        "bounded pressed ledger still releases all keys",
        control.stats().pressed_keys == 0,
    )?;
    let oversize = ClipboardData {
        formats: vec![ClipboardFormat {
            kind: FormatKind::Dib,
            bytes: vec![0; 33],
        }],
    };
    let stamp = backend.clipboard.change_stamp();
    check(
        "clipboard write size cap rejects both external and own writes atomically",
        control.clipboard_set(oversize.clone()).is_err()
            && backend.clipboard.write(&oversize).is_err()
            && backend.clipboard.change_stamp() == stamp,
    )?;
    let mut duplicates = fixture();
    duplicates.formats.push(duplicates.formats[0].clone());
    check(
        "duplicate clipboard formats return an error",
        control.clipboard_set(duplicates).is_err(),
    )?;
    let (mut disabled, disabled_control) = create_with_config(
        VirtualOptions::default(),
        VirtualConfig {
            ledger_capacity: 0,
            ..VirtualConfig::default()
        },
    )?;
    disabled.sink.inject(&InputEvent::MouseDown { button: 1 })?;
    disabled.sink.release_all();
    check(
        "zero-cap ring reports dropped records and still clears pressed state",
        disabled_control.recorded(0).is_empty()
            && disabled_control.stats().dropped == 3
            && disabled_control.stats().pressed_buttons == 0,
    )?;
    Ok(())
}
