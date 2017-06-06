// Copyright (c) 2017-present PyO3 Project and Contributors

use std;

use ffi;
use pointers::PyPtr;
use err::{PyErr, PyResult, PyDowncastError};
use python::{Python, ToPyPointer};
use conversion::FromPyObject;

pub struct PyObject(PyPtr);

pyobject_nativetype!(PyObject, PyObject_Check, PyBaseObject_Type);


impl PyObject {

    #[inline]
    pub fn from_owned_ptr(_py: Python, ptr: *mut ffi::PyObject) -> PyObject {
        unsafe { PyObject(PyPtr::from_owned_ptr(ptr)) }
    }

    #[inline]
    pub fn from_owned_ptr_or_err(py: Python, ptr: *mut ffi::PyObject)
                                     -> PyResult<PyObject> {
        Ok(PyObject(PyPtr::from_owned_ptr_or_err(py, ptr)?))
    }

    #[inline]
    pub fn from_owned_ptr_or_panic(_py: Python, ptr: *mut ffi::PyObject)
                                   -> PyObject {
        PyObject(PyPtr::from_owned_ptr_or_panic(ptr))
    }

    #[inline]
    pub fn from_owned_ptr_or_opt(_py: Python, ptr: *mut ffi::PyObject) -> Option<PyObject>
    {
        if ptr.is_null() {
            None
        } else {
            Some(PyObject(unsafe{PyPtr::from_owned_ptr(ptr)}))
        }
    }

    #[inline]
    pub fn from_borrowed_ptr(_py: Python, ptr: *mut ffi::PyObject) -> PyObject {
        unsafe { PyObject(PyPtr::from_borrowed_ptr(ptr)) }
    }

    #[inline]
    pub fn from_borrowed_ptr_or_opt(_py: Python, ptr: *mut ffi::PyObject) -> Option<PyObject>
    {
        if ptr.is_null() {
            None
        } else {
            Some(PyObject(unsafe{PyPtr::from_borrowed_ptr(ptr)}))
        }
    }

    #[inline]
    pub fn from_borrowed_ptr_or_err(py: Python, ptr: *mut ffi::PyObject) -> PyResult<PyObject>
    {
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(PyObject(unsafe{PyPtr::from_borrowed_ptr(ptr)}))
        }
    }

    /// Transmutes a slice of owned FFI pointers to `&[Py<'p, PyObject>]`.
    /// Undefined behavior if any pointer in the slice is NULL or invalid.
    #[inline]
    pub unsafe fn borrow_from_owned_ptr_slice<'a>(ptr: &'a [*mut ffi::PyObject])
                                                  -> &'a [PyObject] {
        std::mem::transmute(ptr)
    }

    /// Casts the PyObject to a concrete Python object type.
    /// Fails with `PyDowncastError` if the object is not of the expected type.
    #[inline]
    pub fn cast_as<'a, 'p, D>(&'a self, py: Python<'p>) -> Result<&'a D, PyDowncastError<'p>>
        where D: ::PyDowncastFrom
    {
        <D as ::PyDowncastFrom>::downcast_from(py, &self)
    }

    /// Casts the PyObject to a concrete Python object type.
    /// Fails with `PyDowncastError` if the object is not of the expected type.
    #[inline]
    pub fn cast_into<'p, D>(self, py: Python<'p>) -> Result<D, PyDowncastError<'p>>
        where D: ::PyDowncastInto
    {
        <D as ::PyDowncastInto>::downcast_into(py, self)
    }

    /// Casts the PyObject to a concrete Python object type.
    /// Fails with `PyDowncastError` if the object is not of the expected type.
    #[inline]
    pub fn unchecked_cast_into<'p, D>(self) -> D where D: ::PyDowncastInto
    {
        <D as ::PyDowncastInto>::unchecked_downcast_into(self)
    }

    /// Extracts some type from the Python object.
    /// This is a wrapper function around `FromPyObject::extract()`.
    #[inline]
    pub fn extract<'a, D>(&'a self, py: Python) -> PyResult<D> where D: FromPyObject<'a>
    {
        FromPyObject::extract(py, &self)
    }

    pub fn get_refcnt(&self) -> isize {
        unsafe { ffi::Py_REFCNT(self.0.as_ptr()) }

    }
}

impl<'p> PartialEq for PyObject {
    #[inline]
    fn eq(&self, other: &PyObject) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}
