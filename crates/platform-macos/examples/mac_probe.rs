//! Deliberately safe: no tap start, event post, cursor mutation, clipboard
//! payload fetch/write, permission request, input-source selection, or assertion.
#[cfg(target_os = "macos")]
use inject::{ClickNumbering, Injector, NativePoster, INJECTION_TAG};
#[cfg(target_os = "macos")]
use mac::types::SUPER;
#[cfg(target_os = "macos")]
use openergy_platform::{InputEvent, InputSink, Modifiers};
#[cfg(target_os = "macos")]
use openergy_platform_macos::{self as mac, displays, inject, keymap, pasteboard, permissions, MacOptions};
#[cfg(target_os = "macos")]
use openergy_protocol::keys;
#[cfg(target_os = "macos")]
use std::collections::HashMap;
#[cfg(target_os = "macos")]
use std::path::Path;
#[cfg(target_os = "macos")]
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[cfg(target_os = "macos")]
fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
#[cfg(target_os = "macos")]
fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("keymap") if args.len() == 1 => keymap_dump()?,
        Some("events-dry") if args.len() == 1 => {
            events_dry(ClickNumbering::Native)?;
            events_dry(ClickNumbering::Legacy)?;
        }
        Some("perms") if args.len() == 1 => println!("{:?} (preflight only)", permissions::platform()),
        Some("displays") if args.len() == 1 => {
            println!("{:#?}", displays::active()?);
            println!("secure_input={:?}", displays::secure_input());
        }
        Some("pasteboard-info") if args.len() == 1 => {
            let info = pasteboard::info()?;
            println!("change_count={} types={}", info.change_count, info.types.len());
            for name in info.types {
                println!("{name}");
            }
            println!("metadata only; no clipboard contents read");
        }
        Some("dib") if args.len() <= 2 => dib(Path::new(
            args.get(1)
                .map(String::as_str)
                .unwrap_or("crates/platform-macos/verification/fixtures"),
        ))?,
        Some("create") if args.len() == 1 => {
            let backend: openergy_platform::Backend = mac::create(MacOptions::default())?;
            println!(
                "backend={} permissions={:?}; capture not started",
                backend.name,
                backend.screens.permissions()
            );
            if mac::create(MacOptions {
                lines_per_notch: f64::NAN,
                ..MacOptions::default()
            })
            .is_ok()
            {
                return Err("invalid options unexpectedly accepted".into());
            }
            println!("invalid lines_per_notch rejected before construction");
            let (backend, notices): (openergy_platform::Backend, std::sync::mpsc::Receiver<mac::MacNotice>) =
                mac::create_with_notices(MacOptions::default())?;
            if !matches!(notices.try_recv(), Err(std::sync::mpsc::TryRecvError::Empty)) {
                return Err("construction unexpectedly emitted a lifecycle notice".into());
            }
            println!(
                "create_with_notices backend={}; shared Backend type; notice queue empty; observers not started",
                backend.name
            );
        }
        Some("--help") if args.len() == 1 => help(),
        _ => {
            help();
            return Err("invalid arguments".into());
        }
    }
    Ok(())
}
#[cfg(target_os = "macos")]
fn help() {
    println!("mac_probe keymap|events-dry|perms|displays|pasteboard-info|dib [output-dir]|create");
}

#[cfg(target_os = "macos")]
fn keymap_dump() -> openergy_platform_macos::Result<()> {
    let mut keymap = keymap::Keymap::new()?;
    println!("layout={}", keymap.layout_id());
    println!("keycode layer KeyID char mask key_button");
    for code in 0..128 {
        for (name, flags) in [("none", 0), ("shift", keymap::CG_SHIFT)] {
            match keymap.capture(code, flags)? {
                Some(key) => {
                    let ch = if !(0xE000..=0xF8FF).contains(&key.key) {
                        char::from_u32(u32::from(key.key))
                    } else {
                        None
                    };
                    let character = ch.map_or_else(|| "-".into(), |ch| format!("{ch:?}"));
                    println!(
                        "{code:03} {name:5} {:#06X} {character} {:#06X} {}",
                        key.key, key.mask, key.button
                    );
                }
                None => println!("{code:03} {name:5} unmapped"),
            }
        }
    }
    println!("injection plans:");
    for character in "aA1!@#éüß€".chars() {
        println!(
            "{character:?} {:#06X}: {:?}",
            character as u16,
            keymap.plan(character as u16, 0)?
        );
    }
    for (name, key, mask) in [
        ("Return", keys::RETURN, 0),
        ("Tab", keys::TAB, 0),
        ("F5", keys::F5, 0),
        ("Left", keys::LEFT, 0),
        ("Right", keys::RIGHT, 0),
        ("Up", keys::UP, 0),
        ("Down", keys::DOWN, 0),
        ("KP_5", keys::KP_5, 0),
        ("Super+c", b'c' as u16, SUPER),
    ] {
        println!("{name} {key:#06X} mask={mask:#06X}: {:?}", keymap.plan(key, mask)?);
    }
    println!("capture probes (same layout; no events posted):");
    for (name, code, flags) in [
        ("Command+c", 8, keymap::CG_COMMAND),
        ("Control+c", 8, keymap::CG_CONTROL),
        ("Caps+a", 0, keymap::CG_CAPS),
        ("Option+e", 14, keymap::CG_OPTION),
        ("Option+u", 32, keymap::CG_OPTION),
        ("Option+s", 1, keymap::CG_OPTION),
    ] {
        println!("{name}: {:?}", keymap.capture(code, flags)?);
    }
    println!("Unicode fallback U+2603: {:?}", keymap.plan(0x2603, 0)?);
    // Unsupported named keys must not become private-use Unicode text.
    match keymap.plan(keys::F35, 0) {
        Err(err) => println!("F35 unsupported (expected): {err}"),
        Ok(plan) => println!("F35: {plan:?}"),
    }
    println!("layout_refresh_changed={}", keymap.refresh()?);
    Ok(())
}

#[cfg(target_os = "macos")]
fn events_dry(mode: ClickNumbering) -> Result<()> {
    let poster = NativePoster::inspection(mode)?;
    let interval = inject::system_double_click_interval();
    let mut sink = Injector::with_poster(poster, Some(keymap::Keymap::new()?), mode, interval, (100.0, 100.0))?;
    // Same methods and native CGEvent builder as InputSink, with a monotonic
    // timestamp override to exercise double/triple clicks without real sleeps.
    sink.inject(&InputEvent::MouseMoveAbs { x: 110, y: 110 })?;
    sink.mouse_down_at(1, 0.0)?;
    sink.inject(&InputEvent::MouseMoveAbs { x: 120, y: 120 })?;
    sink.inject(&InputEvent::MouseMoveAbs { x: 130, y: 120 })?;
    sink.inject(&InputEvent::MouseUp { button: 1 })?;
    for time in [1.0, 1.0 + interval / 4.0, 1.0 + interval / 2.0] {
        sink.mouse_down_at(1, time)?;
        sink.inject(&InputEvent::MouseUp { button: 1 })?;
    }
    sink.mouse_down_at(3, 2.0 + interval)?;
    sink.inject(&InputEvent::MouseMoveAbs { x: 131, y: 121 })?;
    sink.inject(&InputEvent::MouseUp { button: 3 })?;
    // +60 twice emits +1 then +2 lines. -60 twice emits -1 then -2.
    for delta in [120, -120, 60, 60, -60, -60] {
        sink.inject(&InputEvent::MouseWheel { dx: delta, dy: delta })?;
    }
    for (key, mask, button) in [
        (b'a' as u16, Modifiers::SHIFT, 1),
        (b'c' as u16, Modifiers::SUPER, 2),
        (0x2603, 0, 3),
    ] {
        sink.inject(&InputEvent::KeyDown {
            key,
            modifiers: Modifiers(mask),
            button,
        })?;
        sink.inject(&InputEvent::KeyRepeat {
            key,
            modifiers: Modifiers(mask),
            count: 1,
            button,
        })?;
        sink.inject(&InputEvent::KeyUp {
            key,
            modifiers: Modifiers(mask),
            button,
        })?;
    }
    // Also show an actual shifted physical glyph: logical uppercase A.
    sink.inject(&InputEvent::KeyDown {
        key: b'A' as u16,
        modifiers: Modifiers(Modifiers::SHIFT),
        button: 4,
    })?;
    sink.inject(&InputEvent::KeyUp {
        key: b'A' as u16,
        modifiers: Modifiers(Modifiers::SHIFT),
        button: 4,
    })?;
    // Verify disconnect cleanup through the real ledger and real native builder,
    // while the inspection capability still makes posting impossible.
    sink.inject(&InputEvent::KeyDown {
        key: keys::F5,
        modifiers: Modifiers::NONE,
        button: 10,
    })?;
    sink.mouse_down_at(2, 4.0 + interval)?;
    sink.release_all()?;
    if sink.held_count() != 0 {
        return Err("release_all left held inputs".into());
    }
    // A new session within the double-click interval starts a fresh click chain,
    // while the same shared model keeps its rising event-number counter.
    sink.mouse_down_at(2, 4.0 + interval + interval / 4.0)?;
    sink.inject(&InputEvent::MouseUp { button: 2 })?;

    let mut held = HashMap::new();
    let mut last_number = 0;
    let mut counts = Vec::new();
    let mut wheel = Vec::new();
    println!("mode={mode:?} double_click_interval={interval}s; real CGEvents built, ZERO posts");
    for (i, e) in sink.poster().observations().iter().enumerate() {
        println!("{i:02} type={} xy={:?} button={} number={} click={} flags={:#x} delta={:?} lines={:?} fixed={:?} pixels={:?} keycode={} tag={:#x} unicode={:?} hid_seed={}",
            e.kind,e.location,e.button,e.number,e.click_state,e.flags,e.pointer_delta,e.lines,e.fixed_lines,e.pixels,e.keycode,e.user_data,e.unicode,e.seed);
        if e.user_data != INJECTION_TAG {
            return Err("missing self-event tag".into());
        }
        match e.kind {
            1 | 3 | 25 => {
                if mode == ClickNumbering::Native {
                    if e.number <= last_number.max(e.seed) {
                        return Err("down number did not exceed previous/HID seed".into());
                    }
                    last_number = e.number;
                } else if e.number != 0 {
                    return Err("legacy number is set".into());
                }
                held.insert(e.button, (e.number, e.click_state));
                counts.push(e.click_state);
            }
            2 | 4 | 6 | 7 | 26 | 27 => {
                if held.get(&e.button) != Some(&(e.number, e.click_state)) {
                    return Err("drag/up lost down identity".into());
                }
                if matches!(e.kind, 2 | 4 | 26) {
                    held.remove(&e.button);
                }
            }
            5 if e.number != 0 => return Err("free move has an event number".into()),
            22 => {
                if e.lines.0 != e.lines.1
                    || e.pixels != (e.lines.0 * 10, e.lines.1 * 10)
                    || e.fixed_lines != (e.lines.0 as f64, e.lines.1 as f64)
                {
                    return Err("inconsistent wheel fields".into());
                }
                wheel.push(e.lines.1);
            }
            _ => {}
        }
    }
    if !held.is_empty() || counts != [1, 1, 2, 3, 1, 1, 1] || wheel != [3, -3, 1, 2, -1, -2] {
        return Err("unbalanced click sequence or incorrect click/wheel counts".into());
    }
    if sink.poster().observations().iter().all(|e| e.unicode != [0x2603]) {
        return Err("Unicode fallback absent".into());
    }
    println!("PASS: rising native downs above HID seed, retained drag/up identity, click states {counts:?}, fractional wheel {wheel:?}, Unicode fallback, tag; release resets click history; ledger empty");
    Ok(())
}

#[cfg(target_os = "macos")]
fn dib(dir: &Path) -> Result<()> {
    const CAP: usize = 64 * 1024 * 1024;
    std::fs::create_dir_all(dir)?;
    for alpha_fixture in [false, true] {
        let (width, height) = (64usize, 48usize);
        let mut pixels = Vec::with_capacity(width * height * 4);
        let alphas = [0u8, 1, 2, 17, 64, 127, 128, 200, 254, 255];
        for y in 0..height {
            for x in 0..width {
                pixels.extend_from_slice(&[
                    (x * 3 + y * 7) as u8,
                    (x * 11 + y * 5) as u8,
                    (x * 17 + y * 13) as u8,
                    if alpha_fixture {
                        alphas[(x + y) % alphas.len()]
                    } else {
                        255
                    },
                ]);
            }
        }
        let fixture = pasteboard::RgbaImage { width, height, pixels };
        let input = pasteboard::rgba_to_png(&fixture, CAP)?;
        let before = pasteboard::decode_image(&input, CAP)?;
        let dib = pasteboard::image_to_dib(&input, CAP)?;
        let png = pasteboard::dib_to_png(&dib, CAP)?;
        let tiff = pasteboard::dib_to_tiff(&dib, CAP)?;
        if pasteboard::decode_image(&png, CAP)? != before || pasteboard::decode_image(&tiff, CAP)? != before {
            return Err("PNG/DIB/PNG or TIFF decoded RGBA differs".into());
        }
        let name = if alpha_fixture { "alpha" } else { "opaque" };
        std::fs::write(dir.join(format!("{name}.png")), &input)?;
        std::fs::write(dir.join(format!("{name}.dib")), &dib)?;
        std::fs::write(dir.join(format!("{name}-roundtrip.png")), &png)?;
        std::fs::write(dir.join(format!("{name}-roundtrip.tiff")), &tiff)?;
        println!(
            "{name}: {width}x{height} PNG={} DIB={} PNG={} TIFF={}; decoded RGBA exactly equal ({} bytes)",
            input.len(),
            dib.len(),
            png.len(),
            tiff.len(),
            before.pixels.len()
        );
        if pasteboard::image_to_dib(&input, 64).is_ok() {
            return Err("image cap not enforced".into());
        }
    }
    // Generated CF_DIB fixtures only; no pasteboard or input API is used.
    for top_down in [false, true] {
        let (width, height, stride) = (3usize, 2usize, 12usize);
        let mut dib = vec![0u8; 40 + stride * height];
        dib[..4].copy_from_slice(&40u32.to_le_bytes());
        dib[4..8].copy_from_slice(&(width as i32).to_le_bytes());
        dib[8..12].copy_from_slice(&(if top_down { -(height as i32) } else { height as i32 }).to_le_bytes());
        dib[12..14].copy_from_slice(&1u16.to_le_bytes());
        dib[14..16].copy_from_slice(&24u16.to_le_bytes());
        let expected = vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255,
        ];
        for y in 0..height {
            for x in 0..width {
                let pixel = &expected[(y * width + x) * 4..][..4];
                let row = if top_down { y } else { height - 1 - y };
                dib[40 + row * stride + x * 3..][..3].copy_from_slice(&[pixel[2], pixel[1], pixel[0]]);
            }
        }
        let image = pasteboard::dib_to_image(&dib, CAP)?;
        if image.pixels != expected {
            return Err("24-bpp DIB orientation/colors differ".into());
        }
        let name = if top_down { "24-top-down" } else { "24-bottom-up" };
        std::fs::write(dir.join(format!("{name}.dib")), &dib)?;
        println!("PASS {name}: {width}x{height}, padded rows, exact RGBA");
    }
    if pasteboard::image_to_dib(b"invalid image", CAP).is_ok() || pasteboard::dib_to_png(&[], CAP).is_ok() {
        return Err("malformed image accepted".into());
    }
    println!("PASS: malformed image, empty DIB, and small cap rejected");
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("This probe is available only on macOS.");
    std::process::exit(1);
}
