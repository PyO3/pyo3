use pyo3::prelude::*;

#[pyclass]
struct SubmoduleClass {}

#[pymethods]
impl SubmoduleClass {
    #[new]
    pub fn __new__() -> Self {
        SubmoduleClass {}
    }

    pub fn greeting(&self) -> &'static str {
        "Hello, world!"
    }
}

#[pymodule]
pub fn submodule(m: &Bound<'_, PyModule>) -> PyResult<()> {
    PyModule::add_class::<SubmoduleClass>(m)?;
    Ok(())
}
