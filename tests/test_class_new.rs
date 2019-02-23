use pyo3::prelude::*;
use pyo3::PyRawObject;

#[pyclass]
struct EmptyClassWithNew {}

#[pymethods]
impl EmptyClassWithNew {
    #[new]
    fn new(obj: &PyRawObject) {
        obj.init(EmptyClassWithNew {});
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
        .cast_as::<EmptyClassWithNew>()
        .is_ok());
}

#[pyclass]
struct NewWithOneArg {
    _data: i32,
}

#[pymethods]
impl NewWithOneArg {
    #[new]
    fn new(obj: &PyRawObject, arg: i32) {
        obj.init(NewWithOneArg { _data: arg })
    }
}

#[test]
fn new_with_one_arg() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<NewWithOneArg>();
    let wrp = typeobj.call((42,), None).unwrap();
    let obj = wrp.cast_as::<NewWithOneArg>().unwrap();
    assert_eq!(obj._data, 42);
}

#[pyclass]
struct NewWithTwoArgs {
    _data1: i32,
    _data2: i32,
}

#[pymethods]
impl NewWithTwoArgs {
    #[new]
    fn new(obj: &PyRawObject, arg1: i32, arg2: i32) {
        obj.init(NewWithTwoArgs {
            _data1: arg1,
            _data2: arg2,
        })
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
    let obj = wrp.cast_as::<NewWithTwoArgs>().unwrap();
    assert_eq!(obj._data1, 10);
    assert_eq!(obj._data2, 20);
}
