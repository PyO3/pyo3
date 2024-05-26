use crate::{
    ffi, ffi_ptr_ext::FfiPtrExt, types::any::PyAnyMethods, Borrowed, Bound, PyAny, PyTypeInfo,
    Python,
};

/// Represents the Python `Ellipsis` object.
#[repr(transparent)]
pub struct PyEllipsis(PyAny);

pyobject_native_type_named!(PyEllipsis);
pyobject_native_type_extract!(PyEllipsis);

impl PyEllipsis {
    /// Returns the `Ellipsis` object.
    #[cfg(feature = "gil-refs")]
    #[deprecated(
        since = "0.21.0",
        note = "`PyEllipsis::get` will be replaced by `PyEllipsis::get_bound` in a future PyO3 version"
    )]
    #[inline]
    pub fn get(py: Python<'_>) -> &PyEllipsis {
        Self::get_bound(py).into_gil_ref()
    }

    /// Returns the `Ellipsis` object.
    #[inline]
    pub fn get_bound(py: Python<'_>) -> Borrowed<'_, '_, PyEllipsis> {
        unsafe { ffi::Py_Ellipsis().assume_borrowed(py).downcast_unchecked() }
    }
}

unsafe impl PyTypeInfo for PyEllipsis {
    const NAME: &'static str = "ellipsis";

    const MODULE: Option<&'static str> = None;

    fn type_object_raw(_py: Python<'_>) -> *mut ffi::PyTypeObject {
        unsafe { ffi::Py_TYPE(ffi::Py_Ellipsis()) }
    }

    #[inline]
    fn is_type_of_bound(object: &Bound<'_, PyAny>) -> bool {
        // ellipsis is not usable as a base type
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
    use crate::types::{PyDict, PyEllipsis};
    use crate::{PyTypeInfo, Python};

    #[test]
    fn test_ellipsis_is_itself() {
        Python::with_gil(|py| {
            assert!(PyEllipsis::get_bound(py).is_instance_of::<PyEllipsis>());
            assert!(PyEllipsis::get_bound(py).is_exact_instance_of::<PyEllipsis>());
        })
    }

    #[test]
    fn test_ellipsis_type_object_consistent() {
        Python::with_gil(|py| {
            assert!(PyEllipsis::get_bound(py)
                .get_type()
                .is(&PyEllipsis::type_object_bound(py)));
        })
    }

    #[test]
    fn test_dict_is_not_ellipsis() {
        Python::with_gil(|py| {
            assert!(PyDict::new_bound(py).downcast::<PyEllipsis>().is_err());
        })
    }
}
