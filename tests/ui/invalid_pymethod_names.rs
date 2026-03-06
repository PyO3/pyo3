use pyo3::prelude::*;

#[pyclass]
struct TestClass {
    num: u32,
}

#[pymethods]
impl TestClass {
    #[pyo3(name = "num")]
    #[getter(number)]
//~^ ERROR: `name` may only be specified once
    fn get_num(&self) -> u32 { self.num }
}

#[pymethods]
impl TestClass {
    #[pyo3(name = "foo")]
    #[pyo3(name = "bar")]
//~^ ERROR: `name` may only be specified once
    fn qux(&self) -> u32 { self.num }
}

#[pymethods]
impl TestClass {
    #[pyo3(name = "makenew")]
//~^ ERROR: `name` not allowed with `#[new]`
    #[new]
    fn new(&self) -> Self { Self { num: 0 } }
}

#[pymethods]
impl TestClass {
    #[getter(1)]
//~^ ERROR: expected ident or string literal for property name
    fn get_one(&self) -> Self { Self { num: 0 } }
}

#[pymethods]
impl TestClass {
    #[getter = 1]
//~^ ERROR: expected `#[getter(name)]` to set the name
    fn get_two(&self) -> Self { Self { num: 0 } }
}


fn main() {}
