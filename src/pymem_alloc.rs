//! `GlobalAlloc` backed by CPython's `PyMem_Raw*` (`PYMEM_DOMAIN_RAW`).
//!
//! ```
//! use pyo3::pymem_alloc::PyMemRawAllocator;
//!
//! #[global_allocator]
//! static GLOBAL_ALLOCATOR: PyMemRawAllocator = PyMemRawAllocator;
//! ```

use core::{
    alloc::{GlobalAlloc, Layout},
    ffi::c_void,
    mem::size_of,
    ptr,
};

/// `GlobalAlloc` implementation backed by CPython's `PyMem_Raw*` functions (`PYMEM_DOMAIN_RAW`).
/// Safe to use from any thread, attached or not, since the raw domain doesn't require an attached
/// thread state.
pub struct PyMemRawAllocator;

// CPython documents this alignment as `ALIGNOF_MAX_ALIGN_T`
const MIN_ALIGN: usize = cfg_select! {
    // Windows: 8 for both `MS_WIN32` / `MS_WIN64`.
    target_os = "windows" => 8,
    // macOS: 16 on Intel (`i386` / `x86_64`).
    all(target_vendor = "apple", any(target_arch = "x86", target_arch = "x86_64")) => 16,
    // macOS: 8 on other archs (e.g. `arm64`).
    all(target_vendor = "apple", not(any(target_arch = "x86", target_arch = "x86_64"))) => 8,
    // Other Unix: autoconf-derived at build time, not checked in, not guaranteed > 8.
    _ => 8,
};

/// Header stashing the original allocation pointer before an over-aligned block.
///
/// Recovered by [`recover_raw`] to pass back to `PyMem_RawFree` / `PyMem_RawRealloc`.
#[repr(transparent)]
struct Header(*mut u8);

/// Allocates memory for `layout` with a [`Header`] stashed before an
/// over-aligned block. Returns null on overflow or allocation failure.
///
/// # Safety
///
/// `alloc_fn` must behave like `PyMem_RawMalloc`/`PyMem_RawCalloc`.
#[cold]
unsafe fn raw_alloc_with_header(
    layout: Layout,
    alloc_fn: impl FnOnce(usize) -> *mut c_void,
) -> *mut u8 {
    let Some(total) = layout
        .size()
        .checked_add(layout.align())
        .and_then(|total| total.checked_add(size_of::<Header>()))
    else {
        return ptr::null_mut();
    };

    let raw = alloc_fn(total) as *mut u8;

    if raw.is_null() {
        return ptr::null_mut();
    }

    // SAFETY: `raw` is valid for `total` bytes, enough for `layout` plus a `Header`.
    unsafe { finish_aligned(raw, layout) }
}

/// Writes a [`Header`] before the aligned block within a `raw_alloc_with_header` allocation.
///
/// # Safety
///
/// `raw` must point to an allocation of at least `layout.size() + layout.align() +
/// size_of::<Header>()` bytes.
#[inline]
unsafe fn finish_aligned(raw: *mut u8, layout: Layout) -> *mut u8 {
    let addr = raw as usize + size_of::<Header>();
    let mask = layout.align() - 1;
    let aligned_addr = (addr + mask) & !mask;
    let block = unsafe { raw.add(aligned_addr - raw as usize) };
    // SAFETY: `block` is within the allocation and has at least `size_of::<Header>()`
    // bytes of padding before it, so writing a `Header` there is in-bounds.
    unsafe { ptr::write((block as *mut Header).sub(1), Header(raw)) };
    block
}

/// Recovers the original allocation pointer stashed by [`finish_aligned`] before `ptr`.
///
/// # Safety
///
/// `ptr` must have been returned by [`finish_aligned`], with its `Header` still intact.
#[inline]
unsafe fn recover_raw(ptr: *mut u8) -> *mut u8 {
    // SAFETY: `ptr` has a `Header` written directly before it by `finish_aligned`.
    unsafe { ptr::read((ptr as *mut Header).sub(1)).0 }
}

unsafe impl GlobalAlloc for PyMemRawAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= MIN_ALIGN {
            // SAFETY: `PyMem_RawMalloc` accepts any size, including 0, returning a
            // distinct non-NULL pointer as if `PyMem_RawMalloc(1)` were called.
            //
            // See: https://docs.python.org/3/c-api/memory.html#c.PyMem_RawMalloc
            unsafe { pyo3_ffi::PyMem_RawMalloc(layout.size()) as *mut u8 }
        } else {
            // SAFETY: same zero-size guarantee as above.
            //
            // See: https://docs.python.org/3/c-api/memory.html#c.PyMem_RawMalloc
            unsafe { raw_alloc_with_header(layout, |total| pyo3_ffi::PyMem_RawMalloc(total)) }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.align() <= MIN_ALIGN {
            // SAFETY: `ptr` must have been returned by a prior `PyMem_RawMalloc`,
            // `PyMem_RawRealloc`, or `PyMem_RawCalloc` call, and must not have
            // been freed before.
            //
            // See: https://docs.python.org/3/c-api/memory.html#c.PyMem_RawFree
            unsafe { pyo3_ffi::PyMem_RawFree(ptr as *mut _) }
        } else {
            // SAFETY: `ptr` was returned by `finish_aligned`, so it has a `Header` before it.
            let raw = unsafe { recover_raw(ptr) };
            // SAFETY: `raw` is the original pointer from `PyMem_RawMalloc`/`PyMem_RawCalloc`,
            // which is what `PyMem_RawFree` requires.
            //
            // See: https://docs.python.org/3/c-api/memory.html#c.PyMem_RawFree
            unsafe { pyo3_ffi::PyMem_RawFree(raw as *mut _) }
        }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= MIN_ALIGN {
            // SAFETY: `PyMem_RawCalloc` accepts any size, including 0, returning a
            // distinct non-NULL pointer as if `PyMem_RawCalloc(1, 1)` were called.
            //
            // See: https://docs.python.org/3/c-api/memory.html#c.PyMem_RawCalloc
            unsafe { pyo3_ffi::PyMem_RawCalloc(1, layout.size()) as *mut u8 }
        } else {
            // SAFETY: same zero-size guarantee as above.
            //
            // See: https://docs.python.org/3/c-api/memory.html#c.PyMem_RawCalloc
            unsafe { raw_alloc_with_header(layout, |total| pyo3_ffi::PyMem_RawCalloc(1, total)) }
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: `new_size` is non-zero per the `GlobalAlloc::realloc` contract, and
        // `layout.align()` is unchanged.
        let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };

        if layout.align() <= MIN_ALIGN {
            // SAFETY: CPython's `PyMem_RawRealloc` contract mirrors `PyMem_RawFree`:
            // `ptr` must come from a prior `PyMem_Raw{Malloc,Realloc,Calloc}` call.
            //
            // See: https://docs.python.org/3/c-api/memory.html#c.PyMem_RawRealloc
            return unsafe { pyo3_ffi::PyMem_RawRealloc(ptr as *mut _, new_size) as *mut u8 };
        }

        // SAFETY: `new_layout` has non-zero size and the caller-guaranteed alignment.
        let new_ptr = unsafe { self.alloc(new_layout) };

        if !new_ptr.is_null() {
            // SAFETY: `ptr` is valid for `layout.size()` bytes, `new_ptr` for `new_size` bytes,
            // and they don't overlap since `new_ptr` is a fresh allocation. `ptr`/`layout`
            // match the original allocation, as required by `dealloc`.
            unsafe {
                ptr::copy_nonoverlapping(ptr, new_ptr, layout.size().min(new_size));
                self.dealloc(ptr, layout);
            }
        }
        new_ptr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec::Vec;

    // SAFETY: `ptr` must be valid for reads of `layout.size()` bytes.
    unsafe fn assert_zeroes(ptr: *mut u8, layout: Layout) {
        unsafe {
            for ofs in 0..layout.size() {
                assert_eq!(0, ptr.add(ofs).read());
            }
        }
    }

    #[test]
    fn alloc_dealloc_small_align() {
        let alloc = PyMemRawAllocator;
        let layout = Layout::from_size_align(64, MIN_ALIGN).unwrap();
        // SAFETY: `layout` has non-zero size; `ptr` is deallocated with the same layout.
        unsafe {
            let ptr = alloc.alloc(layout);
            assert!(!ptr.is_null());
            assert_eq!(ptr.addr() % layout.align(), 0);
            ptr.write_bytes(0xAB, layout.size());
            alloc.dealloc(ptr, layout);
        }
    }

    #[test]
    fn alloc_dealloc_over_aligned() {
        let alloc = PyMemRawAllocator;
        for align in [MIN_ALIGN * 2, 64, 256, 4096] {
            let layout = Layout::from_size_align(128, align).unwrap();
            // SAFETY: `layout` has non-zero size; `ptr` is deallocated with the same layout.
            unsafe {
                let ptr = alloc.alloc(layout);
                assert!(!ptr.is_null());
                assert_eq!(ptr.addr() % align, 0, "align={align}");
                ptr.write_bytes(0xCD, layout.size());
                alloc.dealloc(ptr, layout);
            }
        }
    }

    #[test]
    fn alloc_zeroed_small_align() {
        let alloc = PyMemRawAllocator;
        let layout = Layout::from_size_align(256, MIN_ALIGN).unwrap();
        // SAFETY: `layout` has non-zero size; `ptr` is deallocated with the same layout.
        unsafe {
            let ptr = alloc.alloc_zeroed(layout);
            assert!(!ptr.is_null());
            assert_zeroes(ptr, layout);
            alloc.dealloc(ptr, layout);
        }
    }

    #[test]
    fn alloc_zeroed_over_aligned() {
        let alloc = PyMemRawAllocator;
        let layout = Layout::from_size_align(4096, 64).unwrap();
        // SAFETY: `layout` has non-zero size; `ptr` is deallocated with the same layout.
        unsafe {
            let ptr = alloc.alloc_zeroed(layout);
            assert!(!ptr.is_null());
            assert_eq!(ptr.addr() % layout.align(), 0);
            assert_zeroes(ptr, layout);
            alloc.dealloc(ptr, layout);
        }
    }

    #[test]
    fn realloc_grow_preserves_data_small_align() {
        let alloc = PyMemRawAllocator;
        let old_layout = Layout::from_size_align(16, MIN_ALIGN).unwrap();
        // SAFETY: `old_layout` has non-zero size; `new_ptr`/`ptr` are always
        // deallocated with the layout they were allocated/reallocated with.
        unsafe {
            let ptr = alloc.alloc(old_layout);
            assert!(!ptr.is_null());
            ptr.write_bytes(0x42, old_layout.size());

            let new_ptr = alloc.realloc(ptr, old_layout, 256);
            assert!(!new_ptr.is_null());
            for i in 0..old_layout.size() {
                assert_eq!(new_ptr.add(i).read(), 0x42);
            }
            alloc.dealloc(new_ptr, Layout::from_size_align(256, MIN_ALIGN).unwrap());
        }
    }

    #[test]
    fn realloc_shrink_preserves_data_small_align() {
        let alloc = PyMemRawAllocator;
        let old_layout = Layout::from_size_align(256, MIN_ALIGN).unwrap();
        // SAFETY: `old_layout` has non-zero size; `new_size` is non-zero and
        // `new_ptr` is deallocated with the layout it was reallocated with.
        unsafe {
            let ptr = alloc.alloc(old_layout);
            assert!(!ptr.is_null());
            ptr.write_bytes(0x7A, old_layout.size());

            let new_ptr = alloc.realloc(ptr, old_layout, 16);
            assert!(!new_ptr.is_null());
            for i in 0..16 {
                assert_eq!(new_ptr.add(i).read(), 0x7A);
            }
            alloc.dealloc(new_ptr, Layout::from_size_align(16, MIN_ALIGN).unwrap());
        }
    }

    #[test]
    fn realloc_over_aligned_preserves_data() {
        let alloc = PyMemRawAllocator;
        let align = 64;
        let old_layout = Layout::from_size_align(32, align).unwrap();
        // SAFETY: `old_layout` has non-zero size; `new_ptr` is deallocated
        // with the layout it was reallocated with.
        unsafe {
            let ptr = alloc.alloc(old_layout);
            assert!(!ptr.is_null());
            ptr.write_bytes(0x55, old_layout.size());

            let new_ptr = alloc.realloc(ptr, old_layout, 512);
            assert!(!new_ptr.is_null());
            assert_eq!(new_ptr.addr() % align, 0);
            for i in 0..old_layout.size() {
                assert_eq!(new_ptr.add(i).read(), 0x55);
            }
            alloc.dealloc(new_ptr, Layout::from_size_align(512, align).unwrap());
        }
    }

    #[test]
    fn no_overlap_between_allocations() {
        let alloc = PyMemRawAllocator;
        let layouts: Vec<Layout> = vec![
            Layout::from_size_align(16, MIN_ALIGN).unwrap(),
            Layout::from_size_align(128, 64).unwrap(),
            Layout::from_size_align(4096, 4096).unwrap(),
            Layout::from_size_align(64, MIN_ALIGN).unwrap(),
        ];
        // SAFETY: each `layout` has non-zero size; each `ptr` is deallocated
        // with the same layout it was allocated with.
        unsafe {
            let ptrs: Vec<*mut u8> = layouts
                .iter()
                .map(|&layout| {
                    let ptr = alloc.alloc_zeroed(layout);
                    assert!(!ptr.is_null());
                    ptr.write_bytes(0xEE, layout.size());
                    ptr
                })
                .collect();

            for (&ptr, &layout) in ptrs.iter().zip(&layouts) {
                for i in 0..layout.size() {
                    assert_eq!(ptr.add(i).read(), 0xEE);
                }
            }

            for (ptr, layout) in ptrs.into_iter().zip(layouts) {
                alloc.dealloc(ptr, layout);
            }
        }
    }
}
