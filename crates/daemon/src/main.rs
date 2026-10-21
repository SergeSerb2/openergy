//! The Openergy daemon and hand-rolled command-line interface.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::os::unix::net::UnixStream;

use openergy_config::Config;
use openergy_ipc::{methods, read_frame, write_frame, Request, Response};

mod adapter;
mod clipboard_json;
mod events;
mod interfaces;
mod ipc_server;
mod lifetime;
mod logger;
mod runtime;
mod workers;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => std::process::exit(code),
        Err(msg) => {
            eprintln!("openergy: {msg}");
            if msg.starts_with("unknown ") && !msg.contains("usage:") {
                eprintln!("{}", usage());
            }
            std::process::exit(if msg.starts_with("unknown ") || msg.starts_with("usage:") {
                2
            } else {
                1
            });
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
        "trust" => openergy_net::cli::trust(&args[1..]),
        "fingerprint" => openergy_net::cli::fingerprint(&args[1..]),
        "import-synergy" => openergy_config::cli::import_synergy(&args[1..]),
        "export-synergy-conf" => openergy_config::cli::export_synergy_conf(&args[1..]),
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
                if !value_flags.contains(&k) {
                    return Err(format!("unknown option --{k}"));
                }
                map.insert(k.to_string(), val.to_string());
            } else if value_flags.contains(&v) {
                i += 1;
                let val = args.get(i).ok_or_else(|| format!("--{v} requires a value"))?;
                map.insert(v.to_string(), val.clone());
            } else {
                if !["foreground", "raw"].contains(&v) {
                    return Err(format!("unknown flag --{v}"));
                }
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
    if let Ok(path) = std::env::var("OPENERGY_CONFIG") {
        return path.into();
    }
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
        Some("auto") => platform_name(None),
        Some(other) => other.to_string(),
        None if cfg!(target_os = "macos") => "macos".into(),
        None if cfg!(target_os = "windows") => "windows".into(),
        None => "x11".into(),
    }
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
    if parent_pid.is_some_and(|p| p <= 1 || p > i32::MAX as i64) {
        return Err("--parent-pid must be > 1 and fit a process ID".into());
    }
    runtime::run_daemon(
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
