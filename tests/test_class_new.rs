#![feature(proc_macro, specialization)]

extern crate pyo3;

use pyo3::prelude::*;

use pyo3::py::class;
use pyo3::py::methods;

#[class]
struct EmptyClassWithNew {
    token: PyToken,
}

#[methods]
impl EmptyClassWithNew {
    #[__new__]
    fn __new__(obj: &PyRawObject) -> PyResult<()> {
        obj.init(|t| EmptyClassWithNew { token: t })
    }
}

#[test]
fn empty_class_with_new() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<EmptyClassWithNew>();
    assert!(
        typeobj
            .call(NoArgs, NoArgs)
            .unwrap()
            .cast_as::<EmptyClassWithNew>()
            .is_ok()
    );
}

#[class]
struct NewWithOneArg {
    _data: i32,
    token: PyToken,
}

#[methods]
impl NewWithOneArg {
    #[new]
    fn __new__(obj: &PyRawObject, arg: i32) -> PyResult<()> {
        obj.init(|t| NewWithOneArg {
            _data: arg,
            token: t,
        })
    }
}

#[test]
fn new_with_one_arg() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<NewWithOneArg>();
    let wrp = typeobj.call((42,), NoArgs).unwrap();
    let obj = wrp.cast_as::<NewWithOneArg>().unwrap();
    assert_eq!(obj._data, 42);
}

#[class]
struct NewWithTwoArgs {
    _data1: i32,
    _data2: i32,

    token: PyToken,
}

#[methods]
impl NewWithTwoArgs {
    #[new]
    fn __new__(obj: &PyRawObject, arg1: i32, arg2: i32) -> PyResult<()> {
        obj.init(|t| NewWithTwoArgs {
            _data1: arg1,
            _data2: arg2,
            token: t,
        })
    }
}

#[test]
fn new_with_two_args() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<NewWithTwoArgs>();
    let wrp = typeobj
        .call((10, 20), NoArgs)
        .map_err(|e| e.print(py))
        .unwrap();
    let obj = wrp.cast_as::<NewWithTwoArgs>().unwrap();
    assert_eq!(obj._data1, 10);
    assert_eq!(obj._data2, 20);
}
