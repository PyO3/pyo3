#![allow(missing_docs)]

//! Internals of PyO3 which are accessed by code expanded from PyO3's procedural macros.
//!
//! Usage of any of these APIs in downstream code is implicitly acknowledging that these
//! APIs may may change at any time without documentation in the CHANGELOG and without
//! breaking semver guarantees.

pub mod callback;
pub mod concat;
#[cfg(feature = "experimental-async")]
pub mod coroutine;
pub mod exceptions;
pub mod extract_argument;
pub mod freelist;
pub mod frompyobject;
#[cfg(feature = "experimental-inspect")]
pub mod introspection;
pub mod panic;
pub mod pycell;
pub mod pyclass;
pub mod pyclass_init;
pub mod pyfunction;
pub mod pymethods;
pub mod pymodule;
#[doc(hidden)]
pub mod trampoline;
pub mod wrap;
