// Copyright (c) 2015 Daniel Grunwald
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this
// software and associated documentation files (the "Software"), to deal in the Software
// without restriction, including without limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
// to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
// FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

#![feature(core)] // used for a lot of low-level stuff
#![feature(unsafe_no_drop_flag)] // crucial so that PyObject<'p> is binary compatible with *mut ffi::PyObject
#![feature(filling_drop)] // necessary to avoid segfault with unsafe_no_drop_flag
#![feature(optin_builtin_traits)] // for opting out of Sync/Send
#![feature(slice_patterns)] // for tuple_conversion macros
#![feature(utf8_error)] // for translating Utf8Error to python exception
#![allow(unused_imports, unused_variables)]
#![feature(unicode)]

//! Rust bindings to the python interpreter.
//!
//! # Ownership and Lifetimes
//! In python, all objects are implicitly reference counted.
//! In rust, we will use the `PyObject` type to represent a reference to a python object.
//!
//! Because all python objects potentially have multiple owners, the concept
//! concept of rust mutability does not apply to python objects.
//! As a result, this API will allow mutating python objects even if they are not stored
//! in a mutable rust variable.
//!
//! The python interpreter uses a global interpreter lock (GIL)
//! to ensure thread-safety.
//! This API uses the lifetime parameter `PyObject<'p>` to ensure that python objects cannot
//! be accessed without holding the GIL.
//! Throughout this library, the lifetime `'p` always refers to the lifetime of the python interpreter.
//!
//! When accessing existing objects, the lifetime on `PyObject<'p>` is sufficient to ensure that the GIL
//! is held by the current code. But we also need to ensure that the GIL is held when creating new objects.
//! For this purpose, this library uses the marker type `Python<'p>`,
//! which acts like a reference to the whole python interpreter.
//!
//! You can obtain a `Python<'p>` instance by acquiring the GIL, or by calling `python()`
//! on any existing python object.
//!
//! # Error Handling
//! The vast majority of operations in this library will return `PyResult<'p, ...>`.
//! This is an alias for the type `Result<..., PyErr<'p>>`.
//!
//! A `PyErr` represents a python exception. Errors within the rust-cpython library are
//! also exposed as python exceptions.
//!
//! # Example
//! ```
//! extern crate cpython;
//!
//! use cpython::{PythonObject, Python};
//! 
//! fn main() {
//!     let gil_guard = Python::acquire_gil();
//!     let py = gil_guard.python();
//!     let sys = py.import("sys").unwrap();
//!     let version = sys.get("version").unwrap().extract::<String>().unwrap();
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
#[doc(hidden)]
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

/// Creates a python callable object that invokes a Rust function.
///
/// Arguments:
///
/// 1. The `Python<'p>` marker, to ensure this macro is only used while holding the GIL.
/// 2. A Rust function with the signature `<'p>(Python<'p>, &PyTuple<'p>) -> PyResult<'p, T>`
///    for some `T` that implements `ToPyObject`.
/// 
/// See `py_module_initializer!` for example usage.
///
/// # Panic
/// May panic when python runs out of memory.
#[macro_export]
macro_rules! py_func {
    ($py: expr, $f: expr) => ({
        unsafe extern "C" fn wrap_py_func
          (_slf: *mut $crate::_detail::ffi::PyObject, args: *mut $crate::_detail::ffi::PyObject)
          -> *mut $crate::_detail::ffi::PyObject {
            let py = $crate::Python::assume_gil_acquired();
            let args = $crate::PyObject::from_borrowed_ptr(py, args);
            let args: &$crate::PyTuple = $crate::PythonObject::unchecked_downcast_borrow_from(&args);
            match $f(py, args) {
                Ok(val) => {
                    let obj = $crate::ToPyObject::into_py_object(val, py);
                    return $crate::ToPythonPointer::steal_ptr(obj);
                }
                Err(e) => {
                    e.restore();
                    return ::std::ptr::null_mut();
                }
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

