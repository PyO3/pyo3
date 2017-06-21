use ffi;
use pointers::PyPtr;
use python::{ToPyPointer, Python};
use objects::PyObject;
use conversion::{ToPyObject, IntoPyObject};

/// Represents a Python `bool`.
pub struct PyBool(PyPtr);

pyobject_nativetype2!(PyBool, PyBool_Type, PyBool_Check);


impl PyBool {
    /// Depending on `val`, returns `py.True()` or `py.False()`.
    #[inline]
    pub fn new<'p>(py: Python<'p>, val: bool) -> &'p PyBool {
        unsafe {
            py.unchecked_cast_from_ptr(if val { ffi::Py_True() } else { ffi::Py_False() })
        }
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
    fn to_object(&self, py: Python) -> PyObject {
        unsafe {
            PyObject::from_borrowed_ptr(
                py,
                if *self { ffi::Py_True() } else { ffi::Py_False() })
        }
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, _py: Python, f: F) -> R
        where F: FnOnce(*mut ffi::PyObject) -> R
    {
        // Avoid unnecessary Py_INCREF/Py_DECREF pair
        f(unsafe { if *self { ffi::Py_True() } else { ffi::Py_False() } })
    }
}

impl IntoPyObject for bool {
    #[inline]
    fn into_object(self, py: Python) -> PyObject {
        PyBool::new(py, self).into()
    }
}

/// Converts a Python `bool` to a rust `bool`.
///
/// Fails with `TypeError` if the input is not a Python `bool`.
pyobject_extract!(py, obj to bool => {
    Ok(try!(obj.cast_as::<PyBool>(py)).is_true())
});


#[cfg(test)]
mod test {
    use python::Python;
    use objects::PyObject;
    use conversion::ToPyObject;

    #[test]
    fn test_true() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        assert!(py.True().is_true());
        let t: PyObject = py.True().into();
        assert_eq!(true, t.extract(py).unwrap());
        assert!(true.to_object(py) == py.True().into());
    }

    #[test]
    fn test_false() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        assert!(!py.False().is_true());
        let t: PyObject = py.False().into();
        assert_eq!(false, t.extract(py).unwrap());
        assert!(false.to_object(py) == py.False().into());
    }
}
