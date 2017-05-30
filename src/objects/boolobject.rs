use ::{pptr, PyPtr};
use ffi;
use token::PyObjectMarker;
use python::{ToPythonPointer, Python};
use native::PyNativeObject;
use conversion::ToPyObject;

/// Represents a Python `bool`.
pub struct PyBool<'p>(pptr<'p>);

pyobject_nativetype!(PyBool, PyBool_Check, PyBool_Type);

impl<'p> PyBool<'p> {
    /// Depending on `val`, returns `py.True()` or `py.False()`.
    #[inline]
    pub fn new(py: Python<'p>, val: bool) -> PyBool<'p> {
        unsafe { PyBool(
            pptr::from_borrowed_ptr(py, if val { ffi::Py_True() } else { ffi::Py_False() })
        )}
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
    fn to_object(&self, py: Python) -> PyPtr<PyObjectMarker> {
        PyBool::new(py, *self).into_object()
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
    use python::{Python};
    use conversion::ToPyObject;
    use ::PyNativeObject;

    #[test]
    fn test_true() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        assert!(py.True().is_true());
        assert_eq!(true, py.True().as_object().extract().unwrap());
        assert!(true.to_object(py).into_object(py) == py.True().as_object());
    }

    #[test]
    fn test_false() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        assert!(!py.False().is_true());
        assert_eq!(false, py.False().as_object().extract().unwrap());
        assert!(false.to_object(py).into_object(py) == py.False().as_object());
    }
}
