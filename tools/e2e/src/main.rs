//! Step 0 E2E skeleton: spawn the real `openergy` daemon, call `status.get`
//! over IPC, assert the ui-contract shape, stop it with `daemon.stop`.
//! The TOML scenario language lands in WP7. Never binds port 24800.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::net::UnixStream;

use openergy_ipc::{methods, read_frame, write_frame, Request};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => println!("S0 step0-smoke: PASS"),
        Err(e) => {
            eprintln!("S0 step0-smoke: FAIL: {e}");
            std::process::exit(1);
        }
    }
}

fn openergy_bin() -> PathBuf {
    if let Ok(p) = std::env::var("OPENERGY_BIN") {
        return PathBuf::from(p);
    }
    let mut candidates = vec![
        PathBuf::from("target/release/openergy"),
        PathBuf::from("target/debug/openergy"),
    ];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.insert(0, dir.join("openergy"));
        }
    }
    candidates
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from("openergy"))
}

fn run(args: &[String]) -> Result<(), String> {
    let bin = openergy_bin();
    let socket = args
        .windows(2)
        .find(|w| w[0] == "--socket")
        .map(|w| PathBuf::from(&w[1]))
        .unwrap_or_else(|| std::env::temp_dir().join(format!("openergy-e2e-{}.sock", std::process::id())));
    let _ = std::fs::remove_file(&socket);
    let mut child: Child = Command::new(&bin)
        .args(["daemon", "--platform", "virtual", "--foreground", "--socket"])
        .arg(&socket)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn {}: {e}", bin.display()))?;

    let result = (|| -> Result<serde_json::Value, String> {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match ipc(&socket, methods::STATUS_GET, serde_json::json!({})) {
                Ok(v) => return Ok(v),
                Err(e) => {
                    if Instant::now() > deadline {
                        return Err(format!("daemon never answered status.get: {e}"));
                    }
                    if let Ok(Some(st)) = child.try_wait() {
                        return Err(format!("daemon exited early: {st}"));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
        }
    })();

    let status = match result {
        Ok(v) => v,
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e);
        }
    };
    println!(
        "status.get result: {}",
        serde_json::to_string_pretty(&status).unwrap_or_default()
    );
    for field in [
        "daemon",
        "role",
        "state",
        "name",
        "active_screen",
        "cursor",
        "local_screen",
        "server",
        "client",
        "peers",
        "tls",
        "permissions",
        "secure_input",
        "clipboard",
        "pending_trust",
        "last_error",
        "stats",
    ] {
        if status.get(field).is_none() {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("status is missing required field {field:?}"));
        }
    }
    if status.get("role").and_then(|v| v.as_str()) != Some("off") {
        let _ = child.kill();
        let _ = child.wait();
        return Err("fresh virtual daemon role must be \"off\"".into());
    }
    let _ = ipc(&socket, methods::DAEMON_STOP, serde_json::json!({}));
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(st) => {
                if !st.success() {
                    return Err(format!("daemon exit status {st}"));
                }
                println!("daemon stopped cleanly: {st}");
                break;
            }
            None => {
                if Instant::now() > deadline {
                    let _ = child.kill();
                    return Err("daemon did not stop after daemon.stop".into());
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
fn ipc(socket: &PathBuf, method: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
    let mut stream = UnixStream::connect(socket).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| e.to_string())?;
    let req = Request {
        id: 1,
        method: method.to_string(),
        params,
    };
    let v = serde_json::to_value(&req).map_err(|e| e.to_string())?;
    write_frame(&mut stream, &v).map_err(|e| e.to_string())?;
    let resp = read_frame(&mut stream).map_err(|e| e.to_string())?;
    if let Some(err) = resp.get("error") {
        return Err(format!("{method} error: {err}"));
    }
    Ok(resp.get("result").cloned().unwrap_or(serde_json::Value::Null))
}
#[cfg(not(unix))]
fn ipc(_socket: &PathBuf, _method: &str, _params: serde_json::Value) -> Result<serde_json::Value, String> {
    Err("e2e IPC requires Unix sockets in Step 0".into())
}
