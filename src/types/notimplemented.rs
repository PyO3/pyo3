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

impl PyNotImplemented {
    /// Returns the `NotImplemented` object.
    #[inline]
    pub fn get(py: Python<'_>) -> Borrowed<'_, '_, PyNotImplemented> {
        unsafe {
            ffi::Py_NotImplemented()
                .assume_borrowed(py)
                .cast_unchecked()
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
    fn is_type_of(object: &Bound<'_, PyAny>) -> bool {
        // NotImplementedType is not usable as a base type
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
    use crate::types::{PyDict, PyNotImplemented};
    use crate::{PyTypeInfo, Python};

    #[test]
    fn test_notimplemented_is_itself() {
        Python::attach(|py| {
            assert!(PyNotImplemented::get(py).is_instance_of::<PyNotImplemented>());
            assert!(PyNotImplemented::get(py).is_exact_instance_of::<PyNotImplemented>());
        })
    }

    #[test]
    fn test_notimplemented_type_object_consistent() {
        Python::attach(|py| {
            assert!(PyNotImplemented::get(py)
                .get_type()
                .is(PyNotImplemented::type_object(py)));
        })
    }

    #[test]
    fn test_dict_is_not_notimplemented() {
        Python::attach(|py| {
            assert!(PyDict::new(py).cast::<PyNotImplemented>().is_err());
        })
    }
}
