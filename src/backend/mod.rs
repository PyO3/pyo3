#![allow(missing_docs)]

//! Backend contracts and backend marker implementations.

/// CPython backend marker types.
pub mod cpython;
/// RustPython backend marker types.
pub mod rustpython;
/// Backend-neutral semantic specs.
pub mod spec;
/// Backend trait contracts.
pub mod traits;

#[cfg(test)]
mod tests;

pub use spec::{BackendKind, BackendSpec};
pub use traits::{Backend, BackendClassBuilder, BackendFunctionBuilder, BackendInterpreter};
