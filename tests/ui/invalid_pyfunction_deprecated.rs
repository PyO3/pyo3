use pyo3::prelude::*;

#[pyfunction]
#[pyo3(deprecated)]
fn deprecated_function() {}

#[pyfunction]
#[pyo3(deprecated = )]
fn deprecated_function2() {}

#[pyfunction]
#[pyo3(deprecated = "first deprecated")]
#[pyo3(deprecated = "second deprecated")]
fn deprecated_function3() {}

fn main() {}
