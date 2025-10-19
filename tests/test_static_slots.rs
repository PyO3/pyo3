#![cfg(feature = "macros")]

use pyo3::exceptions::PyIndexError;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

use pyo3::py_run;

mod test_utils;

#[pyclass]
struct Count5();

#[pymethods]
impl Count5 {
    #[new]
    fn new() -> Self {
        Self()
    }

    #[staticmethod]
    fn __len__() -> usize {
        5
    }

    #[staticmethod]
    fn __getitem__(idx: isize) -> PyResult<f64> {
        if idx < 0 {
            Err(PyIndexError::new_err("Count5 cannot count backwards"))
        } else if idx > 4 {
            Err(PyIndexError::new_err("Count5 cannot count higher than 5"))
        } else {
            Ok(idx as f64 + 1.0)
        }
    }
}

/// Return a dict with `s = Count5()`.
fn test_dict(py: Python<'_>) -> Bound<'_, pyo3::types::PyDict> {
    let d = [("Count5", py.get_type::<Count5>())]
        .into_py_dict(py)
        .unwrap();
    // Though we can construct `s` in Rust, let's test `__new__` works.
    py_run!(py, *d, "s = Count5()");
    d
}

#[test]
fn test_len() {
    Python::attach(|py| {
        let d = test_dict(py);

        py_assert!(py, *d, "len(s) == 5");
    });
}

#[test]
fn test_getitem() {
    Python::attach(|py| {
        let d = test_dict(py);

        py_assert!(py, *d, "s[4] == 5.0");
    });
}

#[test]
fn test_list() {
    Python::attach(|py| {
        let d = test_dict(py);

        py_assert!(py, *d, "list(s) == [1.0, 2.0, 3.0, 4.0, 5.0]");
    });
}
