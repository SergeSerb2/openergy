use std::sync::atomic::{AtomicBool, Ordering};
pub static SIGNALLED: AtomicBool = AtomicBool::new(false);
extern "C" fn signal(_sig: libc::c_int) {
    SIGNALLED.store(true, Ordering::Relaxed);
}
pub fn install_signals() {
    #[cfg(unix)]
    // Safety: handlers only store a lock-free atomic; no allocation or non-signal-safe calls occur.
    unsafe {
        libc::signal(libc::SIGINT, signal as *const () as usize);
        libc::signal(libc::SIGTERM, signal as *const () as usize);
        libc::signal(libc::SIGPIPE, libc::SIG_IGN);
    }
}
pub struct ParentWatch {
    pid: Option<libc::pid_t>,
    #[cfg(target_os = "macos")]
    kq: libc::c_int,
}
impl ParentWatch {
    pub fn new(pid: Option<i64>) -> Self {
        #[cfg(target_os = "macos")]
        {
            let mut kq = -1;
            if let Some(pid) = pid {
                // Safety: kqueue creates an owned descriptor; kevent receives one fully initialized event.
                unsafe {
                    kq = libc::kqueue();
                    if kq >= 0 {
                        let event = libc::kevent {
                            ident: pid as libc::uintptr_t,
                            filter: libc::EVFILT_PROC,
                            flags: libc::EV_ADD | libc::EV_ONESHOT,
                            fflags: libc::NOTE_EXIT,
                            data: 0,
                            udata: std::ptr::null_mut(),
                        };
                        if libc::kevent(kq, &event, 1, std::ptr::null_mut(), 0, std::ptr::null()) < 0 {
                            libc::close(kq);
                            kq = -1;
                        }
                    }
                }
            }
            Self {
                pid: pid.map(|p| p as libc::pid_t),
                kq,
            }
        }
        #[cfg(not(target_os = "macos"))]
        Self {
            pid: pid.map(|p| p as libc::pid_t),
        }
    }
    pub fn exited(&self) -> bool {
        let Some(pid) = self.pid else { return false };
        #[cfg(target_os = "macos")]
        if self.kq >= 0 {
            let timeout = libc::timespec { tv_sec: 0, tv_nsec: 0 };
            // Safety: zeroed kevent is an output buffer, the timeout is zero, and the descriptor remains owned.
            let ready = unsafe {
                let mut event: libc::kevent = std::mem::zeroed();
                libc::kevent(self.kq, std::ptr::null(), 0, &mut event, 1, &timeout) > 0
            };
            if ready {
                return true;
            }
        }
        #[cfg(unix)]
        // Safety: kill(pid, 0) checks liveness without sending a signal.
        {
            (unsafe { libc::kill(pid, 0) }) < 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
        }
        #[cfg(not(unix))]
        {
            let _ = pid;
            false
        }
    }
}
#[cfg(target_os = "macos")]
impl Drop for ParentWatch {
    fn drop(&mut self) {
        if self.kq >= 0 {
            // Safety: this descriptor was created and is owned by ParentWatch.
            unsafe {
                libc::close(self.kq);
            }
        }
    }
}

pub fn rss_bytes() -> Option<u64> {
    #[cfg(target_os = "macos")]
    {
        let mut info = std::mem::MaybeUninit::<libc::rusage_info_v2>::uninit();
        // Safety: proc_pid_rusage writes the documented v2 structure into a correctly sized output buffer.
        let result = unsafe {
            libc::proc_pid_rusage(
                std::process::id() as i32,
                libc::RUSAGE_INFO_V2,
                info.as_mut_ptr().cast(),
            )
        };
        if result == 0 {
            // Safety: a successful proc_pid_rusage initializes the entire output structure.
            return Some(unsafe { info.assume_init() }.ri_resident_size);
        }
        None
    }
    #[cfg(target_os = "linux")]
    {
        let pages = std::fs::read_to_string("/proc/self/statm")
            .ok()?
            .split_whitespace()
            .nth(1)?
            .parse::<u64>()
            .ok()?;
        // Safety: sysconf queries the immutable system page size.
        let size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
        u64::try_from(size).ok().map(|size| pages * size)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}
