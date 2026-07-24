#[cfg(Py_GIL_DISABLED)]
use crate::cpython::pyatomic::_Py_atomic_load_ssize_relaxed;
use crate::object::*;
use crate::pyport::Py_ssize_t;
use crate::PyByteArray_Check;
#[cfg(not(any(PyPy, GraalPy)))]
use core::ffi::c_char;

#[cfg(not(any(PyPy, GraalPy, Py_LIMITED_API)))]
#[repr(C)]
pub struct PyByteArrayObject {
    pub ob_base: PyVarObject,
    pub ob_alloc: Py_ssize_t,
    pub ob_bytes: *mut c_char,
    pub ob_start: *mut c_char,
    pub ob_exports: Py_ssize_t,
    #[cfg(Py_3_15)]
    pub ob_bytes_object: *mut PyObject,
}

#[cfg(any(PyPy, GraalPy))]
opaque_struct!(pub PyByteArrayObject);

#[inline]
pub(crate) unsafe fn _PyByteArray_CAST(op: *mut PyObject) -> *mut PyByteArrayObject {
    debug_assert_eq!(PyByteArray_Check(op), 1);
    op.cast()
}

#[inline]
#[cfg(not(any(PyPy, GraalPy)))]
pub unsafe fn PyByteArray_AS_STRING(op: *mut PyObject) -> *mut c_char {
    (*_PyByteArray_CAST(op)).ob_start
}

#[inline]
pub unsafe fn PyByteArray_GET_SIZE(op: *mut PyObject) -> Py_ssize_t {
    let byte_array = _PyByteArray_CAST(op);
    #[cfg(Py_GIL_DISABLED)]
    {
        _Py_atomic_load_ssize_relaxed(&raw const (*_PyVarObject_CAST(byte_array.cast())).ob_size)
    }
    #[cfg(not(Py_GIL_DISABLED))]
    {
        Py_SIZE(byte_array.cast())
    }
}
