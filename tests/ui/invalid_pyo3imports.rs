#![cfg(all(feature = "testing", not(any(PyPy, GraalPy))))]
use pyo3::prelude::*;

#[pyfunction]
#[pyo3(name = "addone")]
fn py_addone(num: isize) -> isize {
    num + 1
}

#[pymodule]
#[pyo3(name = "adders")]
fn py_adders(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(py_addone, module)?)?;
    Ok(())
}

#[pyo3test]
#[pyo3import(py_adders: form adders import addone)]
fn typo_first_keyword() {}

#[pyo3test]
#[pyo3import(py_adders: from adders improt addone)]
fn typo_second_keyword() {}

fn main() {}