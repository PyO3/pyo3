use pyo3::prelude::*;

#[pyclass]
struct TestClass {
    num: u32,
}

#[pymethods]
impl TestClass {
    #[name = "num"]
    #[getter(number)]
    fn get_num(&self) -> u32 { self.num }
}

#[pymethods]
impl TestClass {
    #[name = "foo"]
    #[name = "bar"]
    fn qux(&self) -> u32 { self.num }
}

#[pymethods]
impl TestClass {
    #[name = "makenew"]
    #[new]
    fn new(&self) -> Self { Self { num: 0 } }
}

fn main() {}
