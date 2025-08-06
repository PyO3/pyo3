use std::marker::PhantomPinned;
use std::sync::atomic::AtomicU8;

#[repr(transparent)]
#[derive(Debug)]
#[cfg(all(Py_3_13, not(Py_LIMITED_API)))]
pub struct PyMutex {
    pub(crate) _bits: AtomicU8,
    pub(crate) _pin: PhantomPinned,
}

#[cfg(all(Py_3_13, not(Py_LIMITED_API)))]
impl PyMutex {
    pub const fn new() -> PyMutex {
        PyMutex {
            _bits: AtomicU8::new(0),
            _pin: PhantomPinned,
        }
    }
}

#[cfg(all(Py_3_13, not(Py_LIMITED_API)))]
impl Default for PyMutex {
    fn default() -> Self {
        Self::new()
    }
}

extern "C" {
    #[cfg(all(Py_3_13, not(Py_LIMITED_API)))]
    pub fn PyMutex_Lock(m: *mut PyMutex);
    #[cfg(all(Py_3_13, not(Py_LIMITED_API)))]
    pub fn PyMutex_Unlock(m: *mut PyMutex);
}
