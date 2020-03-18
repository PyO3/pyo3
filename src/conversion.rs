// Copyright (c) 2017-present PyO3 Project and Contributors

//! Conversions between various states of Rust and Python types and their wrappers.
use crate::err::{self, PyDowncastError, PyResult};
use crate::object::PyObject;
use crate::type_object::{PyDowncastImpl, PyTypeInfo};
use crate::types::PyTuple;
use crate::{ffi, gil, Py, PyAny, PyCell, PyClass, PyNativeType, PyRef, PyRefMut, Python};
use std::ptr::NonNull;

/// This trait represents that **we can do zero-cost conversion from the object
/// to a FFI pointer**.
///
/// This trait is implemented for types that internally wrap a pointer to a Python object.
///
/// # Example
///
/// ```
/// use pyo3::{AsPyPointer, prelude::*};
/// let gil = Python::acquire_gil();
/// let dict = pyo3::types::PyDict::new(gil.python());
/// // All native object wrappers implement AsPyPointer!!!
/// assert_ne!(dict.as_ptr(), std::ptr::null_mut());
/// ```
pub trait AsPyPointer {
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
impl<T> AsPyPointer for Option<T>
where
    T: AsPyPointer,
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
    T: AsPyPointer,
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

/// Conversion trait that allows various objects to be converted into `PyObject`.
pub trait ToPyObject {
    /// Converts self into a Python object.
    fn to_object(&self, py: Python) -> PyObject;
}

/// This trait has two implementations: The slow one is implemented for
/// all [ToPyObject] and creates a new object using [ToPyObject::to_object],
/// while the fast one is only implemented for AsPyPointer (we know
/// that every AsPyPointer is also ToPyObject) and uses [AsPyPointer::as_ptr()]
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
        F: FnOnce(*mut ffi::PyObject) -> R;
}

impl<T> ToBorrowedObject for T
where
    T: ToPyObject,
{
    default fn with_borrowed_ptr<F, R>(&self, py: Python, f: F) -> R
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

impl<T> ToBorrowedObject for T
where
    T: ToPyObject + AsPyPointer,
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

/// `FromPyObject` is implemented by various types that can be extracted from
/// a Python object reference.
///
/// Normal usage is through the helper methods `PyObject::extract` or
/// `PyAny::extract`:
///
/// ```let obj: PyObject = ...;
/// let value: &TargetType = obj.extract(py)?;
///
/// let any: &PyAny = ...;
/// let value: &TargetType = any.extract()?;
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
/// In cases where the result does not depend on the `'prepared` lifetime,
/// the inherent method `PyObject::extract()` can be used.
///
/// The trait's conversion method takes a `&PyAny` argument but is called
/// `FromPyObject` for historical reasons.
pub trait FromPyObject<'source>: Sized {
    /// Extracts `Self` from the source `PyObject`.
    fn extract(ob: &'source PyAny) -> PyResult<Self>;
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

impl<T> IntoPy<PyObject> for Option<T>
where
    T: IntoPy<PyObject>,
{
    fn into_py(self, py: Python) -> PyObject {
        match self {
            Some(val) => val.into_py(py),
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

impl FromPy<()> for PyObject {
    fn from_py(_: (), py: Python) -> Self {
        py.None()
    }
}

impl<'a, T> FromPy<&'a T> for PyObject
where
    T: AsPyPointer,
{
    #[inline]
    fn from_py(other: &'a T, py: Python) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, other.as_ptr()) }
    }
}

impl<'a, T> FromPyObject<'a> for &'a PyCell<T>
where
    T: PyClass,
{
    fn extract(obj: &'a PyAny) -> PyResult<Self> {
        PyTryFrom::try_from(obj).map_err(Into::into)
    }
}

impl<'a, T> FromPyObject<'a> for T
where
    T: PyClass + Clone,
{
    fn extract(obj: &'a PyAny) -> PyResult<Self> {
        let cell: &PyCell<Self> = PyTryFrom::try_from(obj)?;
        Ok(unsafe { cell.try_borrow_unguarded()?.clone() })
    }
}

impl<'a, T> FromPyObject<'a> for PyRef<'a, T>
where
    T: PyClass,
{
    fn extract(obj: &'a PyAny) -> PyResult<Self> {
        let cell: &PyCell<T> = PyTryFrom::try_from(obj)?;
        cell.try_borrow().map_err(Into::into)
    }
}

impl<'a, T> FromPyObject<'a> for PyRefMut<'a, T>
where
    T: PyClass,
{
    fn extract(obj: &'a PyAny) -> PyResult<Self> {
        let cell: &PyCell<T> = PyTryFrom::try_from(obj)?;
        cell.try_borrow_mut().map_err(Into::into)
    }
}

impl<'a, T> FromPyObject<'a> for Option<T>
where
    T: FromPyObject<'a>,
{
    fn extract(obj: &'a PyAny) -> PyResult<Self> {
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
/// If `T` implements `PyTryFrom`, we can convert `&PyAny` to `&T`.
///
/// This trait is similar to `std::convert::TryFrom`
pub trait PyTryFrom<'v>: Sized + PyDowncastImpl {
    /// Cast from a concrete Python object type to PyObject.
    fn try_from<V: Into<&'v PyAny>>(value: V) -> Result<&'v Self, PyDowncastError>;

    /// Cast from a concrete Python object type to PyObject. With exact type check.
    fn try_from_exact<V: Into<&'v PyAny>>(value: V) -> Result<&'v Self, PyDowncastError>;

    /// Cast a PyAny to a specific type of PyObject. The caller must
    /// have already verified the reference is for this type.
    unsafe fn try_from_unchecked<V: Into<&'v PyAny>>(value: V) -> &'v Self;
}

/// Trait implemented by Python object types that allow a checked downcast.
/// This trait is similar to `std::convert::TryInto`
pub trait PyTryInto<T>: Sized {
    /// Cast from PyObject to a concrete Python object type.
    fn try_into(&self) -> Result<&T, PyDowncastError>;

    /// Cast from PyObject to a concrete Python object type. With exact type check.
    fn try_into_exact(&self) -> Result<&T, PyDowncastError>;
}

// TryFrom implies TryInto
impl<U> PyTryInto<U> for PyAny
where
    U: for<'v> PyTryFrom<'v>,
{
    fn try_into(&self) -> Result<&U, PyDowncastError> {
        U::try_from(self)
    }
    fn try_into_exact(&self) -> Result<&U, PyDowncastError> {
        U::try_from_exact(self)
    }
}

impl<'v, T> PyTryFrom<'v> for T
where
    T: PyDowncastImpl + PyTypeInfo + PyNativeType,
{
    fn try_from<V: Into<&'v PyAny>>(value: V) -> Result<&'v Self, PyDowncastError> {
        let value = value.into();
        unsafe {
            if T::is_instance(value) {
                Ok(Self::try_from_unchecked(value))
            } else {
                Err(PyDowncastError)
            }
        }
    }

    fn try_from_exact<V: Into<&'v PyAny>>(value: V) -> Result<&'v Self, PyDowncastError> {
        let value = value.into();
        unsafe {
            if T::is_exact_instance(value) {
                Ok(Self::try_from_unchecked(value))
            } else {
                Err(PyDowncastError)
            }
        }
    }

    #[inline]
    unsafe fn try_from_unchecked<V: Into<&'v PyAny>>(value: V) -> &'v Self {
        Self::unchecked_downcast(value.into())
    }
}

impl<'v, T> PyTryFrom<'v> for PyCell<T>
where
    T: 'v + PyClass,
{
    fn try_from<V: Into<&'v PyAny>>(value: V) -> Result<&'v Self, PyDowncastError> {
        let value = value.into();
        unsafe {
            if T::is_instance(value) {
                Ok(Self::try_from_unchecked(value))
            } else {
                Err(PyDowncastError)
            }
        }
    }
    fn try_from_exact<V: Into<&'v PyAny>>(value: V) -> Result<&'v Self, PyDowncastError> {
        let value = value.into();
        unsafe {
            if T::is_exact_instance(value) {
                Ok(Self::try_from_unchecked(value))
            } else {
                Err(PyDowncastError)
            }
        }
    }
    #[inline]
    unsafe fn try_from_unchecked<V: Into<&'v PyAny>>(value: V) -> &'v Self {
        Self::unchecked_downcast(value.into())
    }
}

/// Converts `()` to an empty Python tuple.
impl FromPy<()> for Py<PyTuple> {
    fn from_py(_: (), py: Python) -> Py<PyTuple> {
        Py::from_py(PyTuple::empty(py), py)
    }
}

/// Raw level conversion between `*mut ffi::PyObject` and PyO3 types.
pub unsafe trait FromPyPointer<'p>: Sized {
    unsafe fn from_owned_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject) -> Option<&'p Self>;
    unsafe fn from_owned_ptr_or_panic(py: Python<'p>, ptr: *mut ffi::PyObject) -> &'p Self {
        match Self::from_owned_ptr_or_opt(py, ptr) {
            Some(s) => s,
            None => err::panic_after_error(),
        }
    }
    unsafe fn from_owned_ptr(py: Python<'p>, ptr: *mut ffi::PyObject) -> &'p Self {
        Self::from_owned_ptr_or_panic(py, ptr)
    }
    unsafe fn from_owned_ptr_or_err(py: Python<'p>, ptr: *mut ffi::PyObject) -> PyResult<&'p Self> {
        match Self::from_owned_ptr_or_opt(py, ptr) {
            Some(s) => Ok(s),
            None => Err(err::PyErr::fetch(py)),
        }
    }
    unsafe fn from_borrowed_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject)
        -> Option<&'p Self>;
    unsafe fn from_borrowed_ptr_or_panic(py: Python<'p>, ptr: *mut ffi::PyObject) -> &'p Self {
        match Self::from_borrowed_ptr_or_opt(py, ptr) {
            Some(s) => s,
            None => err::panic_after_error(),
        }
    }
    unsafe fn from_borrowed_ptr(py: Python<'p>, ptr: *mut ffi::PyObject) -> &'p Self {
        Self::from_borrowed_ptr_or_panic(py, ptr)
    }
    unsafe fn from_borrowed_ptr_or_err(
        py: Python<'p>,
        ptr: *mut ffi::PyObject,
    ) -> PyResult<&'p Self> {
        match Self::from_borrowed_ptr_or_opt(py, ptr) {
            Some(s) => Ok(s),
            None => Err(err::PyErr::fetch(py)),
        }
    }
}

unsafe impl<'p, T> FromPyPointer<'p> for T
where
    T: 'p + crate::PyNativeType,
{
    unsafe fn from_owned_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject) -> Option<&'p Self> {
        NonNull::new(ptr).map(|p| Self::unchecked_downcast(gil::register_owned(py, p)))
    }
    unsafe fn from_borrowed_ptr_or_opt(
        py: Python<'p>,
        ptr: *mut ffi::PyObject,
    ) -> Option<&'p Self> {
        NonNull::new(ptr).map(|p| Self::unchecked_downcast(gil::register_borrowed(py, p)))
    }
}

unsafe impl<'p, T> FromPyPointer<'p> for PyCell<T>
where
    T: PyClass,
{
    unsafe fn from_owned_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject) -> Option<&'p Self> {
        NonNull::new(ptr).map(|p| &*(gil::register_owned(py, p).as_ptr() as *const PyCell<T>))
    }
    unsafe fn from_borrowed_ptr_or_opt(
        py: Python<'p>,
        ptr: *mut ffi::PyObject,
    ) -> Option<&'p Self> {
        NonNull::new(ptr).map(|p| &*(gil::register_borrowed(py, p).as_ptr() as *const PyCell<T>))
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
