//! Raw FFI declarations for Python's C API.
//!
//! This module provides low level bindings to the Python interpreter.
//! It is meant for advanced users only - regular PyO3 users shouldn't
//! need to interact with this module at all.
//!
//! The contents of this module are not documented here, as it would entail
//! basically copying the documentation from CPython. Consult the [Python/C API Reference
//! Manual][capi] for up-to-date documentation.
//!
//! # Safety
//!
//! The functions in this module lack individual safety documentation, but
//! generally the following apply:
//! - Pointer arguments have to point to a valid Python object of the correct type,
//!   although null pointers are sometimes valid input.
//! - The vast majority can only be used safely while the GIL is held.
//! - Some functions have additional safety requirements, consult the
//!   [Python/C API Reference Manual][capi] for more information.
//!
//! [capi]: https://docs.python.org/3/c-api/index.html

#[cfg(test)]
mod tests;

//  reexport raw bindings exposed in pyo3_ffi
pub use pyo3_ffi::*;

/// Helper to enable #\[pymethods\] to see the workaround for __ipow__ on Python 3.7
#[doc(hidden)]
pub use crate::impl_::pymethods::ipowfunc;
