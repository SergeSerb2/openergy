//! A separate process owns cleanup when the runner exits or receives a signal.
//! EOF on its pipe also covers SIGKILL, where Rust destructors cannot run.
use crate::model::{Failure, Result};
use crate::process::signal_group;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

struct Guardian {
    child: Child,
    pipe: Option<ChildStdin>,
}
static GUARDIAN: OnceLock<Mutex<Guardian>> = OnceLock::new();
pub fn init() -> Result<()> {
    let mut command = Command::new(std::env::current_exe()?);
    command
        .arg("__guardian")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command.spawn()?;
    let pipe = child.stdin.take();
    GUARDIAN
        .set(Mutex::new(Guardian { child, pipe }))
        .map_err(|_| Failure::runner("cleanup guardian initialized twice"))?;
    Ok(())
}
fn send(value: Value) -> Result<()> {
    if let Some(guard) = GUARDIAN.get() {
        let mut guard = guard
            .lock()
            .map_err(|_| Failure::runner("cleanup guardian lock failed"))?;
        if let Some(pipe) = &mut guard.pipe {
            writeln!(pipe, "{value}").map_err(|e| Failure::environment(format!("cleanup guardian pipe: {e}")))?;
            pipe.flush()?;
        }
    }
    Ok(())
}
pub fn process(group: u32, add: bool) -> Result<()> {
    send(json!({"kind":"process","group":group,"add":add}))
}
pub fn lab(name: &str, dir: &std::path::Path, port: u16, add: bool) -> Result<()> {
    send(json!({"kind":"lab","name":name,"dir":dir,"port":port,"add":add}))
}
pub fn finish() {
    if let Some(guard) = GUARDIAN.get() {
        if let Ok(mut guard) = guard.lock() {
            guard.pipe.take();
            let _ = guard.child.wait();
        }
    }
}
fn bounded(mut command: Command, timeout: Duration) {
    command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let Ok(mut child) = command.spawn() else { return };
    let deadline = Instant::now() + timeout;
    loop {
        if child.try_wait().ok().flatten().is_some() {
            return;
        }
        if Instant::now() > deadline {
            #[cfg(unix)]
            let _ = signal_group(child.id(), "KILL");
            let _ = child.kill();
            let _ = child.wait();
            return;
        }
        std::thread::sleep(Duration::from_millis(30));
    }
}
pub fn guardian_main() {
    let mut groups = BTreeSet::new();
    let mut labs = BTreeMap::new();
    for line in std::io::stdin().lock().lines().map_while(std::result::Result::ok) {
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let add = value["add"].as_bool().unwrap_or(false);
        if value["kind"] == "process" {
            if let Some(group) = value["group"].as_u64().filter(|n| *n >= 2 && *n <= u32::MAX as u64) {
                if add {
                    groups.insert(group as u32);
                } else {
                    groups.remove(&(group as u32));
                }
            }
        } else if value["kind"] == "lab" {
            if let Some(name) = value["name"].as_str().filter(|s| s.starts_with("oe-lab-wp6-")) {
                if add {
                    labs.insert(name.to_owned(), value);
                } else {
                    labs.remove(name);
                }
            }
        }
    }
    for group in &groups {
        let _ = signal_group(*group, "CONT");
        let _ = signal_group(*group, "TERM");
    }
    if !groups.is_empty() {
        std::thread::sleep(Duration::from_millis(600));
    }
    for group in groups {
        let _ = signal_group(group, "KILL");
    }
    for (name, value) in labs {
        if let (Some(dir), Some(port)) = (value["dir"].as_str(), value["port"].as_u64()) {
            let mut down = Command::new("bash");
            down.arg(std::path::Path::new(dir).join("lab.sh"))
                .arg("down")
                .env("LAB_NAME", &name)
                .env("LAB_PORT_RANGE", format!("{port}-{port}"));
            bounded(down, Duration::from_secs(10));
            // The script checks the exact owning directory before any removal.
            // Never fall through to an unchecked `docker rm` after down fails.
            let mut rm = Command::new("bash");
            rm.arg(std::path::Path::new(dir).join("lab.sh"))
                .arg("remove")
                .env("LAB_NAME", &name)
                .env("LAB_PORT_RANGE", format!("{port}-{port}"));
            bounded(rm, Duration::from_secs(5));
        }
    }
}
