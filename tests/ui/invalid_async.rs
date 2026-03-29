use pyo3::prelude::*;

#[pyfunction]
async fn check(){}
//~^ ERROR: async functions are only supported with the `experimental-async` feature

#[pyclass]
pub(crate) struct AsyncRange {
    count: i32,
    target: i32,
}
#[pymethods]
impl AsyncRange {
    async fn __anext__(mut _pyself: PyRefMut<'_, Self>) -> PyResult<i32> {
//~^ ERROR: async functions are only supported with the `experimental-async` feature
        Ok(0)
    }

    async fn foo(&self) {}
//~^ ERROR: async functions are only supported with the `experimental-async` feature
}

fn main() {}
