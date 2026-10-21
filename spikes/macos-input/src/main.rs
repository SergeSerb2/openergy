//! Deliberately restricted to read-only platform checks and pure-data paths.
use std::collections::HashMap;
use std::path::Path;

use macos_input::inject::{ClickNumbering, DryRunPoster, Injector, MouseKind, SynthEvent};
use macos_input::types::SUPER;
use macos_input::{Result, displays, inject, keymap, pasteboard, permissions};

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    let command = arguments.first().and_then(|arg| arg.to_str()).unwrap_or("");
    match (command, arguments.len()) {
        ("keymap-dump", 1) => keymap_dump()?,
        ("perms", 1) => {
            let state = permissions::preflight();
            println!(
                "accessibility={} listen={} post={} (preflight only; no prompts)",
                state.accessibility, state.listen, state.post
            );
        }
        ("displays", 1) => {
            let displays = displays::active()?;
            println!(
                "main_display={} active_displays={}",
                displays.main_id,
                displays.active.len()
            );
            for display in displays.active {
                println!(
                    "display={} main={} rect=({},{}) {}x{} points scale={}x{}",
                    display.id,
                    display.main,
                    display.bounds.x,
                    display.bounds.y,
                    display.bounds.width,
                    display.bounds.height,
                    display.scale_x,
                    display.scale_y
                );
            }
            println!("union={:?}", displays.union);
            let secure = displays::secure_input();
            println!(
                "secure_input={} owner={}",
                secure.enabled,
                secure.owner_app.as_deref().unwrap_or("unavailable through public API")
            );
        }
        ("pasteboard-info", 1) => {
            let info = pasteboard::info()?;
            println!("change_count={} types={}", info.change_count, info.types.len());
            for uti in info.types {
                println!("{uti}");
            }
            println!("metadata only; no clipboard contents read");
        }
        ("dib-roundtrip", 3) => dib_roundtrip(Path::new(&arguments[1]), Path::new(&arguments[2]))?,
        ("numbering-plan", 1) => {
            numbering_plan(ClickNumbering::Native)?;
            numbering_plan(ClickNumbering::Legacy)?;
        }
        _ => return Err(
            "usage: spike {keymap-dump|perms|displays|pasteboard-info|dib-roundtrip <png> <out.png>|numbering-plan}"
                .into(),
        ),
    }
    Ok(())
}

fn keymap_dump() -> Result<()> {
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
        ("Return", 0xEF0D, 0),
        ("Tab", 0xEF09, 0),
        ("F5", 0xEFC2, 0),
        ("Left", 0xEF51, 0),
        ("KP_5", 0xEFB5, 0),
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
    match keymap.plan(0xEFE0, 0) {
        Err(err) => println!("F35 unsupported (expected): {err}"),
        Ok(plan) => println!("F35: {plan:?}"),
    }
    println!("layout_refresh_changed={}", keymap.refresh()?);
    Ok(())
}

fn dib_roundtrip(input: &Path, output: &Path) -> std::result::Result<(), Box<dyn std::error::Error>> {
    const MAXIMUM: usize = 64 * 1024 * 1024;
    // Metadata guards the file read itself, before the codec's own limits.
    let length = input.metadata()?.len();
    if length > MAXIMUM as u64 {
        return Err("input file exceeds 64 MiB".into());
    }
    let source = std::fs::read(input)?;
    if !source.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err("input must be a PNG image".into());
    }
    let before = pasteboard::decode_image(&source, MAXIMUM)?;
    let dib = pasteboard::image_to_dib(&source, MAXIMUM)?;
    let png = pasteboard::dib_to_png(&dib, MAXIMUM)?;
    let after = pasteboard::decode_image(&png, MAXIMUM)?;
    if before != after {
        let differences = before
            .pixels
            .iter()
            .zip(&after.pixels)
            .filter(|(left, right)| left != right)
            .count();
        return Err(format!("decoded RGBA differs: {differences} channel values").into());
    }
    let tiff = pasteboard::dib_to_tiff(&dib, MAXIMUM)?;
    if pasteboard::decode_image(&tiff, MAXIMUM)? != before {
        return Err("TIFF decoded RGBA differs from input".into());
    }
    // The caller gets an output only after exact decoded-buffer comparison.
    std::fs::write(output, &png)?;
    println!(
        "dimensions={}x{} input_png={} DIB={} output_png={} decoded_RGBA={}",
        before.width,
        before.height,
        source.len(),
        dib.len(),
        png.len(),
        before.pixels.len()
    );
    println!("decoded RGBA buffers: exactly equal");
    println!("TIFF={} bytes; decoded RGBA buffers: exactly equal", tiff.len());
    println!("output={}", output.display());
    Ok(())
}

fn numbering_plan(mode: ClickNumbering) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let interval = inject::system_double_click_interval();
    let poster = DryRunPoster {
        seed: 1000,
        events: Vec::new(),
    };
    let mut injector = Injector::with_poster(poster, None, mode, interval, (100.0, 100.0))?;
    injector.mouse_down_at(1, 0.0)?;
    injector.move_absolute(110.0, 100.0)?;
    injector.move_relative(10.0, 10.0)?;
    injector.mouse_up(1)?;
    // A completed drag breaks the multi-click chain. These next two downs
    // form a new double-click within NSEvent.doubleClickInterval.
    injector.mouse_down_at(1, 1.0)?;
    injector.mouse_up(1)?;
    injector.mouse_down_at(1, 1.0 + interval / 2.0)?;
    injector.mouse_up(1)?;
    injector.mouse_down_at(3, 2.0 + interval)?;
    injector.mouse_up(3)?;
    injector.release_all()?;
    println!("mode={mode:?} seed=1000 double_click_interval={interval}s (dry-run poster; zero native events)");
    let mut held = HashMap::new();
    let mut previous = 1000;
    let mut states = Vec::new();
    for event in &injector.poster().events {
        if let SynthEvent::Mouse {
            kind,
            button,
            number,
            click_state,
            ..
        } = event
        {
            println!(
                "{} wire_button={} CG_button={} kCGMouseEventNumber={} click_state={}",
                inject::event_type_name(event),
                button,
                inject::wire_to_cg_button(*button).unwrap(),
                number.map_or_else(|| "unset".into(), |number| number.to_string()),
                click_state
            );
            match kind {
                MouseKind::Down => {
                    if mode == ClickNumbering::Native {
                        let number = number.ok_or("missing down number")?;
                        if number <= previous {
                            return Err("down numbers did not increase".into());
                        }
                        previous = number;
                    }
                    held.insert(*button, (*number, *click_state));
                    states.push(*click_state);
                }
                MouseKind::Drag | MouseKind::Up => {
                    if held.get(button) != Some(&(*number, *click_state)) {
                        return Err("drag/up identity differs from its down".into());
                    }
                    if *kind == MouseKind::Up {
                        held.remove(button);
                    }
                }
                MouseKind::Move => {}
            }
        }
    }
    if !held.is_empty() || states != [1, 1, 2, 1] {
        return Err("unbalanced clicks or incorrect multi-click state".into());
    }
    println!("verified: increasing native down numbers; drag/up reuse; click states 1,1,2,1; ledger empty");
    Ok(())
}
