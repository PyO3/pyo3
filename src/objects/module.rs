use std;
use ffi;
use python::Python;
use objects::{PyObject, PyType};
use err::{self, PyResult};

pyobject_newtype!(PyModule, PyModule_Check, PyModule_Type);


impl <'p> PyModule<'p> {
    pub fn import<N>(py : Python<'p>, name : N) -> PyResult<'p, PyModule<'p>> where N: std::c_str::ToCStr {
        let result = name.with_c_str(|name| unsafe {
            err::result_from_owned_ptr(py, ffi::PyImport_ImportModule(name))
        });
        Ok(try!(try!(result).cast_into()))
    }
}

