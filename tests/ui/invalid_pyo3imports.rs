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

#[pyo3test]
#[pyo3import]
fn empty_import() {}

#[pyo3test]
#[pyo3import(py_adders from adders import addone)]
fn no_colon() {}

#[pyo3test]
#[pyo3import(py_adders: from adders import)]
fn missing_function() {}

// This will compile fine with trybuild due to the #[test] which is added to the
// wrapped function. see https://github.com/dtolnay/trybuild/issues/231
//
// Only when actually trying to run the test will you get an error:
// error[E0433]: failed to resolve: use of undeclared crate or module `py_addrs`
// --> tests/ui/invalid_pyo3imports.rs:47:14
//    |
// 47 | #[pyo3import(py_addrs: from adders import addone)]
//    |              ^^^^^^^^ use of undeclared crate or module `py_addrs`
#[pyo3test]
#[pyo3import(py_addrs: from adders import addone)]
fn invalid_rust_method() {}

// This passes without error ... for copy-pasting ;)
#[pyo3test]
#[pyo3import(py_adders: from adders import addone)]
fn good_example() {}

fn main() {}
