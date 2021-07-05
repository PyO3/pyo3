// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use crate::err::{PyErr, PyResult};
use crate::instance::PyNativeType;
use crate::type_object::PyTypeObject;
use crate::{ffi, AsPyPointer, PyAny, Python};

/// Represents a reference to a Python `type object`.
#[repr(transparent)]
pub struct PyType(PyAny);

pyobject_native_type_core!(PyType, ffi::PyType_Type, #checkfunction=ffi::PyType_Check);

impl PyType {
    /// Creates a new type object.
    #[inline]
    pub fn new<T: PyTypeObject>(py: Python) -> &PyType {
        T::type_object(py)
    }

    /// Retrieves the underlying FFI pointer associated with this Python object.
    #[inline]
    pub fn as_type_ptr(&self) -> *mut ffi::PyTypeObject {
        self.as_ptr() as *mut ffi::PyTypeObject
    }

    /// Retrieves the `PyType` instance for the given FFI pointer.
    ///
    /// # Safety
    /// - The pointer must be non-null.
    /// - The pointer must be valid for the entire of the lifetime for which the reference is used.
    #[inline]
    pub unsafe fn from_type_ptr(py: Python, p: *mut ffi::PyTypeObject) -> &PyType {
        py.from_borrowed_ptr(p as *mut ffi::PyObject)
    }

    /// Gets the name of the `PyType`.
    pub fn name(&self) -> PyResult<&str> {
        self.getattr("__qualname__")?.extract()
    }

    /// Checks whether `self` is subclass of type `T`.
    ///
    /// Equivalent to Python's `issubclass` function.
    pub fn is_subclass<T>(&self) -> PyResult<bool>
    where
        T: PyTypeObject,
    {
        let result =
            unsafe { ffi::PyObject_IsSubclass(self.as_ptr(), T::type_object(self.py()).as_ptr()) };
        if result == -1 {
            Err(PyErr::api_call_failed(self.py()))
        } else if result == 1 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check whether `obj` is an instance of `self`.
    ///
    /// Equivalent to Python's `isinstance` function.
    pub fn is_instance<T: AsPyPointer>(&self, obj: &T) -> PyResult<bool> {
        let result = unsafe { ffi::PyObject_IsInstance(obj.as_ptr(), self.as_ptr()) };
        if result == -1 {
            Err(PyErr::api_call_failed(self.py()))
        } else if result == 1 {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
