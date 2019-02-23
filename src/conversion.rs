// Copyright (c) 2017-present PyO3 Project and Contributors

//! Conversions between various states of rust and python types and their wrappers.

use crate::err::{PyDowncastError, PyResult};
use crate::ffi;
use crate::object::PyObject;
use crate::type_object::PyTypeInfo;
use crate::types::PyObjectRef;
use crate::types::PyTuple;
use crate::Py;
use crate::Python;

/// This trait allows retrieving the underlying FFI pointer from Python objects.
///
/// This trait is implemented for types that internally wrap a pointer to a python object.
pub trait ToPyPointer {
    /// Retrieves the underlying FFI pointer (as a borrowed pointer).
    fn as_ptr(&self) -> *mut ffi::PyObject;
}

/// This trait allows retrieving the underlying FFI pointer from Python objects.
pub trait IntoPyPointer {
    /// Retrieves the underlying FFI pointer. Whether pointer owned or borrowed
    /// depends on implementation.
    fn into_ptr(self) -> *mut ffi::PyObject;
}

/// Convert `None` into a null pointer.
impl<T> ToPyPointer for Option<T>
where
    T: ToPyPointer,
{
    #[inline]
    fn as_ptr(&self) -> *mut ffi::PyObject {
        match *self {
            Some(ref t) => t.as_ptr(),
            None => std::ptr::null_mut(),
        }
    }
}

/// Convert `None` into a null pointer.
impl<T> IntoPyPointer for Option<T>
where
    T: IntoPyPointer,
{
    #[inline]
    fn into_ptr(self) -> *mut ffi::PyObject {
        match self {
            Some(t) => t.into_ptr(),
            None => std::ptr::null_mut(),
        }
    }
}

impl<'a, T> IntoPyPointer for &'a T
where
    T: ToPyPointer,
{
    fn into_ptr(self) -> *mut ffi::PyObject {
        let ptr = self.as_ptr();
        if !ptr.is_null() {
            unsafe {
                ffi::Py_INCREF(ptr);
            }
        }
        ptr
    }
}

/// Conversion trait that allows various objects to be converted into `PyObject`
pub trait ToPyObject {
    /// Converts self into a Python object.
    fn to_object(&self, py: Python) -> PyObject;
}

/// This trait has two implementations: The slow one is implemented for
/// all [ToPyObject] and creates a new object using [ToPyObject::to_object],
/// while the fast one is only implemented for ToPyPointer (we know
/// that every ToPyPointer is also ToPyObject) and uses [ToPyPointer::as_ptr()]
///
/// This trait should eventually be replaced with [ManagedPyRef](crate::ManagedPyRef).
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

/// Similar to [std::convert::From], just that it requires a gil token.
pub trait FromPy<T>: Sized {
    /// Performs the conversion.
    fn from_py(_: T, py: Python) -> Self;
}

/// Similar to [std::convert::Into], just that it requires a gil token.
pub trait IntoPy<T>: Sized {
    /// Performs the conversion.
    fn into_py(self, py: Python) -> T;
}

// From implies Into
impl<T, U> IntoPy<U> for T
where
    U: FromPy<T>,
{
    fn into_py(self, py: Python) -> U {
        U::from_py(self, py)
    }
}

// From (and thus Into) is reflexive
impl<T> FromPy<T> for T {
    fn from_py(t: T, _: Python) -> T {
        t
    }
}

/// Conversion trait that allows various objects to be converted into `PyObject`
/// by consuming original object.
pub trait IntoPyObject {
    /// Converts self into a Python object. (Consumes self)
    fn into_object(self, py: Python) -> PyObject;
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
    T: PyTryFrom<'a>,
{
    #[inline]
    default fn extract(ob: &'a PyObjectRef) -> PyResult<&'a T> {
        Ok(T::try_from(ob)?)
    }
}

/// Extract mutable reference to instance from `PyObject`
impl<'a, T> FromPyObject<'a> for &'a mut T
where
    T: PyTryFrom<'a>,
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
    /// Cast from PyObject to a concrete Python object type.
    fn try_into(&self) -> Result<&T, PyDowncastError>;

    /// Cast from PyObject to a concrete Python object type. With exact type check.
    fn try_into_exact(&self) -> Result<&T, PyDowncastError>;

    /// Cast from PyObject to a concrete Python object type.
    fn try_into_mut(&self) -> Result<&mut T, PyDowncastError>;

    /// Cast from PyObject to a concrete Python object type. With exact type check.
    fn try_into_mut_exact(&self) -> Result<&mut T, PyDowncastError>;
}

/// Trait implemented by Python object types that allow a checked downcast.
/// This trait is similar to `std::convert::TryFrom`
pub trait PyTryFrom<'v>: Sized {
    /// Cast from a concrete Python object type to PyObject.
    fn try_from<V: Into<&'v PyObjectRef>>(value: V) -> Result<&'v Self, PyDowncastError>;

    /// Cast from a concrete Python object type to PyObject. With exact type check.
    fn try_from_exact<V: Into<&'v PyObjectRef>>(value: V) -> Result<&'v Self, PyDowncastError>;

    /// Cast from a concrete Python object type to PyObject.
    fn try_from_mut<V: Into<&'v PyObjectRef>>(value: V) -> Result<&'v mut Self, PyDowncastError>;

    /// Cast from a concrete Python object type to PyObject. With exact type check.
    fn try_from_mut_exact<V: Into<&'v PyObjectRef>>(
        value: V,
    ) -> Result<&'v mut Self, PyDowncastError>;

    /// Cast a PyObjectRef to a specific type of PyObject. The caller must
    /// have already verified the reference is for this type.
    unsafe fn try_from_unchecked<V: Into<&'v PyObjectRef>>(value: V) -> &'v Self;

    /// Cast a PyObjectRef to a specific type of PyObject. The caller must
    /// have already verified the reference is for this type.
    #[allow(clippy::mut_from_ref)]
    unsafe fn try_from_mut_unchecked<V: Into<&'v PyObjectRef>>(value: V) -> &'v mut Self;
}

// TryFrom implies TryInto
impl<U> PyTryInto<U> for PyObjectRef
where
    U: for<'v> PyTryFrom<'v>,
{
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

impl<'v, T> PyTryFrom<'v> for T
where
    T: PyTypeInfo,
{
    fn try_from<V: Into<&'v PyObjectRef>>(value: V) -> Result<&'v T, PyDowncastError> {
        let value = value.into();
        unsafe {
            if T::is_instance(value) {
                Ok(PyTryFrom::try_from_unchecked(value))
            } else {
                Err(PyDowncastError)
            }
        }
    }

    fn try_from_exact<V: Into<&'v PyObjectRef>>(value: V) -> Result<&'v T, PyDowncastError> {
        let value = value.into();
        unsafe {
            if T::is_exact_instance(value) {
                Ok(PyTryFrom::try_from_unchecked(value))
            } else {
                Err(PyDowncastError)
            }
        }
    }

    fn try_from_mut<V: Into<&'v PyObjectRef>>(value: V) -> Result<&'v mut T, PyDowncastError> {
        let value = value.into();
        unsafe {
            if T::is_instance(value) {
                Ok(PyTryFrom::try_from_mut_unchecked(value))
            } else {
                Err(PyDowncastError)
            }
        }
    }

    fn try_from_mut_exact<V: Into<&'v PyObjectRef>>(
        value: V,
    ) -> Result<&'v mut T, PyDowncastError> {
        let value = value.into();
        unsafe {
            if T::is_exact_instance(value) {
                Ok(PyTryFrom::try_from_mut_unchecked(value))
            } else {
                Err(PyDowncastError)
            }
        }
    }

    #[inline]
    unsafe fn try_from_unchecked<V: Into<&'v PyObjectRef>>(value: V) -> &'v T {
        let value = value.into();
        let ptr = if T::OFFSET == 0 {
            value as *const _ as *const u8 as *const T
        } else {
            (value.as_ptr() as *const u8).offset(T::OFFSET) as *const T
        };
        &*ptr
    }

    #[inline]
    unsafe fn try_from_mut_unchecked<V: Into<&'v PyObjectRef>>(value: V) -> &'v mut T {
        let value = value.into();
        let ptr = if T::OFFSET == 0 {
            value as *const _ as *mut u8 as *mut T
        } else {
            (value.as_ptr() as *mut u8).offset(T::OFFSET) as *mut T
        };
        &mut *ptr
    }
}

/// Converts `()` to an empty Python tuple.
impl FromPy<()> for Py<PyTuple> {
    fn from_py(_: (), py: Python) -> Py<PyTuple> {
        PyTuple::empty(py)
    }
}

#[cfg(test)]
mod test {
    use crate::types::PyList;
    use crate::Python;

    use super::PyTryFrom;

    #[test]
    fn test_try_from_unchecked() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let list = PyList::new(py, &[1, 2, 3]);
        let val = unsafe { <PyList as PyTryFrom>::try_from_unchecked(list.as_ref()) };
        assert_eq!(list, val);
    }
}
