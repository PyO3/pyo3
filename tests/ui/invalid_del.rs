//! Check that invalid `__del__` (tp_finalize) signatures error as expected.
//!
//! This is a separate file from `invalid_proto_pymethods.rs` because `__del__`
//! is not available on abi3 before Python 3.15, which would add extra errors.

use pyo3::prelude::*;

#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
    #[pyo3(name = "__del__")]
    fn del_expects_no_arguments(&mut self, _extra: i32) {}
    //~^ ERROR: Expected at most 0 non-python arguments
}

#[pymethods]
impl MyClass {
    #[staticmethod]
    #[pyo3(name = "__del__")]
    fn del_must_be_instance_method() {}
    //~^ ERROR: expected instance method for `__del__` function
}

fn main() {}
