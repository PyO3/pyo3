#![deny(deprecated)]

use pyo3::prelude::*;

#[pyclass]
#[text_signature = "()"]
struct TestClass {
    num: u32,
}

#[pymethods]
impl TestClass {
    #[classattr]
    #[name = "num"]
    const DEPRECATED_NAME_CONSTANT: i32 = 0;

    #[name = "num"]
    #[text_signature = "()"]
    fn deprecated_name_pymethod(&self) { }

    #[staticmethod]
    #[name = "custom_static"]
    #[text_signature = "()"]
    fn deprecated_name_staticmethod() {}
}

#[pyclass]
struct DeprecatedCall;

#[pymethods]
impl DeprecatedCall {
    #[call]
    fn deprecated_call(&self) {}
}

#[pyfunction]
#[name = "foo"]
#[text_signature = "()"]
fn deprecated_name_pyfunction() { }

#[pymodule(deprecated_module_name)]
fn my_module(_py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfn(m, "some_name")]
    #[text_signature = "()"]
    fn deprecated_name_pyfn() { }

    Ok(())
}

fn main() {

}


// TODO: ensure name deprecated on #[pyfunction] and #[pymodule]
