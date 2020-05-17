use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

mod common;

#[pyfunction(arg = "true")]
fn optional_bool(arg: Option<bool>) -> String {
    format!("{:?}", arg)
}

#[test]
fn test_optional_bool() {
    // Regression test for issue #932
    let gil = Python::acquire_gil();
    let py = gil.python();
    let f = wrap_pyfunction!(optional_bool)(py);

    py_assert!(py, f, "f() == 'Some(true)'");
    py_assert!(py, f, "f(True) == 'Some(true)'");
    py_assert!(py, f, "f(False) == 'Some(false)'");
    py_assert!(py, f, "f(None) == 'None'");
}
