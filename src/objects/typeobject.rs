// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std::ffi::CStr;
use std::borrow::Cow;

use ffi;
use pyptr::{PyPtr};
use token::{PythonToken, PythonObjectWithToken};
use python::{Python, ToPythonPointer};
use conversion::ToPyTuple;
use objects::{PyObject, PyDict};
use err::PyResult;

/// Represents a reference to a Python type object.
pub struct PyType(PythonToken<PyType>);

pyobject_newtype!(PyType, PyType_Check, PyType_Type);

impl PyType {
    /// Retrieves the underlying FFI pointer associated with this Python object.
    #[inline]
    pub fn as_type_ptr(&self) -> *mut ffi::PyTypeObject {
        self.as_ptr() as *mut ffi::PyTypeObject
    }

    /// Retrieves the PyType instance for the given FFI pointer.
    /// This increments the reference count on the type object.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_type_ptr(_py: Python, p: *mut ffi::PyTypeObject) -> PyPtr<PyType> {
        PyPtr::from_borrowed_ptr(p as *mut ffi::PyObject)
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
    pub fn is_instance(&self, _: Python, obj: &PyObject) -> bool {
        unsafe { ffi::PyObject_TypeCheck(obj.as_ptr(), self.as_type_ptr()) != 0 }
    }

    // /// Calls the type object, thus creating a new instance.
    // /// This is equivalent to the Python expression: `self(*args, **kwargs)`
    #[inline]
    pub fn call<'p, A>(&'p self, args: A, kwargs: Option<&PyDict>) -> PyResult<PyPtr<PyObject>>
        where A: ToPyTuple
    {
        let args = args.to_py_tuple(self.token());
        unsafe {
            PyPtr::from_owned_ptr_or_err(
                self.token(), ffi::PyObject_Call(self.as_ptr(), args.as_ptr(), kwargs.as_ptr()))
        }
    }
}

impl PartialEq for PyType {
    #[inline]
    fn eq(&self, o : &PyType) -> bool {
        self.as_type_ptr() == o.as_type_ptr()
    }
}
impl Eq for PyType { }
