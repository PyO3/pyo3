// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use crate::err::{PyErr, PyResult};
use crate::ffi;
use crate::instance::PyNativeType;
use crate::type_object::{PyTypeInfo, PyTypeObject};
use crate::types::PyAny;
use crate::AsPyPointer;
use crate::Python;
use std::borrow::Cow;
use std::ffi::CStr;

/// Represents a reference to a Python `type object`.
#[repr(transparent)]
pub struct PyType<'a>(PyAny<'a>);

pyobject_native_var_type!(PyType<'py>, ffi::PyType_Type, ffi::PyType_Check);

impl<'py> PyType<'py> {
    /// Creates a new type object.
    #[inline]
    pub fn new<T: PyTypeInfo<'py>>(py: Python<'py>) -> Self {
        <T as PyTypeObject>::type_object().into_scoped(py)
    }

    /// Retrieves the underlying FFI pointer associated with this Python object.
    #[inline]
    pub unsafe fn as_type_ptr(&self) -> *mut ffi::PyTypeObject {
        self.as_ptr() as *mut ffi::PyTypeObject
    }

    /// Retrieves the `PyType` instance for the given FFI pointer.
    /// This increments the reference count on the type object.
    /// Undefined behavior if the pointer is NULL or invalid.
    #[inline]
    pub unsafe fn from_type_ptr(py: Python<'py>, p: *mut ffi::PyTypeObject) -> &Self {
        py.from_borrowed_ptr(p as *mut ffi::PyObject)
    }

    /// Gets the name of the `PyType`.
    pub fn name(&self) -> Cow<str> {
        unsafe { CStr::from_ptr((*self.as_type_ptr()).tp_name).to_string_lossy() }
    }

    /// Checks whether `self` is subclass of type `T`.
    ///
    /// Equivalent to Python's `issubclass` function.
    pub fn is_subclass<T>(&self) -> PyResult<bool>
    where
        T: PyTypeInfo<'py>,
    {
        let result = unsafe { ffi::PyObject_IsSubclass(self.as_ptr(), T::type_object() as *const _ as *mut ffi::PyTypeObject as _)};
        if result == -1 {
            Err(PyErr::fetch(self.py()))
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
            Err(PyErr::fetch(self.py()))
        } else if result == 1 {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
