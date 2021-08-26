use crate::exceptions::PyRuntimeError;
use crate::PyErr;

impl From<anyhow::Error> for PyErr {
    fn from(err: anyhow::Error) -> Self {
        PyRuntimeError::new_err(format!("{:?}", err))
    }
}

#[cfg(test)]
mod test_anyhow {
    use crate::proc_macro::pyfunction;
    use crate::py_run;
    use crate::wrap_pyfunction;
    use crate::{Python, ToPyObject};

    #[pyfunction]
    fn produce_ok_result() -> anyhow::Result<String> {
        Ok(String::from("OK buddy"))
    }

    #[pyfunction]
    fn produce_err_result() -> anyhow::Result<String> {
        anyhow::bail!("error time")
    }

    #[test]
    fn test_anyhow_py_function_ok_result() {
        Python::with_gil(|py| {
            let func = wrap_pyfunction!(produce_ok_result)(py).unwrap();

            py_run!(
                py,
                func,
                r#"
            func()
            "#
            );
        });
    }
}
