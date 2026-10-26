# Openergy — Architecture, Round 2 (frozen for implementation)

**Implementation design record, dated 2026-10-21.** “Frozen” refers to the
original implementation contracts, not a stable public API or a release gate
that has passed. Performance and reliability statements below are design
requirements unless linked to scoped verification. The work breakdown records
the original plan; Windows and X11 now have native implementations, with
acceptance limits described in [readiness.md](readiness.md). Use
[setup.md](setup.md) for current operational instructions and explicit paths.

Openergy shares a keyboard, mouse, and clipboard across computers.
This document revises the first design draft (r1). That draft is no longer in
the tree; it is in Git history and at the tag `evidence-archive-2026-10-25`.
Where a section says "unchanged from r1", the current source is the reference.
This revision resolves every
R2-VERIFY item with byte-level evidence and freezes the contracts that
Step 0 implements. Wire facts cite `docs/re/03-core-protocol.md` (03),
`docs/re/02-service.md` (02), `docs/re/01-electron-ui.md` (01),
`docs/re/04-tray-security-packaging.md` (04), `docs/focus-fix.md`, and
`docs/plan/ui-contract.md`. Where this document and r1 disagree, this
document wins. Section numbers mirror r1 so diffs stay reviewable.

## Goal and success criteria (unchanged, restated)

Openergy runs as one small Rust daemon per machine plus a thin native UI.
It targets unmodified Synergy 1.21.1-beta core (wire 1.8) in both
roles, and Openergy-to-Openergy uses the same dialect. Compatibility evidence
is limited to the builds and environments in [readiness.md](readiness.md).
The proposed v1 exit gate:
replace the user's live topology (Mac client of a Windows Synergy server,
and the reverse) with no stuck keys, no silent clipboard loss, correct
window activation on macOS 27, idle RSS under 30 MB per daemon, and under
1 ms added input latency in-process. The old 144 MiB RSS and 523 MiB disk
Synergy baseline is a measurement of the inspected process set (04), not a
current comparative benchmark. The daemon is one binary; macOS also has a
separate native app. These goals do not establish native desktop acceptance.

Design principles (from r1, retained): synchronous threads over async;
`session` does no I/O; framing and limits are enforced before session;
every failure is visible in status and logs; no silent downgrade from TLS
to plaintext; input contents are never logged.

## 1. Language, runtime, dependencies

### 1.1 Decision

Rust 2021, `mio` poll loop, no async runtime, no tokio. TLS is `rustls`
with the `ring` provider in buffered form. Identity is generated with
`rcgen` (ECDSA P-256). This section is unchanged from r1 except for the
dependency pins below and the approved additions.

### 1.2 Why not tokio

Unchanged from r1 §1.2: the workload is four threads and a handful of
sockets; an executor adds latency jitter, binary size, and a second
control flow with no benefit. All blocking happens on `mio::Poll`, OS run
loops, or bounded channels.

### 1.3 Concurrency primitives (closed set)

`std::sync::mpsc`, `Mutex`, atomics, `mio::Waker`. No other
synchronization crates. Channels on the hot path are bounded; see §3.3.

### 1.4 TLS facts (R2-VERIFY resolved)

TLS wraps the whole stream immediately after TCP connect, before the
framed hello; there is no STARTTLS (03 §2, §8). Synergy uses OpenSSL
`TLS_client_method`/`TLS_server_method` with SSLv3/TLS 1.0/TLS 1.1
disabled, so TLS 1.2 and 1.3 are the effective versions, with OpenSSL
default ciphers (03 §8). Chain verification is replaced by a callback
that always succeeds; application authentication is SHA-256(DER)
fingerprint equality (03 §8). Our `rustls` configuration therefore uses a
custom verifier that captures the peer leaf DER, hashes it, and compares
it with the trust store; it must not require a WebPKI chain. Security
levels: 0 plaintext, 1 encrypted without client pin, 2 encrypted with
pin (03 §8). A Synergy server with `checkPeerFingerprints=true` selects
level 2 and requires a client certificate pinned in `trusted-clients`;
a Synergy client always selects level 2 when TLS is on, always presents
its certificate, and always pins the server in `trusted-servers`,
ignoring `checkPeerFingerprints` (03 §8). Decision 6 follows: our client
always presents a certificate and pins the server; our server pins
clients when `tls.require_client_trust` is on (default on). Synergy's
generator makes RSA-2048, CN=`Synergy`, serial 1, 365 days, PEM key+cert
in one file (03 §8); we deliberately differ: ECDSA P-256 via `rcgen`,
separate 0600 key file. Interop does not depend on key type because only
the DER hash is pinned.

### 1.5 Dependency allowlist (workspace-wide, pinned in Step 0)

Only these external crates may appear. Adding one requires an
architecture amendment.

| Crate | Use | Where |
|---|---|---|
| `mio` | Poll loop, waker | `net`, `daemon` |
| `rustls` + `ring` | TLS (buffered API) | `net` |
| `rustls-pemfile` | PEM parsing | `net` |
| `rcgen` | Self-signed identity | `net`/`daemon` |
| `sha2` | Fingerprints, content hashes | `net`, `config` |
| `serde`, `serde_json` | IPC JSON, config JSON view | `ipc`, `config`, `daemon`, `e2e` |
| `toml` | Config file | `config` |
| `mdns-sd` | Discovery (§2.13) | `net` |
| `log` | Logging facade only | all library crates |
| `libc` | Socket options, signals, kqueue | `net`, `daemon`, platform crates |
| `windows-sys` | Win32 FFI | `platform-windows` only |
| `x11rb` | X11 protocol | `platform-linux` only |
| `objc2`, `objc2-foundation`, `objc2-app-kit` (minimal features) | NSPasteboard, NSWorkspace, NSEvent | `platform-macos` only |
| `core-foundation`, `core-graphics` (or raw FFI) | CGEvent, run loop | `platform-macos` only |

`cargo-deny` (`deny.toml`) enforces licenses (MIT/Apache-2.0 family),
bans duplicate majors where avoidable, and denies wildcard versions.
Release profile: `opt-level=3`, `lto="thin"`, `codegen-units=1`,
`panic="abort"`, `strip="symbols"`. Panic-abort is deliberate: a dead
input thread must kill the process so the supervisor restarts clean
(r1 §3.1), never limp with stuck state.

### 1.6 Swift boundary

The Swift app owns no protocol, platform, or config logic. It renders
`status.get`, sends IPC requests, and manages process lifetime (§5.8).
It never writes TOML. All types it needs are the JSON shapes frozen in
the `ipc` crate (§2.10).

## 2. Repository layout and frozen contracts

### 2.1 Tree

```
Cargo.toml                  # workspace + release profile + pinned deps
crates/
  protocol/                 # wire codec (bytes <-> Message)
  platform/                 # traits, InputEvent, ClipboardData, virtual backend
  config/                   # Config model, TOML, Synergy importers/exporter
  ipc/                      # daemon<->app/CLI/e2e JSON protocol types
  session/                  # Server/Client state machines, layout, clipboard mgr
  net/                      # mio + rustls transport, trust, discovery
  platform-macos/           # CGEventTap/CGEventPost/NSPasteboard
  platform-windows/         # Win32 backend; desktop acceptance pending
  platform-linux/           # X11 backend; Xvfb evidence, Wayland unsupported
  daemon/                   # the `openergy` binary (all subcommands)
tools/
  e2e/                      # `openergy-e2e` scenario runner (real binaries only)
  focus-lab/                # owned elsewhere; do not touch
  interop-lab/              # owned elsewhere; do not touch
apps/macos/                 # SwiftUI app
docs/plan/                  # this document and ui-contract.md
```

The daemon binary is `openergy`. Modes are subcommands (§2.11).
The macOS app bundles this daemon with its own SwiftUI executable.

### 2.2 Dependency direction

Unchanged from r1 §2.2. `protocol`, `config`, `platform`, `ipc` depend on
nothing else in the workspace. `session` depends on `protocol`, `config`,
`platform` traits, and `ipc` event types only. `net` depends on
`protocol` and `ipc` types. Only `daemon` wires concrete backends,
reads paths, environment, and the clock. Time enters `session` as a
`u64` monotonic millisecond argument. `unsafe` is allowed only in
`platform-*` crates and for socket options in `net`/`daemon`, each block
with a one-line safety comment. No library crate opens files except
`config`/`net` trust code through paths the daemon supplies.

### 2.3 `protocol` — wire codec (frozen, facts resolved)

Framing: `u32` big-endian length prefix, then one message whose first
four bytes are an ASCII code; the hello is the only variable prefix
(03 §2). Field encodings: `%1i` one byte, `%2i` u16be, `%4i` u32be,
`%s` u32be length + bytes, `%7s` exactly seven bytes (hello name),
`%4I` u32be count followed by that many u32be integers (03 §2). Hard
caps proven from the binary: packet 4 MiB, `%s` 1 MiB, `%4I` 1,048,576
elements, hello-back payload 1024 bytes (03 §2, §3). These replace r1's
32 MiB guess.

Frozen public surface (implemented as types in Step 0; bodies land in
WP1):

```rust
pub const DEFAULT_PORT: u16 = 24800;
pub const MAX_FRAME_LEN: u32 = 4 * 1024 * 1024;
pub const MAX_STRING_LEN: u32 = 1024 * 1024;
pub const MAX_HELLO_BACK_LEN: usize = 1024;
pub const WIRE_MAJOR: u16 = 1;
pub const WIRE_MINOR: u16 = 8;

pub struct ProtocolVersion { pub major: u16, pub minor: u16 }
pub enum Message {
    Hello { product: String, version: ProtocolVersion },
    HelloBack { product: String, version: ProtocolVersion, name: String },
    QueryInfo, Noop, Bye, InfoAck, KeepAlive, ResetOptions,
    Enter { x: i16, y: i16, sequence: u32, modifiers: u16 },
    Leave,
    ScreenSaver { on: bool },
    ClipboardGrab { id: ClipboardId, sequence: u32 },
    SetOptions { options: Vec<(u32, i32)> },
    LanguageSync { languages: String },
    SecureInputNotification { name: String },
    KeyDown { key: u16, modifiers: u16, button: u16, lang: Option<String> },
    KeyRepeat { key: u16, modifiers: u16, count: u16, button: u16, lang: String },
    KeyUp { key: u16, modifiers: u16, button: u16 },
    MouseDown { button: u8 }, MouseUp { button: u8 },
    MouseMoveAbs { x: i16, y: i16 }, MouseMoveRel { dx: i16, dy: i16 },
    MouseWheel { dx: i16, dy: i16 },
    ScreenInfo { x: i16, y: i16, w: i16, h: i16, warp: i16, mx: i16, my: i16 },
    ClipboardData { id: ClipboardId, sequence: u32, chunk: Chunk },
    FileTransfer { mark: u8, payload: Vec<u8> },
    DragInfo { count: i16, payload: String },
    ErrorIncompatible { major: u16, minor: u16 },
    ErrorBusy, ErrorUnknown, ErrorBad,
    Unknown { code: [u8; 4], payload_len: usize },
}
pub enum Chunk { Start { total_len: u32 }, Data(Vec<u8>), End }
pub struct ClipboardId(pub u8);
pub struct Decoder { /* private */ }
impl Decoder {
    pub fn new(max_frame_len: u32) -> Self;
    pub fn feed(&mut self, bytes: &[u8]) -> Result<(), DecodeError>;
    pub fn next_message(&mut self) -> Option<Result<Message, DecodeError>>;
}
pub fn encode(msg: &Message, out: &mut Vec<u8>) -> Result<(), EncodeError>;
```

Resolved encoding rules. `KeyDown` encodes `DKDL` (with `%s` language)
when `lang` is `Some`, else legacy `DKDN`; a 1.8 server normally emits
`DKDL` and the client still accepts `DKDN` (03 §4). `KeyRepeat` is
`DKRP` with fields key, mask, count, button, then a mandatory trailing
language `%s` in this build, even in its inherited 1.1 writer (03 §4,
§13); `lang` is therefore non-optional. `KeyUp` is `DKUP` with no
language. `DMWM` is horizontal delta first, vertical second; 120 units
is one notch (03 §7). Mouse buttons on the wire are 1 left, 2 middle,
3 right, 4/5 additional; 0 is invalid (03 §7). Key IDs are 16-bit
Deskflow logical values (Return `0xEF0D`, F1–F35 `0xEFBE`–`0xEFE0`,
modifiers `0xEFE1` onward; 03 §7, Appendix A); the codec moves them
opaquely. Modifier mask bits: Shift `0x0001`, Control `0x0002`, Alt
`0x0004`, Meta `0x0008`, Super `0x0010` (macOS Command), AltGr `0x0020`,
Caps `0x1000`, Num `0x2000`, Scroll `0x4000` (03 §7). Constants for
these live in `platform` (§2.5) and are re-exported for `session`.

Unknown-code policy (decision 3, replaces r1 close-on-unknown): because
framing is length-first, the decoder yields `Message::Unknown` after
consuming exactly one whole frame; `session` logs once per code per
connection and keeps the connection. Synergy itself drains unknown
codes (03 §2). A malformed *known* message (truncated fields, bad UTF-8,
oversized `%s`, odd DSOP count) is `DecodeError` and the caller closes
that connection only. A completed packet with one to three code bytes
is an incomplete-message disconnect, matching Synergy (03 §2). One
logical message per packet; trailing bytes inside a known fixed-size
message are `BadValue`.

DSOP is a flat `%4I` sequence of ID/value pairs; the element count is
twice the pair count and must be even (03 §6). Frozen option IDs
(three-letter codes are the u32be of their ASCII): heartbeat `HART`
0x48415254 (ms), `SSCM` corner mask (TL 1, TR 2, BL 4, BR 8), `SSCS`
corner size px, `SSWT` switch delay ms, `SSTT` double-tap ms, `SSNS`/
`SSNC`/`SSNA` switch-needs-modifier booleans, `MDLT` relative moves,
`_KFW` win32KeepForeground, `LTSS` default lock state, `DLTS` disable
lock, `CLPS` clipboard sharing, `CLSZ` clipboard size in KiB, `HDCL`/
`HDNL`/`HDSL` half-duplex locks, `MMFS`/`MMFC`/`MMFA`/`MMFG`/`MMFM`/
`MMFR` modifier remaps (target enum 0 None, 1 Shift, 2 Control, 3 Alt,
4 Meta, 5 Super, 6 AltGr), `XTXU` Xinerama-unaware, `SFOC` preserve
focus (03 §6). There is no `screenSaverSync` DSOP ID in this build; do
not invent one (03 §6). Unknown DSOP IDs are stored raw and ignored as
complete pairs, never a close reason. Named constants for every ID live
in `protocol::options`.

### 2.4 `session` — state machines and layout (frozen shapes)

Shapes unchanged from r1 §2.4 (`Server`/`Client`, `ServerEvent`/
`ClientEvent`, `Action`, `handle(&mut self, ev, out)`), with these
frozen additions. `ConnId(u64)` and `ScreenId` (config index) are
session types. `Action` gains `ReplaceConnection { old: ConnId, new:
ConnId }` for decision 4. `ClientEvent` mirrors `ServerEvent` for the
single-server connection plus `ReconnectTimer`. Invariants from r1 stand
(pressed-state ledger, sequence ownership, one active screen with COUT
before CINN in the same `handle` call, heartbeat math in session).

Sequence semantics now proven: the server increments a u32 switch serial
per screen switch, sends it in CINN, and the client reuses it in
client-originated CCLP/DCLP; server-originated DCLP uses sequence 0 in
this build (03 §5). CCLP announces ownership only; it is not a fetch RPC
(03 §5). Session tracks clipboard owner per `ClipboardId` and pushes
DCLP on enter/change to screens that need it. The inspected receiver
keeps single static reassembly state, so we serialize transfers per
connection and never interleave two DCLP transfers on one connection
(03 §5). Input interleaving during a transfer remains required (§3.5).

Layout engine (`session::layout`): rects plus links with percentage
ranges. Interval selection is start-inclusive, end-exclusive; mapping
normalizes the source interval then scales into the destination
interval; overlapping source ranges are a config error (03 §9). Gates:
switch delay starts a wait timer reset by target/direction change;
double-tap requires retreat into the screen (at least max(jump zone,
3 px threshold)) then a second hit inside the window; dead corners use
per-screen settings before global; modifier gates check masks; no edge
switch while a mouse button is held; explicit lock (Scroll Lock,
default-lock, hotkey) blocks switching unless `disableLockToScreen`
(03 §9). Relative forwarding (DMRM) applies only when
`relativeMouseMoves` is on *and* the cursor is locked to a remote
screen (03 §9) — this narrows r1 §5.1. On active-client disconnect the
server returns to the primary screen center and clears switch state
(03 §9). Enter carries the current modifier mask in CINN (03 §9).

Duplicate-name policy (decision 4): on adoption, if the name is already
connected, replace the old connection when it has missed at least one
heartbeat interval *or* the new connection presents the same TLS
fingerprint; run the old connection's leave/release path, close it, then
adopt the new one. Otherwise send `EBSY` and close the new connection.
As a client receiving `EBSY` from a Synergy server, back off and retry;
the server flatlines the stale connection after 3× heartbeat (03 §3).
This directly fixes the observed failure: a TLS reconnect storm followed
by EBSY "already connected" rejections while the stale half-open
connection lingered, with core warnings hidden at INFO level (04). Our
client logs each EBSY at `warn` with the retry delay, and status shows
`attempt`/`next_retry_ms`.

### 2.5 `platform` — backend traits (frozen)

Traits unchanged from r1 §2.5 (`InputSource`, `InputSink`,
`ScreenSystem`, `ClipboardBackend`, `CaptureDecision`,
`PlatformError::PermissionDenied(kind)` as state, `release_all`
idempotent). Frozen types:

```rust
pub enum InputEvent {
    KeyDown { key: u16, modifiers: Modifiers, button: u16 },
    KeyRepeat { key: u16, modifiers: Modifiers, count: i16, button: u16 },
    KeyUp { key: u16, modifiers: Modifiers, button: u16 },
    MouseDown { button: u8 }, MouseUp { button: u8 },
    MouseMoveAbs { x: i32, y: i32 }, MouseMoveRel { dx: i32, dy: i32 },
    MouseWheel { dx: i32, dy: i32 },
}
pub struct Modifiers(pub u16); // constants SHIFT/CONTROL/ALT/META/SUPER/ALTGR/CAPS/NUM/SCROLL
pub struct ScreenRect { pub x: i32, pub y: i32, pub w: i32, pub h: i32, pub scale: f64 }
pub struct Permissions { pub accessibility: Grant, pub input_monitoring: Grant, pub post_events: Grant }
pub enum Grant { Granted, Denied, Unknown, NotRequired }
```

The virtual backend ships in this crate behind `--platform virtual`. It
records every injected event in order with a monotonic sequence, plus
`release_all` markers and, for mouse events, the *click numbers the
macOS injector would assign* is a macOS concern; the virtual backend
instead records the numbering contract abstractly: it tracks per-button
press generations so E2E can assert unique increasing numbers per down
reused on drag/up once the macOS backend exposes its ledger through the
same record shape (decision 1). Concretely, `RecordedEvent` carries
`click_number: Option<i64>` and `click_state: Option<i64>`; the virtual
backend fills them from a pure numbering model that mirrors §5.3 (seed
supplied by the scenario), and the macOS backend fills them from its
real injector state. Backends that cannot number leave them `None`.
Wheel units at this boundary are Deskflow units (120 per notch,
horizontal first); backends convert (03 §7; §5.1 for the macOS fix).

### 2.6 Clipboard model

Wire container, proven: `u32be count`, then per format `u32be id`,
`u32be len`, bytes, no padding; writer emits IDs in ascending order;
reader skips unknown IDs ≥ 3 by declared length and stops the update on
truncation (03 §5). Format IDs: 0 UTF-8 text, 1 UTF-8 HTML, 2 bitmap as
a Windows DIB (BITMAPINFOHEADER + optional masks/palette + pixels,
*without* the 14-byte BMP file header; DIB header fields little-endian
as on Windows) (03 §5). DCLP chunk marks are numeric 1 start, 2 data,
3 end; the start chunk's `%s` payload is the ASCII decimal total size
(e.g. bytes `31 34` for 14), not a binary integer; data chunks are
512 KiB in Synergy (03 §5, §13). We send data payloads of at most
512 KiB per DCLP packet to stay inside both the 1 MiB `%s` cap and
Synergy's own chunking, sliced further by the scheduler (§3.5) without
changing mark semantics: slicing happens at the *queue* level in `net`,
while DCLP packets remain whole. To keep input latency bounded we
therefore emit DCLP data packets of at most 32 KiB payload (a valid
subset of the 512 KiB behavior; receivers accept any chunk sizes that
sum to the start total). The end chunk carries an empty `%s`.

`platform::ClipboardData` is retargeted to the wire: `FormatKind` is
`TextUtf8 | HtmlUtf8 | Dib` for interchange, plus backend-local `Png`
used only inside the macOS backend before conversion. Decision 5: v1
sends DIB (format 2) for Synergy compatibility; PNG is never put on the
wire toward a Synergy peer because this core establishes no PNG wire
format (03 §11: macOS converter order HTML, BMP, UTF-8, UTF-16, plain
text; PNG/TIFF interchange not established). The macOS backend converts
PNG/TIFF pasteboard data to DIB on read and DIB to a pasteboard image
on write. Marshalling/unmarshalling lives in `session::clipboard`.

Size policy: `clipboardSharingSize` is KiB; the Synergy client sender
refuses when the whole marshalled container is >= limit<<10, the server
forwarder refuses when > limit — an asymmetric equality edge we mirror
only as senders (03 §5). Zero disables sharing. Our own
`clipboard.max_bytes` (default 64 MiB) bounds read, reassembly, and
accept; the Synergy assembler has no useful total cap, so ours is a
hard safety requirement (03 §5). Over-peer-cap sends are refused whole
with a `clipboard.blocked` event and a `warn` log carrying both sizes
(fixes silent loss; r1 §2.6 retained). Loop prevention by own-write
change-stamp, per direction enable flags, cancel-on-newer-grab, and
discard-partial-after-10 s all stand from r1. Clipboard IDs: 0 clipboard,
1 selection (X11); others are dropped at session with a debug log.

### 2.7 `net` — transport, TLS, trust, discovery

Surface from r1 §2.7 retained (`NetHandle`, `NetCommand`, `Priority`,
`NetEvent`), with `NetCommand::ReplaceAndClose { old, }` unnecessary —
replacement is `Close{old}` plus normal adoption. `NetEvent::Connected`
carries `fingerprint: Option<Fingerprint>`, `tls_version:
Option<String>`. TLS configuration: rustls `ClientConfig`/
`ServerConfig` with `ring`, TLS 1.2+1.3, ALPN none, a verifier that
accepts any chain but records leaf DER for pinning, client always
presents its identity (decision 6). Handshake runs on the io thread via
rustls buffered IO; a held untrusted connection completes TLS, then
holds before any `Message` flows (§2.9). Plain mode is explicit config,
off by default, `warn` per connection. No plaintext fallback after a
TLS failure — close and surface `last_error` (04 pain point: never hide
this at info level).

### 2.8 `config` — model, derivation, importers

One TOML file (macOS `~/Library/Application Support/Openergy/
config.toml`; Windows `%APPDATA%\Openergy\config.toml`). Linux's intended
XDG config path is not consistently applied by current command defaults:
the daemon falls back to `~/.config/openergy/config.toml`, while offline
commands honor `XDG_CONFIG_HOME`. Use explicit paths as in [setup.md](setup.md).
Frozen model (Step 0 types; serde
field names are the JSON contract):

```rust
pub struct Config {
    pub version: u32, pub role: Role, pub name: String,
    pub server: ServerCfg, pub client: ClientCfg, pub tls: TlsCfg,
    pub clipboard: ClipboardCfg, pub input: InputCfg, pub switch: SwitchCfg,
    pub hotkeys: HotkeysCfg, pub macos: MacosCfg, pub screens: Vec<ScreenCfg>,
}
pub enum Role { Off, Server, Client }
pub struct ScreenCfg {
    pub name: String,
    pub rect: Option<LayoutRect>,          // {x,y,w,h} layout units (screen points)
    pub links: Option<LinksCfg>,           // explicit override; wins over derivation
    pub modifiers: Option<ModifierRemap>, pub os: Option<String>,
    pub aliases: Vec<String>,
}
```

`[tls]` adds `require_client_trust: bool = true`. `[macos]` adds
`click_numbering: "native" | "legacy"` (default native, decision 1).
`[switch]` uses `dead_corners: Vec<Corner>` + `corner_size_px` per
ui-contract (r1's single `dead_corner_px` is superseded). `[server]`
carries both `listen` and `port` (port wins if both set; validate).
Heartbeat: `heartbeat_ms` under `[server]`/`[client]` sharing, default
3000 (decision 5; 03 §3 modern default 3 s, timeout 3× = 9 s).

Link derivation (decision 7) mirrors the Synergy service converter
(02 config mapping): expand each rect by 0.5 units per side (top/left
clamped at 0, size adjusted for the clamp), probe each ordered pair by
extending one unit in the probe direction (up/left move the origin
without growing size; down/right grow size), collision is strict
positive intersection on both axes, and the overlap interval of the
*unprobed expanded* rects yields percentages
`round(100*(start-o)/d)` / `round(100*(end-o)/d)` computed separately
for source and target, emitted as `direction(s0,s1) = target(t0,t1)`
with no extra clamp; later identical keys overwrite (02). Our
`config::derive_links(&[ScreenCfg]) -> DerivedLinks` implements exactly
this over `rect`; explicit `links` override per screen. `DerivedLinks`
is `Vec<DerivedLink { from, edge, from_range: (u8,u8), to, to_range:
(u8,u8) }>` and is exposed in `config.get` JSON as `derived_links`.
At runtime, fractional precision: the layout engine converts
percentages to fractions start-inclusive/end-exclusive (03 §9).

Functions (frozen signatures): `Config::load_toml`, `to_toml`,
`validate`, `to_json` (the JSON view merge-patch operates on),
`from_json`, `apply_merge_patch(cfg, patch) -> Result<(Config,
warnings), ConfigError>` (RFC 7386), `derive_links`,
`import_synergy_conf(text) -> (Config, ImportReport)`,
`import_synergy3_db(json, core_ini) -> (Config, ImportReport)`,
`import_synergy_dir(path) -> (Config, ImportReport)` (discovers
`synergy.conf`, `synergy-core.conf`, `db.json`, and `tls/` trust files
under a Synergy preferences directory), `export_synergy_conf(cfg)`.
Importer mappings from r1 §2.8 stand, updated with: `synergy.conf`
options use the DSOP names in §2.3; service layout JSON positions map
through the same derivation above rather than ad-hoc adjacency (02);
`db.json` is schema v17 (01) — map `computers[]`, `fix.*`,
`performance.{allow_copy,limit,sync}` (limit is KiB, no unit conversion
in the service; 02), `security.encryption` → `tls.enabled`,
`developers.corePort`, `local-network.{keep_alive,interval}` →
heartbeat (interval already ms in modern configs; validate range).
Every unmapped field is named in `ImportReport.warnings`. Strict TOML:
unknown keys are errors with the key path in the message.

### 2.9 Trust store

Fingerprint is SHA-256 over the certificate DER (`X509_digest`, enum 4)
(03 §8). File format: one record per line, current form
`v2:sha256:<64 lowercase hex>`; readers also accept `v2:sha1:...` (never
matches a SHA-256 check), legacy colon-hex SHA-1 (59 chars), plain hex,
and colon hex; empty lines skipped, invalid lines ignored with a debug
count (03 §8). We write only `v2:sha256:` lines. Two files,
`trusted-servers` and `trusted-clients`, under
`<config-dir>/tls/`, importable from Synergy's `tls/` directory (02:
the service writes trusted-servers as a single overwritten line; we keep
a set). `Fingerprint` Display for humans is uppercase colon-hex;
`to_synergy_line()` yields the `v2:sha256:` form. CLI:
`openergy fingerprint` (colon-hex + files), `openergy fingerprint
--synergy` prints exactly `v2:sha256:<64 lowercase hex>` for pasting
into a Synergy peer's trust file. TOFU hold flow from r1 stands (30 s
hold, `peer.untrusted`, approve resumes). Pending holds appear in
`status.pending_trust` with expiry.

### 2.10 `ipc` — local control protocol (frozen to ui-contract)

Transport: Unix socket, default
`~/Library/Application Support/Openergy/openergy.sock` on macOS
(directory 0700, socket 0600; decision 2 — this settles r1's open
question in favor of Application Support), `$XDG_RUNTIME_DIR/
openergy.sock` on Linux, with `~/.local/share/openergy/openergy.sock` as
fallback. Windows uses a per-user named pipe derived from the socket path;
see the Windows daemon amendment below.
`--socket <path>` overrides (E2E always overrides). Framing `u32` BE
length + UTF-8 JSON. Types `Request{id,method,params}`,
`Response{id,result,error}`, `Event{event,data}` as in r1, plus frozen
`Status` and config JSON types that serialize *exactly* to the
ui-contract shape: every field always present, `null` when unknown.
`status.changed` pushes the full `Status` object, coalesced to at most
10/s (decision 8).

Methods (closed set; name constants in `ipc::methods`): `status.get`,
`config.get` → `{config, derived_links}`, `config.patch {patch}` →
validate → atomic TOML write → live apply → `{config, warnings}`
(decision 7), `config.set` (full TOML text; CLI/import),
`config.import_synergy {path, apply}` → `{config, warnings}` (applies
only when `apply=true`), `connection.connect {host?,port?}`,
`connection.disconnect`, `connection.restart`, `trust.list`,
`trust.approve|reject|revoke {fingerprint}`, `discovery.list`,
`permissions.request {kind}`, `permissions.open_settings {kind}`
(optional), `logs.tail {lines}`, `logs.subscribe`,
`logs.unsubscribe`, `events.subscribe`, `daemon.stop`, and the virtual
surface `virtual.inject`, `virtual.recorded`, `virtual.clipboard`,
`virtual.set_screens`, `virtual.set_permissions` (refused unless
`--platform virtual`). Events (constants in `ipc::events`):
`status.changed`, `config.changed`, `discovery.changed`,
`peer.connected`, `peer.disconnected`, `peer.untrusted`,
`clipboard.blocked`, `permissions.changed`, `log.line` (only to
`logs.subscribe` subscribers), `error`. Errors are
`{code, message}` with stable snake_case codes. Unknown method →
`unknown_method` error, connection stays open.

### 2.11 `daemon` binary and CLI surface (frozen)

```
openergy daemon [--config <path>] [--platform macos|virtual|auto]
                [--socket <path>] [--parent-pid <pid>] [--foreground]
openergy server [--config <path>] [--platform ...] [--socket <path>]
openergy client [--config <path>] [--remote host:port] [--socket <path>]
openergy ctl <method> [--socket <path>] [--json '{...}'] [--raw]
openergy status | connect | disconnect | restart
openergy trust list|approve <fp>|reject <fp>|revoke <fp>
openergy fingerprint [--synergy]
openergy import-synergy --from <dir> [--out config.toml] [--apply]
openergy export-synergy-conf [--out synergy.conf]
openergy version
```

Argument parsing is hand-rolled (no clap): unknown flags exit 2 with a
usage line. `--parent-pid` enables the parent watch (§5.8). `server`/
`client` force role without editing the user's config. `ctl` prints
human output by default, exact JSON with `--raw`. All commands exit
non-zero with one clear line on failure.

### 2.12 Logging

r1 §2.12 stands (in-house logger in daemon, `log` facade in crates,
stderr + rotating file 2 MB × 3 + 1000-line ring for `logs.tail`,
`log.line` events for subscribers). Additions from 04: connection,
TLS, EBSY, and clipboard-block events log at `warn` or above — Synergy
hid core warnings below its INFO filter and the user could not see why
reconnects failed; we must not repeat that. Keystrokes and clipboard
bytes are never logged at any level (kinds, sizes, sequences only).

### 2.13 Discovery (mDNS) — resolved, deliberately incompatible

Synergy 3 discovery is not a Bonjour service type: it uses
`discovery-channel` over multicast DNS TXT queries on UDP 5353 with a
channel name derived from the license serial, advertising peer tuples
for a sync port (24802), then syncing peer databases over a plain
unauthenticated-upgrade WebSocket on 24802 (02). We do **not** replicate
any of that: the channel is serial-gated, and the peer-DB sync protocol
(including its unauthenticated `GET /ping` metadata endpoint, 02) is
out of scope and a stated non-goal. Openergy advertises and browses
`_openergy._tcp.local.` (TXT `name`, `ver=1.8`, `fp` first 16 hex for
display only) via `mdns-sd` on the net thread, snapshots throttled to
1/s, surfaced as `discovery.list` + `discovery.changed`. Discovery
never auto-connects and never auto-trusts. Finding a stock Synergy
server is by configured `remote` host (including `.local` names), which
matches the user's real topology.

## 3. Threading and event flow

Sections 3.1–3.5 and 3.7 stand from r1 without change (four threads:
supervisor, platform capture run loop, io poll loop running session,
clipboard worker; callback never blocks; move coalescing with
flush-before-discrete ordering; three priority queues Input < Control <
Bulk with handshake exception; 4 MiB per-connection queue ceiling →
`SlowPeer` close after one heartbeat of Control-only flushing; Bulk
admitted only when higher queues are empty — now as whole ≤32 KiB DCLP
packets per §2.6; shutdown order leave → CBYE ≤250 ms flush →
release-all → close). Heartbeat defaults change: 3000 ms interval,
dead at 3× (decision 5; 03 §3). Client CALV behavior must match the
binary: the client's keepalive alarm is reset by CALV and option/reset
processing, while the server also treats any valid client traffic as
activity (03 §4) — implement exactly that asymmetry. Hello-phase
deadline: Synergy's client uses a 2 s connect/initial-hello timer
(03 §3); we allow 10 s for hello through DINF on loaded LANs but log at
`warn` past 2 s. Reconnect backoff 250 ms → 30 s, jitter ±20%, reset
after 60 s stable, reset once on network-change, plus EBSY backoff
(§2.4). The io loop also owns the parent-pid watch wakeup and the IPC
listeners; no timer thread exists.

## 4. Protocol compatibility strategy (all R2-VERIFY resolved)

### 4.1 Hello and version negotiation

Server speaks first: framed payload `Synergy` (%7s) + major u16be +
minor u16be = 11 bytes (03 §3; hex in 03 §12). Client hello-back:
%7s + major + minor + %s name; total payload ≤1024 bytes, so names are
≤1009 bytes (03 §3). We send product `Synergy`, version 1.8, in both
roles; as a client we echo the server's product spelling when it is
`Barrier` (both spellings remain; the server-side parser does not even
compare the field, but we send the expected dialect anyway) (03 §3,
§13). As a server we accept hello-back minors 0–8 by selecting the
matching dialect level, but only 1.6+ is a supported target (chunked
clipboard arrives at 1.6, SECN at 1.7, LSYN/DKDL at 1.8; 03 §3) —
below 1.6 we connect for input only and say so in status. Major ≠ 1 or
minor > 8 → `EICV` with our version, then close. Unknown screen name or
alias miss → `EUNK`. Bad hello-back → `EBAD` (03 §3).

### 4.2 Session sequence (exact)

Server per connection: send Hello → read HelloBack → create proxy state
→ send `QINF`, then (1.8) `LSYN` → read `DINF` → send `CIAK` → adopt
(EUNK/EBSY checks, §2.4) → send `CROP`, then `DSOP` → input phase
(03 §3). `LSYN` must precede the first `DSOP`: the Synergy client's
normal parser does not recognize LSYN after setup (03 §3). DSOP
completes client setup and enables the screen (03 §3). Client mirrors:
parse Hello (2 s timer), send HelloBack 1.8, setup parser accepts QINF
(→ reply DINF), CIAK (clears ignore-mouse-until-acked), CROP, DSOP (→
connected), CALV, CNOP, LSYN (setup only), CBYE, and errors (03 §3).
Movement before CIAK after a shape update is ignored by Synergy (03
§4); our client implements the same flag. On display change the client
re-sends DINF unsolicited; the server replies CIAK and revalidates
layout. Config change re-sends CROP+DSOP (never LSYN again).

### 4.3 Options

We send, toward every peer: `HART` (our heartbeat), `CLPS`, `CLSZ`
(our cap in KiB, min(ours, peer expectation) handled sender-side),
`MDLT`, corner/delay/tap options from config, half-duplex and modifier
remaps per screen, `_KFW` toward Windows-OS-hinted screens. Received
DSOP updates session state; unknown IDs ignored as pairs (§2.3). CROP
resets half-duplex options, modifier mapping to identity, and the
keepalive alarm defaults before the following DSOP (03 §4).

### 4.4 Extensions

| Msg | Receiver (v1) | Sender (v1) |
|---|---|---|
| `LSYN%s` | Setup phase only: parse concatenated 2-byte codes (even length; odd → ignore + warn once); store peer layouts; language selection happens per-key via DKDL/DKRP `lang` when `sync_language` (03 §7) | Server at 1.8: send local layout 2-byte codes concatenated, after QINF, before DSOP; update per-key `lang` thereafter |
| `SECN%s` | Advisory: set peer `secure_input_app` in status; log info. It names the app blocking the *server's* capture (03 §4, §11) | Server on macOS: when secure input turns on, send the owner app name (or `unknown`) to 1.7+ clients; empty string when it turns off is not sent by Synergy — we send nothing on off and clear on leave |
| `CSEC%1i` | Advisory status only (peer screensaver on/off). The Mac client's common handler is a no-op in this build (03 §9); we still surface it, never gate input on it in v1 | Server broadcasts on local screensaver/lock change where the backend knows; 0/1 only |
| `DFTR`/`DDRG` | Parse within limits, discard, `warn` once per transfer with size, status notice. No working transfer machine exists in this build (03 §4); never desync, never stick a button (the following DMUP still processes) | Never |
| `CNOP` | Accept, count as activity on the server side only per §3 | May send as processing filler never required |

### 4.5 Safety limits (enforced in `protocol`/`net`, before `session`)

Frame ≤4 MiB, `%s` ≤1 MiB, `%4I` ≤1,048,576 elements and even for DSOP,
hello-back ≤1024 B, names/language/SECN strings valid UTF-8 without NUL
(name ≤1009 B by construction; language strings ≤64 B; SECN ≤255 B —
longer known-message strings are malformed → close), clipboard
reassembly ≤`clipboard.max_bytes` with start-total equality at End,
DFTR/DDRG discard ≤1 MiB each, coordinates clamped by session, flood
watchdog >20k msg/s for 5 s → close `Flood`, server sockets hard cap 16,
TLS alert → close with no plaintext fallback. Unknown codes skip the
frame (decision 3). The decoder uses checked slicing only (`get`, no
indexing) — a review rule, verified by the hostile-peer scenario S13.

### 4.6 Non-promises

Licensing, serial-gated discovery, peer-DB sync on 24802, Sentry, the
service REST API, and Electron behaviors are not compatibility targets
(01, 02). Only the core wire protocol, importable config/trust files,
and the `synergy.conf` grammar (for export/interop) are compat surfaces.

## 5. macOS specifics (`platform-macos`)

### 5.1 Capture: `CGEventTap`

r1 §5.1 stands (session tap, head-insert, active, full mask,
tap-disabled re-enable with loop guard, atomic remote-active flag, no
layout in the callback). Resolved details: key identity comes from
Quartz keycode field 9; translation uses TIS input sources and
`kTISPropertyUnicodeKeyLayoutData` (UCKeyTranslate path) to build
per-layout KeyID maps including dead keys, plus the 60-entry static
special-key table (KeyID u32le, virtual keycode u32le) whose entries —
including shared left/right modifier keycodes — must be reproduced
from 03 Appendix B, not "corrected" (03 §7). The wire `button` for a
Mac sender is virtual keycode + 1 (zero reserved); receivers subtract 1
(03 §7). Modifier conversion uses §2.3 mask bits, never raw
CGEventFlags; Command maps to Super by default (03 §7). Wheel: capture
converts Quartz deltas to Deskflow units (120/notch); continuous
(trackpad) deltas accumulate fractionally (see §5.2 injector rule,
applied symmetrically on capture) so sub-notch motion is not lost.

### 5.2 Injection: `CGEventPost`

One explicit persistent `CGEventSource(kCGEventSourceStateHIDSystemState)`
for the sink's lifetime; post at `kCGHIDEventTap` (decision 1). Warp
only on CINN (`CGWarpMouseCursorPosition` + associate true); moves are
synthesized mouse-moved/dragged events with location and — new —
pointer delta fields set. Buttons map wire→CG as 1→0, 2→2, 3→1, 4→3,
5→4; IDs outside 1–5 are ignored, matching Synergy (03 §7). Keys:
reverse-map KeyID under the current (or LSYN-selected) layout group;
remember the source-`button` → emitted keystrokes association so repeat
and up release the right physical keys (03 §7); fall back to
`CGEventKeyboardSetUnicodeString` only when no keycode produces the
character under any installed group. Wheel (decision 9): Synergy's
lossy `int(delta*3/120)` line conversion (03 §7) is replaced by a
fractional accumulator per axis: accumulate `delta * lines_per_notch /
120.0` (default 3 lines/notch), emit the integer part as line units,
keep the remainder; apply `invert_scroll_y` and `scroll_sensitivity`
before accumulation. Pixel-unit mode is deferred. `release_all` posts
ups for the sink's full ledger (keys *and* buttons — Synergy's
fakeAllKeysUp proves keyboard cleanup only, and mid-drag button release
on network loss is an open limitation there, 03 §9; we close that gap
by construction).

### 5.3 Click numbering (decision 1; replaces r1 §5.3 entirely)

Root cause (Deskflow issue #9852; fixes PR #9978 and #9991; `focus-fix.md`):
synthetic mouse events carry `kCGMouseEventNumber = 0`; macOS 27 then
delivers content clicks without activating the app or raising its
window, while title-bar clicks still activate. No window guessing, no
`NSRunningApplication.activate`, no AX raise — that heuristic is the
user's buggy helper and it activates windows behind menu-bar menus.
The injector keeps `last_number: i64` and `pressed_number[button]` for
its whole lifetime. On every mouse-down: `seed` = sum of
`CGEventSourceCounterForEventType(kCGEventSourceStateHIDSystemState, t)`
for t in left/right/other mouse-down; `number = max(last_number, seed)
+ 1` (guard i64 exhaustion: saturate and log once, never wrap);
`last_number = pressed_number[button] = number`; set
`kCGMouseEventNumber` on the down. Every drag and the up of that button
reuse `pressed_number[button]`; up clears it. Moves with no held button
leave the number unset. Re-seed on *every* down so physical clicks
between synthetic ones cannot overtake us. Click state: per button,
compare each down with the previous down using
`NSEvent.doubleClickInterval` and a 4-point Euclidean threshold;
increment `kCGMouseEventClickState` when close and timely, else 1; a
drag beyond the threshold ends the sequence; drags/ups carry their
down's click state. `[macos] click_numbering = "legacy"` skips number
assignment (nil-source behavior) solely to reproduce the bug; default
`native`. The virtual backend's numbering model (§2.5) lets E2E assert:
numbers unique and strictly increasing per down, identical across a
down's drags and up, first down > supplied seed. Real window-server
activation remains a manual acceptance item (§6.6) on a macOS 27 test
machine using `tools/focus-lab/` scenarios S1–S7.

### 5.4 Permissions (TCC)

r1 §5.4 stands (Accessibility, Input Monitoring, post-events;
preflight without prompt; requests only via IPC; 2 s poll while
missing; no restart after grant; revoke → safe leave). Status values
are the ui-contract strings `granted|denied|unknown|not_required`.
Synergy polls a helper binary every 2 s (02); we call preflight APIs
in-process instead. Which grants the session tap needs on macOS 27.2 is
verified in WP5 against the test machine and recorded in the WP report;
the degrade-honestly behavior does not depend on the answer.

### 5.5 Secure input

Detection matches Synergy: `IsSecureEventInputEnabled`, owner via
IORegistry `IOConsoleUsers`/`kCGSSessionSecureInputPID` + `proc_name`,
`unknown` when unresolved, empty when off (03 §11). Status field
`secure_input.local_app`; server role sends SECN (§4.4). We never work
around secure input and never log content.

### 5.6 Cursor visibility; 5.7 sleep/wake/displays/lock

Unchanged from r1 (balance counter + force-show on leave; IOKit power
notifications with immediate sleep ack; 250 ms debounced display
reconfiguration → DINF resend; lock → leave + release-all;
`prevent_sleep` assertion only while remote-active).

### 5.8 Process model (decision 2; replaces r1 §5.8)

`Openergy.app/Contents/MacOS/Openergy` (SwiftUI) spawns
`Openergy.app/Contents/MacOS/openergy` as a child:
`openergy daemon --parent-pid <app pid> --socket <path>`. The child
inherits the app as TCC responsible process, so the user grants
Accessibility and Input Monitoring once to "Openergy". The daemon arms
a kqueue `EVFILT_PROC`/`NOTE_EXIT` watch on the parent (plus a 1 s
`getppid` sanity poll as belt-and-braces); on parent exit it runs the
§3.7 shutdown order (release-all, CBYE) and exits 0. If the daemon
crashes, the app restarts it with backoff 0.5 s doubling to a 30 s cap.
On launch the app first tries the socket: a terminal-started daemon is
valid and is connected to, never duplicated. Single-instance: binding
the socket is the lock; a second daemon that cannot bind reports the
existing daemon's status socket path and exits 1. Login item is the
app itself via `SMAppService.mainApp`. **No LaunchAgent in v1.** Never
copy the binary to `$TMPDIR` (Synergy does; 02/04) — path and identity
stability keep TCC grants valid across updates. Signing: one identity
for both binaries (`$OPENERGY_SIGN_IDENTITY`, else first Apple
Development/Developer ID, else ad-hoc for dev), hardened runtime, no
sandbox (event taps are incompatible), stable bundle ID; distribution
is notarized direct download + Homebrew cask.

## 6. E2E verification harness (no unit tests — hard rule)

No unit tests, no `#[cfg(test)]`, no mocks, anywhere in the workspace.
`cargo build` must not compile test code because none exists. All
verification runs real `openergy` binaries as subprocesses and observes
IPC status, the virtual ledger, logs, files, exit codes, and raw
sockets (r1 §6.1–6.2 retained, including the TOML scenario format and
the rule that the runner never links `session`/`protocol` internals —
it may use `ipc` and `config` types only). One correction: the e2e
crate lives at `tools/e2e` as workspace member `openergy-e2e`.

Safety rails (decision 11, replaces r1 §6.4 tiers): on this Mac a real
`synergy-core` **server must never run** (it installs a live event tap
on the user's desktop), and port **24800 is never bound** by any test
(Synergy is live here). A real `synergy-core` **client** may run here
only for handshake/TLS/DSOP/keepalive scenarios in which our server
never sends CINN, input, or clipboard (it would move the real cursor).
Full input interop runs only in the isolated Linux Docker lab at
`tools/interop-lab/` (recorded Deskflow 1.27/Barrier 2.4 peers on Xvfb, driven
by xdotool; owned by another agent). The runner enforces: refuse port 24800,
refuse a `synergy-core server` node kind on macOS, refuse
`--allow-desktop` (the flag no longer exists).

Scenario catalogue: r1 S1–S17 retained with these deltas — S9 uses an
8 MiB DIB image (not PNG) with hash compare; S13 adds an unknown-code
frame that must *not* disconnect (decision 3) alongside garbage/
oversized/truncated/flood cases that must; S15 keeps RSS < 30 MB and
localhost p99 move latency < 5 ms; S17 becomes the replacement matrix
(stale-half-open replaced after one missed heartbeat; same-fingerprint
replaced immediately; live healthy incumbent → newcomer gets EBSY).
New: **S18** click-numbering contract over the virtual backend (unique
increasing per down, reused on drag/up, click-state 1→2→3 then reset by
distance/time, first down above seed); **S19** `config.patch` round
trip (merge patch → TOML on disk → live role/port change → warnings on
invalid patch with no file change); **S20** parent-pid exit (spawn
daemon with a short-lived parent; parent exits; daemon exits 0 after
release-all/CBYE observed by a connected virtual peer); **S21**
`fingerprint --synergy` output matches `^v2:sha256:[0-9a-f]{64}$` and a
Synergy-format trusted-servers file imports and pins. Interop I1–I5
move to the Docker lab (full input) plus the Mac-safe client-handshake
subset I1/I2/I5 as specified in the rails above; I3's oversize case
asserts our clamp+`clipboard.blocked` with nothing partial received.
Manual acceptance (r1 §6.6 retained): macOS 27 activation via
focus-lab S1–S7 in legacy vs native modes, real-desktop feel, trackpad
scroll speed against the Windows PC, TCC first-run and revoke flows.

## 7. Work breakdown (paste-ready briefs)

Ground rules for every WP (include verbatim in each brief): work only
in your owned paths; the frozen signatures in Step 0 are the contract —
if a signature cannot express the behavior, stop and report, do not
change another crate's public surface; no unit tests, no
`#[cfg(test)]`, verify only through real binaries (`openergy`,
`openergy-e2e`); no `todo!`/`unimplemented!` reachable from the binary —
return typed errors; `cargo build --release` with zero warnings is the
minimum gate; never bind port 24800; never run a `synergy-core` server
on macOS; never log keystrokes or clipboard bytes; every `unsafe` block
gets a one-line safety comment; Rust 1.99 at `~/.cargo/bin` (`export
PATH="$HOME/.cargo/bin:$PATH"`). Critical path is marked ★.

### WP1 ★ — `protocol` codec. Owns `crates/protocol/`.
Frozen: §2.3 types + `protocol::options` constants. Implements framing,
incremental `Decoder`, all message encode/decode incl. hello/hello-back,
DKDL/DKRP language rules, DCLP marks with ASCII start total, DSOP `%4I`,
caps (4 MiB/1 MiB/1024 B), `Unknown` whole-frame skip, checked slicing.
Verify: raw-socket probe via `openergy-e2e` scenario S13-proto (golden
bytes from 03 §12 transcript echoed by a virtual daemon raw mode is
WP4's; until then a `--platform virtual` daemon + runner socket probe).
Accept: transcript bytes in 03 §12 round-trip exactly; oversized prefix
closes without allocating; zero warnings.

### WP2 ★ — `config` model, derivation, importers. Owns `crates/config/`.
Frozen: §2.8 types/functions. Implements strict TOML, JSON view +
RFC 7386 merge patch, rect→links derivation (02 algorithm),
`synergy.conf` + `db.json` v17 + `synergy-core.conf` importers with
warnings, `export_synergy_conf`. Verify: CLI `import-synergy` +
`export-synergy-conf` on fixtures; scenarios S14, S19. Accept: derived
links for partial-overlap fixture match hand-computed percentages;
invalid patch leaves file untouched.

### WP3 — `platform` virtual backend + numbering model. Owns `crates/platform/`.
Frozen: §2.5/§2.6 traits and types. Implements the virtual
source/sink/clipboard/screen system, recorded ledger with
`click_number`/`click_state` from the §5.3 numbering model (seed is
injectable), scripted screens/permissions. Verify: scenario S18 through
`virtual.*` IPC once WP6 wires it; interim via `openergy-e2e` library
mode is not allowed — wait for WP6 or drive through two daemons.
Accept: ledger ordering and numbering assertions in S18.

### WP4 ★ — `net` transport, TLS, trust, discovery. Owns `crates/net/`.
Frozen: §2.7/§2.9. Implements the mio loop glue used by the daemon
(listener, connector, per-conn priority queues, TLS buffered rustls with
pin verifier, client-cert always, held-untrusted state, trust store
files, rcgen identity with 0600 key, `mdns-sd` advertise/browse).
Verify: S6, S7, S21; Mac-safe I2 handshake against a real client is
forbidden direction — use two Openergy daemons + Docker lab for core
interop. Accept: wrong fingerprint never falls back to plain; held
connection flows no messages before approval.

### WP5 — `platform-macos`. Owns `crates/platform-macos/`.
Frozen: `platform` traits. Implements CGEventTap capture (§5.1),
CGEventPost injection with persistent source, click numbering +
click state (§5.3), fractional wheel (§5.2), TIS/UCKeyTranslate key
maps + Appendix-B table (transcribe from 03, cite in code comment),
NSPasteboard clipboard with PNG/TIFF⇄DIB conversion, permissions
preflight, secure-input owner, cursor hide/show accounting, power/
display/lock callbacks, 4-point click threshold. Verify: builds and
runs as daemon backend; focus-lab manual matrix on a macOS 27 test
machine; S16 with virtual stays WP3's. Accept: `click_numbering=legacy`
reproduces focus-lab legacy failures, `native` passes S1–S7 there;
report which TCC grants macOS 27.2 actually requires. Never drive the
dev machine's desktop from E2E.

### WP6 ★ — `session` + daemon wiring + IPC server. Owns `crates/session/`, `crates/daemon/`, `crates/ipc/` (types frozen in Step 0; server logic lives in daemon).
Implements Server/Client machines (§2.4, §4.2), layout gates, clipboard
manager (§2.6), duplicate replacement, heartbeat asymmetry, the four
threads (§3), full IPC method set incl. `config.patch` live apply,
status builder (exact ui-contract shape, 10/s coalesce), CLI subcommands,
logger + ring + `log.line`, parent-pid watch, single-instance bind.
Verify: S1–S13, S15–S17, S19, S20. Accept: kill-server-mid-press leaves
empty ledgers both sides (S4/S5); typing during 8 MiB DIB transfer is
recorded before transfer completion (S9).

### WP7 — `tools/e2e` scenario runner completion. Owns `tools/e2e/`.
Step 0 ships the skeleton (spawn daemon, `status.get`, `daemon.stop`).
Implements the TOML scenario language (r1 §6.2), node kinds
`openergy` and Docker-lab `deskflow` (via `tools/interop-lab/` scripts,
read-only dependency), raw-socket probes, artifact retention on
failure, rails (no 24800, no macOS synergy-server). Verify: the full
S-catalogue green. Accept: a failing scenario prints child stderr tails
and keeps its temp dir path in the report.

### WP8 — Windows/Linux backend stubs to trait-complete. Owns `crates/platform-windows/`, `crates/platform-linux/`.
Compile-clean implementations returning `PlatformError::NotImplemented`
with correct `permissions()` = NotRequired and real screen enumeration
where cheap (X11 via x11rb connect probe). No behavior claims in v1.
Verify: `cargo check` for both cross targets (§8). Accept: daemon with
`--platform virtual` is unaffected; stubs never panic.

### WP9 — Docs and packaging scripts. Owns `docs/` (except `docs/re/`,
`docs/plan/ui-contract.md`), `scripts/`.
`scripts/bundle.sh` (sign both binaries per §5.8), `README` quickstart
incl. the Synergy-peer trust-file how-to (`openergy fingerprint
--synergy`), `docs/interop.md` capturing Docker-lab results. Accept: a
clean checkout builds, bundles ad-hoc, and the app spawns the daemon.

Ordering: Step 0 (done with this document) → WP1 + WP2 in parallel ★ →
WP3 + WP4 in parallel (WP4 ★) → WP6 ★ (needs WP1–WP4 surfaces) → WP7
alongside WP6 → WP5 in parallel from Step 0 (long pole for the manual
matrix, off the critical path) → WP8, WP9 last. The app
integrates against WP6's IPC from Step 0 types immediately.

## 8. Risks, deferrals, residual unknowns

Risks: Windows-side Synergy 1.21 parity is unverified (Mac build only
analyzed; 03 residual unknowns) — Docker-lab Deskflow plus one manual
Windows session close this; DIB variant coverage (compression/bit
depths) needs fixture images from real Windows clipboards; keymap/dead-key
correctness is the highest-risk macOS area and needs layout-matrix
manual checks (en/fr/de at minimum); supplementary-plane Unicode has no
wire escape (16-bit KeyID) — text via clipboard still works, typed
astral characters degrade to the Unicode-string fallback and are logged.
Deferred (explicit non-goals): Wayland, file drag-and-drop transfer,
pixel wheel mode, auto-update, LaunchAgent mode, peer-DB sync/discovery
compatibility, multi-server chaining. Residual unknowns are limited to
runtime facts WP5/WP7 must report (TCC grant set on macOS 27.2, Windows
binary parity, negotiated TLS version in the wild) — none blocks the
frozen contracts. Cross-target `cargo check` for Windows/GNU and
Linux/GNU covers platform-independent crates in Step 0; platform crates
are checked on their native targets in WP8 (GNU cross-link toolchains
are not required for `check`).

## Sources

Deskflow issue #9852, PRs #9978/#9991 (click numbering);
`docs/re/03-core-protocol.md` (wire authority), `02-service.md`
(converter, trust files, discovery), `01-electron-ui.md` (db.json v17),
`04-tray-security-packaging.md` (baseline, log pain points);
`docs/focus-fix.md`; `docs/plan/ui-contract.md`;
the r1 draft in Git history (superseded where noted).

## Windows daemon amendment — 2026-10-21

The daemon may also use the existing `windows-sys` dependency for local IPC,
console signals, and parent process handles. This extends the original
platform-only FFI allowance; no protocol or JSON shape changes.

Windows configuration uses `%APPDATA%\Openergy`. A socket path selects a stable
local byte pipe named `\\.\pipe\openergy-<hash>`, derived from the current user
SID and the absolute path (ASCII case folded, slash normalized). The hash is
a name selector, not an authentication mechanism. Explicit local pipe names
are also accepted. The kernel enforces a protected DACL with exactly the current
user SID. Remote clients are rejected. The first instance uses
`FILE_FLAG_FIRST_PIPE_INSTANCE`; a replacement listener instance is created
before handing the connected one to a client, so the lock has no gap.

The shared per-client dispatcher retains the 64-client cap, JSON framing, and
event bus. Overlapped operations provide blocking threads with cancellation:
50 ms server reads, 1 s writes, and 10 s CLI I/O. Busy CLI opens wait up to
1 s in 250 ms slices. The accept wait checks stop at most every 50 ms.
Cancellation is reaped before buffers and OVERLAPPED storage leave scope.

Windows parent watches retain an `OpenProcess(PROCESS_SYNCHRONIZE)` handle and
poll `WaitForSingleObject(0)`. Console handlers clear inherited Ctrl+C-ignore,
set the shared stop flag, and wait at most 4 s for cleanup on console close.
Unix socket, signal, and parent-watch behavior is unchanged.
