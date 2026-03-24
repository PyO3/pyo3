#[cfg(not(Py_LIMITED_API))]
use crate::PyMutex;
use crate::PyObject;
// from typedefs.h
#[cfg(all(Py_3_15, Py_LIMITED_API))]
opaque_struct!(pub PyMutex);

#[cfg(Py_3_15)]
#[repr(C)]
pub struct PyCriticalSection_v1 {
    _cs_prev: usize,
    _cs_mutex: *mut PyMutex,
}

#[cfg(Py_3_15)]
#[repr(C)]
pub struct PyCriticalSection2_v1 {
    _cs_base: PyCriticalSection_v1,
    _cs_mutex2: *mut PyMutex,
}

extern "C" {
    pub fn PyCriticalSection_Begin_v1(c: *mut PyCriticalSection_v1, op: *mut PyObject);
    pub fn PyCriticalSection_Env_v1(c: *mut PyCriticalSection_v1);
    pub fn PyCriticalSection2_Begin_v1(
        c: *mut PyCriticalSection_v1,
        a: *mut PyObject,
        b: *mut PyObject,
    );
    pub fn PyCriticalSection2_Env_v1(c: *mut PyCriticalSection_v1);
}

#[cfg(Py_3_15)]
#[repr(C)]
pub struct PyCriticalSection_v0 {
    _cs: *mut PyCriticalSection_v1,
}

#[cfg(Py_3_15)]
#[repr(C)]
pub struct PyCriticalSection2_v0 {
    _cs: *mut PyCriticalSection2_v1,
}

extern "C" {
    pub fn PyCriticalSection_Begin_v0(c: *mut PyCriticalSection_v0, op: *mut PyObject);
    pub fn PyCriticalSection_End_v0(c: *mut PyCriticalSection_v0);
    pub fn PyCriticalSection2_Begin_v0(
        c: *mut PyCriticalSection2_v0,
        a: *mut PyObject,
        b: *mut PyObject,
    );
    pub fn PyCriticalSection2_End_v0(c: *mut PyCriticalSection2_v0);
}
