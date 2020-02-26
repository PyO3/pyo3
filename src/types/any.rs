use crate::conversion::PyTryFrom;
use crate::err::PyDowncastError;
use crate::internal_tricks::Unsendable;
use crate::{ffi, PyObject};

/// Represents a python's [Any](https://docs.python.org/3/library/typing.html#typing.Any) type.
/// We can convert all python objects as `PyAny`.
///
/// In addition, if the inner object is an instance of type `T`, we can downcast
/// `PyAny` into `T`.
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
pub struct PyAny(PyObject, Unsendable);
unsafe impl crate::type_object::PyLayout<PyAny> for ffi::PyObject {
    unsafe fn get_ptr(&self) -> *mut PyAny {
        (&self) as *const &Self as *const _ as *mut _
    }
}
impl crate::type_object::PySizedLayout<PyAny> for ffi::PyObject {}
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
