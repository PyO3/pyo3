use pyo3::prelude::*;

#[pyclass]
struct MyClass {}

// Deliberately a local type with no `From` implementations, so that the error output
// doesn't include a candidate list which varies with the enabled features.
struct NotASelfType;

#[pymethods]
impl MyClass {
    fn method_with_invalid_self_type(_slf: NotASelfType, _py: Python<'_>, _index: u32) {}
    //~^ ERROR: the trait bound `NotASelfType: From<&pyo3::Bound<'_, MyClass>>` is not satisfied
}

fn main() {}
