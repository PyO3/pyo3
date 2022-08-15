use pyo3::prelude::*;

#[pyfunction]
fn issue_219() {
    // issue 219: acquiring GIL inside #[pyfunction] deadlocks.
    #[allow(deprecated)]
    let gil = Python::acquire_gil();
    let _py = gil.python();
}

#[pyfunction]
fn issue_219_2() {
    // issue 219: acquiring GIL inside #[pyfunction] deadlocks.
    Python::with_gil(|_| {});
}

#[pymodule]
pub fn misc(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(issue_219, m)?)?;
    m.add_function(wrap_pyfunction!(issue_219_2, m)?)?;
    Ok(())
}
