# Openergy — Architecture, Round 1

Status: draft for orchestrator review. This document is not code. It freezes the
interfaces that parallel agents build against. Round 2 must revise this document
after the orchestrator returns reverse-engineering findings. Items marked
**R2-VERIFY** need binary evidence before an agent treats them as fact.

## Goal

Build a small, fast, stable replacement for Synergy 3 (Symless).
The program shares one keyboard, mouse, and clipboard across computers on a LAN.

Hard requirements, in order:

1. Wire compatibility with the unmodified Synergy 3 core (`synergy-core` 1.21.1-beta,
   Deskflow-derived) in both directions. Our client must work against their
   server. Our server must work against their client. The live topology is:
   Windows PC = Synergy 3 server, Mac (Apple M5 Pro, macOS 27.2) = client on its left.
2. Total installed size < 15 MB. Idle RSS < 30 MB. Near-zero idle CPU.
   Added latency per input event on a LAN < 1 ms (process-side, excluding network RTT).
3. No stuck keys or buttons, ever. No crash on malformed network input.
   Automatic reconnect. Correct behaviour across sleep/wake, network change,
   display change, and permission revocation.
4. Fix the known Synergy pains: (a) macOS 27 clicks do not activate inactive
   windows, (b) clipboard data over the peer limit is dropped silently,
   (c) spurious "cursor may not be visible" warnings, (d) 524 MB footprint.
5. Cross-platform long term (macOS, Windows, Linux). macOS ships first.

Non-goals for v1 are listed in section 8. Visual and UI design are owned by the
orchestrator. This document owns architecture, protocol, concurrency,
reliability, and the work breakdown only.

## Success criteria (v1 exit gate)

- Our Mac client controls and is controlled correctly against the real Windows
  Synergy 3 server in daily use. No helper app is needed for window activation.
- Our Mac server accepts the real Synergy 3 client and drives it correctly.
- Two Openergy daemons pass the full E2E suite in section 6 on localhost,
  including the interop scenarios against the real `synergy-core` binary.
- Release app bundle < 15 MB. Daemon idle RSS < 30 MB, measured by the E2E
  resource scenario. Idle CPU < 0.1% over a 60 s window with no input.
- Kill, sleep, network-drop, and bad-packet scenarios leave no stuck input
  and recover without user action.

## Design principles

1. **One daemon, thin shells.** All state and logic live in one Rust daemon
   process. The Swift app and the CLI are stateless IPC clients. The daemon
   runs correctly headless. This deletes the Synergy service + core + tray +
   Electron split and its config-generation layer.
2. **The wire is a compatibility surface, not our internal model.** The codec
   translates Deskflow frames to typed internal events at the edge. No other
   crate sees `%2i` formats, four-character codes, or byte order.
3. **Input path is sacred.** Mouse and keyboard frames never wait behind
   clipboard, file, logging, discovery, or IPC work. Any feature that can
   block the input path is rejected or moved off it.
4. **Release-all is a primitive.** Every exit path from an active screen —
   leave, disconnect, crash of a peer, timeout, permission loss — runs one
   `release_all()` routine. There is no second way to leave a screen.
5. **Fail closed on trust, fail safe on input.** Unknown TLS peers are refused
   until a human approves. Malformed frames close one connection, never the
   process, and never inject partial input.
6. **Deterministic core, OS at the edges.** Protocol, session, and layout are
   pure state machines driven by explicit events and an injected clock. OS
   calls live only in platform backends. This is what makes headless E2E
   (section 6) possible without unit tests.
7. **Frozen contracts first.** The traits and enums in section 2 are written
   and merged before parallel work starts. Agents change a frozen contract
   only through the orchestrator.

---

## 1. Language and runtime decision

### 1.1 Decision

**Rust workspace for daemon, CLI, and all platform backends. Native Swift
(SwiftUI + AppKit where needed) for the macOS menu-bar app. No async runtime.
Blocking OS threads plus one `mio` poll loop for network and IPC I/O.
`rustls` (with the `ring` provider) for TLS.**

The orchestrator's leaning is correct. This section only sharpens it: the
important choice is not Rust, it is *no tokio*.

### 1.2 Why not tokio (or any async runtime)

| Concern | Threads + `mio` (chosen) | tokio (rejected) |
|---|---|---|
| Binary size | `mio` + `rustls` only; fits the 15 MB budget with room | Runtime, executor, and macros add MBs and pull a large tree |
| Idle CPU/RSS | Threads park in `poll`/`CFRunLoop`; zero wakeups when idle | Runtime is also idle-capable, but carries more resident state |
| Latency jitter | One I/O thread, no task scheduling, no work stealing | Task wakeups and multi-thread scheduling add jitter we must then explain away |
| OS input APIs | `CGEventTap` needs its own `CFRunLoop` thread; Win32 hooks need a message loop. Threads match the OS model directly | Async tasks must still bridge to those blocking run loops, so async buys nothing on the capture path |
| Debugging | Real stacks, one thread per concern, deterministic order | Task interleavings hide ordering bugs in a latency-sensitive path |
| Connection count | 1 server listener + at most ~8 clients + a few IPC clients. Poll handles this trivially | Async pays off at thousands of connections. We will never have thousands |

Rule: this is a *small, closed* concurrency problem (one capture source,
one active screen, a handful of sockets). Threads with explicit channels make
the data flow auditable. Async would be complexity without a payoff.

If a future feature needs many concurrent transfers, add a bounded worker
thread (section 3.5), not a runtime.

### 1.3 Concurrency primitives (closed set)

Agents use only these. Do not add others without orchestrator approval.

- `std::thread` — named threads, created once at startup. No thread-per-connection.
- `mio::Poll` + `mio::Waker` — the single I/O reactor. `Waker` is the documented
  cross-thread wake mechanism for `Poll`. Source: https://docs.rs/mio/latest/mio/struct.Waker.html
- `std::sync::mpsc` — bounded-by-convention channels between threads.
  Producers that must never block (the capture callback) use `try_send` and a
  drop/coalesce policy (section 3.3), never `send`.
- `std::sync::{Mutex, Condvar, atomic}` — short-held mutexes for slots
  (latest mouse position, config snapshot). No mutex is held across I/O.
- `Arc` — immutable config snapshots and trust store snapshots only.

Rejected: `crossbeam`, `flume`, `parking_lot`, lock-free queues. They are fine
crates, but `std` covers this problem and every dependency costs size,
build time, and audit surface. Revisit only with a measured hot spot.

### 1.4 TLS: `rustls`, used in its buffered (non-async) form

`rustls` exposes a manual connection API: feed ciphertext in, read plaintext
out, drive it from our own poll loop. That fits the no-runtime design and
avoids OpenSSL (size, FIPS baggage, C build). Source for the manual model:
https://docs.rs/rustls/0.23.20/rustls/manual/index.html

- Provider: `ring` (smaller and simpler than `aws-lc-rs`; no CMake/NASM build).
- We do **not** ship `webpki-roots`. There is no public CA in this system.
  Trust is fingerprint pinning only (section 2.9). This also saves size.
- Certificates: self-signed, generated once per install with `rcgen`,
  stored under the config directory with `0600` key permissions.
- Interop constraint: the peer is OpenSSL 3 (Synergy core). We must accept
  the cipher suites and TLS versions Synergy offers. **R2-VERIFY:** exact TLS
  versions/suites and whether Synergy ever sends a client certificate.
  Default assumption: TLS 1.2+1.3, server presents a self-signed cert,
  client may present none. Our verifier is a custom
  `ServerCertVerifier` that pins the SHA-256 fingerprint (and the symmetric
  client-cert check on the server side). ALPN: none (Synergy sets none,
  **R2-VERIFY**).

### 1.5 Dependency allowlist (workspace-wide)

Everything else needs orchestrator approval in the PR that adds it.
`cargo deny` enforces this list plus license and advisory checks.

| Crate | Used by | Why it earns its place |
|---|---|---|
| `mio` | net, daemon | Poll loop and cross-thread wake |
| `rustls`, `rustls-pki-types` | net | TLS without OpenSSL |
| `rcgen` | net (first-run only path) | Self-signed identity generation |
| `sha2` | net, config | Fingerprints |
| `serde`, `serde_json`, `toml` | config, ipc, e2e | Config, IPC, `db.json` import. (Consider `serde` with `derive` only; no other serde formats.) |
| `mdns-sd` | net | mDNS advertise/browse without linking Bonjour C APIs on every OS |
| `libc` | platform-macos, platform-linux | FFI |
| `core-foundation`, `core-graphics` | platform-macos | Event tap, display, run loop bindings |
| `windows-sys` | platform-windows | Win32 bindings (feature-gated, not in the mac build) |
| `x11rb` | platform-linux | Pure-Rust X11 (feature-gated) |

Explicitly banned: `tokio`, `async-std`, `openssl`, `native-tls`, `reqwest`,
`tracing` (see logging, section 2.12), `clap` with all features (hand-rolled
arg parsing is ~200 lines and keeps the CLI surface exact; if the CLI grows,
revisit), `anyhow`/`thiserror` in library crates (typed errors are part of
the frozen contracts; the daemon binary may use a small local error enum).

Build profile (workspace): `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`,
`panic = "abort"`, `strip = true`. Panic-abort is a reliability choice here:
a panicked input thread must not unwind into a half-pressed keyboard state.
The daemon supervises itself by exiting; the LaunchAgent restarts it, and
`release_all` runs from the platform drop guard before abort where possible.
(See section 5.8 for the limits of that guard.)

### 1.6 Language boundary with Swift

The Swift app never calls Rust directly. No FFI, no UniFFI, no shared library.
The only boundary is the local IPC socket (section 2.10). Reasons: the daemon
must run without the app (headless server, SSH administration, E2E), and an
FFI boundary would couple Swift releases to Rust ABI. The cost is one local
socket hop for control actions only — never for input events.

---

## 2. Repository layout and frozen contracts

### 2.1 Tree

```
Cargo.toml                  # workspace, profile, dependency allowlist versions
crates/
  protocol/                 # Deskflow wire codec only. No I/O, no threads, no OS.
  config/                   # Our TOML model + Synergy importers. No I/O policy.
  platform/                 # Platform traits + shared types + virtual backend.
  platform-macos/           # CGEventTap capture, CGEvent injection, NSPasteboard.
  platform-windows/         # Win32 hooks + SendInput. v1: compiles, feature-gated.
  platform-linux/           # X11 (x11rb) capture/inject. v1: compiles, feature-gated.
  session/                  # Server + client state machines, layout engine, clipboard manager.
  net/                      # mio loop, TCP, rustls, trust store, mDNS discovery.
  ipc/                      # Control-protocol message types (shared by daemon, CLI, E2E).
  daemon/                   # Binary crate: `openergy` (daemon + CLI subcommands).
tools/
  e2e/                      # Scenario runner binary `openergy-e2e` + scenario files.
apps/
  macos/                    # Swift menu-bar app (orchestrator-owned design).
docs/
  plan/                     # This document and successors.
assets/                     # Icon etc. (exists)
```

One binary ships: `openergy`. Modes are subcommands (section 2.11).
This deletes Synergy's binary zoo and its `$TMPDIR` core-copy step entirely.

### 2.2 Dependency direction (enforced by construction)

```
apps/macos (Swift) ──IPC socket──▶ daemon
daemon ──▶ net ──▶ protocol
daemon ──▶ session ──▶ protocol, config, platform (traits only)
daemon ──▶ platform-macos / platform-windows / platform-linux ──▶ platform
daemon ──▶ ipc, config
e2e ──▶ ipc, config   (drives real `openergy` binaries as subprocesses)
```

Rules:

- `protocol`, `config`, `platform`, `ipc` depend on nothing else in the workspace.
- `session` depends on the `platform` *traits*, never on a concrete backend.
  The daemon injects the backend. This is what lets E2E inject `virtual`.
- No crate except `daemon` reads environment variables, paths, or the clock
  directly. Time enters `session` as an argument (`Instant`-like `u64` ms
  from a monotonic source the daemon provides). This makes heartbeat and
  switch-delay behaviour reproducible in E2E.
- `unsafe` is allowed only in `platform-*` crates and in `net` for socket
  options. Every `unsafe` block carries a one-line safety comment.
- No crate logs directly to a file. Crates emit `log` records through the
  facade in section 2.12; the daemon owns sinks.

### 2.3 `protocol` — wire codec (frozen)

Responsibility: bytes ↔ typed messages. Framing: `u32` big-endian length
prefix, then one message whose first 4 bytes are an ASCII code. The hello
exchange is the only variable prefix (`Synergy` / `Barrier` / `Deskflow`
plus version — see section 4.1). The codec is incremental: the caller feeds
arbitrary TCP chunks; complete messages come out. It never allocates more
than the configured maximum frame size and never panics on bad input.

```rust
// crates/protocol/src/lib.rs  (sketch — names and shapes are the contract)

pub const DEFAULT_PORT: u16 = 24800;
pub const MAX_FRAME_LEN: u32 = 32 * 1024 * 1024; // hard cap, see §4.5; clipboard uses chunks far below this

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProtocolVersion { pub major: u16, pub minor: u16 } // wire version, e.g. 1.8

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    Hello     { version: ProtocolVersion },                          // server -> client
    HelloBack { version: ProtocolVersion, name: String },            // client -> server
    QueryInfo,                                                       // QINF
    Noop,                                                            // CNOP
    Bye,                                                             // CBYE
    Enter { x: i16, y: i16, sequence: u32, modifiers: u16 },         // CINN
    Leave,                                                           // COUT
    ClipboardGrab  { id: ClipboardId, sequence: u32 },               // CCLP
    SetOptions     { options: Vec<(u32, i32)> },                     // DSOP (raw pairs; session interprets)
    InfoAck,                                                         // CIAK
    KeepAlive,                                                       // CALV
    ResetOptions,                                                    // CROP
    SecureInputNotification { name: String },                        // SECN (Synergy ext., §4.4)
    LanguageSync { language: String },                               // LSYN (Synergy ext., §4.4)
    KeyDown  { key: u16, modifiers: u16, button: u16, lang: Option<String> }, // DKDN / DKDL
    KeyRepeat{ key: u16, modifiers: u16, count: i16, button: u16, lang: Option<String> }, // DKRP
    KeyUp    { key: u16, modifiers: u16, button: u16, lang: Option<String> }, // DKUP
    MouseDown { button: u8 },                                        // DMDN
    MouseUp   { button: u8 },                                        // DMUP
    MouseMoveAbs { x: i16, y: i16 },                                 // DMMV
    MouseMoveRel { dx: i16, dy: i16 },                               // DMRM
    MouseWheel   { dx: i16, dy: i16 },                               // DMWM
    ScreenInfo   { x: i16, y: i16, w: i16, h: i16, warp: i16, mx: i16, my: i16 }, // DINF
    ClipboardData{ id: ClipboardId, sequence: u32, chunk: Chunk },   // DCLP
    FileTransfer { chunk: Chunk },                                   // DFTR (parsed, v1: session declines)
    DragInfo     { count: i16, payload: String },                    // DDRG (parsed, v1: session declines)
    ErrorIncompatible { major: u16, minor: u16 },                    // EICV
    ErrorBusy,                                                       // EBSY
    ErrorUnknown,                                                    // EUNK
    ErrorBad,                                                        // EBAD
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Chunk { Start { total_len: u32 }, Data(Vec<u8>), End }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClipboardId(pub u8); // 0 = clipboard, 1 = selection (X11); others rejected at session layer

pub struct Decoder { /* private: partial-frame buffer, hello state */ }
impl Decoder {
    pub fn new(max_frame_len: u32) -> Self;
    /// Feed bytes. Returns Ok(n) complete messages now available, or a fatal
    /// framing error (caller must close the connection). Never panics.
    pub fn feed(&mut self, bytes: &[u8]) -> Result<(), DecodeError>;
    pub fn next_message(&mut self) -> Option<Result<Message, DecodeError>>;
}

/// Encode into `out` (reused buffer; hot path must not allocate per mouse move).
pub fn encode(msg: &Message, hello_name: &str, out: &mut Vec<u8>) -> Result<(), EncodeError>;

#[derive(Debug)]
pub enum DecodeError { OversizedFrame(u32), BadHello, BadMessage([u8;4]), Truncated, BadUtf8, BadValue(&'static str) }
```

Contract notes:

- Key codes on the wire are Deskflow `KeyID` values (X11 keysym-derived).
  The codec moves them as opaque `u16`. Translation to OS key codes lives in
  platform backends (section 2.5). `button` is the Deskflow per-layout button
  id; backends that can use it (X11) do, others derive from `key`.
- Mouse button ids are Deskflow's (1=left, 2=middle, 3=right, 4/5 = X1/X2…).
  Backends map them; the codec does not reinterpret.
- Version-gated encodings (e.g. the language-string key variants) are chosen
  by `session` setting `lang`; the codec encodes `DKDL`-form iff `lang` is
  `Some`. **R2-VERIFY:** exact on-wire distinction between `DKDN` and the
  language variant in Synergy 1.21.
- Unknown four-character codes are `DecodeError::BadMessage`. The session
  decides whether that is fatal (it is, section 4.5). Synergy extensions we
  choose to tolerate get explicit variants, as `SECN`/`LSYN` above.

### 2.4 `session` — state machines and layout (frozen shapes)

`session` owns *meaning*: who is active, where the cursor goes next, which
clipboard sequence is current, when to heartbeat. It performs no I/O. The
daemon's I/O thread calls it with events and executes the returned actions.
This is the single most important testability decision in the design: E2E
drives real binaries, but the state machines stay deterministic, so E2E
scenarios are reproducible run after run.

```rust
// crates/session/src/lib.rs (sketch)

pub enum ServerEvent {
    ClientConnected { conn: ConnId },
    ClientMessage { conn: ConnId, msg: Message },
    ClientDisconnected { conn: ConnId, reason: DisconnectReason },
    LocalInput(InputEvent),                    // from platform capture
    LocalClipboardChanged { id: ClipboardId }, // payload fetched later via Action::ReadClipboard
    TimerTick { now_ms: u64 },
    ConfigChanged(Arc<Config>),
    DisplayChanged,                            // local geometry changed
}

pub enum Action {
    Send { conn: ConnId, msg: Message },
    SendClipboardChunk { conn: ConnId, id: ClipboardId, sequence: u32, chunk: Chunk }, // low priority, §3.5
    Close { conn: ConnId, reason: CloseReason },
    Inject(InputEvent),                        // client role: to local platform sink
    SetClipboard { id: ClipboardId, data: ClipboardData },  // assembled, size-checked
    ReadClipboard { id: ClipboardId, max_bytes: usize },    // daemon answers with ClipboardPayload event
    HideLocalCursor(bool),
    ReleaseAllLocal,                           // client role safety primitive
    Notify(IpcEvent),                          // status for UI/CLI (via ipc crate types)
}

pub struct Server { /* screens, links, active conn, pressed-state per client, clipboard seqs */ }
impl Server {
    pub fn new(config: Arc<Config>) -> Self;
    pub fn handle(&mut self, ev: ServerEvent, out: &mut Vec<Action>);
}

pub struct Client { /* server conn, active flag, pressed-state, clipboard reassembly */ }
impl Client {
    pub fn new(config: Arc<Config>, screen_name: String) -> Self;
    pub fn handle(&mut self, ev: ClientEvent, out: &mut Vec<Action>);
}
```

Invariants the session crate must maintain (acceptance criteria, section 7):

1. **Pressed-state ledger.** The server records every key/button it has sent
   down to each client; the client records every key/button it has injected.
   On `Leave`, disconnect, heartbeat timeout, or config change, session emits
   the matching ups (server) or `ReleaseAllLocal` (client). No other code
   synthesises key-up events.
2. **Sequence ownership.** Clipboard `sequence` numbers are per-screen,
   monotonically increasing, owned here. Stale-sequence grabs and data are
   dropped silently (logged at debug).
3. **One active screen.** Exactly one screen (local or one client) receives
   input at a time. Switching emits `COUT` to the old client before `CINN`
   to the new one, in the same `handle` call.
4. **Heartbeat math here, timers in the daemon.** Session tracks last-seen
   times from `TimerTick` + message arrival stamps the daemon supplies.
   The daemon only guarantees a tick at least every 250 ms while connected.

#### Layout / edge-switching engine

Pure geometry, inside `session::layout`. Input: screen rectangles (each
screen reports one bounding rectangle via `DINF`; multi-monitor detail is a
backend concern, section 8), link table, cursor position, switch options.
Output: stay, or switch to screen X at entry point (x, y).

```rust
pub struct Rect { pub x: i32, pub y: i32, pub w: i32, pub h: i32 }
pub struct Link { pub from: ScreenId, pub edge: Edge, pub from_range: (u8, u8), pub to: ScreenId, pub to_range: (u8, u8) }
pub enum Edge { Left, Right, Top, Bottom }

pub struct Layout { /* rects + links + dead corners + options */ }
impl Layout {
    pub fn from_config(cfg: &Config) -> Result<Self, LayoutError>; // validates: no dangling screens, ranges 0..=100
    /// Given cursor at (x,y) on `current` moving by (dx,dy), decide a switch.
    /// Applies dead corners, switch delay, and double-tap state externally timed via `now_ms`.
    pub fn switch_target(&mut self, current: ScreenId, x: i32, y: i32, dx: i32, dy: i32, now_ms: u64)
        -> Option<(ScreenId, i32, i32)>;
}
```

Semantics must match Deskflow's `synergy.conf` grammar: a link
`left(0,100) = other(0,100)` maps the exit fraction along our edge linearly
into the entry fraction along the neighbour's opposite edge. Dead corners
(from `db.json` `dead_corners`) suppress switching within a configurable
corner size. `switchDelay` requires the cursor to rest on the edge;
`switchDoubleTap` requires two edge hits inside the window. Both are timed
state in `Layout`, driven by `now_ms`, never by sleeping.

### 2.5 `platform` — backend traits (frozen; the critical contract)

Three separate traits, because server and client need different halves and
the virtual backend must fake each independently. A backend is one struct
implementing whichever traits its role needs; the daemon constructs it once.

```rust
// crates/platform/src/lib.rs (sketch)

#[derive(Clone, Debug, PartialEq)]
pub enum InputEvent {
    KeyDown { key: u16, modifiers: Modifiers, button: u16 },
    KeyRepeat { key: u16, modifiers: Modifiers, count: i16, button: u16 },
    KeyUp { key: u16, modifiers: Modifiers, button: u16 },
    MouseDown { button: u8 },
    MouseUp { button: u8 },
    MouseMoveAbs { x: i32, y: i32 },
    MouseMoveRel { dx: i32, dy: i32 },
    MouseWheel { dx: i32, dy: i32 },   // wheel units: Deskflow convention, backend converts
}

bitflags-like: pub struct Modifiers(pub u16); // Deskflow modifier mask bits; constants in this crate

/// Server role: observe local input. The callback runs on the OS input thread.
/// It must return in < 1 ms and must never block; the daemon supplies a sink
/// that only try-sends/coalesces (§3.3).
pub trait InputSource: Send {
    fn start(&mut self, sink: Box<dyn FnMut(InputEvent) -> CaptureDecision + Send>) -> Result<(), PlatformError>;
    fn stop(&mut self);
    /// Server hides + pins the local cursor while a remote screen is active.
    fn set_cursor_captured(&mut self, captured: bool) -> Result<(), PlatformError>;
    fn warp_cursor(&mut self, x: i32, y: i32) -> Result<(), PlatformError>;
}
pub enum CaptureDecision { PassToOs, Swallow } // Swallow = remote screen active; local OS must not see it

/// Client role: inject remote input locally.
pub trait InputSink: Send {
    fn inject(&mut self, ev: &InputEvent) -> Result<(), PlatformError>;
    fn release_all(&mut self); // unconditional: every key/button this sink pressed goes up. Idempotent.
    fn set_cursor_visible(&mut self, visible: bool);
}

/// Both roles.
pub trait ScreenSystem: Send {
    fn screens(&self) -> Vec<ScreenRect>;          // bounding rect is screens[0..] union; §8
    fn cursor_pos(&self) -> (i32, i32);
    fn clipboard(&mut self) -> &mut dyn ClipboardBackend;
    fn permissions(&self) -> Permissions;          // §5.4: per-OS grant state, polled by daemon
    fn on_display_changed(&mut self, cb: Box<dyn FnMut() + Send>);
    fn prevent_sleep(&mut self, on: bool);         // kiosk-style assertion while active (config-gated)
}

pub trait ClipboardBackend: Send {
    /// Cheap change counter / sequence owned by the OS (e.g. NSPasteboard.changeCount).
    fn change_stamp(&self) -> u64;
    /// Read current contents as zero or more typed representations. May be slow;
    /// only called from the clipboard worker thread (§3.5). `max_bytes` is a hard cap.
    fn read(&mut self, max_bytes: usize) -> Result<ClipboardData, PlatformError>;
    fn write(&mut self, data: &ClipboardData) -> Result<(), PlatformError>;
}
```

Backends:

| Backend | Capture | Inject | Clipboard | Notes |
|---|---|---|---|---|
| `virtual` (in `platform`) | Scripted queue from E2E control socket | Records every event to an in-memory log + IPC query | In-memory value with stamp | Ships in the release binary behind `--platform virtual`; it is product code, not test-only code (§6.2) |
| `platform-macos` | `CGEventTap` (session, head-insert) | `CGEventPost` + activation fix (§5.3) | `NSPasteboard` | v1 deliverable |
| `platform-windows` | `WH_KEYBOARD_LL`/`WH_MOUSE_LL` + raw input for relative moves | `SendInput` | Win32 clipboard | v1: trait-complete stubs that compile under `cfg(windows)`; behaviour lands post-v1 (§8) |
| `platform-linux` | X11 record/XInput2 via `x11rb` | XTest | X11 selections | v1: stubs as above. Wayland is explicitly out (§8) |

Platform error rule: backends return `PlatformError::PermissionDenied(kind)`
as a *state*, not a fatal error. The daemon degrades (no capture) and
reports through IPC so the app can guide the user. Backends never exit the
process on their own.

### 2.6 Clipboard model (in `platform` types + `session` manager)

```rust
pub struct ClipboardData { pub formats: Vec<ClipboardFormat> }
pub struct ClipboardFormat { pub kind: FormatKind, pub bytes: Vec<u8> }
pub enum FormatKind { TextUtf8, HtmlUtf8, Png, Bmp } // wire format ids assigned in `session`, §4.3
```

Rules:

- The wire carries Deskflow's clipboard blob: a marshalled list of
  (format-id, length, bytes) inside `DCLP` chunks. `session::clipboard`
  owns marshalling/unmarshalling; backends only convert OS ↔ `ClipboardData`.
  **R2-VERIFY:** exact Deskflow format ids and marshalling layout from the
  binary or Deskflow source (`ClipboardChunk`/`IClipboard` marshalling).
- Text is UTF-8 on the wire. Backends convert (macOS `public.utf8-plain-text`,
  X11 `UTF8_STRING`, Win32 `CF_UNICODETEXT`) and normalise line endings to
  `\n` on read, to the OS native form on write. No silent lossy conversion:
  undecodable text is dropped as a format, other formats still transfer.
- Images: prefer PNG end-to-end. If a source OS offers only BMP/DIB
  (Windows), the *sending* backend converts to PNG on the clipboard worker
  thread. Receivers accept PNG and BMP. No image crate dependency in v1:
  macOS gets PNG from `NSPasteboard` (`public.png`) directly or via
  `NSImage`→PNG in the backend; if neither is available the format is
  skipped with a log line. Revisit only with evidence.
- Size policy fixes pain (b). Two independent limits:
  - `clipboard.max_bytes` (ours; default 64 MiB): we never read, buffer, or
    accept more than this for one clipboard grab.
  - the peer's advertised `clipboardSharingSize` (from `DSOP`, KB): when a
    Synergy peer caps at 4000 KB, sending more is wasted work. We clamp to
    the peer cap **but log it and surface an IPC event** ("clipboard not
    sent: 6.2 MB exceeds peer limit 4.0 MB"). Never silent, never partial.
  - Between two Openergy peers the cap is `min(both configs)` and large
    images just work. `performance.allow_copy`/`limit`/`sync` from `db.json`
    map onto these fields at import (§2.8).
- Transfer is chunked and preemptible (section 3.5). A new local clipboard
  change cancels an in-flight outbound transfer of the old sequence.
  A partial inbound transfer that never completes is discarded whole after
  a timeout (10 s) — the local clipboard is never half-written.
- Loop prevention: writes we make to the local clipboard bump the OS change
  stamp. Each backend records the stamp of its own last write; the daemon's
  clipboard watcher ignores exactly that stamp. Sequence numbers alone are
  not trusted for loop prevention.
- Clipboard sharing can be disabled per direction in config
  (`clipboard.enabled`, `clipboard.send`, `clipboard.receive`), superseding
  Deskflow's single `clipboardSharing` option; when talking to a Synergy
  peer we still emit its `DSOP` form as the AND of our flags.

### 2.7 `net` — transport, TLS, trust, discovery (frozen surface)

`net` owns sockets. It knows nothing about screens. It moves `Message`s
between the poll loop and `session` over channels, applying the priority
rules of section 3. Its public surface is a `NetHandle` (send side) plus an
event receiver the daemon drains.

```rust
pub struct NetHandle { /* mpsc::Sender<NetCommand> + mio::Waker */ }
pub enum NetCommand {
    Listen { addr: SocketAddr },
    Connect { host: String, port: u16 },            // client role; reconnect policy lives in daemon (§3.6)
    Send { conn: ConnId, msg: Message, priority: Priority },
    Close { conn: ConnId },
    Shutdown,
}
pub enum Priority { Input, Control, Bulk } // Input < Control < Bulk in queue order; §3.4
pub enum NetEvent {
    Listening { addr: SocketAddr },
    Connected { conn: ConnId, peer: SocketAddr, fingerprint: Option<Fingerprint>, tls: bool },
    Message { conn: ConnId, msg: Message },
    Disconnected { conn: ConnId, reason: DisconnectReason },
    UntrustedPeer { conn: ConnId, fingerprint: Fingerprint, addr: SocketAddr }, // held, not connected, §2.9
    Discovered(Vec<DiscoveredPeer>),                 // mDNS browse snapshot
}
```

### 2.8 `config` — our model, Synergy importers

One file, TOML, human-editable, diff-able. Path (macOS):
`~/Library/Application Support/Openergy/config.toml`; Linux:
`$XDG_CONFIG_HOME/openergy/config.toml`; Windows: `%APPDATA%\Openergy\config.toml`.
The daemon watches nothing; config changes arrive via IPC or SIGHUP/`config reload`
and are applied as a fresh `Arc<Config>` snapshot (session validates before swap).

```toml
# config.toml (illustrative, matches the live Mac-client topology)
version = 1
role = "client"              # "server" | "client"
name = "macbook"           # this screen's name on the wire

[server]
listen = "0.0.0.0:24800"     # server role only

[client]
remote = "windows-pc.local:24800"
autoconnect = true

[tls]
enabled = true
# identity files default to <config-dir>/tls/{cert,key}.pem

[clipboard]
enabled = true
max_bytes = 67108864         # 64 MiB hard cap (§2.6)

[input]
relative_mouse = false       # relativeMouseMoves
invert_scroll_y = false
scroll_sensitivity = 1.0
sync_language = true
prevent_sleep = false

[switch]
delay_ms = 0
double_tap_ms = 0            # 0 = off
dead_corner_px = 0

[[screen]]                   # server role: full layout. Client role: only `name` matters.
name = "windows-pc"
links = { left = "macbook" }

[[screen]]
name = "macbook"
links = { right = "windows-pc" }   # ranged form: right = "windows-pc(0,100)"
```

```rust
pub struct Config { pub version: u32, pub role: Role, pub name: String, pub server: ServerCfg,
                    pub client: ClientCfg, pub tls: TlsCfg, pub clipboard: ClipboardCfg,
                    pub input: InputCfg, pub switch: SwitchCfg, pub screens: Vec<ScreenCfg> }
impl Config {
    pub fn load_toml(text: &str) -> Result<Config, ConfigError>;   // strict: unknown keys are errors
    pub fn to_toml(&self) -> String;
    pub fn validate(&self) -> Result<(), ConfigError>;             // names unique, links resolve, port valid
}
pub fn import_synergy_conf(text: &str) -> Result<Config, ConfigError>;       // Deskflow grammar (§ below)
pub fn import_synergy3_db(json: &str, core_ini: Option<&str>) -> Result<Config, ConfigError>; // db.json v17 + synergy-core.conf
pub fn export_synergy_conf(cfg: &Config) -> String; // for interop E2E against real synergy-core (§6.4)
```

Importer rules (fail loudly, never guess silently):

- `synergy.conf`: parse `section: screens|links|options|aliases`.
  Map options `heartbeat`, `relativeMouseMoves`, `clipboardSharing`,
  `clipboardSharingSize` (KB → bytes), `switchDelay`, `switchDoubleTap`,
  `switchCorners`/`switchCornerSize`. Unknown options produce warnings in
  the import report, not dropped silence. Aliases collapse to canonical names.
- `db.json` (schema v17): map `computers[]` (name, layout position → links
  by adjacency when positions are unambiguous; otherwise emit screens without
  links plus a report line), `fix.*` (num/caps/scroll-lock handling flags,
  `invert_scroll`, `sync_language`, `prevent_sleep`), modifier remaps
  (stored per-screen, applied in platform backends), `mouse.relative`,
  `performance.{allow_copy,limit,sync}` → clipboard fields,
  `security.{encryption,key_length}` → `tls.enabled` (we always use modern
  suites; `key_length` is reported as ignored), `developers.corePort` →
  port, `local-network.{keep_alive,interval}` → heartbeat.
  Fields with no equivalent (licensing, `hdk.*`) are listed in the report.
- `synergy-core.conf` INI merges last when present: `[core] coreMode,
  computerName, port, preventSleep`; `[security] tlsEnabled,
  checkPeerFingerprints`; `[client] remoteHost, languageSync, invertYScroll`.
- Import is a pure function → `Config` + an `ImportReport { warnings: Vec<String> }`.
  The CLI prints the report; the daemon never imports implicitly.
  `openergy import-synergy --from <dir>` writes `config.toml` only on success.

### 2.9 Trust store

Fingerprints are the whole PKI. Format matches Synergy's
`tls/trusted-servers` / trusted-clients files so we can import them:
one SHA-256 fingerprint per line, hex, `:`-separated or plain (accept both).

```rust
pub struct Fingerprint(pub [u8; 32]);  // Display = uppercase colon-hex, Synergy-compatible
pub struct TrustStore { /* path, BTreeSet<Fingerprint> per direction */ }
impl TrustStore {
    pub fn load(dir: &Path) -> Result<Self, TrustError>;
    pub fn is_trusted_server(&self, fp: &Fingerprint) -> bool;
    pub fn is_trusted_client(&self, fp: &Fingerprint) -> bool;
    pub fn trust_server(&mut self, fp: Fingerprint) -> Result<(), TrustError>; // atomic rewrite, fsync
    pub fn revoke(&mut self, fp: &Fingerprint) -> Result<(), TrustError>;
}
```

Flow (fail closed): on TLS handshake completion with an unknown fingerprint,
`net` emits `UntrustedPeer` and *holds* the connection (no `Message` flows,
30 s deadline). The daemon forwards an IPC event; the Swift app (or
`openergy trust approve <fp>`) approves or rejects. Approval persists the
fingerprint and resumes the held connection. This is TOFU with an explicit
human step — identical in spirit to Synergy's fingerprint dialog, minus the
Electron. Plain (non-TLS) mode exists for interop and E2E, is off by
default, and logs a warning on every connection.

### 2.10 `ipc` — local control protocol (frozen)

Transport: Unix domain socket on macOS/Linux
(`<runtime-dir>/openergy.sock`, macOS: `~/Library/Caches/Openergy/openergy.sock`
— **decision needed R2:** Application Support vs Caches; must be on a
user-owned path with `0700` dir), named pipe `\\.\pipe\openergy` on Windows.
Framing: `u32` BE length + one UTF-8 JSON object. JSON is chosen over a
binary format deliberately: control traffic is rare and human debugging
(`socat`, `openergy ctl --raw`) matters more than bytes here.

```rust
// crates/ipc/src/lib.rs — serde types, shared verbatim by daemon, CLI, E2E runner
pub struct Request  { pub id: u64, pub method: String, pub params: serde_json::Value }
pub struct Response { pub id: u64, pub result: Option<serde_json::Value>, pub error: Option<IpcError> }
pub struct Event    { pub event: String, pub data: serde_json::Value } // server -> subscribed clients only
```

Methods (v1, closed set — the Swift app and CLI build against exactly these):

| Method | Params | Result | Notes |
|---|---|---|---|
| `status.get` | — | `{role, state, active_screen, peers[], tls, permissions}` | Also the E2E observation point |
| `config.get` / `config.set` | `set`: full TOML text | parsed `Config` JSON / `{warnings[]}` | `set` validates, writes file atomically, live-applies |
| `config.import_synergy` | `{path}` | `{config_toml, report}` | Does not apply until `config.set` |
| `connection.connect` / `.disconnect` | `{host?, port?}` | — | Client role |
| `connection.restart` | — | — | Drops peers, keeps config |
| `trust.list` | — | `{servers[], clients[]}` | |
| `trust.approve` / `trust.reject` / `trust.revoke` | `{fingerprint}` | — | Resumes/drops held connections |
| `discovery.list` | — | `[{name, host, port, fingerprint?}]` | mDNS snapshot (§2.13) |
| `permissions.request` | `{kind}` | `{granted}` | Triggers OS prompt where the OS allows it (§5.4) |
| `virtual.inject` / `virtual.recorded` / `virtual.clipboard` | scripted events | recorded log | Only when `--platform virtual`; E2E surface (§6.2). Refused otherwise |
| `logs.tail` | `{lines}` | `[String]` | Ring buffer (§2.12) |
| `daemon.stop` | — | — | Clean shutdown: release-all, `CBYE`, exit |

Events: `status.changed`, `peer.connected`, `peer.disconnected`,
`peer.untrusted` (approval needed), `clipboard.blocked` (peer-limit notice,
§2.6), `permissions.changed`, `discovery.changed`, `error`.
Subscription is implicit: any IPC client that sends `events.subscribe`
receives events until it disconnects. Events are lossy by design (bounded
queue per client, drop-oldest) — state is always re-readable via `status.get`.

IPC security: same-user only (socket permissions `0600`; pipe ACL current
user). No token in v1 — the attack surface is "same user account", which
already owns the input devices. Document this; do not add network IPC ever.

### 2.11 `daemon` binary and CLI surface (frozen)

```
openergy daemon [--config <path>] [--platform macos|virtual] [--foreground]
openergy server  [--config <path>]        # = daemon with role forced to server (interop/E2E)
openergy client  [--config <path>] [--remote host:port]
openergy ctl <method> [--json '{...}']    # thin IPC client; human output by default
openergy status | connect | disconnect | restart     # ctl aliases
openergy trust list|approve <fp>|reject <fp>|revoke <fp>
openergy fingerprint                      # print our identity fingerprint (for the other machine)
openergy import-synergy --from <Synergy prefs dir> [--out config.toml]
openergy export-synergy-conf [--out synergy.conf]    # for driving a real synergy-core (§6.4)
openergy version
```

`server`/`client` exist so E2E and interop runs never depend on the user's
real config or role. All commands exit non-zero with one clear line on
failure (no backtraces unless `RUST_BACKTRACE` is set; with panic-abort there
is no unwind output by default).

### 2.12 Logging

Small in-house logger in `daemon` (no `tracing`): levels
`error|warn|info|debug`, format `time level target: message`, sinks:
stderr (foreground), rotating file (`<log-dir>/openergy.log`, 2 MB × 3),
and a 1000-line in-memory ring served by `logs.tail`. Library crates use
the `log` facade macros only. Rules: no log call on the per-mouse-move hot
path above `debug`, and `debug` move logging is rate-limited in the daemon.
Input contents (keystrokes, clipboard bytes) are **never** logged at any
level — only event kinds, sizes, and sequence numbers. This is a hard review
rule: it keeps passwords out of log files by construction.

### 2.13 Discovery (mDNS)

Advertise (server role) and browse (client role) service type
`_openergy._tcp.local.` with TXT keys `name`, `ver` (wire version),
`fp` (first 16 hex of fingerprint, for display only — never trusted).
Also browse Synergy's service type if round 2 identifies one, so our client
can find an unmodified Synergy 3 server by name (**R2-VERIFY:** service
type/TXT used by `synergy-service`; Synergy 3 may use its own discovery
rather than mDNS, in which case we browse nothing and rely on `remote` +
`.local` names — acceptable for v1). Discovery never auto-connects and
never auto-trusts; it only fills the `discovery.list` for UI/CLI pickers.
mDNS runs on the net thread's poll loop via `mdns-sd`'s channel, throttled
to one browse snapshot per second.

---

## 3. Threading and event flow (minimum latency)

### 3.1 The four threads (plus OS-owned callback context)

| Thread | Owns | Blocks on | Priority |
|---|---|---|---|
| `main` / supervisor | Startup, config load, signal handling, thread join, LaunchAgent lifecycle | channel + signals | normal |
| `platform` (capture) | The OS input run loop (`CFRunLoop` + event tap on macOS; message loop on Windows; X11 fd on Linux; virtual queue wait) | OS run loop | elevated where the OS allows an unprivileged bump; never real-time |
| `io` (poll loop) | `mio::Poll`: listener, peer sockets, IPC socket, timers, mDNS channel. Runs `session::handle` and executes its `Action`s. Owns all sockets and the `Decoder`s | `poll` with computed timeout | normal (work is tiny; elevation is unnecessary and unmeasurable on LAN) |
| `clipboard` (worker) | Clipboard read/encode/convert, write to OS clipboard, chunk slicing | bounded channel | low |

Client role adds no thread: injection happens on the `io` thread by calling
`InputSink::inject` inline (macOS `CGEventPost` is thread-safe and fast).
**Exception:** if a future backend's inject can block (it must not, by trait
contract), it gets its own queue — never a lock shared with capture.

Thread creation is once-at-startup. If `io` or `platform` dies (panic-abort
makes this process-fatal by design, section 1.5), the supervisor does not
attempt in-process resurrection of input state; the process exits and the
supervisor (LaunchAgent/systemd/scm) restarts it clean. Clipboard worker
failure is non-fatal: the supervisor restarts that thread and logs.

### 3.2 End-to-end path, server role (input leaving this machine)

```
OS input → platform thread callback
  → translate to InputEvent (backend key/button mapping)
  → try_send / coalesce into capture channel  + mio::Waker::wake()
io thread wakes → drain capture channel → session::Server::handle(LocalInput)
  → layout decides stay/switch → Action::Send(Message) list
  → encode into per-connection reused buffer → priority queue (Input)
  → rustls write + socket write (TCP_NODELAY always on)
```

Budget: callback translation < 50 µs, no allocation for mouse move/key
events (events are `Copy`-sized enums by value through the channel);
`session::handle` for a move is a few integer compares plus one encode.
Added latency target < 1 ms holds with two orders of magnitude to spare;
the risk is never CPU, it is accidental blocking — hence the rules below.

### 3.3 Capture-side rules (the callback must never block)

- The capture channel is bounded (capacity 1024 events). The callback uses
  `try_send` only. On full: mouse-move events are *coalesced* (see below),
  then dropped oldest-move-first. Key/button events are never dropped to
  make room for moves; if the channel is full of discrete events (a
  pathological >1000 events inside one poll cycle), the callback returns
  `PassToOs` for that event and increments a counter surfaced in
  `status.get` — visible, never silent, and in practice unreachable.
- **Mouse-move coalescing.** A dedicated latest-value slot
  (`Mutex<Option<(i32,i32)>>` for absolute, atomic accumulators for relative)
  sits beside the channel. The callback writes the slot and wakes the poll
  loop *without* enqueueing. The `io` thread, at the top of each loop
  iteration, takes the slot if the channel has no discrete event pending
  ahead of it in capture order. Ordering rule that makes this safe: a
  discrete event (click, key) first flushes the pending move slot through
  `session` before itself, so a click can never land at a stale position.
  Relative moves accumulate (sum) — they must never be latest-wins.
- The callback does no logging, no allocation, no layout math, no socket
  work. It translates and deposits. Layout/switch decisions happen on `io`.

### 3.4 Outbound path and backpressure

Each connection has three FIFO queues in the `io` thread:
`Input` (moves, keys, buttons, wheel), `Control` (hello, enter/leave,
options, keepalive, grabs), `Bulk` (clipboard/file chunks).
The writer drains strictly in that priority order each time the socket is
writable, with one exception: `Control` messages that are part of the
handshake precede everything until `CONNECTED`. Within `Input`, a queued
absolute move is *replaced* by a newer absolute move if no discrete event
sits between them (the queue holds at most one unsent absolute position per
quiet run). This bounds move backlog to O(1) under a stalled peer.

Backpressure policy under a slow or dead peer:

- Socket send buffer: leave OS default; set `TCP_NODELAY`; do not set
  `SO_SNDBUF` small — LAN peers either keep up or are dead.
- If total queued bytes per connection exceed 4 MiB, the connection is
  unhealthy: stop enqueueing moves, keep trying to flush `Control` for one
  heartbeat interval, then close with reason `SlowPeer`. A peer that cannot
  drain 4 MiB of input on a LAN is hung; keeping it only grows latency for
  the human when it recovers.
- `Bulk` never counts toward that 4 MiB alone — it is admitted only in
  32 KiB slices when `Input` and `Control` are empty (section 3.5).

### 3.5 Clipboard without stalling input

- Detection: the `io` thread polls `ClipboardBackend::change_stamp()` at
  most every 200 ms while connected (cheap counter read; on macOS
  `NSPasteboard.changeCount`), or the backend pushes via callback where the
  OS supports it (X11 selection ownership). No busy-wait.
- On change, `session` emits `ReadClipboard`; the daemon hands it to the
  clipboard worker. The worker reads, converts, marshals (section 2.6), and
  returns one immutable `Vec<u8>` blob + format summary to `io`.
  `io` never touches the OS clipboard. The only `io`-thread cost is
  queueing `CCLP` (Control) and slicing chunks.
- Slicing: `io` serves `Bulk` in ≤ 32 KiB `DCLP` Data chunks, at most one
  slice per poll iteration, only when higher queues are empty. A 10 MiB
  image therefore interleaves with input at frame granularity; worst added
  input delay is one 32 KiB socket write (≈ negligible on LAN, bounded on
  Wi-Fi). TLS record overhead at this slice size is acceptable; do not
  shrink below 16 KiB (rustls record efficiency) without measuring.
- Inbound: chunks reassemble in `session` into a bounded buffer
  (≤ `clipboard.max_bytes`); only the completed blob is unmarshalled and
  handed to the worker for the OS write. A giant inbound transfer can occupy
  memory up to the cap but never blocks decoding of subsequent input frames,
  because the decoder yields messages one at a time and reassembly is a
  `Vec` append. Interleaved input during an inbound clipboard transfer is
  normal protocol behaviour and must keep working (E2E scenario S9).

### 3.6 Reconnect, heartbeat, and timers

- The `io` loop computes its `poll` timeout from the nearest deadline:
  heartbeat send, heartbeat expiry, reconnect attempt, clipboard poll,
  switch-delay expiry, untrusted-peer hold expiry. No timer thread.
- Heartbeat matches Deskflow semantics: `CALV` both ways on the configured
  interval (`heartbeat`, default 5000 ms; from `db.json`
  `local-network.interval` at import). Silence for 3× interval = dead peer:
  session runs the leave path (pressed-state ups / `ReleaseAllLocal`)
  *before* the socket is closed, then the daemon schedules reconnect.
  `CNOP` is accepted and ignored (some peers use it as filler).
- Client reconnect: exponential backoff 250 ms → 30 s cap, jitter ±20%,
  reset after a connection survives 60 s. Network-change signals (platform
  backend callback where available, else connect failure pattern) reset the
  backoff to 250 ms once — fast recovery after Wi-Fi roam without a
  reconnect storm.
- Server: a second connection from a screen name that is already connected
  gets `EBSY` and is closed. The existing connection is not disturbed.
  Address reuse and listener recovery after network change: rebind on
  interface-change notification or bind failure retry at backoff cadence.

### 3.7 Shutdown order (no stuck input on exit)

`daemon.stop` / SIGTERM: (1) session leave path for the active screen
(ups + `COUT`), (2) `CBYE` to all peers, best-effort flush ≤ 250 ms,
(3) `release_all` on the local sink (client role) and cursor restore,
(4) sockets close, threads join, exit 0. SIGKILL cannot be handled — the
peer side covers it: any connection loss without `COUT` still triggers the
receiver's `ReleaseAllLocal` (client) or pressed-state discard (server).
Both sides are safe against the other side vanishing.

---

## 4. Protocol compatibility strategy

Deskflow's published protocol reference is the baseline authority, checked
against the Synergy 1.21 binary in round 2. Reference:
https://deskflow.github.io/deskflow/protocol_reference.html

### 4.1 Hello and version negotiation

- The server speaks first: length-prefixed `Synergy%2i%2i` (major, minor).
  The client answers `Synergy%2i%2i%4i%s`-style hello-back with its version
  and screen name. **R2-VERIFY:** exact hello/hello-back byte layouts and
  the version Synergy core 1.21.1-beta advertises (expected 1.8; the codec
  must be written against captured bytes, not memory).
- We advertise the highest version round 2 proves Synergy 1.21 accepts, and
  we *accept* peers from 1.6 through that version. Version gates feature
  use (see 4.4), never basic input. A peer below 1.6 or above our maximum
  major gets `EICV` and a clean close — not a hang.
- Product name in the hello: the classic protocol overloads this field
  (`Synergy`, `Barrier`, `Deskflow` variants exist in the wild). We send
  exactly the string round 2 proves Synergy 1.21 accepts (expected
  `Synergy`) when `compat.synergy = true` (default). Openergy-to-Openergy
  still works with that string; we do not invent an `Openergy` hello in v1,
  because a vanity string must never cost interop. If round 2 shows the
  core ignores the string beyond length, revisit.
- Screen-name matching is exact and case-sensitive, as Deskflow does it.
  Unknown names get `EUNK`. Aliases from imported config are resolved in
  `config`, not on the wire.

### 4.2 Session sequence (both roles must implement exactly)

Server view per client: `Hello → HelloBack → QINF → DINF → CIAK → DSOP →
CINN/COUT + input…`, with `CALV` throughout and `CROP`+`DSOP` re-sent on
config change. Client mirrors it. `DINF` fields: x, y, w, h, warp size,
current mouse x/y — backends supply real values (virtual backend supplies
scripted ones). On display change, a client re-sends `DINF` unsolicited
(Deskflow behaviour); the server re-runs layout validation and, if the
active cursor is now outside all rects, warps to the nearest valid point.
**R2-VERIFY:** whether Synergy 1.21 sends `DSOP` before or after first
`CINN`, and the full `DSOP` option-id table it emits (clipboard sharing,
size, relative moves, language sync, keep-foreground…). Unknown option ids
in `DSOP` are stored raw and ignored semantically — never a close reason.

### 4.3 Options and feature negotiation

There is no capability handshake in this protocol beyond version + `DSOP`.
So negotiation = conservative sending:

- Send only messages the peer version is known to accept. Keep a per-version
  table in `session` (data, not code branches scattered around).
- `DSOP` options we set toward a Synergy peer mirror its own set so its
  behaviour matches what its service would have configured:
  clipboard sharing on/off, clipboard size KB, relative mouse moves,
  `win32KeepForeground` toward Windows peers (harmless elsewhere),
  language-sync enable. **R2-VERIFY:** the numeric ids.
- Toward an Openergy peer (detected how? — see open question O3 in
  section 8; v1 answer: do not detect; treat every peer as Synergy-class)
  we still speak the same conservative dialect. Openergy-only extensions
  are deferred until they can be added without a detection hack.

### 4.4 Synergy-specific extensions

| Msg | Meaning (to verify) | v1 behaviour as receiver | v1 behaviour as sender |
|---|---|---|---|
| `LSYN%s` | Keyboard language sync — peer asks us to match its input language | Parse; if `sync_language`, ask backend to switch input source where the OS permits (macOS: `TISSelectInputSource` by language; best-effort, log on failure). Never blocks input | Send when our server-side language changes and peer version is Synergy-class with language sync on. **R2-VERIFY:** string format (expected BCP-47-ish or Deskflow language code) and the `DKDL` per-key language field relationship |
| `SECN%s` | Secure-input notification — peer reports an app enabled secure input (name string) | Parse; surface as IPC `status` detail ("secure input active on <peer>: <name>"). No input change | macOS backend reports `IsSecureEventInputEnabled` transitions; send toward Synergy peers. **R2-VERIFY:** trigger semantics — is it advisory only? |
| `DFTR` | File transfer chunks (drag-and-drop file payload) | Parse chunks within limits; session **declines**: discard payload, log size, IPC notice. Must not desync the stream | Never send in v1 |
| `DDRG` | Drag info (file list being dragged across) | Parse; decline as above; ensure the following mouse-up still processes cleanly so a declined drag cannot stick a button | Never send in v1 |
| `CSEC%1i` | Screen saver / secure screen state (from recon list) | Parse; treat as advisory status (peer screen locked/asleep). Do not switch input to a screen reporting locked — **R2-VERIFY** polarity | Send on lock/unlock where backend knows (macOS distributed notifications). Advisory only |

General extension rule: **parse everything Synergy 1.21 can emit, even what
we decline.** An unparsed-but-valid message desyncs a length-prefixed stream
only if we mis-handle framing; since framing is length-first, the safe floor
for a truly unknown code is "skip this frame's bytes, log once per code per
connection, keep going" — but v1 chooses *close on unknown* instead
(section 4.5), because a peer emitting codes outside the verified 1.21 set
is not the peer we promised to interoperate with. Round 2 may soften this
if captures show Synergy emitting codes outside the recon list.

### 4.5 Safety limits (all enforced in `protocol`/`net`, before `session`)

| Limit | Value | On violation |
|---|---|---|
| Frame length prefix | ≤ 32 MiB hard cap (`MAX_FRAME_LEN`); per-message-type expected maxima are far smaller and checked after decode (e.g. `DMMV` is a fixed 8 bytes) | Close connection, reason `BadFrame`. No allocation of the claimed size before the check |
| Hello-phase deadline | Complete hello + `QINF`/`DINF` within 10 s of connect | Close |
| Clipboard reassembly | ≤ `clipboard.max_bytes` (default 64 MiB); chunk `Start` total must match bytes received by `End` (±0) | Discard transfer, close only if framing also desyncs |
| File/drag payload | ≤ 1 MiB parsed-and-discarded in v1 | Discard + close (a file push we declined should not stream forever) |
| String fields (names, language, `SECN` name) | ≤ 255 bytes, valid UTF-8, no NUL | `BadValue` → close |
| Coordinate values | `i16` wire range clamped into known rects by session | Clamp, never trust for allocation or indexing |
| Messages per second per conn | Soft watchdog: > 20k msg/s sustained 5 s → close `Flood` (a broken peer must not pin the `io` thread) | Close |
| Concurrent clients (server) | Configured screens only; hard cap 16 sockets | Extra sockets get `EBSY`/close |
| TLS handshake | rustls defaults; alert = close; no plaintext fallback once TLS is configured | Close. The daemon never silently downgrades a TLS-configured connection to plain |

Fuzz posture without unit tests: the E2E suite includes a hostile-peer
scenario (S13) that sends garbage, oversized prefixes, and truncated frames
over a real socket and asserts the daemon stays up and other connections
survive. The decoder is written in safe Rust with checked slicing; the
review rule is "no indexing, only `get`" in `protocol`.

### 4.6 What "compatible" does not promise

Behavioural quirks of the Synergy *service* (licensing, its own discovery,
its config generation, Sentry) are not compatibility targets. We replace
that layer. Only the core wire protocol, its config *inputs* (as import
sources), and fingerprint trust files are compat surfaces.

---

## 5. macOS specifics (`platform-macos`)

macOS is the v1 shipping backend and the user's daily machine. Everything
in this section lives in `crates/platform-macos`; the Swift app implements
none of it. (The app only renders permission state the daemon reports.)

### 5.1 Capture (server role): `CGEventTap`

- Tap location `cgSessionEventTap`, placement `headInsertEventTap`,
  options `default` (active — we must be able to swallow events while a
  remote screen is active; a listen-only tap cannot). Event mask: key down/up,
  flags-changed, left/right/other mouse down/up/dragged, mouse moved,
  scroll wheel. The tap runs on the `platform` thread's `CFRunLoop` via a
  `CFMachPort` run-loop source.
- The tap callback translates `CGEvent` → `InputEvent`, deposits it per
  section 3.3, and returns the event or null per the current
  `CaptureDecision`. The decision flag (remote-active?) is an atomic the
  `io` thread flips on screen switch; the callback never waits for `io`.
  Consequence, accepted: the switch decision itself is made on `io` from
  deposited moves, so there is at most one poll cycle where the cursor
  visually sits at the local edge before capture engages. Deskflow has the
  same shape. Human perception: sub-frame. Do not "optimise" this by doing
  layout in the callback.
- Tap-disabled handling is mandatory: macOS disables taps that are slow or
  on user-input timeout (`kCGEventTapDisabledByTimeout/UserInput`). The
  callback re-enables the tap immediately and logs; a repeated disable loop
  (>5 in 10 s) reports `permissions.changed` (usually means a grant was
  revoked) instead of spinning.
- Key mapping: `CGEvent` keycode + current layout (`UCKeyTranslate` via
  `TISCopyCurrentKeyboardLayoutInputSource` Unicode data) → Deskflow
  `KeyID`; keep the Deskflow-compatible `button` id derivation Deskflow's
  own macOS code uses. **R2-VERIFY:** against Deskflow source for the
  keymap tables — this is the highest-risk correctness area on macOS
  (dead keys, AltGr-style combos, `fix.*` lock-key semantics). Modifier
  remaps from imported config apply here, in one place.
- Scroll: continuous (trackpad) vs line (wheel) events both map to
  `DMWM` wheel units with `scroll_sensitivity` and `invert_scroll_y`
  applied. Document the unit conversion; Deskflow's wheel unit is
  120ths-of-notch style. **R2-VERIFY** exact scaling Synergy expects,
  especially for high-resolution trackpad deltas, or the Windows PC will
  scroll at the wrong speed.
- Relative mode (games): when `relative_mouse` is on and a remote screen is
  active, capture associates the mouse to the hidden cursor
  (`CGAssociateMouseAndMouseCursorPosition(false)`), reads deltas, emits
  `MouseMoveRel`. Off by default, matching `mouse.relative` import.

### 5.2 Injection (client role): `CGEventPost`

- Create events with a dedicated `CGEventSource`
  (`kCGEventSourceStateHIDSystemState` so injected modifier state composes
  like hardware), set location/flags/button explicitly from the wire event,
  post at `cghidEventTap`. Reuse the source; do not create one per event.
- Mouse warp on `CINN`: `CGWarpMouseCursorPosition` to the entry point,
  then `CGAssociateMouseAndMouseCursorPosition(true)`. Absolute moves use
  synthesized `kCGEventMouseMoved` with location set (not repeated warps —
  warps suppress the local cursor's acceleration/association handling and
  can lose drag state). During a drag, event subtype must be
  `kCGEventLeftMouseDragged` etc. per pressed buttons the sink tracks.
- Keys: `CGEventCreateKeyboardEvent` with the macOS keycode reverse-mapped
  from `KeyID` under the *current* layout, flags set from the wire modifier
  mask. For characters with no keycode under the current layout, fall back
  to `CGEventKeyboardSetUnicodeString` (this is how Deskflow handles
  cross-layout text; **R2-VERIFY** its exact fallback order, plus how
  `LSYN`/language-sync changes it).
- `release_all`: the sink keeps its own pressed ledger (keycodes + buttons
  it pressed). `release_all` posts the ups in reverse order, clears flags,
  restores cursor visibility. Called on leave/disconnect/shutdown and from
  the backend's `Drop` guard. It must be safe to call when nothing is pressed.

### 5.3 Fixing the window-activation bug (pain (a)) — in the injector, not a helper

Observed bug: on macOS 27, clicks injected by the Synergy client reach
controls but do not activate the inactive app unless the title bar is
clicked. Working hypothesis for round 2 to confirm: macOS 27 tightened
activation policy so synthesized HID events from a background agent no
longer trigger the implicit "activate app under pointer" that physical
clicks get. The user's helper app proves the cure: explicitly activate the
app under the pointer on mouse-down.

Design (inside `platform-macos` injection, client role):

1. On injected *mouse-down* only (not move, not up), before posting the
   event: find the topmost window at the target point
   (`CGWindowListCopyWindowInfo` with `optionOnScreenOnly` +
   `excludeDesktopElements`, first window whose bounds contain the point,
   layer 0). Get its owner PID.
2. If that PID is not the frontmost app (`NSWorkspace.frontmostApplication`
   via a lightweight AppKit/Foundation FFI call — the daemon links
   Foundation anyway for pasteboard), activate it:
   `NSRunningApplication.activate(options:)` with
   `[.activateIgnoringOtherApps]` (the deprecated-but-functional flag;
   round 2 checks the macOS 27-preferred API — `activateWithOptions`
   semantics changed across recent releases). **R2-VERIFY:** from a
   LaunchAgent (non-app) process, whether activation requires the
   *calling* process to hold Accessibility, and whether an
   `AXUIElement` `kAXRaiseAction` on the window is also needed for
   correct window ordering within the app.
3. Post the mouse-down immediately after (same `io` thread, no sleep).
   Activation is asynchronous in the window server; physical clicks have
   the same race and macOS orders them correctly because both go through
   the same event stream. If round 2 shows a residual first-click-eaten
   case for specific apps, the fallback knob is
   `input.activation = "ax-raise"` (AX raise + activate), config-visible,
   never a hidden heuristic zoo.
4. Guards: skip activation when the point is on the Dock, menu bar, or our
   own cursor-only overlay (window owner = loginwindow/WindowServer or
   layer ≠ 0 handles most of this); skip during secure input; never
   activate on mouse-*move* (that would steal focus while passing over
   windows — the helper app's rule is mouse-down, keep exactly that).
5. This is a backend behaviour, invisible to `session` and to the wire.
   The virtual backend records `activate(pid)` calls in its log so E2E can
   assert the ordering contract (activate precedes the down event) headlessly
   — the real window-server effect is a manual acceptance check (section 6.6),
   because E2E must not drive the user's real desktop.

### 5.4 Permissions (TCC) — states, prompts, UX flow

Two grants matter: **Accessibility** (post events, AX raise) and **Input
Monitoring** (listen tap). An event *tap* that modifies events needs
Accessibility; listen-only needs Input Monitoring; we need both surfaces,
so we treat both as required for full function and degrade honestly:

| Grant | Needed for | Check (no prompt) | Request |
|---|---|---|---|
| Accessibility | injection, active tap swallow, AX raise | `AXIsProcessTrusted()` | `AXIsProcessTrustedWithOptions(prompt=true)` — but only via IPC `permissions.request` from the app, never spontaneously at daemon start |
| Input Monitoring | capture tap creation on modern macOS | `CGPreflightListenEventAccess()` | `CGRequestListenEventAccess()` (same IPC-only rule) |
| Post-event access (where distinct) | `CGEventPost` | `CGPreflightPostEventAccess()` | `CGRequestPostEventAccess()` |

**R2-VERIFY** on macOS 27.2: which of these the session event tap actually
requires, and whether the checks above still exist in that form.

Flow: daemon computes `Permissions { accessibility, input_monitoring,
post_events }` at start and on a 2 s poll while any is missing (cheap
preflight calls), exposes it in `status.get`, emits `permissions.changed`.
The Swift app (orchestrator design) shows onboarding with deep links
(`x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility`,
`…?Privacy_ListenEvent`). Revocation mid-session: tap creation/posting
starts failing → backend returns `PermissionDenied`, session leaves remote
screens safely (release-all both ways), daemon keeps running and reports.
Restarting the daemon after a grant is **not** required — the backend
retries tap creation when the poll sees the grant appear. (Synergy's
restart dance is a bug we do not copy.)

### 5.5 Secure input

When any app enables secure event input (`IsSecureEventInputEnabled`,
polled on state change + per `CINN`), macOS blocks event taps from seeing
key content. Behaviour: client role — injection of keys may be swallowed by
the OS for password fields; we cannot and must not work around that. We
surface "secure input active" in status, send `SECN` to a Synergy server
(section 4.4), and on the *server* side, if secure input turns on locally
while a remote screen is active, we keep forwarding (the tap still gets
events in the server direction — secure input affects *other* apps reading
events; **R2-VERIFY** the exact tap visibility) but never log content
(we never do anyway, section 2.12).

### 5.6 Cursor visibility (pain (c))

Server side, remote active: hide local cursor
(`CGDisplayHideCursor` for the main display + associate-disconnect in
relative mode) and pin it at the edge point. On return: warp to the
re-entry point, `CGDisplayShowCursor`. Client side on `CINN`: ensure
cursor visible (a previous crashed client may have left it hidden — track a
hide/show balance counter in the backend; `release_all` forces show).
No user-facing warnings are generated for normal transitions. Synergy's
"cursor may not be visible" warning appears to fire on its own show/hide
bookkeeping mismatch; our counter-based approach plus force-show on every
leave removes the class of bug instead of warning about it.

### 5.7 Sleep/wake, displays, lock

- Power: register IOKit power notifications on the platform thread.
  Will-sleep → session leave path + `CBYE`-best-effort (do not delay sleep;
  ack immediately after queueing). Wake → re-query displays, rebuild the
  tap if the run loop dropped it, reset reconnect backoff, re-send `DINF`
  if geometry changed while asleep.
- Displays: `CGDisplayRegisterReconfigurationCallback` → debounce 250 ms →
  daemon `DisplayChanged` event → client re-sends `DINF`; server rebuilds
  `Layout` rects and re-clamps the cursor. Hot-plug during an active drag:
  finish the drag (button ledger wins), then apply geometry.
- Lock/unlock and fast user switching: distributed notifications
  (`com.apple.screenIsLocked/Unlocked`) → on lock, leave remote screens and
  release-all (input must never flow to a screen the user can't see being
  controlled, and the login window session has different TCC). On unlock,
  resume normal operation; reconnection is automatic.
- `prevent_sleep` (config, default off; from `fix.prevent_sleep`):
  `IOPMAssertionCreateWithName(kIOPMAssertPreventUserIdleDisplaySleep)`
  held only while a remote screen is active. Released on leave. Never a
  permanent assertion.

### 5.8 Daemon lifecycle, signing, TCC identity

- The daemon binary ships **inside** `Openergy.app/Contents/MacOS/openergy`.
  The LaunchAgent (`~/Library/LaunchAgents/dev.openergy.daemon.plist`,
  `RunAtLoad`, `KeepAlive` on crash-only — `SuccessfulExit=false`) runs that
  path directly. **Never copy the binary to `$TMPDIR`** (Synergy's approach):
  TCC grants attach to the responsible code identity and path stability
  matters; a temp copy invites re-prompts and orphaned grants.
- The Swift app registers/unregisters the agent (`SMAppService` modern API;
  **R2-VERIFY** macOS 27 preferred form) and can run without the agent for
  first-run setup. `openergy daemon --foreground` is the same binary for
  debugging and E2E.
- Code signing: Developer ID Application identity, hardened runtime,
  **no app sandbox** (event taps and global injection are incompatible with
  the sandbox — this also means no Mac App Store; distribution is
  notarized direct download + Homebrew cask). Stable bundle ID
  `dev.openergy.app` and a designated requirement anchored to the team ID,
  so TCC grants survive updates. Entitlements: the minimum set; document
  each in the repo. The Rust binary inside the app is signed as part of the
  bundle with the same identity; the *responsible process* for TCC is then
  consistent whether the tap runs from the agent or a terminal-launched
  daemon. **R2-VERIFY:** whether grants given to the app cover the embedded
  daemon binary under macOS 27 TCC responsibility rules, or whether the
  daemon needs its own embedded provisioning — this decides whether the
  agent plist points into the bundle or at a helper with its own bundle ID
  (`dev.openergy.daemon`). Decide before WP5 starts; changing identity
  later re-prompts every user.
- Updates: out of scope for v1 beyond "download new notarized build".
  No auto-updater in v1 (section 8).

---

## 6. E2E verification harness (no unit tests)

Project rule is strict: **no unit tests.** Verification is end-to-end only,
through real binaries, real sockets, and the real CLI/IPC. Existing test
doubles that mock away the system are banned. The virtual platform backend
is not a mock: it is a real backend compiled into the shipped binary,
selected explicitly, with a scripted input source and a recording injector,
so the full path — config → daemon → session → codec → TCP/TLS → peer
daemon → injection ledger — runs for real on localhost.

### 6.1 Shape

```
tools/e2e/  (binary: openergy-e2e)
  scenarios/*.toml     # declarative scenarios, one file per scenario
  src/main.rs          # runner: spawn processes, drive IPC + sockets, assert, report
```

The runner is itself Rust in the workspace (uses only `ipc` + `config`
types). It never links `session` or `protocol` internals for assertions —
it observes processes from outside: IPC `status.get`,
`virtual.recorded`, exit codes, log files, and raw-socket probes it opens
itself. That keeps the harness honest: if the harness could call internals,
it would stop proving the real interfaces work.

Runner duties per scenario: create an isolated temp dir (config, TLS
identity, trust store, logs, sockets), pick free high ports, spawn child
processes with explicit env, wait for readiness by polling IPC/TCP (bounded,
with the child's stderr captured for failure reports), run steps, assert,
tear down the whole process tree (SIGTERM, wait, SIGKILL sweep), keep
artifacts on failure only. Scenarios run sequentially by default; port and
temp-dir isolation makes them parallel-safe later.

### 6.2 Virtual backend scripting

`openergy daemon --platform virtual --config <scenario config>` enables
the IPC `virtual.*` methods (refused in any other mode):

- `virtual.inject {events: [...]}` — feed scripted local input as if the OS
  produced it (server role capture source). Events use the same JSON shape
  as `platform::InputEvent` serde encoding.
- `virtual.recorded {since: n}` — every event the injector received
  (client role), plus `release_all` and `activate(pid)` markers, in order,
  each stamped with a monotonic sequence. This ledger is the primary
  assertion surface: "the peer pressed exactly these keys in this order,
  and released everything."
- `virtual.clipboard {get|set}` — read/write the in-memory clipboard with
  formats, so clipboard scenarios assert bytes and format kinds end-to-end.
- `virtual.set_screens {rects}` / `virtual.set_permissions {…}` — script
  geometry and grant state, including mid-scenario changes (display-change
  and revocation scenarios).

Scenario file format (TOML; steps are data so agents can add scenarios
without touching runner code):

```toml
name = "S4-stuck-keys-on-disconnect"
[[node]]           # an openergy daemon
id = "server"; role = "server"; platform = "virtual"
[[node]]
id = "client"; role = "client"; platform = "virtual"; remote = "server"
[[step]]; do = "wait_connected"; node = "client"; timeout_ms = 5000
[[step]]; do = "inject"; node = "server"
events = [ {move_abs = [1919, 540]}, {key_down = "a"} ]   # crosses right edge into client, presses A
[[step]]; do = "assert_recorded"; node = "client"
expect = [ {key_down = "a"} ]                            # ordering + content, not timing
[[step]]; do = "kill"; node = "server"; signal = "KILL"  # no clean leave possible
[[step]]; do = "assert_recorded"; node = "client"
expect_tail = [ {key_up = "a"}, {release_all = true} ]    # peer death released the key
[[step]]; do = "assert_status"; node = "client"; state = "reconnecting"
```

Assertions are about order, content, connection state, exit codes, and
file contents (trust store, exported config) — never about wall-clock
timing beyond generous timeouts. Latency is verified by one dedicated
scenario (S15) with percentile budgets, not by timing every step.

### 6.3 Scenario catalogue (v1 — each maps to a requirement)

| ID | Scenario | Asserts (headline) |
|---|---|---|
| S1 | Two virtual daemons, plain, connect + hello + `QINF/DINF` | `status` connected both sides; correct screen names; clean `daemon.stop` |
| S2 | Edge switch right→left layout (Mac-left-of-Windows mirror) | Cursor crosses at the mapped entry point (range mapping (0,100)→(0,100) and a sub-range case); `COUT` before `CINN` in recorded order |
| S3 | Typing + modifiers + wheel across the edge | Exact recorded sequence; modifiers compose (shift+a arrives as one key event with mask); no extra ups |
| S4 | Kill server mid-key-press (SIGKILL) | Client records key-up + `release_all`; client enters reconnecting; no stuck state after server restarts and reconnects |
| S5 | Server killed while mouse button held (drag) | Button up recorded on client; next click works normally |
| S6 | TLS on, unknown fingerprint | Connection held (no input flows); `peer.untrusted` event; CLI `trust approve` resumes it; trust file contains the fingerprint; restart reconnects without approval |
| S7 | TLS on, wrong fingerprint pre-seeded | Connection refused; daemon alive; clear log/error; no plaintext fallback bytes on the socket (runner probes raw) |
| S8 | Clipboard text both directions, incl. Unicode + CRLF normalisation | Exact bytes at the peer virtual clipboard; loop prevention: no bounce-back grab (sequence stable) |
| S9 | Clipboard 8 MiB PNG while typing | All key events recorded *interleaved before* clipboard completion; image bytes intact (hash compare); input not blocked (typing completes first) |
| S10 | Clipboard over *our* cap; over a low peer-advertised cap | Over-cap data not sent; `clipboard.blocked` IPC event with sizes; small clip still works after |
| S11 | Heartbeat death: firewall-drop the client socket (runner proxy pauses bytes) | Server declares dead in ~3× interval, runs leave path, accepts the client's fresh reconnect; pressed ledger empty |
| S12 | Display change mid-session (`virtual.set_screens`) | Client re-sends `DINF`; server layout respects new rect (switch point moves); no disconnect |
| S13 | Hostile peer: garbage bytes, 4 GiB length prefix, truncated hello, unknown code, flood | Daemon stays up; offending connection closed; a second well-behaved client still connects and types; no partial injection recorded |
| S14 | Config import round-trip (CLI) | Fixture `synergy.conf` + `db.json` v17 + `synergy-core.conf` → `import-synergy` → expected `config.toml` values (links, port, clipboard limit, heartbeat); warnings reported for unmapped fields; daemon boots on the imported config and passes S1-shaped connect |
| S15 | Resource + latency budget | Idle 60 s: RSS < 30 MB per daemon, CPU < 0.1%; 1000 scripted moves: p99 capture→recorded latency on localhost < 5 ms (loose bound catching order-of-magnitude regressions; the < 1 ms process budget is reviewed, not CI-gated at that precision) |
| S16 | Permission revocation (virtual) | Mid-session `set_permissions denied` → safe leave both ledgers, status shows missing grant, daemon alive; grant restored → capture resumes without restart |
| S17 | Duplicate screen name connects | Second connection gets busy/closed; first session undisturbed |

### 6.4 Interop scenarios against the real `synergy-core`

The runner supports node kind `synergy-core`: it copies the real binary
from `/Applications/Synergy.app` (path via env `SYNERGY_CORE_BIN`, never
hard-coded) into the scenario temp dir, writes a generated `synergy.conf`
(our `openergy export-synergy-conf`) and `synergy-core.conf`, and launches
it exactly as the service does (`synergy-core server|client --settings …`)
on a scenario port. Safety rails, all enforced by the runner:

- **Never port 24800, never the user's real config dir.** Scenario env sets
  an isolated `HOME`-equivalent for the child where the binary honours it,
  and explicit `--settings`/config paths inside the temp dir everywhere
  else. The runner refuses to start if the target port is 24800 or if a
  listener already owns the scenario port.
- The live Synergy session on this Mac is a *client* of the Windows PC;
  interop children must not connect to it. Their `remoteHost` is always
  `127.0.0.1:<scenario port>`.
- A real `synergy-core` client injects into the *real* desktop. That
  violates the project rule against driving the user's desktop for tests.
  Therefore interop scenarios are split:
  - **Headless-safe (default suite):** `synergy-core` runs as **server**;
    our virtual client connects to it. We assert on our recorded ledger.
    But a real server needs real input to forward… so the stimulus is the
    runner speaking *client-side observably only*: connect, handshake,
    `DINF`, clipboard grabs the runner triggers via our client's virtual
    clipboard, plus raw-socket protocol probes (handshake bytes, `DSOP`
    capture). No desktop is touched because our client is virtual and the
    real server gets no physical input to forward.
  - **Desktop-touching (opt-in, manual or dedicated CI Mac, never the
    default suite):** our virtual server drives a real `synergy-core`
    client, asserting via the core's log output at debug level. This tier
    runs only with `--allow-desktop` on a machine/account dedicated to it.
    Document it as such; do not sneak it into the default run.
- **R2-VERIFY:** `synergy-core` headless flags (does `server` mode need a
  display/event tap on macOS even with no input? does `--no-tray`/`-f`
  foreground logging suffice for assertions?). If the real server refuses
  to run without a GUI session, the headless-safe tier runs inside the
  user's GUI session but with no input synthesis — still never moving the
  user's real cursor, because our side is virtual.

Interop assertions (I-series):

| ID | Scenario | Asserts |
|---|---|---|
| I1 | Our virtual client ↔ real `synergy-core` server, plain | Handshake versions agree; we reach connected; server log accepts our screen name; heartbeats flow 30 s with no error |
| I2 | Same, TLS on, fingerprint imported from the core's generated cert | Connects only after trust; wrong fingerprint refused by *us*; core accepts our client hello over TLS (**R2-VERIFY:** does the core require a client cert?) |
| I3 | Clipboard Mac→core direction via virtual clipboard grab | Core log/state shows receipt at correct size; oversize-vs-its-`clipboardSharingSize` case: we clamp + log, core sees nothing partial |
| I4 | Our server (virtual capture) ↔ real `synergy-core` client `--allow-desktop` tier only | Scripted typing/clicks appear in core debug log in order; kill-our-server case leaves core with no stuck keys (log + clean subsequent input) |
| I5 | `export-synergy-conf` output is accepted verbatim by the real core | Core starts with zero config errors on our exported grammar, including ranged links |

### 6.5 What E2E deliberately does not do

No unit tests. No mocking of sockets, TLS, files, or platform traits inside
a process. No testing through a real GUI desktop in the default suite (no
computer-use, no driving the user's mouse/keyboard). No screenshot
assertions. If a behaviour cannot be observed through IPC, the virtual
ledger, logs, files, exit codes, or a raw socket, the fix is to add an
observable surface (a status field, a log line with structured content, a
virtual-backend marker) — not to reach inside the process.

### 6.6 Manual acceptance (cannot be automated honestly)

A short checklist the orchestrator runs on the real two-machine topology
before v1: activation fix across Finder/Safari/VS Code/Terminal (click
inactive window body, app activates and click lands), secure-input fields
(1Password/sudo prompt behaviour sane), trackpad scroll feel on Windows,
multi-monitor Mac edge cases, sleep both machines overnight and wake.
Each item names the expected behaviour from section 5. Failures become
new E2E scenarios where a virtual observable exists, or stay on this list.

---

## 7. Work breakdown (parallel packages)

### 7.0 Ground rules for all agents

- **Frozen contracts:** `protocol::Message/Decoder/encode`,
  `platform::{InputSource, InputSink, ScreenSystem, ClipboardBackend,
  InputEvent, ClipboardData}`, `session::{Server, Client}::handle` +
  `Action`, `config::Config` + importers, `ipc` message types + method
  table, `net::{NetCommand, NetEvent, Priority}`. A package that needs a
  contract change stops and escalates to the orchestrator. It does not fork
  the contract locally.
- **Step 0 (orchestrator, before fan-out):** land the workspace skeleton
  with the frozen traits/enums as compiling stubs (`todo!()` bodies are
  banned in stubs that E2E launches; return typed `NotImplemented` errors),
  the dependency allowlist, `cargo-deny` config, and the E2E runner able
  to run a trivial S1-shaped scenario red. Parallel work starts from green
  compile, red scenarios. Estimated: one focused session. Everything else
  parallelises badly without this.
- Each package owns its files exclusively (below). Two packages never edit
  the same crate concurrently. Cross-package needs travel through the
  frozen contracts or the orchestrator.
- Each package ships E2E scenario **files** for its acceptance criteria
  (data, in `tools/e2e/scenarios/`), even when the runner support for a step
  kind is another package's job — the scenario is the spec.
- No unit tests. No `#[cfg(test)]` modules. `cargo test` runs nothing by
  design; CI runs `cargo build --release` + `openergy-e2e run`.
- Size discipline: each package reports its release-binary delta.
  Adding a dependency outside section 1.5 needs orchestrator approval.

### 7.1 Packages

**WP1 — Protocol codec.** Owns `crates/protocol/`.
Implements framing, hello variants, every message in section 2.3, safety
limits (4.5), allocation-free hot encode. Depends on: round-2 captured
hello/`DKDL` bytes (can start against Deskflow reference values, must
re-verify). Acceptance: S13 hostile-peer pieces that are pure framing
(runner raw-socket probes vs a daemon stub), plus byte-exact golden frames
checked by the runner over a real socket (runner sends scripted raw frames,
asserts daemon's virtual ledger; runner reads daemon frames, compares bytes
to fixture files). Golden fixtures live in `tools/e2e/fixtures/frames/`.

**WP2 — Config + importers + layout core.** Owns `crates/config/`,
`crates/session/src/layout.rs` (only that file).
Implements TOML model/validation, `synergy.conf` parser + exporter,
`db.json` v17 + `synergy-core.conf` importers with reports, and the pure
`Layout::switch_target` engine. Acceptance: S14, I5 (export half), S2
layout halves once WP3 lands.

**WP3 — Session state machines + clipboard manager.** Owns
`crates/session/` except `layout.rs`.
Implements `Server`/`Client::handle`, pressed-state ledgers, heartbeat
math, option handling (4.3), extension receive behaviour (4.4), clipboard
sequence/marshal/chunk/reassembly (2.6). Acceptance: S2–S5, S8–S12, S17
(with WP4+WP6 present). This package defines *behaviour*; the others move
bytes and events.

**WP4 — Net: poll loop, TLS, trust, discovery.** Owns `crates/net/`.
Implements `mio` loop, priority queues (3.4), rustls buffered connections,
fingerprint trust store + held-connection flow (2.9), reconnect timers
plumbing (policy numbers from WP3/daemon), mDNS advertise/browse (2.13).
Acceptance: S6, S7, S11, I1, I2.

**WP5 — macOS platform backend.** Owns `crates/platform-macos/`.
Implements section 5 in full: tap, injection + activation fix, keymap,
pasteboard, permissions, display/power/lock notifications. Blocked at the
gate by the TCC-identity decision (5.8) only for the *packaging* half;
capture/injection work is not blocked. Acceptance: S16 (against virtual
first, then real backend in the manual tier), section 6.6 manual
checklist, plus a real-backend smoke scenario that runs only on a
dedicated account (daemon with `--platform macos`, IPC-driven, asserting
`status` permissions + tap liveness without synthesising desktop input).

**WP6 — Daemon + IPC + CLI.** Owns `crates/daemon/`, `crates/ipc/`.
Implements process lifecycle, the four threads and channels (section 3),
action execution, IPC server + method table (2.10), CLI surface (2.11),
logging (2.12), LaunchAgent plist + install plumbing. **This is the
integration package:** it wires WP1–WP5 behind frozen contracts and owns
no behaviour of its own beyond lifecycle. Acceptance: S1, S15, daemon.stop/
shutdown order (3.7) assertions across S4/S5.

**WP7 — Virtual backend + E2E runner + scenarios.** Owns
`crates/platform/src/virtual.rs`, `tools/e2e/`.
Implements the virtual backend (scripted source, recording sink with
`activate` markers, in-memory clipboard/screens/permissions), the runner
(spawn/drive/assert/teardown, raw-socket probes, `synergy-core` node kind
with the section 6.4 safety rails), and owns merging scenario files from
all packages without editing their intent. Acceptance: the runner executes
the full catalogue; S1–S17 + I1–I5 (headless tier) green in CI; failure
output includes child logs and recorded ledgers.

**WP8 — Windows + Linux backend stubs.** Owns `crates/platform-windows/`,
`crates/platform-linux/`.
Trait-complete, compile-green under cross targets (`cargo check --target
x86_64-pc-windows-*` / `x86_64-unknown-linux-gnu` in CI), returning typed
`NotImplemented` at runtime, with the OS-API mapping documented in code
comments for the post-v1 implementation. Acceptance: cross-checks pass;
the daemon on those targets starts, reports the backend as unavailable via
IPC, and exits cleanly. Cheap insurance against macOS-isms leaking into
the traits — if a stub cannot express a trait cleanly, that is a contract
bug to escalate immediately, while fixing it is still free.

**WP9 — Swift IPC client shell (orchestrator-owned design; listed for the
boundary, not delegated here).** Owns `apps/macos/`.
Thin client over section 2.10: status, connect/disconnect, trust approval,
permission onboarding, config editing surface. No logic that the daemon
doesn't own. Acceptance is the orchestrator's (UI design + manual flows).
Architectural constraint only: the app must remain fully functional to
*close* without affecting the daemon, and the daemon without the app.

That is 8 delegable packages (WP1–WP8) + the orchestrator's WP9. If more
parallelism is wanted, split WP3 into (3a) server/client machines and
(3b) clipboard manager — the `session::clipboard` module boundary already
supports it — and split WP5 into (5a) capture/injection and (5b)
pasteboard/permissions/lifecycle. Do not split WP1 or WP6 further.

### 7.2 Ordering and critical path

```
Step 0 (skeleton + frozen stubs + red E2E)
  ├── WP1 (codec) ──────────────┐
  ├── WP2 (config + layout) ────┤
  ├── WP8 (stubs, anytime)      ├─▶ WP3 (session) ──▶ WP6 (daemon integration) ──▶ interop gate I1–I5
  ├── WP4 (net/TLS) ────────────┤        ▲                     ▲
  ├── WP7 (virtual + runner) ───┘        │                     │
  └── WP5 (macOS backend) ───────────────┴─────────────────────┘
WP9 (Swift shell) — starts once IPC (WP6) answers `status.get`; independent after
```

**Critical path:** Step 0 → WP1 → WP3 → WP6 → I-series gate → WP5 manual
acceptance. WP5 is on the *shipping* path but not the *protocol* path:
the whole system can be proven against Synergy with the virtual backend
before the event tap exists. Staff accordingly: the strongest agent goes
to WP3 (behavioural correctness), the second to WP5 (OS correctness).
WP2/WP4/WP7 are parallel from day one. WP8 is filler work that protects
the contracts.

Merge discipline: land in order WP1/WP2/WP4/WP7 (leaves) → WP3 → WP5/WP8 →
WP6 last per increment, in small increments (connect-only first: S1;
then input: S2/S3; then clipboard: S8/S9; then TLS: S6). The branch is
never allowed to sit unbuildable; every merge keeps already-green
scenarios green.

### 7.3 Per-package brief template (for the orchestrator's agent prompts)

Each brief must contain: owned paths, frozen contracts it builds against
(paste the signatures), the R2-VERIFY items it may not assume, its E2E
scenario IDs, the dependency allowlist rule, the no-unit-tests rule, and
"escalate contract changes, never fork them". A brief that lacks the
scenario IDs will produce unverifiable code.

---

## 8. Risks, deferrals, and open questions for round 2

### 8.1 Risks and mitigations

| # | Risk | Impact | Mitigation in this design | Residual |
|---|---|---|---|---|
| R-A | Synergy 1.21 wire details differ from Deskflow reference (hello bytes, `DKDL`, `DSOP` ids, clipboard marshalling) | Interop fails at first contact | Every such item is tagged **R2-VERIFY**; WP1 writes against captured bytes; golden-frame fixtures from real captures | Medium until round 2 lands; low after |
| R-B | macOS 27 TCC/activation APIs behave differently from documented history (responsibility, `activateWithOptions`, tap grants) | Grants don't stick, or activation fix needs AX path | Decision gate before WP5 packaging (5.8); activation has a config-visible AX fallback (5.3); manual checklist 6.6 | Medium — needs on-device proof early. **Do a one-day spike before WP5 proper: minimal tap + inject + activate proof binary on the target Mac** |
| R-C | Key mapping (layouts, dead keys, lock-key `fix.*` semantics, language sync) is subtly wrong | Wrong characters on the peer — the most *felt* failure class | Port Deskflow's mapping approach, verify tables in round 2; Unicode fallback; manual multi-layout checklist | Medium. Budget real time here; do not let an agent improvise a keymap |
| R-D | `synergy-core` won't run headless for I-series | Interop gate weakens to manual testing | Split tiers in 6.4; raw-socket probes still prove handshake/`DSOP`/TLS; `--allow-desktop` tier on a dedicated account | Medium until round 2 answers the headless question |
| R-E | Event-tap callback jank (our callback too slow → macOS disables tap → dropped input) | Intermittent capture loss | Callback does translation only (3.3); tap-disabled re-enable + counters in status | Low |
| R-F | A stuck-key path survives somewhere creative (config reload mid-press, display change mid-drag) | Worst user-visible bug this category has | Single ledger + single leave path (2.4 invariants); S4/S5/S11/S12/S16 each attack a different exit; review rule: any new exit path must call the same routine | Low-medium; keep adding exit-path scenarios as features land |
| R-G | Clipboard format conversion edge cases (RTF-only sources, promised/lazy pasteboard data, huge BMP) | Missing formats, worker stalls | Worker thread isolation; format-skip-with-log rule; PNG-first policy; cap enforcement before read where the OS allows size query | Medium on Windows later; low for macOS text/PNG v1 |
| R-H | Parallel agents drift the frozen contracts | Integration week from hell | Step-0 stubs, escalate-don't-fork rule, WP6 integrates in small increments behind green scenarios | Low if the orchestrator enforces the gate |
| R-I | Binary size creep (TLS + mDNS + serde) past 15 MB | Misses a headline goal | Allowlist + size reporting per package; no webpki-roots; `ring` over aws-lc; check `mdns-sd` cost early, drop discovery before dropping anything on the input path | Low-medium; measure at first integration, not at the end |
| R-J | Legal/licensing: Deskflow is GPL; Synergy is commercial | Wrong code reuse contaminates the project | Clean-room against protocol *behaviour* and the published reference; do not copy Deskflow C++ source into Rust. Reading it to understand wire semantics and reimplementing is the intended posture; the orchestrator picks Openergy's own license (recommend MIT/Apache-2.0 dual, standard for Rust). Get the R2 capture methodology from black-box observation (sockets, logs, configs) wherever possible | Low with discipline; flag for the orchestrator explicitly |

### 8.2 Deferred from v1 (explicit non-goals)

- File transfer (`DFTR`) and cross-screen drag-and-drop (`DDRG`) —
  parsed-and-declined (4.4). Revisit only after v1 ships; they are the
  highest complexity-per-value feature in Deskflow.
- Windows and Linux *behavioural* backends (stubs only, WP8). Wayland is
  out entirely until its input-injection story (portals/libei) gets its
  own design round.
- Openergy-only protocol extensions and peer detection (4.3). v1 speaks
  conservative Synergy dialect to everyone.
- Per-monitor layout modelling beyond one bounding rect per screen
  (Deskflow's own model). Non-rectangular multi-monitor dead zones are a
  post-v1 layout extension.
- Settings UI beyond the IPC method table; auto-updates; crash reporting
  (no Sentry — local logs only, by design and by privacy); accounts,
  licensing, cloud anything. Ever.
- IPv6-only and non-LAN (WAN/relay) operation. TCP over LAN/VPN as-is.
- Screen-saver sync, lock-screen input sharing, `CSEC`-driven behaviours
  beyond advisory status.

### 8.3 Open questions that round 2 (reverse engineering) must answer

Ordered by how much code they unblock:

- **O1 (WP1):** Exact hello/hello-back bytes and advertised version of
  synergy-core 1.21.1-beta; `DKDN` vs language-variant encoding; does the
  core ever emit message codes outside the recon list?
- **O2 (WP1/WP3):** Clipboard marshalling layout and format-id table;
  chunk mark values (start/data/end); max chunk size the core sends.
- **O3 (WP3):** `DSOP` option-id table the core emits/absorbs; `CSEC`
  polarity; `SECN`/`LSYN` payload formats and trigger conditions.
- **O4 (WP4):** TLS profile of the core (versions, suites, client-cert
  requirement, fingerprint algorithm/format in `trusted-servers` — assumed
  SHA-256 over the DER cert; confirm), and whether TLS is on the same port
  via immediate handshake (assumed) vs any STARTTLS-like step.
- **O5 (WP5):** macOS 27 answers — tap grant requirements, activation API
  from an agent process (+ whether AX raise is needed), TCC responsibility
  for a daemon embedded in the app bundle, `SMAppService` form.
- **O6 (WP5):** Wheel-unit scaling and keymap/button derivation in
  Deskflow's macOS code; secure-input tap visibility (5.5).
- **O7 (WP7):** `synergy-core` headless behaviour and logging flags for
  the I-series; which env/config knobs fully isolate it from the live
  install on this Mac.
- **O8 (WP2):** `db.json` v17 full field inventory beyond recon (modifier
  remap encoding, `hdk.*` meaning, layout-position → link derivation
  rules); Synergy 3 discovery mechanism (mDNS service type or proprietary).
- **O9 (WP3):** Does the Synergy 3 *server* (Windows) send anything
  service-specific on the core wire (i.e., is the Windows core's dialect
  identical to the Mac core's)? One capture from the live topology —
  read-only, passive — answers this and de-risks the whole project.

### 8.4 Assumptions this draft stands on (cheap to change now, expensive later)

1. One bounding rectangle per screen is enough for v1 layout (Deskflow parity).
2. JSON-over-socket IPC is acceptable to the Swift app's design (orchestrator).
3. No Mac App Store distribution (sandbox incompatibility) — notarized
   direct distribution only.
4. `panic = "abort"` + supervisor restart is preferred over in-process
   recovery for input-path thread failures.
5. The user's Windows Synergy 3 server stays unmodified and in place during
   development; Openergy proves itself as a client first.

## Sources

- Deskflow protocol reference (protocol roles, state machine, message table): https://deskflow.github.io/deskflow/protocol_reference.html
- `mio::Waker` (cross-thread wake for `Poll`): https://docs.rs/mio/latest/mio/struct.Waker.html
- rustls manual (buffered/manual connection model): https://docs.rs/rustls/0.23.20/rustls/manual/index.html
- All other facts: the orchestrator's recon brief for this round (Synergy 3.7.0-beta / synergy-core 1.21.1-beta on the target Mac), marked **R2-VERIFY** wherever binary confirmation is still owed.
