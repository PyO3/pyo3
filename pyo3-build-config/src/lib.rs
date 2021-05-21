//! Configuration used by PyO3 for conditional support of varying Python versions.
//!
//! The only public API currently exposed is [`use_pyo3_cfgs`], which is intended to be used in
//! build scripts to add a standard set of `#[cfg]` flags for handling multiple Python versions.
//!
//! TODO: tabulate all the flags here

#[allow(dead_code)] // TODO cover this using tests
mod impl_;

#[doc(hidden)]
pub use crate::impl_::{InterpreterConfig, PythonImplementation};

#[doc(hidden)]
pub fn get() -> InterpreterConfig {
    include!(concat!(env!("OUT_DIR"), "/pyo3-build-config.rs"))
}

pub fn use_pyo3_cfgs() {
    get().emit_pyo3_cfgs();
}
