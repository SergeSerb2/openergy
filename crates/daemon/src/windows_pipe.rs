//! Per-user byte pipes. Overlapped operations give blocking callers bounded waits.
use std::{
    io::{self, Read, Write},
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    path::Path,
    ptr::{null, null_mut},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{
        LocalFree, ERROR_ACCESS_DENIED, ERROR_BROKEN_PIPE, ERROR_IO_PENDING, ERROR_NO_DATA, ERROR_PIPE_BUSY,
        ERROR_PIPE_CONNECTED, ERROR_PIPE_NOT_CONNECTED, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
        WAIT_OBJECT_0, WAIT_TIMEOUT,
    },
    Security::{
        Authorization::{
            ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
        },
        GetTokenInformation, TokenUser, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER,
    },
    Storage::FileSystem::{
        CreateFileW, ReadFile, WriteFile, FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_FLAG_OVERLAPPED, OPEN_EXISTING,
        PIPE_ACCESS_DUPLEX,
    },
    System::{
        Pipes::{
            ConnectNamedPipe, CreateNamedPipeW, WaitNamedPipeW, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS,
            PIPE_TYPE_BYTE, PIPE_WAIT,
        },
        Threading::{CreateEventW, GetCurrentProcess, OpenProcessToken, WaitForSingleObject},
        IO::{CancelIoEx, GetOverlappedResult, OVERLAPPED},
    },
};

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn owned(handle: HANDLE) -> io::Result<OwnedHandle> {
    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        Err(io::Error::last_os_error())
    } else {
        // Safety: the successful Win32 creation call transfers one owned handle.
        Ok(unsafe { OwnedHandle::from_raw_handle(handle) })
    }
}
fn user_sid() -> io::Result<String> {
    let mut token = null_mut();
    // Safety: our process pseudo-handle is valid and token is writable output storage.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let token = owned(token)?;
    let mut size = 0;
    // Safety: a null output buffer queries the required token information size.
    unsafe { GetTokenInformation(token.as_raw_handle(), TokenUser, null_mut(), 0, &mut size) };
    let mut buffer = vec![0usize; (size as usize).div_ceil(std::mem::size_of::<usize>())];
    // Safety: the word-aligned buffer has the queried size and token remains owned.
    if unsafe {
        GetTokenInformation(
            token.as_raw_handle(),
            TokenUser,
            buffer.as_mut_ptr().cast(),
            size,
            &mut size,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    // Safety: successful TokenUser output contains a TOKEN_USER and its referenced SID.
    let user = unsafe { &*buffer.as_ptr().cast::<TOKEN_USER>() };
    let mut sid = null_mut();
    // Safety: the SID is valid for the lifetime of buffer; sid receives a LocalAlloc string.
    if unsafe { ConvertSidToStringSidW(user.User.Sid, &mut sid) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // Safety: the API returned a terminated UTF-16 string; it is freed exactly once after copying.
    let result = unsafe {
        let mut length = 0;
        while *sid.add(length) != 0 {
            length += 1;
        }
        let result = String::from_utf16_lossy(std::slice::from_raw_parts(sid, length));
        LocalFree(sid.cast());
        result
    };
    Ok(result)
}
/// A socket path selects a stable pipe within the current user's namespace.
fn pipe_name(path: &Path, sid: &str) -> io::Result<String> {
    let text = path.to_string_lossy();
    if text.contains('\0') {
        return Err(io::Error::other("pipe path contains NUL"));
    }
    if let Some(name) = text.strip_prefix(r"\\.\pipe\") {
        if name.is_empty() || name.contains(['/', '\\']) {
            return Err(io::Error::other("invalid local pipe name"));
        }
        return Ok(text.into_owned());
    }
    let absolute = std::path::absolute(path)?;
    let normalized = absolute.to_string_lossy().replace('/', "\\").to_ascii_lowercase();
    let mut hash = 0xcbf29ce484222325u64;
    for byte in sid.bytes().chain(Some(0)).chain(normalized.bytes()) {
        hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
    }
    Ok(format!(r"\\.\pipe\openergy-{hash:016x}"))
}

pub struct Listener {
    handle: OwnedHandle,
    name: Vec<u16>,
    sid: String,
}
pub fn bind(path: &Path) -> Result<(Listener, ()), String> {
    let sid = user_sid().map_err(|e| e.to_string())?;
    let name = pipe_name(path, &sid).map_err(|e| e.to_string())?;
    let name_w = wide(&name);
    let handle = create(&name_w, &sid, true).map_err(|e| {
        if e.raw_os_error() == Some(ERROR_ACCESS_DENIED as i32) {
            format!("another daemon owns {name}")
        } else {
            format!("bind {name}: {e}")
        }
    })?;
    Ok((
        Listener {
            handle,
            name: name_w,
            sid,
        },
        (),
    ))
}
fn create(name: &[u16], sid: &str, first: bool) -> io::Result<OwnedHandle> {
    let sddl = wide(&format!("D:P(A;;GA;;;{sid})"));
    let mut descriptor = null_mut();
    // Safety: terminated SDDL input and valid output pointer; descriptor is freed below.
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            SDDL_REVISION_1,
            &mut descriptor,
            null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor,
        bInheritHandle: 0,
    };
    let flags = PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | if first { FILE_FLAG_FIRST_PIPE_INSTANCE } else { 0 };
    // Safety: all pointers remain valid through creation; the DACL is copied by the kernel.
    let result = unsafe {
        owned(CreateNamedPipeW(
            name.as_ptr(),
            flags,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
            66,
            65536,
            65536,
            250,
            &attributes,
        ))
    };
    // Safety: descriptor was allocated by the conversion API and is no longer needed.
    unsafe { LocalFree(descriptor) };
    result
}
impl Listener {
    pub fn accept(&mut self, stop: &AtomicBool) -> io::Result<Stream> {
        loop {
            if stop.load(Ordering::Relaxed) {
                return Err(io::Error::new(io::ErrorKind::Interrupted, "IPC stopped"));
            }
            let event = event()?;
            let mut operation = OVERLAPPED {
                hEvent: event.as_raw_handle(),
                ..Default::default()
            };
            // Safety: the handle is overlapped; operation and event live until completion/cancellation.
            let ok = unsafe { ConnectNamedPipe(self.handle.as_raw_handle(), &mut operation) };
            let error = io::Error::last_os_error();
            let connected = if ok != 0 || error.raw_os_error() == Some(ERROR_PIPE_CONNECTED as i32) {
                Ok(())
            } else if error.raw_os_error() == Some(ERROR_IO_PENDING as i32) {
                finish(self.handle.as_raw_handle(), &mut operation, None, Some(stop)).map(|_| ())
            } else {
                Err(error)
            };
            if let Err(error) = connected {
                if matches!(
                    error.raw_os_error().map(|e| e as u32),
                    Some(ERROR_NO_DATA | ERROR_BROKEN_PIPE | ERROR_PIPE_NOT_CONNECTED)
                ) {
                    // A client can close between CreateNamedPipe and ConnectNamedPipe.
                    let next = create(&self.name, &self.sid, false)?;
                    self.handle = next;
                    continue;
                }
                return Err(error);
            }
            // Keep an instance alive continuously so FIRST_PIPE_INSTANCE remains an exclusive lock.
            let next = create(&self.name, &self.sid, false)?;
            let handle = std::mem::replace(&mut self.handle, next);
            return Ok(Stream {
                handle,
                read_timeout: Duration::from_millis(50),
                write_timeout: Duration::from_secs(1),
            });
        }
    }
}
fn event() -> io::Result<OwnedHandle> {
    // Safety: a private manual-reset event has no security attributes or name pointers to retain.
    owned(unsafe { CreateEventW(null(), 1, 0, null()) })
}
/// Always reap cancellation before the operation or its buffer leaves scope.
fn finish(
    handle: HANDLE,
    operation: &mut OVERLAPPED,
    timeout: Option<Duration>,
    stop: Option<&AtomicBool>,
) -> io::Result<usize> {
    let start = Instant::now();
    loop {
        let stopped = stop.is_some_and(|s| s.load(Ordering::Relaxed));
        let expired = timeout.is_some_and(|t| start.elapsed() >= t);
        let wait = if stopped || expired {
            WAIT_TIMEOUT
        } else {
            let ms = timeout
                .map(|t| t.saturating_sub(start.elapsed()).as_millis().clamp(1, 50) as u32)
                .unwrap_or(50);
            // Safety: operation's event remains owned and valid throughout the wait.
            unsafe { WaitForSingleObject(operation.hEvent, ms) }
        };
        let mut bytes = 0;
        if wait == WAIT_OBJECT_0 {
            // Safety: event completion makes the output available; operation and handle remain valid.
            if unsafe { GetOverlappedResult(handle, operation, &mut bytes, 0) } != 0 {
                return Ok(bytes as usize);
            }
            return Err(io::Error::last_os_error());
        }
        if wait == WAIT_TIMEOUT && !stopped && !expired {
            continue;
        }
        let error = if wait != WAIT_TIMEOUT {
            io::Error::last_os_error()
        } else {
            io::Error::new(
                if stopped {
                    io::ErrorKind::Interrupted
                } else {
                    io::ErrorKind::TimedOut
                },
                "named pipe wait ended",
            )
        };
        // Safety: cancel only this live operation and wait until the kernel releases its buffer.
        unsafe {
            CancelIoEx(handle, operation);
            if GetOverlappedResult(handle, operation, &mut bytes, 1) != 0 {
                return Ok(bytes as usize);
            }
        }
        return Err(error);
    }
}

pub struct Stream {
    handle: OwnedHandle,
    read_timeout: Duration,
    write_timeout: Duration,
}
impl Stream {
    pub fn connect(path: &Path) -> io::Result<Self> {
        let sid = user_sid()?;
        let name = wide(&pipe_name(path, &sid)?);
        let start = Instant::now();
        let handle = loop {
            // Safety: the pipe name is terminated and no input pointers are retained by CreateFileW.
            let result = owned(unsafe {
                CreateFileW(
                    name.as_ptr(),
                    GENERIC_READ | GENERIC_WRITE,
                    0,
                    null(),
                    OPEN_EXISTING,
                    FILE_FLAG_OVERLAPPED,
                    null_mut(),
                )
            });
            match result {
                Ok(handle) => break handle,
                Err(e)
                    if e.raw_os_error() == Some(ERROR_PIPE_BUSY as i32) && start.elapsed() < Duration::from_secs(1) =>
                {
                    // Safety: a local, terminated pipe name is valid for this bounded wait.
                    if unsafe { WaitNamedPipeW(name.as_ptr(), 250) } == 0 {
                        let error = io::Error::last_os_error();
                        if error.raw_os_error() != Some(windows_sys::Win32::Foundation::ERROR_SEM_TIMEOUT as i32) {
                            return Err(error);
                        }
                    }
                }
                Err(e) => return Err(e),
            }
        };
        Ok(Self {
            handle,
            read_timeout: Duration::from_secs(10),
            write_timeout: Duration::from_secs(10),
        })
    }
    fn transfer(&mut self, buffer: *mut u8, length: usize, write: bool) -> io::Result<usize> {
        if length == 0 {
            return Ok(0);
        }
        let event = event()?;
        let mut operation = OVERLAPPED {
            hEvent: event.as_raw_handle(),
            ..Default::default()
        };
        let length = length.min(u32::MAX as usize) as u32;
        // Safety: Read/Write supplies a live buffer of length bytes; it stays borrowed until reaped.
        let ok = unsafe {
            if write {
                WriteFile(
                    self.handle.as_raw_handle(),
                    buffer.cast_const(),
                    length,
                    null_mut(),
                    &mut operation,
                )
            } else {
                ReadFile(self.handle.as_raw_handle(), buffer, length, null_mut(), &mut operation)
            }
        };
        if ok == 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() != Some(ERROR_IO_PENDING as i32) {
                return Err(error);
            }
        }
        finish(
            self.handle.as_raw_handle(),
            &mut operation,
            Some(if write { self.write_timeout } else { self.read_timeout }),
            None,
        )
    }
}
impl Read for Stream {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.transfer(buffer.as_mut_ptr(), buffer.len(), false)
    }
}
impl Write for Stream {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.transfer(buffer.as_ptr().cast_mut(), buffer.len(), true)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
