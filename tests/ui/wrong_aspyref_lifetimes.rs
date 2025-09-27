use pyo3::{types::PyDict, Bound, Py, Python};

fn main() {
    let dict: Py<PyDict> = Python::attach(|py| PyDict::new(py).unbind());

    // Should not be able to get access to Py contents outside of `attach`.
    let dict: &Bound<'_, PyDict> = Python::attach(|py| dict.bind(py));

    let _py: Python = dict.py(); // Obtain a Python<'p> without GIL.
}
