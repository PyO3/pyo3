//! Test for [#220](https://github.com/PyO3/pyo3/issues/220)

use pyo3::prelude::*;

#[pymodule(gil_used = false)]
pub mod subclassing {
    use pyo3::prelude::*;
    #[cfg(not(Py_LIMITED_API))]
    use pyo3::types::PyDict;

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

    #[pyclass(extends = Subclassable)]
    pub struct Subclass {}

    #[pymethods]
    impl Subclass {
        #[new]
        fn new() -> (Self, Subclassable) {
            (Subclass {}, Subclassable::new())
        }

        fn __str__(&self) -> &'static str {
            "Subclass"
        }
    }

    #[cfg(not(Py_LIMITED_API))]
    #[pyclass(extends = PyDict)]
    pub struct SubDict {}

    #[cfg(not(Py_LIMITED_API))]
    #[pymethods]
    impl SubDict {
        #[new]
        fn new() -> Self {
            Self {}
        }

        fn __str__(&self) -> &'static str {
            "SubDict"
        }
    }
}
