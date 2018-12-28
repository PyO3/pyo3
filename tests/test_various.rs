#![feature(specialization)]

#[macro_use]
extern crate pyo3;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::isize;

#[macro_use]
mod common;

#[pyclass]
struct MutRefArg {
    n: i32,
}

#[pymethods]
impl MutRefArg {
    fn get(&self) -> PyResult<i32> {
        Ok(self.n)
    }
    fn set_other(&self, other: &mut MutRefArg) -> PyResult<()> {
        other.n = 100;
        Ok(())
    }
}

#[test]
fn mut_ref_arg() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst1 = py.init(|| MutRefArg { n: 0 }).unwrap();
    let inst2 = py.init(|| MutRefArg { n: 0 }).unwrap();

    let d = PyDict::new(py);
    d.set_item("inst1", &inst1).unwrap();
    d.set_item("inst2", &inst2).unwrap();

    py.run("inst1.set_other(inst2)", None, Some(d)).unwrap();
    assert_eq!(inst2.as_ref(py).n, 100);
}

#[pyclass]
struct PyUsize {
    #[prop(get)]
    pub value: usize,
}

#[pyfunction]
fn get_zero() -> PyResult<PyUsize> {
    Ok(PyUsize { value: 0 })
}

#[test]
/// Checks that we can use return a custom class in arbitrary function and use those functions
/// both in rust and python
fn return_custom_class() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    // Using from rust
    assert_eq!(get_zero().unwrap().value, 0);

    // Using from python
    let get_zero = wrap_function!(get_zero)(py);
    py_assert!(py, get_zero, "get_zero().value == 0");
}
