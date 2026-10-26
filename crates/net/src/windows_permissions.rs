//! Protected per-user ACLs, including at creation time before writing a key.
//! Inherited access is removed. Filesystems without persistent ACLs fail closed.
use std::{
    fs::{File, OpenOptions},
    io,
    os::windows::{
        ffi::OsStrExt,
        fs::OpenOptionsExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::Path,
    ptr::null_mut,
};
use windows_sys::Win32::{
    Foundation::{LocalFree, ERROR_SUCCESS, GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE},
    Security::{
        Authorization::{
            ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, SetNamedSecurityInfoW,
            SDDL_REVISION_1, SE_FILE_OBJECT,
        },
        GetSecurityDescriptorDacl, GetTokenInformation, TokenUser, DACL_SECURITY_INFORMATION,
        PROTECTED_DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER,
    },
    Storage::FileSystem::{
        CreateFileW, GetVolumeInformationByHandleW, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, FILE_FLAG_BACKUP_SEMANTICS,
        FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_ALWAYS,
    },
    System::{
        SystemServices::FILE_PERSISTENT_ACLS,
        Threading::{GetCurrentProcess, OpenProcessToken},
    },
};

struct Descriptor(PSECURITY_DESCRIPTOR);
impl Drop for Descriptor {
    fn drop(&mut self) {
        // Safety: conversion allocated this descriptor with LocalAlloc; we own it.
        unsafe { LocalFree(self.0) };
    }
}
fn user_sid() -> io::Result<String> {
    let mut token = null_mut();
    // Safety: the process pseudo-handle is valid and token is writable output storage.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // Safety: a successful OpenProcessToken transfers one owned handle.
    let token = unsafe { OwnedHandle::from_raw_handle(token) };
    let mut size = 0;
    // Safety: null output queries the required size of TokenUser.
    unsafe { GetTokenInformation(token.as_raw_handle(), TokenUser, null_mut(), 0, &mut size) };
    if size < std::mem::size_of::<TOKEN_USER>() as u32 {
        return Err(io::Error::last_os_error());
    }
    let mut buffer = vec![0usize; (size as usize).div_ceil(std::mem::size_of::<usize>())];
    // Safety: word-aligned storage has the queried size and the token is still owned.
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
    // Safety: successful TokenUser output contains TOKEN_USER and the referenced SID.
    let user = unsafe { &*buffer.as_ptr().cast::<TOKEN_USER>() };
    let mut sid = null_mut();
    // Safety: the SID remains in buffer and sid receives a LocalAlloc UTF-16 string.
    if unsafe { ConvertSidToStringSidW(user.User.Sid, &mut sid) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // Safety: the returned string is terminated; copy it before freeing exactly once.
    Ok(unsafe {
        let mut len = 0;
        while *sid.add(len) != 0 {
            len += 1;
        }
        let value = String::from_utf16_lossy(std::slice::from_raw_parts(sid, len));
        LocalFree(sid.cast());
        value
    })
}
impl Descriptor {
    fn new(directory: bool) -> io::Result<Self> {
        let sid = user_sid()?;
        let inheritance = if directory { "OICI" } else { "" };
        let sddl: Vec<u16> = format!("D:P(A;{inheritance};FA;;;{sid})")
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let mut descriptor = null_mut();
        // Safety: SDDL is terminated and descriptor receives a LocalAlloc allocation.
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
        Ok(Self(descriptor))
    }
}

pub(crate) fn restrict(path: &Path, directory: bool) -> io::Result<()> {
    let descriptor = Descriptor::new(directory)?;
    let mut present = 0;
    let mut defaulted = 0;
    let mut dacl = null_mut();
    // Safety: descriptor is valid; all output pointers refer to initialized storage.
    if unsafe { GetSecurityDescriptorDacl(descriptor.0, &mut present, &mut dacl, &mut defaulted) } == 0
        || present == 0
        || dacl.is_null()
    {
        return Err(io::Error::other("private TLS ACL is missing"));
    }
    let mut path_w: Vec<u16> = path.as_os_str().encode_wide().collect();
    if path_w.contains(&0) {
        return Err(io::Error::other("TLS path contains NUL"));
    }
    path_w.push(0);
    // Safety: path is terminated; dacl stays owned by descriptor throughout the call.
    let error = unsafe {
        SetNamedSecurityInfoW(
            path_w.as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            dacl,
            null_mut(),
        )
    };
    if error != ERROR_SUCCESS {
        return Err(io::Error::from_raw_os_error(error as i32));
    }
    let file = OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)?;
    persistent_acls(&file)
}

fn persistent_acls(file: &File) -> io::Result<()> {
    let mut flags = 0;
    // Safety: the file handle remains owned; only the filesystem-flags output is requested.
    if unsafe {
        GetVolumeInformationByHandleW(
            file.as_raw_handle(),
            null_mut(),
            0,
            null_mut(),
            null_mut(),
            &mut flags,
            null_mut(),
            0,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if flags & FILE_PERSISTENT_ACLS == 0 {
        return Err(io::Error::other(
            "TLS storage requires a filesystem with persistent Windows ACLs",
        ));
    }
    Ok(())
}

pub(crate) fn open(path: &Path, exclusive: bool) -> io::Result<File> {
    if !exclusive && path.try_exists()? {
        restrict(path, false)?;
    }
    let descriptor = Descriptor::new(false)?;
    let attributes = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: 0,
    };
    let mut path_w: Vec<u16> = path.as_os_str().encode_wide().collect();
    if path_w.contains(&0) {
        return Err(io::Error::other("TLS path contains NUL"));
    }
    path_w.push(0);
    // Safety: terminated path, attributes and descriptor outlive creation; Windows copies the ACL.
    let handle = unsafe {
        CreateFileW(
            path_w.as_ptr(),
            GENERIC_WRITE | if exclusive { 0 } else { GENERIC_READ },
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            &attributes,
            if exclusive { CREATE_NEW } else { OPEN_ALWAYS },
            FILE_ATTRIBUTE_NORMAL,
            null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    // Safety: successful CreateFileW transferred one owned file handle.
    Ok(unsafe { File::from_raw_handle(handle) })
}
