//! Test for [#220](https://github.com/PyO3/pyo3/issues/220)

use pyo3::prelude::*;

#[pyclass(subclass)]
pub struct Subclassable {}

#[pymethods]
impl Subclassable {
    #[new]
    fn __new__(obj: &PyRawObject) -> PyResult<()> {
        obj.init(|| Subclassable {})
    }
}

#[pymodule]
fn subclassing(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Subclassable>()?;
    Ok(())
}
