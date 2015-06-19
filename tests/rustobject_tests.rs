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
    let inst = t.create_instance(MyObj { drop_called: drop_called.clone() });
    assert!(drop_called.load(Ordering::Relaxed) == false);
    drop(inst);
    assert!(drop_called.load(Ordering::Relaxed) == true);
}


#[test]
fn rustobject_no_init_from_python() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let t = PyRustTypeBuilder::<i32>::new(py, "TypeWithDrop").finish().unwrap();
    assert!(t.call(&NoArgs, None).is_err());
}

