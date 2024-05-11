//! Python type object information

use crate::ffi_ptr_ext::FfiPtrExt;
use crate::types::any::PyAnyMethods;
use crate::types::{PyAny, PyType};
#[cfg(feature = "gil-refs")]
use crate::PyNativeType;
use crate::{ffi, Bound, Python};

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
/// [`PyVarObject`](https://docs.python.org/3.8/c-api/structures.html?highlight=pyvarobject#c.PyVarObject).
/// In addition, that `T` is a concrete representation of `U`.
pub trait PySizedLayout<T>: PyLayout<T> + Sized {}

/// Specifies that this type has a "GIL-bound Reference" form.
///
/// This is expected to be deprecated in the near future, see <https://github.com/PyO3/pyo3/issues/3382>
///
/// # Safety
///
/// - `Py<Self>::as_ref` will hand out references to `Self::AsRefTarget`.
/// - `Self::AsRefTarget` must have the same layout as `UnsafeCell<ffi::PyAny>`.
pub unsafe trait HasPyGilRef {
    /// Utility type to make Py::as_ref work.
    #[cfg(feature = "gil-refs")]
    type AsRefTarget: PyNativeType;
}

#[cfg(feature = "gil-refs")]
unsafe impl<T> HasPyGilRef for T
where
    T: PyNativeType,
{
    type AsRefTarget = Self;
}

#[cfg(not(feature = "gil-refs"))]
unsafe impl<T> HasPyGilRef for T {}

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
pub unsafe trait PyTypeInfo: Sized + HasPyGilRef {
    /// Class name.
    const NAME: &'static str;

    /// Module name, if any.
    const MODULE: Option<&'static str>;

    /// Returns the PyTypeObject instance for this type.
    fn type_object_raw(py: Python<'_>) -> *mut ffi::PyTypeObject;

    /// Returns the safe abstraction over the type object.
    #[inline]
    #[cfg(feature = "gil-refs")]
    #[deprecated(
        since = "0.21.0",
        note = "`PyTypeInfo::type_object` will be replaced by `PyTypeInfo::type_object_bound` in a future PyO3 version"
    )]
    fn type_object(py: Python<'_>) -> &PyType {
        // This isn't implemented in terms of `type_object_bound` because this just borrowed the
        // object, for legacy reasons.
        #[allow(deprecated)]
        unsafe {
            py.from_borrowed_ptr(Self::type_object_raw(py) as _)
        }
    }

    /// Returns the safe abstraction over the type object.
    #[inline]
    fn type_object_bound(py: Python<'_>) -> Bound<'_, PyType> {
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
                .downcast_into_unchecked()
        }
    }

    /// Checks if `object` is an instance of this type or a subclass of this type.
    #[inline]
    #[cfg(feature = "gil-refs")]
    #[deprecated(
        since = "0.21.0",
        note = "`PyTypeInfo::is_type_of` will be replaced by `PyTypeInfo::is_type_of_bound` in a future PyO3 version"
    )]
    fn is_type_of(object: &PyAny) -> bool {
        Self::is_type_of_bound(&object.as_borrowed())
    }

    /// Checks if `object` is an instance of this type or a subclass of this type.
    #[inline]
    fn is_type_of_bound(object: &Bound<'_, PyAny>) -> bool {
        unsafe { ffi::PyObject_TypeCheck(object.as_ptr(), Self::type_object_raw(object.py())) != 0 }
    }

    /// Checks if `object` is an instance of this type.
    #[inline]
    #[cfg(feature = "gil-refs")]
    #[deprecated(
        since = "0.21.0",
        note = "`PyTypeInfo::is_exact_type_of` will be replaced by `PyTypeInfo::is_exact_type_of_bound` in a future PyO3 version"
    )]
    fn is_exact_type_of(object: &PyAny) -> bool {
        Self::is_exact_type_of_bound(&object.as_borrowed())
    }

    /// Checks if `object` is an instance of this type.
    #[inline]
    fn is_exact_type_of_bound(object: &Bound<'_, PyAny>) -> bool {
        unsafe { ffi::Py_TYPE(object.as_ptr()) == Self::type_object_raw(object.py()) }
    }
}

/// Implemented by types which can be used as a concrete Python type inside `Py<T>` smart pointers.
pub trait PyTypeCheck: HasPyGilRef {
    /// Name of self. This is used in error messages, for example.
    const NAME: &'static str;

    /// Checks if `object` is an instance of `Self`, which may include a subtype.
    ///
    /// This should be equivalent to the Python expression `isinstance(object, Self)`.
    fn type_check(object: &Bound<'_, PyAny>) -> bool;
}

impl<T> PyTypeCheck for T
where
    T: PyTypeInfo,
{
    const NAME: &'static str = <T as PyTypeInfo>::NAME;

    #[inline]
    fn type_check(object: &Bound<'_, PyAny>) -> bool {
        T::is_type_of_bound(object)
    }
}

#[inline]
pub(crate) unsafe fn get_tp_alloc(tp: *mut ffi::PyTypeObject) -> Option<ffi::allocfunc> {
    #[cfg(not(Py_LIMITED_API))]
    {
        (*tp).tp_alloc
    }

    #[cfg(Py_LIMITED_API)]
    {
        let ptr = ffi::PyType_GetSlot(tp, ffi::Py_tp_alloc);
        std::mem::transmute(ptr)
    }
}

#[inline]
pub(crate) unsafe fn get_tp_free(tp: *mut ffi::PyTypeObject) -> ffi::freefunc {
    #[cfg(not(Py_LIMITED_API))]
    {
        (*tp).tp_free.unwrap()
    }

    #[cfg(Py_LIMITED_API)]
    {
        let ptr = ffi::PyType_GetSlot(tp, ffi::Py_tp_free);
        debug_assert_ne!(ptr, std::ptr::null_mut());
        std::mem::transmute(ptr)
    }
}
