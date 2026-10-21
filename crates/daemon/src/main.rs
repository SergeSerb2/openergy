//! `openergy` binary: daemon + CLI (architecture-r2 §2.11). Step 0 scope:
//! `version`, `daemon` with a listening IPC socket answering `status.get` /
//! `daemon.stop` / `config.get`, `ctl`, `status`, `fingerprint`.
//! All other subcommands parse and return a typed not-implemented error.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};

use openergy_config::{derive_links, Config};
use openergy_ipc::{events, methods, read_frame, write_frame, Event, IpcError, Request, Response, Status};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => std::process::exit(code),
        Err(msg) => {
            eprintln!("openergy: {msg}");
            std::process::exit(1);
        }
    }
}

fn usage() -> &'static str {
    "usage: openergy daemon|server|client [--config <path>] [--platform macos|virtual|auto] [--socket <path>] [--parent-pid <pid>] [--foreground] [--remote host:port]\n       openergy ctl <method> [--socket <path>] [--json '{...}'] [--raw]\n       openergy status|connect|disconnect|restart [--socket <path>]\n       openergy trust list|approve <fp>|reject <fp>|revoke <fp>\n       openergy fingerprint [--synergy]\n       openergy import-synergy --from <dir> [--out <path>] [--apply]\n       openergy export-synergy-conf [--out <path>]\n       openergy version"
}

fn run(args: &[String]) -> Result<i32, String> {
    let Some(cmd) = args.first().map(String::as_str) else {
        return Err(usage().into());
    };
    match cmd {
        "version" | "--version" | "-V" => {
            println!("openergy {VERSION}");
            Ok(0)
        }
        "daemon" | "server" | "client" => cmd_daemon(cmd, &args[1..]),
        "ctl" => cmd_ctl(&args[1..]),
        "status" => ctl_call(&args[1..], methods::STATUS_GET, serde_json::json!({}), false),
        "connect" => ctl_call(&args[1..], methods::CONNECTION_CONNECT, serde_json::json!({}), false),
        "disconnect" => ctl_call(&args[1..], methods::CONNECTION_DISCONNECT, serde_json::json!({}), false),
        "restart" => ctl_call(&args[1..], methods::CONNECTION_RESTART, serde_json::json!({}), false),
        "trust" => Err("trust: not implemented in Step 0 (WP4/WP6)".into()),
        "fingerprint" => cmd_fingerprint(&args[1..]),
        "import-synergy" => Err("import-synergy: not implemented in Step 0 (WP2)".into()),
        "export-synergy-conf" => Err("export-synergy-conf: not implemented in Step 0 (WP2)".into()),
        "help" | "--help" | "-h" => {
            println!("{}", usage());
            Ok(0)
        }
        other => Err(format!("unknown command {other:?}\n{}", usage())),
    }
}

struct Opts {
    map: BTreeMap<String, String>,
    flags: Vec<String>,
    positional: Vec<String>,
}
fn parse_opts(args: &[String], value_flags: &[&str]) -> Result<Opts, String> {
    let mut map = BTreeMap::new();
    let mut flags = Vec::new();
    let mut positional = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if let Some(v) = a.strip_prefix("--") {
            if let Some((k, val)) = v.split_once('=') {
                map.insert(k.to_string(), val.to_string());
            } else if value_flags.contains(&v) {
                i += 1;
                let val = args.get(i).ok_or_else(|| format!("--{v} requires a value"))?;
                map.insert(v.to_string(), val.clone());
            } else {
                flags.push(v.to_string());
            }
        } else {
            positional.push(a.clone());
        }
        i += 1;
    }
    Ok(Opts { map, flags, positional })
}

fn default_socket() -> PathBuf {
    if let Ok(x) = std::env::var("OPENERGY_SOCKET") {
        return PathBuf::from(x);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    #[cfg(target_os = "macos")]
    {
        PathBuf::from(home).join("Library/Application Support/Openergy/openergy.sock")
    }
    #[cfg(not(target_os = "macos"))]
    {
        if let Ok(rt) = std::env::var("XDG_RUNTIME_DIR") {
            PathBuf::from(rt).join("openergy.sock")
        } else {
            PathBuf::from(home).join(".local/share/openergy/openergy.sock")
        }
    }
}

fn default_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    #[cfg(target_os = "macos")]
    {
        PathBuf::from(home).join("Library/Application Support/Openergy/config.toml")
    }
    #[cfg(not(target_os = "macos"))]
    {
        PathBuf::from(home).join(".config/openergy/config.toml")
    }
}

fn platform_name(given: Option<&str>) -> String {
    match given {
        Some("virtual") => "virtual".into(),
        Some("macos") => "macos".into(),
        Some(other) => other.to_string(),
        None if cfg!(target_os = "macos") => "macos".into(),
        None if cfg!(target_os = "windows") => "windows".into(),
        None => "linux".into(),
    }
}

struct DaemonState {
    started: Instant,
    config: Arc<Mutex<Config>>,
    config_path: Option<PathBuf>,
    platform: String,
    stop: Arc<AtomicBool>,
    log_lines: Mutex<Vec<String>>,
}

impl DaemonState {
    fn status(&self) -> Status {
        let cfg = self.config.lock().map(|c| c.clone()).unwrap_or_default();
        let mut s = Status::idle(VERSION, std::process::id(), &self.platform);
        s.daemon.uptime_s = self.started.elapsed().as_secs();
        s.role = cfg.role.as_str().to_string();
        s.name = if cfg.name.is_empty() {
            None
        } else {
            Some(cfg.name.clone())
        };
        s.tls.enabled = cfg.tls.enabled;
        s.clipboard.enabled = cfg.clipboard.enabled;
        if cfg.role == openergy_config::Role::Server {
            s.server.listen = Some(format!("{}:{}", trim_listen_host(&cfg.server.listen), cfg.server.port));
        }
        if cfg.role == openergy_config::Role::Client && !cfg.client.remote.is_empty() {
            s.client.remote = Some(cfg.client.remote.clone());
        }
        if self.platform == "virtual" {
            s.permissions.accessibility = "not_required".into();
            s.permissions.input_monitoring = "not_required".into();
            s.permissions.post_events = "not_required".into();
        }
        s
    }
    fn log(&self, line: String) {
        eprintln!("{line}");
        if let Ok(mut v) = self.log_lines.lock() {
            v.push(line);
            let excess = v.len().saturating_sub(1000);
            if excess > 0 {
                v.drain(..excess);
            }
        }
    }
}

fn trim_listen_host(listen: &str) -> &str {
    listen.rsplit_once(':').map(|(h, _)| h).unwrap_or(listen)
}

fn cmd_daemon(cmd: &str, args: &[String]) -> Result<i32, String> {
    let opts = parse_opts(args, &["config", "platform", "socket", "parent-pid", "remote"])?;
    for f in &opts.flags {
        if f != "foreground" {
            return Err(format!("unknown flag --{f}"));
        }
    }
    let socket_path = opts.map.get("socket").map(PathBuf::from).unwrap_or_else(default_socket);
    let config_path = opts.map.get("config").map(PathBuf::from);
    let mut config = load_config(config_path.as_deref())?;
    match cmd {
        "server" => config.role = openergy_config::Role::Server,
        "client" => {
            config.role = openergy_config::Role::Client;
            if let Some(r) = opts.map.get("remote") {
                config.client.remote = r.clone();
            }
        }
        _ => {}
    }
    let parent_pid: Option<i64> = match opts.map.get("parent-pid") {
        Some(v) => Some(
            v.parse::<i64>()
                .map_err(|_| "--parent-pid must be an integer".to_string())?,
        ),
        None => None,
    };
    run_daemon(
        socket_path,
        config,
        config_path,
        platform_name(opts.map.get("platform").map(String::as_str)),
        parent_pid,
    )
}

fn load_config(path: Option<&Path>) -> Result<Config, String> {
    match path {
        Some(p) if p.exists() => {
            let text = std::fs::read_to_string(p).map_err(|e| format!("read {}: {e}", p.display()))?;
            Config::load_toml(&text).map_err(|e| format!("config {}: {e}", p.display()))
        }
        Some(_) => Ok(Config::default()),
        None => {
            let p = default_config_path();
            if p.exists() {
                let text = std::fs::read_to_string(&p).map_err(|e| format!("read {}: {e}", p.display()))?;
                Config::load_toml(&text).map_err(|e| format!("config {}: {e}", p.display()))
            } else {
                Ok(Config::default())
            }
        }
    }
}

fn run_daemon(
    socket_path: PathBuf,
    config: Config,
    config_path: Option<PathBuf>,
    platform: String,
    parent_pid: Option<i64>,
) -> Result<i32, String> {
    #[cfg(not(unix))]
    {
        let _ = (socket_path, config, config_path, platform, parent_pid);
        return Err("daemon IPC requires a Unix socket in Step 0".into());
    }
    #[cfg(unix)]
    {
        if let Some(parent) = socket_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
            set_mode(parent, 0o700);
        }
        if socket_path.exists() {
            if UnixStream::connect(&socket_path).is_ok() {
                return Err(format!(
                    "another daemon is already listening on {}",
                    socket_path.display()
                ));
            }
            let _ = std::fs::remove_file(&socket_path);
        }
        let listener = UnixListener::bind(&socket_path).map_err(|e| format!("bind {}: {e}", socket_path.display()))?;
        set_mode(&socket_path, 0o600);
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;
        let stop = Arc::new(AtomicBool::new(false));
        let state = Arc::new(DaemonState {
            started: Instant::now(),
            config: Arc::new(Mutex::new(config)),
            config_path,
            platform,
            stop: stop.clone(),
            log_lines: Mutex::new(Vec::new()),
        });
        state.log(format!(
            "openergy {VERSION} daemon listening on {}",
            socket_path.display()
        ));
        // Parent watch: poll liveness (kqueue NOTE_EXIT lands in WP6; poll is correct, just coarser).
        if let Some(ppid) = parent_pid {
            let stop2 = stop.clone();
            let st = state.clone();
            std::thread::spawn(move || loop {
                if stop2.load(Ordering::Relaxed) {
                    break;
                }
                // Safety: kill(pid, 0) only probes existence; it sends no signal.
                let alive = unsafe { libc::kill(ppid as libc::pid_t, 0) } == 0;
                if !alive {
                    st.log("parent exited; shutting down".into());
                    stop2.store(true, Ordering::Relaxed);
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(250));
            });
        }
        while !stop.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _)) => {
                    // Accepted sockets inherit O_NONBLOCK from the listener on macOS/BSD;
                    // the per-connection thread needs blocking reads.
                    if stream.set_nonblocking(false).is_err() {
                        continue;
                    }
                    let st = state.clone();
                    std::thread::spawn(move || handle_ipc_conn(stream, st));
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(e) => {
                    state.log(format!("accept failed: {e}"));
                    break;
                }
            }
        }
        let _ = std::fs::remove_file(&socket_path);
        state.log("openergy daemon stopped".into());
        Ok(0)
    }
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode));
}

#[cfg(unix)]
fn handle_ipc_conn(mut stream: UnixStream, state: Arc<DaemonState>) {
    let mut subscribed = false;
    let mut log_subscribed = false;
    loop {
        let Ok(value) = read_frame(&mut stream) else { return };
        let Ok(req) = serde_json::from_value::<Request>(value) else {
            return;
        };
        let (resp, stop_after) = dispatch(&state, &req, &mut subscribed, &mut log_subscribed);
        let Ok(resp_v) = serde_json::to_value(&resp) else {
            return;
        };
        if write_frame(&mut stream, &resp_v).is_err() {
            return;
        }
        if req.method == methods::EVENTS_SUBSCRIBE {
            let ev = Event {
                event: events::STATUS_CHANGED.into(),
                data: state.status().to_json(),
            };
            if let Ok(v) = serde_json::to_value(&ev) {
                let _ = write_frame(&mut stream, &v);
            }
        }
        let _ = log_subscribed;
        if stop_after {
            state.stop.store(true, Ordering::Relaxed);
            return;
        }
    }
}

fn dispatch(state: &DaemonState, req: &Request, subscribed: &mut bool, log_subscribed: &mut bool) -> (Response, bool) {
    match req.method.as_str() {
        m if m == methods::STATUS_GET => (Response::ok(req.id, state.status().to_json()), false),
        m if m == methods::CONFIG_GET => {
            let cfg = state.config.lock().map(|c| c.clone()).unwrap_or_default();
            let links = serde_json::to_value(derive_links(&cfg.screens)).unwrap_or(serde_json::json!([]));
            (
                Response::ok(
                    req.id,
                    serde_json::json!({"config": cfg.to_json(), "derived_links": links}),
                ),
                false,
            )
        }
        m if m == methods::CONFIG_PATCH => {
            let Some(patch) = req.params.get("patch").cloned() else {
                return (
                    Response::err(req.id, IpcError::new("bad_params", "config.patch requires {patch}")),
                    false,
                );
            };
            let current = state.config.lock().map(|c| c.clone()).unwrap_or_default();
            match openergy_config::apply_merge_patch(&current, &patch) {
                Ok((new_cfg, warnings)) => {
                    if let Err(e) = write_config_atomic(state, &new_cfg) {
                        return (Response::err(req.id, IpcError::new("config_write_failed", e)), false);
                    }
                    if let Ok(mut slot) = state.config.lock() {
                        *slot = new_cfg.clone();
                    }
                    (
                        Response::ok(
                            req.id,
                            serde_json::json!({"config": new_cfg.to_json(), "warnings": warnings}),
                        ),
                        false,
                    )
                }
                Err(e) => (
                    Response::err(req.id, IpcError::new("invalid_config", e.to_string())),
                    false,
                ),
            }
        }
        m if m == methods::DAEMON_STOP => (Response::ok(req.id, serde_json::json!({"stopping": true})), true),
        m if m == methods::EVENTS_SUBSCRIBE => {
            *subscribed = true;
            (Response::ok(req.id, serde_json::json!({"subscribed": true})), false)
        }
        m if m == methods::LOGS_SUBSCRIBE => {
            *log_subscribed = true;
            (Response::ok(req.id, serde_json::json!({"subscribed": true})), false)
        }
        m if m == methods::LOGS_UNSUBSCRIBE => {
            *log_subscribed = false;
            (Response::ok(req.id, serde_json::json!({"subscribed": false})), false)
        }
        m if m == methods::LOGS_TAIL => {
            let lines: usize = req.params.get("lines").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
            let all = state.log_lines.lock().map(|v| v.clone()).unwrap_or_default();
            let tail: Vec<String> = all.into_iter().rev().take(lines).rev().collect();
            (
                Response::ok(req.id, serde_json::to_value(tail).unwrap_or(serde_json::json!([]))),
                false,
            )
        }
        m if m == methods::TRUST_LIST => (
            Response::ok(req.id, serde_json::json!({"servers": [], "clients": []})),
            false,
        ),
        m if m == methods::DISCOVERY_LIST => (Response::ok(req.id, serde_json::json!([])), false),
        m if m.starts_with("virtual.") && state.platform != "virtual" => (
            Response::err(
                req.id,
                IpcError::new("virtual_unavailable", "virtual.* requires --platform virtual"),
            ),
            false,
        ),
        m if m == methods::VIRTUAL_RECORDED => (Response::ok(req.id, serde_json::json!({"events": []})), false),
        _ => (
            Response::err(
                req.id,
                IpcError::new(
                    "not_implemented",
                    format!("{} is not implemented in Step 0", req.method),
                ),
            ),
            false,
        ),
    }
}

fn write_config_atomic(state: &DaemonState, cfg: &Config) -> Result<(), String> {
    let Some(path) = &state.config_path else { return Ok(()) };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, cfg.to_toml()).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

fn cmd_fingerprint(args: &[String]) -> Result<i32, String> {
    let synergy = args.iter().any(|a| a == "--synergy");
    if synergy {
        // Step 0 has no TLS identity yet (WP4). Say so instead of printing a fake pin.
        return Err("no TLS identity exists yet (Step 0; identity generation lands in WP4)".into());
    }
    Err("no TLS identity exists yet (Step 0; identity generation lands in WP4)".into())
}

fn cmd_ctl(args: &[String]) -> Result<i32, String> {
    let opts = parse_opts(args, &["socket", "json"])?;
    let method = opts
        .positional
        .first()
        .ok_or("ctl requires a method, e.g. openergy ctl status.get")?
        .clone();
    let params: serde_json::Value = match opts.map.get("json") {
        Some(s) => serde_json::from_str(s).map_err(|e| format!("--json: {e}"))?,
        None => serde_json::json!({}),
    };
    let raw = opts.flags.iter().any(|f| f == "raw");
    let socket = opts.map.get("socket").map(PathBuf::from).unwrap_or_else(default_socket);
    let resp = ipc_request(&socket, &method, params)?;
    print_response(&resp, raw);
    Ok(if resp.error.is_some() { 1 } else { 0 })
}

fn ctl_call(args: &[String], method: &str, params: serde_json::Value, raw: bool) -> Result<i32, String> {
    let opts = parse_opts(args, &["socket"])?;
    let socket = opts.map.get("socket").map(PathBuf::from).unwrap_or_else(default_socket);
    let resp = ipc_request(&socket, method, params)?;
    print_response(&resp, raw);
    Ok(if resp.error.is_some() { 1 } else { 0 })
}

fn print_response(resp: &Response, raw: bool) {
    if let Some(err) = &resp.error {
        if raw {
            println!("{}", serde_json::to_string(resp).unwrap_or_default());
        } else {
            println!("error {}: {}", err.code, err.message);
        }
        return;
    }
    let result = resp.result.clone().unwrap_or(serde_json::Value::Null);
    if raw {
        println!("{}", serde_json::to_string(&result).unwrap_or_default());
    } else {
        println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
    }
}

#[cfg(unix)]
fn ipc_request(socket: &Path, method: &str, params: serde_json::Value) -> Result<Response, String> {
    let mut stream = UnixStream::connect(socket)
        .map_err(|e| format!("connect {}: {e} (is the daemon running?)", socket.display()))?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .map_err(|e| e.to_string())?;
    let id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(1);
    let req = Request {
        id,
        method: method.to_string(),
        params,
    };
    let v = serde_json::to_value(&req).map_err(|e| e.to_string())?;
    write_frame(&mut stream, &v).map_err(|e| e.to_string())?;
    let resp_v = read_frame(&mut stream).map_err(|e| e.to_string())?;
    serde_json::from_value(resp_v).map_err(|e| format!("bad IPC response: {e}"))
}
#[cfg(not(unix))]
fn ipc_request(_socket: &Path, _method: &str, _params: serde_json::Value) -> Result<Response, String> {
    Err("IPC client requires Unix sockets in Step 0".into())
}

// Keep Read/Write imports used on all targets.
#[allow(dead_code)]
fn _io_marker(r: &mut dyn Read, w: &mut dyn Write) -> std::io::Result<()> {
    let mut b = [0u8; 0];
    let _ = r.read(&mut b)?;
    w.flush()
}
