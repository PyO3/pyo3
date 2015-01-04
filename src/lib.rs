#![feature(unsafe_destructor)]
#![allow(unused_imports, dead_code, unused_variables)]
#![feature(associated_types)]

extern crate libc;
extern crate "libpython27-sys" as ffi;
pub use ffi::Py_ssize_t;
pub use err::{PyErr, PyResult};
pub use python::Python;
pub use pythonrun::PythonInterpreter;
pub use object::{PythonObject, PyObject, ObjectProtocol};
pub use typeobject::PyType;
pub use pyptr::PyPtr;
pub use module::PyModule;
pub use conversion::{FromPyObject, ToPyObject};

// Fundamentals:
mod python;
mod object;
mod pyptr;
mod err;

// Object Types:
mod typeobject;
mod module;

mod pythonrun;
mod conversion;

#[test]
fn it_works() {
    let interpreter = unsafe { PythonInterpreter::new() };
    let py = interpreter.python();
    let sys = PyModule::import(py, "sys").unwrap();
    let path = sys.as_object().getattr("path").unwrap();
    println!("{0}", path);
    panic!()
}

