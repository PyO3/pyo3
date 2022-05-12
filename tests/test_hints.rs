use pyo3::prelude::*;
use pyo3::types::PyType;

mod common;

#[pyclass(text_signature = "(value, /)", type_signature = "(int) -> None")]
struct CustomNumber(usize);

#[pyclass]
struct CustomNumber2 {
    #[pyo3(get, set, name = "value", type_signature = "int")]
    inner: usize,
}

#[pyfunction]
#[pyo3(type_signature = "(float) -> CustomNumber")]
fn number_from_float(input: f64) -> CustomNumber {
    CustomNumber::from_double(input)
}

#[pymethods]
impl CustomNumber {
    #[new]
    fn new(value: usize) -> Self {
        Self(value)
    }

    #[pyo3(text_signature = "(new, /)", type_signature = "(int) -> None")]
    fn set(&mut self, new: usize) {
        self.0 = new
    }

    #[pyo3(type_signature = "(int) -> CustomNumber")]
    fn __add__(&mut self, other: usize) -> Self {
        Self(self.0 + other)
    }

    #[getter(value)]
    #[pyo3(type_signature = "() -> int")]
    fn get_value(&self) -> usize {
        self.0
    }

    #[setter(value)]
    #[pyo3(type_signature = "(int) -> None")]
    fn set_value(&mut self, new: usize) {
        self.0 = new
    }

    #[classmethod]
    #[pyo3(text_signature = "(value, /)", type_signature = "(float) -> CustomNumber")]
    fn from_float(_cls: &PyType, value: f32) -> Self {
        Self(value as usize)
    }

    #[staticmethod]
    #[pyo3(text_signature = "(value, /)", type_signature = "(float) -> CustomNumber")]
    fn from_double(value: f64) -> Self {
        Self(value as usize)
    }
}
