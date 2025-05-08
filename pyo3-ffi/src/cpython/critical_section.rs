#[cfg(Py_GIL_DISABLED)]
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
    pub fn PyCriticalSection_End(c: *mut PyCriticalSection);
    pub fn PyCriticalSection2_Begin(c: *mut PyCriticalSection2, a: *mut PyObject, b: *mut PyObject);
    pub fn PyCriticalSection2_End(c: *mut PyCriticalSection2);
}
