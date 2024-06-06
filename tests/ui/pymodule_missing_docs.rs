#![deny(missing_docs)]
//! Some crate docs

use pyo3::prelude::*;

/// Some module documentation
#[pymodule]
pub fn python_module(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}

#[cfg(feature = "experimental-declarative-modules")]
/// Some module documentation
#[pymodule]
pub mod declarative_python_module {}

fn main() {}
