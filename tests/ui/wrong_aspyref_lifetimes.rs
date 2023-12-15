use pyo3::{types::PyDict, PyDetached, Python};

fn main() {
    let dict: PyDetached<PyDict> = Python::with_gil(|py| PyDict::new(py).into());

    // Should not be able to get access to Py contents outside of with_gil.
    let dict: &PyDict = Python::with_gil(|py| dict.as_ref(py));

    let _py: Python = dict.py(); // Obtain a Python<'p> without GIL.
}
