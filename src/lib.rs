#![feature(specialization, proc_macro, try_from, fn_must_use)]

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
//! extern crate pyo3;
//!
//! use pyo3::{Python, PyDict, PyResult, ObjectProtocol};
//!
//! fn main() {
//!     let gil = Python::acquire_gil();
//!     hello(gil.python()).unwrap();
//! }
//!
//! fn hello(py: Python) -> PyResult<()> {
//!     let sys = py.import("sys")?;
//!     let version: String = sys.get("version")?.extract()?;
//!
//!     let locals = PyDict::new(py);
//!     locals.set_item("os", py.import("os")?)?;
//!     let user: String = py.eval("os.getenv('USER') or os.getenv('USERNAME')", None, Some(locals))?.extract()?;
//!
//!     println!("Hello {}, I'm Python {}", user, version);
//!     Ok(())
//! }
//! ```
//!
//! # Python extension
//!
//! To allow Python to load the rust code as a Python extension
//! module, you need provide initialization function and annotate it with `#[py::modinit(name)]`.
//! `py::modinit` expands to an `extern "C"` function.
//!
//! Macro syntax: `#[py::modinit(name)]`
//!
//! 1. `name`: The module name as a Rust identifier
//! 2. Decorate init function `Fn(Python, &PyModule) -> PyResult<()>`.
//!    This function will be called when the module is imported, and is responsible
//!    for adding the module's members.
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
//! #![feature(proc_macro, specialization)]
//!
//! extern crate pyo3;
//! use pyo3::{py, Python, PyResult, PyModule, PyString};
//!
//! // add bindings to the generated python module
//! // N.B: names: "libhello" must be the name of the `.so` or `.pyd` file
//!
//! /// Module documentation string
//! #[py::modinit(hello)]
//! fn init_module(py: Python, m: &PyModule) -> PyResult<()> {
//!
//!     // pyo3 aware function. All of our python interface could be declared
//!     // in a separate module.
//!     // Note that the `#[pyfn()]` annotation automatically converts the arguments from
//!     // Python objects to Rust values; and the Rust return value back into a Python object.
//!     #[pyfn(m, "run_rust_func")]
//!     fn run(name: &PyString) -> PyResult<()> {
//!         println!("Rust says: Hello {} of Python!", name);
//!         Ok(())
//!     }
//!
//!     Ok(())
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
//! The full example project can be found at:
//!   <https://github.com/PyO3/setuptools-rust/tree/master/example/>
//!
//! Rust will compile the code into a file named `libhello.so`, but we have to
//! rename the file in order to use it with Python:
//!
//! ```bash
//! cp ./target/debug/libhello.so ./hello.so
//! ```
//!
//! (Note: on macOS you will have to rename `libhello.dynlib` to `libhello.so`. 
//! To build on macOS, use `-C link-arg=-undefined -C link-arg=dynamic_lookup`
//! is required to build the library.
//! `setuptools-rust` includes this by default.
//! See [examples/word-count](https://github.com/PyO3/pyo3/tree/master/examples/word-count).)
//!
//! The extension module can then be imported into Python:
//!
//! ```python,ignore
//! >>> import hello
//! >>> hello.run_rust_func("test")
//! Rust says: Hello Python!
//! ```

extern crate libc;
extern crate spin;
extern crate pyo3cls;
#[macro_use] extern crate log;

#[cfg(not(Py_3))]
mod ffi2;

#[cfg(Py_3)]
mod ffi3;

/// Rust FFI declarations for Python
pub mod ffi {
    #[cfg(not(Py_3))]
    pub use ffi2::*;

    #[cfg(Py_3)]
    pub use ffi3::*;
}

pub use err::{PyErr, PyErrValue, PyResult, PyDowncastError, PyErrArguments};
pub use objects::*;
pub use objectprotocol::ObjectProtocol;
pub use object::PyObject;
pub use noargs::NoArgs;
pub use typeob::{PyTypeInfo, PyRawObject, PyObjectAlloc};
pub use python::{Python, ToPyPointer, IntoPyPointer, IntoPyDictPointer};
pub use pythonrun::{GILGuard, GILPool, prepare_freethreaded_python, prepare_pyo3_library};
pub use instance::{PyToken, PyObjectWithToken, AsPyRef, Py, PyNativeType};
pub use conversion::{FromPyObject, PyTryFrom, PyTryInto,
                     ToPyObject, ToBorrowedObject, IntoPyObject, IntoPyTuple};
pub mod class;
pub use class::*;

/// Procedural macros
pub mod py {
    pub use pyo3cls::{proto, class, methods};

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

mod python;
mod err;
mod conversion;
mod instance;
mod object;
mod objects;
mod objectprotocol;
mod noargs;
mod pythonrun;
#[doc(hidden)]
pub mod callback;
pub mod typeob;
#[doc(hidden)]
pub mod argparse;
pub mod buffer;
pub mod freelist;
pub mod prelude;

// re-export for simplicity
#[doc(hidden)]
pub use std::os::raw::*;
