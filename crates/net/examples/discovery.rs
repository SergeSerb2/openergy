//! Real multicast advertise/browse on the loopback interface and WP4 UDP port.
use openergy_net::{
    discovery::{Discovery, DiscoveryScope},
    Identity,
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
fn main() {
    if let Err(e) = run() {
        eprintln!("discovery: {e}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mode = args.next().ok_or("discovery advertise|browse DIR")?;
    let dir = PathBuf::from(args.next().ok_or("missing DIR")?);
    if args.next().is_some() || !["advertise", "browse"].contains(&mode.as_str()) {
        return Err("invalid arguments".into());
    }
    let mut discovery = Discovery::new(DiscoveryScope::Loopback, 24979)?;
    if mode == "advertise" {
        let identity = Identity::load_or_create(&dir, "wp4-mdns")?;
        discovery.advertise("wp4-loopback", 24970, identity.fingerprint)?;
        println!(
            "ADVERTISE _openergy._tcp.local. name=wp4-loopback ver=1.8 fp={}",
            &identity.fingerprint.to_synergy_line()[10..26]
        );
    }
    let start = Instant::now();
    let mut found = false;
    while start.elapsed() < Duration::from_secs(5) {
        if let Some(peers) = discovery.poll() {
            for peer in peers {
                println!(
                    "RESOLVED name={} host={} port={} fp={:?}",
                    peer.name, peer.host, peer.port, peer.fingerprint_hint
                );
                if peer.name == "wp4-loopback"
                    && peer.port == 24970
                    && peer.fingerprint_hint.as_ref().is_some_and(|fp| fp.len() == 16)
                {
                    found = true;
                }
            }
        }
        if let Some(error) = discovery.errors().into_iter().next() {
            return Err(error.into());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    discovery.shutdown()?;
    if mode == "browse" && !found {
        return Err("no loopback advertisement resolved".into());
    }
    Ok(())
}
