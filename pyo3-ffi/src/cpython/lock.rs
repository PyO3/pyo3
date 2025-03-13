use std::marker::PhantomPinned;
use std::sync::atomic::AtomicU8;

#[repr(transparent)]
#[derive(Debug)]
pub struct PyMutex {
    pub(crate) _bits: AtomicU8,
    pub(crate) _pin: PhantomPinned,
}

unsafe extern "C" {
    pub fn PyMutex_Lock(m: *mut PyMutex);
    pub fn PyMutex_Unlock(m: *mut PyMutex);
}
