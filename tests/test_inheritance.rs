use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::isize;

#[macro_use]
mod common;

#[pyclass]
struct BaseClass {
    #[pyo3(get)]
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
    )
    .map_err(|e| e.print(py))
    .unwrap();
}

#[pymethods]
impl BaseClass {
    #[new]
    fn __new__(obj: &PyRawObject) -> PyResult<()> {
        Ok(obj.init(BaseClass { val1: 10 }))
    }
}

#[pyclass(extends=BaseClass)]
struct SubClass {
    #[pyo3(get)]
    val2: usize,
}

#[pymethods]
impl SubClass {
    #[new]
    fn __new__(obj: &PyRawObject) -> PyResult<()> {
        obj.init(SubClass { val2: 5 });
        BaseClass::__new__(obj)
    }
}

#[test]
fn inheritance_with_new_methods() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let _typebase = py.get_type::<BaseClass>();
    let typeobj = py.get_type::<SubClass>();
    let inst = typeobj.call((), None).unwrap();
    py_run!(py, inst, "assert inst.val1 == 10; assert inst.val2 == 5");
}
