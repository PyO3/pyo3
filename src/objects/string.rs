use std;
use ffi;
use python::{Python, PythonObject};
use objects::PyObject;
use err::{self, PyResult};

pyobject_newtype!(PyBytes, PyBytes_Check, PyBytes_Type);

pyobject_newtype!(PyString, PyString_Check, PyString_Type);

pyobject_newtype!(PyUnicode, PyUnicode_Check, PyUnicode_Type);

