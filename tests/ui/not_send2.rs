use pyo3::prelude::*;
use pyo3::types::PyString;

fn main() {
    Python::attach(|py| {
        let string = PyString::new(py, "foo");

        py.detach(|| {
//~^ ERROR: `*mut pyo3::Python<'static>` cannot be shared between threads safely
            println!("{:?}", string);
        });
    });
}
