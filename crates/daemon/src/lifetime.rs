use std::sync::atomic::{AtomicBool, Ordering};
pub static SIGNALLED: AtomicBool = AtomicBool::new(false);
#[cfg(unix)]
extern "C" fn signal(_sig: libc::c_int) {
    SIGNALLED.store(true, Ordering::Relaxed);
}
#[cfg(windows)]
pub static SHUTDOWN_COMPLETE: AtomicBool = AtomicBool::new(false);
#[cfg(windows)]
unsafe extern "system" fn console_signal(event: u32) -> windows_sys::core::BOOL {
    use windows_sys::Win32::System::{Console::*, Threading::Sleep};
    if !matches!(
        event,
        CTRL_C_EVENT | CTRL_BREAK_EVENT | CTRL_CLOSE_EVENT | CTRL_LOGOFF_EVENT | CTRL_SHUTDOWN_EVENT
    ) {
        return 0;
    }
    SIGNALLED.store(true, Ordering::Relaxed);
    if matches!(event, CTRL_CLOSE_EVENT | CTRL_LOGOFF_EVENT | CTRL_SHUTDOWN_EVENT) {
        // Console-close terminates the process after this handler returns. Allow bounded cleanup first.
        for _ in 0..80 {
            if SHUTDOWN_COMPLETE.load(Ordering::Acquire) {
                break;
            }
            // Safety: Sleep only yields this handler thread for 50 ms.
            unsafe { Sleep(50) };
        }
    }
    1
}
pub fn install_signals() {
    #[cfg(windows)]
    {
        // Safety: clearing the inherited Ctrl+C-ignore flag changes only this process.
        unsafe { windows_sys::Win32::System::Console::SetConsoleCtrlHandler(None, 0) };
        // Safety: the static handler only sets atomics and performs bounded waits on console close.
        if unsafe { windows_sys::Win32::System::Console::SetConsoleCtrlHandler(Some(console_signal), 1) } == 0 {
            log::warn!("console handler unavailable: {}", std::io::Error::last_os_error());
        }
    }
    #[cfg(unix)]
    // Safety: handlers only store a lock-free atomic; no allocation or non-signal-safe calls occur.
    unsafe {
        libc::signal(libc::SIGINT, signal as *const () as usize);
        libc::signal(libc::SIGTERM, signal as *const () as usize);
        libc::signal(libc::SIGPIPE, libc::SIG_IGN);
    }
}
pub struct ParentWatch {
    pid: Option<i64>,
    #[cfg(windows)]
    process: Option<std::os::windows::io::OwnedHandle>,
    #[cfg(target_os = "linux")]
    process: Option<std::os::fd::OwnedFd>,
    #[cfg(target_os = "macos")]
    kq: libc::c_int,
}
impl ParentWatch {
    pub fn new(pid: Option<i64>) -> Self {
        #[cfg(windows)]
        {
            use std::os::windows::io::{FromRawHandle, OwnedHandle};
            let process = pid.and_then(|pid| {
                // Safety: OpenProcess only opens a synchronization handle and does not change the parent.
                let handle = unsafe {
                    windows_sys::Win32::System::Threading::OpenProcess(
                        windows_sys::Win32::System::Threading::PROCESS_SYNCHRONIZE,
                        0,
                        pid as u32,
                    )
                };
                if handle.is_null() {
                    log::warn!("parent watch unavailable: {}", std::io::Error::last_os_error());
                    None
                } else {
                    // Safety: successful OpenProcess transfers this handle to ParentWatch.
                    Some(unsafe { OwnedHandle::from_raw_handle(handle) })
                }
            });
            Self { pid, process }
        }
        #[cfg(target_os = "linux")]
        {
            use std::os::fd::{FromRawFd, OwnedFd};
            let process = pid.and_then(|pid| {
                let pid = libc::pid_t::try_from(pid).ok()?;
                // Safety: pidfd_open queries the validated process ID. Flags
                // zero request a close-on-exec descriptor for the whole process.
                let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0u32) };
                if fd < 0 {
                    None
                } else {
                    // Safety: the successful syscall transfers this descriptor.
                    Some(unsafe { OwnedFd::from_raw_fd(fd as _) })
                }
            });
            Self { pid, process }
        }
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
            Self { pid, kq }
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
        Self { pid }
    }
    pub fn exited(&self) -> bool {
        let Some(pid) = self.pid else { return false };
        #[cfg(target_os = "linux")]
        {
            use std::{io::Read, os::fd::AsRawFd};
            if let Some(process) = &self.process {
                let mut fd = libc::pollfd {
                    fd: process.as_raw_fd(),
                    events: libc::POLLIN,
                    revents: 0,
                };
                // Safety: the descriptor is owned, and the zero timeout cannot
                // block. Exit is readable even before the parent is reaped.
                let ready = unsafe { libc::poll(&mut fd, 1, 0) };
                return ready > 0
                    || (ready < 0 && std::io::Error::last_os_error().kind() != std::io::ErrorKind::Interrupted);
            }
            // Older kernels or a restricted syscall policy may lack pidfd_open.
            // kill(pid, 0) alone still reports an unreaped zombie as present.
            let mut stat = Vec::with_capacity(512);
            if std::fs::File::open(format!("/proc/{pid}/stat"))
                .and_then(|file| file.take(4096).read_to_end(&mut stat))
                .is_ok()
            {
                // comm may contain spaces, parentheses, or non-UTF-8 bytes.
                let state = stat
                    .iter()
                    .rposition(|&b| b == b')')
                    .and_then(|end| stat.get(end + 1..))
                    .and_then(|tail| tail.iter().copied().find(|b| !b.is_ascii_whitespace()));
                if state.is_some_and(|state| matches!(state, b'Z' | b'X' | b'x')) {
                    return true;
                }
            }
        }
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
            (unsafe { libc::kill(pid as libc::pid_t, 0) }) < 0
                && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
        }
        #[cfg(windows)]
        {
            let _ = pid;
            use std::os::windows::io::AsRawHandle;
            self.process.as_ref().is_none_or(|process| {
                // Safety: the owned process handle has SYNCHRONIZE access; this is a zero-time poll.
                unsafe {
                    windows_sys::Win32::System::Threading::WaitForSingleObject(process.as_raw_handle(), 0)
                        != windows_sys::Win32::Foundation::WAIT_TIMEOUT
                }
            })
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
