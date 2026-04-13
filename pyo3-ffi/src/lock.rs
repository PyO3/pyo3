#![cfg(PyRustPython)]

use std::os::raw::c_int;
use std::sync::atomic::{AtomicU8, Ordering};

#[repr(transparent)]
#[derive(Debug)]
pub struct PyMutex {
    pub(crate) bits: AtomicU8,
}

impl PyMutex {
    pub const fn new() -> Self {
        Self {
            bits: AtomicU8::new(0),
        }
    }
}

#[inline]
unsafe fn mutex_from_ptr<'a>(m: *mut PyMutex) -> &'a PyMutex {
    debug_assert!(!m.is_null());
    unsafe { &*m }
}

#[allow(non_snake_case)]
pub unsafe fn PyMutex_Lock(m: *mut PyMutex) {
    let mutex = unsafe { mutex_from_ptr(m) };
    while mutex
        .bits
        .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        std::thread::yield_now();
    }
}

#[allow(non_snake_case)]
pub unsafe fn PyMutex_Unlock(m: *mut PyMutex) {
    let mutex = unsafe { mutex_from_ptr(m) };
    mutex.bits.store(0, Ordering::Release);
}

#[allow(non_snake_case)]
pub unsafe fn PyMutex_IsLocked(m: *mut PyMutex) -> c_int {
    let mutex = unsafe { mutex_from_ptr(m) };
    (mutex.bits.load(Ordering::Acquire) != 0) as c_int
}
