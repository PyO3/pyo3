#![allow(dead_code)]

#[macro_use] extern crate cpython;

use cpython::{PyResult, Python, NoArgs, ObjectProtocol};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

py_class!(class EmptyClass |py| { });

#[test]
fn empty_class() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<EmptyClass>();
    // By default, don't allow creating instances from python.
    assert!(typeobj.call(py, NoArgs, None).is_err());
}

py_class!(class EmptyClassWithNew |py| {
    def __new__(_cls) -> PyResult<EmptyClassWithNew> {
        EmptyClassWithNew::create_instance(py)
    }
});

#[test]
fn empty_class_with_new() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<EmptyClassWithNew>();
    assert!(typeobj.call(py, NoArgs, None).unwrap().cast_into::<EmptyClassWithNew>(py).is_ok());
}

py_class!(class NewWithOneArg |py| {
    data _data: i32;
    def __new__(_cls, arg: i32) -> PyResult<NewWithOneArg> {
        NewWithOneArg::create_instance(py, arg)
    }
});

#[test]
fn new_with_one_arg() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<NewWithOneArg>();
    let obj = typeobj.call(py, (42,), None).unwrap().cast_into::<NewWithOneArg>(py).unwrap();
    assert_eq!(*obj._data(py), 42);
}

py_class!(class NewWithTwoArgs |py| {
    data _data1: i32;
    data _data2: i32;
    def __new__(_cls, arg1: i32, arg2: i32) -> PyResult<NewWithTwoArgs> {
        NewWithTwoArgs::create_instance(py, arg1, arg2)
    }
});

#[test]
fn new_with_two_args() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<NewWithTwoArgs>();
    let obj = typeobj.call(py, (10, 20), None).unwrap().cast_into::<NewWithTwoArgs>(py).unwrap();
    assert_eq!(*obj._data1(py), 10);
    assert_eq!(*obj._data2(py), 20);
}

struct TestDropCall {
    drop_called: Arc<AtomicBool>
}
impl Drop for TestDropCall {
    fn drop(&mut self) {
        self.drop_called.store(true, Ordering::Relaxed);
    }
}

py_class!(class DataIsDropped |py| {
    data member1: TestDropCall;
    data member2: TestDropCall;
});

#[test]
#[allow(dead_code)]
fn data_is_dropped() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let drop_called1 = Arc::new(AtomicBool::new(false));
    let drop_called2 = Arc::new(AtomicBool::new(false));
    let inst = DataIsDropped::create_instance(py,
        TestDropCall { drop_called: drop_called1.clone() },
        TestDropCall { drop_called: drop_called2.clone() });
    assert!(drop_called1.load(Ordering::Relaxed) == false);
    assert!(drop_called2.load(Ordering::Relaxed) == false);
    drop(inst);
    assert!(drop_called1.load(Ordering::Relaxed) == true);
    assert!(drop_called2.load(Ordering::Relaxed) == true);
}

