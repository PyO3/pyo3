use libc::size_t;
use std::ffi::{c_int, c_void};

use crate::object::*;
use crate::pyport::Py_ssize_t;

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyObject_Malloc")]
    pub fn PyObject_Malloc(size: size_t) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyObject_Calloc")]
    pub fn PyObject_Calloc(nelem: size_t, elsize: size_t) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyObject_Realloc")]
    pub fn PyObject_Realloc(ptr: *mut c_void, new_size: size_t) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyObject_Free")]
    pub fn PyObject_Free(ptr: *mut c_void);

    // skipped PyObject_MALLOC
    // skipped PyObject_REALLOC
    // skipped PyObject_FREE
    // skipped PyObject_Del
    // skipped PyObject_DEL

    #[cfg_attr(PyPy, link_name = "PyPyObject_Init")]
    pub fn PyObject_Init(arg1: *mut PyObject, arg2: *mut PyTypeObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyObject_InitVar")]
    pub fn PyObject_InitVar(
        arg1: *mut PyVarObject,
        arg2: *mut PyTypeObject,
        arg3: Py_ssize_t,
    ) -> *mut PyVarObject;

    // skipped PyObject_INIT
    // skipped PyObject_INIT_VAR

    #[cfg_attr(PyPy, link_name = "_PyPyObject_New")]
    pub fn _PyObject_New(arg1: *mut PyTypeObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "_PyPyObject_NewVar")]
    pub fn _PyObject_NewVar(arg1: *mut PyTypeObject, arg2: Py_ssize_t) -> *mut PyVarObject;

    // skipped PyObject_New
    // skipped PyObject_NEW
    // skipped PyObject_NewVar
    // skipped PyObject_NEW_VAR

    pub fn PyGC_Collect() -> Py_ssize_t;

    #[cfg(Py_3_10)]
    #[cfg_attr(PyPy, link_name = "PyPyGC_Enable")]
    pub fn PyGC_Enable() -> c_int;

    #[cfg(Py_3_10)]
    #[cfg_attr(PyPy, link_name = "PyPyGC_Disable")]
    pub fn PyGC_Disable() -> c_int;

    #[cfg(Py_3_10)]
    #[cfg_attr(PyPy, link_name = "PyPyGC_IsEnabled")]
    pub fn PyGC_IsEnabled() -> c_int;

    // skipped PyUnstable_GC_VisitObjects
}

#[inline]
pub unsafe fn PyType_IS_GC(t: *mut PyTypeObject) -> c_int {
    PyType_HasFeature(t, Py_TPFLAGS_HAVE_GC)
}

extern "C" {
    pub fn _PyObject_GC_Resize(arg1: *mut PyVarObject, arg2: Py_ssize_t) -> *mut PyVarObject;

    // skipped PyObject_GC_Resize

    #[cfg_attr(PyPy, link_name = "_PyPyObject_GC_New")]
    pub fn _PyObject_GC_New(arg1: *mut PyTypeObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "_PyPyObject_GC_NewVar")]
    pub fn _PyObject_GC_NewVar(arg1: *mut PyTypeObject, arg2: Py_ssize_t) -> *mut PyVarObject;
    #[cfg(not(PyPy))]
    pub fn PyObject_GC_Track(arg1: *mut c_void);
    #[cfg(not(PyPy))]
    pub fn PyObject_GC_UnTrack(arg1: *mut c_void);
    #[cfg_attr(PyPy, link_name = "PyPyObject_GC_Del")]
    pub fn PyObject_GC_Del(arg1: *mut c_void);

    // skipped PyObject_GC_New
    // skipped PyObject_GC_NewVar

    #[cfg(any(all(Py_3_9, not(PyPy)), Py_3_10))] // added in 3.9, or 3.10 on PyPy
    #[cfg_attr(PyPy, link_name = "PyPyObject_GC_IsTracked")]
    pub fn PyObject_GC_IsTracked(arg1: *mut PyObject) -> c_int;
    #[cfg(any(all(Py_3_9, not(PyPy)), Py_3_10))] // added in 3.9, or 3.10 on PyPy
    #[cfg_attr(PyPy, link_name = "PyPyObject_GC_IsFinalized")]
    pub fn PyObject_GC_IsFinalized(arg1: *mut PyObject) -> c_int;
}

// skipped Py_VISIT
