//! Internals of PyO3 which are accessed by code expanded from PyO3's procedural macros.
//!
//! Usage of any of these APIs in downstream code is implicitly acknowledging that these
//! APIs may may change at any time without documentation in the CHANGELOG and without
//! breaking semver guarantees.

pub mod deprecations;
pub mod extract_argument;
pub mod freelist;
#[doc(hidden)]
pub mod frompyobject;
pub(crate) mod not_send;
pub mod panic;
#[doc(hidden)]
pub mod pyclass;
#[doc(hidden)]
pub mod pyfunction;
#[doc(hidden)]
pub mod pymethods;
pub mod pymodule;
