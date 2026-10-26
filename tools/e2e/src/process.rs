use crate::model::{Failure, Result};
use crate::util::{private_dir, write_private};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

pub struct Process {
    pub child: Child,
    pub group: u32,
    pub stderr: PathBuf,
    pub stdout: PathBuf,
    pub status: Option<ExitStatus>,
    terminated: bool,
}
impl Process {
    pub fn spawn(command: &mut Command, dir: &Path, label: &str) -> Result<Self> {
        private_dir(dir)?;
        let stdout = dir.join(format!("{label}.stdout"));
        let stderr = dir.join(format!("{label}.stderr"));
        write_private(&stdout, &[])?;
        write_private(&stderr, &[])?;
        command
            .stdin(Stdio::null())
            .stdout(std::fs::OpenOptions::new().append(true).open(&stdout)?)
            .stderr(std::fs::OpenOptions::new().append(true).open(&stderr)?);
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        let child = command
            .spawn()
            .map_err(|e| Failure::environment(format!("cannot start {label}: {e}")))?;
        let group = child.id();
        let process = Self {
            child,
            group,
            stderr,
            stdout,
            status: None,
            terminated: false,
        };
        crate::guardian::process(group, true)?;
        Ok(process)
    }
    pub fn poll(&mut self) -> Result<Option<ExitStatus>> {
        if self.status.is_none() {
            self.status = self.child.try_wait()?;
        }
        Ok(self.status)
    }
    pub fn wait(&mut self, timeout: Duration) -> Result<ExitStatus> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(status) = self.poll()? {
                return Ok(status);
            }
            if Instant::now() >= deadline {
                return Err(Failure::environment("child command timed out"));
            }
            std::thread::sleep(Duration::from_millis(15));
        }
    }
    pub fn signal(&self, signal: &str) -> Result<()> {
        signal_group(self.group, signal)
    }
    pub fn terminate(&mut self) -> Result<()> {
        if self.terminated {
            return Ok(());
        }
        // Cleanup is called explicitly and again by Drop. Never signal a stale
        // numeric process group a second time after its ownership has ended.
        self.terminated = true;
        let _ = self.signal("CONT");
        let _ = self.signal("TERM");
        let deadline = Instant::now() + Duration::from_millis(600);
        while Instant::now() < deadline {
            if self.poll().ok().flatten().is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        // Sweep the group even if its leader exited: grandchildren can still be alive.
        let killed = self.signal("KILL");
        self.wait(Duration::from_secs(2))
            .map_err(|_| Failure::environment("owned child did not exit during cleanup"))?;
        // An absent group is normal once the child has already exited.
        if killed.is_err() && signal_group(self.group, "0").is_ok() {
            return Err(Failure::environment("cannot kill the remaining owned process group"));
        }
        crate::guardian::process(self.group, false)?;
        Ok(())
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}
pub fn signal_group(group: u32, signal: &str) -> Result<()> {
    if group < 2 || !["0", "TERM", "KILL", "STOP", "CONT", "INT", "HUP"].contains(&signal) {
        return Err(Failure::runner("invalid owned process group or signal"));
    }
    #[cfg(unix)]
    {
        let status = Command::new("/bin/kill")
            .arg(format!("-{signal}"))
            .arg("--")
            .arg(format!("-{group}"))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        if !status.success() {
            return Err(Failure::environment(format!(
                "signal {signal} failed for owned process group"
            )));
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = (group, signal);
        Err(Failure::environment("process-group signals require Unix"))
    }
}
pub fn command_output(command: &mut Command, dir: &Path, label: &str, timeout: Duration) -> Result<(i32, String)> {
    let mut process = Process::spawn(command, dir, label)?;
    let status = process.wait(timeout)?;
    process.terminate()?;
    let file = std::fs::File::open(&process.stdout)?;
    use std::io::Read;
    let mut stdout = String::new();
    file.take(64 * 1024 * 1024 + 1).read_to_string(&mut stdout)?;
    if stdout.len() > 64 * 1024 * 1024 {
        return Err(Failure::runner("child output exceeds 64 MiB capture limit"));
    }
    write_private(
        &process.stdout,
        format!(
            "captured internally: {} bytes, sha256={}\n",
            stdout.len(),
            crate::util::digest(stdout.as_bytes())
        )
        .as_bytes(),
    )?;
    Ok((status.code().unwrap_or(-1), stdout))
}

/// A real short-lived parent, in the same owned process group as its daemon.
/// The gate file lets S20 establish a live connection before the parent exits.
pub fn parent_main(args: &[String]) -> Result<()> {
    if args.len() != 6 {
        return Err(Failure::runner("invalid internal parent arguments"));
    }
    let mut watcher = Command::new(std::env::current_exe()?)
        .arg("__watch")
        .args(&args[..4])
        .arg(Path::new(&args[3]).with_file_name("daemon.exit"))
        .arg(std::process::id().to_string())
        .stdin(Stdio::null())
        .spawn()?;
    let deadline = Instant::now() + Duration::from_secs(120);
    loop {
        if Path::new(&args[4]).exists() {
            return Ok(());
        }
        if watcher.try_wait()?.is_some() || Instant::now() > deadline {
            let _ = watcher.kill();
            let _ = watcher.wait();
            return Err(Failure::product("parent gate timed out or daemon exited early"));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}
pub fn watch_main(args: &[String]) -> Result<()> {
    if args.len() != 6 {
        return Err(Failure::runner("invalid watcher arguments"));
    }
    let mut daemon = Command::new(&args[0])
        .args([
            "daemon",
            "--platform",
            "virtual",
            "--foreground",
            "--config",
            &args[1],
            "--socket",
            &args[2],
            "--parent-pid",
            &args[5],
        ])
        .stdin(Stdio::null())
        .spawn()?;
    write_private(Path::new(&args[3]), daemon.id().to_string().as_bytes())?;
    let code = daemon.wait()?.code().unwrap_or(-1);
    write_private(Path::new(&args[4]), code.to_string().as_bytes())?;
    Ok(())
}
