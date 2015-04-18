#![feature(core)] // used for a lot of low-level stuff
#![feature(unsafe_no_drop_flag)] // crucial so that PyObject<'p> is binary compatible with *mut ffi::PyObject
#![feature(filling_drop)] // necessary to avoid segfault with unsafe_no_drop_flag
#![feature(optin_builtin_traits)] // for opting out of Sync/Send
#![feature(slice_patterns)] // for tuple_conversion macros
#![feature(utf8_error)] // for translating Utf8Error to python exception
#![allow(unused_imports, dead_code, unused_variables)]

//! Rust bindings to the python interpreter.
//!
//! # Example
//! ```
//! #[macro_use] extern crate cpython;
//!
//! use cpython::{PythonObject, ObjectProtocol, PyModule, Python};
//! 
//! fn main() {
//!     let gil = Python::acquire_gil();
//!     let py = gil.python();
//!     let sys = py.import("sys").unwrap();
//!     let version: String = sys.get("version").unwrap().extract().unwrap();
//!     println!("Hello Python {}", version);
//! }
//! ```

extern crate libc;
extern crate python27_sys as ffi;
pub use ffi::Py_ssize_t;
pub use err::{PyErr, PyResult};
pub use objects::*;
pub use python::{Python, PythonObject, PythonObjectWithCheckedDowncast, PythonObjectWithTypeObject, ToPythonPointer};
pub use pythonrun::{GILGuard, prepare_freethreaded_python};
pub use conversion::{FromPyObject, ToPyObject};
pub use objectprotocol::{ObjectProtocol};

/// Constructs a `&'static CStr` literal.
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

/// Private re-exports for macros. Do not use.
pub mod _detail {
    pub use ffi;
    pub use libc;
    pub use err::from_owned_ptr_or_panic;
}

/// Expands to an `extern "C"` function that allows python to load
/// the rust code as a python extension module.
///
/// The macro takes three arguments:
///
/// 1. The module name as a string literal.
/// 2. The name of the init function as an identifier.
///    The function must be named `init$module_name` so that python 2.7 can load the module.
///    Note: this parameter will be removed in a future version
///    (once Rust supports `concat_ident!` as function name).
/// 3. A function or lambda of type `Fn(Python<'p>, &PyModule<'p>) -> PyResult<'p, ()>`.
///    This function will be called when the module is imported, and is responsible
///    for adding the module's members.
///
/// # Example
/// ```
/// #![crate_type = "dylib"]
/// #[macro_use] extern crate cpython;
/// use cpython::{Python, PyResult, PyObject, PyTuple};
///
/// py_module_initializer!("example", initexample, |py, m| {
///     try!(m.add("__doc__", "Module documentation string"));
///     try!(m.add("run", py_func!(py, run)));
///     Ok(())
/// });
/// 
/// fn run<'p>(py: Python<'p>, args: &PyTuple<'p>) -> PyResult<'p, PyObject<'p>> {
///     println!("Rust says: Hello Python!");
///     Ok(py.None())
/// }
/// # fn main() {}
/// ```
/// The code must be compiled into a file `example.so`.
///
/// ```bash
/// rustc example.rs -o example.so
/// ```
/// It can then be imported into python:
///
/// ```python
/// >>> import example
/// >>> example.run()
/// Rust says: Hello Python!
/// ```
/// 
#[macro_export]
macro_rules! py_module_initializer {
    ($name: tt, $init_funcname: ident, $init: expr) => {
        #[no_mangle]
        pub extern "C" fn $init_funcname() {
            let py = unsafe { $crate::Python::assume_gil_acquired() };
            let name = unsafe { ::std::ffi::CStr::from_ptr(concat!($name, "\0").as_ptr() as *const _) };
            match $crate::PyModule::_init(py, name, $init) {
                Ok(()) => (),
                Err(e) => e.restore()
            }
        }
    }
}

#[macro_export]
macro_rules! py_func {
    ($py: expr, $f: expr) => ({
        unsafe extern "C" fn wrap_py_func
          (_slf: *mut $crate::_detail::ffi::PyObject, args: *mut $crate::_detail::ffi::PyObject)
          -> *mut $crate::_detail::ffi::PyObject {
            let py = $crate::Python::assume_gil_acquired();
            let args = $crate::PyObject::from_borrowed_ptr(py, args);
            let args: &$crate::PyTuple = $crate::PythonObject::unchecked_downcast_borrow_from(&args);
            let result = match $f(py, args) {
                Ok(val) => $crate::ToPyObject::into_py_object(val, py),
                Err(e) => Err(e)
            };
            match result {
                Ok(val) => $crate::ToPythonPointer::steal_ptr(val),
                Err(e) => { e.restore(); ::std::ptr::null_mut() }
            }
        }
        static mut method_def: $crate::_detail::ffi::PyMethodDef = $crate::_detail::ffi::PyMethodDef {
            //ml_name: bytes!(stringify!($f), "\0"),
            ml_name: b"<rust function>\0" as *const u8 as *const $crate::_detail::libc::c_char,
            ml_meth: Some(wrap_py_func),
            ml_flags: $crate::_detail::ffi::METH_VARARGS,
            ml_doc: 0 as *const $crate::_detail::libc::c_char
        };
        let py: Python = $py;
        unsafe {
            let obj = $crate::_detail::ffi::PyCFunction_New(&mut method_def, ::std::ptr::null_mut());
            $crate::_detail::from_owned_ptr_or_panic(py, obj)
        }
    })
}

