use crate::ffi_ptr_ext::FfiPtrExt;
use crate::{ffi, types::any::PyAnyMethods, Borrowed, Bound, PyAny, PyTypeInfo, Python};

/// Represents the Python `None` object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyNone>`][crate::Py] or [`Bound<'py, PyNone>`][Bound].
#[repr(transparent)]
pub struct PyNone(PyAny);

pyobject_native_type_named!(PyNone);

impl PyNone {
    /// Returns the `None` object.
    #[inline]
    pub fn get(py: Python<'_>) -> Borrowed<'_, '_, PyNone> {
        unsafe { ffi::Py_None().assume_borrowed(py).cast_unchecked() }
    }
}

unsafe impl PyTypeInfo for PyNone {
    const NAME: &'static str = "NoneType";

    const MODULE: Option<&'static str> = None;

    fn type_object_raw(_py: Python<'_>) -> *mut ffi::PyTypeObject {
        unsafe { ffi::Py_TYPE(ffi::Py_None()) }
    }

    #[inline]
    fn is_type_of(object: &Bound<'_, PyAny>) -> bool {
        // NoneType is not usable as a base type
        Self::is_exact_type_of(object)
    }

    #[inline]
    fn is_exact_type_of(object: &Bound<'_, PyAny>) -> bool {
        object.is(&**Self::get(object.py()))
    }
}

#[cfg(test)]
mod tests {
    use crate::types::any::PyAnyMethods;
    use crate::types::{PyDict, PyNone};
    use crate::{PyTypeInfo, Python};

    #[test]
    fn test_none_is_itself() {
        Python::attach(|py| {
            assert!(PyNone::get(py).is_instance_of::<PyNone>());
            assert!(PyNone::get(py).is_exact_instance_of::<PyNone>());
        })
    }

    #[test]
    fn test_none_type_object_consistent() {
        Python::attach(|py| {
            assert!(PyNone::get(py).get_type().is(PyNone::type_object(py)));
        })
    }

    #[test]
    fn test_none_is_none() {
        Python::attach(|py| {
            assert!(PyNone::get(py).cast::<PyNone>().unwrap().is_none());
        })
    }

    #[test]
    fn test_dict_is_not_none() {
        Python::attach(|py| {
            assert!(PyDict::new(py).cast::<PyNone>().is_err());
        })
    }
}
