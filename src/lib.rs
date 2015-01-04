#![feature(unsafe_destructor)]
#![allow(unused_imports, dead_code, unused_variables)]
#![feature(associated_types)]
#![feature(globs)]
#![feature(slicing_syntax)]

extern crate libc;
extern crate "libpython27-sys" as ffi;
pub use ffi::Py_ssize_t;
pub use err::{PyErr, PyResult};
pub use python::Python;
pub use object::{PythonObject, PyObject};
pub use typeobject::PyType;
pub use pyptr::PyPtr;
pub use module::PyModule;
pub use conversion::{FromPyObject, ToPyObject};
pub use objectprotocol::{ObjectProtocol};

// Fundamentals:
mod python;
mod object;
mod pyptr;
mod err;

// Object Types:
mod typeobject;
mod module;

mod objectprotocol;
mod pythonrun;
mod conversion;

#[test]
fn it_works() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let sys = PyModule::import(py, "sys").unwrap();
    let path = sys.as_object().getattr("path").unwrap();
    println!("{0}", path);
}

