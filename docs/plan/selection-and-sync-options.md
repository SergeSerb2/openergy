# Selection and sync options: PRIMARY, keyboard language, half-duplex locks

- Status: design + implementation-ready specification. No code in this change.
- Tickets: SER-221 (X11 PRIMARY), SER-215 (keyboard language sync), SER-216
  (half-duplex lock keys).
- Date: 2026-11-09.
- Verification rule for all three: end-to-end only (real X server probe,
  `tools/e2e` scenarios, interop lab). No unit tests, no mocks.

## Summary and order

| Order | Feature | Size | Why |
|---|---|---|---|
| 1 | (c) Half-duplex locks, SER-216 | S | Config, DSOP ids, and lock level-sync already exist. The work is session-local event rewriting. No platform trait change. |
| 2 | (a) PRIMARY selection, SER-221 | M | Linux backend and session slot arrays already carry two ids. The work is generalizing one worker/trait path and opening gates that today stop at id 0. |
| 3 | (b) Keyboard language sync, SER-215 | L | Wire fields exist, but no platform can report or set a language today. It needs a new platform capability on three OSes, and the language-to-layout mapping is lossy. |

**Smallest first PR: (c), half-duplex locks, receiving side plus server
suppression, as specified in §1.** It touches `crates/session` and
`crates/config/V1-LIMITS.md` only, adds no trait, and is verifiable with an
e2e scenario plus the existing `lock-recovery` probe mode pattern.

## 0. Shared facts from the current code

These facts bind all three designs.

- Wire. `crates/protocol/src/lib.rs`: `ClipboardId::CLIPBOARD = 0`,
  `ClipboardId::SELECTION = 1`. DSOP option ids exist for clipboard sharing
  (`CLPS`, `CLSZ`) and half-duplex locks (`HDCL`, `HDNL`, `HDSL`).
  `Message::LanguageSync` (`LSYN`), `KeyDown.lang` (`DKDL` vs `DKDN`), and
  `KeyRepeat.lang` (`DKRP`) are already encoded/decoded in
  `crates/protocol/src/codec.rs`.
- Session clipboard state is already two-wide: server `slots: [Slot; 2]`
  (`crates/session/src/server.rs`), client `clip_sequences: [u32; 2]` and
  `local_owner: [bool; 2]` (`crates/session/src/client.rs`), and
  `Transfers::receive` accepts `id.0 <= 1`
  (`crates/session/src/clipboard.rs`). But the message handlers gate on
  `id.0 == 0` (client `ClipboardGrab`/`ClipboardData`, server
  `ClipboardGrab`/`ClipboardData`), so id 1 never moves.
- Daemon clipboard path is one-wide: `workers::clipboard` polls one
  `change_stamp()` every 200 ms when enabled and suppresses own writes by
  re-reading the stamp after a write (`crates/daemon/src/workers.rs`);
  `Machine::clipboard_changed` hardcodes `ClipboardId::CLIPBOARD` and
  `Action::SetClipboard { id: _, .. }` discards the id
  (`crates/daemon/src/runtime.rs`).
- Platform clipboard trait is single-instance: `ClipboardBackend`
  (`change_stamp`/`read`/`write`/`write_shared`) and `Backend.clipboard: Box<dyn ClipboardBackend>`
  in `crates/platform/src/lib.rs`. Linux is the exception underneath:
  `X11Clipboard` keeps `stamps: [AtomicU64; 2]`, per-selection
  `data`/`owned`, `Selection::{Clipboard, Primary}`, and
  `for_selection`/`read_selection`/`write_selection`
  (`crates/platform-linux/src/clipboard.rs`). `create()` exposes only
  `Selection::Clipboard` (`crates/platform-linux/src/lib.rs`).
- Config: `ClipboardCfg { enabled, send, receive, max_bytes }` defaults all
  on; `InputCfg.sync_language` defaults `true`; per-screen
  `ScreenOptions { half_duplex_caps_lock, … : Option<bool> }`
  (`crates/config/src/lib.rs`). All use `#[serde(default, deny_unknown_fields)]`.
- Lock state today is level-based, not event-based: sources toggle the
  `Modifiers::{CAPS_LOCK, NUM_LOCK, SCROLL_LOCK}` bits on key-down and keep
  them across key-up (Linux `Capture::key` in
  `crates/platform-linux/src/source.rs`; Windows hook in
  `crates/platform-windows/src/source.rs`); sinks re-toggling a mismatch with
  a press/release pair before any non-lock key-down (Linux `Sink::down`,
  Windows `Sink::down`). macOS reports CAPS only; `flags_to_mask` notes
  NumericPad is a location, not a toggle (`crates/platform-macos/src/keymap.rs`).
- Language today: server sends setup `LSYN ""` to wire >= 1.8, then
  `DKDN`/`DKRP` with `lang: None`/`""` (`wire_input` in
  `crates/session/src/common.rs`). Client stores `Peer.languages: Vec<[u8; 2]>`
  during setup phase only; `Peer::observe_language` logs once that sync is not
  implemented. See `crates/config/V1-LIMITS.md`.

## 1. (c) Half-duplex lock keys — SER-216, size S, do first

### 1.1 Terms

- Full-duplex lock (today): one physical press makes KeyDown + KeyUp. The
  toggle state lives in the modifier mask. The receiving sink presses the real
  lock key to toggle, and level-syncs any drift on the next key.
- Half-duplex lock: the screen's OS/keyboard gives only one edge per state
  change (down when the lock turns on, up when it turns off), or cannot hold
  the lock key as a normal held key. Forwarding a normal down+up pair to such
  a screen, or forwarding its single edges as a pair, toggles twice or leaves a
  key held in the sink ledger.

### 1.2 Decision: session rewrites edges; sinks stay level-based

Per lock `L` in {Caps `0xEFE5`, Num `0xEF7F`, Scroll `0xEF14`} (key ids in
`crates/protocol/src/keys.rs`), when the *receiving screen's*
`half_duplex_<L>` is `true`:

1. Receiving side (openergy client): an incoming lock KeyDown is injected as
   an atomic Down+Up pair to the sink. The pair is not entered as a held key
   in the client `Ledger`. An incoming lock KeyUp is dropped in session. The
   sink therefore toggles once, and its existing pre-key level sync
   (actual XOR mask → press/release pair) still corrects drift, and must not
   fire for the lock key itself (already guarded by `key != lock_key`).
2. Sending side (openergy server): do not emit lock KeyUp or KeyRepeat for `L`
   to a peer whose screen has the flag; emit KeyDown once with the post-toggle
   mask. Non-flagged peers get today's stream unchanged.
3. `Enter` (CINN) mask handling is unchanged: locks are not in the
   `synthetic_modifiers & 0x3f` set; the first non-lock key level-syncs them.
4. Flag is `false`/absent: behavior is exactly today, bit for bit.

Rejected: changing sink toggle logic per platform. The sinks already express
"make OS state equal mask". The defect is duplicate/held edges in the session
stream, so the fix belongs where edges are translated.

### 1.3 Touch points

| File | Change |
|---|---|
| `crates/session/src/common.rs` | Add `half_duplex: [bool; 3]` to `Options`; set it in `Options::apply` from `o::HALF_DUPLEX_CAPS_LOCK/NUM_LOCK/SCROLL_LOCK` (today stored only in `raw`). Add `is_half_duplex_lock(key) -> bool`. |
| `crates/session/src/common.rs` | `config_options` unchanged. `screen_options` already emits HDCL/HDNL/HDSL from `ScreenCfg.options`; no change. |
| `crates/session/src/client.rs` | In the `input_message` result path: if event is KeyDown of a half-duplex lock, inject Down then Up and skip `ledger.observe` for the hold; if KeyUp/KeyRepeat of such a lock, return early. Must run before the synthetic-modifier KeyUp block so it is not mistaken for a CINN modifier release. |
| `crates/session/src/server.rs` | In `input()`, before `wire_input` send: if active peer's `options.is_half_duplex_lock(key)` and event is KeyUp/KeyRepeat for that lock, drop. Note: ScrollLock KeyDown (`0xef14`) also toggles lock-to-screen when local; keep that check first, exactly as today — on a remote screen it must still forward as a lock edge. |
| `crates/session/src/common.rs` | `Ledger::release_remote` must not emit a KeyUp for a lock that was injected as a pair. With rule 1 the client ledger never holds it; assert that invariant in code, do not add a special case unless the e2e in §1.5 fails. |
| `crates/config/V1-LIMITS.md` | Remove the half-duplex no-op sentence; state the §1.2 rule in one sentence. |

Config shape: **no change**. `screen[].options.half_duplex_{caps,num,scroll}_lock`
already parse from TOML, synergy.conf (`halfDuplexCapsLock`, … in
`crates/config/src/synergy.rs`), and Synergy 3 `fix.capslock/num_lock/scr_lock`
(`crates/config/src/synergy3.rs`). `Option<bool>` keeps "inherit unset = off".

IPC/status: **no new method or event**. Value settable today via
`config.patch`/`config.set` (whole-config model). Mentions in `Status` are not
added; observable evidence is input behavior, and one `log::info!` when a peer's
effective half-duplex set changes in `Options::apply` (throttle: only on change).

### 1.4 Wire compatibility (honest)

- No new wire bytes. HDCL/HDNL/HDSL in DSOP are classic Synergy options;
  openergy already sends them when configured, so Deskflow/Barrier servers
  already receive them from us. What changes is only that our client now obeys
  them and our server filters edges for flagged peers.
- Deskflow 1.27 / Barrier 2.4 accept these per-screen options in
  synergy.conf and implement their own variant for X11 toggle handling.
  Whether their variant matches §1.2 edge-for-edge is *not assumed*; §1.5
  scenario 4 checks it in the interop lab.
- Risk: servers that send a lock KeyDown and expect the client to hold until
  KeyUp would see a toggle instead of a hold. That expectation does not exist
  for toggle keys in this protocol — the mask carries the state — so this is
  safe for the peers in the lab. Flag off remains the default everywhere.

### 1.5 Verification (end-to-end)

1. New e2e scenario `tools/e2e/scenarios/S26-half-duplex-locks.toml`
   (virtual server + virtual client; client screen options set
   `half_duplex_caps_lock = true` in node config). Steps: enter client;
   `inject` CapsLock down/up then `a` down/up on the server;
   `assert_recorded` on the client shows Caps injected as down+up with no
   held tail and `a` with CAPS mask; repeat Caps press → mask clears.
   Control run in the same scenario with flag off shows today's held
   down…up sequence. Also assert that leaving the screen
   (`release_all`) injects no stray Caps KeyUp.
2. New e2e scenario `P10-half-duplex-raw-peer.toml` (raw node as server):
   send DSOP *without* HDCL, then DKUP for a lock never pressed; client must
   behave as today (ignore per ledger rules), proving default path unchanged.
3. New `x11_probe` mode `half-duplex-lock` (isolated Xvfb `:99` via
   `tools/linux-dev/`): drive the real Linux sink with the §1.2 pair for
   Caps/Num; assert via `xinput query-state`/`query_pointer` mask that the
   lock toggled once and no key remains down; then drive a normal down/up and
   assert the same end state. Follow the existing `lock-recovery` mode
   layout in `crates/platform-linux/examples/x11_probe.rs`.
4. Interop lab (`tools/interop-lab/lab.sh`): Deskflow server with
   `halfDuplexCapsLock = true` for the openergy screen; toggle caps in the
   lab X session, type a letter, read the letter case back. Repeat with
   Barrier. Record any mismatch as a lab finding, not by changing §1.2
   silently.

## 2. (a) X11 PRIMARY selection sharing — SER-221, size M, do second

### 2.1 Decision: a second ClipboardId, never piggybacked

Use the existing `ClipboardId::SELECTION (1)` for PRIMARY on `CCLP`/`DCLP`.
Do not merge PRIMARY into CLIPBOARD id 0, and do not add a new wire option.

- The protocol already has the channel: two clipboard slots are the
  cross-platform model in the reference build (ids 0 and 1,
  `docs/re/03-core-protocol.md`, CCLP row). Codec, `Transfers`, server slots,
  and client owner/sequence arrays already carry id 1 end to end.
- Piggybacking (PRIMARY change → write id 0) would break the core X11
  contract: selecting text must not overwrite the explicit clipboard, and
  middle-click paste must paste the selection, not the last copy. It would
  also make every selection drag clobber a peer clipboard.
- No DSOP id exists for PRIMARY in peers. `CLPS` governs clipboard sharing as
  a whole. Inventing an option id would be ignored by peers at best.

Delivery rules for id 1, when locally enabled (§2.3):

| Sender → Receiver | id 1 behavior |
|---|---|
| X11 → X11 | Full PRIMARY ↔ PRIMARY. Middle-click (mouse button 2, already forwarded) pastes it. |
| X11 → macOS/Windows/virtual-single | Drop at the receiver. Never write the system pasteboard/clipboard. Log once per connection. |
| macOS/Windows → anyone | Never originate id 1. |
| Openergy server relay | Forward id 1 only to a peer that is known X11-capable: its `ScreenCfg.os` is `linux`, or it has itself sent a `CCLP` id 1 on this connection. Unknown peers get id 0 only. |

The relay restriction is needed because reference peers on macOS/Windows treat
both ids against one system clipboard — the client code already guards this
case ("platform clipboard is shared on macOS; an empty X11 selection must not
erase a populated normal clip"). Sending selection churn there would destroy
their clipboard.

Empty PRIMARY is **not** a clear event. Selection owners die whenever an app
exits or a selection is unhighlighted. If a PRIMARY read returns empty formats
or `ClipboardUnavailable`, send nothing and keep prior ownership state. This
differs from id 0, where empty is a real clear and must stay so.

### 2.2 Loop prevention

Three layers exist and extend per-id. No new mechanism is allowed.

1. Platform stamps: Linux `Inner.stamps[2]` are independent; a PRIMARY write
   bumps only stamp 1. The daemon worker keeps one `stamp` and sets
   `stamp = backend.change_stamp()` after its own `Write`
   (`crates/daemon/src/workers.rs`). This becomes `stamps: [u64; 2]`; an own
   write to id *i* re-reads only stamp *i*. A remote PRIMARY write therefore
   never re-fires as a local CLIPBOARD grab, and vice versa.
2. Session ownership: server `slots[2]` and client
   `local_owner`/`clip_sequences[2]` are already per-id. `CCLP` sequence
   comparison and `Transfers::cancel(id)` are per-id. Keep exactly these
   semantics for id 1.
3. Churn coalescing: PRIMARY changes on every selection motion. The worker
   polls at 200 ms; for id 1 only, require the stamp to be unchanged across
   two consecutive polls before emitting `Changed(1)`. `Transfers::enqueue`
   already drops an unstarted transfer for the same id when a newer one is
   queued. `clipboard.max_bytes`/`HARD_CAP` apply to id 1 unchanged.

### 2.3 Config and IPC

```toml
[clipboard]
enabled = true
send = true
receive = true
share_primary_selection = false   # new, opt-in
max_bytes = 67108864
```

| Item | Shape |
|---|---|
| `ClipboardCfg.share_primary_selection: bool` | Default `false`. Mirrors `enabled/send/receive` in `crates/config/src/lib.rs`. Effective primary-send = `enabled && send && share_primary_selection`; same for receive. |
| synergy.conf import/export | No classic key exists. Export warns like directional send/receive in `crates/config/src/cli.rs`: "synergy.conf cannot represent PRIMARY selection sharing". Import leaves `false`. |
| Synergy 3 import | No mapping; do not reuse `fix.*`. Stays `false`. |
| IPC | No new method. `config.patch` carries it. Event payloads gain `"selection": "clipboard" \| "primary"`: `clipboard.transfer` and `clipboard.blocked` in `crates/session/src/clipboard.rs`, so `ClipboardTransfer`/`ClipboardBlocked` in `crates/ipc/src/lib.rs` gain `selection: Option<String>` (additive, `Option` for old consumers). |
| `Status.clipboard` | Add `primary_enabled: bool` to `ClipboardInfo`, set from config ∧ Linux backend in `Runtime` status build (`crates/daemon/src/runtime.rs`). `last_transfer` continues to show the latest of either id, now tagged with `selection`. |

### 2.4 Touch points

| File | Change |
|---|---|
| `crates/platform/src/lib.rs` | `ClipboardBackend` gains defaulted multi-selection methods: `change_stamp_for(id)`, `read_for(id, max)`, `write_for(id, data)` where `id: u8`. Defaults: id 0 delegates to current methods; id 1 returns `PlatformError::NotImplemented` / stamp 0. Existing backends compile unchanged. Do not add a second `Backend.clipboard` field. |
| `crates/platform-linux/src/clipboard.rs` | Implement the three methods on `X11Clipboard` by mapping id 0→`Selection::Clipboard`, 1→`Selection::Primary` onto existing `read_selection`/`write_selection` and `stamps[i]`. Update the "post-v1 extension" comments. |
| `crates/platform/src/virtual_backend.rs` | Second slot: `primary: Arc<ClipboardData>`, `primary_stamp`, own-write stamp per id, mirroring `clipboard_set_shared`. Needed so e2e can run without X11. |
| `crates/platform-macos/src/pasteboard.rs`, `crates/platform-windows/src/clipboard.rs` | No override. Default `NotImplemented` for id 1 is the honest behavior. |
| `crates/daemon/src/workers.rs` | `ClipboardCommand::Read/Write` already carry `ClipboardId` on Read; add `id` to `Write`. `ClipboardEvent::Changed` becomes `Changed(ClipboardId)`. Poll both stamps; id-1 two-poll stability rule from §2.2; per-id own-write stamp refresh. |
| `crates/daemon/src/runtime.rs` | `clipboard_changed(id)` replaces the hardcoded CLIPBOARD; `Action::SetClipboard/SetClipboardShared` stop discarding `id`; `Enabled` gating passes the primary flag into the worker (worker ignores id 1 stamps when off). |
| `crates/daemon/src/adapter.rs` (+ `virtual.clipboard`) | Accept optional `"selection": "primary"` on get/set so e2e steps can drive the virtual primary slot; default stays clipboard. |
| `crates/session/src/server.rs` | Replace `id.0 == 0` gates on `ClipboardGrab`/`ClipboardData` with `id.0 < 2 && primary_allowed(id)` where `primary_allowed` = cfg flag for id 1, always true for id 0. In `push_slots`, skip slot 1 for peers failing the §2.1 relay rule. Track per-peer `sent_primary_grab: bool` on `Peer`. |
| `crates/session/src/client.rs` | Same gate change. Keep the shared-clipboard empty-guard, now reachable: drop empty id-1 payloads. `LocalClipboardChanged`/`payload` already index `[u32; 2]`/`[bool; 2]`; add cfg-flag check for id 1. |
| `crates/session/src/common.rs` | `Peer` gains `sent_primary_grab: bool` and `primary_drop_logged: bool`. |
| `crates/config/src/lib.rs`, `cli.rs` | §2.3 field + export warning. `crates/config/V1-LIMITS.md` notes PRIMARY is X11-only and off by default. |
| `crates/platform-linux/README.md`, `VERIFICATION.md` | Update the "v1 routes only id 0 / no shared API change needed" claims. |

Explicitly unchanged: `crates/protocol/*` (no wire change), Wayland (there is
no Wayland backend; PRIMARY there is out of scope until one exists).

### 2.5 Wire compatibility (honest)

- Deskflow/Barrier on X11: they implement both clipboard ids, with id 1 as
  the X selection. Expect interop, but Barrier's defaults around selection
  sharing have varied by build, so treat this as *verify in the lab*, §2.6.4.
- Deskflow/Barrier/Synergy on macOS/Windows: both ids resolve to the single
  system clipboard. That is why §2.1 drops at our receiver and restricts
  server relay. An openergy server must never blast id 1 at an unknown peer.
- Wire < 1.6 peers have no chunked clipboard at all; the existing
  `p.version.minor >= 6` gates already cover both ids.
- A peer that ignores id 1 `CCLP`/`DCLP` breaks nothing: ids are independent
  slots, and unknown-id `DCLP` (`id.0 > 1`) is already discarded in
  `Transfers::receive`.

### 2.6 Verification (end-to-end)

1. Probe mode changes (`crates/platform-linux/examples/x11_probe.rs`, via
   `tools/linux-dev/`, `DISPLAY=:99`): the current `clipboard` mode asserts
   "PRIMARY does not change shared CLIPBOARD data or stamp" — **keep and
   extend that assertion** (isolation is still required). Add mode
   `primary-sync`: two `X11Clipboard` handles on the same server;
   `xclip -selection primary -i` → read id 1, write id 1 →
   `xclip -selection primary -o` matches; assert CLIPBOARD bytes and stamp
   untouched in both directions; middle-click check: inject button-2 press
   into `xev` window and confirm a paste request targets PRIMARY (selection
   request observed), reusing the `xev`/`xdotool` helpers in the probe.
2. New mode `primary-loop`: write id 1 through the backend, then poll stamps
   as the daemon worker does; assert no phantom `Changed(0)` and that the
   own-write stamp refresh suppresses re-grab of id 1 exactly once.
3. E2e `S27-primary-selection.toml` (two virtual nodes, needs the §2.4
   virtual slot + step arg): `clipboard_set … selection="primary"` on
   server → `assert_clipboard … selection="primary"` on client, and
   `assert_clipboard` (id 0) still shows its prior text. Reverse direction.
   Then `config_patch` `share_primary_selection=false`, set primary again,
   assert the peer primary does not change. This needs one new optional field
   on the existing `clipboard_set`/`assert_clipboard` steps in
   `tools/e2e/src/steps.rs` — no new step kinds.
4. Interop lab: Deskflow server (X11) ↔ openergy client with flag on:
   select text in the lab display, middle-click on openergy side is not
   drivable headlessly, so assert bytes via the lab clipboard helper
   (`lab_clipboard` command path in `tools/e2e` steps) for PRIMARY.
   Then openergy server ↔ Deskflow client on the same display. Repeat the
   send direction against Barrier 2.4. Record which peers actually move id 1
   in `tools/interop-lab/verification.json`.
5. Negative e2e `S28-primary-not-clobber.toml`: raw peer sends `CCLP`/`DCLP`
   id 1 with empty container to an openergy client; assert client id 0
   clipboard unchanged and connection stays up.

## 3. (b) Keyboard language sync — SER-215, size L, do last

### 3.1 What "sync" can honestly mean

The wire carries a two-letter lowercase language code (`en`, `fr`, `de`,
`ru`, …). It does not carry a layout name, variant, or input-method id.
"Sync" can therefore only mean: **when the server's current language changes,
the client selects the first installed keyboard layout whose primary language
matches, if one exists; otherwise it does nothing.** Anything stronger
(exact variant, dead-key table, IME state) is not representable and must not
be promised in UI or docs.

Per platform, the local object that language maps to:

| Platform | Read current + available | Apply a language | Hard limit |
|---|---|---|---|
| X11 | XKB group index + group names (`XkbGetState`, `XkbGetNames`). No code does this today; `keymap.rs` loads only the core mapping. Group names are layout ids (`us`, `de`, `ru`), so a small alias table is needed (`en`→`us`/`gb`, …). | `XkbLockGroup` to the first group whose name matches. | Only groups already configured in the XKB map can be selected. Variants in parentheses are ignored. |
| macOS | TIS input source id + `kTISPropertyInputSourceLanguages`. FFI has `TISCopyCurrentKeyboardLayoutInputSource`/ID today (`crates/platform-macos/src/ffi.rs`), and layout-change notifications already set `layout_dirty` (`screens.rs`, `tap.rs`). | `TISSelectInputSource` on the first enabled *keyboard-layout* source whose languages contain the code. | Input methods (Japanese, Chinese, Korean IMEs) are out of scope. Secure-input fields may refuse the switch. |
| Windows | `HKL` from `GetKeyboardLayout(foreground thread)` — `native::keyboard_layout()` exists. Language = primary LANGID of low word. Available list = `GetKeyboardLayoutList`. | Best effort only: `ActivateKeyboardLayout` affects our thread; the focused app needs `WM_INPUTLANGCHANGEREQUEST` posted to its window, which it may ignore. Report result as attempted, never as confirmed. | Per-app layout settings in Windows can override us at any focus change. |

Graceful no-op boundaries (log once per connection, keep typing):

- `input.sync_language == false` on sender or receiver.
- Empty, odd-length, non-ASCII, or > 2-byte language units; anything not
  `[a-z][a-z]` after lowercasing.
- No installed layout matches the language.
- Platform reports `NotImplemented` (virtual backend records the request
  instead; see §3.4).
- Wire version < 1.8 for `LSYN`/`DKDL` (see §3.2). Language sync is then
  receive-only via `DKRP` suffix if the peer sends one.

### 3.2 Wire design

No protocol change. Fill in fields that are empty today.

1. Setup: server already sends `LSYN` to wire >= 1.8 right after `QINF`
   (must stay before first `DSOP`; reference clients only accept it in
   setup). Change `languages: String::new()` in `server.rs` to the local
   available-languages string when `input.sync_language` is true, else keep
   `""`. Format: concatenated two-byte codes, deduplicated, order =
   platform order, max 64 bytes (`MAX_LANGUAGE_LEN`).
2. Key down: `wire_input` emits `DKDL` (`lang: Some(current)`) only when
   sync is on, peer minor >= 8, and a current language is known. Otherwise
   `DKDN` (`lang: None`), byte-identical to today. Repeat: fill
   `KeyRepeat.lang` under the same rule for all wire versions (modern peers
   require the suffix field present; today we send `""`).
3. There is no DSOP option id for language sync in peers it is a local
   setting (`client/languageSync`). Do not invent one. `DSOP`/`CINN` carry
   nothing new: CINN has no language field, so the language at enter time
   arrives with the first `DKDL`/`DKRP`. Spec consequence: the first
   keystroke after a screen switch may be typed in the old layout if its
   `DKDL` is also the trigger. Mitigation: client applies the language from
   a `DKDL` *before* injecting that key (rule in §3.3).
4. Hello (`Hello`/`HelloBack`) carries no language and stays unchanged.

Default-value warning: `sync_language` defaults `true` today while inert.
Turning on step 1–2 under that default means existing Deskflow/Synergy
clients that implement language sync may start switching layouts after an
openergy upgrade. That is the reference default (`client/languageSync`
defaults true), so keep `true`, but the release note and
`crates/config/V1-LIMITS.md` update must call it out, and §3.6 scenario 1
must prove an openergy↔openergy pair with no second layout installed sees
zero behavior change.

### 3.3 Session and platform design

New platform capability, defaulted so untouched backends keep compiling:

```rust
// crates/platform/src/lib.rs, on ScreenSystem
fn keyboard_languages(&self) -> KeyboardLanguages {
    KeyboardLanguages::default() // available: [], current: None
}
fn set_keyboard_language(&mut self, _lang: &str) -> Result<(), PlatformError> {
    Err(PlatformError::NotImplemented)
}
pub struct KeyboardLanguages { pub available: Vec<[u8; 2]>, pub current: Option<[u8; 2]> }
```

Data flow:

1. Daemon learns the local languages outside the input path: on backend
   start, and whenever the existing layout-change notifications fire (macOS
   `layout_dirty`, Linux `MappingNotify` handling in capture, Windows
   `WM_INPUTLANGCHANGE` on the hook thread → new command). It forwards
   changes as new session events `ServerEvent::LocalLanguages(KeyboardLanguages)`
   / `ClientEvent::LocalLanguages(..)` (server needs available+current;
   client needs available only to decide no-op early — still send both,
   one type).
2. Server stores it and uses it for §3.2.1/§3.2.2. `wire_input` gains a
   `lang: Option<&str>` parameter; delete the unused `_cfg`/`_minor`
   placeholders once wired.
3. Client: keep `Peer.languages` (setup LSYN). Add
   `Peer.active_language: Option<[u8; 2]>`. On `DKDL`/`DKRP` with a valid new
   language while `sync_language` is on, emit new
   `Action::SetKeyboardLanguage([u8; 2])` **before** the `Action::Inject` for
   that same key, update `active_language`, and still inject the key even if
   the layout switch later fails (daemon reports failure only via log /
   `language.sync` event, §3.4). Same-language repeats emit nothing.
4. `Peer::observe_language` (today: log "ignored") is replaced by the real
   path; keep its log-once behavior for the sync-off case.
5. Injection-order caveat: `set_keyboard_language` runs on the sink/screens
   side thread, and sink key translation reads the just-switched layout
   (macOS keymap refresh already handles `layout_dirty` in `inject.rs`;
   Linux `Sink::down` reloads the keymap per key; Windows uses the
   foreground `HKL` per key). No extra ordering work beyond emitting the two
   actions in order through the existing runtime action queue.

### 3.4 Touch points

| File | Change |
|---|---|
| `crates/config/src/lib.rs` | No field change. Rewrite the `sync_language` doc intent in `V1-LIMITS.md`: implemented with §3.1 no-op boundaries. |
| `crates/platform/src/lib.rs` | `KeyboardLanguages` + two defaulted `ScreenSystem` methods. |
| `crates/platform/src/virtual_backend.rs` | Scriptable state: `set_keyboard_languages(available, current)` via `VirtualControl`; `set_keyboard_language` records the request in the event ledger (new `RecordKind::Language`) and updates current if in `available`, else `NotImplemented`. This is what e2e asserts on. |
| `crates/platform-linux/src/{native,keymap}.rs` | `Screens` implements both methods with XKB (`xkb_get_state`, `xkb_get_names` for group names; `xkb_lock_group` to apply, via x11rb `xkb` protocol already a dependency in `source.rs`). Language↔group alias table kept in `keymap.rs` next to keysym tables: identity for `de/fr/ru/…`, plus `en→[us, gb]`, `ja→[jp]`, `ko→[kr]`, `zh→[cn, tw]`. Current language derived from the locked group name through the reverse alias. |
| `crates/platform-macos/src/{ffi,keymap,screens}.rs` | New FFI: `TISCreateInputSourceList`, `TISSelectInputSource`, `kTISPropertyInputSourceLanguages`, `kTISPropertyInputSourceType`, `kTISTypeKeyboardLayout`. List = enabled keyboard-layout sources only. Cache alongside existing `Layout` id; refresh where `layout_dirty` is already handled. |
| `crates/platform-windows/src/{native,sink}.rs` | `keyboard_languages` from `GetKeyboardLayoutList` + `GetLocaleInfoEx(LOCALE_SISO639LANGNAME)` per LANGID. `set_keyboard_language`: post `WM_INPUTLANGCHANGEREQUEST` with matching `HKL` to `GetForegroundWindow`; return Ok on successful post only (no confirmation claim). |
| `crates/session/src/{lib,common,server,client}.rs` | New events `LocalLanguages`, action `SetKeyboardLanguage`, `Peer.active_language`, §3.2 send/rules, `wire_input` signature. |
| `crates/daemon/src/{runtime,workers,adapter}.rs` | Query `keyboard_languages` at machine start and on layout-change health tick; apply `SetKeyboardLanguage` action against `sink`/`screens` owner thread; `virtual.*` IPC passthrough to script languages for e2e (new method name is *not* added to the frozen IPC `methods` mod — reuse `virtual.inject`-style control through `VirtualControl` as `adapter.rs` already does for clipboard reads). |
| IPC/status | `PeerInfo` gains `languages: Vec<String>` and `active_language: Option<String>`; `Status` gains nothing else. New event `language.sync` (`{"language": "de", "result": "applied"\|"no_match"\|"unsupported"}`) published on change only, plus existing generic `error`/notice path for hard failures. Add event name constants next to `CLIPBOARD_BLOCKED` in `crates/ipc/src/lib.rs`. |

Split into three PRs in this order: (b1) trait + virtual + session wiring,
send and receive, no real platform implementation (ships behind the existing
config, provably no-op on real OSes); (b2) X11 implementation + probe mode;
(b3) macOS + Windows implementations. (b1) alone must not be described as
"language sync works" — it makes the wire honest and testable.

### 3.5 Wire compatibility (honest)

- Synergy/Deskflow 1.8 clients implement the receive side: LSYN list in
  setup, then per-key language selects a compatible installed group when
  their `languageSync` is on. Sending §3.2 output to them is the designed
  use. Deskflow in Barrier-name dialect is still wire 1.8 in the lab, so it
  gets `DKDL`.
- Barrier 2.4 is wire 1.6: no `LSYN`, no `DKDL`. It must keep receiving
  `DKDN`. Its `DKRP` may lack the language suffix entirely; our decoder
  already tolerates that only for pre-1.8 hellos — do not relax it.
- Sending a non-empty `LSYN` to a 1.8 client that did not ask for sync is
  normal protocol behavior (the reference server always sends its real
  list). Sending it after `DSOP` would be a bug; the server send point
  must stay in the hello-back branch.
- Language codes are two bytes, full stop. `pt-BR`, `zh-Hant`, layout
  variants: not representable. Collisions (e.g. `en` = US vs GB) resolve to
  the receiver's first match, which may surprise. This is a peer-protocol
  limit, documented in `V1-LIMITS.md`, not a bug to fix locally.

### 3.6 Verification (end-to-end)

1. E2e `S29-language-sync.toml` (virtual pair, after b1): script server
   languages `available=[en, de], current=de`. Raw-observable evidence:
   setup carries LSYN `ende` (raw node variant `P11-language-wire.toml`
   asserts the bytes), first key arrives as `DKDL` with `de`, client
   `virtual.recorded` shows a Language record before that key's sink record.
   Then script server `current=en`, type again, expect one more Language
   record. Control: `config_patch` `input.sync_language=false` → LSYN empty,
   keys are `DKDN`, no Language records.
2. E2e `S30-language-no-match.toml`: server `current=ru`, client
   `available=[en]`. Keys still inject; status/event evidence shows
   `language.sync` result `no_match` once; typing is unbroken.
3. `x11_probe` mode `language-group` (after b2): in the isolated container,
   `setxkbmap -layout us,de`; backend reports `available=[en, de]`,
   `current` follows `setxkbmap`/`xdotool key ISO_Next_Group`;
   `set_keyboard_language("de")` then sink-injected `z` reaches `xev` as
   `y` (QWERTZ) — the end-to-end proof that the group really switched.
   Restore `us` at the end and assert the original keymap is intact, like
   `inject` mode does.
4. Interop lab (after b2): Deskflow 1.27 X11 client with `us,de` layouts ↔
   openergy server scripted (real X11 through the linux-dev container, not
   virtual) to `de`; type `z`, observe `y` in the lab. Barrier client:
   observe `DKDN`-only stream with a raw capture and unchanged layout.
5. Manual real-device check (b3, not automatable headlessly): macOS and
   Windows clients with two layouts; switch server language; confirm the
   OS input indicator changed (or, on Windows, record the app ignored the
   request — that is an accepted §3.1 outcome, filed as a lab note).

## 4. PR sequence

| PR | Scope | Depends on | Ships flag state |
|---|---|---|---|
| 1. Half-duplex locks (§1) | `crates/session`, `V1-LIMITS.md`, 2 e2e scenarios, `half-duplex-lock` probe mode | — | Per-screen flags, default off (unchanged) |
| 2. PRIMARY plumbing | Trait defaults, X11 impl, virtual slot, daemon worker/runtime per-id, session gates + relay rule, config field, `primary-sync`/`primary-loop` probe modes, S27/S28 | — | `share_primary_selection = false` |
| 3. Language b1 | Trait + virtual + session/daemon wiring, LSYN/DKDL send, S29/S30/P11 | — | `sync_language = true`, real platforms still `NotImplemented` |
| 4. Language b2 | X11 XKB read/apply, `language-group` probe mode, lab checks | PR 3 | Active on X11 |
| 5. Language b3 | macOS TIS + Windows HKL best-effort | PR 3 | Active where OS allows |

None of these change the wire format, the frozen IPC method set, or default
behavior with flags off — except PR 3–5 make `sync_language = true` real, per
§3.2. That is the only default-on behavior change and is called out there.

## 5. Open questions: none

Checked against the repo: macro/flag defaults (`sync_language = true` is
intentional per §3.2), probe harnesses (`tools/linux-dev/` + `x11_probe`
modes), e2e step vocabulary (`tools/e2e/src/steps.rs`), and lab peers
(Deskflow 1.27.0 / Barrier 2.4.0, `tools/interop-lab/README.md`). Remaining
unknowns are peer-behavior facts that are assigned as lab verification steps
above, not design inputs: Deskflow/Barrier id-1 defaults (§2.5) and
edge-for-edge half-duplex semantics (§1.4).
