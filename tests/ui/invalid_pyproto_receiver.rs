use pyo3::prelude::*;
use pyo3::class::basic::PyObjectProtocol;
use pyo3::class::mapping::PyMappingProtocol;

#[pyclass]
struct MyClass {}

// Before PyO3 0.14 #[pyproto] allowed receivers. Now only `TryFromPyCell` types (e.g.
// `PyRef<Self>` etc.) are allowed.

#[pyproto]
impl PyObjectProtocol for MyClass {
    fn __str__(&self) -> &'static str {  "hello, world" }
}

#[pyproto]
impl PyMappingProtocol for MyClass {
    fn __delitem__(&mut self, item: &PyAny) { }
}

fn main() {}
