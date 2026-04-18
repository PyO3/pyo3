#![allow(missing_docs)]

//! Backend contracts and backend dispatcher modules.

#[path = "cpython/mod.rs"]
pub mod cpython;
pub mod current;
#[cfg(PyRustPython)]
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

#[cfg(all(feature = "runtime-cpython", feature = "runtime-rustpython"))]
compile_error!("features `runtime-cpython` and `runtime-rustpython` are mutually exclusive");
#[cfg(all(feature = "runtime-rustpython", not(PyRustPython)))]
compile_error!(
    "feature `runtime-rustpython` requires the `PyRustPython` cfg from the build scripts"
);
#[cfg(all(feature = "runtime-cpython", PyRustPython))]
compile_error!("cfg `PyRustPython` is only valid with feature `runtime-rustpython`");

#[cfg(feature = "runtime-cpython")]
pub use self::cpython::CpythonBackend as ActiveBackend;
#[cfg(feature = "runtime-rustpython")]
pub use self::rustpython::RustPythonBackend as ActiveBackend;

/// Returns the backend selected for this build.
pub const fn active_backend_kind() -> BackendKind {
    <ActiveBackend as Backend>::KIND
}

pub use spec::{BackendKind, BackendSpec};
pub use traits::{Backend, BackendClassBuilder, BackendFunctionBuilder, BackendInterpreter};
