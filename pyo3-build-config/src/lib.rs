//! Configuration used by PyO3 for conditional support of varying Python versions.
//!
//! The only public API currently exposed is [`use_pyo3_cfgs`], which is intended to be used in
//! build scripts to add a standard set of `#[cfg]` attributes for handling multiple Python
//! versions.
//!
//! The full list of attributes added are the following:
//!
//! | Flag | Description |
//! | ---- | ----------- |
//! | `#[cfg(Py_3_6)]`, `#[cfg(Py_3_7)]`, `#[cfg(Py_3_8)]`, `#[cfg(Py_3_9)]`, `#[cfg(Py_3_10)]` | These attributes mark code only for a given Python version and up. For example, `#[cfg(Py_3_6)]` marks code which can run on Python 3.6 **and newer**. |
//! | `#[cfg(Py_LIMITED_API)]` | This marks code which is run when compiling with PyO3's `abi3` feature enabled. |
//! | `#[cfg(PyPy)]` | This marks code which is run when compiling for PyPy. |
//!
//! For examples of how to use these attributes, [see PyO3's guide](https://pyo3.rs/main/building_and_distribution/multiple_python_versions.html).

#[allow(dead_code)] // TODO cover this using tests
mod impl_;

#[doc(hidden)]
pub use crate::impl_::{InterpreterConfig, PythonImplementation, PythonVersion};

#[doc(hidden)]
pub fn get() -> InterpreterConfig {
    include!(concat!(env!("OUT_DIR"), "/pyo3-build-config.rs"))
}

pub fn use_pyo3_cfgs() {
    get().emit_pyo3_cfgs();
}
