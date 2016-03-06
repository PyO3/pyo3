// Copyright (c) 2015 Daniel Grunwald
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this
// software and associated documentation files (the "Software"), to deal in the Software
// without restriction, including without limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
// to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
// FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use {Python, NoArgs, PythonObject, rustobject};

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
    let t = rustobject::PyRustTypeBuilder::<MyObj>::new(py, "TypeWithDrop")
        .finish().unwrap();

    let drop_called = Arc::new(AtomicBool::new(false));
    let inst = t.create_instance(py, MyObj { drop_called: drop_called.clone() }, ());
    assert!(drop_called.load(Ordering::Relaxed) == false);
    drop(inst);
    assert!(drop_called.load(Ordering::Relaxed) == true);
}


#[test]
fn no_init_from_python() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let t = rustobject::PyRustTypeBuilder::<i32>::new(py, "MyType")
        .finish().unwrap();
    assert!(t.call(py, &NoArgs, None).is_err());
}


#[test]
fn heaptype_refcount() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let t = rustobject::PyRustTypeBuilder::<i32>::new(py, "MyType")
        .finish().unwrap();
    // TODO: investigate why the refcnt isn't 1.
    //assert_eq!(1, t.as_object().get_refcnt());
    let old_refcnt = t.as_object().get_refcnt(py);
    let inst = t.create_instance(py, 1, ());
    assert_eq!(old_refcnt + 1, t.as_object().get_refcnt(py));
    drop(inst);
    assert_eq!(old_refcnt, t.as_object().get_refcnt(py));
}

