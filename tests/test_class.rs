#![allow(dead_code, unused_variables)]

#[macro_use] extern crate cpython;

use cpython::{PyObject, PythonObject, PyDrop, PyClone, PyResult, Python, NoArgs, ObjectProtocol, PyDict};
use std::mem;
use std::cell::RefCell;
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

py_class!(class InstanceMethod |py| {
    data member: i32;

    def method(&self) -> PyResult<i32> {
        Ok(*self.member(py))
    }
});

#[test]
fn instance_method() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = InstanceMethod::create_instance(py, 42).unwrap();
    assert!(obj.method(py).unwrap() == 42);
    let d = PyDict::new(py);
    d.set_item(py, "obj", obj).unwrap();
    py.run("assert obj.method() == 42", None, Some(&d)).unwrap();
}

py_class!(class InstanceMethodWithArgs |py| {
    data member: i32;

    def method(&self, multiplier: i32) -> PyResult<i32> {
        Ok(*self.member(py) * multiplier)
    }
});

#[test]
fn instance_method_with_args() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = InstanceMethodWithArgs::create_instance(py, 7).unwrap();
    assert!(obj.method(py, 6).unwrap() == 42);
    let d = PyDict::new(py);
    d.set_item(py, "obj", obj).unwrap();
    py.run("assert obj.method(3) == 21", None, Some(&d)).unwrap();
    py.run("assert obj.method(multiplier=6) == 42", None, Some(&d)).unwrap();
}

py_class!(class ClassMethod |py| {
    def __new__(cls) -> PyResult<ClassMethod> {
        ClassMethod::create_instance(py)
    }

    @classmethod
    def method(cls) -> PyResult<String> {
        Ok(format!("{}.method()!", cls.name(py)))
    }
});

#[test]
fn class_method() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = PyDict::new(py);
    d.set_item(py, "C", py.get_type::<ClassMethod>()).unwrap();
    py.run("assert C.method() == 'ClassMethod.method()!'", None, Some(&d)).unwrap();
    py.run("assert C().method() == 'ClassMethod.method()!'", None, Some(&d)).unwrap();
}

py_class!(class ClassMethodWithArgs |py| {
    @classmethod
    def method(cls, input: &str) -> PyResult<String> {
        Ok(format!("{}.method({})", cls.name(py), input))
    }
});

#[test]
fn class_method_with_args() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = PyDict::new(py);
    d.set_item(py, "C", py.get_type::<ClassMethodWithArgs>()).unwrap();
    py.run("assert C.method('abc') == 'ClassMethodWithArgs.method(abc)'", None, Some(&d)).unwrap();
}

py_class!(class StaticMethod |py| {
    def __new__(cls) -> PyResult<StaticMethod> {
        StaticMethod::create_instance(py)
    }

    @staticmethod
    def method() -> PyResult<&'static str> {
        Ok("StaticMethod.method()!")
    }
});

#[test]
fn static_method() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    assert_eq!(StaticMethod::method(py).unwrap(), "StaticMethod.method()!");
    let d = PyDict::new(py);
    d.set_item(py, "C", py.get_type::<StaticMethod>()).unwrap();
    py.run("assert C.method() == 'StaticMethod.method()!'", None, Some(&d)).unwrap();
    py.run("assert C().method() == 'StaticMethod.method()!'", None, Some(&d)).unwrap();
}

py_class!(class StaticMethodWithArgs |py| {
    @staticmethod
    def method(input: i32) -> PyResult<String> {
        Ok(format!("0x{:x}", input))
    }
});

#[test]
fn static_method_with_args() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    assert_eq!(StaticMethodWithArgs::method(py, 1234).unwrap(), "0x4d2");
    let d = PyDict::new(py);
    d.set_item(py, "C", py.get_type::<StaticMethodWithArgs>()).unwrap();
    py.run("assert C.method(1337) == '0x539'", None, Some(&d)).unwrap();
}

py_class!(class StaticData |py| {
    static VAL1 = 123;
    static VAL2 = py.None();
});

#[test]
fn static_data() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let d = PyDict::new(py);
    d.set_item(py, "C", py.get_type::<StaticData>()).unwrap();
    py.run("assert C.VAL1 == 123", None, Some(&d)).unwrap();
    py.run("assert C.VAL2 is None", None, Some(&d)).unwrap();
    assert!(py.run("C.VAL1 = 124", None, Some(&d)).is_err());
}

py_class!(class GCIntegration |py| {
    data self_ref: RefCell<PyObject>;
    data dropped: TestDropCall;

    def __traverse__(&self, visit) {
        visit.call(&*self.self_ref(py).borrow())
    }

    def __clear__(&self) {
        let old_ref = mem::replace(&mut *self.self_ref(py).borrow_mut(), py.None());
        // Release reference only after the mutable borrow has expired.
        old_ref.release_ref(py);
    }
});

#[test]
fn gc_integration() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let drop_called = Arc::new(AtomicBool::new(false));
    let inst = GCIntegration::create_instance(py,
        RefCell::new(py.None()),
        TestDropCall { drop_called: drop_called.clone() }
    ).unwrap();
    *inst.self_ref(py).borrow_mut() = inst.as_object().clone_ref(py);
    inst.release_ref(py);

    py.run("import gc; gc.collect()", None, None).unwrap();
    assert!(drop_called.load(Ordering::Relaxed));
}

