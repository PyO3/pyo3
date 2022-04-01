use crate::object::*;
use crate::pyport::Py_ssize_t;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyListObject {
    pub ob_base: PyVarObject,
    pub ob_item: *mut *mut PyObject,
    pub allocated: Py_ssize_t,
}

// skipped _PyList_Extend
// skipped _PyList_DebugMallocStats
// skipped _PyList_CAST (used inline below)

/// Macro, trading safety for speed
#[inline]
#[cfg(not(PyPy))]
pub unsafe fn PyList_GET_ITEM(op: *mut PyObject, i: Py_ssize_t) -> *mut PyObject {
    *(*(op as *mut PyListObject)).ob_item.offset(i as isize)
}

/// Macro, *only* to be used to fill in brand new lists
#[inline]
#[cfg(not(PyPy))]
pub unsafe fn PyList_SET_ITEM(op: *mut PyObject, i: Py_ssize_t, v: *mut PyObject) {
    *(*(op as *mut PyListObject)).ob_item.offset(i as isize) = v;
}

#[inline]
#[cfg(not(PyPy))]
pub unsafe fn PyList_GET_SIZE(op: *mut PyObject) -> Py_ssize_t {
    Py_SIZE(op)
}
