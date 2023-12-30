//! Python type object information

use crate::types::{PyAny, PyType};
use crate::{ffi, PyNativeType, Python};

/// `T: PyLayout<U>` represents that `T` is a concrete representation of `U` in the Python heap.
/// E.g., `PyCell` is a concrete representation of all `pyclass`es, and `ffi::PyObject`
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
    type AsRefTarget: PyNativeType;
}

unsafe impl<T> HasPyGilRef for T
where
    T: PyNativeType,
{
    type AsRefTarget = Self;
}

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
/// # Example
///
/// To use PyTypeInfo to represent a custom Python class that is not implemented in Rust:
///
/// ```
/// struct Formatter(PyAny);
/// unsafe impl PyTypeInfo for Formatter {
///     const NAME: &'static str = "MyPyType";
///     const MODULE: Option<&'static str> = Option::Some("string");
///     type AsRefTarget = PyAny;
///
///     fn type_object_raw(py: Python<'_>) -> *mut PyTypeObject {
///         let modu = py.import("string").expect("Couldn't import string module");
///         let typ_any = modu.getattr("Formatter").expect("Couldn't get Python class");
///         let typ = typ_any.extract::<&PyType>().expect("Couldn't downcast Python class to PyType");
///         typ.get_type_ptr()
///     }
/// }
/// unsafe impl PyNativeType for Formatter {}
/// ```
pub unsafe trait PyTypeInfo: Sized + HasPyGilRef {
    /// Class name.
    const NAME: &'static str;

    /// Module name, if any.
    const MODULE: Option<&'static str>;

    /// Returns the PyTypeObject instance for this type.
    fn type_object_raw(py: Python<'_>) -> *mut ffi::PyTypeObject;

    /// Returns the safe abstraction over the type object.
    #[inline]
    fn type_object(py: Python<'_>) -> &PyType {
        unsafe { py.from_borrowed_ptr(Self::type_object_raw(py) as _) }
    }

    /// Checks if `object` is an instance of this type or a subclass of this type.
    #[inline]
    fn is_type_of(object: &PyAny) -> bool {
        unsafe { ffi::PyObject_TypeCheck(object.as_ptr(), Self::type_object_raw(object.py())) != 0 }
    }

    /// Checks if `object` is an instance of this type.
    #[inline]
    fn is_exact_type_of(object: &PyAny) -> bool {
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
    fn type_check(object: &PyAny) -> bool;
}

impl<T> PyTypeCheck for T
where
    T: PyTypeInfo,
{
    const NAME: &'static str = <T as PyTypeInfo>::NAME;

    #[inline]
    fn type_check(object: &PyAny) -> bool {
        <T as PyTypeInfo>::is_type_of(object)
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
