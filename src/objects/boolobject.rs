use ffi;
use python::{Python, ToPythonPointer};
use err::PyResult;
use super::PyObject;
use conversion::{ExtractPyObject, ToPyObject};

/// Represents a Python `bool`.
pub struct PyBool<'p>(PyObject<'p>);

pyobject_newtype!(PyBool, PyBool_Check, PyBool_Type);

impl <'p> PyBool<'p> {
    /// Depending on `val`, returns `py.True()` or `py.False()`.
    #[inline]
    pub fn get(py: Python<'p>, val: bool) -> PyBool<'p> {
        if val { py.True() } else { py.False() }
    }

    /// Gets whether this boolean is `true`.
    #[inline]
    pub fn is_true(&self) -> bool {
        self.as_ptr() == unsafe { ::ffi::Py_True() }
    }
}

/// Converts a rust `bool` to a Python `bool`.
impl <'p> ToPyObject<'p> for bool {
    type ObjectType = PyBool<'p>;

    #[inline]
    fn to_py_object(&self, py: Python<'p>) -> PyBool<'p> {
        PyBool::get(py, *self)
    }

    #[inline]
    fn with_borrowed_ptr<F, R>(&self, _py: Python<'p>, f: F) -> R
        where F: FnOnce(*mut ffi::PyObject) -> R
    {
        // Avoid unnecessary Py_INCREF/Py_DECREF pair
        f(unsafe { if *self { ffi::Py_True() } else { ffi::Py_False() } })
    }
}

/// Converts a Python `bool` to a rust `bool`.
///
/// Fails with `TypeError` if the input is not a Python `bool`.
extract!(obj to bool => {
    Ok(try!(obj.cast_as::<PyBool>()).is_true())
});

