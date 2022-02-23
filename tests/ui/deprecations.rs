#![deny(deprecated)]

use pyo3::prelude::*;

#[pyclass]
struct DeprecatedCall;

#[pymethods]
impl DeprecatedCall {
    #[call]
    fn deprecated_call(&self) {}
}

#[pyclass(gc)]
struct DeprecatedGc;

fn main() {

}
