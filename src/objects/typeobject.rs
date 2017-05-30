// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std::ffi::CStr;
use std::borrow::Cow;

use ::pptr;
use ffi;
use token::PythonObjectWithGilToken;
use python::{Python, ToPythonPointer};
use conversion::ToPyTuple;
use objects::{PyObject, PyDict};
use err::PyResult;

/// Represents a reference to a Python type object.
pub struct PyType<'p>(pptr<'p>);

pyobject_nativetype!(PyType, PyType_Check, PyType_Type);


impl<'p> PyType<'p> {
    /// Retrieves the underlying FFI pointer associated with this Python object.
    #[inline]
    pub fn as_type_ptr(&self) -> *mut ffi::PyTypeObject {
        self.as_ptr() as *mut ffi::PyTypeObject
    }

    /// Retrieves the PyType instance for the given FFI pointer.
    /// This increments the reference count on the type object.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_type_ptr(py: Python<'p>, p: *mut ffi::PyTypeObject) -> PyType<'p> {
        PyType(pptr::from_borrowed_ptr(py, p as *mut ffi::PyObject))
    }

    /// Gets the name of the PyType.
    pub fn name<'a>(&'a self) -> Cow<'a, str> {
        unsafe {
            CStr::from_ptr((*self.as_type_ptr()).tp_name).to_string_lossy()
        }
    }

    /// Return true if `self` is a subtype of `b`.
    #[inline]
    pub fn is_subtype_of(&self, b: &PyType) -> bool {
        unsafe { ffi::PyType_IsSubtype(self.as_type_ptr(), b.as_type_ptr()) != 0 }
    }

    /// Return true if `obj` is an instance of `self`.
    #[inline]
    pub fn is_instance<T: ToPythonPointer>(&self, obj: &T) -> bool {
        unsafe { ffi::PyObject_TypeCheck(obj.as_ptr(), self.as_type_ptr()) != 0 }
    }

    // /// Calls the type object, thus creating a new instance.
    // /// This is equivalent to the Python expression: `self(*args, **kwargs)`
    #[inline]
    pub fn call<A>(&'p self, args: A, kwargs: Option<&PyDict>) -> PyResult<PyObject<'p>>
        where A: ToPyTuple
    {
        let args = args.to_py_tuple(self.gil());
        unsafe {
            PyObject::from_owned_ptr_or_err(
                self.gil(), ffi::PyObject_Call(self.as_ptr(), args.as_ptr(), kwargs.as_ptr()))
        }
    }
}

impl<'p> PartialEq for PyType<'p> {
    #[inline]
    fn eq(&self, other: &PyType) -> bool {
        self.as_type_ptr() == other.as_type_ptr()
    }
}
impl<'p> Eq for PyType<'p> { }
