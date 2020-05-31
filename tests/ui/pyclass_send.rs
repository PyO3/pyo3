use pyo3::prelude::*;
use std::rc::Rc;

#[pyclass]
struct NotThreadSafe {
    data: Rc<i32>
}

fn main() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = PyCell::new(py, NotThreadSafe { data: Rc::new(5) }).unwrap().to_object(py);
    drop(gil);

    std::thread::spawn(move || {
        let gil = Python::acquire_gil();
        let py = gil.python();

        // Uh oh, moved Rc to a new thread!
        let c: &PyCell<NotThreadSafe> = obj.as_ref(py).downcast().unwrap();

        assert_eq!(*c.borrow().data, 5);
    }).join().unwrap();
}
