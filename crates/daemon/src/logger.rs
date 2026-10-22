use openergy_ipc::Event;
use serde_json::json;
use std::{
    collections::VecDeque,
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
struct Inner {
    ring: VecDeque<String>,
    file: Option<File>,
    bytes: u64,
}
pub struct Logger {
    inner: Mutex<Inner>,
    path: PathBuf,
    level: log::LevelFilter,
    bus: Arc<crate::events::EventBus>,
}
impl Logger {
    fn open(path: &Path, truncate: bool) -> std::io::Result<File> {
        let mut options = OpenOptions::new();
        options.create(true).write(true).append(!truncate);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
        }
        let file = options.open(path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::{MetadataExt, PermissionsExt};
            let metadata = file.metadata()?;
            // Safety: geteuid only queries this process's effective user ID.
            if !metadata.is_file() || metadata.uid() != unsafe { libc::geteuid() } || metadata.nlink() != 1 {
                return Err(std::io::Error::other("log must be an owned regular file with one link"));
            }
            file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        }
        if truncate {
            file.set_len(0)?;
        }
        Ok(file)
    }
    pub fn new(dir: &Path, bus: Arc<crate::events::EventBus>) -> Arc<Self> {
        let path = dir.join("openergy.log");
        let file = Self::open(&path, false).ok();
        let bytes = file
            .as_ref()
            .and_then(|f| f.metadata().ok())
            .map(|m| m.len())
            .unwrap_or(0);
        let level = std::env::var("OPENERGY_LOG")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(log::LevelFilter::Info);
        Arc::new(Self {
            inner: Mutex::new(Inner {
                ring: VecDeque::with_capacity(1000),
                file,
                bytes,
            }),
            path,
            level,
            bus,
        })
    }
    pub fn install(self: &Arc<Self>) {
        let _ = log::set_boxed_logger(Box::new(SharedLogger(self.clone())));
        log::set_max_level(self.level);
    }
    pub fn tail(&self, n: usize) -> Vec<String> {
        self.inner
            .lock()
            .map(|i| {
                i.ring
                    .iter()
                    .skip(i.ring.len().saturating_sub(n.min(1000)))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
    fn append(&self, line: String) {
        eprintln!("{line}");
        let Ok(mut i) = self.inner.lock() else { return };
        if i.bytes + line.len() as u64 + 1 > 2 * 1024 * 1024 {
            i.file = None;
            let _ = std::fs::rename(self.path.with_extension("log.1"), self.path.with_extension("log.2"));
            let _ = std::fs::rename(&self.path, self.path.with_extension("log.1"));
            i.file = Self::open(&self.path, true).ok();
            i.bytes = 0;
        }
        if let Some(f) = &mut i.file {
            let _ = writeln!(f, "{line}");
        }
        i.bytes += line.len() as u64 + 1;
        if i.ring.len() == 1000 {
            i.ring.pop_front();
        }
        i.ring.push_back(line.clone());
        let event = Event {
            event: "log.line".into(),
            data: json!({"line":line}),
        };
        drop(i);
        self.bus.publish(event);
    }
}
struct SharedLogger(Arc<Logger>);
impl log::Log for SharedLogger {
    fn enabled(&self, m: &log::Metadata<'_>) -> bool {
        m.level() <= self.0.level
    }
    fn log(&self, r: &log::Record<'_>) {
        if self.enabled(r.metadata()) {
            self.0
                .append(format!("{} {} {}: {}", unix_ms(), r.level(), r.target(), r.args()));
        }
    }
    fn flush(&self) {
        if let Ok(mut i) = self.0.inner.lock() {
            if let Some(f) = &mut i.file {
                let _ = f.flush();
            }
        }
    }
}
