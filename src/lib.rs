#![feature(specialization, const_fn, proc_macro)]

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
//! A `PyErr` represents a Python exception. Errors within the `PyO3` library are
//! also exposed as Python exceptions.
//!
//! # Example
//! ```
//! extern crate pyo3;
//!
//! use pyo3::{Python, PyDict, PyResult};
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
//!
//! Expands to an `extern "C"` function that allows Python to load
//! the rust code as a Python extension module.
//!
//! Macro syntax: `#[py::modinit(name)]`
//!
//! 1. `name`: The module name as a Rust identifier
//! 2. Decorate init function `Fn(Python, &PyModule) -> PyResult<()>`.
//!    This function will be called when the module is imported, and is responsible
//!    for adding the module's members.
//!
//! To creates a Python callable object that invokes a Rust function, specify rust
//! function and decroate it with `#[pyfn()]` attribute. `pyfn()` accepts three parameters.
//!
//! 1. `m`: The module name.
//! 2. function name, name of function visible to Python code.
//! 3. arguments description string, i.e. "param1, param2=None, *, param3='default'"
//!
//!
//! # Example
//! ```
//! #![feature(proc_macro)]
//! extern crate pyo3;
//! use pyo3::{py, Python, PyResult, PyObject, PyModule, PyString};
//!
//! #[py::modinit(hello)]
//! fn init_module(py: Python, m: &PyModule) -> PyResult<()> {
//!     m.add(py, "__doc__", "Module documentation string")?;
//!
//!     #[pyfn(m, "run_rust_func")]
//!     fn run(py: Python, name: PyString) -> PyResult<PyObject> {
//!         println!("Rust says: Hello {} of Python!", name);
//!         Ok(py.None())
//!     }
//!
//!     Ok(())
//! }
//!
//! # fn main() {}
//! ```
//!
//! In your `Cargo.toml`, use the `extension-module` feature for the `pyo3` dependency:
//! ```cargo
//! [dependencies.pyo3]
//! version = "*"
//! features = ["extension-module"]
//! ```
//!
//! The full example project can be found at:
//!   https://github.com/PyO3/setuptools-rust/tree/master/example/extensions
//!
//! Rust will compile the code into a file named `libhello.so`, but we have to
//! rename the file in order to use it with Python:
//!
//! ```bash
//! cp ./target/debug/libhello.so ./hello.so
//! ```
//! (Note: on Mac OS you will have to rename `libhello.dynlib` to `libhello.so`)
//!
//! The extension module can then be imported into Python:
//!
//! ```python
//! >>> import hello
//! >>> hello.run_rust_func("test")
//! Rust says: Hello Python!
//! ```

extern crate libc;
extern crate backtrace;
extern crate pyo3cls;
#[macro_use] extern crate log;

#[cfg(not(Py_3))]
mod ffi2;

#[cfg(Py_3)]
mod ffi3;

pub mod ffi {
    #[cfg(not(Py_3))]
    pub use ffi2::*;

    #[cfg(Py_3)]
    pub use ffi3::*;
}

pub use ffi::{Py_ssize_t, Py_hash_t};

pub mod pointers;
pub use pointers::PyPtr;

mod token;
pub use token::{PyToken, PyObjectWithToken, ToInstancePtr, InstancePtr};

pub use err::{PyErr, PyResult, PyDowncastError, ToPyErr};
pub use objects::*;
pub use objectprotocol::ObjectProtocol;
pub use python::{Python, ToPyPointer, IntoPyPointer, PyClone,
                 PyMutDowncastFrom, PyDowncastFrom, PyDowncastInto};
pub use pythonrun::{GILGuard, prepare_freethreaded_python};
pub use conversion::{FromPyObject, RefFromPyObject, ToPyObject, IntoPyObject, IntoPyTuple};
pub use class::{CompareOp};
pub mod class;
pub use class::*;
pub use self::typeob::PyTypeObject;

pub mod py {
    pub use pyo3cls::*;

    #[cfg(Py_3)]
    pub use pyo3cls::mod3init as modinit;

    #[cfg(not(Py_3))]
    pub use pyo3cls::mod2init as modinit;
}

/// Constructs a `&'static CStr` literal.
macro_rules! cstr(
    ($s: tt) => (
        // TODO: verify that $s is a string literal without nuls
        unsafe {
            ::std::ffi::CStr::from_ptr(concat!($s, "\0").as_ptr() as *const _)
        }
    );
);

pub mod python;
mod fmt;
mod err;
mod conversion;
mod objects;
mod objectprotocol;
mod pythonrun;
pub mod callback;
pub mod typeob;
pub mod argparse;
pub mod buffer;
pub mod freelist;

// re-export for simplicity
pub use std::os::raw::*;
