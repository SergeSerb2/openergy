use crate::model::{Failure, Node, Options, Result, Scenario, ScenarioReport, Step};
use crate::process::{command_output, Process};
#[cfg(unix)]
use crate::util::matches;
use crate::util::{self, lookup, merge, private_dir, summary, write_private};
use crate::wire::Raw;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

pub struct State {
    pub spec: Node,
    pub dir: PathBuf,
    pub socket: PathBuf,
    pub config: PathBuf,
    pub port: u16,
    pub reservation: Option<TcpListener>,
    pub process: Option<Process>,
    pub raw: Option<Raw>,
    pub lab: Option<String>,
    pub pid: Option<u32>,
}
pub struct Context {
    pub options: Options,
    pub scenario: Scenario,
    pub dir: PathBuf,
    pub nodes: BTreeMap<String, State>,
    pub vars: BTreeMap<String, Value>,
    pub measurements: Vec<Value>,
    pub commands: usize,
    #[cfg(unix)]
    subscriptions: BTreeMap<String, UnixStream>,
}
impl Context {
    pub fn new(options: &Options, scenario: Scenario, dir: PathBuf) -> Result<Self> {
        private_dir(&dir)?;
        let mut context = Self {
            options: options.clone(),
            scenario,
            dir,
            nodes: BTreeMap::new(),
            vars: BTreeMap::new(),
            measurements: Vec::new(),
            commands: 0,
            #[cfg(unix)]
            subscriptions: BTreeMap::new(),
        };
        let mut ports = Vec::new();
        context.vars.insert("dir".into(), json!(context.dir));
        context
            .vars
            .insert("fixtures".into(), json!(context.dir.join("fixtures")));
        for spec in context.scenario.nodes.clone() {
            if !safe_id(&spec.id) || context.nodes.contains_key(&spec.id) {
                return Err(Failure::runner("node IDs must be unique simple ASCII names"));
            }
            if !["openergy", "lab", "raw"].contains(&spec.kind.as_str()) {
                return Err(Failure::runner(
                    "only openergy, lab and raw nodes are allowed; native synergy-core is forbidden",
                ));
            }
            if spec.kind == "openergy"
                && (spec.platform != "virtual" || !["off", "server", "client"].contains(&spec.role.as_str()))
            {
                return Err(Failure::runner(
                    "Openergy nodes must use virtual and role off/server/client",
                ));
            }
            if spec.kind == "lab" && !matches!(spec.peer.as_deref(), Some("deskflow" | "barrier")) {
                return Err(Failure::runner("lab peer must be deskflow or barrier"));
            }
            let range = if spec.kind == "lab" {
                options.lab_range
            } else {
                options.port_range
            };
            let reservation = util::reserve(range, &ports, spec.port)?;
            let port = reservation.local_addr()?.port();
            ports.push(port);
            let node_dir = context.dir.join(&spec.id);
            private_dir(&node_dir)?;
            let socket = node_dir.join("ctl.sock");
            if socket.as_os_str().len() > 100 {
                return Err(Failure::runner(
                    "scenario Unix socket path exceeds 100 bytes; use a shorter TMPDIR",
                ));
            }
            let config = node_dir.join("config.toml");
            for (key, value) in [
                ("port", json!(port)),
                ("dir", json!(node_dir)),
                ("config", json!(config)),
                ("socket", json!(socket)),
                ("name", json!(spec.name.as_deref().unwrap_or(&spec.id))),
            ] {
                context.vars.insert(format!("{}.{key}", spec.id), value);
            }
            context.nodes.insert(
                spec.id.clone(),
                State {
                    spec,
                    dir: node_dir,
                    socket,
                    config,
                    port,
                    reservation: Some(reservation),
                    process: None,
                    raw: None,
                    lab: None,
                    pid: None,
                },
            );
        }
        context.copy_fixtures()?;
        let ids: Vec<_> = context.scenario.nodes.iter().map(|n| n.id.clone()).collect();
        for id in &ids {
            context.prepare(id)?;
        }
        Ok(context)
    }
    fn copy_fixtures(&self) -> Result<()> {
        let source = self.options.root.join("tools/e2e/fixtures");
        copy_tree(&source, &self.dir.join("fixtures"))?;
        Ok(())
    }
    pub fn expand(&self, value: &Value) -> Result<Value> {
        match value {
            Value::String(text) => {
                if let Some(key) = text
                    .strip_prefix("${")
                    .and_then(|s| s.strip_suffix('}'))
                    .filter(|k| !k.contains("${") && !k.contains('}'))
                {
                    return self.variable(key);
                }
                let mut output = String::new();
                let mut rest = text.as_str();
                while let Some((prefix, tail)) = rest.split_once("${") {
                    output.push_str(prefix);
                    let (key, next) = tail
                        .split_once('}')
                        .ok_or_else(|| Failure::runner("unclosed variable"))?;
                    let v = self.variable(key)?;
                    output.push_str(&v.as_str().map(str::to_owned).unwrap_or_else(|| v.to_string()));
                    rest = next;
                }
                output.push_str(rest);
                Ok(json!(output))
            }
            Value::Array(items) => items
                .iter()
                .map(|v| self.expand(v))
                .collect::<Result<Vec<_>>>()
                .map(Value::Array),
            Value::Object(map) => map
                .iter()
                .map(|(k, v)| Ok((k.clone(), self.expand(v)?)))
                .collect::<Result<serde_json::Map<_, _>>>()
                .map(Value::Object),
            _ => Ok(value.clone()),
        }
    }
    fn variable(&self, key: &str) -> Result<Value> {
        if let Some(value) = self.vars.get(key) {
            return Ok(value.clone());
        }
        let mut pieces = key.splitn(2, '.');
        let head = pieces.next().unwrap_or("");
        let tail = pieces.next().unwrap_or("");
        self.vars
            .get(head)
            .and_then(|v| lookup(v, tail))
            .cloned()
            .ok_or_else(|| Failure::runner(format!("undefined variable {key}")))
    }
    pub fn state(&self, id: &str) -> Result<&State> {
        self.nodes
            .get(id)
            .ok_or_else(|| Failure::runner(format!("unknown node {id}")))
    }
    pub fn state_mut(&mut self, id: &str) -> Result<&mut State> {
        self.nodes
            .get_mut(id)
            .ok_or_else(|| Failure::runner(format!("unknown node {id}")))
    }
    fn prepare(&mut self, id: &str) -> Result<()> {
        let state = self.state(id)?;
        if state.spec.kind == "openergy" {
            let mut cfg = json!({
                "version":1,"role":"off","name":"",
                "server":{"listen":"127.0.0.1","port":state.port,"heartbeat_ms":500},
                "client":{"remote":"","autoconnect":true,"heartbeat_ms":500},
                "tls":{"enabled":false,"require_client_trust":false},
                "clipboard":{"enabled":true,"send":true,"receive":true,"max_bytes":67108864},
                "input":{"relative_mouse":false,"invert_scroll_y":false,"scroll_sensitivity":1.0,"sync_language":true,"prevent_sleep":false},
                "switch":{"delay_ms":0,"double_tap_ms":0,"dead_corners":[],"corner_size_px":0},
                "hotkeys":{},"macos":{"click_numbering":"native"},"screen":[]
            });
            cfg["role"] = json!(state.spec.role);
            cfg["name"] = json!(state.spec.name.as_deref().unwrap_or(id));
            cfg["tls"]["enabled"] = json!(state.spec.tls);
            cfg["tls"]["require_client_trust"] = json!(state.spec.require_client_trust);
            cfg["server"]["listen"] = json!(format!("127.0.0.1:{}", state.port));
            cfg["server"]["port"] = json!(state.port);
            cfg["server"]["heartbeat_ms"] = json!(500);
            cfg["client"]["heartbeat_ms"] = json!(500);
            if let Some(remote) = &state.spec.remote {
                cfg["client"]["remote"] = json!(self.remote(remote, false)?);
            }
            let mut names = std::collections::BTreeSet::new();
            cfg["screen"] = Value::Array(self.scenario.nodes.iter().filter(|n| n.kind != "raw").filter(|n| names.insert(n.name.as_deref().unwrap_or(&n.id))).enumerate().map(|(index, n)| {
                json!({"name":n.name.as_deref().unwrap_or(&n.id),"rect":{"x":index*1920,"y":0,"w":1920,"h":1080}})
            }).collect());
            merge(&mut cfg, &self.expand(&serde_json::to_value(&state.spec.config)?)?);
            self.safe_config(&cfg)?;
            let text = toml::to_string_pretty(&cfg).map_err(|e| Failure::runner(e.to_string()))?;
            write_private(&state.config, text.as_bytes())?;
            private_dir(&state.dir.join("tls"))?;
            for (file, pins) in &state.spec.trust {
                if !["trusted-servers", "trusted-clients"].contains(&file.as_str()) {
                    return Err(Failure::runner(
                        "trust seed must use trusted-servers or trusted-clients",
                    ));
                }
                write_private(&state.dir.join("tls").join(file), pins.join("\n").as_bytes())?;
            }
        } else if state.spec.kind == "lab" {
            for file in ["lab.sh", "container.sh", "session.sh"] {
                std::fs::copy(
                    self.options.root.join("tools/interop-lab").join(file),
                    state.dir.join(file),
                )?;
            }
        }
        Ok(())
    }
    pub fn safe_config(&self, value: &Value) -> Result<()> {
        if let Some(server) = value.get("server") {
            if !server.is_object()
                || server.get("port").is_some_and(Value::is_null)
                || server.get("listen").is_some_and(Value::is_null)
            {
                return Err(Failure::runner(
                    "network settings cannot be removed or reset to production defaults",
                ));
            }
            if let Some(port) = server.get("port").and_then(Value::as_u64) {
                if port > u16::MAX as u64
                    || !(self.options.port_range.0 as u64..=self.options.port_range.1 as u64).contains(&port)
                    || port == 24800
                {
                    return Err(Failure::runner("configuration port is outside --port-range"));
                }
            }
            if let Some(address) = server.get("listen").and_then(Value::as_str) {
                if address != "127.0.0.1" {
                    self.safe_address(address, false)?;
                }
            }
        }
        if let Some(remote) = value
            .pointer("/client/remote")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
        {
            self.safe_address(remote, true)?;
        }
        Ok(())
    }
    fn safe_full_config(&self, value: &Value) -> Result<()> {
        self.safe_config(value)?;
        if value.pointer("/server/port").and_then(Value::as_u64).is_none()
            || value.pointer("/server/listen").and_then(Value::as_str).is_none()
        {
            return Err(Failure::runner(
                "full configurations must explicitly set loopback listen and an assigned port",
            ));
        }
        if value.get("role").and_then(Value::as_str) == Some("client")
            && value
                .pointer("/client/remote")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .is_none()
        {
            return Err(Failure::runner(
                "client configurations require an explicit loopback remote",
            ));
        }
        Ok(())
    }
    pub fn safe_address(&self, address: &str, labs: bool) -> Result<SocketAddr> {
        let parsed = address
            .parse::<SocketAddr>()
            .map_err(|_| Failure::runner("network addresses must be numeric 127.0.0.1:PORT"))?;
        let port = parsed.port();
        let allowed = (self.options.port_range.0..=self.options.port_range.1).contains(&port)
            || (labs && (self.options.lab_range.0..=self.options.lab_range.1).contains(&port));
        if parsed.ip() != Ipv4Addr::LOCALHOST || port == 24800 || !allowed {
            return Err(Failure::runner("refused non-loopback or unassigned network address"));
        }
        Ok(parsed)
    }
    pub fn remote(&self, remote: &str, from_lab: bool) -> Result<String> {
        let (port, local) = if let Some(state) = self.nodes.get(remote) {
            (state.port, format!("127.0.0.1:{}", state.port))
        } else {
            let addr = self.safe_address(remote, true)?;
            (addr.port(), remote.to_owned())
        };
        Ok(if from_lab {
            format!("host.docker.internal:{port}")
        } else {
            local
        })
    }
    pub fn timeout(&self, step: &Step) -> Duration {
        self.scaled(step.number("timeout_ms", self.scenario.timeout_ms))
    }
    /// A scenario or step deadline, stretched by `--slowdown` for slower machines.
    pub fn scaled(&self, ms: u64) -> Duration {
        Duration::from_millis(((ms as f64 * self.options.slowdown).round() as u64).clamp(1, 180_000))
    }
    pub fn start(&mut self, id: &str) -> Result<()> {
        let spec = self.state(id)?.spec.clone();
        if self.state(id)?.process.is_some() || self.state(id)?.raw.is_some() || self.state(id)?.lab.is_some() {
            return Err(Failure::runner(format!("node {id} is already started")));
        }
        if spec.kind == "raw" && spec.role == "server" {
            let port = self.state(id)?.port;
            let listener = match self.state_mut(id)?.reservation.take() {
                Some(listener) => listener,
                None => TcpListener::bind((Ipv4Addr::LOCALHOST, port))
                    .map_err(|e| Failure::environment(format!("raw listener bind: {e}")))?,
            };
            self.state_mut(id)?.raw = Some(Raw {
                listener: Some(listener),
                stream: None,
                pending: Default::default(),
            });
        } else if spec.kind == "raw" {
            self.state_mut(id)?.reservation.take();
            let remote = spec
                .remote
                .as_deref()
                .ok_or_else(|| Failure::runner("raw client requires remote"))?;
            let addr = self.safe_address(&self.remote(remote, false)?, true)?;
            let deadline = Instant::now() + self.scaled(self.scenario.timeout_ms);
            let stream = loop {
                match TcpStream::connect_timeout(&addr, Duration::from_millis(100)) {
                    Ok(stream) => break stream,
                    Err(_) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
                    Err(e) => return Err(Failure::product(format!("raw TCP connect failed: {e}"))),
                }
            };
            stream.set_nodelay(true)?;
            self.state_mut(id)?.raw = Some(Raw {
                listener: None,
                stream: Some(stream),
                pending: Default::default(),
            });
        } else if spec.kind == "openergy" {
            let text = std::fs::read_to_string(&self.state(id)?.config)?;
            let final_config: Value =
                toml::from_str(&text).map_err(|e| Failure::runner(format!("unsafe/unreadable startup config: {e}")))?;
            self.safe_full_config(&final_config)?;
            let state = self.state_mut(id)?;
            state.reservation.take();
            let (dir, config, socket) = (state.dir.clone(), state.config.clone(), state.socket.clone());
            let mut command = if spec.parent {
                let mut c = Command::new(std::env::current_exe()?);
                c.arg("__parent")
                    .arg(&self.options.binary)
                    .arg(&config)
                    .arg(&socket)
                    .arg(dir.join("daemon.pid"))
                    .arg(dir.join("parent.exit"))
                    .arg(id);
                c
            } else {
                let mut c = Command::new(&self.options.binary);
                c.args(["daemon", "--platform", "virtual", "--foreground", "--config"])
                    .arg(&config)
                    .arg("--socket")
                    .arg(&socket);
                c
            };
            isolated_env(&mut command, &dir, &socket);
            self.commands += 1;
            let child = Process::spawn(&mut command, &dir, &format!("daemon-{}", self.commands))?;
            self.state_mut(id)?.process = Some(child);
            let deadline = Instant::now() + self.scaled(self.scenario.timeout_ms);
            loop {
                if let Ok(status) = self.ipc(id, "status.get", json!({}), Duration::from_millis(200)) {
                    self.state_mut(id)?.pid = status.pointer("/daemon/pid").and_then(Value::as_u64).map(|v| v as u32);
                    break;
                }
                let exited = self
                    .state_mut(id)?
                    .process
                    .as_mut()
                    .map(Process::poll)
                    .transpose()?
                    .flatten();
                if let Some(status) = exited {
                    return Err(Failure::product(format!("{id} exited before IPC readiness: {status}")));
                }
                if Instant::now() >= deadline {
                    return Err(Failure::product(format!("{id} IPC readiness timed out")));
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        } else {
            let name = format!(
                "oe-lab-wp6-{}-{}-{}",
                std::process::id(),
                self.dir.file_name().unwrap_or_default().to_string_lossy(),
                id
            );
            self.state_mut(id)?.reservation.take();
            // Register before up: cleanup also covers a partially successful start.
            crate::guardian::lab(&name, &self.state(id)?.dir, self.state(id)?.port, true)?;
            self.state_mut(id)?.lab = Some(name);
            self.lab(id, &["up".into()])?;
            let state = self.state(id)?;
            let mut args = vec![spec.role.clone(), spec.peer.clone().unwrap_or_default()];
            if spec.role == "server" {
                args.extend([
                    "--port".into(),
                    state.port.to_string(),
                    "--screens".into(),
                    spec.screens.clone().unwrap_or_else(|| "lab,client".into()),
                ]);
            } else if spec.role == "client" {
                args.extend([
                    "--server".into(),
                    self.remote(
                        spec.remote
                            .as_deref()
                            .ok_or_else(|| Failure::runner("lab client needs remote"))?,
                        true,
                    )?,
                    "--name".into(),
                    spec.name.clone().unwrap_or_else(|| "lab".into()),
                ]);
            } else {
                return Err(Failure::runner("lab role must be server or client"));
            }
            args.extend(["--tls".into(), if spec.tls { "on" } else { "off" }.into()]);
            if let Some(protocol) = spec.protocol {
                args.extend(["--protocol".into(), protocol]);
            }
            if let Some(position) = spec.position {
                args.extend(["--position".into(), position]);
            }
            if let Some(pin) = spec.trust.get("fingerprint").and_then(|p| p.first()) {
                let pin = self.expand(&json!(pin))?;
                args.extend(["--trust-fingerprint".into(), pin.as_str().unwrap_or_default().into()]);
            }
            self.lab(id, &args)?;
        }
        Ok(())
    }
    pub fn ipc(&self, id: &str, method: &str, params: Value, timeout: Duration) -> Result<Value> {
        if method.starts_with("permissions.") {
            return Err(Failure::runner("native permission requests are forbidden"));
        }
        if method == "config.patch" {
            self.safe_config(params.get("patch").unwrap_or(&Value::Null))?;
        }
        if method == "config.set" {
            self.safe_full_config(params.get("config").unwrap_or(&params))?;
        }
        if method == "connection.connect" {
            if let Some(host) = params.get("host").and_then(Value::as_str) {
                let port = params.get("port").and_then(Value::as_u64).unwrap_or(24800);
                self.safe_address(&format!("{host}:{port}"), true)?;
            } else if params.get("port").is_some() {
                return Err(Failure::runner(
                    "connection.connect port requires explicit loopback host",
                ));
            }
        }
        let state = self.state(id)?;
        if state.spec.kind != "openergy" {
            return Err(Failure::runner("IPC requires an openergy node"));
        }
        let response = ipc_response(&state.socket, method, params, timeout)?;
        if let Some(error) = response.get("error").filter(|e| !e.is_null()) {
            let code = error.get("code").and_then(Value::as_str).unwrap_or("unknown");
            return Err(Failure::product(format!("{id} {method}: IPC error {code}")));
        }
        response
            .get("result")
            .cloned()
            .ok_or_else(|| Failure::product("IPC response has no result"))
    }
    pub fn lab(&mut self, id: &str, args: &[String]) -> Result<String> {
        let state = self.state(id)?;
        let name = state
            .lab
            .as_ref()
            .ok_or_else(|| Failure::runner("lab is not started"))?
            .clone();
        let dir = state.dir.clone();
        let port = state.port;
        let mut command = Command::new("bash");
        command
            .arg(dir.join("lab.sh"))
            .args(args)
            .env("LAB_NAME", name)
            .env("LAB_PORT_RANGE", format!("{port}-{port}"));
        self.commands += 1;
        let (code, output) = command_output(
            &mut command,
            &dir,
            &format!("lab-{}", self.commands),
            Duration::from_secs(30),
        )?;
        if code != 0 {
            return Err(Failure::environment(format!(
                "lab {} returned exit code {code}; see command stderr",
                args.first().map(String::as_str).unwrap_or("command")
            )));
        }
        Ok(output)
    }
    pub fn stop(&mut self, id: &str, graceful: bool) -> Result<()> {
        let kind = self.state(id)?.spec.kind.clone();
        if kind == "openergy" {
            if graceful && self.state(id)?.process.is_some() {
                self.ipc(id, "daemon.stop", json!({}), Duration::from_millis(500))?;
                let process = self
                    .state_mut(id)?
                    .process
                    .as_mut()
                    .ok_or_else(|| Failure::runner("missing process"))?;
                if !process.wait(Duration::from_secs(5))?.success() {
                    return Err(Failure::product("daemon.stop did not exit successfully"));
                }
            }
            if let Some(mut process) = self.state_mut(id)?.process.take() {
                process.terminate()?;
            }
        } else if kind == "lab" && self.state(id)?.lab.is_some() {
            self.lab(id, &["down".into()])?;
            util::sanitize_lab_logs(&self.state(id)?.dir)?;
            self.lab(id, &["remove".into()])?;
            if let Some(name) = self.state_mut(id)?.lab.take() {
                crate::guardian::lab(&name, &self.state(id)?.dir, self.state(id)?.port, false)?;
            }
        } else {
            self.state_mut(id)?.raw.take();
        }
        self.state_mut(id)?.reservation.take();
        Ok(())
    }
    pub fn raw(&mut self, id: &str) -> Result<&mut Raw> {
        self.state_mut(id)?
            .raw
            .as_mut()
            .ok_or_else(|| Failure::runner("raw node is not started"))
    }
    pub fn ledger(&self, id: &str, since: u64, timeout: Duration) -> Result<Vec<Value>> {
        self.ledger_checked(id, since, timeout, false)
    }
    pub fn ledger_checked(&self, id: &str, since: u64, timeout: Duration, complete: bool) -> Result<Vec<Value>> {
        let value = self.ipc(id, "virtual.recorded", json!({"since":since}), timeout)?;
        if complete
            && value
                .pointer("/stats/dropped")
                .and_then(Value::as_u64)
                .is_some_and(|n| n > 0)
            && value
                .pointer("/stats/oldest_sequence")
                .and_then(Value::as_u64)
                .is_none_or(|oldest| since < oldest.saturating_sub(1))
        {
            return Err(Failure::product(
                "ledger overflow prevents a complete empty/exact assertion",
            ));
        }
        value
            .as_array()
            .or_else(|| value.get("events").and_then(Value::as_array))
            .cloned()
            .ok_or_else(|| Failure::product("virtual.recorded must return an events array"))
    }
    pub fn confined(&self, path: &str) -> Result<PathBuf> {
        let path = PathBuf::from(path);
        if !path.starts_with(&self.dir) || path.components().any(|p| matches!(p, std::path::Component::ParentDir)) {
            return Err(Failure::runner(
                "artifact path must stay inside this scenario directory",
            ));
        }
        // Reject existing symlink ancestors, including the final component.
        let relative = path
            .strip_prefix(&self.dir)
            .map_err(|_| Failure::runner("invalid artifact path"))?;
        let mut checked = self.dir.clone();
        for component in relative.components() {
            checked.push(component);
            if std::fs::symlink_metadata(&checked).is_ok_and(|m| m.file_type().is_symlink()) {
                return Err(Failure::runner("artifact paths cannot traverse symlinks"));
            }
        }
        Ok(path)
    }
    pub fn snapshot(&self) {
        for (id, state) in &self.nodes {
            if state.spec.kind != "openergy" {
                continue;
            }
            if let Ok(status) = self.ipc(id, "status.get", json!({}), Duration::from_millis(150)) {
                let _ = write_private(
                    &state.dir.join("last-status.json"),
                    &serde_json::to_vec_pretty(&summary(&status)).unwrap_or_default(),
                );
            }
            if let Ok(ledger) = self.ledger(id, 0, Duration::from_millis(150)) {
                let _ = write_private(
                    &state.dir.join("ledger.json"),
                    &serde_json::to_vec_pretty(&summary(&json!(ledger))).unwrap_or_default(),
                );
            }
        }
    }
    pub fn cleanup(&mut self) -> Vec<Failure> {
        #[cfg(unix)]
        self.subscriptions.clear();
        let mut errors = Vec::new();
        let ids: Vec<_> = self.nodes.keys().cloned().collect();
        for id in ids {
            if let Err(error) = self.stop(&id, false) {
                errors.push(error);
            }
        }
        errors
    }
    #[cfg(unix)]
    pub fn subscribe(&mut self, id: &str, timeout: Duration) -> Result<()> {
        let mut stream = connect_ipc(&self.state(id)?.socket, timeout)?;
        let deadline = Instant::now() + timeout;
        write_ipc(
            &mut stream,
            &json!({"id":1,"method":"events.subscribe","params":{}}),
            deadline,
        )?;
        let response = read_ipc(&mut stream, deadline)?;
        if response.get("error").is_some_and(|v| !v.is_null()) {
            return Err(Failure::product("events.subscribe refused"));
        }
        self.subscriptions.insert(id.into(), stream);
        Ok(())
    }
    #[cfg(unix)]
    pub fn wait_event(&mut self, id: &str, step: &Step) -> Result<Value> {
        let timeout = self.timeout(step);
        let stream = self
            .subscriptions
            .get_mut(id)
            .ok_or_else(|| Failure::runner("subscribe before wait_event"))?;
        let deadline = Instant::now() + timeout;
        loop {
            let value = read_ipc(stream, deadline)?;
            if value.get("event").and_then(Value::as_str) == Some(step.text("event")?)
                && matches(
                    value.get("data").unwrap_or(&Value::Null),
                    &step.fields.get("expect").cloned().unwrap_or_else(|| json!({})),
                )
            {
                return Ok(value);
            }
            if Instant::now() > deadline {
                return Err(Failure::product("event deadline expired"));
            }
        }
    }
}
impl Drop for Context {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
fn safe_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 48 && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
fn copy_tree(source: &Path, dest: &Path) -> Result<()> {
    private_dir(dest)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &dest.join(entry.file_name()))?;
        } else if entry.file_type()?.is_file() {
            std::fs::copy(entry.path(), dest.join(entry.file_name()))?;
        } else {
            return Err(Failure::runner("fixtures cannot contain symlinks"));
        }
    }
    Ok(())
}
pub fn isolated_env(command: &mut Command, dir: &Path, socket: &Path) {
    command
        .env("OPENERGY_CONFIG", dir.join("config.toml"))
        .env("OPENERGY_CONFIG_DIR", dir)
        .env("OPENERGY_TLS_DIR", dir.join("tls"))
        .env("OPENERGY_SOCKET", socket)
        .env("XDG_CONFIG_HOME", dir.join("xdg-config"))
        .env("XDG_DATA_HOME", dir.join("xdg-data"))
        .env("XDG_RUNTIME_DIR", dir)
        .env("OPENERGY_DISABLE_DISCOVERY", "1");
}
#[cfg(unix)]
fn connect_ipc(socket: &Path, timeout: Duration) -> Result<UnixStream> {
    let stream = UnixStream::connect(socket).map_err(|e| Failure::product(format!("IPC connect: {e}")))?;
    let _ = timeout;
    stream.set_nonblocking(true)?;
    Ok(stream)
}
#[cfg(unix)]
pub fn ipc_response(socket: &Path, method: &str, params: Value, timeout: Duration) -> Result<Value> {
    let mut stream = connect_ipc(socket, timeout)?;
    let deadline = Instant::now() + timeout;
    let request = openergy_ipc::Request {
        id: 1,
        method: method.into(),
        params,
    };
    write_ipc(&mut stream, &serde_json::to_value(request)?, deadline)?;
    loop {
        let value = read_ipc(&mut stream, deadline)?;
        if value.get("id").and_then(Value::as_u64) == Some(1) {
            return Ok(value);
        }
        if Instant::now() > deadline {
            return Err(Failure::product("IPC response deadline expired"));
        }
    }
}
#[cfg(not(unix))]
pub fn ipc_response(_: &Path, _: &str, _: Value, _: Duration) -> Result<Value> {
    Err(Failure::environment("this runner requires Unix control sockets"))
}

pub fn run_scenario(options: &Options, scenario: Scenario, dir: PathBuf) -> ScenarioReport {
    let started = Instant::now();
    let mut report = ScenarioReport {
        scenario: scenario.name.clone(),
        requirement: scenario.requirement.clone(),
        outcome: "PASS",
        blocked_by: scenario.blocked_by.clone(),
        failed_step: None,
        elapsed_ms: 0,
        failure: None,
        cleanup_failures: Vec::new(),
        artifact_dir: None,
        stderr_tails: BTreeMap::new(),
        measurements: Vec::new(),
    };
    match Context::new(options, scenario, dir.clone()) {
        Ok(mut context) => {
            let result = (|| {
                for node in context.scenario.nodes.clone() {
                    if node.autostart {
                        context.start(&node.id)?;
                    }
                }
                for (index, original) in context.scenario.steps.clone().into_iter().enumerate() {
                    report.failed_step = Some(index + 1);
                    let expanded = context.expand(&serde_json::to_value(&original.fields)?)?;
                    let step = Step {
                        op: original.op,
                        fields: serde_json::from_value(expanded)?,
                    };
                    step.validate()?;
                    let value = context.execute(&step)?;
                    if let Some(name) = step.fields.get("save").and_then(Value::as_str) {
                        context.vars.insert(name.into(), value);
                    }
                }
                report.failed_step = None;
                Ok(())
            })();
            if result.is_err() || options.keep {
                context.snapshot();
            }
            report.measurements = context.measurements.clone();
            for state in context.nodes.values() {
                if let Some(child) = &state.process {
                    report
                        .stderr_tails
                        .insert(state.spec.id.clone(), util::tail(&child.stderr));
                }
            }
            let cleanup_errors = context.cleanup();
            report.cleanup_failures = cleanup_errors.clone();
            let result = result.and_then(|()| cleanup_errors.into_iter().next().map_or(Ok(()), Err));
            if let Err(error) = result {
                report.failure = Some(error);
            }
            if report.failure.is_some() {
                for node in context.nodes.values() {
                    if let Ok(entries) = std::fs::read_dir(&node.dir) {
                        for entry in entries
                            .flatten()
                            .filter(|e| e.path().extension().is_some_and(|e| e == "stderr"))
                        {
                            report
                                .stderr_tails
                                .entry(format!("{}/{}", node.spec.id, entry.file_name().to_string_lossy()))
                                .or_insert_with(|| util::tail(&entry.path()));
                        }
                    }
                }
            }
        }
        Err(error) => report.failure = Some(error),
    }
    if report.failure.is_some() || options.keep {
        report.artifact_dir = Some(dir.clone());
    } else {
        let _ = std::fs::remove_dir_all(&dir);
    }
    if report.failure.is_some() {
        report.outcome = "FAIL";
    }
    report.elapsed_ms = started.elapsed().as_millis();
    report
}

/// Run socket I/O in blocking mode, bounded per call by the absolute deadline.
/// Polling with short sleeps costs one timer wakeup per socket-buffer refill
/// (8 KiB on macOS); hosted VMs stretch those wakeups enough that an 11 MiB
/// clipboard fixture misses its deadline. The kernel wakes a blocked call as
/// soon as the peer makes progress, so this measures the daemon, not timers.
#[cfg(unix)]
fn blocking_io<T>(stream: &mut UnixStream, io: impl FnOnce(&mut UnixStream) -> Result<T>) -> Result<T> {
    stream.set_nonblocking(false)?;
    let result = io(stream);
    stream.set_nonblocking(true)?;
    result
}
#[cfg(unix)]
fn read_ipc(stream: &mut UnixStream, deadline: Instant) -> Result<Value> {
    fn exact(stream: &mut UnixStream, bytes: &mut [u8], deadline: Instant) -> Result<()> {
        use std::io::Read;
        let mut offset = 0;
        while offset < bytes.len() {
            // macOS rejects socket options with EINVAL once both directions are shut
            // down; the next call then reports EOF or EPIPE without blocking.
            let _ = stream.set_read_timeout(util::io_budget(deadline, "IPC read")?);
            match stream.read(&mut bytes[offset..]) {
                Ok(0) => return Err(Failure::product("IPC connection closed inside frame")),
                Ok(count) => offset += count,
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut | std::io::ErrorKind::Interrupted
                    ) => {}
                Err(e) => return Err(Failure::product(format!("IPC read: {e}"))),
            }
        }
        Ok(())
    }
    blocking_io(stream, |stream| {
        let mut header = [0; 4];
        exact(stream, &mut header, deadline)?;
        let len = u32::from_be_bytes(header) as usize;
        if len > 16 * 1024 * 1024 {
            return Err(Failure::product("IPC response exceeds frozen 16 MiB cap"));
        }
        let mut bytes = vec![0; len];
        exact(stream, &mut bytes, deadline)?;
        serde_json::from_slice(&bytes).map_err(|_| Failure::product("IPC response is invalid JSON"))
    })
}
#[cfg(unix)]
fn write_ipc(stream: &mut UnixStream, value: &Value, deadline: Instant) -> Result<()> {
    use std::io::Write;
    let bytes = serde_json::to_vec(value)?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(Failure::runner("request exceeds frozen 16 MiB IPC cap"));
    }
    let header = (bytes.len() as u32).to_be_bytes();
    blocking_io(stream, |stream| {
        for slice in [header.as_slice(), bytes.as_slice()] {
            let mut offset = 0;
            while offset < slice.len() {
                // macOS rejects socket options with EINVAL once both directions are shut
                // down; the next call then reports EOF or EPIPE without blocking.
                let _ = stream.set_write_timeout(util::io_budget(deadline, "IPC write")?);
                match stream.write(&slice[offset..]) {
                    Ok(0) => return Err(Failure::product("IPC write returned zero")),
                    Ok(count) => offset += count,
                    Err(e)
                        if matches!(
                            e.kind(),
                            std::io::ErrorKind::WouldBlock
                                | std::io::ErrorKind::TimedOut
                                | std::io::ErrorKind::Interrupted
                        ) => {}
                    Err(e) => return Err(Failure::product(format!("IPC write: {e}"))),
                }
            }
        }
        Ok(())
    })
}
