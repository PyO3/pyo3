#![deny(deprecated)]

use pyo3::prelude::*;

#[pymodule]
fn module_with_pyfn(m: &Bound<'_, PyModule>) -> PyResult<()> {
    #[pyfn(m)]
    fn foo() {}

    Ok(())
}

fn main() {}
