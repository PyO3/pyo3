use pyo3::prelude::*;

#[pyclass]
struct ClassWithGetter {
    a: u32,
}

#[pymethods]
impl ClassWithGetter {
    #[getter]
    fn get_num(&self, index: u32) {}
}

fn main() {}
