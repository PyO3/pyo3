use ffi;
use python::{Python, ToPythonPointer};
use err::PyResult;
use super::PyObject;
use conversion::{FromPyObject, ToPyObject};

pyobject_newtype!(PyBool, PyBool_Check, PyBool_Type);

impl <'p> PyBool<'p> {
    /// Depending on `val`, returns `py.True()` or `py.False()`.
    #[inline]
    pub fn get(py: Python<'p>, val: bool) -> PyBool<'p> {
        if val { py.True() } else { py.False() }
    }

    #[inline]
    pub fn is_true(&self) -> bool {
        self.as_ptr() == unsafe { ::ffi::Py_True() }
    }
}

impl <'p> ToPyObject<'p> for bool {
    type ObjectType = PyBool<'p>;

    #[inline]
    fn to_py_object(&self, py: Python<'p>) -> PyBool<'p> {
        PyBool::get(py, *self)
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, py: Python<'p>, f: F) -> R
        where F: FnOnce(*mut ffi::PyObject) -> R
    {
        // Avoid unnecessary Py_INCREF/Py_DECREF pair
        f(unsafe { if *self { ffi::Py_True() } else { ffi::Py_False() } })
    }
}

impl <'p, 'a> FromPyObject<'p, 'a> for bool {
    fn from_py_object(s: &'a PyObject<'p>) -> PyResult<'p, bool> {
        Ok(try!(s.clone().cast_into::<PyBool>()).is_true())
    }
}

