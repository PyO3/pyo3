use crate::conversion::AsPyPointer;
use crate::err::PyDowncastError;
use crate::{ffi, PyObject, PyRef, PyRefMut, PyTryFrom, PyTypeInfo};

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
/// assert!(any.downcast_ref::<PyDict>().is_ok());
/// assert!(any.downcast_ref::<PyList>().is_err());
/// ```
#[repr(transparent)]
pub struct PyAny(PyObject);
pyobject_native_type_named!(PyAny);
pyobject_native_type_convert!(PyAny, ffi::PyBaseObject_Type, ffi::PyObject_Check);

impl PyAny {
    pub fn downcast_ref<T>(&self) -> Result<&T, PyDowncastError>
    where
        T: for<'gil> PyTryFrom<'gil>,
    {
        T::try_from(self)
    }

    pub fn downcast_mut<T>(&self) -> Result<&mut T, PyDowncastError>
    where
        T: for<'gil> PyTryFrom<'gil>,
    {
        T::try_from_mut(self)
    }
}

impl<'a, T> From<PyRef<'a, T>> for &'a PyAny
where
    T: PyTypeInfo,
{
    fn from(pref: PyRef<'a, T>) -> &'a PyAny {
        unsafe { &*(pref.as_ptr() as *const PyAny) }
    }
}

impl<'a, T> From<PyRefMut<'a, T>> for &'a PyAny
where
    T: PyTypeInfo,
{
    fn from(pref: PyRefMut<'a, T>) -> &'a PyAny {
        unsafe { &*(pref.as_ptr() as *const PyAny) }
    }
}
