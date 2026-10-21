//! Offline identity/trust commands. The daemon reloads trust files at most 500 ms
//! after a change while holding a peer. Live approve/reject is daemon IPC's job.
use crate::{default_config_dir, Fingerprint, Identity, TrustStore};
use std::path::PathBuf;

fn arguments(args: &[String]) -> Result<(PathBuf, Vec<String>), String> {
    let mut dir = None;
    let mut rest = Vec::new();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if arg == "--dir" {
            if dir.is_some() {
                return Err("--dir may appear only once".into());
            }
            dir = Some(PathBuf::from(args.next().ok_or("--dir requires a directory")?));
        } else {
            rest.push(arg.clone());
        }
    }
    Ok((
        match dir {
            Some(p) => p,
            None => default_config_dir().map_err(|e| e.to_string())?,
        },
        rest,
    ))
}
pub fn fingerprint(args: &[String]) -> Result<i32, String> {
    let (dir, rest) = arguments(args)?;
    let synergy = match rest.as_slice() {
        [] => false,
        [flag] if flag == "--synergy" => true,
        _ => return Err("usage: openergy fingerprint [--dir D] [--synergy]".into()),
    };
    let name = std::env::var("HOSTNAME").unwrap_or_else(|_| "computer".into());
    let identity = Identity::load_or_create(&dir, &name).map_err(|e| e.to_string())?;
    if synergy {
        println!("{}", identity.fingerprint.to_synergy_line());
    } else {
        println!("{}", identity.fingerprint);
        println!("certificate: {}", identity.cert_path.display());
        println!("private key: {}", identity.key_path.display());
    }
    Ok(0)
}
pub fn trust(args: &[String]) -> Result<i32, String> {
    let (dir, rest) = arguments(args)?;
    let mut store = TrustStore::load(&dir.join("tls")).map_err(|e| e.to_string())?;
    match rest.as_slice() {
        [command] if command == "list" => {
            for fp in store.servers() {
                println!("server {}", fp.to_synergy_line());
            }
            for fp in store.clients() {
                println!("client {}", fp.to_synergy_line());
            }
        }
        [command, value] if command == "approve" || command == "revoke" => {
            let fp = Fingerprint::parse(value).ok_or("expected a SHA-256 certificate fingerprint")?;
            if command == "approve" {
                // Without a live connection, no role is available. Explicit offline approval
                // trusts this identity in both roles. IPC approval knows and writes one role.
                store.trust_server(fp).map_err(|e| e.to_string())?;
                store.trust_client(fp).map_err(|e| e.to_string())?;
            } else {
                store.revoke(&fp).map_err(|e| e.to_string())?;
            }
        }
        [command, path] if command == "import-synergy" => {
            println!(
                "imported {} trust records",
                store.import_synergy(&PathBuf::from(path)).map_err(|e| e.to_string())?
            );
        }
        [command, _] if command == "reject" => {
            return Err("live rejection requires daemon IPC: openergy ctl trust.reject".into())
        }
        _ => {
            return Err("usage: openergy trust list|approve <fp>|revoke <fp>|import-synergy <tls-dir> [--dir D]".into())
        }
    }
    Ok(0)
}
