//@normalize-stderr-test: ".*/src/rust/(.*)" -> "../src/$1"
use pyo3::prelude::*;

fn test_not_send_detach(py: Python<'_>) {
    py.detach(|| drop(py));
    //~^ ERROR: `*mut pyo3::Python<'static>` cannot be shared between threads safely
}

fn main() {
    Python::attach(|py| {
        test_not_send_detach(py);
    })
}
