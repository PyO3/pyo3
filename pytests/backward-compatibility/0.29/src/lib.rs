use pyo3::prelude::*;

/// Some module
#[pymodule]
mod pyo3_backward_compatibility_029 {
    use pyo3::prelude::*;
    use pyo3::types::{PyDict, PyTuple, PyType};
    use std::collections::HashMap;
    use std::path::PathBuf;

    /// Some const
    #[pymodule_export]
    const CONST: usize = 0;

    /// Some function
    #[pyfunction]
    #[pyo3(signature = (_arg1, /, _arg2: "int", *_args, _foo = None, **_kwargs))]
    fn some_fn(
        _arg1: (usize, Vec<PathBuf>, HashMap<String, usize>),
        _arg2: Bound<'_, PyAny>,
        _args: Bound<'_, PyTuple>,
        _foo: Option<&str>,
        _kwargs: Option<Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        Ok(())
    }

    /// Some class
    #[pyclass(eq, ord, extends = PyDict)]
    #[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
    struct MyClass {
        // TODO: parent class
        value: usize,
    }

    #[pymethods]
    impl MyClass {
        #[expect(dead_code)]
        const PI: Self = Self { value: 3 };

        #[new]
        fn new(value: usize) -> Self {
            Self { value }
        }

        #[getter]
        fn value(&self) -> usize {
            self.value
        }

        #[setter]
        fn set_value(&mut self, value: usize) {
            self.value = value;
        }

        #[deleter]
        fn delete_value(&self) {}

        #[staticmethod]
        fn static_method() -> bool {
            true
        }

        #[classmethod]
        fn class_method(_cls: Bound<'_, PyType>) -> &'static str {
            "foo"
        }

        #[classattr]
        fn class_attr() -> f32 {
            0.
        }
    }

    #[pymodule]
    mod submodule {
        use super::*;

        #[pyclass(subclass)]
        struct Class2 {}
    }

    #[pymodule_init]
    fn init(_m: &Bound<'_, PyModule>) -> PyResult<()> {
        Ok(())
    }
}
