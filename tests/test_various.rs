#![feature(specialization)]

extern crate pyo3;

use pyo3::prelude::*;
use std::isize;

#[macro_use]
mod common;

#[pyclass]
struct MutRefArg {
    n: i32,
    token: PyToken,
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
    let inst1 = py.init(|t| MutRefArg { token: t, n: 0 }).unwrap();
    let inst2 = py.init(|t| MutRefArg { token: t, n: 0 }).unwrap();

    let d = PyDict::new(py);
    d.set_item("inst1", &inst1).unwrap();
    d.set_item("inst2", &inst2).unwrap();

    py.run("inst1.set_other(inst2)", None, Some(d)).unwrap();
    assert_eq!(inst2.as_ref(py).n, 100);
}
