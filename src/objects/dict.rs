use std;
use ffi;
use python::{Python, PythonObject};
use objects::PyObject;
use err::{self, PyResult};

pyobject_newtype!(PyDict, PyDict_Check, PyDict_Type);

impl <'p> PyDict<'p> {
    fn new(py: Python<'p>) -> PyDict<'p> {
        unimplemented!()
    }
}

