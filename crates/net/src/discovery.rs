//! mDNS never connects or trusts a peer. The 16-hex TXT fingerprint is display
//! information only. mdns-sd owns its own internal worker thread; this wrapper
//! is polled by the daemon io thread and does not create a forwarding thread.
use crate::{identity::failed, DiscoveredPeer, Fingerprint, NetError};
use mdns_sd::{DaemonEvent, IfKind, LimitedBrowse, Receiver, ServiceDaemon, ServiceInfo};
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

pub const SERVICE_TYPE: &str = "_openergy._tcp.local.";
const BROWSE_RETRY: Duration = Duration::from_secs(1);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscoveryScope {
    Loopback,
    Lan,
}

pub struct Discovery {
    daemon: ServiceDaemon,
    events: Option<LimitedBrowse>,
    monitor: Receiver<DaemonEvent>,
    peers: BTreeMap<String, DiscoveredPeer>,
    published: Option<String>,
    scope: DiscoveryScope,
    dirty: bool,
    last_snapshot: Instant,
    browse_retry: Option<Instant>,
}
impl Discovery {
    /// Loopback excludes all interfaces before browsing or advertising. A custom
    /// UDP port also avoids touching the workstation's Bonjour service during checks.
    pub fn new(scope: DiscoveryScope, mdns_port: u16) -> Result<Self, NetError> {
        Self::with_browse(scope, mdns_port, true)
    }
    pub(crate) fn with_browse(scope: DiscoveryScope, mdns_port: u16, browse: bool) -> Result<Self, NetError> {
        if mdns_port == 24800 {
            return Err(NetError::ProductionPortDenied);
        }
        let daemon = ServiceDaemon::new_with_port(mdns_port).map_err(failed)?;
        let setup = (|| {
            if scope == DiscoveryScope::Loopback {
                daemon.disable_interface(IfKind::All).map_err(failed)?;
                daemon.enable_interface(IfKind::LoopbackV4).map_err(failed)?;
            }
            let monitor = daemon.monitor().map_err(failed)?;
            let events = if browse {
                Some(daemon.browse_limited(SERVICE_TYPE, 64).map_err(failed)?)
            } else {
                None
            };
            Ok((monitor, events))
        })();
        let (monitor, events) = match setup {
            Ok(v) => v,
            Err(e) => {
                let _ = daemon.shutdown();
                return Err(e);
            }
        };
        Ok(Self {
            daemon,
            events,
            monitor,
            peers: BTreeMap::new(),
            published: None,
            scope,
            dirty: false,
            last_snapshot: Instant::now() - Duration::from_secs(1),
            browse_retry: None,
        })
    }
    pub fn advertise(&mut self, name: &str, port: u16, fp: Fingerprint) -> Result<(), NetError> {
        crate::NetDriver::check_port(port)?;
        let hex: String = fp.0.iter().map(|b| format!("{b:02x}")).collect();
        let properties = [("name", name), ("ver", "1.8"), ("fp", &hex[..16])];
        let host = format!("openergy-{}.local.", &hex[..16]);
        let address = if self.scope == DiscoveryScope::Loopback {
            "127.0.0.1"
        } else {
            ""
        };
        let mut info = ServiceInfo::new(SERVICE_TYPE, name, &host, address, port, &properties[..]).map_err(failed)?;
        if self.scope == DiscoveryScope::Lan {
            info = info.enable_addr_auto();
        }
        if let Some(old) = &self.published {
            self.daemon.unregister(old).map_err(failed)?;
        }
        let full = info.get_fullname().to_owned();
        self.daemon.register(info).map_err(failed)?;
        self.published = Some(full);
        Ok(())
    }
    /// Returns changed snapshots at most once per second.
    pub fn poll(&mut self) -> Option<Vec<DiscoveredPeer>> {
        if self.browse_retry.is_some_and(|deadline| Instant::now() >= deadline) {
            // A closed subscription can be recreated on the existing worker.
            // Back off even if its bounded command queue is still full.
            self.browse_retry = Some(Instant::now() + BROWSE_RETRY);
            match self.daemon.browse_limited(SERVICE_TYPE, 64) {
                Ok(events) => {
                    self.events = Some(events);
                    self.browse_retry = None;
                }
                Err(error) => log::warn!("mDNS discovery browse retry failed: {error}"),
            }
        }
        if let Some(events) = &self.events {
            if events.is_closed() {
                self.events = None;
                self.browse_retry = Some(Instant::now() + BROWSE_RETRY);
                log::warn!("mDNS discovery subscription closed; retrying");
                self.dirty |= !self.peers.is_empty();
                self.peers.clear();
            } else if let Some(snapshot) = events.take_snapshot() {
                // Replace the complete selected view. Absence conveys removal;
                // merging would retain peers lost while the caller was paused.
                let peers = snapshot
                    .into_iter()
                    .map(|info| {
                        let peer = DiscoveredPeer {
                            name: info
                                .get_property_val_str("name")
                                .unwrap_or(info.get_fullname())
                                .to_owned(),
                            host: info.get_hostname().to_owned(),
                            port: info.get_port(),
                            fingerprint_hint: info
                                .get_property_val_str("fp")
                                .filter(|s| s.len() == 16 && s.bytes().all(|b| b.is_ascii_hexdigit()))
                                .map(str::to_owned),
                        };
                        (info.get_fullname().to_owned(), peer)
                    })
                    .collect();
                if self.peers != peers {
                    self.peers = peers;
                    self.dirty = true;
                }
            }
        }
        if self.dirty && self.last_snapshot.elapsed() >= Duration::from_secs(1) {
            self.dirty = false;
            self.last_snapshot = Instant::now();
            Some(self.snapshot())
        } else {
            None
        }
    }
    pub fn snapshot(&self) -> Vec<DiscoveredPeer> {
        self.peers.values().cloned().collect()
    }
    pub fn errors(&self) -> Vec<String> {
        self.monitor
            .try_iter()
            .take(64)
            .filter_map(|e| match e {
                DaemonEvent::Error(e) => Some(e.to_string()),
                _ => None,
            })
            .collect()
    }
    /// Requests worker shutdown; success means submission, not a worker join.
    /// mdns-sd can also enqueue Exit and then report a wake-socket error.
    pub fn shutdown(&mut self) -> Result<(), NetError> {
        self.events = None;
        self.browse_retry = None;
        self.published = None;
        // Exit already unregisters services and stops browsing on the owned
        // worker. Separate cleanup commands can fail before Exit is submitted
        // (e.g. no descriptor for mdns-sd's temporary command-wake socket).
        self.daemon.shutdown().map_err(failed)?;
        Ok(())
    }
}
impl Drop for Discovery {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}
