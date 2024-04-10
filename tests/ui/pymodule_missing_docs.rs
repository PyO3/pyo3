#![deny(missing_docs)]
//! Some crate docs

use pyo3::prelude::*;

/// Some module documentation
#[pymodule]
pub fn python_module(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}

fn main() {}
