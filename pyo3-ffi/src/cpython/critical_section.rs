#[cfg(any(Py_3_14, Py_GIL_DISABLED))]
use crate::PyMutex;
use crate::{PyCriticalSection, PyCriticalSection2};

extern_libpython! {
    #[cfg(Py_3_14)]
    pub fn PyCriticalSection_BeginMutex(c: *mut PyCriticalSection, m: *mut PyMutex);
    #[cfg(Py_3_14)]
    pub fn PyCriticalSection2_BeginMutex(
        c: *mut PyCriticalSection2,
        m1: *mut PyMutex,
        m2: *mut PyMutex,
    );
}
