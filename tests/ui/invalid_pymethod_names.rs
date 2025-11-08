use pyo3::prelude::*;

#[pyclass]
struct TestClass {
    num: u32,
}

#[pymethods]
impl TestClass {
    #[pyo3(name = "num")]
    #[getter(number)]
    fn get_num(&self) -> u32 { self.num }
}

#[pymethods]
impl TestClass {
    #[pyo3(name = "foo")]
    #[pyo3(name = "bar")]
    fn qux(&self) -> u32 { self.num }
}

#[pymethods]
impl TestClass {
    #[pyo3(name = "makenew")]
    #[new]
    fn new(&self) -> Self { Self { num: 0 } }
}

#[pymethods]
impl TestClass {
    #[getter(1)]
    fn get_one(&self) -> Self { Self { num: 0 } }
}

#[pymethods]
impl TestClass {
    #[getter = 1]
    fn get_two(&self) -> Self { Self { num: 0 } }
}


fn main() {}
