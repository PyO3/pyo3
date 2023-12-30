#![cfg(feature = "anyhow")]

use pyo3::prelude::PyDictMethods;

#[test]
fn test_anyhow_py_function_ok_result() {
    use pyo3::{py_run_bound, pyfunction, wrap_pyfunction, Python};

    #[pyfunction]
    #[allow(clippy::unnecessary_wraps)]
    fn produce_ok_result() -> anyhow::Result<String> {
        Ok(String::from("OK buddy"))
    }

    Python::with_gil(|py| {
        let func = wrap_pyfunction!(produce_ok_result)(py).unwrap();

        py_run_bound!(
            py,
            func,
            r#"
            func()
            "#
        );
    });
}

#[test]
fn test_anyhow_py_function_err_result() {
    use pyo3::{pyfunction, types::PyDict, wrap_pyfunction, Python};

    #[pyfunction]
    fn produce_err_result() -> anyhow::Result<String> {
        anyhow::bail!("error time")
    }

    Python::with_gil(|py| {
        let func = wrap_pyfunction!(produce_err_result)(py).unwrap();
        let locals = PyDict::new_bound(py);
        locals.set_item("func", func).unwrap();

        py.run_bound(
            r#"
            func()
            "#,
            None,
            Some(&locals),
        )
        .unwrap_err();
    });
}
