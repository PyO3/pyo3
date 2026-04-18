use libc::size_t;
use std::ffi::{c_int, c_void};

use crate::object::*;
use crate::pyport::Py_ssize_t;

#[inline]
pub unsafe fn PyObject_Malloc(size: size_t) -> *mut c_void {
    crate::PyMem_Malloc(size)
}

#[inline]
pub unsafe fn PyObject_Calloc(nelem: size_t, elsize: size_t) -> *mut c_void {
    crate::PyMem_Calloc(nelem, elsize)
}

#[inline]
pub unsafe fn PyObject_Realloc(ptr: *mut c_void, new_size: size_t) -> *mut c_void {
    crate::PyMem_Realloc(ptr, new_size)
}

#[inline]
pub unsafe fn PyObject_Free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    unsafe { crate::PyMem_Free(ptr) };
}

#[inline]
pub unsafe fn PyObject_Init(arg1: *mut PyObject, _arg2: *mut PyTypeObject) -> *mut PyObject {
    arg1
}

#[inline]
pub unsafe fn PyObject_InitVar(
    arg1: *mut PyVarObject,
    _arg2: *mut PyTypeObject,
    arg3: Py_ssize_t,
) -> *mut PyVarObject {
    if !arg1.is_null() {
        (*arg1).ob_size = arg3;
    }
    arg1
}

#[inline]
pub unsafe fn PyObject_New<T>(_typeobj: *mut PyTypeObject) -> *mut T {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyObject_NewVar<T>(_typeobj: *mut PyTypeObject, _n: Py_ssize_t) -> *mut T {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyGC_Collect() -> Py_ssize_t {
    0
}

#[cfg(Py_3_10)]
#[inline]
pub unsafe fn PyGC_Enable() -> c_int {
    0
}

#[cfg(Py_3_10)]
#[inline]
pub unsafe fn PyGC_Disable() -> c_int {
    0
}

#[cfg(Py_3_10)]
#[inline]
pub unsafe fn PyGC_IsEnabled() -> c_int {
    1
}

#[inline]
pub unsafe fn PyType_IS_GC(t: *mut PyTypeObject) -> c_int {
    PyType_HasFeature(t, Py_TPFLAGS_HAVE_GC)
}

#[inline]
pub unsafe fn PyObject_GC_Resize<T>(_op: *mut PyObject, _n: Py_ssize_t) -> *mut T {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyObject_GC_New<T>(_typeobj: *mut PyTypeObject) -> *mut T {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyObject_GC_NewVar<T>(_typeobj: *mut PyTypeObject, _n: Py_ssize_t) -> *mut T {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn PyObject_GC_Track(_arg1: *mut c_void) {}

#[inline]
pub unsafe fn PyObject_GC_UnTrack(_arg1: *mut c_void) {}

#[inline]
pub unsafe fn PyObject_GC_Del(arg1: *mut c_void) {
    if arg1.is_null() {
        return;
    }
    unsafe { crate::PyMem_Free(arg1) };
}

#[cfg(any(all(Py_3_9, not(PyPy)), Py_3_10))]
#[inline]
pub unsafe fn PyObject_GC_IsTracked(_arg1: *mut PyObject) -> c_int {
    0
}

#[cfg(any(all(Py_3_9, not(PyPy)), Py_3_10))]
#[inline]
pub unsafe fn PyObject_GC_IsFinalized(_arg1: *mut PyObject) -> c_int {
    0
}
