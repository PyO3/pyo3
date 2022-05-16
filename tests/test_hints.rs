use pyo3::prelude::*;
use pyo3::types::PyType;

mod common;

// This implementation has a text signature and multiple methods
#[pyclass(name = "RustNumber", text_signature = "(value, /)", type_signature = "(int) -> None")]
struct CustomNumber(usize);

// This implementation has documentation
/// It's basically just a number.
///
/// There are multiple documentation lines here.
#[pyclass]
struct CustomNumber2 {
    #[pyo3(get, set, name = "value", type_signature = "int")]
    inner: usize,
}

// This implementation is simply empty, with no documentation, methods nor fields
#[pyclass]
struct CustomNumber3(usize);

/// There's documentation here too.
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

    /// This is documented.
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

    /// Converts a `float` into a `CustomNumber`.
    ///
    /// :param value: The value we want to convert
    /// :return: The result of the conversion
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

    fn to_3(&self) -> CustomNumber3 {
        CustomNumber3(self.0)
    }

    #[args(n = "None")]
    fn next(&self, n: Option<usize>) -> Vec<Self> {
        todo!()
    }
}
