//! End-to-end transport CLI: real child peers, mutual TLS, live control frames
//! and real offline trust commands across retained/reconfigured directories.
//! No input, clipboard, native backend, mock, or production endpoint is used.
use openergy_net::{
    Fingerprint, NetCommand, NetConfig, NetDriver, NetEvent, NetHandle, Priority, TransportMode, TrustStore,
};
use openergy_session::ConnId;
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    net::{SocketAddr, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn check(value: bool, message: &str) -> Result<()> {
    if value {
        Ok(())
    } else {
        Err(message.to_owned().into())
    }
}
fn cli(binary: &Path, dir: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new(binary)
        .args(args)
        .arg("--dir")
        .arg(dir)
        .env("OPENERGY_CONFIG_DIR", dir)
        .env("OPENERGY_TLS_DIR", dir.join("tls"))
        .env("HOSTNAME", "openergy-directory-lab")
        .env("OPENERGY_DISABLE_DISCOVERY", "1")
        .env("OPENERGY_ALLOW_PORT_24800", "0")
        .output()?;
    check(
        output.status.success(),
        &format!("offline command failed: {}", String::from_utf8_lossy(&output.stderr)),
    )?;
    Ok(String::from_utf8(output.stdout)?)
}
fn identity(binary: &Path, dir: &Path) -> Result<Fingerprint> {
    let output = cli(binary, dir, &["fingerprint"])?;
    Fingerprint::parse(output.lines().next().ok_or("missing fingerprint")?).ok_or("bad fingerprint".into())
}
fn approve(binary: &Path, dir: &Path, fp: Fingerprint) -> Result<()> {
    cli(binary, dir, &["trust", "approve", &fp.to_colon_hex()])?;
    Ok(())
}
fn config(dir: &Path) -> NetConfig {
    NetConfig {
        tls_dir: dir.join("tls"),
        identity_name: "directory-server".into(),
        held_timeout: Duration::from_secs(120),
        ..NetConfig::default()
    }
}
struct Peer {
    child: Child,
    output: PathBuf,
}
impl Drop for Peer {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
struct Rig {
    driver: NetDriver,
    handle: NetHandle,
    events: Vec<NetEvent>,
    active: HashMap<Fingerprint, ConnId>,
    held: HashMap<Fingerprint, ConnId>,
    greeted: HashSet<ConnId>,
    disconnected: HashSet<ConnId>,
    acknowledgements: HashMap<ConnId, usize>,
    errors: Vec<Option<ConnId>>,
    peers: Vec<Peer>,
    peer_binary: PathBuf,
    root: PathBuf,
    addr: SocketAddr,
    listening: bool,
}
impl Rig {
    fn new(root: &Path, dir: &Path, port: u16) -> Result<Self> {
        let (mut driver, handle) = NetDriver::new(config(dir))?;
        driver.set_transport_mode(TransportMode::Frames)?;
        let mut rig = Self {
            driver,
            handle,
            events: Vec::new(),
            active: HashMap::new(),
            held: HashMap::new(),
            greeted: HashSet::new(),
            disconnected: HashSet::new(),
            acknowledgements: HashMap::new(),
            errors: Vec::new(),
            peers: Vec::new(),
            peer_binary: std::env::current_exe()?.with_file_name("tls_peer"),
            root: root.to_owned(),
            addr: ([127, 0, 0, 1], port).into(),
            listening: false,
        };
        rig.driver
            .execute(NetCommand::Listen { addr: rig.addr }, &mut rig.events);
        rig.drain()?;
        check(rig.listening, "listener did not bind the assigned endpoint")?;
        Ok(rig)
    }
    fn drain(&mut self) -> Result<()> {
        for event in std::mem::take(&mut self.events) {
            match event {
                NetEvent::Listening { addr } => self.listening = addr == self.addr,
                NetEvent::Connected {
                    conn, fingerprint, tls, ..
                } => {
                    check(tls, "a peer used plaintext")?;
                    let fp = fingerprint.ok_or("TLS peer omitted fingerprint")?;
                    self.active.insert(fp, conn);
                    self.held.remove(&fp);
                    self.disconnected.remove(&conn);
                    self.handle.send(NetCommand::SendFrame {
                        conn,
                        payload: b"Synergy\x00\x01\x00\x08".to_vec(),
                        priority: Priority::Control,
                    })?;
                }
                NetEvent::UntrustedPeer { conn, fingerprint, .. } => {
                    self.held.insert(fingerprint, conn);
                }
                NetEvent::Frame { conn, payload } if payload == b"CNOP" => {
                    *self.acknowledgements.entry(conn).or_default() += 1;
                }
                NetEvent::Frame { conn, payload } if payload.starts_with(b"Synergy") => {
                    self.greeted.insert(conn);
                }
                NetEvent::Frame { .. } => return Err("unexpected control frame".into()),
                NetEvent::Disconnected { conn, .. } => {
                    self.active.retain(|_, id| *id != conn);
                    self.held.retain(|_, id| *id != conn);
                    self.greeted.remove(&conn);
                    self.acknowledgements.remove(&conn);
                    self.disconnected.insert(conn);
                }
                NetEvent::Error { conn, .. } => self.errors.push(conn),
                NetEvent::ListenFailed { error, .. } | NetEvent::ConnectFailed { error, .. } => {
                    return Err(error.into())
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn pump(&mut self) -> Result<()> {
        self.driver.poll(Some(Duration::from_millis(5)), &mut self.events)?;
        self.drain()
    }
    fn until(&mut self, condition: impl Fn(&Self) -> bool) -> Result<()> {
        let deadline = Instant::now() + Duration::from_secs(3);
        while !condition(self) && Instant::now() < deadline {
            self.pump()?;
        }
        check(condition(self), "transport state timed out")
    }
    fn spawn(&mut self, dir: &Path) -> Result<usize> {
        let index = self.peers.len();
        let output = self.root.join(format!("peer-{index}.stdout"));
        let child = Command::new(&self.peer_binary)
            .args(["client", "--dir"])
            .arg(dir)
            .args([
                "--addr",
                &self.addr.to_string(),
                "--seconds",
                "60",
                "--hold-ms",
                "120000",
                "--echo-control",
            ])
            .env("OPENERGY_ALLOW_PORT_24800", "0")
            .env("OPENERGY_DISABLE_DISCOVERY", "1")
            .stdout(Stdio::from(File::create(&output)?))
            .stderr(Stdio::from(File::create(
                self.root.join(format!("peer-{index}.stderr")),
            )?))
            .spawn()?;
        self.peers.push(Peer { child, output });
        Ok(index)
    }
    fn connected(&mut self, fp: Fingerprint) -> Result<ConnId> {
        self.until(|r| r.active.get(&fp).is_some_and(|id| r.greeted.contains(id)))?;
        Ok(self.active[&fp])
    }
    fn exchange(&mut self, id: ConnId) -> Result<()> {
        check(
            self.driver.conn_info(id).is_some_and(|info| info.certificate_verified),
            "survivor lost directory-specific verification",
        )?;
        let before = self.acknowledgements.get(&id).copied().unwrap_or(0);
        self.handle.send(NetCommand::SendFrame {
            conn: id,
            payload: b"CNOP".to_vec(),
            priority: Priority::Control,
        })?;
        self.until(|r| r.acknowledgements.get(&id).copied().unwrap_or(0) > before)
    }
    fn reload(&mut self) -> Result<()> {
        self.handle.send(NetCommand::ReloadTrust)?;
        self.driver.poll(Some(Duration::ZERO), &mut self.events)?;
        self.drain()
    }
    fn peer_exited(&mut self, index: usize) -> Result<()> {
        let deadline = Instant::now() + Duration::from_secs(2);
        while self.peers[index].child.try_wait()?.is_none() && Instant::now() < deadline {
            self.pump()?;
        }
        let status = self.peers[index]
            .child
            .try_wait()?
            .ok_or("child did not observe disconnect")?;
        check(status.success(), "child peer failed")?;
        check(
            fs::read_to_string(&self.peers[index].output)?.contains("DISCONNECTED"),
            "child did not report real stream closure",
        )
    }
    fn finish(mut self) -> Result<()> {
        self.handle.send(NetCommand::Shutdown)?;
        self.until(|r| r.active.is_empty() && r.held.is_empty())?;
        for i in 0..self.peers.len() {
            self.peer_exited(i)?;
        }
        check(
            TcpStream::connect_timeout(&self.addr, Duration::from_millis(100)).is_err(),
            "owned listener remained open",
        )
    }
}

fn scenario(root: &Path, port: u16, binary: &Path, kind: &str) -> Result<()> {
    fs::create_dir(root)?;
    let a = root.join("server-a");
    let b = root.join("server-b");
    let clients: Vec<_> = (0..3).map(|i| root.join(format!("client-{i}"))).collect();
    let a_fp = identity(binary, &a)?;
    let b_fp = identity(binary, &b)?;
    let fps: Vec<_> = clients.iter().map(|dir| identity(binary, dir)).collect::<Result<_>>()?;
    for (i, dir) in clients.iter().enumerate() {
        approve(binary, dir, if i < 2 { a_fp } else { b_fp })?;
        approve(binary, if i < 2 { &a } else { &b }, fps[i])?;
    }
    let mut rig = Rig::new(root, &a, port)?;
    let waker = rig.driver.waker();
    let p0 = rig.spawn(&clients[0])?;
    let c0 = rig.connected(fps[0])?;
    rig.exchange(c0)?;
    let b_file = b.join("tls/trusted-clients");
    let b_good = fs::read(&b_file)?;
    if kind == "prepare_refusal" {
        fs::write(&b_file, b"\xff")?;
        check(
            rig.driver.reconfigure(config(&b)).is_err(),
            "invalid target directory was accepted",
        )?;
        check(
            rig.driver.identity_fingerprint() == Some(a_fp),
            "failed preparation changed identity",
        )?;
        rig.reload()?;
        rig.exchange(c0)?;
        fs::write(&b_file, &b_good)?;
    }
    let p1 = rig.spawn(&clients[1])?;
    let c1 = rig.connected(fps[1])?;
    rig.exchange(c1)?;
    rig.driver.reconfigure(config(&b))?;
    check(
        Arc::ptr_eq(&waker, &rig.driver.waker()) && rig.driver.identity_fingerprint() == Some(b_fp),
        "reconfiguration changed waker or retained wrong identity",
    )?;
    let p2 = rig.spawn(&clients[2])?;
    let c2 = rig.connected(fps[2])?;
    rig.reload()?;
    for id in [c0, c1, c2] {
        rig.exchange(id)?;
    }
    check(rig.errors.is_empty(), "valid directory switch emitted transport errors")?;
    let old_file = a.join("tls/trusted-clients");
    let a_good = fs::read(&old_file)?;
    match kind {
        "retained_revoke_and_admission" => {
            cli(binary, &a, &["trust", "revoke", &fps[0].to_colon_hex()])?;
            rig.reload()?;
            check(
                rig.disconnected.contains(&c0) && rig.driver.conn_info(c0).is_none(),
                "retained-directory revoke did not close its peer",
            )?;
            rig.peer_exited(p0)?;
            rig.exchange(c1)?;
            rig.exchange(c2)?;
            approve(binary, &a, fps[0])?;
            approve(binary, &clients[0], b_fp)?;
            rig.spawn(&clients[0])?;
            rig.until(|r| r.held.contains_key(&fps[0]))?;
            TrustStore::load(&b.join("tls"))?.trust_server(fps[0])?;
            rig.reload()?;
            check(
                rig.held.contains_key(&fps[0]) && !rig.active.contains_key(&fps[0]),
                "old-directory or wrong-role pin admitted a new peer",
            )?;
            approve(binary, &b, fps[0])?;
            rig.reload()?;
            let replacement = rig.connected(fps[0])?;
            for id in [replacement, c1, c2] {
                rig.exchange(id)?;
            }
        }
        "current_read_failure" => {
            fs::write(&b_file, b"\xff")?;
            rig.reload()?;
            check(
                rig.disconnected.contains(&c2) && !rig.disconnected.contains(&c0) && !rig.disconnected.contains(&c1),
                "current-directory failure affected wrong peers",
            )?;
            check(
                rig.errors.contains(&None)
                    && rig.errors.contains(&Some(c2))
                    && !rig.errors.contains(&Some(c0))
                    && !rig.errors.contains(&Some(c1)),
                "read error used wrong directory scope",
            )?;
            rig.peer_exited(p2)?;
            rig.exchange(c0)?;
            rig.exchange(c1)?;
            fs::write(&b_file, &b_good)?;
            rig.spawn(&clients[2])?;
            let replacement = rig.connected(fps[2])?;
            rig.reload()?;
            for id in [c0, c1, replacement] {
                rig.exchange(id)?;
            }
        }
        "retained_read_failure" => {
            fs::write(&old_file, b"\xff")?;
            rig.reload()?;
            check(
                rig.disconnected.contains(&c0) && rig.disconnected.contains(&c1) && !rig.disconnected.contains(&c2),
                "retained-directory failure affected wrong peers",
            )?;
            check(
                !rig.errors.contains(&None)
                    && rig.errors.contains(&Some(c0))
                    && rig.errors.contains(&Some(c1))
                    && !rig.errors.contains(&Some(c2)),
                "retained-directory errors used wrong scope",
            )?;
            rig.peer_exited(p0)?;
            rig.peer_exited(p1)?;
            rig.exchange(c2)?;
            fs::write(&old_file, &a_good)?;
            rig.driver.reconfigure(config(&a))?;
            check(
                Arc::ptr_eq(&waker, &rig.driver.waker()),
                "reverse reconfiguration changed waker",
            )?;
            rig.reload()?;
            rig.exchange(c2)?;
            rig.spawn(&clients[0])?;
            rig.spawn(&clients[1])?;
            let a0 = rig.connected(fps[0])?;
            let a1 = rig.connected(fps[1])?;
            for id in [a0, a1, c2] {
                rig.exchange(id)?;
            }
        }
        "prepare_refusal" => {}
        _ => return Err("unknown scenario".into()),
    }
    rig.finish()?;
    println!("CASE {{\"case\":\"{kind}\",\"passed\":true,\"mutual_tls\":true,\"bidirectional_frames\":true,\"same_driver_waker_handle\":true,\"child_and_listener_cleanup\":true}}");
    Ok(())
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    check(
        args.len() == 3,
        "usage: trust_directories EMPTY_PRIVATE_DIR FIRST_PORT OPENERGY_BIN",
    )?;
    let root = PathBuf::from(&args[0]);
    let port: u16 = args[1].to_str().ok_or("port is not text")?.parse()?;
    check((25080..=25096).contains(&port), "first port must be in 25080..25096")?;
    check(
        fs::symlink_metadata(&root)?.file_type().is_dir() && fs::read_dir(&root)?.next().is_none(),
        "root must be an existing empty real directory",
    )?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        check(
            fs::metadata(&root)?.permissions().mode() & 0o077 == 0,
            "root must deny group/other access",
        )?;
    }
    let binary = fs::canonicalize(&args[2])?;
    for (i, kind) in [
        "retained_revoke_and_admission",
        "current_read_failure",
        "retained_read_failure",
        "prepare_refusal",
    ]
    .iter()
    .enumerate()
    {
        scenario(&root.join(kind), port + i as u16, &binary, kind).map_err(|error| format!("{kind}: {error}"))?;
    }
    println!("PASS 4/4 real transport directory scenarios");
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("trust_directories: {error}");
        std::process::exit(1);
    }
}
