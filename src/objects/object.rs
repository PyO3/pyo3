// Copyright (c) 2017-present PyO3 Project and Contributors

use std;

use ::pyptr;
use ffi;
use err::{PyResult, PyDowncastError};
use python::{Python, ToPythonPointer};


pub struct PyObject<'p>(pyptr<'p>);

pyobject_nativetype!(PyObject, PyObject_Check, PyBaseObject_Type);


impl<'p> PyObject<'p> {

    #[inline]
    pub fn from_owned_ptr(py: Python<'p>, ptr: *mut ffi::PyObject) -> PyObject<'p> {
        unsafe { PyObject(pyptr::from_owned_ptr(py, ptr)) }
    }

    #[inline]
    pub fn from_owned_ptr_or_err(py: Python<'p>, ptr: *mut ffi::PyObject)
                                     -> PyResult<PyObject<'p>> {
        unsafe { Ok(PyObject(pyptr::from_owned_ptr_or_err(py, ptr)?)) }
    }

    #[inline]
    pub fn from_owned_ptr_or_panic(py: Python<'p>, ptr: *mut ffi::PyObject)
                                   -> PyObject<'p> {
        unsafe { PyObject(pyptr::from_owned_ptr_or_panic(py, ptr)) }
    }

    #[inline]
    pub fn from_owned_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject)
                                     -> Option<PyObject<'p>> {
        unsafe {
            if let Some(ptr) = pyptr::from_owned_ptr_or_opt(py, ptr) {
                Some(PyObject(ptr))
            } else {
                None
            }
        }
    }

    #[inline]
    pub fn from_borrowed_ptr(py: Python<'p>, ptr: *mut ffi::PyObject) -> PyObject<'p> {
        unsafe { PyObject(pyptr::from_borrowed_ptr(py, ptr)) }
    }

    #[inline]
    pub fn from_borrowed_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject)
                                    -> Option<PyObject<'p>> {
        unsafe {
            if let Some(ptr) = pyptr::from_borrowed_ptr_or_opt(py, ptr) {
                Some(PyObject(ptr))
            } else {
                None
            }
        }
    }

    /// Transmutes a slice of owned FFI pointers to `&[Py<'p, PyObject>]`.
    /// Undefined behavior if any pointer in the slice is NULL or invalid.
    #[inline]
    pub unsafe fn borrow_from_owned_ptr_slice<'a>(ptr: &'a [*mut ffi::PyObject])
                                                  -> &'a [PyObject<'p>] {
        std::mem::transmute(ptr)
    }

    /// Casts the PyObject to a concrete Python object type.
    /// Fails with `PyDowncastError` if the object is not of the expected type.
    #[inline]
    pub fn cast_as<D>(&'p self) -> Result<&'p D, PyDowncastError<'p>>
        where D: ::PyDowncastFrom<'p>
    {
        <D as ::PyDowncastFrom>::downcast_from(&self)
    }

    /// Casts the PyObject to a concrete Python object type.
    /// Fails with `PyDowncastError` if the object is not of the expected type.
    #[inline]
    pub fn cast_into<D>(self, py: Python<'p>) -> Result<D, PyDowncastError<'p>>
        where D: ::PyDowncastInto<'p>
    {
        <D as ::PyDowncastInto>::downcast_into(py, self)
    }

    /// Extracts some type from the Python object.
    /// This is a wrapper function around `FromPyObject::extract()`.
    #[inline]
    pub fn extract<D>(&'p self) -> PyResult<D> where D: ::conversion::FromPyObject<'p>
    {
        ::conversion::FromPyObject::extract(&self)
    }

    pub fn get_refcnt(&self) -> isize {
        unsafe { ffi::Py_REFCNT(self.0.as_ptr()) }

    }
}

impl<'p> PartialEq for PyObject<'p> {
    #[inline]
    fn eq(&self, other: &PyObject) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}
