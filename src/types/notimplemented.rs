use crate::{ffi, PyAny, PyTypeInfo, Python};

/// Represents the Python `NotImplemented` object.
#[repr(transparent)]
pub struct PyNotImplemented(PyAny);

pyobject_native_type_named!(PyNotImplemented);
pyobject_native_type_extract!(PyNotImplemented);

impl PyNotImplemented {
    /// Returns the `NotImplemented` object.
    #[inline]
    pub fn get(py: Python<'_>) -> &PyNotImplemented {
        unsafe { py.from_borrowed_ptr(ffi::Py_NotImplemented()) }
    }
}

unsafe impl PyTypeInfo for PyNotImplemented {
    const NAME: &'static str = "NotImplementedType";
    const MODULE: Option<&'static str> = None;

    fn type_object_raw(_py: Python<'_>) -> *mut ffi::PyTypeObject {
        unsafe { ffi::Py_TYPE(ffi::Py_NotImplemented()) }
    }

    #[inline]
    fn is_type_of(object: &PyAny) -> bool {
        // NotImplementedType is not usable as a base type
        Self::is_exact_type_of(object)
    }

    #[inline]
    fn is_exact_type_of(object: &PyAny) -> bool {
        object.is(Self::get(object.py()))
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{PyDict, PyNotImplemented};
    use crate::{PyTypeInfo, Python};

    #[test]
    fn test_notimplemented_is_itself() {
        Python::with_gil(|py| {
            assert!(PyNotImplemented::get(py).is_instance_of::<PyNotImplemented>());
            assert!(PyNotImplemented::get(py).is_exact_instance_of::<PyNotImplemented>());
        })
    }

    #[test]
    fn test_notimplemented_type_object_consistent() {
        Python::with_gil(|py| {
            assert!(PyNotImplemented::get(py)
                .get_type()
                .is(PyNotImplemented::type_object(py)));
        })
    }

    #[test]
    fn test_dict_is_not_notimplemented() {
        Python::with_gil(|py| {
            assert!(PyDict::new(py).downcast::<PyNotImplemented>().is_err());
        })
    }
}
