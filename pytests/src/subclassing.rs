//! Test for [#220](https://github.com/PyO3/pyo3/issues/220)

use pyo3::prelude::*;

#[pymodule(gil_used = false)]
pub mod subclassing {
    use pyo3::prelude::*;

    #[pyclass(subclass)]
    pub struct Subclassable {}

    #[pymethods]
    impl Subclassable {
        #[new]
        fn new() -> Self {
            Subclassable {}
        }

        fn __str__(&self) -> &'static str {
            "Subclassable"
        }
    }
}
