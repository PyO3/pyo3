#[cfg(Py_GIL_DISABLED)]
use crate::cpython::pyatomic::_Py_atomic_load_ssize_relaxed;
use crate::object::*;
#[cfg(not(PyPy))]
use crate::pyport::Py_ssize_t;
#[cfg(not(PyPy))]
use crate::PyList_Check;

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

#[inline]
#[cfg(not(PyPy))]
pub(crate) unsafe fn _PyList_CAST(op: *mut PyObject) -> *mut PyListObject {
    debug_assert_eq!(PyList_Check(op), 1);
    op.cast()
}

#[inline]
#[cfg(not(PyPy))]
pub unsafe fn PyList_GET_SIZE(op: *mut PyObject) -> Py_ssize_t {
    let list = _PyList_CAST(op);
    #[cfg(Py_GIL_DISABLED)]
    {
        _Py_atomic_load_ssize_relaxed(&raw const (*_PyVarObject_CAST(list.cast())).ob_size)
    }
    #[cfg(not(Py_GIL_DISABLED))]
    {
        Py_SIZE(list.cast())
    }
}

/// Macro, trading safety for speed
#[inline]
#[cfg(not(any(PyPy, GraalPy)))]
pub unsafe fn PyList_GET_ITEM(op: *mut PyObject, i: Py_ssize_t) -> *mut PyObject {
    *(*_PyList_CAST(op)).ob_item.offset(i)
}

/// Macro, *only* to be used to fill in brand new lists
#[inline]
#[cfg(not(any(PyPy, GraalPy)))]
pub unsafe fn PyList_SET_ITEM(op: *mut PyObject, i: Py_ssize_t, v: *mut PyObject) {
    *(*_PyList_CAST(op)).ob_item.offset(i) = v;
}

// skipped _PyList_Extend
// skipped _PyList_DebugMallocStats
