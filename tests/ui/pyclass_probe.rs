#![deny(unused_imports)]
use pyo3::prelude::*;

#[pyclass]
pub struct Probe {}

#[pymethods]
impl Probe {
    #[new]
    fn new() -> Self {
        Self {}
    }
}

#[pymodule]
fn probe(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Probe>()?;
    Ok(())
}

#[pyclass]
struct Check5029();

macro_rules! impl_methods {
    ($name:ident) => {
        #[pymethods]
        impl Check5029 {
            fn $name(&self, _value: Option<&str>) -> PyResult<()> {
                Ok(())
            }
        }
    };
}

impl_methods!(some_method);

fn main() {}
