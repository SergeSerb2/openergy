//! Reuses one NetDriver and waker through TLS -> reset -> plaintext. All peers
//! are real child binaries; all sockets stay on assigned loopback ports.
use openergy_net::{Identity, NetCommand, NetConfig, NetDriver, NetEvent, Priority, TransportMode, TrustStore};
use openergy_session::ConnId;
use std::{
    net::TcpStream,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};

struct Peer(Child);
impl Drop for Peer {
    fn drop(&mut self) {
        if matches!(self.0.try_wait(), Ok(None)) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
fn main() {
    if let Err(e) = run() {
        eprintln!("lifecycle: {e}");
        std::process::exit(1);
    }
}
fn cycle(
    driver: &mut NetDriver,
    root: &std::path::Path,
    port: u16,
    plain: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = ([127, 0, 0, 1], port).into();
    let mut events = Vec::new();
    driver.execute(NetCommand::Listen { addr }, &mut events);
    if !events.iter().any(|e| matches!(e, NetEvent::Listening { .. })) {
        return Err("listener failed".into());
    }
    events.clear();
    let binary = std::env::current_exe()?.with_file_name("tls_peer");
    let mut command = Command::new(binary);
    command
        .args(["client", "--dir"])
        .arg(root.join("client"))
        .args(["--addr", &addr.to_string(), "--seconds", "4"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if plain {
        command.arg("--plain");
    }
    let mut peer = Peer(command.spawn()?);
    let start = Instant::now();
    let mut conn: Option<ConnId> = None;
    let mut returned_frame = false;
    while start.elapsed() < Duration::from_secs(3) && !returned_frame {
        driver.poll(Some(Duration::from_millis(20)), &mut events)?;
        for event in std::mem::take(&mut events) {
            match event {
                NetEvent::Connected { conn: id, tls, .. } => {
                    if tls == plain {
                        return Err("wrong TLS setting after reconfigure".into());
                    }
                    conn = Some(id);
                    driver.execute(
                        NetCommand::SendFrame {
                            conn: id,
                            payload: b"Synergy\x00\x01\x00\x08".to_vec(),
                            priority: Priority::Control,
                        },
                        &mut events,
                    );
                }
                NetEvent::Frame { .. } => returned_frame = true,
                NetEvent::UntrustedPeer { .. } => return Err("expected pre-seeded trust".into()),
                NetEvent::Error { message, .. } | NetEvent::ListenFailed { error: message, .. } => {
                    return Err(message.into())
                }
                _ => {}
            }
        }
    }
    let conn = conn.ok_or("no connection")?;
    if !returned_frame {
        return Err("no return frame".into());
    }
    driver.execute(NetCommand::StopListening, &mut events);
    if TcpStream::connect_timeout(&addr, Duration::from_millis(200)).is_ok() {
        return Err("StopListening kept listener open".into());
    }
    if driver.conn_info(conn).is_none() {
        return Err("StopListening closed established peer".into());
    }
    driver.execute(
        NetCommand::SendFrame {
            conn,
            payload: b"CNOP".to_vec(),
            priority: Priority::Control,
        },
        &mut events,
    );
    for _ in 0..5 {
        driver.poll(Some(Duration::from_millis(10)), &mut events)?;
    }
    events.clear();
    driver.execute(NetCommand::Shutdown, &mut events);
    if events
        .iter()
        .filter(|e| matches!(e, NetEvent::Disconnected { .. }))
        .count()
        != 1
        || driver.conn_info(conn).is_some()
    {
        return Err("Shutdown did not disconnect every peer".into());
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    while peer.0.try_wait()?.is_none() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    if peer.0.try_wait()?.is_none() {
        return Err("child did not observe shutdown".into());
    }
    let mut output = String::new();
    if let Some(mut stdout) = peer.0.stdout.take() {
        use std::io::Read;
        stdout.read_to_string(&mut output)?;
    }
    if !output.contains("delivered=2") {
        return Err(format!("peer did not receive frame after StopListening: {output}").into());
    }
    println!(
        "CYCLE tls={} port={port} bidirectional_frames=PASS StopListening_keeps_peer=PASS Shutdown_disconnected=1",
        !plain
    );
    Ok(())
}
fn config(root: &std::path::Path, enabled: bool) -> NetConfig {
    NetConfig {
        tls_enabled: enabled,
        tls_dir: root.join("server/tls"),
        identity_name: "lifecycle-server".into(),
        ..NetConfig::default()
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(std::env::args().nth(1).ok_or("lifecycle DIR")?);
    let server = Identity::load_or_create(&root.join("server"), "lifecycle-server")?;
    let client = Identity::load_or_create(&root.join("client"), "lifecycle-client")?;
    TrustStore::load(&root.join("server/tls"))?.trust_client(client.fingerprint)?;
    TrustStore::load(&root.join("client/tls"))?.trust_server(server.fingerprint)?;
    let (mut driver, handle) = NetDriver::new(config(&root, true))?;
    driver.set_transport_mode(TransportMode::Frames)?;
    let original_waker = driver.waker();
    cycle(&mut driver, &root, 24974, false)?;
    driver.reconfigure(config(&root, false))?;
    if !Arc::ptr_eq(&original_waker, &driver.waker()) {
        return Err("waker changed".into());
    }
    // The original cross-thread handle also remains live after reset.
    handle.send(NetCommand::StopListening)?;
    driver.poll(Some(Duration::ZERO), &mut Vec::new())?;
    cycle(&mut driver, &root, 24975, true)?;
    println!("REUSE same_driver=PASS same_waker=PASS original_handle=PASS TLS_on_to_off=PASS");
    Ok(())
}
