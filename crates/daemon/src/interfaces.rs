//! Address snapshots are read only and sampled only during reconnect backoff.
use std::collections::BTreeSet;
pub type Addresses = BTreeSet<(String, Vec<u8>)>;
#[cfg(unix)]
pub fn snapshot() -> Option<Addresses> {
    use std::{ffi::CStr, ptr};
    let mut head = ptr::null_mut();
    // Safety: getifaddrs initializes a kernel-owned list, which is freed once below.
    if unsafe { libc::getifaddrs(&mut head) } != 0 {
        return None;
    }
    let mut set = Addresses::new();
    let mut current = head;
    while !current.is_null() {
        // Safety: current follows the valid getifaddrs list until the final freeifaddrs.
        let item = unsafe { &*current };
        if item.ifa_flags & (libc::IFF_UP as u32) != 0
            && item.ifa_flags & (libc::IFF_LOOPBACK as u32) == 0
            && !item.ifa_addr.is_null()
            && !item.ifa_name.is_null()
        {
            // Safety: interface names are NUL-terminated and addresses match their advertised family.
            unsafe {
                let name = CStr::from_ptr(item.ifa_name).to_string_lossy().into_owned();
                match i32::from((*item.ifa_addr).sa_family) {
                    libc::AF_INET => {
                        let address = &*item.ifa_addr.cast::<libc::sockaddr_in>();
                        set.insert((name, address.sin_addr.s_addr.to_ne_bytes().to_vec()));
                    }
                    libc::AF_INET6 => {
                        let address = &*item.ifa_addr.cast::<libc::sockaddr_in6>();
                        let mut bytes = address.sin6_addr.s6_addr.to_vec();
                        bytes.extend_from_slice(&address.sin6_scope_id.to_ne_bytes());
                        set.insert((name, bytes));
                    }
                    _ => {}
                }
            }
        }
        current = item.ifa_next;
    }
    // Safety: head is exactly the pointer returned by the successful getifaddrs call.
    unsafe {
        libc::freeifaddrs(head);
    }
    Some(set)
}
#[cfg(target_os = "windows")]
pub fn snapshot() -> Option<Addresses> {
    use std::{ffi::CStr, ptr};
    use windows_sys::Win32::{
        NetworkManagement::IpHelper::{GetAdaptersAddresses, IP_ADAPTER_ADDRESSES_LH},
        Networking::WinSock::{AF_INET, AF_INET6, SOCKADDR_IN, SOCKADDR_IN6},
    };
    let mut size = 16_384u32;
    for _ in 0..3 {
        // u64 storage provides alignment required by the system adapter structures.
        let mut storage = vec![0u64; (size as usize).div_ceil(8)];
        let first = storage.as_mut_ptr().cast::<IP_ADAPTER_ADDRESSES_LH>();
        // Safety: first points to an aligned size-byte writable buffer; the API reports required size on overflow.
        let result = unsafe { GetAdaptersAddresses(0, 0, ptr::null(), first, &mut size) };
        if result == 111 {
            if size > 1024 * 1024 {
                return None;
            }
            continue;
        }
        if result != 0 {
            return None;
        }
        let mut set = Addresses::new();
        let mut current = first;
        while !current.is_null() {
            // Safety: a successful call links all adapter and address nodes inside the live storage buffer.
            unsafe {
                let item = &*current;
                if item.OperStatus == 1 && item.IfType != 24 && !item.AdapterName.is_null() {
                    let name = CStr::from_ptr(item.AdapterName.cast()).to_string_lossy().into_owned();
                    let mut unicast = item.FirstUnicastAddress;
                    while !unicast.is_null() {
                        let address = (*unicast).Address.lpSockaddr;
                        if !address.is_null() {
                            match (*address).sa_family {
                                AF_INET => {
                                    let address = &*address.cast::<SOCKADDR_IN>();
                                    set.insert((name.clone(), address.sin_addr.S_un.S_addr.to_ne_bytes().to_vec()));
                                }
                                AF_INET6 => {
                                    let address = &*address.cast::<SOCKADDR_IN6>();
                                    set.insert((name.clone(), address.sin6_addr.u.Byte.to_vec()));
                                }
                                _ => {}
                            }
                        }
                        unicast = (*unicast).Next;
                    }
                }
                current = item.Next;
            }
        }
        return Some(set);
    }
    None
}
#[cfg(not(any(unix, target_os = "windows")))]
pub fn snapshot() -> Option<Addresses> {
    None
}
