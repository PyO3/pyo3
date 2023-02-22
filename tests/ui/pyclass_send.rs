use pyo3::prelude::*;
use std::rc::Rc;

#[pyclass]
struct NotThreadSafe {
    data: Rc<i32>,
}

fn main() {
    let obj = Python::with_gil(|py| {
        PyCell::new(py, NotThreadSafe { data: Rc::new(5) })
            .unwrap()
            .to_object(py)
    });

    std::thread::spawn(move || {
        Python::with_gil(|py| {
            // Uh oh, moved Rc to a new thread!
            let c: &PyCell<NotThreadSafe> = obj.as_ref(py).downcast().unwrap();

            assert_eq!(*c.borrow().data, 5);
        })
    })
    .join()
    .unwrap();
}
