#[cfg(any(Py_3_14, Py_GIL_DISABLED))]
use crate::PyMutex;
use crate::PyObject;

#[repr(C)]
#[cfg(Py_GIL_DISABLED)]
pub struct PyCriticalSection {
    _cs_prev: usize,
    _cs_mutex: *mut PyMutex,
}

#[repr(C)]
#[cfg(Py_GIL_DISABLED)]
pub struct PyCriticalSection2 {
    _cs_base: PyCriticalSection,
    _cs_mutex2: *mut PyMutex,
}

#[cfg(not(Py_GIL_DISABLED))]
opaque_struct!(pub PyCriticalSection);

#[cfg(not(Py_GIL_DISABLED))]
opaque_struct!(pub PyCriticalSection2);

extern "C" {
    pub fn PyCriticalSection_Begin(c: *mut PyCriticalSection, op: *mut PyObject);
    #[cfg(Py_3_14)]
    pub fn PyCriticalSection_BeginMutex(c: *mut PyCriticalSection, m: *mut PyMutex);
    pub fn PyCriticalSection_End(c: *mut PyCriticalSection);
    pub fn PyCriticalSection2_Begin(c: *mut PyCriticalSection2, a: *mut PyObject, b: *mut PyObject);
    #[cfg(Py_3_14)]
    pub fn PyCriticalSection2_BeginMutex(
        c: *mut PyCriticalSection2,
        m1: *mut PyMutex,
        m2: *mut PyMutex,
    );
    pub fn PyCriticalSection2_End(c: *mut PyCriticalSection2);
}
