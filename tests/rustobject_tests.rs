extern crate cpython;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use cpython::*;

#[test]
fn rustobject_calls_drop() {

    struct MyObj {
        drop_called: Arc<AtomicBool>
    }
    impl Drop for MyObj {
        fn drop(&mut self) {
            self.drop_called.store(true, Ordering::Relaxed);
        }
    }

    let gil = Python::acquire_gil();
    let py = gil.python();
    let t = PyRustTypeBuilder::<MyObj>::new(py, "TypeWithDrop").finish().unwrap();

    let drop_called = Arc::new(AtomicBool::new(false));
    let inst = t.create_instance(MyObj { drop_called: drop_called.clone() }, ());
    assert!(drop_called.load(Ordering::Relaxed) == false);
    drop(inst);
    assert!(drop_called.load(Ordering::Relaxed) == true);
}


#[test]
fn rustobject_no_init_from_python() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let t = PyRustTypeBuilder::<i32>::new(py, "MyType").finish().unwrap();
    assert!(t.call(&NoArgs, None).is_err());
}


#[test]
fn rustobject_heaptype_refcount() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let t = PyRustTypeBuilder::<i32>::new(py, "MyType").finish().unwrap();
    // TODO: investigate why the refcnt isn't 1.
    //assert_eq!(1, t.as_object().get_refcnt());
    let old_refcnt = t.as_object().get_refcnt();
    let inst = t.create_instance(1, ());
    assert_eq!(old_refcnt + 1, t.as_object().get_refcnt());
    drop(inst);
    assert_eq!(old_refcnt, t.as_object().get_refcnt());
}

