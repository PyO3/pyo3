use pyo3::prelude::*;

#[pyclass]
struct ClassWithGetter {}

#[pymethods]
impl ClassWithGetter {
    #[getter]
    fn getter_with_arg(&self, py: Python, index: u32) {}
}

#[pyclass]
struct ClassWithSetter {}

#[pymethods]
impl ClassWithSetter {
    #[setter]
    fn setter_with_no_arg(&mut self, py: Python) {}
}

#[pymethods]
impl ClassWithSetter {
    #[setter]
    fn setter_with_too_many_args(&mut self, py: Python, foo: u32, bar: u32) {}
}

fn main() {}
