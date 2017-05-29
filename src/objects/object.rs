// Copyright (c) 2017-present PyO3 Project and Contributors

use std;

use ::pptr;
use ffi;
use err::{PyResult, PyDowncastError};
use python::{Python, ToPythonPointer};
use token::PythonObjectWithToken;
use typeob::PyTypeInfo;


pub struct PyObject<'p>(pptr<'p>);

pyobject_nativetype!(PyObject, PyObject_Check, PyBaseObject_Type);


impl<'p> PyObject<'p> {

    #[inline]
    pub fn from_owned_ptr(py: Python<'p>, ptr: *mut ffi::PyObject) -> PyObject<'p> {
        unsafe { PyObject(pptr::from_owned_ptr(py, ptr)) }
    }

    #[inline]
    pub fn from_owned_ptr_or_err(py: Python<'p>, ptr: *mut ffi::PyObject)
                                     -> PyResult<PyObject<'p>> {
        unsafe { Ok(PyObject(pptr::from_owned_ptr_or_err(py, ptr)?)) }
    }

    #[inline]
    pub fn from_owned_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject)
                                     -> Option<PyObject<'p>> {
        unsafe {
            if let Some(ptr) = pptr::from_owned_ptr_or_opt(py, ptr) {
                Some(PyObject(ptr))
            } else {
                None
            }
        }
    }

    #[inline]
    pub fn from_borrowed_ptr(py: Python<'p>, ptr: *mut ffi::PyObject) -> PyObject<'p> {
        unsafe { PyObject(pptr::from_borrowed_ptr(py, ptr)) }
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
        where D: PyTypeInfo
    {
        unsafe {
            let ptr = self as *const _ as *mut _;
            let checked = ffi::PyObject_TypeCheck(ptr, D::type_object()) != 0;

            if checked {
                let ptr = ptr as *mut D;
                Ok(ptr.as_ref().unwrap())
            } else {
                Err(PyDowncastError(self.token(), None))
            }
        }
    }

    /// Extracts some type from the Python object.
    /// This is a wrapper function around `FromPyObject::extract()`.
    #[inline]
    pub fn extract<D>(&'p self) -> PyResult<D> where D: ::conversion::FromPyObject<'p>
    {
        ::conversion::FromPyObject::extract(&self)
    }
}
