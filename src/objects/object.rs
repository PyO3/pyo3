// Copyright (c) 2017-present PyO3 Project and Contributors

use std;

use ffi;
use pyptr::{Py, PyPtr};
use python::Python;

pub struct PyObject;

pyobject_newtype!(PyObject, PyObject_Check, PyBaseObject_Type);

impl PyObject {

    #[inline]
    pub fn from_owned_ptr(py: Python, ptr: *mut ffi::PyObject) -> Py<PyObject> {
        unsafe { Py::from_owned_ptr(py, ptr) }
    }

    #[inline]
    pub fn from_borrowed_ptr(py: Python, ptr: *mut ffi::PyObject) -> Py<PyObject> {
        unsafe { Py::from_borrowed_ptr(py, ptr) }
    }

    /// Creates a PyObject instance for the given FFI pointer.
    /// This moves ownership over the pointer into the PyObject.
    /// Returns None for null pointers; undefined behavior if the pointer is invalid.
    #[inline]
    pub unsafe fn from_owned_pptr_opt(py: Python, ptr: *mut ffi::PyObject)
                                      -> Option<PyPtr<PyObject>> {
        if ptr.is_null() {
            None
        } else {
            Some(PyObject::from_owned_ptr(py, ptr).into_pptr())
        }
    }

    /// Returns None for null pointers; undefined behavior if the pointer is invalid.
    #[inline]
    pub unsafe fn from_borrowed_pptr_opt(py: Python, ptr: *mut ffi::PyObject)
                                         -> Option<PyPtr<PyObject>> {
        if ptr.is_null() {
            None
        } else {
            Some(PyObject::from_borrowed_ptr(py, ptr).into_pptr())
        }
    }

    /// Transmutes a slice of owned FFI pointers to `&[Py<'p, PyObject>]`.
    /// Undefined behavior if any pointer in the slice is NULL or invalid.
    #[inline]
    pub unsafe fn borrow_from_owned_ptr_slice<'a>(ptr: &'a [*mut ffi::PyObject])
                                                  -> &'a [Py<'a, PyObject>] {
        std::mem::transmute(ptr)
    }
}
