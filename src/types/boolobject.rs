// Copyright (c) 2017-present PyO3 Project and Contributors
use crate::{
    ffi, AsPyPointer, FromPy, FromPyObject, IntoPy, Py, PyObject, PyResult, Python, ToPyObject,
};

/// Represents a Python `bool`.
#[repr(transparent)]
pub struct PyBool(PyObject);

pyobject_native_type!(PyBool, ffi::PyObject, ffi::PyBool_Type, ffi::PyBool_Check);

impl PyBool {
    /// Depending on `val`, returns `true` or `false`.
    #[inline]
    pub fn new(py: Python, val: bool) -> &PyBool {
        unsafe { py.from_borrowed_ptr(if val { ffi::Py_True() } else { ffi::Py_False() }) }
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
    fn to_object<'p>(&self, py: Python<'p>) -> &'p PyObject {
        unsafe {
            py.from_borrowed_ptr(if *self {
                ffi::Py_True()
            } else {
                ffi::Py_False()
            })
        }
    }
}

impl FromPy<bool> for Py<PyObject> {
    #[inline]
    fn from_py(other: bool, py: Python) -> Self {
        PyBool::new(py, other).into_py(py)
    }
}

/// Converts a Python `bool` to a Rust `bool`.
///
/// Fails with `TypeError` if the input is not a Python `bool`.
impl<'source> FromPyObject<'source> for bool {
    fn extract(obj: &'source PyObject) -> PyResult<Self> {
        Ok(obj.downcast::<PyBool>()?.is_true())
    }
}

#[cfg(test)]
mod test {
    use crate::types::{PyBool, PyObject};
    use crate::Python;
    use crate::ToPyObject;

    #[test]
    fn test_true() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        assert!(PyBool::new(py, true).is_true());
        let t: &PyObject = PyBool::new(py, true).into();
        assert_eq!(true, t.extract::<bool>().unwrap());
        assert_eq!(true.to_object(py), PyBool::new(py, true).as_ref());
    }

    #[test]
    fn test_false() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        assert!(!PyBool::new(py, false).is_true());
        let t: &PyObject = PyBool::new(py, false).into();
        assert_eq!(false, t.extract().unwrap());
        assert_eq!(false.to_object(py), PyBool::new(py, false).as_ref());
    }
}
