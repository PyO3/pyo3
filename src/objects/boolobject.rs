use ::{PyPtr, PyObject};
use ffi;
use python::{PythonToken, ToPythonPointer, Python};
use conversion::{ToPyObject};

/// Represents a Python `bool`.
pub struct PyBool(PythonToken<PyBool>);

pyobject_newtype!(PyBool, PyBool_Check, PyBool_Type);

impl PyBool {
    /// Depending on `val`, returns `py.True()` or `py.False()`.
    #[inline]
    pub fn get(py: Python, val: bool) -> PyPtr<PyBool> {
        if val { py.True() } else { py.False() }
    }

    /// Gets whether this boolean is `true`.
    #[inline]
    pub fn is_true(&self) -> bool {
        self.as_ptr() == unsafe { ::ffi::Py_True() }
    }
}

/// Converts a rust `bool` to a Python `bool`.
impl ToPyObject for bool {
    #[inline]
    fn to_object(&self, py: Python) -> PyPtr<PyObject> {
        PyBool::get(py, *self).into_object()
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, _py: Python, f: F) -> R
        where F: FnOnce(*mut ffi::PyObject) -> R
    {
        // Avoid unnecessary Py_INCREF/Py_DECREF pair
        f(unsafe { if *self { ffi::Py_True() } else { ffi::Py_False() } })
    }
}

/// Converts a Python `bool` to a rust `bool`.
///
/// Fails with `TypeError` if the input is not a Python `bool`.
pyobject_extract!(obj to bool => {
    Ok(try!(obj.cast_as::<PyBool>()).is_true())
});


#[cfg(test)]
mod test {
    use python::{Python, PythonObject};
    use conversion::ToPyObject;

    #[test]
    fn test_true() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        assert!(py.True().is_true());
        assert_eq!(true, py.True().as_object().extract(py).unwrap());
        assert!(true.to_py_object(py).as_object() == py.True().as_object());
    }

    #[test]
    fn test_false() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        assert!(!py.False().is_true());
        assert_eq!(false, py.False().as_object().extract(py).unwrap());
        assert!(false.to_py_object(py).as_object() == py.False().as_object());
    }
}
