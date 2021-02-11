use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

#[pyfunction]
fn issue_219() {
    // issue 219: acquiring GIL inside #[pyfunction] deadlocks.
    let gil = Python::acquire_gil();
    let _py = gil.python();
}

#[pymodule]
fn misc(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(issue_219, m)?)?;
    Ok(())
}
