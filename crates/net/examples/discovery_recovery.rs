//! Real custom-port loopback multicast with a separate publisher process.
//! Exercise normal discovery, an overflowing upstream subscription, recovery,
//! TXT updates and goodbye removal. No native backend or production endpoint.
use mdns_sd::{DaemonStatus, IfKind, ServiceDaemon, ServiceInfo, UnregisterStatus};
use openergy_net::discovery::{Discovery, DiscoveryScope, SERVICE_TYPE};
use std::{
    fs::{self, File},
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const BURST: usize = 800;

fn retry<T>(mut f: impl FnMut() -> mdns_sd::Result<T>) -> Result<T> {
    let start = Instant::now();
    loop {
        match f() {
            Ok(value) => return Ok(value),
            Err(mdns_sd::Error::Again) if start.elapsed() < Duration::from_secs(5) => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => return Err(error.into()),
        }
    }
}
fn daemon(port: u16) -> Result<ServiceDaemon> {
    let daemon = ServiceDaemon::new_with_port(port)?;
    daemon.disable_interface(IfKind::All)?;
    daemon.enable_interface(IfKind::LoopbackV4)?;
    Ok(daemon)
}
fn register(daemon: &ServiceDaemon, instance: &str, name: &str, port: u16, fp: &str) -> Result<String> {
    let properties = [("name", name), ("ver", "1.8"), ("fp", fp)];
    let info = ServiceInfo::new(
        SERVICE_TYPE,
        instance,
        "openergy-recovery-lab.local.",
        "127.0.0.1",
        port,
        &properties[..],
    )?;
    let full = info.get_fullname().to_owned();
    retry(|| daemon.register(info.clone()))?;
    Ok(full)
}
fn unregister(daemon: &ServiceDaemon, names: &[String]) -> Result<()> {
    let mut replies = Vec::new();
    for name in names {
        replies.push(retry(|| daemon.unregister(name))?);
    }
    for reply in replies {
        if !matches!(reply.recv_timeout(Duration::from_secs(5))?, UnregisterStatus::OK) {
            return Err("publisher lost a registered service".into());
        }
    }
    Ok(())
}
fn stop(daemon: &ServiceDaemon) -> Result<()> {
    if retry(|| daemon.shutdown())?.recv_timeout(Duration::from_secs(3))? != DaemonStatus::Shutdown {
        return Err("mDNS worker did not acknowledge shutdown".into());
    }
    Ok(())
}
fn publisher(root: &Path, mdns_port: u16, service_port: u16) -> Result<()> {
    let daemon = daemon(mdns_port)?;
    let initial = register(&daemon, "initial", "initial", service_port, "0123456789abcdef")?;
    thread::sleep(Duration::from_secs(2));
    fs::write(root.join("ready-initial"), "ready")?;
    let mut burst = Vec::new();
    let mut recovery = None;
    for line in io::stdin().lock().lines() {
        let phase = line?;
        match phase.as_str() {
            "burst" => {
                for i in 0..BURST {
                    let name = format!("burst-{i:04}");
                    burst.push(register(&daemon, &name, &name, service_port, "0123456789abcdef")?);
                }
                thread::sleep(Duration::from_secs(6));
            }
            "clear" => {
                unregister(&daemon, &burst)?;
                burst.clear();
                thread::sleep(Duration::from_secs(2));
            }
            "recover" => {
                recovery = Some(register(
                    &daemon,
                    "recovery",
                    "recovery",
                    service_port,
                    "0123456789abcdef",
                )?);
                thread::sleep(Duration::from_secs(2));
            }
            "update" => {
                register(
                    &daemon,
                    "recovery",
                    "recovery-updated",
                    service_port + 1,
                    "invalid-hint",
                )?;
                thread::sleep(Duration::from_secs(2));
            }
            "finish" => {
                if !burst.is_empty() {
                    unregister(&daemon, &burst)?;
                }
                let mut names = vec![initial];
                names.extend(recovery);
                unregister(&daemon, &names)?;
                thread::sleep(Duration::from_secs(2));
                stop(&daemon)?;
                fs::write(root.join("ready-finish"), "ready")?;
                return Ok(());
            }
            _ => return Err("invalid publisher phase".into()),
        }
        fs::write(root.join(format!("ready-{phase}")), "ready")?;
    }
    stop(&daemon)
}

struct Publisher {
    child: Child,
    root: PathBuf,
}
impl Publisher {
    fn ready(&mut self, phase: &str) -> Result<()> {
        let start = Instant::now();
        while !self.root.join(format!("ready-{phase}")).is_file() {
            if self.child.try_wait()?.is_some() {
                return Err(format!("publisher exited during {phase}; inspect publisher.log").into());
            }
            if start.elapsed() > Duration::from_secs(20) {
                return Err(format!("publisher stalled during {phase}").into());
            }
            thread::sleep(Duration::from_millis(20));
        }
        Ok(())
    }
    fn phase(&mut self, phase: &str) -> Result<()> {
        writeln!(self.child.stdin.as_mut().ok_or("publisher stdin missing")?, "{phase}")?;
        self.ready(phase)
    }
}
impl Drop for Publisher {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}
fn observe(
    discovery: &mut Discovery,
    seconds: u64,
    mut accept: impl FnMut(&[openergy_net::DiscoveredPeer]) -> bool,
) -> bool {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(seconds) {
        discovery.poll();
        if accept(&discovery.snapshot()) {
            return true;
        }
        thread::sleep(Duration::from_millis(20));
    }
    false
}
fn probe(root: &Path, mdns_port: u16, service_port: u16) -> Result<()> {
    fs::create_dir(root)?;
    let mut discovery = Discovery::new(DiscoveryScope::Loopback, mdns_port)?;
    let sentinel_daemon = daemon(mdns_port)?;
    let sentinel = sentinel_daemon.browse(SERVICE_TYPE)?;
    let log = File::create(root.join("publisher.log"))?;
    let child = Command::new(std::env::current_exe()?)
        .arg("publisher")
        .arg(root)
        .args([mdns_port.to_string(), service_port.to_string()])
        .stdin(Stdio::piped())
        .stdout(log.try_clone()?)
        .stderr(log)
        .spawn()?;
    let mut publisher = Publisher {
        child,
        root: root.to_owned(),
    };
    publisher.ready("initial")?;
    let mut results = Vec::new();
    results.push((
        "initial-resolution",
        observe(&mut discovery, 5, |peers| peers.iter().any(|p| p.name == "initial")),
    ));
    publisher.phase("burst")?;
    results.push((
        "real-upstream-channel-overflow",
        sentinel.is_disconnected() && sentinel.len() == 1000,
    ));
    // Deliberately stop draining during the publisher's burst. This models a
    // delayed caller, using real network events rather than injected messages.
    let mut largest = 0;
    observe(&mut discovery, 2, |peers| {
        largest = largest.max(peers.len());
        false
    });
    results.push(("snapshot-cap", largest == 64));
    publisher.phase("clear")?;
    observe(&mut discovery, 2, |_| false);
    publisher.phase("recover")?;
    let start = Instant::now();
    results.push((
        "fresh-advertisement-after-overflow",
        observe(&mut discovery, 8, |peers| {
            peers.iter().any(|p| p.name == "initial")
                && peers.iter().any(|p| {
                    p.name == "recovery"
                        && p.port == service_port
                        && p.fingerprint_hint.as_deref() == Some("0123456789abcdef")
                })
        }),
    ));
    println!("RECOVERY_OBSERVATION_MS={}", start.elapsed().as_millis());
    publisher.phase("update")?;
    results.push((
        "updated-port-and-untrusted-hint",
        observe(&mut discovery, 5, |peers| {
            peers
                .iter()
                .any(|p| p.name == "recovery-updated" && p.port == service_port + 1 && p.fingerprint_hint.is_none())
        }),
    ));
    publisher.phase("finish")?;
    results.push((
        "goodbye-removes-all-peers",
        observe(&mut discovery, 5, |peers| peers.is_empty()),
    ));
    results.push(("publisher-exits-zero", publisher.child.wait()?.success()));
    discovery.shutdown()?;
    stop(&sentinel_daemon)?;
    for (name, passed) in &results {
        println!("CHECK {name}={passed}");
    }
    println!("BURST_SERVICES={BURST} SNAPSHOT_PEAK={largest}");
    println!("CHECK worker-shutdown=true");
    if results.iter().all(|(_, passed)| *passed) {
        Ok(())
    } else {
        Err("discovery recovery checks failed".into())
    }
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 4 || !matches!(args[0].as_str(), "probe" | "publisher") {
        return Err("discovery_recovery probe NEW_DIR MDNS_PORT SERVICE_PORT".into());
    }
    let mdns_port: u16 = args[2].parse()?;
    let service_port: u16 = args[3].parse()?;
    if !(25080..=25099).contains(&mdns_port)
        || !(25080..=25098).contains(&service_port)
        || mdns_port == service_port
        || mdns_port == service_port + 1
    {
        return Err("requires distinct assigned loopback ports 25080..=25099, including SERVICE_PORT+1".into());
    }
    if args[0] == "publisher" {
        publisher(Path::new(&args[1]), mdns_port, service_port)
    } else {
        probe(Path::new(&args[1]), mdns_port, service_port)
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("discovery_recovery: {error}");
        std::process::exit(1);
    }
}
