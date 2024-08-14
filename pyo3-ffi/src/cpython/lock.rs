use std::fmt;
use std::sync::atomic::AtomicU8;

#[repr(C)]
pub struct PyMutex {
    pub(crate) _bits: AtomicU8,
}

impl fmt::Debug for PyMutex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PyMutex").finish()
    }
}

extern "C" {
    pub fn PyMutex_Lock(m: *mut PyMutex);
    pub fn PyMutex_UnLock(m: *mut PyMutex);
}
