#![feature(specialization, const_fn)]

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
//! A `PyErr` represents a Python exception. Errors within the PyO3 library are
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
//!     locals.set_item("os", py.import("os")?)?;
//!     let user: String = py.eval("os.getenv('USER') or os.getenv('USERNAME')", None, Some(&locals))?.extract(py)?;
//!
//!     println!("Hello {}, I'm Python {}", user, version);
//!     Ok(())
//! }
//! ```

extern crate libc;

#[allow(unused_imports)]
#[macro_use]
pub extern crate pyo3cls;

pub use pyo3cls::*;

pub mod ffi;
pub use ffi::{Py_ssize_t, Py_hash_t};

pub mod pyptr;
pub use pyptr::{Py, PyPtr};

mod ppptr;
pub use ppptr::pptr;

mod token;
pub use token::{PyObjectMarker, PythonToken, PythonObjectWithToken};

pub use err::{PyErr, PyResult, PyDowncastError};
pub use objects::*;
pub use objectprotocol::ObjectProtocol;
pub use python::{Python, IntoPythonPointer};
pub use pythonrun::{GILGuard, GILProtected, prepare_freethreaded_python};
pub use conversion::{FromPyObject, RefFromPyObject, ToPyObject, IntoPyObject, ToPyTuple};
pub use class::{CompareOp};
pub mod class;
pub use class::*;
pub use self::typeob::PyTypeObject;

#[allow(non_camel_case_types)]

use std::{ptr, mem};

pub mod py {
    pub use pyo3cls::*;
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

// AST coercion macros (https://danielkeep.github.io/tlborm/book/blk-ast-coercion.html)
#[macro_export] #[doc(hidden)]
macro_rules! py_coerce_expr { ($s:expr) => {$s} }

#[macro_export] #[doc(hidden)]
macro_rules! py_replace_expr {
    ($_t:tt $sub:expr) => {$sub};
}

pub mod python;
pub mod native;
mod err;
mod conversion;
mod objects;
mod objectprotocol;
mod pythonrun;
pub mod callback;
pub mod typeob;
pub mod argparse;
pub mod function;
pub mod buffer;

// re-export for simplicity
pub use std::os::raw::*;

/// Expands to an `extern "C"` function that allows Python to load
/// the rust code as a Python extension module.
///
/// Macro syntax: `py_module_initializer!($name, $py2_init, $py3_init, |$py, $m| $body)`
///
/// 1. `name`: The module name as a Rust identifier.
/// 2. `py3_init`: "PyInit_" + $name. Necessary because macros can't use concat_idents!().
/// 4. A lambda of type `Fn(Python, &PyModule) -> PyResult<()>`.
///    This function will be called when the module is imported, and is responsible
///    for adding the module's members.
///
/// # Example
/// ```
/// #[macro_use] extern crate pyo3;
/// use pyo3::{Python, PyResult, PyObject};
///
/// py_module_init!(hello, PyInit_hello, |py, m| {
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
/// In your `Cargo.toml`, use the `extension-module` feature for the `pyo3` dependency:
/// ```cargo
/// [dependencies.pyo3]
/// version = "*"
/// features = ["extension-module"]
/// ```
/// The full example project can be found at:
///   https://github.com/PyO3/setuptools-rust/tree/master/example/extensions
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
macro_rules! py_module_init {
    ($name: ident, $py3: ident, |$py_id: ident, $m_id: ident| $body: expr) => {
        #[no_mangle]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn $py3() -> *mut $crate::ffi::PyObject {
            // Nest init function so that $body isn't in unsafe context
            fn init($py_id: $crate::Python, $m_id: &$crate::PyModule) -> $crate::PyResult<()> {
                $body
            }
            static mut MODULE_DEF: $crate::ffi::PyModuleDef = $crate::ffi::PyModuleDef_INIT;
            // We can't convert &'static str to *const c_char within a static initializer,
            // so we'll do it here in the module initialization:
            MODULE_DEF.m_name = concat!(stringify!($name), "\0").as_ptr() as *const _;
            $crate::py_module_init_impl(&mut MODULE_DEF, init)
        }
    }
}


#[doc(hidden)]
pub unsafe fn py_module_init_impl(
    def: *mut ffi::PyModuleDef,
    init: fn(Python, &PyModule) -> PyResult<()>) -> *mut ffi::PyObject
{
    let guard = callback::AbortOnDrop("py_module_init");
    let py = Python::assume_gil_acquired();
    ffi::PyEval_InitThreads();
    let module = ffi::PyModule_Create(def);
    if module.is_null() {
        mem::forget(guard);
        return module;
    }

    let module = match Py::<PyModule>::cast_from_owned_ptr(py, module) {
        Ok(m) => m,
        Err(e) => {
            PyErr::from(e).restore(py);
            mem::forget(guard);
            return ptr::null_mut();
        }
    };
    let ret = match init(py, &module) {
        Ok(()) => module.into_ptr(),
        Err(e) => {
            e.restore(py);
            ptr::null_mut()
        }
    };
    mem::forget(guard);
    ret
}
