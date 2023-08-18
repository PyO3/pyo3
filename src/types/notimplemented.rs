use crate::{ffi, PyAny, PyDowncastError, PyTryFrom, Python};

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

impl<'v> PyTryFrom<'v> for PyNotImplemented {
    fn try_from<V: Into<&'v PyAny>>(value: V) -> Result<&'v Self, crate::PyDowncastError<'v>> {
        let value: &PyAny = value.into();
        if unsafe { ffi::Py_NotImplemented() == value.as_ptr() } {
            return unsafe { Ok(value.downcast_unchecked()) };
        }
        Err(PyDowncastError::new(value, "NotImplementedType"))
    }

    fn try_from_exact<V: Into<&'v PyAny>>(
        value: V,
    ) -> Result<&'v Self, crate::PyDowncastError<'v>> {
        value.into().downcast()
    }

    unsafe fn try_from_unchecked<V: Into<&'v PyAny>>(value: V) -> &'v Self {
        let ptr = value.into() as *const _ as *const PyNotImplemented;
        &*ptr
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{PyDict, PyNotImplemented};
    use crate::Python;

    #[test]
    fn test_notimplemented_is_itself() {
        Python::with_gil(|py| {
            assert!(PyNotImplemented::get(py)
                .downcast_exact::<PyNotImplemented>()
                .unwrap()
                .is(&py.NotImplemented()));
        })
    }

    #[test]
    fn test_dict_is_not_notimplemented() {
        Python::with_gil(|py| {
            assert!(PyDict::new(py).downcast::<PyNotImplemented>().is_err());
        })
    }
}
