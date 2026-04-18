#[cfg(PyRustPython)]
#[path = "../rustpython.rs"]
mod legacy;

#[cfg(PyRustPython)]
pub use legacy::{
    RustPythonBackend, RustPythonClassBuilder, RustPythonFunctionBuilder, RustPythonInterpreter,
};

#[cfg(PyRustPython)]
pub mod err_state;
#[cfg(PyRustPython)]
pub mod pyclass;
#[cfg(PyRustPython)]
pub mod runtime;
#[cfg(PyRustPython)]
pub mod string;
#[cfg(PyRustPython)]
pub mod sync;
#[cfg(PyRustPython)]
pub mod types;
