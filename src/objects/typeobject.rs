// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std::ffi::CStr;
use std::borrow::Cow;

use ffi;
use object::PyObject;
use python::{Python, ToPyPointer};
use err::{PyErr, PyResult};
use instance::{Py, PyObjectWithToken};
use typeob::{PyTypeInfo, PyTypeObject};

/// Represents a reference to a Python `type object`.
pub struct PyType(PyObject);

pyobject_convert!(PyType);
pyobject_nativetype!(PyType, PyType_Type, PyType_Check);


impl PyType {
    #[inline]
    pub fn new<T: PyTypeInfo>() -> Py<PyType> {
        unsafe {
            Py::from_borrowed_ptr(T::type_object() as *const _ as *mut ffi::PyObject)
        }
    }

    /// Retrieves the underlying FFI pointer associated with this Python object.
    #[inline]
    pub unsafe fn as_type_ptr(&self) -> *mut ffi::PyTypeObject {
        self.as_ptr() as *mut ffi::PyTypeObject
    }

    /// Retrieves the PyType instance for the given FFI pointer.
    /// This increments the reference count on the type object.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_type_ptr(py: Python, p: *mut ffi::PyTypeObject) -> &PyType
    {
        py.from_borrowed_ptr::<PyType>(p as *mut ffi::PyObject)
    }

    /// Gets the name of the PyType.
    pub fn name(&self) -> Cow<str> {
        unsafe {
            CStr::from_ptr((*self.as_type_ptr()).tp_name).to_string_lossy()
        }
    }

    /// Check whether `self` is subclass of type `T` like Python `issubclass` function
    pub fn is_subclass<T>(&self) -> PyResult<bool>
        where T: PyTypeObject
    {
        let result = unsafe {
            ffi::PyObject_IsSubclass(self.as_ptr(), T::type_object().as_ptr())
        };
        if result == -1 {
            Err(PyErr::fetch(self.py()))
        } else if result == 1 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    // Check whether `obj` is an instance of `self`
    pub fn is_instance<T: ToPyPointer>(&self, obj: &T) -> PyResult<bool> {
        let result = unsafe {
            ffi::PyObject_IsInstance(obj.as_ptr(), self.as_ptr())
        };
        if result == -1 {
            Err(PyErr::fetch(self.py()))
        } else if result == 1 {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
