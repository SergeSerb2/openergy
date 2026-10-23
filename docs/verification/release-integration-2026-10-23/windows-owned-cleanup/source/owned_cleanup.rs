//! Diagnostic-only public Discovery lifecycle child. The external OS observer
//! must see this exact worker and its UDP sockets disappear before requesting exit.
use openergy_net::{discovery::{Discovery, DiscoveryScope}, Fingerprint};
use std::{io::{self, BufRead, Write}, sync::atomic::{AtomicBool, AtomicU32, Ordering}, time::{Duration, Instant}};

#[link(name = "kernel32")]
extern "system" { fn GetCurrentThreadId() -> u32; }
fn native_thread_id() -> u32 {
    // SAFETY: This read-only Win32 function has no arguments or preconditions.
    unsafe { GetCurrentThreadId() }
}
static WORKER: AtomicU32 = AtomicU32::new(0);
static WORKER_CONFLICT: AtomicBool = AtomicBool::new(false);
struct WorkerIdentity;
static LOGGER: WorkerIdentity = WorkerIdentity;
impl log::Log for WorkerIdentity {
    fn enabled(&self, meta: &log::Metadata<'_>) -> bool { meta.target() == "mdns_sd::service_daemon" }
    fn log(&self, record: &log::Record<'_>) {
        // Exact e4f48ba exec_command trace site. No worker code or state is changed.
        if !self.enabled(record.metadata()) || record.line() != Some(4058) { return; }
        let tid = native_thread_id();
        match WORKER.compare_exchange(0, tid, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => { println!("WORKER_LOG native_tid={tid} source_line=4058"); let _ = io::stdout().flush(); }
            Err(known) if known != tid => { WORKER_CONFLICT.store(true, Ordering::SeqCst); println!("WORKER_CONFLICT native_tid={tid} previous={known}"); }
            _ => {}
        }
    }
    fn flush(&self) {}
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 || !matches!(args[0].as_str(), "shutdown" | "drop") { return Err("owned_cleanup shutdown|drop MDNS_PORT SERVICE_PORT".into()); }
    let mdns: u16 = args[1].parse()?; let service: u16 = args[2].parse()?;
    if !(25080..=25099).contains(&mdns) || !(25080..=25099).contains(&service) || mdns == service { return Err("requires distinct custom loopback ports 25080..25099".into()); }
    log::set_logger(&LOGGER).map_err(|_| "logger initialization failed")?;
    log::set_max_level(log::LevelFilter::Trace);
    let mut owned: Option<Discovery> = None;
    let mut initialized = false;
    let mut cleaned = false;
    println!("READY pid={} main_tid={} mode={}", std::process::id(), native_thread_id(), args[0]); io::stdout().flush()?;
    for line in io::stdin().lock().lines() {
        match line?.as_str() {
            "init" => {
                if initialized { return Err("initialization allowed once".into()); }
                initialized = true;
                let mut discovery = Discovery::new(DiscoveryScope::Loopback, mdns)?;
                let name = format!("owned-normal-{}", args[0]);
                discovery.advertise(&name, service, Fingerprint([0x42; 32]))?;
                let started = Instant::now();
                loop {
                    discovery.poll();
                    let errors = discovery.errors(); if !errors.is_empty() { return Err(format!("discovery errors: {errors:?}").into()); }
                    if discovery.snapshot().iter().any(|peer| peer.name == name && peer.port == service) { break; }
                    if started.elapsed() >= Duration::from_secs(10) { return Err("ordinary loopback publication did not resolve within 10 seconds".into()); }
                    std::thread::sleep(Duration::from_millis(20));
                }
                let worker = WORKER.load(Ordering::SeqCst);
                if worker == 0 || worker == native_thread_id() || WORKER_CONFLICT.load(Ordering::SeqCst) { return Err("real worker identity was not uniquely observed".into()); }
                owned = Some(discovery);
                println!("LIVE worker_tid={worker} resolved=true init_elapsed_us={}", started.elapsed().as_micros());
            }
            "cleanup" => {
                if !initialized || cleaned { return Err("cleanup requires one initialized instance".into()); }
                if args[0] == "shutdown" {
                    // Deliberately keep the object alive until the later exit command.
                    owned.as_mut().ok_or("missing Discovery")?.shutdown()?;
                } else { drop(owned.take().ok_or("missing Discovery")?); }
                cleaned = true;
                println!("CLEANUP_RETURNED mode={} object_retained={}", args[0], owned.is_some());
            }
            "ping" => { println!("ALIVE pid={} cleanup_done={cleaned} object_retained={}", std::process::id(), owned.is_some()); }
            "exit" => {
                if !cleaned { return Err("exit requires cleanup and external observation".into()); }
                println!("EXITING"); io::stdout().flush()?; return Ok(());
            }
            _ => return Err("invalid private control command".into()),
        }
        io::stdout().flush()?;
    }
    Err("control pipe closed before exit".into())
}
fn main() { if let Err(error) = run() { eprintln!("owned_cleanup: {error}"); std::process::exit(1); } }
