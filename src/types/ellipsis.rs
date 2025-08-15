use crate::{
    ffi, ffi_ptr_ext::FfiPtrExt, types::any::PyAnyMethods, Borrowed, Bound, PyAny, PyTypeInfo,
    Python,
};

/// Represents the Python `Ellipsis` object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyEllipsis>`][crate::Py] or [`Bound<'py, PyEllipsis>`][Bound].
#[repr(transparent)]
pub struct PyEllipsis(PyAny);

pyobject_native_type_named!(PyEllipsis);

impl PyEllipsis {
    /// Returns the `Ellipsis` object.
    #[inline]
    pub fn get(py: Python<'_>) -> Borrowed<'_, '_, PyEllipsis> {
        unsafe { ffi::Py_Ellipsis().assume_borrowed(py).cast_unchecked() }
    }
}

unsafe impl PyTypeInfo for PyEllipsis {
    const NAME: &'static str = "ellipsis";

    const MODULE: Option<&'static str> = None;

    fn type_object_raw(_py: Python<'_>) -> *mut ffi::PyTypeObject {
        unsafe { ffi::Py_TYPE(ffi::Py_Ellipsis()) }
    }

    #[inline]
    fn is_type_of(object: &Bound<'_, PyAny>) -> bool {
        // ellipsis is not usable as a base type
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
    use crate::types::{PyDict, PyEllipsis};
    use crate::{PyTypeInfo, Python};

    #[test]
    fn test_ellipsis_is_itself() {
        Python::attach(|py| {
            assert!(PyEllipsis::get(py).is_instance_of::<PyEllipsis>());
            assert!(PyEllipsis::get(py).is_exact_instance_of::<PyEllipsis>());
        })
    }

    #[test]
    fn test_ellipsis_type_object_consistent() {
        Python::attach(|py| {
            assert!(PyEllipsis::get(py)
                .get_type()
                .is(PyEllipsis::type_object(py)));
        })
    }

    #[test]
    fn test_dict_is_not_ellipsis() {
        Python::attach(|py| {
            assert!(PyDict::new(py).cast::<PyEllipsis>().is_err());
        })
    }
}
