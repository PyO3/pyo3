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

#![feature(unsafe_no_drop_flag)] // crucial so that PyObject<'p> is binary compatible with *mut ffi::PyObject
#![feature(filling_drop)] // necessary to avoid segfault with unsafe_no_drop_flag
#![feature(optin_builtin_traits)] // for opting out of Sync/Send
#![feature(slice_patterns)] // for tuple_conversion macros
#![feature(utf8_error)] // for translating Utf8Error to Python exception
#![feature(plugin)]
#![plugin(interpolate_idents)]
#![allow(unused_imports)] // because some imports are only necessary with python 2.x or 3.x

//! Rust bindings to the Python interpreter.
//!
//! # Ownership and Lifetimes
//! In Python, all objects are implicitly reference counted.
//! In rust, we will use the `PyObject` type to represent a reference to a Python object.
//!
//! Because all Python objects potentially have multiple owners, the concept
//! concept of rust mutability does not apply to Python objects.
//! As a result, this API will allow mutating Python objects even if they are not stored
//! in a mutable rust variable.
//!
//! The Python interpreter uses a global interpreter lock (GIL)
//! to ensure thread-safety.
//! This API uses the lifetime parameter `PyObject<'p>` to ensure that Python objects cannot
//! be accessed without holding the GIL.
//! Throughout this library, the lifetime `'p` always refers to the lifetime of the Python interpreter.
//!
//! When accessing existing objects, the lifetime on `PyObject<'p>` is sufficient to ensure that the GIL
//! is held by the current code. But we also need to ensure that the GIL is held when creating new objects.
//! For this purpose, this library uses the marker type `Python<'p>`,
//! which acts like a reference to the whole Python interpreter.
//!
//! You can obtain a `Python<'p>` instance by acquiring the GIL, or by calling `Python()`
//! on any existing Python object.
//!
//! # Error Handling
//! The vast majority of operations in this library will return `PyResult<'p, ...>`.
//! This is an alias for the type `Result<..., PyErr<'p>>`.
//!
//! A `PyErr` represents a Python exception. Errors within the rust-cpython library are
//! also exposed as Python exceptions.
//!
//! # Example
//! ```
//! extern crate cpython;
//!
//! use cpython::{PythonObject, Python};
//! use cpython::ObjectProtocol; //for call method
//!
//! fn main() {
//!     let gil = Python::acquire_gil();
//!     let py = gil.python();
//!
//!     let sys = py.import("sys").unwrap();
//!     let version: String = sys.get("version").unwrap().extract().unwrap();
//!
//!     let os = py.import("os").unwrap();
//!     let getenv = os.get("getenv").unwrap();
//!     let user: String = getenv.call(("USER",), None).unwrap().extract().unwrap();
//!
//!     println!("Hello {}, I'm Python {}", user, version);
//! }
//! ```

extern crate libc;

#[macro_use]
extern crate abort_on_panic;

#[cfg(feature="python27-sys")]
extern crate python27_sys as ffi;

#[cfg(feature="python3-sys")]
extern crate python3_sys as ffi;

pub use ffi::Py_ssize_t;
pub use err::{PyErr, PyResult};
pub use objects::*;
pub use python::{Python, PythonObject, PythonObjectWithCheckedDowncast, PythonObjectWithTypeObject};
pub use pythonrun::{GILGuard, GILProtected, prepare_freethreaded_python};
pub use conversion::{ExtractPyObject, ToPyObject};
pub use objectprotocol::{ObjectProtocol};
pub use rustobject::{PyRustType, PyRustObject};
pub use rustobject::typebuilder::PyRustTypeBuilder;

#[cfg(feature="python27-sys")]
#[allow(non_camel_case_types)]
pub type Py_hash_t = libc::c_long;

#[cfg(feature="python3-sys")]
#[allow(non_camel_case_types)]
pub type Py_hash_t = ffi::Py_hash_t;

use std::ptr;

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
pub mod argparse;
mod function;
mod rustobject;

/// Private re-exports for macros. Do not use.
#[doc(hidden)]
pub mod _detail {
    pub use ffi;
    pub use libc;
    pub use abort_on_panic::PanicGuard;
    pub use err::from_owned_ptr_or_panic;
    pub use function::py_fn_impl;
    pub use rustobject::method::{py_method_impl, py_class_method_impl};

    /// assume_gil_acquired(), but the returned Python<'p> is bounded by the scope
    /// of the referenced variable.
    /// This is useful in macros to ensure that type inference doesn't set 'p == 'static.
    #[inline]
    pub unsafe fn bounded_assume_gil_acquired<'p, T>(_bound: &'p T) -> super::Python<'p> {
        super::Python::assume_gil_acquired()
    }
}

/// Expands to an `extern "C"` function that allows Python to load
/// the rust code as a Python extension module.
///
/// Macro syntax: `py_module_initializer!($name, |$py, $m| $body)`
///
/// 1. The module name as a string literal.
/// 2. The name of the init function as an identifier.
///    The function must be named `init$module_name` so that Python 2.7 can load the module.
///    Note: this parameter will be removed in a future version
///    (once Rust supports `concat_ident!` as function name).
/// 3. A function or lambda of type `Fn(Python<'p>, &PyModule<'p>) -> PyResult<'p, ()>`.
///    This function will be called when the module is imported, and is responsible
///    for adding the module's members.
///
/// # Example
/// ```
/// #![crate_type = "dylib"]
/// #![feature(plugin)]
/// #![plugin(interpolate_idents)]
/// #[macro_use] extern crate cpython;
/// use cpython::{Python, PyResult, PyObject};
///
/// py_module_initializer!(example, |py, m| {
///     try!(m.add("__doc__", "Module documentation string"));
///     try!(m.add("run", py_fn!(run())));
///     Ok(())
/// });
///
/// fn run<'p>(py: Python<'p>) -> PyResult<'p, PyObject<'p>> {
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
/// It can then be imported into Python:
///
/// ```python
/// >>> import example
/// >>> example.run()
/// Rust says: Hello Python!
/// ```
///
#[macro_export]
#[cfg(feature="python27-sys")]
macro_rules! py_module_initializer {
    ($name: ident, |$py_id: ident, $m_id: ident| $body: expr) => ( interpolate_idents! {
        #[[no_mangle]]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn [ init $name ]() {
            // Nest init function so that $body isn't in unsafe context
            fn init<'pmisip>($py_id: $crate::Python<'pmisip>, $m_id: &$crate::PyModule<'pmisip>) -> $crate::PyResult<'pmisip, ()> {
                $body
            }
            let name = concat!(stringify!($name), "\0").as_ptr() as *const _;
            $crate::py_module_initializer_impl(name, init)
        }
    })
}


#[doc(hidden)]
#[cfg(feature="python27-sys")]
pub unsafe fn py_module_initializer_impl(
    name: *const libc::c_char,
    init: for<'p> fn(Python<'p>, &PyModule<'p>) -> PyResult<'p, ()>
) {
    abort_on_panic!({
        let py = Python::assume_gil_acquired();
        ffi::PyEval_InitThreads();
        let module = ffi::Py_InitModule(name, ptr::null_mut());
        if module.is_null() { return; }

        let module = match PyObject::from_borrowed_ptr(py, module).cast_into::<PyModule>() {
            Ok(m) => m,
            Err(e) => {
                PyErr::from(e).restore();
                return;
            }
        };
        match init(py, &module) {
            Ok(()) => (),
            Err(e) => e.restore()
        }
    })
}

#[macro_export]
#[cfg(feature="python3-sys")]
macro_rules! py_module_initializer {
    ($name: ident, |$py_id: ident, $m_id: ident| $body: expr) => ( interpolate_idents! {
        #[[no_mangle]]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn [ PyInit_ $name ]() -> *mut $crate::_detail::ffi::PyObject {
            // Nest init function so that $body isn't in unsafe context
            fn init<'pmisip>($py_id: $crate::Python<'pmisip>, $m_id: &$crate::PyModule<'pmisip>) -> $crate::PyResult<'pmisip, ()> {
                $body
            }
            static mut module_def: $crate::_detail::ffi::PyModuleDef = $crate::_detail::ffi::PyModuleDef {
                m_base: $crate::_detail::ffi::PyModuleDef_HEAD_INIT,
                m_name: 0 as *const _,
                m_doc: 0 as *const _,
                m_size: 0, // we don't use per-module state
                m_methods: 0 as *mut _,
                m_reload: None,
                m_traverse: None,
                m_clear: None,
                m_free: None
            };
            // We can't convert &'static str to *const c_char within a static initializer,
            // so we'll do it here in the module initialization:
            module_def.m_name = concat!(stringify!($name), "\0").as_ptr() as *const _;
            $crate::py_module_initializer_impl(&mut module_def, init)
        }
    })
}


#[doc(hidden)]
#[cfg(feature="python3-sys")]
pub unsafe fn py_module_initializer_impl(
    def: *mut ffi::PyModuleDef,
    init: for<'p> fn(Python<'p>, &PyModule<'p>) -> PyResult<'p, ()>
) -> *mut ffi::PyObject {
    abort_on_panic!({
        let py = Python::assume_gil_acquired();
        ffi::PyEval_InitThreads();
        let module = ffi::PyModule_Create(def);
        if module.is_null() { return module; }

        let module = match PyObject::from_owned_ptr(py, module).cast_into::<PyModule>() {
            Ok(m) => m,
            Err(e) => {
                PyErr::from(e).restore();
                return ptr::null_mut();
            }
        };
        match init(py, &module) {
            Ok(()) => module.into_object().steal_ptr(),
            Err(e) => {
                e.restore();
                return ptr::null_mut();
            }
        }
    })
}

