#![feature(specialization)]

//! Rust bindings to the Python interpreter.
//!
//! # Ownership and Lifetimes
//! In Python, all objects are implicitly reference counted.
//! In rust, we will use the `PyObject` type to represent a reference to a Python object.
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
//! A `PyErr` represents a Python exception. Errors within the `PyO3` library are
//! also exposed as Python exceptions.
//!
//! # Example
//!
//! ```rust
//! #![feature(specialization)]
//!
//! extern crate pyo3;
//!
//! use pyo3::prelude::*;
//! use pyo3::types::PyDict;
//!
//! fn main() -> PyResult<()> {
//!     let gil = Python::acquire_gil();
//!     let py = gil.python();
//!     let sys = py.import("sys")?;
//!     let version: String = sys.get("version")?.extract()?;
//!
//!     let locals = PyDict::new(py);
//!     locals.set_item("os", py.import("os")?)?;
//!     let code = "os.getenv('USER') or os.getenv('USERNAME') or 'Unknown'";
//!     let user: String = py.eval(code, None, Some(&locals))?.extract()?;
//!
//!     println!("Hello {}, I'm Python {}", user, version);
//!     Ok(())
//! }
//! ```
//!
//! # Python extension
//!
//! To allow Python to load the rust code as a Python extension
//! module, you need an initialization function with `Fn(Python, &PyModule) -> PyResult<()>`
//! that is annotates with `#[pymodinit]`. By default the function name will become the module name,
//! but you can override that with `#[pymodinit(name)]`.
//!
//! To creates a Python callable object that invokes a Rust function, specify rust
//! function and decorate it with `#[pyfn()]` attribute. `pyfn()` accepts three parameters.
//!
//! 1. `m`: The module name.
//! 2. name of function visible to Python code.
//! 3. comma separated arguments, i.e. param="None", "*", param3="55"
//!
//!
//! # Example
//!
//! ```rust
//! #![feature(specialization)]
//!
//! extern crate pyo3;
//! use pyo3::prelude::*;
//!
//! // Add bindings to the generated python module
//! // N.B: names: "librust2py" must be the name of the `.so` or `.pyd` file
//! /// This module is implemented in Rust.
//! #[pymodinit]
//! fn rust2py(py: Python, m: &PyModule) -> PyResult<()> {
//!
//!     #[pyfn(m, "sum_as_string")]
//!     // ``#[pyfn()]` converts the arguments from Python objects to Rust values
//!     // and the Rust return value back into a Python object.
//!     fn sum_as_string_py(a:i64, b:i64) -> PyResult<String> {
//!        let out = sum_as_string(a, b);
//!        Ok(out)
//!     }
//!
//!     Ok(())
//! }
//!
//! // The logic can be implemented as a normal rust function
//! fn sum_as_string(a:i64, b:i64) -> String {
//!     format!("{}", a + b).to_string()
//! }
//!
//! # fn main() {}
//! ```
//!
//! In your `Cargo.toml`, use the `extension-module` feature for the `pyo3` dependency:
//!
//! ```cargo
//! [dependencies.pyo3]
//! version = "*"
//! features = ["extension-module"]
//! ```
//!
//! On windows and linux, you can build normally with `cargo build --release`. On Mac Os, you need to set additional linker arguments. One option is to compile with `cargo rustc --release -- -C link-arg=-undefined -C link-arg=dynamic_lookup`, the other is to create a `.cargo/config` with the following content:
//!
//! ```toml
//! [target.x86_64-apple-darwin]
//! rustflags = [
//!   "-C", "link-arg=-undefined",
//!   "-C", "link-arg=dynamic_lookup",
//! ]
//! ```
//!
//! Also on macOS, you will need to rename the output from \*.dylib to \*.so. On Windows, you will need to rename the output from \*.dll to \*.pyd.
//!
//! [`setuptools-rust`](https://github.com/PyO3/setuptools-rust) can be used to generate a python package and includes the commands above by default. See [examples/word-count](examples/word-count) and the associated setup.py.

#[cfg(test)]
#[macro_use]
extern crate assert_approx_eq;
#[cfg(test)]
#[macro_use]
extern crate indoc;
// We need those types in the macro exports
#[doc(hidden)]
pub extern crate libc;
// We need that reexport for wrap_function
#[doc(hidden)]
pub extern crate mashup;
extern crate pyo3cls;
extern crate spin;

pub use crate::class::*;
pub use crate::conversion::{
    FromPyObject, IntoPyObject, IntoPyTuple, PyTryFrom, PyTryInto, ReturnTypeIntoPyResult,
    ToBorrowedObject, ToPyObject,
};
pub use crate::err::{PyDowncastError, PyErr, PyErrArguments, PyErrValue, PyResult};
pub use crate::instance::{AsPyRef, Py, PyNativeType, PyObjectWithGIL};
pub use crate::noargs::NoArgs;
pub use crate::object::PyObject;
pub use crate::objectprotocol::ObjectProtocol;
pub use crate::python::{IntoPyPointer, Python, ToPyPointer};
pub use crate::pythonrun::{init_once, prepare_freethreaded_python, GILGuard, GILPool};
pub use crate::typeob::{PyObjectAlloc, PyRawObject, PyTypeInfo};
pub use crate::types::exceptions;

/// Rust FFI declarations for Python
pub mod ffi;

#[cfg(not(Py_3))]
mod ffi2;

#[cfg(Py_3)]
mod ffi3;

pub mod class;

/// Constructs a `&'static CStr` literal.
macro_rules! cstr {
    ($s: tt) => {
        // TODO: verify that $s is a string literal without nuls
        unsafe { ::std::ffi::CStr::from_ptr(concat!($s, "\0").as_ptr() as *const _) }
    };
}

pub mod buffer;
#[doc(hidden)]
pub mod callback;
mod conversion;
#[doc(hidden)]
pub mod derive_utils;
mod err;
pub mod freelist;
mod instance;
mod noargs;
mod object;
mod objectprotocol;
pub mod prelude;
pub mod python;
mod pythonrun;
pub mod typeob;
pub mod types;

/// The proc macros, which are also part of the prelude
pub mod proc_macro {
    #[cfg(not(Py_3))]
    pub use pyo3cls::mod2init as pymodinit;
    #[cfg(Py_3)]
    pub use pyo3cls::mod3init as pymodinit;
    /// The proc macro attributes
    pub use pyo3cls::{pyclass, pyfunction, pymethods, pyproto};
}

/// Returns a function that takes a [Python] instance and returns a python function.
///
/// Use this together with `#[pyfunction]` and [types::PyModule::add_wrapped].
#[macro_export]
macro_rules! wrap_function {
    ($function_name:ident) => {{
        // Get the mashup macro and its helpers into scope
        use $crate::mashup::*;

        mashup! {
            // Make sure this ident matches the one in function_wrapper_ident
            m["method"] = __pyo3_get_function_ $function_name;
        }

        m! {
            &"method"
        }
    }};
}

/// Returns a function that takes a [Python] instance and returns a python module.
///
/// Use this together with `#[pymodule]` and [types::PyModule::add_wrapped].
#[cfg(Py_3)]
#[macro_export]
macro_rules! wrap_module {
    ($module_name:ident) => {{
        use $crate::mashup::*;

        mashup! {
            m["method"] = PyInit_ $module_name;
        }

        m! {
            &|py| unsafe { crate::PyObject::from_owned_ptr(py, "method"()) }
        }
    }};
}
