use pyo3::prelude::*;

#[pyfunction]
async fn check(){}

#[pyclass]
pub(crate) struct AsyncRange {
    count: i32,
    target: i32,
}
#[pymethods]
impl AsyncRange {
    async fn __anext__(mut _pyself: PyRefMut<'_, Self>) -> PyResult<i32> {
        Ok(0)
    }

    async fn foo(&self) {}
}

fn main() {}