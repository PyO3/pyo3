//! A global allocator that attaches to the interpreter, as
//! `pyo3_polars::PolarsAllocator` does to find the Polars allocator.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, Ordering};

use pyo3::prelude::*;

struct AttachingAllocator;

static READY: AtomicBool = AtomicBool::new(false);

// SAFETY: every allocation is delegated to `System`.
unsafe impl GlobalAlloc for AttachingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: `Py_IsInitialized` may be called at any time.
        if !READY.load(Ordering::Relaxed) && unsafe { pyo3::ffi::Py_IsInitialized() } != 0 {
            // If attaching allocates, this recurses until the stack overflows.
            Python::attach(|_| ());
            READY.store(true, Ordering::Relaxed);
        }
        // SAFETY: the caller upholds the `GlobalAlloc::alloc` contract.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` was allocated by `System` with this `layout`.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOC: AttachingAllocator = AttachingAllocator;

#[test]
fn test_detach_with_an_attaching_global_allocator() {
    Python::initialize();
    Python::attach(|py| py.detach(|| ()));
}
