use pyo3::prelude::*;

#[pyclass]
struct ClassWithGetter {}

#[pymethods]
impl ClassWithGetter {
    #[getter]
    fn getter_with_arg(&self, py: Python<'_>, index: u32) {}
}

#[pyclass]
struct ClassWithSetter {}

#[pymethods]
impl ClassWithSetter {
    #[setter]
    fn setter_with_no_arg(&mut self, py: Python<'_>) {}
}

#[pymethods]
impl ClassWithSetter {
    #[setter]
    fn setter_with_too_many_args(&mut self, py: Python<'_>, foo: u32, bar: u32) {}
}

#[pyclass]
struct TupleGetterSetterNoName(#[pyo3(get, set)] i32);

#[pyclass]
struct MultipleGet(#[pyo3(get, get)] i32);

#[pyclass]
struct MultipleSet(#[pyo3(set, set)] i32);

#[pyclass]
struct MultipleName(#[pyo3(name = "foo", name = "bar")] i32);

#[pyclass]
struct NameWithoutGetSet(#[pyo3(name = "value")] i32);

fn main() {}
