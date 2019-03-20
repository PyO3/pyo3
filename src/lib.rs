#![feature(specialization)]

//! Rust bindings to the Python interpreter.
//!
//! Look at [the guide](https://pyo3.rs/) for a detailed introduction.
//!
//! # Ownership and Lifetimes
//!
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
//! ## Using rust from python
//!
//! Pyo3 can be used to generate a native python module.
//!
//! **`Cargo.toml`**
//!
//! ```toml
//! [package]
//! name = "string-sum"
//! version = "0.1.0"
//! edition = "2018"
//!
//! [lib]
//! name = "string_sum"
//! crate-type = ["cdylib"]
//!
//! [dependencies.pyo3]
//! version = "0.6.0-alpha.4"
//! features = ["extension-module"]
//! ```
//!
//! **`src/lib.rs`**
//!
//! ```rust
//! use pyo3::prelude::*;
//! use pyo3::wrap_pyfunction;
//!
//! #[pyfunction]
//! /// Formats the sum of two numbers as string
//! fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
//!     Ok((a + b).to_string())
//! }
//!
//! /// This module is a python module implemented in Rust.
//! #[pymodule]
//! fn string_sum(py: Python, m: &PyModule) -> PyResult<()> {
//!     m.add_wrapped(wrap_pyfunction!(sum_as_string))?;
//!
//!     Ok(())
//! }
//! ```
//!
//! On windows and linux, you can build normally with `cargo build --release`. On macOS, you need to set additional linker arguments. One option is to compile with `cargo rustc --release -- -C link-arg=-undefined -C link-arg=dynamic_lookup`, the other is to create a `.cargo/config` with the following content:
//!
//! ```toml
//! [target.x86_64-apple-darwin]
//! rustflags = [
//!   "-C", "link-arg=-undefined",
//!   "-C", "link-arg=dynamic_lookup",
//! ]
//! ```
//!
//! For developing, you can copy and rename the shared library from the target folder: On macOS, rename `libstring_sum.dylib` to `string_sum.so`, on windows `libstring_sum.dll` to `string_sum.pyd` and on linux `libstring_sum.so` to `string_sum.so`. Then open a python shell in the same folder and you'll be able to `import string_sum`.
//!
//! To build, test and publish your crate as python module, you can use [pyo3-pack](https://github.com/PyO3/pyo3-pack) or [setuptools-rust](https://github.com/PyO3/setuptools-rust). You can find an example for setuptools-rust in [examples/word-count](examples/word-count), while pyo3-pack should work on your crate without any configuration.
//!
//! ## Using python from rust
//!
//! Add `pyo3` this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! pyo3 = "0.6.0-alpha.4"
//! ```
//!
//! Example program displaying the value of `sys.version`:
//!
//! ```rust
//! use pyo3::prelude::*;
//! use pyo3::types::IntoPyDict;
//!
//! fn main() -> PyResult<()> {
//!     let gil = Python::acquire_gil();
//!     let py = gil.python();
//!     let sys = py.import("sys")?;
//!     let version: String = sys.get("version")?.extract()?;
//!
//!     let locals = [("os", py.import("os")?)].into_py_dict(py);
//!     let code = "os.getenv('USER') or os.getenv('USERNAME') or 'Unknown'";
//!     let user: String = py.eval(code, None, Some(&locals))?.extract()?;
//!
//!     println!("Hello {}, I'm Python {}", user, version);
//!     Ok(())
//! }
//! ```

pub use crate::class::*;
pub use crate::conversion::{
    AsPyPointer, FromPy, FromPyObject, IntoPy, IntoPyObject, IntoPyPointer, PyTryFrom, PyTryInto,
    ToBorrowedObject, ToPyObject,
};
pub use crate::err::{PyDowncastError, PyErr, PyErrArguments, PyErrValue, PyResult};
pub use crate::gil::{init_once, GILGuard, GILPool};
pub use crate::instance::{AsPyRef, ManagedPyRef, Py, PyNativeType, PyRef, PyRefMut};
pub use crate::object::PyObject;
pub use crate::objectprotocol::ObjectProtocol;
pub use crate::python::{prepare_freethreaded_python, Python};
pub use crate::type_object::{PyObjectAlloc, PyRawObject, PyTypeInfo};

// We need that reexport for wrap_function
#[doc(hidden)]
pub use mashup;
// We need that reexport for pymethods
#[doc(hidden)]
pub use inventory;

/// Raw ffi declarations for the c interface of python
pub mod ffi;

#[cfg(not(Py_3))]
mod ffi2;

#[cfg(Py_3)]
mod ffi3;

pub mod buffer;
#[doc(hidden)]
pub mod callback;
pub mod class;
mod conversion;
#[doc(hidden)]
pub mod derive_utils;
mod err;
pub mod exceptions;
pub mod freelist;
mod gil;
mod instance;
mod object;
mod objectprotocol;
pub mod prelude;
mod python;
pub mod type_object;
pub mod types;

/// The proc macros, which are also part of the prelude
pub mod proc_macro {
    #[cfg(not(Py_3))]
    pub use pyo3cls::pymodule2 as pymodule;
    #[cfg(Py_3)]
    pub use pyo3cls::pymodule3 as pymodule;
    /// The proc macro attributes
    pub use pyo3cls::{pyclass, pyfunction, pymethods, pyproto};
}

/// Returns a function that takes a [Python] instance and returns a python function.
///
/// Use this together with `#[pyfunction]` and [types::PyModule::add_wrapped].
#[macro_export]
macro_rules! wrap_pyfunction {
    ($function_name:ident) => {{
        // Get the mashup macro and its helpers into scope
        use pyo3::mashup::*;

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
macro_rules! wrap_pymodule {
    ($module_name:ident) => {{
        use pyo3::mashup::*;

        mashup! {
            m["method"] = PyInit_ $module_name;
        }

        m! {
            &|py| unsafe { crate::PyObject::from_owned_ptr(py, "method"()) }
        }
    }};
}
