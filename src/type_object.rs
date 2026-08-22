//! Python type object information

use crate::ffi_ptr_ext::FfiPtrExt;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::{type_hint_identifier, PyStaticExpr};
use crate::types::{PyAny, PyType};
use crate::{ffi, Bound, Python};
use core::ptr;

/// `T: PyLayout<U>` represents that `T` is a concrete representation of `U` in the Python heap.
/// E.g., `PyClassObject` is a concrete representation of all `pyclass`es, and `ffi::PyObject`
/// is of `PyAny`.
///
/// This trait is intended to be used internally.
///
/// # Safety
///
/// This trait must only be implemented for types which represent valid layouts of Python objects.
pub unsafe trait PyLayout<T> {}

/// `T: PySizedLayout<U>` represents that `T` is not a instance of
/// [`PyVarObject`](https://docs.python.org/3/c-api/structures.html#c.PyVarObject).
///
/// In addition, that `T` is a concrete representation of `U`.
pub trait PySizedLayout<T>: PyLayout<T> + Sized {}

/// Python type information.
/// All Python native types (e.g., `PyDict`) and `#[pyclass]` structs implement this trait.
///
/// This trait is marked unsafe because:
///  - specifying the incorrect layout can lead to memory errors
///  - the return value of type_object must always point to the same PyTypeObject instance
///
/// It is safely implemented by the `pyclass` macro.
///
/// # Safety
///
/// Implementations must provide an implementation for `type_object_raw` which infallibly produces a
/// non-null pointer to the corresponding Python type object.
///
/// `is_type_of` must only return true for objects which can safely be treated as instances of `Self`.
///
/// `is_exact_type_of` must only return true for objects whose type is exactly `Self`.
pub unsafe trait PyTypeInfo: Sized {
    /// Class name.
    #[deprecated(
        since = "0.28.0",
        note = "prefer using `::type_object(py).name()` to get the correct runtime value"
    )]
    const NAME: &'static str;

    /// Module name, if any.
    #[deprecated(
        since = "0.28.0",
        note = "prefer using `::type_object(py).module()` to get the correct runtime value"
    )]
    const MODULE: Option<&'static str>;

    /// Provides the full python type as a type hint.
    ///
    /// This is also used as a building block for parametrized type hints, e.g. `Vec<T>` is
    /// hinted as `Self::TYPE_HINT[T::OUTPUT_TYPE]`, so it must stay unparametrized (e.g. `list`
    /// rather than `list[Any]`).
    #[cfg(feature = "experimental-inspect")]
    const TYPE_HINT: PyStaticExpr = type_hint_identifier!("_typeshed", "Incomplete");

    /// The type hint used for `Bound<'_, Self>` and `Py<Self>` when no more specific
    /// parametrization is known.
    ///
    /// Defaults to [`TYPE_HINT`](Self::TYPE_HINT). Generic native container types like
    /// [`PyList`](crate::types::PyList) override this to the fully parametrized form (e.g.
    /// `list[Any]`) so that generated stubs pass `mypy --strict`, while keeping `TYPE_HINT`
    /// itself unparametrized for reuse as described above.
    #[cfg(feature = "experimental-inspect")]
    const STANDALONE_TYPE_HINT: PyStaticExpr = Self::TYPE_HINT;

    /// Returns the PyTypeObject instance for this type.
    fn type_object_raw(py: Python<'_>) -> *mut ffi::PyTypeObject;

    /// Returns the safe abstraction over the type object.
    #[inline]
    fn type_object(py: Python<'_>) -> Bound<'_, PyType> {
        // Making the borrowed object `Bound` is necessary for soundness reasons. It's an extreme
        // edge case, but arbitrary Python code _could_ change the __class__ of an object and cause
        // the type object to be freed.
        //
        // By making `Bound` we assume ownership which is then safe against races.
        let tp = Self::type_object_raw(py).cast::<ffi::PyObject>();
        // SAFETY: the pointer is known to be a borrowed type object and we immediately make a new reference
        unsafe { tp.assume_borrowed_unchecked(py).cast_unchecked() }.to_owned()
    }

    /// Checks if `object` is an instance of this type or a subclass of this type.
    #[inline]
    fn is_type_of(object: &Bound<'_, PyAny>) -> bool {
        let tp = Self::type_object_raw(object.py());
        // SAFETY: pointers are known to be correct types and borrowed
        (unsafe { ffi::PyObject_TypeCheck(object.as_ptr(), tp) }) != 0
    }

    /// Checks if `object` is an instance of this type.
    #[inline]
    fn is_exact_type_of(object: &Bound<'_, PyAny>) -> bool {
        ptr::eq(
            // SAFETY: no additional requirements
            unsafe { ffi::Py_TYPE(object.as_ptr()) },
            Self::type_object_raw(object.py()),
        )
    }
}

/// Implemented by types which can be used as a concrete Python type inside `Py<T>` smart pointers.
///
/// # Safety
///
/// This trait is used to determine whether [`Bound::cast`] and similar functions can safely cast
/// to a concrete type. The implementor is responsible for ensuring that `type_check` only returns
/// true for objects which can safely be treated as Python instances of `Self`.
pub unsafe trait PyTypeCheck {
    /// Provides the full python type of the allowed values as a Python type hint.
    #[cfg(feature = "experimental-inspect")]
    const TYPE_HINT: PyStaticExpr;

    /// The type hint used for `Bound<'_, Self>` and `Py<Self>` when no more specific
    /// parametrization is known. Defaults to [`TYPE_HINT`](Self::TYPE_HINT).
    #[cfg(feature = "experimental-inspect")]
    const STANDALONE_TYPE_HINT: PyStaticExpr = Self::TYPE_HINT;

    /// Checks if `object` is an instance of `Self`, which may include a subtype.
    ///
    /// This should be equivalent to the Python expression `isinstance(object, Self)`.
    fn type_check(object: &Bound<'_, PyAny>) -> bool;

    /// Returns the expected type as a possible argument for the `isinstance` and `issubclass` function.
    ///
    /// It may be a single type or a tuple of types.
    fn classinfo_object(py: Python<'_>) -> Bound<'_, PyAny>;
}

// SAFETY: requirements upheld by impl of PyTypeInfo
unsafe impl<T> PyTypeCheck for T
where
    T: PyTypeInfo,
{
    #[cfg(feature = "experimental-inspect")]
    const TYPE_HINT: PyStaticExpr = <T as PyTypeInfo>::TYPE_HINT;

    #[cfg(feature = "experimental-inspect")]
    const STANDALONE_TYPE_HINT: PyStaticExpr = <T as PyTypeInfo>::STANDALONE_TYPE_HINT;

    #[inline]
    fn type_check(object: &Bound<'_, PyAny>) -> bool {
        T::is_type_of(object)
    }

    #[inline]
    fn classinfo_object(py: Python<'_>) -> Bound<'_, PyAny> {
        T::type_object(py).into_any()
    }
}

#[cfg(all(test, feature = "experimental-inspect"))]
mod tests {
    use super::*;
    use crate::platform::prelude::*;
    use crate::types::{PyDict, PyList, PyTuple};
    use crate::{Bound, FromPyObject, IntoPyObject, Py};

    #[test]
    fn container_standalone_type_hints_are_parametrized() {
        assert_eq!(
            <PyList as PyTypeInfo>::STANDALONE_TYPE_HINT.to_string(),
            "builtins.list[typing.Any]"
        );
        assert_eq!(
            <PyDict as PyTypeInfo>::STANDALONE_TYPE_HINT.to_string(),
            "builtins.dict[typing.Any, typing.Any]"
        );
        assert_eq!(
            <PyTuple as PyTypeInfo>::STANDALONE_TYPE_HINT.to_string(),
            "builtins.tuple[typing.Any, ...]"
        );
    }

    #[test]
    fn container_type_hints_stay_unparametrized_for_composition() {
        assert_eq!(
            <PyList as PyTypeInfo>::TYPE_HINT.to_string(),
            "builtins.list"
        );
        assert_eq!(
            <PyDict as PyTypeInfo>::TYPE_HINT.to_string(),
            "builtins.dict"
        );
        assert_eq!(
            <PyTuple as PyTypeInfo>::TYPE_HINT.to_string(),
            "builtins.tuple"
        );
    }

    #[test]
    fn container_bound_and_py_use_standalone_type_hints() {
        macro_rules! assert_container {
            ($ty:ty, $expected:expr) => {
                assert_eq!(
                    <$ty as PyTypeInfo>::STANDALONE_TYPE_HINT.to_string(),
                    $expected
                );
                assert_eq!(
                    <Bound<'_, $ty> as FromPyObject<'_, '_>>::INPUT_TYPE.to_string(),
                    $expected
                );
                assert_eq!(
                    <Py<$ty> as FromPyObject<'_, '_>>::INPUT_TYPE.to_string(),
                    $expected
                );
                assert_eq!(
                    <Bound<'_, $ty> as IntoPyObject<'_>>::OUTPUT_TYPE.to_string(),
                    $expected
                );
                assert_eq!(
                    <Py<$ty> as IntoPyObject<'_>>::OUTPUT_TYPE.to_string(),
                    $expected
                );
            };
        }

        assert_container!(PyList, "builtins.list[typing.Any]");
        assert_container!(PyDict, "builtins.dict[typing.Any, typing.Any]");
        assert_container!(PyTuple, "builtins.tuple[typing.Any, ...]");
    }
}
