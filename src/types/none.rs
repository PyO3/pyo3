use crate::ffi_ptr_ext::FfiPtrExt;
use crate::{ffi, types::any::PyAnyMethods, Borrowed, Bound, PyAny, PyObject, PyTypeInfo, Python};
#[allow(deprecated)]
use crate::{IntoPy, ToPyObject};

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
        unsafe { ffi::Py_None().assume_borrowed(py).downcast_unchecked() }
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

/// `()` is converted to Python `None`.
#[allow(deprecated)]
impl ToPyObject for () {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        PyNone::get(py).into_py(py)
    }
}

#[allow(deprecated)]
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
    use crate::{PyObject, PyTypeInfo, Python};

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
            assert!(PyNone::get(py).get_type().is(&PyNone::type_object(py)));
        })
    }

    #[test]
    fn test_none_is_none() {
        Python::with_gil(|py| {
            assert!(PyNone::get(py).downcast::<PyNone>().unwrap().is_none());
        })
    }

    #[test]
    #[allow(deprecated)]
    fn test_unit_to_object_is_none() {
        use crate::ToPyObject;
        Python::with_gil(|py| {
            assert!(().to_object(py).downcast_bound::<PyNone>(py).is_ok());
        })
    }

    #[test]
    #[allow(deprecated)]
    fn test_unit_into_py_is_none() {
        use crate::IntoPy;
        Python::with_gil(|py| {
            let obj: PyObject = ().into_py(py);
            assert!(obj.downcast_bound::<PyNone>(py).is_ok());
        })
    }

    #[test]
    fn test_dict_is_not_none() {
        Python::with_gil(|py| {
            assert!(PyDict::new(py).downcast::<PyNone>().is_err());
        })
    }
}
