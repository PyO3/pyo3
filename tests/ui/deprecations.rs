#![deny(deprecated)]
#![allow(dead_code)]

use pyo3::prelude::*;

#[pyclass]
struct MyClass;

#[pymethods]
impl MyClass {
    #[__new__]
    fn new() -> Self {
        Self
    }
}

fn main() {}

fn extract_options(obj: &Bound<'_, PyAny>) -> PyResult<Option<i32>> {
    obj.extract()
}

#[pyfunction]
#[pyo3(signature = (_i, _any=None))]
fn pyfunction_option_1(_i: u32, _any: Option<i32>) {}

#[pyfunction]
fn pyfunction_option_2(_i: u32, _any: Option<i32>) {}

#[pyfunction]
fn pyfunction_option_3(_i: u32, _any: Option<i32>, _foo: Option<String>) {}

#[pyfunction]
fn pyfunction_option_4(
    _i: u32,
    #[pyo3(from_py_with = "extract_options")] _any: Option<i32>,
    _foo: Option<String>,
) {
}

#[pyclass]
pub enum SimpleEnumWithoutEq {
    VariamtA,
    VariantB,
}
