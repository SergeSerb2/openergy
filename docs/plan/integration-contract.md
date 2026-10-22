# Integration contract additions (orchestrator, binding)

Step 0 froze the message, event and trait types, but not how the daemon constructs and
drives the transport and the platform backends. These additive APIs close that gap.
Owners implement them in their own crates; WP6 (daemon) consumes them exactly as written.

## 1. `net` driver (owner: WP4, crate `openergy-net`)

The daemon's io thread owns one `NetDriver`. Everything is synchronous; no runtime.

```rust
pub struct NetConfig {
    pub tls_enabled: bool,
    pub tls_dir: std::path::PathBuf,        // identity (cert.pem/key.pem 0600) + trusted-servers/trusted-clients
    pub identity_name: String,              // used in the self-signed cert CN
    pub require_client_trust: bool,         // server side: pin client certs (held until approved)
    pub held_timeout: std::time::Duration,  // default 30 s
    pub max_frame_len: u32,                 // default 4 MiB (protocol cap)
    pub mdns: Option<MdnsConfig>,           // None = discovery off
}
pub struct MdnsConfig { pub advertise: Option<(String /*name*/, u16 /*port*/)>, pub browse: bool, pub loopback_only: bool }

pub struct NetDriver { /* mio::Poll, connections, TLS state, trust store, identity */ }
#[derive(Clone)] pub struct NetHandle { /* mpsc::Sender<NetCommand> + Arc<mio::Waker> */ }

impl NetDriver {
    pub fn new(config: NetConfig) -> Result<(NetDriver, NetHandle), NetError>;
    /// Wake handle for other threads (capture, clipboard worker, IPC) that queue work for the io thread.
    pub fn waker(&self) -> std::sync::Arc<mio::Waker>;
    /// One reactor iteration: wait up to `timeout` (None = block) for readiness, the waker, or NetHandle
    /// commands; perform all I/O; append resulting events to `out`. Never blocks longer than `timeout`.
    pub fn poll(&mut self, timeout: Option<std::time::Duration>, out: &mut Vec<NetEvent>) -> Result<(), NetError>;
    /// Same-thread command execution (the io thread uses this instead of NetHandle).
    pub fn execute(&mut self, cmd: NetCommand, out: &mut Vec<NetEvent>);
    /// Trust the fingerprint (persist to the right trust file) and resume every connection held for it.
    pub fn approve_held(&mut self, fp: &Fingerprint, out: &mut Vec<NetEvent>) -> Result<(), NetError>;
    /// Close every connection held for this fingerprint.
    pub fn reject_held(&mut self, fp: &Fingerprint, out: &mut Vec<NetEvent>);
    pub fn trust_store(&self) -> &TrustStore;
    pub fn trust_store_mut(&mut self) -> &mut TrustStore;
    pub fn identity_fingerprint(&self) -> Option<Fingerprint>;
    /// Apply a new config (TLS toggle, trust dir, mDNS) to future connections; existing ones stay.
    pub fn reconfigure(&mut self, config: NetConfig) -> Result<(), NetError>;
    pub fn conn_info(&self, conn: ConnId) -> Option<ConnInfo>;
}
pub struct ConnInfo { pub peer: std::net::SocketAddr, pub outbound: bool, pub tls_version: Option<String>,
                      pub fingerprint: Option<Fingerprint>, pub queued_bytes: usize }
```

Additive `NetEvent` variants: `ListenFailed { addr, error: String }`,
`ConnectFailed { host: String, port: u16, error: String }`, `Held { conn, fingerprint, addr }`
may replace or alias `UntrustedPeer` (keep `UntrustedPeer` as the name if simpler).
Rules: `NetCommand::Connect` produces exactly one of `Connected`, `UntrustedPeer` (then later
`Connected` or `Disconnected`), or `ConnectFailed`. No `Message` events are delivered for a held
connection. The first frame on a connection is the hello (outbound) or hello-back (accepted);
the protocol `Decoder` recognises both by frame length (hello = 11-byte payload).
The listener refuses port 24800 unless `OPENERGY_ALLOW_PORT_24800=1` is set in the environment
(the production app sets it; tests never do).

## 2. Platform backend construction (owner: WP3 for the shared types, crate `openergy-platform`)

```rust
/// One platform backend split by thread ownership:
/// capture thread owns `source`, io thread owns `sink` + `screens`, clipboard worker owns `clipboard`.
pub struct Backend {
    pub name: &'static str,                          // "virtual" | "macos" | "x11" | "windows"
    pub source: Box<dyn InputSource>,
    pub sink: Box<dyn InputSink>,
    pub screens: Box<dyn ScreenSystem>,
    pub clipboard: Box<dyn ClipboardBackend>,
}

pub mod virtual_backend {
    pub struct VirtualOptions { pub screen_w: i32, pub screen_h: i32, pub hid_seed: i64, pub double_click_ms: u64 }
    /// Returns the backend parts plus a thread-safe control handle for the daemon's `virtual.*` IPC.
    pub fn create(options: VirtualOptions) -> (super::Backend, VirtualControl);
}
```

Every real backend exposes `pub fn create(options: <BackendOptions>) -> Result<openergy_platform::Backend, PlatformError>`:
`openergy_platform_macos::create(MacOptions { click_numbering: ClickNumbering, lines_per_notch: f64 })`,
`openergy_platform_linux::create(X11Options { display: Option<String> })`,
`openergy_platform_windows::create(WindowsOptions {})`.
`VirtualControl` methods and JSON shapes are documented in `crates/platform/VIRTUAL.md`.

## 3. Addenda (live apply and permission prompts)

- `NetCommand::Shutdown` closes the listener and every connection (one `Disconnected` each) but
  leaves the `NetDriver` reusable; later `Listen`/`Connect` work. New `NetCommand::StopListening`
  closes only listeners. `reconfigure(NetConfig)` then `Listen`/`Connect` uses the new settings.
- `ScreenSystem` gains a default method
  `fn request_permission(&mut self, kind: PermissionKind) -> Result<Grant, PlatformError>`
  (default: return the current state). The virtual backend records the request and returns the
  scripted grant; macOS overrides it with AXIsProcessTrustedWithOptions (prompt),
  CGRequestListenEventAccess and CGRequestPostEventAccess. The daemon's `permissions.request`
  calls it.
- IPC stays on its own threads: requests go to the io thread as
  `IoCommand::Ipc { req, reply }` over a channel plus `waker.wake()`; the io loop drains it
  every iteration. No IPC sockets in the net reactor.
