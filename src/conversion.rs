// Copyright (c) 2017-present PyO3 Project and Contributors

//! Defines conversions between Rust and Python types.
use crate::err::{self, PyDowncastError, PyResult};
use crate::type_object::PyTypeInfo;
use crate::types::PyTuple;
use crate::{
    ffi, gil, Py, PyAny, PyCell, PyClass, PyNativeType, PyObject, PyRef, PyRefMut, Python,
};
use std::ptr::NonNull;

/// This trait represents that **we can do zero-cost conversion from the object
/// to a FFI pointer**.
///
/// This trait is implemented for types that internally wrap a pointer to a Python object.
///
/// # Examples
///
/// ```
/// use pyo3::{prelude::*, AsPyPointer};
/// Python::with_gil(|py| {
///     let dict = pyo3::types::PyDict::new(py);
///     // All native object wrappers implement AsPyPointer!!!
///     assert_ne!(dict.as_ptr(), std::ptr::null_mut());
/// });
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
        self.as_ref()
            .map_or_else(std::ptr::null_mut, |t| t.into_ptr())
    }
}

/// Convert `None` into a null pointer.
impl<T> IntoPyPointer for Option<T>
where
    T: IntoPyPointer,
{
    #[inline]
    fn into_ptr(self) -> *mut ffi::PyObject {
        self.map_or_else(std::ptr::null_mut, |t| t.into_ptr())
    }
}

impl<'a, T> IntoPyPointer for &'a T
where
    T: AsPyPointer,
{
    fn into_ptr(self) -> *mut ffi::PyObject {
        unsafe { ffi::_Py_XNewRef(self.as_ptr()) }
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

/// Defines a conversion from a Rust type to a Python object.
///
/// It functions similarly to std's [`Into`](std::convert::Into) trait,
/// but requires a [GIL token](Python) as an argument.
/// Many functions and traits internal to PyO3 require this trait as a bound,
/// so a lack of this trait can manifest itself in different error messages.
///
/// # Examples
/// ## With `#[pyclass]`
/// The easiest way to implement `IntoPy` is by exposing a struct as a native Python object
/// by annotating it with [`#[pyclass]`](crate::prelude::pyclass).
///
/// ```rust
/// use pyo3::prelude::*;
///
/// #[pyclass]
/// struct Number {
///     #[pyo3(get, set)]
///     value: i32,
/// }
/// ```
/// Python code will see this as an instance of the `Number` class with a `value` attribute.
///
/// ## Conversion to a Python object
///
/// However, it may not be desirable to expose the existence of `Number` to Python code.
/// `IntoPy` allows us to define a conversion to an appropriate Python object.
/// ```rust
/// use pyo3::prelude::*;
///
/// struct Number {
///     value: i32,
/// }
///
/// impl IntoPy<PyObject> for Number {
///     fn into_py(self, py: Python) -> PyObject {
///         // delegates to i32's IntoPy implementation.
///         self.value.into_py(py)
///     }
/// }
/// ```
/// Python code will see this as an `int` object.
///
/// ## Dynamic conversion into Python objects.
/// It is also possible to return a different Python object depending on some condition.
/// This is useful for types like enums that can carry different types.
///
/// ```rust
/// use pyo3::prelude::*;
///
/// enum Value {
///     Integer(i32),
///     String(String),
///     None,
/// }
///
/// impl IntoPy<PyObject> for Value {
///     fn into_py(self, py: Python) -> PyObject {
///         match self {
///             Self::Integer(val) => val.into_py(py),
///             Self::String(val) => val.into_py(py),
///             Self::None => py.None(),
///         }
///     }
/// }
/// # fn main() {
/// #     Python::with_gil(|py| {
/// #         let v = Value::Integer(73).into_py(py);
/// #         let v = v.extract::<i32>(py).unwrap();
/// #
/// #         let v = Value::String("foo".into()).into_py(py);
/// #         let v = v.extract::<String>(py).unwrap();
/// #
/// #         let v = Value::None.into_py(py);
/// #         let v = v.extract::<Option<Vec<i32>>>(py).unwrap();
/// #     });
/// # }
/// ```
/// Python code will see this as any of the `int`, `string` or `None` objects.
#[cfg_attr(docsrs, doc(alias = "IntoPyCallbackOutput"))]
pub trait IntoPy<T>: Sized {
    /// Performs the conversion.
    fn into_py(self, py: Python) -> T;
}

/// `FromPyObject` is implemented by various types that can be extracted from
/// a Python object reference.
///
/// Normal usage is through the helper methods `Py::extract` or `PyAny::extract`:
///
/// ```rust,ignore
/// let obj: Py<PyAny> = ...;
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
/// The trait's conversion method takes a `&PyAny` argument but is called
/// `FromPyObject` for historical reasons.
pub trait FromPyObject<'source>: Sized {
    /// Extracts `Self` from the source `PyObject`.
    fn extract(ob: &'source PyAny) -> PyResult<Self>;
}

/// Identity conversion: allows using existing `PyObject` instances where
/// `T: ToPyObject` is expected.
impl<T: ?Sized + ToPyObject> ToPyObject for &'_ T {
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
        self.as_ref()
            .map_or_else(|| py.None(), |val| val.to_object(py))
    }
}

impl<T> IntoPy<PyObject> for Option<T>
where
    T: IntoPy<PyObject>,
{
    fn into_py(self, py: Python) -> PyObject {
        self.map_or_else(|| py.None(), |val| val.into_py(py))
    }
}

/// `()` is converted to Python `None`.
impl ToPyObject for () {
    fn to_object(&self, py: Python) -> PyObject {
        py.None()
    }
}

impl IntoPy<PyObject> for () {
    fn into_py(self, py: Python) -> PyObject {
        py.None()
    }
}

impl<T> IntoPy<PyObject> for &'_ T
where
    T: AsPyPointer,
{
    #[inline]
    fn into_py(self, py: Python) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, self.as_ptr()) }
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
            T::extract(obj).map(Some)
        }
    }
}

/// Trait implemented by Python object types that allow a checked downcast.
/// If `T` implements `PyTryFrom`, we can convert `&PyAny` to `&T`.
///
/// This trait is similar to `std::convert::TryFrom`
pub trait PyTryFrom<'v>: Sized + PyNativeType {
    /// Cast from a concrete Python object type to PyObject.
    fn try_from<V: Into<&'v PyAny>>(value: V) -> Result<&'v Self, PyDowncastError<'v>>;

    /// Cast from a concrete Python object type to PyObject. With exact type check.
    fn try_from_exact<V: Into<&'v PyAny>>(value: V) -> Result<&'v Self, PyDowncastError<'v>>;

    /// Cast a PyAny to a specific type of PyObject. The caller must
    /// have already verified the reference is for this type.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the type is valid or risk type confusion.
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
    T: PyTypeInfo + PyNativeType,
{
    fn try_from<V: Into<&'v PyAny>>(value: V) -> Result<&'v Self, PyDowncastError<'v>> {
        let value = value.into();
        unsafe {
            if T::is_type_of(value) {
                Ok(Self::try_from_unchecked(value))
            } else {
                Err(PyDowncastError::new(value, T::NAME))
            }
        }
    }

    fn try_from_exact<V: Into<&'v PyAny>>(value: V) -> Result<&'v Self, PyDowncastError<'v>> {
        let value = value.into();
        unsafe {
            if T::is_exact_type_of(value) {
                Ok(Self::try_from_unchecked(value))
            } else {
                Err(PyDowncastError::new(value, T::NAME))
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
    fn try_from<V: Into<&'v PyAny>>(value: V) -> Result<&'v Self, PyDowncastError<'v>> {
        let value = value.into();
        unsafe {
            if T::is_type_of(value) {
                Ok(Self::try_from_unchecked(value))
            } else {
                Err(PyDowncastError::new(value, T::NAME))
            }
        }
    }
    fn try_from_exact<V: Into<&'v PyAny>>(value: V) -> Result<&'v Self, PyDowncastError<'v>> {
        let value = value.into();
        unsafe {
            if T::is_exact_type_of(value) {
                Ok(Self::try_from_unchecked(value))
            } else {
                Err(PyDowncastError::new(value, T::NAME))
            }
        }
    }
    #[inline]
    unsafe fn try_from_unchecked<V: Into<&'v PyAny>>(value: V) -> &'v Self {
        Self::unchecked_downcast(value.into())
    }
}

/// Converts `()` to an empty Python tuple.
impl IntoPy<Py<PyTuple>> for () {
    fn into_py(self, py: Python) -> Py<PyTuple> {
        PyTuple::empty(py).into()
    }
}

/// Raw level conversion between `*mut ffi::PyObject` and PyO3 types.
///
/// # Safety
///
/// See safety notes on individual functions.
pub unsafe trait FromPyPointer<'p>: Sized {
    /// Convert from an arbitrary `PyObject`.
    ///
    /// # Safety
    ///
    /// Implementations must ensure the object does not get freed during `'p`
    /// and ensure that `ptr` is of the correct type.
    /// Note that it must be safe to decrement the reference count of `ptr`.
    unsafe fn from_owned_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject) -> Option<&'p Self>;
    /// Convert from an arbitrary `PyObject` or panic.
    ///
    /// # Safety
    ///
    /// Relies on [`from_owned_ptr_or_opt`](#method.from_owned_ptr_or_opt).
    unsafe fn from_owned_ptr_or_panic(py: Python<'p>, ptr: *mut ffi::PyObject) -> &'p Self {
        Self::from_owned_ptr_or_opt(py, ptr).unwrap_or_else(|| err::panic_after_error(py))
    }
    /// Convert from an arbitrary `PyObject` or panic.
    ///
    /// # Safety
    ///
    /// Relies on [`from_owned_ptr_or_opt`](#method.from_owned_ptr_or_opt).
    unsafe fn from_owned_ptr(py: Python<'p>, ptr: *mut ffi::PyObject) -> &'p Self {
        Self::from_owned_ptr_or_panic(py, ptr)
    }
    /// Convert from an arbitrary `PyObject`.
    ///
    /// # Safety
    ///
    /// Relies on [`from_owned_ptr_or_opt`](#method.from_owned_ptr_or_opt).
    unsafe fn from_owned_ptr_or_err(py: Python<'p>, ptr: *mut ffi::PyObject) -> PyResult<&'p Self> {
        Self::from_owned_ptr_or_opt(py, ptr).ok_or_else(|| err::PyErr::fetch(py))
    }
    /// Convert from an arbitrary borrowed `PyObject`.
    ///
    /// # Safety
    ///
    /// Implementations must ensure the object does not get freed during `'p` and avoid type confusion.
    unsafe fn from_borrowed_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject)
        -> Option<&'p Self>;
    /// Convert from an arbitrary borrowed `PyObject`.
    ///
    /// # Safety
    ///
    /// Relies on unsafe fn [`from_borrowed_ptr_or_opt`](#method.from_borrowed_ptr_or_opt).
    unsafe fn from_borrowed_ptr_or_panic(py: Python<'p>, ptr: *mut ffi::PyObject) -> &'p Self {
        Self::from_borrowed_ptr_or_opt(py, ptr).unwrap_or_else(|| err::panic_after_error(py))
    }
    /// Convert from an arbitrary borrowed `PyObject`.
    ///
    /// # Safety
    ///
    /// Relies on unsafe fn [`from_borrowed_ptr_or_opt`](#method.from_borrowed_ptr_or_opt).
    unsafe fn from_borrowed_ptr(py: Python<'p>, ptr: *mut ffi::PyObject) -> &'p Self {
        Self::from_borrowed_ptr_or_panic(py, ptr)
    }
    /// Convert from an arbitrary borrowed `PyObject`.
    ///
    /// # Safety
    ///
    /// Relies on unsafe fn [`from_borrowed_ptr_or_opt`](#method.from_borrowed_ptr_or_opt).
    unsafe fn from_borrowed_ptr_or_err(
        py: Python<'p>,
        ptr: *mut ffi::PyObject,
    ) -> PyResult<&'p Self> {
        Self::from_borrowed_ptr_or_opt(py, ptr).ok_or_else(|| err::PyErr::fetch(py))
    }
}

unsafe impl<'p, T> FromPyPointer<'p> for T
where
    T: 'p + crate::PyNativeType,
{
    unsafe fn from_owned_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject) -> Option<&'p Self> {
        gil::register_owned(py, NonNull::new(ptr)?);
        Some(&*(ptr as *mut Self))
    }
    unsafe fn from_borrowed_ptr_or_opt(
        _py: Python<'p>,
        ptr: *mut ffi::PyObject,
    ) -> Option<&'p Self> {
        NonNull::new(ptr as *mut Self).map(|p| &*p.as_ptr())
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{IntoPyDict, PyAny, PyDict, PyList};
    use crate::{Python, ToPyObject};

    use super::PyTryFrom;

    #[test]
    fn test_try_from() {
        Python::with_gil(|py| {
            let list: &PyAny = vec![3, 6, 5, 4, 7].to_object(py).into_ref(py);
            let dict: &PyAny = vec![("reverse", true)].into_py_dict(py).as_ref();

            assert!(PyList::try_from(list).is_ok());
            assert!(PyDict::try_from(dict).is_ok());

            assert!(PyAny::try_from(list).is_ok());
            assert!(PyAny::try_from(dict).is_ok());
        });
    }

    #[test]
    fn test_try_from_exact() {
        Python::with_gil(|py| {
            let list: &PyAny = vec![3, 6, 5, 4, 7].to_object(py).into_ref(py);
            let dict: &PyAny = vec![("reverse", true)].into_py_dict(py).as_ref();

            assert!(PyList::try_from_exact(list).is_ok());
            assert!(PyDict::try_from_exact(dict).is_ok());

            assert!(PyAny::try_from_exact(list).is_err());
            assert!(PyAny::try_from_exact(dict).is_err());
        });
    }

    #[test]
    fn test_try_from_unchecked() {
        Python::with_gil(|py| {
            let list = PyList::new(py, &[1, 2, 3]);
            let val = unsafe { <PyList as PyTryFrom>::try_from_unchecked(list.as_ref()) };
            assert_eq!(list, val);
        });
    }
}
