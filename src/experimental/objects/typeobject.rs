// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use crate::err::{PyErr, PyResult};
use crate::type_object::PyTypeObject;
use crate::{ffi, objects::{PyNativeObject, PyAny, PyStr}, types::Type, AsPyPointer, Python};

/// Represents a reference to a Python `type object`.
#[repr(transparent)]
pub struct PyType<'py>(pub(crate) PyAny<'py>);
pyo3_native_object!(PyType<'py>, Type, 'py);

impl<'py> PyType<'py> {
    /// Creates a new type object.
    #[inline]
    pub fn new<T: PyTypeObject>(py: Python<'py>) -> Self {
        T::type_object(py).to_owned()
    }

    /// Retrieves the underlying FFI pointer associated with this Python object.
    #[inline]
    pub fn as_type_ptr(&self) -> *mut ffi::PyTypeObject {
        self.as_ptr() as *mut ffi::PyTypeObject
    }

    /// Gets the name of the `PyType`.
    pub fn name(&self) -> PyResult<PyStr<'py>> {
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
