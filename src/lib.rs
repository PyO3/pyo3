#![feature(core)] // used for a lot of low-level stuff
#![feature(unsafe_no_drop_flag)] // crucial so that PyObject<'p> is binary compatible with *mut ffi::PyObject
#![feature(filling_drop)] // necessary to avoid segfault with unsafe_no_drop_flag
#![feature(optin_builtin_traits)] // for opting out of Sync/Send
#![feature(slice_patterns)] // for tuple_conversion macros
#![feature(utf8_error)] // for translating Utf8Error to python exception
#![allow(unused_imports, dead_code, unused_variables)]

extern crate libc;
extern crate python27_sys as ffi;
pub use ffi::Py_ssize_t;
pub use err::{PyErr, PyResult};
pub use objects::*;
pub use python::{Python, PythonObject, PythonObjectWithCheckedDowncast, PythonObjectWithTypeObject};
pub use conversion::{FromPyObject, ToPyObject};
pub use objectprotocol::{ObjectProtocol};
pub use std::ffi::CStr;

#[macro_export]
macro_rules! cstr(
    ($s: tt) => (
        // TODO: verify that $s is a string literal without nuls
        unsafe {
            ::std::ffi::CStr::from_ptr(concat!($s, "\0").as_ptr() as *const _)
        }
    );
);

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
            match $crate::PyModule::_init(py, cstr!($name), $init) {
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

