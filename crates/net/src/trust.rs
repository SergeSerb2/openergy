use crate::{
    identity::{atomic_write, private_dir, private_file, DirectoryLock},
    Fingerprint, TrustError, TrustStore,
};
use std::{
    fs,
    io::{self, Read},
    path::Path,
};

fn fail(e: impl std::fmt::Display) -> TrustError {
    TrustError::Failed(e.to_string())
}
fn read(path: &Path, private: bool) -> Result<Vec<Fingerprint>, TrustError> {
    let mut file = match if private {
        private_file(path)
    } else {
        fs::File::open(path)
    } {
        Ok(file) => file,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(fail(e)),
    };
    let mut text = String::new();
    file.read_to_string(&mut text).map_err(fail)?;
    let mut result = Vec::new();
    let mut ignored = 0;
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if let Some(fp) = Fingerprint::parse(line) {
            if !result.contains(&fp) {
                result.push(fp);
            }
        } else {
            ignored += 1;
        }
    }
    // SHA-1 records are read as legacy, non-matching records, never upgraded to SHA-256.
    if ignored > 0 {
        log::debug!("ignored {ignored} non-SHA256/invalid trust records");
    }
    Ok(result)
}
fn write(path: &Path, values: &[Fingerprint]) -> Result<(), TrustError> {
    let text: String = values.iter().map(|fp| format!("{}\n", fp.to_synergy_line())).collect();
    atomic_write(path, text.as_bytes(), false).map_err(fail)
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
        self.modify(Some((true, fp)), None)
    }
    pub fn trust_client(&mut self, fp: Fingerprint) -> Result<(), TrustError> {
        self.modify(Some((false, fp)), None)
    }
    pub fn revoke(&mut self, fp: &Fingerprint) -> Result<(), TrustError> {
        self.modify(None, Some(*fp))
    }
    pub fn servers(&self) -> &[Fingerprint] {
        &self.servers
    }
    pub fn clients(&self) -> &[Fingerprint] {
        &self.clients
    }
    fn modify(&mut self, add: Option<(bool, Fingerprint)>, remove: Option<Fingerprint>) -> Result<(), TrustError> {
        private_dir(&self.dir).map_err(fail)?;
        let _lock = DirectoryLock::acquire(&self.dir.join(".trust-lock")).map_err(fail)?;
        let mut next = Self::load(&self.dir)?;
        let servers = next.servers.clone();
        let clients = next.clients.clone();
        if let Some(fp) = remove {
            next.servers.retain(|v| *v != fp);
            next.clients.retain(|v| *v != fp);
        }
        if let Some((server, fp)) = add {
            let list = if server { &mut next.servers } else { &mut next.clients };
            if !list.contains(&fp) {
                list.push(fp);
            }
        }
        if next.servers != servers {
            write(&self.dir.join("trusted-servers"), &next.servers)?;
        }
        if next.clients != clients {
            write(&self.dir.join("trusted-clients"), &next.clients)?;
        }
        // An unsuccessful save must not report an unpersisted pin as approved.
        *self = next;
        Ok(())
    }
    /// Import the TLS directory supplied by the caller. Certificate/key files are untouched.
    pub fn import_synergy(&mut self, dir: &Path) -> Result<usize, TrustError> {
        // Imported files are read-only inputs; harden only our destination store.
        let other = Self {
            dir: dir.to_owned(),
            servers: read(&dir.join("trusted-servers"), false)?,
            clients: read(&dir.join("trusted-clients"), false)?,
        };
        let mut added = 0;
        for fp in other.servers {
            if !self.is_trusted_server(&fp) {
                self.trust_server(fp)?;
                added += 1;
            }
        }
        for fp in other.clients {
            if !self.is_trusted_client(&fp) {
                self.trust_client(fp)?;
                added += 1;
            }
        }
        Ok(added)
    }
}
