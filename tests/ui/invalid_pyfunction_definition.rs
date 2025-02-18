#![feature(arbitrary_self_types)]
#[pyo3::pymodule]
mod pyo3_scratch {
    use pyo3::prelude::*;

    #[pyclass]
    struct Foo {}

    #[pymethods]
    impl Foo {
        #[pyfunction]
        fn bug() {}
    }
}

fn main() {}
