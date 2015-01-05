use std;
use ffi;
use python::{Python, PythonObject};
use objects::PyObject;
use pyptr::PyPtr;
use err::{self, PyResult};

pyobject_newtype!(PyDict, PyDict_Check, PyDict_Type);

impl <'p> PyDict<'p> {
    fn new(py: Python<'p>) -> PyResult<'p, PyPtr<'p, PyDict<'p>>> {
        unimplemented!()
    }
}

