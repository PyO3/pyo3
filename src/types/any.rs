use crate::conversion::PyTryFrom;
use crate::err::PyDowncastError;
use crate::ffi;
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
pub struct PyAny<'a>(NonNull<ffi::PyObject>, PhantomData<&'a ffi::PyObject>);
unsafe impl crate::type_object::PyLayout<PyAny<'_>> for ffi::PyObject {}
impl crate::type_object::PySizedLayout<PyAny<'_>> for ffi::PyObject {}
pyobject_native_type_named!(PyAny);
pyobject_native_type_convert!(
    PyAny,
    ffi::PyObject,
    ffi::PyBaseObject_Type,
    Some("builtins"),
    ffi::PyObject_Check
);
pyobject_native_type_extract!(PyAny);

impl PyAny {
    pub fn downcast<T>(&self) -> Result<&T, PyDowncastError>
    where
        for<'py> T: PyTryFrom<'py>,
    {
        <T as PyTryFrom>::try_from(self)
    }
}
