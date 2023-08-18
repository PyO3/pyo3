use crate::{ffi, PyAny, PyDowncastError, PyTryFrom, Python};

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

impl<'v> PyTryFrom<'v> for PyEllipsis {
    fn try_from<V: Into<&'v PyAny>>(value: V) -> Result<&'v Self, crate::PyDowncastError<'v>> {
        let value: &PyAny = value.into();
        if unsafe { ffi::Py_Ellipsis() == value.as_ptr() } {
            return unsafe { Ok(value.downcast_unchecked()) };
        }
        Err(PyDowncastError::new(value, "ellipsis"))
    }

    fn try_from_exact<V: Into<&'v PyAny>>(
        value: V,
    ) -> Result<&'v Self, crate::PyDowncastError<'v>> {
        value.into().downcast()
    }

    unsafe fn try_from_unchecked<V: Into<&'v PyAny>>(value: V) -> &'v Self {
        let ptr = value.into() as *const _ as *const PyEllipsis;
        &*ptr
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{PyDict, PyEllipsis};
    use crate::Python;

    #[test]
    fn test_ellipsis_is_itself() {
        Python::with_gil(|py| {
            assert!(PyEllipsis::get(py)
                .downcast_exact::<PyEllipsis>()
                .unwrap()
                .is_ellipsis());
        })
    }

    #[test]
    fn test_dict_is_not_ellipsis() {
        Python::with_gil(|py| {
            assert!(PyDict::new(py).downcast::<PyEllipsis>().is_err());
        })
    }
}
