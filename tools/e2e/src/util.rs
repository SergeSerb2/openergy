use crate::model::{Failure, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::{Read, Seek, SeekFrom};
use std::net::{Ipv4Addr, TcpListener};
use std::path::Path;

pub fn port_range(value: &str) -> Result<(u16, u16)> {
    let (a, b) = value
        .split_once('-')
        .ok_or_else(|| Failure::runner("port range must be A-B"))?;
    let a = a.parse::<u16>().map_err(|_| Failure::runner("invalid first port"))?;
    let b = b.parse::<u16>().map_err(|_| Failure::runner("invalid last port"))?;
    if a < 1024 || a > b || (a..=b).contains(&24800) || b - a > 100 {
        return Err(Failure::runner(
            "range must have 1..101 unprivileged ports and exclude 24800",
        ));
    }
    Ok((a, b))
}
pub fn reserve(range: (u16, u16), used: &[u16], requested: Option<u16>) -> Result<TcpListener> {
    for port in range.0..=range.1 {
        if used.contains(&port) || requested.is_some_and(|p| p != port) {
            continue;
        }
        if let Ok(listener) = TcpListener::bind((Ipv4Addr::LOCALHOST, port)) {
            return Ok(listener);
        }
    }
    Err(Failure::environment("no free port in configured range"))
}
pub fn merge(target: &mut Value, patch: &Value) {
    if let Some(map) = patch.as_object() {
        if !target.is_object() {
            *target = json!({});
        }
        if let Some(object) = target.as_object_mut() {
            for (key, value) in map {
                if value.is_null() {
                    object.remove(key);
                } else {
                    merge(object.entry(key).or_insert(Value::Null), value);
                }
            }
        }
    } else {
        *target = patch.clone();
    }
}
pub fn lookup<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    if path.starts_with('/') {
        return value.pointer(path);
    }
    if path.is_empty() {
        return Some(value);
    }
    path.split('.').try_fold(value, |v, key| {
        if let Some(items) = v.as_array() {
            key.parse::<usize>().ok().and_then(|i| items.get(i))
        } else {
            v.get(key)
        }
    })
}
/// Object expectations are subsets. Arrays preserve order and length.
pub fn matches(actual: &Value, expected: &Value) -> bool {
    match expected {
        Value::Object(map) => map
            .iter()
            .all(|(key, value)| actual.get(key).is_some_and(|a| matches(a, value))),
        Value::Array(items) => actual
            .as_array()
            .is_some_and(|a| a.len() == items.len() && a.iter().zip(items).all(|(a, e)| matches(a, e))),
        _ => actual == expected,
    }
}
pub fn predicate(actual: Option<&Value>, expected: &Value) -> bool {
    if let Some(ops) = expected.as_object().filter(|m| m.keys().any(|k| k.starts_with('$'))) {
        return ops.iter().all(|(op, rhs)| match op.as_str() {
            "$null" => actual.is_some_and(Value::is_null) == rhs.as_bool().unwrap_or(true),
            "$exists" => actual.is_some() == rhs.as_bool().unwrap_or(true),
            "$ne" => actual != Some(rhs),
            "$gte" => actual
                .and_then(Value::as_f64)
                .zip(rhs.as_f64())
                .is_some_and(|(a, b)| a >= b),
            "$lte" => actual
                .and_then(Value::as_f64)
                .zip(rhs.as_f64())
                .is_some_and(|(a, b)| a <= b),
            "$contains" => actual.is_some_and(|a| match a {
                Value::Array(items) => items.iter().any(|a| matches(a, rhs)),
                Value::String(s) => rhs.as_str().is_some_and(|r| s.contains(r)),
                _ => false,
            }),
            "$len" => actual
                .and_then(|v| {
                    v.as_array()
                        .map(Vec::len)
                        .or_else(|| v.as_object().map(serde_json::Map::len))
                })
                .is_some_and(|n| rhs.as_u64() == Some(n as u64)),
            _ => false,
        });
    }
    actual.is_some_and(|a| matches(a, expected))
}
pub fn fields_match(actual: &Value, expected: &Value) -> Result<bool> {
    let map = expected
        .as_object()
        .ok_or_else(|| Failure::runner("fields must be a table of field paths and predicates"))?;
    Ok(map.iter().all(|(path, expect)| predicate(lookup(actual, path), expect)))
}
pub fn glob(pattern: &str, text: &str) -> bool {
    let (p, t) = (pattern.as_bytes(), text.as_bytes());
    let (mut i, mut j, mut star, mut retry) = (0, 0, None, 0);
    while j < t.len() {
        if i < p.len() && (p[i] == b'?' || p[i] == t[j]) {
            i += 1;
            j += 1;
        } else if i < p.len() && p[i] == b'*' {
            star = Some(i);
            i += 1;
            retry = j;
        } else if let Some(s) = star {
            retry += 1;
            j = retry;
            i = s + 1;
        } else {
            return false;
        }
    }
    while i < p.len() && p[i] == b'*' {
        i += 1;
    }
    i == p.len()
}
pub fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}
pub fn hex(text: &str) -> Result<Vec<u8>> {
    let compact: String = text.chars().filter(|c| !c.is_whitespace() && *c != '_').collect();
    if !compact.len().is_multiple_of(2) || !compact.is_ascii() || compact.len() > 32 * 1024 * 1024 {
        return Err(Failure::runner("invalid or oversized hex payload"));
    }
    compact
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let hi = (pair[0] as char).to_digit(16);
            let lo = (pair[1] as char).to_digit(16);
            hi.zip(lo)
                .map(|(a, b)| (a * 16 + b) as u8)
                .ok_or_else(|| Failure::runner("invalid hex digit"))
        })
        .collect()
}
pub fn summary(value: &Value) -> Value {
    match value {
        Value::Array(items) if items.len() > 128 => {
            json!({"items":items.len(),"tail":items[items.len()-128..].iter().map(summary).collect::<Vec<_>>()})
        }
        Value::Array(items) => Value::Array(items.iter().map(summary).collect()),
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(k, v)| {
                    let secret = matches!(
                        k.as_str(),
                        "events" | "event" | "bytes" | "text" | "payload" | "key" | "button" | "formats"
                    );
                    (
                        k.clone(),
                        if k == "event" && v.is_object() {
                            json!({"type":v.get("type")})
                        } else if secret {
                            json!("redacted")
                        } else {
                            summary(v)
                        },
                    )
                })
                .collect(),
        ),
        Value::String(s) if s.len() > 256 => json!({"chars": s.len(), "content": "redacted"}),
        _ => value.clone(),
    }
}
pub fn tail(path: &Path) -> Vec<String> {
    let Ok(mut file) = std::fs::File::open(path) else {
        return Vec::new();
    };
    let len = file.metadata().map(|m| m.len()).unwrap_or(0);
    let _ = file.seek(SeekFrom::Start(len.saturating_sub(8192)));
    let mut bytes = Vec::new();
    let _ = file.take(8192).read_to_end(&mut bytes);
    String::from_utf8_lossy(&bytes)
        .lines()
        .rev()
        .take(12)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|line| {
            let lower = line.to_ascii_lowercase();
            if [
                "key",
                "clipboard",
                "payload",
                "dkdn",
                "dkdl",
                "dkup",
                "dclp",
                "button",
                "motion",
            ]
            .iter()
            .any(|k| lower.contains(k))
            {
                "[input or clipboard log line redacted]".into()
            } else {
                line.chars().take(400).collect()
            }
        })
        .collect()
}
pub fn private_dir(path: &Path) -> Result<()> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path)?;
    Ok(())
}
pub fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        private_dir(parent)?;
    }
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;
    let mut options = std::fs::OpenOptions::new();
    options.create(true).truncate(true).write(true);
    #[cfg(unix)]
    options.mode(0o600);
    use std::io::Write;
    options.open(path)?.write_all(bytes)?;
    Ok(())
}

pub fn check_deadline(deadline: std::time::Instant, label: &str) -> Result<()> {
    if std::time::Instant::now() >= deadline {
        return Err(Failure::product(format!("{label} absolute deadline expired")));
    }
    Ok(())
}
pub fn wait_io(deadline: std::time::Instant, label: &str) -> Result<()> {
    check_deadline(deadline, label)?;
    std::thread::sleep(
        deadline
            .saturating_duration_since(std::time::Instant::now())
            .min(std::time::Duration::from_micros(250)),
    );
    Ok(())
}

/// Keep third-party diagnostics without retaining synthetic key values.
pub fn sanitize_lab_logs(path: &Path) -> Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            sanitize_lab_logs(&entry.path())?;
        } else if entry.file_type()?.is_file() && entry.path().extension().is_some_and(|s| s == "log") {
            let bytes = std::fs::read(entry.path())?;
            let sanitized = if entry.file_name() == "events.log" {
                format!(
                    "X11 event capture redacted: bytes={}, sha256={}\n",
                    bytes.len(),
                    digest(&bytes)
                )
            } else {
                String::from_utf8_lossy(&bytes)
                    .lines()
                    .map(|line| {
                        let lower = line.to_ascii_lowercase();
                        if [
                            "key",
                            "clipboard",
                            "button",
                            "mouse",
                            "motion",
                            "dkdn",
                            "dkdl",
                            "dkup",
                            "dclp",
                        ]
                        .iter()
                        .any(|kind| lower.contains(kind))
                        {
                            "[input/clipboard diagnostic redacted]".to_owned()
                        } else {
                            line.to_owned()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            write_private(&entry.path(), sanitized.as_bytes())?;
        }
    }
    Ok(())
}
