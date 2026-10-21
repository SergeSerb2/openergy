//! Handlers dispatched by the real `openergy` binary.
use crate::{synergy3::read_text, *};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

fn options(args: &[String], allowed: &[&str], json: bool) -> Result<(BTreeMap<String, String>, bool), String> {
    let mut values = BTreeMap::new();
    let mut as_json = false;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--json" && json {
            if as_json {
                return Err("duplicate --json".into());
            }
            as_json = true;
            continue;
        }
        let (name, value) = if let Some((name, value)) = arg.split_once('=') {
            (name, value.to_string())
        } else {
            if !allowed.contains(&arg.as_str()) {
                return Err(format!("unknown option {arg:?}"));
            }
            (
                arg.as_str(),
                iter.next()
                    .filter(|s| !s.starts_with("--"))
                    .ok_or_else(|| format!("{arg} requires a value"))?
                    .clone(),
            )
        };
        if !allowed.contains(&name) {
            return Err(format!("unknown option {name:?}"));
        }
        if value.is_empty() || values.insert(name.into(), value).is_some() {
            return Err(format!("empty or duplicate option {name}"));
        }
    }
    Ok((values, as_json))
}
fn home() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| "home directory is unavailable; supply an explicit path".into())
}
fn default_config() -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        Ok(home()?.join("Library/Application Support/Openergy/config.toml"))
    }
    #[cfg(target_os = "windows")]
    {
        Ok(std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .ok_or("APPDATA is unavailable; use --config")?
            .join("Openergy/config.toml"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let base = match std::env::var_os("XDG_CONFIG_HOME") {
            Some(path) => PathBuf::from(path),
            None => home()?.join(".config"),
        };
        Ok(base.join("openergy/config.toml"))
    }
}

/// Persist one complete document through a same-directory, exclusive 0600 temp file.
/// On failure the old destination remains intact and our temp file is removed.
pub fn write_atomic(path: &Path, text: &str) -> Result<(), ConfigError> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent).map_err(|e| ConfigError::Io(e.to_string()))?;
    let name = path
        .file_name()
        .ok_or_else(|| ConfigError::Io("output path must name a file".into()))?;
    for _ in 0..32 {
        let mut tmp_name = name.to_os_string();
        tmp_name.push(format!(
            ".tmp.{}.{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let tmp = parent.join(tmp_name);
        let mut open = OpenOptions::new();
        open.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            open.mode(0o600);
        }
        let mut file = match open.open(&tmp) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(ConfigError::Io(e.to_string())),
        };
        let result = (|| {
            file.write_all(text.as_bytes())?;
            file.sync_all()?;
            drop(file);
            fs::rename(&tmp, path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&tmp);
        }
        result.map_err(|e: std::io::Error| ConfigError::Io(e.to_string()))?;
        return Ok(());
    }
    Err(ConfigError::Io("could not reserve an output temp file".into()))
}

pub fn import_synergy(args: &[String]) -> Result<i32, String> {
    if args.iter().any(|s| s == "--help" || s == "-h") {
        println!("openergy import-synergy [--from <dir-or-synergy.conf>] [--out <config.toml>] [--json]");
        return Ok(0);
    }
    let (opts, as_json) = options(args, &["--from", "--out"], true)?;
    let from = opts
        .get("--from")
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or_else(|| home().map(|p| p.join("Library/Preferences/Synergy")))?;
    let (cfg, report) = import_synergy_dir(&from).map_err(|e| e.to_string())?;
    if let Some(path) = opts.get("--out") {
        write_atomic(Path::new(path), &cfg.to_toml()).map_err(|e| e.to_string())?;
    }
    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &serde_json::json!({"config":cfg.to_json(),"derived_links":derive_links(&cfg.screens),"report":report})
            )
            .map_err(|e| e.to_string())?
        );
    } else {
        println!(
            "Imported {} screens, {} directed links; role {}; {} trust candidates.",
            cfg.screens.len(),
            derive_links(&cfg.screens).len(),
            cfg.role.as_str(),
            report.trusted_servers.len()
        );
        for warning in &report.warnings {
            eprintln!("warning: {warning}");
        }
        if let Some(path) = opts.get("--out") {
            println!("Wrote {path}");
        } else {
            println!("{}", cfg.to_toml());
        }
    }
    Ok(0)
}
pub fn export_synergy_conf(args: &[String]) -> Result<i32, String> {
    if args.iter().any(|s| s == "--help" || s == "-h") {
        println!("openergy export-synergy-conf [--config <path>] [--out <file>]");
        return Ok(0);
    }
    let (opts, _) = options(args, &["--config", "--out"], false)?;
    let path = opts
        .get("--config")
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or_else(default_config)?;
    let cfg = Config::load_toml(&read_text(&path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let text = crate::export_synergy_conf(&cfg).map_err(|e| e.to_string())?;
    if !cfg.clipboard.send || !cfg.clipboard.receive {
        eprintln!("warning: synergy.conf cannot represent directional clipboard send/receive permissions");
    }
    for warning in cfg.warnings() {
        eprintln!("warning: {warning}");
    }
    // The core language cannot carry these local/platform/transport settings.
    eprintln!("warning: synergy.conf exports screen/link/core options only; role, remote host, TLS, local input settings and physical rects are not represented");
    if let Some(path) = opts.get("--out") {
        write_atomic(Path::new(path), &text).map_err(|e| e.to_string())?;
    } else {
        print!("{text}");
    }
    Ok(0)
}
