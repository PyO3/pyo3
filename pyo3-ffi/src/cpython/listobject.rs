use crate::object::*;
#[cfg(not(PyPy))]
use crate::pyport::Py_ssize_t;

#[cfg(not(PyPy))]
#[repr(C)]
pub struct PyListObject {
    pub ob_base: PyVarObject,
    pub ob_item: *mut *mut PyObject,
    pub allocated: Py_ssize_t,
}

#[cfg(PyPy)]
pub struct PyListObject {
    pub ob_base: PyObject,
}

// skipped _PyList_Extend
// skipped _PyList_DebugMallocStats
// skipped _PyList_CAST (used inline below)

/// Macro, trading safety for speed
#[inline]
#[cfg(not(any(PyPy, GraalPy)))]
pub unsafe fn PyList_GET_ITEM(op: *mut PyObject, i: Py_ssize_t) -> *mut PyObject {
    *(*(op as *mut PyListObject)).ob_item.offset(i)
}

/// Macro, *only* to be used to fill in brand new lists
#[inline]
#[cfg(not(any(PyPy, GraalPy)))]
pub unsafe fn PyList_SET_ITEM(op: *mut PyObject, i: Py_ssize_t, v: *mut PyObject) {
    *(*(op as *mut PyListObject)).ob_item.offset(i) = v;
}

#[inline]
#[cfg(not(PyPy))]
pub unsafe fn PyList_GET_SIZE(op: *mut PyObject) -> Py_ssize_t {
    Py_SIZE(op)
}
