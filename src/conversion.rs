//! Defines conversions between Rust and Python types.
use crate::err::PyResult;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::pyclass::boolean_struct::False;
use crate::types::any::PyAnyMethods;
use crate::types::PyTuple;
use crate::{ffi, Borrowed, Bound, Py, PyAny, PyClass, PyObject, PyRef, PyRefMut, Python};
#[cfg(feature = "gil-refs")]
use {
    crate::{err, gil, PyNativeType},
    std::ptr::NonNull,
};

/// Returns a borrowed pointer to a Python object.
///
/// The returned pointer will be valid for as long as `self` is. It may be null depending on the
/// implementation.
///
/// # Examples
///
/// ```rust
/// use pyo3::prelude::*;
/// use pyo3::types::PyString;
/// use pyo3::ffi;
///
/// Python::with_gil(|py| {
///     let s: Py<PyString> = "foo".into_py(py);
///     let ptr = s.as_ptr();
///
///     let is_really_a_pystring = unsafe { ffi::PyUnicode_CheckExact(ptr) };
///     assert_eq!(is_really_a_pystring, 1);
/// });
/// ```
///
/// # Safety
///
/// For callers, it is your responsibility to make sure that the underlying Python object is not dropped too
/// early. For example, the following code will cause undefined behavior:
///
/// ```rust,no_run
/// # use pyo3::prelude::*;
/// # use pyo3::ffi;
/// #
/// Python::with_gil(|py| {
///     let ptr: *mut ffi::PyObject = 0xabad1dea_u32.into_py(py).as_ptr();
///
///     let isnt_a_pystring = unsafe {
///         // `ptr` is dangling, this is UB
///         ffi::PyUnicode_CheckExact(ptr)
///     };
/// #    assert_eq!(isnt_a_pystring, 0);
/// });
/// ```
///
/// This happens because the pointer returned by `as_ptr` does not carry any lifetime information
/// and the Python object is dropped immediately after the `0xabad1dea_u32.into_py(py).as_ptr()`
/// expression is evaluated. To fix the problem, bind Python object to a local variable like earlier
/// to keep the Python object alive until the end of its scope.
///
/// Implementors must ensure this returns a valid pointer to a Python object, which borrows a reference count from `&self`.
pub unsafe trait AsPyPointer {
    /// Returns the underlying FFI pointer as a borrowed pointer.
    fn as_ptr(&self) -> *mut ffi::PyObject;
}

/// Conversion trait that allows various objects to be converted into `PyObject`.
pub trait ToPyObject {
    /// Converts self into a Python object.
    fn to_object(&self, py: Python<'_>) -> PyObject;
}

/// Defines a conversion from a Rust type to a Python object.
///
/// It functions similarly to std's [`Into`] trait, but requires a [GIL token](Python)
/// as an argument. Many functions and traits internal to PyO3 require this trait as a bound,
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
/// # #[allow(dead_code)]
/// struct Number {
///     value: i32,
/// }
///
/// impl IntoPy<PyObject> for Number {
///     fn into_py(self, py: Python<'_>) -> PyObject {
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
///     fn into_py(self, py: Python<'_>) -> PyObject {
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
#[cfg_attr(
    diagnostic_namespace,
    diagnostic::on_unimplemented(
        message = "`{Self}` cannot be converted to a Python object",
        note = "`IntoPy` is automatically implemented by the `#[pyclass]` macro",
        note = "if you do not wish to have a corresponding Python type, implement it manually",
        note = "if you do not own `{Self}` you can perform a manual conversion to one of the types in `pyo3::types::*`"
    )
)]
pub trait IntoPy<T>: Sized {
    /// Performs the conversion.
    fn into_py(self, py: Python<'_>) -> T;

    /// Extracts the type hint information for this type when it appears as a return value.
    ///
    /// For example, `Vec<u32>` would return `List[int]`.
    /// The default implementation returns `Any`, which is correct for any type.
    ///
    /// For most types, the return value for this method will be identical to that of [`FromPyObject::type_input`].
    /// It may be different for some types, such as `Dict`, to allow duck-typing: functions return `Dict` but take `Mapping` as argument.
    #[cfg(feature = "experimental-inspect")]
    fn type_output() -> TypeInfo {
        TypeInfo::Any
    }
}

/// Extract a type from a Python object.
///
///
/// Normal usage is through the `extract` methods on [`Bound`] and [`Py`], which forward to this trait.
///
/// # Examples
///
/// ```rust
/// use pyo3::prelude::*;
/// use pyo3::types::PyString;
///
/// # fn main() -> PyResult<()> {
/// Python::with_gil(|py| {
///     // Calling `.extract()` on a `Bound` smart pointer
///     let obj: Bound<'_, PyString> = PyString::new_bound(py, "blah");
///     let s: String = obj.extract()?;
/// #   assert_eq!(s, "blah");
///
///     // Calling `.extract(py)` on a `Py` smart pointer
///     let obj: Py<PyString> = obj.unbind();
///     let s: String = obj.extract(py)?;
/// #   assert_eq!(s, "blah");
/// #   Ok(())
/// })
/// # }
/// ```
///
// /// FIXME: until `FromPyObject` can pick up a second lifetime, the below commentary is no longer
// /// true. Update and restore this documentation at that time.
// ///
// /// Note: depending on the implementation, the lifetime of the extracted result may
// /// depend on the lifetime of the `obj` or the `prepared` variable.
// ///
// /// For example, when extracting `&str` from a Python byte string, the resulting string slice will
// /// point to the existing string data (lifetime: `'py`).
// /// On the other hand, when extracting `&str` from a Python Unicode string, the preparation step
// /// will convert the string to UTF-8, and the resulting string slice will have lifetime `'prepared`.
// /// Since which case applies depends on the runtime type of the Python object,
// /// both the `obj` and `prepared` variables must outlive the resulting string slice.
///
/// During the migration of PyO3 from the "GIL Refs" API to the `Bound<T>` smart pointer, this trait
/// has two methods `extract` and `extract_bound` which are defaulted to call each other. To avoid
/// infinite recursion, implementors must implement at least one of these methods. The recommendation
/// is to implement `extract_bound` and leave `extract` as the default implementation.
pub trait FromPyObject<'py>: Sized {
    /// Extracts `Self` from the source GIL Ref `obj`.
    ///
    /// Implementors are encouraged to implement `extract_bound` and leave this method as the
    /// default implementation, which will forward calls to `extract_bound`.
    #[cfg(feature = "gil-refs")]
    fn extract(ob: &'py PyAny) -> PyResult<Self> {
        Self::extract_bound(&ob.as_borrowed())
    }

    /// Extracts `Self` from the bound smart pointer `obj`.
    ///
    /// Implementors are encouraged to implement this method and leave `extract` defaulted, as
    /// this will be most compatible with PyO3's future API.
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self>;

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

mod from_py_object_bound_sealed {
    /// Private seal for the `FromPyObjectBound` trait.
    ///
    /// This prevents downstream types from implementing the trait before
    /// PyO3 is ready to declare the trait as public API.
    pub trait Sealed {}

    // This generic implementation is why the seal is separate from
    // `crate::sealed::Sealed`.
    impl<'py, T> Sealed for T where T: super::FromPyObject<'py> {}
    impl Sealed for &'_ str {}
    impl Sealed for std::borrow::Cow<'_, str> {}
    impl Sealed for &'_ [u8] {}
    impl Sealed for std::borrow::Cow<'_, [u8]> {}
}

/// Expected form of [`FromPyObject`] to be used in a future PyO3 release.
///
/// The difference between this and `FromPyObject` is that this trait takes an
/// additional lifetime `'a`, which is the lifetime of the input `Bound`.
///
/// This allows implementations for `&'a str` and `&'a [u8]`, which could not
/// be expressed by the existing `FromPyObject` trait once the GIL Refs API was
/// removed.
///
/// # Usage
///
/// Users are prevented from implementing this trait, instead they should implement
/// the normal `FromPyObject` trait. This trait has a blanket implementation
/// for `T: FromPyObject`.
///
/// The only case where this trait may have a use case to be implemented is when the
/// lifetime of the extracted value is tied to the lifetime `'a` of the input `Bound`
/// instead of the GIL lifetime `py`, as is the case for the `&'a str` implementation.
///
/// Please contact the PyO3 maintainers if you believe you have a use case for implementing
/// this trait before PyO3 is ready to change the main `FromPyObject` trait to take an
/// additional lifetime.
///
/// Similarly, users should typically not call these trait methods and should instead
/// use this via the `extract` method on `Bound` and `Py`.
pub trait FromPyObjectBound<'a, 'py>: Sized + from_py_object_bound_sealed::Sealed {
    /// Extracts `Self` from the bound smart pointer `obj`.
    ///
    /// Users are advised against calling this method directly: instead, use this via
    /// [`Bound<'_, PyAny>::extract`] or [`Py::extract`].
    fn from_py_object_bound(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self>;

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

impl<'py, T> FromPyObjectBound<'_, 'py> for T
where
    T: FromPyObject<'py>,
{
    fn from_py_object_bound(ob: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        Self::extract_bound(&ob)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        <T as FromPyObject>::type_input()
    }
}

/// Identity conversion: allows using existing `PyObject` instances where
/// `T: ToPyObject` is expected.
impl<T: ?Sized + ToPyObject> ToPyObject for &'_ T {
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        <T as ToPyObject>::to_object(*self, py)
    }
}

impl IntoPy<PyObject> for &'_ PyAny {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, self.as_ptr()) }
    }
}

impl<T> IntoPy<PyObject> for &'_ T
where
    T: AsRef<PyAny>,
{
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        unsafe { PyObject::from_borrowed_ptr(py, self.as_ref().as_ptr()) }
    }
}

#[allow(deprecated)]
#[cfg(feature = "gil-refs")]
impl<'py, T> FromPyObject<'py> for &'py crate::PyCell<T>
where
    T: PyClass,
{
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        obj.clone().into_gil_ref().downcast().map_err(Into::into)
    }
}

impl<T> FromPyObject<'_> for T
where
    T: PyClass + Clone,
{
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        let bound = obj.downcast::<Self>()?;
        Ok(bound.try_borrow()?.clone())
    }
}

impl<'py, T> FromPyObject<'py> for PyRef<'py, T>
where
    T: PyClass,
{
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        obj.downcast::<T>()?.try_borrow().map_err(Into::into)
    }
}

impl<'py, T> FromPyObject<'py> for PyRefMut<'py, T>
where
    T: PyClass<Frozen = False>,
{
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        obj.downcast::<T>()?.try_borrow_mut().map_err(Into::into)
    }
}

/// Converts `()` to an empty Python tuple.
impl IntoPy<Py<PyTuple>> for () {
    fn into_py(self, py: Python<'_>) -> Py<PyTuple> {
        PyTuple::empty_bound(py).unbind()
    }
}

/// Raw level conversion between `*mut ffi::PyObject` and PyO3 types.
///
/// # Safety
///
/// See safety notes on individual functions.
#[cfg(feature = "gil-refs")]
#[deprecated(since = "0.21.0")]
pub unsafe trait FromPyPointer<'p>: Sized {
    /// Convert from an arbitrary `PyObject`.
    ///
    /// # Safety
    ///
    /// Implementations must ensure the object does not get freed during `'p`
    /// and ensure that `ptr` is of the correct type.
    /// Note that it must be safe to decrement the reference count of `ptr`.
    #[deprecated(
        since = "0.21.0",
        note = "use `Py::from_owned_ptr_or_opt(py, ptr)` or `Bound::from_owned_ptr_or_opt(py, ptr)` instead"
    )]
    unsafe fn from_owned_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject) -> Option<&'p Self>;
    /// Convert from an arbitrary `PyObject` or panic.
    ///
    /// # Safety
    ///
    /// Relies on [`from_owned_ptr_or_opt`](#method.from_owned_ptr_or_opt).
    #[deprecated(
        since = "0.21.0",
        note = "use `Py::from_owned_ptr(py, ptr)` or `Bound::from_owned_ptr(py, ptr)` instead"
    )]
    unsafe fn from_owned_ptr_or_panic(py: Python<'p>, ptr: *mut ffi::PyObject) -> &'p Self {
        #[allow(deprecated)]
        Self::from_owned_ptr_or_opt(py, ptr).unwrap_or_else(|| err::panic_after_error(py))
    }
    /// Convert from an arbitrary `PyObject` or panic.
    ///
    /// # Safety
    ///
    /// Relies on [`from_owned_ptr_or_opt`](#method.from_owned_ptr_or_opt).
    #[deprecated(
        since = "0.21.0",
        note = "use `Py::from_owned_ptr(py, ptr)` or `Bound::from_owned_ptr(py, ptr)` instead"
    )]
    unsafe fn from_owned_ptr(py: Python<'p>, ptr: *mut ffi::PyObject) -> &'p Self {
        #[allow(deprecated)]
        Self::from_owned_ptr_or_panic(py, ptr)
    }
    /// Convert from an arbitrary `PyObject`.
    ///
    /// # Safety
    ///
    /// Relies on [`from_owned_ptr_or_opt`](#method.from_owned_ptr_or_opt).
    #[deprecated(
        since = "0.21.0",
        note = "use `Py::from_owned_ptr_or_err(py, ptr)` or `Bound::from_owned_ptr_or_err(py, ptr)` instead"
    )]
    unsafe fn from_owned_ptr_or_err(py: Python<'p>, ptr: *mut ffi::PyObject) -> PyResult<&'p Self> {
        #[allow(deprecated)]
        Self::from_owned_ptr_or_opt(py, ptr).ok_or_else(|| err::PyErr::fetch(py))
    }
    /// Convert from an arbitrary borrowed `PyObject`.
    ///
    /// # Safety
    ///
    /// Implementations must ensure the object does not get freed during `'p` and avoid type confusion.
    #[deprecated(
        since = "0.21.0",
        note = "use `Py::from_borrowed_ptr_or_opt(py, ptr)` or `Bound::from_borrowed_ptr_or_opt(py, ptr)` instead"
    )]
    unsafe fn from_borrowed_ptr_or_opt(py: Python<'p>, ptr: *mut ffi::PyObject)
        -> Option<&'p Self>;
    /// Convert from an arbitrary borrowed `PyObject`.
    ///
    /// # Safety
    ///
    /// Relies on unsafe fn [`from_borrowed_ptr_or_opt`](#method.from_borrowed_ptr_or_opt).
    #[deprecated(
        since = "0.21.0",
        note = "use `Py::from_borrowed_ptr(py, ptr)` or `Bound::from_borrowed_ptr(py, ptr)` instead"
    )]
    unsafe fn from_borrowed_ptr_or_panic(py: Python<'p>, ptr: *mut ffi::PyObject) -> &'p Self {
        #[allow(deprecated)]
        Self::from_borrowed_ptr_or_opt(py, ptr).unwrap_or_else(|| err::panic_after_error(py))
    }
    /// Convert from an arbitrary borrowed `PyObject`.
    ///
    /// # Safety
    ///
    /// Relies on unsafe fn [`from_borrowed_ptr_or_opt`](#method.from_borrowed_ptr_or_opt).
    #[deprecated(
        since = "0.21.0",
        note = "use `Py::from_borrowed_ptr(py, ptr)` or `Bound::from_borrowed_ptr(py, ptr)` instead"
    )]
    unsafe fn from_borrowed_ptr(py: Python<'p>, ptr: *mut ffi::PyObject) -> &'p Self {
        #[allow(deprecated)]
        Self::from_borrowed_ptr_or_panic(py, ptr)
    }
    /// Convert from an arbitrary borrowed `PyObject`.
    ///
    /// # Safety
    ///
    /// Relies on unsafe fn [`from_borrowed_ptr_or_opt`](#method.from_borrowed_ptr_or_opt).
    #[deprecated(
        since = "0.21.0",
        note = "use `Py::from_borrowed_ptr_or_err(py, ptr)` or `Bound::from_borrowed_ptr_or_err(py, ptr)` instead"
    )]
    unsafe fn from_borrowed_ptr_or_err(
        py: Python<'p>,
        ptr: *mut ffi::PyObject,
    ) -> PyResult<&'p Self> {
        #[allow(deprecated)]
        Self::from_borrowed_ptr_or_opt(py, ptr).ok_or_else(|| err::PyErr::fetch(py))
    }
}

#[cfg(feature = "gil-refs")]
#[allow(deprecated)]
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

/// ```rust,compile_fail
/// use pyo3::prelude::*;
///
/// #[pyclass]
/// struct TestClass {
///     num: u32,
/// }
///
/// let t = TestClass { num: 10 };
///
/// Python::with_gil(|py| {
///     let pyvalue = Py::new(py, t).unwrap().to_object(py);
///     let t: TestClass = pyvalue.extract(py).unwrap();
/// })
/// ```
mod test_no_clone {}
