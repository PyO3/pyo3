// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::{
    ffi,
    objects::{FromPyObject, PyAny, PyNativeObject},
    types::Bool,
    AsPyPointer, IntoPy, PyObject, PyResult, Python, ToPyObject,
};

/// Represents a Python `bool`.
#[repr(transparent)]
pub struct PyBool<'py>(Bool, Python<'py>);

pyo3_native_object!(PyBool<'py>, Bool, 'py);

impl<'py> PyBool<'py> {
    /// Depending on `val`, returns `true` or `false`.
    #[inline]
    pub fn new(py: Python<'py>, val: bool) -> &'py PyBool<'py> {
        unsafe { Self::from_borrowed_ptr(py, if val { ffi::Py_True() } else { ffi::Py_False() }) }
    }

    /// Gets whether this boolean is `true`.
    #[inline]
    pub fn is_true(&self) -> bool {
        self.as_ptr() == unsafe { crate::ffi::Py_True() }
    }
}

/// Converts a Rust `bool` to a Python `bool`.
impl ToPyObject for bool {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyBool::new(py, *self).into()
    }
}

impl IntoPy<PyObject> for bool {
    #[inline]
    fn into_py(self, py: Python) -> PyObject {
        PyBool::new(py, self).into()
    }
}

/// Converts a Python `bool` to a Rust `bool`.
///
/// Fails with `TypeError` if the input is not a Python `bool`.
impl FromPyObject<'_, '_> for bool {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        Ok(obj.downcast::<PyBool>()?.is_true())
    }
}

#[cfg(test)]
mod test {
    use crate::objects::PyBool;
    use crate::Python;
    use crate::ToPyObject;

    #[test]
    fn test_true() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        assert!(PyBool::new(py, true).is_true());
        let t = PyBool::new(py, true);
        assert_eq!(true, t.extract().unwrap());
        assert_eq!(true.to_object(py), PyBool::new(py, true).into());
    }

    #[test]
    fn test_false() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        assert!(!PyBool::new(py, false).is_true());
        let t = PyBool::new(py, false);
        assert_eq!(false, t.extract().unwrap());
        assert_eq!(false.to_object(py), PyBool::new(py, false).into());
    }
}
