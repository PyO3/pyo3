use crate::conversion::{FromPyPointer, PyTryFrom};
use crate::err::PyDowncastError;
use crate::ffi;
use crate::gil;
use crate::python::Python;
use crate::type_marker;
use std::marker::PhantomData;
use std::ptr::NonNull;

/// A Python object with GIL lifetime
///
/// Represents any Python object.  All Python objects can be cast to `PyAny`.
/// In addition, if the inner object is an instance of type `T`, we can downcast
/// `PyAny` into `T`.
///
/// `PyAny` is used as a reference with a lifetime that represents that the GIL
/// is held, therefore its API does not require a `Python<'py>` token.
///
/// See [the guide](https://pyo3.rs/master/types.html) for an explanation
/// of the different Python object types.
///
/// # Example
///
/// ```
/// use pyo3::prelude::*;
/// use pyo3::types::{PyAny, PyDict, PyList};
/// let gil = Python::acquire_gil();
/// let dict = PyDict::new(gil.python());
/// assert!(gil.python().is_instance::<PyAny, _>(dict).unwrap());
/// let any = dict.as_ref();
/// assert!(any.downcast::<PyDict>().is_ok());
/// assert!(any.downcast::<PyList>().is_err());
/// ```
#[repr(transparent)]
pub struct PyAny<'a>(NonNull<ffi::PyObject>, PhantomData<Python<'a>>);

impl<'py> crate::type_object::PySizedLayout<'py, type_marker::Any> for ffi::PyObject {}
pyobject_native_type_named!(PyAny<'py>);
pyobject_native_type_common!(PyAny<'py>);
pyobject_native_type_info!(
    PyAny<'py>,
    ffi::PyObject,
    ffi::PyBaseObject_Type,
    ffi::PyObject_Check,
    type_marker::Any
);
pyobject_native_type_extract!(PyAny<'py>);

impl<'py> PyAny<'py> {
    pub fn downcast<T>(&self) -> Result<&T, PyDowncastError>
    where
        T: PyTryFrom<'py>,
    {
        <T as PyTryFrom>::try_from(self)
    }

    /// Create a PyAny from an owned non-null raw PyObject pointer.
    ///
    /// # Safety
    ///
    /// It must be ensured that the pointer is an owned reference.
    pub unsafe fn from_non_null(_py: Python<'py>, ptr: NonNull<ffi::PyObject>) -> Self {
        Self(ptr, PhantomData)
    }

    /// Create an owned non-null raw PyObject pointer from this PyAny.
    pub fn into_non_null(self) -> NonNull<ffi::PyObject> {
        // Destructure self so that Drop(self) is not called.
        // (Alternative would be to call std::mem::forget(self).)
        let PyAny(ptr, _) = self;
        ptr
    }

    pub fn raw_borrowed<'a>(_py: Python<'py>, ptr_ref: &'a *mut ffi::PyObject) -> &'a Self {
        unsafe { std::mem::transmute(ptr_ref) }
    }
}

impl Clone for PyAny<'_> {
    fn clone(&self) -> Self {
        unsafe { ffi::Py_INCREF(self.0.as_ptr()); }
        Self(self.0, PhantomData)
    }
}

impl std::convert::AsRef<Self> for PyAny<'_> {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Drop for PyAny<'_> {
    fn drop(&mut self) {
        unsafe { ffi::Py_DECREF(self.0.as_ptr()); };
    }
}

unsafe impl<'py> FromPyPointer<'py> for PyAny<'py>
{
    unsafe fn from_owned_ptr_or_opt(py: Python<'py>, ptr: *mut ffi::PyObject) -> Option<Self> {
        Some(Self::from_non_null(py, NonNull::new(ptr)?))
    }
    unsafe fn from_borrowed_ptr_or_opt(
        py: Python<'py>,
        ptr: *mut ffi::PyObject,
    ) -> Option<&'py Self> {
        Some(gil::register_borrowed(py, NonNull::new(ptr)?))
    }
}
