// Copyright (c) 2017-present PyO3 Project and Contributors

use std;

use ffi;
use pyptr::{Py, PyPtr};
use err::{PyErr, PyResult, PyDowncastError};
use python::{Python, PythonToken, Token, PythonObjectWithToken};
use typeob::PyTypeInfo;

pub struct PyObject(PythonToken<PyObject>);

pyobject_newtype!(PyObject, PyObject_Check, PyBaseObject_Type);

impl PyObject {

    #[inline]
    pub fn from_owned_ptr(py: Token, ptr: *mut ffi::PyObject) -> Py<PyObject> {
        unsafe { Py::from_owned_ptr(py, ptr) }
    }

    #[inline]
    pub fn from_borrowed_ptr(py: Token, ptr: *mut ffi::PyObject) -> Py<PyObject> {
        unsafe { Py::from_borrowed_ptr(py, ptr) }
    }

    /// Creates a PyObject instance for the given FFI pointer.
    /// This moves ownership over the pointer into the PyObject.
    /// Returns None for null pointers; undefined behavior if the pointer is invalid.
    #[inline]
    pub unsafe fn from_owned_pptr_opt(py: Token, ptr: *mut ffi::PyObject)
                                      -> Option<PyPtr<PyObject>> {
        if ptr.is_null() {
            None
        } else {
            Some(PyObject::from_owned_ptr(py, ptr).into_pptr())
        }
    }

    /// Returns None for null pointers; undefined behavior if the pointer is invalid.
    #[inline]
    pub unsafe fn from_borrowed_pptr_opt(py: Token, ptr: *mut ffi::PyObject)
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

    /// Casts the PyObject to a concrete Python object type.
    /// Fails with `PyDowncastError` if the object is not of the expected type.
    #[inline]
    pub fn cast_as<'p, D>(&'p self) -> Result<&'p D, PyDowncastError<'p>>
        where D: PyTypeInfo
    {
        unsafe {
            let ptr = self as *const _ as *mut _;
            let checked = unsafe { ffi::PyObject_TypeCheck(ptr, D::type_object()) != 0 };

            if checked {
                Ok(
                    unsafe {
                        let ptr = ptr as *mut D;
                    ptr.as_ref().unwrap() })
            } else {
                Err(PyDowncastError(self.token(), None))
            }
        }
    }
}
