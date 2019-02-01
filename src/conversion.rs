// Copyright (c) 2017-present PyO3 Project and Contributors

//! This module contains some conversion traits

use crate::err::{PyDowncastError, PyResult};
use crate::ffi;
use crate::instance::Py;
use crate::object::PyObject;
use crate::python::{IntoPyPointer, Python, ToPyPointer};
use crate::typeob::PyTypeInfo;
use crate::types::{PyObjectRef, PyTuple};

/// Conversion trait that allows various objects to be converted into `PyObject`
pub trait ToPyObject {
    /// Converts self into a Python object.
    fn to_object(&self, py: Python) -> PyObject;
}

/// This trait has two implementations: The slow one is implemented for
/// all [ToPyObject] and creates a new object using [ToPyObject::to_object],
/// while the fast one is only implemented for ToPyPointer (we know
/// that every ToPyObject is also ToPyObject) and uses [ToPyPointer::as_ptr()]
pub trait ToBorrowedObject: ToPyObject {
    /// Converts self into a Python object and calls the specified closure
    /// on the native FFI pointer underlying the Python object.
    ///
    /// May be more efficient than `to_object` because it does not need
    /// to touch any reference counts when the input object already is a Python object.
    fn with_borrowed_ptr<F, R>(&self, py: Python, f: F) -> R
    where
        F: FnOnce(*mut ffi::PyObject) -> R,
    {
        let ptr = self.to_object(py).into_ptr();
        let result = f(ptr);
        unsafe {
            ffi::Py_XDECREF(ptr);
        }
        result
    }
}

impl<T> ToBorrowedObject for T where T: ToPyObject {}

impl<T> ToBorrowedObject for T
where
    T: ToPyObject + ToPyPointer,
{
    fn with_borrowed_ptr<F, R>(&self, _py: Python, f: F) -> R
    where
        F: FnOnce(*mut ffi::PyObject) -> R,
    {
        f(self.as_ptr())
    }
}

/// Conversion trait that allows various objects to be converted into `PyObject`
/// by consuming original object.
pub trait IntoPyObject {
    /// Converts self into a Python object. (Consumes self)
    fn into_object(self, py: Python) -> PyObject;
}

/// Conversion trait that allows various objects to be converted into `PyTuple` object.
pub trait IntoPyTuple {
    /// Converts self into a PyTuple object.
    fn into_tuple(self, py: Python) -> Py<PyTuple>;
}

/// `FromPyObject` is implemented by various types that can be extracted from
/// a Python object reference.
///
/// Normal usage is through the `PyObject::extract` helper method:
/// ```let obj: PyObject = ...;
/// let value: &TargetType = obj.extract(py)?;
/// ```
///
/// Note: depending on the implementation, the lifetime of the extracted result may
/// depend on the lifetime of the `obj` or the `prepared` variable.
///
/// For example, when extracting `&str` from a python byte string, the resulting string slice will
/// point to the existing string data (lifetime: `'source`).
/// On the other hand, when extracting `&str` from a python unicode string, the preparation step
/// will convert the string to UTF-8, and the resulting string slice will have lifetime `'prepared`.
/// Since only which of these cases applies depends on the runtime type of the python object,
/// both the `obj` and `prepared` variables must outlive the resulting string slice.
///
/// In cases where the result does not depend on the `'prepared` lifetime,
/// the inherent method `PyObject::extract()` can be used.
pub trait FromPyObject<'source>: Sized {
    /// Extracts `Self` from the source `PyObject`.
    fn extract(ob: &'source PyObjectRef) -> PyResult<Self>;
}

/// Identity conversion: allows using existing `PyObject` instances where
/// `T: ToPyObject` is expected.
impl<'a, T: ?Sized> ToPyObject for &'a T
where
    T: ToPyObject,
{
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        <T as ToPyObject>::to_object(*self, py)
    }
}

/// `Option::Some<T>` is converted like `T`.
/// `Option::None` is converted to Python `None`.
impl<T> ToPyObject for Option<T>
where
    T: ToPyObject,
{
    fn to_object(&self, py: Python) -> PyObject {
        match *self {
            Some(ref val) => val.to_object(py),
            None => py.None(),
        }
    }
}

impl<T> IntoPyObject for Option<T>
where
    T: IntoPyObject,
{
    fn into_object(self, py: Python) -> PyObject {
        match self {
            Some(val) => val.into_object(py),
            None => py.None(),
        }
    }
}

/// `()` is converted to Python `None`.
impl ToPyObject for () {
    fn to_object(&self, py: Python) -> PyObject {
        py.None()
    }
}

impl IntoPyObject for () {
    fn into_object(self, py: Python) -> PyObject {
        py.None()
    }
}

impl<'a, T> IntoPyObject for &'a T
where
    T: ToPyPointer,
{
    #[inline]
    fn into_object(self, py: Python) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, self.as_ptr()) }
    }
}

impl<'a, T> IntoPyObject for &'a mut T
where
    T: ToPyPointer,
{
    #[inline]
    fn into_object(self, py: Python) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, self.as_ptr()) }
    }
}

/// Extract reference to instance from `PyObject`
impl<'a, T> FromPyObject<'a> for &'a T
where
    T: PyTryFrom,
{
    #[inline]
    default fn extract(ob: &'a PyObjectRef) -> PyResult<&'a T> {
        Ok(T::try_from(ob)?)
    }
}

/// Extract mutable reference to instance from `PyObject`
impl<'a, T> FromPyObject<'a> for &'a mut T
where
    T: PyTryFrom,
{
    #[inline]
    default fn extract(ob: &'a PyObjectRef) -> PyResult<&'a mut T> {
        Ok(T::try_from_mut(ob)?)
    }
}

impl<'a, T> FromPyObject<'a> for Option<T>
where
    T: FromPyObject<'a>,
{
    fn extract(obj: &'a PyObjectRef) -> PyResult<Self> {
        if obj.as_ptr() == unsafe { ffi::Py_None() } {
            Ok(None)
        } else {
            match T::extract(obj) {
                Ok(v) => Ok(Some(v)),
                Err(e) => Err(e),
            }
        }
    }
}

/// Trait implemented by Python object types that allow a checked downcast.
/// This trait is similar to `std::convert::TryInto`
pub trait PyTryInto<T>: Sized {
    /// The type returned in the event of a conversion error.
    type Error;

    /// Cast from PyObject to a concrete Python object type.
    fn try_into(&self) -> Result<&T, Self::Error>;

    /// Cast from PyObject to a concrete Python object type. With exact type check.
    fn try_into_exact(&self) -> Result<&T, Self::Error>;

    /// Cast from PyObject to a concrete Python object type.
    fn try_into_mut(&self) -> Result<&mut T, Self::Error>;

    /// Cast from PyObject to a concrete Python object type. With exact type check.
    fn try_into_mut_exact(&self) -> Result<&mut T, Self::Error>;
}

/// Trait implemented by Python object types that allow a checked downcast.
/// This trait is similar to `std::convert::TryFrom`
pub trait PyTryFrom: Sized {
    /// Cast from a concrete Python object type to PyObject.
    fn try_from(value: &PyObjectRef) -> Result<&Self, PyDowncastError>;

    /// Cast from a concrete Python object type to PyObject. With exact type check.
    fn try_from_exact(value: &PyObjectRef) -> Result<&Self, PyDowncastError>;

    /// Cast from a concrete Python object type to PyObject.
    fn try_from_mut(value: &PyObjectRef) -> Result<&mut Self, PyDowncastError>;

    /// Cast from a concrete Python object type to PyObject. With exact type check.
    fn try_from_mut_exact(value: &PyObjectRef) -> Result<&mut Self, PyDowncastError>;

    /// Cast a PyObjectRef to a specific type of PyObject. The caller must
    /// have already verified the reference is for this type.
    unsafe fn try_from_unchecked(value: &PyObjectRef) -> &Self;

    /// Cast a PyObjectRef to a specific type of PyObject. The caller must
    /// have already verified the reference is for this type.
    #[allow(clippy::mut_from_ref)]
    unsafe fn try_from_mut_unchecked(value: &PyObjectRef) -> &mut Self;
}

// TryFrom implies TryInto
impl<U> PyTryInto<U> for PyObjectRef
where
    U: PyTryFrom,
{
    type Error = PyDowncastError;

    fn try_into(&self) -> Result<&U, PyDowncastError> {
        U::try_from(self)
    }
    fn try_into_exact(&self) -> Result<&U, PyDowncastError> {
        U::try_from_exact(self)
    }
    fn try_into_mut(&self) -> Result<&mut U, PyDowncastError> {
        U::try_from_mut(self)
    }
    fn try_into_mut_exact(&self) -> Result<&mut U, PyDowncastError> {
        U::try_from_mut_exact(self)
    }
}

impl<T> PyTryFrom for T
where
    T: PyTypeInfo,
{
    fn try_from(value: &PyObjectRef) -> Result<&T, PyDowncastError> {
        unsafe {
            if T::is_instance(value) {
                Ok(PyTryFrom::try_from_unchecked(value))
            } else {
                Err(PyDowncastError)
            }
        }
    }

    fn try_from_exact(value: &PyObjectRef) -> Result<&T, PyDowncastError> {
        unsafe {
            if T::is_exact_instance(value) {
                Ok(PyTryFrom::try_from_unchecked(value))
            } else {
                Err(PyDowncastError)
            }
        }
    }

    fn try_from_mut(value: &PyObjectRef) -> Result<&mut T, PyDowncastError> {
        unsafe {
            if T::is_instance(value) {
                Ok(PyTryFrom::try_from_mut_unchecked(value))
            } else {
                Err(PyDowncastError)
            }
        }
    }

    fn try_from_mut_exact(value: &PyObjectRef) -> Result<&mut T, PyDowncastError> {
        unsafe {
            if T::is_exact_instance(value) {
                Ok(PyTryFrom::try_from_mut_unchecked(value))
            } else {
                Err(PyDowncastError)
            }
        }
    }

    #[inline]
    unsafe fn try_from_unchecked(value: &PyObjectRef) -> &T {
        let ptr = if T::OFFSET == 0 {
            value as *const _ as *const u8 as *const T
        } else {
            (value.as_ptr() as *const u8).offset(T::OFFSET) as *const T
        };
        &*ptr
    }

    #[inline]
    unsafe fn try_from_mut_unchecked(value: &PyObjectRef) -> &mut T {
        let ptr = if T::OFFSET == 0 {
            value as *const _ as *mut u8 as *mut T
        } else {
            (value.as_ptr() as *mut u8).offset(T::OFFSET) as *mut T
        };
        &mut *ptr
    }
}

/// This trait wraps a T: IntoPyObject into PyResult<T> while PyResult<T> remains PyResult<T>.
///
/// This is necessaty because proc macros run before typechecking and can't decide
/// whether a return type is a (possibly aliased) PyResult or not. It is also quite handy because
/// the codegen is currently built on the assumption that all functions return a PyResult.
pub trait ReturnTypeIntoPyResult {
    type Inner;

    fn return_type_into_py_result(self) -> PyResult<Self::Inner>;
}

impl<T: IntoPyObject> ReturnTypeIntoPyResult for T {
    type Inner = T;

    fn return_type_into_py_result(self) -> PyResult<Self::Inner> {
        Ok(self)
    }
}

impl<T: IntoPyObject> ReturnTypeIntoPyResult for PyResult<T> {
    type Inner = T;

    fn return_type_into_py_result(self) -> PyResult<Self::Inner> {
        self
    }
}



#[cfg(test)]
mod test {
    use super::PyTryFrom;
    use crate::types::PyList;
    use crate::Python;

    #[test]
    fn test_try_from_unchecked() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let list = PyList::new(py, &[1, 2, 3]);
        let val = unsafe { <PyList as PyTryFrom>::try_from_unchecked(list.as_ref()) };
        assert_eq!(list, val);
    }
}
