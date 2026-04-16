#[cfg(Py_3_14)]
use std::os::raw::c_int;
use std::sync::atomic::AtomicU8;

#[repr(transparent)]
#[derive(Debug)]
pub struct PyMutex {
    pub(crate) _bits: AtomicU8,
}

#[allow(clippy::new_without_default)]
impl PyMutex {
    pub const fn new() -> PyMutex {
        PyMutex {
            _bits: AtomicU8::new(0),
        }
    }
}

extern_libpython! {
    pub fn PyMutex_Lock(m: *mut PyMutex);
    pub fn PyMutex_Unlock(m: *mut PyMutex);
    #[cfg(Py_3_14)]
    pub fn PyMutex_IsLocked(m: *mut PyMutex) -> c_int;
}
