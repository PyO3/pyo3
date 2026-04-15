#![allow(missing_docs)]

//! Backend contracts and backend dispatcher modules.

#[path = "cpython/mod.rs"]
pub mod cpython;
pub mod current;
#[path = "rustpython/mod.rs"]
pub mod rustpython;
#[cfg(PyRustPython)]
pub(crate) mod rustpython_storage;
/// Backend-neutral semantic specs.
pub mod spec;
/// Backend trait contracts.
pub mod traits;

#[cfg(test)]
mod tests;

pub use spec::{BackendKind, BackendSpec};
pub use traits::{Backend, BackendClassBuilder, BackendFunctionBuilder, BackendInterpreter};
