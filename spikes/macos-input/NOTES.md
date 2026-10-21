# macOS input spike

This is an original implementation from the wire facts in `docs/re/03-core-protocol.md` (§7, §11, Appendices A/B) and Apple's public SDK headers. No Deskflow implementation was used. All changes for this task are inside `spikes/macos-input/`. Nothing was committed.

The crate has no Cargo dependencies. It uses raw, typed FFI to public macOS frameworks and the Objective-C runtime. Its empty `[workspace]` table isolates it from the concurrent root workspace. Copy the library modules, `ffi.rs`, `types.rs`, and the small error/export shim in `lib.rs` together when moving this work into `crates/platform-macos`. The CLI is only a verification harness.

## Verification boundary

Only the six authorized CLI commands were run. No event was posted. No event tap was created, enabled, or started. No cursor was warped, hidden, or shown. No pasteboard content was read or written. `pasteboard-info` read only type names and the change count. No permission request API was called. No unit tests were added or run. The dry-run poster was used only for the explicitly requested numbering simulation.

Verified on arm64, macOS 27.2, Xcode 27.1 (27A9275), SDK 27.0, Rust 1.99.1. See [environment.txt](verification/environment.txt).

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cd spikes/macos-input
cargo build --release
cargo fmt --check
cargo clippy --release -- -D warnings
./target/release/spike keymap-dump
./target/release/spike perms
./target/release/spike displays
./target/release/spike pasteboard-info
./target/release/spike numbering-plan
./target/release/spike dib-roundtrip verification/opaque.png verification/opaque-roundtrip.png
./target/release/spike dib-roundtrip verification/alpha.png verification/alpha-roundtrip.png
```

`cargo build --release` completed with zero warnings. Formatting and Clippy with `-D warnings` also passed. Full outputs are [build.txt](verification/build.txt) and [clippy.txt](verification/clippy.txt).

The safe command results are:

- [keymap-dump.txt](verification/keymap-dump.txt): `com.apple.keylayout.Canadian`; all 128 keycodes for both requested modifier layers; all requested injection samples. Command+C and Control+C both capture `key=99`, with masks `16` and `2`. Option+E captures spacing acute `180`; Option+U captures spacing diaeresis `168`. Caps+A captures `65` with mask `4096`. On this layout, é/ü use Unicode fallback; ß uses Option+S; € uses Shift+Option+2. A snowman also confirms Unicode fallback. Unmapped F35 returns an explicit unsupported-key error. The no-change refresh path returned `false`.
- [perms.txt](verification/perms.txt): `accessibility=true listen=true post=true`; preflight only.
- [displays.txt](verification/displays.txt): main display `4`, one display at `(0,0)`, `2560x1440` points, scale `1x1`; union has the same bounds. Secure input was `true` in the final snapshot, with no public owner name. An earlier snapshot was `false`, so this state changed naturally during the session. This code only reads the state.
- [pasteboard-info.txt](verification/pasteboard-info.txt): change count `153`, four type names: `public.utf8-plain-text`, `NSStringPboardType`, `org.chromium.source-url`, `org.chromium.internal.source-rfh-token`. No payload or URL value was read.
- [numbering-plan.txt](verification/numbering-plan.txt): native down numbers `1001,1002,1003,1004`; both drags and the first up reuse `1001`; each later up reuses its matching down. Down click states are `1,1,2,1`. The system double-click interval is `0.5s`. Legacy omits explicit event numbers. Both sequences finish with an empty ledger.
- [dib-opaque.txt](verification/dib-opaque.txt): `64x48`, input PNG `10678`, DIB `12396`, output PNG `996`, decoded RGBA `12288` bytes. Decoded RGBA buffers are exactly equal. TIFF output is `15654` bytes and also decodes to exactly equal RGBA.
- [dib-alpha.txt](verification/dib-alpha.txt): `64x48`, input PNG `11767`, DIB `12396`, output PNG `3394`, decoded RGBA `12288` bytes. The same exact PNG and TIFF comparisons pass. The fixture includes alpha values `0,1,2,17,64,127,128,200,254,255` and asymmetric colors/rows.

Failure paths use the same real CLI: [dib-invalid.txt](verification/dib-invalid.txt) rejects malformed PNG metadata with exit 1; [dib-oversize.txt](verification/dib-oversize.txt) rejects a compact 5000x5000 PNG because its decoded data exceeds the 64 MiB cap, also with exit 1. Neither command creates an output file. Fixtures were generated from synthetic pixels, with Python's standard PNG chunk/deflate primitives. No user image or clipboard image was used.

## keymap.rs

`Keymap` owns the TIS Unicode layout data through a retained CFData. It obtains that data with `TISCopyCurrentKeyboardLayoutInputSource` and `kTISPropertyUnicodeKeyLayoutData`. Each public translation compares the input-source ID and public Quartz keyboard type to the cached values. A change builds a complete replacement reverse table before swapping it in. No notification observer or extra run loop is needed.

The public Quartz keyboard-type getter replaces legacy `LMGetKbdType`; the latter's SDK contract says it is not thread-safe. The capture thread can therefore construct its own keymap. A missing Unicode layout is an explicit error. An IME uses the underlying keyboard layout exposed by the requested TIS API; this module does not implement the IME composition engine.

Capture passes Shift, Option, and Caps to `UCKeyTranslate`. It never passes Command or Control into character lookup. Those flags still appear in the wire mask. Thus Command+C is logical `c`, mask Super, physical button `8+1=9`. Command maps to Super (`0x0010`), not Meta. Meta and AltGr are accepted on injection as aliases for Command and Option. NumericPad is not treated as NumLock: it marks a key's location. Quartz has no general NumLock or ScrollLock toggle flags, so those wire bits are not invented on capture.

The 60-entry Appendix B table is preserved, including its intentional right/left aliases. Capture recognizes actual right-side Mac virtual keycodes and emits the matching right-side logical key. Return, Tab, Backspace, Escape, forward Delete, F17–F20, and the public volume keycodes supplement the table. Unsupported named keys return errors instead of being typed as private-use Unicode.

Dead-key capture is stateless. It sets `kUCKeyTranslateNoDeadKeysMask` (`1`, not the bit index `0`) with a fresh zero dead state. It reports the layout's spacing accent; if the layer supplies no single character, it tries the same key without Option. It never silently composes a later captured key with an earlier dead key. A decomposed layout result is NFC-normalized only if that makes one BMP character. Other multiple-character outputs and supplementary Unicode have no 16-bit wire KeyID and remain unmapped.

The reverse cache searches keycodes `0..127` with none, Shift, Option, Shift+Option, and Caps. It also searches bounded two-stroke dead-key completions. It prefers fewer strokes, then modifier layers closest to the requested mask, then fewer modifier bits. The plan carries complete required flags per stroke. Character-producing Shift/Option/Caps layers take precedence over incompatible remote layers; Control and Command remain shortcut flags. No representable layout sequence means `InjectionPlan::Unicode`, emitted with `CGEventKeyboardSetUnicodeString`. Longer dead-key sequences also use this fallback.

## inject.rs and the focus fix

`Injector<NativePoster>` owns one persistent HID-system-state CGEventSource (`1`) and posts globally at `kCGHIDEventTap` (`0`). The source uses 10 pixels per line and no local-event suppression interval. Every posted event carries the Openergy self-event tag. `EventPoster` isolates native posting from the required dry-run numbering command. `Injector::with_poster` can omit a keymap for a mouse-only simulation.

Each native mouse-down reads the sum of HID-system left, right, and other down counters. The numbering state computes:

```text
number = max(last_number, seed) + 1
last_number = number
pressed_number[button] = number
```

The addition is checked for overflow. The native poster writes `kCGMouseEventNumber` on the down, on dragged events for that button, and on its up. Down, drag, and up share the down's click state. Legacy mode only omits that explicit number field. Neither mode performs window guessing, sets guessed window fields, raises an AX window, or calls `NSRunningApplication.activate`.

Click history is per button. A new down advances from 1 to 2 to 3 only after a completed click, within `NSEvent.doubleClickInterval` and within Euclidean distance 4 global points. A drag beyond that distance breaks the next click chain. A fourth click starts at 1. The most recently pressed held button owns a dragged movement when several buttons are held. Other held buttons retain their own up identities, and movement invalidates their click chains too.

Wire buttons map `1→0`, `2→2`, `3→1`, `4→3`, `5→4`. Invalid buttons are errors. Absolute moves include delta fields; relative native moves first read the current cursor position. Wheel input is signed `(horizontal,vertical)`, `120` units per notch, default 3 lines per notch. Each axis retains fractional line remainders; emitted whole-line events have matching fixed-point line fields and 10-pixel-per-line point fields. Invalid settings and out-of-range line deltas fail before posting.

Keyboard downs store the resolved native key against the source physical button. Temporary modifiers are pressed only as needed, incompatible layers are released, and the previous modifier state is restored. Dead-key prefixes are complete down/up strokes. Their final base key stays down until the source button's up. Composed-character repeats use the logical Unicode glyph so they do not replay a dead prefix. Up uses the original mapping, even if the layout changes. Modifier aliases that share a native key have a shared hold count. Caps is a toggle; up does not clear its lock state. Native construction samples the existing Caps state.

`release_all` walks successful key/button downs in reverse arrival order. It also retains incomplete temporary key taps so cleanup can release them after a posting failure. It tries the remaining releases after an error and leaves failed entries available for retry. Cursor visibility has a local nesting counter: one native hide on 0→1 and one show on 1→0, against the same display ID. Extra show calls do nothing. `release_all` balances outstanding hides. Drop never posts or changes cursor state; the owner must call cleanup explicitly.

The required numbering behavior is verified through the dry poster. Actual inactive-window activation on macOS 27 is **not verified** on this workstation.

## pasteboard.rs

`info()` and `change_count()` are metadata-only APIs. Content APIs are separate and were not invoked. `read(max_bytes)` fetches only the four requested UTIs, prefers PNG over TIFF, converts images to wire DIB, and checks changeCount again before returning. A changing clipboard returns a retryable error. Plain text and HTML use UTF-8. Read line endings normalize CRLF and CR to LF; writes preserve wire LF as-is.

The cap applies to compressed input, decoded RGBA size, emitted DIB size, and total returned wire formats. ImageIO checks pixel metadata before NSBitmapImageRep decodes pixels. Arithmetic is checked. NSPasteboard has no public length-only query, so the system may allocate an NSData before this code can inspect its length. Native codec scratch allocations and encoded PNG/TIFF allocation before its final length check are also outside a strict allocation bound. The cap is a payload/raster limit, not a hard process-memory ceiling.

`write` validates all formats, constructs image representations and an NSPasteboardItem first, then replaces the general pasteboard. It publishes both `public.png` and `public.tiff`. macOS has no atomic replace-with-rollback contract; a failed final write may leave a cleared pasteboard. This behavior needs integration-level error handling.

The emitted wire bitmap is a 108-byte little-endian BITMAPV4HEADER followed by 32-bpp bottom-up BGRA rows. It uses BI_BITFIELDS (`3`) and masks `00FF0000`, `0000FF00`, `000000FF`, `FF000000` for R/G/B/A, with LCS_sRGB. There is **no 14-byte BMP file header** on the wire. Every 32-bpp row is already DWORD-aligned. Width and height are positive; height defines bottom-up storage.

The reader accepts 40/52/56/108/124-byte DIB headers, BI_RGB or BI_BITFIELDS, positive bottom-up dimensions, and single-plane 32-bpp pixels. It validates sizes and contiguous, nonoverlapping channel masks before native decoding. BI_RGB's reserved high byte is opaque, not alpha; BITFIELDS honors an explicit alpha mask. Palettes, compressed pixel streams, top-down DIB, and embedded V5 profiles are rejected explicitly.

For image writes, the converter adds a correct BMP file header and obtains an NSBitmapImageRep. It then reads validated DIB bit masks itself to preserve alpha and channel interpretation across native BMP decoder differences. PNG and TIFF are encoded through a normalized, explicitly sRGB-tagged NSBitmapImageRep. Merely using the legacy calibrated-RGB initializer changed colors; the real CLI caught that, and explicit sRGB retagging fixed it.

Decoded comparisons use canonical sRGB RGBA8. Native rendering first produces premultiplied RGBA8; the converter unpremultiplies with rounding and sets invisible RGB to zero at alpha 0. The comparisons prove exact equality of those decoded buffers, not preservation of original 16-bit channel precision, metadata, profiles, or hidden transparent RGB bytes.

## permissions.rs, displays.rs, tap.rs, ffi.rs

`permissions::preflight` calls AXIsProcessTrusted, CGPreflightListenEventAccess, and CGPreflightPostEventAccess. Explicit request functions exist separately. Only the AX request builds the prompt options dictionary. The CLI never calls any request function.

`displays::active` enumerates active displays, reports per-display global-point bounds, computes a union that includes negative coordinates, and reports backing/logical mode ratios and the main display. It retries bounded enumeration changes. Secure input uses public IsSecureEventInputEnabled. No supported public API provides its owning PID/name, so `owner_app` is `None`; no undocumented session or IORegistry keys are used.

`EventTap::start` is the only tap-creation entry point. It is not reachable from the CLI. It creates a session tap, head insert, default options, on a dedicated CFRunLoop thread. The callback emits the local InputEvent enum, including separate modifiers, repeats, absolute/relative mouse modes, mapped buttons, and normalized wheel units. Key-up retains the original down's logical identity. Caps flag changes become balanced logical down/up pairs. The callback passes Openergy-tagged events, re-enables disabled-tap notices, and uses an AtomicBool to choose swallow/pass for translated events. Translation errors or a disconnected consumer fail open. Panics cannot unwind across the C ABI.

The event channel is std::mpsc and never waits for the consumer. The owner must drain it promptly to avoid unbounded queue growth. Keep network I/O off the tap thread. The owner must also monitor permission/secure-input state; auto re-enable cannot override revoked trust. The loop has bounded 250 ms runs to avoid a stop-before-run race, separate retained references for thread/controller lifetimes, and an explicit stop/wakeup/join path. Invalidated sources exit instead of spinning.

`ffi.rs` contains the only raw ABI declarations and Objective-C message helpers. Call sites select static public selectors with matching argument and return types. No struct-return Objective-C selector is used. CF create/copy results and retained Objective-C objects have RAII release wrappers; autorelease pools bound native temporaries. The injection and clipboard library APIs still need review in their final application context.

## Work left for an isolated test machine

- Verify actual native click activation in several inactive apps in Native versus Legacy mode. Check controls, title bars, modal dialogs, nonactivating panels, all five buttons, double/triple clicks, drags, and simultaneous buttons. Do not infer activation from this dry-run proof.
- Post real keyboard plans with shortcuts, Caps, both modifier sides, dead-key prefixes, Unicode fallback, repeats, and release_all after disconnect. A normal native repeat keeps the down's physical mapping; check glyph behavior when changing layout during a held key. Test application shortcut behavior when remote modifier layers differ from the local layout.
- Change layouts while a keymap stays alive to exercise actual cache replacement. This session verified only the unchanged-ID path. Check non-Latin layouts, missing Unicode data, and IMEs. F21–F35, most media/browser keys, Fn, and other keys without public virtual-key mappings remain explicit unsupported cases. No private system-defined-event synthesis is included.
- Start the tap, verify real capture and suppression, right/left modifier transitions, fractional wheels/trackpads, self-event filtering, disabled-tap re-enable, shutdown during capture, consumer loss, and permission revocation. The tap was compile-checked only.
- Exercise real pasteboard reads/writes with a private test clipboard, text/HTML LF handling, TIFF input, BI_RGB input, other valid BITFIELDS masks, concurrent clipboard changes, hostile DIB headers, caps near every allocation boundary, and another app consuming the resulting PNG/TIFF. Only synthetic PNG→DIB→PNG/TIFF data paths ran here.
- Check negative-origin multi-display layouts, Retina scaling, rotation, hot-plug, cursor warp/visibility balance, and secure-input transitions. This workstation had one 1x display. Add secure-input owner text only if a public supported API becomes available.

Primary Apple references: installed `CGEvent.h`, `CGEventTypes.h`, `CGEventSource.h`, `Events.h`, `TextInputSources.h`, `UnicodeUtilities.h`, `NSPasteboard.h`, `NSBitmapImageRep.h`, `CGImageSource.h`, and `CGImageProperties.h`; [UCKeyTranslate](https://developer.apple.com/documentation/coreservices/1390584-uckeytranslate), [dead-key option bit](https://developer.apple.com/documentation/coreservices/1390507-key_translation_options_flag/kuckeytranslatenodeadkeysbit?language=objc), and [CGEventSource](https://developer.apple.com/documentation/coregraphics/cgeventsource).
