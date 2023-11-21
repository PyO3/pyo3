use pyo3::prelude::*;
use pyo3::types::PyString;

fn main() {
    Python::with_gil(|py| {
        let string = PyString::new_bound(py, "foo");

        py.allow_threads(|| {
            println!("{:?}", string);
        });
    });
}
