#![feature(unsafe_destructor)]
#![allow(unused_imports, dead_code, unused_variables)]

extern crate libc;
extern crate core;
extern crate "python27-sys" as ffi;
pub use ffi::Py_ssize_t;
pub use err::{PyErr, PyResult};
pub use objects::*;
pub use python::{Python, PythonObject, PythonObjectWithCheckedDowncast, PythonObjectWithTypeObject};
pub use conversion::{FromPyObject, ToPyObject};
pub use objectprotocol::{ObjectProtocol};
pub use cstr::CStr;

mod cstr;
mod python;
mod err;
mod conversion;
mod objects;
mod objectprotocol;
mod pythonrun;

#[test]
fn it_works() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let sys = PyModule::import(py, "sys").unwrap();
    let path = sys.as_object().getattr("path").unwrap();
    println!("{0}", path);
}

