use std;
use ffi;
use python::{Python, PythonObject};
use objects::PyObject;
use err::{self, PyResult};

pyobject_newtype!(PyDict, PyDict_Check, PyDict_Type);

impl <'p> PyDict<'p> {
    /// Creates a new empty dictionary.
    ///
    /// # Panic
    /// May panic when running out of memory.
    pub fn new(py: Python<'p>) -> PyDict<'p> {
        unsafe {
            err::cast_from_owned_ptr_or_panic(py, ffi::PyDict_New())
        }
    }
}

