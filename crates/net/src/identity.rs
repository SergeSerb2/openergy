use crate::{Fingerprint, NetError};
use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime},
};

const MAX_IDENTITY_FILE_LEN: u64 = 1024 * 1024;

pub fn default_config_dir() -> Result<PathBuf, NetError> {
    #[cfg(target_os = "macos")]
    let path = std::env::var_os("HOME").map(|p| PathBuf::from(p).join("Library/Application Support/Openergy"));
    #[cfg(target_os = "windows")]
    let path = std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("Openergy"));
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let path = std::env::var_os("XDG_CONFIG_HOME")
        .map(|p| PathBuf::from(p).join("openergy"))
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".config/openergy")));
    path.ok_or_else(|| NetError::Failed("cannot find config directory; supply --dir".into()))
}

pub(crate) fn failed(e: impl std::fmt::Display) -> NetError {
    NetError::Failed(e.to_string())
}

/// Stable ECDSA P-256 identity. Private key material is never included in Debug.
pub struct Identity {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    pub fingerprint: Fingerprint,
    pub(crate) chain: Vec<CertificateDer<'static>>,
    pub(crate) key: PrivateKeyDer<'static>,
}
impl Identity {
    /// Uses `<config_dir>/tls/{cert,key}.pem`. A partial/corrupt identity is an error,
    /// never a reason to replace an existing identity without the user's knowledge.
    pub fn load_or_create(config_dir: &Path, name: &str) -> Result<Self, NetError> {
        Self::load_or_create_in(&config_dir.join("tls"), name)
    }
    /// Uses exactly `tls_dir`, as required by NetConfig.
    pub fn load_or_create_in(tls_dir: &Path, name: &str) -> Result<Self, NetError> {
        Self::with_wait(tls_dir, name, true)
    }
    /// Nonblocking reader/creator for IO owners. A busy creator lock is an
    /// error, never permission to read a potentially incomplete key/cert pair.
    pub fn try_load_or_create_in(tls_dir: &Path, name: &str) -> Result<Self, NetError> {
        Self::with_wait(tls_dir, name, false)
    }
    fn with_wait(tls_dir: &Path, name: &str, wait: bool) -> Result<Self, NetError> {
        let dir = tls_dir.to_owned();
        private_dir(&dir).map_err(failed)?;
        let cert_path = dir.join("cert.pem");
        let key_path = dir.join("key.pem");
        // Readers must share the creator lock: key.pem is renamed before cert.pem.
        // Observing that in-progress pair must not report a corrupt identity.
        let lock_path = dir.join(".identity-lock");
        let _lock = if wait {
            DirectoryLock::acquire(&lock_path)
        } else {
            DirectoryLock::try_acquire(&lock_path)
        }
        .map_err(failed)?;
        if !entry_exists(&cert_path).map_err(failed)? && !entry_exists(&key_path).map_err(failed)? {
            let key = rcgen::KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256).map_err(failed)?;
            let mut params = rcgen::CertificateParams::default();
            params.distinguished_name = rcgen::DistinguishedName::new();
            params
                .distinguished_name
                .push(rcgen::DnType::CommonName, format!("Openergy {name}"));
            let now = SystemTime::now();
            params.not_before = (now - Duration::from_secs(300)).into();
            params.not_after = (now + Duration::from_secs(3650 * 86400)).into();
            let cert = params.self_signed(&key).map_err(failed)?;
            atomic_write(&key_path, key.serialize_pem().as_bytes(), true).map_err(failed)?;
            atomic_write(&cert_path, cert.pem().as_bytes(), true).map_err(failed)?;
        }
        let chain: Vec<_> = CertificateDer::pem_slice_iter(&read_identity_file(&cert_path)?)
            .collect::<Result<_, _>>()
            .map_err(failed)?;
        let cert = chain.first().ok_or_else(|| failed("identity certificate is empty"))?;
        let fingerprint = Fingerprint::from_der(cert.as_ref());
        let key = PrivateKeyDer::from_pem_slice(&read_identity_file(&key_path)?).map_err(failed)?;
        let certified = rustls::sign::CertifiedKey::from_der(
            chain.clone(),
            key.clone_key(),
            &rustls::crypto::ring::default_provider(),
        )
        .map_err(failed)?;
        certified.keys_match().map_err(failed)?;
        Ok(Self {
            cert_path,
            key_path,
            fingerprint,
            chain,
            key,
        })
    }
}

fn read_identity_file(path: &Path) -> Result<Vec<u8>, NetError> {
    let file = private_file(path).map_err(failed)?;
    let too_large = || failed("TLS identity file exceeds 1 MiB (1048576 bytes)");
    if file.metadata().map_err(failed)?.len() > MAX_IDENTITY_FILE_LEN {
        return Err(too_large());
    }
    let mut bytes = Vec::new();
    file.take(MAX_IDENTITY_FILE_LEN + 1)
        .read_to_end(&mut bytes)
        .map_err(failed)?;
    // Reject growth after metadata, including ignored bytes after the first key.
    if bytes.len() as u64 > MAX_IDENTITY_FILE_LEN {
        return Err(too_large());
    }
    Ok(bytes)
}

fn entry_exists(path: &Path) -> std::io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

/// Restrict the identity/trust directory before creating any file in it.
pub(crate) fn private_dir(path: &Path) -> std::io::Result<()> {
    fs::create_dir_all(path)?;
    if !fs::symlink_metadata(path)?.file_type().is_dir() {
        return Err(std::io::Error::other("TLS directory must be a real directory"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    #[cfg(windows)]
    crate::windows_permissions::restrict(path, true)?;
    #[cfg(not(any(unix, windows)))]
    return Err(std::io::Error::other(
        "private TLS storage is unsupported on this platform",
    ));
    #[cfg(any(unix, windows))]
    Ok(())
}

pub(crate) fn private_file(path: &Path) -> std::io::Result<File> {
    if !fs::symlink_metadata(path)?.file_type().is_file() {
        return Err(std::io::Error::other("TLS identity must be a regular file, not a link"));
    }
    #[cfg(windows)]
    crate::windows_permissions::restrict(path, false)?;
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(path)?;
    if !file.metadata()?.is_file() {
        return Err(std::io::Error::other("TLS identity must be a regular file, not a link"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))?;
    }
    Ok(file)
}

/// The kernel releases this advisory lock even when the process is killed.
pub(crate) struct DirectoryLock {
    _file: File,
}
impl DirectoryLock {
    pub(crate) fn acquire(path: &Path) -> std::io::Result<Self> {
        Self::with_wait(path, true)
    }
    pub(crate) fn try_acquire(path: &Path) -> std::io::Result<Self> {
        Self::with_wait(path, false)
    }
    fn with_wait(path: &Path, wait: bool) -> std::io::Result<Self> {
        // Migrate the old empty directory sentinel without deleting user data.
        if entry_exists(path)? {
            let kind = fs::symlink_metadata(path)?.file_type();
            if kind.is_dir() {
                fs::remove_dir(path)?;
            } else if !kind.is_file() {
                return Err(std::io::Error::other("TLS lock must be a regular file"));
            }
        }
        #[cfg(windows)]
        let file = crate::windows_permissions::open(path, false)?;
        #[cfg(not(windows))]
        let file = {
            let mut options = OpenOptions::new();
            options.read(true).write(true).create(true).truncate(false);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
            }
            options.open(path)?
        };
        if !file.metadata()?.is_file() {
            return Err(std::io::Error::other("TLS lock must be a regular file"));
        }
        if wait {
            file.lock()?;
        } else {
            file.try_lock().map_err(|error| match error {
                std::fs::TryLockError::WouldBlock => std::io::Error::new(
                    std::io::ErrorKind::WouldBlock,
                    "TLS files are busy; retry the operation",
                ),
                std::fs::TryLockError::Error(error) => error,
            })?;
        }
        Ok(Self { _file: file })
    }
}

pub(crate) fn atomic_write(path: &Path, bytes: &[u8], private: bool) -> std::io::Result<()> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let parent = path
        .parent()
        .ok_or_else(|| std::io::Error::other("file has no parent"))?;
    let tmp = parent.join(format!(
        ".oe-{}-{}.tmp",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let mut created = false;
    let result = (|| {
        #[cfg(not(unix))]
        let _ = private;
        #[cfg(windows)]
        let mut file = crate::windows_permissions::open(&tmp, true)?;
        #[cfg(not(windows))]
        let mut file = {
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(if private { 0o600 } else { 0o644 });
            }
            options.open(&tmp)?
        };
        created = true;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&tmp, path)?;
        #[cfg(unix)]
        File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() && created {
        let _ = fs::remove_file(tmp);
    }
    result
}
