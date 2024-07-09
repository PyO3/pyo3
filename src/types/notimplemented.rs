use crate::{
    ffi, ffi_ptr_ext::FfiPtrExt, types::any::PyAnyMethods, Borrowed, Bound, PyAny, PyTypeInfo,
    Python,
};

/// Represents the Python `NotImplemented` object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyNotImplemented>`][crate::Py] or [`Bound<'py, PyNotImplemented>`][Bound].
#[repr(transparent)]
pub struct PyNotImplemented(PyAny);

pyobject_native_type_named!(PyNotImplemented);
pyobject_native_type_extract!(PyNotImplemented);

impl PyNotImplemented {
    /// Returns the `NotImplemented` object.
    #[cfg(feature = "gil-refs")]
    #[deprecated(
        since = "0.21.0",
        note = "`PyNotImplemented::get` will be replaced by `PyNotImplemented::get_bound` in a future PyO3 version"
    )]
    #[inline]
    pub fn get(py: Python<'_>) -> &PyNotImplemented {
        Self::get_bound(py).into_gil_ref()
    }

    /// Returns the `NotImplemented` object.
    #[inline]
    pub fn get_bound(py: Python<'_>) -> Borrowed<'_, '_, PyNotImplemented> {
        unsafe {
            ffi::Py_NotImplemented()
                .assume_borrowed(py)
                .downcast_unchecked()
        }
    }
}

unsafe impl PyTypeInfo for PyNotImplemented {
    const NAME: &'static str = "NotImplementedType";
    const MODULE: Option<&'static str> = None;

    fn type_object_raw(_py: Python<'_>) -> *mut ffi::PyTypeObject {
        unsafe { ffi::Py_TYPE(ffi::Py_NotImplemented()) }
    }

    #[inline]
    fn is_type_of_bound(object: &Bound<'_, PyAny>) -> bool {
        // NotImplementedType is not usable as a base type
        Self::is_exact_type_of_bound(object)
    }

    #[inline]
    fn is_exact_type_of_bound(object: &Bound<'_, PyAny>) -> bool {
        object.is(&**Self::get_bound(object.py()))
    }
}

#[cfg(test)]
mod tests {
    use crate::types::any::PyAnyMethods;
    use crate::types::{PyDict, PyNotImplemented};
    use crate::{PyTypeInfo, Python};

    #[test]
    fn test_notimplemented_is_itself() {
        Python::with_gil(|py| {
            assert!(PyNotImplemented::get_bound(py).is_instance_of::<PyNotImplemented>());
            assert!(PyNotImplemented::get_bound(py).is_exact_instance_of::<PyNotImplemented>());
        })
    }

    #[test]
    fn test_notimplemented_type_object_consistent() {
        Python::with_gil(|py| {
            assert!(PyNotImplemented::get_bound(py)
                .get_type()
                .is(&PyNotImplemented::type_object_bound(py)));
        })
    }

    #[test]
    fn test_dict_is_not_notimplemented() {
        Python::with_gil(|py| {
            assert!(PyDict::new_bound(py)
                .downcast::<PyNotImplemented>()
                .is_err());
        })
    }
}
