#![allow(unstable)]
#![feature(unsafe_destructor)]
#![feature(optin_builtin_traits)]
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

#[macro_use]
mod cstr;
mod python;
mod err;
mod conversion;
mod objects;
mod objectprotocol;
mod pythonrun;

#[macro_export]
macro_rules! py_module_initializer {
    ($name: tt, $init_funcname: ident, $init: expr) => {
        #[no_mangle]
        pub extern "C" fn $init_funcname() {
            let py = unsafe { $crate::Python::assume_gil_acquired() };
            match $crate::PyModule::init(py, cstr!($name), $init) {
                Ok(()) => (),
                Err(e) => e.restore()
            }
        }
    }
}

#[test]
fn it_works() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let sys = PyModule::import(py, cstr!("sys")).unwrap();
    let path = sys.as_object().getattr("path").unwrap();
    println!("{0}", path);
}

