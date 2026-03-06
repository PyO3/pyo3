#[pyo3::pymodule]
mod pyo3_scratch {
    use pyo3::prelude::*;

    #[pyclass]
    struct Foo {}

    #[pymethods]
    impl Foo {
        #[pyfunction]
        fn bug() {}
//~^ ERROR: functions inside #[pymethods] do not need to be annotated with #[pyfunction]
    }
}

fn main() {}
