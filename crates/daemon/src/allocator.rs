//! Large transient buffers bypass Darwin's malloc cache so dropping a clipboard
//! reassembly/IPC buffer releases its resident pages immediately.
use std::alloc::{GlobalAlloc, Layout, System};
use std::ptr;

struct TransferAllocator;
#[global_allocator]
static ALLOCATOR: TransferAllocator = TransferAllocator;

fn mapped(layout: Layout) -> bool {
    // Both supported Mac page sizes are multiples of 4096.
    layout.size() >= 1024 * 1024 && layout.align() <= 4096
}
// Safety: allocation and deallocation select the same owner from the supplied
// layout. Anonymous mappings provide the requested size and page alignment.
unsafe impl GlobalAlloc for TransferAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if mapped(layout) {
            // Safety: this creates a private, writable mapping with no shared file.
            let memory = unsafe {
                libc::mmap(
                    ptr::null_mut(),
                    layout.size(),
                    libc::PROT_READ | libc::PROT_WRITE,
                    libc::MAP_ANON | libc::MAP_PRIVATE,
                    -1,
                    0,
                )
            };
            if memory == libc::MAP_FAILED {
                ptr::null_mut()
            } else {
                memory.cast()
            }
        } else {
            // Safety: System receives the caller's valid allocation layout.
            unsafe { System.alloc(layout) }
        }
    }
    unsafe fn dealloc(&self, memory: *mut u8, layout: Layout) {
        if mapped(layout) {
            // Safety: this exact mapping and length came from alloc for this layout.
            unsafe {
                libc::munmap(memory.cast(), layout.size());
            }
        } else {
            // Safety: the small/aligned allocation was created by System.
            unsafe {
                System.dealloc(memory, layout);
            }
        }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if mapped(layout) {
            // Safety: anonymous mappings from alloc are already zero initialized.
            unsafe { self.alloc(layout) }
        } else {
            // Safety: System receives the caller's valid allocation layout.
            unsafe { System.alloc_zeroed(layout) }
        }
    }
    unsafe fn realloc(&self, memory: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let Ok(next) = Layout::from_size_align(size, layout.align()) else {
            return ptr::null_mut();
        };
        if !mapped(layout) && !mapped(next) {
            // Safety: both layouts belong to System and the old pointer is valid.
            return unsafe { System.realloc(memory, layout, size) };
        }
        // Safety: next is a valid layout; null leaves the old allocation intact.
        let replacement = unsafe { self.alloc(next) };
        if !replacement.is_null() {
            // Safety: allocations do not overlap and both contain the copied range.
            unsafe {
                ptr::copy_nonoverlapping(memory, replacement, layout.size().min(size));
            }
            // Safety: the old pointer is no longer needed and uses its original layout.
            unsafe {
                self.dealloc(memory, layout);
            }
        }
        replacement
    }
}
