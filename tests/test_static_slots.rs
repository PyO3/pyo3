#![cfg(feature = "macros")]

use pyo3::exceptions::PyIndexError;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

use pyo3::py_run;

mod common;

#[pyclass]
struct Vector3 {
    elements: [f64; 3],
}

#[pymethods]
impl Vector3 {
    #[new]
    fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            elements: [x, y, z],
        }
    }

    #[staticmethod]
    fn __len__() -> usize {
        3
    }

    fn __getitem__(&self, idx: isize) -> PyResult<f64> {
        self.elements
            .get(idx as usize)
            .copied()
            .ok_or_else(|| PyIndexError::new_err("list index out of range"))
    }

    fn __setitem__(&mut self, idx: isize, value: f64) {
        self.elements[idx as usize] = value;
    }
}

/// Return a dict with `s = Vector3(1, 2, 3)`.
fn seq_dict(py: Python<'_>) -> &pyo3::types::PyDict {
    let d = [("Vector3", py.get_type::<Vector3>())].into_py_dict(py);
    // Though we can construct `s` in Rust, let's test `__new__` works.
    py_run!(py, *d, "s = Vector3(1, 2, 3)");
    d
}

#[test]
fn test_len() {
    Python::with_gil(|py| {
        let d = seq_dict(py);

        py_assert!(py, *d, "len(s) == 3");
    });
}
