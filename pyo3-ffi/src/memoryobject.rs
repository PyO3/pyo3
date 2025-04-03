use crate::object::*;
use crate::pyport::Py_ssize_t;
use std::os::raw::{c_char, c_int};
use std::ptr::addr_of_mut;

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg(not(Py_LIMITED_API))]
    pub static mut _PyManagedBuffer_Type: PyTypeObject;

    #[cfg_attr(PyPy, link_name = "PyPyMemoryView_Type")]
    pub static mut PyMemoryView_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyMemoryView_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyMemoryView_Type)) as c_int
}

// skipped non-limited PyMemoryView_GET_BUFFER
// skipped non-limited PyMemeryView_GET_BASE

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyMemoryView_FromObject")]
    pub fn PyMemoryView_FromObject(base: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyMemoryView_FromMemory")]
    pub fn PyMemoryView_FromMemory(
        mem: *mut c_char,
        size: Py_ssize_t,
        flags: c_int,
    ) -> *mut PyObject;
    #[cfg(any(Py_3_11, not(Py_LIMITED_API)))]
    #[cfg_attr(PyPy, link_name = "PyPyMemoryView_FromBuffer")]
    pub fn PyMemoryView_FromBuffer(view: *const crate::Py_buffer) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyMemoryView_GetContiguous")]
    pub fn PyMemoryView_GetContiguous(
        base: *mut PyObject,
        buffertype: c_int,
        order: c_char,
    ) -> *mut PyObject;
}

// skipped remainder of file with comment:
/* The structs are declared here so that macros can work, but they shouldn't
be considered public. Don't access their fields directly, use the macros
and functions instead! */
