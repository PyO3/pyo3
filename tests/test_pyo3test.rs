#![cfg(all(feature = "macros", feature = "testing", not(any(PyPy, GraalPy))))]
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

// This is how the test would be written WITHOUT using the pyo3test macro. This validates that
// adders.addone is correctly constructed.
#[test]
fn test_pyo3test_without_macro() {
    pyo3::append_to_inittab!(py_adders);
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let adders = py.import_bound("adders").expect("Failed to import adders");
        let addone = adders
            .getattr("addone")
            .expect("Failed to get addone function");
        let result: PyResult<isize> = match addone.call1((1_isize,)) {
            Ok(r) => r.extract(),
            Err(e) => Err(e),
        };
        let result = result.unwrap();
        let expected_result = 2_isize;
        assert_eq!(result, expected_result);
    });
}

// ... and this is how the test can be written using the pyo3test macro and pyo3import attribute
#[pyo3test]
#[pyo3import(py_adders: from adders import addone)]
fn test_pyo3test_simple_case() {
    let result: PyResult<isize> = match addone.call1((1_isize,)) {
        Ok(r) => r.extract(),
        Err(e) => Err(e),
    };
    let result = result.unwrap();
    let expected_result = 2_isize;
    assert_eq!(result, expected_result);
}
