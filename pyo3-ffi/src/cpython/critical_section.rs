use crate::PyMutex;
use crate::{PyCriticalSection, PyCriticalSection2};

extern_libpython! {
    pub fn PyCriticalSection_BeginMutex(c: *mut PyCriticalSection, m: *mut PyMutex);
    pub fn PyCriticalSection2_BeginMutex(
        c: *mut PyCriticalSection2,
        m1: *mut PyMutex,
        m2: *mut PyMutex,
    );
}
