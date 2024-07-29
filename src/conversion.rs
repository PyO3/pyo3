//! Defines conversions between Rust and Python types.
use crate::err::PyResult;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::pyclass::boolean_struct::False;
use crate::types::PyTuple;
use crate::{
    ffi, Borrowed, Bound, BoundObject, Py, PyAny, PyClass, PyErr, PyRef, PyRefMut, Python,
};
use std::convert::Infallible;

/// Returns a borrowed pointer to a Python object.
///
/// The returned pointer will be valid for as long as `self` is. It may be null depending on the
/// implementation.
///
/// # Examples
///
/// ```rust
/// use pyo3::prelude::*;
/// use pyo3::ffi;
///
/// Python::with_gil(|py| {
///     let s = "foo".into_pyobject(py)?;
///     let ptr = s.as_ptr();
///
///     let is_really_a_pystring = unsafe { ffi::PyUnicode_CheckExact(ptr) };
///     assert_eq!(is_really_a_pystring, 1);
/// #   Ok::<_, PyErr>(())
/// })
/// # .unwrap();
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
///     // ERROR: calling `.as_ptr()` will throw away the temporary object and leave `ptr` dangling.
///     let ptr: *mut ffi::PyObject = 0xabad1dea_u32.into_pyobject(py)?.as_ptr();
///
///     let isnt_a_pystring = unsafe {
///         // `ptr` is dangling, this is UB
///         ffi::PyUnicode_CheckExact(ptr)
///     };
/// #   assert_eq!(isnt_a_pystring, 0);
/// #   Ok::<_, PyErr>(())
/// })
/// # .unwrap();
/// ```
///
/// This happens because the pointer returned by `as_ptr` does not carry any lifetime information
/// and the Python object is dropped immediately after the `0xabad1dea_u32.into_pyobject(py).as_ptr()`
/// expression is evaluated. To fix the problem, bind Python object to a local variable like earlier
/// to keep the Python object alive until the end of its scope.
///
/// Implementors must ensure this returns a valid pointer to a Python object, which borrows a reference count from `&self`.
pub unsafe trait AsPyPointer {
    /// Returns the underlying FFI pointer as a borrowed pointer.
    fn as_ptr(&self) -> *mut ffi::PyObject;
}

/// Defines a conversion from a Rust type to a Python object, which may fail.
///
/// This trait has `#[derive(IntoPyObject)]` to automatically implement it for simple types and
/// `#[derive(IntoPyObjectRef)]` to implement the same for references.
///
/// It functions similarly to std's [`TryInto`] trait, but requires a [GIL token](Python)
/// as an argument.
///
/// The [`into_pyobject`][IntoPyObject::into_pyobject] method is designed for maximum flexibility and efficiency; it
///  - allows for a concrete Python type to be returned (the [`Target`][IntoPyObject::Target] associated type)
///  - allows for the smart pointer containing the Python object to be either `Bound<'py, Self::Target>` or `Borrowed<'a, 'py, Self::Target>`
///    to avoid unnecessary reference counting overhead
///  - allows for a custom error type to be returned in the event of a conversion error to avoid
///    unnecessarily creating a Python exception
///
/// # See also
///
/// - The [`IntoPyObjectExt`] trait, which provides convenience methods for common usages of
///   `IntoPyObject` which erase type information and convert errors to `PyErr`.
#[cfg_attr(
    diagnostic_namespace,
    diagnostic::on_unimplemented(
        message = "`{Self}` cannot be converted to a Python object",
        note = "`IntoPyObject` is automatically implemented by the `#[pyclass]` macro",
        note = "if you do not wish to have a corresponding Python type, implement it manually",
        note = "if you do not own `{Self}` you can perform a manual conversion to one of the types in `pyo3::types::*`"
    )
)]
pub trait IntoPyObject<'py>: Sized {
    /// The Python output type
    type Target;
    /// The smart pointer type to use.
    ///
    /// This will usually be [`Bound<'py, Target>`], but in special cases [`Borrowed<'a, 'py, Target>`] can be
    /// used to minimize reference counting overhead.
    type Output: BoundObject<'py, Self::Target>;
    /// The type returned in the event of a conversion error.
    type Error: Into<PyErr>;

    /// Performs the conversion.
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error>;

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

    /// Converts sequence of Self into a Python object. Used to specialize `Vec<u8>`, `[u8; N]`
    /// and `SmallVec<[u8; N]>` as a sequence of bytes into a `bytes` object.
    #[doc(hidden)]
    fn owned_sequence_into_pyobject<I>(
        iter: I,
        py: Python<'py>,
        _: private::Token,
    ) -> Result<Bound<'py, PyAny>, PyErr>
    where
        I: IntoIterator<Item = Self> + AsRef<[Self]>,
        I::IntoIter: ExactSizeIterator<Item = Self>,
    {
        let mut iter = iter.into_iter().map(|e| e.into_bound_py_any(py));
        let list = crate::types::list::try_new_from_iter(py, &mut iter);
        list.map(Bound::into_any)
    }

    /// Converts sequence of Self into a Python object. Used to specialize `&[u8]` and `Cow<[u8]>`
    /// as a sequence of bytes into a `bytes` object.
    #[doc(hidden)]
    fn borrowed_sequence_into_pyobject<I>(
        iter: I,
        py: Python<'py>,
        _: private::Token,
    ) -> Result<Bound<'py, PyAny>, PyErr>
    where
        Self: private::Reference,
        I: IntoIterator<Item = Self> + AsRef<[<Self as private::Reference>::BaseType]>,
        I::IntoIter: ExactSizeIterator<Item = Self>,
    {
        let mut iter = iter.into_iter().map(|e| e.into_bound_py_any(py));
        let list = crate::types::list::try_new_from_iter(py, &mut iter);
        list.map(Bound::into_any)
    }
}

pub(crate) mod private {
    pub struct Token;

    pub trait Reference {
        type BaseType;
    }

    impl<T> Reference for &'_ T {
        type BaseType = T;
    }
}

impl<'py, T> IntoPyObject<'py> for Bound<'py, T> {
    type Target = T;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, _py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self)
    }
}

impl<'a, 'py, T> IntoPyObject<'py> for &'a Bound<'py, T> {
    type Target = T;
    type Output = Borrowed<'a, 'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, _py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.as_borrowed())
    }
}

impl<'a, 'py, T> IntoPyObject<'py> for Borrowed<'a, 'py, T> {
    type Target = T;
    type Output = Borrowed<'a, 'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, _py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self)
    }
}

impl<'a, 'py, T> IntoPyObject<'py> for &Borrowed<'a, 'py, T> {
    type Target = T;
    type Output = Borrowed<'a, 'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, _py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(*self)
    }
}

impl<'py, T> IntoPyObject<'py> for Py<T> {
    type Target = T;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.into_bound(py))
    }
}

impl<'a, 'py, T> IntoPyObject<'py> for &'a Py<T> {
    type Target = T;
    type Output = Borrowed<'a, 'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.bind_borrowed(py))
    }
}

impl<'a, 'py, T> IntoPyObject<'py> for &&'a T
where
    &'a T: IntoPyObject<'py>,
{
    type Target = <&'a T as IntoPyObject<'py>>::Target;
    type Output = <&'a T as IntoPyObject<'py>>::Output;
    type Error = <&'a T as IntoPyObject<'py>>::Error;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

mod into_pyobject_ext {
    pub trait Sealed {}
    impl<'py, T> Sealed for T where T: super::IntoPyObject<'py> {}
}

/// Convenience methods for common usages of [`IntoPyObject`]. Every type that implements
/// [`IntoPyObject`] also implements this trait.
///
/// These methods:
///   - Drop type information from the output, returning a `PyAny` object.
///   - Always convert the `Error` type to `PyErr`, which may incur a performance penalty but it
///     more convenient in contexts where the `?` operator would produce a `PyErr` anyway.
pub trait IntoPyObjectExt<'py>: IntoPyObject<'py> + into_pyobject_ext::Sealed {
    /// Converts `self` into an owned Python object, dropping type information.
    #[inline]
    fn into_bound_py_any(self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        match self.into_pyobject(py) {
            Ok(obj) => Ok(obj.into_any().into_bound()),
            Err(err) => Err(err.into()),
        }
    }

    /// Converts `self` into an owned Python object, dropping type information and unbinding it
    /// from the `'py` lifetime.
    #[inline]
    fn into_py_any(self, py: Python<'py>) -> PyResult<Py<PyAny>> {
        match self.into_pyobject(py) {
            Ok(obj) => Ok(obj.into_any().unbind()),
            Err(err) => Err(err.into()),
        }
    }

    /// Converts `self` into a Python object.
    ///
    /// This is equivalent to calling [`into_pyobject`][IntoPyObject::into_pyobject] followed
    /// with `.map_err(Into::into)` to convert the error type to [`PyErr`]. This is helpful
    /// for generic code which wants to make use of the `?` operator.
    #[inline]
    fn into_pyobject_or_pyerr(self, py: Python<'py>) -> PyResult<Self::Output> {
        match self.into_pyobject(py) {
            Ok(obj) => Ok(obj),
            Err(err) => Err(err.into()),
        }
    }
}

impl<'py, T> IntoPyObjectExt<'py> for T where T: IntoPyObject<'py> {}

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
///     let obj: Bound<'_, PyString> = PyString::new(py, "blah");
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
/// Note: depending on the implementation, the lifetime of the extracted result may
/// depend on the lifetime of the `obj` or the `prepared` variable.
///
/// For example, when extracting `&str` from a Python byte string, the resulting string slice will
/// point to the existing string data (lifetime: `'py`).
/// On the other hand, when extracting `&str` from a Python Unicode string, the preparation step
/// will convert the string to UTF-8, and the resulting string slice will have lifetime `'prepared`.
/// Since which case applies depends on the runtime type of the Python object,
/// both the `obj` and `prepared` variables must outlive the resulting string slice.
pub trait FromPyObject<'a, 'py>: Sized {
    /// Extracts `Self` from the bound smart pointer `obj`.
    ///
    /// Users are advised against calling this method directly: instead, use this via
    /// [`Bound<'_, PyAny>::extract`] or [`Py::extract`].
    fn extract(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self>;

    /// Deprecated name for [`FromPyObject::extract`]
    #[deprecated(since = "0.23.0", note = "replaced by `FromPyObject::extract`")]
    fn extract_bound(ob: &'a Bound<'py, PyAny>) -> PyResult<Self> {
        Self::extract(ob.as_borrowed())
    }

    /// Extracts the type hint information for this type when it appears as an argument.
    ///
    /// For example, `Vec<u32>` would return `Sequence[int]`.
    /// The default implementation returns `Any`, which is correct for any type.
    ///
    /// For most types, the return value for this method will be identical to that of
    /// [`IntoPyObject::type_output`]. It may be different for some types, such as `Dict`,
    /// to allow duck-typing: functions return `Dict` but take `Mapping` as argument.
    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        TypeInfo::Any
    }
}

/// A data structure that can be extracted without borrowing any data from the input
///
/// This is primarily useful for trait bounds. For example a `FromPyObject` implementation of a
/// wrapper type may be able to borrow data from the input, but a `FromPyObject` implementation of a
/// collection type may only extract owned data.
///
/// ```
/// # use pyo3::prelude::*;
/// pub struct MyWrapper<T>(T);
///
/// impl<'a, 'py, T> FromPyObject<'a, 'py> for MyWrapper<T>
/// where
///     T: FromPyObject<'a, 'py>
/// {
///     fn extract(obj: Borrowed<'a, 'py, PyAny>) -> PyResult<Self> {
///         obj.extract().map(MyWrapper)
///     }
/// }
///
/// pub struct MyVec<T>(Vec<T>);
///
/// impl<'py, T> FromPyObject<'_, 'py> for MyVec<T>
/// where
///     T: FromPyObjectOwned<'py> // 👈 can only extract owned values, because each `item` below
///                               //    is a temporary short lived owned reference
/// {
///     fn extract(obj: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
///         let mut v = MyVec(Vec::new());
///         for item in obj.try_iter()? {
///             v.0.push(item?.extract::<T>()?);
///         }
///         Ok(v)
///     }
/// }
/// ```
pub trait FromPyObjectOwned<'py>: for<'a> FromPyObject<'a, 'py> {}
impl<'py, T> FromPyObjectOwned<'py> for T where T: for<'a> FromPyObject<'a, 'py> {}

impl<T> FromPyObject<'_, '_> for T
where
    T: PyClass + Clone,
{
    fn extract(obj: Borrowed<'_, '_, PyAny>) -> PyResult<Self> {
        let bound = obj.downcast::<Self>()?;
        Ok(bound.try_borrow()?.clone())
    }
}

impl<'py, T> FromPyObject<'_, 'py> for PyRef<'py, T>
where
    T: PyClass,
{
    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        obj.downcast::<T>()?.try_borrow().map_err(Into::into)
    }
}

impl<'py, T> FromPyObject<'_, 'py> for PyRefMut<'py, T>
where
    T: PyClass<Frozen = False>,
{
    fn extract(obj: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        obj.downcast::<T>()?.try_borrow_mut().map_err(Into::into)
    }
}

impl<'py> IntoPyObject<'py> for () {
    type Target = PyTuple;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyTuple::empty(py))
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
