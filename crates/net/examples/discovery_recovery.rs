//! Real custom-port loopback multicast with a separate publisher process.
//! Exercise normal discovery, an overflowing upstream subscription, recovery,
//! TXT updates and goodbye removal, including selected peers while all 800
//! burst advertisements stay live. No native backend or production endpoint.
use mdns_sd::{DaemonStatus, IfKind, LimitedBrowse, ServiceDaemon, ServiceEvent, ServiceInfo, UnregisterStatus};
use openergy_net::discovery::{Discovery, DiscoveryScope, SERVICE_TYPE};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Mutex, OnceLock},
    thread,
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const BURST: usize = 800;
const GUARD_TYPE: &str = "_oe-guard._tcp.local.";

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
    let mut initial = Some(register(
        &daemon,
        "initial",
        "initial",
        service_port,
        "0123456789abcdef",
    )?);
    thread::sleep(Duration::from_secs(2));
    fs::write(root.join("ready-initial"), "ready")?;
    let mut burst = Vec::new();
    let mut recovery = None;
    let mut guard = None;
    let mut startup_services = BTreeSet::new();
    for line in io::stdin().lock().lines() {
        let phase = line?;
        let (phase_name, selected) = phase.split_once(' ').unwrap_or((&phase, ""));
        match phase_name {
            "advertise" => {
                let (ty, value) = selected.split_once(' ').ok_or("missing advertisement value")?;
                let properties = [("name", value)];
                let info = ServiceInfo::new(
                    ty,
                    "witness",
                    "openergy-startup-lab.local.",
                    "127.0.0.1",
                    service_port,
                    &properties[..],
                )?;
                startup_services.insert(info.get_fullname().to_owned());
                retry(|| daemon.register(info.clone()))?;
            }
            "withdraw" => {
                let full = format!("witness.{selected}");
                unregister(&daemon, std::slice::from_ref(&full))?;
                startup_services.remove(&full);
            }
            "guard-start" | "guard-update" | "guard-final" => {
                let properties = [("name", phase_name)];
                let info = ServiceInfo::new(
                    GUARD_TYPE,
                    "guard",
                    "openergy-guard-lab.local.",
                    "127.0.0.1",
                    service_port,
                    &properties[..],
                )?;
                guard = Some(info.get_fullname().to_owned());
                retry(|| daemon.register(info.clone()))?;
            }
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
            "retained-update" => {
                if !burst.iter().any(|full| full == &format!("{selected}.{SERVICE_TYPE}")) {
                    return Err("selected service is not in the live burst".into());
                }
                register(
                    &daemon,
                    selected,
                    &format!("{selected}-retained"),
                    service_port + 1,
                    "invalid-hint",
                )?;
            }
            "retained-remove" => {
                // Keep 800 burst advertisements live even while removing the
                // selected peer. Admission of the replacement is not required.
                burst.push(register(
                    &daemon,
                    "burst-replacement",
                    "burst-replacement",
                    service_port,
                    "0123456789abcdef",
                )?);
                let full = format!("{selected}.{SERVICE_TYPE}");
                unregister(&daemon, std::slice::from_ref(&full))?;
                burst.retain(|name| name != &full);
            }
            "retained-new" => {
                // Remove the first peer, then advertise a new peer after its
                // goodbye. The 800 burst advertisements remain registered.
                unregister(&daemon, &[initial.take().ok_or("initial already removed")?])?;
                thread::sleep(Duration::from_millis(1300));
                burst.push(register(
                    &daemon,
                    "after-removal",
                    "after-removal",
                    service_port,
                    "0123456789abcdef",
                )?);
            }
            "retained-paused-update" => {
                register(
                    &daemon,
                    "after-removal",
                    "paused-intermediate",
                    service_port + 1,
                    "invalid-hint",
                )?;
                thread::sleep(Duration::from_millis(1100));
                register(
                    &daemon,
                    "after-removal",
                    "paused-latest",
                    service_port + 1,
                    "invalid-hint",
                )?;
                thread::sleep(Duration::from_millis(1100));
            }
            "retained-paused-remove" => {
                let full = format!("after-removal.{SERVICE_TYPE}");
                unregister(&daemon, std::slice::from_ref(&full))?;
                burst.retain(|name| name != &full);
                thread::sleep(Duration::from_millis(1300));
            }
            "restore-initial" => {
                initial = Some(register(
                    &daemon,
                    "initial",
                    "initial",
                    service_port,
                    "0123456789abcdef",
                )?);
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
                let mut names: Vec<_> = initial.into_iter().collect();
                names.extend(recovery);
                names.extend(guard);
                names.extend(startup_services);
                unregister(&daemon, &names)?;
                thread::sleep(Duration::from_secs(2));
                stop(&daemon)?;
                fs::write(root.join("ready-finish"), "ready")?;
                return Ok(());
            }
            _ => return Err("invalid publisher phase".into()),
        }
        fs::write(root.join(format!("ready-{phase_name}")), "ready")?;
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
        self.begin(phase)?;
        self.ready(phase.split_once(' ').map_or(phase, |(name, _)| name))
    }
    fn begin(&mut self, phase: &str) -> Result<()> {
        let phase_name = phase.split_once(' ').map_or(phase, |(name, _)| name);
        let ready = self.root.join(format!("ready-{phase_name}"));
        if ready.exists() {
            fs::remove_file(ready)?;
        }
        writeln!(self.child.stdin.as_mut().ok_or("publisher stdin missing")?, "{phase}")?;
        Ok(())
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
#[derive(Default)]
struct Snapshots {
    last: Option<Instant>,
    count: usize,
    peak: usize,
    too_close: bool,
}
impl Snapshots {
    fn poll(&mut self, discovery: &mut Discovery) {
        if let Some(peers) = discovery.poll() {
            let now = Instant::now();
            self.too_close |= self
                .last
                .is_some_and(|last| now.duration_since(last) < Duration::from_secs(1));
            self.last = Some(now);
            self.count += 1;
            self.peak = self.peak.max(peers.len());
        }
        self.peak = self.peak.max(discovery.snapshot().len());
    }
}
fn observe(
    discovery: &mut Discovery,
    snapshots: &mut Snapshots,
    seconds: u64,
    mut accept: impl FnMut(&[openergy_net::DiscoveredPeer]) -> bool,
) -> bool {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(seconds) {
        snapshots.poll(discovery);
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
    let mut emissions = Snapshots::default();
    results.push((
        "initial-resolution",
        observe(&mut discovery, &mut emissions, 5, |peers| {
            peers.iter().any(|p| p.name == "initial")
        }),
    ));
    publisher.phase("burst")?;
    results.push((
        "real-upstream-channel-overflow",
        sentinel.is_disconnected() && sentinel.len() == 1000,
    ));
    let metrics = retry(|| sentinel_daemon.get_metrics())?.recv_timeout(Duration::from_secs(3))?;
    println!("RETAINED_CACHE_METRICS={metrics:?}");
    for attempt in 1..=2 {
        let replay = retry(|| sentinel_daemon.browse(SERVICE_TYPE))?;
        let mut drained = 0;
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(500) {
            thread::sleep(Duration::from_millis(20));
            drained += replay.try_iter().take(256).count();
            if replay.is_disconnected() && replay.is_empty() {
                break;
            }
        }
        println!(
            "RETAINED_REPLAY attempt={attempt} drained={drained} disconnected={} elapsed_ms={}",
            replay.is_disconnected(),
            start.elapsed().as_millis()
        );
    }
    retry(|| sentinel_daemon.stop_browse(SERVICE_TYPE))?;
    let after_stop = retry(|| sentinel_daemon.get_metrics())?.recv_timeout(Duration::from_secs(3))?;
    println!("RETAINED_CACHE_AFTER_STOP={after_stop:?}");
    let limited_replay = retry(|| sentinel_daemon.browse_limited(SERVICE_TYPE, 64))?;
    let start = Instant::now();
    let mut replay_peak = 0;
    while start.elapsed() < Duration::from_secs(1) {
        if let Some(peers) = limited_replay.take_snapshot() {
            replay_peak = replay_peak.max(peers.len());
        }
        thread::sleep(Duration::from_millis(20));
    }
    println!(
        "LIMITED_REPLAY peak={replay_peak} closed={}",
        limited_replay.is_closed()
    );
    results.push((
        "limited-retained-replay",
        replay_peak == 64 && !limited_replay.is_closed(),
    ));
    // Deliberately stop draining during the publisher's burst. This models a
    // delayed caller, using real network events rather than injected messages.
    let mut largest = 0;
    let mut selected = None;
    let mut empty_polls = 0;
    let mut full_polls = 0;
    let mut stable_selection = None;
    let mut selection_stays_stable = true;
    let retained_start = Instant::now();
    while retained_start.elapsed() < Duration::from_secs(6) {
        emissions.poll(&mut discovery);
        let peers = discovery.snapshot();
        largest = largest.max(peers.len());
        // Permit the original one-second recovery backoff. The retained
        // namespace must then remain useful under normal 20ms polling.
        if retained_start.elapsed() >= Duration::from_secs(2) {
            empty_polls += usize::from(peers.is_empty());
            full_polls += usize::from(peers.len() == 64);
            let names: BTreeSet<_> = peers.iter().map(|p| p.name.clone()).collect();
            if let Some(previous) = &stable_selection {
                selection_stays_stable &= *previous == names;
            } else {
                stable_selection = Some(names);
            }
        }
        thread::sleep(Duration::from_millis(20));
    }
    results.push(("retained-namespace-stays-useful", full_polls > 0 && empty_polls == 0));
    results.push(("retained-selection-stable", selection_stays_stable));
    // Choose from the current snapshot immediately before updating. A full
    // display need not admit a new peer, so mutate an actually selected one.
    observe(&mut discovery, &mut emissions, 2, |peers| {
        selected = peers
            .iter()
            .find(|p| p.name.starts_with("burst-"))
            .map(|p| p.name.clone());
        selected.is_some()
    });
    let selected = selected.ok_or("no selected burst peer")?;
    let survivors: BTreeSet<_> = discovery
        .snapshot()
        .iter()
        .filter(|p| p.name != selected && p.name != "initial")
        .map(|p| p.name.clone())
        .collect();
    let mut survivors_stable = true;
    publisher.begin(&format!("retained-update {selected}"))?;
    let updated_name = format!("{selected}-retained");
    let update_seen = observe(&mut discovery, &mut emissions, 8, |peers| {
        peers
            .iter()
            .any(|p| p.name == updated_name && p.port == service_port + 1 && p.fingerprint_hint.is_none())
    });
    publisher.ready("retained-update")?;
    let mut update_stays_fresh = update_seen;
    if update_seen {
        observe(&mut discovery, &mut emissions, 2, |peers| {
            update_stays_fresh &= peers
                .iter()
                .any(|p| p.name == updated_name && p.port == service_port + 1 && p.fingerprint_hint.is_none());
            false
        });
    }
    results.push(("retained-selected-peer-live-update", update_seen && update_stays_fresh));
    publisher.begin(&format!("retained-remove {selected}"))?;
    let mut removal_seen = false;
    let mut removal_regressed = false;
    observe(&mut discovery, &mut emissions, 5, |peers| {
        survivors_stable &= survivors.iter().all(|name| peers.iter().any(|p| &p.name == name));
        let present = peers.iter().any(|p| p.name == selected || p.name == updated_name);
        if removal_seen && present {
            removal_regressed = true;
        }
        removal_seen |= !present && !peers.is_empty();
        false
    });
    publisher.ready("retained-remove")?;
    results.push((
        "retained-selected-peer-goodbye",
        update_seen && update_stays_fresh && removal_seen && !removal_regressed,
    ));
    println!("RETAINED_NAMESPACE burst={BURST} selected={selected} full_polls={full_polls} empty_polls={empty_polls} update_seen={update_seen} update_stays_fresh={update_stays_fresh} removal_seen={removal_seen} removal_regressed={removal_regressed}");
    publisher.begin("retained-new")?;
    let new_peer_seen = observe(&mut discovery, &mut emissions, 6, |peers| {
        survivors_stable &= survivors.iter().all(|name| peers.iter().any(|p| &p.name == name));
        !peers.iter().any(|p| p.name == "initial") && peers.iter().any(|p| p.name == "after-removal")
    });
    publisher.ready("retained-new")?;
    println!(
        "AFTER_REMOVAL peers={:?}",
        discovery.snapshot().iter().map(|p| &p.name).collect::<Vec<_>>()
    );
    let new_metrics = retry(|| sentinel_daemon.get_metrics())?.recv_timeout(Duration::from_secs(3))?;
    println!("AFTER_REMOVAL_CACHE={new_metrics:?}");
    results.push(("retained-first-peer-goodbye-and-new-peer", new_peer_seen));
    observe(&mut discovery, &mut emissions, 2, |peers| {
        survivors_stable &= survivors.iter().all(|name| peers.iter().any(|p| &p.name == name));
        false
    });
    results.push(("retained-selected-survivors-stable", survivors_stable));
    // The caller does not poll while two real advertisements update one peer.
    publisher.phase("retained-paused-update")?;
    emissions.poll(&mut discovery);
    let paused_latest = discovery
        .snapshot()
        .iter()
        .any(|p| p.name == "paused-latest" && p.port == service_port + 1 && p.fingerprint_hint.is_none());
    results.push(("paused-update-coalesces-to-latest", paused_latest));
    publisher.phase("retained-paused-remove")?;
    emissions.poll(&mut discovery);
    let paused_removed = discovery
        .snapshot()
        .iter()
        .all(|p| p.name != "after-removal" && p.name != "paused-intermediate" && p.name != "paused-latest");
    results.push(("paused-goodbye-replaces-view", paused_latest && paused_removed));
    publisher.phase("clear")?;
    publisher.phase("restore-initial")?;
    observe(&mut discovery, &mut emissions, 2, |_| false);
    publisher.phase("recover")?;
    let start = Instant::now();
    results.push((
        "fresh-advertisement-after-overflow",
        observe(&mut discovery, &mut emissions, 8, |peers| {
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
        observe(&mut discovery, &mut emissions, 5, |peers| {
            peers
                .iter()
                .any(|p| p.name == "recovery-updated" && p.port == service_port + 1 && p.fingerprint_hint.is_none())
        }),
    ));
    publisher.phase("finish")?;
    results.push((
        "goodbye-removes-all-peers",
        observe(&mut discovery, &mut emissions, 5, |peers| peers.is_empty()),
    ));
    results.push(("publisher-exits-zero", publisher.child.wait()?.success()));
    results.push(("discovery-shutdown-submitted", discovery.shutdown().is_ok()));
    results.push(("sentinel-worker-shutdown-ack", stop(&sentinel_daemon).is_ok()));
    results.push(("snapshot-cap", emissions.peak == 64));
    results.push((
        "snapshot-emission-spacing",
        !emissions.too_close && emissions.count >= 4,
    ));
    println!(
        "SNAPSHOTS count={} peak={} too_close={}",
        emissions.count, emissions.peak, emissions.too_close
    );
    for (name, passed) in &results {
        println!("CHECK {name}={passed}");
    }
    println!("BURST_SERVICES={BURST} SNAPSHOT_PEAK={largest}");
    if results.iter().all(|(_, passed)| *passed) {
        Ok(())
    } else {
        Err("discovery recovery checks failed".into())
    }
}
// Public-API lifecycle probe. The replay workload is ordinary ServiceInfo
// traffic on a private loopback port; it does not inject commands or DNS bytes.
fn limited_states(daemon: &ServiceDaemon) -> Result<i64> {
    let metrics = retry(|| daemon.get_metrics())?.recv_timeout(Duration::from_secs(3))?;
    println!("STARTUP_METRICS={metrics:?}");
    metrics
        .get("limited-view-states")
        .copied()
        .ok_or_else(|| "missing worker state metric".into())
}
fn limited_has(view: &LimitedBrowse, expected: &str) -> bool {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(4) {
        if view.take_snapshot().is_some_and(|peers| {
            peers
                .iter()
                .any(|peer| peer.get_property_val_str("name") == Some(expected))
        }) {
            return !view.is_closed();
        }
        thread::sleep(Duration::from_millis(20));
    }
    false
}
// The observer never controls the worker. Unique types or one request between
// public metrics barriers match each initial send without a production ID/map.
// Only this executable's startup mode opts in. Publisher children do not log.
const STARTUP_TRACE_TARGET: &str = "mdns_sd::startup";
const CLEANUP_TRACE_TARGET: &str = "mdns_sd::cleanup";
const ATTEMPTS: usize = 8;
const ANCHOR_TYPE: &str = "_oe-anchor._tcp.local.";
static TRACE_CLOCK: OnceLock<Instant> = OnceLock::new();
static TRACE_RECORDS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static CLEANUP_OBSERVE: OnceLock<bool> = OnceLock::new();
struct StartupTrace;
static STARTUP_TRACE: StartupTrace = StartupTrace;
impl log::Log for StartupTrace {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.target() == STARTUP_TRACE_TARGET
            || (metadata.target() == CLEANUP_TRACE_TARGET && CLEANUP_OBSERVE.get().copied().unwrap_or(true))
    }
    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            let event = if record.target() == STARTUP_TRACE_TARGET {
                "SEND"
            } else {
                "REMOVE"
            };
            mark(format_args!("{event} {}", record.args()));
        }
    }
    fn flush(&self) {}
}
fn mark(message: impl std::fmt::Display) {
    let mut records = TRACE_RECORDS.lock().expect("startup trace lock");
    // A fixed finite probe must fail rather than lose observations silently.
    assert!(records.len() < 1024, "startup observation capacity exceeded");
    records.push(format!(
        "at_ns={} {message}",
        TRACE_CLOCK.get_or_init(Instant::now).elapsed().as_nanos()
    ));
}
fn cursor() -> usize {
    TRACE_RECORDS.lock().expect("startup trace lock").len()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Outcome {
    Accepted,
    Disconnected,
}
fn outcome(since: usize, ty: &str, kind: bool) -> Result<Outcome> {
    let key = format!("SEND type={ty} kind={} outcome=", kind_name(kind));
    let records = TRACE_RECORDS.lock().expect("startup trace lock");
    let matches: Vec<_> = records[since..]
        .iter()
        .filter_map(|line| line.split_once(&key))
        .collect();
    if matches.len() != 1 {
        return Err(format!("unverified initial send: {key} matches={}", matches.len()).into());
    }
    match matches[0].1 {
        "Accepted" => Ok(Outcome::Accepted),
        "Disconnected" => Ok(Outcome::Disconnected),
        other => Err(format!("unverified initial send outcome: {other}").into()),
    }
}
fn kind_name(limited: bool) -> &'static str {
    if limited {
        "limited"
    } else {
        "standard"
    }
}
enum View {
    Limited(LimitedBrowse),
    Standard(mdns_sd::Receiver<ServiceEvent>, usize),
}
impl View {
    fn open(daemon: &ServiceDaemon, ty: &str, limited: bool) -> Result<Self> {
        if limited {
            Ok(Self::Limited(retry(|| daemon.browse_limited(ty, 64))?))
        } else {
            Ok(Self::Standard(retry(|| daemon.browse(ty))?, 0))
        }
    }
    fn closed(&self) -> bool {
        match self {
            Self::Limited(view) => view.is_closed(),
            Self::Standard(rx, _) => rx.is_disconnected(),
        }
    }
    fn has(&mut self, value: &str) -> bool {
        match self {
            Self::Limited(view) => limited_has(view, value),
            Self::Standard(rx, starts) => poll_until(Duration::from_secs(4), || {
                let mut found = false;
                for event in rx.try_iter() {
                    match event {
                        ServiceEvent::SearchStarted(_) => *starts += 1,
                        ServiceEvent::ServiceResolved(info) => {
                            found |= info.get_property_val_str("name") == Some(value);
                        }
                        _ => {}
                    }
                }
                found && *starts > 0 && !rx.is_disconnected()
            }),
        }
    }
    fn removed(&mut self) -> bool {
        poll_until(Duration::from_secs(4), || match self {
            Self::Limited(view) => !view.is_closed() && view.take_snapshot().is_some_and(|peers| peers.is_empty()),
            Self::Standard(rx, _) => {
                !rx.is_disconnected()
                    && rx
                        .try_iter()
                        .any(|event| matches!(event, ServiceEvent::ServiceRemoved(..)))
            }
        })
    }
    fn drain(&mut self) {
        if let Self::Standard(rx, starts) = self {
            for event in rx.try_iter() {
                if matches!(event, ServiceEvent::SearchStarted(_)) {
                    *starts += 1;
                }
            }
            mark(format_args!("STANDARD_DRAIN complete observed_starts={starts}"));
        }
    }
    fn repeated_start(&mut self) -> bool {
        match self {
            // The limited API hides its lifetime events. Its observable checks
            // are exact state, same open view, fresh resolution and goodbye.
            Self::Limited(_) => !self.closed(),
            Self::Standard(rx, starts) => {
                // Initial start, first 10..50ms query, then 1/2/4/... seconds.
                // Count actual public events, including the post-barrier drain.
                // Bounded setup should never reach a longer interval; reject
                // unestablished coverage rather than cap its wait and pass.
                let Some(deadline) = standard_query_deadline(*starts) else {
                    return false;
                };
                let seconds = deadline.as_secs();
                mark(format_args!(
                    "REPEAT_WAIT observed_starts={starts} deadline_secs={seconds}"
                ));
                poll_until(deadline, || {
                    for event in rx.try_iter() {
                        if let ServiceEvent::SearchStarted(detail) = event {
                            *starts += 1;
                            mark(format_args!("REPEAT received {detail}"));
                            return !rx.is_disconnected();
                        }
                    }
                    false
                })
            }
        }
    }
    fn standard_cleanup_deadline(&self) -> Option<Duration> {
        match self {
            Self::Standard(_, starts) => standard_query_deadline(*starts),
            Self::Limited(_) => None,
        }
    }
}
fn standard_query_deadline(starts: usize) -> Option<Duration> {
    (starts <= 7).then(|| Duration::from_secs((1_u64 << starts.saturating_sub(2)) + 2))
}
fn automatic_cleanup(
    daemon: &ServiceDaemon,
    since: usize,
    ty: &str,
    limited: bool,
    deadline: Duration,
) -> Result<bool> {
    let prefix = format!("REMOVE type={ty} ");
    let expected = format!("kind={} reason=Disconnected completed=true", kind_name(limited));
    let observe = || {
        let records = TRACE_RECORDS.lock().expect("cleanup trace lock");
        let mut count = 0;
        let mut valid = 0;
        for line in &records[since..] {
            if let Some((_, detail)) = line.split_once(&prefix) {
                count += 1;
                valid += usize::from(detail == expected);
            }
        }
        (count, valid)
    };
    let witnessed_in_time = poll_until(deadline, || observe().0 > 0);
    // Completed normal removal precedes this real command barrier. No explicit
    // stop or same-type replacement is submitted until this check is finished.
    let states = limited_states(daemon)?;
    let (count, valid) = observe();
    let passed = witnessed_in_time && count == 1 && valid == 1;
    mark(format_args!(
        "CLEANUP_BARRIER type={ty} kind={} count={count} valid={valid} passed={passed}",
        kind_name(limited)
    ));
    println!("AUTOMATIC_CLEANUP type={ty} kind={} witnessed_in_time={witnessed_in_time} matched={count} valid={valid} states={states} passed={passed}", kind_name(limited));
    Ok(passed)
}
fn poll_until(deadline: Duration, mut accept: impl FnMut() -> bool) -> bool {
    let start = Instant::now();
    loop {
        if accept() {
            return true;
        }
        if start.elapsed() >= deadline {
            return false;
        }
        thread::sleep(Duration::from_millis(20));
    }
}
fn exact_states(daemon: &ServiceDaemon, expected: i64, deadline: Duration) -> Result<bool> {
    let start = Instant::now();
    loop {
        let actual = limited_states(daemon)?;
        println!(
            "STATE_SAMPLE elapsed_ms={} expected={expected} actual={actual}",
            start.elapsed().as_millis()
        );
        if actual == expected {
            return Ok(true);
        }
        if start.elapsed() >= deadline {
            return Ok(false);
        }
        thread::sleep(Duration::from_millis(20));
    }
}
fn check(results: &mut Vec<(String, bool)>, name: impl Into<String>, passed: bool) {
    let name = name.into();
    println!("CHECK {name}={passed}");
    results.push((name, passed));
}
fn advertise(publisher: &mut Publisher, ty: &str, value: &str) -> Result<()> {
    publisher.phase(&format!("advertise {ty} {value}"))
}
fn held_view(daemon: &ServiceDaemon, ty: &str, limited: bool) -> Result<(View, bool)> {
    let since = cursor();
    mark(format_args!("CALL held type={ty} kind={}", kind_name(limited)));
    let view = View::open(daemon, ty, limited)?;
    limited_states(daemon)?; // Startup result alone precedes replacement commit.
    let accepted = outcome(since, ty, limited)? == Outcome::Accepted;
    Ok((view, accepted))
}
fn startup_probe(root: &Path, mdns_port: u16, service_port: u16) -> Result<()> {
    fs::create_dir(root)?;
    let daemon = daemon(mdns_port)?;
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
    let (mut anchor, anchor_started) = match held_view(&daemon, ANCHOR_TYPE, true) {
        Ok(view) => view,
        Err(error) => {
            // The real missing-observer run must fail and still clean up its
            // ordinary publisher and worker through their normal interfaces.
            publisher.phase("finish")?;
            println!(
                "UNVERIFIED_CLEANUP publisher_exits_zero={} worker_shutdown_ack={}",
                publisher.child.wait()?.success(),
                stop(&daemon).is_ok()
            );
            return Err(error);
        }
    };
    advertise(&mut publisher, ANCHOR_TYPE, "anchor-initial")?;
    check(
        &mut results,
        "independent-guard-starts-resolves-exact-one",
        anchor_started && anchor.has("anchor-initial") && limited_states(&daemon)? == 1,
    );

    // No replay workload, worker pause or pacing is used to obtain cancellation.
    let since = cursor();
    let mut types = Vec::new();
    for index in 0..32 {
        let ty = format!("_oe-c{index:03}._tcp.local.");
        mark(format_args!("CALL drop-distinct type={ty} kind=limited"));
        drop(View::open(&daemon, &ty, true)?);
        mark(format_args!("DROP complete type={ty} kind=limited"));
        types.push(ty);
    }
    let at_barrier = limited_states(&daemon)?;
    let mut failed = 0;
    for ty in &types {
        let actual = outcome(since, ty, true)?;
        failed += usize::from(actual == Outcome::Disconnected);
        println!("DISTINCT type={ty} outcome={actual:?}");
    }
    let accepted = 32 - failed;
    println!(
        "DISTINCT_COVERAGE submitted=32 matched=32 disconnected={failed} accepted={accepted} at_barrier={at_barrier}"
    );
    check(&mut results, "distinct-real-failed-send-coverage", failed > 0);
    check(
        &mut results,
        "distinct-barrier-no-failed-state",
        at_barrier >= 1 && at_barrier <= 1 + accepted as i64,
    );
    // Accepted immediate drops meet their first 10..50ms query/disconnect
    // cleanup. Three seconds includes the next 1s retry and scheduling slack.
    check(
        &mut results,
        "distinct-eventual-exact-guard-state",
        exact_states(&daemon, 1, Duration::from_secs(3))?,
    );
    advertise(&mut publisher, ANCHOR_TYPE, "anchor-after-distinct")?;
    check(
        &mut results,
        "distinct-preserves-guard-identity-fresh-update",
        !anchor.closed() && anchor.has("anchor-after-distinct"),
    );
    // Explicit cleanup occurs AFTER the strict measurements. In the buggy
    // baseline it permits independent replacement cases without hiding red.
    for ty in &types {
        retry(|| daemon.stop_browse(ty))?;
    }
    check(
        &mut results,
        "distinct-explicit-stop-cleanup",
        limited_states(&daemon)? == 1,
    );

    for (case, (old_limited, new_limited)) in [(true, true), (true, false), (false, true), (false, false)]
        .into_iter()
        .enumerate()
    {
        let label = format!("{}-over-{}", kind_name(new_limited), kind_name(old_limited));
        let mut witnessed = false;
        for attempt in 0..ATTEMPTS {
            let ty = format!("_oe-r{case}{attempt}._tcp.local.");
            let (mut old, started) = held_view(&daemon, &ty, old_limited)?;
            advertise(&mut publisher, &ty, "old-live")?;
            let resolved = old.has("old-live");
            let before = limited_states(&daemon)?;
            check(
                &mut results,
                format!("{label}-{attempt}-live-baseline"),
                started && resolved && !old.closed() && before == 1 + i64::from(old_limited),
            );
            let since = cursor();
            mark(format_args!(
                "CALL drop-replacement type={ty} kind={}",
                kind_name(new_limited)
            ));
            drop(View::open(&daemon, &ty, new_limited)?);
            mark(format_args!("DROP complete type={ty} kind={}", kind_name(new_limited)));
            let after = limited_states(&daemon)?;
            let actual = outcome(since, &ty, new_limited)?;
            println!("REPLACEMENT case={label} attempt={attempt} type={ty} outcome={actual:?} before={before} after={after} old_closed={}", old.closed());
            if actual == Outcome::Accepted {
                // This is a real accepted-then-dropped lifecycle, never a
                // failed-start witness. Keep its observations and exact cleanup.
                check(
                    &mut results,
                    format!("{label}-{attempt}-accepted-old-closed"),
                    old.closed(),
                );
                check(
                    &mut results,
                    format!("{label}-{attempt}-accepted-transient-state"),
                    (1..=1 + i64::from(new_limited)).contains(&after),
                );
                let removed = automatic_cleanup(&daemon, since, &ty, new_limited, Duration::from_secs(3))?;
                let exact = exact_states(&daemon, 1, Duration::from_secs(3))?;
                check(
                    &mut results,
                    format!("{label}-{attempt}-accepted-drop-cleanup"),
                    removed && exact,
                );
                retry(|| daemon.stop_browse(&ty))?;
                limited_states(&daemon)?;
                continue;
            }
            witnessed = true;
            old.drain();
            mark(format_args!(
                "DRAIN_AFTER_FAILED_BARRIER type={ty} outcome=Disconnected"
            ));
            check(
                &mut results,
                format!("{label}-failed-preserves-exact-identity-state"),
                after == before && !old.closed(),
            );
            check(
                &mut results,
                format!(
                    "{label}-{}",
                    if old_limited {
                        "failed-limited-view-stays-open"
                    } else {
                        "failed-standard-retains-repeated-search"
                    }
                ),
                old.repeated_start(),
            );
            advertise(&mut publisher, &ty, "fresh-after-failure")?;
            check(
                &mut results,
                format!("{label}-failed-fresh-update"),
                old.has("fresh-after-failure"),
            );
            publisher.phase(&format!("withdraw {ty}"))?;
            check(&mut results, format!("{label}-failed-goodbye"), old.removed());
            advertise(&mut publisher, &ty, "returned-after-goodbye")?;
            let returned = old.has("returned-after-goodbye");
            let returned_states = limited_states(&daemon)?;
            check(
                &mut results,
                format!("{label}-failed-resolves-returned-peer"),
                returned && returned_states == before && !old.closed(),
            );

            let (mut successor, accepted) = held_view(&daemon, &ty, new_limited)?;
            let fresh = successor.has("returned-after-goodbye");
            let successor_states = limited_states(&daemon)?;
            check(
                &mut results,
                format!("{label}-held-successor-start-resolution-closure"),
                accepted
                    && fresh
                    && old.closed()
                    && !successor.closed()
                    && successor_states == 1 + i64::from(new_limited),
            );
            let (mut newest, accepted) = held_view(&daemon, &ty, true)?;
            let fresh = newest.has("returned-after-goodbye");
            let newest_states = limited_states(&daemon)?;
            check(
                &mut results,
                format!("{label}-newest-one-live-generation"),
                accepted && fresh && successor.closed() && !newest.closed() && newest_states == 2,
            );
            drop(old);
            drop(successor);
            mark(format_args!("DROP older-generations-complete type={ty}"));
            advertise(&mut publisher, &ty, "newest-after-older-drops")?;
            let fresh = newest.has("newest-after-older-drops");
            let after_older_drops = limited_states(&daemon)?;
            check(
                &mut results,
                format!("{label}-newest-survives-older-drops"),
                fresh && !newest.closed() && after_older_drops == 2,
            );
            drop(newest);
            mark(format_args!("DROP held-newest-complete type={ty}"));
            // Held views can reach the 1s, 2s or 4s exponential retry. Eight
            // seconds covers that next real retry plus scheduling slack.
            check(
                &mut results,
                format!("{label}-held-drop-exact-cleanup"),
                exact_states(&daemon, 1, Duration::from_secs(8))?,
            );
            let (final_view, accepted) = held_view(&daemon, &ty, true)?;
            retry(|| daemon.stop_browse(&ty))?;
            let stopped_states = limited_states(&daemon)?;
            check(
                &mut results,
                format!("{label}-stop-closes-final-view"),
                accepted && final_view.closed() && stopped_states == 1,
            );
            break;
        }
        println!("FAILED_REPLACEMENT_COVERAGE case={label} witnessed={witnessed} budget={ATTEMPTS}");
        check(&mut results, format!("{label}-disconnected-coverage"), witnessed);
    }
    // Required positive standard-drop case, independent of natural immediate
    // drop outcomes in the failed-start attempt loop.
    let ty = "_oe-drop-std._tcp.local.";
    let (mut active_standard, accepted) = held_view(&daemon, ty, false)?;
    advertise(&mut publisher, ty, "held-standard")?;
    let resolved = active_standard.has("held-standard");
    let committed_states = limited_states(&daemon)?;
    active_standard.drain();
    let repeated = active_standard.repeated_start();
    let cleanup_deadline = active_standard.standard_cleanup_deadline();
    let committed = accepted
        && resolved
        && repeated
        && !active_standard.closed()
        && committed_states == 1
        && cleanup_deadline.is_some();
    check(
        &mut results,
        "required-held-standard-start-resolution-commit-active",
        committed,
    );
    let since = cursor();
    drop(active_standard);
    mark(format_args!("DROP held-active-standard-complete type={ty}"));
    let removed = automatic_cleanup(&daemon, since, ty, false, cleanup_deadline.unwrap_or(Duration::ZERO))?;
    let exact = limited_states(&daemon)? == 1;
    check(
        &mut results,
        "required-accepted-standard-drop-automatic-removal",
        removed && exact,
    );
    println!("ACCEPTED_STANDARD_DROP_COVERAGE type={ty} committed_active={committed} normal_removal={removed}");
    check(
        &mut results,
        "required-accepted-standard-drop-coverage",
        committed && removed,
    );
    if !removed {
        mark(format_args!("EXPLICIT_STOP_AFTER_FAILED_AUTOMATIC_CHECK type={ty}"));
        retry(|| daemon.stop_browse(ty))?;
        limited_states(&daemon)?;
    }
    advertise(&mut publisher, ANCHOR_TYPE, "anchor-final")?;
    let anchor_fresh = anchor.has("anchor-final");
    let anchor_states = limited_states(&daemon)?;
    check(
        &mut results,
        "independent-guard-final-identity-fresh-exact-state",
        anchor_fresh && !anchor.closed() && anchor_states == 1,
    );
    retry(|| daemon.stop_browse(ANCHOR_TYPE))?;
    let final_states = limited_states(&daemon)?;
    check(
        &mut results,
        "final-stop-exact-zero-state",
        anchor.closed() && final_states == 0,
    );
    publisher.phase("finish")?;
    check(&mut results, "publisher-exits-zero", publisher.child.wait()?.success());
    check(&mut results, "worker-shutdown-ack", stop(&daemon).is_ok());
    println!(
        "STARTUP_SUMMARY passed={} total={}",
        results.iter().filter(|(_, ok)| *ok).count(),
        results.len()
    );
    if results.iter().all(|(_, passed)| *passed) {
        Ok(())
    } else {
        Err("observed startup checks or coverage failed".into())
    }
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 4 || !matches!(args[0].as_str(), "probe" | "startup" | "publisher") {
        return Err("discovery_recovery probe|startup NEW_DIR MDNS_PORT SERVICE_PORT".into());
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
    } else if args[0] == "startup" {
        startup_probe(Path::new(&args[1]), mdns_port, service_port)
    } else {
        probe(Path::new(&args[1]), mdns_port, service_port)
    }
}
fn main() {
    if std::env::args().nth(1).as_deref() == Some("startup")
        && std::env::var_os("OPENERGY_STARTUP_OBSERVE").as_deref() != Some(std::ffi::OsStr::new("0"))
    {
        TRACE_CLOCK.get_or_init(Instant::now);
        CLEANUP_OBSERVE.get_or_init(|| {
            std::env::var_os("OPENERGY_STARTUP_CLEANUP_OBSERVE").as_deref() != Some(std::ffi::OsStr::new("0"))
        });
        log::set_logger(&STARTUP_TRACE).expect("startup observer logger");
        log::set_max_level(log::LevelFilter::Trace);
    }
    let result = run();
    for record in TRACE_RECORDS.lock().expect("startup trace lock").iter() {
        eprintln!("STARTUP_TRACE {record}");
    }
    if let Err(error) = result {
        eprintln!("discovery_recovery: {error}");
        std::process::exit(1);
    }
}
