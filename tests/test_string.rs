use pyo3::prelude::*;
use pyo3::py_run;
use pyo3::wrap_pyfunction;

mod common;

#[pyfunction]
fn take_str(_s: &str) -> PyResult<()> {
    Ok(())
}

#[test]
fn test_unicode_encode_error() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let take_str = wrap_pyfunction!(take_str)(py);
    py_run!(
        py,
        take_str,
        r#"
        try:
            take_str('\ud800')
        except UnicodeEncodeError as e:
            error_msg = "'utf-8' codec can't encode character '\\ud800' in position 0: surrogates not allowed"
            assert str(e) == error_msg
        "#
    );
}
