use pyo3::{types::PyDict, Py, Python};

fn main() {
    let dict: Py<PyDict> = Python::with_gil(|py| PyDict::new_bound(py).into());

    // Should not be able to get access to Py contents outside of with_gil.
    let dict: &PyDict = Python::with_gil(|py| dict.as_ref(py));

    let _py: Python = dict.py(); // Obtain a Python<'p> without GIL.
}
