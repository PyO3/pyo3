use std::sync::atomic::AtomicU8;

#[repr(C)]
#[derive(Debug)]
pub struct PyMutex {
    pub(crate) _bits: AtomicU8,
}

extern "C" {
    pub fn PyMutex_Lock(m: *mut PyMutex);
    pub fn PyMutex_UnLock(m: *mut PyMutex);
}
