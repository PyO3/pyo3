use pyo3::prelude::*;
use pyo3::types::PyType;

#[pyclass(generic)]
struct ClassRedefinesClassGetItem {}

#[pymethods]
impl ClassRedefinesClassGetItem {
    #[new]
    fn new() -> ClassRedefinesClassGetItem {
        Self {}
    }

    #[classmethod]
    pub fn __class_getitem__(
        cls: &Bound<'_, PyType>,
        key: &Bound<'_, PyAny>,
    ) -> PyResult<Py<PyAny>> {
        pyo3::types::PyGenericAlias::new(cls.py(), cls.as_any(), key)
    }
}

fn main() {}
