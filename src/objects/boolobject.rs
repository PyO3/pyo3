use ffi;
use pointers::{Ptr, PyPtr};
use python::{ToPyPointer, Python};
use objects::PyObject;
use conversion::ToPyObject;

/// Represents a Python `bool`.
pub struct PyBool<'p>(Ptr<'p>);
pub struct PyBoolPtr(PyPtr);

pyobject_convert!(PyBool);
pyobject_nativetype!(PyBool, PyBool_Check, PyBool_Type, PyBoolPtr);


impl<'p> PyBool<'p> {
    /// Depending on `val`, returns `py.True()` or `py.False()`.
    #[inline]
    pub fn new(py: Python<'p>, val: bool) -> PyBool<'p> {
        unsafe { PyBool(
            Ptr::from_borrowed_ptr(py, if val { ffi::Py_True() } else { ffi::Py_False() })
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
    fn to_object<'p>(&self, py: Python<'p>) -> PyObject<'p> {
        PyBool::new(py, *self).into()
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
    use objects::PyObject;
    use conversion::ToPyObject;

    #[test]
    fn test_true() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        assert!(py.True().is_true());
        let t: PyObject = py.True().into();
        assert_eq!(true, t.extract().unwrap());
        assert!(true.to_object(py) == py.True().into());
    }

    #[test]
    fn test_false() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        assert!(!py.False().is_true());
        let t: PyObject = py.False().into();
        assert_eq!(false, t.extract().unwrap());
        assert!(false.to_object(py) == py.False().into());
    }
}
