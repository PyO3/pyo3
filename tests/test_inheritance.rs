#![feature(proc_macro, specialization)]

extern crate pyo3;

use pyo3::prelude::*;
use std::isize;

use pyo3::{pyclass, pymethods};

#[macro_use]
mod common;

#[pyclass]
struct BaseClass {
    #[prop(get)]
    val1: usize,
}

#[pyclass(subclass)]
struct SubclassAble {}

#[test]
fn subclass() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = PyDict::new(py);
    d.set_item("SubclassAble", py.get_type::<SubclassAble>())
        .unwrap();
    py.run(
        "class A(SubclassAble): pass\nassert issubclass(A, SubclassAble)",
        None,
        Some(d),
    ).map_err(|e| e.print(py))
        .unwrap();
}

#[pymethods]
impl BaseClass {
    #[new]
    fn __new__(obj: &PyRawObject) -> PyResult<()> {
        obj.init(|_| BaseClass { val1: 10 })
    }
}

#[pyclass(base=BaseClass)]
struct SubClass {
    #[prop(get)]
    val2: usize,
}

#[pymethods]
impl SubClass {
    #[new]
    fn __new__(obj: &PyRawObject) -> PyResult<()> {
        obj.init(|_| SubClass { val2: 5 })?;
        BaseClass::__new__(obj)
    }
}

#[test]
fn inheritance_with_new_methods() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let _typebase = py.get_type::<BaseClass>();
    let typeobj = py.get_type::<SubClass>();
    let inst = typeobj.call(NoArgs, NoArgs).unwrap();
    py_run!(py, inst, "assert inst.val1 == 10; assert inst.val2 == 5");
}
