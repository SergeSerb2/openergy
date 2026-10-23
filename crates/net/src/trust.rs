use crate::{
    identity::{atomic_write, private_dir, private_file, DirectoryLock},
    Fingerprint, TrustError, TrustStore,
};
use std::{
    fs,
    io::{self, Read},
    path::Path,
};

// Approximately 14,000 canonical SHA-256 pins per private or imported list.
const MAX_TRUST_FILE_LEN: u64 = 1024 * 1024;
const LINEAR_TRUST_PINS: usize = 32;

/// Keep first-seen order while avoiding one scan of all prior pins per record.
fn dedup_in_order(values: &mut Vec<Fingerprint>) {
    let mut order: Vec<_> = (0..values.len()).collect();
    order.sort_unstable_by_key(|&i| (values[i].0, i));
    let mut duplicate = vec![false; values.len()];
    for pair in order.windows(2) {
        if values[pair[0]] == values[pair[1]] {
            duplicate[pair[1]] = true;
        }
    }
    let mut index = 0;
    values.retain(|_| {
        let keep = !duplicate[index];
        index += 1;
        keep
    });
}

fn fail(e: impl std::fmt::Display) -> TrustError {
    TrustError::Failed(e.to_string())
}
fn too_large() -> TrustError {
    fail("trust list exceeds 1 MiB (1048576 bytes)")
}
fn read(path: &Path, private: bool) -> Result<Vec<Fingerprint>, TrustError> {
    let file = match if private {
        private_file(path)
    } else {
        // Imports are read-only, but opening a FIFO can wait forever before
        // there is a writer. Follow ordinary file links without accepting pipes
        // or devices as trust lists.
        fs::metadata(path).and_then(|metadata| {
            if !metadata.is_file() {
                return Err(io::Error::other("imported trust list must be a regular file"));
            }
            let mut options = fs::OpenOptions::new();
            options.read(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                // A link can change to a FIFO between metadata and open.
                options.custom_flags(libc::O_NONBLOCK);
            }
            let file = options.open(path)?;
            if !file.metadata()?.is_file() {
                return Err(io::Error::other("imported trust list must be a regular file"));
            }
            Ok(file)
        })
    } {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(fail(e)),
    };
    if file.metadata().map_err(fail)?.len() > MAX_TRUST_FILE_LEN {
        return Err(too_large());
    }
    let mut text = String::new();
    file.take(MAX_TRUST_FILE_LEN + 1)
        .read_to_string(&mut text)
        .map_err(fail)?;
    // Also reject growth after metadata instead of accepting a partial list.
    if text.len() as u64 > MAX_TRUST_FILE_LEN {
        return Err(too_large());
    }
    let mut result = Vec::new();
    let mut ignored = 0;
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if let Some(fp) = Fingerprint::parse(line) {
            if result.len() > LINEAR_TRUST_PINS || !result.contains(&fp) {
                result.push(fp);
            }
        } else {
            ignored += 1;
        }
    }
    if result.len() > LINEAR_TRUST_PINS {
        dedup_in_order(&mut result);
    }
    // SHA-1 records are read as legacy, non-matching records, never upgraded to SHA-256.
    if ignored > 0 {
        log::debug!("ignored {ignored} non-SHA256/invalid trust records");
    }
    Ok(result)
}
fn serialize(values: &[Fingerprint], revoking: bool) -> Result<String, TrustError> {
    let mut text: String = values.iter().map(|fp| format!("{}\n", fp.to_synergy_line())).collect();
    if revoking && text.len() as u64 > MAX_TRUST_FILE_LEN {
        // Bare SHA-256 lines are already accepted on read. A revoke must not
        // fail merely because canonical prefixes expand a valid compact list.
        text = values
            .iter()
            .map(|fp| format!("{}\n", fp.to_synergy_line().trim_start_matches("v2:sha256:")))
            .collect();
    }
    if text.len() as u64 > MAX_TRUST_FILE_LEN {
        return Err(too_large());
    }
    Ok(text)
}
impl TrustStore {
    /// `dir` is the TLS directory containing the two trust files.
    pub fn load(dir: &Path) -> Result<Self, TrustError> {
        if dir.try_exists().map_err(fail)? {
            private_dir(dir).map_err(fail)?;
        }
        Ok(Self {
            dir: dir.to_owned(),
            servers: read(&dir.join("trusted-servers"), true)?,
            clients: read(&dir.join("trusted-clients"), true)?,
        })
    }
    pub fn dir(&self) -> &Path {
        &self.dir
    }
    pub fn reload(&mut self) -> Result<(), TrustError> {
        *self = Self::load(&self.dir)?;
        Ok(())
    }
    pub fn is_trusted_server(&self, fp: &Fingerprint) -> bool {
        self.servers.contains(fp)
    }
    pub fn is_trusted_client(&self, fp: &Fingerprint) -> bool {
        self.clients.contains(fp)
    }
    pub fn trust_server(&mut self, fp: Fingerprint) -> Result<(), TrustError> {
        self.modify(&[fp], &[], None, true).map(|_| ())
    }
    pub fn trust_client(&mut self, fp: Fingerprint) -> Result<(), TrustError> {
        self.modify(&[], &[fp], None, true).map(|_| ())
    }
    /// Live IO owners must not wait for another process's trust-file lock.
    pub fn try_trust_server(&mut self, fp: Fingerprint) -> Result<(), TrustError> {
        self.modify(&[fp], &[], None, false).map(|_| ())
    }
    /// Nonblocking counterpart of `trust_client`; a busy store is unchanged.
    pub fn try_trust_client(&mut self, fp: Fingerprint) -> Result<(), TrustError> {
        self.modify(&[], &[fp], None, false).map(|_| ())
    }
    /// Preflight an offline approval for both roles before saving either list.
    pub fn trust_both(&mut self, fp: Fingerprint) -> Result<(), TrustError> {
        self.modify(&[fp], &[fp], None, true).map(|_| ())
    }
    pub fn revoke(&mut self, fp: &Fingerprint) -> Result<(), TrustError> {
        self.modify(&[], &[], Some(*fp), true).map(|_| ())
    }
    /// Nonblocking revocation for a live IO owner; callers can retry on busy.
    pub fn try_revoke(&mut self, fp: &Fingerprint) -> Result<(), TrustError> {
        self.modify(&[], &[], Some(*fp), false).map(|_| ())
    }
    pub fn servers(&self) -> &[Fingerprint] {
        &self.servers
    }
    pub fn clients(&self) -> &[Fingerprint] {
        &self.clients
    }
    fn modify(
        &mut self,
        add_servers: &[Fingerprint],
        add_clients: &[Fingerprint],
        remove: Option<Fingerprint>,
        wait: bool,
    ) -> Result<usize, TrustError> {
        private_dir(&self.dir).map_err(fail)?;
        let lock_path = self.dir.join(".trust-lock");
        let _lock = if wait {
            DirectoryLock::acquire(&lock_path)
        } else {
            DirectoryLock::try_acquire(&lock_path)
        }
        .map_err(fail)?;
        let mut next = Self::load(&self.dir)?;
        let servers = next.servers.clone();
        let clients = next.clients.clone();
        if let Some(fp) = remove {
            next.servers.retain(|v| *v != fp);
            next.clients.retain(|v| *v != fp);
        }
        let mut added = 0;
        for (list, additions) in [(&mut next.servers, add_servers), (&mut next.clients, add_clients)] {
            if additions.len() > LINEAR_TRUST_PINS {
                let before = list.len();
                list.extend_from_slice(additions);
                dedup_in_order(list);
                added += list.len() - before;
                continue;
            }
            for fp in additions {
                if !list.contains(fp) {
                    list.push(*fp);
                    added += 1;
                }
            }
        }
        // Preflight both complete lists before writing either. An oversized
        // import must not save a prefix of the requested approvals.
        let servers = (next.servers != servers)
            .then(|| serialize(&next.servers, remove.is_some()))
            .transpose()?;
        let clients = (next.clients != clients)
            .then(|| serialize(&next.clients, remove.is_some()))
            .transpose()?;
        if let Some(text) = servers {
            atomic_write(&self.dir.join("trusted-servers"), text.as_bytes(), false).map_err(fail)?;
        }
        if let Some(text) = clients {
            atomic_write(&self.dir.join("trusted-clients"), text.as_bytes(), false).map_err(fail)?;
        }
        // An unsuccessful save must not report an unpersisted pin as approved.
        *self = next;
        Ok(added)
    }
    /// Import the TLS directory supplied by the caller. Certificate/key files are untouched.
    pub fn import_synergy(&mut self, dir: &Path) -> Result<usize, TrustError> {
        // Imported files are read-only inputs; harden only our destination store.
        let other = Self {
            dir: dir.to_owned(),
            servers: read(&dir.join("trusted-servers"), false)?,
            clients: read(&dir.join("trusted-clients"), false)?,
        };
        self.modify(&other.servers, &other.clients, None, true)
    }
}
