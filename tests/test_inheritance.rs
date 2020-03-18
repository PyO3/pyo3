use pyo3::prelude::*;
use pyo3::py_run;

use pyo3::types::IntoPyDict;

use pyo3::types::{PyDict, PySet};
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
    fn new() -> Self {
        BaseClass { val1: 10 }
    }
    fn base_method(&self, x: usize) -> usize {
        x * self.val1
    }
    fn base_set(&mut self, fn_: &pyo3::PyAny) -> PyResult<()> {
        let value: usize = fn_.call0()?.extract()?;
        self.val1 = value;
        Ok(())
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
    fn new() -> (Self, BaseClass) {
        (SubClass { val2: 5 }, BaseClass { val1: 10 })
    }
    fn sub_method(&self, x: usize) -> usize {
        x * self.val2
    }
    fn sub_set_and_ret(&mut self, x: usize) -> usize {
        self.val2 = x;
        x
    }
}

#[test]
fn inheritance_with_new_methods() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<SubClass>();
    let inst = typeobj.call((), None).unwrap();
    py_run!(py, inst, "assert inst.val1 == 10; assert inst.val2 == 5");
}

#[test]
fn call_base_and_sub_methods() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let obj = PyCell::new(py, SubClass::new()).unwrap();
    py_run!(
        py,
        obj,
        r#"
    assert obj.base_method(10) == 100
    assert obj.sub_method(10) == 50
"#
    );
}

#[test]
fn mutation_fails() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let obj = PyCell::new(py, SubClass::new()).unwrap();
    let global = Some([("obj", obj)].into_py_dict(py));
    let e = py
        .run("obj.base_set(lambda: obj.sub_set_and_ret(1))", global, None)
        .unwrap_err();
    assert!(e.is_instance::<pyo3::pycell::PyBorrowMutError>(py))
}

#[pyclass]
struct BaseClassWithResult {
    _val: usize,
}

#[pymethods]
impl BaseClassWithResult {
    #[new]
    fn new(value: isize) -> PyResult<Self> {
        Ok(Self {
            _val: std::convert::TryFrom::try_from(value)?,
        })
    }
}

#[pyclass(extends=BaseClassWithResult)]
struct SubClass2 {}

#[pymethods]
impl SubClass2 {
    #[new]
    fn new(value: isize) -> PyResult<(Self, BaseClassWithResult)> {
        let base = BaseClassWithResult::new(value)?;
        Ok((Self {}, base))
    }
}

#[test]
fn handle_result_in_new() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let subclass = py.get_type::<SubClass2>();
    py_run!(
        py,
        subclass,
        r#"
try:
    subclass(-10)
    assert Fals
except ValueError as e:
    pass
except Exception as e:
    raise e
"#
    );
}

#[pyclass(extends=PySet)]
#[derive(Debug)]
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
    let set_sub = pyo3::PyCell::new(py, SetWithName::new()).unwrap();
    py_run!(
        py,
        set_sub,
        r#"set_sub.add(10); assert list(set_sub) == [10]; assert set_sub._name == "Hello :)""#
    );
}

#[pyclass(extends=PyDict)]
#[derive(Debug)]
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
    let dict_sub = pyo3::PyCell::new(py, DictWithName::new()).unwrap();
    py_run!(
        py,
        dict_sub,
        r#"dict_sub[0] = 1; assert dict_sub[0] == 1; assert dict_sub._name == "Hello :)""#
    );
}
