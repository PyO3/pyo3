use pyo3::prelude::*;
use pyo3::py_run;
use pyo3::types::IntoPyDict;
use std::isize;

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
    let d = [("SubclassAble", py.get_type::<SubclassAble>())].into_py_dict(py);

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
    fn new(obj: &PyRawObject) {
        obj.init(BaseClass { val1: 10 })
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
    fn new(obj: &PyRawObject) {
        obj.init(SubClass { val2: 5 });
        BaseClass::new(obj);
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
