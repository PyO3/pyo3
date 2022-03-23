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
pub fn submodule(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<SubmoduleClass>()?;
    Ok(())
}
