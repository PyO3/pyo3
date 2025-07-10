use pyo3::prelude::*;

fn main() {
    Python::attach(|py| {
        let any = py.None().into_bound(py);
        any.call1("foo");
    })
}
