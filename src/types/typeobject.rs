use crate::err::{self, PyResult};
use crate::{ffi, Bound, PyAny, PyTypeInfo, Python};
use std::borrow::Cow;
#[cfg(not(any(Py_LIMITED_API, PyPy)))]
use std::ffi::CStr;

/// Represents a reference to a Python `type object`.
#[repr(transparent)]
pub struct PyType(PyAny);

pyobject_native_type_core!(PyType, pyobject_native_static_type_object!(ffi::PyType_Type), #checkfunction=ffi::PyType_Check);

impl PyType {
    /// Creates a new type object.
    #[inline]
    pub fn new<T: PyTypeInfo>(py: Python<'_>) -> &PyType {
        T::type_object_bound(py).into_gil_ref()
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

    /// Gets the [qualified name](https://docs.python.org/3/glossary.html#term-qualified-name) of the `PyType`.
    pub fn qualname(&self) -> PyResult<String> {
        #[cfg(any(Py_LIMITED_API, PyPy, not(Py_3_11)))]
        let name = self.getattr(intern!(self.py(), "__qualname__"))?.extract();

        #[cfg(not(any(Py_LIMITED_API, PyPy, not(Py_3_11))))]
        let name = {
            use crate::ffi_ptr_ext::FfiPtrExt;
            use crate::types::any::PyAnyMethods;

            let obj = unsafe {
                ffi::PyType_GetQualName(self.as_type_ptr()).assume_owned_or_err(self.py())?
            };

            obj.extract()
        };

        name
    }

    /// Gets the full name, which includes the module, of the `PyType`.
    pub fn name(&self) -> PyResult<Cow<'_, str>> {
        #[cfg(not(any(Py_LIMITED_API, PyPy)))]
        {
            let ptr = self.as_type_ptr();

            let name = unsafe { CStr::from_ptr((*ptr).tp_name) }.to_str()?;

            #[cfg(Py_3_10)]
            if unsafe { ffi::PyType_HasFeature(ptr, ffi::Py_TPFLAGS_IMMUTABLETYPE) } != 0 {
                return Ok(Cow::Borrowed(name));
            }

            Ok(Cow::Owned(name.to_owned()))
        }

        #[cfg(any(Py_LIMITED_API, PyPy))]
        {
            let module = self.getattr(intern!(self.py(), "__module__"))?;

            #[cfg(not(Py_3_11))]
            let name = self.getattr(intern!(self.py(), "__name__"))?;

            #[cfg(Py_3_11)]
            let name = {
                use crate::ffi_ptr_ext::FfiPtrExt;

                unsafe { ffi::PyType_GetName(self.as_type_ptr()).assume_owned_or_err(self.py())? }
            };

            Ok(Cow::Owned(format!("{}.{}", module, name)))
        }
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
        self.is_subclass(T::type_object_bound(self.py()).as_gil_ref())
    }
}

/// Implementation of functionality for [`PyType`].
///
/// These methods are defined for the `Bound<'py, PyType>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
pub trait PyTypeMethods<'py> {
    /// Gets the [qualified name](https://docs.python.org/3/glossary.html#term-qualified-name) of the `PyType`.
    fn qualname(&self) -> PyResult<String>;
}

impl<'py> PyTypeMethods<'py> for Bound<'py, PyType> {
    fn qualname(&self) -> PyResult<String> {
        use crate::types::any::PyAnyMethods;
        #[cfg(any(Py_LIMITED_API, PyPy, not(Py_3_11)))]
        let name = self
            .as_any()
            .getattr(intern!(self.py(), "__qualname__"))?
            .extract();

        #[cfg(not(any(Py_LIMITED_API, PyPy, not(Py_3_11))))]
        let name = {
            use crate::ffi_ptr_ext::FfiPtrExt;

            let obj = unsafe {
                ffi::PyType_GetQualName(self.as_ptr() as *mut ffi::PyTypeObject)
                    .assume_owned_or_err(self.py())?
            };

            obj.extract()
        };

        name
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
