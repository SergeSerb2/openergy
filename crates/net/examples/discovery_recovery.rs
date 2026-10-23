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
    for line in io::stdin().lock().lines() {
        let phase = line?;
        let (phase_name, selected) = phase.split_once(' ').unwrap_or((&phase, ""));
        match phase_name {
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
        self.ready(phase)
    }
    fn begin(&mut self, phase: &str) -> Result<()> {
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
fn startup_probe(root: &Path, mdns_port: u16, service_port: u16) -> Result<()> {
    fs::create_dir(root)?;
    let daemon = daemon(mdns_port)?;
    let cache_feed = daemon.browse(SERVICE_TYPE)?;
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
    publisher.phase("burst")?;
    let mut results = vec![(
        "ordinary-replay-cache-overflow",
        cache_feed.is_disconnected() && cache_feed.len() == 1000,
    )];
    let live = retry(|| daemon.browse_limited(GUARD_TYPE, 64))?;
    publisher.phase("guard-start")?;
    let live_resolved = limited_has(&live, "guard-start");
    let before = limited_states(&daemon)?;
    results.push(("live-handle-resolves", live_resolved && before == 1));

    // Keep this workload bounded below the 100-command queue. The worker must
    // replay ordinary retained records before handling the cancelled startups.
    let mut replays = Vec::new();
    for _ in 0..16 {
        replays.push(retry(|| daemon.browse_cache(SERVICE_TYPE))?);
    }
    for index in 0..32 {
        let ty = format!("_oe-c{index:03}._tcp.local.");
        drop(retry(|| daemon.browse_limited(&ty, 64))?);
    }
    // Failed replacement must preserve both the prior sender and its state.
    drop(retry(|| daemon.browse_limited(GUARD_TYPE, 64))?);
    drop(retry(|| daemon.browse(GUARD_TYPE))?);
    let after_barrier = limited_states(&daemon)?;
    thread::sleep(Duration::from_millis(300));
    let after_settle = limited_states(&daemon)?;
    let replayed = replays
        .iter()
        .filter(|rx| rx.is_disconnected() && rx.len() == 1000)
        .count();
    println!("CANCELLED_STARTUPS distinct=32 replay_commands=16 replayed_overflows={replayed} before={before} after_barrier={after_barrier} after_settle={after_settle} live_closed={}", live.is_closed());
    results.push(("ordinary-replay-commands-drained", replayed == 16));
    results.push(("cancelled-distinct-types-retain-no-state", after_settle == before));
    results.push(("cancelled-replacements-preserve-identity", !live.is_closed()));
    publisher.phase("guard-update")?;
    results.push((
        "surviving-handle-receives-live-update",
        limited_has(&live, "guard-update"),
    ));

    let replacement = retry(|| daemon.browse_limited(GUARD_TYPE, 64))?;
    let replacement_resolved = limited_has(&replacement, "guard-update");
    let after_replacement = limited_states(&daemon)?;
    results.push((
        "successful-limited-replacement-has-new-identity",
        live.is_closed() && replacement_resolved && after_replacement == 1,
    ));
    let standard = retry(|| daemon.browse(GUARD_TYPE))?;
    let start = Instant::now();
    let mut standard_resolved = false;
    while start.elapsed() < Duration::from_secs(4) && !standard_resolved {
        standard_resolved |= standard.try_iter().any(|event| matches!(event, ServiceEvent::ServiceResolved(info) if info.get_property_val_str("name") == Some("guard-update")));
        thread::sleep(Duration::from_millis(20));
    }
    results.push((
        "successful-standard-replacement-releases-state",
        replacement.is_closed() && standard_resolved && limited_states(&daemon)? == 0,
    ));
    drop(replays);
    let mut standard_replays = Vec::new();
    for _ in 0..16 {
        standard_replays.push(retry(|| daemon.browse_cache(SERVICE_TYPE))?);
    }
    drop(retry(|| daemon.browse_limited(GUARD_TYPE, 64))?);
    drop(retry(|| daemon.browse(GUARD_TYPE))?);
    let standard_states = limited_states(&daemon)?;
    let replayed = standard_replays
        .iter()
        .filter(|rx| rx.is_disconnected() && rx.len() == 1000)
        .count();
    results.push((
        "cancelled-replacements-preserve-standard-identity",
        !standard.is_disconnected() && standard_states == 0 && replayed == 16,
    ));
    publisher.phase("guard-final")?;
    let start = Instant::now();
    let mut standard_updated = false;
    while start.elapsed() < Duration::from_secs(4) && !standard_updated {
        standard_updated |= standard.try_iter().any(|event| matches!(event, ServiceEvent::ServiceResolved(info) if info.get_property_val_str("name") == Some("guard-final")));
        thread::sleep(Duration::from_millis(20));
    }
    results.push(("surviving-standard-handle-receives-live-update", standard_updated));
    let final_view = retry(|| daemon.browse_limited(GUARD_TYPE, 64))?;
    let final_resolved = limited_has(&final_view, "guard-final");
    results.push((
        "standard-to-limited-replacement",
        standard.is_disconnected() && final_resolved && limited_states(&daemon)? == 1,
    ));
    retry(|| daemon.stop_browse(GUARD_TYPE))?;
    let after_stop = limited_states(&daemon)?;
    results.push((
        "final-stop-clears-worker-state",
        final_view.is_closed() && after_stop == 0,
    ));
    publisher.phase("finish")?;
    results.push(("publisher-exits-zero", publisher.child.wait()?.success()));
    stop(&daemon)?;
    results.push(("worker-shutdown-ack", final_view.is_closed()));
    for (name, passed) in &results {
        println!("CHECK {name}={passed}");
    }
    if results.iter().all(|(_, passed)| *passed) {
        Ok(())
    } else {
        Err("limited startup checks failed".into())
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
    if let Err(error) = run() {
        eprintln!("discovery_recovery: {error}");
        std::process::exit(1);
    }
}
