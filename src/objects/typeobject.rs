// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std::ffi::CStr;
use std::borrow::Cow;

use ffi;
use token::PyObjectWithGilToken;
use pointers::{Ptr, PyPtr};
use python::{Python, ToPyPointer};
use conversion::ToPyTuple;
use objects::{PyObject, PyDict};
use err::{PyErr, PyResult};

/// Represents a reference to a Python type object.
pub struct PyType<'p>(Ptr<'p>);
pub struct PyTypePtr(PyPtr);

pyobject_convert!(PyType);
pyobject_nativetype!(PyType, PyType_Check, PyType_Type, PyTypePtr);


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
        PyType(Ptr::from_borrowed_ptr(py, p as *mut ffi::PyObject))
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
    pub fn is_instance<T: ToPyPointer>(&self, obj: &T) -> bool {
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


impl PyTypePtr {
    /// Creates a `PyTypePtr` instance for the given FFI pointer.
    /// This moves ownership over the pointer into the `PyTypePtr`.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_owned_ptr(ptr: *mut ffi::PyObject) -> PyTypePtr {
        PyTypePtr(PyPtr::from_owned_ptr(ptr))
    }

    /// Retrieves the owned PyTypePtr instance for the given FFI pointer.
    /// Returns `Err(PyErr)` if the pointer is `null`; undefined behavior if the
    /// pointer is invalid
    #[inline]
    pub unsafe fn from_owned_ptr_or_err(py: Python, ptr: *mut ffi::PyObject)
                                        -> PyResult<PyTypePtr> {
        if ptr.is_null() {
            Err(PyErr::fetch(py))
        } else {
            Ok(PyTypePtr(PyPtr::from_owned_ptr(ptr)))
        }
    }

    /// Creates a `PyTypePtr` instance for the given Python FFI pointer.
    /// Calls Py_INCREF() on the ptr.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_borrowed_ptr(ptr: *mut ffi::PyObject) -> PyTypePtr {
        PyTypePtr(PyPtr::from_owned_ptr(ptr))
    }
}
