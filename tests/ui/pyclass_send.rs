use pyo3::prelude::*;
use std::rc::Rc;

#[pyclass]
struct NotThreadSafe {
    data: Rc<i32>,
}

fn main() {
    let obj = Python::with_gil(|py| {
        Bound::new(py, NotThreadSafe { data: Rc::new(5) })
            .unwrap()
            .unbind(py)
    });

    std::thread::spawn(move || {
        Python::with_gil(|py| {
            // Uh oh, moved Rc to a new thread!
            let c = obj.bind(py).downcast::<NotThreadSafe>().unwrap();

            assert_eq!(*c.borrow().data, 5);
        })
    })
    .join()
    .unwrap();
}
