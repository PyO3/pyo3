use crate::ffi_ptr_ext::FfiPtrExt;
use crate::{ffi, Borrowed, IntoPy, PyAny, PyObject, PyTypeInfo, Python, ToPyObject};

/// Represents the Python `None` object.
#[repr(transparent)]
pub struct PyNone(PyAny);

pyobject_native_type_named!(PyNone);
pyobject_native_type_extract!(PyNone);

impl PyNone {
    /// Returns the `None` object.
    #[inline]
    pub fn get(py: Python<'_>) -> Borrowed<'_, '_, PyNone> {
        unsafe {
            let bound = ffi::Py_None().assume_borrowed(py);
            std::mem::transmute(bound)
        }
    }
}

unsafe impl PyTypeInfo for PyNone {
    const NAME: &'static str = "NoneType";

    const MODULE: Option<&'static str> = None;

    fn type_object_raw(_py: Python<'_>) -> *mut ffi::PyTypeObject {
        unsafe { ffi::Py_TYPE(ffi::Py_None()) }
    }

    #[inline]
    fn is_type_of(object: &PyAny) -> bool {
        // NoneType is not usable as a base type
        Self::is_exact_type_of(object)
    }

    #[inline]
    fn is_exact_type_of(object: &PyAny) -> bool {
        let none = Self::get(object.py());
        object.is(none.as_ref())
    }
}

/// `()` is converted to Python `None`.
impl ToPyObject for () {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        PyNone::get(py).into_py(py)
    }
}

impl IntoPy<PyObject> for () {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        PyNone::get(py).into_py(py)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::any::PyAnyMethods;
    use crate::types::{PyDict, PyNone};
    use crate::{IntoPy, PyObject, PyTypeInfo, Python, ToPyObject};
    #[test]
    fn test_none_is_itself() {
        Python::with_gil(|py| {
            assert!(PyNone::get(py).is_instance_of::<PyNone>());
            assert!(PyNone::get(py).is_exact_instance_of::<PyNone>());
        })
    }

    #[test]
    fn test_none_type_object_consistent() {
        Python::with_gil(|py| {
            assert!(PyNone::get(py).get_type().is(PyNone::type_object(py)));
        })
    }

    #[test]
    fn test_none_is_none() {
        Python::with_gil(|py| {
            assert!(PyNone::get(py).downcast::<PyNone>().unwrap().is_none());
        })
    }

    #[test]
    fn test_unit_to_object_is_none() {
        Python::with_gil(|py| {
            assert!(().to_object(py).downcast::<PyNone>(py).is_ok());
        })
    }

    #[test]
    fn test_unit_into_py_is_none() {
        Python::with_gil(|py| {
            let obj: PyObject = ().into_py(py);
            assert!(obj.downcast::<PyNone>(py).is_ok());
        })
    }

    #[test]
    fn test_dict_is_not_none() {
        Python::with_gil(|py| {
            assert!(PyDict::new(py).downcast::<PyNone>().is_err());
        })
    }
}
