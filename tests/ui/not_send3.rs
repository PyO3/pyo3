use pyo3::prelude::*;
use std::rc::Rc;

fn main() {
    Python::with_gil(|py| {
        let rc = Rc::new(5);

        py.allow_threads(|| {
            println!("{:?}", rc);
        });
    });
}