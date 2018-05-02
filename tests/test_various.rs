#![feature(proc_macro, specialization)]

extern crate pyo3;

use pyo3::prelude::*;
use std::isize;

use pyo3::py::class as pyclass;
use pyo3::py::methods as pymethods;

#[macro_use]
mod common;


#[pyclass(dict)]
struct DunderDictSupport {
    token: PyToken,
}

#[test]
fn dunder_dict_support() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst = Py::new_ref(py, |t| DunderDictSupport{token: t}).unwrap();
    py_run!(py, inst, "inst.a = 1; assert inst.a == 1");
}

#[pyclass(weakref, dict)]
struct WeakRefDunderDictSupport {
    token: PyToken,
}

#[test]
fn weakref_dunder_dict_support() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let inst = Py::new_ref(py, |t| WeakRefDunderDictSupport{token: t}).unwrap();
    py_run!(py, inst, "import weakref; assert weakref.ref(inst)() is inst; inst.a = 1; assert inst.a == 1");
}


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
    let inst1 = py.init(|t| MutRefArg{token: t, n: 0}).unwrap();
    let inst2 = py.init(|t| MutRefArg{token: t, n: 0}).unwrap();

    let d = PyDict::new(py);
    d.set_item("inst1", &inst1).unwrap();
    d.set_item("inst2", &inst2).unwrap();

    py.run("inst1.set_other(inst2)", None, Some(d)).unwrap();
    assert_eq!(inst2.as_ref(py).n, 100);
}
