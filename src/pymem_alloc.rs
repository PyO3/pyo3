// TODO https://github.com/PyO3/pyo3/issues/5487
#![allow(clippy::undocumented_unsafe_blocks)]

//! GlobalAlloc backed by CPython's `PyMem_Raw*` (`PYMEM_DOMAIN_RAW`).
//!
//! ```
//! use pyo3::pymem_alloc::PyMemRawAllocator;
//!
//! #[global_allocator]
//! static GLOBAL_ALLOCATOR: PyMemRawAllocator = PyMemRawAllocator;
//! ```

use core::{
    alloc::{GlobalAlloc, Layout},
    mem::size_of,
    ptr,
};

/// `GlobalAlloc` implementation backed by CPython's `PyMem_Raw*` functions  
/// (`PYMEM_DOMAIN_RAW`). Safe to use from any thread, attached or not,  
/// since the raw domain doesn't require an attached thread state.
pub struct PyMemRawAllocator;

// CPython documents this alignment as `ALIGNOF_MAX_ALIGN_T`/
const MAX_ALIGN: usize = cfg_select! {
    // Windows: 8 for both `MS_WIN32` / `MS_WIN64`.
    target_os = "windows" => 8,
    // macOS: 16 on Intel (`i386` / `x86_64`),
    all(target_vendor = "apple", any(target_arch = "x86", target_arch = "x86_64")) => 16,
    // macOS: 8 on other archs (e.g. `arm64`).
    all(target_vendor = "apple", not(any(target_arch = "x86", target_arch = "x86_64"))) => 8,
    // Other Unix: autoconf-derived at build time, not checked in, not guaranteed > 8.
    _ => 8,
};

/// Bytes reserved before an over-aligned block to stash the original
/// pointer returned by `PyMem_RawMalloc`, so it can be recovered for
/// `PyMem_RawFree` / `PyMem_RawRealloc`.
const HEADER: usize = size_of::<*mut u8>();

#[cold]
unsafe fn raw_alloc_aligned_with_header(layout: Layout) -> *mut u8 {
    let Some(total) = layout
        .size()
        .checked_add(layout.align())
        .and_then(|total| total.checked_add(HEADER))
    else {
        return ptr::null_mut();
    };

    let raw = unsafe { pyo3_ffi::PyMem_RawMalloc(total) } as *mut u8;

    if raw.is_null() {
        return ptr::null_mut();
    }

    unsafe { finish_aligned(raw, layout) }
}

#[cold]
unsafe fn raw_calloc_aligned_with_header(layout: Layout) -> *mut u8 {
    let Some(total) = layout
        .size()
        .checked_add(layout.align())
        .and_then(|total| total.checked_add(HEADER))
    else {
        return ptr::null_mut();
    };
    let raw = unsafe { pyo3_ffi::PyMem_RawCalloc(1, total) } as *mut u8;

    if raw.is_null() {
        return ptr::null_mut();
    }
    unsafe { finish_aligned(raw, layout) }
}

#[inline]
unsafe fn finish_aligned(raw: *mut u8, layout: Layout) -> *mut u8 {
    let addr = raw as usize + HEADER;
    let aligned_addr = (addr + layout.align() - 1) & !(layout.align() - 1);
    let block = unsafe { raw.add(aligned_addr - raw as usize) };
    unsafe { (block.sub(HEADER) as *mut *mut u8).write_unaligned(raw) };

    block
}

#[inline]
unsafe fn recover_raw(ptr: *mut u8) -> *mut u8 {
    unsafe { (ptr.sub(HEADER) as *mut *mut u8).read_unaligned() }
}

unsafe impl GlobalAlloc for PyMemRawAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= MAX_ALIGN {
            unsafe { pyo3_ffi::PyMem_RawMalloc(layout.size()) as *mut u8 }
        } else {
            unsafe { raw_alloc_aligned_with_header(layout) }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.align() <= MAX_ALIGN {
            unsafe { pyo3_ffi::PyMem_RawFree(ptr as *mut _) }
        } else {
            let raw = unsafe { recover_raw(ptr) };
            unsafe { pyo3_ffi::PyMem_RawFree(raw as *mut _) }
        }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= MAX_ALIGN {
            unsafe { pyo3_ffi::PyMem_RawCalloc(1, layout.size()) as *mut u8 }
        } else {
            unsafe { raw_calloc_aligned_with_header(layout) }
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };

        if layout.align() <= MAX_ALIGN && new_layout.align() <= MAX_ALIGN {
            return unsafe { pyo3_ffi::PyMem_RawRealloc(ptr as *mut _, new_size) as *mut u8 };
        }

        let new_ptr = unsafe { self.alloc(new_layout) };
        if !new_ptr.is_null() {
            unsafe {
                ptr::copy_nonoverlapping(ptr, new_ptr, layout.size().min(new_size));
                self.dealloc(ptr, layout);
            }
        }
        new_ptr
    }
}
