#![feature(unsafe_destructor)]
#![allow(unused_imports, dead_code, unused_variables)]
#![feature(associated_types)]
#![feature(globs)]
#![feature(slicing_syntax)]

extern crate libc;
extern crate "python27-sys" as ffi;
pub use ffi::Py_ssize_t;
pub use err::{PyErr, PyResult};
pub use python::{Python, PythonObject, PythonObjectDowncast};
pub use object::PyObject;
pub use typeobject::PyType;
pub use pyptr::PyPtr;
pub use module::PyModule;
pub use conversion::{FromPyObject, ToPyObject};
pub use objectprotocol::{ObjectProtocol};

// Fundamentals:
mod python;
mod pyptr;
mod err;
mod conversion;

// Object Types:
mod object;
mod typeobject;
mod module;

// Python APIs:
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

