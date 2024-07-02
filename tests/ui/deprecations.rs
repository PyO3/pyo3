#![deny(deprecated)]
#![allow(dead_code)]

use pyo3::prelude::*;
use pyo3::types::{PyString, PyType};

#[pyclass]
struct MyClass;

#[pymethods]
impl MyClass {
    #[__new__]
    fn new() -> Self {
        Self
    }

    #[classmethod]
    fn cls_method_gil_ref(_cls: &PyType) {}

    #[classmethod]
    fn cls_method_bound(_cls: &Bound<'_, PyType>) {}

    fn method_gil_ref(_slf: &PyCell<Self>) {}

    fn method_bound(_slf: &Bound<'_, Self>) {}

    #[staticmethod]
    fn static_method_gil_ref(_any: &PyAny) {}

    #[setter]
    fn set_foo_gil_ref(&self, #[pyo3(from_py_with = "extract_gil_ref")] _value: i32) {}

    #[setter]
    fn set_foo_bound(&self, #[pyo3(from_py_with = "extract_bound")] _value: i32) {}

    #[setter]
    fn set_bar_gil_ref(&self, _value: &PyAny) {}

    #[setter]
    fn set_bar_bound(&self, _value: &Bound<'_, PyAny>) {}

    fn __eq__(&self, #[pyo3(from_py_with = "extract_gil_ref")] _other: i32) -> bool {
        true
    }

    fn __contains__(&self, #[pyo3(from_py_with = "extract_bound")] _value: i32) -> bool {
        true
    }
}

fn main() {}

#[pyfunction]
#[pyo3(pass_module)]
fn pyfunction_with_module<'py>(module: &Bound<'py, PyModule>) -> PyResult<Bound<'py, PyString>> {
    module.name()
}

#[pyfunction]
#[pyo3(pass_module)]
fn pyfunction_with_module_gil_ref(_module: &PyModule) -> PyResult<&str> {
    todo!()
}

#[pyfunction]
fn double(x: usize) -> usize {
    x * 2
}

#[pymodule]
fn module_gil_ref(_m: &PyModule) -> PyResult<()> {
    Ok(())
}

#[pymodule]
fn module_gil_ref_with_explicit_py_arg(_py: Python<'_>, _m: &PyModule) -> PyResult<()> {
    Ok(())
}

#[pymodule]
fn module_bound(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(double, m)?)?;
    Ok(())
}

#[pymodule]
fn module_bound_with_explicit_py_arg(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(double, m)?)?;
    Ok(())
}

#[pymodule]
fn module_bound_by_value(m: Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(double, &m)?)?;
    Ok(())
}

fn extract_gil_ref(obj: &PyAny) -> PyResult<i32> {
    obj.extract()
}

fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<i32> {
    obj.extract()
}

fn extract_options(obj: &Bound<'_, PyAny>) -> PyResult<Option<i32>> {
    obj.extract()
}

#[pyfunction]
fn pyfunction_from_py_with(
    #[pyo3(from_py_with = "extract_gil_ref")] _gil_ref: i32,
    #[pyo3(from_py_with = "extract_bound")] _bound: i32,
) {
}

#[pyfunction]
fn pyfunction_gil_ref(_any: &PyAny) {}

#[pyfunction]
#[pyo3(signature = (_any))]
fn pyfunction_option_gil_ref(_any: Option<&PyAny>) {}

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

#[derive(Debug, FromPyObject)]
pub struct Zap {
    #[pyo3(item)]
    name: String,

    #[pyo3(from_py_with = "PyAny::len", item("my_object"))]
    some_object_length: usize,

    #[pyo3(from_py_with = "extract_bound")]
    some_number: i32,
}

#[derive(Debug, FromPyObject)]
pub struct ZapTuple(
    String,
    #[pyo3(from_py_with = "PyAny::len")] usize,
    #[pyo3(from_py_with = "extract_bound")] i32,
);

#[derive(Debug, FromPyObject, PartialEq, Eq)]
pub enum ZapEnum {
    Zip(#[pyo3(from_py_with = "extract_gil_ref")] i32),
    Zap(String, #[pyo3(from_py_with = "extract_bound")] i32),
}

#[derive(Debug, FromPyObject, PartialEq, Eq)]
#[pyo3(transparent)]
pub struct TransparentFromPyWithGilRef {
    #[pyo3(from_py_with = "extract_gil_ref")]
    len: i32,
}

#[derive(Debug, FromPyObject, PartialEq, Eq)]
#[pyo3(transparent)]
pub struct TransparentFromPyWithBound {
    #[pyo3(from_py_with = "extract_bound")]
    len: i32,
}

fn test_wrap_pyfunction(py: Python<'_>, m: &Bound<'_, PyModule>) {
    // should lint
    let _ = wrap_pyfunction!(double, py);

    // should lint but currently does not
    let _ = wrap_pyfunction!(double)(py);

    // should not lint
    let _ = wrap_pyfunction!(double, m);
    let _ = wrap_pyfunction!(double)(m);
    let _ = wrap_pyfunction!(double, m.as_gil_ref());
    let _ = wrap_pyfunction!(double)(m.as_gil_ref());
    let _ = wrap_pyfunction_bound!(double, py);
    let _ = wrap_pyfunction_bound!(double)(py);
}

#[pyclass]
pub enum SimpleEnumWithoutEq {
    VariamtA,
    VariantB,
}
