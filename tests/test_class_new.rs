#![cfg(feature = "macros")]

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass]
struct EmptyClassWithNew {}

#[pymethods]
impl EmptyClassWithNew {
    #[new]
    fn new() -> EmptyClassWithNew {
        EmptyClassWithNew {}
    }
}

#[test]
fn empty_class_with_new() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<EmptyClassWithNew>();
    assert!(typeobj
        .call((), None)
        .unwrap()
        .cast_as::<PyCell<EmptyClassWithNew>>()
        .is_ok());
}

#[pyclass]
struct UnitClassWithNew;

#[pymethods]
impl UnitClassWithNew {
    #[new]
    fn new() -> Self {
        Self
    }
}

#[test]
fn unit_class_with_new() {
    Python::with_gil(|py| {
        let typeobj = py.get_type::<UnitClassWithNew>();
        assert!(typeobj
            .call((), None)
            .unwrap()
            .cast_as::<PyCell<UnitClassWithNew>>()
            .is_ok());
    });
}

#[pyclass]
struct TupleClassWithNew(i32);

#[pymethods]
impl TupleClassWithNew {
    #[new]
    fn new(arg: i32) -> Self {
        Self(arg)
    }
}

#[test]
fn tuple_class_with_new() {
    Python::with_gil(|py| {
        let typeobj = py.get_type::<TupleClassWithNew>();
        let wrp = typeobj.call((42,), None).unwrap();
        let obj = wrp.cast_as::<PyCell<TupleClassWithNew>>().unwrap();
        let obj_ref = obj.borrow();
        assert_eq!(obj_ref.0, 42);
    });
}

#[pyclass]
#[derive(Debug)]
struct NewWithOneArg {
    _data: i32,
}

#[pymethods]
impl NewWithOneArg {
    #[new]
    fn new(arg: i32) -> NewWithOneArg {
        NewWithOneArg { _data: arg }
    }
}

#[test]
fn new_with_one_arg() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<NewWithOneArg>();
    let wrp = typeobj.call((42,), None).unwrap();
    let obj = wrp.cast_as::<PyCell<NewWithOneArg>>().unwrap();
    let obj_ref = obj.borrow();
    assert_eq!(obj_ref._data, 42);
}

#[pyclass]
struct NewWithTwoArgs {
    _data1: i32,
    _data2: i32,
}

#[pymethods]
impl NewWithTwoArgs {
    #[new]
    fn new(arg1: i32, arg2: i32) -> Self {
        NewWithTwoArgs {
            _data1: arg1,
            _data2: arg2,
        }
    }
}

#[test]
fn new_with_two_args() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<NewWithTwoArgs>();
    let wrp = typeobj
        .call((10, 20), None)
        .map_err(|e| e.print(py))
        .unwrap();
    let obj = wrp.cast_as::<PyCell<NewWithTwoArgs>>().unwrap();
    let obj_ref = obj.borrow();
    assert_eq!(obj_ref._data1, 10);
    assert_eq!(obj_ref._data2, 20);
}

#[pyclass(subclass)]
struct SuperClass {
    #[pyo3(get)]
    from_rust: bool,
}

#[pymethods]
impl SuperClass {
    #[new]
    fn new() -> Self {
        SuperClass { from_rust: true }
    }
}

/// Checks that `subclass.__new__` works correctly.
/// See https://github.com/PyO3/pyo3/issues/947 for the corresponding bug.
#[test]
fn subclass_new() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let super_cls = py.get_type::<SuperClass>();
    let source = pyo3::indoc::indoc!(
        r#"
class Class(SuperClass):
    def __new__(cls):
        return super().__new__(cls)  # This should return an instance of Class

    @property
    def from_rust(self):
        return False
c = Class()
assert c.from_rust is False
"#
    );
    let globals = PyModule::import(py, "__main__").unwrap().dict();
    globals.set_item("SuperClass", super_cls).unwrap();
    py.run(source, Some(globals), None)
        .map_err(|e| e.print(py))
        .unwrap();
}

#[pyclass]
#[derive(Debug)]
struct NewWithCustomError {}

struct CustomError;

impl From<CustomError> for PyErr {
    fn from(_error: CustomError) -> PyErr {
        PyValueError::new_err("custom error")
    }
}

#[pymethods]
impl NewWithCustomError {
    #[new]
    fn new() -> Result<NewWithCustomError, CustomError> {
        Err(CustomError)
    }
}

#[test]
fn new_with_custom_error() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<NewWithCustomError>();
    let err = typeobj.call0().unwrap_err();
    assert_eq!(err.to_string(), "ValueError: custom error");
}
