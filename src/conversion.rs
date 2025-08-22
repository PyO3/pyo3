//! Defines conversions between Rust and Python types.
use crate::err::PyResult;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
use crate::pyclass::boolean_struct::False;
use crate::types::PyTuple;
use crate::{Borrowed, Bound, BoundObject, Py, PyAny, PyClass, PyErr, PyRef, PyRefMut, Python};
use std::convert::Infallible;

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

    /// Extracts the type hint information for this type when it appears as a return value.
    ///
    /// For example, `Vec<u32>` would return `List[int]`.
    /// The default implementation returns `Any`, which is correct for any type.
    ///
    /// For most types, the return value for this method will be identical to that of [`FromPyObject::INPUT_TYPE`].
    /// It may be different for some types, such as `Dict`, to allow duck-typing: functions return `Dict` but take `Mapping` as argument.
    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = "typing.Any";

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

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: &'static str = <&'a T as IntoPyObject<'py>>::OUTPUT_TYPE;

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
/// Python::attach(|py| {
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
    /// Provides the type hint information for this type when it appears as an argument.
    ///
    /// For example, `Vec<u32>` would be `collections.abc.Sequence[int]`.
    /// The default value is `typing.Any`, which is correct for any type.
    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = "typing.Any";

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
    /// For most types, the return value for this method will be identical to that of
    /// [`IntoPyObject::type_output`]. It may be different for some types, such as `Dict`,
    /// to allow duck-typing: functions return `Dict` but take `Mapping` as argument.
    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        TypeInfo::Any
    }
}

mod from_py_object_bound_sealed {
    use crate::{pyclass::boolean_struct::False, PyClass, PyClassGuard, PyClassGuardMut};

    /// Private seal for the `FromPyObjectBound` trait.
    ///
    /// This prevents downstream types from implementing the trait before
    /// PyO3 is ready to declare the trait as public API.
    pub trait Sealed {}

    // This generic implementation is why the seal is separate from
    // `crate::sealed::Sealed`.
    impl<'py, T> Sealed for T where T: super::FromPyObject<'py> {}
    impl<T> Sealed for PyClassGuard<'_, T> where T: PyClass {}
    impl<T> Sealed for PyClassGuardMut<'_, T> where T: PyClass<Frozen = False> {}
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
    /// Provides the type hint information for this type when it appears as an argument.
    ///
    /// For example, `Vec<u32>` would be `collections.abc.Sequence[int]`.
    /// The default value is `typing.Any`, which is correct for any type.
    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = "typing.Any";

    /// Extracts `Self` from the bound smart pointer `obj`.
    ///
    /// Users are advised against calling this method directly: instead, use this via
    /// [`Bound<'_, PyAny>::extract`](crate::types::any::PyAnyMethods::extract) or [`Py::extract`].
    fn from_py_object_bound(ob: Borrowed<'a, 'py, PyAny>) -> PyResult<Self>;

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

impl<'py, T> FromPyObjectBound<'_, 'py> for T
where
    T: FromPyObject<'py>,
{
    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = T::INPUT_TYPE;

    fn from_py_object_bound(ob: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        Self::extract_bound(&ob)
    }

    #[cfg(feature = "experimental-inspect")]
    fn type_input() -> TypeInfo {
        <T as FromPyObject>::type_input()
    }
}

impl<T> FromPyObject<'_> for T
where
    T: PyClass + Clone,
{
    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = <T as crate::impl_::pyclass::PyClassImpl>::TYPE_NAME;

    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        let bound = obj.cast::<Self>()?;
        Ok(bound.try_borrow()?.clone())
    }
}

impl<'py, T> FromPyObject<'py> for PyRef<'py, T>
where
    T: PyClass,
{
    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = <T as crate::impl_::pyclass::PyClassImpl>::TYPE_NAME;

    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        obj.cast::<T>()?.try_borrow().map_err(Into::into)
    }
}

impl<'py, T> FromPyObject<'py> for PyRefMut<'py, T>
where
    T: PyClass<Frozen = False>,
{
    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: &'static str = <T as crate::impl_::pyclass::PyClassImpl>::TYPE_NAME;

    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        obj.cast::<T>()?.try_borrow_mut().map_err(Into::into)
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
/// Python::attach(|py| {
///     let pyvalue = Py::new(py, t).unwrap().to_object(py);
///     let t: TestClass = pyvalue.extract(py).unwrap();
/// })
/// ```
mod test_no_clone {}
