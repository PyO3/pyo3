// Copyright (c) 2017-present PyO3 Project and Contributors

//! Defines conversions between Rust and Python types.
use crate::err::{PyDowncastError, PyResult};
use crate::experimental::PyAny;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::pyclass::boolean_struct::False;
use crate::type_object::PyTypeInfo;
use crate::{ffi, AsPyPointer, PyCell, PyClass, PyNativeType, PyRef, PyRefMut};

/// Extract a type from a Python object.
///
///
/// Normal usage is through the `extract` methods on [`Py`] and  [`PyAny`], which forward to this trait.
///
/// # Examples
///
/// ```rust
/// use pyo3::prelude::*;
/// use pyo3::types::PyString;
///
/// # fn main() -> PyResult<()> {
/// Python::with_gil(|py| {
///     let obj: Py<PyString> = PyString::new(py, "blah").into();
///
///     // Straight from an owned reference
///     let s: &str = obj.extract(py)?;
/// #   assert_eq!(s, "blah");
///
///     // Or from a borrowed reference
///     let obj: &PyString = obj.as_ref(py);
///     let s: &str = obj.extract()?;
/// #   assert_eq!(s, "blah");
/// #   Ok(())
/// })
/// # }
/// ```
///
/// Note: depending on the implementation, the lifetime of the extracted result may
/// depend on the lifetime of the `obj` or the `prepared` variable.
///
/// For example, when extracting `&str` from a Python byte string, the resulting string slice will
/// point to the existing string data (lifetime: `'source`).
/// On the other hand, when extracting `&str` from a Python Unicode string, the preparation step
/// will convert the string to UTF-8, and the resulting string slice will have lifetime `'prepared`.
/// Since which case applies depends on the runtime type of the Python object,
/// both the `obj` and `prepared` variables must outlive the resulting string slice.
///
/// The trait's conversion method takes a `&PyAny` argument but is called
/// `FromPyObject` for historical reasons.
pub trait FromPyObject<'source, 'py>: Sized {
    /// Extracts `Self` from the source `PyObject`.
    fn extract(ob: &'source PyAny<'py>) -> PyResult<Self>;

    /// Extracts the type hint information for this type when it appears as an argument.
    ///
    /// For example, `Vec<u32>` would return `Sequence[int]`.
    /// The default implementation returns `Any`, which is correct for any type.
    ///
    /// For most types, the return value for this method will be identical to that of [`IntoPy::type_output`].
    /// It may be different for some types, such as `Dict`, to allow duck-typing: functions return `Dict` but take `Mapping` as argument.
    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        TypeInfo::Any
    }
}

impl<'py, T> FromPyObject<'py, 'py> for &'py PyCell<T>
where
    T: PyClass,
{
    fn extract(obj: &'py PyAny<'py>) -> PyResult<Self> {
        PyTryFrom::try_from(obj).map_err(Into::into)
    }
}

impl<T> FromPyObject<'_, '_> for T
where
    T: PyClass + Clone,
{
    fn extract(obj: &PyAny<'_>) -> PyResult<Self> {
        let cell: &PyCell<Self> = PyTryFrom::try_from(obj)?;
        Ok(unsafe { cell.try_borrow_unguarded()?.clone() })
    }
}

impl<'py, T> FromPyObject<'py, 'py> for PyRef<'py, T>
where
    T: PyClass,
{
    fn extract(obj: &'py PyAny<'py>) -> PyResult<Self> {
        let cell: &PyCell<T> = PyTryFrom::try_from(obj)?;
        cell.try_borrow().map_err(Into::into)
    }
}

impl<'py, T> FromPyObject<'py, 'py> for PyRefMut<'py, T>
where
    T: PyClass<Frozen = False>,
{
    fn extract(obj: &'py PyAny<'py>) -> PyResult<Self> {
        let cell: &PyCell<T> = PyTryFrom::try_from(obj)?;
        cell.try_borrow_mut().map_err(Into::into)
    }
}

impl<'source, 'py, T> FromPyObject<'source, 'py> for Option<T>
where
    T: FromPyObject<'source, 'py>,
{
    fn extract(obj: &'source PyAny<'py>) -> PyResult<Self> {
        if obj.as_ptr() == unsafe { ffi::Py_None() } {
            Ok(None)
        } else {
            T::extract(obj).map(Some)
        }
    }
}

/// Trait implemented by Python object types that allow a checked downcast.
/// If `T` implements `PyTryFrom`, we can convert `&PyAny` to `&T`.
///
/// This trait is similar to `std::convert::TryFrom`
pub trait PyTryFrom<'py>: Sized + PyNativeType + 'py {
    /// Cast from a concrete Python object type to PyObject.
    fn try_from<'a>(value: &'a PyAny<'py>) -> Result<&'a Self, PyDowncastError<'py>>;

    /// Cast from a concrete Python object type to PyObject. With exact type check.
    fn try_from_exact<'a>(value: &'a PyAny<'py>) -> Result<&'a Self, PyDowncastError<'py>>;

    /// Cast a PyAny to a specific type of PyObject. The caller must
    /// have already verified the reference is for this type.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the type is valid or risk type confusion.
    unsafe fn try_from_unchecked<'a>(value: &'a PyAny<'py>) -> &'a Self;
}

/// Trait implemented by Python object types that allow a checked downcast.
/// This trait is similar to `std::convert::TryInto`
pub trait PyTryInto<T>: Sized {
    /// Cast from PyObject to a concrete Python object type.
    fn try_into(&self) -> Result<&T, PyDowncastError<'_>>;

    /// Cast from PyObject to a concrete Python object type. With exact type check.
    fn try_into_exact(&self) -> Result<&T, PyDowncastError<'_>>;
}

// TryFrom implies TryInto
impl<'py, U> PyTryInto<U> for PyAny<'py>
where
    U: PyTryFrom<'py>,
{
    fn try_into(&self) -> Result<&U, PyDowncastError<'_>> {
        <U as PyTryFrom<'_>>::try_from(self)
    }
    fn try_into_exact(&self) -> Result<&U, PyDowncastError<'_>> {
        U::try_from_exact(self)
    }
}

impl<'py, T> PyTryFrom<'py> for T
where
    T: 'py + PyTypeInfo + PyNativeType,
{
    fn try_from<'a>(value: &'a PyAny<'py>) -> Result<&'a Self, PyDowncastError<'py>> {
        unsafe {
            if T::is_type_of(value.as_gil_ref()) {
                Ok(Self::try_from_unchecked(value))
            } else {
                Err(PyDowncastError::new(value.to_gil_ref(), T::NAME))
            }
        }
    }

    fn try_from_exact<'a>(value: &'a PyAny<'py>) -> Result<&'a Self, PyDowncastError<'py>> {
        unsafe {
            if T::is_exact_type_of(value.as_gil_ref()) {
                Ok(Self::try_from_unchecked(value))
            } else {
                Err(PyDowncastError::new(value.to_gil_ref(), T::NAME))
            }
        }
    }

    #[inline]
    unsafe fn try_from_unchecked<'a>(value: &'a PyAny<'py>) -> &'a Self {
        Self::unchecked_downcast(value.as_gil_ref())
    }
}

impl<'py, T> PyTryFrom<'py> for PyCell<T>
where
    T: 'py + PyClass,
{
    fn try_from<'a>(value: &'a PyAny<'py>) -> Result<&'a Self, PyDowncastError<'py>> {
        unsafe {
            if T::is_type_of(value.as_gil_ref()) {
                Ok(Self::try_from_unchecked(value))
            } else {
                Err(PyDowncastError::new(value.to_gil_ref(), T::NAME))
            }
        }
    }
    fn try_from_exact<'a>(value: &'a PyAny<'py>) -> Result<&'a Self, PyDowncastError<'py>> {
        unsafe {
            if T::is_exact_type_of(value.as_gil_ref()) {
                Ok(Self::try_from_unchecked(value))
            } else {
                Err(PyDowncastError::new(value.to_gil_ref(), T::NAME))
            }
        }
    }
    #[inline]
    unsafe fn try_from_unchecked<'a>(value: &'a PyAny<'py>) -> &'a Self {
        Self::unchecked_downcast(value.as_gil_ref())
    }
}

#[cfg(test)]
mod tests {
    use crate::experimental::PyAny;
    use crate::types::{IntoPyDict, PyDict, PyList};
    use crate::{AsPyPointer, PyObject, Python, ToPyObject};

    use super::PyTryFrom;

    #[test]
    fn test_try_from() {
        Python::with_gil(|py| {
            let list = PyAny::from_gil_ref(&vec![3, 6, 5, 4, 7].to_object(py).into_ref(py)).clone();
            let dict =
                PyAny::from_gil_ref(&vec![("reverse", true)].into_py_dict(py).as_ref()).clone();

            assert!(<PyList as PyTryFrom<'_>>::try_from(&list).is_ok());
            assert!(<PyDict as PyTryFrom<'_>>::try_from(&dict).is_ok());

            assert!(<PyAny<'_> as PyTryFrom<'_>>::try_from(&list).is_ok());
            assert!(<PyAny<'_> as PyTryFrom<'_>>::try_from(&dict).is_ok());
        });
    }

    #[test]
    fn test_try_from_exact() {
        Python::with_gil(|py| {
            let list = PyAny::from_gil_ref(&vec![3, 6, 5, 4, 7].to_object(py).into_ref(py)).clone();
            let dict =
                PyAny::from_gil_ref(&vec![("reverse", true)].into_py_dict(py).as_ref()).clone();

            assert!(PyList::try_from_exact(&list).is_ok());
            assert!(PyDict::try_from_exact(&dict).is_ok());

            assert!(PyAny::try_from_exact(&list).is_err());
            assert!(PyAny::try_from_exact(&dict).is_err());
        });
    }

    #[test]
    fn test_try_from_unchecked() {
        Python::with_gil(|py| {
            let list = PyAny::from_gil_ref(&PyList::new(py, &[1, 2, 3]).as_ref()).clone();
            let val = unsafe { <PyList as PyTryFrom>::try_from_unchecked(&list) };
            assert!(list.is(val));
        });
    }

    #[test]
    fn test_option_as_ptr() {
        Python::with_gil(|py| {
            let mut option: Option<PyObject> = None;
            assert_eq!(option.as_ptr(), std::ptr::null_mut());

            let none = py.None();
            option = Some(none.clone());

            let ref_cnt = none.get_refcnt(py);
            assert_eq!(option.as_ptr(), none.as_ptr());

            // Ensure ref count not changed by as_ptr call
            assert_eq!(none.get_refcnt(py), ref_cnt);
        });
    }
}
