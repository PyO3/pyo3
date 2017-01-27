// Copyright (c) 2016 Daniel Grunwald
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
    const_fn, // for GILProtected::new (#24111)
    shared, // for std::ptr::Shared (#27730)
    specialization, // for impl FromPyObject<'source> for Vec<...> (#31844)
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
//! use cpython::{Python, PyDict, PyResult};
//!
//! fn main() {
//!     let gil = Python::acquire_gil();
//!     hello(gil.python()).unwrap();
//! }
//!
//! fn hello(py: Python) -> PyResult<()> {
//!     let sys = py.import("sys")?;
//!     let version: String = sys.get(py, "version")?.extract(py)?;
//!
//!     let locals = PyDict::new(py);
//!     locals.set_item(py, "os", py.import("os")?)?;
//!     let user: String = py.eval("os.getenv('USER') or os.getenv('USERNAME')", None, Some(&locals))?.extract(py)?;
//!
//!     println!("Hello {}, I'm Python {}", user, version);
//!     Ok(())
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
pub use conversion::{FromPyObject, RefFromPyObject, ToPyObject};
pub use py_class::{CompareOp};
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

#[macro_export] #[doc(hidden)]
macro_rules! py_impl_to_py_object_for_python_object {
    ($T: ty) => (
        /// Identity conversion: allows using existing `PyObject` instances where
        /// `T: ToPyObject` is expected.
        impl $crate::ToPyObject for $T {
            type ObjectType = $T;

            #[inline]
            fn to_py_object(&self, py: $crate::Python) -> $T {
                $crate::PyClone::clone_ref(self, py)
            }

            #[inline]
            fn into_py_object(self, _py: $crate::Python) -> $T {
                self
            }

            #[inline]
            fn with_borrowed_ptr<F, R>(&self, _py: $crate::Python, f: F) -> R
                where F: FnOnce(*mut $crate::_detail::ffi::PyObject) -> R
            {
                f($crate::PythonObject::as_object(self).as_ptr())
            }
        }
    )
}

#[macro_export] #[doc(hidden)]
macro_rules! py_impl_from_py_object_for_python_object {
    ($T:ty) => {
        impl <'source> $crate::FromPyObject<'source> for $T {
            #[inline]
            fn extract(py: $crate::Python, obj: &'source $crate::PyObject) -> $crate::PyResult<$T> {
                use $crate::PyClone;
                Ok(try!(obj.clone_ref(py).cast_into::<$T>(py)))
            }
        }

        impl <'source> $crate::FromPyObject<'source> for &'source $T {
            #[inline]
            fn extract(py: $crate::Python, obj: &'source $crate::PyObject) -> $crate::PyResult<&'source $T> {
                Ok(try!(obj.cast_as::<$T>(py)))
            }
        }
    }
}

mod python;
mod err;
mod conversion;
mod objects;
mod objectprotocol;
mod pythonrun;
pub mod argparse;
mod function;
pub mod buffer;
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
    pub use function::{handle_callback, py_fn_impl, AbortOnDrop,
        PyObjectCallbackConverter, PythonObjectCallbackConverter};
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
/// #[macro_use] extern crate cpython;
/// use cpython::{Python, PyResult, PyObject};
///
/// py_module_initializer!(hello, inithello, PyInit_hello, |py, m| {
///     m.add(py, "__doc__", "Module documentation string")?;
///     m.add(py, "run", py_fn!(py, run()))?;
///     Ok(())
/// });
///
/// fn run(py: Python) -> PyResult<PyObject> {
///     println!("Rust says: Hello Python!");
///     Ok(py.None())
/// }
/// # fn main() {}
/// ```
///
/// In your `Cargo.toml`, use the `extension-module` feature for the `cpython` dependency:
/// ```cargo
/// [dependencies.cpython]
/// version = "*"
/// features = ["extension-module"]
/// ```
/// The full example project can be found at:
///   https://github.com/dgrunwald/rust-cpython/tree/master/extensions/hello
///
/// Rust will compile the code into a file named `libhello.so`, but we have to
/// rename the file in order to use it with Python:
///
/// ```bash
/// cp ./target/debug/libhello.so ./hello.so
/// ```
/// (Note: on Mac OS you will have to rename `libhello.dynlib` to `libhello.so`)
///
/// The extension module can then be imported into Python:
///
/// ```python
/// >>> import hello
/// >>> hello.run()
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
            static mut MODULE_DEF: $crate::_detail::ffi::PyModuleDef = $crate::_detail::ffi::PyModuleDef_INIT;
            // We can't convert &'static str to *const c_char within a static initializer,
            // so we'll do it here in the module initialization:
            MODULE_DEF.m_name = concat!(stringify!($name), "\0").as_ptr() as *const _;
            $crate::py_module_initializer_impl(&mut MODULE_DEF, init)
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
