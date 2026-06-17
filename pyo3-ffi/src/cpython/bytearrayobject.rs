use crate::object::*;
use crate::pyport::Py_ssize_t;
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
#[cfg(not(any(PyPy, GraalPy)))]
pub unsafe fn PyByteArray_AS_STRING(op: *mut PyObject) -> *mut c_char {
    let byte_array = op as *mut PyByteArrayObject;
    (*byte_array).ob_start
}

/*
#[inline]
#[cfg(Py_GIL_DISABLED)]
pub unsafe fn PyByteArray_GET_SIZE(op: *mut PyObject) -> Py_ssize_t {
  let byte_array = op as *mut PyByteArrayObject;
  // _Py_atomic_load_ssize_relaxed and _PyVarObject_CAST not implemented
  // Insert Rust equivalent of the next line:
  return _Py_atomic_load_ssize_relaxed(&(_PyVarObject_CAST(byte_array)->ob_size));
}
*/

#[inline]
#[cfg(not(Py_GIL_DISABLED))]
pub unsafe fn PyByteArray_GET_SIZE(op: *mut PyObject) -> Py_ssize_t {
    Py_SIZE(op)
}
