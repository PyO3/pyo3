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
