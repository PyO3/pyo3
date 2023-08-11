use crate::err::{self, PyResult};
use crate::{ffi, PyAny, PyTypeInfo, Python};

/// Represents a reference to a Python `type object`.
#[repr(transparent)]
pub struct PyType(PyAny);

pyobject_native_type_core!(PyType, pyobject_native_static_type_object!(ffi::PyType_Type), #checkfunction=ffi::PyType_Check);

impl PyType {
    /// Creates a new type object.
    #[inline]
    pub fn new<T: PyTypeInfo>(py: Python<'_>) -> &PyType {
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
    pub unsafe fn from_type_ptr(py: Python<'_>, p: *mut ffi::PyTypeObject) -> &PyType {
        py.from_borrowed_ptr(p as *mut ffi::PyObject)
    }

    /// Gets the name of the `PyType`.
    pub fn name(&self) -> PyResult<&str> {
        self.getattr(intern!(self.py(), "__qualname__"))?.extract()
    }

    /// Checks whether `self` is a subclass of `other`.
    ///
    /// Equivalent to the Python expression `issubclass(self, other)`.
    pub fn is_subclass(&self, other: &PyAny) -> PyResult<bool> {
        let result = unsafe { ffi::PyObject_IsSubclass(self.as_ptr(), other.as_ptr()) };
        err::error_on_minusone(self.py(), result)?;
        Ok(result == 1)
    }

    /// Checks whether `self` is a subclass of type `T`.
    ///
    /// Equivalent to the Python expression `issubclass(self, T)`, if the type
    /// `T` is known at compile time.
    pub fn is_subclass_of<T>(&self) -> PyResult<bool>
    where
        T: PyTypeInfo,
    {
        self.is_subclass(T::type_object(self.py()))
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{PyBool, PyLong};
    use crate::Python;

    #[test]
    fn test_type_is_subclass() {
        Python::with_gil(|py| {
            let bool_type = py.get_type::<PyBool>();
            let long_type = py.get_type::<PyLong>();
            assert!(bool_type.is_subclass(long_type).unwrap());
        });
    }

    #[test]
    fn test_type_is_subclass_of() {
        Python::with_gil(|py| {
            assert!(py.get_type::<PyBool>().is_subclass_of::<PyLong>().unwrap());
        });
    }
}
