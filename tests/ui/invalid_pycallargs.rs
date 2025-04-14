use pyo3::prelude::*;

fn main() {
    Python::with_gil(|py| {
        let any = py.None().into_bound(py);
        any.call1("foo");
    })
}
