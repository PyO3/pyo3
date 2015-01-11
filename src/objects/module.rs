use std;
use ffi;
use python::Python;
use objects::{PyObject, PyType};
use err::{self, PyResult};
use cstr::CStr;

pyobject_newtype!(PyModule, PyModule_Check, PyModule_Type);

impl <'p> PyModule<'p> {
    /// Import the python module with the specified name.
    pub fn import(py : Python<'p>, name : &CStr) -> PyResult<'p, PyModule<'p>> {
        let result = try!(unsafe {
            err::result_from_owned_ptr(py, ffi::PyImport_ImportModule(name.as_ptr()))
        });
        Ok(try!(result.cast_into()))
    }
}


