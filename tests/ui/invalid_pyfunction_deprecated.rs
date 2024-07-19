use pyo3::prelude::*;

#[pyfunction]
#[pyo3(deprecated)]
fn deprecated_function() {}

#[pyfunction]
#[pyo3(deprecated = )]
fn deprecated_function2() {}

fn main() {}
