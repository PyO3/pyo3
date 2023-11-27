use crate::{ffi, PyAny, PyTypeInfo, Python};

/// Represents the Python `Ellipsis` object.
#[repr(transparent)]
pub struct PyEllipsis(PyAny);

pyobject_native_type_named!(PyEllipsis);
pyobject_native_type_extract!(PyEllipsis);

impl PyEllipsis {
    /// Returns the `Ellipsis` object.
    #[inline]
    pub fn get(py: Python<'_>) -> &PyEllipsis {
        unsafe { py.from_borrowed_ptr(ffi::Py_Ellipsis()) }
    }
}

unsafe impl PyTypeInfo for PyEllipsis {
    const NAME: &'static str = "ellipsis";

    const MODULE: Option<&'static str> = None;

    fn type_object_raw(_py: Python<'_>) -> *mut ffi::PyTypeObject {
        unsafe { ffi::Py_TYPE(ffi::Py_Ellipsis()) }
    }

    #[inline]
    fn is_type_of(object: &PyAny) -> bool {
        // ellipsis is not usable as a base type
        Self::is_exact_type_of(object)
    }

    #[inline]
    fn is_exact_type_of(object: &PyAny) -> bool {
        object.is(Self::get(object.py()))
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{PyDict, PyEllipsis};
    use crate::{PyTypeInfo, Python};

    #[test]
    fn test_ellipsis_is_itself() {
        Python::with_gil(|py| {
            assert!(PyEllipsis::get(py).is_instance_of::<PyEllipsis>());
            assert!(PyEllipsis::get(py).is_exact_instance_of::<PyEllipsis>());
        })
    }

    #[test]
    fn test_ellipsis_type_object_consistent() {
        Python::with_gil(|py| {
            assert!(PyEllipsis::get(py)
                .get_type()
                .is(PyEllipsis::type_object(py)));
        })
    }

    #[test]
    fn test_dict_is_not_ellipsis() {
        Python::with_gil(|py| {
            assert!(PyDict::new(py).downcast::<PyEllipsis>().is_err());
        })
    }
}
