//! Python type object information

use crate::ffi_ptr_ext::FfiPtrExt;
#[cfg(feature = "experimental-inspect")]
use crate::inspect::{type_hint_identifier, PyStaticExpr};
use crate::types::{PyAny, PyType};
use crate::{ffi, Bound, Python};
use std::ptr;

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
    #[cfg(feature = "experimental-inspect")]
    const TYPE_HINT: PyStaticExpr = type_hint_identifier!("_typeshed", "Incomplete");

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
        unsafe {
            Self::type_object_raw(py)
                .cast::<ffi::PyObject>()
                .assume_borrowed_unchecked(py)
                .to_owned()
                .cast_into_unchecked()
        }
    }

    /// Checks if `object` is an instance of this type or a subclass of this type.
    #[inline]
    fn is_type_of(object: &Bound<'_, PyAny>) -> bool {
        unsafe { ffi::PyObject_TypeCheck(object.as_ptr(), Self::type_object_raw(object.py())) != 0 }
    }

    /// Checks if `object` is an instance of this type.
    #[inline]
    fn is_exact_type_of(object: &Bound<'_, PyAny>) -> bool {
        unsafe {
            ptr::eq(
                ffi::Py_TYPE(object.as_ptr()),
                Self::type_object_raw(object.py()),
            )
        }
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
    /// Name of self. This is used in error messages, for example.
    #[deprecated(
        since = "0.27.0",
        note = "Use ::classinfo_object() instead and format the type name at runtime. Note that using built-in cast features is often better than manual PyTypeCheck usage."
    )]
    const NAME: &'static str;

    /// Provides the full python type of the allowed values as a Python type hint.
    #[cfg(feature = "experimental-inspect")]
    const TYPE_HINT: PyStaticExpr;

    /// Checks if `object` is an instance of `Self`, which may include a subtype.
    ///
    /// This should be equivalent to the Python expression `isinstance(object, Self)`.
    fn type_check(object: &Bound<'_, PyAny>) -> bool;

    /// Returns the expected type as a possible argument for the `isinstance` and `issubclass` function.
    ///
    /// It may be a single type or a tuple of types.
    fn classinfo_object(py: Python<'_>) -> Bound<'_, PyAny>;
}

unsafe impl<T> PyTypeCheck for T
where
    T: PyTypeInfo,
{
    #[allow(deprecated)]
    const NAME: &'static str = T::NAME;

    #[cfg(feature = "experimental-inspect")]
    const TYPE_HINT: PyStaticExpr = <T as PyTypeInfo>::TYPE_HINT;

    #[inline]
    fn type_check(object: &Bound<'_, PyAny>) -> bool {
        T::is_type_of(object)
    }

    #[inline]
    fn classinfo_object(py: Python<'_>) -> Bound<'_, PyAny> {
        T::type_object(py).into_any()
    }
}
