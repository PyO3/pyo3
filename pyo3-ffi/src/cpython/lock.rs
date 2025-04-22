use std::marker::PhantomPinned;
use std::sync::atomic::AtomicU8;

#[repr(transparent)]
#[derive(Debug)]
pub struct PyMutex {
    pub(crate) _bits: AtomicU8,
    pub(crate) _pin: PhantomPinned,
}

impl PyMutex {
    pub const fn new() -> PyMutex {
        PyMutex {
            _bits: AtomicU8::new(0),
            _pin: PhantomPinned,
        }
    }
}

extern "C" {
    pub fn PyMutex_Lock(m: *mut PyMutex);
    pub fn PyMutex_Unlock(m: *mut PyMutex);
}
