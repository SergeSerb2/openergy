//! Offline identity/trust commands. The daemon reloads trust files at most 500 ms
//! after a change while holding a peer. Live approve/reject is daemon IPC's job.
use crate::{default_config_dir, Fingerprint, Identity, TrustStore};
use std::path::PathBuf;

fn arguments(args: &[String]) -> Result<(PathBuf, Vec<String>), String> {
    let mut dir = None;
    let mut rest = Vec::new();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if arg == "--dir" || arg == "--config" {
            if dir.is_some() {
                return Err("--dir may appear only once".into());
            }
            let value = args
                .next()
                .filter(|value| !value.is_empty() && !value.starts_with("--"))
                .ok_or("--dir requires a directory")?;
            let path = PathBuf::from(value);
            dir = Some(if arg == "--config" {
                path.parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .unwrap_or_else(|| std::path::Path::new("."))
                    .to_owned()
            } else {
                path
            });
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
fn usage_error(message: &str) -> Result<i32, String> {
    eprintln!("openergy: {message}");
    Ok(2)
}
pub fn fingerprint(args: &[String]) -> Result<i32, String> {
    let (dir, rest) = match arguments(args) {
        Ok(args) => args,
        Err(message) => return usage_error(&message),
    };
    let synergy = match rest.as_slice() {
        [] => false,
        [flag] if flag == "--synergy" => true,
        _ => return usage_error("usage: openergy fingerprint [--dir D] [--synergy]"),
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
    let (dir, rest) = match arguments(args) {
        Ok(args) => args,
        Err(message) => return usage_error(&message),
    };
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
            return usage_error(
                "usage: openergy trust list|approve <fp>|revoke <fp>|import-synergy <tls-dir> [--dir D]",
            )
        }
    }
    Ok(0)
}
