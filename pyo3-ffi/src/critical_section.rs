#[cfg(any(all(Py_GIL_DISABLED, Py_3_13, not(Py_LIMITED_API)), Py_3_15,))]
use crate::PyMutex;
#[cfg(any(all(Py_GIL_DISABLED, Py_3_13), Py_3_15))]
use crate::PyObject;

#[cfg(all(Py_LIMITED_API, not(Py_3_15)))]
opaque_struct!(pub PyMutex);

#[cfg(any(all(Py_GIL_DISABLED, Py_3_13, not(Py_LIMITED_API)), Py_3_15,))]
#[repr(C)]
pub struct PyCriticalSection {
    _cs_prev: usize,
    _cs_mutex: *mut PyMutex,
}

#[cfg(any(all(Py_GIL_DISABLED, Py_3_13, not(Py_LIMITED_API)), Py_3_15,))]
#[repr(C)]
pub struct PyCriticalSection2 {
    _cs_base: PyCriticalSection,
    _cs_mutex2: *mut PyMutex,
}

#[cfg(all(not(Py_GIL_DISABLED), Py_3_13, not(Py_3_15), not(Py_LIMITED_API)))]
opaque_struct!(pub PyCriticalSection);
#[cfg(all(not(Py_GIL_DISABLED), Py_3_13, not(Py_3_15), not(Py_LIMITED_API)))]
opaque_struct!(pub PyCriticalSection2);

#[cfg(any(all(Py_GIL_DISABLED, Py_3_13), Py_3_15))]
extern_libpython! {
    pub fn PyCriticalSection_Begin(c: *mut PyCriticalSection, op: *mut PyObject);
    pub fn PyCriticalSection_End(c: *mut PyCriticalSection);
    pub fn PyCriticalSection2_Begin(c: *mut PyCriticalSection2, a: *mut PyObject, b: *mut PyObject);
    pub fn PyCriticalSection2_End(c: *mut PyCriticalSection2);
}
