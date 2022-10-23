#![allow(missing_docs)]

//! Internals of PyO3 which are accessed by code expanded from PyO3's procedural macros.
//!
//! Usage of any of these APIs in downstream code is implicitly acknowledging that these
//! APIs may may change at any time without documentation in the CHANGELOG and without
//! breaking semver guarantees.

pub mod deprecations;
pub mod extract_argument;
pub mod freelist;
pub mod frompyobject;
pub(crate) mod not_send;
pub mod panic;
pub mod pycell;
pub mod pyclass;
pub mod pyfunction;
pub mod pymethods;
pub mod pymodule;
