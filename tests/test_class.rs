#[macro_use] extern crate cpython;

use cpython::{PyResult, Python, NoArgs, ObjectProtocol, PyDict};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[test]
fn empty_class() {
    py_class!(class Empty |py| { });

    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<Empty>();
    // By default, don't allow creating instances from python.
    assert!(typeobj.call(py, NoArgs, None).is_err());
}

#[test]
fn empty_class_with_new() {
    py_class!(class Empty |py| {
        /*def __new__(cls) -> PyResult<Empty> {
            Ok(Empty::create_instance(py))
        }*/
    });

    let gil = Python::acquire_gil();
    let py = gil.python();
    let typeobj = py.get_type::<Empty>();
    assert!(typeobj.call(py, NoArgs, None).unwrap().cast_into::<Empty>(py).is_ok());
}

#[test]
fn data_is_dropped() {
    struct MyObj {
        drop_called: Arc<AtomicBool>
    }
    impl Drop for MyObj {
        fn drop(&mut self) {
            self.drop_called.store(true, Ordering::Relaxed);
        }
    }

    py_class!(class C |py| {
        data member1: MyObj;
        data member2: MyObj;
    });

    let gil = Python::acquire_gil();
    let py = gil.python();

    let drop_called1 = Arc::new(AtomicBool::new(false));
    let drop_called2 = Arc::new(AtomicBool::new(false));
    let inst = C::create_instance(py,
        MyObj { drop_called: drop_called1.clone() },
        MyObj { drop_called: drop_called2.clone() });
    assert!(drop_called1.load(Ordering::Relaxed) == false);
    assert!(drop_called2.load(Ordering::Relaxed) == false);
    drop(inst);
    assert!(drop_called1.load(Ordering::Relaxed) == true);
    assert!(drop_called2.load(Ordering::Relaxed) == true);
}

