use pyo3::{types::PyDict, Py, Python};

fn main() {
    #[allow(deprecated)]
    let gil = Python::acquire_gil();
    let dict: Py<PyDict> = PyDict::new(gil.python()).into();
    let dict: &PyDict = dict.as_ref(gil.python());
    drop(gil);

    let _py: Python = dict.py(); // Obtain a Python<'p> without GIL.
}
