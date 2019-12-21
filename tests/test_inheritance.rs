use pyo3::prelude::*;
use pyo3::py_run;

#[cfg(feature = "unsound-subclass")]
use pyo3::types::IntoPyDict;

use pyo3::types::{PyDict, PySet};
mod common;

#[pyclass]
struct BaseClass {
    #[pyo3(get)]
    val1: usize,
}

#[cfg(feature = "unsound-subclass")]
#[pyclass(subclass)]
struct SubclassAble {}

#[cfg(feature = "unsound-subclass")]
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
    fn new() -> Self {
        BaseClass { val1: 10 }
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
    fn new() -> PyClassInitializer<Self> {
        let mut init = PyClassInitializer::from_value(SubClass { val2: 5 });
        init.get_super().init(BaseClass { val1: 10 });
        init
    }
}

#[test]
fn inheritance_with_new_methods() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let _baseobj = py.get_type::<BaseClass>();
    let typeobj = py.get_type::<SubClass>();
    let inst = typeobj.call((), None).unwrap();
    py_run!(py, inst, "assert inst.val1 == 10; assert inst.val2 == 5");
}

#[pyclass(extends=BaseClass)]
struct InvalidSubClass {
    #[pyo3(get)]
    val2: usize,
}

#[pymethods]
impl InvalidSubClass {
    #[new]
    fn new() -> PyClassInitializer<Self> {
        PyClassInitializer::from_value(InvalidSubClass { val2: 5 })
    }
}

#[test]
fn uninit_baseclass_raise_exception() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let _baseclass = py.get_type::<BaseClass>();
    let subclass = py.get_type::<InvalidSubClass>();
    py_expect_exception!(py, subclass, "subclass()", RuntimeError);
}

#[test]
fn uninit_baseclass_returns_err() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let subclass = pyo3::pyclass::PyClassShell::new_ref(py, InvalidSubClass { val2: 5 });
    if let Err(err) = subclass {
        py_run!(
            py,
            err,
            r#"str(err) == "Base class 'BaseClass' is not initialized""#
        )
    } else {
        panic!("Uninitialized class detection failed!!!")
    }
}

#[pyclass(extends=PySet)]
struct SetWithName {
    #[pyo3(get(name))]
    _name: &'static str,
}

#[pymethods]
impl SetWithName {
    #[new]
    fn new() -> Self {
        SetWithName { _name: "Hello :)" }
    }
}

#[test]
fn inherit_set() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let set_sub = pyo3::pyclass::PyClassShell::new_ref(py, SetWithName::new()).unwrap();
    py_run!(
        py,
        set_sub,
        r#"set_sub.add(10); assert list(set_sub) == [10]; assert set_sub._name == "Hello :)""#
    );
}

#[pyclass(extends=PyDict)]
struct DictWithName {
    #[pyo3(get(name))]
    _name: &'static str,
}

#[pymethods]
impl DictWithName {
    #[new]
    fn new() -> Self {
        DictWithName { _name: "Hello :)" }
    }
}

#[test]
fn inherit_dict() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let dict_sub = pyo3::pyclass::PyClassShell::new_ref(py, DictWithName::new()).unwrap();
    py_run!(
        py,
        dict_sub,
        r#"dict_sub[0] = 1; assert dict_sub[0] == 1; assert dict_sub._name == "Hello :)""#
    );
}
