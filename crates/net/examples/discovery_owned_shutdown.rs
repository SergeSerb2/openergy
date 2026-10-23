//! A real private child for discovery worker/socket lifetime checks.
//! Only this child lowers its descriptor limit. The parent must keep it alive
//! after `release` to inspect worker and socket cleanup independently of exit.
#[cfg(unix)]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    use openergy_net::{
        discovery::{Discovery, DiscoveryScope},
        Fingerprint,
    };
    use std::{
        fs::File,
        io::{self, BufRead, Write},
        time::Duration,
    };

    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 || !matches!(args[0].as_str(), "drop" | "shutdown") {
        return Err("discovery_owned_shutdown drop|shutdown MDNS_PORT SERVICE_PORT".into());
    }
    let port: u16 = args[1].parse()?;
    let service_port: u16 = args[2].parse()?;
    if !(25080..=25099).contains(&port) || !(25080..=25099).contains(&service_port) || port == service_port {
        return Err("requires distinct assigned ports 25080..=25099".into());
    }
    let mut discovery = None;
    let mut files = Vec::new();
    let mut original = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    println!("READY pid={}", std::process::id());
    io::stdout().flush()?;
    for line in io::stdin().lock().lines() {
        match line?.as_str() {
            "init" => {
                if discovery.is_some() {
                    return Err("already initialized".into());
                }
                let mut owned = Discovery::new(DiscoveryScope::Loopback, port)?;
                if args[0] == "shutdown" {
                    owned.advertise("shutdown-owned-child", service_port, Fingerprint([0x42; 32]))?;
                }
                if args[0] == "shutdown" {
                    let start = std::time::Instant::now();
                    while !owned.snapshot().iter().any(|peer| peer.name == "shutdown-owned-child") {
                        if start.elapsed() > Duration::from_secs(5) {
                            return Err("owned advertisement did not resolve on loopback".into());
                        }
                        owned.poll();
                        std::thread::sleep(Duration::from_millis(20));
                    }
                    println!("OWNED_ADVERTISEMENT_RESOLVED");
                }
                discovery = Some(owned);
                std::thread::sleep(Duration::from_millis(300));
                println!("LIVE");
            }
            "exhaust" => {
                // SAFETY: get/setrlimit affect this private child process only.
                if unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut original) } != 0 {
                    return Err(io::Error::last_os_error().into());
                }
                let limited = libc::rlimit {
                    rlim_cur: original.rlim_cur.min(128),
                    rlim_max: original.rlim_max,
                };
                if unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &limited) } != 0 {
                    return Err(io::Error::last_os_error().into());
                }
                loop {
                    match File::open("/dev/null") {
                        Ok(file) => files.push(file),
                        Err(error) if error.raw_os_error() == Some(libc::EMFILE) => break,
                        Err(error) => return Err(error.into()),
                    }
                }
                println!("EXHAUSTED soft={} held={}", limited.rlim_cur, files.len());
            }
            "cleanup" => {
                let mut owned = discovery.take().ok_or("not initialized")?;
                if args[0] == "shutdown" {
                    println!("SHUTDOWN_RESULT={:?}", owned.shutdown());
                }
                drop(owned);
                println!("DROPPED");
            }
            "release" => {
                files.clear();
                if original.rlim_max != 0 && unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &original) } != 0 {
                    return Err(io::Error::last_os_error().into());
                }
                println!("RELEASED");
            }
            "exit" => return Ok(()),
            _ => return Err("invalid child command".into()),
        }
        io::stdout().flush()?;
    }
    Ok(())
}
fn main() {
    #[cfg(unix)]
    if let Err(error) = run() {
        eprintln!("discovery_owned_shutdown: {error}");
        std::process::exit(1);
    }
    #[cfg(not(unix))]
    panic!("this descriptor-limit probe requires Unix");
}
