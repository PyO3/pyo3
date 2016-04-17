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

#![cfg_attr(feature="nightly", feature(
    unsafe_no_drop_flag, filling_drop, // (#5016)
    // ^ These two are crucial so that `PyObject` is binary compatible with
    //   `*mut ffi::PyObject`, which we use for efficient slice access and in
    //   some other cases.

    const_fn, // for GILProtected::new (#24111)
    shared, // for std::ptr::Shared (#27730)
    //recover, // for converting panics to python exceptions (#27719)
    // -- TODO wait for stable release and promote recover code from cfg(nightly) (1.9?)

    // -- TODO remove <DUMMY> hack when it's no longer necessary on stable (1.9?)
))]

#![allow(unused_imports)] // because some imports are only necessary with python 2.x or 3.x


//! Rust bindings to the Python interpreter.
//!
//! # Ownership and Lifetimes
//! In Python, all objects are implicitly reference counted.
//! In rust, we will use the `PyObject` type to represent a reference to a Python object.
//!
//! The method `clone_ref()` (from trait `PyClone`) can be used to create additional
//! references to the same Python object.
//!
//! Because all Python objects potentially have multiple owners, the
//! concept of Rust mutability does not apply to Python objects.
//! As a result, this API will allow mutating Python objects even if they are not stored
//! in a mutable Rust variable.
//!
//! The Python interpreter uses a global interpreter lock (GIL)
//! to ensure thread-safety.
//! This API uses a zero-sized `struct Python<'p>` as a token to indicate
//! that a function can assume that the GIL is held.
//!
//! You obtain a `Python` instance by acquiring the GIL,
//! and have to pass it into all operations that call into the Python runtime.
//!
//! # Error Handling
//! The vast majority of operations in this library will return `PyResult<...>`.
//! This is an alias for the type `Result<..., PyErr>`.
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
//!     let py = gil.python(); // obtain `Python` token
//!
//!     let sys = py.import("sys").unwrap();
//!     let version: String = sys.get(py, "version").unwrap().extract(py).unwrap();
//!
//!     let os = py.import("os").unwrap();
//!     let getenv = os.get(py, "getenv").unwrap();
//!     let user: String = getenv.call(py, ("USER",), None).unwrap().extract(py).unwrap();
//!
//!     println!("Hello {}, I'm Python {}", user, version);
//! }
//! ```

extern crate libc;

#[cfg(feature="python27-sys")]
extern crate python27_sys as ffi;

#[cfg(feature="python3-sys")]
extern crate python3_sys as ffi;

pub use ffi::Py_ssize_t;
pub use err::{PyErr, PyResult};
pub use objects::*;
pub use python::{Python, PythonObject, PythonObjectWithCheckedDowncast, PythonObjectDowncastError, PythonObjectWithTypeObject, PyClone, PyDrop};
pub use pythonrun::{GILGuard, GILProtected, prepare_freethreaded_python};
pub use conversion::{ExtractPyObject, ToPyObject};
pub use objectprotocol::{ObjectProtocol};

#[cfg(feature="python27-sys")]
#[allow(non_camel_case_types)]
pub type Py_hash_t = libc::c_long;

#[cfg(feature="python3-sys")]
#[allow(non_camel_case_types)]
pub type Py_hash_t = ffi::Py_hash_t;

use std::{ptr, mem};

/// Constructs a `&'static CStr` literal.
macro_rules! cstr(
    ($s: tt) => (
        // TODO: verify that $s is a string literal without nuls
        unsafe {
            ::std::ffi::CStr::from_ptr(concat!($s, "\0").as_ptr() as *const _)
        }
    );
);

// AST coercion macros (https://danielkeep.github.io/tlborm/book/blk-ast-coercion.html)
#[macro_export] #[doc(hidden)]
macro_rules! py_coerce_expr { ($s:expr) => {$s} }
#[macro_export] #[doc(hidden)]
macro_rules! py_coerce_item { ($s:item) => {$s} }

#[macro_export] #[doc(hidden)]
macro_rules! py_replace_expr {
    ($_t:tt $sub:expr) => {$sub};
}

mod python;
mod err;
mod conversion;
mod objects;
mod objectprotocol;
mod pythonrun;
pub mod argparse;
mod function;
//pub mod rustobject;
pub mod py_class;

/// Private re-exports for macros. Do not use.
#[doc(hidden)]
pub mod _detail {
    pub mod ffi {
        pub use ::ffi::*;
    }
    pub mod libc {
        pub use ::libc::{c_char, c_void, c_int};
    }
    pub use err::{from_owned_ptr_or_panic, result_from_owned_ptr};
    pub use function::{handle_callback, py_fn_impl, AbortOnDrop, PyObjectCallbackConverter};
}

/// Expands to an `extern "C"` function that allows Python to load
/// the rust code as a Python extension module.
///
/// Macro syntax: `py_module_initializer!($name, $py2_init, $py3_init, |$py, $m| $body)`
///
/// 1. `name`: The module name as a Rust identifier.
/// 2. `py2_init`: "init" + $name. Necessary because macros can't use concat_idents!().
/// 3. `py3_init`: "PyInit_" + $name. Necessary because macros can't use concat_idents!().
/// 4. A lambda of type `Fn(Python, &PyModule) -> PyResult<()>`.
///    This function will be called when the module is imported, and is responsible
///    for adding the module's members.
///
/// # Example
/// ```
/// #![crate_type = "dylib"]
/// #[macro_use] extern crate cpython;
/// use cpython::{Python, PyResult, PyObject};
///
/// py_module_initializer!(example, initexample, PyInit_example, |py, m| {
///     try!(m.add(py, "__doc__", "Module documentation string"));
///     try!(m.add(py, "run", py_fn!(py, run())));
///     Ok(())
/// });
///
/// fn run(py: Python) -> PyResult<PyObject> {
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
    ($name: ident, $py2: ident, $py3: ident, |$py_id: ident, $m_id: ident| $body: expr) => {
        #[no_mangle]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn $py2() {
            // Nest init function so that $body isn't in unsafe context
            fn init($py_id: $crate::Python, $m_id: &$crate::PyModule) -> $crate::PyResult<()> {
                $body
            }
            let name = concat!(stringify!($name), "\0").as_ptr() as *const _;
            $crate::py_module_initializer_impl(name, init)
        }
    }
}


#[doc(hidden)]
#[cfg(feature="python27-sys")]
pub unsafe fn py_module_initializer_impl(
    name: *const libc::c_char,
    init: fn(Python, &PyModule) -> PyResult<()>
) {
    let guard = function::AbortOnDrop("py_module_initializer");
    let py = Python::assume_gil_acquired();
    ffi::PyEval_InitThreads();
    let module = ffi::Py_InitModule(name, ptr::null_mut());
    if module.is_null() {
        mem::forget(guard);
        return;
    }

    let module = match PyObject::from_borrowed_ptr(py, module).cast_into::<PyModule>(py) {
        Ok(m) => m,
        Err(e) => {
            PyErr::from(e).restore(py);
            mem::forget(guard);
            return;
        }
    };
    let ret = match init(py, &module) {
        Ok(()) => (),
        Err(e) => e.restore(py)
    };
    mem::forget(guard);
    ret
}

#[macro_export]
#[cfg(feature="python3-sys")]
macro_rules! py_module_initializer {
    ($name: ident, $py2: ident, $py3: ident, |$py_id: ident, $m_id: ident| $body: expr) => {
        #[no_mangle]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn $py3() -> *mut $crate::_detail::ffi::PyObject {
            // Nest init function so that $body isn't in unsafe context
            fn init($py_id: $crate::Python, $m_id: &$crate::PyModule) -> $crate::PyResult<()> {
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
    }
}


#[doc(hidden)]
#[cfg(feature="python3-sys")]
pub unsafe fn py_module_initializer_impl(
    def: *mut ffi::PyModuleDef,
    init: fn(Python, &PyModule) -> PyResult<()>
) -> *mut ffi::PyObject {
    let guard = function::AbortOnDrop("py_module_initializer");
    let py = Python::assume_gil_acquired();
    ffi::PyEval_InitThreads();
    let module = ffi::PyModule_Create(def);
    if module.is_null() {
        mem::forget(guard);
        return module;
    }

    let module = match PyObject::from_owned_ptr(py, module).cast_into::<PyModule>(py) {
        Ok(m) => m,
        Err(e) => {
            PyErr::from(e).restore(py);
            mem::forget(guard);
            return ptr::null_mut();
        }
    };
    let ret = match init(py, &module) {
        Ok(()) => module.into_object().steal_ptr(),
        Err(e) => {
            e.restore(py);
            ptr::null_mut()
        }
    };
    mem::forget(guard);
    ret
}

