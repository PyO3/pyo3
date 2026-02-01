//! Defines conversions between Rust and Python types.
use crate::err::PyResult;
use crate::impl_::pyclass::ExtractPyClassWithClone;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::types::TypeInfo;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::{type_hint_identifier, type_hint_subscript, PyStaticExpr};
use crate::pyclass::boolean_struct::False;
use crate::pyclass::{PyClassGuardError, PyClassGuardMutError};
#[cfg(feature = "experimental-inspect")]
use crate::types::PyList;
use crate::types::PyTuple;
use crate::{
    Borrowed, Bound, BoundObject, Py, PyAny, PyClass, PyClassGuard, PyErr, PyRef, PyRefMut,
    PyTypeCheck, Python,
};
use std::convert::Infallible;
use std::marker::PhantomData;

/// Defines a conversion from a Rust type to a Python object, which may fail.
///
/// This trait has `#[derive(IntoPyObject)]` to automatically implement it for simple types and
/// `#[derive(IntoPyObjectRef)]` to implement the same for references.
///
/// It functions similarly to std's [`TryInto`] trait, but requires a [`Python<'py>`] token
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
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be converted to a Python object",
    note = "`IntoPyObject` is automatically implemented by the `#[pyclass]` macro",
    note = "if you do not wish to have a corresponding Python type, implement it manually",
    note = "if you do not own `{Self}` you can perform a manual conversion to one of the types in `pyo3::types::*`"
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
    const OUTPUT_TYPE: PyStaticExpr = type_hint_identifier!("_typeshed", "Incomplete");

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

    /// The output type of [`IntoPyObject::owned_sequence_into_pyobject`] and [`IntoPyObject::borrowed_sequence_into_pyobject`]
    #[cfg(feature = "experimental-inspect")]
    #[doc(hidden)]
    const SEQUENCE_OUTPUT_TYPE: PyStaticExpr =
        type_hint_subscript!(PyList::TYPE_HINT, Self::OUTPUT_TYPE);
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

impl<'py, T: PyTypeCheck> IntoPyObject<'py> for Bound<'py, T> {
    type Target = T;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = T::TYPE_HINT;

    fn into_pyobject(self, _py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self)
    }
}

impl<'a, 'py, T: PyTypeCheck> IntoPyObject<'py> for &'a Bound<'py, T> {
    type Target = T;
    type Output = Borrowed<'a, 'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = T::TYPE_HINT;

    fn into_pyobject(self, _py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.as_borrowed())
    }
}

impl<'a, 'py, T: PyTypeCheck> IntoPyObject<'py> for Borrowed<'a, 'py, T> {
    type Target = T;
    type Output = Borrowed<'a, 'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = T::TYPE_HINT;

    fn into_pyobject(self, _py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self)
    }
}

impl<'a, 'py, T: PyTypeCheck> IntoPyObject<'py> for &Borrowed<'a, 'py, T> {
    type Target = T;
    type Output = Borrowed<'a, 'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = T::TYPE_HINT;

    fn into_pyobject(self, _py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(*self)
    }
}

impl<'py, T: PyTypeCheck> IntoPyObject<'py> for Py<T> {
    type Target = T;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = T::TYPE_HINT;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.into_bound(py))
    }
}

impl<'a, 'py, T: PyTypeCheck> IntoPyObject<'py> for &'a Py<T> {
    type Target = T;
    type Output = Borrowed<'a, 'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr = T::TYPE_HINT;

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
    const OUTPUT_TYPE: PyStaticExpr = <&'a T as IntoPyObject<'py>>::OUTPUT_TYPE;

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
/// Normal usage is through the `extract` methods on [`Bound`], [`Borrowed`] and [`Py`], which
/// forward to this trait.
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
/// Note: Depending on the Python version and implementation, some [`FromPyObject`] implementations
/// may produce a result that borrows into the Python type. This is described by the input lifetime
/// `'a` of `obj`.
///
/// Types that must not borrow from the input can use [`FromPyObjectOwned`] as a restriction. This
/// is most often the case for collection types. See its documentation for more details.
///
/// # How to implement [`FromPyObject`]?
/// ## `#[derive(FromPyObject)]`
/// The simplest way to implement [`FromPyObject`] for a custom type is to make use of our derive
/// macro.
/// ```rust,no_run
/// # #![allow(dead_code)]
/// use pyo3::prelude::*;
///
/// #[derive(FromPyObject)]
/// struct MyObject {
///     msg: String,
///     list: Vec<u32>
/// }
/// # fn main() {}
/// ```
/// By default this will try to extract each field from the Python object by attribute access, but
/// this can be customized. For more information about the derive macro, its configuration as well
/// as its working principle for other types, take a look at the [guide].
///
/// In case the derive macro is not sufficient or can not be used for some other reason,
/// [`FromPyObject`] can be implemented manually. In the following types without lifetime parameters
/// are handled first, because they are a little bit simpler. Types with lifetime parameters are
/// explained below.
///
/// ## Manual implementation for types without lifetime
/// Types that do not contain lifetime parameters are unable to borrow from the Python object, so
/// the lifetimes of [`FromPyObject`] can be elided:
/// ```rust,no_run
/// # #![allow(dead_code)]
/// use pyo3::prelude::*;
///
/// struct MyObject {
///     msg: String,
///     list: Vec<u32>
/// }
///
/// impl FromPyObject<'_, '_> for MyObject {
///     type Error = PyErr;
///
///     fn extract(obj: Borrowed<'_, '_, PyAny>) -> Result<Self, Self::Error> {
///         Ok(MyObject {
///             msg: obj.getattr("msg")?.extract()?,
///             list: obj.getattr("list")?.extract()?,
///         })
///     }
/// }
///
/// # fn main() {}
/// ```
/// This is basically what the derive macro above expands to.
///
/// ## Manual implementation for types with lifetime parameters
/// For types that contain lifetimes, these lifetimes need to be bound to the corresponding
/// [`FromPyObject`] lifetime. This is roughly how the extraction of a typed [`Bound`] is
/// implemented within PyO3.
///
/// ```rust,no_run
/// # #![allow(dead_code)]
/// use pyo3::prelude::*;
/// use pyo3::types::PyString;
///
/// struct MyObject<'py>(Bound<'py, PyString>);
///
/// impl<'py> FromPyObject<'_, 'py> for MyObject<'py> {
///     type Error = PyErr;
///
///     fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
///         Ok(MyObject(obj.cast()?.to_owned()))
///     }
/// }
///
/// # fn main() {}
/// ```
///
/// # Details
/// [`Cow<'a, str>`] is an example of an output type that may or may not borrow from the input
/// lifetime `'a`. Which variant will be produced depends on the runtime type of the Python object.
/// For a Python byte string, the existing string data can be borrowed for `'a` into a
/// [`Cow::Borrowed`]. For a Python Unicode string, the data may have to be reencoded to UTF-8, and
/// copied into a [`Cow::Owned`]. It does _not_ depend on the Python lifetime `'py`.
///
/// The output type may also depend on the Python lifetime `'py`. This allows the output type to
/// keep interacting with the Python interpreter. See also [`Bound<'py, T>`].
///
/// [`Cow<'a, str>`]: std::borrow::Cow
/// [`Cow::Borrowed`]: std::borrow::Cow::Borrowed
/// [`Cow::Owned`]: std::borrow::Cow::Owned
/// [guide]: https://pyo3.rs/latest/conversions/traits.html#deriving-frompyobject
pub trait FromPyObject<'a, 'py>: Sized {
    /// The type returned in the event of a conversion error.
    ///
    /// For most use cases defaulting to [PyErr] here is perfectly acceptable. Using a custom error
    /// type can be used to avoid having to create a Python exception object in the case where that
    /// exception never reaches Python. This may lead to slightly better performance under certain
    /// conditions.
    ///
    /// # Note
    /// Unfortunately `Try` and thus `?` is based on [`From`], not [`Into`], so implementations may
    /// need to use `.map_err(Into::into)` sometimes to convert a generic `Error` into a [`PyErr`].
    type Error: Into<PyErr>;

    /// Provides the type hint information for this type when it appears as an argument.
    ///
    /// For example, `Vec<u32>` would be `collections.abc.Sequence[int]`.
    /// The default value is `typing.Any`, which is correct for any type.
    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr = type_hint_identifier!("_typeshed", "Incomplete");

    /// Extracts `Self` from the bound smart pointer `obj`.
    ///
    /// Users are advised against calling this method directly: instead, use this via
    /// [`Bound<'_, PyAny>::extract`](crate::types::any::PyAnyMethods::extract) or [`Py::extract`].
    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error>;

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

    /// Specialization hook for extracting sequences for types like `Vec<u8>` and `[u8; N]`,
    /// where the bytes can be directly copied from some python objects without going through
    /// iteration.
    #[doc(hidden)]
    #[inline(always)]
    fn sequence_extractor(
        _obj: Borrowed<'_, 'py, PyAny>,
        _: private::Token,
    ) -> Option<impl FromPyObjectSequence<Target = Self>> {
        struct NeverASequence<T>(PhantomData<T>);

        impl<T> FromPyObjectSequence for NeverASequence<T> {
            type Target = T;

            fn to_vec(&self) -> Vec<Self::Target> {
                unreachable!()
            }

            fn to_array<const N: usize>(&self) -> PyResult<[Self::Target; N]> {
                unreachable!()
            }
        }

        Option::<NeverASequence<Self>>::None
    }

    /// Helper used to make a specialized path in extracting `DateTime<Tz>` where `Tz` is
    /// `chrono::Local`, which will accept "naive" datetime objects as being in the local timezone.
    #[cfg(feature = "chrono-local")]
    #[inline]
    fn as_local_tz(_: private::Token) -> Option<Self> {
        None
    }
}

mod from_py_object_sequence {
    use crate::PyResult;

    /// Private trait for implementing specialized sequence extraction for `Vec<u8>` and `[u8; N]`
    #[doc(hidden)]
    pub trait FromPyObjectSequence {
        type Target;

        fn to_vec(&self) -> Vec<Self::Target>;

        fn to_array<const N: usize>(&self) -> PyResult<[Self::Target; N]>;
    }
}

// Only reachable / implementable inside PyO3 itself.
pub(crate) use from_py_object_sequence::FromPyObjectSequence;

/// A data structure that can be extracted without borrowing any data from the input.
///
/// This is primarily useful for trait bounds. For example a [`FromPyObject`] implementation of a
/// wrapper type may be able to borrow data from the input, but a [`FromPyObject`] implementation of
/// a collection type may only extract owned data.
///
/// For example [`PyList`] will not hand out references tied to its own lifetime, but "owned"
/// references independent of it. (Similar to [`Vec<Arc<T>>`] where you clone the [`Arc<T>`] out).
/// This makes it impossible to collect borrowed types in a collection, since they would not borrow
/// from the original [`PyList`], but the much shorter lived element reference. See the example
/// below.
///
/// ```,no_run
/// # use pyo3::prelude::*;
/// # #[allow(dead_code)]
/// pub struct MyWrapper<T>(T);
///
/// impl<'a, 'py, T> FromPyObject<'a, 'py> for MyWrapper<T>
/// where
///     T: FromPyObject<'a, 'py>
/// {
///     type Error = T::Error;
///
///     fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
///         obj.extract().map(MyWrapper)
///     }
/// }
///
/// # #[allow(dead_code)]
/// pub struct MyVec<T>(Vec<T>);
///
/// impl<'py, T> FromPyObject<'_, 'py> for MyVec<T>
/// where
///     T: FromPyObjectOwned<'py> // 👈 can only extract owned values, because each `item` below
///                               //    is a temporary short lived owned reference
/// {
///     type Error = PyErr;
///
///     fn extract(obj: Borrowed<'_, 'py, PyAny>) -> Result<Self, Self::Error> {
///         let mut v = MyVec(Vec::new());
///         for item in obj.try_iter()? {
///             v.0.push(item?.extract::<T>().map_err(Into::into)?);
///         }
///         Ok(v)
///     }
/// }
/// ```
///
/// [`PyList`]: crate::types::PyList
/// [`Arc<T>`]: std::sync::Arc
pub trait FromPyObjectOwned<'py>: for<'a> FromPyObject<'a, 'py> {}
impl<'py, T> FromPyObjectOwned<'py> for T where T: for<'a> FromPyObject<'a, 'py> {}

impl<'a, 'py, T> FromPyObject<'a, 'py> for T
where
    T: PyClass + Clone + ExtractPyClassWithClone,
{
    type Error = PyClassGuardError<'a, 'py>;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr = <T as crate::PyTypeInfo>::TYPE_HINT;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        Ok(obj.extract::<PyClassGuard<'_, T>>()?.clone())
    }
}

impl<'a, 'py, T> FromPyObject<'a, 'py> for PyRef<'py, T>
where
    T: PyClass,
{
    type Error = PyClassGuardError<'a, 'py>;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr = <T as crate::PyTypeInfo>::TYPE_HINT;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        obj.cast::<T>()
            .map_err(|e| PyClassGuardError(Some(e)))?
            .try_borrow()
            .map_err(|_| PyClassGuardError(None))
    }
}

impl<'a, 'py, T> FromPyObject<'a, 'py> for PyRefMut<'py, T>
where
    T: PyClass<Frozen = False>,
{
    type Error = PyClassGuardMutError<'a, 'py>;

    #[cfg(feature = "experimental-inspect")]
    const INPUT_TYPE: PyStaticExpr = <T as crate::PyTypeInfo>::TYPE_HINT;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        obj.cast::<T>()
            .map_err(|e| PyClassGuardMutError(Some(e)))?
            .try_borrow_mut()
            .map_err(|_| PyClassGuardMutError(None))
    }
}

impl<'py> IntoPyObject<'py> for () {
    type Target = PyTuple;
    type Output = Bound<'py, Self::Target>;
    type Error = Infallible;

    #[cfg(feature = "experimental-inspect")]
    const OUTPUT_TYPE: PyStaticExpr =
        type_hint_subscript!(PyTuple::TYPE_HINT, PyStaticExpr::Tuple { elts: &[] });

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

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "macros")]
    fn test_pyclass_skip_from_py_object() {
        use crate::{types::PyAnyMethods, FromPyObject, IntoPyObject, PyErr, Python};

        #[crate::pyclass(crate = "crate", skip_from_py_object)]
        #[derive(Clone)]
        struct Foo(i32);

        impl<'py> FromPyObject<'_, 'py> for Foo {
            type Error = PyErr;

            fn extract(obj: crate::Borrowed<'_, 'py, crate::PyAny>) -> Result<Self, Self::Error> {
                if let Ok(obj) = obj.cast::<Self>() {
                    Ok(obj.borrow().clone())
                } else {
                    obj.extract::<i32>().map(Self)
                }
            }
        }
        Python::attach(|py| {
            let foo1 = 42i32.into_pyobject(py)?;
            assert_eq!(foo1.extract::<Foo>()?.0, 42);

            let foo2 = Foo(0).into_pyobject(py)?;
            assert_eq!(foo2.extract::<Foo>()?.0, 0);

            Ok::<_, PyErr>(())
        })
        .unwrap();
    }

    #[test]
    #[cfg(feature = "macros")]
    fn test_pyclass_from_py_object() {
        use crate::{types::PyAnyMethods, IntoPyObject, PyErr, Python};

        #[crate::pyclass(crate = "crate", from_py_object)]
        #[derive(Clone)]
        struct Foo(i32);

        Python::attach(|py| {
            let foo1 = 42i32.into_pyobject(py)?;
            assert!(foo1.extract::<Foo>().is_err());

            let foo2 = Foo(0).into_pyobject(py)?;
            assert_eq!(foo2.extract::<Foo>()?.0, 0);

            Ok::<_, PyErr>(())
        })
        .unwrap();
    }
}
